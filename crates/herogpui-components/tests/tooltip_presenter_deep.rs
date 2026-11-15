//! Tooltip presenter boundary tests covering custom external presenters,
//! `SeparateWindow` GPUI popups, hover generation staleness, Escape & re-entry,
//! presenter-swap cleanup, dynamic body & wrapped-text sizing, arrow overhang,
//! trigger-move repositioning, multi-window ID isolation, owner unmount cleanup,
//! capture-exclusion metadata, monitor edges, and DPI.

use crate::harness;

use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui::{
    point, prelude::*, px, size, Bounds, ElementId, Modifiers, Size, TestAppContext,
    VisualTestContext, Window,
};
use harness::{open_host, press, settle, still, wait_ms, Host};
use herogpui_components::{
    position_tooltip_popup, Button, Tooltip, TooltipPlacement, TooltipPresentationRequest,
    TooltipPresenter, TooltipPresenterMode, TooltipTrigger,
};
use herogpui_theme::{set_reduce_motion, ThemeProvider};

#[derive(Clone, Debug, PartialEq)]
enum PresenterEvent {
    Present {
        key: String,
        content: String,
        exclude_from_capture: bool,
        scale_factor: f32,
        placement: TooltipPlacement,
        trigger_bounds: Bounds<gpui::Pixels>,
        trigger_screen_bounds: Bounds<gpui::Pixels>,
        tip_size: Option<Size<gpui::Pixels>>,
    },
    Dismiss {
        key: String,
    },
}

struct RecordingPresenter {
    events: Arc<Mutex<Vec<PresenterEvent>>>,
}

impl TooltipPresenter for RecordingPresenter {
    fn present(
        &self,
        request: &TooltipPresentationRequest,
        _window: &mut Window,
        _cx: &mut gpui::App,
    ) {
        self.events.lock().unwrap().push(PresenterEvent::Present {
            key: format!("{:?}", request.key),
            content: request.content.to_string(),
            exclude_from_capture: request.exclude_from_capture,
            scale_factor: request.scale_factor,
            placement: request.placement,
            trigger_bounds: request.trigger_bounds,
            trigger_screen_bounds: request.trigger_screen_bounds,
            tip_size: request.tip_size,
        });
    }

    fn dismiss(&self, key: &ElementId, _window: Option<&mut Window>, _cx: &mut gpui::App) {
        self.events.lock().unwrap().push(PresenterEvent::Dismiss {
            key: format!("{key:?}"),
        });
    }
}

#[gpui::test]
fn custom_presenter_keeps_stale_hover_escape_and_reentry_lifecycle(cx: &mut TestAppContext) {
    still();
    let events = Arc::new(Mutex::new(Vec::new()));
    let presenter: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events.clone(),
    });
    let presenter_render = presenter.clone();

    let vcx = open_host(cx, move || {
        let presenter = presenter_render.clone();
        gpui::div()
            .p(px(16.))
            .flex()
            .gap(px(24.))
            .child(
                Tooltip::new("Copy link")
                    .id("tip-a")
                    .delay(300)
                    .close_delay(100)
                    .placement(TooltipPlacement::Bottom)
                    .presenter(presenter.clone())
                    .exclude_from_capture(true)
                    .child(
                        gpui::div()
                            .debug_selector(|| "btn-a".into())
                            .child(Button::new("btn-a").label("Copy")),
                    ),
            )
            .child(
                Tooltip::new("Delete item")
                    .id("tip-b")
                    .delay(300)
                    .close_delay(100)
                    .presenter(presenter)
                    .child(
                        gpui::div()
                            .debug_selector(|| "btn-b".into())
                            .child(Button::new("btn-b").label("Delete")),
                    ),
            )
            .into_any_element()
    });

    // Hover button A briefly then move away before the 300ms delay: stale timer must not open.
    let btn_a = vcx.debug_bounds("btn-a").expect("btn-a bounds");
    vcx.simulate_mouse_move(btn_a.center(), None, Modifiers::none());
    wait_ms(vcx, 120);
    vcx.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
    wait_ms(vcx, 300);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .all(|e| matches!(e, PresenterEvent::Dismiss { .. })),
        "stale hover timer must never present after pointer left: {:?}",
        events.lock().unwrap()
    );

    events.lock().unwrap().clear();

    // Hover button A past 300ms -> presenter receives Present with trigger bounds & capture flag.
    vcx.simulate_mouse_move(btn_a.center(), None, Modifiers::none());
    wait_ms(vcx, 320);
    let present = events
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            PresenterEvent::Present { .. } => Some(e.clone()),
            PresenterEvent::Dismiss { .. } => None,
        })
        .expect("presenter must receive Present once delay elapses");
    match present {
        PresenterEvent::Present {
            content,
            exclude_from_capture,
            placement,
            trigger_bounds,
            ..
        } => {
            assert_eq!(content, "Copy link");
            assert!(exclude_from_capture);
            assert_eq!(placement, TooltipPlacement::Bottom);
            assert!(trigger_bounds.size.width > px(0.) && trigger_bounds.size.height > px(0.));
        }
        PresenterEvent::Dismiss { .. } => unreachable!(),
    }

    // Hover moves immediately to sibling button B during warmup -> A dismisses, B presents immediately.
    events.lock().unwrap().clear();
    let btn_b = vcx.debug_bounds("btn-b").expect("btn-b bounds");
    vcx.simulate_mouse_move(btn_b.center(), None, Modifiers::none());
    wait_ms(vcx, 20);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Dismiss { key } if key.contains("tip-a"))),
        "switching to sibling must dismiss tip-a: {:?}",
        events.lock().unwrap()
    );
    assert!(
        events.lock().unwrap().iter().any(|e| matches!(e, PresenterEvent::Present { key, exclude_from_capture, .. } if key.contains("tip-b") && !exclude_from_capture)),
        "warmed-up sibling tip-b must present without waiting full delay: {:?}",
        events.lock().unwrap()
    );

    // Move away and let cooldown expire cleanly.
    vcx.simulate_mouse_move(point(px(600.), px(600.)), None, Modifiers::none());
    wait_ms(vcx, 700);
    events.lock().unwrap().clear();

    // Focus entry via Tab -> opens tip-a; Escape dismisses it; Tab away & Shift+Tab back re-opens.
    press(vcx, "tab");
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { key, .. } if key.contains("tip-a"))),
        "keyboard focus entry must present tip-a: {:?}",
        events.lock().unwrap()
    );
    events.lock().unwrap().clear();

    press(vcx, "escape");
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Dismiss { key } if key.contains("tip-a"))),
        "Escape must dismiss external tooltip: {:?}",
        events.lock().unwrap()
    );
    events.lock().unwrap().clear();

    press(vcx, "tab");
    settle(vcx);
    events.lock().unwrap().clear();
    press(vcx, "shift-tab");
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { key, .. } if key.contains("tip-a"))),
        "focus re-entry after leaving must present tip-a again: {:?}",
        events.lock().unwrap()
    );
}

#[gpui::test]
fn owner_unmount_and_remount_manage_presenter_ownership_cleanly(cx: &mut TestAppContext) {
    still();
    let mounted = Rc::new(Cell::new(true));
    let events = Arc::new(Mutex::new(Vec::new()));
    let presenter: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events.clone(),
    });

    let mounted_render = mounted.clone();
    let presenter_render = presenter.clone();
    let vcx = open_host(cx, move || {
        if mounted_render.get() {
            Tooltip::new("Mounted tip")
                .id("tip-own")
                .trigger(TooltipTrigger::Focus)
                .presenter(presenter_render.clone())
                .child(Button::new("btn-own").label("Owned"))
                .into_any_element()
        } else {
            gpui::div().child("unmounted").into_any_element()
        }
    });

    press(vcx, "tab");
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { key, .. } if key.contains("tip-own"))),
        "focus should present mounted tooltip"
    );
    events.lock().unwrap().clear();

    // Unmount while open: GPUI releases keyed lease and presenter must dismiss automatically.
    mounted.set(false);
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Dismiss { key } if key.contains("tip-own"))),
        "unmounting an open tooltip must trigger Dismiss on the external presenter: {:?}",
        events.lock().unwrap()
    );
    events.lock().unwrap().clear();

    // Remount and focus again: stale cleanup must not swallow the newly mounted tooltip.
    mounted.set(true);
    settle(vcx);
    press(vcx, "tab");
    settle(vcx);
    assert!(
        events
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { key, .. } if key.contains("tip-own"))),
        "remounted tooltip must re-present cleanly on focus: {:?}",
        events.lock().unwrap()
    );
}

#[gpui::test]
fn swapping_presenter_while_open_dismisses_prior_presenter_before_new_one(cx: &mut TestAppContext) {
    still();
    let use_second = Rc::new(Cell::new(false));
    let events_a = Arc::new(Mutex::new(Vec::new()));
    let events_b = Arc::new(Mutex::new(Vec::new()));
    let presenter_a: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events_a.clone(),
    });
    let presenter_b: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events_b.clone(),
    });

    let use_second_render = use_second.clone();
    let a_render = presenter_a.clone();
    let b_render = presenter_b.clone();
    let vcx = open_host(cx, move || {
        let presenter = if use_second_render.get() {
            b_render.clone()
        } else {
            a_render.clone()
        };
        Tooltip::new("Swappable tip")
            .id("tip-swap")
            .trigger(TooltipTrigger::Focus)
            .presenter(presenter)
            .child(Button::new("btn-swap").label("Swap"))
            .into_any_element()
    });

    press(vcx, "tab");
    settle(vcx);
    assert!(
        events_a
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { .. })),
        "presenter A must present initially"
    );
    assert!(events_b.lock().unwrap().is_empty());
    events_a.lock().unwrap().clear();

    // Swap presenter A -> B while tooltip stays open: A must dismiss, B must present.
    use_second.set(true);
    settle(vcx);
    assert!(
        events_a
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Dismiss { key } if key.contains("tip-swap"))),
        "replacing presenter A with B while open must dismiss A: {:?}",
        events_a.lock().unwrap()
    );
    assert!(
        events_b
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Present { key, .. } if key.contains("tip-swap"))),
        "presenter B must present after replacing A: {:?}",
        events_b.lock().unwrap()
    );

    events_a.lock().unwrap().clear();
    events_b.lock().unwrap().clear();

    // Subsequent Escape dismissal must notify only presenter B, never A.
    press(vcx, "escape");
    settle(vcx);
    assert!(
        events_a.lock().unwrap().is_empty(),
        "stale presenter A must receive no further events: {:?}",
        events_a.lock().unwrap()
    );
    assert!(
        events_b
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, PresenterEvent::Dismiss { key } if key.contains("tip-swap"))),
        "current presenter B must receive Dismiss: {:?}",
        events_b.lock().unwrap()
    );
}

#[gpui::test]
fn separate_window_presenter_opens_non_focusing_popup_beyond_compact_owner_and_closes(
    cx: &mut TestAppContext,
) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .p(px(4.))
            .child(
                Tooltip::new("Full tooltip text that exceeds a tiny compact toolbar window width")
                    .id("tip-sep")
                    .presenter_mode(TooltipPresenterMode::SeparateWindow)
                    .exclude_from_capture(true)
                    .child(
                        gpui::div()
                            .debug_selector(|| "btn-sep".into())
                            .child(Button::new("btn-sep").label("S")),
                    ),
            )
            .into_any_element()
    });
    // Compact owner window narrower than the tooltip text.
    let owner_handle = vcx.update(|window, _| {
        window.activate_window();
        window.window_handle()
    });
    vcx.simulate_resize(size(px(80.), px(44.)));
    settle(vcx);
    assert_eq!(
        vcx.update(|_, cx| cx.windows().len()),
        1,
        "start with only the owner window"
    );

    let btn = vcx.debug_bounds("btn-sep").expect("btn-sep bounds");
    vcx.simulate_mouse_move(btn.center(), None, Modifiers::none());
    wait_ms(vcx, 1600);

    let (win_count, active_win) = vcx.update(|_, cx| (cx.windows().len(), cx.active_window()));
    assert_eq!(
        win_count, 2,
        "SeparateWindow presenter should open a secondary GPUI popup window"
    );
    assert_eq!(
        active_win,
        Some(owner_handle),
        "SeparateWindow popup (focus: false) must not steal active focus from owner window"
    );

    // Focus the button and press Escape -> popup window must close cleanly back to 1 window.
    press(vcx, "tab");
    settle(vcx);
    press(vcx, "escape");
    wait_ms(vcx, 100);
    assert_eq!(
        vcx.update(|_, cx| cx.windows().len()),
        1,
        "Escape dismissal must close the SeparateWindow tooltip popup"
    );
}

#[gpui::test]
fn separate_window_measures_large_body_wrapped_text_and_arrow_overhang(cx: &mut TestAppContext) {
    still();
    let events = Arc::new(Mutex::new(Vec::new()));
    let custom_presenter: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events.clone(),
    });
    let show_body = Rc::new(Cell::new(true));

    // Verify `request.tip_size` is populated accurately for both large custom `.body()`
    // surfaces and wrapped unbroken text tokens.
    let show_body_render = show_body.clone();
    let vcx = open_host(cx, move || {
        if show_body_render.get() {
            Tooltip::new("Body summary")
                .id("tip-measure")
                .trigger(TooltipTrigger::Focus)
                .presenter(custom_presenter.clone())
                .body(|_, _| {
                    gpui::div()
                        .w(px(240.))
                        .h(px(84.))
                        .child("Large custom body")
                        .into_any_element()
                })
                .child(Button::new("btn-m1").label("Body"))
                .into_any_element()
        } else {
            Tooltip::new("supercalifragilisticexpialidocious_unbroken_identifier_that_must_wrap_at_320px_cap_for_separate_window")
                .id("tip-wrap")
                .trigger(TooltipTrigger::Focus)
                .presenter(custom_presenter.clone())
                .child(Button::new("btn-m2").label("Wrap"))
                .into_any_element()
        }
    });

    press(vcx, "tab");
    settle(vcx);
    let body_size = events
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            PresenterEvent::Present { tip_size, .. } => *tip_size,
            PresenterEvent::Dismiss { .. } => None,
        })
        .expect("custom body request must provide measured tip_size");
    assert!(
        body_size.width >= px(240.) && body_size.height >= px(84.),
        "large body tip_size must not shrink to 180x32: {body_size:?}"
    );

    events.lock().unwrap().clear();
    show_body.set(false);
    settle(vcx);
    press(vcx, "tab");
    settle(vcx);
    let wrap_size = events
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            PresenterEvent::Present { tip_size, .. } => *tip_size,
            PresenterEvent::Dismiss { .. } => None,
        })
        .expect("wrapped text request must provide measured tip_size");
    assert!(
        wrap_size.width <= px(320.) && wrap_size.height > px(32.),
        "long token must stay within 320px cap and grow taller vertically: {wrap_size:?}"
    );

    // SeparateWindow with a large body + show_arrow must allocate full body size plus the 12px arrow overhang.
    let vcx_sep = open_host(cx, || {
        gpui::div()
            .p(px(40.))
            .child(
                Tooltip::new("Arrow body tip")
                    .id("tip-sep-arrow")
                    .trigger(TooltipTrigger::Focus)
                    .presenter_mode(TooltipPresenterMode::SeparateWindow)
                    .show_arrow(true)
                    .body(|_, _| {
                        gpui::div()
                            .w(px(240.))
                            .h(px(84.))
                            .child("Popup body")
                            .into_any_element()
                    })
                    .child(Button::new("btn-sep-arrow").label("Arrow")),
            )
            .into_any_element()
    });
    let owner_id = vcx_sep.update(|window, _| window.window_handle().window_id());
    press(vcx_sep, "tab");
    settle(vcx_sep);

    let popup_bounds = vcx_sep.update(|_, cx| {
        cx.windows()
            .into_iter()
            .find(|h| h.window_id() > owner_id)
            .and_then(|h| h.update(cx, |_, window, _| window.bounds()).ok())
            .expect("SeparateWindow popup bounds")
    });
    assert!(
        popup_bounds.size.width >= px(240.) && popup_bounds.size.height >= px(84. + 12.),
        "SeparateWindow popup must encompass full body plus 12px arrow gutter: {popup_bounds:?}"
    );
}

#[gpui::test]
fn separate_window_repositions_on_trigger_move_and_skips_idle_repaints(cx: &mut TestAppContext) {
    still();
    let left_pad = Rc::new(Cell::new(px(16.)));
    let pad_render = left_pad.clone();
    let vcx = open_host(cx, move || {
        gpui::div()
            .pt(px(80.))
            .pl(pad_render.get())
            .child(
                Tooltip::new("Moving trigger tooltip")
                    .id("tip-move")
                    .trigger(TooltipTrigger::Focus)
                    .presenter_mode(TooltipPresenterMode::SeparateWindow)
                    .child(Button::new("btn-move").label("Move")),
            )
            .into_any_element()
    });
    let owner_handle = vcx.update(|window, _| {
        window.activate_window();
        window.window_handle()
    });
    settle(vcx);

    press(vcx, "tab");
    settle(vcx);
    let (popup_1, bounds_1) = vcx.update(|_, cx| {
        let popup = cx
            .windows()
            .into_iter()
            .find(|h| *h != owner_handle)
            .expect("initial popup window");
        let b = popup.update(cx, |_, window, _| window.bounds()).unwrap();
        (popup, b)
    });

    // Repeated idle repaints must keep the existing popup window unchanged (no reopen/flicker).
    settle(vcx);
    settle(vcx);
    let popup_idle = vcx.update(|_, cx| cx.windows().into_iter().find(|h| *h != owner_handle));
    assert_eq!(popup_idle, Some(popup_1));

    // Move the trigger rightward while the tooltip stays open: popup origin must follow
    // without stealing focus from the owner window.
    left_pad.set(px(180.));
    settle(vcx);
    let (bounds_2, active_win) = vcx.update(|_, cx| {
        let popup = cx
            .windows()
            .into_iter()
            .find(|h| *h != owner_handle)
            .expect("repositioned popup window");
        let b = popup.update(cx, |_, window, _| window.bounds()).unwrap();
        (b, cx.active_window())
    });
    assert!(
        bounds_2.origin.x > bounds_1.origin.x + px(100.),
        "popup window must follow moved trigger: before={bounds_1:?}, after={bounds_2:?}"
    );
    assert_eq!(active_win, Some(owner_handle));
}

#[gpui::test]
fn separate_window_isolates_identical_ids_across_owner_windows(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ThemeProvider::init(cx);
        set_reduce_motion(true, cx);
    });
    let win_a = cx.add_window(|_, _| {
        Host::new(|| {
            gpui::div()
                .size_full()
                .p(px(20.))
                .flex()
                .items_start()
                .child(
                    Tooltip::new("Shared ID tip A")
                        .id("shared-id")
                        .delay(0)
                        .close_delay(0)
                        .presenter_mode(TooltipPresenterMode::SeparateWindow)
                        .child(
                            gpui::div()
                                .debug_selector(|| "btn-shared-a".into())
                                .child(Button::new("btn-shared-a").label("Win A")),
                        ),
                )
                .into_any_element()
        })
    });
    let mut vcx_a = VisualTestContext::from_window(win_a.into(), cx);
    vcx_a.simulate_mouse_move(point(px(-100.), px(-100.)), None, Modifiers::none());
    settle(&mut vcx_a);
    let btn_a = vcx_a.debug_bounds("btn-shared-a").expect("btn-shared-a");
    vcx_a.simulate_mouse_move(btn_a.center(), None, Modifiers::none());
    wait_ms(&mut vcx_a, 20);
    assert_eq!(vcx_a.update(|_, cx| cx.windows().len()), 2);

    let win_b = cx.add_window(|_, _| {
        Host::new(|| {
            gpui::div()
                .size_full()
                .p(px(20.))
                .flex()
                .items_start()
                .child(
                    Tooltip::new("Shared ID tip B")
                        .id("shared-id")
                        .delay(0)
                        .close_delay(0)
                        .presenter_mode(TooltipPresenterMode::SeparateWindow)
                        .child(
                            gpui::div()
                                .debug_selector(|| "btn-shared-b".into())
                                .child(Button::new("btn-shared-b").label("Win B")),
                        ),
                )
                .into_any_element()
        })
    });
    let mut vcx_b = VisualTestContext::from_window(win_b.into(), cx);
    vcx_b.simulate_mouse_move(point(px(-100.), px(-100.)), None, Modifiers::none());
    settle(&mut vcx_b);
    let btn_b = vcx_b.debug_bounds("btn-shared-b").expect("btn-shared-b");
    let popup_a = vcx_a.update(|_, cx| {
        cx.windows()
            .into_iter()
            .find(|h| *h != win_a.into())
            .expect("popup A")
    });

    // Opening a tooltip with the same ElementId in window B transitions cleanly:
    // TooltipManager closes window A's tip, and window B gets its own popup window (3 total).
    vcx_b.simulate_mouse_move(btn_b.center(), None, Modifiers::none());
    wait_ms(&mut vcx_b, 20);
    let (win_count_after_b, popup_b) = vcx_b.update(|_, cx| {
        let count = cx.windows().len();
        let pb = cx
            .windows()
            .into_iter()
            .find(|h| *h != win_a.into() && *h != win_b.into())
            .expect("popup B");
        (count, pb)
    });
    assert_eq!(win_count_after_b, 3);
    assert_ne!(popup_b, popup_a);

    // Window A's delayed leave/cleanup must not close window B's active popup.
    vcx_a.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
    wait_ms(&mut vcx_a, 20);
    assert_eq!(
        vcx_b.update(|_, cx| cx.windows().len()),
        3,
        "cleaning up window A must leave window B's popup intact"
    );

    vcx_b.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
    wait_ms(&mut vcx_b, 20);
    assert_eq!(vcx_b.update(|_, cx| cx.windows().len()), 2);

    // Re-open in window A and window B? Even stronger: open in window B again, then close window A
    // completely so its PresenterLease::on_release runs asynchronously; window B's popup must survive.
    vcx_b.simulate_mouse_move(btn_b.center(), None, Modifiers::none());
    wait_ms(&mut vcx_b, 20);
    assert_eq!(vcx_b.update(|_, cx| cx.windows().len()), 3);

    vcx_a.update(|window, _| window.remove_window());
    vcx_b.run_until_parked();
    assert_eq!(
        vcx_b.update(|_, cx| cx.windows().len()),
        2,
        "closing owner window A must clean up window A only and keep window B + popup B alive"
    );

    vcx_b.simulate_mouse_move(point(px(500.), px(500.)), None, Modifiers::none());
    wait_ms(&mut vcx_b, 20);
    assert_eq!(vcx_b.update(|_, cx| cx.windows().len()), 1);
}
#[gpui::test]
fn external_presenter_receives_fractional_dpi_and_edge_coordinates(cx: &mut TestAppContext) {
    still();
    let events = Arc::new(Mutex::new(Vec::new()));
    let presenter: Arc<dyn TooltipPresenter> = Arc::new(RecordingPresenter {
        events: events.clone(),
    });
    let presenter_render = presenter.clone();

    let vcx = open_host(cx, move || {
        gpui::div()
            .p(px(4.))
            .child(
                Tooltip::new("DPI edge tip")
                    .id("tip-dpi")
                    .trigger(TooltipTrigger::Focus)
                    .presenter(presenter_render.clone())
                    .child(Button::new("btn-dpi").label("Edge")),
            )
            .into_any_element()
    });
    vcx.simulate_scale_factor_change(1.5);
    settle(vcx);

    press(vcx, "tab");
    settle(vcx);

    let (scale_factor, screen_bounds) = events
        .lock()
        .unwrap()
        .iter()
        .find_map(|e| match e {
            PresenterEvent::Present {
                scale_factor,
                trigger_screen_bounds,
                ..
            } => Some((*scale_factor, *trigger_screen_bounds)),
            PresenterEvent::Dismiss { .. } => None,
        })
        .expect("presenter must report fractional DPI");
    assert_eq!(scale_factor, 1.5);

    let monitor = Bounds {
        origin: point(px(0.), px(0.)),
        size: size(px(800.), px(600.)),
    };
    let pos = position_tooltip_popup(
        screen_bounds,
        size(px(140.), px(32.)),
        monitor,
        TooltipPlacement::Top,
        px(6.),
        scale_factor,
    );
    assert!(f32::from(pos.x) >= 0.0 && f32::from(pos.x) + 140.0 <= 800.0);
    assert!(f32::from(pos.y) >= 0.0 && f32::from(pos.y) + 32.0 <= 600.0);
}

#[gpui::test]
fn separate_window_focus_tooltip_dismisses_on_owner_deactivation(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .p(px(20.))
            .child(
                Tooltip::new("Focus separate tip")
                    .id("tip-focus-deact")
                    .trigger(TooltipTrigger::Focus)
                    .presenter_mode(TooltipPresenterMode::SeparateWindow)
                    .child(Button::new("btn-focus-deact").label("Focus me")),
            )
            .into_any_element()
    });
    vcx.update(|window, _| window.activate_window());
    settle(vcx);

    press(vcx, "tab");
    settle(vcx);
    assert_eq!(
        vcx.update(|_, cx| cx.windows().len()),
        2,
        "focused trigger must open separate popup window"
    );

    vcx.deactivate_window();
    settle(vcx);
    assert_eq!(
        vcx.update(|_, cx| cx.windows().len()),
        1,
        "owner window deactivation must dismiss separate focus tooltip"
    );
}
