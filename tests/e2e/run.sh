#!/bin/sh
# Build the frontends, then run the full e2e suite under cargo-nextest:
# parallel, one process per test (each test owns its compositor/X server),
# wedged tests killed per .config/nextest.toml.
set -eu
cd "$(dirname "$0")/../.."
command -v cargo-nextest >/dev/null 2>&1 || {
    echo "cargo-nextest not found — install it (Arch: pacman -S cargo-nextest;" >&2
    echo "other: https://nexte.st), or run the slow serial fallback yourself:" >&2
    echo "  cargo test -p kime-e2e -- --ignored --test-threads=1" >&2
    exit 1
}
ninja -C build
exec cargo nextest run -p kime-e2e --run-ignored ignored-only
