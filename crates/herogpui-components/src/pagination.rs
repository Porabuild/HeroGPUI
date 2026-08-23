//! Pagination — port of `@heroui/pagination`.

use gpui::{
    prelude::*, px, App, InteractiveElement, IntoElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window,
};
use herogpui_core::{Color, Size};
use herogpui_theme::ActiveTheme;

use crate::icons;

type OnChange = std::sync::Arc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

type Link = std::sync::Arc<dyn Fn(usize, bool) -> gpui::AnyElement + 'static>;

/// HeroUI Pagination (controlled).
#[derive(IntoElement)]
pub struct Pagination {
    /// `link` — v3's render prop for a page link, handed `isActive`.
    link: Option<Link>,
    summary: Option<gpui::SharedString>,
    id: gpui::ElementId,
    page: usize,
    total: usize,
    siblings: usize,
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
            link: None,
            summary: None,
            id: id.into(),
            page: page.max(1),
            total: total.max(1),
            siblings: 1,
            is_disabled: false,
            size: Size::Md,
            on_change: None,
        }
    }

    /// `Pagination.Summary` — the "Page 1 of 10" text v3 composes at the start
    /// of the row (`flex items-center gap-2 text-sm text-muted`).
    pub fn summary(mut self, text: impl Into<gpui::SharedString>) -> Self {
        self.summary = Some(text.into());
        self
    }

    /// `children` on `Pagination.Link` — replaces a page's label.
    ///
    /// The closure receives the page number and `isActive`, the values v3
    /// passes into the same render prop.
    pub fn link(mut self, render: impl Fn(usize, bool) -> gpui::AnyElement + 'static) -> Self {
        self.link = Some(std::sync::Arc::new(render));
        self
    }

    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Every interactive item is a tab stop with a ring. The handles come
        // first: `use_keyed_state` takes `cx` mutably and the theme is borrowed
        // for the rest of the render.
        let base_id = format!("{:?}", self.id);
        let page_focus: Vec<gpui::FocusHandle> = (0..=self.total)
            .map(|n| {
                crate::util::tab_stop_handle(
                    gpui::ElementId::Name(format!("{base_id}-page-{n}-focus").into()),
                    window,
                    cx,
                )
            })
            .collect();
        let prev_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base_id}-prev-focus").into()),
            window,
            cx,
        );
        let next_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{base_id}-next-focus").into()),
            window,
            cx,
        );
        let ring_visible = crate::util::focus_visible(cx);

        let sem = cx.role(Color::Accent);
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

        // `.pagination__content` is `gap-1`, not the 16px this used to leave.
        let mut row = gpui::div().flex().items_center().gap(px(4.));

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
                    radius: crate::util::control_radius(cx),
                },
                &prev_focus,
                (ring_visible && prev_focus.is_focused(window))
                    .then(|| crate::util::focus_ring_shadows(true, cx)),
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

        for p in pages {
            match p {
                PageRef::Num(n) => {
                    let active = n == self.page;
                    let mut btn = gpui::div()
                        .id(gpui::ElementId::Name(format!("{base}-page-{n}").into()))
                        .when_some(
                            page_focus.get(n).filter(|_| !self.is_disabled),
                            |b, handle| b.track_focus(handle),
                        )
                        .flex()
                        .items_center()
                        .justify_center()
                        // `.pagination__link` is `size-8` from `md` up: a square
                        // cell with no padding of its own.
                        .min_w(cell)
                        .h(cell)
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
                            let pressed_bg = colors.default.hover();
                            btn = btn.hover(move |s| s.bg(hover_bg));
                            // `.pagination__link[data-pressed]` deepens the fill
                            // and scales to 0.97.
                            btn = crate::anim::pressed(
                                btn,
                                crate::anim::PressBox {
                                    height: cell,
                                    padding_x: Some(px(6.)),
                                    width: None,
                                    min_width: Some(cell),
                                    text_size: cell_text,
                                    line_height: cell_text,
                                    gap: px(0.),
                                    radius: crate::util::control_radius(cx),
                                    shrink_x: true,
                                    scale: crate::anim::PRESSED_SCALE,
                                },
                                cx,
                            )
                            .active(move |s| s.bg(pressed_bg));
                        }
                        if let Some(cb) = self_on_change.clone() {
                            btn = btn.on_click(move |_, w, cx| cb(n, w, cx));
                        }
                    }
                    // `link` is v3's render prop on `Pagination.Link`: it
                    // receives `isActive`, so a caller can style the current
                    // page without re-deriving which one it is.
                    // `.pagination__item:focus-visible` is `status-focused`.
                    let btn = crate::util::with_focus_ring(
                        btn,
                        ring_visible && page_focus.get(n).is_some_and(|h| h.is_focused(window)),
                        true,
                        Vec::new(),
                        cx,
                    );
                    row = row.child(match &self.link {
                        Some(render) => btn.child(render(n, active)),
                        None => btn.child(n.to_string()),
                    });
                }
                PageRef::Ellipsis => {
                    row = row.child(
                        gpui::div()
                            .flex()
                            .items_center()
                            .justify_center()
                            // `.pagination__ellipsis` is the same `size-8
                            // text-sm` cell as a page link.
                            .size(cell)
                            .text_size(cell_text)
                            .text_color(colors.muted)
                            .child("…"),
                    );
                }
            }
        }

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
                    radius: crate::util::control_radius(cx),
                },
                &next_focus,
                (ring_visible && next_focus.is_focused(window))
                    .then(|| crate::util::focus_ring_shadows(true, cx)),
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

        // `.pagination` is the root: `flex w-full items-center justify-between
        // gap-4` around the summary and the content.
        gpui::div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(16.))
            .children(self.summary.map(|text| {
                gpui::div()
                    .flex()
                    .items_center()
                    // `.pagination__summary` is `gap-2 text-sm text-muted`.
                    .gap(px(8.))
                    .text_size(px(14.))
                    .text_color(colors.muted)
                    .child(text.to_string())
            }))
            .child(row)
    }
}

/// Colours and metrics shared by the nav buttons.
struct NavStyle {
    foreground: gpui::Hsla,
    hover_bg: gpui::Hsla,
    border: gpui::Hsla,
    disabled_opacity: f32,
    cell: gpui::Pixels,
    /// `.pagination__link` is `rounded-3xl`; `--nav` restates only the width.
    radius: gpui::Pixels,
}

fn nav_button(
    id: String,
    icon: &'static str,
    enabled: bool,
    style: NavStyle,
    focus: &gpui::FocusHandle,
    // The focus ring's shadows, when this button is the one holding the focus.
    ring: Option<Vec<gpui::BoxShadow>>,
) -> gpui::Stateful<gpui::Div> {
    let NavStyle {
        foreground,
        hover_bg,
        border,
        disabled_opacity,
        cell,
        radius,
    } = style;
    let mut btn = gpui::div()
        .id(gpui::ElementId::Name(id.into()))
        .track_focus(focus)
        .when_some(ring, |b, shadows| b.shadow(shadows))
        .flex()
        .items_center()
        .justify_center()
        // `.pagination__link--nav` is `w-auto gap-1.5 px-2.5`: the height of a
        // page cell, but as wide as its content needs.
        .h(cell)
        .gap(px(6.))
        .px(px(10.))
        .rounded(radius)
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
