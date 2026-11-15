//! Sparse, typed component defaults and reusable named recipes.
//!
//! Empty styles preserve each component's stock behavior. Recipes are resolved
//! against the active theme during render, not captured when a builder is made.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gpui::{Div, Hsla, Pixels, SharedString, StyleRefinement, Styled};

use crate::ThemeColors;
use herogpui_core::{Color, FieldVariant, Size, Variant};

/// Diagnostic report emitted when a component requests a recipe name that is
/// absent from [`ComponentTheme::recipes`] while a diagnostic handler is
/// installed via [`set_unknown_recipe_diagnostic_handler`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnknownRecipeDiagnostic {
    /// Component family name (for example `"button"` or `"select"`).
    pub component: SharedString,
    /// Requested recipe name that was not found.
    pub requested: SharedString,
    /// Sorted list of recipe names registered on the active theme for `component`.
    pub available: Vec<SharedString>,
}

/// Callback invoked for unrecognized recipe lookups when enabled.
pub type UnknownRecipeDiagnosticHandler = Arc<dyn Fn(&UnknownRecipeDiagnostic) + Send + Sync>;

static UNKNOWN_RECIPE_HANDLER: RwLock<Option<UnknownRecipeDiagnosticHandler>> = RwLock::new(None);

thread_local! {
    static SCOPED_UNKNOWN_RECIPE_HANDLER: RefCell<Option<UnknownRecipeDiagnosticHandler>> =
        const { RefCell::new(None) };
}

/// Installs or clears an opt-in diagnostic callback for unrecognized component
/// recipe names.
///
/// The default handler is `None`, keeping missing recipe lookups silent with
/// graceful fallback to theme defaults and stock styling. Applications can
/// install a logger or panic hook during development without altering release
/// behavior.
///
/// For thread-isolated checks (such as concurrent tests), prefer
/// [`with_unknown_recipe_diagnostic_handler`].
pub fn set_unknown_recipe_diagnostic_handler(handler: Option<UnknownRecipeDiagnosticHandler>) {
    if let Ok(mut slot) = UNKNOWN_RECIPE_HANDLER.write() {
        *slot = handler;
    }
}

/// Runs `f` with a thread-local diagnostic callback installed for unrecognized
/// component recipe names, restoring the previous thread-local handler even if
/// `f` unwinds.
///
/// While active on the calling thread, this handler takes precedence over the
/// process-global handler configured via [`set_unknown_recipe_diagnostic_handler`]
/// without affecting other threads.
pub fn with_unknown_recipe_diagnostic_handler<R>(
    handler: UnknownRecipeDiagnosticHandler,
    f: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<UnknownRecipeDiagnosticHandler>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let prev = self.0.take();
            SCOPED_UNKNOWN_RECIPE_HANDLER.with(|slot| {
                *slot.borrow_mut() = prev;
            });
        }
    }
    let previous = SCOPED_UNKNOWN_RECIPE_HANDLER.with(|slot| slot.borrow_mut().replace(handler));
    let _guard = Restore(previous);
    f()
}

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
#[must_use = "builder methods return a new value; the original is unchanged"]
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
    fn resolve_with(&self, names: &[SharedString], mut on_missing: impl FnMut(&SharedString)) -> T {
        let mut style = self.defaults.clone();
        for name in names {
            if let Some(recipe) = self.recipes.get(name) {
                style.refine(recipe);
            } else {
                on_missing(name);
            }
        }
        style
    }

    /// Resolve defaults, then named recipes in order. Missing names add no
    /// override, so switching to a stock theme restores stock presentation.
    pub fn resolve(&self, names: &[SharedString]) -> T {
        self.resolve_with(names, |_| {})
    }

    /// Like [`ComponentTheme::resolve`], while also notifying any handler
    /// registered with [`set_unknown_recipe_diagnostic_handler`] when a
    /// requested recipe name is absent from `self.recipes`.
    pub fn resolve_named(&self, component: &str, names: &[SharedString]) -> T {
        self.resolve_with(names, |name| {
            if let Some(handler) = SCOPED_UNKNOWN_RECIPE_HANDLER
                .with(|slot| slot.borrow().clone())
                .or_else(|| {
                    UNKNOWN_RECIPE_HANDLER
                        .read()
                        .ok()
                        .and_then(|slot| slot.clone())
                })
            {
                let mut available: Vec<SharedString> = self.recipes.keys().cloned().collect();
                available.sort_by(|a, b| a.as_str().cmp(b.as_str()));
                handler(&UnknownRecipeDiagnostic {
                    component: SharedString::from(component.to_owned()),
                    requested: name.clone(),
                    available,
                });
            }
        })
    }
}

macro_rules! component_style_setter {
    ($(#[$field_meta:meta])* $field:ident: @exact $ty:ty) => {
        $(#[$field_meta])*
        pub fn $field(mut self, value: $ty) -> Self {
            self.$field = Some(value);
            self
        }
    };
    ($(#[$field_meta:meta])* $field:ident: $ty:ty) => {
        $(#[$field_meta])*
        pub fn $field(mut self, value: impl Into<$ty>) -> Self {
            self.$field = Some(value.into());
            self
        }
    };
}

macro_rules! component_style {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $(@$tag:ident)? $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[must_use = "builder methods return a new value; the original is unchanged"]
        #[derive(Clone, Debug, Default)]
        #[non_exhaustive]
        pub struct $name {
            $($(#[$field_meta])* pub $field: Option<$ty>,)*
        }
        impl $name {
            $(
                component_style_setter!($(#[$field_meta])* $field: $(@$tag)? $ty);
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
        #[doc = "Spacing between the Select option rows."]
        panel_gap: Pixels,
        #[doc = "Minimum width of the detached option panel."]
        panel_min_width: Pixels,
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
component_style! {
    /// Tooltip surface presentation and default hover timing.
    TooltipStyle {
        /// Corner radius of the tooltip surface.
        radius: Pixels,
        /// Distance in pixels between the trigger and the tooltip surface.
        offset: Pixels,
        /// Hover delay in milliseconds before opening.
        delay_ms: @exact u64,
        /// Hover delay in milliseconds before closing.
        close_delay_ms: @exact u64,
        /// Uniform inner padding inside the tooltip panel.
        padding: Pixels,
        /// Text size inside the tooltip surface.
        text_size: Pixels,
        /// Surface background color.
        background: ComponentColor,
        /// Text foreground color.
        foreground: ComponentColor,
    }
}
component_style! {
    /// Popover dialog panel presentation.
    PopoverStyle {
        /// Corner radius of the popover panel.
        radius: Pixels,
        /// Uniform inner padding inside the popover panel.
        padding: Pixels,
        /// Distance in pixels between the trigger and the popover panel.
        offset: Pixels,
        /// Detached panel width.
        panel_width: Pixels,
        /// Panel background color.
        background: ComponentColor,
        /// Panel foreground color.
        foreground: ComponentColor,
    }
}
component_style! {
    /// Multi-line TextArea presentation defaults and recipes.
    TextAreaStyle {
        /// Field variant.
        variant: FieldVariant,
        /// Horizontal padding inside the multi-line box.
        padding_x: Pixels,
        /// Text and placeholder size.
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
component_style! {
    /// Tabs list container and indicator presentation.
    TabsStyle {
        /// Corner radius of the list container and sliding indicator.
        radius: Pixels,
        /// List container background color.
        list_bg: ComponentColor,
        /// Active indicator fill color.
        indicator_bg: ComponentColor,
        /// Whether the primary indicator renders its surface shadow.
        indicator_shadow: bool,
        /// Uniform padding inset inside the primary tab list tray.
        list_padding: Pixels,
        /// Whether unselected tabs render the hover wash overlay.
        hover_fill: bool,
        /// Label text size inside each tab.
        text_size: Pixels,
        /// Minimum width of each tab in a vertical tab list.
        vertical_tab_min_width: Pixels,
    }
}

/// Button recipes can combine semantic colors with a sparse GPUI root style.
/// Instance `sx` is refined over this style; instance builders retain precedence.
#[must_use = "builder methods return a new value; the original is unchanged"]
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
#[must_use = "builder methods return a new value; the original is unchanged"]
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
    /// Tooltip defaults and recipes.
    pub tooltip: ComponentTheme<TooltipStyle>,
    /// Popover defaults and recipes.
    pub popover: ComponentTheme<PopoverStyle>,
    /// TextArea defaults and recipes.
    pub textarea: ComponentTheme<TextAreaStyle>,
    /// Tabs defaults and recipes.
    pub tabs: ComponentTheme<TabsStyle>,
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

    /// Sets the tooltip defaults and recipes.
    pub fn tooltip(mut self, tooltip: ComponentTheme<TooltipStyle>) -> Self {
        self.tooltip = tooltip;
        self
    }

    /// Sets the popover defaults and recipes.
    pub fn popover(mut self, popover: ComponentTheme<PopoverStyle>) -> Self {
        self.popover = popover;
        self
    }

    /// Sets the textarea defaults and recipes.
    pub fn textarea(mut self, textarea: ComponentTheme<TextAreaStyle>) -> Self {
        self.textarea = textarea;
        self
    }

    /// Sets the tabs defaults and recipes.
    pub fn tabs(mut self, tabs: ComponentTheme<TabsStyle>) -> Self {
        self.tabs = tabs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThemeColors;
    use gpui::px;
    use herogpui_core::oklch;
    use std::sync::Mutex;

    static DIAGNOSTIC_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn empty_styles_resolve_to_stock_none_fields() {
        let themes = ComponentThemes::default();
        let slider = themes.slider.resolve(&[]);
        assert_eq!(slider.radius, None);
        let button = themes.button.resolve(&[]);
        assert_eq!(button.variant, None);
        assert!(button.style.is_none());
        assert_eq!(themes.tooltip.resolve(&[]).radius, None);
        assert_eq!(themes.popover.resolve(&[]).padding, None);
        assert_eq!(themes.textarea.resolve(&[]).padding_x, None);
        assert_eq!(themes.tabs.resolve(&[]).indicator_shadow, None);
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
    fn tooltip_popover_textarea_and_tabs_styles_refine_cleanly() {
        let themes = ComponentThemes::default()
            .tooltip(
                ComponentTheme::new(TooltipStyle::default().padding(px(6.)).delay_ms(200))
                    .recipe("compact", TooltipStyle::default().text_size(px(11.))),
            )
            .popover(
                ComponentTheme::new(
                    PopoverStyle::default()
                        .padding(px(12.))
                        .panel_width(px(220.)),
                )
                .recipe("flush", PopoverStyle::default().padding(px(4.))),
            )
            .textarea(
                ComponentTheme::new(TextAreaStyle::default().focus_ring(false))
                    .recipe("compact", TextAreaStyle::default().padding_x(px(8.))),
            )
            .tabs(
                ComponentTheme::new(TabsStyle::default().indicator_shadow(false))
                    .recipe("dense", TabsStyle::default().list_padding(px(0.))),
            );

        let tip = themes.tooltip.resolve(&["compact".into()]);
        assert_eq!(tip.padding, Some(px(6.)));
        assert_eq!(tip.delay_ms, Some(200));
        assert_eq!(tip.text_size, Some(px(11.)));

        let pop = themes.popover.resolve(&["flush".into()]);
        assert_eq!(pop.panel_width, Some(px(220.)));
        assert_eq!(pop.padding, Some(px(4.)));

        let ta = themes.textarea.resolve(&["compact".into()]);
        assert_eq!(ta.focus_ring, Some(false));
        assert_eq!(ta.padding_x, Some(px(8.)));

        let tabs = themes.tabs.resolve(&["dense".into()]);
        assert_eq!(tabs.indicator_shadow, Some(false));
        assert_eq!(tabs.list_padding, Some(px(0.)));
    }

    #[test]
    fn unknown_recipe_diagnostics_are_opt_in_and_silent_by_default() {
        let _guard = DIAGNOSTIC_TEST_LOCK.lock().unwrap();
        set_unknown_recipe_diagnostic_handler(None);

        let theme = ComponentTheme::new(TooltipStyle::default().padding(px(6.)))
            .recipe("zeta", TooltipStyle::default())
            .recipe("alpha", TooltipStyle::default());

        // Default handler is None: silent fallback still returns defaults.
        let resolved = theme.resolve_named("tooltip", &["missing".into()]);
        assert_eq!(resolved.padding, Some(px(6.)));

        let events = Arc::new(Mutex::new(Vec::<UnknownRecipeDiagnostic>::new()));
        let recorded = events.clone();
        set_unknown_recipe_diagnostic_handler(Some(Arc::new(move |diag| {
            recorded.lock().unwrap().push(diag.clone());
        })));

        let _ = theme.resolve_named("tooltip", &["alpha".into(), "typo".into()]);
        set_unknown_recipe_diagnostic_handler(None);

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].component, "tooltip");
        assert_eq!(captured[0].requested, "typo");
        assert_eq!(
            captured[0].available,
            vec![SharedString::from("alpha"), SharedString::from("zeta")]
        );
    }

    #[test]
    fn scoped_unknown_recipe_handler_is_thread_local_and_restores_on_unwind() {
        let _guard = DIAGNOSTIC_TEST_LOCK.lock().unwrap();
        set_unknown_recipe_diagnostic_handler(None);

        let theme =
            ComponentTheme::new(TooltipStyle::default()).recipe("known", TooltipStyle::default());
        let seen = Arc::new(Mutex::new(Vec::<SharedString>::new()));
        let sink = seen.clone();

        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            with_unknown_recipe_diagnostic_handler(
                Arc::new(move |diag| {
                    sink.lock().unwrap().push(diag.requested.clone());
                }),
                || {
                    let _ = theme.resolve_named("tooltip", &["inside-before-panic".into()]);
                    // Another thread must not inherit this thread-local handler.
                    std::thread::spawn({
                        let theme = theme.clone();
                        move || {
                            let _ = theme.resolve_named("tooltip", &["other-thread".into()]);
                        }
                    })
                    .join()
                    .unwrap();
                    panic!("simulated unwind");
                },
            );
        }));

        // After unwinding, the scoped handler is restored and no longer active.
        let _ = theme.resolve_named("tooltip", &["after-unwind".into()]);
        assert_eq!(
            seen.lock().unwrap().as_slice(),
            [SharedString::from("inside-before-panic")]
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
