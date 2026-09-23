//! Pickers gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

/// v3.2.6's "Custom Value" currencies: `(id, code, name, symbol)`.
const CURRENCIES: [(&str, &str, &str, &str); 5] = [
    ("usd", "USD", "US Dollar", "$"),
    ("eur", "EUR", "Euro", "€"),
    ("gbp", "GBP", "British Pound", "£"),
    ("jpy", "JPY", "Japanese Yen", "¥"),
    ("chf", "CHF", "Swiss Franc", "₣"),
];

impl Gallery {
    // Pickers
    // -----------------------------------------------------------------------

    pub fn page_autocomplete(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let ac_picked = self.demo_text_value("ac-picked");
        let ac_typed = self.demo_text_value("ac-typed");
        let ac_multi = self.demo_selection("ac-multi");
        let ac_open = self.demo_flag("ac-open", false);
        let ac_muted = cx.colors().muted;
        component_doc_page!(
            "Autocomplete",
            crate::pages::Page::Autocomplete.description(),
            crate::pages::Page::Autocomplete.import_line(),
            vec![
                (
                    "Usage", "Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup anchors to the trigger with an 8px gap, flips when the preferred side cannot fit and the opposite side has more room, keeps the search visible, and scrolls the list within the available height up to 320px; virtual paging follows the visible list height.",
                    specimen_body("ac-main", field_col(vec![h::Autocomplete::new(
                        self.ac_entity.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    // `radius` is the detached panel's corner; the trigger
                    // keeps the field chrome's own.
                    .radius(px(8.))
                    .into_any_element()]), cx),
                ),
                (
                    "Placement",
                    "The field panel can enter above its trigger; the four-pixel entry translation follows the resolved physical side after viewport flipping.",
                    specimen_body("ac-placement-top", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-placement-top", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .placement(h::Placement::Top)
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Long Selected Value",
                    "Selected labels follow HeroUI's wrap-break-word value slot: the trigger grows vertically instead of clipping the label with an ellipsis.",
                    specimen_body("ac-long-value", col(vec![
                        gpui::div()
                            .w(px(260.))
                            .child(
                                h::Autocomplete::new(
                                    self.demo_text("ac-long-value", "", cx),
                                    vec![
                                        h::PickerItem::new(
                                            "long",
                                            "A project name that is intentionally long enough to wrap",
                                        ),
                                        h::PickerItem::new("short", "Short option"),
                                    ],
                                )
                                .label("Project")
                                .placeholder("Choose one")
                                .default_value(["long"]),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` size the trigger; `row_padding_x` / `row_padding_y` size the suggestion rows.",
                    specimen_body("ac-box-customisation", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-custom-box", "", cx),
                        language_items(),
                    )
                    .label("Compact")
                    .placeholder("Pick one")
                    .height(px(28.))
                    .padding_x(px(4.))
                    .is_bare(true)
                    .row_padding_x(px(16.))
                    .row_padding_y(px(2.))
                    .row_hover_bg(cx.colors().accent.soft())
                    .row_font_family(crate::app::MONO_FONT)
                    .into_any_element()]), cx),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    specimen_body("ac-virtualization", col(vec![
                        demo_field(
                            h::Autocomplete::new(
                                self.demo_text("ac-virtual", "", cx),
                                virtual_picker_items(),
                            )
                            .label("User")
                            .placeholder("Select a user")
                            .max_items(1000)
                            .row_height(px(40.)),
                        ),
                    ]), cx),
                ),
                (
                    "Variants",
                    field_col(vec![
                        specimen_body(
                            "ac-variant-Primary",
                            h::Autocomplete::new(self.demo_text("ac-primary", "", cx), language_items())
                                .label("Primary")
                                .placeholder("Select a language")
                                .into_any_element(),
                            cx,
                        ),
                        specimen_body(
                            "ac-variant-Secondary",
                            h::Autocomplete::new(self.demo_text("ac-secondary", "", cx), language_items())
                                .label("Secondary")
                                .placeholder("Select a language")
                                .variant(FieldVariant::Secondary)
                                .into_any_element(),
                            cx,
                        ),
                    ]),
                ),
                (
                    "In Surface",
                    specimen_body("ac-in-surface", field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Autocomplete::new(self.demo_text("ac-surface", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Full Width",
                    specimen_body("ac-full-width", col(vec![gpui::div()
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
                        .into_any_element()]), cx),
                ),
                (
                    "With Description",
                    specimen_body("ac-description", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-desc", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .description("Type to filter the list")
                    .into_any_element()]), cx),
                ),
                (
                    "Required",
                    specimen_body("ac-required", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-required", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_required(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("ac-disabled", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "With Disabled Options",
                    specimen_body("ac-disabled-options", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled-opts", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .disabled_keys([SharedString::from("go"), SharedString::from("python")])
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Allows Empty Collection",
                    specimen_body("ac-empty-collection", field_col(vec![h::Autocomplete::new(
                        self.demo_text("ac-empty", "zzz", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .allows_empty_collection(true)
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "With Sections",
                    specimen_body("ac-sections", field_col(vec![h::Autocomplete::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Multiple Select",
                    specimen_body("ac-multiple-select", field_col(vec![h::Autocomplete::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("ac-controlled", col(vec![
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
                    ]), cx),
                ),
                (
                    "Controlled Multiple",
                    specimen_body("ac-controlled-multiple", field_col(vec![h::Autocomplete::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled Open State",
                    specimen_body("ac-controlled-open", col(vec![
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
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ac-open", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Asynchronous Filtering", "The matches are fetched as the query changes. `filter` is the hook for that -- it decides what counts as a match -- and a spinner beside the field says a request is in flight.",
                    specimen_body("ac-async-filtering", col(vec![
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
                    ]), cx),
                ),
                (
                    "Custom Indicator",
                    specimen_body("ac-custom-indicator", field_col(vec![h::Autocomplete::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Custom Value", "`Autocomplete.Value` takes a render function that is handed the placeholder and selection state — `is_placeholder`, `selected_keys`, `selected_items` and `selected_text`. This one draws the chosen currency as its symbol, code and muted name, and hands the default back while nothing is chosen. The port's rows carry one text run, so each currency row shows its code and name together, the text v3 filters on.",
                    specimen_body("ac-custom-value", col(vec![
                        h::Autocomplete::new(
                            self.demo_text("ac-custom", "", cx),
                            CURRENCIES
                                .iter()
                                .map(|(key, code, name, _)| {
                                    h::PickerItem::new(*key, format!("{code} {name}"))
                                })
                                .collect(),
                        )
                        .label("Currency")
                        .placeholder("Select a currency")
                        .selection_mode(SelectionMode::Single)
                        .default_value(["usd"])
                        .value_content(move |value| {
                            let selected = value
                                .selected_keys
                                .and_then(|keys| keys.first())
                                .and_then(|key| {
                                    CURRENCIES.iter().find(|(id, ..)| *id == key.as_ref())
                                });
                            let Some((_, code, name, symbol)) = selected.filter(|_| !value.is_placeholder) else {
                                return value.default_children;
                            };
                            // `flex min-w-0 items-center gap-1.5`: the
                            // symbol in medium weight, the code, then the
                            // muted, truncating name.
                            gpui::div()
                                .flex()
                                .min_w_0()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    gpui::div()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(*symbol),
                                )
                                .child(*code)
                                .child(
                                    gpui::div()
                                        .min_w_0()
                                        .truncate()
                                        .text_color(ac_muted)
                                        .child(*name),
                                )
                                .into_any_element()
                        })
                        .into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_combo_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
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
                    "Usage", "Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup flips near window edges and scrolls to keep options reachable in short windows.",
                    specimen_body("cb-main", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-usage", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .is_open(self.demo_overlay("cb-usage-open"))
                    // `radius` is the detached panel's corner; the trigger is
                    // the inner input's box and keeps the field chrome's own.
                    .radius(px(8.))
                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                        this.set_demo_flag("cb-usage-open", *open);
                        cx.notify();
                    }))
                    .on_selection_change(cx.listener(|this, _key: &SharedString, _, cx| {
                        this.set_demo_flag("cb-usage-open", false);
                        cx.notify();
                    }))
                    .into_any_element()]), cx),
                ),
                (
                    "Placement",
                    "The list can enter above its field; the placement translation follows the physical side resolved by the viewport positioner.",
                    specimen_body("cb-placement-top", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-placement-top", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .placement(h::Placement::Top)
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Box Customisation",
                    "The trigger forwards `height`, `padding_x` and `is_bare` to its input, and `row_padding_x` / `row_padding_y` size the rows.",
                    specimen_body("cb-box-customisation", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-custom-box", "", cx),
                        language_items(),
                    )
                    .label("Compact")
                    .placeholder("Pick one")
                    .height(px(28.))
                    .padding_x(px(4.))
                    .is_bare(true)
                    .row_padding_x(px(16.))
                    .row_padding_y(px(2.))
                    .row_hover_bg(cx.colors().accent.soft())
                    .row_font_family(crate::app::MONO_FONT)
                    .into_any_element()]), cx),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    specimen_body("cb-virtualization", col(vec![
                        demo_field(
                            h::ComboBox::new(
                                self.demo_text("cb-virtual", "", cx),
                                virtual_picker_items(),
                            )
                            .label("User")
                            .placeholder("Select a user")
                            .max_items(1000)
                            .row_height(px(40.)),
                        ),
                    ]), cx),
                ),
                (
                    "Full Width",
                    specimen_body("cb-full-width", col(vec![gpui::div()
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
                        .into_any_element()]), cx),
                ),
                (
                    "With Description",
                    specimen_body("cb-description", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-desc", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .description("Pick from the list or type your own")
                    .into_any_element()]), cx),
                ),
                (
                    "Required",
                    specimen_body("cb-required", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-required", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_required(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("cb-disabled", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_disabled(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Read Only",
                    specimen_body("cb-read-only", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-readonly", "Rust", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .is_read_only(true)
                    .into_any_element()]), cx),
                ),
                (
                    "In Surface",
                    specimen_body("cb-in-surface", field_col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::ComboBox::new(self.demo_text("cb-surface", "", cx), language_items())
                                .label("Language")
                                .placeholder("Select a language")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                        cx),
                ),
                (
                    "With Disabled Options",
                    specimen_body("cb-disabled-options", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled-opts", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    .disabled_keys([SharedString::from("go")])
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "With Sections",
                    specimen_body("cb-sections", field_col(vec![h::ComboBox::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("cb-controlled", col(vec![
                        h::ComboBox::new(self.demo_text("cb-controlled", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            // `selectedKey` is the controlled selection key
                            // (empty string is v3's `null`); the input shows
                            // that key's label and `inputValue` holds the
                            // typed text. The pick reports the key, which is
                            // what the line below displays.
                            .selected_key(cb_picked.clone())
                            .input_value(cb_typed.clone())
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
                    ]), cx),
                ),
                (
                    "Controlled Input Value",
                    specimen_body("cb-controlled-input", col(vec![
                        h::ComboBox::new(self.demo_text("cb-input", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .on_input_change(cx.listener(|this, text: &str, _, cx| {
                                this.set_demo_text_value("cb-typed", text.to_owned());
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Typed: {cb_typed}"), cx),
                    ]), cx),
                ),
                (
                    "Controlled Selection",
                    specimen_body("cb-controlled-selection", field_col(vec![h::ComboBox::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Multiple Selection",
                    specimen_body("cb-multiple-selection", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-multi-sel", "", cx),
                        language_items(),
                    )
                    .label("Languages")
                    .placeholder("Select languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Value Render Props", "Pick a language with the pointer or keyboard, or clear the input. The value below follows the controlled selection.",
                    specimen_body("cb-value-render", field_col(vec![h::ComboBox::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Long Value Content",
                    "The default `ComboBox.Value` render-prop content follows HeroUI's wrap-break-word contract and grows for long selected labels.",
                    specimen_body("cb-long-value", col(vec![
                        gpui::div()
                            .w(px(260.))
                            .child(
                                h::ComboBox::new(
                                    self.demo_text("cb-long-value", "", cx),
                                    vec![
                                        h::PickerItem::new(
                                            "long",
                                            "A project name that is intentionally long enough to wrap",
                                        ),
                                        h::PickerItem::new("short", "Short option"),
                                    ],
                                )
                                .label("Project")
                                .placeholder("Choose one")
                                .default_value(["long"])
                                .value_content(|value| value.default_children),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Default Selected Key",
                    specimen_body("cb-default-selected", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-default-key", "TypeScript", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Search languages...")
                    .default_value(["typescript"])
                    .into_any_element()]), cx),
                ),
                (
                    "Allows Custom Value",
                    specimen_body("cb-allows-custom-value", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-custom", "Zig", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .allows_custom_value(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Asynchronous Loading", "The list is filled from a request. The spinner beside the field is what says one is in flight; the options are the caller's own data. `allows_empty_collection` keeps the panel up with its empty state while a query has no matches instead of collapsing it.",
                    specimen_body("cb-async-loading", col(vec![
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
                    ]), cx),
                ),
                (
                    "Custom Indicator",
                    specimen_body("cb-custom-indicator", field_col(vec![h::ComboBox::new(
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
                    .into_any_element()]), cx),
                ),
                (
                    "Custom Filtering", "`defaultFilter` here is `useFilter`'s `startsWith`, so it matches on the start of the name only.",
                    specimen_body("cb-custom-filtering", col(vec![
                        h::ComboBox::new(self.demo_text("cb-filter", "", cx), language_items())
                            .label("Language")
                            .placeholder("Select a language")
                            .filter(|item, input| {
                                h::Filter::new(h::Sensitivity::Base).starts_with(item, input)
                            })
                            .default_open(true)
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Menu Trigger",
                    specimen_body("cb-menu-trigger", col(vec![
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
                    ]), cx),
                ),
                (
                    "Form Value", "Items are keyed `PickerItem`s: the selection is the item's key while the input shows its label, and `form_value` decides what a named field submits. The default (`ComboBoxFormValue::Key`) submits the picked key -- save with a pick and the submitted value is `language=rust`, not `Rust` -- and `allows_custom_value` keeps the typed text.",
                    specimen_body("cb-form-value", col(vec![
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
                    ]), cx),
                ),
                (
                    "Validation Behavior",
                    specimen_body("cb-validation-behavior", col(vec![
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
                    ]), cx),
                ),
                (
                    "Custom Validation",
                    specimen_body("cb-custom-validation", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-validate", "Zig", cx),
                        language_items(),
                    )
                    .label("Language")
                    .allows_custom_value(true)
                    .validate(|value| {
                        (!value.is_empty() && !languages().iter().any(|l| l == value))
                            .then(|| "Pick one of the listed languages".into())
                    })
                    .into_any_element()]), cx),
                ),
                (
                    "Custom Value",
                    specimen_body("cb-custom-value", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-custom-value", "", cx),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .allows_custom_value(true)
                    .is_open(self.demo_overlay("cb-custom-open"))
                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                        this.set_demo_flag("cb-custom-open", *open);
                        cx.notify();
                    }))
                    .into_any_element()]), cx),
                ),
                (
                    "Basic Usage", "The plainest combo box: a labeled input over a two-item list, uncontrolled, with the trigger opening the same filtered list.",
                    specimen_body("cb-basic-usage", field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-basic", "", cx),
                        vec![
                            h::PickerItem::new("cat", "Cat"),
                            h::PickerItem::new("dog", "Dog"),
                        ],
                    )
                    .label("Favorite Animal")
                    .placeholder("Search animals...")
                    .into_any_element()]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_select(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.select_lang.clone();
        let is_open = self.select_open;
        let sel_multi = self.select_multi.clone();
        component_doc_page!(
            "Select",
            crate::pages::Page::Select.description(),
            crate::pages::Page::Select.import_line(),
            vec![
                (
                    "Usage", "Use the arrow keys and Enter or Space to select a language. Selection closes the list and keeps focus on the trigger. Enabled options use the pinned 98% press scale over 250ms ease-out-quart inside a stable row slot. Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup flips near window edges and scrolls to keep options reachable in short windows.",
                    specimen_body("sel-main", field_col(vec![h::Select::new("sel-main", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected.clone())
                        .is_open(is_open)
                        // `radius` is the detached panel's corner; the trigger
                        // keeps the field chrome's own.
                        .radius(px(8.))
                        .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                            this.select_open = *open;
                            cx.notify();
                        }))
                        .on_change(cx.listener(|this, key: &Option<SharedString>, _, cx| {
                            this.select_lang = key.clone();
                            this.select_open = false;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Placement",
                    "The list can enter above its trigger; the placement translation follows the physical side resolved by the viewport positioner.",
                    specimen_body("sel-placement-top", field_col(vec![h::Select::new(
                        "sel-placement-top",
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Choose one")
                    .placement(h::Placement::Top)
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "Long Values & Option Focus",
                    "Selected option labels keep the normal foreground and use a check indicator. Long selected values and natural-height option rows wrap inside the available width instead of being clipped.",
                    specimen_body("sel-long-values", col(vec![
                        gpui::div()
                            .w(px(260.))
                            .child(
                                h::Select::new(
                                    "sel-long-values",
                                    vec![
                                        h::PickerItem::new(
                                            "long",
                                            "A project name that is intentionally long enough to wrap",
                                        ),
                                        h::PickerItem::new("short", "Short option"),
                                    ],
                                )
                                .label("Project")
                                .default_value(Some("long".into()))
                                .default_open(true),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "With Clear Button", "Clear the selection with the close control or Backspace/Delete on the closed trigger. An empty selection retains the control's layout space.",
                    specimen_body("sel-clear", field_col(vec![
                        h::Select::new("select-clear-single", language_items())
                            .label("Language").default_value(Some("rust".into()))
                            .clear_button(h::SelectClearButton::new()).into_any_element(),
                        h::Select::new("select-clear-multiple", language_items())
                            .label("Languages").variant(FieldVariant::Secondary)
                            .selection_mode(SelectionMode::Multiple)
                            .default_selected_keys(["rust".into(), "go".into()])
                            .clear_button(h::SelectClearButton::new()).into_any_element(),
                    ]), cx),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` size the trigger; `row_padding_x` / `row_padding_y` size its option rows. This 28px bare trigger uses 12px trigger and row text, 8px row insets and 4px panel padding.",
                    specimen_body("sel-box-customisation", field_col(vec![h::Select::new("sel-custom-box", language_items())
                        .label("Compact")
                        .placeholder("Choose one")
                        .height(px(28.))
                        .padding_x(px(4.))
                        .is_bare(true)
                        .trigger_text_size(px(12.))
                        .row_text_size(px(12.))
                        .panel_padding(px(4.))
                        .row_height(px(28.))
                        .row_padding_x(px(8.))
                        .row_padding_y(px(2.))
                        .row_hover_bg(cx.colors().accent.soft())
                        .row_font_family(crate::app::MONO_FONT)
                        .default_open(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each. The list sizes to the available height. Options remain reachable with the keyboard and mouse wheel.",
                    specimen_body("sel-virtualization", col(vec![
                        demo_field(
                            h::Select::new("sel-virtual", virtual_picker_items())
                                .label("User")
                            .placeholder("Choose one")
                                .row_height(px(40.)),
                        ),
                    ]), cx),
                ),
                (
                    "With Description",
                    specimen_body("sel-description", field_col(vec![h::Select::new("sel-desc", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .description("Used for spell-checking")
                        .into_any_element()]), cx),
                ),
                (
                    "Required",
                    specimen_body("sel-required", field_col(vec![h::Select::new("sel-required", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_required(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled",
                    specimen_body("sel-disabled", field_col(vec![h::Select::new("sel-disabled", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
                (
                    "With Disabled Options",
                    specimen_body("sel-disabled-options", field_col(vec![h::Select::new("sel-disabled-opts", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .disabled_keys([SharedString::from("typescript"), SharedString::from("go")])
                        .default_open(true)
                        .into_any_element()]), cx),
                ),
                (
                    "With Sections",
                    specimen_body("sel-sections", field_col(vec![h::Select::new(
                        "sel-sections",
                        vec![
                            h::PickerItem::new("us", "United States"),
                            h::PickerItem::new("ca", "Canada"),
                            h::PickerItem::new("mx", "Mexico"),
                            h::PickerItem::new("fr", "France"),
                            h::PickerItem::new("de", "Germany"),
                        ],
                    )
                    .label("Country")
                    .placeholder("Select a country")
                    .section_before("us", "North America")
                    .section_before("fr", "Europe")
                    .default_open(true)
                    .into_any_element()]), cx),
                ),
                (
                    "In Surface",
                    specimen_body("sel-in-surface", col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Select::new("sel-surface", language_items())
                                .label("Language")
                                .placeholder("Choose one")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                        cx),
                ),
                (
                    "Controlled Multiple",
                    specimen_body("sel-controlled-multiple", col(vec![
                        h::Select::new("sel-ctl-multi", language_items())
                            .label("Languages")
                            .placeholder("Choose any")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_keys(sel_multi.iter().cloned())
                            .on_selection_change_all(cx.listener(
                                |this, keys: &[SharedString], _, cx| {
                                    this.select_multi = keys.to_vec();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", sel_multi.len()), cx),
                    ]), cx),
                ),
                (
                    "Controlled Open State",
                    specimen_body("sel-open", col(vec![
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
                        h::Select::new("sel-open", language_items())
                            .label("Language")
                            .placeholder("Choose one")
                            .is_open(is_open)
                            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                this.select_open = *open;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Asynchronous Loading", "The list is filled from a request and a spinner shows while it is in flight. The spinner is composed beside the label, since the options are the caller's own data.",
                    specimen_body("sel-async-loading", col(vec![
                        row(vec![
                            h::Select::new("sel-async", language_items())
                                .label("Language")
                                .placeholder("Loading\u{2026}")
                                .into_any_element(),
                            h::Spinner::new("sel-async-spinner")
                                .size(h::SpinnerSize::Sm)
                            .into_any_element(),
                        ]),
                    ]), cx),
                ),
                (
                    "Custom Indicator",
                    specimen_body("sel-custom-indicator", field_col(vec![h::Select::new("sel-indicator", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected.clone())
                        .on_selection_change(cx.listener(
                            |this, key: &Option<SharedString>, _, cx| {
                                this.select_lang = key.clone();
                                cx.notify();
                            },
                        ))
                        .default_open(true)
                        .indicator(|is_selected| {
                            gpui::div()
                                .text_size(px(12.))
                                .child(if is_selected { "\u{2714}" } else { "" })
                                .into_any_element()
                        })
                        .into_any_element()]),
                        cx),
                ),
                (
                    "Row Leading Content",
                    "`item_leading` draws an element before each option's label — v3's `ListBox.Item` render function. The closure takes the row's key and whether it is selected, and returns `None` for a row that takes none.",
                    specimen_body("sel-item-leading", field_col(vec![
                        h::Select::new("sel-leading", swatch_picker_items())
                            .label("Accent")
                            .placeholder("Choose one")
                            .default_open(true)
                            .item_leading(|key, _| {
                                Some(
                                    gpui::div()
                                        .size(px(14.))
                                        .rounded(px(4.))
                                        .bg(swatch_colour(key))
                                        .into_any_element(),
                                )
                            })
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Wrapping Trigger",
                    "The trigger is a `min-h` box, so `padding_y` leaves a single-line value at 36px and only grows a value that has wrapped — v3's `py-2`. The narrow field below forces the wrap.",
                    specimen_body("sel-padding-y", col(vec![
                        gpui::div().w(px(180.)).child(
                            h::Select::new("sel-wrap", layout_picker_items())
                                .label("Layout")
                                .default_value(Some("side".into()))
                                .padding_y(px(8.))
                                .full_width(true),
                        ).into_any_element(),
                    ]), cx),
                ),
                (
                    "Custom Trigger Indicator",
                    specimen_body("sel-trigger-indicator", field_col(vec![
                        h::Select::new("sel-trigger-indicator", language_items())
                            .label("Language")
                            .placeholder("Choose one")
                            .default_open(true)
                            .trigger_indicator(|is_open| {
                                gpui::div()
                                    .text_size(px(16.))
                                    .child(if is_open { "−" } else { "+" })
                                    .into_any_element()
                            })
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Custom Value",
                    specimen_body("sel-custom-value", field_col(vec![h::Select::new("sel-value", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected.clone())
                        .on_selection_change(cx.listener(
                            |this, key: &Option<SharedString>, _, cx| {
                                this.select_lang = key.clone();
                                cx.notify();
                            },
                        ))
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
                        .into_any_element()]), cx),
                ),
                (
                    "Uncontrolled",
                    specimen_body("sel-uncontrolled", field_col(vec![h::Select::new("sel-unc", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .default_value(Some(SharedString::from("rust")))
                        .into_any_element()]), cx),
                ),
                (
                    "Variants",
                    field_col(FieldVariant::ALL
                        .iter()
                        .filter_map(|v| {
                            let key = format!("sel-variant-{v:?}");
                            crate::control::specimen_wanted(&key, cx).then(|| {
                                h::Select::new(el_id(format!("sel-{v:?}")), language_items())
                                    .label(v.label())
                                    .placeholder("Choose one")
                                    .value(selected.clone())
                                    .on_selection_change(cx.listener(
                                        |this, key: &Option<SharedString>, _, cx| {
                                            this.select_lang = key.clone();
                                            cx.notify();
                                        },
                                    ))
                                    .variant(*v)
                            })
                        })
                        .els()),
                ),
                (
                    "Full Width",
                    specimen_body("sel-full-width", col(vec![gpui::div()
                        .w(px(400.))
                        .child(
                            h::Select::new("sel-full", language_items())
                                .label("Language")
                                .placeholder("Choose one")
                                .value(selected.clone())
                                .on_selection_change(cx.listener(
                                    |this, key: &Option<SharedString>, _, cx| {
                                        this.select_lang = key.clone();
                                        cx.notify();
                                    },
                                ))
                                .full_width(true),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Multiple Select",
                    specimen_body("sel-multiple-select", field_col(vec![h::Select::new("sel-multi", language_items())
                        .label("Languages")
                        .placeholder("Pick several")
                        .selection_mode(SelectionMode::Multiple)
                        .default_selected_keys([SharedString::from("rust"), SharedString::from("python")])
                        .default_open(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Controlled", "The selected key lives in the caller: the trigger reads `value` and the caption prints the same state.",
                    specimen_body("sel-controlled", field_col(vec![
                        h::Select::new("sel-controlled", language_items())
                            .label("Language (controlled)")
                            .placeholder("Choose one")
                            .value(self.select_lang_controlled.clone())
                            .on_change(cx.listener(
                                |this, key: &Option<SharedString>, _, cx| {
                                    this.select_lang_controlled = key.clone();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &match self.select_lang_controlled.as_ref() {
                                Some(key) => format!(
                                    "Selected: {}",
                                    language_items()
                                        .iter()
                                        .find(|item| item.key() == key)
                                        .map(|item| item.label().to_string())
                                        .unwrap_or_default()
                                ),
                                None => "Selected: none".to_owned(),
                            },
                            cx,
                        ),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
