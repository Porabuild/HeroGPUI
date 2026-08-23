//! The shared `validate` / `validationErrors` model.
//!
//! v3's `validate` is a function the *component* runs: it receives the current
//! value and returns either nothing or a message to display. Treating it as
//! "the caller validates and passes `isInvalid`" loses that — the component
//! never runs anything and the prop does not exist. This module is the shape
//! every field uses instead, so the eight components that document `validate`
//! behave identically.

use std::sync::Arc;

use gpui::SharedString;

/// `validate` — returns `None` when the value is acceptable, or the message to
/// show when it is not.
///
/// v3 types this `(value) => ValidationError | true | null | undefined`, where
/// a returned error marks the field invalid; `None` here is that `null`.
pub type Validator<T> = Arc<dyn Fn(&T) -> Option<SharedString> + 'static>;
// `T` may be unsized (`str`); a type alias does not enforce bounds, so none
// is written here.

/// The messages a field should display, and whether it is invalid.
///
/// Order matches v3: an explicit `isInvalid` always wins, then server-supplied
/// `validationErrors`, then whatever `validate` returns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Validity {
    pub is_invalid: bool,
    pub messages: Vec<SharedString>,
}

impl Validity {
    /// The first message, which is what a single-line field slot shows.
    pub fn first(&self) -> Option<SharedString> {
        self.messages.first().cloned()
    }
}

/// Resolves a field's validity from the three sources v3 defines.
///
/// * `is_invalid` — the controlled flag; forces invalid even with no message.
/// * `errors` — `validationErrors`, e.g. from a server round-trip.
/// * `validate_result` — whatever the `validate` function returned.
/// * `own_message` — the component's own `errorMessage` prop.
pub fn resolve(
    is_invalid: bool,
    errors: &[SharedString],
    validate_result: Option<SharedString>,
    own_message: Option<SharedString>,
) -> Validity {
    let mut messages: Vec<SharedString> = Vec::new();
    messages.extend(errors.iter().cloned());
    if let Some(m) = validate_result {
        messages.push(m);
    }
    // The component's own message is the fallback, not an addition: showing it
    // alongside a specific validation failure would duplicate the reason.
    if messages.is_empty() {
        if let Some(m) = own_message {
            messages.push(m);
        }
    }
    Validity {
        is_invalid: is_invalid || !messages.is_empty(),
        messages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> SharedString {
        SharedString::from(v.to_owned())
    }

    #[test]
    fn clean_value_is_valid() {
        let v = resolve(false, &[], None, None);
        assert!(!v.is_invalid);
        assert!(v.messages.is_empty());
        assert_eq!(v.first(), None);
    }

    #[test]
    fn is_invalid_alone_marks_the_field() {
        // No message, but the caller says it is wrong.
        let v = resolve(true, &[], None, None);
        assert!(v.is_invalid);
        assert!(v.messages.is_empty());
    }

    #[test]
    fn a_validate_failure_marks_and_reports() {
        let v = resolve(false, &[], Some(s("Too short")), None);
        assert!(v.is_invalid);
        assert_eq!(v.first(), Some(s("Too short")));
    }

    #[test]
    fn server_errors_come_first() {
        let v = resolve(false, &[s("Already taken")], Some(s("Too short")), None);
        assert_eq!(v.messages, vec![s("Already taken"), s("Too short")]);
        assert_eq!(v.first(), Some(s("Already taken")));
    }

    #[test]
    fn own_message_is_a_fallback_not_an_addition() {
        // With a specific failure, the generic message would duplicate it.
        let v = resolve(
            false,
            &[],
            Some(s("Too short")),
            Some(s("Check this field")),
        );
        assert_eq!(v.messages, vec![s("Too short")]);
        // With nothing specific, it is what the field shows.
        let v = resolve(false, &[], None, Some(s("Check this field")));
        assert_eq!(v.messages, vec![s("Check this field")]);
        assert!(v.is_invalid);
    }
}
