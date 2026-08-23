//! CloseButton — port of `@heroui/close-button`.
//!
//! A button for dismissing dialogs, modals and inline content. Mirrors the
//! React API: `variant`, `isDisabled`, `onPress` and a custom-icon slot that
//! replaces the default close glyph.

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// Visual variant of a close button. React exposes a single `default` variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloseButtonVariant {
    #[default]
    Default,
}

type OnPress = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI CloseButton.
#[derive(IntoElement)]
pub struct CloseButton {
    id: ElementId,
    is_disabled: bool,
    /// Replaces the default close glyph (`children` in React).
    icon: Option<AnyElement>,
    on_press: Option<OnPress>,
}

impl CloseButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_disabled: false,
            icon: None,
            on_press: None,
        }
    }



    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Supplies a custom icon in place of the default close glyph.
    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn on_press(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }

}

impl RenderOnce for CloseButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        let (box_size, icon_size) = (px(28.), px(16.));
        let hover_bg = colors.default.with_alpha(0.15);

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .size(box_size)
            .rounded(px(8.))
            .text_color(colors.muted);

        if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        } else {
            el = el
                .hover(move |s| s.bg(hover_bg).text_color(colors.foreground))
                .active(|s| s.opacity(0.7));
        }

        el = match self.icon {
            Some(icon) => el.child(icon),
            None => el.child(
                gpui::svg()
                    .size(icon_size)
                    .path(icons::CLOSE)
                    .text_color(colors.muted),
            ),
        };

        if let Some(on_press) = self.on_press {
            if !self.is_disabled {
                el = el.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            }
        }

        el
    }
}
