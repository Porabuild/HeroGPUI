//! DatePicker, DateRangePicker & DateField — port of the v3
//! `@heroui/date-picker` family: a popover calendar plus ISO text entry.
//!
//! All three share [`DateConstraints`] for `minValue` / `maxValue` /
//! `isDateUnavailable` / `firstDayOfWeek`.

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
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
    pub fn on_open_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
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
            let own = open_own.clone();
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
            root = root.child(gpui::div().absolute().top_full().left(px(0.)).mt(px(6.)).child(cal));
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
        if self.end.is_some() { self.end } else if self.start.is_some() { self.hovered } else { None }
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
    pub fn on_open_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
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
                    format!("daterangepicker-default-{}", self.state.entity_id().as_u64()).into(),
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
                let own = open_own.clone();
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
// DateField (ISO text entry)
// ---------------------------------------------------------------------------

/// Simple ISO date text field bound to an `InputState`; emits parsed dates.
#[derive(IntoElement)]
pub struct DateField {
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
    pub fn validate(
        mut self,
        f: impl Fn(&Option<Date>) -> Option<SharedString> + 'static,
    ) -> Self {
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
                self.state.update(cx, |s, _| s.set_validation_behavior(behavior));
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

        let value = self.state.read(cx).value().to_string();
        let parsed = parse_iso(&value);
        let non_empty = !value.trim().is_empty();

        // `placeholderValue` formats the hint, so a caller can show the shape
        // of a real date rather than the literal pattern.
        let hint = match self.placeholder_value {
            Some(d) => d.format_iso(),
            None => "YYYY-MM-DD".to_string(),
        };
        // The three ways a typed date can be wrong are reported separately, as
        // the calendar grids distinguish them too.
        let rejection = if !non_empty {
            None
        } else if parsed.is_none() {
            Some("Enter a valid date as YYYY-MM-DD.".to_string())
        } else {
            let date = parsed.expect("checked above");
            if self.constraints.out_of_range(date) {
                Some(match (self.constraints.min_value, self.constraints.max_value) {
                    (Some(min), Some(max)) => {
                        format!("Pick a date between {} and {}.", min.format_iso(), max.format_iso())
                    }
                    (Some(min), None) => format!("Pick {} or later.", min.format_iso()),
                    (None, Some(max)) => format!("Pick {} or earlier.", max.format_iso()),
                    (None, None) => "Date is out of range.".to_string(),
                })
            } else if self.constraints.is_unavailable(date) {
                Some("That date is unavailable.".to_string())
            } else {
                None
            }
        };

        // v3 order: the controlled flag, then server errors, then `validate`,
        // then whichever constraint the typed date breaks.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&parsed)),
            rejection.clone().map(Into::into),
        );

        let mut input = crate::input::Input::new(self.state.clone())
            .placeholder(hint)
            .variant(self.variant)
            .is_required(self.is_required)
            .is_invalid(validity.is_invalid);
        if self.full_width {
            input = input.full_width();
        }
        if let Some(label) = &self.label {
            input = input.label(label.clone());
        }
        match validity.first() {
            Some(message) => input = input.error_message(message),
            None => input = input.description("ISO format: YYYY-MM-DD".to_string()),
        }

        if let Some(on_change) = self.on_change.clone() {
            let st = self.state.clone();
            let constraints = self.constraints.clone();
            input = input.on_change(move |_v: &str, window, cx| {
                let text = st.read(cx).value().to_string();
                // A date the constraints reject is reported as no date, so a
                // caller never receives a value it said it would not accept.
                let date = parse_iso(&text).filter(|d| constraints.allows(*d));
                on_change(date, window, cx);
            });
        }

        input.into_any_element()
    }
}






