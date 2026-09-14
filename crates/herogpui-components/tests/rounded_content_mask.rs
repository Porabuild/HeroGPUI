//! Framework-level rounded overflow regression coverage.
//!
//! `overflow_hidden()` is implemented by gpui's shared content mask. This
//! test keeps the radius on that mask when nested surfaces intersect so every
//! component benefits from the same renderer fix.

use gpui::{point, prelude::*, px, Bounds, BoxShadow, ContentMask, Corners, TestAppContext};

mod harness;

#[test]
fn nested_rounded_masks_keep_the_original_curve() {
    let outer = ContentMask {
        bounds: Bounds::from_corners(point(px(0.), px(0.)), point(px(100.), px(100.))),
        corner_radii: Corners::all(px(16.)),
        ..Default::default()
    };
    let inset = ContentMask {
        bounds: Bounds::from_corners(point(px(4.), px(4.)), point(px(96.), px(96.))),
        ..Default::default()
    };

    let intersection = outer.intersect(&inset);
    assert_eq!(intersection.bounds.origin, point(px(4.), px(4.)));
    assert_eq!(intersection.rounded_clips[0].bounds, outer.bounds);
    assert!(intersection.contains(point(px(4.5), px(8.5))));
}

#[gpui::test]
fn rounded_overflow_mask_reaches_child_quads(cx: &mut TestAppContext) {
    let cx = harness::open_host(cx, || {
        gpui::div()
            .w(px(100.))
            .h(px(100.))
            .rounded(px(16.))
            .overflow_hidden()
            .child(gpui::div().size_full().bg(gpui::blue()))
            .into_any_element()
    });

    let (quads, clips) = cx.update(|window, _| (window.painted_quads(), window.painted_clips()));
    let child = quads
        .iter()
        .find(|quad| quad.content_mask.clip_index > 0)
        .expect("the nested child background should be painted");
    assert_eq!(
        clips[child.content_mask.clip_index as usize - 1]
            .radii_x
            .top_left,
        px(16.).scale(2.)
    );
}

#[gpui::test]
fn spread_shadows_keep_the_control_curve(cx: &mut TestAppContext) {
    let host = harness::open_host(cx, || {
        gpui::div()
            .size(px(20.))
            .rounded(px(8.))
            .shadow(vec![BoxShadow {
                color: gpui::blue(),
                offset: point(px(0.), px(0.)),
                blur_radius: px(1.),
                spread_radius: px(4.),
                inset: false,
            }])
            .into_any_element()
    });
    let (shadows, scale) =
        host.update(|window, _| (window.painted_shadows(), window.scale_factor()));
    let shadow = shadows
        .iter()
        .find(|shadow| shadow.element_bounds.size == gpui::size(px(20.), px(20.)).scale(scale))
        .expect("the rounded control shadow should be painted");
    assert_eq!(shadow.corner_radii, Corners::all(px(12.).scale(scale)));
    assert_eq!(
        shadow.element_corner_radii,
        Corners::all(px(8.).scale(scale))
    );
}

#[gpui::test]
fn color_area_rgb_child_quads_carry_rounded_mask(cx: &mut TestAppContext) {
    use herogpui_components::{ColorArea, ColorChannel, ColorSpace, PickerColor};

    let value = PickerColor::hsb(0.58, 0.82, 0.94);
    let cx = harness::open_host(cx, move || {
        ColorArea::new("mask-rgb", value)
            .color_space(ColorSpace::Rgb)
            .x_channel(ColorChannel::Red)
            .y_channel(ColorChannel::Green)
            .size(px(160.), px(120.))
            .into_any_element()
    });
    let (quads, clips) = cx.update(|window, _| (window.painted_quads(), window.painted_clips()));
    let clipped_strips: Vec<_> = quads
        .iter()
        .filter(|quad| {
            quad.bounds.size.width.0 < 20.0
                && quad.bounds.size.height.0 > 200.0
                && quad.content_mask.clip_index > 0
                && clips[quad.content_mask.clip_index as usize - 1]
                    .radii_x
                    .top_left
                    .0
                    > 0.0
        })
        .collect();
    assert!(
        clipped_strips.len() >= 32,
        "RGB gradient strips must inherit the rounded overflow mask"
    );
}

#[test]
fn color_area_border_overlay_is_painted_before_the_thumb() {
    let source = include_str!("../src/color_picker/area.rs");
    let border = source
        .find(".border_color(border_color)")
        .expect("ColorArea must keep its visible border overlay");
    let thumb = source
        .find("let mut thumb = div()")
        .expect("ColorArea must build its thumb after the surface layers");
    assert!(
        border < thumb,
        "the visible border must be a child painted before the thumb so it cannot cover the thumb"
    );
    assert!(
        source.contains("GPUI paints an element's border after its children"),
        "the paint-order workaround must remain documented at its root"
    );
}

#[gpui::test]
fn color_slider_inset_edges_stay_below_the_thumb_without_an_extra_outline(cx: &mut TestAppContext) {
    use herogpui_components::{ColorChannel, ColorSlider, PickerColor};
    let host = harness::open_host(cx, || {
        ColorSlider::new(
            "edge-shadows",
            PickerColor::hsb(207., 0.9, 0.9),
            ColorChannel::Hue,
        )
        .length(px(240.))
        .show_label(false)
        .into_any_element()
    });
    let (quads, shadows, scale) = host.update(|window, _| {
        (
            window.painted_quads(),
            window.painted_shadows(),
            window.scale_factor(),
        )
    });
    let track_size = gpui::size(px(240.), px(20.)).scale(scale);
    assert!(!quads
        .iter()
        .any(|quad| quad.bounds.size == track_size && quad.border_widths.top.0 > 0.));
    let thumb = quads
        .iter()
        .find(|quad| {
            quad.bounds.size == gpui::size(px(16.), px(16.)).scale(scale)
                && quad.border_widths.top == px(3.).scale(scale)
        })
        .unwrap();
    let edges = shadows
        .iter()
        .filter(|shadow| shadow.element_bounds.size == track_size)
        .collect::<Vec<_>>();
    assert_eq!(edges.len(), 4);
    for edge in edges {
        assert_eq!(edge.inset, 1);
        assert_eq!(edge.blur_radius, px(0.).scale(scale));
        assert!(edge.order < thumb.order);
    }
}

#[test]
fn translucent_color_surfaces_use_the_shared_checkerboard() {
    let shared = include_str!("../src/color_picker/mod.rs");
    let swatch = include_str!("../src/color_picker/swatch.rs");
    let slider = include_str!("../src/color_picker/slider.rs");
    assert!(
        shared.contains("pub(super) fn transparency_checker"),
        "color controls must share one checkerboard implementation"
    );
    assert!(
        swatch.contains("transparency_checker(edge, edge)"),
        "ColorSwatch must show transparency against the checkerboard"
    );
    assert!(
        slider.contains("self.channel == ColorChannel::Alpha")
            && slider.contains("transparency_checker(self.length, track_h)")
            && slider.contains("transparency_checker(track_h, self.length)"),
        "ColorSlider alpha must composite its gradient over the checkerboard"
    );
}

#[gpui::test]
fn color_slider_caps_share_the_full_track_clip(cx: &mut TestAppContext) {
    use herogpui_components::{ColorChannel, ColorSlider, ColorSpace, PickerColor};
    use herogpui_core::Orientation;

    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        for (channel, space) in [
            (ColorChannel::Hue, ColorSpace::Hsb),
            (ColorChannel::Saturation, ColorSpace::Hsb),
            (ColorChannel::Brightness, ColorSpace::Hsb),
            (ColorChannel::Lightness, ColorSpace::Hsl),
            (ColorChannel::Alpha, ColorSpace::Hsb),
            (ColorChannel::Red, ColorSpace::Rgb),
            (ColorChannel::Green, ColorSpace::Rgb),
            (ColorChannel::Blue, ColorSpace::Rgb),
        ] {
            harness::still();
            let host = harness::open_host(cx, move || {
                ColorSlider::new(
                    "rounded-slider",
                    PickerColor::hsb(180., 0.8, 0.9).with_alpha(0.35),
                    channel,
                )
                .color_space(space)
                .orientation(orientation)
                .length(px(240.))
                .show_label(false)
                .into_any_element()
            });
            let (quads, clips, scale) = host.update(|window, _| {
                (
                    window.painted_quads(),
                    window.painted_clips(),
                    window.scale_factor(),
                )
            });
            let track_size = if orientation.is_horizontal() {
                gpui::size(px(240.), px(20.))
            } else {
                gpui::size(px(20.), px(240.))
            };
            let track_mask = ContentMask {
                bounds: Bounds::new(point(px(0.), px(0.)), track_size),
                corner_radii: Corners::all(px(10.)),
                ..Default::default()
            }
            .scale(scale);
            let cap_size = if orientation.is_horizontal() {
                gpui::size(px(10.), px(20.))
            } else {
                gpui::size(px(20.), px(10.))
            }
            .scale(scale);
            let caps: Vec<_> = quads
                .iter()
                .filter(|quad| quad.bounds.size == cap_size)
                .collect();
            // The transparent alpha endpoint has only its checkerboard.
            assert_eq!(
                caps.len(),
                if channel == ColorChannel::Alpha { 1 } else { 2 }
            );
            for cap in caps {
                assert_eq!(
                    cap.content_mask.bounds, track_mask.bounds,
                    "{orientation:?} {channel:?}: a narrow cap must stay within the track bounds"
                );
                assert!(
                    cap.content_mask.clip_index > 0,
                    "{orientation:?} {channel:?}: a narrow cap must inherit the track's rounded clip"
                );
                let clip = clips[cap.content_mask.clip_index as usize - 1];
                assert_eq!(clip.bounds, track_mask.bounds);
                assert_eq!(clip.radii_x, track_mask.corner_radii);
                assert_eq!(clip.radii_y, track_mask.corner_radii);
                if channel == ColorChannel::Hue {
                    assert_eq!(
                        cap.background.as_solid(),
                        Some(gpui::red()),
                        "hue caps must match the saturated spectrum, independent of the selected color"
                    );
                }
                assert!(
                    cap.background
                        .as_solid()
                        .is_some_and(|color| color.is_opaque()),
                    "displayed endpoint colors must be opaque"
                );
            }
            let ramps: Vec<_> = quads
                .iter()
                .filter(|quad| quad.background.as_solid().is_none())
                .collect();
            let ramp_bounds = if orientation.is_horizontal() {
                Bounds::from_corners(point(px(10.), px(0.)), point(px(230.), px(20.)))
            } else {
                Bounds::from_corners(point(px(0.), px(10.)), point(px(20.), px(230.)))
            };
            let painted_ramp = ramps
                .iter()
                .map(|quad| quad.bounds)
                .reduce(|bounds, next| bounds.union(&next))
                .expect("the track must paint its gradient");
            assert_eq!(
                painted_ramp,
                ramp_bounds.scale(scale),
                "{orientation:?} {channel:?}: the gradient must span the thumb's travel between the caps"
            );
            for ramp in ramps {
                assert_eq!(ramp.content_mask.bounds, track_mask.bounds);
                assert!(ramp.content_mask.clip_index > 0);
            }
            if channel == ColorChannel::Alpha {
                let checker_colors = [
                    gpui::Hsla::from(gpui::rgb(0xefefef)),
                    gpui::Hsla::from(gpui::rgb(0xf7f7f7)),
                ];
                let cells: Vec<_> = quads
                    .iter()
                    .filter(|quad| {
                        quad.background
                            .as_solid()
                            .is_some_and(|color| checker_colors.contains(&color))
                    })
                    .collect();
                assert!(!cells.is_empty(), "alpha must retain its checkerboard");
                for cell in cells {
                    assert_eq!(cell.content_mask.bounds, track_mask.bounds);
                    assert!(
                        cell.content_mask.clip_index > 0,
                        "checkerboard cells must share the rounded clip"
                    );
                }
            }
            let thumb = quads
                .iter()
                .find(|quad| {
                    quad.bounds.size == gpui::size(px(16.), px(16.)).scale(scale)
                        && quad.border_widths.top == px(3.).scale(scale)
                })
                .expect("the thumb must be painted");
            assert_eq!(
                thumb.content_mask.clip_index, 0,
                "the thumb and its focus ring must stay outside the track clip"
            );
        }
    }
}

#[test]
fn animated_field_fills_keep_the_owner_radius() {
    let input = include_str!("../src/input.rs");
    let input_group = include_str!("../src/input_group.rs");
    let color_field = include_str!("../src/color_picker/field.rs");
    let autocomplete = include_str!("../src/autocomplete.rs");
    let number_field = include_str!("../src/number_field.rs");

    assert!(
        input.contains("|fill| fill.rounded(radius)"),
        "standalone Input hover fills must follow the field's rounded shell"
    );
    assert!(
        input_group.contains("|fill| fill.rounded(radius)"),
        "InputGroup hover fills must follow the shared group radius"
    );
    assert!(
        color_field.contains("|fill| fill.rounded(radius)"),
        "ColorField hover fills must follow the field's rounded shell"
    );
    assert!(
        autocomplete.contains("|fill| fill.rounded(trigger_radius)"),
        "Autocomplete hover fills must follow the trigger radius"
    );
    assert!(
        number_field.contains("|fill| fill.rounded(group_radius)"),
        "NumberField hover fills must follow the group radius"
    );
}
