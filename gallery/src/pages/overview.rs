use gpui::{prelude::*, px, AnyElement, Context};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::{nav_sections, Page};

#[derive(Clone, Copy)]
enum PreviewKind {
    Buttons,
    Collections,
    Colors,
    Controls,
    Data,
    DateTime,
    Feedback,
    Forms,
    Layout,
    Media,
    Navigation,
    Overlays,
    Pickers,
    Typography,
    Utilities,
}

impl Gallery {
    pub fn page_all_components(&mut self, cx: &mut Context<'_, Self>) -> AnyElement {
        let colors = cx.colors();
        let mut page = gpui::div()
            .w(px(860.))
            .max_w(gpui::relative(1.))
            .flex()
            .flex_col()
            .gap(px(18.))
            .child(
                gpui::div()
                    .text_size(px(30.))
                    .line_height(px(38.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(Page::AllComponents.title()),
            )
            .child(
                gpui::div()
                    .text_size(px(15.5))
                    .line_height(px(26.))
                    .text_color(colors.muted)
                    .child(Page::AllComponents.description()),
            );

        for section in nav_sections().into_iter().filter(|section| {
            section
                .items
                .iter()
                .any(|page| !page.import_line().is_empty())
        }) {
            if !crate::control::section_wanted(section.title, cx) {
                continue;
            }
            let mut cards: Vec<AnyElement> = section
                .items
                .into_iter()
                .filter(|page| !page.import_line().is_empty())
                .map(|component| {
                    component_card(component, preview_kind(component), component.title(), cx)
                })
                .collect();
            if section.title == "Buttons" {
                cards.push(component_card(
                    Page::ToggleButton,
                    PreviewKind::Buttons,
                    "Toggle Button Group",
                    cx,
                ));
            }
            page = page
                .mt(px(8.))
                .child(
                    gpui::div()
                        .text_size(px(20.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(section.title.to_owned()),
                )
                .child(gpui::div().flex().flex_wrap().gap(px(14.)).children(cards));
        }

        page.into_any_element()
    }
}

fn component_card(
    page: Page,
    kind: PreviewKind,
    label: &'static str,
    cx: &mut Context<'_, Gallery>,
) -> AnyElement {
    let colors = cx.colors();
    let id = gpui::ElementId::Name(format!("component-card-{label}").into());
    gpui::div()
        .id(id)
        .w(px(270.))
        .max_w(gpui::relative(1.))
        .p(px(4.))
        .rounded(px(14.))
        .border_1()
        .border_color(gpui::transparent_black())
        .cursor_pointer()
        .tab_index(0)
        .hover(move |style| style.bg(colors.default.soft()))
        .focus(move |style| style.border_color(colors.accent.color))
        .child(
            gpui::div()
                .h(px(132.))
                .w_full()
                .rounded(px(12.))
                .border_1()
                .border_color(colors.border)
                .bg(colors.background)
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(component_preview(kind, page, label, cx)),
        )
        .child(
            gpui::div()
                .px(px(4.))
                .pt(px(9.))
                .pb(px(5.))
                .text_size(px(13.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(label),
        )
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_initial_page(page);
            cx.notify();
        }))
        .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                this.set_initial_page(page);
                cx.notify();
            }
        }))
        .into_any_element()
}

fn component_preview(kind: PreviewKind, page: Page, label: &str, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    let accent = colors.accent.color;
    let neutral = colors.default.color;
    let muted = colors.muted;

    match kind {
        PreviewKind::Buttons => gpui::div()
            .flex()
            .items_center()
            .gap(px(5.))
            .children((0..button_count(page, label)).map(|index| {
                gpui::div()
                    .h(px(if page == Page::CloseButton { 28. } else { 26. }))
                    .min_w(px(if page == Page::CloseButton { 28. } else { 44. }))
                    .px(px(10.))
                    .rounded_full()
                    .bg(if index == 0 { accent } else { neutral })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.))
                    .text_color(if index == 0 {
                        colors.accent.foreground
                    } else {
                        colors.default.foreground
                    })
                    .child(if page == Page::CloseButton {
                        "×"
                    } else {
                        "Aa"
                    })
            }))
            .into_any_element(),
        PreviewKind::Collections => collection_preview(page, cx),
        PreviewKind::Colors => color_preview(page, cx),
        PreviewKind::Controls => control_preview(page, cx),
        PreviewKind::Data => data_preview(page, cx),
        PreviewKind::DateTime => date_time_preview(page, cx),
        PreviewKind::Feedback => feedback_preview(page, cx),
        PreviewKind::Forms => form_preview(page, cx),
        PreviewKind::Pickers => picker_preview(page, cx),
        PreviewKind::Layout => layout_preview(page, cx),
        PreviewKind::Media => gpui::div()
            .flex()
            .items_center()
            .child(
                gpui::div()
                    .size(px(48.))
                    .rounded_full()
                    .bg(colors.accent.soft()),
            )
            .child(
                gpui::div()
                    .ml(px(-10.))
                    .size(px(48.))
                    .rounded_full()
                    .border_2()
                    .border_color(colors.background)
                    .bg(colors.warning.soft()),
            )
            .into_any_element(),
        PreviewKind::Navigation => navigation_preview(page, cx),
        PreviewKind::Overlays => overlay_preview(page, cx),
        PreviewKind::Typography => typography_preview(page, cx),
        PreviewKind::Utilities => gpui::div()
            .w(px(150.))
            .h(px(80.))
            .p(px(10.))
            .rounded(px(10.))
            .bg(colors.surface.background)
            .flex()
            .flex_col()
            .justify_between()
            .children((0..4).map(|index| mini_line(110. - index as f32 * 12., muted)))
            .into_any_element(),
    }
}

fn control_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Slider => gpui::div()
            .relative()
            .w(px(156.))
            .h(px(8.))
            .rounded_full()
            .bg(colors.default.color)
            .child(
                gpui::div()
                    .h_full()
                    .w(px(96.))
                    .rounded_full()
                    .bg(colors.accent.color),
            )
            .child(
                gpui::div()
                    .absolute()
                    .left(px(87.))
                    .top(px(-6.))
                    .size(px(20.))
                    .rounded_full()
                    .border_2()
                    .border_color(colors.background)
                    .bg(colors.accent.color),
            )
            .into_any_element(),
        Page::Switch => gpui::div()
            .w(px(48.))
            .h(px(26.))
            .rounded_full()
            .bg(colors.accent.color)
            .p(px(3.))
            .flex()
            .justify_end()
            .child(
                gpui::div()
                    .size(px(20.))
                    .rounded_full()
                    .bg(colors.background),
            )
            .into_any_element(),
        _ => unreachable!("non-control page"),
    }
}

fn data_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Badge => gpui::div()
            .relative()
            .size(px(52.))
            .rounded_full()
            .bg(colors.default.soft())
            .child(
                gpui::div()
                    .absolute()
                    .right(px(-3.))
                    .top(px(-3.))
                    .size(px(20.))
                    .rounded_full()
                    .bg(colors.danger.color)
                    .text_size(px(9.))
                    .text_color(colors.danger.foreground)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("3"),
            )
            .into_any_element(),
        Page::Chip => gpui::div()
            .px(px(13.))
            .py(px(7.))
            .rounded_full()
            .bg(colors.accent.soft())
            .text_size(px(10.))
            .text_color(colors.accent.color)
            .child("Status")
            .into_any_element(),
        Page::Table => gpui::div()
            .w(px(162.))
            .p(px(9.))
            .rounded(px(9.))
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(8.))
            .children((0..3).map(|index| {
                gpui::div()
                    .flex()
                    .justify_between()
                    .child(mini_line(70., colors.muted))
                    .child(mini_line(34. + index as f32 * 4., colors.default.color))
            }))
            .into_any_element(),
        _ => unreachable!("non-data page"),
    }
}

fn date_time_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    if matches!(page, Page::DateField | Page::TimeField) {
        return gpui::div()
            .flex()
            .gap(px(5.))
            .children((0..3).map(|index| {
                gpui::div()
                    .px(px(9.))
                    .py(px(8.))
                    .rounded(px(8.))
                    .bg(colors.field.background)
                    .font_family(crate::app::MONO_FONT)
                    .text_size(px(10.))
                    .child(if page == Page::TimeField {
                        ["09", "30", "AM"][index]
                    } else {
                        ["08", "23", "26"][index]
                    })
            }))
            .into_any_element();
    }
    let grid = calendar_grid(cx);
    if matches!(page, Page::DatePicker | Page::DateRangePicker) {
        gpui::div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .child(field_box(118., cx).child("Select date"))
            .child(grid)
            .into_any_element()
    } else {
        grid.into_any_element()
    }
}

fn feedback_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Alert => gpui::div()
            .w(px(166.))
            .p(px(9.))
            .rounded(px(9.))
            .bg(colors.accent.soft())
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                gpui::div()
                    .size(px(8.))
                    .rounded_full()
                    .bg(colors.accent.color),
            )
            .child(mini_line(106., colors.accent.color))
            .into_any_element(),
        Page::Meter | Page::ProgressBar => progress_line(page == Page::Meter, cx),
        Page::ProgressCircle | Page::Spinner => gpui::div()
            .size(px(if page == Page::ProgressCircle {
                48.
            } else {
                34.
            }))
            .rounded_full()
            .border_2()
            .border_color(colors.accent.color)
            .child(
                gpui::div()
                    .m(px(9.))
                    .size(px(10.))
                    .rounded_full()
                    .bg(colors.background),
            )
            .into_any_element(),
        Page::Skeleton => gpui::div()
            .w(px(154.))
            .flex()
            .flex_col()
            .gap(px(9.))
            .children([132., 110., 76.].map(|width| mini_line(width, colors.default.color)))
            .into_any_element(),
        _ => unreachable!("non-feedback page"),
    }
}

fn form_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Checkbox | Page::CheckboxGroup => selection_rows(true, cx),
        Page::RadioGroup => selection_rows(false, cx),
        Page::InputOtp => gpui::div()
            .flex()
            .gap(px(5.))
            .children((0..5).map(|index| {
                gpui::div()
                    .size(px(27.))
                    .rounded(px(7.))
                    .bg(colors.field.background)
                    .border_1()
                    .border_color(colors.border)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(if index < 3 { "•" } else { "" })
            }))
            .into_any_element(),
        Page::NumberField => field_box(160., cx)
            .justify_between()
            .child("42")
            .child(gpui::div().child("−  +"))
            .into_any_element(),
        Page::Fieldset | Page::Form => gpui::div()
            .w(px(164.))
            .p(px(10.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(mini_line(52., colors.foreground))
            .child(mini_line(126., colors.muted))
            .child(mini_line(108., colors.muted))
            .into_any_element(),
        _ => gpui::div()
            .w(px(168.))
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(mini_line(48., colors.muted))
            .child(field_box(168., cx).child(mini_line(82., colors.field.placeholder)))
            .into_any_element(),
    }
}

fn layout_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Separator => gpui::div()
            .w(px(156.))
            .h(px(1.))
            .bg(colors.separator)
            .into_any_element(),
        Page::Toolbar => gpui::div()
            .flex()
            .gap(px(5.))
            .children((0..4).map(|index| {
                gpui::div().size(px(28.)).rounded(px(7.)).bg(if index == 1 {
                    colors.accent.soft()
                } else {
                    colors.surface_secondary
                })
            }))
            .into_any_element(),
        Page::Card | Page::Surface => gpui::div()
            .w(px(158.))
            .h(px(82.))
            .p(px(10.))
            .rounded(px(if page == Page::Card { 12. } else { 5. }))
            .bg(colors.surface.background)
            .border_1()
            .border_color(colors.border)
            .flex()
            .gap(px(8.))
            .child(
                gpui::div()
                    .w(px(38.))
                    .h_full()
                    .rounded(px(7.))
                    .bg(colors.default.color),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(mini_line(72., colors.muted))
                    .child(mini_line(54., colors.muted)),
            )
            .into_any_element(),
        _ => unreachable!("non-layout page"),
    }
}

fn navigation_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Accordion | Page::Disclosure => gpui::div()
            .w(px(160.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .children((0..3).map(|index| {
                gpui::div()
                    .p(px(7.))
                    .rounded(px(7.))
                    .bg(colors.surface_secondary)
                    .flex()
                    .justify_between()
                    .child(mini_line(74., colors.muted))
                    .child(if index == 0 { "⌃" } else { "⌄" })
            }))
            .into_any_element(),
        Page::Breadcrumbs => gpui::div()
            .flex()
            .gap(px(6.))
            .children(["Home", "/", "Docs", "/", "Page"].map(|item| {
                gpui::div()
                    .text_size(px(10.))
                    .text_color(colors.muted)
                    .child(item)
            }))
            .into_any_element(),
        Page::Link => gpui::div()
            .pb(px(2.))
            .border_b_1()
            .border_color(colors.accent.color)
            .text_size(px(12.))
            .text_color(colors.accent.color)
            .child("Open documentation →")
            .into_any_element(),
        Page::Pagination => gpui::div()
            .flex()
            .gap(px(5.))
            .children((1..=4).map(|index| {
                gpui::div()
                    .size(px(28.))
                    .rounded(px(8.))
                    .bg(if index == 2 {
                        colors.accent.color
                    } else {
                        colors.surface_secondary
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(9.))
                    .child(index.to_string())
            }))
            .into_any_element(),
        Page::Tabs => gpui::div()
            .flex()
            .gap(px(6.))
            .children((0..3).map(|index| {
                gpui::div()
                    .px(px(12.))
                    .py(px(7.))
                    .rounded_full()
                    .bg(if index == 0 {
                        colors.accent.color
                    } else {
                        colors.surface_secondary
                    })
                    .child(mini_line(24., colors.muted))
            }))
            .into_any_element(),
        _ => unreachable!("non-navigation page"),
    }
}

fn overlay_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    let (width, height, top, left) = match page {
        Page::Tooltip => (88., 32., 8., 40.),
        Page::Toast => (132., 44., 38., 28.),
        Page::Drawer => (72., 88., 0., 96.),
        Page::Popover => (112., 58., 18., 44.),
        Page::AlertDialog | Page::Modal => (128., 72., 8., 20.),
        _ => unreachable!("non-overlay page"),
    };
    gpui::div()
        .relative()
        .w(px(168.))
        .h(px(88.))
        .rounded(px(10.))
        .bg(colors.default.soft())
        .child(
            gpui::div()
                .absolute()
                .top(px(top))
                .left(px(left))
                .w(px(width))
                .h(px(height))
                .p(px(9.))
                .rounded(px(10.))
                .bg(colors.overlay.background)
                .shadow(cx.layout().overlay_shadow.clone())
                .flex()
                .flex_col()
                .gap(px(7.))
                .child(mini_line(width - 34., colors.overlay.foreground))
                .child(mini_line(width - 48., colors.muted)),
        )
        .into_any_element()
}

fn picker_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    let field = field_box(166., cx)
        .justify_between()
        .child(mini_line(78., colors.field.placeholder))
        .child("⌄");
    match page {
        Page::Select => field.into_any_element(),
        Page::Autocomplete | Page::ComboBox => gpui::div()
            .flex()
            .flex_col()
            .gap(px(5.))
            .child(field)
            .child(
                gpui::div()
                    .w(px(166.))
                    .p(px(7.))
                    .rounded(px(8.))
                    .bg(colors.overlay.background)
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(mini_line(84., colors.muted))
                    .child(mini_line(62., colors.muted)),
            )
            .into_any_element(),
        _ => unreachable!("non-picker page"),
    }
}

fn typography_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    if page == Page::Kbd {
        gpui::div()
            .flex()
            .gap(px(6.))
            .children(["Ctrl", "K"].map(|key| {
                gpui::div()
                    .px(px(9.))
                    .py(px(6.))
                    .rounded(px(7.))
                    .bg(colors.surface_secondary)
                    .font_family(crate::app::MONO_FONT)
                    .text_size(px(10.))
                    .child(key)
            }))
            .into_any_element()
    } else {
        gpui::div()
            .flex()
            .items_end()
            .gap(px(10.))
            .child(
                gpui::div()
                    .text_size(px(30.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Aa"),
            )
            .child(
                gpui::div()
                    .text_size(px(13.))
                    .text_color(colors.muted)
                    .child("Body"),
            )
            .into_any_element()
    }
}

fn calendar_grid(cx: &gpui::App) -> gpui::Div {
    let colors = cx.colors();
    gpui::div()
        .w(px(142.))
        .p(px(8.))
        .rounded(px(10.))
        .border_1()
        .border_color(colors.border)
        .flex()
        .flex_wrap()
        .gap(px(5.))
        .children((0..14).map(|index| {
            gpui::div().size(px(14.)).rounded(px(4.)).bg(if index == 8 {
                colors.accent.color
            } else {
                colors.surface_secondary
            })
        }))
}

fn field_box(width: f32, cx: &gpui::App) -> gpui::Div {
    let colors = cx.colors();
    gpui::div()
        .w(px(width))
        .h(px(34.))
        .px(px(10.))
        .rounded(px(10.))
        .bg(colors.field.background)
        .border_1()
        .border_color(colors.border)
        .flex()
        .items_center()
}

fn progress_line(with_label: bool, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    gpui::div()
        .w(px(164.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .when(with_label, |preview| {
            preview.child(mini_line(48., colors.muted))
        })
        .child(
            gpui::div()
                .h(px(7.))
                .w_full()
                .rounded_full()
                .bg(colors.default.color)
                .child(
                    gpui::div()
                        .h_full()
                        .w(px(104.))
                        .rounded_full()
                        .bg(colors.accent.color),
                ),
        )
        .into_any_element()
}

fn selection_rows(square: bool, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    gpui::div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .children((0..3).map(|index| {
            gpui::div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    gpui::div()
                        .size(px(15.))
                        .rounded(if square { px(4.) } else { px(8.) })
                        .bg(if index == 0 {
                            colors.accent.color
                        } else {
                            colors.surface_secondary
                        }),
                )
                .child(mini_line(76. - index as f32 * 8., colors.muted))
        }))
        .into_any_element()
}

fn collection_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    match page {
        Page::Dropdown => gpui::div()
            .w(px(150.))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(5.))
            .child(
                gpui::div()
                    .px(px(9.))
                    .py(px(5.))
                    .rounded_full()
                    .bg(colors.accent.soft())
                    .child(mini_line(34., colors.accent.color)),
            )
            .child(
                gpui::div()
                    .w_full()
                    .p(px(8.))
                    .rounded(px(10.))
                    .bg(colors.overlay.background)
                    .border_1()
                    .border_color(colors.border)
                    .flex()
                    .flex_col()
                    .gap(px(7.))
                    .child(mini_line(76., colors.overlay.foreground))
                    .child(mini_line(58., colors.muted)),
            )
            .into_any_element(),
        Page::ListBox => gpui::div()
            .w(px(150.))
            .p(px(8.))
            .rounded(px(10.))
            .bg(colors.surface.background)
            .border_1()
            .border_color(colors.border)
            .flex()
            .flex_col()
            .gap(px(7.))
            .children((0..3).map(|index| {
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(mini_line(70. - index as f32 * 8., colors.muted))
                    .when(index == 0, |row| {
                        row.child(
                            gpui::div()
                                .size(px(6.))
                                .rounded_full()
                                .bg(colors.accent.color),
                        )
                    })
            }))
            .into_any_element(),
        Page::TagGroup => gpui::div()
            .flex()
            .items_center()
            .gap(px(6.))
            .children(
                ["Rust", "GPUI", "UI"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| {
                        gpui::div()
                            .px(px(10.))
                            .py(px(6.))
                            .rounded_full()
                            .bg(if index == 0 {
                                colors.accent.soft()
                            } else {
                                colors.surface_secondary
                            })
                            .text_size(px(9.))
                            .text_color(if index == 0 {
                                colors.accent.color
                            } else {
                                colors.muted
                            })
                            .child(label)
                    }),
            )
            .into_any_element(),
        _ => unreachable!("non-collection page"),
    }
}

fn color_preview(page: Page, cx: &gpui::App) -> AnyElement {
    let colors = cx.colors();
    let swatches = || {
        gpui::div().flex().gap(px(6.)).children(
            [
                colors.accent.color,
                colors.success.color,
                colors.warning.color,
                colors.danger.color,
            ]
            .into_iter()
            .map(|color| gpui::div().size(px(24.)).rounded(px(7.)).bg(color)),
        )
    };
    match page {
        Page::ColorArea => gpui::div()
            .relative()
            .w(px(126.))
            .h(px(72.))
            .rounded(px(10.))
            .bg(colors.accent.soft())
            .border_1()
            .border_color(colors.accent.color)
            .child(
                gpui::div()
                    .absolute()
                    .right(px(18.))
                    .bottom(px(15.))
                    .size(px(11.))
                    .rounded_full()
                    .border_2()
                    .border_color(colors.background)
                    .bg(colors.accent.color),
            )
            .into_any_element(),
        Page::ColorField => gpui::div()
            .w(px(150.))
            .px(px(11.))
            .py(px(9.))
            .rounded(px(10.))
            .bg(colors.field.background)
            .border_1()
            .border_color(colors.border)
            .font_family(crate::app::MONO_FONT)
            .text_size(px(11.))
            .child("#0085F5")
            .into_any_element(),
        Page::ColorPicker => gpui::div()
            .w(px(154.))
            .p(px(8.))
            .rounded(px(12.))
            .bg(colors.overlay.background)
            .shadow(cx.layout().overlay_shadow.clone())
            .flex()
            .flex_col()
            .gap(px(7.))
            .child(
                gpui::div()
                    .w_full()
                    .h(px(48.))
                    .rounded(px(8.))
                    .bg(colors.accent.soft()),
            )
            .child(swatches())
            .into_any_element(),
        Page::ColorSlider => gpui::div()
            .relative()
            .w(px(156.))
            .h(px(12.))
            .rounded_full()
            .bg(colors.accent.soft())
            .child(
                gpui::div()
                    .absolute()
                    .left(px(91.))
                    .top(px(-4.))
                    .size(px(20.))
                    .rounded_full()
                    .border_2()
                    .border_color(colors.background)
                    .bg(colors.accent.color),
            )
            .into_any_element(),
        Page::ColorSwatch => gpui::div()
            .size(px(52.))
            .rounded(px(14.))
            .bg(colors.accent.color)
            .into_any_element(),
        Page::ColorSwatchPicker => swatches().into_any_element(),
        _ => unreachable!("non-color page"),
    }
}

fn mini_line(width: f32, color: gpui::Hsla) -> gpui::Div {
    gpui::div().w(px(width)).h(px(4.)).rounded_full().bg(color)
}

fn button_count(page: Page, label: &str) -> usize {
    if page == Page::ButtonGroup || label == "Toggle Button Group" {
        3
    } else {
        1
    }
}

fn preview_kind(page: Page) -> PreviewKind {
    match page {
        Page::Button | Page::ButtonGroup | Page::CloseButton | Page::ToggleButton => {
            PreviewKind::Buttons
        }
        Page::Dropdown | Page::ListBox | Page::TagGroup => PreviewKind::Collections,
        Page::ColorArea
        | Page::ColorField
        | Page::ColorPicker
        | Page::ColorSlider
        | Page::ColorSwatch
        | Page::ColorSwatchPicker => PreviewKind::Colors,
        Page::Slider | Page::Switch => PreviewKind::Controls,
        Page::Badge | Page::Chip | Page::Table => PreviewKind::Data,
        Page::Calendar
        | Page::DateField
        | Page::DatePicker
        | Page::DateRangePicker
        | Page::RangeCalendar
        | Page::TimeField => PreviewKind::DateTime,
        Page::Alert
        | Page::Meter
        | Page::ProgressBar
        | Page::ProgressCircle
        | Page::Skeleton
        | Page::Spinner => PreviewKind::Feedback,
        Page::Checkbox
        | Page::CheckboxGroup
        | Page::Fieldset
        | Page::FieldSlots
        | Page::Form
        | Page::Input
        | Page::InputGroup
        | Page::InputOtp
        | Page::NumberField
        | Page::RadioGroup
        | Page::SearchField
        | Page::TextArea
        | Page::TextField => PreviewKind::Forms,
        Page::Card | Page::Separator | Page::Surface | Page::Toolbar => PreviewKind::Layout,
        Page::Avatar => PreviewKind::Media,
        Page::Accordion
        | Page::Breadcrumbs
        | Page::Disclosure
        | Page::Link
        | Page::Pagination
        | Page::Tabs => PreviewKind::Navigation,
        Page::AlertDialog
        | Page::Drawer
        | Page::Modal
        | Page::Popover
        | Page::Toast
        | Page::Tooltip => PreviewKind::Overlays,
        Page::Autocomplete | Page::ComboBox | Page::Select => PreviewKind::Pickers,
        Page::Kbd | Page::Typography => PreviewKind::Typography,
        Page::ScrollShadow => PreviewKind::Utilities,
        Page::AllComponents
        | Page::Releases
        | Page::ReleaseCurrent
        | Page::Introduction
        | Page::Installation
        | Page::Theming
        | Page::DarkMode
        | Page::Customization
        | Page::Styling
        | Page::DesignPrinciples => unreachable!("non-component page has no preview"),
    }
}
