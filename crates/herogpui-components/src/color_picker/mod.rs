//! Colors — port of `@heroui/color-area`, `color-field`, `color-picker`,
//! `color-slider`, `color-swatch` and `color-swatch-picker` (v3).
//!
//! All six components share the [`PickerColor`] value type and the
//! [`ColorChannel`] / [`ColorSpace`] vocabulary that React Aria uses.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use gpui::{
    div, prelude::*, px, Animation, AnimationExt, App, Bounds, ElementId, Entity, Hsla,
    InteractiveElement, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels,
    RenderOnce, SharedString, Styled, Window,
};
#[allow(unused_imports)] // children import these through `use super::*`
use herogpui_core::{element_id, FieldVariant, Placement, SizeXl};
use herogpui_theme::ActiveTheme;

use crate::{
    a11y::{self, A11y as _},
    input::Input,
    util,
};

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
///
/// HSB coordinates remain readable through dereferencing. Mutations go through
/// `with_channel_in`, which preserves the selected color model's channels even
/// at achromatic endpoints where converting through RGB/HSB would lose them.
#[derive(Clone, Copy, Debug)]
pub struct PickerColor {
    coordinates: HsbCoordinates,
    model: ColorModel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HsbCoordinates {
    /// Hue in degrees, `0..360`.
    pub hue: f32,
    /// Saturation, `0..1`.
    pub saturation: f32,
    /// Brightness (HSB value), `0..1`.
    pub brightness: f32,
    /// Alpha, `0..1`.
    pub alpha: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ColorModel {
    Hsb,
    Hsl { saturation: f32, lightness: f32 },
}

impl std::ops::Deref for PickerColor {
    type Target = HsbCoordinates;

    fn deref(&self) -> &Self::Target {
        &self.coordinates
    }
}

impl PartialEq for PickerColor {
    fn eq(&self, other: &Self) -> bool {
        self.coordinates == other.coordinates
    }
}

impl Default for PickerColor {
    fn default() -> Self {
        // React Aria's documented default working color.
        Self::hsb(210.0, 1.0, 1.0)
    }
}

impl PickerColor {
    pub fn hsb(hue: f32, saturation: f32, brightness: f32) -> Self {
        Self {
            coordinates: HsbCoordinates {
                hue: normalize_hue(hue),
                saturation: saturation.clamp(0.0, 1.0),
                brightness: brightness.clamp(0.0, 1.0),
                alpha: 1.0,
            },
            model: ColorModel::Hsb,
        }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.coordinates.alpha = alpha.clamp(0.0, 1.0);
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
    // `max` is `r.max(g).max(b)`, so `max == r` asks which channel won, not
    // whether two computed floats are near each other. An epsilon here would
    // make two equally-large channels both match.
    #[allow(clippy::float_cmp)]
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        // `max` is by construction one of `r`/`g`/`b`, so these comparisons
        // select a branch rather than test a quantity; comparing within a
        // tolerance would pick the wrong one when two channels are merely close.
        #[allow(clippy::float_cmp)]
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
            coordinates: HsbCoordinates {
                hue: normalize_hue(hue),
                saturation: if max <= f32::EPSILON {
                    0.0
                } else {
                    delta / max
                },
                brightness: max,
                alpha: 1.0,
            },
            model: ColorModel::Hsb,
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
            format!("#{:02X}{:02X}{:02X}{:02X}", q(r), q(g), q(b), q(self.alpha))
        }
    }

    /// Reads one channel in its own units.
    /// HSL saturation, which is a different quantity from the stored HSB
    /// saturation for every colour that is not fully saturated or achromatic.
    pub fn hsl_saturation(self) -> f32 {
        if let ColorModel::Hsl { saturation, .. } = self.model {
            return saturation;
        }
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
        let l = self.hsl_lightness();
        self.with_hsl_channels(s, l)
    }

    pub(super) fn with_hsl_lightness(self, l: f32) -> Self {
        self.with_hsl_channels(self.hsl_saturation(), l.clamp(0.0, 1.0))
    }

    pub(super) fn with_hsl_channels(self, s: f32, l: f32) -> Self {
        let v = l + s * l.min(1.0 - l);
        let sv = if v <= f32::EPSILON {
            0.0
        } else {
            (2.0 * (1.0 - l / v)).clamp(0.0, 1.0)
        };
        Self {
            coordinates: HsbCoordinates {
                saturation: sv,
                brightness: v,
                ..self.coordinates
            },
            model: ColorModel::Hsl {
                saturation: s,
                lightness: l,
            },
        }
    }

    pub(super) fn hsl_lightness(self) -> f32 {
        match self.model {
            ColorModel::Hsl { lightness, .. } => lightness,
            ColorModel::Hsb => self.brightness * (1.0 - self.saturation / 2.0),
        }
    }

    /// [`PickerColor::channel`] read in `space`.
    pub fn channel_in(self, channel: ColorChannel, space: ColorSpace) -> f32 {
        match (channel, space) {
            (ColorChannel::Saturation, ColorSpace::Hsl) => self.hsl_saturation(),
            (ColorChannel::Lightness, ColorSpace::Hsl) => self.hsl_lightness(),
            _ => self.channel(channel),
        }
    }

    /// [`PickerColor::with_channel`] written in `space`.
    pub fn with_channel_in(self, channel: ColorChannel, space: ColorSpace, value: f32) -> Self {
        match (channel, space) {
            (ColorChannel::Saturation, ColorSpace::Hsl) => self.with_hsl_saturation(value),
            (ColorChannel::Lightness, ColorSpace::Hsl) => self.with_hsl_lightness(value),
            (ColorChannel::Hue, ColorSpace::Hsl) => Self {
                coordinates: HsbCoordinates {
                    hue: normalize_hue(value),
                    ..self.coordinates
                },
                ..self
            },
            (ColorChannel::Alpha, ColorSpace::Hsl) => self.with_alpha(value),
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
            ColorChannel::Lightness => self.hsl_lightness(),
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
                coordinates: HsbCoordinates {
                    hue: normalize_hue(value),
                    ..self.coordinates
                },
                model: ColorModel::Hsb,
            },
            ColorChannel::Saturation => Self {
                coordinates: HsbCoordinates {
                    saturation: value,
                    ..self.coordinates
                },
                model: ColorModel::Hsb,
            },
            ColorChannel::Brightness => Self {
                coordinates: HsbCoordinates {
                    brightness: value,
                    ..self.coordinates
                },
                model: ColorModel::Hsb,
            },
            ColorChannel::Lightness => self.with_hsl_lightness(value),
            ColorChannel::Alpha => Self {
                coordinates: HsbCoordinates {
                    alpha: value,
                    ..self.coordinates
                },
                ..self
            },
            ColorChannel::Red => Self::from_rgb(value / 255.0, g, b).with_alpha(self.alpha),
            ColorChannel::Green => Self::from_rgb(r, value / 255.0, b).with_alpha(self.alpha),
            ColorChannel::Blue => Self::from_rgb(r, g, value / 255.0).with_alpha(self.alpha),
        }
    }
}

// React Stately preserves the exact endpoint 360 as a distinct slider
// position even though it renders the same color as 0.
#[allow(clippy::float_cmp)]
pub(super) fn normalize_hue(hue: f32) -> f32 {
    if hue == 360.0 {
        hue
    } else {
        hue.rem_euclid(360.0)
    }
}

pub(super) type OnColorChange = Arc<dyn Fn(&PickerColor, &mut Window, &mut App) + 'static>;

pub(super) fn color_swatch_indicator_color(swatch: PickerColor) -> Hsla {
    let (r, g, b) = swatch.to_rgb();
    let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if luminance > 0.5 {
        gpui::black()
    } else {
        gpui::white()
    }
}

/// `ColorField`'s `onChange`, which reports `None` when the text is not a
/// colour -- v3 types it `(color: Color | null) => void`.
pub(super) type OnColorFieldChange =
    Arc<dyn Fn(&Option<PickerColor>, &mut Window, &mut App) + 'static>;

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

mod area;
mod field;
mod picker;
mod slider;
mod swatch;
mod swatch_picker;

pub use area::*;
#[allow(unused_imports)]
pub(super) use area::*;
pub use field::*;
#[allow(unused_imports)]
pub(super) use field::*;
pub use picker::*;
#[allow(unused_imports)]
pub(super) use picker::*;
pub use slider::*;
#[allow(unused_imports)]
pub(super) use slider::*;
pub use swatch::*;
#[allow(unused_imports)]
pub(super) use swatch::*;
pub use swatch_picker::*;
#[allow(unused_imports)]
pub(super) use swatch_picker::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_and_hsb_saturation_differ_and_round_trip() {
        // Mid-brightness, half-saturated: the two spaces disagree here, which
        // is exactly the case a colorSpace-unaware slider got wrong.
        let c = PickerColor::hsb(210.0, 0.5, 0.6);
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
    fn equality_uses_public_hsb_coordinates_not_color_model_history() {
        let hsl_black = PickerColor::hsb(40.0, 0.5, 0.5)
            .with_channel_in(ColorChannel::Saturation, ColorSpace::Hsl, 0.75)
            .with_channel_in(ColorChannel::Lightness, ColorSpace::Hsl, 0.0);
        let hsb_black = PickerColor::hsb(hsl_black.hue, 0.0, 0.0);
        assert_eq!(hsl_black, hsb_black);
        assert_eq!(hsl_black.to_hex(), hsb_black.to_hex());
    }

    #[test]
    fn hue_gradients_have_seven_stops_and_reverse_on_the_vertical_axis() {
        let stops = hue_stop_colors(PickerColor::hsb(0.0, 1.0, 1.0), ColorSpace::Hsb);
        assert_eq!(stops[0], stops[6]);
        for pair in stops.windows(2).take(5) {
            assert_ne!(pair[0], pair[1]);
        }
        assert!((hue_band_offset(0, false) - 0.0).abs() < f32::EPSILON);
        assert!((hue_band_offset(5, false) - 5.0 / 6.0).abs() < f32::EPSILON);
        assert!((hue_band_offset(0, true) - 5.0 / 6.0).abs() < f32::EPSILON);
        assert!((hue_band_offset(5, true) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lightness_midpoint_preserves_the_current_hue() {
        let value = PickerColor::hsb(0.0, 1.0, 1.0);
        let (start, middle, end) = lightness_gradient_colors(value, ColorSpace::Hsl, 0.0, 1.0);
        assert_eq!(start, gpui::black());
        assert_eq!(middle, value.to_hsla());
        assert_eq!(end, gpui::white());
    }

    #[test]
    #[allow(clippy::float_cmp)] // the two spaces must agree bit for bit
    fn channel_in_only_diverges_for_saturation() {
        let c = PickerColor::hsb(30.0, 0.4, 0.8);
        for ch in [
            ColorChannel::Hue,
            ColorChannel::Brightness,
            ColorChannel::Lightness,
            ColorChannel::Alpha,
            ColorChannel::Red,
            ColorChannel::Green,
            ColorChannel::Blue,
        ] {
            let (hsl, hsb) = (
                c.channel_in(ch, ColorSpace::Hsl),
                c.channel_in(ch, ColorSpace::Hsb),
            );
            assert!(
                (hsl - hsb).abs() < 1e-6,
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
        for hex in [
            "#FF0000", "#00FF00", "#0000FF", "#123456", "#FFFFFF", "#000000",
        ] {
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
    #[allow(clippy::float_cmp)] // the clamp bounds are exact
    fn channel_values_stay_in_range() {
        let c = PickerColor::default();
        assert!((c.with_channel(ColorChannel::Alpha, 5.0).alpha - 1.0).abs() < 1e-6);
        assert_eq!(c.with_channel(ColorChannel::Alpha, -1.0).alpha, 0.0);
        assert!(
            c.with_channel(ColorChannel::Red, 999.0)
                .channel(ColorChannel::Red)
                <= 255.0
        );
    }

    #[test]
    fn rgb_channel_edit_matches_readback() {
        let c = PickerColor::from_hex("#204060").unwrap();
        let next = c.with_channel(ColorChannel::Green, 128.0);
        assert!((next.channel(ColorChannel::Green) - 128.0).abs() < 1.0);
    }

    #[test]
    fn dark_saturated_blue_uses_a_white_selected_indicator() {
        let blue = PickerColor::from_hex("#0000FF").unwrap();
        assert_eq!(color_swatch_indicator_color(blue), gpui::white());
    }

    #[test]
    fn mid_gray_uses_a_black_selected_indicator() {
        let gray = PickerColor::from_hex("#808080").unwrap();
        assert_eq!(color_swatch_indicator_color(gray), gpui::black());
    }
}
