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
                    specimen_body(
                        "ca-main",
                        col(vec![h::ColorArea::new("ca-usage", value)
                            .default_value(value)
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "With Dots",
                    specimen_body(
                        "ca-dots",
                        col(vec![h::ColorArea::new("ca-dots", value)
                            .show_dots(true)
                            .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Color Space & Channels",
                    specimen_body(
                        "ca-color-channels",
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
                        cx
                    ),
                ),
                (
                    "Controlled",
                    specimen_body(
                        "ca-controlled",
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
                        cx
                    ),
                ),
                (
                    "Render Function",
                    specimen_body(
                        "ca-render",
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
                        cx
                    ),
                ),
                (
                    "Saturation & brightness",
                    specimen_body(
                        "ca-saturation-brightness",
                        col(vec![
                            h::ColorArea::new("ca-main", value)
                                .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                    this.picker_color = *c;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(&format!("Value: {}", value.to_hex()), cx),
                        ]),
                        cx
                    ),
                ),
                (
                    "Disabled",
                    specimen_body(
                        "ca-disabled",
                        col(vec![h::ColorArea::new("ca-disabled", value)
                            .size(px(180.), px(120.))
                            .is_disabled(true)
                            .into_any_element()]),
                        cx
                    ),
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
                    specimen_body("cf-main", field_col(vec![h::ColorField::new("cf-usage", value)
                        .state(self.demo_text("cf-usage", "#0085F5", cx))
                        // v3's Usage is uncontrolled: `defaultValue="#0085F5"`.
                        .default_value(value)
                        .label("Color")
                        .radius(px(4.))
                        .into_any_element()]), cx),
                ),
                (
                    "Empty Value",
                    "ColorField accepts an explicit null value. The empty field keeps its label and placeholder, omits the swatch, and can be filled later.",
                    specimen_body("cf-empty", field_col(vec![h::ColorField::new("cf-empty", None::<h::PickerColor>)
                        .default_value(None::<h::PickerColor>)
                        .state(self.demo_text("cf-empty", "", cx))
                        .label("Brand color")
                        .placeholder("#000000")
                        .is_required(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Box Customisation",
                    "`height`, `padding_x` and `is_bare` cover both paths: the editable field here, and the static display box. The bare specimen is the editable path.",
                    specimen_body("cf-box", field_col(vec![h::ColorField::new("cf-custom-box", value)
                        .state(self.demo_text("cf-custom-box", "#0085F5", cx))
                        .label("Compact editable")
                        .height(px(28.))
                        .padding_x(px(8.))
                        .is_bare(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Variants",
                    specimen_body("cf-variants", col(vec![
                        h::ColorField::new("cf-v-primary", value)
                            .state(self.demo_text("cf-v-primary", "#0085F5", cx))
                            .label("Primary")
                            .into_any_element(),
                        h::ColorField::new("cf-v-secondary", value)
                            .state(self.demo_text("cf-v-secondary", "#0085F5", cx))
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "On Surface",
                    specimen_body("cf-surface", col(vec![h::Surface::new()
                        .padding(px(24.))
                        .gap(px(16.))
                        .child(
                            h::ColorField::new("cf-surface", value)
                                .state(self.demo_text("cf-surface", "#0085F5", cx))
                                .label("Color")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "With Description",
                    specimen_body("cf-description", field_col(vec![h::ColorField::new("cf-desc", value)
                        .state(self.demo_text("cf-desc", "#0085F5", cx))
                        .label("Brand color")
                        .description("Any CSS hex value")
                        .into_any_element()]), cx),
                ),
                (
                    "Required Field",
                    specimen_body("cf-required", field_col(vec![h::ColorField::new("cf-req", value)
                        .state(self.demo_text("cf-req", "", cx))
                        .label("Color")
                        .is_required(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled State",
                    specimen_body("cf-disabled", field_col(vec![h::ColorField::new("cf-dis", value)
                        .state(self.demo_text("cf-dis", "#0085F5", cx))
                        .label("Color")
                        .is_disabled(true)
                        .into_any_element()]), cx),
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
                    specimen_body("cf-validation", field_col(vec![h::ColorField::new("cf-invalid", value)
                        .state(self.demo_text("cf-invalid", "not-a-color", cx))
                        .label("Color")
                        .is_required(true)
                        .is_invalid(true)
                        .validation_errors(["Enter a valid hex colour"])
                        .into_any_element()]), cx),
                ),
                (
                    "Channel Editing",
                    "Edit individual HSL channels:",
                    specimen_body("cf-channel-editing", col(vec![spec_row(vec![
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
                    ]),]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("cf-controlled", col(vec![
                        h::ColorField::new("cf-ctl", value)
                            .state(self.demo_text("cf-ctl", "#0085F5", cx))
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
                    ]), cx),
                ),
                (
                    "Render Function",
                    specimen_body("cf-render", col(vec![{
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
                    }]), cx),
                ),
                (
                    "Form Example",
                    specimen_body("cf-form", col(vec![{
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
                    }]), cx),
                ),
                (
                    "Hex value",
                    specimen_body("cf-hex", field_col(vec![h::ColorField::new("cf-hex", value)
                        .state(self.demo_text("cf-hex", "#0085F5", cx))
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
                        .into_any_element()]), cx),
                ),
                (
                    "Read-only display",
                    specimen_body("cf-readonly", field_col(vec![h::ColorField::new("cf-display", value)
                        .label("Current value")
                        .description("Without a text state the field is a display.")
                        .into_any_element()]), cx),
                ),
                (
                    "Single channel",
                    specimen_body("cf-single-channel", row(vec![
                        h::ColorField::new("cf-hue", value)
                            .channel(h::ColorChannel::Hue)
                            .label("Hue")
                            .into_any_element(),
                        h::ColorField::new("cf-red", value)
                            .channel(h::ColorChannel::Red)
                            .label("Red")
                            .into_any_element(),
                    ]), cx),
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
                    specimen_body("cp-main", col(vec![h::ColorPicker::new("cp-main", value)
                        // v3's Usage is uncontrolled; "Controlled" is separate.
                        .default_value(value)
                        .label("Accent")
                        .font_family(crate::app::MONO_FONT)
                        .show_alpha(true)
                        .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.picker_color = *c;
                            cx.notify();
                        }))
                        .into_any_element()]), cx),
                ),
                (
                    "Open / Placement",
                    "A controlled-open picker exercises the initial overlay mount and placement resolver without requiring a trigger press.",
                    specimen_body("cp-open", col(vec![h::ColorPicker::new("cp-open", value)
                        .is_open(true)
                        .placement(h::Placement::BottomStart)
                        .label("Accent")
                        .font_family(crate::app::MONO_FONT)
                        .show_alpha(true)
                        // Controlled open state intentionally ignores close
                        // requests so the initial placement stays visible in
                        // the visual specimen.
                        .on_open_change(|_, _, _| {})
                        .into_any_element()]), cx),
                ),
                (
                    "Controlled", "The caller owns the color value while the trigger owns its ordinary open state — the same split a dialog trigger composes.",
                    specimen_body("cp-controlled", col(vec![
                        h::ColorPicker::new("cp-controlled", value)
                            .label("Brand")
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            },))
                            .into_any_element(),
                        para(&format!("Value: {}", value.to_hex()), cx),
                    ]), cx),
                ),
                (
                    "With Swatches", "A preset row beside the picker — the default layout.",
                    specimen_body("cp-swatches", col(vec![
                        h::ColorSwatchPicker::new("cp-presets", palette())
                            .value(value)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.picker_color = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "With Fields",
                    specimen_body("cp-fields", col(vec![
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
                    ]), cx),
                ),
                (
                    "With Sliders",
                    specimen_body("cp-sliders", col(vec![
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
                    ]), cx),
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
                    specimen_body(
                        "cs-main",
                        col(vec![h::ColorSlider::new(
                            "cs-usage",
                            value,
                            h::ColorChannel::Hue,
                        )
                        .default_value(value)
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Disabled",
                    specimen_body(
                        "cs-disabled",
                        col(vec![h::ColorSlider::new(
                            "cs-disabled",
                            value,
                            h::ColorChannel::Hue,
                        )
                        .is_disabled(true)
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Vertical",
                    specimen_body(
                        "cs-vertical",
                        row(vec![h::ColorSlider::new(
                            "cs-vertical",
                            value,
                            h::ColorChannel::Hue,
                        )
                        .orientation(Orientation::Vertical)
                        .length(px(160.))
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "Controlled",
                    specimen_body(
                        "cs-controlled",
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
                        cx
                    ),
                ),
                (
                    "Render Function",
                    specimen_body(
                        "cs-render",
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
                        cx
                    ),
                ),
                (
                    "Alpha Channel",
                    specimen_body(
                        "cs-alpha",
                        col(vec![h::ColorSlider::new(
                            "cs-alpha",
                            value,
                            h::ColorChannel::Alpha,
                        )
                        .show_label(true)
                        .into_any_element()]),
                        cx
                    ),
                ),
                (
                    "HSL Channels",
                    specimen_body(
                        "cs-hsl",
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
                        cx
                    ),
                ),
                (
                    "Channels",
                    specimen_body(
                        "cs-channels",
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
                        cx
                    ),
                ),
                (
                    "RGB Channels",
                    specimen_body(
                        "cs-rgb",
                        col([
                            h::ColorChannel::Red,
                            h::ColorChannel::Green,
                            h::ColorChannel::Blue,
                        ]
                        .iter()
                        .map(|ch| {
                            h::ColorSlider::new(el_id(format!("cs-rgb-{ch:?}")), value, *ch)
                                .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                    this.picker_color = *c;
                                    cx.notify();
                                }))
                        })
                        .els()),
                        cx
                    ),
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
                    specimen_body("csw-main", row(vec![h::ColorSwatch::new(
                        h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
                    )
                    .into_any_element()]), cx),
                ),
                (
                    "Transparency",
                    specimen_body("csw-transparency", row(vec![
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
                    ]), cx),
                ),
                (
                    "Accessibility", "A swatch has an accessible colour name. A desktop GPUI surface has no screen-reader tree, so the name is shown as a caption instead of announced.",
                    specimen_body("csw-accessibility", col(vec![
                        row(palette()
                            .into_iter()
                            .map(|c| {
                                let hex = c.to_hex();
                                spec(&hex, h::ColorSwatch::new(c), cx)
                            })
                            .collect()),
                        row(vec![spec(
                            "Brand blue (color_name)",
                            h::ColorSwatch::new(
                                h::PickerColor::from_hex("#0085F5").unwrap_or_default(),
                            )
                            .id("csw-brand-blue")
                            .color_name("Brand blue"),
                            cx,
                        )]),
                    ]), cx),
                ),
                (
                    "Sizes",
                    specimen_body("csw-sizes", row(SizeXl::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::ColorSwatch::new(self.picker_color).size(*s),
                                cx,
                            )
                        })
                        .collect()), cx),
                ),
                (
                    "Shapes",
                    specimen_body("csw-shapes", row(h::SwatchShape::ALL
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
                        .collect()), cx),
                ),
                (
                    "Palette",
                    specimen_body("csw-palette", row(palette()
                        .into_iter()
                        .map(|c| h::ColorSwatch::new(c).size(SizeXl::Lg))
                        .els()), cx),
                ),
                (
                    "Alpha",
                    specimen_body("csw-alpha", row(vec![
                        h::ColorSwatch::new(self.picker_color.with_alpha(1.0))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                        h::ColorSwatch::new(self.picker_color.with_alpha(0.5))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                        h::ColorSwatch::new(self.picker_color.with_alpha(0.15))
                            .size(SizeXl::Lg)
                            .into_any_element(),
                    ]), cx),
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
                    specimen_body("csp-main", col(vec![
                        h::ColorSwatchPicker::new("csp-main", palette())
                            .value(selected)
                            .size(SizeXl::Lg)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Selected: {}", selected.to_hex()), cx),
                    ]), cx),
                ),
                (
                    "Variants",
                    specimen_body("csp-variants", col(vec![
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
                    ]), cx),
                ),
                (
                    "Sizes",
                    specimen_body("csp-sizes", col(SizeXl::ALL
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
                        .els()), cx),
                ),
                (
                    "Disabled",
                    specimen_body("csp-disabled", col(vec![h::ColorSwatchPicker::new("csp-disabled", palette())
                        .value(selected)
                        .is_disabled(true)
                        .into_any_element()]), cx),
                ),
                (
                    "Disabled Item", "`ColorSwatchPicker.Item.isDisabled` dims one swatch — unclickable and out of the tab order — while the rest stay pickable: the difference from the whole-picker `isDisabled` above.",
                    specimen_body("csp-disabled-item", col(vec![
                        h::ColorSwatchPicker::new("csp-disabled-item", palette())
                            .value(selected)
                            .disabled_keys([2])
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Stack Layout",
                    specimen_body("csp-stack", col(vec![h::ColorSwatchPicker::new("csp-stack", palette())
                        .value(selected)
                        .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                            this.swatch_selected = *c;
                            cx.notify();
                        }))
                        .layout(h::SwatchLayout::Stack)
                        .into_any_element()]), cx),
                ),
                (
                    "Default Value",
                    specimen_body("csp-default", col(vec![h::ColorSwatchPicker::new("csp-default", palette())
                        .default_value(palette()[2])
                        .into_any_element()]), cx),
                ),
                (
                    "Controlled",
                    specimen_body("csp-controlled", col(vec![
                        h::ColorSwatchPicker::new("csp-controlled", palette())
                            .value(selected)
                            .on_change(cx.listener(|this, c: &h::PickerColor, _, cx| {
                                this.swatch_selected = *c;
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Selected: {}", selected.to_hex()), cx),
                    ]), cx),
                ),
                (
                    "Custom Indicator", "`indicator` replaces the built-in selected checkmark. The square picker below uses a heart.",
                    specimen_body("csp-indicator", col(vec![
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
                    ]), cx),
                ),
                (
                    "Item Render State", "`item_content` receives each item's color plus selected, hovered, pressed, focused, focus-visible, and disabled state. These custom tiles use that state while the picker keeps navigation and selection.",
                    specimen_body("csp-item-render", col(vec![
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
                    ]), cx),
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
