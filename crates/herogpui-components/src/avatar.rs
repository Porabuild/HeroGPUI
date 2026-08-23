//! Avatar & AvatarGroup — port of `@heroui/avatar`.

use gpui::{
    prelude::*, px, App, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};
use herogpui_core::{Color};
use herogpui_theme::ActiveTheme;


/// Visual style of an avatar fallback (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarVariant {
    /// Solid fill in the avatar color.
    #[default]
    Default,
    /// The color at 15% with colored initials.
    Soft,
}

impl AvatarVariant {
    pub const ALL: [AvatarVariant; 2] = [AvatarVariant::Default, AvatarVariant::Soft];

    pub fn label(self) -> &'static str {
        match self {
            AvatarVariant::Default => "Default",
            AvatarVariant::Soft => "Soft",
        }
    }
}

/// HeroUI Avatar: image or name-initials fallback.
#[derive(IntoElement)]
pub struct Avatar {
    name: SharedString,
    src: Option<SharedString>,
    /// Edge length, set by [`Avatar::size`]. v3 has no custom-pixel prop.
    size_px: gpui::Pixels,
    color: Color,
    variant: AvatarVariant,
    /// Set by [`AvatarGroup`], which rings each member so the stack reads as
    /// separate avatars. v3 does this in the group's CSS, not with a prop.
    is_bordered: bool,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            name: "".into(),
            src: None,
            size_px: px(40.),
            color: Color::Default,
            variant: AvatarVariant::Default,
            is_bordered: false,
        }
    }

    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = name.into();
        self
    }

    pub fn variant(mut self, variant: AvatarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Asset path for the avatar image (`src`).
    pub fn src(mut self, src: impl Into<SharedString>) -> Self {
        self.src = Some(src.into());
        self
    }

    pub fn size(mut self, size: herogpui_core::Size) -> Self {
        self.size_px = match size {
            herogpui_core::Size::Sm => px(32.),
            herogpui_core::Size::Md => px(40.),
            herogpui_core::Size::Lg => px(48.),
        };
        self
    }



    fn with_border(mut self, v: bool) -> Self {
        self.is_bordered = v;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let mut out = String::new();
    for w in words.iter().take(2) {
        if let Some(c) = w.chars().next() {
            out.extend(c.to_uppercase());
        }
    }
    if out.is_empty() {
        "?".to_string()
    } else {
        out
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let neutral = self.color == Color::Default;
        let (bg, fg) = match self.variant {
            AvatarVariant::Default if neutral => {
                (cx.colors().surface_tertiary, cx.colors().foreground)
            }
            AvatarVariant::Default => (sem.color, sem.foreground),
            AvatarVariant::Soft if neutral => (cx.colors().default.soft(), cx.colors().muted),
            AvatarVariant::Soft => (sem.soft(), sem.soft_foreground()),
        };
        let font = self.size_px * 0.375;

        let mut el = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size_px)
            .rounded(crate::util::control_radius(cx))
            .bg(bg)
            .text_color(fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::MEDIUM)
            .overflow_hidden()
            .flex_shrink_0();

        if self.is_bordered {
            el = el.border_2().border_color(cx.colors().background);
        }

        match &self.src {
            Some(path) => {
                el = el.child(gpui::img(path.clone()).size_full());
            }
            None => {
                el = el.child(initials(&self.name));
            }
        }

        el
    }
}

/// Stacked avatar group with overflow counter (`AvatarGroup`).
#[derive(IntoElement)]
pub struct AvatarGroup {
    avatars: Vec<Avatar>,
    max: usize,
    total: Option<usize>,
}

impl AvatarGroup {
    pub fn new(avatars: Vec<Avatar>) -> Self {
        Self {
            avatars,
            max: 3,
            total: None,
        }
    }

    /// Maximum visible avatars before the "+N" chip.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max.max(1);
        self
    }

    /// Overrides the total count used in the "+N" chip.
    pub fn total(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.avatars.len();
        let visible: Vec<Avatar> = self.avatars.into_iter().take(self.max).collect();

        gpui::div()
            .flex()
            .items_center()
            .child(
                gpui::div()
                    .flex()
                    .children(visible.into_iter().enumerate().map(|(i, a)| {
                        gpui::div()
                            .when(i > 0, |d| d.ml(gpui::px(-8.)))
                            .rounded_full()
                            .child(a.with_border(true))
                    })),
            )
            .when(count > self.max || self.total.map(|t| t > self.max).unwrap_or(false), |el| {
                let shown_total = self.total.unwrap_or(count);
                el.child(
                    gpui::div()
                        .ml(px(-8.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(40.))
                        .h(px(40.))
                        .rounded_full()
                        .bg(cx.colors().default.soft())
                        .text_color(cx.colors().muted)
                        .text_size(px(12.))
                        .border_2()
                        .border_color(cx.colors().background)
                        .child(format!("+{}", shown_total - self.max)),
                )
            })
    }
}

