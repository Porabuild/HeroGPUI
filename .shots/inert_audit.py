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

Each demo instance is the text from `h::X::new(` to its `.into_any_element()`.

A nested component's callbacks fall inside the outer one's text, so the reading
errs toward silence rather than noise. `ALLOW` records the instances that are
frozen on purpose, keyed `Component#element-id`, with a reason -- a disabled
control cannot change whatever it is passed.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

SRC = 'crates/herogpui-components/src/'
PAGES = ('gallery/src/pages/components.rs', 'gallery/src/pages/docs.rs')

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
}

# Instances that are meant to be frozen, with the reason.
ALLOW = {
    # v3 draws a disabled control in both states, and neither one moves.
    'ToggleButton#tb-dis-2': 'disabled',
    'Switch#sw-d-on': 'disabled',
    'Checkbox#cb-dis': 'disabled',
    'ColorSwatchPicker#csp-disabled': 'disabled',
    'RadioGroup#rg-d': 'disabled',
}


def controlled_builders():
    """`{struct: {builder fn}}` for the props a component expects to be driven."""
    out = {}
    for name in sorted(os.listdir(SRC)):
        if not name.endswith('.rs'):
            continue
        src = io.open(SRC + name, encoding='utf-8', errors='replace').read()
        # Which struct's render calls `controlled`, and on which fields. Reading
        # this per file instead let `CheckboxGroup`'s `value` count as
        # `Checkbox`'s -- and `Checkbox::value` is the *form* value, which is
        # nobody's state.
        for m in re.finditer(r'\nimpl RenderOnce for (\w+) \{(.*?)\n\}', src, re.S):
            struct, body = m.group(1), m.group(2)
            fields = set()
            for call in re.finditer(r'controlled\(\s*window,\s*cx,(.*?)\n\s*\);', body, re.S):
                for a, b in re.findall(r'self\.(\w+)\.clone\(\)|self\.(\w+),', call.group(1)):
                    fields.add(a or b)
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
        props = set(api_audit.props_for(comp))
        for prop in props:
            cap = prop[0].upper() + prop[1:]
            if ('default' + cap) not in props and ('on' + cap + 'Change') not in props:
                continue
            if '%s.%s' % (comp, prop) in NOT_STATE:
                continue
            out.setdefault(comp, set()).add(re.sub(r'(?<!^)(?=[A-Z])', '_', prop).lower())
    return out


def instances(src):
    """`(struct, element id, source line, the instance's text)` per demo."""
    pattern = (r'h::(\w+)::(?:new|uncontrolled)\(\s*(?:\n\s*)?'
               r'(?:"([^"]*)"|el_id\(format!\("([^"{]*))?')
    for m in re.finditer(pattern, src):
        end = src.find('.into_any_element()', m.end())
        chunk = src[m.end():end if end > 0 else m.end() + 1200]
        eid = m.group(2) or m.group(3) or ''
        yield m.group(1), eid.rstrip('-'), src[:m.start()].count('\n') + 1, chunk


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
    for name in sorted(os.listdir(SRC)):
        if not name.endswith('.rs'):
            continue
        src = io.open(SRC + name, encoding='utf-8', errors='replace').read()
        for m in pattern.finditer(src):
            out.append('%-20s line %-6d %-16s %s%s%s'
                       % (name, src[:m.start()].count('\n') + 1, m.group(1),
                          quote, m.group(2), quote))
    return out


def main():
    builders = controlled_builders()
    for struct, props in documented_state().items():
        builders.setdefault(struct, set()).update(props)
    frozen, allowed = [], 0
    checked = 0
    for path in PAGES:
        src = io.open(path, encoding='utf-8', errors='replace').read()
        for struct, eid, line, chunk in instances(src):
            if struct not in builders:
                continue
            used = sorted(b for b in builders[struct]
                          if '.%s(' % b in chunk and '.%s(None)' % b not in chunk)
            if not used:
                continue
            checked += 1
            if '.on_' in chunk:
                continue
            key = '%s#%s' % (struct, eid)
            if key in ALLOW:
                allowed += 1
                continue
            frozen.append('%-18s %-22s line %-6d sets %s, no callback'
                          % (struct, eid or '(no id)', line, ', '.join(used)))

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


if __name__ == '__main__':
    main()
