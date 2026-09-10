//! DateRangePicker.

use super::*;

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
    /// `value` — v3's controlled range, stored for the first render only.
    /// Each end is its own `Option` so a half-open controlled range keeps its
    /// null end, exactly as the builder spelled it.
    value: Option<(Option<Date>, Option<Date>)>,
    state: Entity<DateRangeState>,
    /// `isOpen` — `None` leaves the picker holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    should_close_on_select: bool,
    label: Option<SharedString>,
    trigger_indicator: Option<gpui::AnyElement>,
    range_separator: Option<gpui::AnyElement>,
    /// The fill the trigger takes on hover, in place of `--field-hover`.
    trigger_hover_bg: Option<gpui::Hsla>,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    validation_behavior: crate::form::ValidationBehavior,
    validate: Option<crate::validation::Validator<Option<(Date, Date)>>>,
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    auto_focus: bool,
    constraints: DateConstraints,
    on_open_change: Option<std::sync::Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    on_change: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    start_form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    end_form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    form_is_disabled: Rc<Cell<bool>>,
    form_default: Rc<RefCell<Option<(Date, Date)>>>,
    form_on_change: Rc<RefCell<Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>>>,
    start_field_state: Rc<RefCell<Option<Entity<crate::input::InputState>>>>,
    end_field_state: Rc<RefCell<Option<Entity<crate::input::InputState>>>>,
    form_restore: std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
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
            value: None,
            state,
            is_open: None,
            default_open: false,
            should_close_on_select: true,
            label: None,
            trigger_indicator: None,
            range_separator: None,
            trigger_hover_bg: None,
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
            sx: None,
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

    /// The fill the trigger takes on hover, in place of `--field-hover`.
    pub fn trigger_hover_bg(mut self, color: impl Into<gpui::Hsla>) -> Self {
        self.trigger_hover_bg = Some(color.into());
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

    /// `value` — v3's controlled-range spelling, as a pure builder.
    ///
    /// The bound [`DateRangeState`] owns the range once the picker renders,
    /// so this seeds the state on the first render only, winning over
    /// [`DateRangePicker::default_value`] the way v3's controlled prop
    /// outranks the uncontrolled seed; calling `.value(..)` twice keeps the
    /// last call, like every other builder here. A later range is an
    /// imperative update rather than a builder: write the caller-owned state
    /// entity.
    pub fn value(mut self, start: Option<Date>, end: Option<Date>) -> Self {
        self.value = Some((start, end));
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
    pub fn on_open_change(mut self, f: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
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

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the picker's root element after every value the component
    /// and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
        self
    }
}

impl RenderOnce for DateRangePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.form_is_disabled.set(self.is_disabled);
        // Every keyed slot and focus handle below hangs off this one base, so
        // no two of them can flatten into a shared key.
        let base_id = gpui::ElementId::named_usize("drp", self.state.entity_id().as_u64() as usize);
        // `value` / `defaultValue` seed the state once, before anything reads
        // it. `value` is v3's controlled spelling, so it outranks the
        // uncontrolled seed; the state owns the range afterwards, and a write
        // to the caller-owned entity is the imperative update.
        if let Some((start, end)) = self.value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::named_usize(
                    "daterangepicker-default",
                    self.state.entity_id().as_u64() as usize,
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.start = start;
                        s.end = end;
                        if let Some(start) = start {
                            s.view_year = start.year;
                            s.view_month = start.month;
                            s.view_day = start.day;
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
                    "daterangepicker-default",
                    self.state.entity_id().as_u64() as usize,
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
            element_id::scoped(&base_id, "open"),
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
                element_id::scoped(&base_id, "start-field"),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, start_initial)),
            )
            .read(cx)
            .clone();
        let end_initial = end_text.clone();
        let end_field_state = window
            .use_keyed_state(
                element_id::scoped(&base_id, "end-field"),
                cx,
                move |_, cx| cx.new(|cx| crate::input::InputState::with_value(cx, end_initial)),
            )
            .read(cx)
            .clone();
        *self.start_field_state.borrow_mut() = Some(start_field_state.clone());
        *self.end_field_state.borrow_mut() = Some(end_field_state.clone());
        let start_sync = window.use_keyed_state(element_id::scoped(&base_id, "start-sync"), cx, {
            let start_text = start_text.clone();
            move |_, _| start_text
        });
        let end_sync = window.use_keyed_state(element_id::scoped(&base_id, "end-sync"), cx, {
            let end_text = end_text.clone();
            move |_, _| end_text
        });
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
                .child(render(DateRangePickerRenderState {
                    is_disabled: self.is_disabled,
                    is_invalid: start_invalid || end_invalid,
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
        let initiator =
            window.use_keyed_state(element_id::scoped(&base_id, "initiator"), cx, |_, _| 0usize);
        let trigger_focus =
            crate::util::tab_stop_handle(element_id::scoped(&base_id, "trigger-focus"), window, cx);
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
                    cb(&true, window, cx);
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
                    cb(&true, window, cx);
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
            .when_some(self.locale.clone(), |field, tag| field.locale(tag))
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
            .when_some(self.locale.clone(), |field, tag| field.locale(tag))
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
            .id(base_id.clone())
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
            let hover_bg = self.trigger_hover_bg.unwrap_or(colors.field.hover());
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
            .id(element_id::scoped(&base_id, "trigger"))
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
                .cursor(crate::util::interactive_cursor(cx))
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
                        cb(&!was_open, window, cx);
                    }
                });
        }
        trigger = trigger
            .a11y_named(a11y::Role::Button, &a11y::Name::labelled("Calendar"))
            .a11y_expanded(is_open);

        field = field
            .a11y_named(a11y::Role::Group, &a11y::Name::maybe(self.label.clone()))
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
                    cb(&false, window, cx);
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

        crate::util::apply_sx(root.track_focus(&blur_scope), &self.sx).into_any_element()
    }
}

// ---------------------------------------------------------------------------
