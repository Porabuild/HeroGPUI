"""For each reported gap, print which v3 table it came from and its description.

Tells apart a real parent prop from a render-prop argument that only exists
inside a child callback.
"""
import io, os, re, sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bundle import resolve as _resolve_bundle

# The pinned v3.2.4 bundle. See .shots/bundle.py: reading upstream live would
# measure this port against whatever HeroUI shipped most recently.
BUNDLE = _resolve_bundle()
bundle = io.open(BUNDLE, encoding='utf-8', errors='replace').read()

WANT = {}
for arg in sys.argv[1:]:
    comp, props = arg.split('=', 1)
    WANT[comp] = props.split(',')

for comp, props in WANT.items():
    print('=' * 70)
    print(comp)
    pattern = r'^### (%s(?:\.[A-Za-z]+)?)\s*$' % re.escape(comp)
    for m in re.finditer(pattern, bundle, re.M):
        heading = m.group(1)
        chunk = bundle[m.end():m.end() + 4000]
        nxt = re.search(r'^### ', chunk, re.M)
        if nxt:
            chunk = chunk[:nxt.start()]
        for row in re.finditer(r'^\|\s*`([a-zA-Z-]+)`\s*\|([^\n]*)$', chunk, re.M):
            if row.group(1) in props:
                desc = re.sub(r'\s+', ' ', row.group(2)).strip()
                print('  %-24s %-22s %s' % (heading, row.group(1), desc[:150]))
