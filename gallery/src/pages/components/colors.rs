//! Colors gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
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
                        .radius(px(4.))
                        .into_any_element()]),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` cover both paths: the editable field here, and the static display box. The bare specimen is the editable path.",
                    field_col(vec![h::ColorField::new("cf-custom-box", value)
                        .state(self.demo_text("cf-custom-box", "#0085F5", cx))
                        .label("Compact editable")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
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
                    gpui::div()
                        .w(px(400.))
                        .child(col(vec![
                            h::ColorField::new("cf-full", value)
                                .state(self.demo_text("cf-full", "#0085F5", cx))
                                .label("Color")
                                .full_width(true)
                                .into_any_element(),
                            h::ColorField::new("cf-full-display", value)
                                .label("Display color")
                                .full_width(true)
                                .into_any_element(),
                        ]))
                        .into_any_element(),
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
                            .on_change(cx.listener(
                                |this, parsed: &Option<h::PickerColor>, _, cx| {
                                    if let Some(c) = parsed {
                                        this.picker_color = *c;
                                    }
                                    cx.notify();
                                },
                            ))
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
                        .on_change(cx.listener(|this, parsed: &Option<h::PickerColor>, _, cx| {
                            // Unparseable text reports None; the swatch
                            // holds its last good colour.
                            if let Some(c) = parsed {
                                this.picker_color = *c;
                            }
                            cx.notify();
                        },))
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
                    "The panel flips near window edges and scrolls to keep the alpha control reachable in short windows.",
                    col(vec![h::ColorPicker::new("cp-main", value)
                        // v3's Usage is uncontrolled; "Controlled" is separate.
                        .default_value(value)
                        .label("Accent")
                        .font_family(crate::app::MONO_FONT)
                        .show_alpha(true)
                        .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.picker_color = *c;
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Controlled", "The caller owns the color value while the trigger owns its ordinary open state — the same split a dialog trigger composes.",
                    col(vec![
                        h::ColorPicker::new("cp-controlled", value)
                            .label("Brand")
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            },))
                            .into_any_element(),
                        para(&format!("Value: {}", value.to_hex()), cx),
                    ]),
                ),
                (
                    "With Swatches", "A preset row beside the picker — the default layout.",
                    col(vec![
                        h::ColorSwatchPicker::new("cp-presets", palette())
                            .value(value)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                        h::ColorSlider::new("cp-sl-alpha", value, h::ColorChannel::Alpha)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
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
                                .on_change(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.picker_color = *c;
                                        cx.notify();
                                    },
                                ))
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
                    "RGB Channels",
                    col([
                        h::ColorChannel::Red,
                        h::ColorChannel::Green,
                        h::ColorChannel::Blue,
                    ]
                    .iter()
                    .map(|ch| {
                        h::ColorSlider::new(el_id(format!("cs-rgb-{ch:?}")), value, *ch).on_change(
                            cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }),
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
                    "Accessibility", "A swatch has an accessible colour name. A desktop GPUI surface has no screen-reader tree, so the name is shown as a caption instead of announced.",
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
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
                                .on_change(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                )),
                            cx,
                        ),
                        spec(
                            "Square",
                            h::ColorSwatchPicker::new("csp-sq", palette())
                                .value(selected)
                                .on_change(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                ))
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
                                .on_change(cx.listener(
                                    |this, c: &h::PickerColor, _, cx| {
                                        this.swatch_selected = *c;
                                        cx.notify();
                                    },
                                ))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Stack Layout",
                    col(vec![h::ColorSwatchPicker::new("csp-stack", palette())
                        .value(selected)
                        .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.swatch_selected = *c;
                            cx.notify();
                        }))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Selected: {}", selected.to_hex()), cx),
                    ]),
                ),
                (
                    "Custom Indicator", "`indicator` replaces the built-in selected checkmark. The square picker below uses a heart.",
                    col(vec![
                        h::ColorSwatchPicker::new("csp-indicator", palette())
                            .value(selected)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
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
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
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
                        .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.swatch_selected = *c;
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
