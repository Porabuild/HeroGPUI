//! Behaviour tests for the pressable components: Button, ButtonGroup,
//! CloseButton, ToggleButton, ToggleButtonGroup, Link, Chip and Alert.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests drive the controls and assert on recorded callbacks and behavioural
//! probes only — never on appearance.
//!
//! Geometry is derived from the components' own constants, not guessed:
//!
//! - The test window is 1920x1080 (the overlays suite pins that: a modal is
//!   centred at y 498 = (1080-84)/2 and a drawer spans x 1600..1920).
//! - A `Button` is `h-[Size::control_height()]` = 36px at `Size::Md`, so a
//!   row starting at the window origin is 36px tall and its centre line is
//!   y = 18. With `full_width(true)` the button spans the whole window, so the
//!   centre column is exactly x = 960.
//! - CloseButton is a square `(box_size, icon_size) = (24, 16)` (`close_button.rs`),
//!   so its centre at the origin is (12, 12); the icon's 4px padding is
//!   irrelevant to the hit box.
//! - ToggleButton `Size::Md` is `h-9` (36px) with `px-4`
//!   (16px) around a label whose advance width is *measured* with the same
//!   text system the renderer shapes with, so its centre x is 16 + w/2.
//! - A full-width ButtonGroup stretches its members with `flex_1`, so member
//!   *i* of three spans 640px and its centre column is 320 + 640i; the seams'
//!   absolutely-positioned separators take no layout space.
//! - Link sizes to its content: the click x is the measured label width over
//!   two, and y = 10 sits inside the ~19px line box whatever the machine's
//!   font metrics come to (the line is `line_height(14 * 1.3)` plus a 1px
//!   bottom padding, per the same derivation Breadcrumbs uses).
//! - A closable Alert is `w_full px-4 py-3`: its 14px close glyph hugs the
//!   content's right edge (1920-16) and starts at the top padding (12px), so
//!   its centre is (1920 - 16 - 7, 12 + 7) = (1897, 19).
//! - Chip is `px-2 py-0.5` around its content, so a CloseButton composed into
//!   its `start_content` slot starts at x = 8 (the leading padding) and its
//!   24px box is vertically centred in the 28px-tall chip: centre (20, 14).
//!
//! The one-frame hover/press lag is handled explicitly in
//! `button_content_render_prop_sees_press`: `track_interaction` hears about a
//! hover or press in a *handler* and stashes it in the keyed `Interaction`
//! slot, so only the render *after* the event can read it. The test forces
//! that frame (`window.refresh()`) after each event and reads the closure's
//! latest snapshot — never the one the event dispatched against.
//!
//! That test used to be `#[ignore]`d as a defect: the button panicked before a
//! single frame drew, because `anim::hover_fade` (the `transition-colors`
//! wrapper) and `util::track_interaction` (attached only when a `content`
//! closure is set) both called `.on_hover` on the button root, and gpui
//! refuses a second one ("calling on_hover more than once on the same element
//! is not supported"). `hover_fade` now reads the interaction slot the
//! `track_interaction` handler keeps when one exists, leaving that handler the
//! element's single hover listener — see `anim.rs`. The assertions below are
//! the contract the render-prop API keeps, and
//! `toggle_button_content_render_prop_sees_state` runs the same cycle on a
//! second component to prove the fix is not button-specific.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    point, prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, Modifiers, MouseButton,
    TestAppContext, VisualTestContext,
};
use herogpui_components::{
    util, Alert, Button, ButtonGroup, Chip, CloseButton, Link, SelectionMode, ToggleButton,
    ToggleButtonGroup,
};

use harness::{click, events, open_host, press};

/// The advance width of `text` shaped the way the components shape it: gpui's
/// default `.SystemUIFont` stack at `size` px and `weight`.
///
/// ToggleButton, Button and Link labels are 14px MEDIUM. All are laid out by
/// the window's own `WindowTextSystem`,
/// so this measurement is the render's measurement (the same helper the
/// collections suite uses for Tabs and Breadcrumbs).
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

/// Forces the frame that carries the state a handler just changed.
///
/// Every event below ends with the window dirty but not necessarily painted,
/// and events hit-test the *last rendered frame*. A explicit refresh makes the
/// frame under test the one the next event will dispatch against, which is
/// also how the hover/press one-frame lag is turned into a determinism.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

#[gpui::test]
fn button_press_reports_once(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let presses = presses.clone();
        Button::new("btn-press")
            .label("Go")
            .full_width(true)
            .on_press(move |_, _, _| presses.borrow_mut().push("press".into()))
            .into_any_element()
    });

    // The button is at the origin, `Size::Md` tall (36px), `full_width` across
    // the 1920px window: centre (960, 18).
    click(cx, 960., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["press"],
        "a click must report exactly one press"
    );

    // gpui moves the focus to a clicked element that tracks one, so the button
    // now holds it. Enter and Space activate it on key *up* by firing the very
    // same `on_click` listener (`ClickEvent::Keyboard`); a component that also
    // bound its own Enter handler in addition to `on_click` would fire twice —
    // the double-fire class this guards.
    flush_frame(cx);
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["press", "press"],
        "Enter must report exactly one more press"
    );
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["press", "press", "press"],
        "Space must report exactly one more press"
    );
}

#[gpui::test]
fn button_disabled_and_pending_do_not_press(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let disabled = presses.clone();
        let pending = presses.clone();
        let probe = presses.clone();
        // Three full-width buttons stacked with 4px gaps: disabled at
        // y 0..36 (centre 18), pending at y 40..76 (centre 58), the probe at
        // y 80..116 (centre 98).
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                Button::new("btn-disabled")
                    .label("Disabled")
                    .full_width(true)
                    .is_disabled(true)
                    .on_press(move |_, _, _| disabled.borrow_mut().push("disabled".into())),
            )
            .child(
                Button::new("btn-pending")
                    .label("Pending")
                    .full_width(true)
                    .is_pending(true)
                    .on_press(move |_, _, _| pending.borrow_mut().push("pending".into())),
            )
            .child(
                Button::new("btn-probe")
                    .label("Probe")
                    .full_width(true)
                    .on_press(move |_, _, _| probe.borrow_mut().push("probe".into())),
            )
            .into_any_element()
    });

    // A click on either inert button must reach no handler at all.
    click(cx, 960., 18.);
    click(cx, 960., 58.);
    assert!(
        recorded.borrow().is_empty(),
        "disabled and pending buttons must not record a press"
    );

    // Neither inert button is a tab stop: `track_focus` (what puts an element
    // in the order) is gated on interactivity, so one Tab must skip both and
    // land on the probe.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["probe"],
        "Tab must skip the disabled and pending buttons and reach the probe"
    );
}

// ---------------------------------------------------------------------------
// ButtonGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn button_group_reports_each_child(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let one = presses.clone();
        let two = presses.clone();
        let three = presses.clone();
        // A full-width group of three: each member's slot is flex_1, so
        // member *i* spans x 640i..640(i+1) and its centre column is
        // 320 + 640i, all at y = 18.
        ButtonGroup::new()
            .full_width(true)
            .button(
                Button::new("bg-one")
                    .label("One")
                    .on_press(move |_, _, _| one.borrow_mut().push("one".into())),
            )
            .button(
                Button::new("bg-two")
                    .label("Two")
                    .on_press(move |_, _, _| two.borrow_mut().push("two".into())),
            )
            .button(
                Button::new("bg-three")
                    .label("Three")
                    .on_press(move |_, _, _| three.borrow_mut().push("three".into())),
            )
            .into_any_element()
    });

    click(cx, 320., 18.);
    click(cx, 960., 18.);
    click(cx, 1600., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "two", "three"],
        "each member of the group must report its own press, exactly once"
    );
}

// ---------------------------------------------------------------------------
// CloseButton
// ---------------------------------------------------------------------------

#[gpui::test]
fn close_button_reports_and_disabled_is_inert(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let dead_presses = events();
    let dead_recorded = dead_presses.clone();
    let probe_presses = events();
    let probe_recorded = probe_presses.clone();
    let cx = open_host(cx, move || {
        let close = presses.clone();
        let dead = dead_presses.clone();
        let probe = probe_presses.clone();
        // `.close-button` is a 24px square (`h-6`, box_size 24). The disabled
        // one sits first at y 0..24 — Tab must step *past* it — then the
        // enabled one at y 28..52 (4px gap) and the full-width probe button at
        // y 56..92.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                CloseButton::new("cb-disabled")
                    .is_disabled(true)
                    .on_press(move |_, _, _| dead.borrow_mut().push("dead".into())),
            )
            .child(
                CloseButton::new("cb-enabled")
                    .on_press(move |_, _, _| close.borrow_mut().push("close".into())),
            )
            .child(
                Button::new("cb-probe")
                    .label("Probe")
                    .full_width(true)
                    .on_press(move |_, _, _| probe.borrow_mut().push("probe".into())),
            )
            .into_any_element()
    });

    // Centre of the enabled box at y 28..52: (12, 40).
    click(cx, 12., 40.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "the close button must report exactly one press"
    );

    // The disabled one sits first, at y 0..24: centre (12, 12) — and must be
    // inert to the pointer.
    click(cx, 12., 12.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "a disabled close button must not report a press"
    );
    assert!(
        dead_recorded.borrow().is_empty(),
        "the disabled close button's own handler must never run"
    );

    // The disabled button must also leave the tab order. The pointer pressed
    // nothing focusable, so the focus is back on the host root: the first Tab
    // reaches the enabled close button (a real tab stop), and the second must
    // skip the disabled one and land on the probe — if the disabled button
    // were a stop, the second Tab would park on it and Space would die there.
    press(cx, "tab");
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        probe_recorded.borrow().as_slice(),
        ["probe"],
        "Tab must skip the disabled close button and reach the probe"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "the enabled close button must not fire again"
    );
}

// ---------------------------------------------------------------------------
// ToggleButton & ToggleButtonGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn toggle_button_uncontrolled_toggles(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let press_changes = changes.clone();
        // `default_selected` seeds the button's *own* state; `is_selected`
        // would be the controlled prop and hand the value back to nobody —
        // the uncontrolled path is the one that toggles.
        ToggleButton::new("tb-standalone")
            .label("Bold")
            .default_selected(true)
            .on_change(move |selected, _, _| changes.borrow_mut().push(format!("{selected}")))
            .on_press(move |_, _, _| press_changes.borrow_mut().push("press".into()))
            .into_any_element()
    });

    // The toggle has `px-4` (16px) around the measured 14px label, so its
    // centre x is 16 + w/2; it is 36px tall
    // (`h-9`), centre y = 18.
    let w =
        cx.update(|window, _| text_width(window.text_system(), "Bold", 14.0, FontWeight::MEDIUM));
    let centre_x = 16. + w / 2.;

    click(cx, centre_x, 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["false", "press"],
        "the first click must report the changed selection before onPress"
    );
    click(cx, centre_x, 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["false", "press", "true", "press"],
        "the second click must select it and keep the callback order"
    );
}

#[gpui::test]
fn toggle_button_group_single_and_multiple(cx: &mut TestAppContext) {
    let single_changes = events();
    let single_seen = single_changes.clone();
    let single_state = Rc::new(RefCell::new(Vec::new()));
    let multi_changes = events();
    let multi_seen = multi_changes.clone();
    let multi_state = Rc::new(RefCell::new(Vec::new()));

    let third_state = single_state;
    let fourth_state = multi_state;
    let cx = open_host(cx, move || {
        let single_changes = single_changes.clone();
        let multi_changes = multi_changes.clone();
        // `Rc::clone` (function form, not `.clone()`): the callback closure
        // must be `'static`, so it needs its own handle to the state; the
        // render reads the same state through another handle.
        let single_state = Rc::clone(&third_state);
        let multi_state = Rc::clone(&fourth_state);
        // The group is controlled by state the test owns (there is no
        // `uncontrolled` seed for a group); the recorded callback is the
        // assertion, and the owned state lets the next frame reflect the pick.
        // Two groups stacked 320px apart so the first (36px tall) never
        // overlaps the second; member *i* of each full-width group spans
        // x 640i..640(i+1), centre 320 + 640i.
        // The cloned keys are bound to a local so the RefCell borrow ends
        // before the callback closure moves the `Rc` into itself.
        let single_selected = single_state.borrow().iter().cloned().collect::<Vec<_>>();
        let multi_selected = multi_state.borrow().iter().cloned().collect::<Vec<_>>();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(320.))
            .child(
                ToggleButtonGroup::new("buttons-single-group")
                    .full_width(true)
                    .selection_mode(SelectionMode::Single)
                    .selected_keys(single_selected)
                    .on_selection_change(move |next, _, _| {
                        *single_state.borrow_mut() = next.to_vec();
                        let joined = next
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        single_changes.borrow_mut().push(joined);
                    })
                    .child_toggle(ToggleButton::new("grp-s-b").key("bold").label("Bold"))
                    .child_toggle(ToggleButton::new("grp-s-i").key("italic").label("Italic"))
                    .child_toggle(
                        ToggleButton::new("grp-s-u")
                            .key("underline")
                            .label("Underline"),
                    ),
            )
            .child(
                ToggleButtonGroup::new("buttons-multiple-group")
                    .full_width(true)
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(multi_selected)
                    .on_selection_change(move |next, _, _| {
                        *multi_state.borrow_mut() = next.to_vec();
                        let joined = next
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        multi_changes.borrow_mut().push(joined);
                    })
                    .child_toggle(ToggleButton::new("grp-m-b").key("bold").label("Bold"))
                    .child_toggle(ToggleButton::new("grp-m-i").key("italic").label("Italic"))
                    .child_toggle(
                        ToggleButton::new("grp-m-u")
                            .key("underline")
                            .label("Underline"),
                    ),
            )
            .into_any_element()
    });

    // Single (first group, y = 18): Bold -> Italic (replaces) -> Italic
    // (clears, `disallowEmptySelection` is off) -> Bold.
    click(cx, 320., 18.);
    assert_eq!(single_seen.borrow().as_slice(), ["bold"]);
    click(cx, 960., 18.);
    assert_eq!(single_seen.borrow().as_slice(), ["bold", "italic"]);
    click(cx, 960., 18.);
    assert_eq!(single_seen.borrow().as_slice(), ["bold", "italic", ""]);
    click(cx, 320., 18.);
    assert_eq!(
        single_seen.borrow().as_slice(),
        ["bold", "italic", "", "bold"],
        "single mode must replace the pick and clear on re-click"
    );

    // Multiple (second group starts at 36 + 320, so y = 374): Bold -> Bold,
    // Italic (accumulates) -> Bold (toggles off).
    click(cx, 320., 374.);
    assert_eq!(multi_seen.borrow().as_slice(), ["bold"]);
    click(cx, 960., 374.);
    assert_eq!(multi_seen.borrow().as_slice(), ["bold", "bold,italic"]);
    click(cx, 320., 374.);
    assert_eq!(
        multi_seen.borrow().as_slice(),
        ["bold", "bold,italic", "italic"],
        "multiple mode must accumulate picks and toggle one off"
    );
}

// ---------------------------------------------------------------------------
// Link
// ---------------------------------------------------------------------------

#[gpui::test]
fn link_press_reports_and_disabled_is_inert(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let go = presses.clone();
        let dead = presses.clone();
        // Links size to their content; the measured label sets the click x.
        // The row: enabled link at x 0..w1, 8px gap, disabled at
        // x w1+8 .. w1+8+w2. A link's box is one 14px line (~19px) plus a
        // 1px bottom padding, so y = 10 is inside it.
        gpui::div()
            .flex()
            .flex_row()
            .gap(px(8.))
            .child(
                Link::new("lnk-go")
                    .label("Home")
                    .on_press(move |_, _, _| go.borrow_mut().push("go".into())),
            )
            .child(
                Link::new("lnk-dead")
                    .label("Disabled")
                    .is_disabled(true)
                    .on_press(move |_, _, _| dead.borrow_mut().push("dead".into())),
            )
            .into_any_element()
    });

    let w1 =
        cx.update(|window, _| text_width(window.text_system(), "Home", 14.0, FontWeight::MEDIUM));
    let w2 = cx
        .update(|window, _| text_width(window.text_system(), "Disabled", 14.0, FontWeight::MEDIUM));

    click(cx, w1 / 2., 10.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["go"],
        "an enabled link must report exactly one press"
    );
    click(cx, w1 + 8. + w2 / 2., 10.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["go"],
        "a disabled link must not report a press"
    );
}

/// A disabled Link must leave the tab order, like every other disabled control
/// in this port (`track_focus` is what puts an element in the order; Button,
/// CloseButton and ToggleButton all gate it on interactivity, and AGENTS.md
/// says the same: v3 gives a disabled control nothing to move to).
///
/// Link used to register its focus handle and ring it unconditionally, so Tab
/// landed on the dead control and Space did nothing — which reads exactly like
/// the broken keyboard AGENTS.md warns about. `track_focus` and
/// `ring_if_focused` are now gated on interactivity like every other
/// pressable's, and this test pins the behaviour: one Tab skips the disabled
/// link and lands on the live probe behind it.
#[gpui::test]
fn disabled_link_leaves_tab_order(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let probe = presses.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                Link::new("lnk-skip")
                    .label("Disabled")
                    .is_disabled(true)
                    .on_press(|_, _, _| {}),
            )
            .child(
                Button::new("lnk-probe")
                    .label("Probe")
                    .full_width(true)
                    .on_press(move |_, _, _| probe.borrow_mut().push("probe".into())),
            )
            .into_any_element()
    });

    // One Tab from the root must skip the disabled link entirely and land on
    // the probe; Space then activates it.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["probe"],
        "a disabled link must not be a tab stop"
    );
}

// ---------------------------------------------------------------------------
// Chip
// ---------------------------------------------------------------------------

#[gpui::test]
fn chip_close_reports_and_plain_chip_has_nothing_to_press(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let close = presses.clone();
        // v3's Chip has no close affordance at all — its API table documents
        // children/className/color/variant/size and nothing else, and this
        // port matches (the port's own docs route removable chips to TagGroup,
        // whose on_remove is driven in collections.rs). The closest pressable
        // surface a Chip can host is a CloseButton composed into its one child
        // slot (`start_content`): the `px-2` chip starts that slot at x = 8,
        // and the 24px close box is vertically centred in the 28px-tall chip,
        // so its centre is (20, 14).
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                Chip::new("Tag")
                    .start_content(
                        CloseButton::new("chip-x")
                            .on_press(move |_, _, _| close.borrow_mut().push("close".into())),
                    ),
            )
            // A plain chip below it (y 32..56) with no composed affordance.
            .child(Chip::new("Tag"))
            .into_any_element()
    });

    // The composed close button reports exactly once.
    click(cx, 20., 14.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "a close affordance composed into a chip must report its press"
    );

    // The plain chip has no press surface anywhere: neither the centre nor
    // the trailing slot where a close button would sit records anything (it
    // has no callback and takes no child slot, so a click cannot even reach
    // one — the probe proves nothing registered).
    click(cx, 24., 44.);
    click(cx, 40., 44.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "a plain chip must have nothing to press"
    );
}

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

#[gpui::test]
fn alert_close_reports(cx: &mut TestAppContext) {
    let closes = events();
    let recorded = closes.clone();
    let cx = open_host(cx, move || {
        let closes = closes.clone();
        Alert::new("Saved")
            .is_closable(move |_, _, _| closes.borrow_mut().push("close".into()))
            .into_any_element()
    });

    // The alert is `w_full px-4 py-3`; its close glyph is the 14px svg in the
    // rightmost content position: right edge 1920-16, centre x 1920-16-7,
    // and it starts at the 12px top padding, so centre y = 12+7.
    click(cx, 1897., 19.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "the alert's close affordance must report the dismissal"
    );

    // A press on the alert body must not dismiss: it has no handler of its
    // own, and the close button sits 1380px away.
    click(cx, 500., 19.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "the alert body must not report a close"
    );
}

// ---------------------------------------------------------------------------
// Render props
// ---------------------------------------------------------------------------

#[gpui::test]
fn button_content_render_prop_sees_press(cx: &mut TestAppContext) {
    // v3 hands a button's children a function and passes in
    // `{isHovered, isPressed, isFocused, isFocusVisible}`. This port computes
    // them and hands them over as `util::InteractiveState` — but gpui reports
    // a hover and a press to a *handler*, so the render can only read what the
    // last frame recorded: both values are one frame behind the pointer. The
    // test therefore forces a frame after each event (`flush_frame`) and reads
    // the closure's latest snapshot, which is the frame *after* the one the
    // event dispatched against.
    let seen = Rc::new(RefCell::new((false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Button::new("btn-state")
            .full_width(true)
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pressed);
                gpui::div().child("state".to_owned()).into_any_element()
            })
            .into_any_element()
    });

    let centre = point(px(960.), px(18.));

    // The first render drew before any pointer event: nothing hovered, nothing
    // pressed.
    assert_eq!(*seen.borrow(), (false, false), "initial state must be idle");

    // Move the pointer onto the button: `on_hover` stashes hover=true, and the
    // forced frame hands it to the closure.
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false),
        "the frame after the move must see the hover"
    );

    // Press down: `on_mouse_down` stashes pressed=true; no mouse-up has run,
    // so the frame after the down sees hover and press together.
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true),
        "the frame after the down must see the press"
    );

    // Release: `on_mouse_up` clears the press; the hover survives.
    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false),
        "the frame after the up must see the press lifted"
    );

    // Leave the button entirely: the hover clears too.
    cx.simulate_mouse_move(
        point(px(4.), px(500.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false),
        "the frame after leaving must see the hover lifted"
    );
}

/// A second component's render prop must survive a full pointer cycle too, so
/// the single-hover-listener fix is not button-specific. `ToggleButton` takes
/// the same `content` closure and hands `isSelected` over with the rest of
/// `util::InteractiveState`, so this drives the same hover/press sequence as
/// the Button test and additionally asserts the toggle bit flips with a click.
#[gpui::test]
fn toggle_button_content_render_prop_sees_state(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        // `default_selected(true)` seeds the button's *own* state, so the first
        // click flips the closure's `is_selected` from true to false (the
        // uncontrolled path — `is_selected(true)` would hand the value to
        // nobody and the toggle would be inert).
        ToggleButton::new("tb-state")
            .default_selected(true)
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pressed, state.is_selected);
                // A fixed 64x16 box stands in for the label a caller's render
                // function would draw. ToggleButton `Size::Md` is 36px tall
                // (`h-9`) with `px-4` (16px) each side, so the centre column is
                // 16 + 32 = 48 and the centre row is 18.
                gpui::div()
                    .w(px(64.))
                    .h(px(16.))
                    .child("bold".to_owned())
                    .into_any_element()
            })
            .into_any_element()
    });

    let centre = point(px(48.), px(18.));

    // The first render drew before any pointer event, and `default_selected`
    // seeded the selection: idle and selected.
    assert_eq!(
        *seen.borrow(),
        (false, false, true),
        "initial state must be idle and selected"
    );

    // Move the pointer onto the toggle: the interaction slot hears the hover,
    // and the forced frame hands it to the closure.
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, true),
        "the frame after the move must see the hover"
    );

    // Press down: the press lands alongside the hover; no mouse-up has run.
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, true),
        "the frame after the down must see the press"
    );

    // Release: the press lifts, and the click the up completes toggles the
    // button's own state off — the frame after the up reports all three.
    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, false),
        "the up must lift the press and toggle the selection off"
    );

    // Leave the toggle entirely: the hover clears too.
    cx.simulate_mouse_move(
        point(px(4.), px(500.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false, false),
        "the frame after leaving must see the hover lifted"
    );
}
