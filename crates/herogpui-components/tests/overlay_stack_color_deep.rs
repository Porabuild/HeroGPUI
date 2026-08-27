//! ColorPicker overlay-stack behavior against the pinned React Aria contract.
//!
//! React Aria 3.51.0's `useOverlay` keeps visible overlays in order and lets
//! only the topmost overlay close on Escape or an outside interaction. These
//! tests keep a ColorPicker inside a Popover so a second event proves the
//! parent remains available after the child closes.

mod harness;

use std::{cell::Cell, rc::Rc};

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext};
use harness::{click, events, open_host, press};
use herogpui_components::{Button, ColorPicker, PickerColor, Popover};

fn reduced_motion() {
    harness::still();
}

#[gpui::test]
fn nested_color_picker_escape_closes_only_the_picker_then_parent(cx: &mut TestAppContext) {
    reduced_motion();
    let picker_open = Rc::new(Cell::new(true));
    let outer_open = Rc::new(Cell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            let picker_open = picker_open.clone();
            let outer_open = outer_open.clone();
            let changes = changes.clone();
            Popover::new(Button::new("color-stack-outer-trigger").label("Outer"))
                .id("color-stack-outer")
                .is_open(outer_open.get())
                .on_open_change({
                    let outer_open = outer_open.clone();
                    let changes = changes.clone();
                    move |value, window, _| {
                        outer_open.set(value);
                        changes.borrow_mut().push(format!("outer:{value}"));
                        window.refresh();
                    }
                })
                .child(
                    ColorPicker::new("color-stack-picker", PickerColor::hsb(210., 0.5, 0.6))
                        .is_open(picker_open.get())
                        .on_open_change({
                            let picker_open = picker_open.clone();
                            move |value, window, _| {
                                picker_open.set(value);
                                changes.borrow_mut().push(format!("picker:{value}"));
                                window.refresh();
                            }
                        }),
                )
                .into_any_element()
        }
    });

    // The parent focus scope first reaches the ColorPicker trigger and then
    // its ColorArea. Escape bubbles from that area through the picker root.
    press(cx, "tab tab");
    press(cx, "escape");
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false"],
        "the first Escape must close only the topmost ColorPicker"
    );

    press(cx, "escape");
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false", "outer:false"],
        "the second Escape must reach the still-open parent Popover"
    );
}

#[gpui::test]
fn nested_color_picker_outside_press_closes_only_the_picker_then_parent(cx: &mut TestAppContext) {
    reduced_motion();
    let picker_open = Rc::new(Cell::new(true));
    let outer_open = Rc::new(Cell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            let picker_open = picker_open.clone();
            let outer_open = outer_open.clone();
            let changes = changes.clone();
            Popover::new(Button::new("color-outside-outer-trigger").label("Outer"))
                .id("color-outside-outer")
                .is_open(outer_open.get())
                .on_open_change({
                    let outer_open = outer_open.clone();
                    let changes = changes.clone();
                    move |value, window, _| {
                        outer_open.set(value);
                        changes.borrow_mut().push(format!("outer:{value}"));
                        window.refresh();
                    }
                })
                .child(
                    ColorPicker::new("color-outside-picker", PickerColor::hsb(210., 0.5, 0.6))
                        .is_open(picker_open.get())
                        .on_open_change({
                            let picker_open = picker_open.clone();
                            move |value, window, _| {
                                picker_open.set(value);
                                changes.borrow_mut().push(format!("picker:{value}"));
                                window.refresh();
                            }
                        }),
                )
                .into_any_element()
        }
    });

    click(cx, 700., 500.);
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false"],
        "the first outside press must close only the topmost ColorPicker"
    );

    click(cx, 700., 500.);
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false", "outer:false"],
        "the second outside press must reach the remaining Popover"
    );
}

#[gpui::test]
fn color_picker_escape_reports_one_close_and_restores_trigger_activation(cx: &mut TestAppContext) {
    reduced_motion();
    let picker_open = Rc::new(Cell::new(false));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            let picker_open = picker_open.clone();
            let changes = changes.clone();
            ColorPicker::new("color-focus-picker", PickerColor::hsb(210., 0.5, 0.6))
                .is_open(picker_open.get())
                .on_open_change({
                    let picker_open = picker_open.clone();
                    move |value, window, _| {
                        picker_open.set(value);
                        changes.borrow_mut().push(format!("picker:{value}"));
                        window.refresh();
                    }
                })
                .into_any_element()
        }
    });

    click(cx, 60., 12.);
    press(cx, "tab");
    press(cx, "escape");
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:true", "picker:false"],
        "Escape must invoke on_open_change exactly once"
    );

    // The closing path returns focus to the trigger, so its click listener is
    // the only activation path when Enter is pressed after Escape.
    press(cx, "enter");
    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:true", "picker:false", "picker:true"],
        "focus restoration must make Enter reopen the picker once"
    );
}

#[gpui::test]
fn clicking_an_open_color_picker_trigger_reports_one_close(cx: &mut TestAppContext) {
    reduced_motion();
    let picker_open = Rc::new(Cell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            ColorPicker::new("color-open-trigger-race", PickerColor::hsb(210., 0.5, 0.6))
                .is_open(picker_open.get())
                .on_open_change({
                    let picker_open = picker_open.clone();
                    let changes = changes.clone();
                    move |value, window, _| {
                        picker_open.set(value);
                        changes.borrow_mut().push(format!("picker:{value}"));
                        window.refresh();
                    }
                })
                .into_any_element()
        }
    });

    click(cx, 60., 12.);

    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false"],
        "an open trigger owns its close; outside dismissal must not report a second change"
    );
}

#[gpui::test]
fn canceled_color_picker_trigger_press_does_not_block_a_later_outside_close(
    cx: &mut TestAppContext,
) {
    reduced_motion();
    let picker_open = Rc::new(Cell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            ColorPicker::new("color-canceled-trigger", PickerColor::hsb(210., 0.5, 0.6))
                .is_open(picker_open.get())
                .on_open_change({
                    let picker_open = picker_open.clone();
                    let changes = changes.clone();
                    move |value, window, _| {
                        picker_open.set(value);
                        changes.borrow_mut().push(format!("picker:{value}"));
                        window.refresh();
                    }
                })
                .into_any_element()
        }
    });

    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(60.), px(12.)), MouseButton::Left, modifiers);
    cx.simulate_mouse_up(point(px(700.), px(500.)), MouseButton::Left, modifiers);
    click(cx, 700., 500.);

    assert_eq!(
        changes.borrow().as_slice(),
        ["picker:false"],
        "a canceled trigger press must not leave later outside dismissal declined"
    );
}

#[gpui::test]
fn controlled_color_picker_without_callback_blocks_parent_outside_dismissal(
    cx: &mut TestAppContext,
) {
    reduced_motion();
    let outer_open = Rc::new(Cell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            Popover::new(Button::new("color-static-outer-trigger").label("Outer"))
                .id("color-static-outer")
                .is_open(outer_open.get())
                .on_open_change({
                    let outer_open = outer_open.clone();
                    let changes = changes.clone();
                    move |value, window, _| {
                        outer_open.set(value);
                        changes.borrow_mut().push(format!("outer:{value}"));
                        window.refresh();
                    }
                })
                .child(
                    ColorPicker::new("color-static-picker", PickerColor::hsb(210., 0.5, 0.6))
                        .is_open(true),
                )
                .into_any_element()
        }
    });

    click(cx, 700., 500.);

    assert!(
        changes.borrow().is_empty(),
        "the topmost controlled picker must consume outside dismissal even when it cannot change"
    );
}
