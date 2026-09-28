//! Phase 5 `text_size` knobs: the builder stores the size and the label site
//! resolves it, with the v3 leading pair following `util::leading_for`
//! (Chip, Breadcrumbs) or the badge's own fractional multiplier.
//!
//! Observed, not read from source: a style probe in the Chip's and Badge's
//! children reads the text size and line height their label draws with, and
//! a breadcrumb item's box is one line of its link, so its height is the
//! resolved leading and its width grows with the type size.

mod harness;

use gpui::{prelude::*, px, AbsoluteLength, DefiniteLength, TestAppContext, TextStyle};
use harness::{open_host, seen_style, settle, still, style_probe, style_sink};
use herogpui_components::{Badge, BadgeAnchor, Breadcrumbs, Chip, Crumb};

fn size_of(style: &TextStyle) -> f32 {
    match style.font_size {
        AbsoluteLength::Pixels(p) => f32::from(p),
        other => panic!("the label size must be absolute pixels, got {other:?}"),
    }
}

fn line_of(style: &TextStyle) -> DefiniteLength {
    style.line_height
}

fn fixed(value: f32) -> DefiniteLength {
    DefiniteLength::Absolute(AbsoluteLength::Pixels(px(value)))
}

fn chip_style(cx: &mut TestAppContext, size: Option<f32>) -> TextStyle {
    still();
    let sink = style_sink();
    let seen = sink.clone();
    let vcx = open_host(cx, move || {
        let chip = Chip::new().child(style_probe(&sink));
        match size {
            Some(s) => chip.text_size(px(s)),
            None => chip,
        }
        .into_any_element()
    });
    settle(vcx);
    seen_style(&seen, "Chip")
}

fn badge_style(cx: &mut TestAppContext, size: Option<f32>) -> TextStyle {
    still();
    let sink = style_sink();
    let seen = sink.clone();
    let vcx = open_host(cx, move || {
        let badge = Badge::new().child(style_probe(&sink));
        let badge = match size {
            Some(s) => badge.text_size(px(s)),
            None => badge,
        };
        gpui::div()
            .flex()
            .items_start()
            .p(px(40.))
            .child(
                BadgeAnchor::new()
                    .child(gpui::div().w(px(64.)).h(px(64.)))
                    .child(badge),
            )
            .into_any_element()
    });
    settle(vcx);
    seen_style(&seen, "Badge")
}

/// The first crumb's item box, `(width, height)`.
fn crumb_box(cx: &mut TestAppContext, size: Option<f32>) -> (f32, f32) {
    still();
    let vcx = open_host(cx, move || {
        let crumbs = Breadcrumbs::new(vec![Crumb::new("Home"), Crumb::new("Docs")]).id("bc");
        match size {
            Some(s) => crumbs.text_size(px(s)),
            None => crumbs,
        }
        .into_any_element()
    });
    settle(vcx);
    let item = vcx
        .debug_bounds("Name(\"bc\")-item-0")
        .expect("the first crumb must be laid out");
    (f32::from(item.size.width), f32::from(item.size.height))
}

#[gpui::test]
fn chip_text_size_reaches_the_label_with_the_leading_for_pair(cx: &mut TestAppContext) {
    let stock = chip_style(cx, None);
    assert_eq!(size_of(&stock), 12., "Md chip labels are text-xs");
    assert_eq!(line_of(&stock), fixed(20.), "the chip's one 20px line");

    let sixteen = chip_style(cx, Some(16.));
    assert_eq!(size_of(&sixteen), 16., "`text_size` must replace the step");
    assert_eq!(
        line_of(&sixteen),
        fixed(24.),
        "a Tailwind step pairs with its own leading through `leading_for`"
    );

    let odd = chip_style(cx, Some(13.));
    assert_eq!(size_of(&odd), 13.);
    assert_eq!(
        line_of(&odd),
        fixed(20.),
        "a size with no Tailwind leading keeps the step's line"
    );
}

#[gpui::test]
fn badge_text_size_reaches_the_label_with_its_fractional_leading(cx: &mut TestAppContext) {
    let stock = badge_style(cx, None);
    assert_eq!(size_of(&stock), 12., "Md badges are text-xs");
    assert_eq!(line_of(&stock), DefiniteLength::Fraction(1.34));

    let big = badge_style(cx, Some(20.));
    assert_eq!(size_of(&big), 20., "`text_size` must replace the step");
    assert_eq!(
        line_of(&big),
        DefiniteLength::Fraction(1.34),
        "the badge's leading is a unitless multiplier, not a `leading_for` step"
    );
}

#[gpui::test]
fn breadcrumbs_text_size_reaches_the_link_with_the_leading_for_pair(cx: &mut TestAppContext) {
    let (stock_w, stock_h) = crumb_box(cx, None);
    assert!(
        (stock_h - 20.).abs() < 0.5,
        "`.breadcrumbs__link` is text-sm leading-5, got a {stock_h}px line"
    );
    let (big_w, big_h) = crumb_box(cx, Some(16.));
    assert!(
        (big_h - 24.).abs() < 0.5,
        "text_size(16) must pair with `leading_for`'s 24px, got {big_h}"
    );
    assert!(
        big_w > stock_w + 1.,
        "the larger type must widen the link ({stock_w} -> {big_w})"
    );
    let (_, odd_h) = crumb_box(cx, Some(15.));
    assert!(
        (odd_h - 20.).abs() < 0.5,
        "a size with no Tailwind leading keeps the 20px line, got {odd_h}"
    );
}
