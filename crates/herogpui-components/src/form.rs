//! Form — port of `@heroui/form` (v3).
//!
//! v3's `Form` is a `<form>`: `name` on each field is what makes a submission
//! carry that field's value, and `onSubmit` receives the collected `FormData`.
//! There is no DOM here, and gpui gives a child no way to find its ancestor, so
//! the form is *told* which fields it owns — `Form::field(..)` — instead of
//! discovering them. Everything downstream of that is the same: names, a
//! collected submission, reset, and an invalid path that runs instead of submit.
//!
//! Submission has two doors that share one implementation
//! ([`Form::run_submission`]): the caller-wired submit button
//! (`Form::submit_handler`, standing in for `<button type="submit">`) and the
//! native form's implicit submission — Enter pressed in a focused field that
//! semantically participates, which the form root's key handler answers. Only
//! the fields that participate carry the Enter reader; a TextArea's Enter is a
//! newline, and a focused submit button submits through its own click.
//!
//! The second door is a GPUI substitute for the browser's implicit submission,
//! not a port of it. A browser picks the *default submitter* — the first
//! submit button in tree order — and skips implicit submission entirely when a
//! form has no submit button and more than one field blocking validation; both
//! rules read the form's children, which are opaque elements here. A browser
//! also lets a field's own keydown handler cancel the keystroke; the controls
//! that own Enter here stop propagation to the same effect. So Enter in a
//! participating field always runs this form's one submission, whoever else
//! could have been the submitter.
//!
//! A blocked Enter-origin submission defers the focus move to the key's
//! release. gpui activates whichever element the frame drawn for the release
//! shows as focused, so focusing a Select trigger or a Switch mid-keystroke
//! would let the release click it — opening or toggling the very control the
//! repair was meant to reach. The form latches the blocked keystroke in keyed
//! state, disarms the release (`prevent_default` gates gpui's activation
//! listener), and moves the focus once the dispatch is over.
//!
//! Read-only controls are the third gate, after disabled and inert: they stay
//! successful (they submit their value) and focusable, but constraint
//! validation bars them — neither a missing value nor a stored error on a
//! read-only field can block a submission, as a native form has it.
//!
//! What is deliberately absent is the HTTP half — `action`, `method`, `encType`
//! and `target`. There is no browser to navigate.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use gpui::{
    px, AnyElement, App, Entity, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    KeyUpEvent, ParentElement, RenderOnce, SharedString, Styled, Window,
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
/// Whether a focused press of Enter submits the form from this field — the
/// reader half of a native form's implicit submission. Only the controls a
/// browser submits a form from carry one: the single-line text family (a
/// `<input type=number>` among them) and, as pinned v3 builds it, the OTP
/// row's single text input. A multi-line field and every non-text compound
/// control (switch, select, checkbox group) never do.
type SubmitsOnEnter = Arc<dyn Fn(&Window, &App) -> bool + 'static>;
/// Whether the rendered control is read-only. A read-only field stays
/// successful and focusable, but HTML constraint validation bars it: neither
/// required emptiness nor stored invalidity may block a submission. Read from
/// the mirror the component's render writes, so the answer is the rendered
/// state and not a stale builder flag.
type ReadReadOnly = Arc<dyn Fn(&App) -> bool + 'static>;

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
    /// Whether a focused Enter submits from this field. `None` on every
    /// non-text or compound field, which never submits implicitly.
    submits_on_enter_of: Option<SubmitsOnEnter>,
    /// Whether the rendered control is read-only — still successful and
    /// focusable, but barred from constraint validation.
    read_only_of: Option<ReadReadOnly>,
    /// The state entity carrying this field's value, when one exists. That
    /// entity outlives the frames, so it is the stable identity the form's
    /// blocked-keystroke latch is keyed by — the same trick
    /// `Input`'s `defaultValue` seed uses.
    state_id: Option<gpui::EntityId>,
}

impl FormField {
    /// A single-line text field, read from its [`InputState`].
    ///
    /// This is the registration for an [`Input`](crate::Input) or
    /// [`SearchField`](crate::SearchField) — the controls a native form
    /// implicitly submits from, so the field carries the Enter reader. A
    /// multi-line field rendered from the same state must register with
    /// [`FormField::text_area`].
    pub fn text(state: Entity<InputState>) -> Self {
        let read_state = state.clone();
        let name_state = state.clone();
        let behavior_state = state.clone();
        let successful_state = state.clone();
        let invalid_state = state.clone();
        let enter_state = state.clone();
        let read_only_state = state.clone();
        let state_id = state.entity_id();
        let focus_state = state;
        Self {
            name: None,
            name_of: Some(Arc::new(move |cx: &App| name_state.read(cx).name())),
            behavior_of: Some(Arc::new(move |cx: &App| {
                behavior_state.read(cx).validation_behavior()
            })),
            successful_of: Some(Arc::new(move |cx: &App| {
                successful_state.read(cx).is_successful()
            })),
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
            submits_on_enter_of: Some(Arc::new(move |window, cx| {
                enter_state.read(cx).focus_handle.is_focused(window)
            })),
            read_only_of: Some(Arc::new(move |cx: &App| {
                read_only_state.read(cx).is_read_only()
            })),
            state_id: Some(state_id),
        }
    }

    /// A multi-line text field, read from its [`InputState`] — the state a
    /// [`TextArea`](crate::TextArea) renders.
    ///
    /// Identical to [`FormField::text`] except that Enter never submits the
    /// form: a native form does not implicitly submit from a `<textarea>`,
    /// whose Enter is a newline. The multiline flag lives on the
    /// [`TextArea`](crate::TextArea) builder, not on the shared state, so
    /// the registration is where the control kind is named.
    pub fn text_area(state: Entity<InputState>) -> Self {
        let field = Self::text(state);
        Self {
            submits_on_enter_of: None,
            ..field
        }
    }

    /// A numeric field, read from its [`NumberState`].
    ///
    /// The field's validity is the one `NumberField::render` resolved —
    /// controlled flags, then server errors, then `validate` — mirrored onto
    /// the inner [`InputState`] exactly as `name` is, so the submission reads
    /// the rendered state rather than a builder snapshot. Read-only travels
    /// the same way: `NumberField` forwards the flag to the inner input,
    /// whose render mirrors it here.
    pub fn number(state: Entity<NumberState>) -> Self {
        let read_state = state.clone();
        let name_state = state.clone();
        let behavior_state = state.clone();
        let successful_state = state.clone();
        let invalid_state = state.clone();
        let enter_state = state.clone();
        let read_only_state = state.clone();
        let state_id = state.entity_id();
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
            successful_of: Some(Arc::new(move |cx: &App| {
                successful_state.read(cx).input.read(cx).is_successful()
            })),
            invalid_of: Some(Arc::new(move |cx: &App| {
                invalid_state.read(cx).input.read(cx).validity().is_invalid
            })),
            read: Arc::new(move |cx: &App| FormValue::Number(read_state.read(cx).value())),
            restore: None,
            is_required: false,
            validation_behavior: ValidationBehavior::Native,
            focus: Some(Arc::new(move |window, cx| {
                let fh = focus_state.read(cx).input.read(cx).focus_handle.clone();
                window.focus(&fh);
            })),
            // `<input type=number>` is a single-line text control: a native
            // form submits from it.
            submits_on_enter_of: Some(Arc::new(move |window, cx| {
                enter_state
                    .read(cx)
                    .input
                    .read(cx)
                    .focus_handle
                    .is_focused(window)
            })),
            read_only_of: Some(Arc::new(move |cx: &App| {
                read_only_state.read(cx).input.read(cx).is_read_only()
            })),
            state_id: Some(state_id),
        }
    }

    /// An OTP field, read from its [`crate::input_otp::OtpState`].
    ///
    /// Pinned v3 builds `InputOTP` on a single text input, and the row's
    /// cells share one focus handle, so a focused Enter here is the same
    /// implicit submission it is in any single-line field. The OTP answers
    /// no Enter of its own — its handler fills cells and walks the caret —
    /// so the keystroke bubbles to the form.
    pub fn code(name: impl Into<SharedString>, state: Entity<crate::input_otp::OtpState>) -> Self {
        let read_state = state.clone();
        let successful_state = state.clone();
        let invalid_state = state.clone();
        let enter_state = state.clone();
        let state_id = state.entity_id();
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
            successful_of: Some(Arc::new(move |cx: &App| {
                successful_state.read(cx).is_successful()
            })),
            invalid_of: Some(Arc::new(move |cx: &App| {
                invalid_state.read(cx).validity().is_invalid
            })),
            focus: Some(Arc::new(move |window, cx| {
                let fh = focus_state.read(cx).focus_handle.clone();
                window.focus(&fh);
            })),
            // The whole row is one text input to the form: Enter while any
            // cell holds the focus submits, exactly as it does from a
            // single-line field.
            submits_on_enter_of: Some(Arc::new(move |window, cx| {
                enter_state.read(cx).focus_handle.is_focused(window)
            })),
            // The OTP row has no read-only prop, so there is nothing to bar.
            read_only_of: None,
            state_id: Some(state_id),
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
            // A caller-held value has no rendered control to be read-only.
            read_only_of: None,
            submits_on_enter_of: None,
            state_id: None,
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
            // A live field belongs to a rendered non-text control — a switch,
            // a select, a checkbox — none of which submits implicitly, and
            // whose read-only state is not mirrored on this shared state.
            read_only_of: None,
            submits_on_enter_of: None,
            state_id: None,
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
            read_only_of: None,
            submits_on_enter_of: None,
            state_id: None,
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
            read_only_of: None,
            submits_on_enter_of: None,
            state_id: None,
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
            read_only_of: None,
            submits_on_enter_of: None,
            state_id: None,
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

    /// Whether Enter pressed right now submits the form from this field —
    /// the field holds the focus and is a single-line text control, the
    /// only thing a native form implicitly submits from.
    fn submits_on_enter(&self, window: &Window, cx: &App) -> bool {
        self.submits_on_enter_of
            .as_ref()
            .is_some_and(|f| f(window, cx))
    }

    /// Whether the rendered control is read-only: still successful and
    /// focusable, but barred from constraint validation — a native form
    /// neither blocks on a read-only field's emptiness nor on its stored
    /// errors, while its value still submits.
    fn is_read_only(&self, cx: &App) -> bool {
        self.read_only_of.as_ref().is_some_and(|f| f(cx))
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
        Self::collect_data(&self.fields, cx)
    }

    /// Collects `fields` into a submission — the body of [`Form::data`],
    /// shared with the submission path.
    fn collect_data(fields: &[FormField], cx: &App) -> FormData {
        let mut entries = Vec::with_capacity(fields.len());
        for field in fields {
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

    /// The names of the fields whose emptiness blocks submission: required,
    /// and not opted out with `validationBehavior: "aria"`. A read-only field
    /// is barred from constraint validation, so its emptiness never blocks.
    fn required_names(fields: &[FormField], cx: &App) -> Vec<SharedString> {
        fields
            .iter()
            .filter(|f| {
                f.is_successful(cx)
                    && f.is_required
                    && !f.is_read_only(cx)
                    && f.blocks_submission(cx)
            })
            .filter_map(|f| f.field_name(cx))
            .collect()
    }

    /// The one submission implementation: collect the named fields, decide
    /// whether validation blocks, and route to `on_submit` or `on_invalid`.
    /// Both doors into a submission — the caller-wired submit button
    /// ([`Form::submit_handler`]) and the form root's implicit Enter key
    /// handler — run this, so a submission is validated, focused and
    /// reported identically however it arrived.
    ///
    /// `defer_focus` is the Enter-origin latch. On the button path it is
    /// `None` and a blocked submit focuses the first invalid field inline,
    /// exactly as a click handler may. On the Enter path it is the keyed
    /// latch: focusing mid-keystroke would leave the newly focused control
    /// holding the focus when the key is *released*, and gpui activates a
    /// focused element's click listeners on release — a blocked Enter in a
    /// text field would open the Select it moved the focus to, or flip the
    /// Switch. So the latch is set instead, the release handler disarms the
    /// release and moves the focus once the dispatch is over.
    #[allow(clippy::too_many_arguments)] // one submission: fields, routing, callbacks, and the Enter-origin latch
    fn run_submission(
        fields: &[FormField],
        errors: &[SharedString],
        behavior: ValidationBehavior,
        on_submit: Option<&OnSubmit>,
        on_invalid: Option<&OnSubmit>,
        defer_focus: Option<&Entity<bool>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let data = Self::collect_data(fields, cx);
        let missing = data.missing_required(&Self::required_names(fields, cx));
        // A field error blocks a native submit however it arose: a
        // required field with no value, or a field whose stored validity
        // says it is in error (`validate`, `isInvalid`,
        // `validationErrors`, or an HTML5 attribute violation). Read-only
        // fields are barred from constraint validation on both counts.
        let own_invalid: Vec<SharedString> = fields
            .iter()
            .filter(|f| {
                f.is_successful(cx)
                    && f.blocks_submission(cx)
                    && !f.is_read_only(cx)
                    && f.is_invalid(cx)
            })
            .filter_map(|f| f.field_name(cx))
            .collect();
        let blocked = behavior == ValidationBehavior::Native
            && (!errors.is_empty() || !missing.is_empty() || !own_invalid.is_empty());
        if blocked {
            // v3, verbatim: "By default, the first invalid field will be
            // focused." A blocked submit moves the focus to the first
            // registered field whose error keeps the form from submitting —
            // the same union the blocked condition above computes. From the
            // button path this is an ordinary click handler; from the Enter
            // path the move is deferred past the keystroke (see
            // `defer_focus`). The tests drive both paths and assert the
            // field holds the focus after.
            let focus = Self::first_invalid_focus(fields, cx);
            match (defer_focus, focus) {
                (Some(latch), Some(_)) => latch.update(cx, |pending, _| *pending = true),
                (_, focus) => {
                    if let Some(focus) = focus {
                        focus(window, cx);
                    }
                }
            }
            if let Some(f) = on_invalid {
                f(&data, window, cx);
            }
        } else if let Some(f) = on_submit {
            f(&data, window, cx);
        }
    }

    /// The focus callback of the first registered field whose error blocks
    /// the submission — the same union [`Self::run_submission`] tests for
    /// the blocked decision, so the focus lands on the field the report is
    /// about. Read-only fields cannot nominate themselves, and a field with
    /// no reachable handle contributes none.
    fn first_invalid_focus(fields: &[FormField], cx: &App) -> Option<FocusField> {
        let data = Self::collect_data(fields, cx);
        fields
            .iter()
            .find(|field| {
                !field.is_read_only(cx)
                    && field.is_successful(cx)
                    && field.blocks_submission(cx)
                    && (field.is_invalid(cx)
                        || (field.is_required
                            && field.field_name(cx).is_some_and(|name| {
                                data.get(&name).is_none_or(FormValue::is_empty)
                            })))
            })
            .and_then(|field| field.focus.clone())
    }

    /// The handler a submit button calls: collects the submission, then routes
    /// it to `onSubmit` or `onInvalid`.
    ///
    /// gpui gives a child no way to reach its form, so the caller wires this to
    /// the button in place of `<button type="submit">`. The form root's Enter
    /// key handler runs the same implementation, so a submission is decided
    /// identically however it arrived.
    #[allow(clippy::arc_with_non_send_sync)] // see `util::shared`
    pub fn submit_handler(&self) -> Arc<dyn Fn(&mut Window, &mut App) + 'static> {
        let fields = self.fields.clone();
        let on_submit = self.on_submit.clone();
        let on_invalid = self.on_invalid.clone();
        let errors = self.validation_errors.clone();
        let behavior = self.validation_behavior;
        Arc::new(move |window: &mut Window, cx: &mut App| {
            // Button origin: no keystroke is in flight, so the focus move
            // needs no deferral.
            Self::run_submission(
                &fields,
                &errors,
                behavior,
                on_submit.as_ref(),
                on_invalid.as_ref(),
                None,
                window,
                cx,
            );
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            validation_errors: errors,
            validation_behavior: behavior,
            fields,
            on_submit,
            on_invalid,
            children,
            ..
        } = self;
        let mut el = gpui::div().flex().flex_col().gap(px(16.)).w_full();
        // Form-level messages sit above the fields, as v3's do.
        for message in &errors {
            el = el.child(crate::field::ErrorMessage::new(message.clone()));
        }
        // The blocked-Enter latch: "the keystroke whose release must not
        // click anything". Keyed window state, because the release can be
        // dispatched against a frame drawn after the press — closures from
        // different frames — and keyed by the first participating field's
        // state entity, the one stable identity a form without a DOM id has
        // (only entity-backed fields — text, number, OTP — can submit on
        // Enter, so a form without one never arms the latch).
        let latch = fields.iter().find_map(|f| f.state_id).map(|id| {
            window.use_keyed_state(
                gpui::ElementId::Name(format!("form-enter-latch-{}", id.as_u64()).into()),
                cx,
                |_, _| false,
            )
        });
        // A native `<form>` also submits when Enter is pressed in a
        // single-line text control — the implicit submission React Aria's
        // Form inherits, and the door this port must wire itself because the
        // submit button is caller-wired. The handler never infers from an
        // arbitrary bubbling Enter: it fires only while a *registered field
        // that participates* — [`FormField::text`], [`FormField::number`] or
        // [`FormField::code`] — holds the focus. A TextArea's Enter is a
        // newline; a focused submit Button submits through its own click,
        // which gpui fires on key-up, so firing here too would submit twice;
        // and the non-text compound controls never submit implicitly.
        let down_fields = fields.clone();
        let down_errors = errors;
        let down_latch = latch.clone();
        el = el.on_key_down(move |ev: &KeyDownEvent, window, cx| {
            // A latch still set when a press arrives is from a keystroke
            // whose release never happened here — always stale, because a
            // release follows its own press through this same dispatch
            // path. Clear it before anything else arms a fresh one.
            if let Some(latch) = &down_latch {
                latch.update(cx, |pending, _| *pending = false);
            }
            // Mirror the single-line field's own Enter branch: the plain key
            // (shift allowed — the rendered control submits on shift+enter
            // too), never a chord.
            let mods = &ev.keystroke.modifiers;
            if ev.keystroke.key.as_str() == "enter"
                && !(mods.control || mods.alt || mods.platform)
                && down_fields.iter().any(|f| f.submits_on_enter(window, cx))
            {
                Self::run_submission(
                    &down_fields,
                    &down_errors,
                    behavior,
                    on_submit.as_ref(),
                    on_invalid.as_ref(),
                    down_latch.as_ref(),
                    window,
                    cx,
                );
            }
        });
        // The release half of the same door: a blocked Enter set the latch
        // instead of moving the focus. Consume it on the plain release,
        // disarm gpui's key-up activation (`prevent_default` gates the
        // listener that fires a focused element's click), and move the focus
        // to the first invalid field only after the keystroke has fully
        // dispatched — the newly focused Select or Switch never holds the
        // focus during a key event, so the release cannot click it.
        let up_fields = fields;
        let up_latch = latch;
        el = el.capture_key_up(move |ev: &KeyUpEvent, window, cx| {
            let Some(latch) = &up_latch else {
                return;
            };
            let mods = &ev.keystroke.modifiers;
            if ev.keystroke.key.as_str() != "enter"
                || mods.control
                || mods.alt
                || mods.platform
                || !*latch.read(cx)
            {
                return;
            }
            latch.update(cx, |pending, _| *pending = false);
            window.prevent_default();
            if let Some(focus) = Self::first_invalid_focus(&up_fields, cx) {
                window.defer(cx, move |window, cx| focus(window, cx));
            }
        });
        el.children(children)
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
