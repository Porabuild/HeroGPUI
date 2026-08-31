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
use herogpui_components::{ComboBox, Input, InputState, MenuTrigger, PickerItem};

use harness::{click, events, open_host, press};

/// Items whose labels are unique, so the key can be the label itself.
fn keyed(labels: &[&str]) -> Vec<PickerItem> {
    labels
        .iter()
        .map(|l| PickerItem::new(l.to_string(), l.to_string()))
        .collect()
}

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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
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
                ComboBox::new(combo_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
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

/// PageUp/PageDown belong to the open suggestion list: pinned React Aria
/// 3.51.0 binds them through `useSelectableCollection`, which a closed
/// ComboBox never runs. The proof is not vacuous: the field is focused but
/// closed under the `MenuTrigger::Input` trigger, so the page keys land on
/// the very handler that answers them on an open list — and a following Down
/// still opens. A page key on the closed field must not open the list and
/// must not commit anything.
#[gpui::test]
fn combo_box_page_keys_ignore_a_closed_field(cx: &mut TestAppContext) {
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
            .menu_trigger(MenuTrigger::Input)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // Clicking into the field focuses it without opening: the Input trigger
    // opens on the first edit, so the presses land on a live handler.
    click(cx, 60., 18.);
    assert!(
        opened.borrow().is_empty(),
        "the probe must begin from a focused, closed field"
    );

    press(cx, "pagedown");
    press(cx, "pageup");
    assert!(
        opened.borrow().is_empty(),
        "a page key on the closed field must not open the list"
    );

    // The presses were delivered: the same focused handler still opens on
    // Down, which also seats the cursor on the first row.
    press(cx, "down");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the page keys must have reached a live handler, not a dead one"
    );

    // Escape closes the list but leaves the cursor seated on "Typst" -- so
    // the page keys now reach the handler *with* a cursor, exactly the state
    // an unconditional end mapping would move (and the Move::To path would
    // even reopen the list with).
    press(cx, "escape");
    press(cx, "pagedown");
    press(cx, "pageup");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "a page key on the closed field must not reopen the list: the only \
         reports are the open and the Escape close"
    );
    assert!(
        recorded.borrow().is_empty(),
        "a page key on the closed field must not commit anything"
    );

    // Down then reopens, walks the retained cursor to the second row, and
    // Enter commits it -- the page keys left the cursor where they found it.
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Rust"],
        "the page keys must have left the closed field answering as before"
    );
}

/// Pinned HeroUI v3.2.4 scrolls the *popover*, with the ListBox element
/// itself `overflow-clip`, so pinned React Aria 3.51.0 never sees a
/// scrollable list behind a ComboBox: with a cursor, page keys take the
/// enabled ends whatever the list's length. Those handlers require a focused
/// key, though — a chevron-opened, selection-less ComboBox has a null cursor,
/// so its page keys are inert; a Down establishes the cursor, and paging from
/// there reaches the first and last enabled rows, skipping the disabled rows
/// at both ends. Paging only moves the highlight until Enter commits it.
#[gpui::test]
fn combo_box_page_keys_reach_enabled_ends_after_a_cursor_exists(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let options: Vec<PickerItem> = (0..20)
        .map(|i| {
            let label = format!("Option {i:02}");
            PickerItem::new(label.clone(), label)
        })
        .collect();
    let state = combo_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, options.clone())
            .max_items(20)
            .menu_trigger(MenuTrigger::Input)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .disabled_keys(["Option 00".into(), "Option 19".into()])
            .into_any_element()
    });

    // Chevron-open with no selection: the cursor is null, so both page keys
    // must be inert. Down from a null cursor enters the first enabled row
    // (Option 01); had either page key created a cursor, Down would hold on
    // Option 18 or step to Option 02, and the commit would betray the
    // unconditional cursor creation.
    click(cx, 298., 18.);
    press(cx, "pagedown");
    press(cx, "pageup");
    assert!(
        recorded.borrow().is_empty(),
        "page keys on a chevron-opened, cursor-less list must commit nothing"
    );
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Option 01"],
        "page keys on a cursor-less list must be inert: Down must still \
         enter the first enabled row"
    );

    // Enter closed the list; the chevron reopens it, two Downs establish the
    // cursor on Option 02, and PageDown must take the last enabled row
    // (Option 18), never the disabled Option 19.
    click(cx, 298., 18.);
    press(cx, "down down");
    press(cx, "pagedown");
    assert!(
        recorded.borrow().as_slice() == ["Option 01"],
        "PageDown must only move the highlight, never commit by itself"
    );
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Option 01", "Option 18"],
        "PageDown with a cursor must reach the last enabled row"
    );

    // Reopen once more; the cursor starts over on the first enabled row and
    // PageUp must walk back to it — which is also the first enabled row, so
    // take two Downs first — never the disabled Option 00.
    click(cx, 298., 18.);
    press(cx, "down down");
    press(cx, "pageup");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Option 01", "Option 18", "Option 01"],
        "PageUp with a cursor must reach the first enabled row"
    );
}
