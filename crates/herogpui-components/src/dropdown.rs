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

type ItemContent =
    std::sync::Arc<dyn Fn(&SharedString, crate::util::InteractiveState) -> AnyElement + 'static>;

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
    /// Set by `Dropdown`: the menu panel is where Escape and an outside press
    /// land, and the open state belongs to the wrapper.
    on_dismiss: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
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
            on_dismiss: None,
        }
    }

    /// What to run when Escape or a press outside the panel dismisses the menu.
    ///
    /// Not a v3 prop: v3's `Dropdown.Menu` is inside the `Dropdown` that owns
    /// `isOpen`, and React Aria's `useOverlay` closes it from there. Crate-only,
    /// because only `Dropdown` can supply it.
    pub(crate) fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::sync::Arc::new(f));
        self
    }

    /// The element id every piece of this menu's state is keyed by.
    ///
    /// Not a v3 prop -- gpui needs an explicit id on a stateful element, and
    /// two menus that share one key share their focus, their cursor and their
    /// typeahead.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
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
    /// The closure receives the item's key and the row's state: `isSelected`,
    /// `isIndeterminate`, `isFocused`, `isPressed` and `isDisabled`, which are
    /// the values v3 passes into the same render prop. The press is a frame
    /// behind the pointer, because gpui reports it to a handler.
    pub fn item_content(
        mut self,
        render: impl Fn(&SharedString, crate::util::InteractiveState) -> AnyElement + 'static,
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
        // The keyboard's own state: which row it is on, the handle that receives
        // the keys, and the letters typed so far.
        let focus_handle = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-focus").into()),
            cx,
            |_, cx| cx.focus_handle().tab_stop(true),
        );
        let focus_handle = focus_handle.read(cx).clone();
        let cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-cursor").into()),
            cx,
            |_, _| None::<usize>,
        );
        let cursor_at = *cursor.read(cx);
        // `.dropdown__popover` is `overflow-y-auto`, and React Aria keeps the
        // focused row in view. `use_keyed_state` takes `cx` mutably, so the
        // handle precedes the theme.
        let menu_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-scroll").into()),
            cx,
            |_, _| gpui::ScrollHandle::new(),
        );
        let menu_scroll_now = menu_scroll.read(cx).clone();
        let typed = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-typed").into()),
            cx,
            |_, _| crate::list_nav::Typeahead::default(),
        );
        // A menu takes focus when it opens, which is what makes the arrows work
        // without a click first.
        crate::util::focus_once(
            window,
            cx,
            gpui::ElementId::Name(format!("{base}-autofocus").into()),
            &focus_handle,
        );

        // The rows a keyboard can land on -- an item that is not disabled -- and
        // the text a typed letter searches.
        let stops: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| match item {
                MenuItem::Item { key, .. } => !self.disabled_keys.contains(key),
                _ => false,
            })
            .map(|(i, _)| i)
            .collect();
        let labels: Vec<String> = self
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Item { label, .. } => label.to_string(),
                _ => String::new(),
            })
            .collect();
        let item_keys: Vec<SharedString> = self
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Item { key, .. } => key.clone(),
                _ => SharedString::default(),
            })
            .collect();

        let colors = cx.colors();

        let mut panel = gpui::div()
            .flex()
            .flex_col()
            // `.dropdown__popover` is `md:min-w-55` (220px) and the menu inside
            // it is `gap-0.5 p-1` -- `.dropdown__menu` overrides `.menu`'s
            // `gap-1` with half a step.
            .min_w(px(220.))
            .gap(px(2.))
            .p(px(4.))
            .bg(colors.overlay.background)
            .rounded(crate::util::container_radius(cx))
            .shadow(cx.layout().overlay_shadow.clone())
            // A long menu scrolls rather than being clipped, and gpui needs an
            // id for that. React Aria sizes the popover to the space the
            // viewport leaves; the closest thing here is a share of the window,
            // since a menu is anchored to a trigger that can be anywhere in it.
            .id(gpui::ElementId::Name(format!("{base}-list").into()))
            .max_h(window.viewport_size().height * 0.6)
            .overflow_y_scroll()
            .track_scroll(&menu_scroll_now)
            .track_focus(&focus_handle)
            .key_context("Menu");

        // v3 gives a floating panel no border: it is `bg-overlay shadow-overlay`
        // and a radius, and dark mode's inset hairline is what separates the
        // panel from the page.
        if let Some(hairline) = cx.layout().overlay_hairline {
            panel = panel
                .border(cx.layout().border_width)
                .border_color(hairline);
        }

        if !stops.is_empty() {
            let held = cursor;
            let stops_for_keys = stops;
            let typed_keys = typed;
            let on_action = self.on_action.clone();
            let on_selection_change = self.on_selection_change.clone();
            let key_scroll = menu_scroll_now;
            let mode = self.selection_mode;
            let selected_now = self.selected_keys.clone();
            let keys = item_keys;
            panel = panel.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let from = *held.read(cx);
                match crate::list_nav::resolve(&stops_for_keys, from, key, false) {
                    crate::list_nav::Move::To(next) => {
                        held.update(cx, |v, cx| {
                            *v = Some(next);
                            cx.notify();
                        });
                        // Keep the focused row on screen: a highlight that walks
                        // out of the panel reads as the arrows having stopped.
                        key_scroll.scroll_to_item(next);
                    }
                    crate::list_nav::Move::Activate => {
                        let Some(item_key) = from.and_then(|i| keys.get(i).cloned()) else {
                            return;
                        };
                        // The same two callbacks a click fires, in the same
                        // order, so a keyboard choice is not a different event.
                        if let Some(cb) = &on_action {
                            cb(&item_key, window, cx);
                        }
                        if let Some(cb) = &on_selection_change {
                            let next = crate::selection::next_selection(
                                &selected_now,
                                &item_key,
                                mode,
                                false,
                            );
                            cb(&next, window, cx);
                        }
                    }
                    crate::list_nav::Move::Ignore => {
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
                            .px(px(8.))
                            .pt(px(6.))
                            .pb(px(4.))
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::MEDIUM)
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
                    let mut row = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-item-{i}").into()))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .px(px(8.))
                        .rounded(crate::util::soft_radius(cx))
                        .text_size(px(14.))
                        .text_color(text_color);
                    // `.menu-item` is `min-h-9 py-1.5`; a described row grows
                    // past the minimum instead of clipping its second line.
                    row = row.min_h(px(36.)).py(px(6.));
                    if is_item_disabled {
                        // `status-disabled` is `--disabled-opacity`; the muted
                        // text alone was this port's own idea of the state.
                        row = row.opacity(cx.layout().disabled_opacity);
                    } else {
                        row = row.cursor_pointer();
                        row = row.hover(move |s| s.bg(colors.default.soft()));
                        // `.menu-item[data-pressed]` is `scale(0.98)`.
                        row = crate::anim::pressed(
                            row,
                            crate::anim::PressBox {
                                height: px(36.),
                                padding_x: Some(px(8.)),
                                width: None,
                                min_width: None,
                                text_size: px(14.),
                                line_height: px(20.),
                                gap: px(12.),
                                radius: crate::util::soft_radius(cx),
                                shrink_x: true,
                                scale: crate::anim::PRESSED_SCALE_SUBTLE,
                            },
                            cx,
                        );
                    }
                    row = when_selected(row, is_selected, sem_primary(cx));
                    // `.menu-item` takes `status-focused` on the row the keyboard
                    // is on -- a ring, not a border, which would shift the row.
                    row = crate::util::with_focus_ring(
                        row,
                        cursor_at == Some(i),
                        true,
                        Vec::new(),
                        cx,
                    );

                    if let Some(icon_path) = icon {
                        row = row.child(
                            gpui::svg()
                                // `.menu-item__indicator` is `size-4`.
                                .size(px(16.))
                                .path(icon_path)
                                .text_color(text_color),
                        );
                    }
                    // `children` on `Dropdown.Item` is a render function in
                    // v3, handed the row's state. A multi-selection item is
                    // indeterminate when some but not all of the menu's keys are
                    // chosen.
                    let is_indeterminate = self.selection_mode == SelectionMode::Multiple
                        && !self.selected_keys.is_empty()
                        && !is_selected;
                    row = row.child(
                        gpui::div().flex_1().child(match &self.item_content {
                            Some(render) => render(
                                &key,
                                crate::util::InteractiveState {
                                    // The pointer state a menu row reports comes
                                    // from the same slot the press animation
                                    // uses; a row is focused when the keyboard
                                    // cursor is on it.
                                    is_hovered: false,
                                    is_pressed: false,
                                    is_focused: cursor_at == Some(i),
                                    is_focus_visible: cursor_at == Some(i)
                                        && crate::util::focus_visible(cx),
                                    is_selected,
                                    is_disabled: is_item_disabled,
                                    is_indeterminate,
                                },
                            ),
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
                                            // A described row composes a
                                            // `Description`, which is `text-xs`.
                                            .text_size(px(12.))
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
                                .text_size(px(12.))
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

        // React Aria dismisses the menu on Escape and on a press outside it.
        let panel = match self.on_dismiss.clone() {
            Some(cb) => crate::util::dismissable(panel, move |window, cx| cb(window, cx)),
            None => panel,
        };

        let zoom = crate::anim::ZoomBox::panel(px(6.), crate::util::container_radius(cx));
        crate::util::floating(if self.exiting {
            crate::anim::exiting(
                panel,
                gpui::ElementId::Name(format!("{base}-panel-out").into()),
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                panel,
                gpui::ElementId::Name(format!("{base}-panel").into()),
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
    /// Keys this dropdown's own state; see [`Dropdown::id`].
    id: gpui::ElementId,
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
    /// The element id this dropdown's state is keyed by.
    ///
    /// Not a v3 prop. It matters more here than it looks: with one shared key,
    /// pressing any trigger on a page opened *every* menu on it, because they
    /// were all reading the same uncontrolled open flag.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

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
            id: gpui::ElementId::Name("dropdown".into()),
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
        let wrap_base = format!("{:?}", self.id);

        // `isOpen` wins; without it the menu holds the flag itself, which is
        // what `defaultOpen` promises. See `Dropdown::uncontrolled`.
        let (is_open, open_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{wrap_base}-open").into()),
            self.is_open,
            self.default_open,
        );
        // `overlay_phase` takes `cx` mutably too, so it goes here.
        let phase = crate::util::overlay_phase(
            window,
            cx,
            gpui::ElementId::Name(format!("{wrap_base}-phase").into()),
            is_open,
        );

        // `trigger="longPress"` needs to know whether the button is still down
        // when the timer fires, so the press is a piece of state rather than a
        // local.
        let holding = window.use_keyed_state(
            gpui::ElementId::Name(format!("{wrap_base}-holding").into()),
            cx,
            |_, _| false,
        );

        // Where the focus goes when the menu closes. React Aria hands it back
        // to the trigger, and the trigger element is the caller's, so the
        // wrapper holds the handle. It is deliberately *not* a tab stop: gpui
        // keeps any tracked handle in the tab order, so Tab carries on from here
        // instead of starting the page over.
        let trigger_focus = window.use_keyed_state(
            gpui::ElementId::Name(format!("{wrap_base}-trigger-focus").into()),
            cx,
            |_, cx| cx.focus_handle(),
        );
        let trigger_handle = trigger_focus.read(cx).clone();
        let mut trigger_wrap = gpui::div()
            .id(gpui::ElementId::Name(format!("{wrap_base}-trigger").into()))
            .track_focus(&trigger_handle)
            .cursor_pointer();
        let dismiss_own = open_own.clone();
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
            // `.dropdown` is `flex flex-col gap-1`.
            .gap(px(4.))
            .items_start()
            .child(trigger_wrap.child(self.trigger));

        // v3 keeps a closing menu on screen for its `[data-exiting]` run.
        if phase != crate::util::OverlayPhase::Closed {
            let mut menu = Menu::new(self.items)
                .id(gpui::ElementId::Name(format!("{wrap_base}-menu").into()))
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
            let dismiss_cb = self.on_open_change.clone();
            if dismiss_cb.is_some() || dismiss_own.is_some() {
                let back_to_trigger = trigger_handle;
                menu = menu.on_dismiss(move |window, cx| {
                    if let Some(held) = &dismiss_own {
                        held.update(cx, |v, cx| {
                            *v = false;
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &dismiss_cb {
                        cb(false, window, cx);
                    }
                    // The menu held the focus for its arrows; hand it back.
                    window.focus(&back_to_trigger);
                });
            }
            let anchor = crate::util::placed_panel(self.placement, px(6.));
            root = root.child(anchor.child(menu));
        }

        root
    }
}
