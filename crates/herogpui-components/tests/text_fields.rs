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
//!   misses, x = 20 focuses). With text in it, each paragraph reports its own
//!   painted bounds, and a click is placed against the one it lands in.
//! - `InputOTP` cells are 38x40 with an 8px gap (`input_otp.rs`), so cell *i*
//!   spans x 46i..46i+38 and every cell centre is y 20; a second instance on
//!   the same page sits `gap(16)` below, cell 0 centre y = 40 + 16 + 20 = 76.
//! - `InputGroup` is `min-h-9` with no padding of its own: the `InputAddon`s
//!   carry `px(12)` each (`input_group.rs`). In a `w(px(400.))` wrapper the
//!   addons are flush against the wrapper's edges — a click at x = 6 lands in
//!   the prefix's left padding and one at x = 394 in the suffix's right
//!   padding for any glyph width. v3.2.4's `InputGroupRoot.handleClick`
//!   focuses the contained input when a click lands on the group outside it,
//!   so an addon click leaves the field focused — and it never touches the
//!   caret, which is `InputState`'s alone and is written only by the field's
//!   own mouse-down (`window.focus` moves the handle, nothing else).
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

use gpui::{
    point, prelude::*, px, Bounds, Focusable, Modifiers, MouseButton, Pixels, TestAppContext,
    VisualTestContext,
};
use herogpui_components::{
    Button, ColorField, Date, DateField, FieldGroup, Fieldset, FieldsetLegend, Input, InputAddon,
    InputGroup, InputOTP, InputState, OtpPattern, OtpState, PickerColor, SearchField, TextArea,
    TextField,
};

use harness::{click, events, open_host, press};

/// One forced redraw, so `debug_bounds` sees the latest laid-out frame.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

const SEARCH_CLEAR_BOX: f32 = 20.;
const SEARCH_CLEAR_PRESS_SCALE: f32 = 0.93;

fn clear_probe(entity_id: u64) -> &'static str {
    Box::leak(format!("input-clear-{entity_id}").into_boxed_str())
}

fn clear_bounds(cx: &mut VisualTestContext, entity_id: u64) -> Bounds<Pixels> {
    cx.debug_bounds(clear_probe(entity_id))
        .unwrap_or_else(|| panic!("the SearchField clear button {entity_id} must paint"))
}

fn bounds_centre(bounds: Bounds<Pixels>) -> gpui::Point<Pixels> {
    point(
        bounds.origin.x + bounds.size.width / 2.,
        bounds.origin.y + bounds.size.height / 2.,
    )
}

fn near(value: Pixels, expected: f32) -> bool {
    (f32::from(value) - expected).abs() < 0.5
}

// ---------------------------------------------------------------------------
// TextField
// ---------------------------------------------------------------------------

#[gpui::test]
fn text_field_accepts_platform_text_without_a_printable_key(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "replace me"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TextField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .into_any_element()
    });
    click(cx, 60., 18.);
    cx.simulate_keystrokes("ctrl-a");
    cx.update(|window, app| {
        window.dispatch_keystroke(
            gpui::Keystroke {
                key: "unidentified".into(),
                key_char: Some("東京😀".into()),
                modifiers: Modifiers::default(),
            },
            app,
        );
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.value().to_owned()),
        "東京😀"
    );
    assert_eq!(recorded.borrow().as_slice(), ["東京😀"]);
}

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
    // (10, 40) is safely inside the 76px box. The field is empty, so there is
    // no paragraph to land in and the click only focuses; see
    // `text_area_click_places_the_caret_inside_a_wrapped_line` for the case
    // where it does move the caret.
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
fn search_field_clear_button_press_scales_centered_to_pinned_value(cx: &mut TestAppContext) {
    let clears = events();
    let state = cx.new(|cx| InputState::with_value(cx, "rust"));
    let state_for_view = state.clone();
    let entity_id = state.entity_id().as_u64();
    let recorded_for_view = clears.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded_for_view.clone();
        SearchField::new(state_for_view.clone())
            .on_clear(move |_, _| recorded.borrow_mut().push("clear".to_owned()))
            .into_any_element()
    });

    flush_frame(cx);
    let at_rest = clear_bounds(cx, entity_id);
    assert!(
        near(at_rest.size.width, SEARCH_CLEAR_BOX) && near(at_rest.size.height, SEARCH_CLEAR_BOX),
        "the resting SearchField clear slot must be a 20px square, got {at_rest:?}"
    );

    let at = bounds_centre(at_rest);
    cx.simulate_mouse_move(at, None, Modifiers::none());
    flush_frame(cx);
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    flush_frame(cx);

    let pressed = clear_bounds(cx, entity_id);
    let scaled = SEARCH_CLEAR_BOX * SEARCH_CLEAR_PRESS_SCALE;
    let inset = (SEARCH_CLEAR_BOX - scaled) / 2.;
    assert!(
        near(pressed.size.width, scaled) && near(pressed.size.height, scaled),
        "a pressed SearchField clear button must scale to 0.93 ({scaled}px), got {pressed:?}"
    );
    assert!(
        near(pressed.origin.x, f32::from(at_rest.origin.x) + inset)
            && near(pressed.origin.y, f32::from(at_rest.origin.y) + inset),
        "the clear scale must stay centered in its original slot, got {pressed:?}"
    );

    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        clears.borrow().as_slice(),
        ["clear"],
        "the completed pressed clear must still activate"
    );
}

#[gpui::test]
fn search_field_clear_button_press_scale_is_instant_with_reduced_motion(cx: &mut TestAppContext) {
    harness::still();
    let state = cx.new(|cx| InputState::with_value(cx, "rust"));
    let state_for_view = state.clone();
    let entity_id = state.entity_id().as_u64();
    let cx = open_host(cx, move || {
        SearchField::new(state_for_view.clone()).into_any_element()
    });

    flush_frame(cx);
    let at_rest = clear_bounds(cx, entity_id);
    let at = bounds_centre(at_rest);
    cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
    flush_frame(cx);

    let pressed = clear_bounds(cx, entity_id);
    let scaled = SEARCH_CLEAR_BOX * SEARCH_CLEAR_PRESS_SCALE;
    let inset = (SEARCH_CLEAR_BOX - scaled) / 2.;
    assert!(
        near(pressed.size.width, scaled) && near(pressed.size.height, scaled),
        "reduced motion must apply the clear scale immediately ({scaled}px), got {pressed:?}"
    );
    assert!(
        near(pressed.origin.x, f32::from(at_rest.origin.x) + inset)
            && near(pressed.origin.y, f32::from(at_rest.origin.y) + inset),
        "reduced-motion clear scale must stay centered, got {pressed:?}"
    );

    cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
}

#[gpui::test]
fn search_field_clear_button_is_inert_when_disabled_read_only_or_empty(cx: &mut TestAppContext) {
    let clears = events();
    let disabled = cx.new(|cx| InputState::with_value(cx, "disabled"));
    let read_only = cx.new(|cx| InputState::with_value(cx, "read-only"));
    let empty = cx.new(|cx| InputState::new(cx));
    let disabled_id = disabled.entity_id().as_u64();
    let read_only_id = read_only.entity_id().as_u64();
    let empty_id = empty.entity_id().as_u64();
    let clears_for_view = clears.clone();
    let cx = open_host(cx, move || {
        let disabled_clears = clears_for_view.clone();
        let read_only_clears = clears_for_view.clone();
        let empty_clears = clears_for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                SearchField::new(disabled.clone())
                    .is_disabled(true)
                    .on_clear(move |_, _| disabled_clears.borrow_mut().push("disabled".to_owned())),
            )
            .child(
                SearchField::new(read_only.clone())
                    .is_read_only(true)
                    .on_clear(move |_, _| {
                        read_only_clears.borrow_mut().push("read-only".to_owned());
                    }),
            )
            .child(
                SearchField::new(empty.clone())
                    .on_clear(move |_, _| empty_clears.borrow_mut().push("empty".to_owned())),
            )
            .into_any_element()
    });

    flush_frame(cx);
    for entity_id in [disabled_id, read_only_id, empty_id] {
        assert!(
            cx.debug_bounds(clear_probe(entity_id)).is_none(),
            "disabled, read-only, and empty SearchFields must not paint a clear button"
        );
    }

    for y in [18., 58., 98.] {
        let at = point(px(298.), px(y));
        cx.simulate_mouse_move(at, None, Modifiers::none());
        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
        flush_frame(cx);
        cx.simulate_mouse_up(at, MouseButton::Left, Modifiers::none());
        flush_frame(cx);
    }
    assert!(
        clears.borrow().is_empty(),
        "disabled, read-only, and invisible clear controls must never activate"
    );
}

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

    let field_focus = cx.update(|_, cx| state.read(cx).focus_handle(cx));
    assert!(
        cx.update(|window, cx| {
            window
                .focused(cx)
                .is_some_and(|focused| focused == field_focus)
        }),
        "pinned `preventFocusOnPress: true`: a pointer clear must restore \
         field focus"
    );

    // The field keeps the focus through the clear click, so typing continues
    // and Enter submits the value.
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

/// The pinned `useSearchField` hands the clear button `excludeFromTabOrder:
/// true` and `preventFocusOnPress: true`. The InputState-owned handle is not
/// a tab stop: Tab seats the input and wraps past the button. Enter/Space
/// activate only the *painted* button, via that same handle — a keyed probe
/// lives on a different element-id path and cannot prove this.
#[gpui::test]
fn search_field_clear_button_is_excluded_from_tab_order(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let clears = events();
    let cleared = clears.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "rust"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let clears = clears.clone();
        SearchField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .on_clear(move |_, _| clears.borrow_mut().push("clear".to_owned()))
            .into_any_element()
    });
    let field_focus = cx.update(|_, cx| state.read(cx).focus_handle(cx));
    let clear_handle = cx.update(|_, cx| state.read(cx).clear_focus_handle());

    // Tab seats the input. The clear button is not the next stop, so the next
    // Tab wraps straight back to the field without ever focusing the button.
    press(cx, "tab");
    assert!(
        cx.update(|window, cx| {
            window
                .focused(cx)
                .is_some_and(|focused| focused == field_focus)
        }),
        "Tab must seat the input"
    );
    for _ in 0..2 {
        press(cx, "tab");
        assert!(
            cx.update(|window, cx| {
                window
                    .focused(cx)
                    .is_some_and(|focused| focused == field_focus)
            }),
            "the clear button must stay out of the tab order (pinned \
             `excludeFromTabOrder: true`); Tab must wrap back to the field"
        );
    }

    // The InputState-owned handle is the one `track_focus` paints on the
    // button. Focusing it, then flushing so paint sees `is_focused`, is the
    // real GPUI Enter/Space click path.
    cx.update(|window, cx| window.focus(&clear_handle, cx));
    flush_frame(cx);
    assert!(
        cx.update(|window, _| clear_handle.is_focused(window)),
        "window.focus on the InputState handle must seat the painted clear \
         button"
    );
    press(cx, "enter");
    assert!(
        cx.update(|window, cx| {
            window
                .focused(cx)
                .is_some_and(|focused| focused == field_focus)
        }),
        "clear activation must return focus to the input"
    );

    state.update(cx, |state, cx| {
        state.set_value("rust");
        cx.notify();
    });
    flush_frame(cx);
    cx.update(|window, cx| window.focus(&clear_handle, cx));
    flush_frame(cx);
    assert!(
        cx.update(|window, _| clear_handle.is_focused(window)),
        "window.focus on the InputState handle must seat the painted clear \
         button"
    );
    press(cx, "space");
    assert!(
        cx.update(|window, cx| {
            window
                .focused(cx)
                .is_some_and(|focused| focused == field_focus)
        }),
        "clear activation must return focus to the input"
    );

    assert_eq!(
        recorded.borrow().as_slice(),
        ["", ""],
        "keyboard activation must report the clear through on_change"
    );
    assert_eq!(
        cleared.borrow().as_slice(),
        ["clear", "clear"],
        "Enter and Space on the focused clear button must report on_clear"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "", "keyboard activation must empty the InputState");
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

    // v3.2.4's `InputGroupRoot.handleClick` focuses the contained input when
    // a click lands on the group outside it, so the addon clicks below must
    // leave the field focused and typing must keep reaching it. Focusing
    // never moves the caret — it is `InputState`'s, written only by the
    // field's own mouse-down — and Left put it at 1, so the text must grow
    // mid-string: after the prefix click "cd" turns "ab" into "acdb" ("a|b"
    // -> "a|cdb" -> "acd|b"), not "abcd" (caret dropped to the end) nor
    // "cdab" (caret dropped to 0); the cursor now rests at 3, so the suffix
    // click must not move it and "x" lands between d and b: "acdxb".
    press(cx, "left");
    click(cx, 6., 18.);
    cx.simulate_input("cd");
    click(cx, 394., 18.);
    cx.simulate_input("x");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab", "acb", "acdb", "acdxb"],
        "an addon click must focus the group's field and leave the caret \
         exactly where it was: typing continues mid-string"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "acdxb",
        "the InputState must hold the text typed through the addon clicks"
    );
}

#[gpui::test]
fn input_group_disabled_propagates_to_the_field(cx: &mut TestAppContext) {
    // v3 puts `isDisabled` on the TextField around the group, and the browser
    // then holds a *disabled* `<input>`: it takes no caret, keeps no focus and
    // answers no key. The group's own flag must reach the field it contains —
    // dimming alone would leave a control that looks off and still types.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new()
                    .is_disabled(true)
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
    flush_frame(cx);

    // The field spans between the addons (x ~32..346, y 0..36): a direct
    // click must not focus it, and neither keys typed after the click nor
    // keys typed blind may reach the disabled state.
    click(cx, 200., 18.);
    cx.simulate_input("x");
    press(cx, "a");
    // v3.2.4's `InputGroupRoot.handleClick` runs `input.focus()` on a click
    // outside the field — a browser no-ops that on a disabled input, so the
    // addon click must leave nothing to type into either.
    click(cx, 6., 18.);
    cx.simulate_input("y");
    assert!(
        recorded.borrow().is_empty(),
        "a group-disabled field must record no typing at all, got {:?}",
        recorded.borrow()
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "",
        "a group-disabled field must hold no text: is_disabled propagates to \
         the inner Input"
    );
}

#[gpui::test]
fn input_group_textarea_click_exception_and_direct_focus(cx: &mut TestAppContext) {
    // Pinned v3.2.4 `InputGroupRoot.handleClick` looks the field up with
    // `querySelector("input")` — a `<textarea>` is not an `input`, so a group
    // holding only a TextArea gets NO click-to-focus: the addon click below
    // must leave the textarea unfocused, while a click inside the textarea
    // itself still focuses it.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new().prefix(InputAddon::new("Note")).text_area(
                    TextArea::new(state_for_view.clone())
                        .rows(3)
                        .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned())),
                ),
            )
            .into_any_element()
    });
    flush_frame(cx);

    // The addon slot is top-aligned with the pinned 8px top padding, so the
    // addon's box spans y 8..28 inside the prefix's x 0..~40: a click there
    // is a click on the group *outside* the textarea, which v3.2.4 ignores
    // for a textarea-only group.
    click(cx, 6., 13.);
    cx.simulate_input("x");
    assert!(
        recorded.borrow().is_empty(),
        "a click on a textarea-only group's addon must not focus the textarea \
         (querySelector(\"input\") finds nothing), got {:?}",
        recorded.borrow()
    );

    // Positive control: the textarea itself is focusable — a click inside it
    // (76px tall for rows 3, wide in the 400px group) must focus it, so the
    // empty recording above is the pinned exception and not a broken field.
    click(cx, 200., 38.);
    cx.simulate_input("ab");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["a", "ab"],
        "a click inside the textarea must focus it: the exception only covers \
         clicks outside the field"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "ab", "the InputState must hold the typed text");
}

#[gpui::test]
fn input_group_button_suffix_presses_then_hands_focus_to_the_field(cx: &mut TestAppContext) {
    // v3.2.4 `handleClick` focuses the contained input on any click outside
    // it — a focusable suffix button included. The button keeps its own press
    // (gpui decides clicks by hover, not focus, so the focus hand-over on
    // mouse-down does not cancel the mouse-up click) and the field ends up
    // holding the focus, so the next keystrokes type into it.
    let changes = events();
    let recorded = changes.clone();
    let presses = events();
    let pressed = presses.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let presses = presses.clone();
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
                    .suffix(
                        Button::new("ig-go")
                            .label("Go")
                            .on_press(move |_, _, _| presses.borrow_mut().push("press".into())),
                    ),
            )
            .into_any_element()
    });
    flush_frame(cx);

    // The suffix Button ("Go", md padding) hangs off the row's right edge in
    // the 400px wrapper: the click lands in its right padding at (394, 18) —
    // on the group, outside the field between the addons.
    click(cx, 394., 18.);
    assert_eq!(
        pressed.borrow().as_slice(),
        ["press"],
        "the focusable suffix must keep its own action"
    );
    let field_handle = cx.update(|_, cx| state.read(cx).focus_handle(cx));
    let focused_on_field =
        cx.update(|window, cx| window.focused(cx).is_some_and(|held| held == field_handle));
    assert!(
        focused_on_field,
        "the click-to-focus must take the focus back from the pressed \
         suffix button and give it to the field"
    );
    cx.simulate_input("x");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["x"],
        "typing after the suffix click must reach the field"
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "x",
        "the InputState must hold the text typed through the suffix button"
    );
}

#[gpui::test]
fn input_group_field_disabled_refuses_addon_focus_from_the_first_frame(cx: &mut TestAppContext) {
    // A field disabled while the group stays enabled must refuse the addon
    // click-to-focus from the FIRST frame. The group used to read the
    // disabled state through the mirror `Input::render` writes into the
    // shared state — a trace of the last rendered frame, and untouched
    // before the first one — so the very first addon click ran
    // `input.focus()` on a disabled field, which a browser refuses.
    //
    // The focusable suffix is the witness. A press on it must leave the
    // BUTTON holding the focus (its own transfer ran deeper, and its handle
    // lives in the focus tree, so the app root does not reclaim it): Enter
    // then activates it a second time. A wrongly focused disabled field,
    // by contrast, is not in the focus tree, so the app root takes the
    // focus back on the next paint and Enter activates nothing.
    let changes = events();
    let recorded = changes.clone();
    let presses = events();
    let pressed = presses.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let presses = presses.clone();
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new()
                    .prefix(InputAddon::new("$"))
                    .input(
                        Input::new(state_for_view.clone())
                            .is_disabled(true)
                            .on_change(move |text, _, _| {
                                changes.borrow_mut().push(text.to_owned());
                            }),
                    )
                    .suffix(
                        Button::new("ig-disabled-go")
                            .label("Go")
                            .on_press(move |_, _, _| presses.borrow_mut().push("press".into())),
                    ),
            )
            .into_any_element()
    });
    let field_handle = cx.update(|_, cx| state.read(cx).focus_handle(cx));

    // The click below runs against the frame `open_host` already painted —
    // the FIRST frame, whose listener decision the mirror read made before
    // any `Input::render` had ever run. It is deliberately the first
    // interaction: a click repaints, and every later frame reads a mirror
    // the previous frame already corrected.
    //
    // The suffix button keeps its press, and the focus must stay with the
    // button — proven by Enter activating it again, which only works while
    // the button holds the focus. A wrongly focused disabled field is not
    // in the focus tree, so the app root takes the focus back on the next
    // paint and Enter activates nothing.
    click(cx, 394., 18.);
    assert_eq!(
        pressed.borrow().as_slice(),
        ["press"],
        "the suffix button must keep its own press beside a disabled field"
    );
    let focused_on_field = |cx: &mut VisualTestContext| {
        cx.update(|window, cx| window.focused(cx).is_some_and(|held| held == field_handle))
    };
    assert!(
        !focused_on_field(cx),
        "a field disabled inside an enabled group must refuse the addon \
         click-to-focus on the first frame"
    );
    flush_frame(cx);
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["press", "press"],
        "the button must still hold the focus after the click: a disabled \
         field must not have taken it from the first frame on"
    );

    // And the plain addons refuse the click-to-focus on the later frames too.
    click(cx, 6., 18.);
    assert!(
        !focused_on_field(cx),
        "the addon click must never focus a disabled field"
    );
    cx.simulate_input("y");

    cx.simulate_input("z");
    assert!(
        recorded.borrow().is_empty(),
        "no keystroke may reach the disabled field, got {:?}",
        recorded.borrow()
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "",
        "the disabled field must hold no text: the group's click-to-focus \
         reads the field's own flag, not a rendered mirror"
    );
}

#[gpui::test]
fn input_group_disabled_group_propagates_to_the_textarea(cx: &mut TestAppContext) {
    // The group's flag reaches the converted multi-line field the same way —
    // `text_area` becomes the one held `Input` slot — so a click inside the
    // box focuses nothing and no key reaches the state.
    let changes = events();
    let recorded = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new().is_disabled(true).text_area(
                    TextArea::new(state_for_view.clone())
                        .rows(3)
                        .on_change(move |text, _, _| {
                            changes.borrow_mut().push(text.to_owned());
                        }),
                ),
            )
            .into_any_element()
    });
    flush_frame(cx);

    // The textarea spans the full row (no addons), 76px tall for rows 3.
    click(cx, 200., 38.);
    cx.simulate_input("x");
    let field_handle = cx.update(|_, cx| state.read(cx).focus_handle(cx));
    let focused_on_field =
        cx.update(|window, cx| window.focused(cx).is_some_and(|held| held == field_handle));
    assert!(
        !focused_on_field,
        "a group-disabled textarea must not take the focus"
    );
    assert!(
        recorded.borrow().is_empty(),
        "a group-disabled textarea must record no typing, got {:?}",
        recorded.borrow()
    );
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        value, "",
        "the propagation must reach the converted multi-line field"
    );
}

#[gpui::test]
// The geometry assertions below compare whole-pixel layout results — a 400px
// request laid out by taffy is exactly 400.0, with no fractional step to
// tolerate — so the exact comparisons are on purpose.
#[allow(clippy::float_cmp)]
fn input_group_full_width_stretches_the_outer_wrapper(cx: &mut TestAppContext) {
    // `.input-group--full-width` is `w-full`, and in this port that width has
    // to reach the OUTER wrapper too: a `w_full` child resolves against a
    // content-sized parent and stretches nothing. Measured in an
    // items-start column, where an unstretched group hugs its content.
    let changes = events();
    let state = cx.new(|cx| InputState::new(cx));
    let entity = state.entity_id().as_u64();
    let plain = cx.new(|cx| InputState::new(cx));
    let plain_entity = plain.entity_id().as_u64();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        gpui::div()
            .w(px(400.))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(16.))
            .child(
                InputGroup::new()
                    .full_width(true)
                    .prefix(InputAddon::new("$"))
                    .input(Input::new(state.clone()).on_change(move |text, _, _| {
                        changes.borrow_mut().push(text.to_owned());
                    }))
                    .suffix(InputAddon::new("USD")),
            )
            .child(
                InputGroup::new()
                    .prefix(InputAddon::new("$"))
                    .input(Input::new(plain.clone()))
                    .suffix(InputAddon::new("USD")),
            )
            .into_any_element()
    });
    flush_frame(cx);

    // `debug_bounds` takes a `&'static str` and the probe keys carry the
    // input's entity id, so the formatted keys are leaked for the test's
    // lifetime.
    let key = |entity: u64, suffix: &str| -> &'static str {
        Box::leak(format!("input-group-{entity}-{suffix}").into_boxed_str())
    };
    let full = cx
        .debug_bounds(key(entity, "group"))
        .expect("the full-width group box must be measurable");
    let plain_bounds = cx
        .debug_bounds(key(plain_entity, "group"))
        .expect("the plain group box must be measurable");
    assert_eq!(
        f32::from(full.size.width),
        400.0,
        "full_width must stretch the group to its container's width, got {}",
        f32::from(full.size.width)
    );
    assert!(
        f32::from(plain_bounds.size.width) < 400.0,
        "without full_width the group must hug its content, got {}",
        f32::from(plain_bounds.size.width)
    );
}

#[gpui::test]
// The geometry assertions below compare whole-pixel sums — 20px lines and the
// pinned 8px slot padding laid out by taffy — so the exact comparisons are on
// purpose.
#[allow(clippy::float_cmp)]
fn input_group_textarea_auto_height_and_top_aligned_addons(cx: &mut TestAppContext) {
    // `:has([data-slot="input-group-textarea"])` switches the group to
    // `items-start` with `height: auto`, and gives each addon
    // `padding-top: 0.5rem` — the 8px that used to be faked by hand in the
    // gallery demo. Probed with fixed-height slots: the prefix wrapper must
    // start at the group's top and stand 8px taller than its 20px probe.
    let state = cx.new(|cx| InputState::new(cx));
    let entity = state.entity_id().as_u64();
    let cx = open_host(cx, move || {
        gpui::div()
            .w(px(400.))
            .child(
                InputGroup::new()
                    .prefix(
                        // A fixed 20px probe: any glyph metrics stay out of the
                        // arithmetic.
                        gpui::div().w(px(16.)).h(px(20.)),
                    )
                    .suffix(gpui::div().w(px(16.)).h(px(20.)))
                    .text_area(TextArea::new(state.clone()).rows(3)),
            )
            .into_any_element()
    });
    flush_frame(cx);

    let key = |suffix: &str| -> &'static str {
        Box::leak(format!("input-group-{entity}-{suffix}").into_boxed_str())
    };
    let group = cx
        .debug_bounds(key("group"))
        .expect("the textarea group box must be measurable");
    assert_eq!(
        f32::from(group.size.height),
        76.0,
        "rows(3) is 20*3 + 16 = 76px and `height: auto` must let the group \
         grow to it, got {}",
        f32::from(group.size.height)
    );
    let prefix = cx
        .debug_bounds(key("prefix"))
        .expect("the textarea group's addon slot must be measurable");
    assert_eq!(
        f32::from(prefix.origin.y),
        f32::from(group.origin.y),
        "the addon slot must be top-aligned with the textarea (items-start), \
         not centred in the taller group"
    );
    assert_eq!(
        f32::from(prefix.size.height),
        28.0,
        "the addon slot carries the pinned 8px top padding over its 20px \
         probe, got {}",
        f32::from(prefix.size.height)
    );
    let suffix = cx
        .debug_bounds(key("suffix"))
        .expect("the textarea group's suffix slot must be measurable");
    assert_eq!(
        f32::from(suffix.origin.y),
        f32::from(group.origin.y),
        "both addon slots must top-align"
    );
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
                FieldGroup::new().child(
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

/// A pointer click inside a WRAPPED paragraph places the caret where it landed.
///
/// This was recorded as impossible (`behaviour_audit`'s
/// `no-wrapped-line-metrics`) on the grounds that gpui reports no position for
/// a wrapped line. It does: `shape_text` answers `WrappedLine`, which derefs to
/// the layout that owns `closest_index_for_position`. The caret is asserted
/// through the next keystroke, which is where it actually matters.
#[gpui::test]
fn text_area_click_places_the_caret_inside_a_wrapped_line(cx: &mut TestAppContext) {
    // One long word-wrapped paragraph, no newlines: every visual line break
    // here is one gpui chose, which is exactly the case that was unreachable.
    let state = cx.new(|cx| InputState::with_value(cx, "aaaa bbbb cccc dddd eeee ffff gggg hhhh"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        // 200px forces the text to wrap into several visual lines inside one
        // paragraph; the TextArea is `rows(3)` = 76px tall at the origin.
        gpui::div()
            .w(px(200.))
            .child(TextArea::new(state_for_view.clone()))
            .into_any_element()
    });

    let end = cx.update(|_, cx| state.read(cx).value().chars().count());

    // Click near the start of the first visual line, well before the end. A
    // plain click leaves no anchor, so the caret is not observable directly --
    // the keystroke below is what reports where it went.
    click(cx, 16., 14.);
    cx.simulate_input("X");
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert!(
        !value.ends_with('X'),
        "typing after a click in a wrapped paragraph must insert at the caret, \
         not append; got {value:?}"
    );
    assert_eq!(
        value.chars().count(),
        end + 1,
        "the click must move the caret, not select and replace"
    );
}

#[gpui::test]
fn input_addons_and_custom_otp_slots_keep_twenty_pixel_lines(cx: &mut TestAppContext) {
    for kind in 0..3 {
        for leading in [None, Some(48.)] {
            let state = cx.new(|cx| InputState::new(cx));
            let otp = cx.new(|cx| OtpState::with_length(cx, 1));
            let cx = open_host(cx, move || {
                let probe = || {
                    gpui::div()
                        .debug_selector(|| "input-leading-text".into())
                        .child("$")
                };
                let control = match kind {
                    0 => Input::new(state.clone())
                        .start_content(probe())
                        .into_any_element(),
                    1 => InputGroup::new()
                        .prefix(
                            gpui::div()
                                .debug_selector(|| "input-leading-text".into())
                                .child(InputAddon::new("$")),
                        )
                        .input(Input::new(state.clone()))
                        .into_any_element(),
                    _ => InputOTP::new(otp.clone())
                        .slot(move |_, _| probe().into_any_element())
                        .into_any_element(),
                };
                gpui::div()
                    .w(px(300.))
                    .when_some(leading, |el, leading| el.line_height(px(leading)))
                    .child(control)
                    .into_any_element()
            });
            assert_eq!(
                cx.debug_bounds("input-leading-text")
                    .expect("slot text paints")
                    .size
                    .height,
                px(20.),
                "kind={kind}, host={leading:?}"
            );
        }
    }
}
