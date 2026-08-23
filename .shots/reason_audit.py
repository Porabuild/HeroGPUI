"""Re-examine every WONT_PORT entry: which components claim it, and what does
v3 say it does?

An excuse recorded once and never revisited is how a gap hides. This prints, for
each excluded prop, the components that document it and the doc row, so each
reason can be checked against what the prop actually is.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

BUNDLE = os.environ.get('HEROUI_BUNDLE',
                        os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt'))
bundle = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
audit = io.open('.shots/api_audit.py', encoding='utf-8').read()

FILES = eval(re.search(r'^FILES = \{.*?^\}', audit, re.M | re.S).group(0)[8:])
WONT = eval(re.search(r'^WONT_PORT = \{.*?^\}', audit, re.M | re.S).group(0)[12:])

only = set(sys.argv[1:])

# prop -> {component: description}
where = {}
for comp in FILES:
    pattern = r'^### (%s(?:\.[A-Za-z]+)?)\s*$' % re.escape(comp)
    for m in re.finditer(pattern, bundle, re.M):
        heading = m.group(1)
        chunk = bundle[m.end():m.end() + 4000]
        nxt = re.search(r'^### ', chunk, re.M)
        if nxt:
            chunk = chunk[:nxt.start()]
        for row in re.finditer(r'^\|\s*`([a-zA-Z-]+)`\s*\|([^\n]*)$', chunk, re.M):
            prop, desc = row.group(1), re.sub(r'\s+', ' ', row.group(2)).strip()
            # Skip the translated duplicates.
            if re.search(r'[一-鿿]', desc):
                continue
            where.setdefault(prop, {}).setdefault(comp, (heading, desc))

by_reason = {}
for key, reason in WONT.items():
    prop = key.split('.')[-1]
    scoped = '.' in key
    by_reason.setdefault(reason, []).append((key, prop, scoped))

for reason in sorted(by_reason):
    if only and reason not in only:
        continue
    print('=' * 78)
    print('REASON: %s' % reason)
    for key, prop, scoped in sorted(by_reason[reason]):
        comps = where.get(prop, {})
        if scoped:
            comp = key.split('.')[0]
            comps = {comp: comps[comp]} if comp in comps else {}
        print('  %s' % key)
        for comp, (heading, desc) in sorted(comps.items()):
            print('      %-22s %-26s %s' % (comp, heading, desc[:110]))
        if not comps:
            print('      (no matching doc row -- stale entry?)')
