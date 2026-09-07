//! Painted scrollbar — the GPUI stand-in for HeroUI's CSS `scrollbar` token.
//!
//! GPUI's `overflow_*_scroll` moves content and `scrollbar_width` only
//! reserves gutter space; it paints nothing. This overlay reads a
//! [`ScrollHandle`] and draws a thumb in `--scrollbar`. Overlay versus
//! always-visible follows [`gpui::App::should_auto_hide_scrollbars`].

use std::time::{Duration, Instant};

use gpui::{
    canvas, div, prelude::*, px, App, Bounds, ElementId, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, Pixels, Point, RenderOnce, ScrollHandle, Window,
};
use herogpui_core::{element_id, Orientation};
use herogpui_theme::ActiveTheme;

const THUMB_MIN: f32 = 24.0;
const TRACK: f32 = 8.0;
const IDLE: Duration = Duration::from_millis(800);

#[derive(Clone, Copy)]
struct Drag {
    pointer: f32,
    offset: f32,
}

#[derive(Clone)]
struct BarState {
    drag: Option<Drag>,
    hover: bool,
    last_offset: f32,
    last_moved: Option<Instant>,
}

impl Default for BarState {
    fn default() -> Self {
        Self {
            drag: None,
            hover: false,
            last_offset: 0.0,
            last_moved: None,
        }
    }
}

/// A painted overlay thumb bound to one [`ScrollHandle`].
#[derive(IntoElement)]
pub struct Scrollbar {
    id: ElementId,
    handle: ScrollHandle,
    orientation: Orientation,
}

impl Scrollbar {
    pub fn new(id: impl Into<ElementId>, handle: ScrollHandle) -> Self {
        Self {
            id: id.into(),
            handle,
            orientation: Orientation::Vertical,
        }
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let handle = self.handle;
        let horizontal = self.orientation.is_horizontal();
        let max = if horizontal {
            f32::from(handle.max_offset().x)
        } else {
            f32::from(handle.max_offset().y)
        };
        if max < 1.0 {
            return div().into_any_element();
        }

        let offset = if horizontal {
            f32::from(handle.offset().x)
        } else {
            f32::from(handle.offset().y)
        };
        let state = window.use_keyed_state(element_id::scoped(&self.id, "bar"), cx, |_, _| {
            BarState::default()
        });
        let snapshot = state.read(cx).clone();
        if (snapshot.last_offset - offset).abs() > 0.5 {
            state.update(cx, |s, _| {
                s.last_offset = offset;
                s.last_moved = Some(Instant::now());
            });
        }

        let auto_hide = cx.should_auto_hide_scrollbars();
        let live = state.read(cx);
        let recently_moved = live.last_moved.is_some_and(|at| at.elapsed() < IDLE);
        let show_thumb = !auto_hide || live.hover || live.drag.is_some() || recently_moved;
        let thumb = cx.colors().scrollbar;
        let axis = self.orientation;
        let handle_move = handle.clone();
        let drag_state = state.clone();
        let hover_state = state.clone();
        let leave_state = state.clone();

        let mut track = div()
            .id(self.id)
            .absolute()
            .overflow_hidden()
            .when(horizontal, |el| {
                el.left_0().right_0().bottom_0().h(px(TRACK))
            })
            .when(!horizontal, |el| {
                el.top_0().bottom_0().right_0().w(px(TRACK))
            })
            .on_mouse_down(MouseButton::Left, {
                let handle = handle.clone();
                let state = state.clone();
                move |ev: &MouseDownEvent, window, cx| {
                    jump_or_grab(&handle, axis, ev.position, &state, window, cx);
                }
            })
            .on_mouse_move({
                let handle = handle_move;
                let state = drag_state;
                move |ev: &MouseMoveEvent, window, cx| {
                    drag_to(&handle, axis, ev.position, &state, window, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let state = state.clone();
                move |_, _, cx| {
                    state.update(cx, |s, cx| {
                        s.drag = None;
                        cx.notify();
                    });
                }
            })
            .on_hover(move |inside, _, cx| {
                if *inside {
                    hover_state.update(cx, |s, cx| {
                        s.hover = true;
                        cx.notify();
                    });
                } else {
                    leave_state.update(cx, |s, cx| {
                        s.hover = false;
                        cx.notify();
                    });
                }
            });

        if show_thumb {
            let paint_handle = handle;
            track = track.child(
                canvas(
                    move |bounds, _, _| bounds,
                    move |bounds, _, window, _| {
                        if let Some(thumb_bounds) = thumb_bounds(&paint_handle, axis, bounds) {
                            window.paint_quad(
                                gpui::fill(thumb_bounds, thumb).corner_radii(px(TRACK / 2.)),
                            );
                        }
                    },
                )
                .size_full(),
            );
        }

        track.into_any_element()
    }
}

fn axis_size(bounds: Bounds<Pixels>, horizontal: bool) -> f32 {
    if horizontal {
        f32::from(bounds.size.width)
    } else {
        f32::from(bounds.size.height)
    }
}

fn axis_origin(bounds: Bounds<Pixels>, horizontal: bool) -> f32 {
    if horizontal {
        f32::from(bounds.origin.x)
    } else {
        f32::from(bounds.origin.y)
    }
}

fn thumb_metrics(
    handle: &ScrollHandle,
    axis: Orientation,
    track: Bounds<Pixels>,
) -> Option<(f32, f32, f32)> {
    let horizontal = axis.is_horizontal();
    let max = if horizontal {
        f32::from(handle.max_offset().x)
    } else {
        f32::from(handle.max_offset().y)
    };
    if max < 1.0 {
        return None;
    }
    let offset = if horizontal {
        f32::from(handle.offset().x)
    } else {
        f32::from(handle.offset().y)
    };
    let track_len = axis_size(track, horizontal);
    if track_len < 1.0 {
        return None;
    }
    let content = track_len + max;
    let thumb_len = (track_len * (track_len / content))
        .max(THUMB_MIN)
        .min(track_len);
    let travel = (track_len - thumb_len).max(0.0);
    let t = (-offset / max).clamp(0.0, 1.0);
    Some((thumb_len, travel * t, max))
}

fn thumb_bounds(
    handle: &ScrollHandle,
    axis: Orientation,
    track: Bounds<Pixels>,
) -> Option<Bounds<Pixels>> {
    let (thumb_len, thumb_start, _) = thumb_metrics(handle, axis, track)?;
    let horizontal = axis.is_horizontal();
    Some(if horizontal {
        Bounds {
            origin: Point {
                x: track.origin.x + px(thumb_start),
                y: track.origin.y,
            },
            size: gpui::Size {
                width: px(thumb_len),
                height: track.size.height,
            },
        }
    } else {
        Bounds {
            origin: Point {
                x: track.origin.x,
                y: track.origin.y + px(thumb_start),
            },
            size: gpui::Size {
                width: track.size.width,
                height: px(thumb_len),
            },
        }
    })
}

fn jump_or_grab(
    handle: &ScrollHandle,
    axis: Orientation,
    pointer: Point<Pixels>,
    state: &gpui::Entity<BarState>,
    window: &mut Window,
    cx: &mut App,
) {
    let track = handle.bounds();
    let Some((thumb_len, thumb_start, max)) = thumb_metrics(handle, axis, track) else {
        return;
    };
    let horizontal = axis.is_horizontal();
    let pointer_v = if horizontal {
        f32::from(pointer.x)
    } else {
        f32::from(pointer.y)
    };
    let origin = axis_origin(track, horizontal);
    let rel = pointer_v - origin;
    let offset = if horizontal {
        f32::from(handle.offset().x)
    } else {
        f32::from(handle.offset().y)
    };
    if rel >= thumb_start && rel <= thumb_start + thumb_len {
        state.update(cx, |s, cx| {
            s.drag = Some(Drag {
                pointer: pointer_v,
                offset,
            });
            cx.notify();
        });
        return;
    }
    let travel = (axis_size(track, horizontal) - thumb_len).max(1.0);
    let t = ((rel - thumb_len / 2.0) / travel).clamp(0.0, 1.0);
    set_axis_offset(handle, axis, -t * max);
    state.update(cx, |s, cx| {
        s.last_offset = -t * max;
        s.last_moved = Some(Instant::now());
        cx.notify();
    });
    let _ = window;
}

fn drag_to(
    handle: &ScrollHandle,
    axis: Orientation,
    pointer: Point<Pixels>,
    state: &gpui::Entity<BarState>,
    _window: &mut Window,
    cx: &mut App,
) {
    let Some(drag) = state.read(cx).drag else {
        return;
    };
    let track = handle.bounds();
    let Some((thumb_len, _, max)) = thumb_metrics(handle, axis, track) else {
        return;
    };
    let horizontal = axis.is_horizontal();
    let pointer_v = if horizontal {
        f32::from(pointer.x)
    } else {
        f32::from(pointer.y)
    };
    let travel = (axis_size(track, horizontal) - thumb_len).max(1.0);
    let delta = (pointer_v - drag.pointer) / travel * max;
    let next = (drag.offset - delta).clamp(-max, 0.0);
    set_axis_offset(handle, axis, next);
    state.update(cx, |s, cx| {
        s.last_offset = next;
        s.last_moved = Some(Instant::now());
        cx.notify();
    });
}

fn set_axis_offset(handle: &ScrollHandle, axis: Orientation, value: f32) {
    let mut point = handle.offset();
    if axis.is_horizontal() {
        point.x = px(value);
    } else {
        point.y = px(value);
    }
    handle.set_offset(point);
}
