"""Re-examine every WONT_PORT entry: which components claim it, and what does
v3 say it does?

An excuse recorded once and never revisited is how a gap hides. This prints, for
each excluded prop, the components that document it and the doc row, so each
reason can be checked against what the prop actually is.
"""
import contextlib
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
PROP_IDENTIFIER = r'[a-zA-Z_][a-zA-Z0-9_-]*'

# prop -> {component: description}
where = {}
for comp in FILES:
    # The component's whole `## API Reference` section, one `###` table at a
    # time -- the same unit `api_audit.props_for` reads. Matching only
    # `### <Comp>` here left every row of `### ListLayout`, `### toast Function`
    # and `### Composition Components` looking undocumented, so a reason that
    # excuses one of them read as stale.
    anchor = r'^[ \t]*### (%s(?:\.[A-Za-z]+)?)[ \t]*$' % re.escape(comp)
    owners = [x for x in A.api_sections() if re.search(anchor, x, re.M)]
    if len(owners) != 1:
        continue
    heads = [(m.end(), m.group(1))
             for m in re.finditer(r'^[ \t]*### (.+?)[ \t]*$', owners[0], re.M)]
    for i, (at, heading) in enumerate(heads):
        chunk = owners[0][at:heads[i + 1][0]] if i + 1 < len(heads) else owners[0][at:]
        for row in re.finditer(
                r'^\|\s*`(%s)`\s*\|([^\n]*)$' % PROP_IDENTIFIER,
                chunk, re.M):
            prop, desc = row.group(1), re.sub(r'\s+', ' ', row.group(2)).strip()
            # Skip the translated duplicates.
            if re.search(r'[一-鿿]', desc):
                continue
            where.setdefault(prop, {}).setdefault(comp, (heading, desc))
        # The `Component | Prop | ...` shape: the row's first cell names the
        # part that owns it and the prop is column two -- `### Year Picker
        # Parts` documents `Calendar.YearPickerGrid` and
        # `Calendar.YearPickerTriggerHeading` that way. The pass above cannot
        # see it, because it demands a backticked simple word in the first
        # cell and the owner carries a dot. The guard here is
        # `api_audit.prop_rows_owned`'s, imported rather than restated so the
        # two readers cannot drift apart: first header exactly `Component`,
        # second header a prop indicator, and a backticked owner and a
        # backticked prop in the body's first two cells. A table of values
        # cannot pass it -- `Modifier Keys | Special Keys` names neither
        # header, `Component | Description` (the composition-parts listing)
        # fails the second, and even a hypothetical `Component | Value` table
        # has no backticked word in column two to extract -- so reading it
        # never invents the props `prop_rows` learnt to leave alone. The
        # owner determines the component: `Calendar.YearPickerGrid` belongs to
        # `Calendar`, and a row for a part of another component stays with
        # that component.
        for tbl in re.finditer(A.TABLE_RE, chunk, re.M):
            cells = tbl.group('head').split('|')
            if len(cells) < 2:
                continue
            first = cells[0].strip().strip('`').lower()
            if first != 'component':
                continue
            second = cells[1].strip().strip('`').lower()
            if second not in A.PROP_HEADERS:
                continue
            for row in re.finditer(
                    r'^\|\s*`([A-Za-z][A-Za-z0-9.]*)`\s*\|\s*`(%s)`\s*\|([^\n]*)$'
                    % PROP_IDENTIFIER,
                    tbl.group('body'), re.M):
                prop = row.group(2)
                desc = re.sub(r'\s+', ' ', row.group(3)).strip()
                # Skip the translated duplicates.
                if re.search(r'[一-鿿]', desc):
                    continue
                where.setdefault(prop, {}).setdefault(
                    row.group(1).split('.')[0], (heading, desc))


def print_entry(key, comps):
    print('  %s' % key)
    for comp, (heading, desc) in sorted(comps.items()):
        print('      %-22s %-26s %s' % (comp, heading, desc[:110]))
    if not comps:
        print('      (no matching doc row -- stale entry?)')


def self_test():
    """Known-positive and known-negative proof for omission-row matching.

    The live positive is the omission that exposed this reader hole. An
    unrelated key must remain absent so the stale-entry report cannot be
    replaced with a broad or silent match.
    """
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    portal = where.get('UNSTABLE_portalContainer', {})
    expect(set(portal) == {'AlertDialog', 'Modal'},
           'documented UNSTABLE_portalContainer row did not resolve for both '
           'backdrops: %r' % (portal,))
    expect(portal.get('AlertDialog', (None,))[0] == 'AlertDialog.Backdrop',
           'AlertDialog portal row resolved under the wrong heading: %r'
           % (portal.get('AlertDialog'),))
    expect(portal.get('Modal', (None,))[0] == 'Modal.Backdrop',
           'Modal portal row resolved under the wrong heading: %r'
           % (portal.get('Modal'),))

    stale = 'UNSTABLE_unknownPortalContainer'
    stale_output = io.StringIO()
    with contextlib.redirect_stdout(stale_output):
        print_entry(stale, where.get(stale, {}))
    expect(stale not in where and
           '(no matching doc row -- stale entry?)' in stale_output.getvalue(),
           'unknown omission key no longer produces the stale-entry report')

    if failures:
        print('self-test FAIL')
        for failure in failures:
            print('- %s' % failure)
        return 1
    print('self-test PASS: documented UNSTABLE_portalContainer resolves for '
          'AlertDialog and Modal; unknown omission keys still print the '
          'stale-entry marker')
    return 0


if __name__ == '__main__' and '--self-test' in sys.argv[1:]:
    sys.exit(self_test())

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
        print_entry(key, comps)
