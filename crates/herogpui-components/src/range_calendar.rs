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
    calendar::{
        add_days, days_from_civil, days_in_month, month_name, month_step, weekday_index, Date,
    },
    calendar_view::{self, PageBehavior, SelectionAlignment, VisibleDuration},
    date_constraints::{DateConstraints, Weekday},
    date_picker::DateRangeState,
    icons, util,
};

type OnRangeChange = Arc<dyn Fn(Option<Date>, Option<Date>, &mut Window, &mut App) + 'static>;

/// HeroUI RangeCalendar.
#[derive(IntoElement)]
pub struct RangeCalendar {
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<(Date, Date)>,
    id: ElementId,
    state: Entity<DateRangeState>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_read_only: bool,
    /// Set by a picker: take the focus as the panel opens. See
    /// [`RangeCalendar::autofocus_grid`].
    autofocus_grid: bool,
    is_invalid: bool,
    focused_value: Option<Date>,
    duration: VisibleDuration,
    page_behavior: PageBehavior,
    selection_alignment: SelectionAlignment,
    /// `isYearPickerOpen` — `None` leaves the component holding the state,
    /// seeded from `defaultYearPickerOpen`.
    year_picker_open: Option<bool>,
    default_year_picker_open: bool,
    on_year_picker_open_change: Option<Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_focus_change: Option<Arc<dyn Fn(Date, &mut Window, &mut App) + 'static>>,
    /// `allowsNonContiguousRanges` — lets a range span unavailable dates.
    allows_non_contiguous_ranges: bool,
    on_change: Option<OnRangeChange>,
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
            constraints: DateConstraints::new(),
            is_disabled: false,
            is_read_only: false,
            autofocus_grid: false,
            is_invalid: false,
            focused_value: None,
            duration: VisibleDuration::default(),
            page_behavior: PageBehavior::default(),
            selection_alignment: SelectionAlignment::default(),
            year_picker_open: None,
            default_year_picker_open: false,
            on_year_picker_open_change: None,
            on_focus_change: None,
            allows_non_contiguous_ranges: false,
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
    /// year-picker trigger.
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

    /// `isDateUnavailable` — blocks individual dates inside the range.
    pub fn is_date_unavailable(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.constraints.is_date_unavailable = Some(Arc::new(f));
        self
    }

    /// `firstDayOfWeek`
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
        self.constraints = constraints;
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

    /// Called with `(start, end)` after every pick.
    pub fn on_change(
        mut self,
        handler: impl Fn(Option<Date>, Option<Date>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn is_selectable(&self, date: Date) -> bool {
        if self.constraints.out_of_range(date) {
            return false;
        }
        // A non-contiguous range may span unavailable dates, so they stay
        // pickable as endpoints.
        if !self.allows_non_contiguous_ranges && self.constraints.is_unavailable(date) {
            return false;
        }
        true
    }
}

/// The per-frame facts every cell needs, bundled so the helpers below take a
/// readable argument list.
struct Frame<'a> {
    start: Option<Date>,
    preview_end: Option<Date>,
    today: Date,
    interactive: bool,
    base: &'a str,
    /// The date wearing the focus ring: `focusedValue` when the caller controls
    /// it, otherwise wherever the arrow keys have walked to. `None` while the
    /// grid does not hold the keyboard.
    focused: Option<Date>,
}

impl RangeCalendar {
    /// One day cell. The range interior is a square fill so the run reads as
    /// continuous; the two ends are pills.
    fn range_cell(&self, date: Date, frame: &Frame<'_>, key: String, cx: &App) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = if self.is_invalid {
            colors.danger
        } else {
            colors.accent
        };
        let serial = days_from_civil(&date);
        let start_day = frame.start.map(|d| days_from_civil(&d));
        let end_day = frame.preview_end.map(|d| days_from_civil(&d));
        let selectable = frame.interactive && self.is_selectable(date);

        let is_start = start_day == Some(serial);
        let is_end = end_day == Some(serial);
        let in_range = match (start_day, end_day) {
            (Some(a), Some(b)) => serial > a.min(b) && serial < a.max(b),
            _ => false,
        };
        let is_today = serial == days_from_civil(&frame.today);

        let mut cell = div()
            .id(ElementId::Name(key.into()))
            .flex()
            .items_center()
            .justify_center()
            .size(px(38.))
            .text_size(px(13.))
            .child(date.day.to_string());

        if is_start || is_end {
            cell = cell
                .rounded_full()
                .bg(accent.color)
                .text_color(accent.foreground)
                .font_weight(gpui::FontWeight::SEMIBOLD);
        } else if in_range {
            cell = cell.bg(accent.soft()).text_color(accent.soft_foreground());
        } else {
            cell = cell.rounded_full();
            if is_today {
                cell = cell.border_1().border_color(accent.color);
            }
            if selectable {
                let hover_bg = colors.default.color;
                cell = cell.cursor_pointer().hover(move |s| s.bg(hover_bg));
            }
        }

        // `.range-calendar__cell[data-pressed]` fills with `bg-default` and
        // scales to 0.95, the same press a calendar cell takes.
        let cell = if selectable {
            let pressed_bg = colors.default.color;
            crate::anim::pressed(
                cell,
                crate::anim::PressBox {
                    height: px(38.),
                    padding_x: None,
                    width: Some(px(38.)),
                    min_width: None,
                    text_size: px(13.),
                    line_height: px(18.),
                    gap: px(0.),
                    radius: px(19.),
                    shrink_x: true,
                    scale: crate::anim::PRESSED_SCALE_DEEP,
                },
                cx,
            )
            .active(move |st| st.bg(pressed_bg))
        } else {
            cell
        };

        // `.range-calendar__cell` takes `status-focused` -- a ring, not a border,
        // which would shrink the cell as the cursor arrived.
        let mut cell =
            util::with_focus_ring(cell, frame.focused == Some(date), true, Vec::new(), cx);

        if !selectable {
            cell = cell.text_color(colors.muted);
            if self.constraints.is_unavailable(date) {
                cell = cell.line_through();
            }
        } else {
            // Tracking the hovered cell is what makes the half-open range
            // preview between the anchor and the cursor.
            let hover_state = self.state.clone();
            cell = cell.on_hover(move |over, _, cx| {
                let over = *over;
                hover_state.update(cx, |s, cx| {
                    if over {
                        s.hovered = Some(date);
                    } else if s.hovered == Some(date) {
                        s.hovered = None;
                    }
                    cx.notify();
                });
            });

            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            cell = cell.on_click(move |_, window, cx| {
                if let Some(cb) = &on_focus {
                    cb(date, window, cx);
                }
                let (next_start, next_end) = state.update(cx, |s, cx| {
                    s.pick(date);
                    cx.notify();
                    (s.start, s.end)
                });
                if let Some(cb) = &on_change {
                    cb(next_start, next_end, window, cx);
                }
            });
        }

        cell.into_any_element()
    }

    /// The seven column headers.
    fn weekday_header(&self, cx: &App) -> gpui::Div {
        let muted = cx.colors().muted;
        let mut row = div().flex().flex_row().w_full();
        for label in self.constraints.first_day_of_week.header_row() {
            row = row.child(
                div()
                    .w(px(38.))
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

        let mut grid = div().flex().flex_col().gap(px(2.));
        let mut row = div().flex().flex_row();
        let mut cell_index = 0usize;

        // Leading blanks so the 1st lands on its weekday.
        for _ in 0..lead {
            row = row.child(div().size(px(38.)));
            cell_index += 1;
        }

        for day in 1..=total {
            if cell_index > 0 && cell_index.is_multiple_of(7) {
                grid = grid.child(row);
                row = div().flex().flex_row();
            }
            row = row.child(self.range_cell(
                Date::new(y, m, day as u32),
                frame,
                format!("{}-{y}-{m}-day-{day}", frame.base),
                cx,
            ));
            cell_index += 1;
        }
        grid.child(row).into_any_element()
    }

    /// The year grid shown while the year picker is open.
    fn year_grid(&self, anchor: Date, base: &str, cx: &App) -> gpui::AnyElement {
        let colors = cx.colors();
        let accent = if self.is_invalid {
            colors.danger
        } else {
            colors.accent
        };
        let years = calendar_view::year_page(anchor.year, 12);
        let mut grid = div().flex().flex_col().gap(px(4.));
        for chunk in years.chunks(3) {
            let mut row = div().flex().gap(px(4.));
            for &year in chunk {
                let is_current = year == anchor.year;
                let mut cell = div()
                    .id(ElementId::Name(format!("{base}-y{year}").into()))
                    .flex_1()
                    .h(px(38.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(13.))
                    .rounded(util::control_radius(cx));
                if is_current {
                    cell = cell
                        .bg(accent.color)
                        .text_color(accent.foreground)
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else {
                    let hover_bg = colors.default.color;
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
        // Inside a picker the grid takes the focus as the panel opens, so the
        // arrows work without hunting for it with Tab.
        if self.autofocus_grid && !self.is_disabled {
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
        let cursor_at = *cursor.read(cx);

        let colors = cx.colors();
        let layout = cx.layout();
        let first_day = self.constraints.first_day_of_week;

        let (stored_anchor, start, preview_end, navigated) = {
            let st = self.state.read(cx);
            (st.anchor(), st.start, st.preview_end(), st.user_navigated)
        };
        // `selectionAlignment` frames the range around the selection start,
        // until the user drives navigation themselves.
        let anchor = match (navigated, start) {
            (false, Some(sel)) => calendar_view::aligned_anchor(
                self.duration,
                self.selection_alignment,
                first_day,
                sel,
            ),
            _ => stored_anchor,
        };
        // Taking the focus puts the ring where v3 would have focused -- the
        // range's start, or today.
        let ring_at = self
            .focused_value
            .or(cursor_at)
            .or(start)
            .or_else(|| Some(Date::today()))
            .filter(|_| grid_focus.is_focused(window));
        let frame = Frame {
            start,
            preview_end,
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
        let nav_btn = |icon: &'static str, target: Date, key: String| {
            let state = state_for_nav.clone();
            let hover_bg = colors.default.color;
            div()
                .id(ElementId::Name(key.into()))
                .flex()
                .items_center()
                .justify_center()
                // `.range-calendar__nav-button` is `size-6 rounded-xl` -- one
                // radius step tighter than the single calendar's, which is
                // `rounded-2xl`.
                .size(px(24.))
                .rounded(util::small_radius(cx))
                .cursor_pointer()
                .text_color(colors.muted)
                .hover(move |s| s.bg(hover_bg))
                .child(
                    gpui::svg()
                        // `.range-calendar__nav-button-icon` is `size-4`.
                        .size(px(16.))
                        .path(icon)
                        .text_color(colors.muted),
                )
                .on_click(move |_, _, cx| {
                    state.update(cx, |s, cx| {
                        s.set_anchor(target);
                        cx.notify();
                    });
                })
        };

        // The heading is a plain label unless a year-picker handler is
        // supplied, in which case it becomes the trigger.
        let heading = |text: String, key: String| -> gpui::AnyElement {
            let label = div()
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
                    div()
                        .id(ElementId::Name(key.into()))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(util::key_radius(cx))
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
                    let hover_bg = colors.default.color;
                    div()
                        .id(ElementId::Name(key.into()))
                        .flex()
                        .items_center()
                        // `.calendar-year-picker__trigger` is `gap-1 rounded-lg`.
                        .gap(px(4.))
                        .px(px(6.))
                        .py(px(2.))
                        .rounded(util::key_radius(cx))
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

        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(colors.surface.foreground)
            .when(!self.is_disabled && !self.is_read_only, |el| {
                el.track_focus(&grid_focus)
            });

        // The same keys the Calendar answers, and Enter picks: the first press
        // sets the range's start, the second its end, which is what `pick` does
        // for a click.
        if !self.is_disabled && !self.is_read_only {
            let held = cursor;
            let from_start = self.focused_value.or(start).unwrap_or_else(Date::today);
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let on_focus = self.on_focus_change.clone();
            let selectable = self.constraints.clone();
            root = root.on_key_down(move |event, window, cx| {
                let from = *held.read(cx);
                let at = from.unwrap_or(from_start);
                let key = event.keystroke.key.as_str();
                let shift = event.keystroke.modifiers.shift;
                if matches!(key, "enter" | "space") {
                    if !selectable.allows(at) {
                        return;
                    }
                    let (next_start, next_end) = state.update(cx, |s, cx| {
                        s.pick(at);
                        cx.notify();
                        (s.start, s.end)
                    });
                    if let Some(cb) = &on_change {
                        cb(next_start, next_end, window, cx);
                    }
                    return;
                }
                let next = match key {
                    "left" => add_days(&at, -1),
                    "right" => add_days(&at, 1),
                    "up" => add_days(&at, -7),
                    "down" => add_days(&at, 7),
                    // React Aria pages by month, and by *year* with shift.
                    "pageup" if shift => month_step(at, -12),
                    "pagedown" if shift => month_step(at, 12),
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
                // The grid follows the cursor across a month boundary, the way
                // React Aria keeps the focused date visible.
                state.update(cx, |s, cx| {
                    if s.view_year != next.year || s.view_month != next.month {
                        s.set_anchor(next);
                        cx.notify();
                    }
                });
                if let Some(cb) = &on_focus {
                    cb(next, window, cx);
                }
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
            let mut row = div().flex().gap(px(20.));
            for (i, &(y, m)) in months.iter().enumerate() {
                let first = i == 0;
                let last = i + 1 == columns;
                let mut col = div().flex().flex_col().gap(px(8.)).w(px(266.));
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
                            nav_btn(icons::CHEVRON_LEFT, nav_target(-1), format!("{base}-prev"))
                                .into_any_element()
                        } else {
                            spacer()
                        })
                        .child(heading(
                            format!("{} {}", month_name(m), y),
                            format!("{base}-heading{i}"),
                        ))
                        .child(if last {
                            nav_btn(icons::CHEVRON_RIGHT, nav_target(1), format!("{base}-next"))
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
                root = root.child(div().flex().flex_row().children(linear.iter().map(|d| {
                    div()
                        .w(px(38.))
                        .text_center()
                        // A header cell is `text-xs`, like the seven-column one.
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child(Weekday::ALL[weekday_index(*d)].short_label().to_owned())
                })));
            }
            let mut grid = div().flex().flex_col().gap(px(2.));
            for chunk in linear.chunks(per_row) {
                let mut line = div().flex().flex_row();
                for &date in chunk {
                    line = line.child(self.range_cell(
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
