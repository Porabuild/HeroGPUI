//! Dropdown & Menu — port of `@heroui/dropdown`, `@heroui/menu` and
//! `@heroui/listbox`.

use gpui::{px, AnyElement, App, ClickEvent, IntoElement, InteractiveElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window};
use herogpui_core::SelectionMode;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// One entry of a dropdown menu.
pub enum MenuItem {
    /// Section caption (`<MenuSection>` title).
    SectionLabel(SharedString),
    Separator,
    Item {
        key: SharedString,
        label: SharedString,
        shortcut: Option<SharedString>,
        icon: Option<&'static str>,
        is_danger: bool,
    },
}

impl MenuItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        MenuItem::Item {
            key: key.into(),
            label: label.into(),
            shortcut: None,
            icon: None,
            is_danger: false,
        }
    }

    pub fn shortcut(mut self, s: impl Into<SharedString>) -> Self {
        if let MenuItem::Item { shortcut, .. } = &mut self {
            *shortcut = Some(s.into());
        }
        self
    }

    pub fn icon(mut self, path: &'static str) -> Self {
        if let MenuItem::Item { icon, .. } = &mut self {
            *icon = Some(path);
        }
        self
    }

    pub fn danger(mut self) -> Self {
        if let MenuItem::Item { is_danger, .. } = &mut self {
            *is_danger = true;
        }
        self
    }
}

type OnSelect = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// Menu panel (`<Menu>` / `<Listbox>`).
/// `type` on `Dropdown.ItemIndicator` — how a selected item is marked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorKind {
    #[default]
    Checkmark,
    Dot,
}

impl IndicatorKind {
    pub const ALL: [IndicatorKind; 2] = [IndicatorKind::Checkmark, IndicatorKind::Dot];

    pub fn label(self) -> &'static str {
        match self {
            IndicatorKind::Checkmark => "Checkmark",
            IndicatorKind::Dot => "Dot",
        }
    }
}

/// `onSelectionChange` — the whole selection after an item is activated.
pub type OnSelectionChange =
    std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Menu {
    id: gpui::ElementId,
    items: Vec<MenuItem>,
    selected_key: Option<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    disabled_keys: Vec<SharedString>,
    indicator: IndicatorKind,
    on_selection_change: Option<OnSelectionChange>,
    on_action: Option<OnSelect>,
    on_select: Option<OnSelect>,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            id: gpui::ElementId::Name("menu".into()),
            items,
            selected_key: None,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            disabled_keys: Vec::new(),
            indicator: IndicatorKind::default(),
            on_selection_change: None,
            on_action: None,
            on_select: None,
        }
    }

    /// `type` on `Dropdown.ItemIndicator` — a check mark or a dot.
    pub fn indicator(mut self, kind: IndicatorKind) -> Self {
        self.indicator = kind;
        self
    }

    /// `selectionMode` — `None` (the default) makes items pure actions.
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `selectedKeys` — the controlled selection.
    pub fn selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.selected_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `disabledKeys` — items that cannot be activated.
    pub fn disabled_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.disabled_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `onSelectionChange` — the whole selection after an item is activated.
    pub fn on_selection_change(
        mut self,
        f: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onAction` — an item was activated, independent of any selection.
    pub fn on_action(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_select(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(std::sync::Arc::new(f));
        self
    }

    pub fn selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.selected_key = Some(key.into());
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let base = format!("{:?}", self.id);

        let mut panel = gpui::div()
            .flex()
            .flex_col()
            .min_w(px(180.))
            .py(px(6.))
            .bg(colors.surface.background)
            .rounded(px(12.))
            .border_1()
            .border_color(colors.separator)
            .shadow(cx.layout().overlay_shadow.clone())
            .overflow_hidden();

        for (i, item) in self.items.into_iter().enumerate() {
            match item {
                MenuItem::Separator => {
                    panel = panel.child(gpui::div().w_full().my(px(4.)).h(cx.layout().border_width).bg(colors.separator));
                }
                MenuItem::SectionLabel(label) => {
                    panel = panel.child(
                        gpui::div()
                            .px(px(12.))
                            .pt(px(8.))
                            .pb(px(2.))
                            .text_size(px(11.))
                            .text_color(colors.muted)
                            .child(label.to_string()),
                    );
                }
                MenuItem::Item {
                    key,
                    label,
                    shortcut,
                    icon,
                    is_danger,
                } => {
                    // Either the single controlled key or membership of the
                    // selection set marks an item.
                    let is_selected = self.selected_key.as_ref() == Some(&key)
                        || self.selected_keys.contains(&key);
                    let is_item_disabled = self.disabled_keys.contains(&key);
                    let text_color = if is_item_disabled {
                        colors.muted
                    } else if is_danger {
                        colors.danger.color
                    } else {
                        colors.foreground
                    };
                    let mut row = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-item-{i}").into()))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(12.))
                        .h(px(32.))
                        .text_size(px(13.5))
                        .text_color(text_color);
                    if !is_item_disabled {
                        row = row.cursor_pointer();
                        row = row.hover(move |s| s.bg(colors.default.soft()));
                    }
                    row = when_selected(row, is_selected, sem_primary(cx));

                    if let Some(icon_path) = icon {
                        row = row.child(
                            gpui::svg().size(px(15.)).path(icon_path).text_color(text_color),
                        );
                    }
                    row = row.child(gpui::div().flex_1().child(label.to_string()));
                    if let Some(sc) = shortcut {
                        row = row.child(
                            gpui::div()
                                .text_size(px(11.5))
                                .text_color(colors.muted)
                                .child(sc.to_string()),
                        );
                    }
                    if is_selected && self.selection_mode != SelectionMode::None {
                        row = match self.indicator {
                            IndicatorKind::Checkmark => row.child(
                                gpui::svg()
                                    .size(px(13.))
                                    .path(crate::icons::CHECK)
                                    // svg() never inherits text colour.
                                    .text_color(sem_primary(cx)),
                            ),
                            IndicatorKind::Dot => row.child(
                                gpui::div()
                                    .size(px(6.))
                                    .rounded_full()
                                    .bg(sem_primary(cx)),
                            ),
                        };
                    }

                    if !is_item_disabled {
                        let on_select = self.on_select.clone();
                        let on_action = self.on_action.clone();
                        let on_selection_change = self.on_selection_change.clone();
                        if on_select.is_some()
                            || on_action.is_some()
                            || on_selection_change.is_some()
                        {
                            let key2 = key.clone();
                            let mode = self.selection_mode;
                            let current = self.selected_keys.clone();
                            row = row.on_click(move |_, window, cx| {
                                if let Some(cb) = &on_select {
                                    cb(&key2, window, cx);
                                }
                                if let Some(cb) = &on_action {
                                    cb(&key2, window, cx);
                                }
                                if let Some(cb) = &on_selection_change {
                                    let next = crate::selection::next_selection(
                                        &current, &key2, mode, false,
                                    );
                                    cb(&next, window, cx);
                                }
                            });
                        }
                    }

                    panel = panel.child(row);
                }
            }
        }

        crate::util::floating(crate::anim::entering(panel, "dropdown-panel", cx))
    }
}

fn sem_primary(cx: &App) -> gpui::Hsla {
    cx.colors().accent.color
}

fn when_selected(el: gpui::Stateful<gpui::Div>, selected: bool, color: gpui::Hsla) -> gpui::Stateful<gpui::Div> {
    if selected {
        el.bg(color.alpha(0.14)).text_color(color)
    } else {
        el
    }
}

/// Dropdown wrapper: trigger + floating menu panel (`Dropdown/DropdownTrigger/
/// DropdownMenu` composition).
#[derive(IntoElement)]
pub struct Dropdown {
    trigger: AnyElement,
    /// `isOpen` — `None` leaves the component holding the flag, seeded from
    /// `defaultOpen`.
    is_open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    items: Vec<MenuItem>,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    disabled_keys: Vec<SharedString>,
    indicator: IndicatorKind,
    on_selection_change: Option<OnSelectionChange>,
    on_action: Option<OnSelect>,
    placement: DropdownPlacement,
    on_toggle: Option<std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_select: Option<OnSelect>,
}

/// `placement` on `Dropdown.Popover`.
///
/// Shares the one placement vocabulary with the pickers and popover; it
/// previously offered only the two bottom-aligned values.
pub use herogpui_core::Placement as DropdownPlacement;

impl Dropdown {
    /// `onOpenChange` — the v3 name for [`Dropdown::on_toggle`], reporting the
    /// next open state rather than the raw click.
    pub fn on_open_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_change = Some(std::sync::Arc::new(handler));
        self
    }

    /// `isOpen` — also accepted positionally by [`Dropdown::new`].
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

    /// An uncontrolled dropdown: the menu holds its own open state, seeded
    /// from [`Dropdown::default_open`], and the trigger toggles it.
    pub fn uncontrolled(trigger: impl IntoElement, items: Vec<MenuItem>) -> Self {
        let mut dd = Self::new(trigger, items, false);
        dd.is_open = None;
        dd
    }

    pub fn new(trigger: impl IntoElement, items: Vec<MenuItem>, is_open: bool) -> Self {
        Self {
            trigger: trigger.into_any_element(),
            is_open: Some(is_open),
            default_open: false,
            on_open_change: None,
            items,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            disabled_keys: Vec::new(),
            indicator: IndicatorKind::default(),
            on_selection_change: None,
            on_action: None,
            placement: DropdownPlacement::BottomStart,
            on_toggle: None,
            on_select: None,
        }
    }

    /// `type` on `Dropdown.ItemIndicator`.
    pub fn indicator(mut self, kind: IndicatorKind) -> Self {
        self.indicator = kind;
        self
    }

    /// `selectionMode` on `Dropdown.Menu`.
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `selectedKeys` on `Dropdown.Menu`.
    pub fn selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.selected_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `disabledKeys` on `Dropdown.Menu`.
    pub fn disabled_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.disabled_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `onSelectionChange` on `Dropdown.Menu`.
    pub fn on_selection_change(
        mut self,
        f: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `onAction` on `Dropdown.Menu`.
    pub fn on_action(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_action = Some(std::sync::Arc::new(f));
        self
    }

    pub fn placement(mut self, p: DropdownPlacement) -> Self {
        self.placement = p;
        self
    }

    pub fn on_toggle(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(std::sync::Arc::new(f));
        self
    }

    pub fn on_select(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Dropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _ = icons::CHEVRON_DOWN;

        // `isOpen` wins; without it the menu holds the flag itself, which is
        // what `defaultOpen` promises. See `Dropdown::uncontrolled`.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            "dropdown-open",
            self.is_open,
            self.default_open,
        );

        let mut trigger_wrap = gpui::div().id("dropdown-trigger").cursor_pointer();
        let on_toggle = self.on_toggle.clone();
        let on_open_change = self.on_open_change.clone();
        if on_toggle.is_some() || on_open_change.is_some() || open_own.is_some() {
            let next_open = !is_open;
            let own = open_own.clone();
            trigger_wrap = trigger_wrap.on_click(move |ev: &ClickEvent, w, cx| {
                // Uncontrolled: flip our own copy, or the trigger would be
                // inert without a caller handler.
                if let Some(held) = &own {
                    held.update(cx, |v, cx| {
                        *v = next_open;
                        cx.notify();
                    });
                }
                if let Some(cb) = &on_toggle {
                    cb(ev, w, cx);
                }
                if let Some(cb) = &on_open_change {
                    cb(next_open, w, cx);
                }
            });
        }

        // A flex column with `items_start` keeps the trigger at its natural
        // width; a plain block root would stretch it (gpui divs are
        // Display::Block, so a block-level flex child fills the line).
        let mut root = gpui::div()
            .relative()
            .flex()
            .flex_col()
            .items_start()
            .child(trigger_wrap.child(self.trigger));

        if is_open {
            let mut menu = Menu::new(self.items)
                .selection_mode(self.selection_mode)
                .selected_keys(self.selected_keys.clone())
                .disabled_keys(self.disabled_keys.clone())
                .indicator(self.indicator);
            if let Some(on_select) = self.on_select.clone() {
                menu = menu.on_select(move |k, w, cx| {
                    on_select(k, w, cx);
                });
            }
            if let Some(on_action) = self.on_action.clone() {
                menu = menu.on_action(move |k, w, cx| on_action(k, w, cx));
            }
            if let Some(cb) = self.on_selection_change.clone() {
                menu = menu.on_selection_change(move |keys, w, cx| cb(keys, w, cx));
            }
            let anchor = crate::util::placed_panel(self.placement, px(6.));
            root = root.child(anchor.child(menu));
        }

        root
    }
}


