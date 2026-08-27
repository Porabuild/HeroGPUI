//! Behaviour tests for the calendar family and the remaining un-driven
//! controls: Calendar, RangeCalendar, ColorPicker, Select and
//! Disclosure / DisclosureGroup / Toolbar.
//!
//! The `.shots/*.py` audits measure props and pixels; these tests drive the
//! controls with simulated input and assert on recorded callbacks, state
//! entities the test owns, and probe clicks that must record nothing.
//!
//! Geometry is derived from the components' own constants, reusing the
//! derivations in `pickers.rs` and `date_picker_close.rs`:
//!
//! - Bare Calendar at the window origin: `CALENDAR_WIDTH` (252) minus six 2px
//!   gaps over seven cells fixes the column centres, and the first cell row
//!   sits at y = 74 (24px nav header + gap 8 + ~16px weekday line + gap 8 +
//!   half a 36px cell). Rows step 38px (36 cell + 2 gap). Day *d* of a month
//!   with `lead` leading blanks sits at `idx = d + lead - 1`, row `idx / 7`,
//!   column `idx % 7`. Only the weekday line height is a text metric, and the
//!   click tolerates it the same way `date_picker_close.rs` does.
//! - Nav buttons live in the header row: `.calendar__header` is `px-0.5`, the
//!   buttons are `size-6`, so previous centres at (14, 12) and next at
//!   (238, 12) inside the 252px-wide column.
//! - Bare RangeCalendar: cells are 38px with no column gaps (row `flex_row`
//!   with no gap), so column *c* centres at `19 + 38c`. The first row centres
//!   at y = 75 (the same header stack as the Calendar, half of a 38px cell)
//!   and rows step 40px (38 cell + 2 gap).
//! - Select: the trigger is the 36px field at the origin (centre (60, 18));
//!   the popover hangs from `placed_field_panel(BottomStart, 6px)` at top 42
//!   with `py(6)`, so option row *i* centres at `66 + 36i`. A `section_before`
//!   heading is drawn *above* its option inside the same slot: the header
//!   occupies the top ~24px and the option the bottom 36px.
//! - ColorPicker: the trigger is the swatch + hex label row, 24px tall
//!   (`SizeXl::Sm` swatch), so the panel hangs at top 30 and the `ColorArea`
//!   (240x160) spans x 8..248, y 38..198 after its 8px top/inline inset.
//! - Disclosure / DisclosureGroup: each trigger is a 36px Button.
//! - Toolbar: driven entirely through the keyboard (Tab, arrows, Enter), so
//!   no geometry is needed — the window's tab order is what moves.
//!
//! ColorPicker retains its panel for the pinned 100ms exit. A closed-proof
//! probe against its uncontrolled path advances the deterministic clock first;
//! Select and the bare calendars have no retained exit surface here.

mod harness;

use std::{
    cell::RefCell,
    collections::{BTreeSet, HashSet},
    rc::Rc,
};

use gpui::{
    point, prelude::*, px, Focusable, Modifiers, MouseButton, SharedString, TestAppContext,
};
use harness::{click, events, open_host, press};
use herogpui_components::{
    calendar::{Date, CALENDAR_WIDTH},
    Button, Calendar, CalendarState, ColorPicker, DateConstraints, DateRangeState, Disclosure,
    DisclosureGroup, Input, InputState, PageBehavior, PickerColor, RangeCalendar, Select,
    SelectionMode, Toolbar, VisibleDuration, Weekday,
};

/// Column *c*'s centre in a bare Calendar: seven cells across `CALENDAR_WIDTH`
/// minus six 2px gaps.
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
/// Calendar, derived from the month's leading blanks (Monday-start default,
/// the same `DateConstraints` the test's calendars use).
fn cal_day(year: i32, month: u32, day: u32) -> (f32, f32) {
    let lead = DateConstraints::new().lead_cells(year, month);
    let idx = day as usize + lead - 1;
    (cal_col_x(idx % 7), cal_row_y(idx / 7))
}

/// Column *c*'s centre in a bare RangeCalendar: 38px cells, no column gaps.
fn range_col_x(col: usize) -> f32 {
    19. + 38. * col as f32
}

/// Row *r*'s centre in a bare RangeCalendar: first row at y = 75, then a
/// 38px cell plus a 2px gap per row.
fn range_row_y(row: usize) -> f32 {
    75. + 40. * row as f32
}

/// The centre of the cell holding `day` of `(year, month)` in a bare
/// RangeCalendar.
fn range_day(year: i32, month: u32, day: u32) -> (f32, f32) {
    let lead = DateConstraints::new().lead_cells(year, month);
    let idx = day as usize + lead - 1;
    (range_col_x(idx % 7), range_row_y(idx / 7))
}

// ---------------------------------------------------------------------------
// Calendar & RangeCalendar
// ---------------------------------------------------------------------------

/// The grid is one tab stop (`util::tab_stop_handle`), so Tab from the host
/// root puts the keyboard inside it; then the arrows walk a day and a week,
/// and Enter and Space both take the ring's date. Asserted through
/// `onFocusChange` and `onChange`, never through the ring drawing.
#[gpui::test]
fn calendar_arrows_walk_days_and_weeks(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        let changes = changes.clone();
        // `defaultValue` seeds the selection and the view month, so the ring
        // starts exactly on 2026-08-15 whatever day the tests run.
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    // Tab moves the focus from the host root into the calendar grid.
    press(cx, "tab");
    press(cx, "right");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16"],
        "Right must walk the cursor a day forward"
    );
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-16"],
        "Enter must select the date the ring is on"
    );
    press(cx, "left");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16", "2026-08-15"],
        "Left must walk a day back"
    );
    press(cx, "down");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16", "2026-08-15", "2026-08-22"],
        "Down must walk a week forward"
    );
    press(cx, "up");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16", "2026-08-15", "2026-08-22", "2026-08-15"],
        "Up must walk a week back"
    );
    press(cx, "space");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-16", "2026-08-15"],
        "Space must select like Enter"
    );
}

/// PageUp / PageDown step a month (a year with shift) and Home / End jump to
/// the ends of the month, per React Aria's `useCalendarGrid` mapping that v3
/// inherits: `End -> focusSectionEnd()`, `Home -> focusSectionStart()`,
/// `PageUp/Down -> focusPrevious/NextSection()`, and `larger` for shift. The
/// nav buttons move the visible month on their own, read back through the
/// state entity the test owns.
#[gpui::test]
fn calendar_page_keys_change_month(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-09-15"],
        "PageDown must step a month forward"
    );
    {
        let (year, month) =
            cx.update(|_, cx| (state.read(cx).view_year, state.read(cx).view_month));
        assert_eq!((year, month), (2026, 9), "the view must follow the cursor");
    }

    // gpui's keystroke syntax joins modifiers with `-`, so a shifted page key
    // is `shift-pagedown`, not `shift+pagedown` (which parses as key "+pagedown").
    press(cx, "shift-pagedown");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-09-15", "2027-09-15"],
        "shift+PageDown must step a year forward"
    );
    press(cx, "shift-pageup");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-09-15", "2027-09-15", "2026-09-15"],
        "shift+PageUp must step a year back"
    );
    press(cx, "pageup");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-09-15", "2027-09-15", "2026-09-15", "2026-08-15"],
        "PageUp must step a month back"
    );
    press(cx, "home");
    assert_eq!(
        focused.borrow().as_slice(),
        [
            "2026-09-15",
            "2027-09-15",
            "2026-09-15",
            "2026-08-15",
            "2026-08-01"
        ],
        "Home must jump to the first day of the month"
    );
    press(cx, "end");
    assert_eq!(
        focused.borrow().as_slice(),
        [
            "2026-09-15",
            "2027-09-15",
            "2026-09-15",
            "2026-08-15",
            "2026-08-01",
            "2026-08-31"
        ],
        "End must jump to the last day of the month"
    );

    // The nav buttons move the visible month directly: previous at (14, 12)
    // and next at (238, 12) inside the 252px header row.
    click(cx, 14., 12.);
    {
        let (year, month) =
            cx.update(|_, cx| (state.read(cx).view_year, state.read(cx).view_month));
        assert_eq!((year, month), (2026, 7), "previous must page back a month");
    }
    click(cx, 238., 12.);
    {
        let (year, month) =
            cx.update(|_, cx| (state.read(cx).view_year, state.read(cx).view_month));
        assert_eq!((year, month), (2026, 8), "next must page forward a month");
    }
}

#[gpui::test]
fn calendar_week_page_keys_move_one_week_and_shift_moves_one_month(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .visible_duration(VisibleDuration::Weeks(2))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    press(cx, "shift-pagedown");
    press(cx, "pageup");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-22", "2026-09-22", "2026-09-15"],
        "week sections move seven days, while shifted paging moves one month"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 9, 7),
        "the two-week window must realign at the edge focus crossed"
    );
}

#[gpui::test]
fn calendar_day_page_keys_honor_the_visible_day_count_even_with_shift(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .visible_duration(VisibleDuration::Days(3))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    press(cx, "shift-pagedown");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-18", "2026-08-21"],
        "day sections use pageBehavior and ignore the shift modifier"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 8, 20),
        "the rolling three-day window must advance with each page"
    );
}

#[gpui::test]
fn range_calendar_single_day_pages_one_day(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        RangeCalendar::new(state_for_view.clone())
            .default_value((Date::new(2026, 8, 15), Date::new(2026, 8, 16)))
            .visible_duration(VisibleDuration::Days(3))
            .page_behavior(PageBehavior::Single)
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 8, 15),
        "single-day paging must advance the range calendar window"
    );
    press(cx, "shift-pageup");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-16", "2026-08-15"],
        "single day paging moves one day and ignores shift"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 8, 14),
        "the range calendar day window must follow both page keys"
    );
}

#[gpui::test]
fn range_calendar_week_page_keys_realign_at_the_visible_boundary(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        RangeCalendar::new(state_for_view.clone())
            .default_value((Date::new(2026, 8, 15), Date::new(2026, 8, 16)))
            .visible_duration(VisibleDuration::Weeks(2))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    press(cx, "shift-pagedown");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-22", "2026-09-22"],
        "range week sections share the pinned week and shifted-month steps"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 9, 21),
        "focus beyond the two-week range must realign it at the start"
    );
}

#[gpui::test]
fn calendar_week_home_end_use_the_locale_week(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .visible_duration(VisibleDuration::Weeks(2))
            .first_day_of_week(Weekday::Mon)
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 7, 27),
        "the Monday-first grid must realign so the locale Sunday remains visible"
    );
    press(cx, "end");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-09", "2026-08-15"],
        "week section bounds use the en-US locale week even when the grid overrides its first day"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 8, 10),
        "End must realign the grid forward after the locale week crosses its visible edge"
    );
}

#[gpui::test]
fn calendar_day_home_end_use_the_visible_window(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        Calendar::new(state.clone())
            .default_value(Date::new(2026, 8, 15))
            .visible_duration(VisibleDuration::Days(3))
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    press(cx, "end");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-14", "2026-08-16"],
        "day section bounds are the already-visible dates, not the focused date's month"
    );
}

#[gpui::test]
fn range_calendar_week_home_end_use_the_locale_week(cx: &mut TestAppContext) {
    let focuses = events();
    let focused = focuses.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let focuses = focuses.clone();
        RangeCalendar::new(state_for_view.clone())
            .default_value((Date::new(2026, 8, 15), Date::new(2026, 8, 16)))
            .visible_duration(VisibleDuration::Weeks(2))
            .first_day_of_week(Weekday::Mon)
            .on_focus_change(move |date, _, _| {
                focuses.borrow_mut().push(date.format_iso());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 7, 27),
        "RangeCalendar must keep the locale-week Home target visible"
    );
    press(cx, "end");
    assert_eq!(
        focused.borrow().as_slice(),
        ["2026-08-09", "2026-08-15"],
        "RangeCalendar must share Calendar's locale-week section bounds"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).anchor()),
        Date::new(2026, 8, 10),
        "RangeCalendar must realign again when End crosses the visible edge"
    );
}

/// A day outside `minValue`/`maxValue` is not selectable by either input
/// path: a click on it records nothing, and Enter with the ring on it is
/// stopped by the same `constraints.allows` gate (`calendar.rs`'s key
/// handler). The in-range neighbours all still answer.
#[gpui::test]
fn calendar_min_max_blocks_out_of_range_days(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Calendar::new(state_for_view.clone())
            .default_value(Date::new(2026, 8, 15))
            .min_value(Date::new(2026, 8, 10))
            .max_value(Date::new(2026, 8, 20))
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                changes.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    // August 2026: day 5 is before min (10) in the same grid.
    let (out_x, out_y) = cal_day(2026, 8, 5);
    click(cx, out_x, out_y);
    assert!(
        changed.borrow().is_empty(),
        "clicking an out-of-range day must not select it"
    );

    let (in_x, in_y) = cal_day(2026, 8, 15);
    click(cx, in_x, in_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-15"],
        "an in-range day must still answer the pointer"
    );

    // The pointer leaves the grid focused on its seeded selection (15); five
    // Rights walk to 20 (inside), six to 21 (outside). Enter on 21 is blocked.
    press(cx, "right right right right right");
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-15", "2026-08-20"],
        "Enter on the in-range end of the walk must select it"
    );
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-15", "2026-08-20"],
        "Enter on an out-of-range day must not select it"
    );
}

/// A RangeCalendar's range is a two-step anchor/extend interaction: the first
/// The first click holds an internal anchor and the second reports the sorted
/// completed range in either direction. A click after a complete range starts
/// a new anchor.
/// The hover preview between the two ends must drive the drawing and never a
/// callback.
#[gpui::test]
fn range_calendar_click_start_then_end_reports_completed_ranges(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    // Pin the visible month so the click arithmetic is independent of the
    // day the suite runs on.
    state.update(cx, |s, _| {
        s.view_year = 2026;
        s.view_month = 8;
        s.view_day = 1;
        s.user_navigated = true;
    });
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        RangeCalendar::new(state_for_view.clone())
            .on_change(move |start, end, _, _| {
                changes
                    .borrow_mut()
                    .push(format!("{}->{}", start.format_iso(), end.format_iso()));
            })
            .into_any_element()
    });

    // First click: the anchor remains internal selection state.
    let (day5_x, day5_y) = range_day(2026, 8, 5);
    click(cx, day5_x, day5_y);
    assert!(changed.borrow().is_empty());

    // Hover a later day: the preview paints the range 5..8 (its render reads
    // `state.hovered`), but no callback may fire on its own.
    let (day8_x, day8_y) = range_day(2026, 8, 8);
    cx.simulate_mouse_move(
        point(px(day8_x), px(day8_y)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    {
        let hovered = cx.update(|_, cx| state.read(cx).hovered);
        assert_eq!(
            hovered,
            Some(Date::new(2026, 8, 8)),
            "the hover must reach the cell it passed over"
        );
    }
    assert_eq!(
        changed.borrow().len(),
        0,
        "the hover preview must not report anything on its own"
    );

    // A second click on an earlier day completes the range in reverse. React
    // Aria sorts the two endpoints rather than discarding the first anchor.
    let (day2_x, day2_y) = range_day(2026, 8, 2);
    click(cx, day2_x, day2_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-02->2026-08-05"],
        "a second pick earlier than the anchor must complete the sorted range"
    );

    // A complete range starts over on the next click, then a later click
    // completes that new range.
    click(cx, day2_x, day2_y);
    let (day12_x, day12_y) = range_day(2026, 8, 12);
    click(cx, day12_x, day12_y);
    assert_eq!(
        changed.borrow().as_slice(),
        ["2026-08-02->2026-08-05", "2026-08-02->2026-08-12",],
        "a later pick must complete and report the whole range"
    );
}

// ---------------------------------------------------------------------------
// ColorPicker (ColorSwatch is a static display; see the report)
// ---------------------------------------------------------------------------

/// The trigger opens the panel, a press on the colour area reports a colour,
/// and Escape closes it again. React Aria resolves a pointer against the
/// area's own bounds, so the expected hex subtracts the area origin before
/// deriving the fractions and compares strings — never floats (`float_cmp`
/// is denied).
#[gpui::test]
fn color_picker_trigger_opens_and_area_reports(cx: &mut TestAppContext) {
    harness::still();
    let colors = events();
    let reported = colors.clone();
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(false));

    // The component's recorded border-box turns this panel-relative point into
    // the pinned local-coordinate colour. The direct offset tests derive the
    // fractions independently; this integration keeps the resulting hex.
    let expected_hex = "#6490BD";

    let cx = open_host(cx, move || {
        let colors = colors.clone();
        let opens = opens.clone();
        let open = open.clone();
        let is_open = *open.borrow();
        ColorPicker::new("cp-area", PickerColor::hsb(210.0, 0.5, 0.6))
            .default_value(PickerColor::hsb(210.0, 0.5, 0.6))
            .is_open(is_open)
            .on_open_change(move |v, window, _| {
                *open.borrow_mut() = v;
                opens.borrow_mut().push(format!("open:{v}"));
                window.refresh();
            })
            .on_change(move |color, _, _| {
                colors.borrow_mut().push(color.to_hex());
            })
            .into_any_element()
    });

    // Trigger: swatch (24px) + hex label, 24px tall at the origin.
    click(cx, 60., 12.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the trigger must open the popover"
    );

    // The area: panel top (24 trigger + 6) + pt-2 (8) puts the 160px-tall
    // area at y 38..198; px-2 puts it at x 8..248. The press reports its
    // local fractions within those bounds.
    click(cx, 120., 80.);
    assert_eq!(
        reported.borrow().as_slice(),
        [expected_hex],
        "a press on the area must report the colour it derives from the position"
    );

    // Tab moves the focus trigger -> area; Escape is read on the picker root
    // (`dismiss_on_escape`), which the focused area's key event bubbles to.
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "escape");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "Escape must close the popover"
    );

    // Closed proof: where the area was, nothing answers now.
    click(cx, 120., 80.);
    assert_eq!(
        reported.borrow().as_slice(),
        [expected_hex],
        "the popover must be gone after escape"
    );
}

#[gpui::test]
fn color_picker_default_trigger_owns_open_state_without_callback(cx: &mut TestAppContext) {
    harness::still();
    let colors = events();
    let reported = colors.clone();
    let cx = open_host(cx, move || {
        let reported = reported.clone();
        ColorPicker::new("cp-default-open", PickerColor::hsb(210.0, 0.5, 0.6))
            .on_change(move |color, _, _| reported.borrow_mut().push(color.to_hex()))
            .into_any_element()
    });

    click(cx, 60., 12.);
    click(cx, 120., 80.);
    assert_eq!(
        colors.borrow().as_slice(),
        ["#6490BD"],
        "the default trigger must open its own popover and expose the color controls"
    );

    press(cx, "tab tab escape");
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(150));
    click(cx, 120., 80.);
    assert_eq!(
        colors.borrow().as_slice(),
        ["#6490BD"],
        "Escape must close the uncontrolled popover without an owner callback"
    );
}

#[gpui::test]
fn pointer_open_color_picker_closes_when_focus_moves_elsewhere(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(false));
    let next = cx.new(|cx| InputState::new(cx));
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        let open = open.clone();
        let is_open = *open.borrow();
        gpui::div()
            .child(
                ColorPicker::new("cp-blur", PickerColor::hsb(210.0, 0.5, 0.6))
                    .is_open(is_open)
                    .on_open_change(move |value, window, _| {
                        *open.borrow_mut() = value;
                        opens.borrow_mut().push(format!("open:{value}"));
                        window.refresh();
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    click(cx, 60., 12.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    cx.update(|window, cx| window.focus(&next.read(cx).focus_handle(cx)));
    cx.update(|window, _| window.refresh());

    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
    assert!(cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)));
}

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// `selectionMode="multiple"` keeps the popover open between picks, and each
/// pick is reported as the caller's selection plus the clicked row (`select.rs`
/// hands its `selected_indices` back and stores nothing of its own, so the
/// caller owns the growing set). This test replays that ownership the way a
/// gallery page would — through an `Rc` the render closure reads and a
/// `window.refresh()` in the callback — and the outside-press dismissal still
/// works when the caller finally walks away.
#[gpui::test]
fn select_multiple_accumulates_and_keeps_the_panel_open(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let opens = events();
    let opened = opens.clone();
    let selection = Rc::new(RefCell::new(BTreeSet::<usize>::new()));

    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let opens = opens.clone();
        let selection = selection.clone();
        let selection_now = selection.borrow().iter().copied().collect::<Vec<_>>();
        Select::new(
            "sel-multi",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_indices(selection_now)
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .on_selection_change_all(move |keys, window, _| {
            *selection.borrow_mut() = keys.iter().copied().collect();
            // The port reports the merged set without storing it, so the next
            // report accumulates only if the caller renders it back in.
            window.refresh();
            let joined = keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            picks.borrow_mut().push(joined);
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Row *i* centres at y = 66 + 36i inside the popover.
    click(cx, 60., 66.);
    assert_eq!(picked.borrow().as_slice(), ["0"]);

    click(cx, 60., 138.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["0", "0,2"],
        "the second report must still contain the first pick"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a multiple select must keep the panel open between picks"
    );

    click(cx, 60., 102.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["0", "0,2", "0,1,2"],
        "the third pick must join the accumulated set"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the panel must still be open after the third pick"
    );

    // Pressing the bare page outside the popover dismisses it: the panel is
    // 320 wide and about 150 tall, so (600, 300) is clear of it.
    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "an outside press must dismiss the popover even in multiple mode"
    );
}

/// Pinned Select accepts a `Key[]` as `defaultValue`. In multiple mode that
/// seed belongs to the Select: later picks toggle against the held set even
/// when the caller only observes `onChange` and never feeds a controlled value
/// back.
#[gpui::test]
fn select_uncontrolled_multiple_default_accumulates_and_toggles(cx: &mut TestAppContext) {
    let picks = events();
    let recorded = picks.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        Select::new(
            "sel-multi-default",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_selected_indices([0, 2])
        .default_open(true)
        .on_selection_change_all(move |keys, _, _| {
            picks.borrow_mut().push(
                keys.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .into_any_element()
    });

    // Row *i* centres at y = 66 + 36i inside the already-open popover.
    click(cx, 60., 102.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0,1,2"],
        "the first pick must extend the uncontrolled default array"
    );

    click(cx, 60., 66.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0,1,2", "1,2"],
        "a later pick must toggle against the Select-owned current set"
    );
}

#[gpui::test]
fn select_uncontrolled_multiple_keyboard_toggles_and_stays_open(cx: &mut TestAppContext) {
    let picks = events();
    let recorded = picks.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        Select::new(
            "sel-multi-default-keys",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_selected_indices([0])
        .on_selection_change_all(move |keys, _, _| {
            picks.borrow_mut().push(
                keys.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down down down enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0,1"],
        "Enter must toggle the highlighted row against the uncontrolled default"
    );

    // The multiple popover remains open after keyboard activation, so its
    // third row still answers a pointer press.
    click(cx, 60., 138.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0,1", "0,1,2"],
        "keyboard activation must not close a multiple Select"
    );
}

#[gpui::test]
fn select_controlled_multiple_waits_for_owner_acceptance(cx: &mut TestAppContext) {
    let picks = events();
    let recorded = picks.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        Select::new(
            "sel-multi-controlled-reject",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_indices([0])
        .default_open(true)
        .on_selection_change_all(move |keys, _, _| {
            picks.borrow_mut().push(
                keys.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .into_any_element()
    });

    click(cx, 60., 102.);
    click(cx, 60., 138.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0,1", "0,2"],
        "controlled proposals must keep starting from the owner-supplied set"
    );
}

/// React Aria's typeahead, which `list_nav::typeahead` implements, works in
/// both shapes: typed on a *closed* select it picks the matching option where
/// it stands (no popover), and typed on an *open* one it moves the highlight,
/// which Enter then activates. Two selects with distinct ids so the two halves
/// do not share a typeahead buffer.
#[gpui::test]
fn select_typeahead_moves_the_highlight(cx: &mut TestAppContext) {
    let picked = events();
    let changes = picked.clone();
    let opened_closed = events();
    let closed_opens = opened_closed.clone();
    let opened_open = events();
    let open_opens = opened_open.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let open_pick_changes = changes.clone();
        let closed_opens = closed_opens.clone();
        let open_opens = open_opens.clone();
        // Two selects 600px apart so the lower one's popover never overlaps
        // the upper trigger.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(600.))
            .child(
                Select::new(
                    "sel-ta-closed",
                    vec!["Alpha".into(), "Astra".into(), "Go".into(), "Zig".into()],
                )
                .on_change(move |i, _, _| {
                    changes.borrow_mut().push(format!("{i:?}"));
                })
                .on_open_change(move |open, _, _| {
                    closed_opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(
                Select::new(
                    "sel-ta-open",
                    vec!["Alpha".into(), "Rust".into(), "Go".into(), "Zig".into()],
                )
                .on_selection_change(move |i, _, _| {
                    open_pick_changes.borrow_mut().push(format!("{i:?}"));
                })
                .on_open_change(move |open, _, _| {
                    open_opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .into_any_element()
    });

    // Closed half: Tab reaches the upper trigger without opening it. A letter
    // picks the matching option where it stands; repeating it walks to the
    // *next* row with that initial (React Aria's `aa` behaviour, which the
    // port's `Typeahead` reproduces), so the second pick lands on Astra only
    // if the first one (Alpha) was remembered as the cursor.
    press(cx, "tab");
    press(cx, "a");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(0)"],
        "a letter on the closed select must pick the matching option"
    );
    assert!(
        opened_closed.borrow().is_empty(),
        "the closed typeahead must not open the popover"
    );
    press(cx, "a");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(0)", "Some(1)"],
        "repeating the letter must walk to the next row with that initial"
    );

    // Open half: the trigger at y 636..672 after the 600px gap. "r" moves the
    // highlight to Rust (index 1), and Enter activates the highlighted row
    // and closes (gpui fires the trigger's click listener on the same key,
    // which is the close).
    click(cx, 60., 654.);
    assert_eq!(opened_open.borrow().as_slice(), ["open:true"]);
    press(cx, "r");
    press(cx, "enter");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(0)", "Some(1)", "Some(1)"],
        "Enter must activate the row the typeahead highlighted"
    );
    assert_eq!(
        opened_open.borrow().as_slice(),
        ["open:true", "open:false"],
        "the Enter pick must dismiss the open popover"
    );
}

/// A `ListBox.Section` heading is decoration: a click on it records nothing
/// (it has no handlers), the popover does not close, and the arrows land on
/// options — the cursor's stops are the option indices, so the section never
/// consumes one. The option under the heading still answers both paths.
///
/// A single-mode pick also *reports* the close it performs: a caller who
/// drives `isOpen` from `onOpenChange` has to hear `open:false`. This was a
/// divergence — the row flipped its own keyed open flag without the callback,
/// so a controlled caller kept believing the panel was open and the next
/// render reopened it — so the panel's closure is asserted here through the
/// callback, exactly once, rather than with a probe click.
#[gpui::test]
fn select_row_pick_reports_the_close_once(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(false));
    let open_for_view = open.clone();

    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let opens = opens.clone();
        let open = open_for_view.clone();
        let is_open = *open.borrow();
        Select::new(
            "sel-sections",
            vec![
                "Apple".into(),
                "Banana".into(),
                "Cherry".into(),
                "Durian".into(),
                "Fig".into(),
            ],
        )
        .section_before(3, "Tropical")
        .is_open(is_open)
        .on_selection_change(move |i, _, _| {
            picks.borrow_mut().push(format!("{i:?}"));
        })
        .on_open_change(move |v, window, _| {
            *open.borrow_mut() = v;
            opens.borrow_mut().push(format!("open:{v}"));
            window.refresh();
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Rows 0..2 occupy y 48..156; the row-3 slot (156..216) holds the
    // `pt-6 pb-2` heading across its top ~24px and the option's 36px below.
    click(cx, 60., 165.);
    assert!(
        picked.borrow().is_empty(),
        "a section heading must not be selectable"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a press on the heading must not dismiss the popover either"
    );

    // The option under the heading is a normal pick: the selection is
    // reported, and the close is reported exactly once — the caller's flag
    // flips to false, so the next render keeps the panel shut.
    click(cx, 60., 195.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(3)"],
        "the option a section announces must still be clickable"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the pick must report the close exactly once"
    );
    assert!(
        !*open.borrow(),
        "the caller driving isOpen must now read closed"
    );

    // The panel is gone, so a press where row 0 was records nothing either
    // way — and nothing else reports a close.
    click(cx, 60., 66.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(3)"],
        "after the pick the popover must be gone, so the old row answers nothing"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "a closed panel must not report anything further"
    );

    // Keyboard: Down three times from the top lands on indices 0, 1, 2, and
    // the next Down lands on the option at index 3 — the section heading is
    // never a stop, so it cannot be activated. Enter commits it, and this
    // close is the trigger's own click listener (gpui fires a focused
    // element's click on Enter), which reports through the same callback — a
    // second `open:false`, one per pick, never two for the same one.
    click(cx, 60., 18.);
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Some(3)", "Some(3)"],
        "the arrows must skip the section heading, stopping on its option"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true", "open:false"],
        "the Enter pick must close through the trigger's own click, once"
    );
}

/// Multiple mode keeps the panel open for the next pick, so a pick must not
/// report `open:false` — a caller driving `isOpen` from `onOpenChange` would
/// close the panel between picks. The picks accumulate through the caller's
/// own set (the port hands the merged selection back and stores nothing), and
/// the panel is still answering after each one.
#[gpui::test]
fn select_multiple_picks_report_no_close(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(false));
    let selection = Rc::new(RefCell::new(BTreeSet::<usize>::new()));
    let open_for_view = open.clone();
    let selection_for_view = selection;

    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let opens = opens.clone();
        let open = open_for_view.clone();
        let selection = selection_for_view.clone();
        // Pre-extracted so no `Ref` borrow survives into the builder chain,
        // which moves `open` into the `on_open_change` closure.
        let is_open = *open.borrow();
        let selection_now = selection.borrow().iter().copied().collect::<Vec<_>>();
        Select::new(
            "sel-multi-close",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .is_open(is_open)
        .selected_indices(selection_now)
        .on_open_change(move |v, window, _| {
            *open.borrow_mut() = v;
            opens.borrow_mut().push(format!("open:{v}"));
            window.refresh();
        })
        .on_selection_change_all(move |keys, window, _| {
            *selection.borrow_mut() = keys.iter().copied().collect();
            window.refresh();
            let joined = keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            picks.borrow_mut().push(joined);
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Row *i* centres at y = 66 + 36i inside the popover.
    click(cx, 60., 66.);
    assert_eq!(picked.borrow().as_slice(), ["0"]);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a multiple pick must not report a close"
    );

    // The panel is still open and answering, so the next pick lands — and it
    // still reports nothing about the open state.
    click(cx, 60., 138.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["0", "0,2"],
        "the second pick must join the accumulated set"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the panel must stay open across picks, with no close reports"
    );
    assert!(
        *open.borrow(),
        "the caller driving isOpen must still read open"
    );
}

// ---------------------------------------------------------------------------
// Disclosure / DisclosureGroup
// ---------------------------------------------------------------------------

/// A single `Disclosure` reports the expansion its trigger moves to, and a
/// `DisclosureGroup` reports which item did the moving — with the reflow that
/// follows (the expanded item's body pushes the next one down) proving the
/// expansion is real, not just a callback.
#[gpui::test]
fn disclosure_toggles_and_group_reports(cx: &mut TestAppContext) {
    let toggles = events();
    let toggled = toggles.clone();
    let group_keys = events();
    let reported = group_keys.clone();
    let single_open = Rc::new(RefCell::new(false));
    let expanded = Rc::new(RefCell::new(HashSet::<SharedString>::new()));

    let cx = open_host(cx, move || {
        let toggles = toggles.clone();
        let group_keys = group_keys.clone();
        let single_open = single_open.clone();
        let expanded = expanded.clone();
        let expanded_set = expanded.borrow().clone();
        // The single Disclosure and the group's first item are 100px apart so
        // neither expands into the other's trigger.
        let is_expanded = *single_open.borrow();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                Disclosure::new("calendars-single-disclosure", "General")
                    .is_expanded(is_expanded)
                    .on_expanded_change(move |next, window, _| {
                        *single_open.borrow_mut() = next;
                        toggles.borrow_mut().push(next.to_string());
                        window.refresh();
                    })
                    .child(gpui::div().h(px(20.))),
            )
            .child(
                DisclosureGroup::new("calendars-disclosure-group")
                    .item("grp-a", "Alpha", gpui::div().h(px(20.)))
                    .item("grp-b", "Beta", gpui::div().h(px(20.)))
                    .expanded_keys(expanded_set)
                    .on_expanded_change(move |keys, window, _| {
                        *expanded.borrow_mut() = keys.clone();
                        let key = keys
                            .iter()
                            .next()
                            .map(ToString::to_string)
                            .unwrap_or_default();
                        group_keys.borrow_mut().push(key);
                        window.refresh();
                    }),
            )
            .into_any_element()
    });

    // The single disclosure's trigger is a 36px Button at the origin.
    click(cx, 60., 18.);
    assert_eq!(
        toggled.borrow().as_slice(),
        ["true"],
        "the disclosure must report expanding"
    );
    click(cx, 60., 18.);
    assert_eq!(
        toggled.borrow().as_slice(),
        ["true", "false"],
        "the disclosure must report collapsing on the next press"
    );

    // The group starts at y 136: item A's trigger centres at (60, 154) and
    // item B's at (60, 190) while both are closed.
    click(cx, 60., 190.);
    assert_eq!(reported.borrow().as_slice(), ["grp-b"]);
    click(cx, 60., 154.);
    assert_eq!(reported.borrow().as_slice(), ["grp-b", "grp-a"]);

    // With A expanded its body (p-2 + a 20px child = 36px) pushes B down to
    // y 208..244; a press at the old spot records nothing and the new one
    // reaches B, which is the expansion made observable.
    click(cx, 60., 190.);
    assert_eq!(
        reported.borrow().as_slice(),
        ["grp-b", "grp-a"],
        "an expanding item must push its sibling below its old spot"
    );
    click(cx, 60., 226.);
    assert_eq!(
        reported.borrow().as_slice(),
        ["grp-b", "grp-a", "grp-b"],
        "the moved item must still answer where the layout puts it"
    );
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------

/// A Toolbar's children keep their own press handlers, and the arrows answer
/// the way v3's one-line description advertises ("A container for interactive
/// controls with arrow key navigation") — which, per the inheritance line on
/// the v3 page, is React Aria's `useToolbar`: the arrows move *inside* the
/// toolbar, wrapping at the ends, and Tab is what leaves it.
///
/// This port used to route the arrows through gpui's window-wide
/// `focus_next`/`focus_prev`, which a toolbar-alone page could not tell apart
/// from the scoped behaviour; a button *after* the toolbar is the
/// distinguisher. Everything is asserted through the keyboard: Tab enters on
/// the first control, three Rights from the third wrap back to the first (the
/// window-wide code would have landed Enter on the sibling), Left from the
/// first wraps to the last, and Tab leaves to the sibling.
#[gpui::test]
fn toolbar_arrows_wrap_inside_and_tab_leaves(cx: &mut TestAppContext) {
    let pressed = events();
    let recorded = pressed.clone();

    let cx = open_host(cx, move || {
        let bold_pressed = pressed.clone();
        let italic_pressed = pressed.clone();
        let underline_pressed = pressed.clone();
        let outside_pressed = pressed.clone();
        // A plain button after the toolbar is the probe a window-wide arrow
        // would land on: nothing sits between the toolbar's last control and
        // it, so the old `focus_next` walked straight out to it.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                Toolbar::new()
                    .gap(px(8.))
                    .child(
                        Button::new("tb-bold")
                            .label("Bold")
                            .on_press(move |_, _, _| bold_pressed.borrow_mut().push("bold".into())),
                    )
                    .child(
                        Button::new("tb-italic")
                            .label("Italic")
                            .on_press(move |_, _, _| {
                                italic_pressed.borrow_mut().push("italic".into());
                            }),
                    )
                    .child(Button::new("tb-underline").label("Underline").on_press(
                        move |_, _, _| {
                            underline_pressed.borrow_mut().push("underline".into());
                        },
                    )),
            )
            .child(
                Button::new("tb-outside")
                    .label("Outside")
                    .on_press(move |_, _, _| {
                        outside_pressed.borrow_mut().push("outside".into());
                    }),
            )
            .into_any_element()
    });

    // Tab enters the toolbar on the first control; two Rights walk to the
    // third, and the third Right wraps back to the first instead of walking
    // on to the sibling. Enter reports which control holds the focus.
    press(cx, "tab");
    press(cx, "right right right");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "Right from the last control must wrap to the first, staying inside \
         the toolbar"
    );

    // Left from the first control wraps to the last, again inside.
    press(cx, "left");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "underline"],
        "Left from the first control must wrap to the last control, not escape"
    );

    // Tab is the way out: from the last control it moves to the sibling after
    // the toolbar, whose press then answers.
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "underline", "outside"],
        "Tab must leave the toolbar for the next control in the window"
    );
}

/// A disabled toolbar child is not a tab stop, so the arrows skip it in both
/// directions: Right from the first control lands on the third, and Left from
/// the third lands back on the first. The ends still wrap, over the enabled
/// controls.
#[gpui::test]
fn toolbar_arrows_skip_a_disabled_child(cx: &mut TestAppContext) {
    let pressed = events();
    let recorded = pressed.clone();

    let cx = open_host(cx, move || {
        let bold_pressed = pressed.clone();
        let underline_pressed = pressed.clone();
        Toolbar::new()
            .gap(px(8.))
            .child(
                Button::new("tb-bold")
                    .label("Bold")
                    .on_press(move |_, _, _| bold_pressed.borrow_mut().push("bold".into())),
            )
            .child(Button::new("tb-italic").label("Italic").is_disabled(true))
            .child(
                Button::new("tb-underline")
                    .label("Underline")
                    .on_press(move |_, _, _| {
                        underline_pressed.borrow_mut().push("underline".into());
                    }),
            )
            .into_any_element()
    });

    // Forward: Right skips the disabled control and lands on the third.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["underline"],
        "Right must skip a disabled child"
    );

    // Backward: Left skips it on the way to the first.
    press(cx, "left");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["underline", "bold"],
        "Left must skip a disabled child"
    );

    // The wrap ends stop on enabled controls: Left from the first wraps to the
    // last, Right from the last wraps to the first, and the disabled child is
    // never the destination.
    press(cx, "left");
    press(cx, "enter");
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["underline", "bold", "underline", "bold"],
        "the arrows must wrap over the enabled controls at both ends"
    );
}
