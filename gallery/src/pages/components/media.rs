//! Media gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Media
    // -----------------------------------------------------------------------

    pub fn page_avatar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        // v3 has no AvatarGroup: its Avatar Group example composes ordinary
        // avatars with layout CSS — a `-space-x-2` overlap and a
        // `ring-2 ring-background` ring on every member — and renders the
        // overflow counter as a plain fallback avatar with `text-xs`.
        fn member(el: impl IntoElement, ring: gpui::Hsla) -> gpui::Div {
            gpui::div()
                .border_2()
                .border_color(ring)
                .rounded_full()
                .child(el)
        }
        let ring = cx.colors().background;
        let overlap = |d: gpui::Div| d.ml(px(-8.)).into_any_element();
        let names = [
            "Ada Lovelace",
            "Grace Hopper",
            "Alan Turing",
            "Katherine Johnson",
            "Margaret Hamilton",
        ];
        // `-space-x-2`: only subsequent siblings get the -8px margin.
        let mut counter_members: Vec<AnyElement> = names
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, n)| {
                let d = member(h::Avatar::new(("counter-member", i)).name(*n), ring);
                if i == 0 {
                    d.into_any_element()
                } else {
                    overlap(d)
                }
            })
            .collect();
        counter_members.push(overlap(member(
            gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(40.))
                .rounded_full()
                .bg(cx.colors().surface_tertiary)
                .text_color(cx.colors().foreground)
                .text_size(px(12.))
                .line_height(px(16.))
                .child(format!("+{}", names.len() - 3)),
            ring,
        )));
        component_doc_page!(
            "Avatar",
            crate::pages::Page::Avatar.description(),
            crate::pages::Page::Avatar.import_line(),
            vec![
                (
                    "Usage",
                    "Fallback text uses 14px text with 20px lines, or 16px text with 24px lines for large avatars.",
                    row(vec![h::Avatar::new("usage-avatar")
                        .name("Jane Doe")
                        .into_any_element()]),
                ),
                (
                    "Fallback Content",
                    spec_row(vec![
                        spec(
                            "Initials",
                            h::Avatar::new("initials-avatar").name("Jane Doe"),
                            cx,
                        ),
                        spec("No name", h::Avatar::new("unnamed-avatar"), cx),
                        // v3's own Fallback Content example drives a
                        // deliberately broken URL with
                        // `<Avatar.Fallback delayMs={600}>`; an unregistered
                        // asset path fails identically here (no network), and
                        // the initials replace the box once the delay elapses.
                        spec(
                            "Broken image",
                            h::Avatar::new("delay-avatar")
                                .name("NA")
                                .src("images/avatar-broken.png")
                                .delay_ms(600),
                            cx,
                        ),
                        spec(
                            "Custom fallback",
                            h::Avatar::new("icon-avatar")
                                .name("HG")
                                .fallback(icon(h::icons::HEART_FILL, cx)),
                            cx,
                        ),
                        spec(
                            "Fallback color",
                            h::Avatar::new("fb-color-avatar")
                                .name("HG")
                                .color(Color::Accent)
                                .variant(h::AvatarVariant::Soft)
                                .fallback_color(Color::Warning),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Sizes",
                    spec_row(
                        Size::ALL
                            .iter()
                            .enumerate()
                            .map(|(i, s)| {
                                spec(
                                    s.label(),
                                    h::Avatar::new(("size-avatar", i))
                                        .name("Ada Lovelace")
                                        .size(*s),
                                    cx,
                                )
                            })
                            .collect()
                    ),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(i, c)| h::Avatar::new(("color-avatar", i)).name("HG").color(*c))
                        .els()),
                ),
                (
                    "Variants",
                    spec_row(
                        h::AvatarVariant::ALL
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                spec(
                                    v.label(),
                                    h::Avatar::new(("variant-avatar", i))
                                        .name("HG")
                                        .color(Color::Accent)
                                        .variant(*v),
                                    cx,
                                )
                            })
                            .collect()
                    ),
                ),
                (
                    "Avatar Group",
                    row(vec![gpui::div()
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap(px(24.))
                        .child(
                            // Basic group: the first four users overlap by 8px.
                            gpui::div()
                                .flex()
                                .children(names.iter().take(4).enumerate().map(|(i, n)| {
                                    let d =
                                        member(h::Avatar::new(("group-member", i)).name(*n), ring);
                                    if i == 0 {
                                        d.into_any_element()
                                    } else {
                                        overlap(d)
                                    }
                                }),),
                        )
                        .child(
                            // Counter group: three members plus the "+N"
                            // fallback avatar, as v3's second row does.
                            gpui::div().flex().children(counter_members),
                        )
                        .into_any_element()]),
                ),
                (
                    "Custom Image Component", "A custom image source composes the same way: the loader below supplies the embedded sample image itself, and `on_load` fires once the image is ready and replaces the fallback.",
                    col(vec![
                        spec(
                            "Custom loader",
                            h::Avatar::new("custom-loader-avatar")
                                .name("JD")
                                .src(sample_avatar_source())
                                .fallback("JD")
                                .on_load(|_, cx| {
                                    h::Toast::new("Avatar image loaded")
                                        .description("on_load fired once for the custom source.")
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                }),
                            cx,
                        ),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
