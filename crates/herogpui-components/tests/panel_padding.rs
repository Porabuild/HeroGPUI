//! Phase 3 overlay panel padding: an unset override keeps v3's stock insets,
//! and a set value resolves both axes and feeds the same padding to the
//! resting panel and its entry-animation `ZoomBox`.

#[test]
fn overlay_padding_resolves_both_axes_and_feeds_the_zoom_box() {
    let popover = include_str!("../src/popover.rs");
    assert!(
        popover.contains("self.padding = Some(padding.into());"),
        "Popover::padding must store the override"
    );
    assert!(popover.contains("let panel_padding_y = self.padding.unwrap_or(px(16.));"));
    assert!(popover.contains("let panel_padding_x = self.padding.unwrap_or(px(16.));"));
    assert!(popover.contains("ZoomBox::panel(panel_padding_y,"));
    assert!(popover.contains(".padding_x(panel_padding_x)"));

    let toast = include_str!("../src/toast.rs");
    assert!(
        toast.contains("self.padding = Some(padding.into());"),
        "Toast::padding must store the override"
    );
    assert!(toast.contains("let panel_padding_y = self.t.padding.unwrap_or(px(12.));"));
    assert!(toast.contains("let panel_padding_x = self.t.padding.unwrap_or(px(16.));"));
    assert!(toast.contains("ZoomBox::panel(panel_padding_y,"));
    assert!(toast.contains(".padding_x(panel_padding_x)"));
}
