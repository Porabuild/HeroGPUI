//! Button — port of `@heroui/button` (v3).
//!
//! v3 replaced v2's `variant` x `color` matrix with a single emphasis scale:
//! `primary | secondary | tertiary | outline | ghost | danger | danger-soft`.
//! There is no `color` or `radius` prop, and `isLoading` became `isPending`.

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, Stateful, Styled, Window,
};
use herogpui_core::{Size, Variant};
use herogpui_theme::ActiveTheme;

use crate::util;

/// A press handler. `Arc` rather than `Box` because it is bound twice: the
/// pointer's `on_click` and the keyboard's Enter/Space both run it.
type OnPress = std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Which edge of a [`crate::button_group::ButtonGroup`] a button sits on.
///
/// `.button-group .button` is `rounded-none`; the first member takes
/// `rounded-s-3xl` and the last `rounded-e-3xl`, so a joined group has one
/// outer radius rather than a rounded box per member. The press scale is also
/// off inside a group (`.button-group .button:active { transform: none }`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroupEdge {
    /// First member: the leading corners are round.
    Start,
    /// Between two others: square on both ends.
    Middle,
    /// Last member: the trailing corners are round.
    End,
    /// The only member, so it keeps the full radius.
    Only,
}

/// HeroUI Button.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    /// v3's `children`-as-a-function: handed `{isHovered, isPressed, isFocused,
    /// isFocusVisible, isDisabled, isPending}` and drawn in place of the label.
    content: Option<std::sync::Arc<dyn Fn(util::InteractiveState) -> AnyElement + 'static>>,
    variant: Variant,
    size: Size,
    full_width: bool,
    is_icon_only: bool,
    /// Set by [`crate::button_group::ButtonGroup`]: which end of the group this
    /// button is, and whether the group stacks.
    group_edge: Option<(GroupEdge, bool)>,
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
            content: None,
            variant: Variant::Primary,
            size: Size::Md,
            full_width: false,
            is_icon_only: false,
            group_edge: None,
            is_disabled: false,
            is_pending: false,
            start_content: None,
            end_content: None,
            children: Vec::new(),
            on_press: None,
        }
    }

    /// v3's render function for a button's children, handed `isHovered`,
    /// `isPressed`, `isFocused`, `isFocusVisible`, `isDisabled` and `isPending`.
    ///
    /// The hover and the press are a frame behind the pointer: gpui reports both
    /// to a handler, so the render that draws them can only read what the last
    /// frame recorded. The button's own hover and press styling does not go
    /// through this -- it is applied by gpui in the same frame.
    pub fn content(
        mut self,
        render: impl Fn(util::InteractiveState) -> AnyElement + 'static,
    ) -> Self {
        self.content = Some(std::sync::Arc::new(render));
        self
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

    /// Joins this button to a group edge. Internal: a caller reaches it by
    /// putting the button in a [`crate::button_group::ButtonGroup`].
    pub(crate) fn group_edge(mut self, edge: GroupEdge, vertical: bool) -> Self {
        self.group_edge = Some((edge, vertical));
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isPending` — blocks presses and hover while retaining the tab stop and focus ring.
    /// A `content` closure receives the pending state and owns any loading indicator.
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
        self.on_press = Some(std::sync::Arc::new(handler));
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
    apply_variant(el, variant, interactive, true, cx)
}

/// The background pair `variant` eases between on hover, or `None` when the
/// variant has no background to ease.
///
/// Used by [`Button`] to run v3's `transition-colors` through
/// [`crate::anim::hover_fade`] instead of swapping the fill on one frame.
pub fn button_hover_colors(variant: Variant, cx: &App) -> Option<(gpui::Hsla, gpui::Hsla)> {
    let colors = cx.colors();
    match variant {
        Variant::Primary => Some((colors.accent.color, colors.accent.hover())),
        Variant::Secondary => Some((colors.default.color, colors.default.hover())),
        Variant::Danger => Some((colors.danger.color, colors.danger.hover())),
        Variant::DangerSoft => Some((colors.danger.soft(), colors.danger.soft_hover())),
        // These three start transparent and fill in on hover.
        Variant::Tertiary | Variant::Outline | Variant::Ghost => {
            Some((gpui::transparent_black(), colors.default.color))
        }
    }
}

/// [`apply_button_variant`], with `hover_bg` off when the caller is going to
/// animate the background itself.
fn apply_variant(
    el: Stateful<Div>,
    variant: Variant,
    interactive: bool,
    hover_bg: bool,
    cx: &App,
) -> Stateful<Div> {
    let colors = cx.colors();
    let layout = cx.layout();

    match variant {
        Variant::Primary => {
            let base = colors.accent;
            let el = el.text_color(base.foreground);
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        // `secondary` is the neutral filled style: v3 maps the removed
        // `bg-secondary` token to `bg-default`.
        Variant::Secondary => {
            let base = colors.default;
            let el = el.text_color(base.foreground);
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
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
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.color)))
                    .active(|s| s.opacity(0.85))
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
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.color)))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Ghost => {
            let base = colors.default;
            let el = el.text_color(colors.muted);
            if interactive {
                el.when(hover_bg, |e| {
                    e.hover(move |s| s.bg(base.color).text_color(colors.foreground))
                })
                .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::Danger => {
            let base = colors.danger;
            let el = el.text_color(base.foreground);
            let el = if hover_bg { el.bg(base.color) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
        Variant::DangerSoft => {
            let base = colors.danger;
            let el = el.text_color(base.soft_foreground());
            let el = if hover_bg { el.bg(base.soft()) } else { el };
            if interactive {
                el.when(hover_bg, |e| e.hover(move |s| s.bg(base.soft_hover())))
                    .active(|s| s.opacity(0.85))
            } else {
                el
            }
        }
    }
}

/// The text colour `variant` paints, for child svgs that cannot inherit
/// `text_color` from their parent.
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

/// [`group_radius`] for any styled element — `ToggleButtonGroup` merges its
/// members' corners the same way `.button-group` does.
pub(crate) fn group_radius_any<T: Styled>(
    el: T,
    edge: Option<(GroupEdge, bool)>,
    radius: gpui::Pixels,
) -> T {
    let Some((edge, vertical)) = edge else {
        return el.rounded(radius);
    };
    match (edge, vertical) {
        (GroupEdge::Only, _) => el.rounded(radius),
        (GroupEdge::Start, false) => el.rounded_tl(radius).rounded_bl(radius),
        (GroupEdge::End, false) => el.rounded_tr(radius).rounded_br(radius),
        (GroupEdge::Start, true) => el.rounded_tl(radius).rounded_tr(radius),
        (GroupEdge::End, true) => el.rounded_bl(radius).rounded_br(radius),
        (GroupEdge::Middle, _) => el,
    }
}

/// Applies `radius` to only the corners a group edge leaves round.
fn group_radius(
    el: Stateful<Div>,
    edge: Option<(GroupEdge, bool)>,
    radius: gpui::Pixels,
) -> Stateful<Div> {
    let Some((edge, vertical)) = edge else {
        return el.rounded(radius);
    };
    match (edge, vertical) {
        (GroupEdge::Only, _) => el.rounded(radius),
        // Horizontal: the start edge rounds its left corners, the end edge its
        // right ones. Vertical: top and bottom.
        (GroupEdge::Start, false) => el.rounded_tl(radius).rounded_bl(radius),
        (GroupEdge::End, false) => el.rounded_tr(radius).rounded_br(radius),
        (GroupEdge::Start, true) => el.rounded_tl(radius).rounded_tr(radius),
        (GroupEdge::End, true) => el.rounded_bl(radius).rounded_br(radius),
        (GroupEdge::Middle, _) => el,
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The handle that says whether this button holds the focus.
        // `use_keyed_state` takes `cx` mutably, so it precedes the tokens.
        let focus_handle = util::tab_stop_handle(
            ElementId::Name(format!("{:?}-focus", self.id).into()),
            window,
            cx,
        );
        // The hover and press this button will report to a `content` closure.
        // Only tracked when one is set: the handlers cost a frame of state.
        let interaction = self.content.as_ref().map(|_| {
            util::interaction(
                ElementId::Name(format!("{:?}-interaction", self.id).into()),
                window,
                cx,
            )
        });
        let layout = cx.layout();
        // Copied out: `hover_fade` below takes `&mut App`, and holding the
        // `layout` borrow across it would be a second borrow of `cx`.
        let disabled_opacity = layout.disabled_opacity;
        let focusable = !self.is_disabled;
        let interactive = focusable && !self.is_pending;
        if !interactive {
            if let Some(slot) = &interaction {
                if *slot.read(cx) != (false, false) {
                    slot.update(cx, |state, _| *state = (false, false));
                }
            }
        }
        // v3's `transition-colors`: the fill eases rather than switching on the
        // frame the pointer arrives. The variant then leaves the background
        // alone so the two do not fight over it.
        let fade = interactive
            .then(|| button_hover_colors(self.variant, cx))
            .flatten();

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
            .map(|e| group_radius(e, self.group_edge, util::control_radius(cx)))
            .text_size(self.size.text_size())
            .line_height(self.size.line_height())
            .h(self.size.control_height());

        el = if self.is_icon_only {
            el.w(self.size.icon_control_size())
        } else {
            el.px(self.size.padding_x()).gap(self.size.gap())
        };

        if self.full_width {
            el = el.w_full();
        }

        el = apply_variant(el, self.variant, interactive, fade.is_none(), cx);

        // The fade's animated layer is glued under everything that follows: the
        // colour transition lives on an inset fill *inside* the button, so the
        // button's own element id — and with it the hover listener latch — never
        // moves when the fill's animation restarts (see `anim::hover_fade`).
        // The interaction slot is handed over when a `content` closure is set:
        // `track_interaction` then owns `on_hover`, and the fade reads the hover
        // bit the slot records instead of binding a second listener.
        if let Some(colors) = fade {
            let edge = self.group_edge;
            let radius = util::control_radius(cx);
            el = crate::anim::hover_fade(
                el,
                ElementId::Name(format!("{:?}-fade", self.id).into()),
                colors,
                interaction.as_ref(),
                move |fill| group_radius_any(fill, edge, radius),
                window,
                cx,
            );
        }

        // `.button:focus-visible` is `status-focused`: a 2px ring, offset from
        // the button by another in the background colour. A disabled button is
        // not a tab stop, which is what `pointer-events-none` amounts to here.
        if focusable {
            el = util::ring_if_focused(
                el.track_focus(&focus_handle),
                &focus_handle,
                true,
                Vec::new(),
                window,
                cx,
            );
        }

        if self.is_disabled || self.is_pending {
            el = el.opacity(disabled_opacity);
        }

        if let Some(start) = self.start_content {
            el = el.child(start);
        }

        if let Some(render) = self.content.clone() {
            let (is_hovered, is_pressed) = if interactive {
                interaction
                    .as_ref()
                    .map(|slot| *slot.read(cx))
                    .unwrap_or_default()
            } else {
                (false, false)
            };
            let focused = focusable && focus_handle.is_focused(window);
            el = el.child(render(util::InteractiveState {
                is_hovered,
                is_pressed,
                is_focused: focused,
                is_focus_visible: focused && util::focus_visible(cx),
                is_selected: false,
                is_disabled: self.is_disabled,
                is_pending: self.is_pending,
                is_indeterminate: false,
            }));
        } else if let Some(label) = self.label {
            el = el.child(label.to_string());
        }
        if interactive {
            if let Some(slot) = &interaction {
                el = util::track_interaction(el, slot);
            }
        }
        el = el.children(self.children);

        if let Some(end) = self.end_content {
            el = el.child(end);
        }

        // v3's `[data-pressed]` scale. Applied last so the press geometry sits
        // on top of whatever the variant did to padding.
        if interactive && self.group_edge.is_none() {
            el = crate::anim::pressed(
                el,
                crate::anim::PressBox {
                    height: self.size.control_height(),
                    padding_x: (!self.is_icon_only).then(|| self.size.padding_x()),
                    width: self.is_icon_only.then(|| self.size.icon_control_size()),
                    // v3's `.button` is `w-fit` with no minimum, so a press has
                    // no floor to scale.
                    min_width: None,
                    text_size: self.size.text_size(),
                    line_height: self.size.line_height(),
                    gap: self.size.gap(),
                    radius: util::control_radius(cx),
                    shrink_x: !self.full_width,
                    scale: crate::anim::PRESSED_SCALE,
                },
                cx,
            );
        }

        if let Some(on_press) = self.on_press {
            if interactive {
                // gpui fires a *focused* element's click listeners on Enter and
                // Space with `ClickEvent::Keyboard`, which is React Aria's press
                // exactly -- so this one binding answers the pointer and the
                // keyboard, and the focus handle above is what switched the
                // second half on.
                el = el.on_click(move |ev: &ClickEvent, window, cx| on_press(ev, window, cx));
            }
        }

        // `fade` was consumed where the fill was added; the button is the
        // plain styled div either way.
        el.into_any_element()
    }
}
