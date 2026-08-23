//! Button — port of `@heroui/button` (v3).
//!
//! v3 replaced v2's `variant` x `color` matrix with a single emphasis scale:
//! `primary | secondary | tertiary | outline | ghost | danger | danger-soft`.
//! There is no `color` or `radius` prop, and `isLoading` became `isPending`.

use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, Stateful, Styled, Window,
};
use herogpui_core::{Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::{spinner::Spinner, util};

type OnPress = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Button.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    variant: Variant,
    size: Size,
    full_width: bool,
    is_icon_only: bool,
    is_disabled: bool,
    is_pending: bool,
    /// Rendered before the label — the leading `<Icon />` child in React.
    start_content: Option<AnyElement>,
    /// Rendered after the label.
    end_content: Option<AnyElement>,
    children: Vec<AnyElement>,
    on_press: Option<OnPress>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            variant: Variant::Primary,
            size: Size::Md,
            full_width: false,
            is_icon_only: false,
            is_disabled: false,
            is_pending: false,
            start_content: None,
            end_content: None,
            children: Vec::new(),
            on_press: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn is_icon_only(mut self, v: bool) -> Self {
        self.is_icon_only = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isPending` — swaps the leading content for a spinner and blocks presses.
    pub fn is_pending(mut self, v: bool) -> Self {
        self.is_pending = v;
        self
    }

    pub fn start_content(mut self, content: impl IntoElement) -> Self {
        self.start_content = Some(content.into_any_element());
        self
    }

    pub fn end_content(mut self, content: impl IntoElement) -> Self {
        self.end_content = Some(content.into_any_element());
        self
    }

    pub fn on_press(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_press = Some(Box::new(handler));
        self
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// Paints a button's fill, border, text and interaction states for `variant`.
///
/// Shared with `ButtonGroup`, which propagates the same variant to its members.
pub fn apply_button_variant(
    el: Stateful<Div>,
    variant: Variant,
    interactive: bool,
    cx: &App,
) -> Stateful<Div> {
    let colors = cx.colors();
    let layout = cx.layout();

    match variant {
        Variant::Primary => {
            let base = colors.accent;
            let el = el.bg(base.color).text_color(base.foreground);
            if interactive {
                el.hover(move |s| s.bg(base.hover()))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        // `secondary` is the neutral filled style: v3 maps the removed
        // `bg-secondary` token to `bg-default`.
        Variant::Secondary => {
            let base = colors.default;
            let el = el.bg(base.color).text_color(base.foreground);
            if interactive {
                el.hover(move |s| s.bg(base.hover()))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Tertiary => {
            let base = colors.default;
            let fg = colors.foreground;
            let el = el.text_color(fg);
            if interactive {
                el.hover(move |s| s.bg(base.color)).active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Outline => {
            let base = colors.default;
            let el = el
                .border(layout.border_width)
                .border_color(colors.border)
                .text_color(colors.foreground);
            if interactive {
                el.hover(move |s| s.bg(base.color)).active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Ghost => {
            let base = colors.default;
            let el = el.text_color(colors.muted);
            if interactive {
                el.hover(move |s| s.bg(base.color).text_color(colors.foreground))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Danger => {
            let base = colors.danger;
            let el = el.bg(base.color).text_color(base.foreground);
            if interactive {
                el.hover(move |s| s.bg(base.hover()))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::DangerSoft => {
            let base = colors.danger;
            let el = el.bg(base.soft()).text_color(base.soft_foreground());
            if interactive {
                el.hover(move |s| s.bg(base.soft_hover()))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
    }
}

/// The minimum width a labelled button holds, so a short label still reads as a
/// button. Shared with the press geometry, which has to scale it.
fn min_width(size: Size) -> gpui::Pixels {
    match size {
        Size::Sm => px(64.),
        Size::Md => px(80.),
        Size::Lg => px(96.),
    }
}

/// The text colour `variant` paints. Needed because gpui svgs never inherit
/// `text_color`, so a pending spinner has to be told what "current" means.
pub fn button_foreground(variant: Variant, cx: &App) -> gpui::Hsla {
    let colors = cx.colors();
    match variant {
        Variant::Primary => colors.accent.foreground,
        Variant::Secondary => colors.default.foreground,
        Variant::Tertiary | Variant::Outline => colors.foreground,
        Variant::Ghost => colors.muted,
        Variant::Danger => colors.danger.foreground,
        Variant::DangerSoft => colors.danger.soft_foreground(),
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let layout = cx.layout();
        let interactive = !self.is_disabled && !self.is_pending;

        let mut el = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .font_weight(gpui::FontWeight::MEDIUM)
            .rounded(util::control_radius(cx))
            .text_size(self.size.text_size())
            .line_height(self.size.line_height())
            .h(self.size.control_height());

        el = if self.is_icon_only {
            el.w(self.size.icon_control_size())
        } else {
            el.px(self.size.padding_x())
                .gap(self.size.gap())
                .min_w(min_width(self.size))
        };

        if self.full_width {
            el = el.w_full();
        }

        el = apply_button_variant(el, self.variant, interactive, cx);

        if self.is_disabled {
            el = el.opacity(layout.disabled_opacity);
        }

        // `isPending` replaces the leading icon with a spinner, matching the
        // documented render-prop pattern.
        if self.is_pending {
            let spinner_id = ElementId::Name(format!("{:?}-spinner", self.id).into());
            el = el.child(
                Spinner::new(spinner_id)
                    .current_color(button_foreground(self.variant, cx)),
            );
        } else if let Some(start) = self.start_content {
            el = el.child(start);
        }

        if let Some(label) = self.label {
            el = el.child(label.to_string());
        }
        el = el.children(self.children);

        if let Some(end) = self.end_content {
            el = el.child(end);
        }

        // v3's `[data-pressed]` scale. Applied last so the press geometry sits
        // on top of whatever the variant did to padding.
        if interactive {
            el = crate::anim::pressed(
                el,
                crate::anim::PressBox {
                    height: self.size.control_height(),
                    padding_x: (!self.is_icon_only).then(|| self.size.padding_x()),
                    width: self.is_icon_only.then(|| self.size.icon_control_size()),
                    min_width: (!self.is_icon_only).then(|| min_width(self.size)),
                    text_size: self.size.text_size(),
                    line_height: self.size.line_height(),
                    gap: self.size.gap(),
                    radius: util::control_radius(cx),
                    shrink_x: !self.full_width,
                },
                cx,
            );
        }

        if let Some(on_press) = self.on_press {
            if interactive {
                el = el.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            }
        }

        el
    }
}
