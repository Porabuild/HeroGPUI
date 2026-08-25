"""Diff every v3 documented prop against our builder methods.

Extracts the `### <Component>` prop tables from the HeroUI v3 agent bundle and
compares them with the `pub fn` builders we expose, so leftovers and omissions
show up as a list instead of by accident.
"""
import io
import re
import sys
import glob

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

import os

# Download once:
#   curl -sL -o heroui-full.txt https://heroui.com/react/llms-full.txt
BUNDLE = os.environ.get(
    'HEROUI_BUNDLE',
    os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt'),
)
SRC = 'crates/herogpui-components/src/'

# React prop -> our builder name, where they legitimately differ.
ALIAS = {
    # A field's children-as-a-function: `content` is that closure, handed
    # `{isFocused, isFocusWithin, isFocusVisible}`.
    'NumberField.isFocused': 'content', 'NumberField.isFocusVisible': 'content',
    'NumberField.isFocusWithin': 'content',
    'ColorField.isFocused': 'content', 'ColorField.isFocusVisible': 'content',
    'ColorField.isFocusWithin': 'content',
    'DateField.isFocused': 'content', 'DateField.isFocusVisible': 'content',
    'DateField.isFocusWithin': 'content',
    'TimeField.isFocused': 'content', 'TimeField.isFocusVisible': 'content',
    'TimeField.isFocusWithin': 'content',
    'TextField.isFocused': 'content', 'TextField.isFocusVisible': 'content',
    'TextField.isFocusWithin': 'content',
    'SearchField.isFocused': 'content', 'SearchField.isFocusVisible': 'content',
    'SearchField.isFocusWithin': 'content',
    # The function half of v3's `action` union: what is handed the form data.
    'Form.action': 'on_submit',
    # v3 hands a button's children a function with the interactive state in it;
    # `content` is that closure, so each render prop resolves to it.
    'RadioGroup.isSelected': 'option_content',
    # v3 documents the per-option props under a `### Radio` heading on the
    # RadioGroup page, and `FOLD_STRUCTS` answers its rows with the real
    # option struct: `RadioOption` owns `value` and `is_disabled`, so those
    # rows resolve against the option type and never the root's same-named
    # builders (the root's `value`/`is_disabled` cover the whole group). One
    # row still projects onto the group on purpose:
    #
    # * `Radio.name` is the option's submission name; every radio in one HTML
    #   group submits under the same name, which is the one `form_field`
    #   reads, so the per-option row is the group's `name` taken once.
    'RadioGroup.Radio.name': 'name',
    'ListBox.isFocused': 'item_content', 'ListBox.isPressed': 'item_content',
    'Dropdown.isDisabled': 'item_content', 'Dropdown.isFocused': 'item_content',
    'Dropdown.isPressed': 'item_content',
    'TagGroup.isHovered': 'tag_content', 'TagGroup.isPressed': 'tag_content',
    'TagGroup.isFocused': 'tag_content', 'TagGroup.isFocusVisible': 'tag_content',
    'TagGroup.isSelected': 'tag_content',
    'Switch.isHovered': 'content', 'Switch.isPressed': 'content',
    'Switch.isFocused': 'content', 'Switch.isFocusVisible': 'content',
    'ToggleButton.isHovered': 'content', 'ToggleButton.isPressed': 'content',
    'ToggleButton.isFocused': 'content', 'ToggleButton.isFocusVisible': 'content',
    'CloseButton.isHovered': 'content', 'CloseButton.isPressed': 'content',
    'CloseButton.isFocused': 'content',
    'Button.isHovered': 'content', 'Button.isPressed': 'content',
    'Button.isFocused': 'content', 'Button.isFocusVisible': 'content',
    # `Calendar.Cell`'s render function is handed `formattedDate` and the flags;
    # `cell` is that closure, so each of its render props resolves to it.
    'RangeCalendar.formattedDate': 'cell', 'RangeCalendar.isSelected': 'cell',
    'RangeCalendar.isUnavailable': 'cell', 'RangeCalendar.isOutsideMonth': 'cell',
    'RangeCalendar.isSelectionStart': 'cell', 'RangeCalendar.isSelectionEnd': 'cell',
    'Calendar.formattedDate': 'cell', 'Calendar.isSelected': 'cell',
    'Calendar.isUnavailable': 'cell', 'Calendar.isOutsideMonth': 'cell',
    # Calendar's value/defaultValue/onChange are unions: multiple mode carries
    # a date array, so the plural builders are the ones that prove the full
    # documented value type rather than only matching the prop name.
    'Calendar.value': 'values', 'Calendar.defaultValue': 'default_values',
    'Calendar.onChange': 'on_change_all',
    # Slider's value/defaultValue are number | number[]; the plural builder
    # proves the multi-thumb form rather than only the scalar.
    'Slider.defaultValue': 'default_values',
    # `Calendar.Cell Render Props` documents `isDisabled` ("whether the cell is
    # disabled") as a value handed *into* the cell render function, and
    # `CalendarCellState::is_disabled` (same for `RangeCalendarCellState`) is
    # exactly that: the port computes the per-day state in order to draw the
    # cell dimmed, and hands it over to the `cell` closure. The root's
    # `is_disabled` disables the whole calendar, which is a different prop, so
    # the part-scoped key is what keeps the two rows apart -- the root table
    # also documents `isDisabled`, and a bare `Calendar.isDisabled` alias would
    # answer for both.
    'Calendar.Cell.isDisabled': 'cell',
    'RangeCalendar.Cell.isDisabled': 'cell',
    # `ColorSlider.Output`'s render function is handed the `color`.
    'ColorSlider.color': 'output',
    # `ProgressBar.ValueLabel` (and the Meter's and the circle's) is a render
    # function handed `percentage` and `valueText`; `value_content` is that
    # closure, so both props resolve to it.
    'Meter.percentage': 'value_content', 'Meter.valueText': 'value_content',
    'ProgressBar.percentage': 'value_content', 'ProgressBar.valueText': 'value_content',
    'ProgressCircle.percentage': 'value_content',
    'ProgressCircle.valueText': 'value_content',
    'onPress': 'on_press', 'onChange': 'on_change', 'onSelectionChange': 'on_selection_change',
    'onOpenChange': 'on_open_change', 'onAction': 'on_action', 'onRemove': 'on_remove',
    'onSubmit': 'on_submit', 'onClose': 'on_close', 'onConfirm': 'on_confirm',
    'onCancel': 'on_cancel', 'onClear': 'on_clear', 'onToggle': 'on_toggle',
    'onSortChange': 'on_sort_change', 'onFocusChange': 'on_focus_change',
    # v3's Slider callback is `(number | number[])`; the scalar builder answers
    # the one-thumb form and this scoped alias proves the array form exists too.
    'Slider.onChangeEnd': 'on_change_end_all',
    'onChangeEnd': 'on_change_end', 'onVisibilityChange': 'on_visibility_change',
    'isDisabled': 'is_disabled', 'isReadOnly': 'is_read_only', 'isRequired': 'is_required',
    'isInvalid': 'is_invalid', 'isSelected': 'is_selected', 'isPending': 'is_pending',
    'isIconOnly': 'is_icon_only', 'isOpen': 'is_open', 'isIndeterminate': 'is_indeterminate',
    'isAttached': 'is_attached', 'isEnabled': 'is_enabled', 'isClearable': 'is_clearable',
    'isDismissable': 'is_dismissible', 'isExternal': 'is_external', 'isStriped': 'is_striped',
    'isBordered': 'is_bordered', 'isBlurred': 'is_blurred', 'isHoverable': 'is_hoverable',
    'isPressable': 'is_pressable', 'isDestructive': 'is_destructive',
    'fullWidth': 'full_width', 'maxVisibleToasts': 'max_visible_toasts',
    'hideSeparator': 'hide_separator', 'hideSteppers': 'hide_steppers',
    'selectionMode': 'selection_mode', 'selectedKeys': 'selected_keys',
    # `defaultSelectedKeys` is deliberately NOT mapped to `selected_keys`: one
    # seeds an uncontrolled control, the other is the controlled value a caller
    # drives. The honest resolution is the snake spelling, and every collection
    # has the real seed builder now: `ToggleButtonGroup::default_selected_keys`
    # first, then `ListBox::default_selected_keys`,
    # `TagGroup::default_selected_keys` and `Dropdown::default_selected_keys`,
    # each seeding keyed uncontrolled state behind `util::controlled` (the
    # controlled-empty distinction is the `is_controlled` flag). The blanket
    # `caller-owns-collection` reason died because a sibling implemented the
    # very thing it claimed impossible, and a blanket reason contradicted by a
    # row it covers cannot stand.
    'disabledKeys': 'disabled_keys',
    'selectedKey': 'selected_key', 'errorMessage': 'error_message',
    'colorSpace': 'color_space', 'xChannel': 'x_channel', 'yChannel': 'y_channel',
    'showDots': 'show_dots', 'colorName': 'color_name', 'hourCycle': 'hour_cycle',
    'placeholderValue': 'placeholder_value', 'minValue': 'min_value', 'maxValue': 'max_value',
    'allowsCustomValue': 'allows_custom_value', 'menuTrigger': 'menu_trigger',
    'hideScrollBar': 'hide_scroll_bar', 'showControls': 'show_controls',
    'emptyState': 'empty_state', 'scaleFactor': 'scale_factor', 'htmlFor': 'label_for',
    'showValueLabel': 'show_value_label', 'weeksInMonth': 'weeks_in_month',
    'firstDayOfWeek': 'first_day_of_week', 'isDateUnavailable': 'is_date_unavailable',
    'focusedValue': 'focused_value',
    # Tabs is the one component with a separate uncontrolled builder.
    # `defaultSelectedKey` elsewhere (ComboBox) is seeded through the caller's
    # `InputState` (`WONT_PORT`'s `state-entity-seeds-it`), not by an alias
    # onto the controlled `selected_key`.
    'Tabs.defaultSelectedKey': 'default_selected_key',
    'renderEmptyState': 'empty_state', 'onInputChange': 'on_input_change',
    'defaultFilter': 'filter', 'autoFocus': 'auto_focus',
    # `Dropdown.ItemIndicator`'s type is our `indicator` builder.
    'Dropdown.type': 'indicator',
    # v3 documents the plain attribute spellings alongside the is* ones.
    'disabled': 'is_disabled', 'readOnly': 'is_read_only', 'required': 'is_required',
    'inputValue': 'input_value', 'shouldFlip': 'should_flip',
    # Avatar.Image's error callback and Avatar.Fallback's show-delay, ported
    # with a custom image loader that observes the load.
    'Avatar.onError': 'on_error',
    'Avatar.delayMs': 'delay_ms',
    'onLoadMore': 'on_load_more', 'isLoading': 'is_pending',
    'sortDescriptor': 'sort_descriptor', 'allowsSorting': 'allows_sorting',
    'isRowHeader': 'is_row_header', 'showIndicator': 'show_indicator',
    'onAction': 'on_action', 'isKeyboardDismissDisabled': 'is_keyboard_dismiss_disabled',
    # The backdrop overlay style is documented as `variant` on the Backdrop
    # part; we spell it `backdrop` on the parent.
    'Modal.variant': 'backdrop',
    'Drawer.variant': 'backdrop',
    'AlertDialog.variant': 'backdrop',
    # Dropdown.Item's "default | danger" variant is our `danger` flag.
    'Dropdown.variant': 'danger',
    # Pagination.Link's press handler is the group's page-change callback.
    'Pagination.onPress': 'on_change',
    # `Pagination.Link.isDisabled` (and the `Previous / Next` halves of it)
    # is per-link; the port's expression of that is the keyed set
    # `disabled_keys`, the same shape `Accordion.Item.isDisabled` already
    # takes: keys 1..total disable the page links, `0` Previous and
    # `total + 1` Next. The root `is_disabled` disables the whole bar, which
    # is a different prop, so only the part-scoped key answers the row.
    'Pagination.Link.isDisabled': 'disabled_keys',
    # `Slider.Thumb.isDisabled` and `Slider.Thumb.name` are per-thumb: one
    # thumb of a range stays put while the others move, and `name` is the
    # name of one thumb's `<input>`. The root's `is_disabled`/`name` -- the
    # whole slider, and its single-name+value submission -- are different
    # props, so the same-named root builders cannot answer either row. The
    # port is monolithic where v3 composes, so both project onto the root by
    # thumb index, the addressing `Slider::values` uses: `disabled_keys`
    # names the immovable thumbs (the pointer's nearest-thumb choice skips
    # them, the arrows and the slider's own Tab cycle skip them, they leave
    # the tab order, and their field is not submitted), and the per-thumb
    # input names are `thumb_names`, with `start_name`/`end_name` retaining
    # the `DateRangePicker` compatibility convention.
    'Slider.Thumb.isDisabled': 'disabled_keys',
    'Slider.Thumb.name': 'thumb_names',
    # Accordion.Trigger's extra press handler, and Accordion.Item's controlled
    # expansion, are expressed on the group.
    'Accordion.onPress': 'on_toggle',
    'Accordion.isExpanded': 'expanded_keys',
    # Accordion.Item's per-item state is expressed on the group, each under a
    # name that names the group-level facility: `disabled_keys` (the render
    # disables an item whose key is in the set), `default_expanded` (the
    # source documents it as "`defaultExpanded` on a single item"), and
    # `on_toggle`, which reports the key of the item that moved -- the
    # per-item half of `onExpandedChange`, whose group half reports the whole
    # set (`Accordion.onExpandedChange`). `isDisabled` on the Trigger is the
    # item's disabled state: v3's trigger is the item's header, and the port
    # draws both from the item.
    'Accordion.Item.isDisabled': 'disabled_keys',
    'Accordion.Trigger.isDisabled': 'disabled_keys',
    'Accordion.Item.defaultExpanded': 'default_expanded',
    'Accordion.Item.onExpandedChange': 'on_toggle',
    # Autocomplete.ClearButton's click handler.
    'Autocomplete.onClick': 'on_clear',
    'ColorSwatchPicker.variant': 'shape',
    # v3 hands these *into* a child render function. The builder that takes the
    # closure is what implements the prop, so the alias points at it: a caller
    # receives the value rather than supplying it.
    'Dropdown.ItemIndicator.isSelected': 'indicator_content',
    'Dropdown.isSelected': 'item_content',
    'InputOTP.index': 'slot',
    'Pagination.isActive': 'link',
    'Table.sortDirection': 'indicator',
    'Slider.index': 'thumb',
    'DateField.segment': 'segment',
    'ListBox.isSelected': 'indicator',
    'Select.selectedItems': 'value_content',
    'Select.isPlaceholder': 'value_content',
    'Select.defaultChildren': 'value_content',
    # `Autocomplete.Value` and `ComboBox.Value` take the same closure, and it is
    # handed `util::SelectionValue` -- v3's four value render props as one value.
    'Autocomplete.selectedItems': 'value_content',
    'Autocomplete.selectedText': 'value_content',
    'Autocomplete.isPlaceholder': 'value_content',
    'Autocomplete.defaultChildren': 'value_content',
    'ComboBox.selectedItem': 'value_content',
    # `getThumbValueLabel` formats one thumb's value; the thumb closure is handed
    # the index and the value, so the caller formats it there.
    'Slider.getThumbValueLabel': 'thumb',
    # Taken positionally or by the state's constructor, so the prop exists --
    # it is just not spelled as a builder.
    'TagGroup.items': 'tags',
    'Table.items': 'row',
    'InputOTP.maxLength': 'with_length',
    # v3 calls these `type`; `type` is a Rust keyword.
    'Input.type': 'input_type',
    'InputGroup.type': 'input_type',
    'Typography.type': 'kind',
    # v3's toast option is spelled `isLoading` and means a spinner; the global
    # `isLoading` alias is the v2 rename to `isPending`, which is a different
    # prop.
    'Toast.isLoading': 'is_loading',
    # `actionProps` is `{children, onPress}` -- a label and a handler.
    'Toast.actionProps': 'action',
}

# Props with no meaningful gpui analogue at all.
SKIP = {
    'className', 'children', 'render', 'style', 'nativeProps', 'id', 'slot', 'ref',
    'asChild', 'key', 'queue', 'srcSet', 'sizes', 'alt',
    'aria-label', 'aria-labelledby', 'aria-describedby', 'aria-details', 'textValue',
    'containerClassName',
}

# Deliberately not ported, with the reason. These are reported separately so
# the "real gap" number stays meaningful.
WONT_PORT = {
    # HeroUI documents this value, but v3.2.4 passes the raw
    # react-aria-components 1.20.0 MenuItemRenderProps to the indicator. That
    # state has no `isIndeterminate` member, so the value is always absent.
    'Dropdown.isIndeterminate': 'pinned-source-does-not-emit',
    # Uncontrolled mode exists (`util::controlled` + `use_keyed_state`), so
    # `defaultOpen`, `defaultSelected`, `defaultExpanded*` and
    # `defaultYearPickerOpen` are implemented, not omitted. The collection-
    # valued seed `defaultSelectedKeys` is implemented too, so it needs no
    # reason here: the old blanket `caller-owns-collection` was disproved by
    # `ToggleButtonGroup`, which implements the same shape as
    # `default_selected_keys`, and a blanket reason contradicted by a row it
    # covers cannot stand -- the siblings followed, and `ListBox`, `TagGroup`
    # and `Dropdown` each seed keyed uncontrolled state behind
    # `util::controlled` now, so the trio is answered rather than reported.
    # Tabs has a real `default_selected_key`; ComboBox's single selection lives
    # in the caller's `InputState`, which is its own uncontrolled seed
    # (`InputState::with_value`).
    'ComboBox.defaultSelectedKey': 'state-entity-seeds-it',
   
    # `ListLayout`/`TableLayout` describe a virtualizer told its geometry in
    # advance. gpui has two: `uniform_list` takes one height and gives it to every
    # row (`rowHeight`), and `list` measures each row it builds -- which is what
    # `estimatedRowHeight` selects, on both components. What is left is about the
    # rows this port does not have rather than about the virtualizer:
    #
    # * A section header here is one line of text, so there is no *variable*
    #   heading height to estimate.
    # * `Table` has no section rows at all -- its groups are expandable rows --
    #   so `headingHeight` has nothing to size; `ListBox` accepts it.
    # * `Table`'s load-more row takes `loaderHeight`; `ListBox` has no loader.
    'estimatedHeadingHeight': 'single-line-headings',
    'Table.headingHeight': 'no-section-rows',
    'ListBox.loaderHeight': 'no-loader-row',
    # The thickness of a drop indicator, on a layout whose drag-and-drop v3
    # itself does not expose: `dragAndDropHooks`, `onReorder` and every other
    # drag prop React Aria's list takes are documented nowhere in v3, so nothing
    # a caller can set could produce an indicator to give a thickness to. Same
    # shape as `state_audit.py`'s `no-disabled-prop` -- a styling knob for a
    # feature the surrounding API does not offer -- and building the drag
    # ourselves would add props v3 does not document, which `extra_audit.py`
    # would then report.
    'dropIndicatorThickness': 'no-drag-source-prop',

    # A `ToastQueue` exists because React state lives outside React. A gpui
    # `Entity` is observable by construction -- `cx.observe(&store, ..)` is the
    # subscription -- so there is no method on the store to add.
    # `wrapUpdate` wraps a queue mutation in a CSS view transition. gpui has no
    # view transitions: a change is drawn on the next frame.
    'Toast.wrapUpdate': 'no-view-transitions',

    # A calendar cell is drawn here, so the values its render function would
    # receive -- the formatted date, whether the day falls outside the month,
    # is selected, is unavailable, starts or ends the range -- are computed and
    # used rather than handed over.
    # Likewise the value label: the component computes the percentage and
    # formats the value (`format_options` chooses how), and `value_label`
    # replaces the text outright.
    # React Aria passes the rendering the component *would* have done, so a
    # render function can fall back to it. A builder that replaces the slot has
    # nothing to hand over: the default is the thing being replaced.

    # An Autocomplete's or ComboBox's text and selection live in the caller's
    # `InputState` and `selected_keys`; the caller owns both, so there is
    # nothing to give back.
    # An HTML5 ValidityState object.
    'validationDetails': 'no-html-forms',
    # The `form` attribute names the HTML form a control submits to. A `Form` is
    # told its fields here (`Form::field`), so there is no id to point at.
    'form': 'no-html-forms',
    # A ComboBox item *is* its text: the list is `Vec<SharedString>`, so an
    # item's key and its label are the same value and there is nothing for
    # `formValue` to choose between.
    'ComboBox.formValue': 'keys-are-the-text',
    # There are no time zones in this port: `Time` is a wall clock, so there is
    # no abbreviation to hide.
    'hideTimeZone': 'no-time-zones',

    # Choosing separators, digit systems and currency placement per locale needs
    # CLDR data; a partial table would be worse than not offering the prop.
    # `formatOptions` itself is implemented -- see `core/src/format.rs`.
    'locale': 'no-intl',
    # The year-picker heading `format` is `DateFormatterOptions`, i.e.
    # `Intl.DateTimeFormatOptions`. Its two rows document `{month: 'short'}`
    # and `{year: 'numeric'}`, but the type also carries `era`, a `calendar`
    # identifier and the state's timeZone, and honoring those needs the same
    # locale-aware `Intl` data as `locale` above -- CLDR data this port does
    # not carry. It can emit a fixed ISO date (`Date::format_iso`) but not
    # honor the options object.
    'Calendar.format': 'no-intl',
    'RangeCalendar.format': 'no-intl',
    # There is no browser to navigate or post to. `action` is the exception:
    # v3's type is `string | FormHTMLAttributes['action']`, and the function half
    # of that union -- the one handed the form data -- is `on_submit`, so it is
    # an alias rather than an omission.
    'method': 'no-http', 'encType': 'no-http',
    'target': 'no-http', 'download': 'no-http', 'rel': 'no-http',
    # A hint for the browser's autofill, which there is none of here.
    'autoComplete': 'no-browser-autofill',
    # ARIA roles have no accessibility layer to reach.
    'role': 'no-a11y-attrs',
    # Browser-only input affordances.
 'inputMode': 'no-soft-keyboard',
   

    # Sub-component/table-parsing artefacts, not real props of ours.
    'state': 'not-a-prop', 'toast': 'not-a-prop',
    # `TextArea` wraps (gpui's default `WhiteSpace::Normal`), so there is
    # multi-line layout -- but no `pre`/`pre-wrap` mode to select between, which
    # is what `wrap` chooses.
    'TextArea.wrap': 'single-wrap-mode',
    # v3 documents exactly one value for these, so there is nothing to select
    # and a builder taking a one-variant enum could not change anything.
    'CloseButton.variant': 'single-valued',
    # "automatically set to 'search'" -- a search field has one input type.
    'SearchField.type': 'single-valued',
    'ScrollShadow.variant': 'single-valued',
    # gpui draws no native scrollbar in a scroll container, so there is none to
    # hide.
    'ScrollShadow.hideScrollBar': 'no-native-scrollbar',
    # An accessible name with no accessibility layer to expose it to.
    'ColorSwatch.colorName': 'no-a11y-attrs',
    # Taken as a constructor argument rather than a builder, because these
    # components are meaningless without it: `ColorArea::new(id, value)`,
    # `ColorSlider::new(id, value, channel)`, `ColorField::new(id, value)`,
    # `ColorPicker::new(id, value)`.
    'ColorArea.value': 'constructor-arg',
    'ColorField.value': 'constructor-arg',
    'ColorPicker.value': 'constructor-arg',
    'ColorSlider.value': 'constructor-arg',
    'ColorSlider.channel': 'constructor-arg',
    # Values v3 passes *into* a child render function. A monolithic builder
    # computes them internally, so there is no prop to accept.
    # gpui exposes no accessibility title attribute.
    'Kbd.title': 'no-a11y-attrs',
    # Browser image-loading attributes with no gpui analogue.
    'Avatar.crossOrigin': 'no-browser-image-attrs',
    'Avatar.loading': 'no-browser-image-attrs',
    # The success half of the image-events pair. The custom loader this port
    # uses can observe a successful load, but the port wires only the failure
    # side (`on_error`) and the fallback -- the parts v3's own examples drive
    # (`delayMs` on a deliberately broken URL); the failure reason was
    # recorded as `no-image-load-events` for `delayMs` before either half
    # existed.
    'Avatar.onLoad': 'no-image-load-events',
}

# Which module(s) implement each documented component.
# Structs that implement part of one component's composition. v3 documents
# these as `### Component.Part` tables, so their builders count toward the
# parent. Only genuine parts belong here -- a neighbouring *component* in the
# same module (CheckboxGroup beside Checkbox, DatePicker beside DateField) does
# not, or a gap on one would be hidden by the other.
COMPANIONS = {
    'Breadcrumbs': ['Crumb'],
    'Toast': ['ToastViewport', 'ToastStore'],
    'ListBox': ['ListBoxItem'],
    'CheckboxGroup': ['CheckboxOption'],
    'Dropdown': ['Menu', 'MenuItem'],
    'Accordion': ['AccordionItem'],
    'Input': ['InputState'],
    'TextField': ['InputState'],
    'SearchField': ['InputState'],
    'TextArea': ['InputState'],
    'NumberField': ['NumberState'],
    'InputOTP': ['OtpState'],
    'Table': ['TableRow', 'TableColumn'],
    # The v3 part is `Tabs.Tab`; the struct is `TabItem`. The old entry named
    # a `Tab` no source file defines, and a companion that does not exist
    # contributes an empty method set -- exactly how `Radio.isDisabled` hid
    # behind a phantom `RadioOption`, and how `Tabs.Tab.isDisabled` hid
    # behind the whole-list `Tabs::is_disabled`.
    'Tabs': ['TabItem'],
    # Not here: `ToggleButton`. ToggleButton is v3's own page (see `FILES`)
    # and a *child* of the group, not a split of it. Its builders used to
    # answer the group's root table -- `ToggleButtonGroup.size` was "green"
    # through `ToggleButton::size`. A companion with its own v3 table answers
    # its own rows only; the group has its own `size` now, inherited by the
    # children that do not set one, so the row is satisfied by the group.
    'Calendar': ['CalendarState'],
    'RangeCalendar': ['DateRangeState'],
    'DatePicker': ['CalendarState'],
    'DateRangePicker': ['DateRangeState'],
    # `SelectOption` was another phantom: select.rs defines only `Select`, and
    # its options are plain `SharedString`s, so there is no per-option struct
    # whose builders could implement a row. (Select.Popover is the only part
    # table with props, and `placement` lives on `Select` itself.)
    'Switch': ['SwitchGroup'],
    # `useFilter` is a hook, so its options and its three returned matchers are
    # one value here: `Filter::new(sensitivity).contains(..)`.
    'Autocomplete': ['Filter'],
    'ComboBox': ['Filter'],
    # Not here: `Input` / `TextArea` for `InputGroup`, nor `ColorSwatch` for
    # `ColorSwatchPicker` -- each is v3's own page, so each answers its own
    # rows, not the container's. The rows v3 composes onto them are preserved
    # where the docs put them: `### Composition Components` on the InputGroup
    # page is `FOLD_STRUCTS` below, and `### ColorSwatchPicker.Item` is the
    # `PART_STRUCTS` entry that still names `ColorSwatch`.
    'TagGroup': ['Tag'],
}

# Which structs answer for each `### Comp.Part` table. A prop documented on a
# part belongs to that part: it must be satisfiable by a builder on one of the
# listed structs, and *never* by a same-named builder on the component root --
# `Tabs::is_disabled` disables the whole list, `Tabs.Tab.isDisabled` disables
# one tab, and only `TabItem` can answer for it. The parts with no entry here
# hold only `className`/`children`-shaped rows, which `SKIP` already covers.
# A part table with real props and no entry is reported rather than folded
# into the root's set, because that silent fallback is the hole this table
# closes.
#
# The two empty entries are no-owners answered by the part-scoped ALIAS rows
# above rather than by a struct. `Slider.Thumb`'s `isDisabled` and `name` are
# per-thumb: the port projects them onto the root by thumb index --
# `disabled_keys` (the immovable thumbs), and the range's named ends
# `start_name`/`end_name`, with `name` for the single-thumb form -- while the
# root's `is_disabled`/`name` cover the whole slider, so a same-named root
# builder must not answer them and no per-thumb struct exists. The same goes
# for `Pagination.Link` and `Pagination.Previous / Pagination.Next`:
# `isDisabled` there is per-link/per-button, expressed as the keyed set
# `disabled_keys`, while the root flag disables the whole bar.
#
# The key matches the v3 heading (`Comp.Part`), with the ` Render Props`
# suffix dropped: the heading `### Calendar.Cell Render Props` is the part
# `Calendar.Cell`.
PART_STRUCTS = {
    'Accordion.Item': ['AccordionItem'],
    'Accordion.Trigger': ['AccordionItem'],
    'AlertDialog.Backdrop': ['AlertDialog'],
    'AlertDialog.Container': ['AlertDialog'],
    'AlertDialog.Dialog': ['AlertDialog'],
    'AlertDialog.Icon': ['AlertDialog'],
    'Autocomplete.ClearButton': ['Autocomplete'],
    'Autocomplete.Filter': ['Autocomplete'],
    'Autocomplete.Popover': ['Autocomplete'],
    'Avatar.Fallback': ['Avatar'],
    'Avatar.Image': ['Avatar'],
    'Breadcrumbs.Item': ['Crumb'],
    'Calendar.Cell': ['Calendar'],
    # The Year Picker parts are drawn by the monolithic calendar, and the root
    # builders that answer them can only mean the part's prop: `visible_years`
    # is the year-picker grid's window size and nothing else, so the part rows
    # `### Year Picker Parts` documents resolve against the root struct -- the
    # same composition projection `Table.LoadMore` uses above.
    'Calendar.YearPickerGrid': ['Calendar'],
    'Calendar.YearPickerTriggerHeading': ['Calendar'],
    'ColorField.Group': ['ColorField'],
    'ColorField.Input': ['ColorField'],
    'ColorPicker.Popover': ['ColorPicker'],
    'ColorSwatchPicker.Item': ['ColorSwatch'],
    'ComboBox.Popover': ['ComboBox'],
    'ComboBox.Value': ['ComboBox'],
    'DateField.Group': ['DateField'],
    'DateField.Input': ['DateField'],
    'DateField.Segment': ['DateField'],
    'Drawer.Backdrop': ['Drawer'],
    'Drawer.Content': ['Drawer'],
    'Drawer.Dialog': ['Drawer'],
    'Dropdown.Item': ['MenuItem'],
    'Dropdown.ItemIndicator': ['Menu'],
    'Dropdown.Menu': ['Menu'],
    'Dropdown.Popover': ['Dropdown'],
    'Dropdown.Section': ['Menu'],
    'InputOTP.Slot': ['InputOTP'],
    'Kbd.Abbr': ['Kbd'],
    'ListBox.Item': ['ListBoxItem'],
    'Modal.Backdrop': ['Modal'],
    'Modal.Container': ['Modal'],
    'Modal.Dialog': ['Modal'],
    'Pagination.Link': [],
    'Pagination.Previous / Pagination.Next': [],
    'Popover.Content': ['Popover'],
    'RangeCalendar.Cell': ['RangeCalendar'],
    # The RangeCalendar halves of the same year-picker parts: drawn by the
    # monolithic root, whose builders can only be the parts' props.
    'RangeCalendar.YearPickerGrid': ['RangeCalendar'],
    'RangeCalendar.YearPickerTriggerHeading': ['RangeCalendar'],
    'Select.Popover': ['Select'],
    'Slider.Thumb': [],
    'Table.Body': ['Table'],
    'Table.Collection': ['Table'],
    'Table.Column': ['TableColumn'],
    'Table.Content': ['Table'],
    'Table.Header': ['Table'],
    'Table.LoadMore': ['Table'],
    'Table.SortableColumnHeader': ['Table'],
    'Tabs.Tab': ['TabItem'],
    'TagGroup.List': ['TagGroup'],
    'TimeField.Group': ['TimeField'],
    'TimeField.Input': ['TimeField'],
    'TimeField.Segment': ['TimeField'],
    'Toast.Indicator': ['Toast'],
    'Toast.Provider': ['ToastViewport'],
    'Tooltip.Content': ['Tooltip'],
}

# Folded headings -- tables on a component's page that are neither the
# component (`### Comp`) nor one of its parts (`### Comp.Part`) -- carry the
# rows v3 documents for something else. `props_for_state` separates them from
# the root table so the rows resolve against the struct that implements them
# rather than against the component by default:
#
# * `### ToastQueue` / `### toast Function`, `### SwitchGroup`, `### useFilter
#   Hook`, `### Render Props`, `### Radio.*`, `### ListLayout` /
#   `### TableLayout` and `### Tag.RemoveButton` are implemented on the
#   component itself or on a same-component companion (`ToastStore`,
#   `SwitchGroup`, `Filter`), which the component's own answer set already
#   contains -- no entry needed;
# * `### Composition Components` documents the *composed* v3 component
#   (`InputGroup.Input`, `SearchField.Input`) by pointing at React Aria's
#   Input, and the port composes the real `Input`/`TextArea` -- so the rows a
#   group documents for the field it wraps resolve against those structs, the
#   same composition `PART_STRUCTS` records for `Comp.Part` tables.
#
# The key is `(component, heading)`; `PART_STRUCTS` stays the table for
# `Comp.Part` headings, so the two mechanisms describe one rule: a row is
# answered by the structs its own heading names.
FOLD_STRUCTS = {
    ('InputGroup', 'Composition Components'): ['Input', 'TextArea'],
    # Per-item tables of a monolithic component. `### Radio` documents the
    # per-option props of a group that now has a real option type:
    # `RadioOption` owns `value` and `is_disabled`, so its rows resolve
    # against that struct and never against the root's same-named builders
    # (the root's `value`/`is_disabled` cover the whole group); the one row
    # the option struct cannot answer, `name`, keeps its fold-scoped alias
    # in `ALIAS`. `### Tag` is the per-item half of `TagGroup`, and there the
    # per-tag struct itself exists, so `Tag` answers its own rows.
    ('RadioGroup', 'Radio'): ['RadioOption'],
    ('TagGroup', 'Tag'): ['Tag'],
}

FILES = {
    'Button': 'button.rs', 'ButtonGroup': 'button_group.rs', 'CloseButton': 'close_button.rs',
    'ToggleButton': 'toggle_button.rs', 'ToggleButtonGroup': 'toggle_button.rs',
    'Dropdown': 'dropdown.rs', 'ListBox': 'list_box.rs', 'TagGroup': 'tag_group.rs',
    'ColorArea': 'color_picker.rs', 'ColorField': 'color_picker.rs',
    'ColorPicker': 'color_picker.rs', 'ColorSlider': 'color_picker.rs',
    'ColorSwatch': 'color_picker.rs', 'ColorSwatchPicker': 'color_picker.rs',
    'Slider': 'slider.rs', 'Switch': 'switch.rs', 'Badge': 'badge.rs', 'Chip': 'chip.rs',
    'Table': 'table.rs', 'Calendar': 'calendar.rs', 'DateField': 'date_picker.rs',
    'DatePicker': 'date_picker.rs', 'DateRangePicker': 'date_picker.rs',
    'RangeCalendar': 'range_calendar.rs', 'TimeField': 'time_field.rs',
    'Alert': 'alert.rs', 'Meter': 'meter.rs', 'ProgressBar': 'progress.rs',
    'ProgressCircle': 'progress.rs', 'Skeleton': 'skeleton.rs', 'Spinner': 'spinner.rs',
    'Checkbox': 'checkbox.rs', 'CheckboxGroup': 'checkbox.rs', 'Fieldset': 'field.rs',
    'Label': 'field.rs', 'Description': 'field.rs', 'ErrorMessage': 'field.rs',
    'FieldError': 'field.rs', 'Form': 'form.rs', 'Input': 'input.rs',
    'InputGroup': 'input_group.rs', 'InputOTP': 'input_otp.rs',
    'NumberField': 'number_field.rs', 'RadioGroup': 'radio_group.rs',
    'SearchField': 'input.rs', 'TextArea': 'textarea.rs', 'TextField': 'input.rs',
    'Card': 'card.rs', 'Separator': 'separator.rs', 'Surface': 'surface.rs',
    'Toolbar': 'toolbar.rs', 'Avatar': 'avatar.rs', 'Accordion': 'accordion.rs',
    'Breadcrumbs': 'breadcrumbs.rs', 'Disclosure': 'disclosure.rs', 'Link': 'link.rs',
    'Pagination': 'pagination.rs', 'Tabs': 'tabs.rs', 'AlertDialog': 'alert_dialog.rs',
    'Drawer': 'drawer.rs', 'Modal': 'modal.rs', 'Popover': 'popover.rs',
    'Toast': 'toast.rs', 'Tooltip': 'tooltip.rs', 'Autocomplete': 'autocomplete.rs',
    'ComboBox': 'combo_box.rs', 'Select': 'select.rs', 'Kbd': 'kbd.rs',
    'Typography': 'typography.rs', 'ScrollShadow': 'scroll_shadow.rs',
}

bundle = io.open(BUNDLE, encoding='utf-8').read()

# Our builder methods, per file.
# Builders are attributed per `impl <Struct>` block, not per file. Several
# components share a module (ColorField and ColorPicker both live in
# color_picker.rs), and a file-level set let one component's prop count as
# another's.
methods = {}
impl_methods = {}
# Props each struct's constructor takes positionally.
constructor_args = {}
for path in glob.glob(SRC + '*.rs'):
    name = path.replace('\\', '/').split('/')[-1]
    src = io.open(path, encoding='utf-8').read()
    methods[name] = set(re.findall(r'pub fn ([a-z_0-9]+)\s*\(', src))
    # Inherent impls only: `impl Foo {`, never `impl Trait for Foo {`.
    for block in re.split(r'^impl\b', src, flags=re.M)[1:]:
        head, _, body = block.partition('{')
        head = head.strip()
        if ' for ' in head:
            continue
        m = re.match(r'\s*(?:<[^>]*>\s*)?([A-Za-z_][A-Za-z0-9_]*)', head)
        if not m:
            continue
        struct_name = m.group(1)
        impl_methods.setdefault(struct_name, set()).update(
            re.findall(r'pub fn ([a-z_0-9]+)\s*\(', body)
        )
        # A prop the constructor takes positionally is implemented, not missing:
        # `ColorArea::new(id, value)` *is* `value`, and `Table::new(columns)` is
        # `columns`. Reading only the builders counted fourteen of these as
        # omissions and filed them under `constructor-arg`.
        for ctor in re.finditer(r'pub fn new\s*\(([^)]*)\)', body):
            for param in ctor.group(1).split(','):
                name = param.split(':')[0].strip().lstrip('&').strip()
                if re.fullmatch(r'[a-z_][a-z_0-9]*', name or ''):
                    constructor_args.setdefault(struct_name, set()).add(name)
# Every struct a table names must be one the sources define -- including the
# fold owners, where a phantom would hand the fold's rows back to the root's
# same-named builders. A phantom contributes an empty method set, so the rows
# it answers for can never fail: `Radio.isDisabled` hid behind a nonexistent
# `RadioOption` in `COMPANIONS`, and `Tabs.Tab.isDisabled` hid behind the
# whole-list `Tabs::is_disabled` once the real struct had been misspelled
# `Tab`. Fail loudly rather than audit a name that is not there.
_phantom = sorted(({
    s for v in COMPANIONS.values() for s in v
} | {
    s for v in PART_STRUCTS.values() for s in v
} | {
    s for v in FOLD_STRUCTS.values() for s in v
}) - set(impl_methods))
if _phantom:
    raise SystemExit(
        'PHANTOM STRUCT IN COMPANIONS/PART_STRUCTS/FOLD_STRUCTS: %s -- every '
        'name must resolve to a real struct' % ', '.join(_phantom))

API_SECTIONS = None


def api_sections():
    """Each component page's `## API Reference` section, verbatim.

    One per page, 71 of them, and the section is the boundary that matters: the
    v2 migration guides on the same page carry tables of their own
    (`### 2. Prop Changes`) whose rows are props v3 *removed*.
    """
    global API_SECTIONS
    if API_SECTIONS is None:
        API_SECTIONS = []
        for m in re.finditer(r'^[ 	]*## API Reference[ 	]*$', bundle, re.M):
            chunk = bundle[m.end():]
            nxt = re.search(r'^[ 	]*## ', chunk, re.M)
            API_SECTIONS.append(chunk[:nxt.start()] if nxt else chunk)
    return API_SECTIONS


def props_for(component):
    """Prop names documented for a component.

    Read the page's whole API Reference section, because **not every table there
    is named after the component**. Matching `### <Comp>` and `### <Comp>.<Part>`
    -- which is what this did -- missed `### ListLayout` and `### TableLayout`
    (the virtualization props), `### Tag` and `### Tag.RemoveButton` on the
    TagGroup page, `### SwitchGroup`, `### Radio.*`, `### Disclosure{Trigger,
    Content}`, `### Composition Components` on the three field pages,
    `### ToastQueue` and `### toast Function`, and `### useFilter Hook`: 139
    documented rows that were never checked against anything.

    The section is the unit rather than the heading because a heading pattern
    cannot tell `### ListLayout` from `### 2. Prop Changes`. Every component
    resolves to exactly one section, which is asserted rather than assumed.
    """
    anchor = r'^[ 	]*### %s(?:\.[A-Za-z]+)?[ 	]*$' % re.escape(component)
    owners = [s for s in api_sections() if re.search(anchor, s, re.M)]
    if len(owners) != 1:
        # Two pages documenting one name would silently merge their tables and
        # invent gaps in both; no page at all means the anchor stopped matching.
        print('API SECTION AMBIGUOUS: %s matched %d sections' % (component, len(owners)))
        return set()
    return prop_rows(owners[0])


def props_for_state(component):
    """`(root props, {part: props}, {heading: props})` for `component`.

    `props_for` reads the whole `## API Reference` section and folds every
    table into one set, which is what other audits want. This splits the same
    section per `###` heading so a row documented on `### Comp.Part` can be
    answered by the *part's* structs and not by a same-named builder on the
    root. The union of the three halves is exactly `props_for`'s set, and the
    section matching is the same assertion (`Comp` or `Comp.Part` resolved to
    exactly one section).

    A heading that is neither the component nor one of its parts (`ListLayout`,
    `SwitchGroup`, `Composition Components`, `ToastQueue`, `toast Function`,
    `useFilter Hook`, `Render Props`) carries rows v3 documents for something
    else -- a composed part or a sibling table -- so each is kept under its own
    heading in the third half. `main` resolves those rows against
    `FOLD_STRUCTS` (or the component's own answer set), the same way
    `PART_STRUCTS` governs tables named `Comp.Part`. The fold is where the
    old code turned `### Radio`'s and `### SwitchGroup`'s rows into root rows,
    which is the laundering the split prevents.
    """
    anchor = r'^[ 	]*### %s(?:\.[A-Za-z]+)?[ 	]*$' % re.escape(component)
    owners = [s for s in api_sections() if re.search(anchor, s, re.M)]
    if len(owners) != 1:
        print('API SECTION AMBIGUOUS: %s matched %d sections' % (component, len(owners)))
        return None
    root = set()
    parts = {}
    folds = {}
    heads = [(m.end(), m.group(1).strip())
             for m in re.finditer(r'^[ 	]*### (.+?)[ 	]*$', owners[0], re.M)]
    for i, (at, heading) in enumerate(heads):
        body = owners[0][at:heads[i + 1][0]] if i + 1 < len(heads) else owners[0][at:]
        chunk_props = prop_rows(body)
        if heading == component:
            root |= chunk_props
        elif heading.startswith(component + '.'):
            part = heading[len(component) + 1:]
            # `### Calendar.Cell Render Props` is the `Calendar.Cell` part; the
            # trailing words describe the table's kind, not the part.
            if part.endswith(' Render Props'):
                part = part[:-len(' Render Props')]
            parts.setdefault(part, set()).update(chunk_props)
        else:
            folds.setdefault(heading, set()).update(chunk_props)
        # A `Component | Prop | ...` table names its owner per row, so its
        # rows carry an ownership the heading cannot: `### Year Picker Parts`
        # is not a `Comp.Part` heading, and the table attributes `visibleYears`
        # to `Calendar.YearPickerGrid`, `format`/`offset` to
        # `Calendar.YearPickerTriggerHeading`. The owner is authoritative
        # regardless of which heading hosts the table. A part named in one of
        # these tables and missing from `PART_STRUCTS` hits the same unowned
        # check below as a `###` part table would.
        for owner, owned in prop_rows_owned(body).items():
            if owner == component:
                root |= owned
            elif owner.startswith(component + '.'):
                parts.setdefault(owner[len(component) + 1:], set()).update(owned)
            else:
                parts.setdefault(owner, set()).update(owned)
    # A part table with real props and no PART_STRUCTS entry would silently
    # fall back to the root's set, which is the hole this fixes -- report it
    # rather than let a future part table launder again.
    for part in sorted(parts):
        if parts[part] - SKIP and '%s.%s' % (component, part) not in PART_STRUCTS:
            print('PART TABLE UNOWNED: %s.%s -- add a PART_STRUCTS entry'
                  % (component, part))
    return root, parts, folds


# The first header cell of a v3 prop table. Anything else is a table of
# *values*: `### Kbd.Content Type` lists the key names `keyValue` accepts under
# `| Modifier Keys | Special Keys | ...`, and reading its first column reported
# `command`, `ctrl`, `option`, `shift` and `win` as five missing Kbd props.
PROP_HEADERS = ('prop', 'name', 'option', 'function', 'method', 'prop name', 'event')

TABLE_RE = r'^\|(?P<head>.+)\|[ \t]*\n\|[ \t:|-]+\|[ \t]*\n(?P<body>(?:\|.*\n?)*)'


def prop_rows(text):
    """Every prop named in the prop tables of `text`.

    A markdown table is header row, divider row, then body; splitting on the
    divider is what tells the two apart, and the header is what says whether the
    first column holds prop names at all.

    One deliberate second shape: v3's `Year Picker Parts` tables put the part
    that owns a row in the first column and the prop in the second:

        | Component | Prop | Type | Default | Description |
        | `Calendar.YearPickerGrid` | `visibleYears` | number | ... |

    The first cell is `Component` -- not in `PROP_HEADERS` -- so the old rule
    silently passed the table over: `visibleYears` and friends were documented
    props that never showed up. The header says which is which, so the rule
    for this shape is one more line on the same reading: the first header is
    exactly `Component` *and* the second header is a prop indicator, and then
    the prop is column two. A table of values cannot pass -- `Modifier Keys |
    Special Keys` names neither header, `Component | Description` (the
    composition-parts listing) fails the second, and even a hypothetical
    `Component | Value` table has no backticked word in column two to extract.
    """
    found = set()
    for tbl in re.finditer(TABLE_RE, text, re.M):
        cells = tbl.group('head').split('|')
        first = cells[0].strip().strip('`').lower()
        if first in PROP_HEADERS:
            found |= set(re.findall(
                r'^\|\s*`([a-zA-Z-]+)`\s*\|', tbl.group('body'), re.M))
            continue
        # The `Component | Prop | ...` shape: per-row part ownership.
        if len(cells) > 1 and first == 'component':
            second = cells[1].strip().strip('`').lower()
            if second in PROP_HEADERS:
                found |= set(re.findall(
                    r'^\|\s*`[A-Za-z][A-Za-z0-9.]*`\s*\|\s*`([a-zA-Z-]+)`\s*\|',
                    tbl.group('body'), re.M))
    return found


def prop_rows_owned(text):
    """The `Component | Prop | ...` tables of `text`, split per owning part.

    Returns `{owner: props}` where `owner` is the part the row's first column
    names (`Calendar.YearPickerGrid`), so the row can be attributed to that
    part rather than to whatever heading happens to host the table. The
    condition is the one `prop_rows` guards with: first header exactly
    `Component`, second header a prop indicator, and a backticked owner and
    prop in the body's first two cells.
    """
    owned = {}
    for tbl in re.finditer(TABLE_RE, text, re.M):
        cells = tbl.group('head').split('|')
        if len(cells) < 2:
            continue
        first = cells[0].strip().strip('`').lower()
        if first != 'component':
            continue
        second = cells[1].strip().strip('`').lower()
        if second not in PROP_HEADERS:
            continue
        for m in re.finditer(
                r'^\|\s*`([A-Za-z][A-Za-z0-9.]*)`\s*\|\s*`([a-zA-Z-]+)`\s*\|',
                tbl.group('body'), re.M):
            owner, prop = m.group(1), m.group(2)
            owned.setdefault(owner, set()).add(prop)
    return owned


def main():
    gap_total = 0
    wont_total = 0
    # Per reason, because one blanket number says nothing about what it covers:
    # 55 interaction values a render function would have received is a different
    # claim from 55 missing props.
    by_reason = {}
    documented = 0
    unattributed = []
    for comp in sorted(FILES):
        f = FILES[comp]
        # Prefer the component's own impl block; fall back to the whole file (and
        # report it) when the Rust struct carries a different name.
        if comp in impl_methods:
            have = set(impl_methods[comp])
        else:
            have = set(methods.get(f, set()))
            unattributed.append(comp)
        have |= constructor_args.get(comp, set())
        for part in COMPANIONS.get(comp, ()):
            have |= impl_methods.get(part, set())
            have |= constructor_args.get(part, set())
        state = props_for_state(comp)
        if state is None:
            continue
        root_doc, part_docs, fold_docs = state
        props = set(root_doc)
        for chunk in part_docs.values():
            props |= chunk
        for chunk in fold_docs.values():
            props |= chunk
        if not props:
            continue
        missing = []
        for p in sorted(props):
            if p in SKIP:
                continue
            documented += 1
            # A prop documented on a `### Comp.Part` table belongs to that part:
            # it has to be a builder on the part's own struct (PART_STRUCTS),
            # never a same-named builder on the root. `Tabs::is_disabled`
            # disables the whole list; only `TabItem` can answer
            # `Tabs.Tab.isDisabled`. Two recorded exceptions exist:
            #
            # * a **part-scoped alias** (`Comp.Part.prop`) -- a per-row human
            #   decision that the prop is implemented under a different
            #   spelling, which may sit anywhere in the component's
            #   implementation (`Accordion.Item.isDisabled` is `disabled_keys`
            #   on the group);
            # * a **component-scoped alias** (`Comp.prop`) on a part row, the
            #   pre-existing mechanism for the same decision
            #   (`Pagination.onPress` is the group's `on_change`).
            # A bare global or snake resolution is checked against the part's
            # structs only, which is the check that used to pass by accident.
            # A name documented on both a root table and a part table
            # (`Calendar.isDisabled` and `Slider.isDisabled` appear on the
            # component *and* on `Cell`/`Thumb`) is one counted row, and the
            # most specific reading governs it: the root builder answers the
            # whole component, and it is the per-part reading the fold used to
            # launder. The part-scoped aliases above are how the implemented
            # per-part readings are recorded. Folded headings sit between the
            # two: `### ToastQueue`, `### Composition Components` and
            # `### Radio` are not `Comp.Part` tables, so their rows hang off
            # the heading itself -- answered through `FOLD_STRUCTS`, the
            # component's own set, or a fold-scoped alias
            # (`Comp.Heading.prop`), never through the root table's set by
            # default when the heading names a different entity.
            part = next((pt for pt in part_docs if p in part_docs[pt]), None)
            fold = next((h for h in fold_docs if p in fold_docs[h]), None)
            part_key = '%s.%s.%s' % (comp, part, p) if part else None
            fold_key = '%s.%s.%s' % (comp, fold, p) if fold else None
            comp_key = '%s.%s' % (comp, p)
            snake = re.sub(r'(?<!^)(?=[A-Z])', '_', p).lower()
            if part and part_key in ALIAS:
                rust = ALIAS[part_key]
                ok = rust in have or p.lower() in have
            elif part is not None:
                owned = set()
                for struct in PART_STRUCTS.get('%s.%s' % (comp, part), ()):
                    owned |= impl_methods.get(struct, set())
                    owned |= constructor_args.get(struct, set())
                if comp_key in ALIAS:
                    rust = ALIAS[comp_key]
                    ok = rust in have or p.lower() in have
                else:
                    rust = ALIAS.get(p, snake)
                    ok = rust in owned or p.lower() in owned
            elif fold is not None:
                # A folded heading answers through the structs its own table
                # describes. The fold-scoped alias key `Comp.Heading.prop` is
                # the third scoped tier: a table that documents per-item props
                # of a monolithic component (`### Radio` on the RadioGroup
                # page) can name a builder outside the fold's own structs --
                # `RadioGroup.Radio.name` is the group's `name`, the shared
                # submission name every option submits under, while the
                # option's own `value` and `is_disabled` live on the
                # `RadioOption` structs the fold names. A fold WITH a
                # `FOLD_STRUCTS` entry is then a scoped table like a part:
                # its rows are answered by that entry's own structs (or by an
                # explicit alias, which may name any builder in the
                # component's set), never by a same-named builder on the
                # component root by default. A fold without an entry
                # (`### Render Props`, `### ToastQueue`, `### ListLayout`)
                # documents the component's own state or a companion it
                # exposes, so the component's answer set stands.
                if fold_key in ALIAS:
                    rust = ALIAS[fold_key]
                    ok = rust in have or p.lower() in have
                elif (comp, fold) in FOLD_STRUCTS:
                    owned = set()
                    for struct in FOLD_STRUCTS[(comp, fold)]:
                        owned |= impl_methods.get(struct, set())
                        owned |= constructor_args.get(struct, set())
                    if comp_key in ALIAS:
                        rust = ALIAS[comp_key]
                        ok = rust in have or rust in owned or p.lower() in have
                    else:
                        rust = ALIAS.get(p, snake)
                        ok = rust in owned or p.lower() in owned
                else:
                    scope = set(have)
                    for struct in FOLD_STRUCTS.get((comp, fold), ()):
                        scope |= impl_methods.get(struct, set())
                        scope |= constructor_args.get(struct, set())
                    rust = ALIAS.get(comp_key) or ALIAS.get(p, snake)
                    ok = rust in scope or p.lower() in scope
            else:
                rust = ALIAS.get(comp_key) or ALIAS.get(p, snake)
                ok = rust in have or p.lower() in have
            if ok:
                continue
            # A reason may be global (`prop`), component-scoped
            # (`Component.prop`) or fold-scoped (`Component.Heading.prop`);
            # the scoped forms keep a blanket name from hiding a real gap
            # elsewhere.
            reason = WONT_PORT.get(fold_key) or WONT_PORT.get(comp_key) or WONT_PORT.get(p)
            if reason:
                wont_total += 1
                by_reason[reason] = by_reason.get(reason, 0) + 1
                continue
            missing.append(p)
        if missing:
            gap_total += len(missing)
            print('%-20s %s' % (comp, ', '.join(missing)))

    print()
    print('documented props considered : %d' % documented)
    print('implemented                 : %d' % (documented - gap_total - wont_total))
    print('deliberately not ported     : %d  (see WONT_PORT)' % wont_total)
    for reason, n in sorted(by_reason.items(), key=lambda kv: (-kv[1], kv[0])):
        print('    %-28s %d' % (reason, n))
    print('REAL GAPS                   : %d' % gap_total)
    if unattributed:
        print()
        print('no impl block matched (checked file-wide): %s' % ', '.join(unattributed))


if __name__ == '__main__':
    main()
