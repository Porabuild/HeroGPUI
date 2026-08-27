//! Behaviour tests for the three search-and-select components: Autocomplete,
//! ComboBox and Dropdown, plus the DatePicker's calendar grid.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests drive the controls and assert on recorded callbacks and behavioural
//! probes only — never on appearance.
//!
//! Geometry is derived from the components' own constants, not guessed:
//!
//! - Picker trigger fields are 36px rows (`util::FIELD_HEIGHT`). Select,
//!   ComboBox and Autocomplete open from the field at (60, 18); DatePicker
//!   composes an editable DateField plus a separate 24px trigger whose centre
//!   is (124, 18).
//! - The pickers' panels hang from `placed_field_panel(BottomStart, 6px)`:
//!   top = trigger bottom + 6 = 42.
//! - Autocomplete: panel `pt(8)` + search wrapper `py(4)` + 36px field + list
//!   `p(6)` puts row *i* at y 100+36i, plus up to 6px of entry-zoom padding
//!   (`ZoomBox::panel(px(6))`); clicking y = 124+36i lands inside every phase
//!   of that animation.
//! - ComboBox: panel `p(4)` puts row *i* at y 46+36i with ≤4px of zoom
//!   padding; clicking y = 64+36i covers it.
//! - Dropdown: same panel shape as Select (row centres y 64+36i), but both
//!   menus here are driven by keyboard, so only their triggers are clicked.
//! - DatePicker: the cell band starts at 42 + 12 (`picker_panel` padding) +
//!   24 (nav header) + 8 + 8 (calendar gaps) + one text line of weekday
//!   header, and cells are 36px tall; the column centres come from
//!   `CALENDAR_WIDTH` minus six 2px gaps over seven columns. Only the weekday
//!   line height is a text metric rather than a constant, which the chosen y
//!   tolerates by ±14px either way.
//!
//! Reduce motion is deliberately **not** set for this process. Only the
//! Dropdown plays its exit through `util::overlay_phase`, and no test here
//! probes a dropdown after dismissing it; the Autocomplete, ComboBox and
//! DatePicker panels leave the tree outright when closed (`show_panel` /
//! `if is_open` gate them, no exit phase), so "is it closed" probes cannot hit
//! an exiting panel.

mod harness;

use std::{cell::RefCell, rc::Rc};

use gpui::{prelude::*, px, TestAppContext};
use herogpui_components::{
    calendar::{CalendarState, Date, CALENDAR_WIDTH},
    Autocomplete, Button, ComboBox, DateConstraints, DatePicker, Dropdown, InputState, MenuItem,
    SelectionMode,
};

use harness::{click, events, open_host, press};

/// An `InputState` entity for the search-field-backed controls, created before
/// the host opens so the test can keep its own handle to it.
fn search_state(cx: &mut TestAppContext) -> gpui::Entity<InputState> {
    cx.new(|cx| InputState::new(cx))
}

#[gpui::test]
fn autocomplete_opens_filters_and_selects(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        // v3's Autocomplete is a trigger whose popover holds a SearchField;
        // typing filters, clicking a row selects.
        Autocomplete::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the trigger must open"
    );

    // The popover autofocuses its search field, so typing goes straight into
    // it and filters the rows down to the one match.
    cx.simulate_input("ty");
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "clicking the matching row must record its text"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // Closed proof by behaviour: the same spot is bare page below the trigger
    // now, so the press must reach nothing. Were the popover still open, the
    // row would record a second "Typst" here.
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "the popover must be closed after choosing an item"
    );
}

#[gpui::test]
fn autocomplete_uncontrolled_selection_sticks(cx: &mut TestAppContext) {
    // The regression behind these tests: an Autocomplete with neither `value`
    // nor `defaultValue` used to hand its clicks back to a set nobody owned —
    // the callbacks fired, but nothing was remembered. The proof here never
    // reads private state: `on_selection_change_all` reports the component's
    // own held set, so the second report naming BOTH picks can only come from
    // state that stuck, and the third click *toggling Alpha back off* is only
    // reachable if the row knew it was already selected.
    let single = events();
    let picked = single.clone();
    let all = events();
    let reported = all.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let single = single.clone();
        let all = all.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
            .selection_mode(SelectionMode::Multiple)
            .on_change(move |item, _, _| single.borrow_mut().push(item.to_string()))
            .on_selection_change_all(move |keys, _, _| {
                let joined = keys
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                all.borrow_mut().push(joined);
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // Multiple mode keeps the panel open between picks: row i sits at
    // y = 124 + 36i.
    click(cx, 60., 124.);
    assert_eq!(reported.borrow().as_slice(), ["Alpha"]);

    click(cx, 60., 196.);
    assert_eq!(
        reported.borrow().as_slice(),
        ["Alpha", "Alpha,Gamma"],
        "the second report must still contain the first pick"
    );

    click(cx, 60., 124.);
    assert_eq!(
        reported.borrow().as_slice(),
        ["Alpha", "Alpha,Gamma", "Gamma"],
        "re-clicking a picked row must toggle it off, which requires \
         remembering it"
    );
    assert_eq!(picked.borrow().as_slice(), ["Alpha", "Gamma", "Alpha"]);
}

#[gpui::test]
fn autocomplete_escape_closes(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Escape reaches the root handler through the focused search field; it
    // closes the popover and refocuses the trigger without selecting.
    press(cx, "escape");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "escape closes the popover"
    );
    assert!(recorded.borrow().is_empty(), "escape must not select");

    // Closed proof: where row one would have been, nothing answers.
    click(cx, 60., 124.);
    assert!(
        recorded.borrow().is_empty() && opened.borrow().len() == 2,
        "the popover must be gone after escape"
    );
}

#[gpui::test]
fn autocomplete_arrows_and_enter_select(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust".into(), "Go".into(), "Python".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Two Downs put the keyboard cursor on the second row; Enter takes it.
    // Enter must not ALSO reopen: the component deliberately does not refocus
    // the trigger inside this keystroke, because gpui activates a focused
    // element on Enter and the trigger's click listener would fire.
    press(cx, "down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Go"],
        "the second row must be selected exactly once"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn autocomplete_forward_typing_focuses_the_first_match_for_enter(cx: &mut TestAppContext) {
    // react-aria 3.51.0's `useAutocomplete.onChange` treats a forward
    // insertion as a request to focus the wrapped collection's first item, so
    // a query can be typed into the search field and Enter commits the top
    // match without an arrow key first. The focus target is
    // `ListKeyboardDelegate.getFirstKey`, which operates on the *filtered*
    // collection: "Alpha" is index 0 of the unfiltered list and must not
    // catch the autofocus — "ru" filters to Rust and Rusty, and the cursor
    // lands on Rust.
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Alpha".into(), "Rust".into(), "Rusty".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Typing forward must move the keyboard cursor onto the first filtered
    // row, so a bare Enter commits it and closes the popover exactly once.
    cx.simulate_input("ru");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Rust"],
        "typing forward must focus the first filtered match so Enter commits it"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the Enter pick must close the popover exactly once"
    );
}

#[gpui::test]
fn autocomplete_forward_typing_focuses_the_first_enabled_match(cx: &mut TestAppContext) {
    // `ListKeyboardDelegate.getFirstKey` is `findNextNonDisabled` — the first
    // *enabled* item, with `disabledKeys` walked past. A literal first match
    // would put the cursor on the disabled "Rust", and Enter would commit a
    // row that cannot be chosen; the autofocus must land on "Rusty" instead.
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust".into(), "Rusty".into(), "Go".into()])
            .disabled_keys(["Rust".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    cx.simulate_input("ru");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Rusty"],
        "a disabled first match must not catch the forward-typing autofocus"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the Enter pick must close the popover exactly once"
    );
}

#[gpui::test]
fn autocomplete_space_resets_focus_to_the_first_match(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust book".into(), "Rust belt".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("rust");
    press(cx, "down");
    press(cx, "space");
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "rust "
    );
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Rust book"],
        "a typed space is a forward insertion and must refocus the first match"
    );
}

#[gpui::test]
fn autocomplete_backspace_clears_virtual_focus(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust".into(), "Rusty".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("ru");
    press(cx, "down");
    press(cx, "backspace");
    press(cx, "enter");
    assert!(
        recorded.borrow().is_empty(),
        "deleting text must clear virtual focus instead of committing a stale row"
    );
}

#[gpui::test]
fn autocomplete_paste_clears_virtual_focus(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust".into(), "Rusty".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("r");
    press(cx, "down");
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("u".to_owned()));
    press(cx, "ctrl-v");
    press(cx, "enter");
    assert!(
        recorded.borrow().is_empty(),
        "pasting text must clear virtual focus instead of committing a stale row"
    );
}

#[gpui::test]
fn autocomplete_modified_space_does_not_activate_a_match(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Rust".into(), "Rusty".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("ru");
    press(cx, "ctrl-space");
    assert!(
        recorded.borrow().is_empty(),
        "modified Space must not activate the virtually focused row"
    );
}

#[gpui::test]
fn autocomplete_controlled_query_update_does_not_focus_a_match(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let query = Rc::new(RefCell::new(String::new()));
    let query_for_view = query.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, vec!["Alpha".into(), "Rust".into(), "Rusty".into()])
            .default_open(true)
            .input_value(query_for_view.borrow().clone())
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    // React Aria focuses the first match from its input-event handler. An
    // owner replacing controlled `inputValue` is not such an event, so a bare
    // Enter after this repaint must still have no collection cursor to commit.
    *query.borrow_mut() = "ru".to_owned();
    cx.update(|window, _| window.refresh());
    press(cx, "enter");
    assert!(
        recorded.borrow().is_empty(),
        "a controlled query prop update must not masquerade as forward typing"
    );
}

/// Pinned React Stately 3.49.0 backs `inputValue` with controlled state: an
/// edit reports the proposed text, but the visible input keeps the owner's
/// value until that owner accepts it. The bound `InputState` is the visible
/// search field in this port, so it must return to the prop after the edit.
#[gpui::test]
fn autocomplete_controlled_query_waits_for_owner_acceptance(cx: &mut TestAppContext) {
    let inputs = events();
    let recorded = inputs.clone();
    let state = search_state(cx);
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let inputs = inputs.clone();
        Autocomplete::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Rust".into(), "Rusty".into()],
        )
        .default_open(true)
        .input_value("ru")
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .into_any_element()
    });

    cx.simulate_input("s");
    assert_eq!(
        (
            recorded.borrow().clone(),
            state.read_with(cx, |state, _| state.value().to_owned())
        ),
        (vec!["rus".to_owned()], "ru".to_owned()),
        "the edit must be reported without replacing the controlled query"
    );
}

/// `Autocomplete.Indicator` is the chevron in the trigger. HeroUI gives that
/// part the open state through its data attribute; the local closure receives
/// the equivalent boolean so custom content can follow the same state.
#[gpui::test]
fn autocomplete_custom_indicator_receives_trigger_open_state(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(None));
    let recorded = seen.clone();
    let state = search_state(cx);

    let cx = open_host(cx, move || {
        let seen = seen.clone();
        Autocomplete::new(state.clone(), vec!["Alpha".into(), "Beta".into()])
            .indicator(move |is_open| {
                *seen.borrow_mut() = Some(is_open);
                gpui::div().into_any_element()
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(
        *recorded.borrow(),
        Some(true),
        "the custom trigger indicator must observe the open state"
    );
}

/// The composed `ListBox.ItemIndicator` remains a separate row-level seam: it
/// receives selection state rather than the trigger's open state.
#[gpui::test]
fn autocomplete_item_indicator_receives_row_selection_state(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let recorded = seen.clone();
    let state = search_state(cx);

    let _cx = open_host(cx, move || {
        let seen = seen.clone();
        Autocomplete::new(state.clone(), vec!["Alpha".into(), "Beta".into()])
            .default_value(["Alpha"])
            .default_open(true)
            .item_indicator(move |is_selected| {
                seen.borrow_mut().push(is_selected);
                gpui::div().into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        recorded.borrow().as_slice(),
        [true, false],
        "the composed item indicator must receive each row's selection state"
    );
}

/// Exit frames are visual only. Once Escape closes the logical popover, the
/// retained shrinking surface must no longer own outside-dismiss interaction
/// or report the same close a second time.
#[gpui::test]
fn autocomplete_exiting_panel_does_not_repeat_outside_dismissal(cx: &mut TestAppContext) {
    let opens = events();
    let recorded = opens.clone();
    let state = search_state(cx);

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        Autocomplete::new(state.clone(), vec!["Alpha".into(), "Beta".into()])
            .default_open(true)
            .on_open_change(move |is_open, _, _| {
                opens.borrow_mut().push(format!("open:{is_open}"));
            })
            .into_any_element()
    });

    press(cx, "escape");
    click(cx, 600., 300.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "an exiting panel must not retain its dismissal listeners"
    );
}

#[gpui::test]
fn combo_box_custom_enter_preserves_multiple_selection(cx: &mut TestAppContext) {
    let singular = events();
    let singular_for_view = singular.clone();
    let plural = events();
    let plural_for_view = plural.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let values = events();
    let values_for_view = values.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let singular = singular_for_view.clone();
        let plural = plural_for_view.clone();
        let opens = opens_for_view.clone();
        let values = values_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_value(["Alpha", "Beta"])
        .allows_custom_value(true)
        .allows_empty_collection(true)
        .on_change(move |item, _, _| singular.borrow_mut().push(item.to_string()))
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .value_content(move |value| {
            values.borrow_mut().push(value.selected_text.to_owned());
            value.default_children
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down");
    cx.simulate_input("zig");
    press(cx, "enter");
    press(cx, "down enter");
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "zig",
        "custom text must remain in the input"
    );
    assert!(
        singular.borrow().is_empty(),
        "multiple-mode custom text must not report a singular selection"
    );
    assert!(
        plural.borrow().is_empty(),
        "multiple-mode custom text must not report a plural selection change"
    );
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("Alpha, Beta"),
        "custom text must not replace the existing multiple selection"
    );
    assert_eq!(
        opens.borrow().as_slice(),
        ["open:true", "open:false"],
        "custom Enter must close the still-open empty list exactly once"
    );
}

#[gpui::test]
fn combo_box_multiple_enter_toggles_the_focused_item(cx: &mut TestAppContext) {
    let singular = events();
    let singular_for_view = singular.clone();
    let plural = events();
    let plural_for_view = plural.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let inputs = events();
    let inputs_for_view = inputs.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let singular = singular_for_view.clone();
        let plural = plural_for_view.clone();
        let opens = opens_for_view.clone();
        let inputs = inputs_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_value(["Alpha", "Beta"])
        .on_change(move |item, _, _| singular.borrow_mut().push(item.to_string()))
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("a");
    press(cx, "down enter enter");
    cx.simulate_input("g");
    press(cx, "down enter");
    assert!(
        singular.borrow().is_empty(),
        "multiple-mode keyboard picks must not use the singular callback"
    );
    assert_eq!(
        plural.borrow().as_slice(),
        ["Beta", "Alpha,Beta", "Alpha,Beta,Gamma"],
        "the focused key must survive query reset by identity so Enter toggles the same item again"
    );
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "",
        "multiple-mode row activation must reset the input text"
    );
    assert_eq!(
        opens.borrow().as_slice(),
        ["open:true"],
        "multiple-mode row activation must leave the list open"
    );
    assert_eq!(
        inputs.borrow().as_slice(),
        ["a", "", "g", ""],
        "typing and selection-driven query resets must both report input changes"
    );
}

#[gpui::test]
fn combo_box_controlled_multiple_enter_waits_for_the_owner(cx: &mut TestAppContext) {
    let singular = events();
    let singular_for_view = singular.clone();
    let plural = events();
    let plural_for_view = plural.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let singular = singular_for_view.clone();
        let plural = plural_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_keys(["Alpha".into(), "Beta".into()])
        .on_change(move |item, _, _| singular.borrow_mut().push(item.to_string()))
        .on_selection_change_all(move |items, window, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
            // Re-render the unchanged controlled prop, as an owner declining
            // the proposed selection would.
            window.refresh();
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down enter enter");
    assert!(
        singular.borrow().is_empty(),
        "controlled multiple picks must not use the singular callback"
    );
    assert_eq!(
        plural.borrow().as_slice(),
        ["Beta", "Beta"],
        "a controlled ComboBox must keep proposing from the owner's unchanged selection"
    );
}

#[gpui::test]
fn combo_box_multiple_pointer_resets_query_and_stays_open(cx: &mut TestAppContext) {
    let singular = events();
    let singular_for_view = singular.clone();
    let plural = events();
    let plural_for_view = plural.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let inputs = events();
    let inputs_for_view = inputs.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let singular = singular_for_view.clone();
        let plural = plural_for_view.clone();
        let opens = opens_for_view.clone();
        let inputs = inputs_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_value(["Alpha", "Beta"])
        .on_change(move |item, _, _| singular.borrow_mut().push(item.to_string()))
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("g");
    press(cx, "down");
    click(cx, 60., 64.);
    press(cx, "enter");
    click(cx, 60., 64.);
    assert!(
        singular.borrow().is_empty(),
        "multiple-mode pointer picks must not use the singular callback"
    );
    assert_eq!(
        plural.borrow().as_slice(),
        ["Alpha,Beta,Gamma", "Alpha,Beta", "Beta"],
        "the first pick must add filtered Gamma, retained Enter must remove Gamma, then the reset list must let the same point remove Alpha"
    );
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "",
        "multiple-mode pointer selection must reset the input query"
    );
    assert_eq!(
        opens.borrow().as_slice(),
        ["open:true"],
        "multiple-mode pointer selection must leave the list open"
    );
    assert_eq!(
        inputs.borrow().as_slice(),
        ["g", ""],
        "the pointer-driven query reset must report the cleared input exactly once"
    );
}

#[gpui::test]
fn combo_box_multiple_focus_survives_a_capped_collection_reset(cx: &mut TestAppContext) {
    let plural = events();
    let plural_for_view = plural.clone();
    let inputs = events();
    let inputs_for_view = inputs.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let plural = plural_for_view.clone();
        let inputs = inputs_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
                "Delta".into(),
                "Epsilon".into(),
                "Zeta".into(),
                "Eta".into(),
                "Theta".into(),
                "Needle".into(),
            ],
        )
        .selection_mode(SelectionMode::Multiple)
        .max_items(2)
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("needle");
    press(cx, "down enter enter");
    assert_eq!(
        plural.borrow().as_slice(),
        ["Needle", ""],
        "the focused key must survive even when the expanded cap does not render that item"
    );
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "",
        "the first selection must still clear the filtered query"
    );
    assert_eq!(
        inputs.borrow().as_slice(),
        ["n", "ne", "nee", "need", "needl", "needle", ""]
    );
}

#[gpui::test]
fn combo_box_multiple_external_query_drops_a_stale_focused_key(cx: &mut TestAppContext) {
    let plural = events();
    let plural_for_view = plural.clone();
    let inputs = events();
    let inputs_for_view = inputs.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_update = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let plural = plural_for_view.clone();
        let inputs = inputs_for_view.clone();
        let opens = opens_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down");
    cx.update(|window, cx| {
        state_for_update.update(cx, |state, cx| {
            state.set_value("g");
            cx.notify();
        });
        window.refresh();
    });
    press(cx, "enter");
    assert!(
        plural.borrow().is_empty(),
        "a caller-driven query must not let Enter select the stale key: {:?}",
        plural.borrow().as_slice()
    );
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "",
        "Enter without a live focused key must reset the input"
    );
    assert_eq!(inputs.borrow().as_slice(), [""]);
    assert_eq!(opens.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn combo_box_single_enter_without_focus_restores_the_selected_label(cx: &mut TestAppContext) {
    let selections = events();
    let selections_for_view = selections.clone();
    let inputs = events();
    let inputs_for_view = inputs.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let state = search_state(cx);
    let state_for_assert = state.clone();
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections_for_view.clone();
        let inputs = inputs_for_view.clone();
        let opens = opens_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .default_value(["Beta"])
        .on_change(move |item, _, _| selections.borrow_mut().push(item.to_string()))
        .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("g");
    press(cx, "enter");
    assert!(
        selections.borrow().is_empty(),
        "Enter without a focused row must not report a new selection"
    );
    assert_eq!(
        cx.update(|_, cx| state_for_assert.read(cx).value().to_owned()),
        "Beta",
        "single mode must restore the selected item's label"
    );
    assert_eq!(inputs.borrow().as_slice(), ["g", "Beta"]);
    assert_eq!(opens.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn combo_box_multiple_show_all_commits_a_row_outside_the_query(cx: &mut TestAppContext) {
    let plural = events();
    let plural_for_view = plural.clone();
    let opens = events();
    let opens_for_view = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let plural = plural_for_view.clone();
        let opens = opens_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .on_open_change(move |open, _, _| opens.borrow_mut().push(format!("open:{open}")))
        .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("g");
    click(cx, 298., 18.);
    click(cx, 298., 18.);
    press(cx, "down enter");
    assert_eq!(
        plural.borrow().as_slice(),
        ["Alpha"],
        "show-all navigation must commit the focused visible row even when it does not match the query"
    );
    assert_eq!(
        opens.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "multiple selection must leave the explicitly reopened full list open"
    );
}

#[gpui::test]
fn combo_box_duplicate_labels_keep_distinct_keyboard_stops(cx: &mut TestAppContext) {
    let plural = events();
    let plural_for_view = plural.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let plural = plural_for_view.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Same".into(), "Same".into(), "Other".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .on_selection_change_all(move |items, _, _| {
            plural.borrow_mut().push(
                items
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    press(cx, "down down down enter");
    assert_eq!(
        plural.borrow().as_slice(),
        ["Other"],
        "the second equal label must remain a real stop before navigation reaches the third row"
    );
}

#[gpui::test]
fn combo_box_typing_filters_and_click_selects(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        // ComboBox is input-shaped: the query is typed into the field itself.
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .placeholder("Search")
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // The chevron button sits at the right end of the 320px-wide field:
    // 320 - 12px padding - half its 20px box. The chevron is used here to
    // open deliberately without focusing the input first. The default focus
    // trigger is covered in combo_box_open.rs; this test's point is the
    // chevron and pick geometry.
    click(cx, 298., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the chevron must open the list"
    );

    // Typing into the field filters the suggestion rows down to one.
    cx.simulate_input("ty");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "ty",
        "the chevron press must leave the ComboBox input focused for typing"
    );
    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "clicking the matching suggestion must select it"
    );

    // Taking a suggestion fills the field and closes the list — read back
    // through the state entity the test itself owns.
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "Typst", "the input must hold the chosen item");
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    click(cx, 60., 64.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "a pointer selection must clear uncontrolled open state"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

#[gpui::test]
fn dropdown_arrows_and_enter_activate(cx: &mut TestAppContext) {
    let actions = events();
    let fired = actions.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let actions = actions.clone();
        let skip_actions = actions.clone();
        let opens = opens.clone();
        // Two dropdowns stacked 320px apart, so the first menu (which ends
        // near y 170) never overlaps the second trigger.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(320.))
            .child(
                Dropdown::uncontrolled(
                    "dd-open",
                    Button::new("dd-open-trigger").label("Actions"),
                    vec![
                        MenuItem::new("open", "Open"),
                        MenuItem::new("close", "Close"),
                    ],
                )
                .id("dd-open")
                .on_action(move |key, _, _| actions.borrow_mut().push(key.to_string()))
                .on_open_change(move |open, _, _| {
                    opens.borrow_mut().push(format!("open:{open}"));
                }),
            )
            .child(
                Dropdown::uncontrolled(
                    "dd-skip",
                    Button::new("dd-skip-trigger").label("Skipper"),
                    vec![
                        MenuItem::new("cut", "Cut"),
                        MenuItem::new("del", "Delete"),
                        MenuItem::new("zoom", "Zoom"),
                    ],
                )
                .id("dd-skip")
                .disabled_keys(["del"])
                .on_action(move |key, _, _| skip_actions.borrow_mut().push(key.to_string())),
            )
            .into_any_element()
    });

    // First menu: the trigger button is 36px tall at the origin, centre
    // (40, 18). Opening moves the focus into the panel, so the arrows work
    // without a click first. An Enter choice closes the menu, as v3's does.
    click(cx, 40., 18.);
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["open"],
        "Down then Enter must activate the first enabled item exactly once"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the Enter pick must dismiss the menu"
    );

    // Second menu: its root starts at 36px + the 320px gap, so its trigger
    // centre is (40, 374). The cut pick closed the menu, so reopen it first;
    // then Down twice must step Cut -> Zoom, skipping the disabled Delete
    // between them.
    click(cx, 40., 374.);
    press(cx, "down");
    press(cx, "enter");
    click(cx, 40., 374.);
    press(cx, "down");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        fired.borrow().as_slice(),
        ["open", "cut", "zoom"],
        "the arrows must skip disabledKeys: activating the second stop \
         records 'zoom', not 'del'"
    );
}

#[gpui::test]
fn date_picker_opens_and_picks_a_day(cx: &mut TestAppContext) {
    let picks = events();
    let recorded = picks.clone();
    let state = cx.new(|cx| CalendarState::new(cx));

    // The visible month is the state's view month, seeded from today. Column
    // c of the first week holds day `c - lead + 1` once the lead blanks are
    // past, so the last column always holds day `7 - lead` — a real day of
    // this month whatever the month is.
    let today = Date::today();
    let lead = DateConstraints::new().lead_cells(today.year, today.month);
    let expected = Date::new(today.year, today.month, (7 - lead) as u32);

    // x: the calendar column is CALENDAR_WIDTH wide (252px = seven cells),
    // six 2px gaps take 12px, so each slot is (252-12)/7 wide and the last
    // column's centre sits at 12 + 6*(w+2) + w/2 from the panel origin.
    // y: panel top (36 trigger + 6 offset) + picker_panel p(12) + nav header
    // h(24) + two calendar gaps of 8 + one weekday-header text line (~16) +
    // half of a 36px cell. Only the text line is a metric rather than a
    // constant; any value it takes in 0..34 keeps y = 128 inside the first
    // week's cells.
    let cell_w = (f32::from(CALENDAR_WIDTH) - 12.) / 7.;
    let day_x = 12. + 6. * (cell_w + 2.) + cell_w / 2.;
    let day_y = 128.;

    let state_for_view = state;
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        DatePicker::new(state_for_view.clone())
            .on_change(move |date, _, _| {
                let iso = date.map(|d| d.format_iso()).unwrap_or_default();
                picks.borrow_mut().push(iso);
            })
            .into_any_element()
    });

    click(cx, 124., 18.);
    click(cx, day_x, day_y);

    assert_eq!(
        recorded.borrow().as_slice(),
        [expected.format_iso()],
        "clicking the last cell of the first week must pick day {}",
        7 - lead
    );
}
