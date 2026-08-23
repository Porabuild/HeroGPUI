//! Slider — port of `@heroui/slider` (single value).

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    prelude::*, px, App, Bounds, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    RenderOnce, Styled, Window,
};
use herogpui_core::{Color, Orientation};
use herogpui_theme::ActiveTheme;

type OnChange = std::sync::Arc<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// HeroUI Slider (single thumb).
#[derive(IntoElement)]
pub struct Slider {
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
            on_change: None,
            on_change_end: None,
        }
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

    pub fn on_change(
        mut self,
        f: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes, for callers that only
    /// want to commit the final value.
    pub fn on_change_end(
        mut self,
        f: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_end = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        let fraction = if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
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

        let mut el = gpui::div().flex().flex_col().gap(px(4.));

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
                            Some(f) => f.format(self.value as f64),
                            None => format!("{}", self.value),
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
                        size: gpui::size(f32::from(bounds.size.width), f32::from(bounds.size.height)),
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
        track = track.child(
            gpui::div()
                .absolute()
                .rounded_full()
                .bg(sem.color)
                .when(vertical, |f| {
                    f.bottom(px(0.)).w(rail_h).h(gpui::relative(fraction))
                })
                .when(!vertical, |f| {
                    f.left(px(0.)).h(rail_h).w(gpui::relative(fraction))
                }),
        );

        // thumb
        track = track.child(
            gpui::div()
                .absolute()
                .when(vertical, |t| {
                    t.bottom(gpui::relative(fraction)).mb(-thumb_px / 2.)
                })
                .when(!vertical, |t| {
                    t.left(gpui::relative(fraction)).ml(-thumb_px / 2.)
                })
                .size(thumb_px)
                .rounded_full()
                .bg(colors.background)
                .border_2()
                .border_color(sem.color)
                .shadow(layout.surface_shadow.clone())
                .flex_shrink_0(),
        );

        if !self.is_disabled {

            let on_change_down = self.on_change.clone();
            let b_down = bounds_slot.clone();
            let d_down = dragging.clone();
            let (min_d, span_d, step_d) = (self.min, range_span, self.step);
            track = track.on_mouse_down(
                gpui::MouseButton::Left,
                move |ev: &MouseDownEvent, window, cx| {
                    d_down.set(true);
                    set_from_x(&b_down, axis_pos(ev.position, vertical), min_d, span_d, step_d, &on_change_down, window, cx);
                },
            );

            let on_change_move = self.on_change.clone();
            let b_move = bounds_slot.clone();
            let d_move = dragging.clone();
            let (min_m, span_m, step_m) = (self.min, range_span, self.step);
            track = track.on_mouse_move(move |ev: &MouseMoveEvent, window, cx| {
                if d_move.get() {
                    set_from_x(&b_move, axis_pos(ev.position, vertical), min_m, span_m, step_m, &on_change_move, window, cx);
                }
            });

            let d_up = dragging.clone();
            let on_change_end = self.on_change_end.clone();
            let b_up = bounds_slot.clone();
            let (min_u, span_u, step_u) = (self.min, range_span, self.step);
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
                                min_u,
                                span_u,
                                step_u,
                                &Some(cb.clone()),
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

#[allow(clippy::too_many_arguments)]
fn set_from_x(
    slot: &Rc<Cell<Bounds<f32>>>,
    x: f32,
    min: f32,
    span: f32,
    step: f32,
    on_change: &Option<OnChange>,
    window: &mut Window,
    cx: &mut App,
) {
    let b = slot.get();
    if b.size.width <= 0.0 || span <= 0.0 {
        return;
    }
    let frac = ((x - b.origin.x) / b.size.width).clamp(0.0, 1.0);
    let raw = min + frac * span;
    let snapped = ((raw / step).round() * step).clamp(min, min + span);
    if let Some(cb) = on_change {
        cb(snapped, window, cx);
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
