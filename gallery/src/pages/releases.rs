use gpui::{prelude::*, px, AnyElement, Context};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::Page;

impl Gallery {
    pub fn page_releases(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let colors = cx.colors();
        gpui::div()
            .w(px(860.))
            .max_w(gpui::relative(1.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                gpui::div()
                    .text_size(px(30.))
                    .line_height(px(38.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(Page::Releases.title()),
            )
            .child(
                gpui::div()
                    .text_size(px(15.5))
                    .line_height(px(26.))
                    .text_color(colors.muted)
                    .child(Page::Releases.description()),
            )
            .mt(px(8.))
            .child(
                gpui::div()
                    .text_size(px(20.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("Current Development"),
            )
            .child(release_card(cx))
            .into_any_element()
    }
}

fn release_card(cx: &Context<'_, Gallery>) -> AnyElement {
    let colors = cx.colors();
    gpui::div()
        .w_full()
        .p(px(22.))
        .rounded(px(14.))
        .border_1()
        .border_color(colors.border)
        .bg(colors.surface.background)
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(
            gpui::div()
                .flex()
                .items_center()
                .child(
                    gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            gpui::div()
                                .text_size(px(22.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                        )
                        .child(
                            gpui::div()
                                .px(px(8.))
                                .py(px(3.))
                                .rounded_full()
                                .bg(colors.accent.soft())
                                .text_size(px(11.))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(colors.accent.color)
                                .child("Workspace version"),
                        ),
                ),
        )
        .child(
            gpui::div()
                .text_size(px(14.))
                .line_height(px(23.))
                .text_color(colors.muted)
                .child(
                    "The current development line delivers the HeroUI v3 component system as native GPUI builders, with theme parity and live documentation.",
                ),
        )
        .child(
            gpui::div()
                .pt(px(4.))
                .border_t_1()
                .border_color(colors.separator)
                .flex()
                .flex_col()
                .gap(px(10.))
                .children([
                    release_item("Component atlas", "Browse the full library by category and jump directly into a live example."),
                    release_item("Source-backed references", "Example source, builder signatures, defaults and composition callbacks stay aligned with the Rust implementation."),
                    release_item("Native verification", "Every documented route participates in the gallery's build, lint and runtime rendering gates."),
                ]),
        )
        .into_any_element()
}

fn release_item(title: &str, detail: &str) -> gpui::Div {
    gpui::div()
        .flex()
        .items_start()
        .gap(px(10.))
        .child(gpui::div().child("•"))
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    gpui::div()
                        .text_size(px(13.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(title.to_owned()),
                )
                .child(gpui::div().text_size(px(13.)).child(detail.to_owned())),
        )
}
