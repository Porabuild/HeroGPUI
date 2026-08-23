//! ScrollShadow — port of `@heroui/scroll-shadow` (v3).
//!
//! A scrollable container with soft fading edges. Mirrors the React API:
//! `orientation`, `variant`, `size`, `offset`, `hideScrollBar`, `isEnabled`
//! and `visibility`.

use gpui::{
    div, prelude::*, px, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels,
    RenderOnce, Styled, Window,
};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;

/// The shadow effect style. v3 ships one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollShadowVariant {
    #[default]
    Fade,
}

/// Which edges show a shadow (`visibility`).
///
/// v3 also has `onVisibilityChange`, which fires when the scroll position
/// crosses a shadow threshold. gpui 0.2.2 does not expose a scroll offset to a
/// `RenderOnce` element, so there is nothing truthful to report and the prop is
/// deliberately absent rather than stubbed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollShadowVisibility {
    /// Derived from the scroll position.
    #[default]
    Auto,
    /// Both edges of the scroll axis.
    Both,
    Top,
    Bottom,
    Left,
    Right,
    None,
}

impl ScrollShadowVisibility {
    /// Whether the leading edge (top / left) should be shaded.
    fn shows_start(self, orientation: Orientation) -> bool {
        match self {
            // gpui does not expose the live scroll offset to a `RenderOnce`
            // element, so `Auto` shades both edges — the fade is decorative and
            // reads correctly at rest as well as mid-scroll.
            Self::Auto | Self::Both => true,
            Self::Top => !orientation.is_horizontal(),
            Self::Left => orientation.is_horizontal(),
            _ => false,
        }
    }

    /// Whether the trailing edge (bottom / right) should be shaded.
    fn shows_end(self, orientation: Orientation) -> bool {
        match self {
            Self::Auto | Self::Both => true,
            Self::Bottom => !orientation.is_horizontal(),
            Self::Right => orientation.is_horizontal(),
            _ => false,
        }
    }
}

/// HeroUI ScrollShadow.
#[derive(IntoElement)]
pub struct ScrollShadow {
    id: ElementId,
    orientation: Orientation,
    /// Gradient depth in pixels (`size`, default 40).
    size: Pixels,
    /// Scroll distance before the shadow appears (`offset`).
    offset: Pixels,
    is_enabled: bool,
    visibility: ScrollShadowVisibility,
    max_h: Option<Pixels>,
    max_w: Option<Pixels>,
    gap: Pixels,
    children: Vec<AnyElement>,
}

impl ScrollShadow {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            orientation: Orientation::Vertical,
            size: px(40.),
            offset: px(0.),
            is_enabled: true,
            visibility: ScrollShadowVisibility::Auto,
            max_h: Some(px(240.)),
            max_w: None,
            gap: px(8.),
            children: Vec::new(),
        }
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Gradient depth in pixels.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.offset = offset.into();
        self
    }

    /// Turns shadow rendering off while keeping the scroll behaviour.
    pub fn is_enabled(mut self, v: bool) -> Self {
        self.is_enabled = v;
        self
    }

    pub fn visibility(mut self, visibility: ScrollShadowVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn max_h(mut self, v: impl Into<Pixels>) -> Self {
        self.max_h = Some(v.into());
        self
    }

    pub fn max_w(mut self, v: impl Into<Pixels>) -> Self {
        self.max_w = Some(v.into());
        self
    }

    /// Gap between children inside the scroll area.
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into();
        self
    }
}

impl ParentElement for ScrollShadow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ScrollShadow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let bg = cx.colors().background;
        let horizontal = self.orientation.is_horizontal();

        let mut scroller = div().id(self.id).overflow_hidden().flex().gap(self.gap);

        scroller = if horizontal {
            scroller.flex_row().overflow_x_scroll()
        } else {
            scroller.flex_col().overflow_y_scroll()
        };

        if let Some(h) = self.max_h {
            scroller = scroller.max_h(h);
        }
        if let Some(w) = self.max_w {
            scroller = scroller.max_w(w);
        }

        scroller = scroller.children(self.children);

        if !self.is_enabled || self.visibility == ScrollShadowVisibility::None {
            return div().child(scroller);
        }

        // The fades are absolutely positioned siblings so they do not scroll
        // with the content.
        // Fade to the *same* colour at zero alpha. Interpolating toward
        // `transparent_black` would drag the midpoint through grey.
        let clear = bg.alpha(0.0);
        let fade = |from_start: bool| {
            let stops = if from_start { (bg, clear) } else { (clear, bg) };
            let angle = if horizontal { 90.0 } else { 180.0 };
            let mut el = div().absolute().bg(gpui::linear_gradient(
                angle,
                gpui::linear_color_stop(stops.0, 0.0),
                gpui::linear_color_stop(stops.1, 1.0),
            ));
            el = if horizontal {
                el.top_0().bottom_0().w(self.size)
            } else {
                el.left_0().right_0().h(self.size)
            };
            match (horizontal, from_start) {
                (true, true) => el.left(self.offset),
                (true, false) => el.right(self.offset),
                (false, true) => el.top(self.offset),
                (false, false) => el.bottom(self.offset),
            }
        };

        div()
            .relative()
            .child(scroller)
            .when(self.visibility.shows_start(self.orientation), |el| {
                el.child(fade(true))
            })
            .when(self.visibility.shows_end(self.orientation), |el| {
                el.child(fade(false))
            })
    }
}
