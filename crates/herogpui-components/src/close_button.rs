//! CloseButton — port of `@heroui/close-button`.
//!
//! A button for dismissing dialogs, modals and inline content. Mirrors the
//! React API: `variant`, `isDisabled`, `onPress` and a custom-icon slot that
//! replaces the default close glyph.

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, Styled, Window,
};
use herogpui_theme::ActiveTheme;

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
}

impl CloseButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_disabled: false,
            icon: None,
            content: None,
            on_press: None,
        }
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
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
        let focus_handle = crate::util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        let interaction = self.content.as_ref().map(|_| {
            crate::util::interaction(
                ElementId::Name(format!("{:?}-interaction", self.id).into()),
                window,
                cx,
            )
        });
        if self.is_disabled {
            if let Some(slot) = &interaction {
                if *slot.read(cx) != (false, false) {
                    slot.update(cx, |state, _| *state = (false, false));
                }
            }
        }

        let colors = cx.colors();
        let layout = cx.layout();
        // `.close-button` is `h-6 p-1` with a `size-4` glyph.
        let (box_size, icon_size) = (px(24.), px(16.));
        let hover_bg = colors.default.hover();

        let mut el = div()
            .id(self.id.clone())
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
            .rounded(crate::util::small_radius(cx))
            .bg(colors.default.color)
            .text_color(colors.muted);

        if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        } else {
            el = el.cursor_pointer().hover(move |s| s.bg(hover_bg));
            // `.close-button--default:active, &[data-pressed="true"]` is
            // `transform: scale(0.93)`. gpui 0.2.2 has no div-level scale, so
            // the press shrinks the 24px box about its centre and the leftover
            // becomes margin — the same geometry `anim::pressed` uses.
            // `.active` is an instant style swap, matching
            // `motion-reduce:transition-none` while preserving the transform.
            const PRESS_SCALE: f32 = 0.93;
            let inset = px(f32::from(box_size) * (1.0 - PRESS_SCALE) / 2.0);
            let pressed = px(f32::from(box_size) * PRESS_SCALE);
            let radius = px(f32::from(crate::util::small_radius(cx)) * PRESS_SCALE);
            el = el.active(move |s| {
                s.h(pressed)
                    .w(pressed)
                    .mt(inset)
                    .mb(inset)
                    .ml(inset)
                    .mr(inset)
                    .rounded(radius)
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
            return el;
        }
        crate::util::ring_if_focused(
            el.track_focus(&focus_handle),
            &focus_handle,
            true,
            Vec::new(),
            window,
            cx,
        )
    }
}
