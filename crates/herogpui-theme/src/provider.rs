//! Global theme provider — the HeroGPUI equivalent of `HeroUIProvider`.

use std::collections::HashMap;

use gpui::{App, Global, SharedString};

use crate::layout::LayoutTheme;
use crate::semantic::{RoleColor, ThemeColors};
use crate::theme::Theme;
use herogpui_core::Color;

/// Holds the active theme, any registered custom themes, and the app-level
/// reduced-motion preference.
pub struct ThemeProvider {
    active: SharedString,
    themes: HashMap<SharedString, Theme>,
    reduce_motion: bool,
}

impl Global for ThemeProvider {}

impl ThemeProvider {
    /// Registers the provider with the default light theme.
    pub fn init(cx: &mut App) {
        Self::init_with(Theme::light(), cx);
    }

    /// Registers the provider starting from an explicit theme.
    pub fn init_with(theme: Theme, cx: &mut App) {
        let mut themes = HashMap::new();
        themes.insert("light".into(), Theme::light());
        themes.insert("dark".into(), Theme::dark());
        let id = theme.id.clone();
        themes.entry(id.clone()).or_insert(theme);
        // gpui does not surface the OS `prefers-reduced-motion` setting, so the
        // env var stands in for it; `set_reduce_motion` is the app-level
        // override, matching v3's `data-reduce-motion` precedence.
        let reduce_motion = std::env::var("HEROGPUI_REDUCE_MOTION")
            .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(false);
        cx.set_global(Self {
            active: id,
            themes,
            reduce_motion,
        });
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<ThemeProvider>()
    }

    pub fn theme(&self) -> &Theme {
        self.themes.get(&self.active).expect("active theme missing")
    }

    pub fn active_id(&self) -> &SharedString {
        &self.active
    }

    /// Registers a custom theme (does not activate it).
    pub fn register(&mut self, theme: Theme) {
        self.active = theme.id.clone();
        self.themes.insert(theme.id.clone(), theme);
    }

    /// Activates a previously registered theme by id.
    pub fn set_active(&mut self, id: impl Into<SharedString>) {
        self.active = id.into();
    }

    /// The app-level equivalent of v3's `data-reduce-motion` attribute.
    pub fn reduce_motion(&self) -> bool {
        self.reduce_motion
    }

    pub fn set_reduce_motion(&mut self, v: bool) {
        self.reduce_motion = v;
    }
}

/// Convenience extension trait giving every GPUI context access to the theme.
///
/// Works with `&App`, `&mut App`, `Context<T>` (they deref to `App`).
pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
    fn colors(&self) -> &ThemeColors;
    fn layout(&self) -> &LayoutTheme;
    fn role(&self, color: Color) -> &RoleColor;
    fn is_dark_theme(&self) -> bool;
    /// Whether animations should be suppressed. Components must check this
    /// before animating; v3 requires no opt-in from the caller.
    fn reduce_motion(&self) -> bool;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        ThemeProvider::get(self).theme()
    }

    fn colors(&self) -> &ThemeColors {
        &self.theme().colors
    }

    fn layout(&self) -> &LayoutTheme {
        &self.theme().layout
    }

    fn role(&self, color: Color) -> &RoleColor {
        match color {
            Color::Default => &self.colors().default,
            Color::Accent => &self.colors().accent,
            Color::Success => &self.colors().success,
            Color::Warning => &self.colors().warning,
            Color::Danger => &self.colors().danger,
        }
    }

    fn is_dark_theme(&self) -> bool {
        self.theme().is_dark()
    }

    fn reduce_motion(&self) -> bool {
        ThemeProvider::get(self).reduce_motion()
    }
}

/// Sets the global theme.
pub fn set_theme(theme: Theme, cx: &mut App) {
    let provider = cx.global_mut::<ThemeProvider>();
    provider.register(theme);
}

/// Activates one of the registered themes by id (`"light"`, `"dark"`, custom).
pub fn use_theme(id: impl Into<SharedString>, cx: &mut App) {
    let provider = cx.global_mut::<ThemeProvider>();
    provider.set_active(id);
}

/// Sets the app-level reduced-motion preference — the equivalent of putting
/// `data-reduce-motion="true"` on the document element. Every animated
/// component honours it without opt-in.
pub fn set_reduce_motion(v: bool, cx: &mut App) {
    cx.global_mut::<ThemeProvider>().set_reduce_motion(v);
}

/// Flips the reduced-motion preference.
pub fn toggle_reduce_motion(cx: &mut App) {
    let next = !ThemeProvider::get(cx).reduce_motion();
    set_reduce_motion(next, cx);
}

/// Switches between the light and dark defaults.
pub fn toggle_light_dark(cx: &mut App) {
    let dark = cx.theme().is_dark();
    let next = if dark { "light" } else { "dark" };
    use_theme(next, cx);
}
