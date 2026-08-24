//! Deeper Tabs keyboard behaviour: the vertical axis and the ends.
//!
//! v3's Tabs API table documents `orientation: "horizontal" | "vertical"`
//! ("Tab layout orientation") and nothing else about the keyboard; the
//! component is built on React Aria Components (v3's own statement of record),
//! whose `useTabList` contract is: a vertical tab list answers Up/Down and
//! ignores Left/Right, a horizontal one answers Left/Right and ignores
//! Up/Down, and Home/End jump to the first/last tab.
//!
//! `collections.rs` already drives horizontal arrows; this file drives the
//! vertical axis (Down must move, the cross-axis Right must be ignored) and
//! the Home/End jumps. Keyboards only — no click geometry to derive.

mod harness;

use gpui::{prelude::*, TestAppContext};
use herogpui_components::{Orientation, TabItem, Tabs};

use harness::{events, open_host, press};

/// Vertical orientation must move the selection with Down and descend back
/// with Up — and a Right key, which belongs to the horizontal axis, must do
/// nothing: React Aria's vertical tab list only roves along its own axis.
#[gpui::test]
fn tabs_vertical_axis_down_moves_right_ignored(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let bubbled = events();
    let outside = bubbled.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let bubbled = bubbled.clone();
        gpui::div()
            .id("tb-vert-outer")
            .on_key_down(move |event, _, _| {
                if matches!(event.keystroke.key.as_str(), "right" | "down") {
                    bubbled.borrow_mut().push(event.keystroke.key.clone());
                }
            })
            .child(
                Tabs::new(
                    "tb-vert",
                    vec![
                        TabItem::new("first", "First"),
                        TabItem::new("second", "Second"),
                        TabItem::new("third", "Third"),
                    ],
                    "first",
                )
                .orientation(Orientation::Vertical)
                .on_selection_change(move |key, _, _| {
                    recorded.borrow_mut().push(key.to_string());
                }),
            )
            .into_any_element()
    });

    // Tab lands on the selected tab, whose roving handle the keyboard travels
    // with as the selection moves, exactly as the horizontal test does.
    press(cx, "tab");
    press(cx, "down");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "a vertical tab list must move the selection down on Down"
    );
    assert!(
        outside.borrow().is_empty(),
        "a consumed axis key must not also move an enclosing navigation control"
    );
    // Right is the horizontal axis's key. A vertical list must ignore it; the
    // selection stays on "second" and no change is reported.
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "a vertical tab list must ignore the horizontal arrow keys"
    );
    assert_eq!(
        outside.borrow().as_slice(),
        ["right"],
        "an ignored cross-axis key must remain available to an enclosing control"
    );
    press(cx, "down");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "after an ignored cross-axis key, Down must still move on"
    );
}

/// Home and End jump the roving selection to the first and last tab
/// respectively — React Aria's `useTabList` is what makes these keys, and the
/// port's `list_nav::resolve` implements them beside the arrows.
#[gpui::test]
fn tabs_home_end_jump_to_the_ends(cx: &mut TestAppContext) {
    let events = events();
    let selected = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Tabs::new(
            "tb-ends",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "end");
    assert_eq!(
        selected.borrow().as_slice(),
        ["third"],
        "End must jump the selection to the last tab"
    );
    press(cx, "home");
    assert_eq!(
        selected.borrow().as_slice(),
        ["third", "first"],
        "Home must jump the selection back to the first tab"
    );
}

/// `selectedKey` is controlled: proposing a different tab reports it, but the
/// rendered selection must remain on the supplied key until the caller changes
/// that prop. Two Right presses therefore both propose `second`; an internal
/// state mutation would make the second press incorrectly propose `third`.
#[gpui::test]
fn tabs_controlled_key_does_not_mutate_behind_the_caller(cx: &mut TestAppContext) {
    let events = events();
    let selected = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Tabs::new(
            "tb-controlled",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "third",
        )
        .selected_key("first")
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "second"],
        "a controlled tab list must not advance until the caller supplies the proposed key"
    );
}
