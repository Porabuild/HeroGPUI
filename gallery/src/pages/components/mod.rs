//! Component gallery pages — one page per HeroUI v3 component.
//!
//! `redundant_clone` reads this file wrongly. Every page is one doc-page macro
//! invocation, and the lint analyses the expanded body:
//! it sees a collection cloned into one example and reports the clone as
//! needless without accounting for the later example that moves the original.
//! Removing the ten it flags does not compile -- `items` feeds ListBox's
//! single-selection example and then its multi-selection one, `options` feeds
//! three RadioGroup examples in a row, and so on. The lint is allowed here for
//! that reason and nowhere else; a genuinely redundant clone in a component
//! crate still fails the gate.
#![allow(clippy::redundant_clone)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use gpui::{prelude::*, px, AnyElement, Context, SharedString};
use herogpui_components as h;
use herogpui_core::{Color, FieldVariant, Orientation, SelectionMode, Size, SizeXl, Variant};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::{muted_para, para};

thread_local! {
    /// The Form "Server Errors" demo's current `validationErrors` record.
    ///
    /// Held outside the page function on purpose: the page rebuilds its
    /// elements every frame, and the record's *identity* is the contract the
    /// demo exists to show. A record minted per frame would re-arm both
    /// fields on every keystroke; this one is cloned per frame — clones keep
    /// the record's revision — and replaced only by the demo's New response
    /// button, which is a genuinely new response.
    pub(super) static FORM_SERVER_RECORD: RefCell<Option<h::ValidationErrors>> = const { RefCell::new(None) };
}

macro_rules! component_doc_section {
    (($heading:expr, $body:expr $(,)?)) => {
        ($heading, None, $body, stringify!($body))
    };
    (($heading:expr, $description:literal, $body:expr $(,)?)) => {
        ($heading, Some($description), $body, stringify!($body))
    };
}

pub(super) fn preview_wrapper(body: impl IntoElement, cx: &gpui::App) -> AnyElement {
    let dot = cx.colors().muted.alpha(0.22);
    let stage = cx.colors().background;
    gpui::div()
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .size_full()
        .bg(stage)
        .child(crate::pages::stage_dot_grid(dot))
        .child(body)
        .into_any_element()
}

macro_rules! component_preview_section {
    (($heading:expr, $body:expr $(,)?), $cx:expr) => {
        if crate::control::section_wanted($heading, $cx) {
            return preview_wrapper($body, $cx);
        }
    };
    (($heading:expr, $description:literal, $body:expr $(,)?), $cx:expr) => {
        if crate::control::section_wanted($heading, $cx) {
            return preview_wrapper($body, $cx);
        }
    };
}

macro_rules! component_doc_page {
    (
        $title:expr,
        $description:expr,
        $import_line:expr,
        vec![$($section:tt),* $(,)?],
        $cx:expr $(,)?
    ) => {
        if crate::control::preview_only($cx) {
            $(component_preview_section!($section, $cx);)*
            gpui::div().into_any_element()
        } else {
            crate::pages::component_doc_page(
                $title,
                $description,
                $import_line,
                vec![$(component_doc_section!($section)),*],
                $cx,
            )
        }
    };
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Typical HeroUI demo field width (`w-[256px]` / `w-64` / `max-w-xs`).
///
/// Fields in this port `max_w(320)` but hug their placeholder or value unless
/// a parent gives them a definite width. Gallery examples that are not the
/// dedicated `full_width` specimen sit in this column so they read as a form
/// control rather than a collapsed chip.
pub(super) const DEMO_FIELD_W: f32 = 256.;

/// v3's Alert "Usage" copy, held apart from the demo so the literal keeps its
/// single spaces: rustfmt joins a `\` line continuation without dropping the
/// indentation that followed it.
pub(super) const ALERT_USAGE_DESCRIPTION: &str = concat!(
    "Check out our latest updates including dark mode support ",
    "and improved accessibility features.",
);

/// v3's Card "Usage" copy. See [`ALERT_USAGE_DESCRIPTION`].
pub(super) const CARD_USAGE_DESCRIPTION: &str = concat!(
    "Visit the Acme Creator Hub to sign up today and start earning ",
    "credits from your fans and followers.",
);

pub(super) fn row(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// Wrapping specimen row.
pub(super) fn spec_row(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_wrap()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

pub(super) fn col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        // Components hug their content in a demo; full_width examples opt back
        // in explicitly. Field examples that need a definite width use
        // field_col / demo_field instead of stretching this helper.
        .items_center()
        .justify_center()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// Column whose children fill the preview's width.
///
/// Overlay demos need it. A modal, alert dialog or drawer panel is
/// `absolute inset-0` inside its frame, and [`col`] aligns its children to the
/// start, so a frame holding only a trigger hugs that trigger and the panel
/// fills a sliver of it -- the dialog came out about 30px wide with its heading
/// broken one character per line.
pub(super) fn stretch_col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// Column that stretches its children across v3's `w-full max-w-xl` example
/// frame. A `w-full` component -- Alert, Toast, Skeleton -- resolves its width
/// against its container, so a hug-content column like [`col`] leaves it at its
/// content width and the example demonstrates the opposite of the rule.
pub(super) fn wide_col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        .w_full()
        .max_w(px(576.))
        .gap(px(16.))
        .children(children)
        .into_any_element()
}

/// Column that stretches children to [`DEMO_FIELD_W`]. Use for Input, Select,
/// ComboBox, Autocomplete, TextArea, SearchField, Date/Time and similar
/// specimens — not for `full_width` examples, which must fill the frame.
pub(super) fn field_col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        .w(px(DEMO_FIELD_W))
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// One control that should occupy the demo field width inside a mixed column
/// (a field beside a `para`, a spinner, or a toggle).
pub(super) fn demo_field(el: impl IntoElement) -> AnyElement {
    gpui::div()
        .w(px(DEMO_FIELD_W))
        .flex()
        .flex_col()
        .child(el)
        .into_any_element()
}

pub(super) fn fixed_demo(width: f32, el: impl IntoElement) -> AnyElement {
    gpui::div().w(px(width)).child(el).into_any_element()
}

/// A labelled specimen — the caption HeroUI puts under each variant.
pub(super) fn spec(label: &str, el: impl IntoElement, cx: &gpui::App) -> AnyElement {
    let muted = cx.colors().muted;
    gpui::div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.))
        .child(el)
        .child(
            gpui::div()
                .text_size(px(11.))
                .text_color(muted)
                .child(label.to_owned()),
        )
        .into_any_element()
}

/// A labelled specimen that fills its row, for a block-level component.
pub(super) fn spec_block(label: &str, el: impl IntoElement, cx: &gpui::App) -> AnyElement {
    let muted = cx.colors().muted;
    gpui::div()
        .flex()
        .flex_col()
        .w_full()
        .gap(px(6.))
        .child(
            gpui::div()
                .text_size(px(11.))
                .text_color(muted)
                .child(label.to_owned()),
        )
        .child(el)
        .into_any_element()
}

/// Collects an iterator of elements into a `Vec<AnyElement>`.
pub(super) trait IntoVecEls {
    fn els(self) -> Vec<AnyElement>;
}

impl<I> IntoVecEls for I
where
    I: IntoIterator,
    I::Item: IntoElement,
{
    fn els(self) -> Vec<AnyElement> {
        self.into_iter()
            .map(IntoElement::into_any_element)
            .collect()
    }
}

pub(super) fn el_id(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

/// The nth of the thousand users v3's virtualization examples list, as
/// `(name, email)`. Same two twenty-name lists, so the rows read the same.
pub(super) fn virtual_user(i: usize) -> (String, String) {
    pub(super) const FIRST: [&str; 20] = [
        "Emma",
        "Liam",
        "Olivia",
        "Noah",
        "Ava",
        "James",
        "Sophia",
        "Oliver",
        "Isabella",
        "Lucas",
        "Mia",
        "Ethan",
        "Charlotte",
        "Mason",
        "Amelia",
        "Logan",
        "Harper",
        "Alexander",
        "Ella",
        "Benjamin",
    ];
    pub(super) const LAST: [&str; 20] = [
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Anderson",
        "Taylor",
        "Thomas",
        "Jackson",
        "White",
        "Harris",
        "Clark",
        "Lewis",
        "Robinson",
        "Walker",
    ];
    let first = FIRST[i % FIRST.len()];
    let last = LAST[(i / FIRST.len()) % LAST.len()];
    (
        format!("{first} {last}"),
        format!("{}.{}@acme.com", first.to_lowercase(), last.to_lowercase()),
    )
}

/// The same thousand names as keyed picker items: the key is the stable
/// `user-N` id, the label the visible name.
pub(super) fn virtual_picker_items() -> Vec<h::PickerItem> {
    (0..1000)
        .map(|i| h::PickerItem::new(format!("user-{i}"), virtual_user(i).0))
        .collect()
}

/// `languages()` as keyed items, for the Autocomplete and ComboBox demos: the
/// slug is the stable key the selection, `disabledKeys` and the form value
/// address, the display name the label the filtering, input text and
/// rendering use.
pub(super) fn language_items() -> Vec<h::PickerItem> {
    languages()
        .into_iter()
        .map(|label| h::PickerItem::new(label.to_lowercase(), label))
        .collect()
}

/// A thousand list rows, for the virtualization demos.
pub(super) fn virtual_users() -> Vec<h::ListBoxItem> {
    (0..1000)
        .map(|i| {
            let (name, email) = virtual_user(i);
            h::ListBoxItem::new(format!("user-{i}"), name).description(email)
        })
        .collect()
}

/// The same thousand users, but every third row carries a description and a
/// section header lands every hundred -- rows of three different heights, which
/// is what `estimated_row_height` virtualizes.
pub(super) fn virtual_users_described() -> Vec<h::ListBoxItem> {
    let mut items = Vec::with_capacity(1010);
    for i in 0..1000 {
        if i % 100 == 0 {
            items.push(h::ListBoxItem::section(format!("Batch {}", i / 100 + 1)));
        }
        let (name, email) = virtual_user(i);
        let item = h::ListBoxItem::new(format!("user-{i}"), name);
        items.push(if i % 3 == 0 {
            item.description(email)
        } else {
            item
        });
    }
    items
}

/// The `ToastViewport` mount every application needs once.
pub(super) const TOAST_SETUP: &str = r#"// Once, in the shell:
div()
    .child(page)
    .child(ToastViewport::new()
        .placement(ToastPlacement::BottomEnd)
        .max_visible_toasts(2))

// Anywhere, afterwards:
Toast::new("Saved")
    .description("Your changes are live.")
    .variant(Color::Success)
    .closable(true)
    .push(Some(Duration::from_secs(4)), cx);"#;

#[cfg(test)]
pub(super) fn toast_setup_block() -> &'static str {
    TOAST_SETUP
}

pub(super) fn overlay_min_h(mut frame: gpui::Div, open: bool, height: f32) -> gpui::Div {
    if open {
        frame = frame.min_h(px(height));
    }
    frame
}

pub(super) fn set_popover_open(
    usage: &mut bool,
    flags: &mut HashMap<&'static str, bool>,
    key: &'static str,
    open: bool,
) {
    if key == "po-usage" {
        *usage = open;
    } else {
        flags.insert(key, open);
    }
}

/// One overlay demo: the trigger, and the panel it opens.
///
/// An overlay needs a positioned ancestor and enough height to show the panel,
/// and each demo owns its own open flag -- v3's pages show one variant per
/// example, so a shared flag would open all of them at once.
pub(super) fn overlay_demo(
    open: bool,
    key: &'static str,
    label: &str,
    panel: AnyElement,
    cx: &mut Context<'_, Gallery>,
) -> AnyElement {
    // The panel is `absolute inset-0` inside this frame, and v3's body is
    // `min-h-0 flex-1`, so a short frame squeezes the body to nothing.
    // Reserve that height only while the panel is open; a closed trigger
    // sitting in a 320px hole was the empty-card gap on overlay pages.
    overlay_min_h(
        gpui::div()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .w_full(),
        open,
        320.,
    )
    .child(
        h::Button::new(el_id(format!("{key}-open")))
            .label(label.to_owned())
            .variant(Variant::Secondary)
            .on_press(cx.listener(move |this, _, _, cx| {
                this.set_demo_flag(key, true);
                cx.notify();
            })),
    )
    .child(panel)
    .into_any_element()
}

/// The demo palette used by the color pages.
pub(super) fn palette() -> Vec<h::PickerColor> {
    [
        "#0085F5", "#17C964", "#F5A524", "#F31260", "#7828C8", "#0E8AAA", "#71717A", "#18181B",
    ]
    .into_iter()
    .filter_map(h::PickerColor::from_hex)
    .collect()
}

mod buttons;
mod collections;
mod colors;
mod controls;
mod data_display;
mod date_and_time;
mod feedback;
mod forms;
mod layout;
mod media;
mod navigation;
mod overlays;
mod pickers;
mod typography;
mod utilities;

// ---------------------------------------------------------------------------
// small shared bits
// ---------------------------------------------------------------------------

/// gpui svgs never inherit `text_color`, so demo icons set it explicitly.
pub(super) fn icon(path: &'static str, cx: &gpui::App) -> AnyElement {
    gpui::svg()
        .size(px(16.))
        .path(path)
        .text_color(cx.colors().foreground)
        .into_any_element()
}

/// One stable custom image source for the Avatar "Custom Image Component"
/// demo: the loader `Arc` is built once, so the avatar's per-source keyed
/// state and its `on_load` keep their identity across frames.
pub(super) fn sample_avatar_source() -> gpui::ImageSource {
    type AvatarLoader = std::sync::Arc<
        dyn Fn(
                &mut gpui::Window,
                &mut gpui::App,
            )
                -> Option<Result<std::sync::Arc<gpui::RenderImage>, gpui::ImageCacheError>>
            + Send
            + Sync,
    >;
    static LOADER: std::sync::OnceLock<AvatarLoader> = std::sync::OnceLock::new();
    let loader: &AvatarLoader = LOADER.get_or_init(|| {
        let image = std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            include_bytes!("../../../assets/herogpui/sample.png").to_vec(),
        ));
        std::sync::Arc::new(move |window: &mut gpui::Window, cx: &mut gpui::App| {
            image.clone().use_render_image(window, cx).map(Ok)
        })
    });
    gpui::ImageSource::Custom(loader.clone())
}

/// A neutral block used as the child of badge demos.
/// The anchor every v3 Badge example uses: an `Avatar` with initials behind
/// the badge. Upstream loads three CDN portraits and falls back to
/// `JD`/`AB`/`CD`; the gallery makes no network calls, so the fallback shows.
pub(super) fn avatar_box(id: impl Into<gpui::ElementId>, name: &'static str) -> AnyElement {
    h::Avatar::new(id).name(name).into_any_element()
}

pub(super) fn languages() -> Vec<SharedString> {
    vec![
        "Rust".into(),
        "TypeScript".into(),
        "Python".into(),
        "Go".into(),
        "Swift".into(),
        "Kotlin".into(),
    ]
}

pub(super) fn toggle_key(set: &mut HashSet<SharedString>, key: &SharedString) {
    if !set.remove(key) {
        set.insert(key.clone());
    }
}

pub(super) fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod example_quality {
    use super::*;

    const SRC: &str = concat!(
        include_str!("mod.rs"),
        include_str!("buttons.rs"),
        include_str!("collections.rs"),
        include_str!("colors.rs"),
        include_str!("controls.rs"),
        include_str!("data_display.rs"),
        include_str!("date_and_time.rs"),
        include_str!("feedback.rs"),
        include_str!("forms.rs"),
        include_str!("layout.rs"),
        include_str!("media.rs"),
        include_str!("navigation.rs"),
        include_str!("overlays.rs"),
        include_str!("pickers.rs"),
        include_str!("typography.rs"),
        include_str!("utilities.rs"),
    );

    fn page_fn<'a>(src: &'a str, name: &str) -> &'a str {
        let needle = format!("    pub fn page_{name}(");
        let start = src
            .find(&needle)
            .unwrap_or_else(|| panic!("missing page_{name}"));
        let rest = &src[start..];
        let next = rest
            .get(needle.len()..)
            .and_then(|rest| rest.find("\n    pub fn page_"))
            .map_or(rest.len(), |i| needle.len() + i);
        &rest[..next]
    }

    fn section_entries(src: &str) -> Vec<(String, usize)> {
        let lines: Vec<&str> = src.split_inclusive('\n').collect();
        let mut entries = Vec::new();
        let mut offset = 0;
        for (index, raw_line) in lines.iter().enumerate() {
            let line = raw_line.trim_end_matches(['\n', '\r']);
            let title = if line.starts_with("                (\"") {
                line.get(18..)
                    .and_then(|rest| rest.find('\"').map(|end| rest[..end].to_owned()))
            } else if line == "                (" {
                lines.get(index + 1).and_then(|next| {
                    let next = next.trim_end_matches(['\n', '\r']);
                    next.strip_prefix("                    \"")
                        .and_then(|rest| rest.find('\"').map(|end| rest[..end].to_owned()))
                })
            } else {
                None
            };
            if let Some(title) = title {
                entries.push((title, offset));
            }
            offset += raw_line.len();
        }
        entries
    }

    fn section_titles(src: &str) -> Vec<String> {
        section_entries(src)
            .into_iter()
            .map(|(title, _)| title)
            .collect()
    }

    fn section_body<'a>(src: &'a str, title: &str) -> &'a str {
        let entries = section_entries(src);
        let (index, (_, start)) = entries
            .iter()
            .enumerate()
            .find(|(_, (entry, _))| entry.eq_ignore_ascii_case(title))
            .unwrap_or_else(|| panic!("missing section {title}"));
        let end = entries
            .get(index + 1)
            .map_or(src.len(), |(_, offset)| *offset);
        &src[*start..end]
    }

    #[test]
    fn demo_field_width_matches_heroui_w256() {
        assert!((DEMO_FIELD_W - 256.0).abs() < f32::EPSILON);
    }

    #[test]
    fn primary_examples_lead_with_usage() {
        for name in [
            "select",
            "autocomplete",
            "combo_box",
            "slider",
            "date_field",
            "alert_dialog",
            "dropdown",
            "popover",
        ] {
            let page = page_fn(SRC, name);
            assert_eq!(
                section_titles(page).first().map(String::as_str),
                Some("Usage"),
                "page_{name} should open with Usage"
            );
        }
    }

    #[test]
    fn disclosure_render_function_uses_live_component_state() {
        let page = page_fn(SRC, "disclosure");
        assert_eq!(
            section_titles(page).get(1).map(String::as_str),
            Some("Render Function")
        );
        let render = section_body(page, "Render Function");
        assert!(render.contains("let render_expanded = self.demo_flag("));
        assert!(render.contains(".is_expanded(render_expanded)"));
        assert!(render.contains(".content(|state|"));
        assert!(render.contains("state.is_expanded"));
        assert!(render.contains("state.is_disabled"));
        assert!(render.contains(".on_expanded_change("));
    }

    #[test]
    fn close_button_follows_pinned_examples_and_exposes_render_state() {
        let page = page_fn(SRC, "close_button");
        assert_eq!(
            &section_titles(page)[..3],
            &["Usage", "Interactive", "With Custom Icon"]
        );
        let custom_icon = section_body(page, "With Custom Icon");
        assert_eq!(custom_icon.matches(".icon(").count(), 1);

        let render = section_body(page, "Render Function");
        assert!(render.contains(".content(move |state|"));
        for field in [
            "state.is_hovered",
            "state.is_pressed",
            "state.is_focused",
            "state.is_disabled",
        ] {
            assert!(render.contains(field), "missing render state {field}");
        }
        assert!(!render.contains("para("));
    }

    #[test]
    fn range_calendar_unavailable_dates_uses_explicit_pinned_ranges() {
        let page = page_fn(SRC, "range_calendar");
        let unavailable = section_body(page, "Unavailable Dates");
        for offset in [2, 5, 6, 9, 12, 13] {
            assert!(
                unavailable.contains(&format!("h::add_days(&today, {offset})")),
                "missing pinned relative date offset {offset}"
            );
        }
        assert!(unavailable.contains(".first_day_of_week(h::Weekday::Mon)"));
        assert!(unavailable.contains("Some days are unavailable"));
        assert!(!unavailable.contains("weekday_index"));
    }

    #[test]
    fn calendar_first_day_example_demonstrates_the_regional_override() {
        let page = page_fn(SRC, "calendar");
        let first_day = section_body(page, "First day of week");
        assert!(first_day.contains(".first_day_of_week(h::Weekday::Mon)"));
    }

    #[test]
    fn gallery_sections_are_preserved_while_reordering() {
        for (name, count) in [
            ("select", 18),
            ("autocomplete", 18),
            ("combo_box", 26),
            ("slider", 15),
            ("date_field", 14),
            ("alert_dialog", 12),
            ("dropdown", 19),
            ("popover", 6),
            ("number_field", 16),
            ("text_area", 6),
            ("date_range_picker", 9),
            ("list_box", 14),
            ("tag_group", 14),
            ("meter", 5),
            ("progress_bar", 6),
        ] {
            assert_eq!(
                section_titles(page_fn(SRC, name)).len(),
                count,
                "page_{name} lost a gallery section"
            );
        }
    }

    #[test]
    fn requested_specimen_dimensions_match_pinned_demos() {
        let number = section_body(page_fn(SRC, "number_field"), "Usage");
        assert!(number.contains("field_col("), "NumberField basic width");
        assert!(
            number.contains(".full_width(true)"),
            "NumberField fills max-w-64"
        );
        assert!(number.contains(".default_value(1024.)"), "NumberField seed");
        assert!(number.contains(".min_value(0.)"), "NumberField minimum");
        assert!(number.contains(".name(\"width\")"), "NumberField form name");

        let list_box = section_body(page_fn(SRC, "list_box"), "Usage");
        assert!(list_box.contains(".w(px(220.))"), "ListBox basic width");

        let text_area = section_body(page_fn(SRC, "text_area"), "Usage");
        let text_area_compact = text_area.split_whitespace().collect::<String>();
        assert!(
            text_area_compact.contains("fixed_demo(384."),
            "TextArea basic width"
        );
        assert!(
            text_area.contains(".full_width()"),
            "TextArea fills its width"
        );
        assert!(text_area.contains(".cols(48)"), "TextArea basic width");
        assert!(text_area.contains(".rows(6)"), "TextArea basic height");
        assert!(
            text_area.contains("placeholder(\"Share a quick project update...\")"),
            "TextArea basic placeholder"
        );

        let date_range = section_body(page_fn(SRC, "date_range_picker"), "Usage");
        assert!(
            date_range.contains(".w(px(320.))"),
            "DateRangePicker basic width"
        );

        let slider = section_body(page_fn(SRC, "slider"), "Usage");
        assert!(slider.contains(".w(px(320.))"), "Slider basic width");

        let meter = section_body(page_fn(SRC, "meter"), "Usage");
        assert!(meter.contains(".w(px(256.))"), "Meter basic width");
        let progress = section_body(page_fn(SRC, "progress_bar"), "Usage");
        assert!(progress.contains(".w(px(256.))"), "ProgressBar basic width");
    }

    #[test]
    fn bounded_scale_demos_keep_pinned_widths() {
        let slider = page_fn(SRC, "slider");
        for title in [
            "Usage",
            "Format options",
            "Range Slider Anatomy",
            "Controlled Value",
            "Custom Value Formatting",
            "Custom Output Display",
            "Range",
            "Range Slider",
            "Basic Usage",
            "Disabled",
            "Disabled Thumb",
            "Form Example",
            "Step & disabled",
        ] {
            let body = section_body(slider, title);
            let compact = body.split_whitespace().collect::<String>();
            assert!(
                compact.contains("fixed_demo(320.") || compact.contains(".w(px(320.))"),
                "Slider {title} should stay at the pinned 320px width"
            );
        }
        let vertical = section_body(slider, "Vertical");
        assert!(!vertical
            .split_whitespace()
            .collect::<String>()
            .contains("fixed_demo(320."));

        for name in ["meter", "progress_bar"] {
            let page = page_fn(SRC, name);
            for title in section_titles(page) {
                let body = section_body(page, &title);
                let compact = body.split_whitespace().collect::<String>();
                assert!(
                    compact.contains("fixed_demo(256.") || compact.contains(".w(px(256.))"),
                    "{name} {title} should stay at the pinned 256px width"
                );
            }
        }
    }

    #[test]
    fn explanatory_copy_precedes_the_live_specimen_as_section_metadata() {
        for (name, title, marker) in [
            ("slider", "Range Slider Anatomy", "fixed_demo("),
            ("slider", "Custom Output Display", ".w(px(320.))"),
            ("slider", "Disabled Thumb", "fixed_demo("),
            ("slider", "Form Example", "fixed_demo("),
            ("calendar", "Cell Indicators", "h::Calendar::new"),
            ("calendar", "Constraints", "h::Calendar::new"),
            ("date_field", "Granularity", "spec_row("),
            (
                "date_range_picker",
                "Format Options",
                "h::DateRangePicker::new",
            ),
            (
                "date_range_picker",
                "Custom Indicator",
                "h::DateRangePicker::new",
            ),
            ("list_box", "Disallow Empty Selection", "gpui::div()"),
            ("list_box", "Escape Key Behavior", "gpui::div()"),
            ("list_box", "Virtualization", "gpui::div()"),
            ("list_box", "Custom Check Icon", "gpui::div()"),
            ("tag_group", "Escape Key Behavior", "h::TagGroup::new"),
            ("autocomplete", "Virtualization", "demo_field("),
            ("autocomplete", "Asynchronous Filtering", "row(vec!["),
            ("autocomplete", "Custom Value", "h::Autocomplete::new"),
            ("combo_box", "Virtualization", "demo_field("),
            ("combo_box", "Asynchronous Loading", "row(vec!["),
            ("combo_box", "Custom Filtering", "h::ComboBox::new"),
            ("select", "Virtualization", "demo_field("),
            ("select", "Asynchronous Loading", "row(vec!["),
        ] {
            let section = section_body(page_fn(SRC, name), title);
            let after_heading = &section[section.find("\",").unwrap() + 2..];
            let marker = after_heading.find(marker).unwrap();
            assert!(
                after_heading.trim_start().starts_with('"'),
                "{name} {title} should declare explanatory copy as section metadata"
            );
            assert!(
                after_heading.find("para(").is_none_or(|para| marker < para),
                "{name} {title} should not render explanatory copy inside its live specimen"
            );
        }
    }

    #[test]
    fn field_examples_use_demo_width_column() {
        for name in [
            "input",
            "text_area",
            "text_field",
            "search_field",
            "input_group",
            "select",
            "combo_box",
            "autocomplete",
            "date_field",
            "time_field",
            "color_field",
        ] {
            let page = page_fn(SRC, name);
            assert!(
                page.contains("field_col(") || page.contains("demo_field("),
                "page_{name} should group field examples at DEMO_FIELD_W"
            );
        }
    }

    #[test]
    fn full_width_examples_stay_on_hugging_column() {
        for name in [
            "input",
            "text_area",
            "text_field",
            "search_field",
            "select",
            "combo_box",
            "autocomplete",
        ] {
            let page = page_fn(SRC, name);
            let full_width = section_body(page, "Full Width");
            assert!(full_width.contains(".full_width"));
            assert!(full_width.contains("col(vec!["));
            assert!(!full_width.contains("field_col("));
        }
    }

    #[test]
    fn select_like_examples_carry_placeholders_and_values() {
        let input = page_fn(SRC, "input");
        assert!(
            input.contains("placeholder(\"Enter your name\")"),
            "{input}"
        );
        assert!(input.contains("placeholder(\"Primary input\")"));
        assert!(input.contains("placeholder(\"Full width input\")"));

        let area = page_fn(SRC, "text_area");
        assert!(area.contains("placeholder(\"Primary textarea\")"));
        assert!(area.contains("placeholder(\"Share a quick project update...\")"));

        let select = page_fn(SRC, "select");
        assert!(select.contains("placeholder(\"Choose one\")"));
        assert!(select.contains("placeholder(\"Pick several\")"));
        assert!(select.contains(".default_value(Some(SharedString::from(\"rust\")))"));

        let combo = page_fn(SRC, "combo_box");
        assert!(combo.contains("placeholder(\"Select a user\")"));
        assert!(combo.contains("placeholder(\"Pick or type\")"));
        assert!(combo.contains("demo_text(\"cb-default-key\", \"TypeScript\""));
        assert!(combo.contains(".default_value([\"typescript\"])"));
        assert!(combo.contains("placeholder(\"Search languages...\")"));

        let ac = page_fn(SRC, "autocomplete");
        assert!(ac.contains("placeholder(\"Select a user\")"));
        assert!(ac.contains("placeholder(\"Select a language\")"));
        assert!(ac.contains("default_value([\"rust\"])"));

        let search = page_fn(SRC, "search_field");
        assert!(search.contains("placeholder(\"Search...\")"));

        let groups = page_fn(SRC, "input_group");
        assert!(groups.contains("placeholder(\"name@email.com\")"));
    }

    #[test]
    fn controlled_empty_examples_remain_empty() {
        let autocomplete = page_fn(SRC, "autocomplete");
        assert!(autocomplete.contains("demo_text(\"ac-controlled\", \"\", cx)"));
        let combo_box = page_fn(SRC, "combo_box");
        assert!(combo_box.contains("demo_text(\"cb-controlled\", \"\", cx)"));
        let select = page_fn(SRC, "select");
        assert!(select.contains("h::Select::new(\"sel-main\", language_items())"));
        assert!(!section_body(select, "Usage").contains("default_value("));
    }

    #[test]
    fn list_box_selection_modes_match_pinned_examples() {
        let page = page_fn(SRC, "list_box");
        assert!(section_body(page, "With Disabled Items")
            .contains(".selection_mode(SelectionMode::None)"));
        assert!(
            section_body(page, "Controlled").contains(".selection_mode(SelectionMode::Multiple)")
        );
        assert!(section_body(page, "Custom Check Icon")
            .contains(".selection_mode(SelectionMode::Multiple)"));
    }

    #[test]
    fn tag_group_remove_example_includes_default_and_custom_content() {
        let body = section_body(page_fn(SRC, "tag_group"), "With Remove Button");
        assert!(body.contains("Default remove button"));
        assert!(body.contains("Custom remove button"));
        assert!(body.contains(".remove_content("));
        assert_eq!(body.matches(".on_remove(").count(), 2);
    }

    #[test]
    fn specimen_rows_are_center_aligned_and_captions_are_compact() {
        let row = SRC
            .split("fn row(")
            .nth(1)
            .and_then(|rest| rest.split("fn spec_row").next())
            .expect("row helper");
        assert!(row.contains(".items_center()"));
        assert!(!row.contains(".items_start()"));
        let spec_row = SRC
            .split("fn spec_row(")
            .nth(1)
            .and_then(|rest| rest.split("fn col(").next())
            .expect("spec_row helper");
        assert!(spec_row.contains(".items_center()"));
        assert!(!spec_row.contains(".items_start()"));

        // Overlay frames stretch to the preview's width so an open panel can
        // fill it; a closed trigger must still sit centered, not top-left.
        let overlay_frames = SRC
            .split("fn overlay_min_h(")
            .nth(1)
            .expect("overlay demo frames")
            .replace(' ', "");
        assert!(overlay_frames.contains(".items_center()\n.w_full(),"));
        assert!(!overlay_frames.contains(".items_start()\n.w_full(),"));

        let avatar = page_fn(SRC, "avatar");
        for caption in [
            "Initials from a name",
            "No name at all",
            "Broken src, delayed fallback",
            "Custom fallback content",
            "Fallback color override",
        ] {
            assert!(!avatar.contains(caption), "stale Avatar caption: {caption}");
        }
        assert!(avatar.contains("\"Initials\""));
        assert!(avatar.contains("\"No name\""));

        let calendar = page_fn(SRC, "calendar");
        assert!(!calendar.contains("Grid: August 2026; heading:"));
        assert!(calendar.contains("\"Same month\""));
        assert!(calendar.contains("\"Heading offset\""));
    }

    #[test]
    fn overlay_height_is_gated_for_every_overlay_page() {
        let helper = SRC
            .split("fn overlay_min_h(")
            .nth(1)
            .and_then(|rest| rest.split("fn overlay_demo(").next())
            .expect("overlay_min_h helper");
        assert!(helper.contains("open: bool"));
        assert!(helper.contains("if open"));
        assert!(helper.contains("frame.min_h(px(height))"));
        for name in ["alert_dialog", "drawer", "modal", "popover"] {
            assert!(
                !page_fn(SRC, name).contains(".min_h(px("),
                "page_{name} has an unconditional overlay min-height"
            );
        }
    }

    #[test]
    fn time_field_examples_separate_system_and_explicit_hour_cycles() {
        let page = page_fn(SRC, "time_field");
        let usage = section_body(page, "Usage");
        assert!(usage.contains("system regional segment order, separators, padding"));
        assert!(!usage.contains(".hour_cycle("));
        assert!(section_body(page, "24-hour").contains(".hour_cycle(h::HourCycle::H24)"));
        assert!(
            section_body(page, "12-hour with seconds").contains(".hour_cycle(h::HourCycle::H12)")
        );
        let leading = section_body(page, "Forced Leading Zeros");
        assert_eq!(leading.matches(".hour_cycle(h::HourCycle::H12)").count(), 2);
        assert_eq!(leading.matches(".show_seconds(true)").count(), 2);
        assert_eq!(
            leading.matches(".should_force_leading_zeros(true)").count(),
            1
        );
        assert!(leading.contains("this prop only forces the hour to two digits"));
    }

    #[test]
    fn date_field_examples_demonstrate_system_format_and_forced_padding() {
        let page = page_fn(SRC, "date_field");
        let leading = section_body(page, "Forced Leading Zeros");
        assert_eq!(
            leading
                .matches(".granularity(h::Granularity::Second)")
                .count(),
            2
        );
        assert_eq!(leading.matches(".hour_cycle(h::HourCycle::H12)").count(), 2);
        assert_eq!(
            leading.matches(".should_force_leading_zeros(true)").count(),
            1
        );
        assert!(leading.contains("system locale controls date and time segment order, separators"));

        let date_picker = section_body(page_fn(SRC, "date_picker"), "Format Options");
        assert!(date_picker.contains("operating system's regional date order"));
        assert!(date_picker.contains("submitted value stay"));
        assert!(!date_picker.contains("needs CLDR data"));

        let range_picker = section_body(page_fn(SRC, "date_range_picker"), "Format Options");
        assert!(range_picker.contains("operating system's regional date order"));
        assert!(range_picker.contains("submitted values"));
        assert!(!range_picker.contains("needs CLDR data"));
    }

    #[test]
    fn popover_live_slots_reserve_height_per_open_demo() {
        let popover = page_fn(SRC, "popover");
        let usage = section_body(popover, "Usage");
        assert!(usage.contains("overlay_min_h("));
        assert!(usage.contains("is_open"));
        assert!(usage.contains("160."));

        let arrow = section_body(popover, "With Arrow");
        assert!(arrow.contains("po_arrow_open"));
        assert!(arrow.contains("po_arrow_custom_open"));
        assert!(!arrow.contains("po_arrow_open || po_arrow_custom_open"));

        let interactive = section_body(popover, "Interactive Content");
        assert!(interactive.contains("overlay_min_h("));
        assert!(interactive.contains("po_interactive_open"));
        assert!(interactive.contains("220."));

        let placement = section_body(popover, "Placement");
        assert!(placement.contains("let open = self.demo_overlay(id)"));
        assert!(!placement.contains("placement_open"));

        let render = section_body(popover, "Render Function");
        assert!(render.contains("po_render_open"));
        assert!(render.contains("160."));

        let custom = section_body(popover, "Custom Styles");
        assert!(custom.contains("po_custom_styles_open"));
        assert!(custom.contains("160."));
    }

    #[test]
    fn popover_height_tracks_each_demo_state() {
        let popover = page_fn(SRC, "popover");
        assert!(popover.contains("po_arrow_open"));
        assert!(popover.contains("po_interactive_open"));
        assert!(popover.contains("let open = self.demo_overlay(id)"));
        assert!(popover.contains(".is_open("));
        assert!(popover.contains(".on_open_change(cx.listener"));
        assert!(!popover.contains(".default_open(self.overlays_open)"));
        for (title, control_count) in [
            ("Usage", 1),
            ("With Arrow", 2),
            ("Interactive Content", 1),
            ("Placement", 1),
            ("Render Function", 1),
            ("Custom Styles", 1),
        ] {
            let section = section_body(popover, title);
            assert_eq!(
                section.matches(".is_open(").count(),
                control_count,
                "{title} must pass each controlled open value to Popover"
            );
            assert_eq!(
                section.matches(".on_open_change(cx.listener").count(),
                control_count,
                "{title} must report each controlled open change"
            );
            assert_eq!(
                section.matches("set_popover_open(").count(),
                control_count,
                "{title} must feed every controlled open callback back into gallery state"
            );
        }
    }

    #[test]
    fn popover_sections_match_pinned_docs_order() {
        assert_eq!(
            section_titles(page_fn(SRC, "popover")),
            [
                "Usage",
                "With Arrow",
                "Interactive Content",
                "Placement",
                "Render Function",
                "Custom Styles",
            ]
        );
    }

    #[test]
    fn popover_missing_dom_apis_are_recorded_honestly() {
        let popover = page_fn(SRC, "popover");
        let render = section_body(popover, "Render Function");
        assert!(render.contains("no content or state render callback"));
        assert!(!render.contains(".render("));

        let custom = section_body(popover, "Custom Styles");
        for builder in [
            ".w(px(224.))",
            ".overflow_hidden()",
            ".rounded(px(12.))",
            ".border_color(custom_border)",
            ".bg(custom_surface)",
            ".shadow(custom_shadow)",
            ".font_family(crate::app::MONO_FONT)",
        ] {
            assert!(
                custom.contains(builder),
                "Custom Styles should use {builder}"
            );
        }
        assert!(!custom.contains("className"));
    }

    #[test]
    fn popover_controls_have_unique_ids_and_stable_triggers() {
        let popover = page_fn(SRC, "popover");
        let owner_ids = [
            "po-arrow",
            "po-arrow-custom",
            "po-interactive",
            "po-pl-top",
            "po-pl-bottom",
            "po-pl-left",
            "po-pl-right",
            "po-render-function",
            "po-custom-styles",
        ];
        let unique_ids: HashSet<_> = owner_ids.into_iter().collect();
        assert_eq!(unique_ids.len(), owner_ids.len());
        let explicit_owner_ids = [
            "po-arrow",
            "po-arrow-custom",
            "po-interactive",
            "po-render-function",
            "po-custom-styles",
        ];
        for id in explicit_owner_ids {
            assert_eq!(
                popover.matches(&format!(".id(\"{id}\")")).count(),
                1,
                "Popover owner id {id} must be explicit and unique"
            );
        }
        assert!(!section_body(popover, "Usage").contains(".id("));
        let placement = section_body(popover, "Placement");
        for id in ["po-pl-top", "po-pl-bottom", "po-pl-left", "po-pl-right"] {
            assert!(
                placement.contains(&format!("(\"{id}\", ")),
                "Placement must include owner id {id}"
            );
        }

        for trigger in [
            "po-trigger",
            "po-arrow-trigger",
            "po-arrow-custom-trigger",
            "po-render-function-trigger",
            "po-custom-styles-trigger",
        ] {
            assert_eq!(
                popover
                    .matches(&format!("h::Button::new(\"{trigger}\")"))
                    .count(),
                1,
                "trigger id {trigger} must be stable in the source"
            );
        }
        assert_eq!(
            placement
                .matches("el_id(format!(\"{id}-trigger\"))")
                .count(),
            1
        );
        assert_eq!(placement.matches(".id(id)").count(), 1);
        assert_eq!(popover.matches("h::Popover::new(").count(), 7);
    }

    #[derive(Default)]
    struct PopoverControlledState {
        usage: bool,
        flags: HashMap<&'static str, bool>,
    }

    impl PopoverControlledState {
        fn on_open_change(&mut self, key: &'static str, open: bool) {
            set_popover_open(&mut self.usage, &mut self.flags, key, open);
        }

        fn is_open(&self, key: &str) -> bool {
            if key == "po-usage" {
                self.usage
            } else {
                self.flags.get(key).copied().unwrap_or(false)
            }
        }
    }

    #[test]
    fn popover_usage_controlled_state_round_trips_open_and_close() {
        let mut state = PopoverControlledState::default();
        state.on_open_change("po-usage", true);
        assert!(state.is_open("po-usage"));
        state.on_open_change("po-usage", false);
        assert!(!state.is_open("po-usage"));
    }

    #[test]
    fn popover_with_arrow_controlled_state_round_trips_each_trigger() {
        let mut state = PopoverControlledState::default();
        for key in ["po-arrow", "po-arrow-custom"] {
            state.on_open_change(key, true);
            assert!(state.is_open(key), "{key} must open independently");
            state.on_open_change(key, false);
            assert!(!state.is_open(key), "{key} must close independently");
        }
        assert_eq!(state.flags.len(), 2);
    }

    #[test]
    fn popover_interactive_content_controlled_state_round_trips_open_and_close() {
        let mut state = PopoverControlledState::default();
        state.on_open_change("po-interactive", true);
        assert!(state.is_open("po-interactive"));
        state.on_open_change("po-interactive", false);
        assert!(!state.is_open("po-interactive"));
    }

    #[test]
    fn popover_placement_controlled_state_round_trips_each_position() {
        let mut state = PopoverControlledState::default();
        for key in ["po-pl-top", "po-pl-bottom", "po-pl-left", "po-pl-right"] {
            state.on_open_change(key, true);
            assert!(state.is_open(key), "{key} must open independently");
            state.on_open_change(key, false);
            assert!(!state.is_open(key), "{key} must close independently");
        }
        assert_eq!(state.flags.len(), 4);
    }

    #[test]
    fn popover_render_function_controlled_state_round_trips_open_and_close() {
        let mut state = PopoverControlledState::default();
        state.on_open_change("po-render-function", true);
        assert!(state.is_open("po-render-function"));
        state.on_open_change("po-render-function", false);
        assert!(!state.is_open("po-render-function"));
    }

    #[test]
    fn popover_custom_styles_controlled_state_round_trips_open_and_close() {
        let mut state = PopoverControlledState::default();
        state.on_open_change("po-custom-styles", true);
        assert!(state.is_open("po-custom-styles"));
        state.on_open_change("po-custom-styles", false);
        assert!(!state.is_open("po-custom-styles"));
    }

    // `example_frame_with_code` centres its demo child, so the hug-content
    // [`col`] leaves a `w-full` Table at its intrinsic width, where each flex
    // row resolves its own column boundaries and the body drifts off the header.
    #[test]
    fn table_examples_give_the_w_full_table_the_frame_width() {
        let page = page_fn(SRC, "table");
        let titles = section_titles(page);
        assert!(!titles.is_empty(), "the Table page lists its examples");
        for title in titles {
            let body = section_body(page, &title);
            let bare = body
                .match_indices("col(")
                .filter(|(at, _)| {
                    body[..*at]
                        .chars()
                        .next_back()
                        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
                })
                .count();
            assert_eq!(
                bare, 0,
                "the {title} example must not wrap a `w-full` Table in the \
                 hug-content `col`: centred in the frame it keeps its intrinsic \
                 width and the body columns drift off their header"
            );
            assert!(
                body.contains("stretch_col("),
                "the {title} example should give the `w-full` Table the preview frame's width"
            );
        }
    }
}
