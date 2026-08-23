"""ToggleButtonGroup page sections."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""                (
                    "Vertical & detached",""",
    """                (
                    "Orientation",
                    row(vec![
                        h::ToggleButtonGroup::new()
                            .child_toggle(h::ToggleButton::new("tbo-h-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-h-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-h-3").label("Month"))
                            .into_any_element(),
                        h::ToggleButtonGroup::new()
                            .orientation(h::SelectionOrientation::Vertical)
                            .child_toggle(h::ToggleButton::new("tbo-v-1").label("Day"))
                            .child_toggle(h::ToggleButton::new("tbo-v-2").label("Week"))
                            .child_toggle(h::ToggleButton::new("tbo-v-3").label("Month"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Full Width",
                    col(vec![gpui::div()
                        .w_full()
                        .child(
                            h::ToggleButtonGroup::new()
                                .full_width(true)
                                .child_toggle(h::ToggleButton::new("tbf-1").label("Left"))
                                .child_toggle(h::ToggleButton::new("tbf-2").label("Center"))
                                .child_toggle(h::ToggleButton::new("tbf-3").label("Right")),
                        )
                        .into_any_element()]),
                ),
                (
                    "Without Separator",
                    row(vec![h::ToggleButtonGroup::new()
                        .separators(false)
                        .child_toggle(h::ToggleButton::new("tbn-1").label("One"))
                        .child_toggle(h::ToggleButton::new("tbn-2").label("Two"))
                        .child_toggle(h::ToggleButton::new("tbn-3").label("Three"))
                        .into_any_element()]),
                ),
                (
                    "Selection Mode",
                    col(vec![
                        para("Single: exactly one member stays selected.", cx),
                        h::ToggleButtonGroup::new()
                            .selection_mode(SelectionMode::Single)
                            .child_toggle(h::ToggleButton::new("tbsm-s-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-s-3").key("c").label("C"))
                            .into_any_element(),
                        para("Multiple: any number of members can be selected.", cx),
                        h::ToggleButtonGroup::new()
                            .selection_mode(SelectionMode::Multiple)
                            .child_toggle(h::ToggleButton::new("tbsm-m-1").key("a").label("A"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-2").key("b").label("B"))
                            .child_toggle(h::ToggleButton::new("tbsm-m-3").key("c").label("C"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical & detached",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched toggle button group page')
