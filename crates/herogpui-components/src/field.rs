//! Form field primitives — port of `@heroui/label`, `@heroui/description`,
//! `@heroui/error-message`, `@heroui/field-error` and `@heroui/fieldset`.
//!
//! These are the composition-friendly slots that HeroUI v3 field components
//! assemble internally, exposed here so applications can build custom fields
//! with the same typography and states.

use gpui::{
    div, px, AnyElement, App, InteractiveElement, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;

/// HeroUI Label — `slot="label"`.
///
/// Mirrors the React API: `isRequired`, `isDisabled`, `isInvalid`. The
/// `htmlFor` prop has no gpui analogue (there is no DOM id graph), so labels
/// are associated by composition instead.
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    is_required: bool,
    is_disabled: bool,
    is_invalid: bool,
    /// `htmlFor` — the field this label names.
    label_for: Option<(gpui::ElementId, gpui::FocusHandle)>,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            is_required: false,
            is_disabled: false,
            is_invalid: false,
            label_for: None,
        }
    }

    /// `htmlFor` — associates the label with a field.
    ///
    /// In HTML this is an id reference; here it is the field's focus handle,
    /// which is what makes the association do the one thing it does visibly:
    /// clicking the label focuses the field. Pass a distinct `id` per label so
    /// the click target has one.
    pub fn label_for(mut self, id: impl Into<gpui::ElementId>, handle: gpui::FocusHandle) -> Self {
        self.label_for = Some((id.into(), handle));
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
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
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let mut el = div()
            .flex()
            .items_center()
            .gap(px(2.))
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(if self.is_invalid {
                colors.danger.color
            } else {
                colors.foreground
            })
            .child(self.text.to_string());

        if self.is_required {
            el = el.child(div().text_color(colors.danger.color).child("*".to_owned()));
        }

        if self.is_disabled {
            el = el.opacity(cx.layout().disabled_opacity);
        }

        // `htmlFor`: clicking the label focuses the field it names.
        match self.label_for {
            Some((id, handle)) if !self.is_disabled => el
                .id(id)
                .cursor_pointer()
                .on_click(move |_: &gpui::ClickEvent, window: &mut Window, _| {
                    window.focus(&handle);
                })
                .into_any_element(),
            _ => el.into_any_element(),
        }
    }
}

/// HeroUI Description — `slot="description"`. De-emphasised helper copy shown
/// beneath a field.
#[derive(IntoElement)]
pub struct Description {
    text: SharedString,
}

impl Description {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for Description {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .text_size(px(12.))
            .line_height(px(16.))
            .text_color(cx.colors().muted)
            .child(self.text.to_string())
    }
}

/// HeroUI ErrorMessage — `slot="errorMessage"`. Always rendered when present.
#[derive(IntoElement)]
pub struct ErrorMessage {
    text: SharedString,
}

impl ErrorMessage {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for ErrorMessage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .text_size(px(12.))
            .line_height(px(16.))
            .text_color(cx.colors().danger.color)
            .child(self.text.to_string())
    }
}

/// HeroUI FieldError — validation-driven error text.
///
/// Unlike [`ErrorMessage`], a `FieldError` manages its own visibility from the
/// validation state: it renders nothing unless the field is invalid and a
/// message is present.
#[derive(IntoElement)]
pub struct FieldError {
    text: Option<SharedString>,
    is_invalid: bool,
}

impl FieldError {
    pub fn new() -> Self {
        Self {
            text: None,
            is_invalid: false,
        }
    }

    /// The validation message. Setting a message also marks the field invalid,
    /// matching React Aria's `ValidationResult` behaviour.
    pub fn message(mut self, text: impl Into<SharedString>) -> Self {
        self.text = Some(text.into());
        self.is_invalid = true;
        self
    }

    /// Overrides visibility independently of the message.
    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }
}

impl Default for FieldError {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for FieldError {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match (self.is_invalid, self.text) {
            (true, Some(text)) => ErrorMessage::new(text).into_any_element(),
            _ => div().into_any_element(),
        }
    }
}

/// HeroUI Fieldset — groups related form controls under a legend.
///
/// Compose with [`FieldsetLegend`], [`FieldsetGroup`] and [`FieldsetActions`],
/// mirroring `Fieldset.Legend` / `.Group` / `.Actions` in React.
#[derive(IntoElement)]
pub struct Fieldset {
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl Fieldset {
    pub fn new() -> Self {
        Self {
            gap: px(24.),
            children: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl Default for Fieldset {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Fieldset {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Fieldset {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(self.gap)
            .text_color(cx.colors().foreground)
            .children(self.children)
    }
}

/// `Fieldset.Legend` — the group's caption.
#[derive(IntoElement)]
pub struct FieldsetLegend {
    text: SharedString,
}

impl FieldsetLegend {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for FieldsetLegend {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .text_size(px(16.))
            .line_height(px(24.))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(cx.colors().foreground)
            .child(self.text.to_string())
    }
}

/// `Fieldset.Group` — layout wrapper for the grouped controls.
#[derive(IntoElement)]
pub struct FieldsetGroup {
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl FieldsetGroup {
    pub fn new() -> Self {
        Self {
            gap: px(12.),
            children: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl Default for FieldsetGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for FieldsetGroup {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FieldsetGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(self.gap)
            .children(self.children)
    }
}

/// `Fieldset.Actions` — trailing row for submit/cancel controls.
#[derive(IntoElement)]
pub struct FieldsetActions {
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl FieldsetActions {
    pub fn new() -> Self {
        Self {
            gap: px(8.),
            children: Vec::new(),
        }
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl Default for FieldsetActions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for FieldsetActions {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FieldsetActions {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(self.gap)
            .children(self.children)
    }
}
