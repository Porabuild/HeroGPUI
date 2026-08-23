//! NumberField — port of `@heroui/number-input` (v1).

use std::sync::Arc;

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

use crate::{icons, input::InputState};

/// State for a numeric input: text + parsed value.
pub struct NumberState {
    pub input: Entity<InputState>,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
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
        }
    }

    /// `defaultValue` — a state seeded with an initial number.
    ///
    /// The uncontrolled entry point; `new` already takes the value, so this is
    /// its documented alias.
    pub fn with_value(cx: &mut gpui::App, value: f64) -> Self {
        Self::new(cx, value)
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    /// Writes a clamped value and syncs the text field.
    pub fn set_value(&mut self, v: f64, cx: &mut App) {
        self.value = v.clamp(self.min, self.max);
        let text = format_number(self.value);
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
    pub fn sync_from_input(&mut self, cx: &mut App) {
        if let Ok(v) = self.input.read(cx).value().trim().parse::<f64>() {
            self.value = v.clamp(self.min, self.max);
        } else {
            let text = format_number(self.value);
            self.input.update(cx, |i, _| i.set_value(text));
        }
    }

    fn bump(&mut self, dir: f64, cx: &mut App) {
        let next = (self.value + dir * self.step).clamp(self.min, self.max);
        self.set_value(next, cx);
    }
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
    label: Option<SharedString>,
    hide_steppers: bool,
    is_disabled: bool,
    variant: FieldVariant,
    full_width: bool,
    min_value: Option<f64>,
    max_value: Option<f64>,
    step: Option<f64>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<f64>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<gpui::SharedString>,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
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
    pub fn validate(
        mut self,
        f: impl Fn(&f64) -> Option<gpui::SharedString> + 'static,
    ) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
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

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn new(state: Entity<NumberState>) -> Self {
        Self {
            state,
            label: None,
            hide_steppers: false,
            is_disabled: false,
            variant: FieldVariant::Primary,
            full_width: false,
            min_value: None,
            max_value: None,
            step: None,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            on_change: None,
        }
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
        let colors = cx.colors().clone();
        let _ = window;
        let layout = cx.layout().clone();

        let h = px(40.);
        // v3 order: the controlled flag, then server errors, then `validate`.
        let value_now = self.state.read(cx).value();
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&value_now)),
            None,
        );

        let btn_px = px(26.);

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
            if min != cur_min || max != cur_max || step != cur_step {
                self.state.update(cx, |s, _| {
                    s.set_range(min, max);
                    s.set_step(step);
                });
            }
        }

        // text field bound to the inner InputState
        let text_state = self.state.clone();
        let on_text_change = self.on_change.clone();
        let mut field = crate::input::Input::new(self.state.read(cx).input.clone())
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_read_only(self.is_read_only)
            .is_required(self.is_required)
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

        // The resolved message has to reach the field's error slot, or
        // `validate` would mark the field invalid without saying why.
        if let Some(message) = validity.first() {
            field = field.error_message(message);
        }

        if let Some(label) = &self.label {
            field = field.label(label.clone());
        }

        if self.full_width {
            field = field.full_width();
        }

        let mut el = gpui::div().flex().items_center().gap(px(6.));
        if self.full_width {
            el = el.w_full();
        }
        el = el.child(field);

        if !self.hide_steppers && !self.is_disabled {
            el = el
                .child(stepper_btn(
                    &self.state,
                    &self.on_change,
                    &colors,
                    h,
                    btn_px,
                    icons::MINUS,
                    -1.0,
                ))
                .child(stepper_btn(
                    &self.state,
                    &self.on_change,
                    &colors,
                    h,
                    btn_px,
                    icons::PLUS,
                    1.0,
                ));
        } else if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        }

        el
    }
}

#[allow(clippy::too_many_arguments)]
fn stepper_btn(
    state: &Entity<NumberState>,
    on_change: &Option<OnChange>,
    colors: &herogpui_theme::ThemeColors,
    h: gpui::Pixels,
    btn_px: gpui::Pixels,
    icon: &'static str,
    dir: f64,
) -> gpui::AnyElement {
    let st = state.clone();
    let on_change = on_change.clone();
    let id =
        gpui::ElementId::Name(format!("num-{}-{dir}", state.entity_id().as_u64()).into());
    let mut b = gpui::div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .w(btn_px)
        .h(h - px(8.))
        .rounded(px(6.))
        .cursor_pointer();
    let hover_bg = colors.default.soft_hover();
    b = b.hover(move |s| s.bg(hover_bg));
    b = b.on_click(move |_, window, cx| {
        st.update(cx, |s, sc| {
            s.bump(dir, sc);
            sc.notify();
        });
        if let Some(cb) = &on_change {
            cb(st.read(cx).value(), window, cx);
        }
    });
    b.child(
        gpui::svg()
            .size(px(11.))
            .path(icon)
            .text_color(colors.foreground),
    )
    .into_any_element()
}
