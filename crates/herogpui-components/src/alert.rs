//! Alert — port of `@heroui/alert`.

use gpui::{
    prelude::*, px, App, ClickEvent, IntoElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

type OnClose = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Alert.
#[derive(IntoElement)]
pub struct Alert {
    title: SharedString,
    description: Option<SharedString>,
    color: Color,
    is_closable: bool,
    on_close: Option<OnClose>,
}

impl Alert {
    /// `status` — the v3 name for [`Alert::color`]; the values are the same
    /// semantic roles.
    /// `status` — the alert's visual status.
    pub fn status(mut self, status: Color) -> Self {
        self.color = status;
        self
    }

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            color: Color::Accent,
            is_closable: false,
            on_close: None,
        }
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Shows a close button (`isClosable`).
    pub fn is_closable(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.is_closable = true;
        self.on_close = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();

        // v3 dropped Alert's `variant`: the color role is the only axis.
        let bg = if self.color == Color::Default {
            colors.surface_secondary
        } else {
            sem.soft()
        };
        let fg = colors.foreground;
        let title_color = if self.color == Color::Default {
            colors.foreground
        } else {
            sem.soft_foreground()
        };

        let mut alert = gpui::div()
            .flex()
            .items_start()
            .gap(px(16.))
            .w_full()
            .px(px(16.))
            .py(px(12.))
            .rounded(crate::util::control_radius(cx))
            .bg(bg)
            .text_color(fg);

        // icon dot
        let icon_color = if self.color == Color::Default {
            colors.muted
        } else {
            sem.color
        };
        alert = alert.child(
            gpui::svg()
                .size(px(18.))
                .path(icons::ELLIPSIS)
                .text_color(icon_color)
                .flex_shrink_0(),
        );

        let mut text_col = gpui::div().flex().flex_col().gap(px(2.)).flex_1();
        text_col = text_col.child(
            gpui::div()
                .text_size(px(14.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(self.title.to_string()),
        );
        if let Some(desc) = self.description {
            text_col = text_col.child(
                gpui::div()
                    // `.alert__description` is `text-sm`.
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .child(desc.to_string()),
            );
        }
        alert = alert.child(text_col);

        if self.is_closable {
            if let Some(on_close) = self.on_close {
                alert = alert.child(
                    gpui::div()
                        .id("alert-close")
                        .cursor_pointer()
                        .flex_shrink_0()
                        .child(gpui::svg().size(px(14.)).path(icons::CLOSE).text_color(fg))
                        .on_click(move |ev, w, cx| on_close(ev, w, cx)),
                );
            }
        }

        alert
    }
}
