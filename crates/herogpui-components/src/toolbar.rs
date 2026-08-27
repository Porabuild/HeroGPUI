//! Toolbar — port of `@heroui/toolbar`.
//!
//! A container for interactive controls with arrow key navigation. Mirrors the
//! React API: `orientation` and `isAttached`. The keyboard contract is the
//! pinned React Aria `useToolbar` (react-aria 3.51.0): the orientation's axis
//! moves between the controls inside the toolbar and is consumed at the ends
//! without wrapping, Tab (Shift+Tab) leaves the *entire* toolbar in one press,
//! and the child the focus left from is restored when the focus comes back. A
//! toolbar nested inside another is a group: pinned detects it with
//! `parentElement.closest('[role="toolbar"]')` and binds it none of the above,
//! so the enclosing toolbar's manager walks straight across its children.

use std::collections::HashMap;

use gpui::{
    div, px, AnyElement, App, BorrowAppContext, ElementId, FocusHandle, InteractiveElement,
    IntoElement, KeyDownEvent, ParentElement, Pixels, RenderOnce, Styled, WeakFocusHandle, Window,
};
use herogpui_core::Orientation;
use herogpui_theme::ActiveTheme;

/// The focus bookkeeping pinned `useToolbar` does with its `lastFocused` ref:
/// which child held the focus while it was inside the subtree, and which child
/// the focus left from. Keyed state, because a per-render cell is one frame
/// long and a focus move is a cross-event fact — the same rule the Slider's
/// drag flag learnt.
#[derive(Clone, Default, PartialEq)]
struct ToolbarFocusEdge {
    /// Whether the subtree held the focus as of the last observed frame.
    inside: bool,
    /// The child that held the focus as of the last inside frame.
    child: Option<FocusHandle>,
    /// The child the focus left from; restored on the next entry.
    last_focused: Option<FocusHandle>,
}

/// Every mounted toolbar's scope handle, per window.
///
/// Pinned `useToolbar` answers "am I nested?" at mount with
/// `ref.current.parentElement.closest('[role="toolbar"]')` — a DOM ancestor
/// query. gpui has no render-time ancestor query, but the *most recently
/// rendered frame*'s dispatch tree is one: `FocusHandle::contains` walks a
/// handle's real element ancestors. This registry supplies the other toolbars'
/// scopes to ask about. Entries are weak and pruned whenever a toolbar
/// renders, so a toolbar that leaves the tree drops out together with its
/// keyed state and nothing accumulates across routes.
#[derive(Default)]
struct ToolbarScopes {
    per_window: HashMap<gpui::WindowId, Vec<WeakFocusHandle>>,
}

impl gpui::Global for ToolbarScopes {}

/// Registers `scope` for this window and reports whether another toolbar's
/// scope contains it in the most recently rendered frame — this port's
/// `closest('[role="toolbar"]')`. The registration dedups because every frame
/// renders every mounted toolbar again.
fn sync_toolbar_scope(scope: &FocusHandle, window: &Window, cx: &mut App) -> bool {
    if cx.try_global::<ToolbarScopes>().is_none() {
        cx.set_global(ToolbarScopes::default());
    }
    let window_id = window.window_handle().window_id();
    cx.update_global::<ToolbarScopes, _>(|scopes, _| {
        let entry = scopes.per_window.entry(window_id).or_default();
        entry.retain(|weak| weak.upgrade().is_some());
        if !entry.iter().any(|weak| weak == scope) {
            entry.push(scope.downgrade());
        }
        entry
            .iter()
            .filter_map(|weak| weak.upgrade())
            .any(|other| other != *scope && other.contains(scope, window))
    })
}

/// HeroUI Toolbar.
#[derive(IntoElement)]
pub struct Toolbar {
    id: Option<ElementId>,
    orientation: Orientation,
    is_attached: bool,
    gap: Option<Pixels>,
    children: Vec<AnyElement>,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            id: None,
            orientation: Orientation::Horizontal,
            is_attached: false,
            gap: None,
            children: Vec::new(),
        }
    }

    /// Names this instance. Two toolbars on one page share their keyed focus
    /// state unless each carries a distinct id — the same silent sharing two
    /// same-named tag groups suffered.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// `isAttached` — renders the toolbar as a fully-rounded surface that hugs
    /// its controls.
    pub fn is_attached(mut self, v: bool) -> Self {
        self.is_attached = v;
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into());
        self
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // `.toolbar` is `gap-2`, attached or not — the sheet's attached
        // variant restates only rounding, fill, padding and shadow.
        let gap = self.gap.unwrap_or(px(8.));

        let base = self
            .id
            .clone()
            .unwrap_or_else(|| ElementId::Name("toolbar-focus".into()));
        let scope = window
            .use_keyed_state(
                ElementId::Name(format!("{base:?}-scope").into()),
                cx,
                |_, cx| cx.focus_handle(),
            )
            .read(cx)
            .clone();
        let edge = window.use_keyed_state(
            ElementId::Name(format!("{base:?}-edge").into()),
            cx,
            |_, _| ToolbarFocusEdge::default(),
        );

        // Pinned `useToolbar` decides at mount whether it is nested, and a
        // nested toolbar renders `role="group"` with its
        // `onKeyDownCapture`/`onFocusCapture`/`onBlurCapture` all undefined —
        // no keyboard or focus management of its own, because the enclosing
        // toolbar's manager walks across its children. The render-time answer
        // reads the previous frame's dispatch tree, so it gates only the
        // render-time focus bookkeeping below; the key handler re-checks at
        // event time against the frame the event actually dispatched against.
        let nested = sync_toolbar_scope(&scope, window, cx);

        // Pinned `useToolbar` records the child the focus leaves from (the
        // Tab branch and the blur-capture) and restores it when the focus
        // re-enters from outside (the focus-capture). GPUI's observer APIs
        // cannot carry that here: their focus paths are blanked for inactive
        // windows — the headless test platform among them — so the edge is
        // observed at render time, the way `util::on_focus_leave`'s render
        // half is. A focus move always invalidates the window, so the frame
        // after a move sees the new state. The update is guarded because the
        // keyed entity notifies its observers on every write: an unconditional
        // write would render forever.
        if !nested {
            let current = edge.read(cx).clone();
            let mut next = current.clone();
            if scope.contains_focused(window, cx) {
                if !next.inside {
                    // Entry from outside: hand the focus to the recorded
                    // child and clear the record, as pinned does. Pinned's
                    // own comment: "If the element was removed, do nothing,
                    // either the first item in the first group, or the last
                    // item in the last group will be focused, depending on
                    // direction." A child that left the frame has no node in
                    // the rendered dispatch tree, so `contains` is the
                    // removal test — its `.focus()` must not run at all, and
                    // the entry's own landing is kept.
                    if let Some(last) = next.last_focused.take() {
                        if scope.contains(&last, window) {
                            window.focus(&last);
                        }
                    }
                }
                next.child = window.focused(cx);
                next.inside = true;
            } else if next.inside {
                next.last_focused = next.child.take();
                next.inside = false;
            }
            if next != current {
                edge.update(cx, |slot, _| *slot = next);
            }
        }

        // The tokens are cloned out of `cx` before any later `&mut` use.
        let colors = cx.colors();
        let surface = colors.surface.background;
        let overlay_shadow = cx.layout().overlay_shadow.clone();

        // `.toolbar` is `grid w-fit grid-flow-col items-center gap-2`, and
        // `.toolbar--vertical` overrides the flow with
        // `grid-flow-row items-start justify-start` — a column whose controls
        // hug the start edge. The horizontal toolbar keeps the base rule's
        // centered cross axis.
        let mut el = div().key_context("Toolbar").flex().gap(gap);

        el = match self.orientation {
            Orientation::Horizontal => el.flex_row().items_center(),
            Orientation::Vertical => el.flex_col().items_start().justify_start(),
        };

        if self.is_attached {
            // `.toolbar--attached` is `p-1 rounded-3xl bg-surface shadow-overlay`.
            // The sheet gives a floating surface no border; light mode
            // separates it with the overlay shadow alone.
            el = el
                .p(px(4.))
                .rounded(crate::util::control_radius(cx))
                .bg(surface)
                .shadow(overlay_shadow);
        }

        el = el.track_focus(&scope);
        // Pinned `useToolbar` handles exactly the orientation's axis —
        // ArrowRight/ArrowLeft when horizontal, ArrowDown/ArrowUp when
        // vertical — through a FocusManager scoped to the toolbar's element
        // whose walk has `wrap` unset, so an arrow at either end moves
        // nothing; the handler still stops propagation and prevents default,
        // so the end is a *consumed* stop.
        //
        // gpui's `focus_next`/`focus_prev` cannot say that: their stop order
        // is window-wide and **wraps around** at the window's ends (a step
        // with no successor restarts from the far end), so a lone toolbar
        // would read its own far child as a legal move. The pinned walker
        // would refuse both, so the window's first and last stops are probed
        // once per key — from no focus, `focus_next` answers the first stop
        // and `focus_prev` the last — and a step at an end is refused. A step
        // that lands outside the subtree is undone by giving the focus back
        // to the child that held it.
        //
        // The probe's temporary blur and refocus have no side effects to
        // fence off, which is source-backed in gpui 0.2.2: focus and blur
        // listeners are dispatched only at frame end, when `draw` compares
        // the previous and current focus paths (and the probe's net move is
        // zero whenever an end refuses it), and a refocus only clears the
        // window's *pending multi-stroke input* — this app binds no
        // multi-stroke keys, so there is never anything to clear. Held-key
        // auto-repeat (`is_held`) runs the same path as a distinct press and
        // `toolbar_held_arrow_repeats_stop_at_ends` pins that.
        let vertical = self.orientation == Orientation::Vertical;
        el.on_key_down(move |event: &KeyDownEvent, window, cx| {
            // The event-time form of the render-time `nested` above, read
            // against the frame this key actually dispatched against: a
            // toolbar mounted one frame ago answered from a tree that did
            // not yet contain it, but by the time a key can reach its
            // handler the frame has painted and the enclosing toolbar is
            // visible. Pinned's nested toolbar binds no `onKeyDownCapture`
            // at all — returning without consuming is exactly that.
            if sync_toolbar_scope(&scope, window, cx) {
                return;
            }
            let key = event.keystroke.key.as_str();
            if key != "tab" {
                let forward = match (vertical, key) {
                    (false, "right") | (true, "down") => true,
                    (false, "left") | (true, "up") => false,
                    // Everything else returns early so it is not consumed:
                    // the perpendicular arrows stay with the children and the
                    // app root, Home/End and an enclosing scroller still see
                    // them.
                    _ => return,
                };
                cx.stop_propagation();
                let held = window.focused(cx);
                let Some(held) = held else {
                    return;
                };
                window.blur();
                window.focus_next();
                let first_stop = window.focused(cx);
                window.blur();
                window.focus_prev();
                let last_stop = window.focused(cx);
                window.focus(&held);
                let at_end = if forward {
                    last_stop.as_ref() == Some(&held)
                } else {
                    first_stop.as_ref() == Some(&held)
                };
                if !at_end {
                    if forward {
                        window.focus_next();
                    } else {
                        window.focus_prev();
                    }
                    if !scope.contains_focused(window, cx) {
                        window.focus(&held);
                    }
                }
                return;
            }
            // Tab: pinned runs `focusFirst` (Shift) or `focusLast`, then lets
            // the native Tab carry on from that end — one press leaves the
            // entire toolbar from any child. There is no native Tab here;
            // the app root owns one step of it, so the walk is ours: step
            // until the focus leaves the subtree, refusing the wrap at the
            // window's far end (a native Tab from the document's last
            // focusable goes nowhere either). Stopping propagation also
            // skips the root's `set_focus_visible`, so set it here — a Tab
            // that moves without ringing looks like it did nothing. The
            // record of the leaving child was made by the frames while it
            // held the focus; the next render sees the departure.
            cx.stop_propagation();
            crate::util::set_focus_visible(true, cx);
            let back = event.keystroke.modifiers.shift;
            let Some(held) = window.focused(cx) else {
                return;
            };
            window.blur();
            window.focus_next();
            let first_stop = window.focused(cx);
            window.blur();
            window.focus_prev();
            let last_stop = window.focused(cx);
            window.focus(&held);
            for _ in 0..256 {
                // The far end of the *window* is the end of the walk: a
                // native Tab from the document's last focusable goes
                // nowhere, so neither does this.
                let at_end = if back {
                    first_stop.as_ref() == window.focused(cx).as_ref()
                } else {
                    last_stop.as_ref() == window.focused(cx).as_ref()
                };
                if at_end {
                    return;
                }
                if back {
                    window.focus_prev();
                } else {
                    window.focus_next();
                }
                if !scope.contains_focused(window, cx) {
                    return;
                }
            }
        })
        .children(self.children)
    }
}
