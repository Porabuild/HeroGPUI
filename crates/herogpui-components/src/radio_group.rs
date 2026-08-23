//! RadioGroup — port of `@heroui/radio`.

use gpui::{prelude::*, px, App, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::{Color, FieldVariant, Orientation};
use herogpui_theme::ActiveTheme;

/// HeroUI RadioGroup.
#[derive(IntoElement)]
pub struct RadioGroup {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
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
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
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
        Some(
            crate::form::FormField::text_value(
                name,
                self.selected
                    .or(self.default_value)
                    .and_then(|i| self.options.get(i).cloned())
                    .unwrap_or_default(),
            )
            .is_required(self.is_required),
        )
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

    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
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

        // *One* handle for the whole group, because a radio group is one tab
        // stop. Which row claims it is what moves: a roving tab stop cannot be
        // done by flipping a handle's `tab_stop`, since that is fixed where the
        // handle is made. `use_keyed_state` takes `cx` mutably, so it precedes
        // the theme.
        let group_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{}-focus", element_id_name(&self.id)).into()),
            window,
            cx,
        );

        // A radio group is *one* tab stop: Tab moves past the whole group and
        // the arrows choose within it, which is the ARIA radio-group pattern
        // React Aria implements. The stop is the selected option, or the first
        // when nothing is selected yet.
        let tab_stop_index = selected.unwrap_or(0);
        let stops: Vec<usize> = (0..self.options.len()).collect();

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();
        // `.radio__control` is `size-4 rounded-lg` — a rounded square, not a
        // circle — and `.radio__indicator` fills it at `rounded-lg` too.
        // The selected dot is the indicator scaled to `0.4286` of the 16px
        // control, which v3's own comment rounds to 6px. (8px is its *pressed*
        // size, `scale: 0.5714`.)
        let (circle, dot, text, gap) = (px(16.), px(6.), px(14.), px(12.));

        // `.radio-group` spaces its options with `mt-4` when vertical and
        // `gap-4` when horizontal — 16px either way.
        let mut group = match self.orientation {
            Orientation::Horizontal => gpui::div().flex().items_center().flex_wrap().gap(px(16.)),
            Orientation::Vertical => gpui::div().flex().flex_col().gap(px(16.)),
        };
        // `.radio-group--secondary` is not a panel: it only repaints the
        // *control* with `--default` and drops its shadow. This used to wrap the
        // whole group in a padded `surface_secondary` card, which v3 has no rule
        // for.
        let control_bg = match self.variant {
            FieldVariant::Primary => colors.field.background,
            FieldVariant::Secondary => colors.default.color,
        };
        let control_shadow = (self.variant == FieldVariant::Primary
            && !layout.field_shadow.is_empty())
        .then(|| layout.field_shadow.clone());

        for (i, label) in self.options.into_iter().enumerate() {
            let is_selected = selected == Some(i);
            // `.radio__control` has no border (`--field-border-width: 0`); it is
            // a filled square. Unselected it is `bg-field` plus `shadow-field`;
            // selected it fills with `bg-accent` and the indicator shrinks to a
            // 6px `bg-accent-foreground` dot (`scale: 0.4286` of 16px).
            let mut circle_el = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{}-opt-{i}-control", element_id_name(&self.id)).into(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .size(circle)
                .rounded(crate::util::key_radius(cx))
                .flex_shrink_0()
                .bg(if is_selected { sem.color } else { control_bg })
                .when_some(control_shadow.clone(), |el, shadows| el.shadow(shadows));

            if is_selected {
                circle_el = circle_el.child(
                    gpui::div()
                        .size(dot)
                        .rounded(crate::util::key_radius(cx))
                        .bg(sem.foreground),
                );
            }
            // `status-invalid-field` is a 1px danger outline over whatever the
            // control already paints — it does not replace the fill, and v3
            // applies it whether or not the option is selected.
            if self.is_invalid {
                circle_el = circle_el.border_1().border_color(colors.danger.color);
            }

            // v3 focuses the radio and rings `.radio__control`: the row takes the
            // focus, the control shows it.
            let focused = i == tab_stop_index
                && group_focus.is_focused(window)
                && crate::util::focus_visible(cx);
            let circle_el = crate::util::with_focus_ring(
                circle_el,
                focused && !self.is_disabled,
                true,
                control_shadow.clone().unwrap_or_default(),
                cx,
            );

            // `.radio__control[data-pressed]` is `scale-95`, and a checked one
            // also fills with `bg-accent-hover`.
            let circle_el = if self.is_disabled || self.is_read_only {
                circle_el
            } else {
                let pressed_fill = sem.hover();
                let circle_el = crate::anim::pressed(
                    circle_el,
                    crate::anim::PressBox {
                        height: circle,
                        padding_x: None,
                        width: Some(circle),
                        min_width: None,
                        text_size: text,
                        line_height: text,
                        gap: px(0.),
                        radius: crate::util::key_radius(cx),
                        shrink_x: true,
                        scale: crate::anim::PRESSED_SCALE_DEEP,
                    },
                    cx,
                );
                if is_selected {
                    circle_el.active(move |s| s.bg(pressed_fill))
                } else {
                    circle_el
                }
            };

            let mut row = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{}-opt-{i}", element_id_name(&self.id)).into(),
                ))
                .when(!self.is_disabled && i == tab_stop_index, |r| {
                    r.track_focus(&group_focus)
                })
                .flex()
                .items_center()
                .gap(gap)
                .text_size(text)
                .text_color(colors.foreground)
                .when(!self.is_disabled && !self.is_read_only, |r| {
                    r.cursor_pointer()
                })
                .when(self.is_disabled, |r| r.opacity(layout.disabled_opacity))
                .child(circle_el)
                .child(label.to_string());

            if !self.is_disabled
                && !self.is_read_only
                && (self.on_change.is_some() || own.is_some())
            {
                let on_change = self.on_change.clone();
                let own = own.clone();
                // The arrows choose the next option and take the focus with
                // them, which is what makes the group one tab stop: the focused
                // radio is always the selected one.
                let key_change = on_change.clone();
                let key_own = own.clone();
                let key_stops = stops.clone();
                row = row.on_key_down(move |event, window, cx| {
                    let key = match event.keystroke.key.as_str() {
                        "down" | "right" => "down",
                        "up" | "left" => "up",
                        other @ ("home" | "end") => other,
                        _ => return,
                    };
                    // The group wraps, as a radio group does.
                    let crate::list_nav::Move::To(next) =
                        crate::list_nav::resolve(&key_stops, Some(i), key, true)
                    else {
                        return;
                    };
                    if let Some(held) = &key_own {
                        held.update(cx, |v, cx| {
                            *v = Some(next);
                            cx.notify();
                        });
                    }
                    if let Some(f) = &key_change {
                        f(next, window, cx);
                    }
                    // No refocusing: the next render has the newly selected row
                    // claim the group's handle, so the focus goes with it.
                });
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
    format!("{id:?}").trim_matches('"').to_owned()
}
