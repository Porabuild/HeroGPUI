"""An element id that is the same for every instance of a component is not an
id -- it is a collision waiting for a second instance.

gpui keys two separate things on an element's *whole* id path (its
`GlobalElementId`):

  * **Element state.** `Window::with_element_state`, and therefore
    `Window::use_keyed_state` and every `util` helper built on it, stores
    per-element state under that path. Two elements with the same path share
    one slot, silently. `util::tab_stop_handle` keeps a *focus handle* there,
    so a duplicate path is two controls with one focus.
  * **Accessibility node ids.** gpui hashes the path into an
    `accesskit::NodeId`. A duplicate path collides two a11y nodes. That half
    only bites once an element also reports a role, which is why a duplicate id
    can sit in this crate looking harmless today -- and why the ids get scoped
    before the roles get added, not after.

Almost every component here is a `RenderOnce`. A `RenderOnce` is inlined into
its parent's element tree and pushes *nothing* onto the id path, and the
`use_keyed_state` calls at the top of `render` run before the component's own
root `div().id(..)` exists. So a component's keys are effectively
window-global: "is this id unique?" cannot be answered by reading the
component, which is why deriving it from the caller-supplied id is the rule
rather than a preference.

| Shape | Verdict |
|---|---|
| `element_id::scoped(&self.id, "part")` | the rule |
| `element_id::indexed(&self.id, "row", i)` | the rule, for a repeated part |
| `.id(self.id.clone())`, `ElementId::named_usize(name, entity_id)` | derived |
| `.id("literal")`, `ElementId::Name("literal".into())` | CONSTANT -- rejected |
| `ElementId::Name(format!(..))`, `.id(format!(..))` | FORMAT -- rejected |

`format!` is rejected for a reason separate from constancy: the separator is
not reserved, so `format!("{parent}-{part}")` with parent `"a-b"` and part
`"c"` produces the identical key to parent `"a"` with part `"b-c"`. Two
different elements, one state slot, no error. `ElementId::NamedChild` keeps the
parent as a compared-and-hashed field of its own, so those two cannot fold
together -- and it allocates nothing per frame. See
`crates/herogpui-core/src/element_id.rs`.

Scope: `crates/*/src/**.rs` only. A *caller* is entitled to a constant --
`gallery/` naming its own demo elements is correct use of the override API, and
so is a behaviour test in `crates/herogpui-components/tests/`. Top-level
`#[cfg(test)] mod` blocks inside a source file are skipped for the same reason.

What this cannot see: an id built into a local and passed on
(`let key = format!(..); div().id(key)`), and whether an
`(name, index)` sibling id has a properly scoped parent. Both stay review
questions. The shapes it does read are the ones that produced every collision
the `element_id` module was written for.

Run from the repository root:

    python3 .shots/element_id_audit.py
    python3 .shots/element_id_audit.py --self-test

The scanner's own self-test runs as part of the audit, not only behind the
flag: a parser that silently matches nothing reports a clean tree, and so does
a clean tree.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SCAN_ROOT = 'crates'

# (path, literal) -> why this component is allowed to mint a constant id.
#
# Every entry is asserted to still match a constant id with that exact literal
# in that exact file, so an entry whose site was fixed or moved fails this
# audit until the entry goes too.
CONSTANT_ALLOWED = {
    ('crates/herogpui-components/src/util.rs', 'herogpui-focus-root'):
        'there is exactly one app focus root per window by construction -- '
        '`util::app_focus_root` wraps the window\'s root element -- so the '
        'singleton is the contract rather than an accident. A second one on '
        'screen would be the bug, and it would be a bug about focus roots, '
        'not about ids.',
    ('crates/herogpui-components/src/modal.rs', 'modal'):
        'the default seed for the *component\'s own* id in `Modal::new()`, '
        'which `.id(..)` overrides. Clause 1 of the rule -- an element that '
        'stands on its own takes its id from its caller -- is satisfied by '
        '`.id(..)`; this is the fallback for a caller that renders one modal '
        'and never names it. Every *part* of the modal now derives from '
        '`self.id`, so overriding it moves the whole subtree.',
    ('crates/herogpui-components/src/alert_dialog.rs', 'alert-dialog'):
        'the default seed for `AlertDialog`\'s own id; see the `modal` entry.',
    ('crates/herogpui-components/src/drawer.rs', 'drawer'):
        'the default seed for `Drawer`\'s own id; see the `modal` entry.',
    ('crates/herogpui-components/src/popover.rs', 'popover'):
        'the default seed for `Popover`\'s own id; see the `modal` entry.',
    ('crates/herogpui-components/src/toolbar.rs', 'toolbar-focus'):
        '`Toolbar::id` is `Option<ElementId>` and this is the fallback for a '
        'toolbar the caller never named. Two unnamed toolbars on one screen '
        'do share this focus scope -- a real gap, recorded here rather than '
        'hidden, and fixable only by making the id required, which is a '
        'public-API change of its own.',
    ('crates/herogpui-components/src/toast.rs', 'toast-region'):
        'the fallback seed for `ToastViewport`\'s own id, which `.id(..)` '
        'overrides; see the `modal` entry. Every other id in the viewport -- '
        'pointer state, focus handle, hotkey interceptor, and each toast\'s '
        'parts -- derives from it, so a named viewport moves its whole '
        'subtree.',
}

# (path, part-of-the-line) -> why this id may still be built with `format!`.
#
# Empty, and worth keeping that way: every `format!`ed id in `crates/` was
# migrated to `element_id::scoped` / `element_id::indexed`. An entry here would
# be a component that cannot name its own parts structurally, which is a
# component to fix rather than to excuse.
FORMAT_ALLOWED = {}

# The floor on id sites found. A parser that silently stops matching finds no
# offenders, which is indistinguishable from a clean tree; so is a walker that
# lost the source directory. 500+ sites exist as this is written, across
# `.id(..)`, `ElementId::` constructors and the `element_id` helpers. The floor
# is far below that and only has to be non-trivial.
ID_SITE_FLOOR = 300

# The floor on uses of the structured helpers themselves -- the count this
# audit exists to protect. Reverting the migration wholesale would satisfy
# "no constant ids" for any component that went back to `format!`... which the
# FORMAT rule catches; but *deleting* the ids outright would satisfy both. 300+
# exist as this is written.
STRUCTURED_FLOOR = 200

# The floor on Rust files walked under `crates/*/src`.
RS_FILE_FLOOR = 40

# `.id("literal")` and `ElementId::Name("literal".into())` -- an element naming
# itself. `\s*` spans the line breaks rustfmt puts inside a long call.
CONSTANT_PATTERNS = (
    re.compile(r'\.id\(\s*"([^"\\]*)"\s*\)'),
    re.compile(r'ElementId::Name\(\s*"([^"\\]*)"'),
    re.compile(r'ElementId::NamedInteger\(\s*"([^"\\]*)"\s*\.into\(\)\s*,\s*\d+\s*\)'),
)

# An id assembled by formatting a string. `el_name` is this crate's local
# `String -> ElementId::Name` wrapper, which hides the same thing.
FORMAT_PATTERNS = (
    re.compile(r'\.id\(\s*format!'),
    re.compile(r'ElementId::Name\(\s*format!'),
    re.compile(r'ElementId::NamedInteger\(\s*format!'),
    re.compile(r'ElementId::named_usize\(\s*format!'),
    re.compile(r'el_name\(\s*format!'),
)

# Everything that mints or passes an id, for the floor. Not a rule -- a
# liveness check on the parser and the walk.
ID_SITE_PATTERN = re.compile(
    r'\.id\(|ElementId::Name\(|ElementId::NamedInteger\(|ElementId::NamedChild\(|'
    r'ElementId::named_usize\(|ElementId::Integer\(|el_name\(|'
    r'element_id::scoped\(|element_id::indexed\(')

# The two helpers the migration moved everything onto.
STRUCTURED_PATTERN = re.compile(r'element_id::scoped\(|element_id::indexed\(')


def rust_files(root):
    """Every `.rs` file under a `<crate>/src` directory below `root`."""
    found = []
    for base, dirs, names in os.walk(root):
        dirs[:] = [d for d in dirs if d != 'target' and not d.startswith('.')]
        normalised = base.replace(os.sep, '/')
        parts = normalised.split('/')
        if 'src' not in parts:
            continue
        for name in names:
            if name.endswith('.rs'):
                found.append('%s/%s' % (normalised, name))
    found.sort()
    return found


def strip_comments(source):
    """`source` with comments blanked out, string literals left intact.

    Blanked rather than deleted so every byte offset still maps to its
    original line. A prose mention of `.id("close")` in a doc comment -- this
    audit's own rationale is full of them -- must not read as code, and a `//`
    inside a string literal must not read as a comment.
    """
    out = []
    index = 0
    length = len(source)
    while index < length:
        char = source[index]
        if char == '"':
            out.append(char)
            index += 1
            while index < length:
                out.append(source[index])
                if source[index] == '\\':
                    if index + 1 < length:
                        out.append(source[index + 1])
                        index += 2
                        continue
                elif source[index] == '"':
                    index += 1
                    break
                index += 1
            continue
        if source.startswith('//', index):
            while index < length and source[index] != '\n':
                out.append(' ')
                index += 1
            continue
        if source.startswith('/*', index):
            while index < length and not source.startswith('*/', index):
                out.append('\n' if source[index] == '\n' else ' ')
                index += 1
            out.append('  ')
            index += 2
            continue
        out.append(char)
        index += 1
    return ''.join(out)


def strip_test_modules(source):
    """`source` with top-level `#[cfg(test)]` blocks blanked out.

    A *caller* is entitled to a constant id: `Button::new("left")` in a
    behaviour test names its own element rather than the component naming
    itself. Found by column -- an attribute in column 0 opens the block and a
    `}` in column 0 closes it -- which holds for rustfmt-formatted source.
    Blanked line by line so offsets still map to their original lines.
    """
    kept = []
    inside = False
    for line in source.split('\n'):
        if inside:
            kept.append('')
            if line == '}':
                inside = False
            continue
        if line.startswith('#[cfg(test)]'):
            inside = True
            kept.append('')
            continue
        kept.append(line)
    return '\n'.join(kept)


def readable(source):
    """The part of a file this audit reads: code, outside test modules."""
    return strip_test_modules(strip_comments(source))


def line_of(source, offset):
    return source.count('\n', 0, offset) + 1


def findings(path, source):
    """`(failures, sites, structured)` for one file's readable source."""
    body = readable(source)
    failures = []

    for pattern in CONSTANT_PATTERNS:
        for match in pattern.finditer(body):
            literal = match.group(1)
            if (path, literal) in CONSTANT_ALLOWED:
                continue
            failures.append(('CONSTANT', path, line_of(body, match.start()),
                             match.group(0).split('\n')[0].strip(), literal))

    for pattern in FORMAT_PATTERNS:
        for match in pattern.finditer(body):
            spelling = match.group(0).split('\n')[0].strip()
            if any(path == entry[0] and entry[1] in spelling
                   for entry in FORMAT_ALLOWED):
                continue
            failures.append(('FORMAT', path, line_of(body, match.start()),
                             spelling, None))

    sites = len(ID_SITE_PATTERN.findall(body))
    structured = len(STRUCTURED_PATTERN.findall(body))
    return failures, sites, structured


def scan(files):
    """`(failures, counts, seen)` over `files`.

    `seen` records which allowlisted constants were actually found, which is
    what makes a stale allowlist entry detectable.
    """
    failures = []
    counts = {'sites': 0, 'structured': 0, 'constant': 0, 'format': 0}
    seen = {'constant': set(), 'format': set()}

    for path in files:
        source = io.open(path, encoding='utf-8', errors='replace').read()
        body = readable(source)
        for pattern in CONSTANT_PATTERNS:
            for match in pattern.finditer(body):
                key = (path, match.group(1))
                if key in CONSTANT_ALLOWED:
                    seen['constant'].add(key)
        for pattern in FORMAT_PATTERNS:
            for match in pattern.finditer(body):
                spelling = match.group(0)
                for entry in FORMAT_ALLOWED:
                    if path == entry[0] and entry[1] in spelling:
                        seen['format'].add(entry)

        file_failures, sites, structured = findings(path, source)
        failures.extend(file_failures)
        counts['sites'] += sites
        counts['structured'] += structured

    counts['constant'] = sum(1 for f in failures if f[0] == 'CONSTANT')
    counts['format'] = sum(1 for f in failures if f[0] == 'FORMAT')
    failures.sort(key=lambda f: (f[1], f[2]))
    return failures, counts, seen


def stale_entries(seen):
    """Allowlist entries whose site no longer exists."""
    stale = []
    for entry, reason in sorted(CONSTANT_ALLOWED.items()):
        if entry not in seen['constant']:
            stale.append(('CONSTANT_ALLOWED', entry, reason))
    for entry, reason in sorted(FORMAT_ALLOWED.items()):
        if entry not in seen['format']:
            stale.append(('FORMAT_ALLOWED', entry, reason))
    return stale


CONSTANT_ADVICE = (
    'derive it from the component\'s caller-supplied id: '
    '`element_id::scoped(&self.id, "part")`, or '
    '`element_id::indexed(&self.id, "part", i)` for one of a repeated part. '
    'A component with no id to derive from needs one before it can have '
    'parts. If the constant really is a per-window singleton, add an entry to '
    '`CONSTANT_ALLOWED` in element_id_audit.py saying why.')

FORMAT_ADVICE = (
    'a formatted id flattens its parent and its part into one string, where '
    'parent "a-b" + part "c" and parent "a" + part "b-c" are the same key. '
    'Use `element_id::scoped` / `element_id::indexed`, which keep the parent '
    'as structure. See crates/herogpui-core/src/element_id.rs.')


def main():
    files = rust_files(SCAN_ROOT)
    if len(files) < RS_FILE_FLOOR:
        raise SystemExit(
            'ELEMENT ID SCAN READ NOTHING: walked %s/ and found %d Rust '
            'source files, expected at least %d -- the walker is broken, and '
            'any verdict it reaches is meaningless.'
            % (SCAN_ROOT, len(files), RS_FILE_FLOOR))

    failures, counts, seen = scan(files)
    stale = stale_entries(seen)

    for verdict, path, line, spelling, _literal in failures:
        print('%-8s %s:%d  %s' % (verdict, path, line, spelling))
        print('         %s'
              % (CONSTANT_ADVICE if verdict == 'CONSTANT' else FORMAT_ADVICE))
    for name, entry, reason in stale:
        print('STALE    `%s` still allows %r in %s (%s), but nothing there '
              'matches any more. Delete the entry.'
              % (name, entry[1], entry[0], reason))
    if failures or stale:
        print()

    print('rust source files walked  : %d' % len(files))
    print('element id sites read     : %d' % counts['sites'])
    print('structured helper uses    : %d' % counts['structured'])
    print('allowlisted constants     : %d' % len(CONSTANT_ALLOWED))
    print('CONSTANT ids              : %d' % counts['constant'])
    print('FORMAT-built ids          : %d' % counts['format'])
    print('stale allowlist entries   : %d' % len(stale))

    if counts['sites'] < ID_SITE_FLOOR:
        raise SystemExit(
            '\nELEMENT ID SCAN READ NOTHING: found only %d id sites under '
            '%s/, expected at least %d. The scanner is reading nothing, and '
            'its "no constant ids anywhere" verdict means nothing either.'
            % (counts['sites'], SCAN_ROOT, ID_SITE_FLOOR))
    if counts['structured'] < STRUCTURED_FLOOR:
        raise SystemExit(
            '\nSTRUCTURED IDS LOST: only %d uses of `element_id::scoped` / '
            '`element_id::indexed` remain under %s/, expected at least %d. '
            'The derived ids were deleted rather than kept -- which satisfies '
            'every other rule here and defeats the point.'
            % (counts['structured'], SCAN_ROOT, STRUCTURED_FLOOR))

    if failures or stale:
        return 1
    return 0


SELF_TEST_SOURCE = '''
use herogpui_core::element_id;

impl RenderOnce for Widget {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A prose mention of .id("in-a-comment") is not code.
        let label = "a // slash in a string is not a comment";
        div()
            .id("textarea")
            .child(div().id(ElementId::Name("nested".into())))
            .child(div().id(format!("{:?}-part", self.id)))
            .child(div().id(element_id::scoped(&self.id, "fine")))
            .child(div().id(element_id::indexed(&self.id, "row", 3)))
            .child(div().id(self.id.clone()))
            .child(div().id(ElementId::named_usize("live", entity)))
            .child(el_name(format!("{base}-hidden")))
    }
}

fn multiline(self) -> gpui::ElementId {
    gpui::ElementId::Name(
        format!("{:?}-wrapped", self.id).into(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_caller_may_name_its_own_element() {
        Button::new("left").id("right");
        div().id(format!("caller-{i}"));
    }
}
'''


def self_test():
    """Known-positive and known-negative proof for the scanner.

    Without it, "no constant or formatted ids anywhere" could equally mean the
    scan never matched anything, or matched everything and misread it.
    """
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    path = 'crates/x/src/a.rs'
    found, sites, structured = findings(path, SELF_TEST_SOURCE)
    by_verdict = {}
    for verdict, _path, line, spelling, _literal in found:
        by_verdict.setdefault(verdict, []).append((line, spelling))

    # --- the synthetic constant ids are caught -----------------------------
    constants = sorted(by_verdict.get('CONSTANT', []))
    expect([line for line, _ in constants] == [9, 10],
           'the two synthetic constant ids -- `.id("textarea")` and '
           '`.id(ElementId::Name("nested".into()))` -- should be flagged and '
           'nothing else: %r' % (constants,))

    # --- the synthetic formatted ids are caught, including the wrapped one --
    formats = sorted(by_verdict.get('FORMAT', []))
    expect([line for line, _ in formats] == [11, 16, 21],
           'the three synthetic formatted ids -- `.id(format!(..))`, '
           '`el_name(format!(..))` and the rustfmt-wrapped '
           '`ElementId::Name(\\n format!(..))` -- should all be flagged: %r'
           % (formats,))
    expect(len(found) == 5,
           'the scan should find exactly the five synthetic offenders: %r'
           % (found,))

    # --- known negatives ---------------------------------------------------
    flagged_lines = {line for _v, _p, line, _s, _l in found}
    for line, why in ((6, 'a doc/line comment mentioning .id("..")'),
                      (7, 'a string literal containing `//`'),
                      (12, 'element_id::scoped'),
                      (13, 'element_id::indexed'),
                      (14, '.id(self.id.clone())'),
                      (15, 'ElementId::named_usize with a live integer'),
                      (30, 'a caller naming its own element in a test module'),
                      (31, 'a caller formatting its own id in a test module')):
        expect(line not in flagged_lines,
               'line %d (%s) is legitimate but was flagged' % (line, why))

    expect(sites >= 9,
           'the id-site counter should see every mint and pass in the sample, '
           'got %d' % sites)
    expect(structured == 2,
           'the sample uses `scoped` once and `indexed` once, got %d'
           % structured)

    # --- the comment and test-module strippers keep line numbers -----------
    expect(readable('a\n// b\nc').split('\n')[2] == 'c',
           'blanking a comment must not move the lines after it')
    expect(len(readable(SELF_TEST_SOURCE).split('\n'))
           == len(SELF_TEST_SOURCE.split('\n')),
           'the readable source must have the same number of lines as the '
           'original, or every reported line number is wrong')

    # --- the allowlists gate, rather than decorate -------------------------
    expect(findings(path, 'div().id("solo")')[0][0][0] == 'CONSTANT',
           'an unallowlisted constant should be flagged')
    CONSTANT_ALLOWED[(path, 'solo')] = 'a self-test entry'
    try:
        expect(findings(path, 'div().id("solo")')[0] == [],
               'an allowlisted constant should be allowed')
        expect(findings('crates/y/src/b.rs', 'div().id("solo")')[0][0][0]
               == 'CONSTANT',
               'an allowlist entry must only cover its own file')
        expect(stale_entries({'constant': set(CONSTANT_ALLOWED),
                              'format': set(FORMAT_ALLOWED)}) == [],
               'a live allowlist entry must not be reported stale')
        stale = stale_entries({'constant': set(), 'format': set()})
        expect([entry for _n, entry, _r in stale] == sorted(CONSTANT_ALLOWED),
               'an allowlist entry whose constant is gone must be reported '
               'stale: %r' % (stale,))
    finally:
        del CONSTANT_ALLOWED[(path, 'solo')]

    # --- the floors are real -----------------------------------------------
    empty_failures, empty_counts, _ = scan([])
    expect(empty_failures == [] and empty_counts['sites'] == 0,
           'scanning nothing should find nothing -- which is exactly why the '
           'floors exist')
    expect(ID_SITE_FLOOR > 0 and STRUCTURED_FLOOR > 0 and RS_FILE_FLOOR > 0,
           'a floor of zero is not a floor')

    # --- every allowlist entry still describes a real site -----------------
    _, _, seen = scan(rust_files(SCAN_ROOT))
    missing = sorted(set(CONSTANT_ALLOWED) - seen['constant'])
    expect(not missing,
           'these CONSTANT_ALLOWED entries match nothing in the tree, so the '
           'allowlist has gone stale: %r' % (missing,))

    if failures:
        print('self-test FAIL')
        for failure in failures:
            print('- %s' % failure)
        return 1
    print('self-test PASS: the scanner flags a synthetic constant id '
          '(`.id("textarea")`, `ElementId::Name("nested")`) and a synthetic '
          'formatted id (`.id(format!(..))`, `el_name(format!(..))` and the '
          'rustfmt-wrapped `ElementId::Name(\\n format!(..))`), reads past '
          'comments and string literals without mistaking prose for code, '
          'leaves derived ids and test-module callers alone, gates each '
          'constant on a per-file allowlist entry, reports a stale entry, and '
          'refuses to call an empty scan a pass.')
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
