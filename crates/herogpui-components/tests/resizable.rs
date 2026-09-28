//! `ResizablePanelGroup` (HeroGPUI extension): panels share the group's
//! length by percentage; a handle drag (with pointer capture) or the arrow,
//! Home and End keys on a focused handle resize the two panels beside it
//! within their limits, controlled or uncontrolled.

mod harness;

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use harness::{events, open_host, press, Events};
use herogpui_components::{Orientation, ResizablePanel, ResizablePanelGroup};

/// 500px shared by the panels plus two 8px handles.
const WIDTH: f32 = 516.;
const HEIGHT: f32 = 200.;

#[derive(Clone, Copy, Default)]
struct Config {
    vertical: bool,
    controlled: bool,
    disabled: bool,
}

fn host(cx: &mut TestAppContext, config: Config) -> (Events, &mut VisualTestContext) {
    let seen = events();
    let s = seen.clone();
    let cx = open_host(cx, move || {
        let s = s.clone();
        let mut group = ResizablePanelGroup::new("split")
            .panel(
                ResizablePanel::new()
                    .default_size(30.)
                    .min_size(20.)
                    .max_size(60.)
                    .child(gpui::div().child("One")),
            )
            .panel(
                ResizablePanel::new()
                    .default_size(40.)
                    .min_size(25.)
                    .child(gpui::div().child("Two")),
            )
            .panel(
                ResizablePanel::new()
                    .default_size(30.)
                    .child(gpui::div().child("Three")),
            )
            .is_disabled(config.disabled)
            .on_resize(move |sizes, _, _| {
                let text: Vec<String> = sizes.iter().map(|v| format!("{v:.0}")).collect();
                s.borrow_mut().push(text.join(","));
            });
        if config.vertical {
            group = group.orientation(Orientation::Vertical);
        }
        if config.controlled {
            group = group.sizes([30., 40., 30.]);
        }
        let (w, h) = if config.vertical {
            (HEIGHT, WIDTH)
        } else {
            (WIDTH, HEIGHT)
        };
        gpui::div()
            .w(px(w))
            .h(px(h))
            .child(group)
            .into_any_element()
    });
    (seen, cx)
}

fn frame(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

fn bounds(cx: &mut VisualTestContext, name: &str) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(Box::leak(name.to_owned().into_boxed_str()))
        .unwrap_or_else(|| panic!("{name} laid out"))
}

/// The panels' lengths along the group's axis, rounded to whole pixels.
fn lengths(cx: &mut VisualTestContext, vertical: bool) -> Vec<f32> {
    (0..3)
        .map(|ix| {
            let b = bounds(cx, &format!("split-panel-{ix}"));
            f32::from(if vertical {
                b.size.height
            } else {
                b.size.width
            })
            .round()
        })
        .collect()
}

fn log(seen: &Events) -> Vec<String> {
    seen.borrow().clone()
}

#[gpui::test]
fn default_sizes_share_the_space_between_the_handles(cx: &mut TestAppContext) {
    let (_, cx) = host(cx, Config::default());
    frame(cx);
    assert_eq!(lengths(cx, false), [150., 200., 150.]);
    let handle = bounds(cx, "split-handle-0");
    assert_eq!(handle.size.width, px(8.));
    assert_eq!(handle.origin.x, px(150.));
}

#[gpui::test]
fn a_drag_resizes_the_pair_clamps_and_outlives_the_handle(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    let start = bounds(cx, "split-handle-0").center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    frame(cx);
    // 50px of the 500 the panels share is 10%.
    let step = point(start.x + px(50.), start.y);
    cx.simulate_mouse_move(step, Some(MouseButton::Left), Modifiers::none());
    frame(cx);
    assert_eq!(lengths(cx, false), [200., 150., 150.]);
    // Far past the limits, and off the group entirely: the second panel's
    // 25% minimum stops the first at 45% (under its own 60% maximum), and
    // the third panel never moves.
    let far = point(px(700.), px(400.));
    cx.simulate_mouse_move(far, Some(MouseButton::Left), Modifiers::none());
    frame(cx);
    assert_eq!(lengths(cx, false), [225., 125., 150.]);
    // And back below the first panel's 20% minimum.
    let back = point(px(-50.), start.y);
    cx.simulate_mouse_move(back, Some(MouseButton::Left), Modifiers::none());
    frame(cx);
    assert_eq!(lengths(cx, false), [100., 250., 150.]);
    cx.simulate_mouse_up(back, MouseButton::Left, Modifiers::none());
    frame(cx);
    // Released: moving no longer resizes.
    cx.simulate_mouse_move(step, None, Modifiers::none());
    frame(cx);
    assert_eq!(lengths(cx, false), [100., 250., 150.]);
    assert_eq!(log(&seen), ["40,30,30", "45,25,30", "20,50,30"]);
}

#[gpui::test]
fn arrow_keys_step_and_home_end_jump_to_the_limits(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    press(cx, "tab");
    frame(cx);
    for key in ["right", "shift-right", "home", "end", "down", "up"] {
        press(cx, key);
        frame(cx);
    }
    assert_eq!(lengths(cx, false), [225., 125., 150.]);
    // The second handle is the next tab stop and moves only its own pair;
    // Left is held by the second panel's 25% minimum.
    press(cx, "tab");
    frame(cx);
    press(cx, "left");
    frame(cx);
    press(cx, "right");
    frame(cx);
    assert_eq!(
        log(&seen),
        [
            "35,35,30", // Right: +5
            "45,25,30", // Shift+Right: +20, held by the second panel's minimum
            "20,50,30", // Home: the first panel's minimum
            "45,25,30", // End: as far as the pair allows; Up/Down are off-axis
            "45,30,25", // the second handle
        ]
    );
}

#[gpui::test]
fn a_vertical_group_resizes_with_up_and_down(cx: &mut TestAppContext) {
    let config = Config {
        vertical: true,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    assert_eq!(lengths(cx, true), [150., 200., 150.]);
    press(cx, "tab");
    frame(cx);
    press(cx, "right");
    frame(cx);
    press(cx, "down");
    frame(cx);
    assert_eq!(lengths(cx, true), [175., 175., 150.]);
    let start = bounds(cx, "split-handle-0").center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    frame(cx);
    let up = point(start.x, start.y - px(50.));
    cx.simulate_mouse_move(up, Some(MouseButton::Left), Modifiers::none());
    frame(cx);
    cx.simulate_mouse_up(up, MouseButton::Left, Modifiers::none());
    frame(cx);
    assert_eq!(log(&seen), ["35,35,30", "25,45,30"]);
}

#[gpui::test]
fn controlled_sizes_report_and_render_only_what_they_are_given(cx: &mut TestAppContext) {
    let config = Config {
        controlled: true,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    press(cx, "tab");
    frame(cx);
    press(cx, "right");
    frame(cx);
    press(cx, "right");
    frame(cx);
    assert_eq!(log(&seen), ["35,35,30", "35,35,30"]);
    assert_eq!(lengths(cx, false), [150., 200., 150.]);
}

#[gpui::test]
fn a_disabled_group_neither_drags_nor_takes_focus(cx: &mut TestAppContext) {
    let config = Config {
        disabled: true,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    let start = bounds(cx, "split-handle-0").center();
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    frame(cx);
    let step = point(start.x + px(50.), start.y);
    cx.simulate_mouse_move(step, Some(MouseButton::Left), Modifiers::none());
    frame(cx);
    cx.simulate_mouse_up(step, MouseButton::Left, Modifiers::none());
    press(cx, "tab");
    frame(cx);
    press(cx, "right");
    frame(cx);
    assert!(log(&seen).is_empty());
    assert_eq!(lengths(cx, false), [150., 200., 150.]);
}
