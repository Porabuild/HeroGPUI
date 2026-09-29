//! Data-table extras on `Table` (HeroGPUI extension, after gpui-kit's
//! table): column reordering by header drag and Alt+Left/Alt+Right,
//! controlled and uncontrolled order, and single-cell selection with a
//! press, Left/Right/Home/End and the row keys, reported by the columns'
//! given indices.

mod harness;

use std::{cell::RefCell, rc::Rc};

use gpui::{
    point, prelude::*, px, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    TestAppContext, VisualTestContext,
};
use harness::{events, open_host, press, still, Events};
use herogpui_components::{SortDescriptor, Table, TableCell, TableColumn, TableRow};

const COLUMNS: [&str; 3] = ["Name", "Role", "Status"];

#[derive(Clone, Default)]
struct Config {
    reorder: bool,
    order: Option<Vec<usize>>,
    default_order: Option<Vec<usize>>,
    cells: bool,
    selected_cell: Option<Option<TableCell>>,
}

fn row(r: usize) -> TableRow {
    TableRow::new(
        (0..COLUMNS.len())
            .map(|c| {
                gpui::div()
                    .debug_selector(move || format!("c-{r}-{c}"))
                    .child(format!("r{r}c{c}"))
                    .into_any_element()
            })
            .collect(),
    )
}

fn host(cx: &mut TestAppContext, config: Config) -> (Events, &mut VisualTestContext) {
    still();
    let seen = events();
    let s = seen.clone();
    let sort = Rc::new(RefCell::new(None::<SortDescriptor>));
    let cx = open_host(cx, move || {
        let (moves, cells, sorts, sort_now) = (s.clone(), s.clone(), s.clone(), sort.clone());
        let mut table = Table::new(Vec::new())
            .id("people")
            .columns(
                COLUMNS
                    .iter()
                    .map(|label| TableColumn::new(*label).allows_sorting(true))
                    .collect(),
            )
            .on_sort_change(move |descriptor, _, _| {
                sorts
                    .borrow_mut()
                    .push(format!("sort:{}", descriptor.column));
                *sort_now.borrow_mut() = Some(descriptor.clone());
            })
            .on_column_move(move |m, _, _| {
                moves
                    .borrow_mut()
                    .push(format!("move:{}->{}:{:?}", m.from, m.to, m.order));
            })
            .on_cell_select(move |cell, _, _| {
                cells
                    .borrow_mut()
                    .push(format!("cell:{}/{}", cell.row, cell.column));
            });
        if let Some(descriptor) = sort.borrow().clone() {
            table = table.sort_descriptor(descriptor);
        }
        table = table.allows_column_reorder(config.reorder);
        if let Some(order) = config.order.clone() {
            table = table.column_order(order);
        }
        if let Some(order) = config.default_order.clone() {
            table = table.default_column_order(order);
        }
        if config.cells {
            table = table.cell_selectable(true);
        }
        if let Some(cell) = config.selected_cell.clone() {
            table = table.selected_cell(cell);
        }
        for r in 0..3 {
            table = table.tree_row(row(r));
        }
        gpui::div().w(px(720.)).child(table).into_any_element()
    });
    (seen, cx)
}

fn frame(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

fn cell(cx: &mut VisualTestContext, r: usize, c: usize) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(Box::leak(format!("c-{r}-{c}").into_boxed_str()))
        .unwrap_or_else(|| panic!("cell {r},{c} laid out"))
}

/// The given columns, left to right, read off the first row's cells.
fn displayed(cx: &mut VisualTestContext) -> Vec<usize> {
    let mut columns: Vec<(f32, usize)> = (0..COLUMNS.len())
        .map(|c| (f32::from(cell(cx, 0, c).left()), c))
        .collect();
    columns.sort_by(|a, b| a.0.total_cmp(&b.0));
    columns.into_iter().map(|(_, c)| c).collect()
}

/// A header's centre, found above its column's first cell.
fn header(cx: &mut VisualTestContext, given: usize) -> gpui::Point<gpui::Pixels> {
    let first = cell(cx, 0, given);
    let at = displayed(cx).iter().position(|c| *c == given).unwrap();
    let track = cx
        .debug_bounds(Box::leak(
            format!("table-header-track-{at}").into_boxed_str(),
        ))
        .expect("header track");
    point(first.left() + px(4.), track.center().y)
}

fn keys(cx: &mut VisualTestContext, list: &str) {
    for key in list.split_whitespace() {
        press(cx, key);
        frame(cx);
    }
}

fn log(seen: &Events) -> Vec<String> {
    seen.borrow().clone()
}

fn mouse(cx: &mut VisualTestContext, at: gpui::Point<gpui::Pixels>, kind: &str) {
    match kind {
        "down" => cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::none(),
            click_count: 1,
            first_mouse: false,
        }),
        "move" => cx.simulate_event(MouseMoveEvent {
            position: at,
            pressed_button: Some(MouseButton::Left),
            modifiers: Modifiers::none(),
        }),
        _ => cx.simulate_event(MouseUpEvent {
            button: MouseButton::Left,
            position: at,
            modifiers: Modifiers::none(),
            click_count: 1,
        }),
    }
    frame(cx);
}

#[gpui::test]
fn a_default_order_puts_columns_and_cells_in_display_order(cx: &mut TestAppContext) {
    let (_seen, cx) = host(
        cx,
        Config {
            default_order: Some(vec![2, 0, 1]),
            ..Config::default()
        },
    );
    frame(cx);
    assert_eq!(displayed(cx), [2, 0, 1]);
    // Every row follows the header.
    assert!(cell(cx, 2, 2).left() < cell(cx, 2, 0).left());
}

#[gpui::test]
fn alt_arrows_move_the_focused_header_and_the_focus_follows(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            reorder: true,
            ..Config::default()
        },
    );
    frame(cx);
    // The body is the table's first tab stop; the next is the first
    // sortable header, Name.
    keys(cx, "tab tab alt-right");
    assert_eq!(log(&seen), ["move:0->1:[1, 0, 2]"]);
    assert_eq!(displayed(cx), [1, 0, 2]);
    // The focus followed Name: the next move takes it further right.
    keys(cx, "alt-right");
    assert_eq!(displayed(cx), [1, 2, 0]);
    // At the end nothing moves; Alt+Left brings it back one.
    keys(cx, "alt-right alt-left");
    assert_eq!(
        log(&seen),
        [
            "move:0->1:[1, 0, 2]",
            "move:1->2:[1, 2, 0]",
            "move:2->1:[1, 0, 2]"
        ]
    );
    // Enter still sorts the focused header — it is Name, where it moved.
    keys(cx, "enter");
    assert_eq!(log(&seen).last().unwrap(), "sort:Name");
}

#[gpui::test]
fn dragging_a_header_onto_another_column_moves_it(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            reorder: true,
            ..Config::default()
        },
    );
    frame(cx);
    let from = header(cx, 0);
    let to = header(cx, 2);
    mouse(cx, from, "down");
    mouse(cx, point(from.x + px(20.), from.y), "move");
    mouse(cx, to, "move");
    assert!(
        cx.debug_bounds("table-column-drop-indicator").is_some(),
        "the drop position is drawn"
    );
    mouse(cx, to, "up");
    assert_eq!(log(&seen), ["move:0->2:[1, 2, 0]"]);
    assert_eq!(displayed(cx), [1, 2, 0]);
    assert!(cx.debug_bounds("table-column-drop-indicator").is_none());
    // A plain click on a header sorts it and moves nothing.
    let status = header(cx, 2);
    cx.simulate_click(status, Modifiers::none());
    frame(cx);
    assert_eq!(log(&seen), ["move:0->2:[1, 2, 0]", "sort:Status"]);
}

#[gpui::test]
fn a_table_without_reordering_ignores_the_drag_and_the_keys(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    let from = header(cx, 0);
    let to = header(cx, 2);
    mouse(cx, from, "down");
    mouse(cx, point(from.x + px(20.), from.y), "move");
    mouse(cx, to, "move");
    mouse(cx, to, "up");
    keys(cx, "tab tab alt-right");
    assert!(log(&seen).iter().all(|e| !e.starts_with("move")));
    assert_eq!(displayed(cx), [0, 1, 2]);
}

#[gpui::test]
fn a_controlled_order_reports_and_waits_for_its_prop(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            reorder: true,
            order: Some(vec![2, 1, 0]),
            ..Config::default()
        },
    );
    frame(cx);
    assert_eq!(displayed(cx), [2, 1, 0]);
    keys(cx, "tab tab alt-right");
    assert_eq!(log(&seen), ["move:0->1:[1, 2, 0]"]);
    assert_eq!(displayed(cx), [2, 1, 0], "the prop still decides");
}

#[gpui::test]
fn a_press_selects_a_cell_and_the_arrows_move_it(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            cells: true,
            ..Config::default()
        },
    );
    frame(cx);
    let at = cell(cx, 1, 1).center();
    cx.simulate_click(at, Modifiers::none());
    frame(cx);
    assert_eq!(log(&seen), ["cell:1/1"]);
    let ring = cx
        .debug_bounds("table-selected-cell")
        .expect("the cell is ringed");
    assert!(ring.contains(&at), "the ring is on the pressed cell");
    keys(cx, "right");
    keys(cx, "right");
    keys(cx, "down");
    keys(cx, "home");
    keys(cx, "up");
    keys(cx, "end");
    assert_eq!(
        log(&seen),
        ["cell:1/1", "cell:1/2", "cell:2/2", "cell:2/0", "cell:1/0", "cell:1/2"],
        "Right clamps at the last column; Up/Down keep the column"
    );
}

#[gpui::test]
fn cells_are_reported_by_their_given_column_after_a_reorder(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            cells: true,
            default_order: Some(vec![2, 0, 1]),
            ..Config::default()
        },
    );
    frame(cx);
    // The leftmost displayed cell is Status (given index 2).
    let at = cell(cx, 0, 2).center();
    cx.simulate_click(at, Modifiers::none());
    frame(cx);
    keys(cx, "right");
    assert_eq!(log(&seen), ["cell:0/2", "cell:0/0"]);
}

#[gpui::test]
fn a_controlled_cell_reports_and_keeps_its_prop(cx: &mut TestAppContext) {
    let (seen, cx) = host(
        cx,
        Config {
            cells: true,
            selected_cell: Some(Some(TableCell::new("0", 0))),
            ..Config::default()
        },
    );
    frame(cx);
    let ring = cx
        .debug_bounds("table-selected-cell")
        .expect("the prop is ringed");
    assert!(ring.contains(&cell(cx, 0, 0).center()));
    // The body is the first tab stop.
    keys(cx, "tab right");
    let reported = log(&seen);
    assert!(
        reported.iter().any(|e| e == "cell:0/1"),
        "the request is reported: {reported:?}"
    );
    let ring = cx.debug_bounds("table-selected-cell").unwrap();
    assert!(
        ring.contains(&cell(cx, 0, 0).center()),
        "the prop still decides"
    );
}

#[gpui::test]
fn virtual_rows_follow_the_column_order(cx: &mut TestAppContext) {
    still();
    let cx = open_host(cx, || {
        Table::new(COLUMNS.iter().map(|c| (*c).into()).collect())
            .id("virtual-people")
            .default_column_order([1, 2, 0])
            .row_height(px(40.))
            .max_h(px(200.))
            .virtual_rows(20, "people", |ix| ix.to_string().into(), row)
            .into_any_element()
    });
    frame(cx);
    assert_eq!(displayed(cx), [1, 2, 0]);
}
