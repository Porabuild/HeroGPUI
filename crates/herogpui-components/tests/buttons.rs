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
//! - A full-width ButtonGroup gives a stretch slot (`flex_1`) to each member
//!   that *resolves* to full width — the pinned `fullWidth ??
//!   context.fullWidth` merge — so three inheriting members span 640px each
//!   and member *i*'s centre column is 320 + 640i; a member with an explicit
//!   `full_width(false)` hugs content instead, and the seams' absolutely-
//!   positioned separators take no layout space.
//! - Link sizes to its content: the click x is the measured label width over
//!   two, and y = 10 sits inside the ~19px line box whatever the machine's
//!   font metrics come to (the line is `line_height(14 * 1.3)` plus a 1px
//!   bottom padding, per the same derivation Breadcrumbs uses).
//! - An Alert is `w_full px-4 py-3` and takes end content only as composed
//!   children (v3 removed `isClosable`): a CloseButton child hugs the
//!   content's right edge (1920-16-24..1920-16) and starts at the top padding
//!   (12px), so its centre is (1920 - 16 - 12, 12 + 12) = (1892, 24).
//! - Chip is `px-2 py-0.5` around its composed children (one 20px `text-xs`
//!   line: the base `leading-5` sets `--tw-leading`, which the re-applied
//!   `text-*` sizes consume), so a CloseButton composed as the leading child
//!   starts at x = 8 (the leading padding) and the 24px close box plus the
//!   2px vertical padding make the chip 28px tall: close centre (20, 14).
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
    point, prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, KeyDownEvent, KeyUpEvent,
    Keystroke, Modifiers, MouseButton, TestAppContext, VisualTestContext,
};
use herogpui_components::{
    util, Alert, Button, ButtonGroup, Chip, ChipLabel, CloseButton, Link, SelectionMode,
    ToggleButton, ToggleButtonGroup,
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
fn button_disabled_skips_focus_while_pending_retains_it_without_pressing(cx: &mut TestAppContext) {
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

    // From the unfocused root, Tab must skip disabled and stop on pending.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert!(
        recorded.borrow().is_empty(),
        "a focused pending button must not activate"
    );

    // The next Tab moves from pending to the live probe.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["probe"],
        "Tab after the focusable pending button must reach the probe"
    );

    // Pointer activation is suppressed for both inert states as well.
    click(cx, 960., 18.);
    click(cx, 960., 58.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["probe"],
        "disabled and pending buttons must not record pointer presses"
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

/// Pinned v3 passes group props through context as defaults: a direct child
/// inherits `isDisabled` when unset, but an explicit `isDisabled={false}` wins.
#[gpui::test]
fn button_group_child_can_override_inherited_disabled(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let inherited = presses.clone();
        let override_enabled = presses.clone();
        ButtonGroup::new()
            .full_width(true)
            .is_disabled(true)
            .button(
                Button::new("bg-inherited-disabled")
                    .label("Inherited")
                    .on_press(move |_, _, _| inherited.borrow_mut().push("inherited".into())),
            )
            .button(
                Button::new("bg-override-enabled")
                    .label("Override")
                    .is_disabled(false)
                    .on_press(move |_, _, _| {
                        override_enabled.borrow_mut().push("override".into());
                    }),
            )
            .into_any_element()
    });

    click(cx, 480., 18.);
    click(cx, 1440., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["override"],
        "an explicit child value must override the ButtonGroup default"
    );
}

/// Pinned button.tsx resolves a direct member's width as
/// `finalFullWidth = fullWidth ?? context.fullWidth`, so an explicit
/// `fullWidth={false}` must free its share of a full-width group's row: the
/// two inheriting members split the leftover and the hugging one keeps its
/// content width. The port used to hand every member an equal `flex_1` slot
/// whenever the group was full-width, which erased the child override.
#[gpui::test]
fn button_group_full_width_context_preserves_explicit_child_false(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let one = presses.clone();
        let two = presses.clone();
        let three = presses.clone();
        ButtonGroup::new()
            .full_width(true)
            .button(
                Button::new("bgf-inherit-one")
                    .label("One")
                    .on_press(move |_, _, _| one.borrow_mut().push("one".into())),
            )
            .button(
                Button::new("bgf-explicit-false")
                    .label("Two")
                    // Explicit override: the group's fullWidth context must
                    // not stretch this member.
                    .full_width(false)
                    .on_press(move |_, _, _| two.borrow_mut().push("two".into())),
            )
            .button(
                Button::new("bgf-inherit-three")
                    .label("Three")
                    .on_press(move |_, _, _| three.borrow_mut().push("three".into())),
            )
            .into_any_element()
    });

    // "Two" hugs its content: 16px padding each side of the measured 14px
    // MEDIUM label. The group fills the 1920px window and the two inheriting
    // members split the leftover, so One spans 0..side, Two
    // side..side+w_two and Three side+w_two..1920, all at y = 18.
    let w_two = cx
        .update(|window, _| text_width(window.text_system(), "Two", 14.0, FontWeight::MEDIUM))
        + 32.;
    let side = (1920. - w_two) / 2.;

    click(cx, side / 2., 18.);
    click(cx, side - 50., 18.);
    click(cx, side + w_two / 2., 18.);
    click(cx, side + w_two + 50., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "one", "two", "three"],
        "an explicit fullWidth=false member must hug content while the \
         inheriting members share the leftover width"
    );
}

/// The override works in the other direction too: `fullWidth={true}` on one
/// member of a group that is not full-width must stretch that member across
/// the row, with the hugging members keeping their content width at the
/// edges. The old slot logic only stretched when the *group* was full-width,
/// so this member stayed content-sized in the centre.
#[gpui::test]
fn button_group_full_width_child_true_stretches_without_group_full_width(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let one = presses.clone();
        let two = presses.clone();
        let three = presses.clone();
        ButtonGroup::new()
            .button(
                Button::new("bgf-hug-one")
                    .label("One")
                    .on_press(move |_, _, _| one.borrow_mut().push("one".into())),
            )
            .button(
                Button::new("bgf-explicit-true")
                    .label("Two")
                    .full_width(true)
                    .on_press(move |_, _, _| two.borrow_mut().push("two".into())),
            )
            .button(
                Button::new("bgf-hug-three")
                    .label("Three")
                    .on_press(move |_, _, _| three.borrow_mut().push("three".into())),
            )
            .into_any_element()
    });

    // One hugs 0..w_one and Three hugs 1920-w_three..1920; the explicit
    // member's stretch slot spans everything between them, at y = 18.
    let w_one = cx
        .update(|window, _| text_width(window.text_system(), "One", 14.0, FontWeight::MEDIUM))
        + 32.;
    let w_three = cx
        .update(|window, _| text_width(window.text_system(), "Three", 14.0, FontWeight::MEDIUM))
        + 32.;

    click(cx, w_one + 60., 18.);
    click(cx, (w_one + 1920. - w_three) / 2., 18.);
    click(cx, 1920. - w_three - 60., 18.);
    click(cx, 1920. - w_three / 2., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["two", "two", "two", "three"],
        "an explicit fullWidth=true member must stretch while its hugging \
         neighbours keep their content width"
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
/// CloseButton and ToggleButton all omit it when disabled, and AGENTS.md says
/// the same: v3 gives a disabled control nothing to move to).
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
        // surface a Chip can host is a CloseButton composed as a leading
        // child: the `px-2` root starts that child at x = 8, and the 24px
        // close box plus the 2px vertical padding around the 20px label line
        // make the chip 28px tall — close centre (20, 14).
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                Chip::new()
                    .child(
                        CloseButton::new("chip-x")
                            .on_press(move |_, _, _| close.borrow_mut().push("close".into())),
                    )
                    .child(ChipLabel::new().child("Tag")),
            )
            // A plain chip below it (y 32..56) with no composed affordance.
            .child(Chip::new().child(ChipLabel::new().child("Tag")))
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
    click(cx, 24., 40.);
    click(cx, 40., 40.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "a plain chip must have nothing to press"
    );
}

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

/// v3's migration guide removes `isClosable`/`onClose`: Alert takes end
/// content only as composed children, and the close affordance is an ordinary
/// `CloseButton` child. The composed one must be both pointer-reachable — a
/// click on it records a press while a click on the alert body records
/// nothing — and keyboard-reachable: one Tab from the host root lands on it,
/// and Space activates it. A sibling alert with no composed close affordance
/// must be dead air at the spot where the removed built-in glyph used to sit.
#[gpui::test]
fn alert_composed_close_button_reports_and_alert_has_no_built_in_close(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let presses = presses.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                Alert::new("Saved")
                    .description("Your changes are live.")
                    .child(
                        CloseButton::new("alert-composed-x")
                            .on_press(move |_, _, _| presses.borrow_mut().push("close".into())),
                    ),
            )
            .child(Alert::new("No close here"))
            .into_any_element()
    });

    // The first alert is `w_full px-4 py-3`: its composed 24px CloseButton
    // hugs the content's right edge (x 1880..1904) and starts at the 12px top
    // padding, so its centre is (1892, 24). The text column's title and
    // description are two 20px lines plus the 2px gap (42px), so the alert
    // spans y 0..66 and the plain sibling starts at y 74; its would-be glyph
    // spot from the removed built-in close is (1897, 74 + 19) = (1897, 93).

    // Keyboard first, while the focus still sits on the host root: one Tab
    // reaches the composed CloseButton — the only stop inside either alert —
    // and Space activates it through the very click listener the pointer uses.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close"],
        "a CloseButton composed into an Alert must be reachable and activatable by keyboard"
    );

    // Pointer next: a click on the composed CloseButton reports the dismissal.
    click(cx, 1892., 24.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close", "close"],
        "a CloseButton composed into an Alert must report its press"
    );

    // A press on the alert body must not dismiss: it has no handler of its
    // own, and the composed close button sits over 1300px away.
    click(cx, 500., 24.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close", "close"],
        "the alert body must not report a close"
    );

    // The sibling alert composes nothing, so where Alert's removed built-in
    // close glyph used to answer there must be nothing to press.
    click(cx, 1897., 93.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["close", "close"],
        "an alert with no composed close affordance must have nothing at the removed built-in close position"
    );
}

// ---------------------------------------------------------------------------
// Render props
// ---------------------------------------------------------------------------

/// Pinned React Aria keeps a pending Button focusable while disabling its
/// hover and press interactions, and hands `isPending` to the children render
/// function independently from `isDisabled`.
#[gpui::test]
fn button_content_render_prop_reports_pending_and_retains_focus(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false, false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Button::new("btn-pending-state")
            .full_width(true)
            .is_pending(true)
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (
                    state.is_pending,
                    state.is_disabled,
                    state.is_focused,
                    state.is_hovered,
                    state.is_pressed,
                );
                gpui::div().child("pending".to_owned()).into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        *seen.borrow(),
        (true, false, false, false, false),
        "pending must be reported separately from disabled"
    );

    press(cx, "tab");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, true, false, false),
        "a pending button must retain its focus stop without becoming interactive"
    );

    let centre = point(px(960.), px(18.));
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, true, false, false),
        "pending must suppress hover and press render state"
    );
}

/// Entering an inert pending period must clear the keyed interaction slot.
/// Otherwise a pointer that leaves while handlers are detached resurfaces as
/// a stale hover as soon as pending ends.
#[gpui::test]
fn button_pending_transition_clears_stale_interaction_state(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false)));
    let record = seen.clone();
    let pending = Rc::new(RefCell::new(false));
    let for_view = pending.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Button::new("btn-pending-transition")
            .full_width(true)
            .is_pending(*for_view.borrow())
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pending);
                gpui::div().child("state".to_owned()).into_any_element()
            })
            .into_any_element()
    });

    cx.simulate_mouse_move(
        point(px(960.), px(18.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(*seen.borrow(), (true, false));

    *pending.borrow_mut() = true;
    flush_frame(cx);
    assert_eq!(*seen.borrow(), (false, true));

    cx.simulate_mouse_move(
        point(px(4.), px(500.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    *pending.borrow_mut() = false;
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false),
        "ending pending must not revive hover recorded before the inert period"
    );
}

/// A disabled native button cannot retain focus. GPUI keeps the keyed focus
/// handle alive across renders, so the render-prop state must gate that stale
/// handle when `is_disabled` changes after focus was already inside.
#[gpui::test]
fn disabling_button_clears_render_prop_focus(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false)));
    let record = seen.clone();
    let disabled = Rc::new(RefCell::new(false));
    let for_view = disabled.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Button::new("btn-disable-focus")
            .full_width(true)
            .is_disabled(*for_view.borrow())
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() =
                    (state.is_focused, state.is_focus_visible, state.is_disabled);
                gpui::div().child("state".to_owned()).into_any_element()
            })
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, false),
        "the enabled button must report keyboard focus"
    );

    *disabled.borrow_mut() = true;
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false, true),
        "disabling Button.Content must stop reporting focus"
    );

    *disabled.borrow_mut() = false;
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false, false),
        "re-enabling must not restore focus without a new user action"
    );
}

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

#[gpui::test]
fn button_content_render_prop_sees_keyboard_press(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(false));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        gpui::div()
            .child(Button::new("btn-key-state").full_width(true).content(
                move |state: util::InteractiveState| {
                    *record.borrow_mut() = state.is_pressed;
                    gpui::div().child("state".to_owned()).into_any_element()
                },
            ))
            .child(Button::new("btn-key-next").label("Next"))
            .into_any_element()
    });

    press(cx, "tab");
    cx.simulate_keystrokes("enter");
    flush_frame(cx);
    assert!(
        *seen.borrow(),
        "the frame after Enter down must report the held keyboard press"
    );

    cx.simulate_keystrokes("tab");
    flush_frame(cx);
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    flush_frame(cx);
    assert!(
        !*seen.borrow(),
        "the frame after Enter up must report the keyboard press lifted"
    );

    press(cx, "shift-tab");
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("space").unwrap(),
        is_held: true,
        prefer_character_input: false,
    });
    flush_frame(cx);
    assert!(
        !*seen.borrow(),
        "a repeated Space without its first keydown must not begin a press"
    );

    cx.simulate_keystrokes("space");
    flush_frame(cx);
    assert!(
        *seen.borrow(),
        "the frame after Space down must report the held keyboard press"
    );

    cx.simulate_mouse_up(
        point(px(4.), px(500.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert!(
        *seen.borrow(),
        "an unrelated mouse-up must not release a held keyboard press"
    );

    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    flush_frame(cx);
    assert!(
        !*seen.borrow(),
        "the frame after Space up must report the keyboard press lifted"
    );
}

#[gpui::test]
fn button_content_render_prop_tracks_interleaved_keyboard_presses(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let first = record.clone();
        let second = record.clone();
        gpui::div()
            .child(Button::new("btn-key-first").content(move |state| {
                first.borrow_mut().0 = state.is_pressed;
                gpui::div().child("first".to_owned()).into_any_element()
            }))
            .child(Button::new("btn-key-second").content(move |state| {
                second.borrow_mut().1 = state.is_pressed;
                gpui::div().child("second".to_owned()).into_any_element()
            }))
            .into_any_element()
    });

    press(cx, "tab");
    cx.simulate_keystrokes("enter");
    flush_frame(cx);
    cx.simulate_keystrokes("tab");
    flush_frame(cx);
    cx.simulate_keystrokes("space");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true),
        "moving focus may begin a second press without losing the first key's release"
    );

    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false),
        "Space up must release only the matching second press"
    );

    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("enter").unwrap(),
    });
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false),
        "Enter up must still release the first press after focus moved"
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

/// `ToggleButtonRenderProps` includes the focus and disabled fields as well as
/// the pointer and selection fields driven above. Keyboard focus must reach
/// the closure without changing selection, and the app's focus-visible flag
/// must be reflected on the following frame.
#[gpui::test]
fn toggle_button_content_render_prop_sees_focus_state(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        ToggleButton::new("tb-focus-state")
            .default_selected(true)
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (
                    state.is_focused,
                    state.is_focus_visible,
                    state.is_disabled,
                    state.is_selected,
                );
                gpui::div()
                    .w(px(64.))
                    .h(px(16.))
                    .child("bold".to_owned())
                    .into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        *seen.borrow(),
        (false, false, false, true),
        "the initial render must report an enabled, unfocused selected toggle"
    );

    press(cx, "tab");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, false, true),
        "keyboard focus and its visible flag must reach the closure without changing selection"
    );
}

/// A disabled toggle still renders its content closure, but it is neither a
/// focus stop nor a press target. The closure must receive that disabled state
/// while both pointer and keyboard activation stay inert.
#[gpui::test]
fn disabled_toggle_button_content_reports_disabled_and_stays_inert(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false)));
    let record = seen.clone();
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        let changes = changes.clone();
        ToggleButton::new("tb-disabled-state")
            .default_selected(true)
            .is_disabled(true)
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_focused, state.is_disabled, state.is_selected);
                gpui::div()
                    .w(px(64.))
                    .h(px(16.))
                    .child("bold".to_owned())
                    .into_any_element()
            })
            .on_change(move |selected, _, _| {
                changes.borrow_mut().push(format!("{selected}"));
            })
            .into_any_element()
    });

    assert_eq!(
        *seen.borrow(),
        (false, true, true),
        "the disabled state must reach the content closure"
    );

    click(cx, 48., 18.);
    press(cx, "tab enter");
    flush_frame(cx);
    assert!(
        recorded.borrow().is_empty(),
        "a disabled toggle must answer neither pointer nor keyboard activation"
    );
    assert_eq!(
        *seen.borrow(),
        (false, true, true),
        "a disabled toggle must remain unfocused and selected"
    );
}
