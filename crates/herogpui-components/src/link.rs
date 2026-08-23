//! Link — port of `@heroui/link` (v3).
//!
//! Mirrors the React API: `href`, `target`, `rel`, `download`, `isDisabled`.
//! Links draw with the `--link` token (which defaults to `--foreground`), not a
//! colour role — v3 removed the `color` prop.

use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    RenderOnce, SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

type OnPress = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Link.
#[derive(IntoElement)]
pub struct Link {
    id: ElementId,
    label: Option<SharedString>,
    href: Option<String>,
    is_disabled: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
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
            on_press: None,
        }
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
        self.on_press = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Link {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `autoFocus` needs a focus target, and a link is not one by default.
        // `focus_once` takes `cx` mutably, so it runs before the tokens.
        let focus = self.auto_focus.then(|| {
            let handle = window.use_keyed_state(
                gpui::ElementId::Name(format!("{:?}-link-focus", self.id).into()),
                cx,
                |_, cx| cx.focus_handle(),
            );
            let handle = handle.read(cx).clone();
            crate::util::focus_once(
                window,
                cx,
                gpui::ElementId::Name(format!("{:?}-link-autofocus", self.id).into()),
                &handle,
            );
            handle
        });

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
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .border_color(color)
            .pb(px(1.))
            .when_some(focus, |el, handle| el.track_focus(&handle));

        if self.is_disabled {
            el = el.opacity(cx.layout().disabled_opacity);
        } else {
            // gpui panics on a second `hover` call, so the underline and the
            // colour shift have to share one closure.
            let hover_color = colors.accent.color;
            el = el.cursor_pointer().hover(move |s| {
                s.text_color(hover_color).border_color(hover_color).border_b_1()
            });
        }

        if let Some(label) = self.label {
            el = el.child(label.to_string());
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
