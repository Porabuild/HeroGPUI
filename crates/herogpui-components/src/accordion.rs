//! Accordion — port of `@heroui/accordion`.

use std::collections::HashSet;

use gpui::{
    prelude::*, px, AnyElement, App, IntoElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::icons;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccordionItemState {
    pub is_expanded: bool,
    pub is_disabled: bool,
}

pub type AccordionIndicatorContent =
    std::sync::Arc<dyn Fn(AccordionItemState, &mut Window, &mut App) -> AnyElement + 'static>;
pub type AccordionItemExpandedChange =
    std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

/// One accordion entry.
pub struct AccordionItem {
    pub key: SharedString,
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub content: AnyElement,
    pub is_disabled: bool,
    pub default_expanded: bool,
    pub indicator: Option<AccordionIndicatorContent>,
    pub on_expanded_change: Option<AccordionItemExpandedChange>,
}

impl AccordionItem {
    pub fn new(key: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            subtitle: None,
            content: gpui::div().into_any_element(),
            is_disabled: false,
            default_expanded: false,
            indicator: None,
            on_expanded_change: None,
        }
    }

    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = el.into_any_element();
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn default_expanded(mut self, v: bool) -> Self {
        self.default_expanded = v;
        self
    }

    pub fn indicator(
        mut self,
        render: impl Fn(AccordionItemState, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.indicator = Some(std::sync::Arc::new(render));
        self
    }

    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(std::sync::Arc::new(handler));
        self
    }
}

/// Card style (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionVariant {
    /// Flush with the page, separated by rules.
    #[default]
    Default,
    /// Each item sits on its own surface.
    Surface,
}

impl AccordionVariant {
    pub const ALL: [AccordionVariant; 2] = [AccordionVariant::Default, AccordionVariant::Surface];

    pub fn label(self) -> &'static str {
        match self {
            AccordionVariant::Default => "Default",
            AccordionVariant::Surface => "Surface",
        }
    }
}

type OnToggle = std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

/// HeroUI Accordion (controlled).
#[derive(IntoElement)]
pub struct Accordion {
    items: Vec<AccordionItem>,
    /// Scopes this instance's keyed state against its neighbours'.
    id: Option<gpui::ElementId>,
    /// `expandedKeys` — `None` leaves the accordion holding the set, seeded
    /// from `defaultExpandedKeys`.
    expanded_keys: Option<HashSet<SharedString>>,
    default_expanded_keys: HashSet<SharedString>,
    is_disabled: bool,
    disabled_keys: HashSet<SharedString>,
    allows_multiple_expanded: bool,
    on_expanded_change:
        Option<std::sync::Arc<dyn Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static>>,
    variant: AccordionVariant,
    hide_separator: bool,
    on_toggle: Option<OnToggle>,
}

impl Accordion {
    /// `allowsMultipleExpanded` — when false, expanding one item collapses the
    /// rest. The caller still owns `expanded_keys`; this only changes what the
    /// toggle callback reports.
    pub fn allows_multiple_expanded(mut self, v: bool) -> Self {
        self.allows_multiple_expanded = v;
        self
    }

    /// `onExpandedChange` — reports the whole expanded set after a toggle,
    /// where [`Accordion::on_toggle`] reports only the key that moved.
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(std::sync::Arc::new(handler));
        self
    }

    pub fn new(items: Vec<AccordionItem>) -> Self {
        Self {
            items,
            id: None,
            expanded_keys: None,
            default_expanded_keys: HashSet::new(),
            is_disabled: false,
            disabled_keys: HashSet::new(),
            // v3's API table documents `allowsMultipleExpanded` with default
            // `false`: expanding one item collapses the others.
            allows_multiple_expanded: false,
            on_expanded_change: None,
            variant: AccordionVariant::Default,
            hide_separator: false,
            on_toggle: None,
        }
    }

    /// Controlled set of open item keys.
    /// `isDisabled` — no item can be toggled.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `disabledKeys` — the listed items cannot be toggled.
    pub fn disabled_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.disabled_keys = keys.into_iter().collect();
        self
    }

    /// Distinguishes this accordion from its neighbours.
    ///
    /// Scopes every keyed slot — the expanded set, the per-trigger focus
    /// handles and the trigger element ids — because two instances of the
    /// same `RenderOnce` type share gpui's keyed-state namespace, so keys
    /// derived from the item keys alone collide whenever two accordions hold
    /// the same items. The default is derived from the item keys, which is
    /// unique unless two id-less accordions hold the same items.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn expanded_keys(mut self, keys: HashSet<SharedString>) -> Self {
        self.expanded_keys = Some(keys);
        self
    }

    /// `defaultExpandedKeys` — the uncontrolled initial set.
    ///
    /// Only consulted when `expandedKeys` is not supplied; the accordion then
    /// owns the set and expands itself on press.
    pub fn default_expanded_keys(mut self, keys: HashSet<SharedString>) -> Self {
        self.default_expanded_keys = keys;
        self
    }

    /// `defaultExpanded` on a single item — shorthand for a one-key default
    /// set.
    pub fn default_expanded(mut self, key: impl Into<SharedString>) -> Self {
        self.default_expanded_keys.insert(key.into());
        self
    }

    pub fn variant(mut self, v: AccordionVariant) -> Self {
        self.variant = v;
        self
    }

    /// `hideSeparator` — drops the rules between items.
    pub fn hide_separator(mut self, v: bool) -> Self {
        self.hide_separator = v;
        self
    }

    /// Fires with the toggled key; the parent flips membership in its set.
    pub fn on_toggle(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Accordion {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The instance's id scopes every keyed slot: gpui namespaces keyed
        // state by the `RenderOnce` type name, not by the instance, so keys
        // built from the item keys alone (`acc-one`, `acc-one-focus`) map to
        // the same slot in every accordion holding those items and the second
        // instance answers no clicks. The fallback is derived from the item
        // keys, which is unique unless two id-less accordions hold the same
        // items.
        let id = self.id.clone().unwrap_or_else(|| {
            let keys: Vec<&str> = self.items.iter().map(|i| i.key.as_ref()).collect();
            gpui::ElementId::Name(format!("acc-{}-expanded", keys.join("-")).into())
        });
        // One tab stop per trigger. `use_keyed_state` takes `cx` mutably, so the
        // handles come before anything borrows the theme.
        let trigger_focus: Vec<gpui::FocusHandle> = self
            .items
            .iter()
            .map(|item| {
                crate::util::tab_stop_handle(
                    gpui::ElementId::Name(format!("acc-{:?}-{}-focus", id, item.key).into()),
                    window,
                    cx,
                )
            })
            .collect();
        // `expandedKeys` wins; without it the accordion holds the set, seeded
        // from `defaultExpandedKeys`. `controlled` takes `cx` mutably, so it
        // precedes the theme tokens.
        let mut default_expanded_keys = self.default_expanded_keys.clone();
        default_expanded_keys.extend(
            self.items
                .iter()
                .filter(|item| item.default_expanded)
                .map(|item| item.key.clone()),
        );
        let (expanded_keys, expanded_own) = crate::util::controlled(
            window,
            cx,
            id.clone(),
            self.expanded_keys.clone(),
            default_expanded_keys,
        );

        let colors = cx.colors().clone();
        let layout = cx.layout().clone();

        // `.accordion` is `w-full` and nothing else: the default variant is
        // flush with the page. Only `.accordion--surface` paints a background
        // and rounds the group. This used to give both variants a rounded white
        // card, so `variant` made no visible difference.
        let mut container = match self.variant {
            AccordionVariant::Default => gpui::div(),
            AccordionVariant::Surface => gpui::div()
                .bg(colors.surface.background)
                .rounded(crate::util::container_radius(cx))
                .overflow_hidden(),
        };

        container = container.w_full().flex().flex_col();

        let count = self.items.len();
        let ring_visible = crate::util::focus_visible(cx);
        for (i, item) in self.items.into_iter().enumerate() {
            let is_open = expanded_keys.contains(&item.key);
            let item_disabled =
                self.is_disabled || item.is_disabled || self.disabled_keys.contains(&item.key);
            let item_state = AccordionItemState {
                is_expanded: is_open,
                is_disabled: item_disabled,
            };

            // `.accordion__trigger:focus-visible` is `status-focused`.
            let header_focus = trigger_focus.get(i);
            let mut header = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("acc-{:?}-{}", id, item.key).into(),
                ))
                .when_some(header_focus.filter(|_| !item_disabled), |h, handle| {
                    h.track_focus(handle)
                })
                .flex()
                .items_center()
                .justify_between()
                // `.accordion__trigger` is `px-4 py-4`.
                .px(px(16.))
                .py(px(16.));

            if item_disabled {
                header = header.opacity(layout.disabled_opacity);
            } else if !is_open {
                // v3 hovers the row surface rather than dimming its text. The
                // default trigger washes the page foreground over the row —
                // `color-mix(in oklab, var(--foreground) 3%, transparent 90%)`,
                // whose weights normalise to 3/93 — while `.accordion--surface`
                // overrides that with the full `bg-default`.
                let hover_bg = match self.variant {
                    AccordionVariant::Default => {
                        herogpui_core::soft_mix(colors.foreground, 3.0 / 93.0)
                    }
                    AccordionVariant::Surface => colors.default.color,
                };
                header = header.cursor_pointer().hover(move |s| s.bg(hover_bg));
            } else {
                header = header.cursor_pointer();
            }

            let mut title_col = gpui::div().flex().flex_col();
            title_col = title_col.child(
                gpui::div()
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(colors.foreground)
                    .child(item.title.to_string()),
            );
            if let Some(sub) = &item.subtitle {
                title_col = title_col.child(
                    gpui::div()
                        .text_size(px(12.))
                        .line_height(px(16.))
                        .text_color(colors.muted)
                        .child(sub.to_string()),
                );
            }
            header = header.child(title_col);

            let indicator = if let Some(render) = &item.indicator {
                gpui::div()
                    .size(px(16.))
                    .flex_shrink_0()
                    .text_color(colors.muted)
                    .child(render(item_state, window, cx))
                    .into_any_element()
            } else {
                gpui::svg()
                    // `.accordion__indicator` is `size-4`.
                    .size(px(16.))
                    .path(if is_open {
                        icons::CHEVRON_UP
                    } else {
                        icons::CHEVRON_DOWN
                    })
                    .text_color(colors.muted)
                    .flex_shrink_0()
                    .into_any_element()
            };
            header = header.child(indicator);

            if !item_disabled {
                let key = item.key.clone();
                let on_toggle = self.on_toggle.clone();
                let on_expanded = self.on_expanded_change.clone();
                let on_item_expanded = item.on_expanded_change.clone();
                let current = expanded_keys.clone();
                let multiple = self.allows_multiple_expanded;
                let own = expanded_own.clone();
                if on_toggle.is_some()
                    || on_expanded.is_some()
                    || on_item_expanded.is_some()
                    || own.is_some()
                {
                    header = header.on_click(move |_, window, cx| {
                        let next = next_expanded(&current, &key, multiple);
                        // Uncontrolled: update our own set, or the header would
                        // be inert.
                        if let Some(held) = &own {
                            let next = next.clone();
                            held.update(cx, |v, cx| {
                                *v = next;
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &on_toggle {
                            cb(&key, window, cx);
                        }
                        if let Some(cb) = &on_expanded {
                            cb(&next, window, cx);
                        }
                        if let Some(cb) = &on_item_expanded {
                            cb(!is_open, window, cx);
                        }
                    });
                }
            }

            let header = crate::util::with_focus_ring(
                header,
                !item_disabled
                    && ring_visible
                    && header_focus.is_some_and(|h| h.is_focused(window)),
                true,
                Vec::new(),
                cx,
            );
            // `.accordion__heading` wraps the trigger and `.accordion__panel`
            // the body; both are plain flex boxes with the metrics on the parts
            // inside them.
            let mut section = gpui::div().flex().flex_col().child(header);
            if is_open {
                section = section.child(
                    gpui::div()
                        .px(px(16.))
                        .pb(px(16.))
                        .pt(px(0.))
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .text_color(colors.muted)
                        .child(item.content),
                );
            }

            // `.accordion__item::after` is a 1px `bg-separator` line at the
            // bottom of every item but the last, inset to 3%/94% on a surface.
            let inset = self.variant == AccordionVariant::Surface;
            section = section.when(!self.hide_separator && i + 1 < count, |s| {
                s.child(
                    gpui::div()
                        .h(px(1.))
                        .rounded(crate::util::hairline_radius(cx))
                        .bg(colors.separator)
                        .map(|line| {
                            if inset {
                                line.mx(gpui::relative(0.03)).w(gpui::relative(0.94))
                            } else {
                                line.w_full()
                            }
                        }),
                )
            });

            container = container.child(section);
        }

        container
    }
}

/// The expanded set after toggling `key`.
///
/// With `allowsMultipleExpanded` off, expanding an item collapses every other.
pub fn next_expanded(
    current: &HashSet<SharedString>,
    key: &SharedString,
    allows_multiple: bool,
) -> HashSet<SharedString> {
    let was_open = current.contains(key);
    if was_open {
        let mut next = current.clone();
        next.remove(key);
        return next;
    }
    if allows_multiple {
        let mut next = current.clone();
        next.insert(key.clone());
        next
    } else {
        HashSet::from([key.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(keys: &[&str]) -> HashSet<SharedString> {
        keys.iter()
            .map(|k| SharedString::from(k.to_string()))
            .collect()
    }

    #[test]
    fn multiple_adds_without_collapsing() {
        let next = next_expanded(&set(&["a"]), &SharedString::from("b"), true);
        assert_eq!(next, set(&["a", "b"]));
    }

    #[test]
    fn single_collapses_the_others() {
        let next = next_expanded(&set(&["a"]), &SharedString::from("b"), false);
        assert_eq!(next, set(&["b"]));
    }

    #[test]
    fn toggling_an_open_item_closes_it_in_both_modes() {
        assert_eq!(
            next_expanded(&set(&["a", "b"]), &SharedString::from("a"), true),
            set(&["b"])
        );
        assert!(next_expanded(&set(&["a"]), &SharedString::from("a"), false).is_empty());
    }

    // The pinned default trigger hover is
    // `color-mix(in oklab, var(--foreground) 3%, transparent 90%)` -- whose
    // weights normalise to 3/93 -- and `.accordion--surface` overrides it
    // with the full `bg-default`. `soft()` is a lighter, wrong token, so the
    // check is mechanical.
    #[test]
    fn the_trigger_hover_uses_the_exact_pinned_tokens() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("accordion.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("herogpui_core::soft_mix(colors.foreground, 3.0 / 93.0)"),
            "the default trigger must hover the 3/93 foreground wash \
             (pinned `color-mix(in oklab, var(--foreground) 3%, transparent 90%)`)"
        );
        assert!(
            source.contains("AccordionVariant::Surface => colors.default.color"),
            "the surface trigger must hover the full `bg-default` \
             (pinned `.accordion--surface .accordion__trigger:hover`)"
        );
        assert!(
            !source.contains("colors.default.soft()"),
            "no accordion trigger may hover a soft token"
        );
    }
}
