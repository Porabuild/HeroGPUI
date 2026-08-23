//! Chip — port of `@heroui/chip`.

use gpui::{
    px, AnyElement, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

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
}

impl Chip {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ChipVariant::default(),
            color: Color::Default,
            size: Size::Md,
            start_content: None,
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
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let fg_default = cx.colors().foreground;

        // `.chip` is `px-2 py-0.5 text-xs`, `--sm` is `px-1 py-0 text-xs` and
        // `--lg` is `px-3 py-1 text-sm`. Like a tag, a chip has no height of its
        // own: it is padding around one 20px line.
        let (pad_x, pad_y, text) = match self.size {
            Size::Sm => (px(4.), px(0.), px(12.)),
            Size::Md => (px(8.), px(2.), px(12.)),
            Size::Lg => (px(12.), px(4.), px(14.)),
        };

        let mut el = gpui::div()
            .flex()
            .items_center()
            .gap(px(2.))
            .px(pad_x)
            .py(pad_y)
            .text_size(text)
            // `leading-5` and `font-medium` on `.chip`.
            .line_height(px(20.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .rounded(crate::util::soft_radius(cx))
            .whitespace_nowrap()
            .overflow_hidden()
            .flex_shrink_0();

        el = match self.variant {
            ChipVariant::Primary => el.bg(sem.color).text_color(sem.foreground),
            ChipVariant::Secondary => {
                el.bg(cx.colors().default.color)
                    .text_color(if self.color == Color::Default {
                        fg_default
                    } else {
                        sem.soft_foreground()
                    })
            }
            ChipVariant::Tertiary => el
                .border(cx.layout().border_width)
                .border_color(cx.colors().border)
                .text_color(if self.color == Color::Default {
                    fg_default
                } else {
                    sem.soft_foreground()
                }),
            ChipVariant::Soft => el
                .bg(sem.soft())
                .text_color(if self.color == Color::Default {
                    fg_default
                } else {
                    sem.soft_foreground()
                }),
        };

        if let Some(start) = self.start_content {
            el = el.child(start);
        }
        // `.chip__label` is `px-0.5`.
        el = el.child(gpui::div().px(px(2.)).child(self.label.to_string()));

        el
    }
}
