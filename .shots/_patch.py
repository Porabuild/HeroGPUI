import io

p = 'gallery/src/pages/components.rs'
s = io.open(p, encoding='utf-8').read()

pairs = [
    # v3 composes the content *inside* the separator, which is what turns it
    # into `.separator__container`. The demo hand-rolled the layout instead.
    ("""                    "With Content",
                    col(vec![gpui::div()
                        .w_full()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(gpui::div().flex_1().child(h::Separator::new()))
                        .child(
                            gpui::div()
                                .text_size(px(12.))
                                .text_color(cx.colors().muted)
                                .child("OR"),
                        )
                        .child(gpui::div().flex_1().child(h::Separator::new()))
                        .into_any_element()]),""",
     """                    "With Content",
                    col(vec![h::Separator::new()
                        .child(gpui::div().text_size(px(12.)).child("OR"))
                        .into_any_element()]),"""),
    # v3's RadioGroup Usage puts a `<Description>` inside every `<Radio>`.
    ("""                    "Usage",
                    col(vec![h::RadioGroup::new("rg-usage", plans())
                        .default_value(Some(0))
                        .into_any_element()]),""",
     """                    "Usage",
                    col(vec![h::RadioGroup::new("rg-usage", plans())
                        .default_value(Some(0))
                        // v3 composes a `<Description>` inside each `<Radio>`.
                        .descriptions([
                            Some("Includes 100 messages per month"),
                            Some("Includes 200 messages per month"),
                            None,
                        ])
                        .into_any_element()]),"""),
]
for old, new in pairs:
    assert old in s, old[:80]
    s = s.replace(old, new, 1)

# The range calendar gets the same "Cell Indicators" demo the calendar has.
old = """                (
                    "Year Picker",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-year", cx))"""
new = """                (
                    "Cell Indicators",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-dots", cx))
                        // `RangeCalendar.CellIndicator` marks a day with a dot,
                        // the same part a `Calendar` draws.
                        .cell_indicator(|d| d.day % 7 == 3)
                        .into_any_element()]),
                ),
                (
                    "Year Picker",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-year", cx))"""
assert old in s
s = s.replace(old, new, 1)
io.open(p, 'w', encoding='utf-8', newline='\n').write(s)

p = '.shots/extra_audit.py'
s = io.open(p, encoding='utf-8').read()
old = """    # v3 composes `<Tabs.Separator />` inside the tab it precedes."""
new = """    # v3's stylesheet declares `.range-calendar__cell-indicator`; only the
    # Calendar's prop table names the part.
    'RangeCalendar.cell_indicator': 'composition',
    # v3 composes `<Tabs.Separator />` inside the tab it precedes."""
assert old in s
io.open(p, 'w', encoding='utf-8', newline='\n').write(s.replace(old, new, 1))
print('ok')
