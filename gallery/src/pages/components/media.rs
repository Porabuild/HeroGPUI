//! Media gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Media
    // -----------------------------------------------------------------------

    pub fn page_avatar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Avatar",
            crate::pages::Page::Avatar.description(),
            crate::pages::Page::Avatar.import_line(),
            vec![
                (
                    "Usage",
                    "Fallback text uses 14px/20px, 12px/16px for small and 16px/24px for large avatars. Here `radius` sets 8px corners and `sx` squares just the top-left corner on the fallback and loaded image.",
                    specimen_body("avatar-main", row(vec![h::Avatar::new("usage-avatar")
                        .name("Jane Doe")
                        .radius(px(8.))
                        .sx(|el| el.rounded_tl(px(0.)))
                        .into_any_element()]), cx),
                ),
                (
                    "Fallback Content",
                    specimen_body("avatar-fallback", spec_row(vec![
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
                    ]), cx),
                ),
                (
                    "Sizes",
                    specimen_body("avatar-sizes", spec_row(
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
                    ), cx),
                ),
                (
                    "Colors",
                    specimen_body("avatar-colors", row(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(i, c)| h::Avatar::new(("color-avatar", i)).name("HG").color(*c))
                        .els()), cx),
                ),
                (
                    "Variants",
                    specimen_body("avatar-variants", spec_row(
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
                    ), cx),
                ),
                (
                    "Custom Image Component", "A custom image source composes the same way: the loader below supplies the embedded sample image itself, and `on_load` fires once the image is ready and replaces the fallback.",
                    specimen_body("avatar-custom-image", col(vec![
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
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_avatar_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        const USERS: [&str; 5] = [
            "John Doe",
            "Kate Wilson",
            "Emily Chen",
            "Michael Brown",
            "Olivia Davis",
        ];
        // One avatar per user, with an id unique to the example it sits in.
        fn users(example: &'static str, take: usize) -> Vec<h::Avatar> {
            USERS
                .iter()
                .take(take)
                .enumerate()
                .map(|(i, name)| h::Avatar::new((example, i)).name(*name))
                .collect()
        }
        // v3's Overlap example mixes image and fallback avatars.
        fn overlap_group(id: &'static str, overlap: h::AvatarGroupOverlap) -> h::AvatarGroup {
            h::AvatarGroup::new(id)
                .overlap(overlap)
                .size(Size::Lg)
                .child(
                    h::Avatar::new((id, 0usize))
                        .src(sample_avatar_source())
                        .name("JD"),
                )
                .child(h::Avatar::new((id, 1usize)).name("AB"))
                .child(
                    h::Avatar::new((id, 2usize))
                        .src(sample_avatar_source())
                        .name("EC"),
                )
                .child(h::Avatar::new((id, 3usize)).name("SM"))
                .count(h::AvatarGroupCount::new((id, 4usize)).child("+2"))
        }
        let colors = cx.colors();
        component_doc_page!(
            "AvatarGroup",
            crate::pages::Page::AvatarGroup.description(),
            crate::pages::Page::AvatarGroup.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("avatar-group-main", row(vec![h::AvatarGroup::new("ag-basic")
                        .children(users("ag-basic-user", 4))
                        .into_any_element()]), cx),
                ),
                (
                    "Max",
                    specimen_body("avatar-group-max", row(vec![h::AvatarGroup::new("ag-max")
                        .max(3)
                        .children(users("ag-max-user", 5))
                        .into_any_element()]), cx),
                ),
                (
                    "With Count",
                    specimen_body("avatar-group-count", row(vec![h::AvatarGroup::new("ag-count")
                        .size(Size::Sm)
                        .children(users("ag-count-user", 3))
                        .count(h::AvatarGroupCount::new("ag-count-total").child(format!("+{}", 12 - 3)))
                        .into_any_element()]), cx),
                ),
                (
                    "Sizes",
                    specimen_body("avatar-group-sizes", col(vec![
                        spec(
                            "Small",
                            h::AvatarGroup::new("ag-size-sm")
                                .size(Size::Sm)
                                .children(users("ag-size-sm-user", 4)),
                            cx,
                        ),
                        spec(
                            "Medium (default)",
                            h::AvatarGroup::new("ag-size-md")
                                .size(Size::Md)
                                .children(users("ag-size-md-user", 4)),
                            cx,
                        ),
                        spec(
                            "Large",
                            h::AvatarGroup::new("ag-size-lg")
                                .size(Size::Lg)
                                .children(users("ag-size-lg-user", 4)),
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Grid",
                    specimen_body("avatar-group-grid", row(vec![h::AvatarGroup::new("ag-grid")
                        .is_grid(true)
                        .max(5)
                        .children(users("ag-grid-user", 5))
                        .into_any_element()]), cx),
                ),
                (
                    "Overlap", "`ring` is a solid 2px outline in the background color. v3's default `clip` cuts a transparent crescent, which gpui cannot mask; the port paints that crescent in the background color, so both read the same on a solid background.",
                    specimen_body("avatar-group-overlap", col(vec![
                        spec("clip", overlap_group("ag-overlap-clip", h::AvatarGroupOverlap::Clip), cx),
                        spec("ring", overlap_group("ag-overlap-ring", h::AvatarGroupOverlap::Ring), cx),
                    ]), cx),
                ),
                (
                    "Custom Styles", "`overlap_distance` and `seam` stand in for the `--avatar-group-overlap` and `--avatar-group-seam` custom properties.",
                    specimen_body("avatar-group-custom-styles", row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .rounded_full()
                        .border_1()
                        .border_color(colors.border)
                        .bg(colors.surface.background)
                        .py(px(4.))
                        .pl(px(4.))
                        .pr(px(12.))
                        .shadow_sm()
                        .child(
                            h::AvatarGroup::new("ag-custom")
                                .overlap(h::AvatarGroupOverlap::Clip)
                                .size(Size::Sm)
                                .overlap_distance(px(11.2))
                                .seam(px(2.))
                                .children(users("ag-custom-user", 3))
                                .child(
                                    h::Avatar::new("ag-custom-icon")
                                        .fallback(icon(h::icons::HEART_FILL, cx)),
                                )
                                .count(h::AvatarGroupCount::new("ag-custom-count").child("+3")),
                        )
                        .child(
                            gpui::div()
                                .text_size(px(14.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(colors.foreground)
                                .child("Assignees"),
                        )
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
