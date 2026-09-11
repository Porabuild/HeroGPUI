//! Data display gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Data display
    // -----------------------------------------------------------------------

    pub fn page_badge(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Badge",
            crate::pages::Page::Badge.description(),
            crate::pages::Page::Badge.import_line(),
            vec![
                (
                    "Usage",
                    // v3 anchors three avatars: a danger count, an accent label
                    // and a success dot pinned to the bottom-right.
                    row(vec![
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 0usize), "Jane Doe"))
                            .child(
                                h::Badge::new()
                                    .color(Color::Danger)
                                    .size(Size::Sm)
                                    .text_size(px(14.))
                                    .child(h::BadgeLabel::new().child("5")),
                            )
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 1usize), "Alex Brown"))
                            .child(
                                h::Badge::new()
                                    .color(Color::Accent)
                                    .size(Size::Sm)
                                    .child(h::BadgeLabel::new().child("New")),
                            )
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 2usize), "Chris Davis"))
                            .child(
                                h::Badge::new()
                                    .color(Color::Success)
                                    .size(Size::Sm)
                                    .placement(h::BadgePlacement::BottomRight),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|sz| {
                            spec(
                                sz.label(),
                                h::BadgeAnchor::new()
                                    .child(avatar_box(("badge-anchor", 1usize), "Alex Brown"))
                                    .child(
                                        h::Badge::new()
                                            .size(*sz)
                                            .child(h::BadgeLabel::new().child("5")),
                                    ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Dot Badge",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                // No children is v3's dot badge.
                                h::BadgeAnchor::new()
                                    .child(avatar_box(("badge-anchor", 2usize), "Chris Davis"))
                                    .child(h::Badge::new().color(*c)),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "With Content",
                    row(vec![
                        spec(
                            "Number",
                            h::BadgeAnchor::new()
                                .child(avatar_box(("badge-anchor", 3usize), "Jane Doe"))
                                .child(
                                    h::Badge::new()
                                        .color(Color::Danger)
                                        .size(Size::Sm)
                                        .child(h::BadgeLabel::new().child("5")),
                                ),
                            cx,
                        ),
                        spec(
                            "Text",
                            h::BadgeAnchor::new()
                                .child(avatar_box(("badge-anchor", 4usize), "Alex Brown"))
                                .child(
                                    h::Badge::new()
                                        .color(Color::Accent)
                                        .child(h::BadgeLabel::new().child("NEW")),
                                ),
                            cx,
                        ),
                        spec(
                            "Icon",
                            // Only plain text is auto-wrapped upstream; an
                            // element child composes straight into the badge.
                            h::BadgeAnchor::new()
                                .child(avatar_box(("badge-anchor", 5usize), "Chris Davis"))
                                .child(
                                    h::Badge::new().color(Color::Success).child(
                                        gpui::svg()
                                            .size(px(10.))
                                            .path(h::icons::CHECK)
                                            .text_color(cx.colors().success.foreground),
                                    ),
                                ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Variants",
                    row(h::BadgeVariant::ALL
                        .iter()
                        .map(|v| {
                            spec(
                                v.label(),
                                h::BadgeAnchor::new()
                                    .child(avatar_box(("badge-anchor", 6usize), "Jane Doe"))
                                    .child(
                                        h::Badge::new()
                                            .color(Color::Accent)
                                            .variant(*v)
                                            .child(h::BadgeLabel::new().child("5")),
                                    ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::BadgeAnchor::new()
                                    .child(avatar_box(("badge-anchor", 7usize), "Alex Brown"))
                                    .child(
                                        h::Badge::new()
                                            .color(*c)
                                            .child(h::BadgeLabel::new().child("5")),
                                    ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Placements",
                    row(vec![
                        // No children is v3's dot badge.
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 8usize), "Chris Davis"))
                            .child(h::Badge::new().color(Color::Success))
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 9usize), "Jane Doe"))
                            .child(
                                h::Badge::new()
                                    .placement(h::BadgePlacement::BottomRight)
                                    .child(h::BadgeLabel::new().child("9")),
                            )
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(("badge-anchor", 10usize), "Alex Brown"))
                            .child(
                                h::Badge::new()
                                    .placement(h::BadgePlacement::TopLeft)
                                    .child(h::BadgeLabel::new().child("New")),
                            )
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_chip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Chip",
            crate::pages::Page::Chip.description(),
            crate::pages::Page::Chip.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Chip::new()
                        .child(h::ChipLabel::new().child("Chip"))
                        .into_any_element()]),
                ),
                (
                    "Statuses",
                    row(vec![
                        h::Chip::new()
                            .color(Color::Success)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Danger)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Vacation"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Icons",
                    row(vec![
                        h::Chip::new()
                            .color(Color::Success)
                            .child(icon(h::icons::CHECK, cx))
                            .child(h::ChipLabel::new().child("Verified"))
                            .into_any_element(),
                        h::Chip::new()
                            .child(icon(h::icons::EXTERNAL_LINK, cx))
                            .child(h::ChipLabel::new().child("Link"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Accent)
                            .child(icon(h::icons::SEARCH, cx))
                            .child(h::ChipLabel::new().child("Search"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants",
                    row(h::ChipVariant::ALL
                        .iter()
                        .map(|v| {
                            h::Chip::new()
                                .variant(*v)
                                .color(Color::Accent)
                                .child(h::ChipLabel::new().child(v.label()))
                        })
                        .els()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            h::Chip::new()
                                .color(*c)
                                .child(h::ChipLabel::new().child(c.label()))
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| {
                            h::Chip::new()
                                .size(*s)
                                .child(h::ChipLabel::new().child(s.label()))
                        })
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let table_page = self.demo_value("tbl-page", 1.) as usize;
        let tbl_expanded = self.demo_selection("tbl-expanded");
        let build = |id: &'static str| {
            h::Table::new(vec!["Name".into(), "Role".into(), "Status".into()])
                .id(id)
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Tony Reichert").into_any_element(),
                        gpui::div().child("CEO").into_any_element(),
                        h::Chip::new()
                            .color(Color::Success)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                    ])
                    .text_value("Tony Reichert"),
                )
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Zoey Lang").into_any_element(),
                        gpui::div().child("Tech Lead").into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                    ])
                    .text_value("Zoey Lang"),
                )
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Jane Fisher").into_any_element(),
                        gpui::div().child("Designer").into_any_element(),
                        h::Chip::new()
                            .color(Color::Danger)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Vacation"))
                            .into_any_element(),
                    ])
                    .text_value("Jane Fisher"),
                )
        };
        component_doc_page!(
            "Table",
            crate::pages::Page::Table.description(),
            crate::pages::Page::Table.import_line(),
            vec![
                ("Usage", "Headers use 12px text with 16px lines; cells use 14px text with 20px lines in both ordinary and virtual rows.", stretch_col(vec![build("tbl-usage").into_any_element()])),
                (
                    "Row Hover",
                    "`row_hover_bg` names the fill a hovered unselected interactive row takes; a selected row keeps its selection fill.",
                    stretch_col(vec![build("tbl-row-hover")
                        .row_hover_bg(cx.colors().accent.soft())
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    stretch_col(h::TableVariant::ALL
                        .iter()
                        .map(|v| {
                            build(match v {
                                h::TableVariant::Primary => "tbl-variant-primary",
                                h::TableVariant::Secondary => "tbl-variant-secondary",
                            })
                            .variant(*v)
                        })
                        .els()),
                ),
                (
                    "Custom sort indicator",
                    // Sorted on load, so the custom indicator is actually
                    // visible: `indicator` only renders for the sorted column.
                    stretch_col(vec![h::Table::new(vec![])
                        .id("tbl-custom-sort-indicator")
                        .column(h::TableColumn::new("Name").allows_sorting(true))
                        .column("Role")
                        .row(vec![
                            gpui::div().child("Tony Reichert").into_any_element(),
                            gpui::div().child("CEO").into_any_element(),
                        ])
                        .row(vec![
                            gpui::div().child("Zoey Lang").into_any_element(),
                            gpui::div().child("Tech Lead").into_any_element(),
                        ])
                        .sort_descriptor(h::SortDescriptor::new(
                            "Name",
                            h::SortDirection::Ascending,
                        ))
                        .indicator(|dir| {
                            gpui::div()
                                .text_size(px(11.))
                                .child(match dir {
                                    h::SortDirection::Ascending => "▲",
                                    h::SortDirection::Descending => "▼",
                                })
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Selection",
                    stretch_col(vec![
                        build("tbl-selection")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(self.table_selection.clone())
                            .on_selection_change(cx.listener(
                                |this, keys: &[SharedString], _, cx| {
                                    this.table_selection = keys.to_vec();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", self.table_selection.len()), cx),
                    ]),
                ),
                (
                    "Sorting",
                    "PageUp moves from the body to the first header — from the top of a virtual body; mid-body it pages by viewport. Enter sorts a sortable header; Down or PageDown returns to the first or last enabled row.",
                    stretch_col(vec![
                        {
                            let mut rows = [
                                ["Tony Reichert", "CEO", "Active"],
                                ["Zoey Lang", "Tech Lead", "Paused"],
                            ];
                            if let Some(sort) = &self.table_sort {
                                let column = usize::from(sort.column.as_ref() == "Role");
                                rows.sort_by(|a, b| match sort.direction {
                                    h::SortDirection::Ascending => a[column].cmp(b[column]),
                                    h::SortDirection::Descending => b[column].cmp(a[column]),
                                });
                            }
                            let mut sortable = h::Table::new(vec![])
                                .id("tbl-sorting")
                                .column(
                                    h::TableColumn::new("Name")
                                        .allows_sorting(true)
                                        .is_row_header(true),
                                )
                                .column(h::TableColumn::new("Role").allows_sorting(true))
                                .column("Status")
                                .on_sort_change(cx.listener(
                                    |this, d: &h::SortDescriptor, _, cx| {
                                        this.table_sort = Some(d.clone());
                                        cx.notify();
                                    },
                                ));
                            for row in rows {
                                sortable = sortable.row(row.into_iter().map(|text| {
                                    gpui::div().child(text).into_any_element()
                                }).collect());
                            }
                            if let Some(d) = self.table_sort.clone() {
                                sortable = sortable.sort_descriptor(d);
                            }
                            sortable.into_any_element()
                        },
                        para(
                            &match &self.table_sort {
                                Some(d) => format!("Sorted by {} {:?}", d.column, d.direction),
                                None => "Unsorted".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Virtualization", "Cells are built elements, which cannot be handed out twice, so a virtual table takes a row factory and asks for the rows the viewport shows — one thousand of them, forty pixels each. The fixed-row body caps at `max_h`, shrinks below it in a bounded parent, and PageUp/PageDown move by the visible viewport, including after resize, while skipping disabled stops.",
                    stretch_col(vec![
                        h::Table::new(vec![])
                            .id("tbl-virtualization")
                            .column(h::TableColumn::new("Name").is_row_header(true))
                            .column("Email")
                            .row_height(px(40.))
                            .max_h(px(320.))
                            .virtual_rows(
                                1000,
                                "virtual-users",
                                |i| i.to_string().into(),
                                |i| {
                                    let (name, email) = virtual_user(i);
                                    h::TableRow::new(vec![
                                        gpui::div().child(name).into_any_element(),
                                        gpui::div().child(email).into_any_element(),
                                    ])
                                },
                            )
                            .virtual_text_value(|i| virtual_user(i).0.into())
                            .into_any_element(),
                        para(
                            "`estimated_row_height` virtualizes rows that differ: gpui's \
                             `list` measures each one it builds, and `loader_height` \
                             fixes the load-more row underneath.",
                            cx,
                        ),
                        h::Table::new(vec![])
                            .id("tbl-virtual-var")
                            .column(h::TableColumn::new("Name").is_row_header(true))
                            .column("Email")
                            .estimated_row_height(px(44.))
                            .loader_height(px(44.))
                            .max_h(px(320.))
                            .is_pending(true)
                            .virtual_rows(
                                1000,
                                "virtual-variable-users",
                                |i| i.to_string().into(),
                                |i| {
                                    let (name, email) = virtual_user(i);
                                    let mut cells = vec![gpui::div()
                                        .flex()
                                        .flex_col()
                                        .child(name)
                                        .when(i % 3 == 0, |el| {
                                            el.child(
                                                gpui::div()
                                                    .text_size(px(12.))
                                                    .child("Signed up this week"),
                                            )
                                        })
                                        .into_any_element()];
                                    cells.push(gpui::div().child(email).into_any_element());
                                    h::TableRow::new(cells)
                                },
                            )
                            .virtual_text_value(|i| virtual_user(i).0.into())
                            .into_any_element(),
                    ]),
                ),
                ("Column Resizing", "Drag a trailing-edge divider, or focus it with Tab, press Enter, and use the arrow keys. This example feeds onResize values back as controlled column widths and reports completion through onResizeEnd. Scroll horizontally to reach wide columns; vertical wheel input does not shift them sideways.", {
                    let resize_name = self.demo_value("tbl-resize-name", 220.);
                    let resize_role = self.demo_value("tbl-resize-role", 180.);
                    let resize_status = self.demo_text_value("tbl-resize-status");
                    stretch_col(vec![
                        para(
                            &format!(
                                "{} Name: {:.0}px · Role: {:.0}px",
                                if resize_status.is_empty() {
                                    "Ready."
                                } else {
                                    resize_status.as_str()
                                },
                                resize_name,
                                resize_role,
                            ),
                            cx,
                        ),
                        h::Table::new(vec![])
                            .id("tbl-column-resizing")
                            .column(
                                h::TableColumn::new("Name")
                                    .allows_resizing(true)
                                    .width(px(resize_name))
                                    .min_width(px(120.)),
                            )
                            .column(
                                h::TableColumn::new("Role")
                                    .allows_resizing(true)
                                    .width(px(resize_role)),
                            )
                            .column(h::TableColumn::new("Status").default_width(px(140.)))
                            .row(vec![
                                gpui::div().child("Tony Reichert").into_any_element(),
                                gpui::div().child("CEO").into_any_element(),
                                gpui::div().child("Active").into_any_element(),
                            ])
                            .row(vec![
                                gpui::div().child("Zoey Lang").into_any_element(),
                                gpui::div().child("Tech Lead").into_any_element(),
                                gpui::div().child("Paused").into_any_element(),
                            ])
                            .on_resize_start(cx.listener(|this, _, _, cx| {
                                this.set_demo_text_value(
                                    "tbl-resize-status",
                                    "Resizing.".to_owned(),
                                );
                                cx.notify();
                            }))
                            .on_resize(cx.listener(
                                |this, widths: &[(SharedString, gpui::Pixels)], _, cx| {
                                    for (column, width) in widths {
                                        match column.as_ref() {
                                            "Name" => this.set_demo_value(
                                                "tbl-resize-name",
                                                f32::from(*width),
                                            ),
                                            "Role" => this.set_demo_value(
                                                "tbl-resize-role",
                                                f32::from(*width),
                                            ),
                                            _ => {}
                                        }
                                    }
                                    cx.notify();
                                },
                            ))
                            .on_resize_end(cx.listener(|this, _, _, cx| {
                                this.set_demo_text_value("tbl-resize-status", "Saved.".to_owned());
                                cx.notify();
                            }))
                            .into_any_element(),
                    ])
                },),
                (
                    "Expandable Rows", "A row's children are nested under it, and `expandedKeys` decides which parents show theirs. The chevron sits in the tree column; Right expands the focused parent, and Left collapses it or returns the row cursor to its parent.",
                    stretch_col(vec![
                        {
                            let cell = |text: &str| gpui::div().child(text.to_owned());
                            h::Table::new(vec!["Title".into(), "Type".into(), "Modified".into()])
                                .id("tbl-expandable-rows")
                                .tree_column(0)
                                .expanded_keys(tbl_expanded.iter().cloned())
                                .on_expanded_change(cx.listener(
                                    |this, keys: &[SharedString], _, cx| {
                                        this.set_demo_selection("tbl-expanded", keys.to_vec());
                                        cx.notify();
                                    },
                                ))
                                .tree_row(
                                    h::TableRow::new(vec![
                                        cell("Documents").into_any_element(),
                                        cell("Folder").into_any_element(),
                                        cell("8/2/2025").into_any_element(),
                                    ])
                                    .key("documents")
                                    .children(vec![
                                        h::TableRow::new(vec![
                                            cell("Reports").into_any_element(),
                                            cell("Folder").into_any_element(),
                                            cell("8/2/2025").into_any_element(),
                                        ])
                                        .key("reports")
                                        .children(vec![
                                            h::TableRow::new(vec![
                                                cell("Weekly Report").into_any_element(),
                                                cell("File").into_any_element(),
                                                cell("7/10/2025").into_any_element(),
                                            ])
                                            .key("weekly"),
                                            h::TableRow::new(vec![
                                                cell("Budget").into_any_element(),
                                                cell("File").into_any_element(),
                                                cell("8/20/2025").into_any_element(),
                                            ])
                                            .key("budget"),
                                        ]),
                                        h::TableRow::new(vec![
                                            cell("Contract.pdf").into_any_element(),
                                            cell("File").into_any_element(),
                                            cell("6/1/2025").into_any_element(),
                                        ])
                                        .key("contract"),
                                    ]),
                                )
                                .tree_row(
                                    h::TableRow::new(vec![
                                        cell("Photos").into_any_element(),
                                        cell("Folder").into_any_element(),
                                        cell("5/5/2025").into_any_element(),
                                    ])
                                    .key("photos")
                                    .children(vec![
                                        h::TableRow::new(vec![
                                            cell("Holiday.jpg").into_any_element(),
                                            cell("Image").into_any_element(),
                                            cell("5/5/2025").into_any_element(),
                                        ])
                                        .key("holiday"),
                                    ]),
                                )
                                .into_any_element()
                        },
                    ]),
                ),
                (
                    "Secondary Variant",
                    stretch_col(vec![build("tbl-secondary-variant")
                        .variant(h::TableVariant::Secondary)
                        .into_any_element()]),
                ),
                (
                    "Async Loading", "`isPending` covers the table while a request is in flight; `onLoadMore` fires when the last row scrolls into view.",
                    stretch_col(vec![
                        build("tbl-async-loading")
                            .is_pending(true)
                            .on_load_more(|_, _| {})
                            // v3's Async Loading example writes `scrollOffset={0}`.
                            .scroll_offset(0.)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Pagination",
                    stretch_col(vec![{
                        let start = table_page.saturating_sub(1) * 2;
                        let people = [
                            ("Tony Reichert", "CEO"),
                            ("Zoey Lang", "Tech Lead"),
                            ("Jane Fisher", "Designer"),
                            ("William Howard", "Support"),
                            ("Kristen Copper", "Sales Manager"),
                            ("Emily Collins", "Marketing"),
                        ];
                        let mut paged =
                            h::Table::new(vec!["Name".into(), "Role".into()]).id("tbl-pagination");
                        for (name, role) in people.iter().skip(start).take(2) {
                            paged = paged.row(vec![
                                gpui::div().child(*name).into_any_element(),
                                gpui::div().child(*role).into_any_element(),
                            ]);
                        }
                        // v3 puts the pagination in a `Table.Footer`, with a
                        // `Pagination.Summary` at its start.
                        paged
                            .footer(
                                h::Pagination::new("tbl-pages", table_page, 3)
                                    .size(Size::Sm)
                                    .summary(format!(
                                        "{} to {} of {} results",
                                        start + 1,
                                        (start + 2).min(people.len()),
                                        people.len()
                                    ))
                                    .on_change(cx.listener(|this, p: &usize, _, cx| {
                                        this.set_demo_value("tbl-page", *p as f32);
                                        cx.notify();
                                    })),
                            )
                            .into_any_element()
                    },]),
                ),
                (
                    "Custom Cells",
                    stretch_col(vec![h::Table::new(vec![
                        "Member".into(),
                        "Role".into(),
                        "Status".into(),
                    ])
                    .id("tbl-custom-cells")
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                h::Avatar::new("tbl-tony")
                                    .name("Tony Reichert")
                                    .size(Size::Sm)
                            )
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .child(gpui::div().child("Tony Reichert"))
                                    .child(
                                        gpui::div()
                                            .text_size(px(11.5))
                                            .text_color(cx.colors().muted)
                                            .child("tony@example.com"),
                                    ),
                            )
                            .into_any_element(),
                        gpui::div().child("CEO").into_any_element(),
                        h::Chip::new()
                            .color(Color::Success)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                    ])
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(h::Avatar::new("tbl-zoey").name("Zoey Lang").size(Size::Sm))
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .child(gpui::div().child("Zoey Lang"))
                                    .child(
                                        gpui::div()
                                            .text_size(px(11.5))
                                            .text_color(cx.colors().muted)
                                            .child("zoey@example.com"),
                                    ),
                            )
                            .into_any_element(),
                        gpui::div().child("Tech Lead").into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                    ])
                    .into_any_element()]),
                ),
                (
                    "Empty State", "An empty collection renders the empty message in place of rows.",
                    stretch_col(vec![
                        h::Table::new(vec!["Name".into(), "Role".into()])
                            .id("tbl-empty-state")
                            .empty_state("No results found")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Loading", "The pending skeleton covers the table while a request is in flight.",
                    stretch_col(vec![
                        build("tbl-loading")
                            .is_pending(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
