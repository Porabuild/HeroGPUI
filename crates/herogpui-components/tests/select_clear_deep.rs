//! HeroUI v3.2.5 Select.ClearButton: real pointer and keyboard dispatch.
mod harness;
use gpui::{prelude::*, px, SharedString, TestAppContext};
use harness::{click, events, open_host, press, still};
use herogpui_components::{FieldVariant, PickerItem, Select, SelectClearButton, SelectionMode};
use std::{cell::RefCell, rc::Rc};

#[gpui::test]
fn clear_click_preserves_focus_and_layout_and_empty_target_opens(cx: &mut TestAppContext) {
    for variant in [FieldVariant::Primary, FieldVariant::Secondary] {
        for multiple in [false, true] {
            for controlled in [false, true] {
                still();
                let recorded = events();
                let log = recorded.clone();
                let values = Rc::new(RefCell::new(Vec::<Vec<SharedString>>::new()));
                let observed = values.clone();
                let cx = open_host(cx, move || {
                    let single = log.clone();
                    let all = log.clone();
                    let clear = log.clone();
                    let part = log.clone();
                    let opened = log.clone();
                    let values = values.clone();
                    let mut select = Select::new(
                        "clear",
                        vec![PickerItem::new("a", "Alpha"), PickerItem::new("b", "Beta")],
                    )
                    .variant(variant)
                    .is_required(true)
                    .full_width(true)
                    .selection_mode(if multiple {
                        SelectionMode::Multiple
                    } else {
                        SelectionMode::Single
                    })
                    .on_change(move |v, _, _| single.borrow_mut().push(format!("single:{v:?}")))
                    .on_selection_change_all(move |v, _, _| {
                        all.borrow_mut().push(format!("all:{v:?}"));
                    })
                    .on_clear(move |_, _| clear.borrow_mut().push("clear".into()))
                    .on_open_change(move |v, _, _| opened.borrow_mut().push(format!("open:{v}")))
                    .clear_button(
                        SelectClearButton::new()
                            .on_click(move |_, _, _| part.borrow_mut().push("part".into())),
                    )
                    .value_content(move |v| {
                        values.borrow_mut().push(v.selected_keys.unwrap().to_vec());
                        v.default_children
                    });
                    if multiple {
                        select = if controlled {
                            select.selected_keys(["a".into(), "b".into()])
                        } else {
                            select.default_selected_keys(["a".into(), "b".into()])
                        };
                    } else {
                        select = if controlled {
                            select.value(Some("a".into()))
                        } else {
                            select.default_value(Some("a".into()))
                        };
                    }
                    gpui::div().w(px(256.)).child(select).into_any_element()
                });
                let before = cx.debug_bounds("select-clear-0").unwrap();
                assert_eq!(before.size.width, px(24.));
                assert_eq!(before.size.height, px(24.));
                // One pixel inside the expanded left edge, outside the 20px visual.
                click(
                    cx,
                    f32::from(before.left()) + 1.,
                    f32::from(before.center().y),
                );
                let expected = if multiple { "all:[]" } else { "single:None" };
                assert_eq!(recorded.borrow().as_slice(), [expected, "clear", "part"]);
                assert_eq!(cx.debug_bounds("select-clear-0").unwrap(), before);
                assert_eq!(observed.borrow().last().unwrap().is_empty(), !controlled);
                // Keyboard must still address the trigger. A controlled owner
                // that rejects the clear request reports again without changing.
                press(cx, "backspace");
                assert!(
                    cx.update(|_, cx| herogpui_components::util::focus_visible(cx)),
                    "clearing by keyboard must enable the focus ring after a pointer press"
                );
                if controlled {
                    assert_eq!(
                        recorded.borrow().as_slice(),
                        [expected, "clear", "part", expected, "clear"]
                    );
                } else {
                    assert_eq!(recorded.borrow().len(), 3);
                    click(
                        cx,
                        f32::from(before.center().x),
                        f32::from(before.center().y),
                    );
                    assert_eq!(recorded.borrow().last().unwrap(), "open:true");
                }
            }
        }
    }
}

#[gpui::test]
fn clear_shortcut_requires_composition_nonempty_enabled_and_closed(cx: &mut TestAppContext) {
    for composed in [false, true] {
        for disabled in [false, true] {
            for open in [false, true] {
                for key in ["backspace", "delete"] {
                    still();
                    let recorded = events();
                    let log = recorded.clone();
                    let cx = open_host(cx, move || {
                        let log = log.clone();
                        let mut select =
                            Select::new("shortcut", vec![PickerItem::new("a", "Alpha")])
                                .default_value(Some("a".into()))
                                .is_disabled(disabled)
                                .is_open(open)
                                .on_clear(move |_, _| log.borrow_mut().push("clear".into()));
                        if composed {
                            select = select.clear_button(SelectClearButton::new());
                        }
                        select.into_any_element()
                    });
                    press(cx, "tab");
                    press(cx, key);
                    assert_eq!(
                        recorded.borrow().len(),
                        usize::from(composed && !disabled && !open),
                        "{composed} {disabled} {open} {key}"
                    );
                }
            }
        }
    }
}

#[gpui::test]
fn clear_pointer_cancellation_secondary_buttons_and_disabled_are_inert(cx: &mut TestAppContext) {
    use gpui::{point, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent};
    for disabled in [false, true] {
        still();
        let recorded = events();
        let log = recorded.clone();
        let cx = open_host(cx, move || {
            let clear = log.clone();
            let opened = log.clone();
            Select::new("pointer", vec![PickerItem::new("a", "Alpha")])
                .default_value(Some("a".into()))
                .is_disabled(disabled)
                .clear_button(
                    SelectClearButton::new()
                        .children([gpui::div().size(px(12.)).into_any_element()]),
                )
                .on_clear(move |_, _| clear.borrow_mut().push("clear".into()))
                .on_open_change(move |v, _, _| opened.borrow_mut().push(format!("open:{v}")))
                .into_any_element()
        });
        let center = cx.debug_bounds("select-clear-0").unwrap().center();
        for button in [MouseButton::Right, MouseButton::Middle, MouseButton::Left] {
            cx.simulate_event(MouseDownEvent {
                button,
                position: center,
                modifiers: Modifiers::none(),
                click_count: 1,
                first_mouse: false,
            });
            cx.update(|w, _| w.refresh());
            assert!(
                recorded.borrow().is_empty(),
                "pointer down must not clear or open"
            );
            let release = if button == MouseButton::Left {
                point(px(500.), px(300.))
            } else {
                center
            };
            cx.simulate_event(MouseMoveEvent {
                position: release,
                pressed_button: Some(button),
                modifiers: Modifiers::none(),
            });
            cx.update(|w, _| w.refresh());
            cx.simulate_event(MouseUpEvent {
                button,
                position: release,
                modifiers: Modifiers::none(),
                click_count: 1,
            });
            cx.update(|w, _| w.refresh());
            assert!(
                recorded.borrow().is_empty(),
                "secondary buttons and canceled presses must be inert"
            );
        }
        click(cx, f32::from(center.x), f32::from(center.y));
        assert_eq!(recorded.borrow().len(), usize::from(!disabled));
    }
}

#[gpui::test]
fn controlled_clear_updates_when_owner_accepts_and_parts_stay_independent(cx: &mut TestAppContext) {
    still();
    let selected = Rc::new(RefCell::new(Some(SharedString::from("a"))));
    let owner = selected.clone();
    let recorded = events();
    let log = recorded.clone();
    let cx = open_host(cx, move || {
        let state = selected.clone();
        let first = log.clone();
        let second = log.clone();
        Select::new("two-clear-parts", vec![PickerItem::new("a", "Alpha")])
            .value(selected.borrow().clone())
            .on_change(move |v, _, cx| {
                *state.borrow_mut() = v.clone();
                cx.refresh_windows();
            })
            .clear_button(
                SelectClearButton::new()
                    .on_click(move |_, _, _| first.borrow_mut().push("first".into())),
            )
            .clear_button(
                SelectClearButton::new()
                    .on_click(move |_, _, _| second.borrow_mut().push("second".into())),
            )
            .into_any_element()
    });
    let first = cx.debug_bounds("select-clear-0").unwrap();
    let second = cx.debug_bounds("select-clear-1").unwrap();
    assert!(first.right() <= second.left());
    click(
        cx,
        f32::from(second.center().x),
        f32::from(second.center().y),
    );
    assert!(owner.borrow().is_none());
    assert_eq!(recorded.borrow().as_slice(), ["second"]);
    press(cx, "delete");
    assert_eq!(recorded.borrow().as_slice(), ["second"]);
}
