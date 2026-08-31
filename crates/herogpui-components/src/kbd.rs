//! Kbd — port of `@heroui/kbd`.

use gpui::{px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window};
use herogpui_theme::ActiveTheme;

/// Visual style of a key (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KbdVariant {
    /// Neutral key on the `bg-default` chip.
    #[default]
    Default,
    /// Transparent key for inline prose.
    Light,
}

impl KbdVariant {
    pub const ALL: [KbdVariant; 2] = [KbdVariant::Default, KbdVariant::Light];

    pub fn label(self) -> &'static str {
        match self {
            KbdVariant::Default => "Default",
            KbdVariant::Light => "Light",
        }
    }
}

/// Keyboard key display (`<Kbd>`).
#[derive(IntoElement)]
pub struct Kbd {
    variant: KbdVariant,
    children: Vec<AnyElement>,
}

impl Kbd {
    pub fn new() -> Self {
        Self {
            variant: KbdVariant::Default,
            children: Vec::new(),
        }
    }

    pub fn variant(mut self, v: KbdVariant) -> Self {
        self.variant = v;
        self
    }
}

impl Default for Kbd {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Kbd {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();

        let (h, text) = (px(24.), px(14.));

        let mut el = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .text_center()
            .gap(px(2.))
            .px(px(8.))
            .h(h)
            .rounded(crate::util::key_radius(cx))
            // Tailwind's `text-sm` pairs 14px with a 20px leading; gpui's phi
            // default would give 14 x 1.618 ≈ 23px.
            .text_size(text)
            .line_height(px(20.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .whitespace_nowrap()
            .text_color(colors.muted);

        el = match self.variant {
            KbdVariant::Default => el.bg(colors.default.color),
            KbdVariant::Light => el.bg(gpui::transparent_black()),
        };

        // `.kbd__content` is the key text itself; `.kbd__abbr` is the `<abbr>`
        // v3 wraps it in for screen readers, which has no analogue here.
        el.children(self.children)
    }
}
