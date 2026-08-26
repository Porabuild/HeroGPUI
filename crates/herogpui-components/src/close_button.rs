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
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .active(|s| s.opacity(0.7));
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
                    is_indeterminate: false,
                }))
            }
            (None, Some(icon)) => el.child(icon),
            (None, None) => el.child(
                gpui::svg()
                    .size(icon_size)
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
