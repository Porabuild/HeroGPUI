//! Per-component `radius` builders under the narrow parity exception: the
//! default stays the owning `util` helper, and the builder replaces it.

#[test]
fn radius_builders_override_their_helper_defaults() {
    for (file, source, stored, resolved) in [
        (
            "button.rs",
            include_str!("../src/button.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::control_radius(cx));",
        ),
        (
            "card.rs",
            include_str!("../src/card.rs"),
            "self.radius = Some(radius.into());",
            "unwrap_or_else(|| crate::util::container_radius(cx))",
        ),
        (
            "chip.rs",
            include_str!("../src/chip.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| crate::util::soft_radius(cx));",
        ),
        (
            "kbd.rs",
            include_str!("../src/kbd.rs"),
            "self.radius = Some(radius.into());",
            ".rounded(self.radius.unwrap_or_else(|| crate::util::key_radius(cx)))",
        ),
    ] {
        assert!(source.contains(stored), "{file}: the builder must store");
        assert!(source.contains(resolved), "{file}: the render must resolve");
    }
}
