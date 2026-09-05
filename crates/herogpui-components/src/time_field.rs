//! TimeField — port of `@heroui/time-field` (v3).
//!
//! Segment-by-segment time entry. Clicking a segment focuses it; the stepper
//! buttons adjust whichever segment has focus.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, OnceLock},
};

use gpui::{
    div, prelude::*, px, App, ElementId, Entity, InteractiveElement, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

use crate::{icons, util};

/// Whether a [`TimeField`] shows a 12- or 24-hour clock (`hourCycle`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HourCycle {
    H12,
    H24,
}

impl Default for HourCycle {
    fn default() -> Self {
        system_time_format().hour_cycle
    }
}

impl HourCycle {
    pub const ALL: [HourCycle; 2] = [HourCycle::H12, HourCycle::H24];

    pub fn label(self) -> &'static str {
        match self {
            HourCycle::H12 => "12-hour",
            HourCycle::H24 => "24-hour",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RegionalTimeFormat {
    hour_cycle: HourCycle,
    minute_pattern: RegionalTimePattern,
    second_pattern: RegionalTimePattern,
    am: String,
    pm: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RegionalTimePattern {
    pub(crate) order: Vec<TimeSegment>,
    pub(crate) literals: Vec<String>,
    pub(crate) hour_has_leading_zero: bool,
    pub(crate) hour_zero_based: bool,
    pub(crate) minute_has_leading_zero: bool,
    pub(crate) second_has_leading_zero: bool,
    pub(crate) am: String,
    pub(crate) pm: String,
}

impl RegionalTimeFormat {
    fn for_locale(locale: &str) -> Option<Self> {
        Self::for_locale_with_cycle(locale, None)
    }

    fn for_locale_with_cycle(locale: &str, hour_cycle: Option<HourCycle>) -> Option<Self> {
        use icu_datetime::{
            fieldsets,
            input::Time as IcuTime,
            provider::{
                fields::{FieldLength, FieldSymbol, Hour as IcuHour},
                pattern::{reference, runtime, PatternItem},
            },
            DateTimeFormatterPreferences, NoCalendarFormatter,
        };
        use icu_locale_core::{
            preferences::extensions::unicode::keywords::HourCycle as IcuHourCycle,
            Locale as IcuLocale,
        };

        let locale = locale.parse::<IcuLocale>().ok()?;
        let mut preferences = DateTimeFormatterPreferences::from(locale);
        if let Some(hour_cycle) = hour_cycle {
            preferences.hour_cycle = Some(match hour_cycle {
                HourCycle::H12 => IcuHourCycle::Clock12,
                HourCycle::H24 => IcuHourCycle::Clock24,
            });
        }
        let parse_pattern = |precision| {
            let formatter = NoCalendarFormatter::try_new(preferences, precision).ok()?;
            let formatted = formatter.format(&IcuTime::start_of_day());
            let pattern: runtime::Pattern<'_> = formatted.pattern().into();
            let pattern = reference::Pattern::from(&pattern);
            let mut order = Vec::with_capacity(4);
            let mut literals = Vec::with_capacity(5);
            let mut literal = String::new();
            let mut hour_has_leading_zero = None;
            let mut hour_zero_based = None;
            let mut hour_cycle = None;
            let mut minute_has_leading_zero = None;
            let mut second_has_leading_zero = None;
            for item in pattern.into_items() {
                let field = match item {
                    PatternItem::Literal(ch) => {
                        literal.push(ch);
                        continue;
                    }
                    PatternItem::Field(field) => field,
                };
                let segment = match field.symbol {
                    FieldSymbol::Hour(hour) => {
                        hour_has_leading_zero = Some(field.length == FieldLength::Two);
                        hour_zero_based = Some(hour == IcuHour::H11);
                        hour_cycle = Some(match hour {
                            IcuHour::H11 | IcuHour::H12 => HourCycle::H12,
                            IcuHour::H23 => HourCycle::H24,
                        });
                        TimeSegment::Hour
                    }
                    FieldSymbol::Minute => {
                        minute_has_leading_zero = Some(field.length == FieldLength::Two);
                        TimeSegment::Minute
                    }
                    FieldSymbol::Second(_) => {
                        second_has_leading_zero = Some(field.length == FieldLength::Two);
                        TimeSegment::Second
                    }
                    FieldSymbol::DayPeriod(_) => TimeSegment::Meridiem,
                    _ => return None,
                };
                if order.contains(&segment) {
                    return None;
                }
                literals.push(std::mem::take(&mut literal));
                order.push(segment);
            }
            literals.push(literal);
            Some((
                RegionalTimePattern {
                    order,
                    literals,
                    hour_has_leading_zero: hour_has_leading_zero?,
                    hour_zero_based: hour_zero_based?,
                    minute_has_leading_zero: minute_has_leading_zero?,
                    second_has_leading_zero: second_has_leading_zero.unwrap_or(false),
                    am: String::new(),
                    pm: String::new(),
                },
                hour_cycle?,
            ))
        };

        let (minute_pattern, hour_cycle) = parse_pattern(fieldsets::T::hm())?;
        let (second_pattern, second_hour_cycle) = parse_pattern(fieldsets::T::hms())?;
        if hour_cycle != second_hour_cycle {
            return None;
        }
        let (am, pm) = if hour_cycle == HourCycle::H12 {
            let formatter = NoCalendarFormatter::try_new(preferences, fieldsets::T::hm()).ok()?;
            (
                minute_pattern
                    .day_period_from_formatted(
                        &formatter
                            .format(&IcuTime::try_new(1, 5, 0, 0).ok()?)
                            .to_string(),
                    )
                    .unwrap_or_else(|| "AM".to_owned()),
                minute_pattern
                    .day_period_from_formatted(
                        &formatter
                            .format(&IcuTime::try_new(13, 5, 0, 0).ok()?)
                            .to_string(),
                    )
                    .unwrap_or_else(|| "PM".to_owned()),
            )
        } else {
            ("AM".to_owned(), "PM".to_owned())
        };
        Some(Self {
            hour_cycle,
            minute_pattern,
            second_pattern,
            am,
            pm,
        })
    }

    fn for_preferences(locale: &locale_config::Locale) -> Option<Self> {
        locale
            .tags_for("time")
            .find_map(|tag| Self::for_locale(tag.as_ref()))
    }
}

fn system_time_format() -> &'static RegionalTimeFormat {
    static SYSTEM_TIME_FORMAT: OnceLock<RegionalTimeFormat> = OnceLock::new();
    SYSTEM_TIME_FORMAT.get_or_init(|| {
        RegionalTimeFormat::for_preferences(&locale_config::Locale::user_default()).unwrap_or(
            RegionalTimeFormat {
                hour_cycle: HourCycle::H24,
                minute_pattern: RegionalTimePattern::fallback(TimeGranularity::Minute, false),
                second_pattern: RegionalTimePattern::fallback(TimeGranularity::Second, false),
                am: "AM".to_owned(),
                pm: "PM".to_owned(),
            },
        )
    })
}

fn system_time_format_for_cycle(hour_cycle: HourCycle) -> &'static RegionalTimeFormat {
    static SYSTEM_H12_FORMAT: OnceLock<RegionalTimeFormat> = OnceLock::new();
    static SYSTEM_H24_FORMAT: OnceLock<RegionalTimeFormat> = OnceLock::new();
    let cache = match hour_cycle {
        HourCycle::H12 => &SYSTEM_H12_FORMAT,
        HourCycle::H24 => &SYSTEM_H24_FORMAT,
    };
    cache.get_or_init(|| {
        locale_config::Locale::user_default()
            .tags_for("time")
            .find_map(|tag| {
                RegionalTimeFormat::for_locale_with_cycle(tag.as_ref(), Some(hour_cycle))
            })
            .unwrap_or(RegionalTimeFormat {
                hour_cycle,
                minute_pattern: RegionalTimePattern::fallback(
                    TimeGranularity::Minute,
                    hour_cycle == HourCycle::H12,
                ),
                second_pattern: RegionalTimePattern::fallback(
                    TimeGranularity::Second,
                    hour_cycle == HourCycle::H12,
                ),
                am: "AM".to_owned(),
                pm: "PM".to_owned(),
            })
    })
}

impl RegionalTimeFormat {
    fn pattern(&self, granularity: TimeGranularity) -> RegionalTimePattern {
        let mut pattern = match granularity {
            TimeGranularity::Hour | TimeGranularity::Minute => self.minute_pattern.clone(),
            TimeGranularity::Second => self.second_pattern.clone(),
        };
        if granularity == TimeGranularity::Hour {
            let mut order = Vec::with_capacity(2);
            let mut literals = Vec::with_capacity(3);
            for (index, segment) in pattern.order.iter().copied().enumerate() {
                if matches!(segment, TimeSegment::Hour | TimeSegment::Meridiem) {
                    literals.push(pattern.literals[index].clone());
                    order.push(segment);
                }
            }
            literals.push(String::new());
            pattern.order = order;
            pattern.literals = literals;
        }
        pattern.am.clone_from(&self.am);
        pattern.pm.clone_from(&self.pm);
        pattern
    }
}

impl RegionalTimePattern {
    fn fallback(granularity: TimeGranularity, twelve_hour: bool) -> Self {
        let order = TimeSegment::order(granularity, twelve_hour);
        let literals = order
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                if index == 0 {
                    String::new()
                } else if *segment == TimeSegment::Meridiem {
                    " ".to_owned()
                } else {
                    ":".to_owned()
                }
            })
            .chain(std::iter::once(String::new()))
            .collect();
        Self {
            order,
            literals,
            hour_has_leading_zero: !twelve_hour,
            hour_zero_based: false,
            minute_has_leading_zero: true,
            second_has_leading_zero: true,
            am: "AM".to_owned(),
            pm: "PM".to_owned(),
        }
    }

    fn day_period_from_formatted(&self, formatted: &str) -> Option<String> {
        let index = self
            .order
            .iter()
            .position(|segment| *segment == TimeSegment::Meridiem)?;
        let name = if index == 0 {
            let formatted = formatted.strip_prefix(self.literals.first()?.as_str())?;
            let numeric_start = formatted
                .char_indices()
                .find_map(|(index, ch)| ch.is_numeric().then_some(index))?;
            formatted[..numeric_start].strip_suffix(self.literals.get(1)?.as_str())?
        } else if index + 1 == self.order.len() {
            let numeric_end = formatted
                .char_indices()
                .filter_map(|(index, ch)| ch.is_numeric().then_some(index + ch.len_utf8()))
                .next_back()?;
            formatted[numeric_end..]
                .strip_prefix(self.literals.get(index)?.as_str())?
                .strip_suffix(self.literals.get(index + 1)?.as_str())?
        } else {
            return None;
        };
        (!name.is_empty()).then(|| name.to_owned())
    }

    pub(crate) fn hint(&self) -> String {
        let mut hint = String::new();
        for (index, segment) in self.order.iter().copied().enumerate() {
            hint.push_str(&self.literals[index]);
            hint.push_str(match segment {
                TimeSegment::Hour => "HH",
                TimeSegment::Minute => "MM",
                TimeSegment::Second => "SS",
                TimeSegment::Meridiem => self.am.as_str(),
            });
        }
        hint.push_str(self.literals.last().map_or("", String::as_str));
        hint
    }
}

pub(crate) fn regional_time_pattern(
    granularity: TimeGranularity,
    hour_cycle: HourCycle,
) -> RegionalTimePattern {
    system_time_format_for_cycle(hour_cycle).pattern(granularity)
}

/// A wall-clock time — the `TimeValue` of `@internationalized/date`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Time {
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Time {
    pub fn new(hour: u32, minute: u32) -> Self {
        Self {
            hour: hour.min(23),
            minute: minute.min(59),
            second: 0,
        }
    }

    pub fn with_second(mut self, second: u32) -> Self {
        self.second = second.min(59);
        self
    }

    /// Adds `delta` to one segment, wrapping within that segment's range.
    pub fn bump(self, segment: TimeSegment, delta: i32) -> Self {
        let wrap =
            |value: u32, len: i32| -> u32 { (((value as i32 + delta) % len + len) % len) as u32 };
        match segment {
            TimeSegment::Hour => Self {
                hour: wrap(self.hour, 24),
                ..self
            },
            TimeSegment::Minute => Self {
                minute: wrap(self.minute, 60),
                ..self
            },
            TimeSegment::Second => Self {
                second: wrap(self.second, 60),
                ..self
            },
            // Flipping the meridiem moves the hour by half a day.
            TimeSegment::Meridiem => Self {
                hour: (self.hour + 12) % 24,
                ..self
            },
        }
    }

    /// The displayed hour and suffix for a 12-hour clock.
    pub fn twelve_hour(self) -> (u32, &'static str) {
        let suffix = if self.hour < 12 { "AM" } else { "PM" };
        let hour = match self.hour % 12 {
            0 => 12,
            h => h,
        };
        (hour, suffix)
    }
}

/// A segment's digits, padded to two unless `shouldForceLeadingZeros` is off.
fn pad2(value: u32, pad: bool) -> String {
    if pad {
        format!("{value:02}")
    } else {
        value.to_string()
    }
}

/// `granularity` — the smallest unit the field shows. v3 defaults a time to
/// `minute`.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum TimeGranularity {
    /// Hour only.
    Hour,
    /// Hour and minute, which is v3's default for a time.
    #[default]
    Minute,
    /// Hour, minute and second.
    Second,
}

/// The editable segments of a [`TimeField`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSegment {
    Hour,
    Minute,
    Second,
    Meridiem,
}

impl TimeSegment {
    /// The segments a field of this granularity shows, in reading order.
    ///
    /// `pub(crate)` because a `DateField` below `day` granularity shows the same
    /// segments, edited the same way: one implementation, so the two fields
    /// cannot disagree about what a minute field looks like.
    pub(crate) fn order(granularity: TimeGranularity, twelve_hour: bool) -> Vec<TimeSegment> {
        let mut out = vec![TimeSegment::Hour];
        if granularity != TimeGranularity::Hour {
            out.push(TimeSegment::Minute);
        }
        if granularity == TimeGranularity::Second {
            out.push(TimeSegment::Second);
        }
        if twelve_hour {
            out.push(TimeSegment::Meridiem);
        }
        out
    }

    /// How many digits this segment holds — the point at which typing moves on.
    pub(crate) fn digits(self) -> usize {
        match self {
            TimeSegment::Meridiem => 0,
            _ => 2,
        }
    }

    /// `time` with this segment set to `value`, clamped to its range.
    pub(crate) fn with_value(
        self,
        time: Time,
        value: u32,
        twelve_hour: bool,
        zero_based_twelve_hour: bool,
    ) -> Time {
        match self {
            TimeSegment::Hour => {
                let hour = if twelve_hour {
                    // 12-hour entry keeps the half of the day the field is in.
                    let pm = time.hour >= 12;
                    let base = if zero_based_twelve_hour {
                        value.min(11)
                    } else {
                        value.clamp(1, 12) % 12
                    };
                    if pm {
                        base + 12
                    } else {
                        base
                    }
                } else {
                    value.min(23)
                };
                Time::new(hour, time.minute).with_second(time.second)
            }
            TimeSegment::Minute => Time::new(time.hour, value.min(59)).with_second(time.second),
            TimeSegment::Second => Time::new(time.hour, time.minute).with_second(value.min(59)),
            TimeSegment::Meridiem => time,
        }
    }
}

/// State entity for [`TimeField`].
pub struct TimeState {
    /// The complete value exposed to callbacks, validation and forms.
    pub value: Option<Time>,
    pub focused: TimeSegment,
    /// The local value used to draw segments while an edit is incomplete.
    display_value: Option<Time>,
    /// Segments cleared from an otherwise complete display. React Stately
    /// keeps this incomplete value locally and defers `onChange` until every
    /// displayed segment is complete again.
    cleared: Vec<TimeSegment>,
    segment_order: Vec<TimeSegment>,
    /// The last controlled prop seen. The outer `Option` distinguishes an
    /// uncontrolled field from an explicitly controlled `None`.
    last_controlled: Option<Option<Time>>,
    /// The field's tab stop, carried on the state the way
    /// [`crate::input::InputState`] carries its own: a `content` closure and
    /// the replacement field it draws must share one handle, or the closure
    /// reads a handle Tab and clicks never reach.
    pub(crate) focus_handle: gpui::FocusHandle,
}

impl TimeState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            value: None,
            focused: TimeSegment::Hour,
            display_value: None,
            cleared: Vec::new(),
            segment_order: Vec::new(),
            last_controlled: None,
            // A field is a tab stop: the handle carries that, not the element.
            focus_handle: cx.focus_handle().tab_stop(true),
        }
    }

    pub fn with_value(cx: &mut App, value: Time) -> Self {
        Self {
            value: Some(value),
            focused: TimeSegment::Hour,
            display_value: Some(value),
            cleared: Vec::new(),
            segment_order: Vec::new(),
            last_controlled: None,
            focus_handle: cx.focus_handle().tab_stop(true),
        }
    }

    /// Adjusts the focused segment, seeding an empty field from `seed`.
    pub fn bump_focused_from(&mut self, delta: i32, seed: Time) {
        let base = self.display_value.unwrap_or(seed);
        let value = base.bump(self.focused, delta);
        self.value = Some(value);
        self.display_value = Some(value);
        self.cleared.retain(|segment| *segment != self.focused);
    }

    /// [`bump_focused_from`](Self::bump_focused_from) seeded at midnight.
    pub fn bump_focused(&mut self, delta: i32) {
        self.bump_focused_from(delta, Time::new(0, 0));
    }

    fn visible_is_complete(&self, segments: &[TimeSegment]) -> bool {
        !segments
            .iter()
            .any(|segment| self.cleared.contains(segment))
    }

    fn set_uncontrolled_value(&mut self, value: Option<Time>) {
        self.value = value;
        self.display_value = value;
        self.cleared.clear();
    }

    fn sync_controlled(&mut self, value: Option<Time>) {
        if self.last_controlled != Some(value) {
            self.last_controlled = Some(value);
            self.display_value = value;
            self.cleared.clear();
        }
        // The controlled prop is always the committed value, but an unchanged
        // prop must not erase the local incomplete display on every render.
        self.value = value;
    }

    fn sync_external_value(&mut self) {
        if self.last_controlled.is_none()
            && self.cleared.is_empty()
            && self.display_value != self.value
        {
            self.display_value = self.value;
        }
    }

    fn sync_segment_order(&mut self, order: &[TimeSegment]) {
        if self.segment_order != order {
            if self.segment_order.is_empty() || !order.contains(&self.focused) {
                self.focused = order[0];
            }
            self.segment_order.clear();
            self.segment_order.extend_from_slice(order);
        }
    }

    /// Writes one display segment and returns whether the currently visible
    /// value became complete. Controlled fields report the candidate and then
    /// keep drawing their prop until the caller supplies the new value.
    fn edit_focused(&mut self, value: Time, segments: &[TimeSegment]) -> bool {
        if self.display_value.is_none() {
            self.cleared = segments.to_vec();
        }
        self.display_value = Some(value);
        self.cleared.retain(|segment| *segment != self.focused);
        if !self.visible_is_complete(segments) {
            return false;
        }

        match self.last_controlled {
            Some(controlled) => {
                self.value = controlled;
                self.display_value = controlled;
                self.cleared.clear();
            }
            None => self.set_uncontrolled_value(Some(value)),
        }
        true
    }

    /// Clears the active display segment. Once every currently displayed
    /// segment is empty, the field's value becomes null and reports that
    /// transition; otherwise React Stately keeps the incomplete display local.
    fn clear_focused(&mut self, segments: &[TimeSegment]) -> bool {
        if self.display_value.is_none() {
            return false;
        }
        if !self.cleared.contains(&self.focused) {
            self.cleared.push(self.focused);
        }
        if segments
            .iter()
            .all(|segment| self.cleared.contains(segment))
        {
            match self.last_controlled {
                Some(Some(value)) => {
                    self.value = Some(value);
                    self.display_value = Some(value);
                    self.cleared.clear();
                }
                Some(None) | None => self.set_uncontrolled_value(None),
            }
            return true;
        }
        false
    }
}

type Segment = Arc<dyn Fn(TimeSegment, SharedString) -> gpui::AnyElement + 'static>;

type OnTimeChange = Arc<dyn Fn(Option<Time>, &mut Window, &mut App) + 'static>;

type TimeFieldFormState = Rc<RefCell<crate::form::LiveFormFieldState>>;

thread_local! {
    static TIME_FIELD_FORM_STATES: RefCell<HashMap<u64, std::rc::Weak<RefCell<crate::form::LiveFormFieldState>>>> =
        RefCell::new(HashMap::new());
}

fn registered_time_field_form_state(entity_id: u64) -> Option<TimeFieldFormState> {
    TIME_FIELD_FORM_STATES.with(|states| {
        states
            .borrow()
            .get(&entity_id)
            .and_then(|state| state.upgrade())
    })
}

fn time_field_form_state(entity_id: u64) -> TimeFieldFormState {
    TIME_FIELD_FORM_STATES.with(|states| {
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

/// The time an HTML `<input type="time">` submits at the field's granularity.
fn time_form_text(value: Option<Time>, granularity: TimeGranularity) -> SharedString {
    value
        .map(|time| match granularity {
            TimeGranularity::Hour | TimeGranularity::Minute => {
                format!("{:02}:{:02}", time.hour, time.minute)
            }
            TimeGranularity::Second => {
                format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second)
            }
        })
        .unwrap_or_default()
        .into()
}

fn sync_time_field_form(
    form_state: &TimeFieldFormState,
    time_state: &Entity<TimeState>,
    is_disabled: bool,
    is_invalid: bool,
    granularity: TimeGranularity,
    cx: &App,
) {
    let mut state = form_state.borrow_mut();
    state.value =
        crate::form::FormValue::Text(time_form_text(time_state.read(cx).value, granularity));
    state.is_successful = !is_disabled;
    state.is_invalid = is_invalid;
    state.focus = Some(time_state.read(cx).focus_handle.clone());
}

fn install_time_field_restore(
    form_state: &TimeFieldFormState,
    time_state: Entity<TimeState>,
    default: Option<Time>,
    granularity: TimeGranularity,
    controlled: bool,
    on_change: Option<OnTimeChange>,
) {
    if default.is_none() && !controlled {
        return;
    }
    let restore_form = form_state.clone();
    form_state.borrow_mut().restore = Some(util::shared(move |window: &mut Window, cx: &mut App| {
        time_state.update(&mut *cx, |state, cx| {
            if controlled {
                state.sync_controlled(default);
            } else {
                state.set_uncontrolled_value(default);
            }
            cx.notify();
        });
        {
            let mut state = restore_form.borrow_mut();
            state.value = crate::form::FormValue::Text(time_form_text(default, granularity));
            state.is_invalid = false;
        }
        if controlled {
            if let Some(callback) = &on_change {
                callback(default, window, cx);
            }
        }
    }) as Arc<dyn Fn(&mut Window, &mut App)>);
}

/// State supplied to v3's TimeField children render function.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeFieldRenderState {
    /// Whether the field is disabled.
    pub is_disabled: bool,
    /// Whether controlled, server, custom or bound validation is invalid.
    pub is_invalid: bool,
    /// Whether segments can be focused but not edited.
    pub is_read_only: bool,
    /// Whether the field is required.
    pub is_required: bool,
    /// Whether the field's input owns focus.
    pub is_focused: bool,
    /// Whether focus is inside the field.
    pub is_focus_within: bool,
    /// Whether keyboard-visible focus chrome should be shown.
    pub is_focus_visible: bool,
}

/// HeroUI TimeField.
#[derive(IntoElement)]
pub struct TimeField {
    /// `segment` — v3's render prop for one editable segment, handed which
    /// segment it is and the text the field would have shown.
    segment: Option<Segment>,
    /// `name` — read back by [`TimeField::form_field`].
    name: Option<SharedString>,
    /// `validationBehavior` — carried on this field's form field.
    validation_behavior: crate::form::ValidationBehavior,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<Time>,
    /// `value` — `Some(None)` is an explicitly controlled empty field.
    controlled_value: Option<Option<Time>>,
    state: Entity<TimeState>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<Option<Time>>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    /// `TimeField.Prefix` — content before the segments, drawn in the
    /// placeholder colour and inert.
    prefix: Option<gpui::AnyElement>,
    /// See [`TimeField::content`].
    content: Option<Arc<dyn Fn(TimeFieldRenderState) -> gpui::AnyElement + 'static>>,
    /// `TimeField.Suffix` — content after the segments
    /// (`.date-input-group__suffix`: `shrink-0 me-3` in the placeholder colour).
    suffix: Option<gpui::AnyElement>,
    variant: FieldVariant,
    hour_cycle: HourCycle,
    /// `granularity` — the smallest unit shown.
    granularity: TimeGranularity,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `shouldForceLeadingZeros` — force the hour to two digits instead of
    /// using the system regional format.
    should_force_leading_zeros: bool,
    full_width: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    is_invalid: bool,
    min_value: Option<Time>,
    max_value: Option<Time>,
    /// `placeholderValue` — seeds the steppers when the field is empty.
    placeholder_value: Option<Time>,
    on_change: Option<OnTimeChange>,
}

impl TimeField {
    /// `value` — writes through to the bound [`TimeState`].
    pub fn value(mut self, time: Option<Time>, cx: &mut App) -> Self {
        self.state.update(cx, |state, _| {
            state.sync_controlled(time);
        });
        self.controlled_value = Some(time);
        self
    }

    pub fn new(state: Entity<TimeState>) -> Self {
        Self {
            segment: None,
            name: None,
            validation_behavior: crate::form::ValidationBehavior::Native,
            default_value: None,
            controlled_value: None,
            state,
            label: None,
            description: None,
            error_message: None,
            content: None,
            prefix: None,
            suffix: None,
            variant: FieldVariant::Primary,
            hour_cycle: HourCycle::default(),
            granularity: TimeGranularity::default(),
            auto_focus: false,
            should_force_leading_zeros: false,
            full_width: false,
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            min_value: None,
            max_value: None,
            placeholder_value: None,
            on_change: None,
        }
    }

    /// `segment` — replaces the contents of each editable segment.
    ///
    /// The closure receives which [`TimeSegment`] it is drawing and the text
    /// the field would have shown, the values v3 passes into the same render
    /// prop.
    pub fn segment(
        mut self,
        render: impl Fn(TimeSegment, SharedString) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.segment = Some(Arc::new(render));
        self
    }

    /// `name` — the name this field submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `validationBehavior` — `Allow` shows the message without blocking form
    /// submission.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// The time is written `HH:MM`, or `HH:MM:SS` at second granularity, which
    /// matches an HTML `<input type="time">`. Needs `cx` because the value
    /// lives in the state entity. The
    /// returned field stays live: submit reads the entity, a disabled field is
    /// unsuccessful, and reset restores `defaultValue` (or reports it to a
    /// controlled owner).
    pub fn form_field(&self, cx: &App) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        let form_state = time_field_form_state(self.state.entity_id().as_u64());
        sync_time_field_form(
            &form_state,
            &self.state,
            self.is_disabled,
            self.is_invalid,
            self.granularity,
            cx,
        );
        install_time_field_restore(
            &form_state,
            self.state.clone(),
            self.default_value,
            self.granularity,
            self.controlled_value.is_some(),
            self.on_change.clone(),
        );
        Some(
            crate::form::FormField::live(name, form_state)
                .is_required(self.is_required)
                .validation_behavior(self.validation_behavior),
        )
    }

    /// `defaultValue` — the uncontrolled initial time.
    ///
    /// Written into the state on the first render only, so it seeds the
    /// component without fighting the user afterwards.
    pub fn default_value(mut self, value: Time) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
    }

    /// `TimeField.Prefix` — content before the segments.
    pub fn prefix(mut self, el: impl IntoElement) -> Self {
        self.prefix = Some(el.into_any_element());
        self
    }

    /// `TimeField.Suffix` — content after the segments.
    /// v3's field `children`-as-a-function, handed the complete
    /// [`TimeFieldRenderState`].
    pub fn content(
        mut self,
        render: impl Fn(TimeFieldRenderState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.content = Some(Arc::new(render));
        self
    }

    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn hour_cycle(mut self, cycle: HourCycle) -> Self {
        self.hour_cycle = cycle;
        self
    }

    /// `granularity` — the smallest unit the field shows.
    pub fn granularity(mut self, granularity: TimeGranularity) -> Self {
        self.granularity = granularity;
        self
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `shouldForceLeadingZeros` — force the hour to two digits. Without this
    /// flag, each numeric segment follows the system regional time pattern.
    pub fn should_force_leading_zeros(mut self, v: bool) -> Self {
        self.should_force_leading_zeros = v;
        self
    }

    /// `granularity="second"`, kept as its own flag because it reads better at
    /// the call site.
    pub fn show_seconds(mut self, v: bool) -> Self {
        self.granularity = if v {
            TimeGranularity::Second
        } else {
            TimeGranularity::Minute
        };
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `validate` — returns the message to show, or `None` when the time is
    /// fine. Receives `None` when the field is empty, as v3's `TimeValue | null`.
    ///
    /// The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&Option<Time>) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `minValue` — the earliest selectable time.
    pub fn min_value(mut self, time: Time) -> Self {
        self.min_value = Some(time);
        self
    }

    /// `maxValue` — the latest selectable time.
    pub fn max_value(mut self, time: Time) -> Self {
        self.max_value = Some(time);
        self
    }

    /// `placeholderValue` — the time the steppers start from when empty.
    pub fn placeholder_value(mut self, time: Time) -> Self {
        self.placeholder_value = Some(time);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(Option<Time>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for TimeField {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` seeds the state once, before anything reads it.
        if self.controlled_value.is_none() {
            if let Some(value) = self.default_value {
                let state = self.state.clone();
                util::seed_once(
                    window,
                    cx,
                    ElementId::Name(
                        format!("timefield-default-{}", self.state.entity_id().as_u64()).into(),
                    ),
                    move |cx| {
                        state.update(cx, |s, cx| {
                            s.set_uncontrolled_value(Some(value));
                            cx.notify();
                        });
                    },
                );
            }
        }
        if let Some(value) = self.controlled_value {
            self.state.update(cx, |state, _| {
                state.sync_controlled(value);
            });
        } else {
            self.state.update(cx, |state, _| {
                state.sync_external_value();
            });
        }

        let entity_id = self.state.entity_id().as_u64();
        if let Some(form_state) = registered_time_field_form_state(entity_id) {
            sync_time_field_form(
                &form_state,
                &self.state,
                self.is_disabled,
                self.is_invalid,
                self.granularity,
                cx,
            );
            install_time_field_restore(
                &form_state,
                self.state.clone(),
                self.default_value,
                self.granularity,
                self.controlled_value.is_some(),
                self.on_change.clone(),
            );
        }
        // A time field has no inner `Input` to hold the focus, so the handle
        // lives on the state itself -- the same answer `InputState` and
        // `NumberState` give. That is what lets a `content` closure and the
        // replacement field it draws share one handle: the field the closure
        // draws tracks the state's handle, so Tab and clicks move the focus
        // the closure is asked to report.
        let focus_handle = self.state.read(cx).focus_handle.clone();
        if self.auto_focus {
            util::focus_once(
                window,
                cx,
                ElementId::Name(format!("timefield-{entity_id}-autofocus").into()),
                &focus_handle,
            );
        }
        // Digits typed into the focused segment but not yet complete, so `1` in
        // the hour segment can still become `12`.
        let typing = window.use_keyed_state(
            ElementId::Name(format!("timefield-{entity_id}-typing").into()),
            cx,
            |_, _| String::new(),
        );

        let regional_time = regional_time_pattern(self.granularity, self.hour_cycle);
        self.state.update(cx, |state, _| {
            state.sync_segment_order(&regional_time.order);
        });

        let colors = cx.colors();
        let layout = cx.layout();
        let navigable = !self.is_disabled;
        let editable = navigable && !self.is_read_only;

        let (value, display_value, focused, cleared) = {
            let st = self.state.read(cx);
            (st.value, st.display_value, st.focused, st.cleared.clone())
        };

        // v3 order: the controlled flag, then server errors, then `validate`,
        // with `errorMessage` as the fallback.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&value)),
            self.error_message.clone(),
        );
        let is_invalid = validity.is_invalid;
        if let Some(form_state) = registered_time_field_form_state(entity_id) {
            sync_time_field_form(
                &form_state,
                &self.state,
                self.is_disabled,
                is_invalid,
                self.granularity,
                cx,
            );
        }
        if let Some(render) = self.content.clone() {
            let focused = focus_handle.is_focused(window);
            return render(TimeFieldRenderState {
                is_disabled: self.is_disabled,
                is_invalid,
                is_read_only: self.is_read_only,
                is_required: self.is_required,
                is_focused: focused,
                is_focus_within: focus_handle.contains_focused(window, cx),
                is_focus_visible: focused && util::focus_visible(cx),
            })
            .into_any_element();
        }

        let segments = regional_time.order.clone();

        let hour_cycle = self.hour_cycle;
        let pad_hour = self.should_force_leading_zeros || regional_time.hour_has_leading_zero;
        let pad_minute = regional_time.minute_has_leading_zero;
        let pad_second = regional_time.second_has_leading_zero;
        let zero_based_twelve_hour = regional_time.hour_zero_based;
        let am = regional_time.am.clone();
        let pm = regional_time.pm.clone();
        let segment_text = move |segment: TimeSegment| -> String {
            if cleared.contains(&segment) {
                return if segment == TimeSegment::Meridiem {
                    am.clone()
                } else {
                    "--".to_owned()
                };
            }
            let Some(t) = display_value else {
                return "--".to_owned();
            };
            match segment {
                TimeSegment::Hour => {
                    if hour_cycle == HourCycle::H12 {
                        let hour = if zero_based_twelve_hour {
                            t.hour % 12
                        } else {
                            t.twelve_hour().0
                        };
                        pad2(hour, pad_hour)
                    } else {
                        pad2(t.hour, pad_hour)
                    }
                }
                TimeSegment::Minute => pad2(t.minute, pad_minute),
                TimeSegment::Second => pad2(t.second, pad_second),
                TimeSegment::Meridiem => {
                    if t.hour < 12 {
                        am.clone()
                    } else {
                        pm.clone()
                    }
                }
            }
        };

        // `.date-input-group__input-container` is `flex flex-1 items-center` with
        // its own horizontal scroll, so a long value stays reachable without
        // widening the field.
        let mut group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .px(px(12.))
            .h(util::FIELD_HEIGHT)
            .rounded(util::field_radius(cx))
            .text_size(util::FIELD_TEXT)
            .font_family(util::MONO_FONT)
            .text_color(colors.field.foreground);

        group = util::apply_field_chrome(
            group,
            self.variant,
            is_invalid,
            focus_handle.is_focused(window),
            cx,
        );

        // v3 drives a time field from the keyboard: the arrows step the focused
        // segment and walk between segments, and digits type into it.
        if navigable {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let buffer = typing;
            let fh = focus_handle.clone();
            let order = segments.clone();
            let twelve_hour = self.hour_cycle == HourCycle::H12;
            let seed = self.placeholder_value.unwrap_or(Time::new(0, 0));
            let min_value = self.min_value;
            let max_value = self.max_value;
            let is_read_only = self.is_read_only;
            group = group
                .track_focus(&focus_handle)
                .key_context("TimeField")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    window.focus(&fh, cx);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let here = order.iter().position(|s| *s == focused).unwrap_or(0);
                    let commit = |time: Time, window: &mut Window, cx: &mut App| {
                        let complete = state.update(cx, |s, cx| {
                            let complete = s.edit_focused(time, &order);
                            cx.notify();
                            complete
                        });
                        if complete {
                            if let Some(cb) = &on_change {
                                cb(Some(time), window, cx);
                            }
                        }
                    };
                    match key {
                        "left" | "right" => {
                            let delta: i32 = if key == "right" { 1 } else { -1 };
                            let next =
                                (here as i32 + delta).clamp(0, order.len() as i32 - 1) as usize;
                            buffer.update(cx, |b, _| b.clear());
                            let segment = order[next];
                            state.update(cx, |s, cx| {
                                s.focused = segment;
                                cx.notify();
                            });
                        }
                        _ if is_read_only => {}
                        "up" | "down" => {
                            let delta = if key == "up" { 1 } else { -1 };
                            buffer.update(cx, |b, _| b.clear());
                            let base = state.read(cx).display_value.unwrap_or(seed);
                            let time = clamp_time(base.bump(focused, delta), min_value, max_value);
                            commit(time, window, cx);
                        }
                        "backspace" | "delete" => {
                            buffer.update(cx, |b, _| b.clear());
                            let emptied = state.update(cx, |s, cx| {
                                let emptied = s.clear_focused(&order);
                                cx.notify();
                                emptied
                            });
                            if emptied {
                                if let Some(cb) = &on_change {
                                    cb(None, window, cx);
                                }
                            }
                        }
                        // The meridiem segment answers `a` and `p`, the way
                        // React Aria's does.
                        "a" | "p" if focused == TimeSegment::Meridiem => {
                            let base = state.read(cx).display_value.unwrap_or(seed);
                            let hour = base.hour % 12 + if key == "p" { 12 } else { 0 };
                            commit(
                                Time::new(hour, base.minute).with_second(base.second),
                                window,
                                cx,
                            );
                        }
                        digit if digit.len() == 1 && digit.chars().all(|c| c.is_ascii_digit()) => {
                            let width = focused.digits();
                            if width == 0 {
                                return;
                            }
                            let text = buffer.update(cx, |b, _| {
                                if b.len() >= width {
                                    b.clear();
                                }
                                b.push_str(digit);
                                b.clone()
                            });
                            let Ok(value) = text.parse::<u32>() else {
                                return;
                            };
                            let base = state.read(cx).display_value.unwrap_or(seed);
                            commit(
                                focused.with_value(
                                    base,
                                    value,
                                    twelve_hour,
                                    zero_based_twelve_hour,
                                ),
                                window,
                                cx,
                            );
                            if text.len() >= width {
                                buffer.update(cx, |b, _| b.clear());
                                if let Some(segment) = order.get(here + 1).copied() {
                                    state.update(cx, |s, cx| {
                                        s.focused = segment;
                                        cx.notify();
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                });
        }
        if self.is_disabled {
            group = group.opacity(layout.disabled_opacity);
        }
        if self.full_width {
            group = group.w_full();
        }

        // `.date-input-group__prefix` is `ms-3 me-0`; the group's own padding
        // already provides the outer inset.
        if let Some(prefix) = self.prefix.take() {
            group = group.child(
                div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .mr(px(4.))
                    .text_color(colors.field.placeholder)
                    .child(prefix),
            );
        }

        for (index, segment) in segments.iter().copied().enumerate() {
            if let Some(literal) = regional_time
                .literals
                .get(index)
                .filter(|literal| !literal.is_empty())
            {
                group = group.child(div().text_color(colors.muted).child(literal.clone()));
            }

            let mut seg = div()
                .id(ElementId::Name(
                    format!("time-{entity_id}-seg-{index}").into(),
                ))
                // `.date-input-group__segment` is `rounded-md px-0.5`.
                .px(px(2.))
                .py(px(1.))
                .rounded(cx.layout().radius_md())
                // `segment` is v3's render prop on `TimeField.Segment`: the
                // closure is handed which segment it is drawing.
                .child(match &self.segment {
                    Some(render) => render(segment, segment_text(segment).into()),
                    None => segment_text(segment).into_any_element(),
                });

            if focused == segment && navigable {
                seg = seg
                    .bg(colors.accent.soft())
                    .text_color(colors.accent.soft_foreground(colors.foreground));
            }

            if navigable {
                let state = self.state.clone();
                seg = seg.cursor_pointer().on_click(move |_, _, cx| {
                    state.update(cx, |s, cx| {
                        s.focused = segment;
                        cx.notify();
                    });
                });
            }

            group = group.child(seg);
        }
        if let Some(literal) = regional_time
            .literals
            .last()
            .filter(|literal| !literal.is_empty())
        {
            group = group.child(div().text_color(colors.muted).child(literal.clone()));
        }

        // Steppers adjust whichever segment is focused.
        if editable {
            let seed = self.placeholder_value.unwrap_or(Time::new(0, 0));
            let min_value = self.min_value;
            let max_value = self.max_value;
            let visible_segments = segments;
            let mut steppers = div().flex().flex_col().ml(px(8.)).flex_shrink_0();
            for (icon, delta, key) in [
                (icons::CHEVRON_UP, 1i32, "up"),
                (icons::CHEVRON_DOWN, -1i32, "down"),
            ] {
                let state = self.state.clone();
                let on_change = self.on_change.clone();
                let visible_segments = visible_segments.clone();
                let hover_bg = colors.default.color;
                steppers = steppers.child(
                    div()
                        .id(ElementId::Name(format!("time-{entity_id}-{key}").into()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(18.))
                        .h(px(14.))
                        .rounded(px(4.))
                        .cursor_pointer()
                        .text_color(colors.muted)
                        .hover(move |s| s.bg(hover_bg))
                        .child(
                            gpui::svg()
                                .size(px(10.))
                                .path(icon)
                                .text_color(colors.muted),
                        )
                        .on_click(move |_, window, cx| {
                            let (next, complete) = state.update(cx, |s, cx| {
                                let base = s.display_value.unwrap_or(seed);
                                let mut next = base.bump(s.focused, delta);
                                // `minValue`/`maxValue` clamp the result.
                                next = clamp_time(next, min_value, max_value);
                                let complete = s.edit_focused(next, &visible_segments);
                                cx.notify();
                                (next, complete)
                            });
                            if complete {
                                if let Some(cb) = &on_change {
                                    cb(Some(next), window, cx);
                                }
                            }
                        }),
                );
            }
            group = group.child(steppers);
        }

        if let Some(suffix) = self.suffix.take() {
            group = group.child(
                div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .ml(px(4.))
                    .text_color(colors.field.placeholder)
                    .child(suffix),
            );
        }

        // `.date-field` is `flex flex-col gap-1`.
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .when(self.full_width, |root| root.w_full());
        if let Some(label) = self.label {
            root = root.child(
                crate::field::Label::new(label)
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(is_invalid),
            );
        }
        root = root.child(group);

        if is_invalid {
            if let Some(message) = validity.first() {
                root = root.child(crate::field::ErrorMessage::new(message));
            }
        } else if let Some(description) = self.description {
            root = root.child(crate::field::Description::new(description));
        }

        root.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hour_wraps_at_midnight() {
        assert_eq!(Time::new(23, 0).bump(TimeSegment::Hour, 1).hour, 0);
        assert_eq!(Time::new(0, 0).bump(TimeSegment::Hour, -1).hour, 23);
    }

    #[test]
    fn minute_wraps_without_touching_the_hour() {
        let t = Time::new(10, 59).bump(TimeSegment::Minute, 1);
        assert_eq!((t.hour, t.minute), (10, 0));
    }

    #[test]
    fn meridiem_flips_half_a_day() {
        assert_eq!(Time::new(9, 30).bump(TimeSegment::Meridiem, 1).hour, 21);
        assert_eq!(Time::new(21, 30).bump(TimeSegment::Meridiem, 1).hour, 9);
    }

    #[test]
    fn twelve_hour_display() {
        assert_eq!(Time::new(0, 0).twelve_hour(), (12, "AM"));
        assert_eq!(Time::new(12, 0).twelve_hour(), (12, "PM"));
        assert_eq!(Time::new(13, 0).twelve_hour(), (1, "PM"));
        assert_eq!(Time::new(11, 0).twelve_hour(), (11, "AM"));
    }

    #[test]
    fn zero_based_twelve_hour_entry_preserves_the_half_day() {
        assert_eq!(
            TimeSegment::Hour
                .with_value(Time::new(1, 0), 0, true, true)
                .hour,
            0
        );
        assert_eq!(
            TimeSegment::Hour
                .with_value(Time::new(13, 0), 0, true, true)
                .hour,
            12
        );
        assert_eq!(
            TimeSegment::Hour
                .with_value(Time::new(13, 0), 12, true, false)
                .hour,
            12
        );
    }

    #[test]
    fn hour_cycle_follows_locale_time_data_and_overrides() {
        assert_eq!(
            RegionalTimeFormat::for_locale("en-US").map(|format| format.hour_cycle),
            Some(HourCycle::H12)
        );
        assert_eq!(
            RegionalTimeFormat::for_locale("de-DE").map(|format| format.hour_cycle),
            Some(HourCycle::H24)
        );
        assert_eq!(
            RegionalTimeFormat::for_locale("en-US-u-hc-h23").map(|format| format.hour_cycle),
            Some(HourCycle::H24)
        );
        assert_eq!(
            RegionalTimeFormat::for_locale("de-DE-u-hc-h12").map(|format| format.hour_cycle),
            Some(HourCycle::H12)
        );
        assert_eq!(RegionalTimeFormat::for_locale("not_a_locale"), None);
    }

    #[test]
    fn hour_cycle_prefers_the_system_time_category() {
        let locale = locale_config::Locale::new("de-DE,time=en-US").unwrap();
        assert_eq!(
            RegionalTimeFormat::for_preferences(&locale).map(|format| format.hour_cycle),
            Some(HourCycle::H12)
        );
    }

    #[test]
    fn hour_padding_follows_locale_time_patterns() {
        assert_eq!(
            RegionalTimeFormat::for_locale("en-US")
                .map(|format| format.minute_pattern.hour_has_leading_zero),
            Some(false)
        );
        assert_eq!(
            RegionalTimeFormat::for_locale("de-DE")
                .map(|format| format.minute_pattern.hour_has_leading_zero),
            Some(true)
        );
        assert_eq!(
            RegionalTimeFormat::for_locale("en-US-u-hc-h23")
                .map(|format| format.minute_pattern.hour_has_leading_zero),
            Some(true)
        );
        let explicit_twelve =
            RegionalTimeFormat::for_locale_with_cycle("en-US-u-hc-h23", Some(HourCycle::H12))
                .unwrap();
        assert_eq!(explicit_twelve.hour_cycle, HourCycle::H12);
        assert!(!explicit_twelve.minute_pattern.hour_has_leading_zero);
    }

    #[test]
    fn segment_order_literals_padding_and_day_periods_follow_locale_patterns() {
        let us = RegionalTimeFormat::for_locale_with_cycle("en-US", Some(HourCycle::H12))
            .unwrap()
            .pattern(TimeGranularity::Second);
        assert_eq!(
            us.order,
            [
                TimeSegment::Hour,
                TimeSegment::Minute,
                TimeSegment::Second,
                TimeSegment::Meridiem,
            ]
        );
        assert_eq!(us.literals, ["", ":", ":", "\u{202f}", ""]);
        assert!(!us.hour_has_leading_zero);
        assert!(!us.hour_zero_based);
        assert!(us.minute_has_leading_zero);
        assert!(us.second_has_leading_zero);
        assert_eq!((us.am.as_str(), us.pm.as_str()), ("AM", "PM"));
        assert_eq!(us.hint(), "HH:MM:SS\u{202f}AM");

        let japanese = RegionalTimeFormat::for_locale_with_cycle("ja-JP", Some(HourCycle::H12))
            .unwrap()
            .pattern(TimeGranularity::Minute);
        assert_eq!(
            japanese.order,
            [
                TimeSegment::Meridiem,
                TimeSegment::Hour,
                TimeSegment::Minute,
            ]
        );
        assert_eq!(japanese.literals, ["", "", ":", ""]);
        assert!(japanese.hour_zero_based);
        assert_eq!(
            (japanese.am.as_str(), japanese.pm.as_str()),
            ("午前", "午後")
        );
        assert_eq!(japanese.hint(), "午前HH:MM");

        let arabic = RegionalTimeFormat::for_locale_with_cycle("ar-EG", Some(HourCycle::H12))
            .unwrap()
            .pattern(TimeGranularity::Minute);
        assert_eq!((arabic.am.as_str(), arabic.pm.as_str()), ("ص", "م"));

        let korean = RegionalTimeFormat::for_locale_with_cycle("ko-KR", Some(HourCycle::H24))
            .unwrap()
            .pattern(TimeGranularity::Second);
        assert_eq!(
            korean.order,
            [TimeSegment::Hour, TimeSegment::Minute, TimeSegment::Second,]
        );
        assert_eq!(korean.literals, ["", ":", ":", ""]);
        assert!(korean.hour_has_leading_zero);
        assert!(korean.minute_has_leading_zero);
        assert!(korean.second_has_leading_zero);
        assert_eq!(korean.hint(), "HH:MM:SS");
    }

    #[test]
    fn hour_granularity_keeps_day_period_order_without_minute_punctuation() {
        let us = RegionalTimeFormat::for_locale_with_cycle("en-US", Some(HourCycle::H12))
            .unwrap()
            .pattern(TimeGranularity::Hour);
        assert_eq!(us.order, [TimeSegment::Hour, TimeSegment::Meridiem]);
        assert_eq!(us.literals, ["", "\u{202f}", ""]);

        let japanese = RegionalTimeFormat::for_locale_with_cycle("ja-JP", Some(HourCycle::H12))
            .unwrap()
            .pattern(TimeGranularity::Hour);
        assert_eq!(japanese.order, [TimeSegment::Meridiem, TimeSegment::Hour]);
        assert_eq!(japanese.literals, ["", "", ""]);
    }

    #[test]
    fn constructors_clamp() {
        assert_eq!(Time::new(99, 99).hour, 23);
        assert_eq!(Time::new(99, 99).minute, 59);
        assert_eq!(Time::new(1, 1).with_second(99).second, 59);
    }
}

/// Clamps `t` into `[min, max]` by total seconds; either bound may be absent.
fn clamp_time(t: Time, min: Option<Time>, max: Option<Time>) -> Time {
    let secs = |x: Time| x.hour * 3600 + x.minute * 60 + x.second;
    let mut out = t;
    if let Some(lo) = min {
        if secs(out) < secs(lo) {
            out = lo;
        }
    }
    if let Some(hi) = max {
        if secs(out) > secs(hi) {
            out = hi;
        }
    }
    out
}

#[cfg(test)]
mod clamp_tests {
    use super::*;

    #[test]
    fn no_bounds_is_identity() {
        let t = Time::new(9, 30);
        assert_eq!(clamp_time(t, None, None), t);
    }

    #[test]
    fn clamps_up_to_min() {
        let min = Time::new(9, 0);
        assert_eq!(clamp_time(Time::new(7, 30), Some(min), None), min);
        // Already inside the range.
        assert_eq!(
            clamp_time(Time::new(10, 0), Some(min), None),
            Time::new(10, 0)
        );
    }

    #[test]
    fn clamps_down_to_max() {
        let max = Time::new(17, 0);
        assert_eq!(clamp_time(Time::new(23, 30), None, Some(max)), max);
        assert_eq!(
            clamp_time(Time::new(12, 0), None, Some(max)),
            Time::new(12, 0)
        );
    }

    #[test]
    fn bounds_are_inclusive() {
        let min = Time::new(9, 0);
        let max = Time::new(17, 0);
        assert_eq!(clamp_time(min, Some(min), Some(max)), min);
        assert_eq!(clamp_time(max, Some(min), Some(max)), max);
    }

    #[test]
    fn compares_by_total_seconds_not_by_field() {
        // 09:59 is below 10:00 even though its minute is larger.
        let min = Time::new(10, 0);
        assert_eq!(clamp_time(Time::new(9, 59), Some(min), None), min);
    }

    #[test]
    fn seconds_participate_in_the_comparison() {
        let min = Time::new(9, 0).with_second(30);
        assert_eq!(
            clamp_time(Time::new(9, 0).with_second(10), Some(min), None),
            min
        );
        let inside = Time::new(9, 0).with_second(45);
        assert_eq!(clamp_time(inside, Some(min), None), inside);
    }
}
