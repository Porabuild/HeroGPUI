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

    fn with_hsl_lightness(self, l: f32) -> Self {
        self.with_hsl_channels(self.hsl_saturation(), l.clamp(0.0, 1.0))
    }

    fn with_hsl_channels(self, s: f32, l: f32) -> Self {
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

    fn hsl_lightness(self) -> f32 {
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
fn normalize_hue(hue: f32) -> f32 {
    if hue == 360.0 {
        hue
    } else {
        hue.rem_euclid(360.0)
    }
}

type OnColorChange = Arc<dyn Fn(PickerColor, &mut Window, &mut App) + 'static>;

fn color_swatch_indicator_color(swatch: PickerColor) -> Hsla {
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
type OnColorFieldChange = Arc<dyn Fn(Option<PickerColor>, &mut Window, &mut App) + 'static>;

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
    /// `ColorSwatchPicker.Item.isDisabled` — the item's own flag, drawn on
    /// the swatch it wraps.
    is_disabled: bool,
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
            // `.color-swatch` is `size-8` (32px), which is `SizeXl::Md` on v3's
            // own swatch scale (16/24/32/36/40).
            size: SizeXl::Md,
            shape: SwatchShape::Circle,
            is_disabled: false,
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

    /// `ColorSwatchPicker.Item.isDisabled` — the swatch an item wraps when it
    /// cannot be chosen.
    ///
    /// The picker draws each item around its swatch and the item's disabled
    /// state is the part's prop; a standalone preview honours the same flag
    /// by dimming — the reduced-opacity look the picker's sheet gives a
    /// disabled item.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }
}

impl RenderOnce for ColorSwatch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        let edge = self.size.swatch_px();
        // `.color-swatch--circle` names a radius per size -- `rounded-lg` at 16px
        // through `rounded-3xl` at 40 -- and every one of them is at least half
        // the edge, so the shape is a circle at every size. `--square` is
        // `rounded-md` throughout.
        let radius = match self.shape {
            SwatchShape::Circle => px(f32::from(edge) / 2.),
            SwatchShape::Square => cx.layout().radius_md(),
        };

        div()
            .size(edge)
            .rounded(radius)
            .flex_shrink_0()
            .overflow_hidden()
            .border(layout.border_width)
            .border_color(colors.border)
            // Checkerboard under the color reveals translucency.
            .bg(colors.surface_secondary)
            .when(self.is_disabled, |el| el.opacity(layout.disabled_opacity))
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

/// State handed to `ColorArea.Thumb`'s render function.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorAreaThumbState {
    pub color: PickerColor,
    pub is_dragging: bool,
    pub is_hovered: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
}

const COLOR_AREA_THUMB_IDLE_PX: f32 = 16.0;
const COLOR_AREA_THUMB_DRAGGING_PX: f32 = 20.0;
const COLOR_AREA_THUMB_TRANSITION_MS: u64 = 150;

#[derive(Clone)]
struct ColorAreaThumbMotion {
    dragging: bool,
    generation: usize,
    from: f32,
    size: Rc<Cell<f32>>,
}

struct ColorAreaThumbMotionFrame {
    generation: usize,
    from: f32,
    to: f32,
    size: Rc<Cell<f32>>,
    animate: bool,
}

impl ColorAreaThumbMotionFrame {
    fn render(self, thumb: gpui::Div) -> gpui::AnyElement {
        if !self.animate {
            self.size.set(self.to);
            return place_color_area_thumb(thumb, self.to).into_any_element();
        }

        let size = self.size;
        let from = self.from;
        let to = self.to;
        thumb
            .with_animation(
                ElementId::Name(format!("color-area-thumb-size-{}", self.generation).into()),
                Animation::new(Duration::from_millis(COLOR_AREA_THUMB_TRANSITION_MS))
                    .with_easing(|t| crate::anim::Curve::Out.at(t)),
                move |thumb, delta| {
                    let next = from + (to - from) * delta;
                    size.set(next);
                    place_color_area_thumb(thumb, next)
                },
            )
            .into_any_element()
    }
}

fn place_color_area_thumb(thumb: gpui::Div, size: f32) -> gpui::Div {
    let inset = (COLOR_AREA_THUMB_DRAGGING_PX - size) / 2.0;
    thumb.size(px(size)).ml(px(inset)).mt(px(inset))
}

fn color_area_thumb_motion(
    id: &ElementId,
    dragging: bool,
    window: &mut Window,
    cx: &mut App,
) -> ColorAreaThumbMotionFrame {
    let state = window.use_keyed_state(
        ElementId::Name(format!("{id:?}-thumb-motion").into()),
        cx,
        |_, _| ColorAreaThumbMotion {
            dragging,
            generation: 0,
            from: if dragging {
                COLOR_AREA_THUMB_DRAGGING_PX
            } else {
                COLOR_AREA_THUMB_IDLE_PX
            },
            size: Rc::new(Cell::new(if dragging {
                COLOR_AREA_THUMB_DRAGGING_PX
            } else {
                COLOR_AREA_THUMB_IDLE_PX
            })),
        },
    );
    let mut current = state.read(cx).clone();
    let to = if dragging {
        COLOR_AREA_THUMB_DRAGGING_PX
    } else {
        COLOR_AREA_THUMB_IDLE_PX
    };
    if current.dragging != dragging {
        current.dragging = dragging;
        current.generation = current.generation.wrapping_add(1);
        current.from = current.size.get();
        state.update(cx, |stored, _| *stored = current.clone());
    }
    if ActiveTheme::reduce_motion(cx) && (current.size.get() - to).abs() > f32::EPSILON {
        current.from = to;
        current.size.set(to);
        state.update(cx, |stored, _| *stored = current.clone());
    }
    ColorAreaThumbMotionFrame {
        generation: current.generation,
        from: current.from,
        to,
        size: current.size,
        animate: current.generation != 0
            && !ActiveTheme::reduce_motion(cx)
            && (current.from - to).abs() > f32::EPSILON,
    }
}

/// ColorArea — a two-dimensional gradient for picking two channels at once.
#[derive(IntoElement)]
pub struct ColorArea {
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
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
    thumb: Option<Arc<dyn Fn(ColorAreaThumbState) -> gpui::AnyElement + 'static>>,
    on_change: Option<OnColorChange>,
    on_change_end: Option<OnColorChange>,
}

impl ColorArea {
    pub fn new(id: impl Into<ElementId>, value: PickerColor) -> Self {
        Self {
            default_value: None,
            id: id.into(),
            value,
            color_space: None,
            x_channel: ColorChannel::Saturation,
            y_channel: ColorChannel::Brightness,
            width: px(224.),
            height: px(224.),
            is_disabled: false,
            show_dots: false,
            thumb: None,
            on_change: None,
            on_change_end: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
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

    /// `ColorArea.Thumb`'s render function — the closure receives the live
    /// color and interaction state while the built-in thumb keeps ownership
    /// of positioning, focus and pointer behavior.
    pub fn thumb(
        mut self,
        render: impl Fn(ColorAreaThumbState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.thumb = Some(Arc::new(render));
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own colour;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-area-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );
        self.value = resolved;
        // `.color-area__thumb[data-focus-visible]` is `status-focused`.
        // `use_keyed_state` takes `cx` mutably, so the handle precedes the
        // theme.
        let area_focus = util::tab_stop_handle(
            ElementId::Name(format!("{:?}-area-focus", self.id).into()),
            window,
            cx,
        );
        let bounds_slot = window.use_keyed_state(
            ElementId::Name(format!("{:?}-area-bounds", self.id).into()),
            cx,
            |_, _| Bounds::<f32> {
                origin: gpui::point(0., 0.),
                size: gpui::size(0., 0.),
            },
        );
        let dragging = window.use_keyed_state(
            ElementId::Name(format!("{:?}-area-dragging", self.id).into()),
            cx,
            |_, _| false,
        );
        let thumb_hovered = window.use_keyed_state(
            ElementId::Name(format!("{:?}-area-thumb-hovered", self.id).into()),
            cx,
            |_, _| false,
        );
        let colors = cx.colors();
        let border_width = f32::from(cx.layout().border_width);
        // `.color-area` is `rounded-2xl`, which is `soft_radius`.
        let radius = util::soft_radius(cx);
        let hue_color = PickerColor::hsb(self.value.hue, 1.0, 1.0).to_hsla();

        let (x_min, x_max) = self.x_channel.range();
        let (y_min, y_max) = self.y_channel.range();
        let color_space = self.color_space.unwrap_or_default();
        let x_norm = ((self.value.channel_in(self.x_channel, color_space) - x_min)
            / (x_max - x_min))
            .clamp(0.0, 1.0);
        let y_norm = ((self.value.channel_in(self.y_channel, color_space) - y_min)
            / (y_max - y_min))
            .clamp(0.0, 1.0);

        // Saturation left-to-right over the hue, brightness/lightness
        // bottom-to-top. RGB uses sampled vertical ramps because gpui has no
        // screen blend mode, which React Aria uses to combine its axes.
        let mut area = div()
            .id(self.id.clone())
            .when(!self.is_disabled, |el| el.track_focus(&area_focus))
            .relative()
            .w(self.width)
            .h(self.height)
            .rounded(radius)
            .border(cx.layout().border_width)
            .border_color(colors.border);

        // `.color-area` is `overflow: visible` -- the thumb is meant to hang
        // over the edge, and upstream can allow that because its gradient stack
        // is the element's own `background` plus an `::after`, both of which
        // take the radius. Here the stack is real children, so the clip that
        // holds them inside the corner goes on this one inner layer rather than
        // on the area: clipping the area cut the thumb in half at every edge.
        let mut layers = div().absolute().inset_0().rounded(radius).overflow_hidden();

        layers = if self.x_channel == ColorChannel::Hue || self.y_channel == ColorChannel::Hue {
            layers.child(color_area_hue_layers(
                self.value,
                color_space,
                self.x_channel,
                self.y_channel,
            ))
        } else {
            match (color_space, self.x_channel, self.y_channel) {
                (ColorSpace::Hsb, ColorChannel::Saturation, ColorChannel::Brightness) => layers
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(gpui::white(), 0.0),
                        gpui::linear_color_stop(hue_color, 1.0),
                    ))
                    .child(div().absolute().inset_0().bg(gpui::linear_gradient(
                        180.0,
                        gpui::linear_color_stop(gpui::transparent_black(), 0.0),
                        gpui::linear_color_stop(gpui::black(), 1.0),
                    ))),
                (ColorSpace::Hsl, ColorChannel::Saturation, ColorChannel::Lightness) => {
                    let gray = self.value.with_hsl_channels(0.0, 0.5).to_hsla();
                    let hue = self.value.with_hsl_channels(1.0, 0.5).to_hsla();
                    layers
                        .bg(gpui::linear_gradient(
                            90.0,
                            gpui::linear_color_stop(gray, 0.0),
                            gpui::linear_color_stop(hue, 1.0),
                        ))
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .right_0()
                                .h(px(f32::from(self.height) / 2.0))
                                .bg(gpui::linear_gradient(
                                    180.0,
                                    gpui::linear_color_stop(gpui::white(), 0.0),
                                    gpui::linear_color_stop(gpui::transparent_white(), 1.0),
                                )),
                        )
                        .child(
                            div()
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .right_0()
                                .h(px(f32::from(self.height) / 2.0))
                                .bg(gpui::linear_gradient(
                                    180.0,
                                    gpui::linear_color_stop(gpui::transparent_black(), 0.0),
                                    gpui::linear_color_stop(gpui::black(), 1.0),
                                )),
                        )
                }
                _ => layers.child(color_area_channel_grid(
                    self.value,
                    color_space,
                    self.x_channel,
                    self.y_channel,
                )),
            }
        };

        let recorder_bounds = bounds_slot.clone();
        area = area.child(
            gpui::canvas(
                move |bounds: Bounds<Pixels>, _, cx| {
                    recorder_bounds.update(cx, |slot, _| {
                        *slot = Bounds {
                            origin: gpui::point(
                                f32::from(bounds.origin.x) - border_width,
                                f32::from(bounds.origin.y) - border_width,
                            ),
                            size: gpui::size(
                                f32::from(bounds.size.width) + border_width * 2.0,
                                f32::from(bounds.size.height) + border_width * 2.0,
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

        // `showDots` — the dot-grid overlay. gpui has no repeating background,
        // so the grid is drawn as rows of small translucent dots.
        if self.show_dots {
            const STEP: f32 = 8.0;
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
            layers = layers.child(grid);
        }

        area = area.child(layers);

        let is_dragging = !self.is_disabled && *dragging.read(cx);
        let is_focused = !self.is_disabled && area_focus.is_focused(window);
        let is_focus_visible = is_focused && util::focus_visible(cx);
        let thumb_state = ColorAreaThumbState {
            color: self.value,
            is_dragging,
            is_hovered: !self.is_disabled && *thumb_hovered.read(cx),
            is_focused,
            is_focus_visible,
            is_disabled: self.is_disabled,
        };
        let thumb_content = self.thumb.as_ref().map(|render| render(thumb_state));
        let thumb_motion = color_area_thumb_motion(&self.id, is_dragging, window, cx);
        let thumb_visual = util::with_focus_ring(
            div()
                // `.color-area__thumb` is `rounded-xl`, which is circular at
                // both the idle and dragging sizes.
                .rounded(px(12.))
                // `.color-area__thumb` is `border: 3px solid white`.
                .border(px(3.))
                .border_color(gpui::white())
                .bg(self.value.to_hsla()),
            is_focus_visible,
            true,
            Vec::new(),
            cx,
        );
        // The stable 20px wrapper owns hover. The changing animation id lives
        // on its listener-free visual child, so a drag transition cannot drop
        // the interaction path or a caller-provided child.
        let mut thumb = div()
            .id(ElementId::Name(format!("{:?}-area-thumb", self.id).into()))
            .absolute()
            .left(px(
                f32::from(self.width) * x_norm - COLOR_AREA_THUMB_DRAGGING_PX / 2.0
            ))
            .top(px(
                f32::from(self.height) * (1.0 - y_norm) - COLOR_AREA_THUMB_DRAGGING_PX / 2.0
            ))
            .size(px(COLOR_AREA_THUMB_DRAGGING_PX))
            .child(thumb_motion.render(thumb_visual))
            .when_some(thumb_content, |thumb, content| thumb.child(content));
        if !self.is_disabled {
            let hovered = thumb_hovered;
            thumb = thumb.on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |value, cx| {
                    if *value != *is_hovered {
                        *value = *is_hovered;
                        cx.notify();
                    }
                });
            });
        }
        area = area.child(thumb);

        if self.is_disabled {
            return area.opacity(cx.layout().disabled_opacity);
        }

        // v3's ColorArea inherits React Aria's keyboard: the arrows move the
        // thumb, left/right on the x channel and up/down on the y, and Page
        // Up/Down move the y while Home/End move the x by the page step
        // (React Aria's `useColorArea` shortcuts step by the page, not to the
        // edge). A two-axis control that answered no key could only be moved
        // with the pointer; the step sizes are ColorSlider's own, so the two
        // colour controls agree.
        let keys_value = self.value;
        let on_change_keys = self.on_change.clone();
        let end_keys = self.on_change_end.clone();
        let own_keys = own.clone();
        let (x_channel, y_channel) = (self.x_channel, self.y_channel);
        let key_space = color_space;
        let (x_min, x_max) = x_channel.range();
        let (y_min, y_max) = y_channel.range();
        // One step per unit for a wide channel (hue, an 8-bit byte), a
        // percentage point for the normalised ones -- the same rule
        // ColorSlider's keys use.
        let x_step = if x_max - x_min > 2.0 { 1.0 } else { 0.01 };
        let y_step = if y_max - y_min > 2.0 { 1.0 } else { 0.01 };
        // A tenth of the range, never less than a step -- React Aria's page.
        let x_page = ((x_max - x_min) / 10.0).max(x_step);
        let y_page = ((y_max - y_min) / 10.0).max(y_step);
        area = area
            .key_context("ColorArea")
            .on_key_down(move |event, window, cx| {
                let x_now = keys_value.channel_in(x_channel, key_space);
                let y_now = keys_value.channel_in(y_channel, key_space);
                let (nx, ny) = match event.keystroke.key.as_str() {
                    "left" => (x_now - x_step, y_now),
                    "right" => (x_now + x_step, y_now),
                    "down" => (x_now, y_now - y_step),
                    "up" => (x_now, y_now + y_step),
                    "pageup" => (x_now, y_now + y_page),
                    "pagedown" => (x_now, y_now - y_page),
                    "home" => (x_now - x_page, y_now),
                    "end" => (x_now + x_page, y_now),
                    _ => return,
                };
                let next = keys_value
                    .with_channel_in(x_channel, key_space, nx.clamp(x_min, x_max))
                    .with_channel_in(y_channel, key_space, ny.clamp(y_min, y_max));
                if next == keys_value {
                    return;
                }
                // Uncontrolled: move our own copy, as the pointer path does.
                if let Some(held) = &own_keys {
                    held.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_change_keys {
                    cb(next, window, cx);
                }
                // A keystroke is a finished change, so `onChangeEnd` fires
                // with it rather than waiting for a release that never comes.
                if let Some(cb) = &end_keys {
                    cb(next, window, cx);
                }
            });

        if self.on_change.is_some() || self.on_change_end.is_some() || own.is_some() {
            let down_bounds = bounds_slot.clone();
            let down_dragging = dragging.clone();
            let down_focus = area_focus;
            let down_change = self.on_change.clone();
            let down_own = own.clone();
            let down_value = self.value;
            let (x_channel, y_channel) = (self.x_channel, self.y_channel);
            area = area.cursor_pointer().on_mouse_down(
                gpui::MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    if event.modifiers.alt || event.modifiers.control || event.modifiers.platform {
                        return;
                    }
                    down_dragging.update(cx, |value, cx| {
                        if !*value {
                            *value = true;
                            cx.notify();
                        }
                    });
                    window.focus(&down_focus, cx);
                    if let Some(next) = area_color_from_pointer(
                        &down_bounds,
                        event.position,
                        down_value,
                        color_space,
                        x_channel,
                        y_channel,
                        cx,
                    ) {
                        if next.changed {
                            report_color_change(next.value, &down_own, &down_change, window, cx);
                        }
                    }
                },
            );

            let global_bounds = bounds_slot;
            let global_dragging = dragging;
            let global_change = self.on_change;
            let global_end = self.on_change_end;
            let global_own = own;
            let global_value = self.value;
            area = area.child(
                gpui::canvas(
                    |bounds, _, _| bounds,
                    move |_, _, window, _| {
                        let move_bounds = global_bounds.clone();
                        let move_dragging = global_dragging.clone();
                        let move_change = global_change.clone();
                        let move_own = global_own.clone();
                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                            if phase == gpui::DispatchPhase::Capture
                                && event.pressed_button == Some(gpui::MouseButton::Left)
                                && *move_dragging.read(cx)
                            {
                                if let Some(next) = area_color_from_pointer(
                                    &move_bounds,
                                    event.position,
                                    global_value,
                                    color_space,
                                    x_channel,
                                    y_channel,
                                    cx,
                                ) {
                                    if next.changed {
                                        report_color_change(
                                            next.value,
                                            &move_own,
                                            &move_change,
                                            window,
                                            cx,
                                        );
                                    }
                                }
                            }
                        });

                        let up_bounds = global_bounds.clone();
                        let up_dragging = global_dragging.clone();
                        let up_end = global_end.clone();
                        window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                            if phase == gpui::DispatchPhase::Capture
                                && event.button == gpui::MouseButton::Left
                            {
                                finish_area_drag(
                                    &up_dragging,
                                    &up_bounds,
                                    event.position,
                                    global_value,
                                    color_space,
                                    x_channel,
                                    y_channel,
                                    &up_end,
                                    window,
                                    cx,
                                );
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            );
        }

        area
    }
}

#[allow(clippy::too_many_arguments)] // the two axes and callback are the color-area drag state
fn finish_area_drag(
    dragging: &Entity<bool>,
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    value: PickerColor,
    color_space: ColorSpace,
    x_channel: ColorChannel,
    y_channel: ColorChannel,
    on_change_end: &Option<OnColorChange>,
    window: &mut Window,
    cx: &mut App,
) {
    if !*dragging.read(cx) {
        return;
    }
    dragging.update(cx, |value, cx| {
        *value = false;
        cx.notify();
    });
    if let (Some(callback), Some(next)) = (
        on_change_end,
        area_color_from_pointer(
            bounds,
            position,
            value,
            color_space,
            x_channel,
            y_channel,
            cx,
        ),
    ) {
        callback(next.value, window, cx);
    }
}

fn color_area_hue_layers(
    value: PickerColor,
    color_space: ColorSpace,
    x_channel: ColorChannel,
    y_channel: ColorChannel,
) -> gpui::Div {
    let hue_is_vertical = y_channel == ColorChannel::Hue;
    let other = if x_channel == ColorChannel::Hue {
        y_channel
    } else {
        x_channel
    };
    let other_is_vertical = y_channel == other;
    let base = match (color_space, other) {
        (ColorSpace::Hsl, ColorChannel::Saturation) => value
            .with_hsl_channels(1.0, value.hsl_lightness())
            .with_alpha(1.0),
        (ColorSpace::Hsl, ColorChannel::Lightness) => value
            .with_hsl_channels(value.hsl_saturation(), 0.5)
            .with_alpha(1.0),
        (ColorSpace::Hsb, ColorChannel::Saturation) => PickerColor::hsb(0.0, 1.0, value.brightness),
        (ColorSpace::Hsb, ColorChannel::Brightness) => PickerColor::hsb(0.0, value.saturation, 1.0),
        _ => PickerColor::hsb(0.0, 1.0, 1.0),
    };

    let mut layers =
        div()
            .absolute()
            .inset_0()
            .child(hue_gradient(base, color_space, hue_is_vertical));
    layers = match other {
        ColorChannel::Saturation => {
            let start = base
                .with_channel_in(other, color_space, other.range().0)
                .to_hsla();
            layers.child(div().absolute().inset_0().bg(gpui::linear_gradient(
                if other_is_vertical { 0.0 } else { 90.0 },
                gpui::linear_color_stop(start, 0.0),
                gpui::linear_color_stop(start.alpha(0.0), 1.0),
            )))
        }
        ColorChannel::Brightness => {
            layers.child(div().absolute().inset_0().bg(gpui::linear_gradient(
                if other_is_vertical { 0.0 } else { 90.0 },
                gpui::linear_color_stop(gpui::black(), 0.0),
                gpui::linear_color_stop(gpui::transparent_black(), 1.0),
            )))
        }
        ColorChannel::Lightness => layers.child(three_stop_gradient(
            other_is_vertical,
            gpui::black(),
            gpui::transparent_black(),
            gpui::white(),
        )),
        _ => layers,
    };
    layers
}

fn hue_gradient(value: PickerColor, color_space: ColorSpace, vertical: bool) -> gpui::Div {
    let stops = hue_stop_colors(value, color_space);
    let mut gradient = div().absolute().inset_0();
    for index in 0..6 {
        gradient = if vertical {
            gradient.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(gpui::relative(hue_band_offset(index, true)))
                    .h(gpui::relative(1.0 / 6.0))
                    .bg(gpui::linear_gradient(
                        0.0,
                        gpui::linear_color_stop(stops[index], 0.0),
                        gpui::linear_color_stop(stops[index + 1], 1.0),
                    )),
            )
        } else {
            gradient.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(gpui::relative(hue_band_offset(index, false)))
                    .w(gpui::relative(1.0 / 6.0))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(stops[index], 0.0),
                        gpui::linear_color_stop(stops[index + 1], 1.0),
                    )),
            )
        };
    }
    gradient
}

fn hue_band_offset(index: usize, vertical: bool) -> f32 {
    if vertical {
        (5 - index) as f32 / 6.0
    } else {
        index as f32 / 6.0
    }
}

fn hue_stop_colors(value: PickerColor, color_space: ColorSpace) -> [Hsla; 7] {
    [0.0, 60.0, 120.0, 180.0, 240.0, 300.0, 360.0].map(|hue| {
        value
            .with_channel_in(ColorChannel::Hue, color_space, hue)
            .to_hsla()
    })
}

fn color_area_channel_grid(
    value: PickerColor,
    color_space: ColorSpace,
    x_channel: ColorChannel,
    y_channel: ColorChannel,
) -> gpui::Div {
    const STRIPS: usize = 64;
    let (x_min, x_max) = x_channel.range();
    let (y_min, y_max) = y_channel.range();
    let mut grid = div().absolute().inset_0().flex().flex_row();
    for index in 0..STRIPS {
        let x = index as f32 / (STRIPS - 1) as f32;
        let at_x = value.with_channel_in(x_channel, color_space, x_min + x * (x_max - x_min));
        let bottom = at_x
            .with_channel_in(y_channel, color_space, y_min)
            .to_hsla();
        let top = at_x
            .with_channel_in(y_channel, color_space, y_max)
            .to_hsla();
        grid = grid.child(div().h_full().flex_1().bg(gpui::linear_gradient(
            0.0,
            gpui::linear_color_stop(bottom, 0.0),
            gpui::linear_color_stop(top, 1.0),
        )));
    }
    grid
}

struct PointerColor {
    value: PickerColor,
    changed: bool,
}

#[allow(clippy::float_cmp)] // snapped channel values are exact state coordinates
fn area_color_from_pointer(
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    value: PickerColor,
    color_space: ColorSpace,
    x_channel: ColorChannel,
    y_channel: ColorChannel,
    cx: &App,
) -> Option<PointerColor> {
    let bounds = *bounds.read(cx);
    if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
        return None;
    }
    let x = ((f32::from(position.x) - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0);
    let y = ((f32::from(position.y) - bounds.origin.y) / bounds.size.height).clamp(0.0, 1.0);
    let (x_min, x_max) = x_channel.range();
    let (y_min, y_max) = y_channel.range();
    let x_value = snap_color_channel(x_channel, x_min + x * (x_max - x_min));
    let y_value = snap_color_channel(y_channel, y_min + (1.0 - y) * (y_max - y_min));
    Some(PointerColor {
        changed: x_value != value.channel_in(x_channel, color_space)
            || y_value != value.channel_in(y_channel, color_space),
        value: value
            .with_channel_in(x_channel, color_space, x_value)
            .with_channel_in(y_channel, color_space, y_value),
    })
}

fn report_color_change(
    next: PickerColor,
    own: &Option<Entity<PickerColor>>,
    on_change: &Option<OnColorChange>,
    window: &mut Window,
    cx: &mut App,
) {
    if let Some(held) = own {
        held.update(cx, |value, cx| {
            *value = next;
            cx.notify();
        });
    }
    if let Some(callback) = on_change {
        callback(next, window, cx);
    }
}

fn live_color_form_state(
    value: crate::form::FormValue,
) -> Rc<RefCell<crate::form::LiveFormFieldState>> {
    Rc::new(RefCell::new(crate::form::LiveFormFieldState {
        value,
        is_invalid: false,
        is_successful: true,
        focus: None,
        restore: None,
    }))
}

fn sync_color_form_state(
    state: &Rc<RefCell<crate::form::LiveFormFieldState>>,
    value: crate::form::FormValue,
    is_successful: bool,
    is_invalid: bool,
) {
    let mut state = state.borrow_mut();
    state.value = value;
    state.is_successful = is_successful;
    state.is_invalid = is_invalid;
}

/// React Aria ColorSlider submits the channel number of a hidden range input.
fn color_slider_form_value(
    value: PickerColor,
    channel: ColorChannel,
    space: ColorSpace,
) -> crate::form::FormValue {
    let value = value.channel_in(channel, space);
    let value = if matches!(
        channel,
        ColorChannel::Saturation | ColorChannel::Brightness | ColorChannel::Lightness
    ) {
        value * 100.0
    } else {
        value
    };
    crate::form::FormValue::Number(f64::from(value))
}

/// React Aria ColorField submits the hex text, or the channel number when
/// `channel` is set (ColorChannelField is a NumberField).
fn color_field_form_value(
    value: PickerColor,
    channel: Option<ColorChannel>,
    space: ColorSpace,
) -> crate::form::FormValue {
    match channel {
        None => crate::form::FormValue::Text(value.to_hex().into()),
        Some(channel) => {
            crate::form::FormValue::Number(f64::from(value.channel_in(channel, space)))
        }
    }
}

fn color_field_display_text(
    value: PickerColor,
    channel: Option<ColorChannel>,
    space: ColorSpace,
) -> String {
    match channel {
        None => value.to_hex(),
        Some(channel) => format_color_channel_value(value, channel, space),
    }
}

// ---------------------------------------------------------------------------
// ColorSlider
// ---------------------------------------------------------------------------

/// State handed to `ColorSlider.Thumb`'s render function.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorSliderThumbState {
    /// React Aria's ColorThumb render color excludes the alpha channel.
    pub color: PickerColor,
    pub is_dragging: bool,
    pub is_hovered: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
}

const COLOR_SLIDER_TRACK_INSET_PX: f32 = 10.0;

/// ColorSlider — adjusts a single channel along a gradient track.
#[derive(IntoElement)]
pub struct ColorSlider {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
    id: ElementId,
    value: PickerColor,
    channel: ColorChannel,
    /// `colorSpace` — only saturation differs between HSB and HSL, so this
    /// picks which one a saturation slider edits.
    color_space: ColorSpace,
    orientation: herogpui_core::Orientation,
    length: Pixels,
    show_label: bool,
    /// `ColorSlider.Output`'s render props: the closure is handed the current
    /// `color` and the formatted channel value.
    output: Option<Arc<dyn Fn(PickerColor, &str) -> gpui::AnyElement + 'static>>,
    thumb: Option<Arc<dyn Fn(ColorSliderThumbState) -> gpui::AnyElement + 'static>>,
    is_disabled: bool,
    on_change: Option<OnColorChange>,
    on_change_end: Option<OnColorChange>,
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
}

impl ColorSlider {
    pub fn new(id: impl Into<ElementId>, value: PickerColor, channel: ColorChannel) -> Self {
        Self {
            name: None,
            default_value: None,
            id: id.into(),
            value,
            channel,
            color_space: ColorSpace::default(),
            orientation: herogpui_core::Orientation::Horizontal,
            length: px(240.),
            show_label: true,
            output: None,
            thumb: None,
            is_disabled: false,
            on_change: None,
            on_change_end: None,
            form_state: live_color_form_state(crate::form::FormValue::Number(0.0)),
        }
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
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
        let value = self.default_value.unwrap_or(self.value);
        // A disabled range input is not a successful control in HTML, so the
        // field stays registered and is omitted from FormData.
        sync_color_form_state(
            &self.form_state,
            color_slider_form_value(value, self.channel, self.color_space),
            !self.is_disabled,
            false,
        );
        Some(crate::form::FormField::live(name, self.form_state.clone()).is_required(false))
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
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

    /// `ColorSlider.Output`'s render function — v3 hands it the `color`, which
    /// is what this closure takes along with the value as v3 formats it.
    pub fn output(
        mut self,
        render: impl Fn(PickerColor, &str) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.output = Some(Arc::new(render));
        self
    }

    /// `ColorSlider.Thumb`'s render function — the closure receives the shared
    /// ColorThumb interaction state while the built-in thumb retains its
    /// positioning, focus and drag behavior.
    pub fn thumb(
        mut self,
        render: impl Fn(ColorSliderThumbState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.thumb = Some(Arc::new(render));
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
            self.value
                .with_channel_in(self.channel, self.color_space, min)
                .to_hsla(),
            self.value
                .with_channel_in(self.channel, self.color_space, max)
                .to_hsla(),
        )
    }
}

fn three_stop_gradient(vertical: bool, start: Hsla, middle: Hsla, end: Hsla) -> gpui::Div {
    if vertical {
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        0.0,
                        gpui::linear_color_stop(middle, 0.0),
                        gpui::linear_color_stop(end, 1.0),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        0.0,
                        gpui::linear_color_stop(start, 0.0),
                        gpui::linear_color_stop(middle, 1.0),
                    )),
            )
    } else {
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left_0()
                    .w(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(start, 0.0),
                        gpui::linear_color_stop(middle, 1.0),
                    )),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .w(gpui::relative(0.5))
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(middle, 0.0),
                        gpui::linear_color_stop(end, 1.0),
                    )),
            )
    }
}

impl RenderOnce for ColorSlider {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own colour;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-slider-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );
        self.value = resolved;
        let form_default = window.use_keyed_state(
            ElementId::Name(format!("{:?}-slider-form-default", self.id).into()),
            cx,
            |_, _| None::<PickerColor>,
        );
        if form_default.read(cx).is_none() {
            let initial = self.value;
            form_default.update(cx, |slot, cx| {
                *slot = Some(initial);
                cx.notify();
            });
        }
        let restore_default = form_default.read(cx).unwrap_or(self.value);
        // Submit the resolved colour's channel. Uncontrolled keyed state is
        // current after interaction; a controlled owner must accept the change
        // before the next render writes it here.
        sync_color_form_state(
            &self.form_state,
            color_slider_form_value(self.value, self.channel, self.color_space),
            !self.is_disabled,
            false,
        );
        let restore_own = own.clone();
        let restore_on_change = self.on_change.clone();
        let restore_form_state = self.form_state.clone();
        let restore_channel = self.channel;
        let restore_space = self.color_space;
        let restore_is_disabled = self.is_disabled;
        let restore: Arc<dyn Fn(&mut Window, &mut App)> = util::shared(move |window, cx| {
            if let Some(own) = &restore_own {
                own.update(cx, |current, cx| {
                    *current = restore_default;
                    cx.notify();
                });
            }
            if let Some(callback) = &restore_on_change {
                callback(restore_default, window, cx);
            }
            sync_color_form_state(
                &restore_form_state,
                color_slider_form_value(restore_default, restore_channel, restore_space),
                !restore_is_disabled,
                false,
            );
        });
        self.form_state.borrow_mut().restore = Some(restore);
        // The handle the keys arrive on. `use_keyed_state` takes `cx` mutably, so
        // it precedes the theme tokens.
        let focus_handle = window.use_keyed_state(
            ElementId::Name(format!("{:?}-slider-focus", self.id).into()),
            cx,
            |_, cx| cx.focus_handle().tab_stop(true),
        );
        let focus_handle = focus_handle.read(cx).clone();
        self.form_state.borrow_mut().focus = Some(focus_handle.clone());
        let bounds_slot = window.use_keyed_state(
            ElementId::Name(format!("{:?}-slider-bounds", self.id).into()),
            cx,
            |_, _| Bounds::<f32> {
                origin: gpui::point(0., 0.),
                size: gpui::size(0., 0.),
            },
        );
        let dragging = window.use_keyed_state(
            ElementId::Name(format!("{:?}-slider-dragging", self.id).into()),
            cx,
            |_, _| false,
        );
        let thumb_hovered = window.use_keyed_state(
            ElementId::Name(format!("{:?}-slider-thumb-hovered", self.id).into()),
            cx,
            |_, _| false,
        );
        let colors = cx.colors();
        let border_width = f32::from(cx.layout().border_width);
        let (min, max) = self.channel.range();
        // Read in the requested space: HSL and HSB saturation are different
        // numbers for the same colour.
        let raw = self.value.channel_in(self.channel, self.color_space);
        let norm = ((raw - min) / (max - min)).clamp(0.0, 1.0);
        // `.color-slider__track` is `relative rounded-2xl` with the gradient
        // inside it; `.color-slider__output` is the value read-out above.
        let track_h = px(20.);

        let vertical = !self.orientation.is_horizontal();
        let mut track = div()
            .id(self.id.clone())
            .relative()
            .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
            .border(cx.layout().border_width)
            .border_color(colors.border);
        track = if vertical {
            track.w(track_h).h(self.length)
        } else {
            track.w(self.length).h(track_h)
        };

        let recorder_bounds = bounds_slot.clone();
        track = track.child(
            gpui::canvas(
                move |bounds: Bounds<Pixels>, _, cx| {
                    recorder_bounds.update(cx, |slot, _| {
                        *slot = Bounds {
                            origin: gpui::point(
                                f32::from(bounds.origin.x) - border_width,
                                f32::from(bounds.origin.y) - border_width,
                            ),
                            size: gpui::size(
                                f32::from(bounds.size.width) + border_width * 2.0,
                                f32::from(bounds.size.height) + border_width * 2.0,
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

        // Hue needs the full spectrum, and lightness needs a midpoint so its
        // hue survives between black and white.
        track = if self.channel == ColorChannel::Lightness {
            let (start, middle, end) =
                lightness_gradient_colors(self.value, self.color_space, min, max);
            track.child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
                    .overflow_hidden()
                    .child(three_stop_gradient(vertical, start, middle, end)),
            )
        } else if self.channel == ColorChannel::Hue {
            let mut spectrum = div()
                .absolute()
                .inset_0()
                .rounded(px(COLOR_SLIDER_TRACK_INSET_PX))
                .overflow_hidden();
            // Six 60-degree bands approximate the continuous hue wheel.
            for i in 0..6 {
                let from = PickerColor::hsb(i as f32 * 60.0, 1.0, 1.0).to_hsla();
                let to = PickerColor::hsb((i as f32 + 1.0) * 60.0, 1.0, 1.0).to_hsla();
                spectrum = if vertical {
                    spectrum.child(
                        div()
                            .absolute()
                            .left_0()
                            .right_0()
                            .top(px(f32::from(self.length) * ((5 - i) as f32 / 6.0)))
                            .h(px(f32::from(self.length) / 6.0 + 1.0))
                            .bg(gpui::linear_gradient(
                                0.0,
                                gpui::linear_color_stop(from, 0.0),
                                gpui::linear_color_stop(to, 1.0),
                            )),
                    )
                } else {
                    spectrum.child(
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
                    )
                };
            }
            track.child(spectrum)
        } else {
            let (from, to) = self.gradient_ends();
            track.bg(gpui::linear_gradient(
                if vertical { 0.0 } else { 90.0 },
                gpui::linear_color_stop(from, 0.0),
                gpui::linear_color_stop(to, 1.0),
            ))
        };

        // A vertical slider's zero end is at the bottom, so the offset is
        // measured from the far edge.
        let travel = (f32::from(self.length) - COLOR_SLIDER_TRACK_INSET_PX * 2.0).max(0.0);
        let thumb_offset = px(COLOR_SLIDER_TRACK_INSET_PX
            + travel * if vertical { 1.0 - norm } else { norm }
            - 8.0);
        let is_dragging = !self.is_disabled && *dragging.read(cx);
        let is_focused = !self.is_disabled && focus_handle.is_focused(window);
        let is_focus_visible = is_focused && util::focus_visible(cx);
        let thumb_state = ColorSliderThumbState {
            color: self.value.with_alpha(1.0),
            is_dragging,
            is_hovered: !self.is_disabled && *thumb_hovered.read(cx),
            is_focused,
            is_focus_visible,
            is_disabled: self.is_disabled,
        };
        let thumb_content = self.thumb.as_ref().map(|render| render(thumb_state));
        let mut thumb = util::with_focus_ring(
            div()
                .id(ElementId::Name(
                    format!("{:?}-slider-thumb", self.id).into(),
                ))
                .absolute()
                .when(vertical, |t| t.left(px(2.)).top(thumb_offset))
                .when(!vertical, |t| t.top(px(2.)).left(thumb_offset))
                // `.color-slider__thumb` is `size-4`.
                .size(px(16.))
                .rounded(px(16.))
                .border(px(3.))
                .border_color(gpui::white())
                .bg(if self.is_disabled {
                    colors.default.color
                } else {
                    self.value.with_alpha(1.0).to_hsla()
                })
                .when_some(thumb_content, |thumb, content| thumb.child(content)),
            is_focus_visible,
            true,
            Vec::new(),
            cx,
        );
        if !self.is_disabled {
            let hovered = thumb_hovered;
            thumb = thumb.on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |value, cx| {
                    if *value != *is_hovered {
                        *value = *is_hovered;
                        cx.notify();
                    }
                });
            });
        }
        track = track.child(thumb);

        if self.is_disabled {
            track = track.opacity(cx.layout().disabled_opacity);
        } else {
            let value = self.value;
            let channel = self.channel;
            let space = self.color_space;
            track = track.cursor_pointer();
            // v3: the arrows step the channel, Home and End take it to its ends,
            // and Page Up/Down move by a tenth of the range -- React Aria's page
            // step. A colour slider with no keyboard is not the same control.
            let keys_value = self.value;
            let on_change_keys = self.on_change.clone();
            let end_keys = self.on_change_end.clone();
            let own_keys = own.clone();
            // One step per unit for a 0-360 hue or an 0-255 byte, and a
            // percentage point for the normalised channels.
            let step = if max - min > 2.0 { 1.0 } else { 0.01 };
            let page = ((max - min) / 10.0).max(step);
            track = track
                .track_focus(&focus_handle)
                .key_context("ColorSlider")
                .on_key_down(move |event, window, cx| {
                    let current = keys_value.channel_in(channel, space);
                    let next = match event.keystroke.key.as_str() {
                        "right" | "up" => current + step,
                        "left" | "down" => current - step,
                        "pageup" => current + page,
                        "pagedown" => current - page,
                        "home" => min,
                        "end" => max,
                        _ => return,
                    };
                    let next = keys_value.with_channel_in(channel, space, next.clamp(min, max));
                    if next == keys_value {
                        return;
                    }
                    if let Some(held) = &own_keys {
                        held.update(cx, |v, cx| {
                            *v = next;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change_keys {
                        cb(next, window, cx);
                    }
                    // A keystroke is a finished change, so `onChangeEnd` fires
                    // with it rather than waiting for a release.
                    if let Some(cb) = &end_keys {
                        cb(next, window, cx);
                    }
                });
            if self.on_change.is_some() || self.on_change_end.is_some() || own.is_some() {
                let down_bounds = bounds_slot.clone();
                let down_dragging = dragging.clone();
                let down_change = self.on_change.clone();
                let down_own = own.clone();
                let focus_for_press = focus_handle;
                track = track.on_mouse_down(
                    gpui::MouseButton::Left,
                    move |event: &MouseDownEvent, window, cx| {
                        if event.modifiers.alt
                            || event.modifiers.control
                            || event.modifiers.platform
                        {
                            return;
                        }
                        down_dragging.update(cx, |value, cx| {
                            if !*value {
                                *value = true;
                                cx.notify();
                            }
                        });
                        window.focus(&focus_for_press, cx);
                        if let Some(next) = slider_color_from_pointer(
                            &down_bounds,
                            event.position,
                            vertical,
                            value,
                            channel,
                            space,
                            cx,
                        ) {
                            if next.changed {
                                report_color_change(
                                    next.value,
                                    &down_own,
                                    &down_change,
                                    window,
                                    cx,
                                );
                            }
                        }
                    },
                );

                let global_bounds = bounds_slot;
                let global_dragging = dragging;
                let global_change = self.on_change;
                let global_end = self.on_change_end;
                let global_own = own;
                track = track.child(
                    gpui::canvas(
                        |bounds, _, _| bounds,
                        move |_, _, window, _| {
                            let move_bounds = global_bounds.clone();
                            let move_dragging = global_dragging.clone();
                            let move_change = global_change.clone();
                            let move_own = global_own.clone();
                            window.on_mouse_event(
                                move |event: &MouseMoveEvent, phase, window, cx| {
                                    if phase == gpui::DispatchPhase::Capture
                                        && event.pressed_button == Some(gpui::MouseButton::Left)
                                        && *move_dragging.read(cx)
                                    {
                                        if let Some(next) = slider_color_from_pointer(
                                            &move_bounds,
                                            event.position,
                                            vertical,
                                            value,
                                            channel,
                                            space,
                                            cx,
                                        ) {
                                            if next.changed {
                                                report_color_change(
                                                    next.value,
                                                    &move_own,
                                                    &move_change,
                                                    window,
                                                    cx,
                                                );
                                            }
                                        }
                                    }
                                },
                            );

                            let up_bounds = global_bounds.clone();
                            let up_dragging = global_dragging.clone();
                            let up_end = global_end.clone();
                            window.on_mouse_event(
                                move |event: &MouseUpEvent, phase, window, cx| {
                                    if phase == gpui::DispatchPhase::Capture
                                        && event.button == gpui::MouseButton::Left
                                    {
                                        finish_slider_drag(
                                            &up_dragging,
                                            &up_bounds,
                                            event.position,
                                            vertical,
                                            value,
                                            channel,
                                            space,
                                            &up_end,
                                            window,
                                            cx,
                                        );
                                    }
                                },
                            );
                        },
                    )
                    .absolute()
                    .inset_0(),
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
        let output = match &self.output {
            // `.color-slider__output`: v3's render prop is handed the colour,
            // which is what a caller needs to draw a swatch or a different
            // unit.
            Some(render) => render(self.value, &display),
            None => div()
                .text_color(colors.muted)
                .child(display)
                .into_any_element(),
        };

        div()
            .flex()
            .flex_col()
            // `.color-slider` is `grid w-full gap-1`.
            .gap(px(4.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w(self.length)
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(
                        div()
                            .text_color(colors.foreground)
                            .child(self.channel.label()),
                    )
                    // The disabled root dims Output through status-disabled,
                    // while the stylesheet restores Label to full opacity.
                    .child(
                        div()
                            .when(self.is_disabled, |output| {
                                output.opacity(cx.layout().disabled_opacity)
                            })
                            .child(output),
                    ),
            )
            .child(track)
    }
}

fn lightness_gradient_colors(
    value: PickerColor,
    color_space: ColorSpace,
    min: f32,
    max: f32,
) -> (Hsla, Hsla, Hsla) {
    (
        value
            .with_channel_in(ColorChannel::Lightness, color_space, min)
            .to_hsla(),
        value
            .with_channel_in(ColorChannel::Lightness, color_space, (max - min) / 2.0)
            .to_hsla(),
        value
            .with_channel_in(ColorChannel::Lightness, color_space, max)
            .to_hsla(),
    )
}

#[allow(clippy::float_cmp)] // snapped channel values are exact state coordinates
fn slider_color_from_pointer(
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    vertical: bool,
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    cx: &App,
) -> Option<PointerColor> {
    let bounds = *bounds.read(cx);
    let (reach, extent) = if vertical {
        (
            bounds.origin.y + bounds.size.height
                - COLOR_SLIDER_TRACK_INSET_PX
                - f32::from(position.y),
            bounds.size.height - COLOR_SLIDER_TRACK_INSET_PX * 2.0,
        )
    } else {
        (
            f32::from(position.x) - bounds.origin.x - COLOR_SLIDER_TRACK_INSET_PX,
            bounds.size.width - COLOR_SLIDER_TRACK_INSET_PX * 2.0,
        )
    };
    if extent <= 0.0 {
        return None;
    }
    let fraction = (reach / extent).clamp(0.0, 1.0);
    let (min, max) = channel.range();
    let next = snap_color_channel(channel, min + fraction * (max - min));
    Some(PointerColor {
        changed: next != value.channel_in(channel, color_space),
        value: value.with_channel_in(channel, color_space, next),
    })
}

#[allow(clippy::too_many_arguments)] // the channel and callback are the color-slider drag state
fn finish_slider_drag(
    dragging: &Entity<bool>,
    bounds: &Entity<Bounds<f32>>,
    position: gpui::Point<Pixels>,
    vertical: bool,
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    on_change_end: &Option<OnColorChange>,
    window: &mut Window,
    cx: &mut App,
) {
    if !*dragging.read(cx) {
        return;
    }
    dragging.update(cx, |value, cx| {
        *value = false;
        cx.notify();
    });
    if let (Some(callback), Some(next)) = (
        on_change_end,
        slider_color_from_pointer(bounds, position, vertical, value, channel, color_space, cx),
    ) {
        callback(next.value, window, cx);
    }
}

// ---------------------------------------------------------------------------
// ColorField
// ---------------------------------------------------------------------------

/// The complete state passed to [`ColorField::content`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorFieldRenderState {
    /// The field cannot receive focus or input.
    pub is_disabled: bool,
    /// Controlled, server, or custom validation currently fails.
    pub is_invalid: bool,
    /// The value can be selected but not changed.
    pub is_read_only: bool,
    /// The field must contain a value before native form submission.
    pub is_required: bool,
    /// The input itself owns keyboard focus.
    pub is_focused: bool,
    /// The input or another composed child owns keyboard focus.
    pub is_focus_within: bool,
    /// Focus was reached through keyboard navigation.
    pub is_focus_visible: bool,
}

/// ColorField — enters a color as text.
///
/// With no `channel` it edits the hex value; with one it edits that channel's
/// numeric value.
#[derive(IntoElement)]
pub struct ColorField {
    /// See [`ColorField::content`].
    content: Option<Arc<dyn Fn(ColorFieldRenderState) -> gpui::AnyElement + 'static>>,
    /// `ColorField.Suffix` — the `me-3` slot after the value, in the
    /// placeholder colour. v3's own example fills the *prefix* with a swatch and
    /// leaves this to the caller (a channel unit, a lock icon).
    suffix: Option<gpui::AnyElement>,
    /// `validationBehavior` — carried on this control's form field.
    validation_behavior: crate::form::ValidationBehavior,
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
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
    form_state: Rc<RefCell<crate::form::LiveFormFieldState>>,
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

    /// v3's field `children`-as-a-function, handed the complete resolved field
    /// state.
    pub fn content(
        mut self,
        render: impl Fn(ColorFieldRenderState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(Arc::new(render));
        self
    }

    /// `ColorField.Suffix` — the slot after the value.
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    pub fn new(id: impl Into<ElementId>, value: PickerColor) -> Self {
        Self {
            content: None,
            suffix: None,
            validation_behavior: crate::form::ValidationBehavior::Native,
            name: None,
            default_value: None,
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
            form_state: live_color_form_state(
                crate::form::FormValue::Text(SharedString::default()),
            ),
        }
    }

    /// `validationBehavior` — `Allow` shows the message without blocking form
    /// submission. Carried on the [`Self::form_field`] this control produces.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
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
        let value = self.default_value.unwrap_or(self.value);
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&value)),
            None,
        );
        sync_color_form_state(
            &self.form_state,
            color_field_form_value(value, self.channel, self.color_space),
            !self.is_disabled,
            validity.is_invalid,
        );
        Some(
            crate::form::FormField::live(name, self.form_state.clone())
                .is_required(self.is_required)
                .validation_behavior(self.validation_behavior),
        )
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `colorSpace` — the space a `channel` value is read in.
    pub fn color_space(mut self, space: ColorSpace) -> Self {
        self.color_space = space;
        self
    }

    /// `validate` — returns the message to show, or `None` when the colour is
    /// fine. The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&PickerColor) -> Option<SharedString> + 'static) -> Self {
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
        color_field_display_text(self.value, self.channel, self.color_space)
    }
}

fn parse_color_field(
    value: PickerColor,
    channel: Option<ColorChannel>,
    color_space: ColorSpace,
    text: &str,
) -> Option<PickerColor> {
    match channel {
        None => PickerColor::from_hex(text),
        Some(channel) => {
            let text = text.trim();
            let mut number: f32 = text.trim_end_matches('%').trim().parse().ok()?;
            if is_normalized_channel(channel) {
                number /= 100.0;
            }
            let (min, max) = channel.range();
            if number < min || number > max {
                return None;
            }
            Some(value.with_channel_in(channel, color_space, number))
        }
    }
}

fn step_color_channel(
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    direction: f32,
) -> PickerColor {
    let (min, max) = channel.range();
    let step = color_channel_step(channel);
    let current = value.channel_in(channel, color_space);
    let next = snap_color_channel(channel, (current + direction * step).clamp(min, max));
    value.with_channel_in(channel, color_space, next)
}

fn color_channel_step(channel: ColorChannel) -> f32 {
    let (min, max) = channel.range();
    if max - min > 2.0 {
        1.0
    } else {
        0.01
    }
}

fn snap_color_channel(channel: ColorChannel, value: f32) -> f32 {
    let (min, max) = channel.range();
    let step = color_channel_step(channel);
    (((value - min) / step).round() * step + min).clamp(min, max)
}

fn is_normalized_channel(channel: ColorChannel) -> bool {
    matches!(
        channel,
        ColorChannel::Saturation
            | ColorChannel::Brightness
            | ColorChannel::Lightness
            | ColorChannel::Alpha
    )
}

fn format_color_channel_value(
    value: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
) -> String {
    let value = value.channel_in(channel, color_space);
    if is_normalized_channel(channel) {
        format!("{}%", (value * 100.0).round())
    } else {
        format!("{}", value.round())
    }
}

#[allow(clippy::too_many_arguments)] // mirrors the state, callback and field channels in one event
fn report_color_field_change(
    next: PickerColor,
    channel: ColorChannel,
    color_space: ColorSpace,
    state: &Entity<crate::input::InputState>,
    own: &Option<Entity<PickerColor>>,
    on_change: &Option<OnColorFieldChange>,
    window: &mut Window,
    cx: &mut App,
) {
    let text = format_color_channel_value(next, channel, color_space);
    state.update(cx, |state, cx| {
        state.set_value(text);
        cx.notify();
    });
    if let Some(held) = own {
        held.update(cx, |value, cx| {
            *value = next;
            cx.notify();
        });
    }
    if let Some(callback) = on_change {
        callback(Some(next), window, cx);
    }
}

impl RenderOnce for ColorField {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own colour;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-field-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );
        self.value = resolved;
        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&self.value)),
            None,
        );
        if let Some(render) = self.content.clone() {
            // v3's field children-as-a-function: the caller builds the parts.
            let focused = self
                .state
                .as_ref()
                .is_some_and(|s| s.read(cx).focus_handle.is_focused(window));
            let within = self
                .state
                .as_ref()
                .is_some_and(|s| s.read(cx).focus_handle.contains_focused(window, cx));
            return render(ColorFieldRenderState {
                is_disabled: self.is_disabled,
                is_invalid: validity.is_invalid,
                is_read_only: self.is_read_only,
                is_required: self.is_required,
                is_focused: focused,
                is_focus_within: within,
                is_focus_visible: focused && util::focus_visible(cx),
            })
            .into_any_element();
        }
        let form_default = window.use_keyed_state(
            ElementId::Name(format!("{:?}-field-form-default", self.id).into()),
            cx,
            |_, _| None::<PickerColor>,
        );
        if form_default.read(cx).is_none() {
            let initial = self.value;
            form_default.update(cx, |slot, cx| {
                *slot = Some(initial);
                cx.notify();
            });
        }
        let restore_default = form_default.read(cx).unwrap_or(self.value);
        // Submit the resolved colour. Uncontrolled keyed state is current after
        // a parsed change; a controlled owner must accept it first.
        sync_color_form_state(
            &self.form_state,
            color_field_form_value(self.value, self.channel, self.color_space),
            !self.is_disabled,
            validity.is_invalid,
        );
        let restore_own = own.clone();
        let restore_on_change = self.on_change.clone();
        let restore_form_state = self.form_state.clone();
        let restore_input = self.state.clone();
        let restore_channel = self.channel;
        let restore_space = self.color_space;
        let restore_is_disabled = self.is_disabled;
        let restore: Arc<dyn Fn(&mut Window, &mut App)> = util::shared(move |window, cx| {
            if let Some(own) = &restore_own {
                own.update(cx, |current, cx| {
                    *current = restore_default;
                    cx.notify();
                });
            }
            if let Some(state) = &restore_input {
                let text =
                    color_field_display_text(restore_default, restore_channel, restore_space);
                state.update(cx, |state, cx| {
                    state.set_value(text);
                    cx.notify();
                });
            }
            if let Some(callback) = &restore_on_change {
                callback(Some(restore_default), window, cx);
            }
            sync_color_form_state(
                &restore_form_state,
                color_field_form_value(restore_default, restore_channel, restore_space),
                !restore_is_disabled,
                false,
            );
        });
        self.form_state.borrow_mut().restore = Some(restore);
        if let Some(state) = &self.state {
            self.form_state.borrow_mut().focus = Some(state.read(cx).focus_handle.clone());
        }
        let colors = cx.colors();
        let layout = cx.layout();
        let text = self.display_text();

        // Editable mode: delegate the text handling to Input and parse on every
        // keystroke, so `onChange` reports exactly what v3's does.
        if let Some(state) = self.state.clone() {
            let mut input = Input::new(state.clone())
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
                input = input.placeholder(text);
            }
            if let Some(label) = self.label.clone() {
                input = input.label(label);
            }
            if let Some(description) = self.description.clone() {
                input = input.description(description);
            }
            if let Some(suffix) = self.suffix.take() {
                input = input.end_content(
                    div()
                        .flex()
                        .items_center()
                        .flex_shrink_0()
                        .text_color(colors.field.placeholder)
                        .child(suffix),
                );
            }
            if self.full_width {
                input = input.full_width();
            }
            if self.on_change.is_some() || own.is_some() {
                let cb = self.on_change.clone();
                let own = own.clone();
                let parse_value = self.value;
                let parse_channel = self.channel;
                let parse_space = self.color_space;
                input = input.on_change(move |text, window, cx| {
                    let next = parse_color_field(parse_value, parse_channel, parse_space, text);
                    // Uncontrolled: keep what was typed, or the swatch would
                    // never follow the text.
                    if let (Some(held), Some(c)) = (&own, next) {
                        held.update(cx, |v, cx| {
                            *v = c;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &cb {
                        cb(next, window, cx);
                    }
                });
            }
            let rendered = input.render(window, cx).into_any_element();
            let Some(channel) = self.channel else {
                return rendered;
            };
            if self.is_disabled || self.is_read_only {
                return rendered;
            }

            let mut field = div()
                .id(ElementId::Name(
                    format!("{:?}-channel-events", self.id).into(),
                ))
                .child(rendered);
            if self.on_change.is_some() || own.is_some() {
                let key_value = self.value;
                let key_space = self.color_space;
                let key_state = state.clone();
                let key_own = own.clone();
                let key_change = self.on_change.clone();
                field = field.on_key_down(move |event, window, cx| {
                    let direction = match event.keystroke.key.as_str() {
                        "up" => 1.0,
                        "down" => -1.0,
                        _ => return,
                    };
                    let next = step_color_channel(key_value, channel, key_space, direction);
                    report_color_field_change(
                        next,
                        channel,
                        key_space,
                        &key_state,
                        &key_own,
                        &key_change,
                        window,
                        cx,
                    );
                    cx.stop_propagation();
                });

                if !self.is_wheel_disabled {
                    let wheel_value = self.value;
                    let wheel_space = self.color_space;
                    let wheel_state = state;
                    let wheel_own = own;
                    let wheel_change = self.on_change;
                    field = field.on_scroll_wheel(move |event, window, cx| {
                        if !wheel_state
                            .read(cx)
                            .focus_handle
                            .contains_focused(window, cx)
                        {
                            return;
                        }
                        let (dx, dy) = match event.delta {
                            gpui::ScrollDelta::Pixels(point) => {
                                (f32::from(point.x), f32::from(point.y))
                            }
                            gpui::ScrollDelta::Lines(point) => (point.x, point.y),
                        };
                        if dy == 0.0 || dy.abs() <= dx.abs() {
                            return;
                        }
                        let next =
                            step_color_channel(wheel_value, channel, wheel_space, dy.signum());
                        report_color_field_change(
                            next,
                            channel,
                            wheel_space,
                            &wheel_state,
                            &wheel_own,
                            &wheel_change,
                            window,
                            cx,
                        );
                        cx.stop_propagation();
                    });
                }
            }
            return field.into_any_element();
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
            .line_height(px(20.))
            .text_color(colors.field.foreground)
            // `.color-input-group__prefix` is `shrink-0 ms-3` in the
            // placeholder colour, and v3's example puts the swatch in it;
            // `.color-input-group__suffix` is its `me-3` twin.
            .child(ColorSwatch::new(self.value).size(SizeXl::Xs))
            .child(div().flex_1().child(text))
            .children(self.suffix.map(|el| {
                div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .text_color(colors.field.placeholder)
                    .child(el)
            }));

        field = util::apply_field_chrome(
            field,
            self.variant,
            self.is_invalid,
            self.state
                .as_ref()
                .is_some_and(|s| s.read(cx).focus_handle.is_focused(window)),
            cx,
        );

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
                let next = (value.channel_in(channel, space) + step * dy.signum()).clamp(min, max);
                cb(
                    Some(value.with_channel_in(channel, space, next)),
                    window,
                    cx,
                );
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

        // `.color-field` is `flex flex-col gap-1`.
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .when(self.full_width, |root| root.w_full());
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

/// State handed to `ColorSwatchPicker.Item`'s render function.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorSwatchPickerItemState {
    pub color: PickerColor,
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_selected: bool,
    pub is_focused: bool,
    pub is_focus_visible: bool,
    pub is_disabled: bool,
}

/// ColorSwatchPicker — chooses from a predefined palette.
#[derive(IntoElement)]
pub struct ColorSwatchPicker {
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
    id: ElementId,
    swatches: Vec<PickerColor>,
    value: Option<PickerColor>,
    size: SizeXl,
    shape: SwatchShape,
    layout: SwatchLayout,
    is_disabled: bool,
    /// `ColorSwatchPicker.Item.isDisabled` — swatches that cannot be chosen,
    /// by index.
    disabled_keys: std::collections::HashSet<usize>,
    item_content:
        Option<Arc<dyn Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static>>,
    indicator: Option<Arc<dyn Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static>>,
    on_change: Option<OnColorChange>,
}

impl ColorSwatchPicker {
    pub fn new(id: impl Into<ElementId>, swatches: Vec<PickerColor>) -> Self {
        Self {
            default_value: None,
            id: id.into(),
            swatches,
            value: None,
            // `.color-swatch` is `size-8` (32px), which is `SizeXl::Md` on v3's
            // own swatch scale (16/24/32/36/40).
            size: SizeXl::Md,
            shape: SwatchShape::Circle,
            layout: SwatchLayout::Grid,
            is_disabled: false,
            disabled_keys: std::collections::HashSet::new(),
            item_content: None,
            indicator: None,
            on_change: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
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

    /// `ColorSwatchPicker.Item.isDisabled` — swatches that cannot be chosen,
    /// by index.
    ///
    /// A disabled swatch draws dimmed (`status-disabled`'s reduced opacity),
    /// answers no click, and leaves the tab order, exactly as a disabled
    /// control must in this port. The dictionary is by palette position — the
    /// same projection `RadioGroup::disabled_keys` gives `Radio.isDisabled` —
    /// since the swatches are a plain list with no keys of their own.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = usize>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `children` on `ColorSwatchPicker.Item` — replaces the built-in swatch
    /// and indicator while the stable item keeps selection and navigation.
    /// The closure receives the item's palette index and the complete pinned
    /// React Aria item render state.
    pub fn item_content(
        mut self,
        render: impl Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.item_content = Some(Arc::new(render));
        self
    }

    /// `ColorSwatchPicker.Indicator` — replaces the selected checkmark. The
    /// render function receives the same item state as its pinned compound
    /// part, including while the indicator is visually hidden.
    pub fn indicator(
        mut self,
        render: impl Fn(usize, ColorSwatchPickerItemState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.indicator = Some(Arc::new(render));
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-swatches-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.or(self.value),
        );
        self.value = resolved;

        // One tab stop for the whole list: v3's picker sits on React Aria's
        // ListBox, which roves a listbox's tabindex, so Tab enters the picker
        // once and the arrows move inside it (AGENTS.md's roving tab stop).
        // Which swatch claims the group's handle is held in keyed state,
        // because a handle's `tab_stop` is fixed where the handle is made;
        // the cursor never rests on a disabled swatch, so a stop stranded on
        // one cannot take the picker out of the tab order. The state precedes
        // the theme borrow.
        let swatch_focus = util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        let cursor = window.use_keyed_state(
            ElementId::Name(format!("{:?}-cursor", self.id).into()),
            cx,
            |_, _| 0usize,
        );
        let enabled: Vec<usize> = (0..self.swatches.len())
            .filter(|index| !(self.is_disabled || self.disabled_keys.contains(index)))
            .collect();
        let at = *cursor.read(cx);
        let cursor_index = enabled
            .iter()
            .copied()
            .find(|i| *i >= at)
            .or_else(|| enabled.first().copied());
        let interactions: Vec<util::Interaction> = if self.item_content.is_some()
            || self.indicator.is_some()
        {
            (0..self.swatches.len())
                .map(|index| {
                    util::interaction(
                        ElementId::Name(format!("{:?}-swatch-{index}-interaction", self.id).into()),
                        window,
                        cx,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let pointer_focus = window.use_keyed_state(
            ElementId::Name(format!("{:?}-pointer-focus", self.id).into()),
            cx,
            |_, _| None::<usize>,
        );
        let item_edge = self.size.swatch_px();
        let border_width = match self.size {
            SizeXl::Xs => px(1.),
            SizeXl::Sm | SizeXl::Md => px(2.),
            SizeXl::Lg | SizeXl::Xl => px(3.),
        };
        let item_radius = match self.shape {
            SwatchShape::Circle => match self.size {
                SizeXl::Xs => px(8.),
                SizeXl::Sm => px(12.),
                SizeXl::Md => px(16.),
                SizeXl::Lg | SizeXl::Xl => px(24.),
            },
            SwatchShape::Square => match self.size {
                SizeXl::Xs => px(6.),
                SizeXl::Sm => px(8.),
                SizeXl::Md | SizeXl::Lg | SizeXl::Xl => px(12.),
            },
        };
        // RAC's grid delegate moves Up and Down between the same column of the
        // adjacent row. A grid row holds the most cells of the active size
        // that fit under the picker's 280px maximum with its 8px gap.
        let grid_columns = if self.layout == SwatchLayout::Grid {
            (((280. + 8.) / (f32::from(item_edge) + 8.)) as usize).max(1)
        } else {
            0
        };
        let swatch_ring = util::focus_visible(cx);

        let clear_pointer_focus = pointer_focus.clone();
        let mut row = div()
            .id(ElementId::Name(format!("{:?}-swatches", self.id).into()))
            .flex()
            .items_center()
            .gap(px(8.))
            .on_mouse_down_out(move |_, _, cx| {
                clear_pointer_focus.update(cx, |focused, cx| {
                    if focused.take().is_some() {
                        cx.notify();
                    }
                });
            });
        row = match self.layout {
            SwatchLayout::Grid => row.flex_row().flex_wrap().max_w(px(280.)),
            SwatchLayout::Stack => row.flex_col(),
        };

        for (index, swatch) in self.swatches.iter().enumerate() {
            let selected = self.value.is_some_and(|v| v.to_hex() == swatch.to_hex());
            // `ColorSwatchPicker.Item.isDisabled` — the item's own flag
            // beside the group-wide one: dimmed, no press, and out of the tab
            // order (`track_focus` below is gated on it, and a stop resting
            // on nothing is what takes a control out of the order).
            let item_disabled = self.is_disabled || self.disabled_keys.contains(&index);
            let (recorded_hover, recorded_press) = interactions
                .get(index)
                .map(|slot| *slot.read(cx))
                .unwrap_or_default();
            let item_focused = !item_disabled
                && cursor_index == Some(index)
                && (swatch_focus.is_focused(window)
                    || *pointer_focus.read(cx) == Some(index)
                    || recorded_press);
            let state = ColorSwatchPickerItemState {
                color: *swatch,
                is_hovered: !item_disabled && recorded_hover,
                is_pressed: !item_disabled && recorded_press,
                is_selected: selected,
                is_focused: item_focused,
                is_focus_visible: item_focused && swatch_ring,
                is_disabled: item_disabled,
            };

            let mut cell = div()
                .id(ElementId::Name(
                    format!("{:?}-swatch-{index}", self.id).into(),
                ))
                .when(cursor_index == Some(index), |c| {
                    c.track_focus(&swatch_focus)
                })
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .size(item_edge)
                .rounded(item_radius)
                .border(border_width);

            if let Some(render) = &self.item_content {
                cell = cell.child(render(index, state));
            } else {
                cell = cell.child({
                    // `.color-swatch-picker__swatch` is `size-full` inside the
                    // border, `scale(1.1)` on hover and `scale(0.77)` when the
                    // item is selected. gpui has no div transform, so each of
                    // those is the size it comes to.
                    let base_edge = f32::from(item_edge) - 2. * f32::from(border_width);
                    let edge = px(if selected {
                        base_edge * 0.77
                    } else {
                        base_edge
                    });
                    let radius = match (self.shape, self.size, selected) {
                        (SwatchShape::Circle, _, _) => item_radius,
                        (SwatchShape::Square, SizeXl::Xs, _) => px(6.),
                        (SwatchShape::Square, SizeXl::Sm, true) => px(6.),
                        (SwatchShape::Square, _, _) => px(8.),
                    };
                    let grown = px(f32::from(edge) * 1.1);
                    div()
                        .size(edge)
                        .rounded(radius)
                        .flex_shrink_0()
                        .overflow_hidden()
                        // The checkerboard that shows through a translucent
                        // colour, as on a plain `ColorSwatch`.
                        .bg(cx.colors().surface_secondary)
                        .when(!item_disabled && !selected, |el| {
                            el.hover(move |st| st.size(grown))
                        })
                        .child(div().size_full().rounded(radius).bg(swatch.to_hsla()))
                });

                // `.color-swatch-picker__indicator` spans the *item* (`absolute
                // inset-0`) and centres a checkmark at `size-1/3` of it -- white by
                // default, black over a light colour (`data-light-color`).
                if let Some(render) = &self.indicator {
                    cell = cell.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(!selected, |indicator| indicator.opacity(0.))
                            .child(render(index, state)),
                    );
                } else if selected {
                    cell = cell.child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                gpui::svg()
                                    .size(px(f32::from(item_edge) / 3.))
                                    .path(crate::icons::CHECK)
                                    .text_color(color_swatch_indicator_color(*swatch)),
                            ),
                    );
                }
            }

            // Selected: `border-color: var(--color-swatch-current)` -- the
            // border takes the swatch's own colour, and the gap the shrunk
            // swatch leaves is what reads as a ring.
            if selected {
                cell = cell.border_color(swatch.to_hsla());
            } else {
                cell = cell.border_color(gpui::transparent_black());
            }

            if item_disabled {
                // v3's sheet: `[data-disabled="true"]` is `status-disabled`,
                // reduced opacity, and no press handler is attached here at
                // all -- a disabled swatch cannot report a choice.
                cell = cell.opacity(cx.layout().disabled_opacity);
            } else if self.on_change.is_some() || own.is_some() {
                let on_change = self.on_change.clone();
                let own = own.clone();
                let value = *swatch;
                cell = cell.cursor_pointer().on_click(move |_, window, cx| {
                    // Uncontrolled: take the selection, or the press would do
                    // nothing.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = Some(value);
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change {
                        cb(value, window, cx);
                    }
                });
            }

            if !item_disabled {
                let moved = cursor.clone();
                let focus = swatch_focus.clone();
                let pointer = pointer_focus.clone();
                if let Some(interaction) = interactions.get(index) {
                    cell = util::track_interaction_on_mouse_down(
                        cell,
                        interaction,
                        move |window, cx| {
                            moved.update(cx, |cursor, cx| {
                                *cursor = index;
                                cx.notify();
                            });
                            pointer.update(cx, |focused, cx| {
                                *focused = Some(index);
                                cx.notify();
                            });
                            window.focus(&focus, cx);
                        },
                    );
                } else {
                    cell = cell.on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                        moved.update(cx, |cursor, cx| {
                            *cursor = index;
                            cx.notify();
                        });
                        pointer.update(cx, |focused, cx| {
                            *focused = Some(index);
                            cx.notify();
                        });
                        window.focus(&focus, cx);
                    });
                }
            }

            // The collection keyboard, the ListBox's: the arrows rove the
            // focus over the enabled swatches and Home and End jump, while a
            // disabled swatch is never a stop. Enter and Space stay with gpui
            // -- a focused element's press fires on key up, so the click
            // handler above is the only path that selects; binding them again
            // here would select twice. Disabled swatches never see this
            // handler (they cannot hold the focus), so the cursor only moves
            // between `enabled`.
            if !item_disabled {
                let stops = enabled.clone();
                let moved = cursor.clone();
                let pointer = pointer_focus.clone();
                let columns = grid_columns;
                cell = cell.on_key_down(move |event, _window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "tab" {
                        pointer.update(cx, |focused, cx| {
                            if focused.take().is_some() {
                                cx.notify();
                            }
                        });
                        return;
                    }
                    let next = match key {
                        // Grid rows: same column, adjacent row. The strided
                        // target must be a real enabled swatch; otherwise the
                        // row does not exist, which is what the geometric
                        // delegate reports for a ragged grid too.
                        "up" | "down" if columns > 0 => {
                            let j = index as isize
                                + if key == "down" {
                                    columns as isize
                                } else {
                                    -(columns as isize)
                                };
                            if j < 0 || !stops.contains(&(j as usize)) {
                                return;
                            }
                            j as usize
                        }
                        key @ ("up" | "down") if columns == 0 => {
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&stops, Some(index), key, false)
                            else {
                                return;
                            };
                            next
                        }
                        key @ ("left" | "right" | "home" | "end") => {
                            let key = match key {
                                "right" => "down",
                                "left" => "up",
                                other => other,
                            };
                            let crate::list_nav::Move::To(next) =
                                crate::list_nav::resolve(&stops, Some(index), key, false)
                            else {
                                return;
                            };
                            next
                        }
                        _ => return,
                    };
                    cx.stop_propagation();
                    // No refocusing: the next render has the swatch at `next`
                    // claim the group's handle, so the focus goes with it.
                    moved.update(cx, |v, cx| {
                        *v = next;
                        cx.notify();
                    });
                });
            }

            let cell =
                util::with_focus_ring(cell, swatch_ring && item_focused, true, Vec::new(), cx);
            row = row.child(cell);
        }

        row
    }
}

// ---------------------------------------------------------------------------
// ColorPicker
// ---------------------------------------------------------------------------

/// ColorPicker — a swatch trigger plus a full picking surface.
#[derive(IntoElement)]
pub struct ColorPicker {
    /// `defaultValue` — set it to hand this component its own state.
    default_value: Option<PickerColor>,
    id: ElementId,
    value: PickerColor,
    label: Option<SharedString>,
    is_open: Option<bool>,
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
            default_value: None,
            id: id.into(),
            value,
            label: None,
            is_open: None,
            placement: Placement::BottomStart,
            show_alpha: false,
            is_disabled: false,
            on_change: None,
            on_open_change: None,
        }
    }

    /// `defaultValue` — the uncontrolled initial colour.
    ///
    /// Supplying it hands the component its own state: the constructor's
    /// `value` becomes the seed, and a change moves the component's copy.
    pub fn default_value(mut self, value: PickerColor) -> Self {
        self.default_value = Some(value);
        self
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
        self.is_open = Some(v);
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (is_open, open_own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-picker-open", self.id).into()),
            self.is_open,
            false,
        );
        // `defaultValue` opts into the component holding its own colour;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (resolved, own) = util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-picker-value", self.id).into()),
            match self.default_value {
                Some(_) => None,
                None => Some(self.value),
            },
            self.default_value.unwrap_or(self.value),
        );
        self.value = resolved;
        // `.color-picker__trigger:focus-visible` is `status-focused`.
        // `use_keyed_state` takes `cx` mutably, so the handle precedes the theme.
        let trigger_focus = util::tab_stop_handle(
            ElementId::Name(format!("{:?}-picker-focus", self.id).into()),
            window,
            cx,
        );
        let base = format!("{:?}", self.id);
        let overlay_open = is_open && !self.is_disabled;
        let (phase, dismissal_token) = util::overlay_scope(
            window,
            cx,
            ElementId::Name(format!("{base}-picker-overlay").into()),
            overlay_open,
            true,
        );
        let exiting = phase == util::OverlayPhase::Exiting;
        // Blur closes the logical open state without pulling focus back to the
        // trigger. Exit animation frames do not keep this watch armed.
        let group_scope = util::close_on_blur(window, cx, &base, overlay_open, {
            let cb = self.on_open_change.clone();
            let own = open_own.clone();
            move |window: &mut Window, cx: &mut App| {
                if let Some(held) = &own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &cb {
                    cb(false, window, cx);
                }
            }
        });
        let panel_state = window.use_keyed_state(
            ElementId::Name(format!("{base}-panel-scroll").into()),
            cx,
            |_, cx| {
                (
                    gpui::ScrollHandle::new(),
                    std::array::from_fn::<_, 3, _>(|_| cx.focus_handle()),
                    None::<usize>,
                )
            },
        );
        let (panel_scroll, child_focus) =
            panel_state.update(cx, |(scroll, scopes, previous), cx| {
                let focused = scopes
                    .iter()
                    .position(|scope| scope.contains_focused(window, cx));
                if focused != *previous {
                    if let Some(index) = focused {
                        scroll.scroll_to_item(index);
                    }
                    *previous = focused;
                }
                (scroll.clone(), scopes.clone())
            });

        let colors = cx.colors();
        let layout = cx.layout();
        let popover_radius = layout.capped(layout.radius_lg() * 2.5);
        let trigger_pressed = Rc::new(Cell::new(false));

        // Trigger: swatch plus the hex value.
        let mut trigger = div()
            .id(ElementId::Name(format!("{base}-trigger").into()))
            .when(!self.is_disabled, |el| el.track_focus(&trigger_focus))
            .flex()
            .flex_row()
            .items_center()
            // `.color-picker__trigger` is `inline-flex items-center gap-3
            // rounded-sm text-sm` -- a swatch beside its value, with no box of
            // its own.
            .gap(px(12.))
            .rounded(util::hairline_radius(cx))
            .text_size(px(14.))
            .line_height(px(20.))
            .text_color(colors.foreground)
            .child(ColorSwatch::new(self.value).size(SizeXl::Sm))
            .child(div().child(self.value.to_hex()));

        if self.is_disabled {
            trigger = trigger.opacity(layout.disabled_opacity);
        } else {
            trigger = trigger.cursor_pointer();
            let pressed = trigger_pressed.clone();
            trigger = trigger.capture_any_mouse_down(move |_, _, cx| {
                pressed.set(true);
                let clear = pressed.clone();
                cx.defer(move |_| clear.set(false));
            });
            if self.on_open_change.is_some() || open_own.is_some() {
                let cb = self.on_open_change.clone();
                let own = open_own.clone();
                let next = !is_open;
                let pressed = trigger_pressed.clone();
                trigger = trigger.on_click(move |_, window, cx| {
                    if let Some(held) = &own {
                        held.update(cx, |value, cx| {
                            *value = next;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &cb {
                        cb(next, window, cx);
                    }
                    pressed.set(false);
                });
            }
        }

        // The popover overlays the page rather than pushing it down.
        let mut root = div().relative().flex().flex_col().gap(px(8.));
        if let Some(label) = self.label {
            root = root.child(crate::field::Label::new(label));
        }
        let trigger = util::ring_if_focused(trigger, &trigger_focus, true, Vec::new(), window, cx);
        let anchor_bounds = Rc::new(Cell::new(None));
        root = root.child(crate::popover::PopoverTriggerMeasure::new(
            trigger,
            anchor_bounds.clone(),
        ));
        root = root.track_focus(&group_scope);

        if phase == util::OverlayPhase::Closed {
            // RAC 1.20.0 unmounts the scroll DOM after close+exit, so a
            // reopen starts at the top. The keyed handle outlives the panel,
            // so reset it only once the exit is gone; touching it while
            // Exiting would shift the visible panel.
            panel_scroll.set_offset(gpui::point(px(0.), px(0.)));
            return root;
        }

        // React Aria dismisses the panel on Escape and on a press outside it.
        // Escape rides on the root: the panel holding the focus would take the
        // arrows away from the area and the sliders inside it.
        let close = util::shared({
            let cb = self.on_open_change.clone();
            let own = open_own;
            move |window: &mut Window, cx: &mut App| -> util::DismissResult {
                window.focus(&trigger_focus, cx);
                if let Some(held) = &own {
                    held.update(cx, |value, cx| {
                        *value = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &cb {
                    cb(false, window, cx);
                }
                util::DismissResult::Handled
            }
        });
        root = util::dismiss_on_escape_with_token(root, dismissal_token.clone(), {
            let close = close.clone();
            move |window, cx| close(window, cx)
        });

        // `.color-picker__popover` is `gap-3 min-w-62 px-2`: a minimum width,
        // not the fixed 264 this used to force.
        let mut panel = div()
            .gap(px(12.))
            .px(px(8.))
            .pt(px(8.))
            .pb(px(12.))
            .min_w(px(248.))
            .id(ElementId::Name(format!("{base}-panel").into()))
            .debug_selector({ let base = base.clone(); move || format!("{base}-panel") })
            .max_h_full()
            .overflow_x_hidden()
            .overflow_y_scroll()
            .restrict_scroll_to_axis()
            .occlude()
            .track_scroll(&panel_scroll)
            .flex()
            .flex_col()
            .rounded(popover_radius)
            .bg(colors.overlay.background)
            // v3 gives a floating panel no border: it is `bg-overlay
            // shadow-overlay` and a radius, and dark mode's inset hairline is
            // what separates the panel from the page.
            .when_some(layout.overlay_hairline, |el, hairline| {
                el.border(layout.border_width).border_color(hairline)
            })
            .when(!layout.overlay_shadow.is_empty(), |e| {
                e.shadow(layout.overlay_shadow.clone())
            });

        let mut area = ColorArea::new(ElementId::Name(format!("{base}-area").into()), self.value)
            .size(px(240.), px(160.));
        {
            let cb = self.on_change.clone();
            let own = own.clone();
            area = area.on_change(move |c, window, cx| {
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = c;
                        cx.notify();
                    });
                }
                if let Some(cb) = &cb {
                    cb(c, window, cx);
                }
            });
        }
        panel = panel.child(
            div()
                .flex_shrink_0()
                .track_focus(&child_focus[0])
                .child(area),
        );

        let mut hue = ColorSlider::new(
            ElementId::Name(format!("{base}-hue").into()),
            self.value,
            ColorChannel::Hue,
        )
        .length(px(240.))
        .show_label(false);
        {
            let cb = self.on_change.clone();
            let own = own.clone();
            hue = hue.on_change(move |c, window, cx| {
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = c;
                        cx.notify();
                    });
                }
                if let Some(cb) = &cb {
                    cb(c, window, cx);
                }
            });
        }
        panel = panel.child(
            div()
                .flex_shrink_0()
                .track_focus(&child_focus[1])
                .debug_selector({
                    let base = base.clone();
                    move || format!("{base}-hue")
                })
                .child(hue),
        );

        if self.show_alpha {
            let mut alpha = ColorSlider::new(
                ElementId::Name(format!("{base}-alpha").into()),
                self.value,
                ColorChannel::Alpha,
            )
            .length(px(240.))
            .show_label(false);
            {
                let cb = self.on_change.clone();
                let own = own;
                alpha = alpha.on_change(move |c, window, cx| {
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = c;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &cb {
                        cb(c, window, cx);
                    }
                });
            }
            panel = panel.child(
                div()
                    .flex_shrink_0()
                    .track_focus(&child_focus[2])
                    .debug_selector({
                        let base = base.clone();
                        move || format!("{base}-alpha")
                    })
                    .child(alpha),
            );
        }

        panel = panel.child(
            div()
                .flex_shrink_0()
                .text_size(px(12.))
                .font_family(util::MONO_FONT)
                .text_color(colors.muted)
                .child(self.value.to_hex()),
        );

        let panel =
            util::dismiss_on_press_outside_with_token(panel, dismissal_token, move |window, cx| {
                if trigger_pressed.get() {
                    return util::DismissResult::Declined;
                }
                close(window, cx)
            });

        let zoom = crate::anim::ZoomBox {
            width: Some(px(256.)),
            padding_x: Some(px(8.)),
            padding_top: Some(px(8.)),
            padding_bottom: Some(px(12.)),
            radius: Some(popover_radius),
            ..Default::default()
        };

        let panel = if exiting {
            crate::anim::exiting(
                panel,
                ElementId::Name(format!("{base}-panel-anim-out").into()),
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                ElementId::Name(format!("{base}-panel-anim").into()),
                zoom,
                crate::anim::Motion::LIST_IN,
                cx,
            )
        };
        root.child(util::floating(crate::popover::scrollable_popover(
            anchor_bounds,
            self.placement,
            panel,
        )))
    }
}

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
