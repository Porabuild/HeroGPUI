//! Behaviour coverage for ScrollShadow paths not exercised by the vertical
//! visibility tests in `virtual_and_feedback.rs`.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    point, prelude::*, px, ScrollDelta, ScrollWheelEvent, TestAppContext, VisualTestContext,
};

use harness::open_host;
use herogpui_components::{Orientation, ScrollShadow, ScrollShadowVisibility};

fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

fn wheel_h(cx: &mut VisualTestContext, x: f32, y: f32, dx: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(x), px(y)),
        delta: ScrollDelta::Pixels(point(px(dx), px(0.))),
        ..Default::default()
    });
    flush_frame(cx);
}

/// Horizontal orientation must derive visibility from the x-axis. Ten 40px
/// blocks with 8px gaps make 472px of content in a 160px viewport, so the
/// exact 312px range can be driven from edge to edge without relying on a
/// screenshot of the gradients.
#[gpui::test]
fn horizontal_scroll_shadow_reports_each_edge(cx: &mut TestAppContext) {
    let recorded = Rc::new(RefCell::new(Vec::new()));
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ScrollShadow::new("ss-horizontal")
            .orientation(Orientation::Horizontal)
            .max_w(px(160.))
            .visibility(ScrollShadowVisibility::Auto)
            .on_visibility_change(move |visibility, _, _| {
                recorded.borrow_mut().push(visibility);
            })
            .children((0..10).map(|_| {
                gpui::div()
                    .w(px(40.))
                    .h(px(40.))
                    .flex_shrink_0()
                    .into_any_element()
            }))
            .into_any_element()
    });
    flush_frame(cx);

    assert_eq!(
        recorded.borrow().as_slice(),
        [ScrollShadowVisibility::Right],
        "at the left edge only the right shadow must be visible"
    );

    wheel_h(cx, 80., 20., -156.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [ScrollShadowVisibility::Right, ScrollShadowVisibility::Both,],
        "mid-scroll both horizontal edges must be visible"
    );

    wheel_h(cx, 80., 20., -156.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [
            ScrollShadowVisibility::Right,
            ScrollShadowVisibility::Both,
            ScrollShadowVisibility::Left,
        ],
        "at the right edge only the left shadow must be visible"
    );

    wheel_h(cx, 80., 20., 312.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [
            ScrollShadowVisibility::Right,
            ScrollShadowVisibility::Both,
            ScrollShadowVisibility::Left,
            ScrollShadowVisibility::Right,
        ],
        "returning to the left edge must restore the right shadow"
    );
}
