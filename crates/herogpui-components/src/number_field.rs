//! NumberField — port of `@heroui/number-field` (v3).

use std::{sync::Arc, time::Duration};

use gpui::{
    prelude::*, px, App, Entity, IntoElement, MouseDownEvent, MouseUpEvent, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::{FieldVariant, NumberFormat};
use herogpui_theme::ActiveTheme;

use crate::{icons, input::InputState};

/// State for a numeric input: text + parsed value.
pub struct NumberState {
    pub input: Entity<InputState>,
    value: f64,
    min: f64,
    max: f64,
    has_min: bool,
    has_max: bool,
    step: f64,
    /// `formatOptions`, written in by [`NumberField::format_options`]. The
    /// state owns it because the state is what turns a value into text.
    format: Option<NumberFormat>,
}

impl NumberState {
    pub fn new(cx: &mut App, initial: f64) -> Self {
        let input = cx.new(|cx| {
            let mut s = InputState::new(cx);
            s.set_value(format_number(initial));
            s
        });
        Self {
            input,
            value: initial,
            min: f64::MIN,
            max: f64::MAX,
            has_min: false,
            has_max: false,
            step: 1.0,
            format: None,
        }
    }

    /// `defaultValue` — a state seeded with an initial number.
    ///
    /// The uncontrolled entry point; `new` already takes the value, so this is
    /// its documented alias.
    pub fn with_value(cx: &mut App, value: f64) -> Self {
        Self::new(cx, value)
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    /// Writes a clamped value and syncs the text field.
    pub fn set_value(&mut self, v: f64, cx: &mut App) {
        self.value = v.clamp(self.min, self.max);
        let text = self.display_text();
        self.input.update(cx, |i, _| i.set_value(text));
    }

    /// The value as the field shows it: through `formatOptions` when one is
    /// set, otherwise the plain number.
    pub fn display_text(&self) -> String {
        match &self.format {
            Some(f) => f.format(self.value),
            None => format_number(self.value),
        }
    }

    /// Installs `formatOptions` and reformats the text to match.
    ///
    /// Typing is untouched — the raw text stays exactly as entered until
    /// something writes a value back, which is when v3 reformats too.
    pub fn set_format(&mut self, format: Option<NumberFormat>, cx: &mut App) {
        if self.format == format {
            return;
        }
        self.format = format;
        let text = self.display_text();
        self.input.update(cx, |i, _| i.set_value(text));
    }

    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min = min;
        self.max = max;
        self.has_min = true;
        self.has_max = true;
    }

    pub fn range(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    fn set_component_range(&mut self, min: Option<f64>, max: Option<f64>) {
        if let Some(min) = min {
            self.min = min;
            self.has_min = true;
        }
        if let Some(max) = max {
            self.max = max;
            self.has_max = true;
        }
    }

    fn bounds(&self) -> (Option<f64>, Option<f64>) {
        (
            self.has_min.then_some(self.min),
            self.has_max.then_some(self.max),
        )
    }

    pub fn step_size(&self) -> f64 {
        self.step
    }

    pub fn set_step(&mut self, step: f64) {
        self.step = if step.is_finite() && step > 0.0 {
            step
        } else {
            1.0
        };
    }

    /// Re-reads the text field and updates the numeric value (or restores the
    /// previous formatted value when unparsable).
    ///
    /// The text may be what `formatOptions` produced (`$1,200.00`), so the
    /// group separators and affixes come off before parsing — the field would
    /// otherwise reject its own output.
    pub fn sync_from_input(&mut self, cx: &mut App) {
        if let Some(v) = parse_number(self.input.read(cx).value()) {
            self.value = v.clamp(self.min, self.max);
        } else {
            let text = self.display_text();
            self.input.update(cx, |i, _| i.set_value(text));
        }
    }

    fn bump(&mut self, dir: f64, cx: &mut App) -> Option<f64> {
        let min = self.has_min.then_some(self.min);
        let max = self.has_max.then_some(self.max);
        let snapped = snap_to_step(self.value, min, max, self.step);
        let next = if (dir > 0.0 && snapped > self.value) || (dir < 0.0 && snapped < self.value) {
            snapped
        } else {
            snap_to_step(self.value + dir * self.step, min, max, self.step)
        };
        if next.to_bits() == self.value.to_bits() {
            return None;
        }
        self.set_value(next, cx);
        Some(next)
    }

    fn snap(&self, value: f64) -> f64 {
        let snapped = snap_to_step(
            value,
            self.has_min.then_some(self.min),
            self.has_max.then_some(self.max),
            self.step,
        );
        if snapped.is_finite() {
            snapped
        } else {
            value.clamp(self.min, self.max)
        }
    }
}

fn snap_to_step(value: f64, min: Option<f64>, max: Option<f64>, step: f64) -> f64 {
    let anchor = min.unwrap_or(0.0);
    let mut snapped = ((value - anchor) / step).round() * step + anchor;
    if let Some(min) = min {
        snapped = snapped.max(min);
    }
    if let Some(max) = max {
        if snapped > max {
            snapped = anchor + ((max - anchor) / step).floor() * step;
        }
    }
    round_to_step_precision(snapped, step)
}

fn round_to_step_precision(value: f64, step: f64) -> f64 {
    let step_text = step.to_string().to_ascii_lowercase();
    let precision = if let Some((mantissa, exponent)) = step_text.split_once('e') {
        let exponent = exponent.parse::<i32>().unwrap_or(0);
        let fraction = mantissa
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len() as i32);
        (fraction - exponent).max(0)
    } else {
        step_text
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len() as i32)
    };
    if precision == 0 {
        return value;
    }
    let scale = 10_f64.powi(precision);
    if !scale.is_finite() {
        return value;
    }
    (value * scale).round() / scale
}

/// Reads a number back out of formatted text: `$1,200.00` -> `1200.0`,
/// `(€12.00)` -> `-12.0`, `42%` -> `42.0`.
///
/// Only the digits, sign and decimal point survive; a formatted value has to
/// round-trip or the field rejects what it just rendered.
pub fn parse_number(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let accounting = text.starts_with('(') && text.ends_with(')');
    let mut digits = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '0'..='9' | '.' => digits.push(c),
            '-' if digits.is_empty() => digits.push('-'),
            _ => {}
        }
    }
    let v: f64 = digits.parse().ok()?;
    Some(if accounting { -v } else { v })
}

fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 && v.abs() < 1e15 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v}")
    }
}

type OnChange = Arc<dyn Fn(f64, &mut Window, &mut App) + 'static>;

/// HeroUI NumberField.
#[derive(IntoElement)]
pub struct NumberField {
    state: Entity<NumberState>,
    /// See [`NumberField::content`].
    content: Option<Arc<dyn Fn(crate::util::FieldFocus) -> gpui::AnyElement + 'static>>,
    /// `Description` — v3 composes it as a sibling of `NumberField.Group`.
    description: Option<SharedString>,
    label: Option<SharedString>,
    hide_steppers: bool,
    is_disabled: bool,
    variant: FieldVariant,
    full_width: bool,
    min_value: Option<f64>,
    max_value: Option<f64>,
    step: Option<f64>,
    /// `formatOptions` — installed into the state, which owns the text.
    format: Option<NumberFormat>,
    /// `name` — forwarded to the inner field's state.
    name: Option<SharedString>,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<f64>,
    /// `validationBehavior` — written into the inner field's state on render.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<f64>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    /// `isWheelDisabled` — suppress focused wheel stepping.
    is_wheel_disabled: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    on_change: Option<OnChange>,
}

impl NumberField {
    /// `minValue` — also settable on [`NumberState::set_range`].
    pub fn min_value(mut self, v: f64) -> Self {
        self.min_value = Some(v);
        self
    }

    /// `maxValue`
    pub fn max_value(mut self, v: f64) -> Self {
        self.max_value = Some(v);
        self
    }

    /// `step`
    pub fn step(mut self, v: f64) -> Self {
        self.step = Some(v);
        self
    }

    /// `name` — the name this field submits under.
    ///
    /// Forwarded to the inner text field's state, which is where
    /// `FormField::number` looks for it.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `defaultValue` — the uncontrolled initial number.
    ///
    /// Written into the state on the first render only. `NumberState::with_value`
    /// does the same at construction; this is the prop spelling.
    pub fn default_value(mut self, value: f64) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `validationBehavior` — see [`crate::input::Input::validation_behavior`].
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// `formatOptions` — how the value is written out.
    ///
    /// ```ignore
    /// NumberField::new(state).format_options(NumberFormat::currency("USD"))
    /// ```
    pub fn format_options(mut self, format: NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    /// `validate` — returns the message to show, or `None` when the number is fine.
    ///
    /// The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&f64) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(Arc::new(f));
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

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `isWheelDisabled` — whether focused wheel input may step the value.
    pub fn is_wheel_disabled(mut self, v: bool) -> Self {
        self.is_wheel_disabled = v;
        self
    }

    /// v3's field `children`-as-a-function, handed `{isFocused, isFocusWithin,
    /// isFocusVisible}`; see [`crate::input::Input::content`].
    pub fn content(
        mut self,
        render: impl Fn(crate::util::FieldFocus) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(Arc::new(render));
        self
    }

    pub fn new(state: Entity<NumberState>) -> Self {
        Self {
            content: None,
            state,
            description: None,
            label: None,
            hide_steppers: false,
            is_disabled: false,
            variant: FieldVariant::Primary,
            full_width: false,
            min_value: None,
            max_value: None,
            step: None,
            format: None,
            name: None,
            default_value: None,
            validation_behavior: None,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            is_wheel_disabled: false,
            auto_focus: false,
            on_change: None,
        }
    }

    /// `Description` — help text under the field.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// Hides the -/+ steppers (`hideStepper`).
    pub fn hide_steppers(mut self, v: bool) -> Self {
        self.hide_steppers = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(mut self, f: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }
}

impl RenderOnce for NumberField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // v3's field children-as-a-function: the caller builds the parts from the
        // focus state, so the field's own stack is skipped entirely.
        if let Some(render) = self.content.clone() {
            let handle = self.state.read(cx).input.read(cx).focus_handle.clone();
            let focused = handle.is_focused(window);
            return render(crate::util::FieldFocus {
                is_focused: focused,
                is_focus_within: handle.contains_focused(window, cx),
                is_focus_visible: focused && crate::util::focus_visible(cx),
            });
        }
        let colors = cx.colors().clone();
        let layout = cx.layout().clone();

        // `.number-field__group` is `h-9`, the one height every v3 field has.
        let h = crate::util::FIELD_HEIGHT;
        // v3 order: the controlled flag, then server errors, then `validate`.
        let value_now = self.state.read(cx).value();
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&value_now)),
            None,
        );

        // `.number-field__increment-button` is `h-full w-10`: a 40px square-ish
        // slot at the end of the group, not the 26px one this used to draw.
        let btn_px = px(40.);

        // Component-level `minValue`/`maxValue`/`step` win over whatever the
        // state was seeded with. Bound presence is stored separately from its
        // number, because f64::MIN/MAX are also legitimate explicit bounds.
        if self.min_value.is_some() || self.max_value.is_some() {
            self.state.update(cx, |s, _| {
                s.set_component_range(self.min_value, self.max_value);
            });
        }
        if let Some(step) = self.step {
            if step.to_bits() != self.state.read(cx).step_size().to_bits() {
                self.state.update(cx, |s, _| s.set_step(step));
            }
        }

        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(value) = self.default_value {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("number-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(value, cx);
                        cx.notify();
                    });
                },
            );
        }

        // `formatOptions` lives in the state, which owns the text. `set_format`
        // is a no-op when it already matches, so this does not loop.
        if let Some(format) = self.format.clone() {
            self.state
                .update(cx, |s, cx| s.set_format(Some(format), cx));
        }

        // text field bound to the inner InputState
        let text_state = self.state.clone();
        let on_text_change = self.on_change.clone();
        let mut field = crate::input::Input::new(self.state.read(cx).input.clone())
            .when_some(self.name.clone(), |f, n| f.name(n))
            .when_some(self.validation_behavior, |f, b| f.validation_behavior(b))
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_read_only(self.is_read_only)
            .is_required(self.is_required)
            .auto_focus(self.auto_focus)
            .is_invalid(validity.is_invalid)
            .on_change(move |_text: &str, w, cx| {
                // The Input already wrote its own text; re-parse here.
                let before = text_state.read(cx).value();
                let after = text_state.update(cx, |s, sc| {
                    s.sync_from_input(sc);
                    sc.notify();
                    s.value()
                });
                if before.to_bits() != after.to_bits() {
                    if let Some(cb) = &on_text_change {
                        cb(after, w, cx);
                    }
                }
            });

        // `.number-field__group` is one box -- `grid h-9 rounded-field bg-field
        // shadow-field overflow-hidden` -- with a 40px decrement button, the
        // input, and a 40px increment button inside it, each separated by a
        // hairline. The steppers used to sit *outside* the field as two loose
        // buttons, which is not a shape v3 has.
        let steppers = !self.hide_steppers;
        field = field.in_group(steppers, steppers);

        let mut group = gpui::div()
            .flex()
            .items_center()
            .h(h)
            .overflow_hidden()
            .text_size(crate::util::FIELD_TEXT);
        group = crate::util::apply_field_chrome(
            group,
            self.variant,
            validity.is_invalid,
            self.state
                .read(cx)
                .input
                .read(cx)
                .focus_handle
                .is_focused(window),
            cx,
        );
        if self.full_width {
            group = group.w_full();
        } else {
            group = group.w(px(220.));
        }

        // `border-field-placeholder/15` is the seam between a stepper and the
        // input; the buttons themselves are transparent.
        let seam = colors.field.placeholder.alpha(0.15);
        if steppers {
            group = group.child(
                stepper_btn(
                    &self.state,
                    &self.on_change,
                    &colors,
                    h,
                    btn_px,
                    icons::MINUS,
                    -1.0,
                    self.is_disabled || self.is_read_only,
                    window,
                    cx,
                )
                .border_r_1()
                .border_color(seam),
            );
        }
        group = group.child(gpui::div().flex_1().min_w_0().child(field));
        if steppers {
            group = group.child(
                stepper_btn(
                    &self.state,
                    &self.on_change,
                    &colors,
                    h,
                    btn_px,
                    icons::PLUS,
                    1.0,
                    self.is_disabled || self.is_read_only,
                    window,
                    cx,
                )
                .border_l_1()
                .border_color(seam),
            );
        }
        // React Aria drives a number field from `useSpinButton`: the arrows
        // step by `step`, Home and End run to the bounds, and Page Up/Down fall
        // back to a plain step because `NumberField` passes no page handlers.
        // The keys arrive at the focused input and bubble to here.
        if !self.is_disabled && !self.is_read_only {
            let key_state = self.state.clone();
            let key_change = self.on_change.clone();
            let (min_value, max_value) = self.state.read(cx).bounds();
            group = group.on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                let dir = match ev.keystroke.key.as_str() {
                    "up" | "pageup" => 1.0,
                    "down" | "pagedown" => -1.0,
                    "home" => {
                        let Some(min) = min_value else {
                            return;
                        };
                        if set_number_value(&key_state, min, cx) {
                            if let Some(cb) = &key_change {
                                cb(key_state.read(cx).value(), window, cx);
                            }
                        }
                        return;
                    }
                    "end" => {
                        let Some(max) = max_value else {
                            return;
                        };
                        if set_number_value(&key_state, max, cx) {
                            if let Some(cb) = &key_change {
                                cb(key_state.read(cx).value(), window, cx);
                            }
                        }
                        return;
                    }
                    _ => return,
                };
                report_bump(&key_state, dir, &key_change, window, cx);
            });
        }
        if !self.is_disabled && !self.is_read_only && !self.is_wheel_disabled {
            let wheel_state = self.state.clone();
            let wheel_change = self.on_change.clone();
            let wheel_focus = self.state.read(cx).input.read(cx).focus_handle.clone();
            group = group.on_scroll_wheel(move |event, window, cx| {
                if !wheel_focus.contains_focused(window, cx) {
                    return;
                }
                let (dx, dy) = match event.delta {
                    gpui::ScrollDelta::Pixels(point) => (f32::from(point.x), f32::from(point.y)),
                    gpui::ScrollDelta::Lines(point) => (point.x, point.y),
                };
                if event.modifiers.control || event.modifiers.platform {
                    return;
                }
                let direction = if dy != 0.0 { dy.signum() } else { dx.signum() };
                if direction != 0.0 {
                    report_bump(
                        &wheel_state,
                        f64::from(direction),
                        &wheel_change,
                        window,
                        cx,
                    );
                }
                cx.stop_propagation();
            });
        }
        if self.is_disabled {
            group = group.opacity(layout.disabled_opacity);
        }

        // The label, description and error slot belong to the field, not to the
        // group: v3 composes them as siblings of `NumberField.Group`.
        let mut el = gpui::div().flex().flex_col().gap(px(4.));
        if self.full_width {
            el = el.w_full();
        }
        if let Some(label) = &self.label {
            el = el.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_invalid(validity.is_invalid)
                    .is_disabled(self.is_disabled),
            );
        }
        el = el.child(group);
        if let Some(message) = validity.first() {
            el = el.child(crate::field::ErrorMessage::new(message));
        } else if let Some(description) = self.description.clone() {
            el = el.child(crate::field::Description::new(description));
        }
        el.into_any_element()
    }
}

/// One stepper cell: `flex h-full w-10 items-center justify-center
/// rounded-none bg-transparent`, pressed at `bg-field-foreground/10`.
#[allow(clippy::too_many_arguments)]
fn stepper_btn(
    state: &Entity<NumberState>,
    on_change: &Option<OnChange>,
    colors: &herogpui_theme::ThemeColors,
    h: gpui::Pixels,
    btn_px: gpui::Pixels,
    icon: &'static str,
    dir: f64,
    is_disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let st = state.clone();
    let on_change = on_change.clone();
    let id = gpui::ElementId::Name(format!("num-{}-{dir}", state.entity_id().as_u64()).into());
    let press = window.use_keyed_state(
        gpui::ElementId::Name(format!("num-{}-{dir}-press", state.entity_id().as_u64()).into()),
        cx,
        |_, _| StepperPress::default(),
    );
    let focus_handle = state.read(cx).input.read(cx).focus_handle.clone();
    let mut b = gpui::div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .w(btn_px)
        .h(h);
    if !is_disabled {
        let release = press.clone();
        b = b.child(
            gpui::canvas(
                |bounds, _, _| bounds,
                move |_, _, window, _| {
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == gpui::DispatchPhase::Capture
                            && event.button == gpui::MouseButton::Left
                        {
                            release.update(cx, |press, _| press.active = false);
                        }
                    });
                },
            )
            .absolute()
            .inset_0(),
        );
        let pressed_bg = colors.field.foreground.alpha(0.1);
        b = b
            .cursor_pointer()
            .hover(move |s| s.bg(pressed_bg))
            .on_mouse_down(
                gpui::MouseButton::Left,
                move |_: &MouseDownEvent, window, cx| {
                    window.focus(&focus_handle);
                    let generation = press.update(cx, |press, _| {
                        press.active = true;
                        press.generation = press.generation.wrapping_add(1);
                        press.generation
                    });
                    report_bump(&st, dir, &on_change, window, cx);

                    let repeat_press = press.downgrade();
                    let repeat_state = st.clone();
                    let repeat_change = on_change.clone();
                    window
                        .spawn(cx, async move |cx| {
                            cx.background_executor()
                                .timer(Duration::from_millis(400))
                                .await;
                            loop {
                                let keep_repeating = cx
                                    .update(|window, cx| {
                                        let Some(press) = repeat_press.upgrade() else {
                                            return false;
                                        };
                                        let active = {
                                            let press = press.read(cx);
                                            press.active && press.generation == generation
                                        };
                                        if !active {
                                            return false;
                                        }
                                        let changed = report_bump(
                                            &repeat_state,
                                            dir,
                                            &repeat_change,
                                            window,
                                            cx,
                                        );
                                        if !changed {
                                            if let Some(press) = repeat_press.upgrade() {
                                                press.update(cx, |press, _| press.active = false);
                                            }
                                        }
                                        changed
                                    })
                                    .unwrap_or(false);
                                if !keep_repeating {
                                    break;
                                }
                                cx.background_executor()
                                    .timer(Duration::from_millis(60))
                                    .await;
                            }
                        })
                        .detach();
                },
            );
    }
    b.child(
        gpui::svg()
            .size(crate::util::FIELD_ICON)
            .path(icon)
            .text_color(colors.field.foreground),
    )
}

#[derive(Default)]
struct StepperPress {
    active: bool,
    generation: u64,
}

fn set_number_value(state: &Entity<NumberState>, value: f64, cx: &mut App) -> bool {
    state.update(cx, |state, cx| {
        let before = state.value();
        let value = state.snap(value);
        state.set_value(value, cx);
        let changed = before.to_bits() != state.value().to_bits();
        if changed {
            cx.notify();
        }
        changed
    })
}

fn report_bump(
    state: &Entity<NumberState>,
    dir: f64,
    on_change: &Option<OnChange>,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let next = state.update(cx, |state, cx| {
        let next = state.bump(dir, cx);
        if next.is_some() {
            cx.notify();
        }
        next
    });
    let Some(next) = next else { return false };
    if let Some(callback) = on_change {
        callback(next, window, cx);
    }
    true
}
