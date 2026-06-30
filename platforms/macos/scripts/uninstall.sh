#!/bin/sh
# Uninstall Funput and reset its state so the next install behaves like a
# brand-new machine (onboarding shows again, no leftover input source or prefs).
#
#   ./scripts/uninstall.sh              # remove everything, including preferences
#   ./scripts/uninstall.sh --keep-prefs # remove the app(s) but keep settings
#
# Run as your normal user — NOT `sudo ./uninstall.sh`. The script invokes sudo only
# for the system-wide locations (/Library, /Applications, the pkg receipt); running
# the whole thing as root would wipe root's preferences instead of yours.
#
# Order: first remove Funput in System Settings -> Keyboard -> Input Sources
# (select Funput, press the - button), then run this, then log out/in or reboot so
# the Text Input Sources database forgets the input source completely.
#
# Best-effort by design (no `set -e`): every step is guarded so a partial install
# still cleans up. `set -u` only, to catch unset-variable typos.
set -u

KEEP_PREFS=
case "${1:-}" in
    --keep-prefs) KEEP_PREFS=1 ;;
    "") ;;
    *) echo "usage: $0 [--keep-prefs]" >&2; exit 2 ;;
esac

LSR="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

# Every place Funput can live: system-wide (.pkg) and per-user (install.sh) input
# method bundles, plus the /Applications launcher stub.
SYSTEM_IME="/Library/Input Methods/Funput.app"
USER_IME="$HOME/Library/Input Methods/Funput.app"
LAUNCHER="/Applications/Funput.app"

echo "Stopping Funput…"
killall Funput 2>/dev/null || true

# Unregister from LaunchServices before deleting, so no dangling records remain.
for app in "$SYSTEM_IME" "$USER_IME" "$LAUNCHER"; do
    [ -e "$app" ] && "$LSR" -u "$app" 2>/dev/null
done

# Dev machines accumulate DerivedData build copies that LaunchServices (and so
# Spotlight) still know about — unregister any so a "Funput" search on this machine
# doesn't resurface a stale build. (Only de-registers; build files stay on disk.)
"$LSR" -dump 2>/dev/null | grep -oE '/Users/[^ ]*DerivedData[^ ]*Funput.app' | sort -u \
    | while read -r p; do "$LSR" -u "$p" 2>/dev/null || true; done

# Per-user bundle: no privileges needed.
if [ -e "$USER_IME" ]; then
    echo "Removing $USER_IME"
    rm -rf "$USER_IME"
fi

# System-wide bundle + launcher + pkg receipt need sudo. Only escalate when there
# is actually something there, so a per-user-only setup never prompts for sudo.
NEED_SUDO=
[ -e "$SYSTEM_IME" ] && NEED_SUDO=1
[ -e "$LAUNCHER" ] && NEED_SUDO=1
pkgutil --pkg-info com.funput.installer >/dev/null 2>&1 && NEED_SUDO=1
if [ -n "$NEED_SUDO" ]; then
    echo "Removing system-wide files (sudo)…"
    [ -e "$SYSTEM_IME" ] && sudo rm -rf "$SYSTEM_IME"
    [ -e "$LAUNCHER" ] && sudo rm -rf "$LAUNCHER"
    sudo pkgutil --forget com.funput.installer 2>/dev/null || true
fi

# Preferences (skipped with --keep-prefs). `defaults delete` goes through cfprefsd
# so its in-memory copy is dropped too; also remove the plist in case it lingers.
if [ -z "$KEEP_PREFS" ]; then
    echo "Removing preferences…"
    defaults delete app.funput.inputmethod.Funput 2>/dev/null || true
    defaults delete app.funput.Funput 2>/dev/null || true
    rm -f "$HOME/Library/Preferences/app.funput.inputmethod.Funput.plist"
    rm -f "$HOME/Library/Preferences/app.funput.Funput.plist"
fi

# Caches and saved window state — always safe to clear.
echo "Removing caches…"
rm -rf "$HOME/Library/Caches/app.funput.inputmethod.Funput"
rm -rf "$HOME/Library/Saved Application State/app.funput.inputmethod.Funput.savedState"

# Make cfprefsd drop any remaining in-memory copy of the deleted preferences.
[ -z "$KEEP_PREFS" ] && killall cfprefsd 2>/dev/null

echo ""
echo "Done. Log out and back in (or reboot) so the input source is fully cleared."
