//! Spinner — port of `@heroui/spinner` (v3).
//!
//! `size` is `sm | md | lg | xl` and `color` is
//! `current | accent | success | warning | danger`, where `current` inherits
//! the surrounding text color (used inside a pending `Button`).

use std::time::Duration;

use gpui::{prelude::*, px, svg, Animation, AnimationExt, App, IntoElement, RenderOnce, Window};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::a11y::A11y as _;
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
    /// One full turn, in milliseconds. v3 changes this with an animation
    /// utility class (`animate-[spin_1.5s_linear_infinite]`), which is its
    /// "Speed" example; there are no classes here, so it is a prop.
    duration_ms: u64,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
}

impl Spinner {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            size: SpinnerSize::default(),
            color: Color::Accent,
            current_color: None,
            duration_ms: 800,
            sx: None,
        }
    }

    /// How long one full turn takes. v3 sets it with an animation utility.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms.max(1);
        self
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

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the spinner's root element after every value the size, the
    /// colour and the active theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.current_color.unwrap_or(match self.color {
            Color::Default => cx.colors().muted,
            other => cx.role(other).color,
        });

        let spinner = svg()
            .size(self.size.px())
            .flex_shrink_0()
            .path(icons::SPINNER)
            .text_color(color);
        let glyph = if ActiveTheme::reduce_motion(cx) {
            crate::util::apply_sx(spinner, &self.sx).into_any_element()
        } else {
            // `with_animation` hands back an `AnimationElement`, which has no
            // style of its own to refine, so the slot lands on the svg the
            // rotation wraps.
            crate::util::apply_sx(spinner, &self.sx)
                .with_animation(
                    self.id.clone(),
                    Animation::new(Duration::from_millis(self.duration_ms)).repeat(),
                    |svg, delta| {
                        let t = if delta.is_finite() {
                            delta.clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        svg.with_transformation(gpui::Transformation::rotate(gpui::percentage(t)))
                    },
                )
                .into_any_element()
        };

        // The node sits on a box *around* the glyph rather than on the glyph
        // itself, which is upstream's own anatomy:
        // `@heroui/react/dist/components/spinner/spinner.js` imports no
        // `react-aria-components` primitive at all and hard-codes
        // `role: "status"` with `"aria-label": "Loading"` on its `dom.span`
        // root, giving the `SpinnerPrimitive` svg inside it `aria-hidden: true`.
        // (`Spinner` is a separate v3 export from `ProgressCircle`, which goes
        // through `useProgressBar`; the two are not the same component.)
        //
        // It also has to be a separate element here: the rotation is applied
        // by `Svg::with_transformation`, an inherent method on `Svg` that a
        // `Stateful<Svg>` no longer exposes, so an id on the glyph and the
        // animation cannot both survive. The `aria-hidden` half needs no
        // builder — the glyph has no id, and an element with no id and no role
        // produces no AccessKit node at all (`gpui-pre-0.3.3`'s
        // `window/a11y.rs`), which is the stronger form of the same thing.
        //
        // Stated after the layout chain, not spliced into it:
        // `.shots/design_audit.py` reads sizes and gaps out of builder chains
        // with character-windowed regexes.
        let root = gpui::div().flex().flex_shrink_0().child(glyph);
        root.id(self.id).a11y_named(
            crate::a11y::Role::Status,
            &crate::a11y::Name::labelled("Loading"),
        )
    }
}
