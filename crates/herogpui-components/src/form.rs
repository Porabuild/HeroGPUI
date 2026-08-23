//! Form — port of `@heroui/form` (v1: layout + submit signal).

use gpui::{px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window};

type OnSubmit = std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// HeroUI Form container (v1): vertical field stack with a submit callback
/// you can wire to an `Input`'s Enter or a submit Button.
#[derive(IntoElement)]
pub struct Form {
    /// `validationErrors` — form-level messages, shown above the fields.
    validation_errors: Vec<gpui::SharedString>,
    is_disabled: bool,
    on_submit: Option<OnSubmit>,
    children: Vec<AnyElement>,
}

/// How invalid children are handled (`validationBehavior`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ValidationBehavior {
    #[default]
    Native,
    Allow,
}

impl Form {
    pub fn new() -> Self {
        Self {
            validation_errors: Vec::new(),
            is_disabled: false,
            on_submit: None,
            children: Vec::new(),
        }
    }

    /// `validationErrors` — form-level messages, shown above the fields.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }


    /// Fired by the caller when the form should be submitted (e.g. from a
    /// submit button or an input's Enter handler).
    pub fn on_submit(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(std::sync::Arc::new(f));
        self
    }

    /// Returns the stored submit callback so buttons inside the form can call it.
    pub fn submit_handler(&self) -> Option<OnSubmit> {
        self.on_submit.clone()
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

