//! Sparse, typed component defaults and reusable named recipes.
//!
//! Empty styles preserve each component's stock behavior. Recipes are resolved
//! against the active theme during render, not captured when a builder is made.

use std::collections::HashMap;

use gpui::{Div, Hsla, Pixels, SharedString, StyleRefinement, Styled};

use crate::ThemeColors;
use herogpui_core::{Color, FieldVariant, Size, Variant};

/// A color resolved from the active palette, or an application-defined literal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComponentColor {
    Literal(Hsla),
    Background,
    Foreground,
    Muted,
    Surface,
    SurfaceForeground,
    SurfaceSecondary,
    SurfaceTertiary,
    Border,
    FieldBackground,
    FieldForeground,
    FieldPlaceholder,
    Role(Color),
    RoleForeground(Color),
    RoleHover(Color),
    RoleSoft(Color),
}

impl From<Hsla> for ComponentColor {
    fn from(color: Hsla) -> Self {
        Self::Literal(color)
    }
}

impl ComponentColor {
    pub fn resolve(self, colors: &ThemeColors) -> Hsla {
        let role = |role| match role {
            Color::Default => &colors.default,
            Color::Accent => &colors.accent,
            Color::Success => &colors.success,
            Color::Warning => &colors.warning,
            Color::Danger => &colors.danger,
        };
        match self {
            Self::Literal(color) => color,
            Self::Background => colors.background,
            Self::Foreground => colors.foreground,
            Self::Muted => colors.muted,
            Self::Surface => colors.surface.background,
            Self::SurfaceForeground => colors.surface.foreground,
            Self::SurfaceSecondary => colors.surface_secondary,
            Self::SurfaceTertiary => colors.surface_tertiary,
            Self::Border => colors.border,
            Self::FieldBackground => colors.field.background,
            Self::FieldForeground => colors.field.foreground,
            Self::FieldPlaceholder => colors.field.placeholder,
            Self::Role(color) => role(color).color,
            Self::RoleForeground(color) => role(color).foreground,
            Self::RoleHover(color) => role(color).hover(),
            Self::RoleSoft(color) => role(color).soft(),
        }
    }
}

/// Sparse overlay semantics for a typed component style.
pub trait ComponentStyle: Clone + Default {
    /// Replace only the properties supplied by `overlay`.
    fn refine(&mut self, overlay: &Self);
}

/// Application-wide defaults plus named overlays for one component family.
#[derive(Clone, Debug)]
pub struct ComponentTheme<T> {
    pub defaults: T,
    pub recipes: HashMap<SharedString, T>,
}

impl<T: Default> Default for ComponentTheme<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> ComponentTheme<T> {
    pub fn new(defaults: T) -> Self {
        Self {
            defaults,
            recipes: HashMap::new(),
        }
    }

    pub fn defaults(mut self, defaults: T) -> Self {
        self.defaults = defaults;
        self
    }

    pub fn recipe(mut self, name: impl Into<SharedString>, style: T) -> Self {
        self.recipes.insert(name.into(), style);
        self
    }
}

impl<T: ComponentStyle> ComponentTheme<T> {
    /// Resolve defaults, then named recipes in order. Missing names add no
    /// override, so switching to a stock theme restores stock presentation.
    pub fn resolve(&self, names: &[SharedString]) -> T {
        let mut style = self.defaults.clone();
        for name in names {
            if let Some(recipe) = self.recipes.get(name) {
                style.refine(recipe);
            }
        }
        style
    }
}

macro_rules! component_style {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Default)]
        pub struct $name {
            $($(#[$field_meta])* pub $field: Option<$ty>,)*
        }
        impl $name {
            $(
                $(#[$field_meta])*
                pub fn $field(mut self, value: impl Into<$ty>) -> Self {
                    self.$field = Some(value.into());
                    self
                }
            )*
        }
        impl ComponentStyle for $name {
            fn refine(&mut self, overlay: &Self) {
                $(if overlay.$field.is_some() {
                    self.$field = overlay.$field.clone();
                })*
            }
        }
    };
}

component_style! {
    /// Slider perimeter radius, including both thumb layers and edge caps.
    SliderStyle { radius: Pixels }
}
component_style! {
    /// Switch track and thumb radius.
    SwitchStyle { radius: Pixels }
}
component_style! {
    /// Select trigger and detached option-panel presentation.
    SelectStyle {
        variant: FieldVariant,
        height: Pixels,
        padding_x: Pixels,
        trigger_text_size: Pixels,
        row_height: Pixels,
        row_padding_x: Pixels,
        row_padding_y: Pixels,
        row_text_size: Pixels,
        panel_padding: Pixels,
        radius: Pixels,
        row_hover_bg: ComponentColor,
        is_bare: bool,
    }
}
component_style! {
    /// Menu panel and row presentation; also inherited by submenus.
    MenuStyle {
        panel_min_width: Pixels,
        panel_max_width: Pixels,
        panel_max_height: Pixels,
        panel_padding: Pixels,
        panel_gap: Pixels,
        row_height: Pixels,
        row_padding_x: Pixels,
        row_padding_y: Pixels,
        row_text_size: Pixels,
        row_gap: Pixels,
        row_hover_bg: ComponentColor,
        row_hover_foreground: ComponentColor,
        radius: Pixels,
        animate_entry: bool,
    }
}
component_style! {
    /// Shared Input/TextField/SearchField presentation. Glyph hit testing uses
    /// the same text size; the stock 20px line advance remains unchanged.
    TextFieldStyle {
        variant: FieldVariant,
        height: Pixels,
        padding_x: Pixels,
        text_size: Pixels,
        radius: Pixels,
        is_bare: bool,
        background: ComponentColor,
        foreground: ComponentColor,
        placeholder: ComponentColor,
    }
}

/// Button recipes can combine semantic colors with a sparse GPUI root style.
/// Instance `sx` is refined over this style; instance builders retain precedence.
#[derive(Clone, Debug, Default)]
pub struct ButtonStyle {
    pub variant: Option<Variant>,
    pub size: Option<Size>,
    pub radius: Option<Pixels>,
    pub background: Option<ComponentColor>,
    pub foreground: Option<ComponentColor>,
    pub hover_bg: Option<ComponentColor>,
    pub hover_foreground: Option<ComponentColor>,
    pub pressed_bg: Option<ComponentColor>,
    pub pressed_foreground: Option<ComponentColor>,
    pub disabled_foreground: Option<ComponentColor>,
    pub style: Option<StyleRefinement>,
}

impl ButtonStyle {
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    pub fn background(mut self, color: impl Into<ComponentColor>) -> Self {
        self.background = Some(color.into());
        self
    }

    pub fn foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.foreground = Some(color.into());
        self
    }

    pub fn hover_bg(mut self, color: impl Into<ComponentColor>) -> Self {
        self.hover_bg = Some(color.into());
        self
    }

    pub fn hover_foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.hover_foreground = Some(color.into());
        self
    }

    pub fn pressed_bg(mut self, color: impl Into<ComponentColor>) -> Self {
        self.pressed_bg = Some(color.into());
        self
    }

    pub fn pressed_foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.pressed_foreground = Some(color.into());
        self
    }

    pub fn disabled_foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.disabled_foreground = Some(color.into());
        self
    }

    /// Capture visual metrics such as height, padding, text size and gap.
    /// Prefer keeping flex placement and per-instance widths at the call site.
    pub fn style(mut self, style: impl FnOnce(Div) -> Div) -> Self {
        self.style = Some(style(gpui::div()).style().clone());
        self
    }
}

impl ComponentStyle for ButtonStyle {
    fn refine(&mut self, overlay: &Self) {
        macro_rules! fields {
            ($($field:ident),*) => {
                $(if overlay.$field.is_some() {
                    self.$field = overlay.$field;
                })*
            };
        }
        fields!(
            variant,
            size,
            radius,
            background,
            foreground,
            hover_bg,
            hover_foreground,
            pressed_bg,
            pressed_foreground,
            disabled_foreground
        );
        if let Some(style) = &overlay.style {
            use gpui::Refineable as _;
            self.style
                .get_or_insert_with(StyleRefinement::default)
                .refine(style);
        }
    }
}

/// Typed theme-owned defaults. No entry changes stock behavior until configured.
#[derive(Clone, Debug, Default)]
pub struct ComponentThemes {
    pub slider: ComponentTheme<SliderStyle>,
    pub switch: ComponentTheme<SwitchStyle>,
    pub select: ComponentTheme<SelectStyle>,
    pub menu: ComponentTheme<MenuStyle>,
    pub button: ComponentTheme<ButtonStyle>,
    pub text_field: ComponentTheme<TextFieldStyle>,
}

impl ComponentThemes {
    pub fn slider(mut self, slider: ComponentTheme<SliderStyle>) -> Self {
        self.slider = slider;
        self
    }

    pub fn switch(mut self, switch: ComponentTheme<SwitchStyle>) -> Self {
        self.switch = switch;
        self
    }

    pub fn select(mut self, select: ComponentTheme<SelectStyle>) -> Self {
        self.select = select;
        self
    }

    pub fn menu(mut self, menu: ComponentTheme<MenuStyle>) -> Self {
        self.menu = menu;
        self
    }

    pub fn button(mut self, button: ComponentTheme<ButtonStyle>) -> Self {
        self.button = button;
        self
    }

    pub fn text_field(mut self, text_field: ComponentTheme<TextFieldStyle>) -> Self {
        self.text_field = text_field;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemeColors;
    use gpui::px;
    use herogpui_core::oklch;

    #[test]
    fn empty_styles_resolve_to_stock_none_fields() {
        let themes = ComponentThemes::default();
        let slider = themes.slider.resolve(&[]);
        assert_eq!(slider.radius, None);
        let button = themes.button.resolve(&[]);
        assert_eq!(button.variant, None);
        assert!(button.style.is_none());
    }

    #[test]
    fn recipes_refine_in_order_and_missing_names_are_ignored() {
        let theme = ComponentTheme::new(MenuStyle::default().panel_gap(px(2.)))
            .recipe(
                "compact",
                MenuStyle::default().row_height(px(28.)).panel_gap(px(0.)),
            )
            .recipe(
                "accent",
                MenuStyle::default().row_hover_bg(ComponentColor::Role(Color::Accent)),
            );
        let missing = theme.resolve(&["missing".into()]);
        assert_eq!(missing.panel_gap, Some(px(2.)));
        assert_eq!(missing.row_height, None);

        let stacked = theme.resolve(&["compact".into(), "accent".into()]);
        assert_eq!(stacked.panel_gap, Some(px(0.)));
        assert_eq!(stacked.row_height, Some(px(28.)));
        assert_eq!(
            stacked.row_hover_bg,
            Some(ComponentColor::Role(Color::Accent))
        );
    }

    #[test]
    fn button_style_merges_refinements() {
        let base = ButtonStyle::default()
            .radius(px(8.))
            .style(|el| el.h(px(36.)).px(px(16.)));
        let overlay = ButtonStyle::default()
            .hover_bg(ComponentColor::Muted)
            .style(|el| el.h(px(28.)));
        let mut merged = base;
        merged.refine(&overlay);
        assert_eq!(merged.radius, Some(px(8.)));
        assert_eq!(merged.hover_bg, Some(ComponentColor::Muted));
        let boxed = Some(Box::new(merged.style.unwrap()));
        let size = {
            // Mirror the component helper: only pixel heights extract.
            match boxed.as_ref().unwrap().size.height {
                Some(gpui::Length::Definite(gpui::DefiniteLength::Absolute(
                    gpui::AbsoluteLength::Pixels(pixels),
                ))) => Some(pixels),
                _ => None,
            }
        };
        assert_eq!(size, Some(px(28.)));
    }

    #[test]
    fn component_color_resolves_roles_against_the_active_palette() {
        let colors = ThemeColors::light();
        assert_eq!(
            ComponentColor::Role(Color::Accent).resolve(&colors),
            colors.accent.color
        );
        let literal = oklch(0.2, 0.0, 0.0);
        assert_eq!(ComponentColor::from(literal).resolve(&colors), literal);
    }
}
