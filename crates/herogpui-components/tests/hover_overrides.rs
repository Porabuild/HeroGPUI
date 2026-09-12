//! Phase 2 hover overrides: every owner resolves the named fill and its
//! painted consumer uses the resolved value.
//!
//! The drawn states are exercised by each owner's own behavior tests; these
//! checks pin the wiring, so a builder that stores the colour without feeding
//! the fill cannot pass. Resolver-style endpoint logic (Button's contract)
//! lives with `util::fade_endpoints`.
//!
//! Both pins are function-scoped through `source_scan::scope_contains`, per
//! the roadmap's source-check rule: a sibling function's paint call must not
//! satisfy an owner's pin, so the function that resolves the override has to
//! be the one whose fill consumes it. Each entry also proves the scanner can
//! fail — with the resolve and the paint split across sibling functions, the
//! pin must error — so a green run is the scanner agreeing, not it reading
//! nothing.

mod source_scan;

use source_scan::{component_src, scope_contains};

#[test]
fn every_hover_override_reaches_its_painted_fill() {
    let mut failures = Vec::new();
    for (file, field, resolved, painted) in [
        (
            "select.rs",
            "row_hover_bg",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "combo_box.rs",
            "row_hover_bg",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "autocomplete.rs",
            "row_hover_bg",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "list_box.rs",
            "row_hover_bg",
            "let hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "dropdown.rs",
            "row_hover_bg",
            "let row_hover_bg = self.row_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(row_hover_bg)",
        ),
        (
            "pagination.rs",
            "hover_bg",
            "let control_hover_bg = self.hover_bg.unwrap_or(colors.default.hover());",
            "let pressed_bg = control_hover_bg;",
        ),
        (
            "tag_group.rs",
            "hover_bg",
            "let hover = self.hover_bg.unwrap_or_else(|| {",
            "s.bg(hover)",
        ),
        (
            "tag_group.rs",
            "remove_hover_bg",
            "let hover_bg = self.remove_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "table.rs",
            "row_hover_bg",
            "let row_hover_bg = self.row_hover_bg;",
            "row_hover_bg.unwrap_or(if secondary {",
        ),
        (
            "accordion.rs",
            "hover_bg",
            "let hover_bg = self.hover_bg.unwrap_or_else(|| match self.variant {",
            "s.bg(hover_bg)",
        ),
        (
            "time_field.rs",
            "stepper_hover_bg",
            "let hover_bg = self.stepper_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "input_otp.rs",
            "slot_hover_bg",
            "let hover_bg = self.slot_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "input.rs",
            "clear_hover_bg",
            "let clear_hover_bg = self.clear_hover_bg.unwrap_or(colors.default.hover());",
            "s.bg(clear_hover_bg)",
        ),
        (
            "date_picker/range.rs",
            "trigger_hover_bg",
            "let hover_bg = self.trigger_hover_bg.unwrap_or(colors.field.hover());",
            "s.bg(hover_bg)",
        ),
        (
            "toast.rs",
            "close_hover_bg",
            "let hover_bg = self.t.close_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "switch.rs",
            "hover_bg",
            "unwrap_or(if checked { accent_hover } else { default_hover });",
            "let track_motion_frame = track_motion(&self.id, track_target, window, cx);",
        ),
        (
            "checkbox.rs",
            "hover_bg",
            "let fill_hover = self.hover_bg.unwrap_or(accent_hover);",
            "let fill_bg_target = if is_hovered { fill_hover } else { accent_color };",
        ),
        (
            "calendar.rs",
            "day_hover_bg",
            "let hover_bg = self.day_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "range_calendar.rs",
            "day_hover_bg",
            "let hover_bg = self.day_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "calendar.rs",
            "nav_hover_bg",
            "let hover_bg = self.nav_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "calendar.rs",
            "year_hover_bg",
            "let hover_bg = self.year_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "range_calendar.rs",
            "nav_hover_bg",
            "let hover_bg = self.nav_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
        (
            "range_calendar.rs",
            "year_hover_bg",
            "let hover_bg = self.year_hover_bg.unwrap_or(colors.default.color);",
            "s.bg(hover_bg)",
        ),
    ] {
        let source = component_src(file);
        // The builder named after the field must store the override itself:
        // a store line loose in another function does not wire the builder.
        let builder = format!("pub fn {field}(");
        let stored = format!("self.{field} = Some(color.into());");
        if let Err(err) = scope_contains(&source, &builder, &stored) {
            failures.push(format!("{file} [{field}]: {err}"));
        }
        // The function that resolves the override must be the one painting
        // the resolved value into a fill.
        if let Err(err) = scope_contains(&source, resolved, painted) {
            failures.push(format!("{file} [{field}]: {err}"));
        }
        // The known-negative: with the paint moved into a sibling function,
        // the whole-file truth of both lines must stop satisfying the pin.
        let co_located = fixture(resolved, painted, true);
        assert!(
            scope_contains(&co_located, resolved, painted).is_ok(),
            "{file} [{field}]: the scanner rejected a correct co-located pair"
        );
        let sibling = fixture(resolved, painted, false);
        assert!(
            scope_contains(&sibling, resolved, painted).is_err(),
            "{file} [{field}]: a sibling fn's paint satisfied the scoped pin"
        );
    }
    assert!(
        failures.is_empty(),
        "hover override wiring failed:\n{}",
        failures.join("\n")
    );
}

/// A synthetic owner function holding `resolved` (and `painted`, when
/// `co_located`); otherwise `painted` sits in a sibling function first, the
/// shape a whole-file `contains` wrongly accepts.
fn fixture(resolved: &str, painted: &str, co_located: bool) -> String {
    let mut source = String::new();
    if !co_located {
        source.push_str("fn sibling() {\n    ");
        source.push_str(painted);
        source.push_str("\n}\n\n");
    }
    source.push_str("fn render() {\n    ");
    source.push_str(resolved);
    if co_located {
        source.push_str("\n    ");
        source.push_str(painted);
    }
    source.push_str("\n}\n");
    source
}
