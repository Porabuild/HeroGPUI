use crate::harness::{self, open_host};

use std::{cell::Cell, rc::Rc};

use gpui::{prelude::*, px, TestAppContext, VisualTestContext};
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
        cx.debug_bounds("dropdown-menu").is_some(),
        "the reduced-motion exit remains mounted for the overlay phase but paints its end state"
    );
    harness::wait_ms(cx, 120);
    assert!(cx.debug_bounds("dropdown-menu").is_none());
}
