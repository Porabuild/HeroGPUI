"""Every part v3's stylesheets declare, and whether this port names it.

`design_audit.py` compares the *metrics* a rule declares, so a part whose rule
sets only a colour, a transform or nothing measurable never reaches it -- and a
part this port does not render at all reaches it as an excuse
(`no-such-part`), which is easy to write and easy to leave. The Disclosure was
worse than that: `.disclosure__body` was cited by a check that pointed at the
Accordion's padding, because the component was built as a one-item Accordion.

This reads the other direction. Every `.component__part` selector in
`packages/styles/components/*.css` is a thing v3 draws; the convention in this
port is to name the selector in a comment where it is implemented
(`// `.disclosure__body` is `p-2`.`), so a selector that appears nowhere in
`crates/herogpui-components/src` is a part nobody has looked at.

    python .shots/part_audit.py            # the report
    python .shots/part_audit.py --all      # every part, mentioned or not

`WONT_DRAW` records the ones that stay unmentioned, with a reason: a `--variant`
modifier this port spells as an enum, a part that only exists to hold a CSS
transition, or a browser affordance with nothing behind it here.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

CACHE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')
SRC = 'crates/herogpui-components/src/'
THEME = 'crates/herogpui-theme/src/'
CORE = 'crates/herogpui-core/src/'

# Parts this port does not name, with the reason.
WONT_DRAW = {
    # v3 styles the element the caller passes as the trigger
    # (`<Button slot="trigger">`); this port takes it as a child and adds no
    # class of its own, so there is nothing here to cite or compare.
    'alert-dialog__trigger': 'trigger-is-a-child',
    'combo-box__trigger': 'trigger-is-a-child',
    'drawer__trigger': 'trigger-is-a-child',
    'dropdown__trigger': 'trigger-is-a-child',
    'modal__trigger': 'trigger-is-a-child',
    'popover__trigger': 'trigger-is-a-child',
    'tooltip__trigger': 'trigger-is-a-child',
    # A `<abbr>` for screen readers, with no geometry.
    'kbd__abbr': 'no-a11y-element',
}


def our_source():
    """Everywhere a part can be accounted for: the port, and the audits.

    The port's convention is to cite the selector in a comment where the part is
    drawn, but one whose *metric* is compared is accounted for by that row in
    `design_audit.py` instead -- `.slider__track` is checked there and mentioned
    in no comment. Both count; what is in neither is what nobody has read.
    """
    text = []
    for root in (SRC, THEME, CORE):
        for name in sorted(os.listdir(root)):
            if name.endswith('.rs'):
                text.append(io.open(root + name, encoding='utf-8',
                                    errors='replace').read())
    here = os.path.dirname(os.path.abspath(__file__))
    for audit in ('design_audit.py', 'state_audit.py', 'anim_audit.py',
                  'token_audit.py'):
        text.append(io.open(os.path.join(here, audit), encoding='utf-8',
                            errors='replace').read())
    return '\n'.join(text)


def parts():
    """`{component: [selector, ...]}` -- every `__part` rule v3 declares."""
    out = {}
    for name in sorted(os.listdir(CACHE)):
        if not name.endswith('.css') or name in ('variables.css', 'utilities.css',
                                                 'shared_theme.css'):
            continue
        css = io.open(os.path.join(CACHE, name), encoding='utf-8',
                      errors='replace').read()
        found = []
        # Only the part selectors: `.x__y`, with or without a `--variant` tail.
        # The tail can hold an underscore (`.fieldset__field_group`), and a
        # pattern without one truncates it to a part that does not exist.
        for m in re.finditer(r'\.([a-z0-9-]+__[a-z0-9_-]+)', css):
            sel = m.group(1)
            if sel not in found:
                found.append(sel)
        if found:
            out[name[:-4]] = found
    return out


def main():
    src = our_source()
    all_parts = parts()
    rows = []
    named = missing = excused = 0
    for comp in sorted(all_parts):
        for sel in all_parts[comp]:
            if '.' + sel in src or sel in src:
                named += 1
                if '--all' in sys.argv:
                    rows.append((' ', comp, sel, ''))
                continue
            base = sel.split('--')[0]
            if '--' in sel and ('.' + base) in src:
                # A `--variant` modifier is an enum here, not a class: the port
                # names the part and matches on `ModalSize`, `FieldVariant`,
                # `full_width`. Counting each modifier separately would ask for
                # a comment per enum arm.
                excused += 1
                if '--all' in sys.argv:
                    rows.append(('~', comp, sel, 'variant-is-an-enum'))
                continue
            reason = WONT_DRAW.get(sel)
            if reason:
                excused += 1
                rows.append(('~', comp, sel, reason))
                continue
            missing += 1
            rows.append(('?', comp, sel, ''))

    for mark, comp, sel, reason in rows:
        print('%s %-22s %-40s %s' % (mark, comp, '.' + sel, reason))
    print()
    print('parts v3 declares  : %d' % sum(len(v) for v in all_parts.values()))
    print('named in the port  : %d' % named)
    print('recorded won-t-draw: %d' % excused)
    # "Unverified", not "missing": most of these are drawn and simply not cited,
    # and the audit cannot tell the two apart -- which is the point of citing the
    # selector where the part is drawn.
    print('UNVERIFIED         : %d' % missing)


if __name__ == '__main__':
    main()
