//! Behaviour tests for everything *positional*: a surface's placement, an
//! orientation, and a hover trigger.
//!
//! The overlay suite (`overlays.rs`) drives Modal / Drawer / AlertDialog /
//! Popover / Tooltip in ONE configuration each. Those are exactly the
//! components where being right once does not mean being right twice: the
//! Drawer has four edges to land on, the Popover eight directions to hang,
//! the Slider two orientations, the Modal six sizes and a scroll mode. Every
//! gallery screenshot page shows one configuration, so nothing here was
//! driven before.
//!
//! Geometry derives from the port's own constants: the Drawer's 384px desktop
//! side width and intrinsic 85vh-capped sheets, the modal width preset
//! (`max-w-xs`…`max-w-lg`), the 260px popover panel, fixed-size probe controls,
//! and the 1920x1080 test window. The
//! arithmetic for every click is written inline in the section that uses it.
//!
//! Two overlay facts this suite depends on (both learned by `overlays.rs`):
//!
//! - **Entry/exit animations run on wall time**, which the test clock does
//!   not drive. The harness's reduced-motion request pins the layout before
//!   the first frame, and the clock is advanced past `EXITING_MS` (100ms)
//!   before any closed-proof probe — or the ghost frame answers it.
//! - **A closed panel is only observable behaviourally**: gpui keeps an
//!   exiting panel mounted for `EXITING_MS`, so a probe click that must land
//!   on nothing waits for the exit first.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    point, prelude::*, px, ElementId, Modifiers, MouseButton, ScrollDelta, ScrollWheelEvent,
    TestAppContext, VisualTestContext,
};
use harness::{click, events, open_host, press, Events};
use herogpui_components::{
    Button, Drawer, DrawerPlacement, Modal, ModalPlacement, ModalScroll, ModalSize, Placement,
    Popover,
};

/// Pins the layout by enabling reduced motion **before** the first frame.
///
/// Every overlay in this suite plays an enter/exit animation on wall time,
/// which the test clock does not drive; without this the panel sits at its
/// t=0 pose for the whole test, and flipping the preference mid-test rebuilds
/// the animated wrapper structure and swallows clicks. The overlay suite's
/// `still()` rule applies to the Drawer, Popover, Tooltip and Modal tests;
/// the Slider plays no animation, so it deliberately skips the call.
fn still() {
    harness::still();
}

/// Pushes the pending frame through. Mouse events hit-test the last rendered
/// frame, so every press/move/release whose effect the next event (or the
/// next assertion) must see needs a redraw first.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// Advances the test clock past `EXITING_MS` (100ms) plus slack and forces
/// the repaint that the exit timer's `notify` only scheduled. A closed-proof
/// probe must not land on the exiting, still-mounted panel.
fn let_exit_finish(cx: &mut VisualTestContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    flush_frame(cx);
}

/// One simulated drag: press at `from`, move to `to` with the button held,
/// release there.
///
/// `from` must land on the surface that starts the drag record — for the
/// Drawer that is the title row, where the header's `on_mouse_down` writes
/// the keyed `(start, offset)`. The move and release handlers live on the
/// overlay that covers the window and always run while the record exists, so
/// `to` does not need to track the header. A single move step is a real
/// pull: the record's offset is recomputed from the current pointer each
/// move, not accumulated.
fn drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
}

fn slow_drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    std::thread::sleep(Duration::from_millis(100));
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    std::thread::sleep(Duration::from_millis(100));
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
}

/// One simulated wheel event at window coordinates (`x`, `y`) scrolling `dy`
/// pixels: **negative moves down** (later content into view), matching the
/// scrollable's `scroll_offset += delta` with negative offsets meaning
/// "scrolled down". Pixels, not lines, so no line height enters the
/// arithmetic. Followed by a redraw so the next event sees the scrolled
/// frame.
fn wheel(cx: &mut VisualTestContext, x: f32, y: f32, dy: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(x), px(y)),
        delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
        ..Default::default()
    });
    flush_frame(cx);
}

/// A fixed-size pressable probe: records `label` on click. Every geometry
/// claim in this suite is proven by placing one of these where the component
/// under test is computed to be and asserting its click records.
fn probe(id: impl Into<ElementId>, label: &'static str, recorded: Events) -> gpui::AnyElement {
    gpui::div()
        .id(id)
        .w(px(40.))
        .h(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.to_owned()))
        .child(label)
        .into_any_element()
}

// ---------------------------------------------------------------------------
// Drawer — all four placements
// ---------------------------------------------------------------------------
//
// Geometry (window 1920x1080, 384px side drawers, `p-6` = 24px inset):
//
// - Side panels are anchored to one edge and run the full height: Right
//   x [1536..1920], Left x [0..384]. Top/bottom sheets size to their content
//   and cap at 85vh.
// - The drag surface is the title row. The panel's first child is the handle
//   bar (`h-1` 4px + `pb-2`/`pt-2` 8px = 12px at y 24..36), then the 24px
//   title line (16px text at `leading-6`), so the title row is y [36..60]
//   measured from the panel's near edge — the Top drawer's row starts 36px
//   below the window top, the Bottom drawer's 36px below y = 760.
// - Dragging *toward the edge* accumulates a positive offset; the release
//   dismisses past 30% of the measured panel or on a fast flick.
// - Each test's landing probe sits after the title row: panel padding 24 +
//   handle 12 + title 24 + `mt-2` 8 = 68px from the near edge, so the body
//   probe's centre is (near_edge + 24 + 20, far_edge + 68 + 18).

#[gpui::test]
fn drawer_right_placement_lands_on_edge_and_drags_shut(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let hits = events();
    let probed = hits.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        // Right drawer: x [1536..1920]. Title row (the drag surface)
        // y [36..60], centre (1728, 48). Body probe centre (1580, 86):
        // x = 1536 + 24 padding + 20 half-probe.
        Drawer::new()
            .id("pl-drawer-right")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .child(probe("pl-drawer-right-probe", "hit", hits))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // Landing: a press where the panel's body is computed to be — 24px
    // padding + 20px half-probe across, 68px down — reaches the probe and
    // reports no dismissal: the panel truly sits at the right edge.
    click(cx, 1580., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the body probe must be reachable"
    );
    assert!(
        recorded.borrow().is_empty(),
        "a press inside the panel is not a backdrop press"
    );

    // Escape dismisses exactly once.
    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must dismiss the drawer"
    );
    let_exit_finish(cx);
    click(cx, 1580., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the panel must be gone after the exit: the probe spot records nothing"
    );

    // A slow 40px pull is activated but below 30% of the measured 384px
    // panel, so it springs back rather than taking the fast-flick path.
    *open.borrow_mut() = true;
    flush_frame(cx);
    slow_drag(cx, (1728., 48.), (1768., 48.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "a sub-threshold pull must spring back without dismissing; recorded: {:?}",
        recorded.borrow().as_slice()
    );
    click(cx, 1580., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the drawer must still be open after the spring-back"
    );

    // A 132px pull passes 30% of the measured panel.
    *open.borrow_mut() = true;
    flush_frame(cx);
    drag(cx, (1728., 48.), (1860., 48.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the far pull must report the close exactly once"
    );
    let_exit_finish(cx);
    click(cx, 1580., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the panel must be gone after the far pull"
    );
}

#[gpui::test]
fn drawer_left_placement_lands_on_edge_and_drags_shut(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let hits = events();
    let probed = hits.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        // Left drawer: x [0..384]. Title row (the drag surface) y [36..60],
        // centre (192, 48). Body probe centre (44, 86).
        Drawer::new()
            .id("pl-drawer-left")
            .is_open(is_open)
            .placement(DrawerPlacement::Left)
            .title("Drag me shut")
            .child(probe("pl-drawer-left-probe", "hit", hits))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the body probe must be reachable"
    );
    assert!(
        recorded.borrow().is_empty(),
        "an inside press must not dismiss"
    );

    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must dismiss the drawer"
    );
    let_exit_finish(cx);
    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the panel must be gone"
    );

    // A slow 40px pull is below 30% of the measured panel and springs back.
    *open.borrow_mut() = true;
    flush_frame(cx);
    slow_drag(cx, (192., 48.), (152., 48.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "a sub-threshold pull must spring back without dismissing; recorded: {:?}",
        recorded.borrow().as_slice()
    );
    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the drawer must still be open after the spring-back"
    );

    // A 132px pull toward the left edge passes 30% of the panel.
    *open.borrow_mut() = true;
    flush_frame(cx);
    drag(cx, (192., 48.), (60., 48.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the far pull must report the close exactly once"
    );
    let_exit_finish(cx);
    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the panel must be gone"
    );
}

/// The Top drawer registers its drag move/release on the window during paint,
/// so the gesture remains live at the window edge instead of losing the
/// header hitbox. A fast pull to y=0 dismisses through v3's velocity path.
#[gpui::test]
fn drawer_top_placement_lands_on_edge_and_maximal_pull_dismisses(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let hits = events();
    let probed = hits.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        // Top drawer is intrinsic-height and full width. Handle at pt(8):
        // y [24..36];
        // title row (the drag surface) y [36..60], centre (960, 48). Body
        // probe centre (44, 86).
        Drawer::new()
            .id("pl-drawer-top")
            .is_open(is_open)
            .placement(DrawerPlacement::Top)
            .title("Drag me shut")
            .child(probe("pl-drawer-top-probe", "hit", hits))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the body probe must be reachable"
    );
    assert!(
        recorded.borrow().is_empty(),
        "an inside press must not dismiss"
    );

    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must dismiss the drawer"
    );
    let_exit_finish(cx);
    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the panel must be gone"
    );

    // Window-level capture preserves the gesture to the top edge. This one
    // simulated move is a fast flick and dismisses even if 48px is below 30%
    // of the measured intrinsic sheet.
    *open.borrow_mut() = true;
    flush_frame(cx);
    drag(cx, (960., 48.), (960., 0.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the top-edge flick must add exactly one dismissal; recorded: {:?}",
        recorded.borrow().as_slice()
    );
    let_exit_finish(cx);
    click(cx, 44., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the top drawer must be gone after the captured flick"
    );
}

#[gpui::test]
fn drawer_bottom_placement_lands_on_edge_and_drags_shut(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let hits = events();
    let probed = hits.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        // This short Bottom drawer is 128px tall: y [952..1080]. Its handle
        // sits at y [976..988], title row at y [988..1012], and the body
        // probe's centre is (44, 1038).
        Drawer::new()
            .id("pl-drawer-bottom")
            .is_open(is_open)
            .placement(DrawerPlacement::Bottom)
            .title("Drag me shut")
            .child(probe("pl-drawer-bottom-probe", "hit", hits))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    click(cx, 44., 1038.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the body probe must be reachable"
    );
    assert!(
        recorded.borrow().is_empty(),
        "an inside press must not dismiss"
    );

    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must dismiss the drawer"
    );
    let_exit_finish(cx);
    click(cx, 44., 1038.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "the panel must be gone"
    );

    // A slow 20px pull is activated but stays below 30% of the measured
    // 128px sheet, so it springs back without taking the fast-flick path.
    *open.borrow_mut() = true;
    flush_frame(cx);
    slow_drag(cx, (960., 1000.), (960., 1020.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "a sub-threshold pull must spring back without dismissing; recorded: {:?}",
        recorded.borrow().as_slice()
    );
    click(cx, 44., 1038.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the drawer must still be open after the spring-back"
    );

    // A 50px pull passes 30% of the measured sheet.
    *open.borrow_mut() = true;
    flush_frame(cx);
    drag(cx, (960., 1000.), (960., 1050.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the far pull must report the close exactly once"
    );
    let_exit_finish(cx);
    click(cx, 44., 1038.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit", "hit"],
        "the panel must be gone"
    );
}

// ---------------------------------------------------------------------------
// Popover — every placement the port supports
// ---------------------------------------------------------------------------
//
// Geometry: the trigger is a fixed 1920x36 strip at y [200..236], so the
// containing block and the trigger share one width and the arithmetic cannot
// drift. The panel is 260 wide and, with `show_close_button(false)` and no
// title, `py-2` (16px) over the 36px probe = 68 tall. The offset is 8px.
//
// For the below/above placements the panel hangs from `top_full` /
// `bottom_full` (8px clear) and stretches left..right, with the content
// centred (Bottom/Top), pinned left (Start) or right (End). The containing
// block is the relative root, whose height is the trigger's outer height
// (mt-200 + 36 = 236), so the vertical span is that line's y plus or minus
// the offset:
//
// | placement  | panel box              | probe centre | outside point |
// |------------|------------------------|--------------|---------------|
// | Bottom     | y 244..312, x 830..1090 | (866, 278)  | (700, 400)    |
// | BottomStart| y 244..312, x 0..260    | (36, 278)   | (300, 400)    |
// | BottomEnd  | y 244..312, x 1660..1920| (1696, 278) | (1600, 400)   |
// | Top        | y 160..228, x 830..1090 | (866, 194)  | (700, 60)     |
// | TopStart   | y 160..228, x 0..260    | (36, 194)   | (300, 60)     |
// | TopEnd     | y 160..228, x 1660..1920| (1696, 194) | (1600, 60)    |
//
// The Left/Right placements anchor to the containing block's edges with
// `top(0)`: the panel hangs flush with the root's top and 8px clear of the
// edge that faces its name. A 1920-wide trigger makes the root as wide as
// the window, so the Left panel's box starts at x = -268 and the Right
// panel's at x = 1928 — both off-window — and `snap_to_window` is what
// decides what a user actually sees:
//
// | placement | requested box           | snapped? | probe centre |
// |-----------|-------------------------|----------|--------------|
// | Left      | x -268..-8, y 0..68     | snap to edge -> x 0..260 | (36, 34) |
// | Right     | x 1928..2188, y 0..68   | snap to edge -> x 1660..1920 | (1696, 34) |
//
// The probe centre is 16px of panel padding + 20px half-probe across and
// 16px down; the outside point is well clear of both the panel and the
// 1920-wide trigger strip.

/// The eight placements a popover supports. The panel coordinates that used to
/// ride along here are gone on purpose: `gpui::anchored` snaps a panel back
/// inside the window, so a derived position is not where the panel lands, and
/// nothing in this port carries a `debug_selector` to ask gpui where it did.
const POP_CASES: [Placement; 8] = [
    Placement::Bottom,
    Placement::BottomStart,
    Placement::BottomEnd,
    Placement::Top,
    Placement::TopStart,
    Placement::TopEnd,
    Placement::Left,
    Placement::Right,
];

/// Every placement opens from its trigger and answers Escape.
///
/// The *coordinates of the panel* are deliberately not asserted. The panel goes
/// through `gpui::anchored` with `snap_to_window`, so its origin is whatever the
/// snapping leaves once the derived position would have fallen off the window,
/// and no element in this port carries a `debug_selector` -- a test cannot ask
/// gpui where the panel actually landed, so a coordinate assertion would be a
/// guess dressed as a proof. What is exact, and worth guarding for all eight
/// placements, is the trigger's own geometry (a 100x36 button at the root's
/// top-left, because `.items_start()` pins it there) and the dismissal.
#[gpui::test]
fn popover_every_placement_opens_from_its_trigger_and_dismisses(cx: &mut TestAppContext) {
    for placement in &POP_CASES {
        still();
        let closes = events();
        let recorded = closes.clone();
        let cx = open_host(cx, move || {
            let recorded = recorded.clone();
            Popover::new(Button::new("pl-pop-trigger").label("Go"))
                .id(gpui::SharedString::from(format!("pl-pop-{placement:?}")))
                .placement(*placement)
                .child(gpui::div().w(px(200.)).h(px(60.)).child("panel"))
                .on_open_change(move |v, window, _| {
                    recorded.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                })
                .into_any_element()
        });

        // The trigger wrap is the root's first child and the root is
        // `items_start`, so the button sits at (0, 0). A `Button` is content
        // sized -- `px-4` either side of a "Go" label at 14px, about 51px wide
        // and `Size::Md`'s 36px tall -- so a press at (20, 18) is inside it
        // whatever the exact advance width comes to.
        click(cx, 20., 18.);
        flush_frame(cx);
        assert_eq!(
            closes.borrow().as_slice(),
            ["open:true"],
            "{placement:?}: the trigger must open the popover"
        );

        // The click focused the trigger, which lives inside the popover root,
        // so Escape reaches the root's key handler and dismisses.
        press(cx, "escape");
        flush_frame(cx);
        assert_eq!(
            closes.borrow().as_slice(),
            ["open:true", "open:false"],
            "{placement:?}: an open popover must answer Escape exactly once"
        );
    }
}

/// A popover opened by its controlled `isOpen` answers Escape.
///
/// It did not: `popover.rs` binds `util::dismiss_on_escape` on its root, a key
/// event only reaches elements on the focused element's path, and the component
/// claimed no focus at all -- `util::panel_focus` had *no callers*, though
/// AGENTS.md's dismissal note says "Popover and the dropdown menu hold the focus
/// themselves". Escape worked only by accident, when a click on the trigger had
/// happened to focus something inside the root; the controlled path (v3's own
/// pattern, and what React Aria does by focusing the dialog on open) left a
/// panel the keyboard could neither reach nor close. The panel now claims the
/// focus when nothing inside the popover already holds it, so the pointer path
/// still leaves the ring on the trigger.
#[gpui::test]
fn popover_controlled_open_answers_escape(cx: &mut TestAppContext) {
    still();
    let closes = events();
    let recorded = closes.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Popover::new(gpui::div().w(px(100.)).h(px(36.)).child("Go"))
            .id("pl-pop-controlled")
            .is_open(true)
            .child(gpui::div().w(px(200.)).h(px(60.)).child("panel"))
            .on_open_change(move |v, window, _| {
                recorded.borrow_mut().push(format!("open:{v}"));
                window.refresh();
            })
            .into_any_element()
    });

    press(cx, "escape");
    flush_frame(cx);
    assert_eq!(
        closes.borrow().as_slice(),
        ["open:false"],
        "an open popover must answer Escape however it was opened"
    );
}

/// Every `ModalSize` renders, opens and dismisses.
///
/// The close button's *position* is not asserted: it is `absolute right-4 top-4`
/// inside a panel whose width comes from the size preset and whose centring
/// depends on the container, and this suite has no way to ask gpui where the
/// panel landed (no element carries a `debug_selector`). Escape needs no
/// coordinates and still proves the dialog is mounted, focused and listening --
/// a size that panicked, drew nothing, or lost its key handling fails here.
#[gpui::test]
fn modal_every_size_opens_and_dismisses(cx: &mut TestAppContext) {
    let sizes = [
        ModalSize::Xs,
        ModalSize::Sm,
        ModalSize::Md,
        ModalSize::Lg,
        ModalSize::Cover,
        ModalSize::Full,
    ];
    for (i, size) in sizes.iter().enumerate() {
        still();
        let closes = events();
        let recorded = closes.clone();
        let open = Rc::new(RefCell::new(false));
        let open_for_view = open.clone();
        let size = *size;
        let cx = open_host(cx, move || {
            let is_open = *open_for_view.borrow();
            let recorded = recorded.clone();
            let held = open_for_view.clone();
            Modal::new()
                .id(gpui::SharedString::from(format!("pl-modal-{i}")))
                .is_open(is_open)
                .size(size)
                .child(gpui::div().w(px(120.)).h(px(40.)).child("body"))
                .on_open_change(move |v, window, _| {
                    *held.borrow_mut() = v;
                    recorded.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                })
                .into_any_element()
        });

        *open.borrow_mut() = true;
        flush_frame(cx);
        press(cx, "escape");
        flush_frame(cx);
        assert_eq!(
            closes.borrow().as_slice(),
            ["open:false"],
            "{size:?}: an open modal must answer Escape exactly once"
        );
        assert!(
            !*open.borrow(),
            "{size:?}: the dismissal must reach the caller's flag"
        );
    }
}

#[gpui::test]
fn modal_full_covers_every_viewport_edge_without_backdrop_dismissal(cx: &mut TestAppContext) {
    still();
    let closes = events();
    let recorded = closes.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Modal::new()
            .id("pl-modal-full-edges")
            .is_open(true)
            .is_dismissible(true)
            .size(ModalSize::Full)
            .child(gpui::div().child("body"))
            .on_open_change(move |v, _, _| recorded.borrow_mut().push(format!("open:{v}")))
            .into_any_element()
    });

    for (x, y) in [(8., 8.), (1912., 8.), (8., 1072.), (1912., 1072.)] {
        click(cx, x, y);
    }
    press(cx, "escape");

    assert_eq!(
        closes.borrow().as_slice(),
        ["open:false"],
        "no Full corner is dismissible backdrop, while Escape still proves the dialog is live"
    );
}

#[gpui::test]
fn modal_long_body_scrolls_to_reach_the_deepest_control(cx: &mut TestAppContext) {
    still();
    let hits = events();
    let probed = hits.clone();
    let closes = events();
    let recorded = closes.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open;

    let cx = open_host(cx, move || {
        let probed = hits.clone();
        let recorded = closes.clone();
        let is_open = *open_flag.borrow();
        let mut body = gpui::div().flex().flex_col().gap(px(10.));
        for i in 0..24 {
            // The click closure owns its `label`; the div renders another
            // copy, so the two start as clones.
            let label = format!("p{i}");
            let click_label = label.clone();
            let recorded = probed.clone();
            body = body.child(
                gpui::div()
                    .id(gpui::SharedString::from(format!("pl-scroll-probe-{i}")))
                    .w_full()
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(move |_, _, _| recorded.borrow_mut().push(click_label.clone()))
                    .child(label),
            );
        }
        Modal::new()
            .id("pl-modal-scroll")
            .is_open(is_open)
            .child(body)
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    recorded.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // The top of the body is reachable at rest: probe 0 sits at body top
    // (panel padding 24) + 18 in the panel's column.
    click(cx, 960., 42.);
    assert!(probed.borrow().contains(&"p0".to_owned()));

    // Scroll down hard and sweep the panel's column (Md width 448, so x 960
    // is inside it) across the lower half of the window. The deepest probe
    // must report at some point, and no sweep press may dismiss the modal:
    // a dismiss would mean the press landed on the scrim because the deep
    // content was clipped, not scrolled into view.
    for _ in 0..6 {
        wheel(cx, 960., 500., -1000.);
    }
    let mut y = 500.;
    while y < 1060. {
        click(cx, 960., y);
        y += 55.;
    }

    assert!(
        recorded.borrow().is_empty(),
        "no press anywhere in the modal's column may dismiss it: deep content \
         must be scrolled into view, not missing"
    );
    assert!(
        probed.borrow().iter().any(|hit| hit == "p23"),
        "the deepest probe must be reachable after scrolling; recorded: {:?}",
        probed.borrow().as_slice()
    );
}

#[gpui::test]
fn modal_cover_inside_scroll_reaches_the_deepest_control(cx: &mut TestAppContext) {
    still();
    let hits = events();
    let probed = hits.clone();
    let closes = events();
    let recorded = closes.clone();

    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let mut body = gpui::div().flex().flex_col().gap(px(10.));
        for i in 0..24 {
            let label = format!("p{i}");
            let click_label = label.clone();
            let hit = hits.clone();
            body = body.child(
                gpui::div()
                    .id(gpui::SharedString::from(format!("pl-cover-probe-{i}")))
                    .w_full()
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(move |_, _, _| hit.borrow_mut().push(click_label.clone()))
                    .child(label),
            );
        }
        Modal::new()
            .id("pl-modal-cover-scroll")
            .is_open(true)
            .is_dismissible(true)
            .size(ModalSize::Cover)
            .child(body)
            .on_open_change(move |v, _, _| recorded.borrow_mut().push(format!("open:{v}")))
            .into_any_element()
    });

    // Cover retains the 40px container inset; its first row therefore starts
    // at 40 + 24px panel padding + half the row height.
    click(cx, 960., 82.);
    assert_eq!(probed.borrow().as_slice(), ["p0"]);

    for _ in 0..6 {
        wheel(cx, 960., 500., -1000.);
    }
    let mut y = 500.;
    while y < 1030. {
        click(cx, 960., y);
        y += 55.;
    }

    assert!(
        closes.borrow().is_empty(),
        "Cover's retained inset must not clip the Inside body or expose backdrop"
    );
    assert!(
        probed.borrow().iter().any(|hit| hit == "p23"),
        "the deepest Cover row must remain reachable; recorded: {:?}",
        probed.borrow().as_slice()
    );
}

#[gpui::test]
fn modal_outside_scroll_keeps_each_placement_top_reachable(cx: &mut TestAppContext) {
    for placement in [
        ModalPlacement::Auto,
        ModalPlacement::Center,
        ModalPlacement::Top,
        ModalPlacement::Bottom,
    ] {
        still();
        let hits = events();
        let probed = hits.clone();

        let cx = open_host(cx, move || {
            let mut body = gpui::div().flex().flex_col().gap(px(10.));
            for i in 0..40 {
                let label = format!("p{i}");
                let click_label = label.clone();
                let recorded = hits.clone();
                body = body.child(
                    gpui::div()
                        .id(gpui::SharedString::from(format!("pl-outside-probe-{i}")))
                        .w_full()
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_click(move |_, _, _| recorded.borrow_mut().push(click_label.clone()))
                        .child(label),
                );
            }
            Modal::new()
                .id("pl-modal-outside")
                .is_open(true)
                .placement(placement)
                .scroll(ModalScroll::Outside)
                .child(body)
                .into_any_element()
        });

        // v3 top-aligns an Outside dialog in its scrolling backdrop. Once a
        // dialog overflows, every placement starts at the scroll origin: the
        // first row is at padding 40 + panel padding 24 + half its 36px box.
        click(cx, 960., 82.);
        assert_eq!(
            probed.borrow().as_slice(),
            ["p0"],
            "{placement:?}: the first row of a tall Outside dialog must remain reachable"
        );
    }
}

#[gpui::test]
fn modal_full_outside_scroll_keeps_deep_content_reachable(cx: &mut TestAppContext) {
    still();
    let hits = events();
    let probed = hits.clone();
    let closes = events();
    let recorded = closes.clone();

    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let mut body = gpui::div().flex().flex_col().gap(px(10.));
        for i in 0..40 {
            let label = format!("p{i}");
            let click_label = label.clone();
            let recorded = hits.clone();
            body = body.child(
                gpui::div()
                    .id(gpui::SharedString::from(format!(
                        "pl-full-outside-probe-{i}"
                    )))
                    .w_full()
                    .h(px(36.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(move |_, _, _| recorded.borrow_mut().push(click_label.clone()))
                    .child(label),
            );
        }
        Modal::new()
            .id("pl-modal-full-outside")
            .is_open(true)
            .is_dismissible(true)
            .size(ModalSize::Full)
            .scroll(ModalScroll::Outside)
            .child(body)
            .on_open_change(move |v, _, _| recorded.borrow_mut().push(format!("open:{v}")))
            .into_any_element()
    });

    // Full removes the container's 40px inset, so the first row starts at the
    // panel's 24px padding plus half its 36px height.
    click(cx, 960., 42.);
    assert_eq!(probed.borrow().as_slice(), ["p0"]);

    for _ in 0..12 {
        wheel(cx, 960., 500., -1000.);
    }
    let mut y = 500.;
    while y < 1060. {
        click(cx, 960., y);
        y += 55.;
    }

    assert!(
        closes.borrow().is_empty(),
        "Full Outside content must enlarge the panel and scroll, not expose backdrop"
    );
    assert!(
        probed.borrow().iter().any(|hit| hit == "p39"),
        "the deepest Full Outside row must remain reachable; recorded: {:?}",
        probed.borrow().as_slice()
    );
}

#[gpui::test]
fn modal_outside_scroll_preserves_each_fitting_placement(cx: &mut TestAppContext) {
    for (placement, row_y) in [
        (ModalPlacement::Auto, 540.),
        (ModalPlacement::Center, 540.),
        (ModalPlacement::Top, 82.),
        (ModalPlacement::Bottom, 998.),
    ] {
        still();
        let hits = events();
        let probed = hits.clone();

        let cx = open_host(cx, move || {
            let recorded = hits.clone();
            Modal::new()
                .id("pl-modal-outside-fitting")
                .is_open(true)
                .placement(placement)
                .scroll(ModalScroll::Outside)
                .child(
                    gpui::div()
                        .id("pl-outside-fitting-probe")
                        .w_full()
                        .h(px(36.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_click(move |_, _, _| recorded.borrow_mut().push("hit".to_owned()))
                        .child("probe"),
                )
                .into_any_element()
        });

        // The test viewport is 1080px high. After the container's 40px
        // padding, the 84px panel (24 + 36 + 24) starts at y=498 for a
        // centered placement, y=40 for Top, and y=956 for Bottom.
        click(cx, 960., row_y);
        assert_eq!(
            probed.borrow().as_slice(),
            ["hit"],
            "{placement:?}: a fitting Outside dialog must keep its v3 placement"
        );
    }
}
