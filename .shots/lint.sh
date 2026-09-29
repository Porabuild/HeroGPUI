#!/usr/bin/env bash
# The lint gate the workspace manifest points at.
#
# `[workspace.lints]` only applies to a crate that says `lints.workspace = true`,
# so a new crate that forgets the line silently opts out of the whole policy.
# This checks that first, then runs clippy with warnings denied so a warning
# fails rather than scrolls past, then cargo-deny when it is installed.
#
# Bash for the reason `.shots/run-tests.sh` is bash: the gate has to run on
# the ubuntu CI runners and on macOS/Linux development machines with no extra
# install step, and `pwsh` is on neither by default. `.shots/lint.ps1` is a
# thin wrapper that calls this script, for Windows shells.
#
# Usage:
#   bash .shots/lint.sh                 # inheritance check + clippy -D warnings (default
#                                       # and --all-features) + deny
#   bash .shots/lint.sh --fix           # apply machine-applicable fixes first
#   bash .shots/lint.sh --require-deny  # a missing cargo-deny fails (what CI calls)
#   bash .shots/lint.sh --self-test     # prove the inheritance check fires
#
# Exit codes:
#   0  PASS
#   1  LINTS NOT INHERITED -- a member manifest has no `[lints] workspace = true`.
#   2  NOTHING CHECKED     -- no member manifests were found; an empty check is
#                             not a pass.
#   3  CLIPPY FAILED       -- clippy reported a warning or error.
#   4  DENY FAILED         -- cargo-deny failed, or is missing under --require-deny.
#   5  USAGE               -- bad arguments, or a failed --self-test.

set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)

red() { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
yellow() { printf '\033[33m%s\033[0m\n' "$*"; }

# ---------------------------------------------------------------------------
# Whether one manifest inherits the workspace lint table. Accepted spellings:
# a `[lints]` table holding `workspace = true`, or the top-level (before any
# table header) dotted/inline forms `lints.workspace = true` and
# `lints = { workspace = true }`. A `workspace = true` under any other table
# -- `[dependencies.foo]`, say -- does not count, which the old PowerShell
# regex (any line reading `workspace = true`) could not tell apart.
# ---------------------------------------------------------------------------
inherits_lints() {
    awk '
        /^[[:space:]]*\[/ {
            section = $0
            sub(/^[[:space:]]*\[+[[:space:]]*/, "", section)
            sub(/[[:space:]]*\]+.*$/, "", section)
            next
        }
        section == "lints" && /^[[:space:]]*workspace[[:space:]]*=[[:space:]]*true[[:space:]]*(#.*)?$/ { found = 1 }
        section == "" && /^[[:space:]]*lints\.workspace[[:space:]]*=[[:space:]]*true[[:space:]]*(#.*)?$/ { found = 1 }
        section == "" && /^[[:space:]]*lints[[:space:]]*=[[:space:]]*\{[[:space:]]*workspace[[:space:]]*=[[:space:]]*true[[:space:]]*\}/ { found = 1 }
        END { exit found ? 0 : 1 }
    ' "$1"
}

# Every workspace member crate under the three member directories. HeroGPUI's
# own crates live under crates/, gallery/ and examples/ (see the root
# Cargo.toml `members`), so a new member lands in one of these globs.
member_manifests() {
    local m
    for m in "$root"/crates/*/Cargo.toml "$root"/gallery/Cargo.toml "$root"/examples/*/Cargo.toml; do
        [ -f "$m" ] && printf '%s\n' "$m"
    done
}

check_inheritance() {
    local manifests=("$@") missing=() m
    if [ "${#manifests[@]}" -eq 0 ]; then
        red "NOTHING CHECKED: no member Cargo.toml found under crates/, gallery/ or examples/"
        return 2
    fi
    for m in "${manifests[@]}"; do
        inherits_lints "$m" || missing+=("${m#"$root"/}")
    done
    if [ "${#missing[@]}" -gt 0 ]; then
        red "these crates do not inherit [workspace.lints]:"
        for m in "${missing[@]}"; do red "  $m"; done
        yellow "add:"
        yellow "[lints]"
        yellow "workspace = true"
        return 1
    fi
    echo "all ${#manifests[@]} crates inherit [workspace.lints]"
}

# ---------------------------------------------------------------------------
# --self-test: the inheritance check over crafted manifests, including the
# known-negative shapes it exists to catch (.shots/AGENTS.md requires one).
# ---------------------------------------------------------------------------
self_test() {
    dir=$(mktemp -d "${TMPDIR:-/tmp}/herogpui-lint-selftest.XXXXXX")
    trap 'rm -rf "${dir:-}"' EXIT INT TERM
    local pass=0 fail=0

    case_() {
        local name=$1 expected=$2 body=$3 got=0
        printf '%s\n' "$body" >"$dir/Cargo.toml"
        inherits_lints "$dir/Cargo.toml" || got=$?
        if [ "$got" -eq "$expected" ]; then
            echo "    OK: $name"
            pass=$((pass + 1))
        else
            echo "    WRONG: $name (got $got, expected $expected)"
            fail=$((fail + 1))
        fi
    }

    case_ "[lints] table" 0 $'[package]\nname = "a"\n\n[lints]\nworkspace = true'
    case_ "[lints] table with a comment" 0 $'[lints]\nworkspace = true # policy'
    case_ "top-level dotted key" 0 $'lints.workspace = true\n[package]\nname = "a"'
    case_ "top-level inline table" 0 $'lints = { workspace = true }\n[package]\nname = "a"'
    case_ "no lints at all" 1 $'[package]\nname = "a"\nversion.workspace = true'
    case_ "workspace = true under a dependency table" 1 $'[package]\nname = "a"\n\n[dependencies.gpui]\nworkspace = true'
    case_ "[lints] table set to false" 1 $'[lints]\nworkspace = false'
    case_ "dotted key inside [package]" 1 $'[package]\nlints.workspace = true'

    local got=0
    check_inheritance >/dev/null 2>&1 || got=$?
    if [ "$got" -eq 2 ]; then
        echo "    OK: an empty member list is NOTHING CHECKED"
        pass=$((pass + 1))
    else
        echo "    WRONG: an empty member list returned $got, expected 2"
        fail=$((fail + 1))
    fi

    echo "self-test: $pass passed, $fail failed"
    [ "$fail" -eq 0 ] || return 5
}

fix=0
require_deny=0
for arg in "$@"; do
    case "$arg" in
        --fix) fix=1 ;;
        --require-deny) require_deny=1 ;;
        --self-test)
            if [ "$#" -ne 1 ]; then
                echo "usage: $0 --self-test" >&2
                exit 5
            fi
            status=0
            self_test || status=$?
            exit "$status"
            ;;
        -h | --help)
            sed -n '2,28p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "unknown argument: $arg (see --help)" >&2
            exit 5
            ;;
    esac
done

cd "$root"

# 1. Every member crate must opt in, or the policy is not what it looks like.
manifests=()
while IFS= read -r m; do manifests+=("$m"); done < <(member_manifests)
status=0
check_inheritance ${manifests[@]+"${manifests[@]}"} || status=$?
[ "$status" -eq 0 ] || exit "$status"

if [ "$fix" -eq 1 ]; then
    # rustc's own lints (elided lifetimes, unused qualifications) are
    # machine-applicable; clippy's stylistic ones mostly are too.
    cargo fix --workspace --all-targets --allow-dirty
    cargo clippy --workspace --all-targets --fix --allow-dirty
fi

# 2. Warnings are failures here. `--all-targets` covers tests, which is where
#    float comparisons and unused imports usually hide. Twice: the default
#    features, which is what a downstream `herogpui = "..."` compiles, and
#    `--all-features`, because a lint (`missing_docs` above all) only fires on
#    code that is compiled, and the feature-gated modules -- herogpui-theme's
#    `serde` and `watch`, herogpui-components' `gallery-source`, the facade's
#    feature split -- are compiled by neither run otherwise. No two features
#    in the workspace exclude each other, so the union is a valid build (the
#    `lint` job's `cargo hack --each-feature` covers each one alone).
if ! cargo clippy --workspace --all-targets -- -D warnings; then
    red "clippy failed (default features)"
    exit 3
fi
if ! cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    red "clippy failed (--all-features)"
    exit 3
fi
green "clippy clean (warnings denied; default and all features)"

# 3. License and advisory compliance for every crate in the graph.
if cargo deny --version >/dev/null 2>&1; then
    if ! cargo deny --manifest-path Cargo.toml check; then
        red "cargo deny failed"
        exit 4
    fi
    green "cargo deny clean"
elif [ "$require_deny" -eq 1 ]; then
    red "cargo-deny is not installed, and --require-deny makes that a failure"
    exit 4
else
    yellow "skipping cargo-deny: not installed (cargo install cargo-deny --locked)"
fi
