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
        (
            "toggle_button.rs",
            include_str!("../src/toggle_button.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::control_radius(cx));",
        ),
        (
            "avatar.rs",
            include_str!("../src/avatar.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| {",
        ),
        (
            "tooltip.rs",
            include_str!("../src/tooltip.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::small_radius(cx));",
        ),
        (
            "modal.rs",
            include_str!("../src/modal.rs"),
            "self.radius = Some(radius.into());",
            "let panel_radius = radius.unwrap_or_else(|| crate::util::container_radius(cx));",
        ),
        (
            "alert.rs",
            include_str!("../src/alert.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::control_radius(cx));",
        ),
        (
            "toast.rs",
            include_str!("../src/toast.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::container_radius(cx));",
        ),
        (
            "popover.rs",
            include_str!("../src/popover.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::container_radius(cx));",
        ),
        (
            "dropdown.rs",
            include_str!("../src/dropdown.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::container_radius(cx));",
        ),
        (
            "table.rs",
            include_str!("../src/table.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::container_radius(cx)),",
        ),
        (
            "skeleton.rs",
            include_str!("../src/skeleton.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::hairline_radius(cx)),",
        ),
        (
            "progress.rs",
            include_str!("../src/progress.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or(radius);",
        ),
        (
            "meter.rs",
            include_str!("../src/meter.rs"),
            "self.radius = Some(radius.into());",
            "p = p.radius(radius);",
        ),
    ] {
        assert!(source.contains(stored), "{file}: the builder must store");
        assert!(source.contains(resolved), "{file}: the render must resolve");
    }
}
