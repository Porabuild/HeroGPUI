//! Forms gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
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
                    "Usage", "Label content uses 14px medium text with 20px lines, independent of surrounding line height. Toggling animates: the accent fill scales and fades in while the check stroke draws, and unselecting undraws it; reduced motion snaps.",
                    col(vec![
                        h::Checkbox::new("cb-1")
                            .is_selected(basic)
                            .hover_bg(cx.colors().accent.soft_hover())
                            .label(gpui::div().child("Accept the terms"))
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.cb_basic = *v;
                                cx.notify();
                            }))
                            .into_any_element(),
                        h::Checkbox::new("cb-2")
                            .is_selected(colored)
                            .variant(FieldVariant::Secondary)
                            .label(gpui::div().child("Subscribe to updates"))
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.cb_color = *v;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Sizes",
                    "`size` is additive, not a v3 prop: `Md` is the pinned 16px control; `Sm` is a 14px control with a 10px indicator and 12px label text.",
                    col(vec![
                        h::Checkbox::new("cb-sz-sm")
                            .size(h::CheckboxSize::Sm)
                            .label(gpui::div().child("Small"))
                            .description("Supporting text aligns with this label")
                            .into_any_element(),
                        h::Checkbox::new("cb-sz-md")
                            .size(h::CheckboxSize::Md)
                            .label(gpui::div().child("Medium"))
                            .description("Supporting text aligns with this label")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants", "`radius(px)` replaces the control's default `rounded-md`; `is_round` keeps its documented circle either way.",
                    col(vec![
                        h::Checkbox::new("cb-v-primary")
                            .default_selected(true)
                            .radius(px(4.))
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
                    "Disabled", "A disabled checkbox keeps its value visible but rejects presses and leaves the tab order.",
                    row(vec![h::Checkbox::new("cb-disabled-feature")
                        .label("Premium Feature")
                        .description("This feature is coming soon")
                        .is_disabled(true)
                        .into_any_element()]),
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
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("cb-controlled", *v);
                                cx.notify();
                            }))
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
                    "Indeterminate", "A \"select all\" checkbox: it starts indeterminate, and the first change clears the dash and follows the press.",
                    col(vec![
                        h::Checkbox::new("cb-indeterminate-select-all")
                            .label("Select all")
                            .description("Shows indeterminate state (dash icon)")
                            .is_selected(self.cb_select_all_selected)
                            .is_indeterminate(self.cb_select_all_indeterminate)
                            .on_change(cx.listener(
                                |this, selected: &bool, _, cx| {
                                    this.cb_select_all_selected = *selected;
                                    this.cb_select_all_indeterminate = false;
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
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
                        .on_change(cx.listener(|this, v: &bool, _, cx| {
                            this.set_demo_flag("cb-controlled", *v);
                            cx.notify();
                        }))
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
                    "Usage", "Option labels use 14px/20px medium text; descriptions use 12px/16px regular text.",
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
                    "Indeterminate", "A \"select all\" checkbox pairs with the group: it is indeterminate while only some children are selected.",
                    col(vec![
                        h::Checkbox::new("cbg-all")
                            .is_selected(selected.len() == 3)
                            .is_indeterminate(!selected.is_empty() && selected.len() < 3)
                            .label(gpui::div().child("All notifications"))
                            .on_change(cx.listener(|this, all: &bool, _, cx| {
                                this.checkbox_group = if *all {
                                    ["email", "sms", "push"]
                                        .into_iter()
                                        .map(SharedString::from)
                                        .collect()
                                } else {
                                    HashSet::new()
                                };
                                cx.notify();
                            }))
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
                    "Controlled", "The selected set lives in the caller and is fed back through `value`; the caption prints the live set.",
                    col(vec![
                        h::CheckboxGroup::new(
                            "cbg-controlled",
                            vec![
                                h::CheckboxOption::new("coding", "Coding"),
                                h::CheckboxOption::new("design", "Design"),
                                h::CheckboxOption::new("writing", "Writing"),
                            ],
                        )
                        .label("Your skills")
                        .value(selected.iter().cloned())
                        .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.checkbox_group = keys.clone();
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(
                            &format!(
                                "Selected: {}",
                                {
                                    let mut names: Vec<&str> =
                                        selected.iter().map(|k| k.as_ref()).collect();
                                    names.sort();
                                    if names.is_empty() {
                                        "None".to_owned()
                                    } else {
                                        names.join(", ")
                                    }
                                }
                            ),
                            cx,
                        ),
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
                    "With Dynamic Messages", "`FieldError` takes a render closure and joins the validation error list. `validationErrors` here is a list, and the field shows them in order.",
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
                ("Server Errors", "`ValidationErrors` is a record of server errors keyed by field name. The Form routes each name into that field's own error slot: editing a field clears only its message while its sibling keeps theirs, Reset hides them all, and a re-render that passes the same record re-arms nothing. New response supplies a genuinely new record — identical content, fresh identity — so both messages re-arm.", {
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
                    "Text adornments inherit the field's 14px text and 20px line height. Platform text replacement, composition updates, and paste use the same editable state as keyboard input.",
                    field_col(vec![h::Input::new(self.demo_text("in-usage", "", cx))
                        .label("Name")
                        .placeholder("Enter your name")
                        .radius(px(4.))
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
                    "Labels use 14px text with 20px lines; descriptions and errors use 12px text with 16px lines, independent of surrounding leading.",
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
                            .clear_hover_bg(cx.colors().accent.soft())
                            .into_any_element(),
                    ]),
                ),
                (
                    "Compact Box",
                    "`height`, `padding_x` and `is_bare` are the box knobs: a 28px chromeless field with a leading icon, for a toolbar or a table cell the caller paints itself. Only the row height changes -- the text keeps its 14px size and 20px line.",
                    field_col(vec![h::TextField::new(self.demo_text("in-compact", "", cx))
                        .placeholder("Filter rows")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .start_content(icon(h::icons::SEARCH, cx))
                        .into_any_element()]),
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
                    "Prefix and suffix text uses 14px text with 20px lines alongside the input.",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .prefix(h::InputAddon::new("https://"))
                        .radius(px(4.))
                        .input(
                            h::Input::new(self.demo_text("ig-usage", "", cx))
                                // v3's group example seeds the input with
                                // `defaultValue`; `value` is the controlled
                                // spelling of the same thing.
                                .default_value("example.com")
                                .placeholder("example.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Box Customisation",
                    "`padding_x` overrides the held field's exposed edges only -- a side with an addon keeps the addon's inset -- `height` resizes the single-line group, and `is_bare` drops the group's chrome.",
                    field_col(vec![h::InputGroup::new()
                        .label("Compact")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .font_family(crate::app::MONO_FONT)
                        .prefix(h::InputAddon::new("$"))
                        .input(
                            h::Input::new(self.demo_text("ig-custom-box", "", cx))
                                .placeholder("0.00"),
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
                        .input(h::Input::new(self.demo_text("ig-loading", "example", cx)))
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
                            h::Input::new(self.demo_text("ig-disabled", "example.com", cx))
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
                                .placeholder("example.com"),
                        )
                        .into_any_element()]),
                ),
                (
                    "Text Suffix",
                    field_col(vec![h::InputGroup::new()
                        .input(
                            h::Input::new(self.demo_text("ig-text-suffix", "", cx))
                                .placeholder("example"),
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
                                .placeholder("name")
                        )
                        .suffix(h::InputAddon::new("@example.com"))
                        .into_any_element()]),
                ),
                (
                    "Copy Button Suffix",
                    field_col(vec![h::InputGroup::new()
                        .label("Website")
                        .input(h::Input::new(self.demo_text("ig-copy", "example.com", cx)))
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
                    "Usage", "Custom slot content inherits 14px/20px text; the built-in digits use 18px/24px text.",
                    // v3 labels the field, explains where the code went, and
                    // splits the six slots into two groups around a separator.
                    col(vec![
                        h::Label::new("Verify account").into_any_element(),
                        muted_para("We've sent a code to a****@gmail.com", cx),
                        h::InputOTP::new(self.otp.clone())
                            .separator()
                            .slot_hover_bg(cx.colors().accent.soft())
                            .radius(px(4.))
                            .on_complete(cx.listener(|this, code: &str, _, cx| {
                                this.otp_done = code.to_owned();
                                cx.notify();
                            }))
                            .into_any_element(),
                        gpui::div()
                            .flex()
                            .items_center()
                            .gap(px(5.))
                            .child(muted_para(
                                &if done.is_empty() {
                                    "Didn't receive a code?".to_owned()
                                } else {
                                    format!("Complete: {done}")
                                },
                                cx,
                            ))
                            .child(h::Link::new("otp-resend").label("Resend").href("#"))
                            .into_any_element(),
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
                            // `value` seeds the caller's copy into the field
                            // once, which is what "controlled" means here.
                            .value(otp_typed.as_str())
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
                    "Box Customisation",
                    "`height` resizes the group and the inner field together; `padding_x` moves the input's inset without touching the steppers; `is_bare` drops the group's paint.",
                    field_col(vec![h::NumberField::new(
                        self.demo_number("nf-custom-box", 5., 0., 20., 1., cx),
                    )
                    .label("Compact")
                    .height(px(28.))
                    .padding_x(px(8.))
                    .is_bare(true)
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
                    "Format Options",
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
                            .on_change(cx.listener(|_, _v: &f64, _, cx| cx.notify()))
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
                    "Usage", "Option content uses 14px medium text with 20px lines, independent of surrounding line height.",
                    col(vec![h::RadioGroup::new("rg-usage", plans())
                        .full_width(true)
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
                    "Sizes",
                    "`size` is additive, not a v3 prop: `Md` is the pinned 16px control; `Sm` is a 14px control with a 5px dot, 12px label text and a 10px row gap.",
                    col(vec![
                        h::RadioGroup::new("rg-sz-sm", plans())
                            .size(h::RadioSize::Sm)
                            .descriptions([Some("100 messages"), Some("200 messages"), None])
                            .default_value("Free")
                            .into_any_element(),
                        h::RadioGroup::new("rg-sz-md", plans())
                            .size(h::RadioSize::Md)
                            .descriptions([Some("100 messages"), Some("200 messages"), None])
                            .default_value("Free")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Variants", "`radius(px)` rounds the control circle in place of `key_radius`; the pressed box scales the same value and the selected dot keeps its own.",
                    col(vec![
                        h::RadioGroup::new("rg-v-primary", plans())
                            .default_value("Free")
                            .radius(px(4.))
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
                    "Horizontal Orientation",
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
                    "Controlled", "The selected plan lives in the caller: the group reads `value` and the caption prints the selected plan.",
                    col(vec![
                        h::RadioGroup::new(
                            "rg-controlled",
                            vec![
                                h::RadioOption::new("Starter"),
                                h::RadioOption::new("Pro"),
                                h::RadioOption::new("Teams"),
                            ],
                        )
                        .label("Subscription plan")
                        .value(self.radio_plan.clone())
                        .on_change(cx.listener(|this, value: &SharedString, _, cx| {
                            this.radio_plan = value.clone();
                            cx.notify();
                        }))
                        .into_any_element(),
                        para(
                            &format!("Selected plan: {}", self.radio_plan),
                            cx,
                        ),
                    ]),
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
        let icons = self.demo_text("sf-icons", "HeroGPUI", cx);
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
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` reach the inner field; a 28px bare search box.",
                    field_col(vec![h::SearchField::new(self.demo_text("sf-custom-box", "", cx))
                        .label("Compact")
                        .placeholder("Search...")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .into_any_element()]),
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
                    "Labels use 14px text with 20px lines; helper and error text use 12px text with 16px lines.",
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
                    "Platform edits support composed text and paste. Enter confirms active composition before inserting a newline.",
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
                    "Labels use 14px text with 20px lines; helper and error text use 12px text with 16px lines.",
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
                    "Rows and Resizing",
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
}
