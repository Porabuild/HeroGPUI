//! Behaviour tests for the date pickers' close-on-select rule.
//!
//! v3 (via the React Aria pickers it composes) treats the two pickers
//! differently: a single `DatePicker` closes as soon as a day is chosen,
//! while a `DateRangePicker` stays open after the first pick so the end can
//! be chosen, and closes only once the range is complete. The defect was that
//! nothing closed the pickers at all — the calendar cell reported the pick
//! and the open flag never moved. These tests assert the close behaviourally,
//! never by appearance.
//!
//! Geometry is derived from the components' own constants, reusing the
//! DatePicker derivation from `pickers.rs` verbatim:
//!
//! - Each picker composes editable date segments and a separate 24px trigger.
//!   The single trigger centres at (124, 18), and the range trigger at
//!   (300, 18).
//! - The DatePicker panel hangs from `placed_panel(BottomStart, 6px)`: top =
//!   36 + 6 = 42, then `picker_panel` padding p-3 (12) brings the calendar to
//!   y = 54. A 24px header, gap 8, one ~16px weekday line and gap 8 later the
//!   first cell row spans y 110..146, centre 128. Only the weekday line is a
//!   text metric, and 128 stays inside the row for any line height the font
//!   can take.
//! - Calendar columns: `CALENDAR_WIDTH` (252) minus six 2px gaps over seven
//!   cells; that, plus the panel's 12px padding, fixes the last column's
//!   centre at `12 + 6*(cell_w + 2) + cell_w/2`.
//! - The DateRangePicker's `RangeCalendar` cells are 38px with no column
//!   gaps, so from the same panel origin the first row's cells centre at
//!   y = 129, the second at 169, and column *c* at `12 + 19 + 38c`.
//! - A bare `Calendar` starts at the window origin, so its first row centres
//!   at y = 74 and the last column at `6*(cell_w + 2) + cell_w/2`.
//!
//! No exiting overlay is involved: both pickers gate the panel on `is_open`
//! (no `util::overlay_phase`), so a closed popover leaves the tree on the
//! next frame and the second-click probe cannot hit an exiting panel.

mod harness;

use gpui::{prelude::*, TestAppContext};
use herogpui_components::{
    calendar::{CalendarState, Date, CALENDAR_WIDTH},
    Calendar, DateConstraints, DatePicker, DateRangePicker, DateRangeState,
};

use harness::{click, events, open_host};

/// The single picker's day coordinates, exactly as `pickers.rs` derives them.
fn day_coords() -> (f32, f32) {
    let cell_w = (f32::from(CALENDAR_WIDTH) - 12.) / 7.;
    let day_x = 12. + 6. * (cell_w + 2.) + cell_w / 2.;
    (day_x, 128.)
}

#[gpui::test]
fn date_picker_closes_when_a_day_is_picked(cx: &mut TestAppContext) {
    let picks = events();
    let recorded = picks.clone();
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::new(cx));

    // Column c of the first week holds day `c - lead + 1` once the lead
    // blanks are past, so the last column always holds day `7 - lead` — a
    // real day of this month whatever the month is.
    let today = Date::today();
    let lead = DateConstraints::new().lead_cells(today.year, today.month);
    let expected = Date::new(today.year, today.month, (7 - lead) as u32);
    let (day_x, day_y) = day_coords();

    let state_for_view = state;
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                picks.borrow_mut().push(iso);
            })
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 124., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the trigger must open the popover"
    );

    click(cx, day_x, day_y);
    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "clicking the last cell of the first week must pick day {}",
        7 - lead
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "a single date picker must close as soon as a day is chosen"
    );

    // Closed proof by behaviour, the same probe `pickers.rs` uses: the same
    // spot is bare page below the trigger now, so the press must reach
    // nothing. Were the popover still open, a second "day" would record here.
    click(cx, day_x, day_y);
    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "the popover must be gone after choosing a day"
    );
}

#[gpui::test]
fn date_picker_change_records_the_pick_once(cx: &mut TestAppContext) {
    // No `on_open_change` is wired: closing must not depend on a caller
    // handler, and the change callback must fire exactly once for the pick.
    let picks = events();
    let recorded = picks.clone();
    let state = cx.new(|cx| CalendarState::new(cx));

    let today = Date::today();
    let lead = DateConstraints::new().lead_cells(today.year, today.month);
    let expected = Date::new(today.year, today.month, (7 - lead) as u32);
    let (day_x, day_y) = day_coords();

    let state_for_view = state;
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        DatePicker::new(state_for_view.clone())
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                picks.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    click(cx, 124., 18.);
    click(cx, day_x, day_y);

    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "the change callback must record the chosen day exactly once"
    );

    // The pick closed the popover even though no open handler exists, so the
    // same coordinates record nothing a second time.
    click(cx, day_x, day_y);
    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "without an open handler the pick must still close the popover"
    );
}

#[gpui::test]
fn date_range_picker_stays_open_until_the_end_is_chosen(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));

    let today = Date::today();
    let lead = DateConstraints::new().lead_cells(today.year, today.month);
    let expected = Date::new(today.year, today.month, 1);
    // Day 8 is a week on: it lands one row down in the same column and is a
    // real, later date whatever the month is.
    let ended = Date::new(today.year, today.month, 8);
    // Column centres inside the range picker's 38px cell band: the first real
    // day of the month sits at row 0, column `lead`; a week later sits in row
    // 1, whose rows are 38px + a 2px row gap below row 0's 129px centre.
    let day_x = 31. + 38. * lead as f32;
    let row_one_y = 129.;
    let row_two_y = 169.;

    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        DateRangePicker::new(state.clone())
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .on_change(move |_, cx| {
                let st = state.read(cx);
                let s = st.start.map(|d| d.format_iso()).unwrap_or_default();
                let e = st.end.map(|d| d.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(format!("{s}->{e}"));
            })
            .into_any_element()
    });

    click(cx, 300., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Choosing the start leaves the panel open to choose the end.
    click(cx, day_x, row_one_y);
    assert_eq!(
        changed.borrow().as_slice(),
        [format!("{}->", expected.format_iso())],
        "the first pick must report the open-ended range"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the first pick must keep the panel open for the end"
    );
    {
        let (start, end) = cx.update(|_, cx| {
            let st = state.read(cx);
            (st.start, st.end)
        });
        assert_eq!(start, Some(expected), "the start must be remembered");
        assert_eq!(end, None, "the end must still be open");
    }

    // Choosing the end completes the range and closes the panel.
    click(cx, day_x, row_two_y);
    assert_eq!(
        changed.borrow().as_slice(),
        [
            format!("{}->", expected.format_iso()),
            format!("{}->{}", expected.format_iso(), ended.format_iso())
        ],
        "the second pick must report the complete range, once"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the panel must close once both ends are chosen"
    );
    {
        let (start, end) = cx.update(|_, cx| {
            let st = state.read(cx);
            (st.start, st.end)
        });
        assert_eq!(start, Some(expected), "the start must survive");
        assert_eq!(end, Some(ended), "the end must be remembered");
    }

    // Closed proof: a press where the start row was records nothing.
    click(cx, day_x, row_one_y);
    assert_eq!(
        changed.borrow().len(),
        2,
        "the popover must be gone after the range is complete"
    );
}

#[gpui::test]
fn bare_calendar_still_records_a_chosen_date(cx: &mut TestAppContext) {
    // A Calendar on its own has no popover and must be untouched by the
    // pickers' closing: clicking a day records it, open flag or not.
    let picks = events();
    let recorded = picks.clone();
    let state = cx.new(|cx| CalendarState::new(cx));

    let today = Date::today();
    let lead = DateConstraints::new().lead_cells(today.year, today.month);
    let expected = Date::new(today.year, today.month, (7 - lead) as u32);
    // The bare calendar starts at the window origin: no panel padding, so the
    // last column loses the pickers' leading 12px, and the first row sits at
    // y = 74 (24px header + gap 8 + ~16px weekday line + gap 8 + half a
    // 36px cell).
    let cell_w = (f32::from(CALENDAR_WIDTH) - 12.) / 7.;
    let day_x = 6. * (cell_w + 2.) + cell_w / 2.;
    let day_y = 74.;

    let state_for_view = state;
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        Calendar::new(state_for_view.clone())
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                picks.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    click(cx, day_x, day_y);
    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "a bare calendar must still record the day it is picked on"
    );
}
