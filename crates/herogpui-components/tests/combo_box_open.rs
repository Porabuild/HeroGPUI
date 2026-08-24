//! Behaviour tests for what opens a ComboBox's suggestion list.
//!
//! The pickers suite drives ComboBox from the chevron; these tests cover the
//! two `menuTrigger` paths it never exercised, the defect this file exists
//! for: typing opened nothing with the default input trigger, and the focus
//! trigger did not open at all. Everything is asserted on recorded callbacks
//! and behavioural probes -- never on appearance.
//!
//! Geometry is borrowed from tests/pickers.rs: the trigger field is a 36px
//! row at the window origin (centre (60, 18)), the chevron sits at the right
//! end of the 320px-wide field (x = 298), and the panel's `p(4)` puts row *i*
//! at y 64+36i with ≤4px of entry-zoom padding, which that y covers in every
//! phase of the animation.
//!
//! Reduce motion is not set: the ComboBox panel leaves the tree outright when
//! the list closes (no exit phase), so a probe click where a row *would* be is
//! a safe "is it closed" check.

mod harness;

use gpui::{prelude::*, TestAppContext};
use herogpui_components::{ComboBox, InputState, MenuTrigger};

use harness::{click, events, open_host, press};

/// An `InputState` entity, created before the host opens so the test can keep
/// its own handle to it.
fn combo_state(cx: &mut TestAppContext) -> gpui::Entity<InputState> {
    cx.new(|cx| InputState::new(cx))
}

#[gpui::test]
fn combo_box_typing_opens_the_list(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        // No `menu_trigger` builder: the default is `MenuTrigger::Input`.
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // Clicking into the field focuses it. With the input trigger the focus is
    // not the gesture -- the first non-empty edit is.
    click(cx, 60., 18.);
    assert!(
        opened.borrow().is_empty(),
        "focus alone must not open the list under `MenuTrigger::Input`"
    );

    // The failing reproduction: typing "ty" must open the list, because the
    // suggestion that matches the text lives inside it.
    cx.simulate_input("ty");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the first non-empty edit must open the list"
    );

    // Row 0 is now drawn at y = 64 and records the pick.
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "clicking the matching suggestion must select it"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "picking must close the list"
    );
}

#[gpui::test]
fn combo_box_focus_trigger_opens_on_focus(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .menu_trigger(MenuTrigger::Focus)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // The gesture is the focus: no keystroke and no chevron. Clicking into
    // the field opens the list on that very frame.
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the field taking focus must open the list"
    );

    // Row 0 is already drawn without any typing, and records the pick.
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "clicking the first suggestion must select it"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "picking must close the list"
    );
}

#[gpui::test]
fn combo_box_manual_trigger_ignores_typing(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .menu_trigger(MenuTrigger::Manual)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("ty");
    assert!(
        opened.borrow().is_empty(),
        "typing must not open a `MenuTrigger::Manual` list"
    );

    // The chevron is still a trigger: x = 320 - 12px field padding - half the
    // 20px button box, at the field's vertical centre.
    click(cx, 298., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the chevron must still open a manual list"
    );

    // Manual keeps the full collection on screen, so row 0 is still "Typst".
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "clicking the first row must select it"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn combo_box_focus_trigger_stays_closed_after_escape(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .menu_trigger(MenuTrigger::Focus)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Escape closes the list and leaves the focus on the field. The one-shot
    // under test is what must keep the panel closed afterwards: were the
    // focus-open check answering the still-held focus every frame, the next
    // render would reopen it.
    press(cx, "escape");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "escape must close the list"
    );

    // The probe is a click where row 0 was. If the panel had come back, the
    // click records "Typst"; a bare page records nothing.
    click(cx, 60., 64.);
    assert!(
        recorded.borrow().is_empty(),
        "the list must stay closed after escape while the field keeps the focus"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "no reopen after escape"
    );
}
