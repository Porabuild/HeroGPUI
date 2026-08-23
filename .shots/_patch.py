"""Calendar and RangeCalendar pages: the examples they were missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# ---------------------------------------------------------------- Calendar
rep("""                (
                    "Constraints",""",
    """                (
                    "Default Value",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-default", cx))
                        .default_value(h::Date::new(2025, 12, 25))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Calendar::new(self.demo_calendar("cal-controlled", cx))
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Min and Max Dates",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-minmax", cx))
                        .min_value(h::Date::new(today.year, today.month, 5))
                        .max_value(h::Date::new(today.year, today.month, 20))
                        .into_any_element()]),
                ),
                (
                    "Unavailable Dates",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-unavailable", cx))
                        // Weekends are struck through, which is v3's own example.
                        .is_date_unavailable(|date| {
                            let weekday = h::weekday_index(date);
                            weekday == 0 || weekday == 6
                        })
                        .into_any_element()]),
                ),
                (
                    "Weeks in Month",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]),
                ),
                (
                    "Multiple Selection",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-multiple", cx))
                        .selection_mode(SelectionMode::Multiple)
                        .into_any_element()]),
                ),
                (
                    "Focused Value",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-focused", cx))
                        .focused_value(h::Date::new(today.year, today.month, 15))
                        .into_any_element()]),
                ),
                (
                    "Cell Indicators",
                    col(vec![
                        para("The marked days are the ones with events.", cx),
                        h::Calendar::new(self.demo_calendar("cal-indicators", cx))
                            .cell_indicator(|date| {
                                [3, 7, 12, 15, 21, 28].contains(&date.day)
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Navigation Icons",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-nav", cx))
                        .nav_icons(h::icons::ARROW_LEFT, h::icons::ARROW_RIGHT)
                        .into_any_element()]),
                ),
                (
                    "Real-World Example",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Pick an appointment"))
                        .child(
                            h::Calendar::new(self.demo_calendar("cal-real", cx))
                                .min_value(today)
                                .is_date_unavailable(|date| {
                                    let weekday = h::weekday_index(date);
                                    weekday == 0 || weekday == 6
                                })
                                .cell_indicator(|date| date.day % 5 == 0),
                        )
                        .child(h::Description::new(
                            "Weekends are unavailable; a dot marks a day with slots left.",
                        ))
                        .into_any_element()]),
                ),
                (
                    "Constraints",""")

# ----------------------------------------------------------- RangeCalendar
rep("""            crate::pages::Page::RangeCalendar.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::RangeCalendar.import_line(),
            vec![
                (
                    "Disabled",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-disabled", cx))
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Year Picker",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-year", cx))
                        .default_year_picker_open(true)
                        .into_any_element()]),
                ),
                (
                    "Default Value",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-default", cx))
                        .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        para("The range lives in the state entity the caller owns.", cx),
                        h::RangeCalendar::new(self.date_range.clone()).into_any_element(),
                    ]),
                ),
                (
                    "Min and Max Dates",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-minmax", cx))
                        .min_value(h::Date::new(today.year, today.month, 5))
                        .max_value(h::Date::new(today.year, today.month, 24))
                        .into_any_element()]),
                ),
                (
                    "Unavailable Dates",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-unavailable", cx))
                        .is_date_unavailable(|date| {
                            let weekday = h::weekday_index(date);
                            weekday == 0 || weekday == 6
                        })
                        .into_any_element()]),
                ),
                (
                    "Anchor-Based Unavailable Dates",
                    col(vec![
                        para(
                            "A range cannot cross an unavailable day unless \\
                             `allowsNonContiguousRanges` says it may, so the anchor decides how \\
                             far the selection reaches.",
                            cx,
                        ),
                        h::RangeCalendar::new(self.demo_range("rc-anchor", cx))
                            .is_date_unavailable(|date| date.day == 15)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Allows Non-Contiguous Ranges",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-noncontig", cx))
                        .is_date_unavailable(|date| date.day == 15)
                        .allows_non_contiguous_ranges(true)
                        .into_any_element()]),
                ),
                (
                    "Weeks in Month",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]),
                ),
                (
                    "Week View",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-week-view", cx))
                        .visible_duration(h::VisibleDuration::Weeks(2))
                        .into_any_element()]),
                ),
                (
                    "Day View",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-day-view", cx))
                        .visible_duration(h::VisibleDuration::Days(5))
                        .into_any_element()]),
                ),
                (
                    "Multiple Months",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-months", cx))
                        .visible_duration(h::VisibleDuration::Months(2))
                        .into_any_element()]),
                ),
                (
                    "Read Only",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-readonly", cx))
                        .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                        .is_read_only(true)
                        .into_any_element()]),
                ),
                (
                    "Invalid",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-invalid", cx))
                        .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                        .is_invalid(true)
                        .into_any_element()]),
                ),
                (
                    "Focused Value",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-focused", cx))
                        .focused_value(h::Date::new(today.year, today.month, 15))
                        .into_any_element()]),
                ),
                (
                    "Cell Indicators",
                    col(vec![
                        para(
                            "A `RangeCalendar` marks its own days: the range's ends and every \\
                             day between them.",
                            cx,
                        ),
                        h::RangeCalendar::new(self.demo_range("rc-indicators", cx))
                            .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Real-World Example",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Choose your stay"))
                        .child(
                            h::RangeCalendar::new(self.demo_range("rc-real", cx))
                                .min_value(today)
                                .is_date_unavailable(|date| date.day == 20),
                        )
                        .child(h::Description::new("The 20th is fully booked."))
                        .into_any_element()]),
                ),
                (
                    "Usage",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched calendar + range calendar pages')
