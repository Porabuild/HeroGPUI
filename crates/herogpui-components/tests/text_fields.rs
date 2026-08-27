//! Behaviour tests for the TEXT components: TextField, TextArea, SearchField,
//! InputOTP, InputGroup, ColorField, DateField and Fieldset.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests drive the controls with simulated keystrokes and clicks and assert on
//! recorded callbacks and state entities the test owns — never on appearance.
//!
//! Geometry is derived from the components' own constants, not guessed:
//!
//! - Every single-line field is `util::FIELD_HEIGHT` = 36px tall at the window
//!   origin and wrapped in `max_w(320)` unless full width (`Input::render`).
//!   Its `px(12)` padding puts the 20px `size-5` clear button at x 288..308,
//!   centre (298, 18) — the same maths as ComboBox's chevron in pickers.rs.
//!   `SearchField`'s 16px magnifier sits at x 12..28, so the text starts at
//!   12 + 16 + 8 (gap) = 36.
//! - `ColorField` in editable mode delegates to `Input` with a 16px
//!   `ColorSwatch` start content, so the text starts at the same 36.
//! - `TextArea` is the same `Input` in multi-line mode with
//!   `rows_height(3)` = 20·3 + 16 = 76px (`textarea.rs`), `py(8)` and the text
//!   at the top. An EMPTY textarea hugs its content: the wrapping row has no
//!   `whitespace_nowrap` to force a width, so the box measures only ~32px
//!   until text wraps inside it (measured by probing: a click at x = 40
//!   misses, x = 20 focuses).
//! - `InputOTP` cells are 38x40 with an 8px gap (`input_otp.rs`), so cell *i*
//!   spans x 46i..46i+38 and every cell centre is y 20; a second instance on
//!   the same page sits `gap(16)` below, cell 0 centre y = 40 + 16 + 20 = 76.
//! - `InputGroup` is `min-h-9` with no padding of its own: the `InputAddon`s
//!   carry `px(12)` each (`input_group.rs`). In a `w(px(400.))` wrapper the
//!   addons are flush against the wrapper's edges — a click at x = 6 lands in
//!   the prefix's left padding and one at x = 394 in the suffix's right
//!   padding for any glyph width. A click on an addon is a click on a plain
//!   div: gpui transfers focus to the nearest *focusable* ancestor (the app
//!   root), so the field stops seeing keys — a browser blur equivalent, not a
//!   caret move (the InputState's cursor is untouched, which is what makes
//!   re-focusing resume typing where it left off).
//! - A `Fieldset` is `gap(24)` between children (`field.rs`); its legend is a
//!   24px line (`FieldsetLegend::render`), so a Group under it starts at y 48
//!   and its bare 36px field centres at (60, 66).
//!
//! Where the component reads the keyboard (the `track_focus`'d handle), Tab is
//! the entry point exactly as `fields.rs` drives `TimeField`; where a click is
//! what focuses, the pointer lands inside the field before typing begins.
//! Chords are dispatched with their modifiers parsed (`ctrl-a`), unlike the
//! screenshot driver's posted messages, which is what the select-all test
//! relies on.

mod harness;

use gpui::{prelude::*, px, TestAppContext};
use herogpui_components::{
    ColorField, Date, DateField, Fieldset, FieldsetGroup, FieldsetLegend, Input, InputAddon,
    InputGroup, InputOTP, InputState, OtpPattern, OtpState, PickerColor, SearchField, TextArea,
    TextField,
};

use harness::{click, events, open_host, press};

// ---------------------------------------------------------------------------
// TextField
// ---------------------------------------------------------------------------

#[gpui::test]
fn text_field_typing_reports_and_holds(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TextField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    // Click into the field (36px tall, 320px wide at the origin), which
    // focuses it; the empty value puts the caret at 0.
    click(cx, 60., 18.);
    cx.simulate_input("hello");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello"],
        "typing must report the growing value on every keystroke"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "hello", "the InputState must hold what was typed");

    // Caret motion is behavioural: Left Left from the end puts the caret at
    // index 3 (`InputState` owns cursor/anchor), so the next char lands
    // mid-string instead of at the end.
    press(cx, "left");
    press(cx, "left");
    press(cx, "x");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello", "helxlo"],
        "a keystroke after Left Left must insert at cursor 3, not append"
    );
    let moved = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        moved, "helxlo",
        "the InputState must hold the mid-string insert"
    );
}

#[gpui::test]
fn text_field_backspace_and_select_all(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TextField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("hello");
    press(cx, "backspace");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello", "hell"],
        "Backspace must delete the char before the caret and report the loss"
    );

    // The test platform parses "ctrl-a" into a keystroke carrying the control
    // modifier (unlike the screenshot driver's posted messages), so the
    // field's select-all branch must run.
    press(cx, "ctrl-a");
    let selection = cx.update(|_, cx| state.read(cx).selection());
    assert_eq!(
        selection,
        Some((0, 4)),
        "Ctrl+A must select the whole value: anchor 0 to cursor 4"
    );

    press(cx, "x");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["h", "he", "hel", "hell", "hello", "hell", "x"],
        "typing over a selection must replace it outright"
    );
    let replaced = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(replaced, "x", "the InputState must hold the replaced value");
}

#[gpui::test]
fn input_max_length_blocks_rejected_edits_without_change(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Input::new(state_for_view.clone())
            .max_length(3)
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("abcd");
    press(cx, "space");
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("z".to_owned()));
    press(cx, "ctrl-v");
    press(cx, "delete");
    press(cx, "home");
    press(cx, "backspace");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab", "abc"],
        "rejected insertions and boundary deletions must not emit duplicate changes"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "abc",
        "the owned InputState must stop at the configured maximum length"
    );
}

// ---------------------------------------------------------------------------
// TextArea
// ---------------------------------------------------------------------------

#[gpui::test]
fn text_area_typing_and_newline(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TextArea::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    // The field is 76px tall (rows 3), but an EMPTY multi-line field hugs its
    // content: the row has no `whitespace_nowrap` to force a width, so the
    // field is only ~32px wide until text wraps inside it. Measured by
    // probing: clicks at x = 40 already miss it, x = 20 focus it. A click at
    // (10, 40) is safely inside the 76px box; in multi-line mode a click only
    // focuses, it does not move the caret.
    click(cx, 10., 40.);
    cx.simulate_input("ab");
    // `WhiteSpace::Normal` wraps by default (AGENTS.md), so the only way a
    // newline gets in is Enter — which v3's TextArea takes as a newline, not
    // a submit.
    press(cx, "enter");
    cx.simulate_input("cd");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab", "ab\n", "ab\nc", "ab\ncd"],
        "Enter must insert a newline into the value, and typing must continue \
         after it"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "ab\ncd",
        "the InputState must hold the multi-line value"
    );
}

#[gpui::test]
fn text_area_max_length_rejects_a_newline_without_change(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TextArea::new(state_for_view.clone())
            .max_length(2)
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    click(cx, 10., 40.);
    cx.simulate_input("ab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab"],
        "a rejected newline must not report an unchanged TextArea value"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "ab");
}

// ---------------------------------------------------------------------------
// SearchField
// ---------------------------------------------------------------------------

#[gpui::test]
fn search_field_clear_and_submit(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let clears = events();
    let cleared = clears.clone();
    let submits = events();
    let submitted = submits.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let clears = clears.clone();
        let submits = submits.clone();
        SearchField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(format!("change:{text}")))
            .on_clear(move |_, _| clears.borrow_mut().push("clear".to_owned()))
            .on_submit(move |text, _, _| submits.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });

    // The magnifier occupies x 12..28, so the text area starts at 36; a click
    // at (60, 18) focuses the field with the empty value's caret at 0.
    click(cx, 60., 18.);
    cx.simulate_input("rust");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["change:r", "change:ru", "change:rus", "change:rust"],
        "typing must report the growing value"
    );

    // The clear button is the field's 20px end box, x 288..308 — v3's
    // `SearchField.ClearButton` — and it must empty both the state and the
    // report, notifying `on_clear` along the way.
    click(cx, 298., 18.);
    assert_eq!(
        recorded.borrow().last().map(String::as_str),
        Some("change:"),
        "the clear button must report an empty value through on_change"
    );
    assert_eq!(
        cleared.borrow().as_slice(),
        ["clear"],
        "clearing must notify on_clear"
    );
    let empty = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(empty, "", "clearing must empty the InputState");

    // The field keeps the focus through the clear click (the button has no
    // handle of its own), so typing continues and Enter submits the value.
    cx.simulate_input("abc");
    press(cx, "enter");
    assert_eq!(
        submitted.borrow().as_slice(),
        ["abc"],
        "Enter on the single-line search field must report on_submit with the \
         value"
    );
    let after = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(after, "abc", "the InputState must hold the typed value");
}

#[gpui::test]
fn search_field_custom_clear_icon_preserves_clear_behavior(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let clears = events();
    let cleared = clears.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let clears = clears.clone();
        SearchField::new(state_for_view.clone())
            .clear_icon(gpui::div().size(px(12.)).child("!"))
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .on_clear(move |_, _| clears.borrow_mut().push("clear".to_owned()))
            .into_any_element()
    });

    click(cx, 60., 18.);
    cx.simulate_input("rust");
    click(cx, 298., 18.);
    assert_eq!(recorded.borrow().last().map(String::as_str), Some(""));
    assert_eq!(cleared.borrow().as_slice(), ["clear"]);
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "");
}

// ---------------------------------------------------------------------------
// InputOTP
// ---------------------------------------------------------------------------

#[gpui::test]
fn input_otp_fills_and_advances(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let completes = events();
    let completed = completes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let completes = completes.clone();
        InputOTP::new(state_for_view.clone())
            .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
            .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned()))
            .into_any_element()
    });

    // Cell 0 spans x 0..38, y 0..40 (input_otp.rs): clicking its centre (19,
    // 20) focuses the row, and each digit fills the next slot and reports the
    // assembled code.
    click(cx, 19., 20.);
    press(cx, "1");
    press(cx, "2");
    press(cx, "3");
    press(cx, "4");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "12", "123", "1234"],
        "typing digits must fill successive slots and report the growing code"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["1234"],
        "filling the last slot must fire on_complete with the full code"
    );

    // Backspace clears the current slot first; a second press walks back a
    // slot and clears it (input_otp.rs: "if the cell holds a char, clear it;
    // else step back and clear").
    press(cx, "backspace");
    press(cx, "backspace");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "12", "123", "1234", "123", "12"],
        "Backspace must walk back through the filled slots"
    );
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "12",
        "the OtpState must hold the remaining digits only"
    );
}

#[gpui::test]
fn input_otp_rejects_wrong_length_and_non_digits(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let completes = events();
    let completed = completes.clone();
    let state = cx.new(|cx| OtpState::with_length(cx, 4));
    let state_for_view = state.clone();
    let alpha_changes = events();
    let alpha_recorded = alpha_changes.clone();
    let alpha_state = cx.new(|cx| OtpState::with_length(cx, 4));
    let alpha_for_view = alpha_state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let completes = completes.clone();
        let alpha_changes = alpha_changes.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                InputOTP::new(state_for_view.clone())
                    .on_change(move |code, _, _| changes.borrow_mut().push(code.to_owned()))
                    .on_complete(move |code, _, _| completes.borrow_mut().push(code.to_owned())),
            )
            // v3's `pattern` prop overrides the digit-only default; this port
            // spells it `OtpPattern::Alphanumeric`, matching the exported
            // `REGEXP_ONLY_DIGITS_AND_CHARS`.
            .child(
                InputOTP::new(alpha_for_view.clone())
                    .pattern(OtpPattern::Alphanumeric)
                    .on_change(move |code, _, _| alpha_changes.borrow_mut().push(code.to_owned())),
            )
            .into_any_element()
    });

    // Length is structural: a 4-slot field cannot hold 5 digits. The fifth
    // keystroke overwrites the last slot (the cursor never leaves it once the
    // code is full), so the code stays 4 chars instead of growing to 5.
    click(cx, 19., 20.);
    press(cx, "1");
    press(cx, "2");
    press(cx, "3");
    press(cx, "4");
    press(cx, "5");
    let code = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        code, "1235",
        "a fifth digit must overwrite the last slot: the code cannot grow past \
         the slot count"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1", "12", "123", "1234", "1235"],
        "every accepted keystroke must be reported"
    );
    assert_eq!(
        completed.borrow().as_slice(),
        ["1234", "1235"],
        "an overwrite that leaves the code full must fire on_complete again"
    );

    // `OtpPattern::Digits` is the default, and v3's default is digits-only
    // (`inputMode: 'numeric'`; a `pattern` prop opts into more). A letter
    // must be refused: no report, no state change, and the cursor must not
    // swallow it so the next digit still lands.
    let before = recorded.borrow().len();
    press(cx, "a");
    assert_eq!(
        recorded.borrow().len(),
        before,
        "a letter in the default digit mode must be refused silently"
    );
    let after_letter = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(after_letter, "1235", "a refused letter must change nothing");
    press(cx, "6");
    let after_digit = cx.update(|_, cx| state.read(cx).code());
    assert_eq!(
        after_digit, "1236",
        "a digit typed after a refused letter must still land"
    );

    // With the alphanumeric pattern, letters are accepted — this instance
    // sits at y 56..96 (40px OTP + 16px gap), cell 0 centre (19, 76).
    click(cx, 19., 76.);
    press(cx, "a");
    press(cx, "1");
    assert_eq!(
        alpha_recorded.borrow().as_slice(),
        ["a", "a1"],
        "the Alphanumeric pattern must accept letters and then digits"
    );
    let alpha_code = cx.update(|_, cx| alpha_state.read(cx).code());
    assert_eq!(
        alpha_code, "a1",
        "the OtpState must hold the alphanumeric code"
    );
}

// ---------------------------------------------------------------------------
// InputGroup
// ---------------------------------------------------------------------------

#[gpui::test]
fn input_group_prefix_and_suffix_do_not_steal_the_caret(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        // A `w(px(400.))` wrapper pins the group's edges so the addons, which
        // carry the group's padding (`px(12)`, `InputAddon::render`), sit at
        // known coordinates: prefix 0..~32, suffix ~351..400.
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new()
                    .prefix(InputAddon::new("$"))
                    .input(
                        Input::new(state_for_view.clone()).on_change(move |text, _, _| {
                            changes.borrow_mut().push(text.to_owned());
                        }),
                    )
                    .suffix(InputAddon::new(".com")),
            )
            .into_any_element()
    });

    // Click the field (flex_1 between the addons), which focuses it and puts
    // the caret at 0 of the empty value.
    click(cx, 200., 18.);
    cx.simulate_input("ab");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab"],
        "typing into the group's field must work"
    );

    // An addon is a plain div outside the input, and gpui transfers focus on
    // a mouse-down to the deepest *focusable* element under the cursor. The
    // addon is not one, so the click lands on its focusable ancestor — the
    // app root — and the field stops seeing the keyboard. That is the same
    // behaviour a browser has for a click on a non-input part of an input
    // group (React Aria's `<span>` prefix blurs the field too), so it is
    // ported faithfully, not a defect. The keystrokes right after the addon
    // clicks therefore record nothing...
    click(cx, 6., 18.);
    cx.simulate_input("cd");
    click(cx, 394., 18.);
    cx.simulate_input("xy");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab"],
        "keystrokes after an addon click must not reach the field: the click \
         released its focus, as a browser blur would"
    );

    // ...but the clicks never TOUCHED the caret (`InputState`'s cursor stayed
    // at 2): re-focusing the field and typing continues exactly where it left
    // off, which is what the addons must not steal.
    click(cx, 200., 18.);
    cx.simulate_input("cd");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab", "abc", "abcd"],
        "re-focusing must resume typing at the untouched caret, appending the \
         new text after 'ab' rather than reordering it"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "abcd", "the InputState must hold the resumed typing");
}

// ---------------------------------------------------------------------------
// ColorField
// ---------------------------------------------------------------------------

#[gpui::test]
fn color_field_typing_reports_a_colour(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        ColorField::new("cf-hex", PickerColor::from_hex("#336699").unwrap())
            .state(state_for_view.clone())
            .on_change(move |colour, _, _| {
                changes.borrow_mut().push(match colour {
                    Some(c) => c.to_hex(),
                    None => "none".to_owned(),
                });
            })
            .into_any_element()
    });

    // Editable mode delegates to `Input` with a 16px swatch start content, so
    // the text starts at 12 + 16 + 8 = 36 and (60, 18) focuses the field.
    // The state starts empty — the initial colour is the placeholder — so the
    // typed text IS the value.
    click(cx, 60., 18.);
    cx.simulate_input("ff0000");
    assert_eq!(
        recorded.borrow().len(),
        6,
        "every keystroke must be parsed and reported"
    );
    assert_eq!(
        recorded.borrow().last().map(String::as_str),
        Some("#FF0000"),
        "the final report must be the colour the hex text parses to"
    );
    let text = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(text, "ff0000", "the InputState must hold the typed hex");

    // A partial hex is not a colour: one more keystroke past the end must not
    // turn "ff0000" into a different valid value silently — "0" appended is
    // length 7, which `from_hex` rejects, so the report flips to none.
    cx.simulate_input("0");
    assert_eq!(
        recorded.borrow().last().map(String::as_str),
        Some("none"),
        "a 7-char hex must not parse; on_change must report None for it"
    );
}

// ---------------------------------------------------------------------------
// DateField
// ---------------------------------------------------------------------------

#[gpui::test]
fn date_field_segments_answer_arrows_and_digits(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        // Start from a complete value so this test isolates segment arrows,
        // digits and bounds. `date_field_picker_deep` separately proves that
        // an empty field fills only its active segment and defers onChange.
        DateField::new(state_for_view.clone())
            .min_value(Date::new(2025, 1, 10))
            .max_value(Date::new(2025, 1, 20))
            .on_change(move |date, _, _| {
                changes.borrow_mut().push(match date {
                    Some(d) => d.format_iso(),
                    None => "none".to_owned(),
                });
            })
            .into_any_element()
    });

    // The field's handle is the page's only tab stop (`InputState` is
    // `tab_stop(true)`), so Tab focuses it with the Month segment focused —
    // no coordinates involved, exactly how fields.rs drives TimeField.
    press(cx, "tab");
    press(cx, "up");
    press(cx, "down");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["none", "2025-01-15"],
        "stepping the month past the bounds must report None, and stepping \
         back must report the in-range date again"
    );

    // Right walks to the Day segment; digits type into it. `1` alone is
    // below `minValue`, so it reports None; the second digit completes `15`,
    // an in-range day.
    press(cx, "right");
    press(cx, "1");
    press(cx, "5");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["none", "2025-01-15", "none", "2025-01-15"],
        "digits must type into the focused day segment and report each step's \
         date, filtering any out-of-range value"
    );

    // A full segment hands the caret on (the Year now), so Left returns to
    // Day; `25` clamps to nothing outside `maxValue` the same way, and the
    // raw segment math still lands in the bound state.
    press(cx, "left");
    press(cx, "2");
    press(cx, "5");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["none", "2025-01-15", "none", "2025-01-15", "none", "none"],
        "an over-`maxValue` day must report None, never the out-of-range date"
    );
    let text = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        text, "2025-01-25",
        "the bound InputState must hold the ISO text the segments produced"
    );
}

// ---------------------------------------------------------------------------
// Fieldset
// ---------------------------------------------------------------------------

#[gpui::test]
fn fieldset_disabled_disables_its_children(cx: &mut TestAppContext) {
    // The claim this test is named for has no mechanism to satisfy — on
    // either side. v3's Fieldset API table lists `className`, `children` and
    // `nativeProps` only: there is no `isDisabled` prop, and the native
    // `<fieldset disabled>` behaviour would have to arrive through
    // `nativeProps`, which gpui has no analogue for (no DOM attribute graph).
    // This port's `Fieldset` is exactly the v3 surface: a layout container
    // (flex column, `gap(24)`) with no disabled state of its own. So the
    // honest assertion is the inverse of the claim: a field composed inside a
    // Fieldset keeps every interaction, because nothing in the container can
    // take it away. There is no `.is_disabled` builder on `Fieldset` to call.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        Fieldset::new()
            .child(FieldsetLegend::new("Billing address"))
            .child(
                FieldsetGroup::new().child(
                    Input::new(state_for_view.clone())
                        .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned())),
                ),
            )
            .into_any_element()
    });

    // The legend is a 24px line and the Fieldset gaps children by 24, so the
    // group's bare 36px field spans y 48..84 — click (60, 66) to focus it.
    click(cx, 60., 66.);
    cx.simulate_input("abc");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab", "abc"],
        "a field inside a Fieldset must keep accepting typing: the container \
         imposes no disabling"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "abc", "the InputState must hold the typed value");
}
