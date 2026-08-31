//! Alert anatomy and geometry against the pinned v3.2.4 `alert.css` and
//! `alert.tsx`.
//!
//! The pinned contract: `.alert` is `flex w-full flex-row items-start
//! justify-start gap-4 bg-surface px-4 py-3 shadow-surface` with
//! `border-radius: min(32px, var(--radius-3xl))`; `.alert__content` is `flex
//! h-full grow flex-col items-start` with **no** gap; `.alert__title` is
//! `text-sm leading-6 font-medium`; `.alert__description` is `text-sm
//! text-muted`; `.alert__indicator` is `flex items-center justify-center p-1`
//! around a `box-content size-4` glyph. `alert.tsx` maps every status to a
//! fixed glyph — default falls through to the same Info as accent, success to
//! the circled check, warning to the triangle, danger to the
//! circle-exclamation — and `status` defaults to `"default"`.
//!
//! The paint-only half (which glyph path, which colour, which weight) leaves
//! no trace in layout on this platform — the test asset source answers
//! `Ok(None)` for every `svg()`, and no text style can be probed from outside
//! the component's own tree — so those are pinned by source scanning in
//! `pinned_source` below, following the `kbd_deep.rs` split.

mod harness;

use gpui::{prelude::*, px, Bounds, Pixels, TestAppContext, VisualTestContext};
use harness::{click, events, open_host};
use herogpui_components::{Alert, Color};

// ---------------------------------------------------------------------------
// Pinned source: the paint-only contract
// ---------------------------------------------------------------------------

#[cfg(test)]
mod pinned_source {
    fn source() -> &'static str {
        include_str!("../src/alert.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present")
    }

    /// The implementation without comments: the struct doc legitimately *names*
    /// the removed v2 props, so a "no seam" scan must read code, not prose.
    fn code() -> String {
        source()
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The v3 API table's `status` row reads `"default"` — the migration guide
    /// removed v2's `color="default"` seed, and `alert.tsx`'s `defaultIcon`
    /// arm falls through for the very same reason: there is no accent default.
    #[test]
    fn the_unstyled_status_is_default() {
        assert!(
            source().contains("color: Color::Default,"),
            "Alert::new must seed `status` with Color::Default, not Accent"
        );
    }

    /// `alert.tsx` `getDefaultIcon`: accent and the `default` fall-through both
    /// draw Info, success the check, warning the triangle, danger the
    /// circle-exclamation. The old port drew one glyph for every status.
    #[test]
    fn every_status_draws_its_pinned_glyph() {
        let src = source();
        assert!(
            src.contains("Color::Default | Color::Accent => icons::INFO_CIRCLE"),
            "default and accent must both draw the Info glyph"
        );
        assert!(
            src.contains("Color::Success => icons::CHECK_CIRCLE"),
            "success must draw the pinned circled-check `SuccessIcon`, not the \
             bare checkbox checkmark"
        );
        assert!(
            src.contains("Color::Warning => icons::WARNING_TRIANGLE"),
            "warning must draw the triangle glyph"
        );
        assert!(
            src.contains("Color::Danger => icons::CIRCLE_EXCLAMATION"),
            "danger must draw the circle-exclamation glyph"
        );
        assert!(
            !src.contains("icons::ELLIPSIS"),
            "the placeholder dots glyph is not any status's pinned glyph"
        );
    }

    /// `.alert__indicator` is `p-1` around `box-content size-4`: a 24px box
    /// with a 16px glyph, not an 18px glyph in a bare box.
    #[test]
    fn the_indicator_is_a_p1_box_around_a_16px_glyph() {
        let src = source();
        assert!(
            src.contains(".p(px(4.))"),
            "the indicator box must carry the p-1 padding"
        );
        assert!(
            src.contains(".size(px(16.))"),
            "the pinned glyph is `size-4` (16px), not 18px"
        );
        assert!(
            !src.contains(".size(px(18.))"),
            "the old 18px glyph has no pinned source"
        );
        assert!(
            src.contains(".items_center()") && src.contains(".justify_center()"),
            "the glyph must be centered inside the p-1 box"
        );
    }

    /// `.alert__title` is `text-sm leading-6 font-medium` — 14px over a 24px
    /// line, medium weight, never semibold.
    #[test]
    fn the_title_is_medium_14_over_24() {
        let src = source();
        assert!(
            src.contains(".line_height(px(24.))"),
            "the title line box is `leading-6` (24px)"
        );
        assert!(
            src.contains("FontWeight::MEDIUM"),
            "the pinned title weight is `font-medium`"
        );
        assert!(
            !src.contains("FontWeight::SEMIBOLD"),
            "the old semibold title weight has no pinned source"
        );
    }

    /// `.alert__description` is `text-sm text-muted`: 14px text with the
    /// `text-sm` line height (20px), painted `text-muted` rather than
    /// inheriting the foreground.
    #[test]
    fn the_description_is_muted_with_the_pinned_leading() {
        let src = source();
        assert!(
            src.contains(".text_color(colors.muted)"),
            "the description must paint `text-muted`, not the inherited foreground"
        );
        assert!(
            src.contains(".line_height(px(20.))"),
            "the description line box is the `text-sm` leading (20px)"
        );
    }

    /// `.alert__content` is `flex h-full grow flex-col items-start` — no gap
    /// utility at all. The old port invented a 2px gap between title and
    /// description, which stretched the card to 66px where the pinned one is
    /// 68px of pure leading.
    #[test]
    fn the_content_column_invents_no_gap() {
        assert!(
            !source().contains("gap(px(2.))"),
            "the content column must not invent a gap: `.alert__content` has none"
        );
        assert!(
            source().contains(".gap(px(16.))"),
            "the root's `gap-4` (16px) between indicator and content must stay"
        );
    }

    /// `.alert` carries `shadow-surface` — the surface elevation token, empty
    /// in dark mode — alongside `bg-surface`.
    #[test]
    fn the_container_paints_the_surface_shadow() {
        assert!(
            source().contains("surface_shadow"),
            "the alert container must apply the surface shadow token"
        );
    }

    /// v3 removed `isClosable`/`onClose` from Alert (`isClosable` is recorded
    /// as removed in the migration guide), so no built-in close affordance may
    /// reappear: no builder, no callback, no close glyph.
    #[test]
    fn no_v2_close_seam_exists() {
        let src = code();
        assert!(
            !src.contains("is_closable") && !src.contains("isClosable"),
            "v3 removed `isClosable`; a close affordance is composed by the caller"
        );
        assert!(
            !src.contains("on_close") && !src.contains("onClose"),
            "v3 removed `onClose` from Alert"
        );
        assert!(
            !src.contains("icons::CLOSE"),
            "Alert must not render a built-in close glyph"
        );
        assert!(
            !src.contains("on_click"),
            "an informational alert carries no click handlers at all"
        );
    }
}

// ---------------------------------------------------------------------------
// Headless geometry: the laid-out half of the same contract
// ---------------------------------------------------------------------------

fn need(cx: &mut VisualTestContext, name: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(name)
        .unwrap_or_else(|| panic!("{name} must paint"))
}

/// Checks the whole laid-out anatomy of an alert with a title and a
/// description at the window's top-left: `px-4 py-3` insets, the 24px
/// `p-1`-boxed indicator at the start, the content column one `gap-4` behind
/// it, a 24px `leading-6` title line and a 20px `text-sm` description line
/// directly beneath it (no invented gap), all inside a `w-full` card 68px
/// tall: 12 + 24 + 20 + 12.
fn assert_pinned_geometry(cx: &mut VisualTestContext) {
    let root = need(cx, "alert-root");
    assert_eq!(
        root.size.width,
        px(1920.),
        "`.alert` is `w-full`: the card spans the window"
    );
    assert_eq!(
        root.size.height,
        px(68.),
        "py-3 (12) + title 24 + description 20 + py-3 (12): the pinned card \
         is 68px tall with no content gap"
    );

    let indicator = need(cx, "alert-indicator");
    assert_eq!(
        (
            indicator.origin.x,
            indicator.origin.y,
            indicator.size.width,
            indicator.size.height
        ),
        (px(16.), px(12.), px(24.), px(24.)),
        "the indicator is a 24px `p-1` box (4 + 16px glyph + 4) at the px-4 \
         py-3 inset"
    );

    let content = need(cx, "alert-content");
    assert_eq!(
        (content.origin.x, content.origin.y),
        (px(56.), px(12.)),
        "the content column starts one `gap-4` (16px) behind the 24px \
         indicator: 16 + 24 + 16"
    );

    let title = need(cx, "alert-title");
    assert_eq!(
        (title.origin.x, title.origin.y, title.size.height),
        (px(56.), px(12.), px(24.)),
        "the title is one 24px `leading-6` line at the content column's top"
    );

    let description = need(cx, "alert-description");
    assert_eq!(
        (
            description.origin.x,
            description.origin.y,
            description.size.height
        ),
        (px(56.), px(36.), px(20.)),
        "the description is one 20px `text-sm` line directly under the title: \
         no invented gap between them"
    );
}

/// The default status arm lays out the full pinned anatomy.
#[gpui::test]
fn alert_default_arm_lays_out_the_pinned_anatomy(cx: &mut TestAppContext) {
    let cx = open_host(cx, || {
        Alert::new("New features available")
            .description("Check out our latest updates including dark mode support.")
            .into_any_element()
    });
    assert_pinned_geometry(cx);
}

/// Every named status arm keeps the same anatomy — in particular the same
/// 24px `p-1` indicator box, which is only 24px when every arm draws the
/// pinned 16px glyph, and the same 68px card.
#[gpui::test]
fn alert_status_arms_keep_the_pinned_anatomy(cx: &mut TestAppContext) {
    for status in [Color::Accent, Color::Success, Color::Warning, Color::Danger] {
        let cx = open_host(cx, move || {
            Alert::new("Status arm")
                .status(status)
                .description("Every arm carries a description.")
                .into_any_element()
        });
        let indicator = need(cx, "alert-indicator");
        assert_eq!(
            (indicator.size.width, indicator.size.height),
            (px(24.), px(24.)),
            "{status:?}: every status arm draws the 16px glyph inside the p-1 box"
        );
        let root = need(cx, "alert-root");
        assert_eq!(
            root.size.height,
            px(68.),
            "{status:?}: the status never changes the card's geometry"
        );
    }
}

/// A bare title-only alert is 48px tall — py-3 plus one `leading-6` line — and
/// renders no description row at all.
#[gpui::test]
fn alert_without_description_keeps_the_leading_only_height(cx: &mut TestAppContext) {
    let cx = open_host(cx, || Alert::new("Profile updated").into_any_element());
    let root = need(cx, "alert-root");
    assert_eq!(
        root.size.height,
        px(48.),
        "py-3 (12) + one 24px title line + py-3 (12) = 48px"
    );
    assert!(
        cx.debug_bounds("alert-description").is_none(),
        "a title-only alert paints no description row"
    );
}

// ---------------------------------------------------------------------------
// The composed-close seam: clicking can never close an alert
// ---------------------------------------------------------------------------

/// v3 removed `isClosable`/`onClose` from Alert, so there is no built-in close
/// affordance to press — neither at the old v2 close-glyph spot nor anywhere
/// on the alert, whose indicator is not interactive either. A probe element
/// below the alert proves the click machinery works, so "nothing recorded"
/// means the presses really landed on inert alert pixels. (The composed
/// CloseButton path — the only v3 way to close — is driven in `feedback.rs`.)
#[gpui::test]
fn alert_has_no_click_close_seam(cx: &mut TestAppContext) {
    let recorded = events();
    let probe = recorded.clone();
    let cx = open_host(cx, move || {
        let probe = probe.clone();
        gpui::div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(Alert::new("Unsaved changes").description("No built-in close here"))
            .child(
                gpui::div()
                    .id("alert-deep-probe")
                    .w(px(40.))
                    .h(px(20.))
                    .cursor_pointer()
                    .on_click(move |_, _, _| probe.borrow_mut().push("probe".into())),
            )
            .into_any_element()
    });

    // The pinned card is 68px tall, so the old v2 close glyph would sit at the
    // top-right inset — x 1890..1904, y 12..26, centre (1897, 19).
    click(cx, 1897., 19.);
    // The indicator box spans x 16..40, y 12..36; its centre (28, 24) must be
    // inert too: v3's indicator is not a control.
    click(cx, 28., 24.);
    assert!(
        recorded.borrow().is_empty(),
        "an alert has no built-in close seam to press, anywhere on the card"
    );

    // The probe sits 10px below the 68px card: y 78..98, centre (20, 88).
    click(cx, 20., 88.);
    assert_eq!(
        recorded.borrow().as_slice(),
        ["probe"],
        "the probe below must confirm the click machinery works"
    );
}
