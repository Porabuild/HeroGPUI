//! Disclosure & DisclosureGroup — HeroUI v3 `Disclosure`.
//!
//! A single collapsible section. `DisclosureGroup` manages multiple with
//! single/multiple expansion. Thin, accessible wrappers around `Accordion`.

use std::collections::HashSet;

use gpui::{
    px, AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};

use crate::accordion::{Accordion, AccordionItem, AccordionVariant};

/// Single Disclosure — like an accordion with one item.
#[derive(IntoElement)]
pub struct Disclosure {
    title: SharedString,
    is_expanded: bool,
    is_disabled: bool,
    children: Vec<AnyElement>,
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

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            is_expanded: false,
            is_disabled: false,
            children: Vec::new(),
            on_toggle: None,
        }
    }

    pub fn is_expanded(mut self, v: bool) -> Self {
        self.is_expanded = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }
}

impl ParentElement for Disclosure {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let item = AccordionItem::new("disclosure", self.title.clone()).content(
            gpui::div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .children(self.children),
        );
        let mut set = HashSet::new();
        if self.is_expanded {
            set.insert(SharedString::from("disclosure"));
        }
        let was_expanded = self.is_expanded;
        Accordion::new(vec![item])
            .expanded_keys(set)
            .variant(AccordionVariant::Surface)
            .is_disabled(self.is_disabled)
            .on_toggle({
                let cb = self.on_toggle.clone();
                move |_k, w, cx| {
                    if let Some(f) = &cb {
                        f(!was_expanded, w, cx);
                    }
                }
            })
            .into_any_element()
    }
}

/// Group of Disclosures — mirrors Accordion but with `Disclosure` naming.
#[derive(IntoElement)]
pub struct DisclosureGroup {
    items: Vec<(SharedString, SharedString, Vec<AnyElement>)>,
    expanded: HashSet<SharedString>,
    on_toggle: Option<std::sync::Arc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>>,
}

impl DisclosureGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            expanded: HashSet::new(),
            on_toggle: None,
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

    pub fn expanded_keys(mut self, keys: HashSet<SharedString>) -> Self {
        self.expanded = keys;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::sync::Arc::new(f));
        self
    }
}

impl Default for DisclosureGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for DisclosureGroup {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let items: Vec<AccordionItem> = self
            .items
            .into_iter()
            .map(|(k, t, children)| {
                let mut it = AccordionItem::new(k, t);
                // join children into one content block
                it = it.content(gpui::div().flex().flex_col().gap(px(4.)).children(children));
                it
            })
            .collect();

        let mut acc = Accordion::new(items).expanded_keys(self.expanded.clone());
        if let Some(cb) = self.on_toggle.clone() {
            acc = acc.on_toggle(move |k, w, cx| cb(k, w, cx));
        }
        acc.into_any_element()
    }
}
