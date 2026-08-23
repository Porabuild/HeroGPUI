//! The gallery application shell: navbar, sidebar, content router and all
//! interactive demo state.

use std::collections::{HashMap, HashSet};

use gpui::{
    prelude::*, px, App, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_components as h;
use herogpui_theme::{toggle_light_dark, toggle_reduce_motion, ActiveTheme};

use crate::pages::{nav_sections, Page};

pub const FONT_FAMILY: &str = if cfg!(target_os = "macos") {
    "Helvetica Neue"
} else if cfg!(target_os = "linux") {
    "Ubuntu"
} else {
    "Segoe UI"
};

pub const MONO_FONT: &str = if cfg!(target_os = "macos") {
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

/// Root view of the gallery window.
pub struct Gallery {
    pub(crate) page: Page,

    // -- demo state ---------------------------------------------------------
    pub button_clicks: u32,
    pub switch_a: bool,
    pub switch_b: bool,
    pub cb_basic: bool,
    pub cb_color: bool,
    pub radio_sel: Option<usize>,
    pub slider_value: f32,
    pub slider_range: Vec<f32>,
    pub tab_underline: SharedString,
    pub tab_solid: SharedString,
    pub accordion_open: HashSet<SharedString>,
    pub modal_open: bool,
    pub dropdown_open: bool,
    pub dropdown_selected: Option<SharedString>,
    pub pagination_page: usize,
    pub alert_visible: bool,
    pub input_submitted: String,

    pub input_name: Entity<h::InputState>,
    pub input_email: Entity<h::InputState>,
    pub input_bio: Entity<h::InputState>,

    // -- parity batch state --------------------------------------------------
    pub select_lang: Option<usize>,
    pub select_multi: Vec<usize>,
    pub select_open: bool,
    pub ac_entity: Entity<h::InputState>,
    pub drawer_open: bool,
    pub otp: Entity<h::OtpState>,
    pub otp_done: String,
    /// InputOTP "Controlled": every keystroke, not just completion.
    pub otp_typed: String,
    pub number: Entity<h::NumberState>,
    pub price: Entity<h::NumberState>,
    pub calendar: Entity<h::CalendarState>,
    pub cal_picked: Option<h::Date>,
    pub date_range: Entity<h::DateRangeState>,
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
    pub combo_state: Entity<h::InputState>,
    pub combo_open: bool,
    pub alert_dialog_open: bool,
    pub picker_color: h::PickerColor,
    pub color_picker_open: bool,
    pub swatch_selected: h::PickerColor,
    pub time: Entity<h::TimeState>,
    pub search_state: Entity<h::InputState>,
    pub search_query: String,
    pub text_field_state: Entity<h::InputState>,
    pub group_amount: Entity<h::InputState>,
    pub cal_year_picker: bool,
    pub table_selection: Vec<SharedString>,
    pub table_sort: Option<h::SortDescriptor>,
    pub dropdown_multi: Vec<SharedString>,
    pub color_field_state: Entity<h::InputState>,

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
        let name = cx.new(|cx| h::InputState::new(cx));
        let email = cx.new(|cx| h::InputState::new(cx));
        // Seeded with newlines so the multi-line surface is visible at rest.
        let bio = cx.new(|cx| {
            h::InputState::with_value(
                cx,
                "Ported HeroUI v3 to GPUI.
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
        let calendar = cx.new(|cx| h::CalendarState::new(cx));
        let date_range = cx.new(|cx| h::DateRangeState::new(cx));
        let combo_state = cx.new(|cx| h::InputState::new(cx));
        let search_state = cx.new(|cx| h::InputState::new(cx));
        let text_field_state = cx.new(|cx| h::InputState::new(cx));
        let group_amount = cx.new(|cx| h::InputState::new(cx));
        let time = cx.new(|cx| h::TimeState::with_value(cx, h::Time::new(9, 30)));
        let color_field_state = cx.new(|cx| h::InputState::new(cx));

        // Re-render the shell whenever toasts change.
        let toasts = h::toast_store(cx);
        cx.observe(&toasts, |_, _, cx| cx.notify()).detach();

        let mut accordion_open = HashSet::new();
        accordion_open.insert(SharedString::from("1"));
        let mut disclosure_group_expanded = HashSet::new();
        disclosure_group_expanded.insert(SharedString::from("item-1"));
        let mut toggle_multiple = HashSet::new();
        toggle_multiple.insert(SharedString::from("bold"));
        toggle_multiple.insert(SharedString::from("underline"));

        Self {
            page: Page::Introduction,
            button_clicks: 0,
            switch_a: true,
            switch_b: false,
            cb_basic: true,
            cb_color: false,
            radio_sel: Some(0),
            slider_value: 40.0,
            slider_range: vec![20.0, 70.0],
            tab_underline: "home".into(),
            tab_solid: "music".into(),
            accordion_open,
            modal_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            dropdown_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            dropdown_selected: None,
            pagination_page: 1,
            alert_visible: true,
            input_submitted: String::new(),
            input_name: name,
            input_email: email,
            input_bio: bio,
            select_lang: None,
            select_multi: vec![0],
            select_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            ac_entity: ac,
            drawer_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            otp,
            otp_done: String::new(),
            otp_typed: String::new(),
            number,
            price,
            calendar,
            cal_picked: None,
            date_range,
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
            combo_state,
            combo_open: false,
            alert_dialog_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            picker_color: h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
            color_picker_open: false,
            swatch_selected: h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
            time,
            search_state,
            search_query: String::new(),
            text_field_state,
            group_amount,
            cal_year_picker: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            table_selection: Vec::new(),
            table_sort: None,
            dropdown_multi: Vec::new(),
            color_field_state,
            demo_text: HashMap::new(),
            demo_number: HashMap::new(),
            demo_time: HashMap::new(),
            demo_otp: HashMap::new(),
            demo_calendar: HashMap::new(),
            demo_range: HashMap::new(),
            demo_flags: HashMap::new(),
            overlays_open: std::env::var("HEROGPUI_OPEN_OVERLAYS").is_ok(),
            toast_placement: h::ToastPlacement::BottomEnd,
            demo_values: HashMap::new(),
            demo_strings: HashMap::new(),
            demo_selections: HashMap::new(),
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let colors = cx.colors().clone();

        // ---- top navbar ----------------------------------------------------
        let is_dark = cx.is_dark_theme();
        let theme_button = h::Button::new("theme-toggle")
            .variant(h::Variant::Tertiary)
            .is_icon_only(true)
            .on_press(cx.listener(|_, _, _, cx| {
                toggle_light_dark(cx);
                cx.notify();
            }))
            .start_content(
                gpui::svg()
                    .size(px(16.))
                    .path(if is_dark {
                        h::icons::SUN
                    } else {
                        h::icons::MOON
                    })
                    .text_color(colors.foreground),
            );

        // v3 exposes reduced motion as an app-level switch that every animated
        // component honours without opt-in.
        let reduce_motion = cx.reduce_motion();
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
            .href("https://github.com/heroui-inc/heroui");

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
                            .child("v0.1.0"),
                    ),
            )
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(github_link)
                    .child(motion_button)
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
            .w(px(238.))
            .flex_shrink_0()
            .overflow_y_scroll()
            .border_r_1()
            .border_color(colors.separator)
            .px(px(14.))
            .py(px(20.))
            .flex()
            .flex_col()
            .gap(px(18.));

        for section in nav_sections().into_iter().filter(|section| {
            section
                .items
                .first()
                .is_some_and(|item| item.docs_root() == active_root)
        }) {
            let mut col = gpui::div().flex().flex_col().gap(px(2.));
            col = col.child(
                gpui::div()
                    .px(px(8.))
                    .pb(px(6.))
                    .text_size(px(11.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.muted)
                    .child(section.title.to_owned()),
            );
            for item in section.items {
                let active = self.page == item;
                let mut row = gpui::div()
                    .id(gpui::ElementId::Name(format!("nav-{item:?}").into()))
                    .px(px(10.))
                    .py(px(6.))
                    .border_l_2()
                    .border_color(if active {
                        colors.accent.color
                    } else {
                        gpui::transparent_black()
                    })
                    .rounded(px(9.))
                    .text_size(px(13.5))
                    .cursor_pointer()
                    .tab_index(0)
                    .focus(move |style| style.bg(colors.default.soft()));
                if active {
                    row = row
                        .bg(colors.accent.soft())
                        .font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    row = row.hover(move |s| s.bg(colors.default.soft()));
                }
                row = row.text_color(if active {
                    colors.accent.color
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
            .px(px(36.))
            .py(px(28.))
            .min_w_0()
            .child(self.render_current_page(cx));

        // The shell records keyboard-versus-pointer input, which is what a focus
        // ring reads, and moves the focus on Tab -- in a browser the platform
        // does both.
        h::util::app_focus_root(gpui::div(), _window, cx)
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.background)
            .text_color(colors.foreground)
            .font_family(FONT_FAMILY)
            .text_size(px(14.))
            .line_height(px(20.))
            .relative()
            .child(navbar)
            .child(gpui::div().flex().flex_1().min_h_0().child(sidebar).child(content))
            // Toasts last so they paint above the shell. Modal and Drawer
            // demos live on their own pages.
            .child(h::ToastViewport::new().placement(self.toast_placement))
    }
}

impl Gallery {
    /// Used by the `HEROGPUI_PAGE` env var to open a specific docs page
    /// (screenshot/testing helper).
    pub fn set_initial_page(&mut self, page: Page) {
        self.page = page;
    }

    /// `HEROGPUI_OPEN_OVERLAYS`, but settable while the app runs: the control
    /// file (see `control.rs`) switches it between batch steps.
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
