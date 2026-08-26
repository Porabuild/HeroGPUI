//! Behaviour tests for the inverted value/output render props: the closures
//! v3 passes render-prop state into, and this port computes and hands over.
//!
//! Everything static about them is measured by the `.shots/*.py` audits; these
//! tests prove each closure *runs at all* (the `Button::content` panic class
//! was a closure the gallery never rendered), that it is handed the right
//! values as the state changes, and that `default_children` -- an `AnyElement`
//! built once and consumed once -- survives being handed straight back, which
//! is v3's own placeholder idiom.
//!
//! Two families, with different proof shapes:
//!
//! - `SelectionValue` (Select / Autocomplete / ComboBox): the closure reads
//!   the selection on every render, so a pick is observed on the next frame.
//!   Geometry, derived from the components' own constants and reused from
//!   pickers.rs: every trigger is 36px tall at the window origin, so its
//!   centre is (60, 18). A Select panel hangs 6px below the trigger with `py(6)`
//!   and 36px rows: row *i* centres at y = 66 + 36i. The Autocomplete panel
//!   stacks a search field first, so row *i* is at y = 124 + 36i ("plus up to
//!   6px of entry-zoom padding", which every phase of the zoom keeps the click
//!   inside). The ComboBox panel is `p(4)`, so row *i* centres at y = 64 + 36i;
//!   its field is 320px wide, so the chevron that opens the list is at
//!   (298, 18). The closure's own output sits *inside* the trigger (Select,
//!   Autocomplete) or under the field (ComboBox), never over the popover rows.
//! - `FieldFocus` (Input, TextField, SearchField, NumberField, TimeField,
//!   DateField, ColorField): the closure *replaces* the field's whole stack,
//!   so its output must draw the parts -- and the focusable input, which this
//!   port composes by rendering the same field bound to the same state
//!   (`Input::content` hands over the state's own handle). That is how the
//!   values can change at all: a click on the replacement field focuses the
//!   shared state handle, and the keyboard-visible flag comes from
//!   `util::set_focus_visible`, which the real app focus root and the shared
//!   harness root both run on a key event. Every replacement field
//!   is 36px tall at the origin (`util::FIELD_HEIGHT`), so the click is at
//!   (60, 18).
//!
//! Numbers are never compared for equality (`clippy::float_cmp` is denied):
//! colours compare as `PickerColor::to_hex()` strings computed with the port's
//! own colour math, and percentages/floats compare as formatted strings
//! (`"{:.0}"` of the percentage, or the component's own formatted `valueText`,
//! which `NumberFormat::percent` writes as `"25%"`).

mod harness;

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::{point, prelude::*, px, Modifiers, MouseButton, TestAppContext, VisualTestContext};
use herogpui_components::{
    util, Autocomplete, CloseButton, ColorChannel, ColorField, ColorSlider, ComboBox, DateField,
    Input, InputState, Meter, NumberField, NumberFormat, NumberState, PickerColor, ProgressBar,
    ProgressCircle, SearchField, Select, SelectionMode, Slider, Switch, TextField, TimeField,
    TimeState,
};

use harness::{click, events, open_host, press};

/// Forces the frame that carries the state a handler just changed.
///
/// A closure set through the value/output builders runs during *render*, so
/// the state a click or keystroke just changed is only visible to it on the
/// next frame. `window.refresh()` produces that frame on demand, which is
/// also how the hover/press one-frame lag is turned into a determinism.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// The snapshot the last render's closure left behind.
fn last_string(seen: &Rc<RefCell<Vec<String>>>) -> String {
    seen.borrow()
        .last()
        .expect("the closure must have run at least once")
        .clone()
}

// ---------------------------------------------------------------------------
// Select.Value / Autocomplete.Value / ComboBox.Value
// ---------------------------------------------------------------------------

/// `Select.Value` renders from the first frame (no panic — the `Button::content`
/// class of defect would die here), is handed `isPlaceholder=true` with empty
/// selections while nothing is chosen, and then observes the pick: the row
/// click moves the uncontrolled selection and the next frame's closure sees
/// `selectedItems=["Alpha"]` at index 0 with the placeholder gone. For the
/// placeholder case the closure hands `defaultChildren` straight back, which
/// v3's own examples do — consuming the `AnyElement` exactly once.
#[gpui::test]
fn select_value_content_hands_placeholder_then_pick(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Select::new(
            "sel-vc",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .value_content(move |v: util::SelectionValue<'_>| {
            let items = v
                .selected_items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let indices = v
                .selected_indices
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            record.borrow_mut().push(format!(
                "{}|{}|{}|{}",
                v.is_placeholder, items, indices, v.selected_text
            ));
            if v.is_placeholder {
                v.default_children
            } else {
                gpui::div()
                    .child(v.selected_text.to_owned())
                    .into_any_element()
            }
        })
        .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "true|||",
        "nothing is chosen, so the first render must report the placeholder"
    );

    // Open the trigger, then pick the first row: its centre is y = 66 + 36*0.
    click(cx, 60., 18.);
    click(cx, 60., 66.);
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Alpha|0|Alpha",
        "the frame after the pick must see the chosen option by text and index"
    );
}

/// `selectionMode="multiple"` is the case `Select.Value` exists for: the
/// trigger's built-in text would shrink a long selection to "N selected", so
/// a closure draws all of it. Each pick accumulates through the caller's own
/// set (the port reports `selected_indices` back and stores nothing), and the
/// closure observes every item with its text and index.
#[gpui::test]
fn select_value_content_multiple_lists_every_item(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let selection = Rc::new(RefCell::new(BTreeSet::<usize>::new()));

    let cx = open_host(cx, move || {
        let record = record.clone();
        let selection = selection.clone();
        let now = selection.borrow().iter().copied().collect::<Vec<_>>();
        Select::new(
            "sel-vc-multi",
            vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
        )
        .selection_mode(SelectionMode::Multiple)
        .selected_indices(now)
        .on_selection_change_all(move |keys, window, _| {
            *selection.borrow_mut() = keys.iter().copied().collect();
            // The port hands the merged set back rather than storing it, so
            // the caller must render it back in for the next frame to differ.
            window.refresh();
        })
        .value_content(move |v: util::SelectionValue<'_>| {
            let items = v
                .selected_items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let indices = v
                .selected_indices
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            record.borrow_mut().push(format!(
                "{}|{}|{}|{}",
                v.is_placeholder, items, indices, v.selected_text
            ));
            if v.is_placeholder {
                v.default_children
            } else {
                gpui::div()
                    .child(v.selected_text.to_owned())
                    .into_any_element()
            }
        })
        .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "true|||",
        "an empty multiple selection must still report the placeholder"
    );

    // Multiple mode keeps the panel open between picks, so row 0 then row 1
    // at y = 66 and y = 102 both land.
    click(cx, 60., 18.);
    click(cx, 60., 66.);
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Alpha|0|Alpha",
        "the first pick must be observed on the next frame"
    );
    click(cx, 60., 102.);
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Alpha,Beta|0,1|Alpha, Beta",
        "the second pick must join the first, by text and by index"
    );
}

/// `Autocomplete.Value` renders from the first frame, hands the placeholder
/// (with `defaultChildren` handed back), and then observes a click on the
/// suggestion list: the row's text and index appear on the frame after the
/// pick. Row 0 of the Autocomplete popover is at y = 124 (42 panel top + 8
/// section padding + 4 search wrapper + 36 search field + 6 list padding +
/// half a 36px row).
#[gpui::test]
fn autocomplete_value_content_hands_placeholder_then_pick(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let record = record.clone();
        Autocomplete::new(
            state_for_view.clone(),
            vec!["Alpha".into(), "Rust".into(), "Go".into()],
        )
        .value_content(move |v: util::SelectionValue<'_>| {
            let items = v
                .selected_items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let indices = v
                .selected_indices
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            record.borrow_mut().push(format!(
                "{}|{}|{}|{}",
                v.is_placeholder, items, indices, v.selected_text
            ));
            if v.is_placeholder {
                v.default_children
            } else {
                gpui::div()
                    .child(v.selected_text.to_owned())
                    .into_any_element()
            }
        })
        .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "true|||",
        "nothing is chosen, so the first render must report the placeholder"
    );

    click(cx, 60., 18.);
    click(cx, 60., 124.);
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Alpha|0|Alpha",
        "the frame after the pick must see the chosen suggestion"
    );
}

/// `ComboBox.Value` draws *below* the input, so it renders from the first
/// frame with the placeholder and observes the pick from the list. The
/// chevron at (298, 18) opens the list without typing; row 0 is at
/// y = 64 (`p(4)` panel, 36px rows).
// `ComboBox::value_content` draws under the field, while the popover is anchored
// to the input group. A value row therefore cannot move the panel away from the
// field geometry.
#[gpui::test]
fn combo_box_value_content_hands_placeholder_then_pick(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let record = record.clone();
        let changes = changes.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .placeholder("Search")
        .on_selection_change_all(move |keys, _, _| {
            changes.borrow_mut().push(keys.len().to_string());
        })
        .value_content(move |v: util::SelectionValue<'_>| {
            let items = v
                .selected_items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let indices = v
                .selected_indices
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            record.borrow_mut().push(format!(
                "{}|{}|{}|{}",
                v.is_placeholder, items, indices, v.selected_text
            ));
            if v.is_placeholder {
                v.default_children
            } else {
                gpui::div()
                    .child(v.selected_text.to_owned())
                    .into_any_element()
            }
        })
        .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "true|||",
        "nothing is chosen, so the first render must report the placeholder"
    );

    click(cx, 298., 18.);
    click(cx, 60., 64.);
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Typst|0|Typst",
        "the frame after the pick must see the chosen item"
    );

    click(cx, 60., 18.);
    press(cx, "ctrl-a");
    press(cx, "backspace");
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "true|||",
        "clearing an uncontrolled ComboBox input must clear ComboBox.Value's selection"
    );
    assert_eq!(
        changed.borrow().as_slice(),
        ["0"],
        "clearing the input must report the empty single selection"
    );
}

/// Pinned React Stately reports `null` when a controlled ComboBox input is
/// cleared, but leaves the selected value with its owner. The slice callback
/// represents that null as an empty selection; the value slot must not change
/// until the owner accepts the request.
#[gpui::test]
fn combo_box_controlled_value_content_waits_for_clear_owner(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::new(cx));
    state.update(cx, |state, _| state.set_value("Typst"));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let record = record.clone();
        let changes = changes.clone();
        ComboBox::new(
            state_for_view.clone(),
            vec!["Typst".into(), "Rust".into(), "Go".into()],
        )
        .selected_keys(["Typst".into()])
        .on_selection_change_all(move |keys, _, _| {
            changes.borrow_mut().push(keys.len().to_string());
        })
        .value_content(move |v: util::SelectionValue<'_>| {
            record
                .borrow_mut()
                .push(format!("{}|{}", v.is_placeholder, v.selected_text));
            v.default_children
        })
        .into_any_element()
    });

    assert_eq!(last_string(&seen), "false|Typst");
    click(cx, 60., 18.);
    press(cx, "ctrl-a");
    press(cx, "backspace");
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "false|Typst",
        "a controlled selection must stay owner-driven after the clear request"
    );
    assert_eq!(changed.borrow().as_slice(), ["0"]);

    press(cx, "backspace");
    flush_frame(cx);
    assert_eq!(
        changed.borrow().as_slice(),
        ["0"],
        "a no-op Backspace on an already empty input must not repeat the clear request"
    );
}

// ---------------------------------------------------------------------------
// ProgressBar / Meter / ProgressCircle ValueLabel
// ---------------------------------------------------------------------------

/// `ProgressBar.ValueLabel` is handed `percentage` (0-100) and the formatted
/// `valueText`. Drive the *value* by changing the `Rc` the render closure
/// reads; the percentage and the text must both move. Compared as formatted
/// strings (`"{:.0}"` of the percentage, and the component's own `"25%"`
/// text), never as floats.
#[gpui::test]
fn progress_bar_value_content_sees_percentage_and_text(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let value = Rc::new(RefCell::new(25.0f32));
    let for_view = value.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        let now = *for_view.borrow();
        ProgressBar::new()
            .value(now)
            .label("Loading")
            .show_value_label(true)
            .value_content(move |percentage, text| {
                record.borrow_mut().push(format!("{percentage:.0}|{text}"));
                gpui::div().child(text.to_owned()).into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "25|25%",
        "value 25 must hand 25 as the percentage and '25%' as the text"
    );

    *value.borrow_mut() = 75.0;
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "75|75%",
        "writing 75 to the source value must move both handed values"
    );
}

/// `Meter.ValueLabel` forwards to the bar's closure, so the same contract
/// holds through `Meter`'s own builder instead of `ProgressBar`'s.
#[gpui::test]
fn meter_value_content_forwards_percentage_and_text(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let value = Rc::new(RefCell::new(25.0f32));
    let for_view = value.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        let now = *for_view.borrow();
        Meter::new(now)
            .show_value(true)
            .value_content(move |percentage, text| {
                record.borrow_mut().push(format!("{percentage:.0}|{text}"));
                gpui::div().child(text.to_owned()).into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "25|25%",
        "a 25 meter must hand its own percentage and text"
    );

    *value.borrow_mut() = 40.0;
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "40|40%",
        "writing 40 must move the handed values through the Meter builder"
    );
}

/// `ProgressCircle.ValueLabel` is the same pair on the ring.
#[gpui::test]
fn progress_circle_value_content_sees_percentage_and_text(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let value = Rc::new(RefCell::new(25.0f32));
    let for_view = value.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        let now = *for_view.borrow();
        ProgressCircle::new()
            .value(now)
            .show_value_label(true)
            .value_content(move |percentage, text| {
                record.borrow_mut().push(format!("{percentage:.0}|{text}"));
                gpui::div().child(text.to_owned()).into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "25|25%",
        "a quarter ring must hand 25 and '25%'"
    );

    *value.borrow_mut() = 50.0;
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "50|50%",
        "writing 50 must move the handed values on the ring as well"
    );
}

// ---------------------------------------------------------------------------
// ColorSlider.Output
// ---------------------------------------------------------------------------

/// `ColorSlider.Output` is handed the current colour and the channel's
/// formatted value. The track is a tab stop, so Tab focuses it and Right
/// steps the hue by 1 (a 0..360 channel, `step = 1`); the closure must see
/// the *new* colour and the *new* display text on the next frame. Colours
/// compare as `to_hex()` strings computed with the port's own colour math —
/// `PickerColor::hsb(180., 1., 1.)` and its 181° successor — and the display
/// as the `°` string the renderer itself writes, so no float equality
/// anywhere.
#[gpui::test]
fn color_slider_output_hands_colour_and_text(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        ColorSlider::new(
            "cs-vc-output",
            PickerColor::hsb(180., 1., 1.),
            ColorChannel::Hue,
        )
        .default_value(PickerColor::hsb(180., 1., 1.))
        .output(move |colour, display| {
            record
                .borrow_mut()
                .push(format!("{}|{}", colour.to_hex(), display));
            gpui::div().child(display.to_owned()).into_any_element()
        })
        .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        format!("{}|180\u{00B0}", PickerColor::hsb(180., 1., 1.).to_hex()),
        "the first render must hand the seeded colour and its 180 degree read-out"
    );

    press(cx, "tab");
    press(cx, "right");
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        format!("{}|181\u{00B0}", PickerColor::hsb(181., 1., 1.).to_hex()),
        "one Right on a focused hue slider must hand the stepped colour and 181 degrees"
    );
}

#[gpui::test]
fn slider_output_hands_formatted_range_and_live_values(cx: &mut TestAppContext) {
    let seen: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        gpui::div()
            .w(px(600.))
            .child(
                Slider::new("slider-vc-output", 0.)
                    .default_values([20., 80.])
                    .format_options(NumberFormat::currency("USD"))
                    .output(move |values, labels| {
                        record.borrow_mut().push(format!(
                            "{},{}|{}",
                            values[0],
                            values[1],
                            labels.join(" \u{2013} ")
                        ));
                        gpui::div()
                            .child(labels.join(" \u{2013} "))
                            .into_any_element()
                    }),
            )
            .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "20,80|$20.00 \u{2013} $80.00",
        "the output must receive every initial value and its formatted thumb label"
    );

    press(cx, "tab");
    press(cx, "right");
    flush_frame(cx);
    assert_eq!(
        last_string(&seen),
        "21,80|$21.00 \u{2013} $80.00",
        "stepping the focused thumb must update the output values and labels"
    );
}

/// Without `formatOptions`, pinned RAC still creates an Intl decimal formatter,
/// so each `getThumbValueLabel` groups thousands before Slider.Output joins the
/// labels. The explicit maximum keeps both seeded thumbs inside the range.
#[gpui::test]
fn slider_output_default_labels_group_thousands(cx: &mut TestAppContext) {
    let seen = events();
    let record = seen.clone();
    open_host(cx, move || {
        let record = record.clone();
        Slider::new("slider-vc-default-grouping", 0.)
            .max_value(10_000.)
            .default_values([1200., 5000.])
            .output(move |_, labels| {
                record.borrow_mut().push(labels.join(" \u{2013} "));
                gpui::div()
                    .child(labels.join(" \u{2013} "))
                    .into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(
        last_string(&seen),
        "1,200 \u{2013} 5,000",
        "format-less output must use the grouped decimal labels v3 gets from Intl"
    );
}

// ---------------------------------------------------------------------------
// Switch.Content / CloseButton.Content
// ---------------------------------------------------------------------------

/// `Switch.Content` hands the interactive state over in place of the label.
/// The hover and the press are reported to a *handler* and stashed in the
/// keyed interaction slot, so every event is followed by a forced frame and
/// the closure's latest snapshot is asserted — the same cycle the Button test
/// proves. The track is 40x20 (`Size::Md`), so the pointer cycle runs at
/// (20, 10). The mouse-up also completes a click, which toggles the
/// default-checked switch off.
#[gpui::test]
fn switch_content_render_prop_sees_pointer_cycle(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Switch::new("sw-vc-pointer")
            .default_selected(true)
            .content(move |state: herogpui_components::SwitchState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pressed, state.is_selected);
                gpui::div()
                    .w(px(48.))
                    .child("label".to_owned())
                    .into_any_element()
            })
            .into_any_element()
    });

    let centre = point(px(20.), px(10.));

    // The first render drew before any pointer event: checked and idle.
    assert_eq!(
        *seen.borrow(),
        (false, false, true),
        "initial state must be idle and checked"
    );

    // Move onto the track: the interaction slot hears the hover, and the
    // forced frame hands it to the closure.
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, true),
        "the frame after the move must see the hover"
    );

    // Press down: the press lands alongside the hover.
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, true),
        "the frame after the down must see the press"
    );

    // Release: the press lifts and the click the up completes toggles the
    // switch's own state off — one frame, both observed.
    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false, false),
        "the up must lift the press and toggle the selection off"
    );

    // Leave the track: the hover clears too.
    cx.simulate_mouse_move(
        point(px(400.), px(400.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false, false),
        "the frame after leaving must see the hover lifted"
    );

    // A press cancelled by leaving the hitbox must not remain latched after
    // the mouse-up is delivered elsewhere.
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    cx.simulate_mouse_move(
        point(px(400.), px(400.)),
        Some(MouseButton::Left),
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, true, false),
        "leaving during a press must clear hover but keep the press until release"
    );
    cx.simulate_mouse_up(
        point(px(400.), px(400.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false, false),
        "releasing outside must clear the press"
    );
}

#[gpui::test]
fn disabled_switch_content_never_reports_hover_or_press(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, true)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Switch::new("sw-vc-disabled")
            .is_disabled(true)
            .content(move |state: herogpui_components::SwitchState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pressed, state.is_disabled);
                gpui::div().w(px(48.)).child("disabled").into_any_element()
            })
            .into_any_element()
    });

    let centre = point(px(20.), px(10.));
    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);

    assert_eq!(
        *seen.borrow(),
        (false, false, true),
        "disabled Switch.Content must not expose hover or press state"
    );
}

/// The focus half of the switch's render props: the track is the page's only
/// tab stop, so Tab focuses it; the keyboard flag (the app root sets it on a
/// key event) turns `isFocusVisible` on; and Space — which gpui activates for
/// a focused element with click listeners — toggles the switch through the
/// same closure.
#[gpui::test]
fn switch_content_sees_focus_and_keyboard_toggle(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        Switch::new("sw-vc-keys")
            .default_selected(true)
            .content(move |state: herogpui_components::SwitchState| {
                *record.borrow_mut() =
                    (state.is_focused, state.is_focus_visible, state.is_selected);
                gpui::div()
                    .w(px(48.))
                    .child("label".to_owned())
                    .into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(*seen.borrow(), (false, false, true), "initially unfocused");

    press(cx, "tab");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, true),
        "Tab must focus the track and report the app root's keyboard-visible flag"
    );

    press(cx, "space");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true, false),
        "Space must toggle the focused switch off through the same closure"
    );
}

/// `CloseButton.Content` hands the interactive state over in place of the
/// glyph, and the hover/press pair comes from the same one-frame-late slot.
/// The button is a 24px square at the origin, so the pointer cycle runs at
/// (12, 12); the completed click also proves the `on_press` callback still
/// fires once when a `content` closure is set (the two helpers bind different
/// slots — a style `hover` and the interaction slot's `on_hover` listener —
/// so this must not panic like Button's first attempt did).
#[gpui::test]
fn close_button_content_render_prop_sees_pointer_cycle(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false)));
    let record = seen.clone();
    let presses = events();
    let pressed = presses.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        let pressed = pressed.clone();
        CloseButton::new("cb-vc-pointer")
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_hovered, state.is_pressed);
                gpui::div()
                    .w(px(16.))
                    .child("x".to_owned())
                    .into_any_element()
            })
            .on_press(move |_, _, _| pressed.borrow_mut().push("press".into()))
            .into_any_element()
    });

    let centre = point(px(12.), px(12.));

    assert_eq!(*seen.borrow(), (false, false), "initially idle");

    cx.simulate_mouse_move(centre, None::<MouseButton>, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false),
        "hover must arrive a frame late"
    );

    cx.simulate_mouse_down(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true),
        "the press must arrive on the frame after the down"
    );

    cx.simulate_mouse_up(centre, MouseButton::Left, Modifiers::none());
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, false),
        "the release must lift the press"
    );
    assert_eq!(
        presses.borrow().as_slice(),
        ["press"],
        "the completed click must still report exactly one press"
    );

    cx.simulate_mouse_move(
        point(px(400.), px(400.)),
        None::<MouseButton>,
        Modifiers::none(),
    );
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (false, false),
        "the frame after leaving must see the hover lifted"
    );
}

/// The focus half for the close button: Tab reaches its tracked handle and
/// the keyboard flag reaches the closure's `isFocusVisible`.
#[gpui::test]
fn close_button_content_sees_focus_and_keyboard_flag(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new((false, false)));
    let record = seen.clone();
    let cx = open_host(cx, move || {
        let record = record.clone();
        CloseButton::new("cb-vc-keys")
            .content(move |state: util::InteractiveState| {
                *record.borrow_mut() = (state.is_focused, state.is_focus_visible);
                gpui::div()
                    .w(px(16.))
                    .child("x".to_owned())
                    .into_any_element()
            })
            .into_any_element()
    });

    assert_eq!(*seen.borrow(), (false, false), "initially unfocused");

    press(cx, "tab");
    flush_frame(cx);
    assert_eq!(
        *seen.borrow(),
        (true, true),
        "Tab must focus the button and report the app root's keyboard-visible flag"
    );
}

// ---------------------------------------------------------------------------
// Field children-as-a-function (FieldFocus)
// ---------------------------------------------------------------------------

/// Drives the standard focus cycle for a field whose `content` closure
/// records the `FieldFocus` it is handed. The closure draws, in its place,
/// the same field bound to the same state — the caller's parts around the
/// still-focusable input, which is exactly the composition both halves of the
/// render-prop inversion describe. The shared state handle is what a click at
/// (60, 18) focuses, and `util::set_focus_visible` is the app root's own
/// keyboard half, so the ring state changes with the flag while the focus
/// stays.
/// The closure log: one `(is_focused, is_focus_within, is_focus_visible)`
/// triple per rendered frame.
type FocusLog = Rc<RefCell<Vec<(bool, bool, bool)>>>;

/// How the test drives the focus into the replacement field: a click at the
/// field box (what a pointer does) or Tab (what a keyboard does).
#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusDrive {
    Click,
    Tab,
}

fn drive_field_focus<B>(cx: &mut TestAppContext, drive: FocusDrive, build: B)
where
    B: Fn(Rc<RefCell<Vec<(bool, bool, bool)>>>) -> gpui::AnyElement + 'static,
{
    let seen: Rc<RefCell<Vec<(bool, bool, bool)>>> = Rc::new(RefCell::new(Vec::new()));
    let for_view = seen.clone();
    let cx = open_host(cx, move || build(for_view.clone()));

    // The first render already handed the closure its initial state; a field
    // that panicked drawing its replacement would have died at open_host.
    assert_eq!(
        *seen.borrow().last().unwrap(),
        (false, false, false),
        "a fresh field must report empty, and the closure must have run"
    );

    // The shared state handle takes focus. A click keeps focus-visible off;
    // Tab records keyboard input at the app root and turns it on.
    match drive {
        FocusDrive::Click => {
            click(cx, 60., 18.);
            flush_frame(cx);
        }
        FocusDrive::Tab => {
            press(cx, "tab");
            flush_frame(cx);
        }
    }
    assert_eq!(
        *seen.borrow().last().unwrap(),
        (true, true, drive == FocusDrive::Tab),
        "the drive must focus the field and report whether it came from the keyboard"
    );

    if drive == FocusDrive::Click {
        cx.update(|_, cx| util::set_focus_visible(true, cx));
        flush_frame(cx);
        assert_eq!(
            *seen.borrow().last().unwrap(),
            (true, true, true),
            "a focused field whose last input was a key must report focus-visible"
        );
    }

    // The flag is a request: dropping it drops the ring, not the focus.
    cx.update(|_, cx| util::set_focus_visible(false, cx));
    flush_frame(cx);
    assert_eq!(
        *seen.borrow().last().unwrap(),
        (true, true, false),
        "focus-visible must clear with the flag while the focus stays"
    );
}

#[gpui::test]
fn input_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        Input::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    Input::new(value.clone())
                        .placeholder("parts")
                        .into_any_element()
                }
            })
            .into_any_element()
    });
}

#[gpui::test]
fn text_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        TextField::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    TextField::new(value.clone())
                        .placeholder("parts")
                        .into_any_element()
                }
            })
            .into_any_element()
    });
}

#[gpui::test]
fn search_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        SearchField::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    SearchField::new(value.clone())
                        .placeholder("parts")
                        .into_any_element()
                }
            })
            .into_any_element()
    });
}

#[gpui::test]
fn number_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| NumberState::new(cx, 0.0));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        NumberField::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    NumberField::new(value.clone()).into_any_element()
                }
            })
            .into_any_element()
    });
}

// Every field's `content` closure is handed a live `FieldFocus`. TimeField uses
// the same state-owned handle for the outer closure and the replacement field
// it draws, so this identical drive observes the segment group's real focus.
#[gpui::test]
fn time_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| TimeState::new(cx));
    let for_view = state;
    // TimeField's segments only reachable state via a keyed handle that has no
    // dispatch node until the control has rendered; a pointer click on the
    // very first frame is reclaimed by the host root before the closure reads
    // it, so this field is driven by Tab, exactly as fields.rs does.
    drive_field_focus(cx, FocusDrive::Tab, move |record: FocusLog| {
        TimeField::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    TimeField::new(value.clone()).into_any_element()
                }
            })
            .into_any_element()
    });
}

#[gpui::test]
fn date_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        DateField::new(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    DateField::new(value.clone()).into_any_element()
                }
            })
            .into_any_element()
    });
}

#[gpui::test]
fn color_field_content_hands_focus_state(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let for_view = state;
    drive_field_focus(cx, FocusDrive::Click, move |record: FocusLog| {
        ColorField::new("cf-vc-outer", PickerColor::from_hex("#336699").unwrap())
            .state(for_view.clone())
            .content({
                let value = for_view.clone();
                move |focus: util::FieldFocus| {
                    record.borrow_mut().push((
                        focus.is_focused,
                        focus.is_focus_within,
                        focus.is_focus_visible,
                    ));
                    ColorField::new("cf-vc-inner", PickerColor::from_hex("#336699").unwrap())
                        .state(value.clone())
                        .into_any_element()
                }
            })
            .into_any_element()
    });
}
