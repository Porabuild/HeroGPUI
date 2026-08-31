//! RangeCalendar — port of `@heroui/range-calendar` (v3).
//!
//! A month grid for selecting a start and end date. Selection is a two-step
//! anchor/extend interaction; the days between the anchor and the hovered day
//! preview as part of the range.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, ElementId, Entity, InteractiveElement, IntoElement, RenderOnce,
    Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::{
    calendar::{add_days, add_months, days_from_civil, days_in_month, weekday_index, Date},
    calendar_view::{self, PageBehavior, SelectionAlignment, VisibleDuration},
    date_constraints::{DateConstraints, Weekday},
    date_picker::DateRangeState,
    icons, util,
};

type OnRangeChange = Arc<dyn Fn(Date, Date, &mut Window, &mut App) + 'static>;
type RangeDateUnavailable = Arc<dyn Fn(Date, Option<Date>) -> bool + 'static>;

/// HeroUI RangeCalendar.
#[derive(IntoElement)]
pub struct RangeCalendar {
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<(Date, Date)>,
    id: ElementId,
    state: Entity<DateRangeState>,
    constraints: DateConstraints,
    range_date_unavailable: Option<RangeDateUnavailable>,
    is_disabled: bool,
    is_read_only: bool,
    /// Set by a picker: take the focus as the panel opens. See
    /// [`RangeCalendar::autofocus_grid`].
    autofocus_grid: bool,
    is_invalid: bool,
    focused_value: Option<Date>,
    duration: VisibleDuration,
    page_behavior: PageBehavior,
    selection_alignment: Option<SelectionAlignment>,
    /// `isYearPickerOpen` — `None` leaves the component holding the state,
    /// seeded from `defaultYearPickerOpen`.
    year_picker_open: Option<bool>,
    default_year_picker_open: bool,
    /// `RangeCalendar.YearPickerGrid.visibleYears` — `None` uses the v3
    /// min/max span or 20-year default.
    visible_years: Option<usize>,
    /// `RangeCalendar.YearPickerTriggerHeading.offset.months`.
    year_heading_offset_months: i32,
    on_year_picker_open_change: Option<Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_focus_change: Option<Arc<dyn Fn(Date, &mut Window, &mut App) + 'static>>,
    /// `allowsNonContiguousRanges` — lets a range span unavailable dates.
    allows_non_contiguous_ranges: bool,
    /// `RangeCalendar.CellIndicator` — the dot under a marked day, the same part
    /// a [`Calendar`](crate::calendar::Calendar) draws.
    cell_indicator: Option<Box<dyn Fn(Date) -> bool + 'static>>,
    /// `RangeCalendar.Cell`'s render props: the closure replaces the day label
    /// and is handed the state v3 passes it, the two range ends included.
    cell: Option<Box<dyn Fn(RangeCalendarCellState) -> gpui::AnyElement + 'static>>,
    on_change: Option<OnRangeChange>,
}

/// What `RangeCalendar.Cell`'s render function is handed -- v3's render props
/// for the cell, one field each.
#[derive(Clone, Debug)]
pub struct RangeCalendarCellState {
    /// The date this cell draws.
    pub date: Date,
    /// `formattedDate` — the day label, as this port writes it.
    pub formatted_date: gpui::SharedString,
    /// `isSelected` — an end of the range or inside it.
    pub is_selected: bool,
    /// `isSelectionStart`
    pub is_selection_start: bool,
    /// `isSelectionEnd`
    pub is_selection_end: bool,
    /// `isUnavailable`
    pub is_unavailable: bool,
    /// `isOutsideMonth`
    pub is_outside_month: bool,
    /// Today, which v3 marks with `data-today`.
    pub is_today: bool,
    /// Outside the min/max range, or the calendar is disabled.
    pub is_disabled: bool,
}

impl RangeCalendar {
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
        self.on_focus_change = Some(Arc::new(handler));
        self
    }

    /// `value` — writes the range through to the bound state.
    pub fn value(self, start: Option<Date>, end: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| {
            s.start = start;
            s.end = end;
        });
        self
    }

    pub fn new(state: Entity<DateRangeState>) -> Self {
        Self {
            default_value: None,
            id: ElementId::Name(format!("range-cal-{}", state.entity_id().as_u64()).into()),
            state,
            constraints: DateConstraints::new().with_hero_calendar_bounds(),
            range_date_unavailable: None,
            is_disabled: false,
            is_read_only: false,
            autofocus_grid: false,
            is_invalid: false,
            focused_value: None,
            duration: VisibleDuration::default(),
            page_behavior: PageBehavior::default(),
            selection_alignment: None,
            year_picker_open: None,
            default_year_picker_open: false,
            visible_years: None,
            year_heading_offset_months: 0,
            on_year_picker_open_change: None,
            on_focus_change: None,
            allows_non_contiguous_ranges: false,
            cell_indicator: None,
            cell: None,
            on_change: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial range.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: (Date, Date)) -> Self {
        self.default_value = Some(value);
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

    /// `selectionAlignment` — where the range start sits inside the visible
    /// range, until the user navigates for themselves.
    pub fn selection_alignment(mut self, alignment: SelectionAlignment) -> Self {
        self.selection_alignment = Some(alignment);
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

    /// `RangeCalendar.YearPickerGrid.visibleYears` — the size of its sliding
    /// window.
    pub fn visible_years(mut self, count: usize) -> Self {
        self.visible_years = Some(count.max(1));
        self
    }

    /// `RangeCalendar.YearPickerTriggerHeading.offset` — shifts its displayed
    /// month.
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
        self.on_year_picker_open_change = Some(Arc::new(f));
        self
    }

    pub fn min_value(mut self, date: Date) -> Self {
        self.constraints.min_value = Some(date);
        self
    }

    pub fn max_value(mut self, date: Date) -> Self {
        self.constraints.max_value = Some(date);
        self
    }

    /// `isDateUnavailable` — blocks a date relative to the active range anchor.
    /// The anchor is `None` until the first endpoint is selected and after the
    /// range is complete.
    pub fn is_date_unavailable(mut self, f: impl Fn(Date, Option<Date>) -> bool + 'static) -> Self {
        self.constraints.is_date_unavailable = None;
        self.range_date_unavailable = Some(Arc::new(f));
        self
    }

    /// `firstDayOfWeek`
    /// `RangeCalendar.Cell`'s render function — draw the day yourself.
    ///
    /// v3 hands it `{formattedDate, isSelected, isUnavailable, isOutsideMonth,
    /// isSelectionStart, isSelectionEnd}`; this port computes every one of those
    /// to draw the cell, so the closure is handed the same state.
    pub fn cell(
        mut self,
        render: impl Fn(RangeCalendarCellState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.cell = Some(Box::new(render));
        self
    }

    /// `RangeCalendar.CellIndicator` — mark the days this returns `true` for.
    pub fn cell_indicator(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.cell_indicator = Some(Box::new(f));
        self
    }

    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.constraints.first_day_of_week = day;
        self
    }

    /// `weeksInMonth`
    pub fn weeks_in_month(mut self, rows: usize) -> Self {
        self.constraints.weeks_in_month = Some(rows);
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `allowsNonContiguousRanges` — permits a selection that spans dates the
    /// unavailable predicate rejects.
    pub fn allows_non_contiguous_ranges(mut self, v: bool) -> Self {
        self.allows_non_contiguous_ranges = v;
        self
    }

    /// All the date constraints at once.
    pub fn constraints(mut self, constraints: DateConstraints) -> Self {
        self.constraints = constraints.with_hero_calendar_bounds();
        self.range_date_unavailable = None;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// Takes the focus the first time the grid renders; see
    /// [`crate::calendar::Calendar::autofocus_grid`]. Crate-only for the same
    /// reason: a standalone calendar must not steal the focus.
    pub(crate) fn autofocus_grid(mut self, v: bool) -> Self {
        self.autofocus_grid = v;
        self
    }

    /// `onChange` — called with `(start, end)` once both endpoints complete a
    /// range. The first pick remains the internal anchor, matching React
    /// Stately's `useRangeCalendarState`.
    pub fn on_change(
        mut self,
        handler: impl Fn(Date, Date, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

/// Resolves one range selection through the same constraint path for clicks
/// and keyboard activation. While selecting the second endpoint, min/max clamp
/// the requested date first; unless non-contiguous ranges are enabled, the
/// first unavailable day then clamps it toward the anchor.
fn resolve_pick(
    start: Option<Date>,
    end: Option<Date>,
    requested: Date,
    constraints: &DateConstraints,
    range_date_unavailable: Option<&RangeDateUnavailable>,
    allows_non_contiguous_ranges: bool,
) -> Option<(Date, Option<Date>)> {
    if start.is_none() || end.is_some() {
        return range_allows(constraints, range_date_unavailable, requested, None)
            .then_some((requested, None));
    }

    let anchor = start?;
    if range_is_unavailable(constraints, range_date_unavailable, requested, Some(anchor)) {
        return None;
    }
    let mut target = requested;
    if let Some(min) = constraints.min_value {
        if days_from_civil(&target) < days_from_civil(&min) {
            target = min;
        }
    }
    if let Some(max) = constraints.max_value {
        if days_from_civil(&target) > days_from_civil(&max) {
            target = max;
        }
    }

    if allows_non_contiguous_ranges {
        if range_is_unavailable(constraints, range_date_unavailable, target, Some(anchor)) {
            return None;
        }
    } else {
        let direction = days_from_civil(&target).cmp(&days_from_civil(&anchor));
        let step = match direction {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
        let mut resolved = anchor;
        while resolved != target {
            let next = add_days(&resolved, step);
            if range_is_unavailable(constraints, range_date_unavailable, next, Some(anchor)) {
                break;
            }
            resolved = next;
        }
        target = resolved;
    }

    if days_from_civil(&target) < days_from_civil(&anchor) {
        Some((target, Some(anchor)))
    } else {
        Some((anchor, Some(target)))
    }
}

fn range_is_unavailable(
    constraints: &DateConstraints,
    range_date_unavailable: Option<&RangeDateUnavailable>,
    date: Date,
    anchor: Option<Date>,
) -> bool {
    match range_date_unavailable {
        Some(predicate) => predicate(date, anchor),
        None => constraints.is_unavailable(date),
    }
}

fn range_allows(
    constraints: &DateConstraints,
    range_date_unavailable: Option<&RangeDateUnavailable>,
    date: Date,
    anchor: Option<Date>,
) -> bool {
    !constraints.out_of_range(date)
        && !range_is_unavailable(constraints, range_date_unavailable, date, anchor)
}

/// Where pinned React Stately moves focus after the first keyboard endpoint:
/// prefer tomorrow, fall back to yesterday, and stay put if neither is valid.
fn keyboard_range_focus(
    anchor: Date,
    constraints: &DateConstraints,
    range_date_unavailable: Option<&RangeDateUnavailable>,
    allows_non_contiguous_ranges: bool,
) -> Option<Date> {
    [add_days(&anchor, 1), add_days(&anchor, -1)]
        .into_iter()
        .find(|date| {
            !constraints.out_of_range(*date)
                && (allows_non_contiguous_ranges
                    || !range_is_unavailable(
                        constraints,
                        range_date_unavailable,
                        *date,
                        Some(anchor),
                    ))
        })
}

/// The per-frame facts every cell needs, bundled so the helpers below take a
/// readable argument list.
struct Frame<'a> {
    start: Option<Date>,
    preview_end: Option<Date>,
    unavailable_anchor: Option<Date>,
    today: Date,
    cursor: &'a Entity<Option<Date>>,
    focus_preview: &'a Entity<bool>,
    selection_before_anchor: &'a Entity<Option<(Date, Date)>>,
    base: &'a str,
    /// The date wearing the focus ring: `focusedValue` when the caller controls
    /// it, otherwise wherever the arrow keys have walked to. `None` while the
    /// grid does not hold the keyboard.
    focused: Option<Date>,
}

/// Where a cell sits in its row: the row's first and last columns drive the
/// pinned row-boundary rounding of the range track.
struct CellSlot {
    column: usize,
    columns: usize,
}

impl CellSlot {
    fn is_first(&self) -> bool {
        self.column == 0
    }

    fn is_last(&self) -> bool {
        self.column + 1 == self.columns
    }
}

impl RangeCalendar {
    /// One day cell. The range track lives on the outer cell and runs under
    /// the caps too; the inner button is the circle that takes the press.
    fn range_cell(
        &self,
        date: Date,
        outside_month: bool,
        frame: &Frame<'_>,
        key: String,
        slot: CellSlot,
        cx: &App,
    ) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = if self.is_invalid {
            colors.danger
        } else {
            colors.accent
        };
        let serial = days_from_civil(&date);
        let start_day = frame.start.map(|d| days_from_civil(&d));
        let end_day = frame.preview_end.map(|d| days_from_civil(&d));
        let unavailable = range_is_unavailable(
            &self.constraints,
            self.range_date_unavailable.as_ref(),
            date,
            frame.unavailable_anchor,
        );
        let disabled = outside_month || self.is_disabled || self.constraints.out_of_range(date);
        let eligible = !disabled && !unavailable;
        let selectable = eligible && !self.is_read_only;

        let is_start = start_day == Some(serial);
        let is_end = end_day == Some(serial);
        let in_range = eligible
            && match (start_day, end_day) {
                (Some(a), Some(b)) => serial > a.min(b) && serial < a.max(b),
                _ => false,
            };
        let is_selected = eligible && (is_start || is_end || in_range);
        let draw_start = is_start && !outside_month;
        let draw_end = is_end && !outside_month;
        let is_today = serial == days_from_civil(&frame.today);

        // The pinned anatomy splits the cell in two. The outer
        // `.range-calendar__cell` carries the range track -- the middle
        // segment's `rounded-none bg-accent-soft`, painted unscaled so the
        // run stays continuous across neighbours -- and the inner
        // `.range-calendar__cell-button`, the circle holding the day glyph,
        // is what takes the 0.9 press scale.
        let track_key = format!("{key}-track");
        let indicator_key = format!("{key}-indicator");
        let mut track = div()
            .relative()
            // The debug selector lets the headless tests read the outer
            // cell's laid-out bounds separately from the pressed button.
            .debug_selector(move || track_key)
            .flex()
            .items_center()
            .justify_center()
            .size(px(36.));
        if is_selected {
            // `[data-selected]:not([data-outside-month])` is
            // `rounded-none bg-accent-soft` -- on the caps too, whose solid
            // button paints over it. The track carries no text colour: the
            // inner button keeps the base `text-foreground` unless
            // `data-today` recolours it. Row ends round off with `lg` so a
            // run crossing a week boundary reads as one shape, and a cap
            // rounds its own side with `3xl`
            // (`rounded-ss-3xl rounded-es-3xl` / `rounded-se-3xl rounded-ee-3xl`).
            let cap_radius = util::control_radius(cx);
            let edge_radius = util::key_radius(cx);
            let left = if draw_start {
                cap_radius
            } else if slot.is_first() {
                edge_radius
            } else {
                px(0.)
            };
            let right = if draw_end {
                cap_radius
            } else if slot.is_last() {
                edge_radius
            } else {
                px(0.)
            };
            track = track.bg(accent.soft());
            track = track
                .rounded_tl(left)
                .rounded_bl(left)
                .rounded_tr(right)
                .rounded_br(right);
        }

        let mut cell = div()
            .id(ElementId::Name(key.clone().into()))
            // The debug selector lets the headless tests read the cell's
            // laid-out bounds.
            .debug_selector(move || key)
            .flex()
            .items_center()
            .justify_center()
            .size(px(36.))
            .text_size(px(13.))
            .child(match &self.cell {
                Some(render) => render(RangeCalendarCellState {
                    date,
                    formatted_date: date.day.to_string().into(),
                    is_selected,
                    is_selection_start: is_start,
                    is_selection_end: is_end,
                    is_unavailable: unavailable,
                    is_outside_month: outside_month,
                    is_today,
                    is_disabled: disabled,
                }),
                None => date.day.to_string().into_any_element(),
            });

        if draw_start || draw_end {
            cell = cell
                .rounded_full()
                .bg(accent.color)
                .text_color(accent.foreground)
                .font_weight(gpui::FontWeight::SEMIBOLD);
        } else if is_today {
            // `.range-calendar__cell[data-today]` fills the button with
            // `bg-accent-soft text-accent-soft-foreground`, today or inside
            // the range alike; only its hover (never on a selected cell)
            // deepens the same soft fill rather than `bg-default`.
            cell = cell
                .rounded_full()
                .bg(accent.soft())
                .text_color(accent.soft_foreground(colors.foreground));
            if selectable {
                cell = cell.cursor_pointer();
                if !is_selected {
                    let hover_bg = accent.soft_hover();
                    cell = cell.hover(move |s| s.bg(hover_bg));
                }
            }
        } else if in_range {
            // The track fill lives on the outer cell, so the pressed inner
            // button stays transparent with the base foreground text and the
            // run never breaks.
        } else {
            cell = cell.rounded_full();
            if selectable {
                let hover_bg = colors.default.color;
                cell = cell.cursor_pointer().hover(move |s| s.bg(hover_bg));
            }
        }

        // `.range-calendar__cell[data-pressed]` scales the inner button to
        // 0.9 and only the range caps recolour, to `bg-accent-hover` even
        // when the cap is also today; a pressed middle or today cell keeps
        // its soft fill, while a pressed plain day shows the hover fill it
        // already wears. The recolour has to merge with the press geometry
        // in one refinement -- a chained `.active` would overwrite it and
        // drop the scale.
        let cell = if selectable {
            let press_box = crate::anim::PressBox {
                height: px(36.),
                padding_x: None,
                width: Some(px(36.)),
                min_width: None,
                text_size: px(13.),
                line_height: px(18.),
                gap: px(0.),
                radius: px(18.),
                shrink_x: true,
                scale: crate::anim::PRESSED_SCALE_RANGE,
            };
            if draw_start || draw_end {
                crate::anim::pressed_with_background(cell, press_box, accent.hover(), cx)
            } else if is_today || in_range {
                crate::anim::pressed(cell, press_box, cx)
            } else {
                let pressed_bg = colors.default.color;
                crate::anim::pressed_with_background(cell, press_box, pressed_bg, cx)
            }
        } else {
            cell
        };

        // `.range-calendar__cell` takes `status-focused` -- a ring, not a border,
        // which would shrink the cell as the cursor arrived.
        let mut cell = util::with_focus_ring(
            cell,
            !outside_month && frame.focused == Some(date),
            true,
            Vec::new(),
            cx,
        );

        if disabled || unavailable {
            cell = cell.text_color(colors.muted);
            if disabled && !outside_month {
                cell = cell.line_through();
            }
        }

        if selectable {
            // Tracking the hovered cell is what makes the half-open range
            // preview between the anchor and the cursor.
            let hover_state = self.state.clone();
            let hover_cursor = frame.cursor.clone();
            let hover_preview = frame.focus_preview.clone();
            let hover_focus = self.on_focus_change.clone();
            cell = cell.on_hover(move |over, window, cx| {
                let over = *over;
                let selecting = hover_state.update(cx, |s, cx| {
                    if over {
                        s.hovered = Some(date);
                    } else if s.hovered == Some(date) {
                        s.hovered = None;
                    }
                    cx.notify();
                    s.start.is_some() && s.end.is_none()
                });
                if over && selecting {
                    hover_cursor.update(cx, |focused, cx| {
                        *focused = Some(date);
                        cx.notify();
                    });
                    hover_preview.update(cx, |preview, _| *preview = true);
                    if let Some(cb) = &hover_focus {
                        cb(date, window, cx);
                    }
                }
            });

            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            let constraints = self.constraints.clone();
            let range_date_unavailable = self.range_date_unavailable.clone();
            let allows_non_contiguous_ranges = self.allows_non_contiguous_ranges;
            let cursor = frame.cursor.clone();
            let focus_preview = frame.focus_preview.clone();
            let selection_before_anchor = frame.selection_before_anchor.clone();
            cell = cell.on_click(move |_, window, cx| {
                if let Some(cb) = &on_focus {
                    cb(date, window, cx);
                }
                let (next, previous) = state.update(cx, |s, cx| {
                    let previous = s.start.zip(s.end);
                    let next = resolve_pick(
                        s.start,
                        s.end,
                        date,
                        &constraints,
                        range_date_unavailable.as_ref(),
                        allows_non_contiguous_ranges,
                    );
                    if let Some((start, end)) = next {
                        s.start = Some(start);
                        s.end = end;
                        s.user_navigated = true;
                        cx.notify();
                    }
                    (next, previous)
                });
                cursor.update(cx, |focused, cx| {
                    *focused = Some(date);
                    cx.notify();
                });
                focus_preview.update(cx, |preview, _| *preview = false);
                selection_before_anchor.update(cx, |saved, _| {
                    if let Some((_, end)) = next {
                        *saved = if end.is_none() { previous } else { None };
                    }
                });
                if let (Some(cb), Some((start, Some(end)))) = (&on_change, next) {
                    cb(start, end, window, cx);
                }
            });
        }

        let mut track = track.child(cell);

        // `.range-calendar__cell-indicator` is a `size-[3px] rounded-xs` dot
        // at `bottom-1`, centred in the cell, in the selected cell's
        // foreground when the day is chosen.
        if self.cell_indicator.as_ref().is_some_and(|f| f(date)) {
            let marker = if is_start || is_end || in_range {
                accent.foreground
            } else {
                colors.muted
            };
            track = track.child(
                div()
                    .absolute()
                    // The debug selector lets the headless tests read the
                    // dot's laid-out bounds.
                    .debug_selector(move || indicator_key)
                    .left(px((36. - 3.) / 2.))
                    .bottom(px(4.))
                    .size(px(3.))
                    .rounded(px(2.))
                    .bg(marker),
            );
        }

        track
            .when(outside_month, |track| track.opacity(0.5))
            .into_any_element()
    }

    /// The seven column headers -- `.range-calendar__grid-header`.
    fn weekday_header(&self, cx: &App) -> gpui::Div {
        let muted = cx.colors().muted;
        let mut row = div().flex().flex_row().w_full();
        for label in self.constraints.first_day_of_week.header_row() {
            row = row.child(
                div()
                    .w(px(36.))
                    .text_center()
                    // `.range-calendar__header-cell` is `text-xs`.
                    .text_size(px(12.))
                    .text_color(muted)
                    .child(label),
            );
        }
        row
    }

    /// The 7-column grid for one month.
    fn month_grid(&self, y: i32, m: u32, frame: &Frame<'_>, cx: &App) -> gpui::AnyElement {
        let lead = self.constraints.lead_cells(y, m);
        let total = days_in_month(y, m) as usize;
        let rows = self.constraints.rows(y, m);

        // The pinned cells carry `my-[2px]` margins, so two rows sit 4px
        // apart vertically while the seven 36px columns touch horizontally.
        let mut grid = div().flex().flex_col().gap(px(4.));
        for row_index in 0..rows {
            let mut row = div().flex().flex_row();
            for column in 0..7 {
                let index = row_index * 7 + column;
                let cell = if index < lead {
                    let (previous_year, previous_month) = add_months(y, m, -1);
                    let day =
                        days_in_month(previous_year, previous_month) as usize - lead + index + 1;
                    self.range_cell(
                        Date::new(previous_year, previous_month, day as u32),
                        true,
                        frame,
                        format!(
                            "{}-{y}-{m}-outside-{previous_year}-{previous_month}-day-{day}",
                            frame.base
                        ),
                        CellSlot { column, columns: 7 },
                        cx,
                    )
                } else {
                    let day = index - lead + 1;
                    if day <= total {
                        self.range_cell(
                            Date::new(y, m, day as u32),
                            false,
                            frame,
                            format!("{}-{y}-{m}-day-{day}", frame.base),
                            CellSlot { column, columns: 7 },
                            cx,
                        )
                    } else {
                        let (next_year, next_month) = add_months(y, m, 1);
                        let next_day = day - total;
                        self.range_cell(
                            Date::new(next_year, next_month, next_day as u32),
                            true,
                            frame,
                            format!(
                                "{}-{y}-{m}-outside-{next_year}-{next_month}-day-{next_day}",
                                frame.base
                            ),
                            CellSlot { column, columns: 7 },
                            cx,
                        )
                    }
                };
                row = row.child(cell);
            }
            grid = grid.child(row);
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
        let accent = if self.is_invalid {
            colors.danger
        } else {
            colors.accent
        };
        let active_year = view.active_year;
        let base = view.base;
        let mut grid = div().flex().flex_col().gap(px(4.)).p(px(4.));
        for chunk in view.years.chunks(3) {
            let mut row = div().flex().gap(px(4.));
            for &year in chunk {
                let is_active = year == active_year;
                let mut cell = div()
                    .id(ElementId::Name(format!("{base}-y{year}").into()))
                    .when(!self.is_disabled && is_active, |cell| {
                        cell.track_focus(year_focus)
                    })
                    .flex_1()
                    .h(px(32.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(14.))
                    .rounded(util::control_radius(cx));
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
                    cell = util::ring_if_focused(cell, year_focus, false, Vec::new(), window, cx);
                }
                row = row.child(cell.child(year.to_string()));
            }
            grid = grid.child(row);
        }
        grid.into_any_element()
    }
}

impl RenderOnce for RangeCalendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            util::seed_once(
                window,
                cx,
                ElementId::Name(
                    format!("rangecalendar-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.start = Some(value.0);
                        s.end = Some(value.1);
                        s.view_year = value.0.year;
                        s.view_month = value.0.month;
                        s.view_day = value.0.day;
                        cx.notify();
                    });
                },
            );
        }

        let base = format!("{:?}", self.id);

        // `isYearPickerOpen` wins; without it the component holds the flag and
        // the heading toggles it, which is what `defaultYearPickerOpen`
        // promises. This borrows `cx` mutably, so it precedes the tokens.
        let (year_picker_open, year_picker_own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{base}-yearpicker").into()),
            self.year_picker_open,
            self.default_year_picker_open,
        );

        // The grid is one tab stop with a cursor inside it, as the Calendar's is.
        // `use_keyed_state` takes `cx` mutably, so both precede the theme.
        let grid_focus =
            util::tab_stop_handle(ElementId::Name(format!("{base}-focus").into()), window, cx);
        let prev_focus = util::tab_stop_handle(
            ElementId::Name(format!("{base}-prev-focus").into()),
            window,
            cx,
        );
        let next_focus = util::tab_stop_handle(
            ElementId::Name(format!("{base}-next-focus").into()),
            window,
            cx,
        );
        let year_focus = util::tab_stop_handle(
            ElementId::Name(format!("{base}-year-focus").into()),
            window,
            cx,
        );
        // Inside a picker the grid takes the focus as the panel opens, so the
        // arrows work without hunting for it with Tab.
        if self.autofocus_grid && !self.is_disabled && !year_picker_open {
            util::focus_once(
                window,
                cx,
                ElementId::Name(format!("{base}-autofocus").into()),
                &grid_focus,
            );
        }
        let cursor = window.use_keyed_state(
            ElementId::Name(format!("{base}-cursor").into()),
            cx,
            |_, _| None::<Date>,
        );
        let focus_preview = window.use_keyed_state(
            ElementId::Name(format!("{base}-focus-preview").into()),
            cx,
            |_, _| false,
        );
        let selection_before_anchor = window.use_keyed_state(
            ElementId::Name(format!("{base}-selection-before-anchor").into()),
            cx,
            |_, _| None::<(Date, Date)>,
        );
        let year_cursor = window.use_keyed_state(
            ElementId::Name(format!("{base}-year-cursor").into()),
            cx,
            |_, _| None::<i32>,
        );
        let year_was_open = window.use_keyed_state(
            ElementId::Name(format!("{base}-year-was-open").into()),
            cx,
            |_, _| false,
        );
        let year_trigger_index = window.use_keyed_state(
            ElementId::Name(format!("{base}-year-trigger-index").into()),
            cx,
            |_, _| 0usize,
        );
        let cursor_at = *cursor.read(cx);
        let focus_preview_at = *focus_preview.read(cx);

        let first_day = self.constraints.first_day_of_week;
        let focused_value = self
            .focused_value
            .map(|date| self.constraints.constrain(date));

        let (stored_anchor, selection_start, selection_end, hovered, navigated) = {
            let st = self.state.read(cx);
            (st.anchor(), st.start, st.end, st.hovered, st.user_navigated)
        };
        let active_preview = hovered.or(focus_preview_at.then_some(cursor_at).flatten());
        let (paint_start, preview_end) = match (selection_start, selection_end, active_preview) {
            (Some(start), None, Some(preview)) => resolve_pick(
                Some(start),
                None,
                preview,
                &self.constraints,
                self.range_date_unavailable.as_ref(),
                self.allows_non_contiguous_ranges,
            )
            .map_or((Some(start), None), |(start, end)| (Some(start), end)),
            _ => (selection_start, selection_end),
        };
        // Pinned React Stately starts a long range at the first visible unit
        // when its end would fall beyond the default centered window. An
        // explicit `selectionAlignment` always wins.
        let selection_alignment =
            self.selection_alignment
                .unwrap_or_else(|| match (selection_start, selection_end) {
                    (Some(start), Some(end)) => {
                        let centered_anchor = calendar_view::aligned_anchor(
                            self.duration,
                            SelectionAlignment::Center,
                            first_day,
                            start,
                        );
                        let (_, centered_end) =
                            calendar_view::visible_range(self.duration, first_day, centered_anchor);
                        if days_from_civil(&end) > days_from_civil(&centered_end) {
                            SelectionAlignment::Start
                        } else {
                            SelectionAlignment::Center
                        }
                    }
                    _ => SelectionAlignment::Center,
                });
        // `selectionAlignment` frames the range around the selection start,
        // until the user drives navigation themselves.
        let anchor = match (navigated, selection_start) {
            (false, Some(sel)) => {
                calendar_view::aligned_anchor(self.duration, selection_alignment, first_day, sel)
            }
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
        // Taking the focus puts the ring where v3 would have focused -- the
        // range's start, or today.
        let ring_at = focused_value
            .or(cursor_at)
            .or(selection_start)
            .or_else(|| Some(Date::today()))
            .filter(|_| grid_focus.is_focused(window));
        let frame = Frame {
            start: paint_start,
            preview_end,
            unavailable_anchor: selection_start.filter(|_| selection_end.is_none()),
            today: Date::today(),
            cursor: &cursor,
            focus_preview: &focus_preview,
            selection_before_anchor: &selection_before_anchor,
            base: &base,
            focused: ring_at,
        };

        let months = calendar_view::month_headings(self.duration, anchor);
        let linear = calendar_view::linear_cells(self.duration, first_day, anchor);
        let columns = months.len().max(1);
        let mut heading_focuses = Vec::with_capacity(columns);
        for index in 0..columns {
            heading_focuses.push(util::tab_stop_handle(
                ElementId::Name(format!("{base}-heading-{index}-focus").into()),
                window,
                cx,
            ));
        }
        let active_heading_index = (*year_trigger_index.read(cx)).min(columns - 1);
        let active_heading_focus = heading_focuses[active_heading_index].clone();

        let colors = cx.colors();
        let layout = cx.layout();

        let nav_target =
            |dir: i32| calendar_view::page(self.duration, self.page_behavior, anchor, dir);
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
        let nav_btn = |icon: &'static str,
                       target: Date,
                       key: String,
                       focus: &gpui::FocusHandle,
                       disabled: bool| {
            let state = state_for_nav.clone();
            // `.range-calendar__nav-button:hover` fills with `bg-default`.
            let hover_bg = colors.default.color;
            let debug_key = key.clone();
            // `.range-calendar__nav-button:active` scales the box to 0.95.
            let press_box = crate::anim::PressBox {
                height: px(24.),
                padding_x: None,
                width: Some(px(24.)),
                min_width: None,
                text_size: px(16.),
                line_height: px(0.),
                gap: px(0.),
                radius: util::small_radius(cx),
                shrink_x: true,
                scale: crate::anim::PRESSED_SCALE_DEEP,
            };
            let button = div()
                .id(ElementId::Name(key.into()))
                // The debug selector lets the headless tests read the
                // button's laid-out bounds.
                .debug_selector(move || debug_key)
                .when(!disabled, |b| b.track_focus(focus))
                .flex()
                .items_center()
                .justify_center()
                // `.range-calendar__nav-button` is `size-6 rounded-xl` -- one
                // radius step tighter than the single calendar's, which is
                // `rounded-2xl`.
                .size(px(24.))
                .rounded(util::small_radius(cx))
                .when(!disabled, |b| {
                    let pressed = b
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(move |_, _, cx| {
                            state.update(cx, |s, cx| {
                                s.set_anchor(target);
                                cx.notify();
                            });
                        });
                    crate::anim::pressed(pressed, press_box, cx)
                })
                .when(disabled, |b| b.opacity(layout.disabled_opacity));
            util::ring_if_focused(button, focus, true, Vec::new(), window, cx).child(
                gpui::svg()
                    // `.range-calendar__nav-button-icon` is `size-4`, painted
                    // `text-accent-soft-foreground` like its button.
                    .size(px(16.))
                    .path(icon)
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
            let label = div()
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
                    let trigger = div()
                        .id(ElementId::Name(key.into()))
                        .when(!self.is_disabled, |trigger| trigger.track_focus(focus))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`
                        // and hovers nothing: only focus and the open state
                        // recolour it.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(util::key_radius(cx))
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
                    util::ring_if_focused(trigger, focus, true, Vec::new(), window, cx)
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
                    let trigger = div()
                        .id(ElementId::Name(key.into()))
                        .when(!self.is_disabled, |trigger| trigger.track_focus(focus))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`
                        // and hovers nothing: only focus and the open state
                        // recolour it.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(util::key_radius(cx))
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
                    util::ring_if_focused(trigger, focus, true, Vec::new(), window, cx)
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

        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(colors.surface.foreground)
            // readOnly blocks selection, not focus or navigation.
            .when(!self.is_disabled && !year_picker_open, |el| {
                el.track_focus(&grid_focus)
            });

        // The same keys the Calendar answers, and Enter picks: the first press
        // sets the range's start, the second its end, which is what `pick` does
        // for a click.
        if !self.is_disabled && !year_picker_open {
            let held = cursor.clone();
            let focus_preview = focus_preview.clone();
            let selection_before_anchor = selection_before_anchor.clone();
            let focus = grid_focus.clone();
            let prev_control = prev_focus.clone();
            let next_control = next_focus.clone();
            let heading_controls = heading_focuses.clone();
            let controlled_focus = focused_value;
            let from_start = focused_value
                .or(selection_start)
                .unwrap_or_else(Date::today);
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            let constraints = self.constraints.clone();
            let range_date_unavailable = self.range_date_unavailable.clone();
            let allows_non_contiguous_ranges = self.allows_non_contiguous_ranges;
            let read_only = self.is_read_only;
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
                let at = controlled_focus.or(*held.read(cx)).unwrap_or(from_start);
                let key = event.keystroke.key.as_str();
                let shift = event.keystroke.modifiers.shift;
                if key == "escape" {
                    let restore = *selection_before_anchor.read(cx);
                    let cancelled = state.update(cx, |s, cx| {
                        if s.start.is_some() && s.end.is_none() {
                            if let Some((start, end)) = restore {
                                s.start = Some(start);
                                s.end = Some(end);
                            } else {
                                s.start = None;
                            }
                            s.hovered = None;
                            cx.notify();
                            true
                        } else {
                            false
                        }
                    });
                    focus_preview.update(cx, |preview, _| *preview = false);
                    if cancelled {
                        selection_before_anchor.update(cx, |saved, _| *saved = None);
                    }
                    return;
                }
                if matches!(key, "enter" | "space") {
                    if read_only {
                        return;
                    }
                    let (next, previous) = state.update(cx, |s, cx| {
                        let previous = s.start.zip(s.end);
                        let next = resolve_pick(
                            s.start,
                            s.end,
                            at,
                            &constraints,
                            range_date_unavailable.as_ref(),
                            allows_non_contiguous_ranges,
                        );
                        if let Some((start, end)) = next {
                            s.start = Some(start);
                            s.end = end;
                            s.hovered = None;
                            s.user_navigated = true;
                            cx.notify();
                        }
                        (next, previous)
                    });
                    if let (Some(cb), Some((start, Some(end)))) = (&on_change, next) {
                        cb(start, end, window, cx);
                    }
                    if let Some((_, end)) = next {
                        focus_preview.update(cx, |preview, _| *preview = end.is_none());
                        selection_before_anchor.update(cx, |saved, _| {
                            *saved = if end.is_none() { previous } else { None };
                        });
                    }
                    if let Some(next_focus) = next.and_then(|(_, end)| {
                        end.is_none().then(|| {
                            keyboard_range_focus(
                                at,
                                &constraints,
                                range_date_unavailable.as_ref(),
                                allows_non_contiguous_ranges,
                            )
                        })?
                    }) {
                        if controlled_focus.is_none() {
                            held.update(cx, |focused, cx| {
                                *focused = Some(next_focus);
                                cx.notify();
                            });
                            state.update(cx, |s, cx| {
                                let next_anchor = calendar_view::anchor_following_focus(
                                    duration,
                                    first_day,
                                    anchor,
                                    visible_start,
                                    visible_end,
                                    next_focus,
                                );
                                if next_anchor != anchor {
                                    s.set_anchor(next_anchor);
                                    cx.notify();
                                }
                            });
                        }
                        if let Some(cb) = &on_focus {
                            cb(next_focus, window, cx);
                        }
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
                    focus_preview.update(cx, |preview, _| *preview = true);
                    // React Aria realigns a week/month window only after focus
                    // leaves it. Day views page the whole window directly.
                    state.update(cx, |s, cx| {
                        s.hovered = None;
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
            root = root.w(crate::calendar::CALENDAR_WIDTH);
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    // `.range-calendar__header` is `px-0.5`.
                    .px(px(2.))
                    .child(div().size(px(24.)))
                    .child(heading(
                        calendar_view::month_heading(
                            anchor.year,
                            anchor.month,
                            self.year_heading_offset_months,
                        ),
                        format!("{base}-yheading"),
                        &active_heading_focus,
                        active_heading_index,
                    ))
                    .child(div().size(px(24.))),
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
            let mut row = div().flex().gap(px(20.));
            for (i, &(y, m)) in months.iter().enumerate() {
                let first = i == 0;
                let last = i + 1 == columns;
                let mut col = div().flex().flex_col().gap(px(8.)).w(px(252.));
                // Only the outer columns carry nav buttons; the rest keep a
                // same-size spacer so every heading lines up.
                // The same box as a nav button, so every heading lines up.
                let spacer = || div().size(px(24.)).into_any_element();
                col = col.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .w_full()
                        // `.range-calendar__header` is `px-0.5`.
                        .px(px(2.))
                        .child(if first {
                            nav_btn(
                                icons::CHEVRON_LEFT,
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
                            calendar_view::month_heading(
                                y,
                                m,
                                self.year_heading_offset_months,
                            ),
                            format!("{base}-heading{i}"),
                            &heading_focuses[i],
                            i,
                        ))
                        .child(if last {
                            nav_btn(
                                icons::CHEVRON_RIGHT,
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
            // Week and day views: a flat run of real dates, so no lead blanks.
            let per_row = if matches!(self.duration, VisibleDuration::Weeks(_)) {
                7
            } else {
                linear.len().max(1)
            };
            root = root.w(crate::calendar::CALENDAR_WIDTH);
            root = root.child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    // `.range-calendar__header` is `px-0.5`.
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
                root = root.child(div().flex().flex_row().children(linear.iter().map(|d| {
                    div()
                        .w(px(36.))
                        .text_center()
                        // A header cell is `text-xs`, like the seven-column one.
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child(Weekday::ALL[weekday_index(*d)].short_label().to_owned())
                })));
            }
            // `.range-calendar__grid` wraps the header and
            // `.range-calendar__grid-body`; each line is a
            // `.range-calendar__grid-row` of `.range-calendar__cell-button`s.
            // Rows sit 4px apart, matching the pinned cell margins.
            let mut grid = div().flex().flex_col().gap(px(4.));
            for chunk in linear.chunks(per_row) {
                let mut line = div().flex().flex_row();
                for (index, &date) in chunk.iter().enumerate() {
                    line = line.child(self.range_cell(
                        date,
                        false,
                        &frame,
                        format!("{base}-{}", date.format_iso()),
                        CellSlot {
                            column: index,
                            columns: chunk.len(),
                        },
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

// The pinned today cell fills `bg-accent-soft` with
// `text-accent-soft-foreground` and hovers `bg-accent-soft-hover`; the old
// port invented an accent border, so the check is mechanical.
#[cfg(test)]
mod hover_tokens {
    #[test]
    fn the_today_cell_uses_the_accent_soft_tokens() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.matches(".bg(accent.soft())").count() >= 2,
            "the today cell must fill `bg-accent-soft` like the range \
             interior (pinned `.range-calendar__cell[data-today]`)"
        );
        assert!(
            source.contains("let hover_bg = accent.soft_hover();"),
            "the today cell must hover `bg-accent-soft-hover` \
             (pinned `.range-calendar__cell[data-today]:hover`)"
        );
        assert!(
            !source.contains("cell.border_1()"),
            "the today cell must not invent an accent border"
        );
    }

    // gpui's `active` refinement overwrites the previous one, so chaining
    // `.active` after `anim::pressed` would drop the 0.9 press scale. The
    // pressed recolour must merge with the geometry in one refinement, which
    // is what `pressed_with_background` exists for.
    #[test]
    fn the_pressed_cell_merges_background_with_the_scale() {
        // Scan the implementation only; this test's own text names the
        // forbidden chaining.
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            !source.contains(".active("),
            "a chained `.active` after `anim::pressed` replaces the pressed \
             scale (gpui overwrites the active refinement)"
        );
    }

    // Pinned pressed state: only the range caps recolour, to
    // `bg-accent-hover` even when the cap is also today; a pressed middle or
    // today cell keeps its `bg-accent-soft` fill, and a pressed plain day
    // shows the hover fill it already wears.
    #[test]
    fn the_pressed_caps_recolour_and_the_interior_keeps_its_fill() {
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("pressed_with_background(cell, press_box, accent.hover(), cx)"),
            "a pressed range cap must fill `bg-accent-hover` \
             (pinned `[data-pressed] [data-selection-start|end]`)"
        );
        assert!(
            source.contains("} else if is_today || in_range {"),
            "a pressed middle or today cell must keep its `bg-accent-soft` \
             fill (the pinned pressed state recolours only the caps)"
        );
    }

    #[test]
    fn the_year_picker_trigger_hovers_nothing() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            !source.contains("colors.default.soft_hover()"),
            "the year-picker trigger must not invent a hover background"
        );
    }

    // Pinned anatomy: the selected track fill lives on the outer cell and
    // recolours nothing, so the interior `.range-calendar__cell-button`
    // keeps the base `text-foreground`; only `data-today` recolours its text,
    // today or inside the range alike.
    #[test]
    fn the_interior_text_stays_base_foreground_unless_today() {
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("track = track.bg(accent.soft());"),
            "the range track fill must not set a text colour (pinned \
             `.range-calendar__cell-button` keeps `text-foreground`)"
        );
        let today = source
            .find("} else if is_today {")
            .expect("the today branch exists");
        let interior = source
            .find("} else if in_range {")
            .expect("the interior branch exists");
        assert!(
            today < interior,
            "the today branch must precede the interior branch, or a today \
             cell inside the range would fall through to the base foreground"
        );
    }

    // Pinned row-boundary rounding: a selected cell in the first column
    // rounds its left side `lg`, the last column its right side, and a cap
    // rounds its own side `3xl` on any column.
    #[test]
    fn the_track_rounds_row_boundaries_and_cap_sides() {
        let source = include_str!("range_calendar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        for snippet in [
            "let cap_radius = util::control_radius(cx);",
            "let edge_radius = util::key_radius(cx);",
            "} else if slot.is_first() {",
            "} else if slot.is_last() {",
            ".rounded_tl(left)",
            ".rounded_bl(left)",
            ".rounded_tr(right)",
            ".rounded_br(right)",
        ] {
            assert!(
                source.contains(snippet),
                "the track rounding must implement the pinned row-boundary \
                 and cap variants; missing {snippet:?}"
            );
        }
    }
}
