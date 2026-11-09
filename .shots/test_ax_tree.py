"""Unit tests for the pure AX-audit rules in ax_tree.py.

These run on any OS: the rules are pure functions over node dicts and never
touch the accessibility APIs or the gallery binary.
"""
import unittest
from unittest.mock import patch

import ax_tree as ax


def N(role, title="", subrole=None, value=None, children=None, **kw):
    node = {"role": role, "title": title, "subrole": subrole,
            "value": value, "children": children or []}
    node.update(kw)
    return node


def page_tree():
    """A miniature gallery page: header chrome + two examples + traffic lights."""
    return [
        N("AXGroup", children=[
            N("AXLink", title="GitHub"),
            N("AXButton", title="Motion off"),
            N("AXButton"),                      # header icon 1 (unnamed)
            N("AXButton"),                      # header icon 2 (unnamed)
            N("AXButton", title="Copy"),        # first example's code button
            N("AXCheckBox", value=0),           # example checkbox, unnamed
            N("AXCheckBox", value=1),           # example checkbox, unnamed
            N("AXButton", title="Expand code"),
            N("AXButton"),                      # example icon-button, unnamed
        ]),
        N("AXButton", subrole="AXCloseButton"),   # traffic light (system)
        N("AXButton", subrole="AXMinimizeButton"),
        N("AXButton", subrole="AXFullScreenButton"),
    ]


class NameTests(unittest.TestCase):
    def test_title_wins(self):
        self.assertEqual(ax.node_name(N("AXButton", title="Save", description="d")), "Save")

    def test_description_then_title_element(self):
        self.assertEqual(ax.node_name(N("AXButton", description="Does a thing")), "Does a thing")
        self.assertEqual(ax.node_name(N("AXButton", titleElement="Label element")), "Label element")

    def test_placeholder_only_for_text_inputs(self):
        self.assertEqual(ax.node_name(N("AXTextField", placeholder="Your name")), "Your name")
        # A placeholder on a non-input is not a name.
        self.assertEqual(ax.node_name(N("AXButton", placeholder="not a name")), "")

    def test_empty_is_unnamed(self):
        self.assertEqual(ax.node_name(N("AXCheckBox", value=1)), "")


class InteractiveTests(unittest.TestCase):
    def test_roles_and_subroles(self):
        for role in ("AXButton", "AXCheckBox", "AXSlider", "AXTextField", "AXLink"):
            self.assertTrue(ax.is_interactive(N(role, title="x")), role)
        self.assertTrue(ax.is_interactive(N("AXCheckBox", subrole="AXSwitch", title="s")))
        self.assertTrue(ax.is_interactive(N("AXRadioButton", subrole="AXTabButton", title="t")))

    def test_system_subroles_are_not_app_content(self):
        for sub in ("AXCloseButton", "AXMinimizeButton", "AXFullScreenButton"):
            self.assertFalse(ax.is_interactive(N("AXButton", subrole=sub)))

    def test_non_interactive_roles(self):
        self.assertFalse(ax.is_interactive(N("AXStaticText", title="hi")))
        self.assertFalse(ax.is_interactive(N("AXGroup")))


class UnnamedTests(unittest.TestCase):
    def test_unnamed_flags_icons_and_checkboxes_not_chrome(self):
        findings = ax.unnamed_findings(page_tree(), "Checkbox")
        roles = sorted(f["role"] for f in findings)
        # Two header icons + two checkboxes + one example icon-button.
        self.assertEqual(roles, ["AXButton", "AXButton", "AXButton",
                                 "AXCheckBox", "AXCheckBox"])
        # Named chrome (GitHub/Motion/Copy/Expand) and traffic lights absent.
        self.assertFalse(any(f.get("subrole") for f in findings))

    def test_unnamed_records_path_frame_value(self):
        f = ax.unnamed_findings(page_tree(), "P")[0]
        self.assertIn("path", f)
        self.assertIn("frame", f)  # may be None; key must exist
        self.assertIn("value", f)


class StateTests(unittest.TestCase):
    def test_checkbox_values(self):
        for v in (0, 1, 2):
            self.assertEqual(ax.state_violations([N("AXCheckBox", value=v)]), [])
        for v in ("on", 5, None):
            self.assertEqual(len(ax.state_violations([N("AXCheckBox", value=v)])), 1, v)

    def test_switch_subrole_still_a_checkbox(self):
        # AXSwitch-subrole nodes keep role AXCheckBox -> value rule applies.
        self.assertEqual(len(ax.state_violations(
            [N("AXCheckBox", subrole="AXSwitch", value="bad")])) , 1)

    def test_slider_range(self):
        good = N("AXSlider", value=5, minValue=0, maxValue=10)
        self.assertEqual(ax.state_violations([good]), [])
        # A numeric value outside the bounds is still a violation.
        self.assertEqual(len(ax.state_violations(
            [N("AXSlider", value=15, minValue=0, maxValue=10)])), 1)
        self.assertEqual(len(ax.state_violations(
            [N("AXSlider", value=-1, minValue=0, maxValue=10)])), 1)
        # A formatted aria-valuetext string is valid beside a full range.
        self.assertEqual(ax.state_violations(
            [N("AXSlider", value="35%", minValue=0, maxValue=100)]), [])
        self.assertEqual(ax.state_violations(
            [N("AXSlider", value="207°", minValue=0, maxValue=360)]), [])
        self.assertEqual(ax.state_violations(
            [N("AXSlider", value="$1,200.00", minValue=0, maxValue=5000)]), [])
        # ...but not without both bounds.
        self.assertEqual(len(ax.state_violations(
            [N("AXSlider", value="35%", minValue=0)])), 1)
        self.assertEqual(len(ax.state_violations(
            [N("AXSlider", value="35%")])) , 1)
        # A missing value is still a violation.
        self.assertEqual(len(ax.state_violations(
            [N("AXSlider", value=None, minValue=0, maxValue=10)])), 1)
        # Incrementor shares the rule.
        self.assertEqual(len(ax.state_violations(
            [N("AXIncrementor", value=-1, minValue=0, maxValue=10)])), 1)


class ChromeExclusionTests(unittest.TestCase):
    def test_example_interactive_excludes_chrome_keeps_examples(self):
        interactive = ax.example_interactive(page_tree())
        roles = [n.get("role") for n, _p in interactive]
        # The two example checkboxes + the example icon-button remain.
        self.assertEqual(sorted(roles), ["AXButton", "AXCheckBox", "AXCheckBox"])
        # Header icons, GitHub/Motion/Copy/Expand and traffic lights are gone.
        self.assertFalse(any(n.get("title") in ax.CHROME_TITLES for n, _p in interactive))
        self.assertFalse(any(n.get("subrole") in ax.SYSTEM_SUBROLES for n, _p in interactive))

    def test_header_icons_excluded_but_later_unnamed_button_kept(self):
        # The two pre-Copy unnamed buttons are header chrome; the post-Copy one
        # is example content and must survive for R3.
        interactive = ax.example_interactive(page_tree())
        unnamed_buttons = [n for n, _p in interactive
                           if n.get("role") == "AXButton" and not ax.node_name(n)]
        self.assertEqual(len(unnamed_buttons), 1)

    def test_non_interactive_page_has_no_example_controls(self):
        static = [N("AXGroup", children=[N("AXStaticText", title="Just text")])]
        self.assertEqual(ax.example_interactive(static), [])


class AllowlistTests(unittest.TestCase):
    def test_matching_entry_is_allowed(self):
        findings = ax.unnamed_findings(page_tree(), "Checkbox")
        key = ax.unnamed_key("Checkbox", findings[0])
        with patch.dict(ax.EXPECTED_UNNAMED, {key: "known header icon"}):
            kept, stale = ax.split_allowlist(findings, "Checkbox")
        self.assertEqual(len(kept), len(findings) - 1)
        self.assertEqual(stale, [])

    def test_bogus_entry_is_stale(self):
        findings = ax.unnamed_findings(page_tree(), "Checkbox")
        bogus = ("Checkbox", "AXButton", "AXBogus", "null")
        with patch.dict(ax.EXPECTED_UNNAMED, {bogus: "no such node"}):
            kept, stale = ax.split_allowlist(findings, "Checkbox")
        self.assertEqual(len(kept), len(findings))
        self.assertEqual(stale, [bogus])


if __name__ == "__main__":
    unittest.main()
