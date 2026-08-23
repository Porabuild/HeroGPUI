//! Spinner — port of `@heroui/spinner` (v3).
//!
//! `size` is `sm | md | lg | xl` and `color` is
//! `current | accent | success | warning | danger`, where `current` inherits
//! the surrounding text color (used inside a pending `Button`).

use std::time::Duration;

use gpui::{prelude::*, px, svg, Animation, AnimationExt, App, IntoElement, RenderOnce, Window};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// Spinner diameter (`size` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinnerSize {
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl SpinnerSize {
    pub const ALL: [SpinnerSize; 4] = [
        SpinnerSize::Sm,
        SpinnerSize::Md,
        SpinnerSize::Lg,
        SpinnerSize::Xl,
    ];

    pub fn px(self) -> gpui::Pixels {
        match self {
            SpinnerSize::Sm => px(16.0),
            SpinnerSize::Md => px(24.0),
            SpinnerSize::Lg => px(32.0),
            SpinnerSize::Xl => px(40.0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SpinnerSize::Sm => "Sm",
            SpinnerSize::Md => "Md",
            SpinnerSize::Lg => "Lg",
            SpinnerSize::Xl => "Xl",
        }
    }
}

impl From<herogpui_core::Size> for SpinnerSize {
    fn from(size: herogpui_core::Size) -> Self {
        match size {
            herogpui_core::Size::Sm => SpinnerSize::Sm,
            herogpui_core::Size::Md => SpinnerSize::Md,
            herogpui_core::Size::Lg => SpinnerSize::Lg,
        }
    }
}

/// A rotating arc spinner, animated on the GPU.
#[derive(IntoElement)]
pub struct Spinner {
    id: gpui::ElementId,
    size: SpinnerSize,
    color: Color,
    /// Set by `color="current"`: the resolved colour of the surrounding text.
    current_color: Option<gpui::Hsla>,
}

impl Spinner {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            size: SpinnerSize::default(),
            color: Color::Accent,
            current_color: None,
        }
    }

    pub fn size(mut self, size: impl Into<SpinnerSize>) -> Self {
        self.size = size.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self.current_color = None;
        self
    }

    /// `color="current"`. gpui svgs do not inherit `text_color`, so the caller
    /// passes the surrounding text colour explicitly.
    pub fn current_color(mut self, color: gpui::Hsla) -> Self {
        self.current_color = Some(color);
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.current_color.unwrap_or(match self.color {
            Color::Default => cx.colors().muted,
            other => cx.role(other).color,
        });

        svg()
            .size(self.size.px())
            .path(icons::SPINNER)
            .text_color(color)
            .with_animation(
                self.id,
                Animation::new(Duration::from_millis(800)).repeat(),
                |svg, delta| {
                    let t = if delta.is_finite() {
                        delta.clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    svg.with_transformation(gpui::Transformation::rotate(gpui::percentage(t)))
                },
            )
    }
}
