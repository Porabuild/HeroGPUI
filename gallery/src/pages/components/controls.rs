//! Controls gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
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
                    "Usage", "Labels and values use 14px text with a 20px line height, independent of the surrounding text style.",
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
                            .on_change(cx.listener(|this, v: &f32, _, cx| {
                                this.slider_value = *v;
                                cx.notify();
                            })),
                    )]),
                ),
                (
                    "Range Slider Anatomy", "A range slider is built from its parts: a `Label`, an `Output`, and a `Track` whose `thumb` closure is handed the state so it can draw one thumb per value.",
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
                                .on_change(cx.listener(|this, v: &f32, _, cx| {
                                    this.set_demo_value("sl-controlled", *v);
                                    cx.notify();
                                })),
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
                    "Custom Output Display", "The output closure receives every live value and its formatted thumb label.",
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
                                            .radius(px(4.))
                                            .text_size(px(12.))
                                            .child(h::ChipLabel::new().child(format!(
                                                "{:.0}% ({})",
                                                values[0], labels[0]
                                            )))
                                            .into_any_element()
                                    })
                                    .on_change(cx.listener(|this, v: &f32, _, cx| {
                                        this.set_demo_value("sl-controlled", *v);
                                        cx.notify();
                                    })),
                            )
                            .into_any_element(),
                    ]),
                ),
                (
                    "Range",
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
                    "Range Slider", "An uncontrolled range specimen: two thumbs seeded at 100 and 500 over a 0-1000 span, stepping by 50 and formatted as US dollars.",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-range-slider", 100.)
                            .default_values([100., 500.])
                            .min_value(0.)
                            .max_value(1000.)
                            .step(50.)
                            .label("Price Range")
                            .show_value(true)
                            .format_options(herogpui_core::NumberFormat::currency("USD")),
                    )]),
                ),
                (
                    "Basic Usage", "The plainest slider: one thumb, a label, and the value readout, uncontrolled.",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-basic", 100.)
                            .min_value(0.)
                            .max_value(200.)
                            .default_value(100.)
                            .label("Storage")
                            .show_value(true),
                    )]),
                ),
                (
                    "Disabled", "A disabled slider keeps its value readable but answers no drag, wheel, or keys.",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-disabled", 30.)
                            .default_value(30.)
                            .label("Volume")
                            .show_value(true)
                            .is_disabled(true),
                    )]),
                ),
                (
                    "Sizes", "`size` is HeroGPUI's additive scale: `Md` is v3's 20px rail with its two-layer thumb, and `Sm` is a 6px pill with a single 12px knob.",
                    col(vec![
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-size-md", 40.)
                                .default_value(40.)
                                .label("Md")
                                .show_value(true),
                        ),
                        fixed_demo(
                            320.,
                            h::Slider::new("sl-size-sm", 40.)
                                .default_value(40.)
                                .size(h::SliderSize::Sm)
                                .label("Sm")
                                .show_value(true),
                        ),
                    ]),
                ),
                (
                    "Vertical Orientation", "The vertical specimen grows upward with its label attached.",
                    col(vec![fixed_demo(
                        320.,
                        h::Slider::new("sl-vert-orientation", 30.)
                            .default_value(30.)
                            .label("Volume")
                            .show_value(true)
                            .orientation(Orientation::Vertical),
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
                        .on_change(cx.listener(|this, v: &f32, _, cx| {
                            this.slider_value = *v;
                            cx.notify();
                        }))
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
                                .on_change(cx.listener(|this, v: &f32, _, cx| {
                                    this.slider_value = *v;
                                    cx.notify();
                                })),
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
                    "Usage", "Content uses 14px text with 20px lines; the built-in label uses 16px text with 24px lines.",
                    col(vec![
                        h::Switch::new("sw-a")
                            .is_selected(a)
                            .hover_bg(cx.colors().accent.soft_hover())
                            .label(gpui::div().child("Enable notifications"))
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.switch_a = *v;
                                cx.notify();
                            }))
                            .into_any_element(),
                        h::Switch::new("sw-b")
                            .is_selected(b)
                            .label(gpui::div().child("Share usage data"))
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.switch_b = *v;
                                cx.notify();
                            }))
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
                            .on_change(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("sw-controlled", *v);
                                cx.notify();
                            }))
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
                                .on_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-wifi", *v);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Switch::new("sw-g-bt")
                                .is_selected(bluetooth)
                                .label(gpui::div().child("Bluetooth"))
                                .on_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-bt", *v);
                                    cx.notify();
                                })),
                        )
                        .child(
                            h::Switch::new("sw-g-air")
                                .is_selected(airplane)
                                .label(gpui::div().child("Airplane mode"))
                                .on_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-group-air", *v);
                                    cx.notify();
                                })),
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
                                .on_change(cx.listener(|this, v: &bool, _, cx| {
                                    this.set_demo_flag("sw-form", *v);
                                    cx.notify();
                                }));
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
                        .on_change(cx.listener(|this, v: &bool, _, cx| {
                            this.set_demo_flag("sw-controlled", *v);
                            cx.notify();
                        }))
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
}
