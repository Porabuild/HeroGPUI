"""Accordion: v3's shape. `.accordion` is `w-full` and nothing else.

Only `--surface` carries a background and a radius; the default variant is
flush with the page. The trigger is `px-4 py-4`, and the separator is a 1px
line at the item's bottom -- inset to 3%/94% in the surface variant.
"""
import io

P = 'crates/herogpui-components/src/accordion.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""        // `default` is flush with the page; `surface` lifts the whole group
        // onto a surface with the surface shadow.
        let mut container = match self.variant {
            AccordionVariant::Default => gpui::div(),
            AccordionVariant::Surface => gpui::div()
                .bg(colors.surface.background)
                .border(layout.border_width)
                .border_color(colors.border)
                .rounded(crate::util::container_radius(cx))
                .when(!layout.surface_shadow.is_empty(), |e| {
                    e.shadow(layout.surface_shadow.clone())
                }),
        };

        container = container
            .flex()
            .flex_col()
            .rounded(crate::util::control_radius(cx))
            .bg(colors.surface.background)
            .overflow_hidden();""",
    """        // `.accordion` is `w-full` and nothing else: the default variant is
        // flush with the page. Only `.accordion--surface` paints a background
        // and rounds the group. This used to give both variants a rounded white
        // card, so `variant` made no visible difference.
        let mut container = match self.variant {
            AccordionVariant::Default => gpui::div(),
            AccordionVariant::Surface => gpui::div()
                .bg(colors.surface.background)
                .rounded(crate::util::container_radius(cx))
                .overflow_hidden()
                .when(!layout.surface_shadow.is_empty(), |e| {
                    e.shadow(layout.surface_shadow.clone())
                }),
        };

        container = container.w_full().flex().flex_col();""")

rep("""                .px(px(12.))
                .py(px(10.));""",
    """                // `.accordion__trigger` is `px-4 py-4`.
                .px(px(16.))
                .py(px(16.));""")

rep("""                section = section.child(
                    gpui::div()
                        .px(px(12.))
                        .pb(px(12.))
                        .pt(px(2.))""",
    """                section = section.child(
                    gpui::div()
                        .px(px(16.))
                        .pb(px(16.))
                        .pt(px(2.))""")

rep("""            section = section.when(!self.hide_separator && i + 1 < count, |s| {
                s.border_b_1().border_color(colors.separator)
            });""",
    """            // `.accordion__item::after` is a 1px `bg-separator` line at the
            // bottom of every item but the last, inset to 3%/94% on a surface.
            let inset = self.variant == AccordionVariant::Surface;
            section = section.when(!self.hide_separator && i + 1 < count, |s| {
                s.child(
                    gpui::div()
                        .h(px(1.))
                        .rounded(crate::util::hairline_radius(cx))
                        .bg(colors.separator)
                        .map(|line| {
                            if inset {
                                line.mx(gpui::relative(0.03)).w(gpui::relative(0.94))
                            } else {
                                line.w_full()
                            }
                        }),
                )
            });""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched accordion')
