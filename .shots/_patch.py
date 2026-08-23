"""The Colors category: every v3 example on all six pages."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# --------------------------------------------------------------- ColorArea
rep("""            vec![
                (
                    "Saturation & brightness",""",
    """            vec![
                (
                    "Usage",
                    col(vec![h::ColorArea::new("ca-usage", value).into_any_element()]),
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
                    "Saturation & brightness",""")

# ------------------------------------------------------------- ColorPicker
rep("""            vec![(
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
            )],""",
    """            vec![
                (
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
                ),
                (
                    "Controlled",
                    col(vec![
                        para(
                            "The trigger's swatch and the readout below it are the same value: \\
                             the caller owns it and the picker reports each change.",
                            cx,
                        ),
                        row(vec![
                            h::ColorSwatch::new(value).into_any_element(),
                            para(&format!("Value: {}", value.to_hex()), cx),
                        ]),
                    ]),
                ),
                (
                    "With Swatches",
                    col(vec![
                        para("A preset row beside the picker, which is v3's own layout.", cx),
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
            ],""")

# ------------------------------------------------------------- ColorSlider
rep("""            vec![
                (
                    "Channels",""",
    """            vec![
                (
                    "Usage",
                    col(vec![h::ColorSlider::new(
                        "cs-usage",
                        value,
                        h::ColorChannel::Hue,
                    )
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
                    "Channels",""")

# ------------------------------------------------------------- ColorSwatch
rep("""            crate::pages::Page::ColorSwatch.import_line(),
            vec![
                (
                    "Sizes",""",
    """            crate::pages::Page::ColorSwatch.import_line(),
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
                    "Accessibility",
                    col(vec![
                        para(
                            "v3 gives a swatch an accessible colour name. gpui has no \\
                             accessibility tree, so the name is shown as a caption instead of \\
                             announced.",
                            cx,
                        ),
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
                    "Sizes",""")

# ------------------------------------------------------- ColorSwatchPicker
rep("""                (
                    "Square, stacked",""",
    """                (
                    "Variants",
                    col(vec![
                        spec(
                            "Circle (default)",
                            h::ColorSwatchPicker::new("csp-circle", palette()).value(selected),
                            cx,
                        ),
                        spec(
                            "Square",
                            h::ColorSwatchPicker::new("csp-sq", palette())
                                .value(selected)
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
                    "Stack Layout",
                    col(vec![h::ColorSwatchPicker::new("csp-stack", palette())
                        .value(selected)
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
                    "Custom Indicator",
                    col(vec![
                        para(
                            "v3 replaces `ColorSwatchPicker.Indicator`. The square shape shows \\
                             the selected item with the same tick in a different frame.",
                            cx,
                        ),
                        h::ColorSwatchPicker::new("csp-indicator", palette())
                            .value(selected)
                            .shape(h::SwatchShape::Square)
                            .size(SizeXl::Lg)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Square, stacked",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched the colors pages')
