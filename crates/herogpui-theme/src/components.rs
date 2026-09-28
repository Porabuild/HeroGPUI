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
    /// A literal color supplied by the application.
    Literal(Hsla),
    /// The theme's `--background`.
    Background,
    /// The theme's `--foreground`.
    Foreground,
    /// The theme's `--muted`.
    Muted,
    /// The `--surface` background.
    Surface,
    /// The `--surface-foreground` color.
    SurfaceForeground,
    /// The `--surface-secondary` color.
    SurfaceSecondary,
    /// The `--surface-tertiary` color.
    SurfaceTertiary,
    /// The theme's `--border`.
    Border,
    /// The `--field-background` color.
    FieldBackground,
    /// The `--field-foreground` color.
    FieldForeground,
    /// The `--field-placeholder` color.
    FieldPlaceholder,
    /// A role's base color.
    Role(Color),
    /// A role's on-color foreground.
    RoleForeground(Color),
    /// A role's hover shade.
    RoleHover(Color),
    /// A role's soft (tinted) shade.
    RoleSoft(Color),
}

impl From<Hsla> for ComponentColor {
    fn from(color: Hsla) -> Self {
        Self::Literal(color)
    }
}

impl ComponentColor {
    /// Resolves this color against `colors`.
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
#[non_exhaustive]
pub struct ComponentTheme<T> {
    /// Defaults applied to every instance of the component.
    pub defaults: T,
    /// Named overlays, refined over the defaults in the order they are requested.
    pub recipes: HashMap<SharedString, T>,
}

impl<T: Default> Default for ComponentTheme<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> ComponentTheme<T> {
    /// Creates a theme with the given defaults and no recipes.
    pub fn new(defaults: T) -> Self {
        Self {
            defaults,
            recipes: HashMap::new(),
        }
    }

    /// Replaces the defaults.
    pub fn defaults(mut self, defaults: T) -> Self {
        self.defaults = defaults;
        self
    }

    /// Registers a named recipe, replacing any earlier recipe of that name.
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
        #[non_exhaustive]
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
    SliderStyle {
        /// Corner radius.
        radius: Pixels,
    }
}
component_style! {
    /// Switch track and thumb radius.
    SwitchStyle {
        /// Corner radius.
        radius: Pixels,
    }
}
component_style! {
    /// Select trigger and detached option-panel presentation.
    SelectStyle {
        /// Field variant.
        variant: FieldVariant,
        /// Trigger height.
        height: Pixels,
        /// Horizontal padding on the trigger.
        padding_x: Pixels,
        /// Vertical padding on the trigger, in place of v3's `py-2`.
        ///
        /// The trigger is `min-h` based, so a single-line value keeps the
        /// resolved height (36px of `line-height: 20px` plus 8px each side)
        /// while a value that wraps to two lines grows the trigger the way
        /// upstream's `py-2` does. Unset leaves the trigger unpadded, which is
        /// the pre-0.10.0 geometry.
        padding_y: Pixels,
        /// Text size in the trigger.
        trigger_text_size: Pixels,
        /// Option row height.
        row_height: Pixels,
        /// Horizontal padding on an option row.
        row_padding_x: Pixels,
        /// Vertical padding on an option row.
        row_padding_y: Pixels,
        /// Text size in an option row.
        row_text_size: Pixels,
        /// Padding inside the option panel.
        panel_padding: Pixels,
        /// Corner radius.
        radius: Pixels,
        /// Background of a hovered option row.
        row_hover_bg: ComponentColor,
        /// Whether the trigger drops its own field chrome.
        is_bare: bool,
    }
}
component_style! {
    /// Menu panel and row presentation; also inherited by submenus.
    MenuStyle {
        /// Minimum panel width.
        panel_min_width: Pixels,
        /// Maximum panel width.
        panel_max_width: Pixels,
        /// Maximum panel height.
        panel_max_height: Pixels,
        /// Padding inside the panel.
        panel_padding: Pixels,
        /// Gap between rows in the panel.
        panel_gap: Pixels,
        /// Row height.
        row_height: Pixels,
        /// Horizontal padding on a row.
        row_padding_x: Pixels,
        /// Vertical padding on a row.
        row_padding_y: Pixels,
        /// Row text size.
        row_text_size: Pixels,
        /// Gap between a row's contents.
        row_gap: Pixels,
        /// Background of a hovered row.
        row_hover_bg: ComponentColor,
        /// Foreground of a hovered row.
        row_hover_foreground: ComponentColor,
        /// Corner radius.
        radius: Pixels,
        /// An absolute horizontal inset on each edge of a
        /// `MenuItem::Separator`, in place of v3's proportional
        /// `ms-[3%] w-[94%]`.
        ///
        /// Unset keeps that proportional rule. Set, the inset is the same
        /// number of pixels at any panel width — AppKit's own menu separator
        /// is inset 15pt on each side inside wider panel bounds.
        separator_inset: Pixels,
        /// Thickness of a `MenuItem::Separator`.
        ///
        /// Unset keeps `LayoutTheme::border_width`, the hairline every other
        /// rule in the port uses.
        separator_thickness: Pixels,
        /// Whether the panel and its submenus play their entry animation.
        animate_entry: bool,
    }
}
component_style! {
    /// Shared Input/TextField/SearchField presentation. Glyph hit testing uses
    /// the same text size; the stock 20px line advance remains unchanged.
    TextFieldStyle {
        /// Field variant.
        variant: FieldVariant,
        /// Field height.
        height: Pixels,
        /// Horizontal padding.
        padding_x: Pixels,
        /// Text size.
        text_size: Pixels,
        /// Corner radius.
        radius: Pixels,
        /// Whether the field drops its own chrome (background, border, shadow and rings).
        is_bare: bool,
        /// Whether the focus ring is drawn.
        focus_ring: bool,
        /// Background color.
        background: ComponentColor,
        /// Foreground (text) color.
        foreground: ComponentColor,
        /// Placeholder color.
        placeholder: ComponentColor,
    }
}

/// Button recipes can combine semantic colors with a sparse GPUI root style.
/// Instance `sx` is refined over this style; instance builders retain precedence.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ButtonStyle {
    /// The variant this recipe selects.
    pub variant: Option<Variant>,
    /// The size this recipe selects.
    pub size: Option<Size>,
    /// Corner radius.
    pub radius: Option<Pixels>,
    /// Resting background.
    pub background: Option<ComponentColor>,
    /// Resting foreground.
    pub foreground: Option<ComponentColor>,
    /// Background while hovered.
    pub hover_bg: Option<ComponentColor>,
    /// Foreground while hovered.
    pub hover_foreground: Option<ComponentColor>,
    /// Background while pressed.
    pub pressed_bg: Option<ComponentColor>,
    /// Foreground while pressed.
    pub pressed_foreground: Option<ComponentColor>,
    /// Foreground while disabled.
    pub disabled_foreground: Option<ComponentColor>,
    /// A sparse GPUI root style refined over the button.
    pub style: Option<StyleRefinement>,
}

impl ButtonStyle {
    /// The variant this recipe selects.
    ///
    /// Ignored on any button whose call site named a variant of its own: the
    /// instance is the more specific source. A recipe that must survive
    /// `.variant(..)` should say what it wants outright —
    /// [`ButtonStyle::hover_bg`], [`ButtonStyle::background`] and their
    /// neighbours all apply whatever variant ends up in force — rather than
    /// express it by selecting a variant whose derived shades happen to match.
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = Some(variant);
        self
    }

    /// Sets the size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the corner radius.
    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// Sets the resting background.
    pub fn background(mut self, color: impl Into<ComponentColor>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Sets the resting foreground.
    pub fn foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.foreground = Some(color.into());
        self
    }

    /// Sets the hover background.
    pub fn hover_bg(mut self, color: impl Into<ComponentColor>) -> Self {
        self.hover_bg = Some(color.into());
        self
    }

    /// Sets the hover foreground.
    pub fn hover_foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.hover_foreground = Some(color.into());
        self
    }

    /// Sets the pressed background.
    pub fn pressed_bg(mut self, color: impl Into<ComponentColor>) -> Self {
        self.pressed_bg = Some(color.into());
        self
    }

    /// Sets the pressed foreground.
    pub fn pressed_foreground(mut self, color: impl Into<ComponentColor>) -> Self {
        self.pressed_foreground = Some(color.into());
        self
    }

    /// Sets the disabled foreground.
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
#[non_exhaustive]
pub struct ComponentThemes {
    /// Slider defaults and recipes.
    pub slider: ComponentTheme<SliderStyle>,
    /// Switch defaults and recipes.
    pub switch: ComponentTheme<SwitchStyle>,
    /// Select defaults and recipes.
    pub select: ComponentTheme<SelectStyle>,
    /// Menu defaults and recipes.
    pub menu: ComponentTheme<MenuStyle>,
    /// Button defaults and recipes.
    pub button: ComponentTheme<ButtonStyle>,
    /// Text field defaults and recipes.
    pub text_field: ComponentTheme<TextFieldStyle>,
}

impl ComponentThemes {
    /// Sets the slider defaults and recipes.
    pub fn slider(mut self, slider: ComponentTheme<SliderStyle>) -> Self {
        self.slider = slider;
        self
    }

    /// Sets the switch defaults and recipes.
    pub fn switch(mut self, switch: ComponentTheme<SwitchStyle>) -> Self {
        self.switch = switch;
        self
    }

    /// Sets the select defaults and recipes.
    pub fn select(mut self, select: ComponentTheme<SelectStyle>) -> Self {
        self.select = select;
        self
    }

    /// Sets the menu defaults and recipes.
    pub fn menu(mut self, menu: ComponentTheme<MenuStyle>) -> Self {
        self.menu = menu;
        self
    }

    /// Sets the button defaults and recipes.
    pub fn button(mut self, button: ComponentTheme<ButtonStyle>) -> Self {
        self.button = button;
        self
    }

    /// Sets the text field defaults and recipes.
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
    fn text_field_theme_can_configure_focus_ring_visibility() {
        let theme = ComponentTheme::new(TextFieldStyle::default().focus_ring(false))
            .recipe("ring", TextFieldStyle::default().focus_ring(true));
        assert_eq!(theme.resolve(&[]).focus_ring, Some(false));
        assert_eq!(theme.resolve(&["ring".into()]).focus_ring, Some(true));
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
