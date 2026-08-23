//! Link — port of `@heroui/link` (v3).
//!
//! Mirrors the React API: `href`, `target`, `rel`, `download`, `isDisabled`.
//! Links draw with the `--link` token (which defaults to `--foreground`), not a
//! colour role — v3 removed the `color` prop.

use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

/// A press handler. `Arc` rather than `Box` because it is bound twice: the
/// pointer's `on_click` and the keyboard's Enter/Space both run it.
type OnPress = std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Link.
#[derive(IntoElement)]
pub struct Link {
    id: ElementId,
    label: Option<SharedString>,
    href: Option<String>,
    is_disabled: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    /// `Link.Icon` — the glyph v3 composes beside the label. `None` when the
    /// caller composes no icon at all.
    icon: Option<gpui::AnyElement>,
    /// Whether the icon comes first. v3 gets this from where `Link.Icon` sits
    /// among the link's children.
    icon_first: bool,
    on_press: Option<OnPress>,
}

impl Link {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            href: None,
            is_disabled: false,
            auto_focus: false,
            icon: None,
            icon_first: false,
            on_press: None,
        }
    }

    /// `Link.Icon` (`.link__icon`) — the glyph beside the label. Pass
    /// `icons::EXTERNAL_LINK` for the one v3 draws by default.
    pub fn icon(mut self, el: impl IntoElement) -> Self {
        self.icon = Some(el.into_any_element());
        self
    }

    /// Puts the icon before the label, which v3 does by ordering the children.
    pub fn icon_first(mut self, v: bool) -> Self {
        self.icon_first = v;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `autoFocus` — take focus on the first render.
    ///
    /// A link is not otherwise a focus target here, so this also makes it one.
    pub fn auto_focus(mut self, v: bool) -> Self {
        self.auto_focus = v;
        self
    }

    /// `onPress` — extra behaviour, in addition to opening `href`.
    pub fn on_press(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for Link {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `autoFocus` needs a focus target, and a link is not one by default.
        // `focus_once` takes `cx` mutably, so it runs before the tokens.
        // A link is a tab stop and rings like one -- `.link:focus-visible` is
        // `status-focused` -- so the handle exists whether or not `autoFocus`
        // asked for it.
        let focus = crate::util::tab_stop_handle(
            ElementId::Name(format!("{:?}-link-focus", self.id).into()),
            window,
            cx,
        );
        if self.auto_focus {
            crate::util::focus_once(
                window,
                cx,
                ElementId::Name(format!("{:?}-link-autofocus", self.id).into()),
                &focus,
            );
        }

        let colors = cx.colors();
        let color = colors.link;

        let mut el = div()
            .id(self.id.clone())
            .flex()
            .items_center()
            .gap(px(3.))
            .w_auto()
            .text_color(color)
            // `.link { @apply font-semibold no-underline hover:underline; }`
            .font_weight(gpui::FontWeight::MEDIUM)
            .rounded(crate::util::small_radius(cx))
            .border_color(color)
            .pb(px(1.))
            .track_focus(&focus)
            // `.link:focus-visible` is `status-focused`.
            .map(|el| {
                crate::util::ring_if_focused(el, &focus, true, Vec::new(), window, cx)
            });

        if self.is_disabled {
            el = el.opacity(cx.layout().disabled_opacity);
        } else {
            // gpui panics on a second `hover` call, so the underline and the
            // colour shift have to share one closure.
            let hover_color = colors.accent.color;
            el = el
                .cursor_pointer()
                .hover(move |s| {
                    s.text_color(hover_color)
                        .border_color(hover_color)
                        .border_b_1()
                })
                // `.link[data-pressed]` keeps the underline and takes the muted
                // decoration to full strength.
                .active(move |s| {
                    s.text_color(hover_color)
                        .border_color(hover_color)
                        .border_b_1()
                        .opacity(0.85)
                });
        }

        // v3 orders `Link.Icon` among the children, so the icon can lead or
        // trail the label.
        if self.icon_first {
            if let Some(icon) = self.icon.take() {
                el = el.child(icon);
            }
        }
        if let Some(label) = self.label.clone() {
            el = el.child(label.to_string());
        }
        if !self.icon_first {
            if let Some(icon) = self.icon.take() {
                el = el.child(icon);
            }
        }

        if !self.is_disabled {
            let href = self.href.clone();
            let user_click = self.on_press;
            el = el.on_click(move |ev: &ClickEvent, window, cx| {
                if let Some(f) = &user_click {
                    f(ev, window, cx);
                }
                if let Some(url) = &href {
                    cx.open_url(url);
                }
            });
        }

        el
    }
}
