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

    for line in missing:
        print('MISSING  ' + line)
    print()
    print('variables declared : %d' % total)
    print('exposed            : %d' % exposed)
    print('not exposed here   : %d' % excused)
    for reason, n in sorted(by_reason.items(), key=lambda kv: (-kv[1], kv[0])):
        print('    %-24s %d' % (reason, n))
    print('MISSING            : %d' % len(missing))


if __name__ == '__main__':
    main()
