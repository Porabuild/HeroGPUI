//! `VirtualList` (HeroGPUI extension): variable row heights are measured,
//! only rows near the viewport are built, and `scroll_to_item` moves a row
//! into view under both strategies.

mod harness;

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use gpui::{prelude::*, px, TestAppContext, VisualTestContext};
use harness::open_host;
use herogpui_components::{VirtualList, VirtualListHandle, VirtualListScroll};

const ROWS: usize = 1_000;
const VIEWPORT: f32 = 300.;

/// Row `ix` is 20px tall, or 60px for every fifth row.
fn row_height(ix: usize) -> f32 {
    if ix.is_multiple_of(5) {
        60.
    } else {
        20.
    }
}

fn host(
    cx: &mut TestAppContext,
) -> (
    VirtualListHandle,
    Rc<RefCell<BTreeSet<usize>>>,
    &mut VisualTestContext,
) {
    let handle = VirtualListHandle::new(ROWS);
    let built = Rc::new(RefCell::new(BTreeSet::new()));
    let (h, b) = (handle.clone(), built.clone());
    let cx = open_host(cx, move || {
        let b = b.clone();
        VirtualList::new("vl", &h, move |ix, _, _| {
            b.borrow_mut().insert(ix);
            gpui::div()
                .h(px(row_height(ix)))
                .w_full()
                .into_any_element()
        })
        .height(px(VIEWPORT))
        .into_any_element()
    });
    (handle, built, cx)
}

fn frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
}

#[gpui::test]
fn only_rows_near_the_viewport_are_built(cx: &mut TestAppContext) {
    let (handle, built, cx) = host(cx);
    frame(cx);
    let built = built.borrow();
    assert!(built.contains(&0));
    assert!(built.len() < 60, "built {} of {ROWS} rows", built.len());
    assert!(!built.contains(&(ROWS - 1)));
    // Variable heights are measured, not assumed uniform.
    let first = handle.bounds_for_item(0).unwrap();
    let second = handle.bounds_for_item(1).unwrap();
    assert_eq!(first.size.height, px(60.));
    assert_eq!(second.size.height, px(20.));
    assert_eq!(second.origin.y, first.origin.y + px(60.));
}

#[gpui::test]
fn scroll_to_item_top_puts_the_row_at_the_top(cx: &mut TestAppContext) {
    let (handle, _, cx) = host(cx);
    frame(cx);
    handle.scroll_to_item(500, VirtualListScroll::Top);
    frame(cx);
    assert_eq!(handle.first_visible_item(), 500);
    let row = handle.bounds_for_item(500).expect("row 500 rendered");
    assert_eq!(row.origin.y, px(0.));
}

#[gpui::test]
fn scroll_to_item_reveal_brings_the_row_fully_into_view(cx: &mut TestAppContext) {
    let (handle, _, cx) = host(cx);
    frame(cx);
    // A row laid out in the overdraw band just below the viewport scrolls
    // the least distance: it lands on the bottom edge.
    let below = (0..ROWS)
        .find(|&ix| {
            handle
                .bounds_for_item(ix)
                .is_some_and(|b| b.bottom() > px(VIEWPORT))
        })
        .unwrap();
    handle.scroll_to_item(below, VirtualListScroll::Reveal);
    frame(cx);
    let row = handle.bounds_for_item(below).unwrap();
    assert!((f32::from(row.bottom()) - VIEWPORT).abs() < 0.5, "{row:?}");
    // An already visible row does not move.
    handle.scroll_to_item(below, VirtualListScroll::Reveal);
    frame(cx);
    assert_eq!(
        handle.bounds_for_item(below).unwrap().origin.y,
        row.origin.y
    );
    // A never-measured row far below still ends fully visible (at the top).
    handle.scroll_to_item(700, VirtualListScroll::Reveal);
    frame(cx);
    let far = handle.bounds_for_item(700).expect("row 700 rendered");
    assert!(
        far.origin.y >= px(0.) && far.bottom() <= px(VIEWPORT),
        "{far:?}"
    );
}

#[gpui::test]
fn item_count_changes_are_honoured(cx: &mut TestAppContext) {
    let (handle, built, cx) = host(cx);
    frame(cx);
    handle.set_item_count(3);
    built.borrow_mut().clear();
    frame(cx);
    assert_eq!(handle.item_count(), 3);
    assert_eq!(*built.borrow(), BTreeSet::from([0, 1, 2]));
    handle.splice(3..3, 2);
    frame(cx);
    assert_eq!(handle.item_count(), 5);
    // Out-of-range targets clamp to the last row instead of panicking.
    handle.scroll_to_item(99, VirtualListScroll::Top);
    frame(cx);
    assert!(handle.bounds_for_item(4).is_some());
}
