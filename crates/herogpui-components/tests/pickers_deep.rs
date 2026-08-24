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
//! - Drawer (window 1920x1080, `PANEL_EXTENT` 320px — v3's `w-80`): the Right
//!   panel is x 1600..1920, y 0..1080, `p-6` (24px). Inside it: the handle
//!   (bar 4px + `pb-2` 8px) at y 24..36, the 24px title line (the drag
//!   surface) at y 36..60, the close trigger (`absolute end-4 top-4` around
//!   the 24px CloseButton) centred at (1892, 28), the body at 24+12+24+8 = 68
//!   (probe centre (1644, 86)) and the footer after the 36px body probe plus
//!   `mt-5` (20px); the footer is `justify_end`, so its probe sits at the
//!   sheet's right edge, centre (1878, 142).
//!
//! The Drawer tests set reduced motion before the first frame (an entry slide
//! would sit at its t=0 off-window pose for the whole test otherwise) and
//! advance the clock past `EXITING_MS` before any closed-proof probe, exactly
//! as `placement.rs` does. The pickers need neither: their panels leave the
//! tree outright when closed.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    point, prelude::*, px, ElementId, Modifiers, MouseButton, TestAppContext, VisualTestContext,
};
use harness::{click, events, open_host, press, Events};
use herogpui_components::{
    Autocomplete, ComboBox, Drawer, DrawerPlacement, InputState, Select, SelectionMode,
};

/// An `InputState` entity for the search-field-backed controls, created before
/// the host opens so the test can keep its own handle to it.
fn search_state(cx: &mut TestAppContext) -> gpui::Entity<InputState> {
    cx.new(|cx| InputState::new(cx))
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
    std::env::set_var("HEROGPUI_REDUCE_MOTION", "1");
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
        Autocomplete::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
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

/// Without `allowsEmptyCollection` the same no-match query unmounts the panel
/// outright: the outside-press probe that dismissed the panel in the prop's
/// presence records nothing here, because there is no mounted panel to hold
/// the listener.
#[gpui::test]
fn autocomplete_without_empty_collection_takes_the_panel_down(cx: &mut TestAppContext) {
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
        Autocomplete::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The default: a query with no match hides the list on the spot.
    cx.simulate_input("zz");
    click(cx, 600., 300.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true"],
        "the no-match query must unmount the panel, so the outside press \
         records no dismissal"
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
        Autocomplete::new(
            state,
            vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
                "Delta".into(),
            ],
        )
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
        Autocomplete::new(
            state,
            vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
                "Delta".into(),
            ],
        )
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
            vec![
                "Alpha".into(),
                "Alpine".into(),
                "Beta".into(),
                "Bravo".into(),
                "Charlie".into(),
            ],
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
        Autocomplete::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
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

/// The clear drops the component's *owned* selection, proven through the
/// toggle in multiple mode: with {Alpha, Beta} seeded, picking Gamma reports
/// the three, and after the clear picks Alpha reports only {"Alpha"}. Were the
/// clear a no-op on the held set, the stored {Alpha, Beta, Gamma} would toggle
/// Alpha *off* and the report would be "Beta,Gamma".
#[gpui::test]
fn autocomplete_clear_empties_the_owned_selection_before_the_next_pick(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let clears = events();
    let cleared = clears.clone();
    let state = search_state(cx);
    let state_for_view = state;

    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let clears = clears.clone();
        let state = state_for_view.clone();
        Autocomplete::new(
            state,
            vec![
                "Alpha".into(),
                "Beta".into(),
                "Gamma".into(),
                "Delta".into(),
            ],
        )
        .selection_mode(SelectionMode::Multiple)
        .default_value(["Alpha", "Beta"])
        .on_clear(move |_, _| clears.borrow_mut().push("cleared".into()))
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
    assert_eq!(cleared.borrow().as_slice(), ["cleared"]);

    // Row 0 (Alpha) after the clear: {"Alpha"}. A stale held set would have
    // reported "Beta,Gamma".
    click(cx, 60., 124.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Alpha,Beta,Gamma", "Alpha"],
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
        Autocomplete::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
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
        Autocomplete::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
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

// ---------------------------------------------------------------------------
// ComboBox
// ---------------------------------------------------------------------------

/// `allowsCustomValue` — v3: *"Whether the ComboBox allows custom values not
/// in the list"*; the example adds *"You can type any animal name, even if
/// it's not in the list"*. What works in this port: the typed text is accepted
/// into the field, the list stays up with an empty state when nothing matches
/// (the port draws "Press Enter to use this value"), a press in that empty
/// region records nothing, and an outside press still dismisses. What does
/// *not* happen is the subject of the next (`#[ignore]`d) test.
#[gpui::test]
fn combo_box_allows_custom_value_keeps_the_list_up_with_the_empty_state(cx: &mut TestAppContext) {
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
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .allows_custom_value(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // The first non-empty edit opens the list under the default Input trigger.
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
/// the RAC docs and lists "Support for custom values"), and RAC commits a
/// custom value: pressing Enter with an unmatched value selects the typed text
/// and fires `onSelectionChange`. The port draws the hint "Press Enter to use
/// this value" — and the Enter handler commits the text it promises: nothing
/// is under the cursor (a no-match query has no cursor row at all), so the
/// typed value becomes the selection and the list closes.
#[gpui::test]
fn combo_box_allows_custom_value_enter_commits_the_text(cx: &mut TestAppContext) {
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
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .allows_custom_value(true)
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("pl");
    assert_eq!(opened.borrow().as_slice(), ["open:true"]);

    // The port's own hint promises this very gesture.
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["pl"],
        "Enter with an unmatched value and allowsCustomValue must commit the \
         text as the selection"
    );
    assert_eq!(
        opened.borrow().as_slice(),
        ["open:true", "open:false"],
        "the commit must close the list"
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
        ComboBox::new(state, vec!["Alpha".into(), "Beta".into(), "Gamma".into()])
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

/// The caret must survive the list opening. Under the default Input trigger
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
        ComboBox::new(state, vec!["Typst".into(), "Rust".into(), "Go".into()])
            .on_change(move |item, _, _| changes.borrow_mut().push(item.to_string()))
            .on_open_change(move |open, _, _| {
                opens.borrow_mut().push(format!("open:{open}"));
            })
            .into_any_element()
    });

    click(cx, 60., 18.);
    // Focus alone does not open the default (Input) trigger.
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

// ---------------------------------------------------------------------------
// Drawer
// ---------------------------------------------------------------------------
//
// v3's Drawer has no size prop (see the header); what has never been driven
// is the rest of the Drawer's surface. Geometry (window 1920x1080, `PANEL_EXTENT`
// 320px, `p-6` = 24px): the Right panel is x 1600..1920; the close trigger
// (`absolute end-4 top-4` around the 24px CloseButton) centres at (1892, 28)
// — x = 1920 - 16 - 12, y = 16 + 12; the title row (the drag surface) spans y
// 36..60; the body probe centre (1644, 86) comes from 24px padding + 12px
// handle + 24px title + `mt-2` 8px + 18px half-probe; the footer probe centre
// (1644, 202) adds the 36px body probe and the footer's `mt-5` (80px).

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
    click(cx, 1644., 86.);
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
    click(cx, 1644., 86.);
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
    click(cx, 1644., 86.);
    assert_eq!(
        hits.borrow().as_slice(),
        ["body"],
        "the body probe must be reachable"
    );

    // Footer: body bottom 104 + 20 (`mt-5`, this port spells v3's 1.25rem as
    // px(20)) + 18 (half-probe). The footer is `justify_end`, so its sole
    // probe sits at the sheet's right edge: 1896 - 18 = 1878.
    click(cx, 1878., 142.);
    assert_eq!(
        hits.borrow().as_slice(),
        ["body", "footer"],
        "the footer probe must be reachable where the footer's own extent \
         puts it"
    );
}
