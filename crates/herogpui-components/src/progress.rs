//! ProgressBar / ProgressCircle — port of `@heroui/progress`.

use gpui::{
    prelude::*, px, AnimationExt, App, IntoElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

#[derive(Clone)]
struct ProgressBarMotion {
    target: f32,
    generation: usize,
    from: f32,
    width: std::rc::Rc<std::cell::Cell<f32>>,
}

impl ProgressBarMotion {
    fn retarget(&mut self, target: f32, animate: bool) -> bool {
        let mut changed = false;
        if (self.target - target).abs() > f32::EPSILON {
            self.target = target;
            self.generation = self.generation.wrapping_add(1);
            self.from = self.width.get();
            changed = true;
        }
        if !animate && (self.width.get() - target).abs() > f32::EPSILON {
            self.from = target;
            self.width.set(target);
            changed = true;
        }
        changed
    }
}

struct ProgressBarMotionFrame {
    generation: usize,
    from: f32,
    to: f32,
    width: std::rc::Rc<std::cell::Cell<f32>>,
    animate: bool,
}

impl ProgressBarMotionFrame {
    fn render(self, fill: gpui::Div) -> gpui::AnyElement {
        if !self.animate {
            self.width.set(self.to);
            return fill.w(gpui::relative(self.to)).into_any_element();
        }

        let width = self.width;
        let from = self.from;
        let to = self.to;
        fill.with_animation(
            gpui::ElementId::Name(format!("progress-bar-fill-width-{}", self.generation).into()),
            gpui::Animation::new(std::time::Duration::from_millis(
                crate::anim::PROGRESS_BAR_FILL_MS,
            ))
            .with_easing(|t| crate::anim::Curve::Out.at(t)),
            move |fill, delta| {
                let next = from + (to - from) * delta;
                width.set(next);
                fill.w(gpui::relative(next))
            },
        )
        .into_any_element()
    }
}

fn progress_bar_motion(
    id: &gpui::ElementId,
    target: f32,
    animate: bool,
    window: &mut Window,
    cx: &mut App,
) -> ProgressBarMotionFrame {
    let state = window.use_keyed_state(
        gpui::ElementId::Name(format!("progress-bar-{id:?}-fill-motion").into()),
        cx,
        |_, _| ProgressBarMotion {
            target,
            generation: 0,
            from: target,
            width: std::rc::Rc::new(std::cell::Cell::new(target)),
        },
    );
    let mut current = state.read(cx).clone();
    if current.retarget(target, animate) {
        state.update(cx, |stored, _| *stored = current.clone());
    }
    let should_animate =
        animate && current.generation != 0 && (current.width.get() - target).abs() > f32::EPSILON;
    ProgressBarMotionFrame {
        generation: current.generation,
        from: current.from,
        to: target,
        width: current.width,
        animate: should_animate,
    }
}

/// Linear progress bar.
#[derive(IntoElement)]
pub struct ProgressBar {
    id: gpui::ElementId,
    value: f32,
    min_value: f32,
    max_value: f32,
    size: Size,
    color: Color,
    is_indeterminate: bool,
    label: Option<String>,
    show_value: bool,
    value_label: Option<SharedString>,
    /// `ProgressBar.ValueLabel`'s render props: the closure is handed
    /// `percentage`, `valueText`, and `isIndeterminate`.
    value_content: Option<std::sync::Arc<dyn Fn(f32, &str, bool) -> gpui::AnyElement + 'static>>,
    /// `formatOptions` — how the generated value label is written.
    format: Option<herogpui_core::NumberFormat>,
}

impl ProgressBar {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            size: Size::Md,
            color: Color::Accent,
            is_indeterminate: false,
            label: None,
            show_value: false,
            value_label: None,
            value_content: None,
            format: None,
        }
    }

    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    pub fn min_value(mut self, v: f32) -> Self {
        self.min_value = v;
        self
    }

    pub fn max_value(mut self, v: f32) -> Self {
        self.max_value = v;
        self
    }

    /// `isIndeterminate` — an unbounded operation; the bar sweeps instead of
    /// filling to a fraction.
    pub fn is_indeterminate(mut self, v: bool) -> Self {
        self.is_indeterminate = v;
        self
    }

    /// `ProgressBar.ValueLabel`'s render function — handed `percentage`,
    /// `valueText`, and `isIndeterminate`.
    pub fn value_content(
        mut self,
        render: impl Fn(f32, &str, bool) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(std::sync::Arc::new(render));
        self
    }

    /// `valueLabel` — replaces the generated percentage.
    pub fn value_label(mut self, text: impl Into<SharedString>) -> Self {
        self.value_label = Some(text.into());
        self
    }

    /// `formatOptions` — v3 defaults to `{style: "percent"}`.
    pub fn format_options(mut self, format: herogpui_core::NumberFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Label rendered above the track (`label` + `showValueLabel`).
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn show_value_label(mut self, v: bool) -> Self {
        self.show_value = v;
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let progress_fill_color = if self.color == Color::Default {
            colors.default.foreground
        } else {
            cx.role(self.color).color
        };
        let progress_track_color = colors.default.color;
        let text_color = colors.foreground;
        let (h, radius) = match self.size {
            Size::Sm => (px(4.), crate::util::micro_radius(cx)),
            Size::Md => (px(8.), crate::util::hairline_radius(cx)),
            Size::Lg => (px(12.), crate::util::mark_radius(cx)),
        };
        // Clamp once at entry so the fill, percentage and every formatted
        // label use the same value, matching React Aria's clamp-before-format
        // behavior. Guarded because
        // `f32::clamp` panics when min > max, which `fraction_of` tolerates.
        let value = if self.min_value <= self.max_value {
            self.value.clamp(self.min_value, self.max_value)
        } else {
            self.value
        };
        let fraction = fraction_of(self.value, self.min_value, self.max_value);
        let fill_motion = progress_bar_motion(
            &self.id,
            if self.is_indeterminate { 0.4 } else { fraction },
            !self.is_indeterminate && !ActiveTheme::reduce_motion(cx),
            window,
            cx,
        );

        let mut el = gpui::div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .gap(px(4.))
            .w_full();

        // `.progress-bar__output` / `.meter__output` is the value beside the
        // label, in the row above the track.
        if self.label.is_some() || self.show_value {
            let value_text = if self.is_indeterminate {
                SharedString::from("")
            } else {
                self.value_label.clone().unwrap_or_else(|| {
                    let format = self
                        .format
                        .clone()
                        .unwrap_or_else(herogpui_core::NumberFormat::percent);
                    // A percent format wants the 0..1 fraction; any other
                    // format wants the value itself.
                    let n = if format.style == herogpui_core::NumberStyle::Percent {
                        fraction as f64
                    } else {
                        value as f64
                    };
                    SharedString::from(format.format(n))
                })
            };
            let percentage = if self.is_indeterminate {
                0.0
            } else {
                fraction * 100.
            };
            el = el.child(
                gpui::div()
                    .flex()
                    .justify_between()
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(self.label.clone().unwrap_or_default())
                    .when(self.show_value, |l| match &self.value_content {
                        // `percentage` is 0-100, with 0 standing in for v3's
                        // undefined indeterminate percentage.
                        Some(render) => {
                            l.child(render(percentage, &value_text, self.is_indeterminate))
                        }
                        None => l.child(value_text.to_string()),
                    }),
            );
        }

        // `.progress-bar__track` / `.meter__track`, with
        // `.progress-bar__fill` / `.meter__fill` inside it.
        let track = gpui::div()
            .w_full()
            .h(h)
            .overflow_hidden()
            .rounded(radius)
            .bg(progress_track_color);

        // Indeterminate bars sweep a 40% segment; reduced motion leaves that
        // same segment static so the state is still legible.
        let track = if self.is_indeterminate && !ActiveTheme::reduce_motion(cx) {
            track
                .child(
                    gpui::div()
                        .relative()
                        .h_full()
                        .w(gpui::relative(0.4))
                        .rounded(radius)
                        .bg(progress_fill_color)
                        .with_animation(
                            "progress-bar-indeterminate",
                            gpui::Animation::new(std::time::Duration::from_millis(
                                crate::anim::PROGRESS_BAR_INDETERMINATE_MS,
                            ))
                            .with_easing(crate::anim::progress_bar_indeterminate_ease())
                            .repeat(),
                            |el, delta| el.left(gpui::relative(delta * 1.8 - 0.4)),
                        ),
                )
                .into_any_element()
        } else if self.is_indeterminate {
            track
                .child(
                    gpui::div()
                        .h_full()
                        .rounded(radius)
                        .bg(progress_fill_color)
                        .w(gpui::relative(0.4)),
                )
                .into_any_element()
        } else {
            let indicator = gpui::div().h_full().rounded(radius).bg(progress_fill_color);
            track
                .child(fill_motion.render(indicator))
                .into_any_element()
        };

        el.child(track)
    }
}

/// Normalises `value` into `0.0..=1.0` across the v3 `minValue`/`maxValue`
/// range, guarding against an empty or inverted range.
fn fraction_of(value: f32, min: f32, max: f32) -> f32 {
    let span = max - min;
    if span.abs() < f32::EPSILON {
        return 0.0;
    }
    ((value - min) / span).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::ProgressBarMotion;
    use std::cell::Cell;
    use std::rc::Rc;

    fn close(left: f32, right: f32) -> bool {
        (left - right).abs() < f32::EPSILON
    }

    #[test]
    fn progress_bar_motion_reverses_from_current_width_and_snaps_without_motion() {
        let width = Rc::new(Cell::new(0.25));
        let mut motion = ProgressBarMotion {
            target: 0.25,
            generation: 0,
            from: 0.25,
            width: width.clone(),
        };

        assert!(motion.retarget(0.75, true));
        assert_eq!(motion.generation, 1);
        assert!(close(motion.from, 0.25));
        assert!(close(width.get(), 0.25));

        width.set(0.5);
        assert!(motion.retarget(0.1, true));
        assert_eq!(motion.generation, 2);
        assert!(close(motion.from, 0.5));
        assert!(close(width.get(), 0.5));

        assert!(motion.retarget(0.9, false));
        assert_eq!(motion.generation, 3);
        assert!(close(motion.from, 0.9));
        assert!(close(width.get(), 0.9));
    }
}

/// Circular progress ring (`ProgressCircle`).
#[derive(IntoElement)]
pub struct ProgressCircle {
    value: f32,
    min_value: f32,
    max_value: f32,
    color: Color,
    size_px: gpui::Pixels,
    is_indeterminate: bool,
    show_value: bool,
    /// `ProgressCircle.ValueLabel`'s render props: `percentage`, `valueText`,
    /// and `isIndeterminate`.
    value_content: Option<std::sync::Arc<dyn Fn(f32, &str, bool) -> gpui::AnyElement + 'static>>,
    /// `formatOptions` — how the generated value label is written.
    format: Option<herogpui_core::NumberFormat>,
}

impl ProgressCircle {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            color: Color::Accent,
            size_px: px(28.),
            is_indeterminate: false,
            show_value: false,
            value_content: None,
            format: None,
        }
    }

    /// `ProgressCircle.ValueLabel`'s render function — handed `percentage`
    /// (0-100), `valueText`, and `isIndeterminate`.
    pub fn value_content(
        mut self,
        render: impl Fn(f32, &str, bool) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.value_content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    pub fn min_value(mut self, v: f32) -> Self {
        self.min_value = v;
        self
    }

    pub fn max_value(mut self, v: f32) -> Self {
        self.max_value = v;
        self
    }

    /// `isIndeterminate` — spins the arc instead of sweeping to a fraction.
    pub fn is_indeterminate(mut self, v: bool) -> Self {
        self.is_indeterminate = v;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// `size` — the ring's diameter: 20 / 28 / 36px for `sm` / `md` / `lg`.
    ///
    /// v3 documents the three-step scale, not a pixel value.
    pub fn size(mut self, s: Size) -> Self {
        self.size_px = match s {
            Size::Sm => px(20.),
            Size::Md => px(28.),
            Size::Lg => px(36.),
        };
        self
    }

    pub fn show_value_label(mut self, v: bool) -> Self {
        self.show_value = v;
        self
    }

    /// `formatOptions` — v3 defaults to `{style: "percent"}`.
    pub fn format_options(mut self, format: herogpui_core::NumberFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl Default for ProgressCircle {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ProgressCircle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let arc_color = if self.color == Color::Default {
            colors.default.foreground
        } else {
            cx.role(self.color).color
        };
        // Clamp once at entry, like the bar: a non-percent label formats the
        // clamped value, and the guard keeps `f32::clamp` from panicking on
        // the inverted range `fraction_of` tolerates.
        let value = if self.min_value <= self.max_value {
            self.value.clamp(self.min_value, self.max_value)
        } else {
            self.value
        };
        let fraction = if self.is_indeterminate {
            // An indeterminate ring shows a fixed quarter arc.
            0.25
        } else {
            fraction_of(self.value, self.min_value, self.max_value)
        };
        // v3 uses stroke-width 4 in a 36-unit viewBox.
        let stroke_w = self.size_px / 9.;
        let spins = self.is_indeterminate && !ActiveTheme::reduce_motion(cx);
        let rotation = std::rc::Rc::new(std::cell::Cell::new(0.0f32));
        let paint_rotation = rotation.clone();

        let arc = gpui::canvas(
            move |bounds, _, _| bounds,
            move |bounds, _, window, _| {
                if fraction <= 0.0 {
                    return;
                }
                let mut builder = gpui::PathBuilder::stroke(stroke_w);
                let center = bounds.center();
                let radius = (bounds.size.width.min(bounds.size.height) / 2.) - stroke_w / 2.;
                // CSS positive rotation is clockwise in screen coordinates;
                // this path's mathematical angle increases counter-clockwise.
                let start = std::f32::consts::FRAC_PI_2 - paint_rotation.get();
                let sweep = std::f32::consts::TAU * fraction;
                let steps = ((sweep / 0.05).ceil() as usize).max(2);
                for i in 0..=steps {
                    let a = start - sweep * i as f32 / steps as f32;
                    let p = gpui::point(center.x + radius * a.cos(), center.y - radius * a.sin());
                    if i == 0 {
                        builder.move_to(p);
                    } else {
                        builder.line_to(p);
                    }
                }
                if let Ok(path) = builder.build() {
                    window.paint_path(path, arc_color);
                }
                // v3's SVG uses stroke-linecap="round". GPUI's public stroke
                // builder uses butt caps, so complete the same geometry with
                // a filled disc at each endpoint.
                for angle in [start, start - sweep] {
                    let cap_center = gpui::point(
                        center.x + radius * angle.cos(),
                        center.y - radius * angle.sin(),
                    );
                    let cap_radius = stroke_w / 2.;
                    let mut cap = gpui::PathBuilder::fill();
                    for step in 0..16 {
                        let angle = std::f32::consts::TAU * step as f32 / 16.;
                        let point = gpui::point(
                            cap_center.x + cap_radius * angle.cos(),
                            cap_center.y + cap_radius * angle.sin(),
                        );
                        if step == 0 {
                            cap.move_to(point);
                        } else {
                            cap.line_to(point);
                        }
                    }
                    cap.close();
                    if let Ok(path) = cap.build() {
                        window.paint_path(path, arc_color);
                    }
                }
            },
        )
        .absolute()
        .inset_0();

        let arc = if spins {
            arc.with_animation(
                "progress-circle-spin",
                gpui::Animation::new(std::time::Duration::from_millis(
                    crate::anim::PROGRESS_CIRCLE_SPIN_MS,
                ))
                .repeat(),
                move |arc, delta| {
                    rotation.set(crate::anim::progress_circle_spin_turn(delta));
                    arc
                },
            )
            .into_any_element()
        } else {
            arc.into_any_element()
        };

        gpui::div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size_px)
            // `.progress-circle__track` and `.progress-circle__track-circle`:
            // v3 draws two
            // SVG circles, one full ring and one dashed to the value.
            .child(
                gpui::div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .border(stroke_w)
                    .border_color(colors.default.color),
            )
            // `.progress-circle__fill-circle` -- the arc, stroked to the same
            // weight as the track it sits on.
            .child(arc)
            .when(self.show_value, |el| {
                let format = self
                    .format
                    .clone()
                    .unwrap_or_else(herogpui_core::NumberFormat::percent);
                // The indeterminate quarter arc is geometry, not a value:
                // React Aria generates no value label for indeterminate
                // progress, so report an empty text and a 0% percentage,
                // like the bar does. The arc keeps drawing regardless.
                let (percentage, value_text) = if self.is_indeterminate {
                    (0.0, String::new())
                } else {
                    let n = if format.style == herogpui_core::NumberStyle::Percent {
                        fraction as f64
                    } else {
                        value as f64
                    };
                    (fraction * 100., format.format(n))
                };
                match &self.value_content {
                    Some(render) => {
                        el.child(render(percentage, &value_text, self.is_indeterminate))
                    }
                    None => el.child(
                        gpui::div()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.foreground)
                            .child(value_text),
                    ),
                }
            })
    }
}
