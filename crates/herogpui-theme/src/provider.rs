//! Global theme provider — the HeroGPUI equivalent of `HeroUIProvider`.

use std::collections::HashMap;

use gpui::{AnyWindowHandle, App, Global, SharedString, Subscription, Window, WindowAppearance};

use crate::layout::LayoutTheme;
use crate::semantic::{RoleColor, ThemeColors};
use crate::theme::{Appearance, Theme};
use herogpui_core::Color;

/// Holds the active theme, any registered custom themes, and the opt-in
/// "follow the OS appearance" mode.
///
/// The reduced-motion preference is deliberately *not* a field here. GPUI owns
/// that flag (`App::reduce_motion`) and its own `Animation`/spring elements read
/// it directly, so a copy on this global would be a second source of truth: a
/// consumer app writing a plain `gpui::Animation`, or any GPUI internal such as
/// animated-image playback, would keep animating after HeroGPUI was told to
/// stop. `ActiveTheme::reduce_motion` therefore reads GPUI's flag and
/// [`set_reduce_motion`] writes it, so the two cannot disagree.
pub struct ThemeProvider {
    active: SharedString,
    themes: HashMap<SharedString, Theme>,
    /// Set only by [`follow_system_appearance`]; nothing else may turn it on.
    /// Following the OS is opt-in because an app that pins `Appearance::Light`
    /// through `init_with`/`use_theme` means it, and must keep it.
    follow_system_appearance: bool,
    /// One retained appearance observer per followed window, keyed so a root
    /// view that re-registers on rebuild replaces its observer instead of
    /// stacking a second one. Retention is the point: `Subscription` is
    /// unsubscribe-on-drop, so a discarded handle syncs once and then goes
    /// quiet forever — the classic failure here.
    appearance_observers: HashMap<AnyWindowHandle, Subscription>,
}

impl Global for ThemeProvider {}

impl ThemeProvider {
    /// Registers the provider with the default light theme.
    ///
    /// Call this once before opening the first window. Rendering a themed
    /// component before initialization panics.
    pub fn init(cx: &mut App) {
        Self::init_with(Theme::light(), cx);
    }

    /// Registers the provider starting from an explicit theme.
    ///
    /// Call this once before opening the first window. Rendering a themed
    /// component before initialization panics.
    pub fn init_with(theme: Theme, cx: &mut App) {
        let mut themes = HashMap::new();
        themes.insert("light".into(), Theme::light());
        themes.insert("dark".into(), Theme::dark());
        let id = theme.id.clone();
        themes.insert(id.clone(), theme);
        // gpui does not surface the OS `prefers-reduced-motion` setting, so the
        // env var stands in for it; `set_reduce_motion` is the app-level
        // override, matching v3's `data-reduce-motion` precedence.
        let reduce_motion = std::env::var("HEROGPUI_REDUCE_MOTION")
            .is_ok_and(|v| v != "0" && !v.eq_ignore_ascii_case("false"));
        cx.set_global(Self {
            active: id,
            themes,
            follow_system_appearance: false,
            appearance_observers: HashMap::new(),
        });
        // The global has to exist first: GPUI's setter refreshes every window
        // when the value changes, and a themed window repainting before
        // `set_global` would panic looking for the provider. GPUI's own default
        // is a plain `false` seeded at app construction (it reads no OS
        // setting), so writing the env-var seed here clobbers nothing.
        cx.set_reduce_motion(reduce_motion);
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

    /// Registers and activates a custom theme.
    pub fn register(&mut self, theme: Theme) {
        self.active = theme.id.clone();
        self.themes.insert(theme.id.clone(), theme);
    }

    /// Activates a previously registered theme by id.
    pub fn set_active(&mut self, id: impl Into<SharedString>) {
        self.active = id.into();
    }

    /// Whether the OS light/dark appearance is being followed — the state a
    /// three-way "Light / Dark / System" setting needs to render itself.
    pub fn follows_system_appearance(&self) -> bool {
        self.follow_system_appearance
    }
}

/// Convenience extension trait giving every GPUI context access to the theme.
///
/// Works with `&App`, `&mut App`, `Context<T>` (they deref to `App`).
pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
    fn colors(&self) -> &ThemeColors;
    fn layout(&self) -> &LayoutTheme;
    fn components(&self) -> &crate::ComponentThemes;
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

    fn components(&self) -> &crate::ComponentThemes {
        &self.theme().components
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
        // GPUI's flag is the single source of truth (see `ThemeProvider`), and
        // `App::reduce_motion` is spelled out rather than called as
        // `self.reduce_motion()` because that would resolve to this very trait
        // method through the inherent-first rule only by luck — inherent wins,
        // so the shorthand happens to work and reads like infinite recursion.
        App::reduce_motion(self)
    }
}

/// Sets the global theme and schedules every open window to repaint.
pub fn set_theme(theme: Theme, cx: &mut App) {
    let provider = cx.global_mut::<ThemeProvider>();
    provider.register(theme);
    cx.refresh_windows();
}

/// Activates one of the registered themes by id (`"light"`, `"dark"`, custom)
/// and schedules every open window to repaint.
pub fn use_theme(id: impl Into<SharedString>, cx: &mut App) {
    let provider = cx.global_mut::<ThemeProvider>();
    provider.set_active(id);
    cx.refresh_windows();
}

/// Sets the app-level reduced-motion preference — the equivalent of putting
/// `data-reduce-motion="true"` on the document element — and schedules every
/// open window to repaint. Every animated component honours it without opt-in,
/// and so does every plain `gpui::Animation`: the value is stored in GPUI's own
/// global, which its animation elements consult themselves.
pub fn set_reduce_motion(v: bool, cx: &mut App) {
    cx.set_reduce_motion(v);
    // GPUI's setter already refreshes on a *change*; this repaints on a
    // no-change write too, which is the contract `theme_repaint.rs` pins for
    // every provider mutation.
    cx.refresh_windows();
}

/// Flips the reduced-motion preference.
pub fn toggle_reduce_motion(cx: &mut App) {
    let next = !App::reduce_motion(cx);
    set_reduce_motion(next, cx);
}

/// Follows the OS light/dark appearance for as long as `window` stays open,
/// activating the registered `"light"` or `"dark"` theme to match.
///
/// Opt-in on purpose: without this call the app keeps whatever theme it
/// activated, so pinning a single appearance stays possible. Call it once per
/// window, from the window's root-view constructor. A custom theme pair can
/// join in by registering under the ids `"light"` and `"dark"`, which is what
/// the switch reads.
///
/// The current appearance is applied immediately, so a window opened on a dark
/// desktop does not paint one light frame first.
///
/// `App::should_auto_hide_scrollbars` is read by the painted `Scrollbar`
/// overlay: thumbs hide after idle when the OS preference is on, and stay
/// painted when it is off.
pub fn follow_system_appearance(window: &mut Window, cx: &mut App) {
    // The per-window `appearance()` over `App::window_appearance()`: it is the
    // value the observer below reports, so the immediate sync and every later
    // sync agree, and the app-level platform query is the one that misbehaves
    // on Linux (longbridge/gpui-kit#104).
    let subscription = window.observe_window_appearance(|window, cx| {
        apply_system_appearance(window.appearance(), cx);
    });
    let handle = window.window_handle();
    let appearance = window.appearance();

    let provider = cx.global_mut::<ThemeProvider>();
    provider.follow_system_appearance = true;
    // Moving the `Subscription` into the global is what keeps the observer
    // alive; binding it to a local here would unsubscribe at the end of this
    // function and the OS switch would silently stop arriving.
    provider.appearance_observers.insert(handle, subscription);

    apply_system_appearance(appearance, cx);
}

/// Stops following the OS appearance, leaving the active theme as it is.
pub fn stop_following_system_appearance(cx: &mut App) {
    let provider = cx.global_mut::<ThemeProvider>();
    provider.follow_system_appearance = false;
    // Dropping the subscriptions is the unsubscribe. Clearing only the flag
    // would leave live observers calling back into a mode that is off.
    provider.appearance_observers.clear();
}

/// Activates the theme matching one OS appearance, if the mode is still on.
fn apply_system_appearance(appearance: WindowAppearance, cx: &mut App) {
    let provider = ThemeProvider::get(cx);
    if !provider.follows_system_appearance() {
        return;
    }
    let id = match Appearance::from(appearance) {
        Appearance::Light => "light",
        Appearance::Dark => "dark",
    };
    // A window reports its appearance on unrelated occasions too (and every
    // followed window reports the same OS switch), so skip the no-op rather
    // than repainting every window once per redundant notification.
    if provider.active_id().as_ref() == id {
        return;
    }
    use_theme(id, cx);
}

/// Switches between the light and dark defaults.
pub fn toggle_light_dark(cx: &mut App) {
    let dark = cx.theme().is_dark();
    let next = if dark { "light" } else { "dark" };
    use_theme(next, cx);
}
