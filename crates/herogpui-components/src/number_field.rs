//! NumberField — port of `@heroui/number-field` (v3).

use std::sync::Arc;

use gpui::{prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::{FieldVariant, NumberFormat};
use herogpui_theme::ActiveTheme;

use crate::{icons, input::InputState};

/// State for a numeric input: text + parsed value.
pub struct NumberState {
    pub input: Entity<InputState>,
    value: f64,
    min: f64,
    max: f64,
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
    }

    pub fn range(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    pub fn step_size(&self) -> f64 {
        self.step
    }

    pub fn set_step(&mut self, step: f64) {
        self.step = step.max(0.0001);
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

    fn bump(&mut self, dir: f64, cx: &mut App) {
        let next = (self.value + dir * self.step).clamp(self.min, self.max);
        self.set_value(next, cx);
    }
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
        // state was seeded with. Only write when something actually differs,
        // so this does not loop through `notify`.
        if self.min_value.is_some() || self.max_value.is_some() || self.step.is_some() {
            let (cur_min, cur_max, cur_step) = {
                let st = self.state.read(cx);
                let (lo, hi) = st.range();
                (lo, hi, st.step_size())
            };
            let min = self.min_value.unwrap_or(cur_min);
            let max = self.max_value.unwrap_or(cur_max);
            let step = self.step.unwrap_or(cur_step);
            // Exact comparison on purpose: these are the same values round-
            // tripped through the state entity, so anything but bit equality
            // means the caller passed a new prop and the state must be written.
            #[allow(clippy::float_cmp)]
            let changed = min != cur_min || max != cur_max || step != cur_step;
            if changed {
                self.state.update(cx, |s, _| {
                    s.set_range(min, max);
                    s.set_step(step);
                });
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
                text_state.update(cx, |s, sc| {
                    s.sync_from_input(sc);
                    sc.notify();
                });
                if let Some(cb) = &on_text_change {
                    cb(text_state.read(cx).value(), w, cx);
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
                    self.is_disabled,
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
                    self.is_disabled,
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
            group = group.on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                let (min, max) = key_state.read(cx).range();
                let dir = match ev.keystroke.key.as_str() {
                    "up" | "pageup" => 1.0,
                    "down" | "pagedown" => -1.0,
                    "home" => {
                        key_state.update(cx, |s, cx| s.set_value(min, cx));
                        if let Some(cb) = &key_change {
                            cb(key_state.read(cx).value(), window, cx);
                        }
                        return;
                    }
                    "end" => {
                        key_state.update(cx, |s, cx| s.set_value(max, cx));
                        if let Some(cb) = &key_change {
                            cb(key_state.read(cx).value(), window, cx);
                        }
                        return;
                    }
                    _ => return,
                };
                key_state.update(cx, |s, cx| s.bump(dir, cx));
                if let Some(cb) = &key_change {
                    cb(key_state.read(cx).value(), window, cx);
                }
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
) -> gpui::Stateful<gpui::Div> {
    let st = state.clone();
    let on_change = on_change.clone();
    let id = gpui::ElementId::Name(format!("num-{}-{dir}", state.entity_id().as_u64()).into());
    let mut b = gpui::div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .flex_shrink_0()
        .w(btn_px)
        .h(h);
    if !is_disabled {
        let pressed_bg = colors.field.foreground.alpha(0.1);
        b = b
            .cursor_pointer()
            .hover(move |s| s.bg(pressed_bg))
            .on_click(move |_, window, cx| {
                st.update(cx, |s, sc| {
                    s.bump(dir, sc);
                    sc.notify();
                });
                if let Some(cb) = &on_change {
                    cb(st.read(cx).value(), window, cx);
                }
            });
    }
    b.child(
        gpui::svg()
            .size(crate::util::FIELD_ICON)
            .path(icon)
            .text_color(colors.field.foreground),
    )
}
