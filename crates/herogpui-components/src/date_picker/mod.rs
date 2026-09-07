//! DatePicker, DateRangePicker & DateField — port of the v3
//! `@heroui/date-picker` family: a popover calendar plus ISO text entry.
//!
//! All three share [`DateConstraints`] for `minValue` / `maxValue` /
//! `isDateUnavailable` / `firstDayOfWeek`.

use gpui::{
    prelude::*, px, App, Entity, Focusable, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::element_id;
use herogpui_theme::ActiveTheme;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::OnceLock,
};

use crate::{
    a11y::{self, A11y as _},
    calendar::{days_from_civil, Calendar, CalendarState, Date},
    date_constraints::{DateConstraints, Weekday},
    icons,
};

pub(super) type OnChange = std::sync::Arc<dyn Fn(&Option<Date>, &mut Window, &mut App) + 'static>;

pub(super) type DateFieldFormState = Rc<RefCell<crate::form::LiveFormFieldState>>;

thread_local! {
    static DATE_FIELD_FORM_STATES: RefCell<HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
    static DATE_PICKER_FORM_STATES: RefCell<HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
    static DATE_RANGE_PICKER_FORM_STATES: RefCell<HashMap<(u64, bool), std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
}

pub(super) fn registered_date_field_form_state(entity_id: u64) -> Option<DateFieldFormState> {
    DATE_FIELD_FORM_STATES.with(|states| {
        states
            .borrow()
            .get(&entity_id)
            .and_then(|state| state.upgrade())
    })
}

pub(super) fn date_field_form_state(entity_id: u64) -> DateFieldFormState {
    DATE_FIELD_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states.get(&entity_id).and_then(|state| state.upgrade()) {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert(entity_id, Rc::downgrade(&state));
        state
    })
}

pub(super) fn date_picker_form_state(entity_id: u64) -> DateFieldFormState {
    DATE_PICKER_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states.get(&entity_id).and_then(|state| state.upgrade()) {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert(entity_id, Rc::downgrade(&state));
        state
    })
}

pub(super) fn date_range_picker_form_state(entity_id: u64, end: bool) -> DateFieldFormState {
    DATE_RANGE_PICKER_FORM_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if let Some(state) = states
            .get(&(entity_id, end))
            .and_then(|state| state.upgrade())
        {
            return state;
        }
        let state = Rc::new(RefCell::new(crate::form::LiveFormFieldState {
            value: crate::form::FormValue::Text(SharedString::default()),
            is_invalid: false,
            is_successful: true,
            focus: None,
            restore: None,
        }));
        states.insert((entity_id, end), Rc::downgrade(&state));
        state
    })
}

#[allow(clippy::arc_with_non_send_sync)]
pub(super) fn install_date_field_restore(
    form_state: &DateFieldFormState,
    input_state: Entity<crate::input::InputState>,
    default_text: SharedString,
) {
    let restore_state = form_state.clone();
    form_state.borrow_mut().restore = Some(std::sync::Arc::new(move |_, cx| {
        let value = default_text.clone();
        input_state.update(cx, |state, cx| {
            state.set_value(value.to_string());
            cx.notify();
        });
        let mut state = restore_state.borrow_mut();
        state.value = crate::form::FormValue::Text(value);
        state.is_invalid = false;
    }));
}

mod field;
mod picker;
mod range;

pub use field::*;
#[allow(unused_imports)]
pub(super) use field::*;
pub use picker::*;
#[allow(unused_imports)]
pub(super) use picker::*;
pub use range::*;
#[allow(unused_imports)]
pub(super) use range::*;
