//! DatePicker, DateRangePicker & DateField — port of the v3
//! `@heroui/date-picker` family: a popover calendar plus ISO text entry.
//!
//! All three share [`DateConstraints`] for `minValue` / `maxValue` /
//! `isDateUnavailable` / `firstDayOfWeek`.

use gpui::{
    prelude::*, px, App, Entity, Focusable, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::OnceLock,
};

use crate::{
    calendar::{days_from_civil, Calendar, CalendarState, Date},
    date_constraints::{DateConstraints, Weekday},
    icons,
};

type OnChange = std::sync::Arc<dyn Fn(Option<Date>, &mut Window, &mut App) + 'static>;

type DateFieldFormState = Rc<RefCell<crate::form::LiveFormFieldState>>;

thread_local! {
    static DATE_FIELD_FORM_STATES: RefCell<HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
    static DATE_PICKER_FORM_STATES: RefCell<HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
    static DATE_RANGE_PICKER_FORM_STATES: RefCell<HashMap<(u64, bool), std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
}

fn registered_date_field_form_state(entity_id: u64) -> Option<DateFieldFormState> {
    DATE_FIELD_FORM_STATES.with(|states| {
        states
            .borrow()
            .get(&entity_id)
            .and_then(|state| state.upgrade())
    })
}

fn date_field_form_state(entity_id: u64) -> DateFieldFormState {
    DATE_FIELD_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states.get(&entity_id).and_then(|state| state.upgrade()) {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert(entity_id, Rc::downgrade(&state));
        state
    })
}

fn date_picker_form_state(entity_id: u64) -> DateFieldFormState {
    DATE_PICKER_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states.get(&entity_id).and_then(|state| state.upgrade()) {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert(entity_id, Rc::downgrade(&state));
        state
    })
}

fn date_range_picker_form_state(entity_id: u64, end: bool) -> DateFieldFormState {
    DATE_RANGE_PICKER_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states
            .get(&(entity_id, end))
            .and_then(|state| state.upgrade())
        {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert((entity_id, end), Rc::downgrade(&state));
        state
    })
}

#[allow(clippy::arc_with_non_send_sync)]
fn install_date_field_restore(
    form_state: &DateFieldFormState,
    input_state: Entity<crate::input::InputState>,
    default_text: SharedString,
) {
    let restore_state = form_state.clone();
    form_state.borrow_mut().restore = Some(std::sync::Arc::new(move |_, cx| {
        let value = default_text.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(value.to_string());
            cx.notify();
        });
        let mut state = restore_state.borrow_mut();
        state.value = crate::form::FormValue::Text(value);
        state.is_invalid = false;
    }));
}

/// The complete state passed to [`DatePicker::content`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DatePickerRenderState {
    /// The picker cannot receive focus, edit its field, or open its calendar.
    pub is_disabled: bool,
    /// Controlled or constraint validation currently fails.
    pub is_invalid: bool,
    /// The field can be focused but not edited or opened.
    pub is_read_only: bool,
    /// The picker must contain a date before native form submission.
    pub is_required: bool,
    /// A composed child currently owns focus.
    pub is_focus_within: bool,
    /// Focus within the picker was reached through keyboard navigation.
    pub is_focus_visible: bool,
    /// The calendar popover is currently open.
    pub is_open: bool,
}

/// HeroUI DatePicker (controlled open state; selection lives in the entity).
#[derive(IntoElement)]
pub struct DatePicker {
    /// The locale whose calendar system the popover's grid is drawn in, when
    /// the caller names one. Forwarded to the embedded [`Calendar`].
    locale: Option<SharedString>,
    /// v3's children-as-a-function root composition.
    content: Option<std::sync::Arc<dyn Fn(DatePickerRenderState) -> gpui::AnyElement + 'static>>,
    /// `name` — read back by [`DatePicker::form_field`].
    name: Option<SharedString>,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<Date>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    validation_behavior: crate::form::ValidationBehavior,
    validate: Option<crate::validation::Validator<Option<Date>>>,
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    auto_focus: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    state: Entity<CalendarState>,
    /// `isOpen` — `None` leaves the picker holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    should_close_on_select: bool,
    label: Option<SharedString>,
    trigger_indicator: Option<gpui::AnyElement>,
    on_change: Option<OnChange>,
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    form_is_disabled: Rc<Cell<bool>>,
    form_default: Rc<RefCell<Option<Date>>>,
    form_on_change: Rc<RefCell<Option<OnChange>>>,
    form_field_state: Rc<RefCell<Option<Entity<crate::input::InputState>>>>,
}

impl DatePicker {
    /// The locale whose calendar system the popover's grid is drawn in.
    ///
    /// Forwarded to the embedded calendar; see [`crate::Calendar::locale`] for
    /// why this is a builder rather than v3's `I18nProvider`.
    pub fn locale(mut self, tag: impl Into<SharedString>) -> Self {
        self.locale = Some(tag.into());
        self
    }

    /// `value` — writes the selection through to the bound state.
    pub fn value(self, date: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| {
            s.selected = date;
            s.selected_dates = date.into_iter().collect();
            if let Some(date) = date {
                s.view_year = date.year;
                s.view_month = date.month;
                s.view_day = date.day;
            }
        });
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

    /// `isReadOnly` — keeps the field focusable but blocks edits, selection and opening.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `isRequired` — blocks native form submission while the picker is empty.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `validationBehavior` — native errors block submission; ARIA-style errors do not.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// `validate` — returns one custom message for the selected date.
    pub fn validate(mut self, f: impl Fn(&Option<Date>) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — server messages take precedence over custom validation.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `autoFocus` — focuses the editable date field on its first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `onOpenChange`
    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn new(state: Entity<CalendarState>) -> Self {
        let form_state = date_picker_form_state(state.entity_id().as_u64());
        let form_is_disabled = Rc::new(Cell::new(false));
        let form_default = Rc::new(RefCell::new(None));
        let form_on_change: Rc<RefCell<Option<OnChange>>> = Rc::new(RefCell::new(None));
        let form_field_state = Rc::new(RefCell::new(None::<Entity<crate::input::InputState>>));
        let restore_form_state = form_state.clone();
        let restore_is_disabled = form_is_disabled.clone();
        let restore_default = form_default.clone();
        let restore_callback = form_on_change.clone();
        let restore_state = state.clone();
        let restore_field_state = form_field_state.clone();
        // FormField's restore slot is an Arc, but this callback is intentionally
        // confined to the single-threaded GPUI app and captures its live state.
        #[allow(clippy::arc_with_non_send_sync)]
        let restore: std::sync::Arc<dyn Fn(&mut Window, &mut App)> =
            std::sync::Arc::new(move |window, cx| {
                let date = *restore_default.borrow();
                restore_state.update(&mut *cx, |state, cx| {
                    state.selected = date;
                    state.selected_dates = date.into_iter().collect();
                    if let Some(date) = date {
                        state.view_year = date.year;
                        state.view_month = date.month;
                        state.view_day = date.day;
                    }
                    cx.notify();
                });
                if let Some(field_state) = restore_field_state.borrow().clone() {
                    let text = date.map(|date| date.format_iso()).unwrap_or_default();
                    field_state.update(cx, |state, cx| {
                        state.set_value(text);
                        cx.notify();
                    });
                }
                let mut form_state = restore_form_state.borrow_mut();
                form_state.value = crate::form::FormValue::Text(
                    date.map(|date| date.format_iso())
                        .unwrap_or_default()
                        .into(),
                );
                form_state.is_invalid = false;
                form_state.is_successful = !restore_is_disabled.get();
                if let Some(callback) = restore_callback.borrow().as_ref() {
                    callback(date, window, cx);
                }
            });
        form_state.borrow_mut().restore = Some(restore);
        Self {
            locale: None,
            content: None,
            name: None,
            default_value: None,
            constraints: DateConstraints::new(),
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            validation_behavior: crate::form::ValidationBehavior::Native,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            auto_focus: false,
            on_open_change: None,
            state,
            is_open: None,
            default_open: false,
            should_close_on_select: true,
            label: None,
            trigger_indicator: None,
            on_change: None,
            form_state,
            form_is_disabled,
            form_default,
            form_on_change,
            form_field_state,
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
        self.form_is_disabled.set(self.is_disabled);
        let selected = self.state.read(cx).selected();
        let text = selected.map(|d| d.format_iso()).unwrap_or_default();
        if self.form_default.borrow().is_none() {
            *self.form_default.borrow_mut() = self.state.read(cx).selected;
        }
        let mut form_state = self.form_state.borrow_mut();
        form_state.value = crate::form::FormValue::Text(text.into());
        form_state.is_successful = !self.is_disabled;
        let constraint_error = selected
            .is_some_and(|date| !self.constraints.allows(date))
            .then(|| SharedString::from("That date is unavailable."));
        form_state.is_invalid = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate
                .as_ref()
                .and_then(|validate| validate(&selected)),
            constraint_error,
        )
        .is_invalid;
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
                .is_required(self.is_required)
                .validation_behavior(self.validation_behavior),
        )
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: Date) -> Self {
        self.default_value = Some(value);
        *self.form_default.borrow_mut() = Some(value);
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

    /// `shouldCloseOnSelect` — whether a calendar pick dismisses the popover.
    pub fn should_close_on_select(mut self, v: bool) -> Self {
        self.should_close_on_select = v;
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// `DatePicker.TriggerIndicator` — replaces the default calendar glyph.
    pub fn trigger_indicator(mut self, indicator: impl IntoElement) -> Self {
        self.trigger_indicator = Some(indicator.into_any_element());
        self
    }

    /// v3's `children` render function, handed the complete resolved root state.
    pub fn content(
        mut self,
        render: impl Fn(DatePickerRenderState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn on_change(mut self, f: impl Fn(Option<Date>, &mut Window, &mut App) + 'static) -> Self {
        let callback = std::sync::Arc::new(f);
        *self.form_on_change.borrow_mut() = Some(callback.clone());
        self.on_change = Some(callback);
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.form_is_disabled.set(self.is_disabled);
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
        let open = is_open && !self.is_disabled;
        let selected = self.state.read(cx).selected;
        let selected_text = selected.map(|date| date.format_iso()).unwrap_or_default();
        self.form_state.borrow_mut().value =
            crate::form::FormValue::Text(selected_text.clone().into());
        let initial_text = selected_text.clone();
        let field_state = window
            .use_keyed_state(
                gpui::ElementId::Name(
                    format!("dp-{}-field-state", self.state.entity_id().as_u64()).into(),
                ),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, initial_text)),
            )
            .read(cx)
            .clone();
        *self.form_field_state.borrow_mut() = Some(field_state.clone());
        let field_sync = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("dp-{}-field-sync", self.state.entity_id().as_u64()).into(),
            ),
            cx,
            {
                let selected_text = selected_text.clone();
                move |_, _| selected_text
            },
        );
        let field_value = field_state.read(cx).value().to_owned();
        let last_field_value = field_sync.read(cx).clone();
        let field_follows_selection =
            field_value == last_field_value && field_value != selected_text;
        if field_follows_selection {
            field_state.update(cx, |state, _| state.set_value(selected_text.clone()));
        }
        let live_field_value = if field_follows_selection {
            selected_text.clone()
        } else {
            field_value
        };
        if live_field_value == selected_text {
            field_sync.update(cx, |value, _| *value = selected_text.clone());
        }
        let (parsed, constraint_error) = if live_field_value.trim().is_empty() {
            (None, None)
        } else {
            let (parsed, _) = parse_value(&live_field_value);
            let error = match parsed {
                None => Some(SharedString::from("Enter a valid date.")),
                Some(date) if !self.constraints.allows(date) => {
                    Some(SharedString::from("That date is unavailable."))
                }
                Some(_) => None,
            };
            (parsed, error)
        };
        let field_invalid = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate
                .as_ref()
                .and_then(|validate| validate(&parsed)),
            constraint_error,
        )
        .is_invalid;
        {
            let mut state = self.form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(live_field_value.into());
            state.is_successful = !self.is_disabled;
            state.is_invalid = field_invalid;
        }
        let field_focus = field_state.read(cx).focus_handle(cx);
        self.form_state.borrow_mut().focus = Some(field_focus.clone());
        if let Some(render) = self.content.clone() {
            let content_scope = window
                .use_keyed_state(
                    gpui::ElementId::Name(
                        format!("dp-{}-content-focus", self.state.entity_id().as_u64()).into(),
                    ),
                    cx,
                    |_, cx| cx.focus_handle().tab_stop(false),
                )
                .read(cx)
                .clone();
            let is_focus_within = content_scope.contains_focused(window, cx);
            return gpui::div()
                .relative()
                .max_w(px(320.))
                .track_focus(&content_scope)
                .child(render(DatePickerRenderState {
                    is_disabled: self.is_disabled,
                    is_invalid: field_invalid,
                    is_read_only: self.is_read_only,
                    is_required: self.is_required,
                    is_focus_within,
                    is_focus_visible: is_focus_within && crate::util::focus_visible(cx),
                    is_open: open,
                }))
                .into_any_element();
        }
        let (overlay_phase, dismissal_token) = crate::util::overlay_scope(
            window,
            cx,
            gpui::ElementId::Name(format!("dp-{}-overlay", self.state.entity_id().as_u64()).into()),
            open,
            // Pickers have no exit animation; remove the calendar immediately
            // so a chosen cell cannot receive the same press again.
            false,
        );
        let panel_visible = overlay_phase != crate::util::OverlayPhase::Closed;
        let panel_open = overlay_phase == crate::util::OverlayPhase::Open;
        let blur_open_own = open_own.clone();
        let blur_open_change = self.on_open_change.clone();
        let blur_scope = crate::util::close_on_blur(
            window,
            cx,
            &format!("dp-{}", self.state.entity_id().as_u64()),
            open,
            move |window, cx| {
                if let Some(held) = &blur_open_own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(callback) = &blur_open_change {
                    callback(false, window, cx);
                }
            },
        );
        let trigger_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(
                format!("dp-{}-trigger-focus", self.state.entity_id().as_u64()).into(),
            ),
            window,
            cx,
        );
        let initiator = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("dp-{}-initiator", self.state.entity_id().as_u64()).into(),
            ),
            cx,
            |_, _| 0usize,
        );
        let trigger_pressed = Rc::new(Cell::new(false));

        let open_own_keys = open_own.clone();
        let open_cb_keys = self.on_open_change.clone();
        let open_picker = crate::util::shared(move |window: &mut Window, cx: &mut App| {
            if is_open {
                return;
            }
            if let Some(held) = &open_own_keys {
                held.update(cx, |open, cx| {
                    *open = true;
                    cx.notify();
                });
            }
            if let Some(cb) = &open_cb_keys {
                cb(true, window, cx);
            }
        });

        let selected_state = self.state.clone();
        let user_change = self.on_change.clone();
        let form_state = self.form_state.clone();
        let field_forced_invalid = self.is_invalid;
        let was_open = is_open;
        let trigger_pressed_for_capture = trigger_pressed.clone();
        let trigger_open_own = open_own.clone();
        let trigger_open_cb = self.on_open_change.clone();
        let trigger_indicator = self.trigger_indicator.unwrap_or_else(|| {
            gpui::svg()
                .size(px(16.))
                .path(icons::CALENDAR)
                .text_color(cx.colors().muted)
                .into_any_element()
        });
        let mut trigger = gpui::div()
            .id(gpui::ElementId::Name(
                format!("dp-{}-trigger", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.))
            .child(
                gpui::div()
                    // `.date-picker__trigger-indicator` is `size-4` and centers
                    // either the default glyph or caller-supplied content.
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .child(trigger_indicator),
            );
        trigger =
            crate::util::ring_if_focused(trigger, &trigger_focus, true, Vec::new(), window, cx);
        if !self.is_disabled && !self.is_read_only {
            let focus_on_press = trigger_focus.clone();
            let trigger_initiator = initiator.clone();
            trigger = trigger
                .capture_any_mouse_down(move |_, _, cx| {
                    trigger_pressed_for_capture.set(true);
                    let pressed = trigger_pressed_for_capture.clone();
                    cx.defer(move |_| pressed.set(false));
                })
                .track_focus(&trigger_focus)
                .cursor_pointer()
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    window.focus(&focus_on_press, cx);
                    cx.stop_propagation();
                })
                .on_click(move |_, window, cx| {
                    trigger_initiator.update(cx, |part, _| *part = 1);
                    if let Some(held) = &trigger_open_own {
                        held.update(cx, |open, cx| {
                            *open = !was_open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &trigger_open_cb {
                        cb(!was_open, window, cx);
                    }
                });
        }
        let mut date_field = DateField::new(field_state)
            .embedded(false)
            .full_width(true)
            .is_disabled(self.is_disabled)
            .is_read_only(self.is_read_only)
            .is_required(self.is_required)
            .validation_behavior(self.validation_behavior)
            .is_invalid(field_invalid)
            .auto_focus(self.auto_focus)
            .constraints(self.constraints.clone())
            .report_invalid_changes()
            .suffix(trigger);
        let open_from_field = open_picker.clone();
        let field_initiator = initiator.clone();
        let field_constraints = self.constraints.clone();
        let field_validate = self.validate.clone();
        let field_validation_errors = self.validation_errors.clone();
        date_field = date_field
            .on_picker_open(move |window, cx| {
                field_initiator.update(cx, |part, _| *part = 0);
                open_from_field(window, cx);
            })
            .on_change(move |date, window, cx| {
                let valid_date = date.filter(|date| field_constraints.allows(*date));
                let constraint_error = (date.is_some() && valid_date.is_none())
                    .then(|| SharedString::from("That date is unavailable."));
                form_state.borrow_mut().is_invalid = crate::validation::resolve(
                    field_forced_invalid,
                    &field_validation_errors,
                    field_validate
                        .as_ref()
                        .and_then(|validate| validate(&valid_date)),
                    constraint_error,
                )
                .is_invalid;
                selected_state.update(cx, |state, cx| {
                    if date.is_none() || valid_date.is_some() {
                        state.selected = valid_date;
                        state.selected_dates = valid_date.into_iter().collect();
                    }
                    if let Some(date) = valid_date {
                        state.view_year = date.year;
                        state.view_month = date.month;
                        state.view_day = date.day;
                    }
                    cx.notify();
                });
                if date.is_none() || valid_date.is_some() {
                    form_state.borrow_mut().value = crate::form::FormValue::Text(
                        valid_date
                            .map(|date| date.format_iso())
                            .unwrap_or_default()
                            .into(),
                    );
                }
                if date.is_none() || valid_date.is_some() {
                    if let Some(cb) = &user_change {
                        cb(valid_date, window, cx);
                    }
                }
            });

        let field = gpui::div()
            .id(gpui::ElementId::Name(
                format!("dp-{}", self.state.entity_id().as_u64()).into(),
            ))
            .w_full()
            .child(date_field);

        let mut root = gpui::div().relative().max_w(px(320.));
        let mut wrapper = gpui::div().flex().flex_col().gap(px(4.)).w_full();
        if let Some(label) = &self.label {
            wrapper = wrapper.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(self.is_invalid),
            );
        }
        wrapper = wrapper.child(field);
        root = root.child(wrapper);

        if panel_visible {
            // React Aria dismisses the panel on Escape, on a press outside it
            // and once a day is chosen; all three write the same flag. Escape
            // rides on the root, not the panel: focusing the panel would take
            // the arrows away from the calendar grid inside it.
            let close_own = open_own;
            let close_cb = self.on_open_change.clone();
            let restore_field = field_focus;
            let restore_trigger = trigger_focus;
            let restore_part = initiator;
            let close = crate::util::shared(move |window: &mut Window, cx: &mut App| {
                if let Some(held) = &close_own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &close_cb {
                    cb(false, window, cx);
                }
                if *restore_part.read(cx) == 1 {
                    window.focus(&restore_trigger, cx);
                } else {
                    window.focus(&restore_field, cx);
                }
            });

            let mut cal = Calendar::new(self.state.clone())
                .constraints(self.constraints.clone())
                .when_some(self.locale.clone(), |cal, tag| cal.locale(tag))
                .is_disabled(self.is_disabled)
                .is_read_only(self.is_read_only)
                // React Aria moves the focus into the calendar as the popover
                // opens, so the arrows work straight away.
                .autofocus_grid(panel_open)
                .is_invalid(field_invalid);
            // The calendar reports the chosen date; the picker owns the open
            // flag, so closing belongs here, in the picker's own reaction to
            // that report, not inside the calendar (a bare `Calendar` has
            // nothing to close). The pick also fires the caller's `on_change`
            // first, so both events read the same selection.
            let pick_close = close.clone();
            let user_change = self.on_change.clone();
            let form_state = self.form_state.clone();
            let calendar_forced_invalid = self.is_invalid;
            let calendar_validate = self.validate.clone();
            let calendar_validation_errors = self.validation_errors.clone();
            let should_close_on_select = self.should_close_on_select;
            cal = cal.on_change(move |d, window, cx| {
                let mut form_state = form_state.borrow_mut();
                form_state.value = crate::form::FormValue::Text(
                    d.map(|date| date.format_iso()).unwrap_or_default().into(),
                );
                form_state.is_invalid = crate::validation::resolve(
                    calendar_forced_invalid,
                    &calendar_validation_errors,
                    calendar_validate.as_ref().and_then(|validate| validate(&d)),
                    None,
                )
                .is_invalid;
                if let Some(cb) = &user_change {
                    cb(d, window, cx);
                }
                if should_close_on_select {
                    pick_close(window, cx);
                }
            });
            let esc = close.clone();
            root = crate::util::dismiss_on_escape_with_token(
                root,
                dismissal_token.clone(),
                move |window, cx| {
                    esc(window, cx);
                    crate::util::DismissResult::Handled
                },
            );
            let outside_close = close.clone();
            root = root.child(crate::util::floating(
                crate::util::placed_panel(herogpui_core::Placement::BottomStart, px(6.)).child(
                    crate::util::dismiss_on_press_outside_with_token(
                        picker_panel(cx),
                        dismissal_token,
                        move |window, cx| {
                            if trigger_pressed.get() {
                                return crate::util::DismissResult::Declined;
                            }
                            outside_close(window, cx);
                            crate::util::DismissResult::Handled
                        },
                    )
                    .child(cal),
                ),
            ));
        }

        root.track_focus(&blur_scope).into_any_element()
    }
}

/// The popover chrome every picker shares — `.date-picker__popover` is
/// `bg-overlay p-3` at `min(32px, calc(--radius * 2.5))` with `--shadow-overlay`.
///
/// The calendars used to paint this themselves, which put a second panel inside
/// the first one and left a standalone `Calendar` looking like a floating card.
fn picker_panel(cx: &App) -> gpui::Div {
    let colors = cx.colors();
    let layout = cx.layout();
    gpui::div()
        // `.date-picker__popover` and `.date-range-picker__popover` are `p-3`.
        .p(px(12.))
        // The pinned 8px base makes their `min(32px, radius * 2.5)` exactly 20px.
        .rounded(px(20.))
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

    /// The range's moving edge while the user hovers before picking the end.
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

/// State passed to DateRangePicker's v3 children render function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRangePickerRenderState {
    /// The whole picker is unavailable.
    pub is_disabled: bool,
    /// Either range end currently fails validation.
    pub is_invalid: bool,
    /// The range can be focused but not edited.
    pub is_read_only: bool,
    /// Both range ends must contain dates before native submission.
    pub is_required: bool,
    /// A composed child currently owns focus.
    pub is_focus_within: bool,
    /// Focus within the picker was reached through keyboard navigation.
    pub is_focus_visible: bool,
    /// The range calendar popover is currently open.
    pub is_open: bool,
}

/// HeroUI DateRangePicker.
#[derive(IntoElement)]
pub struct DateRangePicker {
    /// The locale whose calendar system the popover's grid is drawn in, when
    /// the caller names one. Forwarded to the embedded [`RangeCalendar`].
    locale: Option<SharedString>,
    content:
        Option<std::sync::Arc<dyn Fn(DateRangePickerRenderState) -> gpui::AnyElement + 'static>>,
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
    should_close_on_select: bool,
    label: Option<SharedString>,
    trigger_indicator: Option<gpui::AnyElement>,
    range_separator: Option<gpui::AnyElement>,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    validation_behavior: crate::form::ValidationBehavior,
    validate: Option<crate::validation::Validator<Option<(Date, Date)>>>,
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    auto_focus: bool,
    constraints: DateConstraints,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_change: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    start_form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    end_form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    form_is_disabled: Rc<Cell<bool>>,
    form_default: Rc<RefCell<Option<(Date, Date)>>>,
    form_on_change: Rc<RefCell<Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>>>,
    start_field_state: Rc<RefCell<Option<Entity<crate::input::InputState>>>>,
    end_field_state: Rc<RefCell<Option<Entity<crate::input::InputState>>>>,
    form_restore: std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl DateRangePicker {
    /// The locale whose calendar system the popover's grid is drawn in.
    ///
    /// Forwarded to the embedded calendar; see [`crate::Calendar::locale`] for
    /// why this is a builder rather than v3's `I18nProvider`.
    pub fn locale(mut self, tag: impl Into<SharedString>) -> Self {
        self.locale = Some(tag.into());
        self
    }

    pub fn new(state: Entity<DateRangeState>) -> Self {
        let entity_id = state.entity_id().as_u64();
        let start_form_state = date_range_picker_form_state(entity_id, false);
        let end_form_state = date_range_picker_form_state(entity_id, true);
        let form_is_disabled = Rc::new(Cell::new(false));
        let form_default = Rc::new(RefCell::new(None));
        let form_on_change: Rc<RefCell<Option<std::sync::Arc<dyn Fn(&mut Window, &mut App)>>>> =
            Rc::new(RefCell::new(None));
        let start_field_state = Rc::new(RefCell::new(None::<Entity<crate::input::InputState>>));
        let end_field_state = Rc::new(RefCell::new(None::<Entity<crate::input::InputState>>));
        let restore_start = start_form_state.clone();
        let restore_end = end_form_state.clone();
        let restore_is_disabled = form_is_disabled.clone();
        let restore_default = form_default.clone();
        let restore_callback = form_on_change.clone();
        let restore_state = state.clone();
        let restore_start_field = start_field_state.clone();
        let restore_end_field = end_field_state.clone();
        // FormField's restore slot is an Arc, but this callback is intentionally
        // confined to the single-threaded GPUI app and captures its live state.
        #[allow(clippy::arc_with_non_send_sync)]
        let restore: std::sync::Arc<dyn Fn(&mut Window, &mut App)> =
            std::sync::Arc::new(move |window, cx| {
                let (start, end) = restore_default
                    .borrow()
                    .map(|range: (Date, Date)| (Some(range.0), Some(range.1)))
                    .unwrap_or((None, None));
                restore_state.update(&mut *cx, |state, cx| {
                    state.start = start;
                    state.end = end;
                    if let Some(date) = start {
                        state.view_year = date.year;
                        state.view_month = date.month;
                        state.view_day = date.day;
                    }
                    cx.notify();
                });
                if let Some(field_state) = restore_start_field.borrow().clone() {
                    let text = start.map(|date| date.format_iso()).unwrap_or_default();
                    field_state.update(cx, |state, cx| {
                        state.set_value(text);
                        cx.notify();
                    });
                }
                if let Some(field_state) = restore_end_field.borrow().clone() {
                    let text = end.map(|date| date.format_iso()).unwrap_or_default();
                    field_state.update(cx, |state, cx| {
                        state.set_value(text);
                        cx.notify();
                    });
                }
                {
                    let mut state = restore_start.borrow_mut();
                    state.value = crate::form::FormValue::Text(
                        start
                            .map(|date| date.format_iso())
                            .unwrap_or_default()
                            .into(),
                    );
                    state.is_invalid = false;
                    state.is_successful = !restore_is_disabled.get();
                }
                {
                    let mut state = restore_end.borrow_mut();
                    state.value = crate::form::FormValue::Text(
                        end.map(|date| date.format_iso()).unwrap_or_default().into(),
                    );
                    state.is_invalid = false;
                    state.is_successful = !restore_is_disabled.get();
                }
                if let Some(callback) = restore_callback.borrow().as_ref() {
                    callback(window, cx);
                }
            });
        start_form_state.borrow_mut().restore = Some(restore.clone());
        Self {
            locale: None,
            content: None,
            start_name: None,
            end_name: None,
            default_value: None,
            state,
            is_open: None,
            default_open: false,
            should_close_on_select: true,
            label: None,
            trigger_indicator: None,
            range_separator: None,
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            validation_behavior: crate::form::ValidationBehavior::Native,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            auto_focus: false,
            constraints: DateConstraints::new(),
            on_open_change: None,
            on_change: None,
            start_form_state,
            end_form_state,
            form_is_disabled,
            form_default,
            form_on_change,
            start_field_state,
            end_field_state,
            form_restore: restore,
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
        self.form_is_disabled.set(self.is_disabled);
        self.start_form_state.borrow_mut().restore =
            self.start_name.as_ref().map(|_| self.form_restore.clone());
        self.end_form_state.borrow_mut().restore = (self.start_name.is_none()
            && self.end_name.is_some())
        .then(|| self.form_restore.clone());
        let state = self.state.read(cx);
        if self.form_default.borrow().is_none() {
            if let (Some(start), Some(end)) = (state.start, state.end) {
                *self.form_default.borrow_mut() = Some((start, end));
            }
        }
        let range = state.start.zip(state.end);
        let custom_error = self.validate.as_ref().and_then(|validate| validate(&range));
        let mut out = Vec::new();
        if let Some(name) = self.start_name.clone() {
            let text = state.start.map(|d| d.format_iso()).unwrap_or_default();
            let mut form_state = self.start_form_state.borrow_mut();
            form_state.value = crate::form::FormValue::Text(text.into());
            form_state.is_successful = !self.is_disabled;
            let constraint_error = state
                .start
                .is_some_and(|date| !self.constraints.allows(date))
                .then(|| SharedString::from("That date is unavailable."));
            form_state.is_invalid = crate::validation::resolve(
                self.is_invalid,
                &self.validation_errors,
                custom_error.clone(),
                constraint_error,
            )
            .is_invalid;
            out.push(
                crate::form::FormField::live(name, self.start_form_state.clone())
                    .is_required(self.is_required)
                    .validation_behavior(self.validation_behavior),
            );
        }
        if let Some(name) = self.end_name.clone() {
            let text = state.end.map(|d| d.format_iso()).unwrap_or_default();
            let mut form_state = self.end_form_state.borrow_mut();
            form_state.value = crate::form::FormValue::Text(text.into());
            form_state.is_successful = !self.is_disabled;
            let constraint_error = state
                .end
                .is_some_and(|date| !self.constraints.allows(date))
                .then(|| SharedString::from("That date is unavailable."));
            form_state.is_invalid = crate::validation::resolve(
                self.is_invalid,
                &self.validation_errors,
                custom_error,
                constraint_error,
            )
            .is_invalid;
            out.push(
                crate::form::FormField::live(name, self.end_form_state.clone())
                    .is_required(self.is_required)
                    .validation_behavior(self.validation_behavior),
            );
        }
        out
    }

    /// `defaultValue` — the uncontrolled initial range.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: (Date, Date)) -> Self {
        self.default_value = Some(value);
        *self.form_default.borrow_mut() = Some(value);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isReadOnly` — keeps both fields focusable but blocks edits, selection and opening.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `validationBehavior` — native errors block submission; ARIA-style errors do not.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// `validate` — returns one custom message for the complete selected range.
    pub fn validate(
        mut self,
        f: impl Fn(&Option<(Date, Date)>) -> Option<SharedString> + 'static,
    ) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — server messages take precedence over custom validation.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `autoFocus` — focuses the editable start field on its first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `value` — writes the range through to the bound state.
    pub fn value(self, start: Option<Date>, end: Option<Date>, cx: &mut App) -> Self {
        self.state.update(cx, |st, _| {
            st.start = start;
            st.end = end;
            if let Some(start) = start {
                st.view_year = start.year;
                st.view_month = start.month;
                st.view_day = start.day;
            }
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

    /// `firstDayOfWeek` — overrides the range calendar's first weekday.
    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.constraints.first_day_of_week = day;
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

    pub fn should_close_on_select(mut self, v: bool) -> Self {
        self.should_close_on_select = v;
        self
    }

    pub fn trigger_indicator(mut self, indicator: impl IntoElement) -> Self {
        self.trigger_indicator = Some(indicator.into_any_element());
        self
    }

    pub fn range_separator(mut self, separator: impl IntoElement) -> Self {
        self.range_separator = Some(separator.into_any_element());
        self
    }

    pub fn content(
        mut self,
        render: impl Fn(DateRangePickerRenderState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

    /// Fired after any pick (read `start`/`end` from the entity).
    pub fn on_change(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        let callback = std::sync::Arc::new(f);
        *self.form_on_change.borrow_mut() = Some(callback.clone());
        self.on_change = Some(callback);
        self
    }
}

impl RenderOnce for DateRangePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.form_is_disabled.set(self.is_disabled);
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
        let open = is_open && !self.is_disabled;
        let (start, end) = {
            let st = self.state.read(cx);
            (st.start, st.end)
        };
        let start_text = start.map(|date| date.format_iso()).unwrap_or_default();
        let end_text = end.map(|date| date.format_iso()).unwrap_or_default();
        let start_initial = start_text.clone();
        let start_field_state = window
            .use_keyed_state(
                gpui::ElementId::Name(
                    format!("drp-{}-start-field", self.state.entity_id().as_u64()).into(),
                ),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, start_initial)),
            )
            .read(cx)
            .clone();
        let end_initial = end_text.clone();
        let end_field_state = window
            .use_keyed_state(
                gpui::ElementId::Name(
                    format!("drp-{}-end-field", self.state.entity_id().as_u64()).into(),
                ),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, end_initial)),
            )
            .read(cx)
            .clone();
        *self.start_field_state.borrow_mut() = Some(start_field_state.clone());
        *self.end_field_state.borrow_mut() = Some(end_field_state.clone());
        let start_sync = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("drp-{}-start-sync", self.state.entity_id().as_u64()).into(),
            ),
            cx,
            {
                let start_text = start_text.clone();
                move |_, _| start_text
            },
        );
        let end_sync = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("drp-{}-end-sync", self.state.entity_id().as_u64()).into(),
            ),
            cx,
            {
                let end_text = end_text.clone();
                move |_, _| end_text
            },
        );
        let start_value = start_field_state.read(cx).value().to_owned();
        let end_value = end_field_state.read(cx).value().to_owned();
        let last_start = start_sync.read(cx).clone();
        let last_end = end_sync.read(cx).clone();
        let start_follows_selection = start_value == last_start && start_value != start_text;
        let end_follows_selection = end_value == last_end && end_value != end_text;
        if start_follows_selection {
            start_field_state.update(cx, |state, _| state.set_value(start_text.clone()));
        }
        if end_follows_selection {
            end_field_state.update(cx, |state, _| state.set_value(end_text.clone()));
        }
        let live_start_value = if start_follows_selection {
            start_text.clone()
        } else {
            start_value
        };
        let live_end_value = if end_follows_selection {
            end_text.clone()
        } else {
            end_value
        };
        if live_start_value == start_text {
            start_sync.update(cx, |value, _| *value = start_text.clone());
        }
        if live_end_value == end_text {
            end_sync.update(cx, |value, _| *value = end_text.clone());
        }
        let parse_live_value = |text: &str| {
            if text.trim().is_empty() {
                (None, None)
            } else {
                let (parsed, _) = parse_value(text);
                let error = match parsed {
                    None => Some(SharedString::from("Enter a valid date.")),
                    Some(date) if !self.constraints.allows(date) => {
                        Some(SharedString::from("That date is unavailable."))
                    }
                    Some(_) => None,
                };
                (parsed, error)
            }
        };
        let (parsed_start, start_constraint_error) = parse_live_value(&live_start_value);
        let (parsed_end, end_constraint_error) = parse_live_value(&live_end_value);
        let range = parsed_start.zip(parsed_end);
        let custom_error = self.validate.as_ref().and_then(|validate| validate(&range));
        let start_invalid = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            custom_error.clone(),
            start_constraint_error,
        )
        .is_invalid;
        let end_invalid = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            custom_error,
            end_constraint_error,
        )
        .is_invalid;
        {
            let mut state = self.start_form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(live_start_value.into());
            state.is_successful = !self.is_disabled;
            state.is_invalid = start_invalid;
        }
        {
            let mut state = self.end_form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(live_end_value.into());
            state.is_successful = !self.is_disabled;
            state.is_invalid = end_invalid;
        }
        let start_focus = start_field_state.read(cx).focus_handle(cx);
        let end_focus = end_field_state.read(cx).focus_handle(cx);
        {
            let mut state = self.start_form_state.borrow_mut();
            state.focus = Some(start_focus.clone());
        }
        {
            let mut state = self.end_form_state.borrow_mut();
            state.focus = Some(end_focus.clone());
        }
        if let Some(render) = self.content.clone() {
            let content_scope = window
                .use_keyed_state(
                    gpui::ElementId::Name(
                        format!("drp-{}-content-focus", self.state.entity_id().as_u64()).into(),
                    ),
                    cx,
                    |_, cx| cx.focus_handle().tab_stop(false),
                )
                .read(cx)
                .clone();
            let is_focus_within = content_scope.contains_focused(window, cx);
            return gpui::div()
                .relative()
                .max_w(px(320.))
                .track_focus(&content_scope)
                .child(render(DateRangePickerRenderState {
                    is_disabled: self.is_disabled,
                    is_invalid: start_invalid || end_invalid,
                    is_read_only: self.is_read_only,
                    is_required: self.is_required,
                    is_focus_within,
                    is_focus_visible: is_focus_within && crate::util::focus_visible(cx),
                    is_open: open,
                }))
                .into_any_element();
        }
        let (overlay_phase, dismissal_token) = crate::util::overlay_scope(
            window,
            cx,
            gpui::ElementId::Name(
                format!("drp-{}-overlay", self.state.entity_id().as_u64()).into(),
            ),
            open,
            // Pickers have no exit animation; remove the calendar immediately
            // so a chosen cell cannot receive the same press again.
            false,
        );
        let panel_visible = overlay_phase != crate::util::OverlayPhase::Closed;
        let panel_open = overlay_phase == crate::util::OverlayPhase::Open;
        let blur_open_own = open_own.clone();
        let blur_open_change = self.on_open_change.clone();
        let blur_scope = crate::util::close_on_blur(
            window,
            cx,
            &format!("drp-{}", self.state.entity_id().as_u64()),
            open,
            move |window, cx| {
                if let Some(held) = &blur_open_own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(callback) = &blur_open_change {
                    callback(false, window, cx);
                }
            },
        );
        let initiator = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("drp-{}-initiator", self.state.entity_id().as_u64()).into(),
            ),
            cx,
            |_, _| 0usize,
        );
        let trigger_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(
                format!("drp-{}-trigger-focus", self.state.entity_id().as_u64()).into(),
            ),
            window,
            cx,
        );
        let colors = cx.colors();
        let trigger_pressed = Rc::new(Cell::new(false));

        let open_from_start_own = open_own.clone();
        let open_from_start_cb = self.on_open_change.clone();
        let start_initiator = initiator.clone();
        let open_from_start = crate::util::shared(move |window: &mut Window, cx: &mut App| {
            start_initiator.update(cx, |part, _| *part = 0);
            if !is_open {
                if let Some(held) = &open_from_start_own {
                    held.update(cx, |open, cx| {
                        *open = true;
                        cx.notify();
                    });
                }
                if let Some(cb) = &open_from_start_cb {
                    cb(true, window, cx);
                }
            }
        });
        let open_from_end_own = open_own.clone();
        let open_from_end_cb = self.on_open_change.clone();
        let end_initiator = initiator.clone();
        let open_from_end = crate::util::shared(move |window: &mut Window, cx: &mut App| {
            end_initiator.update(cx, |part, _| *part = 1);
            if !is_open {
                if let Some(held) = &open_from_end_own {
                    held.update(cx, |open, cx| {
                        *open = true;
                        cx.notify();
                    });
                }
                if let Some(cb) = &open_from_end_cb {
                    cb(true, window, cx);
                }
            }
        });

        let start_state = self.state.clone();
        let start_change = self.on_change.clone();
        let start_open = open_from_start.clone();
        let start_constraints = self.constraints.clone();
        let start_forced_invalid = self.is_invalid;
        let start_form_state = self.start_form_state.clone();
        let start_other_form_state = self.end_form_state.clone();
        let start_validate = self.validate.clone();
        let start_validation_errors = self.validation_errors.clone();
        let start_field = DateField::new(start_field_state)
            .embedded(true)
            .is_disabled(self.is_disabled)
            .is_required(self.is_required)
            .validation_behavior(self.validation_behavior)
            .is_invalid(start_invalid)
            .auto_focus(self.auto_focus)
            .constraints(self.constraints.clone())
            .is_read_only(self.is_read_only)
            .report_invalid_changes()
            .on_picker_open(move |window, cx| start_open(window, cx))
            .on_change(move |date, window, cx| {
                let valid_date = date.filter(|date| start_constraints.allows(*date));
                let end = start_state.read(cx).end;
                let range = valid_date.zip(end);
                let custom_error = start_validate
                    .as_ref()
                    .and_then(|validate| validate(&range));
                let constraint_error = (date.is_some() && valid_date.is_none())
                    .then(|| SharedString::from("That date is unavailable."));
                start_form_state.borrow_mut().is_invalid = crate::validation::resolve(
                    start_forced_invalid,
                    &start_validation_errors,
                    custom_error.clone(),
                    constraint_error,
                )
                .is_invalid;
                let end_constraint_error = end
                    .is_some_and(|date| !start_constraints.allows(date))
                    .then(|| SharedString::from("That date is unavailable."));
                start_other_form_state.borrow_mut().is_invalid = crate::validation::resolve(
                    start_forced_invalid,
                    &start_validation_errors,
                    custom_error,
                    end_constraint_error,
                )
                .is_invalid;
                start_state.update(cx, |state, cx| {
                    if date.is_none() || valid_date.is_some() {
                        state.start = valid_date;
                    }
                    cx.notify();
                });
                if date.is_none() || valid_date.is_some() {
                    start_form_state.borrow_mut().value = crate::form::FormValue::Text(
                        valid_date
                            .map(|date| date.format_iso())
                            .unwrap_or_default()
                            .into(),
                    );
                }
                if date.is_none() || valid_date.is_some() {
                    if let Some(cb) = &start_change {
                        cb(window, cx);
                    }
                }
            });
        let end_state = self.state.clone();
        let end_change = self.on_change.clone();
        let end_open = open_from_end.clone();
        let end_constraints = self.constraints.clone();
        let end_forced_invalid = self.is_invalid;
        let end_form_state = self.end_form_state.clone();
        let end_other_form_state = self.start_form_state.clone();
        let end_validate = self.validate.clone();
        let end_validation_errors = self.validation_errors.clone();
        let end_field = DateField::new(end_field_state)
            .embedded(true)
            .is_disabled(self.is_disabled)
            .is_required(self.is_required)
            .validation_behavior(self.validation_behavior)
            .is_invalid(end_invalid)
            .constraints(self.constraints.clone())
            .is_read_only(self.is_read_only)
            .report_invalid_changes()
            .on_picker_open(move |window, cx| end_open(window, cx))
            .on_change(move |date, window, cx| {
                let valid_date = date.filter(|date| end_constraints.allows(*date));
                let start = end_state.read(cx).start;
                let range = start.zip(valid_date);
                let custom_error = end_validate.as_ref().and_then(|validate| validate(&range));
                let constraint_error = (date.is_some() && valid_date.is_none())
                    .then(|| SharedString::from("That date is unavailable."));
                end_form_state.borrow_mut().is_invalid = crate::validation::resolve(
                    end_forced_invalid,
                    &end_validation_errors,
                    custom_error.clone(),
                    constraint_error,
                )
                .is_invalid;
                let start_constraint_error = start
                    .is_some_and(|date| !end_constraints.allows(date))
                    .then(|| SharedString::from("That date is unavailable."));
                end_other_form_state.borrow_mut().is_invalid = crate::validation::resolve(
                    end_forced_invalid,
                    &end_validation_errors,
                    custom_error,
                    start_constraint_error,
                )
                .is_invalid;
                end_state.update(cx, |state, cx| {
                    if date.is_none() || valid_date.is_some() {
                        state.end = valid_date;
                    }
                    cx.notify();
                });
                if date.is_none() || valid_date.is_some() {
                    end_form_state.borrow_mut().value = crate::form::FormValue::Text(
                        valid_date
                            .map(|date| date.format_iso())
                            .unwrap_or_default()
                            .into(),
                    );
                }
                if date.is_none() || valid_date.is_some() {
                    if let Some(cb) = &end_change {
                        cb(window, cx);
                    }
                }
            });

        let mut field = gpui::div()
            .id(gpui::ElementId::Name(
                format!("drp-{}", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .gap(px(4.))
            .w_full()
            .h(crate::util::FIELD_HEIGHT)
            .px(px(12.))
            .text_size(crate::util::FIELD_TEXT);

        field = crate::util::apply_field_chrome(
            field,
            herogpui_core::FieldVariant::Primary,
            start_invalid || end_invalid,
            start_focus.is_focused(window)
                || end_focus.is_focused(window)
                || trigger_focus.is_focused(window),
            cx,
        );

        if !self.is_disabled && !self.is_read_only {
            let hover_bg = colors.field.hover();
            if !is_open {
                field = field.hover(move |s| s.bg(hover_bg));
            }
        }

        let trigger_indicator = self.trigger_indicator.unwrap_or_else(|| {
            gpui::svg()
                .size(px(16.))
                .path(icons::CALENDAR)
                .text_color(colors.field.placeholder)
                .into_any_element()
        });
        let mut trigger = gpui::div()
            .id(gpui::ElementId::Name(
                format!("drp-{}-trigger", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            .items_center()
            .justify_center()
            .size(px(24.))
            .child(
                gpui::div()
                    // `.date-range-picker__trigger-indicator` is `size-4`.
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .child(trigger_indicator),
            );
        if !self.is_disabled && !self.is_read_only {
            let focus_on_press = trigger_focus.clone();
            let trigger_initiator = initiator.clone();
            let open_own_trigger = open_own.clone();
            let open_cb_trigger = self.on_open_change.clone();
            let trigger_pressed_for_capture = trigger_pressed.clone();
            let was_open = is_open;
            trigger = trigger
                .capture_any_mouse_down(move |_, _, cx| {
                    trigger_pressed_for_capture.set(true);
                    let pressed = trigger_pressed_for_capture.clone();
                    cx.defer(move |_| pressed.set(false));
                })
                .track_focus(&trigger_focus)
                .cursor_pointer()
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    window.focus(&focus_on_press, cx);
                    cx.stop_propagation();
                })
                .on_click(move |_, window, cx| {
                    trigger_initiator.update(cx, |part, _| *part = 2);
                    if let Some(held) = &open_own_trigger {
                        held.update(cx, |open, cx| {
                            *open = !was_open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &open_cb_trigger {
                        cb(!was_open, window, cx);
                    }
                });
        }

        field = field
            .child(start_field)
            .child(
                // `.date-range-picker__range-separator` is `px-1` in
                // `--field-placeholder`.
                gpui::div()
                    .px(px(4.))
                    .text_color(colors.field.placeholder)
                    .child(
                        self.range_separator
                            .unwrap_or_else(|| gpui::div().child(" - ").into_any_element()),
                    ),
            )
            .child(end_field)
            .child(trigger);

        let mut root = gpui::div()
            .relative()
            .w_full()
            .max_w(px(320.))
            .flex()
            .flex_col()
            .gap(px(4.));
        if let Some(label) = &self.label {
            root = root.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_invalid(start_invalid || end_invalid),
            );
        }
        root = root.child(field);

        if panel_visible {
            // A calendar has its own intrinsic width, so the panel must be
            // content-sized; `placed_field_panel` would clamp it to the
            // trigger and the grid would spill outside the surface.
            // Escape on the root, the outside press on the panel: see
            // `DatePicker` above.
            let close_own = open_own;
            let close_cb = self.on_open_change.clone();
            let restore_start = start_focus;
            let restore_end = end_focus;
            let restore_trigger = trigger_focus;
            let restore_part = initiator;
            let close = crate::util::shared(move |window: &mut Window, cx: &mut App| {
                if let Some(held) = &close_own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &close_cb {
                    cb(false, window, cx);
                }
                match *restore_part.read(cx) {
                    1 => window.focus(&restore_end, cx),
                    2 => window.focus(&restore_trigger, cx),
                    _ => window.focus(&restore_start, cx),
                }
            });

            // Driving RangeCalendar keeps the hover preview, the constraints
            // and the year picker in one place instead of a second grid. The
            // picker closes only once the range is complete: the first pick
            // leaves the panel open to choose the end, exactly as React Aria
            // does. RangeCalendar reports only that completed range.
            let pick_close = close.clone();
            let user_change = self.on_change.clone();
            let start_form_state = self.start_form_state.clone();
            let end_form_state = self.end_form_state.clone();
            let calendar_forced_invalid = self.is_invalid;
            let calendar_validate = self.validate.clone();
            let calendar_validation_errors = self.validation_errors.clone();
            let should_close_on_select = self.should_close_on_select;
            let range_state = self.state.clone();
            let mut calendar = crate::range_calendar::RangeCalendar::new(self.state.clone())
                .constraints(self.constraints.clone())
                .when_some(self.locale.clone(), |cal, tag| cal.locale(tag))
                .autofocus_grid(panel_open)
                .is_read_only(self.is_read_only)
                .is_invalid(start_invalid || end_invalid);
            calendar = calendar.on_change(move |_start, _end, window, cx| {
                let state = range_state.read(cx);
                let range = state.start.zip(state.end);
                let custom_error = calendar_validate
                    .as_ref()
                    .and_then(|validate| validate(&range));
                let mut start_form_state = start_form_state.borrow_mut();
                start_form_state.value = crate::form::FormValue::Text(
                    state
                        .start
                        .map(|date| date.format_iso())
                        .unwrap_or_default()
                        .into(),
                );
                start_form_state.is_invalid = crate::validation::resolve(
                    calendar_forced_invalid,
                    &calendar_validation_errors,
                    custom_error.clone(),
                    None,
                )
                .is_invalid;
                drop(start_form_state);
                let mut end_form_state = end_form_state.borrow_mut();
                end_form_state.value = crate::form::FormValue::Text(
                    state
                        .end
                        .map(|date| date.format_iso())
                        .unwrap_or_default()
                        .into(),
                );
                end_form_state.is_invalid = crate::validation::resolve(
                    calendar_forced_invalid,
                    &calendar_validation_errors,
                    custom_error,
                    None,
                )
                .is_invalid;
                if let Some(cb) = &user_change {
                    cb(window, cx);
                }
                if should_close_on_select {
                    pick_close(window, cx);
                }
            });
            let esc = close.clone();
            root = crate::util::dismiss_on_escape_with_token(
                root,
                dismissal_token.clone(),
                move |window, cx| {
                    esc(window, cx);
                    crate::util::DismissResult::Handled
                },
            );
            let outside_close = close.clone();
            root = root.child(crate::util::floating(
                crate::util::placed_panel(herogpui_core::Placement::BottomStart, px(6.)).child(
                    crate::util::dismiss_on_press_outside_with_token(
                        picker_panel(cx),
                        dismissal_token,
                        move |window, cx| {
                            if trigger_pressed.get() {
                                return crate::util::DismissResult::Declined;
                            }
                            outside_close(window, cx);
                            crate::util::DismissResult::Handled
                        },
                    )
                    .child(calendar),
                ),
            ));
        }

        if self.is_disabled {
            root = root.opacity(cx.layout().disabled_opacity);
        }

        root.track_focus(&blur_scope).into_any_element()
    }
}

// ---------------------------------------------------------------------------
// DateField (segmented)
// ---------------------------------------------------------------------------

/// One editable part of a [`DateField`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSegment {
    Month,
    Day,
    Year,
}

impl DateSegment {
    /// All date segments in canonical month/day/year order.
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

    fn page_step(self) -> i32 {
        match self {
            DateSegment::Year => 5,
            DateSegment::Month => 2,
            DateSegment::Day => 7,
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
                let year = cycle_value(date.year, delta, 1, 9999);
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(year, date.month));
                Date::new(year, date.month, day)
            }
            DateSegment::Month => {
                let month = cycle_value(date.month as i32, delta, 1, 12) as u32;
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(date.year, month));
                Date::new(date.year, month, day)
            }
            DateSegment::Day => {
                let days = crate::calendar::days_in_month(date.year, date.month);
                let day = cycle_value(date.day as i32, delta, 1, days as i32) as u32;
                Date::new(date.year, date.month, day)
            }
        }
    }

    fn bound(self, date: Date, maximum: bool) -> Date {
        match (self, maximum) {
            (DateSegment::Year, false) => {
                let day = date.day.min(crate::calendar::days_in_month(1, date.month));
                Date::new(1, date.month, day)
            }
            (DateSegment::Year, true) => {
                let day = date
                    .day
                    .min(crate::calendar::days_in_month(9999, date.month));
                Date::new(9999, date.month, day)
            }
            (DateSegment::Month, false) => {
                let day = date.day.min(crate::calendar::days_in_month(date.year, 1));
                Date::new(date.year, 1, day)
            }
            (DateSegment::Month, true) => {
                let day = date.day.min(crate::calendar::days_in_month(date.year, 12));
                Date::new(date.year, 12, day)
            }
            (DateSegment::Day, false) => Date::new(date.year, date.month, 1),
            (DateSegment::Day, true) => Date::new(
                date.year,
                date.month,
                crate::calendar::days_in_month(date.year, date.month),
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegionalDateFormat {
    order: [DateSegment; 3],
    literals: [String; 4],
    month_has_leading_zero: bool,
    day_has_leading_zero: bool,
}

impl RegionalDateFormat {
    fn for_locale(locale: &str) -> Option<Self> {
        use icu_datetime::{
            fieldsets,
            input::Date as IcuDate,
            options::YearStyle,
            provider::{
                fields::{FieldLength, FieldSymbol},
                pattern::{reference, runtime, PatternItem},
            },
            DateTimeFormatter,
        };
        use icu_locale_core::Locale as IcuLocale;

        let locale = locale.parse::<IcuLocale>().ok()?;
        let formatter = DateTimeFormatter::try_new(
            locale.into(),
            fieldsets::YMD::short().with_year_style(YearStyle::Full),
        )
        .ok()?;
        let formatted = formatter.format(&IcuDate::try_new_iso(2000, 1, 1).ok()?);
        let pattern: runtime::Pattern<'_> = formatted.pattern().into();
        let mut order = Vec::with_capacity(3);
        let mut literals = Vec::with_capacity(4);
        let mut literal = String::new();
        let mut month_has_leading_zero = None;
        let mut day_has_leading_zero = None;
        for item in reference::Pattern::from(&pattern).into_items() {
            let field = match item {
                PatternItem::Literal(ch) => {
                    literal.push(ch);
                    continue;
                }
                PatternItem::Field(field) => field,
            };
            let segment = match field.symbol {
                FieldSymbol::Month(_) => {
                    month_has_leading_zero = Some(field.length == FieldLength::Two);
                    DateSegment::Month
                }
                FieldSymbol::Day(_) => {
                    day_has_leading_zero = Some(field.length == FieldLength::Two);
                    DateSegment::Day
                }
                FieldSymbol::Year(_) => DateSegment::Year,
                _ => return None,
            };
            if order.contains(&segment) {
                return None;
            }
            literals.push(std::mem::take(&mut literal));
            order.push(segment);
        }
        literals.push(literal);
        Some(Self {
            order: order.try_into().ok()?,
            literals: literals.try_into().ok()?,
            month_has_leading_zero: month_has_leading_zero?,
            day_has_leading_zero: day_has_leading_zero?,
        })
    }

    fn for_preferences(locale: &locale_config::Locale) -> Option<Self> {
        locale
            .tags_for("time")
            .find_map(|tag| Self::for_locale(tag.as_ref()))
    }

    fn date_hint(&self) -> String {
        let mut hint = self.literals[0].clone();
        for (index, segment) in self.order.iter().enumerate() {
            hint.push_str(match segment {
                DateSegment::Month => "MM",
                DateSegment::Day => "DD",
                DateSegment::Year => "YYYY",
            });
            hint.push_str(&self.literals[index + 1]);
        }
        hint
    }
}

fn system_date_format() -> &'static RegionalDateFormat {
    static SYSTEM_DATE_FORMAT: OnceLock<RegionalDateFormat> = OnceLock::new();
    SYSTEM_DATE_FORMAT.get_or_init(|| {
        RegionalDateFormat::for_preferences(&locale_config::Locale::user_default()).unwrap_or(
            RegionalDateFormat {
                order: DateSegment::ALL,
                literals: [String::new(), "/".to_owned(), "/".to_owned(), String::new()],
                month_has_leading_zero: false,
                day_has_leading_zero: false,
            },
        )
    })
}

fn cycle_value(value: i32, delta: i32, min: i32, max: i32) -> i32 {
    (value - min + delta).rem_euclid(max - min + 1) + min
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

#[derive(Clone)]
struct DateFieldDisplay {
    date: Option<Date>,
    time: Option<crate::time_field::Time>,
    cleared: Vec<FieldSegment>,
    committed: String,
}

impl DateFieldDisplay {
    fn new(committed: &str, segments: &[FieldSegment]) -> Self {
        let (date, time) = parse_value(committed);
        let mut this = Self {
            date,
            time,
            cleared: Vec::new(),
            committed: committed.to_owned(),
        };
        this.sync_visible_segments(segments);
        this
    }

    fn sync(&mut self, committed: &str, segments: &[FieldSegment]) {
        if self.committed != committed {
            let (date, time) = parse_value(committed);
            self.date = date;
            self.time = time;
            self.cleared.clear();
            self.committed = committed.to_owned();
        }
        self.sync_visible_segments(segments);
    }

    fn sync_visible_segments(&mut self, segments: &[FieldSegment]) {
        self.cleared.retain(|segment| segments.contains(segment));
        for segment in segments {
            let missing = match segment {
                FieldSegment::Date(_) => self.date.is_none(),
                FieldSegment::Time(_) => self.time.is_none(),
            };
            if missing && !self.cleared.contains(segment) {
                self.cleared.push(*segment);
            }
        }
    }

    fn edit(
        &mut self,
        focused: FieldSegment,
        date: Date,
        time: Option<crate::time_field::Time>,
        segments: &[FieldSegment],
        granularity: Granularity,
    ) -> Option<(Date, Option<crate::time_field::Time>, String)> {
        self.date = Some(date);
        self.time = time;
        self.cleared.retain(|segment| *segment != focused);
        if segments
            .iter()
            .any(|segment| self.cleared.contains(segment))
        {
            return None;
        }

        let committed = format_value(date, time, granularity);
        self.committed.clone_from(&committed);
        Some((date, time, committed))
    }

    fn clear(&mut self, focused: FieldSegment, segments: &[FieldSegment]) -> bool {
        if !self.cleared.contains(&focused) {
            self.cleared.push(focused);
        }
        if !segments
            .iter()
            .all(|segment| self.cleared.contains(segment))
        {
            return false;
        }

        self.date = None;
        self.time = None;
        self.committed.clear();
        true
    }
}

type FieldSegmentRender =
    std::sync::Arc<dyn Fn(FieldSegment, SharedString) -> gpui::AnyElement + 'static>;

/// State supplied to v3's DateField children render function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateFieldRenderState {
    /// Whether the field is disabled.
    pub is_disabled: bool,
    /// Whether controlled, server, custom or constraint validation is invalid.
    pub is_invalid: bool,
    /// Whether segments can be focused but not edited.
    pub is_read_only: bool,
    /// Whether the field is required.
    pub is_required: bool,
    /// Whether the field's input owns focus.
    pub is_focused: bool,
    /// Whether focus is inside the field.
    pub is_focus_within: bool,
    /// Whether keyboard-visible focus chrome should be shown.
    pub is_focus_visible: bool,
}

/// v3's DateField: three editable segments (month / day / year), with the ISO
/// text kept in the bound `InputState` so the form and `onChange` still see a
/// plain date string.
#[derive(IntoElement)]
pub struct DateField {
    /// See [`DateField::content`].
    content: Option<std::sync::Arc<dyn Fn(DateFieldRenderState) -> gpui::AnyElement + 'static>>,
    /// `segment` — v3's render prop for one editable date or time segment,
    /// handed which segment it is and the text the field would show.
    segment: Option<FieldSegmentRender>,
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
    /// `shouldForceLeadingZeros` — force month, day and hour to two digits
    /// instead of using the system regional format.
    should_force_leading_zeros: bool,
    is_disabled: bool,
    is_read_only: bool,
    on_change: Option<OnChange>,
    embedded: bool,
    bare: bool,
    on_picker_open: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    report_invalid_changes: bool,
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
        let form_state = date_field_form_state(self.state.entity_id().as_u64());
        form_state.borrow_mut().is_successful = !self.is_disabled;
        if let Some(default) = self.default_value {
            install_date_field_restore(
                &form_state,
                self.state.clone(),
                default.format_iso().into(),
            );
        }
        let mut field =
            crate::form::FormField::live(name, form_state).is_required(self.is_required);
        if let Some(behavior) = self.validation_behavior {
            field = field.validation_behavior(behavior);
        }
        Some(field)
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `shouldForceLeadingZeros` — force month, day and hour to two digits.
    /// Without this flag those segments follow the system regional format;
    /// minute and second segments are always two digits.
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

    /// v3's field `children`-as-a-function, handed the complete
    /// [`DateFieldRenderState`].
    pub fn content(
        mut self,
        render: impl Fn(DateFieldRenderState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn new(state: Entity<crate::input::InputState>) -> Self {
        Self {
            content: None,
            segment: None,
            granularity: Granularity::Day,
            hour_cycle: crate::time_field::HourCycle::default(),
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
            should_force_leading_zeros: false,
            is_disabled: false,
            is_read_only: false,
            on_change: None,
            embedded: false,
            bare: false,
            on_picker_open: None,
            report_invalid_changes: false,
        }
    }

    fn embedded(mut self, bare: bool) -> Self {
        self.embedded = true;
        self.bare = bare;
        self
    }

    fn on_picker_open(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_picker_open = Some(std::sync::Arc::new(f));
        self
    }

    fn report_invalid_changes(mut self) -> Self {
        self.report_invalid_changes = true;
        self
    }

    /// `segment` — replaces the contents of each editable segment.
    ///
    /// The closure receives which [`FieldSegment`] it is drawing and the text
    /// the field would have shown, including time segments below day
    /// granularity.
    pub fn segment(
        mut self,
        render: impl Fn(FieldSegment, SharedString) -> gpui::AnyElement + 'static,
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

/// The regional format a field of this granularity accepts, which is the
/// description v3 shows when the caller supplies none of their own.
fn format_hint(
    regional_date: &RegionalDateFormat,
    regional_time: Option<&crate::time_field::RegionalTimePattern>,
) -> String {
    let mut hint = regional_date.date_hint();
    if let Some(regional_time) = regional_time {
        hint.push_str(", ");
        hint.push_str(&regional_time.hint());
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
        let entity_id = self.state.entity_id().as_u64();
        let form_state = registered_date_field_form_state(entity_id);
        if let Some(form_state) = form_state.as_ref() {
            let mut state = form_state.borrow_mut();
            state.value =
                crate::form::FormValue::Text(self.state.read(cx).value().to_owned().into());
            state.is_successful = !self.is_disabled;
            state.focus = Some(self.state.read(cx).focus_handle.clone());
            if let Some(default) = self.default_value {
                drop(state);
                install_date_field_restore(
                    form_state,
                    self.state.clone(),
                    default.format_iso().into(),
                );
            }
        }
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

        let regional_date = system_date_format();

        // Which segment the arrows and typing act on. `use_keyed_state` takes
        // `cx` mutably, so this precedes the theme tokens.
        let first_date_segment = regional_date.order[0];
        let focused_seg = window.use_keyed_state(
            gpui::ElementId::Name(format!("datefield-{entity_id}-seg").into()),
            cx,
            move |_, _| FieldSegment::Date(first_date_segment),
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
        let navigable = !self.is_disabled;

        let text = self.state.read(cx).value().to_owned();
        let (parsed, _) = parse_value(&text);
        let non_empty = !text.trim().is_empty();

        let twelve_hour = self.hour_cycle == crate::time_field::HourCycle::H12;
        let regional_time = self.granularity.time().map(|granularity| {
            crate::time_field::regional_time_pattern(granularity, self.hour_cycle)
        });
        let mut segments: Vec<FieldSegment> = regional_date
            .order
            .iter()
            .copied()
            .map(FieldSegment::Date)
            .collect();
        if let Some(regional_time) = regional_time.as_ref() {
            segments.extend(regional_time.order.iter().copied().map(FieldSegment::Time));
        }
        // A narrower granularity can leave the caret on a slot that is gone.
        if !segments.contains(&focused) {
            focused = segments[0];
        }
        let granularity = self.granularity;
        let display = window.use_keyed_state(
            gpui::ElementId::Name(format!("datefield-{entity_id}-display").into()),
            cx,
            |_, _| DateFieldDisplay::new(&text, &segments),
        );
        display.update(cx, |display, _| display.sync(&text, &segments));
        let (display_date, display_time, cleared) = {
            let display = display.read(cx);
            (display.date, display.time, display.cleared.clone())
        };

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
        let validity_state = self.state.clone();
        if validity_state.read(cx).validity() != &validity {
            validity_state.update(cx, |state, _| state.set_validity(validity.clone()));
        }
        if let Some(form_state) = form_state.as_ref() {
            let mut state = form_state.borrow_mut();
            state.value = crate::form::FormValue::Text(text.clone().into());
            state.is_invalid = is_invalid;
            state.is_successful = !self.is_disabled;
            state.focus = Some(self.state.read(cx).focus_handle.clone());
        }

        let focus_handle = self.state.read(cx).focus_handle.clone();
        if self.auto_focus {
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(format!("datefield-{entity_id}-autofocus").into()),
                &focus_handle,
            );
        }
        if let Some(render) = self.content.clone() {
            let focused = focus_handle.is_focused(window);
            return render(DateFieldRenderState {
                is_disabled: self.is_disabled,
                is_invalid,
                is_read_only: self.is_read_only,
                is_required: self.is_required,
                is_focused: focused,
                is_focus_within: focus_handle.contains_focused(window, cx),
                is_focus_visible: focused && crate::util::focus_visible(cx),
            })
            .into_any_element();
        }

        let pad_month = self.should_force_leading_zeros || regional_date.month_has_leading_zero;
        let pad_day = self.should_force_leading_zeros || regional_date.day_has_leading_zero;
        let pad_hour = self.should_force_leading_zeros
            || regional_time
                .as_ref()
                .is_some_and(|format| format.hour_has_leading_zero);
        let pad_minute = regional_time
            .as_ref()
            .is_none_or(|format| format.minute_has_leading_zero);
        let pad_second = regional_time
            .as_ref()
            .is_none_or(|format| format.second_has_leading_zero);
        let zero_based_twelve_hour = regional_time
            .as_ref()
            .is_some_and(|format| format.hour_zero_based);
        let am = regional_time
            .as_ref()
            .map_or_else(|| "AM".to_owned(), |format| format.am.clone());
        let pm = regional_time
            .as_ref()
            .map_or_else(|| "PM".to_owned(), |format| format.pm.clone());
        let segment_text = move |segment: FieldSegment| -> String {
            use crate::time_field::TimeSegment as T;
            match segment {
                FieldSegment::Date(segment) => {
                    if cleared.contains(&FieldSegment::Date(segment)) {
                        return segment.hint().to_owned();
                    }
                    let Some(d) = display_date else {
                        return segment.hint().to_owned();
                    };
                    match segment {
                        DateSegment::Month if pad_month => format!("{:02}", d.month),
                        DateSegment::Day if pad_day => format!("{:02}", d.day),
                        DateSegment::Month => d.month.to_string(),
                        DateSegment::Day => d.day.to_string(),
                        DateSegment::Year => format!("{:04}", d.year),
                    }
                }
                FieldSegment::Time(segment) => {
                    if cleared.contains(&FieldSegment::Time(segment)) {
                        return if segment == T::Meridiem {
                            am.clone()
                        } else {
                            "--".to_owned()
                        };
                    }
                    let Some(t) = display_time else {
                        return "--".to_owned();
                    };
                    match segment {
                        T::Hour if twelve_hour && pad_hour => {
                            let hour = if zero_based_twelve_hour {
                                t.hour % 12
                            } else {
                                t.twelve_hour().0
                            };
                            format!("{hour:02}")
                        }
                        T::Hour if twelve_hour => {
                            if zero_based_twelve_hour {
                                (t.hour % 12).to_string()
                            } else {
                                t.twelve_hour().0.to_string()
                            }
                        }
                        T::Hour if pad_hour => format!("{:02}", t.hour),
                        T::Hour => t.hour.to_string(),
                        T::Minute if pad_minute => format!("{:02}", t.minute),
                        T::Minute => t.minute.to_string(),
                        T::Second if pad_second => format!("{:02}", t.second),
                        T::Second => t.second.to_string(),
                        T::Meridiem => {
                            if t.hour < 12 {
                                am.clone()
                            } else {
                                pm.clone()
                            }
                        }
                    }
                }
            }
        };

        // An empty field seeds from `placeholderValue`, the way v3 does, so the
        // first arrow press lands on a sensible date instead of jumping a step
        // from nothing.
        let seed = self.placeholder_value.unwrap_or_else(Date::today);
        let mut group = gpui::div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .text_size(crate::util::FIELD_TEXT)
            .font_family(crate::util::MONO_FONT)
            .text_color(colors.field.foreground)
            .when(!self.bare, |el| {
                // `.date-input-group` is `h-9 items-center overflow-hidden`
                // with the segments inside it.
                el.px(px(12.))
                    .h(crate::util::FIELD_HEIGHT)
                    .overflow_hidden()
                    .rounded(crate::util::field_radius(cx))
            })
            .when(self.bare, |el| el.flex_1().min_w_0());

        // v3 drives a date field from the keyboard: the arrows step the focused
        // segment and walk between segments, and digits type into it. Without
        // this the steppers were the only way to change a value at all.
        if navigable {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let constraints = self.constraints.clone();
            let display = display.clone();
            let held = focused_seg.clone();
            let buffer = typing;
            let fh = focus_handle.clone();
            let slots = segments.clone();
            let is_read_only = self.is_read_only;
            let on_picker_open = self.on_picker_open.clone();
            let report_invalid_changes = self.report_invalid_changes;
            group = group
                .track_focus(&focus_handle)
                .key_context("DateField")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    window.focus(&fh, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if ((event.keystroke.modifiers.alt && matches!(key, "down" | "up"))
                        || key == "space")
                        && on_picker_open.is_some()
                        && !is_read_only
                    {
                        if let Some(open) = &on_picker_open {
                            open(window, cx);
                        }
                        return;
                    }
                    let commit = |focused: FieldSegment,
                                  date: Date,
                                  time: Option<crate::time_field::Time>,
                                  window: &mut Window,
                                  cx: &mut App| {
                        let complete = display.update(cx, |display, cx| {
                            let complete = display.edit(focused, date, time, &slots, granularity);
                            cx.notify();
                            complete
                        });
                        if let Some((date, _, committed)) = complete {
                            state.update(cx, |s, cx| {
                                s.set_value(committed);
                                cx.notify();
                            });
                            if let Some(cb) = &on_change {
                                cb(
                                    if report_invalid_changes {
                                        Some(date)
                                    } else {
                                        Some(date).filter(|d| constraints.allows(*d))
                                    },
                                    window,
                                    cx,
                                );
                            }
                        }
                    };
                    let (display_date, display_time) = {
                        let display = display.read(cx);
                        (display.date, display.time)
                    };
                    let seed_time = display_time.unwrap_or_default();
                    match key {
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
                        _ if is_read_only => {}
                        "up" | "down" | "pageup" | "pagedown" => {
                            let direction = if matches!(key, "up" | "pageup") {
                                1
                            } else {
                                -1
                            };
                            let delta = match focused {
                                FieldSegment::Date(segment)
                                    if matches!(key, "pageup" | "pagedown") =>
                                {
                                    direction * segment.page_step()
                                }
                                _ => direction,
                            };
                            let base = display_date.unwrap_or(seed);
                            buffer.update(cx, |b, _| b.clear());
                            // The first press on an empty field takes the seed
                            // itself rather than stepping past it.
                            match focused {
                                FieldSegment::Date(segment) => {
                                    let next = match display_date {
                                        Some(_) => segment.bump(base, delta),
                                        None => base,
                                    };
                                    commit(focused, next, display_time, window, cx);
                                }
                                FieldSegment::Time(segment) => {
                                    let next = match display_time {
                                        Some(t) => t.bump(segment, delta),
                                        None => seed_time,
                                    };
                                    commit(focused, base, Some(next), window, cx);
                                }
                            }
                        }
                        "home" | "end" => {
                            let maximum = key == "end";
                            let base = display_date.unwrap_or(seed);
                            buffer.update(cx, |b, _| b.clear());
                            match focused {
                                FieldSegment::Date(segment) => commit(
                                    focused,
                                    segment.bound(base, maximum),
                                    display_time,
                                    window,
                                    cx,
                                ),
                                FieldSegment::Time(segment) => {
                                    use crate::time_field::TimeSegment as T;
                                    let next = match segment {
                                        T::Meridiem => {
                                            let hour =
                                                seed_time.hour % 12 + if maximum { 12 } else { 0 };
                                            crate::time_field::Time::new(hour, seed_time.minute)
                                                .with_second(seed_time.second)
                                        }
                                        _ => segment.with_value(
                                            seed_time,
                                            if maximum { u32::MAX } else { 0 },
                                            twelve_hour,
                                            zero_based_twelve_hour,
                                        ),
                                    };
                                    commit(focused, base, Some(next), window, cx);
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
                            commit(
                                focused,
                                display_date.unwrap_or(seed),
                                Some(next),
                                window,
                                cx,
                            );
                        }
                        "backspace" | "delete" => {
                            buffer.update(cx, |b, _| b.clear());
                            let emptied = display.update(cx, |display, cx| {
                                let emptied = display.clear(focused, &slots);
                                cx.notify();
                                emptied
                            });
                            if emptied {
                                state.update(cx, |s, cx| {
                                    s.set_value(String::new());
                                    cx.notify();
                                });
                                if let Some(cb) = &on_change {
                                    cb(None, window, cx);
                                }
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
                                FieldSegment::Date(segment) => {
                                    let base = display_date.unwrap_or(seed);
                                    commit(
                                        focused,
                                        segment.with_value(base, value),
                                        display_time,
                                        window,
                                        cx,
                                    );
                                }
                                FieldSegment::Time(segment) => commit(
                                    focused,
                                    display_date.unwrap_or(seed),
                                    Some(segment.with_value(
                                        seed_time,
                                        value,
                                        twelve_hour,
                                        zero_based_twelve_hour,
                                    )),
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

        if !self.bare {
            group = crate::util::apply_field_chrome(
                group,
                self.variant,
                is_invalid,
                focus_handle.is_focused(window),
                cx,
            );
        }
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
            let separator = match (index, segment) {
                (index, FieldSegment::Date(_)) => Some(regional_date.literals[index].clone()),
                (index, FieldSegment::Time(_)) => {
                    let time_index = index - regional_date.order.len();
                    regional_time.as_ref().and_then(|format| {
                        format.literals.get(time_index).map(|literal| {
                            if time_index == 0 {
                                format!(", {literal}")
                            } else {
                                literal.clone()
                            }
                        })
                    })
                }
            };
            if let Some(separator) = separator.filter(|separator| !separator.is_empty()) {
                group = group.child(gpui::div().text_color(colors.muted).child(separator));
            }

            let mut seg = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("date-{entity_id}-seg-{index}").into(),
                ))
                // `.date-input-group__segment` is `rounded-md px-0.5`.
                .px(px(2.))
                .py(px(1.))
                .rounded(cx.layout().radius_md())
                // `segment` is v3's render prop on `DateField.Segment`: the
                // closure is handed which segment it is drawing.
                .child(match &self.segment {
                    Some(render) => render(segment, segment_text(segment).into()),
                    _ => segment_text(segment).into_any_element(),
                });

            if parsed.is_none() {
                seg = seg.text_color(colors.muted);
            }
            if focused == segment {
                seg = seg
                    .bg(colors.accent.soft())
                    .text_color(colors.accent.soft_foreground(colors.foreground));
            }

            if navigable {
                let held = focused_seg.clone();
                seg = seg.cursor_pointer().on_click(move |_, _, cx| {
                    held.update(cx, |s, cx| {
                        *s = segment;
                        cx.notify();
                    });
                });
            }

            group = group.child(seg);
            if index == regional_date.order.len() - 1 && !regional_date.literals[3].is_empty() {
                group = group.child(
                    gpui::div()
                        .text_color(colors.muted)
                        .child(regional_date.literals[3].clone()),
                );
            }
        }
        if let Some(literal) = regional_time
            .as_ref()
            .and_then(|format| format.literals.last())
            .filter(|literal| !literal.is_empty())
        {
            group = group.child(gpui::div().text_color(colors.muted).child(literal.clone()));
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

        if self.embedded {
            if self.is_disabled {
                group = group.opacity(cx.layout().disabled_opacity);
            }
            return group.into_any_element();
        }

        let row = group;

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
                    .unwrap_or_else(|| format_hint(regional_date, regional_time.as_ref()).into());
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
    use std::process::Command;

    use super::*;
    use crate::time_field::{HourCycle, Time, TimeSegment};

    #[test]
    fn a_day_value_is_a_plain_iso_date() {
        let date = Date::new(2025, 2, 3);
        assert_eq!(format_value(date, None, Granularity::Day), "2025-02-03");
        assert_eq!(parse_value("2025-02-03"), (Some(date), None));
    }

    #[test]
    fn date_order_literals_and_padding_follow_locale_patterns() {
        let us = RegionalDateFormat::for_locale("en-US").unwrap();
        assert_eq!(us.order, DateSegment::ALL);
        assert_eq!(us.literals, ["", "/", "/", ""]);
        assert!(!us.month_has_leading_zero);
        assert!(!us.day_has_leading_zero);

        let gb = RegionalDateFormat::for_locale("en-GB").unwrap();
        assert_eq!(
            gb.order,
            [DateSegment::Day, DateSegment::Month, DateSegment::Year]
        );
        assert_eq!(gb.literals, ["", "/", "/", ""]);
        assert!(gb.month_has_leading_zero);
        assert!(gb.day_has_leading_zero);

        let german = RegionalDateFormat::for_locale("de-DE").unwrap();
        assert_eq!(
            german.order,
            [DateSegment::Day, DateSegment::Month, DateSegment::Year]
        );
        assert_eq!(german.literals, ["", ".", ".", ""]);

        let japanese = RegionalDateFormat::for_locale("ja-JP").unwrap();
        assert_eq!(
            japanese.order,
            [DateSegment::Year, DateSegment::Month, DateSegment::Day]
        );
        assert_eq!(japanese.literals, ["", "/", "/", ""]);
        assert_eq!(RegionalDateFormat::for_locale("not_a_locale"), None);
    }

    #[test]
    fn date_padding_prefers_the_system_time_category() {
        let locale = locale_config::Locale::new("en-US,time=en-GB").unwrap();
        assert_eq!(
            RegionalDateFormat::for_preferences(&locale),
            RegionalDateFormat::for_locale("en-GB")
        );
    }

    #[gpui::test]
    fn date_field_hour_cycle_follows_the_system_time_locale(cx: &mut gpui::TestAppContext) {
        const CHILD: &str = "HEROGPUI_DATE_FIELD_TIME_LOCALE_TEST";
        if std::env::var_os(CHILD).is_none() {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "date_picker::tests::date_field_hour_cycle_follows_the_system_time_locale",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .env_remove("LC_ALL")
                .env("LC_TIME", "en_US.UTF-8")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "12-hour DateField locale child failed:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            return;
        }

        assert_eq!(HourCycle::default(), HourCycle::H12);
        let state = cx.new(|cx| crate::input::InputState::new(cx));
        let field = DateField::new(state.clone()).granularity(Granularity::Minute);
        assert_eq!(field.hour_cycle, HourCycle::H12);
        assert_eq!(
            TimeSegment::order(
                field.granularity.time().unwrap(),
                field.hour_cycle == HourCycle::H12,
            ),
            [
                TimeSegment::Hour,
                TimeSegment::Minute,
                TimeSegment::Meridiem,
            ]
        );

        let explicit = DateField::new(state)
            .granularity(Granularity::Minute)
            .hour_cycle(HourCycle::H24);
        assert_eq!(explicit.hour_cycle, HourCycle::H24);
        assert_eq!(
            TimeSegment::order(
                explicit.granularity.time().unwrap(),
                explicit.hour_cycle == HourCycle::H12,
            ),
            [TimeSegment::Hour, TimeSegment::Minute]
        );
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
        let us = RegionalDateFormat::for_locale("en-US").unwrap();
        let gb = RegionalDateFormat::for_locale("en-GB").unwrap();
        let german = RegionalDateFormat::for_locale("de-DE").unwrap();
        assert_eq!(format_hint(&us, None), "MM/DD/YYYY");
        assert_eq!(format_hint(&gb, None), "DD/MM/YYYY");
        assert_eq!(format_hint(&german, None), "DD.MM.YYYY");
        let minute = crate::time_field::RegionalTimePattern {
            order: vec![TimeSegment::Hour, TimeSegment::Minute],
            literals: vec![String::new(), ":".to_owned(), String::new()],
            hour_has_leading_zero: true,
            hour_zero_based: false,
            minute_has_leading_zero: true,
            second_has_leading_zero: false,
            am: "AM".to_owned(),
            pm: "PM".to_owned(),
        };
        assert_eq!(format_hint(&us, Some(&minute)), "MM/DD/YYYY, HH:MM");
        let second_twelve = crate::time_field::RegionalTimePattern {
            order: vec![
                TimeSegment::Hour,
                TimeSegment::Minute,
                TimeSegment::Second,
                TimeSegment::Meridiem,
            ],
            literals: vec![
                String::new(),
                ":".to_owned(),
                ":".to_owned(),
                " ".to_owned(),
                String::new(),
            ],
            hour_has_leading_zero: false,
            hour_zero_based: false,
            minute_has_leading_zero: true,
            second_has_leading_zero: true,
            am: "AM".to_owned(),
            pm: "PM".to_owned(),
        };
        assert_eq!(
            format_hint(&us, Some(&second_twelve)),
            "MM/DD/YYYY, HH:MM:SS AM"
        );
    }
}
