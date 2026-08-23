//! Input & InputState — port of `@heroui/input`.
//!
//! `InputState` is an entity holding the editable value; [`Input`] is the
//! styled element bound to it (controlled like HeroUI's controlled inputs).

use gpui::{
    prelude::*, px, App, Entity, FocusHandle, Focusable, IntoElement, KeyDownEvent, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

/// Editable state of a single-line text input.
pub struct InputState {
    value: String,
    /// Cursor position in char indices (the focused end of the selection).
    cursor: usize,
    /// Selection anchor in char indices; `None` when the caret is collapsed.
    anchor: Option<usize>,
    pub(crate) focus_handle: FocusHandle,
    /// `name` — what this field submits under, written in by the component's
    /// `name` builder. The state carries it because gpui gives a child no way
    /// to reach its `Form`; `FormField::text` reads it back out.
    name: Option<SharedString>,
    /// `validationBehavior` — travels with the name, for the same reason.
    validation_behavior: crate::form::ValidationBehavior,
}

impl InputState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            anchor: None,
            focus_handle: cx.focus_handle(),
            name: None,
            validation_behavior: crate::form::ValidationBehavior::Native,
        }
    }

    /// `defaultValue` — a state seeded with initial text, with the caret at the
    /// end.
    ///
    /// This is the uncontrolled entry point: React needs a `defaultValue` prop
    /// because it has no state object to seed.
    pub fn with_value(cx: &mut App, value: impl Into<String>) -> Self {
        let mut state = Self::new(cx);
        state.set_value(value);
        state
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// The `name` this field submits under, if one was set.
    pub fn name(&self) -> Option<SharedString> {
        self.name.clone()
    }

    /// Sets the submission name. Called by the component's `name` builder, not
    /// usually by hand.
    pub fn set_name(&mut self, name: Option<SharedString>) {
        self.name = name;
    }

    /// Whether this field's invalidity blocks form submission.
    pub fn validation_behavior(&self) -> crate::form::ValidationBehavior {
        self.validation_behavior
    }

    /// Set by the component's `validation_behavior` builder.
    pub fn set_validation_behavior(&mut self, behavior: crate::form::ValidationBehavior) {
        self.validation_behavior = behavior;
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Normalized `(start, end)` char range of the active selection.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        let c = self.cursor;
        if a == c {
            None
        } else {
            Some((a.min(c), a.max(c)))
        }
    }
}

impl Focusable for InputState {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// -- char-index editing helpers -------------------------------------------

/// Checks a whole value against the v3 field constraints.
#[allow(clippy::too_many_arguments)]
fn validate_value(
    value: &str,
    input_type: InputType,
    min_length: Option<usize>,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    pattern: Option<&dyn Fn(&str) -> bool>,
) -> InputValidity {
    // An empty field is "not yet filled in", not invalid; `is_required` is the
    // prop that speaks to emptiness.
    if value.is_empty() {
        return InputValidity::Valid;
    }

    if let Some(f) = pattern {
        if !f(value) {
            return InputValidity::PatternMismatch;
        }
    }

    if min_length.is_some_and(|n| value.chars().count() < n) {
        return InputValidity::TooShort;
    }

    // The numeric bounds only mean anything for a numeric field with a
    // parsable value.
    if input_type == InputType::Number {
        if let Ok(n) = value.parse::<f64>() {
            if min.is_some_and(|lo| n < lo) {
                return InputValidity::BelowMin;
            }
            if max.is_some_and(|hi| n > hi) {
                return InputValidity::AboveMax;
            }
            if let Some(step) = step.filter(|s| *s > 0.0) {
                let base = min.unwrap_or(0.0);
                let steps = (n - base) / step;
                if (steps - steps.round()).abs() > 1e-6 {
                    return InputValidity::OffStep;
                }
            }
        }
    }

    InputValidity::Valid
}

/// Whether `ch` may be inserted, honouring `type` and `maxLength`.
///
/// Takes the measured lengths rather than the whole state so it stays a pure
/// function (an `InputState` needs an `App` for its focus handle).
fn accepts_char(
    len_chars: usize,
    selected_chars: usize,
    ch: char,
    input_type: InputType,
    max_length: Option<usize>,
) -> bool {
    if !input_type.accepts(ch) {
        return false;
    }
    match max_length {
        // A selection is replaced by the keystroke, so it frees up room.
        Some(max) => len_chars.saturating_sub(selected_chars) < max,
        None => true,
    }
}

/// [`accepts_char`] for a live state.
fn state_accepts(
    state: &InputState,
    ch: char,
    input_type: InputType,
    max_length: Option<usize>,
) -> bool {
    let selected = state.selection().map_or(0, |(lo, hi)| hi - lo);
    accepts_char(
        state.value.chars().count(),
        selected,
        ch,
        input_type,
        max_length,
    )
}

fn insert_char(state: &mut InputState, ch: char) {
    delete_selection(state);
    let byte_idx = char_to_byte(&state.value, state.cursor);
    state.value.insert(byte_idx, ch);
    state.cursor += 1;
}

/// Removes the active selection (if any); returns true when it did.
fn delete_selection(state: &mut InputState) -> bool {
    if let Some((lo, hi)) = state.selection() {
        let lo_b = char_to_byte(&state.value, lo);
        let hi_b = char_to_byte(&state.value, hi);
        state.value.replace_range(lo_b..hi_b, "");
        state.cursor = lo;
        state.anchor = None;
        true
    } else {
        false
    }
}

fn backspace(state: &mut InputState) {
    if delete_selection(state) {
        return;
    }
    if state.cursor == 0 {
        return;
    }
    let byte_idx = char_to_byte(&state.value, state.cursor);
    let prev = char_to_byte(&state.value, state.cursor - 1);
    state.value.replace_range(prev..byte_idx, "");
    state.cursor -= 1;
}

fn delete(state: &mut InputState) {
    if delete_selection(state) {
        return;
    }
    let len = state.value.chars().count();
    if state.cursor >= len {
        return;
    }
    let byte_idx = char_to_byte(&state.value, state.cursor);
    let next = char_to_byte(&state.value, state.cursor + 1);
    state.value.replace_range(byte_idx..next, "");
}

fn move_left(state: &mut InputState, extend: bool) {
    if !extend {
        state.anchor = None;
    } else if state.anchor.is_none() {
        state.anchor = Some(state.cursor);
    }
    state.cursor = state.cursor.saturating_sub(1);
}

fn move_right(state: &mut InputState, extend: bool) {
    if !extend {
        state.anchor = None;
    } else if state.anchor.is_none() {
        state.anchor = Some(state.cursor);
    }
    if state.cursor < state.value.chars().count() {
        state.cursor += 1;
    }
}

fn move_home(state: &mut InputState, extend: bool) {
    if !extend {
        state.anchor = None;
    } else if state.anchor.is_none() {
        state.anchor = Some(state.cursor);
    }
    state.cursor = 0;
}

fn move_end(state: &mut InputState, extend: bool) {
    if !extend {
        state.anchor = None;
    } else if state.anchor.is_none() {
        state.anchor = Some(state.cursor);
    }
    state.cursor = state.value.chars().count();
}

fn select_all(state: &mut InputState) {
    state.anchor = Some(0);
    state.cursor = state.value.chars().count();
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map_or(s.len(), |(b, _)| b)
}

/// A validation outcome for the current value.
///
/// v3 leaves validation to React Aria; here the caller reads this and passes
/// the result to `is_invalid` / `error_message`, so the rules stay declarative
/// without us owning a validation lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputValidity {
    Valid,
    /// Shorter than `minLength`.
    TooShort,
    /// Below `min` (numeric types).
    BelowMin,
    /// Above `max` (numeric types).
    AboveMax,
    /// Not a multiple of `step` from `min`.
    OffStep,
    /// Does not satisfy `pattern`.
    PatternMismatch,
}

impl InputValidity {
    pub fn is_valid(self) -> bool {
        matches!(self, InputValidity::Valid)
    }
}

/// The `type` attribute. Only the variants that change rendering or input
/// handling in gpui are modelled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputType {
    #[default]
    Text,
    /// Masks the value with bullets.
    Password,
    Email,
    /// Restricts typing to digits, `-` and `.`.
    Number,
    Tel,
    Url,
    Search,
}

impl InputType {
    pub const ALL: [InputType; 7] = [
        InputType::Text,
        InputType::Password,
        InputType::Email,
        InputType::Number,
        InputType::Tel,
        InputType::Url,
        InputType::Search,
    ];

    pub fn label(self) -> &'static str {
        match self {
            InputType::Text => "Text",
            InputType::Password => "Password",
            InputType::Email => "Email",
            InputType::Number => "Number",
            InputType::Tel => "Tel",
            InputType::Url => "Url",
            InputType::Search => "Search",
        }
    }

    fn masks(self) -> bool {
        matches!(self, InputType::Password)
    }

    /// Whether `ch` may be typed into a field of this type.
    fn accepts(self, ch: char) -> bool {
        match self {
            InputType::Number => ch.is_ascii_digit() || ch == '-' || ch == '.',
            _ => true,
        }
    }
}

type TextCallback = std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>;

/// The character Enter inserts in a multi-line field.
const NEWLINE: char = '\n';

/// Everything the multi-line body needs to draw itself.
struct MultilineBody<'a> {
    value: &'a str,
    cursor: usize,
    selection: Option<(usize, usize)>,
    focused: bool,
    /// The caret's element id, height and colour.
    caret: (gpui::ElementId, gpui::Pixels, gpui::Hsla),
    selection_bg: gpui::Hsla,
}

/// One paragraph per newline, each wrapping, with the caret and selection
/// placed inside the paragraph they fall in.
///
/// gpui does wrap text — the default `WhiteSpace::Normal` — so a real
/// multi-line surface only needed the newlines split out and the caret located
/// within them. The single-line field opts out with `whitespace_nowrap`.
fn multiline_body(b: MultilineBody<'_>, cx: &App) -> gpui::AnyElement {
    let (caret_id, caret_h, caret_color) = b.caret;
    let mut col = gpui::div().flex().flex_col().items_start().w_full();
    // Char offset each line starts at, so the cursor and selection — which are
    // offsets into the whole value — can be mapped into it.
    let mut start = 0usize;
    let lines: Vec<&str> = b.value.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    for (i, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        let end = start + len;
        // `min_w_0` on the text spans is what lets each wrap inside the field
        // instead of pushing the row wider than its box.
        let mut para = gpui::div()
            .flex()
            .flex_wrap()
            .items_center()
            .w_full()
            .min_w_0();

        // The selection, clipped to this line.
        let local_sel = b.selection.and_then(|(lo, hi)| {
            let lo = lo.max(start);
            let hi = hi.min(end);
            (lo < hi).then_some((lo - start, hi - start))
        });

        if let Some((lo, hi)) = local_sel {
            let before: String = line.chars().take(lo).collect();
            let selected: String = line.chars().skip(lo).take(hi - lo).collect();
            let after: String = line.chars().skip(hi).collect();
            para = para
                .child(gpui::div().min_w_0().child(before))
                .child(
                    gpui::div()
                        .min_w_0()
                        .px(px(1.))
                        .rounded(px(4.))
                        .bg(b.selection_bg)
                        .child(selected),
                )
                .child(gpui::div().min_w_0().child(after));
        } else if b.selection.is_none() && b.cursor >= start && b.cursor <= end {
            let at = b.cursor - start;
            let before: String = line.chars().take(at).collect();
            let after: String = line.chars().skip(at).collect();
            para = para.child(gpui::div().min_w_0().child(before));
            if b.focused {
                para = para.child(crate::anim::caret_blink(
                    gpui::div()
                        .w(px(1.5))
                        .h(caret_h)
                        .bg(caret_color)
                        .flex_shrink_0(),
                    caret_id.clone(),
                    cx,
                ));
            }
            para = para.child(gpui::div().min_w_0().child(after));
        } else {
            para = para.child(gpui::div().w_full().min_w_0().child(line.to_string()));
        }

        col = col.child(para);
        // +1 for the newline the split consumed, except after the last line.
        start = end + usize::from(i < last);
    }
    col.into_any_element()
}

/// HeroUI Input.
#[derive(IntoElement)]
pub struct Input {
    /// `validationBehavior` — written into the state on render.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    state: Entity<InputState>,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<str>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    variant: FieldVariant,
    input_type: InputType,
    max_length: Option<usize>,
    min_length: Option<usize>,
    /// `min` / `max` / `step` for the numeric types.
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    /// `pattern` — a predicate over the whole value.
    pattern: Option<std::sync::Arc<dyn Fn(&str) -> bool + 'static>>,
    start_content: Option<gpui::AnyElement>,
    end_content: Option<gpui::AnyElement>,
    /// Stretch beyond the 320px default demo width.
    /// Multi-line only: the height `rows` asks for. `None` leaves v3's
    /// `min-height: 38px`.
    min_h: Option<gpui::Pixels>,
    /// Set by [`crate::input_group::InputGroup`]: `(has_prefix, has_suffix)`.
    /// `InputGroup.Input` has no chrome of its own -- the group paints it -- and
    /// drops the padding on whichever side touches an addon (`ps-0`/`pe-0`).
    in_group: Option<(bool, bool)>,
    full_width: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    is_invalid: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `name` — the submission name, written into the state on render.
    name: Option<SharedString>,
    /// Set by `TextArea`: wrap the text, lay lines out top-down, and let Enter
    /// insert a newline instead of submitting.
    multiline: bool,
    /// `defaultValue` — seeds the state on the first render only.
    default_value: Option<SharedString>,
    is_clearable: bool,
    on_change: Option<TextCallback>,
    on_submit: Option<TextCallback>,
}

impl Input {
    /// The bound state, so wrappers can write through to it.
    pub fn state(&self) -> &Entity<InputState> {
        &self.state
    }

    /// `value` — writes through to the bound [`InputState`].
    pub fn value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_value(value));
        self
    }

    pub fn new(state: Entity<InputState>) -> Self {
        Self {
            validation_behavior: None,
            state,
            label: None,
            placeholder: None,
            description: None,
            error_message: None,
            validate: None,
            validation_errors: Vec::new(),
            variant: FieldVariant::Primary,
            input_type: InputType::Text,
            max_length: None,
            min_length: None,
            min: None,
            max: None,
            step: None,
            pattern: None,
            start_content: None,
            end_content: None,
            min_h: None,
            in_group: None,
            full_width: false,
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            is_invalid: false,
            auto_focus: false,
            name: None,
            default_value: None,
            multiline: false,
            is_clearable: false,
            on_change: None,
            on_submit: None,
        }
    }

    /// `validationBehavior` — `Allow` shows the message without blocking form
    /// submission.
    ///
    /// Stored on the state beside `name`, because the form reads both from
    /// there.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// Turns this into the multi-line surface `TextArea` renders.
    ///
    /// Not a v3 prop: v3 has a separate `<textarea>`, and this is the flag that
    /// switches one implementation between the two.
    pub(crate) fn multiline(mut self, v: bool) -> Self {
        self.multiline = v;
        self
    }

    /// Multi-line only: the height `TextArea::rows` asks for.
    pub(crate) fn min_h(mut self, h: gpui::Pixels) -> Self {
        self.min_h = Some(h);
        self
    }

    /// Renders as v3's `InputGroup.Input`: transparent, unrounded, unshadowed,
    /// and flush against whichever addons surround it.
    pub(crate) fn in_group(mut self, has_prefix: bool, has_suffix: bool) -> Self {
        self.in_group = Some((has_prefix, has_suffix));
        self
    }

    /// `name` — the name this field submits under.
    ///
    /// Stored on the [`InputState`], because gpui gives a child no way to reach
    /// its `Form`; `FormField::text(state)` reads it back out, so the name is
    /// written once here and not repeated at the form.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `defaultValue` — the uncontrolled initial text.
    ///
    /// Written into the state on the first render only, so it seeds the field
    /// without overwriting what the user types. `InputState::with_value` does
    /// the same at construction; this is the prop spelling, for a state the
    /// caller made without one.
    pub fn default_value(mut self, text: impl Into<SharedString>) -> Self {
        self.default_value = Some(text.into());
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
        self
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `validate` — returns the message to show, or `None` when the text is fine.
    ///
    /// The component runs it and surfaces the result, so a caller does not have
    /// to mirror the logic into `is_invalid` / `error_message`.
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
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

    pub fn error_message(mut self, e: impl Into<SharedString>) -> Self {
        self.error_message = Some(e.into());
        self
    }

    /// The `type` attribute — `password` masks, `number` filters keystrokes.
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    /// `maxLength` — refuses keystrokes past this many characters.
    pub fn max_length(mut self, n: usize) -> Self {
        self.max_length = Some(n);
        self
    }

    /// `minLength` — reported by [`Input::validity`], not enforced while typing.
    pub fn min_length(mut self, n: usize) -> Self {
        self.min_length = Some(n);
        self
    }

    /// `min` — the lowest accepted value for the numeric types.
    pub fn min(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }

    /// `max` — the highest accepted value for the numeric types.
    pub fn max(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }

    /// `step` — the numeric granularity, measured from `min` (or 0).
    pub fn step(mut self, v: f64) -> Self {
        self.step = Some(v);
        self
    }

    /// `pattern` — a predicate the whole value must satisfy.
    pub fn pattern(mut self, f: impl Fn(&str) -> bool + 'static) -> Self {
        self.pattern = Some(std::sync::Arc::new(f));
        self
    }

    /// Checks `value` against this field's constraints.
    ///
    /// Callers own their validation lifecycle, so this is a pure query — pass
    /// the outcome back through `is_invalid` / `error_message`.
    pub fn validity(&self, value: &str) -> InputValidity {
        validate_value(
            value,
            self.input_type,
            self.min_length,
            self.min,
            self.max,
            self.step,
            self.pattern.as_deref(),
        )
    }

    pub fn variant(mut self, v: FieldVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn start_content(mut self, el: impl IntoElement) -> Self {
        self.start_content = Some(el.into_any_element());
        self
    }

    pub fn end_content(mut self, el: impl IntoElement) -> Self {
        self.end_content = Some(el.into_any_element());
        self
    }

    /// Fills the parent instead of the 320px default demo width.
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
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

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// Shows a clear button when there is a value.
    pub fn is_clearable(mut self, v: bool) -> Self {
        self.is_clearable = v;
        self
    }

    pub fn on_change(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_submit(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(std::sync::Arc::new(f));
        self
    }
}

impl Input {
    /// The focus handle of the state this field is bound to.
    ///
    /// `InputGroup` rings on `focus-within`, and the only thing inside it that
    /// can hold a focus is this field.
    pub(crate) fn state_focus(&self, cx: &App) -> FocusHandle {
        self.state.read(cx).focus_handle.clone()
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `validationBehavior` travels with the name, on the state.
        if let Some(behavior) = self.validation_behavior {
            let state = self.state.clone();
            if state.read(cx).validation_behavior() != behavior {
                state.update(cx, |s, _| s.set_validation_behavior(behavior));
            }
        }
        // `focus_once` takes `cx` mutably, so it has to run before the theme
        // tokens are borrowed.
        // `defaultValue` seeds the state once, before anything reads it.
        if let Some(text) = self.default_value.clone() {
            let state = self.state.clone();
            crate::util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("input-default-{}", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(text.to_string());
                        cx.notify();
                    });
                },
            );
        }
        // `name` lives on the state so the form can find it. Only write when
        // it differs, so this does not loop through `notify`.
        if self.state.read(cx).name() != self.name {
            let name = self.name.clone();
            self.state.update(cx, |s, _| s.set_name(name));
        }
        let focus_handle = self.state.read(cx).focus_handle.clone();
        if self.auto_focus {
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("input-autofocus-{}", self.state.entity_id().as_u64()).into(),
                ),
                &focus_handle,
            );
        }
        let disabled_opacity = cx.layout().disabled_opacity;
        let colors = cx.colors();
        let accent = colors.accent;
        let focused = focus_handle.is_focused(window);

        let (h, text) = (px(40.), px(14.));

        // v3 order: the controlled flag, then server errors, then `validate`,
        // with `errorMessage` as the fallback.
        let value_now = self.state.read(cx).value().to_owned();
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&value_now)),
            self.error_message.clone(),
        );
        let is_invalid = validity.is_invalid;
        let _border_color = if is_invalid {
            colors.danger.color
        } else if focused {
            colors.focus
        } else {
            colors.border
        };
        let multiline = self.multiline;
        // `.textarea` is `py-2` over a `min-height: 38px`; `rows` raises
        // that floor. Without this the field ignored `rows`, so every
        // TextArea came out one line tall inside a taller wrapper.
        let multiline_h = self.min_h.unwrap_or(px(38.));
        let mut field = gpui::div()
            .id(gpui::ElementId::Name(
                format!("input-{}", self.state.entity_id().as_u64()).into(),
            ))
            .flex()
            // Multi-line: the text starts at the top, the box grows downward
            // with the content, and the width is fixed so lines can wrap.
            .map(|f| {
                if multiline {
                    f.items_start()
                        .min_h(multiline_h)
                        .py(px(8.))
                        .w_full()
                        .overflow_hidden()
                } else {
                    f.items_center().h(h)
                }
            })
            .gap(px(8.))
            // `.input-group__input` keeps `px-3` except on a side that touches
            // an addon, which carries the padding instead.
            .map(|f| match self.in_group {
                None => f.px(px(12.)),
                Some((prefix, suffix)) => f
                    .flex_1()
                    .pl(if prefix { px(0.) } else { px(12.) })
                    .pr(if suffix { px(0.) } else { px(12.) }),
            })
            .text_size(text)
            .rounded(crate::util::field_radius(cx))
            .when(!self.is_disabled, |e| {
                e.cursor(gpui::CursorStyle::IBeam)
                    .track_focus(&focus_handle)
                    .key_context("Input")
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let fh = focus_handle.clone();
                        move |_, window, _| window.focus(&fh)
                    })
            })
            // `status-disabled` is `--disabled-opacity`, which the theme owns.
            .when(self.is_disabled, |e| e.opacity(disabled_opacity));

        // Inside an `InputGroup` the group is the field: v3's
        // `.input-group__input` is `rounded-none border-0 bg-transparent
        // shadow-none`.
        if self.in_group.is_none() {
            field = crate::util::apply_field_chrome(field, self.variant, is_invalid, focused, cx);
        }

        // -- text content -----------------------------------------------------
        let st = self.state.read(cx);
        // `password` renders bullets while keeping the real value in state, so
        // cursor and selection maths stay in char units either way.
        let value = if self.input_type.masks() {
            "\u{2022}".repeat(st.value.chars().count())
        } else {
            st.value.clone()
        };
        let cursor = st.cursor;
        let is_empty = value.is_empty();
        let selection = st.selection();

        // A multi-line field lays its lines out top-down and lets each one
        // wrap; a single-line one is a centred, non-wrapping row.
        let mut row = if self.multiline {
            gpui::div()
                .flex()
                .flex_col()
                .items_start()
                .w_full()
                .min_w_0()
                .flex_1()
        } else {
            gpui::div().flex().items_center().min_w_0().flex_1()
        };

        if self.multiline && !(is_empty && !focused && self.placeholder.is_some()) {
            // One wrapping paragraph per newline, with the caret and any
            // selection placed inside the line they fall in.
            let caret_id =
                gpui::ElementId::Name(format!("caret-{}", self.state.entity_id().as_u64()).into());
            row = row.child(multiline_body(
                MultilineBody {
                    value: &value,
                    cursor,
                    selection,
                    focused,
                    caret: (caret_id, text * 1.3, accent.color),
                    selection_bg: accent.with_alpha(0.24),
                },
                cx,
            ));
        } else if is_empty && !focused && self.placeholder.is_some() {
            row = row.child(
                gpui::div()
                    .text_color(colors.muted)
                    .truncate()
                    .child(self.placeholder.clone().unwrap().to_string()),
            );
        } else if let Some((lo, hi)) = selection {
            // Selected range — three spans.
            let before: String = value.chars().take(lo).collect();
            let selected: String = value.chars().skip(lo).take(hi - lo).collect();
            let after: String = value.chars().skip(hi).collect();
            let sel_bg = accent.with_alpha(0.24);
            row = row.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(before)
                    .child(
                        gpui::div()
                            .px(px(1.))
                            .py(px(1.))
                            .rounded(px(4.))
                            .bg(sel_bg)
                            .child(selected),
                    )
                    .child(after),
            );
        } else {
            let before: String = value.chars().take(cursor).collect();
            let after: String = value.chars().skip(cursor).collect();
            row = row.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(before)
                    .when(focused, |r| {
                        // v3's `@keyframes caret-blink`.
                        r.child(crate::anim::caret_blink(
                            gpui::div()
                                .w(px(1.5))
                                .h(text * 1.3)
                                .bg(accent.color)
                                .flex_shrink_0(),
                            gpui::ElementId::Name(
                                format!("caret-{}", self.state.entity_id().as_u64()).into(),
                            ),
                            cx,
                        ))
                    })
                    .child(after),
            );
        }

        field = field.children(self.start_content);
        field = field.child(row);
        // isClearable — show X when has value and not disabled/readonly
        if self.is_clearable && !is_empty && !self.is_disabled && !self.is_read_only {
            let clear_state = self.state.clone();
            let clear_on_change = self.on_change.clone();
            field = field.child(
                gpui::div()
                    .id(gpui::ElementId::Name(
                        format!("input-clear-{}", self.state.entity_id().as_u64()).into(),
                    ))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(20.))
                    .rounded_full()
                    .cursor_pointer()
                    .text_color(colors.default.color)
                    .hover(|s| s.bg(colors.default.with_alpha(0.15)))
                    .on_click(move |_, window, cx| {
                        clear_state.update(cx, |s, cx| {
                            s.value.clear();
                            s.cursor = 0;
                            s.anchor = None;
                            cx.notify();
                        });
                        if let Some(cb) = &clear_on_change {
                            cb("", window, cx);
                        }
                    })
                    .child("×"),
            );
        }
        field = field.children(self.end_content);

        // -- editing ----------------------------------------------------------
        let state_entity = self.state.clone();
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();
        let is_read_only = self.is_read_only;
        let is_disabled = self.is_disabled;
        field = field.on_key_down(move |ev: &KeyDownEvent, window, cx| {
            if is_disabled {
                return;
            }
            let key: &str = &ev.keystroke.key;
            let mods = ev.keystroke.modifiers;
            let input_type = self.input_type;
            let max_length = self.max_length;
            let multiline = self.multiline;
            let mut changed = false;
            let mut submit = false;

            if key == "a" && (mods.control || mods.platform) {
                // Ctrl+A / Cmd+A : select all
                if !mods.alt {
                    state_entity.update(cx, |s, cx| {
                        select_all(s);
                        cx.notify();
                    });
                }
            } else if mods.control || mods.alt || mods.platform {
                if !is_read_only && (mods.control || mods.platform) && key == "v" {
                    if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
                        state_entity.update(cx, |s, cx| {
                            for ch in text.chars() {
                                if state_accepts(s, ch, input_type, max_length) {
                                    insert_char(s, ch);
                                }
                            }
                            cx.notify();
                        });
                        changed = true;
                    }
                }
            } else {
                let shift = mods.shift;
                match key {
                    "backspace" => {
                        if is_read_only {
                            return;
                        }
                        state_entity.update(cx, |s, cx| {
                            backspace(s);
                            cx.notify();
                        });
                        changed = true;
                    }
                    "delete" => {
                        if is_read_only {
                            return;
                        }
                        state_entity.update(cx, |s, cx| {
                            delete(s);
                            cx.notify();
                        });
                        changed = true;
                    }
                    "left" => state_entity.update(cx, |s, cx| {
                        move_left(s, shift);
                        cx.notify();
                    }),
                    "right" => state_entity.update(cx, |s, cx| {
                        move_right(s, shift);
                        cx.notify();
                    }),
                    "home" => state_entity.update(cx, |s, cx| {
                        move_home(s, shift);
                        cx.notify();
                    }),
                    "end" => state_entity.update(cx, |s, cx| {
                        move_end(s, shift);
                        cx.notify();
                    }),
                    // A multi-line field takes Enter as a newline, the way a
                    // `<textarea>` does; a single-line one submits.
                    "enter" if multiline => {
                        if is_read_only {
                            return;
                        }
                        state_entity.update(cx, |s, cx| {
                            if state_accepts(s, NEWLINE, input_type, max_length) {
                                insert_char(s, NEWLINE);
                            }
                            cx.notify();
                        });
                        changed = true;
                    }
                    "enter" => submit = true,
                    "space" => {
                        if is_read_only {
                            return;
                        }
                        state_entity.update(cx, |s, cx| {
                            if state_accepts(s, ' ', input_type, max_length) {
                                insert_char(s, ' ');
                            }
                            cx.notify();
                        });
                        changed = true;
                    }
                    "escape" => {
                        // Clear selection or dismiss.
                        state_entity.update(cx, |s, cx| {
                            s.anchor = None;
                            cx.notify();
                        });
                    }
                    single if single.chars().count() == 1 && !single.is_empty() => {
                        if is_read_only {
                            return;
                        }
                        let mut c = single.chars().next().unwrap();
                        if c.is_ascii_alphabetic() && !mods.shift {
                            c = c.to_ascii_lowercase();
                        }
                        state_entity.update(cx, |s, cx| {
                            if state_accepts(s, c, input_type, max_length) {
                                insert_char(s, c);
                            }
                            cx.notify();
                        });
                        changed = true;
                    }
                    _ => {}
                }
            }

            if changed {
                if let Some(cb) = &on_change {
                    {
                        let v = state_entity.read(cx).value().to_owned();
                        cb(&v, window, cx);
                    }
                }
            }
            if submit {
                if let Some(cb) = &on_submit {
                    {
                        let v = state_entity.read(cx).value().to_owned();
                        cb(&v, window, cx);
                    }
                }
            }
        });

        // Inside a group the surrounding component owns the label, the
        // description and the error slot -- v3's `InputGroup.Input` is the input
        // and nothing else. Returning the wrapper here would drop a whole
        // labelled column into the group's row.
        if self.in_group.is_some() {
            return field.into_any_element();
        }

        // -- wrapper with label / description / error --------------------------
        let mut el = gpui::div().flex().flex_col().gap(px(4.));
        if self.full_width {
            el = el.w_full();
        } else {
            el = el.max_w(px(320.));
        }
        if let Some(label) = self.label {
            let mut label_row = gpui::div()
                .flex()
                .items_center()
                .gap(px(4.))
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(colors.foreground)
                .child(label.to_string());
            if self.is_required {
                label_row = label_row.child(
                    gpui::div()
                        .text_color(colors.danger.color)
                        .child("*".to_owned()),
                );
            }
            el = el.child(label_row);
        }
        el = el.child(field);
        if let Some(err) = validity.first() {
            el = el.child(
                gpui::div()
                    .text_size(px(12.))
                    .text_color(colors.danger.color)
                    .child(err.to_string()),
            );
        } else if let Some(desc) = self.description {
            el = el.child(
                gpui::div()
                    .text_size(px(12.))
                    .text_color(colors.muted)
                    .child(desc.to_string()),
            );
        }

        el.into_any_element()
    }
}

// ---------------------------------------------------------------------------
// TextField / SearchField
// ---------------------------------------------------------------------------

/// TextField — port of `@heroui/text-field` (v3).
///
/// The composition-friendly field: a label, an [`Input`], and a description or
/// validation message. `Input` is the bare control; `TextField` is the labelled
/// wrapper most applications reach for.
#[derive(IntoElement)]
pub struct TextField {
    inner: Input,
}

impl TextField {
    pub fn new(state: Entity<InputState>) -> Self {
        Self {
            inner: Input::new(state),
        }
    }

    /// `type` — the HTML input type, which v3 sets on the field and this port
    /// forwards to the inner [`Input`].
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.inner = self.inner.input_type(input_type);
        self
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(text);
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.placeholder(text);
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.description(text);
        self
    }

    /// `autoFocus` — see [`Input::auto_focus`].
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.inner = self.inner.auto_focus(v);
        self
    }

    /// `name` — see [`Input::name`].
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.inner = self.inner.name(name);
        self
    }

    /// `defaultValue` — see [`Input::default_value`].
    pub fn default_value(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.default_value(text);
        self
    }

    /// `validationBehavior` — see [`crate::input::Input::validation_behavior`].
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.inner = self.inner.validation_behavior(behavior);
        self
    }

    /// `validate` — see [`Input::validate`].
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.inner = self.inner.validate(f);
        self
    }

    /// `validationErrors` — see [`Input::validation_errors`].
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.inner = self.inner.validation_errors(errors);
        self
    }

    pub fn error_message(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.error_message(text);
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.inner = self.inner.variant(variant);
        self
    }

    pub fn full_width(mut self) -> Self {
        self.inner = self.inner.full_width();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.inner = self.inner.is_disabled(v);
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.inner = self.inner.is_read_only(v);
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.inner = self.inner.is_required(v);
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.inner = self.inner.is_invalid(v);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_submit(handler);
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.inner.render(window, cx)
    }
}

/// SearchField — port of `@heroui/search-field` (v3).
///
/// An [`Input`] specialised for search: a leading magnifier icon and a clear
/// button that appears once there is a value. `onSubmit` fires on Enter and
/// `onClear` when the value is cleared.
#[derive(IntoElement)]
pub struct SearchField {
    state: Entity<InputState>,
    /// `name` — the submission name, forwarded to the inner `Input`.
    name: Option<SharedString>,
    /// `defaultValue` — forwarded to the inner `Input`.
    default_value: Option<SharedString>,
    /// `validationBehavior` — forwarded to the inner `Input`.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    label: Option<SharedString>,
    placeholder: SharedString,
    description: Option<SharedString>,
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    is_invalid: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<str>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<SharedString>,
    on_change: Option<TextCallback>,
    on_submit: Option<TextCallback>,
    on_clear: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    /// `SearchField.SearchIcon` — the leading glyph. `None` draws the magnifier.
    search_icon: Option<gpui::AnyElement>,
    /// Trailing content inside the field, before the clear button. v3 composes
    /// it (a `Kbd` with the shortcut, in its "With Keyboard Shortcut" example).
    end_content: Option<gpui::AnyElement>,
}

impl SearchField {
    pub fn new(state: Entity<InputState>) -> Self {
        Self {
            state,
            name: None,
            default_value: None,
            validation_behavior: None,
            label: None,
            placeholder: "Search".into(),
            description: None,
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            is_invalid: false,
            auto_focus: false,
            validate: None,
            validation_errors: Vec::new(),
            on_change: None,
            on_submit: None,
            on_clear: None,
            search_icon: None,
            end_content: None,
        }
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(handler));
        self
    }

    pub fn on_submit(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(std::sync::Arc::new(handler));
        self
    }

    /// `name` — see [`Input::name`].
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// `defaultValue` — see [`Input::default_value`].
    pub fn default_value(mut self, text: impl Into<SharedString>) -> Self {
        self.default_value = Some(text.into());
        self
    }

    /// `SearchField.SearchIcon` — replaces the leading magnifier.
    pub fn search_icon(mut self, el: impl IntoElement) -> Self {
        self.search_icon = Some(el.into_any_element());
        self
    }

    /// Trailing content inside the field — v3 composes a `Kbd` here.
    pub fn end_content(mut self, el: impl IntoElement) -> Self {
        self.end_content = Some(el.into_any_element());
        self
    }

    /// `validationBehavior` — see [`Input::validation_behavior`].
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// `validate` — see [`Input::validate`].
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — see [`Input::validation_errors`].
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    /// `autoFocus` — take focus on the first render.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `isReadOnly` — the field shows its value but cannot be edited.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `isRequired` — marks the label.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `isInvalid` — applies the danger treatment.
    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// Called after the clear button empties the field.
    pub fn on_clear(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for SearchField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let validate = self.validate.clone();
        let mut input = Input::new(self.state)
            .placeholder(self.placeholder)
            .when_some(self.name, |i, n| i.name(n))
            .when_some(self.default_value, |i, v| i.default_value(v))
            .when_some(self.validation_behavior, |i, b| i.validation_behavior(b))
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_read_only(self.is_read_only)
            .is_required(self.is_required)
            .is_invalid(self.is_invalid)
            .validation_errors(self.validation_errors.clone())
            .auto_focus(self.auto_focus)
            .when_some(validate, |i, f| i.validate(move |v| f(v)))
            .is_clearable(true)
            .start_content(match self.search_icon {
                Some(icon) => icon,
                None => gpui::svg()
                    .size(crate::util::FIELD_ICON)
                    .path(crate::icons::SEARCH)
                    .flex_shrink_0()
                    .text_color(colors.muted)
                    .into_any_element(),
            });
        if let Some(end) = self.end_content {
            input = input.end_content(end);
        }

        if self.full_width {
            input = input.full_width();
        }
        if let Some(label) = self.label {
            input = input.label(label);
        }
        if let Some(description) = self.description {
            input = input.description(description);
        }

        // `isClearable` empties the state and reports "" through `on_change`;
        // `on_clear` is the dedicated notification for that transition.
        let on_clear = self.on_clear.clone();
        let on_change = self.on_change.clone();
        input = input.on_change(move |text, window, cx| {
            if let Some(cb) = &on_change {
                cb(text, window, cx);
            }
            if text.is_empty() {
                if let Some(cb) = &on_clear {
                    cb(window, cx);
                }
            }
        });

        if let Some(on_submit) = self.on_submit {
            input = input.on_submit(move |text, window, cx| on_submit(text, window, cx));
        }

        input.render(window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(
        value: &str,
        ty: InputType,
        min_length: Option<usize>,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    ) -> InputValidity {
        validate_value(value, ty, min_length, min, max, step, None)
    }

    #[test]
    fn empty_is_valid_regardless_of_bounds() {
        // Emptiness is `is_required`'s business, not the bounds'.
        assert_eq!(
            check(
                "",
                InputType::Number,
                Some(5),
                Some(10.0),
                Some(20.0),
                Some(2.0)
            ),
            InputValidity::Valid
        );
    }

    #[test]
    fn min_length_counts_chars_not_bytes() {
        assert_eq!(
            check("ab", InputType::Text, Some(3), None, None, None),
            InputValidity::TooShort
        );
        assert_eq!(
            check("abc", InputType::Text, Some(3), None, None, None),
            InputValidity::Valid
        );
        // Two-byte chars still count as one each.
        assert_eq!(
            check("éé", InputType::Text, Some(3), None, None, None),
            InputValidity::TooShort
        );
    }

    #[test]
    fn numeric_bounds_are_inclusive() {
        let n = InputType::Number;
        assert_eq!(
            check("9", n, None, Some(10.0), None, None),
            InputValidity::BelowMin
        );
        assert_eq!(
            check("10", n, None, Some(10.0), None, None),
            InputValidity::Valid
        );
        assert_eq!(
            check("20", n, None, None, Some(20.0), None),
            InputValidity::Valid
        );
        assert_eq!(
            check("21", n, None, None, Some(20.0), None),
            InputValidity::AboveMax
        );
    }

    #[test]
    fn bounds_are_ignored_for_non_numeric_types() {
        // A text field holding "5" is not subject to `min`.
        assert_eq!(
            check("5", InputType::Text, None, Some(10.0), None, None),
            InputValidity::Valid
        );
    }

    #[test]
    fn unparsable_numeric_input_is_not_a_bounds_error() {
        assert_eq!(
            check("-", InputType::Number, None, Some(0.0), Some(9.0), None),
            InputValidity::Valid
        );
    }

    #[test]
    fn step_is_measured_from_min() {
        let n = InputType::Number;
        // Steps of 3 from 1: 1, 4, 7 ...
        assert_eq!(
            check("4", n, None, Some(1.0), None, Some(3.0)),
            InputValidity::Valid
        );
        assert_eq!(
            check("5", n, None, Some(1.0), None, Some(3.0)),
            InputValidity::OffStep
        );
        // With no min, the base is 0.
        assert_eq!(
            check("6", n, None, None, None, Some(3.0)),
            InputValidity::Valid
        );
        assert_eq!(
            check("5", n, None, None, None, Some(3.0)),
            InputValidity::OffStep
        );
    }

    #[test]
    fn fractional_steps_tolerate_float_error() {
        assert_eq!(
            check("0.3", InputType::Number, None, None, None, Some(0.1)),
            InputValidity::Valid
        );
    }

    #[test]
    fn zero_step_is_ignored_rather_than_dividing_by_zero() {
        assert_eq!(
            check("5", InputType::Number, None, None, None, Some(0.0)),
            InputValidity::Valid
        );
    }

    #[test]
    fn pattern_is_checked_before_the_other_rules() {
        let deny = |_: &str| false;
        assert_eq!(
            validate_value(
                "ab",
                InputType::Text,
                Some(10),
                None,
                None,
                None,
                Some(&deny)
            ),
            InputValidity::PatternMismatch
        );
        let allow = |v: &str| v.starts_with('a');
        assert_eq!(
            validate_value("abc", InputType::Text, None, None, None, None, Some(&allow)),
            InputValidity::Valid
        );
    }

    #[test]
    fn validity_helper_reports_valid() {
        assert!(InputValidity::Valid.is_valid());
        assert!(!InputValidity::TooShort.is_valid());
    }

    #[test]
    fn number_type_rejects_letters() {
        assert!(accepts_char(2, 0, '3', InputType::Number, None));
        assert!(accepts_char(2, 0, '-', InputType::Number, None));
        assert!(accepts_char(2, 0, '.', InputType::Number, None));
        assert!(!accepts_char(2, 0, 'a', InputType::Number, None));
    }

    #[test]
    fn text_type_accepts_anything() {
        assert!(accepts_char(2, 0, 'c', InputType::Text, None));
        assert!(accepts_char(2, 0, '!', InputType::Text, None));
    }

    #[test]
    fn no_max_length_never_blocks() {
        assert!(accepts_char(9999, 0, 'x', InputType::Text, None));
    }

    #[test]
    fn max_length_blocks_at_the_limit() {
        assert!(accepts_char(2, 0, 'c', InputType::Text, Some(3)));
        assert!(!accepts_char(3, 0, 'd', InputType::Text, Some(3)));
        // Already over the limit (e.g. set_value bypassed the gate).
        assert!(!accepts_char(4, 0, 'd', InputType::Text, Some(3)));
    }

    #[test]
    fn max_length_counts_a_replaced_selection_as_free() {
        // 3 chars, all selected: the keystroke replaces them, so it fits.
        assert!(accepts_char(3, 3, 'd', InputType::Text, Some(3)));
        // Only one selected: 2 survive, still room for one more.
        assert!(accepts_char(3, 1, 'd', InputType::Text, Some(3)));
    }

    #[test]
    fn type_filter_wins_over_available_room() {
        assert!(!accepts_char(0, 0, 'a', InputType::Number, Some(10)));
    }

    #[test]
    fn password_masking_preserves_char_count() {
        let value = "sécret";
        let masked = "•".repeat(value.chars().count());
        assert_eq!(masked.chars().count(), value.chars().count());
    }
}
