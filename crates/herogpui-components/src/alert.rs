//! Alert — port of `@heroui/alert`.

use gpui::{
    px, AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// HeroUI Alert.
///
/// v3.2.4's API table carries only `status`/`className`/`children`: the
/// migration guide explicitly removes `isClosable`, `onClose` and
/// `closeButtonProps`, so a close affordance is composed by the caller as an
/// ordinary child (a `CloseButton`) instead of being built in.
#[derive(IntoElement)]
pub struct Alert {
    title: SharedString,
    description: Option<SharedString>,
    color: Color,
    /// Composed children — v3's "Additional content like buttons, close
    /// button, etc.", appended after the content column.
    children: Vec<AnyElement>,
}

impl Alert {
    /// `status` — the v3 name for `color`; the values are the same
    /// semantic roles.
    pub fn status(mut self, status: Color) -> Self {
        self.color = status;
        self
    }

    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            color: Color::Accent,
            children: Vec::new(),
        }
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }
}

impl ParentElement for Alert {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // End content; see the struct doc for the v3 composition rationale.
        self.children.extend(elements);
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
        // `.alert__indicator` is a `p-1` box around the glyph, not the glyph on
        // its own.
        alert = alert.child(
            gpui::div().p(px(4.)).flex_shrink_0().child(
                gpui::svg()
                    .size(px(18.))
                    .path(icons::ELLIPSIS)
                    .text_color(icon_color)
                    .flex_shrink_0(),
            ),
        );

        // `.alert__content` is the column that holds the title and the
        // description, beside the indicator.
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

        // Composed children go last; see the struct doc.
        alert = alert.children(self.children);

        alert
    }
}
