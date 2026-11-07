//! Typography gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Typography
    // -----------------------------------------------------------------------

    pub fn page_kbd(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let muted = cx.colors().muted;
        // v3 `Kbd.Abbr keyValue` + `Kbd.Content` chord, and its abbr-only and
        // content-only forms.
        let chord = |key: h::KbdKey, letter: &'static str| {
            h::Kbd::new()
                .child(h::KbdAbbr::new(key))
                .child(h::KbdContent::new().child(letter))
                .into_any_element()
        };
        let abbr = |key: h::KbdKey| h::Kbd::new().child(h::KbdAbbr::new(key)).into_any_element();
        let content = |text: &'static str| {
            h::Kbd::new()
                .child(h::KbdContent::new().child(text))
                .into_any_element()
        };
        let light_chord = |key: h::KbdKey, letter: &'static str| {
            h::Kbd::new()
                .variant(h::KbdVariant::Light)
                .child(h::KbdAbbr::new(key))
                .child(h::KbdContent::new().child(letter))
                .into_any_element()
        };
        // v3's inline `{" "}` joins have no inline flow in gpui, so each prose
        // line is a wrapping items-center row with the word-space gap.
        let prose = |children: Vec<AnyElement>| {
            gpui::div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(4.))
                .text_size(px(14.))
                .children(children)
                .into_any_element()
        };
        let word = |text: &'static str| gpui::div().child(text).into_any_element();
        component_doc_page!(
            "Kbd",
            crate::pages::Page::Kbd.description(),
            crate::pages::Page::Kbd.import_line(),
            vec![
                (
                    "Usage",
                    gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(16.))
                        .children(vec![
                            chord(h::KbdKey::Command, "K"),
                            chord(h::KbdKey::Shift, "P"),
                            chord(h::KbdKey::Ctrl, "C"),
                            chord(h::KbdKey::Option, "D"),
                        ])
                        .into_any_element(),
                ),
                (
                    "Variants",
                    col(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child("Copy:")
                            .child(chord(h::KbdKey::Command, "C"))
                            .child(light_chord(h::KbdKey::Command, "C"))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child("Paste:")
                            .child(chord(h::KbdKey::Command, "V"))
                            .child(light_chord(h::KbdKey::Command, "V"))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child("Cut:")
                            .child(chord(h::KbdKey::Command, "X"))
                            .child(light_chord(h::KbdKey::Command, "X"))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child("Undo:")
                            .child(chord(h::KbdKey::Command, "Z"))
                            .child(light_chord(h::KbdKey::Command, "Z"))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child("Redo:")
                            .child(
                                h::Kbd::new()
                                    .child(h::KbdAbbr::new(h::KbdKey::Command))
                                    .child(h::KbdAbbr::new(h::KbdKey::Shift))
                                    .child(h::KbdContent::new().child("Z")),
                            )
                            .child(
                                h::Kbd::new()
                                    .variant(h::KbdVariant::Light)
                                    .child(h::KbdAbbr::new(h::KbdKey::Command))
                                    .child(h::KbdAbbr::new(h::KbdKey::Shift))
                                    .child(h::KbdContent::new().child("Z")),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Navigation Keys",
                    col(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                gpui::div()
                                    .text_size(px(14.))
                                    .text_color(muted)
                                    .child("Arrow Keys:"),
                            )
                            .child(abbr(h::KbdKey::Up))
                            .child(abbr(h::KbdKey::Down))
                            .child(abbr(h::KbdKey::Left))
                            .child(abbr(h::KbdKey::Right))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                gpui::div()
                                    .text_size(px(14.))
                                    .text_color(muted)
                                    .child("Page Navigation:"),
                            )
                            .child(abbr(h::KbdKey::PageUp))
                            .child(abbr(h::KbdKey::PageDown))
                            .child(abbr(h::KbdKey::Home))
                            .child(abbr(h::KbdKey::End))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Inline Usage",
                    col(vec![
                        prose(vec![
                            word("Press"),
                            content("Esc"),
                            word("to close the dialog."),
                        ]),
                        prose(vec![
                            word("Use"),
                            chord(h::KbdKey::Command, "K"),
                            word("to open the command palette."),
                        ]),
                        prose(vec![
                            word("Navigate with"),
                            abbr(h::KbdKey::Up),
                            word("and"),
                            abbr(h::KbdKey::Down),
                            word("arrow keys."),
                        ]),
                        prose(vec![
                            word("Save your work with"),
                            chord(h::KbdKey::Command, "S"),
                            word("regularly."),
                        ]),
                    ]),
                ),
                (
                    "Instructional Text",
                    gpui::div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .rounded(cx.layout().radius_lg())
                        .bg(cx.colors().surface.background)
                        .p(px(16.))
                        .child(
                            gpui::div()
                                .text_size(px(14.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child("Quick Actions"),
                        )
                        .children(
                            [
                                ("• Open search:", h::KbdKey::Command, "K"),
                                ("• Toggle sidebar:", h::KbdKey::Command, "B"),
                                ("• New file:", h::KbdKey::Command, "N"),
                                ("• Quick save:", h::KbdKey::Command, "S"),
                            ]
                            .into_iter()
                            .map(|(label, key, letter)| {
                                prose(vec![
                                    word(label),
                                    chord(key, letter),
                                ])
                            })
                            .collect::<Vec<_>>(),
                        )
                        .into_any_element(),
                ),
                (
                    "Special Keys",
                    col(vec![
                        prose(vec![
                            word("Press"),
                            abbr(h::KbdKey::Enter),
                            word("to confirm or"),
                            abbr(h::KbdKey::Escape),
                            word("to cancel."),
                        ]),
                        prose(vec![
                            word("Use"),
                            abbr(h::KbdKey::Tab),
                            word("to navigate between form fields and"),
                            h::Kbd::new()
                                .child(h::KbdAbbr::new(h::KbdKey::Shift))
                                .child(h::KbdAbbr::new(h::KbdKey::Tab))
                                .into_any_element(),
                            word("to go back."),
                        ]),
                        prose(vec![
                            word("Hold"),
                            abbr(h::KbdKey::Space),
                            word("to temporarily enable panning mode."),
                        ]),
                    ]),
                ),
                (
                    "Styling Overrides",
                    "`radius(px)` replaces the `--radius-lg` corners; the arrow keys show it on the same symbols v3's Navigation Keys composes.",
                    row(vec![
                        h::KbdKey::Left,
                        h::KbdKey::Right,
                        h::KbdKey::Up,
                        h::KbdKey::Down,
                        h::KbdKey::Home,
                        h::KbdKey::End,
                    ]
                    .into_iter()
                    .map(|key| {
                        h::Kbd::new()
                            .radius(px(3.))
                            .child(h::KbdAbbr::new(key))
                            .into_any_element()
                    })
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
                    wide_col(vec![
                        h::Typography::new("Build better interfaces")
                            .kind(h::TypographyType::H1)
                            .into_any_element(),
                        h::Typography::new("Typography that stays semantic")
                            .kind(h::TypographyType::H2)
                            .into_any_element(),
                        h::Typography::new("Composable by default")
                            .kind(h::TypographyType::H3)
                            .into_any_element(),
                        h::Typography::new("Small heading")
                            .kind(h::TypographyType::H4)
                            .into_any_element(),
                        h::Typography::new(
                            "HeroUI Typography uses React Aria Components Text as the \
                             primitive, with semantic typography types and render-prop \
                             polymorphism.",
                        )
                        .into_any_element(),
                        h::Typography::new("Smaller muted body copy for secondary descriptions.")
                            .kind(h::TypographyType::BodySm)
                            .color(h::TextColor::Muted)
                            .into_any_element(),
                        h::Typography::new("pnpm add @heroui/react")
                            .kind(h::TypographyType::Code)
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
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
