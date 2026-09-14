#!/bin/sh
# HeroGPUI: make `.vendor/` the patched GPUI sources, before anything runs cargo.
#
# The workspace's `[patch.crates-io]` table points at five gitignored trees that
# `.shots/gpui_patches.py --materialize` builds from the pinned published
# packages plus `docs/upstream/patches/*.patch`. Cargo resolves those paths at
# manifest load -- before any build script, and before a cargo alias could
# intercept anything -- so the only way a developer never has to think about it
# is for everything that shells out to cargo to run this first. A warm run is a
# hash comparison over the materialization stamps; it prints nothing and costs
# a fraction of a second, so it is cheap enough to sit in front of anything.
#
#   sh .shots/materialize.sh          # rebuild, and fail loudly if it cannot
#   sh .shots/materialize.sh --soft   # git-hook mode: warn, then always exit 0
#
# `--soft` is what the hooks in `.githooks/` use. A hook must never wedge a
# checkout, a merge or a rebase, so in that mode every failure is a warning and
# the exit status is 0 regardless; it also leaves `core.hooksPath` alone,
# because hooks running at all means it is already set.

set -u

prefix='herogpui:'
soft=0
[ "${1:-}" = "--soft" ] && soft=1

warn() { printf '%s %s\n' "$prefix" "$1" >&2; }

# Explicitly, not from `$PWD`: git runs hooks from the top of the work tree,
# but a developer runs this from wherever they happen to be, and a linked
# worktree has a top of its own. The script's own location is the fallback for
# a tree that is not a git checkout at all -- an exported archive, say.
root=$(git rev-parse --show-toplevel 2>/dev/null) || root=''
if [ -z "$root" ]; then
    root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." 2>/dev/null && pwd) || root=''
fi

# A revision from before the forks became patches has no materializer and needs
# none. Say nothing: this runs from hooks, and a hook that comments on every
# checkout of an old branch is a hook people turn off.
[ -n "$root" ] && [ -f "$root/.shots/gpui_patches.py" ] || exit 0

python=''
for candidate in python3 python py; do
    command -v "$candidate" >/dev/null 2>&1 || continue
    # Probed, not trusted by name: `python` is still Python 2 on some
    # long-lived distributions and an install-prompt stub on Windows, and
    # `gpui_patches.py` imports `tomllib`, which arrives in 3.11.
    "$candidate" -c 'import sys; raise SystemExit(sys.version_info < (3, 11))' >/dev/null 2>&1 || continue
    python=$candidate
    break
done

if [ -z "$python" ]; then
    warn "no Python 3.11+ on PATH, so .vendor/ was not refreshed."
    warn "cargo will stop at 'failed to load source for dependency' until you run:"
    warn "  python3 .shots/gpui_patches.py --materialize"
    [ "$soft" -eq 1 ] && exit 0
    exit 1
fi

if [ "$soft" -eq 1 ]; then
    set -- --materialize --quiet --no-hook-setup
else
    set -- --materialize --quiet
fi

output=$("$python" "$root/.shots/gpui_patches.py" "$@" 2>&1)
status=$?

# `--quiet` reports only the packages this run actually rebuilt, so the warm
# path has nothing here and stays silent.
if [ -n "$output" ]; then
    printf '%s\n' "$output" | while IFS= read -r line; do
        printf '%s %s\n' "$prefix" "$line"
    done
fi

if [ "$status" -ne 0 ]; then
    warn "could not rebuild the patched GPUI sources under .vendor/ (exit $status)."
    warn "cargo will stop at 'failed to load source for dependency' until that is fixed."
    [ "$soft" -eq 1 ] && exit 0
    exit "$status"
fi
exit 0
