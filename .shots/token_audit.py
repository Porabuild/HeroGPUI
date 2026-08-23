"""Diff every theme variable v3 declares against the tokens this port exposes.

v3's theming story *is* its variables: "override these CSS custom properties and
every component follows". The port's equivalent is `ThemeColors` and
`LayoutTokens`, so a token v3 declares and this port does not expose is a hole in
the theming surface -- a caller cannot reach it, and no component can read it.

Both sides are read mechanically:

- v3: every `--name:` declared in `packages/styles/themes/default/variables.css`,
  which `design_audit.py --fetch` caches.
- ours: the field and accessor names in `crates/herogpui-theme/src/`.

A CSS name maps to Rust by the obvious spelling (`--surface-foreground` ->
`surface.foreground`, `--accent-soft-hover` -> `accent.soft_hover()`), and
`ALIAS` records the ones where the shapes differ -- a role's six variables are
one `RoleColor` here, so `--danger-soft` is `danger.soft()`.

The second pass compares the **values**, per appearance. Every `oklch(..)`
literal v3 declares is read off both sides and diffed, which is how four
transcription errors were found -- a light border a step too pale, a dark border
and separator both too dark, a dark field painted in `--default` instead of the
surface colour -- plus one deliberate deviation (a dark overlay lightened "so
floating panels read"), which the no-improvements rule says to undo rather than
keep. A value that is a `color-mix` is not compared here: those are computed in
`semantic.rs` and checked by its own tests.

The third pass covers `layout.rs`: the lengths, the two tooltip delays and the
three shadows. A shadow is compared layer by layer -- offset, blur and alpha --
which is how the overlay's was caught having drifted to an older sheet's two
layers instead of v3's three, one of which throws its blur *upward*.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

CSS = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css', 'variables.css')
THEME = 'crates/herogpui-theme/src/'

# A CSS variable -> the Rust that exposes it, where the spelling differs.
ALIAS = {
    # The six-variable role shape is one `RoleColor` with derived accessors.
    '--accent': 'accent',
    '--accent-foreground': 'foreground',
    '--accent-hover': 'fn hover',
    '--accent-soft': 'fn soft',
    '--accent-soft-foreground': 'fn soft_foreground',
    '--accent-soft-hover': 'fn soft_hover',
    # `--default` and the status roles reuse the same shape.
    '--default': 'default',
    '--default-foreground': 'foreground',
    '--default-hover': 'fn hover',
    '--default-soft': 'fn soft',
    '--default-soft-foreground': 'fn soft_foreground',
    '--default-soft-hover': 'fn soft_hover',
    '--success': 'success',
    '--success-foreground': 'foreground',
    '--success-hover': 'fn hover',
    '--success-soft': 'fn soft',
    '--success-soft-foreground': 'fn soft_foreground',
    '--success-soft-hover': 'fn soft_hover',
    '--warning': 'warning',
    '--warning-foreground': 'foreground',
    '--warning-hover': 'fn hover',
    '--warning-soft': 'fn soft',
    '--warning-soft-foreground': 'fn soft_foreground',
    '--warning-soft-hover': 'fn soft_hover',
    '--danger': 'danger',
    '--danger-foreground': 'foreground',
    '--danger-hover': 'fn hover',
    '--danger-soft': 'fn soft',
    '--danger-soft-foreground': 'fn soft_foreground',
    '--danger-soft-hover': 'fn soft_hover',
    # Containers: one struct per surface, so the suffix is the field.
    '--surface': 'surface',
    '--surface-foreground': 'foreground',
    '--surface-hover': 'fn hover',
    '--surface-secondary': 'surface_secondary',
    '--surface-secondary-foreground': 'surface_secondary_foreground',
    '--surface-tertiary': 'surface_tertiary',
    '--surface-tertiary-foreground': 'surface_tertiary_foreground',
    '--overlay': 'overlay',
    '--overlay-foreground': 'foreground',
    # Fields.
    '--field-background': 'background',
    '--field-foreground': 'foreground',
    '--field-placeholder': 'placeholder',
    '--field-border': 'border',
    '--field-border-focus': 'fn border_focus',
    '--field-border-hover': 'fn border_hover',
    '--field-hover': 'fn hover',
    '--field-focus': 'fn focus',
    '--field-radius': 'field_radius',
    '--field-border-width': 'field_border_width',
    '--field-shadow': 'field_shadow',
    # Layout and motion.
    '--radius': 'radius',
    '--spacing': 'spacing',
    '--border-width': 'border_width',
    '--disabled-opacity': 'disabled_opacity',
    '--ring-offset-width': 'ring_offset_width',
    '--surface-shadow': 'surface_shadow',
    '--overlay-shadow': 'overlay_shadow',
    '--skeleton-animation': 'skeleton_animation',
    # The two delays carry their unit, since they are `u64` milliseconds
    # rather than a CSS duration.
    '--tooltip-delay': 'tooltip_delay_ms',
    '--tooltip-close-delay': 'tooltip_close_delay_ms',
    # Base.
    '--background-inverse': 'background_inverse',
    '--background-secondary': 'background_secondary',
    '--background-tertiary': 'background_tertiary',
    '--border-secondary': 'border_secondary',
    '--border-tertiary': 'border_tertiary',
    '--separator-secondary': 'separator_secondary',
    '--separator-tertiary': 'separator_tertiary',
    '--segment-foreground': 'foreground',
}

# Variables with no place in this port, with the reason.
WONT_EXPOSE = {
    # CSS cursor keywords. gpui sets a cursor per element
    # (`cursor_pointer()` / `CursorStyle::ResizeLeftRight`), so there is no
    # variable to read.
    '--cursor-disabled': 'cursor-is-per-element',
    '--cursor-interactive': 'cursor-is-per-element',
    # The scrollbar is the platform's here: gpui draws its own, and the
    # `scrollbar-width`/`-color`/`-gutter` properties are the browser's own
    # scrollbar styling API.
    '--scrollbar-color': 'no-styleable-scrollbar',
    '--scrollbar-gutter': 'no-styleable-scrollbar',
    '--scrollbar-width': 'no-styleable-scrollbar',
    '--scrollbar-thumb': 'no-styleable-scrollbar',
    '--scrollbar-track': 'no-styleable-scrollbar',
    # Named constants v3 mixes *into* other tokens (`--eclipse` and `--snow` are
    # the two ends of its greyscale, `--black`/`--white` the literals). Every
    # token that uses them is transcribed with the result already mixed, so the
    # constants have no separate life here.
    '--eclipse': 'mixed-into-the-tokens',
    '--snow': 'mixed-into-the-tokens',
    '--black': 'mixed-into-the-tokens',
    '--white': 'mixed-into-the-tokens',
}


# A variable -> where its value lives in `ThemeColors`, for the value pass. A
# flat name map cannot do this: `background` is a field of the theme, of every
# surface and of the field colours.
VALUE_PATH = {
    '--background': 'background',
    '--foreground': 'foreground',
    '--muted': 'muted',
    '--scrollbar': 'scrollbar',
    '--surface': 'surface.background',
    '--surface-foreground': 'surface.foreground',
    '--surface-secondary': 'surface_secondary',
    '--surface-tertiary': 'surface_tertiary',
    '--overlay': 'overlay.background',
    '--overlay-foreground': 'overlay.foreground',
    '--segment': 'segment.background',
    '--segment-foreground': 'segment.foreground',
    '--field-background': 'field.background',
    '--field-foreground': 'field.foreground',
    '--field-placeholder': 'field.placeholder',
    '--border': 'border',
    '--separator': 'separator',
    '--accent': 'accent',
    '--default': 'default',
    '--success': 'success',
    '--warning': 'warning',
    '--danger': 'danger',
    '--link': 'link',
}


# A length or timing -> its field in `LayoutTokens`, with the expected value in
# the unit that field uses. `1rem` is 16px, as everywhere in v3.
LENGTHS = {
    '--radius': ('radius', 8.0),
    '--spacing': ('spacing', 4.0),
    '--border-width': ('border_width', 1.0),
    '--field-radius': ('field_radius', 12.0),
    '--field-border-width': ('field_border_width', 0.0),
    '--ring-offset-width': ('ring_offset_width', 2.0),
    '--disabled-opacity': ('disabled_opacity', 0.5),
    '--tooltip-delay': ('tooltip_delay_ms', 1500.0),
    '--tooltip-close-delay': ('tooltip_close_delay_ms', 500.0),
}

# The three shadow tokens, per appearance: `(x, y, blur, alpha)` per layer.
# Dark mode keeps only the overlay's, and that one is `inset`, which gpui cannot
# paint -- `overlay_hairline` reproduces it as a one-pixel border instead.
SHADOWS = {
    ('light', '--surface-shadow'): 'surface_shadow',
    ('light', '--overlay-shadow'): 'overlay_shadow',
    ('light', '--field-shadow'): 'field_shadow',
}
SHADOW_NOTE = {
    ('dark', '--surface-shadow'): 'transparent -- no shadow',
    ('dark', '--field-shadow'): 'transparent -- no shadow',
    ('dark', '--overlay-shadow'): 'inset -- `overlay_hairline` draws it as a border',
}


def css_length(var, block, light):
    """A `--name`'s value as a number in px, ms or a bare ratio."""
    value = css_value(var, block, light)
    if value is None:
        return None
    m = re.fullmatch(r'calc\(var\((--[a-z-]+)\)\s*\*\s*([0-9.]+)\)', value)
    if m:
        base = css_length(m.group(1), block, light)
        return None if base is None else base * float(m.group(2))
    m = re.fullmatch(r'(-?[0-9.]+)(rem|px|ms|s)?', value)
    if not m:
        return None
    number = float(m.group(1))
    unit = m.group(2)
    if unit == 'rem':
        return number * 16.0
    if unit == 's':
        return number * 1000.0
    return number


def layout_value(field):
    """The number a `LayoutTokens` field is initialised with.

    Comment lines are dropped first: every field is documented with the CSS it
    ports (`/// `--spacing: 0.25rem``), and a search over the whole file finds
    the doc comment before the code.
    """
    whole = io.open(THEME + 'layout.rs', encoding='utf-8', errors='replace').read()
    # Only the constructors: the struct's own declaration (`pub spacing:
    # Pixels,`) matches the same pattern and comes first in the file.
    src = ''
    for ctor in ('common', 'light', 'dark'):
        m = re.search(r'fn %s\(\) -> Self \{(.*?)\n    \}' % ctor, whole, re.S)
        if m:
            src += m.group(1) + '\n'
    src = '\n'.join(l for l in src.split('\n')
                    if not l.lstrip().startswith(('///', '//')))
    m = re.search(r'\b%s:\s*([^,\n]+)' % re.escape(field), src)
    value = m.group(1).strip() if m else None
    if value is None:
        # `let radius = px(8.0);` with `radius,` in the literal -- Rust's
        # field-init shorthand has no colon to match on.
        local = re.search(r'let %s = ([^;]+);' % re.escape(field), src)
        if not local:
            return None
        value = local.group(1).strip()
    m = re.fullmatch(r'px\(([0-9.]+)\)', value)
    if m:
        return float(m.group(1))
    m = re.fullmatch(r'radius \* ([0-9.]+)', value)
    if m:
        base = layout_value('radius')
        return None if base is None else base * float(m.group(1))
    m = re.fullmatch(r'([0-9.]+)', value)
    if m:
        return float(m.group(1))
    return None


def css_shadow(var, block, light):
    """`[(x, y, blur, alpha), ..]` for a box-shadow token."""
    value = css_value(var, block, light)
    if value is None or 'inset' in value or 'transparent' in value:
        return None
    layers = []
    for layer in re.split(r',(?![^()]*\))', value):
        m = re.match(
            r'\s*(-?[0-9.]+)(?:px)?\s+(-?[0-9.]+)(?:px)?\s+(-?[0-9.]+)(?:px)?\s+'
            r'(-?[0-9.]+)(?:px)?\s+'
            r'rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*([0-9.]+)\s*\)', layer)
        if not m:
            return None
        x, y, blur, _spread, alpha = (float(g) for g in m.groups())
        layers.append((x, y, blur, alpha))
    return layers


def rust_shadow(field):
    """The `shadow(x, y, blur, alpha)` layers a field is built from."""
    src = io.open(THEME + 'layout.rs', encoding='utf-8', errors='replace').read()
    m = re.search(r'\b%s:\s*(vec!\[[^\]]*\]|Vec::new\(\))' % re.escape(field), src, re.S)
    if not m:
        return None
    if m.group(1).startswith('Vec::new'):
        return []
    return [tuple(float(n) for n in call.replace(' ', '').split(','))
            for call in re.findall(r'shadow\(([^)]*)\)', m.group(1))]


def css_blocks():
    """`{appearance: text}` for the default theme's two blocks."""
    lines = io.open(CSS, encoding='utf-8', errors='replace').read().split('\n')
    dark = next(i for i, l in enumerate(lines) if l.strip().startswith('.dark,'))
    # The vibrant palette is an opt-in variant, not the default theme.
    vibrant = next(i for i, l in enumerate(lines) if 'data-vibrant-palette' in l)
    return {
        'light': '\n'.join(lines[:dark]),
        'dark': '\n'.join(lines[dark:vibrant]),
    }


def css_value(var, block, light, depth=0):
    """`var`'s value in `block`, following `var(--x)` and the light fallback."""
    if depth > 4:
        return None
    for text in (block, light):
        m = re.search(r'^\s*%s\s*:\s*([^;]+);' % re.escape(var), text, re.M)
        if not m:
            continue
        value = ' '.join(m.group(1).split())
        inner = re.fullmatch(r'var\((--[a-z0-9-]+)\)', value)
        if inner:
            return css_value(inner.group(1), block, light, depth + 1)
        return value
    return None


def oklch(value):
    """The three numbers of an `oklch(..)` literal, or `None`."""
    if value is None:
        return None
    m = re.fullmatch(r'oklch\(([^)]*)\)', value.strip())
    if not m:
        return None
    parts = [p.strip().rstrip('%') for p in m.group(1).replace(',', ' ').split()]
    try:
        nums = [float(p) for p in parts[:3]]
    except ValueError:
        return None
    if len(nums) < 3:
        return None
    # v3 writes lightness as a percentage in some rules and a fraction in others.
    return (nums[0] / 100 if nums[0] > 1.5 else nums[0], nums[1], nums[2])


def rust_body(fn):
    """The body of `ThemeColors::light()` or `::dark()`."""
    src = io.open(THEME + 'semantic.rs', encoding='utf-8', errors='replace').read()
    start = src.index('    pub fn %s() -> Self {' % fn)
    nxt = src.find('\n    pub fn ', start + 30)
    return src[start:nxt if nxt != -1 else len(src)], src


def fields(text):
    """The `name: value` pairs of one struct literal, depth-aware.

    A flat regex cannot do this: `background` is a field of the theme *and* of
    every surface inside it, and the first match is whichever nested one comes
    first in the file.
    """
    out, name, start, depth = {}, None, 0, 0
    i = 0
    while i < len(text):
        ch = text[i]
        if ch in '({[':
            depth += 1
        elif ch in ')}]':
            depth -= 1
            if depth < 0:
                break
        elif (
            depth == 0
            and ch == ':'
            and name is None
            # `::` is a path, not a field: `RoleColor::new(..)` would otherwise
            # read as a field called `RoleColor`.
            and text[i + 1:i + 2] != ':'
            and text[i - 1:i] != ':'
        ):
            back = re.search(r'([A-Za-z_][A-Za-z_0-9]*)\s*$', text[start:i])
            if back:
                name = back.group(1)
                value_start = i + 1
        elif depth == 0 and ch == ',':
            if name is not None:
                out[name] = text[value_start:i].strip()
            else:
                # Rust's field-init shorthand: `foreground,` is a field whose
                # value is the local of the same name. Three of the theme's
                # colours are written that way.
                short = re.search(r'([A-Za-z_][A-Za-z_0-9]*)\s*$', text[start:i])
                if short and 'let ' not in text[start:i]:
                    out.setdefault(short.group(1), short.group(1))
            name, start = None, i + 1
        i += 1
    if name is not None:
        out.setdefault(name, text[value_start:].strip())
    return out


def literal(value, body, src, depth=0):
    """`value` reduced to an `oklch(..)`, following locals and helpers."""
    if value is None or depth > 3:
        return None
    value = value.strip()
    m = re.match(r'oklch\(([^)]*)\)', value)
    if m:
        return 'oklch(%s)' % m.group(1)
    # A role's first argument is its base colour.
    m = re.match(r'RoleColor::new\(\s*(.+)', value, re.S)
    if m:
        return literal(fields('x: ' + m.group(1)).get('x', m.group(1).split(',')[0]),
                       body, src, depth + 1)
    name = value.rstrip('()')
    m = re.search(r'let %s = (.+?);' % re.escape(name), body, re.S)
    if m:
        return literal(m.group(1), body, src, depth + 1)
    m = re.search(r'(?:pub )?fn %s\(\) -> Hsla \{\s*(.+?)\s*\}' % re.escape(name), src, re.S)
    if m:
        return literal(m.group(1), body, src, depth + 1)
    return None


def rust_value(path, fn):
    """The `oklch(..)` a dotted path resolves to."""
    body, src = rust_body(fn)
    # The *literal*, not the signature: `pub fn light() -> Self {` contains the
    # same two words.
    anchor = '\n        Self {'
    start = body.index(anchor) + len(anchor)
    text = body[start:]
    value = None
    for part in path.split('.'):
        table = fields(text)
        if part not in table:
            return None
        value = table[part]
        inner = re.match(r'[A-Za-z_][A-Za-z_0-9]*\s*\{(.*)\}\s*$', value, re.S)
        text = inner.group(1) if inner else value
    return literal(value, body, src)


def declared():
    """Every `--name` v3's default theme declares."""
    text = io.open(CSS, encoding='utf-8', errors='replace').read()
    return sorted(set(re.findall(r'^\s*(--[a-z0-9-]+)\s*:', text, re.M)))


def ours():
    """Every field and accessor name the theme crate exposes."""
    names = set()
    for name in sorted(os.listdir(THEME)):
        if not name.endswith('.rs'):
            continue
        text = io.open(THEME + name, encoding='utf-8', errors='replace').read()
        names |= set(re.findall(r'^\s*pub ([a-z_0-9]+)\s*:', text, re.M))
        names |= set('fn ' + m for m in re.findall(r'pub fn ([a-z_0-9]+)\s*\(', text))
    return names


def main():
    if not os.path.exists(CSS):
        print('no variables.css cached -- run `python .shots/design_audit.py --fetch`')
        return
    have = ours()
    total = exposed = excused = 0
    missing = []
    by_reason = {}
    for var in declared():
        total += 1
        if var in WONT_EXPOSE:
            excused += 1
            reason = WONT_EXPOSE[var]
            by_reason[reason] = by_reason.get(reason, 0) + 1
            continue
        # The obvious spelling, then the alias.
        # A token is exposed either as a field or as a derived accessor: most of
        # v3's `color-mix` variables are computed here rather than stored, which
        # is what keeps them from drifting out of step with what they mix.
        rust = var.lstrip('-').replace('-', '_')
        if rust in have or ('fn ' + rust) in have or ALIAS.get(var) in have:
            exposed += 1
        else:
            missing.append('%-32s looked for `%s`%s' % (
                var, rust,
                '' if var not in ALIAS else ' or `%s`' % ALIAS[var]))

    # --- second pass: the values ------------------------------------------
    blocks = css_blocks()
    compared = derived = 0
    wrong = []
    for appearance, block in blocks.items():
        for var, path in sorted(VALUE_PATH.items()):
            theirs = oklch(css_value(var, block, blocks['light']))
            if theirs is None:
                derived += 1
                continue
            ours_value = oklch(rust_value(path, appearance))
            if ours_value is None:
                wrong.append('%-6s %-24s %s does not resolve to a literal'
                             % (appearance, var, path))
                continue
            compared += 1
            if any(abs(a - b) > 0.0006 for a, b in zip(theirs, ours_value)):
                wrong.append('%-6s %-24s v3=%s ours=%s' % (appearance, var, theirs, ours_value))

    # --- third pass: the lengths, timings and shadows ----------------------
    lengths = shadows = 0
    for var, (field, _expected) in sorted(LENGTHS.items()):
        theirs = css_length(var, blocks['light'], blocks['light'])
        ours_number = layout_value(field)
        if theirs is None or ours_number is None:
            wrong.append('layout %-24s %s does not resolve to a number' % (var, field))
            continue
        lengths += 1
        if abs(theirs - ours_number) > 0.001:
            wrong.append('layout %-24s v3=%s ours=%s' % (var, theirs, ours_number))

    for (appearance, var), field in sorted(SHADOWS.items()):
        theirs = css_shadow(var, blocks[appearance], blocks['light'])
        ours_layers = rust_shadow(field)
        if theirs is None or ours_layers is None:
            wrong.append('%-6s %-24s %s does not resolve to layers'
                         % (appearance, var, field))
            continue
        shadows += 1
        if len(theirs) != len(ours_layers) or any(
                any(abs(a - b) > 0.001 for a, b in zip(t, o))
                for t, o in zip(theirs, ours_layers)):
            wrong.append('%-6s %-24s v3=%s ours=%s' % (appearance, var, theirs, ours_layers))

    for line in wrong:
        print('WRONG    ' + line)

    for line in missing:
        print('MISSING  ' + line)
    print()
    print('variables declared : %d' % total)
    print('exposed            : %d' % exposed)
    print('not exposed here   : %d' % excused)
    for reason, n in sorted(by_reason.items(), key=lambda kv: (-kv[1], kv[0])):
        print('    %-24s %d' % (reason, n))
    print('MISSING            : %d' % len(missing))
    print('values compared    : %d  (%d derived, checked by semantic.rs tests)'
          % (compared, derived))
    print('lengths compared   : %d' % lengths)
    print('shadows compared   : %d  (%d dark ones are inset or none -- see SHADOW_NOTE)'
          % (shadows, len(SHADOW_NOTE)))
    print('WRONG VALUES       : %d' % len(wrong))


if __name__ == '__main__':
    main()
