#!/usr/bin/env bash
# Runs the workspace tests and decides pass/fail from what the harness
# *reported*, not from cargo's exit status alone.
#
# Two failure shapes motivate this, and they point in opposite directions.
# Both are borrowed verbatim from gpuikit, which paid for them:
#
#   * gpuikit#180 -- SILENT DEATH. A test binary dies without ever printing a
#     `test result:` summary: OOM-killed while linking, killed by a signal, or
#     aborted mid-run. With ~70 test binaries in this workspace one of them
#     going missing is invisible in the scrollback, and in some pipelines cargo
#     still exits 0, so a killed run reads as a pass.
#   * gpuikit#190 -- TEARDOWN ABORT. Every test passed and every summary says
#     `ok`, yet the process still exited non-zero, because a thread with no
#     exit path aborted during teardown. Cargo's status is the only witness,
#     and "all green" tempts everyone to re-run until it goes away.
#
# Neither shape is caught by `cargo test --workspace --locked` alone, which is
# what CI ran before this script existed. So this counts cargo's own `Running`
# announcements against the `test result:` summaries they owe, and treats a
# disagreement between the summaries and the exit status as a failure in either
# direction.
#
# Why bash, in a .shots directory that is otherwise PowerShell: the capture and
# audit tooling here runs on the maintainer's Windows box, but the gate CI
# invokes runs on ubuntu-24.04 runners, and `pwsh` is not installed on every
# development machine either (it is absent from this one). The thing CI calls
# has to run with no extra install step, so it is bash. The PowerShell scripts
# keep their conventions -- a loud named failure, a non-zero exit, and no
# silent empty pass -- which this follows.
#
# Usage:
#   bash .shots/run-tests.sh                        # --workspace --locked
#   bash .shots/run-tests.sh --workspace --locked   # what CI calls
#   bash .shots/run-tests.sh -p herogpui-components # focused iteration
#   bash .shots/run-tests.sh --self-test            # prove the guards fire
#   bash .shots/run-tests.sh --judge-log <file> [--judge-status N]
#
# Exit codes:
#   0  PASS         -- every announced binary reported, all ok, cargo exited 0.
#   1  FAILED       -- at least one `test result: FAILED` summary.
#   2  NOTHING RAN  -- cargo announced no test binaries at all (build broke, or
#                      the filter matched nothing). An empty run is not a pass.
#   3  NO SUMMARY   -- fewer summaries than announced binaries; one died without
#                      reporting. This is the gpuikit#180 shape.
#   4  GREEN ABORT  -- all summaries ok but cargo exited non-zero; nothing failed
#                      *during* the run, so the process died at teardown. This is
#                      the gpuikit#190 shape.
#   5  USAGE        -- bad arguments to this script, or a failed --self-test.

set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)

# ---------------------------------------------------------------------------
# The verdict. Kept as a function over a captured log so --self-test can feed
# it crafted logs; see .shots/AGENTS.md, which requires a known-negative
# demonstration alongside the passing repository result.
# ---------------------------------------------------------------------------
judge() {
    local log=$1 status=$2
    local binaries summaries failed

    # `Running <unittests|tests> ...` and `Doc-tests <crate>` each announce one
    # binary; `test result: ...` is one summary. Cargo indents both
    # announcements, hence the leading whitespace class.
    binaries=$(grep -c -E '^[[:space:]]*(Running|Doc-tests) ' "$log" || true)
    summaries=$(grep -c '^test result:' "$log" || true)
    failed=$(grep -c '^test result: FAILED' "$log" || true)

    echo "cargo exit: $status | test binaries announced: $binaries | summaries: $summaries"

    if [ "$binaries" -eq 0 ]; then
        echo "VERDICT: NOTHING RAN -- cargo announced no test binaries."
        echo "  Look above: the build failed, or the target filter matched nothing."
        echo "  An empty run is not a passing run."
        return 2
    fi

    # This comparison is `<`, NOT `!=`, and under edition 2024 it has to be.
    # Edition 2024 merges a crate's doctests into a single binary, but rustdoc
    # still compiles standalone whatever it cannot merge -- a `compile_fail`
    # block, for one -- and reports a `test result:` line per group. So a single
    # `Doc-tests` line can legitimately owe one summary or several, and only a
    # SHORTFALL means anything. This workspace is edition 2024 (see the root
    # Cargo.toml `[workspace.package]`), so `!=` here would fail every green run
    # that has a compile_fail doctest.
    if [ "$summaries" -lt "$binaries" ]; then
        echo "VERDICT: FAIL (no summary) -- $binaries test binaries were announced but only"
        echo "  $summaries reported a summary. One died without reporting: OOM kill, signal,"
        echo "  or an abort mid-run. This is the gpuikit#180 shape -- grep the output above"
        echo "  for 'signal 9', 'SIGKILL', 'error: test failed' with no summary, or a"
        echo "  'Running' line with nothing after it."
        return 3
    fi

    if [ "$failed" -gt 0 ]; then
        echo "VERDICT: FAIL -- $failed test binary/binaries reported failures."
        echo "  Search the output above for 'test result: FAILED'."
        return 1
    fi

    if [ "$status" -ne 0 ]; then
        echo "VERDICT: FAIL (green abort) -- every test passed and every summary says ok,"
        echo "  yet cargo exited $status. Nothing failed *during* the run, so this is a"
        echo "  teardown abort: some thread was still live when the process exited. This is"
        echo "  the gpuikit#190 shape. Do NOT re-run until it goes green -- find the thread."
        echo "  'cargo tree -i async-io' and 'cargo tree -i smol' are the place to start."
        return 4
    fi

    echo "VERDICT: PASS -- $binaries test binaries announced, $summaries summaries, all ok,"
    echo "  cargo exited 0."
    return 0
}

# ---------------------------------------------------------------------------
# --self-test: run the verdict over crafted logs and check the exit codes.
# The fixtures are generated into a temp directory and removed on exit, so
# nothing is left in the working tree of this frequently dirty checkout.
# ---------------------------------------------------------------------------
self_test() {
    # Deliberately not `local`: the cleanup trap runs after this function has
    # returned, so a local would be out of scope by then.
    dir=$(mktemp -d "${TMPDIR:-/tmp}/herogpui-run-tests-selftest.XXXXXX")
    trap 'rm -rf "${dir:-}"' EXIT INT TERM

    local pass=0 fail=0

    check() {
        local name=$1 expected=$2 log=$3 status=$4
        local got=0
        echo "--- case: $name (expect exit $expected)"
        judge "$log" "$status" || got=$?
        if [ "$got" -eq "$expected" ]; then
            echo "    OK: exit $got"
            pass=$((pass + 1))
        else
            echo "    WRONG: exit $got, expected $expected"
            fail=$((fail + 1))
        fi
        echo
    }

    # 1. Two binaries announced, both reported ok, cargo exited 0.
    cat >"$dir/green.log" <<'EOF'
   Compiling herogpui-core v0.1.0
    Finished `test` profile [unoptimized] target(s) in 1.20s
     Running unittests src/lib.rs (target/debug/deps/herogpui_core-1111111111111111)

running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/tokens.rs (target/debug/deps/tokens-2222222222222222)

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EOF
    check "green run" 0 "$dir/green.log" 0

    # 2. THE #180 SHAPE. Three binaries announced, the third was killed before
    #    it could print a summary. Bare cargo can report this as a pass.
    cat >"$dir/killed.log" <<'EOF'
     Running unittests src/lib.rs (target/debug/deps/herogpui_core-1111111111111111)

running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/tokens.rs (target/debug/deps/tokens-2222222222222222)

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/overlay.rs (target/debug/deps/overlay-3333333333333333)

running 12 tests
EOF
    check "binary killed with no summary (#180)" 3 "$dir/killed.log" 0

    # 3. THE #190 SHAPE. Everything green, cargo still exited non-zero.
    check "all green, cargo exited 101 (#190)" 4 "$dir/green.log" 101

    # 4. A real reported failure.
    cat >"$dir/failed.log" <<'EOF'
     Running unittests src/lib.rs (target/debug/deps/herogpui_core-1111111111111111)

running 7 tests
failures:
    tokens::radius_scale
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
EOF
    check "reported test failure" 1 "$dir/failed.log" 101

    # 5. Nothing announced at all -- a broken build reads as an empty log.
    cat >"$dir/nothing.log" <<'EOF'
   Compiling herogpui-components v0.1.0
error[E0432]: unresolved import `gpui::Hsla`
error: could not compile `herogpui-components` (lib test) due to 1 previous error
EOF
    check "build broke, nothing announced" 2 "$dir/nothing.log" 101

    # 6. One Doc-tests line owing several summaries -- edition 2024 splits
    #    compile_fail blocks out. This is why the comparison is `<`, not `!=`;
    #    with `!=` this legitimate green run would be reported as a failure.
    cat >"$dir/doctests.log" <<'EOF'
     Running unittests src/lib.rs (target/debug/deps/herogpui-4444444444444444)

running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests herogpui

running 5 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s

running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
EOF
    check "one Doc-tests line, three summaries" 0 "$dir/doctests.log" 0

    # 7. CARGO_TERM_COLOR=always is why the run below is forced to `never`: the
    #    escape codes break the anchor and the count sees zero binaries, which
    #    would turn every coloured run into a spurious NOTHING RAN. Asserting
    #    that here keeps the reason for the env var from being "cleaned up".
    printf '\033[1m\033[32m     Running\033[0m unittests src/lib.rs (target/debug/deps/x-1)\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n' >"$dir/coloured.log"
    check "coloured output would count zero binaries" 2 "$dir/coloured.log" 0

    echo "self-test: $pass passed, $fail failed"
    [ "$fail" -eq 0 ] || return 5
    return 0
}

# ---------------------------------------------------------------------------
# Argument handling.
# ---------------------------------------------------------------------------
if [ "${1:-}" = "--self-test" ]; then
    if [ "$#" -ne 1 ]; then
        echo "usage: $0 --self-test" >&2
        exit 5
    fi
    status=0
    self_test || status=$?
    exit "$status"
fi

# --judge-log lets a crafted or archived log be re-judged without running the
# suite, which is how a CI log from a past failure can be replayed against the
# guard. Kept explicit rather than an env var so it shows up in --help.
if [ "${1:-}" = "--judge-log" ]; then
    log=${2:-}
    if [ -z "$log" ] || [ ! -f "$log" ]; then
        echo "usage: $0 --judge-log <captured-log> [--judge-status N]" >&2
        exit 5
    fi
    judge_status=0
    if [ "${3:-}" = "--judge-status" ]; then
        judge_status=${4:-0}
    elif [ "$#" -gt 2 ]; then
        echo "usage: $0 --judge-log <captured-log> [--judge-status N]" >&2
        exit 5
    fi
    echo "judging: $log (simulated cargo exit $judge_status)"
    echo
    status=0
    judge "$log" "$judge_status" || status=$?
    exit "$status"
fi

case "${1:-}" in
    -h | --help)
        sed -n '2,49p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
        exit 0
        ;;
esac

# CI calls this with the same arguments the old bare command used, and a
# developer overrides them. Defaulting keeps `bash .shots/run-tests.sh` honest
# about which suite it judged.
if [ "$#" -eq 0 ]; then
    set -- --workspace --locked
fi

# `--no-fail-fast` is forced, and it is load-bearing for the verdict rather
# than a convenience. Cargo stops launching test binaries at the first one that
# fails, so a single red binary silently truncates the run: this suite reported
# "24 test binaries announced" against a full-suite 68 the moment one submenu
# test broke, and the 44 binaries that never ran looked indistinguishable from
# binaries that passed. The announced-vs-summary comparison below can only
# judge what cargo actually attempted, so the whole suite has to be attempted.
# A caller who genuinely wants to stop early passes `--fail-fast` and it wins,
# because a later cargo argument overrides an earlier one.
set -- --no-fail-fast "$@"

log=$(mktemp "${TMPDIR:-/tmp}/herogpui-tests.XXXXXX")
trap 'rm -f "$log"' EXIT INT TERM

echo "running: cargo test $*"
echo

# The output is CAPTURED TO A FILE, not piped. A shell pipeline hands back its
# LAST command's status, so `cargo test | tee` or `cargo test | grep` reports
# tee's or grep's success -- which is the exact thing this script exists to
# guard against. `set -o pipefail` would fix the status but not the habit, and
# the verdict needs the whole output on disk anyway.
#
# CARGO_TERM_COLOR=never is set for the run being judged because the verdict
# greps cargo's own announcements: under `always` (which CI environments do set)
# every `Running` line arrives wrapped in escape codes, the `^[[:space:]]*Running`
# anchor never matches, and the count sees zero test binaries -- a green suite
# would be reported as NOTHING RAN. Case 7 of --self-test pins this.
status=0
(cd "$root" && CARGO_TERM_COLOR=never cargo test "$@") >"$log" 2>&1 || status=$?

cat "$log"
echo

verdict=0
judge "$log" "$status" || verdict=$?
exit "$verdict"
