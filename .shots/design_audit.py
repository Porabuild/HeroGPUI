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


def SIZE_XL(name):
    """`SizeXl` variant -> pixels, matching `SizeXl::px`."""
    return {'Xs': 16.0, 'Sm': 20.0, 'Md': 24.0, 'Lg': 32.0, 'Xl': 40.0}.get(name)


# (css file, rule selector, metric, our label, our file, regex, transform)
#
# The regex must capture our value in group 1, or the transform turns the match
# into one. `None` means "parse group 1 as a float".
CHECKS = [
    # --- Button, the whole size scale -------------------------------------
    # The radius reaches the button through `group_radius`, which is what lets a
    # grouped member round only its outer corners.
    ('button', '.button', 'radius', 'Button -> util::_radius', SRC + 'button.rs',
     r'group_radius\(e, self\.group_edge, util::(\w+_radius)\(cx\)\)', helper_px),
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
    # --- The sweep: the rest of the measurable geometry -------------------
    ('chip', '.chip', 'gap', 'Chip gap', SRC + 'chip.rs',
     '\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    # chip's tuple is (height, text, pad_x), so the padding is the third slot.
    ('chip', '.chip', 'px', 'Chip padding_x Md', SRC + 'chip.rs',
     'Size::Md => \\(px\\(\\d+(?:\\.\\d*)?\\), px\\(\\d+(?:\\.\\d*)?\\), '
     'px\\((\\d+(?:\\.\\d*)?)\\)', None),
    ('popover', '.popover__dialog', 'p', 'Popover padding', SRC + 'popover.rs',
     '\\.px\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)\\s+\\.py\\(px\\(16\\.\\)\\)', None),
    ('pagination', '.pagination', 'gap', 'Pagination gap', SRC + 'pagination.rs',
     'items_center\\(\\)\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('alert', '.alert', 'gap', 'Alert gap', SRC + 'alert.rs',
     '\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('alert', '.alert', 'px', 'Alert padding_x', SRC + 'alert.rs',
     '\\.px\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('alert', '.alert', 'py', 'Alert padding_y', SRC + 'alert.rs',
     '\\.py\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('link', '.link', 'radius', 'Link -> util::_radius', SRC + 'link.rs',
     '\\.rounded\\(crate::util::(\\w+_radius)\\(cx\\)\\)', helper_px),
    ('badge', '.badge', 'min_w', 'Badge min width Md', SRC + 'badge.rs',
     'Size::Md => \\(px\\((\\d+(?:\\.\\d*)?)\\)', None),
    ('kbd', '.kbd', 'px', 'Kbd padding_x', SRC + 'kbd.rs',
     '\\n            \\.px\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('fieldset', '.fieldset', 'gap', 'Fieldset gap', SRC + 'field.rs',
     '\\n            gap: px\\((\\d+(?:\\.\\d*)?)\\),', None),
    ('checkbox', '.checkbox__control', 'size', 'Checkbox control', SRC + 'checkbox.rs',
     'let \\(box_px, icon_px, text\\) = \\(px\\((\\d+(?:\\.\\d*)?)\\)', None),
    ('checkbox', '.checkbox__control', 'radius', 'Checkbox -> util::_radius', SRC + 'checkbox.rs',
     '\\.rounded\\(crate::util::(\\w+_radius)\\(cx\\)\\)', helper_px),
    ('radio', '.radio__control', 'size', 'Radio control', SRC + 'radio_group.rs',
     'let \\(circle, dot, text, gap\\) = \\(px\\((\\d+(?:\\.\\d*)?)\\)', None),
    # Anchored on the control, since the `secondary` variant's panel also has a
    # radius and comes first in the file.
    ('radio', '.radio__control', 'radius', 'Radio -> util::_radius', SRC + 'radio_group.rs',
     '\\.size\\(circle\\)\\s+\\.rounded\\(crate::util::(\\w+_radius)\\(cx\\)\\)', helper_px),
    ('radio', '.radio__content', 'gap', 'Radio row gap', SRC + 'radio_group.rs',
     'let \\(circle, dot, text, gap\\) = \\(px\\(\\d+(?:\\.\\d*)?\\), px\\(\\d+(?:\\.\\d*)?\\), px\\(\\d+(?:\\.\\d*)?\\), px\\((\\d+(?:\\.\\d*)?)\\)', None),
    ('list-box-item', '.list-box-item', 'gap', 'ListBox row gap', SRC + 'list_box.rs',
     '\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)\\s+\\.px\\(px\\(8\\.\\)\\)', None),
    ('list-box-item', '.list-box-item', 'py', 'ListBox row padding_y', SRC + 'list_box.rs',
     '\\.min_h\\(row_h\\)\\s+\\.py\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    # Anchored on the row, since the panel around it has a radius too.
    ('list-box-item', '.list-box-item', 'radius', 'ListBox row -> util::_radius', SRC + 'list_box.rs',
     '\\.py\\(px\\(6\\.\\)\\)\\s+\\.rounded\\(util::(\\w+_radius)\\(cx\\)\\)', helper_px),
    ('color-swatch', '.color-swatch', 'size', 'ColorSwatch default', SRC + 'color_picker.rs',
     'size: SizeXl::(\\w+),', SIZE_XL),
    ('card', '.card', 'gap', 'Card gap', SRC + 'card.rs',
     '\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('toolbar', '.toolbar', 'gap', 'Toolbar default gap', SRC + 'toolbar.rs',
     'is_attached \\{ px\\(\\d+(?:\\.\\d*)?\\) \\} else \\{ px\\((\\d+(?:\\.\\d*)?)\\) \\}', None),
    ('toast', '.toast', 'gap', 'Toast gap', SRC + 'toast.rs',
     '\\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('tabs', '.tabs', 'gap', 'Tabs gap', SRC + 'tabs.rs',
     '\\n                    \\.gap\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)', None),
    ('progress-bar', '.progress-bar__track', 'radius', 'ProgressBar track', SRC + 'progress.rs',
     '\\.rounded\\(crate::util::(\\w+_radius)\\(cx\\)\\)', helper_px),
    ('checkbox', '.checkbox__content', 'gap', 'Checkbox row gap', SRC + 'checkbox.rs',
     '`\.checkbox__content` is `gap-3`\.\s+\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('checkbox-group', '[data-slot="checkbox"]', 'mt', 'CheckboxGroup option gap',
     SRC + 'checkbox.rs',
     'let mut list = gpui::div\(\)\.flex\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('radio-group', '&[data-orientation="horizontal"]', 'gap', 'RadioGroup horizontal gap',
     SRC + 'radio_group.rs',
     'Orientation::Horizontal => gpui::div\(\)\.flex\(\)\.items_center\(\)\.flex_wrap\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch', '.switch__control', 'w', 'Switch md track width', SRC + 'switch.rs',
     'Size::Md => \(px\((\d+(?:\.\d*)?)\.\), px\(20\.\)', None),
    ('switch', '.switch__control', 'h', 'Switch md track height', SRC + 'switch.rs',
     'Size::Md => \(px\(40\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('switch', '.switch--sm .switch__control', 'w', 'Switch sm track width',
     SRC + 'switch.rs',
     'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\), px\(16\.\)', None),
    ('switch', '.switch--lg .switch__control', 'h', 'Switch lg track height',
     SRC + 'switch.rs',
     'Size::Lg => \(px\(48\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('switch', '.switch__thumb', 'w', 'Switch md thumb width', SRC + 'switch.rs',
     'Size::Md => \(px\(40\.\), px\(20\.\), px\((\d+(?:\.\d*)?)\.?\)', None),
    ('switch', '.switch__thumb', 'h', 'Switch md thumb height', SRC + 'switch.rs',
     'Size::Md => \(px\(40\.\), px\(20\.\), px\(22\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('switch', '.switch__content', 'gap', 'Switch row gap', SRC + 'switch.rs',
     '`\.switch__content` is `gap-3`\.\s+let mut el = gpui::div\(\)\.flex\(\)\.items_center\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__slot', 'w', 'InputOTP slot width', SRC + 'input_otp.rs',
     'let \(cell_w, cell_h, text, slot_gap\) = \(px\((\d+(?:\.\d*)?)\.?\)', None),
    ('input-otp', '.input-otp__slot', 'h', 'InputOTP slot height', SRC + 'input_otp.rs',
     'let \(cell_w, cell_h, text, slot_gap\) = \(px\(38\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('input-otp', '.input-otp__slot', 'text', 'InputOTP slot text', SRC + 'input_otp.rs',
     'px\(38\.\), px\(40\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('input-otp', '.input-otp', 'gap', 'InputOTP slot gap', SRC + 'input_otp.rs',
     'px\(38\.\), px\(40\.\), px\(14\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('select', '.select__trigger', 'min_h', 'Select trigger height',
     SRC + 'select.rs',
     'let \(h, text\) = \(util::(FIELD_HEIGHT)', lambda _: 36.0),
    ('select', '.select__trigger', 'radius', 'field chrome -> util::_radius',
     SRC + 'util.rs',
     'let mut el = el\.rounded\((field_radius)\(cx\)\)', helper_px),
    ('calendar', '.calendar', 'w', 'Calendar width', SRC + 'calendar.rs',
     'CALENDAR_WIDTH: gpui::Pixels = px\((\d+(?:\.\d*)?)\.\)', None),
    ('calendar', '.calendar__cell', 'text', 'Calendar cell text', SRC + 'calendar.rs',
     '\.size\(px\(36\.\)\)\s+\.rounded_full\(\)(?:\s+\.\w+\(\))*\s+\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('list-box-item', '.list-box-item', 'radius', 'Select row -> util::_radius',
     SRC + 'select.rs',
     '\.rounded\(util::(\w+_radius)\(cx\)\)\s+\.px\(px\(8\.\)\)', helper_px),
    ('list-box-item', '.list-box-item', 'gap', 'ComboBox row gap', SRC + 'combo_box.rs',
     '\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\s+\.rounded\(util::soft_radius', None),
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
    """The body of exactly one rule, e.g. `.button` or `.button--lg`.

    v3 uses CSS nesting, so a rule's braces also hold its parts:
    `.radio { @apply gap-1; &__description { @apply text-xs; } }`. Everything
    from the first nested `{` onward belongs to a part, not to this rule --
    reading past it made a radio's *label* measure 12px, which is its
    description's size.
    """
    pattern = '^' + re.escape(selector) + r'\s*\{(.*?)\n\}'
    m = re.search(pattern, css, re.S | re.M)
    if not m:
        return None
    body = m.group(1)
    nested = body.find('{')
    return body[:nested] if nested >= 0 else body


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

    # The switch declares `height: 1.25rem` rather than `h-5`, so a rem
    # declaration counts as the same metric. rem is root-relative (16px), not
    # font-relative -- v3's own comments guess otherwise.
    for prop, metric in (('height', 'h'), ('width', 'w')):
        m = re.search(prop + r':\s*([\d.]+)rem', body)
        if m:
            offer(metric, float(m.group(1)) * 16.0, '')

    for tok, bps in utilities(body).items():
        for bp in bps:
            for prefix, metric in (('h-', 'h'), ('w-', 'w'), ('px-', 'px'),
                                   ('py-', 'py'), ('gap-', 'gap'), ('p-', 'p'),
                                   ('size-', 'size'), ('min-w-', 'min_w'),
                                   ('mt-', 'mt'), ('min-h-', 'min_h')):
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


# Every check above is a number, and a control can match all of them and still
# be invisible: the radio measured 16px at radius 8 while painting a background
# v3 never uses, so an unselected option disappeared into the panel behind it.
#
# These compare the *fill token* instead. Each entry names the v3 rule and the
# token it fills with, plus the theme token our source must read for it -- both
# sides are checked, so a rule that stops declaring the token reports stale
# rather than passing.
FILLS = [
    # (css file, css rule, token in that rule, our file, our token expression)
    ('radio', '.radio__control', 'bg-field',
     SRC + 'radio_group.rs', 'colors.field.background'),
    ('radio-group', '.radio-group--secondary .radio__control', 'var(--default)',
     SRC + 'radio_group.rs', 'colors.default.color'),
    ('checkbox', '.checkbox__control', 'bg-field',
     SRC + 'checkbox.rs', 'colors.field.background'),
    ('checkbox', '.checkbox--secondary .checkbox__control', 'var(--default)',
     SRC + 'checkbox.rs', 'colors.default.color'),
    ('input', '.input--secondary', 'var(--default)',
     SRC + 'util.rs', 'FieldVariant::Secondary => colors.default.color'),
    ('input', '.input', 'bg-field',
     SRC + 'util.rs', 'FieldVariant::Primary => colors.field.background'),
    ('textarea', '.textarea', 'bg-field',
     SRC + 'util.rs', 'FieldVariant::Primary => colors.field.background'),
    # The track's fill is a custom property declared on the root, not a
    # `bg-*` utility on the control.
    ('switch', '.switch', 'var(--default)',
     SRC + 'switch.rs', 'colors.default.color'),
    ('switch', '.switch__thumb', 'bg-white',
     SRC + 'switch.rs', 'herogpui_theme::white()'),
    ('input-otp', '.input-otp__slot', 'bg-field',
     SRC + 'input_otp.rs', 'colors.field.background'),
    # Every floating panel is `bg-overlay`, which is a distinct token from
    # `--surface` -- a panel painted with the surface colour is the right shade
    # in light mode and the wrong one in dark.
    ('select', '.select__popover', 'bg-overlay', SRC + 'select.rs',
     'colors.overlay.background'),
    ('dropdown', '.dropdown__popover', 'bg-overlay', SRC + 'dropdown.rs',
     'colors.overlay.background'),
    ('popover', '.popover', 'bg-overlay', SRC + 'popover.rs',
     'colors.overlay.background'),
    ('tooltip', '.tooltip', 'bg-overlay', SRC + 'tooltip.rs',
     'colors.overlay.background'),
    ('modal', '.modal__dialog', 'bg-overlay', SRC + 'modal.rs',
     'colors.overlay.background'),
    ('drawer', '.drawer__dialog', 'bg-overlay', SRC + 'drawer.rs',
     'colors.overlay.background'),
    # The one floating surface v3 paints with `--surface` rather than
    # `--overlay`. Ours had it the other way round.
    ('toast', '.toast', 'bg-surface', SRC + 'toast.rs',
     'colors.surface.background'),
    ('alert-dialog', '.alert-dialog__dialog', 'bg-overlay',
     SRC + 'alert_dialog.rs', 'colors.overlay.background'),
    ('autocomplete', '.autocomplete__popover', 'bg-overlay',
     SRC + 'autocomplete.rs', 'colors.overlay.background'),
    ('combo-box', '.combo-box__popover', 'bg-overlay', SRC + 'combo_box.rs',
     'colors.overlay.background'),
    ('date-picker', '.date-picker__popover', 'bg-overlay',
     SRC + 'date_picker.rs', 'colors.overlay.background'),
    ('color-picker', '.color-picker__popover', 'bg-overlay',
     SRC + 'color_picker.rs', 'colors.overlay.background'),
]


def check_fills():
    """Each control's fill token, in v3's CSS and in ours."""
    rows, bad, stale = [], 0, 0
    for comp, selector, token, path, expr in FILLS:
        css_path = os.path.join(CACHE, comp + '.css')
        body = None
        if os.path.exists(css_path):
            css = io.open(css_path, encoding='utf-8', errors='replace').read()
            body = rule_body(css, selector)
        if body is None or token not in body:
            rows.append(('?', selector, token, 'not declared in v3 CSS'))
            stale += 1
            continue
        try:
            src = io.open(path, encoding='utf-8').read()
        except OSError:
            src = ''
        if expr in src:
            rows.append((' ', selector, token, expr))
        else:
            rows.append(('!', selector, token, 'ours does not read ' + expr))
            bad += 1
    print()
    print('  %-40s %-16s %s' % ('v3 rule', 'v3 fill', 'ours'))
    for mark, selector, token, note in rows:
        print('%s %-40s %-16s %s' % (mark, selector, token, note))
    print('fills compared   : %d' % len(rows))
    print('fills stale      : %d  (v3 no longer declares it -- re-read the rule)'
          % stale)
    print('WRONG FILLS      : %d' % bad)
    return bad, stale


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
            if body is None:
                # A rule nested inside another one is indented, so the
                # start-of-line match above misses it. `.checkbox-group` puts
                # its option spacing in `[data-slot="checkbox"]` and
                # `.radio-group` puts its horizontal gap in an
                # `&[data-orientation=...]` block; both are only reachable here.
                m = re.search(r'^\s+' + re.escape(selector) + r'\s*\{(.*?)' + chr(10)
                              + r'\s+\}', css, re.S | re.M)
                if m:
                    body = m.group(1)
            if body is None and '__' in selector:
                # A part is nested inside its parent, written either as
                # `&__dialog {` or spelled out as `.popover__dialog {`.
                part = selector.split('__')[1]
                for form in (r'&__' + re.escape(part), re.escape(selector)):
                    m = re.search(form + r'\s*\{(.*?)\n  \}', css, re.S)
                    if m:
                        body = m.group(1)
                        break
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
    check_fills()


if __name__ == '__main__':
    main()
