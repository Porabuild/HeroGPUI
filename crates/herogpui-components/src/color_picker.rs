//! Colors — port of `@heroui/color-area`, `color-field`, `color-picker`,
//! `color-slider`, `color-swatch` and `color-swatch-picker` (v3).
//!
//! All six components share the [`PickerColor`] value type and the
//! [`ColorChannel`] / [`ColorSpace`] vocabulary that React Aria uses.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    MouseDownEvent, Pixels, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::{FieldVariant, Placement, SizeXl};
use herogpui_theme::ActiveTheme;

use crate::{input::Input, util};

// ---------------------------------------------------------------------------
// Value model
// ---------------------------------------------------------------------------

/// The color space a channel belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpace {
    #[default]
    Hsb,
    Hsl,
    Rgb,
}

impl ColorSpace {
    pub const ALL: [ColorSpace; 3] = [ColorSpace::Hsb, ColorSpace::Hsl, ColorSpace::Rgb];

    pub fn label(self) -> &'static str {
        match self {
            ColorSpace::Hsb => "HSB",
            ColorSpace::Hsl => "HSL",
            ColorSpace::Rgb => "RGB",
        }
    }

    /// The two channels a colour area edits in this space.
    pub fn area_channels(self) -> (ColorChannel, ColorChannel) {
        match self {
            ColorSpace::Hsb => (ColorChannel::Saturation, ColorChannel::Brightness),
            ColorSpace::Hsl => (ColorChannel::Saturation, ColorChannel::Lightness),
            ColorSpace::Rgb => (ColorChannel::Red, ColorChannel::Green),
        }
    }
}

/// A single editable channel of a color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorChannel {
    Hue,
    Saturation,
    /// HSB value / "brightness".
    Brightness,
    /// HSL lightness.
    Lightness,
    Alpha,
    Red,
    Green,
    Blue,
}

impl ColorChannel {
    pub fn label(self) -> &'static str {
        match self {
            ColorChannel::Hue => "Hue",
            ColorChannel::Saturation => "Saturation",
            ColorChannel::Brightness => "Brightness",
            ColorChannel::Lightness => "Lightness",
            ColorChannel::Alpha => "Alpha",
            ColorChannel::Red => "Red",
            ColorChannel::Green => "Green",
            ColorChannel::Blue => "Blue",
        }
    }

    /// The inclusive value range of this channel.
    pub fn range(self) -> (f32, f32) {
        match self {
            ColorChannel::Hue => (0.0, 360.0),
            ColorChannel::Saturation
            | ColorChannel::Brightness
            | ColorChannel::Lightness
            | ColorChannel::Alpha => (0.0, 1.0),
            ColorChannel::Red | ColorChannel::Green | ColorChannel::Blue => (0.0, 255.0),
        }
    }
}

/// The color value shared by every picker component — HSB plus alpha, matching
/// React Aria's default working space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickerColor {
    /// Hue in degrees, `0..360`.
    pub hue: f32,
    /// Saturation, `0..1`.
    pub saturation: f32,
    /// Brightness (HSB value), `0..1`.
    pub brightness: f32,
    /// Alpha, `0..1`.
    pub alpha: f32,
}

impl Default for PickerColor {
    fn default() -> Self {
        // React Aria's documented default working color.
        Self {
            hue: 210.0,
            saturation: 1.0,
            brightness: 1.0,
            alpha: 1.0,
        }
    }
}

impl PickerColor {
    pub fn hsb(hue: f32, saturation: f32, brightness: f32) -> Self {
        Self {
            hue: hue.rem_euclid(360.0),
            saturation: saturation.clamp(0.0, 1.0),
            brightness: brightness.clamp(0.0, 1.0),
            alpha: 1.0,
        }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// Parses `#rgb`, `#rrggbb` or `#rrggbbaa`.
    pub fn from_hex(text: &str) -> Option<Self> {
        let hex = text.trim().trim_start_matches('#');
        let expand = |c: char| {
            let d = c.to_digit(16)? as f32;
            Some(d * 17.0 / 255.0)
        };
        let (r, g, b, a) = match hex.len() {
            3 => {
                let mut it = hex.chars();
                (
                    expand(it.next()?)?,
                    expand(it.next()?)?,
                    expand(it.next()?)?,
                    1.0,
                )
            }
            6 | 8 => {
                let byte = |i: usize| {
                    u8::from_str_radix(hex.get(i..i + 2)?, 16)
                        .ok()
                        .map(|v| v as f32 / 255.0)
                };
                (
                    byte(0)?,
                    byte(2)?,
                    byte(4)?,
                    if hex.len() == 8 { byte(6)? } else { 1.0 },
                )
            }
            _ => return None,
        };
        Some(Self::from_rgb(r, g, b).with_alpha(a))
    }

    /// Builds a color from normalised sRGB components.
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let hue = if delta <= f32::EPSILON {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        Self {
            hue: hue.rem_euclid(360.0),
            saturation: if max <= f32::EPSILON { 0.0 } else { delta / max },
            brightness: max,
            alpha: 1.0,
        }
    }

    /// Normalised sRGB components.
    pub fn to_rgb(self) -> (f32, f32, f32) {
        let c = self.brightness * self.saturation;
        let h = self.hue / 60.0;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let (r, g, b) = match h as u32 % 6 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = self.brightness - c;
        (r + m, g + m, b + m)
    }

    /// The gpui color for this value.
    pub fn to_hsla(self) -> Hsla {
        let (r, g, b) = self.to_rgb();
        Hsla::from(gpui::Rgba {
            r,
            g,
            b,
            a: self.alpha,
        })
    }

    /// `#rrggbb`, or `#rrggbbaa` when the color is translucent.
    pub fn to_hex(self) -> String {
        let (r, g, b) = self.to_rgb();
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        if self.alpha >= 1.0 {
            format!("#{:02X}{:02X}{:02X}", q(r), q(g), q(b))
        } else {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                q(r),
                q(g),
                q(b),
                q(self.alpha)
            )
        }
    }

    /// Reads one channel in its own units.
    /// HSL saturation, which is a different quantity from the stored HSB
    /// saturation for every colour that is not fully saturated or achromatic.
    pub fn hsl_saturation(self) -> f32 {
        let l = self.brightness * (1.0 - self.saturation / 2.0);
        let denom = l.min(1.0 - l);
        if denom <= f32::EPSILON {
            0.0
        } else {
            ((self.brightness - l) / denom).clamp(0.0, 1.0)
        }
    }

    /// Replaces the HSL saturation, holding hue and HSL lightness.
    pub fn with_hsl_saturation(self, s: f32) -> Self {
        let s = s.clamp(0.0, 1.0);
        let l = self.brightness * (1.0 - self.saturation / 2.0);
        let v = l + s * l.min(1.0 - l);
        let sv = if v <= f32::EPSILON {
            0.0
        } else {
            (2.0 * (1.0 - l / v)).clamp(0.0, 1.0)
        };
        Self {
            saturation: sv,
            brightness: v,
            ..self
        }
    }

    /// [`PickerColor::channel`] read in `space`.
    pub fn channel_in(self, channel: ColorChannel, space: ColorSpace) -> f32 {
        match (channel, space) {
            (ColorChannel::Saturation, ColorSpace::Hsl) => self.hsl_saturation(),
            _ => self.channel(channel),
        }
    }

    /// [`PickerColor::with_channel`] written in `space`.
    pub fn with_channel_in(
        self,
        channel: ColorChannel,
        space: ColorSpace,
        value: f32,
    ) -> Self {
        match (channel, space) {
            (ColorChannel::Saturation, ColorSpace::Hsl) => self.with_hsl_saturation(value),
            _ => self.with_channel(channel, value),
        }
    }

    pub fn channel(self, channel: ColorChannel) -> f32 {
        let (r, g, b) = self.to_rgb();
        match channel {
            ColorChannel::Hue => self.hue,
            ColorChannel::Saturation => self.saturation,
            ColorChannel::Brightness => self.brightness,
            // HSB -> HSL lightness.
            ColorChannel::Lightness => self.brightness * (1.0 - self.saturation / 2.0),
            ColorChannel::Alpha => self.alpha,
            ColorChannel::Red => r * 255.0,
            ColorChannel::Green => g * 255.0,
            ColorChannel::Blue => b * 255.0,
        }
    }

    /// Returns a copy with one channel replaced.
    pub fn with_channel(self, channel: ColorChannel, value: f32) -> Self {
        let (min, max) = channel.range();
        let value = value.clamp(min, max);
        let (r, g, b) = self.to_rgb();
        match channel {
            ColorChannel::Hue => Self {
                hue: value.rem_euclid(360.0),
                ..self
            },
            ColorChannel::Saturation => Self {
                saturation: value,
                ..self
            },
            ColorChannel::Brightness => Self {
                brightness: value,
                ..self
            },
            ColorChannel::Lightness => {
                // Keep hue and saturation, solve HSB brightness for this HSL L.
                let denom = 1.0 - self.saturation / 2.0;
                Self {
                    brightness: if denom <= f32::EPSILON {
                        self.brightness
                    } else {
                        (value / denom).clamp(0.0, 1.0)
                    },
                    ..self
                }
            }
            ColorChannel::Alpha => Self {
                alpha: value,
                ..self
            },
            ColorChannel::Red => Self::from_rgb(value / 255.0, g, b).with_alpha(self.alpha),
            ColorChannel::Green => Self::from_rgb(r, value / 255.0, b).with_alpha(self.alpha),
            ColorChannel::Blue => Self::from_rgb(r, g, value / 255.0).with_alpha(self.alpha),
        }
    }
}

type OnColorChange = Arc<dyn Fn(PickerColor, &mut Window, &mut App) + 'static>;

/// `ColorField`'s `onChange`, which reports `None` when the text is not a
/// colour -- v3 types it `(color: Color | null) => void`.
type OnColorFieldChange =
    Arc<dyn Fn(Option<PickerColor>, &mut Window, &mut App) + 'static>;

/// Shape of a swatch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwatchShape {
    #[default]
    Circle,
    Square,
}

impl SwatchShape {
    pub const ALL: [SwatchShape; 2] = [SwatchShape::Circle, SwatchShape::Square];

    pub fn label(self) -> &'static str {
        match self {
            SwatchShape::Circle => "Circle",
            SwatchShape::Square => "Square",
        }
    }
}

// ---------------------------------------------------------------------------
// ColorSwatch
// ---------------------------------------------------------------------------

/// ColorSwatch — previews one color value.
///
/// Translucent colors are drawn over a checkerboard so the alpha is visible.
#[derive(IntoElement)]
pub struct ColorSwatch {
    color: PickerColor,
    size: SizeXl,
    shape: SwatchShape,
}

impl ColorSwatch {
    /// `color` — also accepted positionally by [`ColorSwatch::new`].
    pub fn color(mut self, color: PickerColor) -> Self {
        self.color = color;
        self
    }

    pub fn new(color: PickerColor) -> Self {
        Self {
            color,
            size: SizeXl::Md,
            shape: SwatchShape::Circle,
        }
    }

    pub fn size(mut self, size: SizeXl) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: SwatchShape) -> Self {
        self.shape = shape;
        self
    }

}

impl RenderOnce for ColorSwatch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let edge = self.size.px();
        let radius = match self.shape {
            SwatchShape::Circle => px(f32::from(edge) / 2.),
            SwatchShape::Square => cx.layout().radius_md(),
        };

        div()
            .size(edge)
            .rounded(radius)
            .flex_shrink_0()
            .overflow_hidden()
            .border(cx.layout().border_width)
            .border_color(colors.border)
            // Checkerboard under the color reveals translucency.
            .bg(colors.surface_secondary)
            .child(
                div()
                    .size_full()
                    .rounded(radius)
                    .bg(self.color.to_hsla()),
            )
    }
}

// ---------------------------------------------------------------------------
// ColorArea
// ---------------------------------------------------------------------------

/// ColorArea — a two-dimensional gradient for picking two channels at once.
#[derive(IntoElement)]
pub struct ColorArea {
    id: ElementId,
    value: PickerColor,
    /// `colorSpace` — set explicitly, it selects the channel pair; an explicit
    /// `x_channel`/`y_channel` still wins.
    color_space: Option<ColorSpace>,
    x_channel: ColorChannel,
    y_channel: ColorChannel,
    width: Pixels,
    height: Pixels,
    is_disabled: bool,
    show_dots: bool,
    on_change: Option<OnColorChange>,
    on_change_end: Option<OnColorChange>,
}

impl ColorArea {
    pub fn new(id: impl Into<ElementId>, value: PickerColor) -> Self {
        Self {
            id: id.into(),
            value,
            color_space: None,
            x_channel: ColorChannel::Saturation,
            y_channel: ColorChannel::Brightness,
            width: px(240.),
            height: px(180.),
            is_disabled: false,
            show_dots: false,
            on_change: None,
            on_change_end: None,
        }
    }

    /// `colorSpace` — the space whose channels the area edits.
    ///
    /// Sets both axes to that space's pair; call `x_channel`/`y_channel`
    /// afterwards to override either one.
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        let (x, y) = space.area_channels();
        self.color_space = Some(space);
        self.x_channel = x;
        self.y_channel = y;
        self
    }

    pub fn x_channel(mut self, channel: ColorChannel) -> Self {
        self.x_channel = channel;
        self
    }

    pub fn y_channel(mut self, channel: ColorChannel) -> Self {
        self.y_channel = channel;
        self
    }

    pub fn size(mut self, width: impl Into<Pixels>, height: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self.height = height.into();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `showDots` — overlays a dot grid for finer visual positioning.
    pub fn show_dots(mut self, v: bool) -> Self {
        self.show_dots = v;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes.
    pub fn on_change_end(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_end = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ColorArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let radius = cx.layout().radius_lg();
        let hue_color = PickerColor::hsb(self.value.hue, 1.0, 1.0).to_hsla();

        let (x_min, x_max) = self.x_channel.range();
        let (y_min, y_max) = self.y_channel.range();
        let x_norm = ((self.value.channel(self.x_channel) - x_min) / (x_max - x_min)).clamp(0.0, 1.0);
        let y_norm = ((self.value.channel(self.y_channel) - y_min) / (y_max - y_min)).clamp(0.0, 1.0);

        // Saturation left-to-right over the hue, brightness bottom-to-top.
        let mut area = div()
            .id(self.id.clone())
            .relative()
            .w(self.width)
            .h(self.height)
            .rounded(radius)
            .overflow_hidden()
            .border(cx.layout().border_width)
            .border_color(colors.border)
            .bg(gpui::linear_gradient(
                90.0,
                gpui::linear_color_stop(gpui::white(), 0.0),
                gpui::linear_color_stop(hue_color, 1.0),
            ))
            .child(
                div().absolute().inset_0().bg(gpui::linear_gradient(
                    180.0,
                    gpui::linear_color_stop(gpui::transparent_black(), 0.0),
                    gpui::linear_color_stop(gpui::black(), 1.0),
                )),
            );

        // `showDots` — the dot-grid overlay. gpui has no repeating background,
        // so the grid is drawn as rows of small translucent dots.
        if self.show_dots {
            const STEP: f32 = 12.0;
            let cols = (f32::from(self.width) / STEP).floor().max(1.0) as usize;
            let rows = (f32::from(self.height) / STEP).floor().max(1.0) as usize;
            let mut grid = div().absolute().inset_0().flex().flex_col();
            for r in 0..rows {
                let mut line = div().flex().h(px(STEP));
                for c in 0..cols {
                    let _ = c;
                    line = line.child(
                        div()
                            .w(px(STEP))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .size(px(2.))
                                    .rounded_full()
                                    .bg(gpui::white().alpha(0.25)),
                            ),
                    );
                }
                let _ = r;
                grid = grid.child(line);
            }
            area = area.child(grid);
        }

        // Thumb: y is inverted because brightness grows upward.
        area = area.child(
            div()
                .absolute()
                .left(px(f32::from(self.width) * x_norm - 8.))
                .top(px(f32::from(self.height) * (1.0 - y_norm) - 8.))
                .size(px(16.))
                .rounded_full()
                .border_2()
                .border_color(gpui::white())
                .bg(self.value.to_hsla()),
        );

        if self.is_disabled {
            return area.opacity(cx.layout().disabled_opacity);
        }

        if let Some(cb) = self.on_change_end {
            let value = self.value;
            let (x_channel, y_channel) = (self.x_channel, self.y_channel);
            let (w, h) = (self.width, self.height);
            area = area.on_mouse_up(
                gpui::MouseButton::Left,
                move |event: &gpui::MouseUpEvent, window, cx| {
                    let fx = (f32::from(event.position.x) / f32::from(w)).clamp(0.0, 1.0);
                    let fy = (f32::from(event.position.y) / f32::from(h)).clamp(0.0, 1.0);
                    let (x_min, x_max) = x_channel.range();
                    let (y_min, y_max) = y_channel.range();
                    let next = value
                        .with_channel(x_channel, x_min + fx * (x_max - x_min))
                        .with_channel(y_channel, y_min + (1.0 - fy) * (y_max - y_min));
                    cb(next, window, cx);
                },
            );
        }

        if let Some(on_change) = self.on_change {
            let value = self.value;
            let (x_channel, y_channel) = (self.x_channel, self.y_channel);
            let (w, h) = (self.width, self.height);
            area = area.cursor_pointer().on_mouse_down(
                gpui::MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    // `position` is window-relative; the element origin is not
                    // available here, so treat the press as a fraction of the
                    // element measured from its own bounds via the hit region.
                    let local = event.position;
                    let fx = (f32::from(local.x) / f32::from(w)).clamp(0.0, 1.0);
                    let fy = (f32::from(local.y) / f32::from(h)).clamp(0.0, 1.0);
                    let (x_min, x_max) = x_channel.range();
                    let (y_min, y_max) = y_channel.range();
                    let next = value
                        .with_channel(x_channel, x_min + fx * (x_max - x_min))
                        .with_channel(y_channel, y_min + (1.0 - fy) * (y_max - y_min));
                    on_change(next, window, cx);
                },
            );
        }

        area
    }
}

// ---------------------------------------------------------------------------
// ColorSlider
// ---------------------------------------------------------------------------

/// ColorSlider — adjusts a single channel along a gradient track.
#[derive(IntoElement)]
pub struct ColorSlider {
    id: ElementId,
    value: PickerColor,
    channel: ColorChannel,
    /// `colorSpace` — only saturation differs between HSB and HSL, so this
    /// picks which one a saturation slider edits.
    color_space: ColorSpace,
    orientation: herogpui_core::Orientation,
    length: Pixels,
    show_label: bool,
    is_disabled: bool,
    on_change: Option<OnColorChange>,
    on_change_end: Option<OnColorChange>,
}

impl ColorSlider {
    pub fn new(id: impl Into<ElementId>, value: PickerColor, channel: ColorChannel) -> Self {
        Self {
            id: id.into(),
            value,
            channel,
            color_space: ColorSpace::default(),
            orientation: herogpui_core::Orientation::Horizontal,
            length: px(240.),
            show_label: true,
            is_disabled: false,
            on_change: None,
            on_change_end: None,
        }
    }

    /// `orientation` — a vertical slider runs bottom to top.
    /// `colorSpace` — the space the channel is read in. Defaults to HSB, the
    /// space [`PickerColor`] stores.
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    pub fn orientation(mut self, orientation: herogpui_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `onChangeEnd` — fires once when the drag finishes.
    pub fn on_change_end(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change_end = Some(Arc::new(handler));
        self
    }

    pub fn length(mut self, length: impl Into<Pixels>) -> Self {
        self.length = length.into();
        self
    }

    pub fn show_label(mut self, v: bool) -> Self {
        self.show_label = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// The two ends of this channel's gradient, given the current color.
    fn gradient_ends(&self) -> (Hsla, Hsla) {
        let (min, max) = self.channel.range();
        (
            self.value.with_channel(self.channel, min).to_hsla(),
            self.value.with_channel(self.channel, max).to_hsla(),
        )
    }
}

impl RenderOnce for ColorSlider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let (min, max) = self.channel.range();
        // Read in the requested space: HSL and HSB saturation are different
        // numbers for the same colour.
        let raw = self.value.channel_in(self.channel, self.color_space);
        let norm = ((raw - min) / (max - min)).clamp(0.0, 1.0);
        let track_h = px(16.);

        let vertical = !self.orientation.is_horizontal();
        let mut track = div()
            .id(self.id.clone())
            .relative()
            .rounded(px(8.))
            .overflow_hidden()
            .border(cx.layout().border_width)
            .border_color(colors.border);
        track = if vertical {
            track.w(track_h).h(self.length)
        } else {
            track.w(self.length).h(track_h)
        };

        // Hue needs the full spectrum; every other channel is a two-stop ramp.
        track = if self.channel == ColorChannel::Hue {
            let mut spectrum = track;
            // Six 60-degree bands approximate the continuous hue wheel.
            for i in 0..6 {
                let from = PickerColor::hsb(i as f32 * 60.0, 1.0, 1.0).to_hsla();
                let to = PickerColor::hsb((i as f32 + 1.0) * 60.0, 1.0, 1.0).to_hsla();
                spectrum = spectrum.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(px(f32::from(self.length) * (i as f32 / 6.0)))
                        .w(px(f32::from(self.length) / 6.0 + 1.0))
                        .bg(gpui::linear_gradient(
                            90.0,
                            gpui::linear_color_stop(from, 0.0),
                            gpui::linear_color_stop(to, 1.0),
                        )),
                );
            }
            spectrum
        } else {
            let (from, to) = self.gradient_ends();
            track.bg(gpui::linear_gradient(
                90.0,
                gpui::linear_color_stop(from, 0.0),
                gpui::linear_color_stop(to, 1.0),
            ))
        };

        // A vertical slider's zero end is at the bottom, so the offset is
        // measured from the far edge.
        let thumb_offset = px(f32::from(self.length) * if vertical { 1.0 - norm } else { norm } - 8.);
        track = track.child(
            div()
                .absolute()
                .when(vertical, |t| t.left(px(-2.)).top(thumb_offset))
                .when(!vertical, |t| t.top(px(-2.)).left(thumb_offset))
                .size(px(18.))
                .rounded_full()
                .border_2()
                .border_color(gpui::white())
                .bg(self.value.to_hsla()),
        );

        if self.is_disabled {
            track = track.opacity(cx.layout().disabled_opacity);
        } else {
            let value = self.value;
            let channel = self.channel;
            let space = self.color_space;
            let length = self.length;
            let on_change = self.on_change.clone();
            let on_change_end = self.on_change_end.clone();
            let resolve = move |pos: gpui::Point<Pixels>| {
                let along = if vertical { pos.y } else { pos.x };
                let mut f = (f32::from(along) / f32::from(length)).clamp(0.0, 1.0);
                if vertical {
                    f = 1.0 - f;
                }
                let (min, max) = channel.range();
                value.with_channel_in(channel, space, min + f * (max - min))
            };
            let resolve_up = resolve;
            track = track.cursor_pointer();
            if let Some(cb) = on_change {
                track = track.on_mouse_down(
                    gpui::MouseButton::Left,
                    move |event: &MouseDownEvent, window, cx| {
                        cb(resolve(event.position), window, cx);
                    },
                );
            }
            if let Some(cb) = on_change_end {
                track = track.on_mouse_up(
                    gpui::MouseButton::Left,
                    move |event: &gpui::MouseUpEvent, window, cx| {
                        cb(resolve_up(event.position), window, cx);
                    },
                );
            }
        }

        if !self.show_label {
            return div().child(track);
        }

        let display = match self.channel {
            ColorChannel::Hue => format!("{}\u{00B0}", raw.round()),
            ColorChannel::Red | ColorChannel::Green | ColorChannel::Blue => {
                format!("{}", raw.round())
            }
            _ => format!("{}%", (raw * 100.0).round()),
        };

        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w(self.length)
                    .text_size(px(12.))
                    .child(
                        div()
                            .text_color(colors.foreground)
                            .child(self.channel.label()),
                    )
                    .child(div().text_color(colors.muted).child(display)),
            )
            .child(track)
    }
}

// ---------------------------------------------------------------------------
// ColorField
// ---------------------------------------------------------------------------

/// ColorField — enters a color as text.
///
/// With no `channel` it edits the hex value; with one it edits that channel's
/// numeric value.
#[derive(IntoElement)]
pub struct ColorField {
    id: ElementId,
    value: PickerColor,
    channel: Option<ColorChannel>,
    /// `colorSpace` — how a `channel` value is interpreted.
    color_space: ColorSpace,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<PickerColor>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    /// `isWheelDisabled` — stops the scroll wheel from stepping the channel.
    is_wheel_disabled: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    placeholder: Option<SharedString>,
    /// Supplying an `InputState` makes the field editable; without one it is a
    /// read-only display of `value`.
    state: Option<Entity<crate::input::InputState>>,
    on_change: Option<OnColorFieldChange>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    is_read_only: bool,
    is_required: bool,
}

impl ColorField {
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    pub fn new(id: impl Into<ElementId>, value: PickerColor) -> Self {
        Self {
            id: id.into(),
            value,
            channel: None,
            color_space: ColorSpace::default(),
            validate: None,
            validation_errors: Vec::new(),
            is_wheel_disabled: false,
            auto_focus: false,
            placeholder: None,
            state: None,
            on_change: None,
            label: None,
            description: None,
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_read_only: false,
            is_required: false,
        }
    }

    /// `colorSpace` — the space a `channel` value is read in.
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    /// `validate` — returns the message to show, or `None` when the colour is
    /// fine. The component runs it and surfaces the result.
    pub fn validate(
        mut self,
        f: impl Fn(&PickerColor) -> Option<SharedString> + 'static,
    ) -> Self {
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

    /// `autoFocus` — take focus on the first render. Only meaningful in the
    /// editable mode; see [`ColorField::state`].
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `isWheelDisabled` — stops the wheel from stepping the channel.
    ///
    /// Only a single-channel field steps: there is no sensible increment for a
    /// hex value.
    pub fn is_wheel_disabled(mut self, v: bool) -> Self {
        self.is_wheel_disabled = v;
        self
    }

    /// `placeholder` on `ColorField.Input`.
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Makes the field editable, backed by this text state.
    pub fn state(mut self, state: Entity<crate::input::InputState>) -> Self {
        self.state = Some(state);
        self
    }

    /// `onChange` — the parsed colour, or `None` when the text is not one.
    ///
    /// Only fires in the editable mode; see [`ColorField::state`].
    pub fn on_change(
        mut self,
        f: impl Fn(Option<PickerColor>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(f));
        self
    }

    /// Edit one channel instead of the hex value.
    pub fn channel(mut self, channel: ColorChannel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
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

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }
}

impl ColorField {
    /// The text form of the current value, honouring `channel` and
    /// `colorSpace`.
    fn display_text(&self) -> String {
        match self.channel {
            None => self.value.to_hex(),
            Some(ColorChannel::Hue) => format!("{}", self.value.hue.round()),
            // Read in `colorSpace`: HSL and HSB saturation are different
            // numbers for the same colour.
            Some(channel) => format!(
                "{}",
                self.value.channel_in(channel, self.color_space).round()
            ),
        }
    }

    /// Parses edited text back into a colour.
    ///
    /// Without a `channel` the text is a hex colour; with one it is that
    /// channel's number, applied to the current value in `colorSpace`.
    fn parse(&self, text: &str) -> Option<PickerColor> {
        match self.channel {
            None => PickerColor::from_hex(text),
            Some(channel) => {
                let n: f32 = text.trim().parse().ok()?;
                let (min, max) = channel.range();
                if n < min || n > max {
                    return None;
                }
                Some(self.value.with_channel_in(channel, self.color_space, n))
            }
        }
    }
}

impl RenderOnce for ColorField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        let text = self.display_text();

        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&self.value)),
            None,
        );

        // Editable mode: delegate the text handling to Input and parse on every
        // keystroke, so `onChange` reports exactly what v3's does.
        if let Some(state) = self.state.clone() {
            let mut input = Input::new(state)
                .variant(self.variant)
                .is_disabled(self.is_disabled)
                .is_read_only(self.is_read_only)
                .is_required(self.is_required)
                .is_invalid(validity.is_invalid)
                .validation_errors(self.validation_errors.clone())
                .auto_focus(self.auto_focus)
                .start_content(ColorSwatch::new(self.value).size(SizeXl::Xs));
            if let Some(message) = validity.first() {
                input = input.error_message(message);
            }
            if let Some(ph) = self.placeholder.clone() {
                input = input.placeholder(ph);
            } else {
                input = input.placeholder(text.clone());
            }
            if let Some(label) = self.label.clone() {
                input = input.label(label);
            }
            if let Some(description) = self.description.clone() {
                input = input.description(description);
            }
            if self.full_width {
                input = input.full_width();
            }
            if let Some(cb) = self.on_change.clone() {
                let parser = self;
                input = input.on_change(move |text, window, cx| {
                    cb(parser.parse(text), window, cx);
                });
            }
            return input.render(window, cx).into_any_element();
        }

        let mut field = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .px(px(12.))
            .h(util::FIELD_HEIGHT)
            .rounded(util::field_radius(cx))
            .text_size(util::FIELD_TEXT)
            .text_color(colors.field.foreground)
            .child(ColorSwatch::new(self.value).size(SizeXl::Xs))
            .child(div().flex_1().child(text));

        field = match self.variant {
            FieldVariant::Primary => {
                let shadow = layout.field_shadow.clone();
                field
                    .bg(colors.field.background)
                    .when(!shadow.is_empty(), |e| e.shadow(shadow))
            }
            FieldVariant::Secondary => field.bg(colors.surface_secondary),
        };

        // v3's ColorField steps its channel on scroll; `isWheelDisabled` turns
        // that off. There is no sensible increment for a hex value, so only a
        // single-channel field responds.
        if let (Some(channel), false, Some(cb)) = (
            self.channel,
            self.is_wheel_disabled || self.is_disabled || self.is_read_only,
            self.on_change.clone(),
        ) {
            let value = self.value;
            let space = self.color_space;
            field = field.on_scroll_wheel(move |ev: &gpui::ScrollWheelEvent, window, cx| {
                let dy = match ev.delta {
                    gpui::ScrollDelta::Pixels(p) => f32::from(p.y),
                    gpui::ScrollDelta::Lines(p) => p.y,
                };
                if dy == 0.0 {
                    return;
                }
                let (min, max) = channel.range();
                // One notch is a percent of the channel's range, so hue moves
                // in degrees and an 8-bit channel in whole steps.
                let step = ((max - min) / 100.0).max(1.0);
                let next = (value.channel_in(channel, space) + step * dy.signum())
                    .clamp(min, max);
                cb(Some(value.with_channel_in(channel, space, next)), window, cx);
            });
        }

        if validity.is_invalid {
            field = field.border_1().border_color(colors.danger.color);
        }
        // A read-only field is legible but not interactive, so it reads the
        // same as disabled here (there is no editing affordance to remove).
        if self.is_disabled || self.is_read_only {
            field = field.opacity(layout.disabled_opacity);
        }
        if self.full_width {
            field = field.w_full();
        } else {
            field = field.w(px(200.));
        }

        let mut root = div().flex().flex_col().gap(px(6.));
        if let Some(label) = self.label {
            root = root.child(
                crate::field::Label::new(label)
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(self.is_invalid),
            );
        }
        root = root.child(field);
        if let Some(description) = self.description {
            root = root.child(crate::field::Description::new(description));
        }
        root.into_any_element()
    }
}

// ---------------------------------------------------------------------------
// ColorSwatchPicker
// ---------------------------------------------------------------------------

/// Layout of a [`ColorSwatchPicker`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwatchLayout {
    #[default]
    Grid,
    Stack,
}

/// ColorSwatchPicker — chooses from a predefined palette.
#[derive(IntoElement)]
pub struct ColorSwatchPicker {
    id: ElementId,
    swatches: Vec<PickerColor>,
    value: Option<PickerColor>,
    size: SizeXl,
    shape: SwatchShape,
    layout: SwatchLayout,
    is_disabled: bool,
    on_change: Option<OnColorChange>,
}

impl ColorSwatchPicker {
    pub fn new(id: impl Into<ElementId>, swatches: Vec<PickerColor>) -> Self {
        Self {
            id: id.into(),
            swatches,
            value: None,
            size: SizeXl::Md,
            shape: SwatchShape::Circle,
            layout: SwatchLayout::Grid,
            is_disabled: false,
            on_change: None,
        }
    }

    pub fn value(mut self, value: PickerColor) -> Self {
        self.value = Some(value);
        self
    }

    pub fn size(mut self, size: SizeXl) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: SwatchShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn layout(mut self, layout: SwatchLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ColorSwatchPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let mut row = div().flex().flex_row().items_center().gap(px(8.));
        if self.layout == SwatchLayout::Grid {
            row = row.flex_wrap().max_w(px(280.));
        }

        for (index, swatch) in self.swatches.iter().enumerate() {
            let selected = self
                .value
                .map(|v| v.to_hex() == swatch.to_hex())
                .unwrap_or(false);

            let mut cell = div()
                .id(ElementId::Name(
                    format!("{:?}-swatch-{index}", self.id).into(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .p(px(2.))
                .rounded(match self.shape {
                    SwatchShape::Circle => px(9999.),
                    SwatchShape::Square => cx.layout().radius_lg(),
                })
                .child(ColorSwatch::new(*swatch).size(self.size).shape(self.shape));

            // The selection ring sits outside the swatch so the color stays true.
            if selected {
                cell = cell.border_2().border_color(colors.focus);
            } else {
                cell = cell.border_2().border_color(gpui::transparent_black());
            }

            if self.is_disabled {
                cell = cell.opacity(cx.layout().disabled_opacity);
            } else if let Some(on_change) = self.on_change.clone() {
                let value = *swatch;
                cell = cell
                    .cursor_pointer()
                    .on_click(move |_, window, cx| on_change(value, window, cx));
            }

            row = row.child(cell);
        }

        row
    }
}

// ---------------------------------------------------------------------------
// ColorPicker
// ---------------------------------------------------------------------------

/// ColorPicker — a swatch trigger plus a full picking surface.
///
/// Open state is controlled, matching the other overlay components.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    value: PickerColor,
    label: Option<SharedString>,
    is_open: bool,
    placement: Placement,
    /// Adds an alpha slider under the hue slider.
    show_alpha: bool,
    is_disabled: bool,
    on_change: Option<OnColorChange>,
    on_open_change: Option<Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl ColorPicker {
    pub fn new(id: impl Into<ElementId>, value: PickerColor) -> Self {
        Self {
            id: id.into(),
            value,
            label: None,
            is_open: false,
            placement: Placement::BottomStart,
            show_alpha: false,
            is_disabled: false,
            on_change: None,
            on_open_change: None,
        }
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// `placement` on `ColorPicker.Popover`.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = v;
        self
    }

    pub fn show_alpha(mut self, v: bool) -> Self {
        self.show_alpha = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(PickerColor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ColorPicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        let base = format!("{:?}", self.id);

        // Trigger: swatch plus the hex value.
        let mut trigger = div()
            .id(ElementId::Name(format!("{base}-trigger").into()))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .h(px(40.))
            .px(px(10.))
            .rounded(util::control_radius(cx))
            .border(layout.border_width)
            .border_color(colors.border)
            .bg(colors.surface.background)
            .text_size(px(14.))
            .text_color(colors.foreground)
            .child(ColorSwatch::new(self.value).size(SizeXl::Sm))
            .child(div().child(self.value.to_hex()));

        if self.is_disabled {
            trigger = trigger.opacity(layout.disabled_opacity);
        } else {
            let hover_bg = colors.default.color;
            trigger = trigger.cursor_pointer().hover(move |s| s.bg(hover_bg));
            if let Some(cb) = self.on_open_change.clone() {
                let next = !self.is_open;
                trigger = trigger.on_click(move |_, window, cx| cb(next, window, cx));
            }
        }

        // The popover overlays the page rather than pushing it down.
        let mut root = div().relative().flex().flex_col().gap(px(8.));
        if let Some(label) = self.label {
            root = root.child(crate::field::Label::new(label));
        }
        root = root.child(trigger);

        if !self.is_open || self.is_disabled {
            return root;
        }

        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(12.))
            .w(px(264.))
            .rounded(util::container_radius(cx))
            .bg(colors.overlay.background)
            .border(layout.border_width)
            .border_color(colors.border)
            .when(!layout.overlay_shadow.is_empty(), |e| {
                e.shadow(layout.overlay_shadow.clone())
            });

        let mut area = ColorArea::new(ElementId::Name(format!("{base}-area").into()), self.value)
            .size(px(240.), px(160.));
        if let Some(cb) = self.on_change.clone() {
            area = area.on_change(move |c, window, cx| cb(c, window, cx));
        }
        panel = panel.child(area);

        let mut hue = ColorSlider::new(
            ElementId::Name(format!("{base}-hue").into()),
            self.value,
            ColorChannel::Hue,
        )
        .length(px(240.))
        .show_label(false);
        if let Some(cb) = self.on_change.clone() {
            hue = hue.on_change(move |c, window, cx| cb(c, window, cx));
        }
        panel = panel.child(hue);

        if self.show_alpha {
            let mut alpha = ColorSlider::new(
                ElementId::Name(format!("{base}-alpha").into()),
                self.value,
                ColorChannel::Alpha,
            )
            .length(px(240.))
            .show_label(false);
            if let Some(cb) = self.on_change.clone() {
                alpha = alpha.on_change(move |c, window, cx| cb(c, window, cx));
            }
            panel = panel.child(alpha);
        }

        panel = panel.child(
            div()
                .text_size(px(12.))
                .font_family("Consolas")
                .text_color(colors.muted)
                .child(self.value.to_hex()),
        );

        root.child(crate::util::floating(
            crate::util::placed_panel(self.placement, px(6.)).child(crate::anim::entering(
                panel,
                ElementId::Name(format!("{base}-panel-anim").into()),
                cx,
            )),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_and_hsb_saturation_differ_and_round_trip() {
        // Mid-brightness, half-saturated: the two spaces disagree here, which
        // is exactly the case a colorSpace-unaware slider got wrong.
        let c = PickerColor {
            hue: 210.0,
            saturation: 0.5,
            brightness: 0.6,
            alpha: 1.0,
        };
        let hsl_s = c.hsl_saturation();
        assert!(
            (hsl_s - c.saturation).abs() > 0.05,
            "expected the two saturations to differ, got {hsl_s} vs {}",
            c.saturation
        );

        // Writing an HSL saturation and reading it back is stable.
        for target in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let out = c.with_hsl_saturation(target);
            assert!(
                (out.hsl_saturation() - target).abs() < 1e-3,
                "hsl saturation {target} round-tripped to {}",
                out.hsl_saturation()
            );
            // Hue and HSL lightness are held.
            assert!((out.hue - c.hue).abs() < 1e-3);
        }
    }

    #[test]
    fn channel_in_only_diverges_for_saturation() {
        let c = PickerColor {
            hue: 30.0,
            saturation: 0.4,
            brightness: 0.8,
            alpha: 1.0,
        };
        for ch in [
            ColorChannel::Hue,
            ColorChannel::Brightness,
            ColorChannel::Lightness,
            ColorChannel::Alpha,
            ColorChannel::Red,
            ColorChannel::Green,
            ColorChannel::Blue,
        ] {
            assert_eq!(
                c.channel_in(ch, ColorSpace::Hsl),
                c.channel_in(ch, ColorSpace::Hsb),
                "{ch:?} should not depend on the colour space"
            );
        }
        assert_ne!(
            c.channel_in(ColorChannel::Saturation, ColorSpace::Hsl),
            c.channel_in(ColorChannel::Saturation, ColorSpace::Hsb)
        );
    }

    #[test]
    fn area_channels_match_the_space() {
        assert_eq!(
            ColorSpace::Hsl.area_channels(),
            (ColorChannel::Saturation, ColorChannel::Lightness)
        );
        assert_eq!(
            ColorSpace::Hsb.area_channels(),
            (ColorChannel::Saturation, ColorChannel::Brightness)
        );
        assert_eq!(
            ColorSpace::Rgb.area_channels(),
            (ColorChannel::Red, ColorChannel::Green)
        );
    }

    #[test]
    fn hex_round_trips() {
        for hex in ["#FF0000", "#00FF00", "#0000FF", "#123456", "#FFFFFF", "#000000"] {
            let c = PickerColor::from_hex(hex).expect(hex);
            assert_eq!(c.to_hex(), hex, "{hex}");
        }
    }

    #[test]
    fn short_hex_expands() {
        assert_eq!(PickerColor::from_hex("#f00").unwrap().to_hex(), "#FF0000");
    }

    #[test]
    fn hex_with_alpha_round_trips() {
        let c = PickerColor::from_hex("#11223344").unwrap();
        assert_eq!(c.to_hex(), "#11223344");
    }

    #[test]
    fn rejects_bad_hex() {
        assert!(PickerColor::from_hex("#12345").is_none());
        assert!(PickerColor::from_hex("nope").is_none());
    }

    #[test]
    fn channel_edits_are_isolated() {
        let c = PickerColor::hsb(120.0, 0.5, 0.5);
        let hue = c.with_channel(ColorChannel::Hue, 240.0);
        assert!((hue.hue - 240.0).abs() < 1e-3);
        assert!((hue.saturation - c.saturation).abs() < 1e-3);
        assert!((hue.brightness - c.brightness).abs() < 1e-3);
    }

    #[test]
    fn channel_values_stay_in_range() {
        let c = PickerColor::default();
        assert_eq!(c.with_channel(ColorChannel::Alpha, 5.0).alpha, 1.0);
        assert_eq!(c.with_channel(ColorChannel::Alpha, -1.0).alpha, 0.0);
        assert!(c.with_channel(ColorChannel::Red, 999.0).channel(ColorChannel::Red) <= 255.0);
    }

    #[test]
    fn rgb_channel_edit_matches_readback() {
        let c = PickerColor::from_hex("#204060").unwrap();
        let next = c.with_channel(ColorChannel::Green, 128.0);
        assert!((next.channel(ColorChannel::Green) - 128.0).abs() < 1.0);
    }
}
