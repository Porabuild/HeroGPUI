//! Calendar & RangeCalendar — port of `@heroui/calendar` and
//! `@heroui/date-picker`'s range grid (std-only date math).

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window,
};
use herogpui_theme::ActiveTheme;

use crate::calendar_view::{self, PageBehavior, SelectionAlignment, VisibleDuration};
use crate::date_constraints::{DateConstraints, Weekday};

use crate::icons;

/// A plain proleptic-Gregorian date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: i32,
    /// 1–12
    pub month: u32,
    /// 1–31
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    pub fn today() -> Self {
        // std has no civil-date API; derive from UNIX epoch days.
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let days = secs.div_euclid(86_400);
        civil_from_days(days)
    }

    pub fn format_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Days since epoch -> civil date (Howard Hinnant's algorithm).
pub fn civil_from_days(z: i64) -> Date {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    Date::new((if m <= 2 { y + 1 } else { y }) as i32, m as u32, d as u32)
}

/// `date` shifted by `delta` days, crossing month and year boundaries.
pub fn add_days(date: &Date, delta: i64) -> Date {
    civil_from_days(days_from_civil(date) + delta)
}

/// Weekday of `date` with Monday as 0, matching [`first_weekday_pub`].
pub fn weekday_index(date: Date) -> usize {
    // Epoch day 0 (1970-01-01) was a Thursday (=3 with Monday as 0).
    (days_from_civil(&date) + 3).rem_euclid(7) as usize
}

/// Three-letter month name for 1-12.
pub fn month_abbr(month: u32) -> &'static str {
    &MONTH_NAMES[(month.clamp(1, 12) - 1) as usize][..3]
}

/// Days-from-civil for `date` (epoch day number).
pub fn days_from_civil(d: &Date) -> i64 {
    let y = if d.month <= 2 { d.year - 1 } else { d.year } as i64;
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = d.month as i64;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d.day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 28,
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// 0 = Monday … 6 = Sunday for the first day of the month.
fn first_weekday(year: i32, month: u32) -> usize {
    let days = days_from_civil(&Date::new(year, month, 1));
    // Epoch day 0 (1970-01-01) was a Thursday (=3 with Monday as 0).
    ((days + 3).rem_euclid(7)) as usize
}

/// `.calendar` is `w-63 max-w-63` — 63 spacing units, so 252px, which is
/// exactly seven 36px cells.
pub const CALENDAR_WIDTH: gpui::Pixels = px(252.);

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn next_month(y: i32, m: u32) -> (i32, u32) {
    if m == 12 {
        (y + 1, 1)
    } else {
        (y, m + 1)
    }
}

fn prev_month(y: i32, m: u32) -> (i32, u32) {
    if m == 1 {
        (y - 1, 12)
    } else {
        (y, m - 1)
    }
}

/// Steps a (year, month) pair by ±1.
pub fn bump_month(y: i32, m: u32, dir: i32) -> (i32, u32) {
    if dir >= 0 {
        next_month(y, m)
    } else {
        prev_month(y, m)
    }
}

/// English month name for 1–12.
pub fn month_name(month: u32) -> &'static str {
    MONTH_NAMES[(month.clamp(1, 12) - 1) as usize]
}

/// Public wrapper over [`first_weekday`].
pub fn first_weekday_pub(year: i32, month: u32) -> usize {
    first_weekday(year, month)
}

// ---------------------------------------------------------------------------
// Single-date calendar
// ---------------------------------------------------------------------------

/// State entity for [`Calendar`].
pub struct CalendarState {
    pub view_year: i32,
    pub view_month: u32,
    /// Anchor day for the week and day views; the month view ignores it.
    pub view_day: u32,
    pub selected: Option<Date>,
    /// Every selected date, for `selectionMode="multiple"`. A single-selection
    /// calendar leaves this empty and reads `selected`.
    pub selected_dates: Vec<Date>,
    /// Set once the user pages or picks a date, after which
    /// `selectionAlignment` stops re-deriving the visible range.
    pub user_navigated: bool,
}

impl CalendarState {
    pub fn new(_cx: &mut App) -> Self {
        let t = Date::today();
        Self {
            view_year: t.year,
            view_month: t.month,
            view_day: t.day,
            selected: None,
            selected_dates: Vec::new(),
            user_navigated: false,
        }
    }

    pub fn with_selected(_cx: &mut App, selected: Date) -> Self {
        Self {
            view_year: selected.year,
            view_month: selected.month,
            view_day: selected.day,
            selected: Some(selected),
            selected_dates: vec![selected],
            user_navigated: false,
        }
    }

    pub fn selected(&self) -> Option<Date> {
        self.selected
    }

    /// Every selected date, in the order they were picked.
    pub fn selected_dates(&self) -> &[Date] {
        &self.selected_dates
    }

    /// Toggles `date` under `mode`, keeping `selected` pointing at the most
    /// recent pick so single-selection callers keep working.
    pub fn toggle(&mut self, date: Date, mode: herogpui_core::SelectionMode) {
        match mode {
            herogpui_core::SelectionMode::None => {}
            herogpui_core::SelectionMode::Single => {
                self.selected = Some(date);
                self.selected_dates = vec![date];
            }
            herogpui_core::SelectionMode::Multiple => {
                match self.selected_dates.iter().position(|d| *d == date) {
                    Some(i) => {
                        self.selected_dates.remove(i);
                        self.selected = self.selected_dates.last().copied();
                    }
                    None => {
                        self.selected_dates.push(date);
                        self.selected = Some(date);
                    }
                }
            }
        }
        self.user_navigated = true;
    }

    /// The date the visible range starts from.
    pub fn anchor(&self) -> Date {
        Date::new(self.view_year, self.view_month, self.view_day.max(1))
    }

    /// Moves the visible range, recording that the user drove it.
    pub fn set_anchor(&mut self, date: Date) {
        self.view_year = date.year;
        self.view_month = date.month;
        self.view_day = date.day;
        self.user_navigated = true;
    }
}

type OnChange = std::sync::Arc<dyn Fn(Option<Date>, &mut Window, &mut App) + 'static>;

/// HeroUI Calendar (single date, controlled selection through the entity).
#[derive(IntoElement)]
pub struct Calendar {
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<Date>,
    id: gpui::ElementId,
    state: Entity<CalendarState>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_read_only: bool,
    /// `Calendar.CellIndicator` — whether a day carries a mark. v3 uses it for
    /// event dots; the closure is handed the date.
    cell_indicator: Option<Box<dyn Fn(Date) -> bool + 'static>>,
    /// `Calendar.NavButton` children — the paging glyphs, previous then next.
    nav_icons: Option<(&'static str, &'static str)>,
    is_invalid: bool,
    focused_value: Option<Date>,
    selection_mode: herogpui_core::SelectionMode,
    duration: VisibleDuration,
    page_behavior: PageBehavior,
    selection_alignment: SelectionAlignment,
    /// `isYearPickerOpen` — `None` leaves the component holding the state,
    /// seeded from `defaultYearPickerOpen`.
    year_picker_open: Option<bool>,
    default_year_picker_open: bool,
    on_year_picker_open_change:
        Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_focus_change: Option<std::sync::Arc<dyn Fn(Date, &mut Window, &mut App) + 'static>>,
    on_change: Option<OnChange>,
}

impl Calendar {
    /// `focusedValue` — the date carrying the focus ring, independent of the
    /// selection.
    pub fn focused_value(mut self, date: Date) -> Self {
        self.focused_value = Some(date);
        self
    }

    /// `onFocusChange` — fires when a different date takes focus.
    pub fn on_focus_change(
        mut self,
        handler: impl Fn(Date, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_focus_change = Some(std::sync::Arc::new(handler));
        self
    }

    /// `value` — writes the selection through to the bound state.
    pub fn value(self, date: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.selected = date);
        self
    }

    pub fn new(state: Entity<CalendarState>) -> Self {
        Self {
            default_value: None,
            id: gpui::ElementId::Name(format!("cal-{}", state.entity_id().as_u64()).into()),
            state,
            constraints: DateConstraints::new(),
            is_disabled: false,
            is_read_only: false,
            cell_indicator: None,
            nav_icons: None,
            is_invalid: false,
            focused_value: None,
            selection_mode: herogpui_core::SelectionMode::Single,
            duration: VisibleDuration::default(),
            page_behavior: PageBehavior::default(),
            selection_alignment: SelectionAlignment::default(),
            year_picker_open: None,
            default_year_picker_open: false,
            on_year_picker_open_change: None,
            on_focus_change: None,
            on_change: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: Date) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `minValue` — the earliest selectable date.
    pub fn min_value(mut self, date: Date) -> Self {
        self.constraints.min_value = Some(date);
        self
    }

    /// `maxValue` — the latest selectable date.
    pub fn max_value(mut self, date: Date) -> Self {
        self.constraints.max_value = Some(date);
        self
    }

    /// `isDateUnavailable` — blocks individual dates inside the range.
    pub fn is_date_unavailable(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.constraints.is_date_unavailable = Some(std::sync::Arc::new(f));
        self
    }

    /// `firstDayOfWeek` — which weekday starts the grid.
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.constraints.first_day_of_week = day;
        self
    }

    /// `weeksInMonth` — forces the grid to this many rows.
    pub fn weeks_in_month(mut self, rows: usize) -> Self {
        self.constraints.weeks_in_month = Some(rows);
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `Calendar.CellIndicator` — mark the days this returns `true` for.
    pub fn cell_indicator(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.cell_indicator = Some(Box::new(f));
        self
    }

    /// `Calendar.NavButton` children — the previous and next glyphs.
    pub fn nav_icons(mut self, previous: &'static str, next: &'static str) -> Self {
        self.nav_icons = Some((previous, next));
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// All five date constraints at once, for callers that already hold a set.
    pub fn constraints(mut self, constraints: DateConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// `selectionMode` — one date or many.
    ///
    /// `Multiple` marks every date in
    /// [`CalendarState::selected_dates`] and toggles on click.
    pub fn selection_mode(mut self, mode: herogpui_core::SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `visibleDuration` — a month view, a run of weeks, or a rolling window
    /// of days.
    pub fn visible_duration(mut self, duration: VisibleDuration) -> Self {
        self.duration = duration;
        self
    }

    /// `pageBehavior` — whether navigation steps the whole visible range or a
    /// single month/week/day.
    pub fn page_behavior(mut self, behavior: PageBehavior) -> Self {
        self.page_behavior = behavior;
        self
    }

    /// `selectionAlignment` — where the selection sits inside the visible
    /// range, until the user navigates for themselves.
    pub fn selection_alignment(mut self, alignment: SelectionAlignment) -> Self {
        self.selection_alignment = alignment;
        self
    }

    /// `isYearPickerOpen` — swaps the day grid for a year grid.
    pub fn is_year_picker_open(mut self, v: bool) -> Self {
        self.year_picker_open = Some(v);
        self
    }

    /// `defaultYearPickerOpen` — the uncontrolled initial state.
    ///
    /// Only consulted when `isYearPickerOpen` is not supplied; the component
    /// then owns the state and the heading toggles it directly.
    pub fn default_year_picker_open(mut self, v: bool) -> Self {
        self.default_year_picker_open = v;
        self
    }

    /// `onYearPickerOpenChange` — supplying it also turns the heading into the
    /// year-picker trigger, so the affordance never appears without a handler
    /// behind it.
    pub fn on_year_picker_open_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_year_picker_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<Date>, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

/// `date` a month away, clamped to the target month's length -- 31 January plus
/// a month is the end of February, not the 31st.
fn month_step(date: Date, delta: i32) -> Date {
    let (year, month) = bump_month(date.year, date.month, delta);
    Date::new(year, month, date.day.min(days_in_month(year, month)))
}

/// The per-frame facts every cell needs, bundled so the helpers below take a
/// readable argument list.
struct Frame<'a> {
    selected: Option<Date>,
    /// Every selected date, for the multiple mode.
    selected_dates: &'a [Date],
    today: Date,
    interactive: bool,
    base: &'a str,
    /// The date wearing the focus ring: `focusedValue` when the caller controls
    /// it, otherwise wherever the arrow keys have walked to. `None` while the
    /// grid does not hold the keyboard.
    focused: Option<Date>,
}

impl Calendar {
    /// One day cell, shared by the month, week and day views.
    fn day_cell(&self, date: Date, frame: &Frame<'_>, key: String, cx: &App) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = colors.accent;
        // In the multiple mode membership of the set is what marks a date.
        let is_sel = if self.selection_mode == herogpui_core::SelectionMode::Multiple {
            frame.selected_dates.contains(&date)
        } else {
            frame.selected == Some(date)
        };
        let is_today = date == frame.today;
        let selectable = frame.interactive && self.constraints.allows(date);
        let unavailable = self.constraints.is_unavailable(date);

        // Uniform circular hit area centred in the slot.
        let mut circle = gpui::div()
            .id(gpui::ElementId::Name(key.into()))
            .size(px(36.))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(14.));

        let marker = if self.is_invalid {
            colors.danger.color
        } else {
            accent.color
        };

        if is_sel {
            circle = circle
                .bg(marker)
                .text_color(if self.is_invalid {
                    colors.danger.foreground
                } else {
                    accent.foreground
                })
                .font_weight(gpui::FontWeight::SEMIBOLD);
        } else if !selectable {
            // Out-of-range and unavailable days both read as non-interactive;
            // unavailable ones keep a rule through them so the reason is
            // distinguishable.
            circle = circle.text_color(colors.muted);
            if unavailable {
                circle = circle.line_through();
            }
        } else {
            circle = circle.text_color(colors.foreground);
            let hover_bg = colors.default.soft_hover();
            circle = circle.cursor_pointer().hover(move |s| s.bg(hover_bg));
            if is_today {
                circle = circle.border_1().border_color(marker);
            }
        }

        // `.calendar__cell` takes `status-focused`, independently of selection,
        // so it shows on an unselected date too. A ring rather than a border:
        // a border shrinks the 36px circle as the cursor lands on it.
        let circle =
            crate::util::with_focus_ring(circle, frame.focused == Some(date), true, Vec::new(), cx);
        let mut circle = circle;

        if selectable {
            let st = self.state.clone();
            let selection_mode = self.selection_mode;
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            circle = circle.on_click(move |_, window, cx| {
                if let Some(cb) = &on_focus {
                    cb(date, window, cx);
                }
                let mode = selection_mode;
                st.update(cx, |s, cx| {
                    // `toggle` also records that the user took over
                    // navigation, so the alignment pass stops moving the range.
                    s.toggle(date, mode);
                    cx.notify();
                });
                if let Some(cb) = &on_change {
                    cb(Some(date), window, cx);
                }
            });
        }

        // `Calendar.CellIndicator` — a dot under the day, which is what v3's
        // event calendar draws.
        let marked = self.cell_indicator.as_ref().is_some_and(|f| f(date));
        gpui::div()
            .flex_1()
            .h(px(36.))
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .child(circle.child(date.day.to_string()))
            .when(marked, |cell| {
                cell.child(
                    gpui::div()
                        .absolute()
                        .bottom(px(2.))
                        .size(px(4.))
                        .rounded_full()
                        .bg(if is_sel { accent.foreground } else { marker }),
                )
            })
            .into_any_element()
    }

    /// The seven column headers.
    fn weekday_header(&self, cx: &App) -> gpui::Div {
        let muted = cx.colors().muted;
        gpui::div()
            .flex()
            .children(self.constraints.first_day_of_week.header_row().map(|d| {
                gpui::div()
                    .flex_1()
                    .text_center()
                    .text_size(px(11.))
                    .text_color(muted)
                    .child(d.to_owned())
            }))
    }

    /// The 7-column grid for a single month, with its lead blanks and the
    /// muted spill into the next month.
    fn month_grid(&self, y: i32, m: u32, frame: &Frame<'_>, cx: &App) -> gpui::AnyElement {
        let muted = cx.colors().muted;
        let lead = self.constraints.lead_cells(y, m);
        let dim = days_in_month(y, m) as usize;
        let rows = self.constraints.rows(y, m);

        let mut grid = gpui::div().flex().flex_col().gap(px(2.));
        for r in 0..rows {
            let mut line = gpui::div().flex().gap(px(2.));
            for c in 0..7 {
                let idx = r * 7 + c;
                let slot: gpui::AnyElement = if idx < lead {
                    gpui::div().flex_1().h(px(34.)).into_any_element()
                } else {
                    let day_num = idx - lead + 1;
                    if day_num > dim {
                        // muted leading days of the next month
                        let nd = day_num - dim;
                        gpui::div()
                            .flex_1()
                            .h(px(34.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(12.5))
                            .text_color(muted)
                            .child(nd.to_string())
                            .into_any_element()
                    } else {
                        self.day_cell(
                            Date::new(y, m, day_num as u32),
                            frame,
                            format!("{}-{y}-{m}-d{day_num}", frame.base),
                            cx,
                        )
                    }
                };
                line = line.child(slot);
            }
            grid = grid.child(line);
        }
        grid.into_any_element()
    }

    /// The year grid shown while the year picker is open.
    fn year_grid(&self, anchor: Date, base: &str, cx: &App) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = colors.accent;
        let years = calendar_view::year_page(anchor.year, 12);
        let mut grid = gpui::div().flex().flex_col().gap(px(4.));
        for chunk in years.chunks(3) {
            let mut row = gpui::div().flex().gap(px(4.));
            for &year in chunk {
                let is_current = year == anchor.year;
                let mut cell = gpui::div()
                    .id(gpui::ElementId::Name(format!("{base}-y{year}").into()))
                    .flex_1()
                    .h(px(38.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(13.))
                    .rounded(crate::util::control_radius(cx));

                if is_current {
                    cell = cell
                        .bg(accent.color)
                        .text_color(accent.foreground)
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else {
                    let hover_bg = colors.default.soft_hover();
                    cell = cell
                        .text_color(colors.foreground)
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg));
                }

                let st = self.state.clone();
                let on_open = self.on_year_picker_open_change.clone();
                cell = cell.on_click(move |_, window, cx| {
                    st.update(cx, |s, cx| {
                        s.set_anchor(Date::new(year, s.view_month, s.view_day.max(1)));
                        cx.notify();
                    });
                    // Picking a year closes the picker, as in v3.
                    if let Some(cb) = &on_open {
                        cb(false, window, cx);
                    }
                });

                row = row.child(cell.child(year.to_string()));
            }
            grid = grid.child(row);
        }
        grid.into_any_element()
    }
}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("calendar-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.selected = Some(value);
                        s.selected_dates = vec![value];
                        s.view_year = value.year;
                        s.view_month = value.month;
                        s.view_day = value.day;
                        cx.notify();
                    });
                },
            );
        }

        let base = format!("{:?}", self.id);

        // `isYearPickerOpen` wins; without it the component holds the flag and
        // the heading toggles it, which is what `defaultYearPickerOpen`
        // promises. This borrows `cx` mutably, so it precedes the tokens.
        let (year_picker_open, year_picker_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{base}-yearpicker").into()),
            self.year_picker_open,
            self.default_year_picker_open,
        );

        // The grid is one tab stop with a cursor inside it, the way a list is:
        // v3 gives the calendar a roving focus and rings the date it is on.
        // `use_keyed_state` takes `cx` mutably, so both precede the theme.
        let grid_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base}-focus").into()),
            window,
            cx,
        );
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-cursor").into()),
            cx,
            |_, _| None::<Date>,
        );
        let cursor_at = *cursor.read(cx);

        let colors = cx.colors();
        let layout = cx.layout();
        let first_day = self.constraints.first_day_of_week;

        let (stored_anchor, selected, selected_dates, navigated) = {
            let st = self.state.read(cx);
            (
                st.anchor(),
                st.selected,
                st.selected_dates.clone(),
                st.user_navigated,
            )
        };
        // `selectionAlignment` frames the range around the selection, but only
        // until the user drives navigation themselves.
        let anchor = match (navigated, selected) {
            (false, Some(sel)) => calendar_view::aligned_anchor(
                self.duration,
                self.selection_alignment,
                first_day,
                sel,
            ),
            _ => stored_anchor,
        };
        // `focusedValue` wins; without it the keyboard's own cursor does, and it
        // starts from the selection or today.
        // Taking the focus puts the ring on the date v3 would have focused --
        // the selection, or today -- rather than waiting for a keystroke to
        // place it.
        let ring_at = self
            .focused_value
            .or(cursor_at)
            .or(selected)
            .or_else(|| Some(Date::today()))
            .filter(|_| grid_focus.is_focused(window));
        let frame = Frame {
            selected,
            selected_dates: &selected_dates,
            today: Date::today(),
            interactive: !self.is_disabled && !self.is_read_only,
            base: &base,
            focused: ring_at,
        };

        let months = calendar_view::month_headings(self.duration, anchor);
        let linear = calendar_view::linear_cells(self.duration, first_day, anchor);
        let columns = months.len().max(1);

        let nav_target =
            |dir: i32| calendar_view::page(self.duration, self.page_behavior, anchor, dir);
        let state_for_nav = self.state.clone();
        // `Calendar.NavButton` children, defaulting to v3's chevrons.
        let (prev_icon, next_icon) = self
            .nav_icons
            .unwrap_or((icons::CHEVRON_LEFT, icons::CHEVRON_RIGHT));
        let nav_btn = |icon_path: &'static str, target: Date, key: String| {
            let state = state_for_nav.clone();
            let hover_bg = colors.default.soft_hover();
            gpui::div()
                .id(gpui::ElementId::Name(key.into()))
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.))
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(move |_, _, cx| {
                    state.update(cx, |s, cx| {
                        s.set_anchor(target);
                        cx.notify();
                    });
                })
                .child(
                    gpui::svg()
                        .size(px(13.))
                        .path(icon_path)
                        .text_color(colors.foreground),
                )
        };

        // A heading is a plain label unless a year-picker handler is supplied,
        // in which case it becomes the trigger.
        let heading = |text: String, key: String| -> gpui::AnyElement {
            let label = gpui::div()
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(text);
            match &self.on_year_picker_open_change {
                // No handler and no state of our own: a plain label.
                None if year_picker_own.is_none() => label.into_any_element(),
                None => {
                    let own = year_picker_own.clone();
                    let open = year_picker_open;
                    let hover_bg = colors.default.soft_hover();
                    gpui::div()
                        .id(gpui::ElementId::Name(key.into()))
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(crate::util::control_radius(cx))
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(move |_, _, cx| {
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = !open;
                                    cx.notify();
                                });
                            }
                        })
                        .child(label)
                        .child(
                            gpui::svg()
                                .size(px(12.))
                                .path(if open {
                                    icons::CHEVRON_UP
                                } else {
                                    icons::CHEVRON_DOWN
                                })
                                .text_color(colors.muted),
                        )
                        .into_any_element()
                }
                Some(cb) => {
                    let cb = cb.clone();
                    let open = year_picker_open;
                    let own = year_picker_own.clone();
                    let hover_bg = colors.default.soft_hover();
                    gpui::div()
                        .id(gpui::ElementId::Name(key.into()))
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(crate::util::control_radius(cx))
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(move |_, window, cx| {
                            // Uncontrolled: flip our own copy too, or the
                            // caller's handler would be the only thing that
                            // could open it.
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = !open;
                                    cx.notify();
                                });
                            }
                            cb(!open, window, cx);
                        })
                        .child(label)
                        .child(
                            gpui::svg()
                                .size(px(12.))
                                .path(if open {
                                    icons::CHEVRON_UP
                                } else {
                                    icons::CHEVRON_DOWN
                                })
                                .text_color(colors.muted),
                        )
                        .into_any_element()
                }
            }
        };

        let column_width = if columns > 1 {
            px(228.)
        } else {
            CALENDAR_WIDTH
        };

        let mut root = gpui::div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(colors.surface.foreground)
            .track_focus(&grid_focus);

        // v3 drives a calendar from the keyboard: the arrows step a day and a
        // week, Page Up/Down a month, Home and End the ends of the month, and
        // Enter takes the date the ring is on.
        if !self.is_disabled && !self.is_read_only {
            let held = cursor;
            let start = self.focused_value.or(selected).unwrap_or_else(Date::today);
            let constraints = self.constraints.clone();
            let state = self.state.clone();
            let mode = self.selection_mode;
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            root = root.on_key_down(move |event, window, cx| {
                let from = *held.read(cx);
                let at = from.unwrap_or(start);
                let key = event.keystroke.key.as_str();
                if matches!(key, "enter" | "space") {
                    if !constraints.allows(at) {
                        return;
                    }
                    state.update(cx, |s, cx| {
                        s.toggle(at, mode);
                        cx.notify();
                    });
                    if let Some(cb) = &on_change {
                        cb(Some(at), window, cx);
                    }
                    return;
                }
                let next = match key {
                    "left" => add_days(&at, -1),
                    "right" => add_days(&at, 1),
                    "up" => add_days(&at, -7),
                    "down" => add_days(&at, 7),
                    "pageup" => month_step(at, -1),
                    "pagedown" => month_step(at, 1),
                    "home" => Date::new(at.year, at.month, 1),
                    "end" => Date::new(at.year, at.month, days_in_month(at.year, at.month)),
                    _ => return,
                };
                held.update(cx, |v, cx| {
                    *v = Some(next);
                    cx.notify();
                });
                if let Some(cb) = &on_focus {
                    cb(next, window, cx);
                }
            });
        }

        if year_picker_open {
            // The picker replaces the grid area in every view.
            root = root.w(column_width);
            root = root.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(nav_btn(
                        icons::CHEVRON_LEFT,
                        Date::new(anchor.year - 12, anchor.month, anchor.day),
                        format!("{base}-yprev"),
                    ))
                    .child(heading(
                        format!("{} {}", month_name(anchor.month), anchor.year),
                        format!("{base}-yheading"),
                    ))
                    .child(nav_btn(
                        icons::CHEVRON_RIGHT,
                        Date::new(anchor.year + 12, anchor.month, anchor.day),
                        format!("{base}-ynext"),
                    )),
            );
            root = root.child(self.year_grid(anchor, &base, cx));
        } else if self.duration.is_month_view() {
            let mut row = gpui::div().flex().gap(px(20.));
            for (i, &(y, m)) in months.iter().enumerate() {
                let first = i == 0;
                let last = i + 1 == columns;
                let mut col = gpui::div().w(column_width).flex().flex_col().gap(px(8.));
                // Only the outer columns carry nav buttons; the others keep
                // a same-size spacer so every heading lines up.
                let spacer = || gpui::div().size(px(28.)).into_any_element();
                col = col.child(
                    gpui::div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(if first {
                            nav_btn(prev_icon, nav_target(-1), format!("{base}-prev"))
                                .into_any_element()
                        } else {
                            spacer()
                        })
                        .child(heading(
                            format!("{} {}", month_name(m), y),
                            format!("{base}-heading{i}"),
                        ))
                        .child(if last {
                            nav_btn(next_icon, nav_target(1), format!("{base}-next"))
                                .into_any_element()
                        } else {
                            spacer()
                        }),
                );
                col = col.child(self.weekday_header(cx));
                col = col.child(self.month_grid(y, m, &frame, cx));
                row = row.child(col);
            }
            root = root.child(row);
        } else {
            // Week and day views: one flat run of real dates, so there are no
            // lead blanks and no spill into the next month.
            let per_row = if matches!(self.duration, VisibleDuration::Weeks(_)) {
                7
            } else {
                linear.len().max(1)
            };
            root = root.w(CALENDAR_WIDTH);
            root = root.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(nav_btn(
                        icons::CHEVRON_LEFT,
                        nav_target(-1),
                        format!("{base}-prev"),
                    ))
                    .child(heading(
                        calendar_view::range_heading(&linear),
                        format!("{base}-heading"),
                    ))
                    .child(nav_btn(
                        icons::CHEVRON_RIGHT,
                        nav_target(1),
                        format!("{base}-next"),
                    )),
            );
            if per_row == 7 {
                root = root.child(self.weekday_header(cx));
            } else {
                // A day view labels each visible column with its own weekday.
                root = root.child(gpui::div().flex().children(linear.iter().map(|d| {
                    gpui::div()
                        .flex_1()
                        .text_center()
                        .text_size(px(11.))
                        .text_color(colors.muted)
                        .child(Weekday::ALL[weekday_index(*d)].short_label().to_owned())
                })));
            }
            let mut grid = gpui::div().flex().flex_col().gap(px(2.));
            for chunk in linear.chunks(per_row) {
                let mut line = gpui::div().flex().gap(px(2.));
                for &date in chunk {
                    line = line.child(self.day_cell(
                        date,
                        &frame,
                        format!("{base}-{}", date.format_iso()),
                        cx,
                    ));
                }
                grid = grid.child(line);
            }
            root = root.child(grid);
        }

        if self.is_disabled {
            root = root.opacity(layout.disabled_opacity);
        }

        root
    }
}
