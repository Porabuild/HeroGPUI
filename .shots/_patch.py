import io

p = 'crates/herogpui-components/src/disclosure.rs'
s = io.open(p, encoding='utf-8').read()

old = """impl RenderOnce for Disclosure {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let item = AccordionItem::new("disclosure", self.title.clone()).content(
            gpui::div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .children(self.children),
        );
        let mut set = HashSet::new();
        if self.is_expanded {
            set.insert(SharedString::from("disclosure"));
        }
        let was_expanded = self.is_expanded;
        Accordion::new(vec![item])
            .expanded_keys(set)
            .variant(AccordionVariant::Surface)
            .is_disabled(self.is_disabled)
            .on_toggle({
                let cb = self.on_toggle.clone();
                move |_k, w, cx| {
                    if let Some(f) = &cb {
                        f(!was_expanded, w, cx);
                    }
                }
            })
            .into_any_element()
    }
}"""
new = """impl RenderOnce for Disclosure {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A v3 Disclosure is not a small Accordion: `.disclosure` is `relative`,
        // its trigger is whatever the caller passes (`<Button slot="trigger">`
        // in v3's own examples) with a `.disclosure__indicator` chevron, and
        // `.disclosure__body` is `p-2`. Rendering it as a one-item accordion
        // gave it a card, a 16px trigger row and a separator that v3's sheet
        // has no rule for.
        let expanded = self.is_expanded;
        let cb = self.on_toggle.clone();
        let trigger = crate::button::Button::new(gpui::ElementId::Name(
            format!("{}-disclosure-trigger", self.title).into(),
        ))
        .variant(if expanded {
            herogpui_core::Variant::Secondary
        } else {
            herogpui_core::Variant::Tertiary
        })
        .label(self.title.clone())
        .is_disabled(self.is_disabled)
        // `.disclosure__indicator` is `ms-auto size-4` and turns 180 degrees
        // when the panel is open, which is a glyph swap here.
        .end_content(
            gpui::svg()
                .size(px(16.))
                .path(if expanded {
                    crate::icons::CHEVRON_UP
                } else {
                    crate::icons::CHEVRON_DOWN
                })
                .text_color(cx.colors().muted)
                .into_any_element(),
        )
        .on_press(move |_, w, cx| {
            if let Some(f) = &cb {
                f(!expanded, w, cx);
            }
        });

        let mut el = gpui::div().relative().flex().flex_col().child(trigger);
        // v3 keeps the panel mounted and animates its height; `overlay_phase`
        // is what gives a collapsing body its exit here.
        if expanded {
            el = el.child(
                crate::anim::entering(
                    gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{}-disclosure-body", self.title).into(),
                        ))
                        // `.disclosure__body` is `p-2`.
                        .p(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .children(self.children),
                    "disclosure-body",
                    crate::anim::Motion::DISCLOSURE,
                    cx,
                )
                .into_any_element(),
            );
        }
        let _ = window;
        el.into_any_element()
    }
}"""
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)
print('ok')
