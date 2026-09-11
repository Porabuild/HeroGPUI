//! Phase 5 `text_size` knobs: the builder stores the size and the label site
//! resolves it, with the v3 leading pair following `util::leading_for`.

#[test]
fn text_size_builders_reach_their_label_sites() {
    for (file, source) in [
        ("badge.rs", include_str!("../src/badge.rs")),
        ("chip.rs", include_str!("../src/chip.rs")),
        ("breadcrumbs.rs", include_str!("../src/breadcrumbs.rs")),
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
            source.contains("leading_for")
                || source.contains("leading)")
                || source.contains("Fraction(1.34)"),
            "{file}: the leading must stay paired"
        );
    }
}
