//! `Slider.Thumb` and `ColorSwatchPicker.Item` per-part state.
//!
//! v3 documents `isDisabled` and `name` on the *thumb*, not the slider, and
//! `isDisabled` on the *item*, not the swatch picker. This port is monolithic
//! where v3 composes, so the part props project onto the root by index:
//! `Slider::disabled_keys` / `ColorSwatchPicker::disabled_keys` name the
//! immovable members (the same projection `RadioGroup::disabled_keys` gives
//! `Radio.isDisabled`), and a range's two named inputs use the
//! `DateRangePicker` convention (`startName`/`endName`) read back by
//! `form_fields`.
//!
//! Two harness facts these tests depend on, both recorded in AGENTS.md: a
//! mouse event hit-tests the *last rendered frame*, so every press and every
//! drag is followed by a redraw; and a drag's press reports a value of its
//! own before the move does.

mod harness;

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use harness::{click, events, open_host, press};
use herogpui_components::{ColorSwatchPicker, Form, FormData, PickerColor, Slider};

/// Pushes the pending frame through: events hit-test the last rendered frame.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// A drag is down, one move with the button held, then up -- a single jump
/// lands as a click and the component sees no motion at all.
fn drag(cx: &mut VisualTestContext, from: (f32, f32), to: (f32, f32)) {
    let modifiers = Modifiers::none();
    cx.simulate_mouse_down(point(px(from.0), px(from.1)), MouseButton::Left, modifiers);
    flush_frame(cx);
    cx.simulate_mouse_move(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    flush_frame(cx);
    cx.simulate_mouse_up(point(px(to.0), px(to.1)), MouseButton::Left, modifiers);
    flush_frame(cx);
}

/// A two-thumb slider with the lower thumb disabled: the contrast between the
/// two slots is what proves the state is per-thumb rather than wholesale --
/// with the whole-slider `is_disabled` nothing would move at all, while here
/// the free thumb answers every gesture and the disabled one never leaves 20.
///
/// The geometry is the established one: a horizontal slider is `w_full`, so
/// the 600px wrapper fixes the track length and a pointer x maps to
/// `x / 600 * 100`. The y = 9 line is the rail's centre (an 18px thumb on a
/// 4px rail). The roving stop (which thumb the keys move) starts on the first
/// *enabled* thumb, the radio group's rule for its single stop, so the keys
/// act on the free thumb from the start.
#[gpui::test]
fn slider_disabled_thumb_answers_neither_drag_nor_keys(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    // The slider is driven the way a controlled caller drives it: every
    // report is stored and rendered back in next frame, so a second key press
    // steps from the value the first one reported (the component itself does
    // not hold a multi-thumb copy).
    let current_for_view: std::rc::Rc<std::cell::RefCell<Vec<f32>>> =
        std::rc::Rc::new(std::cell::RefCell::new(vec![20., 80.]));
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        let current = current_for_view.clone();
        let values = current.borrow().iter().copied().collect::<Vec<_>>();
        gpui::div()
            .w(px(600.))
            .child(
                Slider::new("pt-slider", 0.)
                    .values(values)
                    .min_value(0.)
                    .max_value(100.)
                    .step(1.)
                    // `Slider.Thumb.isDisabled` on the lower thumb.
                    .disabled_keys([0])
                    .on_change_all(move |vs, _, _| {
                        *current.borrow_mut() = vs.to_vec();
                        let joined = vs
                            .iter()
                            .map(|v| format!("{v}"))
                            .collect::<Vec<_>>()
                            .join(",");
                        recorded.borrow_mut().push(joined);
                    })
                    .into_any_element(),
            )
            .into_any_element()
    });

    // One Tab reaches the slider, then the keys step the *free* thumb: the
    // roving stop skipped the disabled lower thumb at creation, and the
    // slider's own Tab cycle skips it again (there is no other enabled thumb
    // to rove to), so every report says 20 on the left. Each press is flushed
    // before the next, because events hit-test the last rendered frame and
    // the keyboard reads the values that frame was built from.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "right"); // free thumb: 80 -> 81
    flush_frame(cx);
    press(cx, "left"); // 81 -> 80
    flush_frame(cx);
    press(cx, "tab"); // rove: only one enabled thumb, so the stop stays put
    flush_frame(cx);
    press(cx, "right"); // 80 -> 81 again
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["20,81", "20,80", "20,81"],
        "the arrows and Tab must only ever move the enabled thumb"
    );

    // A drag aimed at the disabled thumb's own position (x = 120 is its value
    // 20) must not move it: the nearest *enabled* thumb follows the pointer
    // instead. The press reports the free thumb pulled onto the disabled
    // spot, and the pull to x = 150 (value 25) takes the free thumb to 25 --
    // slot 0 reads "20" in every single report.
    drag(cx, (120., 9.), (150., 9.));
    assert_eq!(
        seen.borrow().as_slice(),
        ["20,81", "20,80", "20,81", "20,20", "20,25"],
        "the disabled thumb must stay put however the pointer aims, while \
         the nearest enabled thumb follows"
    );

    // The contrast, stated once: the disabled value never changes.
    assert!(
        seen.borrow().iter().all(|pair| pair.starts_with("20,")),
        "the disabled thumb's value must be frozen in every report"
    );
}

/// A disabled swatch answers no click while its neighbours do, and it leaves
/// the tab order.
///
/// Cells are 32px with an 8px gap, so the three centres sit at x = 16, 48, 80
/// on the y = 16 centre line. The middle (green) item is disabled: clicking
/// it records nothing, while either neighbour does. The keyboard half is the
/// single-stop rule: a collection is *one* tab stop, so Tab enters the group
/// on the first enabled cell and does not walk it -- Tab cycles a page's
/// controls, and a second Tab leaves the group again (here it lands back on
/// the same stop, there being nothing beyond it in the bare host), so Enter
/// re-picks red. The arrows are what move inside, which the arrowing tests
/// below drive; the clicks above stay the proof that a disabled swatch
/// answers no press.
#[gpui::test]
fn swatch_picker_disabled_item_ignores_click_and_leaves_tab_order(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp", vec![red, green, blue])
            // `ColorSwatchPicker.Item.isDisabled` on the middle item.
            .disabled_keys([1])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    click(cx, 16., 16.);
    flush_frame(cx);
    click(cx, 48., 16.);
    flush_frame(cx);
    click(cx, 80., 16.);
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#F43F5E", "#3B82F6"],
        "the disabled swatch must not report a choice, its neighbours must"
    );

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "enter");
    assert_eq!(
        seen.borrow().as_slice(),
        ["#F43F5E", "#3B82F6", "#F43F5E"],
        "one tab stop: the second Tab cannot walk the group, so Enter \
         re-picks red rather than blue"
    );
}

/// v3's picker inherits React Aria's collection keyboard (3.51.0, the pinned
/// version): the arrow keys rove the *focus* between swatches, skipping a
/// disabled item the way every collection here skips `disabledKeys`. The
/// arrows select nothing: the picker is a listbox with `toggle` selection
/// (react-stately 3.49.0 defaults `selectionBehavior` to `'toggle'`, so
/// `selectOnFocus` is false), and Enter is the press that picks the focused
/// swatch -- the exact sequence the pinned React Aria ColorSwatchPicker test
/// drives: Tab, ArrowRight, then Enter.
#[gpui::test]
fn swatch_picker_arrows_skip_a_disabled_item(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-arrows", vec![red, green, blue])
            .disabled_keys([1])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    // Focus the first (red) cell, then the arrows rove the focus to the next
    // *enabled* item -- blue; green is disabled and must be skipped. The
    // arrow records no selection (focus-only, this listbox's toggle), so the
    // Enter that follows is the press -- and it picks blue, which is how the
    // test proves the arrow skipped the disabled swatch: an Enter landing on
    // the disabled cell would record nothing at all.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "right");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#3B82F6"],
        "the arrows must rove the focus to the next enabled swatch, skipping \
         the disabled one, and Enter must pick it"
    );
}

/// The ListBox keyboard, as the pinned React Aria test for ColorSwatchPicker
/// asserts it: the arrows rove the *focus* between the enabled swatches --
/// RAC's listbox is a `toggle` selection, so focusing does not select -- and
/// Enter picks the focused one. The disabled swatch is skipped by the arrows,
/// which is what an Enter landing on it (recording nothing) would have proven
/// if it had not been.
#[gpui::test]
fn swatch_picker_arrows_rove_focus_and_enter_selects(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-rove", vec![red, green, blue])
            .disabled_keys([1])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    // One Tab enters the picker's single stop on red, the first enabled
    // swatch. The arrow then roves the focus to blue, skipping the disabled
    // middle one -- and selects nothing, because RAC's `toggle` selection
    // keeps focus and selection apart until a press.
    press(cx, "tab");
    flush_frame(cx);
    press(cx, "right");
    flush_frame(cx);
    assert!(
        seen.borrow().is_empty(),
        "the arrows must move focus without selecting"
    );
    // Enter is the press: it picks the focused swatch, which is how the test
    // proves where the arrow took the focus -- an Enter on the disabled cell
    // would record nothing at all.
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#3B82F6"],
        "the arrows must rove to the next enabled swatch, skipping the \
         disabled one, and Enter must pick it"
    );
}

/// Home and End jump to the first and last *enabled* swatch -- RAC's delegate
/// gives the collection both keys, and `stops` keeps a disabled swatch out of
/// both ends, exactly as the arrows keep it out of the step.
#[gpui::test]
fn swatch_picker_home_end_skip_disabled_swatches(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-ends", vec![red, green, blue])
            .disabled_keys([1])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "end");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "home");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#3B82F6", "#F43F5E"],
        "End must jump to the last enabled swatch (skipping the disabled \
         middle one) and Home back to the first"
    );
}

/// The ends hold: RAC's listbox does not wrap its arrows (`shouldFocusWrap` is
/// false), so Left at the first swatch and Right at the last leave the focus
/// where it is -- the selection after the Enter on each proves the stop did
/// not join up.
#[gpui::test]
fn swatch_picker_arrows_clamp_at_the_ends(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-clamp", vec![red, green, blue])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "left");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "end");
    flush_frame(cx);
    press(cx, "right");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#F43F5E", "#3B82F6"],
        "Left at the first swatch and Right at the last must hold, not wrap"
    );
}

/// Every swatch disabled: an empty `stops` leaves no cell claiming the
/// group's handle, so the picker is not in the tab order at all -- Tab walks
/// past it and no key, or Enter, can ever report a choice.
#[gpui::test]
fn swatch_picker_all_disabled_leaves_the_tab_order(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-none", vec![red, green, blue])
            .disabled_keys([0, 1, 2])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert!(
        seen.borrow().is_empty(),
        "an all-disabled picker must not be reachable by tab or Enter"
    );
}

/// The roving stop clamps to the first enabled swatch, so a picker whose
/// first item is disabled is still reachable -- the initial cursor (0) lands
/// on the next enabled one, and Tab enters there rather than nowhere.
#[gpui::test]
fn swatch_picker_first_disabled_still_enters_on_the_next_enabled(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let red = PickerColor::from_hex("#F43F5E").unwrap();
    let green = PickerColor::from_hex("#10B981").unwrap();
    let blue = PickerColor::from_hex("#3B82F6").unwrap();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-first", vec![red, green, blue])
            .disabled_keys([0])
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#10B981"],
        "Tab must land on the first enabled swatch when the first one is disabled"
    );
}

/// A wrapped grid has more than one row, and RAC's grid delegate moves Up and
/// Down between the same column of the adjacent row. A row holds seven 32px
/// cells with an 8px gap under the 280px cap (`32n + 8(n - 1) <= 280`), so
/// the eighth swatch sits alone on the second row: Down from the first column
/// lands on it and Up brings the focus back.
#[gpui::test]
fn swatch_picker_grid_arrows_move_between_rows(cx: &mut TestAppContext) {
    let seen = events();
    let for_view = seen.clone();
    let swatches: Vec<PickerColor> = [
        "#F43F5E", "#10B981", "#3B82F6", "#F5A524", "#7828C8", "#0E8AAA", "#71717A", "#18181B",
    ]
    .into_iter()
    .map(|hex| PickerColor::from_hex(hex).unwrap())
    .collect();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ColorSwatchPicker::new("pt-csp-grid", swatches.clone())
            .on_change(move |c, _, _| recorded.borrow_mut().push(c.to_hex()))
            .into_any_element()
    });

    press(cx, "tab");
    flush_frame(cx);
    press(cx, "down");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "up");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    assert_eq!(
        seen.borrow().as_slice(),
        ["#18181B", "#F43F5E"],
        "Down and Up must move between the rows of a wrapped grid, in the \
         same column"
    );
}

/// A range slider with per-thumb names submits one named value per thumb, in
/// registration order -- the two distinct pairs a two-thumb range submits in
/// HTML (`<input type="range" name="min">` beside `<input ... name="max">`).
///
/// `Form` cannot discover a child's fields, so the slider hands its named
/// ends over the way `DateRangePicker::form_fields` does, and the caller
/// registers one `FormField` per thumb (the loop below is exactly how the
/// gallery wires it). `Form::submit_handler` is the handler a submit button
/// calls; driving it directly keeps the test to the pair the slider actually
/// submits, and the host renders the same control so the page has something
/// to draw.
#[gpui::test]
fn range_slider_submits_two_named_values(cx: &mut TestAppContext) {
    let seen = events();
    let recorded = seen.clone();
    let slider = Slider::new("pt-range-form", 0.)
        .values([20., 80.])
        .min_value(0.)
        .max_value(100.)
        .step(1.)
        .start_name("min")
        .end_name("max");
    let mut form = Form::new().on_submit(move |data: &FormData, _, _| {
        let joined = data
            .iter()
            .map(|(name, value)| format!("{name}={}", value.as_text()))
            .collect::<Vec<_>>()
            .join(",");
        recorded.borrow_mut().push(joined);
    });
    for field in slider.form_fields() {
        form = form.field(field);
    }
    let submit = form.submit_handler();
    let cx = open_host(cx, move || {
        Slider::new("pt-range-form", 0.)
            .values([20., 80.])
            .min_value(0.)
            .max_value(100.)
            .step(1.)
            .start_name("min")
            .end_name("max")
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        seen.borrow().as_slice(),
        ["min=20,max=80"],
        "a two-thumb slider must submit one named value per thumb, in order"
    );
}

/// A disabled thumb's field is omitted from the submission, exactly as an
/// HTML form skips a disabled `<input>`.
#[gpui::test]
fn disabled_range_thumb_submits_nothing(cx: &mut TestAppContext) {
    let seen = events();
    let recorded = seen.clone();
    let slider = Slider::new("pt-range-form-dis", 0.)
        .values([20., 80.])
        .min_value(0.)
        .max_value(100.)
        .step(1.)
        .start_name("min")
        .end_name("max")
        .disabled_keys([0]);
    let mut form = Form::new().on_submit(move |data: &FormData, _, _| {
        let joined = data
            .iter()
            .map(|(name, value)| format!("{name}={}", value.as_text()))
            .collect::<Vec<_>>()
            .join(",");
        recorded.borrow_mut().push(joined);
    });
    for field in slider.form_fields() {
        form = form.field(field);
    }
    let submit = form.submit_handler();
    let cx = open_host(cx, move || {
        Slider::new("pt-range-form-dis", 0.)
            .values([20., 80.])
            .min_value(0.)
            .max_value(100.)
            .step(1.)
            .start_name("min")
            .end_name("max")
            .disabled_keys([0])
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        seen.borrow().as_slice(),
        ["max=80"],
        "a disabled thumb's named input must not submit"
    );
}
