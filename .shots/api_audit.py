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
    'onPress': 'on_press', 'onChange': 'on_change', 'onSelectionChange': 'on_selection_change',
    'onOpenChange': 'on_open_change', 'onAction': 'on_action', 'onRemove': 'on_remove',
    'onSubmit': 'on_submit', 'onClose': 'on_close', 'onConfirm': 'on_confirm',
    'onCancel': 'on_cancel', 'onClear': 'on_clear', 'onToggle': 'on_toggle',
    'onSortChange': 'on_sort_change', 'onFocusChange': 'on_focus_change',
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
    'defaultSelectedKeys': 'selected_keys', 'disabledKeys': 'disabled_keys',
    'selectedKey': 'selected_key', 'defaultValue': 'value', 'errorMessage': 'error_message',
    'colorSpace': 'color_space', 'xChannel': 'x_channel', 'yChannel': 'y_channel',
    'showDots': 'show_dots', 'colorName': 'color_name', 'hourCycle': 'hour_cycle',
    'placeholderValue': 'placeholder_value', 'minValue': 'min_value', 'maxValue': 'max_value',
    'allowsCustomValue': 'allows_custom_value', 'menuTrigger': 'menu_trigger',
    'hideScrollBar': 'hide_scroll_bar', 'showControls': 'show_controls',
    'emptyState': 'empty_state', 'scaleFactor': 'scale_factor', 'htmlFor': 'label_for',
    'showValueLabel': 'show_value_label', 'weeksInMonth': 'weeks_in_month',
    'firstDayOfWeek': 'first_day_of_week', 'isDateUnavailable': 'is_date_unavailable',
    'focusedValue': 'focused_value', 'defaultSelectedKey': 'selected_key',
    'renderEmptyState': 'empty_state', 'onInputChange': 'on_input_change',
    'defaultFilter': 'filter', 'autoFocus': 'auto_focus',
    # `Dropdown.ItemIndicator`'s type is our `indicator` builder.
    'Dropdown.type': 'indicator',
    # v3 documents the plain attribute spellings alongside the is* ones.
    'disabled': 'is_disabled', 'readOnly': 'is_read_only', 'required': 'is_required',
    'inputValue': 'input_value', 'shouldFlip': 'should_flip',
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
    # Accordion.Trigger's extra press handler, and Accordion.Item's controlled
    # expansion, are expressed on the group.
    'Accordion.onPress': 'on_toggle',
    'Accordion.isExpanded': 'expanded_keys',
    # Autocomplete.ClearButton's click handler.
    'Autocomplete.onClick': 'on_clear',
    'ColorSwatchPicker.variant': 'shape',
}

# Props with no meaningful gpui analogue at all.
SKIP = {
    'className', 'children', 'render', 'style', 'nativeProps', 'id', 'slot', 'ref',
    'asChild', 'key', 'queue', 'srcSet', 'sizes', 'alt', 'onLoad', 'onError',
    'aria-label', 'aria-labelledby', 'aria-describedby', 'aria-details', 'textValue',
    'containerClassName',
}

# Deliberately not ported, with the reason. These are reported separately so
# the "real gap" number stays meaningful.
WONT_PORT = {
    # Every component here is controlled; there is no uncontrolled mode.
    'defaultValue': 'controlled-only', 'defaultSelected': 'controlled-only',
    'defaultOpen': 'controlled-only', 'defaultSelectedKey': 'controlled-only',
    'defaultSelectedKeys': 'controlled-only', 'defaultInputValue': 'controlled-only',
   
    # React Aria validation plumbing; callers validate and pass `is_invalid`.
    # 'native' blocks HTML form submission, which does not exist here, so
    # only the 'aria' behaviour is meaningful.
    'validationBehavior': 'aria-behaviour-only',
    # An HTML5 ValidityState object.
    'validationDetails': 'no-html-forms',
    # Fires when native form validation blocks submission.
    'onInvalid': 'no-html-forms',
    # Intl / locale formatting has no gpui equivalent.
    'formatOptions': 'no-intl', 'locale': 'no-intl',
    # HTML form submission: gpui has no form-data model.
    'name': 'no-html-forms', 'action': 'no-html-forms', 'method': 'no-html-forms',
    'encType': 'no-html-forms', 'target': 'no-html-forms', 'onReset': 'no-html-forms',
    'startName': 'no-html-forms', 'endName': 'no-html-forms', 'download': 'no-html-forms',
    'rel': 'no-html-forms',
    # A hint for the browser's autofill, which there is none of here.
    'autoComplete': 'no-browser-autofill',
    # ARIA roles have no accessibility layer to reach.
    'role': 'no-a11y-attrs',
    # Browser-only input affordances.
 'inputMode': 'no-soft-keyboard',
   

    # Sub-component/table-parsing artefacts, not real props of ours.
    'state': 'not-a-prop', 'toast': 'not-a-prop', 'trigger': 'not-a-prop',
    'items': 'not-a-prop', 'ErrorMessage': 'not-a-prop', 'FieldError': 'not-a-prop',
    # Tooltip's `trigger` is a real prop, unlike the sub-component rows
    # above: 'focus' needs a child that takes keyboard focus, and nothing
    # in this library is focusable yet, so it would be a dead builder.
    'Tooltip.trigger': 'no-keyboard-focus',
    # gpui 0.2.2 has no multi-line text layout, so TextArea renders one tall
    # line and there is no wrapping to configure.
    'TextArea.wrap': 'no-multiline-layout',
    # v3 documents exactly one value for these, so there is nothing to select
    # and a builder taking a one-variant enum could not change anything.
    'CloseButton.variant': 'single-valued',
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
    'Slider.index': 'render-prop-arg',
    'InputOTP.index': 'render-prop-arg',
    'DateField.segment': 'render-prop-arg',
    'TimeField.segment': 'render-prop-arg',
    'Dropdown.isSelected': 'render-prop-arg',
    'Dropdown.isIndeterminate': 'render-prop-arg',
    'Table.sortDirection': 'render-prop-arg',
    'Table.columns': 'render-prop-arg',
    'Pagination.isActive': 'render-prop-arg',
    # Custom element slots: our builders take strings/elements positionally
    # rather than an override hook.
    'Table.indicator': 'composition-instead',
    # gpui exposes no accessibility title attribute.
    'Kbd.title': 'no-a11y-attrs',
    # Browser image-loading attributes with no gpui analogue.
    'Avatar.crossOrigin': 'no-browser-image-attrs',
    'Avatar.loading': 'no-browser-image-attrs',
    # gpui's img() reports no load or error events, so a fallback delay has
    # nothing to key off.
    'Avatar.delayMs': 'no-image-load-events',
    # A checkbox's `value` is its form-submission value.
    'Checkbox.value': 'no-html-forms',
    # Column resizing does not exist here, so its width hints have no meaning.
    'Table.defaultWidth': 'no-column-resize',
    'Table.minWidth': 'no-column-resize',
    'Input.type': 'renamed-kind', 'Typography.type': 'renamed-kind', 'htmlFor': 'composition-instead',

    # gpui gives a RenderOnce element no scroll offset, so there is nothing
    # truthful to report.
    'onVisibilityChange': 'no-scroll-offset',
    # OtpState::with_length owns the cell count.
    'InputOTP.maxLength': 'state-owns-length',
    'defaultExpandedKeys': 'controlled-only', 'defaultExpanded': 'controlled-only',
    # A single-date Calendar; RangeCalendar covers the range case and there is
    # no v3-shaped multi-date state here.

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
    'RadioGroup': ['RadioOption'],
    'Dropdown': ['Menu', 'MenuItem'],
    'Accordion': ['AccordionItem'],
    'Input': ['InputState'],
    'TextField': ['InputState'],
    'SearchField': ['InputState'],
    'TextArea': ['InputState'],
    'NumberField': ['NumberState'],
    'InputOTP': ['OtpState'],
    'Table': ['TableRow', 'TableColumn'],
    'Tabs': ['Tab'],
    'ToggleButtonGroup': ['ToggleButton'],
    'Calendar': ['CalendarState'],
    'RangeCalendar': ['DateRangeState'],
    'DatePicker': ['CalendarState'],
    'DateRangePicker': ['DateRangeState'],
    'Select': ['SelectOption'],
    'TagGroup': ['Tag'],
    'ColorSwatchPicker': ['ColorSwatch'],
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
        impl_methods.setdefault(m.group(1), set()).update(
            re.findall(r'pub fn ([a-z_0-9]+)\s*\(', body)
        )


def props_for(component):
    """Prop names documented for a component.

    v3 splits its API across the root table and one table per composed part
    (`### Tooltip.Content`, `### Select.Trigger`, ...). This port is monolithic
    -- those props land on the parent builder -- so both forms are folded
    together here. Reading only the root table hid whole prop tables.
    """
    found = set()
    pattern = r'^### %s(?:\.[A-Za-z]+)?\s*$' % re.escape(component)
    for m in re.finditer(pattern, bundle, re.M):
        chunk = bundle[m.end():m.end() + 4000]
        # stop at the next component heading
        nxt = re.search(r'^### ', chunk, re.M)
        if nxt:
            chunk = chunk[:nxt.start()]
        for row in re.findall(r'^\|\s*`([a-zA-Z-]+)`\s*\|', chunk, re.M):
            found.add(row)
    return found


gap_total = 0
wont_total = 0
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
    for part in COMPANIONS.get(comp, ()):
        have |= impl_methods.get(part, set())
    props = props_for(comp)
    if not props:
        continue
    missing = []
    for p in sorted(props):
        if p in SKIP:
            continue
        documented += 1
        # A scoped alias (`Component.prop`) wins over the global one, so a
        # name that means different things in different components can be
        # mapped per component.
        rust = ALIAS.get('%s.%s' % (comp, p)) or ALIAS.get(
            p, re.sub(r'(?<!^)(?=[A-Z])', '_', p).lower())
        if rust in have or p.lower() in have:
            continue
        # A reason may be global (`prop`) or scoped (`Component.prop`); the
        # scoped form keeps a blanket name from hiding a real gap elsewhere.
        if ('%s.%s' % (comp, p)) in WONT_PORT or p in WONT_PORT:
            wont_total += 1
            continue
        missing.append(p)
    if missing:
        gap_total += len(missing)
        print('%-20s %s' % (comp, ', '.join(missing)))

print()
print('documented props considered : %d' % documented)
print('implemented                 : %d' % (documented - gap_total - wont_total))
print('deliberately not ported     : %d  (see WONT_PORT)' % wont_total)
print('REAL GAPS                   : %d' % gap_total)
if unattributed:
    print()
    print('no impl block matched (checked file-wide): %s' % ', '.join(unattributed))
