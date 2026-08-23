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

# A metric v3 declares that no `CHECKS` row can name, with the reason. Two kinds
# only, and both are about *where* the value lives rather than whether it is
# honoured:
#
# * `drives-the-height` -- a v3 field is `py-2` plus a line box, and this port
#   sets the 36px that comes to instead (`util::FIELD_HEIGHT`, compared as `h`
#   or `min_h` on the same rule). Comparing the padding as well would ask the
#   code for a number it deliberately does not spell.
# * `no-such-part` -- v3 declares a part this port does not render, so there is
#   nothing to measure. Each one names what is missing rather than waving at it.
COVERED_ELSEWHERE = {
    # `.menu`'s own `gap-1 p-1` never applies here: the only menu this port
    # renders is a dropdown's, and `.dropdown__menu` restates both (`gap-0.5`),
    # which is what the two `.dropdown__menu` rows compare.
    ('menu', '.menu', 'gap'): 'restated-by-dropdown-menu',
    ('menu', '.menu', 'p'): 'restated-by-dropdown-menu',
    # A v3 date picker pads a box (`p-1`) around a nested date input group and
    # its button; this port draws one field, so the trigger *is* the field and
    # its padding is the field's (compared as `.input`'s `px`).
    ('date-picker', '.date-picker__trigger', 'p'): 'trigger-is-the-field',
    ('date-range-picker', '.date-range-picker__trigger', 'p'): 'trigger-is-the-field',
    # A Disclosure renders as a one-item Accordion, whose body padding is
    # `.accordion__body-inner`'s and compared there.
    ('disclosure', '.disclosure__body', 'p'): 'accordion-body',
    ('input', '.input', 'py'): 'drives-the-height',
    ('textarea', '.textarea', 'py'): 'drives-the-height',
    ('select', '.select__trigger', 'py'): 'drives-the-height',
    ('autocomplete', '.autocomplete__trigger', 'py'): 'drives-the-height',
    ('input-group', '.input-group__input', 'py'): 'drives-the-height',
    ('search-field', '.search-field__input', 'py'): 'drives-the-height',
    ('number-field', '.number-field__input', 'py'): 'drives-the-height',
    ('color-input-group', '.color-input-group__input', 'py'): 'drives-the-height',
    ('date-input-group', '.date-input-group__input', 'py'): 'drives-the-height',
}

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
    """`SizeXl` variant -> pixels, matching `SizeXl::swatch_px`.

    Read out of the enum rather than restated, since that is the mapping under
    test: a shared 16/20/24/32/40 scale matched neither of v3's two sheets.
    """
    src = io.open(CORE, encoding='utf-8').read()
    body = re.search(r'pub fn swatch_px\(self\) -> gpui::Pixels \{([\s\S]*?)\n    \}', src)
    if not body:
        return None
    m = re.search(r'SizeXl::' + re.escape(name) + r'(?: \| SizeXl::\w+)? => gpui::px\(([0-9.]+)\)',
                  body.group(1))
    return float(m.group(1)) if m else None


# (css file, rule selector, metric, our label, our file, regex, transform)
#
# The regex must capture our value in group 1, or the transform turns the match
# into one. `None` means "parse group 1 as a float".
CHECKS = [
    ('close-button', '.close-button', 'w', 'CloseButton width',
     SRC + 'close_button.rs',
     r'let \(box_size, icon_size\) = \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('close-button', '.close-button', 'radius', 'CloseButton -> util::_radius',
     SRC + 'close_button.rs',
     r'rounded\(crate::util::(\w+_radius)', helper_px),
    ('checkbox', '.checkbox__indicator', 'size', 'Checkbox tick size',
     SRC + 'checkbox.rs',
     r'`\.checkbox__indicator` `size-3`[\s\S]{0,160}?px\((\d+(?:\.\d*)?)\.\), px\(14', None),
    ('color-area', '.color-area', 'radius', 'ColorArea -> util::_radius',
     SRC + 'color_picker.rs',
     r'`\.color-area` is `rounded-2xl`[\s\S]{0,120}?util::(\w+_radius)', helper_px),
    ('color-area', '.color-area__thumb', 'size', 'ColorArea thumb',
     SRC + 'color_picker.rs',
     r'Thumb: y is inverted[\s\S]{0,600}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-area', '.color-area__thumb', 'border', 'ColorArea thumb border',
     SRC + 'color_picker.rs',
     r'Thumb: y is inverted[\s\S]{0,600}?\.border\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('button-group', '.button-group__separator', 'radius', 'ButtonGroup separator -> util::_radius',
     SRC + 'button_group.rs',
     r'let separator_radius = util::(\w+_radius)', helper_px),
    ('calendar', '.calendar__header', 'px', 'Calendar header px',
     SRC + 'calendar.rs',
     r'`\.calendar__header` is `px-0\.5`\.[\s\S]{0,60}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('accordion', '.accordion__body-inner', 'px', 'Accordion body px',
     SRC + 'accordion.rs',
     r'px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert', '.alert__indicator', 'p', 'Alert indicator padding',
     SRC + 'alert.rs',
     r'`\.alert__indicator` is a `p-1` box[\s\S]{0,160}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('textfield', '.textfield', 'gap', 'textfield field column gap',
     SRC + 'input.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('search-field', '.search-field', 'gap', 'search-field field column gap',
     SRC + 'input.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('number-field', '.number-field', 'gap', 'number-field field column gap',
     SRC + 'number_field.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('time-field', '.time-field', 'gap', 'time-field field column gap',
     SRC + 'date_picker.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('select', '.select', 'gap', 'select field column gap',
     SRC + 'select.rs',
     r'let mut wrapper = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-picker', '.date-picker', 'gap', 'date-picker field column gap',
     SRC + 'date_picker.rs',
     r'let mut wrapper = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tag-group', '.tag-group', 'gap', 'tag-group field column gap',
     SRC + 'tag_group.rs',
     r'let mut root = div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tag-group', '.tag-group__list', 'gap', 'TagGroup list gap',
     SRC + 'tag_group.rs',
     r'let mut list = div\(\)\.flex\(\)\.flex_row\(\)\.flex_wrap\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('separator', '.separator--vertical', 'min_h', 'Separator vertical minimum',
     SRC + 'separator.rs',
     r'`\.separator--vertical` is `min-h-2`[\s\S]{0,200}?\.min_h\(gpui::px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('separator', '.separator', 'radius', 'Separator -> util::_radius',
     SRC + 'separator.rs',
     r'let radius = crate::util::(\w+_radius)\(cx\)', helper_px),
    ('toolbar', '.toolbar--attached', 'p', 'Toolbar attached padding',
     SRC + 'toolbar.rs',
     r'`\.toolbar--attached` is `p-1 rounded-3xl`\.[\s\S]{0,60}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toolbar', '.toolbar--attached', 'radius', 'Toolbar attached -> util::_radius',
     SRC + 'toolbar.rs',
     r'`p-1 rounded-3xl`[\s\S]{0,140}?\.rounded\(crate::util::(\w+_radius)', helper_px),
    ('tooltip', '.tooltip', 'p', 'Tooltip padding',
     SRC + 'tooltip.rs',
     r'`\.tooltip` is `p-2` all round[\s\S]{0,120}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch-group', '.switch-group', 'gap', 'SwitchGroup gap',
     SRC + 'switch.rs',
     r'gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\.child', None),
    ('spinner', '.spinner--sm', 'size', 'Spinner Sm',
     SRC + 'spinner.rs',
     r'SpinnerSize::Sm => px\((\d+(?:\.\d*)?)\)', None),
    ('spinner', '.spinner--lg', 'size', 'Spinner Lg',
     SRC + 'spinner.rs',
     r'SpinnerSize::Lg => px\((\d+(?:\.\d*)?)\)', None),
    ('spinner', '.spinner--xl', 'size', 'Spinner Xl',
     SRC + 'spinner.rs',
     r'SpinnerSize::Xl => px\((\d+(?:\.\d*)?)\)', None),
    ('table', '.table__column-resizer', 'px', 'Table resizer grab margin',
     SRC + 'table.rs',
     r'8px grab margin either side[\s\S]{0,80}?\.right\(px\(-(\d+(?:\.\d*)?)\.\)\)', None),
    ('toggle-button', '.toggle-button', 'text', 'ToggleButton Md text',
     SRC + 'toggle_button.rs',
     r'let text = self\.size\.text_size\(\)', lambda _: 14.0),
    ('calendar-year-picker', '.calendar-year-picker__trigger', 'gap', 'Year trigger gap',
     SRC + 'calendar.rs',
     r'`\.calendar-year-picker__trigger` is `gap-1 rounded-lg`\.[\s\S]{0,60}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__trigger', 'radius', 'Year trigger -> util::_radius',
     SRC + 'calendar.rs',
     r'`gap-1 rounded-lg`[\s\S]{0,200}?\.rounded\(crate::util::(\w+_radius)', helper_px),
    ('calendar-year-picker', '.calendar-year-picker__trigger-heading', 'text', 'Year heading text',
     SRC + 'calendar.rs',
     r'let heading = \|text: String[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar', '.calendar__heading', 'text', 'Calendar heading text',
     SRC + 'calendar.rs',
     r'let heading = \|text: String[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('checkbox', '.checkbox__content', 'gap', 'Checkbox content gap',
     SRC + 'checkbox.rs',
     r'`\.checkbox__content` is `gap-3`\.[\s\S]{0,60}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('accordion', '.accordion__trigger', 'text', 'Accordion trigger text',
     SRC + 'accordion.rs',
     r'px\(px\(16\.\)\)[\s\S]{0,300}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('avatar', '.avatar--sm', 'size', 'Avatar Sm box',
     SRC + 'avatar.rs',
     r'herogpui_core::Size::Sm => px\((\d+(?:\.\d*)?)\.\)', None),
    ('avatar', '.avatar--lg', 'size', 'Avatar Lg box',
     SRC + 'avatar.rs',
     r'herogpui_core::Size::Lg => px\((\d+(?:\.\d*)?)\.\)', None),
    ('avatar', '.avatar', 'size', 'Avatar Md box',
     SRC + 'avatar.rs',
     r'herogpui_core::Size::Md => px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge--sm', 'min_w', 'Badge Sm box (square)',
     SRC + 'badge.rs',
     r'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge--lg', 'min_w', 'Badge Lg box (square)',
     SRC + 'badge.rs',
     r'Size::Lg => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge', 'min_w', 'Badge Md box (square)',
     SRC + 'badge.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--sm', 'text', 'Chip Sm text',
     SRC + 'chip.rs',
     r'Size::Sm => \(px\(4\.\), px\(0\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--md', 'text', 'Chip Md text (again)',
     SRC + 'chip.rs',
     r'Size::Md => \(px\(8\.\), px\(2\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('alert', '.alert__title', 'text', 'Alert title text',
     SRC + 'alert.rs',
     r'let mut text_col[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toggle-button', '.toggle-button', 'px', 'ToggleButton Md px',
     SRC + 'toggle_button.rs',
     r'Size::Md => \(px\(36\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('toggle-button', '.toggle-button--sm', 'px', 'ToggleButton Sm px',
     SRC + 'toggle_button.rs',
     r'Size::Sm => \(px\(32\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('toggle-button', '.toggle-button', 'gap', 'ToggleButton Md gap',
     SRC + 'toggle_button.rs',
     r'Size::Md => \(px\(36\.\), px\(16\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('toggle-button', '.toggle-button', 'radius', 'ToggleButton -> util::_radius',
     SRC + 'toggle_button.rs',
     r'let radius = crate::util::(\w+_radius)', helper_px),
    ('toggle-button', '.toggle-button--icon-only', 'w', 'ToggleButton icon-only box',
     SRC + 'toggle_button.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('textarea', '.textarea', 'px', 'TextArea px -> Input',
     SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toast', '.toast__title', 'text', 'Toast title text',
     SRC + 'toast.rs',
     r'`\.toast__title` is `text-sm leading-5 font-medium`\.[\s\S]{0,60}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toast', '.toast__description', 'text', 'Toast description text',
     SRC + 'toast.rs',
     r'`\.toast__description` is `text-sm text-muted`\.[\s\S]{0,60}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toast', '.toast__close-button', 'size', 'Toast close button',
     SRC + 'toast.rs',
     r'`\.toast__close-button` is `size-5`[\s\S]{0,240}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-picker', '.date-picker__popover', 'p', 'DatePicker popover padding',
     SRC + 'date_picker.rs',
     r'are `p-3`\.[\s\S]{0,60}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-range-picker', '.date-range-picker__popover', 'p', 'DateRangePicker popover padding',
     SRC + 'date_picker.rs',
     r'are `p-3`\.[\s\S]{0,60}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-picker', '.color-picker__popover', 'min_w', 'ColorPicker popover min width',
     SRC + 'color_picker.rs',
     r'`gap-3 min-w-62 px-2`[\s\S]{0,300}?\.min_w\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-picker', '.color-picker__popover', 'px', 'ColorPicker popover px',
     SRC + 'color_picker.rs',
     r'`gap-3 min-w-62 px-2`[\s\S]{0,260}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-picker', '.color-picker__popover', 'gap', 'ColorPicker popover gap',
     SRC + 'color_picker.rs',
     r'`gap-3 min-w-62 px-2`[\s\S]{0,200}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__slot-value', 'text', 'InputOTP digit text',
     SRC + 'input_otp.rs',
     r'`\.input-otp__slot-value` is `text-lg[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__caret', 'w', 'InputOTP caret width',
     SRC + 'input_otp.rs',
     r'`\.input-otp__caret` is `h-4[\s\S]{0,200}?\.w\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__caret', 'h', 'InputOTP caret height',
     SRC + 'input_otp.rs',
     r'`\.input-otp__caret` is `h-4[\s\S]{0,240}?\.h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__caret', 'radius', 'InputOTP caret -> util::_radius',
     SRC + 'input_otp.rs',
     r'`\.input-otp__caret` is `h-4[\s\S]{0,300}?\.rounded\(crate::util::(\w+_radius)', helper_px),
    ('input-otp', '.input-otp__slot', 'radius', 'InputOTP slot -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('input-otp', '.input-otp__group', 'gap', 'InputOTP slot gap',
     SRC + 'input_otp.rs',
     r'let \(cell_w, cell_h, text, slot_gap\) = \(px\(38\.\), px\(40\.\), px\(14\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('search-field', '.search-field__clear-button', 'size', 'Clear button box',
     SRC + 'input.rs',
     r'input-clear-[\s\S]{0,200}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar', 'w', 'RangeCalendar width',
     SRC + 'range_calendar.rs',
     r'crate::calendar::(CALENDAR_WIDTH)', lambda _: 252.0),
    ('range-calendar', '.range-calendar__heading', 'text', 'RangeCalendar heading text',
     SRC + 'range_calendar.rs',
     r'let heading = [\s\S]{0,400}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__cell', 'text', 'Table cell text',
     SRC + 'table.rs',
     r'let mut table = gpui::div\([\s\S]{0,80}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('card', '.card', 'p', 'Card padding',
     SRC + 'card.rs',
     r'the card is the padded box[\s\S]{0,320}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('card', '.card', 'gap', 'Card gap',
     SRC + 'card.rs',
     r'the card is the padded box[\s\S]{0,280}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('card', '.card__content', 'gap', 'Card content gap',
     SRC + 'card.rs',
     r'`\.card__content` is[\s\S]{0,280}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('card', '.card__title', 'text', 'Card title text',
     SRC + 'card.rs',
     r'`\.card__header` is[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('card', '.card__description', 'text', 'Card body text',
     SRC + 'card.rs',
     r'`\.card__content` is[\s\S]{0,260}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch', '.switch__content', 'text', 'Switch content text',
     SRC + 'switch.rs',
     r'let mut el = gpui::div\(\)[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch', '.switch__label', 'text', 'Switch label text',
     SRC + 'switch.rs',
     r'`\.switch__label` is `text-base`[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch', '.switch__content', 'gap', 'Switch content gap',
     SRC + 'switch.rs',
     r'`\.switch__content` is `gap-3`[\s\S]{0,300}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('dropdown', '.dropdown__popover', 'min_w', 'Dropdown menu min width',
     SRC + 'dropdown.rs',
     r'`md:min-w-55`[\s\S]{0,160}?\.min_w\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('dropdown', '.dropdown__menu', 'gap', 'Dropdown menu gap',
     SRC + 'dropdown.rs',
     r'`gap-0\.5 p-1`[\s\S]{0,160}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('dropdown', '.dropdown__menu', 'p', 'Dropdown menu padding',
     SRC + 'dropdown.rs',
     r'`gap-0\.5 p-1`[\s\S]{0,200}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-swatch', '.color-swatch--xs', 'size', 'ColorSwatch Xs', CORE,
     r'SizeXl::Xs => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('color-swatch', '.color-swatch--sm', 'size', 'ColorSwatch Sm', CORE,
     r'SizeXl::Sm => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('color-swatch', '.color-swatch--lg', 'size', 'ColorSwatch Lg', CORE,
     r'SizeXl::Lg => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('color-swatch', '.color-swatch--xl', 'size', 'ColorSwatch Xl', CORE,
     r'SizeXl::Xl => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__year-grid', 'gap',
     'Year grid gap', SRC + 'calendar.rs',
     r'`\.calendar-year-picker__year-grid` is `gap-1 p-1`\.[\s\S]{0,80}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__year-grid', 'p',
     'Year grid padding', SRC + 'calendar.rs',
     r'`gap-1 p-1`\.[\s\S]{0,120}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__year-cell', 'h',
     'Year cell height', SRC + 'calendar.rs',
     r'`h-8 px-2\.5[\s\S]{0,120}?\.h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__year-cell', 'px',
     'Year cell px', SRC + 'calendar.rs',
     r'`h-8 px-2\.5[\s\S]{0,160}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar-year-picker', '.calendar-year-picker__year-cell', 'text',
     'Year cell text', SRC + 'calendar.rs',
     r'`h-8 px-2\.5[\s\S]{0,320}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('autocomplete', '.autocomplete__trigger', 'min_h', 'autocomplete height',
     SRC + 'autocomplete.rs',
     r'\.min_h\(util::(FIELD_HEIGHT)\)', lambda _: 36.0),
    ('search-field', '.search-field__group', 'h', 'search-field height',
     SRC + 'input.rs',
     r'let \(h, text\) = \(crate::util::(FIELD_HEIGHT)', lambda _: 36.0),
    ('color-input-group', '.color-input-group', 'h', 'color-input-group height',
     SRC + 'color_picker.rs',
     r'\.h\(util::(FIELD_HEIGHT)\)', lambda _: 36.0),
    ('autocomplete', '.autocomplete__trigger', 'radius', 'autocomplete radius -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('search-field', '.search-field__group', 'radius', 'search-field radius -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('color-input-group', '.color-input-group', 'radius', 'color-input-group radius -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('date-input-group', '.date-input-group', 'radius', 'date-input-group radius -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('select', '.select__trigger', 'radius', 'select radius -> field_radius',
     SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('autocomplete', '.autocomplete__trigger', 'text', '.autocomplete__trigger text -> FIELD_TEXT',
     SRC + 'autocomplete.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('autocomplete', '.autocomplete__value', 'text', '.autocomplete__value text -> FIELD_TEXT',
     SRC + 'autocomplete.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('select', '.select__trigger', 'text', '.select__trigger text -> FIELD_TEXT',
     SRC + 'select.rs',
     r'let \(h, text\) = \(util::FIELD_HEIGHT, util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('select', '.select__value', 'text', '.select__value text -> FIELD_TEXT',
     SRC + 'select.rs',
     r'let \(h, text\) = \(util::FIELD_HEIGHT, util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('color-input-group', '.color-input-group', 'text', '.color-input-group text -> FIELD_TEXT',
     SRC + 'color_picker.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('date-input-group', '.date-input-group', 'text', '.date-input-group text -> FIELD_TEXT',
     SRC + 'date_picker.rs',
     r'\.text_size\(crate::util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('search-field', '.search-field__input', 'px', '.search-field__input px -> Input',
     SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-input-group', '.color-input-group__input', 'px', '.color-input-group__input px -> Input',
     SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-input-group', '.date-input-group__input', 'px', '.date-input-group__input px -> Input',
     SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('number-field', '.number-field__input', 'px', '.number-field__input px -> Input',
     SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('search-field', '.search-field__search-icon', 'size', 'SearchField icon -> FIELD_ICON',
     SRC + 'input.rs',
     r'\.size\(crate::util::(FIELD_ICON)\)', lambda _: 16.0),
    # --- Avatar, Alert, Accordion, the swatches ---------------------------
    ('avatar', '.avatar--sm', 'radius', 'Avatar Sm -> util::_radius', SRC + 'avatar.rs',
     r'if self\.small [\s\S]{0,40}?crate::util::(\w+_radius)', helper_px),
    ('avatar', '.avatar__fallback', 'text', 'Avatar fallback text', SRC + 'avatar.rs',
     r'`\.avatar__fallback` is `text-sm`[\s\S]{0,80}?let font = px\((\d+(?:\.\d*)?)\.\)', None),
    ('alert', '.alert__description', 'text', 'Alert description text', SRC + 'alert.rs',
     r'`\.alert__description` is `text-sm`\.[\s\S]{0,40}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('accordion', '.accordion__indicator', 'size', 'Accordion indicator', SRC + 'accordion.rs',
     r'`\.accordion__indicator` is `size-4`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-swatch-picker', '.color-swatch-picker', 'gap', 'ColorSwatchPicker gap',
     SRC + 'color_picker.rs',
     r'let mut row = div\(\)\.flex\(\)\.flex_row\(\)\.items_center\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__separator', 'w', 'InputOTP separator width',
     SRC + 'input_otp.rs',
     r'group separator every 3 cells[\s\S]{0,200}?\.w\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__separator', 'h', 'InputOTP separator height',
     SRC + 'input_otp.rs',
     r'group separator every 3 cells[\s\S]{0,240}?\.h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-otp', '.input-otp__separator', 'radius', 'InputOTP separator -> util::_radius',
     SRC + 'input_otp.rs',
     r'group separator every 3 cells[\s\S]{0,300}?\.rounded\(crate::util::(\w+_radius)', helper_px),
    # --- The calendars ----------------------------------------------------
    # The two calendars are not the same component twice: the range one's nav
    # button is `rounded-xl` where the single one's is `rounded-2xl`.
    ('calendar', '.calendar__nav-button', 'size', 'Calendar nav button', SRC + 'calendar.rs',
     r'`size-6 rounded-2xl`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar', '.calendar__nav-button', 'radius', 'Calendar nav -> util::_radius',
     SRC + 'calendar.rs',
     r'`size-6 rounded-2xl`[\s\S]{0,80}?\.rounded\(crate::util::(\w+_radius)', helper_px),
    ('calendar', '.calendar__nav-button-icon', 'size', 'Calendar nav icon', SRC + 'calendar.rs',
     r'`\.calendar__nav-button-icon` is `size-4`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar', '.calendar__header-cell', 'text', 'Calendar header cell', SRC + 'calendar.rs',
     r'`\.calendar__header-cell` is `text-xs`\.[\s\S]{0,40}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar', '.calendar__cell', 'text', 'Calendar day cell text', SRC + 'calendar.rs',
     r'Uniform circular hit area[\s\S]{0,260}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # `rounded-3xl` on a 36px box is clamped to a circle by any renderer, so
    # `rounded_full` is the same pixels -- the equality only holds because the
    # cell is smaller than twice the radius, which is why it is stated here.
    ('calendar', '.calendar__cell', 'radius', 'Calendar day cell (circle == 3xl at 36px)',
     SRC + 'calendar.rs',
     r'Uniform circular hit area[\s\S]{0,260}?\.rounded_(full)\(\)', lambda _: 24.0),
    ('calendar', '.calendar__cell-indicator', 'size', 'Calendar cell indicator',
     SRC + 'calendar.rs',
     r'smaller than any radius[\s\S]{0,120}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('calendar', '.calendar', 'w', 'Calendar width', SRC + 'calendar.rs',
     r'CALENDAR_WIDTH: gpui::Pixels = px\((\d+(?:\.\d*)?)\.\)', None),
    ('range-calendar', '.range-calendar__nav-button', 'size', 'RangeCalendar nav button',
     SRC + 'range_calendar.rs',
     r'is `size-6 rounded-xl`[\s\S]{0,200}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__nav-button', 'radius',
     'RangeCalendar nav -> util::_radius', SRC + 'range_calendar.rs',
     r'`size-6 rounded-xl`[\s\S]{0,200}?\.rounded\(util::(\w+_radius)', helper_px),
    ('range-calendar', '.range-calendar__nav-button-icon', 'size', 'RangeCalendar nav icon',
     SRC + 'range_calendar.rs',
     r'`\.range-calendar__nav-button-icon` is `size-4`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__header-cell', 'text', 'RangeCalendar header cell',
     SRC + 'range_calendar.rs',
     r'`\.range-calendar__header-cell` is `text-xs`\.[\s\S]{0,40}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # --- Tabs, Table, Pagination -----------------------------------------
    ('tabs', '.tabs__list', 'p', 'Tabs list padding', SRC + 'tabs.rs',
     r'list = list[\s\S]{0,40}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tabs', '.tabs__tab', 'h', 'Tabs tab height', SRC + 'tabs.rs',
     r'font-medium`\.[\s\S]{0,40}?\.h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tabs', '.tabs__tab', 'px', 'Tabs tab px', SRC + 'tabs.rs',
     r'font-medium`\.[\s\S]{0,40}?\.h\(px\(32\.\)\)[\s\S]{0,40}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tabs', '.tabs__tab', 'text', 'Tabs tab text', SRC + 'tabs.rs',
     r'\.rounded\(crate::util::control_radius\(cx\)\)[\s\S]{0,40}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('tabs', '.tabs__tab', 'radius', 'Tabs tab -> util::_radius', SRC + 'tabs.rs',
     r'\.justify_center\(\)[\s\S]{0,40}?\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('tabs', '.tabs__panel', 'p', 'Tabs panel padding', SRC + 'tabs.rs',
     r'`\.tabs__panel` is `w-full p-2`\.[\s\S]{0,40}?el = el\.child\(gpui::div\(\)\.w_full\(\)\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__column', 'px', 'Table header px', SRC + 'table.rs',
     r'`\.table__column` is `px-4 py-2\.5 text-xs`\.[\s\S]{0,40}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__column', 'py', 'Table header py', SRC + 'table.rs',
     r'`\.table__column` is[\s\S]{0,80}?\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__column', 'text', 'Table header text', SRC + 'table.rs',
     r'`\.table__column` is[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__cell', 'px', 'Table cell px', SRC + 'table.rs',
     r'`\.table__cell` is `px-4 py-3`\.[\s\S]{0,40}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__cell', 'py', 'Table cell py', SRC + 'table.rs',
     r'`\.table__cell` is[\s\S]{0,80}?\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__content', 'text', 'Table text', SRC + 'table.rs',
     r'let mut table = gpui::div\([\s\S]{0,80}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__column-resizer', 'h', 'Table resizer line height', SRC + 'table.rs',
     r'\.w\(px\(1\.\)\)[\s\S]{0,40}?\.h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__column-resizer', 'radius', 'Table resizer -> util::_radius',
     SRC + 'table.rs',
     r'\.h\(px\(16\.\)\)[\s\S]{0,40}?\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('table', '.table__load-more-content', 'gap', 'Table load-more gap', SRC + 'table.rs',
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)[\s\S]{0,40}?\.w_full\(\)[\s\S]{0,40}?//', None),
    ('table', '.table__load-more-content', 'py', 'Table load-more py', SRC + 'table.rs',
     r'`\.table__load-more-content` is `gap-2 py-2`\.[\s\S]{0,40}?\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('pagination', '.pagination__content', 'gap', 'Pagination row gap',
     SRC + 'pagination.rs',
     r'is `gap-1`[\s\S]{0,120}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('pagination', '.pagination__link', 'size', 'Pagination cell Md', SRC + 'pagination.rs',
     r'Size::Md => px\((\d+(?:\.\d*)?)\.\)', None),
    ('pagination', '.pagination__link--nav', 'gap', 'Pagination nav gap',
     SRC + 'pagination.rs',
     r'`w-auto gap-1\.5 px-2\.5`[\s\S]{0,120}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('pagination', '.pagination__link--nav', 'px', 'Pagination nav px',
     SRC + 'pagination.rs',
     r'`w-auto gap-1\.5 px-2\.5`[\s\S]{0,160}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # --- Chip -------------------------------------------------------------
    ('chip', '.chip', 'px', 'Chip Md px', SRC + 'chip.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip', 'py', 'Chip Md py', SRC + 'chip.rs',
     r'Size::Md => \(px\(8\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip', 'text', 'Chip Md text', SRC + 'chip.rs',
     r'Size::Md => \(px\(8\.\), px\(2\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--sm', 'px', 'Chip Sm px', SRC + 'chip.rs',
     r'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--sm', 'py', 'Chip Sm py', SRC + 'chip.rs',
     r'Size::Sm => \(px\(4\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--lg', 'px', 'Chip Lg px', SRC + 'chip.rs',
     r'Size::Lg => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--lg', 'py', 'Chip Lg py', SRC + 'chip.rs',
     r'Size::Lg => \(px\(12\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip--lg', 'text', 'Chip Lg text', SRC + 'chip.rs',
     r'Size::Lg => \(px\(12\.\), px\(4\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('chip', '.chip__label', 'px', 'Chip label px', SRC + 'chip.rs',
     r'`\.chip__label` is `px-0\.5`\.[\s\S]{0,60}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- The field groups -------------------------------------------------
    # Every v3 field is one height (`h-9`) and one radius (`rounded-field`); the
    # number field's steppers are `w-10` slots inside that box.
    ('number-field', '.number-field__group', 'h', 'NumberField group height',
     SRC + 'number_field.rs', r'let h = crate::util::(FIELD_HEIGHT)', lambda _: 36.0),
    ('number-field', '.number-field__decrement-button', 'w', 'NumberField stepper width',
     SRC + 'number_field.rs', r'let btn_px = px\((\d+(?:\.\d*)?)\.\)', None),
    ('number-field', '.number-field__group', 'text', 'NumberField group text',
     SRC + 'number_field.rs', r'\.text_size\(crate::util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('input-group', '.input-group', 'min_h', 'InputGroup height', SRC + 'input_group.rs',
     r'\.min_h\(util::(FIELD_HEIGHT)\)', lambda _: 36.0),
    ('input-group', '.input-group', 'text', 'InputGroup text', SRC + 'input_group.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('input-group', '.input-group__prefix', 'px', 'InputGroup addon px',
     SRC + 'input_group.rs', r'\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('input-group', '.input-group__input', 'px', 'Input px', SRC + 'input.rs',
     r'None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # --- Badge and Tag ----------------------------------------------------
    # Both are padding-driven boxes with a radius *step* per size, not pills:
    # `rounded_full` on a 32px badge is not `rounded-2xl`, and a tag has no
    # height of its own at all.
    ('badge', '.badge', 'min_h', 'Badge Md box', SRC + 'badge.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge--sm', 'min_h', 'Badge Sm box', SRC + 'badge.rs',
     r'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge--lg', 'min_h', 'Badge Lg box', SRC + 'badge.rs',
     r'Size::Lg => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge', 'text', 'Badge Md text', SRC + 'badge.rs',
     r'Size::Md => \(px\(28\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge--lg', 'text', 'Badge Lg text', SRC + 'badge.rs',
     r'Size::Lg => \(px\(32\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('badge', '.badge', 'radius', 'Badge Md -> util::_radius', SRC + 'badge.rs',
     r'Size::Md => \(px\(28\.\), px\(12\.\), crate::util::(\w+_radius)', helper_px),
    ('badge', '.badge--sm', 'radius', 'Badge Sm -> util::_radius', SRC + 'badge.rs',
     r'Size::Sm => \(px\(16\.\), px\(10\.\), crate::util::(\w+_radius)', helper_px),
    ('badge', '.badge--lg', 'radius', 'Badge Lg -> util::_radius', SRC + 'badge.rs',
     r'Size::Lg => \(px\(32\.\), px\(14\.\), crate::util::(\w+_radius)', helper_px),
    ('badge', '.badge', 'gap', 'Badge gap', SRC + 'badge.rs',
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)[\s\S]{0,20}?\.rounded\(radius\)', None),
    ('badge', '.badge__label', 'px', 'Badge label px', SRC + 'badge.rs',
     r'gpui::div\(\)\.px\(px\((\d+(?:\.\d*)?)\.\)\)\.child\(content\)', None),
    ('tag', '.tag', 'gap', 'Tag gap', SRC + 'tag_group.rs',
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)[\s\S]{0,20}?\.px\(pad_x\)', None),
    ('tag', '.tag--sm', 'px', 'Tag Sm px', SRC + 'tag_group.rs',
     r'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--sm', 'py', 'Tag Sm py', SRC + 'tag_group.rs',
     r'Size::Sm => \(px\(8\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--sm', 'text', 'Tag Sm text', SRC + 'tag_group.rs',
     r'Size::Sm => \(px\(8\.\), px\(2\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--md', 'px', 'Tag Md px', SRC + 'tag_group.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--md', 'py', 'Tag Md py', SRC + 'tag_group.rs',
     r'Size::Md => \(px\(8\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--md', 'text', 'Tag Md text', SRC + 'tag_group.rs',
     r'Size::Md => \(px\(8\.\), px\(4\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--lg', 'px', 'Tag Lg px', SRC + 'tag_group.rs',
     r'Size::Lg => \(px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--lg', 'py', 'Tag Lg py', SRC + 'tag_group.rs',
     r'Size::Lg => \(px\(10\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag--lg', 'text', 'Tag Lg text', SRC + 'tag_group.rs',
     r'Size::Lg => \(px\(10\.\), px\(6\.\), px\((\d+(?:\.\d*)?)\.\)', None),
    ('tag', '.tag', 'radius', 'Tag Sm/Md -> util::_radius', SRC + 'tag_group.rs',
     r'Size::Sm \| Size::Md => crate::util::(\w+_radius)', helper_px),
    ('tag', '.tag--lg', 'radius', 'Tag Lg -> util::_radius', SRC + 'tag_group.rs',
     r'Size::Lg => crate::util::(\w+_radius)', helper_px),
    ('tag', '.tag__remove-button', 'size', 'Tag remove button', SRC + 'tag_group.rs',
     r'is `size-3`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # --- The three dialogs ------------------------------------------------
    # v3's dialog is one padded box with unpadded parts, and the spacing between
    # them comes from `+` rules rather than a gap. This port had a padded header,
    # a padded body and a padded footer with a separator between them -- a shape
    # v3 does not have -- so every number here was wrong at once.
    ('modal', '.modal__dialog', 'p', 'Modal dialog p-6', SRC + 'modal.rs',
     r'\.when_some\(self\.size\.max_width\(\), \|e, w\| e\.max_w\(w\)\)\s*\n\s*\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__header', 'gap', 'Modal header gap-3', SRC + 'modal.rs',
     r'let header = if self\.title[\s\S]*?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__heading', 'text', 'Modal heading text-base', SRC + 'modal.rs',
     r'let header = if self\.title[\s\S]*?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__body', 'text', 'Modal body text-sm', SRC + 'modal.rs',
     r'\.id\("modal-body"\)[\s\S]*?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__footer', 'gap', 'Modal footer gap-2', SRC + 'modal.rs',
     r'v3\'s sheet[\s\S]*?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__dialog', 'p', 'AlertDialog dialog p-6',
     SRC + 'alert_dialog.rs',
     r'\.when_some\(self\.size\.max_width\(\), \|e, w\| e\.max_w\(w\)\)\s*\n\s*\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__header', 'gap', 'AlertDialog header gap-3',
     SRC + 'alert_dialog.rs',
     r'let mut header = div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__heading', 'text', 'AlertDialog heading text-base',
     SRC + 'alert_dialog.rs',
     r'header\.child\(\s*\n\s*div\(\)\s*\n\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__icon', 'size', 'AlertDialog icon size-10',
     SRC + 'alert_dialog.rs',
     r'\.flex_shrink_0\(\)\s*\n\s*\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__icon', 'radius', 'AlertDialog icon rounded-3xl',
     SRC + 'alert_dialog.rs',
     r'\.size\(px\(40\.\)\)\s*\n\s*\.rounded\(util::(\w+_radius)\(cx\)\)', helper_px),
    ('alert-dialog', '.alert-dialog__footer', 'gap', 'AlertDialog footer gap-2',
     SRC + 'alert_dialog.rs',
     r'\.justify_end\(\)\s*\n\s*\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\s*\n\s*//', None),
    ('drawer', '.drawer__dialog', 'p', 'Drawer dialog p-6', SRC + 'drawer.rs',
     r'let mut panel = gpui::div\(\)[\s\S]*?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('drawer', '.drawer__header', 'gap', 'Drawer header gap-3', SRC + 'drawer.rs',
     r'let mut header = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('drawer', '.drawer__heading', 'text', 'Drawer heading text-base', SRC + 'drawer.rs',
     r'header = header\.child\([\s\S]*?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('drawer', '.drawer__body', 'text', 'Drawer body text-sm', SRC + 'drawer.rs',
     r'\.when\(has_header, \|b\| b\.mt\(px\(8\.\)\)\)\s*\n\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('drawer', '.drawer__footer', 'gap', 'Drawer footer gap-2', SRC + 'drawer.rs',
     r'not in v3\'s sheet[\s\S]*?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
    ('avatar', '.avatar', 'radius', 'Avatar Md/Lg -> util::_radius', SRC + 'avatar.rs',
     r'if self\.small \{[\s\S]{0,120}?\} else \{[\s\S]{0,40}?crate::util::(\w+_radius)',
     helper_px),
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
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)[\s\S]{0,400}?px\(36\.\)', helper_px),
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
    ('popover', '.popover__dialog', 'p', 'Popover padding', SRC + 'popover.rs',
     '\\.px\\(px\\((\\d+(?:\\.\\d*)?)\\)\\)\\s+\\.py\\(px\\(16\\.\\)\\)', None),
    ('pagination', '.pagination', 'gap', 'Pagination root gap', SRC + 'pagination.rs',
     r'\.justify_between\(\)[\s\S]{0,40}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
    # A row is `min_h(row_h)` on the plain path and a fixed height on the virtual
    # one, so the two arms of that `match` sit between the height and the padding.
    ('list-box-item', '.list-box-item', 'py', 'ListBox row padding_y', SRC + 'list_box.rs',
     r'\.min_h\(row_h\),[\s\S]{0,60}?\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
    ('tabs', '.tabs', 'gap', 'Tabs root gap', SRC + 'tabs.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
    ('toggle-button', '.toggle-button', 'h', 'ToggleButton md height',
     SRC + 'toggle_button.rs',
     r'Size::Md => \(px\((\d+(?:\.\d*)?)\.\), px\(16\.\)', None),
    ('toggle-button', '.toggle-button--sm', 'h', 'ToggleButton sm height',
     SRC + 'toggle_button.rs',
     r'Size::Sm => \(px\((\d+(?:\.\d*)?)\.\), px\(12\.\)', None),
    ('toggle-button', '.toggle-button--lg', 'h', 'ToggleButton lg height',
     SRC + 'toggle_button.rs',
     r'Size::Lg => \(px\((\d+(?:\.\d*)?)\.\), px\(20\.\)', None),
    ('toggle-button-group', '.toggle-button-group--detached', 'gap',
     'ToggleButtonGroup detached gap', SRC + 'toggle_button.rs',
     r'is_detached \{ px\((\d+(?:\.\d*)?)\.\) \} else \{ px\(0\.\) \}', None),
    ('accordion', '.accordion__trigger', 'px', 'Accordion trigger padding_x',
     SRC + 'accordion.rs',
     r'`\.accordion__trigger` is `px-4 py-4`\.\s+\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('accordion', '.accordion__trigger', 'py', 'Accordion trigger padding_y',
     SRC + 'accordion.rs',
     r'\.px\(px\(16\.\)\)\s+\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # `ListLayout`'s `padding` prop overrides it, so the stylesheet's value lives
    # in the field's default rather than at the call site.
    ('list-box', '.list-box', 'p', 'ListBox padding default', SRC + 'list_box.rs',
     r'padding: px\((\d+(?:\.\d*)?)\.\),', None),

    # --- the shared field parts: one struct in `field.rs` each ----------------
    ('label', '.label', 'text', 'Label text', SRC + 'field.rs',
     r'impl RenderOnce for Label[\s\S]{0,400}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('description', '.description', 'text', 'Description text', SRC + 'field.rs',
     r'impl RenderOnce for Description[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('error-message', '.error-message', 'text', 'ErrorMessage text', SRC + 'field.rs',
     r'impl RenderOnce for ErrorMessage[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    # A `FieldError` renders an `ErrorMessage`, so it is the same size ...
    ('field-error', '.field-error', 'text', 'FieldError text -> ErrorMessage', SRC + 'field.rs',
     r'impl RenderOnce for ErrorMessage[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    # ... but only `.field-error` carries the `px-1`.
    ('field-error', '.field-error', 'px', 'FieldError px', SRC + 'field.rs',
     r'`\.field-error` is `px-1`[\s\S]{0,120}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('fieldset', '.fieldset__legend', 'text', 'Fieldset legend text', SRC + 'field.rs',
     r'impl RenderOnce for FieldsetLegend[\s\S]{0,200}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('fieldset', '.fieldset__actions', 'gap', 'Fieldset actions gap', SRC + 'field.rs',
     r'impl FieldsetActions \{[\s\S]{0,160}?gap: px\((\d+(?:\.\d*)?)\.\)', None),

    # --- a collection's section header and its empty state -------------------
    ('header', '.header', 'px', 'Section header px', SRC + 'list_box.rs',
     r'ListBoxItem::Section\(label\)[\s\S]{0,200}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('header', '.header', 'text', 'Section header text', SRC + 'list_box.rs',
     r'ListBoxItem::Section\(label\)[\s\S]{0,300}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('empty-state', '.empty-state', 'p', 'Empty state padding', SRC + 'tag_group.rs',
     r'`\.empty-state` is `p-2 text-sm text-muted`\.[\s\S]{0,60}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('empty-state', '.empty-state', 'text', 'Empty state text', SRC + 'tag_group.rs',
     r'`\.empty-state` is `p-2 text-sm text-muted`\.'
     r'[\s\S]{0,120}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- the dialogs' padded container, and the modal icon -------------------
    ('modal', '.modal__container', 'p', 'Modal container padding', SRC + 'modal.rs',
     r'`\.modal__container` is `p-4 sm:p-10`\.\s*\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__container', 'p', 'AlertDialog container padding',
     SRC + 'alert_dialog.rs',
     r'`\.alert-dialog__container` is `p-4 sm:p-10`\.\s*\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__icon', 'size', 'Modal icon box', SRC + 'modal.rs',
     r'`\.modal__icon` is `size-10 rounded-3xl`[\s\S]{0,900}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('modal', '.modal__icon', 'radius', 'Modal icon radius -> control_radius', SRC + 'modal.rs',
     r'`\.modal__icon` is `size-10 rounded-3xl`[\s\S]{0,1000}?'
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('accordion', '.accordion__body', 'text', 'Accordion body text', SRC + 'accordion.rs',
     r'\.pt\(px\(2\.\)\)\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- every field wrapper is `gap-1` --------------------------------------
    ('checkbox', '.checkbox', 'gap', 'Checkbox content/description gap',
     SRC + 'checkbox.rs',
     r'`\.checkbox` is `gap-1` between its content and description\.\s*'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('checkbox', '.checkbox__content', 'text', 'Checkbox content text',
     SRC + 'checkbox.rs',
     r'\(box_px, icon_px, text\) = \(px\(16\.\), px\(12\.\), px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-field', '.color-field', 'gap', 'ColorField wrapper gap', SRC + 'color_picker.rs',
     r'`\.color-field` is `flex flex-col gap-1`\.\s*'
     r'let mut root = div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-slider', '.color-slider', 'gap', 'ColorSlider wrapper gap', SRC + 'color_picker.rs',
     r'`\.color-slider` is `grid w-full gap-1`\.\s*\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-field', '.date-field', 'gap', 'DateField wrapper gap', SRC + 'time_field.rs',
     r'`\.date-field` is `flex flex-col gap-1`\.\s*'
     r'let mut root = div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('dropdown', '.dropdown', 'gap', 'Dropdown wrapper gap', SRC + 'dropdown.rs',
     r'`\.dropdown` is `flex flex-col gap-1`\.\s*\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('combo-box', '.combo-box', 'gap', 'ComboBox wrapper gap', SRC + 'combo_box.rs',
     r'\.flex\(\)\s*\.flex_col\(\)\s*\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\s*'
     r'\.child\(input\.render', None),
    ('progress-bar', '.progress-bar', 'gap', 'ProgressBar wrapper gap', SRC + 'progress.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\.w_full\(\)',
     None),
    # A Meter renders a ProgressBar, so it is the same wrapper.
    ('meter', '.meter', 'gap', 'Meter wrapper gap -> ProgressBar', SRC + 'progress.rs',
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\.w_full\(\)',
     None),
    ('slider', '.slider', 'gap', 'Slider wrapper gap', SRC + 'slider.rs',
     r'let mut el = gpui::div\(\)\s*\.flex\(\)\s*\.flex_col\(\)\s*'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('switch', '.switch', 'gap', 'Switch content/description gap', SRC + 'switch.rs',
     r'so the text lines up under the label\.[\s\S]{0,300}?'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # An Autocomplete composes an Input, and the Input owns the label column.
    ('autocomplete', '.autocomplete', 'gap', 'Autocomplete wrapper gap -> Input', SRC + 'input.rs',
     r'-- wrapper with label / description / error -+\s*'
     r'let mut el = gpui::div\(\)\.flex\(\)\.flex_col\(\)\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-range-picker', '.date-range-picker', 'gap', 'DateRangePicker wrapper gap',
     SRC + 'date_picker.rs',
     r'let mut wrapper = gpui::div\(\)\.flex\(\)\.flex_col\(\)'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\.w_full\(\)', None),

    # --- a menu row ----------------------------------------------------------
    ('menu-item', '.menu-item', 'gap', 'Menu item gap', SRC + 'dropdown.rs',
     r'let mut row = gpui::div\(\)[\s\S]{0,200}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('menu-item', '.menu-item', 'min_h', 'Menu item min height', SRC + 'dropdown.rs',
     r'`\.menu-item` is `min-h-9 py-1\.5`[\s\S]{0,160}?'
     r'\.min_h\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('menu-item', '.menu-item', 'py', 'Menu item py', SRC + 'dropdown.rs',
     r'`\.menu-item` is `min-h-9 py-1\.5`[\s\S]{0,200}?\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('menu-item', '.menu-item__indicator', 'size', 'Menu item icon', SRC + 'dropdown.rs',
     r'`\.menu-item__indicator` is `size-4`\.\s*\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('list-box-item', '.list-box-item', 'min_h', 'ListBox row min height', SRC + 'list_box.rs',
     r'`\.list-box-item` is `min-h-9`\.\s*'
     r'let row_h = fixed_h\.unwrap_or\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('list-box-item', '.list-box-item', 'px', 'ListBox row px', SRC + 'list_box.rs',
     r'\.gap\(px\(12\.\)\)\s*\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- the three 24/20px icon buttons: close, clear, toast-close ------------
    ('close-button', '.close-button', 'p', 'CloseButton padding', SRC + 'close_button.rs',
     r'\.size\(box_size\)\s*\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('autocomplete', '.autocomplete__clear-button', 'size', 'Autocomplete clear box',
     SRC + 'autocomplete.rs',
     r'and then `size-5`, so 20px[\s\S]{0,200}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('autocomplete', '.autocomplete__clear-button', 'p', 'Autocomplete clear padding',
     SRC + 'autocomplete.rs',
     r'and then `size-5`, so 20px[\s\S]{0,260}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('autocomplete', '.autocomplete__clear-button', 'radius',
     'Autocomplete clear radius -> small_radius', SRC + 'autocomplete.rs',
     r'and then `size-5`, so 20px[\s\S]{0,320}?'
     r'\.rounded\((?:crate::)?util::(\w+_radius)\(cx\)\)', helper_px),
    # --- the shared field radius and type size, per sheet ---------------------
    ('input-group', '.input-group', 'radius', 'input-group radius -> field_radius',
     SRC + 'util.rs', r'pub fn (field_radius)', helper_px),
    ('textarea', '.textarea', 'radius', 'textarea radius -> field_radius',
     SRC + 'util.rs', r'pub fn (field_radius)', helper_px),
    ('number-field', '.number-field__group', 'radius', 'number-field radius -> field_radius',
     SRC + 'util.rs', r'pub fn (field_radius)', helper_px),
    ('date-picker', '.date-picker__trigger', 'radius', 'date-picker radius -> field_radius',
     SRC + 'util.rs', r'pub fn (field_radius)', helper_px),
    ('date-range-picker', '.date-range-picker__trigger', 'radius',
     'date-range-picker radius -> field_radius', SRC + 'util.rs',
     r'pub fn (field_radius)', helper_px),
    ('input-group', '.input-group__input', 'text', 'input-group text -> FIELD_TEXT',
     SRC + 'input_group.rs', r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('number-field', '.number-field__input', 'text', 'number-field text -> FIELD_TEXT',
     SRC + 'number_field.rs',
     r'\.text_size\(crate::util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('combo-box', '.combo-box__value', 'text', 'combo-box value text -> FIELD_TEXT',
     SRC + 'combo_box.rs', r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('color-input-group', '.color-input-group__input', 'text',
     'color-input-group text -> FIELD_TEXT', SRC + 'color_picker.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('date-input-group', '.date-input-group__input', 'text',
     'date-input-group text -> FIELD_TEXT', SRC + 'time_field.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('search-field', '.search-field__group', 'text', 'search-field text -> FIELD_TEXT',
     SRC + 'input.rs',
     r'let \(h, text\) = \(crate::util::FIELD_HEIGHT, crate::util::(FIELD_TEXT)\)',
     lambda _: 14.0),
    ('search-field', '.search-field__input', 'text', 'search-field input -> FIELD_TEXT',
     SRC + 'input.rs',
     r'let \(h, text\) = \(crate::util::FIELD_HEIGHT, crate::util::(FIELD_TEXT)\)',
     lambda _: 14.0),
    ('textarea', '.textarea', 'text', 'textarea text -> FIELD_TEXT', SRC + 'input.rs',
     r'let \(h, text\) = \(crate::util::FIELD_HEIGHT, crate::util::(FIELD_TEXT)\)',
     lambda _: 14.0),
    ('input', '.input', 'px', 'Input padding_x', SRC + 'input.rs',
     r'`\.input-group__input` keeps `px-3`[\s\S]{0,200}?None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('input-group', '.input-group__suffix', 'px', 'InputGroup addon px', SRC + 'input_group.rs',
     r'`__suffix`: `px-3`[\s\S]{0,200}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- a popover type size, wherever the rows set it ------------------------
    ('popover', '.popover', 'text', 'Popover text', SRC + 'popover.rs',
     r'`\.popover` is `text-sm`\.\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('select', '.select__popover', 'text', 'Select popover row text -> FIELD_TEXT',
     SRC + 'select.rs', r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('combo-box', '.combo-box__popover', 'text', 'ComboBox popover row text -> FIELD_TEXT',
     SRC + 'combo_box.rs', r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('autocomplete', '.autocomplete__popover', 'text',
     'Autocomplete popover row text -> FIELD_TEXT', SRC + 'autocomplete.rs',
     r'\.text_size\(util::(FIELD_TEXT)\)', lambda _: 14.0),
    ('dropdown', '.dropdown__popover', 'text', 'Dropdown row text', SRC + 'dropdown.rs',
     r'let mut row = gpui::div\(\)[\s\S]{0,420}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('alert-dialog', '.alert-dialog__body', 'text', 'AlertDialog body text',
     SRC + 'alert_dialog.rs',
     r'`\.alert-dialog__body` is `text-sm[\s\S]{0,240}?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),

    # --- the odds and ends ----------------------------------------------------
    ('kbd', '.kbd', 'radius', 'Kbd radius -> key_radius', SRC + 'util.rs',
     r'pub fn (key_radius)', helper_px),
    ('avatar', '.avatar--lg', 'radius', 'Avatar Lg -> control_radius', SRC + 'avatar.rs',
     r'\} else \{\s*crate::util::(\w+_radius)\(cx\)', helper_px),
    ('switch', '.switch__control', 'radius', 'Switch track radius Md', SRC + 'switch.rs',
     r'Size::Md => \(px\(40\.\), px\(20\.\), px\(22\.\), px\(16\.\), '
     r'px\((\d+(?:\.\d*)?)\.\)', None),
    ('switch', '.switch__thumb', 'radius', 'Switch thumb radius Md', SRC + 'switch.rs',
     r'Size::Md => \(px\(40\.\), px\(20\.\), px\(22\.\), px\(16\.\), px\(12\.\), '
     r'px\((\d+(?:\.\d*)?)\.\)', None),
    ('radio', '.radio__content', 'text', 'Radio content text', SRC + 'radio_group.rs',
     r'let \(circle, dot, text, gap\) = \(px\(16\.\), px\(6\.\), px\((\d+(?:\.\d*)?)\.\)',
     None),
    ('list-box-item', '.list-box-item__indicator', 'size', 'ListBox check size',
     SRC + 'list_box.rs',
     r'`\.list-box-item__indicator` is `size-4`\.\s*\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('toggle-button', '.toggle-button--lg', 'text', 'Size::text_size Lg', CORE,
     r'Label size[\s\S]*?Size::Lg => gpui::px\((\d+(?:\.\d*)?)\)', None),
    ('toggle-button-group', '.toggle-button-group__separator', 'radius',
     'ToggleButtonGroup separator -> hairline_radius', SRC + 'toggle_button.rs',
     r'let separator_radius = crate::util::(\w+_radius)\(cx\)', helper_px),
    ('tabs', '.tabs__separator', 'radius', 'Tabs separator -> hairline_radius', SRC + 'tabs.rs',
     r'`\.tabs__separator` is a `w-px h-1/2 rounded-sm[\s\S]{0,900}?'
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('tabs', '.tabs__indicator', 'radius', 'Tabs selected segment -> control_radius',
     SRC + 'tabs.rs',
     r'`\.tabs__tab` is `h-8 px-4 rounded-3xl text-sm[\s\S]{0,300}?'
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('pagination', '.pagination__summary', 'gap', 'Pagination summary gap', SRC + 'pagination.rs',
     r'`\.pagination__summary` is `gap-2 text-sm text-muted`\.\s*'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('pagination', '.pagination__summary', 'text', 'Pagination summary text',
     SRC + 'pagination.rs',
     r'`\.pagination__summary` is `gap-2 text-sm text-muted`\.[\s\S]{0,120}?'
     r'\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('pagination', '.pagination__link', 'radius', 'Pagination link -> control_radius',
     SRC + 'pagination.rs',
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('pagination', '.pagination__ellipsis', 'size', 'Pagination ellipsis cell',
     SRC + 'pagination.rs',
     r'`\.pagination__ellipsis` is the same `size-8[\s\S]{0,160}?\.size\((cell)\)',
     lambda _: 32.0),
    ('pagination', '.pagination__ellipsis', 'text', 'Pagination ellipsis text',
     SRC + 'pagination.rs',
     r'`\.pagination__ellipsis` is the same `size-8[\s\S]{0,200}?\.text_size\((cell_text)\)',
     lambda _: 14.0),
    ('calendar', '.calendar__cell-indicator', 'radius', 'Calendar cell indicator radius',
     SRC + 'calendar.rs',
     r'`\.calendar__cell-indicator` is `size-\[3px\]`[\s\S]{0,300}?'
     r'\.rounded\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__header', 'px', 'RangeCalendar header px',
     SRC + 'range_calendar.rs',
     r'`\.range-calendar__header` is `px-0\.5`\.\s*\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__cell', 'radius',
     'RangeCalendar cell -> control_radius', SRC + 'range_calendar.rs',
     r'\.rounded\(util::(control_radius)\(cx\)\)', helper_px),

    # --- colour: the swatch shapes, the picker item and its trigger -----------
    ('color-swatch', '.color-swatch--circle', 'radius', 'ColorSwatch circle is edge/2',
     SRC + 'color_picker.rs',
     r'SwatchShape::Circle => px\(f32::from\(edge\) / (\d+)\.\)',
     # The base rule is the `md` swatch, 32px across.
     lambda half: 32.0 / float(half)),
    ('color-swatch', '.color-swatch--square', 'radius', 'ColorSwatch square -> radius_md',
     SRC + 'color_picker.rs',
     r'SwatchShape::Square => cx\.layout\(\)\.radius_(\w+)\(\)', lambda step: RADIUS[step]),
    ('color-swatch-picker', '.color-swatch-picker__item', 'size', 'Swatch picker item box',
     SRC + 'color_picker.rs',
     r'`\.color-swatch-picker__item` is `size-8 rounded-2xl border-2`\.\s*'
     r'\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-swatch-picker', '.color-swatch-picker__item', 'radius',
     'Swatch picker item -> soft_radius', SRC + 'color_picker.rs',
     r'`\.color-swatch-picker__item` is `size-8 rounded-2xl border-2`\.[\s\S]{0,120}?'
     r'\.rounded\(util::(\w+_radius)\(cx\)\)', helper_px),
    ('color-swatch-picker', '.color-swatch-picker__item', 'border', 'Swatch picker item border',
     SRC + 'color_picker.rs',
     r'`\.color-swatch-picker__item` is `size-8 rounded-2xl border-2`\.[\s\S]{0,200}?'
     r'\.border_(\d)\(\)', None),
    ('color-picker', '.color-picker__trigger', 'gap', 'ColorPicker trigger gap',
     SRC + 'color_picker.rs',
     r'`\.color-picker__trigger` is `inline-flex items-center gap-3[\s\S]{0,240}?'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-picker', '.color-picker__trigger', 'radius',
     'ColorPicker trigger -> hairline_radius', SRC + 'color_picker.rs',
     r'`\.color-picker__trigger` is `inline-flex items-center gap-3[\s\S]{0,300}?'
     r'\.rounded\(util::(\w+_radius)\(cx\)\)', helper_px),
    ('color-picker', '.color-picker__trigger', 'text', 'ColorPicker trigger text',
     SRC + 'color_picker.rs',
     r'`\.color-picker__trigger` is `inline-flex items-center gap-3[\s\S]{0,340}?'
     r'\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('color-area', '.color-area__thumb', 'radius', 'ColorArea thumb radius',
     SRC + 'color_picker.rs',
     r'`\.color-area__thumb` is `rounded-xl`[\s\S]{0,200}?'
     r'\.rounded\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- date and time --------------------------------------------------------
    ('date-picker', '.date-picker__trigger', 'text', 'DatePicker trigger text',
     SRC + 'date_picker.rs',
     r'\.px\(px\(12\.\)\)\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-range-picker', '.date-range-picker__trigger', 'text',
     'DateRangePicker trigger text', SRC + 'date_picker.rs',
     r'\.px\(px\(12\.\)\)\s*\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-picker', '.date-picker__trigger-indicator', 'size', 'DatePicker trigger glyph',
     SRC + 'date_picker.rs',
     r'`\.date-picker__trigger-indicator` is `size-4`\.\s*\.size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('date-range-picker', '.date-range-picker__trigger-indicator', 'size',
     'DateRangePicker trigger glyph', SRC + 'date_picker.rs',
     r'`\.date-range-picker__trigger-indicator` is `size-4`\.\s*'
     r'\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-input-group', '.date-input-group__segment', 'px', 'Date segment px',
     SRC + 'time_field.rs',
     r'`\.date-input-group__segment` is `rounded-md px-0\.5`\.\s*'
     r'\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('date-input-group', '.date-input-group__segment', 'radius',
     'Date segment -> radius_md', SRC + 'time_field.rs',
     r'`\.date-input-group__segment` is `rounded-md px-0\.5`\.[\s\S]{0,160}?'
     r'\.rounded\(cx\.layout\(\)\.radius_(\w+)\(\)\)', lambda step: RADIUS[step]),
    ('calendar-year-picker', '.calendar-year-picker__year-cell', 'radius',
     'Year cell -> control_radius', SRC + 'calendar.rs',
     r'`h-8 px-2\.5[\s\S]{0,400}?\.rounded\(crate::util::(\w+_radius)\(cx\)\)', helper_px),
    ('calendar-year-picker', '.calendar-year-picker__trigger-indicator', 'text',
     'Year trigger chevron (v3 sizes it with text-xs)', SRC + 'calendar.rs',
     r'\.size\(px\((\d+(?:\.\d*)?)\.\)\)\s*\.path\(if open \{\s*icons::CHEVRON_UP', None),

    # --- what a Disclosure borrows from the Accordion -------------------------
    ('disclosure', '.disclosure__indicator', 'size', 'Disclosure -> Accordion indicator',
     SRC + 'accordion.rs',
     r'`\.accordion__indicator` is `size-4`\.[\s\S]{0,40}?\.size\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),

    # --- the rest -------------------------------------------------------------
    ('switch-group', '.switch-group__items', 'gap', 'SwitchGroup items gap', SRC + 'switch.rs',
     r'impl RenderOnce for SwitchGroup[\s\S]{0,700}?'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)\s*\.children\(self\.items\)', None),
    ('table', '.table__sortable-column-indicator', 'size', 'Table sort chevron',
     SRC + 'table.rs',
     r'None => gpui::svg\(\)\s*\.size\(px\((\d+(?:\.\d*)?)\.\)\)\s*'
     r'\.path\(descriptor\.direction\.indicator\(\)\)', None),
    ('pagination', '.pagination__link', 'text', 'Pagination link -> Size::text_size Md', CORE,
     r'Label size[\s\S]{0,200}?Size::Md => gpui::px\((\d+(?:\.\d*)?)\)', None),

    ('select', '.select__trigger', 'px', 'Select trigger px', SRC + 'select.rs',
     r'\.min_h\(h\)\s*\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    # An Autocomplete composes an Input, so the trigger is the field.
    ('autocomplete', '.autocomplete__trigger', 'px', 'Autocomplete trigger px -> Input',
     SRC + 'input.rs',
     r'`\.input-group__input` keeps `px-3`[\s\S]{0,200}?None => f\.px\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('toast', '.toast__indicator', 'p', 'Toast indicator padding', SRC + 'toast.rs',
     r'`\.toast__indicator` — `flex shrink-0 items-center justify-center p-1`'
     r'[\s\S]{0,400}?\.p\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__footer', 'px', 'Table footer px', SRC + 'table.rs',
     r'`\.table__footer` is `flex items-center px-4 py-2\.5`\.[\s\S]{0,200}?'
     r'\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('table', '.table__footer', 'py', 'Table footer py', SRC + 'table.rs',
     r'`\.table__footer` is `flex items-center px-4 py-2\.5`\.[\s\S]{0,240}?'
     r'\.py\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    ('table', '.table-root--primary', 'px', 'Table tray px', SRC + 'table.rs',
     r'`\.table-root--primary` is a `bg-surface-secondary px-1 pb-1` tray'
     r'[\s\S]{0,600}?\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    # --- the four parts this port used to skip -------------------------------
    ('date-range-picker', '.date-range-picker__range-separator', 'px',
     'Range separator px', SRC + 'date_picker.rs',
     r'`\.date-range-picker__range-separator` is `px-1`[\s\S]{0,200}?'
     r'\.px\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('radio', '.radio', 'gap', 'Radio content/description gap', SRC + 'radio_group.rs',
     r'`\.radio` is `flex flex-col gap-1` around its content and the[\s\S]{0,420}?'
     r'\.gap\(px\((\d+(?:\.\d*)?)\.\)\)',
     None),
    ('separator', '.separator__container', 'gap', 'Separator container gap',
     SRC + 'separator.rs',
     r'`\.separator__container` is `flex items-center gap-3`[\s\S]{0,300}?'
     r'\.gap\(gpui::px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__cell-indicator', 'size',
     'RangeCalendar cell indicator', SRC + 'range_calendar.rs',
     r'`\.range-calendar__cell-indicator` is a `size-\[3px\][\s\S]{0,600}?'
     r'\.size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('range-calendar', '.range-calendar__cell-indicator', 'radius',
     'RangeCalendar cell indicator radius', SRC + 'range_calendar.rs',
     r'`\.range-calendar__cell-indicator` is a `size-\[3px\][\s\S]{0,660}?'
     r'\.rounded\(px\((\d+(?:\.\d*)?)\.\)\)', None),

    ('toast', '.toast__close-button', 'border', 'Toast close button border', SRC + 'toast.rs',
     r'`sm:border\s*//\s*border-border sm:bg-overlay`[\s\S]{0,200}?'
     r'\.border\(cx\.layout\(\)\.(border_width)\)', lambda _: 1.0),
]


THEME_FILES = (
    ('variables.css', 'https://raw.githubusercontent.com/heroui-inc/heroui/v3'
                      '/packages/styles/themes/default/variables.css'),
    ('shared_theme.css', 'https://raw.githubusercontent.com/heroui-inc/heroui/v3'
                         '/packages/styles/themes/shared/theme.css'),
)


def fetch():
    os.makedirs(CACHE, exist_ok=True)
    for name, url in THEME_FILES:
        subprocess.run(['curl', '-sL', '--max-time', '30', '-o',
                        os.path.join(CACHE, name), url], check=False)
    names = sorted({c for c, *_ in CHECKS})
    for name in names:
        subprocess.run(['curl', '-sL', '--max-time', '30', '-o',
                        os.path.join(CACHE, name + '.css'), COMPONENTS % name],
                       check=False)
    print('fetched %d stylesheets into %s' % (len(names), CACHE))


_WIDTH_VARS = None


def width_vars():
    """`--*-width` declarations, from the theme sheets, first (light) wins."""
    global _WIDTH_VARS
    if _WIDTH_VARS is None:
        _WIDTH_VARS = {}
        for name, _ in THEME_FILES:
            path = os.path.join(CACHE, name)
            if not os.path.exists(path):
                continue
            text = io.open(path, encoding='utf-8', errors='replace').read()
            for m in re.finditer(r'(--[\w-]*width[\w-]*):\s*([^;]+);', text):
                _WIDTH_VARS.setdefault(m.group(1), m.group(2).strip())
    return _WIDTH_VARS


def resolve_width(expr):
    """`var(--border-width-field)` -> 0.0.

    A field's border is the reason this exists. Every field sheet applies
    Tailwind's bare `border` -- 1px -- and then overrides the width with
    `var(--border-width-field)`, which chains through `--field-border-width: 0px`
    to nothing at all. Reading the utility alone claims a 1px border on ten
    components that draw none, and v3's field states are rings for exactly that
    reason.
    """
    for _ in range(8):
        expr = expr.strip()
        m = re.fullmatch(r'(\d+(?:\.\d*)?)px', expr)
        if m:
            return float(m.group(1))
        if expr == '0':
            return 0.0
        m = re.fullmatch(r'var\((--[\w-]+)\s*(?:,\s*(.*))?\)', expr, re.S)
        if not m:
            return None
        value = width_vars().get(m.group(1))
        if value is None:
            value = m.group(2)
        if value is None:
            return None
        expr = value
    return None


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
            # An arbitrary utility carries its own colon --
            # `[border-width:var(--border-width-field)]` -- so only split on one
            # that is outside the brackets.
            if ':' in tok and not tok.startswith('['):
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

    # A border width is sometimes a utility (`border-2`) and sometimes plain CSS
    # -- the colour-area thumb is `border: 3px solid white`, which no `@apply`
    # can spell. Both are the same metric; Tailwind's border scale is in pixels,
    # not spacing steps, so `border-2` is 2px rather than 8.

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
            m = re.fullmatch(r'border-(\d+(?:\.\d*)?)', tok)
            if m:
                offer('border', float(m.group(1)), bp)
            elif tok == 'border':
                offer('border', 1.0, bp)
            else:
                m = re.fullmatch(r'\[border-width:(.+)\]', tok)
                if m:
                    v = resolve_width(m.group(1))
                    if v is not None:
                        offer('border', v, bp)
            if tok.startswith('rounded-') and tok[8:] in RADIUS:
                offer('radius', RADIUS[tok[8:]], bp)
            elif tok == 'rounded':
                offer('radius', RADIUS['lg'], bp)
            if tok.startswith('text-') and tok[5:] in TEXT:
                offer('text', TEXT[tok[5:]], bp)

    # `size-*` and `h-*`/`w-*` set the same properties, so a rule that applies
    # both keeps whichever comes last: `.autocomplete__clear-button` is
    # `h-6 w-6 ... ` and then `size-5`, which makes it 20px, not 24. Reading all
    # three claims a box that is two sizes at once.
    if 'size' in found and ('h' in found or 'w' in found):
        size_at = max((m.start() for m in re.finditer(r'size-\d', body)), default=-1)
        hw_at = max((m.start() for m in re.finditer(r'[hw]-\d', body)), default=-1)
        if size_at > hw_at:
            found.pop('h', None)
            found.pop('w', None)
        else:
            found.pop('size', None)

    # `.modal__body` is `-m-[3px] my-0 overflow-visible p-[3px]`: three pixels of
    # padding cancelled by three of negative margin, so a focus ring on the last
    # control inside it is not clipped. Net padding is zero, and reading the
    # `p-[3px]` alone claims an inset the dialog does not have.
    m = re.search(r'-m-\[(\d+(?:\.\d*)?)px\]', body)
    if m and found.get('p', (None,))[0] == float(m.group(1)):
        offer('p', 0.0, '')

    # A border width is sometimes a utility and sometimes a declaration, and the
    # declaration is what a field uses to override the utility -- so it is read
    # last, and at the same rank, which is what lets it win.
    m = re.search(r'border(?:-width)?:\s*([^;]+);', body)
    if m:
        v = resolve_width(m.group(1).split()[0] if 'var(' not in m.group(1)
                          else m.group(1))
        if v is not None:
            offer('border', v, '')
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


def coverage():
    """Every metric v3 declares, and whether `CHECKS` compares it.

    `CHECKS` is hand-written -- each row has to name where in the Rust the
    metric lives -- so the audit can only be as complete as the list, and
    nothing said how complete that was. This reads the sheets instead: every
    rule, every measurable utility in it, resolved through the same scales. What
    it reports is the audit's own coverage, which is the honest denominator.
    """
    checked = {(c, sel, metric) for c, sel, metric, *_ in CHECKS}
    rows = []
    # A declared *zero* is a reset, not a metric: `min-h-0`, `mt-0`, `p-0` and
    # `rounded-none` say "no minimum", "no margin", "no padding", which is what
    # an element that never sets them already does. Counting them as unchecked
    # geometry would put 38 rows on the list that no code can satisfy or fail.
    resets = 0
    for name in sorted(os.listdir(CACHE)):
        if not name.endswith('.css') or name in ('variables.css', 'utilities.css',
                                                 'shared_theme.css'):
            continue
        comp = name[:-4]
        css = io.open(os.path.join(CACHE, name), encoding='utf-8', errors='replace').read()
        for m in re.finditer(r'^\.([a-z0-9_-]+(?:__[a-z0-9-]+)?(?:--[a-z0-9-]+)?)\s*\{',
                             css, re.M):
            selector = '.' + m.group(1)
            body = rule_body(css, selector)
            if body is None:
                continue
            for metric, value in sorted(measure(body).items()):
                if value == 0.0 and (comp, selector, metric) not in checked:
                    resets += 1
                    continue
                rows.append((comp, selector, metric, value,
                             (comp, selector, metric) in checked))
    excused = [r for r in rows if not r[4] and (r[0], r[1], r[2]) in COVERED_ELSEWHERE]
    todo = [r for r in rows
            if not r[4] and (r[0], r[1], r[2]) not in COVERED_ELSEWHERE]
    if '--all' in sys.argv:
        for comp, selector, metric, value, ok in rows:
            reason = COVERED_ELSEWHERE.get((comp, selector, metric))
            mark = ' ' if ok else ('~' if reason else '!')
            print('%s %-22s %-24s %-7s %-6g %s'
                  % (mark, comp, selector, metric, value, reason or ''))
    else:
        by_comp = {}
        for comp, sel, metric, _v, ok in rows:
            have, total = by_comp.get(comp, (0, 0))
            done = ok or (comp, sel, metric) in COVERED_ELSEWHERE
            by_comp[comp] = (have + (1 if done else 0), total + 1)
        print('  %-24s %s' % ('sheet', 'checked / declared'))
        for comp in sorted(by_comp, key=lambda c: by_comp[c][1] - by_comp[c][0], reverse=True):
            have, total = by_comp[comp]
            mark = ' ' if have == total else '!'
            print('%s %-24s %d / %d' % (mark, comp, have, total))
    print()
    print('metrics v3 declares : %d' % len(rows))
    print('compared by CHECKS  : %d' % (len(rows) - len(todo) - len(excused)))
    print('declared resets     : %d  (a `-0` utility is not a metric)' % resets)
    print('covered elsewhere   : %d  (%s)'
          % (len(excused), ', '.join(sorted({r for r in COVERED_ELSEWHERE.values()}))))
    print('UNCHECKED           : %d  (--all lists them)' % len(todo))


def main():
    if '--fetch' in sys.argv:
        fetch()
        return
    if '--coverage' in sys.argv:
        coverage()
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
