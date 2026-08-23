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

        let sem = cx.colors().accent;
        let colors = cx.colors();
        let layout = cx.layout();

        let selected = self.state.read(cx).selected;
        let h = px(40.);

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
            .rounded(crate::util::field_radius(cx))
            .bg(colors.default.soft())
            .cursor_pointer();

        if !is_open {
            let hover_bg = colors.default.soft_hover();
            field = field.hover(move |s| s.bg(hover_bg));
        } else {
            field = field.border_2().border_color(sem.color);
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
            root = root.child(
                gpui::div()
                    .absolute()
                    .top_full()
                    .left(px(0.))
                    .mt(px(6.))
                    .child(cal),
            );
        }

        root
    }
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
        let accent = if self.is_invalid {
            colors.danger
        } else {
            colors.accent
        };

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
            .text_size(crate::util::FIELD_TEXT)
            .rounded(crate::util::field_radius(cx))
            .bg(colors.default.soft());

        if is_open {
            field = field.border_2().border_color(accent.color);
        } else if self.is_invalid {
            field = field.border_1().border_color(colors.danger.color);
        }

        if !self.is_disabled {
            let hover_bg = colors.default.soft_hover();
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
                    .child(calendar),
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
    state: Entity<crate::input::InputState>,
    label: Option<SharedString>,
    on_change: Option<OnChange>,
}

impl DateField {
    /// `fullWidth`
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// `isRequired`
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `placeholderValue` — the date the empty field formats its hint from.
    pub fn placeholder_value(mut self, date: Date) -> Self {
        self.placeholder_value = Some(date);
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
            state,
            label: None,
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
            |_, _| DateSegment::Month,
        );
        let focused = *focused_seg.read(cx);

        let colors = cx.colors().clone();
        let layout = cx.layout().clone();

        let text = self.state.read(cx).value().to_owned();
        let parsed = parse_iso(&text);
        let non_empty = !text.trim().is_empty();

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

        let segment_text = move |segment: DateSegment| -> String {
            let Some(d) = parsed else {
                return segment.hint().to_owned();
            };
            match segment {
                DateSegment::Month => format!("{:02}", d.month),
                DateSegment::Day => format!("{:02}", d.day),
                DateSegment::Year => format!("{:04}", d.year),
            }
        };

        let mut group = gpui::div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .px(px(12.))
            .h(crate::util::FIELD_HEIGHT)
            .rounded(crate::util::field_radius(cx))
            .text_size(crate::util::FIELD_TEXT)
            .font_family("Consolas")
            .text_color(colors.field.foreground);

        group = match self.variant {
            herogpui_core::FieldVariant::Primary => {
                let shadow = layout.field_shadow;
                group
                    .bg(colors.field.background)
                    .when(!shadow.is_empty(), |e| e.shadow(shadow))
            }
            herogpui_core::FieldVariant::Secondary => group.bg(colors.surface_secondary),
        };
        if is_invalid {
            group = group.border_1().border_color(colors.danger.color);
        }
        if self.full_width {
            group = group.w_full();
        }

        for (index, segment) in DateSegment::ALL.iter().copied().enumerate() {
            if index > 0 {
                group = group.child(gpui::div().text_color(colors.muted).child("/"));
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
                .child(match &self.segment {
                    Some(render) => render(segment, segment_text(segment).into()),
                    None => segment_text(segment).into_any_element(),
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

        // Steppers move whichever segment is focused, seeding an empty field
        // from `placeholderValue` the way v3 does.
        let seed = self.placeholder_value.unwrap_or_else(Date::today);
        let mut steppers = gpui::div().flex().flex_col().ml(px(8.)).flex_shrink_0();
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
                        let next = match current {
                            Some(_) => focused.bump(base, delta),
                            None => base,
                        };
                        state.update(cx, |s, cx| {
                            s.set_value(next.format_iso());
                            cx.notify();
                        });
                        if let Some(cb) = &on_change {
                            cb(Some(next).filter(|d| constraints.allows(*d)), window, cx);
                        }
                    }),
            );
        }
        let row = gpui::div()
            .flex()
            .items_center()
            .child(group)
            .child(steppers);

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
                    .is_invalid(is_invalid),
            );
        }
        el = el.child(row);
        match validity.first() {
            Some(message) => el = el.child(crate::field::ErrorMessage::new(message)),
            None => el = el.child(crate::field::Description::new("MM/DD/YYYY")),
        }
        el.into_any_element()
    }
}
