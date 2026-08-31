//! Chip — port of `@heroui/chip`.

use gpui::{
    px, AnyElement, App, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled,
    Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::{ActiveTheme, ThemeColors};

/// Chip visual style (`primary | secondary | tertiary | soft`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipVariant {
    /// Filled with the role color, labelled in the role foreground.
    Primary,
    /// The base chip: filled with `default`, labelled in the color class's
    /// foreground.
    #[default]
    Secondary,
    /// Transparent fill — `.chip--tertiary` only clears `--chip-bg`, and
    /// chip.css declares no border for any chip.
    Tertiary,
    /// Filled with `--{role}-soft` (the default role mixes at 50%), labelled
    /// in the soft foreground.
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

/// HeroUI Chip root (`Chip`, upstream `.chip`).
///
/// v3's `ChipRoot` renders its children verbatim — an icon, a dot, a
/// [`ChipLabel`], a trailing element — in the order they are composed, and
/// auto-wraps plain-text children in the label part. This port makes that
/// wrap explicit: compose a [`ChipLabel`] where v3's basic usage relies on
/// it.
#[derive(IntoElement)]
pub struct Chip {
    variant: ChipVariant,
    color: Color,
    size: Size,
    children: Vec<AnyElement>,
}

impl Chip {
    pub fn new() -> Self {
        Self {
            variant: ChipVariant::default(),
            color: Color::Default,
            size: Size::Md,
            children: Vec::new(),
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
}

impl Default for Chip {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Chip {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// The chip's label part (`Chip.Label`, upstream `.chip__label`).
///
/// The `.chip__label` `px-0.5` lives here and nowhere else: a chip root's
/// arbitrary icon or dot children take no label padding.
#[derive(IntoElement)]
pub struct ChipLabel {
    children: Vec<AnyElement>,
}

impl ChipLabel {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Default for ChipLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ChipLabel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Resolves one chip's paint pair against v3.2.4's `.chip` cascade: the base
/// rule, the `.chip--{color}` classes, the variant rules, and the compound
/// variant×color rules. `None` paints no background (tertiary's transparent
/// fill); no chip carries a border.
fn paint(colors: &ThemeColors, variant: ChipVariant, color: Color) -> (Option<Hsla>, Hsla) {
    let role = colors.role(color.token());
    let muted_foreground = || {
        if color == Color::Default {
            colors.default.foreground
        } else {
            role.soft_foreground(colors.foreground)
        }
    };
    match variant {
        ChipVariant::Primary => (Some(role.color), role.foreground),
        ChipVariant::Secondary => (Some(colors.default.color), muted_foreground()),
        ChipVariant::Tertiary => (None, muted_foreground()),
        // `.chip--default.chip--soft` fills with `--default-soft`, a 50% mix —
        // not the 15% the accent and status roles use. `RoleColor::soft()`
        // carries that per-role weight itself.
        ChipVariant::Soft => (Some(role.soft()), muted_foreground()),
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let (bg, fg) = paint(colors, self.variant, self.color);
        let radius = crate::util::soft_radius(cx);

        // `.chip` is `px-2 py-0.5 text-xs leading-5 font-medium`, `--sm` is
        // `px-1 py-0 text-xs`, `--md` is `text-xs` and `--lg` is `px-3 py-1
        // text-sm`. Compiled Tailwind 4 lowers `leading-5` to
        // `--tw-leading: var(--leading-5)` and lowers every `text-*` utility
        // to `line-height: var(--tw-leading, <its own pair>)`, so the size
        // rules' restated `text-*` utilities consume the base's 20px line
        // instead of resetting it: one line height at every size. Like a tag,
        // a chip has no height of its own: it is padding around one line.
        let (pad_x, pad_y, text, leading) = match self.size {
            Size::Sm => (px(4.), px(0.), px(12.), px(20.)),
            Size::Md => (px(8.), px(2.), px(12.), px(20.)),
            Size::Lg => (px(12.), px(4.), px(14.), px(20.)),
        };

        let mut el = gpui::div()
            .flex()
            .debug_selector(|| "chip".to_owned())
            .items_center()
            .gap(px(2.))
            .px(pad_x)
            .py(pad_y)
            .text_size(text)
            .line_height(leading)
            // `font-medium` on `.chip`. v3 declares no `whitespace-nowrap` or
            // `overflow-hidden` on a chip, so a label constrained by its
            // parent wraps exactly as upstream's would.
            .font_weight(gpui::FontWeight::MEDIUM)
            .rounded(radius)
            .flex_shrink_0();

        el = match bg {
            Some(bg) => el.bg(bg),
            None => el,
        };
        el = el.text_color(fg);

        el.children(self.children)
    }
}

impl RenderOnce for ChipLabel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // `.chip__label` is `px-0.5`.
        gpui::div()
            .debug_selector(|| "chip-label".to_owned())
            .px(px(2.))
            .children(self.children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pure variant×color paint matrix of `chip.css`: the base rule,
    /// the `.chip--{color}` foreground classes, the variant rules, and the
    /// compound `.chip--{variant}.chip--{color}` cells, over both
    /// appearances. The headless test window cannot sample a fill, so the
    /// cascade is pinned here instead.
    #[test]
    fn paint_matrix_matches_the_chip_css_cascade() {
        for colors in [ThemeColors::light(), ThemeColors::dark()] {
            for color in Color::ALL {
                let role = colors.role(color.token());
                let muted_foreground = if color == Color::Default {
                    colors.default.foreground
                } else {
                    role.soft_foreground(colors.foreground)
                };

                // `.chip--primary.chip--{color}` fills with the role and
                // labels in the role foreground. Default has no compound rule,
                // so the base `--chip-bg: var(--default)` holds and the label
                // stays `currentColor` — which the port resolves to the
                // theme's default foreground because GPUI has no ancestor
                // color context.
                assert_eq!(
                    paint(&colors, ChipVariant::Primary, color),
                    (Some(role.color), role.foreground),
                    "primary×{color:?} must fill with the role and label in its foreground"
                );

                // Secondary keeps the base `--chip-bg: var(--default)`
                // whatever the colour; the colour classes only relabel, to
                // the soft foreground (`--default-foreground` for default).
                assert_eq!(
                    paint(&colors, ChipVariant::Secondary, color),
                    (Some(colors.default.color), muted_foreground),
                    "secondary×{color:?} must keep the default fill and relabel only"
                );

                // `.chip--tertiary` only clears `--chip-bg`; the label comes
                // from the same colour classes as secondary's.
                assert_eq!(
                    paint(&colors, ChipVariant::Tertiary, color),
                    (None, muted_foreground),
                    "tertiary×{color:?} must paint no fill and keep the soft label"
                );

                // `.chip--{color}.chip--soft` fills with `--{color}-soft`,
                // lighter than the role itself, and keeps the soft label.
                assert_eq!(
                    paint(&colors, ChipVariant::Soft, color),
                    (Some(role.soft()), muted_foreground),
                    "soft×{color:?} must fill with the soft mix and keep the soft label"
                );
                assert_ne!(
                    role.soft(),
                    role.color,
                    "the soft fill of {color:?} must not equal the solid fill"
                );
            }
        }

        // The roles are distinct fills: no colour may borrow another's
        // primary background.
        let colors = ThemeColors::light();
        for (a, b) in [
            (Color::Default, Color::Accent),
            (Color::Accent, Color::Success),
            (Color::Success, Color::Warning),
            (Color::Warning, Color::Danger),
        ] {
            assert_ne!(
                colors.role(a.token()).color,
                colors.role(b.token()).color,
                "the {a:?} and {b:?} roles must not share a fill"
            );
        }
    }
}
