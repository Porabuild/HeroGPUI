"""NumberField page: the thirteen v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""                (
                    "Format options",
                    col(vec![h::NumberField::new(self.price.clone())
                        .label("Price")
                        .format_options(h::NumberFormat::currency("USD"))
                        .into_any_element()]),
                ),
            ],""",
    """                (
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
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-desc", 1., 0., 99., 1., cx,
                    ))
                    .label("Quantity")
                    .description("How many licences to buy")
                    .into_any_element()]),
                ),
                (
                    "Required Field",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-req", 1., 0., 99., 1., cx,
                    ))
                    .label("Quantity")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled State",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-dis", 8., 0., 99., 1., cx,
                    ))
                    .label("Quantity")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-full", 3., 0., 99., 1., cx,
                    ))
                    .label("Quantity")
                    .full_width(true)
                    .into_any_element()]),
                ),
                (
                    "Validation",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-invalid", 0., 0., 99., 1., cx,
                    ))
                    .label("Quantity")
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
                    col(vec![
                        para(
                            "v3 replaces the glyphs inside `NumberField.IncrementButton` and \\
                             `DecrementButton`. Ours draws v3's own minus and plus; the pair below \\
                             shows them with and without the steppers.",
                            cx,
                        ),
                        h::NumberField::new(self.demo_number("nf-icons", 1024., 0., 4096., 1., cx))
                            .label("Width")
                            .into_any_element(),
                        h::NumberField::new(self.demo_number("nf-noicons", 512., 0., 4096., 1., cx))
                            .label("Width (no steppers)")
                            .hide_steppers(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "With Chevrons",
                    col(vec![h::NumberField::new(self.demo_number(
                        "nf-chev", 99., 0., 999., 1., cx,
                    ))
                    .label("Amount")
                    .format_options(h::NumberFormat::currency("EUR"))
                    .into_any_element()]),
                ),
            ],""")

rep("""    pub fn page_number_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {""",
    """    pub fn page_number_field(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let nf_controlled = self
            .demo_number("nf-ctl", 5., 0., 20., 1., cx)
            .read(cx)
            .value();""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched number field page')
