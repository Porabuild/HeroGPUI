//! Deeper v3 coverage for edge dates and constraints on the calendar family.
//!
//! `calendars_and_more.rs` already drives the calendar's arrows and week
//! keys, PageUp/PageDown month changes, Home/End, min/max blocking and the
//! RangeCalendar's anchor/extend selection. These two tests push the edges the
//! grid geometry hides: the unavailable-date predicate (v3
//! `isDateUnavailable`: "Marks dates as unavailable") on both input paths,
//! and month paging across the December→January year boundary that lands on a
//! leap February (2028), where the date math must resolve Jan 31 forward to
//! Feb 29 rather than invent a Feb 31.
//!
//! Geometry is the same derivation the rest of the suite uses: a bare
//! Calendar at the window origin, `CALENDAR_WIDTH` (252) minus six 2px gaps
//! over seven cells for the column centres, the first cell row at y = 74 with
//! 38px per row after it, and the month's leading blanks
//! (`DateConstraints::lead_cells`, Monday-start default) for a day's
//! row/column.

mod harness;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext};
use harness::{click, events, open_host, press};
use herogpui_components::{
    calendar::{Date, CALENDAR_WIDTH},
    Calendar, CalendarState, DateConstraints, DateRangeState, RangeCalendar,
};

/// Column *c*'s centre in a bare Calendar: seven cells across
/// `CALENDAR_WIDTH` minus six 2px gaps.
fn cal_col_x(col: usize) -> f32 {
    let cell_w = (f32::from(CALENDAR_WIDTH) - 12.) / 7.;
    col as f32 * (cell_w + 2.) + cell_w / 2.
}

/// Row *r*'s centre in a bare Calendar: the first row at y = 74, then a
/// 36px cell plus a 2px gap per row.
fn cal_row_y(row: usize) -> f32 {
    74. + row as f32 * 38.
}

/// The centre of the cell holding `day` of `(year, month)` in a bare
/// Calendar, derived from the month's leading blanks (the same
/// `DateConstraints` the test's calendars use).
fn cal_day(year: i32, month: u32, day: u32) -> (f32, f32) {
    let lead = DateConstraints::new().lead_cells(year, month);
    let idx = day as usize + lead - 1;
    (cal_col_x(idx % 7), cal_row_y(idx / 7))
}

fn range_day(year: i32, month: u32, day: u32) -> (f32, f32) {
    let lead = DateConstraints::new().lead_cells(year, month);
    let idx = day as usize + lead - 1;
    (19. + 38. * (idx % 7) as f32, 75. + 40. * (idx / 7) as f32)
}

/// `isDateUnavailable` blocks the date on both input paths without blocking
/// the ring from resting on it. The arrows still move the focus onto an
/// unavailable day (`onFocusChange` fires, as React Aria keeps the focused
/// date navigable), but Enter is stopped by the same `constraints.allows`
/// gate (`calendar.rs`'s key handler) and the cell itself carries no click
/// handler at all. The neighbouring available day answers both.
#[gpui::test]
fn calendar_is_date_unavailable_blocks_selection_not_focus(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        let changes = changes.clone();
        // `defaultValue` seeds the selection and the view month, so the grid
        // shows August 2026 whatever day the suite runs on. Day 16 is the
        // only unavailable day in the month, next door to the selected 15th.
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .is_date_unavailable(|date| date.day == 16)
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    // Enter on the unavailable day: Right moves the focus there and reports
    // it, but the pick is stopped.
    press(cx, "tab");
    press(cx, "right");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16"],
        "the arrows must still move the focus onto an unavailable day"
    );
    press(cx, "enter");
    assert!(
        changed.borrow().is_empty(),
        "Enter on an unavailable day must not select it"
    );

    // One more Right reaches the 17th, which is available: Enter selects it.
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-17"],
        "Enter on the neighbouring available day must select it"
    );

    // The pointer path: an unavailable cell gets no handler at all, so a click
    // records nothing -- not even the focus event a keyboard move fires.
    let (unavail_x, unavail_y) = cal_day(2026, 8, 16);
    click(cx, unavail_x, unavail_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-17"],
        "clicking an unavailable day must not select it"
    );
    assert_eq!(
        focused.borrow().len(),
        2,
        "an unavailable cell must not answer the pointer either"
    );

    let (avail_x, avail_y) = cal_day(2026, 8, 17);
    click(cx, avail_x, avail_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-17", "2026-08-17"],
        "clicking the neighbouring available day must still select it"
    );
}

/// Right from Dec 31 2027 crosses the year boundary into January 2028 and the
/// visible month follows the cursor. End then jumps to Jan 31, and PageDown
/// from there resolves to Feb 29 -- the day `days_in_month` grants a leap
/// February, the exact edge a non-leap year would collapse to the 28th. The
/// leap day is a real day: Enter selects it.
#[gpui::test]
fn calendar_pages_dec_to_jan_and_clamps_into_leap_february(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        let changes = changes.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2027, 12, 31))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    // Right from Dec 31 2027 crosses into January 2028: the date upshifts the
    // year and the grid follows the cursor.
    press(cx, "tab");
    press(cx, "right");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2028-01-01"],
        "Right across the year boundary must land on Jan 1"
    );
    {
        let (year, month) =
            cx.update(|_, cx| (state.read(cx).view_year, state.read(cx).view_month));
        assert_eq!((year, month), (2028, 1), "the view must follow the cursor");
    }

    // End jumps to the last day of the visible month, Jan 31.
    press(cx, "end");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2028-01-01", "2028-01-31"],
        "End must jump to the last day of the month"
    );

    // PageDown from Jan 31 into the leap February: the 29th, not a clamped
    // 28th and not a Feb 31.
    press(cx, "pagedown");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2028-01-01", "2028-01-31", "2028-02-29"],
        "PageDown must clamp Jan 31 to Feb 29 in a leap year"
    );
    {
        let (year, month) =
            cx.update(|_, cx| (state.read(cx).view_year, state.read(cx).view_month));
        assert_eq!((year, month), (2028, 2), "the view must follow the cursor");
    }

    // The leap day is a real day: Enter selects it.
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2028-02-29"],
        "Enter must select the leap day"
    );
}

/// Without `allowsNonContiguousRanges`, React Aria constrains the selectable
/// end to the last available day before the first unavailable date after the
/// anchor. August 7 is unavailable here, so requesting August 10 must finish
/// at the neighbouring August 6. This proves the default restriction is about
/// the whole range, not only whether each endpoint is individually available.
#[gpui::test]
fn range_calendar_clamps_around_an_unavailable_date_by_default(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let selected_cells = Rc::new(RefCell::new(HashMap::new()));
    let selected_probe = selected_cells.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    state.update(cx, |state, _| {
        state.view_year = 2026;
        state.view_month = 8;
        state.view_day = 1;
        state.user_navigated = true;
    });
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let selected_cells = selected_cells.clone();
        RangeCalendar::new(state_for_view.clone())
            .is_date_unavailable(|date| date == Date::new(2026, 8, 7))
            .on_change(move |start, end, _, _| {
                let start = start.map(|date| date.format_iso()).unwrap_or_default();
                let end = end.map(|date| date.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(format!("{start}->{end}"));
            })
            .cell(move |state| {
                if (6..=8).contains(&state.date.day) {
                    selected_cells
                        .borrow_mut()
                        .insert(state.date.day, state.is_selected);
                }
                gpui::div().size(px(20.)).into_any_element()
            })
            .into_any_element()
    });

    let (start_x, start_y) = range_day(2026, 8, 5);
    click(cx, start_x, start_y);
    let (end_x, end_y) = range_day(2026, 8, 10);
    cx.simulate_mouse_move(
        point(px(end_x), px(end_y)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    cx.update(|window, _| window.refresh());
    assert_eq!(
        selected_probe.borrow().get(&6),
        Some(&true),
        "the preview must reach the last available day before the gap"
    );
    assert_eq!(
        selected_probe.borrow().get(&7),
        Some(&false),
        "the unavailable day must not be part of the preview"
    );
    assert_eq!(
        selected_probe.borrow().get(&8),
        Some(&false),
        "the default preview must not continue beyond the unavailable day"
    );
    click(cx, end_x, end_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-05->", "2026-08-05->2026-08-06"],
        "a forward range must clamp before the first unavailable day"
    );

    // A completed range starts over on the next click. Extending the new
    // anchor backwards across the same unavailable day clamps symmetrically.
    click(cx, end_x, end_y);
    click(cx, start_x, start_y);

    assert_eq!(
        changed.borrow().as_slice(),
        [
            "2026-08-05->",
            "2026-08-05->2026-08-06",
            "2026-08-10->",
            "2026-08-08->2026-08-10",
        ],
        "a backward range must clamp after the first unavailable day"
    );
}

/// Enabling `allowsNonContiguousRanges` removes only the interior gap
/// constraint. Available endpoints on either side of August 7 may complete the
/// range, while the unavailable day itself remains unavailable as an endpoint.
#[gpui::test]
fn range_calendar_allows_a_gap_but_not_an_unavailable_endpoint(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let unavailable_state = Rc::new(RefCell::new(None));
    let state_probe = unavailable_state.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    state.update(cx, |state, _| {
        state.view_year = 2026;
        state.view_month = 8;
        state.view_day = 1;
        state.user_navigated = true;
    });
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let unavailable_state = unavailable_state.clone();
        RangeCalendar::new(state_for_view.clone())
            .is_date_unavailable(|date| date == Date::new(2026, 8, 7))
            .allows_non_contiguous_ranges(true)
            .on_change(move |start, end, _, _| {
                let start = start.map(|date| date.format_iso()).unwrap_or_default();
                let end = end.map(|date| date.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(format!("{start}->{end}"));
            })
            .cell(move |state| {
                if state.date == Date::new(2026, 8, 7) {
                    *unavailable_state.borrow_mut() =
                        Some((state.is_selected, state.is_unavailable));
                }
                gpui::div().size(px(20.)).into_any_element()
            })
            .into_any_element()
    });

    let (start_x, start_y) = range_day(2026, 8, 5);
    click(cx, start_x, start_y);
    let (end_x, end_y) = range_day(2026, 8, 10);
    click(cx, end_x, end_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-05->", "2026-08-05->2026-08-10"],
        "the enabled mode must allow available endpoints across a gap"
    );
    cx.update(|window, _| window.refresh());
    assert_eq!(
        *state_probe.borrow(),
        Some((false, true)),
        "the unavailable interior date must remain outside the selected cells"
    );

    let (unavailable_x, unavailable_y) = range_day(2026, 8, 7);
    click(cx, unavailable_x, unavailable_y);
    assert_eq!(
        changed.borrow().len(),
        2,
        "an unavailable day must remain unavailable as an endpoint"
    );
}

/// Home can jump straight over both a minimum and an unavailable date. Range
/// resolution must apply both constraints and choose the one nearest the
/// anchor: minimum August 9 beats unavailable August 7 going backward.
#[gpui::test]
fn range_calendar_uses_the_tighter_bound_when_keys_jump(cx: &mut TestAppContext) {
    let backward = events();
    let reported = backward.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    state.update(cx, |state, _| {
        state.view_year = 2026;
        state.view_month = 8;
        state.view_day = 1;
        state.user_navigated = true;
    });
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let backward = backward.clone();
        RangeCalendar::new(state_for_view.clone())
            .min_value(Date::new(2026, 8, 9))
            .is_date_unavailable(|date| date == Date::new(2026, 8, 7))
            .on_change(move |start, end, _, _| {
                let start = start.map(|date| date.format_iso()).unwrap_or_default();
                let end = end.map(|date| date.format_iso()).unwrap_or_default();
                backward.borrow_mut().push(format!("{start}->{end}"));
            })
            .into_any_element()
    });

    press(cx, "tab");
    let (anchor_x, anchor_y) = range_day(2026, 8, 10);
    click(cx, anchor_x, anchor_y);
    cx.update(|window, _| window.refresh());
    press(cx, "home");
    press(cx, "enter");
    assert_eq!(
        reported.borrow().as_slice(),
        ["2026-08-10->", "2026-08-09->2026-08-10"],
        "the minimum must clamp before the farther unavailable boundary"
    );
}

/// The forward mirror of the minimum test: End jumps past maximum August 7
/// and unavailable August 9, so the nearer maximum must finish the range.
#[gpui::test]
fn range_calendar_maximum_beats_a_farther_unavailable_date(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    state.update(cx, |state, _| {
        state.view_year = 2026;
        state.view_month = 8;
        state.view_day = 1;
        state.user_navigated = true;
    });
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        RangeCalendar::new(state_for_view.clone())
            .max_value(Date::new(2026, 8, 7))
            .is_date_unavailable(|date| date == Date::new(2026, 8, 9))
            .on_change(move |start, end, _, _| {
                let start = start.map(|date| date.format_iso()).unwrap_or_default();
                let end = end.map(|date| date.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(format!("{start}->{end}"));
            })
            .into_any_element()
    });

    press(cx, "tab");
    let (anchor_x, anchor_y) = range_day(2026, 8, 5);
    click(cx, anchor_x, anchor_y);
    cx.update(|window, _| window.refresh());
    press(cx, "end");
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-05->", "2026-08-05->2026-08-07"],
        "the maximum must clamp before the farther unavailable boundary"
    );
}

/// In multiple mode v3's `onChange` value is the whole date array. The legacy
/// single-value callback can only name the activated date; `on_change_all`
/// must report the complete toggled set, including a plural `defaultValue`
/// seed, and it must remove a date when that cell is picked again.
#[gpui::test]
fn calendar_multiple_selection_reports_the_full_date_set(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let single = changes.clone();
        let all = changes.clone();
        Calendar::new(state_for_view.clone())
            .selection_mode(herogpui_components::SelectionMode::Multiple)
            .default_values([Date::new(2026, 8, 3), Date::new(2026, 8, 5)])
            .on_change(move |date, _, _| {
                single.borrow_mut().push(format!(
                    "one:{}",
                    date.map(|date| date.format_iso()).unwrap_or_default()
                ));
            })
            .on_change_all(move |dates, _, _| {
                all.borrow_mut().push(format!(
                    "all:{}",
                    dates
                        .iter()
                        .map(Date::format_iso)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            })
            .into_any_element()
    });

    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected_dates().to_vec()),
        [Date::new(2026, 8, 3), Date::new(2026, 8, 5)],
        "the plural defaultValue must seed every selected date"
    );

    let (day5_x, day5_y) = cal_day(2026, 8, 5);
    click(cx, day5_x, day5_y);
    let (day7_x, day7_y) = cal_day(2026, 8, 7);
    click(cx, day7_x, day7_y);
    cx.update(|window, _| window.refresh());
    press(cx, "tab");
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        [
            "one:2026-08-05",
            "all:2026-08-03",
            "one:2026-08-07",
            "all:2026-08-03,2026-08-07",
            "one:2026-08-08",
            "all:2026-08-03,2026-08-07,2026-08-08",
        ],
        "the full callback must report the set after pointer and keyboard toggles"
    );
}
