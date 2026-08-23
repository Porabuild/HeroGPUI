//! Card — port of `@heroui/card`.

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, IntoElement, ParentElement, RenderOnce, Window,
};
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
    radius: gpui::Pixels,
    width: Option<gpui::Pixels>,
    is_pressable: bool,
    is_hoverable: bool,
    is_blurred: bool,
    on_press: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            variant: CardVariant::Default,
            radius: px(14.0),
            width: None,
            is_pressable: false,
            is_hoverable: false,
            is_blurred: false,
            on_press: None,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn radius(mut self, radius: impl Into<gpui::Pixels>) -> Self {
        self.radius = radius.into();
        self
    }

    /// Fixed card width.
    pub fn w(mut self, v: impl Into<gpui::Pixels>) -> Self {
        self.width = Some(v.into());
        self
    }

    /// Makes the card interactive (`isPressable`).
    pub fn is_pressable(mut self, v: bool) -> Self {
        self.is_pressable = v;
        self
    }

    /// Hoverable without press (`isHoverable` — v3).
    pub fn is_hoverable(mut self, v: bool) -> Self {
        self.is_hoverable = v;
        self
    }

    /// Blurred background (`isBlurred` — v3, e.g., footer blur).
    pub fn is_blurred(mut self, v: bool) -> Self {
        self.is_blurred = v;
        self
    }

    pub fn on_press(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(Box::new(f));
        self.is_pressable = true;
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
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(self.radius)
            .bg(if self.is_blurred {
                gpui::hsla(surface_bg.h, surface_bg.s, surface_bg.l, 0.8)
            } else {
                surface_bg
            })
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

        if self.is_blurred {
            el = el.border_1().border_color(colors.separator.alpha(0.5));
        }

        let needs_hover = self.is_pressable || self.is_hoverable;
        if needs_hover {
            let lifted = layout.overlay_shadow.clone();
            let mut stateful = el
                .id("pressable-card")
                .cursor_pointer()
                .hover(move |s| s.shadow(lifted.clone()));
            if let Some(on_press) = self.on_press {
                stateful =
                    stateful.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            } else if self.is_pressable {
                // is_pressable without handler still shows hover affordance
            }
            stateful.into_any_element()
        } else {
            el.into_any_element()
        }
    }
}

/// Padded header section (`CardHeader`).
#[derive(IntoElement)]
pub struct CardHeader {
    children: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new() -> Self {
        Self { children: Vec::new() }
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
        gpui::div()
            .flex()
            .items_center()
            .gap(px(12.))
            .px(px(16.))
            .py(px(12.))
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(gpui::FontWeight::SEMIBOLD)
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
        Self { children: Vec::new() }
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
        gpui::div()
            .flex()
            .flex_col()
            .px(px(16.))
            .py(px(16.))
            .text_size(px(14.))
            .line_height(px(24.))
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
        Self { children: Vec::new() }
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
        gpui::div()
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(16.))
            .py(px(12.))
            .text_size(px(14.))
            .children(self.children)
    }
}
