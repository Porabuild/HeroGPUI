//! Keyboard and pointer contracts inherited by HeroUI v3's field family.
//!
//! The prop tables only say that these fields are read-only or clearable. The
//! behaviour comes from HeroUI v3.2.4's exact dependencies: `react-aria`
//! 3.51.0 and `react-stately` 3.49.0. In those versions, a read-only date
//! segment keeps `tabIndex = 0`, `useSpinButton` disables editing but not
//! segment-to-segment focus, SearchField's Escape shortcut clears a non-empty
//! value and calls `onClear`, and `onClear` otherwise belongs only to the clear
//! affordance. NumberField only wires Home/End when a corresponding bound is
//! present.
//!
//! Geometry is derived from the implementation. NumberField is 220px wide
//! with 40px stepper cells, so the decrement/increment centres are (20, 18)
//! and (200, 18). DateField's 14px stepper column follows the padded
//! `MM/DD/YYYY` segments: with 14px Consolas text its two 14px cells are safely
//! hit at (132, 11) and (132, 25).

mod harness;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    prelude::*, Context, ElementId, Entity, Focusable, Render, TestAppContext, VisualTestContext,
    Window,
};
use herogpui_components::{
    DateField, HourCycle, InputState, NumberField, NumberState, SearchField, Time, TimeField,
    TimeGranularity, TimeSegment, TimeState,
};
use herogpui_theme::ThemeProvider;

use harness::{click, events, open_host, press, Events};

fn refresh(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

struct ControlledTimeHost {
    state: Entity<TimeState>,
    controlled: Rc<Cell<Option<Time>>>,
    changes: Events,
    rendered: Rc<RefCell<Vec<(TimeSegment, String)>>>,
}

impl Render for ControlledTimeHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let root = window
            .use_keyed_state(
                ElementId::Name("controlled-time-root".into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        if !root.contains_focused(window, cx) {
            window.focus(&root);
        }

        let changes = self.changes.clone();
        let rendered = self.rendered.clone();
        gpui::div()
            .track_focus(&root)
            .on_key_down(|event, window, _| {
                if event.keystroke.key == "tab" {
                    window.focus_next();
                }
            })
            .size_full()
            .child(
                TimeField::new(self.state.clone())
                    .value(self.controlled.get(), cx)
                    .segment(move |segment, text| {
                        rendered.borrow_mut().push((segment, text.to_string()));
                        gpui::div().child(text).into_any_element()
                    })
                    .on_change(move |time, _, _| {
                        changes.borrow_mut().push(time.map_or_else(
                            || "none".to_owned(),
                            |time| format!("{:02}:{:02}", time.hour, time.minute),
                        ));
                    }),
            )
    }
}

fn open_controlled_time(
    cx: &mut TestAppContext,
    state: Entity<TimeState>,
    controlled: Rc<Cell<Option<Time>>>,
    changes: Events,
    rendered: Rc<RefCell<Vec<(TimeSegment, String)>>>,
) -> &mut VisualTestContext {
    cx.update(ThemeProvider::init);
    let (_view, cx) = cx.add_window_view(|_, _| ControlledTimeHost {
        state,
        controlled,
        changes,
        rendered,
    });
    cx
}

#[gpui::test]
fn search_field_escape_clears_nonempty_value_and_reports_clear(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
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

    press(cx, "tab");
    press(cx, "escape");

    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(
        (value, changed.borrow().clone(), cleared.borrow().clone()),
        (String::new(), vec![String::new()], vec!["clear".to_owned()]),
        "react-aria 3.51.0's SearchField Escape shortcut must clear a non-empty value, report the empty change, and invoke onClear once"
    );
}

#[gpui::test]
fn search_field_deleting_final_character_does_not_report_clear(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let clears = events();
    let cleared = clears.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "x"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let clears = clears.clone();
        SearchField::new(state_for_view.clone())
            .on_change(move |text, _, _| changes.borrow_mut().push(text.to_owned()))
            .on_clear(move |_, _| clears.borrow_mut().push("clear".to_owned()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "backspace");

    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "", "Backspace must still delete the last character");
    assert_eq!(changed.borrow().as_slice(), [""]);
    assert!(
        cleared.borrow().is_empty(),
        "react-aria 3.51.0 calls onClear from Escape or the clear button, not from an ordinary edit whose result happens to be empty"
    );
}

#[gpui::test]
fn date_field_read_only_stays_focusable_and_navigable_without_editing(cx: &mut TestAppContext) {
    let read_only = Rc::new(Cell::new(true));
    let read_only_for_view = read_only.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DateField::new(state_for_view.clone())
            .is_read_only(read_only_for_view.get())
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    press(cx, "tab");
    let focused = cx.update(|window, cx| state.read(cx).focus_handle(cx).is_focused(window));
    assert!(
        focused,
        "react-aria 3.51.0 leaves read-only date segments in the tab order"
    );

    // Right moves Month -> Day. The editing keys must do nothing while the
    // field is read-only.
    press(cx, "right");
    press(cx, "up");
    press(cx, "9");
    press(cx, "delete");
    assert!(changed.borrow().is_empty());
    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "2025-01-15");

    // Make the same surface editable without changing its focus or cursor.
    // Up now proves that the read-only Right moved the active segment to Day.
    read_only.set(false);
    refresh(cx);
    press(cx, "up");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2025-01-16"],
        "read-only Left/Right must navigate between segments even though Up, digits and Delete cannot edit"
    );
}

#[gpui::test]
fn time_field_read_only_stays_focusable_and_navigable_without_editing(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TimeField::new(state_for_view.clone())
            .is_read_only(true)
            .on_change(move |time, _, _| {
                changes.borrow_mut().push(time.map_or_else(
                    || "none".to_owned(),
                    |time| format!("{:02}:{:02}", time.hour, time.minute),
                ));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "up");
    press(cx, "9");
    press(cx, "delete");

    let snapshot = cx.update(|_, cx| {
        let state = state.read(cx);
        (state.focused, state.value)
    });
    assert_eq!(snapshot, (TimeSegment::Minute, Some(Time::new(9, 30))));
    assert!(
        changed.borrow().is_empty(),
        "read-only navigation may move the active segment but must not emit a value change"
    );
}

#[gpui::test]
fn number_field_read_only_stepper_clicks_are_inert(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| NumberState::new(cx, 5.0));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        NumberField::new(state_for_view.clone())
            .is_read_only(true)
            .on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()))
            .into_any_element()
    });

    click(cx, 20., 18.);
    click(cx, 200., 18.);

    let value = cx.update(|_, cx| state.read(cx).value().to_string());
    assert_eq!(value, "5");
    assert!(
        changed.borrow().is_empty(),
        "react-stately 3.49.0 makes canIncrement/canDecrement false when a NumberField is read-only"
    );
}

#[gpui::test]
fn number_field_unbounded_home_and_end_are_noops(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| NumberState::new(cx, 42.0));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        NumberField::new(state_for_view.clone())
            .on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    press(cx, "end");

    let value = cx.update(|_, cx| state.read(cx).value().to_string());
    assert_eq!(value, "42");
    assert!(
        changed.borrow().is_empty(),
        "react-stately 3.49.0's decrementToMin/incrementToMax do nothing when the matching bound is absent"
    );
}

#[gpui::test]
fn number_field_explicit_extreme_bounds_still_answer_home_and_end(cx: &mut TestAppContext) {
    let changes: Rc<RefCell<Vec<u64>>> = Rc::new(RefCell::new(Vec::new()));
    let changed = changes.clone();
    let state = cx.new(|cx| NumberState::new(cx, 42.0));
    state.update(cx, |state, _| state.set_range(f64::MIN, f64::MAX));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        NumberField::new(state_for_view.clone())
            .on_change(move |value, _, _| changes.borrow_mut().push(value.to_bits()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    let minimum = cx.update(|_, cx| state.read(cx).value().to_bits());
    press(cx, "end");
    let maximum = cx.update(|_, cx| state.read(cx).value().to_bits());

    assert_eq!(
        (minimum, maximum),
        (f64::MIN.to_bits(), f64::MAX.to_bits()),
        "explicit extreme bounds are values, not the absence of bounds"
    );
    assert_eq!(
        changed.borrow().as_slice(),
        [f64::MIN.to_bits(), f64::MAX.to_bits()]
    );
}

#[gpui::test]
fn date_field_disabled_steppers_are_inert(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DateField::new(state_for_view.clone())
            .is_disabled(true)
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    click(cx, 132., 11.);
    click(cx, 132., 25.);

    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "2025-01-15");
    assert!(
        changed.borrow().is_empty(),
        "a disabled DateField's pointer steppers must not change or report its value"
    );
}

#[gpui::test]
fn date_field_read_only_steppers_are_inert(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DateField::new(state_for_view.clone())
            .is_read_only(true)
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    click(cx, 132., 11.);
    click(cx, 132., 25.);

    let value = cx.update(|_, cx| state.read(cx).value().to_owned());
    assert_eq!(value, "2025-01-15");
    assert!(
        changed.borrow().is_empty(),
        "a read-only DateField's pointer steppers must not change or report its value"
    );
}

#[gpui::test]
fn time_field_delete_clears_only_the_active_segment_and_defers_change(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let validations = Rc::new(RefCell::new(Vec::new()));
    let validated = validations.clone();
    let rendered: Rc<RefCell<Vec<(TimeSegment, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let rendered_for_view = rendered.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let validations = validations.clone();
        let rendered = rendered_for_view.clone();
        TimeField::new(state_for_view.clone())
            .segment(move |segment, text| {
                rendered.borrow_mut().push((segment, text.to_string()));
                gpui::div().child(text).into_any_element()
            })
            .validate(move |value| {
                validations.borrow_mut().push(value.is_none());
                None
            })
            .on_change(move |time, _, _| {
                changes.borrow_mut().push(time.map_or_else(
                    || "none".to_owned(),
                    |time| format!("{:02}:{:02}", time.hour, time.minute),
                ));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "delete");
    refresh(cx);

    let latest = |segment| {
        rendered
            .borrow()
            .iter()
            .rev()
            .find_map(|(part, text)| (*part == segment).then(|| text.clone()))
            .unwrap()
    };
    assert_eq!(
        (latest(TimeSegment::Hour), latest(TimeSegment::Minute)),
        ("09".to_owned(), "--".to_owned())
    );
    let value = cx.update(|_, cx| state.read(cx).value);
    assert_eq!(
        value,
        Some(Time::new(9, 30)),
        "a partial display edit must retain the last committed form value"
    );
    assert_eq!(
        validated.borrow().last(),
        Some(&false),
        "validation must keep receiving the committed time while only one segment is empty"
    );
    assert!(
        changed.borrow().is_empty(),
        "react-stately 3.49.0 keeps an incomplete display override and defers onChange until the value is complete or blurred"
    );

    press(cx, "left");
    press(cx, "delete");
    refresh(cx);
    assert_eq!(cx.update(|_, cx| state.read(cx).value), None);
    assert_eq!(validated.borrow().last(), Some(&true));
    assert_eq!(
        changed.borrow().as_slice(),
        ["none"],
        "clearing every visible segment commits null and reports it once"
    );
}

#[gpui::test]
fn time_field_value_builder_writes_through_before_render(cx: &mut TestAppContext) {
    let state = cx.new(|cx| TimeState::new(cx));
    cx.update(|cx| {
        let _field = TimeField::new(state.clone()).value(Some(Time::new(7, 45)), cx);
        assert_eq!(state.read(cx).value, Some(Time::new(7, 45)));
    });
}

#[gpui::test]
fn time_field_direct_public_value_mutation_updates_complete_display(cx: &mut TestAppContext) {
    let rendered: Rc<RefCell<Vec<(TimeSegment, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let rendered_for_view = rendered.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let rendered = rendered_for_view.clone();
        TimeField::new(state_for_view.clone())
            .segment(move |segment, text| {
                rendered.borrow_mut().push((segment, text.to_string()));
                gpui::div().child(text).into_any_element()
            })
            .into_any_element()
    });

    cx.update(|_, cx| {
        state.update(cx, |state, cx| {
            state.value = Some(Time::new(7, 45));
            cx.notify();
        });
    });
    refresh(cx);

    let latest = |segment| {
        rendered
            .borrow()
            .iter()
            .rev()
            .find_map(|(part, text)| (*part == segment).then(|| text.clone()))
            .unwrap()
    };
    assert_eq!(
        (latest(TimeSegment::Hour), latest(TimeSegment::Minute)),
        ("07".to_owned(), "45".to_owned()),
        "direct updates to the public committed value must remain visible when no edit is incomplete"
    );
}

#[gpui::test]
fn time_field_deleting_every_visible_segment_reports_null(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let validations = Rc::new(RefCell::new(Vec::new()));
    let validated = validations.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let validations = validations.clone();
        TimeField::new(state_for_view.clone())
            .validate(move |value| {
                validations.borrow_mut().push(value.is_none());
                None
            })
            .on_change(move |time, _, _| {
                changes.borrow_mut().push(time.map_or_else(
                    || "none".to_owned(),
                    |time| format!("{:02}:{:02}", time.hour, time.minute),
                ));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "delete");
    assert!(changed.borrow().is_empty(), "one missing segment is local");
    press(cx, "right");
    press(cx, "delete");
    refresh(cx);

    let value = cx.update(|_, cx| state.read(cx).value);
    assert_eq!(value, None, "all visible segments empty the field value");
    assert_eq!(changed.borrow().as_slice(), ["none"]);
    assert_eq!(
        validated.borrow().last(),
        Some(&true),
        "validation must receive None after the whole visible value is cleared"
    );
}

#[gpui::test]
fn time_field_reentry_commits_only_after_every_visible_segment_is_complete(
    cx: &mut TestAppContext,
) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TimeField::new(state_for_view.clone())
            .on_change(move |time, _, _| {
                changes.borrow_mut().push(time.map_or_else(
                    || "none".to_owned(),
                    |time| format!("{:02}:{:02}", time.hour, time.minute),
                ));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "delete");
    press(cx, "right");
    press(cx, "delete");
    assert_eq!(changed.borrow().as_slice(), ["none"]);

    press(cx, "left");
    press(cx, "up");
    let partial = cx.update(|_, cx| state.read(cx).value);
    assert_eq!(partial, None, "one entered segment is still incomplete");
    assert_eq!(changed.borrow().as_slice(), ["none"]);

    press(cx, "right");
    press(cx, "up");
    assert_eq!(changed.borrow().as_slice(), ["none", "10:01"]);
    let complete = cx.update(|_, cx| state.read(cx).value);
    assert_eq!(complete, Some(Time::new(10, 1)));
}

#[gpui::test]
fn time_field_controlled_some_restores_after_clear_all(cx: &mut TestAppContext) {
    let controlled = Rc::new(Cell::new(Some(Time::new(9, 30))));
    let changes = events();
    let changed = changes.clone();
    let rendered = Rc::new(RefCell::new(Vec::new()));
    let state = cx.new(|cx| TimeState::new(cx));
    let cx = open_controlled_time(cx, state.clone(), controlled, changes, rendered.clone());

    press(cx, "tab");
    press(cx, "delete");
    refresh(cx);
    let cleared_hour = rendered
        .borrow()
        .iter()
        .rev()
        .find_map(|(part, text)| (*part == TimeSegment::Hour).then(|| text.clone()))
        .unwrap();
    assert_eq!(cleared_hour, "--");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value),
        Some(Time::new(9, 30))
    );
    assert!(changed.borrow().is_empty());

    press(cx, "right");
    press(cx, "delete");
    refresh(cx);

    assert_eq!(changed.borrow().as_slice(), ["none"]);
    let value = cx.update(|_, cx| state.read(cx).value);
    assert_eq!(value, Some(Time::new(9, 30)));
    let latest = |segment| {
        rendered
            .borrow()
            .iter()
            .rev()
            .find_map(|(part, text)| (*part == segment).then(|| text.clone()))
            .unwrap()
    };
    assert_eq!(
        (latest(TimeSegment::Hour), latest(TimeSegment::Minute)),
        ("09".to_owned(), "30".to_owned()),
        "a controlled Some prop restores its display when the caller does not accept null"
    );
}

#[gpui::test]
fn time_field_controlled_none_preserves_partial_display_until_completion(cx: &mut TestAppContext) {
    let controlled = Rc::new(Cell::new(None));
    let changes = events();
    let changed = changes.clone();
    let rendered = Rc::new(RefCell::new(Vec::new()));
    let state = cx.new(|cx| TimeState::new(cx));
    let cx = open_controlled_time(
        cx,
        state.clone(),
        controlled.clone(),
        changes,
        rendered.clone(),
    );

    press(cx, "tab");
    press(cx, "up");
    refresh(cx);
    refresh(cx);
    let latest = |segment| {
        rendered
            .borrow()
            .iter()
            .rev()
            .find_map(|(part, text)| (*part == segment).then(|| text.clone()))
            .unwrap()
    };
    assert_eq!(
        (latest(TimeSegment::Hour), latest(TimeSegment::Minute)),
        ("10".to_owned(), "--".to_owned()),
        "repeated controlled None renders must preserve the local incomplete display"
    );
    assert_eq!(cx.update(|_, cx| state.read(cx).value), None);
    assert!(changed.borrow().is_empty());

    press(cx, "right");
    press(cx, "up");
    assert_eq!(changed.borrow().as_slice(), ["10:01"]);
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value),
        None,
        "without a controlled prop update, completion reports the candidate but keeps the controlled value null"
    );

    controlled.set(Some(Time::new(10, 1)));
    refresh(cx);
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value),
        Some(Time::new(10, 1)),
        "accepting the callback through the controlled prop must restore the committed value"
    );
}

#[gpui::test]
fn time_field_hidden_cleared_segments_do_not_suppress_visible_commits(cx: &mut TestAppContext) {
    let granularity = Rc::new(Cell::new(TimeGranularity::Second));
    let granularity_for_view = granularity.clone();
    let cycle = Rc::new(Cell::new(HourCycle::H24));
    let cycle_for_view = cycle.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| TimeState::with_value(cx, Time::new(9, 30).with_second(45)));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        TimeField::new(state_for_view.clone())
            .granularity(granularity_for_view.get())
            .hour_cycle(cycle_for_view.get())
            .on_change(move |time, _, _| {
                changes.borrow_mut().push(time.map_or_else(
                    || "none".to_owned(),
                    |time| format!("{:02}:{:02}", time.hour, time.minute),
                ));
            })
            .into_any_element()
    });

    // Clear Second, then hide it. Moving back to Hour and stepping must emit
    // the complete Hour+Minute value even though the hidden Second is empty.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "delete");
    granularity.set(TimeGranularity::Minute);
    cycle.set(HourCycle::H12);
    refresh(cx);
    press(cx, "left");
    press(cx, "up");
    assert_eq!(changed.borrow().as_slice(), ["10:30"]);

    // Clear the now-visible meridiem, switch back to H24, and prove that this
    // second hidden segment also cannot suppress the visible commit.
    press(cx, "right");
    press(cx, "right");
    press(cx, "delete");
    cycle.set(HourCycle::H24);
    refresh(cx);
    press(cx, "left");
    press(cx, "up");
    assert_eq!(changed.borrow().as_slice(), ["10:30", "11:30"]);
}
