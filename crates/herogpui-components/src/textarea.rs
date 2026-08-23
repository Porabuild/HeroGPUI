//! TextArea — port of `@heroui/text-area` (v3).
//!
//! Reuses [`InputState`] in multi-line mode: gpui wraps text by default
//! (`WhiteSpace::Normal`), so the field lays one wrapping paragraph out per
//! newline, Enter inserts a newline instead of submitting, and the caret is
//! placed inside the line it falls in. `rows` sets the visible height.

use crate::input::{Input, InputState};
use gpui::{prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, Styled, Window};

/// Multi-line text field.
#[derive(IntoElement)]
pub struct TextArea {
    inner: Input,
    /// `cols`, as a pixel width. `None` leaves the field's natural width.
    min_w: Option<gpui::Pixels>,
}

/// `rows` as a height: one 20px line each, over `.textarea`'s `py-2`.
fn rows_height(rows: u32) -> gpui::Pixels {
    px(rows.max(1) as f32 * 20.0 + 16.0)
}

impl TextArea {
    /// `value` — writes through to the bound [`InputState`].
    pub fn value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.inner.state().update(cx, |s, _| s.set_value(value));
        self
    }

    /// `maxLength` — refuses keystrokes past this many characters.
    pub fn max_length(mut self, n: usize) -> Self {
        self.inner = self.inner.max_length(n);
        self
    }

    /// `minLength` — reported by the inner field's `validity`.
    pub fn min_length(mut self, n: usize) -> Self {
        self.inner = self.inner.min_length(n);
        self
    }

    /// `cols` — visible width, in characters.
    ///
    /// gpui has no `ch` unit, so this is the column count times the size's
    /// character advance, the same approximation [`TextArea::rows`] makes for
    /// height.
    pub fn cols(mut self, cols: u32) -> Self {
        self.min_w = Some(px(cols.max(1) as f32 * 8.0));
        self
    }

    /// `rows` — the visible line count, at one line height per row plus the
    /// field's vertical padding.
    pub fn rows(mut self, rows: u32) -> Self {
        self.inner = self.inner.min_h(rows_height(rows));
        self
    }

    /// `name` — see [`crate::input::Input::name`].
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.inner = self.inner.name(name);
        self
    }

    /// `defaultValue` — see [`crate::input::Input::default_value`].
    pub fn default_value(mut self, text: impl Into<SharedString>) -> Self {
        self.inner = self.inner.default_value(text);
        self
    }

    /// `validationBehavior` — see [`crate::input::Input::validation_behavior`].
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.inner = self.inner.validation_behavior(behavior);
        self
    }

    pub fn variant(mut self, variant: herogpui_core::FieldVariant) -> Self {
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

    /// `validate` — see [`crate::input::Input::validate`].
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.inner = self.inner.validate(f);
        self
    }

    /// `validationErrors` — see [`crate::input::Input::validation_errors`].
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

    pub fn on_change(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }

    pub fn new(state: Entity<InputState>) -> Self {
        Self {
            // v3 documents `rows` as defaulting to 3.
            inner: Input::new(state).multiline(true).min_h(rows_height(3)),
            min_w: None,
        }
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.inner = self.inner.label(l);
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.inner = self.inner.placeholder(p);
        self
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.inner = self.inner.description(d);
        self
    }
}

impl TextArea {
    /// Hands the inner field to [`crate::input_group::InputGroup::text_area`].
    ///
    /// `InputGroup.TextArea` is the same multi-line field with the group's
    /// chrome instead of its own, so the wrapper this normally renders (which
    /// only carries `cols`) is dropped.
    pub(crate) fn into_group_input(self) -> Input {
        self.inner
    }
}

impl RenderOnce for TextArea {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // The field itself is multi-line; this wrapper only gives it the height
        // `rows` asks for and keeps the text at the top of it. The field paints
        // its own chrome (`util::apply_field_chrome`) -- the wrapper used to
        // repaint a `default.soft()` background at a hardcoded 10px radius,
        // neither of which is a v3 value.
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .when_some(self.min_w, |e, w| e.min_w(w))
            .child(self.inner)
    }
}
