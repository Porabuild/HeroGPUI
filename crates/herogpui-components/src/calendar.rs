//! Calendar & RangeCalendar — port of `@heroui/calendar` and
//! `@heroui/date-picker`'s range grid (std-only date math).

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use std::sync::OnceLock;

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
        // v3 marks "today" through React Aria's `today(getLocalTimeZone())`:
        // the OS local zone's civil date, not UTC's. West of UTC the UTC
        // date is ahead of the local one; east of UTC it is behind.
        let now = jiff::Zoned::now();
        civil_date_at(now.timestamp().as_second(), now.offset().seconds())
    }

    pub fn format_iso(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The civil date for `unix_secs` seconds past the UNIX epoch, observed in a
/// zone `utc_offset_secs` east of UTC. Private seam so tests can prove the
/// day crossings of [`Date::today`] without changing the machine timezone.
fn civil_date_at(unix_secs: i64, utc_offset_secs: i32) -> Date {
    civil_from_days((unix_secs + i64::from(utc_offset_secs)).div_euclid(86_400))
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

/// The running locale's abbreviated month name for 1-12.
///
/// CLDR is asked for the abbreviation rather than the first three bytes of the
/// full name: `"Januar"[..3]` happens to read well in German and would panic
/// outright on a Japanese month name whose first character is three bytes wide.
pub fn month_abbr(month: u32) -> &'static str {
    &system_month_abbrs()[(month.clamp(1, 12) - 1) as usize]
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
        2 if is_leap(year) => 29,
        2 => 28,
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

/// The English month names, used when CLDR has nothing to say for the running
/// locale. A calendar with no heading is worse than one headed in English.
const FALLBACK_MONTH_NAMES: [&str; 12] = [
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

/// The month names CLDR gives for one locale, in calendar order.
///
/// v3 renders the heading through `Intl.DateTimeFormat`, so the names follow
/// the reader's locale rather than the source language. The ICU data is
/// compiled into the binary, so this asks CLDR the same question `Intl` does.
pub(crate) fn month_names_for_locale(locale: &str) -> Option<[String; 12]> {
    month_labels_for_locale(locale, icu_datetime::fieldsets::M::long())
}

/// The twelve month labels one `M` field set prints for one locale.
fn month_labels_for_locale(
    locale: &str,
    fieldset: icu_datetime::fieldsets::M,
) -> Option<[String; 12]> {
    use icu_datetime::{input::Date as IcuDate, DateTimeFormatter};
    use icu_locale_core::Locale as IcuLocale;

    let locale = locale.parse::<IcuLocale>().ok()?;
    let formatter = DateTimeFormatter::try_new(locale.into(), fieldset).ok()?;
    let mut labels: Vec<String> = Vec::with_capacity(12);
    for month in 1..=12u8 {
        // Any year and day work: the `M` field set prints the month alone.
        let date = IcuDate::try_new_iso(2000, month, 1).ok()?;
        labels.push(formatter.format(&date).to_string());
    }
    labels.try_into().ok()
}

/// The abbreviated month names CLDR gives for one locale, in calendar order.
pub(crate) fn month_abbrs_for_locale(locale: &str) -> Option<[String; 12]> {
    month_labels_for_locale(locale, icu_datetime::fieldsets::M::medium())
}

/// The running locale's month names, resolved once.
fn system_month_names() -> &'static [String; 12] {
    static SYSTEM_MONTH_NAMES: OnceLock<[String; 12]> = OnceLock::new();
    SYSTEM_MONTH_NAMES.get_or_init(|| {
        crate::date_constraints::system_locale_tags()
            .iter()
            .find_map(|tag| month_names_for_locale(tag))
            .unwrap_or_else(|| FALLBACK_MONTH_NAMES.map(str::to_owned))
    })
}

/// The heading CLDR gives one locale for a year and month.
///
/// The order is the locale's, not a template's: `"January 2026"` in English and
/// a year-first heading in Japanese, which no `"{month} {year}"` format string
/// can produce.
pub(crate) fn month_heading_for_locale(locale: &str, year: i32, month: u32) -> Option<String> {
    use icu_datetime::{fieldsets, input::Date as IcuDate, options::YearStyle, DateTimeFormatter};
    use icu_locale_core::Locale as IcuLocale;

    let locale = locale.parse::<IcuLocale>().ok()?;
    let formatter = DateTimeFormatter::try_new(
        locale.into(),
        fieldsets::YM::long().with_year_style(YearStyle::Full),
    )
    .ok()?;
    let month = u8::try_from(month.clamp(1, 12)).ok()?;
    Some(
        formatter
            .format(&IcuDate::try_new_iso(year, month, 1).ok()?)
            .to_string(),
    )
}

/// The running locale's year-and-month heading locale, resolved once.
fn system_month_heading_locale() -> Option<&'static String> {
    static SYSTEM_HEADING_LOCALE: OnceLock<Option<String>> = OnceLock::new();
    SYSTEM_HEADING_LOCALE
        .get_or_init(|| {
            crate::date_constraints::system_locale_tags()
                .into_iter()
                .find(|tag| month_heading_for_locale(tag, 2000, 1).is_some())
        })
        .as_ref()
}

/// The heading for a year and month, in the running locale.
pub fn month_year_heading(year: i32, month: u32) -> String {
    system_month_heading_locale()
        .and_then(|locale| month_heading_for_locale(locale, year, month))
        .unwrap_or_else(|| format!("{} {year}", month_name(month)))
}

/// The running locale's abbreviated month names, resolved once.
fn system_month_abbrs() -> &'static [String; 12] {
    static SYSTEM_MONTH_ABBRS: OnceLock<[String; 12]> = OnceLock::new();
    SYSTEM_MONTH_ABBRS.get_or_init(|| {
        crate::date_constraints::system_locale_tags()
            .iter()
            .find_map(|tag| month_abbrs_for_locale(tag))
            .unwrap_or_else(|| {
                FALLBACK_MONTH_NAMES.map(|name| name.chars().take(3).collect::<String>())
            })
    })
}

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

/// Steps a (year, month) pair by ±1. `dir` is a *direction*, not a count.
pub fn bump_month(y: i32, m: u32, dir: i32) -> (i32, u32) {
    if dir >= 0 {
        next_month(y, m)
    } else {
        prev_month(y, m)
    }
}

/// The (year, month) `delta` months away, for any size of `delta`.
///
/// `bump_month` reads its argument as a direction and moves one month whatever
/// the magnitude, which made shift+Page Up move a single month instead of a
/// year. Counting in months from year zero keeps the wrap arithmetic in one
/// place.
pub fn add_months(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let total = i64::from(year) * 12 + i64::from(month) - 1 + i64::from(delta);
    #[allow(clippy::cast_possible_truncation)]
    let y = total.div_euclid(12) as i32;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let m = total.rem_euclid(12) as u32 + 1;
    (y, m)
}

/// English month name for 1–12.
pub fn month_name(month: u32) -> &'static str {
    &system_month_names()[(month.clamp(1, 12) - 1) as usize]
}

/// Public wrapper over `first_weekday`.
pub fn first_weekday_pub(year: i32, month: u32) -> usize {
    first_weekday(year, month)
}

// ---------------------------------------------------------------------------
// Calendar
// ---------------------------------------------------------------------------

/// State entity for [`Calendar`].
pub struct CalendarState {
    pub view_year: i32,
    pub view_month: u32,
    /// Anchor day for the week and day views; the month view ignores it.
    pub view_day: u32,
    pub selected: Option<Date>,
    /// Every selected date. Multiple mode toggles this set; `selected` keeps
    /// the most recent value for scalar callers.
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
type OnChangeAll = std::sync::Arc<dyn Fn(&[Date], &mut Window, &mut App) + 'static>;

/// What `Calendar.Cell`'s render function is handed.
///
/// v3's render props for the cell: `formattedDate` is the localized day label,
/// and the state fields say what the cell is.
#[derive(Clone, Debug)]
pub struct CalendarCellState {
    /// The date this cell draws.
    pub date: Date,
    /// `formattedDate` — the day label, as this port writes it.
    pub formatted_date: SharedString,
    /// `isSelected`
    pub is_selected: bool,
    /// `isUnavailable`
    pub is_unavailable: bool,
    /// `isOutsideMonth`
    pub is_outside_month: bool,
    /// Today, which v3 marks with `data-today`.
    pub is_today: bool,
    /// Outside the min/max range, or the calendar is disabled.
    pub is_disabled: bool,
}

/// HeroUI Calendar, with controlled selection through the entity.
#[derive(IntoElement)]
pub struct Calendar {
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<Date>,
    /// `defaultValue` for `selectionMode="multiple"`.
    default_values: Option<Vec<Date>>,
    id: gpui::ElementId,
    state: Entity<CalendarState>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_read_only: bool,
    /// Set by a picker: take the focus as the panel opens. See
    /// [`Calendar::autofocus_grid`].
    autofocus_grid: bool,
    /// The calendar system the grid is drawn in, when the caller names one.
    /// `None` follows the operating system's locale, which is v3's default.
    calendar_system: Option<crate::calendar_system::CalendarSystem>,
    /// `Calendar.CellIndicator` — whether a day carries a mark. v3 uses it for
    /// event dots; the closure is handed the date.
    cell_indicator: Option<Box<dyn Fn(Date) -> bool + 'static>>,
    /// `Calendar.Cell`'s render props: the closure replaces the day label and is
    /// handed the state v3 passes it.
    cell: Option<Box<dyn Fn(CalendarCellState) -> gpui::AnyElement + 'static>>,
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
    /// `Calendar.YearPickerGrid.visibleYears` — `None` uses the v3 default:
    /// the full min/max span when both exist, otherwise 20.
    visible_years: Option<usize>,
    /// `Calendar.YearPickerTriggerHeading.offset.months`.
    year_heading_offset_months: i32,
    on_year_picker_open_change:
        Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_focus_change: Option<std::sync::Arc<dyn Fn(Date, &mut Window, &mut App) + 'static>>,
    on_change: Option<OnChange>,
    on_change_all: Option<OnChangeAll>,
}

impl Calendar {
    /// The locale whose calendar system this grid is drawn in.
    ///
    /// v3 has no prop for this: it wraps the calendar in an `I18nProvider`
    /// whose locale carries the Unicode `-u-ca-` extension, and its
    /// "International Calendars" example is exactly that. gpui has no subtree
    /// context to put a provider in, so the locale is named on the component
    /// the provider would have wrapped. Omit it and the operating system's
    /// locale decides, which is what v3 does by default.
    pub fn locale(mut self, tag: impl AsRef<str>) -> Self {
        self.calendar_system = crate::calendar_system::CalendarSystem::for_locale(tag.as_ref());
        self
    }

    /// The system this grid measures in: the caller's, else the platform's.
    fn system(&self) -> &crate::calendar_system::CalendarSystem {
        self.calendar_system
            .as_ref()
            .unwrap_or_else(|| crate::calendar_system::system())
    }

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
        self.state.update(cx, |s, _| {
            s.selected = date;
            s.selected_dates = date.into_iter().collect();
        });
        self
    }

    /// `value` for `selectionMode="multiple"`.
    pub fn values(self, dates: impl IntoIterator<Item = Date>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| {
            s.selected_dates = dates.into_iter().collect();
            s.selected = s.selected_dates.last().copied();
        });
        self
    }

    pub fn new(state: Entity<CalendarState>) -> Self {
        Self {
            default_value: None,
            default_values: None,
            id: gpui::ElementId::Name(format!("cal-{}", state.entity_id().as_u64()).into()),
            state,
            constraints: DateConstraints::new().with_hero_calendar_bounds(),
            is_disabled: false,
            is_read_only: false,
            autofocus_grid: false,
            calendar_system: None,
            cell_indicator: None,
            cell: None,
            nav_icons: None,
            is_invalid: false,
            focused_value: None,
            selection_mode: herogpui_core::SelectionMode::Single,
            duration: VisibleDuration::default(),
            page_behavior: PageBehavior::default(),
            selection_alignment: SelectionAlignment::default(),
            year_picker_open: None,
            default_year_picker_open: false,
            visible_years: None,
            year_heading_offset_months: 0,
            on_year_picker_open_change: None,
            on_focus_change: None,
            on_change: None,
            on_change_all: None,
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

    /// `defaultValue` for `selectionMode="multiple"`.
    pub fn default_values(mut self, values: impl IntoIterator<Item = Date>) -> Self {
        self.default_values = Some(values.into_iter().collect());
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
    /// `Calendar.Cell`'s render function — draw the day yourself.
    ///
    /// v3 hands it `{formattedDate, isSelected, isUnavailable, isOutsideMonth}`;
    /// this port computes each of those to draw the cell, so the closure is
    /// handed the same [`CalendarCellState`] rather than the values being
    /// unavailable.
    pub fn cell(
        mut self,
        render: impl Fn(CalendarCellState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.cell = Some(Box::new(render));
        self
    }

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

    /// Takes the focus the first time the grid renders.
    ///
    /// Not a v3 prop: React Aria moves the focus into the calendar when a date
    /// picker's popover opens, and a picker whose arrows do nothing until the
    /// user finds it with Tab is not the same component. Crate-only, because a
    /// standalone calendar must *not* steal the focus -- a page with three of
    /// them would fight over it.
    pub(crate) fn autofocus_grid(mut self, v: bool) -> Self {
        self.autofocus_grid = v;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// All five date constraints at once, for callers that already hold a set.
    pub fn constraints(mut self, constraints: DateConstraints) -> Self {
        self.constraints = constraints.with_hero_calendar_bounds();
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

    /// `Calendar.YearPickerGrid.visibleYears` — the size of its sliding window.
    pub fn visible_years(mut self, count: usize) -> Self {
        self.visible_years = Some(count.max(1));
        self
    }

    /// `Calendar.YearPickerTriggerHeading.offset` — shifts its displayed month.
    pub fn offset(mut self, months: i32) -> Self {
        self.year_heading_offset_months = months;
        self
    }

    /// `onYearPickerOpenChange` — reports the built-in heading trigger opening
    /// or closing the year grid.
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

    /// `onChange` for `selectionMode="multiple"`.
    pub fn on_change_all(mut self, f: impl Fn(&[Date], &mut Window, &mut App) + 'static) -> Self {
        self.on_change_all = Some(std::sync::Arc::new(f));
        self
    }
}

/// The per-frame facts every cell needs, bundled so the helpers below take a
/// readable argument list.
struct Frame<'a> {
    selected: Option<Date>,
    /// Every selected date, for the multiple mode.
    selected_dates: &'a [Date],
    today: Date,
    cursor: &'a Entity<Option<Date>>,
    base: &'a str,
    /// The date wearing the focus ring: `focusedValue` when the caller controls
    /// it, otherwise wherever the arrow keys have walked to. `None` while the
    /// grid does not hold the keyboard.
    focused: Option<Date>,
}

impl Calendar {
    /// One day cell, shared by the month, week and day views.
    fn day_cell(
        &self,
        date: Date,
        outside_month: bool,
        frame: &Frame<'_>,
        key: String,
        cx: &App,
    ) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = colors.accent;
        let unavailable = self.constraints.is_unavailable(date);
        let disabled = outside_month || self.is_disabled || self.constraints.out_of_range(date);
        let focusable = !disabled;
        let eligible = focusable && !unavailable;
        let selectable = eligible && !self.is_read_only;
        // In the multiple mode membership of the set is what marks a date.
        let is_sel = eligible
            && if self.selection_mode == herogpui_core::SelectionMode::Multiple {
                frame.selected_dates.contains(&date)
            } else {
                frame.selected == Some(date)
            };
        let is_today = date == frame.today;

        // Uniform circular hit area centred in the slot. The debug selector
        // lets the headless tests read the cell's laid-out bounds.
        let indicator_key = format!("{key}-indicator");
        let mut circle = gpui::div()
            .id(gpui::ElementId::Name(key.clone().into()))
            .debug_selector(move || key)
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

        // `.calendar__cell[data-pressed]` fills with `bg-default` and scales
        // to 0.95 -- every cell, today included.
        let press_box = crate::anim::PressBox {
            height: px(36.),
            padding_x: None,
            width: Some(px(36.)),
            min_width: None,
            text_size: px(14.),
            line_height: px(20.),
            gap: px(0.),
            radius: px(18.),
            shrink_x: true,
            scale: crate::anim::PRESSED_SCALE_DEEP,
        };
        if outside_month {
            circle = circle.text_color(colors.muted);
        } else if is_sel {
            circle = circle
                .bg(marker)
                .text_color(if self.is_invalid {
                    colors.danger.foreground
                } else {
                    accent.foreground
                })
                .font_weight(gpui::FontWeight::SEMIBOLD);
            if selectable {
                // `.calendar__cell[data-pressed][data-selected]` fills
                // `bg-accent-hover`, the one pressed recolour the pinned CSS
                // nests under the scale.
                let pressed_bg = if self.is_invalid {
                    colors.danger.hover()
                } else {
                    accent.hover()
                };
                circle = crate::anim::pressed_with_background(circle, press_box, pressed_bg, cx);
            }
        } else if disabled || unavailable {
            // v3 dims both states and reserves the line-through for disabled
            // in-month dates; unavailable dates remain focusable.
            circle = circle.text_color(colors.muted);
            if disabled {
                circle = circle.line_through();
            }
        } else if is_today {
            // `.calendar__cell[data-today]` fills `bg-accent-soft` with
            // `text-accent-soft-foreground`; its own hover (not selected)
            // deepens the same soft fill rather than the generic `bg-default`.
            circle = circle
                .bg(accent.soft())
                .text_color(accent.soft_foreground(colors.foreground));
            if selectable {
                let hover_bg = accent.soft_hover();
                let pressed_bg = colors.default.color;
                circle = circle.cursor_pointer().hover(move |s| s.bg(hover_bg));
                // A chained `.active` would overwrite the pressed refinement
                // and drop the 0.95 scale; the background must merge with the
                // press geometry in one refinement.
                circle = crate::anim::pressed_with_background(circle, press_box, pressed_bg, cx);
            }
        } else {
            circle = circle.text_color(colors.foreground);
            if selectable {
                // `.calendar__cell:hover` (not selected) fills with `bg-default`,
                // the full token -- same as the pressed fill.
                let hover_bg = colors.default.color;
                let pressed_bg = colors.default.color;
                circle = circle.cursor_pointer().hover(move |s| s.bg(hover_bg));
                circle = crate::anim::pressed_with_background(circle, press_box, pressed_bg, cx);
            }
        }

        // `.calendar__cell` takes `status-focused`, independently of selection,
        // so it shows on an unselected date too. A ring rather than a border:
        // a border shrinks the 36px circle as the cursor lands on it.
        let circle = crate::util::with_focus_ring(
            circle,
            !outside_month && frame.focused == Some(date),
            true,
            Vec::new(),
            cx,
        );
        let mut circle = circle;

        if selectable {
            let cursor = frame.cursor.clone();
            let st = self.state.clone();
            let selection_mode = self.selection_mode;
            let on_change = self.on_change.clone();
            let on_change_all = self.on_change_all.clone();
            let on_focus = self.on_focus_change.clone();
            circle = circle.on_click(move |_, window, cx| {
                cursor.update(cx, |focused, cx| {
                    *focused = Some(date);
                    cx.notify();
                });
                if let Some(cb) = &on_focus {
                    cb(date, window, cx);
                }
                let mode = selection_mode;
                let selected_dates = st.update(cx, |s, cx| {
                    // `toggle` also records that the user took over
                    // navigation, so the alignment pass stops moving the range.
                    s.toggle(date, mode);
                    cx.notify();
                    s.selected_dates.clone()
                });
                if let Some(cb) = &on_change {
                    cb(Some(date), window, cx);
                }
                if let Some(cb) = &on_change_all {
                    cb(&selected_dates, window, cx);
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
            .child(match &self.cell {
                Some(render) => circle.child(render(CalendarCellState {
                    date,
                    formatted_date: self.day_label(date).into(),
                    is_selected: is_sel,
                    is_unavailable: unavailable,
                    is_outside_month: outside_month,
                    is_today,
                    is_disabled: disabled,
                })),
                None => circle.child(self.day_label(date)),
            })
            .when(marked, |cell| {
                cell.child(
                    gpui::div()
                        .absolute()
                        // The debug selector lets the headless tests read the
                        // dot's laid-out bounds.
                        .debug_selector(move || indicator_key)
                        // `.calendar__cell-indicator` hangs at `bottom-1` --
                        // 4px above the cell's lower edge, not the 2px a
                        // first read of the 3px dot suggests -- centred by
                        // the cell's flex alignment.
                        .bottom(px(4.))
                        // `.calendar__cell-indicator` is `size-[3px]` with a
                        // `rounded-[2px]` corner -- smaller than any radius
                        // token, so the literal is v3's own.
                        .size(px(3.))
                        .rounded(px(2.))
                        .bg(if is_sel { accent.foreground } else { marker }),
                )
            })
            .when(outside_month, |cell| cell.opacity(0.5))
            .into_any_element()
    }

    /// The seven column headers.
    /// `.calendar__grid-header` — the seven `.calendar__header-cell` columns.
    fn weekday_header(&self, cx: &App) -> gpui::Div {
        let muted = cx.colors().muted;
        gpui::div()
            .flex()
            .children(self.constraints.first_day_of_week.header_row().map(|d| {
                gpui::div()
                    .flex_1()
                    .text_center()
                    // `.calendar__header-cell` is `text-xs`.
                    .text_size(px(12.))
                    .text_color(muted)
                    .child(d.to_owned())
            }))
    }

    /// The heading over one visible month, named in the view calendar.
    ///
    /// `(year, month)` are the view calendar's. ICU formats an ISO date in
    /// whatever calendar the locale names, so the month is converted back to a
    /// Gregorian day and handed over with that locale: an Indian grid heads
    /// itself in Pausha 1947, not January 2026.
    fn month_heading_text(&self, year: i32, month: u32) -> String {
        let system = self.system();
        let (year, month) = system.add_months(year, month, self.year_heading_offset_months);
        let Some(date) = system.to_gregorian(year, month, 1) else {
            return String::new();
        };
        month_heading_for_locale(system.locale(), date.year, date.month)
            .unwrap_or_else(|| month_year_heading(date.year, date.month))
    }

    /// The number a cell prints: the day of the month in the *view* calendar.
    ///
    /// v3 hands `Calendar.Cell` a `formattedDate` in the calendar the grid is
    /// drawn in, so a date's Gregorian day is the wrong label whenever the two
    /// systems differ.
    fn day_label(&self, date: Date) -> String {
        self.system().from_gregorian(date).2.to_string()
    }

    /// One grid cell whose date the calendar system may not have.
    ///
    /// Every cell is addressed in the view calendar and converted back to a
    /// Gregorian [`Date`] here, so the rest of the component -- constraints,
    /// selection, callbacks -- only ever sees Gregorian. A triple the system
    /// does not have draws a blank rather than a wrong day.
    fn spill_cell(
        &self,
        date: Option<Date>,
        outside_month: bool,
        frame: &Frame<'_>,
        key: String,
        cx: &App,
    ) -> gpui::AnyElement {
        match date {
            Some(date) => self.day_cell(date, outside_month, frame, key, cx),
            None => gpui::div().size(px(36.)).into_any_element(),
        }
    }

    /// The 7-column grid for a single month, including both adjacent months.
    fn month_grid(&self, y: i32, m: u32, frame: &Frame<'_>, cx: &App) -> gpui::AnyElement {
        let system = self.system();
        let lead = self.constraints.lead_cells_in(system, y, m);
        let dim = system.days_in_month(y, m) as usize;
        let rows = self.constraints.rows_in(system, y, m);

        // `.calendar__grid` holds the header and `.calendar__grid-body`, whose
        // children are `.calendar__grid-row`s of cells.
        let mut grid = gpui::div().flex().flex_col().gap(px(2.));
        for r in 0..rows {
            let mut line = gpui::div().flex().gap(px(2.));
            for c in 0..7 {
                let idx = r * 7 + c;
                let slot: gpui::AnyElement = if idx < lead {
                    let (py, pm) = system.add_months(y, m, -1);
                    let day = system.days_in_month(py, pm) as usize - lead + idx + 1;
                    self.spill_cell(
                        system.to_gregorian(py, pm, day as u32),
                        true,
                        frame,
                        format!("{}-{y}-{m}-outside-{py}-{pm}-d{day}", frame.base),
                        cx,
                    )
                } else {
                    let day_num = idx - lead + 1;
                    if day_num > dim {
                        // The next month's leading days: v3 draws them as cells
                        // with `isOutsideMonth`, so the render prop sees them
                        // too -- muted and inert either way. Each spill cell
                        // carries its *own* next-month date: v3 hands the
                        // render prop a real `CalendarDate`, and the closure is
                        // the only identity a caller has.
                        let nd = day_num - dim;
                        let (ny, nm) = system.add_months(y, m, 1);
                        self.spill_cell(
                            system.to_gregorian(ny, nm, nd as u32),
                            true,
                            frame,
                            format!("{}-{y}-{m}-outside-{ny}-{nm}-d{nd}", frame.base),
                            cx,
                        )
                    } else {
                        self.spill_cell(
                            system.to_gregorian(y, m, day_num as u32),
                            false,
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
    fn year_grid(
        &self,
        view: calendar_view::YearGridView<'_>,
        year_focus: &gpui::FocusHandle,
        heading_focus: &gpui::FocusHandle,
        year_picker_own: Option<Entity<bool>>,
        window: &Window,
        cx: &App,
    ) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = colors.accent;
        let active_year = view.active_year;
        let base = view.base;
        // `.calendar-year-picker__year-grid` is `gap-1 p-1`.
        let mut grid = gpui::div().flex().flex_col().gap(px(4.)).p(px(4.));
        for chunk in view.years.chunks(3) {
            let mut row = gpui::div().flex().gap(px(4.));
            for &year in chunk {
                let is_active = year == active_year;
                let mut cell = gpui::div()
                    .id(gpui::ElementId::Name(format!("{base}-y{year}").into()))
                    .when(!self.is_disabled && is_active, |cell| {
                        cell.track_focus(year_focus)
                    })
                    .flex_1()
                    // `.calendar-year-picker__year-cell` is `h-8 px-2.5
                    // rounded-3xl text-sm`.
                    .h(px(32.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.))
                    .rounded(crate::util::control_radius(cx));

                if is_active {
                    cell = cell
                        .bg(accent.color)
                        .text_color(accent.foreground)
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else if !self.is_disabled {
                    // `.calendar-year-picker__year-cell:hover` fills
                    // `bg-default text-default-foreground`.
                    let hover_bg = colors.default.color;
                    let hover_fg = colors.default.foreground;
                    cell = cell
                        .text_color(colors.foreground)
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg).text_color(hover_fg));
                }

                if !self.is_disabled {
                    let st = self.state.clone();
                    let on_open = self.on_year_picker_open_change.clone();
                    let on_focus = self.on_focus_change.clone();
                    let own = year_picker_own.clone();
                    let back_to_trigger = heading_focus.clone();
                    cell = cell.on_click(move |_, window, cx| {
                        let next = st.update(cx, |s, cx| {
                            let day = s.view_day.max(1).min(days_in_month(year, s.view_month));
                            let next = Date::new(year, s.view_month, day);
                            s.set_anchor(next);
                            cx.notify();
                            next
                        });
                        if year != active_year {
                            if let Some(cb) = &on_focus {
                                cb(next, window, cx);
                            }
                        }
                        if let Some(held) = &own {
                            held.update(cx, |open, cx| {
                                *open = false;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_open {
                            cb(false, window, cx);
                        }
                        window.focus(&back_to_trigger);
                    });
                }

                if is_active {
                    cell = crate::util::ring_if_focused(
                        cell,
                        year_focus,
                        false,
                        Vec::new(),
                        window,
                        cx,
                    );
                }
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
        if let Some(values) = self.default_values.clone() {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("calendar-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.selected = values.last().copied();
                        s.selected_dates = values;
                        if let Some(value) = s.selected {
                            s.view_year = value.year;
                            s.view_month = value.month;
                            s.view_day = value.day;
                        }
                        cx.notify();
                    });
                },
            );
        } else if let Some(value) = self.default_value {
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
        let prev_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base}-prev-focus").into()),
            window,
            cx,
        );
        let next_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base}-next-focus").into()),
            window,
            cx,
        );
        let year_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base}-year-focus").into()),
            window,
            cx,
        );
        // Inside a picker the grid takes the focus as the panel opens, so the
        // arrows work without hunting for it with Tab.
        if self.autofocus_grid && !self.is_disabled && !year_picker_open {
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(format!("{base}-autofocus").into()),
                &grid_focus,
            );
        }
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-cursor").into()),
            cx,
            |_, _| None::<Date>,
        );
        let year_cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-year-cursor").into()),
            cx,
            |_, _| None::<i32>,
        );
        let year_was_open = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-year-was-open").into()),
            cx,
            |_, _| false,
        );
        let year_trigger_index = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-year-trigger-index").into()),
            cx,
            |_, _| 0usize,
        );
        let cursor_at = *cursor.read(cx);

        let first_day = self.constraints.first_day_of_week;
        let focused_value = self
            .focused_value
            .map(|date| self.constraints.constrain(date));

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
        let anchor = focused_value.map_or(anchor, |focused| {
            let (visible_start, visible_end) =
                calendar_view::visible_range(self.duration, first_day, anchor);
            calendar_view::anchor_following_focus(
                self.duration,
                first_day,
                anchor,
                visible_start,
                visible_end,
                focused,
            )
        });
        let initial_year = focused_value.unwrap_or(anchor).year;
        let years = calendar_view::year_window(
            initial_year,
            self.visible_years,
            self.constraints.min_value,
            self.constraints.max_value,
        );
        let first_year = years.first().copied().unwrap_or(anchor.year);
        let last_year = years.last().copied().unwrap_or(anchor.year);
        if year_picker_open && !*year_was_open.read(cx) && !self.is_disabled {
            year_cursor.update(cx, |year, _| *year = Some(initial_year));
            window.focus(&year_focus);
        }
        year_was_open.update(cx, |was_open, _| *was_open = year_picker_open);
        let active_year = focused_value
            .map(|date| date.year)
            .or(*year_cursor.read(cx))
            .unwrap_or(initial_year)
            .max(first_year)
            .min(last_year);
        // `focusedValue` wins; without it the keyboard's own cursor does, and it
        // starts from the selection or today.
        // Taking the focus puts the ring on the date v3 would have focused --
        // the selection, or today -- rather than waiting for a keystroke to
        // place it.
        let ring_at = focused_value
            .or(cursor_at)
            .or(selected)
            .or_else(|| Some(Date::today()))
            .filter(|_| grid_focus.is_focused(window));
        let frame = Frame {
            selected,
            selected_dates: &selected_dates,
            today: Date::today(),
            cursor: &cursor,
            base: &base,
            focused: ring_at,
        };

        let months = calendar_view::month_headings(self.duration, anchor);
        let linear = calendar_view::linear_cells(self.duration, first_day, anchor);
        let columns = months.len().max(1);
        let mut heading_focuses = Vec::with_capacity(columns);
        for index in 0..columns {
            heading_focuses.push(crate::util::tab_stop_handle(
                gpui::ElementId::Name(format!("{base}-heading-{index}-focus").into()),
                window,
                cx,
            ));
        }
        let active_heading_index = (*year_trigger_index.read(cx)).min(columns - 1);
        let active_heading_focus = heading_focuses[active_heading_index].clone();

        let colors = cx.colors();
        let layout = cx.layout();

        let nav_target = |dir: i32| {
            calendar_view::page_in(
                self.system(),
                self.duration,
                self.page_behavior,
                anchor,
                dir,
            )
        };
        let (visible_start, visible_end) =
            calendar_view::visible_range(self.duration, first_day, anchor);
        // React Stately checks only the day immediately outside the visible
        // range against minValue/maxValue. Unavailable dates do not block
        // paging, and readOnly prevents selection without preventing paging.
        let previous_disabled =
            self.is_disabled || self.constraints.out_of_range(add_days(&visible_start, -1));
        let next_disabled =
            self.is_disabled || self.constraints.out_of_range(add_days(&visible_end, 1));
        let state_for_nav = self.state.clone();
        // `Calendar.NavButton` children, defaulting to v3's chevrons.
        let (prev_icon, next_icon) = self
            .nav_icons
            .unwrap_or((icons::CHEVRON_LEFT, icons::CHEVRON_RIGHT));
        let nav_btn = |icon_path: &'static str,
                       target: Date,
                       key: String,
                       focus: &gpui::FocusHandle,
                       disabled: bool| {
            let state = state_for_nav.clone();
            // `.calendar__nav-button:hover` fills with `bg-default`.
            let hover_bg = colors.default.color;
            // The pinned `[data-pressed]` is a bare `scale(0.95)` with no
            // background change, so the hover fill must survive as its own
            // refinement and the press stays the backgroundless helper.
            let press = crate::anim::PressBox {
                height: px(24.),
                padding_x: None,
                width: Some(px(24.)),
                min_width: None,
                text_size: px(14.),
                line_height: px(20.),
                gap: px(0.),
                radius: crate::util::soft_radius(cx),
                shrink_x: true,
                scale: crate::anim::PRESSED_SCALE_DEEP,
            };
            let selector = key.clone();
            let button = gpui::div()
                .id(gpui::ElementId::Name(key.into()))
                .debug_selector(move || selector)
                .when(!disabled, |b| b.track_focus(focus))
                .flex()
                .items_center()
                .justify_center()
                // `.calendar__nav-button` is `size-6 rounded-2xl`.
                .size(px(24.))
                .rounded(crate::util::soft_radius(cx))
                .when(!disabled, |b| {
                    crate::anim::pressed(
                        b.cursor_pointer()
                            .hover(move |s| s.bg(hover_bg))
                            .on_click(move |_, _, cx| {
                                state.update(cx, |s, cx| {
                                    s.set_anchor(target);
                                    cx.notify();
                                });
                            }),
                        press,
                        cx,
                    )
                })
                .when(disabled, |b| b.opacity(layout.disabled_opacity));
            crate::util::ring_if_focused(button, focus, true, Vec::new(), window, cx).child(
                gpui::svg()
                        // `.calendar__nav-button-icon` is `size-4`.
                        .size(px(16.))
                        .path(icon_path)
                        // `.calendar__nav-button` is `text-accent-soft-foreground`,
                        // at rest and hovered alike.
                        .text_color(colors.accent.soft_foreground(colors.foreground)),
            )
        };

        // A heading is a plain label only when the picker is controlled without
        // a change handler; otherwise it is the built-in year-picker trigger.
        let heading = |text: String,
                       key: String,
                       focus: &gpui::FocusHandle,
                       index: usize|
         -> gpui::AnyElement {
            let label = gpui::div()
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(text);
            match &self.on_year_picker_open_change {
                // No handler and no state of our own: a plain label.
                None if year_picker_own.is_none() => label.into_any_element(),
                None => {
                    let own = year_picker_own.clone();
                    let opener = year_trigger_index.clone();
                    let open = year_picker_open;
                    let trigger = gpui::div()
                        .id(gpui::ElementId::Name(key.into()))
                        .when(!self.is_disabled, |trigger| trigger.track_focus(focus))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`
                        // and hovers nothing: only focus and the open state
                        // recolour it.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(crate::util::key_radius(cx))
                        .when(!self.is_disabled, |trigger| {
                            trigger
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    opener.update(cx, |value, _| *value = index);
                                    if let Some(held) = &own {
                                        held.update(cx, |value, cx| {
                                            *value = !open;
                                            cx.notify();
                                        });
                                    }
                                })
                        })
                        .when(self.is_disabled, |trigger| {
                            trigger.opacity(layout.disabled_opacity)
                        });
                    crate::util::ring_if_focused(trigger, focus, true, Vec::new(), window, cx)
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
                    let opener = year_trigger_index.clone();
                    let trigger = gpui::div()
                        .id(gpui::ElementId::Name(key.into()))
                        .when(!self.is_disabled, |trigger| trigger.track_focus(focus))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`
                        // and hovers nothing: only focus and the open state
                        // recolour it.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(crate::util::key_radius(cx))
                        .when(!self.is_disabled, |trigger| {
                            trigger
                                .cursor_pointer()
                                .on_click(move |_, window, cx| {
                                    opener.update(cx, |value, _| *value = index);
                                    // Uncontrolled: flip our own copy too, or
                                    // the callback would be the only opener.
                                    if let Some(held) = &own {
                                        held.update(cx, |value, cx| {
                                            *value = !open;
                                            cx.notify();
                                        });
                                    }
                                    cb(!open, window, cx);
                                })
                        })
                        .when(self.is_disabled, |trigger| {
                            trigger.opacity(layout.disabled_opacity)
                        });
                    crate::util::ring_if_focused(trigger, focus, true, Vec::new(), window, cx)
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
            // readOnly blocks selection, not focus or navigation.
            .when(!self.is_disabled && !year_picker_open, |el| {
                el.track_focus(&grid_focus)
            });

        // v3 drives a calendar from the keyboard: arrows step a day and a
        // week, Page Up/Down move the visible section, and Enter takes the
        // date the ring is on.
        if !self.is_disabled && !year_picker_open {
            let held = cursor.clone();
            let focus = grid_focus.clone();
            let prev_control = prev_focus.clone();
            let next_control = next_focus.clone();
            let heading_controls = heading_focuses.clone();
            let controlled_focus = focused_value;
            let start = focused_value.or(selected).unwrap_or_else(Date::today);
            let constraints = self.constraints.clone();
            let state = self.state.clone();
            let mode = self.selection_mode;
            let read_only = self.is_read_only;
            let on_change = self.on_change.clone();
            let on_change_all = self.on_change_all.clone();
            let on_focus = self.on_focus_change.clone();
            let duration = self.duration;
            let page_behavior = self.page_behavior;
            root = root.on_key_down(move |event, window, cx| {
                let header_focused = prev_control.is_focused(window)
                    || next_control.is_focused(window)
                    || heading_controls
                        .iter()
                        .any(|handle| handle.is_focused(window));
                if header_focused || !focus.contains_focused(window, cx) {
                    return;
                }
                let at = controlled_focus.or(*held.read(cx)).unwrap_or(start);
                let key = event.keystroke.key.as_str();
                let shift = event.keystroke.modifiers.shift;
                if matches!(key, "enter" | "space") {
                    if read_only || !constraints.allows(at) {
                        return;
                    }
                    let selected_dates = state.update(cx, |s, cx| {
                        s.toggle(at, mode);
                        cx.notify();
                        s.selected_dates.clone()
                    });
                    if let Some(cb) = &on_change {
                        cb(Some(at), window, cx);
                    }
                    if let Some(cb) = &on_change_all {
                        cb(&selected_dates, window, cx);
                    }
                    return;
                }
                let next = match key {
                    "left" => add_days(&at, -1),
                    "right" => add_days(&at, 1),
                    "up" => add_days(&at, -7),
                    "down" => add_days(&at, 7),
                    "pageup" => {
                        calendar_view::focus_section(duration, page_behavior, at, -1, shift)
                    }
                    "pagedown" => {
                        calendar_view::focus_section(duration, page_behavior, at, 1, shift)
                    }
                    "home" => calendar_view::section_start(duration, visible_start, at),
                    "end" => calendar_view::section_end(duration, visible_end, at),
                    _ => return,
                };
                if controlled_focus.is_none() {
                    held.update(cx, |v, cx| {
                        *v = Some(next);
                        cx.notify();
                    });
                    // React Aria keeps the focused date visible: the grid follows
                    // the cursor once it leaves the current visible range. Day
                    // views page the whole window directly.
                    state.update(cx, |s, cx| {
                        if matches!(key, "pageup" | "pagedown") {
                            let dir = if key == "pageup" { -1 } else { 1 };
                            let next_anchor = match duration {
                                VisibleDuration::Days(_) => calendar_view::focus_section(
                                    duration,
                                    page_behavior,
                                    anchor,
                                    dir,
                                    shift,
                                ),
                                _ if days_from_civil(&next) < days_from_civil(&visible_start) => {
                                    calendar_view::aligned_anchor(
                                        duration,
                                        SelectionAlignment::End,
                                        first_day,
                                        next,
                                    )
                                }
                                _ if days_from_civil(&next) > days_from_civil(&visible_end) => {
                                    calendar_view::aligned_anchor(
                                        duration,
                                        SelectionAlignment::Start,
                                        first_day,
                                        next,
                                    )
                                }
                                _ => anchor,
                            };
                            if next_anchor != anchor {
                                s.set_anchor(next_anchor);
                                cx.notify();
                            }
                        } else {
                            let next_anchor = calendar_view::anchor_following_focus(
                                duration,
                                first_day,
                                anchor,
                                visible_start,
                                visible_end,
                                next,
                            );
                            if next_anchor != anchor {
                                s.set_anchor(next_anchor);
                                cx.notify();
                            }
                        }
                    });
                }
                if let Some(cb) = &on_focus {
                    cb(next, window, cx);
                }
            });
        }

        if !self.is_disabled && year_picker_open {
            let held = year_cursor;
            let focus = year_focus.clone();
            let years_for_keys = years.clone();
            let own = year_picker_own.clone();
            let on_open = self.on_year_picker_open_change.clone();
            let on_focus = self.on_focus_change.clone();
            let controlled_year = focused_value.map(|date| date.year);
            let back_to_trigger = active_heading_focus.clone();
            root = root.on_key_down(move |event, window, cx| {
                if !focus.is_focused(window) {
                    return;
                }
                let key = event.keystroke.key.as_str();
                if key == "escape" {
                    if let Some(held) = &own {
                        held.update(cx, |open, cx| {
                            *open = false;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_open {
                        cb(false, window, cx);
                    }
                    window.focus(&back_to_trigger);
                    cx.stop_propagation();
                    return;
                }

                let current = controlled_year.or(*held.read(cx)).unwrap_or(active_year);
                let index = years_for_keys
                    .iter()
                    .position(|year| *year == current)
                    .unwrap_or(0);
                let next_index = match key {
                    "left" => index.checked_sub(1),
                    "right" => (index + 1 < years_for_keys.len()).then_some(index + 1),
                    "up" => index.checked_sub(3),
                    "down" => (index + 3 < years_for_keys.len()).then_some(index + 3),
                    "home" => Some(0),
                    "end" => years_for_keys.len().checked_sub(1),
                    _ => return,
                };
                if let Some(next_index) = next_index {
                    let next = years_for_keys[next_index];
                    if controlled_year.is_none() {
                        held.update(cx, |year, cx| {
                            *year = Some(next);
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_focus {
                        cb(
                            Date::new(
                                next,
                                anchor.month,
                                anchor.day.min(days_in_month(next, anchor.month)),
                            ),
                            window,
                            cx,
                        );
                    }
                }
                cx.stop_propagation();
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
                    // `.calendar__header` is `px-0.5`.
                    .px(px(2.))
                    .child(gpui::div().size(px(24.)))
                    .child(heading(
                        {
                            let (ay, am, _) = self.system().from_gregorian(anchor);
                            self.month_heading_text(ay, am)
                        },
                        format!("{base}-yheading"),
                        &active_heading_focus,
                        active_heading_index,
                    ))
                    .child(gpui::div().size(px(24.))),
            );
            root = root.child(self.year_grid(
                calendar_view::YearGridView {
                    years: &years,
                    active_year,
                    base: &base,
                },
                &year_focus,
                &active_heading_focus,
                year_picker_own.clone(),
                window,
                cx,
            ));
        } else if self.duration.is_month_view() {
            let mut row = gpui::div().flex().gap(px(20.));
            for (i, &(y, m)) in months.iter().enumerate() {
                let first = i == 0;
                let last = i + 1 == columns;
                let mut col = gpui::div().w(column_width).flex().flex_col().gap(px(8.));
                // Only the outer columns carry nav buttons; the others keep
                // a same-size spacer so every heading lines up.
                // The same box as a nav button, so every heading lines up.
                let spacer = || gpui::div().size(px(24.)).into_any_element();
                col = col.child(
                    gpui::div()
                        .flex()
                        .items_center()
                        .justify_between()
                        // `.calendar__header` is `px-0.5`.
                        .px(px(2.))
                        .child(if first {
                            nav_btn(
                                prev_icon,
                                nav_target(-1),
                                format!("{base}-prev"),
                                &prev_focus,
                                previous_disabled,
                            )
                                .into_any_element()
                        } else {
                            spacer()
                        })
                        .child(heading(
                            self.month_heading_text(y, m),
                            format!("{base}-heading{i}"),
                            &heading_focuses[i],
                            i,
                        ))
                        .child(if last {
                            nav_btn(
                                next_icon,
                                nav_target(1),
                                format!("{base}-next"),
                                &next_focus,
                                next_disabled,
                            )
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
                    // `.calendar__header` is `px-0.5`.
                    .px(px(2.))
                    .child(nav_btn(
                        icons::CHEVRON_LEFT,
                        nav_target(-1),
                        format!("{base}-prev"),
                        &prev_focus,
                        previous_disabled,
                    ))
                    .child(heading(
                        calendar_view::range_heading(&linear),
                        format!("{base}-heading"),
                        &heading_focuses[0],
                        0,
                    ))
                    .child(nav_btn(
                        icons::CHEVRON_RIGHT,
                        nav_target(1),
                        format!("{base}-next"),
                        &next_focus,
                        next_disabled,
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
                        false,
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

#[cfg(test)]
mod tests {

    #[test]
    fn a_locale_extension_moves_the_grid_into_that_calendar() {
        use crate::calendar_system::CalendarSystem;
        let system = CalendarSystem::for_locale("hi-IN-u-ca-indian").unwrap();

        // The anchor stays Gregorian -- it is the caller's state -- and the
        // grid is addressed in the Indian calendar it converts to.
        let anchor = Date::new(2026, 1, 15);
        let months = calendar_view::month_headings_in(&system, VisibleDuration::Months(1), anchor);
        assert_eq!(months, vec![(1947, 10)], "January 2026 is Pausha 1947");

        // Every cell of that month converts back to a real Gregorian date,
        // which is what selection and the constraints see.
        let days = system.days_in_month(1947, 10);
        assert_eq!(days, 30);
        for day in 1..=days {
            assert!(
                system.to_gregorian(1947, 10, day).is_some(),
                "Pausha {day} must be a date"
            );
        }
        assert_eq!(system.to_gregorian(1947, 10, 25), Some(anchor));
    }

    #[test]
    fn the_heading_names_the_view_calendars_month() {
        // ICU formats an ISO date in whatever calendar the locale names, so the
        // Indian heading is the Indian month and its Saka year.
        let heading = month_heading_for_locale("hi-IN-u-ca-indian", 2026, 1).unwrap();
        assert!(
            heading.contains("1947"),
            "an Indian heading names the Saka year, got {heading:?}"
        );
        assert_ne!(
            heading,
            month_heading_for_locale("en-US", 2026, 1).unwrap(),
            "the two calendars must not head the same month identically"
        );
    }

    #[test]
    fn month_names_follow_the_locale() {
        assert_eq!(month_names_for_locale("en-US").unwrap()[0], "January");
        assert_eq!(month_names_for_locale("de-DE").unwrap()[0], "Januar");
        assert_eq!(month_names_for_locale("fr-FR").unwrap()[0], "janvier");
    }

    #[test]
    fn month_abbreviations_come_from_cldr_not_a_byte_slice() {
        assert_eq!(month_abbrs_for_locale("en-US").unwrap()[0], "Jan");
        // The first three *bytes* of this name are one character, so the old
        // `[..3]` slice would have panicked rather than abbreviated.
        let japanese = month_abbrs_for_locale("ja-JP").unwrap();
        assert!(japanese[0].chars().count() < japanese[0].len());
    }

    #[test]
    fn the_heading_takes_its_field_order_from_the_locale() {
        let english = month_heading_for_locale("en-US", 2026, 1).unwrap();
        assert_eq!(english, "January 2026");
        assert_eq!(
            month_heading_for_locale("de-DE", 2026, 1).unwrap(),
            "Januar 2026"
        );
        // Japanese writes the year first, which a "{month} {year}" template
        // cannot reproduce whatever names it is given.
        let japanese = month_heading_for_locale("ja-JP", 2026, 1).unwrap();
        assert!(
            japanese.starts_with("2026"),
            "expected a year-first heading, got {japanese}"
        );
    }

    #[test]
    fn an_unknown_locale_reports_nothing_rather_than_guessing() {
        assert!(month_names_for_locale("not a locale").is_none());
        assert!(month_heading_for_locale("not a locale", 2026, 1).is_none());
    }
    use super::*;

    #[test]
    fn add_months_wraps_the_year_both_ways() {
        assert_eq!(add_months(2026, 8, 1), (2026, 9));
        assert_eq!(add_months(2026, 12, 1), (2027, 1));
        assert_eq!(add_months(2026, 1, -1), (2025, 12));
    }

    #[test]
    fn add_months_counts_rather_than_stepping() {
        // shift+Page Up is a *year*: `bump_month` moved one month whatever the
        // magnitude, so this is the difference the calendar depends on.
        assert_eq!(add_months(2026, 8, -12), (2025, 8));
        assert_eq!(add_months(2026, 8, 12), (2027, 8));
        assert_eq!(add_months(2026, 8, -20), (2024, 12));
    }

    // `Date::today` must be the local zone's civil date (v3
    // `today(getLocalTimeZone())`), not UTC's. `civil_date_at` is the
    // deterministic seam, so the crossings are proven with fixed instants and
    // offsets instead of the machine timezone. 2024-03-10T02:30:00Z is
    // 1710037800 and 2024-01-01T00:00:00Z is 1704067200.

    #[test]
    fn utc_offset_west_moves_today_to_the_previous_day() {
        // 2024-03-10T02:30 UTC is 2024-03-09T19:30 at UTC-07:00 ...
        assert_eq!(civil_date_at(1_710_037_800, -25_200), Date::new(2024, 3, 9));
        // ... and 2024-01-01T02:00 UTC is 2023-12-31T21:00 at UTC-05:00.
        assert_eq!(
            civil_date_at(1_704_074_400, -18_000),
            Date::new(2023, 12, 31)
        );
    }

    #[test]
    fn utc_offset_east_moves_today_to_the_next_day() {
        // 2024-01-01T15:00 UTC is 2024-01-02T00:00 at UTC+09:00 ...
        assert_eq!(civil_date_at(1_704_121_200, 32_400), Date::new(2024, 1, 2));
        // ... and 2023-12-31T22:00 UTC is 2024-01-01T01:00 at UTC+03:00.
        assert_eq!(civil_date_at(1_704_060_000, 10_800), Date::new(2024, 1, 1));
    }

    #[test]
    fn utc_offset_day_crossings_survive_leap_and_year_boundaries() {
        // 2024-02-28T20:00 UTC at UTC+05:00 lands on leap day 2024-02-29 ...
        assert_eq!(civil_date_at(1_709_150_400, 18_000), Date::new(2024, 2, 29));
        // ... and 2024-02-29T20:00 UTC at UTC+05:00 lands on 2024-03-01.
        assert_eq!(civil_date_at(1_709_236_800, 18_000), Date::new(2024, 3, 1));
    }

    #[test]
    fn fractional_offsets_resolve_midnight_aligned_local_dates() {
        // 2024-06-01T18:20 UTC is 2024-06-02T00:05 at Nepal's UTC+05:45.
        assert_eq!(civil_date_at(1_717_266_000, 20_700), Date::new(2024, 6, 2));
    }

    #[test]
    fn pre_epoch_instants_carry_the_crossing_too() {
        // 1969-12-31T23:00 UTC is 1970-01-01T01:00 at UTC+02:00.
        assert_eq!(civil_date_at(-3_600, 7_200), Date::new(1970, 1, 1));
    }

    #[test]
    fn today_reads_the_live_local_zone() {
        // Whatever zone this machine is in, `today` may only land on the UTC
        // date itself or on one of its neighbours; the old UTC-only derivation
        // and a hardcoded zone both break this bound.
        let now = jiff::Zoned::now();
        let utc_date = civil_date_at(now.timestamp().as_second(), 0);
        let diff = days_from_civil(&Date::today()) - days_from_civil(&utc_date);
        assert!(
            diff.abs() <= 1,
            "today {} vs UTC date {}",
            Date::today().format_iso(),
            utc_date.format_iso()
        );
    }

    // The pinned `.calendar__cell:hover` (not selected) fills with
    // `bg-default`, the full token -- the same fill as the pressed state.
    // `soft_hover()` is a lighter, wrong wash, so the check is mechanical.
    #[test]
    fn the_day_cell_hovers_the_full_default() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(
                "let hover_bg = colors.default.color;\n                let pressed_bg = colors.default.color;"
            ),
            "the day cell must hover the full `bg-default` \
             (pinned `.calendar__cell:hover:not([data-selected])`)"
        );
    }

    // The pinned `[data-today]` cell fills `bg-accent-soft` with
    // `text-accent-soft-foreground` and hovers `bg-accent-soft-hover`; the old
    // port invented an accent border instead.
    #[test]
    fn the_today_cell_uses_the_accent_soft_tokens() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(".bg(accent.soft())")
                && source.contains(".text_color(accent.soft_foreground(colors.foreground))"),
            "the today cell must fill `bg-accent-soft` with \
             `text-accent-soft-foreground` (pinned `.calendar__cell[data-today]`)"
        );
        assert!(
            source.contains("let hover_bg = accent.soft_hover();"),
            "the today cell must hover `bg-accent-soft-hover` \
             (pinned `.calendar__cell[data-today]:hover`)"
        );
        assert!(
            !source.contains("circle.border_1()"),
            "the today cell must not invent an accent border"
        );
    }

    // gpui's `active` refinement overwrites the previous one, so chaining
    // `.active` after `anim::pressed` would drop the 0.95 press scale. The
    // pressed background must merge with the geometry in one refinement,
    // which is what `pressed_with_background` exists for.
    #[test]
    fn the_day_cell_press_merges_background_with_the_scale() {
        // Scan the implementation only; this test's own text names the
        // forbidden chaining.
        let source = include_str!("calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert_eq!(
            source
                .matches("crate::anim::pressed_with_background(")
                .count(),
            3,
            "every pressable day branch must merge the pressed background \
             with the press geometry in one refinement"
        );
        assert!(
            !source.contains(".active("),
            "a chained `.active` after `anim::pressed` replaces the pressed \
             scale (gpui overwrites the active refinement)"
        );
    }

    // The pinned `.calendar-year-picker__trigger` has no hover rule at all:
    // only focus and the open state recolour it. A soft wash looks plausible
    // on screen, so the check is mechanical.
    #[test]
    fn the_year_picker_trigger_hovers_nothing() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            !source.contains("colors.default.soft_hover()"),
            "the year-picker trigger must not invent a hover background"
        );
    }

    // The pinned `.calendar__nav-button:active` is a bare `transform:
    // scale(0.95)` with no background change -- the hover fill stays a separate
    // refinement, so the nav button uses the backgroundless `anim::pressed`
    // while the day cells keep their `pressed_with_background` merges.
    #[test]
    fn the_nav_button_presses_the_deep_scale_over_its_hover() {
        // Scan the implementation only; this test's own text names the
        // forbidden chaining.
        let source = include_str!("calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert_eq!(
            source.matches("crate::anim::pressed(").count(),
            1,
            "the nav button must carry the backgroundless press (the pinned \
             `[data-pressed]` only scales), and the day cells keep \
             `pressed_with_background`"
        );
        assert!(
            source.contains("scale: crate::anim::PRESSED_SCALE_DEEP,"),
            "the nav button must press to v3's 0.95 scale"
        );
        assert_eq!(
            source.matches(".hover(move |s| s.bg(hover_bg))").count(),
            3,
            "the pressed refinement must not replace the nav button's \
             `bg-default` hover fill (pinned `.calendar__nav-button:hover`); \
             the two day-cell branches and the nav button share the spelling"
        );
    }
}
