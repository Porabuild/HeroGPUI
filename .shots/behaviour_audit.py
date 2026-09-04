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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bundle import resolve as _resolve_bundle

# The pinned v3.2.4 bundle. See .shots/bundle.py: reading upstream live would
# measure this port against whatever HeroUI shipped most recently.
BUNDLE = _resolve_bundle()
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
    # Breadcrumbs is the one page whose keyboard prose is exactly this sentence,
    # and its keyboard is the link's: Tab reaches every link crumb, Enter or
    # Space activates the focused one, and the current page is not a stop.
    (r'Keyboard navigation support', 'tab-nav'),
    # Breadcrumbs: "Current page indication via aria-current" — the last crumb
    # is the current page, rendered in the link token and inert.
    (r'Current page indication', 'current-page'),
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

# Pinned React Aria `useToolbar` (react-aria 3.51.0) scopes a FocusManager to
# the toolbar's element with `wrap` unset, so an along-axis arrow at either end
# is consumed (`stopPropagation` + `preventDefault` run either way) without
# moving. Tab runs `focusFirst` (Shift: `focusLast`) and lets the native Tab
# carry out of the entire toolbar in one press, and the `lastFocused` ref
# records the child the focus left from and restores it when the focus
# re-enters from outside. HeroUI's page says only "Inherits from React Aria
# Toolbar", so all three are derived claims -- and so is the nested-toolbar
# group contract: pinned detects a toolbar inside another with
# `parentElement.closest('[role="toolbar"]')` and the nested one binds no
# keyboard or focus management of its own, so the outer manager walks across
# its children.
TOOLBAR_FOCUS = ('Toolbar',)

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

# Pinned `ListKeyboardDelegate` moves PageUp/PageDown by one visible rectangle,
# skips disabled/non-option rows, and uses the enabled ends when the list does
# not scroll. Fixed, estimated and plain layouts each own their geometry.
LISTBOX_PAGING = ('ListBox',)

# Select's popover list is the same ListBox behind a trigger, but pinned
# HeroUI v3.2.4 puts the overflow scrolling on the Popover while the ListBox
# element is `overflow-clip`, so pinned React Aria 3.51.0 treats Select's list
# as non-scrollable: with a cursor, page keys take the enabled ends -- the
# port's `stops` already omits disabled rows -- whatever the list's length,
# row height, or scroll state. No viewport step or rectangle geometry may
# survive: the evidence rejects it outright. Those handlers also require
# `manager.focusedKey != null`, so a mouse-opened, selection-less Select --
# cursor null -- must answer nothing, and the evidence demands the cursor
# gate: an unconditional end mapping fails the claim. The closed trigger
# answers no page key at all: pinned `useSelectableCollection` binds them
# only while the list is open.
SELECT_PAGING = ('Select',)

# The other popup collections share Select's composition: pinned HeroUI
# v3.2.4 mounts the ListBox/Menu element with `overflow-clip` inside a popup
# that owns the scrolling, so pinned React Aria 3.51.0's ListKeyboardDelegate
# treats each as non-scrollable and -- only when `manager.focusedKey != null`
# -- takes page keys to the enabled ends; the port's `stops` already omit
# disabled rows. No viewport step or rectangle geometry may survive: the
# evidence rejects it outright. The closed surface answers no page key at all
# (`useSelectableCollection` binds them only while the collection is open):
# the Dropdown's panel handler exists only while open, but the ComboBox drives
# open and navigation from one root handler, so its mapping must also demand
# the open state or a closed field with a retained cursor would move -- and
# its `Move::To` path would even reopen the list.
POPUP_PAGING = ('ComboBox', 'Dropdown')

# Autocomplete is the exception: `autocomplete.css` styles the composed
# `[data-slot="list-box"]` itself `max-h-[320px] min-h-0 overflow-y-auto` --
# the list element *is* the scroller -- so pinned React Aria 3.51.0's
# `ListKeyboardDelegate` sees a scrollable list and pages by one visible
# rectangle of it: from the cursor row's laid-out rect, walk enabled rows
# until one crosses a viewport-sized boundary (320px minus one row), and
# take the enabled end only when the walk runs out. The handlers still
# require `manager.focusedKey != null`, so a mouse-opened, selection-less
# Autocomplete answers nothing until an arrow seats a cursor. The default
# rows are laid out, so the boundary reads real scroll-handle rects (the
# plain ListBox shape); a `rowHeight` list is uniform and pages by
# whole-row steps across its fixed 320px viewport (the fixed ListBox shape).
# No cursor-gated enabled-end mapping may survive -- the evidence rejects it
# outright -- and neither may a mapping with no geometry behind it.
AUTOCOMPLETE_PAGING = ('Autocomplete',)

# Multiple-selection collections answer `Mod+A` -- the platform Mod, Ctrl on
# Windows and Linux, Cmd on macOS -- by selecting every enabled item. v3's own
# pages do not enumerate this inherited shortcut, so it is derived from the
# pinned React Aria 3.51.0 `useSelectableCollection` source (`Mod+A` ->
# `selectAll`, multiple-selection mode only). Select is the one member whose
# report differs: pinned SelectState drops the symbolic `all`, so its plural
# callback stays silent while the uncontrolled set still becomes every enabled
# key, and a controlled owner's state is not the keystroke's to mutate.
SELECT_ALL_KEYS = ('Table', 'ListBox', 'TagGroup', 'Select')

# A nonempty selectable collection clears on Escape by default and consumes the
# key only when it changed selection. HeroUI inherits this from pinned React
# Aria 3.51.0's `useSelectableCollection`. Select is not a member: pinned
# `useSelect` returns `menuProps` with `disallowEmptySelection: true`, so the
# generic clear is unreachable there -- Escape closes the panel on the first
# press and the selection survives it.
ESCAPE_CLEAR_KEYS = ('ListBox', 'TagGroup')

# Pinned React Stately 3.49.0's `useMultipleSelectionState` extends a multiple
# selection from its anchor on Shift+Arrow and Shift+Click -- React Aria
# 3.51.0's `useSelectableCollection` routes both through `extendSelection`.
# The old anchor..current range is replaced by anchor..target, so a reverse
# move shrinks again; only enabled keys join the range; a raw `all` selection
# collapses to the moved-to key; and a first extension without an anchor
# selects the target alone. Home and End are registered per platform:
# macOS installs them only for none, Shift, Alt, and Alt+Shift, so Shift and
# Alt+Shift move the focus alone and every Cmd- or Ctrl-bearing chord is
# entirely inert in either mode; Windows and Linux install none, Shift,
# Control, and Control+Shift, and only Control+Shift extends, and the same
# veto governs a single-mode Select's Home and End. From a null cursor a
# registered Shift+Home/End is wholly inert before cursor seating. Pointer
# press seats the cursor (`useSelectableItem`), so a Shift+Arrow, page, or
# Enter that follows a click starts from the clicked row. The anchor is state
# the collection owns beside its cursor, so it survives closing and reopening
# a popover. HeroUI's Table, TagGroup, ListBox, and Select pages do not
# restate this inherited contract, so it is derived like the other collection
# range work.
RANGE_SELECT = ('TagGroup', 'ListBox', 'Table', 'Select')

# A multiple Select answers no typeahead on its closed trigger: the closed
# pick would report through the single-key callback a set-valued selection
# has no use for. The open RAC ListBox still runs its type-select and moves
# the cursor without selecting, exactly as in single mode. Derived from the
# pinned React Aria Components 1.20.0 Select, whose multiple state gates the
# trigger's letter picker only.
MULTIPLE_TRIGGER_TYPEAHEAD = ('Select',)

# Pinned React Aria 3.51.0's `useSelectableCollection` seats a collection on
# pointer-down: a press on an enabled item moves the roving cursor to it and
# takes the group's focus, so the arrows and Space answer it with no Tab
# first. A press on an interactive child (TagGroup's remove button) belongs
# to the child -- pinned `useSelectableItem` only isolates the child's press
# and hands DOM focus to the button. This port seats the owning tag itself
# after the removal report because the report-only Rust model has no
# persisting native child and keyboard continuity needs a stable roving
# target. Pointer focus is not focus-visible (the modality the app root
# records), and a disabled item takes no press at all. HeroUI inherits this
# without restating it, so it is a derived claim like the range work above.
POINTER_FOCUS = ('TagGroup',)

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

RESIZE_KEYS_EVIDENCE = '<structured resize-key handler>'

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

# ComboBox uses a stronger pinned state contract: leaving the whole field
# commits or reverts its query according to selection mode/custom-value policy,
# then closes without reclaiming focus.
COMBOBOX_BLUR_COMMIT = ('ComboBox',)

# A disabled native control is not successful: it contributes no FormData,
# does not satisfy `required`, and cannot block submission with stale validity.
# The text family shares InputState; NumberField reaches it through
# NumberState.input, while InputOTP carries the same live bit on OtpState.
# Read-only controls deliberately remain successful.
SUCCESSFUL_FORM_CONTROLS = (
    'Input', 'TextField', 'TextArea', 'SearchField', 'NumberField', 'InputOTP',
)

# Pinned React Stately 3.49.0 gives DisclosureGroup a controlled/uncontrolled
# expanded set, single-expand default, multiple opt-in and whole-group disabled
# state. HeroUI forwards that primitive unchanged, but its page has no separate
# Accessibility prose for these state transitions, so keep them as derived
# behavior rather than letting the prop-name audit stand in for implementation.
DISCLOSURE_GROUP_STATE = ('DisclosureGroup',)
DISCLOSURE_STATE = ('Disclosure',)
FORM_TEXT_SUCCESS = (
    r'(?s)pub fn text\(state: Entity<InputState>\)'
    r'(?:(?!pub fn number\().)*?successful_of: Some\(.*?is_successful'
)

# v3's Form "renders a native `<form>` element" and inherits the browser's
# submission behaviour from React Aria's Form primitive, none of which appears
# in a prop table: implicit submission from a participating single-line field,
# onInvalid focusing the first invalid field, disabled controls omitted,
# read-only controls barred from constraint validation while still
# successful. The port's own additions — the suppression a field with its own
# Enter needs, the OTP row's participation — hang off the same derived claim.
# So does the `validationErrors` record contract, which the prop table states
# in one line ("mapped by field name. Displayed immediately and cleared when
# user modifies the field") and whose load-bearing mechanics are four: the
# record routes into named fields' own error slots, an edit suppresses only
# the edited field's messages, reset hides them, and delivery is receipted
# per field — the record revision stored beside the messages — so a new
# record re-arms while a clone does not.
FORM_SUBMISSION = ('Form',)
# The record type itself, with its revision identity: `new` mints, `Clone`
# derives (so a clone shares the revision), and `PartialEq` compares content
# only because structural equality must not decide delivery.
RECORD_IDENTITY = (
    r'(?s)static NEXT_RECORD_REVISION: AtomicU64 = AtomicU64::new\(1\);'
    r'.*?#\[derive\(Clone, Debug\)\]\s*pub struct ValidationErrors \{'
    r'.*?pub fn new\(\) -> Self \{\s*Self \{\s*revision: next_record_revision\(\),'
    r'.*?fn eq\(&self, other: &Self\) -> bool \{\s*self\.entries == other\.entries'
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
    # The Breadcrumbs keyboard is the link's, not a collection's: every link
    # crumb is its own tab stop (`tab_stop_handle`, gated on the crumb being
    # neither the current page nor disabled), Enter and Space activate the
    # focused crumb through gpui's focused-click, and no second key handler
    # is bound for it. The old 'arrows' row was bogus: Breadcrumbs promises
    # no arrow keys, and on_click alone proves neither a tab stop nor the
    # gate that keeps the current page out of the tab order.
    ('Breadcrumbs', 'tab-nav'): (
        'breadcrumbs.rs',
        r'(?s)(?=\(!is_last && !disabled\)\.then\(\|\| \{.{0,120}?crate::util::tab_stop_handle)'
        r'(?=.*\.on_click\()',
    ),
    # The current page is the last crumb: it renders in the link token, is
    # never navigable (no tab stop, no press, even with an href), and the
    # disabled fade never reaches it.
    ('Breadcrumbs', 'current-page'): (
        'breadcrumbs.rs',
        r'(?s)(?=.*text_color\(if is_last \{ current_color \} else \{ muted \}\))'
        r'(?=.*let is_link = !is_last && !disabled;)'
        r'(?=.*let navigable = is_link && \(on_navigate\.is_some\(\) \|\| crumb\.href\.is_some\(\)\);)',
    ),
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
    # The window's first and last tab stops, probed from no focus: gpui's
    # `focus_next`/`focus_prev` wrap around at the window's ends, and the
    # pinned FocusManager's walk has `wrap` unset — an end is a dead stop, so
    # the ends must be known before stepping.
    ('Toolbar', 'toolbar-end-stops'): (
        'toolbar.rs',
        r'(?s)window\.focus_next\(\);\s*\n\s*let first_stop = window\.focused\(cx\);'
        r'.*?window\.focus_prev\(\);\s*\n\s*let last_stop = window\.focused\(cx\);'
        r'.*?at_end',
    ),
    # Tab leaves the entire toolbar in one press, backwards with Shift: the
    # bounded walk steps until the focus leaves the subtree and refuses the
    # window-end wrap (a native Tab from the document's last focusable goes
    # nowhere either).
    ('Toolbar', 'toolbar-tab-exit'): (
        'toolbar.rs',
        r'(?s)let back = event\.keystroke\.modifiers\.shift;.*?for _ in 0\.\.256 \{',
    ),
    # The keyed `lastFocused` record: the exit frame stores the child the
    # focus left from; the entry frame hands the focus to it and clears the
    # record.
    ('Toolbar', 'toolbar-focus-restore'): (
        'toolbar.rs',
        r'(?s)(?=.*if let Some\(last\) = next\.last_focused\.take\(\))'
        r'(?=.*window\.focus\(&last\);)'
        r'(?=.*next\.last_focused = next\.child\.take\(\))',
    ),
    # A nested toolbar binds no management of its own: pinned's
    # `parentElement.closest('[role="toolbar"]')` is answered here by the
    # registry sync against the last rendered dispatch tree
    # (`FocusHandle::contains`), gated over the focus bookkeeping and
    # re-checked at event time so the handler returns without consuming.
    ('Toolbar', 'toolbar-nested'): (
        'toolbar.rs',
        r'(?s)(?=.*fn sync_toolbar_scope)'
        r'(?=.*other\.contains\(scope, window\))'
        r'(?=.*if !nested \{)'
        r'(?=.*if sync_toolbar_scope\(&scope, window, cx\) \{)',
    ),
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
    ('ComboBox', 'blur-commit'): (
        'combo_box.rs',
        r'(?s)\A(?=.*util::on_focus_leave\()(?=.*if allows_custom)'
        r'(?=.*!multiple.*?value\.clear\(\))'
        r'(?=.*let committed = if multiple.*?String::new\(\))'
        r'(?=.*blur_commit\(window, cx\).*?blur_close\(window, cx\))',
    ),
    ('NumberField', 'spin-keys'): ('number_field.rs', r'"up" \| "pageup"'),
    ('Table', 'table-page-down'): ('table.rs', r'"pagedown" => stops\.last\(\)\.copied\(\)'),
    # PageUp leaves the body for the header rather than stopping at the first
    # row, which needs the header focusable whether or not it sorts.
    ('Table', 'table-page-up-header'): (
        'table.rs',
        r'(?s)(?=.*if plain_rows && key_name == "pageup")'
        r'(?=.*window\.focus\(header\))'
        r'(?=.*let header_focus: Vec<gpui::FocusHandle>)',
    ),
    ('ListBox', 'listbox-paging'): (
        'list_box.rs',
        r'(?s)\A(?=.*fixed_page_step)(?=.*variable_page_move)(?=.*plain_page_move)'
        r'(?=.*"pagedown")(?=.*"pageup")',
    ),
    # The cursor-gated enabled-end mapping is required, and the discarded
    # viewport geometry must be gone: the negative lookaheads fail the file if
    # the step / rectangle paging (`fixed_page_step`, the `*_page_move`
    # machinery, or the panel's `bounds_for_item` / `max_offset` reads) ever
    # returns, and the required lookaheads demand the `from.is_some()` gate so
    # an unconditional end mapping -- one that would move a mouse-opened,
    # cursor-less list, contrary to pinned React Aria's
    # `manager.focusedKey != null` requirement -- cannot satisfy the claim.
    ('Select', 'select-paging'): (
        'select.rs',
        r'(?s)\A(?!.*fixed_page_step)(?!.*variable_page_move)'
        r'(?!.*plain_page_move)(?!.*bounds_for_item)(?!.*max_offset)'
        r'(?=.*"pagedown" if from\.is_some\(\) => stops\.last\(\))'
        r'(?=.*"pageup" if from\.is_some\(\) => stops\.first\(\))',
    ),
    # The same load-bearing shape as Select's evidence, per popup owner: the
    # negative lookaheads fail the file if viewport-step paging machinery ever
    # returns, and the required lookaheads demand the cursor gate (plus, for
    # the ComboBox's single root handler, the open gate) so an unconditional
    # end mapping -- which would move a mouse-opened, cursor-less list, or
    # move and even reopen a closed ComboBox with a retained cursor --
    # cannot satisfy the claim.
    ('ComboBox', 'popup-paging'): (
        'combo_box.rs',
        r'(?s)\A(?!.*fixed_page_step)(?!.*variable_page_move)'
        r'(?!.*plain_page_move)(?!.*bounds_for_item)(?!.*max_offset)'
        r'(?=.*"pagedown" if is_open && from\.is_some\(\) => stops\.last\(\))'
        r'(?=.*"pageup" if is_open && from\.is_some\(\) => stops\.first\(\))',
    ),
    # The load-bearing shape in the opposite direction from Select/ComboBox/
    # Dropdown's: Autocomplete's list element is itself the scroller, so the
    # required lookaheads demand the cursor gate (`page_move = from.and_then`)
    # and the real geometry -- the fixed whole-row step for a `rowHeight`
    # list and the laid-out `bounds_for_item` rect walk for the default rows
    # -- while the negative lookaheads fail the file if the cursor-gated
    # enabled-end mapping ever returns.
    ('Autocomplete', 'autocomplete-paging'): (
        'autocomplete.rs',
        r'(?s)\A(?!.*"pagedown" if from\.is_some\(\) => stops\.last\(\))'
        r'(?!.*"pageup" if from\.is_some\(\) => stops\.first\(\))'
        r'(?=.*page_move = from\.and_then)(?=.*fixed_page_step)'
        r'(?=.*bounds_for_item)',
    ),
    ('Dropdown', 'popup-paging'): (
        'dropdown.rs',
        r'(?s)\A(?!.*fixed_page_step)(?!.*variable_page_move)'
        r'(?!.*plain_page_move)(?!.*bounds_for_item)(?!.*max_offset)'
        r'(?=.*"pagedown" if from\.is_some\(\) => stops_for_keys\.last\(\))'
        r'(?=.*"pageup" if from\.is_some\(\) => stops_for_keys\.first\(\))',
    ),
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
    # The whole native submission contract on one shared implementation: the
    # button door (`submit_handler`) and the Enter door (the form root's key
    # handler) both route through `run_submission`, and the Enter door fires
    # only while a field that carries an Enter reader holds the focus.
    ('Form', 'shared-funnel'): (
        'form.rs',
        r'(?s)(?=.*pub fn submit_handler)(?=.*down_fields\.iter\(\)\.any\(\|f\| f\.submits_on_enter\(window, cx\)\))'
        r'(?=.*Self::run_submission\(\s*&down_fields)(?=.*Self::run_submission\(\s*&fields)',
    ),
    # Implicit submission: the reader half (which fields participate — text,
    # number, and the OTP row's single input) beside the gate that reads it.
    ('Form', 'implicit-enter'): (
        'form.rs',
        r'(?s)(?=.*type SubmitsOnEnter = Arc<dyn Fn\(&Window, &App\) -> bool)'
        r'(?=.*submits_on_enter_of: Some\(Arc::new\(move \|window, cx\| \{\s*enter_state\s*\n?\s*\.read\(cx\))'
        r'(?=.*fn submits_on_enter\(&self, window: &Window, cx: &App\))',
    ),
    # onInvalid's default focus, deferred past an Enter-origin keystroke: the
    # first-invalid computation, the release-side latch consumed in a capture
    # handler, and the disarm-plus-deferred move that keeps the release from
    # clicking the control the focus lands on.
    ('Form', 'blocked-focus'): (
        'form.rs',
        r'(?s)(?=.*fn first_invalid_focus)(?=.*capture_key_up)'
        r'(?=.*window\.prevent_default\(\);)(?=.*window\.defer\(cx, move \|window, cx\| focus\(window, cx\)\))',
    ),
    # A TextArea's Enter is a newline: the text-area registration carries no
    # Enter reader, so the form's gate never fires from it.
    ('Form', 'textarea-suppression'): (
        'form.rs',
        r'(?s)pub fn text_area\(state: Entity<InputState>\)'
        r'(?:(?!pub fn number\().)*?submits_on_enter_of: None',
    ),
    # A field with its own onSubmit owns Enter: the callback fires and the
    # keystroke is stopped from also bubbling into the form's submission.
    ('Form', 'field-own-submit-suppression'): (
        'input.rs',
        r'(?s)if submit \{\s*if let Some\(cb\) = &on_submit \{(?:(?!if cleared).)*?cx\.stop_propagation\(\);',
    ),
    # An open ComboBox answers Enter by picking: the list keeps the key, and
    # a closed one — with nothing to commit or revert — lets it bubble.
    ('Form', 'combobox-enter-suppression'): (
        'combo_box.rs',
        r'(?s)crate::list_nav::Move::Activate => \{(?:(?!Move::Ignore).)*?if key == "enter" \{\s*cx\.stop_propagation\(\);',
    ),
    # Constraint validation bars a read-only field: no required emptiness, no
    # stored error, no first-invalid nomination — while its value still
    # submits. The gate reads the mirror `Input::render` writes.
    ('Form', 'read-only-bar'): (
        'form.rs',
        r'(?s)(?=.*type ReadReadOnly = Arc<dyn Fn\(&App\) -> bool)'
        r'(?=.*!f\.is_read_only\(cx\))'
        r'(?=.*!field\.is_read_only\(cx\))'
        r'(?=.*fn is_read_only\(&self, cx: &App\) -> bool)',
    ),
    # Pinned v3 builds InputOTP on a single text input whose cells share one
    # focus handle: a focused Enter participates like any single-line field.
    ('Form', 'otp-enter'): (
        'form.rs',
        r'(?s)pub fn code\(name: impl Into<SharedString>, state: Entity<crate::input_otp::OtpState>\)'
        r'(?:(?!pub fn text_value\().)*?submits_on_enter_of: Some',
    ),
    # HeroUI's Form row: "Server-side validation errors mapped by field name.
    # Displayed immediately and cleared when user modifies the field." The
    # routing half: delivery runs as the stack's LAST child (a zero-size
    # canvas), because the form renders before its fields and the names the
    # routing keys on are written by the fields themselves; each named
    # field's own state receives its messages, and the block decision reads
    # that same channel (`has_server_errors`), so nothing else can render or
    # enforce them. `\s*` around the `.` only tolerates rustfmt wrapping the
    # `record.get(&name)` lookup across lines.
    ('Form', 'server-errors-route'): (
        'form.rs',
        r'(?s)(?=.*fn deliver_server_errors)'
        r'(?=.*record\s*\.\s*get\(&name\))'
        r'(?=.*gpui::canvas\()'
        r'(?=.*field\.deliver_server_errors\(&record, cx\);)'
        r'(?=.*f\.has_server_errors\(cx\))',
    ),
    # "cleared when user modifies the field": the suppression lives in the
    # edited field's own edit path, inside the `changed` gate a caret move
    # cannot set, so only this field's messages clear and its siblings keep
    # theirs. The same stroke refreshes the stored validity mirror (the
    # `refresh_stored_validity` call that follows), so a synchronous submit
    # inside `on_change` never reads the error the edit just answered.
    # (NumberField's steppers and the OTP's accepted edits clear the same
    # slot — and refresh the same mirror — through their own modules.)
    ('Form', 'server-errors-suppress'): (
        'input.rs',
        r'(?s)if changed \{(?:(?!if let Some\(cb\)).){0,400}?clear_routed_errors\(\)',
    ),
    # Reset hides the routed errors without rewinding the field's delivery
    # receipt — the record that delivered already named the field, so a
    # re-render passing a clone cannot resurrect a message.
    ('Form', 'server-errors-reset'): (
        'form.rs',
        r'(?s)for clear in &clear_server_errors \{\s*clear\(cx\);\s*\}',
    ),
    # Re-arm is receipted per field, not per form: each `set_server_errors`
    # closure short-circuits when the field's own state already carries the
    # record's revision, and writes messages + revision in one update — so a
    # genuinely new record (content equal or not) re-arms every named field,
    # a clone re-arms nothing, and reordering or replacing the registrations
    # re-arms nothing either.
    ('Form', 'server-errors-rearm'): (
        'form.rs',
        r"(?s)routed_revision\(\) == revision \{\s*return;\s*\}"
        r".*?set_routed\(messages, revision\)",
    ),
    # The record's identity contract: revisions minted at construction and
    # kept by Clone, with PartialEq comparing content only.
    ('Form', 'server-errors-record'): ('validation.rs', RECORD_IDENTITY),
    ('Dropdown', 'focus-return'): ('dropdown.rs', r'back_to_trigger'),
    # The three dialogs cannot reach the caller's trigger, and do not need to:
    # `Window::focused` names whatever held the focus when the dialog claimed
    # it, and `release_dialog_focus` hands it back from the one place every
    # close path passes through.
    ('Modal', 'focus-return'): ('modal.rs', r'release_dialog_focus'),
    ('Drawer', 'focus-return'): ('drawer.rs', r'release_dialog_focus'),
    ('AlertDialog', 'focus-return'): ('alert_dialog.rs', r'release_dialog_focus'),
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
    # element and its ancestors. The gate that stops the claim from stealing
    # focus from a field inside the dialog lives in `claim_dialog_focus`, which
    # also parks the handle the close hands the focus back to. Shared by all
    # three so they cannot spell it differently.
    ('Modal', 'panel-focus'): ('modal.rs', r'claim_dialog_focus\(&self\.id, &focus_handle'),
    ('Drawer', 'panel-focus'): ('drawer.rs', r'claim_dialog_focus\(&self\.id, &focus_handle'),
    ('AlertDialog', 'panel-focus'): (
        'alert_dialog.rs',
        r'claim_dialog_focus\(&self\.id, &focus_handle',
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
    # A sortable header is a tab stop so gpui fires its click listeners for
    # Enter and Space; a plain one is focusable but not a stop.
    ('Table', 'sort-keys'): ('table.rs', r'crate::util::tab_stop_handle\(id, window, cx\)'),
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
        r'.*?reports_changes\(mode\).*?!disallow_empty'
        r'.*?!selected_now\.is_empty\(\).*?let next = HashSet::new\(\)'
        r'.*?selection_own_for_keys.*?on_selection_change'
        r'.*?stop_propagation\(\)',
    ),
    ('TagGroup', 'escape-clear'): (
        'tag_group.rs',
        r'(?s)key_name == "escape".*?!event\.keystroke\.modifiers\.modified\(\)'
        r'.*?reports_changes\(mode\).*?!disallow_empty'
        r'.*?!selected_now\.is_empty\(\).*?HashSet::new\(\)'
        r'.*?selection_own_for_keys.*?on_selection_change.*?stop_propagation\(\)',
    ),
    # TagGroup's Shift range halves: the same range helper with its raw `all`
    # collapse and replace-old-range semantics, the per-platform Home/End
    # registration gate -- macOS admits none, Shift, Alt, and Alt+Shift and
    # an outside chord returns before anything moves, Windows and Linux
    # admit none, Shift, Control, and Control+Shift; the browser matcher has
    # no Fn flag, so the gate ignores GPUI's function modifier -- plus the
    # extension gate that extends only from Control+Shift off macOS, the
    # seat that ends a raw `all` when an extension lands, and the Shift+Click
    # route.
    ('TagGroup', 'shift-range'): (
        'tag_group.rs',
        r'(?s)\A(?=.*fn extend_selection_range\()(?=.*if range\.is_all \{)'
        r'(?=.*anchor\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*current\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*let extends_selection = modifiers\.shift)'
        r'(?=.*&& mode == SelectionMode::Multiple)'
        r'(?=.*fn home_end_registered\(modifiers: gpui::Modifiers, macos: bool\) -> bool \{)'
        r'(?=.*if macos \{\s*\n\s*!modifiers\.control && !modifiers\.platform\s*\n\s*\} else \{\s*\n\s*!modifiers\.alt && !modifiers\.platform\s*\n\s*\})'
        r'(?=.*matches!\(key, "home" \| "end"\)\s*\n\s*&& !home_end_registered\([^)]*cfg!\(target_os = "macos"\),?\s*\)\s*\{\s*\n\s*return;)'
        r'(?=.*fn shift_home_end_extends\(key_name: &str, control: bool, macos: bool\) -> bool \{)'
        r'(?=.*!matches!\(key_name, "home" \| "end"\) \|\| \(!macos && control\))'
        r'(?=.*shift_home_end_extends\(\s*key_name,\s*modifiers\.control,\s*cfg!\(target_os = "macos"\),?\s*\))'
        r'(?=.*fn seat\(&mut self, target: SharedString\))(?=.*self\.is_all = false;)'
        r'(?=.*ev\.modifiers\(\)\.shift && mode == SelectionMode::Multiple)',
    ),
    # ListBox's Shift range halves: the same range helper with its raw `all`
    # collapse, the per-platform Home/End registration gate -- macOS admits
    # none, Shift, Alt, and Alt+Shift and an outside chord returns before
    # anything moves, Windows and Linux admit none, Shift, Control, and
    # Control+Shift; the browser matcher has no Fn flag, so the gate ignores
    # GPUI's function modifier -- plus the extension gate that extends only
    # from Control+Shift off macOS, the keyboard extension's seat, the
    # Shift+Click route with the pointer seat, and the pointer deselect that
    # ends a raw `all` so the next extension extends instead of collapsing.
    ('ListBox', 'shift-range'): (
        'list_box.rs',
        r'(?s)\A(?=.*fn extend_selection_range\()(?=.*if range\.is_all \{)'
        r'(?=.*anchor\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*current\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*let extends_selection = modifiers\.shift)'
        r'(?=.*&& mode == SelectionMode::Multiple)'
        r'(?=.*fn home_end_registered\(modifiers: gpui::Modifiers, macos: bool\) -> bool \{)'
        r'(?=.*if macos \{\s*\n\s*!modifiers\.control && !modifiers\.platform\s*\n\s*\} else \{\s*\n\s*!modifiers\.alt && !modifiers\.platform\s*\n\s*\})'
        r'(?=.*matches!\(key_name, "home" \| "end"\)\s*\n\s*&& !home_end_registered\([^)]*cfg!\(target_os = "macos"\),?\s*\)\s*\{\s*\n\s*return;)'
        r'(?=.*fn shift_home_end_extends\(key_name: &str, control: bool, macos: bool\) -> bool \{)'
        r'(?=.*!matches!\(key_name, "home" \| "end"\) \|\| \(!macos && control\))'
        r'(?=.*shift_home_end_extends\(\s*key_name,\s*modifiers\.control,\s*cfg!\(target_os = "macos"\),?\s*\))'
        r'(?=.*range\.current = Some\(target\.clone\(\)\);)'
        r'(?=.*ev\.modifiers\(\)\.shift && mode == SelectionMode::Multiple)'
        r'(?=.*range\.anchor = Some\(key\.clone\(\)\);)'
        r'(?=.*if range\.is_all \{\s*\n\s*\*range = ListBoxSelectionRange::default\(\);)',
    ),
    # Table's Shift range halves: the same range helper with its raw `all`
    # collapse and replace-old-range semantics, the per-platform Home/End
    # registration gate -- macOS admits none, Shift, Alt, and Alt+Shift and
    # an outside chord returns before anything moves, Windows and Linux
    # admit none, Shift, Control, and Control+Shift; the browser matcher has
    # no Fn flag, so the gate ignores GPUI's function modifier -- plus the
    # extension gate that extends only from Control+Shift off macOS, the
    # no-cursor Shift+Home/End settle that the same predicate promotes into
    # an extension from Control+Shift, the keyboard extension's seat that
    # ends a raw `all`, and the Shift+Click route.
    ('Table', 'shift-range'): (
        'table.rs',
        r'(?s)\A(?=.*fn extend_selection_range\()(?=.*if range\.is_all \{)'
        r'(?=.*anchor\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*current\.as_ref\(\)\.unwrap_or\(target\))'
        r'(?=.*let extends_selection = modifiers\.shift)'
        r'(?=.*&& mode == SelectionMode::Multiple)'
        r'(?=.*fn home_end_registered\(modifiers: gpui::Modifiers, macos: bool\) -> bool \{)'
        r'(?=.*if macos \{\s*\n\s*!modifiers\.control && !modifiers\.platform\s*\n\s*\} else \{\s*\n\s*!modifiers\.alt && !modifiers\.platform\s*\n\s*\})'
        r'(?=.*matches!\(key_name, "home" \| "end"\)\s*\n\s*&& !home_end_registered\([^)]*cfg!\(target_os = "macos"\),?\s*\)\s*\{\s*\n\s*return;)'
        r'(?=.*fn shift_home_end_extends\(key_name: &str, control: bool, macos: bool\) -> bool \{)'
        r'(?=.*!matches!\(key_name, "home" \| "end"\) \|\| \(!macos && control\))'
        r'(?=.*shift_home_end_extends\(\s*key_name,\s*modifiers\.control,\s*cfg!\(target_os = "macos"\),?\s*\))'
        r'(?=.*let initial_home_end_extends = shift_home_end_extends\(\s*"home",\s*modifiers\.control,\s*cfg!\(target_os = "macos"\),?\s*\);)'
        r'(?=.*range\.current = Some\(target\.clone\(\)\);)'
        r'(?=.*range\.is_all = false;)'
        r'(?=.*ev\.modifiers\(\)\.shift && mode == SelectionMode::Multiple)',
    ),
    # Select's Shift range halves, on option indices instead of keys: the same
    # range helper with its raw `all` collapse and replace-old-range semantics,
    # the anchor held in keyed state beside the cursor (`select-{}-range`) so
    # it survives closing and reopening the popover, the extension gate that
    # extends from plain Shift on the arrows and pages but only from
    # Control+Shift Home/End off macOS and reuses the registration map, the
    # mode-independent registration gate that leaves an unregistered chord
    # entirely inert in either mode, the null-cursor Shift+Home/End guard
    # before cursor seating, the pointer cursor seat for the following
    # keyboard move, and the Shift+Click route.
    ('Select', 'shift-range'): (
        'select.rs',
        r'(?s)\A(?=.*fn extend_selection_range\()(?=.*if range\.is_all \{)'
        r'(?=.*range\.anchor\.unwrap_or\(target\))'
        r'(?=.*range\.current\.unwrap_or\(target\))'
        r'(?=.*select-\{\}-range)'
        r'(?=.*let extends_selection = multiple\s*\n\s*&& modifiers\.shift)'
        r'(?=.*fn home_end_registered\(modifiers: gpui::Modifiers, macos: bool\) -> bool \{)'
        r'(?=.*if macos \{\s*\n\s*!modifiers\.control && !modifiers\.platform\s*\n\s*\} else \{\s*\n\s*!modifiers\.alt && !modifiers\.platform\s*\n\s*\})'
        r'(?=.*if matches!\(key, "home" \| "end"\)\s*&& !home_end_registered\(modifiers, cfg!\(target_os = "macos"\)\)\s*\{\s*\n\s*return;)'
        r'(?=.*if matches!\(key, "home" \| "end"\)\s*&& modifiers\.shift\s*&& from\.is_none\(\)\s*\{\s*\n\s*return;)'
        r'(?=.*let exact_shift_navigation =\s*\n\s*home_end_registered\(modifiers, cfg!\(target_os = "macos"\)\);)'
        r'(?=.*fn shift_home_end_extends\(key_name: &str, control: bool, macos: bool\) -> bool \{)'
        r'(?=.*!matches!\(key_name, "home" \| "end"\) \|\| \(!macos && control\))'
        r'(?=.*shift_home_end_extends\(\s*key,\s*modifiers\.control,\s*cfg!\(target_os = "macos"\),?\s*\))'
        r'(?=.*range\.current = Some\(next\);)'
        r'(?=.*range\.is_all = false;)'
        r'(?=.*let cursor_click = cursor_rows\.clone\(\);)'
        r'(?=.*cursor_click\.update\(cx, \|v, cx\| \{\s*\n\s*\*v = Some\(i\);)'
        r'(?=.*ev\.modifiers\(\)\.shift)',
    ),
    # Select's select-all is the one member that must stay silent: pinned
    # SelectState drops the symbolic `all`, so the evidence demands the Mod+A
    # gate -- with `&& multiple` tied structurally to the branch's own opening
    # brace, not to any later unrelated occurrence -- the enabled-only set,
    # the uncontrolled-only update, and the raw `all` seat. From the gate to
    # the branch's own `cx.stop_propagation()` the tail is one tempered scan:
    # both the reach from the gate to the seat and the reach from the seat to
    # that stop fail the moment the plural callback is ever invoked inside the
    # branch, before or after the seat.
    ('Select', 'select-all'): (
        'select.rs',
        r'(?s)key == "a"\s*\n\s*&& modifiers\.secondary\(\)'
        r'(?=(?:(?!&& multiple).)*&& multiple\s*\n\s*\{)'
        r'(?=.*stops\.iter\(\)\.copied\(\)\.collect\(\))'
        r'(?:(?!on_select_all).){0,2500}?is_all: true'
        r'(?:(?!on_select_all).)*?cx\.stop_propagation\(\)',
    ),
    # A multiple Select answers no typeahead on its closed trigger only: the
    # closed gate returns before the typeahead buffer is ever touched, and the
    # open list's cursor-mover must stay ungated -- the pinned RAC ListBox
    # keeps its type-select in multiple mode, so the old `multiple ||`
    # refusal is banned outright.
    ('Select', 'multiple-trigger-typeahead'): (
        'select.rs',
        r'(?s)\A(?=.*if multiple \{\s*\n\s*return;\s*\n\s*\}\s*\n\s*'
        r'if !crate::list_nav::is_typeahead_key\(key\))'
        r'(?!.*multiple \|\| !crate::list_nav::is_typeahead_key\(key\))',
    ),
    # The pointer seat the port must carry: the tag body's mouse-down seats
    # the cursor and the group handle behind a default_prevented guard and a
    # prevent_default that keeps an ancestor's press-focus from stealing the
    # handle back (gpui's own focus-on-press works that way), the remove
    # button's press stops the body handler, and the remove click reports the
    # removal and then seats the owning tag's focus and cursor.
    ('TagGroup', 'pointer-focus'): (
        'tag_group.rs',
        r'(?s)\A(?=.*if window\.default_prevented\(\) \{\s*\n\s*return;)'
        r'(?=.*on_mouse_down\(gpui::MouseButton::Left, move \|_, window, cx\| \{)'
        r'(?=.*window\.focus\(&focus_for_seat\);)'
        r'(?=.*window\.prevent_default\(\);)'
        r'(?=.*on_mouse_down\(gpui::MouseButton::Left, \|_, _, cx\| \{\s*\n\s*cx\.stop_propagation\(\);)'
        r'(?=.*on_remove\(&HashSet::from\(\[key\.clone\(\)\]\), window, cx\);)'
        r'(?=.*window\.focus\(&focus_for_remove\);)'
        r'(?=.*cursor_for_remove\.update\(cx, \|v, cx\| \{\s*\n\s*\*v = index;)',
    ),
    ('Table', 'resize-bounds'): (
        'table.rs',
        r'(?s)DEFAULT_COLUMN_MIN_WIDTH: f32 = 75\..*?floor\(\)\.min\(max\)\.max\(min\)',
    ),
    ('Table', 'resize-keys'): (
        'table.rs',
        RESIZE_KEYS_EVIDENCE,
    ),
    ('ComboBox', 'custom-value-multiple'): (
        'combo_box.rs',
        r'(?s)multiple-mode.{0,80}?custom input independent from the selected items'
        r'.*?if allows_custom_value\s*&& key == "enter"'
        r'.{0,800}?is_none_or\(.{0,500}?cursor_position\(&rows, focused\).{0,200}?\{'
        r'.{0,600}?if !key_multiple && selected_label != state\.read\(cx\)\.value\(\)'
        r'.{0,800}?key_selection_own.{0,800}?had_selection'
        r'.{0,400}?on_selection_change_all.{0,800}?key_close\(window, cx\)',
    ),
    ('ComboBox', 'multiple-row-keys'): (
        'combo_box.rs',
        r'(?s)\A(?=.*Move::Activate => \{)(?=.*if key_multiple \{)'
        r'(?=.*set_value\(String::new\(\)\))'
        r'(?=.*held\.update)(?=.*hidden_query = Some\(String::new\(\)\))'
        r'(?=.*selected_now\.clone\(\))(?=.*toggle_key\(&mut next, &item_key\))'
        r'(?=.*key_selection_own)(?=.*on_selection_change_all)(?=.*on_input_change)',
    ),
    ('ComboBox', 'multiple-row-pointer'): (
        'combo_box.rs',
        r'(?s)\A(?=.*Multiple mode toggles membership)(?=.*if multiple)'
        r'(?=.*selection_own\.clone\(\))(?=.*row_state\.clone\(\))'
        r'(?=.*cursor_for\(&rows, index, Some\(String::new\(\)\)\))'
        r'(?=.*on_click)(?=.*focus_handle\.focus\(window\))'
        r'(?=.*set_value\(String::new\(\)\))'
        r'(?=.*cursor\.update[^;]*Some\(next_cursor\.clone\(\)\))'
        r'(?=.*toggle_key\(&mut next, &value\))'
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
    ('DisclosureGroup', 'default-expanded'): (
        'disclosure.rs',
        r'(?s)default_expanded_keys.*?crate::util::controlled\(.*?self\.default_expanded',
    ),
    ('DisclosureGroup', 'controlled-expanded'): (
        'disclosure.rs',
        r'(?s)crate::util::controlled\(.*?self\.expanded.*?if let Some\(held\) = &own',
    ),
    ('DisclosureGroup', 'selection-modes'): (
        'disclosure.rs',
        r'accordion::next_expanded\(&current, &key, allows_multiple\)',
    ),
    ('DisclosureGroup', 'default-normalization'): (
        'disclosure.rs',
        r'(?s)!self\.allows_multiple_expanded.*?expanded_own\.is_some\(\)'
        r'.*?expanded\.len\(\) > 1.*?find_map.*?held\.update.*?window\.defer',
    ),
    ('DisclosureGroup', 'group-disabled'): (
        'disclosure.rs',
        r'\.is_disabled\(self\.is_disabled\)',
    ),
    ('Disclosure', 'controlled-uncontrolled'): (
        'disclosure.rs',
        r'(?s:impl RenderOnce for Disclosure \{.*?crate::util::controlled\('
        r'.*?self\.is_expanded.*?self\.default_expanded.*?held\.update)',
    ),
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


def table_resize_keys_evidence(source):
    """Require lifecycle evidence inside each owning resizer handler/arm."""
    try:
        resize = source.split('keyboard-resizing', 1)[1]
        outside_and_keys = resize.split('.on_mouse_down_out(', 1)[1]
        outside, keys = outside_and_keys.split('.on_key_down(', 1)
        arms = keys.split('match key {', 1)[1]
        enter_and_rest = arms.split('"enter" => {', 1)[1]
        enter, finish_and_rest = enter_and_rest.split(
            '"escape" | "space" | "tab" if editing => {', 1)
        finish, arrows_and_rest = finish_and_rest.split(
            '"right" | "up" | "left" | "down" if editing => {', 1)
        arrows, _ = arrows_and_rest.split('_ => {}', 1)
    except (IndexError, ValueError):
        return False

    return all((
        re.search(
            r'(?s)keyboard_out\.read\(cx\) == Some\(column_index\).*?'
            r'\*active = None.*?clear_controlled_resize_proposal.*?'
            r'if let Some\(callback\) = &resize_end_for_outside',
            outside,
        ),
        re.search(
            r'(?s)if editing \{.*?\*active = None.*?'
            r'clear_controlled_resize_proposal.*?'
            r'if let Some\(callback\) = &resize_end_for_keys.*?\} else \{.*?'
            r'\*active = Some\(column_index\).*?'
            r'if let Some\(callback\) = &resize_start_for_keys.*?'
            r'cx\.stop_propagation\(\)',
            enter,
        ),
        re.search(
            r'(?s)\*active = None.*?clear_controlled_resize_proposal.*?'
            r'if let Some\(callback\) = &resize_end_for_keys.*?'
            r'cx\.stop_propagation\(\)',
            finish,
        ),
        re.search(
            r'(?s)matches!\(key, "right" \| "up"\).*?10\..*?-10\..*?'
            r'floor\(\).*?\.min\(max_width\).*?\.max\(min_width\).*?'
            r'if let Some\(callback\) = &resize_for_keys.*?'
            r'cx\.stop_propagation\(\)',
            arrows,
        ),
    ))


def evidence_matches(key, evidence, source):
    if key == ('Table', 'resize-keys'):
        return table_resize_keys_evidence(source)
    return re.search(evidence, source)


def main():
    sections = accessibility_sections()
    sources = {}
    claimed = implemented = excused = 0
    missing, unmapped = [], []
    by_reason = {}

    table_source = io.open(SRC + 'table.rs', encoding='utf-8', errors='replace').read()
    for label, token, replacement in (
        ('outside-click exit', '.on_mouse_down_out(', 'REMOVED_RESIZE_EVIDENCE'),
        (
            'combined exit keys',
            '"escape" | "space" | "tab" if editing',
            'REMOVED_RESIZE_EVIDENCE',
        ),
        (
            'resize start callback',
            'if let Some(callback) = &resize_start_for_keys',
            'REMOVED_RESIZE_EVIDENCE',
        ),
        (
            'resize callback',
            'if let Some(callback) = &resize_for_keys',
            'REMOVED_RESIZE_EVIDENCE',
        ),
        (
            'resize end callback',
            'if let Some(callback) = &resize_end_for_keys',
            'REMOVED_RESIZE_EVIDENCE',
        ),
        (
            'Enter propagation stop',
            'cx.stop_propagation();\n                                    }\n'
            '                                    "escape" | "space" | "tab" if editing',
            'REMOVED_RESIZE_EVIDENCE;\n                                    }\n'
            '                                    "escape" | "space" | "tab" if editing',
        ),
        (
            'exit-key propagation stop',
            'cx.stop_propagation();\n                                    }\n'
            '                                    "right" | "up" | "left" | "down" if editing',
            'REMOVED_RESIZE_EVIDENCE;\n                                    }\n'
            '                                    "right" | "up" | "left" | "down" if editing',
        ),
        (
            'arrow propagation stop',
            'cx.stop_propagation();\n                                    }\n'
            '                                    _ => {}',
            'REMOVED_RESIZE_EVIDENCE;\n                                    }\n'
            '                                    _ => {}',
        ),
    ):
        if token not in table_source:
            print('AUDIT READER ERROR: Table.resize-keys self-test cannot find %s' % label)
            return 1
        mutant = table_source.replace(token, replacement, 1)
        if table_resize_keys_evidence(mutant):
            print('AUDIT READER ERROR: Table.resize-keys does not require %s' % label)
            return 1

    for page in SUCCESSFUL_FORM_CONTROLS:
        key = (page, 'disabled-form-omission')
        if key not in EVIDENCE and key not in WONT_DO:
            unmapped.append('%-14s %-14s' % key)
    for page in CLOSE_ON_BLUR:
        key = (page, 'close-on-blur')
        if key not in EVIDENCE and key not in WONT_DO:
            unmapped.append('%-14s %-14s' % key)
    for page in COMBOBOX_BLUR_COMMIT:
        key = (page, 'blur-commit')
        if key not in EVIDENCE and key not in WONT_DO:
            unmapped.append('%-14s %-14s' % key)
    for page in FORM_SUBMISSION:
        for claim in ('implicit-enter', 'blocked-focus', 'textarea-suppression',
                      'field-own-submit-suppression', 'combobox-enter-suppression',
                      'read-only-bar', 'otp-enter', 'shared-funnel',
                      'server-errors-route', 'server-errors-suppress',
                      'server-errors-reset', 'server-errors-rearm',
                      'server-errors-record'):
            key = (page, claim)
            if key not in EVIDENCE and key not in WONT_DO:
                unmapped.append('%-14s %-14s' % key)

    # The derived claims first, so their numbers land in the same totals.
    # Deduplicated: a component appears in several of these tuples (a text area
    # has both the keys and the caret), and counting it once per tuple inflated
    # every total.
    derived = dict.fromkeys(
        ARROW_NAV + REMOVE_KEY + TOOLBAR_FOCUS + OVERLAY_DISMISS + CLOSE_ON_BLUR + COMBOBOX_BLUR_COMMIT
        + SPIN_KEYS + AREA_KEYS
        + FOCUS_OPEN + TOOLTIP_SEQUENCE + TEXT_KEYS + POINTER_CARET + SORT_KEYS + TREE_KEYS
        + TABLE_TYPEAHEAD + TABLE_PAGING + LISTBOX_PAGING + SELECT_PAGING + POPUP_PAGING
        + AUTOCOMPLETE_PAGING + SELECT_ALL_KEYS
        + ESCAPE_CLEAR_KEYS + RANGE_SELECT + POINTER_FOCUS
        + MULTIPLE_TRIGGER_TYPEAHEAD
        + COMBOBOX_MULTIPLE_KEYS
        + RESIZE_BOUNDS + RESIZE_KEYS
        + LOAD_MORE
        + FOCUS_RETURN + SCROLL_INTO_VIEW + CALENDAR_PAGING + CALENDAR_SECTION_BOUNDS
        + PANEL_FOCUS + SUCCESSFUL_FORM_CONTROLS
        + DISCLOSURE_GROUP_STATE
        + DISCLOSURE_STATE
        + FORM_SUBMISSION
    )
    for page in derived:
        for claim in ('arrow-nav', 'remove-key', 'toolbar-end-stops',
                      'toolbar-tab-exit', 'toolbar-focus-restore', 'toolbar-nested',
                      'dismiss',
                      'spin-keys', 'area-keys',
                      'focus-open', 'global-sequence', 'text-keys', 'pointer-caret', 'sort-keys', 'tree-keys',
                      'table-typeahead', 'table-page-down', 'table-page-up-header',
                      'listbox-paging',
                      'select-paging',
                      'popup-paging',
                      'autocomplete-paging',
                      'select-all', 'escape-clear', 'shift-range', 'pointer-focus', 'resize-bounds',
                      'multiple-trigger-typeahead',
                      'resize-keys', 'focus-return', 'scroll-into-view', 'calendar-paging',
                      'calendar-section-bounds', 'panel-focus', 'load-more',
                      'disabled-form-omission', 'close-on-blur', 'blur-commit',
                      'custom-value-multiple', 'multiple-row-keys',
                      'multiple-row-pointer', 'default-expanded',
                      'controlled-expanded', 'selection-modes',
                      'default-normalization', 'group-disabled',
                      'controlled-uncontrolled',
                      'implicit-enter', 'blocked-focus', 'textarea-suppression',
                      'field-own-submit-suppression', 'combobox-enter-suppression',
                      'read-only-bar', 'otp-enter', 'shared-funnel',
                      'server-errors-route', 'server-errors-suppress',
                      'server-errors-reset', 'server-errors-rearm',
                      'server-errors-record'):
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
            if evidence_matches(key, evidence, sources[module]):
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
        if evidence_matches(key, evidence, sources[module]):
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
            if evidence_matches(key, evidence, sources[module]):
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
