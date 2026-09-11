//! Per-corner `sx` reconciliation for painted parts: an explicit `sx` corner
//! refines a slider track/fill/thumb and a switch track/thumb individually,
//! while an unnamed corner keeps the component's own radius. The per-corner
//! precedence itself is unit-tested on `util::round_sx_corners`.

#[test]
fn slider_and_switch_refine_their_parts_with_sx_corners() {
    for (file, source) in [
        ("slider.rs", include_str!("../src/slider.rs")),
        ("switch.rs", include_str!("../src/switch.rs")),
    ] {
        assert!(
            source.contains("let sx_corners = crate::util::sx_radius(&self.sx);"),
            "{file}: the parts must read the sx corners"
        );
        assert!(
            source.contains("crate::util::round_sx_corners("),
            "{file}: the parts must refine their corners"
        );
    }
    assert_eq!(
        include_str!("../src/slider.rs")
            .matches("crate::util::round_sx_corners(")
            .count(),
        3,
        "the slider track, fill and thumb must each refine their corners"
    );
    assert_eq!(
        include_str!("../src/switch.rs")
            .matches("crate::util::round_sx_corners(")
            .count(),
        3,
        "the switch track, animated fill and thumb must each refine their corners"
    );
    let tabs = include_str!("../src/tabs.rs");
    assert!(
        tabs.contains("let sx_corners = crate::util::sx_radius(&self.sx);")
            && tabs.contains("indicator = crate::util::round_sx_corners(indicator, &sx_corners);"),
        "the Tabs indicator must refine its corners"
    );
}
