//! CloseButton — port of `@heroui/close-button`.
//!
//! A button for dismissing dialogs, modals and inline content. Mirrors the
//! React API: `variant`, `isDisabled`, `onPress` and a custom-icon slot that
//! replaces the default close glyph.

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Styled, Window,
};
use herogpui_core::element_id;
use herogpui_theme::ActiveTheme;

use crate::a11y::{self, A11y as _};
use crate::icons;

/// Visual variant of a close button. React exposes a single `default` variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloseButtonVariant {
    #[default]
    Default,
}

/// A press handler. `Arc` rather than `Box` because it is bound twice: the
/// pointer's `on_click` and the keyboard's Enter/Space both run it.
type OnPress = std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI CloseButton.
#[derive(IntoElement)]
pub struct CloseButton {
    id: ElementId,
    is_disabled: bool,
    /// Replaces the default close glyph (`children` in React).
    icon: Option<AnyElement>,
    /// v3's `children`-as-a-function: handed the interactive state and drawn in
    /// place of the default content.
    content: Option<std::sync::Arc<dyn Fn(crate::util::InteractiveState) -> AnyElement + 'static>>,

    on_press: Option<OnPress>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
    /// Set by [`CloseButton::hover_bg`]: the fill the hover fade eases *to*.
    /// Additive — unset, the fade behaves exactly as it did before.
    hover_bg: Option<gpui::Hsla>,
    /// The corner radius, in place of the owning `small_radius` helper. The
    /// pressed box scales it along with the box.
    radius: Option<Pixels>,
}

impl CloseButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_disabled: false,
            icon: None,
            content: None,
            on_press: None,
            sx: None,
            hover_bg: None,
            radius: None,
        }
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the close button's root element after every value the active
    /// theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
        self
    }

    /// The fill the hover fade eases to, in place of `--default-hover`.
    ///
    /// The fade runs from the resting background — the `sx` background when one
    /// is set, the close button's own `--default` otherwise — to `color`, over
    /// the same `transition-colors` timing the stock button uses. v3 has no
    /// such prop; on the web this is `className="hover:bg-…"`.
    pub fn hover_bg(mut self, color: impl Into<gpui::Hsla>) -> Self {
        self.hover_bg = Some(color.into());
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// The corner radius, in place of the owning `small_radius` helper. The
    /// box is the button's whole shape, so the pressed box scales the same
    /// value rather than snapping back to the helper. Not a v3 prop; the
    /// removed v2 `radius` prop is prohibited and this is a per-component
    /// repository extension.
    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// Supplies a custom icon in place of the default close glyph.
    /// v3's render function for the button's children, handed `isHovered`,
    /// `isPressed`, `isFocused` and `isDisabled`. The hover and press are a
    /// frame behind the pointer, because gpui reports both to a handler.
    pub fn content(
        mut self,
        render: impl Fn(crate::util::InteractiveState) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn on_press(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for CloseButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.close-button:focus-visible` is `status-focused`. The handle has to be
        // read before the theme tokens: `use_keyed_state` takes `cx` mutably.
        let focus_handle =
            crate::util::tab_stop_handle(element_id::scoped(&self.id, "focus"), window, cx);
        let interaction = self.content.as_ref().map(|_| {
            crate::util::interaction(element_id::scoped(&self.id, "interaction"), window, cx)
        });
        if self.is_disabled {
            if let Some(slot) = &interaction {
                if *slot.read(cx) != (false, false) {
                    slot.update(cx, |state, _| *state = (false, false));
                }
            }
        }

        let colors = cx.colors().clone();
        // The box is the button's whole shape, so the press-scale derivation
        // below multiplies this resolved value rather than the helper's.
        let radius = self.radius.unwrap_or_else(|| crate::util::small_radius(cx));
        let sx_corners = crate::util::sx_radius(&self.sx);
        let disabled_opacity = cx.layout().disabled_opacity;
        // `.close-button` is `h-6 p-1` with a `size-4` glyph.
        let (box_size, icon_size) = (px(24.), px(16.));
        // The `sx` slot refines the root, so its background is the resting
        // value the fade must hold; an explicit `hover_bg` eases from it.
        let sx_background = crate::util::sx_background(&self.sx);
        let idle_bg = sx_background.unwrap_or(colors.default.color);
        let hover_end = self.hover_bg.unwrap_or(colors.default.hover());
        let fade = (!self.is_disabled)
            .then_some((colors.default.color, colors.default.hover()))
            .and_then(|pair| crate::util::fade_endpoints(Some(pair), sx_background, self.hover_bg));

        let mut el = div()
            .id(self.id.clone())
            // `close-button.js` hard-codes `aria-label="Close"` on the RAC
            // `Button`, because the default child is an icon with no text.
            .a11y_named(a11y::Role::Button, &a11y::Name::labelled("Close"))
            .debug_selector({
                let id = self.id.clone();
                move || format!("{id:?}")
            })
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .size(box_size)
            .p(px(4.))
            .rounded(radius)
            .map(|el| crate::util::round_sx_corners(el, &sx_corners))
            .when(fade.is_none(), |e| e.bg(idle_bg))
            .text_color(colors.muted);

        if let Some(fade_colors) = fade {
            el = crate::anim::hover_fade(
                el,
                element_id::scoped(&self.id, "fade"),
                fade_colors,
                interaction.as_ref(),
                move |fill| crate::util::round_sx_corners(fill.rounded(radius), &sx_corners),
                window,
                cx,
            );
        }

        if self.is_disabled {
            el = el.opacity(disabled_opacity);
        } else {
            el = crate::util::cursor_interactive(el, cx);
            if fade.is_none() {
                el = el.hover(move |s| s.bg(hover_end));
            }
            // `.close-button--default:active, &[data-pressed="true"]` is
            // `transform: scale(0.93)`. gpui 0.2.2 has no div-level scale, so
            // the press shrinks the 24px box about its centre and the leftover
            // becomes margin — the same geometry `anim::pressed` uses.
            // `.active` is an instant style swap, matching
            // `motion-reduce:transition-none` while preserving the transform.
            const PRESS_SCALE: f32 = 0.93;
            let inset = px(f32::from(box_size) * (1.0 - PRESS_SCALE) / 2.0);
            let pressed = px(f32::from(box_size) * PRESS_SCALE);
            let pressed_radius = px(f32::from(radius) * PRESS_SCALE);
            let pressed_corners =
                crate::anim::pressed_corners(&el.style().corner_radii, radius, PRESS_SCALE);
            el = el.active(move |s| {
                crate::util::round_sx_corners(
                    s.h(pressed)
                        .w(pressed)
                        .mt(inset)
                        .mb(inset)
                        .ml(inset)
                        .mr(inset)
                        .rounded(pressed_radius),
                    &pressed_corners,
                )
            });
        }

        el = match (self.content.clone(), self.icon) {
            (Some(render), _) => {
                let (is_hovered, is_pressed) = interaction
                    .as_ref()
                    .map(|slot| *slot.read(cx))
                    .unwrap_or_default();
                let is_focused = !self.is_disabled && focus_handle.is_focused(window);
                el.child(render(crate::util::InteractiveState {
                    is_hovered,
                    is_pressed,
                    is_focused,
                    is_focus_visible: is_focused && crate::util::focus_visible(cx),
                    is_selected: false,
                    is_disabled: self.is_disabled,
                    is_pending: false,
                    is_indeterminate: false,
                }))
            }
            (None, Some(icon)) => el.child(icon),
            (None, None) => el.child(
                gpui::svg()
                    .debug_selector({
                        let id = self.id.clone();
                        move || format!("{id:?}-icon")
                    })
                    .size(icon_size)
                    .flex_shrink_0()
                    // `.close-button svg` is `-mx-0.5 my-0.5` (2px at the 16px
                    // root). Symmetric margins on a centred 16px child in the
                    // 16px `p-1` content box cancel, but they are representable
                    // on `svg()` through Styled and belong on the default glyph.
                    .mx(px(-2.))
                    .my(px(2.))
                    .path(icons::CLOSE)
                    .text_color(colors.muted),
            ),
        };
        if !self.is_disabled {
            if let Some(slot) = &interaction {
                el = crate::util::track_interaction(el, slot);
            }
        }

        if let Some(on_press) = self.on_press {
            if !self.is_disabled {
                el = el.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            }
        }

        if self.is_disabled {
            let root = div()
                .size(box_size)
                .flex()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .child(el);
            return crate::util::apply_sx(root, &self.sx);
        }
        let el = crate::util::ring_if_focused(
            el.track_focus(&focus_handle),
            &focus_handle,
            true,
            Vec::new(),
            window,
            cx,
        );
        let root = div()
            .size(box_size)
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .child(el);
        crate::util::apply_sx(root, &self.sx)
    }
}
