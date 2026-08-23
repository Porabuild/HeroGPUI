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
            # Destructuring / shorthand init also counts as neither.
            reads = uses - writes
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
