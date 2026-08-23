//! RadioGroup — port of `@heroui/radio`.

use gpui::{prelude::*, px, App, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::{Color, FieldVariant, Orientation};
use herogpui_theme::ActiveTheme;

/// HeroUI RadioGroup.
#[derive(IntoElement)]
pub struct RadioGroup {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<gpui::SharedString>,
    id: gpui::ElementId,
    options: Vec<SharedString>,
    selected: Option<usize>,
    /// Whether `value` was supplied. `Option<usize>` cannot distinguish
    /// "controlled, nothing selected" from "uncontrolled" on its own.
    is_controlled: bool,
    default_value: Option<usize>,
    orientation: Orientation,
    is_disabled: bool,
    variant: FieldVariant,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    on_change: Option<std::sync::Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {

    pub fn variant(mut self, variant: FieldVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn is_invalid(mut self, v: bool) -> Self {
        self.is_invalid = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    /// `isReadOnly` — the value is shown but cannot be changed.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>, options: Vec<SharedString>) -> Self {
        Self {
            name: None,
            id: id.into(),
            options,
            selected: None,
            is_controlled: false,
            default_value: None,
            orientation: Orientation::Vertical,
            is_disabled: false,
            variant: FieldVariant::Primary,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            on_change: None,
        }
    }

    /// `name` — the name this control submits under.
    pub fn name(mut self, name: impl Into<gpui::SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// v3 discovers a field through the DOM; gpui gives a child no way to reach
    /// its ancestor, so the control hands the pair over instead. Borrows, so the
    /// control is still yours to place:
    ///
    /// ```ignore
    /// let field = control.form_field();
    /// form.field(field.unwrap()).child(control)
    /// ```
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        Some(crate::form::FormField::text_value(
                name,
                self.selected
                    .or(self.default_value)
                    .and_then(|i| self.options.get(i).cloned())
                    .unwrap_or_default(),
            ).is_required(self.is_required))
    }

    /// `value` — the selected option, by index. Supplying it makes the group
    /// controlled, even with `None`.
    pub fn value(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self.is_controlled = true;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Only consulted when `value` is not supplied; the group then owns the
    /// selection and a press moves it.
    pub fn default_value(mut self, i: Option<usize>) -> Self {
        self.default_value = i;
        self
    }


    pub fn orientation(mut self, o: Orientation) -> Self {
        self.orientation = o;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (selected, own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            self.is_controlled.then_some(self.selected),
            self.default_value,
        );

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();
        let (circle, dot, text, gap) = (px(18.), px(7.), px(14.), px(10.));

        let mut group = match self.orientation {
            Orientation::Horizontal => gpui::div().flex().items_center().gap(gap * 3.),
            Orientation::Vertical => gpui::div().flex().flex_col().gap(gap),
        };
        // `secondary` groups sit on a surface panel, matching the other
        // low-emphasis field variants.
        if self.variant == FieldVariant::Secondary {
            group = group
                .p(px(12.))
                .rounded(crate::util::field_radius(cx))
                .bg(colors.surface_secondary);
        }

        for (i, label) in self.options.into_iter().enumerate() {
            let is_selected = selected == Some(i);
            let mut circle_el = gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .size(circle)
                .rounded_full()
                .flex_shrink_0();

            if is_selected {
                let marker = if self.is_invalid {
                    colors.danger.color
                } else {
                    sem.color
                };
                circle_el = circle_el
                    .border_2()
                    .border_color(marker)
                    .child(gpui::div().size(dot).rounded_full().bg(marker));
            } else {
                circle_el = circle_el.border_2().border_color(if self.is_invalid {
                    colors.danger.color
                } else {
                    colors.default.soft_hover()
                });
            }

            let mut row = gpui::div()
                .id(gpui::ElementId::Name(format!(
                    "{}-opt-{i}",
                    element_id_name(&self.id)
                )
                .into()))
                .flex()
                .items_center()
                .gap(gap)
                .text_size(text)
                .text_color(colors.foreground)
                .when(!self.is_disabled && !self.is_read_only, |r| r.cursor_pointer())
                .when(self.is_disabled, |r| r.opacity(layout.disabled_opacity))

                .child(circle_el)
                .child(label.to_string());

            if !self.is_disabled
                && !self.is_read_only
                && (self.on_change.is_some() || own.is_some())
            {
                let on_change = self.on_change.clone();
                let own = own.clone();
                row = row.on_click(move |_, window, cx| {
                    // Uncontrolled: move our own selection, or pressing a
                    // radio would do nothing.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = Some(i);
                            cx.notify();
                        });
                    }
                    if let Some(f) = &on_change {
                        f(i, window, cx);
                    }
                });
            }

            group = group.child(row);
        }

        if self.is_required {
            group = group.child(
                gpui::div()
                    .text_size(px(12.))
                    .text_color(colors.danger.color)
                    .child("Required"),
            );
        }

        group
    }
}

fn element_id_name(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_string()
}


