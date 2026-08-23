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
     r'let header = self\.title[\s\S]*?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
    ('modal', '.modal__heading', 'text', 'Modal heading text-base', SRC + 'modal.rs',
     r'let header = self\.title[\s\S]*?\.text_size\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
     r'\.rounded\(crate::util::(\w+_radius)\(cx\)\)[\s\S]{0,400}?px\(32\.\)', helper_px),
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
    ('switch', '.switch__content', 'gap', 'Switch row gap', SRC + 'switch.rs',
     r'`\.switch__content` is `gap-3`\. [\s\S]{0,200}?\.gap\(px\((\d+(?:\.\d*)?)\.\)\)', None),
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
        if not name.endswith('.css') or name in ('variables.css', 'utilities.css'):
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
    todo = [r for r in rows if not r[4]]
    if '--all' in sys.argv:
        for comp, selector, metric, value, ok in rows:
            print('%s %-22s %-24s %-7s %g' % (' ' if ok else '!', comp, selector, metric, value))
    else:
        by_comp = {}
        for comp, _sel, _m, _v, ok in rows:
            have, total = by_comp.get(comp, (0, 0))
            by_comp[comp] = (have + (1 if ok else 0), total + 1)
        print('  %-24s %s' % ('sheet', 'checked / declared'))
        for comp in sorted(by_comp, key=lambda c: by_comp[c][1] - by_comp[c][0], reverse=True):
            have, total = by_comp[comp]
            mark = ' ' if have == total else '!'
            print('%s %-24s %d / %d' % (mark, comp, have, total))
    print()
    print('metrics v3 declares : %d' % len(rows))
    print('compared by CHECKS  : %d' % (len(rows) - len(todo)))
    print('declared resets     : %d  (a `-0` utility is not a metric)' % resets)
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
