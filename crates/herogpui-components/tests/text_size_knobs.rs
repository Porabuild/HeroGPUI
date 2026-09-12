//! Phase 5 `text_size` knobs: the builder stores the size and the label site
//! resolves it, with the v3 leading pair following `util::leading_for`.

#[test]
fn text_size_builders_reach_their_label_sites() {
    // Each owner pairs the resolved size with its own leading mechanism, so
    // the pairing pin is the file's exact spelling: `leading` alone would
    // match any variable, and badge's v3 leadings are Tailwind's unitless
    // fractional multipliers, not `util::leading_for` steps.
    for (file, source, paired) in [
        (
            "badge.rs",
            include_str!("../src/badge.rs"),
            "DefiniteLength::Fraction(1.34)",
        ),
        (
            "chip.rs",
            include_str!("../src/chip.rs"),
            "and_then(crate::util::leading_for)",
        ),
        (
            "breadcrumbs.rs",
            include_str!("../src/breadcrumbs.rs"),
            "leading_for(text_size)",
        ),
    ] {
        assert!(
            source.contains("self.text_size = Some(size.into());"),
            "{file}: the builder must store the size"
        );
        assert!(
            source.contains("self.text_size.unwrap_or("),
            "{file}: the label site must resolve the override"
        );
        assert!(
            source.contains(paired),
            "{file}: the leading must stay paired through `{paired}`"
        );
    }
}
