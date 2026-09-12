//! Pickers gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
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
                    "Usage", "Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup anchors to the trigger with an 8px gap, flips when the preferred side cannot fit and the opposite side has more room, keeps the search visible, and scrolls the list within the available height up to 320px; virtual paging follows the visible list height.",
                    field_col(vec![h::Autocomplete::new(
                        self.ac_entity.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Select a language")
                    // `radius` is the detached panel's corner; the trigger
                    // keeps the field chrome's own.
                    .radius(px(8.))
                    .into_any_element()]),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` size the trigger; `row_padding_x` / `row_padding_y` size the suggestion rows.",
                    field_col(vec![h::Autocomplete::new(
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
                    .into_any_element()]),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    col(vec![
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
                            .on_open_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ac-open", *v);
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Filtering", "The matches are fetched as the query changes. `filter` is the hook for that -- it decides what counts as a match -- and a spinner beside the field says a request is in flight.",
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
                    "Custom Value", "`Autocomplete.Value` takes a render function that is handed the placeholder and selection state — `is_placeholder`, `selected_items` and `selected_text`. This one draws the selection as tags and hands the default back while nothing is chosen.",
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
                    "Usage", "Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup flips near window edges and scrolls to keep options reachable in short windows.",
                    field_col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .is_open(is_open)
                    // `radius` is the detached panel's corner; the trigger is
                    // the inner input's box and keeps the field chrome's own.
                    .radius(px(8.))
                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                        this.combo_open = *open;
                        cx.notify();
                    }))
                    .on_selection_change(cx.listener(|this, _key: &SharedString, _, cx| {
                        this.combo_open = false;
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Box Customisation",
                    "The trigger forwards `height`, `padding_x` and `is_bare` to its input, and `row_padding_x` / `row_padding_y` size the rows.",
                    field_col(vec![h::ComboBox::new(
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
                    .into_any_element()]),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each.",
                    col(vec![
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
                    "Asynchronous Loading", "The list is filled from a request. The spinner beside the field is what says one is in flight; the options are the caller's own data. `allows_empty_collection` keeps the panel up with its empty state while a query has no matches instead of collapsing it.",
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
                    "Form Value", "Items are keyed `PickerItem`s: the selection is the item's key while the input shows its label, and `form_value` decides what a named field submits. The default (`ComboBoxFormValue::Key`) submits the picked key -- save with a pick and the submitted value is `language=rust`, not `Rust` -- and `allows_custom_value` keeps the typed text.",
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
                    "Custom Value",
                    field_col(vec![h::ComboBox::new(
                        self.combo_state.clone(),
                        language_items(),
                    )
                    .label("Language")
                    .placeholder("Pick or type")
                    .allows_custom_value(true)
                    .is_open(is_open)
                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                        this.combo_open = *open;
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Basic Usage", "The plainest combo box: a labeled input over a two-item list, uncontrolled, with the trigger opening the same filtered list.",
                    field_col(vec![h::ComboBox::new(
                        self.demo_text("cb-basic", "", cx),
                        vec![
                            h::PickerItem::new("cat", "Cat"),
                            h::PickerItem::new("dog", "Dog"),
                        ],
                    )
                    .label("Favorite Animal")
                    .placeholder("Search animals...")
                    .into_any_element()]),
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
                    "Usage", "Use the arrow keys and Enter or Space to select a language. Selection closes the list and keeps focus on the trigger. Values and options use 14px text with 20px lines. Section headers use 12px text with 16px lines and keep their own spacing. The popup flips near window edges and scrolls to keep options reachable in short windows.",
                    field_col(vec![h::Select::new("sel-main", language_items())
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
                        .into_any_element()]),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` size the trigger; `row_padding_x` / `row_padding_y` size its option rows. This 28px bare trigger uses 12px trigger and row text, 8px row insets and 4px panel padding.",
                    field_col(vec![h::Select::new("sel-custom-box", language_items())
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
                        .into_any_element()]),
                ),
                (
                    "Virtualization", "`row_height` makes the list geometry computable, so gpui's `uniform_list` builds only the rows in view. A thousand options, forty pixels each. The list sizes to the available height. Options remain reachable with the keyboard and mouse wheel.",
                    col(vec![
                        demo_field(
                            h::Select::new("sel-virtual", virtual_picker_items())
                                .label("User")
                            .placeholder("Choose one")
                            .row_height(px(40.)),
                        ),
                    ]),
                ),
                (
                    "With Description",
                    field_col(vec![h::Select::new("sel-desc", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .description("Used for spell-checking")
                        .into_any_element()]),
                ),
                (
                    "Required",
                    field_col(vec![h::Select::new("sel-required", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    field_col(vec![h::Select::new("sel-disabled", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    field_col(vec![h::Select::new("sel-disabled-opts", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .disabled_keys([SharedString::from("typescript"), SharedString::from("go")])
                        .default_open(true)
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    field_col(vec![h::Select::new(
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
                    .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Select::new("sel-surface", language_items())
                                .label("Language")
                                .placeholder("Choose one")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled Multiple",
                    col(vec![
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
                        h::Select::new("sel-open", language_items())
                            .label("Language")
                            .placeholder("Choose one")
                            .is_open(is_open)
                            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                this.select_open = *open;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Loading", "The list is filled from a request and a spinner shows while it is in flight. The spinner is composed beside the label, since the options are the caller's own data.",
                    col(vec![
                        row(vec![
                            h::Select::new("sel-async", language_items())
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
                    field_col(vec![h::Select::new("sel-indicator", language_items())
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
                ),
                (
                    "Custom Value",
                    field_col(vec![h::Select::new("sel-value", language_items())
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
                        .into_any_element()]),
                ),
                (
                    "Uncontrolled",
                    field_col(vec![h::Select::new("sel-unc", language_items())
                        .label("Language")
                        .placeholder("Choose one")
                        .default_value(Some(SharedString::from("rust")))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    field_col(FieldVariant::ALL
                        .iter()
                        .map(|v| {
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
                        .els()),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
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
                        .into_any_element()]),
                ),
                (
                    "Multiple Select",
                    field_col(vec![h::Select::new("sel-multi", language_items())
                        .label("Languages")
                        .placeholder("Pick several")
                        .selection_mode(SelectionMode::Multiple)
                        .default_selected_keys([SharedString::from("rust"), SharedString::from("python")])
                        .default_open(true)
                        .into_any_element()]),
                ),
                (
                    "Controlled", "The selected key lives in the caller: the trigger reads `value` and the caption prints the same state.",
                    field_col(vec![
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
                    ]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
