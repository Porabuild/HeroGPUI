//! Collection contracts inherited from HeroUI v3.2.4's pinned React Aria
//! 3.51.0 and React Stately 3.49.0 releases.
//!
//! These are deliberately interaction tests. The prop and anatomy audits
//! cannot observe focus entry, distinguish Space selection from Enter action,
//! or tell whether a still-visible load-more sentinel was re-armed after its
//! collection was replaced.

mod harness;

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use gpui::{prelude::*, px, SharedString, TestAppContext, VisualTestContext};
use herogpui_components::{
    Button, ListBox, ListBoxItem, SelectionMode, TabItem, Table, TableColumn, TableRow, Tabs, Tag,
    TagGroup, VirtualTreeMetadata,
};

use harness::{click, events, open_host, press};

fn sorted_join(keys: &HashSet<SharedString>) -> String {
    let mut keys: Vec<String> = keys.iter().map(ToString::to_string).collect();
    keys.sort();
    keys.join(",")
}

fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

fn tall_cell(text: impl Into<SharedString>) -> gpui::AnyElement {
    gpui::div()
        .h(px(80.))
        .flex()
        .items_center()
        .child(text.into())
        .into_any_element()
}

/// React Stately's `SelectionManager::toggleSelection` removes an already
/// selected key unless `disallowEmptySelection` is true. HeroUI does not
/// expose that inherited prop in its v3 table, so this pins its default false
/// behavior without inventing a builder.
#[gpui::test]
fn list_box_single_reselect_clears_by_default(cx: &mut TestAppContext) {
    let selected = Rc::new(RefCell::new(HashSet::<SharedString>::new()));
    let recorded = events();
    let selected_for_view = selected;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = selected_for_view.clone();
        let held = selected.borrow().clone();
        let recorded = for_view.clone();
        ListBox::new(
            "contract-list-clear",
            vec![ListBoxItem::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .selected_keys(held)
        .on_selection_change(move |keys, window, _| {
            *selected.borrow_mut() = keys.clone();
            recorded.borrow_mut().push(sorted_join(keys));
            window.refresh();
        })
        .into_any_element()
    });

    // p-1 plus half a 36px row puts its centre at (60, 22).
    click(cx, 60., 22.);
    flush_frame(cx);
    click(cx, 60., 22.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha", ""],
        "reselecting the only selected row must clear the selection"
    );
}

#[gpui::test]
fn list_box_default_selected_keys_seed_uncontrolled_state(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ListBox::new(
            "contract-list-default-selection",
            vec![ListBoxItem::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .default_selected_keys([SharedString::from("alpha")])
        .on_selection_change(move |keys, window, _| {
            recorded.borrow_mut().push(sorted_join(keys));
            window.refresh();
        })
        .into_any_element()
    });

    click(cx, 60., 22.);
    flush_frame(cx);
    click(cx, 60., 22.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["", "alpha"],
        "the uncontrolled selection must advance from its seed after each press"
    );
}

#[gpui::test]
fn list_box_controlled_empty_overrides_the_uncontrolled_seed(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ListBox::new(
            "contract-list-controlled-empty",
            vec![ListBoxItem::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .default_selected_keys([SharedString::from("alpha")])
        .selected_keys(Vec::<SharedString>::new())
        .on_selection_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    click(cx, 60., 22.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha"],
        "an explicit controlled empty set must not read the uncontrolled seed"
    );
}

#[gpui::test]
fn list_box_instances_keep_separate_keyed_selection(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let first = for_view.clone();
        let second = for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(
                ListBox::new(
                    "contract-list-instance-one",
                    vec![ListBoxItem::new("alpha", "Alpha")],
                )
                .on_selection_change(move |keys, window, _| {
                    first
                        .borrow_mut()
                        .push(format!("one:{}", sorted_join(keys)));
                    window.refresh();
                }),
            )
            .child(
                ListBox::new(
                    "contract-list-instance-two",
                    vec![ListBoxItem::new("alpha", "Alpha")],
                )
                .on_selection_change(move |keys, _, _| {
                    second
                        .borrow_mut()
                        .push(format!("two:{}", sorted_join(keys)));
                }),
            )
            .into_any_element()
    });

    click(cx, 60., 22.);
    flush_frame(cx);
    click(cx, 60., 66.);
    assert_eq!(recorded.borrow().as_slice(), ["one:alpha", "two:alpha"]);
}

/// `useSelectableCollection` moves focus on entry to the first selected item,
/// or the first enabled item when there is no selection. Enter must therefore
/// work immediately after Tab, without an arrow key first.
#[gpui::test]
fn list_box_tab_entry_targets_selected_then_first_item(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected_events = for_view.clone();
        let first_events = for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(
                ListBox::new(
                    "contract-list-selected-entry",
                    vec![
                        ListBoxItem::new("alpha", "Alpha"),
                        ListBoxItem::new("beta", "Beta"),
                    ],
                )
                .selection_mode(SelectionMode::None)
                .selected_key("beta")
                .on_action(move |key, _, _| {
                    selected_events.borrow_mut().push(format!("selected:{key}"));
                }),
            )
            .child(
                ListBox::new(
                    "contract-list-first-entry",
                    vec![
                        ListBoxItem::new("alpha", "Alpha"),
                        ListBoxItem::new("beta", "Beta"),
                    ],
                )
                .on_action(move |key, _, _| {
                    first_events.borrow_mut().push(format!("first:{key}"));
                }),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "enter");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["selected:beta", "first:alpha"],
        "focus entry must target the selected item, then the first item when none is selected"
    );
}

/// Pinned `useSelectableItem` makes Space the selection key and Enter the
/// action key when an item has both behaviors. Neither key may cross-fire the
/// other callback.
#[gpui::test]
fn list_box_space_selects_and_enter_obeys_toggle_action_priority(cx: &mut TestAppContext) {
    let selected = Rc::new(RefCell::new(HashSet::<SharedString>::new()));
    let recorded = events();
    let selected_for_view = selected;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = selected_for_view.clone();
        let held = selected.borrow().clone();
        let actions = for_view.clone();
        let selections = for_view.clone();
        ListBox::new(
            "contract-list-key-intent",
            vec![ListBoxItem::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .selected_keys(held)
        .on_action(move |key, _, _| actions.borrow_mut().push(format!("action:{key}")))
        .on_selection_change(move |keys, window, _| {
            *selected.borrow_mut() = keys.clone();
            selections
                .borrow_mut()
                .push(format!("selection:{}", sorted_join(keys)));
            window.refresh();
        })
        .into_any_element()
    });

    // Establish the cursor independently of the separate focus-entry contract.
    press(cx, "tab");
    press(cx, "down");
    recorded.borrow_mut().clear();
    press(cx, "enter");
    press(cx, "space");
    flush_frame(cx);
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:alpha", "selection:alpha"],
        "Enter performs the primary action while empty, Space selects, and Enter is inert once toggle selection is active"
    );
}

#[gpui::test]
fn list_box_pointer_action_and_selection_are_mutually_exclusive(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let empty_actions = for_view.clone();
        let empty_selections = for_view.clone();
        let selected_actions = for_view.clone();
        let selected_selections = for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(
                ListBox::new(
                    "contract-list-pointer-action",
                    vec![ListBoxItem::new("alpha", "Alpha")],
                )
                .selection_mode(SelectionMode::Single)
                .on_action(move |key, _, _| {
                    empty_actions
                        .borrow_mut()
                        .push(format!("empty-action:{key}"));
                })
                .on_selection_change(move |keys, _, _| {
                    empty_selections
                        .borrow_mut()
                        .push(format!("empty-selection:{}", sorted_join(keys)));
                }),
            )
            .child(
                ListBox::new(
                    "contract-list-pointer-selection",
                    vec![ListBoxItem::new("alpha", "Alpha")],
                )
                .selection_mode(SelectionMode::Single)
                .default_selected_keys([SharedString::from("alpha")])
                .on_action(move |key, _, _| {
                    selected_actions
                        .borrow_mut()
                        .push(format!("selected-action:{key}"));
                })
                .on_selection_change(move |keys, _, _| {
                    selected_selections
                        .borrow_mut()
                        .push(format!("selected-selection:{}", sorted_join(keys)));
                }),
            )
            .into_any_element()
    });

    click(cx, 60., 22.);
    click(cx, 60., 66.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["empty-action:alpha", "selected-selection:"],
        "an empty toggle collection gives the pointer to its primary action, while a selected collection gives it to selection"
    );
}

#[gpui::test]
fn list_box_pointer_focuses_the_pressed_row_for_immediate_enter(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        ListBox::new(
            "contract-list-pointer-focus",
            vec![
                ListBoxItem::new("alpha", "Alpha"),
                ListBoxItem::new("beta", "Beta"),
            ],
        )
        .selection_mode(SelectionMode::None)
        .on_action(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    click(cx, 60., 66.);
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["beta", "beta"],
        "the row pressed by the pointer must become the keyboard action target"
    );
}

/// `useTagGroup` constructs a horizontal `ListKeyboardDelegate` with
/// `shouldFocusWrap: true`, so moving left from the first enabled tag lands on
/// the last enabled tag.
#[gpui::test]
fn tag_group_horizontal_arrows_wrap(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        TagGroup::new(
            "contract-tags-wrap",
            vec![
                Tag::new("alpha", "Alpha"),
                Tag::new("beta", "Beta"),
                Tag::new("gamma", "Gamma"),
            ],
        )
        .selection_mode(SelectionMode::Single)
        .on_selection_change(move |keys, _, _| recorded.borrow_mut().push(sorted_join(keys)))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "left");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["gamma"],
        "Left from the first tag must wrap to the last tag"
    );
}

#[gpui::test]
fn tag_group_default_selected_keys_seed_uncontrolled_state(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        TagGroup::new(
            "contract-tags-default-selection",
            vec![Tag::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .default_selected_keys([SharedString::from("alpha")])
        .on_selection_change(move |keys, window, _| {
            recorded.borrow_mut().push(sorted_join(keys));
            window.refresh();
        })
        .into_any_element()
    });

    click(cx, 30., 14.);
    flush_frame(cx);
    click(cx, 30., 14.);
    assert_eq!(recorded.borrow().as_slice(), ["", "alpha"]);
}

#[gpui::test]
fn tag_group_controlled_empty_overrides_the_uncontrolled_seed(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        TagGroup::new(
            "contract-tags-controlled-empty",
            vec![Tag::new("alpha", "Alpha")],
        )
        .selection_mode(SelectionMode::Single)
        .default_selected_keys([SharedString::from("alpha")])
        .selected_keys(Vec::<SharedString>::new())
        .on_selection_change(move |keys, _, _| {
            recorded.borrow_mut().push(sorted_join(keys));
        })
        .into_any_element()
    });

    click(cx, 30., 14.);
    assert_eq!(recorded.borrow().as_slice(), ["alpha"]);
}

/// A Tag remove button is an action nested inside the selectable tag. Its
/// press removes the tag and must not bubble into the tag's selection press.
#[gpui::test]
fn tag_group_remove_button_does_not_toggle_selection(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let removed = for_view.clone();
        let selected = for_view.clone();
        TagGroup::new(
            "contract-tags-remove",
            vec![Tag::new("alpha", "Alpha"), Tag::new("beta", "Beta")],
        )
        .selection_mode(SelectionMode::Single)
        .tag_content(|_, _| gpui::div().w(px(40.)).h(px(20.)).into_any_element())
        .on_remove(move |key, _, _| removed.borrow_mut().push(format!("remove:{key}")))
        .on_selection_change(move |keys, _, _| {
            selected
                .borrow_mut()
                .push(format!("selection:{}", sorted_join(keys)));
        })
        .into_any_element()
    });

    // A 72px tag plus 6px gap puts Beta's 12px remove button centre at x=136.
    click(cx, 136., 14.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["remove:beta"],
        "the nested remove action must not also select Beta"
    );
}

/// React Stately repairs an uncontrolled tab selection when the selected item
/// disappears, choosing the first enabled item. The new panel must therefore
/// become interactive after the collection update.
#[gpui::test]
fn tabs_uncontrolled_selection_recovers_when_selected_item_disappears(cx: &mut TestAppContext) {
    let include_beta = Rc::new(Cell::new(true));
    let recorded = events();
    let include_for_view = include_beta.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let alpha_events = for_view.clone();
        let mut items = vec![TabItem::new("alpha", "Alpha").content(
            gpui::div()
                .id("contract-alpha-panel")
                .w(px(80.))
                .h(px(40.))
                .on_click(move |_, _, _| alpha_events.borrow_mut().push("alpha-panel".into())),
        )];
        if include_for_view.get() {
            items.push(TabItem::new("beta", "Beta").content(gpui::div().h(px(40.))));
        }
        gpui::div()
            .w(px(240.))
            .child(
                Tabs::new("contract-tabs-recover", items, "beta").on_selection_change({
                    let recorded = for_view.clone();
                    move |key, _, _| recorded.borrow_mut().push(format!("selection:{key}"))
                }),
            )
            .into_any_element()
    });

    include_beta.set(false);
    flush_frame(cx);
    flush_frame(cx);
    // The 40px list plus 8px gap and 8px panel padding put the probe at y=60.
    click(cx, 20., 60.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["selection:alpha", "alpha-panel"],
        "removing the selected tab must select and render the first remaining tab"
    );
}

/// React Aria applies selectable-row press props to the whole table row. The
/// checkbox is one affordance, not the only place a row can be selected.
#[gpui::test]
fn table_row_press_outside_checkbox_selects(cx: &mut TestAppContext) {
    let held = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let recorded = events();
    let held_for_view = held;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let held = held_for_view.clone();
        let selected = held.borrow().clone();
        let recorded = for_view.clone();
        gpui::div()
            .w(px(204.))
            .child(
                Table::new(vec![])
                    .id("contract-table-row-select")
                    .columns(vec![TableColumn::new("Name").default_width(px(160.))])
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(selected)
                    .keyed_row("alpha", vec![tall_cell("Alpha")])
                    .on_selection_change(move |keys, window, _| {
                        *held.borrow_mut() = keys.to_vec();
                        recorded.borrow_mut().push(
                            keys.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                        window.refresh();
                    }),
            )
            .into_any_element()
    });

    // x=100 is in the 160px data cell, outside the 44px checkbox column;
    // y=90 is the centre of the 105px row below the ~37px header.
    click(cx, 100., 90.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["alpha"],
        "pressing a row's data cell must select the row"
    );
}

#[gpui::test]
fn table_action_and_selection_arbitrate_pointer_enter_and_space(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let actions = for_view.clone();
        let selections = for_view.clone();
        Table::new(vec![])
            .id("contract-table-action-selection")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Single)
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .on_row_click(move |index, _, _, _| {
                actions.borrow_mut().push(format!("action:{index}"));
            })
            .on_selection_change(move |keys, window, _| {
                selections.borrow_mut().push(format!(
                    "selection:{}",
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                window.refresh();
            })
            .into_any_element()
    });

    click(cx, 100., 90.);
    press(cx, "space");
    flush_frame(cx);
    press(cx, "enter");
    click(cx, 100., 90.);
    flush_frame(cx);
    click(cx, 100., 90.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:0", "selection:alpha", "selection:", "action:0"],
        "pointer and Enter perform the primary action only while toggle selection is empty; Space selects, and no activation cross-fires"
    );
}

#[gpui::test]
fn table_pointer_row_press_sets_the_keyboard_cursor(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-pointer-cursor")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::None)
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .keyed_row("beta", vec![tall_cell("Beta")])
            .keyed_row("gamma", vec![tall_cell("Gamma")])
            .on_row_click(move |index, _, _, _| {
                recorded.borrow_mut().push(format!("action:{index}"));
            })
            .into_any_element()
    });

    click(cx, 100., 195.);
    press(cx, "enter");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:1", "action:1", "action:2"],
        "Enter must reuse the pressed row and Down must advance from it"
    );
}

#[gpui::test]
fn table_checkbox_press_does_not_move_the_row_cursor(cx: &mut TestAppContext) {
    let selected = Rc::new(RefCell::new(Vec::<SharedString>::new()));
    let recorded = events();
    let selected_for_view = selected;
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let selected = selected_for_view.clone();
        let held = selected.borrow().clone();
        let recorded = for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(Button::new("before-checkbox-table").label("Before"))
            .child(
                Table::new(vec![])
                    .id("contract-table-checkbox-cursor")
                    .columns(vec![TableColumn::new("Name").default_width(px(160.))])
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(held)
                    .keyed_row("alpha", vec![tall_cell("Alpha")])
                    .keyed_row("beta", vec![tall_cell("Beta")])
                    .on_selection_change(move |keys, window, _| {
                        *selected.borrow_mut() = keys.to_vec();
                        recorded.borrow_mut().push(
                            keys.iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                        window.refresh();
                    }),
            )
            .into_any_element()
    });

    click(cx, 100., 231.);
    flush_frame(cx);
    click(cx, 22., 126.);
    flush_frame(cx);
    click(cx, 40., 18.);
    press(cx, "tab");
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["beta", "beta,alpha", "alpha"],
        "the checkbox may select Alpha, but Space must still answer from the Beta row cursor"
    );
}

#[gpui::test]
fn table_uncontrolled_selection_persists_between_row_presses(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-uncontrolled")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Single)
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .on_selection_change(move |keys, window, _| {
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

    click(cx, 100., 90.);
    flush_frame(cx);
    click(cx, 100., 90.);
    assert_eq!(recorded.borrow().as_slice(), ["alpha", ""]);
}

#[gpui::test]
fn table_controlled_empty_remains_caller_owned_between_presses(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-controlled-empty")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Single)
            .selected_keys(Vec::<SharedString>::new())
            .keyed_row("alpha", vec![tall_cell("Alpha")])
            .on_selection_change(move |keys, window, _| {
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

    click(cx, 100., 90.);
    flush_frame(cx);
    click(cx, 100., 90.);
    assert_eq!(recorded.borrow().as_slice(), ["alpha", "alpha"]);
}

#[gpui::test]
fn table_nested_row_press_uses_the_flattened_tree_key(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-tree-key")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Single)
            .expanded_keys([SharedString::from("parent")])
            .tree_row(
                TableRow::new(vec![tall_cell("Parent")])
                    .key("parent")
                    .children(vec![TableRow::new(vec![tall_cell("Child")]).key("child")]),
            )
            .keyed_row("sibling", vec![tall_cell("Sibling")])
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

    click(cx, 120., 190.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["child"],
        "the child row must not borrow the following top-level row's key"
    );
}

#[gpui::test]
fn table_select_all_includes_collapsed_descendants(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-select-all-tree")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::Multiple)
            .selected_keys(["child"])
            .tree_row(
                TableRow::new(vec![tall_cell("Parent")])
                    .key("parent")
                    .children(vec![TableRow::new(vec![tall_cell("Child")]).key("child")]),
            )
            .keyed_row("sibling", vec![tall_cell("Sibling")])
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

    click(cx, 22., 18.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["parent,child,sibling"],
        "a partially selected collapsed tree must select the full recursive collection"
    );
}

#[gpui::test]
fn table_interactive_cell_child_owns_its_pointer_press(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let button_events = for_view.clone();
        let selection_events = for_view.clone();
        let action_events = for_view.clone();
        gpui::div()
            .flex()
            .flex_col()
            .child(Button::new("before-interactive-table").label("Before"))
            .child(
                Table::new(vec![])
                    .id("contract-table-interactive-child")
                    .columns(vec![TableColumn::new("Action").default_width(px(160.))])
                    .selection_mode(SelectionMode::Single)
                    .keyed_row(
                        "alpha",
                        vec![gpui::div()
                            .child(Button::new("cell-action").label("Open").on_press(
                                move |_, _, _| {
                                    button_events.borrow_mut().push("button".into());
                                },
                            ))
                            .into_any_element()],
                    )
                    .keyed_row("beta", vec![tall_cell("Beta")])
                    .on_row_click(move |index, _, _, _| {
                        action_events.borrow_mut().push(format!("action:{index}"));
                    })
                    .on_selection_change(move |_, _, _| {
                        selection_events.borrow_mut().push("selection".into());
                    }),
            )
            .into_any_element()
    });

    click(cx, 180., 181.);
    click(cx, 90., 103.);
    press(cx, "enter");
    click(cx, 40., 18.);
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:1", "button", "button", "action:1"],
        "a nested focusable action must own both pointer and keyboard activation without moving the row cursor"
    );
}

#[gpui::test]
fn table_passive_non_div_cell_content_activates_its_row(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-passive-svg")
            .columns(vec![TableColumn::new("Status").default_width(px(160.))])
            .keyed_row(
                "alpha",
                vec![gpui::svg()
                    .size(px(20.))
                    .path(herogpui_components::icons::CHECK)
                    .into_any_element()],
            )
            .on_row_click(move |index, _, _, _| {
                recorded.borrow_mut().push(format!("action:{index}"));
            })
            .into_any_element()
    });

    click(cx, 70., 70.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:0"],
        "a passive non-Div child must not be mistaken for an interactive control"
    );
}

#[gpui::test]
fn table_tree_chevron_moves_the_row_cursor_to_its_row(cx: &mut TestAppContext) {
    let expanded = Rc::new(Cell::new(true));
    let recorded = events();
    let expanded_for_view = expanded.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let action_events = for_view.clone();
        let expanded_events = for_view.clone();
        let expanded_state = expanded.clone();
        Table::new(vec![])
            .id("contract-table-chevron-cursor")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .selection_mode(SelectionMode::None)
            .expanded_keys(
                expanded_for_view
                    .get()
                    .then(|| SharedString::from("parent")),
            )
            .tree_row(
                TableRow::new(vec![tall_cell("Parent")])
                    .key("parent")
                    .children(vec![TableRow::new(vec![tall_cell("Child")]).key("child")]),
            )
            .keyed_row("sibling", vec![tall_cell("Sibling")])
            .on_row_click(move |index, _, _, _| {
                action_events.borrow_mut().push(format!("action:{index}"));
            })
            .on_expanded_change(move |keys, window, _| {
                expanded_state.set(!keys.is_empty());
                expanded_events.borrow_mut().push(format!(
                    "expanded:{}",
                    keys.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                ));
                window.refresh();
            })
            .into_any_element()
    });

    click(cx, 120., 300.);
    click(cx, 29., 90.);
    flush_frame(cx);
    press(cx, "enter");
    press(cx, "up");
    press(cx, "enter");
    press(cx, "down");
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["action:2", "expanded:", "action:0", "action:0", "action:1"],
        "the chevron must restore its row focus before the collapsed collection remaps navigation"
    );
}

/// `useLoadMoreSentinel` tears down and recreates its observer whenever the
/// collection object changes. Replacing one row with another must therefore
/// re-arm a visible sentinel even though the collection length is unchanged.
#[gpui::test]
fn table_load_more_rearms_after_same_length_collection_replacement(cx: &mut TestAppContext) {
    let second_page = Rc::new(Cell::new(false));
    let recorded = events();
    let page_for_view = second_page.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-load-replace")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .scroll_offset(0.)
            .keyed_row(
                if page_for_view.get() { "beta" } else { "alpha" },
                vec![tall_cell(if page_for_view.get() {
                    "Beta"
                } else {
                    "Alpha"
                })],
            )
            .on_load_more(move |_, _| recorded.borrow_mut().push("load-more".into()))
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(recorded.borrow().as_slice(), ["load-more"]);
    second_page.set(true);
    flush_frame(cx);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["load-more", "load-more"],
        "replacing a visible same-length collection must re-observe its sentinel"
    );
}

#[gpui::test]
fn table_load_more_does_not_rearm_for_expansion_only(cx: &mut TestAppContext) {
    let expanded = Rc::new(Cell::new(false));
    let recorded = events();
    let expanded_for_view = expanded.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let expanded = expanded_for_view.get();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-table-load-expand")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .scroll_offset(0.)
            .expanded_keys(expanded.then(|| SharedString::from("parent")))
            .tree_row(
                TableRow::new(vec![tall_cell("Parent")])
                    .key("parent")
                    .children(vec![TableRow::new(vec![tall_cell("Child")]).key("child")]),
            )
            .on_expanded_change({
                let expanded = expanded_for_view.clone();
                move |keys, window, _| {
                    expanded.set(!keys.is_empty());
                    window.refresh();
                }
            })
            .on_load_more(move |_, _| recorded.borrow_mut().push("load-more".into()))
            .into_any_element()
    });

    flush_frame(cx);
    press(cx, "tab");
    press(cx, "down");
    press(cx, "right");
    flush_frame(cx);
    assert!(expanded.get(), "the tree key must expand the branch");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["load-more"],
        "expansion changes visibility, not the underlying collection identity"
    );
}

#[gpui::test]
fn virtual_table_load_more_does_not_rearm_for_expansion_only(cx: &mut TestAppContext) {
    let expanded = Rc::new(Cell::new(false));
    let recorded = events();
    let expanded_for_view = expanded.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let expanded = expanded_for_view.get();
        let recorded = for_view.clone();
        Table::new(vec![])
            .id("contract-virtual-table-load-expand")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .tree_column(0)
            .row_height(px(80.))
            .max_h(px(160.))
            .scroll_offset(0.)
            .expanded_keys(expanded.then(|| SharedString::from("parent")))
            .virtual_rows(
                2,
                "virtual-tree-data",
                |index| ["parent", "child"][index].into(),
                |index| TableRow::new(vec![tall_cell(["Parent", "Child"][index])]),
            )
            .virtual_tree_metadata(|index| {
                if index == 0 {
                    VirtualTreeMetadata {
                        depth: 0,
                        parent_key: None,
                        has_children: true,
                    }
                } else {
                    VirtualTreeMetadata {
                        depth: 1,
                        parent_key: Some("parent".into()),
                        has_children: false,
                    }
                }
            })
            .on_expanded_change({
                let expanded = expanded_for_view.clone();
                move |keys, window, _| {
                    expanded.set(!keys.is_empty());
                    window.refresh();
                }
            })
            .on_load_more(move |_, _| recorded.borrow_mut().push("load-more".into()))
            .into_any_element()
    });

    flush_frame(cx);
    press(cx, "tab");
    press(cx, "down");
    press(cx, "right");
    flush_frame(cx);
    assert!(
        expanded.get(),
        "the virtual tree key must expand the branch"
    );
    assert_eq!(
        recorded.borrow().as_slice(),
        ["load-more"],
        "virtual expansion changes the visible flatten, not the underlying collection identity"
    );
}

#[gpui::test]
fn virtual_table_load_more_rearms_after_same_count_key_replacement(cx: &mut TestAppContext) {
    let second_page = Rc::new(Cell::new(false));
    let factory_calls = Rc::new(Cell::new(0usize));
    let recorded = events();
    let page_for_view = second_page.clone();
    let calls_for_view = factory_calls.clone();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let page = page_for_view.clone();
        let key_page = page_for_view.clone();
        let calls = calls_for_view.clone();
        let recorded = for_view.clone();
        let identity = if page_for_view.get() {
            "beta-page"
        } else {
            "alpha-page"
        };
        Table::new(vec![])
            .id("contract-virtual-table-load-replace")
            .columns(vec![TableColumn::new("Name").default_width(px(160.))])
            .row_height(px(80.))
            .max_h(px(160.))
            .scroll_offset(0.)
            .virtual_rows(
                1000,
                identity,
                move |index| {
                    if key_page.get() {
                        format!("beta-{index}").into()
                    } else {
                        format!("alpha-{index}").into()
                    }
                },
                move |index| {
                    calls.set(calls.get() + 1);
                    if page.get() {
                        TableRow::new(vec![tall_cell(format!("Beta {index}"))])
                    } else {
                        TableRow::new(vec![tall_cell(format!("Alpha {index}"))])
                    }
                },
            )
            .on_load_more(move |_, _| recorded.borrow_mut().push("load-more".into()))
            .into_any_element()
    });

    flush_frame(cx);
    assert_eq!(recorded.borrow().as_slice(), ["load-more"]);
    let first_page_calls = factory_calls.get();
    assert!(
        first_page_calls <= 16,
        "two frames of a 160px viewport over 80px rows must stay within visible rows plus bounded overdraw; observed {first_page_calls}"
    );
    second_page.set(true);
    flush_frame(cx);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["load-more", "load-more"],
        "a virtual same-count collection replacement must carry its explicit identity into the sentinel"
    );
    assert!(
        factory_calls.get() - first_page_calls <= 16,
        "replacing the collection must keep factory work bounded to the viewport; observed {}",
        factory_calls.get() - first_page_calls
    );
}
