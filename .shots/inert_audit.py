"""Find state that is not per-instance: frozen demos, and shared keys.

Two failures with one cause -- the state a control reads is not the state the
user is changing -- and neither shows up in a screenshot.

Every other audit asks about the library; this one asks about the demo app, and
it exists because a green audit hid a dead component for weeks. `Tabs::new`'s
positional key filled `selectedKey` -- the *controlled* prop -- so
`util::controlled` handed the value straight back with no state entity, the
component skipped its whole interactive block, and every Tabs demo that passed a
literal was inert. It looked completely normal in a screenshot.

The rule is mechanical. A component that calls `util::controlled` has, for each
piece of state, a controlled builder and an uncontrolled `default_*` one. Set the
controlled builder and you have promised to drive it: without an `on_*` callback
the control is frozen. A demo that only wants to *show* the on state should say
`default_selected(true)`, which looks the same and still toggles.

Which builders count is read two ways, because our own code only names half of
them. A component that keeps a fallback calls `util::controlled`, and its
controlled field is right there in the call. A *fully* controlled one -- Select,
ColorSwatchPicker -- has no such call, and v3's prop tables are what identify its
state: a prop `P` documented next to `defaultP` or `onPChange` is state by
definition. That second pass is what found four selects that dropped the choice
on the floor and six swatch pickers that could not be pressed.

Only direct builder calls belong to an instance. Callbacks on nested children,
sibling controls, strings and test fixtures cannot drive its state. This is a
static wiring check: a callback's presence does not prove that it stores and
feeds back the new value. Runtime tests must prove that part.
"""
import os
import re
import sys

from component_source import list_modules, read_module, read_path
from design_audit import _balanced_delimiter_end, mask_comments, mask_literals, strip_cfg_test
from gallery_pages import page_sources

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SRC = 'crates/herogpui-components/src/'
PAGES = page_sources()

# The calls that put a piece of state in the window's keyed store. Each one needs
# a key derived from the component's own id, or every instance shares it.
KEYED = (
    'use_keyed_state', 'controlled', 'overlay_phase', 'focus_once',
    'panel_focus', 'tab_stop_handle',
)

# Props that read like state and are not. A `default*` sibling is the test, and
# `ScrollShadow.visibility` has one without being anything the user changes.
NOT_STATE = {
    'ScrollShadow.visibility': 'static-configuration',
    # `defaultChildren` is not the uncontrolled seed of `children`: it is what
    # the value slot *would have drawn*, handed into the render prop so a caller
    # can return it unchanged (`if (isPlaceholder) return defaultChildren`). The
    # pair reads like state and is a render-prop argument.
    'Autocomplete.children': 'default-is-the-rendering',
    'Select.children': 'default-is-the-rendering',
    'ComboBox.children': 'default-is-the-rendering',
    # These builders seed InputState only once. Later imperative updates and
    # edits belong to the entity; they cannot freeze a gallery control. This
    # port limitation is not evidence of upstream controlled-input parity.
    'ComboBox.inputValue': 'initial-input-state-seed',
    'ComboBox.value': 'initial-input-state-seed',
}

# These owners reject value mutations while disabled. Open overlay state needs
# its own callback even when the trigger is disabled (notably Select).
DISABLED_STATE = {
    'ToggleButton': {'is_selected'}, 'Switch': {'is_selected'},
    'Checkbox': {'is_selected'}, 'ColorSwatchPicker': {'value'},
    'RadioGroup': {'value'}, 'Slider': {'value', 'values'},
}


def group_end(source, opening):
    end = _balanced_delimiter_end(source, opening, source[opening],
                                  {'(': ')', '[': ']', '{': '}'}[source[opening]])
    if end is None:
        raise ValueError(f'unclosed group at offset {opening}')
    return end


def arguments(source, separator=','):
    """Split call arguments, retaining nested expressions as one argument."""
    searchable = mask_literals(mask_comments(source))
    parts, start, cursor = [], 0, 0
    while cursor < len(source):
        if searchable[cursor] in '([{':
            cursor = group_end(searchable, cursor)
        elif searchable[cursor] == separator:
            parts.append(source[start:cursor].strip())
            cursor += 1
            start = cursor
        else:
            cursor += 1
    if source[start:].strip():
        parts.append(source[start:].strip())
    return parts


def controlled_builders():
    """`{struct: {builder fn}}` for the props a component expects to be driven."""
    out = {}
    for name in list_modules(SRC.rstrip('/')):
        src = mask_literals(strip_cfg_test(read_module(name, SRC.rstrip('/'), errors='replace')))
        # Which struct's render calls `controlled`, and on which fields. Reading
        # this per file instead let `CheckboxGroup`'s `value` count as
        # `Checkbox`'s -- and `Checkbox::value` is the *form* value, which is
        # nobody's state.
        for m in re.finditer(r'\nimpl RenderOnce for (\w+) \{(.*?)\n\}', src, re.S):
            struct, body = m.group(1), m.group(2)
            fields = set()
            for call in re.finditer(r'\bcontrolled\(', body):
                end = group_end(body, call.end() - 1)
                args = arguments(body[call.end():end - 1])
                if len(args) != 5:
                    raise ValueError(f'{struct}: unreadable controlled() arguments')
                # Only argument four is controlled state. The seed expression
                # may mention min/max/step, disabled flags and other config.
                fields.update(re.findall(r'\bself\s*\.\s*(\w+)', args[3]))
                if re.fullmatch(r'\w+', args[3]):
                    # A normalized controlled value can be bound first, as in
                    # Slider's range_controlled = self.values...map(...).
                    binding = re.search(r'\blet\s+' + re.escape(args[3])
                                        + r'\s*=\s*self\s*\.\s*(\w+)', body)
                    if binding:
                        fields.add(binding.group(1))
            fields = {field for field in fields if not field.startswith('default')}
            if not fields:
                continue
            # The builders that assign those fields, minus the uncontrolled
            # seeds: `default_selected` is the honest way to show the on state.
            impl = re.search(r'\nimpl %s \{(.*?)\n\}' % struct, src, re.S)
            if not impl:
                continue
            for bm in re.finditer(r'pub fn (\w+)\(\s*mut self.*?\n(.*?)\n    \}',
                                  impl.group(1), re.S):
                fn = bm.group(1)
                if fn.startswith('default') or fn == 'id':
                    continue
                for field in re.findall(r'self\.(\w+) = ', bm.group(2)):
                    if field in fields:
                        out.setdefault(struct, set()).add(fn)
    return out


def documented_state():
    """`{struct: {builder}}` for the state props v3 documents, per component.

    `P` alongside `defaultP` or `onPChange` is a controlled prop; anything else
    in the table is configuration.
    """
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import api_audit

    out = {}
    for comp in sorted(api_audit.FILES):
        props, _, _ = api_audit.props_for_state(comp)
        for prop in props:
            cap = prop[0].upper() + prop[1:]
            if ('default' + cap) not in props and ('on' + cap + 'Change') not in props:
                continue
            if '%s.%s' % (comp, prop) in NOT_STATE:
                continue
            snake = re.sub(r'(?<!^)(?=[A-Z])', '_', prop).lower()
            out.setdefault(comp, set()).add(snake)
            mapped = api_audit.ALIAS.get(f'{comp}.{prop}')
            if mapped and re.fullmatch(r'\w+', mapped):
                out[comp].add(mapped)
    return out


def instances(src):
    """Yield production constructors and their direct fluent builder calls."""
    src = strip_cfg_test(src)
    searchable = mask_literals(src)
    for match in re.finditer(r'\bh::(\w+)::(new|uncontrolled)\s*\(', searchable):
        end = group_end(searchable, match.end() - 1)
        args = arguments(src[match.end():end - 1])
        eid = args[0] if args else '(no id)'
        methods = {}
        while method := re.match(r'\s*\.\s*(\w+)\s*\(', searchable[end:]):
            opening = end + method.end() - 1
            end = group_end(searchable, opening)
            methods[method.group(1)] = src[opening + 1:end - 1].strip()
        yield match.group(1), methods.get('id', eid), src[:match.start()].count('\n') + 1, methods


def callbacks(struct, builder, methods):
    if struct == 'Select':
        if builder == 'selected_keys':
            return {'on_selection_change_all'}
        if builder == 'value':
            return {'on_change', 'on_selection_change'}
    if struct in ('Autocomplete', 'ComboBox') and builder in ('selected_key', 'selected_keys', 'value'):
        mode = methods.get('selection_mode', 'SelectionMode::Single')
        mode_match = re.fullmatch(r'(?:\w+::)*SelectionMode::(Single|Multiple)', mode)
        if not mode_match:
            raise ValueError(f'{struct}: unresolved selection_mode for controlled selection: {mode}')
        if mode_match.group(1) == 'Multiple':
            return {'on_selection_change_all'}
        return {'on_change', 'on_selection_change', 'on_selection_change_all'}
    if builder in ('selected_key', 'selected_keys'):
        if struct == 'ToggleButtonGroup':
            return {'on_selection_change', 'on_change'}
        return {'on_selection_change'}
    if builder in ('expanded_keys', 'is_expanded'):
        if struct == 'Accordion':
            return {'on_expanded_change', 'on_toggle'}
        return {'on_expanded_change'}
    if builder == 'is_open':
        return {'on_open_change'}
    if builder == 'is_year_picker_open':
        return {'on_year_picker_open_change'}
    if builder == 'input_value':
        return {'on_input_change'}
    if builder == 'values':
        return {'on_change_all'}
    if builder in ('value', 'is_selected'):
        return {'on_change'}
    raise ValueError(f'{struct}.{builder}: no state callback mapping')


def collection_nonempty(expression):
    """True/False for known collection cardinality; None for runtime values."""
    source = mask_comments(expression).strip()
    compact = re.sub(r'\s+', '', source)
    if compact in ('Vec::new()', 'std::iter::empty()') or re.fullmatch(r'Vec::<[^>]+>::new\(\)', compact):
        return False
    array = re.fullmatch(r'(?:vec!\s*)?\[(.*)\]', source, re.S)
    if not array:
        return None
    body = array.group(1).strip()
    repeat = arguments(body, ';')
    if len(repeat) > 1:
        return int(repeat[1]) > 0 if len(repeat) == 2 and repeat[1].isdigit() else None
    return bool(arguments(body))


def frozen_instances(src, builders):
    frozen, checked, allowed = [], 0, 0
    for struct, eid, line, methods in instances(src):
        used = set(builders.get(struct, ())) & methods.keys()
        # Slider's positional scalar value is controlled too. A default seed
        # or a nonempty range mode bypasses it, even after a value() call.
        if struct == 'Slider':
            ranges = {b: collection_nonempty(methods[b]) for b in ('values', 'default_values')
                      if b in methods}
            if ranges.get('values') is False:
                used.discard('values')
            if 'default_value' in methods or True in ranges.values():
                used.discard('value')
            else:
                used.add('value')
        if not used:
            continue
        checked += 1
        static = DISABLED_STATE.get(struct, set()) if methods.get('is_disabled') == 'true' else set()
        # Read-only range selection still permits navigation/year switching.
        read_only_value = struct == 'RangeCalendar' and methods.get('is_read_only') == 'true'
        # Their content render path returns before creating a trigger/overlay.
        # The custom child owns open-state actions; a root callback never runs.
        custom_open = struct in ('DatePicker', 'DateRangePicker') and 'content' in methods
        missing = [b for b in sorted(used) if not (callbacks(struct, b, methods) & methods.keys())
                   and b not in static and not (read_only_value and b == 'value')
                   and not (custom_open and b == 'is_open')]
        if missing:
            frozen.append('%-18s %-22s line %-6d sets %s, no matching callback'
                          % (struct, eid, line, ', '.join(missing)))
        elif used <= static or (read_only_value and used == {'value'}) or (custom_open and used == {'is_open'}):
            allowed += 1
    return frozen, checked, allowed


def shared_keys():
    """Keyed state whose key is a bare literal, so every instance shares it.

    `Dropdown` keyed its open flag by the constant `"dropdown-open"`, so pressing
    any trigger on a page opened *every* menu on it. Modal, Drawer and
    AlertDialog shared one exit phase, one focus handle and one drag offset the
    same way, which `HEROGPUI_OPEN_OVERLAYS=1` puts on screen at once. The fix is
    an `id` builder and `format!("{id:?}-open")`; the check is that no literal
    key is left.
    """
    quote = chr(34)
    pattern = re.compile(
        r'(' + '|'.join(KEYED) + r')\(\s*(?:window,\s*)?(?:cx,\s*)?'
        + quote + r'([^' + quote + r']*)' + quote
    )
    out = []
    for name in list_modules(SRC.rstrip('/')):
        src = strip_cfg_test(read_module(name, SRC.rstrip('/'), errors='replace'))
        searchable = mask_literals(src)
        for m in pattern.finditer(src):
            if not re.match(r'\w', searchable[m.start():]):
                continue
            out.append('%-20s line %-6d %-16s %s%s%s'
                       % (name, src[:m.start()].count('\n') + 1, m.group(1),
                          quote, m.group(2), quote))
    return out


def state_builders():
    builders = controlled_builders()
    for struct, props in documented_state().items():
        builders.setdefault(struct, set()).update(props)
    return builders


def main():
    builders = state_builders()
    frozen, allowed = [], 0
    checked = 0
    for path in PAGES:
        src = read_path(path, errors='replace')
        rows, count, static = frozen_instances(src, builders)
        frozen.extend(f'{path}: {row}' for row in rows)
        checked += count
        allowed += static

    for row in frozen:
        print('FROZEN   ' + row)
    shared = shared_keys()
    for row in shared:
        print('SHARED   ' + row)
    print()
    print('controlled components : %d' % len(builders))
    print('driven instances      : %d' % checked)
    print('frozen on purpose     : %d' % allowed)
    print('FROZEN                : %d' % len(frozen))
    print('SHARED KEYS           : %d' % len(shared))
    if not builders or not checked:
        raise ValueError('no controlled components or gallery instances read')
    return 1 if frozen or shared else 0


if __name__ == '__main__':
    sys.exit(main())
