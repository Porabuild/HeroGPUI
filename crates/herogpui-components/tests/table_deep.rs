//! Deeper Table behaviour not covered by the sorting, selection, resize,
//! virtualisation, footer and load-more suites.

mod harness;

use std::cell::{Cell, RefCell};
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

#[gpui::test]
fn table_tree_right_expands_and_left_collapses_the_focused_parent(cx: &mut TestAppContext) {
    let expanded = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let expanded_for_view = expanded;
    let recorded = events();
    let recorded_for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let expanded = expanded_for_view.clone();
        let expanded_now = expanded.borrow().clone();
        let recorded = recorded_for_view.clone();
        Table::new(vec![])
            .id("table-tree-keys")
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
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    cx.update(|_, cx| herogpui_components::util::set_focus_visible(false, cx));
    press(cx, "right");
    assert!(
        cx.update(|_, cx| herogpui_components::util::focus_visible(cx)),
        "a handled tree key must still record keyboard focus visibility"
    );
    flush_frame(cx);
    press(cx, "left");

    assert_eq!(
        recorded.borrow().as_slice(),
        ["parent", ""],
        "Right must expand the focused parent and Left must collapse it again"
    );
}

#[gpui::test]
fn table_tree_left_on_a_child_moves_the_cursor_to_its_parent(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-tree-parent-key")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .tree_column(0)
            .expanded_keys([SharedString::from("parent")])
            .tree_row(
                TableRow::new(vec![gpui::div().child("Parent").into_any_element()])
                    .key("parent")
                    .children(vec![TableRow::new(vec![gpui::div()
                        .child("Child")
                        .into_any_element()])
                    .key("child")]),
            )
            .on_row_click(move |index, _, _, _| {
                recorded.borrow_mut().push(index.to_string());
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    press(cx, "down");
    press(cx, "left");
    press(cx, "enter");

    assert_eq!(
        recorded.borrow().as_slice(),
        ["0"],
        "Left on a child must move the row cursor to its parent before Enter activates"
    );
}

#[gpui::test]
fn callbackless_table_tree_consumes_an_expand_key(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        gpui::div()
            .on_key_down(move |event, _, _| {
                recorded.borrow_mut().push(event.keystroke.key.clone());
            })
            .child(
                Table::new(vec![])
                    .id("table-tree-controlled-read-only")
                    .column(TableColumn::new("Name").default_width(px(320.)))
                    .tree_column(0)
                    .tree_row(
                        TableRow::new(vec![gpui::div().child("Parent").into_any_element()])
                            .key("parent")
                            .children(vec![TableRow::new(vec![gpui::div()
                                .child("Child")
                                .into_any_element()])
                            .key("child")]),
                    ),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "down");
    recorded.borrow_mut().clear();
    cx.update(|_, cx| herogpui_components::util::set_focus_visible(false, cx));
    press(cx, "right");

    assert!(
        recorded.borrow().is_empty(),
        "Right on a collapsed parent must not escape a callbackless controlled tree"
    );
    assert!(
        cx.update(|_, cx| herogpui_components::util::focus_visible(cx)),
        "a consumed tree key must still record keyboard focus visibility"
    );
}

/// React Aria's default `disabledBehavior="all"` disables every interaction
/// on a row, including the expansion button it composes in the tree column.
#[gpui::test]
fn disabled_table_tree_chevron_is_inert(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        gpui::div()
            .w(px(320.))
            .child(
                Table::new(vec![])
                    .id("table-tree-disabled")
                    .column(TableColumn::new("Name").default_width(px(320.)))
                    .tree_column(0)
                    .disabled_keys(["parent"])
                    .tree_row(
                        TableRow::new(vec![gpui::div().child("Parent").into_any_element()])
                            .key("parent")
                            .children(vec![TableRow::new(vec![gpui::div()
                                .child("Child")
                                .into_any_element()])
                            .key("child")]),
                    )
                    .on_expanded_change(move |keys, _, _| {
                        recorded.borrow_mut().push(
                            keys.iter()
                                .map(AsRef::<str>::as_ref)
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                    }),
            )
            .into_any_element()
    });

    click(cx, 29., 58.);
    assert!(
        recorded.borrow().is_empty(),
        "a disabled expandable row must not report an expanded-key change"
    );
}

/// Pinned `TableKeyboardDelegate.getKeyForSearch` searches row text, wraps,
/// and skips rows excluded from the roving collection. The typeahead only
/// moves focus; Enter proves which row it found.
#[gpui::test]
fn table_typeahead_uses_row_text_and_skips_disabled_matches(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-text")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .disabled_keys(["delta"])
            .tree_row(
                TableRow::new(vec![gpui::div().child("Dawn").into_any_element()])
                    .key("dawn")
                    .text_value("Dawn"),
            )
            .tree_row(
                TableRow::new(vec![gpui::div().child("Delta").into_any_element()])
                    .key("delta")
                    .text_value("Delta"),
            )
            .tree_row(
                TableRow::new(vec![gpui::div().child("Denmark").into_any_element()])
                    .key("denmark")
                    .text_value("Denmark"),
            )
            .on_row_click(move |index, _, _, _| recorded.borrow_mut().push(index.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "d e");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["2"],
        "the growing 'de' query must skip disabled Delta and focus Denmark"
    );
}

/// Pinned `useTypeSelect` appends repeated letters verbatim. A failed `dd`
/// query clears the buffer but leaves focus on Dawn; it does not turn the
/// second `d` into a request for the next d-row.
#[gpui::test]
fn table_typeahead_does_not_cycle_a_repeated_letter(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-repeat")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .tree_row(
                TableRow::new(vec![gpui::div().child("Dawn").into_any_element()])
                    .key("dawn")
                    .text_value("Dawn"),
            )
            .tree_row(
                TableRow::new(vec![gpui::div().child("Denmark").into_any_element()])
                    .key("denmark")
                    .text_value("Denmark"),
            )
            .on_row_click(move |index, _, _, _| recorded.borrow_mut().push(index.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "d d");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["0"],
        "a repeated letter must not cycle to the next matching row"
    );
}

/// Once pinned `useTypeSelect` has a query, Space extends it instead of
/// activating the current row. This distinguishes "New York" from "New".
#[gpui::test]
fn table_typeahead_includes_space_after_the_query_starts(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-space")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .tree_row(
                TableRow::new(vec![gpui::div().child("New").into_any_element()])
                    .key("new")
                    .text_value("New"),
            )
            .tree_row(
                TableRow::new(vec![gpui::div().child("New York").into_any_element()])
                    .key("new-york")
                    .text_value("New York"),
            )
            .on_row_click(move |index, _, _, _| recorded.borrow_mut().push(index.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "n e w space");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["1"],
        "Space inside a live query must distinguish the longer prefix"
    );
}

/// Pinned `getStringForKey` accepts any single printable character, not only
/// letters and digits.
#[gpui::test]
fn table_typeahead_accepts_punctuation(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-punctuation")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .tree_row(
                TableRow::new(vec![gpui::div().child("-Dash").into_any_element()])
                    .key("dash")
                    .text_value("-Dash"),
            )
            .on_row_click(move |index, _, _, _| recorded.borrow_mut().push(index.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "-");
    press(cx, "enter");
    assert_eq!(recorded.borrow().as_slice(), ["0"]);
}

/// `useTypeSelect` clears its query after one second. A later Space activates
/// selection again, even when no other key arrived to clear our lazy buffer.
#[gpui::test]
fn table_typeahead_timeout_restores_space_activation(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-timeout")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .selection_mode(SelectionMode::Multiple)
            .tree_row(
                TableRow::new(vec![gpui::div().child("North").into_any_element()])
                    .key("north")
                    .text_value("North"),
            )
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "n");
    std::thread::sleep(std::time::Duration::from_millis(1020));
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["north"],
        "Space after the query timeout must select the focused row"
    );
}

/// Pinned `useTypeSelect` intercepts Space during capture while a query is
/// live, before a focused row checkbox can arm its own keyboard activation.
#[gpui::test]
fn table_typeahead_space_precedes_a_focused_row_checkbox(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-checkbox")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .selection_mode(SelectionMode::Multiple)
            .tree_row(
                TableRow::new(vec![gpui::div().child("North").into_any_element()])
                    .key("north")
                    .text_value("North"),
            )
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "n");
    click(cx, 22., 58.);
    recorded.borrow_mut().clear();
    press(cx, "space");
    assert!(
        recorded.borrow().is_empty(),
        "Space extending a live query must not toggle the focused row checkbox"
    );
}

/// A virtual table's collection still owns every row's text value even though
/// the viewport builds only nearby elements. Typeahead must find an offscreen
/// enabled match without eagerly constructing the full row set.
#[gpui::test]
fn virtual_table_typeahead_searches_unbuilt_row_text(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let builds = Rc::new(Cell::new(0usize));
    let builds_for_view = builds.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        let builds = builds_for_view.clone();
        Table::new(vec![])
            .id("table-typeahead-virtual")
            .column(TableColumn::new("Name").default_width(px(320.)))
            .row_height(px(40.))
            .max_h(px(160.))
            .disabled_keys(["25"])
            .virtual_rows(
                50,
                "typeahead-rows",
                |index| SharedString::from(index.to_string()),
                move |index| {
                    builds.set(builds.get() + 1);
                    TableRow::new(vec![gpui::div()
                        .child(format!("Row {index}"))
                        .into_any_element()])
                    .key(index.to_string())
                },
            )
            .virtual_text_value(|index| match index {
                25 => SharedString::from("Zeta"),
                40 => SharedString::from("Zulu"),
                _ => SharedString::from(format!("Row {index}")),
            })
            .on_row_click(move |index, _, _, _| recorded.borrow_mut().push(index.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "z");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["40"],
        "typeahead must skip disabled offscreen Zeta and focus offscreen Zulu"
    );
    assert!(
        builds.get() < 50,
        "projecting typeahead text must not eagerly construct every virtual row"
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

/// React Aria's inherited `useGrid` contract defaults Escape to clearing a
/// non-empty selection. The table body owns the roving stop, so the clear is
/// reported through the same controlled selection callback as a row press.
#[gpui::test]
fn table_escape_clears_a_non_empty_selection(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-escape-deep")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .selected_keys([SharedString::from("alpha")])
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab escape");
    assert_eq!(
        recorded.borrow().as_slice(),
        [""],
        "Escape must clear the table's non-empty selection"
    );
}

/// Pinned React Aria extends a multiple selection from the last toggled row
/// when Shift is held during arrow navigation.
#[gpui::test]
fn table_shift_arrows_extend_and_reverse_from_selection_anchor(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let held_for_view = held;
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = held_for_view.borrow().clone();
        let held = held_for_view.clone();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-shift-extend")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .selected_keys(selected)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
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

    press(cx, "tab down space");
    flush_frame(cx);
    press(cx, "down shift-space");
    flush_frame(cx);
    press(cx, "shift-space");
    flush_frame(cx);
    press(cx, "up shift-up down");
    flush_frame(cx);
    press(cx, "shift-down");
    flush_frame(cx);
    press(cx, "shift-up");
    flush_frame(cx);
    press(cx, "down shift-enter");
    flush_frame(cx);
    press(cx, "ctrl-a");
    flush_frame(cx);
    press(cx, "shift-up");
    flush_frame(cx);
    press(cx, "ctrl-a");
    flush_frame(cx);
    press(cx, "shift-down");
    assert_eq!(
        recorded.borrow().as_slice(),
        [
            "alpha",
            "alpha,beta",
            "alpha,beta,gamma",
            "alpha,beta",
            "alpha,beta,gamma",
            "alpha,beta",
            "alpha,beta,gamma",
            "gamma",
        ],
        "Shift+Arrow must rebuild the range from the last toggled row"
    );
}

/// Range extension uses the Table's own uncontrolled selection state and skips
/// disabled rows inside the anchor-to-target span.
#[gpui::test]
fn table_uncontrolled_shift_range_skips_disabled_rows(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-shift-uncontrolled")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .disabled_keys([SharedString::from("beta")])
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab down space");
    flush_frame(cx);
    press(cx, "shift-down");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,gamma"],
        "an uncontrolled Shift range must exclude disabled collection rows"
    );
}

/// The pointer select-all control enters the same pinned `all` selection state
/// as Mod+A, so the next Shift move collapses to its target row.
#[gpui::test]
fn table_header_select_all_resets_the_shift_range(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let held_for_view = held;
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = held_for_view.borrow().clone();
        let held = held_for_view.clone();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-header-all-range")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .selected_keys(selected)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
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

    press(cx, "tab down space");
    flush_frame(cx);
    click(cx, 22., 18.);
    flush_frame(cx);
    press(cx, "shift-tab shift-down");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta,gamma", "beta"],
        "header select-all must not leave the previous row anchor active"
    );
}

/// Home and End carry Shift range semantics only with the platform secondary
/// modifier in pinned `useSelectableCollection`.
#[gpui::test]
fn table_shift_home_end_require_the_secondary_modifier(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-shift-home-end")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab down space");
    flush_frame(cx);
    press(cx, "shift-end");
    assert_eq!(recorded.borrow().as_slice(), ["alpha"]);
    press(cx, "space");
    flush_frame(cx);
    press(cx, "ctrl-shift-home");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,gamma", "alpha,beta,gamma"],
        "plain Shift+End must only move focus, while Ctrl+Shift+Home extends"
    );
}

/// Pinned row presses route Shift+Click through `extendSelection`, preserving
/// the anchor established by the prior toggle.
#[gpui::test]
fn table_shift_click_extends_from_the_selection_anchor(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let held_for_view = held;
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = held_for_view.borrow().clone();
        let held = held_for_view.clone();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-shift-click")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .selected_keys(selected)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
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

    press(cx, "tab down space");
    flush_frame(cx);
    let mut modifiers = gpui::Modifiers::none();
    modifiers.shift = true;
    cx.simulate_click(gpui::point(px(100.), px(100.)), modifiers);
    flush_frame(cx);
    cx.simulate_click(gpui::point(px(100.), px(100.)), modifiers);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", "alpha,beta"],
        "Shift+Click must extend rather than re-anchor and toggle"
    );
}

/// Focus entry in pinned `useSelectableCollection` seats the cursor on the
/// first row before the first arrow, so Shift+Down targets the second row.
#[gpui::test]
fn table_first_shift_down_starts_after_the_focused_first_row(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-first-shift-down")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab shift-down");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["beta"],
        "the first Shift+Down must move below the row focused on entry"
    );
}

/// React Stately's raw `all` selection is idempotent even when no enabled key
/// materializes into the port's selection slice.
#[gpui::test]
fn table_mod_a_is_idempotent_when_every_row_is_disabled(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-all-disabled")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .disabled_keys([SharedString::from("alpha"), SharedString::from("beta")])
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab ctrl-a");
    flush_frame(cx);
    press(cx, "ctrl-a");
    assert_eq!(
        recorded.borrow().as_slice(),
        [""],
        "repeated Mod+A over an empty selectable collection must report once"
    );
}

/// Fresh focus starts on the first row, but End still moves to the last row;
/// Shift alone changes focus without extending selection.
#[gpui::test]
fn table_first_shift_end_settles_on_the_last_row(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-first-shift-end")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .keyed_row("alpha", vec![gpui::div().child("Alpha").into_any_element()])
            .keyed_row("beta", vec![gpui::div().child("Beta").into_any_element()])
            .keyed_row("gamma", vec![gpui::div().child("Gamma").into_any_element()])
            .on_selection_change(move |keys, _, _| {
                recorded.borrow_mut().push(
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            })
            .into_any_element()
    });

    press(cx, "tab shift-end");
    assert!(recorded.borrow().is_empty());
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["gamma"],
        "Shift+End must focus the last row without selecting on its own"
    );
}

/// A controlled owner can replace a selection after Mod+A; the raw-all latch
/// must then yield to the new prop value rather than swallowing the next Mod+A.
#[gpui::test]
fn table_controlled_replacement_clears_a_stale_mod_a_latch(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let held_for_view = held.clone();
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = held_for_view.borrow().clone();
        let held = held_for_view.clone();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("table-controlled-all-replacement")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
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

    press(cx, "tab ctrl-a");
    flush_frame(cx);
    *held.borrow_mut() = vec![SharedString::from("alpha")];
    flush_frame(cx);
    press(cx, "ctrl-a");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha,beta", "alpha,beta"],
        "an owner replacement must make Mod+A selectable again"
    );
}
