//! Behaviour tests for the Dropdown menu's viewport correction.
//!
//! The Dropdown renders its menu in a popover positioned against the trigger,
//! following pinned React Aria 3.51.0 `calculatePosition` through RAC 1.20.0's
//! `Menu`/`Popover` (which passes no offset, so the gap is the Popover default
//! of 8px):
//!
//! - when the menu's natural size cannot fit on the preferred side and the
//!   opposite side has more room, the panel flips (`shouldFlip`);
//! - on the cross axis the panel shifts until it fits (`getDelta`), inset by
//!   `containerPadding` -- which `useOverlayPosition` defaults to 12 and v3
//!   does not override;
//! - the panel is capped at the available height (`getMaxHeight`) while a
//!   short menu keeps its natural height, and the popover -- not the menu --
//!   owns the overflow (`.dropdown__popover` is `overflow-y-auto` while
//!   `.dropdown__menu` is `overflow-clip`).
//!
//! Each submenu is its own popover against its row with `placement: 'end
//! top'` (pinned RAC 1.20.0 `Menu.js`): beside the row, top-aligned, with the
//! same 8px gap, flipping sides and capping height on its own. There is no
//! composite shift: the parent and the child are positioned independently,
//! and a press is outside only when it lands in neither panel's bounds, so a
//! click in the gap between them dismisses.
//!
//! These tests assert the part only a real window can show: that the Dropdown
//! measures its trigger, feeds the shared positioner, and lands each panel
//! inside the window -- for shifts, for flips on every side, for the height
//! cap with wheel access to the last row, for short menus, and for submenus.

mod harness;

use gpui::{
    point, prelude::*, px, size, Pixels, ScrollDelta, ScrollWheelEvent, TestAppContext,
    VisualTestContext,
};
use herogpui_components::{Button, Dropdown, MenuItem, Placement};

use harness::{click, open_host};

/// `containerPadding` -- react-aria's default, which v3 does not override.
const INSET: f32 = 12.0;
/// The RAC `Popover` default gap, which `Menu` inherits by passing no offset.
const GAP: f32 = 8.0;

/// Layout rounds to whole pixels, and `float_cmp` is denied.
fn near(a: Pixels, b: f32) -> bool {
    (f32::from(a) - b).abs() < 1.5
}

/// Sizes the window, then lets the measured position settle.
///
/// The correction is measured rather than predicted, so the frame that first
/// lays the menu out is the one that reports its size. These extra frames
/// prove the position is stable rather than papering over a wobble.
fn settle(cx: &mut VisualTestContext, width: f32, height: f32) {
    cx.simulate_resize(size(px(width), px(height)));
    for _ in 0..4 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

/// The position must be a fixed point: refreshing without input changes
/// nothing, or the panel would visibly oscillate.
///
/// Each frame is compared to the *first* reading, not to its neighbour, so a
/// panel drifting a pixel per frame still fails. Origin must stay exact --
/// movement is the thing being ruled out. Size gets `near`'s tolerance, because
/// a height-capped panel is laid out twice per frame by the positioner (once at
/// `MaxContent` to choose a side, once at the cap) and `debug_bounds` reports
/// whichever pass wrote last, so its height is ambiguous by the whole pixel the
/// cap rounds to. That is a measurement seam, not a wobble.
fn assert_settled(cx: &mut VisualTestContext, selector: &'static str) {
    let bounds = cx.debug_bounds(selector).unwrap();
    for _ in 0..3 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        let now = cx.debug_bounds(selector).unwrap();
        assert_eq!(
            (now.origin.x, now.origin.y),
            (bounds.origin.x, bounds.origin.y),
            "the panel moved between frames with no input: {now:?} vs {bounds:?}"
        );
        assert!(
            near(now.size.width, f32::from(bounds.size.width))
                && near(now.size.height, f32::from(bounds.size.height)),
            "the panel size moved by more than the cap's rounding between \
             frames with no input: {now:?} vs {bounds:?}"
        );
    }
}

/// One simulated wheel event: negative `dy` scrolls down, matching the
/// scrollable element's `scroll_offset.y += delta.y`. GPUI routes a wheel to
/// the scroller under the pointer, so the pointer moves there first -- the
/// event position alone does not retarget the cached hit test.
fn wheel(cx: &mut VisualTestContext, x: f32, y: f32, dy: f32) {
    use gpui::Modifiers;
    cx.simulate_mouse_move(point(px(x), px(y)), None, Modifiers::none());
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(x), px(y)),
        delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
        ..Default::default()
    });
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

/// Moves the test clock past the Dropdown's 100ms exit phase, so a
/// closed-proof probe cannot land on the exiting panel.
fn let_exit_finish(cx: &mut TestAppContext) {
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(150));
}

fn describing_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new("commit", "Create a merge commit")
            .description("All commits from this branch are added to the base branch"),
        MenuItem::new("squash", "Squash and merge")
            .description("The commits are combined into one"),
    ]
}

fn plain_items(count: usize) -> Vec<MenuItem> {
    (0..count)
        .map(|i| MenuItem::new(format!("item-{i}"), format!("Item {i}")))
        .collect()
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

/// A trigger pushed down by a tall spacer, so a bottom-placed menu cannot fit
/// below it.
fn host_with_trigger_low(
    cx: &mut TestAppContext,
    spacer: f32,
    placement: Placement,
) -> &mut VisualTestContext {
    open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .child(gpui::div().w(px(1.)).h(px(spacer)))
            .child(
                gpui::div()
                    .debug_selector(|| "dd-trigger".to_owned())
                    .child(
                        Dropdown::uncontrolled(
                            "ddv-low",
                            Button::new("ddv-low-trigger").label("Merge"),
                            describing_items(),
                        )
                        .id("dd-viewport-low")
                        .placement(placement),
                    ),
            )
            .into_any_element()
    })
}

/// A trigger floated mid-window, so a side-placed menu has room to centre on
/// it without hitting the top or bottom inset.
fn host_with_trigger_mid(
    cx: &mut TestAppContext,
    pad: f32,
    placement: Placement,
) -> &mut VisualTestContext {
    open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .child(gpui::div().w(px(1.)).h(px(200.)))
            .child(
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
                                    "ddv-mid",
                                    Button::new("ddv-mid-trigger").label("Merge"),
                                    describing_items(),
                                )
                                .id("dd-viewport-mid")
                                .placement(placement),
                            ),
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
    assert_settled(cx, "dropdown-menu");
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
    assert!(
        near(
            menu.origin.y,
            f32::from(trigger.origin.y + trigger.size.height + px(GAP))
        ),
        "the menu hangs one RAC gap below the trigger: menu={menu:?} \
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
/// keeps the shift on the `right` inset.
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

/// The main-axis contract: a bottom-placed menu that cannot fit below its
/// trigger opens above it when there is more room there, one gap clear.
#[gpui::test]
fn a_menu_near_the_bottom_edge_flips_above_its_trigger(cx: &mut TestAppContext) {
    let cx = host_with_trigger_low(cx, 480., Placement::BottomStart);
    settle(cx, 600., 600.);
    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    click(
        cx,
        f32::from(trigger.center().x),
        f32::from(trigger.center().y),
    );
    settle(cx, 600., 600.);

    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    let menu = cx.debug_bounds("dropdown-menu").unwrap();

    assert!(
        f32::from(menu.origin.y + menu.size.height) <= f32::from(trigger.origin.y) - GAP + 1.5,
        "a flipped menu must sit above its trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(
            menu.origin.y + menu.size.height,
            f32::from(trigger.origin.y) - GAP
        ),
        "and it hangs one gap above the trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(menu.origin.x, INSET),
        "the cross-axis shift still applies after a flip: a menu wider than \
         its trigger lands on the start inset, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        f32::from(menu.origin.y) >= INSET - 1.5,
        "the flipped menu must still clear the top inset, got {menu:?}"
    );
    assert_settled(cx, "dropdown-menu");
}

/// The mirror: a top-placed menu with no room above opens below its trigger.
#[gpui::test]
fn a_top_placed_menu_near_the_top_flips_below(cx: &mut TestAppContext) {
    let cx = host_with_trigger_at(cx, 40., Placement::TopStart);
    settle(cx, 600., 600.);
    click(cx, 80., 18.);
    settle(cx, 600., 600.);

    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    let menu = cx.debug_bounds("dropdown-menu").unwrap();

    assert!(
        f32::from(menu.origin.y) >= f32::from(trigger.origin.y + trigger.size.height),
        "a flipped menu must sit below its trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(
            menu.origin.y,
            f32::from(trigger.origin.y + trigger.size.height + px(GAP))
        ),
        "and it hangs one gap below the trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert_settled(cx, "dropdown-menu");
}

/// Side placements flip the same way: a right-placed menu with no room on the
/// right opens on the left, one gap clear, vertically centred on its trigger.
#[gpui::test]
fn a_right_placed_menu_near_the_right_edge_flips_left(cx: &mut TestAppContext) {
    let width = 600.;
    let cx = host_with_trigger_mid(cx, 420., Placement::Right);
    settle(cx, width, 600.);
    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    click(
        cx,
        f32::from(trigger.center().x),
        f32::from(trigger.center().y),
    );
    settle(cx, width, 600.);

    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    let menu = cx.debug_bounds("dropdown-menu").unwrap();

    assert!(
        f32::from(menu.origin.x + menu.size.width) <= f32::from(trigger.origin.x) - GAP + 1.5,
        "a flipped side menu must sit left of its trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(
            menu.origin.x + menu.size.width,
            f32::from(trigger.origin.x) - GAP
        ),
        "and it hangs one gap off the trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        f32::from(menu.origin.x) >= INSET - 1.5,
        "the flipped menu must still clear the start inset, got {menu:?}"
    );
    assert!(
        near(
            menu.origin.y + menu.size.height / 2.,
            f32::from(trigger.center().y)
        ),
        "flipping keeps the centre alignment: menu={menu:?} trigger={trigger:?}"
    );
    assert_settled(cx, "dropdown-menu");
}

/// And a left-placed menu with no room on the left opens on the right.
#[gpui::test]
fn a_left_placed_menu_near_the_left_edge_flips_right(cx: &mut TestAppContext) {
    let cx = host_with_trigger_mid(cx, 0., Placement::Left);
    settle(cx, 600., 600.);
    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    click(
        cx,
        f32::from(trigger.center().x),
        f32::from(trigger.center().y),
    );
    settle(cx, 600., 600.);

    let trigger = cx.debug_bounds("dd-trigger").unwrap();
    let menu = cx.debug_bounds("dropdown-menu").unwrap();

    assert!(
        f32::from(menu.origin.x) >= f32::from(trigger.origin.x + trigger.size.width),
        "a flipped side menu must sit right of its trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(
            menu.origin.x,
            f32::from(trigger.origin.x + trigger.size.width + px(GAP))
        ),
        "and it hangs one gap off the trigger, got menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        f32::from(menu.origin.x + menu.size.width) <= 600. - INSET + 1.5,
        "the flipped menu must still clear the end inset, got {menu:?}"
    );
    assert_settled(cx, "dropdown-menu");
}

/// The height contract: a tall menu is capped at the available height past the
/// gap and inset, stays inside the window, and still lets the pointer reach
/// its last row through its own scroller.
#[gpui::test]
fn a_tall_menu_caps_to_the_available_height_and_wheels_to_its_last_row(cx: &mut TestAppContext) {
    let height = 420.;
    let actions = harness::events();
    let recorded = actions.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(40.)).h(px(1.)))
            .child(
                gpui::div()
                    .debug_selector(|| "dd-trigger".to_owned())
                    .child(
                        Dropdown::uncontrolled(
                            "ddv-tall",
                            Button::new("ddv-tall-trigger").label("Many"),
                            plain_items(30),
                        )
                        .id("dd-viewport-tall")
                        .item_content(|key, _| {
                            let selector = format!("ddv-tall-item-{key}");
                            gpui::div()
                                .debug_selector(move || selector)
                                .child(key.clone())
                                .into_any_element()
                        })
                        .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string())),
                    ),
            )
            .into_any_element()
    });
    settle(cx, 600., height);
    click(cx, 80., 18.);
    settle(cx, 600., height);

    let trigger = cx.debug_bounds("dd-trigger").unwrap_or_else(|| {
        panic!("the trigger must be laid out");
    });
    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        near(
            menu.origin.y,
            f32::from(trigger.origin.y + trigger.size.height + px(GAP))
        ),
        "a capped menu still hangs one gap below its trigger: menu={menu:?} \
         trigger={trigger:?}"
    );
    assert!(
        near(menu.origin.y + menu.size.height, height - INSET),
        "and its bottom lands on the viewport inset: got {menu:?} in a \
         {height}px window"
    );
    assert!(
        f32::from(menu.size.height) < 500.,
        "thirty rows must be capped, not laid out at their natural height: \
         got {menu:?}"
    );

    // The last row is laid out but clipped outside the capped panel. A wheel
    // over the menu scrolls the menu's own scroller until the row is
    // hit-testable, and activating it reports its key.
    let last_before = cx
        .debug_bounds("ddv-tall-item-item-29")
        .expect("all plain rows are laid out, even clipped ones");
    assert!(
        f32::from(last_before.origin.y + last_before.size.height)
            > f32::from(menu.origin.y + menu.size.height) + 1.5,
        "this case is only meaningful while the last row starts off-panel: \
         row={last_before:?} menu={menu:?}"
    );
    wheel(
        cx,
        f32::from(menu.center().x),
        f32::from(menu.center().y),
        -10000.,
    );
    settle(cx, 600., height);
    let last = cx
        .debug_bounds("ddv-tall-item-item-29")
        .expect("the last row must scroll into view");
    assert!(
        f32::from(last.origin.y) >= f32::from(menu.origin.y) - 1.5
            && f32::from(last.origin.y + last.size.height)
                <= f32::from(menu.origin.y + menu.size.height) + 1.5,
        "the scrolled last row must sit inside the capped panel: row={last:?} \
         menu={menu:?}"
    );
    click(cx, f32::from(last.center().x), f32::from(last.center().y));
    assert_eq!(
        &*actions.borrow(),
        &["item-29"],
        "activating the scrolled last row must report its key"
    );
}

/// A short menu keeps its natural height: the cap is a maximum, not a size.
#[gpui::test]
fn a_short_menu_keeps_its_natural_height(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(40.)).h(px(1.)))
            .child(
                Dropdown::uncontrolled(
                    "ddv-short",
                    Button::new("ddv-short-trigger").label("Few"),
                    plain_items(2),
                )
                .id("dd-viewport-short"),
            )
            .into_any_element()
    });
    settle(cx, 600., 600.);
    click(cx, 80., 18.);
    settle(cx, 600., 600.);

    let menu = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");

    assert!(
        f32::from(menu.size.height) < 200. && f32::from(menu.size.height) > 50.,
        "two rows are laid out at their natural height -- neither stretched \
         toward the available space nor collapsed: got {menu:?}"
    );
}

/// A submenu is its own popover, not part of a measured composite: the parent
/// and the child each land inside the window, the child one gap beside its
/// row, and activating a child row reports its key.
#[gpui::test]
fn an_open_submenu_is_its_own_popover_and_stays_inside(cx: &mut TestAppContext) {
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

    let parent = cx
        .debug_bounds("dropdown-menu")
        .expect("the open menu must be laid out");
    assert!(
        f32::from(parent.origin.x + parent.size.width) <= width - INSET + 1.5,
        "the parent panel must end inside the viewport inset, got {parent:?}"
    );

    let parent_row = cx.debug_bounds("ddv-item-parent").unwrap();
    click(
        cx,
        f32::from(parent_row.center().x),
        f32::from(parent_row.center().y),
    );
    settle(cx, width, 600.);

    // The parent panel keeps its own bounds: opening a submenu does not widen
    // a composite, because each popover is positioned on its own.
    let parent_after = cx.debug_bounds("dropdown-menu").unwrap();
    assert_eq!(
        parent_after, parent,
        "the parent panel must not move when its submenu opens"
    );
    let child = cx
        .debug_bounds("dropdown-submenu")
        .expect("the open submenu must be laid out");
    assert!(
        f32::from(child.origin.x) >= INSET - 1.5
            && f32::from(child.origin.x + child.size.width) <= width - INSET + 1.5
            && f32::from(child.origin.y) >= INSET - 1.5
            && f32::from(child.origin.y + child.size.height) <= 600. - INSET + 1.5,
        "the submenu panel must sit fully inside the window, got {child:?}"
    );
    // No room on the right (the parent already ends on the inset), so the
    // submenu flips to the row's start side, one gap clear. Rows start at the
    // panel's p-1.5 inset, so the flipped edge sits just inside the parent's
    // own left edge.
    assert!(
        near(
            child.origin.x + child.size.width,
            f32::from(parent.origin.x) + 6. - GAP
        ),
        "a submenu with no room on its end side flips to the start side: \
         child={child:?} parent={parent:?} row={parent_row:?}"
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

/// A tall submenu near a low row shifts up while it can, then caps and
/// scrolls: the last child stays reachable through the submenu's own wheel.
#[gpui::test]
fn a_tall_submenu_near_a_low_row_caps_and_wheels_to_its_last_child(cx: &mut TestAppContext) {
    let (width, height) = (600., 500.);
    let actions = harness::events();
    let recorded = actions.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let mut items = plain_items(8);
        items.push(
            MenuItem::new("more", "More").submenu(
                (0..14)
                    .map(|i| MenuItem::new(format!("sub-{i}"), format!("Submenu {i}")))
                    .collect(),
            ),
        );
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(40.)).h(px(1.)))
            .child(
                Dropdown::uncontrolled(
                    "ddvs-tall",
                    Button::new("ddvs-tall-trigger").label("Many"),
                    items,
                )
                .id("dd-viewport-sub-tall")
                .item_content(|key, _| {
                    let selector = format!("ddv-sub-item-{key}");
                    gpui::div()
                        .debug_selector(move || selector)
                        .child(key.clone())
                        .into_any_element()
                })
                .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string())),
            )
            .into_any_element()
    });
    settle(cx, width, height);
    click(cx, 80., 18.);
    settle(cx, width, height);

    let parent_row = cx.debug_bounds("ddv-sub-item-more").unwrap();
    click(
        cx,
        f32::from(parent_row.center().x),
        f32::from(parent_row.center().y),
    );
    settle(cx, width, height);

    let child = cx
        .debug_bounds("dropdown-submenu")
        .expect("the open submenu must be laid out");
    assert!(
        f32::from(child.origin.y) >= INSET - 1.5
            && f32::from(child.origin.y + child.size.height) <= height - INSET + 1.5,
        "a tall submenu must stay inside the vertical insets, got {child:?} \
         in a {height}px window"
    );
    assert!(
        near(child.origin.y, INSET),
        "a submenu that cannot fit even shifted aligns to the start inset, \
         got {child:?}"
    );
    assert!(
        near(child.origin.y + child.size.height, height - INSET),
        "and it caps at the available height, got {child:?}"
    );

    let last_selector = "ddv-sub-item-sub-13";
    let last_before = cx
        .debug_bounds(last_selector)
        .expect("all plain rows are laid out, even clipped ones");
    assert!(
        f32::from(last_before.origin.y + last_before.size.height)
            > f32::from(child.origin.y + child.size.height) + 1.5,
        "this case is only meaningful while the last child starts off-panel: \
         row={last_before:?} submenu={child:?}"
    );
    wheel(
        cx,
        f32::from(child.center().x),
        f32::from(child.center().y),
        -10000.,
    );
    settle(cx, width, height);
    let last = cx
        .debug_bounds(last_selector)
        .expect("the last child must scroll into view");
    click(cx, f32::from(last.center().x), f32::from(last.center().y));
    assert_eq!(
        &*actions.borrow(),
        &["sub-13"],
        "activating the scrolled last child must report its key"
    );
}

/// The gap between the parent and its submenu belongs to neither panel, so a
/// press there is outside and dismisses without activating anything.
#[gpui::test]
fn a_click_in_the_submenu_gap_dismisses_without_action(cx: &mut TestAppContext) {
    let actions = harness::events();
    let recorded = actions.clone();
    let opens = harness::events();
    let opened = opens.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let opened = opened.clone();
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(40.)).h(px(1.)))
            .child(
                Dropdown::uncontrolled(
                    "ddvs-gap",
                    Button::new("ddvs-gap-trigger").label("More"),
                    vec![
                        MenuItem::new("plain", "Plain row"),
                        MenuItem::new("parent", "Share").submenu(vec![
                            MenuItem::new("mail", "Email a link"),
                            MenuItem::new("copy", "Copy the address"),
                        ]),
                    ],
                )
                .id("dd-viewport-sub-gap")
                .item_content(|key, _| {
                    let selector = format!("ddv-gap-item-{key}");
                    gpui::div()
                        .debug_selector(move || selector)
                        .child(key.clone())
                        .into_any_element()
                })
                .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
                .on_open_change(move |open, _, _| {
                    opened.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .into_any_element()
    });
    settle(cx, 1200., 600.);
    click(cx, 80., 18.);
    settle(cx, 1200., 600.);

    let parent_row = cx.debug_bounds("ddv-gap-item-parent").unwrap();
    click(
        cx,
        f32::from(parent_row.center().x),
        f32::from(parent_row.center().y),
    );
    settle(cx, 1200., 600.);

    let parent = cx.debug_bounds("dropdown-menu").unwrap();
    let child = cx.debug_bounds("dropdown-submenu").unwrap();
    // The submenu opens on the row's end side here, where the wide window
    // leaves room. The gap is measured from the row, which starts at the
    // parent panel's p-1.5 inset -- so the inter-panel gap is the 8px offset
    // minus that inset, and its midpoint is outside both panels.
    let gap_lo = f32::from(parent.origin.x + parent.size.width);
    let gap_hi = f32::from(child.origin.x);
    assert!(
        gap_hi - gap_lo >= 1.5,
        "this case needs a real gap between the panels: parent={parent:?} \
         child={child:?}"
    );
    let gap_x = (gap_lo + gap_hi) / 2.;
    let gap_y = f32::from(child.center().y);
    click(cx, gap_x, gap_y);
    let_exit_finish(cx);
    settle(cx, 1200., 600.);

    assert!(
        actions.borrow().is_empty(),
        "a gap press must not activate any row"
    );
    assert_eq!(
        opens.borrow().last().map(String::as_str),
        Some("open:false"),
        "a gap press must dismiss the menu"
    );
    assert!(
        cx.debug_bounds("dropdown-menu").is_none(),
        "the dismissed menu must leave the tree"
    );
}

/// A press far from both panels dismisses an open submenu the same way.
#[gpui::test]
fn a_click_outside_with_a_submenu_open_dismisses_without_action(cx: &mut TestAppContext) {
    let actions = harness::events();
    let recorded = actions.clone();
    let opens = harness::events();
    let opened = opens.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let opened = opened.clone();
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .child(gpui::div().w(px(40.)).h(px(1.)))
            .child(
                Dropdown::uncontrolled(
                    "ddvs-out",
                    Button::new("ddvs-out-trigger").label("More"),
                    vec![
                        MenuItem::new("plain", "Plain row"),
                        MenuItem::new("parent", "Share").submenu(vec![
                            MenuItem::new("mail", "Email a link"),
                            MenuItem::new("copy", "Copy the address"),
                        ]),
                    ],
                )
                .id("dd-viewport-sub-out")
                .item_content(|key, _| {
                    let selector = format!("ddv-out-item-{key}");
                    gpui::div()
                        .debug_selector(move || selector)
                        .child(key.clone())
                        .into_any_element()
                })
                .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
                .on_open_change(move |open, _, _| {
                    opened.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .into_any_element()
    });
    settle(cx, 1200., 600.);
    click(cx, 80., 18.);
    settle(cx, 1200., 600.);

    let parent_row = cx.debug_bounds("ddv-out-item-parent").unwrap();
    click(
        cx,
        f32::from(parent_row.center().x),
        f32::from(parent_row.center().y),
    );
    settle(cx, 1200., 600.);
    assert!(
        cx.debug_bounds("dropdown-submenu").is_some(),
        "the submenu must be open before the outside press"
    );

    click(cx, 1100., 500.);
    let_exit_finish(cx);
    settle(cx, 1200., 600.);

    assert!(
        actions.borrow().is_empty(),
        "an outside press must not activate any row"
    );
    assert_eq!(
        opens.borrow().last().map(String::as_str),
        Some("open:false"),
        "an outside press must dismiss the menu"
    );
    assert!(
        cx.debug_bounds("dropdown-menu").is_none(),
        "the dismissed menu must leave the tree"
    );
}
