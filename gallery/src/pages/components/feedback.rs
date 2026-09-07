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
                    wide_col(vec![h::Alert::new("New features available")
                        .description(ALERT_USAGE_DESCRIPTION)
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    wide_col(Color::ALL
                        .iter()
                        .map(|c| {
                            h::Alert::new(format!("{} alert", c.label()))
                                .description("Something worth reading happened.")
                                .status(*c)
                        })
                        .els()),
                ),
                (
                    "Closable", "There is no built-in close prop: a close affordance is an ordinary child, composed here as a `CloseButton`.",
                    wide_col(vec![
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
                    ]),
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
                    col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::Meter::new("meter-usage", value)
                                .label("Disk usage")
                                .show_value(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            fixed_demo(256., h::Meter::new(("meter-color", index), value).color(*c))
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(Size::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, s)| {
                            fixed_demo(256., h::Meter::new(("meter-size", index), value).size(*s))
                        })
                        .els()),
                ),
                (
                    "Without Label",
                    col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-no-label", value)
                    )]),
                ),
                (
                    "Custom Value Scale",
                    col(vec![fixed_demo(
                        256.,
                        h::Meter::new("meter-custom-scale", 320.)
                            .min_value(0.)
                            .max_value(500.)
                            .label("Storage")
                            .show_value(true)
                            .format_options(herogpui_core::NumberFormat::unit("GB")),
                    )]),
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
                    col(vec![gpui::div()
                        .w(px(256.))
                        .child(
                            h::ProgressBar::new("progress-usage")
                                .value(65.0)
                                .label("Uploading")
                                .show_value_label(true),
                        )
                        .into_any_element()]),
                ),
                (
                    "Colors",
                    col(Color::ALL
                        .iter()
                        .enumerate()
                        .map(|(index, c)| {
                            fixed_demo(
                                256.,
                                h::ProgressBar::new(("progress-color", index))
                                    .value(65.0)
                                    .color(*c),
                            )
                        })
                        .els()),
                ),
                (
                    "Sizes",
                    col(vec![
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-sm")
                                .value(40.0)
                                .size(Size::Sm),
                        ),
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-md")
                                .value(60.0)
                                .size(Size::Md),
                        ),
                        fixed_demo(
                            256.,
                            h::ProgressBar::new("progress-size-lg")
                                .value(80.0)
                                .size(Size::Lg),
                        ),
                    ]),
                ),
                (
                    "Without Label",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-no-label").value(65.0),
                    )]),
                ),
                (
                    "Indeterminate",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-indeterminate")
                            .is_indeterminate(true)
                            .label("Uploading"),
                    )]),
                ),
                (
                    "Custom Value Scale",
                    col(vec![fixed_demo(
                        256.,
                        h::ProgressBar::new("progress-custom-scale")
                            .value(320.0)
                            .min_value(0.0)
                            .max_value(500.0)
                            .label("Downloaded")
                            .show_value_label(true)
                            .format_options(herogpui_core::NumberFormat::unit("MB")),
                    )]),
                ),
            ],
            cx,
        )
    }

    pub fn page_progress_circle(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        component_doc_page!(
            "Progress Circle",
            crate::pages::Page::ProgressCircle.description(),
            crate::pages::Page::ProgressCircle.import_line(),
            vec![
                (
                    "Usage",
                    row(vec![h::ProgressCircle::new().value(60.).into_any_element()]),
                ),
                (
                    "Indeterminate",
                    row(vec![h::ProgressCircle::new()
                        .is_indeterminate(true)
                        .into_any_element()]),
                ),
                (
                    "With Label",
                    row(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(h::ProgressCircle::new().value(75.))
                        .child(gpui::div().text_size(px(14.)).child("75% Complete"))
                        .into_any_element()]),
                ),
                (
                    "Custom SVG Props", "The stroke keeps a fixed 4/36 view-box ratio as the circle scales; custom SVG attributes are not exposed by this canvas-backed port.",
                    col(vec![
                        row(Size::ALL
                            .iter()
                            .map(|sz| {
                                spec(
                                    sz.label(),
                                    h::ProgressCircle::new().value(60.).size(*sz),
                                    cx,
                                )
                            })
                            .collect()),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::ProgressCircle::new().value(70.0).color(*c),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(vec![
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Sm)
                            .into_any_element(),
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Md)
                            .into_any_element(),
                        h::ProgressCircle::new()
                            .value(70.0)
                            .size(Size::Lg)
                            .into_any_element(),
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
                    col(vec![gpui::div()
                        .w(px(250.))
                        .flex()
                        .flex_col()
                        .gap(px(20.))
                        .child(h::Skeleton::new().w(px(218.)).h(px(128.)))
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(12.))
                                .child(h::Skeleton::new().w(px(130.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(174.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(87.)).h(px(12.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "Text Content",
                    col(vec![gpui::div()
                        .w(px(420.))
                        .flex()
                        .flex_col()
                        .gap(px(12.))
                        .children(
                            [1.0_f32, 0.83, 0.66, 1.0, 0.5]
                                .into_iter()
                                .map(|f| h::Skeleton::new().w(px(420. * f)).h(px(16.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "User Profile",
                    col(vec![gpui::div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        // v3 rounds this one with `rounded-full`. Clipping it in
                        // the wrapper is the same result without inventing a
                        // per-instance radius prop v3 does not have.
                        .child(
                            gpui::div()
                                .rounded_full()
                                .overflow_hidden()
                                .flex_shrink_0()
                                .child(h::Skeleton::new().w(px(40.)).h(px(40.))),
                        )
                        .child(
                            gpui::div()
                                .flex()
                                .flex_col()
                                .gap(px(8.))
                                .child(h::Skeleton::new().w(px(144.)).h(px(12.)))
                                .child(h::Skeleton::new().w(px(96.)).h(px(12.))),
                        )
                        .into_any_element()]),
                ),
                (
                    "List Items",
                    col((0..3)
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
                        .els()),
                ),
                (
                    "Grid",
                    col(vec![gpui::div()
                        .flex()
                        .flex_wrap()
                        .gap(px(16.))
                        .children((0..6).map(|_| h::Skeleton::new().w(px(130.)).h(px(96.))))
                        .into_any_element()]),
                ),
                (
                    "Single Shimmer", "One shimmer runs across a whole group: the animation sits on the parent and is turned off on each child.",
                    col(vec![
                        gpui::div()
                            .flex()
                            .gap(px(16.))
                            .children((0..3).map(|_| {
                                h::Skeleton::new()
                                    .w(px(130.))
                                    .h(px(96.))
                                    .animation_type(herogpui_theme::SkeletonAnimation::None)
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Animation Types",
                    row(vec![
                        spec(
                            "Shimmer",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Shimmer),
                            cx,
                        ),
                        spec(
                            "Pulse",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::Pulse),
                            cx,
                        ),
                        spec(
                            "None",
                            h::Skeleton::new()
                                .w(px(160.))
                                .h(px(80.))
                                .animation_type(herogpui_theme::SkeletonAnimation::None),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Loading",
                    col(vec![
                        h::Skeleton::new().w(px(320.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(260.)).h(px(16.)).into_any_element(),
                        h::Skeleton::new().w(px(180.)).h(px(16.)).into_any_element(),
                    ]),
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
                    row(vec![h::Spinner::new("sp-usage").into_any_element()]),
                ),
                (
                    "Speed",
                    row(vec![
                        spec("Slow", h::Spinner::new("sp-slow").duration_ms(1500), cx),
                        spec("Default", h::Spinner::new("sp-default"), cx),
                        spec("Fast", h::Spinner::new("sp-fast").duration_ms(500), cx),
                    ]),
                ),
                (
                    "Colors",
                    row(Color::ALL
                        .iter()
                        .map(|c| {
                            spec(
                                c.label(),
                                h::Spinner::new(el_id(format!("sp-{c:?}"))).color(*c),
                                cx,
                            )
                        })
                        .collect()),
                ),
                (
                    "Sizes",
                    row(h::SpinnerSize::ALL
                        .iter()
                        .map(|s| {
                            spec(
                                s.label(),
                                h::Spinner::new(el_id(format!("sp-sz-{s:?}"))).size(*s),
                                cx,
                            )
                        })
                        .collect()),
                ),
            ],
            cx,
        )
    }

    // -----------------------------------------------------------------------
}
