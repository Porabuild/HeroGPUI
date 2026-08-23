"""Label & Messages, Fieldset, Form: the v3 examples for the field parts."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# ---------------------------------------------------- Label & Messages page
rep("""            crate::pages::Page::FieldSlots.import_line(),
            vec![
                (
                    "Label",""",
    """            crate::pages::Page::FieldSlots.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![
                        h::Label::new("Email").into_any_element(),
                        h::Description::new("We will never share your address.")
                            .into_any_element(),
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
                    col(vec![h::TextField::new(self.demo_text("fs-with-field", "", cx))
                        .label("Email")
                        .input_type(h::InputType::Email)
                        .description("We will never share your email")
                        .into_any_element()]),
                ),
                (
                    "Integration with TextField",
                    col(vec![
                        para(
                            "A `TextField` composes all three parts itself: the label above, the \\
                             input, and the description or the error message below.",
                            cx,
                        ),
                        h::TextField::new(self.demo_text("fs-integration", "", cx))
                            .label("Email")
                            .placeholder("Enter your email")
                            .description("We will never share your email")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Basic Validation",
                    col(vec![h::TextField::new(self.demo_text("fs-validate", "", cx))
                        .label("Password")
                        .input_type(h::InputType::Password)
                        .is_required(true)
                        .validate(|value| {
                            (value.chars().count() < 8)
                                .then(|| "Use at least 8 characters".into())
                        })
                        .into_any_element()]),
                ),
                (
                    "With Dynamic Messages",
                    col(vec![
                        para(
                            "v3's `FieldError` takes a render prop and joins \\
                             `validation.validationErrors`. `validationErrors` here is a list, \\
                             and the field shows them in order.",
                            cx,
                        ),
                        h::TextField::new(self.demo_text("fs-dynamic", "abc", cx))
                            .label("Password")
                            .is_invalid(true)
                            .validation_errors([
                                "Use at least 8 characters",
                                "Include a digit",
                            ])
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
                                && !value
                                    .chars()
                                    .all(|c| c.is_ascii_alphanumeric() || c == '-'))
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
                    "Label",""")

# ---------------------------------------------------------------- Fieldset
rep("""            crate::pages::Page::Fieldset.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::Fieldset.import_line(),
            vec![
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::Fieldset::new()
                                .child(h::FieldsetLegend::new("Profile"))
                                .child(
                                    h::FieldsetGroup::new()
                                        .child(
                                            h::TextField::new(
                                                self.demo_text("fset-name", "", cx),
                                            )
                                            .label("Name")
                                            .variant(FieldVariant::Secondary),
                                        )
                                        .child(
                                            h::TextField::new(
                                                self.demo_text("fset-email", "", cx),
                                            )
                                            .label("Email")
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
                    "Usage",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched field slots + fieldset')
