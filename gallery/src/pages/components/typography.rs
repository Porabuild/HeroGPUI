//! Typography gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Typography
    // -----------------------------------------------------------------------

    pub fn page_kbd(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Kbd",
            crate::pages::Page::Kbd.description(),
            crate::pages::Page::Kbd.import_line(),
            vec![
                (
                    "Navigation Keys",
                    row(vec![
                        h::Kbd::new()
                            .radius(px(3.))
                            .child("\u{2190}")
                            .into_any_element(),
                        h::Kbd::new()
                            .radius(px(3.))
                            .child("\u{2192}")
                            .into_any_element(),
                        h::Kbd::new()
                            .radius(px(3.))
                            .child("\u{2191}")
                            .into_any_element(),
                        h::Kbd::new()
                            .radius(px(3.))
                            .child("\u{2193}")
                            .into_any_element(),
                        h::Kbd::new()
                            .radius(px(3.))
                            .child("Home")
                            .into_any_element(),
                        h::Kbd::new().radius(px(3.)).child("End").into_any_element(),
                    ]),
                ),
                (
                    "Special Keys",
                    row(vec![
                        h::Kbd::new().child("\u{21e7}").into_any_element(),
                        h::Kbd::new().child("\u{2318}").into_any_element(),
                        h::Kbd::new().child("\u{2325}").into_any_element(),
                        h::Kbd::new().child("\u{21b5}").into_any_element(),
                        h::Kbd::new().child("\u{232b}").into_any_element(),
                        h::Kbd::new().child("Esc").into_any_element(),
                    ]),
                ),
                (
                    "Inline Usage",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(14.))
                        .child("Press")
                        .child(h::Kbd::new().child("Ctrl"))
                        .child(h::Kbd::new().child("K"))
                        .child("to open the command palette.")
                        .into_any_element()]),
                ),
                (
                    "Instructional Text",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(13.5))
                        .text_color(cx.colors().muted)
                        .child("Save with")
                        .child(
                            h::Kbd::new()
                                .variant(h::KbdVariant::Light)
                                .child("\u{2318}"),
                        )
                        .child(h::Kbd::new().variant(h::KbdVariant::Light).child("S"))
                        .into_any_element()]),
                ),
                (
                    "Usage",
                    row(vec![
                        h::Kbd::new().child("Ctrl").into_any_element(),
                        h::Kbd::new().child("Shift").into_any_element(),
                        h::Kbd::new().child("K").into_any_element(),
                    ]),
                ),
                (
                    "Variants",
                    row(h::KbdVariant::ALL
                        .iter()
                        .map(|v| spec(v.label(), h::Kbd::new().variant(*v).child("Esc"), cx))
                        .collect()),
                ),
            ],
            cx,
        )
    }

    pub fn page_typography(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let scale = [
            (h::TypographyType::H1, "h1", "36 / 600"),
            (h::TypographyType::H2, "h2", "30 / 600"),
            (h::TypographyType::H3, "h3", "24 / 600"),
            (h::TypographyType::H4, "h4", "20 / 600"),
            (h::TypographyType::H5, "h5", "18 / 600"),
            (h::TypographyType::H6, "h6", "16 / 600"),
            (h::TypographyType::Body, "body", "16 / 400"),
            (h::TypographyType::BodySm, "body-sm", "14 / 400"),
            (h::TypographyType::BodyXs, "body-xs", "12 / 400"),
            (h::TypographyType::Code, "code", "14 / mono"),
        ];
        let muted = cx.colors().muted;
        component_doc_page!(
            "Typography",
            crate::pages::Page::Typography.description(),
            crate::pages::Page::Typography.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Typography::new(
                        "Typography sets the size, weight and line height of a run of text.",
                    )
                    .into_any_element()]),
                ),
                (
                    "Render Props", "`Prose` provides only the `text-foreground` color. GPUI has no per-tag CSS selectors, so the per-tag descendant styles of a typographic stylesheet — `h1`–`h6`, `p`, `code`, `a`, lists — cannot be inherited; children must already be semantic elements.",
                    col(vec![
                        h::Prose::new()
                            .child(h::Typography::paragraph(
                                h::ParagraphSize::Base,
                                "A paragraph inside `Prose` carries its own body metrics.",
                            ))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Scale",
                    col(scale
                        .iter()
                        .map(|(kind, name, meta)| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(24.))
                                .child(
                                    gpui::div()
                                        .w(px(110.))
                                        .flex()
                                        .flex_col()
                                        .child(
                                            gpui::div()
                                                .text_size(px(13.))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(name.to_string()),
                                        )
                                        .child(
                                            gpui::div()
                                                .text_size(px(11.))
                                                .text_color(muted)
                                                .child(meta.to_string()),
                                        ),
                                )
                                .child(
                                    h::Typography::new("Build better interfaces")
                                        .kind(*kind)
                                        .font_family(crate::app::MONO_FONT),
                                )
                        })
                        .els()),
                ),
                (
                    "Colors & weights",
                    col(vec![
                        h::Typography::new("Default foreground").into_any_element(),
                        h::Typography::new("Muted foreground")
                            .color(h::TextColor::Muted)
                            .into_any_element(),
                        h::Typography::new("Medium body")
                            .weight(h::FontWeight::Medium)
                            .into_any_element(),
                        h::Typography::new("Semibold body")
                            .weight(h::FontWeight::Semibold)
                            .into_any_element(),
                        h::Typography::new("Bold body")
                            .weight(h::FontWeight::Bold)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Alignment & truncation",
                    col(vec![
                        h::Typography::new("Centered against the measure")
                            .align(h::TextAlign::Center)
                            .into_any_element(),
                        h::Typography::new("Aligned to the end")
                            .align(h::TextAlign::End)
                            .into_any_element(),
                        h::Typography::new(
                            "Justify falls back to start alignment: gpui has no text-justify.",
                        )
                        .align(h::TextAlign::Justify)
                        .into_any_element(),
                        gpui::div()
                            .w(px(220.))
                            .child(
                                h::Typography::new(
                                    "A long line that truncates to a single ellipsis at the \
                                     edge of its 220px box instead of wrapping.",
                                )
                                .truncate(true),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Primitives",
                    col(vec![
                        h::Typography::heading(3, "Dashboard").into_any_element(),
                        h::Typography::paragraph(
                            h::ParagraphSize::Base,
                            "Paragraph supports base, sm and xs sizes.",
                        )
                        .into_any_element(),
                        // `radius(px)` rounds the `Code` chip alone; the other
                        // kinds paint no box.
                        h::Typography::code("cargo add herogpui")
                            .radius(px(2.))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Prose",
                    col(vec![h::Prose::new()
                        .child(h::Typography::heading(4, "Body heading"))
                        .child(h::Typography::paragraph(
                            h::ParagraphSize::Base,
                            "Prose applies the shared typographic rhythm to already-semantic children.",
                        ))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
