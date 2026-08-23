//! Select — port of `@heroui/select` (single selection, v1).

use gpui::{
    prelude::*, px, App, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{Color, FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{icons, util};

type OnSelectionChange = std::sync::Arc<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;

/// HeroUI Select (controlled).
#[derive(IntoElement)]
pub struct Select {
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
    /// Backs `selectionMode="multiple"`; `selected` backs `single`.
    selected_indices: std::collections::BTreeSet<usize>,
    selection_mode: SelectionMode,
    /// `isOpen` — `None` leaves the component holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    label: Option<SharedString>,
    placeholder: SharedString,
    description: Option<SharedString>,
    variant: FieldVariant,
    is_disabled: bool,
    is_invalid: bool,
    is_required: bool,
    disabled_keys: std::collections::HashSet<usize>,
    full_width: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[usize], &mut Window, &mut App) + 'static>>,
}

impl Select {
    /// `onChange` — the v3 name for [`Select::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    /// `disabledKeys` — indices that cannot be chosen.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = usize>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
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

    /// `fullWidth` — stretch to the container width.
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
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
            selected_indices: std::collections::BTreeSet::new(),
            selection_mode: SelectionMode::Single,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            label: None,
            placeholder: "Select an option".into(),
            description: None,
            variant: FieldVariant::Primary,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            disabled_keys: std::collections::HashSet::new(),
            full_width: false,
            on_open_change: None,
            on_selection_change: None,
            on_selection_change_all: None,
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

    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// The chosen indices under `selectionMode="multiple"`.
    pub fn selected_indices(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.selected_indices = indices.into_iter().collect();
        self
    }

    /// Reports the whole selection, for `selectionMode="multiple"`.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[usize], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(std::sync::Arc::new(handler));
        self
    }

    /// `value` — the selected option, by index. Supplying it makes the select
    /// controlled, even with `None`.
    pub fn value(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self.is_controlled = true;
        self
    }

    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Only consulted when `value` is not supplied; the select then owns the
    /// selection and choosing an option moves it.
    pub fn default_value(mut self, i: Option<usize>) -> Self {
        self.default_value = i;
        self
    }

    /// `placement` on the popover.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }
    /// `defaultOpen` — the uncontrolled initial state.
    ///
    /// Only consulted when `is_open` is not supplied; the component then owns
    /// the flag and its trigger toggles it.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn variant(mut self, v: FieldVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_open_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_selection_change(
        mut self,
        f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl Select {
    fn value_text_single(&self, selected: Option<usize>) -> SharedString {
        selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_else(|| self.placeholder.clone())
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-open", self.id).into()),
            self.is_open,
            self.default_open,
        );
        let (selected, value_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-value", self.id).into()),
            self.is_controlled.then_some(self.selected),
            self.default_value,
        );

        let sem = cx.role(Color::Accent);
        let colors = cx.colors();
        let layout = cx.layout();

        // `.select__trigger` is `min-h-9 ... text-sm`.
        let (h, text) = (util::FIELD_HEIGHT, util::FIELD_TEXT);

        let trigger_id = el_name(format!("select-{}", id_debug(&self.id)));
        let mut field = gpui::div()
            .id(trigger_id)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .min_h(h)
            .px(px(12.))
            .text_size(text)
            .cursor_pointer();

        let _border_color = if is_open { sem.color } else { colors.separator };
        field = util::apply_field_chrome(field, self.variant, self.is_invalid, false, cx);

        if !self.is_disabled {
            let hover_bg = match self.variant {
                FieldVariant::Primary => colors.field.hover(),
                FieldVariant::Secondary => colors.default.soft_hover(),
            };
            field = field.hover(move |s| s.bg(hover_bg));
        } else {
            field = field.opacity(layout.disabled_opacity);
        }

        if self.full_width {
            field = field.w_full();
        }

        let multiple = self.selection_mode == SelectionMode::Multiple;
        let value_text = if multiple {
            let names: Vec<String> = self
                .selected_indices
                .iter()
                .filter_map(|i| self.options.get(*i).map(ToString::to_string))
                .collect();
            if names.is_empty() {
                self.placeholder.clone()
            } else if names.len() <= 2 {
                SharedString::from(names.join(", "))
            } else {
                // Long selections would overflow the trigger.
                SharedString::from(format!("{} selected", names.len()))
            }
        } else {
            self.value_text_single(selected)
        };
        let has_value = if multiple {
            !self.selected_indices.is_empty()
        } else {
            selected.is_some()
        };

        field = field
            .child(
                gpui::div()
                    .flex_1()
                    .truncate()
                    .text_color(if has_value {
                        colors.foreground
                    } else {
                        colors.muted
                    })
                    .child(value_text.to_string()),
            )
            .child(
                gpui::svg()
                    .size(px(16.))
                    .path(if is_open {
                        icons::CHEVRON_UP
                    } else {
                        icons::CHEVRON_DOWN
                    })
                    .text_color(colors.muted)
                    .flex_shrink_0(),
            );

        if !self.is_disabled && (self.on_open_change.is_some() || open_own.is_some()) {
            let on_open_change = self.on_open_change.clone();
            let own = open_own.clone();
            let open = is_open;
            field = field.on_click(move |_, window, cx| {
                // Uncontrolled: flip our own copy, or the trigger would be
                // inert without a caller handler.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = !open;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_open_change {
                    cb(!open, window, cx);
                }
            });
        }

        // listbox panel
        let mut root = gpui::div().relative().max_w(px(320.));
        if self.label.is_some() || self.description.is_some() {
            let mut wrapper = gpui::div().flex().flex_col().gap(px(4.)).w_full();
            // Reuse the shared field slots so the required/invalid/disabled
            // treatments match every other control.
            if let Some(label) = &self.label {
                wrapper = wrapper.child(
                    crate::field::Label::new(label.clone())
                        .is_required(self.is_required)
                        .is_disabled(self.is_disabled)
                        .is_invalid(self.is_invalid),
                );
            }
            wrapper = wrapper.child(field);
            if let Some(desc) = &self.description {
                wrapper = wrapper.child(crate::field::Description::new(desc.clone()));
            }
            root = root.child(wrapper);
        } else {
            root = root.child(field);
        }

        if is_open && !self.options.is_empty() {
            let base = format!("select-list-{}", id_debug(&self.id));
            let mut panel = gpui::div()
                .w_full()
                .flex()
                .flex_col()
                .py(px(6.))
                .bg(colors.overlay.background)
                .rounded(util::container_radius(cx))
                .border_1()
                .border_color(colors.separator)
                .shadow(layout.overlay_shadow.clone())
                .overflow_hidden()
                .max_h(px(280.));

            for (i, opt) in self.options.iter().enumerate() {
                let is_sel = if multiple {
                    self.selected_indices.contains(&i)
                } else {
                    selected == Some(i)
                };
                let opt_disabled = self.disabled_keys.contains(&i);
                let mut item = gpui::div()
                    .id(el_name(format!("{base}-opt-{i}")))
                    .flex()
                    .items_center()
                    .justify_between()
                    // Every menu row in v3 is a `.list-box-item`: `min-h-9
                    // rounded-2xl px-2 py-1.5 gap-3` at `text-sm`.
                    .min_h(util::FIELD_HEIGHT)
                    .rounded(util::soft_radius(cx))
                    .px(px(8.))
                    .py(px(6.))
                    .gap(px(12.))
                    .text_size(util::FIELD_TEXT);

                if opt_disabled {
                    item = item.opacity(layout.disabled_opacity);
                } else {
                    item = item
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.default.soft()));
                }

                if is_sel {
                    item = item
                        .text_color(sem.color)
                        .font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    item = item.text_color(colors.foreground);
                }

                item = item.child(gpui::div().truncate().child(opt.to_string()));

                if is_sel {
                    item = item.child(
                        gpui::svg()
                            .size(px(13.))
                            .path(icons::CHECK)
                            .text_color(sem.color),
                    );
                }

                if !opt_disabled {
                    if multiple {
                        if let Some(cb) = self.on_selection_change_all.clone() {
                            let current = self.selected_indices.clone();
                            item = item.on_click(move |_, window, cx| {
                                let mut next = current.clone();
                                if !next.remove(&i) {
                                    next.insert(i);
                                }
                                let next: Vec<usize> = next.into_iter().collect();
                                cb(&next, window, cx);
                            });
                        }
                    } else if self.on_selection_change.is_some()
                        || value_own.is_some()
                        || open_own.is_some()
                    {
                        let on_select = self.on_selection_change.clone();
                        let value_own = value_own.clone();
                        let open_own = open_own.clone();
                        item = item.on_click(move |_, window, cx| {
                            // Uncontrolled: take the selection and close, or
                            // choosing an option would do nothing.
                            if let Some(held) = &value_own {
                                held.update(cx, |v, cx| {
                                    *v = Some(i);
                                    cx.notify();
                                });
                            }
                            if let Some(held) = &open_own {
                                held.update(cx, |v, cx| {
                                    *v = false;
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &on_select {
                                f(Some(i), window, cx);
                            }
                        });
                    }
                }

                panel = panel.child(item);
            }

            root = root.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(crate::anim::entering_zoom(
                    panel,
                    el_name(format!("{base}-panel")),
                    crate::anim::ZoomBox::panel(px(6.), util::container_radius(cx)),
                    crate::anim::Motion::LIST_IN,
                    cx,
                )),
            ));
        }

        root
    }
}

fn el_name(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

fn id_debug(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_owned()
}
