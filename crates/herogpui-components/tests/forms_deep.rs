//! Behaviour tests that take Form, InputOTP and Fieldset to their edges.
//!
//! `fields.rs` and `text_fields.rs` already drive the basic collect-and-
//! validate path (a one-field invalid submit) and the basic OTP fill/walk.
//! These tests cover the configurations nobody has driven: reset semantics,
//! both `validationBehavior` halves, a form with two invalid fields, `validate`
//! closures that the component actually runs, the OTP edges (seeded values,
//! paste, arrows, no-op backspace, click-to-caret) and Fieldset's
//! composition parts.
//!
//! Geometry is derived from the components' own constants, not guessed:
//!
//! - A `Form` stacks its children `gap(16)` (form.rs). A bare `Input` is
//!   `util::FIELD_HEIGHT` = 36px, an `InputOTP` row is 40px
//!   (`input_otp.rs`: cells 38x40), and a form-level `validationErrors`
//!   message renders an `ErrorMessage` 16px tall *above* the fields, so a
//!   form that carries one shifts every control down by 32px (16 + the gap).
//! - An `InputOTP` cell *i* spans x 46·i..46·i+38 with an 8px gap, so its
//!   centre is (46·i+19, row_top + 20) — 4 cells fill x 0..152 and every
//!   click below that targets cell 0 is (19, ...) (`input_otp.rs`).
//! - An md `Button` is `Size::Md::control_height` = 36px (herogpui-core
//!   enums.rs); an md `Switch` track is 40x20 (`switch.rs`).
//! - A `Fieldset` gaps its children by 24 and its legend is a 24px line
//!   (`field.rs`); `Description` is 16px, `FieldsetGroup` gaps by 12.
//!   Legend(24) + gap(24) + 36px input → the input's centre is (60, 66);
//!   a Group of one input ends at y 84, so `Fieldset.Actions` under it sits
//!   at y 108 and its button centre is (30, 126). With a `Description`
//!   inserted, the group starts at y 88 and its input centre is (60, 106).
//!
//! Every interactive element carries a distinct id and every field its own
//! state entity, so no two components share keyed state on one page
//! (AGENTS.md's documented silent failure).
//!
//! Every test in this file runs in the default suite — none require
//! `-- --ignored`. The two tests that used to be ignored as defects (a
//! field's `validate` message, and a `minLength` violation, each failing to
//! block a native submission) now pass: `Input::render` stores its resolved
//! validity on `InputState`, and `Form`'s blocked set is the missing-required
//! fields union the fields whose stored validity is invalid. Verify with:
//!
//! ```text
//! cargo test -p herogpui-components --test forms_deep
//! ```

mod harness;

use gpui::{prelude::*, px, Focusable, SharedString, TestAppContext};
use herogpui_components::{
    Button, Description, Fieldset, FieldsetActions, FieldsetGroup, FieldsetLegend, Form, FormData,
    FormField, Input, InputOTP, InputState, OtpPattern, OtpState, Switch, ValidationBehavior,
};

use harness::{click, events, open_host, press};

/// A submission rendered the way the gallery's own Form demo renders it:
/// `name=value` pairs in registration order, values through `as_text`.
fn record_data(data: &FormData) -> String {
    data.iter()
        .map(|(name, value)| format!("{name}={}", value.as_text()))
        .collect::<Vec<_>>()
        .join(",")
}

/// The names among `required` whose value the submission lacks, sorted.
///
/// The same computation `Form::submit_handler` runs to decide the invalid
/// path, so the recorded string is what the form itself saw.
fn record_missing(data: &FormData, required: &[&str]) -> String {
    let required: Vec<SharedString> = required
        .iter()
        .map(|s| SharedString::from(s.to_string()))
        .collect();
    let mut names: Vec<String> = data
        .missing_required(&required)
        .iter()
        .map(ToString::to_string)
        .collect();
    names.sort();
    names.join(",")
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

#[gpui::test]
fn form_reset_restores_declared_defaults_then_on_reset(cx: &mut TestAppContext) {
    // v3's `onReset` fires on a native form reset, which restores each control
    // to its *default* value — the controlled state the field was seeded with,
    // not an empty string. Two fields, one with and one without a declared
    // default: reset must put the first back to "ada" and leave the second
    // exactly where typing left it (the port's documented contract — a field
    // with no default "only reports itself"). `on_reset` fires *after* the
    // restore, which the recorded values prove: the closure re-reads the
    // states, so "v1=ada" in the record can only mean the restore had run.
    let resets = events();
    let recorded_resets = resets.clone();
    let name_state = cx.new(|cx| InputState::new(cx));
    let email_state = cx.new(|cx| InputState::new(cx));
    let name_for_view = name_state.clone();
    let email_for_view = email_state.clone();
    let name_for_reset = name_state.clone();
    let email_for_reset = email_state.clone();
    let cx = open_host(cx, move || {
        let resets = resets.clone();
        // A fresh clone per render: the `on_reset` closure below captures it
        // by move, and the form is rebuilt every frame.
        let name_for_reset = name_for_reset.clone();
        let email_for_reset = email_for_reset.clone();
        let form = Form::new()
            .field(
                FormField::text(name_for_view.clone())
                    .is_required(true)
                    .default_text(name_for_view.clone(), "ada"),
            )
            .field(FormField::text(email_for_view.clone()))
            .on_reset(move |_, cx| {
                let v1 = name_for_reset.read(cx).value().to_owned();
                let v2 = email_for_reset.read(cx).value().to_owned();
                resets.borrow_mut().push(format!("v1={v1},v2={v2}"));
            });
        let reset = form.reset_handler();
        form.child(Input::new(name_for_view.clone()).name("name"))
            .child(Input::new(email_for_view.clone()).name("email"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fr-reset")
                        .label("Reset")
                        .on_press(move |_, window, cx| reset(window, cx)),
                ),
            )
            .into_any_element()
    });

    // Two bare Inputs stacked `gap(16)` (field 1 y 0..36, field 2 y 52..88)
    // and the md reset button below (y 104..140) — the same layout fields.rs
    // measures. Type a value into each so both have something a reset could
    // take away.
    click(cx, 60., 18.);
    cx.simulate_input("bob");
    click(cx, 60., 70.);
    cx.simulate_input("kept@x.y");
    click(cx, 60., 122.);

    assert_eq!(
        recorded_resets.borrow().as_slice(),
        ["v1=ada,v2=kept@x.y"],
        "on_reset must fire after the restore, and the restored state must be \
         the declared default, never empty"
    );
    let name = cx.update(|_, cx| name_state.read(cx).value().to_owned());
    assert_eq!(
        name, "ada",
        "a field that declared a default must be restored to it by reset"
    );
    let email = cx.update(|_, cx| email_state.read(cx).value().to_owned());
    assert_eq!(
        email, "kept@x.y",
        "a field without a declared default must keep its typed value"
    );

    // A second reset fires on_reset again and restores the same way.
    click(cx, 60., 122.);
    assert_eq!(
        recorded_resets.borrow().as_slice(),
        ["v1=ada,v2=kept@x.y", "v1=ada,v2=kept@x.y"],
        "every reset must restore and report, not just the first"
    );
}

#[gpui::test]
fn form_allow_field_override_submits_with_empty_required(cx: &mut TestAppContext) {
    // v3: "This behavior can be set at the form level or overridden at
    // individual field level." A required field that opts out (`aria`) shows
    // its message but does not block the form, even when the form itself is
    // `native` — the field's override is what the submit consults. The port
    // reads the override off the field's own state (`InputState` carries
    // `validationBehavior` beside `name`, input.rs), so the assertion is the
    // submit firing with the empty value, which a missing override would
    // have blocked.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(
            Input::new(state.clone())
                .name("name")
                .validation_behavior(ValidationBehavior::Allow),
        )
        .child(
            gpui::div().flex().gap(px(8.)).child(
                Button::new("fb-allow-field")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            ),
        )
        .into_any_element()
    });

    // Field y 0..36, md submit button y 52..88 — centre (60, 70).
    click(cx, 60., 70.);
    assert_eq!(
        submitted.borrow().as_slice(),
        ["name="],
        "a field that overrode to Allow must not block the native form's submit"
    );
    assert!(
        invalid.borrow().is_empty(),
        "onInvalid must not run when no field blocks the submission"
    );
}

#[gpui::test]
fn form_allow_submits_with_missing_required(cx: &mut TestAppContext) {
    // v3: `validationBehavior="aria"` "doesn't block submission" — at the
    // form level. Even an empty required field must not route the submit to
    // onInvalid.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .validation_behavior(ValidationBehavior::Allow)
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(state.clone()).name("name"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fb-allow-form")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    click(cx, 60., 70.);
    assert_eq!(
        submitted.borrow().as_slice(),
        ["name="],
        "an aria form must submit even with the required field empty"
    );
    assert!(
        invalid.borrow().is_empty(),
        "the invalid path must not run when the form allows submission"
    );
}

#[gpui::test]
fn form_two_invalid_fields_report_both_then_valid_submit(cx: &mut TestAppContext) {
    // v3's `onInvalid` fires "when the form validation fails", with the first
    // invalid field focused by default; the *data* the port hands over is what
    // identifies the fields, so the record must show both missing names at
    // once — a single-field record would hide a second failure. Then the same
    // form must accept a valid submit after the failed ones, which proves the
    // invalid path did not poison the valid one. The second field is an
    // `InputOTP` registered with `FormField::code`, exercising the OTP's
    // form integration (v3's "Form Example": `name="code"` and `isRequired`).
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let name_state = cx.new(|cx| InputState::new(cx));
    let otp_state = cx.new(|cx| OtpState::with_length(cx, 4));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .field(FormField::text(name_state.clone()).is_required(true))
            .field(FormField::code("code", otp_state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids
                    .borrow_mut()
                    .push(record_missing(data, &["name", "code"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(name_state.clone()).name("name"))
            .child(InputOTP::new(otp_state.clone()).name("code"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fb-two")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    // Layout with the OTP row in the stack: input y 0..36, OTP y 52..92
    // (40px tall), md submit button y 108..144 — centres (60, 18), (19, 72),
    // (60, 126). Two submits while both fields are empty: each must report
    // the same two missing names.
    click(cx, 60., 126.);
    click(cx, 60., 126.);
    assert_eq!(
        invalid.borrow().as_slice(),
        ["code,name", "code,name"],
        "an empty required Input AND an empty required OTP must both be \
         reported by onInvalid, on every failed submit"
    );
    assert!(
        submitted.borrow().is_empty(),
        "no submission may pass while two required fields are empty"
    );

    // Fill both fields and submit again: the same form must now take the
    // valid path and report both named values.
    click(cx, 60., 18.);
    cx.simulate_input("ada");
    click(cx, 19., 72.);
    press(cx, "1");
    press(cx, "2");
    press(cx, "3");
    press(cx, "4");
    click(cx, 60., 126.);
    assert_eq!(
        submitted.borrow().as_slice(),
        ["name=ada,code=1234"],
        "a fully filled required form must submit with every named value"
    );
}

#[gpui::test]
fn form_validation_errors_block_native_submit(cx: &mut TestAppContext) {
    // v3's `validationErrors` are "server-side validation errors ... displayed
    // immediately", and under `native` invalid fields block submission — the
    // form-level errors the port renders above the fields are the same signal.
    // Both fields are filled, so ONLY the form-level errors can be blocking;
    // the submit must route to onInvalid with the message still present.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .validation_errors(["Email already registered"])
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(state.clone()).name("name"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fb-err-native")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    // The ErrorMessage is a 16px line above the stack: message y 0..16, field
    // y 32..68, button y 84..120 — centres (60, 50) and (60, 102).
    click(cx, 60., 50.);
    cx.simulate_input("ada@x.y");
    click(cx, 60., 102.);
    assert_eq!(
        invalid.borrow().as_slice(),
        [""],
        "form-level validation errors must route a native submit to onInvalid \
         even with every field filled (the empty record is the no-missing-name \
         case: nothing was required-empty)"
    );
    assert!(
        submitted.borrow().is_empty(),
        "a native form carrying validationErrors must not submit"
    );
}

#[gpui::test]
fn form_allow_ignores_validation_errors(cx: &mut TestAppContext) {
    // The `aria` half of the same story: the message is shown ("doesn't block
    // submission"), and the submit goes through with the data — the empty
    // required field is included unblocked.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .validation_behavior(ValidationBehavior::Allow)
            .validation_errors(["Email already registered"])
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(state.clone()).name("name"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fb-err-allow")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    click(cx, 60., 102.);
    assert_eq!(
        submitted.borrow().as_slice(),
        ["name="],
        "an aria form must submit regardless of validationErrors and empty fields"
    );
    assert!(
        invalid.borrow().is_empty(),
        "the invalid path must not run when the form allows submission"
    );
}

#[gpui::test]
fn form_validate_runs_with_the_live_value(cx: &mut TestAppContext) {
    // v3's `validate` is "a function the component runs": the field evaluates
    // it against the current value and shows the returned message. The port
    // runs it in render (validation.rs), which no audit can see — prove it
    // behaviourally by recording the calls and asserting the last one carried
    // the value as typed, for both the Input and the InputOTP.
    let input_calls = events();
    let input_rec = input_calls.clone();
    let otp_calls = events();
    let otp_rec = otp_calls.clone();
    let input_state = cx.new(|cx| InputState::new(cx));
    let otp_state = cx.new(|cx| OtpState::with_length(cx, 4));
    let cx = open_host(cx, move || {
        let input_calls = input_calls.clone();
        let otp_calls = otp_calls.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                Input::new(input_state.clone())
                    .name("name")
                    .validate(move |value| {
                        input_calls.borrow_mut().push(value.to_owned());
                        (value.chars().count() < 3).then(|| "Too short".into())
                    }),
            )
            .child(InputOTP::new(otp_state.clone()).validate(move |code| {
                otp_calls.borrow_mut().push(code.to_owned());
                None
            }))
            .into_any_element()
    });

    // Input at y 0..36, OTP row at y 52..92 (40px) — centres (60, 18) and
    // (19, 72).
    click(cx, 60., 18.);
    cx.simulate_input("abc");
    assert_eq!(
        input_rec.borrow().last().map(String::as_str),
        Some("abc"),
        "the Input's validate closure must have been run with the value as \
         typed, not a stale read"
    );

    click(cx, 19., 72.);
    press(cx, "1");
    press(cx, "2");
    assert_eq!(
        otp_rec.borrow().last().map(String::as_str),
        Some("12"),
        "the InputOTP's validate closure must have been run with the assembled \
         code"
    );
}

#[gpui::test]
fn form_native_submit_focuses_first_invalid_field(cx: &mut TestAppContext) {
    // v3's `onInvalid` row: "Handler called when the form validation fails.
    // By default, the first invalid field will be focused." The reproduction
    // is a native form whose only required field is empty: after pressing
    // Submit the field must hold the focus. The port's submit_handler
    // (form.rs) only routes to onInvalid — nothing moves the focus, so the
    // assert below fails with the field unfocused.
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let invalids = invalids.clone();
        let form = Form::new()
            .field(FormField::text(state_for_view.clone()).is_required(true))
            .on_submit(|_, _, _| {})
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(state_for_view.clone()).name("name"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fd-focus-submit")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    // Field y 0..36, button y 52..88 — the button click both triggers the
    // invalid path and (in v3) hands the focus to the failed field.
    click(cx, 60., 70.);
    assert_eq!(
        invalid.borrow().as_slice(),
        ["name"],
        "the submit must have run the invalid path before focus is considered"
    );
    let focused = cx.update(|window, cx| {
        let handle = state.read(cx).focus_handle(cx);
        handle.is_focused(window)
    });
    assert!(
        focused,
        "the first invalid field must be focused after a failed native submit \
         (v3: 'By default, the first invalid field will be focused'); it is \
         not, which proves the submit path never moves the focus"
    );
}

#[gpui::test]
fn form_native_blocks_on_field_validate(cx: &mut TestAppContext) {
    // v3's Form Validation section: "Provide custom validation functions on
    // TextField components", and `validationBehavior` `native` "blocks form
    // submission on errors" — a field whose `validate` returns a message is
    // in error, so the native submit must route to onInvalid. The value here
    // satisfies `required` (it is non-empty), so the ONLY failure is the
    // validate message; the resolved validity is stored on `InputState`, and
    // the submit consults it, so onSubmit must not run and onInvalid must
    // report the blocked submission.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(
            Input::new(state.clone())
                .name("name")
                .validate(|value| (value.chars().count() < 3).then(|| "Name too short".into())),
        )
        .child(
            gpui::div().flex().gap(px(8.)).child(
                Button::new("fd-validate-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            ),
        )
        .into_any_element()
    });

    // The field's own error line (validate's "Name too short") sits between
    // the field and the button, so the button is 76..112 here, not 52..88:
    // centre (60, 94).
    click(cx, 60., 18.);
    cx.simulate_input("ab");
    click(cx, 60., 94.);
    assert!(
        submitted.borrow().is_empty(),
        "a field validate failure must block the native submit so onSubmit \
         never fires (v3: 'blocks form submission on errors') — a fire \
         means the field's stored validity did not block the native submit"
    );
    assert_eq!(
        invalid.borrow().as_slice(),
        [""],
        "a field validate failure must route the native submit to onInvalid"
    );
}

#[gpui::test]
fn form_native_blocks_on_min_length(cx: &mut TestAppContext) {
    // v3's Form Validation section: "Use built-in HTML5 validation attributes
    // (required, minLength, pattern, etc.)". Under `native` a `minLength`
    // violation is a field error like any other: submission is blocked and
    // onInvalid fires — the violation is resolved into the field's
    // `InputState` validity, which the submit consults, so onSubmit must not
    // run with the 2-char value and onInvalid must report it.
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let form = Form::new()
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_missing(data, &["name"]));
            });
        let submit = form.submit_handler();
        form.child(Input::new(state.clone()).name("name").min_length(5))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("fd-minlen-submit")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("ab");
    click(cx, 60., 70.);
    assert!(
        submitted.borrow().is_empty(),
        "a minLength violation must block the native submit — a fire means \
         the field's stored validity did not block the native submit"
    );
    assert_eq!(
        invalid.borrow().as_slice(),
        [""],
        "a minLength violation must route the native submit to onInvalid"
    );
}

// ---------------------------------------------------------------------------
// InputOTP
// ---------------------------------------------------------------------------

#[gpui::test]
fn input_otp_partial_seed_completes_once_and_again_after_correction(cx: &mut TestAppContext) {
    // A caller-seeded partial value (v3's `value="12"` controlled seed) must
    // leave the next digit for the first empty slot — `set_code` pads with
    // blanks and parks the cursor at the code's end (input_otp.rs), so typing
    // continues at cell 2. `onComplete` (v3: "Handler called when all slots
    // are filled") must fire exactly once for the completion, and again only
    // after the code is broken and re-completed. The second "1234" is the
    // correction: backspace removes the last digit, typing it back completes
    // the field a second time.
    let changes = events();
    let recorded = changes.clone();
    let completes = events();
    let completed = completes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    cx.update(|cx| state.update(cx, |s, _| s.set_code("12")));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let completes = completes.clone();
        InputOTP::new(state_for_view.clone())
            .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
            .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    // Cell 0 centre (19, 20) focuses the row; the seeded cursor is already at
    // cell 2, so the first pressed digit lands there.
    click(cx, 19., 20.);
    press(cx, "3");
    press(cx, "4");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["123", "1234"],
        "typing from a seeded '12' must grow the code through the empty slots"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["1234"],
        "on_complete must fire exactly once when the seeded code is completed"
    );

    // The correction: clear the last slot, refill it — on_complete again.
    press(cx, "backspace");
    press(cx, "4");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["123", "1234", "123", "1234"],
        "backspace then a digit must each report the assembled code"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["1234", "1234"],
        "re-completing the code after a correction must fire on_complete again"
    );
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(code, "1234", "the OtpState must hold the corrected code");
}

#[gpui::test]
fn input_otp_seed_longer_than_slots_truncates(cx: &mut TestAppContext) {
    // Seeding more characters than there are cells must drop the overflow
    // (`set_code` reads one char per cell), and the cursor must rest on the
    // last cell so the next keystroke overwrites rather than panicking off
    // the end. `onComplete` must not fire from the seed itself — it is an
    // event, not a state assertion — but must fire on the user's next edit.
    let completes = events();
    let completed = completes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    cx.update(|cx| state.update(cx, |s, _| s.set_code("1234567")));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let completes = completes.clone();
        InputOTP::new(state_for_view.clone())
            .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    let seeded = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        seeded, "1234",
        "seeding 7 chars into 4 cells must keep only the first 4"
    );

    click(cx, 19., 20.);
    press(cx, "5");
    assert_eq!(
        completed.borrow().as_slice(),
        ["1235"],
        "on_complete must fire only on the user's keystroke, never from the seed"
    );
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "1235",
        "the keystroke after an over-long seed must overwrite the last cell"
    );
}

#[gpui::test]
fn input_otp_arrows_clamp_at_both_ends(cx: &mut TestAppContext) {
    // The arrows move the caret within the cells (`left`/`right` in
    // input_otp.rs); both ends clamp — Left from cell 0 stays, Right from the
    // last cell stays. Caret position is private to `OtpState`, so the probe
    // is where the next digit lands: a clamped caret writes to the cell it
    // could not leave, and a wrap would write somewhere else.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        InputOTP::new(state_for_view.clone())
            .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    // Click cell 0 (19, 20); Left from the first cell clamps, so the "1"
    // lands in cell 0 and the caret steps to cell 1.
    click(cx, 19., 20.);
    press(cx, "left");
    press(cx, "1");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1"],
        "Left from cell 0 must not move: the '1' still landed in cell 0"
    );

    // Right to cell 2; "2" lands there — cells are now 1, _, 2, _. Then
    // Right twice from cell 3 (the caret reached it by typing): the first
    // clamps (cursor+1 == len) and the second cannot overrun at all. A "3"
    // typed from the clamped caret overwrites cell 3 in place — if either
    // Right had left the last cell the digit would land elsewhere, and if it
    // had walked past the end the index would panic.
    press(cx, "right");
    press(cx, "2");
    press(cx, "right");
    press(cx, "right");
    press(cx, "3");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "12", "123"],
        "Right must step within the cells and clamp at the last one: the '3' \
         overwrote the final cell instead of falling off the row"
    );

    // Left four times from the last cell walks to 0 and clamps on the fourth
    // press; a "5" typed from there overwrites cell 0 — if Left had wrapped
    // past the start, the digit would land at the other end instead.
    press(cx, "left");
    press(cx, "left");
    press(cx, "left");
    press(cx, "left");
    press(cx, "5");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "12", "123", "523"],
        "Left must walk to cell 0 and clamp there: the '5' overwrote cell 0"
    );
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(code, "523", "the OtpState must hold the clamped arithmetic");
}

#[gpui::test]
fn input_otp_paste_transformer_runs_before_fill(cx: &mut TestAppContext) {
    // v3 documents `pasteTransformer` as the hook that rewrites pasted text
    // ("Transform pasted text (e.g., remove hyphens)"). The port applies it
    // before the slots fill (input_otp.rs), so a transformer that maps "1" to
    // "A" must be visible in the code — the transformer ran, not the raw
    // clipboard. The Alphanumeric pattern admits the result, and a full paste
    // fires onComplete exactly as a typed code does.
    let completes = events();
    let completed = completes.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let completes = completes.clone();
        let changes = changes.clone();
        InputOTP::new(state_for_view.clone())
            .pattern(OtpPattern::Alphanumeric)
            .paste_transformer(|text| text.replace('1', "A"))
            .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
            .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    // The test platform's clipboard is an in-memory slot the test writes
    // (TestAppContext::write_to_clipboard, gpui test_context.rs), and the
    // component reads the same slot on Ctrl+V.
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("1234".to_owned()));
    click(cx, 19., 20.);
    press(cx, "ctrl-v");
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "A234",
        "the paste transformer must rewrite the clipboard text before the \
         slots fill"
    );
    assert_eq!(
        changed.borrow().as_slice(),
        ["A234"],
        "a valid transformed paste must fire on_change exactly once"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["A234"],
        "a paste that fills every slot must fire on_complete with the \
         transformed code"
    );
}

/// Pinned `input-otp` reports an accepted paste through `onChange`, then fires
/// `onComplete` only when that paste transitions the field from partial to
/// full. Empty/rejected pastes do neither, and an accepted same-value paste on
/// an already-complete field still reports the value without completing twice.
#[gpui::test]
fn input_otp_paste_callbacks_follow_acceptance_and_completion_transitions(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changed = for_view.clone();
        let completed = for_view.clone();
        InputOTP::new(state_for_view.clone())
            .pattern(OtpPattern::Digits)
            .on_change(move |code, _, _| changed.borrow_mut().push(format!("change:{code}")))
            .on_complete(move |code, _, _| {
                completed.borrow_mut().push(format!("complete:{code}"));
            })
            .into_any_element()
    });

    click(cx, 19., 20.);
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("1".to_owned()));
    press(cx, "ctrl-v");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:1"],
        "a partial paste changes the value but must not complete"
    );

    recorded.borrow_mut().clear();
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("letters".to_owned()));
    press(cx, "ctrl-v");
    assert!(
        recorded.borrow().is_empty(),
        "a paste with no accepted characters must report neither callback"
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).code()),
        "1",
        "a paste rejected by the explicit digits pattern must leave the value unchanged"
    );

    cx.update(|_, cx| state.update(cx, |state, _| state.set_code("1234")));
    recorded.borrow_mut().clear();
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("4".to_owned()));
    press(cx, "ctrl-v");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:1234"],
        "an accepted same-value paste reports on_change but does not complete twice"
    );

    cx.update(|_, cx| state.update(cx, |state, _| state.set_code("123")));
    recorded.borrow_mut().clear();
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("4".to_owned()));
    press(cx, "ctrl-v");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:1234", "complete:1234"],
        "the transition to full must report change before completion"
    );
}

/// `slot` is a GPUI render extension that receives the slot index and current
/// character. The closure must receive each slot independently and update
/// after editing rather than remaining a construction-time snapshot.
#[gpui::test]
fn input_otp_slot_content_tracks_each_character(cx: &mut TestAppContext) {
    let snapshots = events();
    let snapshots_for_view = snapshots.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 2));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let snapshots = snapshots_for_view.clone();
        InputOTP::new(state_for_view.clone())
            .slot(move |index, value| {
                snapshots
                    .borrow_mut()
                    .push(format!("{index}:{}", value.unwrap_or('_')));
                gpui::div()
                    .child(value.unwrap_or('_').to_string())
                    .into_any_element()
            })
            .into_any_element()
    });

    assert!(
        snapshots.borrow().ends_with(&["0:_".into(), "1:_".into()]),
        "the initial frame must identify both empty slots"
    );

    click(cx, 19., 20.);
    press(cx, "7");
    assert!(
        snapshots.borrow().ends_with(&["0:7".into(), "1:_".into()]),
        "editing the first slot must update only that slot's render value"
    );
}

#[gpui::test]
fn input_otp_paste_subjects_to_pattern(cx: &mut TestAppContext) {
    // v3's `pattern` prop: "Regex pattern for allowed characters". The paste
    // branch of the port skips non-alphanumeric characters and uppercases the
    // rest, but never consults `pattern`, so a digits-only field ("1234" in
    // the clipboard) accepts "12ab34" as "12AB" — the letters should have
    // been dropped and the digits "34" should have filled the remaining
    // cells, leaving "1234". The paste path is the only one that can land a
    // letter in a digits field: the keystroke path checks `pattern.accepts`.
    let completes = events();
    let completed = completes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let completes = completes.clone();
        InputOTP::new(state_for_view.clone())
            .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    cx.write_to_clipboard(gpui::ClipboardItem::new_string("12ab34".to_owned()));
    click(cx, 19., 20.);
    press(cx, "ctrl-v");
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "1234",
        "pasting into a digits field must drop the letters and fill the \
         digits"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["1234"],
        "the completed code must be the pattern-filtered one"
    );
}

#[gpui::test]
fn input_otp_backspace_on_empty_first_cell_fires_no_change(cx: &mut TestAppContext) {
    // An empty field, cursor on cell 0, presses Backspace. Nothing can be
    // cleared; v3's `onChange` is "Handler called when the value changes", so
    // no change event may fire. The port's backspace branch (input_otp.rs)
    // always reports after the update, even when the update was a no-op.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        InputOTP::new(state_for_view.clone())
            .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    click(cx, 19., 20.);
    press(cx, "backspace");
    assert!(
        recorded.borrow().is_empty(),
        "a backspace that clears nothing must not report a change — the \
         handler fired with the unchanged empty code instead"
    );
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(code, "", "the empty code must stay empty");
}

#[gpui::test]
fn input_otp_click_moves_the_caret(cx: &mut TestAppContext) {
    // A full code, then a click on cell 0 and one digit: the caret must have
    // moved to the clicked slot, so the digit overwrites cell 0 ("9234").
    // The port's `on_mouse_down` calls `window.focus` only and leaves the
    // cursor at the last cell, so the digit overwrites cell 3 instead
    // ("1239"). The code string discriminates: both writes are 'complete'
    // length-4 codes, only the caret position differs.
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        InputOTP::new(state_for_view.clone()).into_any_element()
    });

    click(cx, 19., 20.);
    press(cx, "1");
    press(cx, "2");
    press(cx, "3");
    press(cx, "4");
    // Cell 0's centre again — this click must re-home the caret.
    click(cx, 19., 20.);
    press(cx, "9");
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "9234",
        "clicking a slot must move the caret there so the next digit \
         overwrites that cell — the digit landed on the last cell instead"
    );
}

#[gpui::test]
fn input_otp_disabled_click_does_not_focus(cx: &mut TestAppContext) {
    // v3's `isDisabled` ("Whether the input is disabled") means the control
    // answers no pointer: a click must neither focus it nor start the caret,
    // and the keys must change nothing. The port's `on_mouse_down` is
    // attached unconditionally (input_otp.rs), but the disabled handle is
    // never `track_focus`'d, so the click cannot stick — the focus stays
    // where it was and the keystroke lands nowhere. This is a regression
    // guard for that path, not a defect.
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        InputOTP::new(state_for_view.clone())
            .is_disabled(true)
            .into_any_element()
    });

    click(cx, 19., 20.);
    let focused = cx.update(|window, cx| {
        let handle = state.read(cx).focus_handle(cx);
        handle.is_focused(window)
    });
    assert!(
        !focused,
        "a click on a disabled OTP must not focus it (pointer-events-none)"
    );
    press(cx, "1");
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(code, "", "a disabled OTP must refuse the keystroke too");
}

// ---------------------------------------------------------------------------
// Fieldset
// ---------------------------------------------------------------------------

#[gpui::test]
fn fieldset_parts_compose_and_controls_stay_interactive(cx: &mut TestAppContext) {
    // v3's `Fieldset.Legend`/`.Group`/`.Actions` compose (the docs' Basic
    // anatomy: Legend, Description, Group of fields, Actions) and the controls
    // inside keep answering — a container imposes nothing. One input, one
    // switch and one button, each with its own recorder: a click must reach
    // exactly its own control, which also proves the parts above them (the
    // legend, the description) do not swallow the hits.
    let changes = events();
    let recorded = changes.clone();
    let toggles = events();
    let toggled = toggles.clone();
    let presses = events();
    let pressed = presses.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let toggles = toggles.clone();
        let presses = presses.clone();
        Fieldset::new()
            .child(FieldsetLegend::new("Profile settings"))
            .child(Description::new("Update your profile information."))
            .child(
                FieldsetGroup::new()
                    .child(
                        Input::new(state_for_view.clone()).on_change(move |text, _, _| {
                            changes.borrow_mut().push(text.to_owned());
                        }),
                    )
                    .child(Switch::new("fs-switch").on_change(move |checked, _, _| {
                        toggles.borrow_mut().push(format!("change:{checked}"));
                    })),
            )
            .child(
                FieldsetActions::new().child(
                    Button::new("fs-save")
                        .label("Save")
                        .on_press(move |_, _, _| presses.borrow_mut().push("save".to_owned())),
                ),
            )
            .into_any_element()
    });

    // Legend 24px, gap 24, Description 16px, gap 24 → the group starts at
    // y 88: input y 88..124 (centre 60, 106), switch y 136..156 (track 40x20,
    // centre 20, 146), group height 68 → Actions y 180, md button 180..216
    // (centre 30, 198). All derived from field.rs's gaps and the component
    // heights above.
    click(cx, 60., 106.);
    cx.simulate_input("ada");
    click(cx, 20., 146.);
    click(cx, 30., 198.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ad", "ada"],
        "the input inside the composed fieldset must answer typing"
    );
    assert_eq!(
        toggled.borrow().as_slice(),
        ["change:true"],
        "the switch inside the group must answer its own click"
    );
    assert_eq!(
        pressed.borrow().as_slice(),
        ["save"],
        "the button inside Actions must answer its own click"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "ada", "the InputState must hold the typed value");
}

#[gpui::test]
fn fieldset_nested_fieldset_keeps_inner_controls_alive(cx: &mut TestAppContext) {
    // A Fieldset nested in a Fieldset (native HTML allows it) must not break
    // the controls inside either level: both fields keep their own state and
    // their own recorder, so a click that lands in the inner field's band
    // must not leak to the outer one (or vice versa).
    let outer_changes = events();
    let outer_recorded = outer_changes.clone();
    let inner_changes = events();
    let inner_recorded = inner_changes.clone();
    let outer_state = cx.new(|cx| InputState::new(cx));
    let inner_state = cx.new(|cx| InputState::new(cx));
    let outer_for_view = outer_state.clone();
    let inner_for_view = inner_state.clone();
    let cx = open_host(cx, move || {
        let outer_changes = outer_changes.clone();
        let inner_changes = inner_changes.clone();
        Fieldset::new()
            .child(FieldsetLegend::new("Outer"))
            .child(
                FieldsetGroup::new().child(Input::new(outer_for_view.clone()).on_change(
                    move |text, _, _| {
                        outer_changes.borrow_mut().push(text.to_owned());
                    },
                )),
            )
            .child(Fieldset::new().child(FieldsetLegend::new("Inner")).child(
                FieldsetGroup::new().child(Input::new(inner_for_view.clone()).on_change(
                    move |text, _, _| {
                        inner_changes.borrow_mut().push(text.to_owned());
                    },
                )),
            ))
            .into_any_element()
    });

    // Outer legend 0..24, gap 24 → outer group y 48: outer input 48..84
    // (centre 60, 66). The outer group ends at y 84, gap 24 → inner fieldset
    // y 108: inner legend 108..132, gap 24 → inner group y 156: inner input
    // 156..192 (centre 60, 174).
    click(cx, 60., 66.);
    cx.simulate_input("outer");
    click(cx, 60., 174.);
    cx.simulate_input("inner");
    assert_eq!(
        outer_recorded.borrow().as_slice(),
        ["o", "ou", "out", "oute", "outer"],
        "the outer field must keep typing through the nested fieldset"
    );
    assert_eq!(
        inner_recorded.borrow().as_slice(),
        ["i", "in", "inn", "inne", "inner"],
        "the inner field must keep typing inside the nested fieldset"
    );
    let outer_value = cx.update(|_, cx| outer_state.read(cx).value().to_owned());
    let inner_value = cx.update(|_, cx| inner_state.read(cx).value().to_owned());
    assert_eq!(
        outer_value, "outer",
        "the outer InputState must hold its text"
    );
    assert_eq!(
        inner_value, "inner",
        "the inner InputState must hold its text"
    );
}

#[gpui::test]
fn fieldset_actions_submit_drives_the_forms_submit(cx: &mut TestAppContext) {
    // v3's Basic Fieldset wraps the whole group in a Form and puts its
    // submit button in `Fieldset.Actions`; the button must drive the form's
    // submit handler — the collection crosses the form → fieldset → actions
    // boundary and the named value lands on onSubmit.
    let submits = events();
    let submitted = submits.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let form = Form::new()
            .field(FormField::text(state.clone()).is_required(true))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            });
        let submit = form.submit_handler();
        form.child(
            Fieldset::new()
                .child(FieldsetLegend::new("Billing address"))
                .child(FieldsetGroup::new().child(Input::new(state.clone()).name("street")))
                .child(
                    FieldsetActions::new().child(
                        Button::new("fda-submit")
                            .label("Save")
                            .on_press(move |_, window, cx| submit(window, cx)),
                    ),
                ),
        )
        .into_any_element()
    });

    // Legend 0..24, gap 24 → group y 48: input 48..84 (centre 60, 66);
    // group ends at y 84, gap 24 → Actions y 108: md button 108..144
    // (centre 30, 126).
    click(cx, 60., 66.);
    cx.simulate_input("5th Ave");
    click(cx, 30., 126.);
    assert_eq!(
        submitted.borrow().as_slice(),
        ["street=5th Ave"],
        "the button inside Fieldset.Actions must drive the Form's onSubmit \
         with the named field"
    );
}

#[gpui::test]
fn fieldset_actions_reset_drives_the_forms_reset(cx: &mut TestAppContext) {
    // The reset half of the same anatomy: a `type="reset"` button inside
    // `Fieldset.Actions` restores the declared default through the Form and
    // fires onReset — the composition must not swallow either.
    let resets = events();
    let recorded_resets = resets.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let state_for_reset = state.clone();
    let cx = open_host(cx, move || {
        let resets = resets.clone();
        // A fresh clone per render: the `on_reset` closure below captures it
        // by move, and the form is rebuilt every frame.
        let state_for_reset = state_for_reset.clone();
        let form = Form::new()
            .field(
                FormField::text(state_for_view.clone())
                    .is_required(true)
                    .default_text(state_for_view.clone(), "Main St"),
            )
            .on_reset(move |_, cx| {
                let v = state_for_reset.read(cx).value().to_owned();
                resets.borrow_mut().push(format!("v={v}"));
            });
        let reset = form.reset_handler();
        form.child(
            Fieldset::new()
                .child(FieldsetLegend::new("Billing address"))
                .child(
                    FieldsetGroup::new().child(Input::new(state_for_view.clone()).name("street")),
                )
                .child(
                    FieldsetActions::new().child(
                        Button::new("fda-reset")
                            .label("Cancel")
                            .on_press(move |_, window, cx| reset(window, cx)),
                    ),
                ),
        )
        .into_any_element()
    });

    click(cx, 60., 66.);
    cx.simulate_input("5th Ave");
    click(cx, 30., 126.);
    assert_eq!(
        recorded_resets.borrow().as_slice(),
        ["v=Main St"],
        "the reset button inside Actions must restore the declared default and \
         report it through onReset"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "Main St",
        "the InputState must hold the restored value"
    );
}
