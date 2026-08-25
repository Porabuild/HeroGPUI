//! Behaviour tests for what opens a ComboBox's suggestion list.
//!
//! The pickers suite drives ComboBox from the chevron; these tests cover the
//! `menuTrigger` paths it never exercised. The regressions preserved here are
//! the v3 default Focus trigger, explicit Input and Manual behavior, dismissal
//! and reopening, read-only inertness, and keyboard-open reports. Everything
//! is asserted on recorded callbacks and behavioral probes -- never appearance.
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

use gpui::{prelude::*, Focusable, TestAppContext};
use herogpui_components::{ComboBox, Input, InputState, MenuTrigger};

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
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .menu_trigger(MenuTrigger::Input)
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
fn combo_box_default_trigger_opens_on_focus(cx: &mut TestAppContext) {
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
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // v3 defaults menuTrigger to Focus: no keystroke, chevron or builder.
    // Clicking into the field opens the list on that very frame.
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the default trigger must open when the field takes focus"
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
fn combo_box_focus_open_shows_all_items_before_the_next_edit(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .default_input_value("ru")
        .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
        .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "a Focus open must show the original collection, not the filtered query"
    );
}

#[gpui::test]
fn combo_box_chevron_open_shows_all_items_for_an_input_trigger(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .default_input_value("ru")
        .menu_trigger(MenuTrigger::Input)
        .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
        .into_any_element()
    });

    click(cx, 298., 18.);
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "a chevron press is a manual open and must show the original collection"
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

    // The manual press opens the full collection. A subsequent edit switches
    // back to filtered results while keeping the already-open list visible.
    press(cx, "ctrl-a");
    cx.simulate_input("go");
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Go"],
        "typing in an open manual list must replace show-all with filtered rows"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn combo_box_focus_trigger_reopens_after_a_later_edit(cx: &mut TestAppContext) {
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

    // Force another frame while focus remains in the field. The focus-open
    // one-shot must not answer the still-held focus by reopening the panel.
    cx.update(|window, _| window.refresh());
    assert!(recorded.borrow().is_empty());
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "no reopen after escape"
    );

    cx.simulate_input("t");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "a later edit must reopen a dismissed Focus-triggered list"
    );
}

#[gpui::test]
fn combo_box_escape_does_not_steal_a_later_pointer_focus(cx: &mut TestAppContext) {
    let combo = combo_state(cx);
    let other = combo_state(cx);
    let combo_for_view = combo;
    let other_for_view = other.clone();
    let cx = open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_col()
            .child(
                ComboBox::new(
                    combo_for_view.clone(),
                    vec!["Typst".into(), "Rust".into(), "Go".into()],
                )
                .into_any_element(),
            )
            .child(Input::new(other_for_view.clone()).into_any_element())
            .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "escape");
    click(cx, 60., 54.);
    assert!(
        cx.update(|window, cx| other.read(cx).focus_handle(cx).is_focused(window)),
        "Escape dismissal must not reclaim focus from a later pointer target"
    );
}

#[gpui::test]
fn combo_box_manual_no_match_edit_closes_logical_open_state(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .menu_trigger(MenuTrigger::Manual)
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 298., 18.);
    cx.simulate_input("zz");
    press(cx, "escape");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "an unmatched edit must close Manual state once, not leave a hidden overlay"
    );
}

#[gpui::test]
fn combo_box_arrow_open_shows_all_items_for_an_input_trigger(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .default_input_value("zz")
        .menu_trigger(MenuTrigger::Input)
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "ArrowDown is a manual open and must use the original collection"
    );
}

#[gpui::test]
fn combo_box_read_only_refuses_focus_and_chevron_open(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .is_read_only(true)
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 298., 18.);
    assert!(
        opened.borrow().is_empty(),
        "a read-only ComboBox must not open from focus or its chevron"
    );
}

#[gpui::test]
fn combo_box_arrow_open_reports_to_a_controlled_owner(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = combo_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .is_open(false)
        .menu_trigger(MenuTrigger::Manual)
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "ArrowDown must report a manual open even when the owner has not accepted it"
    );
}
