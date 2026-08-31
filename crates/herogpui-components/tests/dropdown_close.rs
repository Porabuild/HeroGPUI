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
//! is (40, 18); the menu panel hangs 6px below it and ordinary rows begin at
//! y=46. Submenu geometry tests deliberately use measured custom rows instead
//! of extending those ordinary-row coordinates to content with another size.
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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    canvas, point, prelude::*, px, AnyElement, ElementId, Modifiers, MouseButton, SharedString,
    TestAppContext, VisualTestContext,
};
use herogpui_components::{Button, Dropdown, Menu, MenuItem, SelectionMode};

use harness::{click, events, open_host, press};

/// Forces `times` full host re-renders with no input event between them,
/// standing in for the repaints an open menu plays through while mounted.
fn redraw_frames(cx: &mut VisualTestContext, times: usize) {
    for _ in 0..times {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

/// Moves the test clock past the Dropdown's 100ms exit phase, so a
/// closed-proof click cannot land on the exiting panel.
fn let_exit_finish(cx: &mut TestAppContext) {
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(150));
}

/// Reads the Menu's keyed submenu slot from the two `RenderOnce` component
/// wrappers that namespace it. The zero-size canvas runs during prepaint,
/// where `use_keyed_state` is legal.
fn submenu_open_probe(dropdown_id: &'static str, seen: harness::Events) -> AnyElement {
    canvas(
        move |_, window, cx| {
            let wrap_base = format!("{:?}", ElementId::Name(dropdown_id.into()));
            let menu_id = ElementId::Name(format!("{wrap_base}-menu").into());
            let menu_base = format!("{menu_id:?}");
            let key = ElementId::Name(format!("{menu_base}-submenu").into());
            let open = window.with_id(std::any::type_name::<Dropdown>(), |window| {
                window.with_id(std::any::type_name::<Menu>(), |window| {
                    window
                        .use_keyed_state(key, cx, |_, _| None::<SharedString>)
                        .read(cx)
                        .is_some()
                })
            });
            seen.borrow_mut().push(format!("open:{open}"));
        },
        |_, _, _, _| {},
    )
    .size_0()
    .into_any_element()
}

fn last(seen: &harness::Events) -> String {
    seen.borrow().last().cloned().unwrap_or_default()
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
            "ddc",
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
            "ddk",
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

    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "keyboard activation must open the menu"
    );
    // Pinned React Aria opens a keyboard-triggered menu with its first enabled
    // row focused, so Enter activates it without an arrow key first.
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "Enter must activate the initially focused row exactly once"
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
    // This case drives v3's controlled `selectedKeys` / `onSelectionChange`
    // loop. Storing the picks here is what lets the second report contain the
    // first; the uncontrolled seed is covered separately.
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
            "ddm",
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
    let selections = events();
    let selected = selections.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let selections = selections.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dds",
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
        .selection_mode(SelectionMode::Single)
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_selection_change(move |keys, _, _| {
            selections.borrow_mut().push(
                keys.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The submenu trigger row opens a child panel; it must not end the parent
    // menu or fire the parent Menu's onAction. Proving that by behaviour: the
    // plain row below it is still there to click after the submenu row.
    click(cx, 40., 64.);
    assert!(
        fired.borrow().is_empty(),
        "a submenu trigger must not fire the parent Menu's onAction"
    );
    assert!(
        selected.borrow().is_empty(),
        "a submenu trigger must not enter the parent Menu's selection"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a submenu trigger must not dismiss the parent menu"
    );

    click(cx, 40., 102.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["copy"],
        "the row below the submenu trigger must still answer: the parent \
         menu is open"
    );
    assert_eq!(selected.borrow().as_slice(), ["copy"]);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the close only comes from the plain row"
    );
}

#[gpui::test]
fn submenu_opens_from_pointer_click_without_parent_callbacks(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let selections = events();
    let selected = selections.clone();
    let submenu = events();
    let opened = submenu.clone();
    let root_opens = events();
    let root_opened = root_opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let selections = selections.clone();
        let root_opens = root_opens.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-click",
                    Button::new("dd-sub-click-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other")
                        .submenu(vec![MenuItem::new("sms", "SMS")])],
                )
                .id("dd-sub-click")
                .selection_mode(SelectionMode::Single)
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
                .on_selection_change(move |keys, _, _| {
                    selections.borrow_mut().push(
                        keys.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                })
                .on_open_change(move |open, _, _| {
                    root_opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(submenu_open_probe("dd-sub-click", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    click(cx, 40., 64.);
    assert!(fired.borrow().is_empty());
    assert!(selected.borrow().is_empty());
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    assert!(fired.borrow().is_empty());
    assert!(selected.borrow().is_empty());

    click(cx, 280., 64.);
    assert_eq!(fired.borrow().as_slice(), ["sms"]);
    assert!(selected.borrow().is_empty());
    assert_eq!(root_opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn submenu_opens_from_keyboard_press_without_parent_callbacks(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let selections = events();
    let selected = selections.clone();
    let submenu = events();
    let opened = submenu.clone();
    let root_opens = events();
    let root_opened = root_opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let selections = selections.clone();
        let root_opens = root_opens.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-key",
                    Button::new("dd-sub-key-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other")
                        .submenu(vec![MenuItem::new("sms", "SMS")])],
                )
                .id("dd-sub-key")
                .selection_mode(SelectionMode::Single)
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
                .on_selection_change(move |keys, _, _| {
                    selections.borrow_mut().push(
                        keys.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                })
                .on_open_change(move |open, _, _| {
                    root_opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(submenu_open_probe("dd-sub-key", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "enter");
    assert!(fired.borrow().is_empty());
    assert!(selected.borrow().is_empty());
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    assert!(fired.borrow().is_empty());
    assert!(selected.borrow().is_empty());

    press(cx, "enter");
    assert_eq!(fired.borrow().as_slice(), ["sms"]);
    assert_eq!(root_opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn submenu_right_opens_focuses_first_item_and_left_returns(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let selections = events();
    let selected = selections.clone();
    let submenu = events();
    let opened = submenu.clone();
    let root_opens = events();
    let root_opened = root_opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let selections = selections.clone();
        let root_opens = root_opens.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-right",
                    Button::new("dd-sub-right-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other").submenu(vec![
                        MenuItem::new("blocked", "Blocked"),
                        MenuItem::new("sms", "SMS"),
                        MenuItem::new("email", "Email"),
                    ])],
                )
                .id("dd-sub-right")
                .selection_mode(SelectionMode::Single)
                .disabled_keys(["blocked"])
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
                .on_selection_change(move |keys, _, _| {
                    selections.borrow_mut().push(
                        keys.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                })
                .on_open_change(move |open, _, _| {
                    root_opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(submenu_open_probe("dd-sub-right", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    assert!(fired.borrow().is_empty());
    assert!(selected.borrow().is_empty());

    // Left closes the child and returns focus to the parent row. Repeating the
    // cycle after moving to Email proves each ArrowRight re-arms the child's
    // focus-first strategy rather than reviving its old cursor.
    press(cx, "down");
    press(cx, "left");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:false");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    press(cx, "left");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:false");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");

    // ArrowRight uses React Aria's `focusStrategy="first"`, so Enter activates
    // the first enabled child and dismisses the root Dropdown.
    press(cx, "enter");
    assert_eq!(fired.borrow().as_slice(), ["sms"]);
    assert!(selected.borrow().is_empty());
    assert_eq!(root_opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn submenu_opens_from_hover(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let submenu = events();
    let opened = submenu.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-hover",
                    Button::new("dd-sub-hover-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other")
                        .submenu(vec![MenuItem::new("sms", "SMS")])],
                )
                .id("dd-sub-hover")
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string())),
            )
            .child(submenu_open_probe("dd-sub-hover", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    cx.simulate_mouse_move(
        point(px(40.), px(64.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    cx.update(|window, _| window.refresh());

    assert_eq!(last(&opened), "open:true");
    press(cx, "enter");
    assert!(fired.borrow().is_empty());
}

#[gpui::test]
fn disabled_submenu_trigger_never_opens(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let submenu = events();
    let opened = submenu.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-disabled",
                    Button::new("dd-sub-disabled-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other")
                        .submenu(vec![MenuItem::new("sms", "SMS")])],
                )
                .id("dd-sub-disabled")
                .disabled_keys(["share"])
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string())),
            )
            .child(submenu_open_probe("dd-sub-disabled", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    cx.simulate_mouse_move(
        point(px(40.), px(64.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    cx.update(|window, _| window.refresh());
    click(cx, 40., 64.);
    press(cx, "down");
    press(cx, "enter");
    cx.update(|window, _| window.refresh());

    assert_eq!(last(&opened), "open:false");
    assert!(
        fired.borrow().is_empty(),
        "disabled submenu triggers must not open from any interaction path"
    );
}

#[gpui::test]
fn low_row_tall_submenu_is_hit_testable_beyond_parent_scroll_mask(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let submenu = events();
    let opened = submenu.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let mut items = (0..11)
            .map(|i| MenuItem::new(format!("plain-{i}"), format!("Plain {i}")))
            .collect::<Vec<_>>();
        items.push(
            MenuItem::new("more", "More").submenu(
                (0..8)
                    .map(|i| MenuItem::new(format!("sub-{i}"), format!("Submenu {i}")))
                    .collect(),
            ),
        );
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-low",
                    Button::new("dd-sub-low-trigger").label("Actions"),
                    items,
                )
                .id("dd-sub-low")
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string())),
            )
            .child(submenu_open_probe("dd-sub-low", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    // Eleven 36px rows and their 2px gaps put the submenu trigger's centre at
    // y=482. The eighth child ends far below the parent panel's y=504 bottom.
    click(cx, 40., 482.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    assert!(fired.borrow().is_empty());

    click(cx, 280., 748.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["sub-7"],
        "the last child must remain painted and hit-testable outside the parent scroll mask"
    );
}

#[gpui::test]
fn nested_leaf_enter_dismisses_the_root_dropdown(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dd-sub-nested",
            Button::new("dd-sub-nested-trigger").label("Share"),
            vec![MenuItem::new("share", "Share").submenu(vec![
                MenuItem::new("send", "Send").submenu(vec![MenuItem::new("sms", "SMS")])
            ])],
        )
        .id("dd-sub-nested")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    press(cx, "enter");

    assert_eq!(fired.borrow().as_slice(), ["sms"]);
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn root_dismissal_clears_open_submenu_state(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let submenu = events();
    let submenu_opened = submenu.clone();

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-dismiss",
                    Button::new("dd-sub-dismiss-trigger").label("Share"),
                    vec![MenuItem::new("share", "Share")
                        .submenu(vec![MenuItem::new("sms", "SMS")])],
                )
                .id("dd-sub-dismiss")
                .on_open_change(move |open, _, _| {
                    opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(submenu_open_probe("dd-sub-dismiss", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&submenu_opened), "open:true");

    press(cx, "escape");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&submenu_opened), "open:false");
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    let_exit_finish(cx);
    click(cx, 40., 18.);
    click(cx, 40., 64.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&submenu_opened), "open:true");
    click(cx, 700., 700.);
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&submenu_opened), "open:false");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true", "open:false"]
    );
}

#[gpui::test]
fn sibling_submenus_keep_independent_keyboard_cursors(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        Dropdown::uncontrolled(
            "dd-sub-siblings",
            Button::new("dd-sub-siblings-trigger").label("Actions"),
            vec![
                MenuItem::new("first", "First").submenu(
                    (0..5)
                        .map(|i| MenuItem::new(format!("first-{i}"), format!("First {i}")))
                        .collect(),
                ),
                MenuItem::new("second", "Second")
                    .submenu(vec![MenuItem::new("second-a", "Second A")]),
            ],
        )
        .id("dd-sub-siblings")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    press(cx, "down");
    press(cx, "down");
    press(cx, "down");
    press(cx, "left");
    cx.update(|window, _| window.refresh());
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    press(cx, "enter");

    assert_eq!(
        fired.borrow().as_slice(),
        ["second-a"],
        "the long first submenu's cursor must not poison its short sibling"
    );
}

#[gpui::test]
fn keyed_submenu_stays_open_and_keeps_its_cursor_when_parent_rows_reorder(cx: &mut TestAppContext) {
    let reordered = Rc::new(Cell::new(false));
    let reorder_for_render = reordered.clone();
    let actions = events();
    let fired = actions.clone();
    let submenu = events();
    let opened = submenu.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let submenu_item = || {
            MenuItem::new("share", "Share").submenu(vec![
                MenuItem::new("first", "First"),
                MenuItem::new("second", "Second"),
            ])
        };
        let items = if reorder_for_render.get() {
            vec![MenuItem::new("plain", "Plain"), submenu_item()]
        } else {
            vec![submenu_item(), MenuItem::new("plain", "Plain")]
        };
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-sub-reorder",
                    Button::new("dd-sub-reorder-trigger").label("Actions"),
                    items,
                )
                .id("dd-sub-reorder")
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string())),
            )
            .child(submenu_open_probe("dd-sub-reorder", submenu.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    press(cx, "down");

    reordered.set(true);
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true");
    press(cx, "enter");

    assert_eq!(
        fired.borrow().as_slice(),
        ["second"],
        "a keyed reorder must preserve both the open submenu and its child cursor"
    );
}

#[gpui::test]
fn measured_trigger_bounds_anchor_custom_tall_and_wide_rows(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        Menu::new(
            "dd-measured-menu",
            vec![
                MenuItem::new("tall", "Tall").description(
                    "A described row whose custom content is much taller than default",
                ),
                MenuItem::new("more", "More")
                    .submenu(vec![MenuItem::new("child", "A submenu child")]),
            ],
        )
        .id("dd-sub-measured")
        .item_content(|key, _| {
            if key.as_ref() == "tall" {
                gpui::div()
                    .w(px(420.))
                    .h(px(90.))
                    .child("Tall custom content")
                    .into_any_element()
            } else {
                gpui::div()
                    .w(px(420.))
                    .child("A deliberately long submenu trigger label")
                    .into_any_element()
            }
        })
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // The custom first row makes the trigger begin around y=108, while the
    // 420px content makes the child begin beyond x=440. These points miss the
    // old 36/43px row estimate and fixed 220px panel.
    click(cx, 40., 126.);
    cx.update(|window, _| window.refresh());
    click(cx, 500., 130.);

    assert_eq!(fired.borrow().as_slice(), ["child"]);
}

#[gpui::test]
fn blank_quadrant_between_panels_is_outside_the_dropdown(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dd-sub-blank",
            Button::new("dd-sub-blank-trigger").label("Actions"),
            std::iter::once(
                MenuItem::new("more", "More").submenu(vec![MenuItem::new("child", "Child")]),
            )
            .chain((1..8).map(|i| MenuItem::new(format!("plain-{i}"), format!("Plain {i}"))))
            .collect(),
        )
        .id("dd-sub-blank")
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });

    click(cx, 40., 18.);
    click(cx, 40., 64.);
    cx.update(|window, _| window.refresh());
    // x=280 is in the child column and y=300 is in the tall parent's range,
    // but the short child ends near y=84. The flex hull contains this point;
    // neither real panel does.
    click(cx, 280., 300.);

    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

/// PageUp/PageDown belong to the *open* menu: pinned `useSelectableCollection`
/// binds them only while the collection is mounted, which a closed Dropdown
/// never is. The proof is not vacuous: the trigger is focused, so a
/// following Enter still opens and the menu's keyboard cursor still answers.
#[gpui::test]
fn dropdown_page_keys_ignore_a_closed_trigger(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dd-page-closed",
            Button::new("dd-page-closed-trigger").label("Actions"),
            vec![MenuItem::new("one", "One"), MenuItem::new("two", "Two")],
        )
        .id("dd-page-closed")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pagedown");
    press(cx, "pageup");
    assert!(
        opened.borrow().is_empty(),
        "a page key on the closed trigger must not open the menu"
    );
    assert!(
        fired.borrow().is_empty(),
        "a page key on the closed trigger must not fire any action"
    );

    // The presses were delivered: the same focused trigger still opens on
    // Enter, and the menu then picks as before the page keys arrived.
    press(cx, "enter");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the page keys must have reached a live trigger, not a dead one"
    );
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "the page keys must have left the menu answering as before"
    );
}

/// The menu's page handlers require `manager.focusedKey != null`: a
/// mouse-opened menu has a null cursor until an arrow seats one, so both
/// page keys must be inert. Down from a null cursor enters the first row;
/// had either page key created a cursor, Down would hold on the last row or
/// step off the first, and the pick would betray the unconditional cursor
/// creation.
#[gpui::test]
fn dropdown_page_keys_stay_inert_until_a_cursor_is_seated(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dd-page-cursorless",
            Button::new("dd-page-cursorless-trigger").label("Actions"),
            vec![MenuItem::new("one", "One"), MenuItem::new("two", "Two")],
        )
        .id("dd-page-cursorless")
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "pagedown");
    press(cx, "pageup");
    assert!(
        fired.borrow().is_empty(),
        "page keys on a mouse-opened, cursor-less menu must fire nothing"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "page keys on a mouse-opened, cursor-less menu must not dismiss it"
    );
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["one"],
        "page keys on a cursor-less menu must be inert: Down must still \
         seat the cursor on the first row"
    );
}

/// Pinned HeroUI v3.2.4 puts the overflow scrolling on the Popover while the
/// Menu element is `overflow-clip`, so pinned React Aria 3.51.0 treats the
/// menu as non-scrollable: with a cursor, page keys take the enabled ends
/// whatever the menu's length. A keyboard open seats the cursor on the first
/// enabled row, so PageDown must reach the last enabled row -- never the
/// disabled tail -- and PageUp must walk back to the first enabled row --
/// never the disabled head. Paging only moves the highlight until Enter
/// activates it.
#[gpui::test]
fn dropdown_page_keys_reach_enabled_ends_after_a_cursor_exists(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let items = (0..20)
            .map(|i| MenuItem::new(format!("option-{i:02}"), format!("Option {i:02}")))
            .collect::<Vec<_>>();
        Dropdown::uncontrolled(
            "dd-page-ends",
            Button::new("dd-page-ends-trigger").label("Actions"),
            items,
        )
        .id("dd-page-ends")
        .disabled_keys(["option-00", "option-19"])
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // The keyboard open focuses the first *enabled* row (Option 01). PageDown
    // must take the last enabled row (Option 18), skipping the disabled
    // Option 19, and PageUp must walk straight back to Option 01 -- and
    // neither page key may activate by itself.
    press(cx, "tab");
    press(cx, "enter");
    press(cx, "pagedown");
    assert!(
        fired.borrow().is_empty(),
        "PageDown must only move the highlight, never activate by itself"
    );
    press(cx, "pageup");
    assert!(
        fired.borrow().is_empty(),
        "PageUp must only move the highlight, never activate by itself"
    );
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["option-01"],
        "PageDown then PageUp must leave the highlight on the first enabled row"
    );

    // A pointer open seats no cursor: Down from a null cursor enters the
    // first enabled row and a second Down steps to Option 02, and PageDown
    // must take the last enabled row (Option 18) -- the pick betrays any
    // shorter step.
    let_exit_finish(cx);
    click(cx, 40., 18.);
    press(cx, "down down");
    press(cx, "pagedown");
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["option-01", "option-18"],
        "PageDown with a cursor must reach the last enabled row"
    );
}

/// With the submenu open, the page keys answer the child's own collection --
/// its cursor pages to the child's enabled end -- and the parent menu's
/// state stays untouched: the submenu stays open, the parent highlight does
/// not move, and nothing activates until Enter.
#[gpui::test]
fn dropdown_page_keys_page_the_open_submenu_without_disturbing_the_parent(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let submenu = events();
    let opened = submenu.clone();
    let probe = submenu;

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        gpui::div()
            .child(
                Dropdown::uncontrolled(
                    "dd-page-sub",
                    Button::new("dd-page-sub-trigger").label("Share"),
                    vec![MenuItem::new("share", "Other").submenu(vec![
                        MenuItem::new("blocked", "Blocked"),
                        MenuItem::new("sms", "SMS"),
                        MenuItem::new("email", "Email"),
                    ])],
                )
                .id("dd-page-sub")
                .disabled_keys(["blocked"])
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string())),
            )
            .child(submenu_open_probe("dd-page-sub", probe.clone()))
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
    cx.update(|window, _| window.refresh());
    assert_eq!(last(&opened), "open:true", "the submenu must be open");
    assert!(
        fired.borrow().is_empty(),
        "the probe must begin from a submenu with no activation"
    );

    // The child's focus-first cursor sits on SMS; PageDown must page it to
    // the child's last enabled row (Email, skipping the disabled Blocked)
    // while the submenu stays open and nothing activates.
    press(cx, "pagedown");
    cx.update(|window, _| window.refresh());
    assert_eq!(
        last(&opened),
        "open:true",
        "paging the child must not disturb the open submenu"
    );
    assert!(
        fired.borrow().is_empty(),
        "paging the child must not activate anything"
    );

    // Enter activates the paged child row and dismisses the root Dropdown.
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["email"],
        "PageDown in the submenu must reach the child's last enabled row"
    );
}

/// A shared dropdown+submenu host for the union-contract tests below: the
/// keyboard path opens the parent and its child, so no row coordinate is
/// needed to open either panel. The parent panel is 220px wide starting at
/// x=0, so the child panel hangs from x≈224.
fn union_host<'a>(
    cx: &'a mut TestAppContext,
    dropdown_id: &'static str,
) -> (&'a mut VisualTestContext, harness::Events, harness::Events) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        Dropdown::uncontrolled(
            "dd-union-trigger",
            Button::new("dd-union-trigger").label("Share"),
            vec![MenuItem::new("share", "Share").submenu(vec![
                MenuItem::new("sms", "SMS"),
                MenuItem::new("mail", "Mail"),
            ])],
        )
        .id(dropdown_id)
        .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });
    (cx, fired, opened)
}

/// Opens parent + child by keyboard and leaves them on screen.
fn open_union(cx: &mut VisualTestContext) {
    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");
}

/// After idle repaints with the composite still mounted, a press on the child
/// row still reaches it and dismisses exactly once. An unchanged location
/// would still hit even a stale copy of the union; the moved-composite test
/// owns that the union tracks the live panels.
#[gpui::test]
fn child_row_stays_pressable_across_repeated_frames(cx: &mut TestAppContext) {
    let (cx, fired, opened) = union_host(cx, "dd-union-frames");
    open_union(cx);
    redraw_frames(cx, 4);

    // The child panel's first row: past the parent panel's 220px width and
    // its 4px gap, in the child's own padding box.
    click(cx, 280., 72.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["sms"],
        "after repeated frames the child row must still answer its press"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the pick must dismiss the menu exactly once"
    );
}

/// Once the child has closed the union is no longer consulted -- the panel
/// falls back to its unconditional token dismissal -- so the spot the child
/// occupied must simply dismiss the menu like any other blank page.
#[gpui::test]
fn panel_union_drops_bounds_that_left_the_composite(cx: &mut TestAppContext) {
    let (cx, _fired, opened) = union_host(cx, "dd-union-closed");
    open_union(cx);
    redraw_frames(cx, 2);

    press(cx, "left");
    redraw_frames(cx, 3);

    click(cx, 280., 72.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the press where the submenu used to be must dismiss the menu"
    );
}

/// While the child is open the union is the only thing standing between an
/// outside press and the row underneath, so the union must hold exactly the
/// moving composite: shift the whole dropdown down between frames and the
/// pre-move child bounds must not answer a press where they used to cover,
/// while the moved child's own rows must.
#[gpui::test]
fn panel_union_tracks_a_moved_composite_between_frames(cx: &mut TestAppContext) {
    let shift: Rc<Cell<f32>> = Rc::new(Cell::new(0.));
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let content_shift = shift.clone();
    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let opens = opens.clone();
        gpui::div()
            .mt(px(content_shift.get()))
            .child(
                Dropdown::uncontrolled(
                    "dd-union-move-trigger",
                    Button::new("dd-union-move-trigger").label("Share"),
                    vec![MenuItem::new("share", "Share").submenu(vec![
                        MenuItem::new("sms", "SMS"),
                        MenuItem::new("mail", "Mail"),
                    ])],
                )
                .id("dd-union-move")
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
                .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}"))),
            )
            .into_any_element()
    });

    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "right");

    shift.set(80.);
    redraw_frames(cx, 3);

    // The child panel's old first-row spot is now blank page above the moved
    // composite; a union still holding the pre-move bounds would decline this
    // press and keep the menu open. A stale child hit target would fire the
    // row instead of dismissing.
    click(cx, 280., 72.);
    assert!(
        fired.borrow().is_empty(),
        "the press at the pre-move child location must not fire a row"
    );
    assert_eq!(
        last(&opened),
        "open:false",
        "the press at the pre-move child location must dismiss the menu"
    );

    // Reopened from the moved trigger, the child's rows must answer 80px
    // lower.
    let_exit_finish(cx);
    click(cx, 40., 98.);
    assert_eq!(last(&opened), "open:true", "the moved trigger must reopen");
    press(cx, "down");
    press(cx, "right");
    redraw_frames(cx, 1);
    click(cx, 280., 152.);
    assert_eq!(
        fired.borrow().as_slice(),
        ["sms"],
        "the moved child's first row must be inside the union"
    );
    assert_eq!(last(&opened), "open:false", "the pick must dismiss");
}
