//! Behaviour tests for the Dropdown's close-on-activate contract.
//!
//! v3's Dropdown delegates its behaviour to React Aria's Menu: activating an
//! item closes the menu (`selectionMode` `"none"` or `"single"`), a
//! `"multiple"` pick leaves it open so several items can be ticked, and a
//! submenu trigger row opens its child panel instead of ending the menu.
//! `pickers.rs` drives the Dropdown by keyboard only; these tests assert the
//! close itself, on both the pointer and the keyboard path.
//!
//! Geometry (see `pickers.rs`): a `Button` trigger is 36px tall, so its centre
//! is (40, 18); the menu panel hangs 6px below it and its rows centre at
//! y = 64 + 36*i over a 220px-wide panel.
//!
//! Every dropdown gets its own id: two components sharing one share their
//! keyed state, which AGENTS.md records as a silent failure.
//!
//! One harness fact drives the closed-proof probes: the Dropdown plays its
//! `[data-exiting]` run through `util::overlay_phase`, whose timer waits on the
//! real clock. Under the test clock the exiting panel would stay mounted
//! forever, so a probe that must not hit the old rows advances the clock past
//! the 100ms exit before clicking.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{prelude::*, SharedString, TestAppContext};
use herogpui_components::{Button, Dropdown, MenuItem, SelectionMode};

use harness::{click, events, open_host, press};

/// Moves the test clock past the Dropdown's 100ms exit phase, so a
/// closed-proof click cannot land on the exiting panel.
fn let_exit_finish(cx: &mut TestAppContext) {
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(150));
}

#[gpui::test]
fn click_activates_once_and_closes(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            Button::new("ddc-trigger").label("Actions"),
            vec![MenuItem::new("one", "One"), MenuItem::new("two", "Two")],
        )
        .id("dd-close")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the trigger must open the menu"
    );

    click(cx, 40., 64.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "clicking the first row must record its key exactly once"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the pick must dismiss the menu"
    );

    // Closed proof by behaviour: the same spot is bare page below the trigger
    // now. Were the menu still open -- or reopened -- the row would record a
    // second "one" here.
    let_exit_finish(cx);
    click(cx, 40., 64.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "the menu must be gone after the pick: a second click on the row \
         records nothing"
    );
}

#[gpui::test]
fn enter_activates_once_and_closes(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            Button::new("ddk-trigger").label("Actions"),
            vec![MenuItem::new("one", "One"), MenuItem::new("two", "Two")],
        )
        .id("dd-key")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    // Opening moves the focus into the panel, so the arrows work without a
    // click first. Enter must activate exactly once and close exactly once:
    // gpui fires a focused element's click listener on key up, and the
    // trigger's listener toggles the menu, so refocusing the trigger inside
    // this keystroke would reopen it.
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "Down then Enter must activate the first row exactly once"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the Enter pick must dismiss the menu exactly once -- no reopen"
    );

    let_exit_finish(cx);
    click(cx, 40., 64.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "the menu must be gone: a click where the row was records nothing"
    );
}

#[gpui::test]
fn multiple_mode_stays_open_for_consecutive_picks(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let selections = events();
    let recorded = selections.clone();
    let opens = events();
    let opened = opens.clone();
    // The caller owns the selection set -- v3's `selectedKeys` /
    // `onSelectionChange` loop (the collection seed is deliberately not a
    // component prop in this port). Storing the picks here is what lets the
    // second report contain the first.
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let selections = selections.clone();
        let opens = opens.clone();
        let held = held.clone();
        // Read the set out of the guard first, or the borrow outlives this
        // statement and collides with the callback's write.
        let selected_now = held.borrow().clone();
        Dropdown::uncontrolled(
            Button::new("ddm-trigger").label("Fruits"),
            vec![
                MenuItem::new("apple", "Apple"),
                MenuItem::new("banana", "Banana"),
                MenuItem::new("cherry", "Cherry"),
            ],
        )
        .id("dd-multi")
        .selection_mode(SelectionMode::Multiple)
        .selected_keys(selected_now)
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_selection_change(move |keys, window, _cx| {
            *held.borrow_mut() = keys.to_vec();
            let joined = keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            selections.borrow_mut().push(joined);
            // Re-render with the stored selection, or the next pick would
            // compute from an empty set.
            window.refresh();
        })
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // A multiple-mode pick leaves the menu open, so the next row is still
    // there to click without reopening anything.
    click(cx, 40., 64.);
    click(cx, 40., 102.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["apple", "banana"],
        "both rows must be actioned"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["apple", "apple,banana"],
        "the second report must still contain the first pick"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the menu must never have closed between the two picks"
    );
}

#[gpui::test]
fn submenu_trigger_leaves_parent_open(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            Button::new("dds-trigger").label("Share"),
            vec![
                MenuItem::new("share", "Other").submenu(vec![
                    MenuItem::new("sms", "SMS"),
                    MenuItem::new("airdrop", "AirDrop"),
                ]),
                MenuItem::new("copy", "Copy link"),
            ],
        )
        .id("dd-sub")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The submenu trigger row opens a child panel; it must not end the parent
    // menu. Proving that by behaviour: the plain row below it is still there
    // to click after the submenu row.
    click(cx, 40., 64.);
    assert_eq!(fired.borrow().as_slice(), ["share"]);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a submenu trigger must not dismiss the parent menu"
    );

    click(cx, 40., 102.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["share", "copy"],
        "the row below the submenu trigger must still answer: the parent \
         menu is open"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the close only comes from the plain row"
    );
}
