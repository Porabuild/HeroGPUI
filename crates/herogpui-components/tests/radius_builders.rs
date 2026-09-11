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
        (
            "input.rs",
            include_str!("../src/input.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| crate::util::field_radius(cx));",
        ),
        (
            "input_otp.rs",
            include_str!("../src/input_otp.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| crate::util::field_radius(cx));",
        ),
        (
            "time_field.rs",
            include_str!("../src/time_field.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::field_radius(cx));",
        ),
        (
            "date_picker/field.rs",
            include_str!("../src/date_picker/field.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| crate::util::field_radius(cx));",
        ),
        (
            "color_picker/field.rs",
            include_str!("../src/color_picker/field.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::field_radius(cx));",
        ),
        (
            "select.rs",
            include_str!("../src/select.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::container_radius(cx));",
        ),
        (
            "combo_box.rs",
            include_str!("../src/combo_box.rs"),
            "self.radius = Some(radius.into());",
            "let container_radius = self.radius.unwrap_or_else(|| util::container_radius(cx));",
        ),
        (
            "autocomplete.rs",
            include_str!("../src/autocomplete.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::container_radius(cx));",
        ),
        (
            "input_group.rs",
            include_str!("../src/input_group.rs"),
            "self.radius = Some(radius.into());",
            "Some(radius) => input.radius(radius),",
        ),
    ] {
        assert!(source.contains(stored), "{file}: the builder must store");
        assert!(source.contains(resolved), "{file}: the render must resolve");
    }
}

// The five field-family boxes share `util::apply_field_chrome`, which paints
// the helper's radius *after* the field's own chain, so an override would be
// swallowed there. Each of them sets the resolved radius back over the chrome.
// Select, ComboBox and Autocomplete round their detached *panel* instead, and
// their trigger box stays the shared field chrome's, so none of them re-rounds.
#[test]
fn field_radii_survive_the_shared_chrome_and_detached_panels_do_not_re_round() {
    for (file, source) in [
        ("input.rs", include_str!("../src/input.rs")),
        ("input_group.rs", include_str!("../src/input_group.rs")),
        ("time_field.rs", include_str!("../src/time_field.rs")),
        (
            "date_picker/field.rs",
            include_str!("../src/date_picker/field.rs"),
        ),
        (
            "color_picker/field.rs",
            include_str!("../src/color_picker/field.rs"),
        ),
    ] {
        assert!(
            source.contains("The chrome paints the helper's radius last"),
            "{file}: the override must go back over the shared field chrome"
        );
        assert!(
            source.contains(".rounded(radius);"),
            "{file}: the override must go back over the shared field chrome"
        );
    }

    for (file, source) in [
        ("select.rs", include_str!("../src/select.rs")),
        ("combo_box.rs", include_str!("../src/combo_box.rs")),
        ("autocomplete.rs", include_str!("../src/autocomplete.rs")),
    ] {
        assert!(
            !source.contains("The chrome paints the helper's radius last"),
            "{file}: the detached panel's override must not reach the trigger box"
        );
    }
}

// `ColorField`'s editable path renders the inner `Input`'s own box, so the
// override rides along with the field box the way its `height` and
// `padding_x` do; the static display box paints its own.
#[test]
fn color_field_forwards_the_override_to_its_editable_input() {
    assert!(
        include_str!("../src/color_picker/field.rs")
            .contains("Some(radius) => input.radius(radius),"),
        "color_picker/field.rs: the editable path must forward the override to the inner field"
    );
}
