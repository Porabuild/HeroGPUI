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
use herogpui_core::{FieldVariant, Placement, SelectionMode};
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
    menu_trigger: MenuTrigger,
    /// `defaultFilter` — decides whether an item matches the query.
    filter: Option<Arc<dyn Fn(&str, &str) -> bool + 'static>>,
    allows_custom_value: bool,
    max_items: usize,
    full_width: bool,
    is_disabled: bool,
    is_invalid: bool,
    is_required: bool,
    is_read_only: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `ListLayout`'s `rowHeight`, which virtualizes the popover list.
    row_height: Option<gpui::Pixels>,
    /// `validate` — run by the component, not the caller.
    validate: Option<crate::validation::Validator<str>>,
    /// `validationBehavior` — carried on the inner field.
    validation_behavior: Option<crate::form::ValidationBehavior>,
    /// `allowsEmptyCollection` — keeps the panel up with no matches.
    allows_empty_collection: bool,
    /// `name` — the name this field submits under.
    name: Option<SharedString>,
    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    should_focus_wrap: bool,
    /// `ListBox.Section` — a heading above the item with this label.
    sections: Vec<(SharedString, SharedString)>,
    /// `ListBox.ItemIndicator` — draws the tick. The closure is handed whether
    /// the row is the selected one.
    indicator: Option<Box<dyn Fn(bool) -> gpui::AnyElement + 'static>>,
    disabled_keys: std::collections::HashSet<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: std::collections::BTreeSet<SharedString>,
    /// `defaultValue` — set it to hand this component its own selection.
    default_value: Option<std::collections::BTreeSet<SharedString>>,
    /// `defaultInputValue` — seeds the text state on the first render only.
    default_input_value: Option<SharedString>,
    on_selection_change_all: Option<Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>>,
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
    /// `defaultValue` — the uncontrolled initial selection.
    ///
    /// Supplying it hands the component its own selection set, seeded once;
    /// `selected_keys` is the controlled spelling.
    /// `defaultInputValue` — the uncontrolled initial text.
    ///
    /// Written into the state on the first render only; `input_value` is the
    /// controlled spelling.
    pub fn default_input_value(mut self, text: impl Into<SharedString>) -> Self {
        self.default_input_value = Some(text.into());
        self
    }

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
        self.on_selection_change_all = Some(Arc::new(handler));
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

    /// `autoFocus` — take focus on the first render.
    /// `ListLayout`'s `rowHeight` -- and what virtualizes the popover list.
    ///
    /// v3 wraps the list in `<Virtualizer layout={ListLayout}>` inside the
    /// popover; gpui's `uniform_list` builds only the rows in view, and it can do
    /// that because every row is this tall.
    pub fn row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.row_height = Some(h.into());
        self
    }

    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `validate` — returns the message to show, or `None` when the text is
    /// fine. The component runs it and surfaces the result.
    pub fn validate(mut self, f: impl Fn(&str) -> Option<SharedString> + 'static) -> Self {
        self.validate = Some(Arc::new(f));
        self
    }

    /// `validationBehavior` — `Allow` shows the message without blocking a
    /// form submission.
    pub fn validation_behavior(mut self, behavior: crate::form::ValidationBehavior) -> Self {
        self.validation_behavior = Some(behavior);
        self
    }

    /// `allowsEmptyCollection` — keeps the panel open when nothing matches.
    pub fn allows_empty_collection(mut self, v: bool) -> Self {
        self.allows_empty_collection = v;
        self
    }

    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    pub fn should_focus_wrap(mut self, v: bool) -> Self {
        self.should_focus_wrap = v;
        self
    }

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

    /// `name` — the name this field submits under.
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The `Form` field this control submits, when it has a `name`.
    ///
    /// v3 discovers a field through the DOM; gpui gives a child no way to reach
    /// its ancestor, so the control hands the pair over instead.
    pub fn form_field(&self) -> Option<crate::form::FormField> {
        let name = self.name.clone()?;
        Some(
            crate::form::FormField::text(self.state.clone())
                .name(name)
                .is_required(self.is_required),
        )
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
            menu_trigger: MenuTrigger::Input,
            filter: None,
            allows_custom_value: false,
            max_items: 8,
            full_width: false,
            is_disabled: false,
            is_invalid: false,
            is_required: false,
            is_read_only: false,
            auto_focus: false,
            row_height: None,
            validate: None,
            validation_behavior: None,
            allows_empty_collection: false,
            name: None,
            should_focus_wrap: false,
            sections: Vec::new(),
            indicator: None,
            disabled_keys: std::collections::HashSet::new(),
            selection_mode: SelectionMode::Single,
            selected_keys: std::collections::BTreeSet::new(),
            default_value: None,
            default_input_value: None,
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
        self.filter = Some(Arc::new(f));
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `defaultInputValue` seeds the text once, before anything reads it.
        if let Some(text) = self.default_input_value.clone() {
            let state = self.state.clone();
            util::seed_once(
                window,
                cx,
                gpui::ElementId::Name(
                    format!("combobox-{}-default-text", self.state.entity_id().as_u64()).into(),
                ),
                move |cx| {
                    state.update(cx, |s, cx| {
                        s.set_value(text.to_string());
                        cx.notify();
                    });
                },
            );
        }

        // `defaultValue` opts into the component holding its own selection;
        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (selection, selection_own) = util::controlled(
            window,
            cx,
            gpui::ElementId::Name(
                format!("combobox-{}-selection", self.state.entity_id().as_u64()).into(),
            ),
            match self.default_value {
                Some(_) => None,
                None => Some(self.selected_keys.clone()),
            },
            self.default_value.clone().unwrap_or_default(),
        );
        self.selected_keys = selection;

        // `controlled` takes `cx` mutably, so it precedes the theme tokens.
        let (open_state, open_own) = util::controlled(
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
        let entity_id = self.state.entity_id().as_u64();
        let raw_query = self.state.read(cx).value().to_owned();
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
                    .size(util::FIELD_ICON)
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

        let validate = self.validate.clone();
        let mut input = Input::new(self.state.clone())
            .variant(self.variant)
            .is_disabled(self.is_disabled)
            .is_invalid(is_invalid)
            .is_required(self.is_required)
            .is_read_only(self.is_read_only)
            .auto_focus(self.auto_focus)
            .when_some(self.validation_behavior, |i, b| i.validation_behavior(b))
            .when_some(validate, |i, f| i.validate(move |v| f(v)))
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

        // Which suggestion the keyboard is on.
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-cursor").into()),
            cx,
            |_, _| None::<usize>,
        );
        let cursor_at = *cursor.read(cx);
        // React Aria keeps the focused row in view; the panel scrolls and the
        // virtual list scrolls itself. `use_keyed_state` takes `cx` mutably.
        let list_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-list-scroll").into()),
            cx,
            |_, _| gpui::UniformListScrollHandle::new(),
        );
        let panel_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("combobox-{entity_id}-panel-scroll").into()),
            cx,
            |_, _| gpui::ScrollHandle::new(),
        );
        let list_scroll_now = list_scroll.read(cx).clone();
        let panel_scroll_now = panel_scroll.read(cx).clone();

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
            // `.combo-box__input-group` is the field itself, and
            // `--full-width` is the `full_width` flag above.
            .child(input.render(window, cx));

        // `allowsEmptyCollection` keeps the panel up with no matches. Without
        // it an empty result closes the list -- except when a custom value is
        // allowed, since then the empty state is the "press Enter" hint.
        // Up, down, Home, End and Enter walk the suggestions; the inner input
        // keeps left and right for the caret.
        if !self.is_disabled && !self.is_read_only {
            let stops: Vec<usize> = (0..matches.len())
                .filter(|i| {
                    matches
                        .get(*i)
                        .is_some_and(|item| !self.disabled_keys.contains(item))
                })
                .collect();
            let held = cursor;
            let wrap = self.should_focus_wrap;
            let virtual_rows = self.row_height.is_some();
            let key_list_scroll = list_scroll_now.clone();
            let key_panel_scroll = panel_scroll_now.clone();
            let rows = matches.clone();
            let state = self.state.clone();
            let on_selection_change = self.on_selection_change.clone();
            let open_own_keys = open_own.clone();
            let on_open_change = self.on_open_change.clone();
            root = root.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let from = *held.read(cx);
                match crate::list_nav::resolve(&stops, from, key, wrap) {
                    crate::list_nav::Move::To(next) => {
                        held.update(cx, |v, cx| {
                            *v = Some(next);
                            cx.notify();
                        });
                        if virtual_rows {
                            key_list_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                        } else {
                            key_panel_scroll.scroll_to_item(next);
                        }
                        // Walking the list opens it, which is what typing does.
                        if let Some(held) = &open_own_keys {
                            held.update(cx, |v, cx| {
                                *v = true;
                                cx.notify();
                            });
                        }
                    }
                    crate::list_nav::Move::Activate => {
                        let Some(item) = from.and_then(|i| rows.get(i).cloned()) else {
                            return;
                        };
                        // Taking a suggestion fills the field and closes the
                        // list, the way a click does.
                        state.update(cx, |st, cx| {
                            st.set_value(item.to_string());
                            cx.notify();
                        });
                        held.update(cx, |v, _| *v = None);
                        if let Some(held) = &open_own_keys {
                            held.update(cx, |v, cx| {
                                *v = false;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_selection_change {
                            cb(&item, window, cx);
                        }
                        if let Some(cb) = &on_open_change {
                            cb(false, window, cx);
                        }
                    }
                    crate::list_nav::Move::Ignore => {
                        if key == "escape" {
                            held.update(cx, |v, _| *v = None);
                            if let Some(held) = &open_own_keys {
                                held.update(cx, |v, cx| {
                                    *v = false;
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &on_open_change {
                                cb(false, window, cx);
                            }
                        }
                    }
                }
            });
        }

        let show_list = open_state
            && !self.is_disabled
            && (!matches.is_empty() || self.allows_empty_collection || self.allows_custom_value);
        if show_list {
            let panel = div()
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
                .track_scroll(&panel_scroll_now)
                .rounded(container_radius)
                .bg(colors.overlay.background)
                // v3 gives a floating panel no border: `.popover` and friends are
                // `bg-overlay shadow-overlay` and a radius, and dark mode's
                // inset hairline is what separates the panel from the page.
                .when_some(layout.overlay_hairline, |el, hairline| {
                el.border(layout.border_width).border_color(hairline)
                })
                .text_color(colors.overlay.foreground)
                .when(
                    !layout.overlay_shadow.is_empty(),
                    |e: gpui::Stateful<gpui::Div>| e.shadow(layout.overlay_shadow.clone()),
                );

            // React Aria dismisses the list on a press outside it; Escape is
            // read in the field's own key handler above.
            let dismiss_own = open_own;
            let dismiss_cb = self.on_open_change.clone();
            let mut panel = util::dismiss_on_press_outside(panel, move |window, cx| {
                if let Some(held) = &dismiss_own {
                    held.update(cx, |v, cx| {
                        *v = false;
                        cx.notify();
                    });
                }
                if let Some(cb) = &dismiss_cb {
                    cb(false, window, cx);
                }
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
                        .text_size(util::FIELD_TEXT)
                        .text_color(colors.muted)
                        .child(message),
                );
            }

            // Everything a row reads, owned: `uniform_list`'s callback is
            // `'static` and runs again on every scroll, so it cannot borrow
            // `self` or the theme -- and one row builder for both paths is what
            // keeps a virtual list drawing the same row as a short one.
            let matches_len = matches.len();
            let rows = matches;
            let sections = self.sections.clone();
            let row_disabled_keys = self.disabled_keys.clone();
            let row_selected_keys = self.selected_keys.clone();
            let indicator: Option<std::rc::Rc<dyn Fn(bool) -> gpui::AnyElement>> =
                self.indicator.take().map(std::rc::Rc::from);
            let on_change_all = self.on_selection_change_all.clone();
            let on_change_one = self.on_selection_change.clone();
            let on_open = self.on_open_change.clone();
            let row_state = self.state.clone();
            let selection_own = selection_own;
            let row_muted = colors.muted;
            let row_hover_bg = colors.default.color;
            let row_focus = colors.focus;
            let row_accent = colors.accent.color;
            let row_disabled_opacity = layout.disabled_opacity;
            let row_of = move |index: usize, fixed_h: Option<gpui::Pixels>, cx: &mut App| {
                let item = &rows[index];
                // A section header rides above the row it introduces, so the two
                // are one element -- a virtual row is one slot tall.
                let mut head: Vec<gpui::AnyElement> = Vec::new();
                let done = |head: Vec<gpui::AnyElement>, row: gpui::AnyElement| {
                    div()
                        .flex()
                        .flex_col()
                        .when_some(fixed_h, |el, h| el.h(h).w_full())
                        .children(head)
                        .child(row)
                        .into_any_element()
                };
                // `ListBox.Section`'s `Header`, above the item it introduces.
                if let Some((_, label)) = sections.iter().find(|(at, _)| at == item) {
                    head.push(
                        div()
                            .px(px(8.))
                            .pt(px(6.))
                            .pb(px(2.))
                            .text_size(px(12.))
                            .text_color(row_muted)
                            .child(label.to_string())
                            .into_any_element(),
                    );
                }
                let item_disabled = row_disabled_keys.contains(item);
                let hover_bg = row_hover_bg;
                let mut row = div()
                        .id(gpui::ElementId::Name(
                            format!("combobox-{entity_id}-item-{index}").into(),
                        ))
                        // `.list-box-item`: `min-h-9 rounded-2xl px-2 py-1.5 gap-3`.
                        .min_h(util::FIELD_HEIGHT)
                        .px(px(8.))
                        .py(px(6.))
                        .gap(px(12.))
                        .rounded(util::soft_radius(cx))
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(util::FIELD_TEXT)
                        .child(item.to_string());

                if item_disabled {
                    row = row.opacity(row_disabled_opacity);
                } else {
                    row = row.cursor_pointer().hover(move |s| s.bg(hover_bg));
                }

                // `status-focused` on the row the keyboard is on.
                if cursor_at == Some(index) {
                    row = row.border_2().border_color(row_focus);
                }

                // `ListBox.ItemIndicator`: a caller-drawn tick replaces the
                // check glyph, and is asked for on every row so it can draw the
                // unselected state too.
                let row_selected = row_selected_keys.contains(item);
                match &indicator {
                    Some(render) => row = row.child(render(row_selected)),
                    None if multiple && row_selected => {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                .text_color(row_accent),
                        );
                    }
                    None => {}
                }

                if item_disabled {
                    return done(head, row.into_any_element());
                }

                // Multiple mode toggles membership and leaves the panel open.
                if multiple {
                    if on_change_all.is_some() || selection_own.is_some() {
                        let cb = on_change_all.clone();
                        let own = selection_own.clone();
                        let current = row_selected_keys.clone();
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
                    return done(head, row.into_any_element());
                }

                let value = item.clone();
                let state = row_state.clone();
                let on_selection_change = on_change_one.clone();
                let on_open_change = on_open.clone();
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

                done(head, row.into_any_element())
            };

            match self.row_height {
                // Virtual: only the rows in view are built, which is what makes
                // a thousand options affordable.
                Some(row_height) => {
                    panel = panel.child(
                        gpui::uniform_list(
                            gpui::ElementId::Name(format!("combobox-{entity_id}-rows").into()),
                            matches_len,
                            move |range, _window, cx| {
                                range
                                    .map(|i| row_of(i, Some(row_height), cx))
                                    .collect::<Vec<_>>()
                            },
                        )
                        .track_scroll(list_scroll_now)
                        .h(px(240.))
                        .w_full(),
                    );
                }
                None => {
                    for index in 0..matches_len {
                        panel = panel.child(row_of(index, None, cx));
                    }
                }
            }

            root = root.child(util::floating(
                util::placed_field_panel(self.placement, px(6.)).child(crate::anim::entering_zoom(
                    panel,
                    gpui::ElementId::Name(format!("combobox-{entity_id}-anim").into()),
                    crate::anim::ZoomBox::panel(px(4.), container_radius).padding_x(px(4.)),
                    crate::anim::Motion::LIST_IN,
                    cx,
                )),
            ));
        }

        root
    }
}
