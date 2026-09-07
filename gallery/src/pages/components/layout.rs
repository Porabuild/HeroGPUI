//! Layout gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Layout
    // -----------------------------------------------------------------------

    pub fn page_card(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let card = |variant: h::CardVariant| {
            h::Card::new()
                .variant(variant)
                .w(px(260.))
                .child(
                    h::CardHeader::new()
                        .child(h::CardTitle::new().child("Daily report"))
                        .child(h::CardDescription::new().child("Traffic summary for the week")),
                )
                .child(h::CardContent::new().child("Sessions are up 12% week over week."))
                .child(
                    h::CardFooter::new().child(
                        h::Button::new(el_id(format!("card-{variant:?}-cta")))
                            .label("View")
                            .size(Size::Sm)
                            .variant(Variant::Tertiary),
                    ),
                )
        };
        component_doc_page!(
            "Card",
            crate::pages::Page::Card.description(),
            crate::pages::Page::Card.import_line(),
            vec![
                (
                    "Usage",
                    // v3's card is 400px wide, leads with an icon above the
                    // header and closes on a link in the footer.
                    row(vec![h::Card::new()
                        .w(px(400.))
                        .child(
                            gpui::svg()
                                .size(px(24.))
                                .path(h::icons::KEY)
                                .text_color(cx.colors().accent.color),
                        )
                        .child(
                            h::CardHeader::new()
                                .child(h::CardTitle::new().child("Become an Acme Creator!"))
                                .child(h::CardDescription::new().child(CARD_USAGE_DESCRIPTION),),
                        )
                        .child(
                            h::CardFooter::new().child(
                                h::Link::new("card-usage-link")
                                    .label("Creator Hub")
                                    .href("https://example.com")
                                    .icon(
                                        gpui::svg()
                                            .size(px(12.))
                                            .path(h::icons::EXTERNAL_LINK)
                                            .text_color(cx.colors().link),
                                    ),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(h::CardVariant::ALL.iter().map(|v| card(*v)).els()),
                ),
                (
                    "Horizontal Layout",
                    row(vec![h::Card::new()
                        .w(px(420.))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(16.))
                                    .child(
                                        gpui::div()
                                            .size(px(72.))
                                            .flex_shrink_0()
                                            .rounded(px(12.))
                                            .bg(cx.colors().default.color),
                                    )
                                    .child(
                                        gpui::div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.))
                                            .child(gpui::div().child("Weekly digest"))
                                            .child(
                                                gpui::div()
                                                    .text_size(px(12.5))
                                                    .text_color(cx.colors().muted)
                                                    .child("Every Monday, 9am"),
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Avatar",
                    row(vec![h::Card::new()
                        .w(px(200.))
                        .child(
                            gpui::div()
                                .size(px(56.))
                                .rounded(h::util::soft_radius(cx))
                                .bg(cx.colors().default.color),
                        )
                        .child(
                            h::CardHeader::new()
                                .child(h::CardTitle::new().child("Indie Hackers"))
                                .child(h::CardDescription::new().child("148 members")),
                        )
                        .child(
                            h::CardFooter::new()
                                .child(h::Avatar::new("card-martha").name("Martha").size(Size::Sm))
                                .child("By Martha"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Images",
                    row(vec![h::Card::new()
                        .w(px(280.))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .h(px(140.))
                                    .w_full()
                                    .rounded(px(12.))
                                    .bg(cx.colors().default.color),
                            ),
                        )
                        .child(h::CardFooter::new().child("A placeholder for cover art."))
                        .into_any_element()]),
                ),
                (
                    "With Form",
                    row(vec![h::Card::new()
                        .w(px(320.))
                        .child(h::CardHeader::new().child(h::CardTitle::new().child("Sign in")))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("card-email", "", cx))
                                            .label("Email")
                                            .input_type(h::InputType::Email)
                                            .full_width(),
                                    )
                                    .child(
                                        h::TextField::new(self.demo_text("card-password", "", cx))
                                            .label("Password")
                                            .input_type(h::InputType::Password)
                                            .full_width(),
                                    ),
                            ),
                        )
                        .child(
                            h::CardFooter::new()
                                .child(h::Button::new("card-signin").label("Sign in")),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_separator(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Separator",
            crate::pages::Page::Separator.description(),
            crate::pages::Page::Separator.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(gpui::div().child("Above"))
                        .child(h::Separator::new())
                        .child(gpui::div().child("Below"))
                        .into_any_element()]),
                ),
                (
                    "With Surface",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Notifications"))
                        .child(h::Separator::new())
                        .child(gpui::div().child("Privacy"))
                        .into_any_element()]),
                ),
                (
                    "With Content",
                    col(vec![h::Separator::new()
                        .child(gpui::div().text_size(px(12.)).child("OR"))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(h::SeparatorVariant::ALL
                        .iter()
                        .flat_map(|v| {
                            vec![
                                gpui::div()
                                    .text_size(px(12.))
                                    .text_color(cx.colors().muted)
                                    .child(v.label())
                                    .into_any_element(),
                                h::Separator::new().variant(*v).into_any_element(),
                            ]
                        })
                        .collect()),
                ),
                (
                    "Vertical",
                    row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .h(px(24.))
                        .child("Docs")
                        .child(h::Separator::new().orientation(Orientation::Vertical))
                        .child("Blog")
                        .child(h::Separator::new().orientation(Orientation::Vertical))
                        .child("Support")
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_surface(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let panel = |variant: h::SurfaceVariant| {
            h::Surface::new()
                .variant(variant)
                // The upstream examples dress their skeletons through
                // className (`p-6`, `gap-3`, plus `rounded-3xl`/borders).
                // className customization stays unavailable, so the demo's
                // paddings/gaps only exercise the layout knobs this port's
                // Surface builders expose.
                .padding(px(24.))
                .gap(px(12.))
                .child(h::Typography::heading(6, "Surface content").into_any_element())
                .child(
                    h::Typography::paragraph(
                        h::ParagraphSize::Sm,
                        "Nested content inherits the surface foreground.",
                    )
                    .color(h::TextColor::Muted)
                    .into_any_element(),
                )
        };
        component_doc_page!(
            "Surface",
            crate::pages::Page::Surface.description(),
            crate::pages::Page::Surface.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(gpui::div().child("A surface groups related content."))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        panel(h::SurfaceVariant::Default).into_any_element(),
                        panel(h::SurfaceVariant::Secondary).into_any_element(),
                        panel(h::SurfaceVariant::Tertiary).into_any_element(),
                        panel(h::SurfaceVariant::Transparent).into_any_element(),
                    ]),
                ),
                (
                    "With Form Components",
                    field_col(vec![h::Surface::new()
                        // Same as the variants panel: upstream adds its
                        // `p-6` + `gap-4` through className; these explicit
                        // paddings/gaps exercise the port's own layout
                        // knobs, and no radius/border is re-added here.
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::Input::new(self.input_name.clone())
                                .placeholder("Secondary input")
                                .variant(FieldVariant::Secondary),
                        )
                        .child(
                            h::TextArea::new(self.input_bio.clone())
                                .placeholder("Secondary text area")
                                .rows(3),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_toolbar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let bar = |key: &str, attached: bool, orientation: Orientation| {
            h::Toolbar::new()
                // Names the instance: the toolbar's keyed focus state is per
                // id, and three toolbars share this page.
                .id(el_id(format!("toolbar-{key}")))
                .is_attached(attached)
                .orientation(orientation)
                .child(
                    h::ToggleButtonGroup::new(el_id(format!("toolbar-toggle-{key}")))
                        .selection_mode(SelectionMode::Multiple)
                        .separators(true)
                        .child_toggle(
                            h::ToggleButton::new(el_id(format!("tbar-{key}-b"))).label("B"),
                        )
                        .child_toggle(
                            h::ToggleButton::new(el_id(format!("tbar-{key}-i"))).label("I"),
                        ),
                )
                // The bar crosses its own flow, so the divider needs no
                // orientation here.
                .separator()
                .child(
                    h::ButtonGroup::new()
                        .variant(Variant::Tertiary)
                        .size(Size::Sm)
                        .separators(true)
                        .button(h::Button::new(el_id(format!("tbar-{key}-copy"))).label("Copy"))
                        .button(h::Button::new(el_id(format!("tbar-{key}-cut"))).label("Cut")),
                )
        };
        component_doc_page!(
            "Toolbar",
            crate::pages::Page::Toolbar.description(),
            crate::pages::Page::Toolbar.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Toolbar::new()
                        .id("tb-usage")
                        .child(
                            h::Button::new("tb-usage-1")
                                .label("Cut")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .child(
                            h::Button::new("tb-usage-2")
                                .label("Copy")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .child(
                            h::Button::new("tb-usage-3")
                                .label("Paste")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .into_any_element()]),
                ),
                (
                    "With ButtonGroup",
                    col(vec![h::Toolbar::new()
                        .id("tb-button-group")
                        .child(
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .separators(true)
                                .button(h::Button::new("tb-bg-1").label("Left"))
                                .button(h::Button::new("tb-bg-2").label("Center"))
                                .button(h::Button::new("tb-bg-3").label("Right")),
                        )
                        .separator()
                        .child(
                            h::Button::new("tb-bg-4")
                                .label("Reset")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    col(vec![
                        bar("h", false, Orientation::Horizontal).into_any_element()
                    ]),
                ),
                (
                    "Attached",
                    col(vec![
                        bar("attached", true, Orientation::Horizontal).into_any_element()
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![
                        bar("v", false, Orientation::Vertical).into_any_element()
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
