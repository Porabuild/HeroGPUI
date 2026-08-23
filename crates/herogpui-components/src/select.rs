//! Select — port of `@heroui/select` (single selection, v1).

use gpui::{
    prelude::*, px, App, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_core::{Color, FieldVariant, Placement, SelectionMode, Size};
use herogpui_theme::ActiveTheme;

use crate::icons;

type OnSelectionChange = std::sync::Arc<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;

/// HeroUI Select (controlled).
#[derive(IntoElement)]
pub struct Select {
    id: gpui::ElementId,
    options: Vec<SharedString>,
    selected: Option<usize>,
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
    size: Size,
    variant: FieldVariant,
    color: Color,
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
    /// `value` — the v3 name for [`Select::selected`].
    pub fn value(self, index: Option<usize>) -> Self {
        self.selected(index)
    }

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
            id: id.into(),
            options,
            selected: None,
            selected_indices: std::collections::BTreeSet::new(),
            selection_mode: SelectionMode::Single,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            label: None,
            placeholder: "Select an option".into(),
            description: None,
            size: Size::Md,
            variant: FieldVariant::Primary,
            color: Color::Default,
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

    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
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

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }


    pub fn variant(mut self, v: FieldVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_open_change(
        mut self,
        f: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
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
    fn value_text_single(&self) -> SharedString {
        self.selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_else(|| self.placeholder.clone())
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{:?}-open", self.id).into()),
            self.is_open,
            self.default_open,
        );

        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();

        let (h, text) = match self.size {
            Size::Sm => (px(32.), px(13.)),
            Size::Md => (px(40.), px(14.)),
            Size::Lg => (px(48.), px(16.)),
        };

        let trigger_id = el_name(format!("select-{}", id_debug(&self.id)));
        let mut field = gpui::div()
            .id(trigger_id)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .h(h)
            .px(px(12.))
            .text_size(text)
            .rounded(crate::util::control_radius(cx))
            .cursor_pointer();

        let _border_color = if is_open { sem.color } else { colors.separator };
        field = match self.variant {
            FieldVariant::Primary => {
                let shadow = cx.layout().field_shadow.clone();
                field
                    .bg(colors.field.background)
                    .when(!shadow.is_empty(), |e| e.shadow(shadow))
            }
            FieldVariant::Secondary => field.bg(colors.surface_secondary),
        };

        if self.is_invalid {
            field = field.border_1().border_color(colors.danger.color);
        }

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
                .filter_map(|i| self.options.get(*i).map(|o| o.to_string()))
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
            self.value_text_single()
        };
        let has_value = if multiple {
            !self.selected_indices.is_empty()
        } else {
            self.selected.is_some()
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
                    .path(if is_open { icons::CHEVRON_UP } else { icons::CHEVRON_DOWN })
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
                .bg(colors.surface.background)
                .rounded(px(12.))
                .border_1()
                .border_color(colors.separator)
                .shadow(layout.overlay_shadow.clone())
                .overflow_hidden()
                .max_h(px(280.));

            for (i, opt) in self.options.iter().enumerate() {
                let is_sel = if multiple {
                    self.selected_indices.contains(&i)
                } else {
                    self.selected == Some(i)
                };
                let opt_disabled = self.disabled_keys.contains(&i);
                let mut item = gpui::div()
                    .id(el_name(format!("{base}-opt-{i}")))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .h(px(34.))
                    .text_size(px(13.5));

                if opt_disabled {
                    item = item.opacity(layout.disabled_opacity);
                } else {
                    item = item
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.default.soft()));
                }

                if is_sel {
                    item = item.text_color(sem.color).font_weight(gpui::FontWeight::MEDIUM);
                } else {
                    item = item.text_color(colors.foreground);
                }

                item = item.child(gpui::div().truncate().child(opt.to_string()));

                if is_sel {
                    item = item.child(
                        gpui::svg().size(px(13.)).path(icons::CHECK).text_color(sem.color),
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
                    } else if let Some(on_select) = self.on_selection_change.clone() {
                        item = item.on_click(move |_, window, cx| on_select(Some(i), window, cx));
                    }
                }

                panel = panel.child(item);
            }

            root = root.child(crate::util::floating(
                crate::util::placed_field_panel(self.placement, px(6.)).child(
                    crate::anim::entering(panel, el_name(format!("{base}-panel")), cx),
                ),
            ));
        }

        root
    }
}

fn el_name(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

fn id_debug(id: &gpui::ElementId) -> String {
    format!("{id:?}").trim_matches('"').to_string()
}



