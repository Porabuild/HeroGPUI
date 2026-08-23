"""Dropdown page: the fifteen v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""        let items = vec![
            h::MenuItem::new("new", "New file").shortcut("Ctrl N"),
            h::MenuItem::new("copy", "Copy link").shortcut("Ctrl C"),
            h::MenuItem::Separator,
            h::MenuItem::new("delete", "Delete file").danger(),
        ];""",
    """        let items = vec![
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
        let dd_multi = self.dropdown_multi.clone();""")

rep("""            crate::pages::Page::Dropdown.import_line(),
            vec![
                (
                    "Usage",""",
    """            crate::pages::Page::Dropdown.import_line(),
            vec![
                (
                    "With Icons",
                    col(vec![h::Dropdown::uncontrolled(
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
                    .into_any_element()]),
                ),
                (
                    "With Descriptions",
                    col(vec![h::Dropdown::uncontrolled(
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
                    .into_any_element()]),
                ),
                (
                    "With Disabled Items",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Button::new("dd-disabled")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        plain(),
                    )
                    .disabled_keys([SharedString::from("save")])
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::Dropdown::uncontrolled(
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
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Dropdown::new(
                            h::Button::new("dd-controlled")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
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
                            h::Button::new("dd-open")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Single Selection",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Button::new("dd-single")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                            h::MenuItem::new("size", "Size"),
                        ],
                    )
                    .selection_mode(SelectionMode::Single)
                    .selected_key("date")
                    .into_any_element()]),
                ),
                (
                    "Single With Custom Indicator",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Button::new("dd-single-ind")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                        ],
                    )
                    .selection_mode(SelectionMode::Single)
                    .selected_key("name")
                    .indicator(h::IndicatorKind::Dot)
                    .into_any_element()]),
                ),
                (
                    "With Section Level Selection",
                    col(vec![h::Dropdown::uncontrolled(
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
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(dd_multi.clone())
                    .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.dropdown_multi = keys.to_vec();
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "With Keyboard Shortcuts",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Button::new("dd-shortcuts")
                            .label("Edit")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("cut", "Cut").shortcut("Ctrl X"),
                            h::MenuItem::new("copy", "Copy").shortcut("Ctrl C"),
                            h::MenuItem::new("paste", "Paste").shortcut("Ctrl V"),
                        ],
                    )
                    .into_any_element()]),
                ),
                (
                    "With Submenus",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Button::new("dd-submenu")
                            .label("Share")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("link", "Copy link"),
                            h::MenuItem::new("email", "Email"),
                            h::MenuItem::new("other", "Other").submenu(vec![
                                h::MenuItem::new("sms", "SMS"),
                                h::MenuItem::new("airdrop", "AirDrop"),
                                h::MenuItem::new("more", "More\\u{2026}"),
                            ]),
                        ],
                    )
                    .into_any_element()]),
                ),
                (
                    "With Custom Submenu Indicator",
                    col(vec![
                        para(
                            "`Dropdown.SubmenuIndicator` is the chevron on a row that opens \\
                             another panel; hover the row to open it.",
                            cx,
                        ),
                        h::Dropdown::uncontrolled(
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
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Trigger",
                    col(vec![h::Dropdown::uncontrolled(
                        h::Avatar::new().name("Jane Doe"),
                        vec![
                            h::MenuItem::new("profile", "Profile"),
                            h::MenuItem::new("settings", "Settings"),
                            h::MenuItem::Separator,
                            h::MenuItem::new("logout", "Log out").danger(),
                        ],
                    )
                    .into_any_element()]),
                ),
                (
                    "Long Press Trigger",
                    col(vec![
                        para("Hold the button for half a second.", cx),
                        h::Dropdown::uncontrolled(
                            h::Button::new("dd-long")
                                .label("Long press")
                                .variant(Variant::Secondary),
                            plain(),
                        )
                        .trigger(h::DropdownTrigger::LongPress)
                        .into_any_element(),
                    ]),
                ),
                (
                    "Usage",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched dropdown page')
