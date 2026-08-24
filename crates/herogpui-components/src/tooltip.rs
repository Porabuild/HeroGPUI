//! Tooltip — port of `@heroui/tooltip` (v3).
//!
//! The tip is state-driven rather than a pure hover style, because v3's `delay`
//! and `closeDelay` need to know *when* the hover began. State lives in a
//! per-tooltip [`Window::use_keyed_state`] entity, so callers still write a
//! plain builder with no entity to thread through.

use std::time::Duration;

use gpui::{
    prelude::*, px, AnyElement, App, ElementId, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use herogpui_theme::ActiveTheme;

use crate::{anim, icons, util};

/// Where the tip sits relative to its trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl TooltipPlacement {
    pub const ALL: [TooltipPlacement; 4] = [
        TooltipPlacement::Top,
        TooltipPlacement::Bottom,
        TooltipPlacement::Left,
        TooltipPlacement::Right,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TooltipPlacement::Top => "Top",
            TooltipPlacement::Bottom => "Bottom",
            TooltipPlacement::Left => "Left",
            TooltipPlacement::Right => "Right",
        }
    }

    /// The arrow points back at the trigger, so it faces opposite the tip.
    fn arrow_rotation(self) -> f32 {
        match self {
            // The asset's apex is at the bottom, so it points down unrotated:
            // a tip above the trigger needs no rotation at all.
            TooltipPlacement::Top => 0.,
            TooltipPlacement::Bottom => std::f32::consts::PI,
            TooltipPlacement::Left => -std::f32::consts::FRAC_PI_2,
            TooltipPlacement::Right => std::f32::consts::FRAC_PI_2,
        }
    }
}

/// Hover state for one tooltip.
///
/// `generation` is bumped on every hover transition; a timer that fires after a
/// newer transition has been recorded is stale and must not flip the tip. That
/// is what keeps a fast pass over a row of triggers from opening all of them.
///
/// `focus_dismissed` is what Escape trips for a *focus-opened* tip. The focus
/// gate (`contains_focused && focus_visible`) is not something Escape may
/// clear — `focus_visible` is app-wide state every focus ring reads — so the
/// dismissal is remembered per tooltip instead, and dropped again the moment
/// the focus leaves the trigger. A dismissal therefore lasts exactly one
/// focus session: the next focus shows the tip again.
pub struct TooltipHover {
    open: bool,
    generation: u64,
    focus_dismissed: bool,
}

impl TooltipHover {
    fn new() -> Self {
        Self {
            open: false,
            generation: 0,
            focus_dismissed: false,
        }
    }

    /// A closed tip, for a caller that needs the same seed the component uses.
    ///
    /// The state lives in `Window::use_keyed_state` under the tooltip's id, and
    /// a test (or any caller that wants to read the flag) has to hand that call
    /// the identical initialiser or it seeds a different slot.
    pub fn closed() -> Self {
        Self::new()
    }

    /// Whether the tip is currently shown.
    pub fn is_open(&self) -> bool {
        self.open
    }
}

/// `trigger` — what reveals the tip.
///
/// v3's default is `hover`, and React Aria shows a hovered tooltip on keyboard
/// focus as well, so `Hover` means "either". `Focus` is the narrower one: the
/// pointer does nothing and only focus opens it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipTrigger {
    #[default]
    Hover,
    Focus,
}

impl TooltipTrigger {
    pub const ALL: [TooltipTrigger; 2] = [TooltipTrigger::Hover, TooltipTrigger::Focus];

    pub fn label(self) -> &'static str {
        match self {
            TooltipTrigger::Hover => "Hover",
            TooltipTrigger::Focus => "Focus",
        }
    }
}

/// HeroUI Tooltip: wraps a trigger and reveals a tip on hover.
#[derive(IntoElement)]
pub struct Tooltip {
    id: Option<ElementId>,
    content: SharedString,
    is_disabled: bool,
    placement: TooltipPlacement,
    show_arrow: bool,
    offset: Option<Pixels>,
    should_skip_animation: bool,
    delay: Option<u64>,
    close_delay: Option<u64>,
    trigger: TooltipTrigger,
    children: Vec<AnyElement>,
}

impl Tooltip {
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            content: content.into(),
            is_disabled: false,
            placement: TooltipPlacement::Top,
            show_arrow: false,
            offset: None,
            should_skip_animation: false,
            delay: None,
            close_delay: None,
            trigger: TooltipTrigger::default(),
            children: Vec::new(),
        }
    }

    /// Distinguishes this tooltip's hover state from its neighbours'.
    ///
    /// The default key is the tip text, which is unique on most pages; set an
    /// id when two tooltips on one screen share the same content.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// `isDisabled` — suppresses the tip entirely.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn placement(mut self, p: TooltipPlacement) -> Self {
        self.placement = p;
        self
    }

    /// `showArrow` — draws the arrow indicator pointing at the trigger.
    pub fn show_arrow(mut self, v: bool) -> Self {
        self.show_arrow = v;
        self
    }

    /// `offset` — distance from the trigger. Defaults to 3px, or 7px with an
    /// arrow, matching v3.
    pub fn offset(mut self, offset: impl Into<Pixels>) -> Self {
        self.offset = Some(offset.into());
        self
    }

    /// `shouldSkipAnimation` — reveal without the entry animation.
    ///
    /// v3 uses this when moving quickly between neighbouring triggers, where
    /// re-animating each tip reads as flicker.
    pub fn should_skip_animation(mut self, v: bool) -> Self {
        self.should_skip_animation = v;
        self
    }

    /// `trigger` — `hover` (the default, which also answers keyboard focus) or
    /// `focus`, which the pointer cannot open.
    pub fn trigger(mut self, trigger: TooltipTrigger) -> Self {
        self.trigger = trigger;
        self
    }

    /// `delay` — milliseconds to wait before showing. Defaults to the
    /// `--tooltip-delay` theme token.
    pub fn delay(mut self, ms: u64) -> Self {
        self.delay = Some(ms);
        self
    }

    /// `closeDelay` — milliseconds to wait before hiding. Defaults to the
    /// `--tooltip-close-delay` theme token.
    pub fn close_delay(mut self, ms: u64) -> Self {
        self.close_delay = Some(ms);
        self
    }
}

impl ParentElement for Tooltip {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Tooltip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.is_disabled {
            // A disabled tooltip renders its trigger and nothing else.
            return gpui::div()
                .flex()
                .children(self.children)
                .into_any_element();
        }

        let key = self
            .id
            .clone()
            .unwrap_or_else(|| ElementId::Name(format!("tooltip-{}", self.content).into()));
        // The state entity has to be created before the theme tokens are read;
        // `use_keyed_state` takes `cx` mutably and would conflict with them.
        let state = window.use_keyed_state(key.clone(), cx, |_, _| TooltipHover::new());
        // React Aria shows a tooltip on keyboard focus as well as on hover --
        // without it a keyboard user never sees one. The handle is a focus
        // *parent*, not a tab stop (tab stops come from `.tab_stop(true)`), so
        // it reports whether the trigger inside it holds the focus without
        // adding a stop of its own.
        let wrap_focus = window.use_keyed_state(
            ElementId::Name(format!("{key:?}-wrap-focus").into()),
            cx,
            |_, cx| cx.focus_handle(),
        );
        let wrap_handle = wrap_focus.read(cx).clone();
        let focus_held = wrap_handle.contains_focused(window, cx);
        // Escape's dismissal is per focus *session*: once the focus leaves the
        // trigger, the latch is dropped, so the next focus is a fresh one and
        // shows the tip again. Clearing here rather than on the next open is
        // what makes a dismissal not permanent without ever touching the
        // app-wide `focus_visible`.
        if !focus_held && state.read(cx).focus_dismissed {
            state.update(cx, |s, _| s.focus_dismissed = false);
        }
        let focus_open = focus_held && util::focus_visible(cx) && !state.read(cx).focus_dismissed;
        // `trigger="focus"` takes the pointer out of it; `hover` is both, which
        // is React Aria's behaviour for the default.
        let open = match self.trigger {
            TooltipTrigger::Hover => state.read(cx).open || focus_open,
            TooltipTrigger::Focus => focus_open,
        };

        let colors = cx.colors();
        let layout = cx.layout();

        let delay = self.delay.unwrap_or(layout.tooltip_delay_ms);
        let close_delay = self.close_delay.unwrap_or(layout.tooltip_close_delay_ms);
        // v3 pushes the tip further out when the arrow needs room.
        let offset = self
            .offset
            .unwrap_or(if self.show_arrow { px(7.) } else { px(3.) });

        let mut tip = gpui::div()
            .absolute()
            // `.tooltip` is `p-2` all round, not a wider-than-tall pill.
            .p(px(8.))
            .rounded(util::small_radius(cx))
            .bg(colors.overlay.background)
            .text_color(colors.overlay.foreground)
            .text_size(px(12.))
            .line_height(px(16.))
            .whitespace_nowrap()
            .shadow(layout.overlay_shadow.clone())
            .child(self.content.to_string());

        tip = match self.placement {
            TooltipPlacement::Top => tip.bottom_full().mb(offset),
            TooltipPlacement::Bottom => tip.top_full().mt(offset),
            TooltipPlacement::Left => tip.right_full().mr(offset),
            TooltipPlacement::Right => tip.left_full().ml(offset),
        };

        if self.show_arrow {
            let mut arrow = gpui::div().absolute().child(
                gpui::svg()
                    .size(px(8.))
                    .path(icons::TOOLTIP_ARROW)
                    // svg() never inherits text colour; the arrow has to be
                    // tinted to match the tip body explicitly.
                    .text_color(colors.foreground)
                    .with_transformation(gpui::Transformation::rotate(gpui::radians(
                        self.placement.arrow_rotation(),
                    ))),
            );
            arrow = match self.placement {
                TooltipPlacement::Top => arrow
                    .top_full()
                    .left(px(0.))
                    .right(px(0.))
                    .flex()
                    .justify_center(),
                TooltipPlacement::Bottom => arrow
                    .bottom_full()
                    .left(px(0.))
                    .right(px(0.))
                    .flex()
                    .justify_center(),
                TooltipPlacement::Left => arrow
                    .left_full()
                    .top(px(0.))
                    .bottom(px(0.))
                    .flex()
                    .items_center(),
                TooltipPlacement::Right => arrow
                    .right_full()
                    .top(px(0.))
                    .bottom(px(0.))
                    .flex()
                    .items_center(),
            };
            tip = tip.child(arrow);
        }

        let hover_state = state.clone();
        let escape_state = state;
        let mut wrapper = gpui::div()
            // `on_hover` needs a stateful element, so the wrapper carries the id.
            .id(key.clone())
            .track_focus(&wrap_handle)
            .relative()
            .flex()
            .children(self.children)
            .on_hover(move |over, _window, cx: &mut App| {
                let over = *over;
                let wait = if over { delay } else { close_delay };
                let generation = hover_state.update(cx, |s, _| {
                    s.generation += 1;
                    s.generation
                });

                if wait == 0 {
                    hover_state.update(cx, |s, cx| {
                        s.open = over;
                        cx.notify();
                    });
                    return;
                }

                let weak = hover_state.downgrade();
                cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                    cx.background_executor()
                        .timer(Duration::from_millis(wait))
                        .await;
                    if let Some(state) = weak.upgrade() {
                        let _ = state.update(cx, |s, cx| {
                            // A newer hover transition supersedes this timer.
                            if s.generation == generation && s.open != over {
                                s.open = over;
                                cx.notify();
                            }
                        });
                    }
                })
                .detach();
            });

        // React Aria hides a tooltip on Escape, which reaches here from the
        // focused trigger inside the wrapper. The hover flag alone is not
        // enough: a `trigger="focus"` tip reads the focus gate and never
        // looks at `open`, so Escape has to trip `focus_dismissed` as well.
        // The latch is per focus session — it is dropped when the focus
        // leaves (see the render gate) — so the next focus shows the tip
        // again, and `focus_visible` is deliberately left untouched.
        wrapper = util::dismiss_on_escape(wrapper, move |_window, cx| {
            escape_state.update(cx, |s, cx| {
                if s.open {
                    s.open = false;
                    cx.notify();
                }
                if !s.focus_dismissed {
                    s.focus_dismissed = true;
                    cx.notify();
                }
            });
        });

        // A tooltip leaves the way every other overlay does: `overlay_phase`
        // keeps it for its exit run, which is what `[data-exiting]` needs to
        // have something to play.
        let phase = util::overlay_phase(
            window,
            cx,
            ElementId::Name(format!("{key:?}-tip-phase").into()),
            open,
        );
        if phase != util::OverlayPhase::Closed {
            // `absolute` does not lift the tip above later siblings in the page,
            // so it has to paint last.
            let zoom = anim::ZoomBox::panel(px(8.), util::small_radius(cx)).padding_x(px(8.));
            let animated = if self.should_skip_animation {
                tip.into_any_element()
            } else if phase == util::OverlayPhase::Exiting {
                anim::exiting(
                    tip,
                    ElementId::Name(format!("{key:?}-tip-out").into()),
                    zoom,
                    anim::Motion::LIST_OUT,
                    cx,
                )
            } else {
                // `tooltip.css` is `duration-150 ease-smooth zoom-in-90` — the
                // same zoom as a popover, not a slide.
                anim::entering_zoom(
                    tip,
                    ElementId::Name(format!("{key:?}-tip").into()),
                    zoom,
                    anim::Motion::POPOVER_IN,
                    cx,
                )
            };
            wrapper = wrapper.child(util::floating(animated));
        }

        wrapper.into_any_element()
    }
}
