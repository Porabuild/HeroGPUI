"""Table: Secondary Variant, Async Loading, Pagination, Custom Cells."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""                (
                    "Empty and loading",
                    col(vec![
                        h::Table::new(vec!["Name".into(), "Role".into()])
                            .empty_state("Nobody here yet")
                            .into_any_element(),
                        build().is_pending(true).into_any_element(),
                    ]),
                ),
            ],""",
    """                (
                    "Secondary Variant",
                    col(vec![build()
                        .variant(h::TableVariant::Secondary)
                        .into_any_element()]),
                ),
                (
                    "Async Loading",
                    col(vec![
                        para(
                            "`isPending` covers the table while a request is in flight; \\
                             `onLoadMore` fires when the last row scrolls into view.",
                            cx,
                        ),
                        build()
                            .is_pending(true)
                            .on_load_more(|_, _| {})
                            .into_any_element(),
                    ]),
                ),
                (
                    "Pagination",
                    col(vec![
                        {
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
                                h::Table::new(vec!["Name".into(), "Role".into()]);
                            for (name, role) in people.iter().skip(start).take(2) {
                                paged = paged.row(vec![
                                    gpui::div().child(*name).into_any_element(),
                                    gpui::div().child(*role).into_any_element(),
                                ]);
                            }
                            paged.into_any_element()
                        },
                        h::Pagination::new("tbl-pages", 3)
                            .page(table_page)
                            .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                                this.set_demo_value("tbl-page", *p as f32);
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Cells",
                    col(vec![h::Table::new(vec![
                        "Member".into(),
                        "Role".into(),
                        "Status".into(),
                    ])
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(h::Avatar::new().name("Tony Reichert").size(Size::Sm))
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
                        h::Chip::new("Active")
                            .color(Color::Success)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .into_any_element(),
                    ])
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(h::Avatar::new().name("Zoey Lang").size(Size::Sm))
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
                        h::Chip::new("Paused")
                            .color(Color::Warning)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .into_any_element(),
                    ])
                    .into_any_element()]),
                ),
                (
                    "Empty and loading",
                    col(vec![
                        h::Table::new(vec!["Name".into(), "Role".into()])
                            .empty_state("Nobody here yet")
                            .into_any_element(),
                        build().is_pending(true).into_any_element(),
                    ]),
                ),
            ],""")

rep("""    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let build = || {""",
    """    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let table_page = self.demo_value("tbl-page", 1.) as usize;
        let build = || {""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched table page')
