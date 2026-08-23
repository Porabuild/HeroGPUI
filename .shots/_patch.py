"""Select page: the eleven v3 examples it was missing."""
import io

P = 'gallery/src/pages/components.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""            crate::pages::Page::Select.import_line(),
            vec![
                (
                    "Usage",""",
    """            crate::pages::Page::Select.import_line(),
            vec![
                (
                    "With Description",
                    col(vec![h::Select::new("sel-desc", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .description("Used for spell-checking")
                        .into_any_element()]),
                ),
                (
                    "Required",
                    col(vec![h::Select::new("sel-required", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_required(true)
                        .into_any_element()]),
                ),
                (
                    "Disabled",
                    col(vec![h::Select::new("sel-disabled", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .is_disabled(true)
                        .into_any_element()]),
                ),
                (
                    "With Disabled Options",
                    col(vec![h::Select::new("sel-disabled-opts", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .disabled_keys([1, 3])
                        .default_open(true)
                        .into_any_element()]),
                ),
                (
                    "With Sections",
                    col(vec![h::Select::new(
                        "sel-sections",
                        vec![
                            "United States".into(),
                            "Canada".into(),
                            "Mexico".into(),
                            "France".into(),
                            "Germany".into(),
                        ],
                    )
                    .label("Country")
                    .placeholder("Select a country")
                    .section_before(0, "North America")
                    .section_before(3, "Europe")
                    .default_open(true)
                    .into_any_element()]),
                ),
                (
                    "In Surface",
                    col(vec![h::Surface::new()
                        .padding(px(24.))
                        .child(
                            h::Select::new("sel-surface", languages())
                                .label("Language")
                                .placeholder("Choose one")
                                .variant(FieldVariant::Secondary),
                        )
                        .into_any_element()]),
                ),
                (
                    "Controlled Multiple",
                    col(vec![
                        h::Select::new("sel-ctl-multi", languages())
                            .label("Languages")
                            .placeholder("Choose any")
                            .selection_mode(SelectionMode::Multiple)
                            .selected_indices(sel_multi.iter().copied())
                            .on_selection_change_all(cx.listener(
                                |this, indices: &[usize], _, cx| {
                                    this.select_multi = indices.to_vec();
                                    cx.notify();
                                },
                            ))
                            .into_any_element(),
                        para(&format!("{} selected", sel_multi.len()), cx),
                    ]),
                ),
                (
                    "Controlled Open State",
                    col(vec![
                        row(vec![
                            h::Button::new("sel-open-btn")
                                .label(if is_open { "Close" } else { "Open" })
                                .variant(Variant::Secondary)
                                .size(Size::Sm)
                                .on_press(cx.listener(move |this, _, _, cx| {
                                    this.select_open = !this.select_open;
                                    cx.notify();
                                }))
                                .into_any_element(),
                            para(if is_open { "Open" } else { "Closed" }, cx),
                        ]),
                        h::Select::new("sel-open", languages())
                            .label("Language")
                            .placeholder("Choose one")
                            .is_open(is_open)
                            .on_open_change(bool_cb(cx.listener(|this, open: &bool, _, cx| {
                                this.select_open = *open;
                                cx.notify();
                            })))
                            .into_any_element(),
                    ]),
                ),
                (
                    "Asynchronous Loading",
                    col(vec![
                        para(
                            "v3 fills the list from a request and shows a spinner while it is in \\
                             flight. The spinner is composed beside the label, since the options \\
                             are the caller's own data.",
                            cx,
                        ),
                        row(vec![
                            h::Select::new("sel-async", languages())
                                .label("Language")
                                .placeholder("Loading\\u{2026}")
                                .into_any_element(),
                            h::Spinner::new("sel-async-spinner")
                                .size(h::SpinnerSize::Sm)
                                .into_any_element(),
                        ]),
                    ]),
                ),
                (
                    "Custom Indicator",
                    col(vec![h::Select::new("sel-indicator", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
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
                    col(vec![h::Select::new("sel-value", languages())
                        .label("Language")
                        .placeholder("Choose one")
                        .value(selected)
                        .value_content(move |index| match index {
                            Some(i) => gpui::div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .child(
                                    h::Chip::new(format!("#{}", i + 1))
                                        .size(Size::Sm)
                                        .variant(h::ChipVariant::Soft),
                                )
                                .child(
                                    languages()
                                        .get(i)
                                        .cloned()
                                        .unwrap_or_default()
                                        .to_string(),
                                )
                                .into_any_element(),
                            None => gpui::div().child("Choose one").into_any_element(),
                        })
                        .into_any_element()]),
                ),
                (
                    "Usage",""")

rep("""    pub fn page_select(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.select_lang;
        let is_open = self.select_open;""",
    """    pub fn page_select(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let selected = self.select_lang;
        let is_open = self.select_open;
        let sel_multi = self.select_multi.clone();""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched select page')
