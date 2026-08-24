//! Behaviour tests for the FIELD and FORM components: the three group
//! controls (CheckboxGroup, RadioGroup, SwitchGroup), the bare Input, the
//! NumberField steppers, the TimeField segments and the Form's collect-and-
//! validate path.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests drive the controls with simulated clicks and keystrokes and assert on
//! recorded callbacks, state entities the test owns, and behavioural probes
//! only — never on appearance.
//!
//! Geometry is derived from the components' own constants and from gpui's own
//! defaults, not guessed:
//!
//! - Every field row is `util::FIELD_HEIGHT` = 36px tall at the window origin
//!   unless the component adds a label/description on top.
//! - A bare `Input` is wrapped in `max_w(320)` (`Input::render`), and the
//!   field's own `px(12)` padding leaves a 20px `size-5` clear button in
//!   x 288..308, centre (298, 18) — the same maths as ComboBox's chevron in
//!   pickers.rs.
//! - The checkbox control is `size-4` = 16px (`checkbox.rs` `box_px`); the
//!   group rows are `gap(16)` apart and text-sized. A row's height is
//!   `max(16px, the label line)`, and gpui's *default* line height is `phi()`
//!   = 1.618034 × font size (gpui `style.rs`, `Default for TextStyle`; the
//!   port never overrides `line_height` on these rows), so a 14px label line
//!   is 22.65px. Row 1's 16px box therefore centres at
//!   22.65 + 16 + 22.65/2 ≈ 50. The clicks below land at (8, 11) and (8, 46);
//!   row *i*'s box is `[i·(H+16) + (H-16)/2, +16]` tall, so (8, 46) stays
//!   inside row 1's box for any line height H in ~15..25px — the assertion
//!   does not trust the exact default.
//! - A switch track is 40x20 (`switch.rs` `Size::Md`); a `SwitchGroup`'s rows
//!   are `gap(16)` apart with a 16px label line (25.89px at phi), so the two
//!   tracks centre at y ≈ 13 and y ≈ 54.8. Clicks at (20, 13) and (20, 52)
//!   land inside the respective track for any line height in ~17..30px.
//! - `NumberField` is a 220px group (`w(px(220.))` in number_field.rs) with a
//!   40px (`w-10`) stepper cell at each end: the increment button centres at
//!   (200, 18).
//! - A `Form` stacks its children `gap(16)` (form.rs, `gap(px(16.))`). A bare
//!   Input is 36px, so field 1 spans y 0..36, field 2 y 52..88, and the md
//!   submit `Button` (36px — `Size::Md::control_height`, herogpui-core
//!   enums.rs) spans y 104..140; the clicks are (60, 18), (60, 70), (60, 122).
//!
//! Keystrokes are delivered exactly as the shared harness does (`press`), and
//! gpui redraws a dirty window before dispatching the next key event
//! (`Window::dispatch_key_event` draws first), so a roving tab stop — the
//! radios — moves its handle between presses, which is what the arrows
//! depend on.

mod harness;

use std::collections::HashSet;

use gpui::{prelude::*, px, SharedString, TestAppContext};
use herogpui_components::{
    Button, Checkbox, CheckboxGroup, CheckboxOption, Form, FormData, FormField, Input, InputState,
    NumberField, NumberState, RadioGroup, RadioOption, Switch, SwitchGroup, Time, TimeField,
    TimeState,
};

use harness::{click, events, open_host, press};

/// The keys of a selection joined in a stable order.
///
/// The groups report a `HashSet`, which iterates in no particular order;
/// sorting makes the recorded report deterministic.
fn sorted_join(keys: &HashSet<SharedString>) -> String {
    let mut keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
    keys.sort();
    keys.join(",")
}

/// A submission rendered the way the gallery's own Form demo renders it:
/// `name=value` pairs in registration order, values through `as_text`.
fn record_data(data: &FormData) -> String {
    data.iter()
        .map(|(name, value)| format!("{name}={}", value.as_text()))
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
// CheckboxGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn checkbox_group_toggles_two_keys(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        CheckboxGroup::new(
            "cg-two",
            vec![
                CheckboxOption::new("alpha", "Alpha"),
                CheckboxOption::new("beta", "Beta"),
                CheckboxOption::new("gamma", "Gamma"),
            ],
        )
        .default_value(Vec::<SharedString>::new())
        .on_change(move |set, _, _| changes.borrow_mut().push(sorted_join(set)))
        .into_any_element()
    });

    // Row 0's 16px box centre is (8, 11) and row 1's (8, 46) — see the file
    // doc comment; the click lands inside row 1's box for any line height
    // in ~15..25px.
    click(cx, 8., 11.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha"],
        "clicking the first box must select its key"
    );
    click(cx, 8., 46.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta"],
        "clicking the second box must keep the first pick"
    );
    click(cx, 8., 11.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta", "beta"],
        "re-clicking a picked box must remove it, which requires the group \
         to remember the earlier picks"
    );
}

#[gpui::test]
fn checkbox_indeterminate_does_not_block_toggle(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        // `is_indeterminate` only swaps the dash for the check; the box must
        // still respond to a press.
        Checkbox::new("cb-indeterminate")
            .label("Indeterminate")
            .is_indeterminate(true)
            .on_change(move |checked, _, _| {
                changes.borrow_mut().push(format!("change:{checked}"));
            })
            .into_any_element()
    });

    // The 16px control box is centred in a 22.65px row (see the file doc
    // comment), so its band is roughly y 3..19; (8, 11) is its centre.
    click(cx, 8., 11.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:true"],
        "an indeterminate box must still report a change when clicked"
    );
}

// ---------------------------------------------------------------------------
// RadioGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn radio_group_arrows_move_and_select(cx: &mut TestAppContext) {
    let picks = events();
    let picked = picks.clone();
    let switches = events();
    let switched = switches.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        let switches = switches.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                RadioGroup::new(
                    "rg-roving",
                    vec!["One".into(), "Two".into(), "Three".into()],
                )
                .default_value("One")
                .on_change(move |value, _, _| picks.borrow_mut().push(value.to_string())),
            )
            // A second tab stop after the group, so "Tab leaves the group"
            // is observable: Tab must move past the radios to this switch,
            // and the radios must stop answering the arrows.
            .child(
                Switch::new("rg-exit-probe").on_change(move |checked, _, _| {
                    switches.borrow_mut().push(format!("change:{checked}"));
                }),
            )
            .into_any_element()
    });

    // A radio group is ONE tab stop (radio_group.rs: "the stop is the
    // selected option, or the first when nothing is selected yet"): Tab
    // enters on row 0, and each arrow both moves the selection and pulls
    // the focus along (the selected row claims the group's handle on the
    // next render — AGENTS.md's roving tab stop).
    press(cx, "tab");
    press(cx, "down");
    press(cx, "down");
    press(cx, "up");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Two", "Three", "Two"],
        "Down Down Up must move AND select within the one tab stop"
    );

    // Tab must leave the group, not walk its rows: the next stop is the
    // switch, and a Down at the switch would have selected row 2 had the
    // radios kept the keys.
    press(cx, "tab");
    press(cx, "down");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Two", "Three", "Two"],
        "the group must not answer the arrows after Tab leaves it"
    );

    // And the focus is not lost: Space activates the switch that Tab landed
    // on, which proves the group is out of the tab order rather than merely
    // out of focus detail.
    press(cx, "space");
    assert_eq!(
        switched.borrow().as_slice(),
        ["change:true"],
        "Tab must land on the next tab stop — the switch"
    );
}

#[gpui::test]
fn radio_group_disabled_option_is_skipped(cx: &mut TestAppContext) {
    // `Radio.isDisabled` — a disabled option draws dimmed and answers no
    // clicks ("Whether the radio button is disabled" — heroui.com/react,
    // `### Radio`; Interactive States: disabled is "reduced opacity, no
    // pointer events"), and the group's arrows and roving tab stop pass it
    // by. Index 1 is disabled, so Down from row 0 lands on row 2, Up from
    // row 2 wraps past it back to row 0, and a pointer press on the disabled
    // row changes nothing.
    let picks = events();
    let picked = picks.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        RadioGroup::new(
            "rg-disabled-opt",
            vec![
                "One".into(),
                RadioOption::new("Two").is_disabled(true),
                "Three".into(),
            ],
        )
        .default_value("One")
        .on_change(move |value, _, _| picks.borrow_mut().push(value.to_string()))
        .into_any_element()
    });

    // Tab enters the group on the selected option (row 0), and the arrows
    // must skip the disabled row: Down lands on 2 and Up from there wraps
    // back to 0 — never 1.
    press(cx, "tab");
    press(cx, "down");
    press(cx, "up");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Three", "One"],
        "Down and Up must skip the disabled option and land on the enabled \
         ones around it"
    );

    // A pointer press on the disabled option must select nothing: v3's
    // `status-disabled` is `pointer-events: none`, and this port leaves the
    // row without a click handler. Row 1's 16px box centre is (8, 46) — see
    // the file doc comment.
    click(cx, 8., 46.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Three", "One"],
        "the disabled option must not answer a click"
    );

    // The enabled row beside it still does: row 0's box centre is (8, 11).
    click(cx, 8., 11.);
    assert_eq!(
        picked.borrow().as_slice(),
        ["Three", "One", "One"],
        "an enabled option beside a disabled one must still answer a click"
    );
}

#[gpui::test]
fn radio_group_first_option_disabled_stays_reachable(cx: &mut TestAppContext) {
    // AGENTS.md's roving tab stop: a stop that rests on a disabled option
    // takes the group out of the tab order. With row 0 disabled and nothing
    // selected, the group's one stop must fall on the first *enabled* option,
    // so Tab still reaches the group. React Aria then walks from that focused
    // option, so Down advances to the following enabled option.
    let picks = events();
    let picked = picks.clone();
    let cx = open_host(cx, move || {
        let picks = picks.clone();
        RadioGroup::new(
            "rg-first-disabled",
            vec![
                RadioOption::new("One").is_disabled(true),
                "Two".into(),
                "Three".into(),
            ],
        )
        .default_value("")
        .on_change(move |value, _, _| picks.borrow_mut().push(value.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    assert_eq!(
        picked.borrow().as_slice(),
        ["Three"],
        "Tab must reach the group with the first option disabled, and the \
         first Down must advance from the focused option"
    );
}

// ---------------------------------------------------------------------------
// SwitchGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn switch_group_reports_each_switch(cx: &mut TestAppContext) {
    let first = events();
    let first_rec = first.clone();
    let second = events();
    let second_rec = second.clone();
    let cx = open_host(cx, move || {
        let first = first.clone();
        let second = second.clone();
        SwitchGroup::new()
            .child(
                Switch::new("sg-1")
                    .label("One")
                    .on_change(move |checked, _, _| {
                        first.borrow_mut().push(format!("change:{checked}"));
                    }),
            )
            .child(
                Switch::new("sg-2")
                    .label("Two")
                    .on_change(move |checked, _, _| {
                        second.borrow_mut().push(format!("change:{checked}"));
                    }),
            )
            .into_any_element()
    });

    // Each switch's own click handler is what must fire: the md track is the
    // whole hit target (the label has none), and the two tracks centre at
    // (20, 13) and (20, 52) — see the file doc comment; each click lands
    // inside the right track for any line height in ~17..30px.
    click(cx, 20., 13.);
    click(cx, 20., 52.);
    assert_eq!(
        first_rec.borrow().as_slice(),
        ["change:true"],
        "clicking the first track must report that switch's change"
    );
    assert_eq!(
        second_rec.borrow().as_slice(),
        ["change:true"],
        "clicking the second track must report its own change, not the first's"
    );
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

#[gpui::test]
fn input_typing_and_clear(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        Input::new(state)
            .is_clearable(true)
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    // Click into the field (36px tall at the origin, capped at 320px wide),
    // which focuses it; each keystroke must both land in the state the test
    // owns and be reported.
    click(cx, 60., 18.);
    cx.simulate_input("hello");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello"],
        "typing must report the growing value on every keystroke"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "hello", "the InputState must hold what was typed");

    // The clear button is a 20px box at the field's inline end: the 320px
    // wrapper minus the field's px(12) padding minus half the box = 298,
    // vertically centred at 18 (`Input::render`, same geometry as
    // ComboBox's chevron in pickers.rs).
    click(cx, 298., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello", ""],
        "the clear button must report an empty value"
    );
    let cleared = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(cleared, "", "clearing must empty the InputState");
}

// ---------------------------------------------------------------------------
// NumberField
// ---------------------------------------------------------------------------

#[gpui::test]
fn number_field_steppers_and_bounds(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| NumberState::new(cx, 90.0));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let state = state_for_view.clone();
        // Component-level bounds overrule the state's; the steppers bump by
        // `step` and clamp to `[min, max]`.
        NumberField::new(state)
            .min_value(0.0)
            .max_value(100.0)
            .step(10.0)
            .on_change(move |value, _, _| changes.borrow_mut().push(format!("{value}")))
            .into_any_element()
    });

    // The group is 220px wide and 36px tall with a 40px (`w-10`) increment
    // cell at the end: centre (200, 18) (number_field.rs). From 90, one step
    // hits the 100 maximum and a second step must stay there without reporting
    // a duplicate change.
    click(cx, 200., 18.);
    click(cx, 200., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["100"],
        "a bound no-op must not synthesize a duplicate on_change callback"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_string());
    assert_eq!(value, "100", "the NumberState must hold the clamped value");
}

// ---------------------------------------------------------------------------
// TimeField
// ---------------------------------------------------------------------------

#[gpui::test]
fn time_field_segments_answer_arrows(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TimeField::new(state_for_view.clone())
            .on_change(move |time, _, _| {
                let text = time
                    .map(|t| format!("{:02}:{:02}", t.hour, t.minute))
                    .unwrap_or_default();
                changes.borrow_mut().push(text);
            })
            .into_any_element()
    });

    // The field is the page's only tab stop (`TimeField` takes a
    // `tab_stop(true)` handle) and the focused segment starts at Hour
    // (time_field.rs `TimeState`). No coordinates are involved: the arrows
    // are read by the group's `on_key_down`.
    press(cx, "tab");
    press(cx, "up");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["10:30"],
        "Up must step the focused hour segment by one hour"
    );

    // Right walks the segment list to Minute; Up steps it.
    press(cx, "right");
    press(cx, "up");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["10:30", "10:31"],
        "Right must move to the minute segment and Up must step it"
    );
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

#[gpui::test]
fn form_submit_reports_named_fields(cx: &mut TestAppContext) {
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let invalid = invalids.clone();
    let name_state = cx.new(|cx| InputState::new(cx));
    let email_state = cx.new(|cx| InputState::new(cx));
    let name_for_view = name_state;
    let email_for_view = email_state;
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let name_state = name_for_view.clone();
        let email_state = email_for_view.clone();
        // The gallery wires it exactly like this: `Form::field(..)` tells the
        // form which inputs it owns, the inputs' `name` props ride on their
        // states, and `submit_handler()` is what a submit button calls
        // (gpui gives a child no way to reach its ancestor form).
        let form = Form::new()
            .field(FormField::text(name_state.clone()).is_required(true))
            .field(FormField::text(email_state.clone()))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                invalids.borrow_mut().push(record_data(data));
            });
        let submit = form.submit_handler();
        form.child(Input::new(name_state).name("name"))
            .child(Input::new(email_state).name("email"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("form-submit")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    // Layout: two bare Inputs stacked `gap(16)` (field 1 y 0..36, field 2
    // y 52..88) and the md submit button below (y 104..140) — see the file
    // doc comment.
    click(cx, 60., 18.);
    cx.simulate_input("ada");
    click(cx, 60., 70.);
    cx.simulate_input("ada@example.com");
    click(cx, 60., 122.);

    assert_eq!(
        submitted.borrow().as_slice(),
        ["name=ada,email=ada@example.com"],
        "submitting must report every named field, in registration order"
    );
    assert!(
        invalid.borrow().is_empty(),
        "a filled required field must not block the submission"
    );
}

#[gpui::test]
fn form_validation_blocks_submit(cx: &mut TestAppContext) {
    let submits = events();
    let submitted = submits.clone();
    let invalids = events();
    let missing = invalids.clone();
    let name_state = cx.new(|cx| InputState::new(cx));
    let email_state = cx.new(|cx| InputState::new(cx));
    let name_for_view = name_state;
    let email_for_view = email_state;
    let cx = open_host(cx, move || {
        let submits = submits.clone();
        let invalids = invalids.clone();
        let name_state = name_for_view.clone();
        let email_state = email_for_view.clone();
        let form = Form::new()
            .field(FormField::text(name_state.clone()).is_required(true))
            .field(FormField::text(email_state.clone()))
            .on_submit(move |data: &FormData, _, _| {
                submits.borrow_mut().push(record_data(data));
            })
            .on_invalid(move |data: &FormData, _, _| {
                // The submit handler's block check is exactly
                // `data.missing_required(&required_names)`, so re-deriving
                // the required list here records which names it reported.
                let names = data
                    .missing_required(&[SharedString::from("name")])
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                invalids.borrow_mut().push(format!("missing:{names}"));
            });
        let submit = form.submit_handler();
        form.child(Input::new(name_state).name("name"))
            .child(Input::new(email_state).name("email"))
            .child(
                gpui::div().flex().gap(px(8.)).child(
                    Button::new("form-submit")
                        .label("Submit")
                        .on_press(move |_, window, cx| submit(window, cx)),
                ),
            )
            .into_any_element()
    });

    // Fill only the non-required field: `name` stays empty. Same spots as
    // the submit test — field 2 at (60, 70), submit at (60, 122).
    click(cx, 60., 70.);
    cx.simulate_input("ada@example.com");
    click(cx, 60., 122.);

    assert_eq!(
        missing.borrow().as_slice(),
        ["missing:name"],
        "an empty required field must route the submission to on_invalid \
         with that name reported"
    );
    assert!(
        submitted.borrow().is_empty(),
        "onSubmit must not fire while a required field is empty"
    );
}
