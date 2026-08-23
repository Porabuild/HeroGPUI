"""Five transcribed token values that did not match v3."""
import io

P = 'crates/herogpui-theme/src/semantic.rs'
s = io.open(P, encoding='utf-8', newline='').read().replace('\r\n', '\n')


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# --- light: `--border: oklch(90% 0.004 286.32)` ---------------------------
rep("""            border: oklch(0.92, 0.004, 286.32),
            separator: oklch(0.92, 0.004, 286.32),""",
    """            // `--border` is a step darker than `--separator`: 90% against 92%.
            // Both were transcribed as the separator's value.
            border: oklch(0.9, 0.004, 286.32),
            separator: oklch(0.92, 0.004, 286.32),""")

# --- dark: border, separator, overlay, field background -------------------
rep("""            border: oklch(0.22, 0.006, 286.033),
            separator: oklch(0.22, 0.006, 286.033),""",
    """            // `--border: oklch(28% ..)` and `--separator: oklch(25% ..)`; both
            // were a single value, and both too dark.
            border: oklch(0.28, 0.006, 286.033),
            separator: oklch(0.25, 0.006, 286.033),""")

rep("""            // Slightly lighter than surface so floating panels read in dark mode.
            overlay: SurfaceColor {
                background: oklch(0.22, 0.0059, 285.89),
                foreground,
            },""",
    """            // `--overlay` *is* `--surface` in dark mode. This used to lighten it
            // "so floating panels read", which is exactly the kind of
            // improvement the token values are not allowed to make: a v3 dark
            // popover is the same colour as a v3 dark card, and the shadow is
            // what separates them.
            overlay: SurfaceColor {
                background: oklch(0.2103, 0.0059, 285.89),
                foreground,
            },""")

rep("""            field: FieldColors {
                background: default.color,
                foreground,
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            border: oklch(0.28, 0.006, 286.033),""",
    """            field: FieldColors {
                // `--field-background: oklch(0.2103 0.0059 285.89)` -- the
                // surface colour, not `--default`, which is two steps lighter.
                background: oklch(0.2103, 0.0059, 285.89),
                foreground,
                placeholder: muted,
                border: with_alpha(black(), 0.0),
            },

            border: oklch(0.28, 0.006, 286.033),""")

# The test asserting the deviation goes with it.
rep("""    #[test]
    fn dark_surface_is_darker_than_overlay() {""",
    """    #[test]
    #[ignore = "v3 gives dark mode the same colour for both; the shadow separates them"]
    fn dark_surface_is_darker_than_overlay() {""")

io.open(P, 'w', encoding='utf-8', newline='\n').write(s)
print('token values corrected')
