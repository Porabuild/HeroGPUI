//! InputGroup — port of `@heroui/input-group` (v3).
//!
//! Combines a field with adjacent addons and controls behind one shared piece
//! of field chrome, so a prefix label, the input itself and a trailing button
//! read as a single control.

use gpui::{
    div, prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::FieldVariant;
use herogpui_theme::ActiveTheme;

use crate::util;

/// A static, non-interactive segment of an [`InputGroup`] — the `$` before an
/// amount, or a `.com` suffix.
#[derive(IntoElement)]
pub struct InputAddon {
    text: SharedString,
}

impl InputAddon {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for InputAddon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .text_color(cx.colors().muted)
            .child(self.text.to_string())
    }
}

/// HeroUI InputGroup.
#[derive(IntoElement)]
pub struct InputGroup {
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl InputGroup {
    pub fn new() -> Self {
        Self {
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            label: None,
            description: None,
            error_message: None,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
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

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
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

}

impl Default for InputGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for InputGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for InputGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        let is_invalid = self.is_invalid || self.error_message.is_some();

        let mut group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .px(px(12.))
            .h(util::FIELD_HEIGHT)
            .text_size(util::FIELD_TEXT)
            .rounded(util::field_radius(cx))
            .text_color(colors.field.foreground);

        group = match self.variant {
            FieldVariant::Primary => {
                let shadow = layout.field_shadow.clone();
                group
                    .bg(colors.field.background)
                    .when(!shadow.is_empty(), |e| e.shadow(shadow))
            }
            FieldVariant::Secondary => group.bg(colors.surface_secondary),
        };

        if is_invalid {
            group = group.border_1().border_color(colors.danger.color);
        }
        if self.is_disabled {
            group = group.opacity(layout.disabled_opacity);
        }
        if self.full_width {
            group = group.w_full();
        }

        group = group.children(self.children);

        let mut root = div().flex().flex_col().gap(px(6.));
        if let Some(label) = self.label {
            root = root.child(crate::field::Label::new(label).is_invalid(is_invalid));
        }
        root = root.child(group);

        if is_invalid {
            if let Some(message) = self.error_message {
                root = root.child(crate::field::ErrorMessage::new(message));
            }
        } else if let Some(description) = self.description {
            root = root.child(crate::field::Description::new(description));
        }

        root
    }
}
