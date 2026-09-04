//! Behaviour tests for the Dropdown menu's viewport-edge correction.
//!
//! `util::placed_panel` positions a menu against its trigger and nothing else,
//! so a trigger near a window edge used to put part of the menu outside the
//! window: at a 600px width the described rows in the gallery's "With
//! Descriptions" demo were cut off mid-word.
//!
//! React Aria answers this on the *cross* axis by shifting the overlay until it
//! fits. From `calculatePosition`'s `getDelta` in the pinned react-aria
//! (3.51.0): "If any of the overlay edges falls outside of the boundary, shift
//! the overlay the required amount to align one of the overlay's edges with the
//! closest boundary edge", where the boundary is inset by `containerPadding` —
//! which `useOverlayPosition` defaults to 12 and v3 does not override.
//!
//! `viewport_shift` is that rule and has its own unit tests, including
//! the fixed-point property this component depends on. These tests assert the
//! part only a real window can show: that the Dropdown measures its own menu,
//! feeds the rule, and lands the menu inside the window — for the plain case,
//! for a menu wider than the window, for an end-aligned menu, and for a
//! composite widened by an open submenu.
//!
//! Main-axis flipping is outside this correction: `Placement::Left`
//! and `Placement::Right` put the menu beside the trigger, where horizontal
//! overflow is a *main*-axis problem that React Aria answers by flipping the
//! side (`shouldFlip`). This port implements flipping only for `Popover`, so a
//! left- or right-placed menu can still leave the window.

mod harness;

use gpui::{prelude::*, px, size, Pixels, TestAppContext, VisualTestContext};
use herogpui_components::{Button, Dropdown, MenuItem, Placement};

use harness::{click, open_host};

/// `OVERLAY_VIEWPORT_INSET` — react-aria's default `containerPadding`.
const INSET: f32 = 12.0;

/// Layout rounds to whole pixels, and `float_cmp` is denied.
fn near(a: Pixels, b: f32) -> bool {
    (f32::from(a) - b).abs() < 1.5
}

/// Sizes the window, then lets the measured shift settle.
///
/// The correction is measured rather than predicted, so the frame that first
/// lays the menu out is the one that reports its width. `viewport_shift` is a
/// fixed point, so one further frame is enough to land and stay — these extra
/// frames prove that rather than papering over a wobble.
fn settle(cx: &mut VisualTestContext, width: f32, height: f32) {
    cx.simulate_resize(size(px(width), px(height)));
    for _ in 0..4 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

fn describing_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("commit", "Create a merge commit")
            .description("All commits from this branch are added to the base branch"),
        MenuItem::new("squash", "Squash and merge")
            .description("The commits are combined into one"),
    ]
}

/// A trigger pushed to the right of a `pad`-wide spacer, so the menu it opens
/// would hang off the window's right edge.
fn host_with_trigger_at(
    cx: &mut TestAppContext,
    pad: f32,
    placement: Placement,
) -> &mut VisualTestContext {
    open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(pad)).h(px(1.)))
            .child(
                gpui::div()
                    .debug_selector(|| "dd-trigger".to_owned())
                    .child(
                        Dropdown::uncontrolled(
                            "ddv",
                            Button::new("ddv-trigger").label("Merge"),
                            describing_items(),
                        )
                        .id("dd-viewport")
                        .placement(placement),
                    ),
            )
            .into_any_element()
    })
}

/// The regression: at 600px the menu ran past the right edge. It must now end
/// on the inset instead, and it must have actually moved — a menu that merely
/// happened to fit would pass the first assertion on its own.
#[gpui::test]
fn a_menu_near_the_right_edge_shifts_inside_the_window(cx: &mut TestAppContext) {
    let width = 600.;
    let cx = host_with_trigger_at(cx, 420., Placement::BottomStart);
    settle(cx, width, 600.);
    click(cx, 460., 18.);
    settle(cx, width, 600.);

    let trigger = cx
        .debug_bounds("dd-trigger")
        .expect("the trigger must be laid out");
    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        f32::from(menu.origin.x + menu.size.width) <= width - INSET + 1.5,
        "the menu must end on the viewport inset, got {menu:?} in a {width}px window"
    );
    assert!(
        near(menu.origin.x + menu.size.width, width - INSET),
        "an overflowing menu lands *on* the inset rather than somewhere inside \
         it, got {menu:?}"
    );
    assert!(
        f32::from(menu.origin.x) < f32::from(trigger.origin.x),
        "the menu must have been pulled left of its trigger to fit: menu={menu:?} \
         trigger={trigger:?}"
    );
}

/// The other half of the same fix: a menu with room must not move at all, or
/// every dropdown in the app would drift. Start alignment puts its left edge on
/// the trigger's.
#[gpui::test]
fn a_menu_with_room_keeps_its_trigger_alignment(cx: &mut TestAppContext) {
    let cx = host_with_trigger_at(cx, 40., Placement::BottomStart);
    settle(cx, 1200., 600.);
    click(cx, 80., 18.);
    settle(cx, 1200., 600.);

    let trigger = cx
        .debug_bounds("dd-trigger")
        .expect("the trigger must be laid out");
    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        near(menu.origin.x, f32::from(trigger.origin.x)),
        "a menu that fits is positioned by its trigger alone: menu={menu:?} \
         trigger={trigger:?}"
    );
}

#[gpui::test]
fn a_centered_menu_tracks_the_viewport_when_resized(cx: &mut TestAppContext) {
    let cx = host_with_trigger_at(cx, 420., Placement::Bottom);
    settle(cx, 1200., 600.);
    click(cx, 460., 18.);
    settle(cx, 1200., 600.);

    for width in [1200., 520., 1200.] {
        settle(cx, width, 600.);
        let trigger = cx.debug_bounds("dd-trigger").unwrap();
        let menu = cx.debug_bounds("dropdown-menu").unwrap();
        assert!(menu.size.width > trigger.size.width);
        if width < 600. {
            assert!(near(menu.origin.x + menu.size.width, width - INSET));
        } else {
            assert!(near(
                menu.origin.x + menu.size.width / 2.,
                f32::from(trigger.origin.x + trigger.size.width / 2.),
            ));
        }
        for _ in 0..3 {
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds("dropdown-menu").unwrap(), menu);
        }
    }
}

/// An end-aligned menu is pinned by its *right* edge, so it overflows the
/// opposite way. The correction has to push it right, which is the arm that
/// subtracts the shift from the `right` inset.
#[gpui::test]
fn an_end_aligned_menu_at_the_left_edge_shifts_right(cx: &mut TestAppContext) {
    let cx = host_with_trigger_at(cx, 0., Placement::BottomEnd);
    settle(cx, 600., 600.);
    click(cx, 40., 18.);
    settle(cx, 600., 600.);

    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        f32::from(menu.origin.x) >= INSET - 1.5,
        "an end-aligned menu on a left-edge trigger must be pushed off the \
         window edge, got {menu:?}"
    );
    assert!(
        near(menu.origin.x, INSET),
        "and it lands on the inset, got {menu:?}"
    );
}

/// When the menu cannot fit at all, upstream's
/// `Math.max(endTerm, startTerm)` resolves to the start term: the start edge
/// wins and the overflow is left at the end, where a scroll or a clip is at
/// least predictable. A window narrower than `min-w-55` forces that branch.
#[gpui::test]
fn a_menu_wider_than_the_window_aligns_to_the_start_inset(cx: &mut TestAppContext) {
    let cx = host_with_trigger_at(cx, 60., Placement::BottomStart);
    settle(cx, 180., 600.);
    click(cx, 100., 18.);
    settle(cx, 180., 600.);

    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        f32::from(menu.size.width) > 180. - 2. * INSET,
        "this case is only meaningful while the menu is wider than the window \
         can hold, got {menu:?}"
    );
    assert!(
        near(menu.origin.x, INSET),
        "a menu that cannot fit aligns its start edge to the inset, got {menu:?}"
    );
}

/// A submenu is a flex *sibling* of its parent panel, so opening one widens the
/// composite the correction measures. The whole composite has to come inside,
/// not just the parent panel — which is why the Dropdown measures a wrapper
/// around the menu rather than the panel's own recorded bounds.
#[gpui::test]
fn an_open_submenu_widens_the_composite_and_it_stays_inside(cx: &mut TestAppContext) {
    let width = 620.;
    let actions = harness::events();
    let recorded = actions.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(400.)).h(px(1.)))
            .child(
                Dropdown::uncontrolled(
                    "ddvs",
                    Button::new("ddvs-trigger").label("More"),
                    vec![
                        MenuItem::new("plain", "Plain row"),
                        MenuItem::new("parent", "Share").submenu(vec![
                            MenuItem::new("mail", "Email a link"),
                            MenuItem::new("copy", "Copy the address"),
                        ]),
                    ],
                )
                .id("dd-viewport-sub")
                .item_content(|key, _| {
                    let selector = format!("ddv-item-{key}");
                    gpui::div()
                        .debug_selector(move || selector)
                        .child(key.clone())
                        .into_any_element()
                })
                .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string())),
            )
            .into_any_element()
    });
    settle(cx, width, 600.);
    click(cx, 440., 18.);
    settle(cx, width, 600.);

    let parent_only = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    let parent_row = cx.debug_bounds("ddv-item-parent").unwrap();
    click(
        cx,
        f32::from(parent_row.center().x),
        f32::from(parent_row.center().y),
    );
    settle(cx, width, 600.);

    let composite = cx
        .debug_bounds("dropdown-menu")
        .expect("the composite must be laid out");

    assert!(
        f32::from(composite.size.width) > f32::from(parent_only.size.width) + 20.,
        "the open submenu must widen the measured composite: parent={parent_only:?} \
         composite={composite:?}"
    );
    assert!(
        f32::from(composite.origin.x + composite.size.width) <= width - INSET + 1.5,
        "the widened composite must still end inside the viewport inset, got \
         {composite:?} in a {width}px window"
    );
    let child_row = cx.debug_bounds("ddv-item-mail").unwrap();
    click(
        cx,
        f32::from(child_row.center().x),
        f32::from(child_row.center().y),
    );
    cx.run_until_parked();
    assert_eq!(&*actions.borrow(), &["mail"]);
}
