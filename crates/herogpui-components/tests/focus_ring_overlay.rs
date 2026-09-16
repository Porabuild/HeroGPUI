//! Every control draws `status-focused` as the paint-time SVG overlay ring.
//!
//! Source-shape checks, the way `pickers_deep` pins the Select row's ring: the
//! ring is geometry rasterised at paint time, so a headless render has no
//! pixel to read back. What each test guards is the *structure* that lets the
//! overlay work -- the overlay call itself, plus whatever makes the ring
//! escape a clip -- since an `overflow_hidden` ancestor silently swallows the
//! ring rather than failing.

/// The implementation half of a component source, without its `#[cfg(test)]`
/// module, so a string an assertion looks for cannot be satisfied by a test.
fn implementation(source: &str) -> &str {
    source
        .split("#[cfg(test)]")
        .next()
        .expect("the implementation section is always present")
}

#[test]
fn switch_rings_a_carrier_around_the_clipping_track() {
    let source = implementation(include_str!("../src/switch.rs"));
    assert!(
        source.contains("crate::util::ring_overlay_if_focused("),
        "the Switch track must take the overlay ring"
    );
    assert!(
        source.contains("let mut track = gpui::div().relative().child(track);"),
        "the overlay must hang on a non-clipping carrier, not on the track \
         itself, whose clip keeps the thumb shadow inside the perimeter"
    );
    assert!(
        source.contains(".overflow_hidden()"),
        "the track keeps its clip: it is what bounds the thumb's shadow"
    );
    assert!(
        source.contains("crate::button::uniform_ring_radius(None, track_r, &sx_corners)"),
        "an `sx` refinement that breaks the corner symmetry has no scalar \
         radius for the overlay and must fall back to the shadow ring"
    );
}

#[test]
fn toggle_button_rings_without_a_clip() {
    let source = implementation(include_str!("../src/toggle_button.rs"));
    assert!(
        !source.contains(".overflow_hidden()"),
        "ToggleButton must not clip its box -- Button, whose box it mirrors, \
         does not, and the clip would cut the overlay ring"
    );
    assert!(
        source.contains("crate::button::uniform_ring_radius(self.group_edge, radius, &sx_corners)"),
        "a grouped member whose corners do not resolve to one radius must keep \
         the spread-shadow ring, as Button does"
    );
    assert!(
        source.contains("crate::util::ring_overlay_if_focused("),
        "every other ToggleButton must take the overlay ring"
    );
}

#[test]
fn checkbox_rings_without_a_clip() {
    let source = implementation(include_str!("../src/checkbox.rs"));
    assert!(
        !source.contains(".overflow_hidden()"),
        "the checkbox control must not clip: both animated fills carry the \
         control radius themselves, and the clip would cut the overlay ring"
    );
    assert!(
        source.contains("crate::util::with_focus_ring_overlay("),
        "the checkbox control must take the overlay ring"
    );
}

#[test]
fn number_field_group_hands_its_ring_to_a_carrier() {
    let source = implementation(include_str!("../src/number_field.rs"));
    assert!(
        source.contains("crate::util::apply_field_chrome_ringless("),
        "the clipping group must take the chrome without a ring"
    );
    assert!(
        source.contains("crate::util::field_ring_carrier("),
        "the ring must hang on the non-clipping carrier outside the clip"
    );
    assert!(
        source.contains("if ramp_owns_ring {"),
        "past its first flip the chrome ramp owns the ring; the carrier must \
         stand down so the group never draws two"
    );
}

#[test]
fn date_field_group_hands_its_ring_to_a_carrier() {
    let source = implementation(include_str!("../src/date_picker/field.rs"));
    assert!(
        source.contains("crate::util::apply_field_chrome_ringless("),
        "the clipping `.date-input-group` must take the chrome without a ring"
    );
    assert!(
        source.contains("crate::util::field_ring_carrier("),
        "the ring must hang on the non-clipping carrier outside the clip"
    );
}

#[test]
fn input_rings_both_spellings_as_an_overlay() {
    let source = implementation(include_str!("../src/input.rs"));
    assert!(
        source.contains("crate::util::apply_field_chrome_ringless("),
        "both Input spellings must take the chrome without an inline ring"
    );
    assert!(
        source.contains("crate::util::with_field_ring_overlay(field, ring, radius, cx)"),
        "the single-line field does not clip, so it hosts the overlay itself"
    );
    assert!(
        source.contains("crate::util::field_ring_carrier(field_element, ring, radius, cx)"),
        "the multi-line field clips its wrapped text, so its ring rides the \
         non-clipping carrier"
    );
}
