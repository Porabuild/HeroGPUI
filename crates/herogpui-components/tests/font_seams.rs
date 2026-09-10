//! Phase 5 field font seam: the mono token stays the default and an explicit
//! `font_family` overrides it on the box only when set.

#[test]
fn time_and_date_fields_keep_mono_and_accept_a_family_override() {
    for (file, source) in [
        ("time_field.rs", include_str!("../src/time_field.rs")),
        (
            "date_picker/field.rs",
            include_str!("../src/date_picker/field.rs"),
        ),
    ] {
        assert!(
            source.contains("self.font_family = Some(family.into());"),
            "{file}: font_family must store the override"
        );
        assert!(
            source.contains(".when_some(self.font_family.clone(), |group, family| {"),
            "{file}: the override must refine the box after the mono default"
        );
        assert!(
            source.contains("MONO_FONT"),
            "{file}: the mono default must stay for callers that set no family"
        );
    }

    let picker = include_str!("../src/color_picker/picker.rs");
    assert!(picker.contains("self.font_family = Some(family.into());"));
    assert!(picker.contains(".unwrap_or_else(|| util::MONO_FONT.into())"));

    let typography = include_str!("../src/typography.rs");
    assert!(typography.contains("self.font_family = Some(family.into());"));
    assert!(typography.contains("if let Some(family) = self.font_family.clone() {"));
    assert!(typography.contains(".font_family(MONO_FONT)"));
}
