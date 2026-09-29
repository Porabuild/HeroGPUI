#!/usr/bin/env bash
# Publish a stamped web-gallery build (.shots/build-wasm.sh output) as its own
# GitHub prerelease, where the site build (web/scripts/gallery-artifact.mjs
# fetch) downloads it. CI's `wasm-publish` job runs this with a
# `contents: write` GITHUB_TOKEN; nothing else writes these releases.
#
# One release per artifact key, tagged `gallery-<key16>`:
#
#   herogpui-gallery-<wasm sha256[:16]>.wasm   the binaries
#   herogpui-gallery-<glue sha256[:16]>.js
#   herogpui-gallery-<key16>.json              the build-info the site reads
#
# The release is created as a draft, every asset is attached while it is a
# draft, and only then is it published. A published release is never given,
# or relieved of, an asset again, so this works unchanged with GitHub's
# immutable releases turned on (RELEASING.md, one-time setup). What
# immutability still allows is all this script does afterwards: editing a
# published release's title (a master push appends `[master <time>]`, which
# the preview fallback looks for) and deleting whole releases (pruning).
#
# Idempotent: a key that is already published is left alone. Two runs racing
# for one key each build a draft; the first to publish wins and the loser
# deletes its draft.
#
# Usage: bash .shots/publish-gallery.sh <dir> [--master]
# Needs: gh (authenticated via GH_TOKEN), jq, GITHUB_REPOSITORY; optional
# HEAD_SHA (the commit the tag points at) and KEEP (releases kept, 30).

set -euo pipefail

KEEP=${KEEP:-30}
dir=${1:?usage: publish-gallery.sh <dir> [--master]}
master=0
[ "${2:-}" = "--master" ] && master=1
repo=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must name owner/repo}
info="$dir/build-info.json"

key=$(jq -er '.inputsSha256' "$info")
[[ "$key" =~ ^[0-9a-f]{64}$ ]] || { echo "malformed key in $info" >&2; exit 1; }
wasm_name=$(jq -er '.wasm' "$info")
glue_name=$(jq -er '.glue' "$info")
tag="gallery-${key:0:16}"
json_name="herogpui-gallery-${key:0:16}.json"
title="Web gallery ${key:0:16}"

# Run a `gh api` call; on failure print its error, and name an immutability
# rejection plainly instead of leaving a bare HTTP 422 in the log.
api() {
    local out status=0
    out=$(gh api "$@" 2>&1) || status=$?
    if [ "$status" -ne 0 ]; then
        if grep -qi 'immutable' <<<"$out"; then
            echo "::error::GitHub rejected a change to a published, immutable release: $out" >&2
            echo "  .shots/publish-gallery.sh only attaches assets to drafts; if this appears, a" >&2
            echo "  published release was modified (or immutability now covers drafts). See" >&2
            echo "  web/DEPLOYMENT.md section 6 and RELEASING.md, one-time setup." >&2
        else
            echo "$out" >&2
        fi
        return "$status"
    fi
    printf '%s\n' "$out"
}

published_id() {
    # `gh api --jq` can write GitHub's 404 JSON to stdout before returning
    # failure. A command substitution would treat that body as an ID and send
    # a PATCH to a malformed URL. `gh release view` leaves stdout empty for a
    # missing release and returns its numeric database ID for an existing one.
    gh release view "$1" --repo "$repo" --json databaseId,isDraft \
        --jq 'select(.isDraft == false) | .databaseId' 2>/dev/null || true
}

id=$(published_id "$tag")
if [ -n "$id" ]; then
    echo "kept    $tag (already published)"
else
    staging=$(mktemp -d "${TMPDIR:-/tmp}/gallery-publish.XXXXXX")
    trap 'rm -rf "$staging"' EXIT
    cp "$dir/herogpui_web_bg.wasm" "$staging/$wasm_name"
    cp "$dir/herogpui_web.js" "$staging/$glue_name"
    cp "$info" "$staging/$json_name"

    draft=$(api "repos/$repo/releases" -X POST \
        -f tag_name="$tag" -f target_commitish="${HEAD_SHA:-${DEFAULT_BRANCH:-master}}" \
        -f name="$title" -F draft=true -F prerelease=true -f make_latest=false \
        -f body="Web gallery artifact built by CI (inputs ${key}). Downloaded by the website build (web/scripts/gallery-artifact.mjs); not a HeroGPUI release, nothing here is installed by cargo. See web/DEPLOYMENT.md." \
        --jq '.id')
    # The build-info goes last: a draft never serves downloads anyway, but a
    # partial upload then fails before anything is published.
    for name in "$wasm_name" "$glue_name" "$json_name"; do
        api "https://uploads.github.com/repos/$repo/releases/$draft/assets?name=$name" \
            -X POST -H "Content-Type: application/octet-stream" --input "$staging/$name" \
            --jq '.name' >/dev/null
        echo "attached $name"
    done
    if api "repos/$repo/releases/$draft" -X PATCH -F draft=false --jq '.id' >/dev/null; then
        id=$draft
        echo "published $tag"
    else
        id=$(published_id "$tag")
        [ -n "$id" ] || { echo "could not publish $tag" >&2; exit 1; }
        api "repos/$repo/releases/$draft" -X DELETE >/dev/null || true
        echo "kept    $tag (published concurrently; dropped this run's draft)"
    fi
fi

if [ "$master" -eq 1 ]; then
    # A title edit, which immutable releases allow: the preview fallback
    # picks the newest `[master <time>]` release.
    stamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    api "repos/$repo/releases/$id" -X PATCH -f name="$title [master $stamp]" --jq '.id' >/dev/null
    echo "marked  $tag [master $stamp]"

    # Prune: keep the newest $KEEP gallery releases, this key, and the newest
    # master build; delete the rest whole (deleting a release is allowed when
    # it is immutable; its tag can only be deleted once the release is gone).
    # Drafts older than a day are leftovers of cancelled runs.
    releases=$(gh api --paginate "repos/$repo/releases" \
        --jq '.[] | select(.tag_name | test("^gallery-[0-9a-f]{16}$")) | [.id, .tag_name, .draft, .created_at, .name] | @tsv')
    newest_master=$(awk -F'\t' '$3 == "false" && match($5, /\[master [^]]*\]/) { print substr($5, RSTART, RLENGTH) "\t" $2 }' <<<"$releases" |
        sort -r | head -n 1 | cut -f 2)
    kept=$(awk -F'\t' '$3 == "false" { print $4 "\t" $2 }' <<<"$releases" | sort -r | head -n "$KEEP" | cut -f 2)
    cutoff=$(date -u -d '1 day ago' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v-1d +%Y-%m-%dT%H:%M:%SZ)
    while IFS=$'\t' read -r rid rtag rdraft rcreated _; do
        [ -n "$rid" ] || continue
        if [ "$rdraft" = "true" ]; then
            [[ "$rcreated" < "$cutoff" ]] || continue
        elif [ "$rtag" = "$tag" ] || [ "$rtag" = "$newest_master" ] || grep -Fxq "$rtag" <<<"$kept"; then
            continue
        fi
        if api "repos/$repo/releases/$rid" -X DELETE >/dev/null; then
            [ "$rdraft" = "true" ] || api "repos/$repo/git/refs/tags/$rtag" -X DELETE >/dev/null || true
            if [ "$rdraft" = "true" ]; then echo "pruned  $rtag (stale draft)"; else echo "pruned  $rtag"; fi
        else
            echo "::warning::could not delete release $rtag; left in place" >&2
        fi
    done <<<"$releases"
fi
