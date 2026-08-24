//! Behaviour tests for the three surfaces the earlier suite left half driven:
//! the **Table**'s keyboard and its composed parts (sortable headers,
//! horizontal scroll, footer), the **Tabs** overflow scroller and its
//! separator, and the **Accordion**'s modes (multiple-expand, disabled keys,
//! and `DisclosureGroup`'s bodies).
//!
//! `tests/table_and_drag.rs` already covers Table sorting by click, row
//! selection, select-all, resize, load-more and virtualization; `tests/
//! collections.rs` covers ListBox/TagGroup/Tabs/Accordion basics. Nothing here
//! duplicates those — every test drives a path no earlier test did, and every
//! assertion is behavioural (recorded callbacks, or a probe click that must
//! record nothing), never appearance.
//!
//! Geometry is derived from the components' own constants, not guessed, and
//! every number carries its arithmetic in a comment:
//!
//! - Table rows are made exactly 105px tall (`tall_cell`: an `h(80)` filler
//!   plus `py-3` twice and a 1px border), the header spans 30..46px (12px
//!   text in a `py-2.5` cell), and the 44px checkbox column puts row *i*'s
//!   checkbox centre at y = 52 + 105i below the header.
//! - A sortable header is `py-2.5` around a 12px line; the 320px wrapper in
//!   the sort test pins the two flexing headers to 156px each.
//! - Tab labels are *measured* with the window's own text system, exactly as
//!   `tests/collections.rs` does; a tab is `px-4` (16px each side) around its
//!   14px label and the list carries `p-1` (4px).
//! - The Tabs scroller (`overflow-x-auto` in v3's sheet) only overflows when
//!   its list is `w-max` inside a bounded box; the chevrons are 16px circles
//!   at the box's edges (`start-1`/`end-1`, vertically centred), and the
//!   canvas that measures `max_offset` needs a few frames before they appear.
//! - An accordion trigger is `px-4 py-4` (16px all round) around one 20px
//!   line: 52px, with a 1px separator between items. An open body is
//!   `pt-2 pb-4` + its content, so a fixed 40px content makes everything
//!   below shift by an exact 58px.
//! - A `DisclosureGroup` trigger is a md `Button` (`h-9`, 36px) stretched to
//!   the group width, and an open body is `p-2` (8px all round) around its
//!   content.
//!
//! Two regressions pin defects the suite found: the Table's horizontal content
//! must exceed its viewport, and the Tabs chevrons must occlude the tabs below.

mod harness;

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gpui::{
    point, prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, ScrollDelta,
    ScrollWheelEvent, SharedString, TestAppContext, VisualTestContext,
};
use herogpui_components::{
    Accordion, AccordionItem, DisclosureGroup, Pagination, SelectionMode, SortDescriptor,
    SortDirection, TabItem, Table, TableColumn, Tabs,
};

use harness::{click, events, open_host, press, Events};

/// A single 80px-tall table cell, so rows have an exact height: 80 + `py-3`
/// (12px each side) + a 1px row border = 105px, which is what every row-based
/// y in this file derives from.
fn tall_cell(text: impl Into<SharedString>) -> gpui::AnyElement {
    gpui::div()
        .h(px(80.))
        .flex()
        .items_center()
        .child(text.into())
        .into_any_element()
}

/// A full-row-clickable cell that records `label`, used as the column probe:
/// the click lands inside whichever column owns that x at that moment, so the
/// recorded label names the column behaviourally.
fn probe_cell(id: &'static str, label: &'static str, recorded: Events) -> gpui::AnyElement {
    gpui::div()
        .id(id)
        .w_full()
        .h(px(80.))
        .flex()
        .items_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.to_owned()))
        .child(label)
        .into_any_element()
}

/// A fixed-size clickable box recording `label`, for a click target whose
/// geometry is its own rather than a cell's (the footer probe).
fn box_probe(
    id: &'static str,
    label: &'static str,
    w: f32,
    h: f32,
    recorded: Events,
) -> gpui::AnyElement {
    gpui::div()
        .id(id)
        .w(px(w))
        .h(px(h))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .on_click(move |_, _, _| recorded.borrow_mut().push(label.to_owned()))
        .child(label)
        .into_any_element()
}

/// The advance width of `text` shaped the way the components shape it: gpui's
/// default `.SystemUIFont` stack at `size` px and `weight` (copied from
/// `tests/collections.rs` — the labels are laid out by the window's own
/// `WindowTextSystem`, so this measurement is the render's measurement).
fn text_width(system: &gpui::WindowTextSystem, text: &str, size: f32, weight: FontWeight) -> f32 {
    let run = gpui::TextRun {
        len: text.len(),
        font: Font {
            family: ".SystemUIFont".into(),
            features: FontFeatures::default(),
            weight,
            style: FontStyle::default(),
            fallbacks: None,
        },
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = system.shape_line(text.to_owned().into(), px(size), &[run], None);
    f32::from(line.width)
}

/// The keys of an expanded set joined in a stable order.
///
/// A `HashSet` iterates in no particular order, so asserting on a raw join
/// would be flaky; sorting makes the recorded report deterministic.
fn sorted_join(keys: &HashSet<SharedString>) -> String {
    let mut keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
    keys.sort();
    keys.join(",")
}

/// Pushes the pending frame through. Mouse and wheel events hit-test the
/// *last rendered frame*, so anything that changes state — a scroll, an
/// accordion header opening — needs a redraw before the next event, or the
/// next event lands on the stale layout.
fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// One horizontal wheel at window coordinates (`x`, `y`), scrolling `dx`
/// pixels: **negative moves the content left** (later columns into view),
/// matching the scrollable element's `scroll_offset.x += delta.x` with
/// negative offsets meaning "scrolled right". The delta is `Pixels`, not
/// `Lines`, so no line height enters the arithmetic. Followed by a redraw so
/// the next event sees the frame the scroll produced.
fn wheel_h(cx: &mut VisualTestContext, x: f32, y: f32, dx: f32) {
    cx.simulate_event(ScrollWheelEvent {
        position: point(px(x), px(y)),
        delta: ScrollDelta::Pixels(point(px(dx), px(0.))),
        ..Default::default()
    });
    flush_frame(cx);
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

/// The table body is one tab stop with a cursor inside it: v3's grid roves a
/// single stop over the rows, and this port's `list_nav` resolver walks the
/// same four keys every list-shaped control does. The cursor itself is
/// internal, so the proof is behavioural: Tab reaches the table, arrows move
/// the cursor without reporting, and Enter/Space activate the row the cursor
/// sits on (Home/End jump the cursor).
#[gpui::test]
fn table_keyboard_rows_rove_and_activate(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // No sortable columns, so the wrapper (which holds the body's one tab
        // stop) is the only stop on the page and a single Tab lands on it.
        Table::new(vec![])
            .id("tbl-keys")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .keyed_row("beta", vec![tall_cell("Beta")])
            .keyed_row("gamma", vec![tall_cell("Gamma")])
            .on_row_click(move |i, _, _, _| recorded.borrow_mut().push(format!("row:{i}")))
            .into_any_element()
    });

    // Tab: root -> wrapper (the only stop). Down moves the cursor to row 0;
    // Enter activates it. Down again to row 1; Space activates it. Home jumps
    // the cursor back to row 0; End jumps it to the last row.
    press(cx, "tab");
    press(cx, "down");
    press(cx, "enter");
    press(cx, "down");
    press(cx, "space");
    press(cx, "home");
    press(cx, "enter");
    press(cx, "end");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["row:0", "row:1", "row:0", "row:2"],
        "Enter and Space must activate the row the cursor sits on, and Home \
         and End must jump the cursor — with `from = None` an activation must \
         be ignored, so no key before the first arrow can report anything"
    );
}

/// A sortable header is its own tab stop (the port's reading of "one stop per
/// sortable column"), and gpui fires a *focused* element's click listeners on
/// Enter and Space. The descriptor reported by the keys must be exactly the
/// one a click reports: same column flips, feeding the result back continues
/// the cycle.
#[gpui::test]
fn table_sortable_header_answers_enter_space_then_click(cx: &mut TestAppContext) {
    let recorded = events();
    let held: Rc<RefCell<Option<SortDescriptor>>> = Rc::new(RefCell::new(None));
    let held_for_view = held;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        let held_view = held_for_view.clone();
        // Two sortable columns at 160px inside a 320px wrapper: each flexing
        // header cell is ~156px wide, so the click at x = 80 lands in column
        // 0. Sorting is controlled: the caller feeds the descriptor back,
        // which is what makes the second activation on the same column flip.
        let mut table = Table::new(vec![])
            .id("tbl-sort-keys")
            .columns(vec![
                TableColumn::new("Name")
                    .allows_sorting(true)
                    .default_width(px(160.)),
                TableColumn::new("Size")
                    .allows_sorting(true)
                    .default_width(px(160.)),
            ])
            .row(vec![gpui::div().child("A").into_any_element()])
            .row(vec![gpui::div().child("B").into_any_element()]);
        if let Some(d) = held_view.borrow().clone() {
            table = table.sort_descriptor(d);
        }
        gpui::div()
            .w(px(320.))
            .child(
                table
                    .on_sort_change(move |d, _, _| {
                        let dir = if d.direction == SortDirection::Ascending {
                            "asc"
                        } else {
                            "desc"
                        };
                        recorded.borrow_mut().push(format!("{}:{dir}", d.column));
                        *held_view.borrow_mut() = Some(d);
                    })
                    .into_any_element(),
            )
            .into_any_element()
    });

    // Tab 1 lands on the wrapper (the body stop), Tab 2 on the first
    // sortable header's own stop. Enter, Enter, Space all activate the
    // focused header; the header row is `py-2.5` around a 12px line
    // (~37px), so y = 18 is inside it for the click.
    // A keyboard activation fires the header's click listener without marking
    // the window dirty, so the next key dispatch would hit the *last rendered
    // frame* — whose captured `next` descriptor is still the pre-activation
    // one. The harness flushes a frame after each activation (the feed-back
    // lives in test state; a real caller re-renders through its own state).
    press(cx, "tab tab");
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "space");
    flush_frame(cx);
    click(cx, 80., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["Name:asc", "Name:desc", "Name:asc", "Name:desc"],
        "a focused sortable header must sort on Enter and Space exactly like \
         a click: same column flips, feeding the descriptor back continues \
         the cycle"
    );
}

/// The Table's `{id}-scroll-x` container is `overflow-x-auto` — v3's
/// `.table__scroll-container`, which is what lets a table wider than its box
/// scroll instead of being clipped. No prop pins a column (v3's Table page
/// documents no pinned/sticky column and ships no example of one; the only
/// `sticky` in its sheet is the *header* row), so the honest contract is:
/// the whole body slides under a fixed pointer, and a column that was
/// entirely off-screen comes into reach.
///
/// The assertion below pins that contract with fixed-pointer probes before and
/// after the wheel.
#[gpui::test]
fn table_horizontal_scroll_moves_all_columns(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Three 160px columns with a 160px floor inside a 240px wrapper: the
        // folded content is 480px wide against a ~232px viewport (the
        // wrapper's `px-1` tray), and `min_width` forbids flex-shrink from
        // compressing the columns — so a working scroller must slide the
        // body under the pointer.
        gpui::div()
            .w(px(240.))
            .child(
                Table::new(vec![])
                    .id("tbl-hscroll")
                    .columns(vec![
                        TableColumn::new("A")
                            .default_width(px(160.))
                            .min_width(px(160.)),
                        TableColumn::new("B")
                            .default_width(px(160.))
                            .min_width(px(160.)),
                        TableColumn::new("C")
                            .default_width(px(160.))
                            .min_width(px(160.)),
                    ])
                    .row(vec![
                        probe_cell("hs-a", "col-a", recorded.clone()),
                        probe_cell("hs-b", "col-b", recorded.clone()),
                        probe_cell("hs-c", "col-c", recorded),
                    ])
                    .into_any_element(),
            )
            .into_any_element()
    });

    // Column 0 spans x 0..160 (its probe, inset by the cell's `px-4`, spans
    // 16..144), column 1 x 160..320, column 2 x 320..480. The viewport clips
    // at x = 232, so before any scroll only columns 0 and part of 1 are
    // reachable. The row band: header [30,46] + 80px filler + `py-3` twice +
    // a 1px border, so y = 90 is inside row 0 for every header in the bound.
    click(cx, 100., 90.);
    click(cx, 225., 90.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["col-a", "col-b"],
        "before the scroll, x=100 names the first column and x=225 the second"
    );

    // A 120px wheel to the left (negative horizontal delta) must slide the
    // body: column 1 moves to 40..200 (probe 56..184) and column 2 to
    // 200..360 (probe 216..344), so the same fixed x names the next column
    // over and the column that was clipped past x=232 comes under the
    // pointer. The wheel lands inside the scroller at (100, 90).
    wheel_h(cx, 100., 90., -120.);
    click(cx, 100., 90.);
    click(cx, 225., 90.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["col-a", "col-b", "col-b", "col-c"],
        "a horizontal wheel must move the body under a fixed pointer: the x \
         that named the first column must name the second, and a column that \
         was clipped off-screen must come into reach"
    );
}

/// `Table.Footer` is the row under the body where v3 puts a table's
/// pagination. The port exposes it as `Table::footer(content)` — a plain
/// child row — so the proof is that content *inside* it answers inputs: a
/// probe and a `Pagination` both report from within the footer.
#[gpui::test]
fn table_footer_hosts_pagination_and_reports(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // One 105px row under the ~37px header, then the footer
        // (`px-4 py-2.5`, 16px side padding, 10px vertical) holding a fixed
        // 60x32 probe and a size-md pagination 16px apart.
        Table::new(vec![])
            .id("tbl-footer")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .row(vec![tall_cell("A")])
            .footer(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .child(box_probe(
                        "ft-probe",
                        "footer-probe",
                        60.,
                        32.,
                        recorded.clone(),
                    ))
                    .child(
                        Pagination::new("ft-page", 1, 3).on_change(move |page, _, _| {
                            recorded.borrow_mut().push(format!("page:{page}"));
                        }),
                    ),
            )
            .into_any_element()
    });

    // Footer top = header (30..46) + 105; its content band is that plus 10
    // (py-2.5 top) and 32 tall, so y = 168 sits inside it for every header in
    // the bound. The probe spans x 16..76 (footer px-4), so x = 46 is its
    // centre.
    click(cx, 46., 168.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["footer-probe"],
        "the footer must render its content and answer a click inside it"
    );

    // The pagination starts at 16 (footer padding) + 60 (probe) + 16 (gap) =
    // 92. Its size-md row: prev pill `px-2.5` around a 14px glyph (34px),
    // `gap-1` (4px), three 32px page cells, then the next pill — 180px of
    // content, so the next button spans x 92+146..92+180, centre 92+163.
    click(cx, 255., 168.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["footer-probe", "page:2"],
        "a Table.Footer must host a live pagination: pressing its next arrow \
         reports page 2"
    );
}

/// v3's Table *styles* a disabled row — its `### Interactive States` lists
/// `:disabled` / `[aria-disabled="true"]` — but its API Reference documents
/// no prop that sets it (`Table.Content` has no `disabledKeys`, `Table.Row`
/// has none), so like the progress bar's `[aria-disabled]` there is nothing
/// to drive here. The port has no spelling either: every row is always a
/// keyboard stop and always answers its checkbox. This test pins that
/// contract — if a `disabled_keys`-shaped builder ever appears without being
/// wired into the row stops, this fails.
#[gpui::test]
fn table_every_row_selectable_no_disabled_row_api(cx: &mut TestAppContext) {
    let recorded = events();
    let held: Rc<RefCell<Vec<SharedString>>> = Rc::new(RefCell::new(Vec::new()));
    let held_for_view = held;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        let held_view = held_for_view.clone();
        // Selection is controlled; rows report the whole next set and the
        // caller feeds it back. The 204px wrapper fixes the table at the 44px
        // selection column + a 160px data column.
        let mut table = Table::new(vec![])
            .id("tbl-nodis")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .keyed_row("beta", vec![tall_cell("Beta")])
            .keyed_row("gamma", vec![tall_cell("Gamma")])
            .selected_keys(held_view.borrow().iter().cloned());
        table = table.on_selection_change(move |keys, _, _| {
            let joined = keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            recorded.borrow_mut().push(joined);
            *held_view.borrow_mut() = keys.to_vec();
        });
        gpui::div()
            .w(px(204.))
            .child(table.into_any_element())
            .into_any_element()
    });

    // Keyboard: one Tab, then Down Enter three times — the cursor walks every
    // row in order and each Enter selects it. Nothing skips a row, because
    // the keyboard stops *are* every row.
    press(cx, "tab");
    press(cx, "down");
    press(cx, "enter");
    press(cx, "down");
    press(cx, "enter");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta", "alpha,beta,gamma"],
        "every row must be a keyboard stop: Down Down Down Enter Enter Enter \
         selects all three"
    );

    // And the same rows answer their checkboxes: the 44px column is `py-2.5`
    // around a 16px box, so row i's centre is y = 52 + 105i below the header
    // (90 for row 0) and x = 22. Re-clicking the picked row 0 toggles it off.
    click(cx, 22., 90.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta", "alpha,beta,gamma", "beta,gamma"],
        "every row must also answer its checkbox: re-clicking the picked \
         first row deselects it"
    );
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

/// The overflow scroller: `.tabs__list` is `w-max`, so inside a bounded box
/// the row grows past the box and `.tabs__list-container__scroller` scrolls
/// it; the measuring canvas (which reads `ScrollHandle::max_offset` — written
/// during prepaint, so it takes a frame or two to appear) turns the chevrons
/// on only when there is something that way to scroll.
///
/// Geometry, all derived (labels measured with the window's text system):
///
/// - list padding `p-1` (4px), tab = `px-4` (32px) + measured label; tab *i*
///   starts at `4 + sum_{j<i}(w_j + 32)`.
/// - the scroller's visible width is the 240px box; `max` = list width - 240.
/// - the next chevron is a 16px circle at `right-1` (x 220..236) vertically
///   centred on the 40px list (y 12..28); the prev chevron mirrors it at
///   `left-1` (x 4..20).
/// - one chevron click scrolls by `min(120, max)` (the step is 120px).
#[gpui::test]
fn tabs_overflow_scroller_chevrons_scroll_the_list(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        gpui::div()
            .w(px(240.))
            .child(
                Tabs::new(
                    "tb-ol",
                    vec![
                        TabItem::new("one", "One"),
                        TabItem::new("two", "Two"),
                        TabItem::new("three", "Three"),
                        TabItem::new("four", "Four"),
                        TabItem::new("five", "Five"),
                    ],
                    "one",
                )
                .on_selection_change(move |key, _, _| {
                    recorded.borrow_mut().push(key.to_string());
                })
                .into_any_element(),
            )
            .into_any_element()
    });

    let labels = ["One", "Two", "Three", "Four", "Five"];
    let mut starts = [0f32; 5];
    let mut centres = [0f32; 5];
    let mut x = 4.;
    for (i, label) in labels.iter().enumerate() {
        // The shaped advance is rounded *up* to whole pixels by gpui's own
        // text measurement, so the layout widths here mirror the box sizes.
        let w = cx
            .update(|window, _| text_width(window.text_system(), label, 14.0, FontWeight::MEDIUM))
            .ceil();
        starts[i] = x;
        centres[i] = x + (w + 32.) / 2.;
        x += w + 32.;
    }
    let list_width = x + 4.;
    let max = (list_width - 240.).max(0.);
    let scrolled = max.min(120.);

    // The measuring canvas needs a few frames: `max_offset` is written during
    // the scroller's prepaint and the chevron renders one frame later.
    flush_frame(cx);
    flush_frame(cx);
    flush_frame(cx);

    // "Five" sits entirely past the 240px viewport (its centre is about 273),
    // so a click at its would-be centre records nothing.
    click(cx, centres[4], 20.);
    assert!(
        recorded.borrow().is_empty(),
        "the last tab starts beyond the viewport: a click where it would be \
         (x = {}) must record nothing",
        centres[4]
    );

    // The next chevron at (228, 20) must scroll the list left by `scrolled`,
    // sliding "Five" to `centres[4] - scrolled`, where a click now lands on
    // it. Clicking the prev chevron (12, 20) must slide everything back, so
    // the same x finds "Four" again.
    click(cx, 228., 20.);
    flush_frame(cx);
    click(cx, centres[4] - scrolled, 20.);
    click(cx, 12., 20.);
    flush_frame(cx);
    click(cx, centres[4] - scrolled, 20.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["five", "four"],
        "a chevron click must scroll the list (an off-screen tab becomes \
         clickable at its slid position) and must not select anything itself"
    );
}

/// `TabItem::separator()` composes v3's `Tabs.Separator` — a 1px hairline
/// drawn *before* the tab that carries it. It is a plain styled div with no
/// listeners and no place in the arrow stops (which are the tab indices), so
/// it must neither answer a click nor become a stop.
#[gpui::test]
fn tabs_separator_is_not_a_tab(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // The separator sits between "one" and "two" (`w: border_width` —
        // 1px — `h-4`, vertically centred).
        Tabs::new(
            "tb-sep",
            vec![
                TabItem::new("one", "One"),
                TabItem::new("two", "Two").separator(),
                TabItem::new("three", "Three"),
            ],
            "one",
        )
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // The list is `p-1`, so tab "one" spans x 4..(4 + w_one + 32) and the
    // separator is the next 1px, at (4 + w_one + 32)..(4 + w_one + 33).
    // gpui's own text measurement rounds the shaped advance *up* to whole
    // pixels (`TextLayout` ceils the line width) — exactly how the tab's box
    // is sized — so the measured width must be rounded the same way before
    // the boundary arithmetic. A click at the separator's centre, half a
    // pixel from either tab, cannot land on a tab; it falls through to the
    // (listener-less) scroller.
    let w_one =
        cx.update(|window, _| text_width(window.text_system(), "One", 14.0, FontWeight::MEDIUM));
    let tab_one_end = 4. + w_one.ceil() + 32.;
    click(cx, tab_one_end + 0.5, 20.);
    assert!(
        recorded.borrow().is_empty(),
        "clicking the 1px separator must record nothing"
    );

    // The arrows skip it: Tab lands on the selected tab ("one"), and Right
    // moves one -> two -> three — there is no stop between them.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["two", "three"],
        "the separator must not become a stop: Right Right from the first tab \
         reaches third directly"
    );
}

/// The port's spelling of a disabled tab is `Tabs::is_disabled`, which
/// disables the *whole* list. (v3 also documents per-tab `Tabs.Tab.isDisabled`
/// — `id`, `isDisabled`, `className`, `render` — and this port's `TabItem`
/// has no such builder, so a single dead tab among live ones cannot even be
/// expressed; the report carries that parity gap.) A disabled list must leave
/// the tab order and answer no key and no click.
#[gpui::test]
fn tabs_disabled_list_answers_no_key_or_click(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Tabs::new(
            "tb-dead",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .is_disabled(true)
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    // No tab claims the list's focus handle (`track_focus` is gated on
    // `!is_disabled`) and no tab has a click handler, so Tab reaches nothing
    // and every key the live list would answer falls through to the host
    // root, which has no handlers either.
    press(cx, "tab");
    press(cx, "right");
    press(cx, "left");
    press(cx, "home");
    press(cx, "end");
    press(cx, "enter");
    press(cx, "space");

    // A click on the second tab's measured centre (list p-1 + the first
    // tab's label+32 + half of the second's) must record nothing either.
    let w_first =
        cx.update(|window, _| text_width(window.text_system(), "First", 14.0, FontWeight::MEDIUM));
    let w_second =
        cx.update(|window, _| text_width(window.text_system(), "Second", 14.0, FontWeight::MEDIUM));
    let tab1_centre_x = 4. + w_first + 32. + (w_second + 32.) / 2.;
    click(cx, tab1_centre_x, 20.);
    assert!(
        recorded.borrow().is_empty(),
        "a disabled tab list must not answer Tab, the arrows, Enter, Space or \
         a click"
    );
}

// ---------------------------------------------------------------------------
// Accordion
// ---------------------------------------------------------------------------

/// `allowsMultipleExpanded` opted in to true: expanding a second item must
/// not collapse the first, and the reported set must contain both keys at
/// once. (The prop now has to be set explicitly — v3's default is `false`,
/// which `nav_deep.rs` pins. The bodies are a fixed 40px so the second
/// header's seat is exact.)
#[gpui::test]
fn accordion_multiple_expand_keeps_both_open(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Accordion::new(vec![
            AccordionItem::new("one", "Item one").content(gpui::div().h(px(40.))),
            AccordionItem::new("two", "Item two").content(gpui::div().h(px(40.))),
        ])
        .id("acc-multi")
        .allows_multiple_expanded(true)
        .on_expanded_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // Header 0 centre: y 26. With item one open, item two's header sits below
    // 52 (header) + 2+40+16 (body) + 1 (separator) + 26 (half header) = 137.
    // Both must be reported together — the multiple-expand mode.
    click(cx, 60., 26.);
    flush_frame(cx);
    click(cx, 60., 137.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "one,two"],
        "expanding the second item must report both keys, never collapse the \
         first"
    );

    // Closing them one at a time leaves the other open, then clears the set.
    // With item one collapsed again, item two's header moves back up to its
    // natural seat — 52 (header) + 1 (separator) + 26 (half header) = 79 —
    // which is where the closing click must land.
    click(cx, 60., 26.);
    flush_frame(cx);
    click(cx, 60., 79.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", "one,two", "two", ""],
        "closing either item must report the set with the other still open, \
         and the last closure must report an empty set"
    );
}

/// A `disabledKeys` item cannot be toggled by click — its trigger has no
/// click handler — or by key: `track_focus` is gated on the disabled flag, so
/// the trigger leaves the tab order and no key event can ever reach it.
#[gpui::test]
fn accordion_disabled_item_cannot_be_toggled(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Accordion::new(vec![
            AccordionItem::new("one", "Item one"),
            AccordionItem::new("two", "Item two"),
        ])
        .id("acc-dis")
        .disabled_keys(["two".into()])
        .on_expanded_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    // The second header's centre: 52 + 1 (separator) + 26 = 79. It is dimmed
    // but present; a click must record nothing.
    click(cx, 60., 79.);
    assert!(
        recorded.borrow().is_empty(),
        "clicking a disabledKeys trigger must record nothing"
    );

    // Tab lands on the first (enabled) trigger and Enter fires its click
    // listener (gpui activates a focused element with click listeners on
    // Enter/Space). The disabled trigger is not a tab stop, so no Tab can
    // reach it: a second Tab finds no further stop and a second Enter just
    // toggles the first trigger again.
    press(cx, "tab");
    press(cx, "enter");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["one", ""],
        "the enabled trigger must answer Enter (open then close); the disabled \
         one must never appear in any report"
    );
}

/// `DisclosureGroup` stacks `Disclosure`s; an open body (`p-2` all round)
/// pushes the triggers below it down. The group is controlled, so the first
/// item is seeded open and its 40px body shifts the second and third
/// triggers by an exact 56px — the offset arithmetic is the point.
#[gpui::test]
fn disclosure_group_third_item_reports_with_bodies_pushing(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        // Each trigger is a md Button (`h-9`, 36px) stretched to the group's
        // full width by the flex column. With the first body open:
        //   trigger 1      y  0..36
        //   body 1         y 36..92   (8 + 40 + 8)
        //   trigger 2      y 92..128  (centre 110)
        //   trigger 3      y 128..164 (centre 146)
        // Without the open body, trigger 2 would sit at y 36..72 and trigger
        // 3 at 72..108 — the pushed-down seats only exist because the body
        // between them rendered.
        let mut expanded = HashSet::new();
        expanded.insert(SharedString::from("first"));
        DisclosureGroup::new()
            .expanded_keys(expanded)
            .item("first", "Basic settings", gpui::div().h(px(40.)))
            .item("second", "Advanced settings", gpui::div().h(px(40.)))
            .item("third", "Team plan", gpui::div().h(px(40.)))
            .on_toggle(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    // x = 40 is inside every stretched trigger. The third trigger answers at
    // y = 146 — a seat that exists only because the first item's open body
    // pushed it down by 56px; the second answers at 110.
    click(cx, 40., 110.);
    click(cx, 40., 146.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["second", "third"],
        "pressing the pushed-down second and third triggers must report their \
         keys; only the first item's open body can have put them there"
    );
}
