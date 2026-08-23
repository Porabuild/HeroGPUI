"""Table page: the Expandable Rows example, now that tree rows exist."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""                (
                    "Secondary Variant",""",
    """                (
                    "Expandable Rows",
                    col(vec![
                        para(
                            "A row's children are nested under it, and `expandedKeys` decides \\
                             which parents show theirs. The chevron sits in the tree column.",
                            cx,
                        ),
                        {
                            let cell = |text: &str| gpui::div().child(text.to_owned());
                            h::Table::new(vec![
                                "Title".into(),
                                "Type".into(),
                                "Modified".into(),
                            ])
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
                                .children(vec![h::TableRow::new(vec![
                                    cell("Holiday.jpg").into_any_element(),
                                    cell("Image").into_any_element(),
                                    cell("5/5/2025").into_any_element(),
                                ])
                                .key("holiday")]),
                            )
                            .into_any_element()
                        },
                    ]),
                ),
                (
                    "Secondary Variant",""")

rep("""    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let table_page = self.demo_value("tbl-page", 1.) as usize;""",
    """    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let table_page = self.demo_value("tbl-page", 1.) as usize;
        let tbl_expanded = self.demo_selection("tbl-expanded");""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched table page')
