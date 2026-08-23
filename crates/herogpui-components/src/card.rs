//! Card — port of `@heroui/card`.

use gpui::{prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Window};
use herogpui_theme::ActiveTheme;

/// Card surface style (`shadow|bordered`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    /// No background — for cards with custom painting.
    Transparent,
    /// `bg-surface`
    #[default]
    Default,
    /// `bg-surface-secondary`
    Secondary,
    /// `bg-surface-tertiary`
    Tertiary,
}

impl CardVariant {
    pub const ALL: [CardVariant; 4] = [
        CardVariant::Transparent,
        CardVariant::Default,
        CardVariant::Secondary,
        CardVariant::Tertiary,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CardVariant::Transparent => "Transparent",
            CardVariant::Default => "Default",
            CardVariant::Secondary => "Secondary",
            CardVariant::Tertiary => "Tertiary",
        }
    }
}

/// HeroUI Card container.
#[derive(IntoElement)]
pub struct Card {
    variant: CardVariant,
    width: Option<gpui::Pixels>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            variant: CardVariant::Default,
            width: None,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Fixed card width.
    pub fn w(mut self, v: impl Into<gpui::Pixels>) -> Self {
        self.width = Some(v.into());
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();
        // v3 surface alias, falls back to content1
        let surface_bg = colors.surface.background;
        let surface_fg = colors.surface.foreground;
        // `.card` is `flex flex-col gap-3 p-4`: the card is the padded box and
        // its parts (`__header`, `__content`, `__footer`) carry none of their
        // own, which is why this used to double the inset on every section.
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .overflow_hidden()
            .rounded(crate::util::container_radius(cx))
            .bg(surface_bg)
            .text_color(surface_fg)
            .children(self.children);

        if let Some(w) = self.width {
            el = el.w(w);
        }

        // Every non-transparent level gets the surface shadow; the background
        // is what distinguishes them.
        el = match self.variant {
            CardVariant::Transparent => el,
            CardVariant::Default => el.bg(colors.surface.background),
            CardVariant::Secondary => el.bg(colors.surface_secondary),
            CardVariant::Tertiary => el.bg(colors.surface_tertiary),
        };
        if self.variant != CardVariant::Transparent && !layout.surface_shadow.is_empty() {
            el = el.shadow(layout.surface_shadow.clone());
        }
        if self.variant == CardVariant::Transparent {
            el = el.border(layout.border_width).border_color(colors.border);
        }

        el
    }
}

/// Padded header section (`CardHeader`).
#[derive(IntoElement)]
pub struct CardHeader {
    children: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for CardHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for CardHeader {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // `.card__header` is `flex flex-col`; the title inside it is `text-sm
        // leading-6 font-medium`.
        gpui::div()
            .flex()
            .flex_col()
            .text_size(px(14.))
            .line_height(px(24.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .children(self.children)
    }
}

/// Padded body section (`CardBody`).
#[derive(IntoElement)]
pub struct CardBody {
    children: Vec<AnyElement>,
}

impl CardBody {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for CardBody {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for CardBody {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardBody {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // `.card__content` is `flex flex-1 flex-col gap-1` with `text-sm`
        // leading-5 from `.card__description`; the padding is the card's.
        gpui::div()
            .flex()
            .flex_1()
            .flex_col()
            .gap(px(4.))
            .text_size(px(14.))
            .line_height(px(20.))
            .children(self.children)
    }
}

/// Padded footer section (`CardFooter`).
#[derive(IntoElement)]
pub struct CardFooter {
    children: Vec<AnyElement>,
}

impl CardFooter {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for CardFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for CardFooter {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for CardFooter {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // `.card__footer` is `flex flex-row items-center` -- no padding of its
        // own, and the gap comes from whatever the caller puts in it.
        gpui::div()
            .flex()
            .items_center()
            .gap(px(8.))
            .text_size(px(14.))
            .children(self.children)
    }
}
