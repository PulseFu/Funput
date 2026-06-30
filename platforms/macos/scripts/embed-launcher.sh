#!/bin/sh
# Xcode "Embed Launcher" build phase. Builds the launcher stub and embeds it in the
# app's Resources, so the input method can self-install it into ~/Applications on
# first launch (see Funput/LauncherInstaller.swift). Runs before Xcode's final code
# signing, so the embedded launcher is sealed into the notarizable app bundle.
#
# The launcher exists only so users can find "Funput" in Spotlight; the input
# method itself lives in (~)/Library/Input Methods, which Spotlight does not
# surface. See scripts/build-launcher.sh.
set -eu

# Nothing to embed for index/preview builds (no real product is produced).
if [ "${ACTION:-build}" = "indexbuild" ]; then
    exit 0
fi

SCRIPTS="$SRCROOT/scripts"
DEST_RESOURCES="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH"
BUILD_DIR="$DERIVED_FILE_DIR/launcher"

# Sign the launcher with the same identity Xcode is using for this build, so it is
# valid nested code. With code signing disabled (e.g. CI dry runs) fall back to
# ad-hoc; Xcode re-signs the whole product afterwards anyway.
if [ "${CODE_SIGNING_ALLOWED:-YES}" = "NO" ] || [ -z "${EXPANDED_CODE_SIGN_IDENTITY_NAME:-}" ]; then
    LAUNCHER_SIGN_ID="-"
else
    LAUNCHER_SIGN_ID="$EXPANDED_CODE_SIGN_IDENTITY_NAME"
fi

"$SCRIPTS/build-launcher.sh" "$BUILD_DIR" "${MARKETING_VERSION:-0.0.0}" "$LAUNCHER_SIGN_ID" >/dev/null

mkdir -p "$DEST_RESOURCES"
rm -rf "$DEST_RESOURCES/Funput.app"
cp -R "$BUILD_DIR/Funput.app" "$DEST_RESOURCES/Funput.app"
