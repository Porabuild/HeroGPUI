//! Deeper keyboard and read-only behaviour for Switch, RadioGroup and
//! ToggleButtonGroup.
//!
//! The prop surface comes from HeroUI v3's API tables. Inherited behaviour is
//! pinned to the versions HeroUI v3.2.4 uses: react-aria 3.51.0,
//! react-stately 3.49.0 and React Aria Components 1.16.0. In those sources:
//!
//! - `useToggleState` ignores updates while read-only, but `useToggle` keeps
//!   the input focusable because only `isDisabled` reaches `disabled`.
//! - `useRadioGroup` moves DOM focus with the four arrow keys and calls
//!   `setSelectedValue`; `useRadioGroupState` rejects that selection update
//!   while read-only. Its shortcut table has no Home or End entry.
//! - `useToggleGroupState` defaults `selectionMode` to `single` and enforces
//!   `disallowEmptySelection`. `useToggleButtonGroup` delegates focus to
//!   `useToolbar`, whose pinned implementation handles only the orientation's
//!   own arrow axis and moves Tab to the group's edge so normal tabbing leaves
//!   the group.
//!
//! These tests drive those paths through gpui's real headless window. Static
//! audits cannot prove that a read-only control keeps focus, that an arrow
//! moved it, or that Tab escaped a composite widget.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{prelude::*, px, SharedString, TestAppContext, VisualTestContext};
use herogpui_components::{
    Button, Form, Orientation, RadioGroup, RadioOption, SelectionMode, Size, Switch, ToggleButton,
    ToggleButtonGroup, Toolbar,
};

use harness::{click, events, open_host, press};

fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

fn joined(keys: &[SharedString]) -> String {
    keys.iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn three_toggles(prefix: &str) -> [ToggleButton; 3] {
    [
        ToggleButton::new(gpui::ElementId::Name(format!("{prefix}-bold").into()))
            .key("bold")
            .label("Bold"),
        ToggleButton::new(gpui::ElementId::Name(format!("{prefix}-italic").into()))
            .key("italic")
            .label("Italic"),
        ToggleButton::new(gpui::ElementId::Name(format!("{prefix}-underline").into()))
            .key("underline")
            .label("Underline"),
    ]
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

#[gpui::test]
fn switch_read_only_ignores_pointer_press(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Switch::new("readonly-pointer")
            .is_read_only(true)
            .on_change(move |selected, _, _| changes.borrow_mut().push(selected.to_string()))
            .into_any_element()
    });

    // The default md track is 40x20, so its centre is (20, 10).
    click(cx, 20., 10.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [] as [&str; 0],
        "a read-only switch must ignore a pointer press"
    );
}

#[gpui::test]
fn switch_read_only_stays_focusable_but_ignores_space(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let focused = Rc::new(RefCell::new(false));
    let focus_seen = focused.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let focused = focused.clone();
        Switch::new("readonly-keyboard")
            .is_read_only(true)
            .content(move |state| {
                *focused.borrow_mut() = state.is_focused;
                gpui::div().child("Read only").into_any_element()
            })
            .on_change(move |selected, _, _| changes.borrow_mut().push(selected.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    assert!(
        *focus_seen.borrow(),
        "read-only is not disabled: Tab must still focus the switch"
    );

    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        [] as [&str; 0],
        "Space on a focused read-only switch must not toggle it"
    );
}

#[gpui::test]
fn switch_content_receives_complete_field_state(cx: &mut TestAppContext) {
    let states = events();
    let recorded = states.clone();
    let _cx = open_host(cx, move || {
        let states = states.clone();
        Switch::new("switch-complete-state")
            .is_selected(true)
            .is_read_only(true)
            .is_required(true)
            .validate(|_| Some("Invalid selection".into()))
            .content(move |state| {
                states.borrow_mut().push(format!(
                    "selected={}:readonly={}:invalid={}:required={}",
                    state.is_selected, state.is_read_only, state.is_invalid, state.is_required
                ));
                gpui::div().into_any_element()
            })
            .into_any_element()
    });

    assert!(recorded
        .borrow()
        .iter()
        .any(|state| { state == "selected=true:readonly=true:invalid=true:required=true" }));
}

#[gpui::test]
fn switch_field_error_is_outside_the_clickable_content(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Switch::new("switch-field-error")
            .validation_errors(["Selection is unavailable"])
            .on_change(move |selected, _, _| changes.borrow_mut().push(selected.to_string()))
            .into_any_element()
    });

    // The 16px FieldError line starts after the 20px track plus the 4px
    // field gap. It is a sibling of Switch.Content and must not toggle it.
    click(cx, 60., 31.);
    assert!(recorded.borrow().is_empty());

    click(cx, 20., 10.);
    assert_eq!(recorded.borrow().as_slice(), ["true"]);
}

#[gpui::test]
fn switch_form_reads_live_uncontrolled_value_and_reset(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let submits = events.clone();
        let switch = Switch::new("live-switch-form")
            .name("notifications")
            .value("enabled")
            .on_change(move |selected, _, _| {
                changes.borrow_mut().push(format!("change:{selected}"));
            });
        let form = Form::new()
            .field(switch.form_field().expect("named switch field"))
            .on_submit(move |data, _, _| {
                submits.borrow_mut().push(format!(
                    "submit:{}",
                    data.text("notifications")
                        .map_or_else(|| "omitted".to_owned(), |value| value.to_string())
                ));
            });
        let submit = form.submit_handler();
        let reset = form.reset_handler();
        form.child(switch)
            .child(
                Button::new("live-switch-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .child(
                Button::new("live-switch-reset")
                    .label("Reset")
                    .on_press(move |_, window, cx| reset(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    press(cx, "tab");
    press(cx, "space");
    press(cx, "tab");
    press(cx, "space");
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");

    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true", "submit:enabled", "submit:omitted"],
        "submission must read the current uncontrolled state and reset must restore defaultSelected"
    );
}

#[gpui::test]
fn controlled_switch_reset_reports_default_selected(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let switch = Switch::new("controlled-switch-reset")
            .name("notifications")
            .is_selected(true)
            .default_selected(false)
            .on_change(move |selected, _, _| changes.borrow_mut().push(selected.to_string()));
        let form = Form::new().field(switch.form_field().expect("named switch field"));
        let reset = form.reset_handler();
        form.child(switch)
            .child(
                Button::new("controlled-switch-reset-button")
                    .label("Reset")
                    .on_press(move |_, window, cx| reset(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["false"]);
}

#[gpui::test]
fn controlled_switch_form_waits_for_owner_to_accept_change(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let submits = events.clone();
        let switch = Switch::new("controlled-switch-form")
            .name("notifications")
            .value("enabled")
            .is_selected(false)
            .on_change(move |selected, _, _| {
                changes.borrow_mut().push(format!("change:{selected}"));
            });
        let form = Form::new()
            .field(switch.form_field().expect("named switch field"))
            .on_submit(move |data, _, _| {
                submits.borrow_mut().push(format!(
                    "submit:{}",
                    data.text("notifications")
                        .map_or_else(|| "omitted".to_owned(), |value| value.to_string())
                ));
            });
        let submit = form.submit_handler();
        form.child(switch)
            .child(
                Button::new("controlled-switch-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true", "submit:omitted"],
        "a controlled press only reports intent; form data changes after the owner renders it"
    );
}

#[gpui::test]
fn disabled_switch_is_not_a_successful_form_control(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let switch = Switch::new("disabled-switch-snapshot")
            .name("notifications")
            .value("enabled")
            .is_selected(true)
            .is_disabled(true);
        let form = Form::new().field(switch.form_field().expect("named switch field"));
        assert!(
            form.data(cx).get("notifications").is_none(),
            "disabled omission must hold before first render"
        );
    });

    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let submits = events.clone();
        let switch = Switch::new("disabled-switch-form")
            .name("notifications")
            .value("enabled")
            .is_selected(true)
            .is_disabled(true);
        let form = Form::new()
            .field(switch.form_field().expect("named switch field"))
            .on_submit(move |data, _, _| {
                submits.borrow_mut().push(
                    data.text("notifications")
                        .map_or_else(|| "omitted".to_owned(), |value| value.to_string()),
                );
            });
        let submit = form.submit_handler();
        form.child(switch)
            .child(
                Button::new("disabled-switch-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["omitted"]);
}

#[gpui::test]
fn invalid_switch_blocks_form_and_receives_focus(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let invalids = events.clone();
        let submits = events.clone();
        let switch = Switch::new("invalid-switch-form")
            .name("notifications")
            .validate(|selected| (!selected).then(|| "Enable notifications".into()))
            .on_change(move |selected, _, _| {
                changes.borrow_mut().push(format!("change:{selected}"));
            });
        let form = Form::new()
            .field(switch.form_field().expect("named switch field"))
            .on_invalid(move |_, _, _| invalids.borrow_mut().push("invalid".to_owned()))
            .on_submit(move |_, _, _| submits.borrow_mut().push("submit".to_owned()));
        let submit = form.submit_handler();
        form.child(switch)
            .child(
                Button::new("invalid-switch-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["invalid", "change:true"]);
}

// ---------------------------------------------------------------------------
// RadioGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn radio_indicator_receives_field_state_and_reacts_to_selection(cx: &mut TestAppContext) {
    let states = events();
    let recorded = states.clone();
    let cx = open_host(cx, move || {
        let states = states.clone();
        RadioGroup::new(
            "custom-radio-indicator",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .default_value("free")
        .is_required(true)
        .error_message("Choose a plan")
        .indicator(move |label, state| {
            states.borrow_mut().push(format!(
                "{label}:selected={}:invalid={}:required={}",
                state.is_selected, state.is_invalid, state.is_required
            ));
            gpui::div().into_any_element()
        })
        .into_any_element()
    });

    assert!(recorded
        .borrow()
        .iter()
        .any(|state| { state == "Free plan:selected=true:invalid=true:required=true" }));
    assert!(recorded
        .borrow()
        .iter()
        .any(|state| { state == "Pro plan:selected=false:invalid=true:required=true" }));

    click(cx, 48., 46.);
    flush_frame(cx);
    assert!(recorded
        .borrow()
        .iter()
        .any(|state| { state == "Pro plan:selected=true:invalid=true:required=true" }));
}

#[gpui::test]
fn per_radio_field_error_is_outside_the_clickable_content(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        RadioGroup::new(
            "radio-option-error",
            vec![
                RadioOption::new("Free plan")
                    .value("free")
                    .error_message("Unavailable in this region"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()))
        .into_any_element()
    });

    // The error is a sibling below Radio.Content and must not inherit its
    // click listener.
    click(cx, 80., 31.);
    assert!(recorded.borrow().is_empty());

    // Its 16px line plus the 4px radio gap moves the second option down.
    click(cx, 48., 60.);
    assert_eq!(recorded.borrow().as_slice(), ["pro"]);
}

#[gpui::test]
fn invalid_radio_group_blocks_form_and_focuses_its_roving_stop(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let invalids = events.clone();
        let submits = events.clone();
        let group = RadioGroup::new(
            "invalid-radio-form",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .name("plan")
        .error_message("Choose a plan")
        .on_change(move |value, _, _| changes.borrow_mut().push(format!("change:{value}")));
        let form = Form::new()
            .field(group.form_field().expect("named radio field"))
            .on_invalid(move |_, _, _| invalids.borrow_mut().push("invalid".to_owned()))
            .on_submit(move |_, _, _| submits.borrow_mut().push("submit".to_owned()));
        let submit = form.submit_handler();
        form.child(group)
            .child(
                Button::new("invalid-radio-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["invalid", "change:free"],
        "native invalid submission must focus the group's current radio rather than submit"
    );
}

#[gpui::test]
fn disabled_radio_group_is_not_a_successful_form_control(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let group = RadioGroup::new(
            "disabled-radio-snapshot",
            vec![RadioOption::new("Free plan").value("free")],
        )
        .name("plan")
        .default_value("free")
        .is_disabled(true);
        let form = Form::new().field(group.form_field().expect("named radio field"));
        assert!(
            form.data(cx).get("plan").is_none(),
            "disabled omission must be true before the first render as well as after it"
        );
    });

    let submits = events();
    let recorded = submits.clone();
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let group = RadioGroup::new(
            "disabled-radio-form",
            vec![RadioOption::new("Free plan").value("free")],
        )
        .name("plan")
        .default_value("free")
        .is_disabled(true);
        let form = Form::new()
            .field(group.form_field().expect("named radio field"))
            .on_submit(move |data, _, _| {
                submits.borrow_mut().push(
                    data.text("plan")
                        .map_or_else(|| "omitted".to_owned(), |value| value.to_string()),
                );
            });
        let submit = form.submit_handler();
        form.child(group)
            .child(
                Button::new("disabled-radio-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["omitted"]);
}

#[gpui::test]
fn radio_group_reset_restores_the_uncontrolled_default(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let changes = events.clone();
        let submits = events.clone();
        let group = RadioGroup::new(
            "reset-radio-form",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .name("plan")
        .default_value("free")
        .on_change(move |value, _, _| changes.borrow_mut().push(format!("change:{value}")));
        let form = Form::new()
            .field(group.form_field().expect("named radio field"))
            .on_submit(move |data, _, _| {
                submits.borrow_mut().push(format!(
                    "submit:{}",
                    data.text("plan").expect("selected radio value")
                ));
            });
        let reset = form.reset_handler();
        let submit = form.submit_handler();
        form.child(group)
            .child(
                Button::new("reset-radio-reset")
                    .label("Reset")
                    .on_press(move |_, window, cx| reset(window, cx)),
            )
            .child(
                Button::new("reset-radio-submit")
                    .label("Submit")
                    .on_press(move |_, window, cx| submit(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "tab");
    press(cx, "space");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["change:pro", "submit:free"]);
}

#[gpui::test]
fn controlled_radio_reset_reports_the_default_to_its_owner(cx: &mut TestAppContext) {
    let selected = Rc::new(RefCell::new(SharedString::from("pro")));
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let group = RadioGroup::new(
            "controlled-reset-radio",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .name("plan")
        .value(selected.borrow().as_ref())
        .default_value("free")
        .on_change({
            let selected = selected.clone();
            let changes = changes.clone();
            move |value, _, _| {
                *selected.borrow_mut() = value.clone();
                changes.borrow_mut().push(format!("change:{value}"));
            }
        });
        let form = Form::new().field(group.form_field().expect("named radio field"));
        let reset = form.reset_handler();
        form.child(group)
            .child(
                Button::new("controlled-reset-radio-button")
                    .label("Reset")
                    .on_press(move |_, window, cx| reset(window, cx)),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["change:free"]);
}

#[gpui::test]
fn radio_group_read_only_arrows_move_focus_without_selecting(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let focused = Rc::new(RefCell::new(String::new()));
    let focus_seen = focused.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let focused = focused.clone();
        RadioGroup::new(
            "readonly-radios",
            vec!["One".into(), "Two".into(), "Three".into()],
        )
        .default_value("One")
        .is_read_only(true)
        .option_content(move |label, state| {
            if state.is_focused {
                *focused.borrow_mut() = label.to_string();
            }
            gpui::div().child(label.to_string()).into_any_element()
        })
        .on_change(move |index, _, _| changes.borrow_mut().push(index.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    flush_frame(cx);

    assert_eq!(
        focus_seen.borrow().as_str(),
        "Two",
        "Down must move focus to the next radio even when selection is read-only"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        [] as [&str; 0],
        "moving focus in a read-only group must not report a selection"
    );
}

#[gpui::test]
fn radio_group_home_and_end_are_unconsumed_no_ops(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let bubbled = events();
    let outer_seen = bubbled.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let bubbled = bubbled.clone();
        gpui::div()
            .on_key_down(move |event, _, _| {
                if matches!(event.keystroke.key.as_str(), "home" | "end") {
                    bubbled.borrow_mut().push(event.keystroke.key.clone());
                }
            })
            .child(
                RadioGroup::new(
                    "radio-home-end",
                    vec!["One".into(), "Two".into(), "Three".into()],
                )
                .default_value("Two")
                .on_change(move |index, _, _| changes.borrow_mut().push(index.to_string())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    press(cx, "end");

    assert_eq!(
        outer_seen.borrow().as_slice(),
        ["home", "end"],
        "Home and End must bubble because the pinned radio hook does not consume them"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        [] as [&str; 0],
        "Home and End must not change a RadioGroup selection"
    );
}

#[gpui::test]
fn radio_group_read_only_pointer_moves_focus_without_selection(cx: &mut TestAppContext) {
    let focused = Rc::new(RefCell::new(String::new()));
    let focus_seen = focused.clone();
    let selected = Rc::new(RefCell::new(String::new()));
    let selected_seen = selected.clone();
    let cx = open_host(cx, move || {
        let focused = focused.clone();
        let selected = selected.clone();
        RadioGroup::new(
            "readonly-pointer-radios",
            vec!["One".into(), "Two".into(), "Three".into()],
        )
        .value("One")
        .is_read_only(true)
        .option_content(move |label, state| {
            if state.is_focused {
                *focused.borrow_mut() = label.to_string();
            }
            if state.is_selected {
                *selected.borrow_mut() = label.to_string();
            }
            gpui::div().child(label.to_string()).into_any_element()
        })
        .into_any_element()
    });

    click(cx, 48., 46.);
    flush_frame(cx);
    assert_eq!(
        focus_seen.borrow().as_str(),
        "Two",
        "a read-only controlled radio without a callback must focus the clicked option"
    );
    assert_eq!(
        selected_seen.borrow().as_str(),
        "One",
        "read-only pointer focus must not change the controlled selection"
    );
}

#[gpui::test]
fn radio_option_value_drives_callback_and_form_not_visible_label(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let group = RadioGroup::new(
            "radio-valued-form",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .name("plan")
        .default_value("pro");
        let form = Form::new().field(group.form_field().expect("named radio field"));
        assert_eq!(form.data(cx).text("plan"), Some(SharedString::from("pro")));

        for group in [
            RadioGroup::new(
                "radio-controlled-before-default",
                vec![
                    RadioOption::new("Free plan").value("free"),
                    RadioOption::new("Pro plan").value("pro"),
                ],
            )
            .name("plan")
            .value("free")
            .default_value("pro"),
            RadioGroup::new(
                "radio-default-before-controlled",
                vec![
                    RadioOption::new("Free plan").value("free"),
                    RadioOption::new("Pro plan").value("pro"),
                ],
            )
            .name("plan")
            .default_value("pro")
            .value("free"),
        ] {
            let form = Form::new().field(group.form_field().expect("named radio field"));
            assert_eq!(
                form.data(cx).text("plan"),
                Some(SharedString::from("free")),
                "controlled value must win over the default before first render"
            );
        }
    });

    let labels = events();
    let labels_seen = labels.clone();
    let changes = events();
    let changed = changes.clone();
    let cx = open_host(cx, move || {
        let labels = labels.clone();
        let changes = changes.clone();
        RadioGroup::new(
            "radio-valued-callback",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .option_content(move |label, _| {
            labels.borrow_mut().push(label.to_string());
            gpui::div().child(label.to_string()).into_any_element()
        })
        .on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()))
        .into_any_element()
    });

    assert!(labels_seen.borrow().iter().any(|label| label == "Pro plan"));
    click(cx, 48., 46.);
    assert_eq!(changed.borrow().as_slice(), ["pro"]);
}

#[gpui::test]
fn radio_option_disabled_is_skipped_by_arrows_and_pointer(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        RadioGroup::new(
            "radio-option-disabled",
            vec![
                RadioOption::new("One").value("one"),
                RadioOption::new("Two").value("two").is_disabled(true),
                RadioOption::new("Three").value("three"),
            ],
        )
        .default_value("one")
        .on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    click(cx, 8., 46.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["three"],
        "RadioOption::is_disabled must remove the option from arrows and pointer selection"
    );
}

#[gpui::test]
fn radio_group_empty_options_render_and_leave_tab_order(cx: &mut TestAppContext) {
    let probes = events();
    let recorded = probes.clone();
    let cx = open_host(cx, move || {
        let probes = probes.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(RadioGroup::new("empty-radio-group", vec![]))
            .child(Switch::new("empty-radio-probe").on_change(move |_, _, _| {
                probes.borrow_mut().push("probe".into());
            }))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["probe"]);
}

#[gpui::test]
fn radio_group_form_field_reads_changed_uncontrolled_value(cx: &mut TestAppContext) {
    let submissions = events();
    let recorded = submissions.clone();
    let cx = open_host(cx, move || {
        let submissions = submissions.clone();
        let group = RadioGroup::new(
            "live-radio-form",
            vec![
                RadioOption::new("Free plan").value("free"),
                RadioOption::new("Pro plan").value("pro"),
            ],
        )
        .name("plan")
        .default_value("free");
        let form = Form::new()
            .field(group.form_field().expect("named radio field"))
            .on_submit(move |data, _, _| {
                submissions.borrow_mut().push(
                    data.text("plan")
                        .expect("submitted radio value")
                        .to_string(),
                );
            });
        let submit = form.submit_handler();
        gpui::div()
            .flex()
            .flex_col()
            .child(group)
            .child(Button::new("live-radio-submit").label("Submit").on_press(
                move |_, window, cx| {
                    submit(window, cx);
                },
            ))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["pro"],
        "FormField must read the keyed uncontrolled selection after it changes"
    );
}

// ---------------------------------------------------------------------------
// ToggleButtonGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn toggle_button_group_defaults_to_single_selection(cx: &mut TestAppContext) {
    let state = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let state = state.clone();
        let changes = changes.clone();
        let current = state.borrow().clone();
        let [bold, italic, underline] = three_toggles("default-single");
        ToggleButtonGroup::new("default-single-group")
            .full_width(true)
            .selected_keys(current)
            .on_selection_change(move |next, _, _| {
                *state.borrow_mut() = next.to_vec();
                changes.borrow_mut().push(joined(next));
            })
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    // A full-width three-member group divides the 1920px test window into
    // 640px slots. Their centres are x=320 and x=960; md height centres y=18.
    click(cx, 320., 18.);
    click(cx, 960., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "italic"],
        "the default Single mode must replace Bold with Italic"
    );
}

#[gpui::test]
fn toggle_button_group_default_selected_keys_holds_uncontrolled_state(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("default-selected");
        ToggleButtonGroup::new("default-selected-group")
            .full_width(true)
            .default_selected_keys(["bold"])
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    click(cx, 320., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        [""],
        "pressing the initially selected Bold button must clear the uncontrolled seed"
    );
}

#[gpui::test]
fn toggle_button_group_controlled_empty_overrides_default_seed(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("controlled-empty");
        ToggleButtonGroup::new("controlled-empty-group")
            .full_width(true)
            .selected_keys(Vec::<SharedString>::new())
            .default_selected_keys(["bold"])
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    click(cx, 320., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "an explicitly controlled empty selection must override defaultSelectedKeys"
    );
}

#[gpui::test]
fn toggle_button_group_disabled_propagates_to_children(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let probes = events();
    let probed = probes.clone();
    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let probes = probes.clone();
        let [bold, italic, underline] = three_toggles("disabled-group");
        let bold = bold.is_disabled(false);
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                ToggleButtonGroup::new("disabled-group")
                    .is_disabled(true)
                    .on_selection_change(move |next, _, _| {
                        selections.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic)
                    .child_toggle(underline),
            )
            .child(
                Switch::new("disabled-group-probe").on_change(move |_, _, _| {
                    probes.borrow_mut().push("probe".into());
                }),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        selected.borrow().as_slice(),
        ["bold"],
        "an explicitly enabled child must override the disabled group context"
    );
    assert!(probed.borrow().is_empty());
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        probed.borrow().as_slice(),
        ["probe"],
        "the remaining disabled group children must leave the tab order"
    );
}

#[gpui::test]
fn toggle_button_group_horizontal_arrows_stop_at_edges(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("horizontal-edges");
        ToggleButtonGroup::new("horizontal-edges-group")
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "left");
    press(cx, "space");
    press(cx, "right");
    press(cx, "right");
    press(cx, "right");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "underline"],
        "Left at the first member and Right at the last must not wrap"
    );
}

#[gpui::test]
fn nested_toggle_button_group_releases_edge_arrow_to_toolbar(cx: &mut TestAppContext) {
    let presses = events();
    let recorded = presses.clone();
    let cx = open_host(cx, move || {
        let presses = presses.clone();
        let [bold, italic, _] = three_toggles("nested-toolbar");
        Toolbar::new()
            .child(
                ToggleButtonGroup::new("nested-toolbar-group")
                    .child_toggle(bold)
                    .child_toggle(italic),
            )
            .child(
                Button::new("nested-toolbar-next")
                    .label("Next")
                    .on_press(move |_, _, _| presses.borrow_mut().push("next".into())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["next"],
        "an unavailable group-edge arrow must reach the enclosing Toolbar"
    );
}

#[gpui::test]
fn toggle_button_group_skips_disabled_member_with_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("disabled-arrow");
        ToggleButtonGroup::new("disabled-arrow-group")
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic.is_disabled(true))
            .child_toggle(underline)
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "space");
    assert_eq!(recorded.borrow().as_slice(), ["underline"]);
}

#[gpui::test]
fn toggle_button_group_composes_child_press_after_selection(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let child_events = events.clone();
        let child_changes = events.clone();
        let selection_events = events.clone();
        ToggleButtonGroup::new("composed-child-press-group")
            .full_width(true)
            .on_selection_change(move |next, _, _| {
                selection_events
                    .borrow_mut()
                    .push(format!("selection:{}", joined(next)));
            })
            .child_toggle(
                ToggleButton::new("composed-child-press")
                    .key("bold")
                    .label("Bold")
                    .on_change(move |_, _, _| {
                        child_changes.borrow_mut().push("child-change".into());
                    })
                    .on_press(move |_, _, _| child_events.borrow_mut().push("child".into())),
            )
            .into_any_element()
    });

    click(cx, 960., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["selection:bold", "child"],
        "group selection must run once before child onPress and replace child onChange"
    );
}

#[gpui::test]
fn toggle_button_group_controlled_without_callback_silences_child_change(cx: &mut TestAppContext) {
    let events = events();
    let recorded = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        ToggleButtonGroup::new("controlled-child-change-group")
            .full_width(true)
            .selected_keys(["bold"])
            .child_toggle(
                ToggleButton::new("controlled-child-change")
                    .key("bold")
                    .label("Bold")
                    .on_change(move |_, _, _| events.borrow_mut().push("child-change".into())),
            )
            .into_any_element()
    });

    click(cx, 320., 18.);
    assert!(
        recorded.borrow().is_empty(),
        "a controlled group must own selection even without a group callback"
    );
}

#[gpui::test]
fn toggle_button_group_horizontal_uses_horizontal_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("horizontal-axis");
        ToggleButtonGroup::new("horizontal-axis-group")
            .selection_mode(SelectionMode::Single)
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "space");
    press(cx, "left");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["italic", "bold"],
        "Right must move focus to Italic and Left back to Bold before Space activates each"
    );
}

#[gpui::test]
fn toggle_button_group_horizontal_ignores_vertical_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let bubbled = events();
    let outer_seen = bubbled.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let bubbled = bubbled.clone();
        let [bold, italic, underline] = three_toggles("horizontal-cross-axis");
        gpui::div()
            .on_key_down(move |event, _, _| {
                if matches!(event.keystroke.key.as_str(), "up" | "down") {
                    bubbled.borrow_mut().push(event.keystroke.key.clone());
                }
            })
            .child(
                ToggleButtonGroup::new("horizontal-cross-axis-group")
                    .selection_mode(SelectionMode::Single)
                    .on_selection_change(move |next, _, _| {
                        changes.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic)
                    .child_toggle(underline),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "up");
    press(cx, "space");
    assert_eq!(
        outer_seen.borrow().as_slice(),
        ["down", "up"],
        "cross-axis arrows must remain available to an enclosing surface"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "Down and Up are cross-axis keys and must leave focus on Bold"
    );
}

#[gpui::test]
fn toggle_button_group_vertical_uses_vertical_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("vertical-axis");
        ToggleButtonGroup::new("vertical-axis-group")
            .orientation(Orientation::Vertical)
            .selection_mode(SelectionMode::Single)
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "space");
    press(cx, "up");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["italic", "bold"],
        "Down must move focus to Italic and Up back to Bold before Space activates each"
    );
}

#[gpui::test]
fn toggle_button_group_vertical_arrows_stop_at_edges(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("vertical-edges");
        ToggleButtonGroup::new("vertical-edges-group")
            .orientation(Orientation::Vertical)
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "up");
    press(cx, "space");
    press(cx, "down");
    press(cx, "down");
    press(cx, "down");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold", "underline"],
        "Up at the first member and Down at the last must not wrap"
    );
}

#[gpui::test]
fn toggle_button_group_vertical_ignores_horizontal_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let bubbled = events();
    let outer_seen = bubbled.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let bubbled = bubbled.clone();
        let [bold, italic, underline] = three_toggles("vertical-cross-axis");
        gpui::div()
            .on_key_down(move |event, _, _| {
                if matches!(event.keystroke.key.as_str(), "left" | "right") {
                    bubbled.borrow_mut().push(event.keystroke.key.clone());
                }
            })
            .child(
                ToggleButtonGroup::new("vertical-cross-axis-group")
                    .orientation(Orientation::Vertical)
                    .selection_mode(SelectionMode::Single)
                    .on_selection_change(move |next, _, _| {
                        changes.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic)
                    .child_toggle(underline),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "left");
    press(cx, "space");
    assert_eq!(
        outer_seen.borrow().as_slice(),
        ["right", "left"],
        "cross-axis arrows must remain available to an enclosing surface"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "Right and Left are cross-axis keys and must leave focus on Bold"
    );
}

#[gpui::test]
fn toggle_button_group_tab_leaves_for_the_next_control(cx: &mut TestAppContext) {
    let selections = events();
    let selected = selections.clone();
    let probes = events();
    let probed = probes.clone();
    let cx = open_host(cx, move || {
        let selections = selections.clone();
        let probes = probes.clone();
        let [bold, italic, underline] = three_toggles("tab-exit");
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                ToggleButtonGroup::new("tab-exit-group")
                    .selection_mode(SelectionMode::Single)
                    .on_selection_change(move |next, _, _| {
                        selections.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic)
                    .child_toggle(underline),
            )
            .child(Switch::new("tab-exit-probe").on_change(move |_, _, _| {
                probes.borrow_mut().push("probe".into());
            }))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        selected.borrow().as_slice(),
        [] as [&str; 0],
        "Tab must not land on another member of the same toggle group"
    );
    assert_eq!(
        probed.borrow().as_slice(),
        ["probe"],
        "one Tab from inside the group must reach the following control"
    );
}

#[gpui::test]
fn toggle_button_group_reverse_tab_restores_last_focused_member(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("restore-last");
        gpui::div()
            .on_key_down(|event, window, cx| {
                if event.keystroke.key == "tab" {
                    cx.stop_propagation();
                    if event.keystroke.modifiers.shift {
                        window.focus_prev();
                    } else {
                        window.focus_next();
                    }
                }
            })
            .flex()
            .flex_col()
            .child(
                ToggleButtonGroup::new("restore-last-group")
                    .on_selection_change(move |next, _, _| {
                        changes.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic)
                    .child_toggle(underline),
            )
            .child(Switch::new("restore-last-after").on_change(|_, _, _| {}))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "tab");
    press(cx, "shift-tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["italic"],
        "reverse Tab must restore the group's last-focused enabled member"
    );
}

#[gpui::test]
fn toggle_button_group_reentry_skips_a_now_disabled_last_member(cx: &mut TestAppContext) {
    let disable_italic = Rc::new(RefCell::new(false));
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let disabled = *disable_italic.borrow();
        let disable_on_probe = disable_italic.clone();
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("restore-disabled");
        gpui::div()
            .on_key_down(|event, window, cx| {
                if event.keystroke.key == "tab" {
                    cx.stop_propagation();
                    if event.keystroke.modifiers.shift {
                        window.focus_prev();
                    } else {
                        window.focus_next();
                    }
                }
            })
            .flex()
            .flex_col()
            .child(
                ToggleButtonGroup::new("restore-disabled-group")
                    .on_selection_change(move |next, _, _| {
                        changes.borrow_mut().push(joined(next));
                    })
                    .child_toggle(bold)
                    .child_toggle(italic.is_disabled(disabled))
                    .child_toggle(underline),
            )
            .child(
                Switch::new("restore-disabled-after").on_change(move |_, _, _| {
                    *disable_on_probe.borrow_mut() = true;
                }),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "tab");
    press(cx, "space");
    flush_frame(cx);
    press(cx, "shift-tab");
    flush_frame(cx);
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["underline"],
        "focus re-entry must stay on the enabled edge when the remembered member became disabled"
    );
}

#[gpui::test]
fn toggle_button_group_disallow_empty_keeps_the_only_selection(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("disallow-empty");
        ToggleButtonGroup::new("disallow-empty-group")
            .full_width(true)
            .selection_mode(SelectionMode::Single)
            .selected_keys(["bold"])
            .disallow_empty_selection(true)
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic)
            .child_toggle(underline)
            .into_any_element()
    });

    click(cx, 320., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "re-pressing the only selected item must keep it selected"
    );
}

#[gpui::test]
fn toggle_button_group_ids_isolate_identical_child_ids(cx: &mut TestAppContext) {
    let first_bold = Rc::new(RefCell::new(false));
    let first_seen = first_bold.clone();
    let second_italic = Rc::new(RefCell::new(false));
    let second_seen = second_italic.clone();
    let _cx = open_host(cx, move || {
        let first_bold = first_bold.clone();
        let second_italic = second_italic.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(
                ToggleButtonGroup::new("isolated-first-group")
                    .default_selected_keys(["bold"])
                    .child_toggle(ToggleButton::new("shared-bold").key("bold").content(
                        move |state| {
                            *first_bold.borrow_mut() = state.is_selected;
                            gpui::div().child("Bold").into_any_element()
                        },
                    ))
                    .child_toggle(
                        ToggleButton::new("shared-italic")
                            .key("italic")
                            .label("Italic"),
                    ),
            )
            .child(
                ToggleButtonGroup::new("isolated-second-group")
                    .default_selected_keys(["italic"])
                    .child_toggle(ToggleButton::new("shared-bold").key("bold").label("Bold"))
                    .child_toggle(ToggleButton::new("shared-italic").key("italic").content(
                        move |state| {
                            *second_italic.borrow_mut() = state.is_selected;
                            gpui::div().child("Italic").into_any_element()
                        },
                    )),
            )
            .into_any_element()
    });

    assert!(
        *first_seen.borrow(),
        "the first group's Bold seed must remain selected"
    );
    assert!(
        *second_seen.borrow(),
        "the second group must own a separate Italic seed despite identical child ids"
    );
}

#[gpui::test]
fn toggle_button_group_reorder_preserves_uncontrolled_selection(cx: &mut TestAppContext) {
    let reordered = Rc::new(RefCell::new(false));
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let reordered_for_change = reordered.clone();
        let changes = changes.clone();
        let order = if *reordered.borrow() {
            ["italic", "bold", "underline"]
        } else {
            ["bold", "italic", "underline"]
        };
        let mut group = ToggleButtonGroup::new("reorder-stable-group")
            .full_width(true)
            .default_selected_keys(["bold"])
            .on_selection_change(move |next, _, _| {
                *reordered_for_change.borrow_mut() = true;
                changes.borrow_mut().push(joined(next));
            });
        for key in order {
            group = group.child_toggle(
                ToggleButton::new(gpui::ElementId::Name(format!("reorder-{key}").into()))
                    .key(key)
                    .label(key),
            );
        }
        group.into_any_element()
    });

    click(cx, 960., 18.);
    flush_frame(cx);
    click(cx, 320., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["italic", ""],
        "reordering children must not reseed the group's selected keys"
    );
}

#[gpui::test]
fn toggle_button_group_size_propagates_unless_child_overrides(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let [bold, italic, underline] = three_toggles("group-size");
        ToggleButtonGroup::new("group-size-group")
            .full_width(true)
            .size(Size::Lg)
            .on_selection_change(move |next, _, _| changes.borrow_mut().push(joined(next)))
            .child_toggle(bold)
            .child_toggle(italic.size(Size::Sm))
            .child_toggle(underline)
            .into_any_element()
    });

    click(cx, 320., 37.);
    click(cx, 960., 37.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bold"],
        "the group must make an unspecified child large while an explicit small child stays small"
    );
}
