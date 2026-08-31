//! Kbd anatomy and geometry against the pinned v3.2.4 `kbd.css`.
//!
//! Upstream pins `.kbd` to `inline-flex h-6 items-center space-x-0.5
//! rounded-lg bg-default px-2 text-center font-sans text-sm font-medium
//! whitespace-nowrap text-muted` plus `word-spacing: -0.25rem`, and the only
//! variant rule is `.kbd--light { @apply bg-transparent; }` — no border, no
//! shadow, no min-width. `Kbd` exposes no `Styled` refinement, so widths are
//! read through sibling probes and fixed-width parents; heights, padding,
//! gap, and order through `debug_bounds` probes inside the key.
//! Paint-only surfaces (background, text color, radius, font family/weight,
//! text alignment, word spacing) leave no trace in layout and are covered by
//! the `.shots` audits plus code reading, not here.

mod harness;

use gpui::{prelude::*, px, Bounds, Pixels, TestAppContext};
use harness::open_host;
use herogpui_components::Kbd;

fn square(name: &'static str, size: f32) -> gpui::Div {
    gpui::div()
        .size(px(size))
        .debug_selector(move || name.to_owned())
}

/// A one-line text leaf whose measured height is the active line height.
fn line(name: &'static str, text: &'static str) -> gpui::Div {
    gpui::div()
        .debug_selector(move || name.to_owned())
        .child(text)
}

fn need(cx: &mut gpui::VisualTestContext, name: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(name)
        .unwrap_or_else(|| panic!("{name} must paint"))
}

/// `h-6 px-2 space-x-0.5`: two 10px probes sit at the 8px inset, separated
/// by the 2px gap, in composition order, and the whole key is exactly
/// 8 + 10 + 2 + 10 + 8 = 38px wide. The flex parent shrink-wraps a block
/// child as well, so this proves the pinned geometry but cannot distinguish
/// the port's block flex from upstream's inline-flex (recorded in the kbd
/// reference metadata).
#[gpui::test]
fn kbd_measures_the_pinned_base_geometry(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .child(
                Kbd::new()
                    .child(square("kbd-a", 10.))
                    .child(square("kbd-b", 10.)),
            )
            .child(square("kbd-after", 10.))
            .into_any_element()
    });

    let first = need(cx, "kbd-a");
    assert_eq!(first.origin.x, px(8.), "px-2 insets the first child by 8px");
    assert_eq!(
        first.origin.y,
        px(7.),
        "items-center centers a 10px child in the 24px key"
    );

    let second = need(cx, "kbd-b");
    assert_eq!(
        second.origin.x,
        px(20.),
        "space-x-0.5 puts the second child 2px behind the first, which keeps the composition order"
    );
    assert_eq!(second.origin.y, px(7.));

    let after = need(cx, "kbd-after");
    assert_eq!(
        after.origin.x,
        px(38.),
        "the key hugs its content: 8 + 10 + 2 + 10 + 8, with no min-width"
    );
}

/// `h-6` with no vertical padding: a full-height probe fills all 24px at
/// y = 0, and the next sibling in a column starts exactly at the key's
/// bottom edge.
#[gpui::test]
fn kbd_keeps_the_pinned_h_6_height(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .flex_col()
            .items_start()
            .child(
                Kbd::new().child(
                    gpui::div()
                        .h_full()
                        .w(px(4.))
                        .debug_selector(|| "kbd-fill".to_owned()),
                ),
            )
            .child(square("kbd-below", 4.))
            .into_any_element()
    });

    let fill = need(cx, "kbd-fill");
    assert_eq!(
        fill.size.height,
        px(24.),
        "the 24px h-6 is all content box: no vertical padding"
    );
    assert_eq!(fill.origin.y, px(0.));

    let below = need(cx, "kbd-below");
    assert_eq!(below.origin.y, px(24.), "the key is exactly 24px tall");
}

/// `.kbd` carries no `min-width`: an empty key is exactly the two 8px
/// horizontal paddings wide.
#[gpui::test]
fn kbd_with_no_children_hugs_to_its_padding(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .child(Kbd::new())
            .child(square("kbd-after", 10.))
            .into_any_element()
    });

    let after = need(cx, "kbd-after");
    assert_eq!(
        after.origin.x,
        px(16.),
        "8px padding on each side, no min-width"
    );
}

/// `text-sm` is 14/20 in Tailwind 4.3.0; the port pins the 20px leading
/// explicitly because gpui's phi default would round 14 x 1.618 to 23. The
/// 16px default reference beside it still rounds to 26.
#[gpui::test]
fn kbd_text_runs_at_text_sm(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .child(Kbd::new().child(line("kbd-text", "K")))
            .child(line("default-text", "K"))
            .into_any_element()
    });

    let inside = need(cx, "kbd-text");
    assert_eq!(
        inside.size.height,
        px(20.),
        "the key's line height is the pinned text-sm leading"
    );
    let outside = need(cx, "default-text");
    assert_eq!(
        outside.size.height,
        px(26.),
        "the unstyled reference runs at the 16px default"
    );
}

/// `whitespace-nowrap` keeps `Shift Tab` on one line even when the key is
/// pinned to 36px so the paddings leave only 20px of content — the line
/// probe keeps the single 20px line height while an unwrapped reference
/// beside it wraps past its own single 26px default line.
#[gpui::test]
fn kbd_keeps_content_on_one_line(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        gpui::div()
            .flex()
            .child(
                gpui::div()
                    .w(px(36.))
                    .child(Kbd::new().child(line("kbd-nowrap", "Shift Tab").w(px(20.)))),
            )
            .child(line("kbd-wrapped", "Shift Tab").w(px(20.)))
            .into_any_element()
    });

    let inside = need(cx, "kbd-nowrap");
    assert_eq!(
        inside.size.height,
        px(20.),
        "nowrap keeps the content on one line at the pinned text-sm leading"
    );

    let reference = need(cx, "kbd-wrapped");
    assert!(
        reference.size.height > px(26.),
        "harness check: at 20px the same text wraps without nowrap — taller \
         than its own single 26px default line"
    );
}
