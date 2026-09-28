//! `ContextMenu` (HeroGPUI extension): a secondary press opens the shared
//! Dropdown `Menu` at the pointer; choosing an item, Escape or a press
//! outside closes it; a disabled context menu ignores the press.

mod harness;

use gpui::{
    point, prelude::*, px, Bounds, Modifiers, MouseButton, Pixels, TestAppContext,
    VisualTestContext,
};
use harness::{events, open_host, press, still, Events};
use herogpui_components::{ContextMenu, MenuItem};

const PANEL: &str = "context-menu";

fn host(cx: &mut TestAppContext, disabled: bool) -> (Events, &mut VisualTestContext) {
    still();
    let seen = events();
    let s = seen.clone();
    let cx = open_host(cx, move || {
        let (a, o) = (s.clone(), s.clone());
        ContextMenu::new(
            "ctx",
            gpui::div().w(px(400.)).h(px(300.)),
            vec![
                MenuItem::new("copy", "Copy"),
                MenuItem::new("paste", "Paste"),
                MenuItem::new("delete", "Delete").danger(),
            ],
        )
        .is_disabled(disabled)
        .disabled_keys(["paste"])
        .on_action(move |key, _, _| a.borrow_mut().push(format!("action:{key}")))
        .on_open_change(move |open, _, _| o.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });
    (seen, cx)
}

fn right_click(cx: &mut VisualTestContext, x: f32, y: f32) {
    let at = point(px(x), px(y));
    cx.simulate_mouse_down(at, MouseButton::Right, Modifiers::none());
    cx.simulate_mouse_up(at, MouseButton::Right, Modifiers::none());
    frame(cx);
}

fn frame(cx: &mut VisualTestContext) {
    // Past any exit run (a closing menu stays mounted for its motion).
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(300));
    for _ in 0..3 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

fn panel(cx: &mut VisualTestContext) -> Option<Bounds<Pixels>> {
    cx.debug_bounds(PANEL)
}

#[gpui::test]
fn secondary_press_opens_at_the_pointer(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, false);
    frame(cx);
    assert!(panel(cx).is_none(), "closed until a secondary press");
    cx.simulate_click(point(px(100.), px(80.)), Modifiers::none());
    frame(cx);
    assert!(panel(cx).is_none(), "a primary press does not open it");

    right_click(cx, 100., 80.);
    let bounds = panel(cx).expect("the menu opened");
    assert_eq!(bounds.origin, point(px(100.), px(80.)));
    assert_eq!(seen.borrow().as_slice(), ["open:true"]);
}

#[gpui::test]
fn choosing_an_item_acts_and_closes(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, false);
    frame(cx);
    right_click(cx, 60., 40.);
    let bounds = panel(cx).unwrap();
    // The first row sits at the top of the panel, inside its padding.
    cx.simulate_click(bounds.origin + point(px(24.), px(18.)), Modifiers::none());
    frame(cx);
    assert!(
        seen.borrow().contains(&"action:copy".to_owned()),
        "{:?}",
        seen.borrow()
    );
    assert!(
        seen.borrow().contains(&"open:false".to_owned()),
        "{:?}",
        seen.borrow()
    );
    assert!(panel(cx).is_none());
}

#[gpui::test]
fn escape_and_outside_press_close(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, false);
    frame(cx);
    right_click(cx, 60., 40.);
    assert!(panel(cx).is_some());
    press(cx, "escape");
    frame(cx);
    assert!(panel(cx).is_none(), "Escape closes");

    right_click(cx, 60., 40.);
    assert!(panel(cx).is_some());
    cx.simulate_click(point(px(390.), px(290.)), Modifiers::none());
    frame(cx);
    assert!(panel(cx).is_none(), "an outside press closes");
    assert!(!seen.borrow().iter().any(|e| e.starts_with("action:")));
}

#[gpui::test]
fn disabled_ignores_the_secondary_press(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, true);
    frame(cx);
    right_click(cx, 60., 40.);
    assert!(panel(cx).is_none());
    assert!(seen.borrow().is_empty());
}
