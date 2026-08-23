//! TextArea — port of `@heroui/text-area` (v3).
//!
//! Reuses [`InputState`]; gpui 0.2.2 has no multi-line text layout, so this
//! renders a taller, top-aligned single-line surface and `rows` sets its height.

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::input::{Input, InputState};

/// Multi-line text field.
#[derive(IntoElement)]
pub struct TextArea {
    inner: Input,
    min_h: gpui::Pixels,
    /// `cols`, as a pixel width. `None` leaves the field's natural width.
    min_w: Option<gpui::Pixels>,
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

    /// `rows` — visible line count. gpui has no multi-line text layout in this
    /// version, so this sets the control height at ~20px per row.
    /// `cols` — visible width, in characters.
    ///
    /// gpui has no `ch` unit, so this is the column count times the size's
    /// character advance, the same approximation [`TextArea::rows`] makes for
    /// height.
    pub fn cols(mut self, cols: u32) -> Self {
        self.min_w = Some(px(cols.max(1) as f32 * 8.0));
        self
    }

    pub fn rows(mut self, rows: u32) -> Self {
        self.min_h = px(rows.max(1) as f32 * 20.0 + 20.0);
        self
    }


    /// `name` — see [`crate::input::Input::name`].
    pub fn name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.inner = self.inner.name(name);
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
    pub fn validate(
        mut self,
        f: impl Fn(&str) -> Option<gpui::SharedString> + 'static,
    ) -> Self {
        self.inner = self.inner.validate(f);
        self
    }

    /// `validationErrors` — see [`crate::input::Input::validation_errors`].
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.inner = self.inner.validation_errors(errors);
        self
    }

    pub fn error_message(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.inner = self.inner.error_message(text);
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.inner = self.inner.on_change(handler);
        self
    }

    pub fn new(state: Entity<InputState>) -> Self {
        Self {
            inner: Input::new(state),
            min_h: px(80.),
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

impl RenderOnce for TextArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        // Render the input inside a tall surface; the caret row is vertically
        // top-aligned by overriding alignment via a wrapper.
        gpui::div()
            .flex()
            .flex_col()
            .min_h(self.min_h)
            .when_some(self.min_w, |e, w| e.min_w(w))
            .bg(colors.default.soft())
            .rounded(px(10.))
            .child(self.inner)
    }
}
