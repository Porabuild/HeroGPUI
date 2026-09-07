"""An example rustdoc never compiles is not an example -- it is prose that
looks like Rust.

`crates/` held 17 doc-comment fences, of which rustdoc compiled exactly one.
Twelve were examples carrying ` ```ignore `: every `form_field` example across
Checkbox, CheckboxGroup, Switch, RadioGroup, Select, Slider, Autocomplete,
InputOTP, ColorSlider and ColorField, plus `BadgeAnchor` and
`NumberField::format_options`. rustdoc skips an `ignore` block, so
`cargo test --doc` reported `1 passed; 12 ignored` and went green -- while
every one of the twelve referred to bindings (`control`, `form`, `avatar`,
`state`) that nothing in the block defined, so not one of them would have
compiled if anything had asked. Nothing checked them because nothing could.

| Fence | Rule |
|---|---|
| bare, `rust` | allowed -- an example that compiles *and runs* |
| `no_run` | allowed only with an entry in `NO_RUN` |
| `should_panic`, `compile_fail` | allowed only with an entry in `NO_RUN`'s siblings |
| a language tag (`text`, `js`, `css`, ...) | allowed -- not Rust, and it says so |
| `ignore`, or any attribute list containing it | never allowed, no escape hatch |
| anything else | rejected by name |

`ignore` has no allowlist on purpose. Every other fence states something true
about the block -- `no_run` says "this compiles but must not run", `text` says
"this is not Rust". `ignore` states nothing; it only asks rustdoc to look away,
which is the outcome this audit exists to prevent. An example that cannot be
made to compile is an example whose prose was doing the work all along, and the
prose can stay without it.

The fence is read as rustdoc reads it: an *attribute list*, comma- or
space-separated. ` ```rust,ignore ` and ` ```ignore-macos ` are as invisible to
`cargo test --doc` as a bare ` ```ignore `, so an exact-string match for
`ignore` is an evasion waiting to happen and this scans the tokens instead.

Both floors below exist because a parser that silently found nothing reports
success. Without them, deleting all twelve examples -- or a typo in the doc-comment
prefix match -- would leave "no ```ignore anywhere" true and meaningless.

Run from the repository root:

    python3 .shots/fence_audit.py
    python3 .shots/fence_audit.py --self-test

The scanner's own self-test runs as part of the audit, not only behind the
flag: its verdict is worth exactly what the parser behind it is worth.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SCAN_ROOT = 'crates'

# (path relative to the repository root, why running it is not an option).
# Every entry is asserted to still match a `no_run` fence in that file, so an
# example that stops needing the exemption fails this audit until its entry
# goes too.
#
# Empty, and worth keeping that way: `no_run` costs the one thing a doctest is
# for. `crates/herogpui/src/lib.rs` used to be here for a block that is a `use`
# and two comments -- there was never anything in it to run.
NO_RUN = {}

# (path, why the block is worth a whole-program link). rustdoc does check a
# `compile_fail` block, but it can never fold one into the merged doctest
# binary, so each is its own link of gpui. None today; the empty list is how
# that stays deliberate.
COMPILE_FAIL = {}

# (path, what the panic proves).
SHOULD_PANIC = {}

ALLOWLISTS = {
    'no_run': ('NO_RUN', NO_RUN,
               '`no_run` costs the one thing a doctest is for -- running. Add '
               'an entry saying why running it is not an option, or make it '
               'run.'),
    'compile_fail': ('COMPILE_FAIL', COMPILE_FAIL,
                     'a `compile_fail` block cannot join the merged doctest '
                     'binary, so each one is a whole-program link of gpui. '
                     'Add an entry saying the link is worth it.'),
    'should_panic': ('SHOULD_PANIC', SHOULD_PANIC,
                     'a doctest that asserts a panic is asserting an API '
                     'contract most readers will not expect. Add an entry '
                     'saying what the panic proves.'),
}

# Fences that need no justification: a Rust block rustdoc compiles and runs.
RUST_PLAIN = ('', 'rust')

# Not Rust, and claiming not to be. rustdoc never compiles these, which is
# honest rather than evasive -- the block is prose in a box.
LANGUAGE_TAGS = ('text', 'js', 'ts', 'jsx', 'tsx', 'css', 'html', 'json',
                 'sh', 'bash', 'powershell', 'toml', 'yaml', 'diff', 'console')

# The floor on doc-comment fences found anywhere under `crates/`. 17 exist as
# this is written (12 component examples, 1 umbrella facade example, 4 prose
# blocks in the behaviour tests). A scan that reads fewer than this has most
# likely stopped reading doc comments at all, and its "no ```ignore anywhere"
# verdict would be an artefact of finding nothing.
TOTAL_FENCE_FLOOR = 15

# The floor on fences rustdoc will actually *compile* -- the number this audit
# protects. Deleting the twelve examples instead of fixing them would satisfy
# every rule above while leaving the crate with no checked examples, so the
# count of compiled examples is asserted directly. 13 exist as this is written.
COMPILED_FENCE_FLOOR = 12

# The floor on Rust files walked. 146 exist as this is written; a walker that
# finds a handful has lost a directory.
RS_FILE_FLOOR = 100


def rust_files(root):
    """Every `.rs` file under `root`, sorted, skipping build output."""
    found = []
    for base, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d != 'target' and not d.startswith('.')]
        for name in names:
            if name.endswith('.rs'):
                found.append(os.path.join(base, name).replace(os.sep, '/'))
    found.sort()
    return found


def doc_fences(source):
    """`(line number, info string)` for every doc-comment code block opener.

    Only `//!` and `///` lines are read, so a fence inside ordinary code, a
    `//` comment or a string literal is not a doc example and not this audit's
    business. Openers and closers alternate, which is how a closing ` ``` ` is
    told from the next opener without parsing the block between them.
    """
    fences = []
    is_open = False
    for index, line in enumerate(source.split('\n'), 1):
        stripped = line.lstrip()
        rest = None
        for prefix in ('//!', '///'):
            if stripped.startswith(prefix):
                rest = stripped[len(prefix):]
                break
        if rest is None:
            continue
        body = rest.lstrip()
        if not body.startswith('```'):
            continue
        if is_open:
            is_open = False
            continue
        is_open = True
        fences.append((index, body[3:].strip()))
    return fences


def tokens_of(info):
    """A fence's info string as rustdoc reads it: an attribute list.

    rustdoc splits on commas and whitespace, so ` ```rust,ignore ` carries both
    `rust` and `ignore`. Matching the whole string against `"ignore"` would
    miss that, and miss ` ```ignore-macos ` too.
    """
    return [token for token in re.split(r'[,\s]+', info.strip()) if token]


def is_ignore(token):
    """rustdoc skips the block for `ignore` and for any `ignore-<target>`."""
    lowered = token.lower()
    return lowered == 'ignore' or lowered.startswith('ignore-')


def classify(path, info):
    """`(kind, verdict, detail)` for one fence.

    `kind` is `'compiled'` when rustdoc will build the block, `'prose'` when it
    is a declared non-Rust block. `verdict` is `None` when the fence is
    allowed, otherwise the failure headline.
    """
    tokens = tokens_of(info)

    for token in tokens:
        if is_ignore(token):
            return 'ignored', 'IGNORE', (
                'rustdoc never compiles a `%s` block, so nothing keeps it '
                'true. There is no allowlist for this. Make the example '
                'compile (`gpui::TestAppContext::single()` plus '
                '`add_window_view`, with `ThemeProvider::init` first -- the '
                'twelve converted examples under '
                'crates/herogpui-components/src/ are the worked shape), or '
                'delete it: an example whose prose was doing the work loses '
                'nothing by going.' % info)

    if not tokens:
        return 'compiled', None, ''

    # A single language tag is prose in a box.
    if len(tokens) == 1 and tokens[0].lower() in LANGUAGE_TAGS:
        return 'prose', None, ''

    kind = 'compiled'
    verdict = None
    detail = ''
    for token in tokens:
        lowered = token.lower()
        if lowered in RUST_PLAIN:
            continue
        if lowered in ALLOWLISTS:
            name, allowlist, why = ALLOWLISTS[lowered]
            if path not in allowlist:
                verdict = verdict or lowered.upper()
                detail = detail or (
                    'is a ```%s example, but %s has no entry in `%s` in '
                    'fence_audit.py. %s' % (lowered, path, name, why))
            continue
        verdict = verdict or 'UNKNOWN'
        detail = detail or (
            'carries the fence attribute `%s`, which this repository has no '
            'rule for. Either it is a typo, or it is a rustdoc attribute '
            'worth a deliberate decision -- add it to `LANGUAGE_TAGS`, or '
            'give it an allowlist in fence_audit.py.' % token)
    return kind, verdict, detail


def scan(files):
    """`(failures, counts, seen)` over `files`.

    `seen` maps an allowlisted attribute to the paths that used it, which is
    what makes a stale allowlist entry detectable.
    """
    failures = []
    counts = {'total': 0, 'compiled': 0, 'prose': 0}
    seen = {name: set() for name in ALLOWLISTS}

    for path in files:
        source = io.open(path, encoding='utf-8', errors='replace').read()
        for line, info in doc_fences(source):
            counts['total'] += 1
            kind, verdict, detail = classify(path, info)
            if kind in counts:
                counts[kind] += 1
            for token in tokens_of(info):
                lowered = token.lower()
                if lowered in seen:
                    seen[lowered].add(path)
            if verdict:
                failures.append((verdict, path, line, info, detail))
    return failures, counts, seen


def stale_entries(seen):
    """Allowlist entries whose file no longer carries the fence they excuse."""
    stale = []
    for attribute, (name, allowlist, _) in sorted(ALLOWLISTS.items()):
        for path, reason in sorted(allowlist.items()):
            if path not in seen[attribute]:
                stale.append((name, attribute, path, reason))
    return stale


def main():
    files = rust_files(SCAN_ROOT)
    if len(files) < RS_FILE_FLOOR:
        raise SystemExit(
            'FENCE SCAN READ NOTHING: walked %s/ and found %d Rust files, '
            'expected at least %d -- the walker is broken, and any verdict it '
            'reaches is meaningless.' % (SCAN_ROOT, len(files), RS_FILE_FLOOR))

    failures, counts, seen = scan(files)
    stale = stale_entries(seen)

    for verdict, path, line, info, detail in failures:
        print('%-8s %s:%d  ```%s' % (verdict, path, line, info or '(bare)'))
        print('         %s' % detail)
    for name, attribute, path, reason in stale:
        print('STALE    `%s` still allows `%s` (%s), but that file has no '
              '```%s example any more. Delete the entry.'
              % (name, path, reason, attribute))
    if failures or stale:
        print()

    print('rust files walked         : %d' % len(files))
    print('doc-comment fences        : %d' % counts['total'])
    print('compiled by rustdoc       : %d' % counts['compiled'])
    print('declared non-Rust prose   : %d' % counts['prose'])
    print('IGNORED (never compiled)  : %d'
          % sum(1 for f in failures if f[0] == 'IGNORE'))
    print('unjustified / unknown     : %d'
          % sum(1 for f in failures if f[0] != 'IGNORE'))
    print('stale allowlist entries   : %d' % len(stale))

    # A parser that silently found nothing reports success. These two floors
    # are what make the verdict above mean something.
    if counts['total'] < TOTAL_FENCE_FLOOR:
        raise SystemExit(
            '\nFENCE SCAN READ NOTHING: found only %d doc-comment fences '
            'under %s/, expected at least %d. The scanner is reading nothing, '
            'and its "no ```ignore anywhere" verdict means nothing either.'
            % (counts['total'], SCAN_ROOT, TOTAL_FENCE_FLOOR))
    if counts['compiled'] < COMPILED_FENCE_FLOOR:
        raise SystemExit(
            '\nCOMPILED EXAMPLES LOST: only %d fences under %s/ are ones '
            'rustdoc will compile, expected at least %d. Examples were '
            'deleted or downgraded to prose rather than kept true -- which '
            'satisfies every other rule here and defeats the point.'
            % (counts['compiled'], SCAN_ROOT, COMPILED_FENCE_FLOOR))

    if failures or stale:
        return 1
    return 0


def self_test():
    """Known-positive and known-negative proof for the scanner.

    Without this, "found no ```ignore anywhere" could just mean the scan never
    saw a fence, or that it saw them and misread every one.
    """
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    # --- the parser reads doc fences, and only doc fences -------------------
    source = '\n'.join([
        '//! ```',
        '//! let module_level = 1;',
        '//! ```',
        '//!',
        '//! ```text',
        '//! not rust',
        '//! ```',
        '',
        '// ``` a plain comment fence is not a doc example',
        '/// ```no_run',
        '/// let item_level = 2;',
        '/// ```',
        'fn documented() {',
        '    let s = "``` a fence inside a string literal";',
        '}',
    ])
    expect(doc_fences(source) == [(1, ''), (5, 'text'), (10, 'no_run')],
           'the parser should report three openers -- bare, text, no_run -- '
           'and neither the closers, the `//` comment, nor the string '
           'literal: %r' % (doc_fences(source),))

    # An indented fence inside a method's doc comment is the shape this
    # repository actually uses.
    indented = '    /// ```ignore\n    /// let x = 1;\n    /// ```'
    expect(doc_fences(indented) == [(1, 'ignore')],
           'the parser missed an indented `///` fence: %r'
           % (doc_fences(indented),))

    # --- the synthetic bad fence is caught ---------------------------------
    kind, verdict, detail = classify('crates/x/src/a.rs', 'ignore')
    expect(verdict == 'IGNORE' and kind == 'ignored',
           'a synthetic ```ignore fence was not flagged: %r'
           % ((kind, verdict),))
    expect('no allowlist' in detail,
           'the ```ignore failure should say there is no allowlist: %r'
           % detail)

    # The evasions that read as `ignore` to rustdoc.
    for info in ('rust,ignore', 'ignore-macos', ' ignore ', 'IGNORE',
                 'no_run,ignore', 'rust ignore'):
        expect(classify('crates/x/src/a.rs', info)[1] == 'IGNORE',
               'the fence ```%s is skipped by rustdoc but was not flagged'
               % info)

    # --- known negatives ---------------------------------------------------
    for info in ('', 'rust', 'text', 'js', 'css'):
        kind, verdict, _ = classify('crates/x/src/a.rs', info)
        expect(verdict is None,
               'the fence ```%s is legitimate but was flagged %s'
               % (info or '(bare)', verdict))
    expect(classify('crates/x/src/a.rs', '')[0] == 'compiled',
           'a bare fence is compiled by rustdoc')
    expect(classify('crates/x/src/a.rs', 'text')[0] == 'prose',
           'a `text` fence is declared non-Rust prose')
    expect(classify('crates/x/src/a.rs', 'rust')[0] == 'compiled',
           'a `rust` fence is compiled by rustdoc')

    # An unrecognised attribute is rejected by name rather than waved through.
    expect(classify('crates/x/src/a.rs', 'edition2015')[1] == 'UNKNOWN',
           'an unknown fence attribute should be rejected by name')

    # --- the allowlists gate, rather than decorate -------------------------
    expect(classify('crates/x/src/a.rs', 'no_run')[1] == 'NO_RUN',
           '`no_run` with no allowlist entry should be flagged')
    NO_RUN['crates/x/src/a.rs'] = 'a self-test entry'
    try:
        expect(classify('crates/x/src/a.rs', 'no_run')[1] is None,
               '`no_run` with an allowlist entry should be allowed')
        expect(classify('crates/y/src/b.rs', 'no_run')[1] == 'NO_RUN',
               'an allowlist entry must only cover its own file')
        # ... and a stale entry fails rather than rots.
        expect(stale_entries({'no_run': {'crates/x/src/a.rs'},
                              'compile_fail': set(),
                              'should_panic': set()}) == [],
               'a live allowlist entry must not be reported stale')
        stale = stale_entries({'no_run': set(), 'compile_fail': set(),
                               'should_panic': set()})
        expect([entry[2] for entry in stale] == ['crates/x/src/a.rs'],
               'an allowlist entry whose file no longer has a ```no_run '
               'example must be reported stale: %r' % (stale,))
    finally:
        del NO_RUN['crates/x/src/a.rs']

    # --- the floors are real ------------------------------------------------
    empty_failures, empty_counts, _ = scan([])
    expect(empty_failures == [] and empty_counts['total'] == 0,
           'scanning nothing should find nothing -- which is exactly why the '
           'floors exist')
    expect(TOTAL_FENCE_FLOOR > 0 and COMPILED_FENCE_FLOOR > 0
           and RS_FILE_FLOOR > 0,
           'a floor of zero is not a floor')

    # --- the real tree still holds the examples this audit was made for ----
    files = rust_files(SCAN_ROOT)
    _, counts, _ = scan(files)
    expect(counts['compiled'] >= COMPILED_FENCE_FLOOR,
           'the scan found %d compiled fences in the real tree, below the '
           'floor of %d' % (counts['compiled'], COMPILED_FENCE_FLOOR))

    if failures:
        print('self-test FAIL')
        for failure in failures:
            print('- %s' % failure)
        return 1
    print('self-test PASS: the scanner reads doc fences and only doc fences, '
          'flags ```ignore and every attribute list rustdoc reads as ignore '
          '(`rust,ignore`, `ignore-macos`), rejects unknown attributes by '
          'name, gates `no_run` on a per-file allowlist, reports a stale '
          'allowlist entry, and refuses to call an empty scan a pass.')
    return 0


if __name__ == '__main__':
    if '--self-test' in sys.argv[1:]:
        sys.exit(self_test())
    # The audit's verdict is worth what the parser behind it is worth, so the
    # self-test runs first rather than only behind the flag.
    if self_test() != 0:
        sys.exit(1)
    print()
    sys.exit(main())
