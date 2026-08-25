//! The date constraints v3 puts on every date component.
//!
//! `Calendar`, `RangeCalendar`, `DateField`, `DatePicker` and
//! `DateRangePicker` all accept `minValue`, `maxValue`, `isDateUnavailable`,
//! `firstDayOfWeek` and `weeksInMonth`. Modelling them once keeps the five
//! components from drifting apart.

use std::sync::Arc;

use crate::calendar::{days_from_civil, days_in_month, first_weekday_pub, Date};

/// The first column of a month grid (`firstDayOfWeek`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Weekday {
    Sun,
    #[default]
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
}

impl Weekday {
    /// In Monday-first order, matching [`first_weekday_pub`].
    pub const ALL: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    /// Index with Monday as 0.
    pub fn monday_index(self) -> usize {
        match self {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Weekday::Sun => "Su",
            Weekday::Mon => "Mo",
            Weekday::Tue => "Tu",
            Weekday::Wed => "We",
            Weekday::Thu => "Th",
            Weekday::Fri => "Fr",
            Weekday::Sat => "Sa",
        }
    }

    /// The seven column headers, starting from this day.
    pub fn header_row(self) -> [&'static str; 7] {
        let start = self.monday_index();
        let mut out = [""; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = Weekday::ALL[(start + i) % 7].short_label();
        }
        out
    }
}

/// Predicate marking individual dates unavailable (`isDateUnavailable`).
pub type DateUnavailable = Arc<dyn Fn(Date) -> bool + 'static>;

/// Shared date constraints.
#[derive(Clone, Default)]
pub struct DateConstraints {
    pub min_value: Option<Date>,
    pub max_value: Option<Date>,
    pub is_date_unavailable: Option<DateUnavailable>,
    pub first_day_of_week: Weekday,
    /// `weeksInMonth` — forces the grid to this many rows.
    pub weeks_in_month: Option<usize>,
}

impl DateConstraints {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether `date` may be selected.
    pub fn allows(&self, date: Date) -> bool {
        if self.out_of_range(date) {
            return false;
        }
        match &self.is_date_unavailable {
            Some(f) => !f(date),
            None => true,
        }
    }

    /// Whether `date` falls outside `[min_value, max_value]`, ignoring the
    /// unavailable predicate. Out-of-range days are muted while unavailable
    /// days are struck through, so callers need to tell them apart.
    pub fn out_of_range(&self, date: Date) -> bool {
        let day = days_from_civil(&date);
        if let Some(min) = self.min_value {
            if day < days_from_civil(&min) {
                return true;
            }
        }
        if let Some(max) = self.max_value {
            if day > days_from_civil(&max) {
                return true;
            }
        }
        false
    }

    /// Whether `date` is blocked only by the unavailable predicate.
    pub fn is_unavailable(&self, date: Date) -> bool {
        match &self.is_date_unavailable {
            Some(f) => f(date),
            None => false,
        }
    }

    /// Blank leading cells before the 1st, honouring `first_day_of_week`.
    pub fn lead_cells(&self, year: i32, month: u32) -> usize {
        let first = first_weekday_pub(year, month);
        (first + 7 - self.first_day_of_week.monday_index()) % 7
    }

    /// Rows the grid should render for this month.
    pub fn rows(&self, year: i32, month: u32) -> usize {
        if let Some(rows) = self.weeks_in_month.filter(|rows| *rows > 0) {
            return rows;
        }
        let cells = self.lead_cells(year, month) + days_in_month(year, month) as usize;
        cells.div_ceil(7)
    }
}

impl std::fmt::Debug for DateConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DateConstraints")
            .field("min_value", &self.min_value)
            .field("max_value", &self.max_value)
            .field(
                "is_date_unavailable",
                &self.is_date_unavailable.as_ref().map(|_| "<fn>"),
            )
            .field("first_day_of_week", &self.first_day_of_week)
            .field("weeks_in_month", &self.weeks_in_month)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> Date {
        Date::new(y, m, day)
    }

    #[test]
    fn unconstrained_allows_everything() {
        let c = DateConstraints::new();
        assert!(c.allows(d(1900, 1, 1)));
        assert!(c.allows(d(2099, 12, 31)));
    }

    #[test]
    fn min_and_max_are_inclusive() {
        let c = DateConstraints {
            min_value: Some(d(2026, 3, 10)),
            max_value: Some(d(2026, 3, 20)),
            ..Default::default()
        };
        assert!(!c.allows(d(2026, 3, 9)));
        assert!(c.allows(d(2026, 3, 10)));
        assert!(c.allows(d(2026, 3, 20)));
        assert!(!c.allows(d(2026, 3, 21)));
    }

    #[test]
    fn range_and_unavailable_are_distinguishable() {
        let c = DateConstraints {
            min_value: Some(d(2026, 3, 10)),
            is_date_unavailable: Some(Arc::new(|date: Date| date.day == 15)),
            ..Default::default()
        };
        // Blocked by the predicate, not the range.
        assert!(!c.allows(d(2026, 3, 15)));
        assert!(!c.out_of_range(d(2026, 3, 15)));
        assert!(c.is_unavailable(d(2026, 3, 15)));
        // Blocked by the range, not the predicate.
        assert!(c.out_of_range(d(2026, 3, 1)));
        assert!(!c.is_unavailable(d(2026, 3, 1)));
    }

    #[test]
    fn first_day_of_week_shifts_the_lead() {
        // 2026-03-01 is a Sunday: Monday-start needs six blanks, Sunday none.
        let monday = DateConstraints::new();
        assert_eq!(monday.lead_cells(2026, 3), 6);

        let sunday = DateConstraints {
            first_day_of_week: Weekday::Sun,
            ..Default::default()
        };
        assert_eq!(sunday.lead_cells(2026, 3), 0);
    }

    #[test]
    fn header_row_starts_on_the_configured_day() {
        assert_eq!(
            Weekday::Mon.header_row(),
            ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        );
        assert_eq!(
            Weekday::Sun.header_row(),
            ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
        );
        assert_eq!(
            Weekday::Sat.header_row(),
            ["Sa", "Su", "Mo", "Tu", "We", "Th", "Fr"]
        );
    }

    #[test]
    fn rows_cover_every_day() {
        let c = DateConstraints::new();
        for (y, m) in [(2026, 2), (2026, 3), (2026, 8), (2024, 2), (2027, 5)] {
            let rows = c.rows(y, m);
            let needed = c.lead_cells(y, m) + days_in_month(y, m) as usize;
            assert!(
                rows * 7 >= needed,
                "{y}-{m}: {rows} rows for {needed} cells"
            );
        }
    }

    #[test]
    fn weeks_in_month_overrides_without_clamping() {
        let forced = DateConstraints {
            weeks_in_month: Some(6),
            ..Default::default()
        };
        assert_eq!(forced.rows(2026, 2), 6);

        let extended = DateConstraints {
            weeks_in_month: Some(7),
            ..Default::default()
        };
        assert_eq!(extended.rows(2026, 2), 7);

        let locale_default = DateConstraints {
            weeks_in_month: Some(0),
            ..Default::default()
        };
        assert_eq!(locale_default.rows(2026, 2), 5);
    }
}
