"""What each component *contains*, and whether this port composes it.

Fourteen audits sat at zero while the Autocomplete was built inside out. v3's
Autocomplete is a Select whose popover holds a `SearchField`; this port had built
an `Input` with a suggestion panel -- the ComboBox's anatomy -- and every audit
passed, each for its own reason. `design_audit.py` measured the trigger against
the `Input` it composed and the numbers matched, because a field and a select
trigger are both `min-h-9 rounded-field px-3`. `part_audit.py` found the
selectors in comments. `api_audit.py` matched `inputValue` because it really is
an `Autocomplete.Filter` prop. A prop diff cannot see a structure.

The signal that can is already in the stylesheets. v3 marks every component's
root with `data-slot`, so a `[data-slot="X"]` selector *nested inside* sheet C is
v3 saying "a C contains an X":

    .autocomplete__popover {
      [data-slot="search-field"] { @apply shrink-0 px-3 py-1 outline-none; }
    }

That is the sentence the old Autocomplete contradicted. This reads all of them --
one claim per (sheet, contained component) -- and asks whether the module that
draws C mentions the symbol that draws X.

    python .shots/anatomy_audit.py          # the report
    python .shots/anatomy_audit.py --all    # every claim, met or not

`SLOT` maps a slot name to the symbol; a slot that names one of the sheet's own
parts (`autocomplete-clear-button-icon` under `.autocomplete`) is
`part_audit.py`'s business and is skipped here. `WONT_COMPOSE` records a claim
this port answers differently, with the reason.
"""
import io
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from state_audit import MODULE  # noqa: E402  (one table of sheet -> module)

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

CSS = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')
SRC = 'crates/herogpui-components/src/'

# The symbol that draws each contained component. A slot is only listed when v3
# nests it somewhere; the value is a regex searched in the containing module.
SLOT = {
    'search-field': r'SearchField::new',
    'list-box': r'list_nav|row_of|uniform_list|gpui::list|list_box::',
    'list-box-item': r'row_of|list_box::|FIELD_HEIGHT',
    'list-box-item-indicator': r'icons::CHECK|indicator',
    'menu-item': r'row|MenuItem',
    'dropdown-menu': r'menu',
    # A component either composes the shared part or draws the same thing
    # itself: `Meter` writes its own label element, and a `ComboBox` hands its
    # label to the `Input` it wraps (`input = input.label(..)`). Both are the
    # part being rendered; which of the two it is, `design_audit.py` measures.
    'label': r'field::Label|Label::new|\.label\(|label:',
    'description': r'field::Description|Description::new|description',
    'field-error': r'field::FieldError|FieldError::new|ErrorMessage::new|error_message',
    'error-message': r'field::ErrorMessage|ErrorMessage::new|error_message',
    'submenu-indicator': r'submenu|CHEVRON_RIGHT',
    'date-picker-trigger': r'trigger',
    'date-range-picker-trigger': r'trigger',
    'separator': r'Separator|separator',
    'checkbox': r'Checkbox',
    'radio': r'Radio',
    'input': r'Input::new|InputState',
    'textarea': r'TextArea|textarea',
    'spinner': r'Spinner|spinner',
    'spinner-icon': r'Spinner|spinner',
    'empty-state': r'No results found|No matching options|empty_state|empty_fg',
    'calendar-grid': r'calendar_view|month_grid|grid',
    'range-calendar-grid': r'calendar_view|month_grid|grid',
    'close-button-icon': r'CloseButton|icons::CLOSE',
    'link-icon': r'icons::|Link',
    'overlay-arrow': r'arrow',
    'popover-overlay-arrow': r'arrow',
}

# A claim this port answers differently, with the reason.
WONT_COMPOSE = {
    # v3's floating panels can grow a little triangle pointing at the trigger
    # (`<Popover.Overlay showArrow>`); v3 removed `showArrow` from the v3 prop
    # tables, `extra_audit.py` would report the builder, and every sheet styles
    # the arrow only as a fill colour and a rotation. Nothing here draws one.
    ('autocomplete', 'popover-overlay-arrow'): 'no-arrow-prop',
    ('combo-box', 'popover-overlay-arrow'): 'no-arrow-prop',
    ('select', 'popover-overlay-arrow'): 'no-arrow-prop',
    ('dropdown', 'popover-overlay-arrow'): 'no-arrow-prop',
    ('popover', 'popover-overlay-arrow'): 'no-arrow-prop',
    ('tooltip', 'overlay-arrow'): 'no-arrow-prop',
    # v3 lets a `<Label>` *wrap* its control (`label.css` styles a checkbox and
    # a radio inside one). This port's `field::Label` takes text, and a
    # `Checkbox` or `Radio` draws its own label beside the box -- which is the
    # other arrangement v3 supports and the one every v3 example uses.
    ('label', 'checkbox'): 'control-draws-its-label',
    ('label', 'radio'): 'control-draws-its-label',
}


def sheets():
    """`{sheet: css}` for every component stylesheet."""
    out = {}
    for name in sorted(os.listdir(CSS)):
        if not name.endswith('.css') or name in ('variables.css', 'utilities.css',
                                                 'shared_theme.css', 'index.css'):
            continue
        out[name[:-4]] = io.open(os.path.join(CSS, name), encoding='utf-8',
                                 errors='replace').read()
    return out


def contained(sheet, css):
    """The components `sheet` nests, as slot names.

    A slot whose name starts with the sheet's own component name is one of its
    parts (`.autocomplete`'s `autocomplete-default-indicator`), which
    `part_audit.py` reads; what matters here is a *foreign* component.
    """
    found = []
    # `:not(...)` names what a rule *excludes*: `.button` sizes every svg
    # `:not([data-slot="link-icon"] svg)`, which says a Button does not contain a
    # link icon -- the opposite of a containment claim.
    css = re.sub(r':not\([^()]*(?:\([^()]*\)[^()]*)*\)', '', css)
    for m in re.finditer(r'\[data-slot="([a-z0-9-]+)"\]', css):
        slot = m.group(1)
        # `--checkmark` and friends are a part's own modifier.
        if '--' in slot or slot.startswith(sheet):
            continue
        if slot not in found:
            found.append(slot)
    return found


def main():
    rows = []
    claims = met = excused = missing = unmapped = 0
    for sheet, css in sheets().items():
        for slot in contained(sheet, css):
            module = MODULE.get(sheet)
            if module is None:
                # A sheet with no module of its own (`menu.css` is the
                # dropdown's panel, `field-error.css` a shared part). Those are
                # named in state_audit's table when they draw a state; here a
                # missing entry means the sheet is not a component this port
                # renders on its own.
                continue
            claims += 1
            reason = WONT_COMPOSE.get((sheet, slot))
            if reason:
                excused += 1
                rows.append(('~', sheet, slot, reason))
                continue
            pattern = SLOT.get(slot)
            if pattern is None:
                unmapped += 1
                rows.append(('!', sheet, slot, 'no SLOT entry -- map it'))
                continue
            src = io.open(SRC + module, encoding='utf-8', errors='replace').read()
            if re.search(pattern, src):
                met += 1
                if '--all' in sys.argv:
                    rows.append((' ', sheet, slot, module))
                continue
            missing += 1
            rows.append(('?', sheet, slot, 'not composed in ' + module))

    for mark, sheet, slot, note in rows:
        print('%s %-22s %-28s %s' % (mark, sheet, slot, note))
    print()
    print('containment claims : %d' % claims)
    print('composed here      : %d' % met)
    print('recorded won-t     : %d' % excused)
    print('UNMAPPED           : %d' % unmapped)
    print('NOT COMPOSED       : %d' % missing)


if __name__ == '__main__':
    main()
