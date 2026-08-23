"""Table page: the Column Resizing example."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""                (
                    "Expandable Rows",""",
    """                (
                    "Column Resizing",
                    col(vec![
                        para(
                            "Drag the divider on a resizable column's trailing edge. The width \\
                             is per column and survives the drag.",
                            cx,
                        ),
                        h::Table::new(vec![])
                            .column(
                                h::TableColumn::new("Name")
                                    .allows_resizing(true)
                                    .default_width(px(220.))
                                    .min_width(px(120.)),
                            )
                            .column(
                                h::TableColumn::new("Role")
                                    .allows_resizing(true)
                                    .default_width(px(180.)),
                            )
                            .column("Status")
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
                            .into_any_element(),
                    ]),
                ),
                (
                    "Expandable Rows",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched table page')
