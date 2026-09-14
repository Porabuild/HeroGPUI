//! Real Metal rasterization: clipped content must match GPUI's rounded quad.
#![cfg(target_os = "macos")]

use gpui::{
    point, px, size, white, AtlasKey, AtlasTile, Bounds, ContentMask, Corners, DevicePixels,
    ImageId, MonochromeSprite, PathBuilder, PlatformHeadlessRenderer, PolychromeSprite, Quad,
    RenderImageParams, RenderSvgParams, RoundedClip, ScaledPixels, Scene, Shadow, Underline,
};
use gpui_apple::metal_renderer::MetalHeadlessRenderer;
use std::borrow::Cow;

mod harness;

#[derive(Clone, Copy, Debug)]
enum Fill {
    Solid,
    Gradient,
    Path,
    Shadow,
    Underline,
    Monochrome,
    Image,
}

#[test]
fn linked_elliptical_clips_render_the_exact_intersection() {
    let mut renderer = MetalHeadlessRenderer::new();
    let viewport = size(DevicePixels(112), DevicePixels(112));
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(112.), px(112.))).scale(1.);
    let nodes = [
        RoundedClip {
            bounds: Bounds::new(point(px(8.), px(8.)), size(px(88.), px(88.))).scale(1.),
            radii_x: Corners::all(px(32.).scale(1.)),
            radii_y: Corners::all(px(32.).scale(1.)),
            ..Default::default()
        },
        RoundedClip {
            bounds: Bounds::new(point(px(14.), px(4.)), size(px(86.), px(78.))).scale(1.),
            radii_x: Corners::all(px(22.).scale(1.)),
            radii_y: Corners::all(px(35.).scale(1.)),
            ..Default::default()
        },
    ];
    let mut expected = vec![255u8; 112 * 112];
    for node in nodes {
        let mut scene = Scene::default();
        let index = scene.insert_clip(node);
        scene.insert_primitive(Quad {
            bounds,
            content_mask: ContentMask {
                bounds,
                clip_index: index,
                ..Default::default()
            },
            background: white().into(),
            ..Default::default()
        });
        scene.finish();
        let image = renderer.render_scene_to_image(&scene, viewport).unwrap();
        for (want, pixel) in expected.iter_mut().zip(image.pixels()) {
            *want = (*want).min(pixel[0]);
        }
    }
    let mut scene = Scene::default();
    // A decoy catches a wrong node stride or a zero/one-based index mix-up.
    scene.insert_clip(RoundedClip {
        bounds: Bounds::new(point(px(0.), px(0.)), size(px(2.), px(2.))).scale(1.),
        ..nodes[0]
    });
    let outer = scene.insert_clip(nodes[0]);
    let index = scene.insert_clip(RoundedClip {
        parent: outer,
        ..nodes[1]
    });
    scene.insert_primitive(Quad {
        bounds,
        content_mask: ContentMask {
            bounds,
            clip_index: index,
            ..Default::default()
        },
        background: white().into(),
        ..Default::default()
    });
    scene.finish();
    let actual = renderer.render_scene_to_image(&scene, viewport).unwrap();
    let logical = nodes.map(|node| RoundedClip {
        bounds: node.bounds.map(|value| px(value.0)),
        radii_x: node.radii_x.map(|value| px(value.0)),
        radii_y: node.radii_y.map(|value| px(value.0)),
        ..Default::default()
    });
    let mut antialiased_pixels = 0;
    for (x, y, pixel) in actual.enumerate_pixels() {
        assert!(pixel[0].abs_diff(expected[(y * 112 + x) as usize]) <= 1);
        if pixel[0] > 0 && pixel[0] < 255 {
            antialiased_pixels += 1;
        }
        // Independently check ellipse membership away from the AA fringe.
        let probes = [-1., 0., 1.]
            .into_iter()
            .flat_map(|dx| {
                [-1., 0., 1.].map(move |dy| point(px(x as f32 + 0.5 + dx), px(y as f32 + 0.5 + dy)))
            })
            .collect::<Vec<_>>();
        if probes
            .iter()
            .all(|point| logical.iter().all(|node| node.contains(*point)))
        {
            assert_eq!(pixel[0], 255, "inside ({x},{y})");
        }
        if logical
            .iter()
            .any(|node| probes.iter().all(|point| !node.contains(*point)))
        {
            assert_eq!(pixel[0], 0, "outside ({x},{y})");
        }
    }
    assert!(
        antialiased_pixels > 30,
        "rounded edges must remain antialiased"
    );
}

#[gpui::test]
fn fractional_nested_rectangles_keep_the_parent_curve_in_actual_window_paint(
    cx: &mut gpui::TestAppContext,
) {
    use gpui::prelude::*;
    let host = harness::open_host(cx, || {
        gpui::canvas(
            |_, _, _| {},
            |_, _, window, _| {
                let parent = ContentMask {
                    bounds: Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.))),
                    corner_radii: Corners::all(px(16.)),
                    ..Default::default()
                };
                let child = ContentMask {
                    bounds: Bounds::from_corners(
                        point(px(4.7), px(4.7)),
                        point(px(95.3), px(95.3)),
                    ),
                    ..Default::default()
                };
                window.with_content_mask(Some(parent), |window| {
                    window.with_content_mask(Some(child), |window| {
                        window.paint_quad(gpui::fill(parent.bounds, white()));
                    });
                });
            },
        )
        .size(px(100.))
        .into_any_element()
    });
    let (quads, clips, scale) = host.update(|window, _| {
        (
            window.painted_quads(),
            window.painted_clips(),
            window.scale_factor(),
        )
    });
    let mut scene = Scene::default();
    for clip in clips {
        scene.insert_clip(clip);
    }
    for quad in quads {
        scene.insert_primitive(quad);
    }
    scene.finish();
    let mut renderer = MetalHeadlessRenderer::new();
    let image = renderer
        .render_scene_to_image(&scene, size(DevicePixels(220), DevicePixels(220)))
        .unwrap();
    let position = (4.7 * scale).floor() as u32;
    let corner = image.get_pixel(position, position)[0];
    assert!(
        corner > 0 && corner < 230,
        "parent edge must retain partial coverage, got {corner}"
    );
    assert_eq!(image.get_pixel(30, 30)[0], 255);
}

#[gpui::test]
fn border_strip_optimization_preserves_the_parent_clip(cx: &mut gpui::TestAppContext) {
    use gpui::prelude::*;
    let host = harness::open_host(cx, || {
        gpui::canvas(
            |_, _, _| {},
            |_, _, window, _| {
                let mask = ContentMask {
                    bounds: Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.))),
                    corner_radii: Corners::all(px(16.)),
                    ..Default::default()
                };
                window.with_content_mask(Some(mask), |window| {
                    window.paint_quad(gpui::quad(
                        mask.bounds,
                        Corners::default(),
                        gpui::transparent_black(),
                        gpui::Edges::all(px(2.)),
                        white(),
                        Default::default(),
                    ));
                });
            },
        )
        .size(px(100.))
        .into_any_element()
    });
    let (quads, clips) = host.update(|window, _| (window.painted_quads(), window.painted_clips()));
    assert!(quads.len() > 1, "the border must exercise strip splitting");
    let mut actual_scene = Scene::default();
    let mut reference_scene = Scene::default();
    for clip in clips {
        actual_scene.insert_clip(clip);
        reference_scene.insert_clip(clip);
    }
    let mut whole = quads[0];
    whole.content_mask.bounds = whole.bounds;
    reference_scene.insert_primitive(whole);
    for quad in quads {
        actual_scene.insert_primitive(quad);
    }
    actual_scene.finish();
    reference_scene.finish();
    let mut renderer = MetalHeadlessRenderer::new();
    let viewport = size(DevicePixels(220), DevicePixels(220));
    let actual = renderer
        .render_scene_to_image(&actual_scene, viewport)
        .unwrap();
    let expected = renderer
        .render_scene_to_image(&reference_scene, viewport)
        .unwrap();
    for (got, want) in actual.pixels().zip(expected.pixels()) {
        assert!(got[0].abs_diff(want[0]) <= 1);
    }
}

fn paint(
    scene: &mut Scene,
    fill: Fill,
    bounds: Bounds<ScaledPixels>,
    mask: ContentMask<ScaledPixels>,
    tiles: [AtlasTile; 2],
) {
    // Keep atlas filtering at the source tile's outer edge away from the clip
    // under test, including for very wide or tall destination rectangles.
    let bounds = if matches!(fill, Fill::Monochrome | Fill::Image) {
        bounds.dilate(ScaledPixels(256.))
    } else {
        bounds
    };
    match fill {
        Fill::Solid | Fill::Gradient => scene.insert_primitive(Quad {
            bounds,
            content_mask: mask,
            background: if matches!(fill, Fill::Gradient) {
                gpui::linear_gradient(
                    90.,
                    gpui::linear_color_stop(white(), 0.),
                    gpui::linear_color_stop(white(), 1.),
                )
            } else {
                white().into()
            },
            ..Default::default()
        }),
        Fill::Path => {
            let mut builder = PathBuilder::fill();
            let bounds = bounds.map(|value| px(value.0));
            builder.move_to(bounds.origin);
            builder.line_to(bounds.top_right());
            builder.line_to(bounds.bottom_right());
            builder.line_to(bounds.bottom_left());
            builder.close();
            let mut path = builder.build().unwrap().scale(1.);
            path.content_mask = mask;
            path.color = white().into();
            scene.insert_primitive(path);
        }
        Fill::Shadow => scene.insert_primitive(Shadow {
            order: 0,
            blur_radius: px(0.).scale(1.),
            bounds,
            corner_radii: Default::default(),
            content_mask: mask,
            color: white(),
            element_bounds: bounds,
            element_corner_radii: Default::default(),
            inset: 0,
            pad: 0,
        }),
        Fill::Underline => scene.insert_primitive(Underline {
            order: 0,
            pad: 0,
            bounds,
            content_mask: mask,
            color: white(),
            thickness: bounds.size.height,
            wavy: false.into(),
        }),
        Fill::Monochrome => scene.insert_primitive(MonochromeSprite {
            order: 0,
            pad: 0,
            bounds,
            content_mask: mask,
            color: white(),
            tile: tiles[0],
            transformation: Default::default(),
        }),
        Fill::Image => scene.insert_primitive(PolychromeSprite {
            order: 0,
            pad: 0,
            grayscale: false.into(),
            opacity: 1.,
            bounds,
            content_mask: mask,
            corner_radii: Default::default(),
            tile: tiles[1],
        }),
    }
}

#[test]
fn rounded_clip_pixels_match_rounded_quads_across_sizes_and_content() {
    let mut renderer = MetalHeadlessRenderer::new();
    let atlas = renderer.sprite_atlas();
    let tile_size = size(DevicePixels(64), DevicePixels(64));
    let keys = [
        AtlasKey::Svg(RenderSvgParams {
            path: "clip-test".into(),
            size: tile_size,
        }),
        AtlasKey::Image(RenderImageParams {
            image_id: ImageId(0),
            frame_index: 0,
        }),
    ];
    let tiles = [0, 1].map(|index| {
        atlas
            .get_or_insert_with(&keys[index], &mut || {
                Ok(Some((
                    tile_size,
                    Cow::Owned(vec![255; 4096 * if index == 0 { 1 } else { 4 }]),
                )))
            })
            .unwrap()
            .unwrap()
    });

    for scale in [1., 1.25, 2.] {
        for (width, height) in [
            (1., 1.),
            (0.5, 70.),
            (70., 0.5),
            (10., 20.),
            (20., 10.),
            (23.5, 71.25),
            (160., 20.),
            (20., 160.),
            (4096., 3.),
            (3., 4096.),
        ] {
            let logical =
                Bounds::new(point(px(8.25), px(8.75)), size(px(width), px(height))).scale(scale);
            // Window snaps each original element's edges before painting.
            let bounds = Bounds::from_corners(
                logical
                    .origin
                    .map(|value| ScaledPixels((value.0 - 0.5).ceil())),
                logical
                    .bottom_right()
                    .map(|value| ScaledPixels((value.0 - 0.5).ceil())),
            );
            let viewport = size(
                DevicePixels((bounds.right().0 + 10.).ceil() as i32),
                DevicePixels((bounds.bottom().0 + 10.).ceil() as i32),
            );
            let canvas = Bounds::new(
                point(px(0.), px(0.)),
                size(px(viewport.width.0 as f32), px(viewport.height.0 as f32)),
            )
            .scale(1.);
            for asymmetric in [false, true] {
                let radius = px(width.min(height) * 0.5)
                    .scale(scale)
                    .min(bounds.size.width.min(bounds.size.height) * 0.5);
                let radii = if asymmetric {
                    Corners {
                        top_left: radius,
                        top_right: radius * 0.25,
                        bottom_right: radius * 0.75,
                        bottom_left: px(0.).scale(1.),
                    }
                } else {
                    Corners::all(radius)
                };
                let mut reference = Scene::default();
                reference.insert_primitive(Quad {
                    bounds,
                    corner_radii: radii,
                    background: white().into(),
                    content_mask: ContentMask {
                        bounds: canvas,
                        ..Default::default()
                    },
                    ..Default::default()
                });
                reference.finish();
                let expected = renderer
                    .render_scene_to_image(&reference, viewport)
                    .unwrap();
                for fill in [
                    Fill::Solid,
                    Fill::Gradient,
                    Fill::Path,
                    Fill::Shadow,
                    Fill::Underline,
                    Fill::Monochrome,
                    Fill::Image,
                ] {
                    let mut scene = Scene::default();
                    let index = scene.insert_clip(RoundedClip {
                        bounds,
                        radii_x: radii,
                        radii_y: radii,
                        ..Default::default()
                    });
                    paint(
                        &mut scene,
                        fill,
                        canvas,
                        ContentMask {
                            bounds,
                            clip_index: index,
                            ..Default::default()
                        },
                        tiles,
                    );
                    scene.finish();
                    let actual = renderer.render_scene_to_image(&scene, viewport).unwrap();
                    for (x, y, pixel) in actual.enumerate_pixels() {
                        // Paths are rasterized into an MSAA intermediate; compare
                        // full pixels exactly and allow one sample at an AA edge.
                        let want = expected.get_pixel(x, y)[0];
                        let tolerance = if matches!(fill, Fill::Path) && want > 0 && want < 255 {
                            40
                        } else if matches!(fill, Fill::Gradient) {
                            // The existing gradient shader dithers RGB by 2/255
                            // and alpha by 3/255 before compositing.
                            5
                        } else {
                            3
                        };
                        assert!(pixel[0].abs_diff(want) <= tolerance,
                            "{fill:?}, {width}×{height} at {scale}×, asymmetric={asymmetric}, ({x},{y}): {} != {want}", pixel[0]);
                    }
                }
            }
        }
    }
}

#[gpui::test]
fn slider_caps_keep_their_color_without_a_gray_outline(cx: &mut gpui::TestAppContext) {
    use gpui::prelude::*;
    use herogpui_components::{ColorChannel, ColorSlider, PickerColor};
    use herogpui_core::Orientation;
    let mut renderer = MetalHeadlessRenderer::new();
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let host = harness::open_host(cx, move || {
            ColorSlider::new(
                "edge-layers",
                PickerColor::hsb(207., 0.9, 0.9),
                ColorChannel::Hue,
            )
            .length(px(240.))
            .orientation(orientation)
            .show_label(false)
            .into_any_element()
        });
        let (quads, shadows, clips) = host.update(|window, _| {
            (
                window.painted_quads(),
                window.painted_shadows(),
                window.painted_clips(),
            )
        });
        let canvas = Bounds::new(point(px(0.), px(0.)), size(px(500.), px(500.))).scale(1.);
        let mut scene = Scene::default();
        for clip in clips {
            scene.insert_clip(clip);
        }
        scene.insert_primitive(Quad {
            bounds: canvas,
            content_mask: ContentMask {
                bounds: canvas,
                ..Default::default()
            },
            background: gpui::rgb(0xf4f4f4).into(),
            ..Default::default()
        });
        let mut primitives = quads
            .iter()
            .map(|q| (q.order, gpui::Primitive::Quad(*q)))
            .collect::<Vec<_>>();
        primitives.extend(
            shadows
                .iter()
                .map(|s| (s.order, gpui::Primitive::Shadow(*s))),
        );
        primitives.sort_by_key(|p| p.0);
        for (_, p) in primitives {
            scene.insert_primitive(p);
        }
        scene.finish();
        let image = renderer
            .render_scene_to_image(&scene, size(DevicePixels(500), DevicePixels(500)))
            .unwrap();
        let points = if orientation.is_horizontal() {
            [(1, 19), (478, 19)]
        } else {
            [(19, 1), (19, 478)]
        };
        for (x, y) in points {
            let pixel = image.get_pixel(x, y);
            assert!(
                pixel[0] > 200 && pixel[1] < 5 && pixel[2] < 5,
                "{orientation:?} endpoint must keep its red hue under inset shading: {pixel:?}"
            );
        }
    }
}
