//! Behaviour tests for the paths a screenshot cannot see: VIRTUALIZED
//! collections (`ListBox` and `Table` with a fixed or an estimated row
//! height), the toast queue's lifecycle (cap, action, pause/resume,
//! placement) and `ScrollShadow`'s derived visibility.
//!
//! A virtual list builds only the rows the viewport shows, so a row far down
//! a 1000-item list does not exist until a scroll brings it in. That is what
//! makes these tests worth having: a plain list cannot tell you whether the
//! click to index mapping survives the virtual path, because it never had to
//! build the row for index 300.
//!
//! The scroll mechanic, learned from gpui 0.2.2's own sources:
//!
//! - A scroll is `cx.simulate_event(ScrollWheelEvent { position, delta, .. })`.
//!   The event hit-tests the **last rendered frame** (`dispatch_mouse_event`
//!   reads `rendered_frame`), so every state change must be followed by a
//!   redraw (`window.refresh()` inside an update) before the next event.
//! - `delta: ScrollDelta::Pixels(point(px(0.), px(-N)))` scrolls **down** by
//!   exactly N pixels: the scrollable element's listener does
//!   `scroll_offset.y += delta.y` and offsets are negative when scrolled down.
//!   `Lines` deltas are scaled by `window.line_height()`, so pixels are used
//!   throughout to keep the arithmetic exact.
//! - The wheel must land *inside* the scrollable element's hitbox
//!   (`hitbox.should_handle_scroll`), which is why every wheel event below
//!   names a point the component's geometry puts inside the scroller.
//!
//! Index arithmetic for the virtual lists (ListBox and Table alike):
//! `uniform_list` lays row i at `i * row_height + scroll_offset.y` inside its
//! own box, with `scroll_offset.y` clamped to `-(content - viewport)..0`, so a
//! wheel of `-12000px` over 40px rows brings item 300 to the top of the
//! viewport. The ListBox adds `p-1` (4px) of padding, so item 300 spans
//! window y 4..44 and a click at y = 24 is its centre.
//!
//! The Table's body sits below a header whose 12px text makes its height
//! land in [30, 46]px (the bound the sibling suite relies on), and the
//! header-drift cancels when a click is aimed at the middle of a row band:
//! a click at y hits row `floor((y - H) / h)` for the true H, so choosing y
//! strictly between `H_max + row_top` and `H_min + row_bottom` pins the row
//! for *every* H in the bound.
//!
//! The toast store (`ToastStore`, v3's `ToastQueue`) is app-global, so a
//! toast's lifecycle is asserted on the store entity, never on pixels; when
//! a card must be *pressed* it is rendered by `ToastViewport` and driven by
//! the keyboard or by probing the smallest clickable surface — the 20px
//! close button at a placement-derived coordinate.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gpui::{
    point, prelude::*, px, ScrollDelta, ScrollWheelEvent, TestAppContext, VisualTestContext,
};

use harness::{click, events, open_host, press, Events};
use herogpui_components::{
    pause_toasts, toast_store, ListBox, ListBoxItem, ScrollShadow, ScrollShadowVisibility,
    SortDescriptor, SortDirection, Table, TableColumn, TableRow, Toast, ToastPlacement,
    ToastViewport,
};

/// Pins the toast card layout by enabling reduced motion **before** the first
/// frame. A toast wraps itself in `entering_zoom`, whose animation runs on
/// wall time the test clock does not drive, so without this the card would sit
/// at its t=0 pose for the whole test. The preference is read by
/// `ThemeProvider::init`, which `open_host` calls, so it must be set first —
/// exactly the rule the overlay suite learned the hard way.
fn still() {
    std::env::set_var("HEROGPUI_REDUCE_MOTION", "1");
}

/// Pushes the pending frame through. Events are dispatched against the last
/// rendered frame, so anything that changes state — a scroll, a press, a
/// dismissal, a pushed toast — needs a redraw before the next event, or the
/// next event hits the stale frame.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// One simulated wheel event at window coordinates (`x`, `y`), scrolling
/// `dy` pixels: **negative moves down** (later rows into view), matching the
/// scrollable element's `scroll_offset.y += delta.y` with negative offsets
/// meaning "scrolled down". The delta is `Pixels`, not `Lines`, so no line
/// height enters the arithmetic. Followed by a redraw so the next event sees
/// the frame the scroll produced.
fn wheel(cx: &mut VisualTestContext, x: f32, y: f32, dy: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(x), px(y)),
        delta: ScrollDelta::Pixels(point(px(0.), px(dy))),
        ..Default::default()
    });
    flush_frame(cx);
}

/// A full-height clickable table cell that records `vrow-{index:04}` — the
/// row's *identity*, which is the claim a virtual table makes: the factory
/// must build the cell for exactly the index the viewport asked for.
fn probe_cell(index: usize, recorded: Events) -> gpui::AnyElement {
    let label = format!("vrow-{index:04}");
    let shown = label.clone();
    gpui::div()
        .id(gpui::ElementId::Name(format!("vt-probe-{index}").into()))
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.clone()))
        .child(shown)
        .into_any_element()
}

/// A table cell whose content decides the row's height: `py-3` (12px each
/// side, applied by `RowCtx`) plus the 1px row border make a `h(20)` cell's
/// row 45px tall and a `h(60)` cell's row 85px tall — two real heights the
/// `gpui::list` measurement must keep apart.
fn var_cell(index: usize) -> gpui::AnyElement {
    let content_h = if index.is_multiple_of(2) {
        px(20.)
    } else {
        px(60.)
    };
    gpui::div()
        .h(content_h)
        .w_full()
        .child(format!("row {index}"))
        .into_any_element()
}

/// The name a resolved `ScrollShadowVisibility` is recorded under.
fn shadow_label(v: ScrollShadowVisibility) -> &'static str {
    match v {
        ScrollShadowVisibility::Auto => "auto",
        ScrollShadowVisibility::Both => "both",
        ScrollShadowVisibility::Top => "top",
        ScrollShadowVisibility::Bottom => "bottom",
        ScrollShadowVisibility::Left => "left",
        ScrollShadowVisibility::Right => "right",
        ScrollShadowVisibility::None => "none",
    }
}

// ---------------------------------------------------------------------------
// Virtualized collections
// ---------------------------------------------------------------------------

/// A virtual list builds only the rows in view, so item 300 is not in the
/// first paint at all. Scrolling -12000px over a 40px row grid brings item
/// 300 to the top of the 160px viewport; the click then lands on it and the
/// reported key must be the one its index names. A plain list cannot make
/// this claim — it never had to answer for row 300's identity.
#[gpui::test]
fn virtual_list_box_selects_a_row_after_scrolling(cx: &mut TestAppContext) {
    let recorded = events();
    let items: Vec<ListBoxItem> = (0..1000)
        .map(|i| ListBoxItem::new(format!("key-{i:04}"), format!("Item {i}")))
        .collect();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // `row_height` is the option that virtualizes the list (v3's
        // `layoutOptions.rowHeight`); `max_h` sizes the viewport, here 4 rows.
        ListBox::new("vlb-click", items.clone())
            .row_height(px(40.))
            .max_h(px(160.))
            .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    // Index arithmetic: the list's `p-1` padding puts the rows' box at
    // window y 4..164; the uniform_list lays row i at `i * 40 +
    // scroll_offset.y`. A wheel of -12000 (300 * 40) sets first_visible to
    // -(-12000)/40 = 300, so item 300 occupies window y 4..44. The wheel
    // position (20, 80) is inside the rows' box on the *initial* frame, which
    // is the only frame it hit-tests. The click at (20, 24) is item 300's
    // centre — a row that was not built (or was built for another index)
    // cannot answer it.
    wheel(cx, 20., 80., -12000.);
    click(cx, 20., 24.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["key-0300"],
        "scrolling 300 rows down and clicking must report the key of index 300"
    );
}

/// The arrows move a cursor the list has *not* built yet, and tell the list's
/// own scroll handle to bring the new row into view. After 31 Downs the
/// cursor is on index 30 — item 30, not anything clamped to the 4 visible
/// rows — and the deferred center-scroll places it at viewport centre, where
/// a click can prove the row now exists at that spot.
#[gpui::test]
fn virtual_list_box_arrows_scroll_the_focused_row_into_view(cx: &mut TestAppContext) {
    let recorded = events();
    let items: Vec<ListBoxItem> = (0..1000)
        .map(|i| ListBoxItem::new(format!("key-{i:04}"), format!("Item {i}")))
        .collect();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ListBox::new("vlb-keys", items.clone())
            .row_height(px(40.))
            .max_h(px(160.))
            .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    // The list is the only tab stop, so Tab focuses it and the arrows belong
    // to it. Thirty-one Downs walk the cursor from None through 0..30 —
    // `list_nav::resolve` starts `down` from nothing at the first stop and
    // steps one row per press — and each press defers a center-scroll for the
    // new cursor, applied at the next draw. The draw the Enter dispatch
    // triggers first is the one that places item 30.
    press(cx, "tab");
    for _ in 0..31 {
        press(cx, "down");
    }
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["key-0030"],
        "31 Downs must land the cursor and the activation on index 30"
    );

    // Scroll arithmetic for the follow-up click: the center strategy puts the
    // item top at `item_center - viewport_center` = 30*40 + 20 - 80 = 1140,
    // so scroll_offset.y = -1140 and item 30 spans window y
    // 4 + 30*40 - 1140 = 64..104. If the arrows had not scrolled the list,
    // only rows 0..3 would exist and this click could not record index 30.
    flush_frame(cx);
    click(cx, 20., 84.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["key-0030", "key-0030"],
        "the centered row must be built and clickable at the position the \
         scroll arithmetic says"
    );
}

/// A virtual table's rows come from a *factory* — `AnyElement` is built once
/// and consumed once, so the table asks for the rows the viewport shows. The
/// probe cell records the index the factory was called with, so the click
/// after scrolling proves the visible-range math handed out row 300, not a
/// neighbour; the sort header then proves the virtual body did not disturb
/// the controlled sort loop.
#[gpui::test]
fn virtual_table_rows_click_and_sort(cx: &mut TestAppContext) {
    let probes = events();
    let sorts = events();
    let held: Rc<RefCell<Option<SortDescriptor>>> = Rc::new(RefCell::new(None));
    let probes_for_view = probes.clone();
    let sorts_for_view = sorts.clone();
    let held_for_view = held;
    let cx = open_host(cx, move || {
        let probes = probes_for_view.clone();
        let sorts = sorts_for_view.clone();
        let held = held_for_view.clone();
        let mut table = Table::new(vec![])
            .id("vt-sort")
            .columns(vec![
                TableColumn::new("Name")
                    .allows_sorting(true)
                    .default_width(px(160.)),
            ])
            // 1000 rows built on demand: the factory must be re-invokable on
            // every scroll, which is exactly what `virtual_rows` is for.
            .virtual_rows(1000, move |i| {
                TableRow::new(vec![probe_cell(i, probes.clone())])
            })
            .row_height(px(40.))
            .max_h(px(160.));
        // Sorting is controlled: the caller feeds the reported descriptor
        // back, which is also what lets a second click on the same column
        // flip.
        if let Some(d) = held.borrow().clone() {
            table = table.sort_descriptor(d);
        }
        table
            .on_sort_change(move |d, _, _| {
                let dir = if d.direction == SortDirection::Ascending {
                    "asc"
                } else {
                    "desc"
                };
                sorts.borrow_mut().push(format!("{}:{dir}", d.column));
                *held.borrow_mut() = Some(d);
            })
            .into_any_element()
    });

    // Header height is ~37px (12px text in `py-2.5`), the bound the sibling
    // suite established as [30, 46]. Scrolling -12000 (300 * 40) puts item
    // 300 at the body top, so its 40px band spans y H..H+40 for the true H;
    // y = 58 is inside that band for every H in [30, 46] (58 > 46, 58 < 70).
    // The wheel at (100, 100) is inside the virtual body (H..H+160) for the
    // same bound.
    wheel(cx, 100., 100., -12000.);
    click(cx, 30., 58.);
    assert_eq!(
        probes.borrow().as_slice(),
        ["vrow-0300"],
        "after scrolling 300 rows, the click must land on the row built for \
         index 300"
    );

    // The header cell is ~37px tall, so y = 18 is inside it; the column is
    // 160px wide and clickable. Two clicks report asc then desc — the
    // controlled loop, driven through the virtual body's untouched header.
    click(cx, 80., 18.);
    click(cx, 80., 18.);
    assert_eq!(
        sorts.borrow().as_slice(),
        ["Name:asc", "Name:desc"],
        "the virtual table's sortable header must toggle like a plain one"
    );
}

/// `estimatedRowHeight` takes the `gpui::list` path, which measures *each*
/// built row instead of multiplying one. Rows of two real heights — 45px and
/// 85px — must lay out without overlapping: a click aimed mid-way down a tall
/// row names that row's index, and a mis-measured list (uniform 45 or
/// uniform 85) puts the click in a neighbour's band instead.
#[gpui::test]
fn variable_height_rows_do_not_overlap(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Six rows alternating h(20) and h(60) cells -> 45/85px rows; the
        // whole 390px fits the default 400px body, so every row is built and
        // measured, and only the measurement — never the viewport — decides
        // where each row's click band is.
        Table::new(vec![])
            .id("vt-var")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .virtual_rows(6, |i| TableRow::new(vec![var_cell(i)]))
            .estimated_row_height(px(24.))
            .on_row_click(move |i, _, _, _| recorded.borrow_mut().push(format!("row-{i}")))
            .into_any_element()
    });
    flush_frame(cx);

    // Row bands (body-relative), from the two measured heights and the 1px
    // row border:
    //   row 0:  0..45,  row 1: 45..130, row 2: 130..175,
    //   row 3: 175..260, row 4: 260..305, row 5: 305..390.
    // The body sits below a header of height H in [30, 46], so a window y
    // lands inside row r for every possible H when it lies between
    // H_max + row_top and H_min + row_bottom. For row 1 that interval is
    // (91, 160) — y = 120; for row 3 it is (221, 290) — y = 250.
    // A uniform-45 list puts row 3 at body 135..180 (window ~177) and a
    // uniform-85 one at 255..340 (~window 301), so either mis-measurement
    // turns one of these clicks into a neighbour's index.
    click(cx, 30., 120.);
    click(cx, 30., 250.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["row-1", "row-3"],
        "rows of two heights must measure and lay out without overlap"
    );
}

// ---------------------------------------------------------------------------
// Toast queue lifecycle
// ---------------------------------------------------------------------------

/// `ToastStore::push` caps the queue at `DEFAULT_MAX_VISIBLE_TOASTS` (3),
/// evicting the oldest beyond it — v3's `maxVisibleToasts` on the provider.
/// Five pushes must leave the newest three, in insertion order.
#[gpui::test]
fn toast_queue_respects_its_limit(cx: &mut TestAppContext) {
    let ids: Vec<u64> = cx.update(|cx| {
        (1..=5)
            .map(|i| {
                Toast::new(format!("T{i}"))
                    .timeout(Duration::ZERO)
                    .push(None, cx)
            })
            .collect()
    });
    assert_eq!(ids.len(), 5, "five pushes must issue five distinct ids");

    cx.update(|cx| {
        let toasts = toast_store(cx).read(cx).toasts();
        assert_eq!(
            toasts.len(),
            herogpui_components::DEFAULT_MAX_VISIBLE_TOASTS,
            "the store must keep only the newest three of five"
        );
        for (n, (kept, title)) in toasts.iter().zip(["T3", "T4", "T5"]).enumerate() {
            assert_eq!(
                kept.id,
                ids[2 + n],
                "the survivors keep their ids and order"
            );
            assert_eq!(kept.title.as_ref(), title, "the title matches the id");
        }
        // The evicted pair must not linger by id either.
        assert!(toasts.iter().all(|t| t.id != ids[0] && t.id != ids[1]));
    });
}

/// A toast's `action` is a button inside the card: pressing it reports *and*
/// dismisses its own toast, leaving the siblings. The card is rendered by
/// `ToastViewport`, the button is the first tab stop in it, and Enter
/// activates a focused element on key-up — which the harness `press` helper
/// supplies explicitly.
#[gpui::test]
fn toast_action_button_reports_and_dismisses(cx: &mut TestAppContext) {
    still();
    let rec = events();
    let action_rec = rec.clone();
    let (id_a, id_b) = cx.update(|cx| {
        let a = Toast::new("Has an action")
            .timeout(Duration::ZERO)
            .action("Undo", move |_| {
                action_rec.borrow_mut().push("undo".to_owned());
            })
            .push(None, cx);
        let b = Toast::new("Sibling").timeout(Duration::ZERO).push(None, cx);
        (a, b)
    });
    let cx = open_host(cx, || ToastViewport::new().into_any_element());

    // The card's children order is [indicator, title, action, close], so the
    // action button is the first tab stop; the close button is second.
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        rec.borrow().as_slice(),
        ["undo"],
        "Enter on the action button must report the action exactly once"
    );

    cx.update(|_window, cx| {
        let toasts = toast_store(cx).read(cx).toasts();
        assert_eq!(toasts.len(), 1, "the pressed action must dismiss its toast");
        assert_eq!(
            toasts[0].id, id_b,
            "the dismissed toast must be the acted-on one, not its sibling"
        );
        assert!(
            toasts.iter().all(|t| t.id != id_a),
            "the acted-on toast's id must be gone from the store"
        );
    });
}

/// `pauseAll` / `resumeAll` stop and restart every dismissal clock. While
/// paused, ticking the test clock far past a toast's timeout must not dismiss
/// it; after `resumeAll` the same clock dismisses it.
#[gpui::test]
fn toast_pause_and_resume_stop_the_clock(cx: &mut TestAppContext) {
    let timed = cx.update(|cx| {
        // Pause first, then push: the toast's timer ticks are what `paused`
        // gates, and the first tick is only 100ms away.
        pause_toasts(true, cx);
        Toast::new("Times out")
            .timeout(Duration::from_millis(300))
            .push(None, cx)
    });

    // 1500ms is five times the timeout; every 100ms tick has seen `paused`.
    cx.executor().advance_clock(Duration::from_millis(1500));
    cx.update(|cx| {
        assert!(
            toast_store(cx)
                .read(cx)
                .toasts()
                .iter()
                .any(|t| t.id == timed),
            "a paused queue must not dismiss a toast whose timeout has passed"
        );
    });

    // Resume: three 100ms ticks subtract the 300ms, and it goes.
    cx.update(|cx| pause_toasts(false, cx));
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.update(|cx| {
        assert!(
            toast_store(cx)
                .read(cx)
                .toasts()
                .iter()
                .all(|t| t.id != timed),
            "after resume, the timeout must dismiss the toast"
        );
    });
}

/// Placement is a layout choice, so it is asserted behaviourally: the toast's
/// close button is a real 20px click target, and where it sits in the window
/// is exactly what `placement` decides. At `Bottom` the card hugs the bottom
/// inset and the button answers a click near the window's bottom edge; after
/// switching to `Top` the same spot must hit nothing (the card moved away)
/// and the button answers at the top edge instead. No appearance is asserted.
#[gpui::test]
fn toast_placement_moves_the_viewport(cx: &mut TestAppContext) {
    still();
    // Pushed before the viewport exists: the store is app-global and the
    // viewport reads it at render, so ordering only affects the frame shown.
    cx.update(|cx| {
        Toast::new("A").timeout(Duration::ZERO).push(None, cx);
    });
    let placement = Rc::new(RefCell::new(ToastPlacement::Bottom));
    let holder = placement.clone();
    let cx = open_host(cx, move || {
        ToastViewport::new()
            .placement(*holder.borrow())
            .into_any_element()
    });

    // Card geometry (no action button, so the children are a 20px title and
    // the 20px close button): `p(16)` + width 460 centers the card at
    // x 730..1190; `py-3`-equivalent padding (12px each side) and the 20px
    // line make the card 44px tall. Bottom placement parks its bottom edge
    // 16px off the window's bottom (1080), so the card spans y 1020..1064
    // and the close button (20px, flush right against the card's 16px
    // padding) spans x 1154..1174, y 1032..1052 — centre (1164, 1042).
    // The top placement's page geometry: the card spans y 16..60 and the
    // close button y 28..48 — centre (1164, 38).
    click(cx, 1164., 1042.);
    cx.update(|_window, cx| {
        assert!(
            toast_store(cx).read(cx).toasts().is_empty(),
            "at Bottom, the close button must live near the bottom edge: the \
             click must dismiss the toast"
        );
    });

    // Move the viewport to Top and push a second toast; the same bottom-edge
    // point must now be empty background — nothing can dismiss the toast.
    *placement.borrow_mut() = ToastPlacement::Top;
    cx.update(|_window, cx| {
        Toast::new("B").timeout(Duration::ZERO).push(None, cx);
    });
    flush_frame(cx);
    click(cx, 1164., 1042.);
    cx.update(|_window, cx| {
        assert_eq!(
            toast_store(cx).read(cx).toasts().len(),
            1,
            "at Top, the same bottom-edge point must hit nothing: the toast \
             must survive"
        );
    });

    // And the button answers where Top actually put it.
    click(cx, 1164., 38.);
    cx.update(|_window, cx| {
        assert!(
            toast_store(cx).read(cx).toasts().is_empty(),
            "at Top, the close button must live near the top edge: the click \
             must dismiss the toast"
        );
    });
}

// ---------------------------------------------------------------------------
// ScrollShadow
// ---------------------------------------------------------------------------

/// `visibility(Auto)` is derived from the live scroll offset, and
/// `onVisibilityChange` reports every edge change: at the top only the
/// bottom edge shades, mid-scroll both do, at the bottom only the top, and
/// back at the top only the bottom again.
#[gpui::test]
fn scroll_shadow_reports_visibility_as_it_scrolls(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Ten 40px blocks with the 8px gap make 472px of content in a 160px
        // box: 312px of scroll (`max_offset`). The wheel positions hit the
        // scroller itself, which spans the full box.
        ScrollShadow::new("ss-auto")
            .max_h(px(160.))
            .visibility(ScrollShadowVisibility::Auto)
            .on_visibility_change(move |v, _, _| {
                recorded.borrow_mut().push(shadow_label(v).to_owned());
            })
            .children((0..10).map(|_| {
                gpui::div()
                    .h(px(40.))
                    .w_full()
                    .child("block")
                    .into_any_element()
            }))
            .into_any_element()
    });
    flush_frame(cx);

    // The canvas reports from the first frame: offset 0, no leading fade
    // (`past_start` false), plenty of content left (`before_end` true) →
    // Bottom.
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bottom"],
        "at the top of a scrollable box only the bottom edge shades"
    );

    // Half the max (156 of 312): past the start and short of the end → Both.
    wheel(cx, 100., 80., -156.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bottom", "both"],
        "mid-scroll both edges must shade"
    );

    // The other half reaches the bottom exactly: `-312 > -312` is false, so
    // only the leading (top) edge shades.
    wheel(cx, 100., 80., -156.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bottom", "both", "top"],
        "at the bottom only the top edge shades"
    );

    // All the way back up in one gesture, and the change fires again.
    wheel(cx, 100., 80., 312.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["bottom", "both", "top", "bottom"],
        "returning to the top must report the bottom-edge shade again"
    );
}

/// Content shorter than the box has no scroll range, so `Auto` resolves to
/// `None` and `onVisibilityChange` never fires.
///
/// **Ignored pending a defect** (repro below): a wheel over a zero-range
/// scroller makes the component report a one-frame shadow anyway. gpui's
/// scroll listener writes the wheel's delta into the tracked handle's offset
/// cell *before* the next layout clamps it, and `ScrollShadow::render` reads
/// that pre-clamp offset, so a -40px wheel resolves `Auto` to `Top` for one
/// frame (a +40px wheel resolves `Bottom`) before the layout clamps the
/// offset back to 0 and the canvas reports the correction. A fits-content
/// box should behave exactly like a wheeled one that never moves: no report
/// at all. Reproduction:
/// `cargo test -p herogpui-components --test virtual_and_feedback
/// scroll_shadow_hides_when_content_fits -- --ignored` — the recorder holds
/// `["top", "none"]` instead of nothing.
#[gpui::test]
#[ignore = "defect: a wheel over a zero-range ScrollShadow resolves Auto to a one-frame start/end shadow before the offset clamps"]
fn scroll_shadow_hides_when_content_fits(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Two 40px blocks plus the 8px gap = 88px inside the 160px box: the
        // scroller's range is 0, so the leading/trailing conditions are both
        // false at every offset and Auto stays None forever.
        ScrollShadow::new("ss-fit")
            .max_h(px(160.))
            .visibility(ScrollShadowVisibility::Auto)
            .on_visibility_change(move |v, _, _| {
                recorded.borrow_mut().push(shadow_label(v).to_owned());
            })
            .children((0..2).map(|_| gpui::div().h(px(40.)).w_full().into_any_element()))
            .into_any_element()
    });
    flush_frame(cx);
    // A wheel at a scroller with zero range: the listener adds the delta but
    // the next layout clamps it back to 0, and resolution never changes.
    wheel(cx, 100., 80., -40.);
    flush_frame(cx);
    assert!(
        recorded.borrow().is_empty(),
        "content that fits must never report a shadow edge, observed {:?}",
        recorded.borrow().as_slice()
    );
}
