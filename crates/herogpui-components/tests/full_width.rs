//! Phase 6 `full_width` seams: each container stores the flag and expands its
//! root, without redistributing the children.

#[test]
fn full_width_builders_reach_their_roots() {
    for (file, source, applied) in [
        (
            "tag_group.rs",
            include_str!("../src/tag_group.rs"),
            "root = root.w_full();",
        ),
        (
            "pagination.rs",
            include_str!("../src/pagination.rs"),
            "el = el.w_full();",
        ),
        (
            "breadcrumbs.rs",
            include_str!("../src/breadcrumbs.rs"),
            "el = el.w_full();",
        ),
        (
            "radio_group.rs",
            include_str!("../src/radio_group.rs"),
            "root = root.w_full();",
        ),
        (
            "toolbar.rs",
            include_str!("../src/toolbar.rs"),
            "if self.full_width { el.w_full() } else { el }",
        ),
    ] {
        assert!(
            source.contains("self.full_width = v;"),
            "{file}: the builder must store the flag"
        );
        assert!(
            source.contains("if self.full_width {"),
            "{file}: the root must read the flag"
        );
        assert!(
            source.contains(applied),
            "{file}: the root must expand with w_full"
        );
    }
}
