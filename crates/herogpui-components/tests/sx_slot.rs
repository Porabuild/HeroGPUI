//! Behaviour tests for the `sx` slot: the one caller-owned styling slot every
//! component exposes, this port's answer to React's `sx`.
//!
//! Appearance itself is the `.shots/*.py` audits' job; these tests probe the
//! slot through the hit footprint the overridden geometry leaves, and through
//! the press callback that footprint gates. The window is the harness'
//! 1920x1080 and the button sits at the origin, so an override to `13x12`
//! puts the footprint at `0..13 x 0..12` exactly — a click at (6, 6) is inside
//! it and one at (20, 6) is inside the 36px control height and the label width
//! the size ladder would have given, but outside the override.

mod harness;

use gpui::{point, prelude::*, px, Modifiers, MouseMoveEvent, TestAppContext};
use herogpui_components::Button;

use harness::{click, events, open_host};
use herogpui_components::{Chip, ChipLabel};

#[gpui::test]
fn sx_pixel_size_replaces_the_control_footprint(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let pressed = changes.clone();
        Button::new("sx-sized")
            .label("Much wider than the override")
            .sx(|el| el.w(px(13.)).h(px(12.)))
            .on_press(move |_, _, _| pressed.borrow_mut().push("press".into()))
            .into_any_element()
    });
    click(cx, 6., 6.);
    click(cx, 20., 6.);
    assert_eq!(recorded.borrow().as_slice(), ["press"]);
}

#[gpui::test]
fn sx_background_keeps_the_press_path_through_the_hover_fade(cx: &mut TestAppContext) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let pressed = changes.clone();
        Button::new("sx-bg")
            .label("Tinted")
            .sx(|el| {
                el.bg(gpui::rgba(0xffa500ff))
                    .text_color(gpui::rgba(0x000000ff))
            })
            .on_press(move |_, _, _| pressed.borrow_mut().push("press".into()))
            .into_any_element()
    });
    // Hover first: an `sx` background replaces the fade's endpoints, and the
    // fade still owns the single hover listener the press path depends on.
    cx.simulate_event(MouseMoveEvent {
        position: point(px(36.), px(18.)),
        pressed_button: None,
        modifiers: Modifiers::none(),
    });
    cx.update(|window, _| window.refresh());
    click(cx, 36., 18.);
    assert_eq!(recorded.borrow().as_slice(), ["press"]);
}

#[gpui::test]
fn sx_replaces_the_ladder_height_on_a_plain_root(cx: &mut TestAppContext) {
    // No fade, no press skin: this pins the plain `apply_sx` path every other
    // component shares — the refinement lands after the size ladder's own `h`
    // and therefore wins.
    let cx = open_host(cx, move || {
        Chip::new()
            .child(ChipLabel::new().child("Sized"))
            .sx(|el| el.h(px(40.)))
            .into_any_element()
    });
    cx.run_until_parked();
    let bounds = cx.debug_bounds("chip").unwrap();
    assert!(
        (f32::from(bounds.size.height) - 40.0).abs() < 0.5,
        "the sx height should replace the ladder height, got {bounds:?}"
    );
}

#[gpui::test]
fn sx_background_with_hover_bg_keeps_the_press_path_through_the_hover_fade(
    cx: &mut TestAppContext,
) {
    let changes = events();
    let recorded = changes.clone();
    let cx = open_host(cx, move || {
        let pressed = changes.clone();
        Button::new("sx-bg-hover")
            .label("Tinted")
            .sx(|el| {
                el.bg(gpui::rgba(0xffa500ff))
                    .text_color(gpui::rgba(0x000000ff))
            })
            // Naming the hover fill un-freezes the fade the `sx` background
            // pinned: its two endpoints now differ, so the animated fill is
            // actually mounted while the pointer is over the button — and the
            // element id, and with it the single hover listener the press path
            // rides on, still has to survive that.
            .hover_bg(gpui::rgba(0x804000ff))
            .on_press(move |_, _, _| pressed.borrow_mut().push("press".into()))
            .into_any_element()
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(px(36.), px(18.)),
        pressed_button: None,
        modifiers: Modifiers::none(),
    });
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    click(cx, 36., 18.);
    assert_eq!(recorded.borrow().as_slice(), ["press"]);
}

fn surface_accordion(sx: Option<gpui::Hsla>) -> gpui::AnyElement {
    use herogpui_components::{Accordion, AccordionItem, AccordionVariant};
    let accordion = Accordion::new(vec![
        AccordionItem::new("one", "Item one"),
        AccordionItem::new("two", "Item two"),
    ])
    .id("acc-surface")
    .variant(AccordionVariant::Surface);
    match sx {
        Some(card) => accordion.sx(move |s| s.bg(card)).into_any_element(),
        None => accordion.into_any_element(),
    }
}

/// The stock card and both resting triggers paint the surface; this is what
/// lets the override case below fail.
#[gpui::test]
fn accordion_surface_triggers_rest_on_the_stock_card_fill(cx: &mut TestAppContext) {
    use herogpui_theme::ActiveTheme;
    let cx = open_host(cx, || surface_accordion(None));
    let surface = cx.update(|_, cx| cx.colors().surface.background);
    assert!(
        harness::painted(cx).filled(surface).len() >= 3,
        "the stock card and its two triggers paint the surface"
    );
}

/// A Surface accordion's root is its card, and each closed trigger rests on
/// the card fill through the hover fade's resting endpoint. With a solid `sx`
/// card colour that endpoint is the override, so no trigger paints the stock
/// surface over it — at rest, hovered, or while its fade runs back out.
#[gpui::test]
fn accordion_surface_triggers_rest_on_the_sx_card_fill(cx: &mut TestAppContext) {
    use harness::{has_color, painted, wait_real};
    use herogpui_theme::ActiveTheme;

    let card = gpui::hsla(0.33, 0.6, 0.45, 1.0);
    let cx = open_host(cx, move || surface_accordion(Some(card)));
    let (surface, wash) =
        cx.update(|_, cx| (cx.colors().surface.background, cx.colors().default.color));
    let rest = painted(cx);
    assert!(
        rest.filled(surface).is_empty(),
        "no trigger may cover the sx card with the stock surface:\n{}",
        rest.describe()
    );
    assert!(
        rest.filled(card).len() >= 3,
        "the card and both triggers rest on the sx fill"
    );

    // The first trigger is 52px tall at the origin.
    cx.simulate_mouse_move(point(px(60.), px(26.)), None, Modifiers::none());
    wait_real(cx, 250);
    let hovered = painted(cx);
    assert!(
        has_color(&hovered.solids(), wash),
        "a hovered trigger fades to the bg-default wash, not to the sx fill"
    );
    assert!(hovered.filled(surface).is_empty());

    cx.simulate_mouse_move(point(px(-100.), px(-100.)), None, Modifiers::none());
    wait_real(cx, 250);
    let left = painted(cx);
    assert!(left.filled(surface).is_empty());
    assert!(left.filled(card).len() >= 3);
}
