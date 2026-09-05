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
    point, prelude::*, px, Font, FontFeatures, FontStyle, FontWeight, KeyDownEvent, Keystroke,
    Modifiers, MouseButton, TestAppContext, VisualTestContext, WindowTextSystem,
};
use herogpui_components::{Button, KeyboardActivation, Orientation, TabItem, Tabs, TabsVariant};

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

fn assert_pointer_selection_timing(
    cx: &mut TestAppContext,
    variant: TabsVariant,
    id: &'static str,
    indicator_id: &'static str,
) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            id,
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .variant(variant)
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });
    flush_frame(cx);
    flush_frame(cx);
    flush_frame(cx);
    flush_frame(cx);
    let first_indicator = cx
        .debug_bounds(indicator_id)
        .expect("the selected-tab indicator must be painted before pointer input");

    let first = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    let second = cx.update(|window, _| text_width(window.text_system(), "Second")) + 32.;
    let second_x = 4. + first + second / 2.;

    cx.simulate_mouse_down(
        point(px(second_x), px(20.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "pinned useSelectableItem selects on pointer down before any release"
    );

    cx.simulate_mouse_up(
        point(px(second_x), px(20.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "a completed pointer click must report exactly once"
    );

    let third = cx.update(|window, _| text_width(window.text_system(), "Third")) + 32.;
    let third_x = 4. + first + second + third / 2.;
    cx.simulate_mouse_down(
        point(px(third_x), px(20.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    flush_frame(cx);
    cx.simulate_mouse_move(
        point(px(500.), px(120.)),
        Some(MouseButton::Left),
        Modifiers::none(),
    );
    flush_frame(cx);
    cx.simulate_mouse_up(
        point(px(500.), px(120.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third"],
        "dragging out must neither revert nor repeat the press-start selection"
    );
    std::thread::sleep(Duration::from_millis(300));
    flush_frame(cx);
    let indicator = cx
        .debug_bounds(indicator_id)
        .expect("the selected-tab indicator must remain painted after dragging out");
    assert!(
        f32::from(indicator.origin.x) >= f32::from(first_indicator.origin.x) + first + second - 1.,
        "dragging out must leave the committed selection on the third tab"
    );
}

/// Pinned react-aria 3.51.0's `useSelectableItem` selects a plain Tab on
/// pointer down. Its pointer-up path does nothing, so dragging out after that
/// press leaves the new selection committed.
#[gpui::test]
fn tabs_select_on_mouse_down_and_keep_a_drag_out(cx: &mut TestAppContext) {
    assert_pointer_selection_timing(
        cx,
        TabsVariant::Primary,
        "tb-primary-pointer-timing",
        "Name(\"tb-primary-pointer-timing\")-indicator",
    );
    assert_pointer_selection_timing(
        cx,
        TabsVariant::Secondary,
        "tb-secondary-pointer-timing",
        "Name(\"tb-secondary-pointer-timing\")-indicator",
    );
}

fn assert_pointer_release_preserves_roving_focus(
    cx: &mut TestAppContext,
    variant: TabsVariant,
    id: &'static str,
) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            id,
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
                TabItem::new("third", "Third"),
            ],
            "first",
        )
        .variant(variant)
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    let first = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    let second = cx.update(|window, _| text_width(window.text_system(), "Second")) + 32.;
    let second_point = point(px(4. + first + second / 2.), px(20.));
    cx.simulate_mouse_down(second_point, MouseButton::Left, Modifiers::none());
    press(cx, "right");
    assert_eq!(selected.borrow().as_slice(), ["second", "third"]);

    cx.simulate_mouse_up(second_point, MouseButton::Left, Modifiers::none());
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second", "third", "first"],
        "pointer release must not restore the tab that received pointer down"
    );
}

/// A plain Tab's pointer-up path is a no-op in pinned `useSelectableItem`.
/// Moving with an arrow while the pointer remains held must therefore survive
/// the eventual release rather than snapping the roving key back.
#[gpui::test]
fn tabs_pointer_release_does_not_revert_roving_focus(cx: &mut TestAppContext) {
    assert_pointer_release_preserves_roving_focus(
        cx,
        TabsVariant::Primary,
        "tb-primary-pointer-held",
    );
    assert_pointer_release_preserves_roving_focus(
        cx,
        TabsVariant::Secondary,
        "tb-secondary-pointer-held",
    );
}

/// Pinned `useSingleSelectListState` enables duplicate selection events for
/// Tabs. Clicking or activating the already-selected tab must still notify a
/// controlled owner, even though the selected key itself does not change.
#[gpui::test]
fn tabs_already_selected_tab_reports_every_activation(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            "tb-repeat-selection",
            vec![
                TabItem::new("first", "First"),
                TabItem::new("second", "Second"),
            ],
            "first",
        )
        .selected_key("first")
        .on_selection_change(move |key, _, _| recorded.borrow_mut().push(key.to_string()))
        .into_any_element()
    });

    let first = cx.update(|window, _| text_width(window.text_system(), "First")) + 32.;
    click(cx, 4. + first / 2., 20.);
    press(cx, "enter");
    press(cx, "space");

    assert_eq!(
        selected.borrow().as_slice(),
        ["first", "first", "first"],
        "pointer, Enter, and Space must each report the already-selected key"
    );
}

/// Pinned `useTabPanel` puts a selected panel with no tabbable child into the
/// tab order. The second Tab therefore leaves the list, and arrow keys from
/// the plain panel must not continue roving or selecting tabs.
#[gpui::test]
fn tabs_second_tab_lands_on_the_panel_and_mutes_arrows(cx: &mut TestAppContext) {
    let recorded = events();
    let selected = recorded.clone();
    let after_presses = events();
    let after = after_presses.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        let after_presses = after_presses.clone();
        gpui::div()
            .child(
                Tabs::new(
                    "tb-panel-stop",
                    vec![
                        TabItem::new("first", "First").content(gpui::div().child("First panel")),
                        TabItem::new("second", "Second").content(gpui::div().child("Second panel")),
                        TabItem::new("third", "Third").content(gpui::div().child("Third panel")),
                    ],
                    "first",
                )
                .on_selection_change(move |key, _, _| {
                    recorded.borrow_mut().push(key.to_string());
                }),
            )
            .child(
                Button::new("tb-panel-after")
                    .label("After tabs")
                    .on_press(move |_, _, _| after_presses.borrow_mut().push("after".into())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert!(
        after.borrow().is_empty(),
        "the second Tab must stop on the plain panel before the following button"
    );
    press(cx, "right");
    assert!(
        selected.borrow().is_empty(),
        "the selected panel must own focus after the second Tab"
    );

    press(cx, "shift-tab");
    press(cx, "right");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "reverse Tab from the panel must return to the roving list stop"
    );

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "Tab from the plain panel must reach the following button"
    );
    press(cx, "shift-tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "reverse Tab must leave the following button before the panel probe"
    );
    press(cx, "left");
    assert_eq!(
        selected.borrow().as_slice(),
        ["second"],
        "reverse Tab from the following control must stop on the plain panel"
    );
}

/// A selected tab without content has no TabPanel stop. Moving from a tab with
/// content to one without it must remove that stop, and moving back must add it
/// again without leaving stale focus state behind.
#[gpui::test]
fn tabs_uncontrolled_content_switch_updates_panel_traversal(cx: &mut TestAppContext) {
    let after_presses = events();
    let after = after_presses.clone();
    let cx = open_host(cx, move || {
        let after_presses = after_presses.clone();
        gpui::div()
            .child(Tabs::new(
                "tb-content-switch",
                vec![
                    TabItem::new("first", "First").content(gpui::div().child("First panel")),
                    TabItem::new("second", "Second"),
                ],
                "first",
            ))
            .child(
                Button::new("tb-content-switch-after")
                    .label("After tabs")
                    .on_press(move |_, _, _| after_presses.borrow_mut().push("after".into())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "right");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "a no-content selection must leave the list directly for the next stop"
    );

    press(cx, "shift-tab");
    press(cx, "left");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "restoring content must put the plain panel back before the next stop"
    );
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(after.borrow().as_slice(), ["after", "after"]);
}

/// A panel with a tabbable child is not itself a stop in pinned `useTabPanel`.
/// The second Tab must reach that child directly rather than adding an extra
/// wrapper stop before it.
#[gpui::test]
fn tabs_second_tab_enters_the_panels_first_tabbable_child(cx: &mut TestAppContext) {
    let recorded = events();
    let pressed = recorded.clone();
    let cx = open_host(cx, move || {
        let button_events = recorded.clone();
        let selection_events = recorded.clone();
        gpui::div()
            .child(
                Tabs::new(
                    "tb-panel-child",
                    vec![
                        TabItem::new("first", "First").content(
                            gpui::div().child(
                                Button::new("tb-panel-button")
                                    .label("Panel action")
                                    .on_press(move |_, _, _| {
                                        button_events.borrow_mut().push("pressed".into());
                                    }),
                            ),
                        ),
                        TabItem::new("second", "Second").content(gpui::div().child("Second panel")),
                    ],
                    "first",
                )
                .on_selection_change(move |key, _, _| {
                    selection_events.borrow_mut().push(key.to_string());
                }),
            )
            .child(Button::new("tb-panel-child-after").label("After tabs"))
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["pressed"],
        "the first tabbable panel child must be the second stop"
    );

    press(cx, "tab");
    press(cx, "shift-tab");
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["pressed", "pressed"],
        "reverse Tab from the following control must reach the panel child"
    );

    press(cx, "shift-tab");
    press(cx, "right");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["pressed", "pressed", "second"],
        "reverse Tab from the first panel child must return to the roving list stop"
    );
}

/// A disabled tab list contributes no stop, but it does not disable the
/// selected panel's own controls. Pinned `useTabPanel` therefore yields
/// directly to the first tabbable panel child rather than its wrapper.
#[gpui::test]
fn tabs_all_disabled_enters_the_panels_first_tabbable_child(cx: &mut TestAppContext) {
    let recorded = events();
    let pressed = recorded.clone();
    let cx = open_host(cx, move || {
        let recorded = recorded.clone();
        Tabs::new(
            "tb-disabled-panel-child",
            vec![TabItem::new("first", "First").content(
                gpui::div().child(
                    Button::new("tb-disabled-panel-button")
                        .label("Panel action")
                        .on_press(move |_, _, _| recorded.borrow_mut().push("pressed".into())),
                ),
            )],
            "first",
        )
        .is_disabled(true)
        .into_any_element()
    });

    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["pressed"],
        "the panel child must be the first stop when every tab is disabled"
    );
}

/// Windows sends repeated Tab key-downs while the key is held and only one
/// key-up at release. The synthetic wrapper must not add a repeat stop before
/// the first real panel child.
#[gpui::test]
fn tabs_all_disabled_held_tab_advances_past_the_panel_child(cx: &mut TestAppContext) {
    let recorded = events();
    let pressed = recorded.clone();
    let cx = open_host(cx, move || {
        let child_events = recorded.clone();
        let after_events = recorded.clone();
        gpui::div()
            .child(
                Tabs::new(
                    "tb-disabled-held-tab",
                    vec![TabItem::new("first", "First").content(
                        gpui::div().child(
                            Button::new("tb-disabled-held-child")
                                .label("Panel action")
                                .on_press(move |_, _, _| {
                                    child_events.borrow_mut().push("child".into());
                                }),
                        ),
                    )],
                    "first",
                )
                .is_disabled(true),
            )
            .child(
                Button::new("tb-disabled-held-after")
                    .label("After tabs")
                    .on_press(move |_, _, _| after_events.borrow_mut().push("after".into())),
            )
            .into_any_element()
    });

    cx.simulate_keystrokes("tab");
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse("tab").expect("tab is a valid keystroke"),
        is_held: true,
        prefer_character_input: false,
    });
    press(cx, "enter");
    assert_eq!(
        pressed.borrow().as_slice(),
        ["after"],
        "the first repeat must advance from the panel child to the following stop"
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

/// When a controlled owner selects a tab with no panel, the focused control in
/// the old panel disappears. The next Tab must re-enter on the selected tab,
/// and the following Tab must leave directly for the next external stop.
#[gpui::test]
fn tabs_controlled_no_content_update_releases_removed_panel_focus(cx: &mut TestAppContext) {
    let selected_key = Rc::new(RefCell::new(String::from("first")));
    let selected_for_view = selected_key.clone();
    let proposals = events();
    let proposed = proposals.clone();
    let panel_presses = events();
    let panel = panel_presses.clone();
    let after_presses = events();
    let after = after_presses.clone();
    let cx = open_host(cx, move || {
        let proposals = proposals.clone();
        let panel_presses = panel_presses.clone();
        let after_presses = after_presses.clone();
        gpui::div()
            .child(Button::new("tb-controlled-no-content-before").label("Before tabs"))
            .child(
                Tabs::new(
                    "tb-controlled-no-content",
                    vec![
                        TabItem::new("first", "First").content(
                            gpui::div().child(
                                Button::new("tb-controlled-panel-button")
                                    .label("Panel action")
                                    .on_press(move |_, _, _| {
                                        panel_presses.borrow_mut().push("panel".into());
                                    }),
                            ),
                        ),
                        TabItem::new("second", "Second"),
                    ],
                    "first",
                )
                .selected_key(selected_for_view.borrow().clone())
                .on_selection_change(move |key, _, _| {
                    proposals.borrow_mut().push(key.to_string());
                }),
            )
            .child(
                Button::new("tb-controlled-no-content-after")
                    .label("After tabs")
                    .on_press(move |_, _, _| after_presses.borrow_mut().push("after".into())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(panel.borrow().as_slice(), ["panel"]);

    *selected_key.borrow_mut() = String::from("second");
    cx.update(|window, _| window.refresh());
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "two Tabs after the focused panel unmounts must reach the following stop"
    );
    assert!(
        proposed.borrow().is_empty(),
        "focus recovery must not reactivate the already-selected no-content tab"
    );
}

/// A controlled selection change also removes the focused child when both the
/// old and new tabs have panels. Recovery must re-enter on the selected tab,
/// pass through the new plain panel, and then leave for the following stop.
#[gpui::test]
fn tabs_controlled_content_update_releases_removed_panel_focus(cx: &mut TestAppContext) {
    let selected_key = Rc::new(RefCell::new(String::from("first")));
    let selected_for_view = selected_key.clone();
    let proposals = events();
    let proposed = proposals.clone();
    let panel_presses = events();
    let panel = panel_presses.clone();
    let after_presses = events();
    let after = after_presses.clone();
    let cx = open_host(cx, move || {
        let proposals = proposals.clone();
        let panel_presses = panel_presses.clone();
        let after_presses = after_presses.clone();
        gpui::div()
            .child(Button::new("tb-controlled-content-before").label("Before tabs"))
            .child(
                Tabs::new(
                    "tb-controlled-content",
                    vec![
                        TabItem::new("first", "First").content(
                            gpui::div().child(
                                Button::new("tb-controlled-content-panel-button")
                                    .label("Panel action")
                                    .on_press(move |_, _, _| {
                                        panel_presses.borrow_mut().push("panel".into());
                                    }),
                            ),
                        ),
                        TabItem::new("second", "Second").content(gpui::div().child("Second panel")),
                    ],
                    "first",
                )
                .selected_key(selected_for_view.borrow().clone())
                .on_selection_change(move |key, _, _| {
                    proposals.borrow_mut().push(key.to_string());
                }),
            )
            .child(
                Button::new("tb-controlled-content-after")
                    .label("After tabs")
                    .on_press(move |_, _, _| after_presses.borrow_mut().push("after".into())),
            )
            .into_any_element()
    });

    press(cx, "tab");
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(panel.borrow().as_slice(), ["panel"]);

    *selected_key.borrow_mut() = String::from("second");
    cx.update(|window, _| window.refresh());
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "tab");
    press(cx, "enter");
    assert_eq!(
        after.borrow().as_slice(),
        ["after"],
        "three Tabs after the focused panel changes must reach the following stop"
    );
    assert!(
        proposed.borrow().is_empty(),
        "focus recovery must not activate the externally selected content tab"
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
/// with arrows without selecting. Enter and Space activate the focused tab
/// afterward, so a controlled owner receives one proposal per completed press.
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
    press(cx, "space");
    assert_eq!(
        recorded.borrow().as_slice(),
        ["second", "second"],
        "Space must activate the manually focused tab"
    );
}
