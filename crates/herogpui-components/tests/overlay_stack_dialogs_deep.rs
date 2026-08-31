//! Explicit overlay-stack behavior for Modal, Drawer, and AlertDialog.

mod harness;

use std::{cell::RefCell, rc::Rc};

use gpui::{prelude::*, TestAppContext};
use harness::{click, events, open_host, press};
use herogpui_components::{AlertDialog, Drawer, Modal};

fn still() {
    harness::still();
}

#[gpui::test]
fn nested_modals_escape_only_the_topmost_then_reaches_parent(cx: &mut TestAppContext) {
    still();
    let outer_open = Rc::new(RefCell::new(true));
    let inner_open = Rc::new(RefCell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let outer_open = outer_open.clone();
        let changes = changes.clone();
        move || {
            let outer_open_change = outer_open.clone();
            let inner_open_change = inner_open.clone();
            let outer_changes = changes.clone();
            let inner_changes = changes.clone();
            Modal::new()
                .id("dialog-stack-outer")
                .is_open(*outer_open.borrow())
                .is_keyboard_dismiss_disabled(false)
                .on_open_change(move |open, window, _| {
                    *outer_open_change.borrow_mut() = open;
                    outer_changes.borrow_mut().push(format!("outer:{open}"));
                    window.refresh();
                })
                .child(
                    Modal::new()
                        .id("dialog-stack-inner")
                        .is_open(*inner_open.borrow())
                        .is_keyboard_dismiss_disabled(false)
                        .on_open_change(move |open, window, _| {
                            *inner_open_change.borrow_mut() = open;
                            inner_changes.borrow_mut().push(format!("inner:{open}"));
                            window.refresh();
                        }),
                )
                .into_any_element()
        }
    });

    press(cx, "escape");
    assert_eq!(changes.borrow().as_slice(), ["inner:false"]);
    assert!(*outer_open.borrow());

    press(cx, "escape");
    assert_eq!(changes.borrow().as_slice(), ["inner:false", "outer:false"]);
}

#[gpui::test]
fn nested_modal_and_drawer_outside_press_closes_only_topmost_once(cx: &mut TestAppContext) {
    still();
    let outer_open = Rc::new(RefCell::new(true));
    let drawer_open = Rc::new(RefCell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let outer_open = outer_open.clone();
        let changes = changes.clone();
        move || {
            let outer_open_change = outer_open.clone();
            let drawer_open_change = drawer_open.clone();
            let outer_changes = changes.clone();
            let drawer_changes = changes.clone();
            Modal::new()
                .id("dialog-outside-outer")
                .is_open(*outer_open.borrow())
                .on_open_change(move |open, window, _| {
                    *outer_open_change.borrow_mut() = open;
                    outer_changes.borrow_mut().push(format!("outer:{open}"));
                    window.refresh();
                })
                .child(
                    Drawer::new()
                        .id("dialog-outside-drawer")
                        .is_open(*drawer_open.borrow())
                        .on_open_change(move |open, window, _| {
                            *drawer_open_change.borrow_mut() = open;
                            drawer_changes.borrow_mut().push(format!("drawer:{open}"));
                            window.refresh();
                        }),
                )
                .into_any_element()
        }
    });

    click(cx, 10., 10.);
    assert_eq!(changes.borrow().as_slice(), ["drawer:false"]);
    assert!(*outer_open.borrow());

    click(cx, 10., 10.);
    assert_eq!(changes.borrow().as_slice(), ["drawer:false", "outer:false"]);
}

#[gpui::test]
fn alert_dialog_escape_is_topmost_and_does_not_double_report(cx: &mut TestAppContext) {
    still();
    let modal_open = Rc::new(RefCell::new(true));
    let alert_open = Rc::new(RefCell::new(true));
    let changes = events();

    let cx = open_host(cx, {
        let changes = changes.clone();
        move || {
            let modal_open_change = modal_open.clone();
            let alert_open_change = alert_open.clone();
            let modal_changes = changes.clone();
            let alert_changes = changes.clone();
            Modal::new()
                .id("alert-stack-modal")
                .is_open(*modal_open.borrow())
                .is_keyboard_dismiss_disabled(false)
                .on_open_change(move |open, window, _| {
                    *modal_open_change.borrow_mut() = open;
                    modal_changes.borrow_mut().push(format!("modal:{open}"));
                    window.refresh();
                })
                .child(
                    AlertDialog::new("Confirm")
                        .id("alert-stack-alert")
                        .is_open(*alert_open.borrow())
                        .is_dismissible(true)
                        .is_keyboard_dismiss_disabled(false)
                        .on_open_change(move |open, window, _| {
                            *alert_open_change.borrow_mut() = open;
                            alert_changes.borrow_mut().push(format!("alert:{open}"));
                            window.refresh();
                        }),
                )
                .into_any_element()
        }
    });

    press(cx, "escape");
    assert_eq!(changes.borrow().as_slice(), ["alert:false"]);

    press(cx, "escape");
    assert_eq!(changes.borrow().as_slice(), ["alert:false", "modal:false"]);
}
