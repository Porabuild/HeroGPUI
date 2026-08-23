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
    'selectedKey': 'selected_key', 'errorMessage': 'error_message',
    'colorSpace': 'color_space', 'xChannel': 'x_channel', 'yChannel': 'y_channel',
    'showDots': 'show_dots', 'colorName': 'color_name', 'hourCycle': 'hour_cycle',
    'placeholderValue': 'placeholder_value', 'minValue': 'min_value', 'maxValue': 'max_value',
    'allowsCustomValue': 'allows_custom_value', 'menuTrigger': 'menu_trigger',
    'hideScrollBar': 'hide_scroll_bar', 'showControls': 'show_controls',
    'emptyState': 'empty_state', 'scaleFactor': 'scale_factor', 'htmlFor': 'label_for',
    'showValueLabel': 'show_value_label', 'weeksInMonth': 'weeks_in_month',
    'firstDayOfWeek': 'first_day_of_week', 'isDateUnavailable': 'is_date_unavailable',
    'focusedValue': 'focused_value', 'defaultSelectedKey': 'selected_key',
    # Tabs is the one component with a separate uncontrolled builder.
    'Tabs.defaultSelectedKey': 'default_selected_key',
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
    # v3 hands these *into* a child render function. The builder that takes the
    # closure is what implements the prop, so the alias points at it: a caller
    # receives the value rather than supplying it.
    'Dropdown.isSelected': 'item_content',
    'Dropdown.isIndeterminate': 'item_content',
    'InputOTP.index': 'slot',
    'Pagination.isActive': 'link',
    'Table.sortDirection': 'indicator',
    'Slider.index': 'thumb',
    'DateField.segment': 'segment',
    # Taken positionally or by the state's constructor, so the prop exists --
    # it is just not spelled as a builder.
    'TagGroup.items': 'tags',
    'Table.items': 'row',
    'InputOTP.maxLength': 'with_length',
    # v3 calls these `type`; `type` is a Rust keyword.
    'Input.type': 'input_type',
    'Typography.type': 'kind',
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
    # Uncontrolled mode exists (`util::controlled` + `use_keyed_state`), so
    # `defaultOpen`, `defaultSelected`, `defaultExpanded*` and
    # `defaultYearPickerOpen` are implemented, not omitted. What is left here is
    # the collection-valued seed: `Vec`/`Selection` state that the caller owns
    # as an entity, where a one-shot initial value has nothing to seed.
    'defaultSelectedKeys': 'caller-owns-collection',
    # Tabs has a real `default_selected_key`; ComboBox's selection lives in the
    # caller's `InputState`, so there is nothing separate to seed.
    # ComboBox's single selection lives in the caller's `InputState`, which is
    # its own uncontrolled seed (`InputState::with_value`).
    'ComboBox.defaultSelectedKey': 'state-entity-seeds-it',
   
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
    # A date field shows no time, so it has neither a granularity below a day
    # nor an hour cycle. `TimeField` implements both.
    'DateField.granularity': 'date-only-field',
    'DateField.hourCycle': 'date-only-field',

    # Choosing separators, digit systems and currency placement per locale needs
    # CLDR data; a partial table would be worse than not offering the prop.
    # `formatOptions` itself is implemented -- see `core/src/format.rs`.
    'locale': 'no-intl',
    # There is no browser to navigate or post to.
    'action': 'no-http', 'method': 'no-http', 'encType': 'no-http',
    'target': 'no-http', 'download': 'no-http', 'rel': 'no-http',
    # A hint for the browser's autofill, which there is none of here.
    'autoComplete': 'no-browser-autofill',
    # ARIA roles have no accessibility layer to reach.
    'role': 'no-a11y-attrs',
    # Browser-only input affordances.
 'inputMode': 'no-soft-keyboard',
   

    # Sub-component/table-parsing artefacts, not real props of ours.
    'state': 'not-a-prop', 'toast': 'not-a-prop',
    # Tooltip's `trigger` is a real prop, unlike the sub-component rows
    # above: 'focus' needs a child that takes keyboard focus, and nothing
    # in this library is focusable yet, so it would be a dead builder.
    'Tooltip.trigger': 'no-keyboard-focus',
    # `TextArea` wraps (gpui's default `WhiteSpace::Normal`), so there is
    # multi-line layout -- but no `pre`/`pre-wrap` mode to select between, which
    # is what `wrap` chooses.
    'TextArea.wrap': 'single-wrap-mode',
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
    # gpui exposes no accessibility title attribute.
    'Kbd.title': 'no-a11y-attrs',
    # Browser image-loading attributes with no gpui analogue.
    'Avatar.crossOrigin': 'no-browser-image-attrs',
    'Avatar.loading': 'no-browser-image-attrs',
    # gpui's img() reports no load or error events, so a fallback delay has
    # nothing to key off.
    'Avatar.delayMs': 'no-image-load-events',

    # gpui gives a RenderOnce element no scroll offset, so there is nothing
    # truthful to report.
    'onVisibilityChange': 'no-scroll-offset',
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


# The first header cell of a v3 prop table. Anything else is a table of
# *values*: `### Kbd.Content Type` lists the key names `keyValue` accepts under
# `| Modifier Keys | Special Keys | ...`, and reading its first column reported
# `command`, `ctrl`, `option`, `shift` and `win` as five missing Kbd props.
PROP_HEADERS = ('prop', 'name', 'option', 'function', 'method', 'prop name', 'event')


def prop_rows(text):
    """Every prop named in the prop tables of `text`.

    A markdown table is header row, divider row, then body; splitting on the
    divider is what tells the two apart, and the header is what says whether the
    first column holds prop names at all.
    """
    found = set()
    for tbl in re.finditer(
            r'^\|(?P<head>.+)\|[ \t]*\n\|[ \t:|-]+\|[ \t]*\n(?P<body>(?:\|.*\n?)*)',
            text, re.M):
        first = tbl.group('head').split('|')[0].strip().strip('`').lower()
        if first not in PROP_HEADERS:
            continue
        found |= set(re.findall(r'^\|\s*`([a-zA-Z-]+)`\s*\|', tbl.group('body'), re.M))
    return found


def main():
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
        have |= constructor_args.get(comp, set())
        for part in COMPANIONS.get(comp, ()):
            have |= impl_methods.get(part, set())
            have |= constructor_args.get(part, set())
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


if __name__ == '__main__':
    main()
