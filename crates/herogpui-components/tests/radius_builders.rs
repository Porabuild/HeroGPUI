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
        (
            "alert_dialog.rs",
            include_str!("../src/alert_dialog.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| util::container_radius(cx));",
        ),
        (
            "accordion.rs",
            include_str!("../src/accordion.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::container_radius(cx)),",
        ),
        (
            "close_button.rs",
            include_str!("../src/close_button.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or_else(|| crate::util::small_radius(cx));",
        ),
        (
            "link.rs",
            include_str!("../src/link.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| crate::util::small_radius(cx)),",
        ),
        (
            "checkbox.rs",
            include_str!("../src/checkbox.rs"),
            "self.radius = Some(radius.into());",
            "self.radius.unwrap_or_else(|| crate::util::mark_radius(cx))",
        ),
        (
            "radio_group.rs",
            include_str!("../src/radio_group.rs"),
            "self.radius = Some(radius.into());",
            "let control_radius = self.radius.unwrap_or_else(|| crate::util::key_radius(cx));",
        ),
        (
            "typography.rs",
            include_str!("../src/typography.rs"),
            "self.radius = Some(radius.into());",
            ".rounded(self.radius.unwrap_or_else(|| crate::util::mark_radius(cx)))",
        ),
        (
            "badge.rs",
            include_str!("../src/badge.rs"),
            "self.radius = Some(radius.into());",
            "let radius = self.radius.unwrap_or(radius);",
        ),
        (
            "tag_group.rs",
            include_str!("../src/tag_group.rs"),
            "self.radius = Some(radius.into());",
            ".unwrap_or_else(|| Self::step_radius(self.size, cx));",
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

// The dialog's panel is painted twice — once as the resting box and once as
// the geometry the entry zoom interpolates — so one hoisted binding has to
// feed both, the way Select's and Popover's panels do.
#[test]
fn alert_dialog_panel_and_entry_zoom_share_one_binding() {
    let source = include_str!("../src/alert_dialog.rs");
    assert!(
        source.contains("crate::anim::ZoomBox::panel(px(24.), radius)"),
        "alert_dialog.rs: the entry zoom must interpolate the resolved radius"
    );
    assert!(
        !source.contains("ZoomBox::panel(px(24.), util::container_radius(cx))"),
        "alert_dialog.rs: the zoom must not fall back to the helper on its own"
    );
}

// Only the `Surface` variant paints a card, so the override lands inside that
// branch; the separators' hairline rule is an inner part and keeps its own.
#[test]
fn accordion_override_is_inert_on_the_flush_default_variant() {
    let source = include_str!("../src/accordion.rs");
    assert!(
        source.contains("AccordionVariant::Default => gpui::div(),"),
        "accordion.rs: the Default variant must keep painting no card"
    );
    assert!(
        source.contains(".rounded(crate::util::hairline_radius(cx))"),
        "accordion.rs: the separator's hairline mark is an inner part"
    );
}

// The close button's box is its whole shape, so the press-scale derivation
// multiplies the resolved value instead of the helper's — the same way the
// `Button` press box follows its override.
#[test]
fn close_button_press_scale_follows_the_override() {
    let source = include_str!("../src/close_button.rs");
    assert!(
        source.contains("let pressed_radius = px(f32::from(radius) * PRESS_SCALE);"),
        "close_button.rs: the pressed box must scale the resolved radius"
    );
    assert!(
        !source.contains("crate::util::small_radius(cx)) * PRESS_SCALE"),
        "close_button.rs: the pressed box must not read the helper directly"
    );
}

// `is_round` is documented shape semantics, so the override replaces the
// helper's *value* in the `else` branch alone and the circle survives it; the
// control and its animated fill layers both read `control_radius`.
#[test]
fn checkbox_override_never_overrides_the_round_circle() {
    let source = include_str!("../src/checkbox.rs");
    let circle = source
        .find("let control_radius = if self.is_round {")
        .expect("checkbox.rs: the documented `is_round` branch must stay");
    let circle_px = source[circle..]
        .find("box_px / 2.0")
        .expect("checkbox.rs: the round control keeps `box_px / 2.0`");
    let override_at = source[circle..]
        .find("self.radius.unwrap_or_else(|| crate::util::mark_radius(cx))")
        .expect("checkbox.rs: the override replaces the helper's value");
    assert!(
        circle_px < override_at,
        "checkbox.rs: the override must live in the `else` branch, after the circle"
    );
    assert!(
        !source.contains("self.radius.unwrap_or(box_px"),
        "checkbox.rs: the override must not replace the circle itself"
    );
}

// The control circle's radius feeds its pressed box too, so a pressed radio
// does not snap back to the helper; the selected dot inside is an inner part
// and keeps its own.
#[test]
fn radio_control_press_follows_the_override_and_the_dot_does_not() {
    let source = include_str!("../src/radio_group.rs");
    assert!(
        source.contains("radius: control_radius,"),
        "radio_group.rs: the pressed box must scale the resolved radius"
    );
    assert!(
        !source.contains("radius: crate::util::key_radius(cx)"),
        "radio_group.rs: the pressed box must not read the helper directly"
    );
    assert!(
        source.contains(".rounded(crate::util::key_radius(cx))"),
        "radio_group.rs: the selected dot is an inner part and keeps its own"
    );
}

// The `Code` chip is the only kind that paints a box, so the override has to
// sit inside that branch — a heading's `radius` would otherwise lie.
#[test]
fn typography_override_sits_inside_the_code_branch() {
    let source = include_str!("../src/typography.rs");
    let mono = source
        .find("if self.kind.is_mono() {")
        .expect("typography.rs: the mono branch must stay");
    let override_at = source
        .find(".rounded(self.radius.unwrap_or_else(|| crate::util::mark_radius(cx)))")
        .expect("typography.rs: the override must replace the helper's value");
    assert!(
        mono < override_at,
        "typography.rs: the override must sit inside the `Code` branch"
    );
    assert!(
        source[override_at + ".rounded(self.radius".len()..]
            .find(".rounded(self.radius")
            .is_none(),
        "typography.rs: only the `Code` chip rounds"
    );
}

// Both size-stepped boxes keep their step as the fallback, and the tag's
// remove button is an inner part that stays a circle.
#[test]
fn badge_and_tag_steps_stay_the_fallback_and_inner_parts_keep_their_own() {
    let badge = include_str!("../src/badge.rs");
    for helper in [
        "crate::util::small_radius(cx),",
        "crate::util::control_radius(cx),",
        "crate::util::soft_radius(cx),",
    ] {
        assert!(
            badge.contains(helper),
            "badge.rs: the size step must stay the fallback ({helper})"
        );
    }
    assert!(
        badge.contains("let radius = self.radius.unwrap_or(radius);"),
        "badge.rs: the override replaces the step's value"
    );

    let tag = include_str!("../src/tag_group.rs");
    assert!(
        tag.contains("Size::Sm | Size::Md => crate::util::small_radius(cx),")
            && tag.contains("Size::Lg => crate::util::soft_radius(cx),"),
        "tag_group.rs: the size step must stay the fallback"
    );
    assert!(
        tag.contains(".rounded_full()"),
        "tag_group.rs: the remove button inside a chip stays a circle"
    );
}
