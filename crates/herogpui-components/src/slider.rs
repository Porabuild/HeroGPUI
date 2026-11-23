//! Slider — port of `@heroui/slider` (single or multi-thumb).

use std::{cell::RefCell, rc::Rc};

use gpui::{
    prelude::*, px, AnimationExt, App, Bounds, IntoElement, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, RenderOnce, Styled, Window,
};
use herogpui_core::{element_id, Color, Orientation};
use herogpui_theme::ActiveTheme;

use crate::a11y::A11y as _;

type DragCallback = std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>;

type Thumb = std::sync::Arc<dyn Fn(usize, f32) -> gpui::AnyElement + 'static>;
type Output = std::sync::Arc<dyn Fn(&[f32], &[String]) -> gpui::AnyElement + 'static>;
type OnChangeAll = std::sync::Arc<dyn Fn(&[f32], &mut Window, &mut App) + 'static>;

type OnChange = std::sync::Arc<dyn Fn(&f32, &mut Window, &mut App) + 'static>;

const SLIDER_THUMB_TRANSITION_MS: u64 = 250;

#[derive(Clone)]
struct SliderThumbMotion {
    target: f32,
    generation: usize,
    from: f32,
    scale: Rc<std::cell::Cell<f32>>,
}

struct SliderThumbMotionFrame {
    id: gpui::ElementId,
    generation: usize,
    from: f32,
    to: f32,
    scale: Rc<std::cell::Cell<f32>>,
    animate: bool,
}

impl SliderThumbMotionFrame {
    fn render(self, inner: gpui::Div, vertical: bool) -> gpui::AnyElement {
        let apply_scale = move |inner: gpui::Div, scale: f32| {
            if vertical {
                inner.w(px(16. * scale)).h(px(24. * scale))
            } else {
                inner.w(px(24. * scale)).h(px(16. * scale))
            }
        };
        if !self.animate {
            self.scale.set(self.to);
            return apply_scale(inner, self.to).into_any_element();
        }

        let scale = self.scale;
        let from = self.from;
        let to = self.to;
        inner
            .with_animation(
                element_id::indexed(&self.id, "scale", self.generation),
                gpui::Animation::new(std::time::Duration::from_millis(SLIDER_THUMB_TRANSITION_MS))
                    .with_easing(|t| crate::anim::Curve::Out.at(t)),
                move |inner, delta| {
                    let next = from + (to - from) * delta;
                    scale.set(next);
                    apply_scale(inner, next)
                },
            )
            .into_any_element()
    }
}

fn slider_thumb_motion(
    id: &gpui::ElementId,
    index: usize,
    target: f32,
    window: &mut Window,
    cx: &mut App,
) -> SliderThumbMotionFrame {
    let motion_id = element_id::indexed(id, "thumb-motion", index);
    let state = window.use_keyed_state(motion_id.clone(), cx, |_, _| SliderThumbMotion {
        target,
        generation: 0,
        from: target,
        scale: Rc::new(std::cell::Cell::new(target)),
    });
    let mut current = state.read(cx).clone();
    if (current.target - target).abs() > f32::EPSILON {
        current.from = current.scale.get();
        current.target = target;
        current.generation = current.generation.wrapping_add(1);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    SliderThumbMotionFrame {
        id: motion_id,
        generation: current.generation,
        from: current.from,
        to: target,
        scale: current.scale,
        animate: current.generation != 0
            && (current.from - target).abs() > f32::EPSILON
            && !ActiveTheme::reduce_motion(cx),
    }
}

fn format_value_labels(
    values: &[f32],
    format: Option<&herogpui_core::NumberFormat>,
) -> Vec<String> {
    let default_format = herogpui_core::NumberFormat::decimal();
    let format = format.unwrap_or(&default_format);
    values
        .iter()
        .map(|value| format.format(f64::from(*value)))
        .collect()
}

fn default_output(labels: &[String]) -> String {
    labels.join(" \u{2013} ")
}

/// Slider scale. HeroUI v3.2.4 draws exactly one slider size, which is
/// [`SliderSize::Md`] here and stays the default; `Sm` is HeroGPUI's additive
/// compact rail — an 8px track with a single-layer 14px round knob inside an
/// 18px interactive hit box, fully rounded on every part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderSize {
    /// The compact scale.
    Sm,
    #[default]
    /// The default scale.
    Md,
}

/// A label at a slider value. It does not affect snapping or pointer input.
pub struct SliderMark {
    value: f32,
    children: Vec<gpui::AnyElement>,
}

impl SliderMark {
    /// Position of this mark. Invalid and out-of-range values are omitted.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }
}

impl ParentElement for SliderMark {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

/// HeroUI Slider.
#[must_use = "a component does nothing until it is rendered: add it as a child or return it from `render`"]
#[derive(IntoElement)]
pub struct Slider {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<gpui::SharedString>,
    id: gpui::ElementId,
    value: f32,
    /// The rail/knob scale; see [`SliderSize`].
    size: SliderSize,
    min: f32,
    max: f32,
    step: f32,
    steps: Vec<f32>,
    show_steps: bool,
    marks: Vec<SliderMark>,
    on_drag_start: Option<DragCallback>,
    on_drag_end: Option<DragCallback>,
    is_disabled: bool,
    orientation: Orientation,
    label: Option<String>,
    /// `aria-label` — the explicit accessible name, which wins over the
    /// label-derived name exactly as the HTML name computation ranks it.
    aria_label: Option<gpui::SharedString>,
    show_value: bool,
    /// `formatOptions` — how the value read-out is written.
    format: Option<herogpui_core::NumberFormat>,
    /// `defaultValue` — set it to hand the slider its own state; the
    /// constructor's value then only seeds it.
    default_value: Option<f32>,
    /// `defaultValue`'s range form. This is the uncontrolled counterpart to
    /// [`Self::values`], which is the controlled multi-thumb value.
    default_values: Option<Vec<f32>>,
    /// `value: number[]` — a multi-thumb slider. `None` is the single-thumb
    /// form, which is `Slider::new`'s value.
    values: Option<Vec<f32>>,
    /// `children` on `Slider.Thumb` — v3's render prop, handed each thumb's
    /// `index` and value.
    thumb: Option<Thumb>,
    /// `Slider.Output`'s render props: every value and its formatted label.
    output: Option<Output>,
    on_change_all: Option<OnChangeAll>,
    on_change: Option<OnChange>,
    on_change_end: Option<OnChange>,
    on_change_end_all: Option<OnChangeAll>,
    /// `Slider.Thumb.isDisabled` — thumbs that cannot move, by index.
    disabled_keys: std::collections::HashSet<usize>,
    /// `startName` / `endName` on the range — read back by
    /// [`Self::form_fields`].
    start_name: Option<gpui::SharedString>,
    end_name: Option<gpui::SharedString>,
    /// Per-thumb `Slider.Thumb.name`, in thumb-index order.
    thumb_names: Vec<gpui::SharedString>,
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    form_thumb_states: Rc<RefCell<Vec<Rc<RefCell<crate::form::LiveFormFieldState>>>>>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
    recipes: Vec<gpui::SharedString>,
}

impl Slider {
    /// The rail and knob scale. `Md` is v3's geometry and the default.
    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
        self
    }

    /// `orientation` — a vertical slider runs bottom to top.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `value` — also accepted positionally by [`Slider::new`].
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Creates a slider with the given id and initial value.
    pub fn new(id: impl Into<gpui::ElementId>, value: f32) -> Self {
        Self {
            name: None,
            id: id.into(),
            value,
            size: SliderSize::default(),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            steps: Vec::new(),
            show_steps: false,
            marks: Vec::new(),
            on_drag_start: None,
            on_drag_end: None,
            is_disabled: false,
            orientation: Orientation::Horizontal,
            label: None,
            aria_label: None,
            show_value: false,
            format: None,
            default_value: None,
            default_values: None,
            values: None,
            thumb: None,
            output: None,
            on_change_all: None,
            on_change: None,
            on_change_end: None,
            on_change_end_all: None,
            disabled_keys: std::collections::HashSet::new(),
            start_name: None,
            end_name: None,
            thumb_names: Vec::new(),
            form_state: live_form_state(),
            form_thumb_states: Rc::new(RefCell::new(Vec::new())),
            sx: None,
            recipes: Vec::new(),
        }
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `Slider.Thumb.isDisabled` — thumbs that cannot be dragged, stepped by
    /// the keys, or reached by Tab, by thumb index — the same addressing
    /// [`Slider::values`] uses, and the same shape as a `Select`'s
    /// `disabledKeys` for its options. A disabled thumb stays put: the
    /// pointer's nearest-thumb choice can land on it but then does nothing; the
    /// own Tab cycle skip it, it leaves the tab order, and its field is not
    /// submitted. Disabling the whole *slider* is still
    /// [`Slider::is_disabled`].
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = usize>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `startName` on the range — the name the first thumb submits under.
    ///
    /// A two-thumb slider submits one pair per named end, the way
    /// `DateRangePicker`'s `startName`/`endName` do; [`Slider::form_fields`]
    /// reads the pair back. A single-thumb slider uses [`Slider::name`].
    pub fn start_name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.start_name = Some(name.into());
        self
    }

    /// `endName` on the range — the name the last thumb submits under.
    pub fn end_name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.end_name = Some(name.into());
        self
    }

    /// `Slider.Thumb.name` in thumb-index order.
    ///
    /// Entries override the compatibility [`Slider::start_name`] and
    /// [`Slider::end_name`] values at the same index. Names beyond the number
    /// of rendered thumbs are ignored; unnamed middle thumbs submit nothing.
    pub fn thumb_names(
        mut self,
        names: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.thumb_names = names.into_iter().map(Into::into).collect();
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// v3 discovers a field through the DOM; gpui gives a child no way to reach
    /// its ancestor, so the control hands the pair over instead. Borrows, so the
    /// control is still yours to place:
    ///
    /// ```
    /// # use gpui::{prelude::*, Window};
    /// # use herogpui_components::{Form, Slider};
    /// # struct Demo;
    /// # impl Render for Demo {
    /// #     fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    /// #         let form = Form::new();
    /// #         let control = Slider::new("volume", 0.5).name("volume");
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
    /// #     }
    /// # }
    /// # let mut tcx = gpui::TestAppContext::single();
    /// # tcx.update(herogpui_theme::ThemeProvider::init);
    /// # let _ = tcx.add_window_view(|_, _| Demo);
    /// ```
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        // A disabled thumb submits nothing, the way a disabled input is
        // skipped in HTML.
        if self.disabled_keys.contains(&0) {
            return None;
        }
        sync_form_value(
            &self.form_state,
            self.default_value.unwrap_or(self.value),
            !self.is_disabled,
        );
        Some(crate::form::FormField::live(name, self.form_state.clone()))
    }

    /// The `Form` fields this slider submits: one per named end of the range,
    /// read from the current thumb values. A disabled thumb's field is
    /// omitted, exactly as an HTML form skips a disabled `<input>`.
    pub fn form_fields(&self) -> Vec<crate::form::FormField> {
        if self.is_disabled {
            return Vec::new();
        }
        let thumbs: Vec<f32> = match &self.values {
            Some(v) if !v.is_empty() => v.clone(),
            _ => self
                .default_values
                .clone()
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| vec![self.default_value.unwrap_or(self.value)]),
        };
        let mut out = Vec::new();
        for (index, name) in self.names_for(thumbs.len()).into_iter().enumerate() {
            if let Some(name) = name {
                if !self.disabled_keys.contains(&index) {
                    let state = self.form_thumb_state(index);
                    if let Some(value) = thumbs.get(index).copied() {
                        sync_form_value(&state, value, true);
                    }
                    out.push(crate::form::FormField::live(name, state));
                }
            }
        }
        out
    }

    fn names_for(&self, count: usize) -> Vec<Option<gpui::SharedString>> {
        (0..count)
            .map(|index| {
                self.thumb_names.get(index).cloned().or_else(|| {
                    if count == 1 && index == 0 {
                        self.name.clone()
                    } else if index == 0 {
                        self.start_name.clone()
                    } else if index + 1 == count {
                        self.end_name.clone()
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    fn form_thumb_state(&self, index: usize) -> Rc<RefCell<crate::form::LiveFormFieldState>> {
        let mut states = self.form_thumb_states.borrow_mut();
        if states.len() <= index {
            states.resize_with(index + 1, live_form_state);
        }
        states[index].clone()
    }

    /// `minValue`
    pub fn min_value(mut self, v: f32) -> Self {
        self.min = v;
        self
    }

    /// `maxValue`
    pub fn max_value(mut self, v: f32) -> Self {
        self.max = v;
        self
    }

    /// Sets the step between values; a non-positive or non-finite step falls back to 1.
    pub fn step(mut self, v: f32) -> Self {
        self.steps.clear();
        self.step = if v.is_finite() && v > 0.0 { v } else { 1.0 };
        self
    }

    /// Disables pointer snapping. Arrow keys move by 1% of the range.
    /// This and `step`/`steps` replace the previous snapping mode.
    pub fn continuous(mut self, continuous: bool) -> Self {
        self.steps.clear();
        self.step = if continuous { 0.0 } else { 1.0 };
        self
    }

    /// Arbitrary snap points, sorted and deduplicated. Non-finite and out-of-range
    /// points are ignored; no usable points falls back to continuous movement.
    /// Arrow keys visit adjacent points; pointer ties choose the lower point.
    pub fn steps(mut self, steps: impl IntoIterator<Item = f32>) -> Self {
        self.steps = steps
            .into_iter()
            .filter(|value| value.is_finite())
            .collect();
        self.steps.sort_by(f32::total_cmp);
        self.steps.dedup();
        self.step = 0.0;
        self
    }

    /// `Slider.Steps`: decorative dots at the numeric step increments.
    /// Invalid ranges and more than 1,000 dots produce no indicators.
    /// This does not change the snapping configured by `step` or `steps`.
    pub fn show_steps(mut self, show: bool) -> Self {
        self.show_steps = show;
        self
    }

    /// `Slider.Marks`: labels at arbitrary values, independent of snapping.
    /// Empty labels use `format_options`, like the value output.
    pub fn marks(mut self, marks: impl IntoIterator<Item = SliderMark>) -> Self {
        self.marks = marks.into_iter().collect();
        self
    }

    /// Fires before the first pointer value write, including a track press.
    /// Keyboard changes do not start a drag.
    pub fn on_drag_start(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_drag_start = Some(std::sync::Arc::new(f));
        self
    }

    /// Fires once after the pointer release's `on_change_end`, even when the
    /// value did not change. Release outside the track also ends the drag.
    pub fn on_drag_end(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_drag_end = Some(std::sync::Arc::new(f));
        self
    }

    /// Sets whether the slider is disabled.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Sets the label text.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// `aria-label` — the explicit accessible name, which wins over the
    /// label-derived name exactly as the HTML name computation ranks it.
    pub fn aria_label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// `formatOptions` — how the value read-out is written.
    pub fn format_options(mut self, format: herogpui_core::NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets whether the current value is shown.
    pub fn show_value(mut self, v: bool) -> Self {
        self.show_value = v;
        self
    }

    /// `defaultValue` — the uncontrolled initial value.
    ///
    /// Supplying it makes the slider own its value: the constructor's `value`
    /// becomes the seed and dragging moves the slider's own copy, so a caller
    /// with nothing to store can leave `on_change` off entirely.
    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `defaultValue` for an uncontrolled multi-thumb slider.
    ///
    /// This is the range form of [`Slider::default_value`]. The values seed
    /// the slider's own keyed state; they do not make it controlled.
    pub fn default_values(mut self, values: impl IntoIterator<Item = f32>) -> Self {
        self.default_values = Some(values.into_iter().collect());
        self
    }

    /// `value` as an array — a multi-thumb slider.
    ///
    /// Dragging moves whichever thumb is nearest the pointer and reports the
    /// whole set through [`Slider::on_change_all`].
    pub fn values(mut self, values: impl IntoIterator<Item = f32>) -> Self {
        self.values = Some(values.into_iter().collect());
        self
    }

    /// `children` on `Slider.Thumb` — replaces a thumb.
    ///
    /// The closure receives the thumb's `index` and its value, the values v3
    /// passes into the same render prop. With a single-thumb slider the index
    /// is always 0.
    pub fn thumb(mut self, render: impl Fn(usize, f32) -> gpui::AnyElement + 'static) -> Self {
        self.thumb = Some(std::sync::Arc::new(render));
        self
    }

    /// `Slider.Output`'s render function — handed every current thumb value
    /// and the corresponding labels produced by `formatOptions`.
    pub fn output(
        mut self,
        render: impl Fn(&[f32], &[String]) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.output = Some(std::sync::Arc::new(render));
        self
    }

    /// `onChange` for the multi-thumb form — reports every value.
    pub fn on_change_all(mut self, f: impl Fn(&[f32], &mut Window, &mut App) + 'static) -> Self {
        self.on_change_all = Some(std::sync::Arc::new(f));
        self
    }

    /// Sets the handler run when the value changes.
    pub fn on_change(mut self, f: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes, for callers that only
    /// want to commit the final value.
    pub fn on_change_end(mut self, f: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change_end = Some(std::sync::Arc::new(f));
        self
    }

    /// `onChangeEnd` for the multi-thumb form — reports every final value.
    pub fn on_change_end_all(
        mut self,
        f: impl Fn(&[f32], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_end_all = Some(std::sync::Arc::new(f));
        self
    }

    /// Named theme overlay from [`herogpui_theme::ComponentThemes::slider`].
    /// Stackable; a missing name adds no override.
    pub fn recipe(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.recipes.push(name.into());
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the slider's root element after every value the orientation
    /// and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        crate::util::refine_sx(&mut self.sx, style);
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        // Supplying `default_value` is what opts into the uncontrolled mode.
        let steps: std::sync::Arc<[f32]> = self
            .steps
            .iter()
            .copied()
            .filter(|value| *value >= self.min && *value <= self.max)
            .collect::<Vec<_>>()
            .into();
        let range_mode = self
            .values
            .as_ref()
            .is_some_and(|values| !values.is_empty())
            || self
                .default_values
                .as_ref()
                .is_some_and(|values| !values.is_empty());
        let (value, own) = if range_mode {
            (self.value, None)
        } else {
            crate::util::controlled(
                window,
                cx,
                element_id::scoped(&self.id, "value"),
                match self.default_value {
                    Some(_) => None,
                    None => Some(self.value),
                },
                self.default_value.unwrap_or(self.value),
            )
        };

        let (range_values, range_own) = if range_mode {
            let range_default = self
                .default_values
                .clone()
                .filter(|values| !values.is_empty())
                .unwrap_or_else(|| vec![self.default_value.unwrap_or(self.value)]);
            let range_controlled = self
                .values
                .clone()
                .filter(|values| !values.is_empty())
                .map(|values| normalize_values(&values, self.min, self.max, self.step));
            let range_default = normalize_values(&range_default, self.min, self.max, self.step);
            crate::util::controlled(
                window,
                cx,
                element_id::scoped(&self.id, "values"),
                range_controlled,
                range_default,
            )
        } else {
            (Vec::new(), None)
        };

        // One thumb or many: the single-value form is a set of one, so the rest
        // of the render does not branch. Computed before the keyed states
        // because the roving stop's starting position has to skip a disabled
        // thumb.
        let thumbs: Vec<f32> = if range_mode {
            range_values
                .into_iter()
                .map(|value| snap_value(value, self.min, self.max, self.step, &steps))
                .collect()
        } else {
            vec![value]
        };
        let form_defaults =
            window.use_keyed_state(element_id::scoped(&self.id, "form-defaults"), cx, |_, _| {
                None::<Vec<f32>>
            });
        if form_defaults.read(cx).is_none() {
            let initial = thumbs.clone();
            form_defaults.update(cx, |slot, cx| {
                *slot = Some(initial);
                cx.notify();
            });
        }
        sync_form_values(
            &self.form_state,
            &self.form_thumb_states,
            &thumbs,
            self.is_disabled,
            &self.disabled_keys,
        );

        // Form reset is a live operation: the hidden range inputs in v3 read
        // the current thumb values, and reset restores the initial values. A
        // range registers one restore callback for the pair so its controlled
        // onChange callback fires exactly once per reset.
        let restore_defaults = form_defaults
            .read(cx)
            .clone()
            .unwrap_or_else(|| thumbs.clone());
        // Restore is stored on these fields. Weak references break that cycle,
        // including the range's first named thumb. Retain each thumb's identity
        // separately because registered FormFields can outlive the builder.
        let restore_form_state = Rc::downgrade(&self.form_state);
        let restore_thumb_states: Vec<_> = self
            .form_thumb_states
            .borrow()
            .iter()
            .map(Rc::downgrade)
            .collect();
        let restore_own = own.clone();
        let restore_range_own = range_own.clone();
        let restore_on_change = self.on_change.clone();
        let restore_on_change_all = self.on_change_all.clone();
        let restore_range_mode = range_mode;
        let restore_is_disabled = self.is_disabled;
        let restore_disabled_keys = self.disabled_keys.clone();
        let restore: std::sync::Arc<dyn Fn(&mut Window, &mut App)> =
            crate::util::shared(move |window, cx| {
                if restore_range_mode {
                    if let Some(own) = &restore_range_own {
                        own.update(cx, |current, cx| {
                            *current = restore_defaults.clone();
                            cx.notify();
                        });
                    }
                    if let Some(callback) = &restore_on_change_all {
                        callback(&restore_defaults, window, cx);
                    }
                } else if let Some(value) = restore_defaults.first().copied() {
                    if let Some(own) = &restore_own {
                        own.update(cx, |current, cx| {
                            *current = value;
                            cx.notify();
                        });
                    }
                    if let Some(callback) = &restore_on_change {
                        callback(&value, window, cx);
                    }
                }
                if let Some((state, value)) =
                    restore_form_state.upgrade().zip(restore_defaults.first())
                {
                    sync_form_value(
                        &state,
                        *value,
                        !restore_is_disabled && !restore_disabled_keys.contains(&0),
                    );
                }
                for (index, (state, value)) in restore_thumb_states
                    .iter()
                    .zip(&restore_defaults)
                    .enumerate()
                {
                    if let Some(state) = state.upgrade() {
                        sync_form_value(
                            &state,
                            *value,
                            !restore_is_disabled && !restore_disabled_keys.contains(&index),
                        );
                    }
                }
            });
        self.form_state.borrow_mut().restore = Some(restore.clone());
        let named_indices: Vec<usize> = self
            .names_for(thumbs.len())
            .into_iter()
            .enumerate()
            .filter_map(|(index, name)| {
                (name.is_some() && !self.disabled_keys.contains(&index)).then_some(index)
            })
            .collect();
        let restore = self.form_state.borrow().restore.clone();
        for (position, index) in named_indices.into_iter().enumerate() {
            let state = self.form_thumb_state(index);
            state.borrow_mut().restore = if position == 0 { restore.clone() } else { None };
        }
        // `Slider.Thumb.isDisabled` — which thumbs may move, by index. A
        // disabled thumb is never the pointer's target, is skipped by the
        // arrows and by the slider's own Tab cycle, and does not take the
        // focus ring.
        let thumb_enabled: Vec<bool> = (0..thumbs.len())
            .map(|i| !self.is_disabled && !self.disabled_keys.contains(&i))
            .collect();
        let any_enabled = thumb_enabled.iter().any(|&enabled| enabled);

        // The keyboard's own state: the handle that receives the keys and which
        // thumb they move. React Aria focuses one thumb at a time, so a range
        // slider needs to know which.
        let focus_handle =
            window.use_keyed_state(element_id::scoped(&self.id, "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            });
        let focus_handle = focus_handle.read(cx).clone();
        // The roving stop starts on the first *enabled* thumb, the radio
        // group's rule for its single stop: a stop resting on a disabled
        // thumb would have nothing to answer the keys.
        let active_init = thumb_enabled
            .iter()
            .position(|&enabled| enabled)
            .unwrap_or(0);
        let active_thumb =
            window.use_keyed_state(element_id::scoped(&self.id, "thumb"), cx, |_, _| {
                active_init
            });
        let active_at = *active_thumb.read(cx);

        // The drag's state hangs off the window's keyed store, keyed by this
        // slider's own id -- the same shape the Table's column resize uses
        // (`{id}-resizing`). The press repaints the window, and a per-render
        // `Rc<Cell>` would be rebuilt fresh afterwards, which is the defect
        // this replaces: every `on_mouse_move` after the press read a new
        // `false` and no thumb ever moved. Keyed by the id means two sliders
        // on one page can never share a drag.
        let bounds_slot =
            window.use_keyed_state(element_id::scoped(&self.id, "bounds"), cx, |_, _| Bounds::<
                f32,
            > {
                origin: gpui::point(0., 0.),
                size: gpui::size(0., 0.),
            });
        let dragging =
            window.use_keyed_state(element_id::scoped(&self.id, "dragging"), cx, |_, _| {
                DragState::default()
            });

        let sem = *cx.role(Color::Accent);
        let default = *cx.role(Color::Default);
        let colors = cx.colors().clone();
        let layout = cx.layout().clone();

        let to_fraction = |v: f32| {
            if self.max > self.min {
                ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let fractions: Vec<f32> = thumbs.iter().copied().map(to_fraction).collect();
        // A multi-thumb slider fills between its outermost thumbs; a single one
        // fills from the low end.
        let (fill_from, fill_to) = if thumbs.len() > 1 {
            (
                fractions.iter().copied().fold(1.0f32, f32::min),
                fractions.iter().copied().fold(0.0f32, f32::max),
            )
        } else {
            (0.0, fractions[0])
        };

        // v3's track is 20px on its cross axis. Its thumb is a two-layer pill:
        // 28x20 around a 24x16 centre horizontally, transposed vertically.
        let track_cross = px(20.);
        let thumb_along = px(28.);
        // `Sm` is the additive compact rail: an 8px track with a single-layer
        // 14x14 round knob centered inside an 18px interactive hit box so the
        // overhanging handle always receives pointer input.
        let small = self.size == SliderSize::Sm;
        let (track_cross, thumb_along) = if small {
            (px(8.), px(14.))
        } else {
            (track_cross, thumb_along)
        };
        // The knob overhangs the 8px rail, so it centers on the cross axis by
        // half the overhang; `Md`'s thumb matches the rail and needs none.
        let thumb_cross = if small { px(14.) } else { track_cross };
        let hit_cross = if small { px(18.) } else { track_cross };
        let cross_overhang = (thumb_cross - track_cross) / 2.;
        let range_span = self.max - self.min;

        // `useSlider` names the whole control `role="group"`; only the thumb
        // inputs are sliders. The group carries the label, the thumbs carry
        // the value.
        let group_name = crate::a11y::Name::maybe(
            self.aria_label
                .clone()
                .or_else(|| self.label.clone().map(gpui::SharedString::from)),
        );
        // A horizontal slider fills its container, as v3's does; without a
        // width the label and value read-out collapse together instead of
        // sitting at opposite ends.
        // The accessibility calls come after the layout chain deliberately:
        // `design_audit.py` reads this wrapper's gap through a pattern
        // anchored on `gpui::div()` followed by its layout calls.
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .when(self.orientation.is_horizontal(), |e| e.w_full())
            .id(element_id::scoped(&self.id, "group"))
            .a11y_named(crate::a11y::Role::Group, &group_name);

        // `.slider__output` is `text-sm font-medium tabular-nums` beside the
        // label; the two share the row above the track.
        let show_output = self.show_value || self.output.is_some();
        if self.label.is_some() || show_output {
            el = el.child(
                gpui::div()
                    .flex()
                    .justify_between()
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child(self.label.clone().unwrap_or_default())
                    .when(show_output, |l| {
                        let formatted_values = format_value_labels(&thumbs, self.format.as_ref());
                        let output = match &self.output {
                            Some(render) => render(&thumbs, &formatted_values),
                            None => gpui::div()
                                .child(default_output(&formatted_values))
                                .into_any_element(),
                        };
                        l.child(
                            gpui::div()
                                .font_features(crate::util::tabular_font_features())
                                .child(output),
                        )
                    }),
            );
        }

        // A vertical slider swaps the axis: the rail runs top to bottom and
        // the fill grows upward from the zero end.
        let vertical = !self.orientation.is_horizontal();
        let locale = crate::i18n::locale(cx);
        let rtl = !vertical && crate::i18n::is_rtl(&locale);
        let sx_radius = crate::util::sx_radius(&self.sx);
        let explicit_sx_corners = [
            sx_radius.top_left,
            sx_radius.top_right,
            sx_radius.bottom_left,
            sx_radius.bottom_right,
        ]
        .iter()
        .any(Option::is_some);
        let sx_corners = crate::util::fill_unspecified_corners(
            sx_radius,
            cx.theme()
                .components
                .slider
                .resolve_named("slider", &self.recipes)
                .radius,
        );
        let mut track = gpui::div()
            .relative()
            .flex()
            .items_center()
            .rounded(crate::util::small_radius(cx))
            .when(small, |t| t.rounded_full())
            .bg(default.color);
        track = if vertical {
            if small {
                track.w(track_cross).h_full()
            } else {
                track.w(track_cross).h(px(160.))
            }
        } else {
            track.w_full().h(track_cross)
        };
        track = crate::util::round_sx_corners(track, &sx_corners);

        // v3's track keeps transparent 12px borders along the slider axis
        // (`border-x-[0.75rem]` horizontal, `border-y-[0.75rem]` vertical —
        // half the 24px inner thumb), so the full border box is the pointer
        // range while the fill and the thumb percentages resolve against the
        // content box that border leaves. The end borders turn accent when
        // the fill reaches that end (`data-fill-start`/`data-fill-end`).
        let axis_inset = px(12.);
        let axis_inset = if small { px(8.) } else { axis_inset };
        let fill_start = fill_reaches_start(thumbs.len(), fill_from, fill_to);
        let fill_end = fill_reaches_end(fill_to);
        let track_radius = crate::util::small_radius(cx);
        // Half the 8px rail rounds `Sm`'s end caps into a pill.
        let track_radius = if small {
            track_cross / 2.
        } else {
            track_radius
        };
        for (visible, at_start) in [(fill_start, true), (fill_end, false)] {
            if visible {
                track = track.child(fill_cap(
                    vertical,
                    if rtl { !at_start } else { at_start },
                    axis_inset,
                    track_cross,
                    track_radius,
                    &sx_corners,
                    explicit_sx_corners,
                    sem.color,
                ));
            }
        }

        // The box the fill and thumbs position against: the track inset by
        // 12px per axis edge, which is where the transparent border leaves
        // its background — the same containing block the browser gives v3's
        // percentage-positioned fill and thumb.
        let mut content = gpui::div().absolute();
        content = if vertical {
            content
                .top(axis_inset)
                .bottom(axis_inset)
                .left(px(0.))
                .right(px(0.))
        } else {
            content
                .left(axis_inset)
                .right(axis_inset)
                .top(px(0.))
                .bottom(px(0.))
        };

        // `.slider__fill` is `pointer-events-none absolute bg-accent`.
        let fill_span = (fill_to - fill_from).max(0.0);
        let mut fill_corners = sx_corners;
        // A cap owns the perimeter rounding; its join with a single-thumb
        // fill is internal. Keep range and no-corner-override styling intact.
        if !range_mode
            && [
                sx_corners.top_left,
                sx_corners.top_right,
                sx_corners.bottom_left,
                sx_corners.bottom_right,
            ]
            .iter()
            .any(Option::is_some)
        {
            if vertical {
                if fill_start {
                    fill_corners.bottom_left = Some(px(0.));
                    fill_corners.bottom_right = Some(px(0.));
                }
                if fill_end {
                    fill_corners.top_left = Some(px(0.));
                    fill_corners.top_right = Some(px(0.));
                }
            } else {
                if (fill_start && !rtl) || (fill_end && rtl) {
                    fill_corners.top_left = Some(px(0.));
                    fill_corners.bottom_left = Some(px(0.));
                }
                if (fill_end && !rtl) || (fill_start && rtl) {
                    fill_corners.top_right = Some(px(0.));
                    fill_corners.bottom_right = Some(px(0.));
                }
            }
        }
        content = content.child(
            gpui::div()
                .absolute()
                .bg(sem.color)
                .when(small, |f| f.rounded_full())
                .when(vertical, |f| {
                    f.bottom(gpui::relative(fill_from))
                        .w_full()
                        .h(gpui::relative(fill_span))
                })
                .when(!vertical, |f| {
                    f.left(gpui::relative(if rtl { 1. - fill_to } else { fill_from }))
                        .h_full()
                        .w(gpui::relative(fill_span))
                })
                .map(|f| crate::util::round_sx_corners(f, &fill_corners)),
        );

        // `.slider__steps` holds decorative `.slider__step` dots above the fill.
        if self.show_steps {
            for (index, value) in step_values(self.min, self.max, self.step)
                .into_iter()
                .enumerate()
            {
                let fraction = to_fraction(value);
                let filled = fraction >= fill_from && fraction <= fill_to;
                content = content.child(
                    gpui::div()
                        .absolute()
                        .size(px(3.))
                        .rounded_full()
                        .bg(if filled {
                            sem.foreground.alpha(0.25)
                        } else {
                            colors.muted.alpha(0.25)
                        })
                        .debug_selector(move || format!("slider-step-{index}"))
                        .when(vertical, |dot| {
                            dot.bottom(gpui::relative(fraction))
                                .mb(px(-1.5))
                                .left(track_cross / 2. - px(1.5))
                        })
                        .when(!vertical, |dot| {
                            dot.left(gpui::relative(if rtl { 1. - fraction } else { fraction }))
                                .ml(px(-1.5))
                                .top(track_cross / 2. - px(1.5))
                        }),
                );
            }
        }
        // `.slider__marks` and `.slider__mark` follow the usable track bounds.
        let mut marks = gpui::div()
            .grid()
            .grid_cols(1)
            .relative()
            .text_size(px(12.))
            .line_height(px(16.))
            .font_features(crate::util::tabular_font_features())
            .when(self.is_disabled, |marks| {
                marks.opacity(layout.disabled_opacity)
            });
        let has_marks = self
            .marks
            .iter()
            .any(|mark| mark.value.is_finite() && mark.value >= self.min && mark.value <= self.max);
        for (index, mark) in self.marks.into_iter().enumerate() {
            if !mark.value.is_finite() || mark.value < self.min || mark.value > self.max {
                continue;
            }
            let fraction = to_fraction(mark.value);
            let filled = fraction >= fill_from && fraction <= fill_to;
            let children = if mark.children.is_empty() {
                vec![gpui::div()
                    .child(format_value_labels(&[mark.value], self.format.as_ref()).remove(0))
                    .into_any_element()]
            } else {
                mark.children
            };
            let mark = gpui::div()
                .relative()
                .col_start(1)
                .row_start(1)
                .flex()
                .debug_selector(move || format!("slider-mark-{index}"))
                .text_color(if filled {
                    colors.foreground
                } else {
                    colors.muted
                })
                .when(vertical, |mark| {
                    mark.top(gpui::relative(1. - fraction))
                        .h(px(0.))
                        .items_center()
                })
                .when(!vertical, |mark| {
                    mark.left(gpui::relative(if rtl { 1. - fraction } else { fraction }))
                        .w(px(0.))
                        .justify_center()
                })
                .child(gpui::div().flex_shrink_0().children(children));
            let mark_id = element_id::indexed(&self.id, "mark-color", index);
            let target = if filled {
                colors.foreground
            } else {
                colors.muted
            };
            let mut color = crate::anim::Tween::keyed(&mark_id, "color", target, window, cx);
            let reduce = ActiveTheme::reduce_motion(cx);
            color.snap_if_reduced(reduce);
            marks = if color.animates(reduce) {
                let from = color.from();
                let live = color.value();
                marks.child(
                    mark.with_animation(
                        element_id::indexed(&mark_id, "transition", color.generation()),
                        gpui::Animation::new(std::time::Duration::from_millis(200))
                            .with_easing(|t| crate::anim::Curve::Smooth.at(t)),
                        move |mark, t| {
                            let next = herogpui_core::mix_oklab(from, target, t);
                            live.set(next);
                            mark.text_color(next)
                        },
                    ),
                )
            } else {
                color.settle();
                marks.child(mark)
            };
        }
        marks = if vertical {
            marks
                .grid_cols_max_content(1)
                .h(px(160.) - axis_inset * 2.)
                .min_w(px(32.))
                .my(axis_inset)
        } else {
            marks.min_h(px(16.)).mx(axis_inset)
        };

        // thumbs
        let dragging_at = dragging.read(cx).active;
        let reduce_motion = ActiveTheme::reduce_motion(cx);
        // `useSliderThumb` puts `aria-valuetext` on every thumb, filled with
        // `state.getThumbValueLabel(index)` — the same formatted string the
        // output row shows.
        let thumb_labels = format_value_labels(&thumbs, self.format.as_ref());
        // The corner the overlay focus ring has to stay concentric with, which
        // is whatever the branches below leave on the thumb wrapper: `Sm` is a
        // `rounded_full` pill, so its corner is half the cross axis; the other
        // sizes are a `small_radius` rounded rect; and a caller-rendered thumb
        // gets no rounding of its own. An `sx` radius override replaces all
        // three, and only its top-left is readable as one radius -- an
        // asymmetric override leaves the ring on the uniform corner.
        let thumb_radius = sx_corners.top_left.unwrap_or(match (&self.thumb, small) {
            (Some(_), _) => px(0.),
            (None, true) => thumb_cross / 2.,
            (None, false) => crate::util::small_radius(cx),
        });
        let commit_ctx = SliderCommitContext {
            range_mode,
            range_own: range_own.clone(),
            own: own.clone(),
            on_change: self.on_change.clone(),
            on_change_all: self.on_change_all.clone(),
            on_change_end: self.on_change_end.clone(),
            on_change_end_all: self.on_change_end_all.clone(),
            form_state: self.form_state.clone(),
            form_thumb_states: self.form_thumb_states.clone(),
            disabled_keys: self.disabled_keys.clone(),
            is_disabled: self.is_disabled,
        };
        let pending_a11y_thumbs = Rc::new(RefCell::new(thumbs.clone()));
        for (index, f) in fractions.iter().copied().enumerate() {
            // Each thumb is an `<input type="range">` upstream: role `slider`,
            // with the thumb's own clamped min/max, the step and the
            // orientation.
            let thumb_min = if range_mode && self.min <= self.max {
                index
                    .checked_sub(1)
                    .and_then(|i| thumbs.get(i))
                    .copied()
                    .unwrap_or(self.min)
            } else {
                self.min
            };
            let thumb_max = if range_mode && self.min <= self.max {
                thumbs.get(index + 1).copied().unwrap_or(self.max)
            } else {
                self.max
            };
            let thumb_range =
                crate::a11y::Range::new(thumb_min as f64, thumb_max as f64, thumbs[index] as f64)
                    .step(self.step as f64)
                    .text(thumb_labels.get(index).cloned());
            let mut thumb_el = gpui::div()
                .id(element_id::indexed(&self.id, "thumb", index))
                .a11y_named(crate::a11y::Role::Slider, &group_name)
                .a11y_range(&thumb_range)
                .a11y_orientation(self.orientation)
                .absolute()
                .flex()
                .items_center()
                .justify_center()
                .when(vertical, |t| {
                    t.bottom(gpui::relative(f))
                        .mb(-thumb_along / 2.)
                        .w(thumb_cross)
                        .h(thumb_along)
                        .when(small, |t| t.left(-cross_overhang))
                })
                .when(!vertical, |t| {
                    t.left(gpui::relative(if rtl { 1. - f } else { f }))
                        .ml(-thumb_along / 2.)
                        .w(thumb_along)
                        .h(thumb_cross)
                        .when(small, |t| t.top(-cross_overhang))
                })
                .flex_shrink_0();
            if thumb_enabled[index] {
                let pending_thumbs = pending_a11y_thumbs.clone();
                let action_steps = steps.clone();
                let (min, max, step) = (self.min, self.max, self.step);
                for action in [
                    crate::a11y::Action::SetValue,
                    crate::a11y::Action::Increment,
                    crate::a11y::Action::Decrement,
                ] {
                    let pending_thumbs = pending_thumbs.clone();
                    let action_commit = commit_ctx.clone();
                    let action_steps = action_steps.clone();
                    thumb_el = thumb_el.a11y_action(action, move |data, window, cx| {
                        apply_slider_a11y_action(
                            action,
                            data,
                            index,
                            min,
                            max,
                            step,
                            &action_steps,
                            &pending_thumbs,
                            &action_commit,
                            window,
                            cx,
                        );
                    });
                }
                if range_span.is_finite() && range_span > 0.0 {
                    let thumb_drag_start = self.on_drag_start.clone();
                    let thumb_focus = focus_handle.clone();
                    let thumb_active = active_thumb.clone();
                    let thumb_dragging = dragging.clone();
                    let thumb_values = thumbs.clone();
                    thumb_el = thumb_el.on_mouse_down(
                        gpui::MouseButton::Left,
                        move |ev: &MouseDownEvent, window, cx| {
                            // Match React Aria `useSliderThumb` / `useSlider`: pressing
                            // directly on a thumb focuses it and begins dragging from its
                            // current value without jumping to the pointer coordinate.
                            cx.stop_propagation();
                            if let Some(cb) = &thumb_drag_start {
                                cb(window, cx);
                            }
                            window.focus(&thumb_focus, cx);
                            thumb_active.update(cx, |active, cx| {
                                if *active != index {
                                    *active = index;
                                    cx.notify();
                                }
                            });
                            thumb_dragging.update(cx, |drag, cx| {
                                drag.active = Some(index);
                                drag.thumb_anchor = Some(ThumbDragAnchor {
                                    start_fraction: f,
                                    start_value: thumb_values[index],
                                    start_position: ev.position,
                                });
                                drag.values = thumb_values.clone();
                                cx.notify();
                            });
                        },
                    );
                }
            }
            thumb_el = match &self.thumb {
                Some(render) => thumb_el.child(render(index, thumbs[index])),
                None if !small => {
                    let target_scale = if dragging_at == Some(index) && !reduce_motion {
                        0.9
                    } else {
                        1.0
                    };
                    let inner = gpui::div()
                        .rounded(crate::util::key_radius(cx))
                        .bg(sem.foreground)
                        .shadow(layout.field_shadow.clone())
                        .map(|inner| crate::util::round_sx_corners(inner, &sx_corners));
                    thumb_el
                        .rounded(crate::util::small_radius(cx))
                        .bg(sem.color)
                        .child(
                            slider_thumb_motion(&self.id, index, target_scale, window, cx)
                                .render(inner, vertical),
                        )
                }
                // `Sm` is a single-layer round knob in the foreground token;
                // there is no inner mark to nest.
                None => thumb_el.rounded_full().bg(colors.foreground),
            };
            thumb_el = crate::util::round_sx_corners(thumb_el, &sx_corners);
            // `.slider__thumb` takes `status-focused` -- the thumb the keys
            // move, while the slider holds a keyboard focus. A disabled thumb
            // never takes it: the roving stop skips disabled thumbs, and this
            // guard also covers a stop stranded by `disabled_keys` changing
            // between frames.
            if index == active_at.min(fractions.len().saturating_sub(1)) && thumb_enabled[index] {
                // The ring rides on `thumb_el` rather than on the scaled
                // inner mark: the wrapper is the element the roving focus
                // lands on, and it is the one whose corner `thumb_radius`
                // describes. The inner mark shrinks under a drag, so a ring
                // there would breathe with the press.
                thumb_el = crate::util::ring_overlay_if_focused(
                    thumb_el,
                    &focus_handle,
                    true,
                    thumb_radius,
                    Vec::new(),
                    window,
                    cx,
                );
            }
            content = content.child(thumb_el);
        }
        track = track.child(content);

        let mut interactive = if small {
            let container = gpui::div()
                .id(self.id.clone())
                .relative()
                .flex()
                .items_center()
                .justify_center();
            let container = if vertical {
                container.w(hit_cross).h(px(160.))
            } else {
                container.w_full().h(hit_cross)
            };
            container.child(track)
        } else {
            track.id(self.id.clone())
        };

        if !self.is_disabled {
            interactive = crate::util::cursor_interactive(interactive, cx);
        } else {
            interactive = interactive.opacity(layout.disabled_opacity);
        }

        // bounds recorder
        let recorder_bounds = bounds_slot.clone();
        interactive = interactive.child(
            gpui::canvas(
                move |bounds: Bounds<gpui::Pixels>, _, cx| {
                    recorder_bounds.update(cx, |slot, _| {
                        *slot = Bounds {
                            origin: gpui::point(
                                f32::from(bounds.origin.x),
                                f32::from(bounds.origin.y),
                            ),
                            size: gpui::size(
                                f32::from(bounds.size.width),
                                f32::from(bounds.size.height),
                            ),
                        };
                    });
                    bounds
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );

        // v3 drives a slider from the keyboard: the arrows step it, Home and End
        // jump to the ends, and Page Up/Down move by a tenth of the range --
        // React Aria's page step. Without this the pointer was the only way to
        // move a value at all.
        if !self.is_disabled || dragging.read(cx).active.is_some() {
            let keys_thumbs = thumbs.clone();
            let key_commit = commit_ctx;
            let held_thumb = active_thumb.clone();
            let keys_enabled = thumb_enabled.clone();
            let (min, max) = (self.min, self.max);
            let snap_step = self.step;
            let step = if snap_step == 0.0 {
                (max - min) / 100.0
            } else {
                snap_step
            };
            let key_steps = steps.clone();
            // A tenth of the range, but never less than one step.
            let page = ((max - min) / 10.0).max(step);
            // A slider whose per-thumb disabled set covers every thumb leaves
            // the tab order, exactly as the group-wide flag does: the handle
            // exists but nothing tracks it, so Tab walks past the control.
            if any_enabled {
                interactive = interactive.track_focus(&focus_handle);
            }
            interactive =
                interactive
                    .key_context("Slider")
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        let key = match (rtl, key) {
                            (true, "left") => "right",
                            (true, "right") => "left",
                            _ => key,
                        };
                        let index = (*held_thumb.read(cx)).min(keys_thumbs.len().saturating_sub(1));
                        let current = keys_thumbs.get(index).copied().unwrap_or(min);
                        // Tab-like movement between the thumbs of a range slider:
                        // the roving stop advances to the next *enabled* thumb,
                        // so it never lands on a disabled one (AGENTS.md's "a
                        // disabled control must leave the tab order"). The scan
                        // is bounded, so an all-disabled set cannot spin.
                        if key == "tab" && keys_thumbs.len() > 1 {
                            let mut next_i = index;
                            for _ in 0..keys_thumbs.len() {
                                next_i = (next_i + 1) % keys_thumbs.len();
                                if keys_enabled[next_i] {
                                    break;
                                }
                            }
                            // `next_i == index` only when no other thumb is
                            // enabled; the stop stays where it is rather than
                            // landing on a disabled thumb.
                            if next_i != index {
                                held_thumb.update(cx, |v, cx| {
                                    *v = next_i;
                                    cx.notify();
                                });
                            }
                            return;
                        }
                        // A disabled thumb answers no key: v3 disables its input,
                        // so the arrows never move it.
                        if !keys_enabled.get(index).copied().unwrap_or(true) {
                            return;
                        }
                        if !min.is_finite() || !max.is_finite() || min > max {
                            return;
                        }
                        let next = match key {
                            "right" | "up" => current + step,
                            "left" | "down" => current - step,
                            "pageup" => current + page,
                            "pagedown" => current - page,
                            "home" => min,
                            "end" => max,
                            _ => return,
                        };
                        // Snapped and clamped the same way a drag is, so the two
                        // cannot land on different values.
                        let next = match key {
                            "right" | "up" if !key_steps.is_empty() => {
                                step_discrete_value(&key_steps, current, true)
                            }
                            "left" | "down" if !key_steps.is_empty() => {
                                step_discrete_value(&key_steps, current, false)
                            }
                            _ => snap_value(next, min, max, snap_step, &key_steps),
                        };
                        key_commit.commit_thumb(index, next, &keys_thumbs, window, cx);
                    });

            let target_down = DragTarget {
                rtl,
                min: self.min,
                span: range_span,
                step: self.step,
                steps: steps.clone(),
                thumbs: thumbs.clone(),
                enabled: thumb_enabled.clone(),
            };
            let on_change_down = self.on_change.clone();
            let all_down = self.on_change_all.clone();
            let focus_for_press = focus_handle.clone();
            let active_for_press = active_thumb;
            let own_down = own.clone();
            let range_own_down = range_own.clone();
            let form_state_down = self.form_state.clone();
            let form_thumb_states_down = self.form_thumb_states.clone();
            let form_disabled_keys_down = self.disabled_keys.clone();
            let form_is_disabled_down = self.is_disabled;
            let b_down = bounds_slot.clone();
            let d_down = dragging.clone();
            let on_drag_start = self.on_drag_start.clone();
            interactive = interactive.on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &MouseDownEvent, window, cx| {
                    let Some(value) =
                        value_from_drag(&b_down, ev.position, None, vertical, &target_down, cx)
                    else {
                        return;
                    };
                    let Some(index) =
                        nearest_thumb(&target_down.thumbs, &target_down.enabled, value)
                    else {
                        return;
                    };
                    if let Some(cb) = &on_drag_start {
                        cb(window, cx);
                    }
                    // React Aria focuses and activates the chosen thumb before
                    // it writes the track press, so the following keys and
                    // pointer moves keep that thumb's identity.
                    window.focus(&focus_for_press, cx);
                    active_for_press.update(cx, |active, cx| {
                        *active = index;
                        cx.notify();
                    });
                    let next = set_thumb(
                        index,
                        value,
                        &target_down.thumbs,
                        range_mode,
                        &range_own_down,
                        &on_change_down,
                        &all_down,
                        &own_down,
                        window,
                        cx,
                    );
                    sync_form_values(
                        &form_state_down,
                        &form_thumb_states_down,
                        &next,
                        form_is_disabled_down,
                        &form_disabled_keys_down,
                    );
                    d_down.update(cx, |drag, cx| {
                        drag.active = Some(index);
                        drag.values = next;
                        drag.thumb_anchor = None;
                        cx.notify();
                    });
                },
            );

            let is_disabled_move = self.is_disabled;
            let target_move = DragTarget {
                rtl,
                min: self.min,
                span: range_span,
                step: self.step,
                steps,
                thumbs,
                enabled: thumb_enabled,
            };
            let on_change_move = self.on_change.clone();
            let all_move = self.on_change_all.clone();
            let own_move = own;
            let range_own_move = range_own;
            let form_state_move = self.form_state.clone();
            let form_thumb_states_move = self.form_thumb_states.clone();
            let form_disabled_keys_move = self.disabled_keys.clone();
            let form_is_disabled_move = self.is_disabled;
            let b_move = bounds_slot;
            let on_drag_end = self.on_drag_end.clone();
            let on_change_end = self.on_change_end.clone();
            let on_change_end_all = self.on_change_end_all.clone();
            interactive = interactive.child(
                gpui::canvas(
                    |bounds, _, _| bounds,
                    move |_, _, window, _| {
                        let d_move = dragging.clone();
                        window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
                            if is_disabled_move
                                || phase != gpui::DispatchPhase::Capture
                                || ev.pressed_button != Some(gpui::MouseButton::Left)
                            {
                                return;
                            }
                            let (Some(index), thumb_anchor) = ({
                                let drag = d_move.read(cx);
                                (drag.active, drag.thumb_anchor)
                            }) else {
                                return;
                            };
                            let Some(value) = value_from_drag(
                                &b_move,
                                ev.position,
                                thumb_anchor,
                                vertical,
                                &target_move,
                                cx,
                            ) else {
                                return;
                            };
                            let values = d_move.read(cx).values.clone();
                            if values
                                .get(index)
                                .is_some_and(|current| current.to_bits() == value.to_bits())
                            {
                                return;
                            }
                            let next = set_thumb(
                                index,
                                value,
                                &values,
                                range_mode,
                                &range_own_move,
                                &on_change_move,
                                &all_move,
                                &own_move,
                                window,
                                cx,
                            );
                            sync_form_values(
                                &form_state_move,
                                &form_thumb_states_move,
                                &next,
                                form_is_disabled_move,
                                &form_disabled_keys_move,
                            );
                            d_move.update(cx, |drag, _| drag.values = next);
                        });

                        let d_up = dragging;
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Capture
                                || event.button != gpui::MouseButton::Left
                            {
                                return;
                            }
                            if d_up.read(cx).active.is_none()
                                && d_up.read(cx).thumb_anchor.is_none()
                            {
                                return;
                            }
                            let final_values = d_up.update(cx, |drag, cx| {
                                drag.thumb_anchor = None;
                                let values = drag.active.take().map(|_| drag.values.clone());
                                cx.notify();
                                values
                            });
                            if let Some(values) = final_values {
                                if range_mode {
                                    if let Some(cb) = &on_change_end_all {
                                        cb(&values, window, cx);
                                    }
                                } else if let Some(cb) = &on_change_end {
                                    cb(&values[0], window, cx);
                                }
                                if let Some(cb) = &on_drag_end {
                                    cb(window, cx);
                                }
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            );
        }

        if any_enabled {
            interactive = crate::util::record_focus_bounds(interactive, &focus_handle, window, cx);
        }
        el = if vertical && has_marks {
            el.child(
                gpui::div()
                    .flex()
                    .gap(px(8.))
                    .child(interactive)
                    .child(marks),
            )
        } else {
            el.child(interactive).when(has_marks, |el| el.child(marks))
        };
        el = crate::util::apply_sx(el, &self.sx);
        el
    }
}

/// Keep cap coverage at the axis inset while letting explicit corners resolve
/// against the track thickness rather than GPUI's narrower cap-strip box.
#[allow(clippy::too_many_arguments)] // the axis, corner and colour inputs are all independent
fn fill_cap(
    vertical: bool,
    at_start: bool,
    inset: gpui::Pixels,
    cross: gpui::Pixels,
    radius: gpui::Pixels,
    corners: &gpui::Corners<Option<gpui::Pixels>>,
    explicit_sx_corners: bool,
    color: gpui::Hsla,
) -> gpui::Div {
    let explicit = explicit_sx_corners;
    // Paint a square-cap footprint under the clipped 12px axis strip. The
    // clip controls coverage; the larger paint box gives GPUI enough length
    // to preserve the full 10px radius instead of clamping it to 6px.
    let length = inset.max(cross);
    // `inset` is the cap's length along the slider axis, not its thickness.
    // Limiting an ordinary cap by it made the medium 20px rail use a 6px
    // radius (12px inset / 2), leaving the filled start visibly squarer than
    // the HeroUI rounded-xl track. The cross-axis half thickness is the
    // geometric limit; an explicit partial `sx` shape keeps the old 6px
    // fallback for its unspecified corners so the caller's corner remains
    // the only custom edge.
    let fallback = if explicit {
        radius.min(inset.min(cross) / 2.)
    } else {
        radius.min(cross / 2.)
    };
    let mut clip = gpui::div().absolute().overflow_hidden();
    let mut paint = gpui::div().absolute().bg(color);
    if vertical {
        clip = clip.left(px(0.)).right(px(0.)).h(inset);
        paint = paint.left(px(0.)).right(px(0.)).h(length);
        if at_start {
            let bottom_left = corners.bottom_left.unwrap_or(fallback);
            let bottom_right = corners.bottom_right.unwrap_or(fallback);
            clip = clip
                .bottom(px(0.))
                .rounded_bl(bottom_left)
                .rounded_br(bottom_right);
            paint = paint
                .bottom(px(0.))
                .rounded_bl(bottom_left)
                .rounded_br(bottom_right);
        } else {
            let top_left = corners.top_left.unwrap_or(fallback);
            let top_right = corners.top_right.unwrap_or(fallback);
            clip = clip.top(px(0.)).rounded_tl(top_left).rounded_tr(top_right);
            paint = paint.top(px(0.)).rounded_tl(top_left).rounded_tr(top_right);
        }
    } else {
        clip = clip.top(px(0.)).bottom(px(0.)).w(inset);
        paint = paint.top(px(0.)).bottom(px(0.)).w(length);
        if at_start {
            let top_left = corners.top_left.unwrap_or(fallback);
            let bottom_left = corners.bottom_left.unwrap_or(fallback);
            clip = clip
                .left(px(0.))
                .rounded_tl(top_left)
                .rounded_bl(bottom_left);
            paint = paint
                .left(px(0.))
                .rounded_tl(top_left)
                .rounded_bl(bottom_left);
        } else {
            let top_right = corners.top_right.unwrap_or(fallback);
            let bottom_right = corners.bottom_right.unwrap_or(fallback);
            clip = clip
                .right(px(0.))
                .rounded_tr(top_right)
                .rounded_br(bottom_right);
            paint = paint
                .right(px(0.))
                .rounded_tr(top_right)
                .rounded_br(bottom_right);
        }
    }
    clip.child(paint)
}

fn live_form_state() -> Rc<RefCell<crate::form::LiveFormFieldState>> {
    Rc::new(RefCell::new(crate::form::LiveFormFieldState {
        commit_validation: None,
        reset_validation: None,
        value: crate::form::FormValue::Number(0.0),
        is_invalid: false,
        is_successful: true,
        focus: None,
        restore: None,
    }))
}

fn sync_form_value(
    state: &Rc<RefCell<crate::form::LiveFormFieldState>>,
    value: f32,
    is_successful: bool,
) {
    let mut state = state.borrow_mut();
    state.value = crate::form::FormValue::Number(f64::from(value));
    state.is_successful = is_successful;
    state.is_invalid = false;
}

fn sync_form_values(
    single: &Rc<RefCell<crate::form::LiveFormFieldState>>,
    thumbs_state: &Rc<RefCell<Vec<Rc<RefCell<crate::form::LiveFormFieldState>>>>>,
    values: &[f32],
    is_disabled: bool,
    disabled_keys: &std::collections::HashSet<usize>,
) {
    if let Some(value) = values.first().copied() {
        sync_form_value(single, value, !is_disabled && !disabled_keys.contains(&0));
    }
    let mut states = thumbs_state.borrow_mut();
    if states.len() < values.len() {
        states.resize_with(values.len(), live_form_state);
    }
    for (index, value) in values.iter().copied().enumerate() {
        sync_form_value(
            &states[index],
            value,
            !is_disabled && !disabled_keys.contains(&index),
        );
    }
}

fn normalize_values(values: &[f32], min: f32, max: f32, step: f32) -> Vec<f32> {
    let mut previous = min;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let upper = values.get(index + 1).copied().unwrap_or(max).min(max);
            let normalized = snap_to_step(*value, previous, upper.max(previous), step);
            previous = normalized;
            normalized
        })
        .collect()
}

/// Compute decimal steps without cumulative addition drift or an off-step endpoint.
fn step_values(min: f32, max: f32, step: f32) -> Vec<f32> {
    if !min.is_finite() || !max.is_finite() || max < min || !step.is_finite() || step <= 0. {
        return Vec::new();
    }
    let intervals = (max - min) / step;
    let count = (intervals + f32::EPSILON * intervals.max(1.) * 4.).floor() + 1.;
    if !(1. ..=1000.).contains(&count) {
        return Vec::new();
    }
    let precision = decimal_places(min).max(decimal_places(step));
    let scale = 10_f64.powi(precision);
    (0..count as usize)
        .map(|index| {
            let raw = f64::from(min) + index as f64 * f64::from(step);
            (((raw * scale).round() / scale) as f32).min(max)
        })
        .collect()
}

fn decimal_places(value: f32) -> i32 {
    let value = value.to_string();
    let (coefficient, exponent) = value
        .split_once(['e', 'E'])
        .map_or((value.as_str(), 0), |(coefficient, exponent)| {
            (coefficient, exponent.parse::<i32>().unwrap_or(0))
        });
    let fraction = coefficient
        .split_once('.')
        .map_or(0, |(_, digits)| digits.len() as i32);
    (fraction - exponent).max(0)
}

/// Everything a drag needs to turn a pointer position into a value.
struct DragTarget {
    rtl: bool,
    min: f32,
    span: f32,
    step: f32,
    steps: std::sync::Arc<[f32]>,
    /// The current thumb set. With more than one, the nearest moves.
    thumbs: Vec<f32>,
    /// Which thumbs may follow the pointer — `Slider.Thumb.isDisabled`'s
    /// index set, resolved per position.
    enabled: Vec<bool>,
}

#[derive(Clone, Copy)]
struct ThumbDragAnchor {
    start_fraction: f32,
    start_value: f32,
    start_position: gpui::Point<gpui::Pixels>,
}

#[derive(Default)]
struct DragState {
    active: Option<usize>,
    values: Vec<f32>,
    thumb_anchor: Option<ThumbDragAnchor>,
}

/// Writes one thumb's value through the uncontrolled copy and both callbacks.
///
/// The pointer path reaches this through `value_from_pointer`, which turns a position
/// into a value first; the keyboard already has the value.
#[allow(clippy::too_many_arguments)]
fn set_thumb(
    index: usize,
    value: f32,
    thumbs: &[f32],
    range_mode: bool,
    range_own: &Option<gpui::Entity<Vec<f32>>>,
    on_change: &Option<OnChange>,
    on_change_all: &Option<OnChangeAll>,
    own: &Option<gpui::Entity<f32>>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<f32> {
    if range_mode {
        let mut next = thumbs.to_vec();
        if let Some(slot) = next.get_mut(index) {
            let min = index
                .checked_sub(1)
                .and_then(|i| thumbs.get(i))
                .copied()
                .unwrap_or(f32::NEG_INFINITY);
            let max = thumbs.get(index + 1).copied().unwrap_or(f32::INFINITY);
            *slot = value.clamp(min, max);
        }
        if next
            .iter()
            .zip(thumbs)
            .any(|(next, current)| next.to_bits() != current.to_bits())
        {
            if let Some(own) = range_own {
                own.update(cx, |current, cx| {
                    *current = next.clone();
                    cx.notify();
                });
            }
            if let Some(cb) = on_change_all {
                cb(&next, window, cx);
            }
        }
        return next;
    }
    if thumbs
        .first()
        .is_some_and(|current| current.to_bits() == value.to_bits())
    {
        return vec![value];
    }
    if let Some(held) = own {
        held.update(cx, |v, cx| {
            *v = value;
            cx.notify();
        });
    }
    if let Some(cb) = on_change {
        cb(&value, window, cx);
    }
    vec![value]
}

fn value_from_drag(
    slot: &gpui::Entity<Bounds<f32>>,
    pos: gpui::Point<gpui::Pixels>,
    anchor: Option<ThumbDragAnchor>,
    vertical: bool,
    target: &DragTarget,
    cx: &App,
) -> Option<f32> {
    let b = *slot.read(cx);
    let extent = if vertical {
        b.size.height
    } else {
        b.size.width
    };
    if extent <= 0.0 || !target.span.is_finite() || target.span <= 0.0 {
        return None;
    }
    let frac = match anchor {
        Some(anchor) => {
            let delta_px = if vertical {
                f32::from(anchor.start_position.y) - f32::from(pos.y)
            } else {
                let dx = f32::from(pos.x) - f32::from(anchor.start_position.x);
                if target.rtl {
                    -dx
                } else {
                    dx
                }
            };
            if delta_px == 0.0 {
                return Some(anchor.start_value);
            }
            (anchor.start_fraction + delta_px / extent).clamp(0.0, 1.0)
        }
        None => {
            let frac = axis_fraction(pos, b, vertical);
            if target.rtl {
                1. - frac
            } else {
                frac
            }
        }
    };
    let raw = target.min + frac * target.span;
    Some(snap_value(
        raw,
        target.min,
        target.min + target.span,
        target.step,
        &target.steps,
    ))
}

fn nearest_thumb(thumbs: &[f32], enabled: &[bool], value: f32) -> Option<usize> {
    thumbs
        .iter()
        .enumerate()
        .min_by(|(a_i, a), (b_i, b)| {
            (**a - value)
                .abs()
                .total_cmp(&(**b - value).abs())
                .then_with(|| b_i.cmp(a_i))
        })
        .map(|(i, _)| i)
        .filter(|index| enabled.get(*index).copied().unwrap_or(true))
}

fn snap_value(value: f32, min: f32, max: f32, step: f32, points: &[f32]) -> f32 {
    points
        .iter()
        .copied()
        .min_by(|a, b| (a - value).abs().total_cmp(&(b - value).abs()))
        .unwrap_or_else(|| snap_to_step(value, min, max, step))
}

fn snap_to_step(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let min64 = f64::from(min);
    let max64 = f64::from(max);
    let step64 = f64::from(step);
    let val64 = f64::from(value);
    if !min64.is_finite()
        || !max64.is_finite()
        || min64 > max64
        || !step64.is_finite()
        || step64 < 0.0
        || !val64.is_finite()
    {
        return value;
    }
    if step64 == 0.0 {
        return val64.clamp(min64, max64) as f32;
    }
    let intervals = (val64 - min64) / step64;
    if !intervals.is_finite() {
        return val64.clamp(min64, max64) as f32;
    }
    let mut snapped = intervals.round() * step64 + min64;
    if !snapped.is_finite() {
        return val64.clamp(min64, max64) as f32;
    }
    if snapped < min64 {
        snapped = min64;
    } else if snapped > max64 {
        let floor_steps = ((max64 - min64) / step64).floor();
        let capped = min64 + floor_steps * step64;
        snapped = if capped.is_finite() && capped >= min64 {
            capped
        } else {
            max64
        };
    }
    let rounded = round_to_step_precision(snapped, min, step);
    if !rounded.is_finite() {
        return val64.clamp(min64, max64) as f32;
    }
    rounded.clamp(min64, max64) as f32
}

fn round_to_step_precision(value: f64, min: f32, step: f32) -> f64 {
    let precision = decimal_places(min).max(decimal_places(step));
    if precision <= 0 {
        return value;
    }
    let scale = 10_f64.powi(precision);
    let scaled = value * scale;
    if !scale.is_finite() || !scaled.is_finite() {
        return value;
    }
    scaled.round() / scale
}

fn step_discrete_value(steps: &[f32], current: f32, forward: bool) -> f32 {
    if forward {
        steps
            .iter()
            .copied()
            .find(|value| *value > current)
            .unwrap_or(current)
    } else {
        steps
            .iter()
            .copied()
            .rev()
            .find(|value| *value < current)
            .unwrap_or(current)
    }
}

/// `data-fill-start`: whether the fill's low end sits at the range's start,
/// which paints the track's start border accent. A single thumb fills from
/// the low end as soon as its value is above min; a range fills from its
/// outermost thumbs, so it starts at the very beginning only when one of
/// them rests on min.
fn fill_reaches_start(thumb_count: usize, fill_from: f32, fill_to: f32) -> bool {
    if thumb_count > 1 {
        fill_from == 0.0
    } else {
        fill_to > 0.0
    }
}

/// `data-fill-end`: whether the fill's high end sits at the range's end,
/// which paints the track's end border accent.
fn fill_reaches_end(fill_to: f32) -> bool {
    fill_to >= 1.0
}

#[derive(Clone)]
struct SliderCommitContext {
    range_mode: bool,
    range_own: Option<gpui::Entity<Vec<f32>>>,
    own: Option<gpui::Entity<f32>>,
    on_change: Option<OnChange>,
    on_change_all: Option<OnChangeAll>,
    on_change_end: Option<OnChange>,
    on_change_end_all: Option<OnChangeAll>,
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
    form_thumb_states: Rc<RefCell<Vec<Rc<RefCell<crate::form::LiveFormFieldState>>>>>,
    disabled_keys: std::collections::HashSet<usize>,
    is_disabled: bool,
}

impl SliderCommitContext {
    fn commit_thumb(
        &self,
        index: usize,
        next: f32,
        current_thumbs: &[f32],
        window: &mut Window,
        cx: &mut App,
    ) -> Vec<f32> {
        let next_values = set_thumb(
            index,
            next,
            current_thumbs,
            self.range_mode,
            &self.range_own,
            &self.on_change,
            &self.on_change_all,
            &self.own,
            window,
            cx,
        );
        sync_form_values(
            &self.form_state,
            &self.form_thumb_states,
            &next_values,
            self.is_disabled,
            &self.disabled_keys,
        );
        if self.range_mode {
            if let Some(callback) = &self.on_change_end_all {
                callback(&next_values, window, cx);
            }
        } else if let Some(callback) = &self.on_change_end {
            callback(&next_values[0], window, cx);
        }
        next_values
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_slider_a11y_action(
    action: crate::a11y::Action,
    data: Option<&crate::a11y::ActionData>,
    index: usize,
    min: f32,
    max: f32,
    step: f32,
    steps: &[f32],
    pending_thumbs: &Rc<RefCell<Vec<f32>>>,
    commit: &SliderCommitContext,
    window: &mut Window,
    cx: &mut App,
) {
    let current_thumbs: Vec<f32> = if commit.range_mode {
        commit.range_own.as_ref().map_or_else(
            || pending_thumbs.borrow().clone(),
            |entity| {
                entity
                    .read(cx)
                    .iter()
                    .copied()
                    .map(|v| snap_value(v, min, max, step, steps))
                    .collect()
            },
        )
    } else {
        commit.own.as_ref().map_or_else(
            || pending_thumbs.borrow().clone(),
            |entity| vec![snap_value(*entity.read(cx), min, max, step, steps)],
        )
    };
    let Some(&live_current) = current_thumbs.get(index) else {
        return;
    };
    let Some(next) = resolve_slider_a11y_action(action, data, min, max, step, steps, live_current)
    else {
        return;
    };
    let next_values = commit.commit_thumb(index, next, &current_thumbs, window, cx);
    *pending_thumbs.borrow_mut() = next_values;
}

fn resolve_slider_a11y_action(
    action: crate::a11y::Action,
    data: Option<&crate::a11y::ActionData>,
    min: f32,
    max: f32,
    step: f32,
    steps: &[f32],
    current: f32,
) -> Option<f32> {
    if !min.is_finite()
        || !max.is_finite()
        || min > max
        || !step.is_finite()
        || step < 0.0
        || !current.is_finite()
    {
        return None;
    }
    let min64 = f64::from(min);
    let max64 = f64::from(max);
    let delta64 = if step == 0.0 {
        (max64 - min64) / 100.0
    } else {
        f64::from(step)
    };
    let raw = match action {
        crate::a11y::Action::SetValue => {
            let Some(crate::a11y::ActionData::NumericValue(value)) = data else {
                return None;
            };
            if !value.is_finite() {
                return None;
            }
            value.clamp(min64, max64) as f32
        }
        crate::a11y::Action::Increment if !steps.is_empty() => {
            step_discrete_value(steps, current, true)
        }
        crate::a11y::Action::Decrement if !steps.is_empty() => {
            step_discrete_value(steps, current, false)
        }
        crate::a11y::Action::Increment => {
            let next = (f64::from(current) + delta64).clamp(min64, max64);
            if !next.is_finite() {
                return None;
            }
            next as f32
        }
        crate::a11y::Action::Decrement => {
            let next = (f64::from(current) - delta64).clamp(min64, max64);
            if !next.is_finite() {
                return None;
            }
            next as f32
        }
        _ => return None,
    };
    let snapped = snap_value(raw, min, max, step, steps);
    snapped.is_finite().then_some(snapped)
}

/// Where the pointer sits along the slider's own axis, as a 0..1 fraction of
/// the track.
///
/// The axis decides *both* halves of the sum, which is what the earlier version
/// got wrong: it returned `-y` for a vertical slider and left the caller
/// subtracting `origin.x` and dividing by `width`, so a vertical track (18px
/// wide, y growing downward) produced a negative numerator and every press and
/// every drag clamped to the minimum. A vertical slider is also inverted -- its
/// zero end is at the *bottom* -- so the fraction is measured up from the
/// track's bottom edge.
fn axis_fraction(pos: gpui::Point<gpui::Pixels>, bounds: Bounds<f32>, vertical: bool) -> f32 {
    let (reach, extent) = if vertical {
        (
            bounds.origin.y + bounds.size.height - f32::from(pos.y),
            bounds.size.height,
        )
    } else {
        (f32::from(pos.x) - bounds.origin.x, bounds.size.width)
    };
    if extent <= 0.0 {
        return 0.0;
    }
    (reach / extent).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{default_output, fill_reaches_end, fill_reaches_start, format_value_labels};

    #[test]
    fn range_output_formats_every_thumb_and_joins_the_labels() {
        let format = herogpui_core::NumberFormat::currency("USD");
        let labels = format_value_labels(&[20., 80.], Some(&format));
        assert_eq!(default_output(&labels), "$20.00 \u{2013} $80.00");
    }

    // The `data-fill-start` / `data-fill-end` rules of v3.2.4's SliderTrack:
    // a single thumb's fill grows from the low end (`fillWidth > 0`), and
    // either form ends when its fill reaches 100%.
    #[test]
    fn fill_start_follows_single_and_range_track_rules() {
        assert!(!fill_reaches_start(1, 0.0, 0.0));
        assert!(fill_reaches_start(1, 0.0, 0.25));
        assert!(!fill_reaches_start(2, 0.25, 0.75));
        assert!(fill_reaches_start(2, 0.0, 0.75));
    }

    #[test]
    fn fill_end_requires_the_full_range() {
        assert!(!fill_reaches_end(0.99));
        assert!(fill_reaches_end(1.0));
    }

    #[test]
    fn default_thumb_drag_scale_uses_a_keyed_reduced_motion_transition() {
        let source = include_str!("slider.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(source.contains("SLIDER_THUMB_TRANSITION_MS: u64 = 250"));
        assert!(source.contains("let motion_id = element_id::indexed(id, \"thumb-motion\", index)"));
        assert!(source.contains("Curve::Out.at(t)"));
        assert!(source.contains("!ActiveTheme::reduce_motion(cx)"));
    }

    #[test]
    fn slider_output_uses_shared_tabular_numeric_features() {
        let source = include_str!("slider.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the Slider implementation is always present");
        assert!(source.contains("font_features(crate::util::tabular_font_features())"));
    }

    #[test]
    fn resolve_slider_a11y_action_handles_snapping_steps_continuous_and_invalid_inputs() {
        use crate::a11y::{Action, ActionData};

        // SetValue snaps to step and clamps within finite bounds.
        assert_eq!(
            super::resolve_slider_a11y_action(
                Action::SetValue,
                Some(&ActionData::NumericValue(37.0)),
                0.0,
                100.0,
                5.0,
                &[],
                20.0,
            ),
            Some(35.0)
        );
        assert_eq!(
            super::resolve_slider_a11y_action(
                Action::SetValue,
                Some(&ActionData::NumericValue(f64::MAX)),
                0.0,
                100.0,
                5.0,
                &[],
                20.0,
            ),
            Some(100.0)
        );

        // Reject non-finite values, wrong ActionData variants, and missing data.
        for bad in [
            Some(ActionData::NumericValue(f64::NAN)),
            Some(ActionData::NumericValue(f64::INFINITY)),
            Some(ActionData::NumericValue(f64::NEG_INFINITY)),
            Some(ActionData::Value("bad".into())),
            None,
        ] {
            assert_eq!(
                super::resolve_slider_a11y_action(
                    Action::SetValue,
                    bad.as_ref(),
                    0.0,
                    100.0,
                    5.0,
                    &[],
                    20.0,
                ),
                None
            );
        }

        // Invalid bounds are rejected across all actions.
        for (min, max) in [
            (100.0, 0.0),
            (f32::NAN, 100.0),
            (0.0, f32::NAN),
            (f32::NEG_INFINITY, 100.0),
            (0.0, f32::INFINITY),
        ] {
            assert_eq!(
                super::resolve_slider_a11y_action(
                    Action::SetValue,
                    Some(&ActionData::NumericValue(50.0)),
                    min,
                    max,
                    5.0,
                    &[],
                    20.0,
                ),
                None
            );
            assert_eq!(
                super::resolve_slider_a11y_action(
                    Action::Increment,
                    None,
                    min,
                    max,
                    5.0,
                    &[],
                    20.0
                ),
                None
            );
            assert_eq!(
                super::resolve_slider_a11y_action(
                    Action::Decrement,
                    None,
                    min,
                    max,
                    5.0,
                    &[],
                    20.0
                ),
                None
            );
        }

        // Discrete .steps(...) visits exact stops.
        let steps = [10.0, 35.0, 80.0];
        let v1 = super::resolve_slider_a11y_action(
            Action::Increment,
            None,
            0.0,
            100.0,
            1.0,
            &steps,
            10.0,
        )
        .unwrap();
        assert_eq!(v1, 35.0);
        let v2 =
            super::resolve_slider_a11y_action(Action::Increment, None, 0.0, 100.0, 1.0, &steps, v1)
                .unwrap();
        assert_eq!(v2, 80.0);
        let v3 =
            super::resolve_slider_a11y_action(Action::Decrement, None, 0.0, 100.0, 1.0, &steps, v2)
                .unwrap();
        assert_eq!(v3, 35.0);

        // Continuous step == 0.0 uses (max - min) / 100.0.
        let c1 =
            super::resolve_slider_a11y_action(Action::Increment, None, 0.0, 200.0, 0.0, &[], 25.0)
                .unwrap();
        assert_eq!(c1, 27.0);
        let c2 =
            super::resolve_slider_a11y_action(Action::Decrement, None, 0.0, 200.0, 0.0, &[], c1)
                .unwrap();
        assert_eq!(c2, 25.0);
        let c3 =
            super::resolve_slider_a11y_action(Action::Decrement, None, 0.0, 200.0, 0.0, &[], c2)
                .unwrap();
        assert_eq!(c3, 23.0);

        // Fractional min offset precision (min = 0.25, step = 0.1) must preserve min's decimal places.
        assert_eq!(
            super::resolve_slider_a11y_action(
                Action::SetValue,
                Some(&ActionData::NumericValue(0.35)),
                0.25,
                1.0,
                0.1,
                &[],
                0.25,
            ),
            Some(0.35)
        );
        assert_eq!(
            super::resolve_slider_a11y_action(Action::Increment, None, 0.25, 1.0, 0.1, &[], 0.25,),
            Some(0.35)
        );
    }

    #[gpui::test]
    fn range_set_thumb_clamps_to_neighbor_and_reports_end_values(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext;
        use std::sync::{Arc, Mutex};

        let (_, vcx) = cx.add_window_view(|_, _| gpui::Empty);
        let all_changes: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
        let recorded = all_changes.clone();
        let on_all: Option<super::OnChangeAll> = Some(Arc::new(move |vals: &[f32], _, _| {
            recorded.lock().expect("test mutex").push(vals.to_vec());
        }));

        vcx.update(|window, cx| {
            let range_own = Some(cx.new(|_| vec![20.0_f32, 80.0]));
            let next_raw = super::resolve_slider_a11y_action(
                crate::a11y::Action::SetValue,
                Some(&crate::a11y::ActionData::NumericValue(f64::MAX)),
                0.0,
                100.0,
                5.0,
                &[],
                20.0,
            )
            .unwrap();
            let result = super::set_thumb(
                0,
                next_raw,
                &[20.0, 80.0],
                true,
                &range_own,
                &None,
                &on_all,
                &None,
                window,
                cx,
            );
            assert_eq!(result, vec![80.0, 80.0]);
            assert_eq!(
                range_own.as_ref().unwrap().read(cx).as_slice(),
                [80.0, 80.0]
            );
            assert_eq!(
                all_changes.lock().expect("test mutex").as_slice(),
                [vec![80.0, 80.0]]
            );
        });
    }

    #[test]
    fn resolve_slider_a11y_action_handles_sub_f32_steps_and_invalid_steps() {
        use crate::a11y::{Action, ActionData};

        // `0..1` with `step = 1e-39` must not overflow f32 division into infinity/NaN.
        assert_eq!(
            super::resolve_slider_a11y_action(
                Action::SetValue,
                Some(&ActionData::NumericValue(0.5)),
                0.0,
                1.0,
                1e-39,
                &[],
                0.0,
            ),
            Some(0.5)
        );

        // Negative or non-finite step and non-finite current values are rejected.
        for bad_step in [-1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                super::resolve_slider_a11y_action(
                    Action::SetValue,
                    Some(&ActionData::NumericValue(50.0)),
                    0.0,
                    100.0,
                    bad_step,
                    &[],
                    20.0,
                ),
                None
            );
        }
        assert_eq!(
            super::resolve_slider_a11y_action(
                Action::Increment,
                None,
                0.0,
                100.0,
                1.0,
                &[],
                f32::NAN,
            ),
            None
        );
    }

    #[gpui::test]
    fn apply_slider_a11y_action_handles_unsnapped_range_consecutive_actions_form_sync_and_end_callbacks(
        cx: &mut gpui::TestAppContext,
    ) {
        use crate::a11y::Action;
        use gpui::AppContext;
        use std::{
            cell::RefCell,
            collections::HashSet,
            rc::Rc,
            sync::{Arc, Mutex},
        };

        let (_, vcx) = cx.add_window_view(|_, _| gpui::Empty);

        vcx.update(|window, cx| {
            // 1. Uncontrolled range with initially unsnapped `[27., 80.]` and `steps([10., 35., 80.])`:
            // Increment then Decrement before repaint moves `27 -> 35 -> 80 -> 35`, syncing form and end callbacks.
            let steps = [10.0_f32, 35.0, 80.0];
            let range_own = Some(cx.new(|_| vec![27.0_f32, 80.0]));
            let initial_snapped: Vec<f32> = range_own
                .as_ref()
                .unwrap()
                .read(cx)
                .iter()
                .copied()
                .map(|v| super::snap_value(v, 0.0, 100.0, 1.0, &steps))
                .collect();
            assert_eq!(initial_snapped, vec![35.0, 80.0]);
            let pending = Rc::new(RefCell::new(initial_snapped));
            let range_changes: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
            let range_ends: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
            let rc_log = range_changes.clone();
            let re_log = range_ends.clone();
            let on_all: Option<super::OnChangeAll> = Some(Arc::new(move |vals: &[f32], _, _| {
                rc_log.lock().unwrap().push(vals.to_vec());
            }));
            let on_end_all: Option<super::OnChangeAll> =
                Some(Arc::new(move |vals: &[f32], _, _| {
                    re_log.lock().unwrap().push(vals.to_vec());
                }));
            let form_state = super::live_form_state();
            let thumb0_state = super::live_form_state();
            let form_thumbs = Rc::new(RefCell::new(vec![thumb0_state.clone()]));
            let disabled_keys = HashSet::new();
            let range_commit = super::SliderCommitContext {
                range_mode: true,
                range_own: range_own.clone(),
                own: None,
                on_change: None,
                on_change_all: on_all,
                on_change_end: None,
                on_change_end_all: on_end_all,
                form_state,
                form_thumb_states: form_thumbs,
                disabled_keys: disabled_keys.clone(),
                is_disabled: false,
            };

            super::apply_slider_a11y_action(
                Action::Increment,
                None,
                0,
                0.0,
                100.0,
                1.0,
                &steps,
                &pending,
                &range_commit,
                window,
                cx,
            );
            super::apply_slider_a11y_action(
                Action::Decrement,
                None,
                0,
                0.0,
                100.0,
                1.0,
                &steps,
                &pending,
                &range_commit,
                window,
                cx,
            );

            assert_eq!(
                range_own.as_ref().unwrap().read(cx).as_slice(),
                [35.0, 80.0]
            );
            assert_eq!(
                range_changes.lock().unwrap().as_slice(),
                [vec![80.0, 80.0], vec![35.0, 80.0]]
            );
            assert_eq!(
                range_ends.lock().unwrap().as_slice(),
                [vec![80.0, 80.0], vec![35.0, 80.0]]
            );
            assert_eq!(
                thumb0_state.borrow().value,
                crate::form::FormValue::Number(35.0)
            );

            // 2. Controlled single slider (`own = None`): consecutive Increment then Decrement before
            // repaint advances `pending_thumbs` cleanly `20 -> 25 -> 20`.
            let single_pending = Rc::new(RefCell::new(vec![20.0_f32]));
            let single_changes: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let single_ends: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
            let sc_log = single_changes.clone();
            let se_log = single_ends.clone();
            let on_one: Option<super::OnChange> = Some(Arc::new(move |val: &f32, _, _| {
                sc_log.lock().unwrap().push(*val);
            }));
            let on_end_one: Option<super::OnChange> = Some(Arc::new(move |val: &f32, _, _| {
                se_log.lock().unwrap().push(*val);
            }));
            let single_form = super::live_form_state();
            let single_form_thumbs = Rc::new(RefCell::new(Vec::new()));
            let single_commit = super::SliderCommitContext {
                range_mode: false,
                range_own: None,
                own: None,
                on_change: on_one,
                on_change_all: None,
                on_change_end: on_end_one,
                on_change_end_all: None,
                form_state: single_form.clone(),
                form_thumb_states: single_form_thumbs,
                disabled_keys,
                is_disabled: false,
            };

            super::apply_slider_a11y_action(
                Action::Increment,
                None,
                0,
                0.0,
                100.0,
                5.0,
                &[],
                &single_pending,
                &single_commit,
                window,
                cx,
            );
            super::apply_slider_a11y_action(
                Action::Decrement,
                None,
                0,
                0.0,
                100.0,
                5.0,
                &[],
                &single_pending,
                &single_commit,
                window,
                cx,
            );

            assert_eq!(single_changes.lock().unwrap().as_slice(), [25.0, 20.0]);
            assert_eq!(single_ends.lock().unwrap().as_slice(), [25.0, 20.0]);
            assert_eq!(
                single_form.borrow().value,
                crate::form::FormValue::Number(20.0)
            );
        });
    }
}

crate::util::impl_component_styled!(Slider);
