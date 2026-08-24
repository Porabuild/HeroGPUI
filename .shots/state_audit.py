"""Diff v3's *state* styling against the code that draws it.

Every v3 component stylesheet reaches for the same handful of state utilities:

    .button { &:focus-visible { @apply status-focused; } }
    .input  { &[data-focused="true"] { @apply status-focused-field; } }

which is a mechanical list of "this component styles that state", per component,
straight from the CSS. `design_audit.py` measures the *resting* look -- heights,
radii, fills -- and says nothing about these; that is how a port with no focus
ring anywhere passed every other audit.

Each `status-*` maps to the code that implements it. Two of them need more than
a symbol's presence:

- `status-focused-field` is drawn by `apply_field_chrome`, but only if the caller
  hands it a real focus flag. A literal `false` there is the bug this audit is
  for -- eight fields shipped that way -- so it is a forbidden pattern, not a
  missing one.
- `status-focused` is the ring, and a component draws it through
  `ring_if_focused` / `with_focus_ring`.

The stylesheets are not the only list. Each component page also states its states
in prose, under `### Interactive States`:

    * **Hover**: `:hover` or `[data-hovered="true"]`
    * **Active/Pressed**: `:active` or `[data-pressed="true"]` (includes scale)

which covers what the CSS utilities do not -- selected, open, dragging, today,
outside-month, frontmost -- so the second pass reads those 36 state names across
46 pages and asks the same question of each.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

CSS = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')
BUNDLE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt')
SRC = 'crates/herogpui-components/src/'

# Which module implements each stylesheet. A component split across files in v3
# (`menu-item.css` is the Dropdown's row, `search-field.css` is a field in
# `input.rs`) lands on the module that draws it.
MODULE = {
    'accordion': 'accordion.rs',
    'alert-dialog': 'alert_dialog.rs',
    'autocomplete': 'autocomplete.rs',
    'button': 'button.rs',
    # `calendar_view.rs` computes the grid; `calendar.rs` draws the cells.
    'calendar': 'calendar.rs',
    'calendar-year-picker': 'calendar.rs',
    'checkbox': 'checkbox.rs',
    'close-button': 'close_button.rs',
    'color-area': 'color_picker.rs',
    'color-input-group': 'color_picker.rs',
    'color-picker': 'color_picker.rs',
    'color-slider': 'color_picker.rs',
    'color-swatch-picker': 'color_picker.rs',
    'combo-box': 'combo_box.rs',
    'date-input-group': 'date_picker.rs',
    'date-picker': 'date_picker.rs',
    'date-range-picker': 'date_picker.rs',
    'disclosure': 'disclosure.rs',
    'drawer': 'drawer.rs',
    'dropdown': 'dropdown.rs',
    'input': 'input.rs',
    'input-group': 'input_group.rs',
    'input-otp': 'input_otp.rs',
    'label': 'field.rs',
    'link': 'link.rs',
    'list-box-item': 'list_box.rs',
    'menu-item': 'dropdown.rs',
    'meter': 'meter.rs',
    'modal': 'modal.rs',
    'number-field': 'number_field.rs',
    'pagination': 'pagination.rs',
    'popover': 'popover.rs',
    'progress-bar': 'progress.rs',
    'progress-circle': 'progress.rs',
    'radio': 'radio_group.rs',
    'range-calendar': 'range_calendar.rs',
    'search-field': 'input.rs',
    'select': 'select.rs',
    'slider': 'slider.rs',
    'switch': 'switch.rs',
    'table': 'table.rs',
    'tabs': 'tabs.rs',
    'tag': 'tag_group.rs',
    'textarea': 'textarea.rs',
    'toggle-button': 'toggle_button.rs',
    'tooltip': 'tooltip.rs',
}

# Where a state is drawn by something other than its usual symbol.
EVIDENCE_OVERRIDE = {
    # Five of v3's sheets map to `color_picker.rs`, so a module-wide search
    # cannot tell which of its controls rings. Each names its own handle.
    ('color-area', 'status-focused'): r'ring_if_focused\(area,',
    ('color-slider', 'status-focused'): r'ring_if_focused\(track,',
    ('color-picker', 'status-focused'): r'ring_if_focused\(\s*trigger,',
    ('color-swatch-picker', 'status-focused'): r'swatch_focus',
    # The OTP draws its own flush ring on the active slot, so there is no
    # `apply_field_chrome` to find.
    ('input-otp', 'status-focused-field'): r'with_focus_ring',
}

# The code that implements each state.
EVIDENCE = {
    'status-focused': r'ring_if_focused|with_focus_ring|focus_ring_shadows|util::focusable',
    'status-focused-field': r'apply_field_chrome|focus_ring_shadows\(false',
    'status-disabled': r'disabled_opacity',
    'status-pending': r'is_pending',
    'status-invalid-field': r'is_invalid|validity',
}

# Where a state is drawn by a *different* module than the sheet's own: an
# overlay's `status-disabled` belongs to the close button inside it, and a
# picker's popover rows are the list's.
ELSEWHERE = {
    ('modal', 'status-disabled'): 'close_button.rs',
    ('alert-dialog', 'status-disabled'): 'close_button.rs',
    ('drawer', 'status-disabled'): 'close_button.rs',
    ('popover', 'status-disabled'): 'close_button.rs',
    ('menu-item', 'status-disabled'): 'dropdown.rs',
    # A `TextArea` *is* an `Input` with a taller box, and a `Disclosure` is a
    # one-item `Accordion`: the state is drawn where the element is.
    ('textarea', 'status-disabled'): 'input.rs',
    ('disclosure', 'status-disabled'): 'accordion.rs',
    # `calendar_view.rs` is the grid; the component that takes `isDisabled` and
    # dims is `Calendar`.
    ('calendar', 'status-disabled'): 'calendar.rs',
    # What takes the focus inside a ComboBox is the `Input` it composes, and its
    # chrome is what rings. (An Autocomplete's trigger is its own element and
    # rings itself -- `.autocomplete__trigger:focus-visible` is `status-focused`,
    # the offset ring, not a field's flush one.)
    ('combo-box', 'status-focused-field'): 'input.rs',
    # A `TextArea` composes an `Input` with a taller box, so the ring is the
    # field's own.
    ('textarea', 'status-focused-field'): 'input.rs',
    # A `Disclosure` is a one-item `Accordion`, which is what draws its trigger.
    ('disclosure', 'status-focused'): 'accordion.rs',
}

# For a state whose evidence is a *call*, the argument matters as much as the
# call: `apply_field_chrome(.., is_invalid, false, cx)` draws no ring at all, and
# eight fields shipped that way. So the pattern demands at least one call in the
# module whose focus argument is something other than the literal `false` -- a
# module with both kinds (a field *and* a trigger that rings for itself) passes,
# and one with only `false` does not.
REQUIRED = {
    'status-focused-field':
        r'apply_field_chrome\((?:[^()]|\([^()]*\))*?,\s*(?!false\b)[a-z_@][^,]*,\s*cx',
}

# States this port does not draw, with the reason.
WONT_DO = {
    # v3 rings the whole dialog when the focus lands on the panel itself rather
    # than on something inside it. Here the panel takes the focus on open, so
    # ringing it would mean every modal opens with a ring around it.
    ('modal', 'status-focused'): 'panel-holds-focus',
    ('alert-dialog', 'status-focused'): 'panel-holds-focus',
    ('drawer', 'status-focused'): 'panel-holds-focus',
    ('popover', 'status-focused'): 'panel-holds-focus',
    # A tooltip is never focusable: it follows a trigger that is.
    ('tooltip', 'status-focused'): 'not-focusable',
    # `.label` dims with the field it belongs to, which is the field's own
    # `status-disabled` -- there is no separate label opacity.
    ('label', 'status-disabled'): 'field-dims-its-label',
    # v3 styles `[aria-disabled]` on these, but documents no prop that could set
    # it: there is nothing to disable a progress bar *with*. `extra_audit.py`
    # would flag the builder as undocumented if this port invented one.
    ('meter', 'status-disabled'): 'no-disabled-prop',
    ('progress-bar', 'status-disabled'): 'no-disabled-prop',
    ('progress-circle', 'status-disabled'): 'no-disabled-prop',
    ('table', 'status-disabled'): 'no-disabled-prop',
    # Same for `[data-pending]`: v3 styles it on the close button and the menu
    # but documents `isPending` on neither, so nothing can put them in it.
    ('close-button', 'status-pending'): 'no-pending-prop',
    ('dropdown', 'status-pending'): 'no-pending-prop',
}


# --- the prose pass -------------------------------------------------------
#
# One state name -> the code that draws it. A name is normalised to letters, so
# "Active/Pressed" and "Pressed" are one entry and "Focus visible" is
# `focusvisible`.
PROSE_EVIDENCE = {
    'disabled': r'is_disabled|disabled_opacity',
    'hover': r'\.hover\(',
    'hovered': r'\.hover\(',
    'focus': r'ring_if_focused|with_focus_ring|is_focused|track_focus',
    # A field's ring is drawn by its chrome, not by a ring call of its own.
    'focusvisible': r'ring_if_focused|with_focus_ring|focus_visible|apply_field_chrome',
    'focuswithin': r'ring_if_focused|with_focus_ring|focus_within|is_focused',
    'selected': r'is_selected|selected|is_start|in_range',
    'selectionstart': r'is_start',
    'selectionend': r'is_end',
    'rangemiddle': r'in_range',
    'invalid': r'is_invalid|validity',
    'pressed': r'anim::pressed|\.active\(',
    'activepressed': r'anim::pressed|\.active\(',
    'active': r'anim::pressed|\.active\(|is_active|active',
    'placement': r'placement',
    'open': r'is_open',
    'entering': r'anim::entering|entering_zoom|entering_from',
    'exiting': r'anim::exiting|overlay_phase',
    'dragging': r'dragging|on_mouse_move',
    'required': r'is_required',
    'today': r'is_today',
    'unavailable': r'unavailable',
    'outsidemonth': r'in_month|outside',
    'expanded': r'expanded',
    'expandedmanagement': r'expanded_keys',
    'pending': r'is_pending',
    'sortable': r'allows_sorting',
    'indeterminate': r'is_indeterminate',
    'readonly': r'is_read_only',
    'empty': r'is_empty|empty_state|\.is_empty\(\)',
    'current': r'is_current|current|is_last',
    'hidden': r'is_open|expanded',
    'activepage': r'active',
    'filled': r'filled|ch != ',
    'frontmost': r'depth',
    'index': r'depth',
}

# Prose states this port does not draw, with the reason.
PROSE_WONT_DO = {
    # React Aria's drag-and-drop reordering, which no list here implements.
    ('Table', 'droptarget'): 'no-drag-and-drop',
    ('Table', 'dragging'): 'no-drag-and-drop',
    # The docs claim these, and the component's own stylesheet has no rule for
    # them: `slider.css` styles no hover, `tag.css` no press. Following the CSS
    # rather than the prose is the same choice `design_audit.py` makes.
    ('Slider', 'hover'): 'not-in-the-stylesheet',
    ('ColorSlider', 'hover'): 'not-in-the-stylesheet',
    ('TagGroup', 'pressed'): 'not-in-the-stylesheet',
    # v3 rings the panel itself when the focus lands on it rather than on
    # something inside; here the panel takes the focus on open, so a ring would
    # be permanent.
    ('Popover', 'focus'): 'panel-holds-focus',
    # No prop can disable a table, so nothing can put it in that state.
    ('Table', 'disabled'): 'no-disabled-prop',
    # v3's pressed rule is on `.checkbox__control` and only for the
    # indeterminate box, whose fill it takes to `bg-accent-hover`. gpui needs a
    # stateful element for `.active`, and the box is a plain div drawn inside a
    # row that already carries the id -- a second id there would collide with it.
    ('Checkbox', 'pressed'): 'no-id-to-press',
}

# A state drawn by a component this one composes.
PROSE_ELSEWHERE = {
    # An overlay's hover and press states belong to the close button and the
    # buttons inside it. All three dialogs compose `CloseButton` now -- they used
    # to hand-roll a circle with its own `.hover`, which is not a shape v3 has.
    ('AlertDialog', 'hover'): 'close_button.rs',
    ('Modal', 'hover'): 'close_button.rs',
    ('Drawer', 'hover'): 'close_button.rs',
    ('AlertDialog', 'active'): 'button.rs',
    # The drawer's own sheet presses `.drawer__trigger` (built by the caller);
    # inside the component the only pressed thing is the composed close
    # button, which is where the evidence lives.
    ('Drawer', 'active'): 'close_button.rs',
    # These compose an `Input`, which is what hovers and takes the focus.
    ('InputGroup', 'hover'): 'input.rs',
    ('ComboBox', 'focus'): 'input.rs',
    # A `Disclosure` is a one-item `Accordion`.
    ('Disclosure', 'focus'): 'accordion.rs',
    # A `TextArea` is an `Input` with a taller box: the element the pointer is
    # over, and the one that hovers, is the field.
    ('TextArea', 'hover'): 'input.rs',
}

# The page's module, where the component's name does not give it away.
PROSE_MODULE = {
    'ColorArea': 'color_picker.rs',
    'ColorField': 'color_picker.rs',
    'ColorPicker': 'color_picker.rs',
    'ColorSlider': 'color_picker.rs',
    'ColorSwatchPicker': 'color_picker.rs',
    'DisclosureGroup': 'disclosure.rs',
    'SearchField': 'input.rs',
    'TextField': 'input.rs',
    'TextArea': 'textarea.rs',
    'InputOTP': 'input_otp.rs',
    'DatePicker': 'date_picker.rs',
    'DateRangePicker': 'date_picker.rs',
    'DateField': 'date_picker.rs',
    'RangeCalendar': 'range_calendar.rs',
    'AlertDialog': 'alert_dialog.rs',
    'ListBox': 'list_box.rs',
    'TagGroup': 'tag_group.rs',
    'NumberField': 'number_field.rs',
    'RadioGroup': 'radio_group.rs',
    'ToggleButton': 'toggle_button.rs',
    'InputGroup': 'input_group.rs',
    'ComboBox': 'combo_box.rs',
    'ProgressBar': 'progress.rs',
    'ProgressCircle': 'progress.rs',
}


def prose_states():
    """`{page: [normalised state, ...]}` from each page's Interactive States."""
    if not os.path.exists(BUNDLE):
        return {}
    text = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
    out, page = {}, None
    for m in re.finditer(r'^(#|###) (.+?)[ \t]*$', text, re.M):
        if m.group(1) == '#':
            page = m.group(2).strip()
        elif m.group(2).strip() == 'Interactive States' and page:
            chunk = text[m.end():]
            nxt = re.search(r'^#{1,4} ', chunk, re.M)
            chunk = chunk[:nxt.start()] if nxt else chunk
            names = [re.sub(r'[^a-z]', '', n.lower())
                     for n in re.findall(r'^\* \*\*(.+?)\*\*', chunk.strip(), re.M)]
            if names:
                out.setdefault(page, names)
    return out


def module_for(page):
    if page in PROSE_MODULE:
        return PROSE_MODULE[page]
    return re.sub(r'(?<!^)(?=[A-Z])', '_', page).lower() + '.rs'


def statuses(path):
    """The `status-*` utilities a stylesheet applies."""
    text = io.open(path, encoding='utf-8', errors='replace').read()
    return sorted(set(re.findall(r'status-[a-z-]+', text)))


def main():
    if not os.path.isdir(CSS):
        print('no CSS cache at %s -- run `python .shots/design_audit.py --fetch`' % CSS)
        return
    sources = {}
    claimed = drawn = excused = 0
    missing, unmapped = [], []
    by_reason = {}

    for name in sorted(os.listdir(CSS)):
        if not name.endswith('.css'):
            continue
        sheet = name[:-4]
        # `utilities.css` is where the states are *defined*.
        if sheet == 'utilities':
            continue
        states = statuses(os.path.join(CSS, name))
        if not states:
            # A sheet with no state utilities has nothing to ask about.
            continue
        if sheet not in MODULE:
            unmapped.append(sheet)
            continue
        for status in states:
            key = (sheet, status)
            module = ELSEWHERE.get(key, MODULE[sheet])
            if module not in sources:
                path = SRC + module
                sources[module] = (io.open(path, encoding='utf-8', errors='replace').read()
                                   if os.path.exists(path) else '')
            code = sources[module]
            claimed += 1
            if key in WONT_DO:
                excused += 1
                by_reason[WONT_DO[key]] = by_reason.get(WONT_DO[key], 0) + 1
                continue
            pattern = EVIDENCE_OVERRIDE.get(key) or EVIDENCE.get(status)
            if pattern is None:
                missing.append('%-22s %-22s (no EVIDENCE for this state)' % (sheet, status))
                continue
            required = None if key in EVIDENCE_OVERRIDE else REQUIRED.get(status)
            if required and not re.search(required, code, re.S):
                missing.append('%-22s %-22s %s: called, but never with a focus flag'
                               % (sheet, status, module))
                continue
            if re.search(pattern, code):
                drawn += 1
            else:
                missing.append('%-22s %-22s %s: /%s/' % (sheet, status, module, pattern))

    # --- second pass: the states the docs state in prose -------------------
    prose_claimed = prose_drawn = prose_excused = 0
    for page, states in sorted(prose_states().items()):
        module = module_for(page)
        path = SRC + module
        if not os.path.exists(path):
            missing.append('%-22s %-22s (no module named %s)' % (page, '-', module))
            continue
        if module not in sources:
            sources[module] = io.open(path, encoding='utf-8', errors='replace').read()
        for state in states:
            prose_claimed += 1
            elsewhere = PROSE_ELSEWHERE.get((page, state))
            if elsewhere:
                if elsewhere not in sources:
                    sources[elsewhere] = io.open(SRC + elsewhere, encoding='utf-8',
                                                 errors='replace').read()
                code = sources[elsewhere]
            else:
                code = sources[module]
            if (page, state) in PROSE_WONT_DO:
                prose_excused += 1
                reason = PROSE_WONT_DO[(page, state)]
                by_reason[reason] = by_reason.get(reason, 0) + 1
                continue
            pattern = PROSE_EVIDENCE.get(state)
            if pattern is None:
                missing.append('%-22s %-22s (no PROSE_EVIDENCE)' % (page, state))
                continue
            if re.search(pattern, code):
                prose_drawn += 1
            else:
                missing.append('%-22s %-22s %s: /%s/' % (page, state, module, pattern))

    for line in missing:
        print('MISSING  ' + line)
    for sheet in unmapped:
        print('UNMAPPED %s.css  (add it to MODULE)' % sheet)

    print()
    print('states claimed  : %d' % claimed)
    print('drawn           : %d' % drawn)
    print('not drawn here  : %d' % excused)
    for reason, n in sorted(by_reason.items(), key=lambda kv: (-kv[1], kv[0])):
        print('    %-22s %d' % (reason, n))
    print('prose states    : %d claimed, %d drawn, %d not drawn here'
          % (prose_claimed, prose_drawn, prose_excused))
    print('MISSING         : %d' % len(missing))
    print('UNMAPPED        : %d' % len(unmapped))


if __name__ == '__main__':
    main()
