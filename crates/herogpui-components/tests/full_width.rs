//! Phase 6 `full_width` seams: each container stores the flag and expands its
//! root, without redistributing the children.
//!
//! Measured, not read from source: each container sits at the start of a
//! 400px flex row followed by a 10px probe. A root that hugs its content
//! leaves the probe right after that content; a root that took `w_full`
//! claims the row, which pushes the probe to its far end. The first child
//! stays where it was either way, since the flag widens the root only.

mod harness;

use gpui::{prelude::*, px, AnyElement, TestAppContext};
use harness::{open_host, probe, settle, still};
use herogpui_components::{
    Breadcrumbs, Button, Crumb, Pagination, RadioGroup, RadioOption, Tag, TagGroup, Toolbar,
};

const ROW: f32 = 400.;

type Build = fn(bool) -> AnyElement;

fn cases() -> Vec<(&'static str, Build)> {
    vec![
        ("TagGroup", |full| {
            TagGroup::new("tg", vec![Tag::new("a", "Alpha")])
                .full_width(full)
                .into_any_element()
        }),
        ("Pagination", |full| {
            Pagination::new("pg", 1, 3)
                .full_width(full)
                .into_any_element()
        }),
        ("Breadcrumbs", |full| {
            Breadcrumbs::new(vec![Crumb::new("Home"), Crumb::new("Docs")])
                .full_width(full)
                .into_any_element()
        }),
        ("RadioGroup", |full| {
            RadioGroup::new("rg", vec![RadioOption::new("One")])
                .full_width(full)
                .into_any_element()
        }),
        ("Toolbar", |full| {
            Toolbar::new()
                .id("tb")
                .full_width(full)
                .child(Button::new("b").label("Bold").into_any_element())
                .into_any_element()
        }),
    ]
}

/// The probe's left edge after one render of `build(full)`.
fn probe_x(cx: &mut TestAppContext, build: Build, full: bool) -> f32 {
    still();
    let vcx = open_host(cx, move || {
        gpui::div()
            .flex()
            .flex_row()
            .items_start()
            .w(px(ROW))
            .child(build(full))
            .child(probe("after"))
            .into_any_element()
    });
    settle(vcx);
    let after = vcx
        .debug_bounds("after")
        .expect("the trailing probe must be laid out");
    f32::from(after.origin.x)
}

#[gpui::test]
fn full_width_builders_reach_their_roots(cx: &mut TestAppContext) {
    for (name, build) in cases() {
        let hugging = probe_x(cx, build, false);
        assert!(
            hugging < ROW / 2.,
            "{name}: without the flag the root must hug its content, the \
             probe sat at x={hugging}"
        );
        let full = probe_x(cx, build, true);
        assert!(
            full > ROW - 20.,
            "{name}: `full_width(true)` must expand the root across the row, \
             the probe sat at x={full} (hugging: {hugging})"
        );
    }
}
