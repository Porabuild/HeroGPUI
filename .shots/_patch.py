"""ListBox and TagGroup pages: every v3 example."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# ----------------------------------------------------------------- ListBox
rep("""            crate::pages::Page::ListBox.import_line(),
            vec![
                (
                    "Single selection",""",
    """            crate::pages::Page::ListBox.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(h::ListBox::new(
                            "lb-usage",
                            vec![
                                h::ListBoxItem::new("inbox", "Inbox"),
                                h::ListBoxItem::new("sent", "Sent"),
                                h::ListBoxItem::new("drafts", "Drafts"),
                            ],
                        ))
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
                            .disabled_keys([SharedString::from("drafts")]),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(h::ListBox::new(
                            "lb-sections",
                            vec![
                                h::ListBoxItem::section("Mail"),
                                h::ListBoxItem::new("inbox", "Inbox"),
                                h::ListBoxItem::new("sent", "Sent"),
                                h::ListBoxItem::separator(),
                                h::ListBoxItem::section("Archive"),
                                h::ListBoxItem::new("2024", "2024"),
                                h::ListBoxItem::new("2025", "2025"),
                            ],
                        ))
                        .into_any_element()]),
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
                                .selected_keys(selection.iter().cloned())
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
                    "Custom Check Icon",
                    col(vec![
                        para(
                            "v3 replaces `ListBox.ItemIndicator`. A row's `variant` is what \\
                             carries the indicator style here, so the danger row below shows the \\
                             same tick in its own colour.",
                            cx,
                        ),
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
                                .selected_key("keep"),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Single selection",""")

# ---------------------------------------------------------------- TagGroup
rep("""            crate::pages::Page::TagGroup.import_line(),
            vec![
                (
                    "Removable",""",
    """            crate::pages::Page::TagGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::TagGroup::new("tg-usage", tags())
                        .label("Skills")
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::TagGroup::new("tg-disabled", tags())
                        .label("Skills")
                        .is_disabled(true)
                        .into_any_element()]),
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
                    col(vec![h::TagGroup::new("tg-remove", tags())
                        .label("Skills")
                        .on_remove(cx.listener(|_, _key: &SharedString, _, cx| cx.notify()))
                        .into_any_element()]),
                ),
                (
                    "Removable",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched list box + tag group')
