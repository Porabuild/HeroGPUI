//! ColorSwatch.

use super::*;

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
    id: Option<ElementId>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
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
            id: None,
            sx: None,
        }
    }

    /// Names this swatch so it can report `role="img"`. Unnamed swatches
    /// produce no AccessKit node — a constant id would fold every instance
    /// into one image.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
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

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the swatch's root element after every value the size, the
    /// shape and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(util::capture_sx(style));
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

        let el = div()
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
            );
        let el = util::apply_sx(el, &self.sx);
        match self.id {
            Some(id) => el
                .id(id)
                .a11y_named(
                    a11y::Role::Image,
                    &a11y::Name::labelled(self.color.to_hex()),
                )
                .into_any_element(),
            None => el.into_any_element(),
        }
    }
}

// ---------------------------------------------------------------------------
