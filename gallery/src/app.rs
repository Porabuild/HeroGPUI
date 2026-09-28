//! The gallery application shell: navbar, sidebar, content router and all
//! interactive demo state.

use std::collections::{HashMap, HashSet};

use gpui::{
    prelude::*, px, App, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_components as h;
use herogpui_theme::{
    presets, toggle_light_dark, toggle_reduce_motion, use_theme, ActiveTheme, ThemeProvider,
};

use crate::pages::{nav_sections, Page};

pub const FONT_FAMILY: &str = if cfg!(target_arch = "wasm32") {
    // The web shell bundles Inter Variable and JetBrains Mono; the system
    // families below do not exist in the browser and panic text resolution.
    "Inter Variable"
} else if cfg!(target_os = "macos") {
    "Helvetica Neue"
} else if cfg!(target_os = "linux") {
    "Ubuntu"
} else {
    "Segoe UI"
};

pub const MONO_FONT: &str = if cfg!(target_arch = "wasm32") {
    // The web shell bundles JetBrains Mono; the literal family "monospace"
    // resolves to nothing there and panics text resolution.
    "JetBrains Mono"
} else if cfg!(target_os = "macos") {
    "Menlo"
} else {
    "Consolas"
};

/// How many demo toasts have closed.
///
/// A toast outlives the button that pushed it, and its `onClose` runs from the
/// dismissal with no view in hand, so the count cannot live in the `Gallery`
/// entity's demo state. It lives in a global, and closing a toast refreshes the
/// windows that show it.
struct ToastsClosed(usize);
impl gpui::Global for ToastsClosed {}

pub fn bump_toast_closed(cx: &mut App) {
    let next = cx.try_global::<ToastsClosed>().map_or(0, |c| c.0) + 1;
    cx.set_global(ToastsClosed(next));
    cx.refresh_windows();
}

pub fn toasts_closed(cx: &App) -> usize {
    cx.try_global::<ToastsClosed>().map_or(0, |c| c.0)
}

pub(crate) fn reset_toast_closed(cx: &mut App) {
    cx.set_global(ToastsClosed(0));
}

/// Root view of the gallery window.
pub struct Gallery {
    pub(crate) page: Page,
    pub(crate) control_frame: Option<crate::control::FrameAck>,
    pub(crate) toast_promises: Vec<gpui::Task<()>>,

    // -- demo state ---------------------------------------------------------
    pub button_clicks: u32,
    pub button_upload_pending: bool,
    pub switch_a: bool,
    pub switch_b: bool,
    pub cb_basic: bool,
    pub cb_color: bool,
    pub cb_select_all_selected: bool,
    pub cb_select_all_indeterminate: bool,
    pub radio_sel: Option<usize>,
    pub radio_plan: SharedString,
    pub slider_value: f32,
    pub slider_range: Vec<f32>,
    pub tab_underline: SharedString,
    pub tab_solid: SharedString,
    pub accordion_open: HashSet<SharedString>,
    pub modal_open: bool,
    pub dropdown_open: bool,
    pub dropdown_selected: Option<SharedString>,
    /// Last item chosen in the ContextMenu extension demo.
    pub context_menu_last: SharedString,
    /// Last menu and item chosen in the MenuBar extension demo.
    pub menu_bar_last: SharedString,
    /// Last sizes reported by the ResizablePanelGroup extension demo.
    pub resizable_sizes: SharedString,
    /// Selection of the TreeView extension demo.
    pub tree_selected: SharedString,
    /// Whether the CommandPalette extension demo is open (Cmd/Ctrl-K).
    pub command_palette_open: bool,
    /// Last command the CommandPalette extension demo ran.
    pub command_palette_last: SharedString,
    /// Active item of the Sidebar extension demo.
    pub sidebar_active: SharedString,
    /// Whether the Sidebar extension demo is collapsed to icons.
    pub sidebar_collapsed: bool,
    /// Last window action the TitleBar extension demo reported.
    pub title_bar_last: SharedString,
    /// Scroll state of the VirtualList extension demo.
    pub virtual_list: h::VirtualListHandle,
    pub dropdown_last_basic: SharedString,
    pub dropdown_doc_marks: Vec<SharedString>,
    pub lb_pair_selection: HashSet<SharedString>,
    pub select_lang_controlled: Option<SharedString>,
    pub pagination_page: usize,
    pub alert_visible: bool,
    pub input_submitted: String,

    pub input_name: Entity<h::InputState>,
    pub input_email: Entity<h::InputState>,
    pub input_bio: Entity<h::InputState>,

    // -- parity batch state --------------------------------------------------
    pub select_lang: Option<SharedString>,
    pub select_multi: Vec<SharedString>,
    pub select_open: bool,
    pub ac_entity: Entity<h::InputState>,
    pub drawer_open: bool,
    pub otp: Entity<h::OtpState>,
    pub otp_done: String,
    /// InputOTP "Controlled": every keystroke, not just completion.
    pub otp_typed: String,
    pub number: Entity<h::NumberState>,
    pub price: Entity<h::NumberState>,
    pub cal_picked: Option<h::Date>,
    pub calendar_focus: h::Date,
    pub range_calendar_focus: h::Date,
    pub date_picker_open: bool,
    pub range_open: bool,
    pub date_input: Entity<h::InputState>,
    pub date_iso: Option<h::Date>,
    /// Kept on the view so the store outlives registration and for tests.
    #[allow(dead_code)]
    pub toasts: Entity<h::ToastStore>,
    pub popover_open: bool,
    pub disclosure_expanded: bool,
    pub disclosure_group_expanded: HashSet<SharedString>,
    /// ToggleButton "Controlled": v3's own like/unlike demo.
    pub toggle_like: bool,
    pub toggle_single: Option<SharedString>,
    pub toggle_multiple: HashSet<SharedString>,
    pub meter_value: f32,

    // -- v3 additions --------------------------------------------------------
    pub close_button_presses: u32,
    pub list_selection: HashSet<SharedString>,
    /// Remaining tag keys, so the remove demo can actually remove.
    pub tags: Vec<SharedString>,
    pub tag_selection: HashSet<SharedString>,
    pub checkbox_group: HashSet<SharedString>,
    pub alert_dialog_open: bool,
    pub picker_color: h::PickerColor,
    pub swatch_selected: h::PickerColor,
    pub search_state: Entity<h::InputState>,
    pub search_query: String,
    pub group_amount: Entity<h::InputState>,
    pub cal_year_picker: bool,
    pub table_selection: Vec<SharedString>,
    pub table_sort: Option<h::SortDescriptor>,
    pub dropdown_multi: Vec<SharedString>,

    // -- per-demo state -----------------------------------------------------
    //
    // v3's examples each own their state: its "Controlled" demo and its
    // "Disabled State" demo are separate fields. Sharing one entity across a
    // page would make typing in one demo change every other, so these are
    // keyed by demo id and created on first render -- a page only pays for the
    // demos it actually shows.
    pub demo_text: HashMap<&'static str, Entity<h::InputState>>,
    pub demo_number: HashMap<&'static str, Entity<h::NumberState>>,
    pub demo_time: HashMap<&'static str, Entity<h::TimeState>>,
    pub demo_otp: HashMap<&'static str, Entity<h::OtpState>>,
    pub demo_calendar: HashMap<&'static str, Entity<h::CalendarState>>,
    pub demo_range: HashMap<&'static str, Entity<h::DateRangeState>>,
    pub demo_flags: HashMap<&'static str, bool>,
    /// `HEROGPUI_OPEN_OVERLAYS=1`: every overlay demo starts open, so a smoke
    /// run and a screenshot both see the panel rather than just its trigger.
    pub overlays_open: bool,
    /// Which corner the shell's `ToastViewport` sits in -- the Toast page's
    /// "Placements" demo sets it, which is the only way one viewport can show
    /// what `placement` does.
    pub toast_placement: h::ToastPlacement,
    pub demo_values: HashMap<&'static str, f32>,
    pub demo_strings: HashMap<&'static str, String>,
    pub demo_selections: HashMap<&'static str, Vec<SharedString>>,
}

impl Gallery {
    /// The gallery-wide CommandPalette extension demo: every page, grouped by
    /// its navigation section, plus the two appearance toggles. Rendered at the
    /// shell's root so Cmd-K (Ctrl-K off macOS) opens it from any page.
    fn command_palette(&self, cx: &mut Context<'_, Self>) -> h::CommandPalette {
        let mut pages: Vec<(SharedString, Page)> = Vec::new();
        let mut items = Vec::new();
        for section in nav_sections() {
            for page in section.items {
                let key = SharedString::from(format!("page:{page:?}"));
                if pages.iter().any(|(known, _)| *known == key) {
                    continue;
                }
                items.push(
                    h::CommandItem::new(key.clone(), page.title())
                        .group(section.title)
                        .icon(h::IconName::File),
                );
                pages.push((key, page));
            }
        }
        items.push(
            h::CommandItem::new("toggle-theme", "Toggle dark mode")
                .group("Preferences")
                .icon(h::IconName::Moon)
                .keywords(["theme", "light", "dark", "appearance"]),
        );
        items.push(
            h::CommandItem::new("toggle-motion", "Toggle reduced motion")
                .group("Preferences")
                .keywords(["animation", "accessibility"]),
        );
        h::CommandPalette::new("gallery-command-palette", items)
            .placeholder("Search pages and commands")
            .is_open(self.command_palette_open)
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.command_palette_open = *open;
                cx.notify();
            }))
            .on_select(cx.listener(move |this, key: &SharedString, _, cx| {
                match key.as_ref() {
                    "toggle-theme" => toggle_light_dark(cx),
                    "toggle-motion" => toggle_reduce_motion(cx),
                    _ => {
                        if let Some((_, page)) = pages.iter().find(|(known, _)| known == key) {
                            this.page = *page;
                        }
                    }
                }
                this.command_palette_last = key.clone();
                cx.notify();
            }))
    }

    /// Cmd-K (Ctrl-K off macOS) toggles the CommandPalette demo from anywhere
    /// in the shell.
    fn toggle_command_palette(
        &mut self,
        event: &gpui::KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        if h::is_command_palette_shortcut(&event.keystroke) {
            self.command_palette_open = !self.command_palette_open;
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn toast_viewport(&self) -> h::ToastViewport {
        h::ToastViewport::new()
            .placement(self.toast_placement)
            .is_expanded(self.demo_flag("toast-expanded", false))
    }

    pub(crate) fn track_toast_promise(&mut self, task: gpui::Task<()>) {
        self.toast_promises.retain(|task| !task.is_ready());
        self.toast_promises.push(task);
    }

    /// The text state for one demo, created on first use.
    ///
    /// `initial` seeds it the way v3's `defaultValue` does, and only on the
    /// first call -- later renders return the state the user has been editing.
    pub fn demo_text(
        &mut self,
        key: &'static str,
        initial: &str,
        cx: &mut App,
    ) -> Entity<h::InputState> {
        if let Some(state) = self.demo_text.get(key) {
            return state.clone();
        }
        let initial = initial.to_owned();
        let state = cx.new(|cx| h::InputState::with_value(cx, initial));
        self.demo_text.insert(key, state.clone());
        state
    }

    /// The numeric state for one demo, created on first use.
    pub fn demo_number(
        &mut self,
        key: &'static str,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
        cx: &mut App,
    ) -> Entity<h::NumberState> {
        if let Some(state) = self.demo_number.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| {
            let mut n = h::NumberState::new(cx, value);
            n.set_range(min, max);
            n.set_step(step);
            n
        });
        self.demo_number.insert(key, state.clone());
        state
    }

    /// The time state for one demo, created on first use.
    pub fn demo_time(&mut self, key: &'static str, cx: &mut App) -> Entity<h::TimeState> {
        if let Some(state) = self.demo_time.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| h::TimeState::with_value(cx, h::Time::new(9, 30)));
        self.demo_time.insert(key, state.clone());
        state
    }

    /// The one-time-code state for one demo, created on first use.
    pub fn demo_otp(
        &mut self,
        key: &'static str,
        length: usize,
        cx: &mut App,
    ) -> Entity<h::OtpState> {
        if let Some(state) = self.demo_otp.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| h::OtpState::with_length(cx, length));
        self.demo_otp.insert(key, state.clone());
        state
    }

    /// The calendar state for one demo, created on first use.
    pub fn demo_calendar(&mut self, key: &'static str, cx: &mut App) -> Entity<h::CalendarState> {
        if let Some(state) = self.demo_calendar.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| h::CalendarState::new(cx));
        self.demo_calendar.insert(key, state.clone());
        state
    }

    /// The date-range state for one demo, created on first use.
    pub fn demo_range(&mut self, key: &'static str, cx: &mut App) -> Entity<h::DateRangeState> {
        if let Some(state) = self.demo_range.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| h::DateRangeState::new(cx));
        self.demo_range.insert(key, state.clone());
        state
    }

    /// A boolean a demo owns (selected, open, checked).
    pub fn demo_flag(&self, key: &str, default: bool) -> bool {
        self.demo_flags.get(key).copied().unwrap_or(default)
    }

    pub fn set_demo_flag(&mut self, key: &'static str, v: bool) {
        self.demo_flags.insert(key, v);
    }

    /// Whether one overlay demo is open. Defaults to `HEROGPUI_OPEN_OVERLAYS`
    /// so a capture run shows the panel, not just the button that opens it.
    pub fn demo_overlay(&self, key: &str) -> bool {
        self.demo_flags
            .get(key)
            .copied()
            .unwrap_or(self.overlays_open)
    }

    /// A plain string a demo owns (the last picked key, a status line).
    pub fn demo_text_value(&self, key: &str) -> String {
        self.demo_strings.get(key).cloned().unwrap_or_default()
    }

    pub fn set_demo_text_value(&mut self, key: &'static str, value: String) {
        self.demo_strings.insert(key, value);
    }

    /// A multi-selection a demo owns.
    pub fn demo_selection(&self, key: &str) -> Vec<SharedString> {
        self.demo_selections.get(key).cloned().unwrap_or_default()
    }

    pub fn set_demo_selection(&mut self, key: &'static str, value: Vec<SharedString>) {
        self.demo_selections.insert(key, value);
    }

    /// A numeric value a demo owns (slider, progress).
    pub fn demo_value(&self, key: &str, default: f32) -> f32 {
        self.demo_values.get(key).copied().unwrap_or(default)
    }

    pub fn set_demo_value(&mut self, key: &'static str, v: f32) {
        self.demo_values.insert(key, v);
    }

    pub fn new(cx: &mut Context<'_, Self>) -> Self {
        register_theme_presets(cx);
        let name = cx.new(|cx| h::InputState::new(cx));
        let email = cx.new(|cx| h::InputState::new(cx));
        // Seeded with newlines so the multi-line surface is visible at rest.
        let bio = cx.new(|cx| {
            h::InputState::with_value(
                cx,
                "Built with HeroGPUI.
Enter inserts a newline here, and a long paragraph wraps inside the field instead of running off the edge.",
            )
        });
        let ac = cx.new(|cx| h::InputState::new(cx));
        let date_input = cx.new(|cx| h::InputState::new(cx));
        let otp = cx.new(|cx| h::OtpState::with_length(cx, 6));
        let number = cx.new(|cx| {
            let mut n = h::NumberState::new(cx, 5.0);
            n.set_range(0.0, 20.0);
            n.set_step(1.0);
            n
        });
        let price = cx.new(|cx| {
            let mut n = h::NumberState::new(cx, 1200.0);
            n.set_range(0.0, 100_000.0);
            n.set_step(50.0);
            n
        });
        let search_state = cx.new(|cx| h::InputState::new(cx));
        let group_amount = cx.new(|cx| h::InputState::new(cx));

        // Re-render the shell whenever toasts change.
        let toasts = h::toast_store(cx);
        cx.observe(&toasts, |_, _, cx| cx.notify()).detach();

        let mut accordion_open = HashSet::new();
        accordion_open.insert(SharedString::from("1"));
        let mut disclosure_group_expanded = HashSet::new();
        disclosure_group_expanded.insert(SharedString::from("returns"));
        let mut toggle_multiple = HashSet::new();
        toggle_multiple.insert(SharedString::from("bold"));
        toggle_multiple.insert(SharedString::from("underline"));

        Self {
            page: Page::Introduction,
            control_frame: None,
            toast_promises: Vec::new(),
            button_clicks: 0,
            button_upload_pending: false,
            switch_a: true,
            switch_b: false,
            cb_basic: true,
            cb_color: false,
            cb_select_all_selected: false,
            cb_select_all_indeterminate: true,
            radio_sel: Some(0),
            radio_plan: SharedString::from("Pro"),
            slider_value: 40.0,
            slider_range: vec![20.0, 70.0],
            tab_underline: "home".into(),
            tab_solid: "music".into(),
            accordion_open,
            modal_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            dropdown_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            dropdown_selected: None,
            context_menu_last: SharedString::from("none yet"),
            menu_bar_last: SharedString::from("none yet"),
            resizable_sizes: SharedString::from("30, 70"),
            tree_selected: SharedString::from("nothing"),
            command_palette_open: false,
            command_palette_last: SharedString::from("none yet"),
            sidebar_active: SharedString::from("home"),
            sidebar_collapsed: false,
            title_bar_last: SharedString::from("none yet"),
            virtual_list: h::VirtualListHandle::new(1000),
            dropdown_last_basic: SharedString::from("none yet"),
            dropdown_doc_marks: vec![SharedString::from("bold")],
            lb_pair_selection: {
                let mut keys = HashSet::new();
                keys.insert(SharedString::from("opt-1"));
                keys
            },
            select_lang_controlled: None,
            pagination_page: 1,
            alert_visible: true,
            input_submitted: String::new(),
            input_name: name,
            input_email: email,
            input_bio: bio,
            select_lang: None,
            select_multi: vec![SharedString::from("Rust")],
            select_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            ac_entity: ac,
            drawer_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            otp,
            otp_done: String::new(),
            otp_typed: String::new(),
            number,
            price,
            cal_picked: None,
            calendar_focus: h::Date::today(),
            range_calendar_focus: h::Date::today(),
            date_picker_open: false,
            range_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            date_input,
            date_iso: None,
            toasts,
            popover_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            disclosure_expanded: true,
            disclosure_group_expanded,
            toggle_like: false,
            toggle_single: Some(SharedString::from("center")),
            toggle_multiple,
            meter_value: 60.0,

            close_button_presses: 0,
            list_selection: HashSet::from([SharedString::from("inbox")]),
            tags: ["design", "engineering", "product", "research"]
                .into_iter()
                .map(SharedString::from)
                .collect(),
            tag_selection: HashSet::new(),
            checkbox_group: HashSet::from([SharedString::from("email")]),
            alert_dialog_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            picker_color: h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
            swatch_selected: h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
            search_state,
            search_query: String::new(),
            group_amount,
            cal_year_picker: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            table_selection: Vec::new(),
            table_sort: None,
            dropdown_multi: Vec::new(),
            demo_text: HashMap::new(),
            demo_number: HashMap::new(),
            demo_time: HashMap::new(),
            demo_otp: HashMap::new(),
            demo_calendar: HashMap::new(),
            demo_range: HashMap::new(),
            demo_flags: HashMap::new(),
            overlays_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            // v3's `ToastProvider` defaults `placement` to `"bottom"`, so the
            // shell starts there and the Toast page's Placements demo moves it.
            toast_placement: h::ToastPlacement::default(),
            demo_values: HashMap::new(),
            demo_strings: HashMap::new(),
            demo_selections: HashMap::new(),
        }
    }
}

/// The ids the navbar theme picker offers, in display order: v3's two base
/// themes, then every built-in preset from `herogpui_theme::presets`.
pub fn theme_choices() -> Vec<SharedString> {
    ["light", "dark"]
        .into_iter()
        .chain(presets::PRESETS.iter().map(|(id, _)| *id))
        .map(SharedString::new_static)
        .collect()
}

/// The picker label for a theme id: the id with its first letter capitalised.
fn theme_label(id: &str) -> SharedString {
    let mut chars = id.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect::<String>())
        .unwrap_or_default()
        .into()
}

/// Registers the built-in presets the theme picker switches between, without
/// activating one. Idempotent: a `reset=1` control request builds a fresh
/// `Gallery` and must not re-insert the presets (re-inserting the active one
/// would swap its tokens in place under a live window).
pub fn register_theme_presets(cx: &mut App) {
    let registered = ThemeProvider::get(cx).theme_ids();
    if presets::PRESETS
        .iter()
        .all(|(id, _)| registered.iter().any(|known| known == id))
    {
        return;
    }
    presets::register_presets(cx);
}

/// Activates the theme the picker chose. An id the provider does not know is
/// ignored, leaving the current theme in place.
pub fn select_theme(id: &str, cx: &mut App) {
    let _ = use_theme(SharedString::from(id.to_owned()), cx);
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        if let Some(completion) = self.control_frame.take() {
            completion.rendered(_window, cx);
        }
        let colors = cx.colors().clone();

        if crate::control::preview_only(cx) {
            return h::extend::app_focus_root(gpui::div(), _window, cx)
                .size_full()
                .bg(colors.background)
                .text_color(colors.foreground)
                .font_family(FONT_FAMILY)
                .text_size(px(14.))
                .line_height(px(20.))
                .relative()
                .on_key_down(cx.listener(Self::toggle_command_palette))
                .child(self.render_current_page(cx))
                .child(self.command_palette(cx))
                .child(self.toast_viewport());
        }

        // ---- top navbar ----------------------------------------------------
        let is_dark = cx.is_dark_theme();
        let theme_button = h::Button::new("theme-toggle")
            .variant(h::Variant::Tertiary)
            .is_icon_only(true)
            .on_press(cx.listener(|_, _, _, cx| {
                toggle_light_dark(cx);
                cx.notify();
            }))
            .child(
                gpui::svg()
                    .size(px(16.))
                    .path(if is_dark {
                        h::icons::SUN
                    } else {
                        h::icons::MOON
                    })
                    .text_color(colors.foreground),
            );

        // Every registered base theme and preset, bound to the active id so
        // the light/dark toggle, `HEROGPUI_THEME` and a control request's
        // `theme=` all show up here too.
        let active_theme = cx.theme().id.clone();
        let theme_picker = gpui::div()
            .flex()
            .items_center()
            .gap(px(6.))
            .child(h::Icon::new(h::IconName::Palette).color(colors.muted))
            .child(
                gpui::div()
                    .w(px(118.))
                    .debug_selector(|| "theme-picker".into())
                    .child(
                        h::Select::new(
                            "theme-picker",
                            theme_choices()
                                .into_iter()
                                .map(|id| h::PickerItem::new(id.clone(), theme_label(&id)))
                                .collect(),
                        )
                        .value(Some(active_theme))
                        .full_width(true)
                        .height(px(32.))
                        .trigger_text_size(px(13.))
                        .on_change(cx.listener(
                            |_, key: &Option<SharedString>, _, cx| {
                                if let Some(id) = key {
                                    select_theme(id, cx);
                                }
                                cx.notify();
                            },
                        )),
                    ),
            );

        // v3 exposes reduced motion as an app-level switch that every animated
        // component honours without opt-in.
        let reduce_motion = ActiveTheme::reduce_motion(&**cx);
        let motion_button = h::Button::new("motion-toggle")
            .variant(if reduce_motion {
                h::Variant::Secondary
            } else {
                h::Variant::Tertiary
            })
            .size(h::Size::Sm)
            .label(if reduce_motion {
                "Motion off"
            } else {
                "Motion on"
            })
            .on_press(cx.listener(|_, _, _, cx| {
                toggle_reduce_motion(cx);
                cx.notify();
            }));

        let github_link = h::Link::new("gh-link")
            .label("GitHub")
            .href("https://github.com/Porabuild/HeroGPUI");

        let navbar_top = gpui::div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(60.))
            .px(px(20.))
            .bg(colors.background)
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        gpui::div()
                            .size(px(26.))
                            .rounded(px(7.))
                            .bg(colors.accent.color)
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(colors.accent.foreground)
                            .text_size(px(15.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("H"),
                    )
                    .child(
                        gpui::div()
                            .text_size(px(17.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("HeroGPUI"),
                    )
                    .child(
                        gpui::div()
                            .px(px(6.))
                            .py(px(2.))
                            .rounded_full()
                            .bg(colors.accent.soft())
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(colors.accent.color)
                            .child(concat!("v", env!("CARGO_PKG_VERSION"))),
                    ),
            )
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(github_link)
                    .child(motion_button)
                    .child(theme_picker)
                    .child(theme_button),
            );

        let active_root = self.page.docs_root();
        let mut docs_tabs = gpui::div()
            .h(px(38.))
            .px(px(20.))
            .flex()
            .items_center()
            .gap(px(18.));
        for (label, target) in [
            ("Getting Started", Page::Introduction),
            ("Components", Page::AllComponents),
            ("Releases", Page::Releases),
        ] {
            let active = active_root == target;
            let mut tab = gpui::div()
                .id(gpui::ElementId::Name(format!("docs-tab-{target:?}").into()))
                .h_full()
                .px(px(4.))
                .border_b_2()
                .border_color(if active {
                    colors.accent.color
                } else {
                    gpui::transparent_black()
                })
                .flex()
                .items_center()
                .text_size(px(13.))
                .font_weight(if active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::NORMAL
                })
                .text_color(if active {
                    colors.foreground
                } else {
                    colors.muted
                })
                .cursor_pointer()
                .tab_index(0)
                .focus(move |style| style.bg(colors.default.soft()));
            if !active {
                tab = tab.hover(move |style| style.text_color(colors.foreground));
            }
            docs_tabs = docs_tabs.child(
                tab.child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_initial_page(target);
                        cx.notify();
                    }))
                    .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            this.set_initial_page(target);
                            cx.notify();
                        }
                    })),
            );
        }

        let navbar = gpui::div()
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(colors.separator)
            .bg(colors.background)
            .child(navbar_top)
            .child(docs_tabs);

        // ---- sidebar ---------------------------------------------------------
        let mut sidebar = gpui::div()
            .id("sidebar")
            .w(px(232.))
            .flex_shrink_0()
            .overflow_y_scroll()
            .restrict_scroll_to_axis()
            .border_r_1()
            .border_color(colors.separator)
            .px(px(16.))
            .py(px(22.))
            .flex()
            .flex_col()
            .gap(px(20.));

        for section in nav_sections().into_iter().filter(|section| {
            section
                .items
                .first()
                .is_some_and(|item| item.docs_root() == active_root)
        }) {
            let mut col = gpui::div().flex().flex_col().gap(px(1.));
            col = col.child(
                gpui::div()
                    .px(px(10.))
                    .pb(px(7.))
                    .text_size(px(10.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.muted)
                    .child(section.title.to_owned()),
            );
            for item in section.items {
                let active = self.page == item;
                let mut row = gpui::div()
                    .id(gpui::ElementId::Name(format!("nav-{item:?}").into()))
                    .px(px(10.))
                    .py(px(5.))
                    .rounded(px(6.))
                    .text_size(px(13.))
                    .cursor_pointer()
                    .tab_index(0)
                    .focus(move |style| style.bg(colors.default.hover()));
                if active {
                    row = row
                        .bg(colors.default.soft())
                        .font_weight(gpui::FontWeight::SEMIBOLD);
                } else {
                    row = row.hover(move |s| s.bg(colors.default.soft()));
                }
                row = row.text_color(if active {
                    colors.foreground
                } else {
                    colors.muted
                });
                col = col.child(
                    row.child(item.title())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.page = item;
                            this.dropdown_open = false;
                            cx.notify();
                        }))
                        .on_key_down(cx.listener(
                            move |this, event: &gpui::KeyDownEvent, _, cx| {
                                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                    this.page = item;
                                    this.dropdown_open = false;
                                    cx.notify();
                                }
                            },
                        )),
                );
            }
            sidebar = sidebar.child(col);
        }

        // ---- content ---------------------------------------------------------

        let content = gpui::div()
            .id("content")
            .flex_1()
            .overflow_y_scroll()
            .restrict_scroll_to_axis()
            .px(px(36.))
            .py(px(28.))
            .min_w_0()
            .child(self.render_current_page(cx));

        // The shell records keyboard-versus-pointer input, which is what a focus
        // ring reads, and moves the focus on Tab -- in a browser the platform
        // does both.
        h::extend::app_focus_root(gpui::div(), _window, cx)
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.foreground)
            .font_family(FONT_FAMILY)
            .text_size(px(14.))
            .line_height(px(20.))
            .relative()
            .on_key_down(cx.listener(Self::toggle_command_palette))
            .child(navbar)
            .child(gpui::div().flex().flex_1().min_h_0().child(sidebar).child(content))
            .child(self.command_palette(cx))
            // Toasts last so they paint above the shell. Modal and Drawer
            // demos live on their own pages.
            .child(self.toast_viewport())
    }
}

impl Gallery {
    /// Used by the `HEROGPUI_PAGE` env var to open a specific docs page
    /// (screenshot/testing helper).
    pub fn set_initial_page(&mut self, page: Page) {
        self.page = page;
    }

    /// `HEROGPUI_OPEN_OVERLAYS`, but settable while the app runs: the control
    /// file (see `control.rs`) switches it between batch steps, and the web
    /// bootstrap applies `?overlays=1` through this before the first frame.
    pub fn set_overlays_open(&mut self, open: bool) {
        self.overlays_open = open;
        // The keyed demos read `overlays_open` through `demo_overlay`, but the
        // nine dialogs with a field of their own are seeded once at startup, so
        // a control-file step has to move those too -- otherwise `overlays=1`
        // opens every overlay except the Modal, the Drawer and the Dropdown.
        self.modal_open = open;
        self.dropdown_open = open;
        self.select_open = open;
        self.drawer_open = open;
        self.range_open = open;
        self.popover_open = open;
        self.alert_dialog_open = open;
        self.cal_year_picker = open;
    }
}
