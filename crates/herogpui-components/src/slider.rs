//! Slider — port of `@heroui/slider` (single value).

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    prelude::*, px, App, Bounds, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    RenderOnce, Styled, Window,
};
use herogpui_core::{Color, Orientation};
use herogpui_theme::ActiveTheme;

type Thumb = std::sync::Arc<dyn Fn(usize, f32) -> gpui::AnyElement + 'static>;
type OnChangeAll = std::sync::Arc<dyn Fn(&[f32], &mut Window, &mut App) + 'static>;

type OnChange = std::sync::Arc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// HeroUI Slider (single thumb).
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
    /// `value: number[]` — a multi-thumb slider. `None` is the single-thumb
    /// form, which is `Slider::new`'s value.
    values: Option<Vec<f32>>,
    /// `children` on `Slider.Thumb` — v3's render prop, handed each thumb's
    /// `index` and value.
    thumb: Option<Thumb>,
    on_change_all: Option<OnChangeAll>,
    on_change: Option<OnChange>,
    on_change_end: Option<OnChange>,
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
            values: None,
            thumb: None,
            on_change_all: None,
            on_change: None,
            on_change_end: None,
        }
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.name = Some(name.into());
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
        Some(
            crate::form::FormField::number_value(
                name,
                self.default_value.unwrap_or(self.value) as f64,
            )
            .is_required(false),
        )
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
        self.step = v.max(0.0001);
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

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        // One thumb or many: the single-value form is a set of one, so the rest
        // of the render does not branch.
        let thumbs: Vec<f32> = match &self.values {
            Some(v) if !v.is_empty() => v.clone(),
            _ => vec![value],
        };
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

        // Shared slot where a canvas records the track bounds each frame so
        // mouse handlers can map positions to values.
        let bounds_slot: Rc<Cell<Bounds<f32>>> = Rc::new(Cell::new(Bounds {
            origin: gpui::point(0., 0.),
            size: gpui::size(0., 0.),
        }));
        let dragging: Rc<Cell<bool>> = Rc::new(Cell::new(false));

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
                move |bounds: Bounds<gpui::Pixels>, _, _| {
                    recorder_bounds.set(Bounds {
                        origin: gpui::point(f32::from(bounds.origin.x), f32::from(bounds.origin.y)),
                        size: gpui::size(
                            f32::from(bounds.size.width),
                            f32::from(bounds.size.height),
                        ),
                    });
                    bounds
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );

        // rail
        track = track.child(
            gpui::div()
                .absolute()
                .when(vertical, |r| r.top(px(0.)).bottom(px(0.)).w(rail_h))
                .when(!vertical, |r| r.left(px(0.)).right(px(0.)).h(rail_h))
                .rounded_full()
                .bg(colors.default.soft_hover()),
        );

        // fill
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
            track = track.child(thumb_el);
        }

        if !self.is_disabled {
            let target_down = DragTarget {
                min: self.min,
                span: range_span,
                step: self.step,
                thumbs: thumbs.clone(),
            };
            let on_change_down = self.on_change.clone();
            let all_down = self.on_change_all.clone();
            let own_down = own.clone();
            let b_down = bounds_slot.clone();
            let d_down = dragging.clone();
            track = track.on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &MouseDownEvent, window, cx| {
                    d_down.set(true);
                    set_from_x(
                        &b_down,
                        axis_pos(ev.position, vertical),
                        &target_down,
                        &on_change_down,
                        &all_down,
                        &own_down,
                        window,
                        cx,
                    );
                },
            );

            let target_move = DragTarget {
                min: self.min,
                span: range_span,
                step: self.step,
                thumbs: thumbs.clone(),
            };
            let on_change_move = self.on_change.clone();
            let all_move = self.on_change_all.clone();
            let own_move = own;
            let b_move = bounds_slot.clone();
            let d_move = dragging.clone();
            track = track.on_mouse_move(move |ev: &MouseMoveEvent, window, cx| {
                if d_move.get() {
                    set_from_x(
                        &b_move,
                        axis_pos(ev.position, vertical),
                        &target_move,
                        &on_change_move,
                        &all_move,
                        &own_move,
                        window,
                        cx,
                    );
                }
            });

            let d_up = dragging;
            let on_change_end = self.on_change_end.clone();
            let b_up = bounds_slot;
            let target_up = DragTarget {
                min: self.min,
                span: range_span,
                step: self.step,
                thumbs,
            };
            track = track.on_mouse_up(
                gpui::MouseButton::Left,
                move |ev: &MouseUpEvent, window, cx| {
                    let was_dragging = d_up.get();
                    d_up.set(false);
                    if was_dragging {
                        if let Some(cb) = &on_change_end {
                            set_from_x(
                                &b_up,
                                axis_pos(ev.position, vertical),
                                &target_up,
                                &Some(cb.clone()),
                                &None,
                                // The final value is already in our own copy;
                                // `onChangeEnd` only reports it.
                                &None,
                                window,
                                cx,
                            );
                        }
                    }
                },
            );
        }

        el.child(track)
    }
}

/// Everything a drag needs to turn a pointer position into a value.
struct DragTarget {
    min: f32,
    span: f32,
    step: f32,
    /// The current thumb set. With more than one, the nearest moves.
    thumbs: Vec<f32>,
}

#[allow(clippy::too_many_arguments)] // one struct per call site would be worse
fn set_from_x(
    slot: &Rc<Cell<Bounds<f32>>>,
    x: f32,
    target: &DragTarget,
    on_change: &Option<OnChange>,
    on_change_all: &Option<OnChangeAll>,
    own: &Option<gpui::Entity<f32>>,
    window: &mut Window,
    cx: &mut App,
) {
    let b = slot.get();
    if b.size.width <= 0.0 || target.span <= 0.0 {
        return;
    }
    let frac = ((x - b.origin.x) / b.size.width).clamp(0.0, 1.0);
    let raw = target.min + frac * target.span;
    let snapped =
        ((raw / target.step).round() * target.step).clamp(target.min, target.min + target.span);

    if target.thumbs.len() > 1 {
        // Multi-thumb: whichever thumb is nearest the pointer follows it, so a
        // range slider does not swap ends under the cursor.
        let mut next = target.thumbs.clone();
        let nearest = next
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (**a - snapped).abs().total_cmp(&(**b - snapped).abs()))
            .map_or(0, |(i, _)| i);
        next[nearest] = snapped;
        next.sort_by(f32::total_cmp);
        if let Some(cb) = on_change_all {
            cb(&next, window, cx);
        }
        return;
    }

    // Uncontrolled: move our own copy, or dragging would do nothing.
    if let Some(held) = own {
        held.update(cx, |v, cx| {
            *v = snapped;
            cx.notify();
        });
    }
    if let Some(cb) = on_change {
        cb(snapped, window, cx);
    }
    if let Some(cb) = on_change_all {
        cb(&[snapped], window, cx);
    }
}

/// The pointer coordinate along the slider's own axis.
///
/// A vertical slider is inverted: its zero end is at the bottom, and
/// `set_from_x` subtracts the track origin either way.
fn axis_pos(pos: gpui::Point<gpui::Pixels>, vertical: bool) -> f32 {
    if vertical {
        -f32::from(pos.y)
    } else {
        f32::from(pos.x)
    }
}
