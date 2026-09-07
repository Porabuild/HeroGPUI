//! DatePicker.

use super::*;

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
    /// `value` — v3's controlled date, stored for the first render only. The
    /// outer `Option` distinguishes an unset builder from `value(null)`, the
    /// explicitly controlled empty date.
    value: Option<Option<Date>>,
    constraints: DateConstraints,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    validation_behavior: crate::form::ValidationBehavior,
    validate: Option<crate::validation::Validator<Option<Date>>>,
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    auto_focus: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
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
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
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

    /// `value` — v3's controlled-date spelling, as a pure builder.
    ///
    /// The bound [`CalendarState`] owns the selection once the picker
    /// renders, so this seeds the state on the first render only, winning
    /// over [`DatePicker::default_value`] the way v3's controlled prop
    /// outranks the uncontrolled seed; calling `.value(..)` twice keeps the
    /// last call, like every other builder here. `None` is v3's `null` — an
    /// explicitly controlled empty date, which still outranks
    /// `default_value`. A later date is an imperative update rather than a
    /// builder: write the caller-owned state entity.
    pub fn value(mut self, date: Option<Date>) -> Self {
        self.value = Some(date);
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
    pub fn on_open_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
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
                    callback(&date, window, cx);
                }
            });
        form_state.borrow_mut().restore = Some(restore);
        Self {
            locale: None,
            content: None,
            name: None,
            default_value: None,
            value: None,
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
            sx: None,
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

    pub fn on_change(mut self, f: impl Fn(&Option<Date>, &mut Window, &mut App) + 'static) -> Self {
        let callback = std::sync::Arc::new(f);
        *self.form_on_change.borrow_mut() = Some(callback.clone());
        self.on_change = Some(callback);
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the picker's root element after every value the component
    /// and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.form_is_disabled.set(self.is_disabled);
        // Every keyed slot and focus handle below hangs off this one base, so
        // no two of them can flatten into a shared key.
        let base_id = gpui::ElementId::named_usize("dp", self.state.entity_id().as_u64() as usize);
        // `value` / `defaultValue` seed the state once, before anything reads
        // it. `value` is v3's controlled spelling, so it outranks the
        // uncontrolled seed; the state owns the selection afterwards, and a
        // write to the caller-owned entity is the imperative update.
        if let Some(date) = self.value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::named_usize(
                    "datepicker-default",
                    self.state.entity_id().as_u64() as usize,
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.selected = date;
                        s.selected_dates = date.into_iter().collect();
                        if let Some(date) = date {
                            s.view_year = date.year;
                            s.view_month = date.month;
                            s.view_day = date.day;
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
                gpui::ElementId::named_usize(
                    "datepicker-default",
                    self.state.entity_id().as_u64() as usize,
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
            element_id::scoped(&base_id, "open"),
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
                element_id::scoped(&base_id, "field-state"),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, initial_text)),
            )
            .read(cx)
            .clone();
        *self.form_field_state.borrow_mut() = Some(field_state.clone());
        let field_sync = window.use_keyed_state(element_id::scoped(&base_id, "field-sync"), cx, {
            let selected_text = selected_text.clone();
            move |_, _| selected_text
        });
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
                    element_id::scoped(&base_id, "content-focus"),
                    cx,
                    |_, cx| cx.focus_handle().tab_stop(false),
                )
                .read(cx)
                .clone();
            let is_focus_within = content_scope.contains_focused(window, cx);
            let content_root = gpui::div()
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
                }));
            return crate::util::apply_sx(content_root, &self.sx).into_any_element();
        }
        let (overlay_phase, dismissal_token) = crate::util::overlay_scope(
            window,
            cx,
            element_id::scoped(&base_id, "overlay"),
            open,
            // Pickers have no exit animation; remove the calendar immediately
            // so a chosen cell cannot receive the same press again.
            false,
        );
        let panel_visible = overlay_phase != crate::util::OverlayPhase::Closed;
        let panel_open = overlay_phase == crate::util::OverlayPhase::Open;
        let blur_open_own = open_own.clone();
        let blur_open_change = self.on_open_change.clone();
        let blur_scope =
            crate::util::close_on_blur(window, cx, &base_id, open, move |window, cx| {
                if let Some(held) = &blur_open_own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(callback) = &blur_open_change {
                    callback(&false, window, cx);
                }
            });
        let trigger_focus =
            crate::util::tab_stop_handle(element_id::scoped(&base_id, "trigger-focus"), window, cx);
        let initiator =
            window.use_keyed_state(element_id::scoped(&base_id, "initiator"), cx, |_, _| 0usize);
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
                cb(&true, window, cx);
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
            .id(element_id::scoped(&base_id, "trigger"))
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
        // `useDatePicker` derives the trigger from `useOverlayTrigger`, so
        // it is a button with `aria-expanded`. `aria-haspopup` has no gpui
        // builder (see `crate::a11y`).
        trigger = trigger
            .a11y_named(a11y::Role::Button, &a11y::Name::labelled("Calendar"))
            .a11y_expanded(is_open);
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
                        cb(&!was_open, window, cx);
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
            .when_some(self.locale.clone(), |field, tag| field.locale(tag))
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
                        cb(&valid_date, window, cx);
                    }
                }
            });

        // `useDatePicker` is `role: 'group'` on the field box.
        let field = gpui::div()
            .id(base_id)
            .a11y_named(a11y::Role::Group, &a11y::Name::maybe(self.label.clone()))
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
                    cb(&false, window, cx);
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
                    calendar_validate.as_ref().and_then(|validate| validate(d)),
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

        crate::util::apply_sx(root.track_focus(&blur_scope), &self.sx).into_any_element()
    }
}

/// The popover chrome every picker shares — `.date-picker__popover` is
/// `bg-overlay p-3` at `min(32px, calc(--radius * 2.5))` with `--shadow-overlay`.
///
/// The calendars used to paint this themselves, which put a second panel inside
/// the first one and left a standalone `Calendar` looking like a floating card.
pub(super) fn picker_panel(cx: &App) -> gpui::Div {
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
