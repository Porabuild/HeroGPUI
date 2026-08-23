//! Tabs — port of `@heroui/tabs`.

use gpui::{prelude::*, px, AnyElement, App, IntoElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;


/// Tab bar style (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// Filled indicator behind the selected tab.
    #[default]
    Primary,
    /// Underline indicator beneath the selected tab.
    Secondary,
}

impl TabsVariant {
    pub const ALL: [TabsVariant; 2] = [TabsVariant::Primary, TabsVariant::Secondary];

    pub fn label(self) -> &'static str {
        match self {
            TabsVariant::Primary => "Primary",
            TabsVariant::Secondary => "Secondary",
        }
    }
}

/// One tab: key + label + panel content.
pub struct TabItem {
    pub key: SharedString,
    pub label: SharedString,
    pub content: Option<AnyElement>,
}

impl TabItem {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            content: None,
        }
    }

    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }
}

type OnChange = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// HeroUI Tabs (controlled).
#[derive(IntoElement)]
pub struct Tabs {
    id: gpui::ElementId,
    items: Vec<TabItem>,
    /// `selectedKey` — `None` leaves the tabs holding the selection, seeded
    /// from `defaultSelectedKey`.
    selected_key: Option<SharedString>,
    default_selected_key: Option<SharedString>,
    variant: TabsVariant,
    is_disabled: bool,
    hide_separator: bool,
    orientation: Orientation,
    on_change: Option<OnChange>,
}

impl Tabs {
    /// `orientation` — a vertical tab list stacks its tabs.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `selectedKey` — also accepted positionally by [`Tabs::new`].
    pub fn selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.selected_key = Some(key.into());
        self
    }

    /// `defaultSelectedKey` — the uncontrolled initial tab.
    ///
    /// Only consulted when `selectedKey` is not supplied; the tabs then own the
    /// selection and switch themselves on press.
    pub fn default_selected_key(mut self, key: impl Into<SharedString>) -> Self {
        self.default_selected_key = Some(key.into());
        self
    }

    /// `onSelectionChange` — the v3 name for [`Tabs::on_change`].
    pub fn on_selection_change(
        self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change(handler)
    }

    pub fn new(
        id: impl Into<gpui::ElementId>,
        items: Vec<TabItem>,
        selected_key: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            items,
            selected_key: Some(selected_key.into()),
            default_selected_key: None,
            variant: TabsVariant::Primary,
            is_disabled: false,
            hide_separator: false,
            orientation: Orientation::Horizontal,
            on_change: None,
        }
    }

    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }



    /// `hideSeparator` — drops the rail under a `secondary` tab list.
    pub fn hide_separator(mut self, v: bool) -> Self {
        self.hide_separator = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let base_id = format!("{:?}", self.id);

        // `selectedKey` wins; without it the tabs hold the selection, seeded
        // from `defaultSelectedKey` (falling back to the first tab so something
        // is always active). `controlled` takes `cx` mutably, so it precedes
        // the theme tokens.
        let fallback = self
            .default_selected_key
            .clone()
            .or_else(|| self.items.first().map(|i| i.key.clone()))
            .unwrap_or_default();
        let (selected_key, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{base_id}-selected").into()),
            self.selected_key.clone(),
            fallback,
        );

        let colors = cx.colors();
        let layout = cx.layout();

        let vertical = self.orientation == Orientation::Vertical;
        let mut list = gpui::div().flex();
        if vertical {
            list = list.flex_col().items_start();
        }

        // v3 keeps two indicator styles: `primary` fills a segment behind the
        // selected tab, `secondary` underlines it.
        match self.variant {
            TabsVariant::Primary => {
                list = list
                    .gap(px(4.))
                    .p(px(4.))
                    .rounded(crate::util::control_radius(cx))
                    .bg(colors.surface_secondary);
                for item in &self.items {
                    let active = item.key == selected_key;
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .px(px(14.))
                        .py(px(6.))
                        .rounded(crate::util::control_radius(cx))
                        .text_size(px(14.))
                        .when(!self.is_disabled, |t| t.cursor_pointer());
                    if active {
                        tab = tab
                            .bg(colors.segment.background)
                            .text_color(colors.segment.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .when(!layout.surface_shadow.is_empty(), |t| {
                                t.shadow(layout.surface_shadow.clone())
                            });
                    } else {
                        tab = tab.text_color(colors.muted);
                        if !self.is_disabled {
                            tab = tab.hover(move |s| s.text_color(colors.foreground));
                        }
                    }
                    if !self.is_disabled
                        && (self.on_change.is_some() || selection_own.is_some())
                    {
                        let key = item.key.clone();
                        let cb = self.on_change.clone();
                        let own = selection_own.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            // Uncontrolled: move our own selection, or pressing
                            // a tab would do nothing.
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = key.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &cb {
                                f(&key, window, cx);
                            }
                        });
                    }
                    list = list.child(tab.child(item.label.to_string()));
                }
            }
            TabsVariant::Secondary => {
                list = list.gap(px(16.));
                if !self.hide_separator {
                    list = list.border_b_1().border_color(colors.separator);
                }
                for item in &self.items {
                    let active = item.key == selected_key;
                    let mut tab = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{base_id}-tab-{}", item.key).into(),
                        ))
                        .px(px(2.))
                        .pb(px(6.))
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .border_b_2()
                        .when(!self.is_disabled, |t| t.cursor_pointer());
                    tab = if active {
                        tab.border_color(colors.accent.color)
                            .text_color(colors.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM)
                    } else {
                        tab.border_color(gpui::transparent_black())
                            .text_color(colors.muted)
                    };
                    if !self.is_disabled
                        && (self.on_change.is_some() || selection_own.is_some())
                    {
                        let key = item.key.clone();
                        let cb = self.on_change.clone();
                        let own = selection_own.clone();
                        tab = tab.on_click(move |_, window, cx| {
                            // Uncontrolled: move our own selection, or pressing
                            // a tab would do nothing.
                            if let Some(held) = &own {
                                held.update(cx, |v, cx| {
                                    *v = key.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(f) = &cb {
                                f(&key, window, cx);
                            }
                        });
                    }
                    list = list.child(tab.child(item.label.to_string()));
                }
            }
        }

        // Active panel
        let mut items = self.items;
        let active_idx = items.iter().position(|i| i.key == selected_key);
        let mut el = gpui::div().flex().flex_col().gap(px(16.)).child(list);

        if let Some(idx) = active_idx {
            if let Some(content) = items.swap_remove(idx).content {
                el = el.child(gpui::div().pt(px(10.)).child(content));
            }
        }

        el
    }
}



