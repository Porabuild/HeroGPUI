"""Autocomplete page: all sixteen missing v3 examples."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            crate::pages::Page::Autocomplete.import_line(),
            vec![(
                "Usage",""",
    """            crate::pages::Page::Autocomplete.import_line(),
            vec![
                (
                    "Variants",
                    col(vec![
                        h::Autocomplete::new(self.demo_text("ac-primary", "", cx), languages())
                            .label("Primary")
                            .into_any_element(),
                        h::Autocomplete::new(self.demo_text("ac-secondary", "", cx), languages())
                            .label("Secondary")
                            .variant(FieldVariant::Secondary)
                            .into_any_element(),
                    ]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Autocomplete::new(
                                self.demo_text("ac-surface", "", cx),
                                languages(),
                            )
                            .label("Language")
                            .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Full Width",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-full", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .full_width(true)
                    .into_any_element()]),
                ),
                (
                    "With Description",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-desc", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .description("Type to filter the list")
                    .into_any_element()]),
                ),
                (
                    "Required",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-required", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .is_required(true)
                    .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled", "Rust", cx),
                        languages(),
                    )
                    .label("Language")
                    .is_disabled(true)
                    .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-disabled-opts", "", cx),
                        languages(),
                    )
                    .label("Language")
                    .disabled_keys([SharedString::from("Go"), SharedString::from("Python")])
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Allows Empty Collection",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-empty", "zzz", cx),
                        languages(),
                    )
                    .label("Language")
                    .allows_empty_collection(true)
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-sections", "", cx),
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
                    "Multiple Select",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-multi-select", "", cx),
                        languages(),
                    )
                    .label("Languages")
                    .selection_mode(SelectionMode::Multiple)
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "Controlled",
                    col(vec![
                        h::Autocomplete::new(self.demo_text("ac-controlled", "", cx), languages())
                            .label("Language")
                            .on_selection_change(cx.listener(
                                |this, key: &SharedString, _, cx| {
                                    this.set_demo_text_value("ac-picked", key.to_string());
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(
                            &if ac_picked.is_empty() {
                                "Nothing picked yet".to_owned()
                            } else {
                                format!("Picked: {ac_picked}")
                            },
                            cx,
                        ),
                    ]),
                ),
                (
                    "Controlled Multiple",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-ctl-multi", "", cx),
                        languages(),
                    )
                    .label("Languages")
                    .selection_mode(SelectionMode::Multiple)
                    .selected_keys(ac_multi.iter().cloned())
                    .on_selection_change_all(cx.listener(|this, keys: &[SharedString], _, cx| {
                        this.set_demo_selection("ac-multi", keys.to_vec());
                        cx.notify();
                    }))
                    .into_any_element()]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("ac-open-btn")
                                .label(if ac_open { "Close" } else { "Open" })
                                .size(Size::Sm)
                                .variant(Variant::Secondary)
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.set_demo_flag("ac-open", !ac_open);
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if ac_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Autocomplete::new(self.demo_text("ac-open", "", cx), languages())
                            .label("Language")
                            .is_open(ac_open)
                            .on_open_change(bool_cb(cx.listener(|this, v: &bool, _, cx| {
                                this.set_demo_flag("ac-open", *v);
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Filtering",
                    col(vec![
                        para(
                            "v3 fetches the matches as the query changes. `filter` is the hook \\
                             for that -- it decides what counts as a match -- and a spinner \\
                             beside the field says a request is in flight.",
                            cx,
                        ),
                        row(vec![
                            h::Autocomplete::new(self.demo_text("ac-async", "", cx), languages())
                                .label("Language")
                                .filter(|query, item| {
                                    item.to_lowercase().contains(&query.to_lowercase())
                                })
                                .into_any_element(),
                            h::Spinner::new("ac-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    col(vec![h::Autocomplete::new(
                        self.demo_text("ac-indicator", "", cx),
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
                    "Custom Value",
                    col(vec![
                        para(
                            "An `Autocomplete` keeps whatever is typed in its own state, so an \\
                             unmatched query stays put -- which is v3's custom-value behaviour.",
                            cx,
                        ),
                        h::Autocomplete::new(self.demo_text("ac-custom", "Zig", cx), languages())
                            .label("Language")
                            .allows_empty_collection(true)
                            .into_any_element(),
                    ]),
                ),
                (
                    "Usage",""")

rep("""    pub fn page_autocomplete(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {""",
    """    pub fn page_autocomplete(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let ac_picked = self.demo_text_value("ac-picked");
        let ac_multi = self.demo_selection("ac-multi");
        let ac_open = self.demo_flag("ac-open", false);""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched autocomplete page')
