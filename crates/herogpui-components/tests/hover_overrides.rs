//! Phase 2 hover overrides: every owner resolves the named fill and its
//! painted consumer uses the resolved value.
//!
//! The drawn states are exercised by each owner's own behavior tests; these
//! checks pin the wiring, so a builder that stores the colour without feeding
//! the fill cannot pass. Resolver-style endpoint logic (Button's contract)
//! lives with `util::fade_endpoints`.

#[test]
fn every_hover_override_reaches_its_painted_fill() {
    for (file, source, stored, resolved, painted) in [
        (
            "select.rs",
            include_str!("../src/select.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "combo_box.rs",
            include_str!("../src/combo_box.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "autocomplete.rs",
            include_str!("../src/autocomplete.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "list_box.rs",
            include_str!("../src/list_box.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "dropdown.rs",
            include_str!("../src/dropdown.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "pagination.rs",
            include_str!("../src/pagination.rs"),
            "self.hover_bg = Some(color.into());",
            "let control_hover_bg = self.hover_bg.unwrap_or(colors.default.hover());",
            "let pressed_bg = control_hover_bg;",
        ),
        (
            "tag_group.rs",
            include_str!("../src/tag_group.rs"),
            "self.hover_bg = Some(color.into());",
            "let hover = self.hover_bg.unwrap_or_else(|| {",
            "s.bg(hover)",
        ),
        (
            "tag_group.rs",
            include_str!("../src/tag_group.rs"),
            "self.remove_hover_bg = Some(color.into());",
            "let hover_bg = self.remove_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "table.rs",
            include_str!("../src/table.rs"),
            "self.row_hover_bg = Some(color.into());",
            "let row_hover_bg = self.row_hover_bg;",
            "row_hover_bg.unwrap_or(if secondary {",
        ),
        (
            "accordion.rs",
            include_str!("../src/accordion.rs"),
            "self.hover_bg = Some(color.into());",
            "let hover_bg = self.hover_bg.unwrap_or_else(|| match self.variant {",
            "s.bg(hover_bg)",
        ),
        (
            "time_field.rs",
            include_str!("../src/time_field.rs"),
            "self.stepper_hover_bg = Some(color.into());",
            "let hover_bg = self.stepper_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "input_otp.rs",
            include_str!("../src/input_otp.rs"),
            "self.slot_hover_bg = Some(color.into());",
            "let hover_bg = self.slot_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "input.rs",
            include_str!("../src/input.rs"),
            "self.clear_hover_bg = Some(color.into());",
            "let clear_hover_bg = self.clear_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(clear_hover_bg)",
        ),
        (
            "date_picker/range.rs",
            include_str!("../src/date_picker/range.rs"),
            "self.trigger_hover_bg = Some(color.into());",
            "let hover_bg = self.trigger_hover_bg.unwrap_or(colors.field.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "toast.rs",
            include_str!("../src/toast.rs"),
            "self.close_hover_bg = Some(color.into());",
            "let hover_bg = self.t.close_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
    ] {
        assert!(
            source.contains(stored),
            "{file}: the builder must store the override"
        );
        assert!(
            source.contains(resolved),
            "{file}: the override must feed the resolved fill"
        );
        assert!(
            source.contains(painted),
            "{file}: the painted fill must consume the resolved value"
        );
    }
}
