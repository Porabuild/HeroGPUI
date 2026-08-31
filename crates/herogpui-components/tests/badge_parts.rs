//! Badge part-composition behaviour: `BadgeAnchor`, `Badge`, `BadgeLabel`.
//!
//! v3.2.4's badge is three composed parts (`badge.tsx`), not one struct: the
//! `.badge-anchor` wrapper owns the anchored element and the badge, the
//! `.badge` root positions itself against that wrapper, and `.badge__label`
//! (`px-0.5`) is the only horizontal padding in the sheet. The flattened
//! `Badge::child(anchor) + Badge::content(label)` seam could not express any
//! of that; these tests are the contract the real part types must satisfy.
//!
//! Geometry is derived from the component's own constants on the 1920x1080
//! test window: a 64px anchor whose top-left sits at (100, 40), via a padded
//! flex row (v3's `.badge-anchor` is `inline-flex`, which GPUI 0.2.2 lacks —
//! the wrapper only hugs its child inside a flex parent), with an md badge
//! overhanging a quarter of its 28px min box (7px) past the anchor's corner.

mod harness;

use gpui::{prelude::*, px, AnyElement, Bounds, Pixels, TestAppContext, VisualTestContext};
use harness::open_host;
use herogpui_components::{Badge, BadgeAnchor, BadgeLabel, BadgePlacement, Size};

/// The box a debug selector painted, last match wins (the harness selector
/// map keeps only the most recent paint of a name).
fn probe(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} must paint"))
}

fn bounds_str(b: &Bounds<Pixels>) -> String {
    format!(
        "{:.1}..{:.1} x {:.1}..{:.1}",
        f32::from(b.origin.x),
        f32::from(b.origin.x + b.size.width),
        f32::from(b.origin.y),
        f32::from(b.origin.y + b.size.height)
    )
}

/// One 64px anchor at (100, 40) with a badge of the given composition. The
/// row is flex because the anchor wrapper only hugs its child there.
fn anchored(badge: Badge) -> AnyElement {
    gpui::div()
        .pt(px(40.))
        .pl(px(100.))
        .flex()
        .items_start()
        .child(
            BadgeAnchor::new()
                .child(gpui::div().w(px(64.)).h(px(64.)))
                .child(badge),
        )
        .into_any_element()
}

/// The three parts compose: the anchor wrapper owns the anchored element and
/// the badge, the badge overhangs the anchor's top-right corner by a quarter
/// of its box, and the label slot paints inside the badge box.
#[gpui::test]
fn badge_parts_compose_anchor_badge_and_label(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        anchored(Badge::new().child(BadgeLabel::new().child("5")))
    });

    let anchor = probe(cx, "badge-anchor");
    assert_eq!(
        bounds_str(&anchor),
        "100.0..164.0 x 40.0..104.0",
        "the anchor wrapper must hug the 64px anchored element"
    );
    let badge = probe(cx, "badge");
    assert_eq!(
        bounds_str(&badge),
        "143.0..171.0 x 33.0..61.0",
        "the md badge must overhang the anchor's top-right corner by 7px"
    );
    let label = probe(cx, "badge-label");
    assert!(
        label.origin.x >= badge.origin.x
            && label.origin.x + label.size.width <= badge.origin.x + badge.size.width
            && label.origin.y >= badge.origin.y
            && label.origin.y + label.size.height <= badge.origin.y + badge.size.height,
        "the label slot must paint inside the badge box"
    );
}

/// The dot is the omitted label: a badge with no children keeps the exact min
/// box and paints no label slot, while any child — label or not — is content.
#[gpui::test]
fn an_omitted_label_renders_the_dot_and_any_child_keeps_content(cx: &mut TestAppContext) {
    {
        let cx = open_host(cx, || anchored(Badge::new()));
        assert_eq!(
            bounds_str(&probe(cx, "badge")),
            "143.0..171.0 x 33.0..61.0",
            "the md dot must keep the 28px min box"
        );
        assert!(
            cx.debug_bounds("badge-label").is_none(),
            "a dot must paint no label slot"
        );
    }
    {
        // A non-label child (v3's icon case) is still content: the badge
        // grows to the child plus its own border and paints no label slot.
        let cx = open_host(cx, || {
            anchored(Badge::new().child(gpui::div().w(px(40.)).h(px(40.))))
        });
        assert_eq!(
            bounds_str(&probe(cx, "badge")),
            "129.0..171.0 x 33.0..75.0",
            "a bare 40px child must grow the badge past the 28px min box"
        );
        assert!(
            cx.debug_bounds("badge-label").is_none(),
            "a bare child must not invent a label slot"
        );
    }
}

/// `.badge__label` is `px-0.5`: the label pads its content 2px each side and
/// the badge grows by the label plus its own 1px border, the label centred in
/// the badge on both axes.
#[gpui::test]
fn badge_label_carries_the_pinned_padding(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        anchored(Badge::new().child(BadgeLabel::new().child(gpui::div().w(px(30.)).h(px(10.)))))
    });

    let badge = probe(cx, "badge");
    let label = probe(cx, "badge-label");
    assert_eq!(
        format!("{:.1}", f32::from(label.size.width)),
        "34.0",
        "px-0.5 must pad the label 2px on each side"
    );
    assert_eq!(
        format!("{:.1}", f32::from(badge.size.width)),
        "36.0",
        "the badge must grow past its 28px min by the label plus its border"
    );
    assert_eq!(
        format!("{:.1}", f32::from(label.origin.x - badge.origin.x)),
        "1.0",
        "the label must sit flush behind the badge's left border"
    );
    assert_eq!(
        format!(
            "{:.1}",
            f32::from(badge.origin.x + badge.size.width - (label.origin.x + label.size.width))
        ),
        "1.0",
        "the label must sit flush before the badge's right border"
    );
    // Vertical centring: 1px border + (28 - 2 - 10) / 2.
    assert_eq!(
        format!("{:.1}", f32::from(label.origin.y - badge.origin.y)),
        "9.0",
        "the label must centre vertically inside the badge"
    );
}

/// A labelled badge holds its anchor corner at a non-default placement too:
/// an sm badge at bottom-left overhangs 4px down and left of the anchor.
#[gpui::test]
fn a_labelled_badge_holds_its_anchor_corner_at_the_bottom_left(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        anchored(
            Badge::new()
                .size(Size::Sm)
                .placement(BadgePlacement::BottomLeft)
                .child(BadgeLabel::new().child(gpui::div().w(px(14.)).h(px(8.)))),
        )
    });

    let badge = probe(cx, "badge");
    assert_eq!(
        bounds_str(&badge),
        "96.0..116.0 x 92.0..108.0",
        "the sm bottom-left badge must overhang 4px past the anchor's corner"
    );
    let label = probe(cx, "badge-label");
    assert!(
        label.origin.x >= badge.origin.x
            && label.origin.x + label.size.width <= badge.origin.x + badge.size.width
            && label.origin.y >= badge.origin.y
            && label.origin.y + label.size.height <= badge.origin.y + badge.size.height,
        "the label slot must paint inside the badge box at the non-default corner"
    );
}

/// Two anchored badges in one window stay isolated: the second badge reads
/// its corner off its own 40px anchor, not off the first anchor or the row.
/// The harness selector map keeps only the last painted match, so the probes
/// below can only see the second instance — the first instance's box is not
/// probeable through the shared selector name, but its paint is what the
/// frame the second is measured against is built from.
#[gpui::test]
fn badge_instances_anchor_to_their_own_anchor(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .pt(px(40.))
            .pl(px(100.))
            .flex()
            .items_start()
            .gap(px(36.))
            .child(
                BadgeAnchor::new()
                    .child(gpui::div().w(px(64.)).h(px(64.)))
                    .child(Badge::new()),
            )
            .child(
                BadgeAnchor::new()
                    .child(gpui::div().w(px(40.)).h(px(40.)))
                    .child(
                        Badge::new()
                            .size(Size::Sm)
                            .placement(BadgePlacement::BottomLeft),
                    ),
            )
            .into_any_element()
    });

    let anchor = probe(cx, "badge-anchor");
    assert_eq!(
        bounds_str(&anchor),
        "200.0..240.0 x 40.0..80.0",
        "the second anchor must hug its own 40px element, not stretch to the first"
    );
    assert_eq!(
        bounds_str(&probe(cx, "badge")),
        "196.0..212.0 x 68.0..84.0",
        "the second badge must overhang its own anchor's corner, not the first's"
    );
}

/// v3's placement translate is `±25%` of the badge's own box, but GPUI 0.2.2
/// has no div-level transform, so the port overhangs a quarter of the *min*
/// box and a badge grown past it keeps that min-box offset: 7px here, where
/// upstream would translate a quarter of the grown 36px box (9px). This pins
/// the honest limitation — the one grown-box geometry the port deliberately
/// does not match — so it is visible if the framework ever gains transforms.
#[gpui::test]
fn a_grown_badge_keeps_the_min_box_overhang(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        anchored(Badge::new().child(BadgeLabel::new().child(gpui::div().w(px(30.)).h(px(10.)))))
    });

    let badge = probe(cx, "badge");
    assert_eq!(
        bounds_str(&badge),
        "135.0..171.0 x 33.0..61.0",
        "the grown badge must keep the 7px min-box overhang, not a quarter of \
         its grown width"
    );
}
