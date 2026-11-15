//! Deep geometry and clipping checks for overlay focus rings inside
//! virtualized and non-virtualized scrollable `Select` panels and scrollable
//! `Menu` panels, including zero `panel_padding`, narrow panels, scrolling,
//! and fractional DPI scaling.

use crate::harness;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gpui::{
    point, prelude::*, px, Bounds, Pixels, ScrollDelta, ScrollEvent, SharedString, TestAppContext,
    VisualTestContext,
};
use harness::{open_host, painted, press, settle, still, Painted};
use herogpui_components::{Menu, MenuItem, PickerItem, Select, SelectionMode};
use herogpui_theme::ActiveTheme;

/// Full radial reach outside a row's border-box required for the painted overlay
/// ring: `ring_offset_width (2px) + RING_WIDTH (2px) + RING_BLUR_REACH (3px)`.
const EXPECTED_RING_REACH: f32 = 7.0;

type MaskMap = Rc<RefCell<HashMap<SharedString, Bounds<Pixels>>>>;

fn row_mask_probe(key: &SharedString, masks: MaskMap) -> gpui::AnyElement {
    let key = key.clone();
    gpui::canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            masks
                .borrow_mut()
                .insert(key.clone(), window.content_mask().bounds);
        },
    )
    .size_0()
    .into_any_element()
}

fn find_row_ring_gap(
    scene: &Painted,
    bg: gpui::Hsla,
    gap: f32,
    row_bounds: Bounds<Pixels>,
) -> Option<&gpui::Quad> {
    let exp_x = f32::from(row_bounds.origin.x) - gap;
    let exp_y = f32::from(row_bounds.origin.y) - gap;
    let exp_w = f32::from(row_bounds.size.width) + gap * 2.0;
    let exp_h = f32::from(row_bounds.size.height) + gap * 2.0;
    let pos_tol = 1.5;

    scene.quads.iter().find(|q| {
        if q.border_color != bg {
            return false;
        }
        let borders = scene.borders(q);
        if !borders.iter().all(|w| *w >= 1.0 && *w <= gap + 0.5) {
            return false;
        }
        let qb = scene.bounds(q);
        (f32::from(qb.origin.x) - exp_x).abs() <= pos_tol
            && (f32::from(qb.origin.y) - exp_y).abs() <= pos_tol
            && (f32::from(qb.size.width) - exp_w).abs() <= pos_tol * 2.0
            && (f32::from(qb.size.height) - exp_h).abs() <= pos_tol * 2.0
    })
}

/// Asserts that:
/// 1. The target row's offset ring gap quad is painted and its own gap band is unclipped.
/// 2. The active ancestor `content_mask` captured inside `row_key`'s paint pass
///    clears ALL four outer edges of `row_bounds` by `EXPECTED_RING_REACH` (7px logical).
fn assert_all_four_ring_edges_unclipped(
    vcx: &mut VisualTestContext,
    label: &str,
    row_key: &str,
    row_bounds: Bounds<Pixels>,
    masks: &MaskMap,
) {
    let scene = painted(vcx);
    let (bg, gap) = vcx.update(|_, cx| {
        (
            cx.colors().background,
            f32::from(cx.layout().ring_offset_width),
        )
    });
    let ring = find_row_ring_gap(&scene, bg, gap, row_bounds).unwrap_or_else(|| {
        panic!(
            "{label}: expected offset ring gap quad around row {:?}, scene:\n{}",
            row_bounds,
            scene.describe()
        )
    });
    assert!(
        !scene.band_is_clipped(ring),
        "{label}: ring gap band itself was clipped (bounds={:?}, mask={:?})",
        scene.bounds(ring),
        scene.mask(ring),
    );
    let mask = *masks
        .borrow()
        .get(&SharedString::from(row_key.to_owned()))
        .unwrap_or_else(|| panic!("{label}: missing paint content_mask for {row_key}"));
    let tol = 0.75 / scene.scale.max(1.0);

    let row_left = f32::from(row_bounds.origin.x);
    let row_top = f32::from(row_bounds.origin.y);
    let row_right = row_left + f32::from(row_bounds.size.width);
    let row_bottom = row_top + f32::from(row_bounds.size.height);

    let mask_left = f32::from(mask.origin.x);
    let mask_top = f32::from(mask.origin.y);
    let mask_right = mask_left + f32::from(mask.size.width);
    let mask_bottom = mask_top + f32::from(mask.size.height);

    assert!(
        mask_left <= row_left - EXPECTED_RING_REACH + tol,
        "{label}: LEFT ring edge clipped! mask_left={mask_left:.2}, row_left={row_left:.2}, scale={}",
        scene.scale
    );
    assert!(
        mask_right >= row_right + EXPECTED_RING_REACH - tol,
        "{label}: RIGHT ring edge clipped! mask_right={mask_right:.2}, row_right={row_right:.2}, scale={}",
        scene.scale
    );
    assert!(
        mask_top <= row_top - EXPECTED_RING_REACH + tol,
        "{label}: TOP ring edge clipped! mask_top={mask_top:.2}, row_top={row_top:.2}, scale={}",
        scene.scale
    );
    assert!(
        mask_bottom >= row_bottom + EXPECTED_RING_REACH - tol,
        "{label}: BOTTOM ring edge clipped! mask_bottom={mask_bottom:.2}, row_bottom={row_bottom:.2}, scale={}",
        scene.scale
    );
}

fn select_items(count: usize) -> Vec<PickerItem> {
    (0..count)
        .map(|i| PickerItem::new(format!("row-{i:02}"), format!("Item {i:02}")))
        .collect()
}

fn menu_items(count: usize) -> Vec<MenuItem> {
    (0..count)
        .map(|i| MenuItem::new(format!("item-{i:02}"), format!("Action {i:02}")))
        .collect()
}

#[gpui::test]
fn virtualized_select_all_four_ring_edges_with_zero_padding_narrow_panel_and_scrolling(
    cx: &mut TestAppContext,
) {
    still();
    let masks: MaskMap = Rc::new(RefCell::new(HashMap::new()));
    let masks_render = masks.clone();
    let vcx = open_host(cx, move || {
        let masks_for_rows = masks_render.clone();
        gpui::div()
            .p(px(24.))
            .child(
                Select::new("sel-virt", select_items(18))
                    .aria_label("Virtualized Select")
                    .default_value(Some(SharedString::from("row-01")))
                    .row_height(px(36.))
                    .panel_padding(px(0.))
                    .panel_min_width(px(64.))
                    .w(px(64.))
                    .item_content(move |key, _, _, _| {
                        gpui::div()
                            .flex()
                            .items_center()
                            .child(row_mask_probe(key, masks_for_rows.clone()))
                            .child(key.clone())
                            .into_any_element()
                    }),
            )
            .into_any_element()
    });

    // Open with Tab -> Enter; selected row `row-01` gets initial cursor focus.
    press(vcx, "tab");
    press(vcx, "enter");
    settle(vcx);

    let panel_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-virt\")-panel")
        .expect("panel must open");
    let selected_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-virt\")-opt-1")
        .expect("selected row 1 bounds");
    assert_eq!(
        selected_bounds.size,
        gpui::size(panel_bounds.size.width, px(36.)),
        "zero-padding virtual row geometry must match panel width x 36px"
    );
    assert_all_four_ring_edges_unclipped(
        vcx,
        "virtualized select selected row (row-01)",
        "row-01",
        selected_bounds,
        &masks,
    );

    // Move to first row (`row-00`).
    press(vcx, "home");
    settle(vcx);
    let first_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-virt\")-opt-0")
        .expect("first row 0 bounds");
    assert_eq!(first_bounds.origin.x, panel_bounds.origin.x);
    assert_eq!(
        first_bounds.size,
        gpui::size(panel_bounds.size.width, px(36.))
    );
    assert_all_four_ring_edges_unclipped(
        vcx,
        "virtualized select first row (row-00)",
        "row-00",
        first_bounds,
        &masks,
    );

    // Move to last row (`row-17`), forcing scroll to bottom.
    press(vcx, "end");
    settle(vcx);
    let last_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-virt\")-opt-17")
        .expect("last row 17 bounds");
    assert_eq!(last_bounds.origin.x, panel_bounds.origin.x);
    assert_eq!(
        last_bounds.size,
        gpui::size(panel_bounds.size.width, px(36.))
    );
    assert_all_four_ring_edges_unclipped(
        vcx,
        "virtualized select last row (row-17)",
        "row-17",
        last_bounds,
        &masks,
    );
}

#[gpui::test]
fn non_virtualized_select_all_four_ring_edges_with_zero_padding_narrow_panel_and_scrolling(
    cx: &mut TestAppContext,
) {
    still();
    let masks: MaskMap = Rc::new(RefCell::new(HashMap::new()));
    let masks_render = masks.clone();
    // 32 items * 36px = 1152px > 1080px window height, forcing scroll even without row_height.
    let vcx = open_host(cx, move || {
        let masks_for_rows = masks_render.clone();
        gpui::div()
            .pt(px(260.))
            .pl(px(24.))
            .child(
                Select::new("sel-nat", select_items(32))
                    .aria_label("Natural Select")
                    .default_value(Some(SharedString::from("row-01")))
                    .panel_padding(px(0.))
                    .panel_min_width(px(64.))
                    .w(px(64.))
                    .item_content(move |key, _, _, _| {
                        gpui::div()
                            .flex()
                            .items_center()
                            .child(row_mask_probe(key, masks_for_rows.clone()))
                            .child(key.clone())
                            .into_any_element()
                    }),
            )
            .into_any_element()
    });

    press(vcx, "tab");
    press(vcx, "enter");
    settle(vcx);

    let panel_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-nat\")-panel")
        .expect("natural panel");
    let selected_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-nat\")-opt-1")
        .expect("natural selected row 1");
    assert_eq!(selected_bounds.origin.x, panel_bounds.origin.x);
    assert_eq!(selected_bounds.size.width, panel_bounds.size.width);
    assert_all_four_ring_edges_unclipped(
        vcx,
        "natural select selected row (row-01)",
        "row-01",
        selected_bounds,
        &masks,
    );

    press(vcx, "home");
    settle(vcx);
    let first_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-nat\")-opt-0")
        .expect("natural first row 0");
    assert_eq!(first_bounds.origin.y, panel_bounds.origin.y);
    assert_all_four_ring_edges_unclipped(
        vcx,
        "natural select first row (row-00)",
        "row-00",
        first_bounds,
        &masks,
    );

    press(vcx, "end");
    settle(vcx);
    let last_bounds = vcx
        .debug_bounds("select-list-Name(\"sel-nat\")-opt-31")
        .expect("natural last row 31");
    assert!(
        f32::from(last_bounds.origin.y + last_bounds.size.height)
            <= f32::from(panel_bounds.origin.y + panel_bounds.size.height) + 0.5,
        "last natural row must scroll cleanly into panel bottom"
    );
    assert_all_four_ring_edges_unclipped(
        vcx,
        "natural select last row (row-31)",
        "row-31",
        last_bounds,
        &masks,
    );
}

#[gpui::test]
fn scrollable_menu_all_four_ring_edges_with_zero_padding_first_selected_and_last(
    cx: &mut TestAppContext,
) {
    still();
    let masks: MaskMap = Rc::new(RefCell::new(HashMap::new()));
    let masks_render = masks.clone();
    let vcx = open_host(cx, move || {
        let masks_for_rows = masks_render.clone();
        gpui::div()
            .p(px(24.))
            .child(
                Menu::new("menu-gutters", menu_items(12))
                    .panel_padding(px(0.))
                    .panel_gap(px(0.))
                    .panel_min_width(px(64.))
                    .panel_max_height(px(144.))
                    .selection_mode(SelectionMode::Single)
                    .selected_keys(["item-01"])
                    .item_content(move |key, _| {
                        let sel = format!("mrow-{key}");
                        gpui::div()
                            .flex()
                            .items_center()
                            .child(
                                gpui::div()
                                    .w(px(10.))
                                    .h(px(10.))
                                    .debug_selector(move || sel),
                            )
                            .child(row_mask_probe(key, masks_for_rows.clone()))
                            .child(key.clone())
                            .into_any_element()
                    }),
            )
            .into_any_element()
    });

    // Establish keyboard focus-visible modality cleanly before navigating the Menu.
    press(vcx, "down");
    settle(vcx);

    let check_menu_row = |vcx: &mut VisualTestContext, label: &str, row_key: &str| {
        let scene = painted(vcx);
        let (bg, gap) = vcx.update(|_, cx| {
            (
                cx.colors().background,
                f32::from(cx.layout().ring_offset_width),
            )
        });
        let rings = scene.ring_gaps(bg, gap);
        assert_eq!(rings.len(), 1, "{label}: expected 1 menu ring gap");
        let ring_bounds = scene.bounds(rings[0]);
        let row_bounds = Bounds {
            origin: point(
                ring_bounds.origin.x + px(gap),
                ring_bounds.origin.y + px(gap),
            ),
            size: gpui::size(
                ring_bounds.size.width - px(gap * 2.),
                ring_bounds.size.height - px(gap * 2.),
            ),
        };
        // Row geometry inside zero-padding narrow menu must still match 64px min width & 36px height.
        assert!(
            f32::from(row_bounds.size.width) >= 64. - 0.5
                && f32::from(row_bounds.size.height) >= 36. - 0.5,
            "{label}: unexpected row geometry {:?}",
            row_bounds.size
        );
        assert_all_four_ring_edges_unclipped(vcx, label, row_key, row_bounds, &masks);
    };

    // Navigate to first row.
    press(vcx, "home");
    settle(vcx);
    let y_first = vcx
        .debug_bounds("mrow-item-00")
        .expect("menu first probe")
        .origin
        .y;
    check_menu_row(vcx, "zero-padding menu first row (item-00)", "item-00");

    // Move down to selected row (`item-01`) and check row step height without geometry shift.
    press(vcx, "down");
    settle(vcx);
    let y_selected = vcx
        .debug_bounds("mrow-item-01")
        .expect("menu selected probe")
        .origin
        .y;
    assert!(
        (f32::from(y_selected - y_first) - 36.0).abs() < 0.5,
        "zero-padding gap(0) menu row step must stay 36px, got {}",
        f32::from(y_selected - y_first)
    );
    check_menu_row(vcx, "zero-padding menu selected row (item-01)", "item-01");

    // Jump to last row (`item-11`) so the menu scrolls to its bottom endpoint with 0 panel padding.
    press(vcx, "end");
    settle(vcx);
    check_menu_row(
        vcx,
        "zero-padding menu scrolled last row (item-11)",
        "item-11",
    );

    // Scroll back slightly with wheel then return to End to confirm bottom gutter holds on wheel scroll.
    let end_probe = vcx.debug_bounds("mrow-item-11").expect("menu last probe");
    vcx.simulate_event(ScrollEvent {
        position: end_probe.origin,
        delta: ScrollDelta::Pixels(point(px(0.), px(20.))),
        ..Default::default()
    });
    settle(vcx);
    press(vcx, "down");
    settle(vcx);
    check_menu_row(
        vcx,
        "zero-padding menu post-wheel bottom row (item-11)",
        "item-11",
    );
}

#[gpui::test]
fn focus_ring_gutters_survive_fractional_dpi_scaling(cx: &mut TestAppContext) {
    for scale in [1.25_f32, 1.5_f32] {
        still();
        let masks: MaskMap = Rc::new(RefCell::new(HashMap::new()));
        let masks_render = masks.clone();
        let vcx = open_host(cx, move || {
            let masks_for_rows = masks_render.clone();
            gpui::div()
                .p(px(24.))
                .child(
                    Select::new("sel-dpi", select_items(16))
                        .aria_label("DPI Select")
                        .default_value(Some(SharedString::from("row-01")))
                        .row_height(px(36.))
                        .panel_padding(px(0.))
                        .panel_min_width(px(64.))
                        .w(px(64.))
                        .item_content(move |key, _, _, _| {
                            gpui::div()
                                .flex()
                                .items_center()
                                .child(row_mask_probe(key, masks_for_rows.clone()))
                                .child(key.clone())
                                .into_any_element()
                        }),
                )
                .into_any_element()
        });
        vcx.simulate_scale_factor_change(scale);
        settle(vcx);

        press(vcx, "tab");
        press(vcx, "enter");
        settle(vcx);
        let b1 = vcx
            .debug_bounds("select-list-Name(\"sel-dpi\")-opt-1")
            .expect("dpi row 1");
        assert_all_four_ring_edges_unclipped(
            vcx,
            &format!("select selected @ {scale}x"),
            "row-01",
            b1,
            &masks,
        );

        press(vcx, "home");
        settle(vcx);
        let b0 = vcx
            .debug_bounds("select-list-Name(\"sel-dpi\")-opt-0")
            .expect("dpi row 0");
        assert_all_four_ring_edges_unclipped(
            vcx,
            &format!("select first @ {scale}x"),
            "row-00",
            b0,
            &masks,
        );

        press(vcx, "end");
        settle(vcx);
        let b15 = vcx
            .debug_bounds("select-list-Name(\"sel-dpi\")-opt-15")
            .expect("dpi row 15");
        assert_all_four_ring_edges_unclipped(
            vcx,
            &format!("select last @ {scale}x"),
            "row-15",
            b15,
            &masks,
        );
        assert!(
            (f32::from(b15.size.height) - 36.0).abs() < 0.75,
            "fractional DPI must preserve 36px row height, got {:?}",
            b15.size
        );
    }
}
