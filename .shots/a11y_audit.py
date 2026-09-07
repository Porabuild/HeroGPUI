"""A component that reports no role reports nothing at all.

gpui only puts an element in the AccessKit tree when it has *both* a
`GlobalElementId` and a role (`gpui-pre-0.3.3/src/window/a11y.rs`: "Nodes
without `GlobalElementId`s cannot produce an AccessKit `NodeId`, and so are not
included in the accessibility tree"; `Interactivity::a11y_role` then filters out
everything with no `override_role`). `element_id_audit.py` guards the first
half. This guards the second: for every component whose upstream equivalent
reports a role, the port must state one.

The contract is not this repository's opinion about ARIA. Every in-scope
component delegates to React Aria Components 1.20.0, which delegates to React
Aria 3.51.0's hooks, and the hooks decide the role. Each row below names the
hook it was read from, in the pinned packages under `web/node_modules`.

## What the four tables mean

| Table | Meaning | How a wrong entry shows up |
|---|---|---|
| `EXPOSES_A_ROLE` | upstream reports a role, and so must the port | the `impl RenderOnce` block must call `a11y`/`a11y_named`; missing is a failure |
| `DELEGATES` | this component renders another one, which carries the node | the block must *not* set a role of its own, and must name its delegate |
| `NO_NODE` | upstream reports no node here either, with the reason | the block must not set a role; if it does, the reason has gone stale |
| `PENDING` | upstream *does* report a node and the port does not yet | the block must not set a role; the entry says what a later wave has to do |

`PENDING` is the difference between "recorded" and "forgotten". A component
listed there is a known gap with a named cause, not an exemption: deleting its
entry and adding an `EXPOSES_A_ROLE` row is exactly the diff a later wave
writes. The same applies one level up, to whole modules: `PENDING_MODULES`
lists every component module the waves so far did not reach, so a module can
never fall out of the audit by never having been mentioned in it.

## Which waves are under contract

Wave 1 brought the form controls in. Wave 2 brought the overlays and the
disclosure family — `modal`, `drawer`, `alert_dialog`, `popover`, `tooltip`,
`dropdown`, `toast`, `disclosure`, `accordion` — and its recurring finding is
worth stating once here, because it is the reason several of those rows say
what they say: React Aria puts the *trigger* half of an overlay contract
(`aria-expanded`, `aria-haspopup`, `aria-controls`) onto the caller's own
button by injecting props through React context, and gpui has no equivalent —
an element cannot be modified by the parent it is handed to. So a component
that builds its own trigger row (`Accordion`) states `aria-expanded`, and a
component whose trigger is a caller-supplied element (`Disclosure`,
`Dropdown`, `Popover`, `Tooltip`) cannot. Wrapping the caller's element in a
second node with a button role would report the trigger twice.

Wave 3 brought the collections, the navigation bars and the pickers —
`list_box`, `table`, `tabs`, `toolbar`, `breadcrumbs`, `pagination`,
`tag_group`, `chip`, `select`, `combo_box`, `autocomplete`. Three things it
learned are worth keeping here:

* **Set and grid position is guarded upstream too.** `useOption`,
  `useGridRow` and `useGrid` add `aria-posinset`/`aria-setsize`/
  `aria-rowindex`/`aria-rowcount`/`aria-colcount` only `if (isVirtualized)`,
  because that is when the rendered rows are a window onto a longer
  collection. This port virtualizes in exactly the same situations, so the
  guard ports with the attribute rather than the attribute alone.
* **What a port cannot count, it does not claim.** A tree table's rows carry
  `aria-level` and `aria-expanded` here, but not the
  `aria-posinset`/`aria-setsize` beside them: upstream counts a row's
  siblings out of the tree collection, and the port's row builder is handed a
  flattened list of visible rows.
* **The trigger-ownership rule bites hardest in the pickers.**
  `useComboBox` does not decorate a trigger button — it turns the *text
  input* into the widget (`role: 'combobox'`, `aria-expanded`,
  `aria-activedescendant`). `ComboBox` and `Autocomplete` here compose a
  `crate::input::Input`, whose role its own render decides, so the field
  keeps its text-input role and the combobox half is stated on the parts the
  picker does own: the popup listbox, its options, and — through gpui's
  descendant-side `aria_active_descendant` — the highlighted row.

`picker_item.rs` and `list_nav.rs` are in that wave's scope but define no
`impl RenderOnce` at all, so there is nothing for any of the four tables to
say about them; they are data and keyboard-resolution helpers.

Wave 4 brought the status and content surface — `spinner`, `alert`, `avatar`,
`badge`, `card`, `form`, `kbd`, `scroll_shadow`, `separator`, `skeleton`,
`surface`, `typography`. Its finding is the inverse of the earlier waves:
two thirds of that surface imports no RAC primitive, authors no `role` and
no `aria-*`, and renders a `dom.div` or `dom.span` with a `data-slot`. A
node there would be an invention, so those rows sit in `NO_NODE`. Two
exceptions keep the wave from being a shrug:

* **`Spinner` is the one status node.** `spinner/spinner.js` hard-codes
  `role: "status"` with `"aria-label": "Loading"` on its root span; it is
  not `ProgressCircle` and does not go through `useProgressBar`.
* **`Separator` is a real gap, not an absence.** RAC's `Separator` defaults
  to `<hr>` (implicit separator) and switches to a `div` with
  `useSeparator`'s `role: 'separator'` when vertical. AccessKit 0.24 has no
  `Role::Separator` (`Splitter` is a pane splitter). `.id()` exists; the
  role cannot be claimed until AccessKit grows the member.

Wave 5 brought the remaining pickers and overlay-backed fields —
`calendar`, `range_calendar`, `date_picker`, `time_field`, `color_picker`.
Three things it learned:

* **A calendar is an application around a grid.** `useCalendarBase` is
  `role: 'application'` named by the visible range; `useCalendarGrid` is
  `role: 'grid'`; `useCalendarCell` is a `gridcell` wrapping a `button`.
  This port draws one pressable circle per date, so the button is the node
  and the gridcell wrapper (no id) stays off the tree.
* **Date and time fields are groups of textboxes.** `useDateField` /
  `useTimeField` put `role: 'group'` on the box and `role: 'textbox'` on
  each segment (`useDateSegment` deletes the `spinbutton` role it first
  builds). The hidden native input is `presentation` and has no counterpart
  element.
* **`ColorSwatch` reports `role="img"` when the caller names it.**
  `useColorSwatch` is `role: 'img'` named by the colour. Unnamed previews
  stay off the tree so two instances do not fold into one image.

## The other two rules

* **No raw gpui accessibility builders outside `a11y.rs`.** gpui exposes 25 of
  them; using them directly would put the "when is `aria-checked` mixed", "does
  an indeterminate bar report a value" decisions in each component instead of
  once in `crates/herogpui-components/src/a11y.rs`, which is where they are
  checked against the pinned hooks. Components call the `a11y_*` wrappers.
* **No accessibility state without a role.** Setting `aria-checked` or a value
  range on an element that reports no role produces no node, so the call is
  silently dead. Any block using the state helpers must also set a role.

Run from the repository root:

    python3 .shots/a11y_audit.py
    python3 .shots/a11y_audit.py --self-test

The self-test runs as part of the audit, not only behind the flag: a parser
that matches nothing reports a clean tree, and so does a clean tree.
"""
import io
import os
import re
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from component_source import list_modules, read_module, read_path

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SRC = 'crates/herogpui-components/src'

# The module this wave centralised the contract in. It is the only file allowed
# to call gpui's own accessibility builders.
FOUNDATION = 'a11y.rs'

# The wave this audit currently enforces, named so a failure message can say
# which slice of the port is under contract.
WAVE = ('waves 1-5 — form controls, overlays and disclosure, '
        'collections, navigation and pickers, status and content, '
        'calendars and overlay-backed fields')


# --------------------------------------------------------------------------
# (module, struct) -> the role, and the pinned upstream source that decides it.
#
# Every reference is a file under `web/node_modules`, at the versions
# `docs/agents/parity.md` pins: react-aria 3.51.0, react-aria-components 1.20.0,
# @heroui/react 3.2.4.
# --------------------------------------------------------------------------
EXPOSES_A_ROLE = {
    ('button.rs', 'Button'):
        'Role::Button. `@heroui/react/dist/components/button/button.js` renders '
        'RAC `Button`, which is a native `<button>`; `react-aria/.../button/'
        'useButton.js` only adds an explicit `role: \'button\'` when the '
        'element type is not a button.',
    ('close_button.rs', 'CloseButton'):
        'Role::Button with a literal name. `close-button/close-button.js` '
        'hard-codes `aria-label="Close"` on the RAC `Button`, because its '
        'default child is an icon with no text.',
    ('toggle_button.rs', 'ToggleButton'):
        'Role::Button plus `aria-pressed` (`react-aria/.../button/'
        'useToggleButton.js`), swapped for Role::RadioButton plus '
        '`aria-checked` inside a single-selection group '
        '(`useToggleButtonGroup.js`: `buttonProps.role = \'radio\'`, '
        '`delete buttonProps[\'aria-pressed\']`).',
    ('toggle_button.rs', 'ToggleButtonGroup'):
        'Role::RadioGroup when the group selects one member, else '
        'Role::Toolbar, with `aria-orientation` in both cases. '
        '`useToggleButtonGroup.js` starts from `useToolbar` and overrides only '
        'the role.',
    ('checkbox.rs', 'Checkbox'):
        'Role::CheckBox with tri-state `aria-checked`. `useCheckbox.js` renders '
        'a native `<input type="checkbox">` and sets its `indeterminate` '
        'property, which is what makes a checkbox report `aria-checked="mixed"`.',
    ('checkbox.rs', 'CheckboxGroup'):
        'Role::Group. `react-aria/.../checkbox/useCheckboxGroup.js` returns '
        '`role: \'group\'`, named and described through `useField`.',
    ('switch.rs', 'Switch'):
        'Role::Switch. `react-aria/.../switch/useSwitch.js` is `useToggle` with '
        '`role: \'switch\'` forced onto the input.',
    ('radio_group.rs', 'RadioGroup'):
        'Role::RadioGroup with `aria-orientation` (always present, defaulting '
        'to vertical) on the group, and Role::RadioButton with `aria-checked` '
        'on each option — `useRadioGroup.js` and the native '
        '`<input type="radio">` each option renders.',
    ('input.rs', 'Input'):
        'The role of the rendered input type: `useTextField.js` passes `type` '
        'through untouched (`inputOnlyProps = { type, pattern }`), and a '
        'multi-line field is RAC `TextArea`, a `<textarea>`. Mapped in '
        '`InputType::a11y_role`.',
    ('input_otp.rs', 'InputOTP'):
        'Role::TextInput. v3 wraps the `input-otp` package, which renders one '
        'real `<input>` behind the slots with `aria-placeholder` and '
        '`autocomplete="one-time-code"` and no role of its own; the slots are '
        'presentational divs.',
    ('number_field.rs', 'NumberField'):
        'Role::Group on the box and Role::Button on each stepper. '
        '`useNumberField.js` returns `groupProps` with `role: \'group\'`, and '
        'names the steppers "Increase {label}" / "Decrease {label}". The '
        '`spinbutton` role `useSpinButton` produces is deleted again '
        '(`role: null`) before it reaches the input.',
    ('slider.rs', 'Slider'):
        'Role::Group on the control (`useSlider.js`: `groupProps.role = '
        '\'group\'`) and Role::Slider on each thumb, which is an '
        '`<input type="range">` carrying min/max/step/value, '
        '`aria-orientation` and `aria-valuetext` '
        '(`useSliderThumb.js`).',
    ('progress.rs', 'ProgressBar'):
        'Role::ProgressIndicator, or Role::Meter when rendering for '
        '`Meter`. `useProgressBar.js` keeps `aria-valuemin`/`aria-valuemax` '
        'always and drops `aria-valuenow`/`aria-valuetext` when indeterminate.',
    ('link.rs', 'Link'):
        'Role::Link. RAC `Link` renders a native `<a>`; `useLink.js` adds the '
        'explicit role only for a non-anchor element.',

    # ---- wave 4: status and content -------------------------------------
    ('spinner.rs', 'Spinner'):
        'Role::Status named "Loading". `spinner/spinner.js` imports no '
        '`react-aria-components` primitive at all: it hard-codes '
        '`role: "status"` with `"aria-label": "Loading"` on its `dom.span` '
        'root, and `aria-hidden: true` on the `SpinnerPrimitive` svg inside '
        'that root. A `Spinner` is not a `ProgressCircle`: v3 exports the two '
        'separately, and only the circle goes through `useProgressBar`.',

    # ---- wave 5: calendars and overlay-backed fields ---------------------
    ('calendar.rs', 'Calendar'):
        'Role::Application on the root (`useCalendarBase.js`: '
        '`role: \'application\'`, named by the visible range), Role::Grid '
        'on each month (`useCalendarGrid.js`: `role: \'grid\'`), Role::Button '
        'plus `aria-selected` on each day (`useCalendarCell.js` puts '
        '`role: \'gridcell\'` on the `<td>` and `role: \'button\'` on the '
        'inner date; this port draws one circle, which is the pressable '
        'half), Role::Button named "Previous"/"Next" on the nav, and '
        'Role::Button plus `aria-expanded` on the year-picker trigger. The '
        'weekday header is `aria-hidden` upstream.',
    ('range_calendar.rs', 'RangeCalendar'):
        'The same `useCalendarBase` / `useCalendarGrid` / `useCalendarCell` '
        'contract as `Calendar`, through `useRangeCalendar.js`. Caps and '
        'in-range days still report as the pressable button; the painted '
        'track under a run has no id and no node.',
    ('date_picker.rs', 'DatePicker'):
        'Role::Group on the field box (`useDatePicker.js`: `role: \'group\'`) '
        'and Role::Button plus `aria-expanded` on the calendar trigger '
        '(`useOverlayTrigger`). The composed `DateField` and `Calendar` '
        'carry their own nodes.',
    ('date_picker.rs', 'DateRangePicker'):
        'Role::Group on the field box (`useDateRangePicker.js`: '
        '`role: \'group\'`) and Role::Button plus `aria-expanded` on the '
        'trigger, the same overlay-trigger half as `DatePicker`.',
    ('date_picker.rs', 'DateField'):
        'Role::Group on the box (`useDateField.js`: `role: \'group\'`) and '
        'Role::TextInput on each segment. `useDateSegment.js` first builds '
        'a `spinbutton` and then rewrites it to `role: \'textbox\'`. The '
        'hidden native input is `role: \'presentation\'` and has no '
        'counterpart element.',
    ('time_field.rs', 'TimeField'):
        'Role::Group on the box — `useTimeField` is `useDateField` — '
        'Role::TextInput on each segment, and Role::Button named '
        '"Increase"/"Decrease" on the steppers (`useNumberField`\'s '
        'stepper naming, reused because `useDateSegment` is a spinbutton '
        'under the textbox rewrite).',
    ('color_picker.rs', 'ColorArea'):
        'Role::Group on the area (`useColorArea.js`: `role: \'group\'`). '
        'The thumb is `role: \'presentation\'` and the two hidden range '
        'inputs have no counterpart element.',
    ('color_picker.rs', 'ColorSlider'):
        'Role::Slider on the track. `useColorSlider` is `useSlider` with '
        'one thumb, so the interactive track is the slider; '
        '`aria-orientation` ports.',
    ('color_picker.rs', 'ColorField'):
        'Role::TextInput on the hex/channel box (`useColorField.js`: '
        '`role: \'textbox\'`). A path that composes `crate::input::Input` '
        'lets that field carry the node instead of stating a second one.',
    ('color_picker.rs', 'ColorSwatchPicker'):
        'Role::RadioGroup on the swatch list and Role::RadioButton plus '
        '`aria-selected` on each swatch. RAC\'s `ColorSwatchPicker` is a '
        'single-select collection of pressable swatches; the checkmark '
        'indicator is `role: "presentation"` in '
        '`color-swatch-picker/color-swatch-picker.js`.',
    ('color_picker.rs', 'ColorPicker'):
        'Role::Button plus `aria-expanded` on the trigger and Role::Dialog '
        'on the popover panel. v3 composes a swatch trigger that opens a '
        '`Popover` around the area/sliders; the overlay is the same '
        '`useDialog` contract wave 2 already carries for `Popover`.',
    ('color_picker.rs', 'ColorSwatch'):
        'Role::Image named by the colour hex. `useColorSwatch.js` is '
        '`role: \'img\'`. Only a swatch the caller named reports a node — '
        '`ColorSwatch::new(color)` takes no id, and a constant would fold '
        'every preview into one image.',

    # ---- remaining id-optional groups -----------------------------------
    ('button_group.rs', 'ButtonGroup'):
        'Role::Group. RAC `Group` in `button-group/button-group.js` is '
        '`role="group"`. Only a group the caller named reports a node.',
    ('input_group.rs', 'InputGroup'):
        'Role::Group. RAC `Group` in `input-group/input-group.js` is '
        '`role="group"`. The held field carries its own node; the group '
        'reports only when the caller named it.',
    ('progress.rs', 'ProgressCircle'):
        'Role::ProgressIndicator with the same `a11y::Range` contract as '
        '`ProgressBar`. `useProgressBar.js` is `role="progressbar"` for '
        'the ring as well as the bar. Only a named ring reports a node.',
    ('field.rs', 'Fieldset'):
        'Role::Group. A native `<fieldset>` is `group`, named by its '
        '`<legend>`. The legend is a sibling component rather than a prop, '
        'so a named fieldset reports the group without inlining that text.',
    ('toast.rs', 'ToastViewport'):
        'Role::Region named "{n} notifications." from the pinned en-US '
        '`useToastRegion.js` strings. Only a viewport the caller named '
        'reports a landmark — two viewports sharing a constant id would '
        'fold every toast path inside them.',

    # ---- wave 2: overlays and disclosure --------------------------------
    ('modal.rs', 'Modal'):
        'Role::Dialog on the panel. `modal/modal.js` wraps RAC `Modal` / '
        '`ModalOverlay` around a `Dialog`, and `react-aria/.../dialog/'
        'useDialog.js` defaults it to `role: \'dialog\'`, named by the '
        'composed `Heading` through `aria-labelledby` (inlined as the title '
        'here). Nothing says "modal": `useDialog` sets no `aria-modal` on '
        'purpose, and AccessKit\'s single role enum has no modality flag.',
    ('drawer.rs', 'Drawer'):
        'Role::Dialog on the panel. `drawer/drawer.js` is the same RAC '
        '`Modal`/`ModalOverlay` + `Dialog` composition as the modal, on a '
        'window edge, so `useDialog.js` gives it the identical contract.',
    ('alert_dialog.rs', 'AlertDialog'):
        'Role::AlertDialog on the panel. `alert-dialog/alert-dialog.js` is '
        'the one dialog in v3 that names its own role (`role: '
        '"alertdialog"` on the RAC `Dialog`). `useDialog.js` also makes the '
        'body the description for that role alone — `aria-describedby` falls '
        'back to the content id only `when role === \'alertdialog\'` — so the '
        'description text joins the accessible name here.',
    ('popover.rs', 'Popover'):
        'Role::Dialog on the panel. `popover/popover.js` composes RAC '
        '`Popover` around a `Dialog`; the `Popover` wrapper reports nothing '
        'and `useDialog.js` supplies the role. Named by the composed '
        '`Heading`, i.e. this port\'s `title`.',
    ('tooltip.rs', 'Tooltip'):
        'Role::Tooltip on the tip. `react-aria/.../tooltip/useTooltip.js` is '
        'a single `role: \'tooltip\'`. Upstream leaves the tip unnamed and '
        'points the trigger\'s `aria-describedby` at it; with no id graph the '
        'port names the tip with the content that describedby resolved to.',
    ('dropdown.rs', 'Menu'):
        'Role::Menu on the list panel (`react-aria/.../menu/useMenu.js`: '
        '`role: \'menu\'`), plus a role per row from '
        '`.../menu/useMenuItem.js`: `menuitem`, or `menuitemradio` / '
        '`menuitemcheckbox` when the row is selectable and is not a submenu '
        'trigger, with `aria-checked` under the same guard and '
        '`aria-expanded` on a submenu trigger instead. `MenuItem::Separator` '
        'and `MenuItem::SectionLabel` report nothing: v3\'s `Dropdown` '
        'composes no separator part at all, and `useMenuSection.js` gives '
        'the section heading `role: \'presentation\'` and puts the label on '
        'the group through `aria-labelledby`, which needs a wrapping group '
        'the port\'s flat item list does not have.',
    ('accordion.rs', 'Accordion'):
        'Role::Button plus `aria-expanded` on each trigger row and '
        'Role::Group on each open panel. `accordion/accordion.js` renders an '
        'RAC `Button slot="trigger"` and a `DisclosurePanel` inside a '
        '`Disclosure`; `react-aria/.../disclosure/useDisclosure.js` supplies '
        '`aria-expanded` on the button and `role: \'group\'` + '
        '`aria-labelledby: triggerId` on the panel. The trigger row is this '
        'component\'s own element, so unlike `Disclosure` it can state both '
        'halves.',
    ('disclosure.rs', 'Disclosure'):
        'Role::Group on the open body, named by the trigger\'s text — '
        '`DisclosurePanel` in `Disclosure.mjs` is `role: role = \'group\'` '
        'and `useDisclosure.js` labels it `aria-labelledby: triggerId`. The '
        'trigger\'s own `aria-expanded` is *not* stated: v3 composes the '
        'trigger as a caller-supplied `<Button slot="trigger">` and RAC '
        'injects the prop through React context, which gpui has no '
        'equivalent of (see the note in `crates/herogpui-components/src/'
        'a11y.rs`).',
    # ---- wave 3: collections, navigation and pickers --------------------
    ('list_box.rs', 'ListBox'):
        'Role::ListBox on the list and Role::ListBoxOption on each option. '
        '`react-aria/dist/private/listbox/useListBox.mjs` is one literal '
        '`role: \'listbox\'` with `\'aria-orientation\': orientation`, and '
        '`.../listbox/useOption.mjs` is `role: \'option\'` with '
        '`\'aria-selected\': selectionMode !== \'none\' ? isSelected : '
        'undefined` and an `aria-posinset`/`aria-setsize` pair added only '
        '`if (isVirtualized)` — which is exactly this port\'s `row_height` / '
        '`estimated_row_height` paths. `ListBoxItem::Section` and '
        '`::Separator` report nothing: `.../listbox/useListBoxSection.mjs` '
        'gives the heading `role: \'presentation\'` and puts the label on a '
        'wrapping group the port\'s flat item list does not have, the same '
        'shape as `Dropdown`\'s section label.',
    ('tabs.rs', 'Tabs'):
        'Role::TabList on the list, Role::Tab plus `aria-selected` on each '
        'tab, Role::TabPanel on the open panel. '
        '`react-aria/dist/private/tabs/useTabList.mjs` is `role: \'tablist\'` '
        'with `\'aria-orientation\': orientation`; `.../tabs/useTab.mjs` is '
        '`role: \'tab\'` with `\'aria-selected\': isSelected` and '
        '`\'aria-controls\': isSelected ? tabPanelId : undefined`; '
        '`.../tabs/useTabPanel.mjs` is `role: \'tabpanel\'`, labelled by the '
        'selected tab. RAC\'s `Tabs.mjs` sets no role of its own. Only '
        '`aria-controls` is dropped — no gpui builder and no id graph.',
    ('toolbar.rs', 'Toolbar'):
        'Role::Toolbar, or Role::Group when nested, with `aria-orientation` '
        'in both cases. `react-aria/dist/private/toolbar/useToolbar.mjs` is '
        '`role: !isInToolbar ? \'toolbar\' : \'group\'`, and RAC\'s '
        '`Toolbar.mjs` spreads those props unmodified. The port already '
        'computes that same nested answer to switch off its key handling. A '
        'toolbar reports a node only when the caller gave it an id: '
        '`Toolbar::new()` takes none, and the constant its keyed focus scope '
        'falls back to would fold every unnamed toolbar into one node.',
    ('breadcrumbs.rs', 'Breadcrumbs'):
        'Role::List on the bar, Role::ListItem on each crumb, Role::Link on '
        'each crumb\'s label. RAC\'s `Breadcrumbs.mjs` puts '
        '`useBreadcrumbs`\' `navProps` on an `<ol>` rather than a `<nav>` '
        '(`...dom.ol, {...mergeProps(DOMProps, navProps)}`), carrying '
        '`\'aria-label\': ariaLabel || strings.format(\'breadcrumbs\')` — '
        '`Breadcrumbs` in the pinned en-US strings. Each `Breadcrumb` is a '
        '`<li>` holding a `<Link>`, which `@heroui/react`\'s '
        '`breadcrumbs.js` composes explicitly. Its `aria-current="page"` has '
        'no gpui builder.',
    ('pagination.rs', 'Pagination'):
        'Role::Navigation on the bar and Role::Button on every page link and '
        'nav arrow. `@heroui/react/dist/components/pagination/pagination.js` '
        'is the one component here that uses no RAC collection at all: a '
        'plain `dom.nav` with `role: "navigation"` and `"aria-label": '
        '"pagination"` hard-coded, around a `dom.ul`/`dom.li` list whose '
        'links are RAC `Button`s with `"aria-current": isActive ? "page" : '
        'undefined`. The `<ul>`/`<li>` layer has no counterpart element — the '
        'port draws the cells straight into the row — and `aria-current` has '
        'no gpui builder.',
    ('tag_group.rs', 'TagGroup'):
        'Role::Grid on the tag list, or Role::Group when it is empty, and '
        'Role::Row plus `aria-selected` on each tag with Role::Button on its '
        'remove control. `react-aria/dist/private/tag/useTagGroup.mjs` is '
        'literally `role: state.collection.size ? \'grid\' : \'group\'` over '
        '`useGridList`, and `.../tag/useTag.mjs` builds on '
        '`.../gridlist/useGridListItem.mjs`\'s `role: \'row\'`. The '
        '`role: \'gridcell\'` node that hook also returns has no counterpart '
        'element: this port draws a tag\'s contents straight into the row. '
        'The remove button\'s `Remove` label is the pinned en-US string.',
    ('table.rs', 'Table'):
        'Role::Grid on the table — Role::TreeGrid when the collection nests '
        '— Role::Row on the header row and every body row, Role::RowGroup on '
        'the body, Role::ColumnHeader on every header cell and '
        'Role::GridCell (Role::RowHeader for a row-header column) on every '
        'body cell. `react-aria/dist/private/table/useTable.mjs` overrides '
        '`.../grid/useGrid.mjs`\'s `role: \'grid\'` to `\'treegrid\'` only '
        '`if (state.treeColumn != null)`; `.../table/useTableCell.mjs` swaps '
        '`gridcell` for `rowheader` on a `rowHeaderColumnKeys` column; '
        '`aria-rowcount`/`aria-colcount`/`aria-rowindex` are set only '
        '`if (isVirtualized)` and `aria-colindex` always. `aria-sort` and '
        '`aria-multiselectable` have no gpui builder, the `rowgroup` around '
        'the single header row has no counterpart element, and a tree row\'s '
        '`aria-posinset`/`aria-setsize` are deliberately not guessed: the '
        'row builder sees a flattened visible list, not the sibling count '
        'upstream reads off the tree collection.',
    ('select.rs', 'Select'):
        'Role::Button plus `aria-expanded` on the trigger, Role::ListBox on '
        'the popover list, Role::ListBoxOption plus `aria-selected` on each '
        'row. `react-aria/dist/private/select/useSelect.mjs` derives the '
        'trigger from `useMenuTrigger({type: \'listbox\'})`, so '
        '`.../overlays/useOverlayTrigger.mjs` gives it '
        '`\'aria-haspopup\': \'listbox\'`, `\'aria-expanded\': isOpen` and '
        '`\'aria-controls\'`, and `@heroui/react`\'s `select.js` renders it '
        'as an RAC `Button`. The list is RAC `ListBox`, i.e. `useListBox`. '
        '`aria-haspopup` and `aria-controls` have no gpui builder.',
    ('combo_box.rs', 'ComboBox'):
        'Role::Group on the field box, Role::Button plus `aria-expanded` on '
        'the chevron trigger, Role::ListBox on the popup and '
        'Role::ListBoxOption plus `aria-selected` on each row, with '
        '`aria-activedescendant` on the highlighted one. '
        '`@heroui/react`\'s `combo-box.js` composes RAC `Group` '
        '(`Group.mjs`: `role: props.role ?? \'group\'`), an RAC `Button` and '
        'RAC `ComboBox`\'s `ListBox`. What is *not* here is '
        '`useComboBox`\'s `role: \'combobox\'`: it goes on the text input, '
        'and this component composes a `crate::input::Input` whose role its '
        'own render decides — see the picker note in '
        '`crates/herogpui-components/src/a11y.rs`.',
    ('autocomplete.rs', 'Autocomplete'):
        'Role::Button plus `aria-expanded` on the trigger, Role::Button '
        'named `Clear selection` on the clear control, Role::ListBox on the '
        'popover list and Role::ListBoxOption plus `aria-selected` on each '
        'row, with `aria-activedescendant` on the highlighted one. v3\'s '
        'Autocomplete is a Select whose popover holds a search field: '
        '`@heroui/react`\'s `autocomplete.js` imports `Select, '
        'SelectStateContext, SelectValue` from '
        '`react-aria-components/Select`, not `ComboBox`, and hard-codes '
        '`"aria-label": "Clear selection"` on the clear button. The search '
        'field inside the popover is a `crate::input::SearchField`, so the '
        '`aria-autocomplete`/`aria-activedescendant` half of '
        '`useAutocomplete`\'s input contract is not this component\'s to '
        'state; the highlighted row states the descendant half instead.',

    ('toast.rs', 'ToastCardEl'):
        'Role::AlertDialog on the card. `toast/toast.js` renders RAC\'s '
        '`UNSTABLE_Toast`, whose props come from `react-aria/.../toast/'
        'useToast.js`: `role: \'alertdialog\'`, `aria-modal: \'false\'`, named '
        'by the title and described by the description. The `aria-modal` '
        'half and the inner `role: \'alert\'` + `aria-atomic` content node '
        'are recorded omissions in `a11y.rs`; the second exists only to make '
        'a live announcement, and gpui exposes no live-region builder.',
}


# --------------------------------------------------------------------------
# (module, struct) -> (the struct that carries the node, why)
#
# A wrapper that renders another component of this port and nothing else. It
# must not set a role: doing so would either duplicate the node or shadow it.
# --------------------------------------------------------------------------
DELEGATES = {
    ('input.rs', 'TextField'):
        ('Input', 'self.inner.render',
         '`TextField::render` is `self.inner.render(..)` over a held `Input`. '
         'Upstream is the same shape: `textfield/textfield.js` composes RAC '
         '`TextField` around the same input.'),
    ('input.rs', 'SearchField'):
        ('Input', 'Input::new',
         'builds an `Input` with `InputType::Search`, which is the '
         '`<input type="search">` `useSearchField.js` renders (its `type` '
         'defaults to `\'search\'`).'),
    ('textarea.rs', 'TextArea'):
        ('Input', 'self.inner',
         'wraps a multi-line `Input`, which reports the multiline text role. '
         'The wrapper only supplies the height `rows` asks for.'),
    ('meter.rs', 'Meter'):
        ('ProgressBar', 'ProgressBar::new',
         '`useMeter.js` is `useProgressBar` with the role changed and nothing '
         'else, so the port delegates the whole rendering and passes the role '
         'down through `ProgressBar::as_meter`.'),
    ('dropdown.rs', 'Dropdown'):
        ('Menu', 'Menu::new',
         'a `Dropdown` is a trigger plus a `Menu`, and both nodes belong to '
         'somebody else: `Menu` carries `role="menu"` and its rows, and the '
         'trigger is whatever element the caller passed in — usually a '
         '`Button`, which reports its own `role="button"`. Upstream\'s '
         '`useMenuTrigger` adds `aria-expanded` / `aria-haspopup` / '
         '`aria-controls` to that same caller button through React context; '
         'gpui cannot inject props into an element it was handed, and a '
         'second node wrapped around the caller\'s button would report the '
         'trigger twice.'),
}


# --------------------------------------------------------------------------
# (module, struct) -> why upstream reports no accessibility node here either.
#
# Not "we skipped it": each of these renders something that is presentational
# or text upstream, so a node here would be an invention rather than parity.
# --------------------------------------------------------------------------
NO_NODE = {
    ('toggle_button.rs', 'ToggleSeparator'):
        'v3 renders the group separator with `aria-hidden` '
        '(`toggle-button-group/toggle-button-group.js`); it is a 1px rule '
        'between two members, and announcing it would put a stop between them.',
    ('switch.rs', 'SwitchGroup'):
        '`switch-group/switch-group.js` renders two plain `<div>`s and no RAC '
        'primitive at all — unlike `CheckboxGroup`, it is layout only, with no '
        'grouping semantics upstream to port.',
    ('input_group.rs', 'InputAddon'):
        'the prefix and suffix slots are plain `<div>`s in '
        '`input-group/input-group.js`; the text they hold is decoration beside '
        'the field, and the field carries its own name.',
    ('field.rs', 'Label'):
        'a `<label>` is not an accessibility node of its own — it *names* one. '
        'gpui has no id-reference graph, so the text reaches the control '
        'through `a11y::Name::field`, which is where `useField`\'s '
        '`aria-labelledby` lands in this port.',
    ('field.rs', 'Description'):
        'the description text is announced as part of the control\'s '
        'description, not as a node: `useField.js` folds its id into the '
        'control\'s `aria-describedby`, which `a11y::Name::field` inlines.',
    ('field.rs', 'ErrorMessage'):
        'same as `Description` — `useField.js` folds the error id into the '
        'control\'s `aria-describedby`.',
    ('field.rs', 'FieldError'):
        'the visibility-managing wrapper around `ErrorMessage`; it renders an '
        'empty div when the field is valid and has nothing of its own to '
        'announce.',
    ('field.rs', 'FieldsetLegend'):
        'a `<legend>` names its `<fieldset>` rather than being a node. gpui '
        'has no ancestor context, so the legend text stays visual: a named '
        '`Fieldset` reports `role="group"` without inlining this caption.',
    ('field.rs', 'FieldGroup'):
        '`Fieldset.Group` is `w-full space-y-4` and nothing else — a layout '
        'stack with no upstream role.',
    ('field.rs', 'FieldsetActions'):
        '`Fieldset.Actions` is `gap-2 pt-1` — the trailing button row\'s '
        'layout, with no upstream role.',
    ('chip.rs', 'Chip'):
        '`@heroui/react/dist/components/chip/chip.js` imports no '
        '`react-aria-components` primitive at all: it is a plain '
        '`dom.span` with a `data-slot="chip"` and no `role` or `aria-*` '
        'anywhere in the file. A chip is decoration beside content, and the '
        'content it decorates carries its own node.',
    ('chip.rs', 'ChipLabel'):
        'the `.chip__label` span inside that same plain `dom.span` — text, '
        'not a node.',
    ('disclosure.rs', 'DisclosureGroup'):
        'RAC\'s `DisclosureGroup` (`Disclosure.mjs`) renders a bare '
        '`<div>` with `filterDOMProps` and a `data-disabled` attribute and '
        'sets no role — unlike `DisclosurePanel` two functions below it, '
        'which spells `role: role = \'group\'` out. The group is expansion '
        'policy, not a landmark; each `Disclosure` it renders carries its '
        'own node.',
    # ---- wave 4: status and content -------------------------------------
    #
    # This is the largest single finding of the wave, and it is a finding
    # rather than a gap: two thirds of v3's display surface imports no
    # `react-aria-components` primitive at all, authors no `role` and no
    # `aria-*`, and renders a `dom.div` or a `dom.span` with a `data-slot`.
    # A node here would be an invention. Each row names the file that was
    # read and the tags it renders, so the claim is checkable rather than a
    # shrug.
    ('alert.rs', 'Alert'):
        '`alert/alert.js` imports no RAC primitive — only its own '
        '`SurfaceContext` — and `AlertRoot`, `AlertIndicator`, '
        '`AlertContent`, `AlertTitle` and `AlertDescription` render '
        '`dom.div` / `dom.span` / `dom.p` with a `data-slot` and nothing '
        'else. The `role="alert"` a reader half expects here is a different '
        'component: `alert-dialog/alert-dialog.js` sets '
        '`role: "alertdialog"`, and that one is under contract from wave 2. '
        'v3 also removed `isClosable`/`onClose`, so the close affordance is '
        'a composed `CloseButton` carrying its own node.',
    ('avatar.rs', 'Avatar'):
        '`avatar/avatar.js` composes `@radix-ui/react-avatar`, not a RAC '
        'primitive, and neither the HeroUI wrapper nor the Radix package '
        'writes a `role` or any `aria-*` anywhere. An avatar is a picture '
        'beside the name it belongs to, and the name is the node. v3 ships '
        'no `AvatarGroup`, so there is no group contract to port either.',
    ('badge.rs', 'BadgeAnchor'):
        '`badge/badge.js` renders every one of its parts as a plain '
        '`dom.span` with a `data-slot` and no `role` or `aria-*`; the anchor '
        'is the positioning box the badge is pinned to, and the thing it is '
        'pinned to carries its own node.',
    ('badge.rs', 'Badge'):
        'the same `dom.span` in the same file. A badge is a count or a dot '
        'decorating a control, the way `Chip` decorates content — see that '
        'row for the identical shape.',
    ('badge.rs', 'BadgeLabel'):
        'the label span inside that same plain `dom.span` — text, not a node.',
    ('card.rs', 'Card'):
        '`card/card.js` imports no RAC primitive; `CardRoot` is a `dom.div` '
        'with a `data-slot`, no `role` and no `aria-*`. A card is a box '
        'around content that already reports itself.',
    ('card.rs', 'CardHeader'):
        '`CardHeader` is a `dom.div` in the same file — layout above the '
        'body, with no upstream role.',
    ('card.rs', 'CardTitle'):
        '`CardTitle` is a `dom.h3` in the same file, and that native tag is '
        'the *only* semantic it carries: `card.js` authors no `role` and no '
        '`aria-*`, and there is no hook behind it to read one out of. '
        'Claiming `Role::Heading` here would be reasoning from the HTML-AAM '
        'mapping table rather than from the pinned source, which '
        '`docs/agents/parity.md` rules out; nothing else in this port '
        'reports a heading, so the card title would be the only tagged text '
        'in a window full of untagged text. `CardTitle::new()` also takes no '
        'id, so it could not carry a node today in any case.',
    ('card.rs', 'CardDescription'):
        '`CardDescription` is a `dom.p` in the same file, with no authored '
        'role or `aria-*` — same reading as `CardTitle` above.',
    ('card.rs', 'CardContent'):
        '`CardContent` is a `dom.div` in the same file: the body slot.',
    ('card.rs', 'CardFooter'):
        '`CardFooter` is a `dom.div` in the same file: the trailing action '
        'row\'s layout, and the actions in it are `Button`s with their own '
        'nodes.',
    ('form.rs', 'Form'):
        '`form/form.js` is one line of delegation to RAC `Form`, and '
        '`react-aria-components/dist/private/Form.mjs` renders '
        '`dom.form` with `noValidate: validationBehavior !== \'native\'` and '
        'the filtered DOM props — it calls no hook and sets no `role` and no '
        '`aria-*` at all. Neither file gives the form an accessible name, so '
        'there is nothing for a landmark to be named by; and `Form::new()` '
        'takes no id, so no node could be produced here regardless. The '
        'validation contract the port does carry reaches the tree through '
        'each field\'s own `a11y::Name::field`.',
    ('kbd.rs', 'Kbd'):
        '`kbd/kbd.js` imports no RAC primitive: `KbdRoot` is a `dom.kbd`, '
        '`KbdContent` a `dom.span`, and `KbdAbbr` a `dom.abbr` that sets '
        '`title` — not `aria-label`. No `role` and no `aria-*` anywhere in '
        'the file. gpui also has no `title`-tooltip equivalent, which is why '
        '`.kbd__abbr` has no analogue in the port.',
    ('scrollbar.rs', 'Scrollbar'):
        'HeroUI styles the browser\'s native scrollbar with `--scrollbar`; '
        'there is no RAC Scrollbar component and no role. This overlay is '
        'the GPUI paint stand-in for that token.',
    ('scroll_shadow.rs', 'ScrollShadow'):
        '`scroll-shadow/scroll-shadow.js` renders a plain `"div"` with '
        '`data-orientation`, `data-scroll-shadow-size`, `data-slot` and a '
        'style, and its `use-scroll-shadow.js` hook is scroll geometry with '
        'no ARIA in it. The fades are paint over content that reports '
        'itself.',
    ('skeleton.rs', 'Skeleton'):
        '`skeleton/skeleton.js` renders a `dom.div` with a `data-slot` and '
        'no `role` or `aria-*`. A placeholder announces nothing on purpose: '
        'the content it stands in for is not there yet.',
    ('surface.rs', 'Surface'):
        '`surface/surface.js` is a `dom.div` with a `data-slot` — a styling '
        'wrapper that `alert.js` and `card.js` consume through '
        '`SurfaceContext` to flip on-surface contrast. No `role`, no '
        '`aria-*`.',
    ('typography.rs', 'Typography'):
        '`typography/typography.js` delegates to RAC `Text` with '
        '`elementType: defaultElementByType[type]` (`body`/`body-sm`/'
        '`body-xs` -> `p`, `code` -> `code`, `h1`..`h6` -> `h1`..`h6`), and '
        '`react-aria-components/dist/private/Text.mjs` is '
        '`let { elementType = \'span\', ...domProps } = props` followed by a '
        '`createElement` — no hook call, no `role`, no `aria-*`. The tag is '
        'the whole of it, and reading a role off the tag would be reasoning '
        'from HTML-AAM rather than from the pinned source. See '
        '`card.rs`\'s `CardTitle` for the same decision stated at length.',
    ('typography.rs', 'Prose'):
        '`Prose` is a separate export in that same file and a plain `"div"` '
        'with no `elementType`, no `role` and no `aria-*`: it is the '
        'long-form text rhythm, applied to whatever is composed inside it.',
}


# --------------------------------------------------------------------------
# (module, struct) -> what a later wave has to do to flip it on.
#
# Every entry here is a real gap: upstream reports a node and the port does
# not. The remaining row is AccessKit 0.24 having no `Role::Separator`, not
# a missing caller id. Do not invent `Role::Splitter` to clear it.
# --------------------------------------------------------------------------
PENDING = {
    ('separator.rs', 'Separator'):
        'upstream is RAC `Separator` (`separator/separator.js` wrapping '
        '`react-aria-components/dist/private/Separator.mjs`), which defaults '
        'to `<hr>` — HTML-AAM\'s implicit separator — and switches to a '
        '`div` with `useSeparator`\'s `role: \'separator\'` plus '
        '`aria-orientation` when the orientation is vertical. The port now '
        'takes an optional `.id()`, but AccessKit 0.24 has no '
        '`Role::Separator` (`Role::Splitter` is a pane splitter, not a '
        'rule). Claiming `Splitter` would be a lie; the id is there so a '
        'later AccessKit bump can flip this row without a public-API change.',
}


# --------------------------------------------------------------------------
# module -> the wave that will bring it under contract.
#
# Modules with no `impl RenderOnce` at all are not listed: there is nothing to
# expose. Everything that renders and is not in this wave is here, so a module
# cannot escape the audit by never being mentioned.
# --------------------------------------------------------------------------
PENDING_MODULES = {
}

# The modules this wave owns. Derived, so the two halves cannot drift: a module
# is in the wave exactly when one of the four tables names it.
def wave_modules():
    modules = set()
    for table in (EXPOSES_A_ROLE, DELEGATES, NO_NODE, PENDING):
        modules.update(module for module, _struct in table)
    return modules


# Floors. A scanner that reads nothing finds no gaps, which is
# indistinguishable from a tree with no gaps.
MODULE_FLOOR = 50
IMPL_FLOOR = 50
ROLE_CALL_FLOOR = 80

# The port's own wrappers. `a11y` is the role; the rest are the states.
ROLE_CALL = re.compile(r'\.a11y(?:_named)?\(')
STATE_CALL = re.compile(
    r'\.a11y_(?:name|checked|pressed|expanded|range|orientation|text'
    r'|selected|set_position|level|grid_size|row_index|column_index'
    r'|active_descendant)\(')

# gpui's own accessibility builders. Allowed only in the foundation module.
#
# `.role(` is deliberately *not* matched bare: `ActiveTheme::role(Color::..)`
# and `ThemeBuilder::role(&str, ..)` are colour roles and share the spelling,
# which is the reason the port wraps gpui's `role` as `a11y` in the first
# place. Matched here only when the argument is an AccessKit role.
RAW_CALLS = (
    re.compile(r'\.aria_[a-z_]+\('),
    re.compile(r'\.on_a11y_action\('),
    re.compile(r'\.a11y_synthetic_children\('),
    re.compile(r'\.role\(\s*(?:accesskit::|gpui::accesskit::)?Role::'),
    re.compile(r'\.role\(\s*accesskit::'),
)

IMPL_HEADER = re.compile(r'^impl RenderOnce for ([A-Za-z0-9_]+)\b', re.M)


def strip_comments(source):
    """`source` with comments blanked, string literals intact.

    Blanked rather than deleted so every offset still maps to its line. This
    file's own prose is full of `.aria_label(` and `role="group"`, and so are
    the components' comments — none of that is code.
    """
    out = []
    index = 0
    length = len(source)
    while index < length:
        char = source[index]
        if char == '"':
            out.append(char)
            index += 1
            while index < length:
                out.append(source[index])
                if source[index] == '\\':
                    if index + 1 < length:
                        out.append(source[index + 1])
                        index += 2
                        continue
                elif source[index] == '"':
                    index += 1
                    break
                index += 1
            continue
        if source.startswith('//', index):
            while index < length and source[index] != '\n':
                out.append(' ')
                index += 1
            continue
        if source.startswith('/*', index):
            while index < length and not source.startswith('*/', index):
                out.append('\n' if source[index] == '\n' else ' ')
                index += 1
            out.append('  ')
            index += 2
            continue
        out.append(char)
        index += 1
    return ''.join(out)


def strip_test_modules(source):
    """`source` with top-level `#[cfg(test)]` blocks blanked out.

    A test may name whatever role it likes; it is not the component's contract.
    Found by column, which holds for rustfmt-formatted source.
    """
    kept = []
    inside = False
    for line in source.split('\n'):
        if inside:
            kept.append('')
            if line == '}':
                inside = False
            continue
        if line.startswith('#[cfg(test)]'):
            inside = True
            kept.append('')
            continue
        kept.append(line)
    return '\n'.join(kept)


def readable(source):
    return strip_test_modules(strip_comments(source))


def impl_blocks(body):
    """`{struct: block_source}` for every `impl RenderOnce for X` in `body`.

    Anchored on the `impl` header and closed by brace matching from the first
    `{` after it, as `docs/agents/parity.md` requires: a fixed character window
    would read into the next component.
    """
    blocks = {}
    for match in IMPL_HEADER.finditer(body):
        name = match.group(1)
        start = body.find('{', match.end())
        if start == -1:
            continue
        depth = 0
        index = start
        while index < len(body):
            if body[index] == '{':
                depth += 1
            elif body[index] == '}':
                depth -= 1
                if depth == 0:
                    break
            index += 1
        blocks[name] = body[start:index + 1]
    return blocks


def module_files():
    """Every component module, by bare file name (`foo.rs` or directory `foo/`)."""
    return list_modules(SRC)


def audit(sources, whole_tree=True):
    """`(failures, counts)` for `{module: source}`.

    `sources` is passed in rather than read here so the self-test can drive the
    same code over a synthetic tree; `whole_tree=False` then suppresses the two
    checks that are only meaningful over the real tree — a table row is only
    stale when *no* module defines its component, which a one-file sample
    cannot decide.
    """
    failures = []
    counts = {'modules': 0, 'impls': 0, 'role_calls': 0, 'raw': 0}
    seen = set()

    covered = wave_modules()

    for module in sorted(sources):
        source = sources[module]
        body = readable(source)
        counts['modules'] += 1
        counts['role_calls'] += len(ROLE_CALL.findall(body))

        if module != FOUNDATION:
            for pattern in RAW_CALLS:
                for match in pattern.finditer(body):
                    counts['raw'] += 1
                    failures.append((
                        'RAW', module, match.group(0).strip(),
                        'gpui\'s own accessibility builders belong in '
                        '`%s`, where each one is checked against the pinned '
                        'React Aria hook that decides it. Call the `a11y_*` '
                        'wrapper instead.' % FOUNDATION))

        blocks = impl_blocks(body)
        counts['impls'] += len(blocks)
        if not blocks:
            continue

        in_wave = module in covered
        if not in_wave:
            if module not in PENDING_MODULES:
                failures.append((
                    'UNCLASSIFIED', module, ', '.join(sorted(blocks)),
                    'this module renders components and appears in no table: '
                    'not in %s, and not in PENDING_MODULES either. Add it to '
                    'the wave, or record which wave will reach it.' % WAVE))
            continue

        for name, block in sorted(blocks.items()):
            key = (module, name)
            tables = [t for t in (EXPOSES_A_ROLE, DELEGATES, NO_NODE, PENDING)
                      if key in t]
            if len(tables) != 1:
                failures.append((
                    'UNCLASSIFIED', module, name,
                    'a component in %s must appear in exactly one of '
                    'EXPOSES_A_ROLE / DELEGATES / NO_NODE / PENDING; it '
                    'appears in %d.' % (WAVE, len(tables))))
                continue
            seen.add(key)
            table = tables[0]
            has_role = bool(ROLE_CALL.search(block))
            has_state = bool(STATE_CALL.search(block))

            if table is EXPOSES_A_ROLE:
                if not has_role:
                    failures.append((
                        'NO ROLE', module, name,
                        'expected %s, but `impl RenderOnce for %s` calls '
                        'neither `a11y` nor `a11y_named`. Without a role gpui '
                        'omits the element from the tree entirely.'
                        % (EXPOSES_A_ROLE[key], name)))
            elif table is DELEGATES:
                target, evidence, why = DELEGATES[key]
                if has_role:
                    failures.append((
                        'DOUBLE ROLE', module, name,
                        'recorded as delegating to `%s` (%s), but it sets a '
                        'role of its own. Either the delegation ended — move '
                        'it to EXPOSES_A_ROLE — or the node is now reported '
                        'twice.' % (target, why)))
                elif evidence not in block or target not in body:
                    # Two halves, because a wrapper reaches its delegate
                    # through a held field: the render must still show the
                    # handover (`evidence`), and the module must still name
                    # the type that carries the node (`target`).
                    failures.append((
                        'LOST DELEGATE', module, name,
                        'recorded as delegating to `%s` through `%s`, which '
                        'its render no longer does. Nothing carries its '
                        'accessibility node.' % (target, evidence)))
            else:
                label = 'NO_NODE' if table is NO_NODE else 'PENDING'
                if has_role:
                    failures.append((
                        'STALE %s' % label, module, name,
                        'it now sets a role, so the recorded reason (%s) is '
                        'out of date. Move it to EXPOSES_A_ROLE with the '
                        'upstream hook that decides the role.' % table[key]))

            if has_state and not has_role:
                failures.append((
                    'STATE WITHOUT ROLE', module, name,
                    'sets accessibility state (`a11y_checked`, `a11y_range`, '
                    '...) on an element with no role. gpui reports no node for '
                    'a roleless element, so the state is dead code.'))

    if not whole_tree:
        failures.sort(key=lambda f: (f[1], f[2], f[0]))
        return failures, counts

    for table_name, table in (('EXPOSES_A_ROLE', EXPOSES_A_ROLE),
                              ('DELEGATES', DELEGATES),
                              ('NO_NODE', NO_NODE),
                              ('PENDING', PENDING)):
        for key in sorted(set(table) - seen):
            failures.append((
                'STALE ENTRY', key[0], key[1],
                '`%s` still has a row for `%s`, but no `impl RenderOnce for '
                '%s` exists there any more. Delete the row.'
                % (table_name, key[1], key[1])))

    overlap = sorted(wave_modules() & set(PENDING_MODULES))
    for module in overlap:
        failures.append((
            'DOUBLE WAVE', module, '',
            'listed in PENDING_MODULES and also covered by %s. A module is in '
            'one wave or the other.' % WAVE))

    failures.sort(key=lambda f: (f[1], f[2], f[0]))
    return failures, counts


def read_sources():
    return {name: read_module(name, SRC, errors='replace') for name in module_files()}


def main():
    if not os.path.isdir(SRC):
        raise SystemExit(
            'A11Y SCAN READ NOTHING: %s does not exist, so its verdict is '
            'meaningless.' % SRC)

    sources = read_sources()
    failures, counts = audit(sources)

    for verdict, module, name, advice in failures:
        print('%-19s %s%s' % (verdict, module, ('  ' + name) if name else ''))
        print('%s%s' % (' ' * 20, advice))
    if failures:
        print()

    print('component modules read    : %d' % counts['modules'])
    print('RenderOnce impls read     : %d' % counts['impls'])
    print('role calls found          : %d' % counts['role_calls'])
    print('components with a role    : %d' % len(EXPOSES_A_ROLE))
    print('delegating to another     : %d' % len(DELEGATES))
    print('no node upstream either   : %d' % len(NO_NODE))
    print('pending components        : %d' % len(PENDING))
    print('pending modules           : %d' % len(PENDING_MODULES))
    print('raw gpui a11y calls       : %d' % counts['raw'])
    print('failures                  : %d' % len(failures))

    if counts['modules'] < MODULE_FLOOR:
        raise SystemExit(
            '\nA11Y SCAN READ NOTHING: found only %d modules under %s, '
            'expected at least %d.' % (counts['modules'], SRC, MODULE_FLOOR))
    if counts['impls'] < IMPL_FLOOR:
        raise SystemExit(
            '\nA11Y SCAN READ NOTHING: parsed only %d `impl RenderOnce` '
            'blocks, expected at least %d. The block parser is broken, and '
            '"every component has its role" means nothing.'
            % (counts['impls'], IMPL_FLOOR))
    if counts['role_calls'] < ROLE_CALL_FLOOR:
        raise SystemExit(
            '\nA11Y CONTRACT LOST: only %d `a11y` / `a11y_named` calls remain '
            'in %s, expected at least %d. Deleting the roles satisfies every '
            'other rule here and defeats the point.'
            % (counts['role_calls'], SRC, ROLE_CALL_FLOOR))

    return 1 if failures else 0


# --------------------------------------------------------------------------
# Self-test
# --------------------------------------------------------------------------
SAMPLE_GOOD = '''
use crate::a11y::{self, A11y as _};

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A comment mentioning .aria_label( and .role(Role::Button) is prose.
        div()
            .id(self.id.clone())
            .a11y_named(a11y::Role::Button, &a11y::Name::none())
            .child("ok")
    }
}
'''

SAMPLE_MISSING = '''
impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().id(self.id.clone()).child("ok")
    }
}
'''

SAMPLE_RAW = '''
impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id.clone())
            .a11y(a11y::Role::Button)
            .aria_label("Close")
    }
}
'''

SAMPLE_STATE_ONLY = '''
impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().id(self.id.clone()).a11y_checked(true, false)
    }
}
'''

SAMPLE_THEME_ROLE = '''
impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(Color::Accent);
        div()
            .id(self.id.clone())
            .a11y(a11y::Role::Button)
            .bg(sem.color)
    }
}
'''


def self_test():
    """Known-positive and known-negative proof for the scanner."""
    problems = []

    def expect(condition, message):
        if not condition:
            problems.append(message)

    def sample(sources):
        return audit(sources, whole_tree=False)

    def verdicts(sources):
        return sorted({f[0] for f in sample(sources)[0]})

    good = {'button.rs': SAMPLE_GOOD}

    # --- the happy path is silent, and the parser really read it -----------
    failures, counts = sample(good)
    expect(failures == [],
           'a component that states its role should raise nothing: %r'
           % (failures,))
    expect(counts['impls'] == 1 and counts['role_calls'] == 1,
           'the block parser and the role counter should each see exactly one '
           'thing in the sample: %r' % (counts,))

    # --- a missing role is the failure this audit exists for ---------------
    expect(verdicts({'button.rs': SAMPLE_MISSING}) == ['NO ROLE'],
           'a component in EXPOSES_A_ROLE with no `a11y` call must fail with '
           'NO ROLE: %r' % (verdicts({'button.rs': SAMPLE_MISSING}),))

    # --- raw gpui builders outside the foundation module -------------------
    expect(verdicts({'button.rs': SAMPLE_RAW}) == ['RAW'],
           'a raw `.aria_label(..)` outside %s must fail: %r'
           % (FOUNDATION, verdicts({'button.rs': SAMPLE_RAW})))
    expect(sample({FOUNDATION: '.aria_label("x").on_a11y_action(a, b)'})[0]
           == [],
           'the foundation module is where the raw builders live; it must not '
           'flag itself')

    # --- state with no role is dead code -----------------------------------
    expect(verdicts({'button.rs': SAMPLE_STATE_ONLY})
           == ['NO ROLE', 'STATE WITHOUT ROLE'],
           'accessibility state on a roleless element must fail: %r'
           % (verdicts({'button.rs': SAMPLE_STATE_ONLY}),))

    # --- the colour-role false friend must not be mistaken for a11y --------
    expect(sample({'button.rs': SAMPLE_THEME_ROLE})[0] == [],
           '`cx.role(Color::Accent)` is a colour role and must not read as a '
           'raw accessibility builder — the whole reason gpui\'s `role` is '
           'wrapped as `a11y`: %r' % (audit({'button.rs': SAMPLE_THEME_ROLE})[0],))

    # --- the four tables are exclusive and complete ------------------------
    invented = {'button.rs': SAMPLE_GOOD.replace('for Button', 'for Invented')}
    expect(verdicts(invented) == ['UNCLASSIFIED'],
           'a component in no table must be reported unclassified: %r'
           % (verdicts(invented),))
    # And over the whole tree, the row it left behind is reported stale.
    expect([f[0] for f in audit(invented)[0] if f[1] == 'button.rs'
            and f[2] == 'Button'] == ['STALE ENTRY'],
           'the EXPOSES_A_ROLE row for a component that no longer exists must '
           'be reported stale')
    expect('UNCLASSIFIED' in verdicts({'invented.rs': SAMPLE_MISSING}),
           'a rendering module in neither the wave nor PENDING_MODULES must '
           'be reported')
    # No rendering module remains out of contract, so the out-of-scope rule
    # is checked by injecting a temporary pending row.
    PENDING_MODULES['__pending__.rs'] = 'none'
    expect(sample({'__pending__.rs': SAMPLE_MISSING})[0] == [],
           'a module recorded in PENDING_MODULES is out of scope for this '
           'wave and must not be flagged')
    del PENDING_MODULES['__pending__.rs']

    # --- a stale NO_NODE / PENDING reason ----------------------------------
    stale_pending = {'separator.rs':
                     SAMPLE_GOOD.replace('for Button', 'for Separator')}
    expect(verdicts(stale_pending) == ['STALE PENDING'],
           'a PENDING component that has since gained a role must be reported '
           'so the entry gets deleted: %r' % (verdicts(stale_pending),))
    stale_no_node = {'switch.rs':
                     SAMPLE_GOOD.replace('for Button', 'for SwitchGroup')}
    expect(verdicts(stale_no_node) == ['STALE NO_NODE'],
           'a NO_NODE component that has since gained a role must be reported: '
           '%r' % (verdicts(stale_no_node),))

    # --- delegation is checked in both directions --------------------------
    delegating = {'meter.rs':
                  'impl RenderOnce for Meter {\n'
                  '    fn render(self) -> impl IntoElement {\n'
                  '        ProgressBar::new(self.id).as_meter()\n    }\n}\n'}
    expect(sample(delegating)[0] == [],
           'a real delegation must pass: %r' % (sample(delegating)[0],))
    lost = {'meter.rs':
            'impl RenderOnce for Meter {\n'
            '    fn render(self) -> impl IntoElement {\n'
            '        div().child("nothing")\n    }\n}\n'}
    expect(verdicts(lost) == ['LOST DELEGATE'],
           'a delegate that vanished must be reported: %r' % (verdicts(lost),))
    doubled = {'meter.rs':
               SAMPLE_GOOD.replace('for Button', 'for Meter')}
    expect(verdicts(doubled) == ['DOUBLE ROLE'],
           'a delegating wrapper that also sets a role must be reported: %r'
           % (verdicts(doubled),))

    # --- brace matching, not a character window ----------------------------
    two = ('impl RenderOnce for A {\n  fn render(self) { if x { y } }\n}\n'
           'impl RenderOnce for B {\n  fn render(self) { }\n}\n')
    parsed = impl_blocks(two)
    expect(sorted(parsed) == ['A', 'B'],
           'both impl blocks should be found: %r' % (sorted(parsed),))
    expect('for B' not in parsed['A'],
           'block A must stop at its own closing brace, not run into B')

    # --- comments and strings are not code ---------------------------------
    expect(readable('a\n// b\nc').split('\n')[2] == 'c',
           'blanking a comment must not move the lines after it')
    expect(readable('#[cfg(test)]\nmod t {\n .aria_label("x")\n}\n')
           .strip() == '',
           'a top-level test module must be blanked out')

    # --- floors and empties ------------------------------------------------
    empty_failures, empty_counts = sample({})
    expect(empty_failures == [] and empty_counts['impls'] == 0,
           'scanning nothing finds nothing — which is exactly why the floors '
           'exist')
    expect(MODULE_FLOOR > 0 and IMPL_FLOOR > 0 and ROLE_CALL_FLOOR > 0,
           'a floor of zero is not a floor')

    # --- every table row still describes a real component ------------------
    live = audit(read_sources())[0] if os.path.isdir(SRC) else []
    expect(not [f for f in live if f[0] == 'STALE ENTRY'],
           'a table row that matches no component in the tree has gone stale: '
           '%r' % ([f for f in live if f[0] == 'STALE ENTRY'],))

    if problems:
        print('self-test FAIL')
        for problem in problems:
            print('- %s' % problem)
        return 1
    print('self-test PASS: the scanner fails a component that states no role, '
          'a raw gpui accessibility builder outside %s, accessibility state '
          'on a roleless element, a component in no table, a module in no '
          'wave, a stale NO_NODE/PENDING reason, a lost or doubled delegate '
          'and a stale table row; it matches `impl` blocks by brace, reads '
          'past comments and test modules, and does not mistake the theme\'s '
          '`cx.role(Color::..)` for an accessibility role.' % FOUNDATION)
    return 0


if __name__ == '__main__':
    if '--self-test' in sys.argv[1:]:
        sys.exit(self_test())
    if self_test() != 0:
        sys.exit(1)
    print()
    sys.exit(main())
