//! Date and time gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Date and time
    // -----------------------------------------------------------------------

    pub fn page_calendar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let picked = self.cal_picked;
        let today = h::Date::today();
        let focused = self.calendar_focus;
        component_doc_page!(
            "Calendar",
            crate::pages::Page::Calendar.description(),
            crate::pages::Page::Calendar.import_line(),
            vec![
                (
                    "International Calendars",
                    "Indian and Hebrew calendar grids use English labels here. Navigation follows each calendar while selected values remain Gregorian dates.",
                    row(vec![
                        specimen_body(
                            "cal-indian",
                            spec(
                                "Indian calendar",
                                h::Calendar::new(self.demo_calendar("cal-indian", cx))
                                    .locale("en-US-u-ca-indian")
                                    .default_value(h::Date::new(2026, 1, 15))
                                    .into_any_element(),
                                cx,
                            ),
                            cx,
                        ),
                        specimen_body(
                            "cal-hebrew",
                            spec(
                                "Hebrew calendar",
                                h::Calendar::new(self.demo_calendar("cal-hebrew", cx))
                                    .locale("en-US-u-ca-hebrew")
                                    .default_value(h::Date::new(2024, 3, 25))
                                    .into_any_element(),
                                cx,
                            ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Usage",
                    "Day, month and year labels use 14px/20px medium text; weekday labels use 12px/16px medium text. Enabled navigation hover fills ease over the pinned 100ms curve while the deep press scale stays on the button.",
                    specimen_body("cal-usage", col(vec![
                        h::Calendar::new(self.demo_calendar("cal-usage", cx))
                            .day_hover_bg(cx.colors().accent.soft())
                            .nav_hover_bg(cx.colors().accent.soft())
                            .year_hover_bg(cx.colors().accent.soft())
                            .on_change(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match picked {
                                Some(d) => format!("Selected: {}", d.format_iso()),
                                None => "No date selected".to_owned(),
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Default Value",
                    "The seven 36px day columns align with the weekday headings without horizontal gaps.",
                    specimen_body("cal-default", col(vec![h::Calendar::new(self.demo_calendar("cal-default", cx))
                        .default_value(h::Date::new(2025, 12, 25))
                        .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("cal-controlled", col(vec![
                        h::Calendar::new(self.demo_calendar("cal-controlled", cx))
                            .on_focus_change(cx.listener(|this, d: &h::Date, _, cx| {
                                this.set_demo_text_value("cal-focus", d.format_iso());
                                cx.notify();
                            }))
                            .on_change(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Min and Max Dates",
                    specimen_body("cal-minmax", col(vec![h::Calendar::new(self.demo_calendar("cal-minmax", cx))
                        .min_value(h::Date::new(today.year, today.month, 5))
                        .max_value(h::Date::new(today.year, today.month, 20))
                        .into_any_element()]), cx),
                ),
                (
                    "Unavailable Dates",
                    specimen_body("cal-unavailable", col(vec![h::Calendar::new(self.demo_calendar("cal-unavailable", cx))
                        // Weekends are struck through, which is v3's own example.
                        .is_date_unavailable(|date| {
                            let weekday = h::weekday_index(date);
                            weekday == 0 || weekday == 6
                        })
                        .into_any_element()]), cx),
                ),
                (
                    "Weeks in Month",
                    specimen_body("cal-weeks", col(vec![h::Calendar::new(self.demo_calendar("cal-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]), cx),
                ),
                (
                    "Multiple Selection",
                    specimen_body("cal-multiple", col({
                        let cal = self.demo_calendar("cal-multiple", cx);
                        // v3's `onChange` carries the whole selection in
                        // multiple mode; the summary below re-reads the state,
                        // and `window.refresh()` forces the repaint that makes
                        // the new frame visible.
                        let dates = cal.read(cx).selected_dates().to_vec();
                        let summary = if dates.is_empty() {
                            "No dates selected".to_owned()
                        } else {
                            format!(
                                "Selected: {}",
                                dates
                                    .iter()
                                    .map(|d| d.format_iso())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };
                        vec![
                            h::Calendar::new(cal)
                                .selection_mode(SelectionMode::Multiple)
                                .on_change_all(|_, window, _| window.refresh())
                                .into_any_element(),
                            para(&summary, cx),
                        ]
                    }), cx),
                ),
                (
                    "Focused Value",
                    specimen_body("cal-focused", col(vec![h::Calendar::new(self.demo_calendar("cal-focused", cx))
                        .focused_value(focused)
                        .on_focus_change(cx.listener(
                            |this, date: &h::Date, _, cx| {
                                this.calendar_focus = *date;
                                cx.notify();
                            },
                        ))
                        .into_any_element()]), cx),
                ),
                (
                    "Cell Indicators", "The marked days are the ones with events.",
                    specimen_body("cal-indicators", col(vec![
                        h::Calendar::new(self.demo_calendar("cal-indicators", cx))
                            .cell_indicator(|date| {
                                [3, 7, 12, 15, 21, 28].contains(&date.day)
                            })
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Custom Navigation Icons",
                    specimen_body("cal-nav", col(vec![h::Calendar::new(self.demo_calendar("cal-nav", cx))
                        .nav_icons(h::icons::ARROW_LEFT, h::icons::ARROW_RIGHT)
                        .into_any_element()]), cx),
                ),
                (
                    "Real-World Example",
                    specimen_body("cal-real", col(vec![h::Surface::new()
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
                        .into_any_element()]), cx),
                ),
                (
                    "Constraints", "minValue/maxValue mute the days outside the range; isDateUnavailable strikes through the ones it rejects.",
                    specimen_body("cal-constraints", col(vec![
                        h::Calendar::new(self.demo_calendar("cal-constraints", cx))
                            .min_value(h::Date::new(today.year, today.month, 5))
                            .max_value(h::Date::new(today.year, today.month, 24))
                            .is_date_unavailable(|d: h::Date| d.day.is_multiple_of(7))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "First day of week",
                    specimen_body("cal-first-day", col(vec![h::Calendar::new(self.demo_calendar("cal-first-day", cx))
                        .first_day_of_week(h::Weekday::Mon)
                        .weeks_in_month(6)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled", "A disabled calendar keeps the current month readable but rejects presses, arrows, and leaves the tab order.",
                    specimen_body("cal-disabled", col(vec![h::Calendar::new(self.demo_calendar("cal-disabled", cx))
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Read Only", "A read-only calendar stays focusable and navigable but rejects mutations.",
                    specimen_body("cal-readonly", col(vec![h::Calendar::new(self.demo_calendar("cal-readonly", cx))
                        .is_read_only(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Multiple Months",
                    "Scroll horizontally to explore both months in narrow layouts.",
                    specimen_body("cal-months", stretch_col(vec![h::Calendar::new(self.demo_calendar("cal-months", cx))
                        .visible_duration(h::VisibleDuration::Months(2))
                        .into_any_element()]), cx),
                ),
                (
                    "Week View",
                    specimen_body("cal-week-view", col(vec![h::Calendar::new(self.demo_calendar("cal-week-view", cx))
                        .visible_duration(h::VisibleDuration::Weeks(2))
                        .page_behavior(h::PageBehavior::Single)
                        .into_any_element()]), cx),
                ),
                (
                    "Day View",
                    "Seven weekday columns show the requested dates, disabled leading dates and blank trailing cells.",
                    specimen_body("cal-day-view", col(vec![h::Calendar::new(self.demo_calendar("cal-day-view", cx))
                        .visible_duration(h::VisibleDuration::Days(5))
                        .into_any_element()]), cx),
                ),
                (
                    "Year Picker",
                    specimen_body("cal-year-picker", col(vec![h::Calendar::new(self.demo_calendar("cal-year-picker", cx))
                        .is_year_picker_open(self.cal_year_picker)
                        .on_year_picker_open_change(cx.listener(
                            |this, open: &bool, _, cx| {
                                this.cal_year_picker = *open;
                                cx.notify();
                            },
                        ))
                        .into_any_element()]), cx),
                ),
                (
                    "Heading Offset", "`Calendar.YearPickerTriggerHeading.offset` shifts the month heading -- also the year-picker trigger -- while the grid stays on the visible month. Both grids above show August; only the headings differ.",
                    col({
                        let august = h::Date::new(2026, 8, 10);
                        vec![
                            row(vec![
                                specimen_body("cal-heading-anchor", spec(
                                    "Same month",
                                    h::Calendar::new(self.demo_calendar("cal-heading-anchor", cx))
                                        .default_value(august)
                                        .into_any_element(), cx),
                                    cx,
                                ),
                                specimen_body("cal-heading-offset", spec(
                                    "Heading offset",
                                    h::Calendar::new(self.demo_calendar("cal-heading-offset", cx))
                                        .default_value(august)
                                        .offset(1)
                                        .into_any_element(),
                                    cx,
                                ), cx),
                            ]),
                        ]
                    }),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let iso = self.date_iso;
        component_doc_page!(
            "Date Field",
            crate::pages::Page::DateField.description(),
            crate::pages::Page::DateField.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("df-usage", field_col(vec![
                        h::DateField::new(self.date_input.clone())
                            // v3's Usage seeds the field with `defaultValue`.
                            .default_value(h::Date::new(2025, 12, 25))
                            .label("Start date")
                            .radius(px(4.))
                            .on_change(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.date_iso = *d;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match iso {
                                Some(d) => format!("Parsed: {}", d.format_iso()),
                                None => {
                                    "Type digits, or step a segment with the arrow keys".to_owned()
                                }
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` resize the box in one call. A 28px bare box with an 8px inset.",
                    specimen_body("df-box", field_col(vec![h::DateField::new(self.demo_text("df-custom-box", "", cx))
                        .label("Compact")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .font_family(crate::app::MONO_FONT)
                        .into_any_element()]), cx),
                ),
                (
                    "Granularity", "`granularity` sets the smallest unit the field shows. Below `day` it grows the time segments -- the same ones a `TimeField` has, so the arrows step them and digits type into them -- and the bound state holds an ISO date-and-time.",
                    {
                        let mut examples = vec![spec_row(
                            h::Granularity::ALL
                                .iter()
                                .copied()
                                .map(|granularity| {
                                    let key = match granularity {
                                        h::Granularity::Day => "df-gran-day",
                                        h::Granularity::Hour => "df-gran-hour",
                                        h::Granularity::Minute => "df-gran-minute",
                                        h::Granularity::Second => "df-gran-second",
                                    };
                                    specimen_body(key, spec(
                                        granularity.label(),
                                        h::DateField::new(self.demo_text(
                                            key,
                                            "2025-02-03T08:45:09",
                                            cx,
                                        ))
                                        .granularity(granularity),
                                        cx,
                                    ), cx)
                                })
                                .collect(),
                        )];
                        examples.push(specimen_body(
                            "df-gran-12h",
                            h::DateField::new(self.demo_text(
                                    "df-gran-12h",
                                    "2025-02-03T20:45",
                                    cx,
                                ))
                                .label("Twelve-hour clock")
                                .granularity(h::Granularity::Minute)
                                .hour_cycle(h::HourCycle::H12)
                                .into_any_element(),
                            cx,
                        ));
                        col(examples)
                    },
                ),
                (
                    "Forced Leading Zeros", "The system locale controls date and time segment order, separators, padding, and day-period names. The prop forces month, day, and hour segments to two digits.",
                    field_col(vec![
                        specimen_body("df-leading-locale", h::DateField::new(self.demo_text(
                            "df-leading-locale",
                            "2025-02-03T08:05:07",
                            cx,
                        ))
                        .label("System locale")
                        .granularity(h::Granularity::Second)
                        .hour_cycle(h::HourCycle::H12)
                        .into_any_element(), cx),
                        specimen_body("df-leading-forced", h::DateField::new(self.demo_text(
                            "df-leading-forced",
                            "2025-02-03T08:05:07",
                            cx,
                        ))
                        .label("Forced two-digit fields")
                        .granularity(h::Granularity::Second)
                        .hour_cycle(h::HourCycle::H12)
                        .should_force_leading_zeros(true)
                        .into_any_element(), cx),
                    ]),
                ),
                (
                    "With Icons",
                    specimen_body("df-icon", field_col(vec![h::DateField::new(self.demo_text("df-icon", "", cx))
                        .label("Date")
                        .prefix(icon(h::icons::MOON, cx))
                        .into_any_element()]), cx),
                ),
                (
                    "Variants",
                    col(vec![
                        specimen_body(
                            "df-primary",
                            h::DateField::new(self.demo_text("df-primary", "", cx))
                                .label("Primary")
                                .into_any_element(),
                            cx,
                        ),
                        specimen_body(
                            "df-secondary",
                            h::DateField::new(self.demo_text("df-secondary", "", cx))
                                .label("Secondary")
                                .variant(FieldVariant::Secondary)
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "In Surface",
                    specimen_body("df-surface", col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::DateField::new(self.demo_text("df-surface", "", cx))
                                .label("Date")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "With Description",
                    specimen_body("df-desc", field_col(vec![h::DateField::new(self.demo_text("df-desc", "", cx))
                        .label("Date")
                        .description("Month, day and year")
                        .into_any_element()]), cx),
                ),
                (
                    "Required Field",
                    specimen_body("df-req", field_col(vec![h::DateField::new(self.demo_text("df-req", "", cx))
                        .label("Date")
                        .is_required(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled State",
                    specimen_body("df-dis", field_col(vec![h::DateField::new(self.demo_text(
                        "df-dis",
                        "2025-12-25",
                        cx,
                    ))
                    .label("Date")
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Full Width",
                    specimen_body("df-full", col(vec![h::DateField::new(self.demo_text("df-full", "", cx))
                        .label("Date")
                        .full_width(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Validation",
                    specimen_body("df-invalid", field_col(vec![h::DateField::new(self.demo_text(
                        "df-invalid",
                        "",
                        cx,
                    ))
                    .label("Date")
                    .is_required(true)
                    .is_invalid(true)
                    .validation_errors(["Pick a date"])
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("df-ctl", col(vec![
                        h::DateField::new(self.demo_text("df-ctl", "", cx))
                            .label("Date")
                            .on_change(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.date_iso = *d;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match iso {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "With Validation",
                    specimen_body("df-validate", field_col(vec![h::DateField::new(self.demo_text(
                        "df-validate",
                        "",
                        cx,
                    ))
                    .label("Date")
                    .min_value(h::Date::new(2025, 1, 1))
                    .max_value(h::Date::new(2025, 12, 31))
                    .description("Must fall in 2025")
                    .into_any_element()]), cx),
                ),
                (
                    "Form Example",
                    specimen_body("df-form", col(vec![{
                        let field = self.demo_text("df-form", "", cx);
                        h::Form::new()
                            .field(h::FormField::text(field.clone()).name("start"))
                            .child(
                                h::DateField::new(field)
                                    .label("Start date")
                                    .is_required(true),
                            )
                            .child(h::Button::new("df-form-submit").label("Save"))
                            .into_any_element()
                    }]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.date_picker_open;
        let render_props_gallery = cx.entity().downgrade();
        component_doc_page!(
            "Date Picker",
            crate::pages::Page::DatePicker.description(),
            crate::pages::Page::DatePicker.import_line(),
            vec![
                (
                    "International Calendar",
                    "The field and the popover grid share the locale override. Segment order follows that locale; selected values stay Gregorian ISO dates.",
                    specimen_body("dp-indian", field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-indian", cx),
                    )
                    .label("Date")
                    .locale("en-US-u-ca-indian")
                    .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("dp-disabled", field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-disabled", cx),
                    )
                    .label("Date")
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("dp-controlled", col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-controlled", cx))
                            .label("Date")
                            .on_change(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match self.cal_picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]), cx),
                ),
                (
                    "Validation",
                    specimen_body("dp-invalid", field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-invalid", cx),
                    )
                    .label("Date")
                    // `minValue`/`maxValue` bound the calendar: everything
                    // outside the range is unselectable.
                    .min_value(h::Date::new(2025, 12, 1))
                    .max_value(h::Date::new(2026, 6, 30))
                    .is_invalid(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Format Options", "The trigger follows the operating system's regional date order, separators, and numeric padding. Its state and submitted value stay ISO-formatted.",
                    specimen_body("dp-format", col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-format", cx))
                            .label("Date")
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Form Example",
                    specimen_body("dp-form", col(vec![h::Form::new()
                        .child(
                            h::DatePicker::new(self.demo_calendar("dp-form", cx))
                                .label("Start date"),
                        )
                        .child(h::Button::new("dp-form-submit").label("Save"))
                        .into_any_element()]), cx),
                ),
                (
                    "Custom Indicator", "A custom indicator replaces the default calendar glyph; this example uses a check without changing the trigger behavior.",
                    specimen_body("dp-indicator", col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-indicator", cx))
                            .label("Date")
                            .trigger_indicator(icon(h::icons::CHECK, cx))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Render Function",
                    specimen_body("dp-render-props", field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-render-props", cx)
                    )
                    .is_open(is_open)
                    .is_required(true)
                    .content(move |state| {
                        let gallery = render_props_gallery.clone();
                        gpui::div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(8.))
                            .child(format!(
                                "{} · {} · {}",
                                if state.is_required {
                                    "required"
                                } else {
                                    "optional"
                                },
                                if state.is_invalid { "invalid" } else { "valid" },
                                if state.is_open { "open" } else { "closed" },
                            ))
                            .child(
                                h::Button::new("dp-render-props-toggle")
                                    .label(if state.is_open { "Close" } else { "Open" })
                                    .on_press(move |_, _, cx| {
                                        if let Some(gallery) = gallery.upgrade() {
                                            gallery.update(cx, |gallery, cx| {
                                                gallery.date_picker_open =
                                                    !gallery.date_picker_open;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .into_any_element()
                    })
                    .into_any_element()]), cx),
                ),
                (
                    "Usage",
                    specimen_body("dp-usage", field_col(vec![h::DatePicker::new(self.demo_calendar("dp-usage", cx))
                        // v3's Usage seeds the picker with `defaultValue`.
                        .default_value(h::Date::new(2025, 12, 25))
                        .label("Due date")
                        .is_open(is_open)
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.date_picker_open = *open;
                            cx.notify();
                        }))
                        .on_change(cx.listener(
                        |this, d: &Option<h::Date>, _, cx| {
                                this.cal_picked = *d;
                                this.date_picker_open = false;
                                cx.notify();
                            },
                        ))
                        .into_any_element()]), cx),
                ),
                (
                    "Placement",
                    "The calendar popover accepts the shared 22-value placement vocabulary. HeroUI defaults to a centered bottom popover; this specimen opts into top placement.",
                    specimen_body("dp-placement", field_col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-placement", cx))
                            .label("Top placement")
                            .placement(h::Placement::Top)
                            .default_open(true)
                            .into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_range_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.range_open;
        let render_props_gallery = cx.entity().downgrade();
        component_doc_page!(
            "Date Range Picker",
            crate::pages::Page::DateRangePicker.description(),
            crate::pages::Page::DateRangePicker.import_line(),
            vec![
                (
                    "International Calendar",
                    "Both fields and the popover grid share the locale override. Segment order follows that locale; selected values stay Gregorian ISO dates.",
                    specimen_body("drp-indian", field_col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-indian", cx),
                    )
                    .label("Stay")
                    .locale("en-US-u-ca-indian")
                    .trigger_hover_bg(cx.colors().accent.soft())
                    .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("drp-disabled", field_col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-disabled", cx),
                    )
                    .label("Stay")
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled", "The range lives in the state entity the caller owns.",
                    specimen_body("drp-controlled", col(vec![
                        {
                            // `value` seeds the caller's copy in, and
                            // `on_change` is how the caller gets one: a range
                            // picker reports the change with no arguments, so
                            // the range is read out of the state entity.
                            let state = self.demo_range("drp-controlled", cx);
                            let held = cx.entity().downgrade();
                            let (start, end) = {
                                let range = state.read(cx);
                                (range.start, range.end)
                            };
                            h::DateRangePicker::new(state.clone())
                                .label("Stay")
                                .value(start, end)
                                .on_change(move |_, cx| {
                                    let (start, end) = {
                                        let range = state.read(cx);
                                        (range.start, range.end)
                                    };
                                    if let Some(gallery) = held.upgrade() {
                                        gallery.update(cx, |gallery, cx| {
                                            gallery.set_demo_text_value(
                                                "drp-controlled",
                                                match (start, end) {
                                                    (Some(a), Some(b)) => format!(
                                                        "{} to {}",
                                                        a.format_iso(),
                                                        b.format_iso()
                                                    ),
                                                    _ => String::new(),
                                                },
                                            );
                                            cx.notify();
                                        });
                                    }
                                })
                                .into_any_element()
                        },
                    ]), cx),
                ),
                (
                    "Validation",
                    specimen_body("drp-invalid", field_col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-invalid", cx),
                    )
                    .label("Stay")
                    .is_invalid(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Format Options", "Both ends follow the operating system's regional date order, separators, and numeric padding. Their state and submitted values stay ISO-formatted.",
                    specimen_body("drp-format", col(vec![
                        h::DateRangePicker::new(self.demo_range("drp-format", cx))
                            .label("Stay")
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Form Example",
                    specimen_body("drp-form", col(vec![h::Form::new()
                        .child(
                            h::DateRangePicker::new(self.demo_range("drp-form", cx))
                                .label("Stay")
                                // v3 submits a range as two named fields.
                                .start_name("check_in")
                                .end_name("check_out"),
                        )
                        .child(h::Button::new("drp-form-submit").label("Book"))
                        .into_any_element()]), cx),
                ),
                (
                    "Custom Indicator", "Custom indicators replace the default calendar glyph and range separator without changing field or trigger behavior.",
                    specimen_body("drp-indicator", col(vec![
                        h::DateRangePicker::new(self.demo_range("drp-indicator", cx))
                            .label("Stay")
                            .trigger_indicator(icon(h::icons::CHECK, cx))
                            .range_separator(gpui::div().child("to"))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Render Function",
                    specimen_body("drp-render-props", col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-render-props", cx),
                    )
                    .is_open(is_open)
                    .is_required(true)
                    .content(move |state| {
                        let gallery = render_props_gallery.clone();
                        gpui::div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(8.))
                            .child(format!(
                                "{} · {} · {}",
                                if state.is_required {
                                    "required"
                                } else {
                                    "optional"
                                },
                                if state.is_invalid { "invalid" } else { "valid" },
                                if state.is_open { "open" } else { "closed" },
                            ))
                            .child(
                                h::Button::new("drp-render-props-toggle")
                                    .label(if state.is_open { "Close" } else { "Open" })
                                    .on_press(move |_, _, cx| {
                                        if let Some(gallery) = gallery.upgrade() {
                                            gallery.update(cx, |gallery, cx| {
                                                gallery.range_open = !gallery.range_open;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .into_any_element()
                    })
                    .into_any_element()]), cx),
                ),
                (
                    "Usage",
                    specimen_body("drp-usage", col(vec![gpui::div()
                        .w(px(320.))
                        .child(
                            h::DateRangePicker::new(self.demo_range("drp-usage", cx))
                                .label("Trip dates")
                                // v3's Usage seeds the range and bounds it.
                                .default_value((
                                    h::Date::new(2025, 12, 8),
                                    h::Date::new(2025, 12, 14),
                                ))
                                .min_value(h::Date::new(2025, 1, 1))
                                .is_open(is_open)
                                .on_open_change(cx.listener(
                                    |this, open: &bool, _, cx| {
                                        this.range_open = *open;
                                        cx.notify();
                                    },
                                ))
                                .on_change(|_, _cx| {}),
                        )
                        .into_any_element()]),
                        cx),
                ),
                (
                    "Placement",
                    "The range calendar popover accepts the shared 22-value placement vocabulary. HeroUI defaults to a centered bottom popover; this specimen opts into top placement.",
                    specimen_body("drp-placement", field_col(vec![
                        h::DateRangePicker::new(self.demo_range("drp-placement", cx))
                            .label("Top placement")
                            .placement(h::Placement::Top)
                            .default_open(true)
                            .into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_range_calendar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let today = h::Date::today();
        let focused = self.range_calendar_focus;
        component_doc_page!(
            "Range Calendar",
            crate::pages::Page::RangeCalendar.description(),
            crate::pages::Page::RangeCalendar.import_line(),
            vec![
                (
                    "International Calendars",
                    "An Indian-calendar range aligned to the end of a two-month view. The January 21-22 Gregorian selection stays in the second displayed month.",
                    specimen_body("rc-indian", col(vec![h::RangeCalendar::new(self.demo_range("rc-indian", cx))
                        .locale("en-US-u-ca-indian")
                        .day_hover_bg(cx.colors().accent.soft())
                        .nav_hover_bg(cx.colors().accent.soft())
                        .year_hover_bg(cx.colors().accent.soft())
                        .default_value((h::Date::new(2026, 1, 21), h::Date::new(2026, 1, 22)))
                        .visible_duration(h::VisibleDuration::Months(2))
                        .selection_alignment(h::SelectionAlignment::End)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    "Disabled day cells retain the same 14px/20px medium typography as selectable and selected dates.",
                    specimen_body("rc-disabled", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-disabled", cx),
                    )
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Cell Indicators",
                    specimen_body("rc-dots", col(vec![h::RangeCalendar::new(self.demo_range("rc-dots", cx))
                        // `RangeCalendar.CellIndicator` marks a day with a dot,
                        // the same part a `Calendar` draws.
                        .cell_indicator(|d| d.day % 7 == 3)
                        .into_any_element()]), cx),
                ),
                (
                    "Year Picker",
                    "Years scroll within the day-grid area. Opening and keyboard navigation reveal the focused year without expanding the calendar.",
                    specimen_body("rc-year", col(vec![h::RangeCalendar::new(self.demo_range("rc-year", cx))
                        .default_year_picker_open(true)
                        // `firstDayOfWeek` reorders the seven columns.
                        .first_day_of_week(h::Weekday::Mon)
                        .into_any_element()]), cx),
                ),
                (
                    "Heading Offset", "`RangeCalendar.YearPickerTriggerHeading.offset` shifts the month heading -- also the year-picker trigger -- while the grid stays on the visible month. Both grids above show August; only the headings differ.",
                    col({
                        let august = (h::Date::new(2026, 8, 10), h::Date::new(2026, 8, 16));
                        vec![
                            row(vec![
                                specimen_body("rc-heading-anchor", spec(
                                    "Grid: August 2026; heading: August 2026 (offset 0)",
                                    h::RangeCalendar::new(self.demo_range("rc-heading-anchor", cx))
                                        .default_value(august)
                                        .into_any_element(),
                                    cx,
                                ), cx),
                                specimen_body("rc-heading-offset", spec(
                                    "Grid: August 2026; heading: September 2026 (offset +1)",
                                    h::RangeCalendar::new(self.demo_range("rc-heading-offset", cx))
                                        .default_value(august)
                                        .offset(1)
                                        .into_any_element(),
                                    cx,
                                ), cx),
                            ]),
                        ]
                    }),
                ),
                (
                    "Default Value",
                    specimen_body("rc-default", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-default", cx),
                    )
                    .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled", "The range lives in the state entity the caller owns.",
                    specimen_body("rc-controlled", col(vec![
                        h::RangeCalendar::new(self.demo_range("rc-controlled", cx))
                            .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                            .on_focus_change(cx.listener(|this, d: &h::Date, _, cx| {
                                this.set_demo_text_value("rc-focus", d.format_iso());
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Min and Max Dates",
                    specimen_body("rc-minmax", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-minmax", cx),
                    )
                    .min_value(h::Date::new(today.year, today.month, 5))
                    .max_value(h::Date::new(today.year, today.month, 24))
                    .into_any_element()]), cx),
                ),
                (
                    "Unavailable Dates",
                    specimen_body("rc-unavailable", col({
                        let blocked_ranges = [
                            (h::add_days(&today, 2), h::add_days(&today, 5)),
                            (h::add_days(&today, 12), h::add_days(&today, 13)),
                        ];
                        vec![
                            h::RangeCalendar::new(self.demo_range("rc-unavailable", cx))
                                .default_value((h::add_days(&today, 6), h::add_days(&today, 9)))
                                .first_day_of_week(h::Weekday::Mon)
                                .is_date_unavailable(move |date, _| {
                                    let date = h::days_from_civil(&date);
                                    blocked_ranges.iter().any(|(start, end)| {
                                        date >= h::days_from_civil(start)
                                            && date <= h::days_from_civil(end)
                                    })
                                })
                                .into_any_element(),
                            gpui::div()
                                .child("Some days are unavailable")
                                .into_any_element(),
                        ]
                    }), cx),
                ),
                (
                    "Anchor-Based Unavailable Dates", "After the first date is selected, earlier dates become unavailable because the predicate receives that active anchor.",
                    specimen_body("rc-anchor", col(vec![
                        h::RangeCalendar::new(self.demo_range("rc-anchor", cx))
                            .is_date_unavailable(|date, anchor| {
                                anchor.is_some_and(|anchor| {
                                    (date.year, date.month, date.day)
                                        < (anchor.year, anchor.month, anchor.day)
                                })
                            })
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Allows Non-Contiguous Ranges",
                    specimen_body("rc-noncontig", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-noncontig", cx),
                    )
                    .is_date_unavailable(|date, _| date.day == 15)
                    .allows_non_contiguous_ranges(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Weeks in Month",
                    specimen_body("rc-weeks", col(vec![h::RangeCalendar::new(self.demo_range("rc-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]), cx),
                ),
                (
                    "Week View",
                    specimen_body("rc-week-view", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-week-view", cx),
                    )
                    .visible_duration(h::VisibleDuration::Weeks(2))
                    .into_any_element()]), cx),
                ),
                (
                    "Day View",
                    "Seven weekday columns show the requested dates, disabled leading dates and blank trailing cells.",
                    specimen_body("rc-day-view", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-day-view", cx),
                    )
                    .visible_duration(h::VisibleDuration::Days(5))
                    .into_any_element()]), cx),
                ),
                (
                    "Multiple Months",
                    "Scroll horizontally to explore both months in narrow layouts.",
                    specimen_body("rc-months", stretch_col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-months", cx),
                    )
                    .visible_duration(h::VisibleDuration::Months(2))
                    .into_any_element()]), cx),
                ),
                (
                    "Read Only",
                    "Browse dates without changing the supplied range.",
                    specimen_body("rc-readonly", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-readonly", cx),
                    )
                    .value(Some(h::Date::new(2025, 12, 8)), Some(h::Date::new(2025, 12, 14)))
                    .is_read_only(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Invalid",
                    specimen_body("rc-invalid", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-invalid", cx),
                    )
                    .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                    .is_invalid(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Focused Value",
                    specimen_body("rc-focused", col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-focused", cx),
                    )
                    .focused_value(focused)
                    .on_focus_change(cx.listener(|this, date: &h::Date, _, cx| {
                        this.range_calendar_focus = *date;
                        cx.notify();
                    },))
                    .into_any_element()]), cx),
                ),
                (
                    "Real-World Example",
                    specimen_body("rc-real", col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Choose your stay"))
                        .child(
                            h::RangeCalendar::new(self.demo_range("rc-real", cx))
                                .min_value(today)
                                .is_date_unavailable(|date, _| date.day == 20),
                        )
                        .child(h::Description::new("The 20th is fully booked."))
                        .into_any_element()]), cx),
                ),
                (
                    "Usage",
                    specimen_body("rc-usage", col(vec![h::RangeCalendar::new(self.demo_range("rc-usage", cx))
                        .nav_hover_bg(cx.colors().accent.soft())
                        .on_change(|_start, _end, _, _cx| {})
                        .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_time_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Time Field",
            crate::pages::Page::TimeField.description(),
            crate::pages::Page::TimeField.import_line(),
            vec![
                (
                    "24-hour",
                    specimen_body("tmf-24-hour", field_col(vec![h::TimeField::new(self.demo_time("tmf-24-hour", cx))
                        .label("Start time")
                        .hour_cycle(h::HourCycle::H24)
                        .description("Click a segment, then use the steppers.")
                        .on_change(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "12-hour with seconds",
                    specimen_body("tmf-12-hour-seconds", field_col(vec![h::TimeField::new(self.demo_time("tmf-12-hour-seconds", cx))
                        .label("Reminder")
                        .hour_cycle(h::HourCycle::H12)
                        .show_seconds(true)
                        .on_change(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Forced Leading Zeros", "The system locale controls numeric padding; this prop only forces the hour to two digits.",
                    field_col(vec![
                        specimen_body("tmf-leading-locale", h::TimeField::new(self.demo_time("tmf-leading-locale", cx))
                            .label("Locale default")
                            .hour_cycle(h::HourCycle::H12)
                            .show_seconds(true)
                            .into_any_element(), cx),
                        specimen_body("tmf-leading-forced", h::TimeField::new(self.demo_time("tmf-leading-forced", cx))
                            .label("Forced two-digit hour")
                            .hour_cycle(h::HourCycle::H12)
                            .show_seconds(true)
                            .should_force_leading_zeros(true)
                            .into_any_element(), cx),
                    ]),
                ),
                (
                    "Usage", "Uses your system regional segment order, separators, padding, day-period names, and 12- or 24-hour cycle.",
                    specimen_body("tmf-usage", field_col(vec![
                        h::TimeField::new(self.demo_time("tmf-usage", cx))
                            .label("Time")
                            .radius(px(4.))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Box Customisation",
                    "The same box seam as the other fields: `height`, `padding_x` and `is_bare`.",
                    specimen_body("tmf-custom-box", field_col(vec![h::TimeField::new(self.demo_time("tmf-custom-box", cx))
                        .label("Compact")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .stepper_hover_bg(cx.colors().accent.soft())
                        .font_family(crate::app::MONO_FONT)
                        .into_any_element()]), cx),
                ),
                (
                    "With Icons",
                    specimen_body("tmf-icon", field_col(vec![h::TimeField::new(self.demo_time("tmf-icon", cx))
                        .label("Time")
                        .prefix(icon(h::icons::SUN, cx))
                        .into_any_element()]), cx),
                ),
                (
                    "On Surface",
                    specimen_body("tmf-surface", col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::TimeField::new(self.demo_time("tmf-surface", cx))
                                .label("Time")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "With Description",
                    specimen_body("tmf-desc", field_col(vec![h::TimeField::new(self.demo_time("tmf-desc", cx))
                        .label("Time")
                        .description("Hour and minute")
                        .into_any_element()]), cx),
                ),
                (
                    "Required Field",
                    specimen_body("tmf-req", field_col(vec![h::TimeField::new(self.demo_time("tmf-req", cx))
                        .label("Time")
                        .is_required(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled State",
                    specimen_body("tmf-dis", field_col(vec![h::TimeField::new(self.demo_time("tmf-dis", cx))
                        .label("Time")
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Full Width",
                    specimen_body("tmf-full", gpui::div()
                        .w(px(400.))
                        .child(
                            h::TimeField::new(self.demo_time("tmf-full", cx))
                                .label("Time")
                                .full_width(true),
                        )
                        .into_any_element(), cx),
                ),
                (
                    "Validation",
                    specimen_body("tmf-invalid", field_col(vec![h::TimeField::new(self.demo_time("tmf-invalid", cx))
                        .label("Time")
                        // `minValue`/`maxValue` clamp what the segments accept.
                        .min_value(h::Time::new(9, 0))
                        .max_value(h::Time::new(17, 30))
                        .is_required(true)
                        .is_invalid(true)
                        .error_message("Pick a time")
                        .into_any_element()]), cx),
                ),
                (
                    "Controlled", "The field owns the value; `on_change` reports each edit.",
                    specimen_body("tmf-ctl", col(vec![
                        h::TimeField::new(self.demo_time("tmf-ctl", cx))
                            .label("Time")
                            .on_change(
                                cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "With Validation",
                    specimen_body("tmf-validate", field_col(vec![h::TimeField::new(self.demo_time("tmf-validate", cx))
                        .label("Meeting time")
                        .description("Office hours are 09:00 to 17:00")
                        .validate(|value| {
                            value
                                .filter(|t| t.hour < 9 || t.hour >= 17)
                                .map(|_| "Pick a time inside office hours".into())
                        })
                        .into_any_element()]), cx),
                ),
                (
                    "Form Example",
                    specimen_body("tmf-form", col(vec![{
                        let state = self.demo_time("tmf-form", cx);
                        let field = h::TimeField::new(state)
                            .label("Start time")
                            .name("start_time")
                            .is_required(true);
                        h::Form::new()
                            .child(field)
                            .child(h::Button::new("tmf-form-submit").label("Save"))
                            .into_any_element()
                    }]), cx),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
