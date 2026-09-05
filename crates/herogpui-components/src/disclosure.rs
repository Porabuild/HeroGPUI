//! Disclosure & DisclosureGroup — HeroUI v3 `Disclosure`.
//!
//! A single collapsible section: a trigger and a `p-2` body, which is all v3's
//! `disclosure.css` gives it -- not an `Accordion` with one item, which is what
//! this used to render. `DisclosureGroup` keeps the accordion, whose group
//! behaviour (single or multiple expansion) it needs.

use std::collections::HashSet;

use gpui::{
    px, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

/// The values HeroUI passes to a Disclosure children render function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureRenderState {
    pub is_expanded: bool,
    pub is_disabled: bool,
}

type DisclosureContent = std::sync::Arc<dyn Fn(DisclosureRenderState) -> AnyElement + 'static>;

/// Single Disclosure — like an accordion with one item.
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    title: SharedString,
    is_expanded: Option<bool>,
    default_expanded: bool,
    is_disabled: bool,
    children: Vec<AnyElement>,
    content: Option<DisclosureContent>,
    on_toggle: Option<std::sync::Arc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
}

impl Disclosure {
    /// `onExpandedChange` — reports the expansion the press moves to.
    pub fn on_expanded_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(std::sync::Arc::new(handler));
        self
    }

    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            is_expanded: None,
            default_expanded: false,
            is_disabled: false,
            children: Vec::new(),
            content: None,
            on_toggle: None,
        }
    }

    pub fn is_expanded(mut self, v: bool) -> Self {
        self.is_expanded = Some(v);
        self
    }

    /// `defaultExpanded` — seeds the disclosure's own expansion state.
    pub fn default_expanded(mut self, v: bool) -> Self {
        self.default_expanded = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `children` as a render function — replaces static body children and is
    /// handed the current expanded and disabled state on every render.
    pub fn content(
        mut self,
        render: impl Fn(DisclosureRenderState) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }
}

impl ParentElement for Disclosure {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Disclosure {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A v3 Disclosure is not a small Accordion: `.disclosure` is `relative`,
        // its trigger is whatever the caller passes (`<Button slot="trigger">`
        // in v3's own examples) with a `.disclosure__indicator` chevron, and
        // `.disclosure__body` is `p-2`. Rendering it as a one-item accordion
        // gave it a card, a 16px trigger row and a separator that v3's sheet
        // has no rule for.
        let (expanded, expanded_own) = crate::util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-expanded", self.id).into()),
            self.is_expanded,
            self.default_expanded,
        );
        let cb = self.on_toggle.clone();
        let children = match self.content {
            Some(render) => vec![render(DisclosureRenderState {
                is_expanded: expanded,
                is_disabled: self.is_disabled,
            })],
            None => self.children,
        };
        // `.disclosure__trigger` is `inline-block` with the focus ring on it;
        // v3 passes a Button, which is what this builds.
        let trigger = crate::button::Button::new(ElementId::Name(
            format!("{:?}-trigger", self.id).into(),
        ))
        .variant(if expanded {
            herogpui_core::Variant::Secondary
        } else {
            herogpui_core::Variant::Tertiary
        })
        .label(self.title.clone())
        .is_disabled(self.is_disabled)
        // `.disclosure__indicator` is `ms-auto size-4` and turns 180 degrees
        // when the panel is open, which is a glyph swap here. It trails the
        // label as the button's ordered child, the way v3 composes it.
        .child(
            gpui::svg()
                .size(px(16.))
                .path(if expanded {
                    crate::icons::CHEVRON_UP
                } else {
                    crate::icons::CHEVRON_DOWN
                })
                .text_color(cx.colors().muted)
                .into_any_element(),
        )
        .on_press(move |_, w, cx| {
            if let Some(held) = &expanded_own {
                held.update(cx, |expanded, cx| {
                    *expanded = !*expanded;
                    cx.notify();
                });
            }
            if let Some(f) = &cb {
                f(!expanded, w, cx);
            }
        });

        let mut el = gpui::div()
            .id(self.id)
            .relative()
            .flex()
            .flex_col()
            .child(trigger);
        // v3 transitions measured height and opacity. gpui cannot animate an
        // unmeasured content height, so this preserves the 200ms entry fade;
        // collapsed content leaves the tree immediately.
        if expanded {
            el = el.child(
                crate::anim::entering(
                    gpui::div()
                        // `.disclosure__body` is `p-2`.
                        .p(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .children(children),
                    "disclosure-body",
                    crate::anim::Motion::DISCLOSURE,
                    cx,
                )
                .into_any_element(),
            );
        }
        let _ = window;
        el.into_any_element()
    }
}

/// Group of Disclosures — mirrors Accordion but with `Disclosure` naming.
#[derive(IntoElement)]
pub struct DisclosureGroup {
    id: ElementId,
    items: Vec<(SharedString, SharedString, Vec<AnyElement>)>,
    expanded: Option<HashSet<SharedString>>,
    default_expanded: HashSet<SharedString>,
    allows_multiple_expanded: bool,
    is_disabled: bool,
    on_expanded_change:
        Option<std::sync::Arc<dyn Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static>>,
}

impl DisclosureGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            expanded: None,
            default_expanded: HashSet::new(),
            allows_multiple_expanded: false,
            is_disabled: false,
            on_expanded_change: None,
        }
    }

    pub fn item(
        mut self,
        key: impl Into<SharedString>,
        title: impl Into<SharedString>,
        content: impl IntoElement,
    ) -> Self {
        self.items
            .push((key.into(), title.into(), vec![content.into_any_element()]));
        self
    }

    /// `expandedKeys` — the caller-owned expanded set.
    pub fn expanded_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.expanded = Some(keys.into_iter().map(Into::into).collect());
        self
    }

    /// `defaultExpandedKeys` — seeds the group's own expanded set.
    pub fn default_expanded_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.default_expanded = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `allowsMultipleExpanded` — otherwise opening one item closes the rest.
    pub fn allows_multiple_expanded(mut self, v: bool) -> Self {
        self.allows_multiple_expanded = v;
        self
    }

    /// `isDisabled` — disables every disclosure in the group.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `onExpandedChange` — reports the complete next expanded set.
    pub fn on_expanded_change(
        mut self,
        f: impl Fn(&HashSet<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for DisclosureGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (mut expanded, expanded_own) = crate::util::controlled(
            window,
            cx,
            ElementId::Name(format!("{:?}-expanded", self.id).into()),
            self.expanded,
            self.default_expanded,
        );
        if !self.allows_multiple_expanded && expanded_own.is_some() && expanded.len() > 1 {
            let retained = self
                .items
                .iter()
                .find_map(|(key, _, _)| expanded.contains(key).then(|| key.clone()));
            let next = retained.into_iter().collect::<HashSet<_>>();
            if let Some(held) = &expanded_own {
                let held_next = next.clone();
                held.update(cx, |expanded, cx| {
                    *expanded = held_next;
                    cx.notify();
                });
            }
            if let Some(callback) = self.on_expanded_change.clone() {
                let reported = next.clone();
                window.defer(cx, move |window, cx| callback(&reported, window, cx));
            }
            expanded = next;
        }
        // `.disclosure-group` is `w-full` and nothing else: v3's group *is* a
        // column of `Disclosure`s, one of which may be open, so this renders
        // them rather than an `Accordion` -- which would give every row the
        // card, the padded trigger and the separator the sheet has no rule for.
        let mut el = gpui::div().id(self.id.clone()).w_full().flex().flex_col();
        for (key, title, children) in self.items {
            let is_expanded = expanded.contains(&key);
            let mut disclosure = Disclosure::new(key.clone(), title)
                .is_expanded(is_expanded)
                .is_disabled(self.is_disabled)
                .children(children);
            if expanded_own.is_some() || self.on_expanded_change.is_some() {
                let current = expanded.clone();
                let own = expanded_own.clone();
                let cb = self.on_expanded_change.clone();
                let allows_multiple = self.allows_multiple_expanded;
                disclosure = disclosure.on_expanded_change(move |_next, window, cx| {
                    let next = crate::accordion::next_expanded(&current, &key, allows_multiple);
                    if let Some(held) = &own {
                        let held_next = next.clone();
                        held.update(cx, |expanded, cx| {
                            *expanded = held_next;
                            cx.notify();
                        });
                    }
                    if let Some(f) = &cb {
                        f(&next, window, cx);
                    }
                });
            }
            el = el.child(disclosure);
        }
        el.into_any_element()
    }
}
