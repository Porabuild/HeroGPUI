//! Pagination — port of `@heroui/pagination`.

use gpui::{prelude::*, px, App, IntoElement, InteractiveElement, RenderOnce, StatefulInteractiveElement, Styled, Window};
use herogpui_core::{Size, Color};
use herogpui_theme::ActiveTheme;

use crate::icons;

type OnChange = std::sync::Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// HeroUI Pagination (controlled).
#[derive(IntoElement)]
pub struct Pagination {
    id: gpui::ElementId,
    page: usize,
    total: usize,
    siblings: usize,
    color: Color,
    show_controls: bool,
    is_disabled: bool,
    size: Size,
    on_change: Option<OnChange>,
}

impl Pagination {
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn new(id: impl Into<gpui::ElementId>, page: usize, total: usize) -> Self {
        Self {
            id: id.into(),
            page: page.max(1),
            total: total.max(1),
            siblings: 1,
            color: Color::Accent,
            show_controls: true,
            is_disabled: false,
            size: Size::Md,
            on_change: None,
        }
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }


    pub fn show_controls(mut self, v: bool) -> Self {
        self.show_controls = v;
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let colors = cx.colors();
        let layout = cx.layout();
        let base = format!("{:?}", self.id);

        let self_on_change: Option<OnChange> = self.on_change.clone();
        let pages = visible_pages(self.page, self.total, self.siblings);

        // v3 sizes the items; the nav buttons and page cells share it.
        let cell = match self.size {
            Size::Sm => px(28.),
            Size::Md => px(32.),
            Size::Lg => px(40.),
        };
        let cell_text = self.size.text_size();

        let mut row = gpui::div().flex().items_center().gap(px(4.));

        if self.show_controls {
            row = row.child(
                nav_button(
                    format!("{base}-prev"),
                    icons::CHEVRON_LEFT,
                    self.page > 1 && !self.is_disabled,
                    NavStyle {
                        foreground: colors.foreground,
                        hover_bg: colors.default.color,
                        border: colors.border,
                        disabled_opacity: layout.disabled_opacity,
                        cell,
                    },
                )
                .on_click({
                    let cb: Option<OnChange> = self.on_change.clone();
                    move |_, w, cx| {
                        if let Some(cb) = &cb {
                            cb(self.page - 1, w, cx);
                        }
                    }
                }),
            );
        }

        for p in pages {
            match p {
                PageRef::Num(n) => {
                    let active = n == self.page;
                    let mut btn = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-page-{n}").into()))
                        .flex()
                        .items_center()
                        .justify_center()
                        .min_w(cell)
                        .h(cell)
                        .px(px(6.))
                        .text_size(cell_text)
                        .rounded(crate::util::control_radius(cx))
                        .when(!self.is_disabled, |b| b.cursor_pointer());

                    if active {
                        btn = btn.bg(sem.color).text_color(sem.foreground);
                    } else {
                        btn = btn
                            .text_color(colors.foreground)
                            .border_1()
                            .border_color(colors.default.soft_hover());
                        if !self.is_disabled {
                            let hover_bg = colors.default.color;
                            btn = btn.hover(move |s| s.bg(hover_bg));
                        }
                        if let Some(cb) = self_on_change.clone() {
                            btn = btn.on_click(move |_, w, cx| cb(n, w, cx));
                        }
                    }
                    row = row.child(btn.child(n.to_string()));
                }
                PageRef::Ellipsis => {
                    row = row.child(
                        gpui::div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(24.))
                            .h(px(32.))
                            .text_size(px(13.5))
                            .text_color(colors.muted)
                            .child("…"),
                    );
                }
            }
        }

        if self.show_controls {
            row = row.child(
                nav_button(
                    format!("{base}-next"),
                    icons::CHEVRON_RIGHT,
                    self.page < self.total && !self.is_disabled,
                    NavStyle {
                        foreground: colors.foreground,
                        hover_bg: colors.default.color,
                        border: colors.border,
                        disabled_opacity: layout.disabled_opacity,
                        cell,
                    },
                )
                .on_click({
                    let cb: Option<OnChange> = self.on_change.clone();
                    move |_, w, cx| {
                        if let Some(cb) = &cb {
                            cb(self.page + 1, w, cx);
                        }
                    }
                }),
            );
        }

        row
    }
}

/// Colours and metrics shared by the nav buttons.
struct NavStyle {
    foreground: gpui::Hsla,
    hover_bg: gpui::Hsla,
    border: gpui::Hsla,
    disabled_opacity: f32,
    cell: gpui::Pixels,
}

fn nav_button(
    id: String,
    icon: &'static str,
    enabled: bool,
    style: NavStyle,
) -> gpui::Stateful<gpui::Div> {
    let NavStyle {
        foreground,
        hover_bg,
        border,
        disabled_opacity,
        cell,
    } = style;
    let mut btn = gpui::div()
        .id(gpui::ElementId::Name(id.into()))
        .flex()
        .items_center()
        .justify_center()
        .size(cell)
        .rounded(util_radius())
        .border_1()
        .border_color(border)
        .text_color(foreground);
    if enabled {
        btn = btn.cursor_pointer().hover(move |s| s.bg(hover_bg));
    } else {
        btn = btn.opacity(disabled_opacity);
    }
    // gpui svgs do not inherit text colour.
    btn.child(gpui::svg().size(px(14.)).path(icon).text_color(foreground))
}

fn util_radius() -> gpui::Pixels {
    px(10.)
}

enum PageRef {
    Num(usize),
    Ellipsis,
}

fn visible_pages(page: usize, total: usize, siblings: usize) -> Vec<PageRef> {
    // total numbers to show = 2*siblings + 5 (first, last, current, 2 ellipsis slots)
    let total_slots = 2 * siblings + 5;
    if total <= total_slots {
        return (1..=total).map(PageRef::Num).collect();
    }

    let left = (page - siblings).max(2);
    let right = (page + siblings).min(total - 1);

    let mut out = vec![PageRef::Num(1)];
    if left > 2 {
        out.push(PageRef::Ellipsis);
    }
    for n in left..=right {
        out.push(PageRef::Num(n));
    }
    if right < total - 1 {
        out.push(PageRef::Ellipsis);
    }
    out.push(PageRef::Num(total));
    out
}





