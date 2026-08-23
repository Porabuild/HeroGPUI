//! InputGroup — port of `@heroui/input-group` (v3).
//!
//! Combines a field with adjacent addons and controls behind one shared piece
//! of field chrome, so a prefix label, the input itself and a trailing button
//! read as a single control.

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
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
        // `.input-group__prefix` / `__suffix`: `px-3`, transparent, and drawn in
        // `--field-placeholder`.
        div()
            .flex()
            .items_center()
            .flex_shrink_0()
            .px(px(12.))
            .text_color(cx.colors().field.placeholder)
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
    is_required: bool,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    /// `InputGroup.Prefix` — the leading addon.
    prefix: Option<AnyElement>,
    /// `InputGroup.Suffix` — the trailing addon.
    suffix: Option<AnyElement>,
    /// `InputGroup.Input` / `InputGroup.TextArea` — held rather than rendered
    /// so the group can strip its chrome and tell it which sides an addon
    /// occupies.
    input: Option<crate::input::Input>,
    children: Vec<AnyElement>,
}

impl InputGroup {
    pub fn new() -> Self {
        Self {
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            label: None,
            description: None,
            error_message: None,
            prefix: None,
            suffix: None,
            input: None,
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

    /// `isRequired` — marks the group's label as required. v3's examples get
    /// this from the `TextField` around the group.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
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

    /// `InputGroup.Prefix` — content before the field.
    pub fn prefix(mut self, el: impl IntoElement) -> Self {
        self.prefix = Some(el.into_any_element());
        self
    }

    /// `InputGroup.Suffix` — content after the field.
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    /// `InputGroup.Input` — the field itself.
    ///
    /// Taken as an [`crate::input::Input`] rather than an element so the group
    /// can strip its chrome: v3's group paints the box, and the inner input is
    /// transparent and flush against the addons. Passing one as a plain child
    /// instead leaves a second, smaller field drawn inside the group.
    pub fn input(mut self, input: crate::input::Input) -> Self {
        self.input = Some(input);
        self
    }

    /// `InputGroup.TextArea` — a multi-line field in the same shared chrome.
    pub fn text_area(mut self, text_area: crate::textarea::TextArea) -> Self {
        self.input = Some(text_area.into_group_input());
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.input-group` rings on `focus-within`, and what is inside it is a
        // real `Input` or `TextArea`, so their state is where the focus is.
        // A `TextArea` is converted to an `Input` by `text_area`, so there is one
        // slot to ask.
        let focus_within = self
            .input
            .as_ref()
            .is_some_and(|input| input.state_focus(cx).is_focused(window));
        let colors = cx.colors();
        let layout = cx.layout();
        let is_invalid = self.is_invalid || self.error_message.is_some();

        // `.input-group` is `inline-flex min-h-9 items-center` with no padding
        // of its own: the prefix, the input and the suffix each carry `px-3`,
        // which is what keeps the addons flush with the field's edges.
        let mut group = div()
            .flex()
            .flex_row()
            .items_center()
            .min_h(util::FIELD_HEIGHT)
            .text_size(util::FIELD_TEXT)
            .text_color(colors.field.foreground);

        // v3 rings the *group* on `focus-within`, so the state comes from the
        // field inside it.
        group = util::apply_field_chrome(group, self.variant, is_invalid, focus_within, cx);
        if self.is_disabled {
            group = group.opacity(layout.disabled_opacity);
        }
        if self.full_width {
            group = group.w_full();
        }

        // Order matters: prefix, field, suffix, then anything else the caller
        // put in. The field is told which sides an addon occupies so it can
        // drop that padding.
        let (has_prefix, has_suffix) = (self.prefix.is_some(), self.suffix.is_some());
        if let Some(prefix) = self.prefix {
            group = group.child(prefix);
        }
        if let Some(input) = self.input {
            group = group.child(input.in_group(has_prefix, has_suffix));
        }
        if let Some(suffix) = self.suffix {
            group = group.child(suffix);
        }
        group = group.children(self.children);

        let mut root = div().flex().flex_col().gap(px(6.));
        if let Some(label) = self.label {
            root = root.child(
                crate::field::Label::new(label)
                    .is_invalid(is_invalid)
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled),
            );
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
