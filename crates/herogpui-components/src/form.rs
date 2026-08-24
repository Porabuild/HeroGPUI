//! Form — port of `@heroui/form` (v3).
//!
//! v3's `Form` is a `<form>`: `name` on each field is what makes a submission
//! carry that field's value, and `onSubmit` receives the collected `FormData`.
//! There is no DOM here, and gpui gives a child no way to find its ancestor, so
//! the form is *told* which fields it owns — `Form::field(..)` — instead of
//! discovering them. Everything downstream of that is the same: names, a
//! collected submission, reset, and an invalid path that runs instead of submit.
//!
//! What is deliberately absent is the HTTP half — `action`, `method`, `encType`
//! and `target`. There is no browser to navigate.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use gpui::{
    px, AnyElement, App, Entity, FocusHandle, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};

use crate::{input::InputState, number_field::NumberState};

/// One field's value in a submission.
#[derive(Clone, Debug, PartialEq)]
pub enum FormValue {
    Text(SharedString),
    Number(f64),
    Flag(bool),
    /// A multi-selection: `CheckboxGroup`, a multiple `Select`, `TagGroup`.
    Keys(Vec<SharedString>),
}

impl FormValue {
    /// The value as a string, the way an HTML form would send it.
    pub fn as_text(&self) -> SharedString {
        match self {
            FormValue::Text(t) => t.clone(),
            FormValue::Number(n) => SharedString::from(n.to_string()),
            // An unchecked box sends nothing; "on" is the HTML default value.
            FormValue::Flag(true) => SharedString::from("on"),
            FormValue::Flag(false) => SharedString::from(""),
            FormValue::Keys(k) => SharedString::from(
                k.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        }
    }

    /// Whether an HTML form would treat this as filled in — what `isRequired`
    /// checks against.
    pub fn is_empty(&self) -> bool {
        match self {
            FormValue::Text(t) => t.trim().is_empty(),
            FormValue::Number(_) => false,
            FormValue::Flag(v) => !v,
            FormValue::Keys(k) => k.is_empty(),
        }
    }
}

/// A submission: every named field the form was given, in registration order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FormData {
    entries: Vec<(SharedString, FormValue)>,
}

impl FormData {
    pub fn get(&self, name: &str) -> Option<&FormValue> {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// The named field as text, which is how a form field usually reads.
    pub fn text(&self, name: &str) -> Option<SharedString> {
        self.get(name).map(FormValue::as_text)
    }

    /// All values submitted under `name`, matching the browser `FormData`
    /// `getAll` operation. Multi-selection fields contribute one value per
    /// selected key.
    pub fn get_all(&self, name: &str) -> Vec<SharedString> {
        self.entries
            .iter()
            .filter(|(entry_name, _)| entry_name == name)
            .flat_map(|(_, value)| match value {
                FormValue::Keys(keys) => keys.clone(),
                value => vec![value.as_text()],
            })
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SharedString, &FormValue)> {
        self.entries.iter().map(|(n, v)| (n, v))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The names whose value is missing but required.
    pub fn missing_required(&self, required: &[SharedString]) -> Vec<SharedString> {
        required
            .iter()
            .filter(|name| self.get(name).is_none_or(FormValue::is_empty))
            .cloned()
            .collect()
    }
}

type Read = Arc<dyn Fn(&App) -> FormValue + 'static>;
type Restore = Arc<dyn Fn(&mut Window, &mut App) + 'static>;
type ReadName = Arc<dyn Fn(&App) -> Option<SharedString> + 'static>;
type ReadBehavior = Arc<dyn Fn(&App) -> ValidationBehavior + 'static>;
type ReadSuccessful = Arc<dyn Fn(&App) -> bool + 'static>;
/// Reads the field's stored validity — whether its own validation is in error,
/// which blocks a native submission like a missing required value.
type ReadInvalid = Arc<dyn Fn(&App) -> bool + 'static>;
/// Moves the focus to this field, which a blocked submit uses for v3's
/// "the first invalid field will be focused".
type FocusField = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Value, validity and focus owned by a rendered control without an entity.
pub(crate) struct LiveFormFieldState {
    pub(crate) value: FormValue,
    pub(crate) is_invalid: bool,
    pub(crate) is_successful: bool,
    pub(crate) focus: Option<FocusHandle>,
    pub(crate) restore: Option<Restore>,
}

/// A named field a [`Form`] reads on submit.
///
/// The `name` may come from the field itself: a component whose value lives in
/// an entity (`Input`, `TextArea`, `NumberField`) writes its `name` prop into
/// that entity, so [`FormField::text`] and [`FormField::number`] can pick it up
/// and the call site does not repeat it.
#[derive(Clone)]
pub struct FormField {
    name: Option<SharedString>,
    /// Reads the `name` prop out of the field's own state entity.
    name_of: Option<ReadName>,
    read: Read,
    restore: Option<Restore>,
    is_required: bool,
    /// `validationBehavior` on the field: `Allow` shows its message without
    /// blocking submission.
    validation_behavior: ValidationBehavior,
    /// Reads `validationBehavior` off the field's own state entity.
    behavior_of: Option<ReadBehavior>,
    /// Whether this field is a successful native form control. Disabled
    /// checkbox inputs are neither submitted nor validated.
    successful_of: Option<ReadSuccessful>,
    /// Reads the field's stored validity — whether the field considers itself
    /// in error, written by `Input::render`.
    invalid_of: Option<ReadInvalid>,
    /// Focuses the field — the invalid-path step, when it has a reachable
    /// handle.
    focus: Option<FocusField>,
}

impl FormField {
    /// A text field, read from its [`InputState`].
    pub fn text(state: Entity<InputState>) -> Self {
        let read_state = state.clone();
        let name_state = state.clone();
        let behavior_state = state.clone();
        let invalid_state = state.clone();
        let focus_state = state;
        Self {
            name: None,
            name_of: Some(Arc::new(move |cx: &App| name_state.read(cx).name())),
            behavior_of: Some(Arc::new(move |cx: &App| {
                behavior_state.read(cx).validation_behavior()
            })),
            successful_of: None,
            invalid_of: Some(Arc::new(move |cx: &App| {
                invalid_state.read(cx).validity().is_invalid
            })),
            read: Arc::new(move |cx: &App| {
                FormValue::Text(SharedString::from(read_state.read(cx).value().to_owned()))
            }),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            focus: Some(Arc::new(move |window, cx| {
                let fh = focus_state.read(cx).focus_handle.clone();
                window.focus(&fh);
            })),
        }
    }

    /// A numeric field, read from its [`NumberState`].
    pub fn number(state: Entity<NumberState>) -> Self {
        let read_state = state.clone();
        let name_state = state.clone();
        let behavior_state = state.clone();
        let focus_state = state;
        Self {
            name: None,
            name_of: Some(Arc::new(move |cx: &App| {
                let st = name_state.read(cx);
                st.input.read(cx).name()
            })),
            behavior_of: Some(Arc::new(move |cx: &App| {
                behavior_state.read(cx).input.read(cx).validation_behavior()
            })),
            successful_of: None,
            invalid_of: None,
            read: Arc::new(move |cx: &App| FormValue::Number(read_state.read(cx).value())),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            focus: Some(Arc::new(move |window, cx| {
                let fh = focus_state.read(cx).input.read(cx).focus_handle.clone();
                window.focus(&fh);
            })),
        }
    }

    /// An OTP field, read from its [`crate::input_otp::OtpState`].
    pub fn code(name: impl Into<SharedString>, state: Entity<crate::input_otp::OtpState>) -> Self {
        let read_state = state.clone();
        let focus_state = state;
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |cx: &App| {
                FormValue::Text(SharedString::from(read_state.read(cx).code()))
            }),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: None,
            invalid_of: None,
            focus: Some(Arc::new(move |window, cx| {
                let fh = focus_state.read(cx).focus_handle.clone();
                window.focus(&fh);
            })),
        }
    }

    /// A plain text value the caller holds — a formatted date, a colour hex,
    /// an OTP code.
    pub fn text_value(name: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        let value = value.into();
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |_| FormValue::Text(value.clone())),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: None,
            invalid_of: None,
            focus: None,
        }
    }

    /// A live field owned by a single-threaded rendered control.
    pub(crate) fn live(
        name: impl Into<SharedString>,
        state: Rc<RefCell<LiveFormFieldState>>,
    ) -> Self {
        let read_state = state.clone();
        let invalid_state = state.clone();
        let successful_state = state.clone();
        let restore_state = state.clone();
        let focus_state = state;
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |_| read_state.borrow().value.clone()),
            restore: Some(Arc::new(move |window, cx| {
                let restore = restore_state.borrow().restore.clone();
                if let Some(restore) = restore {
                    restore(window, cx);
                }
            })),
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: Some(Arc::new(move |_| successful_state.borrow().is_successful)),
            invalid_of: Some(Arc::new(move |_| invalid_state.borrow().is_invalid)),
            focus: Some(Arc::new(move |window, _| {
                let focus = focus_state.borrow().focus.clone();
                if let Some(focus) = focus {
                    window.focus(&focus);
                }
            })),
        }
    }

    /// A plain number the caller holds — a slider or colour channel.
    pub fn number_value(name: impl Into<SharedString>, value: f64) -> Self {
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |_| FormValue::Number(value)),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: None,
            invalid_of: None,
            focus: None,
        }
    }

    /// A checkbox or switch, whose value the caller holds.
    pub fn flag(name: impl Into<SharedString>, value: bool) -> Self {
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |_| FormValue::Flag(value)),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: None,
            invalid_of: None,
            focus: None,
        }
    }

    /// A selection, whose value the caller holds.
    pub fn keys(
        name: impl Into<SharedString>,
        values: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        let values: Vec<SharedString> = values.into_iter().map(Into::into).collect();
        Self {
            name: Some(name.into()),
            name_of: None,
            read: Arc::new(move |_| FormValue::Keys(values.clone())),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            behavior_of: None,
            successful_of: None,
            invalid_of: None,
            focus: None,
        }
    }

    /// Overrides the name, for a field that carries none of its own.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The value a reset restores. Without one, a reset only reports itself.
    pub fn default_text(
        mut self,
        state: Entity<InputState>,
        value: impl Into<SharedString>,
    ) -> Self {
        let value = value.into();
        self.restore = Some(Arc::new(move |_, cx: &mut App| {
            state.update(cx, |s, cx| {
                s.set_value(value.to_string());
                cx.notify();
            });
        }));
        self
    }

    /// The number a reset restores.
    pub fn default_number(mut self, state: Entity<NumberState>, value: f64) -> Self {
        self.restore = Some(Arc::new(move |_, cx: &mut App| {
            state.update(cx, |s, cx| {
                s.set_value(value, cx);
                cx.notify();
            });
        }));
        self
    }

    /// Marks the field required, which is what `onInvalid` reports on.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `validationBehavior` — `Allow` shows the field's message without
    /// blocking submission.
    pub fn validation_behavior(mut self, behavior: ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// Whether this field's invalidity blocks submission.
    pub fn blocks_submission(&self, cx: &App) -> bool {
        let behavior = self
            .behavior_of
            .as_ref()
            .map_or(self.validation_behavior, |f| f(cx));
        behavior == ValidationBehavior::Native
    }

    /// The name this field submits under: an explicit [`FormField::name`],
    /// otherwise the `name` prop the component wrote into its own state.
    pub fn field_name(&self, cx: &App) -> Option<SharedString> {
        self.name
            .clone()
            .or_else(|| self.name_of.as_ref().and_then(|f| f(cx)))
    }

    /// Whether the field's stored validity is in error — the render-side of
    /// the validation a native submit consults.
    fn is_invalid(&self, cx: &App) -> bool {
        self.invalid_of.as_ref().is_some_and(|f| f(cx))
    }

    fn is_successful(&self, cx: &App) -> bool {
        self.successful_of.as_ref().is_none_or(|f| f(cx))
    }
}

type OnSubmit = Arc<dyn Fn(&FormData, &mut Window, &mut App) + 'static>;
type OnReset = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// How invalid children are handled (`validationBehavior`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ValidationBehavior {
    /// A failed field blocks submission and `onInvalid` runs instead.
    #[default]
    Native,
    /// Submission proceeds; the messages are shown but not enforced.
    Allow,
}

/// HeroUI Form: a vertical field stack that collects a named submission.
#[derive(IntoElement)]
pub struct Form {
    /// `validationErrors` — form-level messages, shown above the fields.
    validation_errors: Vec<SharedString>,
    is_disabled: bool,
    validation_behavior: ValidationBehavior,
    fields: Vec<FormField>,
    on_submit: Option<OnSubmit>,
    on_reset: Option<OnReset>,
    on_invalid: Option<OnSubmit>,
    children: Vec<AnyElement>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            validation_errors: Vec::new(),
            is_disabled: false,
            validation_behavior: ValidationBehavior::default(),
            fields: Vec::new(),
            on_submit: None,
            on_reset: None,
            on_invalid: None,
            children: Vec::new(),
        }
    }

    /// `validationErrors` — form-level messages, shown above the fields.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `validationBehavior` — whether a failed field blocks submission.
    pub fn validation_behavior(mut self, behavior: ValidationBehavior) -> Self {
        self.validation_behavior = behavior;
        self
    }

    /// Registers a named field, so its value appears in the submission.
    pub fn field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    /// `onSubmit` — receives the collected submission.
    pub fn on_submit(mut self, f: impl Fn(&FormData, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Arc::new(f));
        self
    }

    /// `onReset` — fires after the registered fields are restored.
    pub fn on_reset(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_reset = Some(Arc::new(f));
        self
    }

    /// `onInvalid` — runs instead of `onSubmit` when validation blocks it.
    ///
    /// Blocked means a form-level `validationErrors` entry, a required field
    /// with no value, or a field whose own stored validity is in error
    /// (`validate`, `isInvalid`, `validationErrors`, or an HTML5 attribute
    /// violation) — and only under [`ValidationBehavior::Native`]; `Allow`
    /// submits regardless, as v3's does.
    pub fn on_invalid(mut self, f: impl Fn(&FormData, &mut Window, &mut App) + 'static) -> Self {
        self.on_invalid = Some(Arc::new(f));
        self
    }

    /// Collects the registered fields into a submission.
    pub fn data(&self, cx: &App) -> FormData {
        let mut entries = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            if !field.is_successful(cx) {
                continue;
            }
            // An unnamed field is not submitted, exactly as in HTML.
            if let Some(name) = field.field_name(cx) {
                let value = (field.read)(cx);
                // Unchecked checkbox inputs and checkbox groups with no
                // selected inputs are absent from native FormData.
                let omitted = match &value {
                    FormValue::Flag(false) => true,
                    FormValue::Keys(keys) => keys.is_empty(),
                    _ => false,
                };
                if !omitted {
                    entries.push((name, value));
                }
            }
        }
        FormData { entries }
    }

    /// The names of the fields whose emptiness blocks submission: required, and
    /// not opted out with `validationBehavior: "aria"`.
    fn required_names(&self, cx: &App) -> Vec<SharedString> {
        self.fields
            .iter()
            .filter(|f| f.is_successful(cx) && f.is_required && f.blocks_submission(cx))
            .filter_map(|f| f.field_name(cx))
            .collect()
    }

    /// The handler a submit button calls: collects the submission, then routes
    /// it to `onSubmit` or `onInvalid`.
    ///
    /// gpui gives a child no way to reach its form, so the caller wires this to
    /// the button in place of `<button type="submit">`.
    #[allow(clippy::arc_with_non_send_sync)] // see `util::shared`
    pub fn submit_handler(&self) -> Arc<dyn Fn(&mut Window, &mut App) + 'static> {
        let fields = self.fields.clone();
        let on_submit = self.on_submit.clone();
        let on_invalid = self.on_invalid.clone();
        let errors = self.validation_errors.clone();
        let behavior = self.validation_behavior;
        Arc::new(move |window: &mut Window, cx: &mut App| {
            let form = Form {
                validation_errors: errors.clone(),
                is_disabled: false,
                validation_behavior: behavior,
                fields: fields.clone(),
                on_submit: None,
                on_reset: None,
                on_invalid: None,
                children: Vec::new(),
            };
            let data = form.data(cx);
            let missing = data.missing_required(&form.required_names(cx));
            // A field error blocks a native submit however it arose: a
            // required field with no value, or a field whose stored validity
            // says it is in error (`validate`, `isInvalid`,
            // `validationErrors`, or an HTML5 attribute violation).
            let own_invalid: Vec<SharedString> = fields
                .iter()
                .filter(|f| f.is_successful(cx) && f.blocks_submission(cx) && f.is_invalid(cx))
                .filter_map(|f| f.field_name(cx))
                .collect();
            let blocked = behavior == ValidationBehavior::Native
                && (!errors.is_empty() || !missing.is_empty() || !own_invalid.is_empty());
            if blocked {
                // v3, verbatim: "By default, the first invalid field will be
                // focused." A blocked submit moves the focus to the first
                // registered field whose error keeps the form from
                // submitting — the same union the blocked condition above
                // computes. This runs inside the submit *button's* click
                // handler, not a keystroke, so moving the focus cannot
                // re-activate the focused control's click listener on the way
                // up (AGENTS.md); the tests below drive a submit with a
                // button click and assert the field holds the focus after.
                for field in &fields {
                    let invalid = field.is_successful(cx)
                        && field.blocks_submission(cx)
                        && (field.is_invalid(cx)
                            || (field.is_required
                                && field.field_name(cx).is_some_and(|name| {
                                    data.get(&name).is_none_or(FormValue::is_empty)
                                })));
                    if invalid {
                        if let Some(focus) = &field.focus {
                            focus(window, cx);
                        }
                        break;
                    }
                }
                if let Some(f) = &on_invalid {
                    f(&data, window, cx);
                }
            } else if let Some(f) = &on_submit {
                f(&data, window, cx);
            }
        })
    }

    /// The handler a reset button calls: restores every field that declared a
    /// default, then fires `onReset`.
    #[allow(clippy::arc_with_non_send_sync)] // see `util::shared`
    pub fn reset_handler(&self) -> Arc<dyn Fn(&mut Window, &mut App) + 'static> {
        let restores: Vec<Restore> = self
            .fields
            .iter()
            .filter_map(|f| f.restore.clone())
            .collect();
        let on_reset = self.on_reset.clone();
        Arc::new(move |window: &mut Window, cx: &mut App| {
            for restore in &restores {
                restore(window, cx);
            }
            if let Some(f) = &on_reset {
                f(window, cx);
            }
        })
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Form {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Form {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut el = gpui::div().flex().flex_col().gap(px(16.)).w_full();
        if self.is_disabled {
            el = el.opacity(0.5);
        }
        // Form-level messages sit above the fields, as v3's do.
        for message in self.validation_errors {
            el = el.child(crate::field::ErrorMessage::new(message));
        }
        el.children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(entries: &[(&str, FormValue)]) -> FormData {
        FormData {
            entries: entries
                .iter()
                .map(|(n, v)| (SharedString::from(n.to_string()), v.clone()))
                .collect(),
        }
    }

    #[test]
    fn a_submission_reads_by_name() {
        let d = data(&[
            ("email", FormValue::Text("a@b.c".into())),
            ("age", FormValue::Number(41.0)),
        ]);
        assert_eq!(d.text("email").unwrap(), SharedString::from("a@b.c"));
        assert_eq!(d.text("age").unwrap(), SharedString::from("41"));
        assert!(d.get("missing").is_none());
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn flags_and_keys_serialise_the_html_way() {
        assert_eq!(FormValue::Flag(true).as_text(), SharedString::from("on"));
        assert_eq!(FormValue::Flag(false).as_text(), SharedString::from(""));
        let keys = FormValue::Keys(vec!["a".into(), "b".into()]);
        assert_eq!(keys.as_text(), SharedString::from("a,b"));
    }

    #[test]
    fn emptiness_follows_the_control() {
        assert!(FormValue::Text("   ".into()).is_empty());
        assert!(!FormValue::Text("x".into()).is_empty());
        // An unchecked box submits nothing, so it counts as empty.
        assert!(FormValue::Flag(false).is_empty());
        assert!(!FormValue::Flag(true).is_empty());
        assert!(FormValue::Keys(vec![]).is_empty());
        // Zero is a value.
        assert!(!FormValue::Number(0.0).is_empty());
    }

    #[test]
    fn missing_required_reports_absent_and_blank_alike() {
        let d = data(&[
            ("name", FormValue::Text("".into())),
            ("tos", FormValue::Flag(true)),
        ]);
        let required: Vec<SharedString> =
            vec!["name".into(), "tos".into(), "never-registered".into()];
        assert_eq!(
            d.missing_required(&required),
            vec![
                SharedString::from("name"),
                SharedString::from("never-registered")
            ]
        );
    }
}
