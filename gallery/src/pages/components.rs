//! Component gallery pages — one page per HeroUI v3 component.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use gpui::{prelude::*, px, AnyElement, Context, SharedString};
use herogpui_components as h;
use herogpui_core::{Color, FieldVariant, Orientation, SelectionMode, Size, SizeXl, Variant};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::para;

thread_local! {
    /// The Form "Server Errors" demo's current `validationErrors` record.
    ///
    /// Held outside the page function on purpose: the page rebuilds its
    /// elements every frame, and the record's *identity* is the contract the
    /// demo exists to show. A record minted per frame would re-arm both
    /// fields on every keystroke; this one is cloned per frame — clones keep
    /// the record's revision — and replaced only by the demo's New response
    /// button, which is a genuinely new response.
    static FORM_SERVER_RECORD: RefCell<Option<h::ValidationErrors>> = const { RefCell::new(None) };
}

macro_rules! component_doc_section {
    (($heading:expr, $body:expr $(,)?)) => {
        ($heading, None, $body, stringify!($body))
    };
    (($heading:expr, $description:literal, $body:expr $(,)?)) => {
        ($heading, Some($description), $body, stringify!($body))
    };
}

macro_rules! component_preview_section {
    (($heading:expr, $body:expr $(,)?), $cx:expr) => {
        if crate::control::section_wanted($heading, $cx) {
            return ($body).into_any_element();
        }
    };
    (($heading:expr, $description:literal, $body:expr $(,)?), $cx:expr) => {
        if crate::control::section_wanted($heading, $cx) {
            return ($body).into_any_element();
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
const DEMO_FIELD_W: f32 = 256.;

fn row(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_wrap()
        .w_full()
        .items_start()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// Wrapping specimen row. Top-aligned so a wrapped caption cannot pull its
/// neighbour down the way `row`'s `items_center` does.
fn spec_row(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_wrap()
        .w_full()
        .items_start()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

fn col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        // Components hug their content in a demo; `full_width` examples opt back
        // in explicitly. Field examples that need a definite width use
        // `field_col` / `demo_field` instead of stretching this helper.
        .items_start()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

/// Column that stretches children to [`DEMO_FIELD_W`]. Use for Input, Select,
/// ComboBox, Autocomplete, TextArea, SearchField, Date/Time and similar
/// specimens — not for `full_width` examples, which must fill the frame.
fn field_col(children: Vec<AnyElement>) -> AnyElement {
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
fn demo_field(el: impl IntoElement) -> AnyElement {
    gpui::div()
        .w(px(DEMO_FIELD_W))
        .flex()
        .flex_col()
        .child(el)
        .into_any_element()
}

fn fixed_demo(width: f32, el: impl IntoElement) -> AnyElement {
    gpui::div().w(px(width)).child(el).into_any_element()
}

/// A labelled specimen — the caption HeroUI puts under each variant.
fn spec(label: &str, el: impl IntoElement, cx: &gpui::App) -> AnyElement {
    let muted = cx.colors().muted;
    gpui::div()
        .flex()
        .flex_col()
        .items_start()
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
fn spec_block(label: &str, el: impl IntoElement, cx: &gpui::App) -> AnyElement {
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
trait IntoVecEls {
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

fn el_id(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

/// The nth of the thousand users v3's virtualization examples list, as
/// `(name, email)`. Same two twenty-name lists, so the rows read the same.
fn virtual_user(i: usize) -> (String, String) {
    const FIRST: [&str; 20] = [
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
    const LAST: [&str; 20] = [
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
fn virtual_picker_items() -> Vec<h::PickerItem> {
    (0..1000)
        .map(|i| h::PickerItem::new(format!("user-{i}"), virtual_user(i).0))
        .collect()
}

/// `languages()` as keyed items, for the Autocomplete and ComboBox demos: the
/// slug is the stable key the selection, `disabledKeys` and the form value
/// address, the display name the label the filtering, input text and
/// rendering use.
fn language_items() -> Vec<h::PickerItem> {
    languages()
        .into_iter()
        .map(|label| h::PickerItem::new(label.to_lowercase(), label))
        .collect()
}

/// A thousand names, for the pickers' virtualization demos.
fn virtual_names() -> Vec<SharedString> {
    (0..1000)
        .map(|i| SharedString::from(virtual_user(i).0))
        .collect()
}

/// A thousand list rows, for the virtualization demos.
fn virtual_users() -> Vec<h::ListBoxItem> {
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
fn virtual_users_described() -> Vec<h::ListBoxItem> {
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

/// Adapter: turn a `cx.listener` over `&bool` into a `Fn(bool, ..)` callback.
fn bool_cb(
    l: impl Fn(&bool, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(bool, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn f32_cb(
    l: impl Fn(&f32, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(f32, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn f64_cb(
    l: impl Fn(&f64, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(f64, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn opt_color_cb(
    l: impl Fn(&Option<h::PickerColor>, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(Option<h::PickerColor>, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn sort_cb(
    l: impl Fn(&h::SortDescriptor, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(h::SortDescriptor, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn date_cb(
    l: impl Fn(&h::Date, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(h::Date, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn shadow_vis_cb(
    l: impl Fn(&h::ScrollShadowVisibility, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(h::ScrollShadowVisibility, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn opt_date_cb(
    l: impl Fn(&Option<h::Date>, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(Option<h::Date>, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn usize_cb(
    l: impl Fn(&usize, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(usize, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn opt_usize_cb(
    l: impl Fn(&Option<usize>, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(Option<usize>, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn color_cb(
    l: impl Fn(&h::PickerColor, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(h::PickerColor, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

fn opt_time_cb(
    l: impl Fn(&Option<h::Time>, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl Fn(Option<h::Time>, &mut gpui::Window, &mut gpui::App) + 'static {
    move |v, w, cx| l(&v, w, cx)
}

/// The `ToastViewport` mount every application needs once.
const TOAST_SETUP: &str = r#"// Once, in the shell:
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

fn overlay_min_h(mut frame: gpui::Div, open: bool, height: f32) -> gpui::Div {
    if open {
        frame = frame.min_h(px(height));
    }
    frame
}

fn set_popover_open(
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
fn overlay_demo(
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
            .items_start()
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
fn palette() -> Vec<h::PickerColor> {
    [
        "#0085F5", "#17C964", "#F5A524", "#F31260", "#7828C8", "#0E8AAA", "#71717A", "#18181B",
    ]
    .into_iter()
    .filter_map(h::PickerColor::from_hex)
    .collect()
}

// ---------------------------------------------------------------------------
// Buttons
// ---------------------------------------------------------------------------

impl Gallery {
    pub fn page_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let clicks = self.button_clicks;
        let pending_foreground = cx.colors().accent.foreground;
        component_doc_page!(
            "Button",
            crate::pages::Page::Button.description(),
            crate::pages::Page::Button.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Button::new("btn-usage")
                        .label("Click me")
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(Variant::ALL
                        .iter()
                        .map(|v| {
                            h::Button::new(el_id(format!("btn-v-{v:?}")))
                                .label(v.label())
                                .variant(*v)
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| {
                            h::Button::new(el_id(format!("btn-s-{s:?}")))
                                .label(s.label())
                                .size(*s)
                        })
                        .els()),
                ),
                (
                    "With icons",
                    row(vec![
                        h::Button::new("btn-i-1")
                            .child(icon(h::icons::SEARCH, cx))
                            .child("Search")
                            .into_any_element(),
                        h::Button::new("btn-i-2")
                            .variant(Variant::Secondary)
                            .child(icon(h::icons::PLUS, cx))
                            .child("Add member")
                            .into_any_element(),
                        h::Button::new("btn-i-3")
                            .variant(Variant::Danger)
                            .child(icon(h::icons::CLOSE, cx))
                            .child("Delete")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Icon only",
                    row(vec![
                        h::Button::new("btn-io-1")
                            .is_icon_only(true)
                            .variant(Variant::Tertiary)
                            .child(icon(h::icons::ELLIPSIS, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-2")
                            .is_icon_only(true)
                            .variant(Variant::Secondary)
                            .child(icon(h::icons::PLUS, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-3")
                            .is_icon_only(true)
                            .variant(Variant::Danger)
                            .child(icon(h::icons::CLOSE, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Pending & disabled",
                    row(vec![
                        h::Button::new("btn-pending")
                            .is_pending(true)
                            .content(move |state| {
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .when(state.is_pending, |content| {
                                        content.child(
                                            h::Spinner::new("btn-pending-spinner")
                                                .size(h::SpinnerSize::Sm)
                                                .current_color(pending_foreground),
                                        )
                                    })
                                    .child(if state.is_pending {
                                        "Uploading"
                                    } else {
                                        "Upload"
                                    })
                                    .into_any_element()
                            })
                            .into_any_element(),
                        h::Button::new("btn-disabled")
                            .label("Disabled")
                            .is_disabled(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Full width",
                    col(vec![h::Button::new("btn-full")
                        .label("Continue")
                        .full_width(true)
                        .into_any_element()]),
                ),
                (
                    "Social Buttons", "v3 stacks full-width tertiary buttons behind a leading brand mark. The marks are trademarks, so this port shows the same layout with its own glyphs.",
                    col(vec![
                        gpui::div()
                            .flex()
                            .flex_col()
                            .w(px(280.))
                            .gap(px(12.))
                            .child(
                                h::Button::new("btn-soc-1")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::MAIL, cx))
                                    .child("Sign in with Email"),
                            )
                            .child(
                                h::Button::new("btn-soc-2")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::KEY, cx))
                                    .child("Sign in with a passkey"),
                            )
                            .child(
                                h::Button::new("btn-soc-3")
                                    .variant(Variant::Tertiary)
                                    .full_width(true)
                                    .child(icon(h::icons::GLOBE, cx))
                                    .child("Single sign-on"),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Press handler",
                    col(vec![h::Button::new("btn-press")
                        .label(format!("Pressed {clicks} times"))
                        .on_press(cx.listener(|this, _, _, cx| {
                            this.button_clicks += 1;
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_button_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Button Group",
            crate::pages::Page::ButtonGroup.description(),
            crate::pages::Page::ButtonGroup.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::ButtonGroup::new()
                        .separators(true)
                        .button(h::Button::new("bgu-1").label("Merge pull request"))
                        .button(
                            h::Button::new("bgu-2")
                                .is_icon_only(true)
                                .child(icon(h::icons::CHEVRON_DOWN, cx)),
                        )
                        .into_any_element()]),
                ),
                (
                    "Merged",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        .separators(true)
                        .button(h::Button::new("bg-1").label("Day"))
                        .button(h::Button::new("bg-2").label("Week"))
                        .button(h::Button::new("bg-3").label("Month"))
                        .into_any_element()]),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .map(|sz| {
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .size(*sz)
                                .separators(true)
                                .button(
                                    h::Button::new(el_id(format!("bgs-{sz:?}-1"))).label("Left"),
                                )
                                .button(
                                    h::Button::new(el_id(format!("bgs-{sz:?}-2"))).label("Middle"),
                                )
                                .button(
                                    h::Button::new(el_id(format!("bgs-{sz:?}-3"))).label("Right"),
                                )
                        })
                        .els()),
                ),
                (
                    "With Icons",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Tertiary)
                        .separators(true)
                        .button(
                            h::Button::new("bgi-1")
                                .child(icon(h::icons::COPY, cx))
                                .child("Fork"),
                        )
                        .button(
                            h::Button::new("bgi-2")
                                .child(icon(h::icons::PLUS, cx))
                                .child("Star"),
                        )
                        .button(
                            h::Button::new("bgi-3")
                                .is_icon_only(true)
                                .child(icon(h::icons::ELLIPSIS, cx)),
                        )
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(Variant::GROUP
                        .iter()
                        .map(|v| {
                            h::ButtonGroup::new()
                                .variant(*v)
                                .separators(true)
                                .button(h::Button::new(el_id(format!("bgv-{v:?}-1"))).label("One"))
                                .button(h::Button::new(el_id(format!("bgv-{v:?}-2"))).label("Two"))
                        })
                        .els()),
                ),
                (
                    "Orientation",
                    row(vec![
                        h::ButtonGroup::new()
                            .variant(Variant::Secondary)
                            .separators(true)
                            .button(h::Button::new("bgo-l").label("Left"))
                            .button(h::Button::new("bgo-c").label("Center"))
                            .button(h::Button::new("bgo-r").label("Right"))
                            .into_any_element(),
                        h::ButtonGroup::new()
                            .variant(Variant::Secondary)
                            .orientation(Orientation::Vertical)
                            .separators(true)
                            .button(h::Button::new("bgv-top").label("Top"))
                            .button(h::Button::new("bgv-mid").label("Middle"))
                            .button(h::Button::new("bgv-bot").label("Bottom"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
                        .w_full()
                        .child(
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .full_width(true)
                                .separators(true)
                                .button(h::Button::new("bgf-1").label("Cancel"))
                                .button(h::Button::new("bgf-2").label("Save draft"))
                                .button(h::Button::new("bgf-3").label("Publish")),
                        )
                        .into_any_element()]),
                ),
                (
                    "Without Separator",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        // v3: omit the `<ButtonGroup.Separator />` child
                        // composition — the port's default draws no dividers.
                        .button(h::Button::new("bgn-1").label("One"))
                        .button(h::Button::new("bgn-2").label("Two"))
                        .button(h::Button::new("bgn-3").label("Three"))
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        .is_disabled(true)
                        .separators(true)
                        .button(h::Button::new("bgd2-1").label("One"))
                        .button(h::Button::new("bgd2-2").label("Two"))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_close_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let presses = self.close_button_presses;
        component_doc_page!(
            "Close Button",
            crate::pages::Page::CloseButton.description(),
            crate::pages::Page::CloseButton.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::CloseButton::new("cb-usage").into_any_element()]),
                ),
                (
                    "Interactive",
                    col(vec![
                        h::CloseButton::new("cb-press")
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.close_button_presses += 1;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Pressed {presses} times"), cx),
                    ]),
                ),
                (
                    "With Custom Icon",
                    row(vec![spec(
                        "Custom icon",
                        h::CloseButton::new("cb-icon-1").icon(icon(h::icons::CLOSE_CIRCLE, cx)),
                        cx,
                    ),]),
                ),
                (
                    "Render Function", "Hover, focus, or press the button to drive the custom icon from its live render state.",
                    {
                        let muted = cx.colors().muted;
                        let foreground = cx.colors().foreground;
                        col(vec![h::CloseButton::new("cb-render-state")
                            .content(move |state| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_pressed {
                                        h::icons::CLOSE_CIRCLE
                                    } else {
                                        h::icons::CLOSE
                                    })
                                    .text_color(if state.is_disabled {
                                        muted
                                    } else if state.is_hovered || state.is_focused {
                                        foreground
                                    } else {
                                        muted
                                    })
                                    .into_any_element()
                            })
                            .into_any_element()])
                    }
                ),
                (
                    "Disabled",
                    row(vec![h::CloseButton::new("cb-disabled")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_toggle_button(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let liked = self.toggle_like;
        let single = self.toggle_single.clone();
        let multiple = self.toggle_multiple.clone();
        component_doc_page!(
            "Toggle Button",
            crate::pages::Page::ToggleButton.description(),
            crate::pages::Page::ToggleButton.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::ToggleButton::new("tb-usage")
                        .label("Bold")
                        .into_any_element()]),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|sz| {
                            h::ToggleButton::new(el_id(format!("tb-sz-{sz:?}")))
                                .label(sz.label())
                                .size(*sz)
                                .default_selected(true)
                        })
                        .els()),
                ),
                (
                    "Icon Only",
                    row(vec![
                        h::ToggleButton::new("tb-io-1")
                            .is_icon_only(true)
                            .default_selected(true)
                            .child(icon(h::icons::EYE, cx))
                            .into_any_element(),
                        h::ToggleButton::new("tb-io-2")
                            .is_icon_only(true)
                            .child(icon(h::icons::COPY, cx))
                            .into_any_element(),
                        h::ToggleButton::new("tb-io-3")
                            .is_icon_only(true)
                            .child(icon(h::icons::SEARCH, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Disabled",
                    row(vec![
                        h::ToggleButton::new("tb-dis-1")
                            .label("Off")
                            .is_disabled(true)
                            .into_any_element(),
                        h::ToggleButton::new("tb-dis-2")
                            .label("On")
                            .is_selected(true)
                            .is_disabled(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ToggleButton::new("tb-like")
                            .label(if liked { "Liked" } else { "Like" })
                            .is_selected(liked)
                            .child(icon(
                                if liked {
                                    h::icons::HEART_FILL
                                } else {
                                    h::icons::HEART
                                },
                                cx,
                            ))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.toggle_like = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(
                            if liked {
                                "Status: selected"
                            } else {
                                "Status: not selected"
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Single selection",
                    row(vec![h::ToggleButtonGroup::new("toggle-single")
                        .selection_mode(SelectionMode::Single)
                        .separators(true)
                        .selected_keys(single.into_iter().collect::<Vec<_>>())
                        .child_toggle(h::ToggleButton::new("tb-left").key("left").label("Left"))
                        .child_toggle(
                            h::ToggleButton::new("tb-center")
                                .key("center")
                                .label("Center"),
                        )
                        .child_toggle(h::ToggleButton::new("tb-right").key("right").label("Right"))
                        .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.toggle_single = keys.first().cloned();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    row(vec![h::ToggleButtonGroup::new("toggle-multiple")
                        .selection_mode(SelectionMode::Multiple)
                        .separators(true)
                        .selected_keys(multiple.iter().cloned().collect::<Vec<_>>())
                        .child_toggle(h::ToggleButton::new("tb-bold").key("bold").label("Bold"))
                        .child_toggle(
                            h::ToggleButton::new("tb-italic")
                                .key("italic")
                                .label("Italic"),
                        )
                        .child_toggle(
                            h::ToggleButton::new("tb-underline")
                                .key("underline")
                                .label("Underline"),
                        )
                        .on_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.toggle_multiple = keys.iter().cloned().collect();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(vec![
                        h::ToggleButton::new("tb-v-default")
                            .label("Default")
                            .default_selected(true)
                            .into_any_element(),
                        h::ToggleButton::new("tb-v-ghost")
                            .label("Ghost")
                            .variant(h::ToggleVariant::Ghost)
                            .default_selected(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Orientation",
                    row(vec![
                        h::ToggleButtonGroup::new("toggle-orientation-horizontal")
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbo-h-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-h-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-h-3").label("Month"))
                            .into_any_element(),
                        h::ToggleButtonGroup::new("toggle-orientation-vertical")
                            .orientation(Orientation::Vertical)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbo-v-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-v-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-v-3").label("Month"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
                        .w_full()
                        .child(
                            h::ToggleButtonGroup::new("toggle-full-width")
                                .full_width(true)
                                .separators(true)
                                .child_toggle(h::ToggleButton::new("tbf-1").label("Left"))
                                .child_toggle(h::ToggleButton::new("tbf-2").label("Center"))
                                .child_toggle(h::ToggleButton::new("tbf-3").label("Right")),
                        )
                        .into_any_element()]),
                ),
                (
                    "Without Separator",
                    row(vec![h::ToggleButtonGroup::new("toggle-without-separator")
                        // v3: omit the `<ToggleButtonGroup.Separator />` child
                        // composition — the port's default draws no dividers.
                        .child_toggle(h::ToggleButton::new("tbn-1").label("One"))
                        .child_toggle(h::ToggleButton::new("tbn-2").label("Two"))
                        .child_toggle(h::ToggleButton::new("tbn-3").label("Three"))
                        .into_any_element()]),
                ),
                (
                    "Selection Mode", "Single: exactly one member stays selected.",
                    col(vec![
                        h::ToggleButtonGroup::new("toggle-selection-single")
                            .selection_mode(SelectionMode::Single)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbsm-s-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-3").key("c").label("C"))
                            .into_any_element(),
                        para("Multiple: any number of members can be selected.", cx),
                        h::ToggleButtonGroup::new("toggle-selection-multiple")
                            .selection_mode(SelectionMode::Multiple)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbsm-m-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-3").key("c").label("C"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Default Selected Keys", "Uncontrolled: `defaultSelectedKeys` seeds the group's own selection, and the group keeps ownership from there — clicking a member still toggles it.",
                    col(vec![
                        h::ToggleButtonGroup::new("toggle-default-single")
                            .selection_mode(SelectionMode::Single)
                            .separators(true)
                            .default_selected_keys(["center"])
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-left").key("left").label("Left")
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-center")
                                    .key("center")
                                    .label("Center"),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-s-right")
                                    .key("right")
                                    .label("Right"),
                            )
                            .into_any_element(),
                        para("Multiple: any number of members can be selected.", cx),
                        h::ToggleButtonGroup::new("toggle-default-multiple")
                            .selection_mode(SelectionMode::Multiple)
                            .separators(true)
                            .default_selected_keys(["bold", "underline"])
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-bold").key("bold").label("Bold")
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-italic")
                                    .key("italic")
                                    .label("Italic"),
                            )
                            .child_toggle(
                                h::ToggleButton::new("tbu-m-underline")
                                    .key("underline")
                                    .label("Underline"),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical & detached",
                    row(vec![
                        h::ToggleButtonGroup::new("toggle-vertical")
                            .orientation(Orientation::Vertical)
                            .separators(true)
                            .child_toggle(h::ToggleButton::new("tbv-1").label("Top"))
                            .child_toggle(h::ToggleButton::new("tbv-2").label("Bottom"))
                            .into_any_element(),
                        h::ToggleButtonGroup::new("toggle-detached")
                            .is_detached(true)
                            .child_toggle(h::ToggleButton::new("tbd-1").label("A"))
                            .child_toggle(h::ToggleButton::new("tbd-2").label("B"))
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Collections
    // -----------------------------------------------------------------------

    pub fn page_dropdown(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.dropdown_open;
        let selected = self
            .dropdown_selected
            .clone()
            .unwrap_or_else(|| SharedString::from("none"));
        let items = vec![
            h::MenuItem::new("new", "New file").shortcut("Ctrl N"),
            h::MenuItem::new("copy", "Copy link").shortcut("Ctrl C"),
            h::MenuItem::Separator,
            h::MenuItem::new("delete", "Delete file").danger(),
        ];
        // A `MenuItem` is moved into the menu that shows it, so each demo builds
        // its own list.
        let plain = || {
            vec![
                h::MenuItem::new("new", "New file"),
                h::MenuItem::new("open", "Open file"),
                h::MenuItem::new("save", "Save"),
            ]
        };
        let dd_multi = self.dropdown_multi.clone();
        component_doc_page!(
            "Dropdown",
            crate::pages::Page::Dropdown.description(),
            crate::pages::Page::Dropdown.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Dropdown::new(
                            "dd-trigger-dd",
                            h::Button::new("dd-trigger")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            items,
                            is_open,
                        )
                        .id("dd-trigger-dd")
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
                        .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.dropdown_selected = Some(key.clone());
                            this.dropdown_open = false;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(&format!("Last action: {selected}"), cx),
                    ]),
                ),
                (
                    "With Icons",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-icons-dd",
                        h::Button::new("dd-icons")
                            .label("File")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("new", "New file").icon(h::icons::PLUS),
                            h::MenuItem::new("copy", "Copy").icon(h::icons::COPY),
                            h::MenuItem::new("delete", "Delete")
                                .icon(h::icons::CLOSE)
                                .danger(),
                        ],
                    )
                    .id("dd-icons-dd")
                    .into_any_element()]),
                ),
                (
                    "With Descriptions",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-desc-dd",
                        h::Button::new("dd-desc")
                            .label("Merge")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("merge", "Create a merge commit").description(
                                "All commits from this branch are added to the base branch",
                            ),
                            h::MenuItem::new("squash", "Squash and merge")
                                .description("The commits are combined into one"),
                            h::MenuItem::new("rebase", "Rebase and merge")
                                .description("The commits are rebased onto the base branch"),
                        ],
                    )
                    .id("dd-desc-dd")
                    .into_any_element()]),
                ),
                (
                    "With Disabled Items",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-disabled-dd",
                        h::Button::new("dd-disabled")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        plain(),
                    )
                    .id("dd-disabled-dd")
                    .disabled_keys([SharedString::from("save")])
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-sections-dd",
                        h::Button::new("dd-sections")
                            .label("Actions")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::SectionLabel("File".into()),
                            h::MenuItem::new("new", "New file"),
                            h::MenuItem::new("open", "Open file"),
                            h::MenuItem::Separator,
                            h::MenuItem::SectionLabel("Danger".into()),
                            h::MenuItem::new("delete", "Delete").danger(),
                        ],
                    )
                    .id("dd-sections-dd")
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Dropdown::new(
                            "dd-controlled-dd",
                            h::Button::new("dd-controlled")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .id("dd-controlled-dd")
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
                        .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                            this.dropdown_selected = Some(key.clone());
                            this.dropdown_open = false;
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(&format!("Last action: {selected}"), cx),
                    ]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("dd-open-btn")
                                .label(if is_open { "Close menu" } else { "Open menu" })
                                .size(Size::Sm)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.dropdown_open = !this.dropdown_open;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if is_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Dropdown::new(
                            "dd-open-dd",
                            h::Button::new("dd-open")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            plain(),
                            is_open,
                        )
                        .id("dd-open-dd")
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Single Selection",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-single-dd",
                        h::Button::new("dd-single")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                            h::MenuItem::new("size", "Size"),
                        ],
                    )
                    .id("dd-single-dd")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("date")])
                    .into_any_element()]),
                ),
                (
                    "Single With Custom Indicator",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-single-ind-dd",
                        h::Button::new("dd-single-ind")
                            .label("Sort by")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                        ],
                    )
                    .id("dd-single-ind-dd")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("name")])
                    .indicator(h::IndicatorKind::Dot)
                    .into_any_element()]),
                ),
                (
                    "Render Props", "The composed Dropdown forwards item and indicator render state into the live menu. Open it and choose rows to watch the selection move.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-render-props-dd",
                            h::Button::new("dd-render-props")
                                .label("Render state")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("name", "Name"),
                                h::MenuItem::new("date", "Date"),
                                h::MenuItem::new("size", "Size"),
                            ],
                        )
                        .selection_mode(SelectionMode::Multiple)
                        .default_selected_keys([SharedString::from("name")])
                        .item_content(|key, state| {
                            gpui::div()
                                .child(format!(
                                    "{key}: {}",
                                    if state.is_selected {
                                        "selected"
                                    } else {
                                        "idle"
                                    }
                                ))
                                .into_any_element()
                        })
                        .indicator_content(|_, selected, _| {
                            gpui::div()
                                .w(px(16.))
                                .child(if selected { "✓" } else { "" })
                                .into_any_element()
                        })
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Section Level Selection",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-section-sel-dd",
                        h::Button::new("dd-section-sel")
                            .label("View")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::SectionLabel("Sort".into()),
                            h::MenuItem::new("name", "Name"),
                            h::MenuItem::new("date", "Date"),
                            h::MenuItem::Separator,
                            h::MenuItem::SectionLabel("Show".into()),
                            h::MenuItem::new("hidden", "Hidden files"),
                        ],
                    )
                    .id("dd-section-sel-dd")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(dd_multi)
                    .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.dropdown_multi = keys.to_vec();
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "With Keyboard Shortcuts",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-shortcuts-dd",
                        h::Button::new("dd-shortcuts")
                            .label("Edit")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("cut", "Cut").shortcut("Ctrl X"),
                            h::MenuItem::new("copy", "Copy").shortcut("Ctrl C"),
                            h::MenuItem::new("paste", "Paste").shortcut("Ctrl V"),
                        ],
                    )
                    .id("dd-shortcuts-dd")
                    .into_any_element()]),
                ),
                (
                    "With Submenus",
                    col(vec![h::Dropdown::uncontrolled(
                        "dd-submenu-dd",
                        h::Button::new("dd-submenu")
                            .label("Share")
                            .variant(Variant::Secondary),
                        vec![
                            h::MenuItem::new("link", "Copy link"),
                            h::MenuItem::new("email", "Email"),
                            h::MenuItem::new("other", "Other").submenu(vec![
                                h::MenuItem::new("sms", "SMS"),
                                h::MenuItem::new("airdrop", "AirDrop"),
                                h::MenuItem::new("more", "More\u{2026}"),
                            ]),
                        ],
                    )
                    .id("dd-submenu-dd")
                    .into_any_element()]),
                ),
                (
                    "With Custom Submenu Indicator", "`Dropdown.SubmenuIndicator` is the chevron on a row that opens another panel; hover the row to open it.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-submenu-ind-dd",
                            h::Button::new("dd-submenu-ind")
                                .label("More")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("profile", "Profile"),
                                h::MenuItem::new("workspace", "Workspace").submenu(vec![
                                    h::MenuItem::new("members", "Members"),
                                    h::MenuItem::new("billing", "Billing"),
                                ]),
                            ],
                        )
                        .id("dd-submenu-ind-dd")
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Trigger",
                    col(vec![h::Dropdown::uncontrolled(
                        "Jane Doe-dd",
                        h::Avatar::new("dd-trigger-avatar").name("Jane Doe"),
                        vec![
                            h::MenuItem::new("profile", "Profile"),
                            h::MenuItem::new("settings", "Settings"),
                            h::MenuItem::Separator,
                            h::MenuItem::new("logout", "Log out").danger(),
                        ],
                    )
                    .id("Jane Doe-dd")
                    .into_any_element()]),
                ),
                (
                    "Long Press Trigger", "Hold the button for half a second.",
                    col(vec![
                        h::Dropdown::uncontrolled(
                            "dd-long-dd",
                            h::Button::new("dd-long")
                                .label("Long press")
                                .variant(Variant::Secondary),
                            plain(),
                        )
                        .id("dd-long-dd")
                        .trigger(h::DropdownTrigger::LongPress)
                        .into_any_element(),
                    ]),
                ),
                (
                    "Multiple selection",
                    col(vec![
                        h::Dropdown::new(
                            "dd-multi-dd",
                            h::Button::new("dd-multi-trigger")
                                .label("Columns")
                                .variant(Variant::Secondary),
                            vec![
                                h::MenuItem::new("name", "Name"),
                                h::MenuItem::new("role", "Role"),
                                h::MenuItem::new("status", "Status"),
                            ],
                            self.dropdown_open,
                        )
                        .id("dd-multi-trigger-dd")
                        .selection_mode(SelectionMode::Multiple)
                        .selected_keys(self.dropdown_multi.clone())
                        .disabled_keys(vec!["status"])
                        .on_selection_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.dropdown_multi = keys.to_vec();
                            cx.notify();
                        }))
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.dropdown_open = *open;
                            cx.notify();
                        })))
                        .into_any_element(),
                        para(
                            &format!("Showing {} columns", self.dropdown_multi.len()),
                            cx,
                        ),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_list_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selection = self.list_selection.clone();
        let items = vec![
            h::ListBoxItem::section("Mail"),
            h::ListBoxItem::new("inbox", "Inbox").description("24 unread"),
            h::ListBoxItem::new("sent", "Sent"),
            h::ListBoxItem::new("drafts", "Drafts").is_disabled(true),
            h::ListBoxItem::separator(),
            h::ListBoxItem::new("trash", "Move to trash").danger(),
        ];
        component_doc_page!(
            "List Box",
            crate::pages::Page::ListBox.description(),
            crate::pages::Page::ListBox.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(220.))
                        .child(h::ListBox::new(
                            "lb-usage",
                            vec![
                                h::ListBoxItem::new("inbox", "Inbox"),
                                h::ListBoxItem::new("sent", "Sent"),
                                h::ListBoxItem::new("drafts", "Drafts"),
                            ],
                        )
                        .selection_mode(SelectionMode::Single))
                        .into_any_element()]),
                ),
                (
                    "With Disabled Items",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-disabled",
                                vec![
                                    h::ListBoxItem::new("inbox", "Inbox"),
                                    h::ListBoxItem::new("sent", "Sent"),
                                    h::ListBoxItem::new("drafts", "Drafts"),
                                ],
                            )
                            .selection_mode(SelectionMode::None)
                            .disabled_keys([SharedString::from("drafts")]),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(h::ListBox::new(
                            "lb-sections",
                            vec![
                                h::ListBoxItem::section("Mail"),
                                h::ListBoxItem::new("inbox", "Inbox"),
                                h::ListBoxItem::new("sent", "Sent"),
                                h::ListBoxItem::separator(),
                                h::ListBoxItem::section("Archive"),
                                h::ListBoxItem::new("2024", "2024"),
                                h::ListBoxItem::new("2025", "2025"),
                            ],
                        ))
                        .into_any_element()]),
                ),
                (
                    "Multi Select",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new(
                                "lb-multi-select",
                                vec![
                                    h::ListBoxItem::new("inbox", "Inbox"),
                                    h::ListBoxItem::new("sent", "Sent"),
                                    h::ListBoxItem::new("spam", "Spam"),
                                ],
                            )
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(selection.iter().cloned())
                            .on_selection_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.list_selection = keys.clone();
                                    cx.notify();
                                },
                            )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-controlled",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                        h::ListBoxItem::new("spam", "Spam"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Multiple)
                                .selected_keys(selection.iter().cloned())
                                // `onAction` fires on a press, selection or not.
                                .on_action(cx.listener(|this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("lb-action", key.to_string());
                                    cx.notify();
                                }))
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                            )
                            .into_any_element(),
                        para(&format!("{} selected", selection.len()), cx),
                    ]),
                ),
                (
                    "Disallow Empty Selection", "The inherited React Aria policy keeps the final selected row selected, including when Escape would otherwise clear the collection.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-keep-selection",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Single)
                                .default_selected_keys([SharedString::from("inbox")])
                                .disallow_empty_selection(true),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Escape Key Behavior", "Press Escape while this list is focused. The `None` policy preserves the selection and leaves Escape available to an enclosing surface.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-escape-none",
                                    vec![
                                        h::ListBoxItem::new("inbox", "Inbox"),
                                        h::ListBoxItem::new("sent", "Sent"),
                                    ],
                                )
                                .selection_mode(SelectionMode::Single)
                                .default_selected_keys([SharedString::from("inbox")])
                                .escape_key_behavior(h::EscapeKeyBehavior::None),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Virtualization", "v3 wraps the list in React Aria's `Virtualizer` with `ListLayout`; `row_height` carries that here, because a fixed row height is what lets the geometry be computed instead of laid out. gpui's `uniform_list` then builds only the rows in view — one thousand users, fifty pixels each.",
                    col(vec![
                        gpui::div()
                            .w(px(300.))
                            .child(
                                h::ListBox::new("lb-virtual", virtual_users())
                                    .selection_mode(SelectionMode::None)
                                    .row_height(px(50.))
                                    .max_h(px(400.)),
                            )
                            .into_any_element(),
                        gpui::div()
                            .w(px(300.))
                            .child(
                                h::ListBox::new("lb-virtual-var", virtual_users_described())
                                    .selection_mode(SelectionMode::None)
                                    .estimated_row_height(px(44.))
                                    .heading_height(px(28.))
                                    .max_h(px(400.)),
                            )
                            .into_any_element(),
                        para(
                            "`estimated_row_height` is the other half: rows that are \
                             *not* all one height, measured as they are built. Every \
                             third row here carries a description, so it is taller.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Check Icon", "v3 replaces `ListBox.ItemIndicator`. A row's `variant` is what carries the indicator style here, so the danger row below shows the same tick in its own colour.",
                    col(vec![
                        gpui::div()
                            .w(px(280.))
                            .child(
                                h::ListBox::new(
                                    "lb-check",
                                    vec![
                                        h::ListBoxItem::new("keep", "Keep"),
                                        h::ListBoxItem::new("delete", "Delete").danger(),
                                    ],
                                )
                                .selection_mode(SelectionMode::Multiple)
                                .default_selected_keys([SharedString::from("keep")]),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Single selection",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new("lb-single", items.clone())
                                .selection_mode(SelectionMode::Single)
                                .selected_keys(selection.iter().cloned())
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new("lb-multi", items)
                                .selection_mode(SelectionMode::Multiple)
                                .selected_keys(selection.iter().cloned())
                                .on_selection_change(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.list_selection = keys.clone();
                                        cx.notify();
                                    },
                                )),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_tag_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let tag_keys = self.tags.clone();
        // Each demo needs its own list, so this is a factory rather than one
        // vector: a `Tag` is moved into the group that shows it.
        let tags = move || -> Vec<h::Tag> {
            tag_keys
                .iter()
                .map(|k| h::Tag::new(k.clone(), title_case(k)))
                .collect()
        };
        let selection = self.tag_selection.clone();
        let tag_selection = selection.clone();
        component_doc_page!(
            "Tag Group",
            crate::pages::Page::TagGroup.description(),
            crate::pages::Page::TagGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::TagGroup::new("tg-usage", tags())
                        .label("Skills")
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![
                        h::TagGroup::new("tg-disabled", tags())
                            .label("Skills")
                            .is_disabled(true)
                            .into_any_element(),
                        h::TagGroup::new("tg-disabled-keys", tags())
                            .label("Some disabled")
                            // `disabledKeys` disables individual tags rather
                            // than the whole group.
                            .disabled_keys([SharedString::from("rust")])
                            .into_any_element(),
                    ]),
                ),
                (
                    "Selection Modes",
                    col(vec![
                        spec(
                            "Single",
                            h::TagGroup::new("tg-single", tags())
                                .selection_mode(SelectionMode::Single),
                            cx,
                        ),
                        spec(
                            "Multiple",
                            h::TagGroup::new("tg-multiple", tags())
                                .selection_mode(SelectionMode::Multiple),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::TagGroup::new("tg-controlled", tags())
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(tag_selection.iter().cloned())
                            .on_selection_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.tag_selection = keys.clone();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", tag_selection.len()), cx),
                    ]),
                ),
                (
                    "Disallow Empty Selection",
                    col(vec![h::TagGroup::new(
                        "tg-disallow-empty",
                        vec![h::Tag::new("design", "Design"), h::Tag::new("code", "Code")],
                    )
                    .label("At least one skill")
                    .selection_mode(SelectionMode::Single)
                    .default_selected_keys([SharedString::from("design")])
                    .disallow_empty_selection(true)
                    .into_any_element()]),
                ),
                (
                    "Escape Key Behavior", "With `None`, Escape preserves the selected tag and bubbles to any enclosing interaction that handles it.",
                    col(vec![
                        h::TagGroup::new(
                            "tg-escape-none",
                            vec![h::Tag::new("design", "Design"), h::Tag::new("code", "Code")],
                        )
                        .label("Skills")
                        .selection_mode(SelectionMode::Single)
                        .default_selected_keys([SharedString::from("design")])
                        .escape_key_behavior(h::EscapeKeyBehavior::None)
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Error Message",
                    col(vec![h::TagGroup::new("tg-error", tags())
                        .label("Skills")
                        .description("Pick at least one")
                        .into_any_element()]),
                ),
                (
                    "With List Data",
                    col(vec![h::TagGroup::new(
                        "tg-list",
                        ["Design", "Research", "Writing", "Support", "Ops"]
                            .into_iter()
                            .map(|name| h::Tag::new(name.to_lowercase(), name))
                            .collect(),
                    )
                    .label("Teams")
                    .into_any_element()]),
                ),
                (
                    "With Prefix",
                    col(vec![h::TagGroup::new(
                        "tg-prefix",
                        vec![
                            h::Tag::new("rust", "Rust").icon(h::icons::CHECK),
                            h::Tag::new("gpui", "GPUI").icon(h::icons::CHECK),
                        ],
                    )
                    .label("Verified")
                    .into_any_element()]),
                ),
                (
                    "With Remove Button",
                    col(vec![
                        spec(
                            "Default remove button",
                            h::TagGroup::new("tg-remove-button", tags())
                                .label("Skills")
                                .on_remove(cx.listener(
                                    |this, keys: &HashSet<SharedString>, _, cx| {
                                        this.tags.retain(|key| !keys.contains(key));
                                        this.tag_selection.retain(|key| !keys.contains(key));
                                        cx.notify();
                                    },
                                )),
                            cx,
                        ),
                        spec(
                            "Custom remove button",
                            {
                                let custom_tags: Vec<h::Tag> = tags()
                                    .into_iter()
                                    .map(|tag| {
                                        tag.remove_content(|| {
                                            gpui::div().child("−").into_any_element()
                                        })
                                    })
                                    .collect();
                                h::TagGroup::new("tg-custom-remove-button", custom_tags)
                                    .label("Skills")
                                    .on_remove(cx.listener(
                                        |this, keys: &HashSet<SharedString>, _, cx| {
                                            this.tags.retain(|key| !keys.contains(key));
                                            this.tag_selection.retain(|key| !keys.contains(key));
                                            cx.notify();
                                        },
                                    ))
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Removable",
                    col(vec![h::TagGroup::new("tg-remove", tags())
                        .label("Team")
                        .description("Remove a tag to see the group update.")
                        .empty_state("All tags removed")
                        .on_remove(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.tags.retain(|key| !keys.contains(key));
                            this.tag_selection.retain(|key| !keys.contains(key));
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Selectable",
                    col(vec![h::TagGroup::new("tg-select", tags())
                        .selection_mode(SelectionMode::Multiple)
                        .selected_keys(selection.iter().cloned())
                        .on_selection_change(cx.listener(
                            |this, keys: &HashSet<SharedString>, _, cx| {
                                this.tag_selection = keys.clone();
                                cx.notify();
                            },
                        ))
                        .into_any_element()]),
                ),
                (
                    "Variants & sizes",
                    col(vec![
                        h::TagGroup::new("tg-default", tags()).into_any_element(),
                        h::TagGroup::new("tg-surface", tags())
                            .variant(h::TagVariant::Surface)
                            .into_any_element(),
                        h::TagGroup::new("tg-sm", tags())
                            .size(Size::Sm)
                            .into_any_element(),
                        h::TagGroup::new("tg-lg", tags())
                            .size(Size::Lg)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Colors
    // -----------------------------------------------------------------------

    pub fn page_color_area(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.picker_color;
        component_doc_page!(
            "Color Area",
            crate::pages::Page::ColorArea.description(),
            crate::pages::Page::ColorArea.import_line(),
            vec![
                (
                    "Usage",
                    // v3: `<ColorArea defaultValue="hsl(30, 100%, 50%)" />`.
                    col(vec![h::ColorArea::new("ca-usage", value)
                        .default_value(value)
                        .into_any_element()]),
                ),
                (
                    "With Dots",
                    col(vec![h::ColorArea::new("ca-dots", value)
                        .show_dots(true)
                        .into_any_element()]),
                ),
                (
                    "Color Space & Channels",
                    row(vec![
                        spec(
                            "Saturation / Brightness (HSB)",
                            h::ColorArea::new("ca-hsb", value)
                                .color_space(h::ColorSpace::Hsb)
                                .x_channel(h::ColorChannel::Saturation)
                                .y_channel(h::ColorChannel::Brightness)
                                .size(px(160.), px(120.)),
                            cx,
                        ),
                        spec(
                            "Red / Green (RGB)",
                            h::ColorArea::new("ca-rgb", value)
                                .color_space(h::ColorSpace::Rgb)
                                .x_channel(h::ColorChannel::Red)
                                .y_channel(h::ColorChannel::Green)
                                .size(px(160.), px(120.)),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ColorArea::new("ca-controlled", value)
                            .size(px(180.), px(120.))
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        row(vec![
                            h::ColorSwatch::new(value).into_any_element(),
                            para(&format!("Value: {}", value.to_hex()), cx),
                        ]),
                    ]),
                ),
                (
                    "Render Function",
                    col(vec![h::ColorArea::new("ca-render-state", value)
                        .default_value(value)
                        .thumb(|state| {
                            gpui::div()
                                .absolute()
                                .inset_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    gpui::div()
                                        .size(px(if state.is_dragging { 6. } else { 4. }))
                                        .rounded_full()
                                        .bg(if state.is_hovered {
                                            gpui::black()
                                        } else {
                                            gpui::white()
                                        }),
                                )
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Saturation & brightness",
                    col(vec![
                        h::ColorArea::new("ca-main", value)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(&format!("Value: {}", value.to_hex()), cx),
                    ]),
                ),
                (
                    "Disabled",
                    col(vec![h::ColorArea::new("ca-disabled", value)
                        .size(px(180.), px(120.))
                        .is_disabled(true)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.picker_color;
        component_doc_page!(
            "Color Field",
            crate::pages::Page::ColorField.description(),
            crate::pages::Page::ColorField.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::ColorField::new("cf-usage", value)
                        .state(self.demo_text("cf-usage", "#0085F5", cx))
                        // v3's Usage is uncontrolled: `defaultValue="#0085F5"`.
                        .default_value(value)
                        .label("Color")
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::ColorField::new("cf-v-primary", value)
                            .state(self.demo_text("cf-v-primary", "#0085F5", cx))
                            .label("Primary")
                            .into_any_element(),
                        h::ColorField::new("cf-v-secondary", value)
                            .state(self.demo_text("cf-v-secondary", "#0085F5", cx))
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "On Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::ColorField::new("cf-surface", value)
                                .state(self.demo_text("cf-surface", "#0085F5", cx))
                                .label("Color")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::ColorField::new("cf-desc", value)
                        .state(self.demo_text("cf-desc", "#0085F5", cx))
                        .label("Brand color")
                        .description("Any CSS hex value")
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::ColorField::new("cf-req", value)
                        .state(self.demo_text("cf-req", "", cx))
                        .label("Color")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::ColorField::new("cf-dis", value)
                        .state(self.demo_text("cf-dis", "#0085F5", cx))
                        .label("Color")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::ColorField::new("cf-full", value)
                        .state(self.demo_text("cf-full", "#0085F5", cx))
                        .label("Color")
                        .full_width(true)
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![h::ColorField::new("cf-invalid", value)
                        .state(self.demo_text("cf-invalid", "not-a-color", cx))
                        .label("Color")
                        .is_required(true)
                        .is_invalid(true)
                        .validation_errors(["Enter a valid hex colour"])
                        .into_any_element()]),
                ),
                (
                    "Channel Editing",
                    "Edit individual HSL channels:",
                    col(vec![spec_row(vec![
                        h::ColorField::new("cf-ch-hue", value)
                                .state(self.demo_text("cf-ch-hue", "", cx))
                                // `colorSpace` names the channel set; `channel`
                                // picks one of them.
                                .color_space(h::ColorSpace::Hsl)
                                .channel(h::ColorChannel::Hue)
                                // `ColorField.Suffix` -- the unit after the value.
                                .suffix(gpui::div().child("\u{00b0}"))
                                .label("Hue")
                                .into_any_element(),
                        h::ColorField::new("cf-ch-sat", value)
                            .state(self.demo_text("cf-ch-sat", "", cx))
                            .channel(h::ColorChannel::Saturation)
                            .label("Saturation")
                            .into_any_element(),
                        h::ColorField::new("cf-ch-light", value)
                            .state(self.demo_text("cf-ch-light", "", cx))
                            .channel(h::ColorChannel::Lightness)
                            .label("Lightness")
                            .into_any_element(),
                        h::ColorSwatch::new(value).into_any_element(),
                    ]),]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ColorField::new("cf-ctl", value)
                            .state(self.color_field_state.clone())
                            .label("Color")
                            .on_change(opt_color_cb(cx.listener(
                                |this, parsed: &Option<h::PickerColor>, _, cx| {
                                    if let Some(c) = parsed {
                                        this.picker_color = *c;
                                    }
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        row(vec![
                            h::ColorSwatch::new(value).into_any_element(),
                            para(&format!("Value: {}", value.to_hex()), cx),
                        ]),
                    ]),
                ),
                (
                    "Render Function",
                    col(vec![{
                        let state = self.demo_text("cf-render", "#0085F5", cx);
                        let inner = state.clone();
                        h::ColorField::new("cf-render-root", value)
                            .state(state)
                            .is_required(true)
                            .content(move |field| {
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.))
                                    .child(
                                        h::ColorField::new("cf-render-input", value)
                                            .state(inner.clone())
                                            .label("Rendered color")
                                            .is_disabled(field.is_disabled)
                                            .is_invalid(field.is_invalid)
                                            .is_read_only(field.is_read_only)
                                            .is_required(field.is_required),
                                    )
                                    .child(format!(
                                        "required={} focused={} focus-within={} focus-visible={}",
                                        field.is_required,
                                        field.is_focused,
                                        field.is_focus_within,
                                        field.is_focus_visible,
                                    ))
                                    .into_any_element()
                            })
                            .into_any_element()
                    }]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let state = self.demo_text("cf-form", "#0085F5", cx);
                        h::Form::new()
                            .field(h::FormField::text(state.clone()).name("color"))
                            .child(
                                h::ColorField::new("cf-form", value)
                                    .state(state)
                                    .label("Brand color")
                                    .name("color")
                                    .is_required(true),
                            )
                            .child(h::Button::new("cf-form-submit").label("Save"))
                            .into_any_element()
                    }]),
                ),
                (
                    "Hex value",
                    field_col(vec![h::ColorField::new("cf-hex", value)
                        .state(self.color_field_state.clone())
                        .label("Brand color")
                        .description("Type a hex value such as #0085F5.")
                        .placeholder(value.to_hex())
                        .on_change(opt_color_cb(cx.listener(
                            |this, parsed: &Option<h::PickerColor>, _, cx| {
                                // Unparseable text reports None; the swatch
                                // holds its last good colour.
                                if let Some(c) = parsed {
                                    this.picker_color = *c;
                                }
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
                (
                    "Read-only display",
                    field_col(vec![h::ColorField::new("cf-display", value)
                        .label("Current value")
                        .description("Without a text state the field is a display.")
                        .into_any_element()]),
                ),
                (
                    "Single channel",
                    row(vec![
                        h::ColorField::new("cf-hue", value)
                            .channel(h::ColorChannel::Hue)
                            .label("Hue")
                            .into_any_element(),
                        h::ColorField::new("cf-red", value)
                            .channel(h::ColorChannel::Red)
                            .label("Red")
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.picker_color;
        component_doc_page!(
            "Color Picker",
            crate::pages::Page::ColorPicker.description(),
            crate::pages::Page::ColorPicker.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::ColorPicker::new("cp-main", value)
                        // v3's Usage is uncontrolled; "Controlled" is separate.
                        .default_value(value)
                        .label("Accent")
                        .show_alpha(true)
                        .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.picker_color = *c;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Controlled", "The caller owns the color value while the trigger owns its ordinary open state, matching v3's internal DialogTrigger.",
                    col(vec![
                        h::ColorPicker::new("cp-controlled", value)
                            .label("Brand")
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            },)))
                            .into_any_element(),
                        para(&format!("Value: {}", value.to_hex()), cx),
                    ]),
                ),
                (
                    "With Swatches", "A preset row beside the picker, which is v3's own layout.",
                    col(vec![
                        h::ColorSwatchPicker::new("cp-presets", palette())
                            .value(value)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Fields",
                    col(vec![
                        h::ColorField::new("cp-field", value)
                            .state(self.demo_text("cp-field", "#0085F5", cx))
                            .label("Hex")
                            .into_any_element(),
                        row(vec![
                            h::ColorField::new("cp-field-h", value)
                                .state(self.demo_text("cp-field-h", "", cx))
                                .channel(h::ColorChannel::Hue)
                                .label("H")
                                .into_any_element(),
                            h::ColorField::new("cp-field-s", value)
                                .state(self.demo_text("cp-field-s", "", cx))
                                .channel(h::ColorChannel::Saturation)
                                .label("S")
                                .into_any_element(),
                            h::ColorField::new("cp-field-l", value)
                                .state(self.demo_text("cp-field-l", "", cx))
                                .channel(h::ColorChannel::Lightness)
                                .label("L")
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "With Sliders",
                    col(vec![
                        h::ColorSlider::new("cp-sl-hue", value, h::ColorChannel::Hue)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        h::ColorSlider::new("cp-sl-alpha", value, h::ColorChannel::Alpha)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_slider(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.picker_color;
        let channels = [
            h::ColorChannel::Hue,
            h::ColorChannel::Saturation,
            h::ColorChannel::Brightness,
            h::ColorChannel::Alpha,
        ];
        component_doc_page!(
            "Color Slider",
            crate::pages::Page::ColorSlider.description(),
            crate::pages::Page::ColorSlider.import_line(),
            vec![
                (
                    "Usage",
                    // v3's Usage is uncontrolled; "Controlled" is its own
                    // example further down.
                    col(vec![h::ColorSlider::new(
                        "cs-usage",
                        value,
                        h::ColorChannel::Hue,
                    )
                    .default_value(value)
                    .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::ColorSlider::new(
                        "cs-disabled",
                        value,
                        h::ColorChannel::Hue,
                    )
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Vertical",
                    row(vec![h::ColorSlider::new(
                        "cs-vertical",
                        value,
                        h::ColorChannel::Hue,
                    )
                    .orientation(Orientation::Vertical)
                    .length(px(160.))
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ColorSlider::new("cs-controlled", value, h::ColorChannel::Hue)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        row(vec![
                            h::ColorSwatch::new(value).into_any_element(),
                            para(&format!("Value: {}", value.to_hex()), cx),
                        ]),
                    ]),
                ),
                (
                    "Render Function",
                    col(vec![h::ColorSlider::new(
                        "cs-render-state",
                        value,
                        h::ColorChannel::Hue,
                    )
                    .default_value(value)
                    .thumb(|state| {
                        gpui::div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                gpui::div()
                                    .size(px(if state.is_dragging { 6. } else { 4. }))
                                    .rounded_full()
                                    .bg(if state.is_hovered {
                                        gpui::black()
                                    } else {
                                        gpui::white()
                                    }),
                            )
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Alpha Channel",
                    col(vec![h::ColorSlider::new(
                        "cs-alpha",
                        value,
                        h::ColorChannel::Alpha,
                    )
                    .show_label(true)
                    .into_any_element()]),
                ),
                (
                    "HSL Channels",
                    col([
                        h::ColorChannel::Hue,
                        h::ColorChannel::Saturation,
                        h::ColorChannel::Lightness,
                    ]
                    .iter()
                    .map(|ch| {
                        h::ColorSlider::new(el_id(format!("cs-hsl-{ch:?}")), value, *ch)
                            .color_space(h::ColorSpace::Hsl)
                            .show_label(true)
                    })
                    .els()),
                ),
                (
                    "Channels",
                    col(channels
                        .iter()
                        .map(|ch| {
                            h::ColorSlider::new(el_id(format!("cs-{ch:?}")), value, *ch)
                                .on_change(color_cb(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.picker_color = *c;
                                        cx.notify();
                                    },
                                )))
                                // `ColorSlider.Output`'s render function is
                                // handed the colour: a swatch beside the value.
                                .output(|color, text| {
                                    gpui::div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.))
                                        .child(
                                            gpui::div()
                                                .size(px(10.))
                                                .rounded_full()
                                                .bg(color.to_hsla()),
                                        )
                                        .child(text.to_owned())
                                        .into_any_element()
                                })
                        })
                        .els()),
                ),
                (
                    "RGB channels",
                    col([
                        h::ColorChannel::Red,
                        h::ColorChannel::Green,
                        h::ColorChannel::Blue,
                    ]
                    .iter()
                    .map(|ch| {
                        h::ColorSlider::new(el_id(format!("cs-rgb-{ch:?}")), value, *ch).on_change(
                            color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            })),
                        )
                    })
                    .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_swatch(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Color Swatch",
            crate::pages::Page::ColorSwatch.description(),
            crate::pages::Page::ColorSwatch.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::ColorSwatch::new(
                        h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
                    )
                    .into_any_element()]),
                ),
                (
                    "Transparency",
                    row(vec![
                        spec(
                            "50% alpha",
                            h::ColorSwatch::new(
                                h::PickerColor::from_hex("#0085F5")
                                    .unwrap_or_default()
                                    .with_alpha(0.5),
                            ),
                            cx,
                        ),
                        spec(
                            "Fully transparent",
                            h::ColorSwatch::new(
                                h::PickerColor::from_hex("#0085F5")
                                    .unwrap_or_default()
                                    .with_alpha(0.0),
                            ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Accessibility", "v3 gives a swatch an accessible colour name. gpui has no accessibility tree, so the name is shown as a caption instead of announced.",
                    col(vec![
                        row(palette()
                            .into_iter()
                            .map(|c| {
                                let hex = c.to_hex();
                                spec(&hex, h::ColorSwatch::new(c), cx)
                            })
                            .collect()),
                    ]),
                ),
                (
                    "Sizes",
                    row(SizeXl::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::ColorSwatch::new(self.picker_color).size(*s),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Shapes",
                    row(h::SwatchShape::ALL
                        .iter()
                        .map(|shape| {
                            spec(
                                shape.label(),
                                h::ColorSwatch::new(self.picker_color)
                                    .size(SizeXl::Lg)
                                    .shape(*shape),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Palette",
                    row(palette()
                        .into_iter()
                        .map(|c| h::ColorSwatch::new(c).size(SizeXl::Lg))
                        .els()),
                ),
                (
                    "Alpha",
                    row(vec![
                        h::ColorSwatch::new(self.picker_color.with_alpha(1.0))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                        h::ColorSwatch::new(self.picker_color.with_alpha(0.5))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                        h::ColorSwatch::new(self.picker_color.with_alpha(0.15))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_swatch_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.swatch_selected;
        component_doc_page!(
            "Color Swatch Picker",
            crate::pages::Page::ColorSwatchPicker.description(),
            crate::pages::Page::ColorSwatchPicker.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-main", palette())
                            .value(selected)
                            .size(SizeXl::Lg)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(&format!("Selected: {}", selected.to_hex()), cx),
                    ]),
                ),
                (
                    "Variants",
                    col(vec![
                        spec(
                            "Circle (default)",
                            h::ColorSwatchPicker::new("csp-circle", palette())
                                .value(selected)
                                .on_change(color_cb(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                ))),
                            cx,
                        ),
                        spec(
                            "Square",
                            h::ColorSwatchPicker::new("csp-sq", palette())
                                .value(selected)
                                .on_change(color_cb(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                )))
                                .shape(h::SwatchShape::Square),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Sizes",
                    col(SizeXl::ALL
                        .iter()
                        .map(|sz| {
                            h::ColorSwatchPicker::new(el_id(format!("csp-{sz:?}")), palette())
                                .value(selected)
                                .on_change(color_cb(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                )))
                                .size(*sz)
                        })
                        .els()),
                ),
                (
                    "Disabled",
                    col(vec![h::ColorSwatchPicker::new("csp-disabled", palette())
                        .value(selected)
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled Item", "`ColorSwatchPicker.Item.isDisabled` dims one swatch — unclickable and out of the tab order — while the rest stay pickable: the difference from the whole-picker `isDisabled` above.",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-disabled-item", palette())
                            .value(selected)
                            .disabled_keys([2])
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Stack Layout",
                    col(vec![h::ColorSwatchPicker::new("csp-stack", palette())
                        .value(selected)
                        .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.swatch_selected = *c;
                            cx.notify();
                        })))
                        .layout(h::SwatchLayout::Stack)
                        .into_any_element()]),
                ),
                (
                    "Default Value",
                    col(vec![h::ColorSwatchPicker::new("csp-default", palette())
                        .default_value(palette()[2])
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-controlled", palette())
                            .value(selected)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(&format!("Selected: {}", selected.to_hex()), cx),
                    ]),
                ),
                (
                    "Custom Indicator", "v3 replaces `ColorSwatchPicker.Indicator`. The square picker uses a heart in place of the default selected checkmark.",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-indicator", palette())
                            .value(selected)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            })))
                            .shape(h::SwatchShape::Square)
                            .size(SizeXl::Lg)
                            .indicator(|_, _| {
                                gpui::svg()
                                    .size(px(12.))
                                    .path(h::icons::HEART_FILL)
                                    .text_color(gpui::white())
                                    .into_any_element()
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Item Render State", "`item_content` receives each item's color plus selected, hovered, pressed, focused, focus-visible, and disabled state. These custom tiles use that state while the picker keeps navigation and selection.",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-item-content", palette())
                            .value(selected)
                            .size(SizeXl::Xl)
                            .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            })))
                            .item_content(|_, state| {
                                gpui::div()
                                    .size_full()
                                    .rounded(px(8.))
                                    .bg(state.color.to_hsla())
                                    .when(state.is_pressed, |tile| tile.opacity(0.65))
                                    .when(state.is_selected, |tile| {
                                        tile.child(
                                            gpui::svg()
                                                .size(px(12.))
                                                .path(h::icons::HEART_FILL)
                                                .text_color(gpui::white()),
                                        )
                                    })
                                    .into_any_element()
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Square, stacked",
                    col(vec![h::ColorSwatchPicker::new("csp-square", palette())
                        .value(selected)
                        .shape(h::SwatchShape::Square)
                        .layout(h::SwatchLayout::Stack)
                        .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.swatch_selected = *c;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Controls
    // -----------------------------------------------------------------------

    pub fn page_slider(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let volume = self.demo_value("sl-controlled", 40.);
        let value = self.slider_value;
        component_doc_page!(
            "Slider",
            crate::pages::Page::Slider.description(),
            crate::pages::Page::Slider.import_line(),
            vec![
                (
                    "Usage",
                    // v3: `<Slider defaultValue={30}>` -- uncontrolled, with
                    // "Controlled Value" below for the other half.
                    col(vec![gpui::div()
                        .w(px(320.))
                        .child(
                            h::Slider::new("sl-main", 30.)
                                .default_value(30.)
                                .label("Volume")
                                .show_value(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Format options",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-fmt", value)
                            .label("Budget")
                            .show_value(true)
                            .format_options(h::NumberFormat::currency("EUR"))
                            .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                                this.slider_value = *v;
                                cx.notify();
                            }))),
                    )]),
                ),
                (
                    "Range Slider Anatomy", "v3 builds a range slider from its parts: a `Label`, an `Output`, and a `Track` whose render prop is handed the state so it can draw one `Thumb` per value. The `thumb` closure is that render prop.",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-anatomy", 25.)
                                .default_values([25., 75.])
                                .label("Price range")
                                .show_value(true)
                                .thumb(|index, value| {
                                    gpui::div()
                                        .id(("sl-anatomy-thumb", index))
                                        .size(px(18.))
                                        .rounded_full()
                                        .border_2()
                                        .border_color(gpui::white())
                                        .bg(gpui::rgb(0x0085F5))
                                        // The closure is handed the value the slider
                                        // already computed for this thumb, so the
                                        // caller never re-derives it.
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            gpui::div()
                                                .absolute()
                                                .top(px(-20.))
                                                .text_size(px(11.))
                                                .child(format!("{value:.0}")),
                                        )
                                        .into_any_element()
                                }),
                        ),
                    ]),
                ),
                (
                    "Controlled Value",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-controlled", volume)
                                .label("Volume")
                                .show_value(true)
                                .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                                    this.set_demo_value("sl-controlled", *v);
                                    cx.notify();
                                }))),
                        ),
                        para(&format!("Value: {volume:.0}"), cx),
                    ]),
                ),
                (
                    "Custom Value Formatting",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-fmt-pct", 0.35)
                                .min_value(0.)
                                .max_value(1.)
                                .step(0.01)
                                .label("Opacity")
                                .show_value(true)
                                .format_options(herogpui_core::NumberFormat::percent()),
                        ),
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-fmt-cur", 1200.)
                                .min_value(0.)
                                .max_value(5000.)
                                .step(50.)
                                .label("Budget")
                                .show_value(true)
                                .format_options(herogpui_core::NumberFormat::currency("USD")),
                        ),
                    ]),
                ),
                (
                    "Custom Output Display", "v3's `Slider.Output` takes a render prop. The closure receives every live value and its formatted thumb label.",
                    col(vec![
                        gpui::div()
                            .w(px(320.))
                            .child(
                                h::Slider::new("sl-output", volume)
                                    .label("Brightness")
                                    .output(|values, labels| {
                                        h::Chip::new()
                                            .variant(h::ChipVariant::Soft)
                                            .color(Color::Accent)
                                            .child(h::ChipLabel::new().child(format!(
                                                "{:.0}% ({})",
                                                values[0], labels[0]
                                            )))
                                            .into_any_element()
                                    })
                                    .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                                        this.set_demo_value("sl-controlled", *v);
                                        cx.notify();
                                    }))),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Range (multi-thumb)",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-range", value)
                            .label("Price range")
                            .values(self.slider_range.clone())
                            .on_change_all(cx.listener(|this, vs: &[f32], _, cx| {
                                this.slider_range = vs.to_vec();
                                cx.notify();
                            })),
                    )]),
                ),
                (
                    "Disabled Thumb", "`Slider.Thumb.isDisabled` fixes one thumb — dimmed, out of the roving tab stop, answering no drag or keys — while the other thumb keeps moving: the contrast a whole-slider `isDisabled` cannot show.",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-lock", value)
                                .label("Price range")
                                .values(self.slider_range.clone())
                                .disabled_keys([0])
                                .on_change_all(cx.listener(|this, vs: &[f32], _, cx| {
                                    this.slider_range = vs.to_vec();
                                    cx.notify();
                                })),
                        ),
                    ]),
                ),
                (
                    "Form Example", "`Slider.Thumb.name` names each end of a range. `form_fields` hands the pair to the `Form` — it is told its fields, with no context propagation — so a submission carries one value per named thumb.",
                    col(vec![
                        fixed_demo(320., {
                            // v3 renders one `<input name=…>` per thumb; the form reads
                            // them back through `form_fields`, as DateRangePicker does.
                            let slider = h::Slider::new("sl-form", value)
                                .label("Price range")
                                .values(self.slider_range.clone())
                                .start_name("min")
                                .end_name("max")
                                .on_change_all(cx.listener(|this, vs: &[f32], _, cx| {
                                    this.slider_range = vs.to_vec();
                                    cx.notify();
                                }));
                            let mut form = h::Form::new();
                            for field in slider.form_fields() {
                                form = form.field(field);
                            }
                            let form =
                                form.on_submit(cx.listener(|this, data: &h::FormData, _, cx| {
                                    this.input_submitted = format!(
                                        "min={}, max={}",
                                        data.text("min").unwrap_or_default(),
                                        data.text("max").unwrap_or_default(),
                                    );
                                    cx.notify();
                                }));
                            let submit = form.submit_handler();
                            form.child(slider.into_any_element())
                                .child(
                                    h::Button::new("sl-form-submit")
                                        .label("Save")
                                        .on_press(move |_, window, cx| submit(window, cx)),
                                )
                                .into_any_element()
                        }),
                        para(
                            &if self.input_submitted.is_empty() {
                                "Nothing submitted yet".to_owned()
                            } else {
                                format!("Submitted: {}", self.input_submitted)
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Vertical",
                    row(vec![h::Slider::new("sl-vert", value)
                        .orientation(Orientation::Vertical)
                        .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                            this.slider_value = *v;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Step & disabled",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-step", value)
                                .step(10.0)
                                .label("Step 10")
                                .show_value(true)
                                .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                                    this.slider_value = *v;
                                    cx.notify();
                                }))),
                        ),
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-disabled", value)
                                .is_disabled(true)
                                .label("Disabled"),
                        ),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_switch(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let (a, b) = (self.switch_a, self.switch_b);
        let controlled = self.demo_flag("sw-controlled", false);
        let wifi = self.demo_flag("sw-group-wifi", true);
        let bluetooth = self.demo_flag("sw-group-bt", false);
        let airplane = self.demo_flag("sw-group-air", false);
        let terms = self.demo_flag("sw-form", false);
        component_doc_page!(
            "Switch",
            crate::pages::Page::Switch.description(),
            crate::pages::Page::Switch.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Switch::new("sw-a")
                            .is_selected(a)
                            .label(gpui::div().child("Enable notifications"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.switch_a = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                        h::Switch::new("sw-b")
                            .is_selected(b)
                            .label(gpui::div().child("Share usage data"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.switch_b = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::Switch::new(el_id(format!("sw-{s:?}")))
                                    .default_selected(true)
                                    .size(*s),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "With Icons",
                    row(vec![
                        h::Switch::new("sw-icon-1")
                            .default_selected(true)
                            .thumb_icons(icon(h::icons::MOON, cx), icon(h::icons::SUN, cx))
                            .label(gpui::div().child("Appearance"))
                            .into_any_element(),
                        h::Switch::new("sw-icon-2")
                            .thumb_icons(icon(h::icons::EYE_OFF, cx), icon(h::icons::EYE, cx))
                            .label(gpui::div().child("Show preview"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Without Label",
                    row(vec![
                        h::Switch::new("sw-nolabel-1")
                            .default_selected(true)
                            .into_any_element(),
                        h::Switch::new("sw-nolabel-2").into_any_element(),
                    ]),
                ),
                (
                    "With Description",
                    col(vec![h::Switch::new("sw-desc")
                        .default_selected(true)
                        .label(gpui::div().child("Sync across devices"))
                        .description("Changes are pushed to every signed-in device.")
                        .into_any_element()]),
                ),
                (
                    "Default Selected",
                    col(vec![h::Switch::new("sw-default")
                        .default_selected(true)
                        .label(gpui::div().child("On by default"))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Switch::new("sw-controlled")
                            .is_selected(controlled)
                            .label(gpui::div().child("Notifications"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("sw-controlled", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(
                            if controlled {
                                "Status: selected"
                            } else {
                                "Status: not selected"
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Label Position",
                    col(vec![
                        h::Switch::new("sw-lp-after")
                            .label(gpui::div().child("Label after"))
                            .into_any_element(),
                        h::Switch::new("sw-lp-before")
                            .label_first(true)
                            .label(gpui::div().child("Label before"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Group",
                    col(vec![h::SwitchGroup::new()
                        .orientation(Orientation::Vertical)
                        .child(
                            h::Switch::new("sw-g-wifi")
                                .is_selected(wifi)
                                .label(gpui::div().child("Wi-Fi"))
                                .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-wifi", *v);
                                    cx.notify();
                                }))),
                        )
                        .child(
                            h::Switch::new("sw-g-bt")
                                .is_selected(bluetooth)
                                .label(gpui::div().child("Bluetooth"))
                                .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-bt", *v);
                                    cx.notify();
                                }))),
                        )
                        .child(
                            h::Switch::new("sw-g-air")
                                .is_selected(airplane)
                                .label(gpui::div().child("Airplane mode"))
                                .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-air", *v);
                                    cx.notify();
                                }))),
                        )
                        .into_any_element()]),
                ),
                (
                    "Group Horizontal",
                    row(vec![h::SwitchGroup::new()
                        .orientation(Orientation::Horizontal)
                        .child(
                            h::Switch::new("sw-gh-1")
                                .default_selected(true)
                                .label(gpui::div().child("Email")),
                        )
                        .child(h::Switch::new("sw-gh-2").label(gpui::div().child("SMS")))
                        .child(h::Switch::new("sw-gh-3").label(gpui::div().child("Push")))
                        .into_any_element()]),
                ),
                (
                    "Form Integration",
                    col(vec![
                        {
                            let switch = h::Switch::new("sw-form")
                                .name("terms")
                                // `value` is what a checked switch submits.
                                .value("accepted")
                                .is_selected(terms)
                                .is_required(true)
                                .label(gpui::div().child("Accept the terms"))
                                .is_invalid(!terms)
                                .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-form", *v);
                                    cx.notify();
                                })));
                            let form = h::Form::new()
                                .field(switch.form_field().expect("named switch field"))
                                .on_submit(cx.listener(|this, data: &h::FormData, _, cx| {
                                    this.input_submitted =
                                        format!("terms={}", data.text("terms").unwrap_or_default());
                                    cx.notify();
                                }))
                                .on_invalid(cx.listener(|this, _: &h::FormData, _, cx| {
                                    this.input_submitted =
                                        "Accept the terms before submitting".to_owned();
                                    cx.notify();
                                }));
                            let submit = form.submit_handler();
                            form.child(switch)
                                .child(
                                    h::Button::new("sw-form-submit")
                                        .label("Submit")
                                        .on_press(move |_, window, cx| submit(window, cx)),
                                )
                                .into_any_element()
                        },
                        para(
                            if self.input_submitted.is_empty() {
                                "Toggle the required switch, then submit the live form field."
                            } else {
                                &self.input_submitted
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Render Props",
                    col(vec![h::Switch::new("sw-render")
                        .is_selected(controlled)
                        .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                            this.set_demo_flag("sw-controlled", *v);
                            cx.notify();
                        })))
                        // v3's children-as-a-function: the closure is handed
                        // `isSelected`, `isHovered`, `isPressed`, `isFocused` and
                        // `isFocusVisible` and draws the label from them.
                        .content(|state| {
                            let mut parts = Vec::new();
                            if state.is_selected {
                                parts.push("selected");
                            }
                            if state.is_hovered {
                                parts.push("hovered");
                            }
                            if state.is_pressed {
                                parts.push("pressed");
                            }
                            if state.is_focus_visible {
                                parts.push("focus-visible");
                            }
                            gpui::div()
                                .child(if parts.is_empty() {
                                    "idle".to_owned()
                                } else {
                                    parts.join(" + ")
                                })
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    row(vec![
                        h::Switch::new("sw-d-off")
                            .is_disabled(true)
                            .into_any_element(),
                        h::Switch::new("sw-d-on")
                            .is_selected(true)
                            .is_disabled(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Data display
    // -----------------------------------------------------------------------

    pub fn page_badge(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Badge",
            crate::pages::Page::Badge.description(),
            crate::pages::Page::Badge.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::BadgeAnchor::new()
                        .child(avatar_box(cx))
                        .child(h::Badge::new().child(h::BadgeLabel::new().child("5")))
                        .into_any_element()]),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|sz| {
                            spec(
                                sz.label(),
                                h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                    h::Badge::new()
                                        .size(*sz)
                                        .child(h::BadgeLabel::new().child("5")),
                                ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Dot Badge",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                // No children is v3's dot badge.
                                h::BadgeAnchor::new()
                                    .child(avatar_box(cx))
                                    .child(h::Badge::new().color(*c)),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "With Content",
                    row(vec![
                        spec(
                            "Number",
                            h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                h::Badge::new()
                                    .color(Color::Danger)
                                    .size(Size::Sm)
                                    .child(h::BadgeLabel::new().child("5")),
                            ),
                            cx,
                        ),
                        spec(
                            "Text",
                            h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                h::Badge::new()
                                    .color(Color::Accent)
                                    .child(h::BadgeLabel::new().child("NEW")),
                            ),
                            cx,
                        ),
                        spec(
                            "Icon",
                            // Only plain text is auto-wrapped upstream; an
                            // element child composes straight into the badge.
                            h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                h::Badge::new().color(Color::Success).child(
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::CHECK)
                                        .text_color(cx.colors().success.foreground),
                                ),
                            ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Variants",
                    row(h::BadgeVariant::ALL
                        .iter()
                        .map(|v| {
                            spec(
                                v.label(),
                                h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                    h::Badge::new()
                                        .color(Color::Accent)
                                        .variant(*v)
                                        .child(h::BadgeLabel::new().child("5")),
                                ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::BadgeAnchor::new().child(avatar_box(cx)).child(
                                    h::Badge::new()
                                        .color(*c)
                                        .child(h::BadgeLabel::new().child("5")),
                                ),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Dot & placement",
                    row(vec![
                        // No children is v3's dot badge.
                        h::BadgeAnchor::new()
                            .child(avatar_box(cx))
                            .child(h::Badge::new().color(Color::Success))
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(cx))
                            .child(
                                h::Badge::new()
                                    .placement(h::BadgePlacement::BottomRight)
                                    .child(h::BadgeLabel::new().child("9")),
                            )
                            .into_any_element(),
                        h::BadgeAnchor::new()
                            .child(avatar_box(cx))
                            .child(
                                h::Badge::new()
                                    .placement(h::BadgePlacement::TopLeft)
                                    .child(h::BadgeLabel::new().child("New")),
                            )
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_chip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Chip",
            crate::pages::Page::Chip.description(),
            crate::pages::Page::Chip.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Chip::new()
                        .child(h::ChipLabel::new().child("Chip"))
                        .into_any_element()]),
                ),
                (
                    "Statuses",
                    row(vec![
                        h::Chip::new()
                            .color(Color::Success)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Danger)
                            .variant(h::ChipVariant::Soft)
                            .child(h::ChipLabel::new().child("Vacation"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Icons",
                    row(vec![
                        h::Chip::new()
                            .color(Color::Success)
                            .child(icon(h::icons::CHECK, cx))
                            .child(h::ChipLabel::new().child("Verified"))
                            .into_any_element(),
                        h::Chip::new()
                            .child(icon(h::icons::EXTERNAL_LINK, cx))
                            .child(h::ChipLabel::new().child("Link"))
                            .into_any_element(),
                        h::Chip::new()
                            .color(Color::Accent)
                            .child(icon(h::icons::SEARCH, cx))
                            .child(h::ChipLabel::new().child("Search"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants",
                    row(h::ChipVariant::ALL
                        .iter()
                        .map(|v| {
                            h::Chip::new()
                                .variant(*v)
                                .color(Color::Accent)
                                .child(h::ChipLabel::new().child(v.label()))
                        })
                        .els()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            h::Chip::new()
                                .color(*c)
                                .child(h::ChipLabel::new().child(c.label()))
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| {
                            h::Chip::new()
                                .size(*s)
                                .child(h::ChipLabel::new().child(s.label()))
                        })
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let table_page = self.demo_value("tbl-page", 1.) as usize;
        let tbl_expanded = self.demo_selection("tbl-expanded");
        let build = |id: &'static str| {
            h::Table::new(vec!["Name".into(), "Role".into(), "Status".into()])
                .id(id)
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Tony Reichert").into_any_element(),
                        gpui::div().child("CEO").into_any_element(),
                        h::Chip::new()
                            .color(Color::Success)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                    ])
                    .text_value("Tony Reichert"),
                )
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Zoey Lang").into_any_element(),
                        gpui::div().child("Tech Lead").into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                    ])
                    .text_value("Zoey Lang"),
                )
                .tree_row(
                    h::TableRow::new(vec![
                        gpui::div().child("Jane Fisher").into_any_element(),
                        gpui::div().child("Designer").into_any_element(),
                        h::Chip::new()
                            .color(Color::Danger)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Vacation"))
                            .into_any_element(),
                    ])
                    .text_value("Jane Fisher"),
                )
        };
        component_doc_page!(
            "Table",
            crate::pages::Page::Table.description(),
            crate::pages::Page::Table.import_line(),
            vec![
                ("Usage", col(vec![build("tbl-usage").into_any_element()])),
                (
                    "Variants",
                    col(h::TableVariant::ALL
                        .iter()
                        .map(|v| {
                            build(match v {
                                h::TableVariant::Primary => "tbl-variant-primary",
                                h::TableVariant::Secondary => "tbl-variant-secondary",
                            })
                            .variant(*v)
                        })
                        .els()),
                ),
                (
                    "Custom sort indicator",
                    // Sorted on load, so the custom indicator is actually
                    // visible: `indicator` only renders for the sorted column.
                    col(vec![h::Table::new(vec![])
                        .id("tbl-custom-sort-indicator")
                        .column(h::TableColumn::new("Name").allows_sorting(true))
                        .column("Role")
                        .row(vec![
                            gpui::div().child("Tony Reichert").into_any_element(),
                            gpui::div().child("CEO").into_any_element(),
                        ])
                        .row(vec![
                            gpui::div().child("Zoey Lang").into_any_element(),
                            gpui::div().child("Tech Lead").into_any_element(),
                        ])
                        .sort_descriptor(h::SortDescriptor::new(
                            "Name",
                            h::SortDirection::Ascending,
                        ))
                        .indicator(|dir| {
                            gpui::div()
                                .text_size(px(11.))
                                .child(match dir {
                                    h::SortDirection::Ascending => "▲",
                                    h::SortDirection::Descending => "▼",
                                })
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Selection",
                    col(vec![
                        build("tbl-selection")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(self.table_selection.clone())
                            .on_selection_change(cx.listener(
                                |this, keys: &[SharedString], _, cx| {
                                    this.table_selection = keys.to_vec();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", self.table_selection.len()), cx),
                    ]),
                ),
                (
                    "Sorting",
                    col(vec![
                        {
                            let mut sortable = h::Table::new(vec![])
                                .id("tbl-sorting")
                                .column(
                                    h::TableColumn::new("Name")
                                        .allows_sorting(true)
                                        .is_row_header(true),
                                )
                                .column(h::TableColumn::new("Role").allows_sorting(true))
                                .column("Status")
                                .row(vec![
                                    gpui::div().child("Tony Reichert").into_any_element(),
                                    gpui::div().child("CEO").into_any_element(),
                                    gpui::div().child("Active").into_any_element(),
                                ])
                                .row(vec![
                                    gpui::div().child("Zoey Lang").into_any_element(),
                                    gpui::div().child("Tech Lead").into_any_element(),
                                    gpui::div().child("Paused").into_any_element(),
                                ])
                                .on_sort_change(sort_cb(cx.listener(
                                    |this, d: &h::SortDescriptor, _, cx| {
                                        this.table_sort = Some(d.clone());
                                        cx.notify();
                                    },
                                )));
                            if let Some(d) = self.table_sort.clone() {
                                sortable = sortable.sort_descriptor(d);
                            }
                            sortable.into_any_element()
                        },
                        para(
                            &match &self.table_sort {
                                Some(d) => format!("Sorted by {} {:?}", d.column, d.direction),
                                None => "Unsorted".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Virtualization", "v3 wraps the table in `Virtualizer` with `TableLayout`. Cells here are built elements, which cannot be handed out twice, so a virtual table takes a row factory and asks for the rows the viewport shows — one thousand of them, forty pixels each.",
                    col(vec![
                        h::Table::new(vec![])
                            .id("tbl-virtualization")
                            .column(h::TableColumn::new("Name").is_row_header(true))
                            .column("Email")
                            .row_height(px(40.))
                            .max_h(px(320.))
                            .virtual_rows(
                                1000,
                                "virtual-users",
                                |i| i.to_string().into(),
                                |i| {
                                    let (name, email) = virtual_user(i);
                                    h::TableRow::new(vec![
                                        gpui::div().child(name).into_any_element(),
                                        gpui::div().child(email).into_any_element(),
                                    ])
                                },
                            )
                            .virtual_text_value(|i| virtual_user(i).0.into())
                            .into_any_element(),
                        para(
                            "`estimated_row_height` virtualizes rows that differ: gpui's \
                             `list` measures each one it builds, and `loader_height` \
                             fixes the load-more row underneath.",
                            cx,
                        ),
                        h::Table::new(vec![])
                            .id("tbl-virtual-var")
                            .column(h::TableColumn::new("Name").is_row_header(true))
                            .column("Email")
                            .estimated_row_height(px(44.))
                            .loader_height(px(44.))
                            .max_h(px(320.))
                            .is_pending(true)
                            .virtual_rows(
                                1000,
                                "virtual-variable-users",
                                |i| i.to_string().into(),
                                |i| {
                                    let (name, email) = virtual_user(i);
                                    let mut cells = vec![gpui::div()
                                        .flex()
                                        .flex_col()
                                        .child(name)
                                        .when(i % 3 == 0, |el| {
                                            el.child(
                                                gpui::div()
                                                    .text_size(px(12.))
                                                    .child("Signed up this week"),
                                            )
                                        })
                                        .into_any_element()];
                                    cells.push(gpui::div().child(email).into_any_element());
                                    h::TableRow::new(cells)
                                },
                            )
                            .virtual_text_value(|i| virtual_user(i).0.into())
                            .into_any_element(),
                    ]),
                ),
                ("Column Resizing", "Drag a trailing-edge divider, or focus it with Tab, press Enter, and use the arrow keys. This example feeds onResize values back as controlled column widths and reports completion through onResizeEnd.", {
                    let resize_name = self.demo_value("tbl-resize-name", 220.);
                    let resize_role = self.demo_value("tbl-resize-role", 180.);
                    let resize_status = self.demo_text_value("tbl-resize-status");
                    col(vec![
                        para(
                            &format!(
                                "{} Name: {:.0}px · Role: {:.0}px",
                                if resize_status.is_empty() {
                                    "Ready."
                                } else {
                                    resize_status.as_str()
                                },
                                resize_name,
                                resize_role,
                            ),
                            cx,
                        ),
                        h::Table::new(vec![])
                            .id("tbl-column-resizing")
                            .column(
                                h::TableColumn::new("Name")
                                    .allows_resizing(true)
                                    .width(px(resize_name))
                                    .min_width(px(120.)),
                            )
                            .column(
                                h::TableColumn::new("Role")
                                    .allows_resizing(true)
                                    .width(px(resize_role)),
                            )
                            .column(h::TableColumn::new("Status").default_width(px(140.)))
                            .row(vec![
                                gpui::div().child("Tony Reichert").into_any_element(),
                                gpui::div().child("CEO").into_any_element(),
                                gpui::div().child("Active").into_any_element(),
                            ])
                            .row(vec![
                                gpui::div().child("Zoey Lang").into_any_element(),
                                gpui::div().child("Tech Lead").into_any_element(),
                                gpui::div().child("Paused").into_any_element(),
                            ])
                            .on_resize_start(cx.listener(|this, _, _, cx| {
                                this.set_demo_text_value(
                                    "tbl-resize-status",
                                    "Resizing.".to_owned(),
                                );
                                cx.notify();
                            }))
                            .on_resize(cx.listener(
                                |this, widths: &[(SharedString, gpui::Pixels)], _, cx| {
                                    for (column, width) in widths {
                                        match column.as_ref() {
                                            "Name" => this.set_demo_value(
                                                "tbl-resize-name",
                                                f32::from(*width),
                                            ),
                                            "Role" => this.set_demo_value(
                                                "tbl-resize-role",
                                                f32::from(*width),
                                            ),
                                            _ => {}
                                        }
                                    }
                                    cx.notify();
                                },
                            ))
                            .on_resize_end(cx.listener(|this, _, _, cx| {
                                this.set_demo_text_value("tbl-resize-status", "Saved.".to_owned());
                                cx.notify();
                            }))
                            .into_any_element(),
                    ])
                },),
                (
                    "Expandable Rows", "A row's children are nested under it, and `expandedKeys` decides which parents show theirs. The chevron sits in the tree column; Right expands the focused parent, and Left collapses it or returns the row cursor to its parent.",
                    col(vec![
                        {
                            let cell = |text: &str| gpui::div().child(text.to_owned());
                            h::Table::new(vec!["Title".into(), "Type".into(), "Modified".into()])
                                .id("tbl-expandable-rows")
                                .tree_column(0)
                                .expanded_keys(tbl_expanded.iter().cloned())
                                .on_expanded_change(cx.listener(
                                    |this, keys: &[SharedString], _, cx| {
                                        this.set_demo_selection("tbl-expanded", keys.to_vec());
                                        cx.notify();
                                    },
                                ))
                                .tree_row(
                                    h::TableRow::new(vec![
                                        cell("Documents").into_any_element(),
                                        cell("Folder").into_any_element(),
                                        cell("8/2/2025").into_any_element(),
                                    ])
                                    .key("documents")
                                    .children(vec![
                                        h::TableRow::new(vec![
                                            cell("Reports").into_any_element(),
                                            cell("Folder").into_any_element(),
                                            cell("8/2/2025").into_any_element(),
                                        ])
                                        .key("reports")
                                        .children(vec![
                                            h::TableRow::new(vec![
                                                cell("Weekly Report").into_any_element(),
                                                cell("File").into_any_element(),
                                                cell("7/10/2025").into_any_element(),
                                            ])
                                            .key("weekly"),
                                            h::TableRow::new(vec![
                                                cell("Budget").into_any_element(),
                                                cell("File").into_any_element(),
                                                cell("8/20/2025").into_any_element(),
                                            ])
                                            .key("budget"),
                                        ]),
                                        h::TableRow::new(vec![
                                            cell("Contract.pdf").into_any_element(),
                                            cell("File").into_any_element(),
                                            cell("6/1/2025").into_any_element(),
                                        ])
                                        .key("contract"),
                                    ]),
                                )
                                .tree_row(
                                    h::TableRow::new(vec![
                                        cell("Photos").into_any_element(),
                                        cell("Folder").into_any_element(),
                                        cell("5/5/2025").into_any_element(),
                                    ])
                                    .key("photos")
                                    .children(vec![
                                        h::TableRow::new(vec![
                                            cell("Holiday.jpg").into_any_element(),
                                            cell("Image").into_any_element(),
                                            cell("5/5/2025").into_any_element(),
                                        ])
                                        .key("holiday"),
                                    ]),
                                )
                                .into_any_element()
                        },
                    ]),
                ),
                (
                    "Secondary Variant",
                    col(vec![build("tbl-secondary-variant")
                        .variant(h::TableVariant::Secondary)
                        .into_any_element()]),
                ),
                (
                    "Async Loading", "`isPending` covers the table while a request is in flight; `onLoadMore` fires when the last row scrolls into view.",
                    col(vec![
                        build("tbl-async-loading")
                            .is_pending(true)
                            .on_load_more(|_, _| {})
                            // v3's Async Loading example writes `scrollOffset={0}`.
                            .scroll_offset(0.)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Pagination",
                    col(vec![{
                        let start = table_page.saturating_sub(1) * 2;
                        let people = [
                            ("Tony Reichert", "CEO"),
                            ("Zoey Lang", "Tech Lead"),
                            ("Jane Fisher", "Designer"),
                            ("William Howard", "Support"),
                            ("Kristen Copper", "Sales Manager"),
                            ("Emily Collins", "Marketing"),
                        ];
                        let mut paged =
                            h::Table::new(vec!["Name".into(), "Role".into()]).id("tbl-pagination");
                        for (name, role) in people.iter().skip(start).take(2) {
                            paged = paged.row(vec![
                                gpui::div().child(*name).into_any_element(),
                                gpui::div().child(*role).into_any_element(),
                            ]);
                        }
                        // v3 puts the pagination in a `Table.Footer`, with a
                        // `Pagination.Summary` at its start.
                        paged
                            .footer(
                                h::Pagination::new("tbl-pages", table_page, 3)
                                    .size(Size::Sm)
                                    .summary(format!(
                                        "{} to {} of {} results",
                                        start + 1,
                                        (start + 2).min(people.len()),
                                        people.len()
                                    ))
                                    .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                                        this.set_demo_value("tbl-page", *p as f32);
                                        cx.notify();
                                    }))),
                            )
                            .into_any_element()
                    },]),
                ),
                (
                    "Custom Cells",
                    col(vec![h::Table::new(vec![
                        "Member".into(),
                        "Role".into(),
                        "Status".into(),
                    ])
                    .id("tbl-custom-cells")
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(
                                h::Avatar::new("tbl-tony")
                                    .name("Tony Reichert")
                                    .size(Size::Sm)
                            )
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .child(gpui::div().child("Tony Reichert"))
                                    .child(
                                        gpui::div()
                                            .text_size(px(11.5))
                                            .text_color(cx.colors().muted)
                                            .child("tony@example.com"),
                                    ),
                            )
                            .into_any_element(),
                        gpui::div().child("CEO").into_any_element(),
                        h::Chip::new()
                            .color(Color::Success)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Active"))
                            .into_any_element(),
                    ])
                    .row(vec![
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(h::Avatar::new("tbl-zoey").name("Zoey Lang").size(Size::Sm))
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .child(gpui::div().child("Zoey Lang"))
                                    .child(
                                        gpui::div()
                                            .text_size(px(11.5))
                                            .text_color(cx.colors().muted)
                                            .child("zoey@example.com"),
                                    ),
                            )
                            .into_any_element(),
                        gpui::div().child("Tech Lead").into_any_element(),
                        h::Chip::new()
                            .color(Color::Warning)
                            .variant(h::ChipVariant::Soft)
                            .size(Size::Sm)
                            .child(h::ChipLabel::new().child("Paused"))
                            .into_any_element(),
                    ])
                    .into_any_element()]),
                ),
                (
                    "Empty and loading",
                    col(vec![
                        h::Table::new(vec!["Name".into(), "Role".into()])
                            .id("tbl-empty-and-loading")
                            .empty_state("Nobody here yet")
                            .into_any_element(),
                        build("tbl-empty-and-loading-2")
                            .is_pending(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Date and time
    // -----------------------------------------------------------------------

    pub fn page_calendar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let picked = self.cal_picked;
        let today = h::Date::today();
        let focused = self.calendar_focus;
        component_doc_page!(
            "Calendar",
            crate::pages::Page::Calendar.description(),
            crate::pages::Page::Calendar.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Calendar::new(self.calendar.clone())
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match picked {
                                Some(d) => format!("Selected: {}", d.format_iso()),
                                None => "No date selected".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Default Value",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-default", cx))
                        .default_value(h::Date::new(2025, 12, 25))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Calendar::new(self.demo_calendar("cal-controlled", cx))
                            .on_focus_change(date_cb(cx.listener(|this, d: &h::Date, _, cx| {
                                this.set_demo_text_value("cal-focus", d.format_iso());
                                cx.notify();
                            })))
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Min and Max Dates",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-minmax", cx))
                        .min_value(h::Date::new(today.year, today.month, 5))
                        .max_value(h::Date::new(today.year, today.month, 20))
                        .into_any_element()]),
                ),
                (
                    "Unavailable Dates",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-unavailable", cx))
                        // Weekends are struck through, which is v3's own example.
                        .is_date_unavailable(|date| {
                            let weekday = h::weekday_index(date);
                            weekday == 0 || weekday == 6
                        })
                        .into_any_element()]),
                ),
                (
                    "Weeks in Month",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]),
                ),
                (
                    "Multiple Selection",
                    col({
                        let cal = self.demo_calendar("cal-multiple", cx);
                        // v3's `onChange` carries the whole selection in
                        // multiple mode; the summary below re-reads the state,
                        // and `window.refresh()` forces the repaint that makes
                        // the new frame visible.
                        let dates = cal.read(cx).selected_dates().to_vec();
                        let summary = if dates.is_empty() {
                            "No dates selected".to_owned()
                        } else {
                            format!(
                                "Selected: {}",
                                dates
                                    .iter()
                                    .map(|d| d.format_iso())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };
                        vec![
                            h::Calendar::new(cal)
                                .selection_mode(SelectionMode::Multiple)
                                .on_change_all(|_, window, _| window.refresh())
                                .into_any_element(),
                            para(&summary, cx),
                        ]
                    }),
                ),
                (
                    "Focused Value",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-focused", cx))
                        .focused_value(focused)
                        .on_focus_change(date_cb(cx.listener(
                            |this, date: &h::Date, _, cx| {
                                this.calendar_focus = *date;
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
                (
                    "Cell Indicators", "The marked days are the ones with events.",
                    col(vec![
                        h::Calendar::new(self.demo_calendar("cal-indicators", cx))
                            .cell_indicator(|date| {
                                [3, 7, 12, 15, 21, 28].contains(&date.day)
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Navigation Icons",
                    col(vec![h::Calendar::new(self.demo_calendar("cal-nav", cx))
                        .nav_icons(h::icons::ARROW_LEFT, h::icons::ARROW_RIGHT)
                        .into_any_element()]),
                ),
                (
                    "Real-World Example",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Pick an appointment"))
                        .child(
                            h::Calendar::new(self.demo_calendar("cal-real", cx))
                                .min_value(today)
                                .is_date_unavailable(|date| {
                                    let weekday = h::weekday_index(date);
                                    weekday == 0 || weekday == 6
                                })
                                .cell_indicator(|date| date.day % 5 == 0),
                        )
                        .child(h::Description::new(
                            "Weekends are unavailable; a dot marks a day with slots left.",
                        ))
                        .into_any_element()]),
                ),
                (
                    "Constraints", "minValue/maxValue mute the days outside the range; isDateUnavailable strikes through the ones it rejects.",
                    col(vec![
                        h::Calendar::new(self.calendar.clone())
                            .min_value(h::Date::new(today.year, today.month, 5))
                            .max_value(h::Date::new(today.year, today.month, 24))
                            .is_date_unavailable(|d: h::Date| d.day.is_multiple_of(7))
                            .into_any_element(),
                    ]),
                ),
                (
                    "First day of week",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .first_day_of_week(h::Weekday::Mon)
                        .weeks_in_month(6)
                        .into_any_element()]),
                ),
                (
                    "Read only & disabled",
                    row(vec![
                        h::Calendar::new(self.calendar.clone())
                            .is_read_only(true)
                            .into_any_element(),
                        h::Calendar::new(self.calendar.clone())
                            .is_disabled(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Multiple months",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .visible_duration(h::VisibleDuration::Months(2))
                        .into_any_element()]),
                ),
                (
                    "Week view",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .visible_duration(h::VisibleDuration::Weeks(2))
                        .page_behavior(h::PageBehavior::Single)
                        .into_any_element()]),
                ),
                (
                    "Day view",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .visible_duration(h::VisibleDuration::Days(5))
                        .into_any_element()]),
                ),
                (
                    "Year picker",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .is_year_picker_open(self.cal_year_picker)
                        .on_year_picker_open_change(bool_cb(cx.listener(
                            |this, open: &bool, _, cx| {
                                this.cal_year_picker = *open;
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
                (
                    "Heading Offset", "`Calendar.YearPickerTriggerHeading.offset` shifts the month heading -- also the year-picker trigger -- while the grid stays on the visible month. Both grids above show August; only the headings differ.",
                    col({
                        let august = h::Date::new(2026, 8, 10);
                        vec![
                            row(vec![
                                spec(
                                    "Same month",
                                    h::Calendar::new(self.demo_calendar("cal-heading-anchor", cx))
                                        .default_value(august)
                                        .into_any_element(),
                                    cx,
                                ),
                                spec(
                                    "Heading offset",
                                    h::Calendar::new(self.demo_calendar("cal-heading-offset", cx))
                                        .default_value(august)
                                        .offset(1)
                                        .into_any_element(),
                                    cx,
                                ),
                            ]),
                        ]
                    }),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let iso = self.date_iso;
        component_doc_page!(
            "Date Field",
            crate::pages::Page::DateField.description(),
            crate::pages::Page::DateField.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![
                        h::DateField::new(self.date_input.clone())
                            // v3's Usage seeds the field with `defaultValue`.
                            .default_value(h::Date::new(2025, 12, 25))
                            .label("Start date")
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.date_iso = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match iso {
                                Some(d) => format!("Parsed: {}", d.format_iso()),
                                None => {
                                    "Type digits, or step a segment with the arrow keys".to_owned()
                                }
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Granularity", "`granularity` sets the smallest unit the field shows. Below `day` it grows the time segments -- the same ones a `TimeField` has, so the arrows step them and digits type into them -- and the bound state holds an ISO date-and-time.",
                    col(vec![
                        spec_row(
                            h::Granularity::ALL
                                .iter()
                                .copied()
                                .map(|granularity| {
                                    let key = match granularity {
                                        h::Granularity::Day => "df-gran-day",
                                        h::Granularity::Hour => "df-gran-hour",
                                        h::Granularity::Minute => "df-gran-minute",
                                        h::Granularity::Second => "df-gran-second",
                                    };
                                    spec(
                                        granularity.label(),
                                        h::DateField::new(self.demo_text(
                                            key,
                                            "2025-02-03T08:45:09",
                                            cx,
                                        ))
                                        .granularity(granularity),
                                        cx,
                                    )
                                })
                                .collect()
                        ),
                        h::DateField::new(self.demo_text("df-gran-12h", "2025-02-03T20:45", cx))
                            .label("Twelve-hour clock")
                            .granularity(h::Granularity::Minute)
                            .hour_cycle(h::HourCycle::H12)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Forced Leading Zeros", "The system locale controls date and time segment order, separators, padding, and day-period names. The prop forces month, day, and hour segments to two digits.",
                    field_col(vec![
                        h::DateField::new(self.demo_text(
                            "df-leading-locale",
                            "2025-02-03T08:05:07",
                            cx,
                        ))
                        .label("System locale")
                        .granularity(h::Granularity::Second)
                        .hour_cycle(h::HourCycle::H12)
                        .into_any_element(),
                        h::DateField::new(self.demo_text(
                            "df-leading-forced",
                            "2025-02-03T08:05:07",
                            cx,
                        ))
                        .label("Forced two-digit fields")
                        .granularity(h::Granularity::Second)
                        .hour_cycle(h::HourCycle::H12)
                        .should_force_leading_zeros(true)
                        .into_any_element(),
                    ]),
                ),
                (
                    "With Icons",
                    field_col(vec![h::DateField::new(self.demo_text("df-icon", "", cx))
                        .label("Date")
                        .prefix(icon(h::icons::MOON, cx))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::DateField::new(self.demo_text("df-primary", "", cx))
                            .label("Primary")
                            .into_any_element(),
                        h::DateField::new(self.demo_text("df-secondary", "", cx))
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::DateField::new(self.demo_text("df-surface", "", cx))
                                .label("Date")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::DateField::new(self.demo_text("df-desc", "", cx))
                        .label("Date")
                        .description("Month, day and year")
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::DateField::new(self.demo_text("df-req", "", cx))
                        .label("Date")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::DateField::new(self.demo_text(
                        "df-dis",
                        "2025-12-25",
                        cx,
                    ))
                    .label("Date")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::DateField::new(self.demo_text("df-full", "", cx))
                        .label("Date")
                        .full_width(true)
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![h::DateField::new(self.demo_text(
                        "df-invalid",
                        "",
                        cx,
                    ))
                    .label("Date")
                    .is_required(true)
                    .is_invalid(true)
                    .validation_errors(["Pick a date"])
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::DateField::new(self.demo_text("df-ctl", "", cx))
                            .label("Date")
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.date_iso = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match iso {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Validation",
                    field_col(vec![h::DateField::new(self.demo_text(
                        "df-validate",
                        "",
                        cx,
                    ))
                    .label("Date")
                    .min_value(h::Date::new(2025, 1, 1))
                    .max_value(h::Date::new(2025, 12, 31))
                    .description("Must fall in 2025")
                    .into_any_element()]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let field = self.demo_text("df-form", "", cx);
                        h::Form::new()
                            .field(h::FormField::text(field.clone()).name("start"))
                            .child(
                                h::DateField::new(field)
                                    .label("Start date")
                                    .is_required(true),
                            )
                            .child(h::Button::new("df-form-submit").label("Save"))
                            .into_any_element()
                    }]),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.date_picker_open;
        let render_props_gallery = cx.entity().downgrade();
        component_doc_page!(
            "Date Picker",
            crate::pages::Page::DatePicker.description(),
            crate::pages::Page::DatePicker.import_line(),
            vec![
                (
                    "Disabled",
                    field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-disabled", cx),
                    )
                    .label("Date")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-controlled", cx))
                            .label("Date")
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match self.cal_picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Validation",
                    field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-invalid", cx),
                    )
                    .label("Date")
                    // `minValue`/`maxValue` bound the calendar: everything
                    // outside the range is unselectable.
                    .min_value(h::Date::new(2025, 12, 1))
                    .max_value(h::Date::new(2026, 6, 30))
                    .is_invalid(true)
                    .into_any_element()]),
                ),
                (
                    "Format Options", "The trigger follows the operating system's regional date order, separators, and numeric padding. Its state and submitted value stay ISO-formatted.",
                    col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-format", cx))
                            .label("Date")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![h::Form::new()
                        .child(
                            h::DatePicker::new(self.demo_calendar("dp-form", cx))
                                .label("Start date"),
                        )
                        .child(h::Button::new("dp-form-submit").label("Save"))
                        .into_any_element()]),
                ),
                (
                    "Custom Indicator", "v3 lets TriggerIndicator replace the default calendar glyph; this example uses a check without changing the trigger behavior.",
                    col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-indicator", cx))
                            .label("Date")
                            .trigger_indicator(icon(h::icons::CHECK, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Render Function",
                    field_col(vec![h::DatePicker::new(
                        self.demo_calendar("dp-render-props", cx)
                    )
                    .is_open(is_open)
                    .is_required(true)
                    .content(move |state| {
                        let gallery = render_props_gallery.clone();
                        gpui::div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(8.))
                            .child(format!(
                                "{} · {} · {}",
                                if state.is_required {
                                    "required"
                                } else {
                                    "optional"
                                },
                                if state.is_invalid { "invalid" } else { "valid" },
                                if state.is_open { "open" } else { "closed" },
                            ))
                            .child(
                                h::Button::new("dp-render-props-toggle")
                                    .label(if state.is_open { "Close" } else { "Open" })
                                    .on_press(move |_, _, cx| {
                                        if let Some(gallery) = gallery.upgrade() {
                                            gallery.update(cx, |gallery, cx| {
                                                gallery.date_picker_open =
                                                    !gallery.date_picker_open;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Usage",
                    field_col(vec![h::DatePicker::new(self.calendar.clone())
                        // v3's Usage seeds the picker with `defaultValue`.
                        .default_value(h::Date::new(2025, 12, 25))
                        .label("Due date")
                        .is_open(is_open)
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.date_picker_open = *open;
                            cx.notify();
                        })))
                        .on_change(opt_date_cb(cx.listener(
                            |this, d: &Option<h::Date>, _, cx| {
                                this.cal_picked = *d;
                                this.date_picker_open = false;
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_date_range_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.range_open;
        let render_props_gallery = cx.entity().downgrade();
        component_doc_page!(
            "Date Range Picker",
            crate::pages::Page::DateRangePicker.description(),
            crate::pages::Page::DateRangePicker.import_line(),
            vec![
                (
                    "Disabled",
                    field_col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-disabled", cx),
                    )
                    .label("Stay")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled", "The range lives in the state entity the caller owns.",
                    col(vec![
                        {
                            // `value` writes the caller's copy back in, and
                            // `on_change` is how the caller gets one: a range
                            // picker reports the change with no arguments, so
                            // the range is read out of the state entity.
                            let state = self.date_range.clone();
                            let held = cx.entity().downgrade();
                            let (start, end) = {
                                let range = self.date_range.read(cx);
                                (range.start, range.end)
                            };
                            h::DateRangePicker::new(self.date_range.clone())
                                .label("Stay")
                                .value(start, end, cx)
                                .on_change(move |_, cx| {
                                    let (start, end) = {
                                        let range = state.read(cx);
                                        (range.start, range.end)
                                    };
                                    if let Some(gallery) = held.upgrade() {
                                        gallery.update(cx, |gallery, cx| {
                                            gallery.set_demo_text_value(
                                                "drp-controlled",
                                                match (start, end) {
                                                    (Some(a), Some(b)) => format!(
                                                        "{} to {}",
                                                        a.format_iso(),
                                                        b.format_iso()
                                                    ),
                                                    _ => String::new(),
                                                },
                                            );
                                            cx.notify();
                                        });
                                    }
                                })
                                .into_any_element()
                        },
                    ]),
                ),
                (
                    "Validation",
                    field_col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-invalid", cx),
                    )
                    .label("Stay")
                    .is_invalid(true)
                    .into_any_element()]),
                ),
                (
                    "Format Options", "Both ends follow the operating system's regional date order, separators, and numeric padding. Their state and submitted values stay ISO-formatted.",
                    col(vec![
                        h::DateRangePicker::new(self.demo_range("drp-format", cx))
                            .label("Stay")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![h::Form::new()
                        .child(
                            h::DateRangePicker::new(self.demo_range("drp-form", cx))
                                .label("Stay")
                                // v3 submits a range as two named fields.
                                .start_name("check_in")
                                .end_name("check_out"),
                        )
                        .child(h::Button::new("drp-form-submit").label("Book"))
                        .into_any_element()]),
                ),
                (
                    "Custom Indicator", "v3 lets TriggerIndicator and RangeSeparator replace their default content without changing field or trigger behavior.",
                    col(vec![
                        h::DateRangePicker::new(self.demo_range("drp-indicator", cx))
                            .label("Stay")
                            .trigger_indicator(icon(h::icons::CHECK, cx))
                            .range_separator(gpui::div().child("to"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Render Function",
                    col(vec![h::DateRangePicker::new(
                        self.demo_range("drp-render-props", cx),
                    )
                    .is_open(is_open)
                    .is_required(true)
                    .content(move |state| {
                        let gallery = render_props_gallery.clone();
                        gpui::div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(8.))
                            .child(format!(
                                "{} · {} · {}",
                                if state.is_required {
                                    "required"
                                } else {
                                    "optional"
                                },
                                if state.is_invalid { "invalid" } else { "valid" },
                                if state.is_open { "open" } else { "closed" },
                            ))
                            .child(
                                h::Button::new("drp-render-props-toggle")
                                    .label(if state.is_open { "Close" } else { "Open" })
                                    .on_press(move |_, _, cx| {
                                        if let Some(gallery) = gallery.upgrade() {
                                            gallery.update(cx, |gallery, cx| {
                                                gallery.range_open = !gallery.range_open;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(320.))
                        .child(
                            h::DateRangePicker::new(self.date_range.clone())
                                .label("Trip dates")
                                // v3's Usage seeds the range and bounds it.
                                .default_value((
                                    h::Date::new(2025, 12, 8),
                                    h::Date::new(2025, 12, 14),
                                ))
                                .min_value(h::Date::new(2025, 1, 1))
                                .is_open(is_open)
                                .on_open_change(bool_cb(cx.listener(
                                    |this, open: &bool, _, cx| {
                                        this.range_open = *open;
                                        cx.notify();
                                    },
                                )))
                                .on_change(|_, _cx| {}),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_range_calendar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let today = h::Date::today();
        let focused = self.range_calendar_focus;
        component_doc_page!(
            "Range Calendar",
            crate::pages::Page::RangeCalendar.description(),
            crate::pages::Page::RangeCalendar.import_line(),
            vec![
                (
                    "Disabled",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-disabled", cx),
                    )
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Cell Indicators",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-dots", cx))
                        // `RangeCalendar.CellIndicator` marks a day with a dot,
                        // the same part a `Calendar` draws.
                        .cell_indicator(|d| d.day % 7 == 3)
                        .into_any_element()]),
                ),
                (
                    "Year Picker",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-year", cx))
                        .default_year_picker_open(true)
                        // `firstDayOfWeek` reorders the seven columns.
                        .first_day_of_week(h::Weekday::Mon)
                        .into_any_element()]),
                ),
                (
                    "Heading Offset", "`RangeCalendar.YearPickerTriggerHeading.offset` shifts the month heading -- also the year-picker trigger -- while the grid stays on the visible month. Both grids above show August; only the headings differ.",
                    col({
                        let august = (h::Date::new(2026, 8, 10), h::Date::new(2026, 8, 16));
                        vec![
                            row(vec![
                                spec(
                                    "Grid: August 2026; heading: August 2026 (offset 0)",
                                    h::RangeCalendar::new(self.demo_range("rc-heading-anchor", cx))
                                        .default_value(august)
                                        .into_any_element(),
                                    cx,
                                ),
                                spec(
                                    "Grid: August 2026; heading: September 2026 (offset +1)",
                                    h::RangeCalendar::new(self.demo_range("rc-heading-offset", cx))
                                        .default_value(august)
                                        .offset(1)
                                        .into_any_element(),
                                    cx,
                                ),
                            ]),
                        ]
                    }),
                ),
                (
                    "Default Value",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-default", cx),
                    )
                    .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                    .into_any_element()]),
                ),
                (
                    "Controlled", "The range lives in the state entity the caller owns.",
                    col(vec![
                        h::RangeCalendar::new(self.date_range.clone())
                            .value(
                                Some(h::Date::new(2025, 12, 8)),
                                Some(h::Date::new(2025, 12, 14)),
                                cx,
                            )
                            .on_focus_change(date_cb(cx.listener(|this, d: &h::Date, _, cx| {
                                this.set_demo_text_value("rc-focus", d.format_iso());
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Min and Max Dates",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-minmax", cx),
                    )
                    .min_value(h::Date::new(today.year, today.month, 5))
                    .max_value(h::Date::new(today.year, today.month, 24))
                    .into_any_element()]),
                ),
                (
                    "Unavailable Dates",
                    col({
                        let blocked_ranges = [
                            (h::add_days(&today, 2), h::add_days(&today, 5)),
                            (h::add_days(&today, 12), h::add_days(&today, 13)),
                        ];
                        vec![
                            h::RangeCalendar::new(self.demo_range("rc-unavailable", cx))
                                .default_value((h::add_days(&today, 6), h::add_days(&today, 9)))
                                .first_day_of_week(h::Weekday::Mon)
                                .is_date_unavailable(move |date, _| {
                                    let date = h::days_from_civil(&date);
                                    blocked_ranges.iter().any(|(start, end)| {
                                        date >= h::days_from_civil(start)
                                            && date <= h::days_from_civil(end)
                                    })
                                })
                                .into_any_element(),
                            gpui::div()
                                .child("Some days are unavailable")
                                .into_any_element(),
                        ]
                    }),
                ),
                (
                    "Anchor-Based Unavailable Dates", "After the first date is selected, earlier dates become unavailable because the predicate receives that active anchor.",
                    col(vec![
                        h::RangeCalendar::new(self.demo_range("rc-anchor", cx))
                            .is_date_unavailable(|date, anchor| {
                                anchor.is_some_and(|anchor| {
                                    (date.year, date.month, date.day)
                                        < (anchor.year, anchor.month, anchor.day)
                                })
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Allows Non-Contiguous Ranges",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-noncontig", cx),
                    )
                    .is_date_unavailable(|date, _| date.day == 15)
                    .allows_non_contiguous_ranges(true)
                    .into_any_element()]),
                ),
                (
                    "Weeks in Month",
                    col(vec![h::RangeCalendar::new(self.demo_range("rc-weeks", cx))
                        .weeks_in_month(6)
                        .into_any_element()]),
                ),
                (
                    "Week View",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-week-view", cx),
                    )
                    .visible_duration(h::VisibleDuration::Weeks(2))
                    .into_any_element()]),
                ),
                (
                    "Day View",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-day-view", cx),
                    )
                    .visible_duration(h::VisibleDuration::Days(5))
                    .into_any_element()]),
                ),
                (
                    "Multiple Months",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-months", cx),
                    )
                    .visible_duration(h::VisibleDuration::Months(2))
                    .into_any_element()]),
                ),
                (
                    "Read Only",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-readonly", cx),
                    )
                    .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                    .is_read_only(true)
                    .into_any_element()]),
                ),
                (
                    "Invalid",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-invalid", cx),
                    )
                    .default_value((h::Date::new(2025, 12, 8), h::Date::new(2025, 12, 14)))
                    .is_invalid(true)
                    .into_any_element()]),
                ),
                (
                    "Focused Value",
                    col(vec![h::RangeCalendar::new(
                        self.demo_range("rc-focused", cx),
                    )
                    .focused_value(focused)
                    .on_focus_change(date_cb(cx.listener(|this, date: &h::Date, _, cx| {
                        this.range_calendar_focus = *date;
                        cx.notify();
                    },)))
                    .into_any_element()]),
                ),
                (
                    "Real-World Example",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Choose your stay"))
                        .child(
                            h::RangeCalendar::new(self.demo_range("rc-real", cx))
                                .min_value(today)
                                .is_date_unavailable(|date, _| date.day == 20),
                        )
                        .child(h::Description::new("The 20th is fully booked."))
                        .into_any_element()]),
                ),
                (
                    "Usage",
                    col(vec![h::RangeCalendar::new(self.date_range.clone())
                        .on_change(|_start, _end, _, _cx| {})
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_time_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Time Field",
            crate::pages::Page::TimeField.description(),
            crate::pages::Page::TimeField.import_line(),
            vec![
                (
                    "24-hour",
                    field_col(vec![h::TimeField::new(self.time.clone())
                        .label("Start time")
                        .hour_cycle(h::HourCycle::H24)
                        .description("Click a segment, then use the steppers.")
                        .on_change(opt_time_cb(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        ))
                        .into_any_element()]),
                ),
                (
                    "12-hour with seconds",
                    field_col(vec![h::TimeField::new(self.time.clone())
                        .label("Reminder")
                        .hour_cycle(h::HourCycle::H12)
                        .show_seconds(true)
                        .on_change(opt_time_cb(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        ))
                        .into_any_element()]),
                ),
                (
                    "Forced Leading Zeros", "The system locale controls numeric padding; this prop only forces the hour to two digits.",
                    field_col(vec![
                        h::TimeField::new(self.demo_time("tmf-leading-locale", cx))
                            .label("Locale default")
                            .hour_cycle(h::HourCycle::H12)
                            .show_seconds(true)
                            .into_any_element(),
                        h::TimeField::new(self.demo_time("tmf-leading-forced", cx))
                            .label("Forced two-digit hour")
                            .hour_cycle(h::HourCycle::H12)
                            .show_seconds(true)
                            .should_force_leading_zeros(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage", "Uses your system regional segment order, separators, padding, day-period names, and 12- or 24-hour cycle.",
                    field_col(vec![
                        h::TimeField::new(self.demo_time("tmf-usage", cx))
                            .label("Time")
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Icons",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-icon", cx))
                        .label("Time")
                        .prefix(icon(h::icons::SUN, cx))
                        .into_any_element()]),
                ),
                (
                    "On Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::TimeField::new(self.demo_time("tmf-surface", cx))
                                .label("Time")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-desc", cx))
                        .label("Time")
                        .description("Hour and minute")
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-req", cx))
                        .label("Time")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-dis", cx))
                        .label("Time")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::TimeField::new(self.demo_time("tmf-full", cx))
                        .label("Time")
                        .full_width(true)
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-invalid", cx))
                        .label("Time")
                        // `minValue`/`maxValue` clamp what the segments accept.
                        .min_value(h::Time::new(9, 0))
                        .max_value(h::Time::new(17, 30))
                        .is_required(true)
                        .is_invalid(true)
                        .error_message("Pick a time")
                        .into_any_element()]),
                ),
                (
                    "Controlled", "The field owns the value; `on_change` reports each edit.",
                    col(vec![
                        h::TimeField::new(self.demo_time("tmf-ctl", cx))
                            .label("Time")
                            .on_change(opt_time_cb(
                                cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                            ))
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Validation",
                    field_col(vec![h::TimeField::new(self.demo_time("tmf-validate", cx))
                        .label("Meeting time")
                        .description("Office hours are 09:00 to 17:00")
                        .validate(|value| {
                            value
                                .filter(|t| t.hour < 9 || t.hour >= 17)
                                .map(|_| "Pick a time inside office hours".into())
                        })
                        .into_any_element()]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let state = self.demo_time("tmf-form", cx);
                        let field = h::TimeField::new(state)
                            .label("Start time")
                            .name("start_time")
                            .is_required(true);
                        h::Form::new()
                            .child(field)
                            .child(h::Button::new("tmf-form-submit").label("Save"))
                            .into_any_element()
                    }]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Feedback
    // -----------------------------------------------------------------------

    pub fn page_alert(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Alert",
            crate::pages::Page::Alert.description(),
            crate::pages::Page::Alert.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Alert::new("Heads up")
                        .description("This is an alert with a title and a description.")
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .map(|c| {
                            h::Alert::new(format!("{} alert", c.label()))
                                .description("Something worth reading happened.")
                                .status(*c)
                        })
                        .els()),
                ),
                (
                    "Closable", "v3 removed `isClosable`/`onClose`; a close affordance is an ordinary child, the way the pinned example composes a `CloseButton`.",
                    col(vec![
                        if self.alert_visible {
                            h::Alert::new("Saved")
                                .description("Your changes are live.")
                                .status(Color::Success)
                                .child(h::CloseButton::new("alert-closable-close").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.alert_visible = false;
                                        cx.notify();
                                    }),
                                ))
                                .into_any_element()
                        } else {
                            h::Button::new("alert-restore")
                                .label("Bring it back")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.alert_visible = true;
                                    cx.notify();
                                }))
                                .into_any_element()
                        },
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_meter(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.meter_value;
        component_doc_page!(
            "Meter",
            crate::pages::Page::Meter.description(),
            crate::pages::Page::Meter.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::Meter::new("meter-usage", value)
                                .label("Disk usage")
                                .show_value(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            fixed_demo(256., h::Meter::new(("meter-color", index), value).color(*c))
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, s)| {
                            fixed_demo(256., h::Meter::new(("meter-size", index), value).size(*s))
                        })
                        .els()),
                ),
                (
                    "Without Label",
                    col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-no-label", value)
                    )]),
                ),
                (
                    "Custom Value Scale",
                    col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-custom-scale", 320.)
                            .min_value(0.)
                            .max_value(500.)
                            .label("Storage")
                            .show_value(true)
                            .format_options(herogpui_core::NumberFormat::unit("GB")),
                    )]),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_bar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Progress Bar",
            crate::pages::Page::ProgressBar.description(),
            crate::pages::Page::ProgressBar.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::ProgressBar::new("progress-usage")
                                .value(65.0)
                                .label("Uploading")
                                .show_value_label(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            fixed_demo(
                                256.,
                                h::ProgressBar::new(("progress-color", index))
                                    .value(65.0)
                                    .color(*c),
                            )
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(vec![
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-sm")
                                .value(40.0)
                                .size(Size::Sm),
                        ),
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-md")
                                .value(60.0)
                                .size(Size::Md),
                        ),
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-lg")
                                .value(80.0)
                                .size(Size::Lg),
                        ),
                    ]),
                ),
                (
                    "Without Label",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-no-label").value(65.0),
                    )]),
                ),
                (
                    "Indeterminate",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-indeterminate")
                            .is_indeterminate(true)
                            .label("Uploading"),
                    )]),
                ),
                (
                    "Custom Value Scale",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-custom-scale")
                            .value(320.0)
                            .min_value(0.0)
                            .max_value(500.0)
                            .label("Downloaded")
                            .show_value_label(true)
                            .format_options(herogpui_core::NumberFormat::unit("MB")),
                    )]),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_circle(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Progress Circle",
            crate::pages::Page::ProgressCircle.description(),
            crate::pages::Page::ProgressCircle.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::ProgressCircle::new().value(60.).into_any_element()]),
                ),
                (
                    "Indeterminate",
                    row(vec![h::ProgressCircle::new()
                        .is_indeterminate(true)
                        .into_any_element()]),
                ),
                (
                    "With Label",
                    row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(h::ProgressCircle::new().value(75.))
                        .child(gpui::div().text_size(px(14.)).child("75% Complete"))
                        .into_any_element()]),
                ),
                (
                    "Custom SVG Props", "v3 overrides `strokeWidth` on the composed circle parts. The stroke here keeps v3's fixed 4/36 view-box ratio as the circle scales; custom SVG attributes are not exposed by this canvas-backed port.",
                    col(vec![
                        row(Size::ALL
                            .iter()
                            .map(|sz| {
                                spec(
                                    sz.label(),
                                    h::ProgressCircle::new().value(60.).size(*sz),
                                    cx,
                                )
                            })
                            .collect()),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::ProgressCircle::new().value(70.0).color(*c),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(vec![
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Sm)
                            .into_any_element(),
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Md)
                            .into_any_element(),
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Lg)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_skeleton(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Skeleton",
            crate::pages::Page::Skeleton.description(),
            crate::pages::Page::Skeleton.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w(px(250.))
                        .flex()
                        .flex_col()
                        .gap(px(20.))
                        .child(h::Skeleton::new().w(px(218.)).h(px(128.)))
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(12.))
                                .child(h::Skeleton::new().w(px(130.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(174.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(87.)).h(px(12.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "Text Content",
                    col(vec![gpui::div()
                        .w(px(420.))
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .children(
                            [1.0_f32, 0.83, 0.66, 1.0, 0.5]
                                .into_iter()
                                .map(|f| h::Skeleton::new().w(px(420. * f)).h(px(16.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "User Profile",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        // v3 rounds this one with `rounded-full`. Clipping it in
                        // the wrapper is the same result without inventing a
                        // per-instance radius prop v3 does not have.
                        .child(
                            gpui::div()
                                .rounded_full()
                                .overflow_hidden()
                                .flex_shrink_0()
                                .child(h::Skeleton::new().w(px(40.)).h(px(40.))),
                        )
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(h::Skeleton::new().w(px(144.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(96.)).h(px(12.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "List Items",
                    col((0..3)
                        .map(|_| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(12.))
                                .child(h::Skeleton::new().w(px(40.)).h(px(40.)))
                                .child(
                                    gpui::div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.))
                                        .child(h::Skeleton::new().w(px(320.)).h(px(12.)))
                                        .child(h::Skeleton::new().w(px(256.)).h(px(12.))),
                                )
                        })
                        .els()),
                ),
                (
                    "Grid",
                    col(vec![gpui::div()
                        .flex()
                        .flex_wrap()
                        .gap(px(16.))
                        .children((0..6).map(|_| h::Skeleton::new().w(px(130.)).h(px(96.))))
                        .into_any_element()]),
                ),
                (
                    "Single Shimmer", "v3 runs one shimmer across a whole group by putting the animation on the parent and turning it off on each child.",
                    col(vec![
                        gpui::div()
                            .flex()
                            .gap(px(16.))
                            .children((0..3).map(|_| {
                                h::Skeleton::new()
                                    .w(px(130.))
                                    .h(px(96.))
                                    .animation_type(herogpui_theme::SkeletonAnimation::None)
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Animation Types",
                    row(vec![
                        spec(
                            "Shimmer",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Shimmer),
                            cx,
                        ),
                        spec(
                            "Pulse",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Pulse),
                            cx,
                        ),
                        spec(
                            "None",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::None),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Loading",
                    col(vec![
                        h::Skeleton::new().w(px(320.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(260.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(180.)).h(px(16.)).into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_spinner(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Spinner",
            crate::pages::Page::Spinner.description(),
            crate::pages::Page::Spinner.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Spinner::new("sp-usage").into_any_element()]),
                ),
                (
                    "Speed",
                    row(vec![
                        spec("Slow", h::Spinner::new("sp-slow").duration_ms(1500), cx),
                        spec("Default", h::Spinner::new("sp-default"), cx),
                        spec("Fast", h::Spinner::new("sp-fast").duration_ms(500), cx),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::Spinner::new(el_id(format!("sp-{c:?}"))).color(*c),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(h::SpinnerSize::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::Spinner::new(el_id(format!("sp-sz-{s:?}"))).size(*s),
                                cx,
                            )
                        })
                        .collect()),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Forms
    // -----------------------------------------------------------------------

    pub fn page_checkbox(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let (basic, colored) = (self.cb_basic, self.cb_color);
        let cb_controlled = self.demo_flag("cb-controlled", false);
        component_doc_page!(
            "Checkbox",
            crate::pages::Page::Checkbox.description(),
            crate::pages::Page::Checkbox.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Checkbox::new("cb-1")
                            .is_selected(basic)
                            .label(gpui::div().child("Accept the terms"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.cb_basic = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                        h::Checkbox::new("cb-2")
                            .is_selected(colored)
                            .variant(FieldVariant::Secondary)
                            .label(gpui::div().child("Subscribe to updates"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.cb_color = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::Checkbox::new("cb-v-primary")
                            .default_selected(true)
                            .label(gpui::div().child("Primary"))
                            .into_any_element(),
                        h::Checkbox::new("cb-v-secondary")
                            .default_selected(true)
                            .variant(FieldVariant::Secondary)
                            .label(gpui::div().child("Secondary"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Full Rounded",
                    col(vec![
                        h::Checkbox::new("cb-round-1")
                            .is_round(true)
                            .default_selected(true)
                            .label(gpui::div().child("Round control"))
                            .into_any_element(),
                        h::Checkbox::new("cb-round-2")
                            .is_round(true)
                            .label(gpui::div().child("Round, unchecked"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "External Label",
                    row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(h::Checkbox::new("cb-external"))
                        .child(h::Label::new("Send me marketing emails"))
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    col(vec![h::Checkbox::new("cb-desc")
                        .default_selected(true)
                        .label(gpui::div().child("Weekly digest"))
                        .description("One email every Monday morning.")
                        .into_any_element()]),
                ),
                (
                    "Default Selected",
                    col(vec![h::Checkbox::new("cb-default")
                        .default_selected(true)
                        .label(gpui::div().child("On by default"))
                        .into_any_element()]),
                ),
                (
                    "Invalid",
                    col(vec![h::Checkbox::new("cb-invalid")
                        .is_required(true)
                        .is_invalid(true)
                        .validation_errors(["You must accept the terms"])
                        .label(gpui::div().child("Accept the terms"))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Checkbox::new("cb-controlled")
                            .is_selected(cb_controlled)
                            .label(gpui::div().child("Notifications"))
                            .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("cb-controlled", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(
                            if cb_controlled {
                                "Status: selected"
                            } else {
                                "Status: not selected"
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Form Integration",
                    col(vec![h::Checkbox::new("cb-form")
                        .name("terms")
                        .value("accepted")
                        .is_required(true)
                        .label(gpui::div().child("Accept the terms"))
                        .into_any_element()]),
                ),
                (
                    "Render Props",
                    col(vec![h::Checkbox::new("cb-render")
                        .is_selected(cb_controlled)
                        .on_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                            this.set_demo_flag("cb-controlled", *v);
                            cx.notify();
                        })))
                        .content(|state| {
                            gpui::div()
                                .child(if state.is_selected {
                                    "Terms accepted"
                                } else {
                                    "Accept terms"
                                })
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Custom Indicator",
                    row(vec![
                        h::Checkbox::new("cb-ind-heart")
                            .default_selected(true)
                            .indicator(move |state| {
                                if state.is_selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::HEART_FILL)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("Heart"))
                            .into_any_element(),
                        h::Checkbox::new("cb-ind-plus")
                            .default_selected(true)
                            .indicator(move |state| {
                                if state.is_selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::PLUS)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("Plus"))
                            .into_any_element(),
                        h::Checkbox::new("cb-ind-minus")
                            .is_indeterminate(true)
                            .indicator(move |state| {
                                if state.is_indeterminate {
                                    gpui::div()
                                        .w(px(10.))
                                        .h(px(2.))
                                        .rounded_full()
                                        .bg(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("Indeterminate"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Indeterminate & disabled",
                    row(vec![
                        h::Checkbox::new("cb-ind")
                            .is_indeterminate(true)
                            .into_any_element(),
                        h::Checkbox::new("cb-dis")
                            .is_selected(true)
                            .is_disabled(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_checkbox_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.checkbox_group.clone();
        let group_options = || {
            vec![
                h::CheckboxOption::new("email", "Email").description("Product news and offers"),
                h::CheckboxOption::new("sms", "SMS"),
                h::CheckboxOption::new("push", "Push").is_disabled(true),
            ]
        };
        let options = group_options();
        component_doc_page!(
            "Checkbox Group",
            crate::pages::Page::CheckboxGroup.description(),
            crate::pages::Page::CheckboxGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::CheckboxGroup::new("cbg-usage", group_options())
                        .label("Notifications")
                        .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::CheckboxGroup::new("cbg-surface", group_options())
                                .label("Notifications")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::CheckboxGroup::new("cbg-disabled", group_options())
                        .label("Notifications")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Indeterminate", "v3 pairs a \"select all\" checkbox with the group: it is indeterminate while only some children are selected.",
                    col(vec![
                        h::Checkbox::new("cbg-all")
                            .is_selected(selected.len() == 3)
                            .is_indeterminate(!selected.is_empty() && selected.len() < 3)
                            .label(gpui::div().child("All notifications"))
                            .on_change(bool_cb(cx.listener(|this, all: &bool, _, cx| {
                                this.checkbox_group = if *all {
                                    ["email", "sms", "push"]
                                        .into_iter()
                                        .map(SharedString::from)
                                        .collect()
                                } else {
                                    HashSet::new()
                                };
                                cx.notify();
                            })))
                            .into_any_element(),
                        h::CheckboxGroup::new("cbg-ind", group_options())
                            .value(selected.iter().cloned())
                            .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                                this.checkbox_group = keys.clone();
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Validation",
                    col(vec![h::CheckboxGroup::new("cbg-validate", group_options())
                        .label("Notifications")
                        .is_required(true)
                        .is_invalid(selected.is_empty())
                        .error_message("Pick at least one channel")
                        .value(selected.iter().cloned())
                        .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.checkbox_group = keys.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Features and Add-ons Example",
                    col(vec![h::CheckboxGroup::new(
                        "cbg-addons",
                        vec![
                            h::CheckboxOption::new("analytics", "Analytics")
                                .description("Usage dashboards and funnels"),
                            h::CheckboxOption::new("backups", "Daily backups")
                                .description("Restore any of the last 30 days"),
                            h::CheckboxOption::new("sso", "Single sign-on")
                                .description("SAML and OIDC"),
                        ],
                    )
                    .label("Add-ons")
                    .description("Billed monthly, cancel any time.")
                    .into_any_element()]),
                ),
                (
                    "With Custom Indicator", "A custom indicator belongs to a standalone Checkbox, not to this port's group options; here two standalone checkboxes draw hearts.",
                    col(vec![
                        h::Checkbox::new("cbg-ci-1")
                            .default_selected(true)
                            .indicator(move |state| {
                                if state.is_selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::HEART_FILL)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("Email"))
                            .into_any_element(),
                        h::Checkbox::new("cbg-ci-2")
                            .indicator(move |state| {
                                if state.is_selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::HEART_FILL)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("SMS"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![h::CheckboxGroup::new("cbg-v", options.clone())
                        .label("Notifications")
                        .description("Pick how we reach you.")
                        .value(selected.iter().cloned())
                        .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.checkbox_group = keys.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Horizontal & invalid",
                    col(vec![h::CheckboxGroup::new("cbg-h", options)
                        .label("Channels")
                        .orientation(Orientation::Horizontal)
                        .error_message("Choose at least two channels.")
                        .value(selected.iter().cloned())
                        .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.checkbox_group = keys.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    col(vec![h::CheckboxGroup::new("cbg-unc", group_options())
                        .label("Channels")
                        .default_value(vec![SharedString::from("email")])
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_fieldset(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Fieldset",
            crate::pages::Page::Fieldset.description(),
            crate::pages::Page::Fieldset.import_line(),
            vec![
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::Fieldset::new()
                                .child(h::FieldsetLegend::new("Profile"))
                                .child(
                                    h::FieldGroup::new()
                                        .child(
                                            h::TextField::new(self.demo_text("fset-name", "", cx))
                                                .label("Name")
                                                .placeholder("Ada Lovelace")
                                                .variant(FieldVariant::Secondary),
                                        )
                                        .child(
                                            h::TextField::new(self.demo_text("fset-email", "", cx))
                                                .label("Email")
                                                .placeholder("ada@example.com")
                                                .variant(FieldVariant::Secondary),
                                        ),
                                )
                                .child(h::FieldsetActions::new().child(
                                    h::Button::new("fset-save").label("Save").size(Size::Sm),
                                )),
                        )
                        .into_any_element()]),
                ),
                (
                    "Usage",
                    field_col(vec![h::Fieldset::new()
                        .child(h::FieldsetLegend::new("Shipping address"))
                        .child(
                            h::FieldGroup::new()
                                .child(
                                    h::TextField::new(self.input_name.clone())
                                        .label("Street")
                                        .placeholder("221B Baker Street"),
                                )
                                .child(
                                    h::TextField::new(self.input_email.clone())
                                        .label("City")
                                        .placeholder("London"),
                                ),
                        )
                        .child(
                            h::FieldsetActions::new()
                                .child(
                                    h::Button::new("fs-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary),
                                )
                                .child(h::Button::new("fs-save").label("Save")),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_field_slots(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Label & Messages",
            crate::pages::Page::FieldSlots.description(),
            crate::pages::Page::FieldSlots.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Label::new("Email").into_any_element(),
                        h::Description::new("We will never share your address.").into_any_element(),
                        h::ErrorMessage::new("Enter a valid email address.").into_any_element(),
                    ]),
                ),
                (
                    "With Required Indicator",
                    col(vec![h::Label::new("Email")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "With Disabled State",
                    col(vec![h::Label::new("Email")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "With Invalid State",
                    col(vec![h::Label::new("Email")
                        .is_invalid(true)
                        .into_any_element()]),
                ),
                (
                    "With Form Fields",
                    field_col(vec![h::TextField::new(self.demo_text(
                        "fs-with-field",
                        "",
                        cx,
                    ))
                    .label("Email")
                    .placeholder("Enter your email")
                    .input_type(h::InputType::Email)
                    .description("We will never share your email")
                    .into_any_element()]),
                ),
                (
                    "Integration with TextField", "A `TextField` composes all three parts itself: the label above, the input, and the description or the error message below.",
                    col(vec![
                        demo_field(
                            h::TextField::new(self.demo_text("fs-integration", "", cx))
                                .label("Email")
                                .placeholder("Enter your email")
                                .description("We will never share your email"),
                        ),
                    ]),
                ),
                (
                    "Basic Validation",
                    field_col(vec![h::TextField::new(self.demo_text(
                        "fs-validate",
                        "",
                        cx,
                    ))
                    .label("Password")
                    .placeholder("••••••••")
                    .input_type(h::InputType::Password)
                    .is_required(true)
                    .validate(|value| {
                        (value.chars().count() < 8).then(|| "Use at least 8 characters".into())
                    })
                    .into_any_element()]),
                ),
                (
                    "With Dynamic Messages", "v3's `FieldError` takes a render prop and joins `validation.validationErrors`. `validationErrors` here is a list, and the field shows them in order.",
                    col(vec![
                        h::TextField::new(self.demo_text("fs-dynamic", "abc", cx))
                            .label("Password")
                            .is_invalid(true)
                            .validation_errors(["Use at least 8 characters", "Include a digit"])
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Validation Logic",
                    col(vec![h::TextField::new(self.demo_text("fs-custom", "", cx))
                        .label("Username")
                        .description("Letters, digits and dashes only")
                        .validate(|value| {
                            (!value.is_empty()
                                && !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
                            .then(|| "Letters, digits and dashes only".into())
                        })
                        .into_any_element()]),
                ),
                (
                    "Multiple Error Messages",
                    col(vec![h::TextField::new(self.demo_text("fs-multi", "", cx))
                        .label("Password")
                        .is_invalid(true)
                        .validation_errors([
                            "Use at least 8 characters",
                            "Include an uppercase letter",
                            "Include a digit",
                        ])
                        .into_any_element()]),
                ),
                (
                    "Label",
                    col(vec![
                        h::Label::new("Email").into_any_element(),
                        h::Label::new("Email").is_required(true).into_any_element(),
                        h::Label::new("Email").is_invalid(true).into_any_element(),
                        h::Label::new("Email").is_disabled(true).into_any_element(),
                    ]),
                ),
                (
                    "Description & error",
                    col(vec![
                        h::Description::new("We will never share your address.").into_any_element(),
                        h::ErrorMessage::new("Enter a valid email address.").into_any_element(),
                    ]),
                ),
                (
                    "FieldError", "A FieldError with no message renders nothing.",
                    col(vec![
                        h::FieldError::new()
                            .message("This field is required.")
                            .into_any_element(),
                        h::FieldError::new().into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_form(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let submitted = self.input_submitted.clone();
        component_doc_page!(
            "Form",
            crate::pages::Page::Form.description(),
            crate::pages::Page::Form.import_line(),
            vec![
                (
                    "Usage", "The wired Submit button and Enter in a focused field run the same submission: with the required Name empty, either door reports the invalid path instead.",
                    col(vec![
                        {
                            // `name` rides on each field's state, so the form finds
                            // it without the call site repeating the name.
                            let form = h::Form::new()
                                .field(
                                    h::FormField::text(self.input_name.clone())
                                        .is_required(true)
                                        .default_text(self.input_name.clone(), ""),
                                )
                                .field(
                                    h::FormField::text(self.input_email.clone())
                                        .default_text(self.input_email.clone(), ""),
                                )
                                .on_submit(cx.listener(|this, data: &h::FormData, _, cx| {
                                    this.input_submitted = data
                                        .iter()
                                        .map(|(n, v)| format!("{n}={}", v.as_text()))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    cx.notify();
                                }))
                                .on_invalid(cx.listener(|this, _: &h::FormData, _, cx| {
                                    this.input_submitted = "Name is required".to_owned();
                                    cx.notify();
                                }))
                                .on_reset({
                                    let l = cx.listener(
                                        |this: &mut Self, _: &(), _: &mut gpui::Window, cx| {
                                            this.input_submitted = String::new();
                                            cx.notify();
                                        },
                                    );
                                    move |w: &mut gpui::Window, cx: &mut gpui::App| l(&(), w, cx)
                                });
                            let submit = form.submit_handler();
                            let reset = form.reset_handler();
                            form.child(
                                h::TextField::new(self.input_name.clone())
                                    .name("name")
                                    .label("Name")
                                    .placeholder("Ada Lovelace")
                                    .is_required(true),
                            )
                            .child(
                                h::TextField::new(self.input_email.clone())
                                    .name("email")
                                    .label("Email")
                                    .placeholder("ada@example.com")
                                    .description("We reply within a day."),
                            )
                            .child(
                                gpui::div()
                                    .flex()
                                    .gap(px(8.))
                                    .child(
                                        h::Button::new("form-submit")
                                            .label("Submit")
                                            .on_press(move |_, w, cx| submit(w, cx)),
                                    )
                                    .child(
                                        h::Button::new("form-reset")
                                            .label("Reset")
                                            .variant(Variant::Tertiary)
                                            .on_press(move |_, w, cx| reset(w, cx)),
                                    ),
                            )
                            .into_any_element()
                        },
                        para(
                            &if submitted.is_empty() {
                                "Nothing submitted yet".to_owned()
                            } else {
                                format!("Submitted: {submitted}")
                            },
                            cx,
                        ),
                    ]),
                ),
                ("Server Errors", "`validationErrors` is HeroUI's `ValidationErrors` record — server errors keyed by field name, `Record<string, string | string[]>`. The Form routes each name into that field's own error slot: editing a field clears only its message while its sibling keeps theirs, Reset hides them all, and a re-render that passes the same record re-arms nothing. New response supplies a genuinely new record — identical content, fresh identity — so both messages re-arm.", {
                    let email = self.demo_text("form-srv-email", "ada@example.com", cx);
                    let name = self.demo_text("form-srv-name", "Ada", cx);
                    let report = self.demo_text_value("form-srv-report");
                    FORM_SERVER_RECORD.with_borrow_mut(|slot| {
                        slot.get_or_insert_with(|| {
                            h::ValidationErrors::new()
                                .set("email", "Already registered")
                                .set("name", "That name is taken")
                        });
                    });
                    // A clone per frame keeps the record's identity, so the
                    // page's own re-renders never re-arm a field the user has
                    // edited; only the New response button mints a record.
                    let record = FORM_SERVER_RECORD
                        .with_borrow(|slot| slot.as_ref().expect("seeded below").clone());
                    let form = h::Form::new()
                        .validation_errors(record)
                        .field(h::FormField::text(email.clone()))
                        .field(h::FormField::text(name.clone()))
                        .on_submit(cx.listener(|this, data: &h::FormData, _, cx| {
                            let body = data
                                .iter()
                                .map(|(n, v)| format!("{n}={}", v.as_text()))
                                .collect::<Vec<_>>()
                                .join(", ");
                            this.set_demo_text_value("form-srv-report", body);
                            cx.notify();
                        }))
                        .on_invalid(cx.listener(|this, _: &h::FormData, _, cx| {
                            this.set_demo_text_value(
                                "form-srv-report",
                                "onInvalid: a routed server error is still blocking".to_owned(),
                            );
                            cx.notify();
                        }));
                    let submit = form.submit_handler();
                    let reset = form.reset_handler();
                    col(vec![
                        form.child(
                            h::TextField::new(email)
                                .name("email")
                                .label("Email")
                                .description("Edit this and only its server message clears."),
                        )
                        .child(h::TextField::new(name).name("name").label("Name"))
                        .child(
                            gpui::div()
                                .flex()
                                .gap(px(8.))
                                .child(
                                    h::Button::new("form-srv-submit")
                                        .label("Submit")
                                        .on_press(move |_, w, cx| submit(w, cx)),
                                )
                                .child(
                                    h::Button::new("form-srv-reset")
                                        .label("Reset")
                                        .variant(Variant::Tertiary)
                                        .on_press(move |_, w, cx| reset(w, cx)),
                                )
                                .child(
                                    h::Button::new("form-srv-rearm")
                                        .label("New response")
                                        .on_press(|_, window, _| {
                                            FORM_SERVER_RECORD.with_borrow_mut(|slot| {
                                                // Same content, fresh record: a
                                                // new response re-arms every
                                                // named field.
                                                *slot = Some(
                                                    h::ValidationErrors::new()
                                                        .set("email", "Already registered")
                                                        .set("name", "That name is taken"),
                                                );
                                            });
                                            window.refresh();
                                        }),
                                ),
                        )
                        .into_any_element(),
                        para(
                            &if report.is_empty() {
                                "Submit while a message is showing and onInvalid runs.".to_owned()
                            } else {
                                report
                            },
                            cx,
                        ),
                    ])
                },),
            ],
            cx,
        )
    }

    pub fn page_input(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let input_controlled = self
            .demo_text("in-controlled", "", cx)
            .read(cx)
            .value()
            .to_owned();
        component_doc_page!(
            "Input",
            crate::pages::Page::Input.description(),
            crate::pages::Page::Input.import_line(),
            vec![
                (
                    "Variants",
                    field_col(vec![
                        h::Input::new(self.demo_text("in-variant-primary", "", cx))
                            .label(FieldVariant::Primary.label())
                            .placeholder("Primary input")
                            .variant(FieldVariant::Primary)
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-variant-secondary", "", cx))
                            .label(FieldVariant::Secondary.label())
                            .placeholder("Secondary input")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",
                    field_col(vec![h::Input::new(self.demo_text("in-usage", "", cx))
                        .label("Name")
                        .placeholder("Enter your name")
                        .into_any_element()]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::Input::new(self.demo_text("in-surface", "", cx))
                                .label("Name")
                                .placeholder("Your name")
                                .variant(FieldVariant::Secondary)
                                .description("The lower-emphasis variant, for use on a surface"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::Input::new(self.demo_text("in-full", "", cx))
                        .label("Name")
                        .placeholder("Full width input")
                        .full_width()
                        .into_any_element()]),
                ),
                (
                    "Input Types",
                    field_col(vec![
                        h::Input::new(self.demo_text("in-pw", "", cx))
                            .label("Password")
                            .input_type(h::InputType::Password)
                            .placeholder("Secret")
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-num", "", cx))
                            .label("Age")
                            .input_type(h::InputType::Number)
                            // `min`/`max` bound a numeric input, which is what
                            // its validity is checked against.
                            .min(18.)
                            .max(120.)
                            .placeholder("21")
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-email", "", cx))
                            .label("Email")
                            .input_type(h::InputType::Email)
                            .placeholder("user@example.com")
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-url", "", cx))
                            .label("Website")
                            .input_type(h::InputType::Url)
                            .placeholder("https://example.com")
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-tel", "", cx))
                            .label("Phone")
                            .input_type(h::InputType::Tel)
                            .placeholder("+1 (555) 000-0000")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        demo_field(
                            h::Input::new(self.demo_text("in-controlled", "", cx))
                                .label("Name")
                                .placeholder("Enter your name")
                                .on_change(|_, _, _| {}),
                        ),
                        para(&format!("Value: {input_controlled}"), cx),
                    ]),
                ),
                (
                    "States",
                    field_col(vec![
                        h::Input::new(self.demo_text("in-required", "", cx))
                            .label("Required")
                            .placeholder("Enter a value")
                            .is_required(true)
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-invalid", "", cx))
                            .label("Invalid")
                            .placeholder("Taken name")
                            .error_message("That name is taken.")
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-disabled", "", cx))
                            .label("Disabled")
                            .placeholder("Unavailable")
                            .is_disabled(true)
                            .into_any_element(),
                        h::Input::new(self.demo_text("in-clearable", "Ada", cx))
                            .label("Clearable")
                            .placeholder("Ada Lovelace")
                            .is_clearable(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_input_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let ig_reveal = self.demo_flag("ig-reveal", false);
        component_doc_page!(
            "Input Group",
            crate::pages::Page::InputGroup.description(),
            crate::pages::Page::InputGroup.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .prefix(h::InputAddon::new("https://"))
                        .input(
                            h::Input::new(self.demo_text("ig-usage", "", cx))
                                // v3's group example seeds the input with
                                // `defaultValue`; `value` is the controlled
                                // spelling of the same thing.
                                .default_value("heroui.com")
                                .placeholder("heroui.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    field_col(vec![
                        h::InputGroup::new()
                            .label("Primary")
                            .prefix(h::InputAddon::new("@"))
                            .input(
                                h::Input::new(self.demo_text("ig-v-primary", "", cx))
                                    .placeholder("name@email.com"),
                            )
                            .into_any_element(),
                        h::InputGroup::new()
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .prefix(h::InputAddon::new("@"))
                            .input(
                                h::Input::new(self.demo_text("ig-v-secondary", "", cx))
                                    .placeholder("name@email.com"),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::InputGroup::new()
                                .label("Handle")
                                .variant(FieldVariant::Secondary)
                                .prefix(h::InputAddon::new("@"))
                                .input(
                                    h::Input::new(self.demo_text("ig-surface", "", cx))
                                        .placeholder("name@email.com"),
                                ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Loading State",
                    field_col(vec![h::InputGroup::new()
                        .label("Checking availability")
                        .input(h::Input::new(self.demo_text("ig-loading", "heroui", cx)))
                        .suffix(
                            gpui::div()
                                .pr(px(8.))
                                .child(h::Spinner::new("ig-spinner").size(h::SpinnerSize::Sm)),
                        )
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .is_required(true)
                        .prefix(h::InputAddon::new("https://"))
                        .input(
                            h::Input::new(self.demo_text("ig-required", "", cx))
                                .placeholder("name@email.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .is_disabled(true)
                        .prefix(h::InputAddon::new("https://"))
                        .input(
                            h::Input::new(self.demo_text("ig-disabled", "heroui.com", cx))
                                .is_disabled(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::InputGroup::new()
                        .label("Website")
                        .full_width(true)
                        .prefix(h::InputAddon::new("https://"))
                        .input(
                            h::Input::new(self.demo_text("ig-full", "", cx))
                                .placeholder("name@email.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Text Prefix",
                    field_col(vec![h::InputGroup::new()
                        .prefix(h::InputAddon::new("https://"))
                        .input(
                            h::Input::new(self.demo_text("ig-text-prefix", "", cx))
                                .placeholder("heroui.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Text Suffix",
                    field_col(vec![h::InputGroup::new()
                        .input(
                            h::Input::new(self.demo_text("ig-text-suffix", "", cx))
                                .placeholder("heroui"),
                        )
                        .suffix(h::InputAddon::new(".com"))
                        .into_any_element()]),
                ),
                (
                    "Icon Prefix and Text Suffix",
                    field_col(vec![h::InputGroup::new()
                        .prefix(gpui::div().pl(px(12.)).child(icon(h::icons::MAIL, cx)))
                        .input(
                            h::Input::new(self.demo_text("ig-icon-text", "", cx))
                                .placeholder("heroui"),
                        )
                        .suffix(h::InputAddon::new("@heroui.com"))
                        .into_any_element()]),
                ),
                (
                    "Copy Button Suffix",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .input(h::Input::new(self.demo_text("ig-copy", "heroui.com", cx)))
                        .suffix(
                            gpui::div().pr(px(4.)).child(
                                h::Button::new("ig-copy-btn")
                                    .is_icon_only(true)
                                    .variant(Variant::Ghost)
                                    .size(Size::Sm)
                                    .child(icon(h::icons::COPY, cx)),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Icon Prefix and Copy Button",
                    field_col(vec![h::InputGroup::new()
                        .prefix(gpui::div().pl(px(12.)).child(icon(h::icons::KEY, cx)))
                        .input(h::Input::new(self.demo_text(
                            "ig-key",
                            "sk_live_51H...",
                            cx,
                        )))
                        .suffix(
                            gpui::div().pr(px(4.)).child(
                                h::Button::new("ig-key-copy")
                                    .is_icon_only(true)
                                    .variant(Variant::Ghost)
                                    .size(Size::Sm)
                                    .child(icon(h::icons::COPY, cx)),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Password Toggle",
                    field_col(vec![h::InputGroup::new()
                        .label("Password")
                        .input(
                            h::Input::new(self.demo_text("ig-pw", "correct horse", cx)).input_type(
                                if ig_reveal {
                                    h::InputType::Text
                                } else {
                                    h::InputType::Password
                                },
                            ),
                        )
                        .suffix(
                            gpui::div().pr(px(4.)).child(
                                h::Button::new("ig-pw-toggle")
                                    .is_icon_only(true)
                                    .variant(Variant::Ghost)
                                    .size(Size::Sm)
                                    .child(icon(
                                        if ig_reveal {
                                            h::icons::EYE_OFF
                                        } else {
                                            h::icons::EYE
                                        },
                                        cx,
                                    ))
                                    .on_press(cx.listener(move |this, _, _, cx| {
                                        this.set_demo_flag("ig-reveal", !ig_reveal);
                                        cx.notify();
                                    })),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Keyboard Shortcut",
                    field_col(vec![h::InputGroup::new()
                        .prefix(gpui::div().pl(px(12.)).child(icon(h::icons::SEARCH, cx)))
                        .input(
                            h::Input::new(self.demo_text("ig-kbd", "", cx)).placeholder("Search"),
                        )
                        .suffix(
                            gpui::div()
                                .pr(px(8.))
                                .flex()
                                .gap(px(4.))
                                .child(h::Kbd::new().variant(h::KbdVariant::Light).child("Ctrl"))
                                .child(h::Kbd::new().variant(h::KbdVariant::Light).child("K")),
                        )
                        .into_any_element()]),
                ),
                (
                    "Badge Suffix",
                    field_col(vec![h::InputGroup::new()
                        .label("Plan")
                        .input(h::Input::new(self.demo_text("ig-badge", "Pro", cx)))
                        .suffix(
                            gpui::div().pr(px(8.)).child(
                                h::Chip::new()
                                    .size(Size::Sm)
                                    .variant(h::ChipVariant::Soft)
                                    .color(Color::Accent)
                                    .child(h::ChipLabel::new().child("Trial")),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .is_required(true)
                        .is_invalid(true)
                        .error_message("Enter a valid URL")
                        .prefix(h::InputAddon::new("https://"))
                        .input(h::Input::new(self.demo_text("ig-invalid", "not a url", cx)))
                        .into_any_element()]),
                ),
                (
                    "With Prefix Icon",
                    field_col(vec![h::InputGroup::new()
                        .prefix(gpui::div().pl(px(12.)).child(icon(h::icons::GLOBE, cx)))
                        .input(
                            h::Input::new(self.demo_text("ig-prefix-icon", "", cx))
                                .placeholder("name@email.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Suffix Icon",
                    field_col(vec![h::InputGroup::new()
                        .input(
                            h::Input::new(self.demo_text("ig-suffix-icon", "", cx))
                                .placeholder("name@email.com"),
                        )
                        .suffix(gpui::div().pr(px(12.)).child(icon(h::icons::CHECK, cx)))
                        .into_any_element()]),
                ),
                (
                    "With Prefix and Suffix",
                    field_col(vec![h::InputGroup::new()
                        .prefix(gpui::div().pl(px(12.)).child(icon(h::icons::SEARCH, cx)))
                        .input(
                            h::Input::new(self.demo_text("ig-both", "", cx))
                                .placeholder("Search..."),
                        )
                        .suffix(gpui::div().pr(px(12.)).child(icon(h::icons::CLOSE, cx)))
                        .into_any_element()]),
                ),
                (
                    "With TextArea",
                    field_col(vec![h::InputGroup::new()
                        .label("Note")
                        .prefix(
                            // The group's `:has(textarea)` rule owns the 8px
                            // addon top padding; only the addon's horizontal
                            // inset is spelled here, as on every bare-icon
                            // addon slot.
                            gpui::div().pl(px(12.)).child(icon(h::icons::COPY, cx)),
                        )
                        .text_area(
                            h::TextArea::new(self.demo_text("ig-area", "", cx))
                                .placeholder("Assign tasks or ask anything...")
                                .rows(3),
                        )
                        .into_any_element()]),
                ),
                (
                    "Usage Example",
                    field_col(vec![h::InputGroup::new()
                        .label("Amount")
                        .description("Billed in US dollars.")
                        .prefix(h::InputAddon::new("$"))
                        .input(
                            h::Input::new(self.demo_text("ig-example", "", cx)).placeholder("0.00"),
                        )
                        .suffix(h::InputAddon::new("USD"))
                        .into_any_element()]),
                ),
                (
                    "TextArea Usage Example",
                    field_col(vec![h::InputGroup::new()
                        .label("Changelog")
                        .description("Markdown is supported.")
                        .text_area(
                            h::TextArea::new(self.demo_text("ig-area-2", "", cx))
                                .placeholder("Share a quick project update...")
                                .rows(4),
                        )
                        .into_any_element()]),
                ),
                (
                    "Addons",
                    field_col(vec![h::InputGroup::new()
                        .label("Amount")
                        .description("Charged monthly.")
                        .prefix(h::InputAddon::new("$"))
                        .input(h::Input::new(self.group_amount.clone()).placeholder("0.00"))
                        .suffix(h::InputAddon::new("USD"))
                        .into_any_element()]),
                ),
                (
                    "With a trailing action",
                    field_col(vec![h::InputGroup::new()
                        .variant(FieldVariant::Secondary)
                        .input(h::Input::new(self.input_email.clone()).placeholder("Email"))
                        .suffix(
                            gpui::div()
                                .pl(px(8.))
                                .pr(px(4.))
                                .child(h::Button::new("ig-send").label("Send").size(Size::Sm)),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_input_otp(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let done = self.otp_done.clone();
        let otp_typed = self.otp_typed.clone();
        component_doc_page!(
            "Input OTP",
            crate::pages::Page::InputOtp.description(),
            crate::pages::Page::InputOtp.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::InputOTP::new(self.otp.clone())
                            .on_complete(cx.listener(|this, code: &str, _, cx| {
                                this.otp_done = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if done.is_empty() {
                                "Enter six digits".to_owned()
                            } else {
                                format!("Complete: {done}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-primary", 6, cx)).into_any_element(),
                        h::InputOTP::new(self.demo_otp("otp-secondary", 6, cx))
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::InputOTP::new(self.demo_otp("otp-surface", 6, cx))
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    col(vec![h::InputOTP::new(self.demo_otp("otp-disabled", 6, cx))
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Four Digits",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-four", 4, cx)).into_any_element()
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-controlled", 6, cx))
                            // `value` writes the caller's copy back into the
                            // field, which is what "controlled" means.
                            .value(&otp_typed, cx)
                            .on_change(cx.listener(|this, code: &str, _, cx| {
                                this.otp_typed = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if otp_typed.is_empty() {
                                "Nothing typed yet".to_owned()
                            } else {
                                format!("Value: {otp_typed}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "On Complete",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-complete", 6, cx))
                            .on_complete(cx.listener(|this, code: &str, _, cx| {
                                this.otp_done = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if done.is_empty() {
                                "`onComplete` fires once every slot is filled".to_owned()
                            } else {
                                format!("Completed with {done}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Slots",
                    "The GPUI `slot` extension receives each slot's live index and character.",
                    col(vec![h::InputOTP::new(self.demo_otp(
                        "otp-custom-slots",
                        4,
                        cx
                    ))
                    .slot(|index, value| {
                        gpui::div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .text_size(px(11.))
                            .child(value.unwrap_or('·').to_string())
                            .child(format!("#{index}"))
                            .into_any_element()
                    })
                    .into_any_element(),]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let state = self.demo_otp("otp-form", 6, cx);
                        h::Form::new()
                            .field(h::FormField::code("code", state.clone()).is_required(true))
                            .child(h::InputOTP::new(state).name("code"))
                            .child(h::Button::new("otp-form-submit").label("Verify"))
                            .into_any_element()
                    }]),
                ),
                (
                    "With Pattern",
                    col(vec![
                        spec(
                            "Digits (default)",
                            h::InputOTP::new(self.demo_otp("otp-pat-digits", 4, cx))
                                .pattern(h::OtpPattern::Digits),
                            cx,
                        ),
                        spec(
                            "Alphanumeric",
                            h::InputOTP::new(self.demo_otp("otp-pat-alnum", 4, cx))
                                .pattern(h::OtpPattern::Alphanumeric),
                            cx,
                        ),
                        spec(
                            "Any character",
                            h::InputOTP::new(self.demo_otp("otp-pat-any", 4, cx))
                                .pattern(h::OtpPattern::Any),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Validation",
                    col(vec![
                        h::InputOTP::new(self.demo_otp("otp-validate", 6, cx))
                            .validate(|code| {
                                (code.chars().count() < 6).then(|| "Enter all six digits".into())
                            })
                            .into_any_element(),
                        h::InputOTP::new(self.demo_otp("otp-invalid", 6, cx))
                            // `isInvalid` from the outside: what a rejected code
                            // looks like when the server says so.
                            .is_invalid(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_number_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let nf_controlled = self
            .demo_number("nf-ctl", 5., 0., 20., 1., cx)
            .read(cx)
            .value();
        component_doc_page!(
            "Number Field",
            crate::pages::Page::NumberField.description(),
            crate::pages::Page::NumberField.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::NumberField::new(self.demo_number(
                        "nf-usage",
                        1024.,
                        0.,
                        4096.,
                        1.,
                        cx,
                    ))
                    // v3's basic example seeds the field with `defaultValue`.
                    .default_value(1024.)
                    .min_value(0.)
                    .name("width")
                    .full_width(true)
                    .label("Width")
                    .into_any_element()]),
                ),
                (
                    "Without steppers",
                    col(vec![h::NumberField::new(self.number.clone())
                        .label("Quantity")
                        .hide_steppers(true)
                        .into_any_element()]),
                ),
                (
                    "Format options",
                    col(vec![h::NumberField::new(self.price.clone())
                        .label("Price")
                        .format_options(h::NumberFormat::currency("USD"))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::NumberField::new(self.demo_number("nf-primary", 5., 0., 20., 1., cx))
                            .label("Primary")
                            .into_any_element(),
                        h::NumberField::new(self.demo_number("nf-secondary", 5., 0., 20., 1., cx))
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::NumberField::new(self.demo_number(
                                "nf-surface",
                                2.,
                                0.,
                                10.,
                                1.,
                                cx,
                            ))
                            .label("Seats")
                            .variant(FieldVariant::Secondary)
                            .description("The secondary variant, for use on a surface"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-desc", 1., 0., 99., 1., cx),
                    )
                    .label("Quantity")
                    .description("How many licences to buy")
                    .into_any_element()]),
                ),
                (
                    "Required Field",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-req", 1., 0., 99., 1., cx),
                    )
                    .label("Quantity")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled State",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-dis", 8., 0., 99., 1., cx),
                    )
                    .label("Quantity")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-full", 3., 0., 99., 1., cx),
                    )
                    .label("Quantity")
                    .full_width(true)
                    .into_any_element()]),
                ),
                (
                    "Validation",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-invalid",
                        0.,
                        0.,
                        99.,
                        1.,
                        cx,
                    ))
                    .label("Quantity")
                    // `minValue`/`maxValue` on the component, which is what
                    // clamps the steppers and the typed value.
                    .min_value(1.)
                    .max_value(99.)
                    .is_required(true)
                    .is_invalid(true)
                    .validation_errors(["Order at least one"])
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::NumberField::new(self.demo_number("nf-ctl", 5., 0., 20., 1., cx))
                            .label("Quantity")
                            .on_change(f64_cb(cx.listener(|_, _v: &f64, _, cx| cx.notify())))
                            .into_any_element(),
                        para(&format!("Value: {nf_controlled}"), cx),
                    ]),
                ),
                (
                    "Step Values",
                    col(vec![
                        h::NumberField::new(self.demo_number("nf-step-5", 10., 0., 100., 5., cx))
                            .label("Step 5")
                            .into_any_element(),
                        h::NumberField::new(self.demo_number(
                            "nf-step-tenth",
                            1.5,
                            0.,
                            10.,
                            0.1,
                            cx,
                        ))
                        .label("Step 0.1")
                        .into_any_element(),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![{
                        let seats = self.demo_number("nf-form", 1., 1., 99., 1., cx);
                        h::Form::new()
                            .field(h::FormField::number(seats.clone()).name("seats"))
                            .child(
                                h::NumberField::new(seats)
                                    .label("Seats")
                                    .name("seats")
                                    .is_required(true),
                            )
                            .child(h::Button::new("nf-form-submit").label("Buy"))
                            .into_any_element()
                    }]),
                ),
                (
                    "With Validation",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-validate",
                        200.,
                        0.,
                        1000.,
                        10.,
                        cx,
                    ))
                    .label("Budget")
                    .description("At least 100")
                    .validate(|value| (*value < 100.).then(|| "Budget must be at least 100".into()))
                    .into_any_element()]),
                ),
                (
                    "Custom Icons",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-icons", 1024., 0., 4096., 1., cx,)
                    )
                    .label("Width (Custom Icons)")
                    .description("Custom icon children")
                    .decrement_icon(icon(h::icons::CHEVRON_LEFT, cx))
                    .increment_icon(icon(h::icons::CHEVRON_RIGHT, cx))
                    .into_any_element()]),
                ),
                (
                    "With Chevrons",
                    col(vec![h::NumberField::new(
                        self.demo_number("nf-chev", 99., 0., 999., 1., cx),
                    )
                    .label("Amount")
                    .format_options(h::NumberFormat::currency("EUR"))
                    .vertical_steppers(true)
                    .increment_icon(
                        gpui::svg()
                            .size(px(11.))
                            .path(h::icons::CHEVRON_UP)
                            .text_color(cx.colors().foreground),
                    )
                    .decrement_icon(
                        gpui::svg()
                            .size(px(11.))
                            .path(h::icons::CHEVRON_DOWN)
                            .text_color(cx.colors().foreground),
                    )
                    .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_radio_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.radio_sel;
        let indicator_color = cx.role(Color::Accent).foreground;
        let options: Vec<h::RadioOption> = vec!["Free".into(), "Pro".into(), "Enterprise".into()];
        let selected_value = SharedString::from(match selected {
            Some(0) => "Free",
            Some(1) => "Pro",
            Some(2) => "Enterprise",
            _ => "",
        });
        let plans =
            || -> Vec<h::RadioOption> { vec!["Free".into(), "Pro".into(), "Enterprise".into()] };
        component_doc_page!(
            "Radio Group",
            crate::pages::Page::RadioGroup.description(),
            crate::pages::Page::RadioGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::RadioGroup::new("rg-usage", plans())
                        .default_value("Free")
                        // v3's own example opens with the group's `<Label>` and
                        // `<Description>`, then a `<Description>` per `<Radio>`.
                        .label("Plan selection")
                        .description("Choose the plan that suits you best")
                        .descriptions([
                            Some("Includes 100 messages per month"),
                            Some("Includes 200 messages per month"),
                            None,
                        ])
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::RadioGroup::new("rg-v-primary", plans())
                            .default_value("Free")
                            .into_any_element(),
                        h::RadioGroup::new("rg-v-secondary", plans())
                            .default_value("Pro")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::RadioGroup::new("rg-surface", plans())
                                .default_value("Free")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    col(vec![
                        h::RadioGroup::new("rg-validate", plans())
                            .default_value("")
                            .label("Plan")
                            .is_required(true)
                            // v3 composes a `<FieldError>` in the group;
                            // supplying it is what marks the group invalid.
                            .error_message("Choose a plan to continue")
                            .into_any_element(),
                        h::RadioGroup::new(
                            "rg-option-error",
                            vec![
                                h::RadioOption::new("Standard delivery"),
                                h::RadioOption::new("Express delivery")
                                    .error_message("Unavailable for this address"),
                            ],
                        )
                        .label("Delivery speed")
                        .into_any_element(),
                    ]),
                ),
                (
                    "Delivery & Payment",
                    col(vec![
                        h::RadioGroup::new(
                            "rg-delivery",
                            vec![
                                "Standard — 5 to 7 days".into(),
                                "Express — 2 days".into(),
                                "Overnight".into(),
                            ],
                        )
                        .default_value("Standard — 5 to 7 days")
                        .into_any_element(),
                        h::Separator::new().into_any_element(),
                        h::RadioGroup::new(
                            "rg-payment",
                            vec!["Card".into(), "Bank transfer".into(), "Invoice".into()],
                        )
                        .default_value("Card")
                        .orientation(Orientation::Horizontal)
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Indicator", "The checkmark replaces `Radio.Indicator` while the control, selection and focus behavior stay owned by the radio.",
                    col(vec![
                        h::RadioGroup::new("rg-indicator", plans())
                            .default_value("Enterprise")
                            .indicator(move |_, state| {
                                if state.is_selected {
                                    gpui::svg()
                                        .size(px(12.))
                                        .path(h::icons::CHECK)
                                        .text_color(indicator_color)
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![h::RadioGroup::new("rg-v", options.clone())
                        .value(selected_value.clone())
                        .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.radio_sel = match value.as_ref() {
                                "Free" => Some(0),
                                "Pro" => Some(1),
                                "Enterprise" => Some(2),
                                _ => None,
                            };
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    col(vec![h::RadioGroup::new("rg-unc", options.clone())
                        .default_value("Pro")
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    col(vec![h::RadioGroup::new("rg-h", options.clone())
                        .value(selected_value.clone())
                        .orientation(Orientation::Horizontal)
                        .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.radio_sel = match value.as_ref() {
                                "Free" => Some(0),
                                "Pro" => Some(1),
                                "Enterprise" => Some(2),
                                _ => None,
                            };
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![
                        // v3's own example disables the whole group
                        // (`<RadioGroup isDisabled>`); `Radio.isDisabled`
                        // instead disables one option — dimmed, unclickable,
                        // skipped by the arrows.
                        h::RadioGroup::new("rg-d", options)
                            .value(selected_value)
                            .is_disabled(true)
                            .into_any_element(),
                        h::RadioGroup::new(
                            "rg-d-opt",
                            vec![
                                h::RadioOption::new("Free").is_disabled(true),
                                "Pro".into(),
                                "Enterprise".into(),
                            ],
                        )
                        .default_value("Enterprise")
                        .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_search_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let query = self.search_query.clone();
        // Each demo owns its state, the way v3's examples do.
        let surface = self.demo_text("sf-surface", "", cx);
        let described = self.demo_text("sf-desc", "", cx);
        let required = self.demo_text("sf-required", "", cx);
        let disabled = self.demo_text("sf-disabled", "Read-only query", cx);
        let full = self.demo_text("sf-full", "", cx);
        let invalid = self.demo_text("sf-invalid", "ab", cx);
        let controlled = self.demo_text("sf-controlled", "", cx);
        let validated = self.demo_text("sf-validated", "", cx);
        let form_field = self.demo_text("sf-form", "", cx);
        let icons = self.demo_text("sf-icons", "HeroUI", cx);
        let shortcut = self.demo_text("sf-shortcut", "", cx);
        let render_props = self.demo_text("sf-render-props", "hero", cx);
        let controlled_text = controlled.read(cx).value().to_owned();

        component_doc_page!(
            "Search Field",
            crate::pages::Page::SearchField.description(),
            crate::pages::Page::SearchField.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        demo_field(
                            h::SearchField::new(self.search_state.clone())
                                .label("Search docs")
                                .placeholder("Search components")
                                .on_change(cx.listener(|this, text: &str, _, cx| {
                                    this.search_query = text.to_owned();
                                    cx.notify();
                                })),
                        ),
                        para(
                            &if query.is_empty() {
                                "Type to search".to_owned()
                            } else {
                                format!("Query: {query}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Variants",
                    field_col(vec![
                        h::SearchField::new(self.demo_text("sf-v-primary", "", cx))
                            .label("Primary")
                            .placeholder("Search...")
                            .into_any_element(),
                        h::SearchField::new(self.demo_text("sf-v-secondary", "", cx))
                            .label("Secondary")
                            .placeholder("Search...")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::SearchField::new(surface)
                                .label("Search")
                                .placeholder("Search...")
                                .variant(FieldVariant::Secondary)
                                .description("Enter keywords to search"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::SearchField::new(described)
                        .label("Search")
                        .placeholder("Search products...")
                        .description("Searches titles and body text")
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::SearchField::new(required)
                        .label("Search")
                        .placeholder("Enter search query...")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::SearchField::new(disabled)
                        .label("Search")
                        .placeholder("Search...")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::SearchField::new(full)
                        .label("Search")
                        .placeholder("Search...")
                        .full_width()
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![h::SearchField::new(invalid)
                        .label("Search")
                        .placeholder("Search...")
                        .is_required(true)
                        .is_invalid(true)
                        .validation_errors(["Search query must be at least 3 characters"])
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        demo_field(
                            h::SearchField::new(controlled)
                                .label("Search")
                                .placeholder("Search...")
                                .on_change(|_, _, _| {})
                                // Enter submits: v3's `onSubmit`.
                                .on_submit(cx.listener(|this, text: &str, _, cx| {
                                    this.search_query = text.to_owned();
                                    cx.notify();
                                })),
                        ),
                        para(
                            &if controlled_text.is_empty() {
                                "Empty".to_owned()
                            } else {
                                format!("Value: {controlled_text}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Render Props",
                    field_col(vec![{
                        let parts = render_props.clone();
                        h::SearchField::new(render_props)
                            .content(move |state| {
                                h::SearchField::new(parts.clone())
                                    .label(format!(
                                        "{} · {} · {}",
                                        if state.is_empty { "empty" } else { "has value" },
                                        if state.is_focus_within {
                                            "focused"
                                        } else {
                                            "unfocused"
                                        },
                                        state.value
                                    ))
                                    .placeholder("Search")
                                    .into_any_element()
                            })
                            .into_any_element()
                    }]),
                ),
                (
                    "With Validation", "`validate` is run by the component: it returns the message, and the field shows it. Type one or two characters.",
                    col(vec![
                        demo_field(
                            h::SearchField::new(validated)
                                .label("Search")
                                .placeholder("Search...")
                                .is_required(true)
                                .description("Enter at least 3 characters to search")
                                .validate(|value| {
                                    (!value.is_empty() && value.chars().count() < 3).then(|| {
                                        "Search query must be at least 3 characters".into()
                                    })
                                }),
                        ),
                    ]),
                ),
                (
                    "Form Example",
                    field_col(vec![{
                        let field = h::SearchField::new(form_field)
                            .label("Search")
                            .placeholder("Search products...")
                            .name("query")
                            .is_required(true)
                            .validate(|value| {
                                (value.chars().count() < 3)
                                    .then(|| "Enter at least 3 characters".into())
                            });
                        h::Form::new()
                            .field(h::FormField::text(self.demo_text("sf-form", "", cx)))
                            .child(field)
                            .child(h::Button::new("sf-form-submit").label("Search"))
                            .into_any_element()
                    }]),
                ),
                (
                    "Custom Icons",
                    field_col(vec![h::SearchField::new(icons)
                        .label("Search")
                        .placeholder("Search...")
                        .search_icon(icon(h::icons::GLOBE, cx))
                        .clear_icon(icon(h::icons::CHECK, cx))
                        .into_any_element()]),
                ),
                (
                    "With Keyboard Shortcut",
                    field_col(vec![h::SearchField::new(shortcut)
                        .label("Search")
                        .placeholder("Search...")
                        .end_content(h::Kbd::new().child("Shift S"))
                        .description("Press Shift+S to focus")
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_text_area(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let ta_controlled = self
            .demo_text("ta-controlled", "", cx)
            .read(cx)
            .value()
            .to_owned();
        component_doc_page!(
            "Text Area",
            crate::pages::Page::TextArea.description(),
            crate::pages::Page::TextArea.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![fixed_demo(
                        384.,
                        h::TextArea::new(self.demo_text("ta-usage", "", cx))
                            .placeholder("Share a quick project update...")
                            .cols(48)
                            .rows(6)
                            .full_width(),
                    )]),
                ),
                (
                    "Variants",
                    field_col(vec![
                        h::TextArea::new(self.demo_text("ta-primary", "", cx))
                            .label("Primary")
                            .placeholder("Primary textarea")
                            .rows(3)
                            .into_any_element(),
                        h::TextArea::new(self.demo_text("ta-secondary", "", cx))
                            .label("Secondary")
                            .placeholder("Secondary textarea")
                            .variant(FieldVariant::Secondary)
                            .rows(3)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::TextArea::new(self.demo_text("ta-surface", "", cx))
                                .label("Notes")
                                .placeholder("Describe your product")
                                .variant(FieldVariant::Secondary)
                                .rows(3),
                        )
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::TextArea::new(self.demo_text("ta-full", "", cx))
                        .label("Notes")
                        .placeholder("Full width textarea")
                        .rows(3)
                        .full_width()
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        demo_field(
                            h::TextArea::new(self.demo_text("ta-controlled", "", cx))
                                .label("Notes")
                                .placeholder("Compose an announcement...")
                                .rows(3)
                                .on_change(|_, _, _| {}),
                        ),
                        para(&format!("{} characters", ta_controlled.chars().count()), cx),
                    ]),
                ),
                (
                    "Rows",
                    field_col(vec![h::TextArea::new(self.input_bio.clone())
                        .label("Six rows")
                        .placeholder("Write out the full meeting notes...")
                        .rows(6)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_text_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let tf_controlled = self
            .demo_text("tf-controlled", "", cx)
            .read(cx)
            .value()
            .to_owned();
        let tf_render_props = self.demo_text("tf-render-props", "Ada", cx);
        component_doc_page!(
            "Text Field",
            crate::pages::Page::TextField.description(),
            crate::pages::Page::TextField.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::TextField::new(self.text_field_state.clone())
                        .label("Full name")
                        .placeholder("Ada Lovelace")
                        .description("As it appears on your ID.")
                        .into_any_element()]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::TextField::new(self.demo_text("tf-surface", "", cx))
                                .label("Full name")
                                .placeholder("John")
                                .variant(FieldVariant::Secondary)
                                .description("Use the secondary variant on a surface"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::TextField::new(self.demo_text("tf-desc", "", cx))
                        .label("Full name")
                        .placeholder("Enter username")
                        .description("As it appears on your ID.")
                        .into_any_element()]),
                ),
                (
                    "Required Field",
                    field_col(vec![h::TextField::new(self.demo_text("tf-req", "", cx))
                        .label("Full name")
                        .placeholder("John Doe")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    field_col(vec![h::TextField::new(self.demo_text(
                        "tf-dis",
                        "Ada Lovelace",
                        cx,
                    ))
                    .label("Full name")
                    .placeholder("Auto-generated")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::TextField::new(self.demo_text("tf-full", "", cx))
                        .label("Full name")
                        .placeholder("John")
                        .full_width()
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    field_col(vec![
                        h::TextField::new(self.demo_text("tf-validate", "", cx,))
                            .label("Full name")
                            .placeholder("jane_doe")
                            .is_required(true)
                            .validate(|value| value
                                .trim()
                                .is_empty()
                                .then(|| "Name is required".into()))
                            .into_any_element(),
                        h::TextField::new(self.demo_text("tf-invalid", "", cx))
                        .label("Full name")
                        .placeholder("Ada Lovelace")
                        // `isInvalid` marks it invalid from the outside, which is
                        // what a server-side error looks like.
                        .is_invalid(true)
                        .error_message("Name is required")
                        .into_any_element()
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        demo_field(
                            h::TextField::new(self.demo_text("tf-controlled", "", cx))
                                .label("Full name")
                                .placeholder("Jane")
                                .on_change(|_, _, _| {}),
                        ),
                        para(&format!("Value: {tf_controlled}"), cx),
                    ]),
                ),
                (
                    "Render Props",
                    field_col(vec![{
                        let field = tf_render_props.clone();
                        h::TextField::new(tf_render_props)
                            .content(move |state| {
                                h::TextField::new(field.clone())
                                    .label(format!(
                                        "{} · {} · {}",
                                        if state.is_required {
                                            "required"
                                        } else {
                                            "optional"
                                        },
                                        if state.is_invalid { "invalid" } else { "valid" },
                                        if state.is_focus_within {
                                            "focused"
                                        } else {
                                            "unfocused"
                                        },
                                    ))
                                    .placeholder("Enter your email")
                                    .is_required(true)
                                    .into_any_element()
                            })
                            .is_required(true)
                            .into_any_element()
                    }]),
                ),
                (
                    "Error Message",
                    field_col(vec![h::TextField::new(self.text_field_state.clone())
                        .label("Full name")
                        .placeholder("Ada Lovelace")
                        .is_required(true)
                        .error_message("This field is required.")
                        .into_any_element()]),
                ),
                (
                    "TextArea",
                    field_col(vec![h::TextArea::new(self.demo_text("tf-area", "", cx))
                        .label("Bio")
                        .placeholder("Write your message here...")
                        .rows(4)
                        .description("A `TextField` whose input is multi-line")
                        .into_any_element()]),
                ),
                (
                    "Input Types",
                    field_col(vec![
                        h::TextField::new(self.demo_text("tf-pw", "", cx))
                            .label("Password")
                            .placeholder("••••••••")
                            .input_type(h::InputType::Password)
                            .into_any_element(),
                        h::TextField::new(self.demo_text("tf-email", "", cx))
                            .label("Email")
                            .placeholder("user@example.com")
                            .input_type(h::InputType::Email)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Layout
    // -----------------------------------------------------------------------

    pub fn page_card(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let card = |variant: h::CardVariant| {
            h::Card::new()
                .variant(variant)
                .w(px(260.))
                .child(
                    h::CardHeader::new()
                        .child(h::CardTitle::new().child("Daily report"))
                        .child(h::CardDescription::new().child("Traffic summary for the week")),
                )
                .child(h::CardContent::new().child("Sessions are up 12% week over week."))
                .child(
                    h::CardFooter::new().child(
                        h::Button::new(el_id(format!("card-{variant:?}-cta")))
                            .label("View")
                            .size(Size::Sm)
                            .variant(Variant::Tertiary),
                    ),
                )
        };
        component_doc_page!(
            "Card",
            crate::pages::Page::Card.description(),
            crate::pages::Page::Card.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Card::new()
                        .w(px(260.))
                        .child(
                            h::CardHeader::new()
                                .child(h::CardTitle::new().child("Daily report"))
                                .child(
                                    h::CardDescription::new().child("Traffic summary for the week"),
                                ),
                        )
                        .child(h::CardContent::new().child("Sessions are up 12% week over week."))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(h::CardVariant::ALL.iter().map(|v| card(*v)).els()),
                ),
                (
                    "Horizontal Layout",
                    row(vec![h::Card::new()
                        .w(px(420.))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(16.))
                                    .child(
                                        gpui::div()
                                            .size(px(72.))
                                            .flex_shrink_0()
                                            .rounded(px(12.))
                                            .bg(cx.colors().default.color),
                                    )
                                    .child(
                                        gpui::div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.))
                                            .child(gpui::div().child("Weekly digest"))
                                            .child(
                                                gpui::div()
                                                    .text_size(px(12.5))
                                                    .text_color(cx.colors().muted)
                                                    .child("Every Monday, 9am"),
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Avatar",
                    row(vec![h::Card::new()
                        .w(px(200.))
                        .child(
                            gpui::div()
                                .size(px(56.))
                                .rounded(h::util::soft_radius(cx))
                                .bg(cx.colors().default.color),
                        )
                        .child(
                            h::CardHeader::new()
                                .child(h::CardTitle::new().child("Indie Hackers"))
                                .child(h::CardDescription::new().child("148 members")),
                        )
                        .child(
                            h::CardFooter::new()
                                .child(h::Avatar::new("card-martha").name("Martha").size(Size::Sm))
                                .child("By Martha"),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Images",
                    row(vec![h::Card::new()
                        .w(px(280.))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .h(px(140.))
                                    .w_full()
                                    .rounded(px(12.))
                                    .bg(cx.colors().default.color),
                            ),
                        )
                        .child(h::CardFooter::new().child("A placeholder for cover art."))
                        .into_any_element()]),
                ),
                (
                    "With Form",
                    row(vec![h::Card::new()
                        .w(px(320.))
                        .child(h::CardHeader::new().child(h::CardTitle::new().child("Sign in")))
                        .child(
                            h::CardContent::new().child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("card-email", "", cx))
                                            .label("Email")
                                            .input_type(h::InputType::Email)
                                            .full_width(),
                                    )
                                    .child(
                                        h::TextField::new(self.demo_text("card-password", "", cx))
                                            .label("Password")
                                            .input_type(h::InputType::Password)
                                            .full_width(),
                                    ),
                            ),
                        )
                        .child(
                            h::CardFooter::new()
                                .child(h::Button::new("card-signin").label("Sign in")),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_separator(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Separator",
            crate::pages::Page::Separator.description(),
            crate::pages::Page::Separator.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![gpui::div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .child(gpui::div().child("Above"))
                        .child(h::Separator::new())
                        .child(gpui::div().child("Below"))
                        .into_any_element()]),
                ),
                (
                    "With Surface",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Notifications"))
                        .child(h::Separator::new())
                        .child(gpui::div().child("Privacy"))
                        .into_any_element()]),
                ),
                (
                    "With Content",
                    col(vec![h::Separator::new()
                        .child(gpui::div().text_size(px(12.)).child("OR"))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(h::SeparatorVariant::ALL
                        .iter()
                        .flat_map(|v| {
                            vec![
                                gpui::div()
                                    .text_size(px(12.))
                                    .text_color(cx.colors().muted)
                                    .child(v.label())
                                    .into_any_element(),
                                h::Separator::new().variant(*v).into_any_element(),
                            ]
                        })
                        .collect()),
                ),
                (
                    "Vertical",
                    row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .h(px(24.))
                        .child("Docs")
                        .child(h::Separator::new().orientation(Orientation::Vertical))
                        .child("Blog")
                        .child(h::Separator::new().orientation(Orientation::Vertical))
                        .child("Support")
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_surface(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let panel = |variant: h::SurfaceVariant| {
            h::Surface::new()
                .variant(variant)
                // The upstream examples dress their skeletons through
                // className (`p-6`, `gap-3`, plus `rounded-3xl`/borders).
                // className customization stays unavailable, so the demo's
                // paddings/gaps only exercise the layout knobs this port's
                // Surface builders expose.
                .padding(px(24.))
                .gap(px(12.))
                .child(h::Typography::heading(6, "Surface content").into_any_element())
                .child(
                    h::Typography::paragraph(
                        h::ParagraphSize::Sm,
                        "Nested content inherits the surface foreground.",
                    )
                    .color(h::TextColor::Muted)
                    .into_any_element(),
                )
        };
        component_doc_page!(
            "Surface",
            crate::pages::Page::Surface.description(),
            crate::pages::Page::Surface.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(gpui::div().child("A surface groups related content."))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        panel(h::SurfaceVariant::Default).into_any_element(),
                        panel(h::SurfaceVariant::Secondary).into_any_element(),
                        panel(h::SurfaceVariant::Tertiary).into_any_element(),
                        panel(h::SurfaceVariant::Transparent).into_any_element(),
                    ]),
                ),
                (
                    "With form components",
                    field_col(vec![h::Surface::new()
                        // Same as the variants panel: upstream adds its
                        // `p-6` + `gap-4` through className; these explicit
                        // paddings/gaps exercise the port's own layout
                        // knobs, and no radius/border is re-added here.
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::Input::new(self.input_name.clone())
                                .placeholder("Secondary input")
                                .variant(FieldVariant::Secondary),
                        )
                        .child(
                            h::TextArea::new(self.input_bio.clone())
                                .placeholder("Secondary text area")
                                .rows(3),
                        )
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_toolbar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let bar = |key: &str, attached: bool, orientation: Orientation| {
            h::Toolbar::new()
                // Names the instance: the toolbar's keyed focus state is per
                // id, and three toolbars share this page.
                .id(el_id(format!("toolbar-{key}")))
                .is_attached(attached)
                .orientation(orientation)
                .child(
                    h::ToggleButtonGroup::new(el_id(format!("toolbar-toggle-{key}")))
                        .selection_mode(SelectionMode::Multiple)
                        .separators(true)
                        .child_toggle(
                            h::ToggleButton::new(el_id(format!("tbar-{key}-b"))).label("B"),
                        )
                        .child_toggle(
                            h::ToggleButton::new(el_id(format!("tbar-{key}-i"))).label("I"),
                        ),
                )
                .child(h::Separator::new().orientation(match orientation {
                    Orientation::Horizontal => Orientation::Vertical,
                    Orientation::Vertical => Orientation::Horizontal,
                }))
                .child(
                    h::ButtonGroup::new()
                        .variant(Variant::Tertiary)
                        .size(Size::Sm)
                        .separators(true)
                        .button(h::Button::new(el_id(format!("tbar-{key}-copy"))).label("Copy"))
                        .button(h::Button::new(el_id(format!("tbar-{key}-cut"))).label("Cut")),
                )
        };
        component_doc_page!(
            "Toolbar",
            crate::pages::Page::Toolbar.description(),
            crate::pages::Page::Toolbar.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Toolbar::new()
                        .id("tb-usage")
                        .child(
                            h::Button::new("tb-usage-1")
                                .label("Cut")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .child(
                            h::Button::new("tb-usage-2")
                                .label("Copy")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .child(
                            h::Button::new("tb-usage-3")
                                .label("Paste")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .into_any_element()]),
                ),
                (
                    "With ButtonGroup",
                    col(vec![h::Toolbar::new()
                        .id("tb-button-group")
                        .child(
                            h::ButtonGroup::new()
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .separators(true)
                                .button(h::Button::new("tb-bg-1").label("Left"))
                                .button(h::Button::new("tb-bg-2").label("Center"))
                                .button(h::Button::new("tb-bg-3").label("Right")),
                        )
                        .child(h::Separator::new().orientation(Orientation::Vertical))
                        .child(
                            h::Button::new("tb-bg-4")
                                .label("Reset")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm),
                        )
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    col(vec![
                        bar("h", false, Orientation::Horizontal).into_any_element()
                    ]),
                ),
                (
                    "Attached",
                    col(vec![
                        bar("attached", true, Orientation::Horizontal).into_any_element()
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![
                        bar("v", false, Orientation::Vertical).into_any_element()
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Media
    // -----------------------------------------------------------------------

    pub fn page_avatar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        // v3 has no AvatarGroup: its Avatar Group example composes ordinary
        // avatars with layout CSS — a `-space-x-2` overlap and a
        // `ring-2 ring-background` ring on every member — and renders the
        // overflow counter as a plain fallback avatar with `text-xs`.
        fn member(el: impl IntoElement, ring: gpui::Hsla) -> gpui::Div {
            gpui::div()
                .border_2()
                .border_color(ring)
                .rounded_full()
                .child(el)
        }
        let ring = cx.colors().background;
        let overlap = |d: gpui::Div| d.ml(px(-8.)).into_any_element();
        let names = [
            "Ada Lovelace",
            "Grace Hopper",
            "Alan Turing",
            "Katherine Johnson",
            "Margaret Hamilton",
        ];
        // `-space-x-2`: only subsequent siblings get the -8px margin.
        let mut counter_members: Vec<AnyElement> = names
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, n)| {
                let d = member(h::Avatar::new(("counter-member", i)).name(*n), ring);
                if i == 0 {
                    d.into_any_element()
                } else {
                    overlap(d)
                }
            })
            .collect();
        counter_members.push(overlap(member(
            gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(40.))
                .rounded_full()
                .bg(cx.colors().surface_tertiary)
                .text_color(cx.colors().foreground)
                .text_size(px(12.))
                .child(format!("+{}", names.len() - 3)),
            ring,
        )));
        component_doc_page!(
            "Avatar",
            crate::pages::Page::Avatar.description(),
            crate::pages::Page::Avatar.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Avatar::new("usage-avatar")
                        .name("Jane Doe")
                        .into_any_element()]),
                ),
                (
                    "Fallback Content",
                    spec_row(vec![
                        spec(
                            "Initials",
                            h::Avatar::new("initials-avatar").name("Jane Doe"),
                            cx,
                        ),
                        spec("No name", h::Avatar::new("unnamed-avatar"), cx),
                        // v3's own Fallback Content example drives a
                        // deliberately broken URL with
                        // `<Avatar.Fallback delayMs={600}>`; an unregistered
                        // asset path fails identically here (no network), and
                        // the initials replace the box once the delay elapses.
                        spec(
                            "Broken image",
                            h::Avatar::new("delay-avatar")
                                .name("NA")
                                .src("images/avatar-broken.png")
                                .delay_ms(600),
                            cx,
                        ),
                        spec(
                            "Custom fallback",
                            h::Avatar::new("icon-avatar")
                                .name("HG")
                                .fallback(icon(h::icons::HEART_FILL, cx)),
                            cx,
                        ),
                        spec(
                            "Fallback color",
                            h::Avatar::new("fb-color-avatar")
                                .name("HG")
                                .color(Color::Accent)
                                .variant(h::AvatarVariant::Soft)
                                .fallback_color(Color::Warning),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Sizes",
                    spec_row(
                        Size::ALL
                            .iter()
                            .enumerate()
                            .map(|(i, s)| {
                                spec(
                                    s.label(),
                                    h::Avatar::new(("size-avatar", i))
                                        .name("Ada Lovelace")
                                        .size(*s),
                                    cx,
                                )
                            })
                            .collect()
                    ),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(i, c)| h::Avatar::new(("color-avatar", i)).name("HG").color(*c))
                        .els()),
                ),
                (
                    "Variants",
                    spec_row(
                        h::AvatarVariant::ALL
                            .iter()
                            .enumerate()
                            .map(|(i, v)| {
                                spec(
                                    v.label(),
                                    h::Avatar::new(("variant-avatar", i))
                                        .name("HG")
                                        .color(Color::Accent)
                                        .variant(*v),
                                    cx,
                                )
                            })
                            .collect()
                    ),
                ),
                (
                    "Group",
                    row(vec![gpui::div()
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap(px(24.))
                        .child(
                            // Basic group: the first four users overlap by 8px.
                            gpui::div()
                                .flex()
                                .children(names.iter().take(4).enumerate().map(|(i, n)| {
                                    let d =
                                        member(h::Avatar::new(("group-member", i)).name(*n), ring);
                                    if i == 0 {
                                        d.into_any_element()
                                    } else {
                                        overlap(d)
                                    }
                                }),),
                        )
                        .child(
                            // Counter group: three members plus the "+N"
                            // fallback avatar, as v3's second row does.
                            gpui::div().flex().children(counter_members),
                        )
                        .into_any_element()]),
                ),
                (
                    "Custom Image Component", "v3 composes a custom image element with `asChild` on `Avatar.Image`. The port's equivalent is a custom gpui `ImageSource`: the loader below supplies the embedded sample image itself, and `on_load` fires once the image is ready and replaces the fallback.",
                    col(vec![
                        spec(
                            "Custom loader",
                            h::Avatar::new("custom-loader-avatar")
                                .name("JD")
                                .src(sample_avatar_source())
                                .fallback("JD")
                                .on_load(|_, cx| {
                                    h::Toast::new("Avatar image loaded")
                                        .description("on_load fired once for the custom source.")
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                }),
                            cx,
                        ),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Navigation
    // -----------------------------------------------------------------------

    pub fn page_accordion(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let open = self.accordion_open.clone();
        let items = || {
            vec![
                h::AccordionItem::new("1", "What is HeroGPUI?")
                    .content(gpui::div().child("A faithful Rust port of HeroUI v3 for GPUI.")),
                h::AccordionItem::new("2", "Does it support dark mode?")
                    .content(gpui::div().child("Yes — every token has a light and dark value.")),
                h::AccordionItem::new("3", "Is it production ready?")
                    .subtitle("Short answer")
                    .content(
                        gpui::div().child("The component set is complete; the API is settling."),
                    ),
            ]
        };
        component_doc_page!(
            "Accordion",
            crate::pages::Page::Accordion.description(),
            crate::pages::Page::Accordion.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Accordion::new(items())
                        .id("acc-usage")
                        .default_expanded("1")
                        .into_any_element()]),
                ),
                (
                    "Without Separator",
                    col(vec![h::Accordion::new(items())
                        .id("acc-nosep")
                        .hide_separator(true)
                        .default_expanded("1")
                        .into_any_element()]),
                ),
                (
                    "Multiple Expanded",
                    col(vec![h::Accordion::new(items())
                        .id("acc-multi")
                        .allows_multiple_expanded(true)
                        .default_expanded_keys(
                            [SharedString::from("1"), SharedString::from("2")]
                                .into_iter()
                                .collect(),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled State",
                    col(vec![
                        spec_block(
                            "The whole group",
                            h::Accordion::new(items()).id("acc-dis").is_disabled(true),
                            cx,
                        ),
                        spec_block(
                            "One item",
                            h::Accordion::new(items())
                                .id("acc-dis-one")
                                .disabled_keys([SharedString::from("2")]),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Accordion::new(items())
                            .id("acc-controlled")
                            .expanded_keys(open.clone())
                            .on_expanded_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.accordion_open = keys.clone();
                                    cx.notify();
                                },
                            ))
                            .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                                toggle_key(&mut this.accordion_open, key);
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("{} expanded", open.len()), cx),
                    ]),
                ),
                (
                    "Custom Indicator",
                    col(vec![h::Accordion::new(vec![
                        h::AccordionItem::new("shipping", "Shipping details")
                            .content(gpui::div().child("Free shipping on orders over $50."))
                            .indicator(|state, _, cx| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_expanded {
                                        h::icons::MINUS
                                    } else {
                                        h::icons::PLUS
                                    })
                                    .text_color(cx.colors().muted)
                                    .into_any_element()
                            }),
                        h::AccordionItem::new("returns", "Returns policy")
                            .content(gpui::div().child("Returns are accepted within thirty days."))
                            .indicator(|state, _, cx| {
                                gpui::svg()
                                    .size(px(16.))
                                    .path(if state.is_expanded {
                                        h::icons::MINUS
                                    } else {
                                        h::icons::PLUS
                                    })
                                    .text_color(cx.colors().muted)
                                    .into_any_element()
                            }),
                    ])
                    .id("acc-indicator")
                    .default_expanded("shipping")
                    .into_any_element()]),
                ),
                (
                    "FAQ Layout",
                    col(vec![h::Surface::new()
                        .padding(px(20.))
                        .gap(px(12.))
                        .child(gpui::div().child("Frequently asked"))
                        .child(
                            h::Accordion::new(vec![
                                h::AccordionItem::new("ship", "When does it ship?").content(
                                    gpui::div().child("Orders leave the warehouse next day."),
                                ),
                                h::AccordionItem::new("returns", "Can I return it?").content(
                                    gpui::div().child("Within thirty days, in any condition."),
                                ),
                                h::AccordionItem::new("warranty", "Is there a warranty?")
                                    .content(gpui::div().child("Two years, parts and labour.")),
                            ])
                            .id("acc-faq")
                            .variant(h::AccordionVariant::Surface),
                        )
                        .into_any_element()]),
                ),
                (
                    "Default",
                    col(vec![h::Accordion::new(items())
                        .expanded_keys(open.clone())
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Surface",
                    col(vec![h::Accordion::new(items())
                        .variant(h::AccordionVariant::Surface)
                        .expanded_keys(open.clone())
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Hidden separator",
                    col(vec![h::Accordion::new(items())
                        .hide_separator(true)
                        .expanded_keys(open)
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.accordion_open, key);
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_breadcrumbs(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let last_nav = self.demo_text_value("bc-nav");
        let crumbs = || {
            vec![
                h::Crumb::new("Home").href("#"),
                h::Crumb::new("Components").href("#"),
                h::Crumb::new("Breadcrumbs"),
            ]
        };
        component_doc_page!(
            "Breadcrumbs",
            crate::pages::Page::Breadcrumbs.description(),
            crate::pages::Page::Breadcrumbs.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        // `cx.listener` takes one event argument, and
                        // `on_navigate` receives the index and the crumb
                        // alongside the click, so the view state is reached
                        // through the entity directly.
                        h::Breadcrumbs::new(crumbs())
                            .id("bc-usage")
                            .on_navigate({
                                let view = cx.entity().downgrade();
                                move |idx: usize, crumb: &h::Crumb, _, _, cx: &mut gpui::App| {
                                    let _ = view.update(cx, |this, cx| {
                                        this.set_demo_text_value(
                                            "bc-nav",
                                            format!("{idx}: {}", crumb.label),
                                        );
                                        cx.notify();
                                    });
                                }
                            })
                            .into_any_element(),
                        para(
                            &format!(
                                "The current page is \"Breadcrumbs\" -- inert, no tab stop. \
                                 Last navigation: {}",
                                if last_nav.is_empty() {
                                    "none yet".to_owned()
                                } else {
                                    last_nav
                                }
                            ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Navigation Levels",
                    col(vec![
                        h::Breadcrumbs::new(vec![h::Crumb::new("Home").href("#")])
                            .id("bc-level-1")
                            .into_any_element(),
                        h::Breadcrumbs::new(vec![
                            h::Crumb::new("Home").href("#"),
                            h::Crumb::new("Library"),
                        ])
                        .id("bc-level-2")
                        .into_any_element(),
                        h::Breadcrumbs::new(vec![
                            h::Crumb::new("Home").href("#"),
                            h::Crumb::new("Library").href("#"),
                            h::Crumb::new("Data"),
                        ])
                        .id("bc-level-3")
                        .into_any_element(),
                    ]),
                ),
                (
                    "Disabled State",
                    col(vec![h::Breadcrumbs::new(vec![
                        h::Crumb::new("Home").href("#"),
                        h::Crumb::new("Archive").href("#"),
                        h::Crumb::new("2025"),
                    ])
                    .id("bc-disabled")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Custom Separator",
                    col(vec![h::Breadcrumbs::new(crumbs())
                        .id("bc-sep-custom")
                        .separator_render(|_| {
                            gpui::div()
                                .text_size(px(12.))
                                .child("→".to_owned())
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Separators",
                    col(vec![
                        h::Breadcrumbs::new(crumbs())
                            .id("bc-sep-slash")
                            .separator(h::BreadcrumbSeparator::Slash)
                            .into_any_element(),
                        h::Breadcrumbs::new(crumbs())
                            .id("bc-sep-chevron")
                            .separator(h::BreadcrumbSeparator::Chevron)
                            .into_any_element(),
                        h::Breadcrumbs::new(crumbs())
                            .id("bc-sep-dash")
                            .separator(h::BreadcrumbSeparator::Dash)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_disclosure(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let expanded = self.disclosure_expanded;
        let group = self.disclosure_group_expanded.clone();
        component_doc_page!(
            "Disclosure",
            crate::pages::Page::Disclosure.description(),
            crate::pages::Page::Disclosure.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Disclosure::new(
                        "disclosure-usage",
                        "Shipping details"
                    )
                    .child(gpui::div().child("Ships in 2-4 business days."))
                    .into_any_element()]),
                ),
                (
                    "Render Function",
                    "The body is built from the Disclosure's current expanded and disabled state.",
                    {
                        let render_expanded = self.demo_flag("disclosure-render", true);
                        col(vec![h::Disclosure::new("disclosure-render", "Account details")
                            .is_expanded(render_expanded)
                            .on_expanded_change(bool_cb(cx.listener(
                                |this, value: &bool, _, cx| {
                                    this.set_demo_flag("disclosure-render", *value);
                                    cx.notify();
                                },
                            )))
                            .content(|state| {
                                gpui::div()
                                    .child(format!(
                                        "The render closure received is_expanded={} and is_disabled={}.",
                                        state.is_expanded, state.is_disabled
                                    ))
                                    .into_any_element()
                            })
                            .into_any_element()])
                    }
                ),
                (
                    "Controlled",
                    col(vec![
                        h::DisclosureGroup::new("disclosure-controlled")
                            .item("returns", "Returns", gpui::div().child("Thirty days."))
                            .item("warranty", "Warranty", gpui::div().child("Two years."))
                            .expanded_keys(group.clone())
                            .on_expanded_change(cx.listener(
                                |this, keys: &HashSet<SharedString>, _, cx| {
                                    this.disclosure_group_expanded = keys.clone();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} expanded", group.len()), cx),
                    ]),
                ),
                (
                    "Single",
                    col(vec![h::Disclosure::new(
                        "disclosure-single",
                        "Shipping details"
                    )
                    .is_expanded(expanded)
                    .on_expanded_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                        this.disclosure_expanded = *v;
                        cx.notify();
                    })))
                    .child(gpui::div().child("Ships in 2-4 business days."))
                    .into_any_element()]),
                ),
                (
                    "Group",
                    col(vec![h::DisclosureGroup::new("disclosure-group")
                        .item(
                            "item-1",
                            "Returns",
                            gpui::div().child("Free returns within 30 days."),
                        )
                        .item(
                            "item-2",
                            "Warranty",
                            gpui::div().child("Two years of coverage."),
                        )
                        .default_expanded_keys(["item-1"])
                        .into_any_element()]),
                ),
                (
                    "Disabled Group",
                    col(vec![h::DisclosureGroup::new("disclosure-disabled-group")
                        .item("returns", "Returns", gpui::div().child("Thirty days."))
                        .item("warranty", "Warranty", gpui::div().child("Two years."))
                        .default_expanded_keys(["returns"])
                        .is_disabled(true)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_link(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Link",
            crate::pages::Page::Link.description(),
            crate::pages::Page::Link.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Link::new("ln-hover")
                        .label("Hover to see the underline")
                        .href("#")
                        .into_any_element()]),
                ),
                (
                    "Icon Placement",
                    col(vec![
                        h::Link::new("ln-icon-end")
                            .label("Icon at end (default)")
                            .icon(icon(h::icons::EXTERNAL_LINK, cx))
                            .href("#")
                            .into_any_element(),
                        h::Link::new("ln-icon-start")
                            .label("Icon at start")
                            .icon(icon(h::icons::EXTERNAL_LINK, cx))
                            .icon_first(true)
                            .href("#")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Text Decoration", "The pinned `.link` carries `no-underline decoration-[1.5px]`; hover recolours the decoration to `decoration-muted/50` and press to `decoration-muted`. The text colour itself never changes state; a different decoration is the caller's own styling on the element they own.",
                    col(vec![
                        h::Link::new("ln-decor")
                            .label("Underlined on hover")
                            .href("#")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Icon",
                    col(vec![h::Link::new("ln-custom-icon")
                        .label("Open the docs")
                        .icon(icon(h::icons::ARROW_RIGHT, cx))
                        .href("#")
                        .into_any_element()]),
                ),
                (
                    "Render Function", "v3's `render` hands the link's interactive state to a caller-built element. The root keeps the `href`, `onPress`, focus and disabled wiring; GPUI has no DOM props to spread, so the closure draws the content from the state alone.",
                    col(vec![
                        h::Link::new("ln-render")
                            .href("#")
                            .render(|state| {
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .child("Call to action")
                                    .child(gpui::div().text_size(px(12.)).opacity(0.6).child(
                                        if state.is_hovered {
                                            "hovered"
                                        } else if state.is_pressed {
                                            "pressed"
                                        } else if state.is_focus_visible {
                                            "focus-visible"
                                        } else if state.is_focused {
                                            "focused"
                                        } else if state.is_disabled {
                                            "disabled"
                                        } else {
                                            "custom render"
                                        },
                                    ))
                                    .into_any_element()
                            })
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_pagination(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let page = self.pagination_page;
        component_doc_page!(
            "Pagination",
            crate::pages::Page::Pagination.description(),
            crate::pages::Page::Pagination.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Pagination::new("pg-main", page, 10)
                        .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .map(|sz| {
                            h::Pagination::new(el_id(format!("pg-{sz:?}")), page, 8).size(*sz)
                        })
                        .els()),
                ),
                (
                    "Disabled",
                    col(vec![h::Pagination::new("pg-disabled", page, 8)
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled Links",
                    col(vec![h::Pagination::new("pg-disabled-links", page, 8)
                        .disabled_keys([0, 5, 9])
                        .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Simple (Previous / Next)",
                    col(vec![row(vec![
                        h::Button::new("pg-prev")
                            .label("Previous")
                            .variant(Variant::Tertiary)
                            .is_disabled(page <= 1)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.pagination_page =
                                    this.pagination_page.saturating_sub(1).max(1);
                                cx.notify();
                            }))
                            .into_any_element(),
                        gpui::div()
                            .text_size(px(13.5))
                            .child(format!("Page {page} of 8"))
                            .into_any_element(),
                        h::Button::new("pg-next")
                            .label("Next")
                            .variant(Variant::Tertiary)
                            .is_disabled(page >= 8)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.pagination_page = (this.pagination_page + 1).min(8);
                                cx.notify();
                            }))
                            .into_any_element(),
                    ])]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Pagination::new("pg-controlled", page, 8)
                            .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            })))
                            .into_any_element(),
                        para(&format!("Page {page}"), cx),
                    ]),
                ),
                (
                    "With Ellipsis",
                    col(vec![h::Pagination::new("pg-ellipsis", page, 24)
                        .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "With Summary",
                    col(vec![h::Pagination::new("pg-summary", page, 12)
                        .summary(format!(
                            "Showing {}-{} of 120 items",
                            (page.saturating_sub(1)) * 10 + 1,
                            (page * 10).min(120)
                        ))
                        .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Render Props", "`link` receives each page number and `isActive`, so custom page content does not have to re-derive the current page.",
                    col(vec![
                        h::Pagination::new("pg-render-props", page, 5)
                            .link(|page, is_active| {
                                gpui::div()
                                    .child(if is_active {
                                        format!("[{page}]")
                                    } else {
                                        page.to_string()
                                    })
                                    .into_any_element()
                            })
                            .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Icons", "`previous_icon` and `next_icon` replace the built-in chevrons on v3's composed Pagination.PreviousIcon and Pagination.NextIcon parts.",
                    col(vec![
                        h::Pagination::new("pg-custom", page, 5)
                            .previous_icon(icon(h::icons::ARROW_LEFT, cx))
                            .next_icon(icon(h::icons::ARROW_RIGHT, cx))
                            .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                                this.pagination_page = *p;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Without controls",
                    col(vec![h::Pagination::new("pg-plain", page, 10)
                        .on_change(usize_cb(cx.listener(|this, p: &usize, _, cx| {
                            this.pagination_page = *p;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_tabs(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let primary = self.tab_solid.clone();
        let secondary = self.tab_underline.clone();
        let items = || {
            vec![
                h::TabItem::new("home", "Home").content(gpui::div().child("The home panel.")),
                h::TabItem::new("music", "Music").content(gpui::div().child("The music panel.")),
                h::TabItem::new("videos", "Videos").content(gpui::div().child("The videos panel.")),
            ]
        };
        component_doc_page!(
            "Tabs",
            crate::pages::Page::Tabs.description(),
            crate::pages::Page::Tabs.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Tabs::new(
                        "tabs-usage",
                        vec![
                            h::TabItem::new("photos", "Photos")
                                .content(gpui::div().child("Your photo library.")),
                            h::TabItem::new("music", "Music")
                                .content(gpui::div().child("Playlists and albums.")),
                            h::TabItem::new("videos", "Videos")
                                .content(gpui::div().child("Everything you have filmed.")),
                        ],
                        "photos",
                    )
                    .into_any_element()]),
                ),
                (
                    "Vertical",
                    col(vec![h::Tabs::new(
                        "tabs-vertical",
                        vec![
                            h::TabItem::new("account", "Account")
                                .content(gpui::div().child("Name, email and password.")),
                            h::TabItem::new("billing", "Billing")
                                .content(gpui::div().child("Cards and invoices.")),
                            h::TabItem::new("team", "Team")
                                .content(gpui::div().child("Members and roles.")),
                        ],
                        "account",
                    )
                    .orientation(Orientation::Vertical)
                    .into_any_element()]),
                ),
                (
                    "Overflow",
                    "More tabs than fit scroll along their axis.",
                    col(vec![
                        para("Horizontal", cx),
                        // The list only overflows inside a bounded box, which is
                        // how v3's own example frames it.
                        gpui::div()
                            .w(px(420.))
                            .child(h::Tabs::new(
                                "tabs-overflow",
                                (1..=12)
                                    .map(|n| {
                                        h::TabItem::new(
                                            SharedString::from(format!("t{n}")),
                                            SharedString::from(format!("Section {n}")),
                                        )
                                        .content(gpui::div().child(format!("Content {n}")))
                                    })
                                    .collect(),
                                "t1",
                            ))
                            .into_any_element(),
                        para("Vertical", cx),
                        gpui::div()
                            .h(px(200.))
                            .child(
                                h::Tabs::new(
                                    "tabs-overflow-vertical",
                                    (1..=8)
                                        .map(|n| {
                                            h::TabItem::new(
                                                SharedString::from(format!("v{n}")),
                                                SharedString::from(format!("Section {n}")),
                                            )
                                            .content(gpui::div().child(format!("Content {n}")))
                                        })
                                        .collect(),
                                    "v1",
                                )
                                .orientation(Orientation::Vertical),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Disabled Tab",
                    col(vec![h::Tabs::new(
                        "tabs-disabled",
                        vec![
                            h::TabItem::new("open", "Open")
                                .content(gpui::div().child("Open items.")),
                            // v3: `<Tabs.Tab isDisabled>` on a single tab; the
                            // selected "open" tab stays live.
                            h::TabItem::new("closed", "Closed")
                                .is_disabled(true)
                                .content(gpui::div().child("Closed items.")),
                        ],
                        "open",
                    )
                    .into_any_element()]),
                ),
                (
                    "With Separator",
                    col(vec![h::Tabs::new(
                        "tabs-separator",
                        vec![
                            h::TabItem::new("one", "One").content(gpui::div().child("First.")),
                            // v3: `<Tabs.Separator />` inside every tab but
                            // the first.
                            h::TabItem::new("two", "Two")
                                .separator()
                                .content(gpui::div().child("Second.")),
                            h::TabItem::new("three", "Three")
                                .separator()
                                .content(gpui::div().child("Third.")),
                        ],
                        "one",
                    )
                    .into_any_element(),]),
                ),
                (
                    "Primary",
                    col(vec![h::Tabs::new("tabs-primary", items(), primary.clone())
                        .selected_key(primary)
                        .on_selection_change(cx.listener(|this, key: &SharedString, _, cx| {
                            this.tab_solid = key.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Secondary",
                    col(vec![h::Tabs::new(
                        "tabs-secondary",
                        items(),
                        secondary.clone(),
                    )
                    .selected_key(secondary)
                    .variant(h::TabsVariant::Secondary)
                    .on_selection_change(cx.listener(|this, key: &SharedString, _, cx| {
                        this.tab_underline = key.clone();
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Overlays
    // -----------------------------------------------------------------------

    pub fn page_alert_dialog(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.alert_dialog_open;
        component_doc_page!(
            "Alert Dialog",
            crate::pages::Page::AlertDialog.description(),
            crate::pages::Page::AlertDialog.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![{
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start()
                                .w_full(),
                            is_open,
                            240.,
                        )
                        .child(
                            h::Button::new("ad-open")
                                .label("Delete project")
                                .variant(Variant::Danger)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.alert_dialog_open = true;
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::AlertDialog::new("Delete this project?").id("ad-usage")
                                .description(
                                    "This removes the project and every deployment. \
                                     This action cannot be undone.",
                                )
                                .is_open(is_open)
                                .child(h::AlertDialogCloseTrigger::new())
                                .footer_child(
                                    h::Button::new("ad-usage-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.alert_dialog_open = false;
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-usage-confirm")
                                        .label("Delete")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.alert_dialog_open = false;
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.alert_dialog_open = *v;
                                    cx.notify();
                                }))),
                        )
                        .into_any_element()
                    }]),
                ),
                (
                    "Sizes",
                    col([
                        ("ad-size-xs", "Xs", h::AlertDialogSize::Xs),
                        ("ad-size-sm", "Sm", h::AlertDialogSize::Sm),
                        ("ad-size-md", "Md", h::AlertDialogSize::Md),
                        ("ad-size-lg", "Lg", h::AlertDialogSize::Lg),
                        ("ad-size-cover", "Cover", h::AlertDialogSize::Cover),
                    ]
                    .into_iter()
                    .map(|(key, label, size)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("Size: {label}")).id(key)
                                .description("Every size shares one panel style.")
                                .is_open(open)
                                .size(size)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Statuses",
                    col([
                        ("ad-st-default", "Default", Color::Default),
                        ("ad-st-accent", "Accent", Color::Accent),
                        ("ad-st-success", "Success", Color::Success),
                        ("ad-st-warning", "Warning", Color::Warning),
                        ("ad-st-danger", "Danger", Color::Danger),
                    ]
                    .into_iter()
                    .map(|(key, label, status)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("{label} status")).id(key)
                                .description("The status colours the icon above the title.")
                                .is_open(open)
                                .status(status)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Placements",
                    col([
                        ("ad-pl-auto", "Auto", h::ModalPlacement::Auto),
                        ("ad-pl-center", "Center", h::ModalPlacement::Center),
                        ("ad-pl-top", "Top", h::ModalPlacement::Top),
                        ("ad-pl-bottom", "Bottom", h::ModalPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::AlertDialog::new(format!("Placement: {label}")).id(key)
                                .description("The panel keeps its own size.")
                                .is_open(open)
                                .placement(placement)
                                .child(h::AlertDialogCloseTrigger::new())
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Backdrop Variants",
                    col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "ad-bd-opaque",
                                herogpui_core::Backdrop::Blur => "ad-bd-blur",
                                herogpui_core::Backdrop::Transparent => "ad-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                backdrop.label(),
                                h::AlertDialog::new(format!("Backdrop: {}", backdrop.label())).id(key)
                                    .description("The scrim behind the panel.")
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .child(h::AlertDialogCloseTrigger::new())
                                    .on_open_change(bool_cb(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    )))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Controlled State",
                    col(vec![overlay_demo(
                        self.demo_overlay("ad-controlled"),
                        "ad-controlled",
                        "Open (controlled)",
                        h::AlertDialog::new("Controlled").id("ad-controlled")
                            .description("The flag lives with the caller; closing reports through onOpenChange.")
                            .is_open(self.demo_overlay("ad-controlled"))
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-controlled", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Icon",
                    col(vec![overlay_demo(
                        self.demo_overlay("ad-icon"),
                        "ad-icon",
                        "Open with a status icon",
                        h::AlertDialog::new("Heads up").id("ad-icon")
                            .description("The status picks the icon, so a warning dialog shows the warning glyph.")
                            .is_open(self.demo_overlay("ad-icon"))
                            .status(Color::Warning)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-icon", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Backdrop",
                    col(vec![overlay_demo(
                        self.demo_overlay("ad-custom-bd"),
                        "ad-custom-bd",
                        "Open with a blurred backdrop",
                        h::AlertDialog::new("Blurred").id("ad-custom-bd")
                            .description("The page behind the panel is blurred.")
                            .is_open(self.demo_overlay("ad-custom-bd"))
                            .backdrop(herogpui_core::Backdrop::Blur)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-custom-bd", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Dismiss Behavior",
                    col(vec![overlay_demo(
                        self.demo_overlay("ad-dismiss"),
                        "ad-dismiss",
                        "Open a non-dismissable dialog",
                        h::AlertDialog::new("Confirm first").id("ad-dismiss")
                            .description("The backdrop and Escape are both inert; the composed X and the actions still close.")
                            .is_open(self.demo_overlay("ad-dismiss"))
                            .is_dismissible(false)
                            .is_keyboard_dismiss_disabled(true)
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-dismiss", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Close Methods",
                    col(vec![
                        overlay_demo(
                            self.demo_overlay("ad-close"),
                            "ad-close",
                            "Open (destructive confirm)",
                            h::AlertDialog::new("Delete for ever?").id("ad-close")
                                .description("A composed footer retires the built-in pair: the danger confirm and the cancel are ordinary Buttons the caller wires to close. The X is not composed here, so the corner slot is bare.")
                                .is_open(self.demo_overlay("ad-close"))
                                .footer_child(
                                    h::Button::new("ad-close-cancel")
                                        .label("Keep")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-close", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-close-confirm")
                                        .label("Delete")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-close", false);
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-close", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                        overlay_demo(
                            self.demo_overlay("ad-pending"),
                            "ad-pending",
                            "Open (pending confirm)",
                            h::AlertDialog::new("Deploying").id("ad-pending")
                                .description("A composed footer Button carries the pending state: it shows a spinner and swallows the press while the action is in flight, so no close is reported; only the cancel closes.")
                                .is_open(self.demo_overlay("ad-pending"))
                                .footer_child(
                                    h::Button::new("ad-pending-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-pending", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-pending-confirm")
                                        .label("Deploy")
                                        .is_pending(true),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-pending", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Animations",
                    col(vec![overlay_demo(
                        self.demo_overlay("ad-anim"),
                        "ad-anim",
                        "Open and watch the panel",
                        h::AlertDialog::new("Animated").id("ad-anim")
                            .description("The panel shrinks in from 105% over 250ms and leaves at 95% over 100ms.")
                            .is_open(self.demo_overlay("ad-anim"))
                            .child(h::AlertDialogCloseTrigger::new())
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ad-anim", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Trigger",
                    col(vec![{
                        let open = self.demo_overlay("ad-custom");
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start()
                                .w_full(),
                            open,
                            120.,
                        )
                        .child(
                            gpui::div()
                                .id("ad-custom-trigger")
                                .cursor_pointer()
                                .child(
                                    h::Chip::new()
                                        .color(Color::Danger)
                                        .variant(h::ChipVariant::Soft)
                                        .child(h::ChipLabel::new().child("Delete account")),
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("ad-custom", true);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::AlertDialog::new("Delete this account?").id("ad-custom")
                                .description("Any element can open an alert dialog; a composed CloseTrigger draws the corner X and a composed footer owns the danger confirm.")
                                .is_open(open)
                                .child(h::AlertDialogCloseTrigger::new())
                                .footer_child(
                                    h::Button::new("ad-custom-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-custom", false);
                                            cx.notify();
                                        })),
                                )
                                .footer_child(
                                    h::Button::new("ad-custom-confirm")
                                        .label("Delete account")
                                        .variant(Variant::Danger)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("ad-custom", false);
                                            cx.notify();
                                        })),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("ad-custom", *v);
                                    cx.notify();
                                })))
                        )
                        .into_any_element()
                    }]),
                ),
            ],
            cx,
        )
    }

    pub fn page_drawer(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.drawer_open;
        component_doc_page!(
            "Drawer",
            crate::pages::Page::Drawer.description(),
            crate::pages::Page::Drawer.import_line(),
            vec![
                (
                    "Placement",
                    col([
                        ("dr-left", "Left", h::DrawerPlacement::Left),
                        ("dr-right", "Right", h::DrawerPlacement::Right),
                        ("dr-top", "Top", h::DrawerPlacement::Top),
                        ("dr-bottom", "Bottom", h::DrawerPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Drawer::new()
                                .id(key)
                                .is_open(open)
                                .placement(placement)
                                .title(format!("From the {label}"))
                                .is_dismissible(true)
                                .child(h::DrawerCloseTrigger::new())
                                .child(gpui::div().child("The panel slides in along its edge."))
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Non-Dismissable",
                    col(vec![overlay_demo(
                        self.demo_overlay("dr-no-dismiss"),
                        "dr-no-dismiss",
                        "Open a non-dismissable drawer",
                        h::Drawer::new()
                            .id("dr-no-dismiss")
                            .is_open(self.demo_overlay("dr-no-dismiss"))
                            .title("Finish first")
                            .is_dismissible(false)
                            .is_keyboard_dismiss_disabled(true)
                            .child(gpui::div().child("The backdrop and Escape are both inert."))
                            .footer_child(
                                h::Button::new("dr-no-dismiss-ok").label("Done").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.set_demo_flag("dr-no-dismiss", false);
                                        cx.notify();
                                    }),
                                ),
                            )
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-no-dismiss", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Scrollable Content",
                    col(vec![overlay_demo(
                        self.demo_overlay("dr-scroll"),
                        "dr-scroll",
                        "Open a long drawer",
                        h::Drawer::new()
                            .id("dr-scroll")
                            .is_open(self.demo_overlay("dr-scroll"))
                            .title("Release notes")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
                            .child(gpui::div().flex().flex_col().gap(px(8.)).children(
                                (1..=20).map(|n| {
                                    gpui::div().child(format!("Change {n} of twenty."))
                                }),
                            ),)
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-scroll", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Controlled State",
                    col(vec![
                        para(
                            &format!(
                                "The flag lives with the caller: {}",
                                if self.demo_overlay("dr-controlled") {
                                    "open"
                                } else {
                                    "closed"
                                }
                            ),
                            cx,
                        ),
                        overlay_demo(
                            self.demo_overlay("dr-controlled"),
                            "dr-controlled",
                            "Open (controlled)",
                            h::Drawer::new()
                                .id("dr-controlled")
                                .is_open(self.demo_overlay("dr-controlled"))
                                .title("Controlled")
                                .is_dismissible(true)
                                .child(h::DrawerCloseTrigger::new())
                                .child(gpui::div().child("Closing reports through onOpenChange."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("dr-controlled", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Form",
                    col(vec![overlay_demo(
                        self.demo_overlay("dr-form"),
                        "dr-form",
                        "Open a form drawer",
                        h::Drawer::new()
                            .id("dr-form")
                            .is_open(self.demo_overlay("dr-form"))
                            .title("New issue")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("dr-form-title", "", cx))
                                            .label("Title"),
                                    )
                                    .child(
                                        h::TextArea::new(self.demo_text("dr-form-body", "", cx))
                                            .label("Description")
                                            .rows(3),
                                    ),
                            )
                            .footer_child(h::Button::new("dr-form-save").label("Create").on_press(
                                cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("dr-form", false);
                                    cx.notify();
                                }),
                            ))
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-form", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Navigation Drawer",
                    col(vec![overlay_demo(
                        self.demo_overlay("dr-nav"),
                        "dr-nav",
                        "Open the navigation",
                        h::Drawer::new()
                            .id("dr-nav")
                            .is_open(self.demo_overlay("dr-nav"))
                            .placement(h::DrawerPlacement::Left)
                            .title("Menu")
                            .is_dismissible(true)
                            .child(h::DrawerCloseTrigger::new())
                            .child(h::ListBox::new(
                                "dr-nav-list",
                                vec![
                                    h::ListBoxItem::new("home", "Home"),
                                    h::ListBoxItem::new("projects", "Projects"),
                                    h::ListBoxItem::new("settings", "Settings"),
                                    h::ListBoxItem::separator(),
                                    h::ListBoxItem::new("logout", "Log out").danger(),
                                ],
                            ))
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("dr-nav", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Backdrop Variants",
                    col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "dr-bd-opaque",
                                herogpui_core::Backdrop::Blur => "dr-bd-blur",
                                herogpui_core::Backdrop::Transparent => "dr-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                backdrop.label(),
                                h::Drawer::new()
                                    .id(key)
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .title(format!("Backdrop: {}", backdrop.label()))
                                    .is_dismissible(true)
                                    .child(h::DrawerCloseTrigger::new())
                                    .child(gpui::div().child("The scrim behind the panel."))
                                    .on_open_change(bool_cb(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    )))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Usage",
                    col(vec![overlay_min_h(
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_start()
                            .w_full(),
                        is_open,
                        240.,
                    )
                    .child(
                        h::Button::new("dr-open")
                            .label("Open drawer")
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.drawer_open = true;
                                cx.notify();
                            }),)
                    )
                    .child(
                        h::Drawer::new()
                            .id("dr-usage")
                            .is_open(is_open)
                            .title("Settings")
                            .placement(h::DrawerPlacement::Right)
                            .child(h::DrawerCloseTrigger::new())
                            .child(gpui::div().child("Panel content goes here."))
                            .footer_child(h::Button::new("dr-done").label("Done").on_press(
                                cx.listener(|this, _, _, cx| {
                                    this.drawer_open = false;
                                    cx.notify();
                                }),
                            ))
                            .on_close(cx.listener(|this, _, _, cx| {
                                this.drawer_open = false;
                                cx.notify();
                            })),
                    )
                    .into_any_element(),]),
                ),
            ],
            cx,
        )
    }

    pub fn page_modal(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.modal_open;
        let md_controlled = self.demo_overlay("md-controlled");
        let md_form = self.demo_overlay("md-form");
        let md_custom = self.demo_overlay("md-custom");
        let md_bd_custom = self.demo_overlay("md-bd-custom");
        let md_no_dismiss = self.demo_overlay("md-no-dismiss");
        let md_close = self.demo_overlay("md-close");
        let md_anim = self.demo_overlay("md-anim");
        component_doc_page!(
            "Modal",
            crate::pages::Page::Modal.description(),
            crate::pages::Page::Modal.import_line(),
            vec![
                (
                    "Sizes",
                    col([
                        ("md-size-xs", "Xs", h::ModalSize::Xs),
                        ("md-size-sm", "Sm", h::ModalSize::Sm),
                        ("md-size-md", "Md", h::ModalSize::Md),
                        ("md-size-lg", "Lg", h::ModalSize::Lg),
                        ("md-size-cover", "Cover", h::ModalSize::Cover),
                        ("md-size-full", "Full", h::ModalSize::Full),
                    ]
                    .into_iter()
                    .map(|(key, label, size)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .size(size)
                                .title(format!("Size: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Every size shares one panel style."))
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Placement",
                    col([
                        ("md-place-auto", "Auto", h::ModalPlacement::Auto),
                        ("md-place-center", "Center", h::ModalPlacement::Center),
                        ("md-place-top", "Top", h::ModalPlacement::Top),
                        ("md-place-bottom", "Bottom", h::ModalPlacement::Bottom),
                    ]
                    .into_iter()
                    .map(|(key, label, placement)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .placement(placement)
                                .title(format!("Placement: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("The panel keeps its own size."))
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Scroll Behavior",
                    col([
                        ("md-scroll-inside", "Inside", h::ModalScroll::Inside),
                        ("md-scroll-outside", "Outside", h::ModalScroll::Outside),
                    ]
                    .into_iter()
                    .map(|(key, label, scroll)| {
                        let open = self.demo_overlay(key);
                        overlay_demo(
                            open,
                            key,
                            label,
                            h::Modal::new()
                                .id(key)
                                .is_open(open)
                                .scroll(scroll)
                                .title(format!("Scroll: {label}"))
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().flex().flex_col().gap(px(8.)).children(
                                    (1..=12).map(|n| {
                                        gpui::div().child(format!("Paragraph {n} of twelve."))
                                    }),
                                ))
                                .on_open_change(bool_cb(cx.listener(
                                    move |this, v: &bool, _, cx| {
                                        this.set_demo_flag(key, *v);
                                        cx.notify();
                                    },
                                )))
                                .into_any_element(),
                            cx,
                        )
                    })
                    .collect()),
                ),
                (
                    "Controlled State",
                    col(vec![
                        para(
                            &format!(
                                "The flag lives with the caller: {}",
                                if md_controlled { "open" } else { "closed" }
                            ),
                            cx,
                        ),
                        overlay_demo(
                            self.demo_overlay("md-controlled"),
                            "md-controlled",
                            "Open (controlled)",
                            h::Modal::new()
                                .id("md-controlled")
                                .is_open(md_controlled)
                                .title("Controlled")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Closing reports through onOpenChange."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-controlled", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Form",
                    col(vec![overlay_demo(
                        self.demo_overlay("md-form"),
                        "md-form",
                        "Open form modal",
                        h::Modal::new()
                            .id("md-form")
                            .is_open(md_form)
                            .title("Invite a teammate")
                            .is_dismissible(true)
                            .child(h::ModalCloseTrigger::new())
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(12.))
                                    .child(
                                        h::TextField::new(self.demo_text("md-form-name", "", cx))
                                            .label("Name"),
                                    )
                                    .child(
                                        h::TextField::new(self.demo_text("md-form-email", "", cx))
                                            .label("Email")
                                            .input_type(h::InputType::Email),
                                    ),
                            )
                            .footer_child(
                                h::Button::new("md-form-send")
                                    .label("Send invite")
                                    .on_press(cx.listener(|this, _, _, cx| {
                                        this.set_demo_flag("md-form", false);
                                        cx.notify();
                                    })),
                            )
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("md-form", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                        cx,
                    )]),
                ),
                (
                    "Custom Trigger",
                    col(vec![overlay_min_h(
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_start()
                            .w_full(),
                        md_custom,
                        120.,
                    )
                        .child(
                            gpui::div()
                                .id("md-custom-trigger")
                                .cursor_pointer()
                                .child(h::Avatar::new("md-custom-avatar").name("Jane Doe"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.set_demo_flag("md-custom", true);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Modal::new()
                                .id("md-custom")
                                .is_open(md_custom)
                                .title("Jane Doe")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Any element can open a modal."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-custom", *v);
                                    cx.notify();
                                }))),
                        )
                        .into_any_element()]),
                ),
                (
                    "Backdrop Variants",
                    col(herogpui_core::Backdrop::ALL
                        .iter()
                        .map(|backdrop| {
                            let key: &'static str = match backdrop {
                                herogpui_core::Backdrop::Opaque => "md-bd-opaque",
                                herogpui_core::Backdrop::Blur => "md-bd-blur",
                                herogpui_core::Backdrop::Transparent => "md-bd-transparent",
                            };
                            let open = self.demo_overlay(key);
                            overlay_demo(
                                open,
                                key,
                                backdrop.label(),
                                h::Modal::new()
                                    .id(key)
                                    .is_open(open)
                                    .backdrop(*backdrop)
                                    .title(format!("Backdrop: {}", backdrop.label()))
                                    .is_dismissible(true)
                                    .child(h::ModalCloseTrigger::new())
                                    .child(gpui::div().child("The scrim behind the panel."))
                                    .on_open_change(bool_cb(cx.listener(
                                        move |this, v: &bool, _, cx| {
                                            this.set_demo_flag(key, *v);
                                            cx.notify();
                                        },
                                    )))
                                    .into_any_element(),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Custom Backdrop", "v3 restyles the backdrop with a class. `Backdrop::Blur` is the strongest variant the token set has; anything past it is the caller's own scrim.",
                    col(vec![
                        overlay_demo(
                            self.demo_overlay("md-bd-custom"),
                            "md-bd-custom",
                            "Open with a blurred backdrop",
                            h::Modal::new()
                                .id("md-bd-custom")
                                .is_open(md_bd_custom)
                                .backdrop(h::Backdrop::Blur)
                                .title("Blurred")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("The page behind is blurred."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-bd-custom", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Dismiss Behavior", "`isDismissible` decides whether the backdrop closes it; `isKeyboardDismissDisabled` decides whether Escape does.",
                    col(vec![
                        overlay_demo(
                            self.demo_overlay("md-no-dismiss"),
                            "md-no-dismiss",
                            "Open a non-dismissable modal",
                            h::Modal::new()
                                .id("md-no-dismiss")
                                .is_open(md_no_dismiss)
                                .title("Confirm first")
                                .is_dismissible(false)
                                .is_keyboard_dismiss_disabled(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child(
                                    "The backdrop and Escape are both inert; the composed X and the button still close.",
                                ))
                                .footer_child(
                                    h::Button::new("md-no-dismiss-ok").label("Got it").on_press(
                                        cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("md-no-dismiss", false);
                                            cx.notify();
                                        }),
                                    ),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-no-dismiss", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Close Methods", "v3 spells the close affordance by composition: the `Close Methods` example closes through footer buttons and composes no close trigger, so the corner slot stays bare. Every other example composes the built-in close trigger for the corner X.",
                    col(vec![
                        overlay_demo(
                            self.demo_overlay("md-close"),
                            "md-close",
                            "Open (no close trigger)",
                            h::Modal::new()
                                .id("md-close")
                                .is_open(md_close)
                                .title("Close me")
                                .is_dismissible(true)
                                .child(
                                    gpui::div().child(
                                        "The corner slot is bare; the footer button closes."
                                    )
                                )
                                .footer_child(
                                    h::Button::new("md-close-ok").label("Close").on_press(
                                        cx.listener(|this, _, _, cx| {
                                            this.set_demo_flag("md-close", false);
                                            cx.notify();
                                        }),
                                    ),
                                )
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-close", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Animations", "v3 overrides the panel's duration and easing per instance with a class. The motion here is the one its stylesheet declares: the panel shrinks in from 105% over 250ms on `ease-out-quad` and leaves at 95% over 100ms. `Motion on` in the navbar switches it off, which is the `prefers-reduced-motion` path.",
                    col(vec![
                        overlay_demo(
                            self.demo_overlay("md-anim"),
                            "md-anim",
                            "Open and watch the panel",
                            h::Modal::new()
                                .id("md-anim")
                                .is_open(md_anim)
                                .title("Animated")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Close and reopen to see it again."))
                                .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("md-anim", *v);
                                    cx.notify();
                                })))
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Usage",
                    col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start()
                                .w_full(),
                            is_open,
                            280.,
                        )
                        .child(
                            h::Button::new("md-open")
                                .label("Open modal")
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.modal_open = true;
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Modal::new()
                                .id("md-usage")
                                .is_open(is_open)
                                // `Modal.Icon` sits above the heading.
                                .icon(h::icons::MAIL)
                                .title("Create account")
                                .is_dismissible(true)
                                .child(h::ModalCloseTrigger::new())
                                .child(gpui::div().child("Sign up to get started with HeroGPUI."))
                                .footer_child(
                                    h::Button::new("md-cancel")
                                        .label("Cancel")
                                        .variant(Variant::Tertiary)
                                        .on_press(cx.listener(|this, _, _, cx| {
                                            this.modal_open = false;
                                            cx.notify();
                                        })),
                                )
                                .footer_child(h::Button::new("md-ok").label("Sign up").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.modal_open = false;
                                        cx.notify();
                                    }),
                                ))
                                .on_close(cx.listener(|this, _, _, cx| {
                                    this.modal_open = false;
                                    cx.notify();
                                })),
                        )
                        .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_popover(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.popover_open;
        let po_following = self.demo_flag("po-following", false);
        let po_arrow_open = self.demo_overlay("po-arrow");
        let po_arrow_custom_open = self.demo_overlay("po-arrow-custom");
        let po_interactive_open = self.demo_overlay("po-interactive");
        let po_render_open = self.demo_overlay("po-render-function");
        let po_custom_styles_open = self.demo_overlay("po-custom-styles");
        let colors = cx.colors().clone();
        let custom_border = colors.border.alpha(0.8);
        let custom_surface = colors.surface.background.alpha(0.9);
        let custom_tint = colors
            .default
            .color
            .alpha(if cx.is_dark_theme() { 0.08 } else { 0.06 });
        let custom_shadow = cx.layout().overlay_shadow.clone();
        let usage_slot = gpui::div().relative().flex().flex_col().items_start();
        component_doc_page!(
            "Popover",
            crate::pages::Page::Popover.description(),
            crate::pages::Page::Popover.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![overlay_min_h(usage_slot, is_open, 160.)
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-trigger")
                                            .label("Open popover")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .is_open(is_open)
                            .title("Quick note")
                            .placement(h::PopoverPlacement::Bottom)
                            .child(gpui::div().child("Popovers are anchored to their trigger."))
                            .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                set_popover_open(
                                    &mut this.popover_open,
                                    &mut this.demo_flags,
                                    "po-usage",
                                    *open,
                                );
                                cx.notify();
                            }))),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Arrow", "`PopoverArrow::new()` composes v3's `Popover.Arrow` part: the built-in 12px curve follows the resolved side when the panel flips and preserves the configured offset. A custom child element takes the resolved position but no rotation: upstream rotates it through its `data-slot` placement CSS, which GPUI 0.2.2 cannot reproduce on an arbitrary element (only `svg()` transforms at construction).",
                    col(vec![
                        gpui::div()
                            .relative()
                            .flex()
                            .flex_wrap()
                            .items_start()
                            .gap(px(24.))
                            .pl(px(48.))
                            .child(
                                overlay_min_h(
                                    gpui::div()
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .items_start(),
                                    po_arrow_open,
                                    160.,
                                )
                                .child(
                                h::Popover::new(
                                    h::Button::new("po-arrow-trigger")
                                        .label("Offset by 12px")
                                        .variant(Variant::Secondary),
                                )
                                .id("po-arrow")
                                .is_open(po_arrow_open)
                                .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-arrow",
                                        *open,
                                    );
                                    cx.notify();
                                })))
                                .offset(px(12.))
                                .title("Anchored")
                                .child(gpui::div().child("Twelve pixels clear of the trigger."))
                                .child(h::PopoverArrow::new()),
                            )
                            )
                            .child(
                                overlay_min_h(
                                    gpui::div()
                                        .relative()
                                        .flex()
                                        .flex_col()
                                        .items_start(),
                                    po_arrow_custom_open,
                                    160.,
                                )
                                .child(
                                    h::Popover::new(
                                        h::Button::new("po-arrow-custom-trigger")
                                            .label("Custom arrow")
                                            .variant(Variant::Secondary),
                                    )
                                    .id("po-arrow-custom")
                                    .is_open(po_arrow_custom_open)
                                    .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                        set_popover_open(
                                            &mut this.popover_open,
                                            &mut this.demo_flags,
                                            "po-arrow-custom",
                                            *open,
                                        );
                                        cx.notify();
                                    })))
                                    .offset(px(12.))
                                    .title("Custom arrow")
                                    .child(gpui::div().child("A caller-drawn element, not the curve."))
                                    .child(
                                        h::PopoverArrow::new().child(
                                            gpui::svg()
                                                .size(px(12.))
                                                .path(h::icons::TOOLTIP_ARROW)
                                                .text_color(cx.colors().accent.foreground),
                                        ),
                                    ),
                                )
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Interactive Content",
                    col(vec![{
                        let interactive_slot = gpui::div()
                            .relative()
                            .flex()
                            .flex_col()
                            .items_start();
                        overlay_min_h(interactive_slot, po_interactive_open, 220.)
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .pr(px(96.))
                                    .child(h::Avatar::new("po-avatar").name("Sarah Johnson").size(Size::Sm))
                                    .child(gpui::div().child("Sarah Johnson")),
                            )
                            .id("po-interactive")
                            .is_open(po_interactive_open)
                            .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                set_popover_open(
                                    &mut this.popover_open,
                                    &mut this.demo_flags,
                                    "po-interactive",
                                    *open,
                                );
                                cx.notify();
                            })))
                            .title("Sarah Johnson")
                            .child(
                                gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(10.))
                                    .child(gpui::div().child("Design lead, Berlin"))
                                    .child(
                                        h::Button::new("po-follow")
                                            .label(if po_following {
                                                "Following"
                                            } else {
                                                "Follow"
                                            })
                                            .size(Size::Sm)
                                            .variant(if po_following {
                                                Variant::Secondary
                                            } else {
                                                Variant::Primary
                                            })
                                            .on_press(cx.listener(move |this, _, _, cx| {
                                                this.set_demo_flag("po-following", !po_following);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                        )
                        .into_any_element()
                    }]),
                ),
                (
                    "Placement",
                    col(vec![gpui::div()
                            .relative()
                            .flex()
                            .flex_wrap()
                            .gap(px(24.))
                        .children(
                            [
                                ("po-pl-top", "Top", h::PopoverPlacement::Top),
                                ("po-pl-bottom", "Bottom", h::PopoverPlacement::Bottom),
                                ("po-pl-left", "Left", h::PopoverPlacement::Left),
                                ("po-pl-right", "Right", h::PopoverPlacement::Right),
                            ]
                            .into_iter()
                            .map(|(id, label, placement)| {
                                let open = self.demo_overlay(id);
                                let mut placement_slot = gpui::div()
                                    .relative()
                                    .flex()
                                    .flex_col()
                                    .items_start();
                                if open && matches!(placement, h::PopoverPlacement::Top) {
                                    placement_slot = placement_slot.pt(px(96.));
                                }
                                if open
                                    && matches!(
                                        placement,
                                        h::PopoverPlacement::Top | h::PopoverPlacement::Left
                                    )
                                {
                                    placement_slot = placement_slot.pl(px(104.));
                                }
                                overlay_min_h(placement_slot, open, 260.).child(
                                    h::Popover::new(
                                        h::Button::new(el_id(format!("{id}-trigger")))
                                            .label(label)
                                            .variant(Variant::Secondary)
                                            .size(Size::Sm),
                                    )
                                    .id(id)
                                    .is_open(open)
                                    .on_open_change(bool_cb(cx.listener(
                                        move |this, open: &bool, _, cx| {
                                            set_popover_open(
                                                &mut this.popover_open,
                                                &mut this.demo_flags,
                                                id,
                                                *open,
                                            );
                                            cx.notify();
                                        },
                                    )))
                                    .placement(placement)
                                    .title(label)
                                .child(gpui::div().child("Anchored to its trigger.")),
                                )
                            }),
                        )
                        .into_any_element()]),
                ),
                (
                    "Render Function", "The pinned Render Function replaces the Popover content's DOM element with a callback. This GPUI Popover has no content or state render callback, so the controlled panel records that limitation instead of faking an API.",
                    col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start(),
                            po_render_open,
                            160.,
                        )
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-render-function-trigger")
                                            .label("Click me")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .id("po-render-function")
                            .is_open(po_render_open)
                            .on_open_change(bool_cb(cx.listener(
                                |this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-render-function",
                                        *open,
                                    );
                                    cx.notify();
                                },
                            )))
                            .child(
                                gpui::div()
                                    .child(
                                        gpui::div()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child("Popover Title"),
                                    )
                                    .child(
                                        gpui::div()
                                            .mt(px(8.))
                                            .text_size(px(14.))
                                            .text_color(colors.muted)
                                            .child(
                                                "This is the popover content. You can put any content here.",
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Styles", "The pinned DOM styling is expressed here with public GPUI builders: `w`, `overflow_hidden`, `rounded`, `border_color`, `bg`, `shadow`, spacing, and `font_family`, using the active theme tokens. GPUI 0.2.2 has no DOM class, ring, gradient, or backdrop-blur hooks, so the styling belongs to the composed content element.",
                    col(vec![
                        overlay_min_h(
                            gpui::div()
                                .relative()
                                .flex()
                                .flex_col()
                                .items_start(),
                            po_custom_styles_open,
                            160.,
                        )
                        .child(
                            h::Popover::new(
                                gpui::div()
                                    .pr(px(96.))
                                    .child(
                                        h::Button::new("po-custom-styles-trigger")
                                            .label("Details")
                                            .variant(Variant::Secondary),
                                    ),
                            )
                            .id("po-custom-styles")
                            .is_open(po_custom_styles_open)
                            .on_open_change(bool_cb(cx.listener(
                                |this, open: &bool, _, cx| {
                                    set_popover_open(
                                        &mut this.popover_open,
                                        &mut this.demo_flags,
                                        "po-custom-styles",
                                        *open,
                                    );
                                    cx.notify();
                                },
                            )))
                            .child(
                                gpui::div()
                                    .relative()
                                    .w(px(224.))
                                    .overflow_hidden()
                                    .rounded(px(12.))
                                    .border_1()
                                    .border_color(custom_border)
                                    .bg(custom_surface)
                                    .shadow(custom_shadow)
                                    .p(px(16.))
                                    .child(
                                        gpui::div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .right_0()
                                            .h(px(48.))
                                            .bg(custom_tint),
                                    )
                                    .child(
                                        gpui::div()
                                            .relative()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(colors.foreground)
                                            .child("Keyboard shortcuts"),
                                    )
                                    .child(
                                        gpui::div()
                                            .relative()
                                            .mt(px(12.))
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.))
                                            .text_size(px(14.))
                                            .child(
                                                gpui::div()
                                                    .flex()
                                                    .justify_between()
                                                    .gap(px(16.))
                                                    .child(
                                                        gpui::div()
                                                            .text_color(colors.muted)
                                                            .child("Save"),
                                                    )
                                                    .child(
                                                        gpui::div()
                                                            .font_family(crate::app::MONO_FONT)
                                                            .text_color(colors.foreground)
                                                            .child("⌘ S"),
                                                    ),
                                            )
                                            .child(
                                                gpui::div()
                                                    .flex()
                                                    .justify_between()
                                                    .gap(px(16.))
                                                    .child(
                                                        gpui::div()
                                                            .text_color(colors.muted)
                                                            .child("Search"),
                                                    )
                                                    .child(
                                                        gpui::div()
                                                            .font_family(crate::app::MONO_FONT)
                                                            .text_color(colors.foreground)
                                                            .child("⌘ K"),
                                                    ),
                                            ),
                                    ),
                            ),
                        )
                        .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_toast(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let toast_closed = crate::app::toasts_closed(cx);
        component_doc_page!(
            "Toast",
            crate::pages::Page::Toast.description(),
            crate::pages::Page::Toast.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::Button::new("toast-usage")
                        .label("Show a toast")
                        .variant(Variant::Secondary)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Saved")
                                .description("Your changes are live.")
                                .closable(true)
                                .push(Some(std::time::Duration::from_secs(4)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            let color = *c;
                            h::Button::new(el_id(format!("toast-v-{c:?}")))
                                .label(c.label())
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(format!("{} toast", color.label()))
                                        .description("One variant per status colour.")
                                        .variant(color)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                        })
                        .els()),
                ),
                (
                    "Placements", "The viewport decides where the stack sits. This gallery mounts one `ToastViewport` in its shell; each button moves it and pushes a toast into that corner.",
                    col(vec![
                        row([
                            ("TopStart", h::ToastPlacement::TopStart),
                            ("Top", h::ToastPlacement::Top),
                            ("TopEnd", h::ToastPlacement::TopEnd),
                            ("BottomStart", h::ToastPlacement::BottomStart),
                            ("Bottom", h::ToastPlacement::Bottom),
                            ("BottomEnd", h::ToastPlacement::BottomEnd),
                        ]
                        .into_iter()
                        .map(|(label, placement)| {
                            h::Button::new(el_id(format!("toast-pl-{label}")))
                                .label(label)
                                .variant(Variant::Tertiary)
                                .size(Size::Sm)
                                // Move the viewport, then push into it: six
                                // buttons that all pushed into the same corner
                                // showed nothing about `placement`.
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.toast_placement = placement;
                                    h::Toast::new(label)
                                        .description("Pushed into the shell's viewport.")
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                        .collect()),
                    ]),
                ),
                (
                    "Simple Toasts",
                    row(vec![h::Button::new("toast-simple")
                        .label("Title only")
                        .variant(Variant::Secondary)
                        .size(Size::Sm)
                        .on_press(|_, _, cx| {
                            h::Toast::new("Copied to the clipboard")
                                .push(Some(std::time::Duration::from_secs(3)), cx);
                        })
                        .into_any_element()]),
                ),
                (
                    "Custom Indicators", "The variant picks the glyph — success shows a tick, danger a crossed circle. `indicator` overrides it with any icon, and `indicator(None)` is v3's `indicator={null}`: no glyph at all.",
                    col(vec![
                        row(vec![
                            h::Button::new("toast-ind-success")
                                .label("Success")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::success("Deployed")
                                        .description("Build 412 is live.")
                                        .closable(true)
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-danger")
                                .label("Danger")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::error("Deploy failed")
                                        .description("Two tests did not pass.")
                                        .closable(true)
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-custom")
                                .label("Custom glyph")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::new("One new message")
                                        .description("From martha@heroui.com.")
                                        .indicator(SharedString::from(h::icons::MAIL))
                                        .push(None, cx);
                                })
                                .into_any_element(),
                            h::Button::new("toast-ind-none")
                                .label("No glyph")
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(|_, _, cx| {
                                    h::Toast::success("Saved").indicator(None).push(None, cx);
                                })
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Toast Rendering", "A toast is a title, a description and a status. Anything richer is the caller's own panel: v3's example renders its own body inside the queue's slot.",
                    col(vec![
                        row(vec![h::Button::new("toast-custom")
                            .label("Push a two-line toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                h::Toast::new("Jane invited you")
                                    .description("Acme workspace \u{2014} Owner")
                                    .variant(Color::Accent)
                                    .closable(true)
                                    .push(Some(std::time::Duration::from_secs(5)), cx);
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Promise & Loading", "`toast.promise` shows a loading toast while the work runs, then replaces it. `Toast::loading` is the pending half: a spinner, and no timeout, so it waits to be closed.",
                    col(vec![
                        row(vec![h::Button::new("toast-promise")
                            .label("Upload a file")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, window, cx| {
                                let id = h::Toast::loading("Uploading\u{2026}")
                                    .description("document.pdf")
                                    .push(None, cx);
                                // The resolution replaces the pending toast,
                                // which is what v3's promise helper does.
                                window
                                    .spawn(cx, async move |cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(1500))
                                            .await;
                                        cx.update(|_window, cx| {
                                            h::dismiss_toast(id, cx);
                                            h::Toast::success("Uploaded")
                                                .description("document.pdf \u{2014} 1 KB")
                                                .closable(true)
                                                .action("View", |_| {})
                                                .push(None, cx);
                                        })
                                        .ok();
                                    })
                                    .detach();
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Callbacks", "`onClose` runs however the toast goes -- dismissed by hand or timed out -- so the count follows the toast, not the button.",
                    col(vec![
                        para(&format!("Toasts closed so far: {toast_closed}"), cx),
                        row(vec![h::Button::new("toast-callback")
                            .label("Push a closable toast")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                h::Toast::new("Dismiss me")
                                    .description("Or wait four seconds.")
                                    .closable(true)
                                    .on_close(crate::app::bump_toast_closed)
                                    .push(None, cx);
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Custom Queues", "`maxVisibleToasts` caps visibility without dropping overflow: the ones past the cap wait their turn. Push four and watch one queue.",
                    col(vec![
                        row(vec![h::Button::new("toast-queue")
                            .label("Push four")
                            .variant(Variant::Secondary)
                            .size(Size::Sm)
                            .on_press(|_, _, cx| {
                                for n in 1..=4 {
                                    h::Toast::new(format!("Message {n}"))
                                        .description("Three are visible at a time.")
                                        .push(Some(std::time::Duration::from_secs(3)), cx);
                                }
                            })
                            .into_any_element()]),
                    ]),
                ),
                (
                    "Setup", "A toast needs a viewport somewhere in the tree. This gallery mounts one in its shell, which is why every page can push.",
                    col(vec![
                        crate::pages::code_block(TOAST_SETUP, cx),
                    ]),
                ),
                (
                    "Push a toast",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            let color = *c;
                            h::Button::new(el_id(format!("toast-{c:?}")))
                                .label(c.label())
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(move |_, _, cx| {
                                    h::Toast::new(format!("{} toast", color.label()))
                                        .description("Pushed from the gallery.")
                                        .variant(color)
                                        .closable(true)
                                        .push(Some(std::time::Duration::from_secs(4)), cx);
                                })
                        })
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_tooltip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Tooltip",
            crate::pages::Page::Tooltip.description(),
            crate::pages::Page::Tooltip.import_line(),
            vec![
                (
                    "With Arrow",
                    row(vec![
                        h::Tooltip::new("With an arrow")
                            .show_arrow(true)
                            // `offset` is the gap between trigger and panel.
                            .offset(px(10.))
                            .child(
                                h::Button::new("tt-arrow-on")
                                    .label("Arrow")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Without one")
                            .show_arrow(false)
                            .child(
                                h::Button::new("tt-arrow-off")
                                    .label("No arrow")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Custom Triggers",
                    row(vec![
                        h::Tooltip::new("Jane Doe")
                            .delay(0)
                            .show_arrow(true)
                            .child(h::Avatar::new("tt-avatar").name("Jane Doe").size(Size::Sm))
                            .into_any_element(),
                        h::Tooltip::new("Verified account")
                            .delay(0)
                            .child(
                                h::Chip::new()
                                    .color(Color::Success)
                                    .variant(h::ChipVariant::Soft)
                                    .child(h::ChipLabel::new().child("Verified")),
                            )
                            .into_any_element(),
                        h::Tooltip::new("What is this?")
                            .delay(0)
                            .child(icon(h::icons::SEARCH, cx))
                            .into_any_element(),
                        h::Tooltip::new("Tab to me")
                            .delay(0)
                            // `trigger="focus"`: the pointer does nothing and
                            // keyboard focus is what opens it.
                            .trigger(h::TooltipTrigger::Focus)
                            .child(
                                h::Button::new("tt-focus-only")
                                    .label("Focus only")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",
                    row(vec![
                        h::Tooltip::new("Above the trigger")
                            .placement(h::TooltipPlacement::Top)
                            .child(
                                h::Button::new("tt-top")
                                    .label("Top")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Below the trigger")
                            .placement(h::TooltipPlacement::Bottom)
                            .child(
                                h::Button::new("tt-bottom")
                                    .label("Bottom")
                                    .variant(Variant::Secondary),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Placements",
                    row(h::TooltipPlacement::ALL
                        .iter()
                        .map(|p| {
                            h::Tooltip::new(p.label())
                                .placement(*p)
                                .show_arrow(true)
                                .child(
                                    h::Button::new(el_id(format!("tip-{}", p.label())))
                                        .label(p.label())
                                        .variant(Variant::Secondary)
                                        .into_any_element(),
                                )
                                .into_any_element()
                        })
                        .collect()),
                ),
                (
                    "Delay",
                    row(vec![
                        h::Tooltip::new("Opens at once")
                            .delay(0)
                            .child(
                                h::Button::new("tip-instant")
                                    .label("No delay")
                                    .into_any_element(),
                            )
                            .into_any_element(),
                        h::Tooltip::new("Waits half a second")
                            .delay(500)
                            .close_delay(0)
                            .child(h::Button::new("tip-slow").label("500ms").into_any_element())
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Pickers
    // -----------------------------------------------------------------------

    pub fn page_autocomplete(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let ac_picked = self.demo_text_value("ac-picked");
        let ac_typed = self.demo_text_value("ac-typed");
        let ac_multi = self.demo_selection("ac-multi");
        let ac_open = self.demo_flag("ac-open", false);
        component_doc_page!(
            "Autocomplete",
            crate::pages::Page::Autocomplete.description(),
            crate::pages::Page::Autocomplete.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::Autocomplete::new(
                        self.ac_entity.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .into_any_element()]),
                ),
                (
                    "Virtualization", "v3 wraps the popover's list in React Aria's `Virtualizer`; `row_height` carries that here, and gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    col(vec![
                        demo_field(
                            h::Autocomplete::new(
                                self.demo_text("ac-virtual", "", cx),
                                virtual_picker_items(),
                            )
                            .label("User")
                            .placeholder("Select a user")
                            .row_height(px(40.)),
                        ),
                    ]),
                ),
                (
                    "Variants",
                    field_col(vec![
                        h::Autocomplete::new(self.demo_text("ac-primary", "", cx), language_items())
                            .label("Primary")
                            .placeholder("Select a language")
                            .into_any_element(),
                        h::Autocomplete::new(self.demo_text("ac-secondary", "", cx), language_items())
                            .label("Secondary")
                            .placeholder("Select a language")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Autocomplete::new(self.demo_text("ac-surface", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
                        .w(px(400.))
                        .child(
                            h::Autocomplete::new(
                                self.demo_text("ac-full", "", cx),
                                language_items(),
                            )
                            .label("Language")
                            .placeholder("Select a language")
                            .full_width(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-desc", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .description("Type to filter the list")
                    .into_any_element()]),
                ),
                (
                    "Required",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-required", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled-opts", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .disabled_keys([SharedString::from("go"), SharedString::from("python")])
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Allows Empty Collection",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-empty", "zzz", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .allows_empty_collection(true)
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-sections", "", cx),
                        vec![
                            h::PickerItem::new("rust", "Rust"),
                            h::PickerItem::new("go", "Go"),
                            h::PickerItem::new("typescript", "TypeScript"),
                            h::PickerItem::new("python", "Python"),
                        ],
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .section_before("rust", "Systems")
                    .section_before("typescript", "Scripting")
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Multiple Select",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-multi-select", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    // `defaultValue` is `Key | Key[]`: the uncontrolled
                    // selection, seeded once, by key.
                    .default_value(["rust"])
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Autocomplete::new(self.demo_text("ac-controlled", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .input_value(ac_typed)
                            .on_input_change(cx.listener(|this, text: &str, _, cx| {
                                this.set_demo_text_value("ac-typed", text.to_owned());
                                cx.notify();
                            }))
                            .on_change(cx.listener(|this, key: &SharedString, _, cx| {
                                this.set_demo_text_value("ac-picked", key.to_string());
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(
                            &if ac_picked.is_empty() {
                                "Nothing picked yet".to_owned()
                            } else {
                                format!("Picked: {ac_picked}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled Multiple",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-ctl-multi", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(ac_multi.iter().cloned())
                    .on_selection_change_all(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.set_demo_selection("ac-multi", keys.to_vec());
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("ac-open-btn")
                                .label(if ac_open { "Close" } else { "Open" })
                                .size(Size::Sm)
                                .variant(Variant::Secondary)
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.set_demo_flag("ac-open", !ac_open);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if ac_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Autocomplete::new(self.demo_text("ac-open", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .is_open(ac_open)
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ac-open", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Filtering", "v3 fetches the matches as the query changes. `filter` is the hook for that -- it decides what counts as a match -- and a spinner beside the field says a request is in flight.",
                    col(vec![
                        row(vec![
                            h::Autocomplete::new(self.demo_text("ac-async", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                // `useFilter({sensitivity: "base"}).contains`:
                                // case and accents both ignored, so "cafe"
                                // finds "Café". The closure receives
                                // `(item_label, input)`.
                                .filter(|item, input| {
                                    h::Filter::new(h::Sensitivity::Base).contains(item, input)
                                })
                                .into_any_element(),
                            h::Spinner::new("ac-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-indicator", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .default_open(true)
                    .indicator(|is_open| {
                        gpui::div()
                            .text_size(px(16.))
                            .child(if is_open { "−" } else { "+" })
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Custom Value", "`Autocomplete.Value` takes a render function, and v3 hands it `defaultChildren`, `isPlaceholder`, `selectedItems` and `selectedText`. This one draws the selection as tags and hands the default back while nothing is chosen, which is what v3's own example does.",
                    col(vec![
                        h::Autocomplete::new(self.demo_text("ac-custom", "", cx), language_items())
                            .label("Languages")
                            .placeholder("Select languages")
                            .selection_mode(SelectionMode::Multiple)
                            .default_value(["rust", "go"])
                            .value_content(|value| {
                                if value.is_placeholder {
                                    return value.default_children;
                                }
                                // v3's own example draws the selection as a
                                // `TagGroup` of `Tag`s, which is what a
                                // multiple-selection trigger looks like there.
                                h::TagGroup::new(
                                    "ac-custom-tags",
                                    value
                                        .selected_items
                                        .iter()
                                        .map(|item| h::Tag::new(item.clone(), item.clone()))
                                        .collect(),
                                )
                                .size(Size::Sm)
                                .into_any_element()
                            })
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_combo_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.combo_open;
        let cb_picked = self.demo_text_value("cb-picked");
        let cb_typed = self.demo_text_value("cb-typed");
        let cb_multi = self.demo_selection("cb-multi");
        let cb_value = self
            .demo_selections
            .get("cb-value")
            .cloned()
            .unwrap_or_else(|| vec![SharedString::from("rust")]);
        component_doc_page!(
            "Combo Box",
            crate::pages::Page::ComboBox.description(),
            crate::pages::Page::ComboBox.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .is_open(is_open)
                    .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                        this.combo_open = *open;
                        cx.notify();
                    })))
                    .on_selection_change(cx.listener(|this, _key: &SharedString, _, cx| {
                        this.combo_open = false;
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Virtualization", "v3 wraps the popover's list in React Aria's `Virtualizer`; `row_height` carries that here, and gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    col(vec![
                        demo_field(
                            h::ComboBox::new(
                                self.demo_text("cb-virtual", "", cx),
                                virtual_picker_items(),
                            )
                            .label("User")
                            .placeholder("Select a user")
                            .row_height(px(40.)),
                        ),
                    ]),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
                        .w(px(400.))
                        .child(
                            h::ComboBox::new(
                                self.demo_text("cb-full", "", cx),
                                language_items(),
                            )
                            .label("Language")
                            .placeholder("Pick or type")
                            .full_width(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Description",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-desc", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .description("Pick from the list or type your own")
                    .into_any_element()]),
                ),
                (
                    "Required",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-required", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Read Only",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-readonly", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_read_only(true)
                    .into_any_element()]),
                ),
                (
                    "In Surface",
                    field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::ComboBox::new(self.demo_text("cb-surface", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled-opts", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .disabled_keys([SharedString::from("go")])
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-sections", "", cx),
                        vec![
                            h::PickerItem::new("rust", "Rust"),
                            h::PickerItem::new("go", "Go"),
                            h::PickerItem::new("typescript", "TypeScript"),
                            h::PickerItem::new("python", "Python"),
                        ],
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .section_before("rust", "Systems")
                    .section_before("typescript", "Scripting")
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ComboBox::new(self.demo_text("cb-controlled", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            // `selectedKey` is the controlled selection key
                            // (empty string is v3's `null`); the input shows
                            // that key's label and `inputValue` holds the
                            // typed text. The pick reports the key, which is
                            // what the line below displays.
                            .selected_key(cb_picked.clone(), cx)
                            .input_value(cb_typed.clone(), cx)
                            .on_selection_change(cx.listener(
                                |this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("cb-picked", key.to_string());
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &if cb_picked.is_empty() {
                                "Nothing picked yet".to_owned()
                            } else {
                                format!("Picked: {cb_picked}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled Input Value",
                    col(vec![
                        h::ComboBox::new(self.demo_text("cb-input", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .on_input_change(cx.listener(|this, text: &str, _, cx| {
                                this.set_demo_text_value("cb-typed", text.to_owned());
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Typed: {cb_typed}"), cx),
                    ]),
                ),
                (
                    "Controlled Selection",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-ctl-sel", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(cb_multi.iter().cloned())
                    .on_selection_change_all(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.set_demo_selection("cb-multi", keys.to_vec());
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Multiple Selection",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-multi-sel", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Value Render Props",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-value", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .selected_keys(cb_value.iter().cloned())
                    .on_selection_change_all(cx.listener(
                        |this, keys: &[SharedString], _, cx| {
                            this.set_demo_selection("cb-value", keys.to_vec());
                            cx.notify();
                        },
                    ))
                    .value_content(|value| {
                        if value.is_placeholder {
                            gpui::div()
                                .text_size(px(14.))
                                .child("No language selected")
                                .into_any_element()
                        } else {
                            value.default_children
                        }
                    })
                    .into_any_element()]),
                ),
                (
                    "Default Selected Key",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-default-key", "TypeScript", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Search languages...")
                    .default_value(["typescript"])
                    .into_any_element()]),
                ),
                (
                    "Allows Custom Value",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-custom", "Zig", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .allows_custom_value(true)
                    .into_any_element()]),
                ),
                (
                    "Asynchronous Loading", "v3 fills the list from a request. The spinner beside the field is what says one is in flight; the options are the caller's own data. `allowsEmptyCollection` keeps the panel up with its empty state while a query has no matches instead of collapsing it.",
                    col(vec![
                        row(vec![
                            h::ComboBox::new(self.demo_text("cb-async", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .allows_empty_collection(true)
                                // v3 pairs the flag with async loading: type a
                                // query nothing matches and the panel stays up
                                // with "No matching options" instead of closing.
                                .filter(|item, input| {
                                    h::Filter::new(h::Sensitivity::Base).contains(item, input)
                                })
                                .into_any_element(),
                            h::Spinner::new("cb-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-indicator", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .indicator(|is_selected| {
                        gpui::div()
                            .text_size(px(12.))
                            .child(if is_selected { "\u{2714}" } else { "" })
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Custom Filtering", "`defaultFilter` here is `useFilter`'s `startsWith`, so it matches on the start of the name only.",
                    col(vec![
                        h::ComboBox::new(self.demo_text("cb-filter", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .filter(|item, input| {
                                h::Filter::new(h::Sensitivity::Base).starts_with(item, input)
                            })
                            .default_open(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Menu Trigger",
                    col(vec![
                        spec(
                            "Input (opens as you type)",
                            h::ComboBox::new(self.demo_text("cb-mt-input", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .menu_trigger(h::MenuTrigger::Input),
                            cx,
                        ),
                        spec(
                            "Manual (only the chevron opens it)",
                            h::ComboBox::new(self.demo_text("cb-mt-manual", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .menu_trigger(h::MenuTrigger::Manual),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Form Value", "Items are keyed `PickerItem`s: the selection is the item's key while the input shows its label, and v3's `formValue` decides what a named field submits. The default (`key`) submits the picked key -- save with a pick and the submitted value is `language=rust`, not `Rust` -- and `allowsCustomValue` forces the typed text.",
                    col(vec![
                        {
                            let combo = h::ComboBox::new(
                                self.demo_text("cb-form", "", cx),
                                language_items(),
                            )
                            .label("Language")
                            .placeholder("Select a language")
                            .name("language")
                            .is_required(true);
                            h::Form::new()
                                .field(combo.form_field().expect("named combo field"))
                                .on_submit(cx.listener(|this, data: &h::FormData, _, cx| {
                                    this.set_demo_text_value(
                                        "cb-form-submitted",
                                        data.text("language").unwrap_or_default().to_string(),
                                    );
                                    cx.notify();
                                }))
                                .child(combo)
                                .child(h::Button::new("cb-form-submit").label("Save"))
                                .into_any_element()
                        },
                        para(
                            &if self.demo_text_value("cb-form-submitted").is_empty() {
                                "Nothing submitted yet".to_owned()
                            } else {
                                format!(
                                    "Submitted: {}",
                                    self.demo_text_value("cb-form-submitted")
                                )
                            },
                            cx,
                        ),
                        spec(
                            "allowsCustomValue submits the text",
                            h::ComboBox::new(
                                self.demo_text("cb-form-custom", "", cx),
                                language_items(),
                            )
                            .label("Language")
                            .placeholder("Pick or type")
                            .name("custom-language")
                            .allows_custom_value(true)
                            .form_value(h::ComboBoxFormValue::Text),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Validation Behavior",
                    col(vec![
                        spec(
                            "Native (blocks the submit)",
                            h::ComboBox::new(self.demo_text("cb-vb-native", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .is_required(true)
                                .validation_behavior(h::ValidationBehavior::Native),
                            cx,
                        ),
                        spec(
                            "Allow (shows the message, submits anyway)",
                            h::ComboBox::new(self.demo_text("cb-vb-allow", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .is_required(true)
                                .validation_behavior(h::ValidationBehavior::Allow),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Validation",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-validate", "Zig", cx),
                        language_items(),
                    )
                    .label("Language")
                    .allows_custom_value(true)
                    .validate(|value| {
                        (!value.is_empty() && !languages().iter().any(|l| l == value))
                            .then(|| "Pick one of the listed languages".into())
                    })
                    .into_any_element()]),
                ),
                (
                    "Custom values allowed",
                    field_col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .allows_custom_value(true)
                    .is_open(is_open)
                    .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                        this.combo_open = *open;
                        cx.notify();
                    })))
                    .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_select(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.select_lang;
        let is_open = self.select_open;
        let sel_multi = self.select_multi.clone();
        component_doc_page!(
            "Select",
            crate::pages::Page::Select.description(),
            crate::pages::Page::Select.import_line(),
            vec![
                (
                    "Usage",
                    field_col(vec![h::Select::new("sel-main", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
                        .is_open(is_open)
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.select_open = *open;
                            cx.notify();
                        })))
                        .on_change(opt_usize_cb(cx.listener(
                            |this, i: &Option<usize>, _, cx| {
                                this.select_lang = *i;
                                this.select_open = false;
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
                (
                    "Virtualization", "v3 wraps the popover's list in React Aria's `Virtualizer`; `row_height` carries that here, and gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    col(vec![
                        demo_field(
                            h::Select::new("sel-virtual", virtual_names())
                                .label("User")
                            .placeholder("Choose one")
                            .row_height(px(40.)),
                        ),
                    ]),
                ),
                (
                    "With Description",
                    field_col(vec![h::Select::new("sel-desc", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .description("Used for spell-checking")
                        .into_any_element()]),
                ),
                (
                    "Required",
                    field_col(vec![h::Select::new("sel-required", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    field_col(vec![h::Select::new("sel-disabled", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    field_col(vec![h::Select::new("sel-disabled-opts", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .disabled_keys([1, 3])
                        .default_open(true)
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    field_col(vec![h::Select::new(
                        "sel-sections",
                        vec![
                            "United States".into(),
                            "Canada".into(),
                            "Mexico".into(),
                            "France".into(),
                            "Germany".into(),
                        ],
                    )
                    .label("Country")
                    .placeholder("Select a country")
                    .section_before(0, "North America")
                    .section_before(3, "Europe")
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Select::new("sel-surface", languages())
                                .label("Language")
                                .placeholder("Choose one")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled Multiple",
                    col(vec![
                        h::Select::new("sel-ctl-multi", languages())
                            .label("Languages")
                            .placeholder("Choose any")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_indices(sel_multi.iter().copied())
                            .on_selection_change_all(cx.listener(
                                |this, indices: &[usize], _, cx| {
                                    this.select_multi = indices.to_vec();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", sel_multi.len()), cx),
                    ]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("sel-open-btn")
                                .label(if is_open { "Close" } else { "Open" })
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.select_open = !this.select_open;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if is_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Select::new("sel-open", languages())
                            .label("Language")
                            .placeholder("Choose one")
                            .is_open(is_open)
                            .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                this.select_open = *open;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Loading", "v3 fills the list from a request and shows a spinner while it is in flight. The spinner is composed beside the label, since the options are the caller's own data.",
                    col(vec![
                        row(vec![
                            h::Select::new("sel-async", languages())
                                .label("Language")
                                .placeholder("Loading\u{2026}")
                                .into_any_element(),
                            h::Spinner::new("sel-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    field_col(vec![h::Select::new("sel-indicator", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
                        .on_selection_change(opt_usize_cb(cx.listener(
                            |this, i: &Option<usize>, _, cx| {
                                this.select_lang = *i;
                                cx.notify();
                            },
                        )))
                        .default_open(true)
                        .indicator(|is_selected| {
                            gpui::div()
                                .text_size(px(12.))
                                .child(if is_selected { "\u{2714}" } else { "" })
                                .into_any_element()
                        })
                        .into_any_element()]),
                ),
                (
                    "Custom Value",
                    field_col(vec![h::Select::new("sel-value", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
                        .on_selection_change(opt_usize_cb(cx.listener(
                            |this, i: &Option<usize>, _, cx| {
                                this.select_lang = *i;
                                cx.notify();
                            },
                        )))
                        .value_content(move |value| match value.selected_indices.first() {
                            Some(i) => gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    h::Chip::new()
                                        .size(Size::Sm)
                                        .variant(h::ChipVariant::Soft)
                                        .child(h::ChipLabel::new().child(format!("#{}", i + 1))),
                                )
                                .child(languages().get(*i).cloned().unwrap_or_default().to_string())
                                .into_any_element(),
                            // v3's own example hands `defaultChildren` back for
                            // the placeholder case rather than rebuilding it.
                            None => value.default_children,
                        })
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    field_col(vec![h::Select::new("sel-unc", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .default_value(Some(0))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    field_col(FieldVariant::ALL
                        .iter()
                        .map(|v| {
                            h::Select::new(el_id(format!("sel-{v:?}")), languages())
                                .label(v.label())
                                .placeholder("Choose one")
                                .value(selected)
                                .on_selection_change(opt_usize_cb(cx.listener(
                                    |this, i: &Option<usize>, _, cx| {
                                        this.select_lang = *i;
                                        cx.notify();
                                    },
                                )))
                                .variant(*v)
                        })
                        .els()),
                ),
                (
                    "Full width",
                    col(vec![gpui::div()
                        .w(px(400.))
                        .child(
                            h::Select::new("sel-full", languages())
                                .label("Language")
                                .placeholder("Choose one")
                                .value(selected)
                                .on_selection_change(opt_usize_cb(cx.listener(
                                    |this, i: &Option<usize>, _, cx| {
                                        this.select_lang = *i;
                                        cx.notify();
                                    },
                                )))
                                .full_width(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    field_col(vec![h::Select::new("sel-multi", languages())
                        .label("Languages")
                        .placeholder("Pick several")
                        .selection_mode(SelectionMode::Multiple)
                        .default_selected_indices([0, 2])
                        .default_open(true)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Typography
    // -----------------------------------------------------------------------

    pub fn page_kbd(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Kbd",
            crate::pages::Page::Kbd.description(),
            crate::pages::Page::Kbd.import_line(),
            vec![
                (
                    "Navigation Keys",
                    row(vec![
                        h::Kbd::new().child("\u{2190}").into_any_element(),
                        h::Kbd::new().child("\u{2192}").into_any_element(),
                        h::Kbd::new().child("\u{2191}").into_any_element(),
                        h::Kbd::new().child("\u{2193}").into_any_element(),
                        h::Kbd::new().child("Home").into_any_element(),
                        h::Kbd::new().child("End").into_any_element(),
                    ]),
                ),
                (
                    "Special Keys",
                    row(vec![
                        h::Kbd::new().child("\u{21e7}").into_any_element(),
                        h::Kbd::new().child("\u{2318}").into_any_element(),
                        h::Kbd::new().child("\u{2325}").into_any_element(),
                        h::Kbd::new().child("\u{21b5}").into_any_element(),
                        h::Kbd::new().child("\u{232b}").into_any_element(),
                        h::Kbd::new().child("Esc").into_any_element(),
                    ]),
                ),
                (
                    "Inline Usage",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(14.))
                        .child("Press")
                        .child(h::Kbd::new().child("Ctrl"))
                        .child(h::Kbd::new().child("K"))
                        .child("to open the command palette.")
                        .into_any_element()]),
                ),
                (
                    "Instructional Text",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(13.5))
                        .text_color(cx.colors().muted)
                        .child("Save with")
                        .child(
                            h::Kbd::new()
                                .variant(h::KbdVariant::Light)
                                .child("\u{2318}"),
                        )
                        .child(h::Kbd::new().variant(h::KbdVariant::Light).child("S"))
                        .into_any_element()]),
                ),
                (
                    "Usage",
                    row(vec![
                        h::Kbd::new().child("Ctrl").into_any_element(),
                        h::Kbd::new().child("Shift").into_any_element(),
                        h::Kbd::new().child("K").into_any_element(),
                    ]),
                ),
                (
                    "Variants",
                    row(h::KbdVariant::ALL
                        .iter()
                        .map(|v| spec(v.label(), h::Kbd::new().variant(*v).child("Esc"), cx))
                        .collect()),
                ),
            ],
            cx,
        )
    }

    pub fn page_typography(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let scale = [
            (h::TypographyType::H1, "h1", "36 / 600"),
            (h::TypographyType::H2, "h2", "30 / 600"),
            (h::TypographyType::H3, "h3", "24 / 600"),
            (h::TypographyType::H4, "h4", "20 / 600"),
            (h::TypographyType::H5, "h5", "18 / 600"),
            (h::TypographyType::H6, "h6", "16 / 600"),
            (h::TypographyType::Body, "body", "16 / 400"),
            (h::TypographyType::BodySm, "body-sm", "14 / 400"),
            (h::TypographyType::BodyXs, "body-xs", "12 / 400"),
            (h::TypographyType::Code, "code", "14 / mono"),
        ];
        let muted = cx.colors().muted;
        component_doc_page!(
            "Typography",
            crate::pages::Page::Typography.description(),
            crate::pages::Page::Typography.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Typography::new(
                        "Typography sets the size, weight and line height of a run of text.",
                    )
                    .into_any_element()]),
                ),
                (
                    "Render Props", "`Prose` provides only the `text-foreground` color. GPUI has no per-tag CSS selectors, so the per-tag descendant styles in v3's `.typography-prose` — `h1`–`h6`, `p`, `code`, `a`, lists — cannot be inherited; children must already be semantic elements.",
                    col(vec![
                        h::Prose::new()
                            .child(h::Typography::paragraph(
                                h::ParagraphSize::Base,
                                "A paragraph inside `Prose` carries its own body metrics.",
                            ))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Scale",
                    col(scale
                        .iter()
                        .map(|(kind, name, meta)| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(24.))
                                .child(
                                    gpui::div()
                                        .w(px(110.))
                                        .flex()
                                        .flex_col()
                                        .child(
                                            gpui::div()
                                                .text_size(px(13.))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(name.to_string()),
                                        )
                                        .child(
                                            gpui::div()
                                                .text_size(px(11.))
                                                .text_color(muted)
                                                .child(meta.to_string()),
                                        ),
                                )
                                .child(
                                    h::Typography::new("Build better interfaces").kind(*kind),
                                )
                        })
                        .els()),
                ),
                (
                    "Colors & weights",
                    col(vec![
                        h::Typography::new("Default foreground").into_any_element(),
                        h::Typography::new("Muted foreground")
                            .color(h::TextColor::Muted)
                            .into_any_element(),
                        h::Typography::new("Bold body")
                            .weight(h::FontWeight::Bold)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Alignment & truncation",
                    col(vec![
                        h::Typography::new("Centered against the measure")
                            .align(h::TextAlign::Center)
                            .into_any_element(),
                        h::Typography::new("Aligned to the end")
                            .align(h::TextAlign::End)
                            .into_any_element(),
                        h::Typography::new(
                            "Justify falls back to start alignment: gpui has no text-justify.",
                        )
                        .align(h::TextAlign::Justify)
                        .into_any_element(),
                        gpui::div()
                            .w(px(220.))
                            .child(
                                h::Typography::new(
                                    "A long line that truncates to a single ellipsis at the \
                                     edge of its 220px box instead of wrapping.",
                                )
                                .truncate(true),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Primitives",
                    col(vec![
                        h::Typography::heading(3, "Dashboard").into_any_element(),
                        h::Typography::paragraph(
                            h::ParagraphSize::Base,
                            "Paragraph supports base, sm and xs sizes.",
                        )
                        .into_any_element(),
                        h::Typography::code("cargo add herogpui").into_any_element(),
                    ]),
                ),
                (
                    "Prose",
                    col(vec![h::Prose::new()
                        .child(h::Typography::heading(4, "Body heading"))
                        .child(h::Typography::paragraph(
                            h::ParagraphSize::Base,
                            "Prose applies HeroUI's typographic rhythm to already-semantic children.",
                        ))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Utilities
    // -----------------------------------------------------------------------

    pub fn page_scroll_shadow(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let lines: Vec<AnyElement> = (1..=20)
            .map(|i| {
                gpui::div()
                    .py(px(4.))
                    .child(format!("Row {i}"))
                    .into_any_element()
            })
            .collect();
        component_doc_page!(
            "Scroll Shadow",
            crate::pages::Page::ScrollShadow.description(),
            crate::pages::Page::ScrollShadow.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::ScrollShadow::new("ss-usage")
                        .max_h(px(180.))
                        .children((1..=14).map(|n| {
                            gpui::div().py(px(6.)).child(format!("Row {n} of fourteen"))
                        }),)
                        .into_any_element()]),
                ),
                (
                    "Orientation",
                    col(vec![
                        spec(
                            "Vertical",
                            h::ScrollShadow::new("ss-or-v").max_h(px(140.)).children(
                                (1..=10).map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                            ),
                            cx,
                        ),
                        spec(
                            "Horizontal",
                            h::ScrollShadow::new("ss-or-h")
                                .orientation(Orientation::Horizontal)
                                .max_w(px(360.))
                                .gap(px(12.))
                                .children((1..=12).map(|n| {
                                    gpui::div()
                                        .flex_shrink_0()
                                        .w(px(90.))
                                        .h(px(60.))
                                        .rounded(px(10.))
                                        .bg(cx.colors().default.color)
                                        .child(format!("{n}"))
                                })),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Shadow Size",
                    col(vec![
                        spec(
                            "8px",
                            h::ScrollShadow::new("ss-size-sm")
                                .size(px(8.))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "40px",
                            h::ScrollShadow::new("ss-size-lg")
                                .size(px(40.))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "With Card",
                    col(vec![h::Card::new()
                        .w(px(320.))
                        .child(
                            h::CardHeader::new().child(h::CardTitle::new().child("Release notes"))
                        )
                        .child(h::CardContent::new().child(
                            h::ScrollShadow::new("ss-card").max_h(px(140.)).children(
                                (1..=12).map(|n| {
                                    gpui::div().py(px(6.)).child(format!("Change {n}"))
                                }),
                            ),
                        ),)
                        .into_any_element()]),
                ),
                (
                    "Hide Scroll Bar", "gpui draws no scrollbar inside a scroll container, so this is the default rather than a prop: the shadows are the only affordance.",
                    col(vec![
                        h::ScrollShadow::new("ss-no-bar")
                            .max_h(px(140.))
                            .children(
                                (1..=12).map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Visibility Change",
                    col(vec![
                        spec(
                            "Auto (follows the scroll position)",
                            h::ScrollShadow::new("ss-vis-auto")
                                .visibility(h::ScrollShadowVisibility::Auto)
                                // `onVisibilityChange` fires when the shaded
                                // edges change, which `Auto` does as it scrolls.
                                .on_visibility_change(shadow_vis_cb(cx.listener(
                                    |this, visibility: &h::ScrollShadowVisibility, _, cx| {
                                        this.set_demo_text_value(
                                            "ss-visibility",
                                            format!("{visibility:?}"),
                                        );
                                        cx.notify();
                                    },
                                )))
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "Both edges, always",
                            h::ScrollShadow::new("ss-vis-both")
                                .visibility(h::ScrollShadowVisibility::Both)
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                        spec(
                            "Top only",
                            h::ScrollShadow::new("ss-vis-top")
                                .visibility(h::ScrollShadowVisibility::Top)
                                .max_h(px(120.))
                                .children(
                                    (1..=10)
                                        .map(|n| gpui::div().py(px(6.)).child(format!("Row {n}"))),
                                ),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Vertical",
                    col(vec![h::ScrollShadow::new("ss-main")
                        .max_h(px(200.))
                        .children(lines)
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    col(vec![h::ScrollShadow::new("ss-h")
                        .orientation(Orientation::Horizontal)
                        .max_w(px(520.))
                        .size(px(56.))
                        .children((1..=14).map(|i| {
                            gpui::div()
                                .flex_shrink_0()
                                .px(px(16.))
                                .py(px(10.))
                                .rounded(px(10.))
                                .bg(cx.colors().surface_secondary)
                                .child(format!("Card {i}"))
                                .into_any_element()
                        }))
                        .into_any_element()]),
                ),
                (
                    "Shadows disabled",
                    col(vec![h::ScrollShadow::new("ss-off")
                        .max_h(px(140.))
                        .is_enabled(false)
                        .children((1..=10).map(|i| {
                            gpui::div()
                                .py(px(4.))
                                .child(format!("Row {i}"))
                                .into_any_element()
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }
}

// ---------------------------------------------------------------------------
// small shared bits
// ---------------------------------------------------------------------------

/// gpui svgs never inherit `text_color`, so demo icons set it explicitly.
fn icon(path: &'static str, cx: &gpui::App) -> AnyElement {
    gpui::svg()
        .size(px(16.))
        .path(path)
        .text_color(cx.colors().foreground)
        .into_any_element()
}

/// One stable custom image source for the Avatar "Custom Image Component"
/// demo: the loader `Arc` is built once, so the avatar's per-source keyed
/// state and its `on_load` keep their identity across frames.
fn sample_avatar_source() -> gpui::ImageSource {
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
            include_bytes!("../../assets/herogpui/sample.png").to_vec(),
        ));
        std::sync::Arc::new(move |window: &mut gpui::Window, cx: &mut gpui::App| {
            image.clone().use_render_image(window, cx).map(Ok)
        })
    });
    gpui::ImageSource::Custom(loader.clone())
}

/// A neutral block used as the child of badge demos.
fn avatar_box(cx: &gpui::App) -> AnyElement {
    gpui::div()
        .size(px(36.))
        .rounded(px(10.))
        .bg(cx.colors().surface_tertiary)
        .into_any_element()
}

fn languages() -> Vec<SharedString> {
    vec![
        "Rust".into(),
        "TypeScript".into(),
        "Python".into(),
        "Go".into(),
        "Swift".into(),
        "Kotlin".into(),
    ]
}

fn toggle_key(set: &mut HashSet<SharedString>, key: &SharedString) {
    if !set.remove(key) {
        set.insert(key.clone());
    }
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod example_quality {
    use super::*;

    const SRC: &str = include_str!("components.rs");

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
            ("select", 17),
            ("autocomplete", 18),
            ("combo_box", 25),
            ("slider", 11),
            ("date_field", 14),
            ("alert_dialog", 12),
            ("dropdown", 17),
            ("popover", 6),
            ("number_field", 16),
            ("text_area", 6),
            ("date_range_picker", 8),
            ("list_box", 11),
            ("tag_group", 13),
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
            "Range (multi-thumb)",
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
        assert!(select.contains("default_value(Some(0))"));

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
        assert!(select.contains("h::Select::new(\"sel-main\", languages())"));
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
    fn spec_rows_are_top_aligned_and_captions_are_compact() {
        let row = SRC
            .split("fn row(")
            .nth(1)
            .and_then(|rest| rest.split("fn spec_row").next())
            .expect("row helper");
        assert!(row.contains(".items_start()"));
        assert!(!row.contains(".items_center()"));
        assert!(SRC.contains("fn spec_row"));

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
        assert!(popover.contains(".on_open_change(bool_cb(cx.listener"));
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
                section
                    .matches(".on_open_change(bool_cb(cx.listener")
                    .count(),
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
}
