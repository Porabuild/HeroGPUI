//! Deeper Table behaviour not covered by the sorting, selection, resize,
//! virtualisation, footer and load-more suites.

mod harness;

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{prelude::*, px, SharedString, TestAppContext, VisualTestContext};
use herogpui_components::{SelectionMode, Table, TableColumn, TableRow};

use harness::{click, events, open_host, press};

fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

/// v3's tree-table example controls `expandedKeys` through
/// `onExpandedChange`. The first data row starts after the ~37px header; its
/// chevron is 18px square after the tree cell's 16px left padding, so (29, 58)
/// is its centre inside the primary table's 4px tray.
#[gpui::test]
fn table_tree_chevron_reports_expand_then_collapse(cx: &mut TestAppContext) {
    let expanded = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let expanded_for_view = expanded;
    let recorded = events();
    let recorded_for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let expanded = expanded_for_view.clone();
        let expanded_now = expanded.borrow().clone();
        let recorded = recorded_for_view.clone();
        gpui::div()
            .w(px(320.))
            .child(
                Table::new(vec![])
                    .id("table-tree-deep")
                    .column(TableColumn::new("Name").default_width(px(320.)))
                    .tree_column(0)
                    .expanded_keys(expanded_now)
                    .tree_row(
                        TableRow::new(vec![gpui::div().child("Parent").into_any_element()])
                            .key("parent")
                            .children(vec![TableRow::new(vec![gpui::div()
                                .child("Child")
                                .into_any_element()])
                            .key("child")]),
                    )
                    .on_expanded_change(move |keys, window, _| {
                        *expanded.borrow_mut() = keys.to_vec();
                        recorded.borrow_mut().push(
                            keys.iter()
                                .map(AsRef::<str>::as_ref)
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                        window.refresh();
                    }),
            )
            .into_any_element()
    });

    click(cx, 29., 58.);
    flush_frame(cx);
    click(cx, 29., 58.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["parent", ""],
        "the same tree chevron must report the controlled expanded set on open and close"
    );
}

/// `Table.Body.renderEmptyState` is interactive content, not a painted label.
/// With a 320px table its full-width 40px probe is centred below the ~37px
/// header and the empty wrapper's 28px top padding, so (160, 85) is inside it.
#[gpui::test]
fn table_empty_state_keeps_its_content_interactive(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        gpui::div()
            .w(px(320.))
            .child(
                Table::new(vec!["Name".into()])
                    .id("table-empty-deep")
                    .empty_state(
                        gpui::div()
                            .id("table-empty-probe")
                            .w_full()
                            .h(px(40.))
                            .on_click(move |_, _, _| recorded.borrow_mut().push("empty".into())),
                    ),
            )
            .into_any_element()
    });

    click(cx, 160., 85.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["empty"],
        "an empty table must preserve the behavior of its renderEmptyState content"
    );
}

/// `selectionMode="none"` is action-only. Enter must still activate the row,
/// but `onSelectionChange` has no selection to report and must remain silent.
#[gpui::test]
fn table_none_mode_keyboard_fires_only_the_row_action(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let actions = for_view.clone();
        let selections = for_view.clone();
        Table::new(vec![])
            .id("table-none-deep")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::None)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .on_row_click(move |index, _, _, _| {
                actions.borrow_mut().push(format!("action:{index}"));
            })
            .on_selection_change(move |keys, _, _| {
                selections.borrow_mut().push(format!(
                    "selection:{}",
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:0"],
        "none mode must preserve the row action without reporting selection"
    );
}

/// Single selection replaces the current row and clears when the selected row
/// is activated again. The caller feeds each controlled set back before the
/// next key, so the test exercises the table's real per-frame selection input.
#[gpui::test]
fn table_single_mode_keyboard_replaces_then_clears(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let held_for_view = held;
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let held = held_for_view.clone();
        let selected = held.borrow().clone();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-single-deep")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Single)
            .selected_keys(selected)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .on_selection_change(move |keys, window, _| {
                *held.borrow_mut() = keys.to_vec();
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
                window.refresh();
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "down");
    press(cx, "enter");
    flush_frame(cx);
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "beta", ""],
        "single mode must replace the selected key and permit clearing it"
    );
}
