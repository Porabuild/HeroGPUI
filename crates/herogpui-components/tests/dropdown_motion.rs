use crate::harness::{self, open_host};

use std::{cell::Cell, rc::Rc};

use gpui::{
    div, prelude::*, px, App, Modifiers, MouseButton, TestAppContext, VisualTestContext, Window,
};
use herogpui_components::{Button, Dropdown, MenuItem};
use herogpui_theme::{set_theme, ComponentTheme, ComponentThemes, MenuStyle, Theme};

#[derive(Clone, Copy, Debug)]
struct MotionSample {
    opacity: f32,
    height: f32,
    radius: f32,
}

fn refresh_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

fn install_menu_theme(cx: &mut VisualTestContext, style: MenuStyle) {
    cx.update(|window, cx| {
        set_theme(
            Theme::builder("menu-motion-probe", Theme::light())
                .components(ComponentThemes::default().menu(ComponentTheme::new(style)))
                .build(),
            cx,
        );
        window.refresh();
    });
    cx.run_until_parked();
}

fn large_motion_theme(cx: &mut VisualTestContext) {
    install_menu_theme(
        cx,
        MenuStyle::default().panel_padding(px(30.)).radius(px(32.)),
    );
}

fn close_enough(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.3
}

fn panel_fill(cx: &mut VisualTestContext) -> Option<(f32, f32, gpui::Bounds<gpui::Pixels>)> {
    let bounds = cx.debug_bounds("dropdown-menu")?;
    let painted = harness::painted(cx);
    let mut best: Option<(f32, f32)> = None;
    for quad in &painted.quads {
        let quad_bounds = painted.bounds(quad);
        let same_box = close_enough(f32::from(quad_bounds.origin.x), f32::from(bounds.origin.x))
            && close_enough(f32::from(quad_bounds.origin.y), f32::from(bounds.origin.y))
            && close_enough(
                f32::from(quad_bounds.size.width),
                f32::from(bounds.size.width),
            )
            && close_enough(
                f32::from(quad_bounds.size.height),
                f32::from(bounds.size.height),
            );
        if !same_box {
            continue;
        }
        let Some(fill) = quad.background.as_solid() else {
            continue;
        };
        let radius = painted.corners(quad)[0];
        if best.is_none_or(|(alpha, _)| fill.a > alpha) {
            best = Some((fill.a, radius));
        }
    }
    best.map(|(alpha, radius)| (alpha, radius, bounds))
}

fn motion_sample(cx: &mut VisualTestContext) -> MotionSample {
    let Some((opacity, radius, bounds)) = panel_fill(cx) else {
        panic!("the open menu panel must paint a measurable fill");
    };
    MotionSample {
        opacity,
        height: f32::from(bounds.size.height),
        radius,
    }
}

fn assert_continues(before: MotionSample, after: MotionSample, phase: &str) {
    assert!(
        (after.opacity - before.opacity).abs() < 0.025,
        "{phase}: opacity must continue from the painted frame: before={before:?} after={after:?}"
    );
    assert!(
        (after.height - before.height).abs() < 0.15,
        "{phase}: panel padding must continue from the painted frame: before={before:?} after={after:?}"
    );
    assert!(
        (after.radius - before.radius).abs() < 0.15,
        "{phase}: panel radius must continue from the painted frame: before={before:?} after={after:?}"
    );
}

fn controlled_dropdown(open: Rc<Cell<bool>>, id: &'static str) -> impl Fn() -> gpui::AnyElement {
    move || {
        Dropdown::new(
            id,
            Button::new(format!("{id}-trigger")).label("Actions"),
            vec![MenuItem::new("one", "One"), MenuItem::new("two", "Two")],
            open.get(),
        )
        .id(id)
        .into_any_element()
    }
}

#[gpui::test]
fn menu_motion_reverses_from_the_painted_frame_and_settles(cx: &mut TestAppContext) {
    let open = Rc::new(Cell::new(false));
    let cx = open_host(cx, controlled_dropdown(open.clone(), "dd-motion"));
    large_motion_theme(cx);

    open.set(true);
    refresh_frame(cx);
    harness::wait_ms(cx, 25);
    let entry_mid = motion_sample(cx);
    assert!(
        entry_mid.opacity > 0.05 && entry_mid.opacity < 0.95,
        "the entry sample must be mid-flight, not at either endpoint: {entry_mid:?}"
    );

    open.set(false);
    refresh_frame(cx);
    let exit_start = motion_sample(cx);
    assert_continues(entry_mid, exit_start, "mid-entry close");

    harness::wait_ms(cx, 20);
    let closing = motion_sample(cx);
    assert!(
        closing.opacity < exit_start.opacity,
        "exit opacity must move monotonically toward zero: start={exit_start:?} now={closing:?}"
    );
    assert!(
        closing.height > exit_start.height && closing.radius > exit_start.radius,
        "the early exit must move padding and radius monotonically toward the 95% exit end: start={exit_start:?} now={closing:?}"
    );

    open.set(true);
    refresh_frame(cx);
    let reopen_start = motion_sample(cx);
    assert_continues(closing, reopen_start, "closing reopen");

    harness::wait_ms(cx, 30);
    let reopening = motion_sample(cx);
    assert!(
        reopening.opacity > reopen_start.opacity,
        "reopen opacity must move monotonically toward one: start={reopen_start:?} now={reopening:?}"
    );
    assert!(
        reopening.height > reopen_start.height && reopening.radius > reopen_start.radius,
        "reopen padding and radius must move monotonically toward full size: start={reopen_start:?} now={reopening:?}"
    );

    harness::wait_ms(cx, 160);
    let settled = motion_sample(cx);
    assert!(
        settled.opacity > 0.999,
        "the 150ms entry must settle at full opacity: {settled:?}"
    );
    harness::wait_ms(cx, 30);
    let rested = motion_sample(cx);
    assert_continues(settled, rested, "settled entry fixed point");

    open.set(false);
    refresh_frame(cx);
    let settled_exit_start = motion_sample(cx);
    assert_continues(rested, settled_exit_start, "settled close");
    harness::wait_ms(cx, 90);
    let late_exit = motion_sample(cx);
    assert!(
        late_exit.opacity < 0.05,
        "the 100ms exit must be essentially transparent at 90ms: {late_exit:?}"
    );
    assert!(
        late_exit.height < settled_exit_start.height && late_exit.radius < settled_exit_start.radius,
        "a settled exit must shrink padding and radius toward the 95% end: start={settled_exit_start:?} now={late_exit:?}"
    );
    assert!(
        cx.debug_bounds("dropdown-menu").is_some(),
        "the panel must remain mounted through the exit run"
    );

    harness::wait_ms(cx, 30);
    assert!(
        cx.debug_bounds("dropdown-menu").is_none(),
        "the panel must unmount after the settled 100ms exit"
    );

    open.set(true);
    refresh_frame(cx);
    let remounted = motion_sample(cx);
    assert!(
        remounted.opacity < 0.05,
        "a fresh open after a completed exit must restart the entry fade: {remounted:?}"
    );
    assert!(
        remounted.height < rested.height - 4.0 && remounted.radius < rested.radius - 2.0,
        "a fresh open must restart at the 90% entry frame rather than reuse the 95% exit end: remounted={remounted:?} settled={rested:?}"
    );
}

#[gpui::test]
fn menu_entry_opt_out_is_instant_while_exit_still_runs(cx: &mut TestAppContext) {
    let open = Rc::new(Cell::new(false));
    let cx = open_host(cx, controlled_dropdown(open.clone(), "dd-instant-motion"));
    install_menu_theme(
        cx,
        MenuStyle::default()
            .animate_entry(false)
            .panel_padding(px(30.))
            .radius(px(32.)),
    );

    open.set(true);
    refresh_frame(cx);
    let first = motion_sample(cx);
    assert!(
        first.opacity > 0.999,
        "animate_entry(false) must paint the first entry frame at full opacity: {first:?}"
    );
    harness::wait_ms(cx, 25);
    let later = motion_sample(cx);
    assert_continues(first, later, "instant entry fixed point");

    open.set(false);
    refresh_frame(cx);
    harness::wait_ms(cx, 20);
    let exiting = motion_sample(cx);
    assert!(
        exiting.opacity < 1.0,
        "animate_entry(false) must not suppress the exit fade: {exiting:?}"
    );
}

#[gpui::test]
fn reduced_motion_snaps_menu_entry_and_exit_without_animation(cx: &mut TestAppContext) {
    harness::still();
    let open = Rc::new(Cell::new(false));
    let cx = open_host(cx, controlled_dropdown(open.clone(), "dd-reduced-motion"));

    open.set(true);
    refresh_frame(cx);
    let entered = motion_sample(cx);
    assert!(
        entered.opacity > 0.999,
        "reduced motion must snap the entry to full opacity: {entered:?}"
    );
    let pending = cx.update(|window, cx| window.simulate_next_frame(cx));
    assert_eq!(
        pending, 0,
        "reduced motion must not schedule an entry animation"
    );

    open.set(false);
    refresh_frame(cx);
    let pending = cx.update(|window, cx| window.simulate_next_frame(cx));
    assert_eq!(
        pending, 0,
        "reduced motion must not schedule an exit animation"
    );
    assert!(
        cx.debug_bounds("dropdown-menu").is_none(),
        "reduced motion has no exit to play, so the close unmounts the panel immediately"
    );
    harness::wait_ms(cx, 120);
    assert!(cx.debug_bounds("dropdown-menu").is_none());
}

/// Counts how often gpui creates the keyed state inside one row's caller
/// content: a remount of the row subtree reruns the creation closure.
#[derive(IntoElement)]
struct MountProbe(Rc<Cell<u32>>);

impl RenderOnce for MountProbe {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        window.use_keyed_state("mount-probe-state", cx, |_, _| self.0.set(self.0.get() + 1));
        div()
            .w(px(90.))
            .h(px(22.))
            .debug_selector(|| "motion-probe-row".into())
            .child("Row")
    }
}

#[gpui::test]
fn menu_state_and_click_survive_entry_settlement(cx: &mut TestAppContext) {
    let mounts = Rc::new(Cell::new(0u32));
    let actions = Rc::new(Cell::new(0u32));
    let counters = (mounts.clone(), actions.clone());
    let cx = open_host(cx, move || {
        let (mounts, actions) = counters.clone();
        herogpui_components::Menu::new("motion-menu", vec![MenuItem::new("row", "Row")])
            .item_content(move |_, _| MountProbe(mounts.clone()).into_any_element())
            .on_action(move |_, _, _| actions.set(actions.get() + 1))
            .into_any_element()
    });
    refresh_frame(cx);
    let entry_mounts = mounts.get();
    assert!(entry_mounts > 0, "the row content must mount once on open");

    // The press starts at 120ms of the 150ms entry and the release lands at
    // 180ms: the click spans the settlement frame, and the row must be the
    // same element-state instance before and after it.
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(120));
    refresh_frame(cx);
    let pos = cx
        .debug_bounds("motion-probe-row")
        .expect("row must be mounted mid-entry")
        .center();
    cx.simulate_mouse_move(pos, None, Modifiers::none());
    cx.simulate_mouse_down(pos, MouseButton::Left, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(60));
    refresh_frame(cx);
    cx.simulate_mouse_up(pos, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.get(),
        1,
        "a click spanning the entry settlement must still activate the row"
    );
    assert_eq!(
        mounts.get(),
        entry_mounts,
        "settlement must not remount menu row state"
    );
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(actions.get(), 2, "the settled row must stay clickable");
    assert_eq!(
        mounts.get(),
        entry_mounts,
        "the settled row must be the same keyed state instance"
    );
}

#[gpui::test]
fn menu_row_state_survives_close_and_reopen_reversal(cx: &mut TestAppContext) {
    let exiting = Rc::new(Cell::new(false));
    let mounts = Rc::new(Cell::new(0u32));
    let control = (exiting.clone(), mounts.clone());
    let cx = open_host(cx, move || {
        let (exiting, mounts) = control.clone();
        herogpui_components::Menu::new("motion-reversal-menu", vec![MenuItem::new("row", "Row")])
            .item_content(move |_, _| MountProbe(mounts.clone()).into_any_element())
            .exiting(exiting.get())
            .into_any_element()
    });
    refresh_frame(cx);
    let entry_mounts = mounts.get();
    assert!(entry_mounts > 0);

    // Reverse mid-entry into the exit, then reverse back into the entry; the
    // motion must continue from painted frames without ever remounting rows.
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(60));
    refresh_frame(cx);
    exiting.set(true);
    refresh_frame(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(40));
    refresh_frame(cx);
    exiting.set(false);
    refresh_frame(cx);
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(40));
    refresh_frame(cx);
    assert_eq!(
        mounts.get(),
        entry_mounts,
        "close/reopen reversals must not remount menu row state"
    );
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(200));
    refresh_frame(cx);
    assert_eq!(
        mounts.get(),
        entry_mounts,
        "settling after a reversal must not remount menu row state"
    );
    assert!(
        cx.debug_bounds("motion-probe-row").is_some(),
        "the reopened menu rows must still be mounted"
    );
}

#[gpui::test]
fn reduced_motion_closed_menu_cannot_run_an_action(cx: &mut TestAppContext) {
    harness::still();
    let open = Rc::new(Cell::new(true));
    let actions = Rc::new(Cell::new(0u32));
    let control = (open.clone(), actions.clone());
    let cx = open_host(cx, move || {
        let (open, actions) = control.clone();
        Dropdown::new(
            "reduced-close-probe",
            Button::new("reduced-close-trigger").label("Open"),
            vec![MenuItem::new("row", "Row")],
            open.get(),
        )
        .item_content(move |_, _| {
            div()
                .w(px(90.))
                .h(px(22.))
                .debug_selector(|| "reduced-close-row".into())
                .child("Row")
                .into_any_element()
        })
        .on_action(move |_, _, _| actions.set(actions.get() + 1))
        .id("reduced-close-dropdown")
        .into_any_element()
    });
    refresh_frame(cx);
    let pos = cx
        .debug_bounds("reduced-close-row")
        .expect("the open menu must paint its row")
        .center();

    open.set(false);
    refresh_frame(cx);
    assert!(
        cx.debug_bounds("reduced-close-row").is_none(),
        "reduced motion must drop the closed menu immediately rather than retain a transparent panel"
    );
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.get(),
        0,
        "no action may fire from the position a closed menu occupied"
    );
}

#[gpui::test]
fn reduced_motion_retained_standalone_menu_is_inert_while_exiting(cx: &mut TestAppContext) {
    harness::still();
    let exiting = Rc::new(Cell::new(false));
    let actions = Rc::new(Cell::new(0u32));
    let control = (exiting.clone(), actions.clone());
    let cx = open_host(cx, move || {
        let (exiting, actions) = control.clone();
        herogpui_components::Menu::new("reduced-retained-menu", vec![MenuItem::new("row", "Row")])
            .item_content(move |_, _| {
                div()
                    .w(px(90.))
                    .h(px(22.))
                    .debug_selector(|| "reduced-retained-row".into())
                    .child("Row")
                    .into_any_element()
            })
            .on_action(move |_, _, _| actions.set(actions.get() + 1))
            .exiting(exiting.get())
            .into_any_element()
    });
    refresh_frame(cx);
    let pos = cx
        .debug_bounds("reduced-retained-row")
        .expect("the open retained menu must paint its row")
        .center();
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(actions.get(), 1, "the open menu's row must activate");

    exiting.set(true);
    refresh_frame(cx);
    assert!(
        cx.debug_bounds("reduced-retained-row").is_none(),
        "a retained menu exiting under reduced motion must not keep mounted rows"
    );
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        actions.get(),
        1,
        "no action may fire from an invisible exiting menu"
    );

    exiting.set(false);
    refresh_frame(cx);
    assert!(
        cx.debug_bounds("reduced-retained-row").is_some(),
        "flipping back to open re-renders the rows for the caller-retained menu"
    );
}

#[gpui::test]
fn normal_motion_retained_standalone_menu_keeps_rows_while_exiting(cx: &mut TestAppContext) {
    let exiting = Rc::new(Cell::new(false));
    let control = exiting.clone();
    let cx = open_host(cx, move || {
        herogpui_components::Menu::new("normal-retained-menu", vec![MenuItem::new("row", "Row")])
            .item_content(move |_, _| {
                div()
                    .w(px(90.))
                    .h(px(22.))
                    .debug_selector(|| "normal-retained-row".into())
                    .child("Row")
                    .into_any_element()
            })
            .exiting(control.get())
            .into_any_element()
    });
    refresh_frame(cx);
    harness::wait_ms(cx, 200);
    exiting.set(true);
    refresh_frame(cx);
    assert!(
        cx.debug_bounds("normal-retained-row").is_some(),
        "the visible exit run keeps its rows mounted under normal motion"
    );
}

#[gpui::test]
fn reduced_motion_retained_exit_releases_dismissal_order(cx: &mut TestAppContext) {
    harness::still();
    let exiting = Rc::new(Cell::new(false));
    let lower_dismissals = Rc::new(Cell::new(0u32));
    let upper_dismissals = Rc::new(Cell::new(0u32));
    let control = (
        exiting.clone(),
        lower_dismissals.clone(),
        upper_dismissals.clone(),
    );
    let cx = open_host(cx, move || {
        let (exiting, lower_dismissals, upper_dismissals) = control.clone();
        let lower = herogpui_components::Menu::new(
            "lower-overlay",
            vec![MenuItem::new("lower-row", "Lower")],
        )
        .on_dismiss(move |_, _, _| lower_dismissals.set(lower_dismissals.get() + 1));
        div()
            .size_full()
            .child(lower)
            .child(
                herogpui_components::Menu::new(
                    "upper-overlay",
                    vec![MenuItem::new("upper-row", "Upper")],
                )
                .exiting(exiting.get())
                .item_content(move |_, _| {
                    div()
                        .w(px(90.))
                        .h(px(22.))
                        .debug_selector(|| "upper-retained-row".into())
                        .child("Upper")
                        .into_any_element()
                })
                .on_dismiss(move |_, _, _| upper_dismissals.set(upper_dismissals.get() + 1)),
            )
            .into_any_element()
    });
    refresh_frame(cx);
    let pos = cx.update(|window, _| {
        let size = window.viewport_size();
        gpui::point(size.width - px(10.), size.height - px(10.))
    });
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        (upper_dismissals.get(), lower_dismissals.get()),
        (1, 0),
        "an outside press dismisses only the topmost open overlay"
    );

    exiting.set(true);
    refresh_frame(cx);
    assert!(
        cx.debug_bounds("upper-retained-row").is_none(),
        "the stripped reduced-motion exit must not paint rows"
    );
    cx.simulate_click(pos, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        lower_dismissals.get(),
        1,
        "the lower overlay must dismiss once the hidden upper releases its registration"
    );
    assert_eq!(
        upper_dismissals.get(),
        1,
        "the hidden upper must not absorb another dismissal while stripped"
    );
}
