"""Diff v3's component stylesheets against the metrics this port renders.

The prop and animation audits both read documentation, and neither says anything
about whether a control is the right *size*: `api_audit.py` was perfectly happy
with a button whose corner radius was a third of v3's. This reads the real
stylesheets from the v3 branch of the React repo, resolves the Tailwind
utilities through v3's own token scales, and compares the result with the
constants this port renders from.

    python .shots/design_audit.py --fetch    # once; caches under $TEMP
    python .shots/design_audit.py

Both sides are mechanical -- v3's from its CSS, ours from the Rust that defines
each metric -- so neither can quietly go stale. A check whose pattern stops
matching is reported as unreadable rather than skipped.

Two things this got wrong before they were fixed, both worth keeping in mind:

* **Scope each rule.** Reading every `@apply` in a file mixes the base rule with
  the size modifiers, which made a medium button measure 32px because
  `.button--sm` lives in the same file.
* **Prefer the largest breakpoint.** v3's sheet is mobile-first, so `.button` is
  `h-10 md:h-9`: 40px on a phone, 36px from `md` up. A desktop app is past every
  breakpoint, so the `md` value is the one to match. Taking the base value makes
  every control a step too big.
"""
import io
import os
import re
import subprocess
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

CACHE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')
COMPONENTS = ('https://raw.githubusercontent.com/heroui-inc/heroui/v3'
              '/packages/styles/components/%s.css')

# v3's own scales, from themes/shared/theme.css and themes/default/variables.css.
SPACING = 4.0            # --spacing: 0.25rem
RADIUS_BASE = 8.0        # --radius: 0.5rem
RADIUS = {
    'none': 0.0, 'xs': RADIUS_BASE * 0.25, 'sm': RADIUS_BASE * 0.5,
    'md': RADIUS_BASE * 0.75, 'lg': RADIUS_BASE, 'xl': RADIUS_BASE * 1.5,
    '2xl': RADIUS_BASE * 2, '3xl': RADIUS_BASE * 3, '4xl': RADIUS_BASE * 4,
    'full': 9999.0, 'field': RADIUS_BASE * 1.5,
}
TEXT = {'xs': 12.0, 'sm': 14.0, 'base': 16.0, 'lg': 18.0, 'xl': 20.0, '2xl': 24.0}
BREAKPOINTS = ['', 'sm', 'md', 'lg', 'xl', '2xl']

# A metric v3 deliberately does not declare. Absent means zero here, not
# unreadable: `.button` has no `min-w-*` because a v3 button hugs its label, and
# that is a real expectation to hold us to.
ABSENT_IS_ZERO = {('button', '.button', 'min_w')}

CORE = 'crates/herogpui-core/src/enums.rs'
LAYOUT = 'crates/herogpui-theme/src/layout.rs'
SRC = 'crates/herogpui-components/src/'

def helper_px(name):
    """Resolve one of `util`'s radius helpers to pixels, by reading it.

    The helper names moved once already -- `control_radius` went from
    `radius_lg` to `radius_3xl` -- so this follows the source rather than
    restating the mapping here and going stale.
    """
    src = io.open(SRC + 'util.rs', encoding='utf-8').read()
    body = re.search(
        r'pub fn ' + re.escape(name) + r'\(cx: &App\) -> Pixels \{(.*?)\n\}',
        src, re.S)
    if not body:
        return None
    step = re.search(r'radius_(\w+)\(\)', body.group(1))
    if step:
        return RADIUS.get(step.group(1))
    if 'field_radius' in body.group(1):
        return RADIUS['field']
    return None


# (css file, rule selector, metric, our label, our file, regex, transform)
#
# The regex must capture our value in group 1, or the transform turns the match
# into one. `None` means "parse group 1 as a float".
CHECKS = [
    # --- Button, the whole size scale -------------------------------------
    ('button', '.button', 'radius', 'Button -> util::_radius', SRC + 'button.rs',
     r'\.rounded\(util::(\w+_radius)\(cx\)\)', helper_px),
    ('button', '.button', 'h', 'Size::control_height Md', CORE,
     r'Control height[\s\S]*?Size::Md => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button--sm', 'h', 'Size::control_height Sm', CORE,
     r'Control height[\s\S]*?Size::Sm => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button--lg', 'h', 'Size::control_height Lg', CORE,
     r'Control height[\s\S]*?Size::Lg => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button', 'px', 'Size::padding_x Md', CORE,
     r'Horizontal padding[\s\S]*?Size::Md => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button--sm', 'px', 'Size::padding_x Sm', CORE,
     r'Horizontal padding[\s\S]*?Size::Sm => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button', 'text', 'Size::text_size Md', CORE,
     r'Label size[\s\S]*?Size::Md => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button--lg', 'text', 'Size::text_size Lg', CORE,
     r'Label size[\s\S]*?Size::Lg => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('button', '.button', 'gap', 'Size::gap Md', CORE,
     r"icon and its label[\s\S]*?Size::Sm \| Size::Md => gpui::px\((\d+(?:\.\d*)?)\)", None),
    # `.button` is `w-fit` with no `min-w-*`: a v3 button hugs its label. Ours
    # used to force 64/80/96px, which made every short label sit in a wide pill.
    ('button', '.button', 'min_w', 'Button min_w (none)', SRC + 'button.rs',
     r'el\.px\(self\.size\.padding_x\(\)\)\.gap\(self\.size\.gap\(\)\)',
     lambda _: 0.0),
    ('button', '.button--icon-only', 'w', 'Size::icon_control_size Md', CORE,
     r'Control height[\s\S]*?Size::Md => gpui::px\((\d+(?:\.\d*)?)\)', None),

    # --- Fields -----------------------------------------------------------
    ('input', '.input', 'radius', 'util::field_radius', SRC + 'input.rs',
     r'\.rounded\(crate::util::(field_radius)\(cx\)\)', helper_px),
    ('input', '.input', 'text', 'util::FIELD_TEXT', SRC + 'util.rs',
     r'pub const FIELD_TEXT: Pixels = gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('date-input-group', '.date-input-group', 'h', 'util::FIELD_HEIGHT',
     SRC + 'util.rs',
     r'pub const FIELD_HEIGHT: Pixels = gpui::px\((\d+(?:\.\d*)?)\)', None),

    # --- Everything else --------------------------------------------------
    ('spinner', '.spinner', 'size', 'SpinnerSize::Md', SRC + 'spinner.rs',
     r'SpinnerSize::Md => px\((\d+(?:\.\d*)?)\)', None),
    ('avatar', '.avatar', 'size', 'Avatar default size', SRC + 'avatar.rs',
     r'size_px: px\((\d+(?:\.\d*)?)\)', None),
    ('avatar', '.avatar', 'radius', 'Avatar -> util::_radius', SRC + 'avatar.rs',
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('close-button', '.close-button', 'h', 'CloseButton box', SRC + 'close_button.rs',
     r'let \(box_size, icon_size\) = \(px\((\d+(?:\.\d*)?)\)', None),
    ('kbd', '.kbd', 'h', 'Kbd height', SRC + 'kbd.rs',
     r'let \(h, min_w, text\) = \(px\((\d+(?:\.\d*)?)\)', None),
    ('kbd', '.kbd', 'text', 'Kbd text', SRC + 'kbd.rs',
     r'let \(h, min_w, text\) = \(px\(\d+(?:\.\d*)?\), px\(\d+(?:\.\d*)?\), px\((\d+(?:\.\d*)?)\)',
     None),
    ('skeleton', '.skeleton', 'radius', 'Skeleton -> util::_radius', SRC + 'skeleton.rs',
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    # The menu row has no `rounded` call at all, so its radius is 0.
    ('menu-item', '.menu-item', 'radius', 'Menu row -> util::_radius', SRC + 'dropdown.rs',
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)\s+\.h\(px\(32\.\)\)', helper_px),
    ('menu-item', '.menu-item', 'px', 'Menu row padding_x', SRC + 'dropdown.rs',
     r'\.px\(px\((\d+(?:\.\d*)?)\)\)\s+\.rounded\(crate::util::\w+_radius\(cx\)\)',
     None),
    ('toast', '.toast', 'px', 'Toast padding_x', SRC + 'toast.rs',
     r'\.px\(px\((\d+(?:\.\d*)?)\)\)\s*\n\s*\.py\(', None),
    ('toast', '.toast', 'py', 'Toast padding_y', SRC + 'toast.rs',
     r'\.py\(px\((\d+(?:\.\d*)?)\)\)\s*\n\s*\.rounded', None),
    ('tooltip', '.tooltip', 'text', 'Tooltip text', SRC + 'tooltip.rs',
     r'\.text_size\(px\((\d+(?:\.\d*)?)\)\)', None),
    ('chip', '.chip', 'radius', 'Chip -> util::_radius', SRC + 'chip.rs',
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
]


def fetch():
    os.makedirs(CACHE, exist_ok=True)
    names = sorted({c for c, *_ in CHECKS})
    for name in names:
        subprocess.run(['curl', '-sL', '--max-time', '30', '-o',
                        os.path.join(CACHE, name + '.css'), COMPONENTS % name],
                       check=False)
    print('fetched %d stylesheets into %s' % (len(names), CACHE))


def rule_body(css, selector):
    """The body of exactly one rule, e.g. `.button` or `.button--lg`."""
    pattern = '^' + re.escape(selector) + r'\s*\{(.*?)\n\}'
    m = re.search(pattern, css, re.S | re.M)
    return m.group(1) if m else None


def utilities(body):
    """Every `@apply` utility in one rule body, keyed by breakpoint."""
    out = {}
    for m in re.finditer(r'@apply ([^;]+);', body):
        for tok in m.group(1).split():
            bp = ''
            if ':' in tok:
                head, _, rest = tok.partition(':')
                if head not in BREAKPOINTS:
                    continue  # a state variant (hover:, data-[...]:), not a size
                bp, tok = head, rest
            out.setdefault(tok, set()).add(bp)
    return out


def measure(body):
    """One rule's metrics, preferring the largest breakpoint's value."""
    found = {}

    def offer(metric, value, bp):
        rank = BREAKPOINTS.index(bp) if bp in BREAKPOINTS else 0
        if metric not in found or rank >= found[metric][1]:
            found[metric] = (value, rank)

    def px(tok):
        m = re.fullmatch(r'(\d+(?:\.\d*)?)', tok)
        if m:
            return float(m.group(1)) * SPACING
        m = re.fullmatch(r'\[(\d+(?:\.\d*)?)px\]', tok)
        return float(m.group(1)) if m else None

    for tok, bps in utilities(body).items():
        for bp in bps:
            for prefix, metric in (('h-', 'h'), ('w-', 'w'), ('px-', 'px'),
                                   ('py-', 'py'), ('gap-', 'gap'), ('p-', 'p'),
                                   ('size-', 'size'), ('min-w-', 'min_w')):
                if tok.startswith(prefix):
                    v = px(tok[len(prefix):])
                    if v is not None:
                        offer(metric, v, bp)
            if tok.startswith('rounded-') and tok[8:] in RADIUS:
                offer('radius', RADIUS[tok[8:]], bp)
            elif tok == 'rounded':
                offer('radius', RADIUS['lg'], bp)
            if tok.startswith('text-') and tok[5:] in TEXT:
                offer('text', TEXT[tok[5:]], bp)
    return {k: v for k, (v, _) in found.items()}


def our_value(path, pattern, transform):
    try:
        src = io.open(path, encoding='utf-8').read()
    except OSError:
        return None
    m = re.search(pattern, src)
    if not m:
        return None
    group = m.group(1) if m.groups() else None
    if transform:
        return transform(group)
    return float(group) if group is not None else None


def main():
    if '--fetch' in sys.argv:
        fetch()
        return
    if not os.path.isdir(CACHE):
        print('no cache: run `python .shots/design_audit.py --fetch` first')
        sys.exit(2)

    rows, mismatched, unreadable = [], 0, 0
    for comp, selector, metric, label, path, pattern, transform in CHECKS:
        css_path = os.path.join(CACHE, comp + '.css')
        want = None
        if os.path.exists(css_path):
            css = io.open(css_path, encoding='utf-8', errors='replace').read()
            body = rule_body(css, selector)
            # A size modifier inherits whatever it does not restate.
            if body is not None:
                want = measure(body).get(metric)
                if want is None and '--' in selector:
                    base = rule_body(css, selector.split('--')[0])
                    if base:
                        want = measure(base).get(metric)
                if want is None and (comp, selector, metric) in ABSENT_IS_ZERO:
                    want = 0.0
        got = our_value(path, pattern, transform)
        if want is None or got is None:
            unreadable += 1
            rows.append(('?', selector, metric, want, got, label))
            continue
        ok = abs(want - got) < 0.51
        if not ok:
            mismatched += 1
        rows.append((' ' if ok else '!', selector, metric, want, got, label))

    print('  %-22s %-7s %6s %6s  %s' % ('v3 rule', 'metric', 'v3', 'ours', 'ours defined by'))
    for mark, selector, metric, want, got, label in rows:
        print('%s %-22s %-7s %6s %6s  %s' % (
            mark, selector, metric,
            'n/a' if want is None else '%g' % want,
            'n/a' if got is None else '%g' % got, label))
    print()
    print('metrics compared : %d' % len(rows))
    print('unreadable       : %d  (a pattern stopped matching -- fix it)' % unreadable)
    print('MISMATCHED       : %d' % mismatched)


if __name__ == '__main__':
    main()
