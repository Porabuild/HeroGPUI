//! Behaviour tests for the pickers' remaining props and the Drawer's
//! un-driven surface: `allowsEmptyCollection` on Autocomplete,
//! `allowsCustomValue` and `validate` on ComboBox, `shouldFocusWrap` /
//! `disabledKeys` on Select and Autocomplete, the clear affordance on
//! Autocomplete, and the Drawer's size.
//!
//! "The Drawer's size" needs a correction first, because v3 has no such prop:
//! `### Drawer.Content` documents only `placement` and `className`, and
//! `drawer.css` sizes the sheet by placement (`w-80 max-w-[85vw] sm:w-96` for
//! left/right, `max-h-[85vh]` for top/bottom). The size presets live on the
//! **Modal** (`Modal.Container size="xs"…"full"`), whose six values
//! `placement.rs` already drives. There is nothing in the port to test for a
//! Drawer size either (`drawer.rs` has `id`, `is_open`, `placement`, … and no
//! size), so this file pins the Drawer props that have never been driven
//! instead: the close button, `on_close`, `isKeyboardDismissDisabled`,
//! `hideCloseButton`, `isDismissible` and the footer slot.
//!
//! What is deliberately NOT duplicated here:
//!
//! - `combo_box_open.rs` drives `MenuTrigger::Manual` (typing does not open,
//!   the chevron does) along with the Focus and Input triggers.
//! - `pickers.rs` drives Autocomplete open/filter/select, escape, and the
//!   arrows-and-Enter pick; ComboBox chevron-open and click-select.
//! - `calendars_and_more.rs` drives Select typeahead, multiple mode and the
//!   section heading (for Select).
//! - `placement.rs` drives every Drawer placement and every ModalSize.
//!
//! Geometry is derived from the port's own constants, like the rest of the
//! suite:
//!
//! - Every trigger field is a 36px row (`util::FIELD_HEIGHT`) at the window
//!   origin, so its centre is (60, 18), and the pickers' panels hang from
//!   `placed_field_panel(BottomStart, 6px)`: top = 42.
//! - Autocomplete: panel `pt(8)` + search wrapper `py(4)` + 36px field + list
//!   `p(6)` puts row *i* at y 100+36i; clicking y = 124+36i lands inside it in
//!   every phase of the entry zoom. A `section_before` heading rides above its
//!   item inside the same slot: `pt(6) pb(2)` at 12px (~19.4px line at gpui's
//!   phi default), so for the slot starting at y S the heading occupies
//!   roughly S+6..S+27 and the row S+27..S+63; the heading probe clicks
//!   S+14 and the option S+42.
//! - The Autocomplete clear button is the 20px (`size-5`) box in the trigger's
//!   flex row; the 320px trigger (`max_w(320)`) has `pr(28)`, so the button
//!   ends at x = 292 and centres at (282, 18), clear of the absolute chevron
//!   (`right(8)`, 16px, x 296..312).
//! - ComboBox: panel `p(4)` puts row *i* at y 46+36i; clicking y = 64+36i
//!   covers it. The field caps at `max_w(320)`, so the chevron centre is
//!   (298, 18).
//! - Select: panel `py(6)` puts option *i*'s centre at y 66+36i.
//! - Drawer (window 1920x1080, 384px desktop side width): the Right panel is
//!   x 1536..1920, y 0..1080, `p-6` (24px). Inside it: the handle
//!   (bar 4px + `pb-2` 8px) at y 24..36, the 24px title line (the drag
//!   surface) at y 36..60, the close trigger (`absolute end-4 top-4` around
//!   the 24px CloseButton) centred at (1892, 28), the body at 24+12+24+8 = 68
//!   (probe centre (1580, 86)). The body is `flex_1`, so the footer stays at
//!   the bottom of the full-height side sheet; its 40x36 probe centres at
//!   (1876, 1038).
//!
//! The Drawer tests set reduced motion before the first frame (an entry slide
//! would sit at its t=0 off-window pose for the whole test otherwise) and
//! advance the clock past `EXITING_MS` before any closed-proof probe, exactly
//! as `placement.rs` does. The pickers need neither: their panels leave the
//! tree outright when closed.

mod harness;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    point, prelude::*, px, ElementId, Focusable, Modifiers, MouseButton, TestAppContext,
    VisualTestContext,
};
use harness::{click, events, open_host, press, Events};
use herogpui_components::{
    Autocomplete, ComboBox, Drawer, DrawerPlacement, Form, FormData, Input, InputState,
    MenuTrigger, PickerItem, Select, SelectionMode,
};

/// An `InputState` entity for the search-field-backed controls, created before
/// the host opens so the test can keep its own handle to it.
fn search_state(cx: &mut TestAppContext) -> gpui::Entity<InputState> {
    cx.new(|cx| InputState::new(cx))
}

/// Items whose labels are unique, so the key can be the label itself. Tests
/// that need duplicate labels build explicit keys instead.
fn keyed(labels: &[&str]) -> Vec<PickerItem> {
    labels
        .iter()
        .map(|l| PickerItem::new(l.to_string(), l.to_string()))
        .collect()
}

/// A fixed-size pressable probe: records `label` on click. Every geometry
/// claim in the Drawer section is proven by placing one of these where the
/// component under test is computed to be and asserting its click records.
fn probe(id: impl Into<ElementId>, label: &'static str, recorded: Events) -> gpui::AnyElement {
    gpui::div()
        .id(id)
        .w(px(40.))
        .h(px(36.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.to_owned()))
        .child(label)
        .into_any_element()
}

// -- Drawer helpers, as `placement.rs` uses them -----------------------------

/// Pins the layout by enabling reduced motion **before** the first frame:
/// without it the panel sits at its t=0 pose (fully off-window) for the whole
/// test, because entry animations run on wall time the test clock does not
/// drive.
fn still() {
    harness::still();
}

/// Pushes the pending frame through. Mouse events hit-test the last rendered
/// frame, so a press whose effect the next event must see needs a redraw
/// first.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// Advances the test clock past `EXITING_MS` (100ms) plus slack and forces the
/// repaint the exit timer only scheduled. A closed-proof probe must not land
/// on the exiting, still-mounted panel.
fn let_exit_finish(cx: &mut VisualTestContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    flush_frame(cx);
}

/// One simulated drag: press at `from`, move to `to` with the button held,
/// release there. For the Drawer the press must land on the title row, where
/// the header's `on_mouse_down` writes the drag record.
fn drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
}

// ---------------------------------------------------------------------------
// Autocomplete
// ---------------------------------------------------------------------------

#[gpui::test]
fn autocomplete_programmatic_focus_departure_closes_without_refocusing(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let search = search_state(cx);
    let next = search_state(cx);
    let search_for_view = search;
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        gpui::div()
            .child(
                Autocomplete::new(search_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .default_open(true)
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    flush_frame(cx);
    cx.update(|window, cx| {
        let next = next.read(cx).focus_handle(cx);
        window.focus(&next);
    });
    flush_frame(cx);

    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
    assert!(cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)));
}

/// `allowsEmptyCollection` keeps the popover mounted when a query matches
/// nothing: v3 documents it as *"Whether the autocomplete allows an empty
/// collection. When true, the autocomplete can function even with no items."*
/// and its example prose adds *"This is useful for scenarios where the list
/// might be empty initially or when all items are filtered out."* The empty
/// state has no handlers, so a press where a row would be records nothing and
/// does not close; only a press well outside the panel dismisses it.
#[gpui::test]
fn autocomplete_allows_empty_collection_keeps_the_panel_up_with_no_matches(
    cx: &mut TestAppContext,
) {
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
        Autocomplete::new(state, keyed(&["Typst", "Rust", "Go"]))
            .allows_empty_collection(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // "zz" matches nothing. The empty state ("No results found") sits where
    // the rows would be; the press lands inside the panel, so it records
    // nothing and must not dismiss either.
    cx.simulate_input("zz");
    click(cx, 60., 124.);
    assert!(
        recorded.borrow().is_empty(),
        "the empty state must not select anything"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a press inside the empty panel must not dismiss it"
    );

    // Only a press outside the mounted panel dismisses it — which proves the
    // panel was still up after the no-match query. Were the panel gone, this
    // outside press would reach nothing and record no dismissal.
    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the empty panel must still answer its outside-press dismissal"
    );
    assert!(recorded.borrow().is_empty());
}

/// `allowsEmptyCollection` is **not** a close-on-filtered-empty flag. v3's
/// Autocomplete root is a React Aria `Select` and its filtering is a separate
/// layer (`Autocomplete.Filter` is RAC's `Autocomplete`), so a query that
/// prunes an open popover to zero changes nothing about the popover's open
/// state: the panel stays mounted and its composed "No results found" state
/// renders, with or without the prop. The prop only governs
/// whether a *collection with no items at all* may function at all.
#[gpui::test]
fn autocomplete_filtered_empty_without_the_prop_keeps_the_panel_mounted(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, keyed(&["Typst", "Rust", "Go"]))
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // "zz" matches nothing. The empty state ("No results found") sits where
    // the rows would be; the press lands inside the still-mounted panel, so
    // it records nothing and must not dismiss it either.
    cx.simulate_input("zz");
    click(cx, 60., 124.);
    assert!(
        recorded.borrow().is_empty(),
        "the empty state must not select anything"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "a filtered-empty query must keep the popover mounted even without \
         the prop, so a press inside it cannot dismiss it"
    );

    // The panel is still mounted, so a press outside it dismisses it and
    // reports the close.
    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the mounted empty panel must still answer its outside-press dismissal"
    );
    assert!(recorded.borrow().is_empty());
}

/// A collection with no items at all is a different gate, and this is what
/// `allowsEmptyCollection` is for. React Aria's `useSelectState.toggle` on
/// an empty collection early-returns ("Don't open if the collection is
/// empty."), so the trigger of a zero-item Autocomplete without the prop
/// neither opens nor reports `onOpenChange(true)` — and a second press must
/// not toggle either.
#[gpui::test]
fn autocomplete_zero_item_collection_without_the_prop_refuses_the_trigger(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, Vec::new())
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // The trigger press is the `useSelectState.toggle()` act: with an empty
    // collection and no prop it must do nothing at all — no open, no report.
    click(cx, 60., 18.);
    assert!(
        opened.borrow().is_empty(),
        "a zero-item collection without allowsEmptyCollection must refuse \
         the trigger toggle and never report open"
    );

    // The refusal covers the toggle in both directions: a second press must
    // not open-and-close either (or report anything).
    click(cx, 60., 18.);
    assert!(
        opened.borrow().is_empty(),
        "a zero-item collection without allowsEmptyCollection must refuse \
         every trigger toggle"
    );
    assert!(recorded.borrow().is_empty());
}

/// v3 also uses `allowsEmptyCollection` for a collection that *starts* empty
/// ("the autocomplete can function even with no items"): with `items = []` the
/// trigger still opens, the empty state is still shown, and nothing can be
/// selected — the panel is dismissible by an outside press like any other.
#[gpui::test]
fn autocomplete_allows_empty_collection_with_no_items_opens_anyway(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, Vec::new())
            .allows_empty_collection(true)
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
        "an empty collection must still open under the prop"
    );

    // The empty state occupies the popover; nothing there answers.
    click(cx, 60., 124.);
    assert!(recorded.borrow().is_empty());
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the empty panel must still dismiss from an outside press"
    );
}

/// `disabledKeys` rows render but cannot be chosen: a pointer press on one
/// records nothing (the row has no handler and sits inside the panel, so the
/// press neither selects nor dismisses), and the arrows skip it — the cursor's
/// stops are the enabled indices, so Down from the first row lands on the
/// third.
#[gpui::test]
fn autocomplete_disabled_rows_are_unclickable_and_not_a_stop(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma", "Delta"]))
            .disabled_keys(["Beta".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The keyboard leg runs on a freshly opened panel: the popover's search
    // field takes the focus on the first frame, so Down lands on Alpha (0),
    // the next Down skips the disabled Beta and lands on Gamma (2), and Enter
    // takes the row the cursor is on. The flush renders the opened panel
    // before the keys arrive (mouse events hit-test the last rendered frame).
    flush_frame(cx);
    press(cx, "down");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Gamma"],
        "the arrows must skip the disabled row"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // Reopen for the pointer leg: flush the closed state so the next trigger
    // press is a genuine open (a press against the last drawn frame would
    // toggle that frame's `was_open` instead).
    flush_frame(cx);
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"]
    );
    flush_frame(cx);

    // Row 1 is Beta, y = 124 + 36. A press on it must record nothing and must
    // not dismiss the panel.
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Gamma"],
        "a disabled row must not answer the pointer"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "a press on a disabled row must not dismiss the panel either"
    );
}

/// A `ListBox.Section` heading rides above the item it introduces and is not a
/// selectable thing of its own: a click on the heading records nothing and
/// keeps the panel open, and the arrows land on options — the heading never
/// consumes a stop, so three Downs from the top reach the item under it.
#[gpui::test]
fn autocomplete_section_heading_is_never_a_stop(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma", "Delta"]))
            .section_before("Gamma", "Tropical")
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Row 0 (Alpha) is a normal pick: y = 124.
    click(cx, 60., 124.);
    assert_eq!(recorded.borrow().as_slice(), ["Alpha"]);
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // The section's slot starts at y = 100 + 36*2 = 172: the heading spans
    // ~172..199 (pt-6 + a 12px line + pb-2) and the option 36px below it.
    // A press at 172+14 = 186 hits the heading: no selection, no dismissal.
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"]
    );
    click(cx, 60., 186.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha"],
        "a section heading must not be selectable"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "a press on the heading must not dismiss the panel"
    );

    // The option the heading announces is a normal pick: y = 172 + 42 = 214.
    click(cx, 60., 214.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha", "Gamma"],
        "the option under the heading must still be clickable"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true", "open:false"]
    );

    // Keyboard: the stops are the four item indices (the heading is a drawing,
    // not a row), so Down, Down, Down steps 0 -> 1 -> 2 and Enter takes the
    // item the heading introduces.
    click(cx, 60., 18.);
    press(cx, "down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha", "Gamma", "Gamma"],
        "the arrows must step over the heading to the option under it"
    );
}

/// `max_items` caps the list after filtering. With five items and a cap of
/// two, the empty query shows exactly two rows: the third Down clamps on the
/// second stop instead of reaching the third item (an uncapped list would
/// land on "Beta"), and a query matching four items still caps to two — the
/// third Down clamps again, so "Alpine" comes out both times.
#[gpui::test]
fn autocomplete_max_items_caps_the_long_list(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(
            state,
            keyed(&["Alpha", "Alpine", "Beta", "Bravo", "Charlie"]),
        )
        .max_items(2)
        .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
        .into_any_element()
    });

    click(cx, 60., 18.);
    // Empty query: the first two items, so the stops are {0, 1}. The fourth
    // Down is the clamp: 0 -> 1 -> 1, and Enter reports "Alpine" — an
    // uncapped list would have stepped to "Beta".
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpine"],
        "down past the capped ends must clamp, not reach the third item"
    );

    // "a" matches Alpha, Alpine, Bravo and Charlie: filtered down to the same
    // two, so the keyboard behaves identically — the cap is applied after
    // filtering.
    click(cx, 60., 18.);
    cx.simulate_input("a");
    press(cx, "down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpine", "Alpine"],
        "a filtering query must still cap to max_items rows"
    );
}

/// `on_clear` appears only when there is something to clear (v3's data-empty
/// state): with a seeded selection the 20px button at (282, 18) answers, and
/// after it fires the button is gone.
///
/// The clear button must not *also* open the popover. gpui dispatches a click
/// to the hit element and on up to its ancestors, and the clear button lives
/// *inside* the trigger, so its click would bubble into the trigger's own
/// `on_click`. v3 cannot do this: React Aria's trigger press is bound to
/// pointer events, so a bubbled DOM `click` is inert there. The button's own
/// handler stops that propagation.
#[gpui::test]
fn autocomplete_clear_button_fires_on_clear_and_vanishes(cx: &mut TestAppContext) {
    let clears = events();
    let cleared = clears.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let clears = clears.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma"]))
            .default_value(["Alpha"])
            .on_clear(move |_, _| clears.borrow_mut().push("cleared".into()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // The trigger holds the seeded selection, so the clear button is drawn at
    // x 292 - 10 (the 20px box ends at the 28px right padding of the 320px
    // trigger), centre (282, 18). A press there fires on_clear and must not
    // open anything.
    click(cx, 282., 18.);
    assert_eq!(
        cleared.borrow().as_slice(),
        ["cleared"],
        "the clear button must fire on_clear"
    );
    assert!(
        opened.borrow().is_empty(),
        "clearing is not an open gesture — the pressed clear button must not \
         bubble its click into the trigger"
    );
    flush_frame(cx);

    // The button is gone once nothing is selected, so a second press on the
    // same spot cannot clear again.
    click(cx, 282., 18.);
    assert_eq!(
        cleared.borrow().as_slice(),
        ["cleared"],
        "after the clear the button must be gone"
    );
}

#[gpui::test]
fn autocomplete_clear_button_works_without_an_on_clear_callback(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta"]))
            .default_value(["Alpha"])
            .on_selection_change_all(move |keys, _, _| {
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

    click(cx, 282., 18.);
    assert_eq!(
        selected.borrow().as_slice(),
        [""],
        "ClearButton must report the empty selection even without on_clear"
    );
    assert!(
        opened.borrow().is_empty(),
        "clearing without on_clear must not bubble into the trigger"
    );
}

#[gpui::test]
fn autocomplete_controlled_clear_waits_for_the_owner(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta"]))
            .value(["Alpha"])
            .on_selection_change_all(move |keys, _, _| {
                selections.borrow_mut().push(keys.len().to_string());
            })
            .into_any_element()
    });

    click(cx, 282., 18.);
    flush_frame(cx);
    click(cx, 282., 18.);
    assert_eq!(
        selected.borrow().as_slice(),
        ["0", "0"],
        "controlled clear must report empty without mutating the owner's value"
    );
}

#[gpui::test]
fn autocomplete_clear_handles_a_controlled_key_missing_from_items(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta"]))
            .value(["loading-item"])
            .on_selection_change_all(move |keys, _, _| {
                selections.borrow_mut().push(keys.len().to_string());
            })
            .into_any_element()
    });

    click(cx, 282., 18.);
    assert_eq!(
        selected.borrow().as_slice(),
        ["0"],
        "a selected key remains clearable while its item is not loaded"
    );
}

#[gpui::test]
fn autocomplete_disabled_clear_is_inert(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha"]))
            .default_value(["Alpha"])
            .is_disabled(true)
            .on_selection_change_all(move |_, _, _| {
                selections.borrow_mut().push("changed".into());
            })
            .into_any_element()
    });

    click(cx, 282., 18.);
    assert!(
        selected.borrow().is_empty(),
        "a disabled Autocomplete must not expose an active clear button"
    );
}

#[gpui::test]
fn autocomplete_read_only_clear_is_inert(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha"]))
            .default_value(["Alpha"])
            .is_read_only(true)
            .on_selection_change_all(move |_, _, _| {
                selections.borrow_mut().push("changed".into());
            })
            .into_any_element()
    });

    click(cx, 282., 18.);
    assert!(
        selected.borrow().is_empty(),
        "a read-only Autocomplete must not expose an active clear button"
    );
}

/// The clear drops the component's *owned* selection, proven through the
/// toggle in multiple mode: with {Alpha, Beta} seeded, picking Gamma reports
/// the three, and after the clear picks Alpha reports only {"Alpha"}. Were the
/// clear a no-op on the held set, the stored {Alpha, Beta, Gamma} would toggle
/// Alpha *off* and the report would be "Beta,Gamma".
#[gpui::test]
fn autocomplete_clear_empties_the_owned_selection_before_the_next_pick(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let clear_events = picks.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma", "Delta"]))
            .selection_mode(SelectionMode::Multiple)
            .default_value(["Alpha", "Beta"])
            .on_clear(move |_, _| clear_events.borrow_mut().push("clear:cleared".into()))
            .on_selection_change_all(move |keys, _, _| {
                let joined = keys
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                picks.borrow_mut().push(joined);
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // Row 2 (Gamma) joins the seeded pair: y = 124 + 2*36 = 196.
    click(cx, 60., 196.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Alpha,Beta,Gamma"],
        "the pick must join the seeded selection"
    );

    // The clear button sits on the trigger, above the open panel, so it can
    // be pressed while the panel is up.
    click(cx, 282., 18.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Alpha,Beta,Gamma", "", "clear:cleared"],
        "selection change must report before on_clear"
    );

    // Row 0 (Alpha) after the clear: {"Alpha"}. A stale held set would have
    // reported "Beta,Gamma".
    click(cx, 60., 124.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Alpha,Beta,Gamma", "", "clear:cleared", "Alpha"],
        "the clear must empty the owned selection, so the next pick starts \
         from nothing"
    );
}

/// `shouldFocusWrap = false` holds at the ends: Down from the last row
/// clamps there (Enter then reports the last item, not a wrapped first), and
/// Up from the first row clamps at the top. The cursor persists across
/// reopenings, which is what makes the second leg deterministic.
#[gpui::test]
fn autocomplete_no_wrap_holds_at_both_ends(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma"]))
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    // Four Downs from the top: 0 -> 1 -> 2 -> 2 (clamped). Enter reports the
    // last row — a wrapping list would have come back around to "Alpha".
    click(cx, 60., 18.);
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Gamma"],
        "down past the end must hold on the last row without wrap"
    );

    // Reopened, the cursor is still on Gamma (2); three Ups step 2 -> 1 -> 0
    // -> 0 (clamped) and Enter reports the first row.
    click(cx, 60., 18.);
    press(cx, "up up up");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Gamma", "Alpha"],
        "up past the start must hold on the first row without wrap"
    );
}

/// `shouldFocusWrap = true` joins the ends: Down from the last row wraps to
/// the first, and Up from the first wraps to the last.
#[gpui::test]
fn autocomplete_wrap_joins_both_ends(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, keyed(&["Alpha", "Beta", "Gamma"]))
            .should_focus_wrap(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .into_any_element()
    });

    // Fourth Down wraps 2 -> 0; Enter reports "Alpha".
    click(cx, 60., 18.);
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha"],
        "down past the end must wrap to the first row"
    );

    // Reopened, the cursor is on Alpha (0); one Up wraps to Gamma (2).
    click(cx, 60., 18.);
    press(cx, "up");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha", "Gamma"],
        "up past the start must wrap to the last row"
    );
}

// -- Autocomplete keyed identity ----------------------------------------------
//
// Pinned v3.2.4 / React Aria Components 1.20.0 keep a stable `Key` separate
// from each item's `textValue`: the selection, `disabledKeys`, the callbacks
// and the form value address items by key, while filtering and the visible
// text use labels. The tests below use two items that share the label "Rust"
// with the distinct keys "rust" and "rusty" — under the old label-as-key
// identity every one of these expectations fails, because the duplicates
// aliased each other's selection, disabled state and row element id.

/// Two same-label items with distinct keys: the duplicate-labelled collection
/// the keyed-identity tests in this section drive.
fn duplicate_labels() -> Vec<PickerItem> {
    vec![
        PickerItem::new("rust", "Rust"),
        PickerItem::new("rusty", "Rust"),
        PickerItem::new("go", "Go"),
    ]
}

/// Picking the *second* of two same-label rows must report that row's own key
/// and render the label once — not alias onto the first item's key.
#[gpui::test]
fn autocomplete_duplicate_labels_pick_the_second_by_its_key(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let values = Rc::new(RefCell::new(Vec::new()));
    let seen_values = values.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let seen_values = seen_values.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, duplicate_labels())
            .value_content(move |v| {
                seen_values.borrow_mut().push(format!(
                    "text:{} keys:{} indices:{}",
                    v.selected_text,
                    v.selected_keys
                        .unwrap_or(&[])
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    v.selected_indices
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ));
                v.default_children
            })
            .on_change(move |key, _, _| changes.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    // Row 1 is the second "Rust", key "rusty": y = 124 + 36.
    click(cx, 60., 18.);
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty"],
        "the second same-label row must report its own key, not the first \
         item's key"
    );
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust keys:rusty indices:1"),
        "the trigger must render the selected key's label once, with the \
         selected key and collection index of the row that was picked"
    );

    // Reopening and picking the first "Rust" reports the *other* key — the
    // two rows are distinct selections.
    click(cx, 60., 18.);
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty", "rust"],
        "the first same-label row must keep its own key too"
    );
}

/// With both same-label items selected, the trigger renders each selected
/// item once in the selection's own order — two picks with one label are two
/// labels, and the render props see the keys behind them.
#[gpui::test]
fn autocomplete_duplicate_labels_selected_together_render_each_once(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let values = Rc::new(RefCell::new(Vec::new()));
    let seen_values = values.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let seen_values = seen_values.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, duplicate_labels())
            .selection_mode(SelectionMode::Multiple)
            .default_value(["rust", "rusty"])
            .value_content(move |v| {
                seen_values.borrow_mut().push(format!(
                    "text:{} keys:{} indices:{}",
                    v.selected_text,
                    v.selected_keys
                        .unwrap_or(&[])
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    v.selected_indices
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ));
                v.default_children
            })
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust, Rust keys:rust,rusty indices:0,1"),
        "both same-label selections must render, once each, in the \
         selection's own order — the keys prove they are two items, not one"
    );

    // Adding "Go" keeps every selected key distinct from its label, and the
    // callback reports the selection set's insertion order: the seeded pair
    // first, "go" appended last — not the collection's or a sorted order.
    click(cx, 60., 18.);
    click(cx, 60., 196.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust,rusty,go"],
        "the complete selection must report all three keys in insertion \
         order — a sorted set would have reported go,rust,rusty"
    );
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust, Rust, Go keys:rust,rusty,go indices:0,1,2"),
        "the trigger text must stay in the selection's own order with one \
         render per selected key"
    );
}

/// `disabledKeys` addresses one key: disabling the first "Rust" leaves the
/// same-label sibling enabled and clickable, and as a keyboard stop.
#[gpui::test]
fn autocomplete_duplicate_labels_disable_one_key_keeps_the_sibling_enabled(
    cx: &mut TestAppContext,
) {
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
        Autocomplete::new(state, duplicate_labels())
            .disabled_keys(["rust".into()])
            .on_change(move |key, _, _| changes.borrow_mut().push(key.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Keyboard leg on the freshly opened panel (a pointer press inside the
    // panel blurs the autofocused search field, so it always comes first):
    // Down enters the first *enabled* row — the sibling "rusty" at index 1 —
    // so Enter commits "rusty". Disabling both same-label items (the
    // aliasing bug) would have made "go" the first enabled stop instead.
    flush_frame(cx);
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty"],
        "disabling one key must leave its same-label sibling enabled and \
         selectable"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // Pointer leg on a reopened panel: a press on the disabled key's row
    // records nothing and leaves the panel up, while the same-label sibling
    // row next to it still answers.
    flush_frame(cx);
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"]
    );
    flush_frame(cx);
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty"],
        "the disabled key's row must not answer the pointer"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "a press on a disabled row must not dismiss the panel either"
    );
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty", "rusty"],
        "the same-label sibling row must answer the pointer"
    );
}

/// Two same-label rows are two interactive elements in the plain list path:
/// in multiple mode each row toggles only its own key.
#[gpui::test]
fn autocomplete_duplicate_labels_keep_distinct_rows_in_the_plain_path(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, duplicate_labels())
            .selection_mode(SelectionMode::Multiple)
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    // Row 0 at y = 124, row 1 at y = 160. The second click must ADD "rusty"
    // beside "rust" — one aliased row id would have made it toggle the first
    // pick off.
    click(cx, 60., 18.);
    click(cx, 60., 124.);
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust", "rust,rusty"],
        "the two same-label rows must be distinct clickable rows"
    );
    // A third click on the first row toggles only "rust" back off.
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust", "rust,rusty", "rusty"],
        "each row must toggle only its own key"
    );
}

/// The virtual (`row_height`) list builds its rows through `uniform_list`,
/// but the identity contract is the same: two same-label rows are two
/// distinct interactive rows that each report their own key.
#[gpui::test]
fn autocomplete_duplicate_labels_keep_distinct_rows_in_the_virtual_path(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, duplicate_labels())
            .selection_mode(SelectionMode::Multiple)
            .row_height(px(36.))
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    // Same row geometry as the plain path: row 0 at y = 124, row 1 at
    // y = 160.
    click(cx, 60., 18.);
    click(cx, 60., 124.);
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust", "rust,rusty"],
        "the two same-label rows must stay distinct rows under virtualization"
    );
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust", "rust,rusty", "rusty"],
        "each virtual row must toggle only its own key"
    );
}

/// A held selection is a set of keys: reordering the collection must not move
/// it onto another item, and the trigger keeps rendering the held key's label
/// from wherever that item now sits.
#[gpui::test]
fn autocomplete_selection_survives_an_item_reorder_by_key(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let values = Rc::new(RefCell::new(Vec::new()));
    let seen_values = values.clone();
    let state = search_state(cx);
    let state_for_view = state;
    let items = Rc::new(RefCell::new(duplicate_labels()));
    let items_for_view = items.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let seen_values = seen_values.clone();
        let state = state_for_view.clone();
        let items = items_for_view.borrow().clone();
        Autocomplete::new(state, items)
            .default_value(["rusty"])
            .value_content(move |v| {
                seen_values.borrow_mut().push(format!(
                    "text:{} keys:{}",
                    v.selected_text,
                    v.selected_keys
                        .unwrap_or(&[])
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ));
                v.default_children
            })
            .on_change(move |key, _, _| changes.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust keys:rusty"),
        "the seeded key must resolve to its item's label"
    );

    // Reorder the collection: "rusty" moves from index 1 to the end.
    *items.borrow_mut() = vec![
        PickerItem::new("go", "Go"),
        PickerItem::new("rust", "Rust"),
        PickerItem::new("rusty", "Rust"),
    ];
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust keys:rusty"),
        "the held selection must follow its key through the reorder, not \
         the row the label used to sit on"
    );

    // Picking the row that now sits at index 1 commits that row's own key.
    click(cx, 60., 18.);
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rust"],
        "the row's key must be reported from its new position"
    );
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust keys:rust"),
        "the trigger must show the newly committed key's label"
    );
}

// -- Autocomplete selection order ---------------------------------------------
//
// Pinned react-stately 3.49.0's `useSelectState` keeps `selectedKeys` as a
// JavaScript `Set`, which iterates in insertion order: `selectedItems`,
// `selectedText`, the `onChange` slice and the form value all follow the
// order keys were picked (or the owner listed) in — never the collection's
// order and never a sorted order.

/// Nonlexicographic keys whose collection order differs from any pick order:
/// a sorted set would report Alpha,Mike,Zulu; the pick order below never is.
fn misordered() -> Vec<PickerItem> {
    keyed(&["Zulu", "Alpha", "Mike"])
}

/// Multiple picks must report and render in insertion order, whichever way
/// each pick was made: the keyboard leg toggles "Mike" on first, then pointer
/// picks append "Zulu" and "Alpha", and toggling "Zulu" off removes it in
/// place — re-adding appends it at the end.
#[gpui::test]
fn autocomplete_multiple_picks_follow_insertion_order(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Autocomplete::new(state, misordered())
            .selection_mode(SelectionMode::Multiple)
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    // Keyboard leg: three Downs walk 0 -> 1 -> 2 and Enter toggles "Mike" on.
    click(cx, 60., 18.);
    press(cx, "down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Mike"],
        "the keyboard pick must report the toggled key"
    );

    // Pointer legs: "Zulu" then "Alpha" append in pick order.
    click(cx, 60., 124.);
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Mike", "Mike,Zulu", "Mike,Zulu,Alpha"],
        "pointer picks must append in pick order — a sorted set would have \
         reported Alpha,Mike,Zulu"
    );

    // Toggling "Zulu" off removes it in place — "Alpha" keeps its position
    // behind "Mike" — and toggling it back on appends it at the end.
    click(cx, 60., 124.);
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [
            "Mike",
            "Mike,Zulu",
            "Mike,Zulu,Alpha",
            "Mike,Alpha",
            "Mike,Alpha,Zulu"
        ],
        "remove-in-place must keep the remaining order and re-adding must \
         append at the end"
    );
}

/// A controlled selection's order is the owner's order: the render props see
/// the keys exactly as `value` listed them, and a pick appends to that order
/// in the reported slice.
#[gpui::test]
fn autocomplete_controlled_value_keeps_the_owner_order(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let values = Rc::new(RefCell::new(Vec::new()));
    let seen_values = values.clone();
    let current = Rc::new(RefCell::new(vec![
        gpui::SharedString::from("k-mid"),
        gpui::SharedString::from("k-last"),
    ]));
    let state = search_state(cx);
    let state_for_view = state;
    let current_for_view = current;
    let items = vec![
        PickerItem::new("k-last", "One"),
        PickerItem::new("k-mid", "Two"),
        PickerItem::new("k-first", "Three"),
    ];

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let seen_values = seen_values.clone();
        let current = current_for_view.clone();
        let state = state_for_view.clone();
        let selected = current.borrow().clone();
        Autocomplete::new(state, items.clone())
            .selection_mode(SelectionMode::Multiple)
            .value(selected)
            .value_content(move |v| {
                seen_values.borrow_mut().push(format!(
                    "text:{} keys:{} indices:{}",
                    v.selected_text,
                    v.selected_keys
                        .unwrap_or(&[])
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    v.selected_indices
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ));
                v.default_children
            })
            .on_selection_change_all(move |keys, _, _| {
                *current.borrow_mut() = keys.to_vec();
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Two, One keys:k-mid,k-last indices:1,0"),
        "the controlled render must follow the owner's listed order — keys, \
         resolved items and collection indices alike — not the collection's \
         or a sorted order"
    );

    // Picking "Three" appends to the owner's order; the owner accepts the
    // reported slice, so the next frame renders it back.
    click(cx, 60., 18.);
    click(cx, 60., 196.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["k-mid,k-last,k-first"],
        "the reported slice must append the pick to the owner's order"
    );
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Two, One, Three keys:k-mid,k-last,k-first indices:1,0,2"),
        "the trigger render props must agree with the reported slice"
    );
}

/// The uncontrolled default order persists, and reordering the collection
/// must not reorder the selected keys' history: only the indices re-resolve
/// to wherever each key's item now sits.
#[gpui::test]
fn autocomplete_default_order_persists_through_a_collection_reorder(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let values = Rc::new(RefCell::new(Vec::new()));
    let seen_values = values.clone();
    let state = search_state(cx);
    let state_for_view = state;
    let items = Rc::new(RefCell::new(duplicate_labels()));
    let items_for_view = items.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let seen_values = seen_values.clone();
        let state = state_for_view.clone();
        let items = items_for_view.borrow().clone();
        Autocomplete::new(state, items)
            .selection_mode(SelectionMode::Multiple)
            .default_value(["rusty", "rust"])
            .value_content(move |v| {
                seen_values.borrow_mut().push(format!(
                    "text:{} keys:{} indices:{}",
                    v.selected_text,
                    v.selected_keys
                        .unwrap_or(&[])
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                    v.selected_indices
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ));
                v.default_children
            })
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust, Rust keys:rusty,rust indices:1,0"),
        "the default's listed order must persist — a sorted set would have \
         reported rust,rusty"
    );

    // Reorder the collection: "go" moves to the front. The key history must
    // not move; only the indices re-resolve.
    *items.borrow_mut() = vec![
        PickerItem::new("go", "Go"),
        PickerItem::new("rust", "Rust"),
        PickerItem::new("rusty", "Rust"),
    ];
    flush_frame(cx);
    assert_eq!(
        values.borrow().last().map(String::as_str),
        Some("text:Rust, Rust keys:rusty,rust indices:2,1"),
        "a collection reorder must not reorder the selected keys' history"
    );

    // The next pick appends to the preserved history, wherever its row sits.
    click(cx, 60., 18.);
    click(cx, 60., 124.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["rusty,rust,go"],
        "the pick must append to the preserved selection order"
    );
}

/// The form value follows the same order: `FormData::get_all` must return the
/// picked keys in pick order, one value per key.
#[gpui::test]
fn autocomplete_form_get_all_follows_the_selection_order(cx: &mut TestAppContext) {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = misordered();
    let field = Autocomplete::new(state.clone(), items.clone())
        .selection_mode(SelectionMode::Multiple)
        .name("lang")
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form.borrow_mut().push(data.get_all("lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .selection_mode(SelectionMode::Multiple)
            .name("lang")
            .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 60., 124.);
    click(cx, 60., 160.);
    flush_frame(cx);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        [vec![
            gpui::SharedString::from("Zulu"),
            gpui::SharedString::from("Alpha")
        ]],
        "FormData::get_all must return the picked keys in pick order — a \
         sorted set would have returned Alpha,Zulu"
    );
}

/// Filtering matches labels, never keys: a query naming a key matches
/// nothing, while the same-label duplicates both answer a label query and
/// remain individually addressable.
#[gpui::test]
fn autocomplete_filter_matches_labels_not_keys(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    // Opaque keys that appear in no label: anything a "k" query reached would
    // be filtering on keys.
    let items = vec![
        PickerItem::new("k1", "Rust"),
        PickerItem::new("k2", "Rust"),
        PickerItem::new("k3", "Go"),
    ];

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        let items = items.clone();
        Autocomplete::new(state, items)
            .on_change(move |key, _, _| changes.borrow_mut().push(key.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // Typing "k2" — an existing item's key — must match nothing: the query
    // only ever reaches labels. The empty state sits where the rows would
    // be, so a press there records nothing and dismisses nothing.
    cx.simulate_input("k2");
    click(cx, 60., 124.);
    assert!(
        recorded.borrow().is_empty(),
        "a key-shaped query must not match any row: filtering runs on labels"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the filtered-empty panel must stay mounted"
    );

    // Reopen (that press blurred the search field, so the fresh popover
    // refocuses it) and filter by label instead: clear the key-shaped query,
    // type "rus", which matches both same-label duplicates, and pick the
    // second one — still its own row.
    click(cx, 60., 18.);
    click(cx, 60., 18.);
    flush_frame(cx);
    press(cx, "backspace backspace");
    cx.simulate_input("rus");
    click(cx, 60., 160.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["k2"],
        "a label query must surface both duplicates, each pickable by key"
    );
}

fn form_entry(data: &FormData, name: &str) -> String {
    data.get(name)
        .map_or_else(|| "omitted".to_owned(), |value| value.as_text().to_string())
}

/// Named Autocomplete must submit the live keyed selection, not the snapshot
/// `form_field()` saw when the Form was first told about the control.
#[gpui::test]
fn autocomplete_form_submits_live_uncontrolled_selection(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta", "Gamma"]);
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .default_value(["Alpha"])
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_rebuild = state.clone();
    let items_for_rebuild = items.clone();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .default_value(["Alpha"])
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["Alpha"]);

    click(cx, 60., 18.);
    click(cx, 60., 160.);
    flush_frame(cx);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        ["Alpha", "Beta"],
        "FormData must follow the runtime selection after a pick"
    );
    cx.update(|_, cx| {
        let rebuilt = Autocomplete::new(state_for_rebuild.clone(), items_for_rebuild.clone())
            .name("lang")
            .default_value(["Alpha"]);
        let form = Form::new().field(rebuilt.form_field().expect("rebuilt named Autocomplete"));
        assert_eq!(
            form_entry(&form.data(cx), "lang"),
            "Beta",
            "rebuilding form_field must not overwrite a live pick with defaultValue"
        );
    });
}

/// A controlled Autocomplete reports the pick but keeps submitting the owner's
/// value until that owner writes it back through `value`.
#[gpui::test]
fn autocomplete_form_waits_for_controlled_owner_acceptance(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let changes = events();
    let recorded = changes.clone();
    let current = Rc::new(RefCell::new(vec![gpui::SharedString::from("Alpha")]));
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta", "Gamma"]);
    let selected = current.borrow().clone();
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .value(selected)
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let current_for_view = current;
    let cx = open_host(cx, move || {
        let selected = current_for_view.borrow().clone();
        let current = current_for_view.clone();
        let changes = changes.clone();
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .value(selected)
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
                *current.borrow_mut() = keys.to_vec();
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 60., 160.);
    flush_frame(cx);
    assert_eq!(recorded.borrow().as_slice(), ["Beta"]);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        ["Beta"],
        "once the owner accepts the pick, FormData must follow that value"
    );
}

/// Controlled form data stays on the owner's value when the owner ignores the
/// change callback — the pick must not write through on its own.
#[gpui::test]
fn autocomplete_form_keeps_owner_value_when_controlled_pick_is_ignored(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let changes = events();
    let recorded = changes.clone();
    let current = Rc::new(RefCell::new(vec![gpui::SharedString::from("Alpha")]));
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta", "Gamma"]);
    let selected = current.borrow().clone();
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .value(selected)
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let current_for_view = current;
    let cx = open_host(cx, move || {
        let selected = current_for_view.borrow().clone();
        let changes = changes.clone();
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .value(selected)
            .on_selection_change_all(move |keys, _, _| {
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 60., 160.);
    flush_frame(cx);
    assert_eq!(recorded.borrow().as_slice(), ["Beta"]);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        ["Alpha"],
        "a controlled pick that the owner does not accept must not change FormData"
    );
}

#[gpui::test]
fn autocomplete_disabled_form_field_is_omitted(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta"]);
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .default_value(["Alpha"])
        .is_disabled(true)
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .default_value(["Alpha"])
            .is_disabled(true)
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["omitted"]);
}

#[gpui::test]
fn autocomplete_disabled_form_field_becomes_successful_after_rerender(cx: &mut TestAppContext) {
    let disabled = Rc::new(Cell::new(true));
    let disabled_for_view = disabled.clone();
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta"]);
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .default_value(["Alpha"])
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .default_value(["Alpha"])
            .is_disabled(disabled_for_view.get())
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["omitted"]);

    disabled.set(false);
    flush_frame(cx);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["omitted", "Alpha"]);
}

#[gpui::test]
fn autocomplete_read_only_form_field_remains_successful(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta"]);
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .default_value(["Alpha"])
        .is_read_only(true)
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .default_value(["Alpha"])
            .is_read_only(true)
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["Alpha"]);
}

#[gpui::test]
fn autocomplete_form_reset_restores_uncontrolled_default(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta", "Gamma"]);
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .default_value(["Alpha"])
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(form_entry(data, "lang"));
    });
    let submit = form.submit_handler();
    let reset = form.reset_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .default_value(["Alpha"])
            .into_any_element()
    });

    click(cx, 60., 18.);
    click(cx, 60., 160.);
    flush_frame(cx);
    cx.update(|window, cx| submit(window, cx));
    cx.update(|window, cx| reset(window, cx));
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        ["Beta", "Alpha"],
        "native reset must restore defaultValue in FormData without waiting for a repaint"
    );
}

#[gpui::test]
fn autocomplete_form_reset_reports_controlled_default_to_owner(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let picks = events();
    let recorded_picks = picks.clone();
    let current = Rc::new(RefCell::new(vec![gpui::SharedString::from("Beta")]));
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta", "Gamma"]);
    let selected = current.borrow().clone();
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .value(selected)
        .default_value(["Alpha"])
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field);
    let reset = form.reset_handler();
    let state_for_view = state;
    let current_for_view = current;
    let cx = open_host(cx, move || {
        let selected = current_for_view.borrow().clone();
        let current = current_for_view.clone();
        let changes = changes.clone();
        let picks = picks.clone();
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .value(selected)
            .default_value(["Alpha"])
            .on_selection_change_all(move |keys, _, _| {
                *current.borrow_mut() = keys.to_vec();
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .on_change(move |key, _, _| picks.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    cx.update(|window, cx| reset(window, cx));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha"],
        "controlled reset must report defaultValue so the owner can update"
    );
    assert!(
        recorded_picks.borrow().is_empty(),
        "form reset must not invoke Autocomplete's pick-only scalar callback"
    );
}

#[gpui::test]
fn autocomplete_form_reset_reports_disabled_controlled_default_to_owner(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let current = Rc::new(RefCell::new(vec![gpui::SharedString::from("Beta")]));
    let state = search_state(cx);
    let items = keyed(&["Alpha", "Beta"]);
    let selected = current.borrow().clone();
    let field = Autocomplete::new(state.clone(), items.clone())
        .name("lang")
        .value(selected)
        .default_value(["Alpha"])
        .is_disabled(true)
        .form_field()
        .expect("named Autocomplete");
    let form = Form::new().field(field);
    let reset = form.reset_handler();
    let state_for_view = state;
    let current_for_view = current;
    let cx = open_host(cx, move || {
        let selected = current_for_view.borrow().clone();
        let current = current_for_view.clone();
        let changes = changes.clone();
        Autocomplete::new(state_for_view.clone(), items.clone())
            .name("lang")
            .value(selected)
            .default_value(["Alpha"])
            .is_disabled(true)
            .on_selection_change_all(move |keys, _, _| {
                *current.borrow_mut() = keys.to_vec();
                changes.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    cx.update(|window, cx| reset(window, cx));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha"],
        "disabled controlled Autocomplete still notifies its owner on reset"
    );
}

// ---------------------------------------------------------------------------
// ComboBox
// ---------------------------------------------------------------------------

#[gpui::test]
fn combo_box_blur_restores_selected_text_closes_and_keeps_destination_focus(
    cx: &mut TestAppContext,
) {
    let callbacks = events();
    let recorded = callbacks.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let inputs = callbacks.clone();
        let opens = callbacks.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .default_value(["Alpha"])
                    .default_input_value("mismatch")
                    .default_open(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_input_change(move |value, _, _| {
                        inputs.borrow_mut().push(format!("input:{value}"));
                    })
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    cx.update(|window, cx| window.focus(&next.read(cx).focus_handle(cx)));
    flush_frame(cx);

    assert_eq!(
        combo.read_with(cx, |state, _| state.value().to_owned()),
        "Alpha"
    );
    assert_eq!(recorded.borrow().as_slice(), ["input:Alpha", "open:false"]);
    assert!(cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)));
}

#[gpui::test]
fn combo_box_custom_single_blur_keeps_text_and_clears_selection(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let inputs = events();
    let input_changes = inputs.clone();
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let inputs = inputs.clone();
        let opens = opens.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .default_value(["Alpha"])
                    .default_input_value("Custom")
                    .default_open(true)
                    .allows_custom_value(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_selection_change_all(move |keys, _, _| {
                        selections.borrow_mut().push(keys.join(","));
                    })
                    .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    cx.update(|window, cx| window.focus(&next.read(cx).focus_handle(cx)));
    flush_frame(cx);

    assert_eq!(
        combo.read_with(cx, |state, _| state.value().to_owned()),
        "Custom"
    );
    assert_eq!(selected.borrow().as_slice(), [""]);
    assert!(input_changes.borrow().is_empty());
    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
}

#[gpui::test]
fn combo_box_custom_multiple_blur_preserves_query_and_selection(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let inputs = events();
    let input_changes = inputs.clone();
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let inputs = inputs.clone();
        let opens = opens.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .selection_mode(SelectionMode::Multiple)
                    .default_value(["Alpha"])
                    .default_input_value("Custom")
                    .default_open(true)
                    .allows_custom_value(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_selection_change_all(move |keys, _, _| {
                        selections.borrow_mut().push(keys.join(","));
                    })
                    .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    cx.update(|window, cx| window.focus(&next.read(cx).focus_handle(cx)));
    flush_frame(cx);

    assert_eq!(
        combo.read_with(cx, |state, _| state.value().to_owned()),
        "Custom"
    );
    assert!(selected.borrow().is_empty());
    assert!(input_changes.borrow().is_empty());
    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
}

#[gpui::test]
fn combo_box_noncustom_multiple_blur_clears_only_the_query(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let inputs = events();
    let input_changes = inputs.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let inputs = inputs.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .selection_mode(SelectionMode::Multiple)
                    .default_value(["Alpha"])
                    .default_input_value("query")
                    .menu_trigger(MenuTrigger::Manual)
                    .on_selection_change_all(move |keys, _, _| {
                        selections.borrow_mut().push(keys.join(","));
                    })
                    .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned())),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    cx.update(|window, cx| window.focus(&next.read(cx).focus_handle(cx)));
    flush_frame(cx);

    assert_eq!(combo.read_with(cx, |state, _| state.value().to_owned()), "");
    assert!(selected.borrow().is_empty());
    assert_eq!(input_changes.borrow().as_slice(), [""]);
}

#[gpui::test]
fn combo_box_click_away_commits_and_closes_exactly_once(cx: &mut TestAppContext) {
    let inputs = events();
    let input_changes = inputs.clone();
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next;

    let cx = open_host(cx, move || {
        let inputs = inputs.clone();
        let opens = opens.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(300.))
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .default_value(["Alpha"])
                    .default_input_value("mismatch")
                    .default_open(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_input_change(move |value, _, _| inputs.borrow_mut().push(value.to_owned()))
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    click(cx, 60., 354.);
    flush_frame(cx);

    assert_eq!(
        combo.read_with(cx, |state, _| state.value().to_owned()),
        "Alpha"
    );
    assert_eq!(input_changes.borrow().as_slice(), ["Alpha"]);
    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
}

#[gpui::test]
fn controlled_combo_box_click_away_reports_one_close(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next;

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(300.))
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .selected_keys(["Alpha".into()])
                    .default_input_value("mismatch")
                    .is_open(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    click(cx, 60., 354.);
    flush_frame(cx);
    flush_frame(cx);

    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
}

#[gpui::test]
fn combo_box_tab_commits_the_highlight_then_moves_focus_on(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let opens = opens.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .default_open(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_change(move |value, _, _| selections.borrow_mut().push(value.to_string()))
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    press(cx, "down");
    press(cx, "tab");
    flush_frame(cx);

    assert_eq!(selected.borrow().as_slice(), ["Alpha"]);
    assert_eq!(
        combo.read_with(cx, |state, _| state.value().to_owned()),
        "Alpha"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
    assert!(cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)));
}

#[gpui::test]
fn combo_box_multiple_tab_adds_the_highlight_then_moves_focus_on(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let opens = events();
    let opened = opens.clone();
    let combo = search_state(cx);
    let next = search_state(cx);
    let combo_for_view = combo.clone();
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let opens = opens.clone();
        gpui::div()
            .child(
                ComboBox::new(combo_for_view.clone(), keyed(&["Alpha", "Beta"]))
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .menu_trigger(MenuTrigger::Manual)
                    .on_selection_change_all(move |keys, _, _| {
                        selections.borrow_mut().push(keys.join(","));
                    })
                    .on_open_change(move |open, _, _| {
                        opens.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    cx.update(|window, cx| window.focus(&combo.read(cx).focus_handle(cx)));
    flush_frame(cx);
    press(cx, "down");
    press(cx, "tab");
    flush_frame(cx);

    assert_eq!(selected.borrow().as_slice(), ["Alpha"]);
    assert_eq!(combo.read_with(cx, |state, _| state.value().to_owned()), "");
    assert_eq!(opened.borrow().as_slice(), ["open:false"]);
    assert!(cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)));
}

/// `allowsEmptyCollection` keeps the panel mounted when filtering removes
/// every row. The unmatched text remains in the field, the empty-state region
/// is inert, and an outside press still dismisses the panel.
#[gpui::test]
fn combo_box_allows_empty_collection_keeps_the_list_up_with_the_empty_state(
    cx: &mut TestAppContext,
) {
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
            .allows_empty_collection(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // The default Focus trigger opens the list; allowsEmptyCollection keeps
    // it mounted when the edit filters every row away.
    cx.simulate_input("pl");
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "pl",
        "the unmatched text must be accepted into the field"
    );

    // Nothing matches "pl", but the list stays up: the press where a row
    // would be records nothing (the empty state has no handlers) and must not
    // dismiss the panel.
    click(cx, 60., 64.);
    assert!(recorded.borrow().is_empty());
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The panel is still mounted, so an outside press dismisses it.
    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the empty list must still answer its outside-press dismissal"
    );
}

/// v3 inherits React Aria for the ComboBox (its Accessibility section links
/// the RAC docs and lists "Support for custom values"). Pinned react-stately
/// 3.49.0's `commitCustomValue` keeps the typed text and sets the selected
/// key to `null` — it does not select the text. With no selection to change,
/// `onSelectionChange` fires nothing; the slice callback spells the `null`
/// when one did.
#[gpui::test]
fn combo_box_allows_custom_value_enter_commits_the_text(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let slices = events();
    let sliced = slices.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let slices = slices.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
            .allows_custom_value(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_selection_change_all(move |keys, _, _| {
                slices.borrow_mut().push(
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

    click(cx, 60., 18.);
    cx.simulate_input("pl");
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // The closed field still commits the custom value on Enter: the text
    // stays, but no key is selected, so the single-key callback — which
    // cannot spell `null` — and the slice callback both stay silent.
    press(cx, "enter");
    assert!(
        recorded.borrow().is_empty(),
        "a committed custom value carries a null selected key, so the \
         single-key callback must not report the typed text: {:?}",
        recorded.borrow().as_slice()
    );
    assert!(
        sliced.borrow().is_empty(),
        "with no selection to change, the null must not be reported"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the closed custom-value commit must not report another close"
    );
}

#[gpui::test]
fn combo_box_empty_filtered_collection_closes_and_reports(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        ComboBox::new(state_for_view.clone(), keyed(&["Typst", "Rust", "Go"]))
            .menu_trigger(MenuTrigger::Input)
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("t");
    cx.simulate_input("z");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "an empty filtered collection must close the list and report the transition"
    );
}

/// `validate` is a function the *component* runs — that is the whole point of
/// the prop, and the half that is behaviour. The message it returns is drawn
/// by the field (appearance, the audits' business); what a caller can observe
/// is that the validator is invoked with the current value on every edit and
/// that the field stays fully interactive under it: typing still filters,
/// rows still open and pick. The validator itself records what it was asked.
#[gpui::test]
fn combo_box_validate_runs_on_every_edit_and_leaves_the_field_interactive(cx: &mut TestAppContext) {
    let seen = events();
    let asked = seen.clone();
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let state = search_state(cx);
    let state_for_view = state.clone();

    let cx = open_host(cx, move || {
        let seen = seen.clone();
        let changes = changes.clone();
        let opens = opens.clone();
        let state = state_for_view.clone();
        ComboBox::new(state, keyed(&["Alpha", "Beta", "Gamma"]))
            .validate(move |value| {
                seen.borrow_mut().push(value.to_owned());
                if value.is_empty() {
                    None
                } else {
                    (value.chars().count() < 3).then(|| "Too short".into())
                }
            })
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // `validate` runs during the render after an edit, so each keystroke is
    // flushed before the record is read. "a" then "l" keep the query matching
    // Alpha (a query that matched nothing would close the list).
    cx.simulate_input("a");
    flush_frame(cx);
    assert!(
        asked.borrow().contains(&"a".to_owned()),
        "the first edit must run the validator with its value"
    );

    cx.simulate_input("l");
    flush_frame(cx);
    assert!(
        asked.borrow().contains(&"al".to_owned()),
        "the second edit must re-run the validator with the whole value"
    );

    // The field never locked up under validation: typing continues to filter
    // (the list is up) and the keyboard still walks and picks the matching
    // row. (The pointer is deliberately not used here: a failing `validate`
    // draws its message under the field, which pushes the list down from its
    // no-message position, and the keyboard path is independent of that
    // geometry — the point of this test is that the field answers at all.)
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Alpha"],
        "the field must stay fully interactive under a failing validator"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the pick must close the list as usual"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "Alpha",
        "the pick must still write the chosen item into the field"
    );
}

/// The caret must survive the list opening. Under an explicit Input trigger
/// the first keystroke opens the list; the proof reads the caret from the
/// state entity the test owns via the selection anchor — pressing shift+Left
/// after "typ" selects the "p" (anchor 3, cursor 2) only if the caret is
/// where the typing left it. A list that slurped or reset the caret would
/// report a different range (or none). The keyboard path then still works, so
/// Down + Enter pick the only matching row.
#[gpui::test]
fn combo_box_caret_stays_at_the_end_when_the_list_opens(cx: &mut TestAppContext) {
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
        ComboBox::new(state, keyed(&["Typst", "Rust", "Go"]))
            .menu_trigger(MenuTrigger::Input)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // Focus alone does not open an explicit Input trigger.
    assert!(opened.borrow().is_empty());

    cx.simulate_input("typ");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the first non-empty edit must open the list"
    );

    // Caret at the end of "typ" is char 3; shift+Left anchors at 3 and moves
    // the cursor to 2, so the state reports the selected "p" (2, 3).
    press(cx, "shift-left");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selection()),
        Some((2, 3)),
        "the caret must still sit at the end after the list opened — a reset \
         caret would select from position 0"
    );

    // The keyboard still reaches the list through the focused field: Down
    // highlights the only row matching "typ", Enter takes it.
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Typst"],
        "the pick through the focused field must still work"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "Typst",
        "the pick must write the chosen item into the field"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);
}

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// v3's `Select.Value` render props include `isPlaceholder` ("Whether the
/// value is a placeholder"), and the port hands it over. Recording what the
/// closure is told each frame pins the placeholder ↔ value flip without
/// reading a pixel: the placeholder state until a pick, the chosen text
/// afterwards, and still after a reopening (the selection sticks).
#[gpui::test]
fn select_value_render_props_flip_placeholder_with_a_pick(cx: &mut TestAppContext) {
    let flags = Rc::new(RefCell::new(Vec::<(bool, String)>::new()));
    let seen_flags = flags.clone();
    let picked = events();
    let changes = picked.clone();

    let cx = open_host(cx, move || {
        let flags = flags.clone();
        let changes = changes.clone();
        Select::new(
            "sel-ph",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .value_content(move |value| {
            flags
                .borrow_mut()
                .push((value.is_placeholder, value.selected_text.to_owned()));
            value.default_children
        })
        .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
        .into_any_element()
    });

    // The first render shows the placeholder state.
    assert_eq!(
        seen_flags.borrow().first(),
        Some(&(true, String::new())),
        "nothing selected must render the placeholder state"
    );

    // Pick row 2 (Gamma), centre y = 66 + 2*36 = 138.
    click(cx, 60., 18.);
    click(cx, 60., 138.);
    assert_eq!(picked.borrow().as_slice(), ["Some(2)"]);
    assert_eq!(
        seen_flags.borrow().last(),
        Some(&(false, "Gamma".to_owned())),
        "a pick must flip the value out of the placeholder state"
    );
    assert!(
        seen_flags.borrow().iter().any(|(_, text)| text == "Gamma"),
        "the chosen text must reach the value render props"
    );

    // Reopening renders the same chosen state: the uncontrolled selection
    // sticks across close and open.
    click(cx, 60., 18.);
    assert_eq!(
        seen_flags.borrow().last(),
        Some(&(false, "Gamma".to_owned())),
        "the selection must persist into a later render"
    );
}

/// `disabledKeys` options render but cannot be chosen: a pointer press on one
/// records nothing and leaves the panel open, and the arrows skip it — Down
/// from the first option lands on the third.
#[gpui::test]
fn select_disabled_rows_are_unclickable_and_not_a_stop(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        Select::new(
            "sel-dis",
            vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
                "Delta".into(),
            ],
        )
        .disabled_keys([1])
        .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
        .on_open_change(move |open, _, _| {
            opens.borrow_mut().push(format!("open:{open}"));
        })
        .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The keyboard leg runs on a freshly opened panel: the trigger holds the
    // focus (its mouse-down claims it), so Down lands on Alpha (0), the next
    // Down skips the disabled Beta and lands on Gamma (2), and Enter takes
    // the highlighted option — closing through the trigger's own click
    // listener, which gpui fires on the same keystroke. The flush renders
    // the opened panel before the keys arrive.
    flush_frame(cx);
    press(cx, "down");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(2)"],
        "the arrows must skip the disabled option"
    );
    assert_eq!(opened.borrow().as_slice(), ["open:true", "open:false"]);

    // Reopen for the pointer leg: flush the closed state so the next trigger
    // press is a genuine open.
    flush_frame(cx);
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"]
    );
    flush_frame(cx);

    // Row 1 (Beta) centres at y = 66 + 36 = 102. A press on it must record
    // nothing and must not dismiss the panel.
    click(cx, 60., 102.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(2)"],
        "a disabled option must not answer the pointer"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "a press on a disabled option must not dismiss the panel"
    );
}

/// A select whose whole collection is disabled opens but answers nothing: the
/// rows have no handlers and the arrow-key resolver has no stops, so neither
/// path can choose anything, and nothing is ever reported.
///
/// The trigger of an *open* select still closes it. v3's trigger is one press
/// that simply closes the list — React Aria's press handling never both
/// dismisses a popover and presses the element it belongs to, and its Button
/// press is pointer-bound, so nothing reopens it. The port's trigger press is
/// marked as "not outside" for the panel's outside-press dismissal, so the
/// trigger's own click owns the close and reports it exactly once.
#[gpui::test]
fn select_fully_disabled_collection_answers_nothing(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opens = opens.clone();
        Select::new("sel-all-off", vec!["Alpha".into(), "Beta".into()])
            .disabled_keys([0, 1])
            .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    // The trigger still opens: being disabled is a property of the options.
    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);
    flush_frame(cx);

    // Both rows are unclickable: y = 66 and 102.
    click(cx, 60., 66.);
    click(cx, 60., 102.);
    assert!(recorded.borrow().is_empty());

    // ..and the keyboard has no stops to walk (the trigger holds the focus).
    press(cx, "down");
    press(cx, "up");
    assert!(recorded.borrow().is_empty());

    // The trigger still closes; nothing was ever selected. (Enter is left out
    // of the keyboard leg on purpose: gpui fires a focused trigger's click on
    // Enter, which would toggle the panel on its own; this click is the
    // deterministic close.)
    click(cx, 60., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "a fully disabled collection must still close on its trigger"
    );
    assert!(
        recorded.borrow().is_empty(),
        "no interaction with an all-disabled collection may select anything"
    );
}

/// `shouldFocusWrap = false` holds at the ends: the fourth Down clamps on the
/// last option (Enter reports it, not a wrapped first) and three Ups from a
/// reopened cursor clamp on the first.
#[gpui::test]
fn select_no_wrap_holds_at_both_ends(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Select::new(
            "sel-wrap-off",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
        .into_any_element()
    });

    // The first Down after opening starts the walk; 0 -> 1 -> 2 -> 2 (clamp).
    click(cx, 60., 18.);
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(2)"],
        "down past the end must hold on the last option without wrap"
    );

    // The cursor persisted on Gamma (2); 2 -> 1 -> 0 -> 0 (clamp).
    click(cx, 60., 18.);
    press(cx, "up up up");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(2)", "Some(0)"],
        "up past the start must hold on the first option without wrap"
    );
}

/// `shouldFocusWrap = true` joins the ends: Down from the last wraps to the
/// first, Up from the first wraps to the last.
#[gpui::test]
fn select_wrap_joins_both_ends(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Select::new(
            "sel-wrap-on",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .should_focus_wrap(true)
        .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
        .into_any_element()
    });

    // Fourth Down wraps 2 -> 0.
    click(cx, 60., 18.);
    press(cx, "down down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(0)"],
        "down past the end must wrap to the first option"
    );

    // The cursor is on Alpha (0); one Up wraps to Gamma (2).
    click(cx, 60., 18.);
    press(cx, "up");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Some(0)", "Some(2)"],
        "up past the start must wrap to the last option"
    );
}

/// Pinned React Aria `usePopover`: an open Select's list also closes when the
/// *focus* leaves it, and the keystroke that moved the focus is not swallowed.
/// Tab from the trigger lands on the next control while the list dismisses:
/// `onOpenChange(false)` reports exactly once, the rows are gone, and the next
/// input owns the focus. That direct probe catches a focus-stealing closer.
#[gpui::test]
fn select_tab_to_the_next_control_closes_and_keeps_the_focus_moving(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let opens = events();
    let opened = opens.clone();
    let next = search_state(cx);
    let next_for_view = next.clone();

    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let opened_open = opens.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(160.))
            .child(
                // Two options put Alpha's row centre at y 66 and Beta's at
                // y 102, both clear of the Input the gap parks below (y 196+).
                Select::new("sel-blur-out", vec!["Alpha".into(), "Beta".into()])
                    .on_change(move |i, _, _| changes.borrow_mut().push(format!("{i:?}")))
                    .on_open_change(move |open, _, _| {
                        opened_open.borrow_mut().push(format!("open:{open}"));
                    }),
            )
            .child(Input::new(next_for_view.clone()))
            .into_any_element()
    });

    // Open by pointer, as every test above does: the mouse-down claims the
    // trigger's focus, so the focus starts inside the surface that closes on
    // its loss. The flush renders the armed, open state before any key lands.
    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);
    flush_frame(cx);

    press(cx, "tab");
    // One frame for the focus change to fire the blur closer...
    flush_frame(cx);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "Tabbing off an open select must report onOpenChange(false) exactly once"
    );
    // ...and one more for the panel to leave the tree outright, so the
    // closed-proof clicks hit-test the frame without it.
    flush_frame(cx);
    assert!(
        cx.update(|window, cx| next.read(cx).focus_handle(cx).is_focused(window)),
        "Tab must leave focus on the next control rather than return it to the Select"
    );

    // Where the rows were: two clicks must find nothing there any more.
    click(cx, 60., 66.);
    assert!(
        recorded.borrow().is_empty(),
        "a click where Alpha was must find no panel after the blur close"
    );
    click(cx, 60., 102.);
    assert!(recorded.borrow().is_empty());
}

// ---------------------------------------------------------------------------
// Drawer
// ---------------------------------------------------------------------------
//
// v3's Drawer has no size prop (see the header); what has never been driven
// is the rest of the Drawer's surface. Geometry (window 1920x1080, 384px
// desktop side width, `p-6` = 24px): the Right panel is x 1536..1920; the close trigger
// (`absolute end-4 top-4` around the 24px CloseButton) centres at (1892, 28)
// — x = 1920 - 16 - 12, y = 16 + 12; the title row (the drag surface) spans y
// 36..60; the body probe centre (1580, 86) comes from 24px padding + 12px
// handle + 24px title + `mt-2` 8px + 18px half-probe. The body is `flex_1`,
// so the footer stays at the bottom; its 40x36 probe centres at (1876, 1038).

/// Every dismissal path reports through `on_close` *and* `onOpenChange`, and
/// the close button is a path of its own. The pointer press on the close
/// trigger and the Escape key each fire both callbacks exactly once; after the
/// exit the sheet is gone, so the probe spot records nothing.
#[gpui::test]
fn drawer_close_button_reports_on_close_and_open_change(cx: &mut TestAppContext) {
    still();
    let closes = events();
    let closed = closes.clone();
    let opens = events();
    let opened = opens.clone();
    let probed = events();
    let hits = probed.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let closes = closes.clone();
        let opens = opens.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("pd-drawer-close")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .child(probe("pd-drawer-close-probe", "hit", hits))
            .on_close(move |_, _, _| closes.borrow_mut().push("close".to_owned()))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    opens.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // The close trigger answers the pointer and reports through both
    // callbacks exactly once.
    click(cx, 1892., 28.);
    assert_eq!(
        closed.borrow().as_slice(),
        ["close"],
        "the close button must fire on_close"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:false"],
        "the close button must report the dismissal through onOpenChange"
    );
    let_exit_finish(cx);
    click(cx, 1580., 86.);
    assert!(
        probed.borrow().is_empty(),
        "the sheet must be gone after the exit"
    );

    // Escape takes the same shared path.
    *open.borrow_mut() = true;
    flush_frame(cx);
    press(cx, "escape");
    assert_eq!(
        closed.borrow().as_slice(),
        ["close", "close"],
        "escape must fire on_close too"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:false", "open:false"],
        "escape must report the dismissal through onOpenChange"
    );
}

/// `isKeyboardDismissDisabled` silences Escape but nothing else: the close
/// button still dismisses, so the keyboard-disabled drawer is not a stuck
/// one.
#[gpui::test]
fn drawer_keyboard_dismiss_disabled_silences_escape_but_not_the_button(cx: &mut TestAppContext) {
    still();
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open;

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("pd-drawer-kbd")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .is_keyboard_dismiss_disabled(true)
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    opens.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    press(cx, "escape");
    assert!(
        opened.borrow().is_empty(),
        "isKeyboardDismissDisabled must silence Escape"
    );

    click(cx, 1892., 28.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:false"],
        "the close button must still dismiss a keyboard-disabled drawer"
    );
}

/// `hideCloseButton` removes the affordance — the spot where it would sit is
/// bare panel padding, so a press there records nothing — while the keyboard
/// dismissal stays intact.
#[gpui::test]
fn drawer_hide_close_button_keeps_escape_dismissal(cx: &mut TestAppContext) {
    still();
    let opens = events();
    let opened = opens.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open;

    let cx = open_host(cx, move || {
        let opens = opens.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("pd-drawer-hide-close")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .hide_close_button(true)
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    opens.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // No button: the press lands on the panel's own padding, which is inside
    // the panel, so nothing records (the panel's outside-press dismissal only
    // fires outside it).
    click(cx, 1892., 28.);
    assert!(
        opened.borrow().is_empty(),
        "with hideCloseButton the close-trigger spot must be inert"
    );

    press(cx, "escape");
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:false"],
        "escape must still dismiss a drawer without a close button"
    );
}

/// `isDismissible = false` takes the outside-press and the drag off the table
/// (the header's mouse-down that starts the drag record is not even attached)
/// while the panel itself still works — its body probe answers — and Escape,
/// the keyboard dismissal, still closes it. This is v3's Non-Dismissable
/// example: the scrim press and a 100px pull must both report nothing.
#[gpui::test]
fn drawer_not_dismissible_ignores_outside_press_and_drag(cx: &mut TestAppContext) {
    still();
    let closes = events();
    let closed = closes.clone();
    let opens = events();
    let opened = opens.clone();
    let probed = events();
    let hits = probed.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open;

    let cx = open_host(cx, move || {
        let closes = closes.clone();
        let opens = opens.clone();
        let hits = hits.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("pd-drawer-not-dismissible")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .is_dismissible(false)
            .child(probe("pd-drawer-not-dismissible-probe", "hit", hits))
            .on_close(move |_, _, _| closes.borrow_mut().push("close".to_owned()))
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    opens.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // The panel is still there and usable.
    click(cx, 1580., 86.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["hit"],
        "a non-dismissible drawer must still render its body"
    );

    // A press on the dimmed region outside the panel..
    click(cx, 100., 100.);
    // ..and a 100px pull on the title row, over the drag threshold.
    drag(cx, (1760., 48.), (1860., 48.));
    assert!(
        closed.borrow().is_empty() && opened.borrow().is_empty(),
        "neither the outside press nor the drag may dismiss a \
         non-dismissible drawer"
    );

    // Keyboard dismissal is not the backdrop; it still works.
    press(cx, "escape");
    assert_eq!(
        closed.borrow().as_slice(),
        ["close"],
        "escape must still fire on_close"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:false"],
        "escape must still dismiss a non-dismissible drawer"
    );
}

/// The footer slot renders as a real part of the sheet's column: with a title
/// and a body probe, the footer lands 20px (`mt-5`, v3's 1.25rem) after the
/// body, and the pressable it carries is reachable exactly there — proving
/// the slot exists in the layout and is not painted over by the body.
#[gpui::test]
fn drawer_footer_sits_after_the_body_and_both_answer(cx: &mut TestAppContext) {
    still();
    let probed = events();
    let hits = probed;
    let hits_for_view = hits.clone();
    let open = Rc::new(RefCell::new(true));
    let open_flag = open;

    let cx = open_host(cx, move || {
        let hits = hits_for_view.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("pd-drawer-footer")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .child(probe("pd-drawer-footer-body", "body", hits.clone()))
            .footer_child(probe("pd-drawer-footer-foot", "footer", hits))
            .into_any_element()
    });

    // Body: 24 (p-6) + 12 (handle) + 24 (title) + 8 (mt-2) + 18 (half-probe).
    click(cx, 1580., 86.);
    assert_eq!(
        hits.borrow().as_slice(),
        ["body"],
        "the body probe must be reachable"
    );

    // The body is `flex_1`, so the footer consumes the bottom 36px of the
    // panel's 24px inset: y 1020..1056. It is `justify_end`, so its 40px probe
    // consumes x 1856..1896.
    click(cx, 1876., 1038.);
    assert_eq!(
        hits.borrow().as_slice(),
        ["body", "footer"],
        "the footer probe must be reachable where the footer's own extent \
         puts it"
    );
}
