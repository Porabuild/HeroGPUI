//! Visual geometry and paint-order tests for in-window [`Tooltip`] positioning.
//!
//! Covers viewport-edge clamping and primary-axis flipping (`Left`, `Right`,
//! `Top`, `Bottom`), arrow and entry-slide alignment after flips, narrow-window
//! wrapping (multi-word prose and long unbroken URLs), rich `.body(...)`
//! surfaces, fractional DPI alignment, and deferred Z-order above open
//! [`Popover`] panels.

use crate::harness;

use gpui::{prelude::*, px, size, Bounds, Modifiers, Pixels, Size, TestAppContext};
use harness::{open_host, painted, press, settle, still, wait_ms};
use herogpui_components::{Button, Popover, Tooltip, TooltipPlacement, TooltipTrigger};
use herogpui_theme::{
    set_theme, ActiveTheme, ComponentTheme, ComponentThemes, Theme, TooltipStyle,
};

const EDGE_MARGIN: f32 = 8.0;

fn assert_inside_viewport(bounds: Bounds<Pixels>, viewport: Size<Pixels>, label: &str) {
    let left = f32::from(bounds.left());
    let top = f32::from(bounds.top());
    let right = f32::from(bounds.right());
    let bottom = f32::from(bounds.bottom());
    let vw = f32::from(viewport.width);
    let vh = f32::from(viewport.height);
    assert!(
        left >= EDGE_MARGIN - 0.5 && right <= vw - EDGE_MARGIN + 0.5,
        "{label} horizontal bounds [{left}, {right}] must stay inside [{EDGE_MARGIN}, {}] (viewport {vw}x{vh})",
        vw - EDGE_MARGIN
    );
    assert!(
        top >= EDGE_MARGIN - 0.5 && bottom <= vh - EDGE_MARGIN + 0.5,
        "{label} vertical bounds [{top}, {bottom}] must stay inside [{EDGE_MARGIN}, {}] (viewport {vw}x{vh})",
        vh - EDGE_MARGIN
    );
}

#[gpui::test]
fn in_window_tooltip_clamps_and_flips_on_left_and_right_edges(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(24.))
            // Row 1: Top-placed tooltips flush against left and right edges (must clamp horizontally).
            .child(
                gpui::div()
                    .w_full()
                    .flex()
                    .justify_between()
                    .child(
                        Tooltip::new("Left edge clamped tooltip label")
                            .id("edge-left-top")
                            .placement(TooltipPlacement::Top)
                            .delay(0)
                            .close_delay(0)
                            .child(
                                gpui::div()
                                    .debug_selector(|| "trig-left-top".into())
                                    .w(px(24.))
                                    .h(px(24.))
                                    .child("L"),
                            ),
                    )
                    .child(
                        Tooltip::new("Right edge clamped tooltip label")
                            .id("edge-right-top")
                            .placement(TooltipPlacement::Top)
                            .delay(0)
                            .close_delay(0)
                            .child(
                                gpui::div()
                                    .debug_selector(|| "trig-right-top".into())
                                    .w(px(24.))
                                    .h(px(24.))
                                    .child("R"),
                            ),
                    ),
            )
            // Row 2: Left-placed tooltip at left edge and Right-placed tooltip at right edge (must flip with arrow).
            .child(
                gpui::div()
                    .w_full()
                    .flex()
                    .justify_between()
                    .child(
                        Tooltip::new("Flips from left to right")
                            .id("edge-left-flip")
                            .placement(TooltipPlacement::Left)
                            .show_arrow(true)
                            .delay(0)
                            .close_delay(0)
                            .child(
                                gpui::div()
                                    .debug_selector(|| "trig-left-flip".into())
                                    .w(px(24.))
                                    .h(px(24.))
                                    .child("LF"),
                            ),
                    )
                    .child(
                        Tooltip::new("Flips from right to left")
                            .id("edge-right-flip")
                            .placement(TooltipPlacement::Right)
                            .show_arrow(true)
                            .delay(0)
                            .close_delay(0)
                            .child(
                                gpui::div()
                                    .debug_selector(|| "trig-right-flip".into())
                                    .w(px(24.))
                                    .h(px(24.))
                                    .child("RF"),
                            ),
                    ),
            )
            .into_any_element()
    });

    let viewport = size(px(500.), px(360.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    // 1. Left-edge trigger with Top placement: clamps inside left margin and stays above trigger.
    let trig_lt = vcx.debug_bounds("trig-left-top").expect("trig-left-top");
    vcx.simulate_mouse_move(trig_lt.center(), None, Modifiers::none());
    settle(vcx);
    let tip_lt = vcx
        .debug_bounds("edge-left-top-tip")
        .expect("edge-left-top-tip");
    assert_inside_viewport(tip_lt, viewport, "left-edge Top tooltip");
    assert!(
        tip_lt.bottom() <= trig_lt.top(),
        "left-edge Top tooltip must stay above trigger: tip={tip_lt:?}, trig={trig_lt:?}"
    );

    // 2. Right-edge trigger with Top placement: clamps inside right margin and stays above trigger.
    let trig_rt = vcx.debug_bounds("trig-right-top").expect("trig-right-top");
    vcx.simulate_mouse_move(trig_rt.center(), None, Modifiers::none());
    settle(vcx);
    let tip_rt = vcx
        .debug_bounds("edge-right-top-tip")
        .expect("edge-right-top-tip");
    assert_inside_viewport(tip_rt, viewport, "right-edge Top tooltip");
    assert!(
        tip_rt.bottom() <= trig_rt.top(),
        "right-edge Top tooltip must stay above trigger: tip={tip_rt:?}, trig={trig_rt:?}"
    );

    // 3. Left-edge trigger with Left placement + arrow: flips to Right and moves arrow to left edge of tip.
    let trig_lf = vcx.debug_bounds("trig-left-flip").expect("trig-left-flip");
    vcx.simulate_mouse_move(trig_lf.center(), None, Modifiers::none());
    settle(vcx);
    let tip_lf = vcx
        .debug_bounds("edge-left-flip-tip")
        .expect("edge-left-flip-tip");
    let arrow_lf = vcx
        .debug_bounds("edge-left-flip-arrow")
        .expect("edge-left-flip-arrow");
    assert_inside_viewport(tip_lf, viewport, "left-edge flipped Right tooltip");
    assert!(
        tip_lf.left() >= trig_lf.right(),
        "Left tooltip at left edge must flip to Right: tip={tip_lf:?}, trig={trig_lf:?}"
    );
    assert!(
        arrow_lf.right() <= tip_lf.left() + px(1.),
        "flipped Right tooltip arrow must sit on left side of tip: arrow={arrow_lf:?}, tip={tip_lf:?}"
    );

    // 4. Right-edge trigger with Right placement + arrow: flips to Left and moves arrow to right edge of tip.
    let trig_rf = vcx
        .debug_bounds("trig-right-flip")
        .expect("trig-right-flip");
    vcx.simulate_mouse_move(trig_rf.center(), None, Modifiers::none());
    settle(vcx);
    let tip_rf = vcx
        .debug_bounds("edge-right-flip-tip")
        .expect("edge-right-flip-tip");
    let arrow_rf = vcx
        .debug_bounds("edge-right-flip-arrow")
        .expect("edge-right-flip-arrow");
    assert_inside_viewport(tip_rf, viewport, "right-edge flipped Left tooltip");
    assert!(
        tip_rf.right() <= trig_rf.left(),
        "Right tooltip at right edge must flip to Left: tip={tip_rf:?}, trig={trig_rf:?}"
    );
    assert!(
        arrow_rf.left() >= tip_rf.right() - px(1.),
        "flipped Left tooltip arrow must sit on right side of tip: arrow={arrow_rf:?}, tip={tip_rf:?}"
    );
}

#[gpui::test]
fn in_window_tooltip_flips_on_top_and_bottom_edges_with_aligned_arrows(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .child(
                Tooltip::new("Top edge flips below")
                    .id("edge-top-flip")
                    .placement(TooltipPlacement::Top)
                    .show_arrow(true)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-top-edge".into())
                            .w(px(64.))
                            .h(px(28.))
                            .child("Top"),
                    ),
            )
            .child(
                Tooltip::new("Bottom edge flips above")
                    .id("edge-bottom-flip")
                    .placement(TooltipPlacement::Bottom)
                    .show_arrow(true)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-bottom-edge".into())
                            .w(px(64.))
                            .h(px(28.))
                            .child("Bottom"),
                    ),
            )
            .into_any_element()
    });

    let viewport = size(px(500.), px(300.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    // Top-edge trigger requesting Top -> must flip below trigger with arrow on top edge of tip.
    let trig_top = vcx.debug_bounds("trig-top-edge").expect("trig-top-edge");
    vcx.simulate_mouse_move(trig_top.center(), None, Modifiers::none());
    settle(vcx);
    let tip_top = vcx
        .debug_bounds("edge-top-flip-tip")
        .expect("edge-top-flip-tip");
    let arrow_top = vcx
        .debug_bounds("edge-top-flip-arrow")
        .expect("edge-top-flip-arrow");
    assert_inside_viewport(tip_top, viewport, "top-edge flipped Bottom tooltip");
    assert!(
        tip_top.top() >= trig_top.bottom(),
        "Top tooltip at top edge must flip below trigger: tip={tip_top:?}, trig={trig_top:?}"
    );
    assert!(
        arrow_top.bottom() <= tip_top.top() + px(1.),
        "flipped Bottom tooltip arrow must sit above tip pointing at trigger: arrow={arrow_top:?}, tip={tip_top:?}"
    );

    // Bottom-edge trigger requesting Bottom -> must flip above trigger with arrow on bottom edge of tip.
    let trig_bot = vcx
        .debug_bounds("trig-bottom-edge")
        .expect("trig-bottom-edge");
    vcx.simulate_mouse_move(trig_bot.center(), None, Modifiers::none());
    settle(vcx);
    let tip_bot = vcx
        .debug_bounds("edge-bottom-flip-tip")
        .expect("edge-bottom-flip-tip");
    let arrow_bot = vcx
        .debug_bounds("edge-bottom-flip-arrow")
        .expect("edge-bottom-flip-arrow");
    assert_inside_viewport(tip_bot, viewport, "bottom-edge flipped Top tooltip");
    assert!(
        tip_bot.bottom() <= trig_bot.top(),
        "Bottom tooltip at bottom edge must flip above trigger: tip={tip_bot:?}, trig={trig_bot:?}"
    );
    assert!(
        arrow_bot.top() >= tip_bot.bottom() - px(1.),
        "flipped Top tooltip arrow must sit below tip pointing at trigger: arrow={arrow_bot:?}, tip={tip_bot:?}"
    );
}

#[gpui::test]
fn in_window_flipped_tooltip_preserves_aligned_entry_slide_animation(cx: &mut TestAppContext) {
    // Deliberately keep motion enabled so `anim::entering_zoom` applies entry slide.
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .child(
                Tooltip::new("Animated top flip")
                    .id("anim-top-flip")
                    .placement(TooltipPlacement::Top)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "anim-trig-top".into())
                            .w(px(64.))
                            .h(px(28.))
                            .child("Top"),
                    ),
            )
            .child(
                Tooltip::new("Animated bottom flip")
                    .id("anim-bottom-flip")
                    .placement(TooltipPlacement::Bottom)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "anim-trig-bottom".into())
                            .w(px(64.))
                            .h(px(28.))
                            .child("Bottom"),
                    ),
            )
            .into_any_element()
    });

    let viewport = size(px(500.), px(300.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    // Top-edge trigger flips Top -> Bottom: entry slide is -4px (starts above settled y and slides down).
    let trig_top = vcx.debug_bounds("anim-trig-top").expect("anim-trig-top");
    vcx.simulate_mouse_move(trig_top.center(), None, Modifiers::none());
    settle(vcx);
    let top_t0 = vcx
        .debug_bounds("anim-top-flip-tip")
        .expect("anim-top-flip-tip at t=0");
    wait_ms(vcx, 250);
    let top_settled = vcx
        .debug_bounds("anim-top-flip-tip")
        .expect("anim-top-flip-tip settled");
    assert!(
        top_settled.top() >= trig_top.bottom(),
        "flipped Top tooltip must settle below trigger: settled={top_settled:?}, trig={trig_top:?}"
    );
    assert!(
        top_t0.top() < top_settled.top() - px(1.),
        "flipped Bottom entry slide must start higher (slide_y = -4px) and move down: t0={top_t0:?}, settled={top_settled:?}"
    );

    // Bottom-edge trigger flips Bottom -> Top: entry slide is +4px (starts below settled y and slides up).
    let trig_bot = vcx
        .debug_bounds("anim-trig-bottom")
        .expect("anim-trig-bottom");
    vcx.simulate_mouse_move(trig_bot.center(), None, Modifiers::none());
    settle(vcx);
    let bot_t0 = vcx
        .debug_bounds("anim-bottom-flip-tip")
        .expect("anim-bottom-flip-tip at t=0");
    wait_ms(vcx, 250);
    let bot_settled = vcx
        .debug_bounds("anim-bottom-flip-tip")
        .expect("anim-bottom-flip-tip settled");
    assert!(
        bot_settled.bottom() <= trig_bot.top(),
        "flipped Bottom tooltip must settle above trigger: settled={bot_settled:?}, trig={trig_bot:?}"
    );
    assert!(
        bot_t0.top() > bot_settled.top() + px(1.),
        "flipped Top entry slide must start lower (slide_y = +4px) and move up: t0={bot_t0:?}, settled={bot_settled:?}"
    );
}

#[gpui::test]
fn in_window_tooltip_wraps_multi_word_and_unbroken_url_in_narrow_viewport(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .p(px(12.))
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(16.))
            .child(
                Tooltip::new("Short")
                    .id("narrow-short")
                    .placement(TooltipPlacement::Bottom)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-narrow-short".into())
                            .w(px(40.))
                            .h(px(24.))
                            .child("S"),
                    ),
            )
            .child(
                Tooltip::new("This multi word tooltip wraps cleanly inside a narrow window")
                    .id("narrow-multi")
                    .placement(TooltipPlacement::Bottom)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-narrow-multi".into())
                            .w(px(40.))
                            .h(px(24.))
                            .child("M"),
                    ),
            )
            .child(
                Tooltip::new(
                    "https://example.com/very/long/unbroken/path/segment/for/narrow/window/tooltip",
                )
                .id("narrow-url")
                .placement(TooltipPlacement::Bottom)
                .delay(0)
                .close_delay(0)
                .child(
                    gpui::div()
                        .debug_selector(|| "trig-narrow-url".into())
                        .w(px(40.))
                        .h(px(24.))
                        .child("U"),
                ),
            )
            .into_any_element()
    });

    // Narrow viewport (200px wide): max_w must cap at (200 - 16) = 184px.
    let viewport = size(px(200.), px(320.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    let trig_short = vcx
        .debug_bounds("trig-narrow-short")
        .expect("trig-narrow-short");
    vcx.simulate_mouse_move(trig_short.center(), None, Modifiers::none());
    settle(vcx);
    let tip_short = vcx
        .debug_bounds("narrow-short-tip")
        .expect("narrow-short-tip");
    assert_inside_viewport(tip_short, viewport, "narrow short tooltip");

    let trig_multi = vcx
        .debug_bounds("trig-narrow-multi")
        .expect("trig-narrow-multi");
    vcx.simulate_mouse_move(trig_multi.center(), None, Modifiers::none());
    settle(vcx);
    let tip_multi = vcx
        .debug_bounds("narrow-multi-tip")
        .expect("narrow-multi-tip");
    assert_inside_viewport(tip_multi, viewport, "narrow multi-word tooltip");
    assert!(
        tip_multi.size.width <= px(184.5),
        "multi-word tooltip must cap to viewport.width - 2*margin (184px): {tip_multi:?}"
    );
    assert!(
        tip_multi.size.height > tip_short.size.height,
        "multi-word tooltip must wrap vertically in narrow viewport: multi={tip_multi:?}, short={tip_short:?}"
    );

    let trig_url = vcx
        .debug_bounds("trig-narrow-url")
        .expect("trig-narrow-url");
    vcx.simulate_mouse_move(trig_url.center(), None, Modifiers::none());
    settle(vcx);
    let tip_url = vcx.debug_bounds("narrow-url-tip").expect("narrow-url-tip");
    assert_inside_viewport(tip_url, viewport, "narrow unbroken URL tooltip");
    assert!(
        tip_url.size.width <= px(184.5),
        "unbroken URL tooltip must cap to viewport.width - 2*margin (184px): {tip_url:?}"
    );
    assert!(
        tip_url.size.height > tip_short.size.height,
        "unbroken URL tooltip must wrap vertically in narrow viewport: url={tip_url:?}, short={tip_short:?}"
    );
}

#[gpui::test]
fn in_window_rich_body_tooltip_stays_in_viewport_and_does_not_collapse_to_trigger(
    cx: &mut TestAppContext,
) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .justify_end()
            .items_start()
            .child(
                Tooltip::new("Rich summary")
                    .id("rich-corner")
                    .placement(TooltipPlacement::Top)
                    .delay(0)
                    .close_delay(0)
                    .body(|_, _| {
                        gpui::div()
                            .debug_selector(|| "rich-corner-body".into())
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child("Rich tooltip heading")
                            .child("Additional rich details that should stay inside the window")
                            .into_any_element()
                    })
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-rich-corner".into())
                            .w(px(24.))
                            .h(px(24.))
                            .child("?"),
                    ),
            )
            .into_any_element()
    });

    // Wide viewport first: top-right 24x24 trigger must flip below, clamp horizontally,
    // and NOT collapse its rich body to the 24px trigger width.
    let wide = size(px(500.), px(300.));
    vcx.simulate_resize(wide);
    settle(vcx);

    let trig_wide = vcx
        .debug_bounds("trig-rich-corner")
        .expect("trig-rich-corner");
    vcx.simulate_mouse_move(trig_wide.center(), None, Modifiers::none());
    settle(vcx);

    let tip_wide = vcx
        .debug_bounds("rich-corner-tip")
        .expect("rich-corner-tip");
    assert_inside_viewport(tip_wide, wide, "rich body tooltip in wide viewport");
    assert!(
        tip_wide.top() >= trig_wide.bottom(),
        "top-right rich tooltip must flip below trigger: tip={tip_wide:?}, trig={trig_wide:?}"
    );
    assert!(
        tip_wide.size.width > px(100.) && tip_wide.size.width <= px(320.5),
        "rich body tooltip must size naturally up to 320px instead of collapsing to 24px trigger: {tip_wide:?}"
    );

    // Resize to narrow viewport while keeping the tooltip hovered: must cap to 184px and stay inside.
    let narrow = size(px(200.), px(300.));
    vcx.simulate_resize(narrow);
    settle(vcx);
    let trig_narrow = vcx
        .debug_bounds("trig-rich-corner")
        .expect("trig-rich-corner narrow");
    vcx.simulate_mouse_move(trig_narrow.center(), None, Modifiers::none());
    settle(vcx);

    let tip_narrow = vcx
        .debug_bounds("rich-corner-tip")
        .expect("rich-corner-tip narrow");
    assert_inside_viewport(tip_narrow, narrow, "rich body tooltip in narrow viewport");
    assert!(
        tip_narrow.size.width <= px(184.5),
        "rich body tooltip must respect narrow viewport max_w cap (184px): {tip_narrow:?}"
    );
}

#[gpui::test]
fn in_window_tooltip_inside_open_popover_positions_accurately_and_paints_above_popovers(
    cx: &mut TestAppContext,
) {
    still();
    let tip_bg = gpui::hsla(0.55, 0.85, 0.42, 1.0);
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .p(px(24.))
            .flex()
            .gap(px(24.))
            .items_start()
            .child(
                Popover::new(Button::new("pop-outer-trigger").label("Color picker"))
                    .id("pop-outer")
                    .default_open(true)
                    .show_close_button(false)
                    .title("Controls")
                    .child(
                        Tooltip::new("Copy hex color value")
                            .id("pop-inner-tip")
                            .recipe("accent-tip")
                            .trigger(TooltipTrigger::Focus)
                            .placement(TooltipPlacement::Top)
                            .show_arrow(true)
                            .delay(0)
                            .close_delay(0)
                            .child(
                                gpui::div()
                                    .debug_selector(|| "pop-inner-trigger".into())
                                    .child(Button::new("pop-inner-btn").label("Copy")),
                            ),
                    ),
            )
            // A second open Popover rendered AFTER the first proves deferred priority (> 0),
            // not merely tree order inside the first popover.
            .child(
                Popover::new(Button::new("pop-sibling-trigger").label("Sibling"))
                    .id("pop-sibling")
                    .default_open(true)
                    .show_close_button(false)
                    .title("Sibling panel")
                    .child(gpui::div().w(px(180.)).h(px(60.)).child("Sibling body")),
            )
            .into_any_element()
    });

    // Give the tooltip a distinct solid background so we can locate its exact painted quad.
    vcx.update(|window, cx| {
        let theme = Theme::builder("tooltip-zorder", Theme::light())
            .components(
                ComponentThemes::default().tooltip(
                    ComponentTheme::new(TooltipStyle::default())
                        .recipe("accent-tip", TooltipStyle::default().background(tip_bg)),
                ),
            )
            .build();
        set_theme(theme, cx);
        window.refresh();
    });

    let viewport = size(px(560.), px(360.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    // Focus the tooltip trigger inside the open popover (via Tab) to open the tooltip.
    let trig = vcx
        .debug_bounds("pop-inner-trigger")
        .expect("pop-inner-trigger inside open popover");
    for _ in 0..4 {
        press(vcx, "tab");
        settle(vcx);
        if vcx.debug_bounds("pop-inner-tip-tip").is_some() {
            break;
        }
    }

    let tip = vcx
        .debug_bounds("pop-inner-tip-tip")
        .expect("tooltip inside open popover must render");
    assert_inside_viewport(tip, viewport, "tooltip inside popover");

    // Verify the tooltip is anchored near its trigger inside the popover (not shifted to (0, 0)
    // or double-offset by the popover's own element offset).
    let vertical_gap = if tip.bottom() <= trig.top() {
        f32::from(trig.top() - tip.bottom())
    } else {
        f32::from(tip.top() - trig.bottom())
    };
    assert!(
        vertical_gap <= 24.0,
        "tooltip inside popover must stay anchored to its trigger: tip={tip:?}, trig={trig:?}"
    );

    // Verify paint order: the tooltip surface quad must paint AFTER every Popover panel quad
    // (including the sibling Popover rendered later in tree order).
    let scene = painted(vcx);
    let panel_bg = vcx.update(|_, cx| cx.colors().overlay.background);
    let popover_panel_max_ix = scene
        .quads
        .iter()
        .enumerate()
        .filter(|(_, q)| {
            q.background.as_solid() == Some(panel_bg)
                && f32::from(scene.bounds(q).size.width) >= 200.0
        })
        .map(|(i, _)| i)
        .max()
        .expect("open popovers must paint their overlay panel quads");
    let tip_quad_ix = scene
        .quads
        .iter()
        .position(|q| q.background.as_solid() == Some(tip_bg))
        .expect("tooltip surface must paint its background quad");
    assert!(
        tip_quad_ix > popover_panel_max_ix,
        "tooltip (quad #{tip_quad_ix}) must paint on top of all open popovers (last panel quad #{popover_panel_max_ix})"
    );
}

#[gpui::test]
fn in_window_tooltip_snaps_clamped_coordinates_to_fractional_dpi_grid(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .items_center()
            .justify_start()
            .child(
                Tooltip::new("Fractional DPI clamped tooltip")
                    .id("dpi-clamped")
                    .placement(TooltipPlacement::Top)
                    .delay(0)
                    .close_delay(0)
                    .child(
                        gpui::div()
                            .debug_selector(|| "trig-dpi".into())
                            .w(px(20.))
                            .h(px(20.))
                            .child("D"),
                    ),
            )
            .into_any_element()
    });

    vcx.simulate_scale_factor_change(1.5);
    let viewport = size(px(300.), px(200.));
    vcx.simulate_resize(viewport);
    settle(vcx);

    let trig = vcx.debug_bounds("trig-dpi").expect("trig-dpi");
    vcx.simulate_mouse_move(trig.center(), None, Modifiers::none());
    settle(vcx);

    let tip = vcx
        .debug_bounds("dpi-clamped-tip")
        .expect("dpi-clamped-tip");
    assert_inside_viewport(tip, viewport, "1.5x DPI clamped tooltip");

    let phys_x = f32::from(tip.origin.x) * 1.5;
    let phys_y = f32::from(tip.origin.y) * 1.5;
    assert!(
        (phys_x - phys_x.round()).abs() < 1e-3 && (phys_y - phys_y.round()).abs() < 1e-3,
        "clamped origin must align to 1.5x physical pixel grid: origin={:?} -> phys=({phys_x}, {phys_y})",
        tip.origin
    );
}

#[gpui::test]
fn in_window_tooltip_shrinks_margin_to_fit_compact_viewport(cx: &mut TestAppContext) {
    still();
    let vcx = open_host(cx, || {
        gpui::div()
            .size_full()
            .flex()
            .items_center()
            .pl(px(12.))
            .child(
                Tooltip::new("Compact viewport tooltip")
                    .id("compact-viewport")
                    .placement(TooltipPlacement::Right)
                    .delay(0)
                    .body(|_, _| {
                        gpui::div()
                            .w(px(180.))
                            .h(px(32.))
                            .child("Fits 52px bar")
                            .into_any_element()
                    })
                    .child(
                        gpui::div()
                            .id("compact-trigger")
                            .debug_selector(|| "compact-trigger".into())
                            .w(px(80.))
                            .h(px(28.)),
                    ),
            )
            .into_any_element()
    });
    for scale in [1.0_f32, 2.0] {
        vcx.simulate_scale_factor_change(scale);
        vcx.simulate_resize(size(px(360.), px(52.)));
        vcx.update(|window, _| window.refresh());
        let trigger = vcx
            .debug_bounds("compact-trigger")
            .expect("compact trigger bounds");
        vcx.simulate_mouse_move(trigger.center(), None, Modifiers::none());
        settle(vcx);
        let tip = vcx
            .debug_bounds("compact-viewport-tip")
            .expect("compact tooltip must paint");
        let viewport = vcx.update(|window, _| window.viewport_size());
        assert!(
            f32::from(tip.top()) >= -0.01
                && f32::from(tip.bottom()) <= f32::from(viewport.height) + 0.01
                && f32::from(tip.left()) >= -0.01
                && f32::from(tip.right()) <= f32::from(viewport.width) + 0.01,
            "compact tooltip {tip:?} must stay unclipped inside {viewport:?} at {scale}x"
        );
    }
}
