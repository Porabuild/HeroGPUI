//! Typography — port of `@heroui/typography`.
//!
//! Semantic typography primitives for headings, body copy and inline code.
//! Mirrors the React API: `type`, `align`, `color`, `weight`, `truncate`, plus
//! the `Typography.Heading` / `Typography.Paragraph` / `Typography.Code` /
//! `Typography.Prose` convenience primitives.

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window,
};
use herogpui_theme::ActiveTheme;

const MONO_FONT: &str = "Consolas";

/// Semantic typography style (`type` prop).
///
/// Scale values are taken verbatim from the HeroUI v3 typography scale:
/// `h1` 36/600/1.11 through `body-xs` 12/400/1.25.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TypographyType {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    #[default]
    Body,
    BodySm,
    BodyXs,
    Code,
}

impl TypographyType {
    /// `(font-size, line-height)` in pixels.
    pub fn metrics(self) -> (Pixels, Pixels) {
        match self {
            Self::H1 => (px(36.), px(40.)),
            Self::H2 => (px(30.), px(35.)),
            Self::H3 => (px(24.), px(30.)),
            Self::H4 => (px(20.), px(27.)),
            Self::H5 => (px(18.), px(25.)),
            Self::H6 => (px(16.), px(24.)),
            Self::Body => (px(16.), px(28.)),
            Self::BodySm => (px(14.), px(21.)),
            Self::BodyXs => (px(12.), px(15.)),
            Self::Code => (px(14.), px(21.)),
        }
    }

    /// Default weight for this type — headings are semibold, body normal.
    pub fn default_weight(self) -> FontWeight {
        match self {
            Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6 => FontWeight::Semibold,
            _ => FontWeight::Normal,
        }
    }

    /// Whether this type renders in the monospace family.
    pub fn is_mono(self) -> bool {
        matches!(self, Self::Code)
    }

    /// Maps a heading level (1-6) to the matching type; higher levels clamp.
    pub fn heading(level: u8) -> Self {
        match level {
            1 => Self::H1,
            2 => Self::H2,
            3 => Self::H3,
            4 => Self::H4,
            5 => Self::H5,
            _ => Self::H6,
        }
    }
}

/// Text alignment (`align` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

/// Text color (`color` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextColor {
    #[default]
    Default,
    Muted,
}

/// Font weight override (`weight` prop).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Medium,
    Semibold,
    Bold,
}

impl FontWeight {
    fn to_gpui(self) -> gpui::FontWeight {
        match self {
            Self::Normal => gpui::FontWeight::NORMAL,
            Self::Medium => gpui::FontWeight::MEDIUM,
            Self::Semibold => gpui::FontWeight::SEMIBOLD,
            Self::Bold => gpui::FontWeight::BOLD,
        }
    }
}

/// Paragraph size for [`Typography::paragraph`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParagraphSize {
    #[default]
    Base,
    Sm,
    Xs,
}

/// HeroUI Typography.
#[derive(IntoElement)]
pub struct Typography {
    kind: TypographyType,
    align: TextAlign,
    color: TextColor,
    weight: Option<FontWeight>,
    truncate: bool,
    text: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl Typography {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            kind: TypographyType::default(),
            align: TextAlign::default(),
            color: TextColor::default(),
            weight: None,
            truncate: false,
            text: Some(text.into()),
            children: Vec::new(),
        }
    }

    /// Empty container — use with [`ParentElement`] children.
    pub fn container() -> Self {
        Self {
            kind: TypographyType::default(),
            align: TextAlign::default(),
            color: TextColor::default(),
            weight: None,
            truncate: false,
            text: None,
            children: Vec::new(),
        }
    }

    /// `Typography.Heading level={1..6}`.
    pub fn heading(level: u8, text: impl Into<SharedString>) -> Self {
        Self::new(text).kind(TypographyType::heading(level))
    }

    /// `Typography.Paragraph size="base" | "sm" | "xs"`.
    pub fn paragraph(size: ParagraphSize, text: impl Into<SharedString>) -> Self {
        Self::new(text).kind(match size {
            ParagraphSize::Base => TypographyType::Body,
            ParagraphSize::Sm => TypographyType::BodySm,
            ParagraphSize::Xs => TypographyType::BodyXs,
        })
    }

    /// `Typography.Code`.
    pub fn code(text: impl Into<SharedString>) -> Self {
        Self::new(text).kind(TypographyType::Code)
    }

    /// The `type` prop — named `kind` because `type` is a Rust keyword.
    pub fn kind(mut self, kind: TypographyType) -> Self {
        self.kind = kind;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn color(mut self, color: TextColor) -> Self {
        self.color = color;
        self
    }

    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn truncate(mut self, v: bool) -> Self {
        self.truncate = v;
        self
    }
}

impl ParentElement for Typography {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Typography {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let (size, line_height) = self.kind.metrics();
        let weight = self.weight.unwrap_or_else(|| self.kind.default_weight());

        let mut el = div()
            .text_size(size)
            .line_height(line_height)
            .font_weight(weight.to_gpui())
            .text_color(match self.color {
                TextColor::Default => colors.foreground,
                TextColor::Muted => colors.muted,
            });

        if self.kind.is_mono() {
            el = el.font_family(MONO_FONT);
        }

        // gpui has no `text-justify`; justify falls back to start alignment.
        el = match self.align {
            TextAlign::Start | TextAlign::Justify => el.text_left(),
            TextAlign::Center => el.text_center(),
            TextAlign::End => el.text_right(),
        };

        if self.truncate {
            el = el.truncate();
        }

        if let Some(text) = self.text {
            el = el.child(text.to_string());
        }
        el.children(self.children)
    }
}

/// `Typography.Prose` — applies HeroUI's typographic rhythm to a block of
/// already-semantic rich-text children.
#[derive(IntoElement)]
pub struct Prose {
    children: Vec<AnyElement>,
    gap: Pixels,
}

impl Prose {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            gap: px(12.),
        }
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl Default for Prose {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Prose {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Prose {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (size, line_height) = TypographyType::Body.metrics();
        div()
            .flex()
            .flex_col()
            .gap(self.gap)
            .text_size(size)
            .line_height(line_height)
            .text_color(cx.colors().foreground)
            .children(self.children)
    }
}
