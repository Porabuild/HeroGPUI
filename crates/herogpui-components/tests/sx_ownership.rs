//! The part-scoped `sx` ownership inventory.
//!
//! An `sx` override is resolved per painted part and per property, not once
//! for a whole component: a tab-list background is not a selected-tab
//! background, and a slider track color is not its value fill. Each entry
//! below names the source function that owns one painted part or derived
//! metric and the extractor it must read once wired — `util::sx_background`,
//! `util::sx_padding` or `util::sx_radius`.
//!
//! Entries marked pending are the known unconverted consumers; the marker
//! must still exist, so deleting or renaming a consumer fails the test rather
//! than silently shrinking the inventory. Wiring an entry means flipping it to
//! `Part::wired` in the same change. Scoping is to the enclosing function, so
//! a sibling's extractor call never satisfies an entry (see the fixtures at
//! the bottom).

mod source_scan;

use source_scan::{component_src, scope_contains};

struct Part {
    file: &'static str,
    part: &'static str,
    marker: &'static str,
    reader: &'static str,
    wired: bool,
}

impl Part {
    const fn wired(
        file: &'static str,
        part: &'static str,
        marker: &'static str,
        reader: &'static str,
    ) -> Self {
        Self {
            file,
            part,
            marker,
            reader,
            wired: true,
        }
    }

    const fn pending(
        file: &'static str,
        part: &'static str,
        marker: &'static str,
        reader: &'static str,
    ) -> Self {
        Self {
            file,
            part,
            marker,
            reader,
            wired: false,
        }
    }
}

const INVENTORY: &[Part] = &[
    // Wired: Button freezes both hover-fade endpoints on the resolved resting
    // background, and the `sx` background is the resting value.
    Part::wired(
        "button.rs",
        "fill and hover-fade endpoints",
        "crate::anim::hover_fade(",
        "util::sx_background",
    ),
    // Pending conversions. `sx_background` is the intended extractor; the
    // consumer's own state precedence is settled in the same change.
    Part::pending(
        "toggle_button.rs",
        "fill and hover-fade endpoints",
        "crate::anim::hover_fade(",
        "util::sx_background",
    ),
    Part::pending(
        "close_button.rs",
        "fill and hover-fade endpoints",
        "crate::anim::hover_fade(",
        "util::sx_background",
    ),
    Part::pending(
        "switch.rs",
        "track hover fill",
        ".child(track_motion_frame.render(",
        "util::sx_background",
    ),
    Part::pending(
        "checkbox.rs",
        "control fill fade",
        "\"fill-fade\"",
        "util::sx_background",
    ),
    Part::pending(
        "select.rs",
        "trigger hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "list_box.rs",
        "row hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "pagination.rs",
        "control hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "accordion.rs",
        "header hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "calendar.rs",
        "day hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "range_calendar.rs",
        "day hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "combo_box.rs",
        "row hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "autocomplete.rs",
        "clear-button hover fill",
        ".hover(move |st| st.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "dropdown.rs",
        "row hover fill",
        "row = row.hover(move |s| s.bg(colors.default.color));",
        "util::sx_background",
    ),
    Part::pending(
        "tag_group.rs",
        "tag hover fill",
        ".hover(move |s| s.bg(hover))",
        "util::sx_background",
    ),
    Part::pending(
        "time_field.rs",
        "stepper hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "input_otp.rs",
        "slot hover fill",
        "cell = cell.hover(move |s| s.bg(hover_bg));",
        "util::sx_background",
    ),
    Part::pending(
        "input_group.rs",
        "group hover fill",
        ".hover(move |style| style.bg(hover_bg)",
        "util::sx_background",
    ),
    Part::pending(
        "number_field.rs",
        "group hover fill",
        ".hover(move |style| style.bg(hover_bg)",
        "util::sx_background",
    ),
    Part::pending(
        "input.rs",
        "clear-button hover fill",
        ".hover(move |s| s.bg(clear_hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "date_picker/range.rs",
        "trigger hover fill",
        ".hover(move |s| s.bg(hover_bg))",
        "util::sx_background",
    ),
    Part::pending(
        "toast.rs",
        "close-button hover fill",
        "close_btn = close_btn.hover(move |s| s.bg(hover_bg));",
        "util::sx_background",
    ),
];

#[test]
fn every_inventoried_part_owns_its_extractor_once_wired() {
    let mut failures = Vec::new();
    let mut pending = Vec::new();
    for part in INVENTORY {
        let source = component_src(part.file);
        if !source.contains(part.marker) {
            failures.push(format!(
                "{} [{}]: marker {:?} is gone — the consumer was removed or renamed; \
                 update or delete this inventory entry",
                part.file, part.part, part.marker
            ));
            continue;
        }
        if part.wired {
            if let Err(err) = scope_contains(&source, part.marker, part.reader) {
                failures.push(format!("{} [{}]: {err}", part.file, part.part));
            }
        } else {
            pending.push(format!("{} [{}] -> {}", part.file, part.part, part.reader));
        }
    }
    assert!(
        failures.is_empty(),
        "sx ownership wiring failed:\n{}",
        failures.join("\n")
    );
    eprintln!(
        "pending sx ownership entries ({}):\n{}",
        pending.len(),
        pending.join("\n")
    );
}

/// A sibling's extractor call must not satisfy a consumer's entry.
#[test]
fn the_scanner_rejects_a_sibling_extractor_call() {
    let source = "\
fn owner() {
    paint();
}

fn sibling() {
    let bg = util::sx_background(&sx);
    paint_sibling(bg);
}
";
    assert!(
        scope_contains(source, "paint();", "util::sx_background").is_err(),
        "the sibling call must not satisfy the owner"
    );
    assert!(
        scope_contains(
            source,
            "let bg = util::sx_background",
            "util::sx_background"
        )
        .is_ok(),
        "the owning call must satisfy its own entry"
    );
}

/// A removed consumer must fail loudly instead of passing on an empty scope.
#[test]
fn the_scanner_requires_the_marker_to_exist() {
    let source = "fn owner() {\n    let bg = util::sx_background(&sx);\n    paint(bg);\n}\n";
    assert!(
        scope_contains(source, "removed_consumer", "util::sx_background").is_err(),
        "a missing marker is a failure, not an empty pass"
    );
}

/// The scope is the enclosing function, including a marker that is itself the
/// function signature.
#[test]
fn the_scanner_scopes_to_the_enclosing_function() {
    let good = "fn owner() {\n    let bg = util::sx_background(&sx);\n    paint(bg);\n}\n";
    assert!(scope_contains(good, "paint(bg)", "util::sx_background").is_ok());

    let bad = "fn owner() {\n    paint();\n}\n\nfn helper() {\n    util::sx_background(&sx);\n}\n";
    assert!(scope_contains(bad, "paint();", "util::sx_background").is_err());

    let signature = "fn owner() {\n    util::sx_background(&sx);\n}\n";
    assert!(scope_contains(signature, "fn owner()", "util::sx_background").is_ok());
}
