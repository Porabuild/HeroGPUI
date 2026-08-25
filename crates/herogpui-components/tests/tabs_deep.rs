//! Deeper Tabs keyboard behaviour: the vertical axis and the ends.
//!
//! v3's Tabs API table documents `orientation: "horizontal" | "vertical"`
//! ("Tab layout orientation") and nothing else about the keyboard; the
//! component is built on React Aria Components (v3's own statement of record),
//! whose `TabsKeyboardDelegate` contract is: a vertical tab list answers both
//! Up/Down and Left/Right, a horizontal one answers Left/Right and ignores
//! Up/Down, and Home/End jump to the first/last tab.
//!
//! `collections.rs` already drives basic horizontal arrows; this file drives
//! both vertical axes, end wrapping, pointer-to-keyboard focus handoff, and
//! orientation layout.

mod harness;

use std::{cell::RefCell, rc::Rc, time::Duration};

use gpui::{
    prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, TestAppContext, VisualTestContext,
    WindowTextSystem,
};
use herogpui_components::{KeyboardActivation, Orientation, TabItem, Tabs, TabsVariant};

use harness::{click, events, open_host, press};

fn flush_frame(cx: &mut VisualTestContext) {
    cx.update(|window, _| window.refresh());
}

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

/// A pointer press selects and focuses the tab in pinned React Aria. The next
/// arrow therefore starts at the pressed tab rather than at the old keyboard
/// stop or outside the list.
#[gpui::test]
fn tabs_pointer_selection_hands_off_to_roving_keyboard_focus(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            "tb-pointer-focus",
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

    let first = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    let second = cx.update(|window, _| text_width(window.text_system(), "Second")) + 32.;
    click(cx, 4. + first + second / 2., 20.);
    cx.update(|window, _| window.refresh());
    press(cx, "right");

    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "Right after a pointer selection must continue from the pressed tab"
    );
}

/// The pinned SelectionIndicator is one measured child that owns the selected
/// pill. It starts on the selected tab and settles onto the next tab after its
/// 250ms geometry transition.
#[gpui::test]
fn tabs_indicator_moves_between_measured_tab_boxes(cx: &mut TestAppContext) {
    let cx = open_host(cx, move || {
        Tabs::new(
            "tb-indicator-geometry",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
            ],
            "first",
        )
        .into_any_element()
    });

    flush_frame(cx);
    flush_frame(cx);
    flush_frame(cx);
    flush_frame(cx);
    let first = cx
        .debug_bounds("Name(\"tb-indicator-geometry\")-indicator")
        .expect("the measured selected-tab indicator must be painted");
    let expected_width = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    assert!(
        (f32::from(first.size.width) - expected_width).abs() < 1.,
        "the primary indicator must match the selected tab width"
    );
    assert!(
        (f32::from(first.size.height) - 32.).abs() < f32::EPSILON,
        "the primary indicator must match the tab height"
    );

    press(cx, "tab");
    press(cx, "right");
    flush_frame(cx);
    let moving = cx
        .debug_bounds("Name(\"tb-indicator-geometry\")-indicator")
        .expect("the moving indicator must remain painted");
    assert!(
        f32::from(moving.origin.x) < f32::from(first.origin.x + first.size.width) - 1.,
        "the indicator must animate rather than snap to the second tab"
    );
    std::thread::sleep(Duration::from_millis(300));
    flush_frame(cx);
    let second = cx
        .debug_bounds("Name(\"tb-indicator-geometry\")-indicator")
        .expect("the indicator must remain painted after moving");
    assert!(
        f32::from(second.origin.x) >= f32::from(first.origin.x + first.size.width) - 1.,
        "after 250ms the indicator must settle onto the second tab"
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

/// Vertical orientation consumes Down/Up and Right/Left: React Aria's Tabs
/// delegate keeps the horizontal pair active in both orientations.
#[gpui::test]
fn tabs_vertical_axes_down_and_right_move(cx: &mut TestAppContext) {
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
    // TabsKeyboardDelegate exposes Left/Right regardless of orientation, so
    // Right advances just like Down and remains inside the tab list.
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "a vertical tab list must advance on Right as well as Down"
    );
    assert!(
        outside.borrow().is_empty(),
        "a consumed Right key must not reach an enclosing control"
    );
    press(cx, "down");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third", "first"],
        "Down from the last vertical tab must wrap to the first"
    );
}

/// v3's vertical orientation lays the tab list and panel side by side. The
/// list has an 80px minimum tab width, so the panel content must begin to its
/// right rather than below it at the root's leading edge.
#[gpui::test]
fn tabs_vertical_panel_is_to_the_right_of_the_list(cx: &mut TestAppContext) {
    let panel_bounds = Rc::new(RefCell::new(None));
    let for_view = panel_bounds.clone();
    let cx = open_host(cx, move || {
        let panel_bounds = for_view.clone();
        Tabs::new(
            "tb-vertical-layout",
            vec![
                TabItem::new("first", "A").content(gpui::div().size(px(40.)).child(gpui::canvas(
                    move |bounds, _, _| {
                        *panel_bounds.borrow_mut() = Some(bounds);
                        bounds
                    },
                    |_, _, _, _| {},
                ))),
                TabItem::new("second", "B"),
            ],
            "first",
        )
        .orientation(Orientation::Vertical)
        .into_any_element()
    });

    cx.update(|window, _| window.refresh());
    let bounds = panel_bounds
        .borrow()
        .expect("the selected vertical panel must be painted");
    assert!(
        f32::from(bounds.origin.x) > 100.,
        "the vertical panel must sit to the right of the minimum-width tab list"
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

/// Pinned `TabsKeyboardDelegate` wraps internally: its next/previous methods
/// join the collection ends independently of `shouldFocusWrap`.
#[gpui::test]
fn tabs_arrow_keys_wrap_at_the_ends(cx: &mut TestAppContext) {
    let events = events();
    let selected = events.clone();
    let cx = open_host(cx, move || {
        let events = events.clone();
        Tabs::new(
            "tb-wrap",
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
    press(cx, "left");
    press(cx, "right");

    assert_eq!(
        selected.borrow().as_slice(),
        ["third", "first"],
        "Left at the first tab must wrap to the last, and Right must wrap back"
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
