//! Kbd — port of `@heroui/kbd`.

use gpui::{prelude::*, px, AnyElement, App, IntoElement, ParentElement, RenderOnce, Styled, Window};
use herogpui_theme::ActiveTheme;


/// Visual style of a key (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KbdVariant {
    /// Raised key with a border and the field shadow.
    #[default]
    Default,
    /// Flat, borderless key for inline prose.
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

const MONO_FONT: &str = "Consolas";

impl RenderOnce for Kbd {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let layout = cx.layout();

        let (h, min_w, text) = (px(24.), px(24.), px(14.));

        let mut el = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .px(px(4.))
            .min_w(min_w)
            .h(h)
            .rounded(crate::util::control_radius(cx))
            .text_size(text)
            .font_family(MONO_FONT);

        el = match self.variant {
            KbdVariant::Default => el
                .bg(colors.surface.background)
                .border(layout.border_width)
                .border_color(colors.border)
                .text_color(colors.foreground)
                .when(!layout.field_shadow.is_empty(), |e: gpui::Div| {
                    e.shadow(layout.field_shadow.clone())
                }),
            KbdVariant::Light => el
                .bg(colors.default.soft())
                .text_color(colors.muted),
        };

        el.children(self.children)
    }
}


