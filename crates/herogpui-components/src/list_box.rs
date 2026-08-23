//! ListBox — port of `@heroui/list-box` (v3).
//!
//! A selectable list of options. Mirrors the React API: `selectionMode`,
//! `selectedKeys`, `disabledKeys`, `onSelectionChange`, `onAction`, and the
//! `default | danger` item variant. Sections are expressed with
//! [`ListBoxItem::section`] headers and [`ListBoxItem::separator`].

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    div, prelude::*, px, App, ElementId, InteractiveElement, IntoElement, RenderOnce, SharedString,
    Styled, Window,
};
use herogpui_core::SelectionMode;
use herogpui_theme::ActiveTheme;

use crate::{icons, util};

/// Visual variant of a list item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListBoxItemVariant {
    #[default]
    Default,
    /// Destructive action — danger text, danger-soft hover.
    Danger,
}

/// One row of a [`ListBox`].
#[derive(Clone)]
pub enum ListBoxItem {
    /// A selectable option.
    Option {
        key: SharedString,
        label: SharedString,
        description: Option<SharedString>,
        /// Asset path of a leading icon.
        icon: Option<SharedString>,
        /// Trailing shortcut hint.
        shortcut: Option<SharedString>,
        variant: ListBoxItemVariant,
        is_disabled: bool,
    },
    /// A non-interactive section header.
    Section(SharedString),
    /// A horizontal rule between groups.
    Separator,
}

impl ListBoxItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::Option {
            key: key.into(),
            label: label.into(),
            description: None,
            icon: None,
            shortcut: None,
            variant: ListBoxItemVariant::Default,
            is_disabled: false,
        }
    }

    pub fn section(label: impl Into<SharedString>) -> Self {
        Self::Section(label.into())
    }

    pub fn separator() -> Self {
        Self::Separator
    }

    /// Secondary line beneath the label.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        if let Self::Option { description, .. } = &mut self {
            *description = Some(text.into());
        }
        self
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        if let Self::Option { icon, .. } = &mut self {
            *icon = Some(path.into());
        }
        self
    }

    pub fn shortcut(mut self, text: impl Into<SharedString>) -> Self {
        if let Self::Option { shortcut, .. } = &mut self {
            *shortcut = Some(text.into());
        }
        self
    }

    pub fn variant(mut self, v: ListBoxItemVariant) -> Self {
        if let Self::Option { variant, .. } = &mut self {
            *variant = v;
        }
        self
    }

    /// Shorthand for [`ListBoxItemVariant::Danger`].
    pub fn danger(self) -> Self {
        self.variant(ListBoxItemVariant::Danger)
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        if let Self::Option { is_disabled, .. } = &mut self {
            *is_disabled = v;
        }
        self
    }

    /// The item's key, or `None` for headers and separators.
    pub fn key(&self) -> Option<&SharedString> {
        match self {
            Self::Option { key, .. } => Some(key),
            _ => None,
        }
    }
}

type OnSelectionChange = Arc<dyn Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static>;
type OnAction = Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;
/// `ListBox.ItemIndicator`'s render function, handed `isSelected`.
type Indicator = Arc<dyn Fn(bool) -> gpui::AnyElement + 'static>;

/// HeroUI ListBox.
#[derive(IntoElement)]
pub struct ListBox {
    id: ElementId,
    items: Vec<ListBoxItem>,
    selection_mode: SelectionMode,
    selected_keys: HashSet<SharedString>,
    disabled_keys: HashSet<SharedString>,
    /// Applies to every item unless the item overrides it.
    variant: ListBoxItemVariant,
    max_h: Option<gpui::Pixels>,
    /// `shouldFocusWrap` — whether arrow keys wrap at the ends.
    should_focus_wrap: bool,
    /// `ListLayout`'s `rowHeight`. Setting it virtualizes the list: a fixed row
    /// height is what lets the geometry be computed instead of laid out.
    row_height: Option<gpui::Pixels>,
    /// `ListLayout`'s `estimatedRowHeight` — the estimate that virtualizes a
    /// list whose rows are *not* all one height.
    estimated_row_height: Option<gpui::Pixels>,
    /// `ListLayout`'s `headingHeight` — a section row's height when the list is
    /// virtual.
    heading_height: Option<gpui::Pixels>,
    /// `ListLayout`'s `gap` and `padding`, which override the stylesheet's.
    gap: gpui::Pixels,
    padding: gpui::Pixels,
    /// `ListBox.ItemIndicator` — draw the tick yourself. v3 hands its render
    /// function `isSelected`, so this closure receives it.
    indicator: Option<Indicator>,
    /// `children` on `ListBox.Item` — a render function handed the row's key and
    /// its state.
    item_content:
        Option<Arc<dyn Fn(&SharedString, util::InteractiveState) -> gpui::AnyElement + 'static>>,
    on_selection_change: Option<OnSelectionChange>,
    on_action: Option<OnAction>,
}

impl ListBox {
    pub fn new(id: impl Into<ElementId>, items: Vec<ListBoxItem>) -> Self {
        Self {
            id: id.into(),
            items,
            selection_mode: SelectionMode::Single,
            selected_keys: HashSet::new(),
            disabled_keys: HashSet::new(),
            variant: ListBoxItemVariant::Default,
            should_focus_wrap: false,
            row_height: None,
            estimated_row_height: None,
            heading_height: None,
            // `.list-box` is `p-1` with `mt-1` between children.
            gap: px(4.),
            padding: px(4.),
            max_h: None,
            indicator: None,
            item_content: None,
            on_selection_change: None,
            on_action: None,
        }
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn selected_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.selected_keys = keys.into_iter().collect();
        self
    }

    /// Convenience for `selectionMode="single"`.
    pub fn selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.selected_keys = HashSet::from([key.into()]);
        self
    }

    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    pub fn variant(mut self, variant: ListBoxItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Caps the list height and scrolls beyond it.
    /// `shouldFocusWrap` — whether the arrow keys wrap at the ends of the list.
    pub fn should_focus_wrap(mut self, v: bool) -> Self {
        self.should_focus_wrap = v;
        self
    }

    pub fn max_h(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.max_h = Some(h.into());
        self
    }

    /// `ListLayout`'s `rowHeight` — **and** what virtualizes the list.
    ///
    /// v3 wraps the list in `<Virtualizer layout={ListLayout}
    /// layoutOptions={{rowHeight: 50}}>`; the wrapper has no separate identity
    /// here, so the option that defines the layout carries it. gpui's
    /// `uniform_list` builds only the rows the viewport shows, and it can do
    /// that because every row is this tall.
    pub fn row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.row_height = Some(h.into());
        self
    }

    /// `ListLayout`'s `estimatedRowHeight` — virtualize rows that are *not* all
    /// the same height.
    ///
    /// `rowHeight` maps to `uniform_list`, which measures one row and multiplies;
    /// this maps to gpui's `list`, which measures each row it builds and keeps a
    /// running total, so a described row and a plain one can differ. The estimate
    /// is what it renders beyond the viewport (`overdraw`) while it learns the
    /// real heights.
    pub fn estimated_row_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.estimated_row_height = Some(h.into());
        self
    }

    /// `ListLayout`'s `headingHeight` — how tall a section row is in a virtual
    /// list, where a row cannot size itself.
    pub fn heading_height(mut self, h: impl Into<gpui::Pixels>) -> Self {
        self.heading_height = Some(h.into());
        self
    }

    /// `ListLayout`'s `gap`, overriding the stylesheet's `mt-1`.
    pub fn gap(mut self, gap: impl Into<gpui::Pixels>) -> Self {
        self.gap = gap.into();
        self
    }

    /// `ListLayout`'s `padding`, overriding the stylesheet's `p-1`.
    pub fn padding(mut self, padding: impl Into<gpui::Pixels>) -> Self {
        self.padding = padding.into();
        self
    }

    /// `ListBox.ItemIndicator` — draw the selected tick yourself.
    ///
    /// The closure is handed `isSelected`, the value v3 passes into the same
    /// render function, so a caller can return its own glyph, or nothing.
    /// `children` on `ListBox.Item` — replaces a row's label.
    ///
    /// The closure is handed the row's key and the state v3 passes into the same
    /// render prop: `isSelected`, `isFocused`, `isPressed` and `isDisabled`. The
    /// press is a frame behind the pointer, because gpui reports it to a handler.
    pub fn item_content(
        mut self,
        render: impl Fn(&SharedString, util::InteractiveState) -> gpui::AnyElement + 'static,
    ) -> Self {
        self.item_content = Some(Arc::new(render));
        self
    }

    pub fn indicator(mut self, render: impl Fn(bool) -> gpui::AnyElement + 'static) -> Self {
        self.indicator = Some(Arc::new(render));
        self
    }

    /// Called with the full selection after a toggle.
    pub fn on_selection_change(
        mut self,
        handler: impl Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(Arc::new(handler));
        self
    }

    /// Called when an item is activated, regardless of selection mode.
    pub fn on_action(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ListBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Which row the keyboard is on, and the handle that receives the keys.
        // `use_keyed_state` takes `cx` mutably, so both precede the tokens.
        let base = format!("{:?}", self.id);
        let focus_handle = window.use_keyed_state(
            ElementId::Name(format!("{base}-focus").into()),
            cx,
            |_, cx| cx.focus_handle().tab_stop(true),
        );
        let focus_handle = focus_handle.read(cx).clone();
        let cursor = window.use_keyed_state(
            ElementId::Name(format!("{base}-cursor").into()),
            cx,
            |_, _| None::<usize>,
        );
        let cursor_at = *cursor.read(cx);
        // React Aria keeps the focused row in view. Two handles, because the
        // virtual list owns its own scrolling and a plain one does not.
        let list_scroll = window.use_keyed_state(
            ElementId::Name(format!("{base}-list-scroll").into()),
            cx,
            |_, _| gpui::UniformListScrollHandle::new(),
        );
        let box_scroll = window.use_keyed_state(
            ElementId::Name(format!("{base}-box-scroll").into()),
            cx,
            |_, _| gpui::ScrollHandle::new(),
        );
        // `gpui::list`'s state is intrusive -- the caller holds it -- so a
        // variable-height list keeps one here, seeded with the item count and
        // the estimate it overdraws by.
        let list_state = {
            let count = self.items.len();
            let overdraw = self.estimated_row_height.unwrap_or(px(36.)) * 3.;
            window.use_keyed_state(
                ElementId::Name(format!("{base}-list-state").into()),
                cx,
                move |_, _| gpui::ListState::new(count, gpui::ListAlignment::Top, overdraw),
            )
        };
        let list_scroll_now = list_scroll.read(cx).clone();
        let box_scroll_now = box_scroll.read(cx).clone();
        // The letters typed so far. A search that reset every frame could only
        // ever match one letter.
        let typed = window.use_keyed_state(
            ElementId::Name(format!("{base}-typed").into()),
            cx,
            |_, _| crate::list_nav::Typeahead::default(),
        );

        let colors = cx.colors();

        // `.list-box` is `relative w-full overflow-clip p-1` with `mt-1` between
        // children, and nothing else: the popover around it paints the panel.
        // This used to draw its own surface, border and radius, which put a
        // second panel inside every picker.
        let mut list = div()
            .id(self.id.clone())
            .relative()
            .w_full()
            .flex()
            .flex_col()
            .gap(self.gap)
            .p(self.padding)
            .overflow_hidden()
            .text_color(colors.foreground)
            .track_focus(&focus_handle)
            .key_context("ListBox")
            // A click has to move the keyboard's focus onto the list, or the
            // arrow keys would go nowhere after a pointer selection.
            .on_mouse_down(gpui::MouseButton::Left, {
                let fh = focus_handle.clone();
                move |_, window, _| window.focus(&fh)
            });

        // A virtualized list scrolls inside `uniform_list`, which owns the
        // scroll offset it computes the visible range from; a second scroller
        // around it would move the rows without telling it.
        if let (Some(max_h), None) = (self.max_h, self.row_height) {
            list = list
                .max_h(max_h)
                .overflow_y_scroll()
                .track_scroll(&box_scroll_now);
        }

        // The rows a keyboard can land on: an item that is not disabled.
        // Sections and separators are skipped, so the cursor never stops on
        // something that cannot be chosen.
        let stops: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| match item {
                ListBoxItem::Option {
                    key, is_disabled, ..
                } => !is_disabled && !self.disabled_keys.contains(key),
                _ => false,
            })
            .map(|(i, _)| i)
            .collect();

        if !stops.is_empty() {
            let held = cursor;
            let stops_for_keys = stops;
            let wrap = self.should_focus_wrap;
            let virtual_rows = self.row_height.is_some();
            let key_list_scroll = list_scroll_now.clone();
            let key_box_scroll = box_scroll_now;
            let keys: Vec<SharedString> = self
                .items
                .iter()
                .map(|item| item.key().cloned().unwrap_or_default())
                .collect();
            // Every row's text, so typeahead can search it. A row that cannot be
            // landed on has no label here, so it is never a match.
            let labels: Vec<String> = self
                .items
                .iter()
                .map(|item| match item {
                    ListBoxItem::Option { label, .. } => label.to_string(),
                    _ => String::new(),
                })
                .collect();
            let typed_keys = typed;
            let mode = self.selection_mode;
            let selected_now = self.selected_keys.clone();
            let on_selection_change = self.on_selection_change.clone();
            let on_action = self.on_action.clone();
            list = list.on_key_down(move |event, window, cx| {
                let from = *held.read(cx);
                match crate::list_nav::resolve(
                    &stops_for_keys,
                    from,
                    event.keystroke.key.as_str(),
                    wrap,
                ) {
                    crate::list_nav::Move::To(next) => {
                        held.update(cx, |v, cx| {
                            *v = Some(next);
                            cx.notify();
                        });
                        if virtual_rows {
                            key_list_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                        } else {
                            key_box_scroll.scroll_to_item(next);
                        }
                    }
                    crate::list_nav::Move::Activate => {
                        let Some(item_key) = from.and_then(|i| keys.get(i).cloned()) else {
                            return;
                        };
                        if let Some(cb) = &on_action {
                            cb(&item_key, window, cx);
                        }
                        if let Some(cb) = &on_selection_change {
                            // The same answer a click gives: `Single` collapses
                            // to this key, `Multiple` toggles it.
                            let next = match mode {
                                SelectionMode::None => selected_now.clone(),
                                SelectionMode::Single => HashSet::from([item_key]),
                                SelectionMode::Multiple => {
                                    let mut set = selected_now.clone();
                                    if !set.remove(&item_key) {
                                        set.insert(item_key.clone());
                                    }
                                    set
                                }
                            };
                            cb(&next, window, cx);
                        }
                    }
                    crate::list_nav::Move::Ignore => {
                        // Typeahead: letters jump to the row that starts with
                        // them, which is the other half of v3's keyboard.
                        let key = event.keystroke.key.as_str();
                        if !crate::list_nav::is_typeahead_key(key) {
                            return;
                        }
                        let now = std::time::Instant::now();
                        let (query, repeat) = typed_keys.update(cx, |t, _| {
                            let query = t.push(key, now);
                            (query, t.is_repeat())
                        });
                        if let Some(found) = crate::list_nav::typeahead(
                            &labels,
                            &stops_for_keys,
                            from,
                            &query,
                            repeat,
                        ) {
                            held.update(cx, |v, cx| {
                                *v = Some(found);
                                cx.notify();
                            });
                        }
                    }
                }
            });
        }

        // With `rowHeight` set the list is virtual: only the rows the viewport
        // shows are built, which is what makes a thousand of them affordable.
        // `uniform_list` measures row 0 and multiplies, so the row builder is
        // told the height rather than left to size itself.
        // `estimatedRowHeight` virtualizes a list whose rows differ: gpui's
        // `list` measures each row it builds, where `uniform_list` measures one
        // and multiplies. Its state is intrusive -- the caller has to hold it --
        // so it lives in the window's keyed store, and a change in the item
        // count resets it.
        if self.estimated_row_height.is_some() {
            let height = self.max_h.unwrap_or(px(400.));
            let count = self.items.len();
            let rows = std::rc::Rc::new(self);
            let state = list_state.read(cx).clone();
            if state.item_count() != count {
                state.reset(count);
            }
            return list
                .child(
                    gpui::list(state, move |index, _window, cx| {
                        rows.row(index, cursor_at, None, cx)
                    })
                    .h(height)
                    .w_full(),
                )
                .into_any_element();
        }

        if let Some(row_height) = self.row_height {
            let height = self.max_h.unwrap_or(px(400.));
            let list_id = self.id.clone();
            let count = self.items.len();
            let rows = std::rc::Rc::new(self);
            return list
                .child(
                    gpui::uniform_list(
                        ElementId::Name(format!("{base}-rows").into()),
                        count,
                        move |range, _window, cx| {
                            range
                                .map(|i| rows.row(i, cursor_at, Some(row_height), cx))
                                .collect::<Vec<_>>()
                        },
                    )
                    .track_scroll(list_scroll_now)
                    .id(list_id)
                    .h(height)
                    .w_full(),
                )
                .into_any_element();
        }

        let mut items = Vec::with_capacity(self.items.len());
        for index in 0..self.items.len() {
            items.push(self.row(index, cursor_at, None, cx));
        }
        list.children(items).into_any_element()
    }
}

impl ListBox {
    /// One row, by index.
    ///
    /// Shared by the plain and the virtualized paths so the two cannot drift:
    /// `fixed_h` is `Some` only for the virtual one, where every row -- a
    /// heading and a separator included -- is one `rowHeight` tall because that
    /// is the number the scroll geometry is computed from.
    fn row(
        &self,
        index: usize,
        cursor_at: Option<usize>,
        fixed_h: Option<gpui::Pixels>,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let colors = cx.colors();
        // `.list-box-item` is `min-h-9`.
        let row_h = fixed_h.unwrap_or(px(36.));
        let text_size = util::FIELD_TEXT;
        let sized = |el: gpui::Div| match fixed_h {
            Some(h) => el.h(h),
            None => el,
        };
        match &self.items[index] {
            ListBoxItem::Separator => sized(
                div()
                    .my(px(4.))
                    .mx(px(4.))
                    .h(cx.layout().border_width)
                    .bg(colors.separator),
            )
            .into_any_element(),
            ListBoxItem::Section(label) => sized(
                div()
                    .when_some(self.heading_height, |el, h| el.h(h))
                    .px(px(8.))
                    .pt(px(6.))
                    .pb(px(4.))
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.muted)
                    .child(label.to_string()),
            )
            .into_any_element(),
            ListBoxItem::Option {
                key,
                label,
                description,
                icon,
                shortcut,
                variant,
                is_disabled,
            } => {
                let variant = if *variant == ListBoxItemVariant::Default {
                    self.variant
                } else {
                    *variant
                };
                let disabled = *is_disabled || self.disabled_keys.contains(key);
                let selected = self.selected_keys.contains(key);

                let (fg, hover_bg) = match variant {
                    ListBoxItemVariant::Default => (colors.foreground, colors.default.color),
                    ListBoxItemVariant::Danger => {
                        (colors.danger.soft_foreground(), colors.danger.soft())
                    }
                };

                let mut row = div()
                    .id(ElementId::Name(
                        format!("{:?}-item-{index}", self.id).into(),
                    ))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.))
                    .px(px(8.))
                    // A virtual row is laid out on its own, so it takes the width
                    // it is given rather than inheriting a stretch.
                    .map(|el| match fixed_h {
                        Some(h) => el.h(h).w_full(),
                        None => el.min_h(row_h),
                    })
                    .py(px(6.))
                    .rounded(util::soft_radius(cx))
                    .text_size(text_size)
                    .text_color(fg);

                if disabled {
                    row = row.opacity(cx.layout().disabled_opacity);
                } else {
                    row = row.cursor_pointer().hover(move |s| s.bg(hover_bg));
                }

                if selected {
                    row = row.bg(match variant {
                        ListBoxItemVariant::Default => colors.accent.soft(),
                        ListBoxItemVariant::Danger => colors.danger.soft(),
                    });
                }

                // `.list-box-item` takes `status-focused` on the row the keyboard
                // is on. A ring rather than a border: a border would move the
                // row's content by two pixels as the cursor arrived.
                let row =
                    util::with_focus_ring(row, cursor_at == Some(index), true, Vec::new(), cx);
                let mut row = row;

                if let Some(path) = icon {
                    row = row.child(
                        gpui::svg()
                            .size(util::FIELD_ICON)
                            .path(path.clone())
                            .flex_shrink_0()
                            .text_color(fg),
                    );
                }

                // Label plus optional description stack -- or the render
                // function, which v3 hands the row's state.
                if let Some(render) = &self.item_content {
                    let focused = cursor_at == Some(index);
                    return row
                        .child(render(
                            key,
                            util::InteractiveState {
                                is_hovered: false,
                                is_pressed: false,
                                is_focused: focused,
                                is_focus_visible: focused && util::focus_visible(cx),
                                is_selected: selected,
                                is_disabled: disabled,
                                is_indeterminate: false,
                            },
                        ))
                        .into_any_element();
                }
                row = row.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(div().child(label.to_string()))
                        .when_some(description.clone(), |el, d| {
                            el.child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(colors.muted)
                                    .child(d.to_string()),
                            )
                        }),
                );

                if let Some(render) = &self.indicator {
                    row = row.child(render(selected));
                } else if selected && self.selection_mode != SelectionMode::None {
                    row = row.child(
                        gpui::svg()
                            // `.list-box-item__indicator` is `size-4`.
                            .size(px(16.))
                            .path(icons::CHECK)
                            .flex_shrink_0()
                            .text_color(colors.accent.color),
                    );
                } else if let Some(sc) = shortcut {
                    row = row.child(
                        div()
                            // A shortcut is a `Kbd`, which is `text-xs`.
                            .text_size(px(12.))
                            .text_color(colors.muted)
                            .child(sc.to_string()),
                    );
                }

                if !disabled {
                    let key = key.clone();
                    let mode = self.selection_mode;
                    let current = self.selected_keys.clone();
                    let on_selection_change = self.on_selection_change.clone();
                    let on_action = self.on_action.clone();
                    row = row.on_click(move |_, window, cx| {
                        if let Some(action) = &on_action {
                            action(&key, window, cx);
                        }
                        if let Some(change) = &on_selection_change {
                            let next = match mode {
                                SelectionMode::None => current.clone(),
                                SelectionMode::Single => HashSet::from([key.clone()]),
                                SelectionMode::Multiple => {
                                    let mut set = current.clone();
                                    if !set.remove(&key) {
                                        set.insert(key.clone());
                                    }
                                    set
                                }
                            };
                            change(&next, window, cx);
                        }
                    });
                }

                row.into_any_element()
            }
        }
    }
}
