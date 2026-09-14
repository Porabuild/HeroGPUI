"""Regression cases for gallery instance ownership and controlled-state wiring."""
import unittest
from unittest.mock import patch

import inert_audit as audit

BUILDERS = audit.state_builders()


class InertAuditTests(unittest.TestCase):
    builders = BUILDERS

    def scan(self, source, builders=None):
        return audit.frozen_instances(source, builders or self.builders)[0]

    def test_only_the_controlled_argument_identifies_state(self):
        source = '''
impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (value, own) = util::controlled(window, cx, key,
            self.value.clone(), normalize(self.default_value, self.min, self.max, self.step));
    }
}
impl Slider {
    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }
    pub fn min_value(mut self, value: f32) -> Self {
        self.min = value;
        self
    }
    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = value;
        self
    }
}
'''
        with patch.object(audit, 'list_modules', return_value=['slider.rs']), \
             patch.object(audit, 'read_module', return_value=source):
            self.assertEqual(audit.controlled_builders(), {'Slider': {'value'}})

    def test_nested_sibling_and_string_callbacks_cannot_drive_parent(self):
        source = '''
h::Switch::new("parent").is_selected(true)
    .label(h::Button::new("child").on_press(|_, _, _| {}))
    .description(".on_change(fake)");
h::Switch::new("sibling").is_selected(false).on_change(|_, _, _| {});
'''
        gaps = self.scan(source)
        self.assertEqual(len(gaps), 1)
        self.assertIn('"parent"', gaps[0])

    def test_comments_raw_strings_and_test_modules_are_not_instances(self):
        source = '''
// h::Switch::new("comment").is_selected(true)
let text = r#"h::Switch::new("quoted").is_selected(true)"#;
#[cfg(test)]
mod tests { fn fixture() { h::Switch::new("test").is_selected(true); } }
h::Switch::new("production").default_selected(true);
'''
        self.assertEqual(self.scan(source), [])
        self.assertEqual([row[1] for row in audit.instances(source)], ['"production"'])

    def test_each_controlled_axis_needs_its_own_callback(self):
        source = 'h::Select::new("menu").selected_keys(keys).is_open(true).on_open_change(f)'
        gaps = self.scan(source)
        self.assertEqual(len(gaps), 1)
        self.assertIn('sets selected_keys', gaps[0])
        self.assertEqual(self.scan(source + '.on_selection_change_all(f)'), [])

    def test_slider_constructor_requires_live_value_or_uncontrolled_seed(self):
        prefix = 'h::Slider::new("slider", 0.35).min_value(0.).max_value(1.).step(0.01)'
        self.assertEqual(len(self.scan(prefix)), 1)
        self.assertEqual(len(self.scan(prefix + '.on_drag_end(f)')), 1)
        self.assertEqual(len(self.scan(prefix + '.default_values([])')), 1)
        self.assertEqual(self.scan(prefix + '.value(0.5).default_value(0.35)'), [])
        for suffix in ('.default_value(0.35)', '.default_values([0.35, 0.7])',
                       '.on_change(f)', '.is_disabled(true)', '.values([0.2, 0.8]).on_change_all(f)'):
            with self.subTest(suffix=suffix):
                self.assertEqual(self.scan(prefix + suffix), [])
        self.assertEqual(len(self.scan(prefix + '.values(vs).on_change(f)')), 1)

    def test_callback_aliases_are_owner_scoped(self):
        self.assertEqual(self.scan('h::Accordion::new(items).expanded_keys(keys).on_toggle(f)',
                                   {'Accordion': {'expanded_keys'}}), [])
        self.assertEqual(self.scan('h::ToggleButtonGroup::new("group").selected_keys(keys).on_change(f)',
                                   {'ToggleButtonGroup': {'selected_keys'}}), [])
        self.assertEqual(len(self.scan('h::Select::new("menu").selected_keys(keys).on_change_end(f)')), 1)

    def test_production_range_discovery_and_empty_or_unknown_defaults(self):
        self.assertIn('values', self.builders['Slider'])
        prefix = 'h::Slider::new("range", 0.35)'
        self.assertIn('sets values', self.scan(prefix + '.values([0.2, 0.8])')[0])
        for expression in ('Vec::new()', 'Vec::<f32>::new()', 'std::iter::empty()',
                           'vec![ ]', '[0.5; 0]', '[0.5; count]', 'runtime_values()'):
            with self.subTest(expression=expression):
                self.assertIn('sets value,', self.scan(prefix + f'.default_values({expression})')[0])
                self.assertEqual(self.scan(prefix + f'.default_values({expression}).default_value(0.35)'), [])
        self.assertIn('sets value,', self.scan(prefix + '.values(vs).on_change_all(f)')[0])
        self.assertEqual(self.scan(prefix + '.values(vs).on_change_all(f).on_change(f)'), [])

    def test_selection_callbacks_match_owner_and_mode(self):
        for source in (
            'h::Select::new(items).value(None).on_selection_change_all(f)',
            'h::Select::new(items).selected_keys(keys).selection_mode(SelectionMode::Multiple).on_selection_change(f)',
            'h::Autocomplete::new(state, items).selected_keys(keys).selection_mode(SelectionMode::Multiple).on_change(f)',
            'h::ComboBox::new(state, items).selected_key(key).on_input_change(f)',
        ):
            with self.subTest(source=source):
                self.assertEqual(len(self.scan(source)), 1)
        for source in (
            'h::Select::new(items).value(None).on_change(f)',
            'h::Select::new(items).selected_keys(keys).selection_mode(SelectionMode::Multiple).on_selection_change_all(f)',
            'h::ComboBox::new(state, items).selected_key(key).selection_mode(h::SelectionMode::Single).on_change(f)',
            'h::ComboBox::new(state, items).selected_key(key).on_selection_change_all(f)',
            'h::Autocomplete::new(state, items).selected_keys(keys).on_change(f)',
        ):
            with self.subTest(source=source):
                self.assertEqual(self.scan(source), [])

    def test_disabled_trigger_does_not_excuse_live_overlay_state(self):
        source = 'h::Select::new(items).is_disabled(true).is_open(true)'
        self.assertIn('sets is_open', self.scan(source)[0])
        self.assertEqual(self.scan(source + '.on_open_change(f)'), [])

    def test_read_only_range_still_needs_year_picker_feedback(self):
        source = 'h::RangeCalendar::new(state).value(start, end).is_read_only(true)'
        self.assertEqual(self.scan(source), [])
        self.assertEqual(len(self.scan(source + '.is_year_picker_open(true)')), 1)

    def test_unclosed_source_does_not_pass(self):
        with self.assertRaisesRegex(ValueError, 'unclosed group'):
            self.scan('h::Switch::new("broken").is_selected(true')


if __name__ == '__main__':
    unittest.main()
