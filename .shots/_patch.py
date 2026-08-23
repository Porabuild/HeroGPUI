"""Disclosure, DatePicker and DateRangePicker pages."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# -------------------------------------------------------------- Disclosure
rep("""            crate::pages::Page::Disclosure.import_line(),
            vec![
                (
                    "Single",""",
    """            crate::pages::Page::Disclosure.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::Disclosure::new("Shipping details")
                        .child(gpui::div().child("Ships in 2-4 business days."))
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::DisclosureGroup::new()
                            .item("returns", "Returns", gpui::div().child("Thirty days."))
                            .item("warranty", "Warranty", gpui::div().child("Two years."))
                            .expanded_keys(group.clone())
                            .on_toggle(cx.listener(|this, key: &SharedString, _, cx| {
                                toggle_key(&mut this.disclosure_group_expanded, key);
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("{} expanded", group.len()), cx),
                    ]),
                ),
                (
                    "Single",""")

# -------------------------------------------------------------- DatePicker
rep("""            crate::pages::Page::DatePicker.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::DatePicker.import_line(),
            vec![
                (
                    "Disabled",
                    col(vec![h::DatePicker::new(self.demo_calendar("dp-disabled", cx))
                        .label("Date")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::DatePicker::new(self.demo_calendar("dp-controlled", cx))
                            .label("Date")
                            .on_change(opt_date_cb(cx.listener(
                                |this, d: &Option<h::Date>, _, cx| {
                                    this.cal_picked = *d;
                                    cx.notify();
                                },
                            )))
                            .into_any_element(),
                        para(
                            &match self.cal_picked {
                                Some(d) => format!("Value: {}", d.format_iso()),
                                None => "No value".to_owned(),
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Validation",
                    col(vec![h::DatePicker::new(self.demo_calendar("dp-invalid", cx))
                        .label("Date")
                        .is_required(true)
                        .is_invalid(true)
                        .into_any_element()]),
                ),
                (
                    "Format Options",
                    col(vec![
                        para(
                            "The trigger shows the date in the ISO order this port formats in; \\
                             `locale` is what v3 varies it with, and that needs CLDR data.",
                            cx,
                        ),
                        h::DatePicker::new(self.demo_calendar("dp-format", cx))
                            .label("Date")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![h::Form::new()
                        .child(
                            h::DatePicker::new(self.demo_calendar("dp-form", cx))
                                .label("Start date")
                                .is_required(true),
                        )
                        .child(h::Button::new("dp-form-submit").label("Save"))
                        .into_any_element()]),
                ),
                (
                    "Custom Indicator",
                    col(vec![
                        para(
                            "v3 replaces the trigger's calendar glyph. The chevron here turns \\
                             with the panel, which is the same affordance.",
                            cx,
                        ),
                        h::DatePicker::new(self.demo_calendar("dp-indicator", cx))
                            .label("Date")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",""")

# --------------------------------------------------------- DateRangePicker
rep("""            crate::pages::Page::DateRangePicker.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::DateRangePicker.import_line(),
            vec![
                (
                    "Disabled",
                    col(vec![h::DateRangePicker::new(self.demo_range("drp-disabled", cx))
                        .label("Stay")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        para("The range lives in the state entity the caller owns.", cx),
                        h::DateRangePicker::new(self.date_range.clone())
                            .label("Stay")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Validation",
                    col(vec![h::DateRangePicker::new(self.demo_range("drp-invalid", cx))
                        .label("Stay")
                        .is_required(true)
                        .is_invalid(true)
                        .into_any_element()]),
                ),
                (
                    "Format Options",
                    col(vec![
                        para(
                            "Both ends are shown in the ISO order this port formats in; `locale` \\
                             is what v3 varies it with, and that needs CLDR data.",
                            cx,
                        ),
                        h::DateRangePicker::new(self.demo_range("drp-format", cx))
                            .label("Stay")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Form Example",
                    col(vec![h::Form::new()
                        .child(
                            h::DateRangePicker::new(self.demo_range("drp-form", cx))
                                .label("Stay")
                                .is_required(true),
                        )
                        .child(h::Button::new("drp-form-submit").label("Book"))
                        .into_any_element()]),
                ),
                (
                    "Custom Indicator",
                    col(vec![
                        para(
                            "v3 replaces the trigger's calendar glyph; the chevron here turns \\
                             with the panel instead.",
                            cx,
                        ),
                        h::DateRangePicker::new(self.demo_range("drp-indicator", cx))
                            .label("Stay")
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched disclosure + pickers')
