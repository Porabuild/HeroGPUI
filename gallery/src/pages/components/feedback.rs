//! Feedback gallery pages.
#![allow(clippy::redundant_clone)]

use super::*;

impl Gallery {
    // Feedback
    // -----------------------------------------------------------------------

    pub fn page_alert(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Alert",
            crate::pages::Page::Alert.description(),
            crate::pages::Page::Alert.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("alert-main", wide_col(vec![h::Alert::new("New features available")
                        .description(ALERT_USAGE_DESCRIPTION)
                        .radius(px(8.))
                        .into_any_element()]), cx),
                ),
                (
                    "Colors",
                    wide_col(Color::ALL
                        .iter()
                        .map(|c| {
                            let key = format!("alert-color-{c:?}");
                            specimen_body(&key, h::Alert::new(format!("{} alert", c.label()))
                                .description("Something worth reading happened.")
                                .status(*c)
                                .into_any_element(), cx)
                        })
                        .els()),
                ),
                (
                    "Custom Indicator",
                    specimen_body(
                        "alert-custom-indicator",
                        h::Alert::new("Review required")
                            .description("A caller-owned warning mark stays inside the pinned indicator box.")
                            .status(Color::Warning)
                            .indicator(icon(h::icons::INFO_CIRCLE, cx))
                            .into_any_element(),
                        cx,
                    ),
                ),
                (
                    "Closable", "There is no built-in close prop: a close affordance is an ordinary child, composed here as a `CloseButton`.",
                    specimen_body("alert-closable", wide_col(vec![
                        if self.alert_visible {
                            h::Alert::new("Saved")
                                .description("Your changes are live.")
                                .status(Color::Success)
                                .child(h::CloseButton::new("alert-closable-close").on_press(
                                    cx.listener(|this, _, _, cx| {
                                        this.alert_visible = false;
                                        cx.notify();
                                    }),
                                ))
                                .into_any_element()
                        } else {
                            h::Button::new("alert-restore")
                                .label("Bring it back")
                                .variant(Variant::Tertiary)
                                .size(Size::Sm)
                                .on_press(cx.listener(|this, _, _, cx| {
                                    this.alert_visible = true;
                                    cx.notify();
                                }))
                                .into_any_element()
                        },
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_meter(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let value = self.meter_value;
        component_doc_page!(
            "Meter",
            crate::pages::Page::Meter.description(),
            crate::pages::Page::Meter.import_line(),
            vec![
                (
                    "Usage", "Labels and values use 14px text with a 20px line height, independent of the surrounding text style.",
                    specimen_body("meter-main", col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::Meter::new("meter-usage", value)
                                .label("Disk usage")
                                .radius(px(4.))
                                .show_value(true),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            let key = format!("meter-color-{c:?}");
                            specimen_body(&key, fixed_demo(256., h::Meter::new(("meter-color", index), value).color(*c)), cx)
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, s)| {
                            let key = format!("meter-size-{s:?}");
                            specimen_body(&key, fixed_demo(256., h::Meter::new(("meter-size", index), value).size(*s)), cx)
                        })
                        .els()),
                ),
                (
                    "Without Label",
                    specimen_body("meter-no-label", col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-no-label", value)
                    )]), cx),
                ),
                (
                    "Custom Value Scale",
                    specimen_body("meter-custom-scale", col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-custom-scale", 320.)
                            .min_value(0.)
                            .max_value(500.)
                            .label("Storage")
                            .show_value(true)
                            .format_options(herogpui_core::NumberFormat::unit("GB")),
                    )]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_bar(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Progress Bar",
            crate::pages::Page::ProgressBar.description(),
            crate::pages::Page::ProgressBar.import_line(),
            vec![
                (
                    "Usage", "Labels and values use 14px text with a 20px line height, independent of the surrounding text style.",
                    specimen_body("progress-main", col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::ProgressBar::new("progress-usage")
                                .value(65.0)
                                .label("Uploading")
                                .radius(px(4.))
                                .show_value_label(true),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            let key = format!("progress-color-{c:?}");
                            specimen_body(&key, fixed_demo(
                                256.,
                                h::ProgressBar::new(("progress-color", index))
                                    .value(65.0)
                                    .color(*c),
                            ), cx)
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(vec![
                        specimen_body("progress-size-Sm", fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-sm")
                                .value(40.0)
                                .size(Size::Sm),
                        ), cx),
                        specimen_body("progress-size-Md", fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-md")
                                .value(60.0)
                                .size(Size::Md),
                        ), cx),
                        specimen_body("progress-size-Lg", fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-lg")
                                .value(80.0)
                                .size(Size::Lg),
                        ), cx),
                    ]),
                ),
                (
                    "Without Label",
                    specimen_body("progress-no-label", col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-no-label").value(65.0),
                    )]), cx),
                ),
                (
                    "Indeterminate",
                    specimen_body("progress-indeterminate", col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-indeterminate")
                            .is_indeterminate(true)
                            .label("Uploading"),
                    )]), cx),
                ),
                (
                    "Custom Value Scale",
                    specimen_body("progress-custom-scale", col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-custom-scale")
                            .value(320.0)
                            .min_value(0.0)
                            .max_value(500.0)
                            .label("Downloaded")
                            .show_value_label(true)
                            .format_options(herogpui_core::NumberFormat::unit("MB")),
                    )]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_circle(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let circle_transition = self.demo_flag("progress-circle-transition", false);
        let circle_value = if circle_transition { 85.0 } else { 25.0 };
        component_doc_page!(
            "Progress Circle",
            crate::pages::Page::ProgressCircle.description(),
            crate::pages::Page::ProgressCircle.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("progress-circle-main", row(vec![h::ProgressCircle::new().value(60.).into_any_element()]), cx),
                ),
                (
                    "Value Transition",
                    "Determinate value changes interpolate the retained arc over HeroUI's 300ms ease-out stroke transition; reduced motion settles directly.",
                    specimen_body("progress-circle-transition", row(vec![
                        h::ProgressCircle::new()
                            .id("progress-circle-transition-ring")
                            .value(circle_value)
                            .into_any_element(),
                        h::Button::new("progress-circle-transition-toggle")
                            .label(if circle_transition { "Show 25%" } else { "Show 85%" })
                            .size(Size::Sm)
                            .variant(Variant::Secondary)
                            .on_press(cx.listener(move |this, _, _, cx| {
                                this.set_demo_flag("progress-circle-transition", !circle_transition);
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Indeterminate",
                    specimen_body("progress-circle-indeterminate", row(vec![h::ProgressCircle::new()
                        .is_indeterminate(true)
                        .into_any_element()]), cx),
                ),
                (
                    "With Label",
                    specimen_body("progress-circle-label", row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(h::ProgressCircle::new().value(75.))
                        .child(gpui::div().text_size(px(14.)).child("75% Complete"))
                        .into_any_element()]), cx),
                ),
                (
                    "Custom SVG Props", "The stroke keeps a fixed 4/36 view-box ratio as the circle scales; custom SVG attributes are not exposed by this canvas-backed port.",
                    col(vec![
                        row(Size::ALL
                            .iter()
                            .map(|sz| {
                                {
                                    let key = format!("progress-circle-size-{sz:?}");
                                    specimen_body(&key, spec(
                                        sz.label(),
                                        h::ProgressCircle::new().value(60.).size(*sz),
                                        cx,
                                    ), cx)
                                }
                            })
                            .collect()),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                                {
                                    let key = format!("progress-circle-color-{c:?}");
                                    specimen_body(&key, spec(
                                        c.label(),
                                        h::ProgressCircle::new().value(70.0).color(*c),
                                        cx,
                                    ), cx)
                                }
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(vec![
                        specimen_body("progress-circle-size-Sm", h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Sm)
                            .into_any_element(), cx),
                        specimen_body("progress-circle-size-Md", h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Md)
                            .into_any_element(), cx),
                        specimen_body("progress-circle-size-Lg", h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Lg)
                            .into_any_element(), cx),
                    ]),
                ),
            ],
            cx,
        )
    }

    pub fn page_skeleton(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Skeleton",
            crate::pages::Page::Skeleton.description(),
            crate::pages::Page::Skeleton.import_line(),
            vec![
                (
                    "Usage",
                    specimen_body("skeleton-main", col(vec![gpui::div()
                        .w(px(250.))
                        .flex()
                        .flex_col()
                        .gap(px(20.))
                        .child(h::Skeleton::new().w(px(218.)).h(px(128.)).radius(px(8.)))
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(12.))
                                .child(h::Skeleton::new().w(px(130.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(174.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(87.)).h(px(12.))),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "Text Content",
                    specimen_body("skeleton-text", col(vec![gpui::div()
                        .w(px(420.))
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .children(
                            [1.0_f32, 0.83, 0.66, 1.0, 0.5]
                                .into_iter()
                                .map(|f| h::Skeleton::new().w(px(420. * f)).h(px(16.))),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "User Profile",
                    specimen_body("skeleton-profile", col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        // v3 rounds this one with `rounded-full`. A clipping
                        // wrapper does not round it on vanilla GPUI (the clip
                        // is rectangular), so the skeleton carries the radius
                        // itself: half its size is `rounded-full`.
                        .child(
                            gpui::div()
                                .flex_shrink_0()
                                .child(h::Skeleton::new().w(px(40.)).h(px(40.)).radius(px(20.))),
                        )
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(h::Skeleton::new().w(px(144.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(96.)).h(px(12.))),
                        )
                        .into_any_element()]), cx),
                ),
                (
                    "List Items",
                    specimen_body("skeleton-list", col((0..3)
                        .map(|_| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(12.))
                                .child(h::Skeleton::new().w(px(40.)).h(px(40.)))
                                .child(
                                    gpui::div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.))
                                        .child(h::Skeleton::new().w(px(320.)).h(px(12.)))
                                        .child(h::Skeleton::new().w(px(256.)).h(px(12.))),
                                )
                        })
                        .els()), cx),
                ),
                (
                    "Grid",
                    specimen_body("skeleton-grid", col(vec![gpui::div()
                        .flex()
                        .flex_wrap()
                        .gap(px(16.))
                        .children((0..6).map(|_| h::Skeleton::new().w(px(130.)).h(px(96.))))
                        .into_any_element()]), cx),
                ),
                (
                    "Single Shimmer", "One shimmer runs across a whole group: the animation sits on the parent and is turned off on each child.",
                    specimen_body("skeleton-single-shimmer", col(vec![
                        h::Skeleton::new()
                            .w(px(422.))
                            .h(px(96.))
                            .animation_type(herogpui_theme::SkeletonAnimation::Shimmer)
                            .child(
                                gpui::div()
                                    .flex()
                                    .gap(px(16.))
                                    .children((0..3).map(|_| {
                                        h::Skeleton::new()
                                            .w(px(130.))
                                            .h(px(96.))
                                            .animation_type(herogpui_theme::SkeletonAnimation::None)
                                    })),
                            )
                            .into_any_element(),
                    ]), cx),
                ),
                (
                    "Animation Types",
                    row(vec![
                        specimen_body("skeleton-animation-Shimmer", spec(
                            "Shimmer",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Shimmer),
                            cx,
                        ), cx),
                        specimen_body("skeleton-animation-Pulse", spec(
                            "Pulse",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Pulse),
                            cx,
                        ), cx),
                        specimen_body("skeleton-animation-None", spec(
                            "None",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::None),
                            cx,
                        ), cx),
                    ]),
                ),
                (
                    "Loading",
                    specimen_body("skeleton-loading", col(vec![
                        h::Skeleton::new().w(px(320.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(260.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(180.)).h(px(16.)).into_any_element(),
                    ]), cx),
                ),
            ],
            cx,
        )
    }

    pub fn page_spinner(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Spinner",
            crate::pages::Page::Spinner.description(),
            crate::pages::Page::Spinner.import_line(),
            vec![
                (
                    "Usage",
                    "The indicator keeps its diameter in flex layouts. Turn Motion off to stop rotation.",
                    specimen_body("spinner-main", row(vec![h::Spinner::new("sp-usage").into_any_element()]), cx),
                ),
                (
                    "Speed",
                    row(vec![
                        specimen_body("spinner-speed-Slow", spec("Slow", h::Spinner::new("sp-slow").duration_ms(1500), cx), cx),
                        specimen_body("spinner-speed-Default", spec("Default", h::Spinner::new("sp-default"), cx), cx),
                        specimen_body("spinner-speed-Fast", spec("Fast", h::Spinner::new("sp-fast").duration_ms(500), cx), cx),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            {
                                let key = format!("spinner-color-{c:?}");
                                specimen_body(&key, spec(
                                c.label(),
                                h::Spinner::new(el_id(format!("sp-{c:?}"))).color(*c),
                                cx,
                                ), cx)
                            }
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(h::SpinnerSize::ALL
                        .iter()
                        .map(|s| {
                            {
                                let key = format!("spinner-size-{s:?}");
                                specimen_body(&key, spec(
                                s.label(),
                                h::Spinner::new(el_id(format!("sp-sz-{s:?}"))).size(*s),
                                cx,
                                ), cx)
                            }
                        })
                        .collect()),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
