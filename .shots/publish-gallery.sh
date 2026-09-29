#!/usr/bin/env bash
# Publish a stamped web-gallery build (.shots/build-wasm.sh output) to this
# repository's `gallery-artifacts` prerelease, where the site build
# (web/scripts/gallery-artifact.mjs fetch) downloads it. CI's `wasm-publish`
# job runs this with a `contents: write` GITHUB_TOKEN; nothing else writes
# the release.
#
#   herogpui-gallery-<wasm sha256[:16]>.wasm  content-addressed binaries, so
#   herogpui-gallery-<glue sha256[:16]>.js    two runs racing for one key
#                                             cannot mix their files
#   herogpui-gallery-<inputs sha256[:16]>.json  the build-info for that key,
#                                             uploaded last: its presence
#                                             means the binaries are complete
#   herogpui-gallery-master.json              master's latest build-info, the
#                                             preview fallback (--master only)
#
# Idempotent: an existing asset is kept, never overwritten (except the master
# pointer). With --master it also prunes builds beyond the newest $KEEP keys.
#
# Usage: bash .shots/publish-gallery.sh <dir> [--master]
# Needs: gh (authenticated via GH_TOKEN), jq, GITHUB_REPOSITORY.

set -euo pipefail

TAG=gallery-artifacts
KEEP=${KEEP:-60}
dir=${1:?usage: publish-gallery.sh <dir> [--master]}
master=0
[ "${2:-}" = "--master" ] && master=1
repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must name owner/repo}
info="$dir/build-info.json"

key=$(jq -er '.inputsSha256' "$info")
wasm_name=$(jq -er '.wasm' "$info")
glue_name=$(jq -er '.glue' "$info")
key_name="herogpui-gallery-${key:0:16}.json"
[[ "$key" =~ ^[0-9a-f]{64}$ ]] || { echo "malformed key in $info" >&2; exit 1; }

if ! gh release view "$TAG" -R "$repo" >/dev/null 2>&1; then
    # A concurrent run may create it first; only a release that still does
    # not exist afterwards is a failure.
    gh release create "$TAG" -R "$repo" --prerelease --latest=false \
        --target "${DEFAULT_BRANCH:-master}" --title "Web gallery artifacts (CI)" \
        --notes "Built by CI's wasm job and downloaded by the website build (web/scripts/gallery-artifact.mjs). Not a HeroGPUI release: nothing here is installed by cargo. See web/DEPLOYMENT.md." ||
        gh release view "$TAG" -R "$repo" >/dev/null
fi

assets() { gh release view "$TAG" -R "$repo" --json assets --jq '.assets[].name'; }
existing=$(assets)
has() { grep -Fxq "$1" <<<"$existing"; }

staging=$(mktemp -d "${TMPDIR:-/tmp}/gallery-publish.XXXXXX")
trap 'rm -rf "$staging"' EXIT

upload() {
    local path=$1 name=$2
    if has "$name"; then
        echo "kept    $name (already published)"
        return
    fi
    cp "$path" "$staging/$name"
    if ! gh release upload "$TAG" -R "$repo" "$staging/$name"; then
        existing=$(assets)
        has "$name" || return 1
        echo "kept    $name (published concurrently)"
        return
    fi
    echo "added   $name"
}

upload "$dir/herogpui_web_bg.wasm" "$wasm_name"
upload "$dir/herogpui_web.js" "$glue_name"
upload "$info" "$key_name"

if [ "$master" -eq 1 ]; then
    cp "$info" "$staging/herogpui-gallery-master.json"
    gh release upload "$TAG" -R "$repo" --clobber "$staging/herogpui-gallery-master.json"
    echo "pointed herogpui-gallery-master.json at ${key:0:16}"

    # Prune: keep the newest $KEEP keys plus whatever the pointer names. A
    # binary shared by several keys survives while any kept key names it.
    listing=$(gh release view "$TAG" -R "$repo" --json assets \
        --jq '.assets | sort_by(.createdAt) | reverse | .[] | .name')
    keys=$(grep -E '^herogpui-gallery-[0-9a-f]{16}\.json$' <<<"$listing" || true)
    kept=$(head -n "$KEEP" <<<"$keys")
    gh release download "$TAG" -R "$repo" -p 'herogpui-gallery-*.json' -D "$staging/json"
    referenced=$(
        for name in $kept herogpui-gallery-master.json; do
            jq -r '.wasm, .glue' "$staging/json/$name"
        done | sort -u
    )
    for name in $(grep -E '^herogpui-gallery-[0-9a-f]{16}\.(json|wasm|js)$' <<<"$listing" || true); do
        if grep -Fxq "$name" <<<"$kept" || grep -Fxq "$name" <<<"$referenced"; then continue; fi
        gh release delete-asset "$TAG" "$name" -R "$repo" --yes
        echo "pruned  $name"
    done
fi
