//! Phase 2 hover overrides: every owner resolves the named fill and its
//! painted consumer uses the resolved value.
//!
//! The field-trigger seams (`Select`/`Autocomplete::trigger_hover_bg`,
//! `Autocomplete::clear_hover_bg`, `NumberField`/`InputGroup::group_hover_bg`)
//! are checked on the painted scene instead, at the bottom of this file: the
//! pointer hovers the part, the fade settles on the real clock and the frame
//! must hold the override (and a disabled trigger must not).
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

mod harness;
mod source_scan;

use gpui::{point, prelude::*, px, AnyElement, Modifiers, TestAppContext, VisualTestContext};
use herogpui_components::{
    Autocomplete, Input, InputGroup, InputState, NumberField, NumberState, PickerItem, Select,
};
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
            "hover_fade_with_duration_and_easing(",
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
            "let hover_bg = (!is_selected && !is_disabled).then(|| {",
            "row_hover_bg.unwrap_or(if self.secondary {",
        ),
        (
            "accordion.rs",
            "hover_bg",
            "let hover_bg = self.hover_bg.unwrap_or_else(|| match self.variant {",
            "(idle_bg, hover_bg),",
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
            "let hover_bg = self.slot_hover_bg.unwrap_or(match self.variant {",
            // `.input-otp__slot` hovers through the pinned 150ms shell
            // transition, so the override resolves into the chrome ramp's
            // hovered endpoint rather than an immediate fill.
            "bg: hovered_bg,",
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
            // `.calendar__nav-button` hovers through the pinned 100ms
            // background transition, so the override resolves into the
            // fade's hover endpoint rather than an immediate fill.
            "colors.default.color, 0.0), hover_bg),",
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
            "colors.default.color, 0.0), hover_bg),",
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

// Painted-scene checks for the field-trigger hover seams: the pointer is
// moved onto the control and the 150ms fade is allowed to settle on the real
// clock, then the frame's quads must hold the override colour. The colour is
// one no theme token uses, so finding it proves the seam reached the fill.

const OVERRIDE: gpui::Hsla = gpui::Hsla {
    h: 0.83,
    s: 0.7,
    l: 0.42,
    a: 1.0,
};

fn picker_items() -> Vec<PickerItem> {
    ["Rust", "Go"]
        .iter()
        .map(|l| PickerItem::new(l.to_string(), l.to_string()))
        .collect()
}

fn paints_override(cx: &mut VisualTestContext) -> bool {
    harness::has_color(&harness::painted(cx).solids(), OVERRIDE)
}

/// Polls the real clock for up to two seconds until `want` holds: gpui runs
/// the fade on wall time, so a loaded machine can take several frames.
fn eventually(cx: &mut VisualTestContext, want: bool) -> bool {
    for _ in 0..20 {
        if paints_override(cx) == want {
            return true;
        }
        harness::wait_real(cx, 100);
    }
    paints_override(cx) == want
}

/// Whether hovering (`x`, `y`) settles on a frame that paints `OVERRIDE`.
fn hover_paints_override(cx: &mut VisualTestContext, x: f32, y: f32) -> bool {
    // The first frame is laid out with the pointer at the origin, before the
    // host parks it outside; that crossing's fade runs out first, and the
    // override is a hover endpoint only, so nothing paints it at rest.
    assert!(
        eventually(cx, false),
        "the override is a hover endpoint only; nothing paints it at rest"
    );
    cx.simulate_mouse_move(point(px(x), px(y)), None, Modifiers::none());
    eventually(cx, true)
}

fn open(
    cx: &mut TestAppContext,
    build: impl Fn() -> AnyElement + 'static,
) -> &mut VisualTestContext {
    harness::open_host(cx, build)
}

#[gpui::test]
fn select_trigger_hover_bg_is_the_trigger_hover_endpoint(cx: &mut TestAppContext) {
    let cx = open(cx, || {
        Select::new("hover-select", picker_items())
            .trigger_hover_bg(OVERRIDE)
            .into_any_element()
    });
    assert!(hover_paints_override(cx, 20., 18.));
}

#[gpui::test]
fn a_disabled_select_never_paints_its_trigger_hover_bg(cx: &mut TestAppContext) {
    let cx = open(cx, || {
        Select::new("hover-select-off", picker_items())
            .trigger_hover_bg(OVERRIDE)
            .is_disabled(true)
            .into_any_element()
    });
    assert!(
        !hover_paints_override(cx, 20., 18.),
        "disabled outranks the hover override"
    );
}

#[gpui::test]
fn autocomplete_trigger_hover_bg_is_the_trigger_hover_endpoint(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open(cx, move || {
        Autocomplete::new(state.clone(), picker_items())
            .trigger_hover_bg(OVERRIDE)
            .into_any_element()
    });
    assert!(hover_paints_override(cx, 20., 18.));
}

#[gpui::test]
fn autocomplete_clear_hover_bg_fills_the_hovered_clear_button(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let base = format!("autocomplete-{}", state.entity_id().as_u64());
    let cx = open(cx, move || {
        Autocomplete::new(state.clone(), picker_items())
            .default_value(["Rust"])
            .clear_hover_bg(OVERRIDE)
            .into_any_element()
    });
    cx.update(|window, _| window.refresh());
    let clear: &'static str = Box::leak(format!("{base}-clear-visual").into_boxed_str());
    let bounds = cx
        .debug_bounds(clear)
        .expect("a selected Autocomplete renders its clear button");
    let centre = bounds.center();
    assert!(hover_paints_override(
        cx,
        f32::from(centre.x),
        f32::from(centre.y)
    ));
}

#[gpui::test]
fn number_field_group_hover_bg_is_the_group_hover_endpoint(cx: &mut TestAppContext) {
    let state = cx.new(|cx| NumberState::new(cx, 4.));
    let cx = open(cx, move || {
        NumberField::new(state.clone())
            .group_hover_bg(OVERRIDE)
            .into_any_element()
    });
    assert!(hover_paints_override(cx, 20., 18.));
}

#[gpui::test]
fn input_group_group_hover_bg_is_the_group_hover_endpoint(cx: &mut TestAppContext) {
    let state = cx.new(|cx| InputState::new(cx));
    let cx = open(cx, move || {
        InputGroup::new()
            .group_hover_bg(OVERRIDE)
            .input(Input::new(state.clone()))
            .into_any_element()
    });
    assert!(hover_paints_override(cx, 20., 18.));
}
