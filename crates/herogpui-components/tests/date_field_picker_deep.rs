//! Deep inherited contracts for the v3 date field and picker family.
//!
//! HeroUI 3.2.4 composes React Aria's date controls. Its pinned
//! `react-stately` 3.49.0 keeps an incomplete segmented display separate from
//! the committed value, and its pinned `react-aria` 3.51.0 opens picker groups
//! with Alt+ArrowDown or Alt+ArrowUp.

mod harness;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{prelude::*, TestAppContext, VisualTestContext};
use herogpui_components::{
    CalendarState, Date, DateField, DatePicker, DateRangePicker, DateRangeState, DateSegment, Form,
    InputState, ValidationBehavior,
};

use harness::{click, events, open_host, press};

fn refresh(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

#[gpui::test]
fn date_field_delete_keeps_an_incomplete_display_without_committing(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let rendered: Rc<RefCell<Vec<(DateSegment, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let rendered_for_view = rendered.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        let rendered = rendered_for_view.clone();
        DateField::new(state_for_view.clone())
            .segment(move |segment, text| {
                rendered.borrow_mut().push((segment, text.to_string()));
                gpui::div().child(text).into_any_element()
            })
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "delete");
    refresh(cx);

    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "2025-01-15"
    );
    assert!(
        changed.borrow().is_empty(),
        "clearing one date segment is an incomplete local edit, not a committed value"
    );
    let latest = |segment| {
        rendered
            .borrow()
            .iter()
            .rev()
            .find_map(|(part, text)| (*part == segment).then(|| text.clone()))
            .unwrap()
    };
    assert_eq!(latest(DateSegment::Month), "mm");
    assert_eq!(latest(DateSegment::Day), "15");
    assert_eq!(latest(DateSegment::Year), "2025");

    press(cx, "right");
    press(cx, "delete");
    assert!(changed.borrow().is_empty());
    press(cx, "right");
    press(cx, "delete");

    assert_eq!(cx.update(|_, cx| state.read(cx).value().to_owned()), "");
    assert_eq!(
        changed.borrow().as_slice(),
        ["none"],
        "only clearing every visible segment commits null"
    );
}

#[gpui::test]
fn date_field_reentry_waits_until_every_segment_is_complete(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-01-15"));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DateField::new(state_for_view.clone())
            .placeholder_value(Date::new(2025, 1, 15))
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    press(cx, "tab");
    for index in 0..3 {
        press(cx, "delete");
        if index < 2 {
            press(cx, "right");
        }
    }
    assert_eq!(changed.borrow().as_slice(), ["none"]);

    press(cx, "left");
    press(cx, "left");
    press(cx, "0");
    press(cx, "1");
    press(cx, "1");
    press(cx, "5");
    assert_eq!(
        changed.borrow().as_slice(),
        ["none"],
        "two entered segments still leave the date incomplete"
    );
    assert_eq!(cx.update(|_, cx| state.read(cx).value().to_owned()), "");

    press(cx, "up");
    assert_eq!(changed.borrow().as_slice(), ["none", "2026-01-15"]);
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "2026-01-15"
    );
}

#[gpui::test]
fn date_field_page_and_bound_keys_follow_react_stately_steps(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-03-31"));
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DateField::new(state.clone())
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "pageup");
    press(cx, "pagedown");
    press(cx, "right");
    press(cx, "pagedown");
    press(cx, "home");
    press(cx, "end");
    press(cx, "right");
    press(cx, "pageup");
    press(cx, "pagedown");
    press(cx, "home");
    press(cx, "end");

    assert_eq!(
        changed.borrow().as_slice(),
        [
            "2025-05-31",
            "2025-03-31",
            "2025-03-24",
            "2025-03-01",
            "2025-03-31",
            "2030-03-31",
            "2025-03-31",
            "0001-03-31",
            "9999-03-31",
        ]
    );
}

#[gpui::test]
fn date_field_read_only_page_and_bound_keys_are_inert(cx: &mut TestAppContext) {
    let read_only = Rc::new(Cell::new(true));
    let read_only_for_view = read_only.clone();
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-03-31"));
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
    press(cx, "right");
    press(cx, "pageup");
    press(cx, "home");
    press(cx, "end");
    assert!(changed.borrow().is_empty());
    assert_eq!(
        cx.update(|_, cx| state.read(cx).value().to_owned()),
        "2025-03-31"
    );

    read_only.set(false);
    refresh(cx);
    press(cx, "pageup");
    assert_eq!(
        changed.borrow().as_slice(),
        ["2025-03-07"],
        "read-only Right still moves Month to Day, whose PageUp step is seven"
    );
}

#[gpui::test]
fn date_picker_alt_open_escape_restore_and_space_reopen(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "alt-down");
    assert_eq!(opened.borrow().as_slice(), ["true"]);
    press(cx, "escape");
    assert_eq!(opened.borrow().as_slice(), ["true", "false"]);
    press(cx, "space");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true", "false", "true"],
        "Escape must restore the initiating field so Space reopens directly"
    );
}

#[gpui::test]
fn date_picker_selection_restore_allows_space_reopen(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "alt-up");
    press(cx, "enter");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true", "false"],
        "the Enter that completes selection must close once, not refire the trigger"
    );
    press(cx, "space");
    assert_eq!(opened.borrow().as_slice(), ["true", "false", "true"]);
}

#[gpui::test]
fn date_picker_segments_do_not_open_but_the_trigger_does(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 6, 15)));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    click(cx, 20., 18.);
    assert!(
        opened.borrow().is_empty(),
        "a pointer press on an editable segment focuses it without opening the calendar"
    );
    // Three date segments occupy roughly 82px after the 12px inset; the
    // separate 24px trigger follows them rather than stretching to the edge.
    click(cx, 124., 18.);
    assert_eq!(opened.borrow().as_slice(), ["true"]);
    press(cx, "escape");
    press(cx, "space");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true", "false", "true"],
        "trigger-initiated dismissal restores the separate trigger"
    );
}

#[gpui::test]
fn date_picker_trigger_is_a_separate_keyboard_stop(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(opened.borrow().as_slice(), ["true"]);
    press(cx, "escape");
    press(cx, "space");
    assert_eq!(opened.borrow().as_slice(), ["true", "false", "true"]);
}

#[gpui::test]
fn date_picker_incomplete_segment_survives_a_repaint(cx: &mut TestAppContext) {
    let changes = events();
    let changed = changes.clone();
    let state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 1, 15)));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let changes = changes.clone();
        DatePicker::new(state_for_view.clone())
            .on_change(move |date, _, _| {
                changes
                    .borrow_mut()
                    .push(date.map_or_else(|| "none".to_owned(), |date| date.format_iso()));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "delete");
    refresh(cx);
    press(cx, "right");
    press(cx, "delete");
    press(cx, "right");
    press(cx, "delete");

    assert_eq!(changed.borrow().as_slice(), ["none"]);
}

#[gpui::test]
fn date_range_picker_is_a_keyboard_stop_and_opens_with_alt_arrow(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DateRangePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "alt-up");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true"],
        "Alt+ArrowUp can only reach the picker after its field enters the tab order"
    );
    press(cx, "escape");
    press(cx, "space");
    assert_eq!(opened.borrow().as_slice(), ["true", "false", "true"]);
}

#[gpui::test]
fn date_range_picker_completed_selection_restores_the_initiating_field(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DateRangePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "alt-down");
    press(cx, "enter");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true"],
        "the first range endpoint keeps the calendar open"
    );
    press(cx, "enter");
    assert_eq!(opened.borrow().as_slice(), ["true", "false"]);
    press(cx, "space");
    assert_eq!(
        opened.borrow().as_slice(),
        ["true", "false", "true"],
        "a completed range restores its initiating field for direct reopening"
    );
}

#[gpui::test]
fn date_range_picker_segments_do_not_open_but_the_trigger_does(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| {
        DateRangeState::with_range(
            cx,
            Some(Date::new(2025, 6, 15)),
            Some(Date::new(2025, 6, 20)),
        )
    });
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DateRangePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    click(cx, 20., 18.);
    assert!(opened.borrow().is_empty());
    click(cx, 300., 18.);
    assert_eq!(opened.borrow().as_slice(), ["true"]);
    press(cx, "escape");
    press(cx, "space");
    assert_eq!(opened.borrow().as_slice(), ["true", "false", "true"]);
}

#[gpui::test]
fn date_picker_field_is_editable_like_the_composed_v3_date_field(cx: &mut TestAppContext) {
    let state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 6, 15)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DatePicker::new(state_for_view.clone()).into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 1, 15))
    );
}

#[gpui::test]
fn date_range_picker_start_field_is_editable(cx: &mut TestAppContext) {
    let state = cx.new(|cx| {
        DateRangeState::with_range(
            cx,
            Some(Date::new(2025, 6, 15)),
            Some(Date::new(2025, 6, 20)),
        )
    });
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DateRangePicker::new(state_for_view.clone()).into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).start),
        Some(Date::new(2025, 1, 15))
    );
}

#[gpui::test]
fn date_range_picker_keeps_start_and_end_fields_distinct(cx: &mut TestAppContext) {
    let state = cx.new(|cx| {
        DateRangeState::with_range(
            cx,
            Some(Date::new(2025, 6, 15)),
            Some(Date::new(2025, 6, 20)),
        )
    });
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DateRangePicker::new(state_for_view.clone()).into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).start),
        Some(Date::new(2025, 1, 15))
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).end),
        Some(Date::new(2025, 6, 20))
    );

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).start),
        Some(Date::new(2025, 1, 15))
    );
    assert_eq!(
        cx.update(|_, cx| state.read(cx).end),
        Some(Date::new(2025, 1, 20))
    );
}

#[gpui::test]
fn date_picker_trigger_toggles_uncontrolled_without_duplicate_outside_close(
    cx: &mut TestAppContext,
) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    click(cx, 124., 18.);
    click(cx, 124., 18.);
    assert_eq!(opened.borrow().as_slice(), ["true", "false"]);

    click(cx, 124., 18.);
    click(cx, 400., 18.);
    assert_eq!(
        opened.borrow().as_slice(),
        ["true", "false", "true", "false"]
    );
}

#[gpui::test]
fn date_picker_trigger_toggles_controlled_open_state(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let is_open = Rc::new(Cell::new(false));
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state;
    let open_for_view = is_open;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DatePicker::new(state_for_view.clone())
            .is_open(open_for_view.get())
            .on_open_change({
                let open_for_view = open_for_view.clone();
                move |open, _, _| {
                    open_for_view.set(open);
                    opens.borrow_mut().push(open.to_string());
                }
            })
            .into_any_element()
    });

    click(cx, 124., 18.);
    refresh(cx);
    click(cx, 124., 18.);
    assert_eq!(opened.borrow().as_slice(), ["true", "false"]);
}

#[gpui::test]
fn read_only_pickers_stay_navigable_but_do_not_edit_select_or_open(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let date_state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 6, 15)));
    let range_state = cx.new(|cx| {
        DateRangeState::with_range(
            cx,
            Some(Date::new(2025, 6, 15)),
            Some(Date::new(2025, 6, 20)),
        )
    });
    let date_for_view = date_state.clone();
    let range_for_view = range_state.clone();
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(
                DatePicker::new(date_for_view.clone())
                    .is_read_only(true)
                    .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string())),
            )
            .child(DateRangePicker::new(range_for_view.clone()).is_read_only(true))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "home");
    press(cx, "alt-down");
    click(cx, 124., 18.);
    assert_eq!(
        cx.update(|_, cx| date_state.read(cx).selected),
        Some(Date::new(2025, 6, 15))
    );
    assert!(opened.borrow().is_empty());

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| {
            let state = range_state.read(cx);
            (state.start, state.end)
        }),
        (Some(Date::new(2025, 6, 15)), Some(Date::new(2025, 6, 20)))
    );
    click(cx, 300., 18.);
    assert!(opened.borrow().is_empty());
}

#[gpui::test]
fn date_range_text_edits_preserve_endpoint_identity_when_crossing(cx: &mut TestAppContext) {
    let state = cx.new(|cx| {
        DateRangeState::with_range(
            cx,
            Some(Date::new(2025, 6, 15)),
            Some(Date::new(2025, 6, 20)),
        )
    });
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DateRangePicker::new(state_for_view.clone()).into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "end");
    assert_eq!(
        cx.update(|_, cx| {
            let state = state.read(cx);
            (state.start, state.end)
        }),
        (Some(Date::new(2025, 6, 30)), Some(Date::new(2025, 6, 20)))
    );

    press(cx, "tab");
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| {
            let state = state.read(cx);
            (state.start, state.end)
        }),
        (Some(Date::new(2025, 6, 30)), Some(Date::new(2025, 1, 20)))
    );
}

#[gpui::test]
fn date_picker_invalid_text_keeps_last_valid_calendar_value(cx: &mut TestAppContext) {
    let state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 6, 15)));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DatePicker::new(state_for_view.clone())
            .min_value(Date::new(2025, 6, 10))
            .max_value(Date::new(2025, 6, 20))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "end");
    refresh(cx);
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 6, 15)),
        "an invalid complete field edit must not replace the last valid picker value"
    );

    press(cx, "2");
    press(cx, "0");
    press(cx, "2");
    press(cx, "5");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 6, 15)),
        "the valid correction keeps the unchanged month/day selection"
    );
}

#[gpui::test]
fn date_range_picker_trigger_toggles_without_duplicate_outside_close(cx: &mut TestAppContext) {
    let opens = events();
    let opened = opens.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state;
    let cx = open_host(cx, move || {
        let opens = opens.clone();
        DateRangePicker::new(state_for_view.clone())
            .on_open_change(move |open, _, _| opens.borrow_mut().push(open.to_string()))
            .into_any_element()
    });

    click(cx, 300., 18.);
    click(cx, 300., 18.);
    click(cx, 300., 18.);
    assert_eq!(opened.borrow().as_slice(), ["true", "false", "true"]);
}

#[gpui::test]
fn date_field_form_reads_live_invalidity_after_render(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let state = cx.new(|cx| InputState::with_value(cx, "2025-06-15"));
    let form_field = DateField::new(state.clone())
        .name("date")
        .form_field()
        .expect("named date field");
    let form = Form::new()
        .field(form_field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let state_for_view = state;
    let cx = open_host(cx, move || {
        DateField::new(state_for_view.clone())
            .min_value(Date::new(2025, 6, 20))
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
}

#[gpui::test]
fn date_picker_form_invalid_submit_focuses_its_field(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let state = cx.new(|cx| CalendarState::with_selected(cx, Date::new(2025, 6, 15)));
    let picker = DatePicker::new(state.clone()).name("date").is_invalid(true);
    let form_field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new()
        .field(form_field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .is_invalid(true)
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
    press(cx, "home");
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 1, 15)),
        "an invalid picker submit focuses the actual DateField"
    );
}

#[gpui::test]
fn required_empty_date_picker_blocks_form_and_focuses_its_field(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let state = cx.new(|cx| CalendarState::new(cx));
    let picker = DatePicker::new(state.clone())
        .name("date")
        .is_required(true);
    let form_field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new()
        .field(form_field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .is_required(true)
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
    for key in ["0", "1", "0", "1", "2", "0", "2", "5"] {
        press(cx, key);
    }
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 1, 1)),
        "a required picker submit focuses the actual DateField"
    );
}

#[gpui::test]
fn required_empty_date_range_picker_blocks_form_and_focuses_start(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let state = cx.new(|cx| DateRangeState::new(cx));
    let picker = DateRangePicker::new(state.clone())
        .start_name("start")
        .end_name("end")
        .is_required(true);
    let fields = cx.update(|cx| picker.form_fields(cx));
    let form = fields
        .into_iter()
        .fold(Form::new(), Form::field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DateRangePicker::new(state_for_view.clone())
                    .start_name("start")
                    .end_name("end")
                    .is_required(true)
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
    for key in ["0", "1", "0", "1", "2", "0", "2", "5"] {
        press(cx, key);
    }
    assert_eq!(
        cx.update(|_, cx| state.read(cx).start),
        Some(Date::new(2025, 1, 1)),
        "a required range submit focuses the start DateField first"
    );
}

#[gpui::test]
fn date_picker_custom_validation_blocks_native_form_submission(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let selected = Date::new(2025, 6, 15);
    let state = cx.new(|cx| CalendarState::with_selected(cx, selected));
    let picker = DatePicker::new(state.clone())
        .name("date")
        .validate(move |value| {
            (*value == Some(selected)).then(|| "That date is already booked".into())
        });
    let form_field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new()
        .field(form_field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state;
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .validate(move |value| {
                        (*value == Some(selected)).then(|| "That date is already booked".into())
                    })
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
}

#[gpui::test]
fn date_picker_aria_custom_validation_does_not_block_submission(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let selected = Date::new(2025, 6, 15);
    let state = cx.new(|cx| CalendarState::with_selected(cx, selected));
    let picker = DatePicker::new(state.clone())
        .name("date")
        .validation_behavior(ValidationBehavior::Allow)
        .validate(move |value| {
            (*value == Some(selected)).then(|| "That date is already booked".into())
        });
    let form_field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new().field(form_field).on_submit(move |data, _, _| {
        submitted_for_form
            .borrow_mut()
            .push(data.text("date").unwrap_or_default().to_string());
    });
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state;
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .validation_behavior(ValidationBehavior::Allow)
                    .validate(move |value| {
                        (*value == Some(selected)).then(|| "That date is already booked".into())
                    })
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), [selected.format_iso()]);
}

#[gpui::test]
fn date_picker_auto_focuses_its_editable_field(cx: &mut TestAppContext) {
    let state = cx.new(|cx| CalendarState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DatePicker::new(state_for_view.clone())
            .auto_focus(true)
            .into_any_element()
    });

    for key in ["0", "1", "0", "1", "2", "0", "2", "5"] {
        press(cx, key);
    }
    assert_eq!(
        cx.update(|_, cx| state.read(cx).selected),
        Some(Date::new(2025, 1, 1))
    );
}

#[gpui::test]
fn date_range_picker_custom_validation_blocks_native_form_submission(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let selected = (Date::new(2025, 6, 15), Date::new(2025, 6, 20));
    let state = cx.new(|cx| DateRangeState::with_range(cx, Some(selected.0), Some(selected.1)));
    let picker = DateRangePicker::new(state.clone())
        .start_name("start")
        .end_name("end")
        .validate(move |value| (*value == Some(selected)).then(|| "That range is booked".into()));
    let fields = cx.update(|cx| picker.form_fields(cx));
    let form = fields
        .into_iter()
        .fold(Form::new(), Form::field)
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state;
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DateRangePicker::new(state_for_view.clone())
                    .start_name("start")
                    .end_name("end")
                    .validate(move |value| {
                        (*value == Some(selected)).then(|| "That range is booked".into())
                    })
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);
}

#[gpui::test]
fn date_range_picker_aria_custom_validation_does_not_block_submission(cx: &mut TestAppContext) {
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let selected = (Date::new(2025, 6, 15), Date::new(2025, 6, 20));
    let state = cx.new(|cx| DateRangeState::with_range(cx, Some(selected.0), Some(selected.1)));
    let picker = DateRangePicker::new(state.clone())
        .start_name("start")
        .end_name("end")
        .validation_behavior(ValidationBehavior::Allow)
        .validate(move |value| (*value == Some(selected)).then(|| "That range is booked".into()));
    let fields = cx.update(|cx| picker.form_fields(cx));
    let form = fields
        .into_iter()
        .fold(Form::new(), Form::field)
        .on_submit(move |data, _, _| {
            submitted_for_form.borrow_mut().push(format!(
                "{}:{}",
                data.text("start").unwrap_or_default(),
                data.text("end").unwrap_or_default()
            ));
        });
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state;
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DateRangePicker::new(state_for_view.clone())
                    .start_name("start")
                    .end_name("end")
                    .validation_behavior(ValidationBehavior::Allow)
                    .validate(move |value| {
                        (*value == Some(selected)).then(|| "That range is booked".into())
                    })
            })
            .into_any_element()
    });

    cx.update(|window, cx| submit(window, cx));
    assert_eq!(
        submitted.borrow().as_slice(),
        [format!(
            "{}:{}",
            selected.0.format_iso(),
            selected.1.format_iso()
        )]
    );
}

#[gpui::test]
fn date_range_picker_auto_focuses_its_start_field(cx: &mut TestAppContext) {
    let state = cx.new(|cx| DateRangeState::new(cx));
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        DateRangePicker::new(state_for_view.clone())
            .auto_focus(true)
            .into_any_element()
    });

    for key in ["0", "1", "0", "1", "2", "0", "2", "5"] {
        press(cx, key);
    }
    assert_eq!(
        cx.update(|_, cx| state.read(cx).start),
        Some(Date::new(2025, 1, 1))
    );
}

#[gpui::test]
fn date_picker_reset_restores_display_and_validity_after_repaint(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let default = Date::new(2025, 6, 15);
    let state = cx.new(|cx| CalendarState::with_selected(cx, default));
    let picker = DatePicker::new(state.clone())
        .name("date")
        .default_value(default)
        .min_value(Date::new(2025, 6, 10))
        .max_value(Date::new(2025, 6, 20));
    let form_field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new()
        .field(form_field)
        .on_submit(move |data, _, _| {
            submitted_for_form
                .borrow_mut()
                .push(data.text("date").unwrap_or_default().to_string());
        })
        .on_invalid(move |_, _, _| invalids_for_form.borrow_mut().push("invalid".to_owned()));
    let submit = form.submit_handler();
    let reset = form.reset_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .default_value(default)
                    .min_value(Date::new(2025, 6, 10))
                    .max_value(Date::new(2025, 6, 20))
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "end");
    refresh(cx);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["invalid"]);

    invalids.borrow_mut().clear();
    cx.update(|window, cx| reset(window, cx));
    refresh(cx);
    cx.update(|window, cx| submit(window, cx));
    assert!(invalids.borrow().is_empty());
    assert_eq!(submitted.borrow().as_slice(), [default.format_iso()]);
    assert_eq!(cx.update(|_, cx| state.read(cx).selected), Some(default));
}

#[gpui::test]
fn date_picker_invalid_form_data_uses_the_displayed_text(cx: &mut TestAppContext) {
    let invalids = events();
    let recorded = invalids.clone();
    let selected = Date::new(2025, 6, 15);
    let state = cx.new(|cx| CalendarState::with_selected(cx, selected));
    let picker = DatePicker::new(state.clone())
        .name("date")
        .min_value(Date::new(2025, 6, 10))
        .max_value(Date::new(2025, 6, 20));
    let field = cx.update(|cx| picker.form_field(cx).expect("named picker field"));
    let form = Form::new().field(field).on_invalid(move |data, _, _| {
        recorded
            .borrow_mut()
            .push(data.text("date").unwrap_or_default().to_string());
    });
    let submit = form.submit_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DatePicker::new(state_for_view.clone())
                    .name("date")
                    .min_value(Date::new(2025, 6, 10))
                    .max_value(Date::new(2025, 6, 20))
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "end");
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["9999-06-15"]);
    assert_eq!(cx.update(|_, cx| state.read(cx).selected), Some(selected));
}

#[gpui::test]
fn date_range_picker_end_only_field_resets_after_invalid_text_and_repaint(cx: &mut TestAppContext) {
    let invalids = events();
    let invalids_for_form = invalids.clone();
    let submitted = events();
    let submitted_for_form = submitted.clone();
    let default = (Date::new(2025, 6, 10), Date::new(2025, 6, 20));
    let state = cx.new(|cx| DateRangeState::with_range(cx, Some(default.0), Some(default.1)));
    let picker = DateRangePicker::new(state.clone())
        .end_name("end")
        .default_value(default)
        .min_value(Date::new(2025, 6, 1))
        .max_value(Date::new(2025, 6, 30));
    let fields = cx.update(|cx| picker.form_fields(cx));
    assert_eq!(fields.len(), 1);
    let form = Form::new()
        .field(fields.into_iter().next().unwrap())
        .on_submit(move |data, _, _| {
            submitted_for_form
                .borrow_mut()
                .push(data.text("end").unwrap_or_default().to_string());
        })
        .on_invalid(move |data, _, _| {
            invalids_for_form
                .borrow_mut()
                .push(data.text("end").unwrap_or_default().to_string());
        });
    let submit = form.submit_handler();
    let reset = form.reset_handler();
    let picker = Rc::new(RefCell::new(Some(picker)));
    let picker_for_view = picker;
    let state_for_view = state.clone();
    let cx = open_host(cx, move || {
        picker_for_view
            .borrow_mut()
            .take()
            .unwrap_or_else(|| {
                DateRangePicker::new(state_for_view.clone())
                    .end_name("end")
                    .default_value(default)
                    .min_value(Date::new(2025, 6, 1))
                    .max_value(Date::new(2025, 6, 30))
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    press(cx, "end");
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(invalids.borrow().as_slice(), ["9999-06-20"]);

    cx.update(|window, cx| reset(window, cx));
    refresh(cx);
    cx.update(|window, cx| submit(window, cx));
    assert_eq!(submitted.borrow().as_slice(), [default.1.format_iso()]);
    assert_eq!(
        cx.update(|_, cx| {
            let state = state.read(cx);
            (state.start, state.end)
        }),
        (Some(default.0), Some(default.1))
    );
}
