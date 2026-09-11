//! Overlay panel padding measured on the RESTING panel.
//!
//! `Popover::padding` / `Toast::padding` resolve one pair that the resting
//! panel's chain and its entry-animation `ZoomBox` both consume. Under
//! reduced motion the zoom returns the panel untouched, so the chain is the
//! only consumer there — the reduced-motion bounds test holds the resting
//! panel to the resolved pair, and the wiring checks pin the motion path to
//! the same source binding.
//!
//! The popover panel carries no debug selector of its own, so the probe is
//! caller content inside it. The panel is a `flex_col` whose padding is
//! symmetric, so a widthless probe stretched across the content box answers
//! the x inset twice over: its width is the pinned 260px panel width minus
//! both x insets. Its top edge sits one y inset below the panel's top edge,
//! which is the measured trigger bottom plus the documented 8px `offset`
//! (Bottom placement, no arrow, and no flip — the host window is tall
//! enough that the panel never turns upward).

mod harness;

use gpui::{prelude::*, px, TestAppContext, VisualTestContext};
use herogpui_components::Popover;

use harness::open_host;

/// The popover panel's pinned width (`.w(px(260.))`, `Popover::render`).
const PANEL_WIDTH: f32 = 260.;
/// The positioner's default `offset` ("8px in v3", `Popover::new`).
const OFFSET: f32 = 8.;

fn near(value: f32, expected: f32) -> bool {
    (value - expected).abs() < 0.5
}

/// A zero-behaviour 20px content block with no width of its own, so the
/// panel's cross-axis stretch makes its measured width the panel's content
/// width.
fn panel_probe(name: &'static str) -> gpui::AnyElement {
    gpui::div()
        .h(px(20.))
        .debug_selector(move || name.to_owned())
        .into_any_element()
}

/// A fixed 40x36 trigger block whose box is measurable, so the panel's
/// placement arithmetic never rests on an assumed trigger size.
fn trigger_probe(name: &'static str) -> gpui::AnyElement {
    gpui::div()
        .w(px(40.))
        .h(px(36.))
        .debug_selector(move || name.to_owned())
        .into_any_element()
}

/// One controlled-open popover over a fixed trigger. `wide` picks between
/// the `.padding(px(24.))` instance and the stock default.
fn popover(id: &'static str, wide: bool) -> Popover {
    let popover = Popover::new(trigger_probe(Box::leak(
        format!("trig-{id}").into_boxed_str(),
    )))
    .id(id)
    .is_open(true)
    .should_flip(false);
    let popover = if wide {
        popover.padding(px(24.))
    } else {
        popover
    };
    popover.child(panel_probe(Box::leak(
        format!("probe-{id}").into_boxed_str(),
    )))
}

/// Asserts the probe spans the pinned panel width minus both x insets —
/// symmetric `.px()` padding makes that the x inset exactly — and sits one
/// `padding_y` below the panel's top edge (Bottom placement, the measured
/// trigger bottom plus `offset`). The horizontal origin is deliberately not
/// asserted: a panel centred on a trigger near the left edge is clamped into
/// the viewport, which is placement policy `placement.rs` already pins.
fn assert_resting_insets(cx: &mut VisualTestContext, id: &str, padding_x: f32, padding_y: f32) {
    let trigger = cx
        .debug_bounds(Box::leak(format!("trig-{id}").into_boxed_str()))
        .unwrap_or_else(|| panic!("the {id} trigger must paint"));
    let probe = cx
        .debug_bounds(Box::leak(format!("probe-{id}").into_boxed_str()))
        .unwrap_or_else(|| panic!("the open {id} panel's content must paint"));
    assert!(
        near(f32::from(probe.size.width), PANEL_WIDTH - 2. * padding_x),
        "{id}: the stretched probe must span the pinned panel width minus \
         both x insets, got {} (probe {probe:?})",
        f32::from(probe.size.width)
    );
    let panel_top = f32::from(trigger.origin.y) + f32::from(trigger.size.height) + OFFSET;
    let y_inset = f32::from(probe.origin.y) - panel_top;
    assert!(
        near(y_inset, padding_y),
        "{id}: the resting panel's y inset must be the resolved padding, got \
         {y_inset} (probe {probe:?}, trigger {trigger:?})"
    );
}

fn assert_both_popovers(cx: &mut VisualTestContext) {
    // The stock default is the ZOOM default — 14 x-wide / 12 y-wide — which
    // is what proves the chain and the zoom resolve one pair.
    assert_resting_insets(cx, "plain", 14., 12.);
    assert_resting_insets(cx, "wide", 24., 24.);
}

#[gpui::test]
fn popover_padding_reaches_the_resting_panel_with_reduced_motion(cx: &mut TestAppContext) {
    // Under reduced motion `entering_zoom` returns the panel untouched, so
    // the chain below is the only thing that can carry the padding.
    harness::still();
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .flex_col()
            .child(popover("plain", false))
            .child(popover("wide", true))
            .into_any_element()
    });
    cx.update(|window, _| window.refresh());
    assert_both_popovers(cx);
}

/// The popover's motion-path half, by wiring. gpui's `with_animation` derives
/// its delta from real wall-clock time (`scheduler::Instant`), which the test
/// executor's `advance_clock` cannot move, so no deterministic frame past the
/// zoom's start exists in a test — and `entering_zoom` only ever hands back
/// the untouched element under reduced motion, the path the bounds test above
/// observes. What the motion path adds is *which* value the zoom interpolates,
/// and that is pinned here: the bindings the `ZoomBox` feeds must be resolved
/// exactly once, above the panel's construction, and consumed by the resting
/// chain, so the frame the zoom lands on is by construction the same padding
/// the chain paints.
#[test]
fn popover_panel_chain_consumes_the_padding_the_zoom_interpolates() {
    let popover = include_str!("../src/popover.rs");
    let panel_at = popover
        .find("let mut panel = gpui::div()")
        .expect("popover.rs: the panel construction must stay");
    let chain_end_at = popover[panel_at..]
        .find(".shadow(layout.overlay_shadow.clone());")
        .map(|at| panel_at + at)
        .expect("popover.rs: the panel chain must keep its shadow end");
    let zoom_at = popover[panel_at..]
        .find("crate::anim::ZoomBox::panel(panel_padding_y, radius)")
        .map(|at| panel_at + at)
        .expect("popover.rs: the panel zoom must interpolate the hoisted pair");

    for binding in [
        "let panel_padding_y = self.padding.unwrap_or(px(12.));",
        "let panel_padding_x = self.padding.unwrap_or(px(14.));",
    ] {
        assert_eq!(
            popover.matches(binding).count(),
            1,
            "popover.rs: {binding} must be resolved exactly once"
        );
        let at = popover
            .find(binding)
            .expect("popover.rs: the binding must exist");
        assert!(
            at < panel_at,
            "popover.rs: {binding} must be hoisted above the panel chain so \
             the resting panel and its zoom consume one pair"
        );
    }

    for consumer in [".px(panel_padding_x)", ".py(panel_padding_y)"] {
        let at = popover[panel_at..chain_end_at].find(consumer).map_or_else(
            || panic!("popover.rs: the resting panel chain must consume {consumer}"),
            |at| panel_at + at,
        );
        assert!(
            at < zoom_at,
            "popover.rs: the panel chain's {consumer} must come from the \
             hoisted binding, not a second resolution"
        );
    }
    assert!(
        popover[panel_at..].contains(".padding_x(panel_padding_x)"),
        "popover.rs: the zoom must interpolate the same x binding the panel \
         paints"
    );
}

/// The toast card's half: a scoped source check, because the card offers no
/// caller-owned content slot a bounds probe could ride in. The bindings the
/// card's `ZoomBox` interpolates must be hoisted above the card's
/// construction and consumed by the card chain, so the reduced-motion
/// resting padding is by construction the value the zoom targets.
#[test]
fn toast_card_chain_consumes_the_padding_the_zoom_interpolates() {
    let toast = include_str!("../src/toast.rs");
    let card_at = toast
        .find("let mut card = gpui::div()")
        .expect("toast.rs: the card construction must stay");
    let chain_end_at = toast[card_at..]
        .find(".overflow_hidden();")
        .map(|at| card_at + at)
        .expect("toast.rs: the card chain must keep its overflow_hidden end");
    let zoom_at = toast[card_at..]
        .find("crate::anim::ZoomBox::panel(panel_padding_y, radius)")
        .map(|at| card_at + at)
        .expect("toast.rs: the card zoom must interpolate the hoisted pair");

    for binding in [
        "let panel_padding_y = self.t.padding.unwrap_or(px(10.));",
        "let panel_padding_x = self.t.padding.unwrap_or(px(16.));",
    ] {
        assert_eq!(
            toast.matches(binding).count(),
            1,
            "toast.rs: {binding} must be resolved exactly once"
        );
        let at = toast
            .find(binding)
            .expect("toast.rs: the binding must exist");
        assert!(
            at < card_at,
            "toast.rs: {binding} must be hoisted above the card chain so the \
             resting card and its zoom consume one pair"
        );
    }

    for consumer in [".px(panel_padding_x)", ".py(panel_padding_y)"] {
        let at = toast[card_at..chain_end_at].find(consumer).map_or_else(
            || panic!("toast.rs: the resting card chain must consume {consumer}"),
            |at| card_at + at,
        );
        assert!(
            at < zoom_at,
            "toast.rs: the card chain's {consumer} must come from the hoisted \
             binding, not a second resolution"
        );
    }
    assert!(
        toast[card_at..].contains(".padding_x(panel_padding_x)"),
        "toast.rs: the zoom must interpolate the same x binding the card paints"
    );
}
