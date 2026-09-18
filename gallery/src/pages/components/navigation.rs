//! Navigation gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Navigation
    // -----------------------------------------------------------------------

    pub fn page_accordion(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let open = self.accordion_open.clone();
        let items = || {
            vec![
                h::AccordionItem::new("1", "What is HeroGPUI?")
                    .content(gpui::div().child("A native Rust component library for GPUI.")),
                h::AccordionItem::new("2", "Does it support dark mode?")
                    .content(gpui::div().child("Yes — every token has a light and dark value.")),
                h::AccordionItem::new("3", "Is it production ready?")
                    .subtitle("Short answer")
                    .content(
                        gpui::div().child("The component set is complete; the API is settling."),
                    ),
            ]
        };
        component_doc_page!(
            "Accordion",
            crate::pages::Page::Accordion.description(),
            crate::pages::Page::Accordion.import_line(),
            vec![
                (
                    "Usage",
                    "Titles, subtitles and body text keep their line heights when used inside larger text containers; opening and closing measures the body and animates height plus opacity over 200ms.",
                    specimen_body("acc-main", col(vec![h::Accordion::new(items())
                        .id("acc-usage")
                        .default_expanded("1")
                        .into_any_element()]), cx),
                ),
                (
                    "Hover Colour",
                    "`hover_bg` names the fill a hovered, enabled, closed trigger takes; open rows do not hover.",
                    specimen_body("acc-hover", col(vec![h::Accordion::new(items())
                        .id("acc-hover")
                        .hover_bg(cx.colors().accent.soft())
                        .into_any_element()]), cx),
                ),
                (
                    "Without Separator",
                    specimen_body("acc-no-separator", col(vec![h::Accordion::new(items())
                        .id("acc-nosep")
                        .hide_separator(true)
                        .default_expanded("1")
                        .into_any_element()]), cx),
                ),
                (
                    "Multiple Expanded",
                    specimen_body("acc-multiple", col(vec![h::Accordion::new(items())
                        .id("acc-multi")
                        .allows_multiple_expanded(true)
                        .default_expanded_keys(
                            [SharedString::from("1"), SharedString::from("2")]
                                .into_iter()
                                .collect(),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled State",
                    specimen_body("acc-disabled", col(vec![
                        spec_block(
                            "The whole group",
                            h::Accordion::new(items()).id("acc-dis").is_disabled(true),
                            cx,
                        ),
                        spec_block(
                            "One item",
                            h::Accordion::new(items())
                                .id("acc-dis-one")
                                .disabled_keys([SharedString::from("2")]),
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("acc-controlled", col(vec![
                        h::Accordion::new(items())
                            .id("acc-controlled")
                            .expanded_keys(open.clone())
                            .on_expanded_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.accordion_open = keys.clone();
                                    cx.notify();
                                },
                            ))
                            .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                                toggle_key(&mut this.accordion_open, key);
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("{} expanded", open.len()), cx),
                    ]), cx),
                ),
                (
                    "Custom Indicator",
                    specimen_body("acc-custom-indicator", col(vec![h::Accordion::new(vec![
                        h::AccordionItem::new("shipping", "Shipping details")
                            .content(gpui::div().child("Free shipping on orders over $50."))
                            .indicator(|state, _, cx| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_expanded {
                                        h::icons::MINUS
                                    } else {
                                        h::icons::PLUS
                                    })
                                    .text_color(cx.colors().muted)
                                    .into_any_element()
                            }),
                        h::AccordionItem::new("returns", "Returns policy")
                            .content(gpui::div().child("Returns are accepted within thirty days."))
                            .indicator(|state, _, cx| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_expanded {
                                        h::icons::MINUS
                                    } else {
                                        h::icons::PLUS
                                    })
                                    .text_color(cx.colors().muted)
                                    .into_any_element()
                            }),
                    ])
                    .id("acc-indicator")
                    .default_expanded("shipping")
                    .into_any_element()]), cx),
                ),
                (
                    "FAQ Layout",
                    specimen_body("acc-faq", col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Frequently asked"))
                        .child(
                            h::Accordion::new(vec![
                                h::AccordionItem::new("ship", "When does it ship?").content(
                                    gpui::div().child("Orders leave the warehouse next day."),
                                ),
                                h::AccordionItem::new("returns", "Can I return it?").content(
                                    gpui::div().child("Within thirty days, in any condition."),
                                ),
                                h::AccordionItem::new("warranty", "Is there a warranty?")
                                    .content(gpui::div().child("Two years, parts and labour.")),
                            ])
                            .id("acc-faq")
                            .variant(h::AccordionVariant::Surface),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Default",
                    specimen_body("acc-default", col(vec![h::Accordion::new(items())
                        .id("acc-default")
                        .expanded_keys(open.clone())
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Surface", "`radius(px)` rounds the `Surface` item card; the flush `Default` variant paints no card, so it is inert there.",
                    specimen_body("acc-surface", col(vec![h::Accordion::new(items())
                        .id("acc-surface")
                        .variant(h::AccordionVariant::Surface)
                        .radius(px(8.))
                        .expanded_keys(open.clone())
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Hidden separator",
                    specimen_body("acc-hidden-separator", col(vec![h::Accordion::new(items())
                        .id("acc-hidden-separator")
                        .hide_separator(true)
                        .expanded_keys(open)
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_breadcrumbs(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let last_nav = self.demo_text_value("bc-nav");
        let crumbs = || {
            vec![
                h::Crumb::new("Home").href("#"),
                h::Crumb::new("Components").href("#"),
                h::Crumb::new("Breadcrumbs"),
            ]
        };
        component_doc_page!(
            "Breadcrumbs",
            crate::pages::Page::Breadcrumbs.description(),
            crate::pages::Page::Breadcrumbs.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body(
                        "bc-main",
                        col(vec![
                            // `cx.listener` takes one event argument, and
                            // `on_navigate` receives the index and the crumb
                            // alongside the click, so the view state is reached
                            // through the entity directly.
                            h::Breadcrumbs::new(crumbs())
                                .id("bc-usage")
                                .full_width(true)
                                .text_size(px(16.))
                                .on_navigate({
                                    let view = cx.entity().downgrade();
                                    move |idx: &usize,
                                          crumb: &h::Crumb,
                                          _,
                                          _,
                                          cx: &mut gpui::App| {
                                        let _ = view.update(cx, |this, cx| {
                                            this.set_demo_text_value(
                                                "bc-nav",
                                                format!("{idx}: {}", crumb.label),
                                            );
                                            cx.notify();
                                        });
                                    }
                                })
                                .into_any_element(),
                            para(
                                &format!(
                                    "The current page is \"Breadcrumbs\" -- inert, no tab stop. \
                                 Last navigation: {}",
                                    if last_nav.is_empty() {
                                        "none yet".to_owned()
                                    } else {
                                        last_nav
                                    }
                                ),
                                cx,
                            ),
                        ]),
                        cx
                    ),
                ),
                (
                    "Navigation Levels",
                    specimen_body(
                        "bc-levels",
                        col(vec![
                            h::Breadcrumbs::new(vec![h::Crumb::new("Home").href("#")])
                                .id("bc-level-1")
                                .into_any_element(),
                            h::Breadcrumbs::new(vec![
                                h::Crumb::new("Home").href("#"),
                                h::Crumb::new("Library"),
                            ])
                            .id("bc-level-2")
                            .into_any_element(),
                            h::Breadcrumbs::new(vec![
                                h::Crumb::new("Home").href("#"),
                                h::Crumb::new("Library").href("#"),
                                h::Crumb::new("Data"),
                            ])
                            .id("bc-level-3")
                            .into_any_element(),
                        ]),
                        cx
                    ),
                ),
                (
                    "Disabled State",
                    specimen_body(
                        "bc-disabled",
                        col(vec![h::Breadcrumbs::new(vec![
                            h::Crumb::new("Home").href("#"),
                            h::Crumb::new("Archive").href("#"),
                            h::Crumb::new("2025"),
                        ])
                        .id("bc-disabled")
                        .is_disabled(true)
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Custom Separator",
                    specimen_body(
                        "bc-custom-separator",
                        col(vec![h::Breadcrumbs::new(crumbs())
                            .id("bc-sep-custom")
                            .separator_render(|_| {
                                gpui::div()
                                    .text_size(px(12.))
                                    .child("→".to_owned())
                                    .into_any_element()
                            })
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Separators",
                    specimen_body(
                        "bc-separators",
                        col(vec![
                            h::Breadcrumbs::new(crumbs())
                                .id("bc-sep-slash")
                                .separator(h::BreadcrumbSeparator::Slash)
                                .into_any_element(),
                            h::Breadcrumbs::new(crumbs())
                                .id("bc-sep-chevron")
                                .separator(h::BreadcrumbSeparator::Chevron)
                                .into_any_element(),
                            h::Breadcrumbs::new(crumbs())
                                .id("bc-sep-dash")
                                .separator(h::BreadcrumbSeparator::Dash)
                                .into_any_element(),
                        ]),
                        cx
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_disclosure(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let expanded = self.disclosure_expanded;
        let group = self.disclosure_group_expanded.clone();
        component_doc_page!(
            "Disclosure",
            crate::pages::Page::Disclosure.description(),
            crate::pages::Page::Disclosure.import_line(),
            vec![
                (
                    "Usage",
                    "The measured panel animates height and opacity over 200ms and stays mounted until the close transition completes.",
                    specimen_body(
                        "dis-main",
                        col(vec![h::Disclosure::new(
                            "disclosure-usage",
                            "Shipping details"
                        )
                        .child(gpui::div().child("Ships in 2-4 business days."))
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Render Function",
                    "The body is built from the Disclosure's current expanded and disabled state.",
                    {
                        let render_expanded = self.demo_flag("disclosure-render", true);
                        specimen_body("dis-render-props", col(vec![h::Disclosure::new("disclosure-render", "Account details")
                            .is_expanded(render_expanded)
                            .on_expanded_change(cx.listener(
                                |this, value: &bool, _, cx| {
                                    this.set_demo_flag("disclosure-render", *value);
                                    cx.notify();
                                },
                            ))
                            .content(|state| {
                                gpui::div()
                                    .child(format!(
                                        "The render closure received is_expanded={} and is_disabled={}.",
                                        state.is_expanded, state.is_disabled
                                    ))
                                    .into_any_element()
                            })
                            .into_any_element()]), cx)
                    }
                ),
                (
                    "Controlled",
                    specimen_body(
                        "dis-controlled",
                        col(vec![
                            h::DisclosureGroup::new("disclosure-controlled")
                                .item("returns", "Returns", gpui::div().child("Thirty days."))
                                .item("warranty", "Warranty", gpui::div().child("Two years."))
                                .expanded_keys(group.clone())
                                .on_expanded_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.disclosure_group_expanded = keys.clone();
                                        cx.notify();
                                    },
                                ))
                                .into_any_element(),
                            para(&format!("{} expanded", group.len()), cx),
                        ]),
                        cx
                    ),
                ),
                (
                    "Single",
                    specimen_body(
                        "dis-single",
                        col(vec![h::Disclosure::new(
                            "disclosure-single",
                            "Shipping details"
                        )
                        .is_expanded(expanded)
                        .on_expanded_change(cx.listener(|this, v: &bool, _, cx| {
                            this.disclosure_expanded = *v;
                            cx.notify();
                        }))
                        .child(gpui::div().child("Ships in 2-4 business days."))
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Group",
                    specimen_body(
                        "dis-group",
                        col(vec![h::DisclosureGroup::new("disclosure-group")
                            .item(
                                "item-1",
                                "Returns",
                                gpui::div().child("Free returns within 30 days."),
                            )
                            .item(
                                "item-2",
                                "Warranty",
                                gpui::div().child("Two years of coverage."),
                            )
                            .default_expanded_keys(["item-1"])
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Disabled Group",
                    specimen_body(
                        "dis-disabled",
                        col(vec![h::DisclosureGroup::new("disclosure-disabled-group")
                            .item("returns", "Returns", gpui::div().child("Thirty days."))
                            .item("warranty", "Warranty", gpui::div().child("Two years."))
                            .default_expanded_keys(["returns"])
                            .is_disabled(true)
                            .into_any_element()]),
                        cx
                    ),
                ),
            ],
            cx,
        )
    }

    pub fn page_link(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Link",
            crate::pages::Page::Link.description(),
            crate::pages::Page::Link.import_line(),
            vec![
                (
                    "Usage", "`radius(px)` replaces the root's default `rounded-xl`; the link paints no fill of its own, so this one carries an `sx` background to show the corner.",
                    specimen_body("link-main", col(vec![h::Link::new("ln-hover")
                        .label("Hover to see the underline")
                        .href("#")
                        .radius(px(4.))
                        .sx(|el| el.bg(gpui::rgba(0x22c55e33)))
                        .into_any_element()]), cx),
                ),
                (
                    "Icon Placement",
                    specimen_body("link-icon-placement", col(vec![
                        h::Link::new("ln-icon-end")
                            .label("Icon at end (default)")
                            .icon(icon(h::icons::EXTERNAL_LINK, cx))
                            .href("#")
                            .into_any_element(),
                        h::Link::new("ln-icon-start")
                            .label("Icon at start")
                            .icon(icon(h::icons::EXTERNAL_LINK, cx))
                            .icon_first(true)
                            .href("#")
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Text Decoration", "The pinned `.link` carries `no-underline decoration-[1.5px]`; hover recolours the decoration to `decoration-muted/50` and press to `decoration-muted`. The text colour itself never changes state; a different decoration is the caller's own styling on the element they own.",
                    specimen_body("link-decoration", col(vec![
                        h::Link::new("ln-decor")
                            .label("Underlined on hover")
                            .href("#")
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Custom Icon",
                    specimen_body("link-custom-icon", col(vec![h::Link::new("ln-custom-icon")
                        .label("Open the docs")
                        .icon(icon(h::icons::ARROW_RIGHT, cx))
                        .href("#")
                        .into_any_element()]), cx),
                ),
                (
                    "Render Function", "The `render` closure hands the link's interactive state to a caller-built element. The root keeps the `href`, press, focus and disabled wiring, and the closure draws the content from the state alone.",
                    specimen_body("link-render-props", col(vec![
                        h::Link::new("ln-render")
                            .href("#")
                            .render(|state| {
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child("Call to action")
                                    .child(gpui::div().text_size(px(12.)).opacity(0.6).child(
                                        if state.is_hovered {
                                            "hovered"
                                        } else if state.is_pressed {
                                            "pressed"
                                        } else if state.is_focus_visible {
                                            "focus-visible"
                                        } else if state.is_focused {
                                            "focused"
                                        } else if state.is_disabled {
                                            "disabled"
                                        } else {
                                            "custom render"
                                        },
                                    ))
                                    .into_any_element()
                            })
                            .into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_pagination(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let page = self.pagination_page;
        component_doc_page!(
            "Pagination",
            crate::pages::Page::Pagination.description(),
            crate::pages::Page::Pagination.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("pagination-main", col(vec![h::Pagination::new("pg-main", page, 10)
                        .full_width(true)
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Hover Colour",
                    "`hover_bg` names the fill enabled links and nav buttons take while hovered or pressed; the per-size press scale is unchanged.",
                    specimen_body("pagination-hover", col(vec![h::Pagination::new("pg-hover-bg", page, 6)
                        .hover_bg(cx.colors().accent.soft_hover())
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Sizes",
                    "Summary and link typography scale together and keep their line height in larger text containers.",
                    specimen_body("pagination-sizes", col(Size::ALL
                        .iter()
                        .map(|sz| {
                            h::Pagination::new(el_id(format!("pg-{sz:?}")), page, 8).size(*sz)
                                .summary(format!("Page {page} of 8"))
                        })
                        .els()), cx),
                ),
                (
                    "Disabled",
                    specimen_body("pagination-disabled", col(vec![h::Pagination::new("pg-disabled", page, 8)
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled Links",
                    specimen_body("pagination-disabled-links", col(vec![h::Pagination::new("pg-disabled-links", page, 8)
                        .disabled_keys([0, 5, 9])
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Simple (Previous / Next)",
                    specimen_body("pagination-simple", col(vec![row(vec![
                        h::Button::new("pg-prev")
                            .label("Previous")
                            .variant(Variant::Tertiary)
                            .is_disabled(page <= 1)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.pagination_page =
                                    this.pagination_page.saturating_sub(1).max(1);
                                cx.notify();
                            }))
                            .into_any_element(),
                        gpui::div()
                            .text_size(px(13.5))
                            .child(format!("Page {page} of 8"))
                            .into_any_element(),
                        h::Button::new("pg-next")
                            .label("Next")
                            .variant(Variant::Tertiary)
                            .is_disabled(page >= 8)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.pagination_page = (this.pagination_page + 1).min(8);
                                cx.notify();
                            }))
                            .into_any_element(),
                    ])]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("pagination-controlled", col(vec![
                        h::Pagination::new("pg-controlled", page, 8)
                            .on_change(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Page {page}"), cx),
                    ]), cx),
                ),
                (
                    "With Ellipsis",
                    specimen_body("pagination-ellipsis", col(vec![h::Pagination::new("pg-ellipsis", page, 24)
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "With Summary",
                    specimen_body("pagination-summary", col(vec![h::Pagination::new("pg-summary", page, 12)
                        .summary(format!(
                            "Showing {}-{} of 120 items",
                            (page.saturating_sub(1)) * 10 + 1,
                            (page * 10).min(120)
                        ))
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Render Props", "`link` receives each page number and `isActive`, so custom page content does not have to re-derive the current page.",
                    specimen_body("pagination-render-props", col(vec![
                        h::Pagination::new("pg-render-props", page, 5)
                            .link(|page, is_active| {
                                gpui::div()
                                    .child(if is_active {
                                        format!("[{page}]")
                                    } else {
                                        page.to_string()
                                    })
                                    .into_any_element()
                            })
                            .on_change(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Custom Icons", "`previous_icon` and `next_icon` replace the built-in chevrons on the composed previous and next parts.",
                    specimen_body("pagination-custom-icons", col(vec![
                        h::Pagination::new("pg-custom", page, 5)
                            .previous_icon(icon(h::icons::ARROW_LEFT, cx))
                            .next_icon(icon(h::icons::ARROW_RIGHT, cx))
                            .on_change(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Without controls",
                    specimen_body("pagination-without-controls", col(vec![h::Pagination::new("pg-plain", page, 10)
                        .on_change(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_tabs(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let primary = self.tab_solid.clone();
        let secondary = self.tab_underline.clone();
        let items = || {
            vec![
                h::TabItem::new("home", "Home").content(gpui::div().child("The home panel.")),
                h::TabItem::new("music", "Music").content(gpui::div().child("The music panel.")),
                h::TabItem::new("videos", "Videos").content(gpui::div().child("The videos panel.")),
            ]
        };
        component_doc_page!(
            "Tabs",
            crate::pages::Page::Tabs.description(),
            crate::pages::Page::Tabs.import_line(),
            vec![
                (
                    "Usage",
                    "Tab labels use a 20px line height independently of surrounding text.",
                    specimen_body("tabs-main", col(vec![h::Tabs::new(
                        "tabs-usage",
                        vec![
                            h::TabItem::new("photos", "Photos")
                                .content(gpui::div().child("Your photo library.")),
                            h::TabItem::new("music", "Music")
                                .content(gpui::div().child("Playlists and albums.")),
                            h::TabItem::new("videos", "Videos")
                                .content(gpui::div().child("Everything you have filmed.")),
                        ],
                        "photos",
                    )
                    .into_any_element()]), cx),
                ),
                (
                    "Sizes",
                    "`size` is additive, not a v3 prop: `Md` is the pinned 32px box with 16px padding and a 14px label; `Sm` is 28/12/12 with 16px leading. The secondary underline keeps its thickness.",
                    specimen_body("tabs-sizes", col(vec![
                        h::Tabs::new(
                            "tabs-size-sm",
                            vec![
                                h::TabItem::new("a", "Account")
                                    .content(gpui::div().child("Small step.")),
                                h::TabItem::new("b", "Billing")
                                    .content(gpui::div().child("Small step.")),
                            ],
                            "a",
                        )
                        .size(h::TabsSize::Sm)
                        .into_any_element(),
                        h::Tabs::new(
                            "tabs-size-md",
                            vec![
                                h::TabItem::new("a", "Account")
                                    .content(gpui::div().child("Medium step.")),
                                h::TabItem::new("b", "Billing")
                                    .content(gpui::div().child("Medium step.")),
                            ],
                            "a",
                        )
                        .size(h::TabsSize::Md)
                        .into_any_element(),
                    ]), cx),
                ),
                (
                    "Vertical",
                    specimen_body("tabs-vertical", col(vec![h::Tabs::new(
                        "tabs-vertical",
                        vec![
                            h::TabItem::new("account", "Account")
                                .content(gpui::div().child("Name, email and password.")),
                            h::TabItem::new("billing", "Billing")
                                .content(gpui::div().child("Cards and invoices.")),
                            h::TabItem::new("team", "Team")
                                .content(gpui::div().child("Members and roles.")),
                        ],
                        "account",
                    )
                    .orientation(Orientation::Vertical)
                    .into_any_element()]), cx),
                ),
                (
                    "Wrapping Labels",
                    "The label slot releases its min-content width, so long labels wrap inside constrained horizontal shares and vertical tab columns. Tabs keep the pinned fixed 32px height and ship no truncation utility, so wrapped lines paint past the pill exactly as the v3.2.5 sheet renders them.",
                    specimen_body("tabs-wrapping", col(vec![
                        h::Tabs::new(
                            "tabs-wrapping",
                            vec![
                                h::TabItem::new(
                                    "profile",
                                    "Profile and account security and recovery",
                                )
                                .content(gpui::div().child("Security settings.")),
                                h::TabItem::new(
                                    "notifications",
                                    "Notification preferences and delivery",
                                )
                                .content(gpui::div().child("Notification settings.")),
                                h::TabItem::new("billing", "Billing and invoices")
                                    .content(gpui::div().child("Billing settings.")),
                            ],
                            "profile",
                        )
                        .full_width(true)
                        .sx(|el| el.w(px(320.)))
                        .into_any_element(),
                        gpui::div()
                            .w(px(320.))
                            .h(px(180.))
                            .child(
                                h::Tabs::new(
                                    "tabs-vertical-wrapping",
                                    vec![
                                        h::TabItem::new(
                                            "profile",
                                            "Profile and account security and recovery settings",
                                        )
                                        .content(gpui::div().child("Security settings.")),
                                        h::TabItem::new("billing", "Billing and invoices")
                                            .content(gpui::div().child("Billing settings.")),
                                    ],
                                    "profile",
                                )
                                .orientation(Orientation::Vertical)
                                .into_any_element(),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Full Width",
                    "v3 stretches a tab list with `<Tabs.List className=\"w-full\">` and `<Tabs.Trigger className=\"flex-1\">`; `full_width(true)` is that pair.",
                    specimen_body("tabs-full-width", col(vec![gpui::div()
                        .w(px(420.))
                        .child(
                            h::Tabs::new(
                                "tabs-full-width",
                                vec![
                                    h::TabItem::new("all", "All")
                                        .content(gpui::div().child("Everything.")),
                                    h::TabItem::new("unread", "Unread")
                                        .content(gpui::div().child("Only unread.")),
                                    h::TabItem::new("archived", "Archived")
                                        .content(gpui::div().child("Put away.")),
                                ],
                                "all",
                            )
                            .full_width(true),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Alignment",
                    "Align content inside each tab to Start, Center (the default), or End. Selection and the indicator keep the same geometry.",
                    specimen_body("tabs-alignment", col(h::TabsVariant::ALL.into_iter().map(|variant| {
                        row(h::TabsAlign::ALL.into_iter().map(|align| {
                            spec(
                                &format!("{} / {}", variant.label(), align.label()),
                                h::Tabs::new(
                                    format!("tabs-align-{variant:?}-{align:?}"),
                                    vec![
                                        h::TabItem::new("general", "General"),
                                        h::TabItem::new("billing", "Subscription & Billing"),
                                        h::TabItem::new("privacy", "Privacy"),
                                    ],
                                    "general",
                                )
                                .variant(variant)
                                .orientation(Orientation::Vertical)
                                .align(align)
                                .sx(|el| el.h(px(116.))),
                                cx,
                            )
                        }).collect())
                    }).collect()), cx),
                ),
                (
                    "Overflow",
                    "More tabs than fit scroll along their axis. Wheel input from the other axis continues to the page.",
                    specimen_body("tabs-overflow", col(vec![
                        para("Horizontal", cx),
                        // The list only overflows inside a bounded box, which is
                        // how v3's own example frames it.
                        gpui::div()
                            .w(px(420.))
                            .child(h::Tabs::new(
                                "tabs-overflow",
                                (1..=12)
                                    .map(|n| {
                                        h::TabItem::new(
                                            SharedString::from(format!("t{n}")),
                                            SharedString::from(format!("Section {n}")),
                                        )
                                        .content(gpui::div().child(format!("Content {n}")))
                                    })
                                    .collect(),
                                "t1",
                            ))
                            .into_any_element(),
                        para("Vertical", cx),
                        gpui::div()
                            .h(px(200.))
                            .child(
                                h::Tabs::new(
                                    "tabs-overflow-vertical",
                                    (1..=8)
                                        .map(|n| {
                                            h::TabItem::new(
                                                SharedString::from(format!("v{n}")),
                                                SharedString::from(format!("Section {n}")),
                                            )
                                            .content(gpui::div().child(format!("Content {n}")))
                                        })
                                        .collect(),
                                    "v1",
                                )
                                .orientation(Orientation::Vertical),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Disabled Tab",
                    specimen_body("tabs-disabled", col(vec![h::Tabs::new(
                        "tabs-disabled",
                        vec![
                            h::TabItem::new("open", "Open")
                                .content(gpui::div().child("Open items.")),
                            // v3: `<Tabs.Tab isDisabled>` on a single tab; the
                            // selected "open" tab stays live.
                            h::TabItem::new("closed", "Closed")
                                .is_disabled(true)
                                .content(gpui::div().child("Closed items.")),
                        ],
                        "open",
                    )
                    .into_any_element()]), cx),
                ),
                (
                    "With Separator",
                    specimen_body("tabs-separator", col(vec![h::Tabs::new(
                        "tabs-separator",
                        vec![
                            h::TabItem::new("one", "One").content(gpui::div().child("First.")),
                            // v3: `<Tabs.Separator />` inside every tab but
                            // the first.
                            h::TabItem::new("two", "Two")
                                .separator()
                                .content(gpui::div().child("Second.")),
                            h::TabItem::new("three", "Three")
                                .separator()
                                .content(gpui::div().child("Third.")),
                        ],
                        "one",
                    )
                    .into_any_element(),]), cx),
                ),
                (
                    "Icon-Only Segmented Control",
                    "The capture-toolbar shape: `trigger` renders an element instead of the label text — `label` stays the accessible name — and `radius`, `list_bg`, `indicator_bg` and `indicator_shadow` restyle the tray and its pill. The icon carries no accessible name of its own and svg does not inherit the tab's text colour, so it is set explicitly.",
                    specimen_body("tabs-icon-segmented", col(vec![h::Tabs::new(
                        "tabs-icon-segmented",
                        vec![
                            h::TabItem::new("light", "Light").trigger(
                                gpui::svg()
                                    .size(px(16.))
                                    .path(h::icons::SUN)
                                    .text_color(cx.colors().foreground),
                            ),
                            h::TabItem::new("dark", "Dark").trigger(
                                gpui::svg()
                                    .size(px(16.))
                                    .path(h::icons::MOON)
                                    .text_color(cx.colors().foreground),
                            ),
                            h::TabItem::new("system", "System").trigger(
                                gpui::svg()
                                    .size(px(16.))
                                    .path(h::icons::GEAR)
                                    .text_color(cx.colors().foreground),
                            ),
                        ],
                        "light",
                    )
                    .radius(px(24.))
                    .list_bg(cx.colors().muted.alpha(0.10))
                    .indicator_bg(cx.colors().muted.alpha(0.25))
                    .indicator_shadow(false)
                    .into_any_element()]), cx),
                ),
                (
                    "Icon-Only Toolbar Segments",
                    "The fixed-box toolbar shape: `width`, `height` and `padding_x` fix every icon segment at 32px with no side padding — v3 spells that `TabsTrigger className=\"size-8 p-0\"` — while `list_padding(0)` removes the primary tray's inset the way `Tabs.List className=\"p-0\"` does, and `hover_fill(false)` drops the unselected-tab wash so the pointer only changes the label colour, not the fill.",
                    specimen_body("tabs-icon-toolbar", col(vec![h::Tabs::new(
                        "tabs-icon-toolbar",
                        vec![
                            h::TabItem::new("preview", "Preview")
                                .width(px(32.))
                                .height(px(32.))
                                .padding_x(px(0.))
                                .trigger(
                                    gpui::svg()
                                        .size(px(16.))
                                        .path(h::icons::EYE)
                                        .text_color(cx.colors().foreground),
                                ),
                            h::TabItem::new("duplicate", "Duplicate")
                                .width(px(32.))
                                .height(px(32.))
                                .padding_x(px(0.))
                                .trigger(
                                    gpui::svg()
                                        .size(px(16.))
                                        .path(h::icons::COPY)
                                        .text_color(cx.colors().foreground),
                                ),
                            h::TabItem::new("discard", "Discard")
                                .width(px(32.))
                                .height(px(32.))
                                .padding_x(px(0.))
                                .trigger(
                                    gpui::svg()
                                        .size(px(16.))
                                        .path(h::icons::TRASH)
                                        .text_color(cx.colors().foreground),
                                ),
                        ],
                        "preview",
                    )
                    .list_padding(px(0.))
                    .hover_fill(false)
                    .into_any_element()]), cx),
                ),
                (
                    "Secondary Variant",
                    specimen_body("tabs-secondary", col(vec![h::Tabs::new(
                        "tabs-secondary",
                        items(),
                        secondary.clone(),
                    )
                    .selected_key(secondary)
                    .variant(h::TabsVariant::Secondary)
                    .on_selection_change(cx.listener(|this, key: &SharedString, _, cx| {
                        this.tab_underline = key.clone();
                        cx.notify();
                    }))
                    .into_any_element()]), cx),
                ),
                (
                    "Secondary Variant Vertical",
                    specimen_body("tabs-secondary-vertical", col(vec![h::Tabs::new(
                        "tabs-secondary-vertical",
                        items(),
                        "home",
                    )
                    .variant(h::TabsVariant::Secondary)
                    .orientation(Orientation::Vertical)
                    .into_any_element()]), cx),
                ),
                (
                    "Primary",
                    specimen_body("tabs-primary", col(vec![h::Tabs::new("tabs-primary", items(), primary.clone())
                        .selected_key(primary)
                        .on_selection_change(cx.listener(|this, key: &SharedString, _, cx| {
                            this.tab_solid = key.clone();
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
