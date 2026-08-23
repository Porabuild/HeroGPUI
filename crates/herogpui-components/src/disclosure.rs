//! Disclosure & DisclosureGroup — HeroUI v3 `Disclosure`.
//!
//! A single collapsible section: a trigger and a `p-2` body, which is all v3's
//! `disclosure.css` gives it -- not an `Accordion` with one item, which is what
//! this used to render. `DisclosureGroup` keeps the accordion, whose group
//! behaviour (single or multiple expansion) it needs.

use std::collections::HashSet;

use gpui::{
    px, AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // A v3 Disclosure is not a small Accordion: `.disclosure` is `relative`,
        // its trigger is whatever the caller passes (`<Button slot="trigger">`
        // in v3's own examples) with a `.disclosure__indicator` chevron, and
        // `.disclosure__body` is `p-2`. Rendering it as a one-item accordion
        // gave it a card, a 16px trigger row and a separator that v3's sheet
        // has no rule for.
        let expanded = self.is_expanded;
        let cb = self.on_toggle.clone();
        let trigger = crate::button::Button::new(gpui::ElementId::Name(
            format!("{}-disclosure-trigger", self.title).into(),
        ))
        .variant(if expanded {
            herogpui_core::Variant::Secondary
        } else {
            herogpui_core::Variant::Tertiary
        })
        .label(self.title.clone())
        .is_disabled(self.is_disabled)
        // `.disclosure__indicator` is `ms-auto size-4` and turns 180 degrees
        // when the panel is open, which is a glyph swap here.
        .end_content(
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
            if let Some(f) = &cb {
                f(!expanded, w, cx);
            }
        });

        let mut el = gpui::div().relative().flex().flex_col().child(trigger);
        // v3 keeps the panel mounted and animates its height; `overlay_phase`
        // is what gives a collapsing body its exit here.
        if expanded {
            el = el.child(
                crate::anim::entering(
                    gpui::div()
                        // `.disclosure__body` is `p-2`.
                        .p(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .children(self.children),
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
        // `.disclosure-group` is `w-full` and nothing else: v3's group *is* a
        // column of `Disclosure`s, one of which may be open, so this renders
        // them rather than an `Accordion` -- which would give every row the
        // card, the padded trigger and the separator the sheet has no rule for.
        let mut el = gpui::div().w_full().flex().flex_col();
        for (key, title, children) in self.items {
            let expanded = self.expanded.contains(&key);
            let cb = self.on_toggle.clone();
            el = el.child(
                Disclosure::new(title)
                    .is_expanded(expanded)
                    .on_expanded_change(move |_next, window, cx| {
                        if let Some(f) = &cb {
                            f(&key, window, cx);
                        }
                    })
                    .children(children),
            );
        }
        el.into_any_element()
    }
}
