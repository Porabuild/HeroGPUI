//! Behaviour tests for the navigation family's keyboard and edge cases:
//! Accordion, DisclosureGroup, Breadcrumbs, Pagination and Toolbar.
//!
//! `tests/collections.rs` drives Accordion's click-toggle and its single-expand
//! mode, Pagination's next/page clicks and Breadcrumbs' last-crumb rule;
//! `tests/table_tabs_accordion.rs` covers Accordion's multiple-expand and
//! disabled keys, DisclosureGroup's click path, and the horizontal Toolbar's
//! arrow scoping (which was fixed and pinned there). Nothing here duplicates
//! those — every test drives a keyboard contract or an edge the earlier
//! suites left undriven, and every assertion is behavioural (recorded
//! callbacks, or a probe click that must record nothing), never appearance.
//!
//! ---------------------------------------------------------------------------
//! The v3 contracts, quoted from https://heroui.com/react/llms-full.txt
//! (pages read by their line ranges, September 2026 snapshot):
//!
//! * **Accordion** — the page has **no `## Accessibility` section** (its
//!   sections are Usage, Anatomy, Examples, Customization, API Reference), so
//!   the inherited behaviour is the contract: the trigger is a native button
//!   (`AccordionTrigger` renders `<Button slot="trigger">`), which means Tab
//!   reaches each trigger in turn and Enter/Space toggle it — and nothing else
//!   is bound: no arrow keys between triggers, no Home/End. The page's own
//!   "Interactive States" list only `:focus-visible` on the trigger. The API
//!   table documents `allowsMultipleExpanded` with **default `false`**, which
//!   this port defaults to `true` instead (a parity note, not a keyboard gap).
//!   v3's migration page says "content always mounted in v3"; the RAC panel is
//!   `hidden` when collapsed, so a control in a closed body is *not* reachable
//!   by Tab either way — the port unmounts the body, with the same Tab effect.
//! * **Breadcrumbs** — HAS an `## Accessibility` section: "Breadcrumbs uses
//!   React Aria Components' Breadcrumbs primitive, which provides: Proper
//!   ARIA attributes... **Keyboard navigation support**... The last breadcrumb
//!   item (without `href`) automatically becomes the current page indicator."
//!   The API table has no `maxItems`-style collapse prop — only `separator`,
//!   `isDisabled`, `children` and `render` — so the port's lack of overflow
//!   truncation matches v3 rather than losing something.
//! * **DisclosureGroup** — props: `expandedKeys`, `defaultExpandedKeys`,
//!   `onExpandedChange: (keys: Set<Key>) => void`, `allowsMultipleExpanded`
//!   (default `false` — expanding one collapses the others), `isDisabled`.
//!   The port exposes only `expanded_keys` + `on_toggle(key)`: no mode, no
//!   default seed, no disabled; it never collapses anything itself.
//! * **Pagination** — v3 is a composition (`Pagination.Link/Previous/Next/
//!   Ellipsis`); the v2 `page`/`total`/`siblings`/`boundaries` props were
//!   **removed** ("Removed (compose items manually)"). The port keeps the v2
//!   API. Its `## Accessibility` section claims: "Keyboard navigation via Tab
//!   key through all interactive elements", "Ellipsis marked with
//!   `aria-hidden` to avoid screen reader confusion", "Disabled states
//!   properly communicated to assistive technology via `isDisabled`", and the
//!   note that press handlers "from React Aria" normalise pointer *and*
//!   keyboard presses — so a disabled Previous must fire no press at all.
//! * **Toolbar** — "Inherits from React Aria Toolbar". React Aria's
//!   `useToolbar` handles exactly the orientation's axis — horizontal:
//!   ArrowRight/ArrowLeft; vertical: ArrowDown/ArrowUp — nothing else, and
//!   Tab moves out of the toolbar. This port maps both axes in both
//!   orientations (a deviation pinned by one test below).
//!
//! ---------------------------------------------------------------------------
//! Geometry, derived from the components' own constants — every number
//! carries its arithmetic in the test that uses it:
//!
//! - Accordion: a trigger is `px-4 py-4` (16px all round) around one 20px
//!   line, so 52px; items are joined by a 1px separator. An open body is
//!   `pt-2 pb-4` (2/16) around its content, so with a 36px Button inside it
//!   is 54px tall. Header 0 spans y 0..52 (centre 26); header 1 sits at
//!   53..105 (centre 79); with item 0 open and a 36px body, header 1 sits at
//!   107..159 and the open body's button spans y 107..143.
//! - DisclosureGroup: each trigger is a md `Button` (`h-9`, 36px) stretched to
//!   the group width; an open body is `p-2` (8px all round) around a 36px
//!   probe, so 52px. With both items open the probes sit at y 44..80 (centre
//!   62) and y 132..168 (centre 150).
//! - Breadcrumbs: crumb labels are *measured* with the window's own text
//!   system (`Window::text_system().shape_line`), because a click target's x
//!   depends on the label's advance width in the renderer's font. The label
//!   line is `line_height(text_size * 1.3)` = 18.2px, so its centre y is 9.1;
//!   every non-last crumb is followed by `gap-8` (8px) + a 12px chevron.
//! - Pagination: `size-md` cells are 32px squares at y 0..32 (centre y 16); a
//!   nav button is `px-2.5` (10px each side) around a 14px glyph = 34px; the
//!   row gaps items by `gap-1` (4px). Prev spans x 0..34 (centre 17), page
//!   cell *k* spans 38+36k .. 70+36k (centre 54+36k), and the next button
//!   starts at 38+36·(cell count) — 146 for three cells (centre 163), 290 for
//!   seven (centre 307), 74 for one (centre 91).
//! - Toolbar: driven entirely through the keyboard (Tab, arrows, Enter), so no
//!   geometry enters; the window's tab order is what moves.
//!
//! Each instance gets its own element id; two components sharing an id share
//! their keyed state, which AGENTS.md documents as a silent failure. The
//! `press` helper releases the last key because gpui activates a focused
//! element's click listeners on key **up** (Enter/Space only — verified in
//! gpui's `div.rs`, which maps Enter/Space and nothing else).

mod harness;

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, SharedString, TestAppContext,
    VisualTestContext,
};
use herogpui_components::{
    Accordion, AccordionItem, Breadcrumbs, Button, Crumb, DisclosureGroup, Orientation, Pagination,
    Toolbar,
};

use harness::{click, events, open_host, press, Events};

/// A div probe: a full-width, 36px-tall clickable strip recording `label` when
/// pressed. It is *not* a tab stop (no `track_focus`), so it can be used as
/// body content without inserting its own stop into a keyboard walk.
fn probe(id: &'static str, label: &'static str, recorded: Events) -> gpui::AnyElement {
    gpui::div()
        .id(id)
        .w_full()
        .h(px(36.))
        .flex()
        .items_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.to_owned()))
        .into_any_element()
}

/// Pushes the pending frame through. Keyboard navigation reads the *last
/// rendered frame*'s tab stops (`rendered_frame.tab_stops`), so a stop that
/// appears or disappears with a state change — an accordion body, a
/// disclosure body — must be painted before the next Tab can see it.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// The keys of an expanded set joined in a stable order.
///
/// A `HashSet` iterates in no particular order, so asserting on a raw join
/// would be flaky; sorting makes the recorded report deterministic.
fn sorted_join(keys: &HashSet<SharedString>) -> String {
    let mut keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
    keys.sort();
    keys.join(",")
}

/// The advance width of `text` shaped the way the components shape it: gpui's
/// default `.SystemUIFont` stack at `size` px and `weight` (copied from
/// `tests/collections.rs` — breadcrumb labels are laid out by the window's own
/// `WindowTextSystem`, so this measurement is the render's measurement).
fn text_width(system: &gpui::WindowTextSystem, text: &str, size: f32, weight: FontWeight) -> f32 {
    let run = gpui::TextRun {
        len: text.len(),
        font: Font {
            family: ".SystemUIFont".into(),
            features: FontFeatures::default(),
            weight,
            style: FontStyle::default(),
            fallbacks: None,
        },
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = system.shape_line(text.to_owned().into(), px(size), &[run], None);
    f32::from(line.width)
}

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------
//
// The v3 page has no Accessibility section; the trigger is a native button, so
// the contract is: Tab reaches each trigger in turn, Enter and Space toggle
// it, and nothing else is bound — no arrow keys between triggers, no
// Home/End. Each trigger is its own tab stop in this port, which is the
// native-button model.

/// Tab reaches each trigger, Enter and Space both toggle, and the keys v3
/// never binds — the arrows, Home and End — must do nothing while a trigger
/// holds the focus. Every trigger in this port is its own tab stop, so Tab
/// steps one trigger at a time; the assertion that the arrows and Home/End
/// record nothing is what pins the *absence* of a roving-stop keyboard,
/// which is v3's button-model contract.
#[gpui::test]
fn accordion_keyboard_toggles_and_ignores_unbound_keys(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Accordion::new(vec![
            AccordionItem::new("alpha", "Alpha"),
            AccordionItem::new("beta", "Beta"),
        ])
        .id("nvd-acc-keys")
        .on_expanded_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Tab lands on the first trigger (nothing else on the page is a stop),
    // and Enter fires its click listener on key-up. The report is the
    // expanded set: "alpha" opens, then "" closes.
    press(cx, "tab");
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(recorded.borrow().as_slice(), ["alpha"]);
    press(cx, "space");
    flush_frame(cx);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", ""],
        "Space on the focused trigger must toggle it shut again"
    );

    // With item 0 closed, the next Tab reaches the second trigger (the
    // closed body contributes no stop) and Space opens it.
    press(cx, "tab");
    press(cx, "space");
    flush_frame(cx);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "", "beta"],
        "Tab must step from the first trigger to the second, which Space opens"
    );

    // None of the keys v3's button-model never binds may do anything on a
    // focused trigger.
    press(cx, "right");
    press(cx, "left");
    press(cx, "up");
    press(cx, "down");
    press(cx, "home");
    press(cx, "end");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "", "beta"],
        "an accordion trigger is a button: the arrows and Home/End must not \
         move the focus or toggle anything"
    );
}

/// `defaultExpandedKeys` seeds the *uncontrolled* set — the accordion owns it
/// and toggles itself on press. The proof runs backwards: the first press on
/// the seeded-open trigger must report the empty set (it was open to begin
/// with), and a second press on the other trigger must report only that key.
#[gpui::test]
fn accordion_default_expanded_seed_opens_and_toggles(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Accordion::new(vec![
            AccordionItem::new("alpha", "Alpha").content(gpui::div().h(px(40.))),
            AccordionItem::new("beta", "Beta").content(gpui::div().h(px(40.))),
        ])
        .id("nvd-acc-seed")
        // v3's `defaultExpandedKeys` — no `expanded_keys`, so the accordion
        // holds the set itself.
        .default_expanded_keys(HashSet::from(["alpha".into()]))
        .on_expanded_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Header 0 spans y 0..52 (py-4 twice around a 20px line), centre y 26.
    // The seed means it is open from the first frame, so the first press
    // reports the empty set — a closed item would have reported "alpha".
    click(cx, 60., 26.);
    flush_frame(cx);
    assert_eq!(
        recorded.borrow().as_slice(),
        [""],
        "the seed must leave the item open, so the first press closes it"
    );

    // Item 0 now closed, item 1's header sits at 52 + 1 (separator) + half a
    // header = 79. Pressing it opens beta and only beta — the accordion's
    // own state, not a caller's set, answered.
    click(cx, 60., 79.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["", "beta"],
        "the open item must have been closed before the second press, whose \
         report is the set the accordion itself now holds"
    );
}

/// A focusable control inside an open item's body is part of the Tab walk;
/// inside a closed item it does not exist — the port unmounts the body. v3's
/// migration page says content is "always mounted in v3", but the react-aria
/// panel is `hidden` when collapsed, so the Tab contract is the same: reach
/// it open, skip it closed. Both halves are asserted.
#[gpui::test]
fn accordion_body_control_tab_reachable_open_skipped_closed(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Accordion::new(vec![
            AccordionItem::new("alpha", "Alpha"),
            AccordionItem::new("beta", "Beta").content(
                Button::new("nvd-acc-body-btn")
                    .label("Go")
                    .on_press(move |_, _, _| recorded.borrow_mut().push("body".to_owned())),
            ),
        ])
        .id("nvd-acc-body")
        .on_expanded_change(move |_, _, _| {})
        .into_any_element()
    });

    // Both closed: header 1 spans y 53..105 (52px + 1px separator). If item
    // beta were open, its body's button would sit at y 107..143 (52 + 1 +
    // pt-2 + 36) — a click there must record nothing while closed.
    click(cx, 60., 125.);
    assert!(
        recorded.borrow().is_empty(),
        "a closed item's body must not exist: where its button would be, a \
         click records nothing"
    );

    // Tab reaches trigger 0, then trigger 1 directly — the closed body
    // contributes no stop. Enter opens beta, reporting through the accordion.
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    flush_frame(cx);

    // The body is mounted now, so its Button is the next stop after the
    // trigger: Tab from the trigger reaches it and Enter fires *it*.
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["body"],
        "an open item's body control must join the Tab walk after its trigger"
    );

    // And the same seat answers the pointer.
    click(cx, 60., 125.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["body", "body"],
        "the open body's button must answer a click at its laid-out place"
    );
}

/// v3's API table documents `allowsMultipleExpanded` with default `false` —
/// expanding one item collapses the others — but this port defaults it to
/// `true` (a parity note, not a keyboard gap). Both halves of the flipped
/// default are asserted in one host: a plain accordion (prop unset)
/// collapses the first panel when the second opens, while
/// `.allows_multiple_expanded(true)` reports both keys at once. The
/// accordions are stacked 100px apart so each header's seat is a constant,
/// and they use distinct item keys — two instances with the same keys share
/// gpui's component-namespaced keyed state (the silent collision AGENTS.md
/// warns about), which would make the second one inert without proving
/// anything about the default.
#[gpui::test]
fn accordion_default_single_expand_and_opt_in_multiple(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        let items = || {
            vec![
                AccordionItem::new("one", "Item one").content(gpui::div().h(px(40.))),
                AccordionItem::new("two", "Item two").content(gpui::div().h(px(40.))),
            ]
        };
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                // Prop unset: the v3 default, single-expand.
                Accordion::new(items())
                    .id("nvd-acc-single-default")
                    .on_expanded_change(move |keys, _, _| {
                        recorded.borrow_mut().push(sorted_join(keys));
                    }),
            )
            .child({
                let recorded = for_view.clone();
                Accordion::new(vec![
                    AccordionItem::new("three", "Item three").content(gpui::div().h(px(40.))),
                    AccordionItem::new("four", "Item four").content(gpui::div().h(px(40.))),
                ])
                .id("nvd-acc-multi-optin")
                .allows_multiple_expanded(true)
                .on_expanded_change(move |keys, _, _| {
                    recorded.borrow_mut().push(sorted_join(keys));
                })
            })
            .into_any_element()
    });

    // Row 0 starts at the origin: header 0 centre y 26; with item one open
    // its 58px body (pt-2 + 40 + pb-4) pushes header 1 down to centre 137.
    // The second press must report exactly "two" — the first panel collapsed.
    click(cx, 60., 26.);
    flush_frame(cx);
    click(cx, 60., 137.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "two"],
        "with the prop unset (v3's default false) opening the second item \
         must collapse the first"
    );

    // Row 1 starts at y 263 (row 0 ends at 163 + the 100px gap): header 0
    // centre 289, and with item three open header 1 centre 400. The same two
    // presses report both keys this time — the opt-in multiple mode
    // (sorted_join puts "four" before "three").
    click(cx, 60., 289.);
    flush_frame(cx);
    click(cx, 60., 400.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "two", "three", "four,three"],
        ".allows_multiple_expanded(true) must keep both panels open"
    );
}

/// Two `Accordion`s with *identical* item keys on one page collide: gpui's
/// `RenderOnce` prepaint namespaces keyed state by the component's type
/// name, not by the instance's id, so keys derived from the item keys alone
/// (`acc-one`, `acc-one-focus`) resolve to the same slot in both instances
/// and the second one answers no clicks. The fix keys every slot by the
/// accordion's own id first; these two instances carry distinct ids but
/// identical item keys, so any residual sharing shows up here. Clicking the
/// second's identically-keyed header must toggle only the second — recorded
/// in its own recorder, from its own set — and leave the first, whose panel
/// stays open throughout, completely untouched.
#[gpui::test]
fn accordion_identical_item_keys_stay_independent_per_instance(cx: &mut TestAppContext) {
    let first = events();
    let second = events();
    let probed = events();
    let for_first = first.clone();
    let for_second = second.clone();
    let for_probed = probed.clone();
    let cx = open_host(cx, move || {
        let probed = for_probed.clone();
        let items = move || {
            // The probe is the open body's seat: a click there later only
            // records while item "one" is still open.
            let probed = probed.clone();
            vec![
                AccordionItem::new("one", "Item one").content(probe(
                    "nvd-acc-dup-1-body",
                    "one-open",
                    probed,
                )),
                AccordionItem::new("two", "Item two").content(gpui::div().h(px(40.))),
            ]
        };
        // Distinct ids, identical item keys — the collision AGENTS.md
        // documents for TagGroup, here triggered by caller-supplied keys.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child({
                let first = for_first.clone();
                Accordion::new(items())
                    .id("nvd-acc-dup-1")
                    .on_expanded_change(move |keys, _, _| {
                        first.borrow_mut().push(sorted_join(keys));
                    })
            })
            .child({
                let second = for_second.clone();
                Accordion::new(items())
                    .id("nvd-acc-dup-2")
                    .on_expanded_change(move |keys, _, _| {
                        second.borrow_mut().push(sorted_join(keys));
                    })
            })
            .into_any_element()
    });

    // Row 0: header 0 spans y 0..52 (centre 26). Opening item "one" must be
    // recorded by the first accordion only.
    click(cx, 60., 26.);
    flush_frame(cx);
    assert_eq!(first.borrow().as_slice(), ["one"]);
    assert!(
        second.borrow().is_empty(),
        "the second accordion's recorder must stay silent when the first is \
         clicked — got {:?}",
        second.borrow()
    );

    // Row 1 starts at y 260 (row 0 ends at 160 with the probe body + the
    // 100px gap): header 0 centre 286. The identically-keyed header must
    // answer the click and open the *second* accordion's own item, echoing
    // nothing into the first's recorder.
    click(cx, 60., 286.);
    flush_frame(cx);
    assert_eq!(
        second.borrow().as_slice(),
        ["one"],
        "the second accordion's identically-keyed header must answer the \
         click and open its own item"
    );
    assert_eq!(
        first.borrow().as_slice(),
        ["one"],
        "opening the second accordion must leave the first's set untouched"
    );

    // A second press on the same seat closes it again — the second accordion
    // answers the pointer on its own, start to finish.
    click(cx, 60., 286.);
    flush_frame(cx);
    assert_eq!(
        second.borrow().as_slice(),
        ["one", ""],
        "the second accordion must close its own item on the next press"
    );
    assert_eq!(
        first.borrow().as_slice(),
        ["one"],
        "closing the second accordion's item must not close the first's"
    );

    // And the first is not just silent, it is *open*: with item "one" closed
    // in the second, the first still renders its body probe at y 55..91
    // (pt-2 + a 36px probe inside the 54px body), centre 73.
    click(cx, 60., 73.);
    assert_eq!(
        probed.borrow().as_slice(),
        ["one-open"],
        "the first accordion's item must still be open after the second \
         accordion's whole open/close cycle"
    );
    assert_eq!(
        first.borrow().as_slice(),
        ["one"],
        "the probe press must not have toggled the first accordion itself"
    );
}

// ---------------------------------------------------------------------------
// DisclosureGroup
// ---------------------------------------------------------------------------
//
// v3 documents `expandedKeys`/`defaultExpandedKeys`/`onExpandedChange`,
// `allowsMultipleExpanded` (default false: expanding one collapses the rest)
// and `isDisabled`. This port exposes only `expanded_keys` and
// `on_toggle(key)`: it has no mode prop and never collapses anything — the
// caller's set is rendered verbatim. These tests pin that actual contract
// (keyboard toggling works; both items stay open exactly when the caller's
// set says so) rather than the v3 default the port cannot express.

// `redundant_clone` falsely fires on the `toggled.clone()` below: the host
// re-renders the `Fn` content closure every frame, so the recorder must stay
// in its environment and each frame's inner closure needs a fresh copy — the
// clone is load-bearing, and "used once" only looks at one call.
#[allow(clippy::redundant_clone)]
#[gpui::test]
fn disclosure_group_keyboard_toggles_and_keeps_both_open(cx: &mut TestAppContext) {
    let toggled = events();
    let reported = toggled.clone();
    let pressed = events();
    let probes = pressed.clone();
    let held: Rc<RefCell<HashSet<SharedString>>> = Rc::new(RefCell::new(HashSet::new()));
    let held_for_view = held.clone();
    let cx = open_host(cx, move || {
        let toggled = toggled.clone();
        let pressed = pressed.clone();
        let held_view = held_for_view.clone();
        let set = held_view.borrow().clone();
        DisclosureGroup::new()
            // The caller owns the set (the port is controlled); it *inserts*
            // every toggled key, which is the only way two items could ever
            // be open at once — the component itself never collapses.
            .expanded_keys(set)
            .item(
                "dga",
                "Alpha",
                probe("nvd-dg-probe-a", "A-body", pressed.clone()),
            )
            .item(
                "dgb",
                "Beta",
                probe("nvd-dg-probe-b", "B-body", pressed),
            )
            .on_toggle(move |key, window, _| {
                toggled.borrow_mut().push(key.to_string());
                held_view.borrow_mut().insert(key.clone());
                window.refresh();
            })
            .into_any_element()
    });

    // Each trigger is a md Button (36px, a tab stop) stretched to the group
    // width. Tab lands on the first and Space toggles it; the second
    // trigger's seat is pushed down by the first item's open body.
    press(cx, "tab");
    press(cx, "space");
    flush_frame(cx);
    assert_eq!(reported.borrow().as_slice(), ["dga"]);
    press(cx, "tab");
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        reported.borrow().as_slice(),
        ["dga", "dgb"],
        "Tab must step from the first disclosure's trigger to the second's, \
         which Enter toggles"
    );

    // With both items open the probes' seats are exact: trigger A 0..36, its
    // p-2 body 36..88 with the probe at 44..80 (centre 62); trigger B
    // 88..124, its body 124..176 with the probe at 132..168 (centre 150).
    // Both answering in one frame proves the group kept both expanded — the
    // port renders its caller's set and collapses nothing itself.
    click(cx, 60., 62.);
    click(cx, 60., 150.);
    assert_eq!(
        probes.borrow().as_slice(),
        ["A-body", "B-body"],
        "expanding the second item must not collapse the first: both open \
         bodies' controls answer, which is exactly what the caller's set says"
    );
}

// ---------------------------------------------------------------------------
// Breadcrumbs
// ---------------------------------------------------------------------------
//
// v3's API table documents no overflow/collapse prop (no `maxItems`-style
// truncation anywhere on the page), so the port's always-visible, wrapping
// row matches v3 rather than losing behaviour. The last-crumb rule is pinned
// in `collections.rs`; here: a disabled breadcrumb answers no press, and the
// Accessibility section's "keyboard navigation support" claim is checked
// against a port whose crumbs carry no focus handles at all.

#[gpui::test]
fn breadcrumbs_disabled_answers_no_click(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Breadcrumbs::new(vec![
            Crumb::new("Build"),
            Crumb::new("Deploy"),
            Crumb::new("Live"),
        ])
        .is_disabled(true)
        .on_navigate(move |index, crumb, _, _, _| {
            recorded
                .borrow_mut()
                .push(format!("{index}:{}", crumb.label));
        })
        .into_any_element()
    });

    // v3: `isDisabled` "disables all links". The labels are measured with the
    // window's own text system; the first label centres at (w_build/2, 9.1)
    // and the second starts at w_build + 20 (gap-8 + a 12px chevron). Neither
    // may record.
    let w_build =
        cx.update(|window, _| text_width(window.text_system(), "Build", 14.0, FontWeight::NORMAL));
    let w_deploy =
        cx.update(|window, _| text_width(window.text_system(), "Deploy", 14.0, FontWeight::NORMAL));
    click(cx, w_build / 2., 9.1);
    click(cx, w_build + 20. + w_deploy / 2., 9.1);
    assert!(
        recorded.borrow().is_empty(),
        "a disabled breadcrumb must not answer a press on any non-last crumb"
    );
}

/// v3's Breadcrumbs Accessibility section: "Breadcrumbs uses React Aria
/// Components' Breadcrumbs primitive, which provides: ... **Keyboard
/// navigation support**", and "The last breadcrumb item (without `href`)
/// automatically becomes the current page indicator." Every crumb that
/// carries an `href` is a link, so Tab reaches each in turn and Enter
/// activates it; only the last — the current page — is inert and stays out
/// of the walk. The fixture has three crumbs, two of them with `href`s, so
/// two Tab+Enter pairs must navigate two crumbs.
#[gpui::test]
fn breadcrumbs_tab_reaches_each_link_and_enter_navigates(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Breadcrumbs::new(vec![
            Crumb::new("Build").href("#/build"),
            Crumb::new("Deploy").href("#/deploy"),
            Crumb::new("Live"),
        ])
        .on_navigate(move |index, crumb, _, _, _| {
            recorded
                .borrow_mut()
                .push(format!("{index}:{}", crumb.label));
        })
        .into_any_element()
    });

    // The documented contract: Tab reaches each href crumb in turn and Enter
    // activates it (React Aria's press on a link). The last crumb has no
    // `href` — it is the current page — so the walk stops after the second.
    press(cx, "tab");
    press(cx, "enter");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0:Build", "1:Deploy"],
        "Tab must reach each href crumb so that Enter navigates it, and the \
         last crumb, as the current page, must stay out of the walk"
    );
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------
//
// v3's Accessibility section claims Tab-key navigation through the interactive
// elements, an `aria-hidden` ellipsis, and disabled states that genuinely
// block presses ("Press events handled across mouse, touch, and keyboard
// interactions via React Aria"). The port spells the v2 API (`page`, `total`,
// `on_change`) that v3 removed; `siblings` exists as a *field* with no
// builder, so it is frozen at 1 — another write-only prop on the parity list.
// The tests below pin the edge cells: disabled arrows and ellipses must answer
// nothing, the active page remains pressable, the derived page set must be
// exactly the v3 "Controlled"-example arithmetic (siblings = boundaries = 1),
// and Tab must walk prev, cells and next.

/// At page 1 the Previous arrow is disabled (v3: `isDisabled` communicates
/// "disabled states properly" and React Aria's press never fires for them),
/// and at the last page the Next arrow is. Neither disabled arrow is a tab stop
/// or answers a pointer press; the first Tab instead reaches active page 1,
/// which remains a live link and reports itself.
#[gpui::test]
fn pagination_disabled_arrows_are_inert(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Two independents rows 100px apart so each can be hit without the
        // other: on page 1 the prev is disabled; on page 3 of 3 the next is.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                Pagination::new("nvd-pg-d1", 1, 3)
                    .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string())),
            )
            .child({
                let recorded = for_view.clone();
                Pagination::new("nvd-pg-d2", 3, 3)
                    .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            })
            .into_any_element()
    });

    // Row 0 at y 0..32 (centre 16): the first Tab must NOT land on the
    // disabled prev (a disabled control is not a stop), and neither a click
    // nor Enter on it may report. Enter therefore activates page 1.
    press(cx, "tab");
    press(cx, "enter");
    click(cx, 17., 16.);
    // Row 1 at y 132..164 (centre 148): the disabled next must not report
    // the page after the last.
    click(cx, 163., 148.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1"],
        "the disabled arrows must answer nothing while the first enabled tab \
         stop, active page 1, reports itself — got {:?}",
        recorded.borrow()
    );
}

/// `Pagination::new` clamps `total` to at least 1, so a single-page
/// pagination still renders prev, the one active page link and next. The two
/// arrows are disabled at the bounds, while pressing the active link reports
/// page 1 exactly like any other enabled Pagination.Link.
#[gpui::test]
fn pagination_single_page_has_working_tab_order(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("nvd-pg-solo", 1, 1)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // One cell: prev spans x 0..34 (centre 17), the cell 38..70 (centre 54),
    // next starts at 38+36 = 74 (centre 91); the row centres on y 16.
    click(cx, 17., 16.);
    click(cx, 91., 16.);
    click(cx, 54., 16.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1"],
        "on a single page only the active page link may answer — got {:?}",
        recorded.borrow()
    );
}

/// Total 1 represents the no-movement state with a cell that IS the current
/// page. This one pins the shape that does pass: navigation reports are
/// recorded, but the hopeless directions stay inert (nothing here is
/// disabled-by-construction, so both arrows are enabled and report what they
/// navigate to).
#[gpui::test]
fn pagination_two_pages_edge_reports(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("nvd-pg-two", 1, 2)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // Two cells: prev 0..34 (centre 17), cell 1 at 38..70 (centre 54), cell 2
    // at 74..106 (centre 90), next starts at 38+72 = 110 (centre 127); y 16.
    click(cx, 127., 16.);
    click(cx, 90., 16.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["2", "2"],
        "on a two-page feed the next arrow reports page 2, and the page-2 \
         cell (never the current page here) reports page 2 as well"
    );
}

/// v3 marks the ellipsis `aria-hidden`, but its active page remains a live
/// React Aria Button: `aria-current` styles and identifies it without disabling
/// its forwarded `onPress`. Page 5 of 10 renders 1 … 4 5 6 … 10.
#[gpui::test]
fn pagination_ellipses_are_inert_and_active_cell_reports(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("nvd-pg-el", 5, 10)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // 10 > 2*1+5, so `visible_pages` yields 1, …, 4, 5, 6, …, 10 — seven
    // cells at centres 54+36k: 54, 90, 126, 162, 198, 234, 270 (y 16).
    // Page 5 — the current page — is the cell at k = 3 (centre 162).
    click(cx, 90., 16.); // first ellipsis
    click(cx, 162., 16.); // the active page 5
    click(cx, 234., 16.); // second ellipsis
    assert_eq!(
        recorded.borrow().as_slice(),
        ["5"],
        "the ellipses must stay inert while the active page reports its page"
    );
    click(cx, 126., 16.);
    click(cx, 270., 16.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["5", "4", "10"],
        "the page cells around the ellipses must report their own pages"
    );
}

/// Which numbers appear is the port's `visible_pages` arithmetic — the v3
/// "Controlled"-example scheme (first, last, current, one sibling each side,
/// no `dotsJump`) — and it is fixed: `siblings`/`boundaries` are v2 props
/// this port's `siblings` field cannot even set (it has no builder). The
/// seats below are derived from that fixed scheme, so the test pins both the
/// arithmetic and its immobility: page 6 of 12 renders 1 … 5 6 7 … 12.
#[gpui::test]
fn pagination_middle_page_shows_the_derived_set(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("nvd-pg-set", 6, 12)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // 12 > 7, page 6: left = 5, right = 7, so the cells are 1, …, 5, 6, 7,
    // …, 12 — seven cells at centres 54+36k (54, 90, 126, 162, 198, 234,
    // 270), y 16. The next button starts at 38+36*7 = 290 (centre 307).
    click(cx, 126., 16.); // cell 2 -> "5"
    click(cx, 162., 16.); // cell 3 -> "6" (the current page remains live)
    click(cx, 198., 16.); // cell 4 -> "7"
    click(cx, 234., 16.); // second ellipsis -> nothing
    click(cx, 270., 16.); // cell 6 -> "12"
    click(cx, 307., 16.); // next -> "7"
    assert_eq!(
        recorded.borrow().as_slice(),
        ["5", "6", "7", "12", "7"],
        "the middle-page window must render exactly 1 … 5 6 7 … 12 with the \
         ellipsis inert and the next arrow reporting the following page"
    );
}

/// v3's Accessibility section: "Keyboard navigation via Tab key through all
/// interactive elements." The port gives every cell and both arrows their own
/// tab stop, so the walk is prev -> every cell -> next; Enter activates
/// whatever holds the focus. The active cell remains a live Button;
/// `aria-current` identifies it without disabling it.
#[gpui::test]
fn pagination_keyboard_reaches_arrows_and_cells(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Pagination::new("nvd-pg-keys", 2, 3)
            .on_change(move |page, _, _| recorded.borrow_mut().push(page.to_string()))
            .into_any_element()
    });

    // Tab order: prev, cell 1, cell 2 (active), cell 3, next. Each Enter
    // activates the focused element on key-up.
    press(cx, "tab");
    press(cx, "enter"); // prev -> reports page 1
    press(cx, "tab");
    press(cx, "enter"); // cell 1 -> reports page 1
    press(cx, "tab");
    press(cx, "enter"); // cell 2 (active) -> reports page 2
    press(cx, "tab");
    press(cx, "enter"); // cell 3 -> reports page 3
    press(cx, "tab");
    press(cx, "enter"); // next -> reports page 3 (2 + 1)
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "1", "2", "3", "3"],
        "Tab must walk prev, every page cell and next, activating each on Enter"
    );
}

// ---------------------------------------------------------------------------
// Toolbar
// ---------------------------------------------------------------------------
//
// "Inherits from React Aria Toolbar". React Aria's `useToolbar` moves the
// focus with the orientation's axis only — vertical: ArrowDown/ArrowUp, with
// Left/Right doing nothing — and Tab is what leaves. The horizontal scoping
// (wrap inside, Tab leaves, disabled children skipped) is pinned in
// `tests/calendars_and_more.rs`; here is the vertical axis, the perpendicular
// keys the port answers anyway, and a toolbar whose only child is disabled.

#[gpui::test]
fn toolbar_vertical_arrows_wrap_and_tab_leaves(cx: &mut TestAppContext) {
    let pressed = events();
    let recorded = pressed.clone();
    let outside_pressed = events();
    let outside = outside_pressed.clone();
    let cx = open_host(cx, move || {
        let bold = pressed.clone();
        let italic = pressed.clone();
        let underline = pressed.clone();
        let outside = outside_pressed.clone();
        // A vertical toolbar (the v3 "Vertical" example is exactly Buttons
        // stacked) with a plain button after it as the Tab-out probe.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(100.))
            .child(
                Toolbar::new()
                    .orientation(Orientation::Vertical)
                    .child(
                        Button::new("nvd-vtb-bold")
                            .label("Bold")
                            .on_press(move |_, _, _| bold.borrow_mut().push("bold".into())),
                    )
                    .child(
                        Button::new("nvd-vtb-italic")
                            .label("Italic")
                            .on_press(move |_, _, _| italic.borrow_mut().push("italic".into())),
                    )
                    .child(
                        Button::new("nvd-vtb-underline")
                            .label("Underline")
                            .on_press(move |_, _, _| {
                                underline.borrow_mut().push("underline".into());
                            }),
                    ),
            )
            .child(
                Button::new("nvd-vtb-outside")
                    .label("Outside")
                    .on_press(move |_, _, _| outside.borrow_mut().push("outside".into())),
            )
            .into_any_element()
    });

    // Tab enters on the first control; Down walks bold -> italic -> underline,
    // the third Down wraps back to bold (the window-wide walk would have
    // landed Enter on the sibling), and Up from the first wraps to the last.
    // Enter reports which control holds the focus.
    press(cx, "tab");
    press(cx, "down down down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "Down from the last control must wrap to the first, staying inside \
         the vertical toolbar"
    );
    press(cx, "up");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "underline"],
        "Up from the first control must wrap to the last"
    );
    // Tab is the way out, exactly as in the horizontal case.
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        outside.borrow().as_slice(),
        ["outside"],
        "Tab must leave the vertical toolbar for the next control in the window"
    );
}

/// React Aria's `useToolbar` handles only the orientation's axis: in vertical
/// mode ArrowRight/ArrowLeft "return early" (the source is explicit that
/// nothing else is handled). The port's handler maps *both* axes in *both*
/// orientations, so Right moves the focus in a vertical toolbar — a key the
/// inherited contract deliberately ignores.
#[gpui::test]
fn toolbar_vertical_ignores_perpendicular_arrows(cx: &mut TestAppContext) {
    let pressed = events();
    let recorded = pressed.clone();
    let cx = open_host(cx, move || {
        let bold = pressed.clone();
        let italic = pressed.clone();
        Toolbar::new()
            .orientation(Orientation::Vertical)
            .child(
                Button::new("nvd-vpa-b")
                    .label("Bold")
                    .on_press(move |_, _, _| bold.borrow_mut().push("bold".into())),
            )
            .child(
                Button::new("nvd-vpa-i")
                    .label("Italic")
                    .on_press(move |_, _, _| italic.borrow_mut().push("italic".into())),
            )
            .into_any_element()
    });

    // Right is not a vertical-toolbar key: the focus must stay on Bold and
    // Enter must fire Bold. The port steps the focus to Italic instead.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "Right must not move the focus in a vertical toolbar"
    );
}

/// A toolbar whose only child is disabled contributes nothing to the Tab
/// order: the disabled child is not a stop and has no press, and the
/// toolbar's own scope handle is a non-stop, so Tab walks past the whole
/// toolbar and every key the group would have answered does nothing.
#[gpui::test]
fn toolbar_only_child_disabled_answers_nothing(cx: &mut TestAppContext) {
    let pressed = events();
    let recorded = pressed.clone();
    let cx = open_host(cx, move || {
        let pressed = pressed.clone();
        Toolbar::new()
            .gap(px(8.))
            .child(
                Button::new("nvd-tbl-lone")
                    .label("Lone")
                    .is_disabled(true)
                    .on_press(move |_, _, _| pressed.borrow_mut().push("lone".into())),
            )
            .into_any_element()
    });

    // The disabled button (h-9, ~63px wide for this label) sits at the
    // origin: a click at (40, 18) must record nothing — `on_click` is only
    // attached when the button is interactive.
    click(cx, 40., 18.);
    // And no key reaches it: Tab walks past (the scope handle is not a
    // stop), the arrows find no stop to move to, and Enter/Space activate
    // nothing. All four directions are included in case the step-and-check
    // wrap ever lands outside the empty toolbar.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "down");
    press(cx, "left");
    press(cx, "up");
    press(cx, "enter");
    press(cx, "space");
    assert!(
        recorded.borrow().is_empty(),
        "a toolbar whose only child is disabled must answer no key and no \
         click, and must not strand the focus"
    );
}
