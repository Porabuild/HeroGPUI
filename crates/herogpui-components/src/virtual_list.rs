//! `VirtualList` — a vertical list that renders only the rows in (and just
//! around) the viewport, with **variable row heights** and programmatic
//! scrolling to a row (HeroGPUI extension; HeroUI v3 has no such component).
//!
//! It wraps GPUI's own `list` element, which measures each row the first time
//! it is laid out and keeps a height summary, so rows need no declared
//! height. The fixed-height fast path that `Table`, `ListBox` and `ComboBox`
//! use (`uniform_list`, via their `row_height`) stays where it is: those
//! components keep their existing virtualisation, because rewiring them onto
//! a variable-height primitive would change their measured layout.
//!
//! State lives in a caller-owned [`VirtualListHandle`], which is how a view
//! scrolls the list from outside (`scroll_to_item`) and tells it that the
//! collection changed (`set_item_count`, `splice`):
//!
//! ```
//! use herogpui_components::{VirtualListScroll, VirtualList, VirtualListHandle};
//! use gpui::{div, prelude::*, px};
//!
//! struct Log {
//!     rows: Vec<String>,
//!     list: VirtualListHandle,
//! }
//!
//! impl Log {
//!     fn new(rows: Vec<String>) -> Self {
//!         let list = VirtualListHandle::new(rows.len());
//!         Self { rows, list }
//!     }
//!
//!     fn jump_to_end(&self) {
//!         // Clamped to the last row; an empty list stays put.
//!         self.list.scroll_to_item(usize::MAX, VirtualListScroll::Reveal);
//!     }
//!
//!     fn view(&self) -> VirtualList {
//!         let rows = self.rows.clone();
//!         VirtualList::new("log", &self.list, move |ix, _window, _cx| {
//!             div().p(px(4.)).child(rows[ix].clone()).into_any_element()
//!         })
//!         .height(px(240.))
//!     }
//! }
//! ```

use gpui::{
    div, list, prelude::*, AnyElement, App, Bounds, ElementId, ListAlignment, ListOffset,
    ListState, Pixels, Window,
};

/// Where [`VirtualListHandle::scroll_to_item`] places the row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VirtualListScroll {
    /// Scroll the least distance that makes the row fully visible; a row
    /// already fully visible does not move. The distance needs the heights
    /// of the rows in between, so a row that was not laid out in the last
    /// frame (far outside the viewport, never measured) is brought to the
    /// top edge instead, as [`Top`](Self::Top) does.
    #[default]
    Reveal,
    /// Put the row's top edge at the top of the viewport.
    Top,
}

/// The scroll and measurement state of one [`VirtualList`]. Cheap to clone;
/// clones share the state. Keep one per list in the owning view.
#[derive(Clone)]
pub struct VirtualListHandle {
    state: ListState,
}

impl std::fmt::Debug for VirtualListHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualListHandle")
            .field("item_count", &self.item_count())
            .finish()
    }
}

/// Rows rendered beyond each edge of the viewport, so a short scroll shows
/// already-laid-out rows instead of a blank band.
const OVERDRAW: f32 = 200.;

impl VirtualListHandle {
    /// A handle for a list of `item_count` rows, scrolled to the top.
    pub fn new(item_count: usize) -> Self {
        Self {
            state: ListState::new(item_count, ListAlignment::Top, gpui::px(OVERDRAW)),
        }
    }

    /// The number of rows the list renders.
    pub fn item_count(&self) -> usize {
        self.state.item_count()
    }

    /// Replaces the collection: `item_count` rows, all remeasured, scrolled
    /// back to the top. For an append or a local edit, [`splice`](Self::splice)
    /// keeps the scroll position and the other rows' measurements.
    pub fn set_item_count(&self, item_count: usize) {
        self.state.reset(item_count);
    }

    /// Replaces the rows in `old_range` with `count` new rows, keeping every
    /// other row's measured height and the scroll position.
    pub fn splice(&self, old_range: std::ops::Range<usize>, count: usize) {
        self.state.splice(old_range, count);
    }

    /// Discards every measured height, for when row content changed size
    /// without the count changing (a font or density switch).
    pub fn remeasure(&self) {
        self.state.remeasure();
    }

    /// Scrolls so row `ix` is visible, per `strategy`. An index past the end
    /// scrolls to the end. Takes effect on the next frame.
    pub fn scroll_to_item(&self, ix: usize, strategy: VirtualListScroll) {
        let ix = ix.min(self.item_count().saturating_sub(1));
        match strategy {
            VirtualListScroll::Reveal if self.state.bounds_for_item(ix).is_some() => {
                self.state.scroll_to_reveal_item(ix);
            }
            VirtualListScroll::Reveal | VirtualListScroll::Top => {
                self.state.scroll_to(ListOffset {
                    item_ix: ix,
                    offset_in_item: gpui::px(0.),
                });
            }
        }
    }

    /// Scrolls by `distance` (positive moves the content up, towards later rows).
    pub fn scroll_by(&self, distance: Pixels) {
        self.state.scroll_by(distance);
    }

    /// The index of the first row at the top of the viewport.
    pub fn first_visible_item(&self) -> usize {
        self.state.logical_scroll_top().item_ix
    }

    /// Row `ix`'s window-coordinate bounds from the last frame, when it was
    /// rendered.
    pub fn bounds_for_item(&self, ix: usize) -> Option<Bounds<Pixels>> {
        self.state.bounds_for_item(ix)
    }
}

type RenderRow = dyn FnMut(usize, &mut Window, &mut App) -> AnyElement;

/// A virtualised, variable-row-height vertical list. See the
/// [module docs](self).
#[derive(IntoElement)]
pub struct VirtualList {
    id: ElementId,
    handle: VirtualListHandle,
    render_row: Box<RenderRow>,
    height: Option<Pixels>,
}

impl VirtualList {
    /// A list over `handle` whose row `ix` is `render_row(ix, ..)`. Only
    /// rows near the viewport are built, each frame, so `render_row` should
    /// be cheap and must not assume it sees every index.
    pub fn new(
        id: impl Into<ElementId>,
        handle: &VirtualListHandle,
        render_row: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            handle: handle.clone(),
            render_row: Box::new(render_row),
            height: None,
        }
    }

    /// A fixed viewport height. Without one the list fills its parent's
    /// height (`size_full`), which then needs a definite height itself.
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = Some(height.into());
        self
    }
}

impl RenderOnce for VirtualList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let rows = list(self.handle.state, self.render_row).size_full();
        let root = div().id(self.id).w_full().flex().flex_col();
        match self.height {
            Some(height) => root.h(height),
            None => root.size_full(),
        }
        .child(rows)
    }
}
