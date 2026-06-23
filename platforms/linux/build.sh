#!/usr/bin/env bash
# Build the Funput Linux package(s): Rust core + Settings UI + input-method
# shell(s), bundled into .deb(s). Run on Debian/Ubuntu with the build deps
# installed (see platforms/linux/README.md).
#
# Usage: platforms/linux/build.sh [build-dir]
#   FUNPUT_FRAMEWORK=fcitx5|ibus|all   which shell(s) to package (default: all)
#
# Each shell builds into <build-dir>/<framework>/ and yields its own .deb:
# fcitx5 → funput_<ver>_<arch>.deb, ibus → funput-ibus_<ver>_<arch>.deb.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "${HERE}/../.." && pwd)"
BUILD_DIR="${1:-${HERE}/build}"
FRAMEWORK="${FUNPUT_FRAMEWORK:-all}"

# Shared steps (run once, reused by every shell).
echo "==> [1/3] Rust core (funput-ffi cdylib)"
cargo build --release -p funput-ffi --manifest-path "${APP_ROOT}/Cargo.toml"

echo "==> [2/3] Settings UI (Svelte → dist)"
pnpm --dir "${APP_ROOT}/platforms/ui" install --frozen-lockfile
pnpm --dir "${APP_ROOT}/platforms/ui" build

echo "==> [3/3] Settings app (Tauri)"
# --locked: build from the committed src-tauri Cargo.lock (reproducible, no re-resolve).
cargo build --release --locked --manifest-path "${HERE}/src-tauri/Cargo.toml"

# Build one shell into its own subdir and produce a .deb via CPack.
build_shell() {
    local name="$1" out="${BUILD_DIR}/$1"
    echo "==> ${name} shell + .deb (CMake/CPack)"
    cmake -S "${HERE}/${name}" -B "${out}" -DCMAKE_BUILD_TYPE=Release
    cmake --build "${out}" --parallel
    ( cd "${out}" && cpack -G DEB )
}

case "${FRAMEWORK}" in
    fcitx5) build_shell fcitx5 ;;
    ibus)   build_shell ibus ;;
    all)    build_shell fcitx5; build_shell ibus ;;
    *) echo "Unknown FUNPUT_FRAMEWORK='${FRAMEWORK}' (want fcitx5|ibus|all)" >&2; exit 1 ;;
esac

echo "==> Done. Package(s):"
find "${BUILD_DIR}" -maxdepth 2 -name '*.deb'
