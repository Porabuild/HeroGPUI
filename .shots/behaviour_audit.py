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
# group walks its tags and removes with Delete, and a swatch picker roves its
# focus with the arrows while Enter is the press that selects.
ARROW_NAV = ('RadioGroup', 'Tabs', 'Toolbar', 'TagGroup', 'ColorSwatchPicker')
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

# React Aria's ColorArea moves its thumb with the arrow keys -- left/right on
# the x channel, up/down on the y -- and Page Up/Down and Home/End move by the
# page step (React Aria's `useColorArea` keyboard shortcuts). v3's own page has
# no Accessibility section at all: it inherits from React Aria ColorArea, where
# the keys are documented. The claim is derived rather than read, like
# NumberField's, because a page with no prose is invisible to the CLAIMS
# patterns -- which is exactly why the missing handler went unnoticed.
AREA_KEYS = ('ColorArea',)

# React Aria shows a tooltip on keyboard focus as well as on hover; a hover-only
# tooltip is invisible to a keyboard user, and v3's own page says "shown on hover
# or focus".
FOCUS_OPEN = ('Tooltip',)

# Pinned React Stately owns one app-global Tooltip sequence: the first hover is
# delayed, a second tooltip opens immediately during cooldown, and it closes
# the previous tooltip rather than leaving two descriptions visible.
TOOLTIP_SEQUENCE = ('Tooltip',)

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

# A `Table::tree_row` inherits React Aria Table's row keyboard contract: Right
# expands a collapsed parent, Left collapses an expanded parent, and Left on a
# child returns the roving cursor to its parent. HeroUI documents the chevron
# composition but not these inherited keys, so this is derived from the pinned
# `react-aria` 3.51.0 `useTableRow` source. Virtual rows carry the same contract
# when `virtual_tree_metadata` supplies their cheap preorder tree projection.
TREE_KEYS = ('Table',)

# React Aria's Table wires `useTypeSelect` through `TableKeyboardDelegate`.
# HeroUI inherits that behavior without restating it in the Table page's short
# accessibility section, so keep the full-row text search as a derived claim.
TABLE_TYPEAHEAD = ('Table',)

# Pinned React Aria's Table keyboard delegate pages every body, not only a
# virtualized one. PageDown reaches the enabled body end; PageUp enters the
# first column header, which needs a unified header/body focus model.
TABLE_PAGING = ('Table',)

# Multiple-selection collections answer `Mod+A` -- the platform Mod, Ctrl on
# Windows and Linux, Cmd on macOS -- by selecting every enabled item. v3's own
# pages do not enumerate this inherited shortcut, so it is derived from the
# pinned React Aria 3.51.0 `useSelectableCollection` source (`Mod+A` ->
# `selectAll`, multiple-selection mode only).
SELECT_ALL_KEYS = ('Table', 'ListBox', 'TagGroup')

# A nonempty selectable collection clears on Escape by default and consumes the
# key only when it changed selection. HeroUI inherits this from pinned React
# Aria 3.51.0's `useSelectableCollection`.
ESCAPE_CLEAR_KEYS = ('ListBox', 'TagGroup')

# Pinned React Stately keeps custom input independent from a ComboBox's
# multiple selection. Enter on an unmatched value neither replaces the selected
# set nor reports either selection callback; Enter on a focused row toggles it
# through the plural callback, clears the query and leaves the list open. A row
# press carries the same multiple-mode contract.
COMBOBOX_MULTIPLE_KEYS = ('ComboBox',)

# HeroUI forwards Table.Column's resize props to React Aria, whose pinned
# TableColumnLayout clamps every committed width to minWidth (75px by default)
# and maxWidth. A draggable line without those bounds is not the same control.
RESIZE_BOUNDS = ('Table',)

# React Aria's inherited ColumnResizer range enters keyboard edit mode with
# Enter, moves by ten pixels per arrow, and exits with Enter, Escape, Space,
# Tab or a pointer press elsewhere. HeroUI's own prop table does not enumerate
# those inherited keys.
RESIZE_KEYS = ('Table',)

# `Table.LoadMore` is an end-of-collection sentinel. HeroUI documents the
# visibility trigger in the composed part's prop table and Async Loading
# example rather than under Accessibility, so this inherited behavior is a
# derived claim like Table's resize keys.
LOAD_MORE = ('Table',)

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

# React Aria pages the focused section according to the visible unit: months
# move by one month (a year with shift), weeks by one week (a month with shift),
# and days by pageBehavior with shift ignored. The visible window follows the
# focused section rather than leaving an invisible cursor behind.
CALENDAR_PAGING = ('Calendar', 'RangeCalendar')

# Home and End call React Stately's focusSectionStart/focusSectionEnd. Month
# views use the focused month, week views use the locale week (not the grid's
# firstDayOfWeek override), and day views use the current visible window.
CALENDAR_SECTION_BOUNDS = ('Calendar', 'RangeCalendar')

# An open floating panel holds the focus so the keyboard can reach it: a key
# event only travels to the focused element and its ancestors, so an open panel
# that focuses nothing is a panel no key can dismiss -- which is how a Popover
# opened through its controlled `isOpen` lost Escape (the key went to the app
# root). v3's floating pages say "Proper focus management" in as many words, and
# React Aria's overlays move the focus in on open: the dialog and the menu take
# it themselves, and the Autocomplete hands it to the `SearchField` its popover
# stacks above the list. The pickers move it to the calendar grid; without that
# the grid was a tab stop the user had to *find*, and the arrows did nothing
# until they did. Select and ComboBox are deliberately not here: their trigger
# and input hold the focus and own every key, so the panel never needs to claim
# it -- the Popover hole cannot open for a component whose trigger is a tab stop.
PANEL_FOCUS = (
    'Popover', 'Dropdown', 'Autocomplete', 'Modal', 'Drawer',
    'AlertDialog', 'DatePicker', 'DateRangePicker',
)

OVERLAY_DISMISS = (
    'Popover', 'Dropdown', 'Select', 'ComboBox', 'Autocomplete',
    'DatePicker', 'DateRangePicker', 'ColorPicker', 'Tooltip',
)

# Pinned React Aria `usePopover` closes these picker panels when focus leaves
# the trigger-plus-panel scope. Blur dismissal does not return focus to the
# trigger; the destination keeps it.
CLOSE_ON_BLUR = (
    'Select', 'Autocomplete', 'DatePicker', 'DateRangePicker', 'ColorPicker',
)

# A disabled native control is not successful: it contributes no FormData,
# does not satisfy `required`, and cannot block submission with stale validity.
# The text family shares InputState; NumberField reaches it through
# NumberState.input, while InputOTP carries the same live bit on OtpState.
# Read-only controls deliberately remain successful.
SUCCESSFUL_FORM_CONTROLS = (
    'Input', 'TextField', 'TextArea', 'SearchField', 'NumberField', 'InputOTP',
)
FORM_TEXT_SUCCESS = (
    r'(?s)pub fn text\(state: Entity<InputState>\)'
    r'(?:(?!pub fn number\().)*?successful_of: Some\(.*?is_successful'
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
    # Both colour controls live in one module, so ColorSlider's evidence
    # cannot tell the area apart from the slider -- the area's claim names the
    # wiring instead: the key context is what attaches the keyboard to it.
    ('ColorArea', 'area-keys'): ('color_picker.rs', r'key_context\("ColorArea"\)\s+\.on_key_down'),
    ('Modal', 'escape'): ('modal.rs', r'dismiss_on_escape_with_token'),
    ('Drawer', 'escape'): ('drawer.rs', r'dismiss_on_escape_with_token'),
    ('AlertDialog', 'escape'): ('alert_dialog.rs', r'dismiss_on_escape_with_token'),
    ('Drawer', 'drag-dismiss'): (
        'drawer.rs',
        r'window\.on_mouse_event\(move \|ev: &gpui::MouseMoveEvent',
    ),
    ('Modal', 'focus-trap'): ('modal.rs', r'trap_tab'),
    ('Drawer', 'focus-trap'): ('drawer.rs', r'trap_tab'),
    ('AlertDialog', 'focus-trap'): ('alert_dialog.rs', r'trap_tab'),
    # `Tab` cycles the dialog's own controls. gpui's tab order is the window's
    # and a tab group only *orders* its children, so the trap is done by moving
    # and checking -- see `util::trap_tab`.
    ('Modal', 'tab-cycle'): ('modal.rs', r'trap_tab'),
    ('Drawer', 'tab-cycle'): ('drawer.rs', r'trap_tab'),
    ('AlertDialog', 'tab-cycle'): ('alert_dialog.rs', r'trap_tab'),
    ('Breadcrumbs', 'arrows'): ('breadcrumbs.rs', r'on_click'),
    # `onPress` is a press, not a click: Enter and Space run the same handler.
    # gpui does that itself -- a *focused* element's click listeners fire with
    # `ClickEvent::Keyboard` -- so the evidence is the focus handle, which is
    # what the element was missing. Binding the handler again fires it twice.
    # The ARIA patterns these primitives name: a radio group, a tab list, a
    # toolbar, a tag group and a swatch picker are each *one* tab stop, with
    # the arrows moving inside (the picker's rove is focus-only; Enter or
    # Space is the press that selects). That is behaviour v3 claims by
    # inheriting them, and it is why `list_nav` is shared as widely as it is.
    ('RadioGroup', 'arrow-nav'): ('radio_group.rs', r'list_nav::resolve'),
    ('Tabs', 'arrow-nav'): ('tabs.rs', r'list_nav::resolve'),
    ('Toolbar', 'arrow-nav'): ('toolbar.rs', r'focus_next'),
    ('TagGroup', 'arrow-nav'): ('tag_group.rs', r'list_nav::resolve'),
    ('ColorSwatchPicker', 'arrow-nav'): ('color_picker.rs', r'list_nav::resolve'),
    ('TagGroup', 'remove-key'): (
        'tag_group.rs',
        r'(?s)"delete" \| "backspace".*?selected_now\.contains\(&key_for_remove\)'
        r'.*?cb\(&selected_now.*?HashSet::from\(\[key_for_remove\.clone\(\)\]\)',
    ),
    # Dismissal: the panel reads the press, and Escape reads wherever the focus
    # is -- on the panel when it holds it, on the component root otherwise (a
    # panel that claims the focus silences the calendar grid inside it).
    # The popover reads Escape on its *root* and the press on the panel: it
    # leaves the focus on whatever opened it, so there is nothing to hand back.
    ('Popover', 'dismiss'): ('popover.rs', r'dismiss_on_press_outside'),
    # Dropdown's submenu surface is a union of several measured panels, not
    # one rectangular `dismissable` hitbox. Prove both halves of dismissal:
    # outside presses test that union and Escape closes from the focused menu.
    ('Dropdown', 'dismiss'): (
        'dropdown.rs',
        r'(?s)(?=.*panel_union.*dismiss_on_press_outside_with_token_event)'
        r'(?=.*dismiss_on_escape_with_token)',
    ),
    ('Select', 'dismiss'): ('select.rs', r'dismiss_on_press_outside'),
    ('ComboBox', 'dismiss'): ('combo_box.rs', r'dismiss_on_press_outside'),
    ('Autocomplete', 'dismiss'): ('autocomplete.rs', r'dismiss_on_press_outside'),
    ('DatePicker', 'dismiss'): ('date_picker.rs', r'dismiss_on_press_outside'),
    ('DateRangePicker', 'dismiss'): ('date_picker.rs', r'dismiss_on_escape'),
    ('ColorPicker', 'dismiss'): ('color_picker.rs', r'dismiss_on_press_outside'),
    ('Tooltip', 'dismiss'): ('tooltip.rs', r'dismiss_on_escape'),
    ('Select', 'close-on-blur'): ('select.rs', r'util::close_on_blur'),
    ('Autocomplete', 'close-on-blur'): ('autocomplete.rs', r'util::close_on_blur'),
    ('DatePicker', 'close-on-blur'): (
        'date_picker.rs',
        r'(?s)close_on_blur\(.*?&format!\("dp-\{\}"',
    ),
    ('DateRangePicker', 'close-on-blur'): (
        'date_picker.rs',
        r'(?s)close_on_blur\(.*?&format!\("drp-\{\}"',
    ),
    ('ColorPicker', 'close-on-blur'): ('color_picker.rs', r'util::close_on_blur'),
    ('NumberField', 'spin-keys'): ('number_field.rs', r'"up" \| "pageup"'),
    ('Table', 'table-page-down'): ('table.rs', r'"pagedown" => stops\.last\(\)\.copied\(\)'),
    ('Tooltip', 'focus-open'): ('tooltip.rs', r'contains_focused'),
    ('Tooltip', 'global-sequence'): (
        'tooltip.rs',
        r'(?s)(?=.*struct TooltipManager)(?=.*prepare_tooltip_open)(?=.*start_tooltip_cooldown)',
    ),
    ('Input', 'disabled-form-omission'): (
        'form.rs',
        FORM_TEXT_SUCCESS,
    ),
    ('TextField', 'disabled-form-omission'): (
        'form.rs',
        FORM_TEXT_SUCCESS,
    ),
    ('TextArea', 'disabled-form-omission'): (
        'form.rs',
        FORM_TEXT_SUCCESS,
    ),
    ('SearchField', 'disabled-form-omission'): (
        'form.rs',
        FORM_TEXT_SUCCESS,
    ),
    ('NumberField', 'disabled-form-omission'): (
        'form.rs',
        r'(?s)pub fn number\((?:(?!pub fn code).)*?successful_of: Some\(.*?is_successful',
    ),
    ('InputOTP', 'disabled-form-omission'): (
        'form.rs',
        r'(?s)pub fn code\((?:(?!pub fn text_value\().)*?successful_of: Some\(.*?is_successful',
    ),
    ('Input', 'text-keys'): ('input.rs', r'fn word_target'),
    ('TextArea', 'text-keys'): ('input.rs', r'fn vertical_target'),
    ('TextField', 'text-keys'): ('input.rs', r'key_char'),
    ('Dropdown', 'focus-return'): ('dropdown.rs', r'back_to_trigger'),
    # The panel itself claims the focus, only when nothing inside already holds
    # it -- a click on the trigger leaves the ring where the user put it, while
    # a controlled open, which focuses nothing, still gets a panel the keyboard
    # can reach. This is the regression the claim exists for.
    ('Popover', 'panel-focus'): ('popover.rs', r'util::panel_focus\(window, cx, &base, claim\)'),
    # A menu takes the focus when it opens; the one-shot is spent on the same
    # handle the panel tracks, or the arrows land on a handle no element owns.
    ('Dropdown', 'panel-focus'): (
        'dropdown.rs',
        r'(?s:impl RenderOnce for Menu \{(?:(?!\nimpl ).)*?'
        r'util::focus_once\(window, cx, autofocus, &focus_handle\)'
        r'(?:(?!\nimpl ).)*?track_focus\(&focus_handle\))',
    ),
    # v3 writes `<SearchField autoFocus>` inside `Autocomplete.Filter`, so the
    # query field takes the focus as the popover opens -- once per opening, or
    # it would steal the focus back on every frame (a controlled caller typing
    # elsewhere would be robbed of the field).
    ('Autocomplete', 'panel-focus'): ('autocomplete.rs', r'window\.focus\(&search_focus\)'),
    # The dialogs claim the focus on open the same way the popover does: Escape
    # has to reach the overlay, and a key event only travels to the focused
    # element and its ancestors. The gate is what stops the claim from stealing
    # focus from a field inside the dialog. Shared by all three so they cannot
    # spell it differently.
    ('Modal', 'panel-focus'): (
        'modal.rs',
        r'if !focus_handle\.contains_focused\(window, cx\)\s*\{\s*window\.focus\(&focus_handle\);\s*\}',
    ),
    ('Drawer', 'panel-focus'): (
        'drawer.rs',
        r'if !focus_handle\.contains_focused\(window, cx\)\s*\{\s*window\.focus\(&focus_handle\);\s*\}',
    ),
    ('AlertDialog', 'panel-focus'): (
        'alert_dialog.rs',
        r'if !focus_handle\.contains_focused\(window, cx\)\s*\{\s*window\.focus\(&focus_handle\);\s*\}',
    ),
    # A picker moves the focus into the open calendar, so the grid answers the
    # arrows without the user having to find its tab stop first.
    ('DatePicker', 'panel-focus'): ('date_picker.rs', r'autofocus_grid\(panel_open\)'),
    ('DateRangePicker', 'panel-focus'): ('date_picker.rs', r'autofocus_grid\(panel_open\)'),
    ('Calendar', 'calendar-paging'): ('calendar.rs', r'calendar_view::focus_section'),
    ('RangeCalendar', 'calendar-paging'): ('range_calendar.rs', r'calendar_view::focus_section'),
    ('Calendar', 'calendar-section-bounds'): (
        'calendar.rs',
        r'(?s)(?=.*calendar_view::section_start)(?=.*calendar_view::section_end)',
    ),
    ('RangeCalendar', 'calendar-section-bounds'): (
        'range_calendar.rs',
        r'(?s)(?=.*calendar_view::section_start)(?=.*calendar_view::section_end)',
    ),
    ('Select', 'scroll-into-view'): ('select.rs', r'scroll_to_item'),
    ('ComboBox', 'scroll-into-view'): ('combo_box.rs', r'scroll_to_item'),
    ('Autocomplete', 'scroll-into-view'): ('autocomplete.rs', r'scroll_to_item'),
    ('ListBox', 'scroll-into-view'): ('list_box.rs', r'scroll_to_item'),
    ('Dropdown', 'scroll-into-view'): ('dropdown.rs', r'scroll_to_item'),
    ('Table', 'sort-keys'): ('table.rs', r'sort_focus'),
    ('Table', 'tree-keys'): (
        'table.rs',
        r'(?s)tree_rows\.get\(index\).*?key_name == "right"'
        r'.*?key_name == "left".*?Some\(parent\.clone\(\)\)',
    ),
    ('Table', 'table-typeahead'): ('table.rs', r'list_nav::typeahead\(\s*searchable_labels'),
    # `Mod+A` on a focused table: the platform-Mod check, the multiple-mode
    # gate, and the selectable-row set it reports.
    ('Table', 'select-all'): (
        'table.rs',
        r'(?s)key_name == "a".*?modifiers\.secondary\(\).*?modifiers\.platform'
        r'.*?SelectionMode::Multiple'
        r'.*?selectable_collection_keys\.clone\(\)',
    ),
    ('ListBox', 'select-all'): (
        'list_box.rs',
        r'(?s)key_name == "a".*?modifiers\.secondary\(\).*?modifiers\.platform'
        r'.*?mode == SelectionMode::Multiple.*?stops_for_keys.*?filter_map'
        r'.*?next\.iter\(\)\.all.*?selected_now\.contains.*?if !all_selected'
        r'.*?on_selection_change.*?stop_propagation\(\)',
    ),
    ('TagGroup', 'select-all'): (
        'tag_group.rs',
        r'(?s)key_name == "a".*?modifiers\.secondary\(\).*?modifiers\.platform'
        r'.*?mode == SelectionMode::Multiple.*?selectable_keys.*?if !all_selected'
        r'.*?selection_own_for_keys.*?on_selection_change.*?stop_propagation\(\)',
    ),
    ('ListBox', 'escape-clear'): (
        'list_box.rs',
        r'(?s)if !stops\.is_empty\(\) \|\| !self\.selected_keys\.is_empty\(\)'
        r'.*?key_name == "escape".*?!event\.keystroke\.modifiers\.modified\(\)'
        r'.*?reports_changes\(mode\)'
        r'.*?!selected_now\.is_empty\(\).*?let next = HashSet::new\(\)'
        r'.*?selection_own_for_keys.*?on_selection_change'
        r'.*?stop_propagation\(\)',
    ),
    ('TagGroup', 'escape-clear'): (
        'tag_group.rs',
        r'(?s)key_name == "escape".*?!event\.keystroke\.modifiers\.modified\(\)'
        r'.*?reports_changes\(mode\).*?!selected_now\.is_empty\(\).*?HashSet::new\(\)'
        r'.*?selection_own_for_keys.*?on_selection_change.*?stop_propagation\(\)',
    ),
    ('Table', 'resize-bounds'): (
        'table.rs',
        r'(?s)DEFAULT_COLUMN_MIN_WIDTH: f32 = 75\..*?floor\(\)\.min\(max\)\.max\(min\)',
    ),
    ('Table', 'resize-keys'): (
        'table.rs',
        r'(?s)keyboard-resizing.*?on_mouse_down_out.*?== Some\(column_index\)'
        r'.*?\*active = None.*?"enter" =>.*?\*active = if editing \{ None \}'
        r' else \{ Some\(column_index\) \}.*?stop_propagation\(\)'
        r'.*?"escape" \| "space" if editing =>.*?\*active = None'
        r'.*?stop_propagation\(\).*?"tab" if editing =>.*?\*active = None'
        r'.*?stop_propagation\(\).*?"right" \| "up" \| "left" \| "down" if editing'
        r'.*?matches!\(key, "right" \| "up"\).*?10\..*?-10\.'
        r'.*?floor\(\).*?\.min\(max_width\).*?\.max\(min_width\)'
        r'.*?stop_propagation\(\)',
    ),
    ('ComboBox', 'custom-value-multiple'): (
        'combo_box.rs',
        r'(?s)if allows_custom_value\s*&& key == "enter"'
        r'.{0,800}?is_none_or\(.{0,500}?cursor_position\(&rows, focused\).{0,200}?\{'
        r'(?:(?!on_selection_change_all).){0,3500}?if !multiple'
        r'.{0,800}?key_selection_own.{0,800}?open_own_keys.{0,400}?\*v = false'
        r'.{0,800}?if !multiple.{0,400}?on_selection_change'
        r'.{0,800}?if was_open.{0,400}?on_open_change',
    ),
    ('ComboBox', 'multiple-row-keys'): (
        'combo_box.rs',
        r'(?s)\A(?=.*Move::Activate => \{)(?=.*if multiple \{)'
        r'(?=.*set_value\(String::new\(\)\))'
        r'(?=.*held\.update)(?=.*hidden_query = Some\(String::new\(\)\))'
        r'(?=.*selected_now\.clone\(\))(?=.*next\.remove\(&item\))'
        r'(?=.*next\.insert\(item\.clone\(\)\))(?=.*key_selection_own)'
        r'(?=.*on_selection_change_all)(?=.*on_input_change)',
    ),
    ('ComboBox', 'multiple-row-pointer'): (
        'combo_box.rs',
        r'(?s)\A(?=.*Multiple mode toggles membership)(?=.*if multiple)'
        r'(?=.*selection_own\.clone\(\))(?=.*row_state\.clone\(\))'
        r'(?=.*cursor_for\(&rows, index, Some\(String::new\(\)\)\))'
        r'(?=.*on_click)(?=.*focus_handle\.focus\(window\))'
        r'(?=.*set_value\(String::new\(\)\))'
        r'(?=.*cursor\.update[^;]*Some\(next_cursor\.clone\(\)\))'
        r'(?=.*next\.remove\(&value\))(?=.*next\.insert\(value\.clone\(\)\))'
        r'(?=.*if let Some\(held\) = &own)(?=.*if let Some\(cb\) = &cb)'
        r'(?=.*if let Some\(cb\) = &input_change)',
    ),
    ('Table', 'load-more'): (
        'table.rs',
        r'(?s)virtual_end_is_near.*?last_item_size\.is_some_and'
        r'.*?logical_scroll_top\(\).*?bounds_for_item.*?scroll_offset > 0\.'
        r'.*?remaining <= margin',
    ),
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
    # Pinned TableKeyboardDelegate lets PageUp leave the body for the first
    # column header. This port models sortable headers and the body as separate
    # tab stops, so it falls back to the first enabled row until it has one
    # roving grid focus model across both regions.
    ('Table', 'table-page-up-header'): 'missing-header-body-focus-model',
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

    for page in SUCCESSFUL_FORM_CONTROLS:
        key = (page, 'disabled-form-omission')
        if key not in EVIDENCE and key not in WONT_DO:
            unmapped.append('%-14s %-14s' % key)
    for page in CLOSE_ON_BLUR:
        key = (page, 'close-on-blur')
        if key not in EVIDENCE and key not in WONT_DO:
            unmapped.append('%-14s %-14s' % key)

    # The derived claims first, so their numbers land in the same totals.
    # Deduplicated: a component appears in several of these tuples (a text area
    # has both the keys and the caret), and counting it once per tuple inflated
    # every total.
    derived = dict.fromkeys(
        ARROW_NAV + REMOVE_KEY + OVERLAY_DISMISS + CLOSE_ON_BLUR + SPIN_KEYS + AREA_KEYS
        + FOCUS_OPEN + TOOLTIP_SEQUENCE + TEXT_KEYS + POINTER_CARET + SORT_KEYS + TREE_KEYS
        + TABLE_TYPEAHEAD + TABLE_PAGING + SELECT_ALL_KEYS
        + ESCAPE_CLEAR_KEYS
        + COMBOBOX_MULTIPLE_KEYS
        + RESIZE_BOUNDS + RESIZE_KEYS
        + LOAD_MORE
        + FOCUS_RETURN + SCROLL_INTO_VIEW + CALENDAR_PAGING + CALENDAR_SECTION_BOUNDS
        + PANEL_FOCUS + SUCCESSFUL_FORM_CONTROLS
    )
    for page in derived:
        for claim in ('arrow-nav', 'remove-key', 'dismiss', 'spin-keys', 'area-keys',
                      'focus-open', 'global-sequence', 'text-keys', 'pointer-caret', 'sort-keys', 'tree-keys',
                      'table-typeahead', 'table-page-down', 'table-page-up-header',
                      'select-all', 'escape-clear', 'resize-bounds',
                      'resize-keys', 'focus-return', 'scroll-into-view', 'calendar-paging',
                      'calendar-section-bounds', 'panel-focus', 'load-more',
                      'disabled-form-omission', 'close-on-blur',
                      'custom-value-multiple', 'multiple-row-keys',
                      'multiple-row-pointer'):
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
    return 1 if missing or unmapped else 0


if __name__ == '__main__':
    sys.exit(main())
