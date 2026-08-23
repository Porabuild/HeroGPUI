//! Component gallery pages — one page per HeroUI v3 component.

use std::collections::HashSet;

use gpui::{prelude::*, px, AnyElement, Context, SharedString};
use herogpui_components as h;
use herogpui_core::{Color, FieldVariant, Orientation, SelectionMode, Size, SizeXl, Variant};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::{doc_page, para};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn row(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_wrap()
        .w_full()
        .items_center()
        .gap(px(12.))
        .children(children)
        .into_any_element()
}

fn col(children: Vec<AnyElement>) -> AnyElement {
    gpui::div()
        .flex()
        .flex_col()
        // Components hug their content in a demo; `full_width` examples opt back
        // in explicitly.
        .items_start()
        .gap(px(12.))
        .children(children)
        .into_any_element()
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
        doc_page(
            "Button",
            crate::pages::Page::Button.description(),
            crate::pages::Page::Button.import_line(),
            vec![
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
                            .label("Search")
                            .start_content(icon(h::icons::SEARCH, cx))
                            .into_any_element(),
                        h::Button::new("btn-i-2")
                            .label("Add member")
                            .variant(Variant::Secondary)
                            .start_content(icon(h::icons::PLUS, cx))
                            .into_any_element(),
                        h::Button::new("btn-i-3")
                            .label("Delete")
                            .variant(Variant::Danger)
                            .start_content(icon(h::icons::CLOSE, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Icon only",
                    row(vec![
                        h::Button::new("btn-io-1")
                            .is_icon_only(true)
                            .variant(Variant::Tertiary)
                            .start_content(icon(h::icons::ELLIPSIS, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-2")
                            .is_icon_only(true)
                            .variant(Variant::Secondary)
                            .start_content(icon(h::icons::PLUS, cx))
                            .into_any_element(),
                        h::Button::new("btn-io-3")
                            .is_icon_only(true)
                            .variant(Variant::Danger)
                            .start_content(icon(h::icons::CLOSE, cx))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Pending & disabled",
                    row(vec![
                        h::Button::new("btn-pending")
                            .label("Uploading")
                            .is_pending(true)
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
        doc_page(
            "Button Group",
            crate::pages::Page::ButtonGroup.description(),
            crate::pages::Page::ButtonGroup.import_line(),
            vec![
                (
                    "Merged",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        .button(h::Button::new("bg-1").label("Day"))
                        .button(h::Button::new("bg-2").label("Week"))
                        .button(h::Button::new("bg-3").label("Month"))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(Variant::GROUP
                        .iter()
                        .map(|v| {
                            h::ButtonGroup::new()
                                .variant(*v)
                                .button(h::Button::new(el_id(format!("bgv-{v:?}-1"))).label("One"))
                                .button(h::Button::new(el_id(format!("bgv-{v:?}-2"))).label("Two"))
                        })
                        .els()),
                ),
                (
                    "Vertical",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        .orientation(Orientation::Vertical)
                        .button(h::Button::new("bgv-top").label("Top"))
                        .button(h::Button::new("bgv-mid").label("Middle"))
                        .button(h::Button::new("bgv-bot").label("Bottom"))
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    row(vec![h::ButtonGroup::new()
                        .variant(Variant::Secondary)
                        .is_disabled(true)
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
        doc_page(
            "Close Button",
            crate::pages::Page::CloseButton.description(),
            crate::pages::Page::CloseButton.import_line(),
            vec![
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
        let single = self.toggle_single.clone();
        let multiple = self.toggle_multiple.clone();
        doc_page(
            "Toggle Button",
            crate::pages::Page::ToggleButton.description(),
            crate::pages::Page::ToggleButton.import_line(),
            vec![
                (
                    "Single selection",
                    row(vec![h::ToggleButtonGroup::new()
                        .selection_mode(SelectionMode::Single)
                        .selected_keys(single.into_iter().collect::<Vec<_>>())
                        .child_toggle(h::ToggleButton::new("tb-left").key("left").label("Left"))
                        .child_toggle(
                            h::ToggleButton::new("tb-center")
                                .key("center")
                                .label("Center"),
                        )
                        .child_toggle(h::ToggleButton::new("tb-right").key("right").label("Right"))
                        .on_change(cx.listener(|this, keys: &[SharedString], _, cx| {
                            this.toggle_single = keys.first().cloned();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    row(vec![h::ToggleButtonGroup::new()
                        .selection_mode(SelectionMode::Multiple)
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
                            .is_selected(true)
                            .into_any_element(),
                        h::ToggleButton::new("tb-v-ghost")
                            .label("Ghost")
                            .variant(h::ToggleVariant::Ghost)
                            .is_selected(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical & detached",
                    row(vec![
                        h::ToggleButtonGroup::new()
                            .orientation(Orientation::Vertical)
                            .child_toggle(h::ToggleButton::new("tbv-1").label("Top"))
                            .child_toggle(h::ToggleButton::new("tbv-2").label("Bottom"))
                            .into_any_element(),
                        h::ToggleButtonGroup::new()
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
        doc_page(
            "Dropdown",
            crate::pages::Page::Dropdown.description(),
            crate::pages::Page::Dropdown.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Dropdown::new(
                            h::Button::new("dd-trigger")
                                .label("Actions")
                                .variant(Variant::Secondary),
                            items,
                            is_open,
                        )
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
                    "Multiple selection",
                    col(vec![
                        h::Dropdown::new(
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
        doc_page(
            "List Box",
            crate::pages::Page::ListBox.description(),
            crate::pages::Page::ListBox.import_line(),
            vec![
                (
                    "Single selection",
                    col(vec![gpui::div()
                        .w(px(280.))
                        .child(
                            h::ListBox::new("lb-single", items.clone())
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
        let tags: Vec<h::Tag> = self
            .tags
            .iter()
            .map(|k| h::Tag::new(k.clone(), title_case(k)))
            .collect();
        let selection = self.tag_selection.clone();
        doc_page(
            "Tag Group",
            crate::pages::Page::TagGroup.description(),
            crate::pages::Page::TagGroup.import_line(),
            vec![
                (
                    "Removable",
                    col(vec![h::TagGroup::new("tg-remove", tags.clone())
                        .label("Team")
                        .description("Remove a tag to see the group update.")
                        .empty_state("All tags removed")
                        .on_remove(cx.listener(|this, key: &SharedString, _, cx| {
                            this.tags.retain(|k| k != key);
                            this.tag_selection.remove(key);
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Selectable",
                    col(vec![h::TagGroup::new("tg-select", tags.clone())
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
                        h::TagGroup::new("tg-default", tags.clone()).into_any_element(),
                        h::TagGroup::new("tg-surface", tags.clone())
                            .variant(h::TagVariant::Surface)
                            .into_any_element(),
                        h::TagGroup::new("tg-sm", tags.clone())
                            .size(Size::Sm)
                            .into_any_element(),
                        h::TagGroup::new("tg-lg", tags)
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
        doc_page(
            "Color Area",
            crate::pages::Page::ColorArea.description(),
            crate::pages::Page::ColorArea.import_line(),
            vec![
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
        doc_page(
            "Color Field",
            crate::pages::Page::ColorField.description(),
            crate::pages::Page::ColorField.import_line(),
            vec![
                (
                    "Hex value",
                    col(vec![h::ColorField::new("cf-hex", value)
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
                    col(vec![h::ColorField::new("cf-display", value)
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
                (
                    "Variants",
                    row(vec![
                        h::ColorField::new("cf-primary", value)
                            .label("Primary")
                            .into_any_element(),
                        h::ColorField::new("cf-secondary", value)
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_color_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.picker_color;
        let is_open = self.color_picker_open;
        doc_page(
            "Color Picker",
            crate::pages::Page::ColorPicker.description(),
            crate::pages::Page::ColorPicker.import_line(),
            vec![(
                "Usage",
                col(vec![h::ColorPicker::new("cp-main", value)
                    .label("Accent")
                    .is_open(is_open)
                    .show_alpha(true)
                    .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                        this.color_picker_open = *open;
                        cx.notify();
                    })))
                    .on_change(color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                        this.picker_color = *c;
                        cx.notify();
                    })))
                    .into_any_element()]),
            )],
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
        doc_page(
            "Color Slider",
            crate::pages::Page::ColorSlider.description(),
            crate::pages::Page::ColorSlider.import_line(),
            vec![
                (
                    "Channels",
                    col(channels
                        .iter()
                        .map(|ch| {
                            h::ColorSlider::new(el_id(format!("cs-{ch:?}")), value, *ch).on_change(
                                color_cb(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                    this.picker_color = *c;
                                    cx.notify();
                                })),
                            )
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
        doc_page(
            "Color Swatch",
            crate::pages::Page::ColorSwatch.description(),
            crate::pages::Page::ColorSwatch.import_line(),
            vec![
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
        doc_page(
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
        let value = self.slider_value;
        doc_page(
            "Slider",
            crate::pages::Page::Slider.description(),
            crate::pages::Page::Slider.import_line(),
            vec![
                (
                    "Format options",
                    col(vec![h::Slider::new("sl-fmt", value)
                        .label("Budget")
                        .show_value(true)
                        .format_options(h::NumberFormat::currency("EUR"))
                        .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                            this.slider_value = *v;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Usage",
                    col(vec![h::Slider::new("sl-main", value)
                        .label("Volume")
                        .show_value(true)
                        .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                            this.slider_value = *v;
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Range (multi-thumb)",
                    col(vec![h::Slider::new("sl-range", value)
                        .label("Price range")
                        .values(self.slider_range.clone())
                        .on_change_all(cx.listener(|this, vs: &[f32], _, cx| {
                            this.slider_range = vs.to_vec();
                            cx.notify();
                        }))
                        .into_any_element()]),
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
                        h::Slider::new("sl-step", value)
                            .step(10.0)
                            .label("Step 10")
                            .show_value(true)
                            .on_change(f32_cb(cx.listener(|this, v: &f32, _, cx| {
                                this.slider_value = *v;
                                cx.notify();
                            })))
                            .into_any_element(),
                        h::Slider::new("sl-disabled", value)
                            .is_disabled(true)
                            .label("Disabled")
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_switch(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let (a, b) = (self.switch_a, self.switch_b);
        doc_page(
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
                                    .is_selected(true)
                                    .size(*s),
                                cx,
                            )
                        })
                        .collect()),
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
        doc_page(
            "Badge",
            crate::pages::Page::Badge.description(),
            crate::pages::Page::Badge.import_line(),
            vec![
                (
                    "Variants",
                    row(h::BadgeVariant::ALL
                        .iter()
                        .map(|v| {
                            spec(
                                v.label(),
                                h::Badge::new()
                                    .content("5")
                                    .color(Color::Accent)
                                    .variant(*v)
                                    .child(avatar_box(cx)),
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
                                h::Badge::new().content("5").color(*c).child(avatar_box(cx)),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Dot & placement",
                    row(vec![
                        // No content is v3's dot badge.
                        h::Badge::new()
                            .color(Color::Success)
                            .child(avatar_box(cx))
                            .into_any_element(),
                        h::Badge::new()
                            .content("9")
                            .placement(h::BadgePlacement::BottomRight)
                            .child(avatar_box(cx))
                            .into_any_element(),
                        h::Badge::new()
                            .content("New")
                            .placement(h::BadgePlacement::TopLeft)
                            .child(avatar_box(cx))
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_chip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Chip",
            crate::pages::Page::Chip.description(),
            crate::pages::Page::Chip.import_line(),
            vec![
                (
                    "Variants",
                    row(h::ChipVariant::ALL
                        .iter()
                        .map(|v| h::Chip::new(v.label()).variant(*v).color(Color::Accent))
                        .els()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| h::Chip::new(c.label()).color(*c))
                        .els()),
                ),
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| h::Chip::new(s.label()).size(*s))
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_table(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let build = || {
            h::Table::new(vec!["Name".into(), "Role".into(), "Status".into()])
                .row(vec![
                    gpui::div().child("Tony Reichert").into_any_element(),
                    gpui::div().child("CEO").into_any_element(),
                    h::Chip::new("Active")
                        .color(Color::Success)
                        .size(Size::Sm)
                        .into_any_element(),
                ])
                .row(vec![
                    gpui::div().child("Zoey Lang").into_any_element(),
                    gpui::div().child("Tech Lead").into_any_element(),
                    h::Chip::new("Paused")
                        .color(Color::Warning)
                        .size(Size::Sm)
                        .into_any_element(),
                ])
                .row(vec![
                    gpui::div().child("Jane Fisher").into_any_element(),
                    gpui::div().child("Designer").into_any_element(),
                    h::Chip::new("Vacation")
                        .color(Color::Danger)
                        .size(Size::Sm)
                        .into_any_element(),
                ])
        };
        doc_page(
            "Table",
            crate::pages::Page::Table.description(),
            crate::pages::Page::Table.import_line(),
            vec![
                ("Usage", col(vec![build().into_any_element()])),
                (
                    "Variants",
                    col(h::TableVariant::ALL
                        .iter()
                        .map(|v| build().variant(*v))
                        .els()),
                ),
                (
                    "Custom sort indicator",
                    // Sorted on load, so the custom indicator is actually
                    // visible: `indicator` only renders for the sorted column.
                    col(vec![h::Table::new(vec![])
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
                        build()
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
                    "Empty and loading",
                    col(vec![
                        h::Table::new(vec!["Name".into(), "Role".into()])
                            .empty_state("Nobody here yet")
                            .into_any_element(),
                        build().is_pending(true).into_any_element(),
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
        doc_page(
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
                    "Constraints",
                    col(vec![
                        h::Calendar::new(self.calendar.clone())
                            .min_value(h::Date::new(today.year, today.month, 5))
                            .max_value(h::Date::new(today.year, today.month, 24))
                            .is_date_unavailable(|d: h::Date| d.day.is_multiple_of(7))
                            .into_any_element(),
                        para(
                            "minValue/maxValue mute the days outside the range;                              isDateUnavailable strikes through the ones it rejects.",
                            cx,
                        ),
                    ]),
                ),
                (
                    "First day of week",
                    col(vec![h::Calendar::new(self.calendar.clone())
                        .first_day_of_week(h::Weekday::Sun)
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
            ],
            cx,
        )
    }

    pub fn page_date_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let iso = self.date_iso;
        doc_page(
            "Date Field",
            crate::pages::Page::DateField.description(),
            crate::pages::Page::DateField.import_line(),
            vec![(
                "Usage",
                col(vec![
                    h::DateField::new(self.date_input.clone())
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
                            None => "Type digits, or step a segment with the arrow keys".to_owned(),
                        },
                        cx,
                    ),
                ]),
            )],
            cx,
        )
    }

    pub fn page_date_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.date_picker_open;
        doc_page(
            "Date Picker",
            crate::pages::Page::DatePicker.description(),
            crate::pages::Page::DatePicker.import_line(),
            vec![(
                "Usage",
                col(vec![h::DatePicker::new(self.calendar.clone())
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
            )],
            cx,
        )
    }

    pub fn page_date_range_picker(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.range_open;
        doc_page(
            "Date Range Picker",
            crate::pages::Page::DateRangePicker.description(),
            crate::pages::Page::DateRangePicker.import_line(),
            vec![(
                "Usage",
                col(vec![h::DateRangePicker::new(self.date_range.clone())
                    .label("Trip dates")
                    .is_open(is_open)
                    .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                        this.range_open = *open;
                        cx.notify();
                    })))
                    .on_change(|_, _cx| {})
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_range_calendar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Range Calendar",
            crate::pages::Page::RangeCalendar.description(),
            crate::pages::Page::RangeCalendar.import_line(),
            vec![(
                "Usage",
                col(vec![h::RangeCalendar::new(self.date_range.clone())
                    .on_change(|_start, _end, _, _cx| {})
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_time_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Time Field",
            crate::pages::Page::TimeField.description(),
            crate::pages::Page::TimeField.import_line(),
            vec![
                (
                    "24-hour",
                    col(vec![h::TimeField::new(self.time.clone())
                        .label("Start time")
                        .description("Click a segment, then use the steppers.")
                        .on_change(opt_time_cb(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        ))
                        .into_any_element()]),
                ),
                (
                    "12-hour with seconds",
                    col(vec![h::TimeField::new(self.time.clone())
                        .label("Reminder")
                        .hour_cycle(h::HourCycle::H12)
                        .show_seconds(true)
                        .on_change(opt_time_cb(
                            cx.listener(|_, _t: &Option<h::Time>, _, cx| cx.notify()),
                        ))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Feedback
    // -----------------------------------------------------------------------

    pub fn page_alert(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Alert",
            crate::pages::Page::Alert.description(),
            crate::pages::Page::Alert.import_line(),
            vec![
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
                    "Closable",
                    col(if self.alert_visible {
                        vec![h::Alert::new("Saved")
                            .description("Your changes are live.")
                            .status(Color::Success)
                            .is_closable(cx.listener(|this, _, _, cx| {
                                this.alert_visible = false;
                                cx.notify();
                            }))
                            .into_any_element()]
                    } else {
                        vec![h::Button::new("alert-restore")
                            .label("Bring it back")
                            .variant(Variant::Tertiary)
                            .size(Size::Sm)
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.alert_visible = true;
                                cx.notify();
                            }))
                            .into_any_element()]
                    }),
                ),
            ],
            cx,
        )
    }

    pub fn page_meter(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.meter_value;
        doc_page(
            "Meter",
            crate::pages::Page::Meter.description(),
            crate::pages::Page::Meter.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Meter::new(value)
                        .label("Disk usage")
                        .show_value(true)
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .map(|c| h::Meter::new(value).color(*c))
                        .els()),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .map(|s| h::Meter::new(value).size(*s))
                        .els()),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_bar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Progress Bar",
            crate::pages::Page::ProgressBar.description(),
            crate::pages::Page::ProgressBar.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::ProgressBar::new()
                        .value(65.0)
                        .label("Uploading")
                        .show_value_label(true)
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .map(|c| h::ProgressBar::new().value(65.0).color(*c))
                        .els()),
                ),
                (
                    "Sizes",
                    col(vec![
                        h::ProgressBar::new()
                            .value(40.0)
                            .size(Size::Sm)
                            .into_any_element(),
                        h::ProgressBar::new()
                            .value(60.0)
                            .size(Size::Md)
                            .into_any_element(),
                        h::ProgressBar::new()
                            .value(80.0)
                            .size(Size::Lg)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_circle(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Progress Circle",
            crate::pages::Page::ProgressCircle.description(),
            crate::pages::Page::ProgressCircle.import_line(),
            vec![
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
                            .show_value_label(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_skeleton(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Skeleton",
            crate::pages::Page::Skeleton.description(),
            crate::pages::Page::Skeleton.import_line(),
            vec![(
                "Loading",
                col(vec![
                    h::Skeleton::new().w(px(320.)).h(px(16.)).into_any_element(),
                    h::Skeleton::new().w(px(260.)).h(px(16.)).into_any_element(),
                    h::Skeleton::new().w(px(180.)).h(px(16.)).into_any_element(),
                ]),
            )],
            cx,
        )
    }

    pub fn page_spinner(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Spinner",
            crate::pages::Page::Spinner.description(),
            crate::pages::Page::Spinner.import_line(),
            vec![
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
        doc_page(
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
        doc_page(
            "Checkbox Group",
            crate::pages::Page::CheckboxGroup.description(),
            crate::pages::Page::CheckboxGroup.import_line(),
            vec![
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
        doc_page(
            "Fieldset",
            crate::pages::Page::Fieldset.description(),
            crate::pages::Page::Fieldset.import_line(),
            vec![(
                "Usage",
                col(vec![h::Fieldset::new()
                    .child(h::FieldsetLegend::new("Shipping address"))
                    .child(
                        h::FieldsetGroup::new()
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
            )],
            cx,
        )
    }

    pub fn page_field_slots(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Label & Messages",
            crate::pages::Page::FieldSlots.description(),
            crate::pages::Page::FieldSlots.import_line(),
            vec![
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
                    "FieldError",
                    col(vec![
                        h::FieldError::new()
                            .message("This field is required.")
                            .into_any_element(),
                        para("A FieldError with no message renders nothing.", cx),
                        h::FieldError::new().into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_form(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let submitted = self.input_submitted.clone();
        doc_page(
            "Form",
            crate::pages::Page::Form.description(),
            crate::pages::Page::Form.import_line(),
            vec![(
                "Usage",
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
                                .is_required(true),
                        )
                        .child(
                            h::TextField::new(self.input_email.clone())
                                .name("email")
                                .label("Email")
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
            )],
            cx,
        )
    }

    pub fn page_input(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Input",
            crate::pages::Page::Input.description(),
            crate::pages::Page::Input.import_line(),
            vec![
                (
                    "Variants",
                    col(FieldVariant::ALL
                        .iter()
                        .map(|v| {
                            h::Input::new(self.input_name.clone())
                                .label(v.label())
                                .placeholder("Type here")
                                .variant(*v)
                        })
                        .els()),
                ),
                (
                    "States",
                    col(vec![
                        h::Input::new(self.input_name.clone())
                            .label("Required")
                            .is_required(true)
                            .into_any_element(),
                        h::Input::new(self.input_name.clone())
                            .label("Invalid")
                            .error_message("That name is taken.")
                            .into_any_element(),
                        h::Input::new(self.input_name.clone())
                            .label("Disabled")
                            .is_disabled(true)
                            .into_any_element(),
                        h::Input::new(self.input_name.clone())
                            .label("Clearable")
                            .is_clearable(true)
                            .into_any_element(),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_input_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Input Group",
            crate::pages::Page::InputGroup.description(),
            crate::pages::Page::InputGroup.import_line(),
            vec![
                (
                    "Addons",
                    col(vec![h::InputGroup::new()
                        .label("Amount")
                        .description("Charged monthly.")
                        .prefix(h::InputAddon::new("$"))
                        .input(h::Input::new(self.group_amount.clone()).placeholder("0.00"))
                        .suffix(h::InputAddon::new("USD"))
                        .into_any_element()]),
                ),
                (
                    "With a trailing action",
                    col(vec![h::InputGroup::new()
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
        doc_page(
            "Input OTP",
            crate::pages::Page::InputOtp.description(),
            crate::pages::Page::InputOtp.import_line(),
            vec![(
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
            )],
            cx,
        )
    }

    pub fn page_number_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Number Field",
            crate::pages::Page::NumberField.description(),
            crate::pages::Page::NumberField.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::NumberField::new(self.number.clone())
                        .label("Quantity")
                        .on_change(f64_cb(cx.listener(|_, _v: &f64, _, cx| cx.notify())))
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
            ],
            cx,
        )
    }

    pub fn page_radio_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.radio_sel;
        let options: Vec<SharedString> = vec!["Free".into(), "Pro".into(), "Enterprise".into()];
        doc_page(
            "Radio Group",
            crate::pages::Page::RadioGroup.description(),
            crate::pages::Page::RadioGroup.import_line(),
            vec![
                (
                    "Vertical",
                    col(vec![h::RadioGroup::new("rg-v", options.clone())
                        .value(selected)
                        .on_change(usize_cb(cx.listener(|this, i: &usize, _, cx| {
                            this.radio_sel = Some(*i);
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    col(vec![h::RadioGroup::new("rg-unc", options.clone())
                        .default_value(Some(1))
                        .into_any_element()]),
                ),
                (
                    "Horizontal",
                    col(vec![h::RadioGroup::new("rg-h", options.clone())
                        .value(selected)
                        .orientation(Orientation::Horizontal)
                        .on_change(usize_cb(cx.listener(|this, i: &usize, _, cx| {
                            this.radio_sel = Some(*i);
                            cx.notify();
                        })))
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::RadioGroup::new("rg-d", options)
                        .value(selected)
                        .is_disabled(true)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_search_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let query = self.search_query.clone();
        doc_page(
            "Search Field",
            crate::pages::Page::SearchField.description(),
            crate::pages::Page::SearchField.import_line(),
            vec![(
                "Usage",
                col(vec![
                    h::SearchField::new(self.search_state.clone())
                        .label("Search docs")
                        .placeholder("Search components")
                        .on_change(cx.listener(|this, text: &str, _, cx| {
                            this.search_query = text.to_owned();
                            cx.notify();
                        }))
                        .into_any_element(),
                    para(
                        &if query.is_empty() {
                            "Type to search".to_owned()
                        } else {
                            format!("Query: {query}")
                        },
                        cx,
                    ),
                ]),
            )],
            cx,
        )
    }

    pub fn page_text_area(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Text Area",
            crate::pages::Page::TextArea.description(),
            crate::pages::Page::TextArea.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::TextArea::new(self.input_bio.clone())
                        .label("Bio")
                        .placeholder("Tell us about yourself")
                        .description("Markdown is supported.")
                        .into_any_element()]),
                ),
                (
                    "Rows",
                    col(vec![h::TextArea::new(self.input_bio.clone())
                        .label("Six rows")
                        .rows(6)
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_text_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Text Field",
            crate::pages::Page::TextField.description(),
            crate::pages::Page::TextField.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::TextField::new(self.text_field_state.clone())
                        .label("Full name")
                        .placeholder("Ada Lovelace")
                        .description("As it appears on your ID.")
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    col(vec![h::TextField::new(self.text_field_state.clone())
                        .label("Full name")
                        .is_required(true)
                        .error_message("This field is required.")
                        .into_any_element()]),
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
                .child(h::CardHeader::new().child("Daily report"))
                .child(h::CardBody::new().child("Sessions are up 12% week over week."))
                .child(
                    h::CardFooter::new().child(
                        h::Button::new(el_id(format!("card-{variant:?}-cta")))
                            .label("View")
                            .size(Size::Sm)
                            .variant(Variant::Tertiary),
                    ),
                )
        };
        doc_page(
            "Card",
            crate::pages::Page::Card.description(),
            crate::pages::Page::Card.import_line(),
            vec![(
                "Variants",
                row(h::CardVariant::ALL.iter().map(|v| card(*v)).els()),
            )],
            cx,
        )
    }

    pub fn page_separator(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Separator",
            crate::pages::Page::Separator.description(),
            crate::pages::Page::Separator.import_line(),
            vec![
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
        doc_page(
            "Surface",
            crate::pages::Page::Surface.description(),
            crate::pages::Page::Surface.import_line(),
            vec![
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
                    col(vec![h::Surface::new()
                        .child(
                            h::Input::new(self.input_name.clone())
                                .placeholder("Secondary input")
                                .variant(FieldVariant::Secondary),
                        )
                        .child(
                            h::TextArea::new(self.input_bio.clone())
                                .placeholder("Secondary text area"),
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
                .is_attached(attached)
                .orientation(orientation)
                .child(
                    h::ToggleButtonGroup::new()
                        .selection_mode(SelectionMode::Multiple)
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
                        .button(h::Button::new(el_id(format!("tbar-{key}-copy"))).label("Copy"))
                        .button(h::Button::new(el_id(format!("tbar-{key}-cut"))).label("Cut")),
                )
        };
        doc_page(
            "Toolbar",
            crate::pages::Page::Toolbar.description(),
            crate::pages::Page::Toolbar.import_line(),
            vec![
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
        doc_page(
            "Avatar",
            crate::pages::Page::Avatar.description(),
            crate::pages::Page::Avatar.import_line(),
            vec![
                (
                    "Sizes",
                    row(Size::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::Avatar::new().name("Ada Lovelace").size(*s),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| h::Avatar::new().name("HG").color(*c))
                        .els()),
                ),
                (
                    "Variants",
                    row(h::AvatarVariant::ALL
                        .iter()
                        .map(|v| {
                            spec(
                                v.label(),
                                h::Avatar::new().name("HG").color(Color::Accent).variant(*v),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Group",
                    row(vec![h::AvatarGroup::new(vec![
                        h::Avatar::new().name("Ada Lovelace"),
                        h::Avatar::new().name("Grace Hopper"),
                        h::Avatar::new().name("Alan Turing"),
                        h::Avatar::new().name("Katherine Johnson"),
                        h::Avatar::new().name("Margaret Hamilton"),
                    ])
                    .max(3)
                    .into_any_element()]),
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
        doc_page(
            "Accordion",
            crate::pages::Page::Accordion.description(),
            crate::pages::Page::Accordion.import_line(),
            vec![
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
        let crumbs = || {
            vec![
                h::Crumb::new("Home").href("#"),
                h::Crumb::new("Components").href("#"),
                h::Crumb::new("Breadcrumbs"),
            ]
        };
        doc_page(
            "Breadcrumbs",
            crate::pages::Page::Breadcrumbs.description(),
            crate::pages::Page::Breadcrumbs.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Breadcrumbs::new(crumbs()).into_any_element()]),
                ),
                (
                    "Separators",
                    col(vec![
                        h::Breadcrumbs::new(crumbs())
                            .separator(h::BreadcrumbSeparator::Slash)
                            .into_any_element(),
                        h::Breadcrumbs::new(crumbs())
                            .separator(h::BreadcrumbSeparator::Chevron)
                            .into_any_element(),
                        h::Breadcrumbs::new(crumbs())
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
        doc_page(
            "Disclosure",
            crate::pages::Page::Disclosure.description(),
            crate::pages::Page::Disclosure.import_line(),
            vec![
                (
                    "Single",
                    col(vec![h::Disclosure::new("Shipping details")
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
                    col(vec![h::DisclosureGroup::new()
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
                        .expanded_keys(group)
                        .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                            toggle_key(&mut this.disclosure_group_expanded, key);
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    pub fn page_link(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Link",
            crate::pages::Page::Link.description(),
            crate::pages::Page::Link.import_line(),
            vec![(
                "Usage",
                col(vec![h::Link::new("ln-hover")
                    .label("Hover to see the underline")
                    .href("#")
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_pagination(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let page = self.pagination_page;
        doc_page(
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
        doc_page(
            "Tabs",
            crate::pages::Page::Tabs.description(),
            crate::pages::Page::Tabs.import_line(),
            vec![
                (
                    "Primary",
                    col(vec![h::Tabs::new("tabs-primary", items(), primary)
                        .on_selection_change(cx.listener(|this, key: &SharedString, _, cx| {
                            this.tab_solid = key.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Secondary",
                    col(vec![h::Tabs::new("tabs-secondary", items(), secondary)
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
        doc_page(
            "Alert Dialog",
            crate::pages::Page::AlertDialog.description(),
            crate::pages::Page::AlertDialog.import_line(),
            vec![(
                "Usage",
                col(vec![gpui::div()
                    .relative()
                    .flex()
                    .flex_col()
                    .items_start()
                    .w_full()
                    .min_h(px(240.))
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
                        h::AlertDialog::new("Delete this project?")
                            .description(
                                "This removes the project and every deployment. \
                                 This action cannot be undone.",
                            )
                            .is_open(is_open)
                            .is_destructive(true)
                            .confirm_label("Delete")
                            .on_cancel(cx.listener(|this, _, _, cx| {
                                this.alert_dialog_open = false;
                                cx.notify();
                            }))
                            .on_confirm(cx.listener(|this, _, _, cx| {
                                this.alert_dialog_open = false;
                                cx.notify();
                            })),
                    )
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_drawer(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.drawer_open;
        doc_page(
            "Drawer",
            crate::pages::Page::Drawer.description(),
            crate::pages::Page::Drawer.import_line(),
            vec![(
                "Usage",
                col(vec![gpui::div()
                    .relative()
                    .flex()
                    .flex_col()
                    .items_start()
                    .w_full()
                    .min_h(px(240.))
                    .child(
                        h::Button::new("dr-open")
                            .label("Open drawer")
                            .on_press(cx.listener(|this, _, _, cx| {
                                this.drawer_open = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        h::Drawer::new()
                            .is_open(is_open)
                            .title("Settings")
                            .placement(h::DrawerPlacement::Right)
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
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_modal(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.modal_open;
        doc_page(
            "Modal",
            crate::pages::Page::Modal.description(),
            crate::pages::Page::Modal.import_line(),
            vec![(
                "Usage",
                col(vec![gpui::div()
                    .relative()
                    .flex()
                    .flex_col()
                    .items_start()
                    .w_full()
                    .min_h(px(280.))
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
                            .is_open(is_open)
                            .title("Create account")
                            .is_dismissible(true)
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
                    .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_popover(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.popover_open;
        doc_page(
            "Popover",
            crate::pages::Page::Popover.description(),
            crate::pages::Page::Popover.import_line(),
            vec![(
                "Usage",
                col(vec![h::Popover::new(
                    h::Button::new("po-trigger")
                        .label("Open popover")
                        .variant(Variant::Secondary),
                )
                .is_open(is_open)
                .title("Quick note")
                .placement(h::PopoverPlacement::Bottom)
                .child(gpui::div().child("Popovers are anchored to their trigger."))
                .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                    this.popover_open = *open;
                    cx.notify();
                })))
                .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_toast(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Toast",
            crate::pages::Page::Toast.description(),
            crate::pages::Page::Toast.import_line(),
            vec![(
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
            )],
            cx,
        )
    }

    pub fn page_tooltip(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Tooltip",
            crate::pages::Page::Tooltip.description(),
            crate::pages::Page::Tooltip.import_line(),
            vec![
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
        doc_page(
            "Autocomplete",
            crate::pages::Page::Autocomplete.description(),
            crate::pages::Page::Autocomplete.import_line(),
            vec![(
                "Usage",
                col(vec![h::Autocomplete::new(
                    self.ac_entity.clone(),
                    languages(),
                )
                .label("Language")
                .placeholder("Start typing")
                .into_any_element()]),
            )],
            cx,
        )
    }

    pub fn page_combo_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.combo_open;
        doc_page(
            "Combo Box",
            crate::pages::Page::ComboBox.description(),
            crate::pages::Page::ComboBox.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        languages(),
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
                    "Custom values allowed",
                    col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        languages(),
                    )
                    .label("Language")
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
        doc_page(
            "Select",
            crate::pages::Page::Select.description(),
            crate::pages::Page::Select.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Select::new("sel-main", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
                        .is_open(is_open)
                        .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                            this.select_open = *open;
                            cx.notify();
                        })))
                        .on_selection_change(opt_usize_cb(cx.listener(
                            |this, i: &Option<usize>, _, cx| {
                                this.select_lang = *i;
                                this.select_open = false;
                                cx.notify();
                            },
                        )))
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    col(vec![h::Select::new("sel-unc", languages())
                        .label("Language")
                        .default_value(Some(0))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(FieldVariant::ALL
                        .iter()
                        .map(|v| {
                            h::Select::new(el_id(format!("sel-{v:?}")), languages())
                                .label(v.label())
                                .value(selected)
                                .variant(*v)
                        })
                        .els()),
                ),
                (
                    "Full width",
                    col(vec![h::Select::new("sel-full", languages())
                        .label("Language")
                        .value(selected)
                        .full_width(true)
                        .into_any_element()]),
                ),
                (
                    "Multiple selection",
                    col(vec![
                        h::Select::new("sel-multi", languages())
                            .label("Languages")
                            .placeholder("Pick several")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_indices(self.select_multi.iter().copied())
                            .is_open(true)
                            .on_selection_change_all(cx.listener(|this, next: &[usize], _, cx| {
                                this.select_multi = next.to_vec();
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("{} selected", self.select_multi.len()), cx),
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
    // Typography
    // -----------------------------------------------------------------------

    pub fn page_kbd(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        doc_page(
            "Kbd",
            crate::pages::Page::Kbd.description(),
            crate::pages::Page::Kbd.import_line(),
            vec![
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
        doc_page(
            "Typography",
            crate::pages::Page::Typography.description(),
            crate::pages::Page::Typography.import_line(),
            vec![
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
        doc_page(
            "Scroll Shadow",
            crate::pages::Page::ScrollShadow.description(),
            crate::pages::Page::ScrollShadow.import_line(),
            vec![
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
