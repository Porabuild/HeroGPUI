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
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import api_audit as A  # noqa: E402  (for the builders each struct actually has)

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
        chunk = bundle[m.end():m.end() + 8000]
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
        else:
            # A scoped entry overrides the bare one in `api_audit`, so a
            # component listed under its own reason must not also appear here --
            # that is how a precise reason gets read as if it were the blanket
            # one.
            comps = {c: v for c, v in comps.items()
                     if WONT.get('%s.%s' % (c, prop), reason) == reason}
        # An excuse for a prop the component now implements is dead weight:
        # `api_audit` never reaches WONT_PORT for it.
        # Only the prop's own spelling counts. A global ALIAS points at a
        # *different* prop (`defaultSelectedKeys` -> `selected_keys`), so
        # following it here would report every controlled builder as proof that
        # the uncontrolled one exists.
        snake = re.sub(r'(?<!^)(?=[A-Z])', '_', prop).lower()
        live = {}
        for c, v in comps.items():
            names = A.impl_methods.get(c, ())
            rust = A.ALIAS.get('%s.%s' % (c, prop), snake)
            if rust in names or prop.lower() in names:
                live[c] = v
        if live and len(live) == len(comps):
            print('  %s -- STALE: every component that documents it implements '
                  'it' % key)
        elif live:
            print('  %s  (implemented by %s; this entry covers the others)'
                  % (key, ', '.join(sorted(live))))
        comps = {c: v for c, v in comps.items() if c not in live}
        print('  %s' % key)
        for comp, (heading, desc) in sorted(comps.items()):
            print('      %-22s %-26s %s' % (comp, heading, desc[:110]))
        if not comps:
            print('      (no matching doc row -- stale entry?)')
