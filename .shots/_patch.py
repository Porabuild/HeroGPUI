"""ComboBox page: the twenty-one v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            crate::pages::Page::ComboBox.import_line(),
            vec![
                (
                    "Usage",""",
    """            crate::pages::Page::ComboBox.import_line(),
            vec![
                (
                    "Full Width",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-full", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .full_width(true)
                    .into_any_element()]),
                ),
                (
                    "With Description",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-desc", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .description("Pick from the list or type your own")
                    .into_any_element()]),
                ),
                (
                    "Required",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-required", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled", "Rust", cx),
                        languages(),
                    )
                    .label("Language")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "Read Only",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-readonly", "Rust", cx),
                        languages(),
                    )
                    .label("Language")
                    .is_read_only(true)
                    .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::ComboBox::new(self.demo_text("cb-surface", "", cx), languages())
                                .label("Language")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-disabled-opts", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .disabled_keys([SharedString::from("Go")])
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-sections", "", cx),
                        vec![
                            "Rust".into(),
                            "Go".into(),
                            "TypeScript".into(),
                            "Python".into(),
                        ],
                    )
                    .label("Language")
                    .section_before("Rust", "Systems")
                    .section_before("TypeScript", "Scripting")
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::ComboBox::new(self.demo_text("cb-controlled", "", cx), languages())
                            .label("Language")
                            .on_selection_change(cx.listener(
                                |this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("cb-picked", key.to_string());
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &if cb_picked.is_empty() {
                                "Nothing picked yet".to_owned()
                            } else {
                                format!("Picked: {cb_picked}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled Input Value",
                    col(vec![
                        h::ComboBox::new(self.demo_text("cb-input", "", cx), languages())
                            .label("Language")
                            .on_input_change(cx.listener(|this, text: &str, _, cx| {
                                this.set_demo_text_value("cb-typed", text.to_owned());
                                cx.notify();
                            }))
                            .into_any_element(),
                        para(&format!("Typed: {cb_typed}"), cx),
                    ]),
                ),
                (
                    "Controlled Selection",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-ctl-sel", "", cx),
                        languages(),
                    )
                    .label("Languages")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(cb_multi.iter().cloned())
                    .on_selection_change_all(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.set_demo_selection("cb-multi", keys.to_vec());
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Multiple Selection",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-multi-sel", "", cx),
                        languages(),
                    )
                    .label("Languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Default Selected Key",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-default-key", "TypeScript", cx),
                        languages(),
                    )
                    .label("Language")
                    .into_any_element()]),
                ),
                (
                    "Allows Custom Value",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-custom", "Zig", cx),
                        languages(),
                    )
                    .label("Language")
                    .allows_custom_value(true)
                    .into_any_element()]),
                ),
                (
                    "Asynchronous Loading",
                    col(vec![
                        para(
                            "v3 fills the list from a request. The spinner beside the field is \\
                             what says one is in flight; the options are the caller's own data.",
                            cx,
                        ),
                        row(vec![
                            h::ComboBox::new(self.demo_text("cb-async", "", cx), languages())
                                .label("Language")
                                .into_any_element(),
                            h::Spinner::new("cb-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-indicator", "", cx),
                        languages(),
                    )
                    .label("Languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .indicator(|is_selected| {
                        gpui::div()
                            .text_size(px(12.))
                            .child(if is_selected { "\\u{2714}" } else { "" })
                            .into_any_element()
                    })
                    .into_any_element()]),
                ),
                (
                    "Custom Filtering",
                    col(vec![
                        para("`defaultFilter` here matches on the start of the name only.", cx),
                        h::ComboBox::new(self.demo_text("cb-filter", "", cx), languages())
                            .label("Language")
                            .filter(|query, item| {
                                item.to_lowercase().starts_with(&query.to_lowercase())
                            })
                            .default_open(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Menu Trigger",
                    col(vec![
                        spec(
                            "Input (opens as you type)",
                            h::ComboBox::new(self.demo_text("cb-mt-input", "", cx), languages())
                                .label("Language")
                                .menu_trigger(h::MenuTrigger::Input),
                            cx,
                        ),
                        spec(
                            "Manual (only the chevron opens it)",
                            h::ComboBox::new(self.demo_text("cb-mt-manual", "", cx), languages())
                                .label("Language")
                                .menu_trigger(h::MenuTrigger::Manual),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Form Value",
                    col(vec![
                        para(
                            "A ComboBox item *is* its text here -- the list is a `Vec<SharedString>` \\
                             -- so the key and the label are the same value and there is nothing \\
                             for v3's `formValue` to choose between. The field submits the text.",
                            cx,
                        ),
                        {
                            let state = self.demo_text("cb-form", "", cx);
                            let field = h::ComboBox::new(state.clone(), languages())
                                .label("Language")
                                .name("language")
                                .is_required(true);
                            h::Form::new()
                                .field(h::FormField::text(state).name("language").is_required(true))
                                .child(field)
                                .child(h::Button::new("cb-form-submit").label("Save"))
                                .into_any_element()
                        },
                    ]),
                ),
                (
                    "Validation Behavior",
                    col(vec![
                        spec(
                            "Native (blocks the submit)",
                            h::ComboBox::new(self.demo_text("cb-vb-native", "", cx), languages())
                                .label("Language")
                                .is_required(true)
                                .validation_behavior(h::ValidationBehavior::Native),
                            cx,
                        ),
                        spec(
                            "Allow (shows the message, submits anyway)",
                            h::ComboBox::new(self.demo_text("cb-vb-allow", "", cx), languages())
                                .label("Language")
                                .is_required(true)
                                .validation_behavior(h::ValidationBehavior::Allow),
                            cx,
                        ),
                    ]),
                ),
                (
                    "Custom Validation",
                    col(vec![h::ComboBox::new(
                        self.demo_text("cb-validate", "Zig", cx),
                        languages(),
                    )
                    .label("Language")
                    .allows_custom_value(true)
                    .validate(|value| {
                        (!value.is_empty() && !languages().iter().any(|l| l == value))
                            .then(|| "Pick one of the listed languages".into())
                    })
                    .into_any_element()]),
                ),
                (
                    "Usage",""")

rep("""    pub fn page_combo_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.combo_open;""",
    """    pub fn page_combo_box(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let is_open = self.combo_open;
        let cb_picked = self.demo_text_value("cb-picked");
        let cb_typed = self.demo_text_value("cb-typed");
        let cb_multi = self.demo_selection("cb-multi");""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched combo box page')
