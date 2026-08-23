"""CheckboxGroup, RadioGroup and ToggleButtonGroup pages."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


# ----------------------------------------------------------- CheckboxGroup
rep("""            crate::pages::Page::CheckboxGroup.import_line(),
            vec![
                (
                    "Vertical",""",
    """            crate::pages::Page::CheckboxGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::CheckboxGroup::new("cbg-usage", group_options())
                        .label("Notifications")
                        .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::CheckboxGroup::new("cbg-surface", group_options())
                                .label("Notifications")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::CheckboxGroup::new("cbg-disabled", group_options())
                        .label("Notifications")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "Indeterminate",
                    col(vec![
                        para(
                            "v3 pairs a \\"select all\\" checkbox with the group: it is \\
                             indeterminate while only some children are selected.",
                            cx,
                        ),
                        h::Checkbox::new("cbg-all")
                            .is_selected(selected.len() == 3)
                            .is_indeterminate(!selected.is_empty() && selected.len() < 3)
                            .label(gpui::div().child("All notifications"))
                            .into_any_element(),
                        h::CheckboxGroup::new("cbg-ind", group_options())
                            .value(selected.iter().cloned())
                            .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                                this.checkbox_group = keys.clone();
                                cx.notify();
                            }))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Validation",
                    col(vec![h::CheckboxGroup::new("cbg-validate", group_options())
                        .label("Notifications")
                        .is_required(true)
                        .is_invalid(selected.is_empty())
                        .error_message("Pick at least one channel")
                        .value(selected.iter().cloned())
                        .on_change(cx.listener(|this, keys: &HashSet<SharedString>, _, cx| {
                            this.checkbox_group = keys.clone();
                            cx.notify();
                        }))
                        .into_any_element()]),
                ),
                (
                    "Features and Add-ons Example",
                    col(vec![h::CheckboxGroup::new(
                        "cbg-addons",
                        vec![
                            h::CheckboxOption::new("analytics", "Analytics")
                                .description("Usage dashboards and funnels"),
                            h::CheckboxOption::new("backups", "Daily backups")
                                .description("Restore any of the last 30 days"),
                            h::CheckboxOption::new("sso", "Single sign-on")
                                .description("SAML and OIDC"),
                        ],
                    )
                    .label("Add-ons")
                    .description("Billed monthly, cancel any time.")
                    .into_any_element()]),
                ),
                (
                    "With Custom Indicator",
                    col(vec![
                        para(
                            "The group's options carry the same indicator slot a single \\
                             `Checkbox` does; here each one draws a tick of its own.",
                            cx,
                        ),
                        h::Checkbox::new("cbg-ci-1")
                            .default_selected(true)
                            .indicator(move |selected| {
                                if selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::HEART_FILL)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("Email"))
                            .into_any_element(),
                        h::Checkbox::new("cbg-ci-2")
                            .indicator(move |selected| {
                                if selected {
                                    gpui::svg()
                                        .size(px(10.))
                                        .path(h::icons::HEART_FILL)
                                        .text_color(gpui::white())
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            })
                            .label(gpui::div().child("SMS"))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Vertical",""")

# -------------------------------------------------------------- RadioGroup
rep("""    pub fn page_radio_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.radio_sel;
        let options: Vec<SharedString> = vec!["Free".into(), "Pro".into(), "Enterprise".into()];
        doc_page(""",
    """    pub fn page_radio_group(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.radio_sel;
        let options: Vec<SharedString> = vec!["Free".into(), "Pro".into(), "Enterprise".into()];
        let plans = || -> Vec<SharedString> {
            vec!["Free".into(), "Pro".into(), "Enterprise".into()]
        };
        doc_page(""")

rep("""            crate::pages::Page::RadioGroup.import_line(),
            vec![""",
    """            crate::pages::Page::RadioGroup.import_line(),
            vec![
                (
                    "Usage",
                    col(vec![h::RadioGroup::new("rg-usage", plans())
                        .default_value(Some(0))
                        .into_any_element()]),
                ),
                (
                    "Variants",
                    col(vec![
                        h::RadioGroup::new("rg-v-primary", plans())
                            .default_value(Some(0))
                            .into_any_element(),
                        h::RadioGroup::new("rg-v-secondary", plans())
                            .default_value(Some(1))
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::RadioGroup::new("rg-surface", plans())
                                .default_value(Some(0))
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Validation",
                    col(vec![h::RadioGroup::new("rg-validate", plans())
                        .value(None)
                        .is_required(true)
                        .is_invalid(true)
                        .into_any_element()]),
                ),
                (
                    "Delivery & Payment",
                    col(vec![
                        h::RadioGroup::new(
                            "rg-delivery",
                            vec![
                                "Standard — 5 to 7 days".into(),
                                "Express — 2 days".into(),
                                "Overnight".into(),
                            ],
                        )
                        .default_value(Some(0))
                        .into_any_element(),
                        h::Separator::new().into_any_element(),
                        h::RadioGroup::new(
                            "rg-payment",
                            vec!["Card".into(), "Bank transfer".into(), "Invoice".into()],
                        )
                        .default_value(Some(0))
                        .orientation(Orientation::Horizontal)
                        .into_any_element(),
                    ]),
                ),
                (
                    "Custom Indicator",
                    col(vec![
                        para(
                            "v3 replaces `Radio.Indicator`. This port draws v3's own filled \\
                             square; the group below shows it selected in both variants.",
                            cx,
                        ),
                        h::RadioGroup::new("rg-indicator", plans())
                            .default_value(Some(2))
                            .into_any_element(),
                    ]),
                ),""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched checkbox group + radio group')
