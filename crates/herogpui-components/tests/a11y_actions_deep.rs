//! Accessibility action-dispatch wiring and behavior checks for `Select` and `Slider`.
//!
//! # Why these tests combine unit helpers, integration hosts, and source checks
//!
//! `gpui-pre-0.3.11`'s `TestPlatform` leaves `PlatformWindow::a11y_init` as an
//! empty default (`window.is_a11y_active() == false`) and `handle_a11y_action`
//! is `pub(crate)`, so this pinned GPUI does not expose
//! `TestAppContext::activate_test_a11y()` / `send_test_a11y_action()` yet.
//! Consequently:
//!
//! - The exact `Expand`/`Collapse` (`Select`) and
//!   `SetValue`/`Increment`/`Decrement` (`Slider`) handler computations,
//!   including controlled/uncontrolled state, discrete `.steps(...)`,
//!   continuous `step == 0.0`, invalid bounds (`min > max`, `NaN`, `±Infinity`),
//!   wrong `ActionData`, and range-thumb neighbor clamping, are tested directly
//!   via unit tests in `select.rs` and `slider.rs`.
//! - This suite pins the element-level `.a11y_action(...)` registration and
//!   disabled-gate invariants alongside live-window integration parity for the
//!   same states.
//! - Actual OS accessibility adapters (UIA/NSAccessibility/AT-SPI) and live
//!   screen readers still require host verification outside headless CI.

use crate::harness;

use std::{cell::RefCell, rc::Rc};

use gpui::{prelude::*, px, TestAppContext};
use harness::{click, open_host, press, settle, still, wait_ms};
use herogpui_components::{PickerItem, Select, Slider};

#[test]
fn select_wires_expand_and_collapse_a11y_actions_when_enabled() {
    let source = include_str!("../src/select.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("select implementation section");
    assert!(
        source
            .contains("!self.is_disabled && (self.on_open_change.is_some() || open_own.is_some())")
            && source.contains("(a11y::Action::Expand, true)")
            && source.contains("(a11y::Action::Collapse, false)")
            && source.contains("field = field.a11y_action(action,")
            && source.contains("apply_select_a11y_action("),
        "Select trigger must register Expand/Collapse AccessKit actions only when enabled"
    );
}

#[test]
fn slider_wires_set_value_increment_and_decrement_a11y_actions_per_enabled_thumb() {
    let source = include_str!("../src/slider.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("slider implementation section");
    assert!(
        source.contains("if thumb_enabled[index]")
            && source.contains("crate::a11y::Action::SetValue")
            && source.contains("crate::a11y::Action::Increment")
            && source.contains("crate::a11y::Action::Decrement")
            && source.contains("thumb_el = thumb_el.a11y_action(action,")
            && source.contains("resolve_slider_a11y_action("),
        "Slider must register SetValue/Increment/Decrement AccessKit actions on enabled thumbs only"
    );
    assert!(
        source.contains("!min.is_finite() || !max.is_finite() || min > max")
            && source.contains("thumb_min")
            && source.contains("thumb_max")
            && source.contains("Range::new(thumb_min as f64, thumb_max as f64")
            && source.contains("sync_form_values")
            && source.contains("on_change_end_all")
            && source.contains("on_change_end"),
        "Slider AccessKit actions must guard invalid bounds, publish neighbor-clamped thumb limits, sync form state, and emit change-end callbacks"
    );
}

#[gpui::test]
fn disabled_select_ignores_open_attempts_and_controlled_select_round_trips(
    cx: &mut TestAppContext,
) {
    still();
    let disabled_events: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let disabled_seen = disabled_events.clone();
    let controlled_open: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
    let controlled_events: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let controlled_seen = controlled_events.clone();

    let vcx = open_host(cx, move || {
        let disabled_cb = disabled_events.clone();
        let controlled_flag = controlled_open.clone();
        let is_open_now = *controlled_flag.borrow();
        let controlled_cb = controlled_events.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                Select::new("disabled-select", vec![PickerItem::new("a", "Alpha")])
                    .aria_label("Disabled select")
                    .is_disabled(true)
                    .on_open_change(move |open, _, _| disabled_cb.borrow_mut().push(*open)),
            )
            .child(
                Select::new("controlled-select", vec![PickerItem::new("a", "Alpha")])
                    .aria_label("Controlled select")
                    .is_open(is_open_now)
                    .on_open_change(move |open, _, _| {
                        *controlled_flag.borrow_mut() = *open;
                        controlled_cb.borrow_mut().push(*open);
                    }),
            )
            .into_any_element()
    });

    click(vcx, 20., 16.);
    assert!(
        disabled_seen.borrow().is_empty(),
        "disabled Select must not emit open changes"
    );

    press(vcx, "tab");
    press(vcx, "enter");
    settle(vcx);
    assert_eq!(controlled_seen.borrow().as_slice(), [true]);
    assert!(vcx
        .debug_bounds("select-list-Name(\"controlled-select\")-panel")
        .is_some());

    press(vcx, "escape");
    settle(vcx);
    wait_ms(vcx, 150);
    assert_eq!(controlled_seen.borrow().as_slice(), [true, false]);
    assert!(vcx
        .debug_bounds("select-list-Name(\"controlled-select\")-panel")
        .is_none());
}

#[gpui::test]
fn slider_disabled_thumb_range_steps_and_invalid_bounds_behave_consistently(
    cx: &mut TestAppContext,
) {
    still();
    let range_events: Rc<RefCell<Vec<Vec<f32>>>> = Rc::new(RefCell::new(Vec::new()));
    let range_seen = range_events.clone();
    let step_events: Rc<RefCell<Vec<f32>>> = Rc::new(RefCell::new(Vec::new()));
    let step_seen = step_events.clone();
    let invalid_events: Rc<RefCell<Vec<f32>>> = Rc::new(RefCell::new(Vec::new()));
    let invalid_seen = invalid_events.clone();

    let vcx = open_host(cx, move || {
        let r_cb = range_events.clone();
        let s_cb = step_events.clone();
        let i_cb = invalid_events.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .w(px(220.))
            .child(
                Slider::new("range", 20.0)
                    .aria_label("Range slider")
                    .min_value(0.0)
                    .max_value(100.0)
                    .step(5.0)
                    .default_values([20.0, 80.0])
                    .disabled_keys([1usize])
                    .on_change_all(move |vals, _, _| r_cb.borrow_mut().push(vals.to_vec())),
            )
            .child(
                Slider::new("steps", 10.0)
                    .aria_label("Step slider")
                    .min_value(0.0)
                    .max_value(100.0)
                    .steps([10.0, 35.0, 80.0])
                    .default_value(10.0)
                    .on_change(move |v, _, _| s_cb.borrow_mut().push(*v)),
            )
            .child(
                Slider::new("invalid", 20.0)
                    .aria_label("Invalid bounds")
                    .min_value(100.0)
                    .max_value(0.0)
                    .default_value(20.0)
                    .on_change(move |v, _, _| i_cb.borrow_mut().push(*v)),
            )
            .into_any_element()
    });

    // Tab to range slider (thumb 0 is enabled; thumb 1 is disabled).
    press(vcx, "tab");
    press(vcx, "right");
    assert_eq!(range_seen.borrow().as_slice(), [vec![25.0, 80.0]]);

    // Tabbing again skips the disabled second thumb and moves to the discrete-step slider.
    press(vcx, "tab");
    press(vcx, "right");
    press(vcx, "right");
    press(vcx, "left");
    assert_eq!(step_seen.borrow().as_slice(), [35.0, 80.0, 35.0]);

    // Invalid-bounds slider never emits changes on key steps.
    press(vcx, "tab");
    press(vcx, "right");
    press(vcx, "left");
    assert!(invalid_seen.borrow().is_empty());
}
