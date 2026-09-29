#!/usr/bin/env bash
# Build the web-gallery artifact: the one recipe CI's `wasm` job and a
# developer both run.
#
#   nightly wasm32 build of crates/herogpui-web
#   -> wasm-bindgen (the CLI must match the crate in Cargo.lock)
#   -> binaryen wasm-opt (pinned version, see WASM_OPT_VERSION)
#   -> herogpui_web.js + herogpui_web_bg.wasm + index.html + build-info.json
#
# The artifact is not committed. The default output is web/public/gallery/,
# which is gitignored for these files: `pnpm run build` then uses this local
# build instead of downloading CI's (web/scripts/gallery-artifact.mjs).
#
# Usage:
#   bash .shots/build-wasm.sh                   # -> web/public/gallery/
#   bash .shots/build-wasm.sh --out DIR         # anywhere else (CI)
#   bash .shots/build-wasm.sh --source ci       # stamp as a CI build
#   bash .shots/build-wasm.sh --no-opt          # skip wasm-opt (no binaryen)
#
# `+nightly` is required (upstream's `multithreaded` default pulls in
# `wasm_thread`, a `#![feature]` crate). Never set RUSTFLAGS for this target:
# `.cargo/config.toml` states `rustflags = []`, and the variable would replace
# that list. Set CARGO_BUILD_JOBS to limit parallelism on a shared machine.

set -euo pipefail

# binaryen release whose wasm-opt CI installs (sha256-pinned in ci.yml).
WASM_OPT_VERSION=133
# -O1, not -Oz: measured on this artifact (web/DEPLOYMENT.md, section 6),
# -O1 is the only level that shrinks the brotli/gzip transfer as well as the
# raw module; -O2..-Oz cut raw bytes further but compress ~3-4% worse, and
# visitors download the compressed bytes. Rust's wasm32 target emits these
# features (plus per-function atomics from the `multithreaded` code), and
# wasm-opt validates against an explicit list rather than detecting it.
WASM_OPT_ARGS=(
    -O1
    --enable-bulk-memory
    --enable-nontrapping-float-to-int
    --enable-sign-ext
    --enable-mutable-globals
    --enable-reference-types
    --enable-multivalue
    --enable-threads
)

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
wasm_toolchain=$(cat "$root/.shots/wasm-toolchain.txt")
[[ "$wasm_toolchain" =~ ^nightly-[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || {
    echo "invalid .shots/wasm-toolchain.txt: $wasm_toolchain" >&2
    exit 1
}
out="$root/web/public/gallery"
source=local
opt=1
while [ "$#" -gt 0 ]; do
    case "$1" in
        --out) out=${2:?--out needs a directory}; shift 2 ;;
        --source) source=${2:?--source needs ci or local}; shift 2 ;;
        --no-opt) opt=0; shift ;;
        -h | --help) sed -n '2,24p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown argument: $1 (see --help)" >&2; exit 2 ;;
    esac
done
cd "$root"

if [ -n "${RUSTFLAGS:-}" ]; then
    echo "RUSTFLAGS is set; unset it (.cargo/config.toml owns the wasm32 rustflags)" >&2
    exit 1
fi

lock_bindgen=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
cli_bindgen=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}') || true
if [ -z "$lock_bindgen" ] || [ "$lock_bindgen" != "$cli_bindgen" ]; then
    echo "wasm-bindgen CLI ${cli_bindgen:-missing} does not match Cargo.lock's $lock_bindgen:" >&2
    echo "  cargo install -f wasm-bindgen-cli --version $lock_bindgen" >&2
    exit 1
fi

opt_version=none
if [ "$opt" -eq 1 ]; then
    opt_version=$(wasm-opt --version 2>/dev/null) || true
    case "$opt_version" in
        *"version $WASM_OPT_VERSION "* | *"version ${WASM_OPT_VERSION}") ;;
        *)
            echo "wasm-opt from binaryen version_$WASM_OPT_VERSION is required (found: ${opt_version:-none})." >&2
            echo "  https://github.com/WebAssembly/binaryen/releases/tag/version_$WASM_OPT_VERSION" >&2
            echo "  or pass --no-opt for an unoptimised local build" >&2
            exit 1
            ;;
    esac
fi

cargo +"$wasm_toolchain" build --locked --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web

staging=$(mktemp -d "${TMPDIR:-/tmp}/herogpui-wasm.XXXXXX")
trap 'rm -rf "$staging"' EXIT
wasm-bindgen --target web --no-typescript --out-dir "$staging" \
    target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm
raw=$(wc -c <"$staging/herogpui_web_bg.wasm" | tr -d ' ')
if [ "$opt" -eq 1 ]; then
    wasm-opt "${WASM_OPT_ARGS[@]}" "$staging/herogpui_web_bg.wasm" -o "$staging/optimised.wasm"
    mv "$staging/optimised.wasm" "$staging/herogpui_web_bg.wasm"
    echo "wasm-opt ${WASM_OPT_ARGS[0]}: $raw -> $(wc -c <"$staging/herogpui_web_bg.wasm" | tr -d ' ') bytes"
fi

mkdir -p "$out"
cp "$staging/herogpui_web.js" "$staging/herogpui_web_bg.wasm" "$out/"
cp crates/herogpui-web/index.html "$out/index.html"
node web/scripts/gallery-artifact.mjs stamp --dir "$out" --source "$source" \
    --toolchain "$(rustc +"$wasm_toolchain" -V)" --wasm-bindgen "$cli_bindgen" --wasm-opt "$opt_version"
