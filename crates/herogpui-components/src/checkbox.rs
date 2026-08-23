//! Checkbox — port of `@heroui/checkbox`.

use gpui::{
    prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// HeroUI Checkbox.
#[derive(IntoElement)]
pub struct Checkbox {
    id: gpui::ElementId,
    /// `isSelected` — `None` leaves the component holding the state, seeded
    /// from `defaultSelected`.
    checked: Option<bool>,
    default_checked: bool,
    is_indeterminate: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_required: bool,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<bool>>,
    /// `validationErrors` — messages from a server round-trip.
    validation_errors: Vec<gpui::SharedString>,
    is_invalid: bool,
    variant: herogpui_core::FieldVariant,
    children: Vec<AnyElement>,
    on_change: Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl Checkbox {

    /// `isReadOnly` — shows the value but refuses changes.
    /// `validate` — returns the message to show, or `None` when the state is fine.
    ///
    /// The component runs it and surfaces the result.
    pub fn validate(
        mut self,
        f: impl Fn(&bool) -> Option<gpui::SharedString> + 'static,
    ) -> Self {
        self.validate = Some(std::sync::Arc::new(f));
        self
    }

    /// `validationErrors` — messages produced elsewhere, shown ahead of
    /// whatever `validate` returns.
    pub fn validation_errors(
        mut self,
        errors: impl IntoIterator<Item = impl Into<gpui::SharedString>>,
    ) -> Self {
        self.validation_errors = errors.into_iter().map(Into::into).collect();
        self
    }

    /// `isRequired` — marks the label as required.
    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `isInvalid` — draws the control in the danger role.
    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    /// `variant` — `Secondary` drops the shadow for use on a surface.
    pub fn variant(mut self, variant: herogpui_core::FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: None,
            default_checked: false,
            is_indeterminate: false,
            is_disabled: false,
            is_read_only: false,
            is_required: false,
            validate: None,
            validation_errors: Vec::new(),
            is_invalid: false,
            variant: herogpui_core::FieldVariant::Primary,
            children: Vec::new(),
            on_change: None,
        }
    }

    /// `isSelected` — the controlled state; `None` leaves the component
    /// holding it, seeded from `defaultSelected`.
    pub fn is_selected(mut self, v: bool) -> Self {
        self.checked = Some(v);
        self
    }

    /// `defaultSelected` — the uncontrolled initial state.
    ///
    /// Only consulted when `checked` is not supplied; the component then owns
    /// the state and toggles itself on click.
    pub fn default_selected(mut self, v: bool) -> Self {
        self.default_checked = v;
        self
    }

    /// Shows a dash instead of the check (`isIndeterminate`).
    pub fn is_indeterminate(mut self, v: bool) -> Self {
        self.is_indeterminate = v;
        self
    }



    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Label content.
    pub fn label(mut self, el: impl IntoElement) -> Self {
        self.children.push(el.into_any_element());
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl ParentElement for Checkbox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (checked, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-checked", self.id).into()),
            self.checked,
            self.default_checked,
        );

        // v3 order: the controlled flag, then server errors, then `validate`.
        let validity = crate::validation::resolve(
            self.is_invalid,
            &self.validation_errors,
            self.validate.as_ref().and_then(|f| f(&checked)),
            None,
        );

        // `isInvalid` outranks the colour role, as it does on every field.
        let sem = if validity.is_invalid {
            cx.role(Color::Danger)
        } else {
            cx.role(Color::Accent)
        };
        let colors = cx.colors();
        let layout = cx.layout();

        let (box_px, icon_px, text) = (px(18.), px(12.), px(14.));

        let active = checked || self.is_indeterminate;

        let mut boxel = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .size(box_px)
            .rounded(crate::util::control_radius(cx))
            .flex_shrink_0()
            // `Primary` carries the field shadow; `Secondary` is the flat
            // variant meant for use on a surface.
            .when(
                self.variant == herogpui_core::FieldVariant::Primary
                    && !layout.field_shadow.is_empty(),
                |e| e.shadow(layout.field_shadow.clone()),
            );

        if active {
            boxel = boxel.bg(sem.color).border_2().border_color(sem.color);
        } else {
            boxel = boxel
                .bg(colors.default.soft())
                .border_2()
                .border_color(colors.default.soft_hover());
        }

        if self.is_indeterminate {
            boxel = boxel.child(
                gpui::div()
                    .w(icon_px)
                    .h(px(2.))
                    .rounded_full()
                    .bg(sem.foreground),
            );
        } else if checked {
            boxel =
                boxel.child(gpui::svg().size(icon_px).path(icons::CHECK).text_color(sem.foreground));
        }

        let row = gpui::div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .gap(px(8.))
            .when(!self.is_disabled && !self.is_read_only, |r| r.cursor_pointer())
            .when(self.is_disabled, |r| r.opacity(layout.disabled_opacity))
            .children(
                std::iter::once(boxel.into_any_element())
                    .chain(self.children)
                    .chain(self.is_required.then(|| {
                        gpui::div()
                            .text_color(colors.danger.color)
                            .child("*")
                            .into_any_element()
                    })),
            )
            .text_size(text)
            .text_color(colors.foreground);

        if !self.is_disabled && !self.is_read_only && (self.on_change.is_some() || own.is_some()) {
            let on_change = self.on_change;
            return row
                .on_click(move |_, window, cx| {
                    // Uncontrolled: flip our own copy, or nothing could ever
                    // change it.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = !checked;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change {
                        cb(!checked, window, cx);
                    }
                })
                .into_any_element();
        }
        row.into_any_element()
    }
}

// ---------------------------------------------------------------------------
// CheckboxGroup
// ---------------------------------------------------------------------------

/// One option in a [`CheckboxGroup`].
#[derive(Clone)]
pub struct CheckboxOption {
    key: gpui::SharedString,
    label: gpui::SharedString,
    description: Option<gpui::SharedString>,
    is_disabled: bool,
}

impl CheckboxOption {
    pub fn new(
        key: impl Into<gpui::SharedString>,
        label: impl Into<gpui::SharedString>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: None,
            is_disabled: false,
        }
    }

    pub fn description(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn key(&self) -> &gpui::SharedString {
        &self.key
    }
}

type OnGroupChange =
    std::sync::Arc<dyn Fn(&std::collections::HashSet<gpui::SharedString>, &mut Window, &mut App)>;

/// CheckboxGroup — port of `@heroui/checkbox-group` (v3).
///
/// A set of checkboxes sharing a label, orientation, validation state and
/// selected-value set.
#[derive(IntoElement)]
pub struct CheckboxGroup {
    id: gpui::ElementId,
    options: Vec<CheckboxOption>,
    label: Option<gpui::SharedString>,
    description: Option<gpui::SharedString>,
    error_message: Option<gpui::SharedString>,
    value: Option<std::collections::HashSet<gpui::SharedString>>,
    default_value: std::collections::HashSet<gpui::SharedString>,
    orientation: herogpui_core::Orientation,
    variant: herogpui_core::FieldVariant,
    is_disabled: bool,
    is_read_only: bool,
    is_invalid: bool,
    is_required: bool,
    on_change: Option<OnGroupChange>,
}

impl CheckboxGroup {
    pub fn new(id: impl Into<gpui::ElementId>, options: Vec<CheckboxOption>) -> Self {
        Self {
            id: id.into(),
            options,
            label: None,
            description: None,
            error_message: None,
            value: None,
            default_value: std::collections::HashSet::new(),
            orientation: herogpui_core::Orientation::Vertical,
            variant: herogpui_core::FieldVariant::Primary,
            is_disabled: false,
            is_read_only: false,
            is_invalid: false,
            is_required: false,
            on_change: None,
        }
    }

    pub fn label(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn description(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn error_message(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.error_message = Some(text.into());
        self
    }

    /// `value` — the selected keys, controlled.
    pub fn value(mut self, keys: impl IntoIterator<Item = gpui::SharedString>) -> Self {
        self.value = Some(keys.into_iter().collect());
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Only consulted when `value` is not supplied; the group then owns the
    /// selection and each checkbox toggles its own key in it.
    pub fn default_value(
        mut self,
        keys: impl IntoIterator<Item = gpui::SharedString>,
    ) -> Self {
        self.default_value = keys.into_iter().collect();
        self
    }

    pub fn orientation(mut self, orientation: herogpui_core::Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn variant(mut self, variant: herogpui_core::FieldVariant) -> Self {
        self.variant = variant;
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

    /// `isReadOnly` — every option shows its state but cannot be toggled.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// Called with the complete selection after any box is toggled.
    pub fn on_change(
        mut self,
        handler: impl Fn(&std::collections::HashSet<gpui::SharedString>, &mut Window, &mut App)
            + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for CheckboxGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (value, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            self.value.clone(),
            self.default_value.clone(),
        );

        let colors = cx.colors();
        let is_invalid = self.is_invalid || self.error_message.is_some();

        let mut root = gpui::div().flex().flex_col().gap(px(8.));

        if let Some(label) = &self.label {
            root = root.child(
                crate::field::Label::new(label.clone())
                    .is_required(self.is_required)
                    .is_disabled(self.is_disabled)
                    .is_invalid(is_invalid),
            );
        }

        let mut list = gpui::div().flex().gap(px(10.));
        list = match self.orientation {
            herogpui_core::Orientation::Vertical => list.flex_col(),
            herogpui_core::Orientation::Horizontal => list.flex_row().flex_wrap(),
        };

        for (index, option) in self.options.iter().enumerate() {
            let key = option.key.clone();
            let checked = value.contains(&key);
            let disabled = self.is_disabled || option.is_disabled;

            let mut label_el = gpui::div()
                .flex()
                .flex_col()
                .child(gpui::div().child(option.label.to_string()));
            if let Some(description) = &option.description {
                label_el = label_el.child(
                    gpui::div()
                        .text_size(px(12.))
                        .text_color(colors.muted)
                        .child(description.to_string()),
                );
            }

            let selection = value.clone();
            let on_change = self.on_change.clone();
            let own = own.clone();
            list = list.child(
                Checkbox::new(gpui::ElementId::Name(
                    format!("{:?}-opt-{index}", self.id).into(),
                ))
                .is_selected(checked)
                .is_disabled(disabled)
                .is_read_only(self.is_read_only)
                .is_invalid(is_invalid)
                .variant(self.variant)
                .label(label_el)
                .on_change(move |_next, window, cx| {
                    let mut set = selection.clone();
                    if !set.remove(&key) {
                        set.insert(key.clone());
                    }
                    // Uncontrolled: keep the new set, or ticking a box would
                    // do nothing.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = set.clone();
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_change {
                        cb(&set, window, cx);
                    }
                }),
            );
        }

        root = root.child(list);

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
