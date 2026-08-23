//! Autocomplete — port of `@heroui/autocomplete`.
//!
//! Reuses [`InputState`]; the suggestion menu opens automatically while the
//! field is focused and the query matches items.

use gpui::{
    prelude::*, px, App, Entity, IntoElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use herogpui_core::{FieldVariant, Placement, SelectionMode};
use herogpui_theme::ActiveTheme;

use crate::{
    icons,
    input::{Input, InputState},
    util,
};

type OnSelectionChange = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// HeroUI Autocomplete.
#[derive(IntoElement)]
pub struct Autocomplete {
    /// `name` — the name this control submits under; read back by
    /// [`Self::form_field`].
    name: Option<SharedString>,
    state: Entity<InputState>,
    items: Vec<SharedString>,
    max_items: usize,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    variant: FieldVariant,
    full_width: bool,
    is_disabled: bool,
    is_read_only: bool,
    is_invalid: bool,
    is_required: bool,
    /// `disabledKeys` — suggestions that render but cannot be chosen.
    disabled_keys: std::collections::HashSet<SharedString>,
    /// `ListBox.Section` — a heading above the item with this label.
    sections: Vec<(SharedString, SharedString)>,
    /// `ListBox.ItemIndicator` — draws the tick. The closure is handed whether
    /// the row is selected.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    /// `allowsEmptyCollection` — keep the panel open with an empty state
    /// instead of hiding it when nothing matches.
    allows_empty_collection: bool,
    selection_mode: SelectionMode,
    selected_keys: std::collections::BTreeSet<SharedString>,
    /// `defaultValue` — set it to hand this component its own selection.
    default_value: Option<std::collections::BTreeSet<SharedString>>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
    /// `isOpen`. `None` follows focus, which is the v3 default behaviour.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    /// `filter` — decides whether an item matches the query. Defaults to a
    /// case-insensitive substring test.
    filter: Option<std::sync::Arc<dyn Fn(&str, &str) -> bool + 'static>>,
    input_value: Option<String>,
    on_input_change: Option<std::sync::Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_clear: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl Autocomplete {
    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// The chosen item labels under `selectionMode="multiple"`.
    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Supplying it hands the component its own selection set, seeded once;
    /// `selected_keys` is the controlled spelling.
    pub fn default_value(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_value = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self
    }

    /// Reports the whole selection, for `selectionMode="multiple"`.
    pub fn on_selection_change_all(
        mut self,
        handler: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change_all = Some(std::sync::Arc::new(handler));
        self
    }

    /// `value` — writes the query through to the bound state.
    pub fn value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_value(value));
        self
    }

    /// `isOpen` — forces the suggestion panel open regardless of focus.
    /// `placement` on `Autocomplete.Popover`.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// `defaultOpen` — the popover starts open, until the field takes focus.
    ///
    /// After the first focus the popover follows focus, so a blur closes it;
    /// the seed only covers the render before the user has engaged.
    pub fn default_open(mut self, v: bool) -> Self {
        self.default_open = v;
        self
    }

    pub fn is_open(mut self, v: bool) -> Self {
        self.is_open = Some(v);
        self
    }

    /// `onOpenChange`
    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(handler));
        self
    }

    /// `filter` — replaces the default case-insensitive substring match.
    ///
    /// Called as `filter(item_text, input)`.
    pub fn filter(mut self, f: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.filter = Some(std::sync::Arc::new(f));
        self
    }

    /// `inputValue` — the controlled query text.
    ///
    /// Unlike [`Autocomplete::value`] this does not write through to the bound
    /// state, so a caller can hold the query itself.
    pub fn input_value(mut self, value: impl Into<String>) -> Self {
        self.input_value = Some(value.into());
        self
    }

    /// `onInputChange` — fires on every keystroke in the query field.
    pub fn on_input_change(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_input_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onClick` on `Autocomplete.ClearButton` — supplying it renders the clear
    /// affordance, so the button never appears without a handler behind it.
    pub fn on_clear(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(std::sync::Arc::new(f));
        self
    }

    /// `onChange` — the v3 name for [`Autocomplete::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    pub fn new(state: Entity<InputState>, items: Vec<SharedString>) -> Self {
        Self {
            name: None,
            state,
            items,
            max_items: 8,
            label: None,
            placeholder: None,
            description: None,
            error_message: None,
            variant: FieldVariant::Primary,
            full_width: false,
            is_disabled: false,
            is_read_only: false,
            is_invalid: false,
            is_required: false,
            disabled_keys: std::collections::HashSet::new(),
            sections: Vec::new(),
            indicator: None,
            allows_empty_collection: false,
            selection_mode: SelectionMode::Single,
            selected_keys: std::collections::BTreeSet::new(),
            default_value: None,
            on_selection_change_all: None,
            filter: None,
            input_value: None,
            on_input_change: None,
            on_clear: None,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            on_open_change: None,
            on_selection_change: None,
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
            crate::form::FormField::keys(name, self.selected_keys.clone())
                .is_required(self.is_required),
        )
    }

    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = n.max(1);
        self
    }

    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
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

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
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

    /// `disabledKeys`
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// `allowsEmptyCollection`
    /// `ListBox.Section` — a heading rendered above `item`.
    pub fn section_before(
        mut self,
        item: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.sections.push((item.into(), label.into()));
        self
    }

    /// `ListBox.ItemIndicator` — draw the selected tick yourself.
    pub fn indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.indicator = Some(Box::new(render));
        self
    }

    pub fn allows_empty_collection(mut self, v: bool) -> Self {
        self.allows_empty_collection = v;
        self
    }

    pub fn on_selection_change(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }
}

fn el_name(s: String) -> gpui::ElementId {
    gpui::ElementId::Name(s.into())
}

impl RenderOnce for Autocomplete {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (selection, selection_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(
                format!("autocomplete-{}-selection", self.state.entity_id().as_u64()).into(),
            ),
            match self.default_value {
                Some(_) => None,
                None => Some(self.selected_keys.clone()),
            },
            self.default_value.clone().unwrap_or_default(),
        );
        self.selected_keys = selection;

        // The `defaultOpen` seed: `isOpen` wins over it, then focus, and the
        // first focus spends the seed so a later blur closes the popover
        // instead of leaving it pinned open. `use_keyed_state` takes `cx`
        // mutably, so this precedes the theme tokens.
        let focused = self.state.read(cx).focus_handle.is_focused(window);
        let seed = window.use_keyed_state(
            gpui::ElementId::Name(
                format!("autocomplete-{:?}-default-open", self.state.entity_id()).into(),
            ),
            cx,
            {
                let default_open = self.default_open;
                move |_, _| default_open
            },
        );
        let seeded = *seed.read(cx);
        if focused && seeded {
            seed.update(cx, |v, _| *v = false);
        }

        let colors = cx.colors();
        let layout = cx.layout();

        // A controlled `inputValue` wins over whatever the entity holds.
        let raw_query = match &self.input_value {
            Some(v) => v.clone(),
            None => self.state.read(cx).value().to_owned(),
        };
        let query = raw_query.to_lowercase();

        let open = self.is_open.unwrap_or(focused || seeded);
        let multiple = self.selection_mode == SelectionMode::Multiple;
        let matches: Vec<SharedString> = if open {
            let custom = self.filter.clone();
            self.items
                .iter()
                .filter(|it| match &custom {
                    // A custom filter owns the whole decision, including what
                    // an empty query means.
                    Some(f) => f(it.as_ref(), &raw_query),
                    None => !query.is_empty() && it.to_lowercase().contains(&query),
                })
                .take(self.max_items)
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        let is_invalid = self.is_invalid || self.error_message.is_some();
        let hover_bg = colors.default.soft_hover();
        let mut input = Input::new(self.state.clone())
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_read_only(self.is_read_only)
            .is_invalid(is_invalid)
            .is_required(self.is_required)
            .end_content({
                let mut trailing = gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .flex_shrink_0();

                // The clear affordance only exists when a handler backs it, and
                // only when there is something to clear.
                if let Some(cb) = self.on_clear.clone() {
                    if !raw_query.is_empty() {
                        trailing = trailing.child(
                            gpui::div()
                                .id(el_name("autocomplete-clear".to_owned()))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(18.))
                                .rounded_full()
                                .cursor_pointer()
                                .hover(|st| st.bg(hover_bg))
                                .child(
                                    gpui::svg()
                                        .size(px(11.))
                                        .path(icons::CLOSE)
                                        .text_color(colors.muted),
                                )
                                .on_click(move |_, window, cx| cb(window, cx)),
                        );
                    }
                }

                let mut chevron = gpui::div()
                    .id(el_name("autocomplete-trigger".to_owned()))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(
                        gpui::svg()
                            .size(px(14.))
                            .path(icons::CHEVRON_DOWN)
                            .text_color(colors.muted),
                    );
                if let Some(cb) = self.on_open_change.clone() {
                    let next = !open;
                    chevron = chevron
                        .cursor_pointer()
                        .on_click(move |_, window, cx| cb(next, window, cx));
                }
                trailing.child(chevron)
            });

        // `onInputChange` reports every keystroke in the query field.
        if let Some(cb) = self.on_input_change.clone() {
            input = input.on_change(move |text, window, cx| cb(text, window, cx));
        }
        if let Some(label) = &self.label {
            input = input.label(label.clone());
        }
        if let Some(ph) = &self.placeholder {
            input = input.placeholder(ph.clone());
        }
        if self.full_width {
            input = input.full_width();
        }
        if is_invalid {
            if let Some(message) = &self.error_message {
                input = input.error_message(message.clone());
            }
        } else if let Some(description) = &self.description {
            input = input.description(description.clone());
        }

        let mut root = gpui::div().relative().child(input);
        root = if self.full_width {
            root.w_full()
        } else {
            root.max_w(px(320.))
        };

        // `allowsEmptyCollection` keeps the panel up with an empty state.
        let show_panel =
            !self.is_disabled && open && (!matches.is_empty() || self.allows_empty_collection);
        if show_panel {
            let base = "autocomplete-list";
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
                .overflow_hidden();

            for item in &matches {
                // `ListBox.Section`'s `Header`, above the item it introduces.
                if let Some((_, label)) = self.sections.iter().find(|(at, _)| at == item) {
                    panel = panel.child(
                        gpui::div()
                            .px(px(8.))
                            .pt(px(6.))
                            .pb(px(2.))
                            .text_size(px(12.))
                            .text_color(colors.muted)
                            .child(label.to_string()),
                    );
                }
                let item_disabled = self.disabled_keys.contains(item);
                let mut row = gpui::div()
                    .id(el_name(format!("{base}-{item}")))
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
                    .text_size(util::FIELD_TEXT)
                    .text_color(colors.foreground)
                    .child(item.to_string());

                if item_disabled {
                    row = row.opacity(layout.disabled_opacity);
                } else {
                    row = row
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.default.soft()));
                }

                // A multiple selection check-marks every chosen item, unless
                // `ListBox.ItemIndicator` is drawn by the caller.
                let row_selected = self.selected_keys.contains(item);
                match &self.indicator {
                    Some(render) => row = row.child(render(row_selected)),
                    None if multiple && row_selected => {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                .text_color(colors.accent.color),
                        );
                    }
                    None => {}
                }

                if multiple && !item_disabled {
                    if self.on_selection_change_all.is_some() || selection_own.is_some() {
                        let cb = self.on_selection_change_all.clone();
                        let own = selection_own.clone();
                        let current = self.selected_keys.clone();
                        let value = item.clone();
                        row = row.on_click(move |_, window, cx| {
                            let mut next = current.clone();
                            if !next.remove(&value) {
                                next.insert(value.clone());
                            }
                            // Uncontrolled: keep the new set, or picking an
                            // item would do nothing.
                            if let Some(held) = &own {
                                let set = next.clone();
                                held.update(cx, |v, cx| {
                                    *v = set;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &cb {
                                let next: Vec<SharedString> = next.into_iter().collect();
                                cb(&next, window, cx);
                            }
                        });
                    }
                    panel = panel.child(row);
                    continue;
                }

                if let Some(on_select) = self.on_selection_change.clone().filter(|_| !item_disabled)
                {
                    let value = item.clone();
                    let state = self.state.clone();
                    row = row.on_click(move |_, window, cx| {
                        state.update(cx, |s, sc| {
                            s.set_value(value.to_string());
                            sc.notify();
                        });
                        on_select(&value, window, cx);
                    });
                }

                panel = panel.child(row);
            }

            if matches.is_empty() {
                panel = panel.child(
                    gpui::div()
                        .px(px(12.))
                        .py(px(6.))
                        .text_size(util::FIELD_TEXT)
                        .text_color(colors.muted)
                        .child("No matching options"),
                );
            }

            root = root.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(crate::anim::entering_zoom(
                    panel,
                    el_name("autocomplete-panel".to_owned()),
                    crate::anim::ZoomBox::panel(px(6.), util::container_radius(cx)),
                    crate::anim::Motion::FLUID_IN,
                    cx,
                )),
            ));
        }

        root
    }
}
