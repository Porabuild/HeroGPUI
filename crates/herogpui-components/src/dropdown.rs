//! Dropdown & Menu — port of `@heroui/dropdown`, `@heroui/menu` and
//! `@heroui/listbox`.

use gpui::{
    px, AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
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
        /// `Description` inside a `Dropdown.Item` — v3's "With Descriptions".
        description: Option<SharedString>,
        /// `Dropdown.SubmenuTrigger` — the rows this item opens. The row grows a
        /// trailing indicator and the panel appears beside it.
        submenu: Vec<MenuItem>,
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
            description: None,
            submenu: Vec::new(),
        }
    }

    /// `Description` — the second line v3 composes inside an item.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        if let MenuItem::Item { description, .. } = &mut self {
            *description = Some(text.into());
        }
        self
    }

    /// `Dropdown.SubmenuTrigger` — the rows this item opens.
    pub fn submenu(mut self, items: Vec<MenuItem>) -> Self {
        if let MenuItem::Item { submenu, .. } = &mut self {
            *submenu = items;
        }
        self
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

type ItemContent = std::sync::Arc<dyn Fn(&SharedString, bool, bool) -> AnyElement + 'static>;

#[derive(IntoElement)]
pub struct Menu {
    /// Set by `Dropdown` while the menu is playing its `[data-exiting]` run.
    exiting: bool,
    /// `children` on `Dropdown.Item` — v3's render prop, handed the item's
    /// key, `isSelected` and `isIndeterminate`.
    item_content: Option<ItemContent>,
    id: gpui::ElementId,
    items: Vec<MenuItem>,
    selected_key: Option<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    disabled_keys: Vec<SharedString>,
    indicator: IndicatorKind,
    on_selection_change: Option<OnSelectionChange>,
    on_action: Option<OnSelect>,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            exiting: false,
            item_content: None,
            id: gpui::ElementId::Name("menu".into()),
            items,
            selected_key: None,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            disabled_keys: Vec::new(),
            indicator: IndicatorKind::default(),
            on_selection_change: None,
            on_action: None,
        }
    }

    /// Plays the menu's exit instead of its entry.
    ///
    /// Not a v3 prop: v3's menu leaves the tree with a `[data-exiting]`
    /// attribute, and this is the flag that stands in for it.
    pub fn exiting(mut self, v: bool) -> Self {
        self.exiting = v;
        self
    }

    /// `children` on `Dropdown.Item` — replaces an item's label.
    ///
    /// The closure receives the item's key, `isSelected` and
    /// `isIndeterminate`, the values v3 passes into the same render prop.
    pub fn item_content(
        mut self,
        render: impl Fn(&SharedString, bool, bool) -> AnyElement + 'static,
    ) -> Self {
        self.item_content = Some(std::sync::Arc::new(render));
        self
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
    pub fn on_action(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(std::sync::Arc::new(f));
        self
    }

    pub fn selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.selected_key = Some(key.into());
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = format!("{:?}", self.id);
        // Which submenu is open, if any. `use_keyed_state` takes `cx` mutably,
        // so it precedes everything that borrows the theme.
        let submenu_state = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-submenu").into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let submenu_open = submenu_state.read(cx).clone();
        let colors = cx.colors();

        let mut panel = gpui::div()
            .flex()
            .flex_col()
            .min_w(px(180.))
            .py(px(6.))
            .bg(colors.overlay.background)
            .rounded(crate::util::container_radius(cx))
            .border_1()
            .border_color(colors.separator)
            .shadow(cx.layout().overlay_shadow.clone())
            .overflow_hidden();

        for (i, item) in self.items.into_iter().enumerate() {
            match item {
                MenuItem::Separator => {
                    panel = panel.child(
                        gpui::div()
                            .w_full()
                            .my(px(4.))
                            .h(cx.layout().border_width)
                            .bg(colors.separator),
                    );
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
                    description,
                    submenu,
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
                    let has_description = description.is_some();
                    let mut row = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-item-{i}").into()))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(8.))
                        .rounded(crate::util::soft_radius(cx))
                        .text_size(px(13.5))
                        .text_color(text_color);
                    // A described row grows instead of clipping its second line,
                    // which is what `.list-box-item`'s `min-h-9` does.
                    row = if has_description {
                        row.min_h(px(32.)).py(px(6.))
                    } else {
                        row.h(px(32.))
                    };
                    if !is_item_disabled {
                        row = row.cursor_pointer();
                        row = row.hover(move |s| s.bg(colors.default.soft()));
                    }
                    row = when_selected(row, is_selected, sem_primary(cx));

                    if let Some(icon_path) = icon {
                        row = row.child(
                            gpui::svg()
                                .size(px(15.))
                                .path(icon_path)
                                .text_color(text_color),
                        );
                    }
                    // `children` on `Dropdown.Item` is a render function in
                    // v3, handed `isSelected` and `isIndeterminate`. A
                    // multi-selection item is indeterminate when some but not
                    // all of the menu's keys are chosen.
                    let is_indeterminate = self.selection_mode == SelectionMode::Multiple
                        && !self.selected_keys.is_empty()
                        && !is_selected;
                    row = row.child(
                        gpui::div().flex_1().child(match &self.item_content {
                            Some(render) => render(&key, is_selected, is_indeterminate),
                            None => match &description {
                                // `Label` over `Description`, which is how v3
                                // composes a described item.
                                Some(text) => gpui::div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.))
                                    .child(gpui::div().child(label.to_string()))
                                    .child(
                                        gpui::div()
                                            .text_size(px(11.5))
                                            .text_color(colors.muted)
                                            .child(text.to_string()),
                                    )
                                    .into_any_element(),
                                None => label.to_string().into_any_element(),
                            },
                        }),
                    );
                    if let Some(sc) = shortcut {
                        row = row.child(
                            gpui::div()
                                .text_size(px(11.5))
                                .text_color(colors.muted)
                                .child(sc.to_string()),
                        );
                    }
                    // `Dropdown.SubmenuIndicator` — the chevron that says a row
                    // opens another panel.
                    let has_submenu = !submenu.is_empty();
                    if has_submenu {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHEVRON_RIGHT)
                                .text_color(colors.muted),
                        );
                    }

                    if is_selected && self.selection_mode != SelectionMode::None {
                        row = match self.indicator {
                            IndicatorKind::Checkmark => row.child(
                                gpui::svg()
                                    .size(px(13.))
                                    .path(icons::CHECK)
                                    // svg() never inherits text colour.
                                    .text_color(sem_primary(cx)),
                            ),
                            IndicatorKind::Dot => row
                                .child(gpui::div().size(px(6.)).rounded_full().bg(sem_primary(cx))),
                        };
                    }

                    if !is_item_disabled {
                        let on_action = self.on_action.clone();
                        let on_selection_change = self.on_selection_change.clone();
                        if on_action.is_some() || on_selection_change.is_some() {
                            let key2 = key.clone();
                            let mode = self.selection_mode;
                            let current = self.selected_keys.clone();
                            row = row.on_click(move |_, window, cx| {
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

                    // `Dropdown.SubmenuTrigger`: the child panel is anchored to
                    // the row and opens while the row is hovered. gpui paints in
                    // tree order, so it goes through `util::floating` like every
                    // other floating surface.
                    if has_submenu {
                        let open_key = SharedString::from(format!("{base}-sub-{i}"));
                        let is_sub_open = submenu_open.as_ref() == Some(&open_key);
                        let held = submenu_state.clone();
                        let open_key2 = open_key.clone();
                        row = row.on_hover(move |hovered, _window, cx| {
                            let next = if *hovered {
                                Some(open_key2.clone())
                            } else {
                                None
                            };
                            held.update(cx, |v, cx| {
                                if *v != next {
                                    *v = next.clone();
                                    cx.notify();
                                }
                            });
                        });
                        let mut slot = gpui::div().relative().child(row);
                        if is_sub_open {
                            slot = slot.child(crate::util::floating(
                                gpui::div()
                                    .absolute()
                                    .left_full()
                                    .top(px(-6.))
                                    .ml(px(4.))
                                    .child({
                                        let mut sub = Menu::new(submenu).indicator(self.indicator);
                                        if let Some(cb) = self.on_action.clone() {
                                            sub = sub.on_action(move |key, window, cx| {
                                                cb(key, window, cx);
                                            });
                                        }
                                        sub
                                    }),
                            ));
                        }
                        panel = panel.child(slot);
                    } else {
                        panel = panel.child(row);
                    }
                }
            }
        }

        let zoom = crate::anim::ZoomBox::panel(px(6.), crate::util::container_radius(cx));
        crate::util::floating(if self.exiting {
            crate::anim::exiting(
                panel,
                "dropdown-panel-out",
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                "dropdown-panel",
                zoom,
                crate::anim::Motion::POPOVER_IN,
                cx,
            )
        })
    }
}

fn sem_primary(cx: &App) -> gpui::Hsla {
    cx.colors().accent.color
}

fn when_selected(
    el: gpui::Stateful<gpui::Div>,
    selected: bool,
    color: gpui::Hsla,
) -> gpui::Stateful<gpui::Div> {
    if selected {
        el.bg(color.alpha(0.14)).text_color(color)
    } else {
        el
    }
}

/// `trigger` — what opens the menu.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DropdownTrigger {
    /// A press opens it, which is v3's default.
    #[default]
    Press,
    /// A press held for `LONG_PRESS_MS` opens it.
    LongPress,
}

/// How long `trigger="longPress"` waits. React Aria uses 500ms.
const LONG_PRESS_MS: u64 = 500;

/// Dropdown wrapper: trigger + floating menu panel (`Dropdown/DropdownTrigger/
/// DropdownMenu` composition).
#[derive(IntoElement)]
pub struct Dropdown {
    trigger: AnyElement,
    /// `trigger` — press (the default) or long press.
    trigger_kind: DropdownTrigger,
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
}

/// `placement` on `Dropdown.Popover`.
///
/// Shares the one placement vocabulary with the pickers and popover; it
/// previously offered only the two bottom-aligned values.
pub use herogpui_core::Placement as DropdownPlacement;

impl Dropdown {
    /// `onOpenChange` — reports the open state the trigger moves to.
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
    /// `trigger` — `Press` (default) or `LongPress`.
    pub fn trigger(mut self, kind: DropdownTrigger) -> Self {
        self.trigger_kind = kind;
        self
    }

    pub fn uncontrolled(trigger: impl IntoElement, items: Vec<MenuItem>) -> Self {
        let mut dd = Self::new(trigger, items, false);
        dd.is_open = None;
        dd
    }

    pub fn new(trigger: impl IntoElement, items: Vec<MenuItem>, is_open: bool) -> Self {
        Self {
            trigger: trigger.into_any_element(),
            trigger_kind: DropdownTrigger::default(),
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
    pub fn on_action(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(std::sync::Arc::new(f));
        self
    }

    pub fn placement(mut self, p: DropdownPlacement) -> Self {
        self.placement = p;
        self
    }
}

impl RenderOnce for Dropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _ = icons::CHEVRON_DOWN;

        // `isOpen` wins; without it the menu holds the flag itself, which is
        // what `defaultOpen` promises. See `Dropdown::uncontrolled`.
        let (is_open, open_own) =
            crate::util::controlled(window, cx, "dropdown-open", self.is_open, self.default_open);
        // `overlay_phase` takes `cx` mutably too, so it goes here.
        let phase = crate::util::overlay_phase(window, cx, "dropdown-phase", is_open);

        // `trigger="longPress"` needs to know whether the button is still down
        // when the timer fires, so the press is a piece of state rather than a
        // local.
        let holding = window.use_keyed_state("dropdown-holding", cx, |_, _| false);

        let mut trigger_wrap = gpui::div().id("dropdown-trigger").cursor_pointer();
        let on_open_change = self.on_open_change.clone();
        if on_open_change.is_some() || open_own.is_some() {
            let next_open = !is_open;
            let own = open_own;
            match self.trigger_kind {
                DropdownTrigger::Press => {
                    trigger_wrap = trigger_wrap.on_click(move |_ev: &ClickEvent, w, cx| {
                        // Uncontrolled: flip our own copy, or the trigger would
                        // be inert without a caller handler.
                        if let Some(held) = &own {
                            held.update(cx, |v, cx| {
                                *v = next_open;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_open_change {
                            cb(next_open, w, cx);
                        }
                    });
                }
                DropdownTrigger::LongPress => {
                    let up_holding = holding.clone();
                    trigger_wrap = trigger_wrap
                        .on_mouse_down(gpui::MouseButton::Left, {
                            let holding = holding;
                            move |_, window, cx| {
                                holding.update(cx, |v, _| *v = true);
                                let holding = holding.clone();
                                let own = own.clone();
                                let on_open_change = on_open_change.clone();
                                // Open only if the button is still down when the
                                // timer expires; a quick click leaves it shut.
                                // `window.spawn` rather than `cx.spawn`: the
                                // callback needs a `Window`, and only a window
                                // async context can hand one back.
                                window
                                    .spawn(cx, async move |cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(LONG_PRESS_MS))
                                            .await;
                                        cx.update(|window, cx| {
                                            if !*holding.read(cx) {
                                                return;
                                            }
                                            if let Some(held) = &own {
                                                held.update(cx, |v, cx| {
                                                    *v = true;
                                                    cx.notify();
                                                });
                                            }
                                            if let Some(cb) = &on_open_change {
                                                cb(true, window, cx);
                                            }
                                        })
                                        .ok();
                                    })
                                    .detach();
                            }
                        })
                        .on_mouse_up(gpui::MouseButton::Left, move |_, _window, cx| {
                            up_holding.update(cx, |v, _| *v = false);
                        });
                }
            }
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

        // v3 keeps a closing menu on screen for its `[data-exiting]` run.
        if phase != crate::util::OverlayPhase::Closed {
            let mut menu = Menu::new(self.items)
                .exiting(phase == crate::util::OverlayPhase::Exiting)
                .selection_mode(self.selection_mode)
                .selected_keys(self.selected_keys.clone())
                .disabled_keys(self.disabled_keys.clone())
                .indicator(self.indicator);
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
