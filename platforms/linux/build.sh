#!/usr/bin/env bash
# Build the Funput Linux package(s): Rust core + Settings UI + input-method
# shell(s), bundled into a .deb (Debian/Ubuntu) or .rpm (Fedora/openSUSE). Run on
# the matching distro with the build deps installed (see platforms/linux/README.md).
#
# Usage: platforms/linux/build.sh [build-dir]
#   FUNPUT_FRAMEWORK=fcitx5|ibus|all   which shell(s) to package (default: all)
#   FUNPUT_PKG=deb|rpm                 package format (default: auto from host)
#
# Each shell builds into <build-dir>/<framework>/ and yields its own package:
# fcitx5 → funput_<ver>_<arch>.deb, ibus → funput-ibus_<ver>_<arch>.deb
# (.rpm on Fedora/openSUSE: funput-<ver>.<arch>.rpm / funput-ibus-<ver>.<arch>.rpm).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "${HERE}/../.." && pwd)"
BUILD_DIR="${1:-${HERE}/build}"
FRAMEWORK="${FUNPUT_FRAMEWORK:-all}"

# Package format: explicit override, else pick by what's available on the host —
# dpkg → .deb (Debian/Ubuntu), rpmbuild → .rpm (Fedora/openSUSE).
PKG="${FUNPUT_PKG:-}"
if [ -z "${PKG}" ]; then
    if command -v dpkg >/dev/null 2>&1; then PKG="deb"
    elif command -v rpmbuild >/dev/null 2>&1; then PKG="rpm"
    else echo "No dpkg or rpmbuild found; set FUNPUT_PKG=deb|rpm." >&2; exit 1
    fi
fi
CPACK_GEN="$(echo "${PKG}" | tr '[:lower:]' '[:upper:]')"  # deb→DEB, rpm→RPM

# Shared steps (run once, reused by every shell).
echo "==> [1/2] Rust core (funput-ffi cdylib)"
cargo build --release -p funput-ffi --manifest-path "${APP_ROOT}/Cargo.toml"

echo "==> [2/2] Settings app (GTK4 + libadwaita)"
# Native GTK4/libadwaita GUI (needs libgtk-4-dev + libadwaita-1-dev). Its own crate,
# excluded from the root workspace so macOS/Windows `cargo test --workspace` stays green.
cargo build --release --manifest-path "${HERE}/settings-gtk/Cargo.toml"

# Build one shell into its own subdir and produce a package via CPack.
build_shell() {
    local name="$1" out="${BUILD_DIR}/$1"
    echo "==> ${name} shell + .${PKG} (CMake/CPack)"
    cmake -S "${HERE}/${name}" -B "${out}" -DCMAKE_BUILD_TYPE=Release
    cmake --build "${out}" --parallel
    ( cd "${out}" && cpack -G "${CPACK_GEN}" )
}

case "${FRAMEWORK}" in
    fcitx5) build_shell fcitx5 ;;
    ibus)   build_shell ibus ;;
    all)    build_shell fcitx5; build_shell ibus ;;
    *) echo "Unknown FUNPUT_FRAMEWORK='${FRAMEWORK}' (want fcitx5|ibus|all)" >&2; exit 1 ;;
esac

echo "==> Done. Package(s):"
find "${BUILD_DIR}" -maxdepth 2 -name "*.${PKG}"
