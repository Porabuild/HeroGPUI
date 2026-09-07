//! Utilities gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Utilities
    // -----------------------------------------------------------------------

    pub fn page_scroll_shadow(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let lines: Vec<AnyElement> = (1..=20)
            .map(|i| {
                gpui::div()
                    .py(px(4.))
                    .child(format!("Row {i}"))
                    .into_any_element()
            })
            .collect();
        component_doc_page!(
            "Scroll Shadow",
            crate::pages::Page::ScrollShadow.description(),
            crate::pages::Page::ScrollShadow.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::ScrollShadow::new("ss-usage")
                        .max_h(px(180.))
                        .children((1..=14).map(|n| {
                            gpui::div().py(px(6.)).child(format!("Row {n} of fourteen"))
                        }),)
                        .into_any_element()]),
                ),
                (
                    "Orientation",
                    col(vec![
                        spec(
                            "Vertical",
                            h::ScrollShadow::new("ss-or-v").max_h(px(140.)).children(
                                (1..=10).map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                            ),
                            cx,
                        ),
                        spec(
                            "Horizontal",
                            h::ScrollShadow::new("ss-or-h")
                                .orientation(Orientation::Horizontal)
                                .max_w(px(360.))
                                .gap(px(12.))
                                .children((1..=12).map(|n| {
                                    gpui::div()
                                        .flex_shrink_0()
                                        .w(px(90.))
                                        .h(px(60.))
                                        .rounded(px(10.))
                                        .bg(cx.colors().default.color)
                                        .child(format!("{n}"))
                                })),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Shadow Size",
                    col(vec![
                        spec(
                            "8px",
                            h::ScrollShadow::new("ss-size-sm")
                                .size(px(8.))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "40px",
                            h::ScrollShadow::new("ss-size-lg")
                                .size(px(40.))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Card",
                    col(vec![h::Card::new()
                        .w(px(320.))
                        .child(
                            h::CardHeader::new().child(h::CardTitle::new().child("Release notes"))
                        )
                        .child(h::CardContent::new().child(
                            h::ScrollShadow::new("ss-card").max_h(px(140.)).children(
                                (1..=12).map(|n| {
                                    gpui::div().py(px(6.)).child(format!("Change {n}"))
                                }),
                            ),
                        ),)
                        .into_any_element()]),
                ),
                (
                    "Hide Scroll Bar", "gpui draws no scrollbar inside a scroll container, so this is the default rather than a prop: the shadows are the only affordance.",
                    col(vec![
                        h::ScrollShadow::new("ss-no-bar")
                            .max_h(px(140.))
                            .children(
                                (1..=12).map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Visibility Change",
                    col(vec![
                        spec(
                            "Auto (follows the scroll position)",
                            h::ScrollShadow::new("ss-vis-auto")
                                .visibility(h::ScrollShadowVisibility::Auto)
                                // `onVisibilityChange` fires when the shaded
                                // edges change, which `Auto` does as it scrolls.
                                .on_visibility_change(cx.listener(
                                    |this, visibility: &h::ScrollShadowVisibility, _, cx| {
                                        this.set_demo_text_value(
                                            "ss-visibility",
                                            format!("{visibility:?}"),
                                        );
                                        cx.notify();
                                    },
                                ))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "Both edges, always",
                            h::ScrollShadow::new("ss-vis-both")
                                .visibility(h::ScrollShadowVisibility::Both)
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "Top only",
                            h::ScrollShadow::new("ss-vis-top")
                                .visibility(h::ScrollShadowVisibility::Top)
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![h::ScrollShadow::new("ss-main")
                        .max_h(px(200.))
                        .children(lines)
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    "Scroll sideways to reveal more cards. Vertical wheel input continues to the page.",
                    col(vec![h::ScrollShadow::new("ss-h")
                        .orientation(Orientation::Horizontal)
                        .max_w(px(520.))
                        .size(px(56.))
                        .children((1..=14).map(|i| {
                            gpui::div()
                                .flex_shrink_0()
                                .px(px(16.))
                                .py(px(10.))
                                .rounded(px(10.))
                                .bg(cx.colors().surface_secondary)
                                .child(format!("Card {i}"))
                                .into_any_element()
                        }))
                        .into_any_element()]),
                ),
                (
                    "Shadows disabled",
                    col(vec![h::ScrollShadow::new("ss-off")
                        .max_h(px(140.))
                        .is_enabled(false)
                        .children((1..=10).map(|i| {
                            gpui::div()
                                .py(px(4.))
                                .child(format!("Row {i}"))
                                .into_any_element()
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }
}
