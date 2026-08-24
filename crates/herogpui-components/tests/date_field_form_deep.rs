//!
//! DateField's hidden native input follows the v3 form contract: disabled
//! fields are unsuccessful, read-only fields remain successful, and the
//! registered value follows the live segmented state.

mod harness;

use std::{cell::Cell, rc::Rc};

use gpui::{prelude::*, AppContext, TestAppContext};
use herogpui_components::{Date, DateField, Form, InputState};

use harness::{events, open_host};

#[gpui::test]
fn enabled_date_field_submits_current_value_and_reset_restores_default(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let default = Date::new(2025, 1, 15);
    let field = DateField::new(state.clone())
        .name("date")
        .default_value(default)
        .form_field()
        .expect("named DateField");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(data.text("date").unwrap_or_default().to_string());
    });
    let submit = form.submit_handler();
    let reset = form.reset_handler();
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DateField::new(state_for_view.clone())
            .name("date")
            .default_value(default)
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["2025-01-15"]);

    cx.update(|_, cx| state.update(cx, |state, _| state.set_value("2025-02-03")));
    cx.update(|window, _| window.refresh());
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["2025-01-15", "2025-02-03"]);

    cx.update(|window, cx| reset(window, cx));
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        ["2025-01-15", "2025-02-03", "2025-01-15"]
    );
}

#[gpui::test]
fn disabled_date_field_is_omitted_from_submission(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let field = DateField::new(state.clone())
        .name("date")
        .is_disabled(true)
        .form_field()
        .expect("named DateField");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form.borrow_mut().push(
            data.text("date")
                .unwrap_or_else(|| "<omitted>".into())
                .to_string(),
        );
    });
    let submit = form.submit_handler();
    let cx = open_host(cx, move || {
        DateField::new(state.clone())
            .name("date")
            .is_disabled(true)
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["<omitted>"]);
}

#[gpui::test]
fn disabled_date_field_becomes_successful_after_rerender(cx: &mut TestAppContext) {
    let disabled = Rc::new(Cell::new(true));
    let disabled_for_view = disabled.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let field = DateField::new(state.clone())
        .name("date")
        .form_field()
        .expect("named DateField");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form.borrow_mut().push(
            data.text("date")
                .unwrap_or_else(|| "<omitted>".into())
                .to_string(),
        );
    });
    let submit = form.submit_handler();
    let cx = open_host(cx, move || {
        DateField::new(state.clone())
            .name("date")
            .is_disabled(disabled_for_view.get())
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["<omitted>"]);

    disabled.set(false);
    cx.update(|window, _| window.refresh());
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["<omitted>", "2025-01-15"]);
}

#[gpui::test]
fn read_only_date_field_remains_successful(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let field = DateField::new(state.clone())
        .name("date")
        .is_read_only(true)
        .form_field()
        .expect("named DateField");
    let form = Form::new().field(field).on_submit(move |data, _, _| {
        submitted_for_form.borrow_mut().push(
            data.text("date")
                .unwrap_or_else(|| "<omitted>".into())
                .to_string(),
        );
    });
    let submit = form.submit_handler();
    let cx = open_host(cx, move || {
        DateField::new(state.clone())
            .name("date")
            .is_read_only(true)
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), ["2025-01-15"]);
}
