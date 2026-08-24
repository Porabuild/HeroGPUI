//! Behaviour tests for the per-part disabling added to Pagination:
//! `Pagination.Link.isDisabled`, `Pagination.Previous.isDisabled` and
//! `Pagination.Next.isDisabled`.
//!
//! `tests/nav_deep.rs` pins the whole-bar and bound-based disabling (the
//! arrows at page 1 / the last page, the inert active cell and ellipses, the
//! Tab walk). This suite drives the *individual* knobs: v3's part tables
//! document `isDisabled` on the Link and on the Previous/Next buttons, and a
//! monolithic port projects all three onto one builder,
//! `Pagination::disabled_keys`, keyed by the page each control navigates to:
//! a page number in `1..=total` disables that link, `0` names Previous and
//! `total + 1` names Next. Every assertion is behavioural (recorded
//! callbacks, or a probe click that must record nothing), never appearance.
//!
//! ---------------------------------------------------------------------------
//! The v3 contract, quoted from https://heroui.com/react/llms-full.txt
//! (the Pagination page's `## API Reference`, September 2026 snapshot):
//!
//! ### Pagination.Link
//!
//! | Prop         | Type                      | Default | Description                      |
//! | `isActive`   | `boolean`                 | `false` | Whether this is the current page |
//! | `isDisabled` | `boolean`                 | `false` | Whether the link is disabled     |
//! | `onPress`    | `(e: PressEvent) => void` | -       | Press handler (from React Aria)  |
//!
//! ### Pagination.Previous / Pagination.Next
//!
//! | Prop         | Type                      | Default | Description                                         |
//! | `isDisabled` | `boolean`                 | `false` | Whether the button is disabled                      |
//! | `onPress`    | `(e: PressEvent) => void` | -       | Press handler (from React Aria)                     |
//!
//! The same page's Accessibility section adds the behavioural contract this
//! suite checks: "Disabled states properly communicated to assistive
//! technology via `isDisabled`", with press handlers "from React Aria"
//! normalising pointer *and* keyboard presses — so a disabled link or button
//! must fire no press at all. AGENTS.md states the tab-order half: a
//! disabled control must leave the tab order (`track_focus` is what puts it
//! in), and it was a real defect in this very file for the arrows earlier —
//! `nav_deep.rs` now guards those; the tests below extend the rule to a
//! per-link disable.
//!
//! ---------------------------------------------------------------------------
//! Geometry, reusing `tests/nav_deep.rs`'s derivation (which comments the
//! arithmetic): `size-md` cells are 32px squares at y 0..32 (centre y 16); a
//! nav button is `px-2.5` (10px each side) around a 14px glyph = 34px; the
//! row gaps items by `gap-1` (4px). Prev spans x 0..34 (centre 17), page
//! cell *k* spans 38+36k .. 70+36k (centre 54+36k), and the next button
//! starts at 38+36·(cell count) — 218 for five cells (centre 235). Five
//! cells are rendered because `total <= 2*siblings + 5`, and every fixture
//! below uses total 5 to keep the whole window visible.
//!
//! One harness fact shapes the ordering of every test: gpui moves the focus
//! to the element a click lands on, so a pointer probe *before* a keyboard
//! walk makes the walk start from the probed control. The keyboard walk
//! therefore always comes first, and the pointer probes after it.
//!
//! Each instance gets its own element id; two components sharing an id share
//! their keyed state, which AGENTS.md documents as a silent failure. The
//! `press` helper releases the last key because gpui activates a focused
//! element's click listeners on key **up**.

mod harness;

use std::collections::HashSet;

use gpui::{prelude::*, TestAppContext};
use herogpui_components::Pagination;

use harness::{click, events, open_host, press};

/// `Pagination.Link.isDisabled`: a link whose page number is in
/// `disabled_keys` answers neither a click nor Enter, holds no tab stop, and
/// leaves its neighbours alone. Fixture: page 2 of 5 with page 4 disabled —
/// prev, all five cells (1 2 3 4 5) and next render; cell 2 is the active
/// (inert) page and cell 4 (centre x 162) is the disabled one. The Tab walk
/// must go prev -> 1 -> 2 (active, silent) -> 3 -> 5 -> next — the Enter
/// after cell 3 landing on cell 5 is what proves cell 4 is not a stop — and
/// a click at cell 4's seat must record nothing while a click on either
/// adjacent cell still reports its page.
#[gpui::test]
fn disabled_link_answers_nothing_is_skipped_and_neighbours_report(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("parts-pg-link", 2, 5)
            .disabled_keys(HashSet::from([4]))
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // Five cells at centres 54+36k: 54, 90, 126, 162 (= the disabled page 4),
    // 198; prev at 17, next at 235, all on y 16.
    press(cx, "tab");
    press(cx, "enter"); // prev -> report page 1
    press(cx, "tab");
    press(cx, "enter"); // cell 1 -> report page 1
    press(cx, "tab");
    press(cx, "enter"); // cell 2 (the active page, `aria-current`) -> nothing
    press(cx, "tab");
    press(cx, "enter"); // cell 3 -> report page 3
    press(cx, "tab");
    press(cx, "enter"); // cell 5: the disabled cell 4 must hold no tab stop
    press(cx, "tab");
    press(cx, "enter"); // next -> report page 3 (2 + 1)
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "1", "3", "5", "3"],
        "Tab must walk prev, every enabled cell and next, skipping the \
         disabled cell 4 — got {:?}",
        recorded.borrow()
    );

    // The disabled seat answers no pointer press, and its neighbours still do.
    click(cx, 126., 16.); // cell 3, adjacent on the left
    click(cx, 198., 16.); // cell 5, adjacent on the right
    click(cx, 162., 16.); // the disabled cell 4
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "1", "3", "5", "3", "3", "5"],
        "a click on the disabled cell must record nothing while the adjacent \
         cells 3 and 5 still report their own pages — got {:?}",
        recorded.borrow()
    );
}

/// `Pagination.Previous.isDisabled`: the key `0` force-disables Previous on
/// a page where the bounds would allow it (page 3 of 5). The arrow must
/// answer no click and hold no tab stop — the first stop is cell 1, so the
/// first Enter reports page 1, where a live prev would have been the first
/// stop and reported page 2. The Next arrow and the cells are untouched,
/// which is what proves the gate is per-arrow rather than wholesale.
#[gpui::test]
fn disabled_previous_answers_nothing_and_leaves_tab_order(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("parts-pg-prev", 3, 5)
            .disabled_keys(HashSet::from([0]))
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // Page 3 of 5: prev (x 0..34, centre 17) would normally report page 2.
    press(cx, "tab");
    press(cx, "enter"); // cell 1: prev holds no stop, or the Enter was page 2
    press(cx, "tab");
    press(cx, "enter"); // cell 2 -> report page 2
    press(cx, "tab");
    press(cx, "enter"); // cell 3 (the active page) -> nothing
    press(cx, "tab");
    press(cx, "enter"); // cell 4 -> report page 4
    press(cx, "tab");
    press(cx, "enter"); // cell 5 -> report page 5
    press(cx, "tab");
    press(cx, "enter"); // next -> report page 4 (3 + 1)
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "2", "4", "5", "4"],
        "the force-disabled prev must hold no tab stop (the first Enter \
         reports cell 1, not page 2) while the cells and Next still report — \
         got {:?}",
        recorded.borrow()
    );

    // Its own seat answers no pointer press; the cells and Next still do.
    click(cx, 17., 16.);
    click(cx, 162., 16.); // cell 4
    click(cx, 235., 16.); // next
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "2", "4", "5", "4", "4", "4"],
        "a click on the force-disabled prev must record nothing while cell 4 \
         and Next still report — got {:?}",
        recorded.borrow()
    );
}

/// `Pagination.Next.isDisabled`: the key `total + 1` (6 for a total of 5)
/// force-disables Next on a page where the bounds would allow it. The arrow
/// answers no click, and after cell 5 the Tab walk wraps to prev — an enter
/// there reports page 2, where a live Next would have been the next stop and
/// reported page 4. Previous and the cells are untouched.
#[gpui::test]
fn disabled_next_answers_nothing_and_leaves_tab_order(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("parts-pg-next", 3, 5)
            .disabled_keys(HashSet::from([6]))
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // Page 3 of 5: next (starts at 38+36*5 = 218, centre 235) would normally
    // report page 4.
    press(cx, "tab");
    press(cx, "enter"); // prev -> report page 2
    press(cx, "tab");
    press(cx, "enter"); // cell 1 -> report page 1
    press(cx, "tab");
    press(cx, "enter"); // cell 2 -> report page 2
    press(cx, "tab");
    press(cx, "enter"); // cell 3 (the active page) -> nothing
    press(cx, "tab");
    press(cx, "enter"); // cell 4 -> report page 4
    press(cx, "tab");
    press(cx, "enter"); // cell 5 -> report page 5
    press(cx, "tab");
    press(cx, "enter"); // next is no stop: the walk wraps to prev -> page 2
    assert_eq!(
        recorded.borrow().as_slice(),
        ["2", "1", "2", "4", "5", "2"],
        "the force-disabled next must hold no tab stop (after cell 5 the walk \
         wraps to prev instead of reporting page 4), while prev and the \
         enabled cells still report — got {:?}",
        recorded.borrow()
    );

    // Its own seat answers no pointer press; prev still does.
    click(cx, 235., 16.);
    click(cx, 17., 16.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["2", "1", "2", "4", "5", "2", "2"],
        "a click on the force-disabled next must record nothing while prev \
         still reports page 2 — got {:?}",
        recorded.borrow()
    );
}

/// The per-link gate must not regress the whole-bar disable: under
/// `is_disabled(true)` *no* cell answers a press (the click listener is
/// attached only for a link that is not disabled) and the bar contributes no
/// tab stop at all, so Tab stays put and Enter activates nothing.
#[gpui::test]
fn whole_bar_disabled_cells_answer_nothing_and_bar_holds_no_stop(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("parts-pg-bar", 1, 5)
            .is_disabled(true)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // No stop anywhere in the bar: Tab from the root moves nowhere and Enter
    // has nothing focused that owns a click listener.
    press(cx, "tab");
    press(cx, "enter");
    // Cell 2 (centre 90) is a non-active cell whose click the port used to
    // attach even under `is_disabled`; the active cell 1 (centre 54), prev
    // (17) and next (235) complete the sweep.
    click(cx, 90., 16.);
    click(cx, 54., 16.);
    click(cx, 17., 16.);
    click(cx, 235., 16.);
    assert!(
        recorded.borrow().is_empty(),
        "a wholly disabled pagination must answer no press and hold no tab \
         stop — got {:?}",
        recorded.borrow()
    );
}
