"""Find props that are stored but never read.

A field that a builder assigns and nothing ever reads is worse than a missing
one: the API promises behaviour it does not have. This counts, per component
struct, the field's declaration, its initialiser, its builder assignment and
every other mention. A field whose only mentions are those three is write-only.

Known blind spot: `self.<field>` is matched across the whole module, so when two
structs in one file share a field name, a read in either satisfies both. That
hid an unwired `SearchField::validate` sitting beside `Input::validate`. Shared
names are listed at the end so they get checked by hand instead of passing
silently.

A `RenderOnce` render that takes `self` by value reads its fields by
destructuring — `let Self { validation_errors: record, .. } = self` never
spells `self.validation_errors` — so a destructuring that BINDS the field
counts as a read: the shorthand (`field,` or a trailing `field }`) and the
renamed binding (`field: record`) both move the field out for the body that
follows. `field: _` binds nothing and reads nothing, so it does not count —
an explicit ignore is exactly how a write-only field would hide.
"""
import io
import re
import sys
import glob

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SRC = 'crates/herogpui-components/src/'
findings = []
seen = {}

for path in sorted(glob.glob(SRC + '*.rs')):
    name = path.replace('\\', '/').split('/')[-1]
    src = io.open(path, encoding='utf-8').read()

    # Component structs only: those that derive IntoElement are the builders
    # whose fields are the public API surface.
    for m in re.finditer(
        r'#\[derive\([^)]*IntoElement[^)]*\)\]\s*pub struct (\w+)\s*\{(.*?)\n\}',
        src, re.S
    ):
        struct, body = m.group(1), m.group(2)
        fields = re.findall(r'^\s{4}(?:pub )?([a-z_][a-z_0-9]*)\s*:', body, re.M)
        for f in fields:
            # `self.f` in a builder assignment is a write; anything else is a read.
            # `=(?![=>])` so neither a comparison (`self.f == x`) nor a match
            # arm (`self.f => ...`) is mistaken for an assignment.
            writes = len(re.findall(r'self\s*\.\s*%s\s*=(?![=>])' % re.escape(f), src))
            # `self\s*\.\s*` so a wrapped builder chain still counts as a read.
            uses = len(re.findall(r'self\s*\.\s*%s\b' % re.escape(f), src))
            # A binding destructuring counts as a read. For field F the three
            # accepted shapes after `let Self { ... F` are, explicitly:
            #   `F,` / `F }`  — shorthand binding, `\s*[,}]`
            #   `F: binding`  — renamed binding, `:\s*` + an identifier that
            #                   is not exactly `_` (the `(?!_(?![A-Za-z0-9_]))`
            #                   lookahead rejects a lone underscore while
            #                   still accepting `_`-prefixed names, which do
            #                   bind and can be read)
            #   `F: _`        — matches neither branch: an ignore, not a read
            destructures = len(re.findall(
                r'let Self \{[^}]*\b%s(?:\s*[,}]|\s*:\s*(?!_(?![A-Za-z0-9_]))[A-Za-z_]\w*)'
                % re.escape(f), src))
            reads = uses - writes + destructures
            seen.setdefault((name, f), set()).add(struct)
            if reads == 0:
                findings.append('%-22s %-22s %s' % (name, struct, f))

shared = ['%-22s %-20s %s' % (n, f, ', '.join(sorted(sts)))
          for (n, f), sts in sorted(seen.items()) if len(sts) > 1]

if findings:
    print('write-only fields (%d):' % len(findings))
    print('\n'.join(findings))
    sys.exit(1)

print('no write-only fields')
if shared:
    print()
    print('shared field names (counted module-wide -- verify by hand):')
    print('\n'.join(shared))
