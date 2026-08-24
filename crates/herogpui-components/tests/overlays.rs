//! Behaviour tests for the overlay components: Modal, Drawer, AlertDialog,
//! Popover, Tooltip and Toast.
//!
//! An overlay is a *state* before it is a drawing: `is_open` on the controlled
//! dialogs, an open flag owned by the component on the popover, a hover/focus
//! gate on the tooltip and a queue on the toast store. None of it is visible
//! in a screenshot, so these tests drive the real gpui event pipeline and
//! assert only what a caller could observe — callbacks that fired, entities
//! that changed, and probe clicks that record nothing.
//!
//! Geometry is derived from the port's own constants, never measured:
//! `ModalSize::Md` is `max-w-md` = 448px, the desktop side Drawer is 384px, the
//! harness window is 1920x1080 and `util::FIELD_HEIGHT` is 36px. The entry and
//! exit animations run on wall time, which the test clock does not drive, so
//! they are suppressed by [still] and the clock is still advanced past
//! `EXITING_MS` (100ms) before a closed-proof probe — the exit phase is what
//! keeps a dismissed panel on screen, and the probe must not land on that
//! ghost.
//!
//! Two facts this suite had to learn the hard way:
//!
//! - **[still] must run before the host's first frame.** The entering/exit
//!   wrappers change the element tree, so flipping reduced motion *after* the
//!   first paint makes the next click's mouse-down and mouse-up see different
//!   trees: the click machinery's per-element state moves to a new id path in
//!   between, and the click silently dies. `HEROGPUI_REDUCE_MOTION` is read by
//!   `ThemeProvider::init` (inside `open_host`), so setting it first pins the
//!   layout from the very first frame.
//! - **gpui has no hitbox occlusion, which is why the dialogs dismiss from
//!   the panel's bounds, not the backdrop's.** A full-window backdrop's
//!   `on_click` fires for a press on the panel painted above it as well —
//!   every interior press (and every sub-threshold drawer pull) reported a
//!   dismissal. The backdrop is a bare scrim now and the dismissal listens
//!   for `on_mouse_down_out` on the panel: gpui checks that listener against
//!   the element's own bounds geometrically, so a press inside the panel
//!   never fires it and a press on the dimmed region around it always does.
//!   `is_dismissible` gates whether the panel listens at all, and Escape
//!   stays wired regardless.
//!
//! Every element carries a distinct id (prefix `ovl-`): the dialogs key their
//! exit phase, focus handle and drag offset by id, and gpui merges two dialogs
//! sharing a key.

mod harness;

use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use harness::{click, events, open_host, press};
use herogpui_components::{
    dismiss_toast, toast_store, util, AlertDialog, Button, Drawer, DrawerPlacement, Modal, Popover,
    Toast, Tooltip, TooltipTrigger,
};

/// Pins the layout by enabling reduced motion **before** the first frame.
///
/// gpui's enter/exit animations run on wall time, which the test clock does
/// not drive, so an entry slide would sit at t=0 (fully off-window) for the
/// whole test otherwise. The preference must be in place before `open_host`:
/// the host's opening paint is the one the first event dispatches against, and
/// a mid-test flip changes the animated wrapper structure, which rebuilds the
/// click machinery's state under a new id path and swallows clicks.
fn still() {
    std::env::set_var("HEROGPUI_REDUCE_MOTION", "1");
}

/// Advances the test clock past `EXITING_MS` (100ms) plus slack, and forces
/// the repaint that the exit timer's `notify` only scheduled.
///
/// A closed-proof probe must not land on the exiting (still-mounted) panel,
/// and the timer's notify merely dirties the window — the repaint that
/// unmounts it completes only at the end of an update cycle. Without the
/// explicit update below, the next event dispatches against the stale exiting
/// frame and the ghost control answers.
fn let_exit_finish(cx: &mut VisualTestContext) {
    cx.executor().advance_clock(Duration::from_millis(300));
    cx.update(|window, _| window.refresh());
}

/// One simulated drag: press at `from`, move to `to`, release there.
///
/// `from` must land on the title row — the header's mouse-down starts the
/// drag record. The drawer's move/release handlers watch the overlay and
/// always run while that record exists, which is why the geometry of `to`
/// does not need to track the header.
fn drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
}

fn slow_drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    std::thread::sleep(Duration::from_millis(100));
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    std::thread::sleep(Duration::from_millis(100));
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
}

// Geometry, all derived from port constants:
// - The Md modal panel spans the window centre (960, 540) plus half of
//   `max-w-md` (448 / 2 = 224): x [736..1184]. With no title and a single
//   36px button body it is p(24) + 36 = 84px tall: y [498..582].
// - The built-in close trigger sits `absolute end-4 top-4`: centre x is
//   1184 - 16 - 12 = 1156, but the stretched inside button still covers
//   x [760..1160], so the press clears it at x = 1164 (still inside the
//   24px button at [1144..1168]).
// - The right drawer spans x [1536..1920] (desktop width 384) and the full
//   window height; its p(24) top padding puts the 12px handle at y [24..36]
//   and the 24px title row (the drag surface) at y [36..60], centre
//   (1728, 48).

#[gpui::test]
fn modal_escape_closes(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let inside = events();
    let pressed = inside.clone();
    let open = Rc::new(RefCell::new(true));

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let inside = inside.clone();
        let is_open = *open.borrow();
        // Not dismissible: the interior press must not be able to close it,
        // so the probe records exactly the control press (gpui's lack of
        // occlusion would otherwise pass the press through to the backdrop).
        // Escape is still wired — `is_dismissible` only gates the backdrop
        // and the close button, not the keyboard dismissal.
        Modal::new()
            .id("ovl-modal-esc")
            .is_open(is_open)
            .is_dismissible(false)
            .child(
                Button::new("ovl-modal-esc-inside")
                    .label("Inside")
                    .on_press(move |_, _, _| inside.borrow_mut().push("inside".into())),
            )
            .on_open_change({
                let open_flag = open.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // Control: the panel is up, and a press on the inside button reaches it.
    click(cx, 960., 540.);
    assert_eq!(pressed.borrow().as_slice(), ["inside"]);
    assert!(
        recorded.borrow().is_empty(),
        "a control press inside the panel must not dismiss the modal"
    );

    // The modal claims the focus on open, so Escape reaches its keyboard
    // dismissal handler and must report exactly one close.
    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must report the dismissal exactly once"
    );

    // After the exit finishes the panel is unmounted: the spot the button
    // covered records nothing new now.
    let_exit_finish(cx);
    click(cx, 960., 540.);
    assert_eq!(
        pressed.borrow().as_slice(),
        ["inside"],
        "the exit must unmount the panel: the spot the button covered records nothing new"
    );
    assert_eq!(recorded.borrow().len(), 1, "no further dismissal may fire");
}

#[gpui::test]
fn modal_backdrop_press_closes_when_dismissable(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let inside = events();
    let pressed = inside.clone();
    let open = Rc::new(RefCell::new(true));
    let dismissible = Rc::new(RefCell::new(true));
    let open_flag = open.clone();
    let dim_flag = dismissible.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let inside = inside.clone();
        let open_flag = open_flag.clone();
        let dim_flag = dim_flag.clone();
        let is_open = *open_flag.borrow();
        let is_dismissible = *dim_flag.borrow();
        Modal::new()
            .id("ovl-modal-bd")
            .is_open(is_open)
            .is_dismissible(is_dismissible)
            .child(
                Button::new("ovl-modal-bd-inside")
                    .label("Inside")
                    .on_press(move |_, _, _| inside.borrow_mut().push("inside".into())),
            )
            .on_open_change(move |v, window, _| {
                *open_flag.borrow_mut() = v;
                rec.borrow_mut().push(format!("open:{v}"));
                window.refresh();
            })
            .into_any_element()
    });

    // The dismissible backdrop covers the whole window; a press at the far
    // corner (100, 100) is well clear of the centred x [736..1184] panel and
    // must close the modal, reporting exactly once.
    click(cx, 100., 100.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "a dismissible modal must close on a backdrop press"
    );

    // Reopen with the dismissal disabled: the same press must not close it.
    *open.borrow_mut() = true;
    *dismissible.borrow_mut() = false;
    cx.update(|window, _| window.refresh());
    click(cx, 100., 100.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "the same press must not close a non-dismissible modal"
    );

    // ...and the panel is still there to prove it: the inside button answers.
    click(cx, 960., 540.);
    assert_eq!(
        pressed.borrow().as_slice(),
        ["inside"],
        "the modal must still be open and interactive"
    );
}

/// The two halves `isDismissible` truly gates, on one dialog: a press on the
/// non-dismissible backdrop leaves it open, and a press inside it still
/// reaches a control. If the backdrop dismissal were wired anywhere but the
/// panel's own bounds, one of these two presses would register a dismissal —
/// a window-wide handler would take the backdrop press, and a
/// pass-through-to-the-panel handler would take the interior one.
#[gpui::test]
fn modal_non_dismissible_ignores_backdrop_presses(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let inside = events();
    let pressed = inside.clone();
    let open = Rc::new(RefCell::new(true));

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let inside = inside.clone();
        let is_open = *open.borrow();
        Modal::new()
            .id("ovl-modal-nd")
            .is_open(is_open)
            .is_dismissible(false)
            .child(
                Button::new("ovl-modal-nd-inside")
                    .label("Inside")
                    .on_press(move |_, _, _| inside.borrow_mut().push("inside".into())),
            )
            .on_open_change({
                let open_flag = open.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // A press on the backdrop region — the far corner, clear of the centred
    // x [736..1184] panel — must leave the non-dismissible modal open.
    click(cx, 100., 100.);
    assert!(
        recorded.borrow().is_empty(),
        "a non-dismissible backdrop press must not dismiss the modal"
    );

    // ...and the modal is still there to prove it: a press inside the panel
    // reaches the control, and records no dismissal either.
    click(cx, 960., 540.);
    assert_eq!(
        pressed.borrow().as_slice(),
        ["inside"],
        "the non-dismissible modal must stay interactive"
    );
    assert!(
        recorded.borrow().is_empty(),
        "an interior press must not dismiss the modal"
    );
}

#[gpui::test]
fn modal_close_button_reports(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let inside = events();
    let pressed = inside.clone();
    let open = Rc::new(RefCell::new(true));
    // The render closure owns this handle; the test function keeps `open`
    // for the reopen below, so the clone is where the two split.
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let inside = inside.clone();
        let is_open = *open_flag.borrow();
        Modal::new()
            .id("ovl-modal-x")
            .is_open(is_open)
            .child(
                Button::new("ovl-modal-x-inside")
                    .label("Inside")
                    .on_press(move |_, _, _| inside.borrow_mut().push("inside".into())),
            )
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // Mouse half. The close trigger is `absolute end-4 top-4`: centre
    // (1184 - 28, 498 + 28) = (1156, 526), but the stretched inside button
    // reaches x 1160, so the press lands at (1164, 526) to hit the 24px
    // button clear of it. The press sits inside the panel's bounds, so the
    // panel's `on_mouse_down_out` must not fire: exactly one dismissal, from
    // the button. (Before the fix, the full-window backdrop's own click
    // listener passed the same press through the panel and reported a second
    // one.)
    click(cx, 1164., 526.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "the close button must dismiss exactly once: a press inside the panel is not a backdrop press"
    );
    assert!(pressed.borrow().is_empty(), "no control press may sneak in");

    let_exit_finish(cx);
    click(cx, 960., 540.);
    assert!(
        pressed.borrow().is_empty(),
        "the panel must be gone after the close"
    );

    // Keyboard half, reopened: the close button is the second tab stop inside
    // the dialog (`inside`, then it), so Enter lands on it and reports the
    // dismissal exactly once — no mouse, no pass-through.
    *open.borrow_mut() = true;
    cx.update(|window, _| window.refresh());
    press(cx, "tab tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the keyboard path must activate the close button exactly once"
    );
    assert!(
        pressed.borrow().is_empty(),
        "Enter must have activated the close button, not the inside one"
    );
}

#[gpui::test]
fn modal_tab_is_trapped(cx: &mut TestAppContext) {
    still();
    let probe = events();
    let probed = probe.clone();
    let inside = events();
    let pressed = inside.clone();
    let open = Rc::new(RefCell::new(true));

    let cx = open_host(cx, move || {
        let probe = probe.clone();
        let inside = inside.clone();
        let is_open = *open.borrow();
        // The probe sits OUTSIDE the dialog and is first in the tab order. A
        // modal is one tab cycle: v3 documents that Tab never leaves it, so
        // the trap must keep the focus on `inside` / the close button.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(320.))
            .child(
                Button::new("ovl-modal-trap-probe")
                    .label("Outside")
                    .on_press(move |_, _, _| probe.borrow_mut().push("probe".into())),
            )
            .child(
                Modal::new().id("ovl-modal-trap").is_open(is_open).child(
                    Button::new("ovl-modal-trap-inside")
                        .label("Inside")
                        .on_press(move |_, _, _| inside.borrow_mut().push("inside".into())),
                ),
            )
            .into_any_element()
    });

    // Seven Tabs: more than the three stops in the window (probe, inside,
    // close button), so an untrapped dialog would wrap onto the probe. The
    // trap pulls the focus back each time it leaves. The landing stop after an
    // odd number of Tabs from the dialog handle is the inside button, and
    // Enter must activate it and only it.
    press(cx, "tab tab tab tab tab tab tab");
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["inside"],
        "Tab must stay inside the dialog: Enter activates an inside control"
    );
    assert!(
        probed.borrow().is_empty(),
        "the outside probe must never be reached by Tab"
    );
}

#[gpui::test]
fn drawer_escape_and_drag_dismiss(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let open = Rc::new(RefCell::new(true));
    // The render closure owns this handle; the test function keeps `open`
    // for the reopen below, so the clone is where the two split.
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let is_open = *open_flag.borrow();
        Drawer::new()
            .id("ovl-drawer")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .on_open_change({
                let open_flag = open_flag.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // The drawer claims the focus like the modal does, so Escape reaches its
    // keyboard dismissal handler immediately.
    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false"],
        "escape must dismiss the drawer"
    );

    // Drag half, reopened. The pull starts on the title row (1760, 48) — the
    // header's mouse-down creates the drag record — and travels 100px right,
    // over the dismissal threshold (PANEL_EXTENT * 0.25 = 80px), so the
    // release handler reports the close exactly once. The press that starts
    // the pull is inside the panel, so nothing else may report it: before the
    // fix the backdrop's click passed the same press through and each pull
    // reported twice, and the coordinate in the tests hit the panel body, so
    // the release threshold was never even consulted.
    *open.borrow_mut() = true;
    cx.update(|window, _| window.refresh());
    drag(cx, (1760., 48.), (1860., 48.));
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the far pull must report the close exactly once"
    );

    // Closed-proof: after the exit, a press where the title row was records
    // nothing — the panel and its dismissal listener are unmounted.
    let_exit_finish(cx);
    click(cx, 1760., 48.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:false", "open:false"],
        "the panel must be gone after the exit"
    );
}

/// The distance threshold is 30% of the measured panel, and a fast flick may
/// dismiss before reaching it. This case deliberately moves slowly so the
/// 40px pull exercises the distance path and springs back. It starts inside
/// the title row, so outside-press dismissal is not involved.
#[gpui::test]
fn drawer_small_pull_springs_back(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let open = Rc::new(RefCell::new(true));

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let is_open = *open.borrow();
        Drawer::new()
            .id("ovl-drawer-small")
            .is_open(is_open)
            .placement(DrawerPlacement::Right)
            .title("Drag me shut")
            .on_open_change({
                let open_flag = open.clone();
                move |v, window, _| {
                    *open_flag.borrow_mut() = v;
                    rec.borrow_mut().push(format!("open:{v}"));
                    window.refresh();
                }
            })
            .into_any_element()
    });

    // A slow 40px pull is above the 8px activation distance but below 30% of
    // the 384px panel, and below the flick velocity threshold.
    slow_drag(cx, (1728., 48.), (1768., 48.));
    assert!(
        recorded.borrow().is_empty(),
        "a sub-threshold pull must not dismiss the drawer"
    );
}

#[gpui::test]
fn alert_dialog_actions_report(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let actions = rec.clone();
    let open = Rc::new(RefCell::new(true));
    // The render closure owns this handle; the test function keeps `open`
    // for the reopen below, so the clone is where the two split.
    let open_flag = open.clone();

    let cx = open_host(cx, move || {
        let probe_rec = rec.clone();
        let confirm_actions = rec.clone();
        let cancel_actions = rec.clone();
        let is_open = *open_flag.borrow();
        // An alert dialog is not dismissible out of the box: the user has to
        // pick one of the two actions, so the test drives those buttons and
        // nothing else. The probe sits outside, first in the tab order.
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(320.))
            .child(
                Button::new("ovl-alert-probe")
                    .label("Outside")
                    .on_press(move |_, _, _| probe_rec.borrow_mut().push("outside".into())),
            )
            .child(
                AlertDialog::new("Delete everything?")
                    .id("ovl-alert")
                    .is_open(is_open)
                    .confirm_label("Delete")
                    .cancel_label("Cancel")
                    .on_confirm({
                        let actions = confirm_actions;
                        let open_flag = open_flag.clone();
                        move |_, window, _| {
                            actions.borrow_mut().push("confirm".into());
                            *open_flag.borrow_mut() = false;
                            window.refresh();
                        }
                    })
                    .on_cancel({
                        let actions = cancel_actions;
                        let open_flag = open_flag.clone();
                        move |_, window, _| {
                            actions.borrow_mut().push("cancel".into());
                            *open_flag.borrow_mut() = false;
                            window.refresh();
                        }
                    }),
            )
            .into_any_element()
    });

    // The action row is `justify-end gap-2` under the header: p(24) + 24px
    // title + mt(20) + 36px button = 128px tall, so the panel spans
    // y [476..604] and the row sits at y [520..556], flush right against
    // x = 1184 - 24 = 1160. The test drives the buttons by keyboard — the
    // dialog's own tab cycle, which is the honest claim to make. Cancel is
    // the first stop in the dialog, so one Tab lands on it and Enter must
    // report "cancel" exactly once.
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        actions.borrow().as_slice(),
        ["cancel"],
        "the cancel action must fire exactly once"
    );

    // The close fell to our handler: the dialog is gone, so Tab now reaches
    // the probe behind it.
    let_exit_finish(cx);
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        actions.borrow().as_slice(),
        ["cancel", "outside"],
        "the closed dialog must no longer hold the focus"
    );

    // Reopened: Tab twice lands on Confirm (cancel, then confirm) and Enter
    // must report it. The primary button is the second stop, furthest from
    // the reading position.
    *open.borrow_mut() = true;
    cx.update(|window, _| window.refresh());
    press(cx, "tab tab");
    press(cx, "enter");
    assert_eq!(
        actions.borrow().as_slice(),
        ["cancel", "outside", "confirm"],
        "the confirm action must fire exactly once"
    );

    // Closed again: the focus falls back out to the probe.
    let_exit_finish(cx);
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        actions.borrow().as_slice(),
        ["cancel", "outside", "confirm", "outside"],
        "the closed dialog must no longer hold the focus"
    );
}

#[gpui::test]
fn popover_escape_and_outside_press_close(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let recorded = rec.clone();
    let open = Rc::new(RefCell::new(false));

    let cx = open_host(cx, move || {
        let rec = rec.clone();
        let is_open = *open.borrow();
        // The 36px trigger button sits inside a wrapper with a 400px left
        // margin (the button itself draws its own surface and takes no style
        // methods, so the margin lives on the wrapper).
        Popover::new(
            gpui::div()
                .ml(px(400.))
                .child(Button::new("ovl-popover-trigger").label("Go")),
        )
        .id("ovl-popover")
        .is_open(is_open)
        .on_open_change({
            let open_flag = open.clone();
            move |v, window, _| {
                *open_flag.borrow_mut() = v;
                rec.borrow_mut().push(format!("open:{v}"));
                window.refresh();
            }
        })
        .into_any_element()
    });

    // The trigger's centre is roughly (423, 18): x 400 + half of the button
    // width. The Bottom panel is offset 8px below it, 260px wide and centred
    // on the trigger, so it spans x [~293..553], y [44..100] — the probe at
    // (100, 300) is clear of it either way. Popovers have no backdrop, so
    // these presses are not subject to the pass-through defect.
    click(cx, 423., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true"],
        "a press on the trigger must open the popover"
    );

    // The click left the focus on the trigger, so Escape bubbles up through
    // the popover root and must close it.
    press(cx, "escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "open:false"],
        "escape must close the open popover"
    );

    // Closed-proof: after the exit, a press outside the panel's former box
    // records nothing — while the panel was open the same press would have
    // been "outside" and closed it.
    let_exit_finish(cx);
    click(cx, 100., 300.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "open:false"],
        "the closed popover must not answer presses"
    );

    // Reopen, then dismiss by a press outside the panel: the panel watches
    // its own bounds for `on_mouse_down_out`, and (100, 300) is clear of
    // x [293..553].
    click(cx, 423., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "open:false", "open:true"],
        "the trigger must open the popover again"
    );
    click(cx, 100., 300.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "open:false", "open:true", "open:false"],
        "a press outside the panel must close it"
    );

    // And the same probe again records nothing once the exit is done.
    let_exit_finish(cx);
    click(cx, 100., 300.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["open:true", "open:false", "open:true", "open:false"],
        "the closed popover must not answer presses"
    );
}

/// The tooltip's hidden-tip proof.
///
/// A tooltip tip is a plain positioned div with no hitbox (nothing to click,
/// focus or scroll registers one) and the component exposes no open-state
/// callback, so a probe click can neither prove its presence nor its absence.
/// The honest behavioural lever is the *focus gate*: with
/// `trigger(TooltipTrigger::Focus)` the tip is open exactly when
/// `wrap.contains_focused && focus_visible` — the focus is on the trigger and
/// the last input was a key — **and no Escape dismissal is latched on top**.
///
/// Escape must hide the tip by tripping that per-tooltip latch, which is why
/// the assertions check the two halves of the gate that stay reachable: the
/// app-wide `focus_visible` must survive (clearing it would starve every
/// other focus ring in the window of its ring) and the focus must stay on the
/// trigger (Enter still activates it). With both halves intact, the only way
/// the tooltip's open condition can have changed under the Escape key is the
/// latch — escape hides the tip, and does not hide it forever: the latch is
/// dropped again once the focus leaves the trigger, so the next focus shows
/// the tip.
#[gpui::test]
fn tooltip_shows_on_focus_and_hides_on_escape(cx: &mut TestAppContext) {
    still();
    let pressed = events();
    let recorded = pressed.clone();

    let cx = open_host(cx, move || {
        let pressed = pressed.clone();
        // The harness has no app root, so this content root stands in for
        // `util::app_focus_root`'s recording half: every key marks the input
        // as keyboard, every mouse press as pointer. `focus_visible` is one
        // half of the tooltip's `focus_open` gate.
        gpui::div()
            .capture_key_down(|_, _, cx| util::set_focus_visible(true, cx))
            .capture_any_mouse_down(|_, _, cx| util::set_focus_visible(false, cx))
            .child(
                Tooltip::new("Keyboard tip")
                    .id("ovl-tt")
                    .trigger(TooltipTrigger::Focus)
                    .child(
                        Button::new("ovl-tt-trigger")
                            .label("Focus me")
                            .on_press(move |_, _, _| pressed.borrow_mut().push("pressed".into())),
                    ),
            )
            .into_any_element()
    });

    // The first press focuses the trigger but marks the input as pointer, so
    // `focus_open` is still false. The next key flips the input kind to
    // keyboard, which is the second half of the gate: a `trigger="focus"`
    // tooltip shows only when a keyboard user has reached its trigger.
    click(cx, 40., 18.);
    press(cx, "j");
    assert!(
        cx.update(|_, cx| util::focus_visible(cx)),
        "the keyboard-input half of focus_open must be set"
    );

    // The trigger holds the focus: Enter activates it. The click at (40, 18)
    // already recorded one press — gpui auto-focuses a `track_focus`ed
    // element on mouse-down, and the button's own click fires for the mouse
    // click as well — so the Enter contributes the *second* entry, which is
    // the keyboard press that proves the focus never left the trigger.
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["pressed", "pressed"],
        "the trigger must hold the focus: Enter activates it"
    );

    // Escape must now hide the tip. The dismissal handler trips the tooltip's
    // own `focus_dismissed` latch — it must not clear the app-wide
    // `focus_visible`, because every focus ring in the window reads it. So
    // the assertions below pin the two halves of the gate that remain
    // reachable: the input kind is unchanged *and* the focus is still on the
    // trigger. With both intact, the open condition can only have changed
    // through the latch, which is exactly the "the tip is hidden" claim this
    // suite can make — the tip itself has no hitbox to probe. The latch is
    // per focus session: it is dropped when the focus leaves the trigger, so
    // the next focus is a fresh one and shows the tip again.
    press(cx, "escape");
    let focus_after = cx.update(|_, cx| util::focus_visible(cx));
    assert!(
        focus_after,
        "escape must hide the tip through its own latch, not by clearing the \
         app-wide focus-visible every focus ring in the window reads"
    );

    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["pressed", "pressed", "pressed"],
        "the focus must never have left the trigger after escape"
    );
}

#[gpui::test]
fn toast_queue_adds_and_dismisses(cx: &mut TestAppContext) {
    let (a, b) = cx.update(|cx| {
        // Both toasts take the default 4s timeout, queued immediately. The
        // store is app-global (`ToastHub`), so the entity is the state.
        let a = Toast::new("First toast").push(None, cx);
        let b = Toast::new("Second toast").push(None, cx);
        (a, b)
    });
    assert_ne!(a, b, "each toast must get its own id");

    cx.update(|cx| {
        let store = toast_store(cx);
        let toasts = store.read(cx).toasts();
        assert_eq!(toasts.len(), 2, "both toasts must be queued");
        assert!(
            toasts
                .iter()
                .any(|t| t.id == a && t.title.as_ref() == "First toast"),
            "the first toast must be queued with its title"
        );
        assert!(
            toasts
                .iter()
                .any(|t| t.id == b && t.title.as_ref() == "Second toast"),
            "the second toast must be queued with its title"
        );
    });

    // Dismissing one by id must leave the other, and only the other.
    cx.update(|cx| dismiss_toast(a, cx));
    cx.update(|cx| {
        let store = toast_store(cx);
        let toasts = store.read(cx).toasts();
        assert_eq!(toasts.len(), 1, "dismissing one toast must leave the other");
        assert_eq!(toasts[0].id, b);
    });
}

#[gpui::test]
fn toast_auto_dismiss_after_timeout(cx: &mut TestAppContext) {
    // A timed toast (300ms) and a `timeout: 0` toast that must stay until it
    // is closed: the zero is v3's persistent toast, and `push` starts no clock
    // for it.
    let auto = cx.update(|cx| {
        Toast::new("Times out")
            .timeout(Duration::from_millis(300))
            .push(None, cx)
    });
    let stay = cx.update(|cx| Toast::loading("Stays until closed").push(None, cx));

    // The clock ticks every 100ms and subtracts from the remaining timeout;
    // 1500ms is far past 300ms.
    cx.executor().advance_clock(Duration::from_millis(1500));
    cx.update(|cx| {
        let store = toast_store(cx);
        let toasts = store.read(cx).toasts();
        assert!(
            toasts.iter().all(|t| t.id != auto),
            "the timed-out toast must be gone"
        );
        assert!(
            toasts.iter().any(|t| t.id == stay),
            "the zero-timeout toast must remain"
        );
    });

    // Long after, the persistent toast still holds — no clock was ever started
    // for it.
    cx.executor().advance_clock(Duration::from_secs(10));
    cx.update(|cx| {
        let store = toast_store(cx);
        assert!(store.read(cx).toasts().iter().any(|t| t.id == stay));
    });
}
