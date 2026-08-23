//! ComboBox — port of `@heroui/combo-box` (v3).
//!
//! A text input combined with a selectable list. Unlike
//! [`Select`](crate::select::Select) the value is typed, and unlike
//! [`Autocomplete`](crate::autocomplete::Autocomplete) the list can be opened
//! without typing and `allowsCustomValue` decides whether input outside the
//! collection is accepted.

use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, Entity, InteractiveElement, IntoElement, RenderOnce, SharedString,
    Styled, Window,
};
use herogpui_core::{FieldVariant, Placement, SelectionMode, Size};
use herogpui_theme::ActiveTheme;

use crate::{
    icons,
    input::{Input, InputState},
    util,
};

/// When the suggestion list opens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuTrigger {
    /// Open on input, and when the trigger button is pressed.
    #[default]
    Input,
    /// Open only when the trigger button is pressed.
    Manual,
    /// Open as soon as the field gains focus.
    Focus,
}

type OnSelectionChange = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
type OnOpenChange = Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// HeroUI ComboBox (controlled open state).
#[derive(IntoElement)]
pub struct ComboBox {
    state: Entity<InputState>,
    items: Vec<SharedString>,
    /// `isOpen` — `None` leaves the component holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    placement: Placement,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    description: Option<SharedString>,
    error_message: Option<SharedString>,
    variant: FieldVariant,
    size: Size,
    menu_trigger: MenuTrigger,
    /// `defaultFilter` — decides whether an item matches the query.
    filter: Option<std::sync::Arc<dyn Fn(&str, &str) -> bool + 'static>>,
    allows_custom_value: bool,
    max_items: usize,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    disabled_keys: std::collections::HashSet<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: std::collections::BTreeSet<SharedString>,
    on_selection_change_all:
        Option<std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
    on_input_change: Option<Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_open_change: Option<OnOpenChange>,
}

impl ComboBox {
    /// `selectionMode`
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// The chosen item labels under `selectionMode="multiple"`.
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

    /// `selectedKey` — writes the chosen item through to the bound state.
    pub fn selected_key(self, key: impl Into<String>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_value(key));
        self
    }

    /// `value` — the v3 alias of [`ComboBox::input_value`].
    pub fn value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.input_value(value, cx)
    }

    /// `disabledKeys` — items that render but cannot be chosen.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    /// `inputValue` — writes the typed text through to the bound state.
    pub fn input_value(self, value: impl Into<String>, cx: &mut App) -> Self {
        self.state.update(cx, |s, _| s.set_value(value));
        self
    }

    /// `onInputChange` — fires as the text changes, where
    /// [`ComboBox::on_selection_change`] fires only on a pick.
    pub fn on_input_change(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_input_change = Some(Arc::new(handler));
        self
    }

    /// `onChange` — the v3 name for [`ComboBox::on_selection_change`].
    pub fn on_change(
        self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change(handler)
    }

    pub fn new(state: Entity<InputState>, items: Vec<SharedString>) -> Self {
        Self {
            state,
            items,
            is_open: None,
            default_open: false,
            placement: Placement::BottomStart,
            label: None,
            placeholder: None,
            description: None,
            error_message: None,
            variant: FieldVariant::Primary,
            size: Size::Md,
            menu_trigger: MenuTrigger::Input,
            filter: None,
            allows_custom_value: false,
            max_items: 8,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            disabled_keys: std::collections::HashSet::new(),
            selection_mode: SelectionMode::Single,
            selected_keys: std::collections::BTreeSet::new(),
            on_selection_change_all: None,
            on_input_change: None,
            on_selection_change: None,
            on_open_change: None,
        }
    }

    /// `placement` on `ComboBox.Popover`.
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

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
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

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn menu_trigger(mut self, trigger: MenuTrigger) -> Self {
        self.menu_trigger = trigger;
        self
    }

    /// `defaultFilter` — replaces the default case-insensitive substring
    /// match.
    ///
    /// Called as `filter(item_text, input)`, and owns the whole decision —
    /// including what an empty query means.
    pub fn filter(mut self, f: impl Fn(&str, &str) -> bool + 'static) -> Self {
        self.filter = Some(std::sync::Arc::new(f));
        self
    }

    /// `allowsCustomValue` — accept text that matches no item.
    pub fn allows_custom_value(mut self, v: bool) -> Self {
        self.allows_custom_value = v;
        self
    }

    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = n.max(1);
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

    pub fn is_required(mut self, v: bool) -> Self {
        self.is_required = v;
        self
    }

    pub fn on_selection_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Arc::new(handler));
        self
    }

    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ComboBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (open_state, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(
                format!("combobox-{}-open", self.state.entity_id().as_u64()).into(),
            ),
            self.is_open,
            self.default_open,
        );

        // Owned copies: `input.render` below needs `cx` mutably.
        let colors = cx.colors().clone();
        let layout = cx.layout().clone();
        let container_radius = util::container_radius(cx);
        let control_radius = util::control_radius(cx);
        let entity_id = self.state.entity_id().as_u64();
        let raw_query = self.state.read(cx).value().to_string();
        let query = raw_query.to_lowercase();
        let is_invalid = self.is_invalid || self.error_message.is_some();
        let multiple = self.selection_mode == SelectionMode::Multiple;

        // `Manual` shows the full collection; the typing triggers filter it.
        // A custom `defaultFilter` owns the whole decision, so it also runs on
        // an empty query.
        let custom = self.filter.clone();
        let matches: Vec<SharedString> = match &custom {
            Some(f) if self.menu_trigger != MenuTrigger::Manual => self
                .items
                .iter()
                .filter(|item| f(item.as_ref(), &raw_query))
                .take(self.max_items)
                .cloned()
                .collect(),
            _ if self.menu_trigger == MenuTrigger::Manual || query.is_empty() => {
                self.items.iter().take(self.max_items).cloned().collect()
            }
            _ => self
                .items
                .iter()
                .filter(|item| item.to_lowercase().contains(&query))
                .take(self.max_items)
                .cloned()
                .collect(),
        };

        let on_open_change = self.on_open_change.clone();
        let is_open = open_state;
        let mut trigger = div()
            .id(gpui::ElementId::Name(
                format!("combobox-{entity_id}-trigger").into(),
            ))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .size(px(20.))
            .rounded(px(6.))
            .text_color(colors.muted)
            .child(
                gpui::svg()
                    .size(self.size.icon_size())
                    .path(icons::CHEVRON_DOWN)
                    .text_color(colors.muted),
            );
        if !self.is_disabled {
            let hover_bg = colors.default.hover();
            trigger = trigger.cursor_pointer().hover(move |s| s.bg(hover_bg));
            if on_open_change.is_some() || open_own.is_some() {
                let own = open_own.clone();
                trigger = trigger.on_click(move |_, window, cx| {
                    // Uncontrolled: flip our own copy, or the chevron would be
                    // inert without a caller handler.
                    if let Some(held) = &own {
                        held.update(cx, |v, cx| {
                            *v = !is_open;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &on_open_change {
                        cb(!is_open, window, cx);
                    }
                });
            }
        }

        let mut input = Input::new(self.state.clone())
            .variant(self.variant)
            .size(self.size)
            .is_disabled(self.is_disabled)
            .is_invalid(is_invalid)
            .is_required(self.is_required)
            .is_read_only(self.is_read_only)
            .end_content(trigger);
        if let Some(cb) = self.on_input_change.clone() {
            input = input.on_change(move |text, window, cx| cb(text, window, cx));
        }

        if self.full_width {
            input = input.full_width();
        }
        if let Some(label) = self.label.clone() {
            input = input.label(label);
        }
        if let Some(placeholder) = self.placeholder.clone() {
            input = input.placeholder(placeholder);
        }
        if is_invalid {
            if let Some(message) = self.error_message.clone() {
                input = input.error_message(message);
            }
        } else if let Some(description) = self.description.clone() {
            input = input.description(description);
        }

        let mut root = div()
            // The panel overlays the page rather than pushing it down, so the
            // root has to be its positioning context.
            .relative()
            // Without a placeholder the inner input has no intrinsic width and
            // the trigger collapses to just its chevron, which is unclickable.
            .when(!self.full_width, |e| e.min_w(px(180.)))
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(input.render(window, cx));

        let show_list = open_state && !self.is_disabled;
        if show_list {
            let mut panel = div()
                .id(gpui::ElementId::Name(
                    format!("combobox-{entity_id}-panel").into(),
                ))
                .w_full()
                .flex()
                .flex_col()
                .gap(px(2.))
                .p(px(4.))
                .max_h(px(240.))
                .overflow_y_scroll()
                .rounded(container_radius)
                .bg(colors.overlay.background)
                .border(layout.border_width)
                .border_color(colors.border)
                .text_color(colors.overlay.foreground)
                .when(!layout.overlay_shadow.is_empty(), |e: gpui::Stateful<gpui::Div>| {
                    e.shadow(layout.overlay_shadow.clone())
                });

            if matches.is_empty() {
                // `allowsCustomValue` means an unmatched query is still valid,
                // so the empty state has to say something different.
                let message = if self.allows_custom_value {
                    "Press Enter to use this value"
                } else {
                    "No matching options"
                };
                panel = panel.child(
                    div()
                        .px(px(8.))
                        .py(px(6.))
                        .text_size(self.size.text_size())
                        .text_color(colors.muted)
                        .child(message),
                );
            }

            for (index, item) in matches.iter().enumerate() {
                let item_disabled = self.disabled_keys.contains(item);
                let hover_bg = colors.default.color;
                let mut row = div()
                    .id(gpui::ElementId::Name(
                        format!("combobox-{entity_id}-item-{index}").into(),
                    ))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(control_radius)
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_size(self.size.text_size())
                    .child(item.to_string());

                if item_disabled {
                    row = row.opacity(layout.disabled_opacity);
                } else {
                    row = row.cursor_pointer().hover(move |s| s.bg(hover_bg));
                }

                if multiple && self.selected_keys.contains(item) {
                    row = row.child(
                        gpui::svg()
                            .size(px(13.))
                            .path(icons::CHECK)
                            .text_color(colors.accent.color),
                    );
                }

                if item_disabled {
                    panel = panel.child(row);
                    continue;
                }

                // Multiple mode toggles membership and leaves the panel open.
                if multiple {
                    if let Some(cb) = self.on_selection_change_all.clone() {
                        let current = self.selected_keys.clone();
                        let value = item.clone();
                        row = row.on_click(move |_, window, cx| {
                            let mut next = current.clone();
                            if !next.remove(&value) {
                                next.insert(value.clone());
                            }
                            let next: Vec<SharedString> = next.into_iter().collect();
                            cb(&next, window, cx);
                        });
                    }
                    panel = panel.child(row);
                    continue;
                }

                let value = item.clone();
                let state = self.state.clone();
                let on_selection_change = self.on_selection_change.clone();
                let on_open_change = self.on_open_change.clone();
                row = row.on_click(move |_, window, cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(value.to_string());
                        cx.notify();
                    });
                    if let Some(cb) = &on_selection_change {
                        cb(&value, window, cx);
                    }
                    if let Some(cb) = &on_open_change {
                        cb(false, window, cx);
                    }
                });

                panel = panel.child(row);
            }

            root = root.child(crate::util::floating(
                crate::util::placed_field_panel(self.placement, px(6.)).child(
                    crate::anim::entering(
                        panel,
                        gpui::ElementId::Name(format!("combobox-{entity_id}-anim").into()),
                        cx,
                    ),
                ),
            ));
        }

        root
    }
}
