//! Collections gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Collections
    // -----------------------------------------------------------------------

    pub fn page_dropdown(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.dropdown_open;
        let selected = self
            .dropdown_selected
            .clone()
            .unwrap_or_else(|| SharedString::from("none"));
        let items = vec![
            h::MenuItem::new("new", "New file").shortcut("Ctrl N"),
            h::MenuItem::new("copy", "Copy link").shortcut("Ctrl C"),
            h::MenuItem::Separator,
            h::MenuItem::new("delete", "Delete file").danger(),
        ];
        // A `MenuItem` is moved into the menu that shows it, so each demo builds
        // its own list.
        let plain = || {
            vec![
                h::MenuItem::new("new", "New file"),
                h::MenuItem::new("open", "Open file"),
                h::MenuItem::new("save", "Save"),
            ]
        };
        let dd_multi = self.dropdown_multi.clone();
        component_doc_page!(
            "Dropdown",
            crate::pages::Page::Dropdown.description(),
            crate::pages::Page::Dropdown.import_line(),
            vec![
                (
                    "Usage", "The menu positions against the measured trigger: all eight placements flip to the side with more room when the preferred side cannot fit, keep a 12px cross-axis viewport inset, and cap the scroller to the available height with short menus keeping their natural height.",
                    col(vec![
                        h::Dropdown::new(
                            "dd-trigger-dd",
                            h::Button::new("dd-trigger")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            items,
                            is_open,
                        )
                        .id("dd-trigger-dd")
                        .radius(px(4.))
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        }))
                        .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.dropdown_selected = Some(key.clone());
                            this.dropdown_open = false;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(&format!("Last action: {selected}"), cx),
                    ]),
                ),
                (
                    "Row Hover",
                    "`row_hover_bg` names the fill a hovered menu row takes, in place of `--default`.",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-hover-trigger",
                        h::Button::new("dd-hover")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        plain(),
                    )
                    .row_hover_bg(cx.colors().accent.soft())
                    .into_any_element()]),
                ),
                (
                    "Standalone Compact Menu", "An embedded panel with 28px rows, 12px text and caller-owned dismissal. Its submenu inherits presentation and actions.",
                    col(vec![
                        h::Button::new("dd-standalone-show").label("Show menu")
                            .on_press(cx.listener(|this, _, _, cx| { this.set_demo_flag("dd-standalone", true); cx.notify(); })).into_any_element(),
                        if self.demo_flag("dd-standalone", false) {
                            h::Menu::new("dd-standalone", vec![h::MenuItem::new("tools", "Tools")
                                .submenu(vec![h::MenuItem::new("copy", "Copy")])])
                                .panel_min_width(px(180.)).panel_max_width(px(240.)).panel_max_height(px(160.))
                                .row_height(px(28.)).row_padding_x(px(8.)).row_padding_y(px(2.))
                                .row_text_size(px(12.)).row_gap(px(8.)).panel_padding(px(4.)).panel_gap(px(0.))
                                .animate_entry(false)
                                .row_hover_bg(cx.colors().accent.color)
                                .row_hover_foreground(cx.colors().accent.foreground)
                                .on_action(cx.listener(|this, key: &SharedString, _, cx| { this.dropdown_selected = Some(key.clone()); cx.notify(); }))
                                .on_dismiss(cx.listener(|this, _: &bool, _, cx| { this.set_demo_flag("dd-standalone", false); cx.notify(); }))
                                .into_any_element()
                        } else { gpui::div().into_any_element() },
                    ]),
                ),
                (
                    "With Icons",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-icons-dd",
                        h::Button::new("dd-icons")
                            .label("File")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("new", "New file").icon(h::icons::PLUS),
                            h::MenuItem::new("copy", "Copy").icon(h::icons::COPY),
                            h::MenuItem::new("delete", "Delete")
                                .icon(h::icons::CLOSE)
                                .danger(),
                        ],
                    )
                    .id("dd-icons-dd")
                    .into_any_element()]),
                ),
                (
                    "With Descriptions", "Labels use 14px text with 20px lines; descriptions and section headers use 12px text with 16px lines.",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-desc-dd",
                        h::Button::new("dd-desc")
                            .label("Merge")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("merge", "Create a merge commit").description(
                                "All commits from this branch are added to the base branch",
                            ),
                            h::MenuItem::new("squash", "Squash and merge")
                                .description("The commits are combined into one"),
                            h::MenuItem::new("rebase", "Rebase and merge")
                                .description("The commits are rebased onto the base branch"),
                        ],
                    )
                    .id("dd-desc-dd")
                    .placement(h::Placement::BottomStart)
                    .into_any_element()]),
                ),
                (
                    "With Disabled Items",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-disabled-dd",
                        h::Button::new("dd-disabled")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        plain(),
                    )
                    .id("dd-disabled-dd")
                    .disabled_keys([SharedString::from("save")])
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-sections-dd",
                        h::Button::new("dd-sections")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::SectionLabel("File".into()),
                            h::MenuItem::new("new", "New file"),
                            h::MenuItem::new("open", "Open file"),
                            h::MenuItem::Separator,
                            h::MenuItem::SectionLabel("Danger".into()),
                            h::MenuItem::new("delete", "Delete").danger(),
                        ],
                    )
                    .id("dd-sections-dd")
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Dropdown::new(
                            "dd-controlled-dd",
                            h::Button::new("dd-controlled")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .id("dd-controlled-dd")
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        }))
                        .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.dropdown_selected = Some(key.clone());
                            this.dropdown_open = false;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(&format!("Last action: {selected}"), cx),
                    ]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("dd-open-btn")
                                .label(if is_open { "Close menu" } else { "Open menu" })
                                .size(Size::Sm)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.dropdown_open = !this.dropdown_open;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if is_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Dropdown::new(
                            "dd-open-dd",
                            h::Button::new("dd-open")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .id("dd-open-dd")
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        }))
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Single Selection",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-single-dd",
                        h::Button::new("dd-single")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                            h::MenuItem::new("size", "Size"),
                        ],
                    )
                    .id("dd-single-dd")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("date")])
                    .into_any_element()]),
                ),
                (
                    "Single With Custom Indicator",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-single-ind-dd",
                        h::Button::new("dd-single-ind")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                        ],
                    )
                    .id("dd-single-ind-dd")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("name")])
                    .indicator(h::IndicatorKind::Dot)
                    .into_any_element()]),
                ),
                (
                    "Render Props", "The composed Dropdown forwards item and indicator render state into the live menu. Open it and choose rows to watch the selection move.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-render-props-dd",
                            h::Button::new("dd-render-props")
                                .label("Render state")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("name", "Name"),
                                h::MenuItem::new("date", "Date"),
                                h::MenuItem::new("size", "Size"),
                            ],
                        )
                        .selection_mode(SelectionMode::Multiple)
                        .default_selected_keys([SharedString::from("name")])
                        .item_content(|key, state| {
                            gpui::div()
                                .child(format!(
                                    "{key}: {}",
                                    if state.is_selected {
                                        "selected"
                                    } else {
                                        "idle"
                                    }
                                ))
                                .into_any_element()
                        })
                        .indicator_content(|_, selected, _| {
                            gpui::div()
                                .w(px(16.))
                                .child(if selected { "✓" } else { "" })
                                .into_any_element()
                        })
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Section Level Selection",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-section-sel-dd",
                        h::Button::new("dd-section-sel")
                            .label("View")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::SectionLabel("Sort".into()),
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                            h::MenuItem::Separator,
                            h::MenuItem::SectionLabel("Show".into()),
                            h::MenuItem::new("hidden", "Hidden files"),
                        ],
                    )
                    .id("dd-section-sel-dd")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(dd_multi)
                    .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.dropdown_multi = keys.to_vec();
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "With Keyboard Shortcuts",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-shortcuts-dd",
                        h::Button::new("dd-shortcuts")
                            .label("Edit")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("cut", "Cut").shortcut("Ctrl X"),
                            h::MenuItem::new("copy", "Copy").shortcut("Ctrl C"),
                            h::MenuItem::new("paste", "Paste").shortcut("Ctrl V"),
                        ],
                    )
                    .id("dd-shortcuts-dd")
                    .into_any_element()]),
                ),
                (
                    "With Submenus", "Submenus position independently against their row end top and flip sides to the side with more room when needed; the parent stays anchored and outside presses still dismiss the whole menu.",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-submenu-dd",
                        h::Button::new("dd-submenu")
                            .label("Share")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("link", "Copy link"),
                            h::MenuItem::new("email", "Email"),
                            h::MenuItem::new("other", "Other").submenu(vec![
                                h::MenuItem::new("sms", "SMS"),
                                h::MenuItem::new("airdrop", "AirDrop"),
                                h::MenuItem::new("more", "More\u{2026}"),
                            ]),
                        ],
                    )
                    .id("dd-submenu-dd")
                    .into_any_element()]),
                ),
                (
                    "With Custom Submenu Indicator", "`Dropdown.SubmenuIndicator` is the chevron on a row that opens another panel; hover the row to open it.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-submenu-ind-dd",
                            h::Button::new("dd-submenu-ind")
                                .label("More")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("profile", "Profile"),
                                h::MenuItem::new("workspace", "Workspace").submenu(vec![
                                    h::MenuItem::new("members", "Members"),
                                    h::MenuItem::new("billing", "Billing"),
                                ]),
                            ],
                        )
                        .id("dd-submenu-ind-dd")
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Trigger",
                    col(vec![h::Dropdown::uncontrolled(
                        "Jane Doe-dd",
                        h::Avatar::new("dd-trigger-avatar").name("Jane Doe"),
                        vec![
                            h::MenuItem::new("profile", "Profile"),
                            h::MenuItem::new("settings", "Settings"),
                            h::MenuItem::Separator,
                            h::MenuItem::new("logout", "Log out").danger(),
                        ],
                    )
                    .id("Jane Doe-dd")
                    .into_any_element()]),
                ),
                (
                    "Long Press Trigger", "Hold the button for half a second.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-long-dd",
                            h::Button::new("dd-long")
                                .label("Long press")
                                .variant(Variant::Secondary),
                            plain(),
                        )
                        .id("dd-long-dd")
                        .trigger(h::DropdownTrigger::LongPress)
                        .into_any_element(),
                    ]),
                ),
                (
                    "Basic Usage", "The plainest action menu: `on_action` receives the chosen key, and a danger item renders in the danger palette.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-basic-dd",
                            h::Button::new("dd-basic")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("new-file", "New file"),
                                h::MenuItem::new("open-file", "Open file"),
                                h::MenuItem::new("delete-file", "Delete file").danger(),
                            ],
                        )
                        .id("dd-basic-dd")
                        .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.dropdown_last_basic = key.clone();
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(
                            &format!("Last action: {}", self.dropdown_last_basic),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled Selection", "The selected keys live in the caller and are fed back through `selected_keys`; the caption reads the same set the menu renders.",
                    col(vec![
                        h::Dropdown::new(
                            "dd-controlled-sel-dd",
                            h::Button::new("dd-controlled-sel")
                                .label("Formatting")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("bold", "Bold"),
                                h::MenuItem::new("italic", "Italic"),
                            ],
                            self.dropdown_open,
                        )
                        .id("dd-controlled-sel-dd")
                        .selection_mode(SelectionMode::Multiple)
                        .selected_keys(self.dropdown_doc_marks.clone())
                        .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.dropdown_doc_marks = keys.to_vec();
                            cx.notify();
                        }))
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(
                            &format!(
                                "Active: {}",
                                if self.dropdown_doc_marks.is_empty() {
                                    "none".to_owned()
                                } else {
                                    self.dropdown_doc_marks
                                        .iter()
                                        .map(|k| k.as_ref())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                }
                            ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Multiple Selection",
                    col(vec![
                        h::Dropdown::new(
                            "dd-multi-dd",
                            h::Button::new("dd-multi-trigger")
                                .label("Columns")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("name", "Name"),
                                h::MenuItem::new("role", "Role"),
                                h::MenuItem::new("status", "Status"),
                            ],
                            self.dropdown_open,
                        )
                        .id("dd-multi-trigger-dd")
                        .selection_mode(SelectionMode::Multiple)
                        .selected_keys(self.dropdown_multi.clone())
                        .disabled_keys(vec!["status"])
                        .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.dropdown_multi = keys.to_vec();
                            cx.notify();
                        }))
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(
                            &format!("Showing {} columns", self.dropdown_multi.len()),
                            cx,
                        ),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_list_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selection = self.list_selection.clone();
        let items = vec![
            h::ListBoxItem::section("Mail"),
            h::ListBoxItem::new("inbox", "Inbox").description("24 unread"),
            h::ListBoxItem::new("sent", "Sent"),
            h::ListBoxItem::new("drafts", "Drafts").is_disabled(true),
            h::ListBoxItem::separator(),
            h::ListBoxItem::new("trash", "Move to trash").danger(),
        ];
        component_doc_page!(
            "List Box",
            crate::pages::Page::ListBox.description(),
            crate::pages::Page::ListBox.import_line(),
            vec![
                (
                    "Usage", "Labels use 14px text with 20px lines; descriptions and section headers use 12px text with 16px lines.",
                    col(vec![gpui::div()
                        .w(px(220.))
                        .child(h::ListBox::new(
                            "lb-usage",
                            vec![
                                h::ListBoxItem::new("inbox", "Inbox"),
                                h::ListBoxItem::new("sent", "Sent"),
                                h::ListBoxItem::new("drafts", "Drafts"),
                            ],
                        )
                        .selection_mode(SelectionMode::Single))
                        .into_any_element()]),
                ),
                (
                    "Row Padding",
                    "`row_padding_x` / `row_padding_y` resize the option rows; section headings keep their own inset.",
                    col(vec![gpui::div()
                        .w(px(220.))
                        .child(
                            h::ListBox::new(
                                "lb-row-padding",
                                vec![
                                    h::ListBoxItem::new("a", "Alpha"),
                                    h::ListBoxItem::new("b", "Beta"),
                                ],
                            )
                            .row_padding_x(px(16.))
                            .row_padding_y(px(2.))
                            .row_hover_bg(cx.colors().accent.soft()),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Disabled Items",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-disabled",
                                vec![
                                    h::ListBoxItem::new("inbox", "Inbox"),
                                    h::ListBoxItem::new("sent", "Sent"),
                                    h::ListBoxItem::new("drafts", "Drafts"),
                                ],
                            )
                            .selection_mode(SelectionMode::None)
                            .disabled_keys([SharedString::from("drafts")]),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![gpui::div()
                        .w(px(256.))
                        .child(h::ListBox::new(
                            "lb-sections",
                            vec![
                                h::ListBoxItem::section("Actions"),
                                h::ListBoxItem::new("new-file", "New file")
                                    .description("Create a new file")
                                    .shortcut("⌘ N"),
                                h::ListBoxItem::new("edit-file", "Edit file")
                                    .description("Make changes")
                                    .shortcut("⌘ E"),
                                h::ListBoxItem::separator(),
                                h::ListBoxItem::section("Danger zone"),
                                h::ListBoxItem::new("delete-file", "Delete file")
                                    .description("Move to trash")
                                    .shortcut("⌘ ⇧ D")
                                    .danger(),
                            ],
                        ).on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.set_demo_text_value("lb-section-action", key.to_string());
                            cx.notify();
                        })))
                        .into_any_element(), para(&format!("Selected item: {}", self.demo_text_value("lb-section-action")), cx)]),
                ),
                (
                    "Multi Select",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-multi-select",
                                vec![
                                    h::ListBoxItem::new("inbox", "Inbox"),
                                    h::ListBoxItem::new("sent", "Sent"),
                                    h::ListBoxItem::new("spam", "Spam"),
                                ],
                            )
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(selection.iter().cloned())
                            .on_selection_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.list_selection = keys.clone();
                                    cx.notify();
                                },
                            )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-controlled",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                        h::ListBoxItem::new("spam", "Spam"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Multiple)
                                .selected_keys(selection.iter().cloned())
                                // `onAction` fires on a press, selection or not.
                                .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("lb-action", key.to_string());
                                    cx.notify();
                                }))
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                            )
                            .into_any_element(),
                        para(&format!("{} selected", selection.len()), cx),
                    ]),
                ),
                (
                    "Disallow Empty Selection", "The inherited selection policy keeps the final selected row selected, including when Escape would otherwise clear the collection.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-keep-selection",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Single)
                                .default_selected_keys([SharedString::from("inbox")])
                                .disallow_empty_selection(true),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Escape Key Behavior", "Press Escape while this list is focused. The `None` policy preserves the selection and leaves Escape available to an enclosing surface.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-escape-none",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Single)
                                .default_selected_keys([SharedString::from("inbox")])
                                .escape_key_behavior(h::EscapeKeyBehavior::None),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable instead of laid out, so gpui's `uniform_list` builds only the rows in view — one thousand users, fifty pixels each. The fixed-row list caps at `max_h`, shrinks below it in a bounded parent, and PageUp/PageDown move by the visible viewport, including after resize, while skipping disabled stops.",
                    col(vec![
                        gpui::div()
                            .w(px(300.))
                            .child(
                                h::ListBox::new("lb-virtual", virtual_users())
                                    .selection_mode(SelectionMode::None)
                                    .row_height(px(50.))
                                    .max_h(px(400.)),
                            )
                            .into_any_element(),
                        gpui::div()
                            .w(px(300.))
                            .child(
                                h::ListBox::new("lb-virtual-var", virtual_users_described())
                                    .selection_mode(SelectionMode::None)
                                    .estimated_row_height(px(44.))
                                    .heading_height(px(28.))
                                    .max_h(px(400.)),
                            )
                            .into_any_element(),
                        para(
                            "`estimated_row_height` is the other half: rows that are \
                             *not* all one height, measured as they are built. Every \
                             third row here carries a description, so it is taller.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Check Icon", "There is no separate indicator part; a row's `variant` carries the indicator style, so the danger row below shows the same tick in its own colour.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-check",
                                    vec![
                                        h::ListBoxItem::new("keep", "Keep"),
                                        h::ListBoxItem::new("delete", "Delete").danger(),
                                    ],
                                )
                                .selection_mode(SelectionMode::Multiple)
                                .default_selected_keys([SharedString::from("keep")]),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Single selection",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new("lb-single", items.clone())
                                .selection_mode(SelectionMode::Single)
                                .selected_keys(selection.iter().cloned())
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new("lb-multi", items)
                                .selection_mode(SelectionMode::Multiple)
                                .selected_keys(selection.iter().cloned())
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Basic Usage", "The plainest list: single selection over rows that carry a label and a description, uncontrolled.",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-basic",
                                vec![
                                    h::ListBoxItem::new("bob", "Bob")
                                        .description("bob@example.com"),
                                    h::ListBoxItem::new("alice", "Alice")
                                        .description("alice@example.com"),
                                ],
                            )
                            .selection_mode(SelectionMode::Single),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled Selection", "The selected keys live in the caller and are fed back through `selected_keys`; the caption reads the same set the list renders.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-controlled-selection",
                                    vec![
                                        h::ListBoxItem::new("opt-1", "Option 1"),
                                        h::ListBoxItem::new("opt-2", "Option 2"),
                                        h::ListBoxItem::new("opt-3", "Option 3"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Multiple)
                                .selected_keys(self.lb_pair_selection.iter().cloned())
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.lb_pair_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                            )
                            .into_any_element(),
                        para(&format!("{} selected", self.lb_pair_selection.len()), cx),
                    ]),
                ),
                (
                    "Custom Indicator", "The pair-page twin of the check-icon example: a row's `variant` restyles its selection indicator, here a danger tick on the delete row.",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-indicator",
                                vec![
                                    h::ListBoxItem::new("archive", "Archive"),
                                    h::ListBoxItem::new("discard", "Discard").danger(),
                                ],
                            )
                            .selection_mode(SelectionMode::Multiple)
                            .default_selected_keys([SharedString::from("archive")]),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_tag_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let tag_keys = self.tags.clone();
        // Each demo needs its own list, so this is a factory rather than one
        // vector: a `Tag` is moved into the group that shows it.
        let tags = move || -> Vec<h::Tag> {
            tag_keys
                .iter()
                .map(|k| h::Tag::new(k.clone(), title_case(k)))
                .collect()
        };
        let selection = self.tag_selection.clone();
        let tag_selection = selection.clone();
        component_doc_page!(
            "Tag Group",
            crate::pages::Page::TagGroup.description(),
            crate::pages::Page::TagGroup.import_line(),
            vec![
                (
                    "Usage", "`radius(px)` replaces the chip's size-step corner.",
                    col(vec![h::TagGroup::new("tg-usage", tags())
                        .full_width(true)
                        .label("Skills")
                        .radius(px(2.))
                        .into_any_element()]),
                ),
                (
                    "Hover Colour",
                    "`hover_bg` names a hovered tag's fill and `remove_hover_bg` the remove button's, which stays a circle.",
                    col(vec![h::TagGroup::new("tg-hover-bg", tags())
                        .label("Skills")
                        .selection_mode(SelectionMode::Single)
                        .hover_bg(cx.colors().accent.soft())
                        .remove_hover_bg(cx.colors().accent.soft_hover())
                        .on_remove(cx.listener(
                            |this, keys: &HashSet<SharedString>, _, cx| {
                                this.tags.retain(|key| !keys.contains(key));
                                this.tag_selection.retain(|key| !keys.contains(key));
                                cx.notify();
                            },
                        ))
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![
                        h::TagGroup::new("tg-disabled", tags())
                            .label("Skills")
                            .is_disabled(true)
                            .into_any_element(),
                        h::TagGroup::new("tg-disabled-keys", tags())
                            .label("Some disabled")
                            // `disabledKeys` disables individual tags rather
                            // than the whole group.
                            .disabled_keys([SharedString::from("rust")])
                            .into_any_element(),
                    ]),
                ),
                (
                    "Selection Modes",
                    col(vec![
                        spec(
                            "Single",
                            h::TagGroup::new("tg-single", tags())
                                .selection_mode(SelectionMode::Single),
                            cx,
                        ),
                        spec(
                            "Multiple",
                            h::TagGroup::new("tg-multiple", tags())
                                .selection_mode(SelectionMode::Multiple),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::TagGroup::new("tg-controlled", tags())
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(tag_selection.iter().cloned())
                            .on_selection_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.tag_selection = keys.clone();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", tag_selection.len()), cx),
                    ]),
                ),
                (
                    "Disallow Empty Selection",
                    col(vec![h::TagGroup::new(
                        "tg-disallow-empty",
                        vec![h::Tag::new("design", "Design"), h::Tag::new("code", "Code")],
                    )
                    .label("At least one skill")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("design")])
                    .disallow_empty_selection(true)
                    .into_any_element()]),
                ),
                (
                    "Escape Key Behavior", "With `None`, Escape preserves the selected tag and bubbles to any enclosing interaction that handles it.",
                    col(vec![
                        h::TagGroup::new(
                            "tg-escape-none",
                            vec![h::Tag::new("design", "Design"), h::Tag::new("code", "Code")],
                        )
                        .label("Skills")
                        .selection_mode(SelectionMode::Single)
                        .default_selected_keys([SharedString::from("design")])
                        .escape_key_behavior(h::EscapeKeyBehavior::None)
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Error Message",
                    col(vec![h::TagGroup::new("tg-error", tags())
                        .label("Skills")
                        .description("Pick at least one")
                        .into_any_element()]),
                ),
                (
                    "With List Data",
                    col(vec![h::TagGroup::new(
                        "tg-list",
                        ["Design", "Research", "Writing", "Support", "Ops"]
                            .into_iter()
                            .map(|name| h::Tag::new(name.to_lowercase(), name))
                            .collect(),
                    )
                    .label("Teams")
                    .into_any_element()]),
                ),
                (
                    "With Prefix",
                    col(vec![h::TagGroup::new(
                        "tg-prefix",
                        vec![
                            h::Tag::new("rust", "Rust").icon(h::icons::CHECK),
                            h::Tag::new("gpui", "GPUI").icon(h::icons::CHECK),
                        ],
                    )
                    .label("Verified")
                    .into_any_element()]),
                ),
                (
                    "With Remove Button",
                    "The 24px remove target extends around the 12px icon without changing tag spacing.",
                    col(vec![
                        spec(
                            "Default remove button",
                            h::TagGroup::new("tg-remove-button", tags())
                                .label("Skills")
                                .on_remove(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.tags.retain(|key| !keys.contains(key));
                                        this.tag_selection.retain(|key| !keys.contains(key));
                                        cx.notify();
                                    },
                                )),
                            cx,
                        ),
                        spec(
                            "Custom remove button",
                            {
                                let custom_tags: Vec<h::Tag> = tags()
                                    .into_iter()
                                    .map(|tag| {
                                        tag.remove_content(|| {
                                            gpui::div().child("−").into_any_element()
                                        })
                                    })
                                    .collect();
                                h::TagGroup::new("tg-custom-remove-button", custom_tags)
                                    .label("Skills")
                                    .on_remove(cx.listener(
                                        |this, keys: &HashSet<SharedString>, _, cx| {
                                            this.tags.retain(|key| !keys.contains(key));
                                            this.tag_selection.retain(|key| !keys.contains(key));
                                            cx.notify();
                                        },
                                    ))
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Removable",
                    col(vec![h::TagGroup::new("tg-remove", tags())
                        .label("Team")
                        .description("Remove a tag to see the group update.")
                        .empty_state("All tags removed")
                        .on_remove(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.tags.retain(|key| !keys.contains(key));
                            this.tag_selection.retain(|key| !keys.contains(key));
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Selectable",
                    col(vec![h::TagGroup::new("tg-select", tags())
                        .selection_mode(SelectionMode::Multiple)
                        .selected_keys(selection.iter().cloned())
                        .on_selection_change(cx.listener(
                            |this, keys: &HashSet<SharedString>, _, cx| {
                                this.tag_selection = keys.clone();
                                cx.notify();
                            },
                        ))
                        .into_any_element()]),
                ),
                (
                    "Sizes",
                    "Small and medium tags use 16px line boxes; large tags use 20px, independent of inherited text styles.",
                    col(vec![
                        h::TagGroup::new("tg-sm", tags())
                            .size(Size::Sm)
                            .into_any_element(),
                        h::TagGroup::new("tg-md", tags())
                            .size(Size::Md)
                            .into_any_element(),
                        h::TagGroup::new("tg-lg", tags())
                            .size(Size::Lg)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants", "The default variant draws each tag as its own bordered pill; the surface variant tints the tag group so it reads inside a panel.",
                    col(vec![
                        h::TagGroup::new("tg-default", tags()).into_any_element(),
                        h::TagGroup::new("tg-surface", tags())
                            .variant(h::TagVariant::Surface)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
