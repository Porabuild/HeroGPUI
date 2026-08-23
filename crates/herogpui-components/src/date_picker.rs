//! DatePicker, DateRangePicker & DateField — port of the v3
//! `@heroui/date-picker` family: a popover calendar plus ISO text entry.
//!
//! All three share [`DateConstraints`] for `minValue` / `maxValue` /
//! `isDateUnavailable` / `firstDayOfWeek`.

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::{
    calendar::{days_from_civil, Calendar, CalendarState, Date},
    date_constraints::{DateConstraints, Weekday},
    icons,
};

type OnChange = std::sync::Arc<dyn Fn(Option<Date>, &mut Window, &mut App) + 'static>;

/// HeroUI DatePicker (controlled open state; selection lives in the entity).
#[derive(IntoElement)]
pub struct DatePicker {
    /// `name` — read back by [`DatePicker::form_field`].
    name: Option<SharedString>,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<Date>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_invalid: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    state: Entity<CalendarState>,
    /// `isOpen` — `None` leaves the picker holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    label: Option<SharedString>,
    placeholder: SharedString,
    on_change: Option<OnChange>,
}

impl DatePicker {
    /// `value` — writes the selection through to the bound state.
    pub fn value(self, date: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.selected = date);
        self
    }

    /// `minValue`
    pub fn min_value(mut self, date: Date) -> Self {
        self.constraints.min_value = Some(date);
        self
    }

    /// `maxValue`
    pub fn max_value(mut self, date: Date) -> Self {
        self.constraints.max_value = Some(date);
        self
    }

    /// `isDateUnavailable`
    pub fn is_date_unavailable(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.constraints.is_date_unavailable = Some(std::sync::Arc::new(f));
        self
    }

    /// `firstDayOfWeek`
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.constraints.first_day_of_week = day;
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

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `onOpenChange`
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn new(state: Entity<CalendarState>) -> Self {
        Self {
            name: None,
            default_value: None,
            constraints: DateConstraints::new(),
            is_disabled: false,
            is_invalid: false,
            on_open_change: None,
            state,
            is_open: None,
            default_open: false,
            label: None,
            placeholder: "Select a date".into(),
            on_change: None,
        }
    }

    /// `name` — the name this picker submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `Form` field this picker submits, when it has a `name`.
    ///
    /// The date is written ISO-8601, which is what an HTML `<input type="date">`
    /// submits. Needs `cx` because the selection lives in the state entity.
    pub fn form_field(&self, cx: &App) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        let text = self
            .state
            .read(cx)
            .selected()
            .map(|d| d.format_iso())
            .unwrap_or_default();
        Some(crate::form::FormField::text_value(name, text))
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: Date) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }
    /// `defaultOpen` — the uncontrolled initial popover state.
    ///
    /// Only consulted when `is_open` is not supplied; the picker then owns the
    /// flag and its trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<Date>, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("datepicker-default-{}", self.state.entity_id().as_u64()).into(),
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

        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("dp-{}-open", self.state.entity_id().as_u64()).into()),
            self.is_open,
            self.default_open,
        );

        let colors = cx.colors();
        let layout = cx.layout();

        let selected = self.state.read(cx).selected;
        // `.date-input-group` is `h-9`.
        let h = crate::util::FIELD_HEIGHT;

        let mut field = gpui::div()
            .id(gpui::ElementId::Name(
                format!("dp-{}", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .w_full()
            .h(h)
            .px(px(12.))
            .text_size(px(14.))
            .cursor_pointer();

        field = crate::util::apply_field_chrome(
            field,
            herogpui_core::FieldVariant::Primary,
            self.is_invalid,
            is_open,
            cx,
        );
        if !is_open {
            let hover_bg = colors.field.hover();
            field = field.hover(move |s| s.bg(hover_bg));
        }

        field = field
            .child(
                gpui::div()
                    .flex_1()
                    .truncate()
                    .text_color(if selected.is_some() {
                        colors.foreground
                    } else {
                        colors.muted
                    })
                    .child(match selected {
                        Some(d) => d.format_iso(),
                        None => self.placeholder.to_string(),
                    }),
            )
            .child(
                gpui::svg()
                    .size(px(15.))
                    .path(icons::ELLIPSIS)
                    .text_color(colors.muted),
            );

        if self.is_invalid {
            field = field.border_2().border_color(colors.danger.color);
        }

        // The trigger owns opening the popover.
        if self.is_disabled {
            field = field.opacity(layout.disabled_opacity);
        } else if self.on_open_change.is_some() || open_own.is_some() {
            let cb = self.on_open_change.clone();
            let own = open_own;
            let next = !is_open;
            field = field.on_click(move |_, window, cx| {
                // Uncontrolled: flip our own copy, or the trigger would be
                // inert without a caller handler.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                }
                if let Some(f) = &cb {
                    f(next, window, cx);
                }
            });
        }

        let mut root = gpui::div().relative().max_w(px(320.));
        let mut wrapper = gpui::div().flex().flex_col().gap(px(4.)).w_full();
        if let Some(label) = &self.label {
            wrapper = wrapper.child(
                crate::field::Label::new(label.clone())
                    .is_disabled(self.is_disabled)
                    .is_invalid(self.is_invalid),
            );
        }
        wrapper = wrapper.child(field);
        root = root.child(wrapper);

        if is_open {
            let mut cal = Calendar::new(self.state.clone())
                .constraints(self.constraints.clone())
                .is_disabled(self.is_disabled)
                .is_invalid(self.is_invalid);
            if let Some(on_change) = self.on_change.clone() {
                cal = cal.on_change(move |d, window, cx| on_change(d, window, cx));
            }
            root = root.child(crate::util::floating(
                crate::util::placed_panel(herogpui_core::Placement::BottomStart, px(6.))
                    .child(picker_panel(cx).child(cal)),
            ));
        }

        root
    }
}

/// The popover chrome every picker shares — `.date-picker__popover` is
/// `bg-overlay p-2` at `min(32px, --radius-3xl)` with `--shadow-overlay`.
///
/// The calendars used to paint this themselves, which put a second panel inside
/// the first one and left a standalone `Calendar` looking like a floating card.
fn picker_panel(cx: &App) -> gpui::Div {
    let colors = cx.colors();
    let layout = cx.layout();
    gpui::div()
        .p(px(8.))
        .rounded(crate::util::container_radius(cx))
        .bg(colors.overlay.background)
        .text_color(colors.overlay.foreground)
        .when(!layout.overlay_shadow.is_empty(), |e| {
            e.shadow(layout.overlay_shadow.clone())
        })
}

// ---------------------------------------------------------------------------
// DateRangePicker
// ---------------------------------------------------------------------------

/// State entity for [`DateRangePicker`].
pub struct DateRangeState {
    pub view_year: i32,
    pub view_month: u32,
    /// Anchor day for the week and day views; the month view ignores it.
    pub view_day: u32,
    pub start: Option<Date>,
    pub end: Option<Date>,
    /// Live cell under the cursor — drives the hover preview range.
    pub hovered: Option<Date>,
    /// Set once the user pages or picks, after which `selectionAlignment`
    /// stops re-deriving the visible range.
    pub user_navigated: bool,
}

impl DateRangeState {
    pub fn new(_cx: &mut App) -> Self {
        let t = Date::today();
        Self {
            view_year: t.year,
            view_month: t.month,
            view_day: t.day,
            start: None,
            end: None,
            hovered: None,
            user_navigated: false,
        }
    }

    /// `defaultValue` — a state seeded with an initial range.
    pub fn with_range(cx: &mut App, start: Option<Date>, end: Option<Date>) -> Self {
        let mut state = Self::new(cx);
        state.start = start;
        state.end = end;
        state
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

    /// The range's moving edge while the user hovers before picking nd.
    pub fn preview_end(&self) -> Option<Date> {
        if self.end.is_some() {
            self.end
        } else if self.start.is_some() {
            self.hovered
        } else {
            None
        }
    }

    /// Click logic: first click sets start; second click sets end (or restarts
    /// when earlier than start).
    pub fn pick(&mut self, d: Date) {
        self.user_navigated = true;
        match (self.start, self.end) {
            (_, Some(_)) | (None, _) => {
                self.start = Some(d);
                self.end = None;
            }
            (Some(s), None) => {
                if days_from_civil(&d) < days_from_civil(&s) {
                    self.start = Some(d);
                } else {
                    self.end = Some(d);
                }
            }
        }
    }
}

/// HeroUI DateRangePicker.
#[derive(IntoElement)]
pub struct DateRangePicker {
    /// `startName` / `endName` — read back by
    /// [`DateRangePicker::form_fields`].
    start_name: Option<SharedString>,
    end_name: Option<SharedString>,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<(Date, Date)>,
    state: Entity<DateRangeState>,
    /// `isOpen` — `None` leaves the picker holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    label: Option<SharedString>,
    placeholder: SharedString,
    is_disabled: bool,
    is_invalid: bool,
    constraints: DateConstraints,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_change: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl DateRangePicker {
    pub fn new(state: Entity<DateRangeState>) -> Self {
        Self {
            start_name: None,
            end_name: None,
            default_value: None,
            state,
            is_open: None,
            default_open: false,
            label: None,
            placeholder: "Select range".into(),
            is_disabled: false,
            is_invalid: false,
            constraints: DateConstraints::new(),
            on_open_change: None,
            on_change: None,
        }
    }

    /// `startName` — the name the range's start submits under.
    pub fn start_name(mut self, name: impl Into<SharedString>) -> Self {
        self.start_name = Some(name.into());
        self
    }

    /// `endName` — the name the range's end submits under.
    pub fn end_name(mut self, name: impl Into<SharedString>) -> Self {
        self.end_name = Some(name.into());
        self
    }

    /// The `Form` fields this picker submits: one per named end of the range,
    /// each written ISO-8601.
    pub fn form_fields(&self, cx: &App) -> Vec<crate::form::FormField> {
        let state = self.state.read(cx);
        let mut out = Vec::new();
        if let Some(name) = self.start_name.clone() {
            let text = state.start.map(|d| d.format_iso()).unwrap_or_default();
            out.push(crate::form::FormField::text_value(name, text));
        }
        if let Some(name) = self.end_name.clone() {
            let text = state.end.map(|d| d.format_iso()).unwrap_or_default();
            out.push(crate::form::FormField::text_value(name, text));
        }
        out
    }

    /// `defaultValue` — the uncontrolled initial range.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: (Date, Date)) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The trigger text shown before a range is picked.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `value` — writes the range through to the bound state.
    pub fn value(self, start: Option<Date>, end: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |st, _| {
            st.start = start;
            st.end = end;
        });
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

    /// All the date constraints at once.
    pub fn constraints(mut self, constraints: DateConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// `onOpenChange` — reports the popover toggling, including trigger clicks.
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }
    /// `defaultOpen` — the uncontrolled initial popover state.
    ///
    /// Only consulted when `is_open` is not supplied; the picker then owns the
    /// flag and its trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    /// Fired after any pick (read `start`/`end` from the entity).
    pub fn on_change(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for DateRangePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!(
                        "daterangepicker-default-{}",
                        self.state.entity_id().as_u64()
                    )
                    .into(),
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

        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("drp-{}-open", self.state.entity_id().as_u64()).into()),
            self.is_open,
            self.default_open,
        );

        let colors = cx.colors();
        let (start, end) = {
            let st = self.state.read(cx);
            (st.start, st.end)
        };
        let label_text = match (start, end) {
            (Some(s), Some(e)) => format!("{} \u{2013} {}", s.format_iso(), e.format_iso()),
            (Some(s), None) => format!("{} \u{2013} \u{2026}", s.format_iso()),
            _ => self.placeholder.to_string(),
        };
        let has_range = start.is_some();

        let mut field = gpui::div()
            .id(gpui::ElementId::Name(
                format!("drp-{}", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .gap(px(8.))
            .w_full()
            .h(crate::util::FIELD_HEIGHT)
            .px(px(12.))
            .text_size(crate::util::FIELD_TEXT);

        field = crate::util::apply_field_chrome(
            field,
            herogpui_core::FieldVariant::Primary,
            self.is_invalid,
            is_open,
            cx,
        );

        if !self.is_disabled {
            let hover_bg = colors.field.hover();
            field = field.cursor_pointer();
            if !is_open {
                field = field.hover(move |s| s.bg(hover_bg));
            }
            // The trigger previously had no click handler at all, so nothing
            // could open the popover.
            if self.on_open_change.is_some() || open_own.is_some() {
                let cb = self.on_open_change.clone();
                let own = open_own;
                let next = !is_open;
                field = field.on_click(move |_, window, cx| {
                    // Uncontrolled: flip our own copy too.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    if let Some(f) = &cb {
                        f(next, window, cx);
                    }
                });
            }
        }

        field = field
            .child(
                gpui::div()
                    .flex_1()
                    .text_color(if has_range {
                        colors.foreground
                    } else {
                        colors.default.color
                    })
                    .child(label_text),
            )
            .child(
                gpui::svg()
                    .size(px(15.))
                    .path(icons::ARROW_RIGHT)
                    .text_color(colors.default.color),
            );

        let mut root = gpui::div()
            .relative()
            .w_full()
            .max_w(px(320.))
            .flex()
            .flex_col()
            .gap(px(4.));
        if let Some(label) = &self.label {
            root = root.child(crate::field::Label::new(label.clone()));
        }
        root = root.child(field);

        if is_open && !self.is_disabled {
            // Driving RangeCalendar keeps the hover preview, the constraints
            // and the year picker in one place instead of a second grid.
            let on_change = self.on_change.clone();
            let mut calendar = crate::range_calendar::RangeCalendar::new(self.state.clone())
                .constraints(self.constraints.clone())
                .is_invalid(self.is_invalid);
            if let Some(cb) = on_change {
                calendar = calendar.on_change(move |_s, _e, window, cx| cb(window, cx));
            }
            // A calendar has its own intrinsic width, so the panel must be
            // content-sized; `placed_field_panel` would clamp it to the
            // trigger and the grid would spill outside the surface.
            root = root.child(crate::util::floating(
                crate::util::placed_panel(herogpui_core::Placement::BottomStart, px(6.))
                    .child(picker_panel(cx).child(calendar)),
            ));
        }

        if self.is_disabled {
            root = root.opacity(cx.layout().disabled_opacity);
        }

        root
    }
}

// ---------------------------------------------------------------------------
// DateField (segmented)
// ---------------------------------------------------------------------------

/// One editable part of a [`DateField`], in en-US reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSegment {
    Month,
    Day,
    Year,
}

impl DateSegment {
    /// The segments a date field shows, in the order it shows them.
    pub const ALL: [DateSegment; 3] = [DateSegment::Month, DateSegment::Day, DateSegment::Year];

    pub fn label(self) -> &'static str {
        match self {
            DateSegment::Month => "month",
            DateSegment::Day => "day",
            DateSegment::Year => "year",
        }
    }

    /// The placeholder this segment shows with no value, sized like its digits.
    fn hint(self) -> &'static str {
        match self {
            DateSegment::Month => "mm",
            DateSegment::Day => "dd",
            DateSegment::Year => "yyyy",
        }
    }

    /// How many digits this segment holds — the point at which typing moves on.
    fn digits(self) -> usize {
        match self {
            DateSegment::Year => 4,
            _ => 2,
        }
    }

    /// `date` with this segment set to `value`, clamped to what the calendar
    /// allows (February 31st becomes the 28th or 29th).
    fn with_value(self, date: Date, value: u32) -> Date {
        match self {
            DateSegment::Year => {
                let year = value as i32;
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(year, date.month));
                Date::new(year, date.month, day)
            }
            DateSegment::Month => {
                let month = value.clamp(1, 12);
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(date.year, month));
                Date::new(date.year, month, day)
            }
            DateSegment::Day => Date::new(
                date.year,
                date.month,
                value.clamp(1, crate::calendar::days_in_month(date.year, date.month)),
            ),
        }
    }

    /// `date` with this segment moved by `delta`, keeping the result a real
    /// calendar date (31 January + 1 month is the end of February, not the 31st).
    fn bump(self, date: Date, delta: i32) -> Date {
        match self {
            DateSegment::Year => {
                let year = date.year + delta;
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(year, date.month));
                Date::new(year, date.month, day)
            }
            DateSegment::Month => {
                // Months are 1..=12, so wrap through a zero-based index.
                let zero = date.month as i32 - 1 + delta;
                let year = date.year + zero.div_euclid(12);
                let month = zero.rem_euclid(12) as u32 + 1;
                let day = date.day.min(crate::calendar::days_in_month(year, month));
                Date::new(year, month, day)
            }
            DateSegment::Day => {
                // Stepping past either end of the month rolls into the next.
                let days = crate::calendar::days_in_month(date.year, date.month);
                let target = date.day as i32 + delta;
                if target < 1 {
                    let prev = DateSegment::Month.bump(Date::new(date.year, date.month, 1), -1);
                    let last = crate::calendar::days_in_month(prev.year, prev.month);
                    Date::new(prev.year, prev.month, last)
                } else if target > days as i32 {
                    let next = DateSegment::Month.bump(Date::new(date.year, date.month, 1), 1);
                    Date::new(next.year, next.month, 1)
                } else {
                    Date::new(date.year, date.month, target as u32)
                }
            }
        }
    }
}

/// `granularity` — the smallest unit a date field shows.
///
/// v3 defaults a date to `day`; anything smaller adds the time segments, which
/// is why its own example switches `defaultValue` from `parseDate` to
/// `parseZonedDateTime` when the granularity drops below a day.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Granularity {
    #[default]
    Day,
    Hour,
    Minute,
    Second,
}

impl Granularity {
    pub const ALL: [Granularity; 4] = [
        Granularity::Day,
        Granularity::Hour,
        Granularity::Minute,
        Granularity::Second,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Granularity::Day => "Day",
            Granularity::Hour => "Hour",
            Granularity::Minute => "Minute",
            Granularity::Second => "Second",
        }
    }

    /// The time granularity this asks for, or `None` for a plain date.
    fn time(self) -> Option<crate::time_field::TimeGranularity> {
        use crate::time_field::TimeGranularity as T;
        match self {
            Granularity::Day => None,
            Granularity::Hour => Some(T::Hour),
            Granularity::Minute => Some(T::Minute),
            Granularity::Second => Some(T::Second),
        }
    }
}

/// One editable slot of a date field: a date part, or -- below `day`
/// granularity -- a time part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldSegment {
    Date(DateSegment),
    Time(crate::time_field::TimeSegment),
}

type DateSegmentRender =
    std::sync::Arc<dyn Fn(DateSegment, SharedString) -> gpui::AnyElement + 'static>;

/// v3's DateField: three editable segments (month / day / year), with the ISO
/// text kept in the bound `InputState` so the form and `onChange` still see a
/// plain date string.
#[derive(IntoElement)]
pub struct DateField {
    /// `segment` — v3's render prop for one editable segment,
    /// handed which segment it is and the text the field would show.
    segment: Option<DateSegmentRender>,
    /// `validationBehavior` — written into the text state on render.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    /// `defaultValue` — seeds the text state on the first render only.
    default_value: Option<Date>,
    full_width: bool,
    is_required: bool,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<Option<Date>>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    variant: herogpui_core::FieldVariant,
    constraints: DateConstraints,
    placeholder_value: Option<Date>,
    /// `granularity` — `day`, or a time unit, in which case the field grows the
    /// segments for it and the bound state holds an ISO date-and-time.
    granularity: Granularity,
    /// `hourCycle` — 12- or 24-hour, for the hour segment `granularity` adds.
    hour_cycle: crate::time_field::HourCycle,
    /// `DateField.Prefix` — content before the segments, drawn in the
    /// placeholder colour and inert (`pointer-events-none`).
    prefix: Option<gpui::AnyElement>,
    /// `DateField.Suffix` — content after the segments.
    suffix: Option<gpui::AnyElement>,
    state: Entity<crate::input::InputState>,
    label: Option<SharedString>,
    /// `Description` — composed inside the field in v3's own example.
    description: Option<SharedString>,
    /// `name` — the name this field submits under.
    name: Option<SharedString>,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `shouldForceLeadingZeros` — pad the month and day to two digits.
    should_force_leading_zeros: bool,
    is_disabled: bool,
    is_read_only: bool,
    on_change: Option<OnChange>,
}

impl DateField {
    /// `fullWidth`
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// `isRequired`
    /// `Description` — help text under the field.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// `name` — the name this field submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        Some(
            crate::form::FormField::text(self.state.clone())
                .name(name)
                .is_required(self.is_required),
        )
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `shouldForceLeadingZeros` — whether the month and day are padded to two
    /// digits. On by default, which is what the `MM/DD/YYYY` hint promises.
    pub fn should_force_leading_zeros(mut self, v: bool) -> Self {
        self.should_force_leading_zeros = v;
        self
    }

    /// `isDisabled` — greys the field out and stops it answering keys.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isReadOnly` — shows the value but refuses edits.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `placeholderValue` — the date the empty field formats its hint from.
    pub fn placeholder_value(mut self, date: Date) -> Self {
        self.placeholder_value = Some(date);
        self
    }

    /// `granularity` — the smallest unit the field shows.
    ///
    /// Below `day` the field grows the time segments and the bound state holds
    /// an ISO date-and-time (`2025-02-03T08:45`), which is the value a form
    /// submits; `on_change` still reports the date part.
    pub fn granularity(mut self, granularity: Granularity) -> Self {
        self.granularity = granularity;
        self
    }

    /// `hourCycle` — whether the hour segment `granularity` adds is 12- or
    /// 24-hour.
    pub fn hour_cycle(mut self, cycle: crate::time_field::HourCycle) -> Self {
        self.hour_cycle = cycle;
        self
    }

    /// `DateField.Prefix` — content before the segments.
    pub fn prefix(mut self, el: impl IntoElement) -> Self {
        self.prefix = Some(el.into_any_element());
        self
    }

    /// `DateField.Suffix` — content after the segments.
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    /// `validate` — returns the message to show, or `None` when the date is fine.
    ///
    /// The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&Option<Date>) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    /// `isInvalid` — forces the danger treatment regardless of the text.
    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `variant` — `Secondary` drops the field shadow.
    pub fn variant(mut self, variant: herogpui_core::FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `value` — writes the date through to the bound text state as ISO.
    pub fn value(self, date: Option<Date>, cx: &mut App) -> Self {
        let text = date.map(|d| d.format_iso()).unwrap_or_default();
        self.state.update(cx, |st, _| st.set_value(text));
        self
    }

    /// `minValue` — the earliest date the field accepts.
    pub fn min_value(mut self, date: Date) -> Self {
        self.constraints.min_value = Some(date);
        self
    }

    /// `maxValue` — the latest date the field accepts.
    pub fn max_value(mut self, date: Date) -> Self {
        self.constraints.max_value = Some(date);
        self
    }

    /// `isDateUnavailable` — rejects individual dates inside the range.
    pub fn is_date_unavailable(mut self, f: impl Fn(Date) -> bool + 'static) -> Self {
        self.constraints.is_date_unavailable = Some(std::sync::Arc::new(f));
        self
    }

    /// All the date constraints at once, for callers that already hold a set.
    pub fn constraints(mut self, constraints: DateConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn new(state: Entity<crate::input::InputState>) -> Self {
        Self {
            segment: None,
            granularity: Granularity::Day,
            hour_cycle: crate::time_field::HourCycle::H24,
            validation_behavior: None,
            default_value: None,
            full_width: false,
            is_required: false,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            variant: herogpui_core::FieldVariant::Primary,
            constraints: DateConstraints::new(),
            placeholder_value: None,
            prefix: None,
            suffix: None,
            state,
            label: None,
            description: None,
            name: None,
            auto_focus: false,
            // v3 defaults this on for the en-US order this port formats in.
            should_force_leading_zeros: true,
            is_disabled: false,
            is_read_only: false,
            on_change: None,
        }
    }

    /// `segment` — replaces the contents of each editable segment.
    ///
    /// The closure receives which [`DateSegment`] it is drawing and the text the
    /// field would have shown, the values v3 passes into the same render prop.
    pub fn segment(
        mut self,
        render: impl Fn(DateSegment, SharedString) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.segment = Some(std::sync::Arc::new(render));
        self
    }

    /// `validationBehavior` — see [`crate::input::Input::validation_behavior`].
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// `defaultValue` — the uncontrolled initial date.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: Date) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<Date>, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

fn parse_iso(text: &str) -> Option<Date> {
    let parts: Vec<&str> = text.trim().split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    let d: u32 = parts[2].parse().ok()?;
    if !(1..=12).contains(&m) || d == 0 || d > crate::calendar::days_in_month(y, m) {
        return None;
    }
    Some(Date::new(y, m, d))
}

/// The format a field of this granularity accepts, which is the description v3
/// shows when the caller supplies none of their own.
fn format_hint(granularity: Granularity, twelve_hour: bool) -> String {
    let mut hint = String::from("MM/DD/YYYY");
    match granularity {
        Granularity::Day => return hint,
        Granularity::Hour => hint.push_str(", HH"),
        Granularity::Minute => hint.push_str(", HH:MM"),
        Granularity::Second => hint.push_str(", HH:MM:SS"),
    }
    if twelve_hour {
        hint.push_str(" AM");
    }
    hint
}

/// A date field's value: the date, and the time when `granularity` asks for
/// one. `T` or a space separates them, as ISO 8601 allows both.
fn parse_value(text: &str) -> (Option<Date>, Option<crate::time_field::Time>) {
    let text = text.trim();
    let (date, time) = match text.split_once(['T', ' ']) {
        Some((d, t)) => (d, Some(t)),
        None => (text, None),
    };
    (parse_iso(date), time.and_then(parse_time))
}

/// `HH`, `HH:MM` or `HH:MM:SS`. A missing part is zero, which is what makes an
/// hour-granularity value round-trip.
fn parse_time(text: &str) -> Option<crate::time_field::Time> {
    let mut parts = text.trim().split(':');
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = match parts.next() {
        Some(m) => m.parse().ok()?,
        None => 0,
    };
    let second: u32 = match parts.next() {
        Some(sec) => sec.parse().ok()?,
        None => 0,
    };
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(crate::time_field::Time::new(hour, minute).with_second(second))
}

/// The text the state holds, which is also what a form submits: an ISO date,
/// widened by exactly as much time as the granularity shows.
fn format_value(
    date: Date,
    time: Option<crate::time_field::Time>,
    granularity: Granularity,
) -> String {
    let t = time.unwrap_or_default();
    match granularity {
        Granularity::Day => date.format_iso(),
        Granularity::Hour => format!("{}T{:02}", date.format_iso(), t.hour),
        Granularity::Minute => format!("{}T{:02}:{:02}", date.format_iso(), t.hour, t.minute),
        Granularity::Second => format!(
            "{}T{:02}:{:02}:{:02}",
            date.format_iso(),
            t.hour,
            t.minute,
            t.second
        ),
    }
}

impl RenderOnce for DateField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `validationBehavior` travels with the name, on the text state.
        if let Some(behavior) = self.validation_behavior {
            if self.state.read(cx).validation_behavior() != behavior {
                self.state
                    .update(cx, |s, _| s.set_validation_behavior(behavior));
            }
        }
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("datefield-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(value.format_iso());
                        cx.notify();
                    });
                },
            );
        }

        let entity_id = self.state.entity_id().as_u64();
        // Which segment the arrows and typing act on. `use_keyed_state` takes
        // `cx` mutably, so this precedes the theme tokens.
        let focused_seg = window.use_keyed_state(
            gpui::ElementId::Name(format!("datefield-{entity_id}-seg").into()),
            cx,
            |_, _| FieldSegment::Date(DateSegment::Month),
        );
        let mut focused = *focused_seg.read(cx);
        // Digits typed into the focused segment but not yet complete, so `1` in
        // the month segment can still become `12`. Cleared whenever focus moves.
        let typing = window.use_keyed_state(
            gpui::ElementId::Name(format!("datefield-{entity_id}-typing").into()),
            cx,
            |_, _| String::new(),
        );

        let colors = cx.colors().clone();
        let _layout = cx.layout().clone();

        let text = self.state.read(cx).value().to_owned();
        let (parsed, parsed_time) = parse_value(&text);
        let non_empty = !text.trim().is_empty();

        // The slots this field shows: the three date parts, then the time parts
        // `granularity` asks for. `TimeSegment::order` is the time field's own,
        // so a minute field looks the same wherever it appears.
        let twelve_hour = self.hour_cycle == crate::time_field::HourCycle::H12;
        let mut segments: Vec<FieldSegment> = DateSegment::ALL
            .iter()
            .copied()
            .map(FieldSegment::Date)
            .collect();
        if let Some(time_granularity) = self.granularity.time() {
            segments.extend(
                crate::time_field::TimeSegment::order(time_granularity, twelve_hour)
                    .into_iter()
                    .map(FieldSegment::Time),
            );
        }
        // A narrower granularity can leave the caret on a slot that is gone.
        if !segments.contains(&focused) {
            focused = segments[0];
        }
        let granularity = self.granularity;

        // The three ways a date can be wrong are reported separately, as the
        // calendar grids distinguish them too.
        let rejection = if !non_empty {
            None
        } else if parsed.is_none() {
            Some("Enter a valid date.".to_owned())
        } else {
            let date = parsed.expect("checked above");
            if self.constraints.out_of_range(date) {
                Some(
                    match (self.constraints.min_value, self.constraints.max_value) {
                        (Some(min), Some(max)) => format!(
                            "Pick a date between {} and {}.",
                            min.format_iso(),
                            max.format_iso()
                        ),
                        (Some(min), None) => format!("Pick {} or later.", min.format_iso()),
                        (None, Some(max)) => format!("Pick {} or earlier.", max.format_iso()),
                        (None, None) => "Date is out of range.".to_owned(),
                    },
                )
            } else if self.constraints.is_unavailable(date) {
                Some("That date is unavailable.".to_owned())
            } else {
                None
            }
        };

        // v3 order: the controlled flag, then server errors, then `validate`,
        // then whichever constraint the date breaks.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&parsed)),
            rejection.map(Into::into),
        );
        let is_invalid = validity.is_invalid;

        let pad = self.should_force_leading_zeros;
        let segment_text = move |segment: FieldSegment| -> String {
            use crate::time_field::TimeSegment as T;
            match segment {
                FieldSegment::Date(segment) => {
                    let Some(d) = parsed else {
                        return segment.hint().to_owned();
                    };
                    match segment {
                        DateSegment::Month if pad => format!("{:02}", d.month),
                        DateSegment::Day if pad => format!("{:02}", d.day),
                        DateSegment::Month => d.month.to_string(),
                        DateSegment::Day => d.day.to_string(),
                        DateSegment::Year => format!("{:04}", d.year),
                    }
                }
                FieldSegment::Time(segment) => {
                    let Some(t) = parsed_time else {
                        return if segment == T::Meridiem {
                            "AM".to_owned()
                        } else {
                            "--".to_owned()
                        };
                    };
                    match segment {
                        T::Hour if twelve_hour => format!("{:02}", t.twelve_hour().0),
                        T::Hour => format!("{:02}", t.hour),
                        T::Minute => format!("{:02}", t.minute),
                        T::Second => format!("{:02}", t.second),
                        T::Meridiem => t.twelve_hour().1.to_owned(),
                    }
                }
            }
        };

        // An empty field seeds from `placeholderValue`, the way v3 does, so the
        // first arrow press lands on a sensible date instead of jumping a step
        // from nothing.
        let seed = self.placeholder_value.unwrap_or_else(Date::today);
        let focus_handle = self.state.read(cx).focus_handle.clone();
        if self.auto_focus {
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(format!("datefield-{entity_id}-autofocus").into()),
                &focus_handle,
            );
        }

        let mut group = gpui::div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            // `.date-input-group` is `h-9 items-center overflow-hidden` with the
            // segments inside it and `ms-3`/`me-3` on the prefix and suffix.
            .px(px(12.))
            .h(crate::util::FIELD_HEIGHT)
            .overflow_hidden()
            .rounded(crate::util::field_radius(cx))
            .text_size(crate::util::FIELD_TEXT)
            .font_family("Consolas")
            .text_color(colors.field.foreground);

        // v3 drives a date field from the keyboard: the arrows step the focused
        // segment and walk between segments, and digits type into it. Without
        // this the steppers were the only way to change a value at all.
        if !self.is_disabled && !self.is_read_only {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let constraints = self.constraints.clone();
            let held = focused_seg.clone();
            let buffer = typing;
            let fh = focus_handle.clone();
            let slots = segments.clone();
            group = group
                .track_focus(&focus_handle)
                .key_context("DateField")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                    window.focus(&fh);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    // A value is only ever written back whole, so every branch
                    // produces a date -- and, below `day`, a time -- and commits
                    // the pair the same way.
                    let commit = |date: Date,
                                  time: Option<crate::time_field::Time>,
                                  window: &mut Window,
                                  cx: &mut App| {
                        state.update(cx, |s, cx| {
                            s.set_value(format_value(date, time, granularity));
                            cx.notify();
                        });
                        if let Some(cb) = &on_change {
                            cb(Some(date).filter(|d| constraints.allows(*d)), window, cx);
                        }
                    };
                    let seed_time = parsed_time.unwrap_or_default();
                    match key {
                        "up" | "down" => {
                            let delta = if key == "up" { 1 } else { -1 };
                            let base = parsed.unwrap_or(seed);
                            buffer.update(cx, |b, _| b.clear());
                            // The first press on an empty field takes the seed
                            // itself rather than stepping past it.
                            match focused {
                                FieldSegment::Date(segment) => {
                                    let next = match parsed {
                                        Some(_) => segment.bump(base, delta),
                                        None => base,
                                    };
                                    commit(next, parsed_time, window, cx);
                                }
                                FieldSegment::Time(segment) => {
                                    let next = match parsed_time {
                                        Some(t) => t.bump(segment, delta),
                                        None => seed_time,
                                    };
                                    commit(base, Some(next), window, cx);
                                }
                            }
                        }
                        // The meridiem answers its own letters, as v3's does.
                        "a" | "p"
                            if focused
                                == FieldSegment::Time(crate::time_field::TimeSegment::Meridiem) =>
                        {
                            let hour = seed_time.hour % 12 + if key == "p" { 12 } else { 0 };
                            let next = crate::time_field::Time::new(hour, seed_time.minute)
                                .with_second(seed_time.second);
                            commit(parsed.unwrap_or(seed), Some(next), window, cx);
                        }
                        "left" | "right" => {
                            let delta = if key == "right" { 1 } else { -1 };
                            buffer.update(cx, |b, _| b.clear());
                            let here = slots.iter().position(|s| *s == focused).unwrap_or(0) as i32;
                            let next = (here + delta).clamp(0, slots.len() as i32 - 1) as usize;
                            let next = slots[next];
                            held.update(cx, |seg, cx| {
                                *seg = next;
                                cx.notify();
                            });
                        }
                        "backspace" | "delete" => {
                            buffer.update(cx, |b, _| b.clear());
                            state.update(cx, |s, cx| {
                                s.set_value(String::new());
                                cx.notify();
                            });
                            if let Some(cb) = &on_change {
                                cb(None, window, cx);
                            }
                        }
                        digit if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) => {
                            let digits = match focused {
                                FieldSegment::Date(segment) => segment.digits(),
                                FieldSegment::Time(segment) => segment.digits(),
                            };
                            if digits == 0 {
                                return;
                            }
                            let text = buffer.update(cx, |b, _| {
                                if b.len() >= digits {
                                    b.clear();
                                }
                                b.push_str(digit);
                                b.clone()
                            });
                            let Ok(value) = text.parse::<u32>() else {
                                return;
                            };
                            match focused {
                                FieldSegment::Date(segment) => commit(
                                    segment.with_value(parsed.unwrap_or(seed), value),
                                    parsed_time,
                                    window,
                                    cx,
                                ),
                                FieldSegment::Time(segment) => commit(
                                    parsed.unwrap_or(seed),
                                    Some(segment.with_value(seed_time, value, twelve_hour)),
                                    window,
                                    cx,
                                ),
                            }
                            // A full segment hands the caret on, which is what
                            // makes `12252025` type a whole date.
                            if text.len() >= digits {
                                buffer.update(cx, |b, _| b.clear());
                                let here =
                                    slots.iter().position(|s| *s == focused).unwrap_or(0) as i32;
                                let next = (here + 1).clamp(0, slots.len() as i32 - 1) as usize;
                                let next = slots[next];
                                held.update(cx, |seg, cx| {
                                    *seg = next;
                                    cx.notify();
                                });
                            }
                        }
                        _ => {}
                    }
                });
        }

        group = crate::util::apply_field_chrome(group, self.variant, is_invalid, false, cx);
        if self.full_width {
            group = group.w_full();
        }

        // `.date-input-group__prefix` is `ms-3 me-0`; the shell's own `px-3`
        // already provides that inset, so the slot only needs to sit inline and
        // inherit the placeholder colour.
        if let Some(prefix) = self.prefix {
            group = group.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .mr(px(4.))
                    .text_color(colors.field.placeholder)
                    .child(prefix),
            );
        }

        for (index, segment) in segments.iter().copied().enumerate() {
            // `/` between the date parts, `:` between the time parts, a comma
            // where the two meet and a space before the meridiem -- the order
            // v3 formats `02/03/2025, 08:45 AM` in.
            use crate::time_field::TimeSegment as T;
            let separator = match (index, segment) {
                (0, _) => None,
                (_, FieldSegment::Date(_)) => Some("/"),
                (_, FieldSegment::Time(T::Hour)) => Some(","),
                (_, FieldSegment::Time(T::Meridiem)) => Some(" "),
                (_, FieldSegment::Time(_)) => Some(":"),
            };
            if let Some(separator) = separator {
                group = group.child(
                    gpui::div()
                        .text_color(colors.muted)
                        .child(separator)
                        .when(separator == ",", |el| el.mr(px(2.))),
                );
            }

            let mut seg = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("date-{entity_id}-seg-{index}").into(),
                ))
                .px(px(4.))
                .py(px(1.))
                .rounded(px(4.))
                // `segment` is v3's render prop on `DateField.Segment`: the
                // closure is handed which segment it is drawing.
                .child(match (&self.segment, segment) {
                    // v3's render prop names a *date* segment; a time slot has
                    // none to hand over, so it draws itself.
                    (Some(render), FieldSegment::Date(part)) => {
                        render(part, segment_text(segment).into())
                    }
                    _ => segment_text(segment).into_any_element(),
                });

            if parsed.is_none() {
                seg = seg.text_color(colors.muted);
            }
            if focused == segment {
                seg = seg
                    .bg(colors.accent.soft())
                    .text_color(colors.accent.soft_foreground());
            }

            let held = focused_seg.clone();
            seg = seg.cursor_pointer().on_click(move |_, _, cx| {
                held.update(cx, |s, cx| {
                    *s = segment;
                    cx.notify();
                });
            });

            group = group.child(seg);
        }

        if let Some(suffix) = self.suffix {
            group = group.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .ml(px(4.))
                    .text_color(colors.field.placeholder)
                    .child(suffix),
            );
        }

        // Steppers move whichever segment is focused. v3 has no stepper on a
        // date field -- it expects the arrow keys, which now work -- so these
        // are a pointer affordance, kept inside the shell where v3 puts its
        // `__suffix` rather than floating outside it.
        let mut steppers = gpui::div().flex().flex_col().ml(px(4.)).flex_shrink_0();
        for (icon, delta, key) in [
            (icons::CHEVRON_UP, 1i32, "up"),
            (icons::CHEVRON_DOWN, -1i32, "down"),
        ] {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let constraints = self.constraints.clone();
            let current = parsed;
            steppers = steppers.child(
                gpui::div()
                    .id(gpui::ElementId::Name(
                        format!("date-{entity_id}-{key}").into(),
                    ))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(14.))
                    .cursor_pointer()
                    .text_color(colors.muted)
                    .child(
                        gpui::svg()
                            .size(px(10.))
                            .path(icon)
                            .text_color(colors.muted),
                    )
                    .on_click(move |_, window, cx| {
                        let base = current.unwrap_or(seed);
                        // An empty field takes the seed itself on the first
                        // press, so one click does not jump a whole step.
                        let (date, time) = match focused {
                            FieldSegment::Date(part) => (
                                match current {
                                    Some(_) => part.bump(base, delta),
                                    None => base,
                                },
                                parsed_time,
                            ),
                            FieldSegment::Time(part) => (
                                base,
                                Some(match parsed_time {
                                    Some(t) => t.bump(part, delta),
                                    None => crate::time_field::Time::default(),
                                }),
                            ),
                        };
                        state.update(cx, |s, cx| {
                            s.set_value(format_value(date, time, granularity));
                            cx.notify();
                        });
                        if let Some(cb) = &on_change {
                            cb(Some(date).filter(|d| constraints.allows(*d)), window, cx);
                        }
                    }),
            );
        }
        let row = group.child(steppers);

        // -- label / description / error wrapper ------------------------------
        let mut el = gpui::div().flex().flex_col().gap(px(4.));
        if !self.full_width {
            el = el.max_w(px(320.));
        } else {
            el = el.w_full();
        }
        if let Some(label) = self.label.clone() {
            el = el.child(
                crate::field::Label::new(label)
                    .is_required(self.is_required)
                    .is_invalid(is_invalid)
                    .is_disabled(self.is_disabled),
            );
        }
        el = el.child(row);
        // The format hint is the description v3 shows when the caller supplies
        // none of their own.
        match validity.first() {
            Some(message) => el = el.child(crate::field::ErrorMessage::new(message)),
            None => {
                let description = self
                    .description
                    .clone()
                    .unwrap_or_else(|| format_hint(granularity, twelve_hour).into());
                el = el.child(crate::field::Description::new(description));
            }
        }
        if self.is_disabled {
            el = el.opacity(cx.layout().disabled_opacity);
        }
        el.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_field::{HourCycle, Time, TimeSegment};

    #[test]
    fn a_day_value_is_a_plain_iso_date() {
        let date = Date::new(2025, 2, 3);
        assert_eq!(format_value(date, None, Granularity::Day), "2025-02-03");
        assert_eq!(parse_value("2025-02-03"), (Some(date), None));
    }

    #[test]
    fn a_time_value_round_trips_at_every_granularity() {
        let date = Date::new(2025, 2, 3);
        let time = Time::new(8, 45).with_second(9);
        for (granularity, text) in [
            (Granularity::Hour, "2025-02-03T08"),
            (Granularity::Minute, "2025-02-03T08:45"),
            (Granularity::Second, "2025-02-03T08:45:09"),
        ] {
            assert_eq!(format_value(date, Some(time), granularity), text);
            let (d, t) = parse_value(text);
            assert_eq!(d, Some(date));
            // Only as much of the time as the granularity wrote comes back.
            let t = t.expect("a time");
            assert_eq!(t.hour, 8);
            assert_eq!(
                (t.minute, t.second),
                match granularity {
                    Granularity::Hour => (0, 0),
                    Granularity::Minute => (45, 0),
                    _ => (45, 9),
                }
            );
        }
    }

    #[test]
    fn a_space_separates_as_well_as_a_t() {
        assert_eq!(
            parse_value("2025-02-03 08:45"),
            parse_value("2025-02-03T08:45")
        );
    }

    #[test]
    fn an_impossible_time_is_no_time() {
        assert_eq!(parse_value("2025-02-03T24:00").1, None);
        assert_eq!(parse_value("2025-02-03T08:60").1, None);
        assert_eq!(parse_value("2025-02-03Tzz").1, None);
    }

    #[test]
    fn granularity_picks_the_time_slots() {
        let slots = |g: Granularity, twelve: bool| {
            g.time()
                .map(|t| TimeSegment::order(t, twelve))
                .unwrap_or_default()
        };
        assert!(slots(Granularity::Day, false).is_empty());
        assert_eq!(slots(Granularity::Hour, false), vec![TimeSegment::Hour]);
        assert_eq!(
            slots(Granularity::Second, false),
            vec![TimeSegment::Hour, TimeSegment::Minute, TimeSegment::Second]
        );
        // A 12-hour clock adds the meridiem, wherever the granularity stops.
        assert_eq!(
            slots(Granularity::Hour, true),
            vec![TimeSegment::Hour, TimeSegment::Meridiem]
        );
    }

    #[test]
    fn the_hint_says_what_the_field_takes() {
        assert_eq!(format_hint(Granularity::Day, false), "MM/DD/YYYY");
        assert_eq!(format_hint(Granularity::Minute, false), "MM/DD/YYYY, HH:MM");
        assert_eq!(
            format_hint(Granularity::Second, true),
            "MM/DD/YYYY, HH:MM:SS AM"
        );
        // The cycle only shows where there is an hour to qualify.
        assert_eq!(format_hint(Granularity::Day, true), "MM/DD/YYYY");
        let _ = HourCycle::H12;
    }
}
