//! The gallery application shell: navbar, sidebar, content router and all
//! interactive demo state.

use std::collections::HashSet;

use gpui::{
    prelude::*, px, Context, Entity, IntoElement, ParentElement, Render, SharedString,
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
}

impl Gallery {
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

        let navbar = gpui::div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(60.))
            .px(px(20.))
            .border_b_1()
            .border_color(colors.separator)
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

        // ---- sidebar ---------------------------------------------------------
        let mut sidebar = gpui::div()
            .id("sidebar")
            .w(px(250.))
            .flex_shrink_0()
            .overflow_y_scroll()
            .border_r_1()
            .border_color(colors.separator)
            .px(px(12.))
            .py(px(16.))
            .flex()
            .flex_col()
            .gap(px(16.));

        for section in nav_sections() {
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
                    .px(px(8.))
                    .py(px(5.))
                    .rounded(px(8.))
                    .text_size(px(13.5))
                    .cursor_pointer();
                if active {
                    row = row
                        .bg(colors.default.soft())
                        .font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    row = row.hover(move |s| s.bg(colors.default.soft()));
                }
                row = row.text_color(if active {
                    colors.foreground
                } else {
                    colors.muted
                });
                col = col.child(row.child(item.title()).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.page = item;
                        this.dropdown_open = false;
                        cx.notify();
                    },
                )));
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

        gpui::div()
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
            .child(h::ToastViewport::new())
    }
}

impl Gallery {
    /// Used by the `HEROGPUI_PAGE` env var to open a specific docs page
    /// (screenshot/testing helper).
    pub fn set_initial_page(&mut self, page: Page) {
        self.page = page;
    }
}
