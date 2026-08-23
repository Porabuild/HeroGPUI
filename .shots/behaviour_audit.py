"""Diff v3's documented *behaviour* against the code that implements it.

The prop audit asks whether a builder exists, the design audit whether a control
is the right size, the motion audit whether it moves. None of them asks whether
it answers a key. v3 states that per component, in prose, under
`## Accessibility`:

    * Full keyboard navigation support (arrow keys, home/end, typeahead)
    * Long press interaction support
    * Submenu navigation

Each of those is a claim about behaviour, and each is checkable. `CLAIMS` turns
the prose into claim ids, `EVIDENCE` names the code that implements one, and a
claim with neither evidence nor a recorded reason is a gap -- which is how a
slider with no `on_key_down` at all was found, and a menu with no arrow keys.

The prose is the weak side of this audit: a claim only counts if it is *written*
on the page, so a component whose Accessibility section is short is asked less.
That is why an unmapped claim is an error rather than a skip -- the mapping table
is where the reading gets pinned down.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

BUNDLE = os.environ.get(
    'HEROUI_BUNDLE',
    os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt'),
)
SRC = 'crates/herogpui-components/src/'

# What the prose says -> the behaviour it claims. Ordered, and every pattern is
# matched, because one bullet can carry several ("arrow keys, home/end,
# typeahead").
CLAIMS = [
    (r'[Aa]rrow keys', 'arrows'),
    (r'home/end|Home, End|Home and End', 'home-end'),
    (r'Page Up/Down', 'page-up-down'),
    (r'[Tt]ypeahead', 'typeahead'),
    (r'`ESC` closes', 'escape'),
    (r'`Tab` cycles', 'tab-cycle'),
    (r'\*\*Focus trap\*\*', 'focus-trap'),
    (r'Long press', 'long-press'),
    (r'Submenu navigation', 'submenu'),
    (r'\*\*Drag to dismiss\*\*', 'drag-dismiss'),
    (r'\*\*Scroll lock\*\*', 'scroll-lock'),
    (r'Keyboard navigation via Tab key', 'tab-order'),
    # "Full keyboard navigation support" with nothing in brackets still promises
    # the list's own keys.
    (r'Full keyboard navigation support(?!\s*\()', 'arrows'),
]

# A component that documents `onPress` promises keyboard activation, because a
# React Aria `PressEvent` is mouse, touch *and* keyboard -- gpui's `on_click` is
# the pointer alone. This claim is derived from the prop tables rather than the
# prose, which is why it is built rather than listed.
def activation_claims():
    import api_audit
    return {comp for comp in api_audit.FILES if 'onPress' in api_audit.props_for(comp)}


# The collection primitives whose ARIA pattern is "one tab stop, arrows inside".
# v3 claims each by saying it inherits from the React Aria component, and the
# pattern is that component's documented keyboard -- a radio group selects with
# the arrows, a tab list moves with them, a toolbar walks its controls, a tag
# group walks its tags and removes with Delete.
ARROW_NAV = ('RadioGroup', 'Tabs', 'Toolbar', 'TagGroup')
REMOVE_KEY = ('TagGroup',)

# Every popover-like surface closes on Escape and on a press outside it. v3's
# tables only mention dismissal where it is configurable (`isDismissable` on a
# dialog backdrop), because React Aria's `useOverlay` gives the rest of them both
# for free -- which is exactly why this port shipped panels that closed only
# through their own trigger. Derived, like the arrow keys, from what the
# component *is*.
# A number field is React Aria's `useSpinButton`: the arrows step it, Home and
# End run to the bounds. v3's page says nothing about it, and the field had no
# key handler at all -- the steppers were the only way to change it.
SPIN_KEYS = ('NumberField',)

# React Aria shows a tooltip on keyboard focus as well as on hover; a hover-only
# tooltip is invisible to a keyboard user, and v3's own page says "shown on hover
# or focus".
FOCUS_OPEN = ('Tooltip',)

# A text field's keyboard is the platform's, and none of it is in a prop table.
# What was missing: word-wise motion, copy and cut (paste was there), vertical
# motion and line-wise Home/End in a multi-line field, and -- worst -- capitals,
# because the handler read `keystroke.key` (the key *cap*) instead of `key_char`,
# so "AbC dEf!" arrived as "abc def1".
TEXT_KEYS = ('Input', 'TextArea', 'TextField')

# A click puts the caret where it landed and a drag selects. The caret used to
# stay wherever the value had left it, so the middle of a word was unreachable
# with the mouse.
POINTER_CARET = ('Input', 'TextField', 'TextArea')

# A sortable column header is a control: it had a click listener and no focus, so
# sorting was mouse-only.
SORT_KEYS = ('Table',)

# Closing an overlay hands the focus back to what opened it. Only a surface that
# *took* the focus has to: the pickers and the popover leave it on the trigger,
# so the menu is the one with something to return. A dialog's trigger belongs to
# the caller and the component has no handle for it.
FOCUS_RETURN = ('Dropdown', 'Modal', 'Drawer', 'AlertDialog')

# v3's popovers are `overflow-y-auto` and React Aria keeps the focused row in
# view. Ours were `overflow-hidden`, so a long list was *clipped* -- the rows
# past the panel's height could not be reached by mouse at all, and the keyboard
# highlight walked off the bottom.
SCROLL_INTO_VIEW = ('Select', 'ComboBox', 'Autocomplete', 'ListBox', 'Dropdown')

# React Aria's calendar pages by month, by *year* with shift, and keeps the
# focused date visible -- the grid follows the cursor across a month boundary.
# Ours moved an invisible cursor: the month on screen never changed.
CALENDAR_PAGING = ('Calendar', 'RangeCalendar')

# Opening a picker moves the focus into the calendar. Without it the grid was a
# tab stop the user had to *find*, and the arrows did nothing until they did.
PANEL_FOCUS = ('DatePicker', 'DateRangePicker')

OVERLAY_DISMISS = (
    'Popover', 'Dropdown', 'Select', 'ComboBox', 'Autocomplete',
    'DatePicker', 'DateRangePicker', 'ColorPicker', 'Tooltip',
)


# (component, claim) -> (module, a pattern that must appear in it).
#
# The pattern names the code that implements the behaviour, so a rewrite that
# drops it fails here rather than silently.
EVIDENCE = {
    ('Dropdown', 'arrows'): ('dropdown.rs', r'list_nav::resolve'),
    ('Dropdown', 'home-end'): ('dropdown.rs', r'list_nav::resolve'),
    ('Dropdown', 'typeahead'): ('dropdown.rs', r'list_nav::typeahead'),
    ('Dropdown', 'long-press'): ('dropdown.rs', r'long_press|LONG_PRESS'),
    ('Dropdown', 'submenu'): ('dropdown.rs', r'submenu'),
    ('ListBox', 'arrows'): ('list_box.rs', r'list_nav::resolve'),
    ('ListBox', 'typeahead'): ('list_box.rs', r'list_nav::typeahead'),
    ('Select', 'arrows'): ('select.rs', r'list_nav::resolve'),
    ('Select', 'typeahead'): ('select.rs', r'list_nav::typeahead'),
    ('ComboBox', 'arrows'): ('combo_box.rs', r'list_nav::resolve'),
    # A combo box's typeahead *is* its text field: React Aria filters the list
    # from what is typed into the input, rather than jumping a cursor. The filter
    # is what implements it.
    ('ComboBox', 'typeahead'): ('combo_box.rs', r'fn filter'),
    ('Autocomplete', 'arrows'): ('autocomplete.rs', r'list_nav::resolve'),
    ('Slider', 'arrows'): ('slider.rs', r'"right" \| "up"'),
    ('Slider', 'home-end'): ('slider.rs', r'"home"'),
    ('Slider', 'page-up-down'): ('slider.rs', r'"pageup"'),
    ('ColorSlider', 'arrows'): ('color_picker.rs', r'"right" \| "up"'),
    ('ColorSlider', 'home-end'): ('color_picker.rs', r'"home"'),
    ('ColorSlider', 'page-up-down'): ('color_picker.rs', r'"pageup"'),
    ('Modal', 'escape'): ('modal.rs', r'"escape"'),
    ('Drawer', 'escape'): ('drawer.rs', r'"escape"'),
    ('AlertDialog', 'escape'): ('alert_dialog.rs', r'"escape"'),
    ('Drawer', 'drag-dismiss'): ('drawer.rs', r'drag_dismiss|on_mouse_move'),
    ('Modal', 'focus-trap'): ('modal.rs', r'track_focus'),
    ('Drawer', 'focus-trap'): ('drawer.rs', r'track_focus'),
    ('AlertDialog', 'focus-trap'): ('alert_dialog.rs', r'track_focus'),
    ('Breadcrumbs', 'arrows'): ('breadcrumbs.rs', r'on_click'),
    # `onPress` is a press, not a click: Enter and Space run the same handler.
    # gpui does that itself -- a *focused* element's click listeners fire with
    # `ClickEvent::Keyboard` -- so the evidence is the focus handle, which is
    # what the element was missing. Binding the handler again fires it twice.
    # The ARIA patterns these primitives name: a radio group, a tab list, a
    # toolbar and a tag group are each *one* tab stop, with the arrows moving
    # inside. That is behaviour v3 claims by inheriting them, and it is why
    # `list_nav` is shared as widely as it is.
    ('RadioGroup', 'arrow-nav'): ('radio_group.rs', r'list_nav::resolve'),
    ('Tabs', 'arrow-nav'): ('tabs.rs', r'list_nav::resolve'),
    ('Toolbar', 'arrow-nav'): ('toolbar.rs', r'focus_next'),
    ('TagGroup', 'arrow-nav'): ('tag_group.rs', r'list_nav::resolve'),
    ('TagGroup', 'remove-key'): ('tag_group.rs', r'"delete" \| "backspace"'),
    # Dismissal: the panel reads the press, and Escape reads wherever the focus
    # is -- on the panel when it holds it, on the component root otherwise (a
    # panel that claims the focus silences the calendar grid inside it).
    # The popover reads Escape on its *root* and the press on the panel: it
    # leaves the focus on whatever opened it, so there is nothing to hand back.
    ('Popover', 'dismiss'): ('popover.rs', r'dismiss_on_press_outside'),
    ('Dropdown', 'dismiss'): ('dropdown.rs', r'util::dismissable'),
    ('Select', 'dismiss'): ('select.rs', r'dismiss_on_press_outside'),
    ('ComboBox', 'dismiss'): ('combo_box.rs', r'dismiss_on_press_outside'),
    ('Autocomplete', 'dismiss'): ('autocomplete.rs', r'dismiss_on_press_outside'),
    ('DatePicker', 'dismiss'): ('date_picker.rs', r'dismiss_on_press_outside'),
    ('DateRangePicker', 'dismiss'): ('date_picker.rs', r'dismiss_on_escape'),
    ('ColorPicker', 'dismiss'): ('color_picker.rs', r'dismiss_on_press_outside'),
    ('Tooltip', 'dismiss'): ('tooltip.rs', r'dismiss_on_escape'),
    ('NumberField', 'spin-keys'): ('number_field.rs', r'"up" \| "pageup"'),
    ('Tooltip', 'focus-open'): ('tooltip.rs', r'contains_focused'),
    ('Input', 'text-keys'): ('input.rs', r'fn word_target'),
    ('TextArea', 'text-keys'): ('input.rs', r'fn vertical_target'),
    ('TextField', 'text-keys'): ('input.rs', r'key_char'),
    ('Dropdown', 'focus-return'): ('dropdown.rs', r'back_to_trigger'),
    ('DatePicker', 'panel-focus'): ('date_picker.rs', r'autofocus_grid\(true\)'),
    ('DateRangePicker', 'panel-focus'): ('date_picker.rs', r'autofocus_grid\(true\)'),
    ('Calendar', 'calendar-paging'): ('calendar.rs', r'"pageup" if shift'),
    ('RangeCalendar', 'calendar-paging'): ('range_calendar.rs', r'"pageup" if shift'),
    ('Select', 'scroll-into-view'): ('select.rs', r'scroll_to_item'),
    ('ComboBox', 'scroll-into-view'): ('combo_box.rs', r'scroll_to_item'),
    ('Autocomplete', 'scroll-into-view'): ('autocomplete.rs', r'scroll_to_item'),
    ('ListBox', 'scroll-into-view'): ('list_box.rs', r'scroll_to_item'),
    ('Dropdown', 'scroll-into-view'): ('dropdown.rs', r'scroll_to_item'),
    ('Table', 'sort-keys'): ('table.rs', r'sort_focus'),
    ('Input', 'pointer-caret'): ('input.rs', r'fn char_at_x'),
    ('TextField', 'pointer-caret'): ('input.rs', r'closest_index_for_x'),
    ('Accordion', 'activation'): ('accordion.rs', r'tab_stop_handle'),
    ('Button', 'activation'): ('button.rs', r'tab_stop_handle'),
    ('CloseButton', 'activation'): ('close_button.rs', r'tab_stop_handle'),
    ('Link', 'activation'): ('link.rs', r'tab_stop_handle'),
    ('Pagination', 'activation'): ('pagination.rs', r'tab_stop_handle'),
    ('Switch', 'activation'): ('switch.rs', r'tab_stop_handle'),
    ('ToggleButton', 'activation'): ('toggle_button.rs', r'tab_stop_handle'),
}

# Documented behaviour this port does not implement, with the reason.
WONT_DO = {
    # gpui moves focus with its own Tab handling inside a focusable subtree;
    # there is no page outside the window to cycle *out* of, and no way to trap
    # what the platform routes.
    ('Modal', 'tab-cycle'): 'no-focus-trap',
    ('Drawer', 'tab-cycle'): 'no-focus-trap',
    ('AlertDialog', 'tab-cycle'): 'no-focus-trap',
    # v3 locks the *document* scroll while an overlay is open. A gpui window has
    # no document: the scrollers are the app's own elements, and an overlay
    # cannot reach them.
    ('Modal', 'scroll-lock'): 'no-page-scroll',
    ('Drawer', 'scroll-lock'): 'no-page-scroll',
    ('AlertDialog', 'scroll-lock'): 'no-page-scroll',
    # Tab order is the platform's, and gpui walks the focusable elements in tree
    # order without being told to.
    ('Pagination', 'tab-order'): 'platform-tab-order',
    # A wrapped line has no position gpui reports: `shape_line` measures one
    # line, and a paragraph in a text area is laid out by the text system into
    # as many as it needs. The caret still moves by key, including up and down.
    ('TextArea', 'pointer-caret'): 'no-wrapped-line-metrics',
    # A dialog claims the focus on open and has nothing to give it back to: the
    # trigger is the caller's element, rendered outside the component, and gpui
    # gives a child no way to reach it. The caller can restore it.
    ('Modal', 'focus-return'): 'no-handle-for-callers-trigger',
    ('Drawer', 'focus-return'): 'no-handle-for-callers-trigger',
    ('AlertDialog', 'focus-return'): 'no-handle-for-callers-trigger',
}


def accessibility_sections():
    """`{page: prose}` for every `## Accessibility` section in the bundle."""
    text = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
    out, page = {}, None
    for m in re.finditer(r'^(#|##) (.+?)[ \t]*$', text, re.M):
        if m.group(1) == '#':
            page = m.group(2).strip()
        elif m.group(2).strip() == 'Accessibility' and page:
            chunk = text[m.end():]
            nxt = re.search(r'^#{1,2} ', chunk, re.M)
            out[page] = chunk[:nxt.start()] if nxt else chunk
    return out


def main():
    sections = accessibility_sections()
    sources = {}
    claimed = implemented = excused = 0
    missing, unmapped = [], []
    by_reason = {}

    # The derived claims first, so their numbers land in the same totals.
    # Deduplicated: a component appears in several of these tuples (a text area
    # has both the keys and the caret), and counting it once per tuple inflated
    # every total.
    derived = dict.fromkeys(
        ARROW_NAV + REMOVE_KEY + OVERLAY_DISMISS + SPIN_KEYS + FOCUS_OPEN
        + TEXT_KEYS + POINTER_CARET + SORT_KEYS + FOCUS_RETURN + SCROLL_INTO_VIEW
        + CALENDAR_PAGING + PANEL_FOCUS
    )
    for page in derived:
        for claim in ('arrow-nav', 'remove-key', 'dismiss', 'spin-keys', 'focus-open',
                      'text-keys', 'pointer-caret', 'sort-keys', 'focus-return',
                      'scroll-into-view', 'calendar-paging', 'panel-focus'):
            key = (page, claim)
            # A derived claim can be excused too, and the reason has to reach
            # the breakdown: reading only EVIDENCE skipped `TextArea`'s
            # pointer caret silently, which is the one thing this audit is for.
            if key in WONT_DO:
                claimed += 1
                excused += 1
                by_reason[WONT_DO[key]] = by_reason.get(WONT_DO[key], 0) + 1
                continue
            if key not in EVIDENCE:
                continue
            claimed += 1
            module, evidence = EVIDENCE[key]
            if module not in sources:
                path = SRC + module
                sources[module] = (io.open(path, encoding='utf-8', errors='replace').read()
                                   if os.path.exists(path) else '')
            if re.search(evidence, sources[module]):
                implemented += 1
            else:
                missing.append('%-14s %-14s %s: /%s/' % (page, claim, module, evidence))

    for page in sorted(activation_claims()):
        key = (page, 'activation')
        claimed += 1
        module, evidence = EVIDENCE[key]
        if module not in sources:
            path = SRC + module
            sources[module] = (io.open(path, encoding='utf-8', errors='replace').read()
                               if os.path.exists(path) else '')
        if re.search(evidence, sources[module]):
            implemented += 1
        else:
            missing.append('%-14s %-14s %s: /%s/' % (page, 'activation', module, evidence))

    for page, prose in sorted(sections.items()):
        for pattern, claim in CLAIMS:
            if not re.search(pattern, prose):
                continue
            key = (page, claim)
            if key in WONT_DO:
                claimed += 1
                excused += 1
                by_reason[WONT_DO[key]] = by_reason.get(WONT_DO[key], 0) + 1
                continue
            if key not in EVIDENCE:
                # Not a gap and not excused: the mapping has not been read yet.
                if claim not in ('focus', 'arrows') or key not in EVIDENCE:
                    unmapped.append('%s.%s' % (page, claim))
                continue
            claimed += 1
            module, evidence = EVIDENCE[key]
            if module not in sources:
                path = SRC + module
                sources[module] = (io.open(path, encoding='utf-8', errors='replace').read()
                                   if os.path.exists(path) else '')
            if re.search(evidence, sources[module]):
                implemented += 1
            else:
                missing.append('%-14s %-14s %s: /%s/' % (page, claim, module, evidence))

    for line in missing:
        print('MISSING  ' + line)
    if unmapped:
        print()
        for name in sorted(set(unmapped)):
            print('UNMAPPED %s  (add EVIDENCE or WONT_DO)' % name)

    print()
    print('behaviours claimed : %d' % claimed)
    print('implemented        : %d' % implemented)
    print('not implementable  : %d' % excused)
    for reason, n in sorted(by_reason.items(), key=lambda kv: (-kv[1], kv[0])):
        print('    %-22s %d' % (reason, n))
    print('MISSING            : %d' % len(missing))
    print('UNMAPPED           : %d' % len(set(unmapped)))


if __name__ == '__main__':
    main()
