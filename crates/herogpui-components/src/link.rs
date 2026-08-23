//! Link — port of `@heroui/link` (v3).
//!
//! Mirrors the React API: `href`, `target`, `rel`, `download`, `isDisabled`.
//! Links draw with the `--link` token (which defaults to `--foreground`), not a
//! colour role — v3 removed the `color` prop.

use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// When the underline is drawn.
///
/// Not a v3 prop — v3 expresses this with Tailwind utilities, which have no
/// gpui equivalent, so it is exposed here instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Underline {
    None,
    #[default]
    Hover,
    Always,
}

impl Underline {
    pub const ALL: [Underline; 3] = [Underline::None, Underline::Hover, Underline::Always];

    pub fn label(self) -> &'static str {
        match self {
            Underline::None => "None",
            Underline::Hover => "Hover",
            Underline::Always => "Always",
        }
    }
}

type OnClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Link.
#[derive(IntoElement)]
pub struct Link {
    id: ElementId,
    label: Option<SharedString>,
    href: Option<String>,
    underline: Underline,
    size: Pixels,
    is_disabled: bool,
    is_external: bool,
    /// `autoFocus` — take focus on the first render.
    auto_focus: bool,
    on_click: Option<OnClick>,
}

impl Link {
    /// `onPress` — the v3 name for [`Link::on_click`].
    pub fn on_press(
        self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click(handler)
    }

    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            href: None,
            underline: Underline::default(),
            size: px(14.),
            is_disabled: false,
            is_external: false,
            auto_focus: false,
            on_click: None,
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




    pub fn underline(mut self, underline: Underline) -> Self {
        self.underline = underline;
        self
    }

    pub fn text_size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
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

    /// Appends an external-link glyph.
    pub fn is_external(mut self, v: bool) -> Self {
        self.is_external = v;
        self
    }

    /// Extra click behaviour, in addition to opening `href`.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
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
            .text_size(self.size)
            .border_color(color)
            .pb(px(1.))
            .when(self.underline == Underline::Always, |el| el.border_b_1())
            .when_some(focus, |el, handle| el.track_focus(&handle));

        if self.is_disabled {
            el = el.opacity(cx.layout().disabled_opacity);
        } else {
            // gpui panics on a second `hover` call, so the underline and the
            // colour shift have to share one closure.
            let underline_on_hover = self.underline == Underline::Hover;
            let hover_color = colors.accent.color;
            el = el.cursor_pointer().hover(move |s| {
                let s = s.text_color(hover_color).border_color(hover_color);
                if underline_on_hover {
                    s.border_b_1()
                } else {
                    s
                }
            });
        }

        if let Some(label) = self.label {
            el = el.child(label.to_string());
        }

        if self.is_external {
            el = el.child(
                gpui::svg()
                    .size(px(12.))
                    .path(icons::EXTERNAL_LINK)
                    .flex_shrink_0()
                    .text_color(color),
            );
        }

        if !self.is_disabled {
            let href = self.href.clone();
            let user_click = self.on_click;
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
