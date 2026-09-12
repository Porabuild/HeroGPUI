//! Focus rings follow keyboard-control modality, not HeroUI's Escape restore.
//!
//! Pointer open/close, pointer picks, and Escape after a pointer session must
//! leave the ring off. Tab, arrows, and Enter/Space turn it on; a later
//! pointer press turns it off again.

mod harness;

use gpui::{prelude::*, px, TestAppContext};
use harness::{click, events, open_host, press, still};
use herogpui_components::{util, Button, Dropdown, MenuItem, PickerItem, Select};

fn ring_on(cx: &mut gpui::VisualTestContext) -> bool {
    cx.update(|_, cx| util::focus_visible(cx))
}

fn last_open(log: &harness::Events) -> Option<String> {
    log.borrow().last().cloned()
}

#[gpui::test]
fn pointer_dropdown_escape_does_not_show_focus_ring(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        Dropdown::uncontrolled(
            "fv-dd-esc",
            Button::new("fv-dd-esc-trigger").label("Open"),
            vec![MenuItem::new("a", "Alpha"), MenuItem::new("b", "Beta")],
        )
        .id("fv-dd-esc")
        .on_open_change(move |open, _, _| {
            log.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:true"));
    assert!(
        !ring_on(cx),
        "opening with the pointer must not show a ring"
    );

    press(cx, "escape");
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:false"));
    assert!(
        !ring_on(cx),
        "Escape after a pointer session must not switch on the focus ring"
    );
}

#[gpui::test]
fn pointer_dropdown_toggle_close_does_not_show_focus_ring(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        Dropdown::uncontrolled(
            "fv-dd-toggle",
            Button::new("fv-dd-toggle-trigger").label("Open"),
            vec![MenuItem::new("a", "Alpha")],
        )
        .id("fv-dd-toggle")
        .on_open_change(move |open, _, _| {
            log.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:true"));
    click(cx, 40., 18.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:false"));
    assert!(
        !ring_on(cx),
        "closing with the same pointer must leave the trigger un-ringed"
    );
}

#[gpui::test]
fn pointer_dropdown_item_click_does_not_show_focus_ring(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let picked = events();
    let picks = picked.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        let picks = picks.clone();
        Dropdown::uncontrolled(
            "fv-dd-pick",
            Button::new("fv-dd-pick-trigger").label("Open"),
            vec![MenuItem::new("a", "Alpha"), MenuItem::new("b", "Beta")],
        )
        .id("fv-dd-pick")
        .on_open_change(move |open, _, _| {
            log.borrow_mut().push(format!("open:{open}"));
        })
        .on_action(move |key, _, _| {
            picks.borrow_mut().push(key.to_string());
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:true"));
    click(cx, 40., 64.);
    cx.update(|window, _| window.refresh());
    assert_eq!(picked.borrow().as_slice(), ["a"]);
    assert_eq!(last_open(&opened).as_deref(), Some("open:false"));
    assert!(
        !ring_on(cx),
        "a pointer pick must restore the trigger without a focus ring"
    );
}

#[gpui::test]
fn keyboard_dropdown_keeps_ring_after_escape(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        Dropdown::uncontrolled(
            "fv-dd-keys",
            Button::new("fv-dd-keys-trigger").label("Open"),
            vec![MenuItem::new("a", "Alpha")],
        )
        .id("fv-dd-keys")
        .on_open_change(move |open, _, _| {
            log.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    press(cx, "tab");
    assert!(ring_on(cx), "Tab must switch on the focus ring");
    press(cx, "enter");
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:true"));
    press(cx, "escape");
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:false"));
    assert!(
        ring_on(cx),
        "Escape must not clear a keyboard-session focus ring"
    );
}

#[gpui::test]
fn pointer_then_arrow_switches_to_keyboard(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        Dropdown::uncontrolled(
            "fv-dd-arrow",
            Button::new("fv-dd-arrow-trigger").label("Open"),
            vec![MenuItem::new("a", "Alpha"), MenuItem::new("b", "Beta")],
        )
        .id("fv-dd-arrow")
        .on_open_change(move |open, _, _| {
            log.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    cx.update(|window, _| window.refresh());
    assert!(!ring_on(cx));
    press(cx, "down");
    assert!(
        ring_on(cx),
        "arrowing a pointer-opened menu switches to keyboard controls"
    );
}

#[gpui::test]
fn pointer_press_hides_existing_keyboard_ring(cx: &mut TestAppContext) {
    still();
    let cx = open_host(cx, || {
        Button::new("fv-btn").label("Ready").into_any_element()
    });
    press(cx, "tab");
    assert!(ring_on(cx));
    click(cx, 40., 18.);
    assert!(
        !ring_on(cx),
        "continuing with the pointer must hide the keyboard ring"
    );
}

#[gpui::test]
fn pointer_select_escape_does_not_show_focus_ring(cx: &mut TestAppContext) {
    still();
    let opened = events();
    let log = opened.clone();
    let cx = open_host(cx, move || {
        let log = log.clone();
        gpui::div()
            .w(px(256.))
            .child(
                Select::new(
                    "fv-select",
                    vec![PickerItem::new("a", "Alpha"), PickerItem::new("b", "Beta")],
                )
                .on_open_change(move |open, _, _| {
                    log.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:true"));
    assert!(!ring_on(cx));
    press(cx, "escape");
    cx.update(|window, _| window.refresh());
    assert_eq!(last_open(&opened).as_deref(), Some("open:false"));
    assert!(
        !ring_on(cx),
        "Escape after a pointer-opened Select must not ring the trigger"
    );
}
