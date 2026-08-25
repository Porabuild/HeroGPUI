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

use std::{cell::RefCell, rc::Rc};

use gpui::{
    prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, TestAppContext, WindowTextSystem,
};
use herogpui_components::{KeyboardActivation, Orientation, TabItem, Tabs, TabsVariant};

use harness::{click, events, open_host, press};

fn text_width(system: &WindowTextSystem, text: &str) -> f32 {
    let run = gpui::TextRun {
        len: text.len(),
        font: Font {
            family: ".SystemUIFont".into(),
            features: FontFeatures::default(),
            weight: FontWeight::MEDIUM,
            style: FontStyle::default(),
            fallbacks: None,
        },
        color: gpui::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line = system.shape_line(text.to_owned().into(), px(14.), &[run], None);
    f32::from(line.width)
}

/// `Tabs.Tab.isDisabled` is per item, not the root's disabled state. A dead
/// tab takes neither a click nor a roving stop, while its siblings remain live.
#[gpui::test]
fn tabs_disabled_item_is_skipped_by_keys_and_clicks(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            "tb-disabled-item",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second").is_disabled(true),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    let first = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    let second = cx.update(|window, _| text_width(window.text_system(), "Second")) + 32.;
    click(cx, 4. + first + second / 2., 20.);
    assert!(
        selected.borrow().is_empty(),
        "a disabled tab must not answer the pointer"
    );

    press(cx, "tab");
    press(cx, "right");
    press(cx, "left");
    press(cx, "end");
    press(cx, "home");
    assert_eq!(
        selected.borrow().as_slice(),
        ["third", "first", "third", "first"],
        "arrows and Home/End must skip the disabled middle tab"
    );
}

/// Theme changes rebuild every rendered element. The uncontrolled selection is
/// keyed component state, so switching from light to dark must not reseed it
/// from `defaultSelectedKey` between two arrow presses.
#[gpui::test]
fn tabs_uncontrolled_selection_survives_dark_mode_switch(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            "tb-dark-state",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    cx.update(|window, cx| {
        herogpui_theme::toggle_light_dark(cx);
        window.refresh();
    });
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "a theme rebuild must not reseed uncontrolled component state"
    );
}

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
/// rendered selection remains on the supplied key until the caller changes
/// that prop. Roving focus is independent, so consecutive Right presses report
/// `second` and then `third`.
#[gpui::test]
fn tabs_controlled_selection_keeps_independent_roving_focus(cx: &mut TestAppContext) {
    let proposals = events();
    let selected = proposals.clone();
    let panels = events();
    let clicked_panel = panels.clone();
    let cx = open_host(cx, move || {
        let events = proposals.clone();
        let first_panel = panels.clone();
        let second_panel = panels.clone();
        let third_panel = panels.clone();
        Tabs::new(
            "tb-controlled",
            vec![
                TabItem::new("first", "First").content(
                    gpui::canvas(
                        move |_, _, _| first_panel.borrow_mut().push("first".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
                TabItem::new("second", "Second").content(
                    gpui::canvas(
                        move |_, _, _| second_panel.borrow_mut().push("second".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
                TabItem::new("third", "Third").content(
                    gpui::canvas(
                        move |_, _, _| third_panel.borrow_mut().push("third".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
            ],
            "third",
        )
        .selected_key("first")
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    clicked_panel.borrow_mut().clear();
    press(cx, "right");
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "controlled selection must not prevent the roving focus key from advancing"
    );
    assert!(
        !clicked_panel.borrow().is_empty()
            && clicked_panel.borrow().iter().all(|panel| panel == "first"),
        "controlled arrow proposals must keep rendering only the caller-selected panel"
    );
}

/// Pinned `useTabListState` synchronizes focusedKey to selectedKey only while
/// the tab list is not focused. An owner update during keyboard navigation
/// changes the panel but must not change the next arrow's starting tab.
#[gpui::test]
fn tabs_controlled_owner_update_does_not_steal_focused_key(cx: &mut TestAppContext) {
    let selected_key = Rc::new(RefCell::new(String::from("first")));
    let selected_for_view = selected_key.clone();
    let proposals = events();
    let recorded = proposals.clone();
    let cx = open_host(cx, move || {
        let proposals = proposals.clone();
        Tabs::new(
            "tb-controlled-owner-update",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .selected_key(selected_for_view.borrow().clone())
        .on_selection_change(move |key, _, _| proposals.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    *selected_key.borrow_mut() = String::from("third");
    cx.update(|window, _| window.refresh());
    press(cx, "right");

    assert_eq!(
        recorded.borrow().as_slice(),
        ["second", "third"],
        "an owner selection update while focused must not reset the arrow-key origin"
    );
}

/// Pinned list state repairs a removed focused key to the next surviving
/// enabled key, falling back to the previous one only when there is no next
/// item. The controlled selection remains independent throughout.
#[gpui::test]
fn tabs_removed_focused_key_repairs_to_the_next_survivor(cx: &mut TestAppContext) {
    let keys = Rc::new(RefCell::new(vec!["first", "second", "third"]));
    let keys_for_view = keys.clone();
    let proposals = events();
    let recorded = proposals.clone();
    let cx = open_host(cx, move || {
        let proposals = proposals.clone();
        let items = keys_for_view
            .borrow()
            .iter()
            .map(|key| TabItem::new(*key, *key))
            .collect();
        Tabs::new("tb-controlled-removal", items, "first")
            .selected_key("first")
            .on_selection_change(move |key, _, _| proposals.borrow_mut().push(key.to_string()))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    keys.borrow_mut().remove(1);
    cx.update(|window, _| window.refresh());
    press(cx, "right");

    assert_eq!(
        recorded.borrow().as_slice(),
        ["second", "first"],
        "removing the focused second tab must repair focus to third before Right wraps to first"
    );
}

/// HeroUI v3.2.4 pins `react-aria` 3.51.0 and `react-stately` 3.49.0: the
/// vertical tab delegate maps Down to the next tab, while controlled state
/// reports the proposal without replacing the caller's selected key.
#[gpui::test]
fn tabs_controlled_secondary_vertical_advances_roving_focus(cx: &mut TestAppContext) {
    let proposals = events();
    let selected = proposals.clone();
    let panels = events();
    let clicked_panel = panels.clone();
    let cx = open_host(cx, move || {
        let events = proposals.clone();
        let first_panel = panels.clone();
        let second_panel = panels.clone();
        let third_panel = panels.clone();
        Tabs::new(
            "tb-controlled-secondary-vertical",
            vec![
                TabItem::new("first", "First").content(
                    gpui::canvas(
                        move |_, _, _| first_panel.borrow_mut().push("first".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
                TabItem::new("second", "Second").content(
                    gpui::canvas(
                        move |_, _, _| second_panel.borrow_mut().push("second".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
                TabItem::new("third", "Third").content(
                    gpui::canvas(
                        move |_, _, _| third_panel.borrow_mut().push("third".into()),
                        |_, _, _, _| {},
                    )
                    .size(px(32.)),
                ),
            ],
            "first",
        )
        .variant(TabsVariant::Secondary)
        .orientation(Orientation::Vertical)
        .selected_key("first")
        .on_selection_change(move |key, _, _| events.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    clicked_panel.borrow_mut().clear();
    press(cx, "down");
    press(cx, "down");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "controlled vertical tabs must move roving focus without mutating the selected key"
    );
    assert!(
        !clicked_panel.borrow().is_empty()
            && clicked_panel.borrow().iter().all(|panel| panel == "first"),
        "controlled vertical arrow proposals must keep rendering only the first panel"
    );
}

/// Pinned React Aria's `keyboardActivation="manual"` moves the roving focus
/// with arrows without selecting. Enter activates the focused tab afterward,
/// so a controlled owner receives exactly one proposal at that point.
#[gpui::test]
fn tabs_manual_activation_waits_for_enter(cx: &mut TestAppContext) {
    let recorded = events();
    let for_view = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = for_view.clone();
        Tabs::new(
            "tb-manual",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .selected_key("first")
        .keyboard_activation(KeyboardActivation::Manual)
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    assert!(
        recorded.borrow().is_empty(),
        "manual activation must not select merely because focus moved"
    );

    cx.update(|window, _| window.refresh());
    press(cx, "enter");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["second"],
        "Enter must activate the manually focused tab"
    );
}
