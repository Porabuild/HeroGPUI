//! Tooltip — port of `@heroui/tooltip` (v3).
//!
//! The tip is state-driven rather than a pure hover style, because v3's `delay`
//! and `closeDelay` need to know *when* the hover began. State lives in a
//! per-tooltip [`Window::use_keyed_state`] entity, so callers still write a
//! plain builder with no entity to thread through.
//!
//! By default the floating tip renders inside the owning GPUI window. Callers
//! whose owner window is too compact for the tip (for example toolbar palettes
//! or floating HUDs) can supply a [`TooltipPresenter`] or choose
//! [`TooltipPresenterMode::SeparateWindow`] while retaining HeroGPUI's hover,
//! focus, delay, warmup, cooldown, Escape, and unmount dismissal lifecycle.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use gpui::{
    point, prelude::*, px, size, AnyElement, AnyWindowHandle, App, Bounds, DisplayId, Element,
    ElementId, Entity, GlobalElementId, InspectorElementId, IntoElement, LayoutId, ParentElement,
    Pixels, Point, Render, RenderOnce, SharedString, Size, StatefulInteractiveElement, Styled,
    Window, WindowBackgroundAppearance, WindowBounds, WindowId, WindowKind, WindowOptions,
};
use herogpui_core::{element_id, PlacementAlign};
use herogpui_theme::ActiveTheme;

use crate::{
    a11y::{self, A11y as _},
    anim, icons, popover, util,
};

/// Overhang space reserved around `SeparateTooltipView` on the trigger-facing
/// side when `show_arrow` is enabled (`ARROW_SIZE` matches `.size(px(12.))`).
const TOOLTIP_ARROW_SIZE: Pixels = px(12.);

/// HeroUI's default tooltip width cap (`max-w-xs` = 320px).
const TOOLTIP_MAX_WIDTH: Pixels = px(320.);

/// Minimum inset kept between an in-window tooltip and the window viewport edges.
const TOOLTIP_VIEWPORT_MARGIN: Pixels = px(8.);

/// Deferred draw priority for in-window tooltips, lifted above `util::floating`
/// (priority `0`) so tooltips triggered inside popovers or menus paint on top.
const TOOLTIP_DEFERRED_PRIORITY: usize = 10;

/// Where the tip sits relative to its trigger.
///
/// Shares the one placement vocabulary with the popovers and pickers — the
/// full 22-value React Aria union. The logical `start`/`end` aliases resolve
/// to the same pixels as their `left`/`right` spellings because this port has
/// no RTL mode.
pub use herogpui_core::Placement as TooltipPlacement;

/// Controls which surface presents an open [`Tooltip`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPresenterMode {
    /// Renders the tooltip inside the owning GPUI window (default).
    #[default]
    InWindow,
    /// Presents the tooltip in a separate non-activating GPUI popup window
    /// (`WindowKind::PopUp`), or delegates to a custom [`TooltipPresenter`]
    /// when one is supplied via [`Tooltip::presenter`].
    ///
    /// If opening the popup window fails (for example on backends without
    /// secondary popup windows), HeroGPUI falls back to [`Self::InWindow`] for
    /// that open session so the tooltip still displays.
    SeparateWindow,
}

/// Inputs delivered to a [`TooltipPresenter`] when showing or refreshing a
/// tooltip outside the owning window's layout tree.
#[derive(Clone)]
#[non_exhaustive]
pub struct TooltipPresentationRequest {
    /// Stable tooltip element key, matching the trigger's keyed hover state.
    pub key: ElementId,
    /// Accessible tooltip label (and default text when `body` is `None`).
    pub content: SharedString,
    /// Trigger bounds in owning-window logical coordinates.
    pub trigger_bounds: Bounds<Pixels>,
    /// Trigger bounds translated by `window_bounds.origin` in the active
    /// platform backend's coordinate space.
    ///
    /// Note: GPUI reports `window_bounds` and `display_bounds` in global
    /// desktop coordinates on Windows/X11, whereas on macOS AppKit `window_bounds`
    /// are display-relative with `display_bounds.origin == (0, 0)`. Pair with
    /// [`Self::display_id`] when positioning native macOS surfaces across
    /// multiple monitors.
    pub trigger_screen_bounds: Bounds<Pixels>,
    /// Owning window bounds in the platform backend's logical coordinates (see
    /// [`Self::trigger_screen_bounds`] for macOS display-relative semantics).
    pub window_bounds: Bounds<Pixels>,
    /// Owning display identifier, if known (needed on macOS to map
    /// display-relative coordinates onto a specific monitor).
    pub display_id: Option<DisplayId>,
    /// Active display visible work-area bounds in the platform backend's logical
    /// coordinates, if known (`origin` excludes taskbar/dock/menu bar where
    /// provided by GPUI; on macOS AppKit coordinates are display-relative).
    pub display_bounds: Option<Bounds<Pixels>>,
    /// Owning window display scale factor (`1.0`, `1.25`, `1.5`, `2.0`, ...).
    pub scale_factor: f32,
    /// Preferred placement relative to the trigger.
    pub placement: TooltipPlacement,
    /// Distance in logical pixels between the trigger and the tooltip surface.
    pub offset: Pixels,
    /// Whether the tip requests an arrow indicator pointing at the trigger.
    pub show_arrow: bool,
    /// Corner radius resolved for the tip surface.
    pub radius: Pixels,
    /// Uniform inner padding resolved for the tip surface.
    pub padding: Pixels,
    /// Text size resolved for the tip surface.
    pub text_size: Pixels,
    /// Optional surface background override resolved from [`herogpui_theme::TooltipStyle`].
    pub background: Option<gpui::Hsla>,
    /// Optional text foreground override resolved from [`herogpui_theme::TooltipStyle`].
    pub foreground: Option<gpui::Hsla>,
    /// Whether the presenter is asked to exclude the popup surface from screen
    /// capture and recordings.
    ///
    /// Native OS presenters (such as AppKit sharing-type exclusion or Win32
    /// `SetWindowDisplayAffinity`) enforce this at the platform window level.
    /// `gpui-pre-0.3.11`'s cross-platform `WindowOptions` has no capture
    /// exclusion flag, so [`TooltipPresenterMode::SeparateWindow`] carries it
    /// as advisory metadata only.
    pub exclude_from_capture: bool,
    /// Measured intrinsic size of the rendered tip surface (both default text
    /// and custom [`Tooltip::body`]).
    pub tip_size: Option<Size<Pixels>>,
    /// Custom tip body factory supplied via [`Tooltip::body`], if any.
    #[allow(clippy::type_complexity)]
    pub body: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
}

/// Boundary for presenting [`Tooltip`] surfaces outside the owning window.
///
/// HeroGPUI continues to own hover/focus triggers, open/close delays, global
/// warmup and cooldown, stale-timer `generation` checks, Escape and press
/// dismissal, and unmount cleanup. Implementations only show, reposition, and
/// close the surface for a given `key`.
pub trait TooltipPresenter: 'static {
    /// Shows, updates, or repositions the tooltip described by `request`.
    fn present(&self, request: &TooltipPresentationRequest, window: &mut Window, cx: &mut App);

    /// Dismisses any surface currently presented for `key`.
    ///
    /// `window` is `Some` when dismissal runs from a live owning window frame
    /// or event, and `None` if cleanup fired after the owning window closed.
    fn dismiss(&self, key: &ElementId, window: Option<&mut Window>, cx: &mut App);
}

fn flip_tooltip_placement(placement: TooltipPlacement) -> TooltipPlacement {
    match placement {
        TooltipPlacement::Top => TooltipPlacement::Bottom,
        TooltipPlacement::TopStart => TooltipPlacement::BottomStart,
        TooltipPlacement::TopLeft => TooltipPlacement::BottomLeft,
        TooltipPlacement::TopEnd => TooltipPlacement::BottomEnd,
        TooltipPlacement::TopRight => TooltipPlacement::BottomRight,
        TooltipPlacement::Bottom => TooltipPlacement::Top,
        TooltipPlacement::BottomStart => TooltipPlacement::TopStart,
        TooltipPlacement::BottomLeft => TooltipPlacement::TopLeft,
        TooltipPlacement::BottomEnd => TooltipPlacement::TopEnd,
        TooltipPlacement::BottomRight => TooltipPlacement::TopRight,
        TooltipPlacement::Left => TooltipPlacement::Right,
        TooltipPlacement::LeftTop => TooltipPlacement::RightTop,
        TooltipPlacement::LeftBottom => TooltipPlacement::RightBottom,
        TooltipPlacement::Start => TooltipPlacement::End,
        TooltipPlacement::StartTop => TooltipPlacement::EndTop,
        TooltipPlacement::StartBottom => TooltipPlacement::EndBottom,
        TooltipPlacement::Right => TooltipPlacement::Left,
        TooltipPlacement::RightTop => TooltipPlacement::LeftTop,
        TooltipPlacement::RightBottom => TooltipPlacement::LeftBottom,
        TooltipPlacement::End => TooltipPlacement::Start,
        TooltipPlacement::EndTop => TooltipPlacement::StartTop,
        TooltipPlacement::EndBottom => TooltipPlacement::StartBottom,
    }
}

fn snap_axis_inside(val: f32, min_bound: f32, max_bound: f32, scale_factor: f32) -> f32 {
    let clamped = val.clamp(min_bound, max_bound);
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return clamped;
    }
    let mut min_px = (min_bound * scale_factor).ceil();
    let mut max_px = (max_bound * scale_factor).floor();
    if min_px / scale_factor < min_bound {
        min_px += 1.0;
    }
    if max_px / scale_factor > max_bound {
        max_px -= 1.0;
    }
    if min_px <= max_px {
        ((clamped * scale_factor).round().clamp(min_px, max_px)) / scale_factor
    } else {
        clamped
    }
}

fn resolve_tooltip_popup_placement(
    trigger_screen_bounds: Bounds<Pixels>,
    popup_size: Size<Pixels>,
    monitor_bounds: Bounds<Pixels>,
    placement: TooltipPlacement,
    offset: Pixels,
    scale_factor: f32,
) -> (Point<Pixels>, TooltipPlacement) {
    let trig_x = f32::from(trigger_screen_bounds.origin.x);
    let trig_y = f32::from(trigger_screen_bounds.origin.y);
    let trig_w = f32::from(trigger_screen_bounds.size.width).max(0.0);
    let trig_h = f32::from(trigger_screen_bounds.size.height).max(0.0);
    let trig_right = trig_x + trig_w;
    let trig_bottom = trig_y + trig_h;

    let pop_w = f32::from(popup_size.width).max(1.0);
    let pop_h = f32::from(popup_size.height).max(1.0);
    let off = f32::from(offset);

    let mon_left = f32::from(monitor_bounds.origin.x);
    let mon_top = f32::from(monitor_bounds.origin.y);
    let mon_right = mon_left + f32::from(monitor_bounds.size.width).max(0.0);
    let mon_bottom = mon_top + f32::from(monitor_bounds.size.height).max(0.0);

    let (x, y, flipped) = if placement.is_side() {
        let left_x = trig_x - off - pop_w;
        let right_x = trig_right + off;
        let (chosen_x, flipped) = if placement.is_start_side() {
            if left_x < mon_left && right_x + pop_w <= mon_right {
                (right_x, true)
            } else {
                (left_x, false)
            }
        } else if right_x + pop_w > mon_right && left_x >= mon_left {
            (left_x, true)
        } else {
            (right_x, false)
        };
        let chosen_y = match placement.align() {
            PlacementAlign::Start => trig_y,
            PlacementAlign::End => trig_bottom - pop_h,
            PlacementAlign::Center => trig_y + (trig_h - pop_h) * 0.5,
        };
        (chosen_x, chosen_y, flipped)
    } else {
        let above_y = trig_y - off - pop_h;
        let below_y = trig_bottom + off;
        let (chosen_y, flipped) = if placement.is_above() {
            if above_y < mon_top && below_y + pop_h <= mon_bottom {
                (below_y, true)
            } else {
                (above_y, false)
            }
        } else if below_y + pop_h > mon_bottom && above_y >= mon_top {
            (above_y, true)
        } else {
            (below_y, false)
        };
        let chosen_x = match placement.align() {
            PlacementAlign::Start => trig_x,
            PlacementAlign::End => trig_right - pop_w,
            PlacementAlign::Center => trig_x + (trig_w - pop_w) * 0.5,
        };
        (chosen_x, chosen_y, flipped)
    };

    let resolved_placement = if flipped {
        flip_tooltip_placement(placement)
    } else {
        placement
    };

    let max_x = (mon_right - pop_w).max(mon_left);
    let max_y = (mon_bottom - pop_h).max(mon_top);
    let snapped_x = snap_axis_inside(x, mon_left, max_x, scale_factor);
    let snapped_y = snap_axis_inside(y, mon_top, max_y, scale_factor);

    (point(px(snapped_x), px(snapped_y)), resolved_placement)
}

/// Computes the top-left desktop-logical origin for a separate tooltip popup
/// relative to `trigger_screen_bounds`, constrained to `monitor_bounds`.
///
/// Mirrors native popup positioning rules:
/// - Flips across `placement`'s primary axis when the preferred side overflows
///   `monitor_bounds` and the opposite side has room.
/// - Clamps to `monitor_bounds` edges (supporting negative multi-monitor
///   origins and oversized popups pinned at the monitor start edge).
/// - Snaps coordinates to the physical pixel grid inside `monitor_bounds` for
///   `scale_factor` so clamped edges stay both bounded and grid-aligned.
pub fn position_tooltip_popup(
    trigger_screen_bounds: Bounds<Pixels>,
    popup_size: Size<Pixels>,
    monitor_bounds: Bounds<Pixels>,
    placement: TooltipPlacement,
    offset: Pixels,
    scale_factor: f32,
) -> Point<Pixels> {
    resolve_tooltip_popup_placement(
        trigger_screen_bounds,
        popup_size,
        monitor_bounds,
        placement,
        offset,
        scale_factor,
    )
    .0
}

fn separate_popup_bounds(
    tip_origin: Point<Pixels>,
    tip_size: Size<Pixels>,
    resolved_placement: TooltipPlacement,
    show_arrow: bool,
    monitor: Bounds<Pixels>,
    scale_factor: f32,
) -> Bounds<Pixels> {
    if !show_arrow {
        return Bounds {
            origin: tip_origin,
            size: tip_size,
        };
    }
    let arrow = f32::from(TOOLTIP_ARROW_SIZE);
    let (raw_x, raw_y, win_w, win_h) = if resolved_placement.is_side() {
        let w = f32::from(tip_size.width) + arrow;
        let h = f32::from(tip_size.height);
        let x = if resolved_placement.is_start_side() {
            f32::from(tip_origin.x)
        } else {
            f32::from(tip_origin.x) - arrow
        };
        (x, f32::from(tip_origin.y), w, h)
    } else {
        let w = f32::from(tip_size.width);
        let h = f32::from(tip_size.height) + arrow;
        let y = if resolved_placement.is_above() {
            f32::from(tip_origin.y)
        } else {
            f32::from(tip_origin.y) - arrow
        };
        (f32::from(tip_origin.x), y, w, h)
    };

    let mon_left = f32::from(monitor.origin.x);
    let mon_top = f32::from(monitor.origin.y);
    let mon_right = mon_left + f32::from(monitor.size.width).max(0.0);
    let mon_bottom = mon_top + f32::from(monitor.size.height).max(0.0);
    let max_x = (mon_right - win_w).max(mon_left);
    let max_y = (mon_bottom - win_h).max(mon_top);
    let x = snap_axis_inside(raw_x, mon_left, max_x, scale_factor);
    let y = snap_axis_inside(raw_y, mon_top, max_y, scale_factor);
    Bounds {
        origin: point(px(x), px(y)),
        size: size(px(win_w), px(win_h)),
    }
}

/// The arrow points back at the trigger, so it faces opposite the tip.
fn arrow_rotation(placement: TooltipPlacement) -> f32 {
    if placement.is_above() {
        // The asset's apex is at the bottom, so it points down unrotated:
        // a tip above the trigger needs no rotation at all.
        0.
    } else if placement.is_side() {
        if placement.is_start_side() {
            -std::f32::consts::FRAC_PI_2
        } else {
            std::f32::consts::FRAC_PI_2
        }
    } else {
        std::f32::consts::PI
    }
}

/// HeroUI's `slide-in-from-*` entry offset for the physical side. Aligned
/// top/bottom placements share the same motion as their centered form.
fn entry_offset(placement: TooltipPlacement) -> (f32, f32) {
    if placement.is_above() {
        (0.0, 4.0)
    } else if placement.is_side() {
        if placement.is_start_side() {
            (4.0, 0.0)
        } else {
            (-4.0, 0.0)
        }
    } else {
        (0.0, -4.0)
    }
}

/// GPUI's pinned text wrapper breaks normal prose at spaces but has no
/// `overflow-wrap: anywhere` style. HeroUI applies that rule to tooltip text
/// so a long URL or token still fits the 320px cap. Zero-width break
/// opportunities preserve the visible and accessible text while allowing the
/// existing normal wrapper to split an unbroken token when it reaches the cap.
fn tooltip_text_with_break_opportunities(text: &str) -> String {
    let mut chars = text.chars().peekable();
    let mut result = String::with_capacity(text.len());
    while let Some(ch) = chars.next() {
        result.push(ch);
        if !ch.is_whitespace()
            && ch != '\u{200b}'
            && chars
                .peek()
                .is_some_and(|next| !next.is_whitespace() && *next != '\u{200b}')
        {
            result.push('\u{200b}');
        }
    }
    result
}

fn in_window_viewport_bounds(viewport: Size<Pixels>) -> Bounds<Pixels> {
    let margin = f32::from(TOOLTIP_VIEWPORT_MARGIN);
    let width = f32::from(viewport.width).max(0.0);
    let height = f32::from(viewport.height).max(0.0);
    let inset_x = margin.min(width * 0.5);
    let inset_y = margin.min(height * 0.5);
    Bounds {
        origin: point(px(inset_x), px(inset_y)),
        size: size(
            px((width - inset_x * 2.0).max(0.0)),
            px((height - inset_y * 2.0).max(0.0)),
        ),
    }
}

fn in_window_tooltip_max_width(viewport: Size<Pixels>) -> Pixels {
    (viewport.width - TOOLTIP_VIEWPORT_MARGIN * 2.)
        .max(px(0.))
        .min(TOOLTIP_MAX_WIDTH)
}

fn selector_base(id: &ElementId) -> String {
    match id {
        ElementId::Name(name) => name.to_string(),
        other => format!("{other:?}"),
    }
}

fn tooltip_display_content_for_width(
    text: &str,
    natural_width: Pixels,
    max_width: Pixels,
    available_text_width: Pixels,
    mut measure_word: impl FnMut(&str) -> Pixels,
) -> (String, bool) {
    if natural_width <= max_width {
        return (text.to_owned(), false);
    }
    let mut result = String::with_capacity(text.len());
    let mut idx = 0;
    while idx < text.len() {
        let rest = &text[idx..];
        let ch = rest.chars().next().unwrap();
        if ch.is_whitespace() {
            let ws_len = rest
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(rest.len());
            result.push_str(&rest[..ws_len]);
            idx += ws_len;
        } else {
            let word_len = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
            let word = &rest[..word_len];
            if measure_word(word) > available_text_width {
                result.push_str(&tooltip_text_with_break_opportunities(word));
            } else {
                result.push_str(word);
            }
            idx += word_len;
        }
    }
    (result, true)
}

fn measure_tooltip_text_width(window: &Window, text: &str, text_size: Pixels) -> Pixels {
    util::shape_multiline_max_width(text, &window.text_style().font(), text_size, window)
}

struct TooltipSurfaceSpec<'a> {
    key: &'a ElementId,
    content: SharedString,
    body: Option<AnyElement>,
    radius: Pixels,
    padding: Pixels,
    text_size: Pixels,
    max_width: Pixels,
    background: Option<gpui::Hsla>,
    foreground: Option<gpui::Hsla>,
    show_arrow: bool,
    placement: TooltipPlacement,
}

fn build_tooltip_surface(
    spec: TooltipSurfaceSpec<'_>,
    window: &mut Window,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let colors = cx.colors();
    let layout = cx.layout();
    let tip_bg = spec.background.unwrap_or(colors.overlay.background);
    let tip_fg = spec.foreground.unwrap_or(colors.overlay.foreground);
    let line_height = util::leading_for(spec.text_size).unwrap_or(px(16.));
    let hairline_width = if layout.overlay_hairline.is_some() {
        layout.border_width * 2.
    } else {
        px(0.)
    };
    let chrome_width = spec.padding * 2. + hairline_width;
    let max_width = spec.max_width.max(px(0.));
    let available_text_width = (max_width - chrome_width).max(px(0.));

    let (body_child, fixed_width, keep_nowrap): (AnyElement, Option<Pixels>, bool) = match spec.body
    {
        Some(body) => (body, None, false),
        None => {
            // Measure lines separately so embedded `\n` never trips GPUI's
            // single-line `shape_line` debug assertion, and insert zero-width
            // break opportunities only inside individual words wider than the
            // available text width.
            let natural_text_width =
                measure_tooltip_text_width(window, spec.content.as_ref(), spec.text_size);
            let natural_width = natural_text_width + chrome_width;
            let (display, wraps) = tooltip_display_content_for_width(
                spec.content.as_ref(),
                natural_width,
                max_width,
                available_text_width,
                |word| measure_tooltip_text_width(window, word, spec.text_size),
            );
            let tooltip_width = if wraps {
                max_width
            } else {
                natural_width.min(max_width)
            };
            let display_content: SharedString = display.into();
            (
                display_content.into_any_element(),
                Some(tooltip_width),
                !wraps,
            )
        }
    };

    let mut tip = gpui::div()
        .id(element_id::scoped(spec.key, "tip"))
        // The placement anchor lives on an outer wrapper. Keeping the painted
        // surface relative lets entry slide offsets work without clobbering
        // the anchor constraint.
        .relative()
        .flex_shrink_0()
        // `.tooltip` is `p-2` all round, not a wider-than-tall pill.
        .p(spec.padding)
        .map(|el| match fixed_width {
            Some(width) => el.w(width),
            None => el.max_w(max_width),
        })
        .rounded(spec.radius)
        .bg(tip_bg)
        .text_color(tip_fg)
        .text_size(spec.text_size)
        .line_height(line_height)
        // GPUI's normal wrapper can round a max-content width down by a glyph
        // fraction and split the last letter of a short label (e.g. `Left`).
        .when(keep_nowrap, |el| el.whitespace_nowrap())
        .when_some(layout.overlay_hairline, |el, hairline| {
            el.border(layout.border_width).border_color(hairline)
        })
        .shadow(layout.overlay_shadow.clone())
        .debug_selector({
            let name = format!("{}-tip", selector_base(spec.key));
            move || name
        })
        .child(body_child);

    if spec.show_arrow {
        let mut arrow = gpui::div()
            .absolute()
            .debug_selector({
                let name = format!("{}-arrow", selector_base(spec.key));
                move || name
            })
            .child(
                gpui::svg()
                    .size(TOOLTIP_ARROW_SIZE)
                    .path(icons::TOOLTIP_ARROW)
                    // svg() never inherits text colour; the arrow has to be tinted
                    // to match the tip body explicitly.
                    .text_color(tip_bg)
                    .with_transformation(gpui::Transformation::rotate(gpui::radians(
                        arrow_rotation(spec.placement),
                    ))),
            );
        arrow = if spec.placement.is_side() {
            let base = if spec.placement.is_start_side() {
                arrow.left_full()
            } else {
                arrow.right_full()
            };
            match spec.placement.align() {
                PlacementAlign::Start => base.top(px(0.)),
                PlacementAlign::End => base.bottom(px(0.)),
                PlacementAlign::Center => base.top(px(0.)).bottom(px(0.)).flex().items_center(),
            }
        } else {
            let base = if spec.placement.is_above() {
                arrow.top_full()
            } else {
                arrow.bottom_full()
            };
            match spec.placement.align() {
                PlacementAlign::Start => base.left(px(0.)),
                PlacementAlign::End => base.right(px(0.)),
                PlacementAlign::Center => base.left(px(0.)).right(px(0.)).flex().justify_center(),
            }
        };
        tip = tip.child(arrow);
    }

    tip
}

#[derive(Default)]
struct ActiveTooltipPresentations {
    next_generation: u64,
    active: HashMap<(WindowId, ElementId), u64>,
}

impl gpui::Global for ActiveTooltipPresentations {}

struct ActiveLease {
    key: ElementId,
    presenter: Arc<dyn TooltipPresenter>,
    generation: u64,
}

struct PresenterLease {
    window_id: WindowId,
    window_handle: AnyWindowHandle,
    active: Option<ActiveLease>,
}

impl PresenterLease {
    fn new(window: &Window) -> Self {
        Self {
            window_id: window.window_handle().window_id(),
            window_handle: window.window_handle(),
            active: None,
        }
    }
}

fn presenter_same_identity(a: &Arc<dyn TooltipPresenter>, b: &Arc<dyn TooltipPresenter>) -> bool {
    Arc::as_ptr(a).cast::<()>() == Arc::as_ptr(b).cast::<()>()
}

fn separate_window_presenter_arc() -> Arc<dyn TooltipPresenter> {
    thread_local! {
        static INSTANCE: Arc<dyn TooltipPresenter> = Arc::new(SeparateWindowTooltipPresenter);
    }
    INSTANCE.with(Arc::clone)
}

fn take_active_presentation_if_current(
    map_key: &(WindowId, ElementId),
    generation: u64,
    cx: &mut App,
) -> bool {
    let Some(global) = cx.try_global::<ActiveTooltipPresentations>() else {
        return false;
    };
    if global.active.get(map_key) != Some(&generation) {
        return false;
    }
    cx.update_global::<ActiveTooltipPresentations, _>(|global, _| {
        global.active.remove(map_key);
    });
    true
}

fn record_active_presentation(
    lease: &Entity<PresenterLease>,
    key: &ElementId,
    presenter: Arc<dyn TooltipPresenter>,
    window: &mut Window,
    cx: &mut App,
) {
    if cx.try_global::<ActiveTooltipPresentations>().is_none() {
        cx.set_global(ActiveTooltipPresentations::default());
    }
    let window_id = window.window_handle().window_id();
    let prev = lease.update(cx, |state, _| state.active.take());
    if let Some(prev) = prev {
        let same_key = prev.key == *key;
        let same_presenter = presenter_same_identity(&prev.presenter, &presenter);
        if same_key
            && same_presenter
            && cx
                .global::<ActiveTooltipPresentations>()
                .active
                .get(&(window_id, key.clone()))
                == Some(&prev.generation)
        {
            lease.update(cx, |state, _| {
                state.active = Some(prev);
            });
            return;
        }
        let prev_map_key = (window_id, prev.key.clone());
        if take_active_presentation_if_current(&prev_map_key, prev.generation, cx) {
            prev.presenter.dismiss(&prev.key, Some(window), cx);
        }
    }

    let map_key = (window_id, key.clone());
    let generation = cx.update_global::<ActiveTooltipPresentations, _>(|global, _| {
        global.next_generation = global.next_generation.saturating_add(1);
        let generation = global.next_generation;
        global.active.insert(map_key, generation);
        generation
    });
    lease.update(cx, |state, _| {
        state.active = Some(ActiveLease {
            key: key.clone(),
            presenter,
            generation,
        });
    });
}

fn dismiss_lease(lease: &Entity<PresenterLease>, window: &mut Window, cx: &mut App) {
    let active = lease.update(cx, |state, _| state.active.take());
    let Some(active) = active else {
        return;
    };
    let map_key = (window.window_handle().window_id(), active.key.clone());
    if take_active_presentation_if_current(&map_key, active.generation, cx) {
        active.presenter.dismiss(&active.key, Some(window), cx);
    }
}

/// Built-in [`TooltipPresenter`] used by [`TooltipPresenterMode::SeparateWindow`].
///
/// Opens a separate popup GPUI window (`WindowKind::AnchoredPopup` with `grab: false`
/// on Wayland, or `WindowKind::PopUp` with `focus: false` on macOS/Windows/X11)
/// positioned near the trigger via [`position_tooltip_popup`] and closes it cleanly
/// when dismissed, deactivated, or unmounted. On Windows, clicking the popup
/// window itself activates it and immediately triggers owner-deactivation dismissal;
/// applications requiring click-through `WS_EX_NOACTIVATE` or native capture-exclusion
/// handles should attach a native [`TooltipPresenter`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SeparateWindowTooltipPresenter;

#[derive(Clone, PartialEq)]
struct SeparateTooltipSnapshot {
    key: ElementId,
    content: SharedString,
    radius: Pixels,
    padding: Pixels,
    text_size: Pixels,
    background: Option<gpui::Hsla>,
    foreground: Option<gpui::Hsla>,
    show_arrow: bool,
    placement: TooltipPlacement,
    tip_size: Size<Pixels>,
    display_id: Option<DisplayId>,
    scale_factor_bits: u32,
}

#[derive(Clone, PartialEq)]
struct TooltipMeasureKey {
    content: SharedString,
    padding: Pixels,
    text_size: Pixels,
    hairline_width: Pixels,
    font: gpui::Font,
    scale_factor_bits: u32,
}

fn resolve_trigger_display(
    window: &Window,
    trigger_screen_bounds: Bounds<Pixels>,
    cx: &App,
) -> Option<std::rc::Rc<dyn gpui::PlatformDisplay>> {
    let window_display = window.display(cx);
    let displays = cx.displays();
    if displays.is_empty() {
        return window_display;
    }
    let first_bounds = displays[0].bounds();
    let has_distinct_global_bounds = displays.iter().any(|d| d.bounds() != first_bounds);
    if !has_distinct_global_bounds {
        return window_display.or_else(|| displays.into_iter().next());
    }
    let trig_center = trigger_screen_bounds.center();
    let window_id = window_display.as_ref().map(|d| d.id());
    let mut best: Option<(f32, bool, std::rc::Rc<dyn gpui::PlatformDisplay>)> = None;
    for display in displays {
        let db = display.bounds();
        let ix = trigger_screen_bounds.intersect(&db);
        let area = f32::from(ix.size.width).max(0.0) * f32::from(ix.size.height).max(0.0);
        let contains_center = db.contains(&trig_center);
        if area <= 0.0 && !contains_center {
            continue;
        }
        let is_window_display = window_id == Some(display.id());
        let replace = match &best {
            None => true,
            Some((best_area, best_center, _)) => {
                area > *best_area
                    || ((area - *best_area).abs() <= f32::EPSILON
                        && ((contains_center && !*best_center)
                            || (contains_center == *best_center && is_window_display)))
            }
        };
        if replace {
            best = Some((area, contains_center, display));
        }
    }
    best.map(|(_, _, d)| d).or(window_display)
}

#[cfg(target_os = "linux")]
fn anchored_popup_options(
    parent: AnyWindowHandle,
    trigger_bounds: Bounds<Pixels>,
    placement: TooltipPlacement,
    offset: Pixels,
    show_arrow: bool,
) -> gpui::popup::PopupOptions {
    use gpui::popup::{PopupAnchor, PopupConstraintAdjustment, PopupGravity, PopupOptions};

    let arrow_shift = if show_arrow {
        TOOLTIP_ARROW_SIZE
    } else {
        px(0.)
    };
    let (anchor, gravity, popup_offset) = if placement.is_side() {
        let is_start = placement.is_start_side();
        let (anchor, gravity) = match (is_start, placement.align()) {
            (true, PlacementAlign::Start) => (PopupAnchor::TopLeft, PopupGravity::TopLeft),
            (true, PlacementAlign::End) => (PopupAnchor::BottomLeft, PopupGravity::BottomLeft),
            (true, PlacementAlign::Center) => (PopupAnchor::Left, PopupGravity::Left),
            (false, PlacementAlign::Start) => (PopupAnchor::TopRight, PopupGravity::TopRight),
            (false, PlacementAlign::End) => (PopupAnchor::BottomRight, PopupGravity::BottomRight),
            (false, PlacementAlign::Center) => (PopupAnchor::Right, PopupGravity::Right),
        };
        let dx = if is_start {
            -(offset - arrow_shift)
        } else {
            offset - arrow_shift
        };
        (anchor, gravity, point(dx, px(0.)))
    } else {
        let is_above = placement.is_above();
        let (anchor, gravity) = match (is_above, placement.align()) {
            (true, PlacementAlign::Start) => (PopupAnchor::TopLeft, PopupGravity::TopLeft),
            (true, PlacementAlign::End) => (PopupAnchor::TopRight, PopupGravity::TopRight),
            (true, PlacementAlign::Center) => (PopupAnchor::Top, PopupGravity::Top),
            (false, PlacementAlign::Start) => (PopupAnchor::BottomLeft, PopupGravity::BottomLeft),
            (false, PlacementAlign::End) => (PopupAnchor::BottomRight, PopupGravity::BottomRight),
            (false, PlacementAlign::Center) => (PopupAnchor::Bottom, PopupGravity::Bottom),
        };
        let dy = if is_above {
            -(offset - arrow_shift)
        } else {
            offset - arrow_shift
        };
        (anchor, gravity, point(px(0.), dy))
    };

    PopupOptions {
        parent,
        anchor_rect: trigger_bounds,
        anchor,
        gravity,
        constraint_adjustment: PopupConstraintAdjustment::FLIP_X
            | PopupConstraintAdjustment::FLIP_Y
            | PopupConstraintAdjustment::SLIDE_X
            | PopupConstraintAdjustment::SLIDE_Y,
        offset: popup_offset,
        grab: false,
    }
}

struct SeparateWindowEntry {
    generation: u64,
    handle: Option<AnyWindowHandle>,
    bounds: Option<Bounds<Pixels>>,
    snapshot: Option<SeparateTooltipSnapshot>,
}

#[derive(Default)]
struct SeparateTooltipRegistry {
    next_generation: u64,
    windows: HashMap<(WindowId, ElementId), SeparateWindowEntry>,
    failed: HashSet<(WindowId, ElementId)>,
    #[cfg(test)]
    force_open_failure_for_test: bool,
}

impl gpui::Global for SeparateTooltipRegistry {}

fn is_separate_tooltip_failed(window_id: WindowId, key: &ElementId, cx: &App) -> bool {
    cx.try_global::<SeparateTooltipRegistry>()
        .is_some_and(|reg| reg.failed.contains(&(window_id, key.clone())))
}

fn clear_separate_tooltip_failed(window_id: WindowId, key: &ElementId, cx: &mut App) {
    if cx.try_global::<SeparateTooltipRegistry>().is_some() {
        cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
            reg.failed.remove(&(window_id, key.clone()));
        });
    }
}

struct SeparateTooltipView {
    key: ElementId,
    content: SharedString,
    #[allow(clippy::type_complexity)]
    body: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
    radius: Pixels,
    padding: Pixels,
    text_size: Pixels,
    background: Option<gpui::Hsla>,
    foreground: Option<gpui::Hsla>,
    show_arrow: bool,
    placement: TooltipPlacement,
    owner_handle: AnyWindowHandle,
}

impl Render for SeparateTooltipView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let rendered_body = self.body.as_ref().map(|body| body(window, cx));
        let key = self.key.clone();
        let owner_handle = self.owner_handle;
        let tip = build_tooltip_surface(
            TooltipSurfaceSpec {
                key: &self.key,
                content: self.content.clone(),
                body: rendered_body,
                radius: self.radius,
                padding: self.padding,
                text_size: self.text_size,
                max_width: TOOLTIP_MAX_WIDTH,
                background: self.background,
                foreground: self.foreground,
                show_arrow: self.show_arrow,
                placement: self.placement,
            },
            window,
            cx,
        )
        .a11y_named(
            a11y::Role::Tooltip,
            &a11y::Name::labelled(self.content.clone()),
        );
        gpui::div()
            .size_full()
            .flex()
            .items_start()
            .justify_start()
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    let key = key.clone();
                    let _ = owner_handle.update(cx, |_, owner_window, cx| {
                        let state = owner_window.with_id(std::any::type_name::<Tooltip>(), |w| {
                            w.use_keyed_state(key.clone(), cx, |_, _| TooltipHover::new())
                        });
                        state.update(cx, |s, cx| {
                            if s.close(true) {
                                cx.notify();
                            }
                        });
                        SeparateWindowTooltipPresenter.dismiss(&key, Some(owner_window), cx);
                        owner_window.activate_window();
                    });
                    window.remove_window();
                    cx.stop_propagation();
                }
            })
            .when(self.show_arrow, |el| {
                if self.placement.is_side() {
                    if self.placement.is_start_side() {
                        el.pr(TOOLTIP_ARROW_SIZE)
                    } else {
                        el.pl(TOOLTIP_ARROW_SIZE)
                    }
                } else if self.placement.is_above() {
                    el.pb(TOOLTIP_ARROW_SIZE)
                } else {
                    el.pt(TOOLTIP_ARROW_SIZE)
                }
            })
            .child(tip)
    }
}

impl TooltipPresenter for SeparateWindowTooltipPresenter {
    fn present(&self, request: &TooltipPresentationRequest, window: &mut Window, cx: &mut App) {
        if cx.try_global::<SeparateTooltipRegistry>().is_none() {
            cx.set_global(SeparateTooltipRegistry::default());
        }
        let owner_handle = window.window_handle();
        let map_key = (owner_handle.window_id(), request.key.clone());

        let tip_size = request.tip_size.unwrap_or_else(|| size(px(180.), px(32.)));
        let monitor = request.display_bounds.unwrap_or(Bounds {
            origin: point(px(0.), px(0.)),
            size: size(px(1920.), px(1080.)),
        });
        let (tip_origin, resolved_placement) = resolve_tooltip_popup_placement(
            request.trigger_screen_bounds,
            tip_size,
            monitor,
            request.placement,
            request.offset,
            request.scale_factor,
        );
        let popup_bounds = separate_popup_bounds(
            tip_origin,
            tip_size,
            resolved_placement,
            request.show_arrow,
            monitor,
            request.scale_factor,
        );
        let display_id = request
            .display_id
            .or_else(|| window.display(cx).map(|display| display.id()));
        let new_snapshot = SeparateTooltipSnapshot {
            key: request.key.clone(),
            content: request.content.clone(),
            radius: request.radius,
            padding: request.padding,
            text_size: request.text_size,
            background: request.background,
            foreground: request.foreground,
            show_arrow: request.show_arrow,
            placement: resolved_placement,
            tip_size,
            display_id,
            scale_factor_bits: request.scale_factor.to_bits(),
        };

        let (live_handle, prev_bounds, prev_snapshot) = {
            let existing = cx.global::<SeparateTooltipRegistry>().windows.get(&map_key);
            let existing_handle = existing.and_then(|entry| entry.handle);
            let prev_bounds = existing.and_then(|entry| entry.bounds);
            let prev_snapshot = existing.and_then(|entry| entry.snapshot.clone());
            let live_handle = existing_handle.filter(|handle| cx.windows().contains(handle));
            (live_handle, prev_bounds, prev_snapshot)
        };

        if live_handle.is_some()
            && request.body.is_none()
            && prev_bounds == Some(popup_bounds)
            && prev_snapshot.as_ref() == Some(&new_snapshot)
        {
            return;
        }

        let origin_changed = prev_bounds.is_some_and(|prev| prev.origin != popup_bounds.origin);
        let display_changed =
            prev_snapshot.is_some_and(|prev| prev.display_id != new_snapshot.display_id);
        let size_changed = prev_bounds.is_none_or(|prev| prev.size != popup_bounds.size);
        let generation = cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
            reg.next_generation = reg.next_generation.saturating_add(1);
            let generation = reg.next_generation;
            reg.windows.insert(
                map_key.clone(),
                SeparateWindowEntry {
                    generation,
                    handle: live_handle,
                    bounds: Some(popup_bounds),
                    snapshot: Some(new_snapshot.clone()),
                },
            );
            generation
        });

        let key = request.key.clone();
        let content = request.content.clone();
        let body = request.body.clone();
        let radius = request.radius;
        let padding = request.padding;
        let text_size = request.text_size;
        let background = request.background;
        let foreground = request.foreground;
        let show_arrow = request.show_arrow;
        let trigger_bounds = request.trigger_bounds;
        let offset = request.offset;
        let requested_placement = request.placement;

        cx.defer(move |cx| {
            let still_current = cx
                .try_global::<SeparateTooltipRegistry>()
                .and_then(|reg| reg.windows.get(&map_key))
                .is_some_and(|entry| entry.generation == generation);
            if !still_current {
                return;
            }
            if let Some(handle) = live_handle {
                if !origin_changed && !display_changed {
                    let updated = handle.update(cx, |view, window, cx| {
                        if let Ok(view) = view.downcast::<SeparateTooltipView>() {
                            view.update(cx, |state, cx| {
                                state.key = key.clone();
                                state.content = content.clone();
                                state.body = body.clone();
                                state.radius = radius;
                                state.padding = padding;
                                state.text_size = text_size;
                                state.background = background;
                                state.foreground = foreground;
                                state.show_arrow = show_arrow;
                                state.placement = resolved_placement;
                                cx.notify();
                            });
                        }
                        if size_changed {
                            window.resize(popup_bounds.size);
                        }
                    });
                    if updated.is_ok() {
                        return;
                    }
                } else {
                    cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
                        if let Some(entry) = reg.windows.get_mut(&map_key)
                            && entry.generation == generation
                        {
                            entry.handle = None;
                        }
                    });
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                }
            }

            #[cfg(test)]
            let force_fail = cx
                .try_global::<SeparateTooltipRegistry>()
                .is_some_and(|reg| reg.force_open_failure_for_test);
            #[cfg(not(test))]
            let force_fail = false;

            let open_kind =
                |kind: WindowKind,
                 view_key: ElementId,
                 view_content: SharedString,
                 view_body: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
                 cx: &mut App| {
                    let options = WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(popup_bounds)),
                        titlebar: None,
                        focus: false,
                        show: true,
                        kind,
                        is_movable: false,
                        is_resizable: false,
                        is_minimizable: false,
                        display_id,
                        window_background: WindowBackgroundAppearance::Transparent,
                        ..Default::default()
                    };
                    cx.open_window(options, move |_, cx| {
                        cx.new(|_| SeparateTooltipView {
                            key: view_key,
                            content: view_content,
                            body: view_body,
                            radius,
                            padding,
                            text_size,
                            background,
                            foreground,
                            show_arrow,
                            placement: resolved_placement,
                            owner_handle,
                        })
                    })
                    .ok()
                };
            let opened = if force_fail {
                None
            } else {
                #[cfg(target_os = "linux")]
                {
                    open_kind(
                        WindowKind::AnchoredPopup(anchored_popup_options(
                            owner_handle,
                            trigger_bounds,
                            requested_placement,
                            offset,
                            show_arrow,
                        )),
                        key.clone(),
                        content.clone(),
                        body.clone(),
                        cx,
                    )
                    .or_else(|| open_kind(WindowKind::PopUp, key, content, body, cx))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = (trigger_bounds, offset, requested_placement);
                    open_kind(WindowKind::PopUp, key, content, body, cx)
                }
            };
            match opened {
                Some(opened) => {
                    let any_handle: AnyWindowHandle = opened.into();
                    let mut obsolete = true;
                    cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
                        if let Some(entry) = reg.windows.get_mut(&map_key)
                            && entry.generation == generation
                        {
                            entry.handle = Some(any_handle);
                            obsolete = false;
                        }
                    });
                    if obsolete {
                        let _ = any_handle.update(cx, |_, window, _| window.remove_window());
                    }
                }
                None => {
                    let mut fallback_needed = false;
                    cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
                        if reg
                            .windows
                            .get(&map_key)
                            .is_some_and(|entry| entry.generation == generation)
                        {
                            reg.windows.remove(&map_key);
                            reg.failed.insert(map_key.clone());
                            fallback_needed = true;
                        }
                    });
                    if fallback_needed {
                        let _ = owner_handle.update(cx, |_, window, _| window.refresh());
                    }
                }
            }
        });
    }

    fn dismiss(&self, key: &ElementId, window: Option<&mut Window>, cx: &mut App) {
        if cx.try_global::<SeparateTooltipRegistry>().is_none() {
            return;
        }
        let handles: Vec<AnyWindowHandle> = if let Some(window) = window {
            let map_key = (window.window_handle().window_id(), key.clone());
            cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
                reg.next_generation = reg.next_generation.saturating_add(1);
                reg.windows
                    .remove(&map_key)
                    .and_then(|entry| entry.handle)
                    .into_iter()
                    .collect()
            })
        } else {
            let open_windows: HashSet<WindowId> =
                cx.windows().into_iter().map(|h| h.window_id()).collect();
            cx.update_global::<SeparateTooltipRegistry, _>(|reg, _| {
                reg.next_generation = reg.next_generation.saturating_add(1);
                let mut removed = Vec::new();
                reg.windows.retain(|(wid, k), entry| {
                    if k == key && !open_windows.contains(wid) {
                        if let Some(handle) = entry.handle {
                            removed.push(handle);
                        }
                        false
                    } else {
                        true
                    }
                });
                reg.failed
                    .retain(|(wid, k)| !(k == key && !open_windows.contains(wid)));
                removed
            })
        };
        if !handles.is_empty() {
            cx.defer(move |cx| {
                for handle in handles {
                    let _ = handle.update(cx, |_, window, _| window.remove_window());
                }
            });
        }
    }
}

struct TooltipTriggerPresent {
    child: AnyElement,
    trigger_bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    presenter: Arc<dyn TooltipPresenter>,
    lease: Entity<PresenterLease>,
    request: Option<TooltipPresentationRequest>,
}

impl Element for TooltipTriggerPresent {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.child.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
        if let Some(mut req) = self.request.take() {
            if let Some(trigger_bounds) = self.trigger_bounds.get() {
                req.trigger_bounds = trigger_bounds;
                let trigger_screen_bounds = Bounds {
                    origin: req.window_bounds.origin + trigger_bounds.origin,
                    size: trigger_bounds.size,
                };
                if trigger_screen_bounds != req.trigger_screen_bounds {
                    req.trigger_screen_bounds = trigger_screen_bounds;
                    let display = resolve_trigger_display(window, trigger_screen_bounds, cx);
                    req.display_id = display.as_ref().map(|d| d.id());
                    req.display_bounds = display.map(|d| d.visible_bounds());
                }
            }
            record_active_presentation(&self.lease, &req.key, self.presenter.clone(), window, cx);
            self.presenter.present(&req, window, cx);
        } else {
            dismiss_lease(&self.lease, window, cx);
        }
    }
}

impl IntoElement for TooltipTriggerPresent {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct TooltipPositioner {
    trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    resolved_placement: popover::ResolvedPlacementHandle,
    requested_placement: TooltipPlacement,
    built_placement: TooltipPlacement,
    offset: Pixels,
    container: AnyElement,
}

impl TooltipPositioner {
    fn new(
        trigger: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
        resolved_placement: popover::ResolvedPlacementHandle,
        requested_placement: TooltipPlacement,
        built_placement: TooltipPlacement,
        offset: Pixels,
        child: impl IntoElement,
    ) -> Self {
        Self {
            trigger,
            resolved_placement,
            requested_placement,
            built_placement,
            offset,
            container: gpui::div().flex().child(child).into_any_element(),
        }
    }
}

impl Element for TooltipPositioner {
    type RequestLayoutState = ();
    type PrepaintState = bool;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let _ = self.container.request_layout(window, cx);
        let layout = window.request_layout(
            gpui::Style {
                position: gpui::Position::Absolute,
                ..gpui::Style::default()
            },
            [],
            cx,
        );
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let Some(trigger) = self.trigger.get() else {
            return false;
        };
        let tip_size = self.container.layout_as_root(
            size(
                gpui::AvailableSpace::MaxContent,
                gpui::AvailableSpace::MaxContent,
            ),
            window,
            cx,
        );
        let viewport_bounds = in_window_viewport_bounds(window.viewport_size());
        let (origin, resolved) = resolve_tooltip_popup_placement(
            trigger,
            tip_size,
            viewport_bounds,
            self.requested_placement,
            self.offset,
            window.scale_factor(),
        );
        if self.resolved_placement.get() != Some(resolved) {
            self.resolved_placement.set(Some(resolved));
        }
        if resolved != self.built_placement {
            window.defer(cx, |window, _| window.refresh());
        }
        window.with_absolute_element_offset(origin, |window| {
            self.container.prepaint(window, cx);
        });
        true
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepainted: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !*prepainted {
            return;
        }
        self.container.paint(window, cx);
    }
}

impl IntoElement for TooltipPositioner {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
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
/// dismissal is remembered per tooltip instead, and dropped on either edge of
/// the focus session. A dismissal therefore lasts only for the current focus:
/// the next keyboard focus shows the tip again.
pub struct TooltipHover {
    open: bool,
    generation: u64,
    focus_dismissed: bool,
    focus_open: bool,
    was_focused: bool,
}

impl TooltipHover {
    fn new() -> Self {
        Self {
            open: false,
            generation: 0,
            focus_dismissed: false,
            focus_open: false,
            was_focused: false,
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

    /// Whether keyboard-visible focus opened the tip in this focus session.
    pub fn is_focus_open(&self) -> bool {
        self.focus_open && !self.focus_dismissed
    }

    fn close(&mut self, dismiss_focus: bool) -> bool {
        self.generation += 1;
        let was_open = self.open || self.is_focus_open();
        self.open = false;
        if dismiss_focus {
            self.focus_dismissed = true;
        }
        was_open
    }
}

#[derive(Default)]
struct TooltipManager {
    entries: Vec<gpui::WeakEntity<TooltipHover>>,
    warmed_up: bool,
    cooldown_generation: u64,
}

impl gpui::Global for TooltipManager {}

fn ensure_tooltip_manager(cx: &mut App) {
    if cx.try_global::<TooltipManager>().is_none() {
        cx.set_global(TooltipManager::default());
    }
}

fn prepare_tooltip_open(current: &gpui::WeakEntity<TooltipHover>, cx: &mut App) -> bool {
    ensure_tooltip_manager(cx);
    let (warmed_up, others) = cx.update_global::<TooltipManager, _>(|manager, _| {
        manager.entries.retain(|entry| entry.upgrade().is_some());
        let others = manager
            .entries
            .iter()
            .filter(|entry| *entry != current)
            .filter_map(gpui::WeakEntity::upgrade)
            .collect::<Vec<_>>();
        manager.entries.retain(|entry| entry == current);
        if manager.entries.is_empty() {
            manager.entries.push(current.clone());
        }
        (manager.warmed_up, others)
    });
    // Entity updates run after the global borrow is released. `current` may
    // itself be mid-update when a hover timer calls this helper.
    for other in others {
        other.update(cx, |state, cx| {
            if state.close(true) {
                cx.notify();
            }
        });
    }
    warmed_up
}

fn mark_tooltip_open(cx: &mut App) {
    cx.update_global::<TooltipManager, _>(|manager, _| {
        manager.warmed_up = true;
        manager.cooldown_generation += 1;
    });
}

fn start_tooltip_cooldown(
    current: &gpui::WeakEntity<TooltipHover>,
    close_delay: u64,
    cx: &mut App,
) {
    ensure_tooltip_manager(cx);
    let generation = cx.update_global::<TooltipManager, _>(|manager, _| {
        if !manager.warmed_up || !manager.entries.iter().any(|entry| entry == current) {
            return None;
        }
        manager.cooldown_generation += 1;
        Some(manager.cooldown_generation)
    });
    let Some(generation) = generation else {
        return;
    };
    let cooldown = cx.layout().tooltip_cooldown_ms.max(close_delay);
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        cx.background_executor()
            .timer(Duration::from_millis(cooldown))
            .await;
        cx.update_global::<TooltipManager, _>(|manager, _| {
            if manager.cooldown_generation == generation {
                manager.warmed_up = false;
                manager.entries.clear();
            }
        });
    })
    .detach();
}

/// `trigger` — what reveals the tip.
///
/// v3's default is `hover`, and React Aria shows a hovered tooltip on keyboard
/// focus as well, so `Hover` means "either". `Focus` is the narrower one: the
/// pointer does nothing and only focus opens it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipTrigger {
    #[default]
    /// Shown while the trigger is hovered.
    Hover,
    /// Shown while the trigger is focused.
    Focus,
}

impl TooltipTrigger {
    /// Every trigger mode, in declaration order.
    pub const ALL: [TooltipTrigger; 2] = [TooltipTrigger::Hover, TooltipTrigger::Focus];

    /// A human-readable label for this trigger mode.
    pub fn label(self) -> &'static str {
        match self {
            TooltipTrigger::Hover => "Hover",
            TooltipTrigger::Focus => "Focus",
        }
    }
}

/// HeroUI Tooltip: wraps a trigger and reveals a tip on hover.
#[must_use = "a component does nothing until it is rendered: add it as a child or return it from `render`"]
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
    presenter_mode: TooltipPresenterMode,
    presenter: Option<Arc<dyn TooltipPresenter>>,
    exclude_from_capture: bool,
    /// A caller-drawn tip body, in place of the measured single line.
    #[allow(clippy::type_complexity)]
    body: Option<Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>>,
    children: Vec<AnyElement>,
    /// The corner radius, in place of the owning `small_radius` helper.
    radius: Option<Pixels>,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
    recipes: Vec<SharedString>,
}

impl Tooltip {
    /// Creates a tooltip with the given content.
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
            presenter_mode: TooltipPresenterMode::default(),
            presenter: None,
            exclude_from_capture: false,
            body: None,
            children: Vec::new(),
            radius: None,
            sx: None,
            recipes: Vec::new(),
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

    /// Selects whether the tooltip renders inside the owning window or in a
    /// separate non-activating popup window.
    ///
    /// When a custom [`Tooltip::presenter`] is also provided, that presenter
    /// takes precedence whenever mode is [`TooltipPresenterMode::SeparateWindow`]
    /// (and setting [`Tooltip::presenter`] alone also activates external
    /// presentation).
    pub fn presenter_mode(mut self, mode: TooltipPresenterMode) -> Self {
        self.presenter_mode = mode;
        self
    }

    /// Attaches a custom [`TooltipPresenter`] to render the tip outside the
    /// owning window while retaining HeroGPUI's hover, focus, delay, cooldown,
    /// Escape, and unmount lifecycle.
    pub fn presenter(mut self, presenter: Arc<dyn TooltipPresenter>) -> Self {
        self.presenter = Some(presenter);
        self
    }

    /// Requests that external or native tooltip presenters exclude the tooltip
    /// surface from screenshots and screen recordings when supported by the
    /// host OS backend.
    pub fn exclude_from_capture(mut self, exclude: bool) -> Self {
        self.exclude_from_capture = exclude;
        self
    }

    /// Draws the tip's body yourself, in place of the text handed to
    /// [`Tooltip::new`].
    ///
    /// v3's `Tooltip` takes children, so a tip is free to compose a small
    /// table, a key/value list or a swatch legend. This port shapes the tip's
    /// text to reproduce CSS `max-content` capped at 320px, which only a string
    /// can go through; an element body skips text shaping and takes its own
    /// intrinsic width under the same 320px cap instead.
    ///
    /// The string from [`Tooltip::new`] is still required and still used: it
    /// remains the tip's accessible name and the default hover key, so a rich
    /// tip cannot ship without something a screen reader can read. Pass the
    /// text the body conveys.
    ///
    /// The closure runs only while the tip is on screen, never for a closed
    /// tooltip, and it runs again on each frame of the reveal.
    ///
    /// ```
    /// # use herogpui_components::tooltip::Tooltip;
    /// # use gpui::{div, IntoElement, ParentElement};
    /// Tooltip::new("Tokens: name, kind, scope")
    ///     .body(|_, _| {
    ///         div()
    ///             .child("name — the identifier")
    ///             .child("kind — the token class")
    ///             .into_any_element()
    ///     })
    ///     .child(div().child("tokens"));
    /// ```
    pub fn body(mut self, render: impl Fn(&mut Window, &mut App) -> AnyElement + 'static) -> Self {
        self.body = Some(Arc::new(render));
        self
    }

    /// `isDisabled` — suppresses the tip entirely.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// Sets where the tooltip sits relative to its trigger.
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

    /// The corner radius, in place of the owning `small_radius` helper. The
    /// tip's entry zoom interpolates the same value, so both follow the
    /// override. Not a v3 prop; the removed v2 `radius` prop is prohibited
    /// and this is a per-component repository extension.
    pub fn radius(mut self, radius: impl Into<Pixels>) -> Self {
        self.radius = Some(radius.into());
        self
    }

    /// Named theme overlay from [`herogpui_theme::ComponentThemes::tooltip`].
    /// Stackable; a missing name adds no override.
    pub fn recipe(mut self, name: impl Into<SharedString>) -> Self {
        self.recipes.push(name.into());
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the tooltip's root element — the wrapper the trigger and the
    /// floating tip sit in — after every value the placement and the active
    /// theme chose, so they win.
    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        util::refine_sx(&mut self.sx, style);
        self
    }

    fn resolved_presenter(&self) -> Option<Arc<dyn TooltipPresenter>> {
        if let Some(presenter) = &self.presenter {
            Some(presenter.clone())
        } else if self.presenter_mode == TooltipPresenterMode::SeparateWindow {
            Some(separate_window_presenter_arc())
        } else {
            None
        }
    }
}

impl ParentElement for Tooltip {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

fn get_or_init_presenter_lease(
    key: &ElementId,
    window: &mut Window,
    cx: &mut App,
) -> Entity<PresenterLease> {
    window.use_keyed_state(
        element_id::scoped(key, "presenter-lease"),
        cx,
        |window, cx| {
            cx.on_release(|lease: &mut PresenterLease, cx| {
                let Some(active) = lease.active.take() else {
                    return;
                };
                let map_key = (lease.window_id, active.key.clone());
                let window_handle = lease.window_handle;
                cx.defer(move |cx| {
                    if !take_active_presentation_if_current(&map_key, active.generation, cx) {
                        return;
                    }
                    let mut dismissed_with_window = false;
                    let _ = window_handle.update(cx, |_, window, cx| {
                        active.presenter.dismiss(&active.key, Some(window), cx);
                        dismissed_with_window = true;
                    });
                    if !dismissed_with_window {
                        active.presenter.dismiss(&active.key, None, cx);
                    }
                });
            })
            .detach();
            PresenterLease::new(window)
        },
    )
}

impl RenderOnce for Tooltip {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let key = self
            .id
            .clone()
            .unwrap_or_else(|| ElementId::Name(self.content.clone()));
        let window_id = window.window_handle().window_id();
        let using_separate_fallback = self.presenter.is_none()
            && self.presenter_mode == TooltipPresenterMode::SeparateWindow
            && is_separate_tooltip_failed(window_id, &key, cx);
        let external_presenter = if using_separate_fallback {
            None
        } else {
            self.resolved_presenter()
        };
        let has_active_lease = cx
            .try_global::<ActiveTooltipPresentations>()
            .is_some_and(|global| global.active.contains_key(&(window_id, key.clone())));
        let lease = if (!self.is_disabled && external_presenter.is_some()) || has_active_lease {
            Some(get_or_init_presenter_lease(&key, window, cx))
        } else {
            None
        };

        if self.is_disabled {
            if let Some(lease) = &lease {
                dismiss_lease(lease, window, cx);
            }
            clear_separate_tooltip_failed(window_id, &key, cx);
            // A disabled tooltip renders its trigger and nothing else.
            return util::apply_sx(gpui::div().flex().children(self.children), &self.sx)
                .into_any_element();
        }

        // The state entity has to be created before the theme tokens are read;
        // `use_keyed_state` takes `cx` mutably and would conflict with them.
        let state = window.use_keyed_state(key.clone(), cx, |_, _| TooltipHover::new());
        let weak_hover_activation = state.downgrade();
        let _activation_watch = window.use_keyed_state(
            element_id::scoped(&key, "activation-watch"),
            cx,
            move |window, cx| {
                cx.observe_window_activation(window, move |_, window, cx| {
                    if !window.is_window_active()
                        && let Some(hover) = weak_hover_activation.upgrade()
                    {
                        hover.update(cx, |s, cx| {
                            if s.close(true) {
                                cx.notify();
                            }
                        });
                    }
                })
            },
        );
        let current_tooltip = state.downgrade();
        let (
            theme_radius,
            theme_offset,
            theme_delay_ms,
            theme_close_delay_ms,
            theme_padding,
            theme_text_size,
            theme_background,
            theme_foreground,
        ) = {
            let tooltip_theme = cx
                .theme()
                .components
                .tooltip
                .resolve_named("tooltip", &self.recipes);
            (
                tooltip_theme.radius,
                tooltip_theme.offset,
                tooltip_theme.delay_ms,
                tooltip_theme.close_delay_ms,
                tooltip_theme.padding,
                tooltip_theme.text_size,
                tooltip_theme.background,
                tooltip_theme.foreground,
            )
        };
        let (delay, close_delay) = {
            let layout = cx.layout();
            (
                self.delay
                    .or(theme_delay_ms)
                    .unwrap_or(layout.tooltip_delay_ms),
                self.close_delay
                    .or(theme_close_delay_ms)
                    .unwrap_or(layout.tooltip_close_delay_ms),
            )
        };
        let tip_padding = theme_padding.unwrap_or(px(8.));
        let tip_text_size = theme_text_size.unwrap_or(px(12.));
        let resolved_offset =
            self.offset
                .or(theme_offset)
                .unwrap_or(if self.show_arrow { px(7.) } else { px(3.) });
        // React Aria explicitly removes the Trigger wrapper's tab index: the
        // caller's trigger is the stop, and this handle only reports whether a
        // descendant currently owns focus.
        let wrap_focus =
            window.use_keyed_state(element_id::scoped(&key, "wrap-focus"), cx, |_, cx| {
                cx.focus_handle()
            });
        let wrap_handle = wrap_focus.read(cx).clone();
        let focus_held = wrap_handle.contains_focused(window, cx);
        // Escape's dismissal is per focus *session*: once the focus leaves the
        // trigger, the latch is dropped, so the next focus is a fresh one and
        // shows the tip again. Clearing here rather than on the next open is
        // what makes a dismissal not permanent without ever touching the
        // app-wide `focus_visible`.
        if focus_held != state.read(cx).was_focused {
            let keyboard_focus = focus_held && util::focus_visible(cx);
            let leaving_keyboard_focus = state.read(cx).is_focus_open() && !focus_held;
            state.update(cx, |s, cx| {
                let closed = leaving_keyboard_focus && s.close(false);
                s.was_focused = focus_held;
                s.focus_open = keyboard_focus;
                // Either edge ends the previous dismissal session. Clearing
                // on arrival matters when hover was dismissed before focus.
                s.focus_dismissed = false;
                if keyboard_focus {
                    // An immediate focus open replaces a pending hover warmup,
                    // just as React Stately clears its global warmup timeout.
                    s.generation += 1;
                }
                if closed {
                    cx.notify();
                }
            });
            if keyboard_focus {
                let _ = prepare_tooltip_open(&current_tooltip, cx);
                mark_tooltip_open(cx);
            } else if leaving_keyboard_focus {
                start_tooltip_cooldown(&current_tooltip, close_delay, cx);
            }
        }
        let state_snapshot = state.read(cx);
        let focus_open = state_snapshot.is_focus_open();
        let hover_open = state_snapshot.is_open();
        // `trigger="focus"` takes the pointer out of it; `hover` is both, which
        // is React Aria's behaviour for the default.
        let open = match self.trigger {
            TooltipTrigger::Hover => hover_open || focus_open,
            TooltipTrigger::Focus => focus_open,
        };

        let hover_state = state.clone();
        let dismiss_current = current_tooltip.clone();
        let dismiss_tooltip = util::shared(move |cx: &mut App| {
            state.update(cx, |s, cx| {
                if s.close(true) {
                    cx.notify();
                }
            });
            start_tooltip_cooldown(&dismiss_current, close_delay, cx);
            util::DismissResult::Handled
        });
        // Press dismissal belongs to the trigger. The tip is a sibling here;
        // v3 portals it outside the trigger, so pressing the surface itself
        // must not trip `shouldCloseOnPress`.
        let trigger_div = gpui::div()
            .flex()
            .children(self.children)
            .capture_any_mouse_down({
                let dismiss_tooltip = dismiss_tooltip.clone();
                move |_, _, cx| {
                    dismiss_tooltip(cx);
                }
            })
            .on_key_down({
                let dismiss_tooltip = dismiss_tooltip.clone();
                move |_, _, cx| {
                    // RAC wires `onKeyDown: onPressStart` on the trigger: any
                    // key dismisses an already-open tooltip immediately.
                    dismiss_tooltip(cx);
                }
            });
        let hover_enabled = self.trigger == TooltipTrigger::Hover;

        let (phase, overlay_token) = util::overlay_scope(
            window,
            cx,
            element_id::scoped(&key, "tip-phase"),
            open,
            external_presenter.is_none(),
        );
        if using_separate_fallback && !open && phase == util::OverlayPhase::Closed {
            clear_separate_tooltip_failed(window_id, &key, cx);
        }

        let radius = self
            .radius
            .or(theme_radius)
            .unwrap_or_else(|| util::small_radius(cx));
        let background = theme_background.map(|color| color.resolve(cx.colors()));
        let foreground = theme_foreground.map(|color| color.resolve(cx.colors()));

        let using_external_presenter = external_presenter.is_some();
        let trigger_bounds = window
            .use_keyed_state(element_id::scoped(&key, "trigger-bounds"), cx, |_, _| {
                std::rc::Rc::new(std::cell::Cell::new(None::<Bounds<Pixels>>))
            })
            .read(cx)
            .clone();
        let trigger: AnyElement = if let Some(presenter) = external_presenter {
            let lease = lease.expect("external presenter always allocates a lease");
            let request = if phase != util::OverlayPhase::Closed {
                let tip_size = if let Some(render) = self.body.as_ref() {
                    let rendered_body = render(window, cx);
                    let measured = build_tooltip_surface(
                        TooltipSurfaceSpec {
                            key: &key,
                            content: self.content.clone(),
                            body: Some(rendered_body),
                            radius,
                            padding: tip_padding,
                            text_size: tip_text_size,
                            max_width: TOOLTIP_MAX_WIDTH,
                            background,
                            foreground,
                            show_arrow: false,
                            placement: self.placement,
                        },
                        window,
                        cx,
                    )
                    .into_any_element()
                    .layout_as_root(
                        size(
                            gpui::AvailableSpace::MaxContent,
                            gpui::AvailableSpace::MaxContent,
                        ),
                        window,
                        cx,
                    );
                    Some(size(measured.width.ceil(), measured.height.ceil()))
                } else {
                    let measure_cache = window.use_keyed_state(
                        element_id::scoped(&key, "presenter-measure-cache"),
                        cx,
                        |_, _| None::<(TooltipMeasureKey, Size<Pixels>)>,
                    );
                    let hairline_width = {
                        let layout = cx.layout();
                        if layout.overlay_hairline.is_some() {
                            layout.border_width * 2.
                        } else {
                            px(0.)
                        }
                    };
                    let measure_key = TooltipMeasureKey {
                        content: self.content.clone(),
                        padding: tip_padding,
                        text_size: tip_text_size,
                        hairline_width,
                        font: window.text_style().font(),
                        scale_factor_bits: window.scale_factor().to_bits(),
                    };
                    if let Some(cached) = measure_cache
                        .read(cx)
                        .as_ref()
                        .and_then(|(k, sz)| (*k == measure_key).then_some(*sz))
                    {
                        Some(cached)
                    } else {
                        let measured = build_tooltip_surface(
                            TooltipSurfaceSpec {
                                key: &key,
                                content: self.content.clone(),
                                body: None,
                                radius,
                                padding: tip_padding,
                                text_size: tip_text_size,
                                max_width: TOOLTIP_MAX_WIDTH,
                                background,
                                foreground,
                                show_arrow: false,
                                placement: self.placement,
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                        .layout_as_root(
                            size(
                                gpui::AvailableSpace::MaxContent,
                                gpui::AvailableSpace::MaxContent,
                            ),
                            window,
                            cx,
                        );
                        let sz = size(measured.width.ceil(), measured.height.ceil());
                        measure_cache.update(cx, |slot, _| {
                            *slot = Some((measure_key, sz));
                        });
                        Some(sz)
                    }
                };
                let initial_bounds = trigger_bounds.get().unwrap_or_default();
                let window_bounds = window.bounds();
                let trigger_screen_bounds = Bounds {
                    origin: window_bounds.origin + initial_bounds.origin,
                    size: initial_bounds.size,
                };
                let display = resolve_trigger_display(window, trigger_screen_bounds, cx);
                let display_id = display.as_ref().map(|d| d.id());
                let display_bounds = display.map(|d| d.visible_bounds());
                Some(TooltipPresentationRequest {
                    key: key.clone(),
                    content: self.content.clone(),
                    trigger_bounds: initial_bounds,
                    trigger_screen_bounds,
                    window_bounds,
                    display_id,
                    display_bounds,
                    scale_factor: window.scale_factor(),
                    placement: self.placement,
                    offset: resolved_offset,
                    show_arrow: self.show_arrow,
                    radius,
                    padding: tip_padding,
                    text_size: tip_text_size,
                    background,
                    foreground,
                    exclude_from_capture: self.exclude_from_capture,
                    tip_size,
                    body: self.body.take(),
                })
            } else {
                None
            };
            TooltipTriggerPresent {
                child: popover::PopoverTriggerMeasure::new(trigger_div, trigger_bounds.clone())
                    .into_any_element(),
                trigger_bounds: trigger_bounds.clone(),
                presenter,
                lease,
                request,
            }
            .into_any_element()
        } else {
            if let Some(lease) = &lease {
                dismiss_lease(lease, window, cx);
            }
            popover::PopoverTriggerMeasure::new(trigger_div, trigger_bounds.clone())
                .into_any_element()
        };

        let mut wrapper = gpui::div()
            // `on_hover` needs a stateful element, so the wrapper carries the id.
            .id(key.clone())
            .track_focus(&wrap_handle)
            .relative()
            .flex()
            .child(trigger)
            .on_hover(move |over, _window, cx: &mut App| {
                if !hover_enabled {
                    return;
                }
                let over = *over;
                let current = hover_state.downgrade();
                let warmed_up = over && prepare_tooltip_open(&current, cx);
                if !over {
                    // GPUI dispatches sibling hover listeners in reverse paint
                    // order. An outgoing tooltip may run after the incoming
                    // one opened, so only the manager's current entry may cool.
                    start_tooltip_cooldown(&current, close_delay, cx);
                }
                let wait = if over {
                    if warmed_up { 0 } else { delay }
                } else {
                    close_delay
                };
                let generation = hover_state.update(cx, |s, _| {
                    s.generation += 1;
                    s.generation
                });

                if wait == 0 {
                    hover_state.update(cx, |s, cx| {
                        if over {
                            mark_tooltip_open(cx);
                            s.open = true;
                            cx.notify();
                        } else if s.close(true) {
                            cx.notify();
                        }
                    });
                    return;
                }

                let weak = hover_state.downgrade();
                cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                    cx.background_executor()
                        .timer(Duration::from_millis(wait))
                        .await;
                    if let Some(state) = weak.upgrade() {
                        state.update(cx, |s, cx| {
                            // A newer hover transition supersedes this timer.
                            if s.generation == generation {
                                if over {
                                    mark_tooltip_open(cx);
                                    if !s.open {
                                        s.open = true;
                                        cx.notify();
                                    }
                                } else if s.close(true) {
                                    cx.notify();
                                }
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
        let captured_dismiss = dismiss_tooltip.clone();
        util::capture_escape(&overlay_token, move |_window, cx| captured_dismiss(cx), cx);
        wrapper = util::dismiss_on_escape_with_token(wrapper, overlay_token, move |_window, cx| {
            dismiss_tooltip(cx)
        });

        // When an external presenter is active, the surface renders outside this
        // window while HeroGPUI retains all lifecycle timing and dismissal.
        if using_external_presenter {
            return util::apply_sx(wrapper, &self.sx).into_any_element();
        }

        // A tooltip leaves the way every other overlay does: `overlay_scope`
        // keeps it for its exit run, which is what `[data-exiting]` needs to
        // have something to play and gives Escape a stack position.
        //
        // The tip — and the max-content line shaping it is sized from — is
        // only built while it is visible: `shape_line` is the most expensive
        // call in this render, and a closed tooltip has no surface to size.
        if phase != util::OverlayPhase::Closed {
            let (resolved_placement, effective_placement) =
                popover::field_placement_feedback(window, cx, &key, self.placement);
            // The caller's body is built first: it takes `cx` mutably, and the
            // theme reads below hold it borrowed for the rest of this block.
            let body = self.body.take().map(|render| render(window, cx));
            let tip = build_tooltip_surface(
                TooltipSurfaceSpec {
                    key: &key,
                    content: self.content.clone(),
                    body,
                    radius,
                    padding: tip_padding,
                    text_size: tip_text_size,
                    max_width: in_window_tooltip_max_width(window.viewport_size()),
                    background,
                    foreground,
                    show_arrow: self.show_arrow,
                    placement: effective_placement,
                },
                window,
                cx,
            )
            // `tooltip/tooltip.js` renders the RAC `Tooltip`, and
            // `react-aria/dist/private/tooltip/useTooltip.js` is a single
            // `role: 'tooltip'`. Upstream leaves the tip unnamed and points the
            // *trigger*'s `aria-describedby` at it; with no id graph the port names
            // the tip with its own content instead, which is the text that
            // describedby would have resolved to.
            .a11y_named(a11y::Role::Tooltip, &a11y::Name::labelled(self.content));

            // `absolute` does not lift the tip above later siblings in the page,
            // so it has to paint last.
            let (slide_x, slide_y) = entry_offset(effective_placement);
            let zoom = anim::ZoomBox::panel(tip_padding, radius).padding_x(tip_padding);
            let zoom = anim::ZoomBox {
                slide_x: (slide_x != 0.0).then(|| px(slide_x)),
                slide_y: (slide_y != 0.0).then(|| px(slide_y)),
                ..zoom
            };
            let animated = if self.should_skip_animation {
                tip.into_any_element()
            } else if phase == util::OverlayPhase::Exiting {
                anim::exiting(
                    tip,
                    element_id::scoped(&key, "tip-out"),
                    zoom,
                    anim::Motion::LIST_OUT,
                    cx,
                )
            } else {
                // `tooltip.css` is `duration-150 ease-smooth zoom-in-90` — the
                // same zoom as a popover, not a slide.
                anim::entering_zoom(
                    tip,
                    element_id::scoped(&key, "tip"),
                    zoom,
                    anim::Motion::POPOVER_IN,
                    cx,
                )
            };
            let positioner = TooltipPositioner::new(
                trigger_bounds,
                resolved_placement,
                self.placement,
                effective_placement,
                resolved_offset,
                animated,
            );
            wrapper =
                wrapper.child(util::floating(positioner).with_priority(TOOLTIP_DEFERRED_PRIORITY));
        }

        util::apply_sx(wrapper, &self.sx).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size, Bounds};

    use super::{
        arrow_rotation, entry_offset, flip_tooltip_placement, in_window_tooltip_max_width,
        position_tooltip_popup, resolve_tooltip_popup_placement, separate_popup_bounds,
        tooltip_display_content_for_width, tooltip_text_with_break_opportunities,
        SeparateTooltipRegistry, TooltipPlacement,
    };

    #[test]
    fn tooltip_text_adds_breaks_without_changing_whitespace() {
        assert_eq!(
            tooltip_text_with_break_opportunities("longtoken"),
            "l\u{200b}o\u{200b}n\u{200b}g\u{200b}t\u{200b}o\u{200b}k\u{200b}e\u{200b}n"
        );
        assert_eq!(
            tooltip_text_with_break_opportunities("two words\nnext"),
            "t\u{200b}w\u{200b}o w\u{200b}o\u{200b}r\u{200b}d\u{200b}s\nn\u{200b}e\u{200b}x\u{200b}t"
        );
    }

    #[test]
    fn short_tooltips_keep_their_label_while_only_overflowing_words_get_breaks() {
        let char_w = |s: &str| px(s.chars().count() as f32 * 8.);

        // Fits within max width: unchanged, does not wrap.
        assert_eq!(
            tooltip_display_content_for_width("Left", px(40.), px(320.), px(304.), char_w),
            ("Left".to_owned(), false)
        );

        // Multi-word sentence wider than max width, but each individual word fits:
        // wraps cleanly at spaces without inserting zero-width breaks into normal words.
        let (multi, wraps_multi) = tooltip_display_content_for_width(
            "Wrap cleanly at spaces",
            px(220.),
            px(120.),
            px(104.),
            char_w,
        );
        assert!(wraps_multi);
        assert_eq!(multi, "Wrap cleanly at spaces");
        assert!(!multi.contains('\u{200b}'));

        // Mixed sentence with an unbroken URL wider than available width:
        // only the overflowing URL gets zero-width break opportunities.
        let url = "https://example.com/unbroken/path";
        let mixed = format!("Visit {url} now");
        let (display, wraps_mixed) =
            tooltip_display_content_for_width(&mixed, px(400.), px(160.), px(144.), char_w);
        assert!(wraps_mixed);
        assert!(display.starts_with("Visit "));
        assert!(display.ends_with(" now"));
        let middle = display
            .strip_prefix("Visit ")
            .and_then(|s| s.strip_suffix(" now"))
            .unwrap();
        assert!(middle.contains('\u{200b}'));
        assert_eq!(middle.replace('\u{200b}', ""), url);

        // Narrow viewport max-width helper caps to (viewport.width - margin * 2).min(320).
        assert_eq!(
            in_window_tooltip_max_width(size(px(800.), px(600.))),
            px(320.)
        );
        assert_eq!(
            in_window_tooltip_max_width(size(px(200.), px(300.))),
            px(184.)
        );
        assert_eq!(in_window_tooltip_max_width(size(px(12.), px(100.))), px(0.));
    }

    #[test]
    fn entry_offsets_follow_the_tooltip_side() {
        assert_eq!(entry_offset(TooltipPlacement::Top), (0.0, 4.0));
        assert_eq!(entry_offset(TooltipPlacement::TopStart), (0.0, 4.0));
        assert_eq!(entry_offset(TooltipPlacement::TopLeft), (0.0, 4.0));
        assert_eq!(entry_offset(TooltipPlacement::TopEnd), (0.0, 4.0));
        assert_eq!(entry_offset(TooltipPlacement::TopRight), (0.0, 4.0));
        assert_eq!(entry_offset(TooltipPlacement::Bottom), (0.0, -4.0));
        assert_eq!(entry_offset(TooltipPlacement::BottomStart), (0.0, -4.0));
        assert_eq!(entry_offset(TooltipPlacement::BottomLeft), (0.0, -4.0));
        assert_eq!(entry_offset(TooltipPlacement::BottomEnd), (0.0, -4.0));
        assert_eq!(entry_offset(TooltipPlacement::BottomRight), (0.0, -4.0));
        assert_eq!(entry_offset(TooltipPlacement::Left), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::LeftTop), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::LeftBottom), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::Start), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::StartTop), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::StartBottom), (4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::Right), (-4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::RightTop), (-4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::RightBottom), (-4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::End), (-4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::EndTop), (-4.0, 0.0));
        assert_eq!(entry_offset(TooltipPlacement::EndBottom), (-4.0, 0.0));
    }

    #[test]
    #[allow(clippy::float_cmp)] // the rotations are quarter-turn constants
    fn arrow_rotations_face_the_tooltip_side() {
        assert_eq!(arrow_rotation(TooltipPlacement::Top), 0.);
        assert_eq!(arrow_rotation(TooltipPlacement::TopRight), 0.);
        assert_eq!(
            arrow_rotation(TooltipPlacement::Bottom),
            std::f32::consts::PI
        );
        assert_eq!(
            arrow_rotation(TooltipPlacement::BottomLeft),
            std::f32::consts::PI
        );
        assert_eq!(
            arrow_rotation(TooltipPlacement::Left),
            -std::f32::consts::FRAC_PI_2
        );
        assert_eq!(
            arrow_rotation(TooltipPlacement::StartBottom),
            -std::f32::consts::FRAC_PI_2
        );
        assert_eq!(
            arrow_rotation(TooltipPlacement::Right),
            std::f32::consts::FRAC_PI_2
        );
        assert_eq!(
            arrow_rotation(TooltipPlacement::EndTop),
            std::f32::consts::FRAC_PI_2
        );
    }

    #[test]
    fn placement_list_includes_the_supported_aligned_edges() {
        assert_eq!(TooltipPlacement::ALL.len(), 22);
        for placement in [
            TooltipPlacement::TopStart,
            TooltipPlacement::TopLeft,
            TooltipPlacement::TopEnd,
            TooltipPlacement::TopRight,
            TooltipPlacement::BottomStart,
            TooltipPlacement::BottomLeft,
            TooltipPlacement::BottomEnd,
            TooltipPlacement::BottomRight,
            TooltipPlacement::LeftTop,
            TooltipPlacement::LeftBottom,
            TooltipPlacement::RightTop,
            TooltipPlacement::RightBottom,
            TooltipPlacement::Start,
            TooltipPlacement::StartTop,
            TooltipPlacement::StartBottom,
            TooltipPlacement::End,
            TooltipPlacement::EndTop,
            TooltipPlacement::EndBottom,
        ] {
            assert!(TooltipPlacement::ALL.contains(&placement));
            let _ = flip_tooltip_placement(placement);
        }
    }

    #[test]
    fn popup_positioner_flips_on_monitor_edges_and_supports_negative_origins_and_dpi() {
        let monitor = Bounds {
            origin: point(px(-1920.), px(0.)),
            size: size(px(1920.), px(1080.)),
        };
        let popup = size(px(120.), px(32.));

        // Near top edge: `Top` flips below trigger.
        let near_top = Bounds {
            origin: point(px(-1800.), px(8.)),
            size: size(px(40.), px(24.)),
        };
        let (p, resolved) = resolve_tooltip_popup_placement(
            near_top,
            popup,
            monitor,
            TooltipPlacement::Top,
            px(6.),
            1.25,
        );
        assert_eq!(resolved, TooltipPlacement::Bottom);
        // 8 + 24 + 6 = 38; snapped to 1.25x grid (38 * 1.25 = 47.5 -> 48 / 1.25 = 38.4).
        assert_eq!(p.y, px(38.4));
        assert!(f32::from(p.x) >= -1920. && f32::from(p.x) + 120. <= 0.);

        // With arrow overhang on flipped Bottom placement, popup expands upward
        // by 12px so the upward arrow paints unobstructed.
        let arrow_bounds = separate_popup_bounds(p, popup, resolved, true, monitor, 1.25);
        assert_eq!(arrow_bounds.size, size(px(120.), px(44.)));
        assert!(
            (f32::from(arrow_bounds.origin.y) - 26.4).abs() < 1e-4,
            "got {:?}",
            arrow_bounds.origin.y
        );

        // Near left edge: `Left` flips right; near right edge clamps cleanly.
        let near_left = Bounds {
            origin: point(px(-1914.), px(400.)),
            size: size(px(30.), px(24.)),
        };
        let (p_left, resolved_left) = resolve_tooltip_popup_placement(
            near_left,
            popup,
            monitor,
            TooltipPlacement::Left,
            px(6.),
            1.5,
        );
        assert_eq!(resolved_left, TooltipPlacement::Right);
        // -1914 + 30 + 6 = -1878, which aligns cleanly on a 1.5x physical grid (-2817 px).
        assert_eq!(p_left.x, px(-1878.));
        assert!(f32::from(p_left.y) >= 0. && f32::from(p_left.y) + 32. <= 1080.);

        // Clamping a fractional popup width at a monitor edge must stay BOTH
        // strictly inside monitor bounds AND aligned to the 1.5x physical grid.
        let small_mon = Bounds {
            origin: point(px(0.), px(0.)),
            size: size(px(100.), px(100.)),
        };
        let frac_popup = size(px(30.2), px(20.));
        let right_trig = Bounds {
            origin: point(px(85.), px(40.)),
            size: size(px(15.), px(20.)),
        };
        let p_frac = position_tooltip_popup(
            right_trig,
            frac_popup,
            small_mon,
            TooltipPlacement::BottomStart,
            px(4.),
            1.5,
        );
        let px_x = f32::from(p_frac.x) * 1.5;
        assert!(
            (px_x - px_x.round()).abs() < 1e-4,
            "expected grid alignment, got {px_x}"
        );
        assert!(
            f32::from(p_frac.x) + 30.2 <= 100.0 + 1e-5,
            "clamped coordinate must remain inside monitor: {p_frac:?}"
        );

        // Near bottom-right corner on a compact monitor clamps cleanly inside bounds.
        let tiny_monitor = Bounds {
            origin: point(px(0.), px(0.)),
            size: size(px(100.), px(60.)),
        };
        let corner_trig = Bounds {
            origin: point(px(70.), px(40.)),
            size: size(px(24.), px(16.)),
        };
        let p_clamped = position_tooltip_popup(
            corner_trig,
            popup,
            tiny_monitor,
            TooltipPlacement::BottomEnd,
            px(4.),
            2.0,
        );
        assert_eq!(p_clamped, point(px(0.), px(4.)));
    }

    fn set_force_open_failure_for_test(cx: &mut gpui::App, fail: bool) {
        cx.default_global::<SeparateTooltipRegistry>()
            .force_open_failure_for_test = fail;
    }

    #[gpui::test]
    fn separate_window_failure_falls_back_inline_then_recovers_after_close(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::{prelude::*, AnyElement, Context, Render, Window};

        struct TestView(Box<dyn Fn(&mut Window, &mut Context<'_, Self>) -> AnyElement>);
        impl Render for TestView {
            fn render(
                &mut self,
                window: &mut Window,
                cx: &mut Context<'_, Self>,
            ) -> impl IntoElement {
                (self.0)(window, cx)
            }
        }

        cx.update(herogpui_theme::ThemeProvider::init);
        cx.update(|cx| set_force_open_failure_for_test(cx, true));

        let (_, vcx) = cx.add_window_view(|_, _| {
            TestView(Box::new(|_, _| {
                gpui::div()
                    .size_full()
                    .p(px(16.))
                    .flex()
                    .items_start()
                    .child(
                        super::Tooltip::new("First line\nSecond line\r\nThird line")
                            .id("fallback-tip")
                            .presenter_mode(super::TooltipPresenterMode::SeparateWindow)
                            .delay(0)
                            .close_delay(0)
                            .should_skip_animation(true)
                            .body(|_, _| {
                                gpui::div()
                                    .debug_selector(|| "fallback-inline-body".into())
                                    .w(px(140.))
                                    .h(px(36.))
                                    .child("Inline fallback body")
                                    .into_any_element()
                            })
                            .child(
                                gpui::div()
                                    .debug_selector(|| "fallback-trigger".into())
                                    .w(px(60.))
                                    .h(px(28.))
                                    .child("Trigger"),
                            ),
                    )
                    .into_any_element()
            }))
        });

        vcx.run_until_parked();
        let trigger = vcx
            .debug_bounds("fallback-trigger")
            .expect("trigger bounds");

        // Hover while popup opening fails: must fall back cleanly to InWindow tip
        // without creating a popup window.
        vcx.simulate_mouse_move(trigger.center(), None, gpui::Modifiers::none());
        vcx.run_until_parked();
        vcx.update(|window, _| window.refresh());
        vcx.run_until_parked();
        assert_eq!(vcx.update(|_, cx| cx.windows().len()), 1);
        assert!(
            vcx.debug_bounds("fallback-inline-body").is_some(),
            "inline fallback surface must render inside the owner window when PopUp fails"
        );

        // Move pointer off so tooltip closes cleanly and clears the failure latch.
        vcx.simulate_mouse_move(point(px(400.), px(400.)), None, gpui::Modifiers::none());
        vcx.run_until_parked();
        vcx.background_executor
            .advance_clock(std::time::Duration::from_millis(200));
        vcx.run_until_parked();
        vcx.update(|window, _| window.refresh());
        vcx.run_until_parked();

        // Allow popup windows again and re-hover: SeparateWindow must recover.
        vcx.update(|_, cx| set_force_open_failure_for_test(cx, false));
        vcx.simulate_mouse_move(trigger.center(), None, gpui::Modifiers::none());
        vcx.run_until_parked();
        vcx.update(|window, _| window.refresh());
        vcx.run_until_parked();
        assert_eq!(
            vcx.update(|_, cx| cx.windows().len()),
            2,
            "cleared failure latch should reopen SeparateWindow on next hover session"
        );
    }
}

crate::util::impl_component_styled!(Tooltip);
