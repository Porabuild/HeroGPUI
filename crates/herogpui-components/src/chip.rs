//! Chip — port of `@heroui/chip`.

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// Chip visual style (`solid|bordered|light|flat|dot|faded|shadow`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipVariant {
    /// Filled with the chip color.
    Primary,
    /// Filled with `default`, colored text.
    #[default]
    Secondary,
    /// Bordered, transparent fill.
    Tertiary,
    /// The color at 15% with colored text.
    Soft,
}

impl ChipVariant {
    pub const ALL: [ChipVariant; 4] = [
        ChipVariant::Primary,
        ChipVariant::Secondary,
        ChipVariant::Tertiary,
        ChipVariant::Soft,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ChipVariant::Primary => "Primary",
            ChipVariant::Secondary => "Secondary",
            ChipVariant::Tertiary => "Tertiary",
            ChipVariant::Soft => "Soft",
        }
    }
}

/// HeroUI Chip.
#[derive(IntoElement)]
pub struct Chip {
    label: SharedString,
    variant: ChipVariant,
    color: Color,
    size: Size,
    start_content: Option<AnyElement>,
    on_close: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Chip {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ChipVariant::default(),
            color: Color::Default,
            size: Size::Md,
            start_content: None,
            on_close: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }


    pub fn start_content(mut self, el: impl IntoElement) -> Self {
        self.start_content = Some(el.into_any_element());
        self
    }

    /// Shows a close button (`onClose`).
    pub fn on_close(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let fg_default = cx.colors().foreground;

        let (height, text, pad_x) = match self.size {
            Size::Sm => (px(20.), px(11.), px(6.)),
            Size::Md => (px(24.), px(12.), px(8.)),
            Size::Lg => (px(28.), px(14.), px(10.)),
        };

        let mut el = gpui::div()
            .flex()
            .items_center()
            .gap(px(4.))
            .h(height)
            .px(pad_x)
            .text_size(text)
            .line_height(px(16.))
            .rounded(crate::util::control_radius(cx))
            .whitespace_nowrap()
            .overflow_hidden()
            .flex_shrink_0();

        el = match self.variant {
            ChipVariant::Primary => el.bg(sem.color).text_color(sem.foreground),
            ChipVariant::Secondary => el
                .bg(cx.colors().default.color)
                .text_color(if self.color == Color::Default {
                    fg_default
                } else {
                    sem.soft_foreground()
                }),
            ChipVariant::Tertiary => el
                .border(cx.layout().border_width)
                .border_color(cx.colors().border)
                .text_color(if self.color == Color::Default {
                    fg_default
                } else {
                    sem.soft_foreground()
                }),
            ChipVariant::Soft => el.bg(sem.soft()).text_color(if self.color == Color::Default {
                fg_default
            } else {
                sem.soft_foreground()
            }),
        };

        if let Some(start) = self.start_content {
            el = el.child(start);
        }
        el = el.child(self.label.to_string());

        if let Some(on_close) = self.on_close {
            let icon_color = cx.colors().foreground;
            el = el.child(
                gpui::div()
                    .id("chip-close")
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(height - px(6.))
                    .rounded_full()
                    .hover(|s| s.bg(gpui::black().alpha(0.12)))
                    .on_click(move |ev: &ClickEvent, window, cx| on_close(ev, window, cx))
                    .child(
                        gpui::svg()
                            .size(px(10.))
                            .path(icons::CLOSE)
                            .text_color(icon_color),
                    ),
            );
        }

        el
    }
}
