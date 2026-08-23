//! ProgressBar / ProgressCircle — port of `@heroui/progress`.

use gpui::{
    prelude::*, px, App, AnimationExt, IntoElement, RenderOnce, SharedString, Styled,
    Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

/// Linear progress bar.
#[derive(IntoElement)]
pub struct ProgressBar {
    value: f32,
    min_value: f32,
    max_value: f32,
    size: Size,
    color: Color,
    is_indeterminate: bool,
    label: Option<String>,
    show_value: bool,
    value_label: Option<SharedString>,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            size: Size::Md,
            color: Color::Accent,
            is_indeterminate: false,
            label: None,
            show_value: false,
            value_label: None,
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

    /// `valueLabel` — replaces the generated percentage.
    pub fn value_label(mut self, text: impl Into<SharedString>) -> Self {
        self.value_label = Some(text.into());
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

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();
        let h = match self.size {
            Size::Sm => px(4.),
            Size::Md => px(8.),
            Size::Lg => px(16.),
        };
        let fraction = fraction_of(self.value, self.min_value, self.max_value);

        let mut el = gpui::div().flex().flex_col().gap(px(4.)).w_full();

        if self.label.is_some() || self.show_value {
            let value_text = self.value_label.clone().unwrap_or_else(|| {
                if self.is_indeterminate {
                    SharedString::from("")
                } else {
                    SharedString::from(format!("{}%", (fraction * 100.0).round() as u32))
                }
            });
            el = el.child(
                gpui::div()
                    .flex()
                    .justify_between()
                    .text_size(px(12.))
                    .text_color(colors.foreground)
                    .child(self.label.clone().unwrap_or_default())
                    .when(self.show_value, |l| l.child(value_text.to_string())),
            );
        }

        let track = gpui::div()
            .w_full()
            .h(h)
            .overflow_hidden()
            .rounded_full()
            .bg(colors.default.soft_hover());

        // Indeterminate bars sweep a short segment; reduced motion falls back
        // to a static two-thirds fill so the state is still legible.
        let track = if self.is_indeterminate && !cx.reduce_motion() {
            let fill = sem.color;
            track
                .child(
                    gpui::div()
                        .relative()
                        .h_full()
                        .w(gpui::relative(0.35))
                        .rounded_full()
                        .bg(fill)
                        .with_animation(
                            "progress-indeterminate",
                            gpui::Animation::new(std::time::Duration::from_millis(1200)).repeat(),
                            |el, delta| el.left(gpui::relative(delta * 1.35 - 0.35)),
                        ),
                )
                .into_any_element()
        } else if self.is_indeterminate {
            track
                .child(
                    gpui::div()
                        .h_full()
                        .rounded_full()
                        .bg(sem.color)
                        .w(gpui::relative(0.66)),
                )
                .into_any_element()
        } else {
            let indicator = gpui::div()
                .h_full()
                .rounded_full()
                .bg(sem.color)
                .w(gpui::relative(fraction));
            track.child(indicator).into_any_element()
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
}

impl ProgressCircle {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min_value: 0.0,
            max_value: 100.0,
            color: Color::Accent,
            size_px: px(48.),
            is_indeterminate: false,
            show_value: false,
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

    /// `isIndeterminate` — spins the arc instead of sweeping to a fraction.
    pub fn is_indeterminate(mut self, v: bool) -> Self {
        self.is_indeterminate = v;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// `size` — the ring's diameter: 32 / 48 / 64px for `sm` / `md` / `lg`.
    ///
    /// v3 documents the three-step scale, not a pixel value.
    pub fn size(mut self, s: Size) -> Self {
        self.size_px = match s {
            Size::Sm => px(32.),
            Size::Md => px(48.),
            Size::Lg => px(64.),
        };
        self
    }


    pub fn show_value_label(mut self, v: bool) -> Self {
        self.show_value = v;
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
        let arc_color = cx.role(self.color).color;
        let fraction = if self.is_indeterminate {
            // An indeterminate ring shows a fixed quarter arc.
            0.25
        } else {
            fraction_of(self.value, self.min_value, self.max_value)
        };
        // v3 scales the ring weight with the circle, ~10% of the diameter.
        let stroke_w = self.size_px * 0.1;

        gpui::div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size_px)
            // Track ring
            .child(
                gpui::div()
                    .absolute()
                    .inset_0()
                    .rounded_full()
                    .border(stroke_w)
                    .border_color(colors.default.soft_hover()),
            )
            // Value arc
            .child(
                gpui::canvas(
                    move |bounds, _, _| bounds,
                    move |bounds, _, window, _| {
                        if fraction <= 0.0 {
                            return;
                        }
                        let mut builder = gpui::PathBuilder::stroke(stroke_w);
                        let center = bounds.center();
                        let radius =
                            (bounds.size.width.min(bounds.size.height) / 2.) - stroke_w / 2.;
                        let start = std::f32::consts::FRAC_PI_2;
                        let sweep = std::f32::consts::TAU * fraction;
                        let steps = ((sweep / 0.05).ceil() as usize).max(2);
                        for i in 0..=steps {
                            let a = start - sweep * i as f32 / steps as f32;
                            let p = gpui::point(
                                center.x + radius * a.cos(),
                                center.y - radius * a.sin(),
                            );
                            if i == 0 {
                                builder.move_to(p);
                            } else {
                                builder.line_to(p);
                            }
                        }
                        if let Ok(path) = builder.build() {
                            window.paint_path(path, arc_color);
                        }
                    },
                )
                .absolute()
                .inset_0(),
            )
            .when(self.show_value, |el| {
                el.child(
                    gpui::div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(colors.foreground)
                        .child(format!(
                            "{}%",
                            (fraction * 100.).round() as u32
                        )),
                )
            })
    }
}


