//! Kbd — port of `@heroui/kbd`.

use gpui::{px, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window};
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
    /// Every Kbd variant, in declaration order.
    pub const ALL: [KbdVariant; 2] = [KbdVariant::Default, KbdVariant::Light];

    /// A human-readable label for this variant.
    pub fn label(self) -> &'static str {
        match self {
            KbdVariant::Default => "Default",
            KbdVariant::Light => "Light",
        }
    }
}

/// Keyboard key display (`<Kbd>`).
#[must_use = "a component does nothing until it is rendered: add it as a child or return it from `render`"]
#[derive(IntoElement)]
pub struct Kbd {
    variant: KbdVariant,
    children: Vec<AnyElement>,
    /// The corner radius, in place of `--radius-lg`.
    radius: Option<Pixels>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
}

impl Kbd {
    /// Creates an empty Kbd.
    pub fn new() -> Self {
        Self {
            variant: KbdVariant::Default,
            children: Vec::new(),
            radius: None,
            sx: None,
        }
    }

    /// Sets the Kbd variant.
    pub fn variant(mut self, v: KbdVariant) -> Self {
        self.variant = v;
        self
    }

    /// The corner radius, in place of `--radius-lg`. Not a v3 prop; the
    /// removed v2 `radius` prop is prohibited and this is a per-component
    /// repository extension.
    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the key's root element after every value the variant and the
    /// active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        crate::util::refine_sx(&mut self.sx, style);
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
            .rounded(self.radius.unwrap_or_else(|| crate::util::key_radius(cx)))
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
        el = el.children(self.children);
        el = crate::util::apply_sx(el, &self.sx);
        el
    }
}

crate::util::impl_component_styled!(Kbd);
