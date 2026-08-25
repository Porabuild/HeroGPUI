//! Dropdown & Menu — port of `@heroui/dropdown`, `@heroui/menu` and
//! `@heroui/listbox`.

use gpui::{
    px, AnyElement, App, Bounds, ClickEvent, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
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
type ItemIndicatorContent =
    std::sync::Arc<dyn Fn(&SharedString, bool, bool) -> AnyElement + 'static>;
type OnDismiss = std::rc::Rc<dyn Fn(bool, &mut Window, &mut App) + 'static>;
type PanelBounds = std::rc::Rc<std::cell::RefCell<Vec<Bounds<Pixels>>>>;

#[derive(IntoElement)]
pub struct Menu {
    /// Set by `Dropdown` while the menu is playing its `[data-exiting]` run.
    exiting: bool,
    /// Submenus are already inside their parent's deferred draw and cannot
    /// defer a second time.
    deferred: bool,
    panel_bounds: Option<PanelBounds>,
    focus_first: Option<gpui::Entity<bool>>,
    on_back: Option<std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>>,
    /// `children` on `Dropdown.Item` — v3's render prop, handed the item's
    /// key, selection, focus, disabled and pressed state.
    item_content: Option<ItemContent>,
    /// `children` on `Dropdown.ItemIndicator` — handed the item's key,
    /// `isSelected` and `isIndeterminate`.
    indicator_content: Option<ItemIndicatorContent>,
    id: gpui::ElementId,
    items: Vec<MenuItem>,
    selected_key: Option<SharedString>,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    default_selected_keys: Vec<SharedString>,
    selection_is_controlled: bool,
    disallow_empty_selection: bool,
    disabled_keys: Vec<SharedString>,
    indicator: IndicatorKind,
    on_selection_change: Option<OnSelectionChange>,
    on_action: Option<OnSelect>,
    /// Set by `Dropdown`: the menu panel is where Escape and an outside press
    /// land, and the open state belongs to the wrapper. The `bool` says
    /// whether the trigger should take the focus back: Escape, an outside
    /// press and a mouse pick can, because no key-up follows them; an Enter
    /// pick cannot, because gpui activates a focused element on key up and the
    /// trigger's click listener would reopen the menu it just closed.
    on_dismiss: Option<OnDismiss>,
    overlay_token: Option<crate::util::OverlayToken>,
    dropdown_composition: bool,
}

impl Menu {
    pub fn new(id: impl Into<gpui::ElementId>, items: Vec<MenuItem>) -> Self {
        Self {
            exiting: false,
            deferred: true,
            panel_bounds: None,
            focus_first: None,
            on_back: None,
            item_content: None,
            indicator_content: None,
            id: id.into(),
            items,
            selected_key: None,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            default_selected_keys: Vec::new(),
            selection_is_controlled: false,
            disallow_empty_selection: false,
            disabled_keys: Vec::new(),
            indicator: IndicatorKind::default(),
            on_selection_change: None,
            on_action: None,
            on_dismiss: None,
            overlay_token: None,
            dropdown_composition: false,
        }
    }

    /// What to run when the menu closes: an item activation, Escape, or a
    /// press outside the panel.
    ///
    /// Not a v3 prop: v3's `Dropdown.Menu` is inside the `Dropdown` that owns
    /// `isOpen`, and React Aria's `useOverlay` closes it from there. Crate-only,
    /// because only `Dropdown` can supply it. The `bool` is whether to return
    /// the focus to the trigger — see the field docs for why a key pick passes
    /// `false`.
    pub(crate) fn on_dismiss(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(f));
        self
    }

    pub(crate) fn overlay_token(mut self, token: crate::util::OverlayToken) -> Self {
        self.overlay_token = Some(token);
        self
    }

    pub(crate) fn dropdown_composition(mut self) -> Self {
        self.dropdown_composition = true;
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

    pub(crate) fn embedded(mut self, panel_bounds: PanelBounds) -> Self {
        self.deferred = false;
        self.panel_bounds = Some(panel_bounds);
        self
    }

    pub(crate) fn focus_first(mut self, state: gpui::Entity<bool>) -> Self {
        self.focus_first = Some(state);
        self
    }

    pub(crate) fn on_back(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(std::sync::Arc::new(f));
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

    /// `children` on `Dropdown.ItemIndicator` — replaces the built-in mark.
    pub fn indicator_content(
        mut self,
        render: impl Fn(&SharedString, bool, bool) -> AnyElement + 'static,
    ) -> Self {
        self.indicator_content = Some(std::sync::Arc::new(render));
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
        self.selection_is_controlled = true;
        self
    }

    /// `defaultSelectedKeys` — seeds the menu's own selection state.
    pub fn default_selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_selected_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `disallowEmptySelection` — prevents removing the last selected item.
    pub fn disallow_empty_selection(mut self, value: bool) -> Self {
        self.disallow_empty_selection = value;
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base = format!("{:?}", self.id);
        let overlay_token = if let Some(token) = self.overlay_token.clone() {
            Some(token)
        } else if self.on_dismiss.is_some() {
            let (_, token) = crate::util::overlay_scope(
                window,
                cx,
                gpui::ElementId::Name(format!("{base}-overlay").into()),
                true,
                self.exiting,
            );
            Some(token)
        } else {
            None
        };
        let (selected_keys, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{base}-selected").into()),
            self.selection_is_controlled
                .then(|| self.selected_keys.clone()),
            self.default_selected_keys.clone(),
        );
        self.selected_keys = selected_keys;
        // Which submenu is open, if any. `use_keyed_state` takes `cx` mutably,
        // so it precedes everything that borrows the theme.
        let submenu_state = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-submenu").into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let submenu_open = submenu_state.read(cx).clone();
        let submenu_focus = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-submenu-focus").into()),
            cx,
            |_, _| false,
        );
        let focus_first = self
            .focus_first
            .as_ref()
            .is_some_and(|state| *state.read(cx));
        let dismiss = self.on_dismiss.clone().map(|cb| {
            let submenu_state = submenu_state.clone();
            let submenu_focus = submenu_focus.clone();
            std::rc::Rc::new(move |refocus, window: &mut Window, cx: &mut App| {
                submenu_state.update(cx, |value, cx| {
                    if value.is_some() {
                        *value = None;
                        cx.notify();
                    }
                });
                submenu_focus.update(cx, |value, _| *value = false);
                cb(refocus, window, cx);
            }) as OnDismiss
        });
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
        let mut cursor_at = *cursor.read(cx);
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
        // One hover/press slot per item, for an `item_content` closure. The
        // slots exist only when the closure is set: `track_interaction`'s
        // handlers cost a frame of state, and the closure is the only reader
        // (the press v3's `Dropdown.Item` render props document).
        let interaction: Vec<crate::util::Interaction> = if self.item_content.is_some() {
            (0..self.items.len())
                .map(|i| {
                    crate::util::interaction(
                        gpui::ElementId::Name(format!("{base}-item-{i}-interaction").into()),
                        window,
                        cx,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        // A menu takes focus when it opens, which is what makes the arrows work
        // without a click first. The one-shot re-arms while the menu plays its
        // exit, so a menu that reopens after a dismissal -- a pick or Escape
        // hands the focus back to the trigger -- is keyboard-driven again.
        let autofocus = gpui::ElementId::Name(format!("{base}-autofocus").into());
        if self.exiting {
            let done = window.use_keyed_state(autofocus, cx, |_, _| false);
            done.update(cx, |d, _| *d = false);
        } else if focus_first {
            window.focus(&focus_handle);
        } else {
            crate::util::focus_once(window, cx, autofocus, &focus_handle);
        }

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
        if focus_first {
            if let Some(first) = stops.first().copied() {
                cursor.update(cx, |value, cx| {
                    *value = Some(first);
                    cx.notify();
                });
                cursor_at = Some(first);
            }
            if let Some(state) = &self.focus_first {
                state.update(cx, |value, _| *value = false);
            }
        }
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
        // Whether each row is a submenu trigger. Such a row opens a child
        // panel instead of ending the menu, so activating it must not close
        // the parent -- React Aria returns before the close for a trigger.
        let item_has_submenu: Vec<bool> = self
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Item { submenu, .. } => !submenu.is_empty(),
                _ => false,
            })
            .collect();
        let panel_bounds = window.use_keyed_state(
            gpui::ElementId::Name(format!("{base}-panel-bounds").into()),
            cx,
            |_, _| None::<Bounds<Pixels>>,
        );
        let item_bounds = self
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Item { key, submenu, .. } if !submenu.is_empty() => {
                    Some(window.use_keyed_state(
                        gpui::ElementId::Name(format!("{base}-item-{key}-bounds").into()),
                        cx,
                        |_, _| None::<Bounds<Pixels>>,
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let all_panel_bounds = self
            .panel_bounds
            .clone()
            .unwrap_or_else(|| std::rc::Rc::new(std::cell::RefCell::new(Vec::new())));

        let colors = cx.colors();
        let dropdown_composition = self.dropdown_composition;

        let mut panel = gpui::div()
            .relative()
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
        if dropdown_composition {
            // Dropdown's nested `[data-slot="dropdown-menu"]` overrides the
            // standalone Menu p-1 inset with p-1.5.
            panel = panel.p(px(6.));
        }

        let recorded_panel_bounds = panel_bounds.clone();
        let registered_panel_bounds = all_panel_bounds.clone();
        panel = panel.child(
            gpui::canvas(
                move |bounds, _, cx| {
                    registered_panel_bounds.borrow_mut().push(bounds);
                    recorded_panel_bounds.update(cx, |value, cx| {
                        if value.as_ref() != Some(&bounds) {
                            *value = Some(bounds);
                            cx.notify();
                        }
                    });
                    bounds
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        );

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
            let selection_own_for_keys = selection_own.clone();
            let key_scroll = menu_scroll_now;
            let mode = self.selection_mode;
            let disallow_empty = self.disallow_empty_selection;
            let selected_now = self.selected_keys.clone();
            let keys = item_keys;
            let has_submenu = item_has_submenu;
            let submenu_open_for_keys = submenu_state.clone();
            let submenu_focus_for_keys = submenu_focus.clone();
            let submenu_base_for_keys = base.clone();
            let on_back = self.on_back.clone();
            let local_submenu = submenu_state.clone();
            let local_submenu_focus = submenu_focus.clone();
            let dismiss = dismiss.clone();
            panel = panel.on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                let from = *held.read(cx);
                if key == "left" {
                    if let Some(cb) = &on_back {
                        local_submenu.update(cx, |value, cx| {
                            if value.is_some() {
                                *value = None;
                                cx.notify();
                            }
                        });
                        local_submenu_focus.update(cx, |value, _| *value = false);
                        cb(window, cx);
                        cx.stop_propagation();
                    }
                    return;
                }
                if key == "right" {
                    let Some(i) = from else {
                        return;
                    };
                    if has_submenu.get(i).copied().unwrap_or(false) {
                        let Some(item_key) = keys.get(i) else {
                            return;
                        };
                        let open_key =
                            SharedString::from(format!("{submenu_base_for_keys}-sub-{item_key}"));
                        submenu_open_for_keys.update(cx, |value, cx| {
                            *value = Some(open_key);
                            cx.notify();
                        });
                        submenu_focus_for_keys.update(cx, |value, _| *value = true);
                        cx.stop_propagation();
                    }
                    return;
                }
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
                        let Some(i) = from else {
                            return;
                        };
                        let Some(item_key) = keys.get(i).cloned() else {
                            return;
                        };
                        let has_submenu = has_submenu[i];
                        // A submenu trigger opens its child; it is neither a
                        // selection nor a menu-level action in React Aria.
                        if has_submenu {
                            let open_key = SharedString::from(format!(
                                "{submenu_base_for_keys}-sub-{item_key}"
                            ));
                            submenu_open_for_keys.update(cx, |value, cx| {
                                *value = Some(open_key);
                                cx.notify();
                            });
                            submenu_focus_for_keys.update(cx, |value, _| *value = true);
                            return;
                        }
                        if crate::selection::reports_changes(mode) {
                            let next = crate::selection::next_selection(
                                &selected_now,
                                &item_key,
                                mode,
                                disallow_empty,
                            );
                            let blocked_last_removal = disallow_empty
                                && selected_now.len() == 1
                                && selected_now.contains(&item_key);
                            if !blocked_last_removal {
                                if let Some(held) = &selection_own_for_keys {
                                    held.update(cx, |value, cx| {
                                        *value = next.clone();
                                        cx.notify();
                                    });
                                }
                                if let Some(cb) = &on_selection_change {
                                    cb(&next, window, cx);
                                }
                            }
                        }
                        if let Some(cb) = &on_action {
                            cb(&item_key, window, cx);
                        }
                        // React Aria always closes for Enter. Space stays open
                        // only in multiple mode, so another item can be ticked.
                        // The focus is deliberately not sent back to the
                        // trigger from a key: gpui activates a focused element
                        // on key up, which would reopen the menu.
                        if key == "enter" || mode != SelectionMode::Multiple {
                            if let Some(cb) = &dismiss {
                                cb(false, window, cx);
                            }
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

        let mut open_submenu = None;
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
                    let has_submenu = !submenu.is_empty();
                    let text_color = if is_item_disabled {
                        colors.muted
                    } else if is_danger {
                        colors.danger.color
                    } else {
                        colors.foreground
                    };
                    let mut row = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-item-{i}").into()))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .px(px(8.))
                        .rounded(crate::util::soft_radius(cx))
                        .text_size(px(14.))
                        .text_color(text_color);
                    if let Some(recorded_item_bounds) = item_bounds[i].clone() {
                        row = row.child(
                            gpui::canvas(
                                move |bounds, _, cx| {
                                    recorded_item_bounds.update(cx, |value, cx| {
                                        if value.as_ref() != Some(&bounds) {
                                            *value = Some(bounds);
                                            cx.notify();
                                        }
                                    });
                                    bounds
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        );
                    }
                    // `.menu-item` is `min-h-9 py-1.5`; a described row grows
                    // past the minimum instead of clipping its second line.
                    row = row.min_h(px(36.)).py(px(6.));
                    if dropdown_composition {
                        // Dropdown's nested `[data-slot="menu-item"]` uses
                        // px-2.5, while standalone Menu remains px-2.
                        row = row.px(px(10.));
                    }
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
                                padding_x: Some(if dropdown_composition {
                                    px(10.)
                                } else {
                                    px(8.)
                                }),
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

                    // HeroUI passes React Aria's raw MenuItem state to the
                    // indicator. The pinned 1.20.0 state has no indeterminate
                    // member, so the documented value is always falsy.
                    let is_indeterminate = false;
                    let indicator_content = if let Some(render) = &self.indicator_content {
                        Some(render(&key, is_selected, is_indeterminate))
                    } else if is_selected && self.selection_mode != SelectionMode::None {
                        Some(match self.indicator {
                            IndicatorKind::Checkmark => gpui::svg()
                                .size(px(13.))
                                .path(icons::CHECK)
                                // svg() never inherits text colour.
                                .text_color(sem_primary(cx))
                                .into_any_element(),
                            IndicatorKind::Dot => gpui::div()
                                .size(px(6.))
                                .rounded_full()
                                .bg(sem_primary(cx))
                                .into_any_element(),
                        })
                    } else {
                        None
                    };
                    let mut indicator = if self.indicator_content.is_some()
                        || self.selection_mode != SelectionMode::None
                    {
                        let mut cell = gpui::div()
                            .size(px(16.))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center();
                        // The row's normal gap is 12px. v3 positions the 16px
                        // cell 4px from its content, so cancel the extra 8px.
                        cell = if has_submenu {
                            cell.ml(px(-8.))
                        } else {
                            cell.mr(px(-8.))
                        };
                        if let Some(content) = indicator_content {
                            cell = cell.child(content);
                        }
                        Some(cell.into_any_element())
                    } else {
                        None
                    };
                    // The indicator owns v3's leading 16px cell. Submenu rows
                    // move it beside the trailing submenu chevron instead.
                    if !has_submenu {
                        if let Some(content) = indicator.take() {
                            row = row.child(content);
                        }
                    }
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
                    // v3, handed the row's state.
                    row = row.child(
                        gpui::div().flex_1().child(match &self.item_content {
                            Some(render) => {
                                // The slot's press is a frame behind the
                                // pointer, because gpui reports it to a handler
                                // rather than to the render that draws it. v3's
                                // `Dropdown.Item` render-props table lists no
                                // `isHovered`, so the hover the slot also
                                // tracks is not handed over; a row is focused
                                // when the keyboard cursor is on it.
                                let (_, is_pressed) = interaction
                                    .get(i)
                                    .map(|slot| *slot.read(cx))
                                    .unwrap_or_default();
                                render(
                                    &key,
                                    crate::util::InteractiveState {
                                        is_hovered: false,
                                        is_pressed,
                                        is_focused: cursor_at == Some(i),
                                        is_focus_visible: cursor_at == Some(i)
                                            && crate::util::focus_visible(cx),
                                        is_selected,
                                        is_disabled: is_item_disabled,
                                        is_indeterminate,
                                    },
                                )
                            }
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
                    // The slot's hover and press handlers keep the press the
                    // closure reads current. Attached even on a disabled row:
                    // the closure is handed `is_disabled` and may draw it.
                    if let Some(slot) = interaction.get(i) {
                        row = crate::util::track_interaction(row, slot);
                    }
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
                    if has_submenu {
                        row = row.child(
                            gpui::svg()
                                .size(px(13.))
                                .path(icons::CHEVRON_RIGHT)
                                .text_color(colors.muted),
                        );
                        if let Some(content) = indicator {
                            row = row.child(content);
                        }
                    }

                    if !is_item_disabled && !has_submenu {
                        let on_action = self.on_action.clone();
                        let on_selection_change = self.on_selection_change.clone();
                        let selection_own = selection_own.clone();
                        let dismiss = dismiss.clone();
                        // Attached even with no callback to run, because v3's
                        // close happens on the click, not in a handler.
                        let key2 = key.clone();
                        let mode = self.selection_mode;
                        let disallow_empty = self.disallow_empty_selection;
                        let current = self.selected_keys.clone();
                        row = row.on_click(move |_, window, cx| {
                            if crate::selection::reports_changes(mode) {
                                let next = crate::selection::next_selection(
                                    &current,
                                    &key2,
                                    mode,
                                    disallow_empty,
                                );
                                let blocked_last_removal =
                                    disallow_empty && current.len() == 1 && current.contains(&key2);
                                if !blocked_last_removal {
                                    if let Some(held) = &selection_own {
                                        held.update(cx, |value, cx| {
                                            *value = next.clone();
                                            cx.notify();
                                        });
                                    }
                                    if let Some(cb) = &on_selection_change {
                                        cb(&next, window, cx);
                                    }
                                }
                            }
                            if let Some(cb) = &on_action {
                                cb(&key2, window, cx);
                            }
                            // A pointer pick stays open only in multiple mode.
                            // The trigger gets focus back from a click; only a
                            // keyboard key-up cannot safely refocus it.
                            if mode != SelectionMode::Multiple {
                                if let Some(cb) = &dismiss {
                                    cb(true, window, cx);
                                }
                            }
                        });
                    }

                    // `Dropdown.SubmenuTrigger`: the child panel is anchored to
                    // the row and opens while the row is hovered. gpui paints in
                    // tree order, so it goes through `util::floating` like every
                    // other floating surface.
                    if has_submenu {
                        let open_key = SharedString::from(format!("{base}-sub-{key}"));
                        let is_sub_open = submenu_open.as_ref() == Some(&open_key);
                        let held = submenu_state.clone();
                        let hover_focus = submenu_focus.clone();
                        let open_key2 = open_key.clone();
                        // The hover that opens the child panel lives on the
                        // wrapper, not the row: `track_interaction` above has
                        // claimed the row's single `on_hover` when an
                        // `item_content` closure is set, and gpui refuses a
                        // second listener on one element.
                        let mut slot = gpui::div()
                            .id(gpui::ElementId::Name(
                                format!("{base}-sub-{key}-wrap").into(),
                            ))
                            .relative()
                            .child(row);
                        if !is_item_disabled {
                            let click_held = submenu_state.clone();
                            let click_focus = submenu_focus.clone();
                            let click_key = open_key.clone();
                            slot = slot
                                .on_hover(move |hovered, _window, cx| {
                                    if *hovered {
                                        held.update(cx, |value, cx| {
                                            if value.as_ref() != Some(&open_key2) {
                                                *value = Some(open_key2.clone());
                                                cx.notify();
                                            }
                                        });
                                        hover_focus.update(cx, |value, _| *value = false);
                                    }
                                })
                                .on_click(move |_, _, cx| {
                                    click_held.update(cx, |value, cx| {
                                        *value = Some(click_key.clone());
                                        cx.notify();
                                    });
                                    click_focus.update(cx, |value, _| *value = false);
                                });
                        }
                        if is_sub_open {
                            open_submenu = Some((i, open_key, submenu));
                        }
                        panel = panel.child(slot);
                    } else {
                        panel = panel.child(row);
                    }
                }
            }
        }

        // The flex surface's rectangular hull includes blank space between
        // unequal-height panels. The outside listener lives on the root panel
        // and checks the union of the actual parent and descendant bounds.
        if self.deferred {
            if let Some(cb) = dismiss.clone() {
                if open_submenu.is_some() {
                    // A submenu is a sibling of the parent panel. Keep the
                    // union-boundary check so its real bounds remain inside
                    // the menu surface; the shared token still gates Escape.
                    let panel_union = all_panel_bounds.clone();
                    if let Some(token) = overlay_token.clone() {
                        panel = crate::util::dismiss_on_press_outside_with_token_event(
                            panel,
                            token,
                            move |event, window, cx| {
                                let inside = panel_union.borrow().iter().any(|bounds| {
                                    event.position.x >= bounds.origin.x
                                        && event.position.x <= bounds.origin.x + bounds.size.width
                                        && event.position.y >= bounds.origin.y
                                        && event.position.y <= bounds.origin.y + bounds.size.height
                                });
                                if inside {
                                    crate::util::DismissResult::Declined
                                } else {
                                    cb(true, window, cx);
                                    crate::util::DismissResult::Handled
                                }
                            },
                        );
                    }
                } else if let Some(token) = overlay_token.clone() {
                    // The explicit token gates a simple menu against any
                    // overlay above it.
                    panel = crate::util::dismiss_on_press_outside_with_token(
                        panel,
                        token,
                        move |window, cx| {
                            cb(true, window, cx);
                            crate::util::DismissResult::Handled
                        },
                    );
                }
            }
        }

        // Parent and child menus share one deferred surface. The submenu is a
        // sibling of the parent's overflow scroller, so a low trigger cannot
        // clip a tall child.
        let mut surface = gpui::div()
            .relative()
            .flex()
            .items_start()
            .gap(px(4.))
            .child(panel);
        if let Some((index, submenu_id, submenu)) = open_submenu {
            let item_bounds_now = item_bounds[index]
                .as_ref()
                .and_then(|bounds| bounds.read(cx).to_owned());
            let panel_bounds_now = panel_bounds.read(cx).to_owned();
            let top = match (item_bounds_now, panel_bounds_now) {
                (Some(item), Some(parent)) if item.origin.y > parent.origin.y => {
                    item.origin.y - parent.origin.y
                }
                _ => px(0.),
            };
            let mut sub = Menu::new(submenu_id.clone(), submenu)
                .id(gpui::ElementId::Name(format!("{submenu_id}-menu").into()))
                .indicator(self.indicator)
                .disabled_keys(self.disabled_keys)
                .embedded(all_panel_bounds)
                .focus_first(submenu_focus.clone());
            sub.item_content = self.item_content.clone();
            sub.indicator_content = self.indicator_content.clone();
            if let Some(token) = overlay_token.clone() {
                sub = sub.overlay_token(token);
            }
            let close_state = submenu_state;
            let close_focus_state = submenu_focus;
            let parent_focus = focus_handle.clone();
            sub = sub.on_back(move |window, cx| {
                close_state.update(cx, |value, cx| {
                    *value = None;
                    cx.notify();
                });
                close_focus_state.update(cx, |value, _| *value = false);
                window.focus(&parent_focus);
            });
            if let Some(cb) = self.on_action.clone() {
                sub = sub.on_action(move |key, window, cx| cb(key, window, cx));
            }
            if let Some(cb) = dismiss.clone() {
                sub = sub.on_dismiss(move |refocus, window, cx| cb(refocus, window, cx));
            }
            surface = surface.child(gpui::div().pt(top).child(sub));
        }

        // Escape bubbles from the focused descendant to this root surface.
        let surface = if self.deferred {
            match dismiss {
                Some(cb) => match overlay_token {
                    Some(token) => crate::util::dismiss_on_escape_with_token(
                        surface,
                        token,
                        move |window, cx| {
                            cb(true, window, cx);
                            crate::util::DismissResult::Handled
                        },
                    ),
                    None => surface,
                },
                None => surface,
            }
        } else {
            surface
        };

        let zoom = crate::anim::ZoomBox::panel(px(6.), crate::util::container_radius(cx));
        let panel = if self.exiting {
            crate::anim::exiting(
                surface,
                gpui::ElementId::Name(format!("{base}-panel-out").into()),
                zoom,
                crate::anim::Motion::LIST_OUT,
                cx,
            )
        } else {
            crate::anim::entering_zoom(
                surface,
                gpui::ElementId::Name(format!("{base}-panel").into()),
                zoom,
                crate::anim::Motion::POPOVER_IN,
                cx,
            )
        };
        if self.deferred {
            crate::util::floating(panel).into_any_element()
        } else {
            panel.into_any_element()
        }
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
    item_content: Option<ItemContent>,
    indicator_content: Option<ItemIndicatorContent>,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    default_selected_keys: Vec<SharedString>,
    selection_is_controlled: bool,
    disallow_empty_selection: bool,
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

    pub fn uncontrolled(
        id: impl Into<gpui::ElementId>,
        trigger: impl IntoElement,
        items: Vec<MenuItem>,
    ) -> Self {
        let mut dd = Self::new(id, trigger, items, false);
        dd.is_open = None;
        dd
    }

    pub fn new(
        id: impl Into<gpui::ElementId>,
        trigger: impl IntoElement,
        items: Vec<MenuItem>,
        is_open: bool,
    ) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            trigger_kind: DropdownTrigger::default(),
            is_open: Some(is_open),
            default_open: false,
            on_open_change: None,
            items,
            item_content: None,
            indicator_content: None,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            default_selected_keys: Vec::new(),
            selection_is_controlled: false,
            disallow_empty_selection: false,
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

    /// `children` on `Dropdown.Item` — replaces each item's label with a
    /// render closure receiving its key and interactive state.
    pub fn item_content(
        mut self,
        render: impl Fn(&SharedString, crate::util::InteractiveState) -> AnyElement + 'static,
    ) -> Self {
        self.item_content = Some(std::sync::Arc::new(render));
        self
    }

    /// `children` on `Dropdown.ItemIndicator` — replaces the built-in mark.
    pub fn indicator_content(
        mut self,
        render: impl Fn(&SharedString, bool, bool) -> AnyElement + 'static,
    ) -> Self {
        self.indicator_content = Some(std::sync::Arc::new(render));
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
        self.selection_is_controlled = true;
        self
    }

    /// `defaultSelectedKeys` on `Dropdown.Menu` — seeds uncontrolled selection.
    pub fn default_selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_selected_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `disallowEmptySelection` on `Dropdown.Menu`.
    pub fn disallow_empty_selection(mut self, value: bool) -> Self {
        self.disallow_empty_selection = value;
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
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        let (selected_keys, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{wrap_base}-selected").into()),
            self.selection_is_controlled
                .then(|| self.selected_keys.clone()),
            self.default_selected_keys.clone(),
        );
        self.selected_keys = selected_keys;
        // `overlay_scope` takes `cx` mutably too, so it goes here.
        let (phase, overlay_token) = crate::util::overlay_scope(
            window,
            cx,
            gpui::ElementId::Name(format!("{wrap_base}-phase").into()),
            is_open,
            true,
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
            let mut menu = Menu::new(
                gpui::ElementId::Name(format!("{wrap_base}-menu-content").into()),
                self.items,
            )
            .id(gpui::ElementId::Name(format!("{wrap_base}-menu").into()))
            .dropdown_composition()
            .exiting(phase == crate::util::OverlayPhase::Exiting)
            .selection_mode(self.selection_mode)
            .selected_keys(self.selected_keys.clone())
            .disallow_empty_selection(self.disallow_empty_selection)
            .disabled_keys(self.disabled_keys.clone())
            .indicator(self.indicator);
            menu.item_content = self.item_content.clone();
            menu.indicator_content = self.indicator_content.clone();
            menu = menu.overlay_token(overlay_token);
            if let Some(on_action) = self.on_action.clone() {
                menu = menu.on_action(move |k, w, cx| on_action(k, w, cx));
            }
            let selection_cb = self.on_selection_change.clone();
            if selection_cb.is_some() || selection_own.is_some() {
                menu = menu.on_selection_change(move |keys, w, cx| {
                    if let Some(held) = &selection_own {
                        held.update(cx, |value, cx| {
                            *value = keys.to_vec();
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &selection_cb {
                        cb(keys, w, cx);
                    }
                });
            }
            let dismiss_cb = self.on_open_change.clone();
            if dismiss_cb.is_some() || dismiss_own.is_some() {
                let back_to_trigger = trigger_handle;
                menu = menu.on_dismiss(move |refocus, window, cx| {
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
                    // An Enter pick runs this inside the key event, where gpui
                    // would activate the trigger on key up and reopen the menu
                    // -- the keyboard path asks for no refocus for that reason.
                    if refocus {
                        window.focus(&back_to_trigger);
                    }
                });
            }
            let anchor = crate::util::placed_panel(self.placement, px(6.));
            root = root.child(anchor.child(menu));
        }

        root
    }
}
