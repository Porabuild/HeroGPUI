//! Slider — port of `@heroui/slider` (single or multi-thumb).

use std::{cell::RefCell, rc::Rc};

use gpui::{
    prelude::*, px, App, Bounds, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    RenderOnce, Styled, Window,
};
use herogpui_core::{Color, Orientation};
use herogpui_theme::ActiveTheme;

type Thumb = std::sync::Arc<dyn Fn(usize, f32) -> gpui::AnyElement + 'static>;
type OnChangeAll = std::sync::Arc<dyn Fn(&[f32], &mut Window, &mut App) + 'static>;

type OnChange = std::sync::Arc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// HeroUI Slider.
#[derive(IntoElement)]
pub struct Slider {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<gpui::SharedString>,
    id: gpui::ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    is_disabled: bool,
    orientation: Orientation,
    label: Option<String>,
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
}

impl Slider {
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

    pub fn new(id: impl Into<gpui::ElementId>, value: f32) -> Self {
        Self {
            name: None,
            id: id.into(),
            value,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            is_disabled: false,
            orientation: Orientation::Horizontal,
            label: None,
            show_value: false,
            format: None,
            default_value: None,
            default_values: None,
            values: None,
            thumb: None,
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
    /// ```ignore
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
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

    pub fn step(mut self, v: f32) -> Self {
        self.step = if v.is_finite() && v > 0.0 { v } else { 1.0 };
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    /// `formatOptions` — how the value read-out is written.
    pub fn format_options(mut self, format: herogpui_core::NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

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

    /// `onChange` for the multi-thumb form — reports every value.
    pub fn on_change_all(mut self, f: impl Fn(&[f32], &mut Window, &mut App) + 'static) -> Self {
        self.on_change_all = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes, for callers that only
    /// want to commit the final value.
    pub fn on_change_end(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
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
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        // Supplying `default_value` is what opts into the uncontrolled mode.
        let (value, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );

        let range_mode = self.values.is_some() || self.default_values.is_some();
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
        let (range_values, range_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-values", self.id).into()),
            range_controlled,
            range_default,
        );

        // One thumb or many: the single-value form is a set of one, so the rest
        // of the render does not branch. Computed before the keyed states
        // because the roving stop's starting position has to skip a disabled
        // thumb.
        let thumbs: Vec<f32> = if range_mode {
            range_values
        } else {
            vec![value]
        };
        let form_defaults = window.use_keyed_state(
            gpui::ElementId::Name(format!("{:?}-form-defaults", self.id).into()),
            cx,
            |_, _| None::<Vec<f32>>,
        );
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
        let restore_form_state = self.form_state.clone();
        let restore_thumb_states = self.form_thumb_states.clone();
        let restore_own = own.clone();
        let restore_range_own = range_own.clone();
        let restore_on_change = self.on_change.clone();
        let restore_on_change_all = self.on_change_all.clone();
        let restore_is_disabled = self.is_disabled;
        let restore_disabled_keys = self.disabled_keys.clone();
        let restore: std::sync::Arc<dyn Fn(&mut Window, &mut App)> =
            crate::util::shared(move |window, cx| {
                if restore_defaults.len() > 1 {
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
                        callback(value, window, cx);
                    }
                }
                sync_form_values(
                    &restore_form_state,
                    &restore_thumb_states,
                    &restore_defaults,
                    restore_is_disabled,
                    &restore_disabled_keys,
                );
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
            .map(|i| !self.disabled_keys.contains(&i))
            .collect();
        let any_enabled = thumb_enabled.iter().any(|&enabled| enabled);

        // The keyboard's own state: the handle that receives the keys and which
        // thumb they move. React Aria focuses one thumb at a time, so a range
        // slider needs to know which.
        let focus_handle = window.use_keyed_state(
            gpui::ElementId::Name(format!("{:?}-focus", self.id).into()),
            cx,
            |_, cx| cx.focus_handle().tab_stop(true),
        );
        let focus_handle = focus_handle.read(cx).clone();
        // The roving stop starts on the first *enabled* thumb, the radio
        // group's rule for its single stop: a stop resting on a disabled
        // thumb would have nothing to answer the keys.
        let active_init = thumb_enabled
            .iter()
            .position(|&enabled| enabled)
            .unwrap_or(0);
        let active_thumb = window.use_keyed_state(
            gpui::ElementId::Name(format!("{:?}-thumb", self.id).into()),
            cx,
            |_, _| active_init,
        );
        let active_at = *active_thumb.read(cx);

        // The drag's state hangs off the window's keyed store, keyed by this
        // slider's own id -- the same shape the Table's column resize uses
        // (`{id}-resizing`). The press repaints the window, and a per-render
        // `Rc<Cell>` would be rebuilt fresh afterwards, which is the defect
        // this replaces: every `on_mouse_move` after the press read a new
        // `false` and no thumb ever moved. Keyed by the id means two sliders
        // on one page can never share a drag.
        let bounds_slot = window.use_keyed_state(
            gpui::ElementId::Name(format!("{:?}-bounds", self.id).into()),
            cx,
            |_, _| Bounds::<f32> {
                origin: gpui::point(0., 0.),
                size: gpui::size(0., 0.),
            },
        );
        let dragging = window.use_keyed_state(
            gpui::ElementId::Name(format!("{:?}-dragging", self.id).into()),
            cx,
            |_, _| DragState::default(),
        );

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

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

        let rail_h = px(4.);
        let thumb_px = px(18.);
        let range_span = self.max - self.min;

        // A horizontal slider fills its container, as v3's does; without a
        // width the label and value read-out collapse together instead of
        // sitting at opposite ends.
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .when(self.orientation.is_horizontal(), |e| e.w_full());

        // `.slider__output` is `text-sm font-medium tabular-nums` beside the
        // label; the two share the row above the track.
        if self.label.is_some() || self.show_value {
            el = el.child(
                gpui::div()
                    .flex()
                    .justify_between()
                    .text_size(px(12.))
                    .text_color(colors.foreground)
                    .child(self.label.clone().unwrap_or_default())
                    .when(self.show_value, |l| {
                        l.child(match &self.format {
                            Some(f) => f.format(value as f64),
                            None => format!("{value}"),
                        })
                    }),
            );
        }

        // A vertical slider swaps the axis: the rail runs top to bottom and
        // the fill grows upward from the zero end.
        let vertical = !self.orientation.is_horizontal();
        let mut track = gpui::div()
            .id(self.id.clone())
            .relative()
            .flex()
            .items_center();
        track = if vertical {
            track.w(thumb_px).h(px(160.))
        } else {
            track.w_full().h(thumb_px)
        };

        if !self.is_disabled {
            track = track.cursor_pointer();
        } else {
            track = track.opacity(layout.disabled_opacity);
        }

        // bounds recorder
        let recorder_bounds = bounds_slot.clone();
        track = track.child(
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

        // `.slider__track` is `relative rounded-xl bg-default` -- the rail the
        // fill and the thumbs sit on.
        track = track.child(
            gpui::div()
                .absolute()
                .when(vertical, |r| r.top(px(0.)).bottom(px(0.)).w(rail_h))
                .when(!vertical, |r| r.left(px(0.)).right(px(0.)).h(rail_h))
                .rounded_full()
                .bg(colors.default.soft_hover()),
        );

        // `.slider__fill` is `pointer-events-none absolute bg-accent`.
        let fill_span = (fill_to - fill_from).max(0.0);
        track = track.child(
            gpui::div()
                .absolute()
                .rounded_full()
                .bg(sem.color)
                .when(vertical, |f| {
                    f.bottom(gpui::relative(fill_from))
                        .w(rail_h)
                        .h(gpui::relative(fill_span))
                })
                .when(!vertical, |f| {
                    f.left(gpui::relative(fill_from))
                        .h(rail_h)
                        .w(gpui::relative(fill_span))
                }),
        );

        // thumbs
        for (index, f) in fractions.iter().copied().enumerate() {
            let mut thumb_el = gpui::div()
                .absolute()
                .when(vertical, |t| t.bottom(gpui::relative(f)).mb(-thumb_px / 2.))
                .when(!vertical, |t| t.left(gpui::relative(f)).ml(-thumb_px / 2.))
                .size(thumb_px)
                .flex_shrink_0();
            thumb_el = match &self.thumb {
                Some(render) => thumb_el.child(render(index, thumbs[index])),
                None => thumb_el
                    .rounded_full()
                    .bg(colors.background)
                    .border_2()
                    .border_color(sem.color)
                    .shadow(layout.surface_shadow.clone()),
            };
            // `.slider__thumb` takes `status-focused` -- the thumb the keys
            // move, while the slider holds a keyboard focus. A disabled thumb
            // never takes it: the roving stop skips disabled thumbs, and this
            // guard also covers a stop stranded by `disabled_keys` changing
            // between frames.
            if index == active_at.min(fractions.len().saturating_sub(1)) && thumb_enabled[index] {
                thumb_el = crate::util::ring_if_focused(
                    thumb_el,
                    &focus_handle,
                    true,
                    Vec::new(),
                    window,
                    cx,
                );
            }
            track = track.child(thumb_el);
        }

        // v3 drives a slider from the keyboard: the arrows step it, Home and End
        // jump to the ends, and Page Up/Down move by a tenth of the range --
        // React Aria's page step. Without this the pointer was the only way to
        // move a value at all.
        if !self.is_disabled {
            let keys_thumbs = thumbs.clone();
            let on_change_keys = self.on_change.clone();
            let all_keys = self.on_change_all.clone();
            let end_keys = self.on_change_end.clone();
            let end_all_keys = self.on_change_end_all.clone();
            let own_keys = own.clone();
            let range_own_keys = range_own.clone();
            let form_state_keys = self.form_state.clone();
            let form_thumb_states_keys = self.form_thumb_states.clone();
            let form_disabled_keys = self.disabled_keys.clone();
            let form_is_disabled = self.is_disabled;
            let held_thumb = active_thumb.clone();
            let keys_enabled = thumb_enabled.clone();
            let (min, max, step) = (self.min, self.max, self.step);
            // A tenth of the range, but never less than one step.
            let page = ((max - min) / 10.0).max(step);
            // A slider whose per-thumb disabled set covers every thumb leaves
            // the tab order, exactly as the group-wide flag does: the handle
            // exists but nothing tracks it, so Tab walks past the control.
            if any_enabled {
                track = track.track_focus(&focus_handle);
            }
            track = track
                .key_context("Slider")
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
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
                    let next = snap_to_step(next, min, max, step);
                    let next_values = set_thumb(
                        index,
                        next,
                        &keys_thumbs,
                        &range_own_keys,
                        &on_change_keys,
                        &all_keys,
                        &own_keys,
                        window,
                        cx,
                    );
                    sync_form_values(
                        &form_state_keys,
                        &form_thumb_states_keys,
                        &next_values,
                        form_is_disabled,
                        &form_disabled_keys,
                    );
                    // A keystroke is a finished change, so `onChangeEnd` fires
                    // with it rather than waiting for a release that never comes.
                    if next_values.len() > 1 {
                        if let Some(cb) = &end_all_keys {
                            cb(&next_values, window, cx);
                        }
                    } else if let Some(cb) = &end_keys {
                        cb(next_values[0], window, cx);
                    }
                });

            let target_down = DragTarget {
                min: self.min,
                span: range_span,
                step: self.step,
                thumbs: thumbs.clone(),
                enabled: thumb_enabled.clone(),
            };
            let on_change_down = self.on_change.clone();
            let all_down = self.on_change_all.clone();
            let focus_for_press = focus_handle;
            let active_for_press = active_thumb;
            let own_down = own.clone();
            let range_own_down = range_own.clone();
            let form_state_down = self.form_state.clone();
            let form_thumb_states_down = self.form_thumb_states.clone();
            let form_disabled_keys_down = self.disabled_keys.clone();
            let form_is_disabled_down = self.is_disabled;
            let b_down = bounds_slot.clone();
            let d_down = dragging.clone();
            track = track.on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &MouseDownEvent, window, cx| {
                    let Some(value) =
                        value_from_pointer(&b_down, ev.position, vertical, &target_down, cx)
                    else {
                        return;
                    };
                    let Some(index) =
                        nearest_thumb(&target_down.thumbs, &target_down.enabled, value)
                    else {
                        return;
                    };
                    // React Aria focuses and activates the chosen thumb before
                    // it writes the track press, so the following keys and
                    // pointer moves keep that thumb's identity.
                    window.focus(&focus_for_press);
                    active_for_press.update(cx, |active, cx| {
                        *active = index;
                        cx.notify();
                    });
                    let next = set_thumb(
                        index,
                        value,
                        &target_down.thumbs,
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
                    d_down.update(cx, |drag, _| {
                        drag.active = Some(index);
                        drag.values = next;
                    });
                },
            );

            let target_move = DragTarget {
                min: self.min,
                span: range_span,
                step: self.step,
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
            let on_change_end = self.on_change_end.clone();
            let on_change_end_all = self.on_change_end_all.clone();
            track = track.child(
                gpui::canvas(
                    |bounds, _, _| bounds,
                    move |_, _, window, _| {
                        let d_move = dragging.clone();
                        window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Capture
                                || ev.pressed_button != Some(gpui::MouseButton::Left)
                            {
                                return;
                            }
                            let (Some(index), values) = ({
                                let drag = d_move.read(cx);
                                (drag.active, drag.values.clone())
                            }) else {
                                return;
                            };
                            if let Some(value) =
                                value_from_pointer(&b_move, ev.position, vertical, &target_move, cx)
                            {
                                let next = set_thumb(
                                    index,
                                    value,
                                    &values,
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
                            }
                        });

                        let d_up = dragging;
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Capture
                                || event.button != gpui::MouseButton::Left
                            {
                                return;
                            }
                            let final_values = d_up.update(cx, |drag, cx| {
                                let values = drag.active.take().map(|_| drag.values.clone());
                                cx.notify();
                                values
                            });
                            if let Some(values) = final_values {
                                if values.len() > 1 {
                                    if let Some(cb) = &on_change_end_all {
                                        cb(&values, window, cx);
                                    }
                                } else if let Some(cb) = &on_change_end {
                                    cb(values[0], window, cx);
                                }
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            );
        }

        el.child(track)
    }
}

fn live_form_state() -> Rc<RefCell<crate::form::LiveFormFieldState>> {
    Rc::new(RefCell::new(crate::form::LiveFormFieldState {
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

/// Everything a drag needs to turn a pointer position into a value.
struct DragTarget {
    min: f32,
    span: f32,
    step: f32,
    /// The current thumb set. With more than one, the nearest moves.
    thumbs: Vec<f32>,
    /// Which thumbs may follow the pointer — `Slider.Thumb.isDisabled`'s
    /// index set, resolved per position.
    enabled: Vec<bool>,
}

#[derive(Default)]
struct DragState {
    active: Option<usize>,
    values: Vec<f32>,
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
    range_own: &Option<gpui::Entity<Vec<f32>>>,
    on_change: &Option<OnChange>,
    on_change_all: &Option<OnChangeAll>,
    own: &Option<gpui::Entity<f32>>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<f32> {
    if thumbs.len() > 1 {
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
        cb(value, window, cx);
    }
    vec![value]
}

fn value_from_pointer(
    slot: &gpui::Entity<Bounds<f32>>,
    pos: gpui::Point<gpui::Pixels>,
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
    if extent <= 0.0 || target.span <= 0.0 {
        return None;
    }
    let frac = axis_fraction(pos, b, vertical);
    let raw = target.min + frac * target.span;
    Some(snap_to_step(
        raw,
        target.min,
        target.min + target.span,
        target.step,
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

fn snap_to_step(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let mut snapped = ((value - min) / step).round() * step + min;
    if snapped < min {
        snapped = min;
    } else if snapped > max {
        snapped = min + ((max - min) / step).floor() * step;
    }
    round_to_step_precision(snapped, step)
}

fn round_to_step_precision(value: f32, step: f32) -> f32 {
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
    let scale = 10_f32.powi(precision);
    if !scale.is_finite() {
        return value;
    }
    (value * scale).round() / scale
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
