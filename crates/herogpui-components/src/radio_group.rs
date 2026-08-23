//! RadioGroup — port of `@heroui/radio`.

use gpui::{prelude::*, px, App, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::{FieldVariant, Color, Orientation, Size};
use herogpui_theme::ActiveTheme;

/// HeroUI RadioGroup.
#[derive(IntoElement)]
pub struct RadioGroup {
    id: gpui::ElementId,
    options: Vec<SharedString>,
    selected: Option<usize>,
    color: Color,
    size: Size,
    orientation: Orientation,
    is_disabled: bool,
    variant: FieldVariant,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    on_change: Option<std::sync::Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {
    /// `value` — the v3 name for [`RadioGroup::selected`].
    pub fn value(self, index: Option<usize>) -> Self {
        self.selected(index)
    }

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
            id: id.into(),
            options,
            selected: None,
            color: Color::Accent,
            size: Size::Md,
            orientation: Orientation::Vertical,
            is_disabled: false,
            variant: FieldVariant::Primary,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            on_change: None,
        }
    }

    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();
        let (circle, dot, text, gap) = match self.size {
            Size::Sm => (px(14.), px(5.), px(13.), px(8.)),
            Size::Md => (px(18.), px(7.), px(14.), px(10.)),
            Size::Lg => (px(22.), px(9.), px(16.), px(12.)),
        };

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
            let is_selected = self.selected == Some(i);
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

            if !self.is_disabled && !self.is_read_only {
                if let Some(on_change) = &self.on_change {
                    row = row.on_click({
                        let on_change = on_change.clone();
                        move |_, window, cx| on_change(i, window, cx)
                    });
                }
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


