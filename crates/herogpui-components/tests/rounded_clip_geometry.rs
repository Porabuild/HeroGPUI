//! Size-independent clipping contracts, separate from any component's layout.

use gpui::{point, px, size, Bounds, ClipRegion, ContentMask, Corners, Edges, Overflow, Style};

#[test]
fn intersections_preserve_original_shapes_across_sizes_and_aspect_ratios() {
    for (width, height) in [
        (0., 10.),
        (0.5, 80.),
        (80., 0.5),
        (1., 1.),
        (10., 20.),
        (20., 10.),
        (23.5, 71.25),
        (100., 100.),
        (8192., 37.),
    ] {
        let bounds = Bounds::new(point(px(0.25), px(0.75)), size(px(width), px(height)));
        let radius = width.min(height) * 0.5;
        let outer: ClipRegion = ContentMask {
            bounds,
            corner_radii: Corners {
                top_left: px(radius),
                top_right: px(radius * 0.25),
                bottom_right: px(radius * 0.75),
                bottom_left: px(0.),
            },
            ..Default::default()
        }
        .into();
        for (dx, dy) in [(0., 0.), (0.04, 0.04), (0., 0.4), (0.4, 0.)] {
            let inner: ClipRegion = ContentMask {
                bounds: Bounds::new(
                    point(
                        bounds.left() + px(width * dx),
                        bounds.top() + px(height * dy),
                    ),
                    size(px(width * 0.6), px(height * 0.6)),
                ),
                corner_radii: Corners::all(px(radius * 0.6)),
                ..Default::default()
            }
            .into();
            let intersection = outer.intersect(&inner);
            for x in 0..41 {
                for y in 0..41 {
                    let point = point(
                        bounds.left() + px(width * x as f32 / 40.),
                        bounds.top() + px(height * y as f32 / 40.),
                    );
                    assert_eq!(
                        intersection.contains(point),
                        outer.contains(point) && inner.contains(point),
                        "{width}×{height}, inset {dx},{dy}, point {point:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn narrow_rectangular_clips_do_not_shrink_ancestor_radii() {
    let outer = ContentMask {
        bounds: Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.))),
        corner_radii: Corners::all(px(16.)),
        ..Default::default()
    };
    let narrow = ContentMask {
        bounds: Bounds::new(point(px(0.), px(0.)), size(px(10.), px(100.))),
        ..Default::default()
    };
    let region = outer.intersect(&narrow);
    assert_eq!(region.rounded_clips[0].bounds, outer.bounds);
    assert!(!region.contains(point(px(0.5), px(5.5))));
    assert!(region.contains(point(px(9.5), px(16.5))));
}

#[test]
fn scrolling_and_clip_overflow_keep_rounded_corners() {
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(80.)));
    for x in [Overflow::Hidden, Overflow::Scroll, Overflow::Clip] {
        for y in [Overflow::Hidden, Overflow::Scroll, Overflow::Clip] {
            let style = Style {
                overflow: point(x, y),
                corner_radii: Corners::all(px(16.)).map(|radius| (*radius).into()),
                ..Default::default()
            };
            let mask = style
                .overflow_mask(bounds, px(16.), 1.)
                .expect("overflow must clip");
            assert_eq!(mask.rounded_clips.len(), 1);
            assert!(!mask.contains(point(px(0.5), px(0.5))));
            assert!(mask.contains(point(px(16.), px(0.5))));
        }
    }
    for overflow in [
        point(Overflow::Visible, Overflow::Scroll),
        point(Overflow::Scroll, Overflow::Visible),
    ] {
        let style = Style {
            overflow,
            corner_radii: Corners::all(px(16.)).map(|radius| (*radius).into()),
            ..Default::default()
        };
        let mask = style.overflow_mask(bounds, px(16.), 1.).unwrap();
        assert!(!mask.contains(point(px(0.5), px(0.5))));
    }
}

#[test]
fn fractional_borders_use_the_same_device_pixels_as_the_painted_border() {
    let style = Style {
        overflow: point(Overflow::Hidden, Overflow::Hidden),
        corner_radii: Corners::all(px(10.)).map(|radius| (*radius).into()),
        border_widths: Edges {
            left: px(0.25).into(),
            ..Default::default()
        },
        border_color: Some(gpui::white()),
        ..Default::default()
    };
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(20.), px(20.)));
    for scale in [1., 1.25, 2.] {
        let mask = style.overflow_mask(bounds, px(16.), scale).unwrap();
        assert_eq!(mask.bounds.left(), px(1. / scale));
        assert_eq!(mask.rounded_clips[0].radii_x.top_left, px(10. - 1. / scale));
        assert_eq!(mask.rounded_clips[0].radii_y.top_left, px(10.));
    }
}

#[test]
fn transparent_and_unequal_borders_preserve_the_correct_curve() {
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.)));
    let mut style = Style {
        overflow: point(Overflow::Hidden, Overflow::Hidden),
        corner_radii: Corners::all(px(50.)).map(|radius| (*radius).into()),
        border_widths: Edges {
            left: px(10.).into(),
            ..Default::default()
        },
        border_color: Some(gpui::transparent_black()),
        ..Default::default()
    };
    let transparent = style.overflow_mask(bounds, px(16.), 1.).unwrap();
    assert_eq!(transparent.bounds, bounds);
    assert_eq!(transparent.rounded_clips[0].radii_x.top_left, px(50.));
    style.border_color = Some(gpui::white());
    let visible = style.overflow_mask(bounds, px(16.), 1.).unwrap();
    assert_eq!(visible.bounds.left(), px(10.));
    assert_eq!(visible.rounded_clips[0].radii_x.top_left, px(40.));
    assert_eq!(visible.rounded_clips[0].radii_y.top_left, px(50.));
    assert!(!visible.contains(point(px(11.), px(10.))));
    assert!(visible.contains(point(px(50.), px(0.5))));
}

#[test]
fn replay_remaps_clip_chains_and_clear_releases_the_frame() {
    let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(100.))).scale(1.);
    let mut previous = gpui::Scene::default();
    let mut index = 0;
    for radius in 1..65 {
        index = previous.insert_clip(gpui::RoundedClip {
            bounds,
            radii_x: Corners::all(px(radius as f32 / 2.).scale(1.)),
            radii_y: Corners::all(px(radius as f32 / 2.).scale(1.)),
            parent: index,
            padding: 0,
        });
    }
    previous.insert_primitive(gpui::Quad {
        bounds,
        content_mask: ContentMask {
            bounds,
            clip_index: index,
            ..Default::default()
        },
        background: gpui::white().into(),
        ..Default::default()
    });
    let mut next = gpui::Scene::default();
    next.insert_clip(gpui::RoundedClip {
        bounds,
        radii_x: Corners::all(px(49.).scale(1.)),
        radii_y: Corners::all(px(49.).scale(1.)),
        ..Default::default()
    });
    next.replay(0..previous.len(), &previous);
    let mut replayed = next.quads[0].content_mask.clip_index;
    assert_ne!(replayed, index);
    for original in previous.rounded_clips.iter().rev() {
        let clip = next.rounded_clips[replayed as usize - 1];
        assert_eq!(clip.bounds, original.bounds);
        assert_eq!(clip.radii_x, original.radii_x);
        replayed = clip.parent;
    }
    assert_eq!(replayed, 0);
    next.clear();
    assert!(next.rounded_clips.is_empty());
}
