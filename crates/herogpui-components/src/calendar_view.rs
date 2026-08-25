//! The v3 calendar view model: `visibleDuration`, `pageBehavior` and
//! `selectionAlignment`.
//!
//! v3 lets a calendar show a month grid, several month grids side by side, a
//! run of week rows, or a rolling window of days. All three views share one
//! anchor date; everything below is pure so the geometry can be tested without
//! a window.

use crate::calendar::{add_days, add_months, bump_month, days_in_month, month_name, Date};
use crate::date_constraints::Weekday;

/// `visibleDuration` — how much time one calendar shows at once.
///
/// React spells this `{months?: n, weeks?: n, days?: n}`; an enum makes the
/// three mutually exclusive forms unrepresentable in combination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisibleDuration {
    /// `{months: n}` — n month grids side by side.
    Months(usize),
    /// `{weeks: n}` — one grid of n week rows.
    Weeks(usize),
    /// `{days: n}` — a rolling window of n consecutive days.
    Days(usize),
}

impl Default for VisibleDuration {
    fn default() -> Self {
        VisibleDuration::Months(1)
    }
}

impl VisibleDuration {
    /// The unit count, never zero — a calendar always shows something.
    pub fn count(self) -> usize {
        match self {
            VisibleDuration::Months(n) | VisibleDuration::Weeks(n) | VisibleDuration::Days(n) => {
                n.max(1)
            }
        }
    }

    /// Month view keeps the classic 7-column grid with lead/trail cells; the
    /// week and day views are a flat run of dates instead.
    pub fn is_month_view(self) -> bool {
        matches!(self, VisibleDuration::Months(_))
    }
}

/// `pageBehavior` — whether navigation advances the whole visible range or a
/// single unit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageBehavior {
    /// `'visible'` — step by the visible duration.
    #[default]
    Visible,
    /// `'single'` — step one month/week/day at a time.
    Single,
}

/// `selectionAlignment` — where the selection sits inside the visible range on
/// first render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionAlignment {
    Start,
    #[default]
    Center,
    End,
}

impl SelectionAlignment {
    /// How many units before the selection the range starts, for a range of
    /// `count` units.
    pub fn lead_units(self, count: usize) -> usize {
        let count = count.max(1);
        match self {
            SelectionAlignment::Start => 0,
            // A 2-up range centres by leading with one unit, matching v3.
            SelectionAlignment::Center => (count - 1) / 2,
            SelectionAlignment::End => count - 1,
        }
    }
}

/// The first day of the week containing `date`, honouring `firstDayOfWeek`.
pub fn week_start(date: Date, first_day: Weekday) -> Date {
    // `first_weekday_pub` is Monday-indexed, so shift into the caller's frame
    // before taking the remainder.
    let dow = crate::calendar::weekday_index(date);
    let offset = (dow + 7 - first_day.monday_index()) % 7;
    add_days(&date, -(offset as i64))
}

/// Clamps `day` into `(year, month)` — paging from the 31st into a short month
/// must not produce an invalid date.
fn clamp_day(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day.min(days_in_month(year, month)))
}

/// The anchor after paging by `dir` (-1 back, +1 forward).
pub fn page(duration: VisibleDuration, behavior: PageBehavior, anchor: Date, dir: i32) -> Date {
    let step = match behavior {
        PageBehavior::Visible => duration.count(),
        PageBehavior::Single => 1,
    };
    match duration {
        VisibleDuration::Months(_) => {
            let (mut y, mut m) = (anchor.year, anchor.month);
            for _ in 0..step {
                let (ny, nm) = bump_month(y, m, dir);
                y = ny;
                m = nm;
            }
            clamp_day(y, m, anchor.day)
        }
        VisibleDuration::Weeks(_) => add_days(&anchor, dir as i64 * step as i64 * 7),
        VisibleDuration::Days(_) => add_days(&anchor, dir as i64 * step as i64),
    }
}

/// The focused section reached by PageUp/PageDown in pinned React Aria.
///
/// Month and week views move one displayed unit regardless of `pageBehavior`;
/// Shift moves their next larger unit. Day views delegate to page navigation,
/// so they honor `pageBehavior` and ignore Shift.
pub fn focus_section(
    duration: VisibleDuration,
    behavior: PageBehavior,
    anchor: Date,
    dir: i32,
    larger: bool,
) -> Date {
    match (duration, larger) {
        (VisibleDuration::Days(_), _) => page(duration, behavior, anchor, dir),
        (VisibleDuration::Weeks(_), false) => {
            page(VisibleDuration::Weeks(1), PageBehavior::Single, anchor, dir)
        }
        (VisibleDuration::Weeks(_), true) => page(
            VisibleDuration::Months(1),
            PageBehavior::Single,
            anchor,
            dir,
        ),
        (VisibleDuration::Months(_), false) => page(
            VisibleDuration::Months(1),
            PageBehavior::Single,
            anchor,
            dir,
        ),
        (VisibleDuration::Months(_), true) => page(
            VisibleDuration::Months(12),
            PageBehavior::Visible,
            anchor,
            dir,
        ),
    }
}

/// The anchor that puts `selection` at the requested position in the range.
///
/// v3 applies this on initial render only, so callers seed state with it
/// rather than recomputing every frame.
pub fn aligned_anchor(
    duration: VisibleDuration,
    alignment: SelectionAlignment,
    first_day: Weekday,
    selection: Date,
) -> Date {
    let lead = alignment.lead_units(duration.count());
    match duration {
        VisibleDuration::Months(_) => {
            let (mut y, mut m) = (selection.year, selection.month);
            for _ in 0..lead {
                let (ny, nm) = bump_month(y, m, -1);
                y = ny;
                m = nm;
            }
            clamp_day(y, m, selection.day)
        }
        VisibleDuration::Weeks(_) => {
            add_days(&week_start(selection, first_day), -(lead as i64) * 7)
        }
        VisibleDuration::Days(_) => add_days(&selection, -(lead as i64)),
    }
}

/// The `(year, month)` heading of each month grid in a month view.
pub fn month_headings(duration: VisibleDuration, anchor: Date) -> Vec<(i32, u32)> {
    if !duration.is_month_view() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(duration.count());
    let (mut y, mut m) = (anchor.year, anchor.month);
    for _ in 0..duration.count() {
        out.push((y, m));
        let (ny, nm) = bump_month(y, m, 1);
        y = ny;
        m = nm;
    }
    out
}

/// The consecutive dates a week or day view shows; empty for a month view,
/// which builds its cells from lead offsets instead.
pub fn linear_cells(duration: VisibleDuration, first_day: Weekday, anchor: Date) -> Vec<Date> {
    let n = duration.count();
    match duration {
        VisibleDuration::Months(_) => Vec::new(),
        VisibleDuration::Weeks(_) => {
            let start = week_start(anchor, first_day);
            (0..n as i64 * 7).map(|i| add_days(&start, i)).collect()
        }
        VisibleDuration::Days(_) => (0..n as i64).map(|i| add_days(&anchor, i)).collect(),
    }
}

/// The first and last dates in the visible range, excluding month-grid spill
/// cells. React Stately tests the immediately adjacent days to decide whether
/// the previous and next controls are disabled.
pub(crate) fn visible_range(
    duration: VisibleDuration,
    first_day: Weekday,
    anchor: Date,
) -> (Date, Date) {
    match duration {
        VisibleDuration::Months(_) => {
            let months = month_headings(duration, anchor);
            let (start_year, start_month) = months[0];
            let (end_year, end_month) = months[months.len() - 1];
            (
                Date::new(start_year, start_month, 1),
                Date::new(end_year, end_month, days_in_month(end_year, end_month)),
            )
        }
        VisibleDuration::Weeks(_) | VisibleDuration::Days(_) => {
            let cells = linear_cells(duration, first_day, anchor);
            (cells[0], cells[cells.len() - 1])
        }
    }
}

/// The heading for a week or day view, e.g. `Aug 3 – Aug 9, 2026`.
pub fn range_heading(cells: &[Date]) -> String {
    match (cells.first(), cells.last()) {
        (Some(a), Some(b)) if a == b => {
            format!(
                "{} {}, {}",
                crate::calendar::month_abbr(a.month),
                a.day,
                a.year
            )
        }
        (Some(a), Some(b)) if a.year == b.year => format!(
            "{} {} \u{2013} {} {}, {}",
            crate::calendar::month_abbr(a.month),
            a.day,
            crate::calendar::month_abbr(b.month),
            b.day,
            b.year
        ),
        (Some(a), Some(b)) => format!(
            "{} {}, {} \u{2013} {} {}, {}",
            crate::calendar::month_abbr(a.month),
            a.day,
            a.year,
            crate::calendar::month_abbr(b.month),
            b.day,
            b.year
        ),
        _ => String::new(),
    }
}

/// The sliding year-picker window around `view_year`.
///
/// HeroUI defaults to 20 visible years, except when both date bounds are set:
/// then it shows their full inclusive year span. An explicit `visibleYears`
/// wins, and the window stays inside either bound.
pub(crate) fn year_window(
    view_year: i32,
    visible_years: Option<usize>,
    min_value: Option<Date>,
    max_value: Option<Date>,
) -> Vec<i32> {
    let available = min_value.zip(max_value).map(|(min, max)| {
        let span = i64::from(max.year) - i64::from(min.year) + 1;
        usize::try_from(span.max(1)).unwrap_or(usize::MAX)
    });
    let requested = visible_years.or(available).unwrap_or(20).max(1);
    let count = requested.min(available.unwrap_or(requested));
    let count_i64 = i64::try_from(count).unwrap_or(i64::MAX);
    let mut start = i64::from(view_year) - count_i64 / 2;

    if let Some(min) = min_value {
        start = start.max(i64::from(min.year));
    }
    if let Some(max) = max_value {
        start = start.min(i64::from(max.year) - count_i64 + 1);
    }
    if let Some(min) = min_value {
        start = start.max(i64::from(min.year));
    }

    (0..count)
        .filter_map(|offset| {
            let year = start + i64::try_from(offset).ok()?;
            i32::try_from(year).ok()
        })
        .collect()
}

/// The shared render inputs for Calendar and RangeCalendar's year grid.
pub(crate) struct YearGridView<'a> {
    pub(crate) years: &'a [i32],
    pub(crate) active_year: i32,
    pub(crate) base: &'a str,
}

/// A year-picker trigger heading shifted from its calendar month's start.
/// React Aria's `offset={{months: n}}` adds the duration before formatting.
pub(crate) fn month_heading(year: i32, month: u32, offset_months: i32) -> String {
    let (year, month) = add_months(year, month, offset_months);
    format!("{} {year}", month_name(month))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> Date {
        Date::new(y, m, day)
    }

    #[test]
    fn visible_duration_never_zero() {
        assert_eq!(VisibleDuration::Months(0).count(), 1);
        assert_eq!(VisibleDuration::Weeks(0).count(), 1);
    }

    #[test]
    fn month_view_pages_by_visible_range() {
        let two = VisibleDuration::Months(2);
        // 'visible' advances the whole range; 'single' moves one month.
        assert_eq!(
            page(two, PageBehavior::Visible, d(2026, 8, 1), 1),
            d(2026, 10, 1)
        );
        assert_eq!(
            page(two, PageBehavior::Single, d(2026, 8, 1), 1),
            d(2026, 9, 1)
        );
    }

    #[test]
    fn month_paging_clamps_into_short_months() {
        // Jan 31 -> Feb must not yield Feb 31.
        let one = VisibleDuration::Months(1);
        assert_eq!(
            page(one, PageBehavior::Visible, d(2026, 1, 31), 1),
            d(2026, 2, 28)
        );
        assert_eq!(
            page(one, PageBehavior::Visible, d(2024, 1, 31), 1),
            d(2024, 2, 29)
        );
    }

    #[test]
    fn month_paging_crosses_the_year() {
        let one = VisibleDuration::Months(1);
        assert_eq!(
            page(one, PageBehavior::Visible, d(2026, 12, 5), 1),
            d(2027, 1, 5)
        );
        assert_eq!(
            page(one, PageBehavior::Visible, d(2026, 1, 5), -1),
            d(2025, 12, 5)
        );
    }

    #[test]
    fn week_and_day_views_page_in_days() {
        let w = VisibleDuration::Weeks(2);
        assert_eq!(
            page(w, PageBehavior::Visible, d(2026, 8, 3), 1),
            d(2026, 8, 17)
        );
        assert_eq!(
            page(w, PageBehavior::Single, d(2026, 8, 3), 1),
            d(2026, 8, 10)
        );
        let day = VisibleDuration::Days(3);
        assert_eq!(
            page(day, PageBehavior::Visible, d(2026, 8, 3), 1),
            d(2026, 8, 6)
        );
        assert_eq!(
            page(day, PageBehavior::Single, d(2026, 8, 3), -1),
            d(2026, 8, 2)
        );
    }

    #[test]
    fn focused_sections_follow_the_displayed_unit() {
        let at = d(2026, 8, 15);
        assert_eq!(
            focus_section(
                VisibleDuration::Weeks(2),
                PageBehavior::Visible,
                at,
                1,
                false
            ),
            d(2026, 8, 22)
        );
        assert_eq!(
            focus_section(
                VisibleDuration::Weeks(2),
                PageBehavior::Visible,
                at,
                1,
                true
            ),
            d(2026, 9, 15)
        );
        assert_eq!(
            focus_section(VisibleDuration::Days(3), PageBehavior::Visible, at, 1, true),
            d(2026, 8, 18)
        );
        assert_eq!(
            focus_section(
                VisibleDuration::Days(3),
                PageBehavior::Single,
                at,
                -1,
                false
            ),
            d(2026, 8, 14)
        );
    }

    #[test]
    fn week_start_honours_first_day_of_week() {
        // 2026-08-22 is a Saturday.
        assert_eq!(week_start(d(2026, 8, 22), Weekday::Mon), d(2026, 8, 17));
        assert_eq!(week_start(d(2026, 8, 22), Weekday::Sun), d(2026, 8, 16));
        assert_eq!(week_start(d(2026, 8, 22), Weekday::Sat), d(2026, 8, 22));
    }

    #[test]
    fn alignment_places_the_selection() {
        let two = VisibleDuration::Months(2);
        // React Aria's alignCenter halves the duration then backs off again on
        // an even count, so a 2-up range leads by nothing and Center == Start.
        assert_eq!(
            aligned_anchor(
                two,
                SelectionAlignment::Center,
                Weekday::Mon,
                d(2026, 8, 10)
            ),
            d(2026, 8, 10)
        );
        assert_eq!(
            aligned_anchor(two, SelectionAlignment::Start, Weekday::Mon, d(2026, 8, 10)),
            d(2026, 8, 10)
        );
        // A 3-up range does centre exactly.
        assert_eq!(
            aligned_anchor(
                VisibleDuration::Months(3),
                SelectionAlignment::Center,
                Weekday::Mon,
                d(2026, 8, 10)
            ),
            d(2026, 7, 10)
        );
        assert_eq!(
            aligned_anchor(two, SelectionAlignment::End, Weekday::Mon, d(2026, 8, 10)),
            d(2026, 7, 10)
        );
        let three = VisibleDuration::Months(3);
        assert_eq!(
            aligned_anchor(three, SelectionAlignment::End, Weekday::Mon, d(2026, 8, 10)),
            d(2026, 6, 10)
        );
    }

    #[test]
    fn month_headings_walk_forward_from_the_anchor() {
        assert_eq!(
            month_headings(VisibleDuration::Months(3), d(2026, 11, 1)),
            vec![(2026, 11), (2026, 12), (2027, 1)]
        );
        assert!(month_headings(VisibleDuration::Weeks(2), d(2026, 8, 1)).is_empty());
    }

    #[test]
    fn linear_cells_are_contiguous() {
        let cells = linear_cells(VisibleDuration::Weeks(2), Weekday::Mon, d(2026, 8, 22));
        assert_eq!(cells.len(), 14);
        assert_eq!(cells[0], d(2026, 8, 17));
        assert_eq!(cells[13], d(2026, 8, 30));
        let days = linear_cells(VisibleDuration::Days(3), Weekday::Mon, d(2026, 8, 30));
        assert_eq!(days, vec![d(2026, 8, 30), d(2026, 8, 31), d(2026, 9, 1)]);
    }

    #[test]
    fn visible_range_excludes_month_spill_and_spans_linear_views() {
        assert_eq!(
            visible_range(VisibleDuration::Months(2), Weekday::Mon, d(2026, 8, 15)),
            (d(2026, 8, 1), d(2026, 9, 30))
        );
        assert_eq!(
            visible_range(VisibleDuration::Weeks(1), Weekday::Mon, d(2026, 8, 22)),
            (d(2026, 8, 17), d(2026, 8, 23))
        );
        assert_eq!(
            visible_range(VisibleDuration::Days(3), Weekday::Mon, d(2026, 8, 30)),
            (d(2026, 8, 30), d(2026, 9, 1))
        );
    }

    #[test]
    fn range_heading_collapses_and_spans() {
        let one = linear_cells(VisibleDuration::Days(1), Weekday::Mon, d(2026, 8, 3));
        assert_eq!(range_heading(&one), "Aug 3, 2026");
        let week = linear_cells(VisibleDuration::Weeks(1), Weekday::Mon, d(2026, 8, 5));
        assert_eq!(range_heading(&week), "Aug 3 \u{2013} Aug 9, 2026");
        let span = linear_cells(VisibleDuration::Days(3), Weekday::Mon, d(2026, 12, 31));
        assert_eq!(span, vec![d(2026, 12, 31), d(2027, 1, 1), d(2027, 1, 2)]);
        assert_eq!(range_heading(&span), "Dec 31, 2026 \u{2013} Jan 2, 2027");
    }

    #[test]
    fn year_window_centers_and_clamps_to_bounds() {
        let years = year_window(2026, None, None, None);
        assert_eq!(years.len(), 20);
        assert_eq!(years[10], 2026);

        assert_eq!(
            year_window(2026, None, Some(d(2024, 6, 1)), Some(d(2028, 6, 1))),
            vec![2024, 2025, 2026, 2027, 2028]
        );
        assert_eq!(
            year_window(2026, Some(3), Some(d(2024, 6, 1)), Some(d(2028, 6, 1))),
            vec![2025, 2026, 2027]
        );
        assert_eq!(
            year_window(2024, Some(3), Some(d(2024, 6, 1)), None),
            vec![2024, 2025, 2026]
        );
        assert_eq!(
            year_window(2028, Some(3), None, Some(d(2028, 6, 1))),
            vec![2026, 2027, 2028]
        );
    }

    #[test]
    fn month_heading_offset_crosses_the_year() {
        assert_eq!(month_heading(2026, 8, 0), "August 2026");
        assert_eq!(month_heading(2026, 12, 2), "February 2027");
        assert_eq!(month_heading(2026, 1, -2), "November 2025");
    }
}
