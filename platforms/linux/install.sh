#!/usr/bin/env bash
# Funput Linux installer: detect the distro + arch, pick the matching package from
# the latest GitHub release, and install it with the native package manager.
#
# This is the "one command, auto-detect" front end over GitHub Releases — there is
# no hosted apt/dnf repo yet, so it downloads a versioned asset and installs it
# (no automatic upgrades; re-run this script to update). Apt and dnf are different
# repo formats, so a single repo can never serve every distro — only this kind of
# detect-then-fetch script gives the unified experience.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Funput/Funput/main/platforms/linux/install.sh | bash
#   ./install.sh [--ibus | --fcitx5] [--version vX.Y.Z]
#
# Framework: defaults to IBus, except KDE Plasma sessions default to Fcitx5.
# Override with --ibus / --fcitx5.
set -euo pipefail

REPO="Funput/Funput"
FRAMEWORK=""        # "ibus" | "fcitx5"; empty = auto-detect
VERSION="latest"    # "latest" or a tag like v1.2026.1

die() { echo "Error: $*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --ibus)    FRAMEWORK="ibus" ;;
    --fcitx5)  FRAMEWORK="fcitx5" ;;
    --version) shift; VERSION="${1:?--version needs a tag}" ;;
    -h|--help)
      sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) die "unknown argument: $1 (try --help)" ;;
  esac
  shift
done

# --- Distro family ---------------------------------------------------------
[ -r /etc/os-release ] || die "cannot read /etc/os-release; unsupported system"
# shellcheck disable=SC1091
. /etc/os-release
# Match on ID first, then ID_LIKE so derivatives (Linux Mint, Pop!_OS, Nobara,
# openSUSE variants, Manjaro/EndeavourOS) resolve to the right family.
case " ${ID:-} ${ID_LIKE:-} " in
  *" debian "*|*" ubuntu "*) FAMILY="debian" ;;
  *" fedora "*|*" rhel "*)   FAMILY="fedora" ;;
  *" suse "*|*" opensuse "*) FAMILY="suse" ;;
  *" arch "*)                FAMILY="arch" ;;
  *) die "unsupported distro (ID=${ID:-?}, ID_LIKE=${ID_LIKE:-?}). Build from source: platforms/linux/build.sh" ;;
esac

# Arch is community-packaged via the AUR, not a release asset.
if [ "$FAMILY" = "arch" ]; then
  cat >&2 <<'EOF'
Arch Linux is packaged through the AUR, not the GitHub release assets.
Install with an AUR helper, e.g.:
  yay -S funput-ibus     # IBus (GNOME)
  yay -S funput          # Fcitx5 (KDE / full features)
EOF
  exit 1
fi

# --- Framework -------------------------------------------------------------
if [ -z "$FRAMEWORK" ]; then
  case " ${XDG_CURRENT_DESKTOP:-} " in
    *KDE*|*plasma*|*Plasma*) FRAMEWORK="fcitx5" ;;
    *) FRAMEWORK="ibus" ;;
  esac
  echo "Auto-selected framework: $FRAMEWORK (override with --ibus / --fcitx5)"
fi

# --- Arch + package-name pattern -------------------------------------------
# .deb uses amd64/arm64; .rpm uses x86_64/aarch64. The CPack file names are:
#   deb  funput_<v>_<arch>.deb        funput-ibus_<v>_<arch>.deb
#   rpm  funput-<v>.<arch>.rpm        funput-ibus-<v>.<arch>.rpm
MACHINE="$(uname -m)"
if [ "$FAMILY" = "debian" ]; then
  FORMAT="deb"
  case "$MACHINE" in
    x86_64) PKG_ARCH="amd64" ;;
    aarch64|arm64) PKG_ARCH="arm64" ;;
    *) die "unsupported CPU arch: $MACHINE" ;;
  esac
  if [ "$FRAMEWORK" = "ibus" ]; then
    PATTERN="funput-ibus_[^/]*_${PKG_ARCH}\.deb"
  else
    PATTERN="funput_[^/]*_${PKG_ARCH}\.deb"
  fi
else
  # fedora + suse → .rpm
  FORMAT="rpm"
  case "$MACHINE" in
    x86_64) PKG_ARCH="x86_64" ;;
    aarch64|arm64) PKG_ARCH="aarch64" ;;
    *) die "unsupported CPU arch: $MACHINE" ;;
  esac
  if [ "$FRAMEWORK" = "ibus" ]; then
    PATTERN="funput-ibus-[^/]*\.${PKG_ARCH}\.rpm"
  else
    # funput- followed by a digit = version, to not match funput-ibus-*.
    PATTERN="funput-[0-9][^/]*\.${PKG_ARCH}\.rpm"
  fi
fi

# --- Resolve the download URL from the GitHub release ----------------------
have curl || die "curl is required"
if [ "$VERSION" = "latest" ]; then
  API="https://api.github.com/repos/${REPO}/releases/latest"
else
  API="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
fi

echo "Looking up ${FRAMEWORK} ${FORMAT} (${PKG_ARCH}) in ${REPO} ${VERSION} release…"
# Pull browser_download_url lines from the API JSON and pick the one matching the
# package pattern. Avoids a jq dependency on the user's machine.
URL="$(curl -fsSL "$API" \
  | grep -Eo '"browser_download_url": *"[^"]*"' \
  | sed -E 's/.*"(https[^"]*)".*/\1/' \
  | grep -E "/${PATTERN}$" \
  | head -n1 || true)"
[ -n "$URL" ] || die "no matching asset (${PATTERN}) in the ${VERSION} release"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FILE="$TMP/$(basename "$URL")"
echo "Downloading $(basename "$URL")…"
curl -fsSL "$URL" -o "$FILE"

# Verify the checksum if the release ships a sibling <file>.sha256.
if SUM="$(curl -fsSL "${URL}.sha256" 2>/dev/null)" && [ -n "$SUM" ]; then
  echo "$SUM  $FILE" | sha256sum -c - >/dev/null \
    && echo "Checksum OK." \
    || die "checksum mismatch — download corrupt, retry"
fi

# --- Install ---------------------------------------------------------------
SUDO=""
[ "$(id -u)" -eq 0 ] || SUDO="sudo"
echo "Installing $(basename "$FILE")…"
case "$FAMILY" in
  debian) $SUDO apt-get install -y "$FILE" ;;
  fedora) $SUDO dnf install -y "$FILE" ;;
  suse)   $SUDO zypper --non-interactive install --allow-unsigned-rpm "$FILE" ;;
esac

# --- Post-install hint -----------------------------------------------------
echo
echo "Installed. Next steps:"
if [ "$FRAMEWORK" = "ibus" ]; then
  echo "  1. ibus restart            # load the newly registered engine"
  echo "  2. Settings → Keyboard → Input Sources → + → Vietnamese → Funput"
else
  echo "  1. fcitx5-configtool       # + → add Funput (Vietnamese group)"
  echo "  2. log out/in if Fcitx5 was not already running"
fi
echo "  Open \"Funput\" from the app menu to switch Telex/VNI."
