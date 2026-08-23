//! TimeField — port of `@heroui/time-field` (v3).
//!
//! Segment-by-segment time entry. Clicking a segment focuses it; the stepper
//! buttons adjust whichever segment has focus.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, ElementId, Entity, InteractiveElement, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

use crate::{icons, util};

/// Whether a [`TimeField`] shows a 12- or 24-hour clock (`hourCycle`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HourCycle {
    H12,
    #[default]
    H24,
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
    pub(crate) fn with_value(self, time: Time, value: u32, twelve_hour: bool) -> Time {
        match self {
            TimeSegment::Hour => {
                let hour = if twelve_hour {
                    // 12-hour entry keeps the half of the day the field is in.
                    let pm = time.hour >= 12;
                    let base = value.clamp(1, 12) % 12;
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
    pub value: Option<Time>,
    pub focused: TimeSegment,
}

impl TimeState {
    pub fn new(_cx: &mut App) -> Self {
        Self {
            value: None,
            focused: TimeSegment::Hour,
        }
    }

    pub fn with_value(_cx: &mut App, value: Time) -> Self {
        Self {
            value: Some(value),
            focused: TimeSegment::Hour,
        }
    }

    /// Adjusts the focused segment, seeding an empty field from `seed`.
    pub fn bump_focused_from(&mut self, delta: i32, seed: Time) {
        let base = self.value.unwrap_or(seed);
        self.value = Some(base.bump(self.focused, delta));
    }

    /// [`bump_focused_from`](Self::bump_focused_from) seeded at midnight.
    pub fn bump_focused(&mut self, delta: i32) {
        self.bump_focused_from(delta, Time::new(0, 0));
    }
}

type Segment = Arc<dyn Fn(TimeSegment, SharedString) -> gpui::AnyElement + 'static>;

type OnTimeChange = Arc<dyn Fn(Option<Time>, &mut Window, &mut App) + 'static>;

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
    /// `TimeField.Suffix` — content after the segments.
    suffix: Option<gpui::AnyElement>,
    variant: FieldVariant,
    hour_cycle: HourCycle,
    /// `granularity` — the smallest unit shown.
    granularity: TimeGranularity,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `shouldForceLeadingZeros` — pad the hour, minute and second to two
    /// digits. On by default, which is what a time field usually wants.
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
    pub fn value(self, time: Option<Time>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.value = time);
        self
    }

    pub fn new(state: Entity<TimeState>) -> Self {
        Self {
            segment: None,
            name: None,
            validation_behavior: crate::form::ValidationBehavior::Native,
            default_value: None,
            state,
            label: None,
            description: None,
            error_message: None,
            prefix: None,
            suffix: None,
            variant: FieldVariant::Primary,
            hour_cycle: HourCycle::H24,
            granularity: TimeGranularity::default(),
            auto_focus: false,
            should_force_leading_zeros: true,
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
    /// The time is written `HH:MM`, which is what an HTML `<input type="time">`
    /// submits. Needs `cx` because the value lives in the state entity.
    pub fn form_field(&self, cx: &App) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        let text = self
            .state
            .read(cx)
            .value
            .map(|t| format!("{:02}:{:02}", t.hour, t.minute))
            .unwrap_or_default();
        Some(
            crate::form::FormField::text_value(name, text)
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

    /// `shouldForceLeadingZeros` — whether the segments are padded to two
    /// digits.
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
                        s.value = Some(value);
                        cx.notify();
                    });
                },
            );
        }

        let entity_id = self.state.entity_id().as_u64();
        // A time field has no inner `Input`, so it owns its focus handle. Keyed
        // state keeps it across frames; `use_keyed_state` takes `cx` mutably, so
        // it precedes the theme tokens.
        let focus_handle = window.use_keyed_state(
            ElementId::Name(format!("timefield-{entity_id}-focus").into()),
            cx,
            |_, cx| cx.focus_handle(),
        );
        let focus_handle = focus_handle.read(cx).clone();
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

        let colors = cx.colors();
        let layout = cx.layout();
        let interactive = !self.is_disabled && !self.is_read_only;

        let (value, focused) = {
            let st = self.state.read(cx);
            (st.value, st.focused)
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

        // The segments this field shows, in reading order -- `granularity`
        // decides where the list stops.
        let mut segments = vec![TimeSegment::Hour];
        if self.granularity != TimeGranularity::Hour {
            segments.push(TimeSegment::Minute);
        }
        if self.granularity == TimeGranularity::Second {
            segments.push(TimeSegment::Second);
        }
        if self.hour_cycle == HourCycle::H12 {
            segments.push(TimeSegment::Meridiem);
        }

        let hour_cycle = self.hour_cycle;
        let pad = self.should_force_leading_zeros;
        let segment_text = move |segment: TimeSegment| -> String {
            let Some(t) = value else {
                return "--".to_owned();
            };
            match segment {
                TimeSegment::Hour => {
                    if hour_cycle == HourCycle::H12 {
                        pad2(t.twelve_hour().0, pad)
                    } else {
                        pad2(t.hour, pad)
                    }
                }
                TimeSegment::Minute => pad2(t.minute, pad),
                TimeSegment::Second => pad2(t.second, pad),
                TimeSegment::Meridiem => t.twelve_hour().1.to_owned(),
            }
        };

        let mut group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.))
            .px(px(12.))
            .h(util::FIELD_HEIGHT)
            .rounded(util::field_radius(cx))
            .text_size(util::FIELD_TEXT)
            .font_family("Consolas")
            .text_color(colors.field.foreground);

        group = util::apply_field_chrome(group, self.variant, is_invalid, false, cx);

        // v3 drives a time field from the keyboard: the arrows step the focused
        // segment and walk between segments, and digits type into it.
        if interactive {
            let state = self.state.clone();
            let on_change = self.on_change.clone();
            let buffer = typing;
            let fh = focus_handle.clone();
            let order = TimeSegment::order(self.granularity, self.hour_cycle == HourCycle::H12);
            let twelve_hour = self.hour_cycle == HourCycle::H12;
            let seed = self.placeholder_value.unwrap_or(Time::new(9, 0));
            group = group
                .track_focus(&focus_handle)
                .key_context("TimeField")
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, _| {
                    window.focus(&fh);
                })
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let here = order.iter().position(|s| *s == focused).unwrap_or(0);
                    let commit = |time: Time, window: &mut Window, cx: &mut App| {
                        state.update(cx, |s, cx| {
                            s.value = Some(time);
                            cx.notify();
                        });
                        if let Some(cb) = &on_change {
                            cb(Some(time), window, cx);
                        }
                    };
                    match key {
                        "up" | "down" => {
                            let delta = if key == "up" { 1 } else { -1 };
                            buffer.update(cx, |b, _| b.clear());
                            state.update(cx, |s, cx| {
                                s.bump_focused_from(delta, seed);
                                cx.notify();
                            });
                            let next = state.read(cx).value;
                            if let (Some(cb), Some(time)) = (&on_change, next) {
                                cb(Some(time), window, cx);
                            }
                        }
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
                        "backspace" | "delete" => {
                            buffer.update(cx, |b, _| b.clear());
                            state.update(cx, |s, cx| {
                                s.value = None;
                                cx.notify();
                            });
                            if let Some(cb) = &on_change {
                                cb(None, window, cx);
                            }
                        }
                        // The meridiem segment answers `a` and `p`, the way
                        // React Aria's does.
                        "a" | "p" if focused == TimeSegment::Meridiem => {
                            let base = state.read(cx).value.unwrap_or(seed);
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
                            let base = state.read(cx).value.unwrap_or(seed);
                            commit(focused.with_value(base, value, twelve_hour), window, cx);
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
            // Colons separate the numeric segments; the meridiem gets a space.
            if segment == TimeSegment::Meridiem {
                group = group.child(div().w(px(4.)));
            } else if index > 0 {
                group = group.child(div().text_color(colors.muted).child(":"));
            }

            let mut seg = div()
                .id(ElementId::Name(
                    format!("time-{entity_id}-seg-{index}").into(),
                ))
                .px(px(4.))
                .py(px(1.))
                .rounded(px(4.))
                // `segment` is v3's render prop on `TimeField.Segment`: the
                // closure is handed which segment it is drawing.
                .child(match &self.segment {
                    Some(render) => render(segment, segment_text(segment).into()),
                    None => segment_text(segment).into_any_element(),
                });

            if focused == segment && interactive {
                seg = seg
                    .bg(colors.accent.soft())
                    .text_color(colors.accent.soft_foreground());
            }

            if interactive {
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

        // Steppers adjust whichever segment is focused.
        if interactive {
            let seed = self.placeholder_value.unwrap_or(Time::new(0, 0));
            let min_value = self.min_value;
            let max_value = self.max_value;
            let mut steppers = div().flex().flex_col().ml(px(8.)).flex_shrink_0();
            for (icon, delta, key) in [
                (icons::CHEVRON_UP, 1i32, "up"),
                (icons::CHEVRON_DOWN, -1i32, "down"),
            ] {
                let state = self.state.clone();
                let on_change = self.on_change.clone();
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
                            let next = state.update(cx, |s, cx| {
                                s.bump_focused_from(delta, seed);
                                // `minValue`/`maxValue` clamp the result.
                                if let Some(v) = s.value {
                                    s.value = Some(clamp_time(v, min_value, max_value));
                                }
                                cx.notify();
                                s.value
                            });
                            if let Some(cb) = &on_change {
                                cb(next, window, cx);
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

        let mut root = div().flex().flex_col().gap(px(6.));
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

        root
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
