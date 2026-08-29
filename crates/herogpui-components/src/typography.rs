//! Typography — port of `@heroui/typography` (HeroUI v3.2.4).
//!
//! Semantic typography primitives for headings, body copy and inline code.
//! Mirrors the React API: `type`, `align`, `color`, `weight`, `truncate`,
//! plus the `Typography.Heading` / `Typography.Paragraph` / `Typography.Code`
//! / `Typography.Prose` convenience primitives.
//!
//! Paint-only omissions, each because GPUI 0.2.2 lacks the primitive: the
//! headings' `tracking-tight` (no letter-spacing), `align: justify` (falls
//! back to start alignment), and the semantic `h1`–`h6`/`p`/`code` elements
//! (rendered as plain divs). The mono family is Tailwind's default mono stack
//! resolved on Windows. Upstream's per-tag `Prose` descendant styles cannot
//! be ported because GPUI has no ancestor context propagation.

use gpui::{
    div, px, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, SharedString, Styled,
    Window,
};
use herogpui_theme::ActiveTheme;

const MONO_FONT: &str = "Consolas";

/// Semantic typography style (`type` prop).
///
/// `(font-size, line-height)` pairs resolve the pinned `typography.css`
/// through Tailwind 4.3.0's default text scale: headings use the default
/// `text-*` leading (`h1` 36/40 through `h6` 16/24), body `leading-7`
/// (16/28), body-sm `leading-6` (14/24), body-xs `leading-5` (12/20) and
/// code the plain `text-sm` leading (14/20).
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
            Self::H2 => (px(30.), px(36.)),
            Self::H3 => (px(24.), px(32.)),
            Self::H4 => (px(20.), px(28.)),
            Self::H5 => (px(18.), px(28.)),
            Self::H6 => (px(16.), px(24.)),
            Self::Body => (px(16.), px(28.)),
            Self::BodySm => (px(14.), px(24.)),
            Self::BodyXs => (px(12.), px(20.)),
            Self::Code => (px(14.), px(20.)),
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

        // `typography--code` paints the `rounded-md bg-default px-1.5
        // py-0.5` chip on top of the mono `text-sm` run.
        if self.kind.is_mono() {
            el = el
                .font_family(MONO_FONT)
                .bg(colors.default.color)
                .rounded(crate::util::mark_radius(cx))
                .px(px(6.))
                .py(px(2.));
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

/// `Typography.Prose` — upstream's `.typography-prose` block: a plain
/// container that sets `text-foreground` and renders its children in order.
/// The per-tag descendant styles (`p`, `code`, `a`, lists, …) cannot be
/// ported because GPUI has no ancestor context propagation; children must be
/// already-semantic elements such as [`Typography`], which carry their own
/// metrics.
#[derive(IntoElement)]
pub struct Prose {
    children: Vec<AnyElement>,
}

impl Prose {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
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
        div()
            .text_color(cx.colors().foreground)
            .children(self.children)
    }
}
