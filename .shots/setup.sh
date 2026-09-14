#!/bin/sh
# HeroGPUI: the one command a fresh clone needs, and the last one.
#
#   sh .shots/setup.sh
#
# Git deliberately never runs a repository's own hooks on `clone` -- it would
# mean executing code from the remote before anyone had read it -- so the first
# materialization of `.vendor/` in a new clone cannot be automated. This is that
# step. It also points `core.hooksPath` at the versioned `.githooks/`, and from
# then on post-checkout, post-merge and post-rewrite keep `.vendor/` in step
# with whatever `docs/upstream/patches/` says on the branch you are on.
#
# Idempotent, and it never takes a setting that is already spoken for: a
# `core.hooksPath` you (or your global config) already set is left exactly as
# it is, and you keep running `.shots/materialize.sh` yourself in that case.

set -u

prefix='herogpui:'

root=$(git rev-parse --show-toplevel 2>/dev/null) || root=''
if [ -z "$root" ]; then
    root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." 2>/dev/null && pwd) || root=''
fi
if [ -z "$root" ]; then
    printf '%s %s\n' "$prefix" "cannot find the repository root; run this from inside the checkout." >&2
    exit 1
fi

# Enabled here in shell rather than only from `gpui_patches.py`, so the hooks
# get switched on even when this machine has no usable Python yet: the
# materialization below will complain, but the next `git pull` will fix itself.
if [ -d "$root/.githooks" ] && [ -z "$(git -C "$root" config --get core.hooksPath 2>/dev/null)" ]; then
    if git -C "$root" config core.hooksPath .githooks 2>/dev/null; then
        printf '%s %s\n' "$prefix" "git hooks enabled: core.hooksPath = .githooks"
        printf '%s %s\n' "$prefix" "  checkout, merge and rebase now rebuild .vendor/ for you."
    fi
fi

exec sh "$root/.shots/materialize.sh"
