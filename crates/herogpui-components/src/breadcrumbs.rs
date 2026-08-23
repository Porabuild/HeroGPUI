//! Breadcrumbs — port of `@heroui/breadcrumbs`.

use gpui::{prelude::*, px, App, ClickEvent, IntoElement, InteractiveElement, RenderOnce, SharedString, Styled, Window};
use herogpui_theme::ActiveTheme;

use crate::icons;

/// BreadcrumbSeparator style (`separator`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BreadcrumbSeparator {
    Slash,
    #[default]
    Chevron,
    Dash,
}

/// One breadcrumb item.
#[derive(Clone)]
pub struct Crumb {
    pub label: SharedString,
    pub href: Option<String>,
}

impl Crumb {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            href: None,
        }
    }

    pub fn href(mut self, href: impl Into<String>) -> Self {
        self.href = Some(href.into());
        self
    }
}

type OnNavigate = std::sync::Arc<dyn Fn(usize, &Crumb, &ClickEvent, &mut Window, &mut App) + 'static>;

/// HeroUI Breadcrumbs.
#[derive(IntoElement)]
pub struct Breadcrumbs {
    items: Vec<Crumb>,
    separator: BreadcrumbSeparator,
    is_disabled: bool,
    on_navigate: Option<OnNavigate>,
}

impl Breadcrumbs {
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn new(items: Vec<Crumb>) -> Self {
        Self {
            items,
            separator: BreadcrumbSeparator::Chevron,
            is_disabled: false,
            on_navigate: None,
        }
    }


    pub fn separator(mut self, s: BreadcrumbSeparator) -> Self {
        self.separator = s;
        self
    }

    /// Called with the index and crumb when a segment is clicked.
    pub fn on_navigate(
        mut self,
        f: impl Fn(usize, &Crumb, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(std::sync::Arc::new(f));
        self
    }
}

impl RenderOnce for Breadcrumbs {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let text_size = px(14.);
        let muted = colors.muted;
        let disabled = self.is_disabled;
        let disabled_opacity = cx.layout().disabled_opacity;
        let active_color = colors.foreground;
        let sep_icon = match self.separator {
            BreadcrumbSeparator::Slash => "/",
            BreadcrumbSeparator::Dash => "-",
            BreadcrumbSeparator::Chevron => "",
        };
        let separator = self.separator;
        let items = self.items.clone();
        let on_navigate = self.on_navigate;
        let item_count = items.len();

        let crumbs: Vec<gpui::AnyElement> = items
            .into_iter()
            .enumerate()
            .map(|(i, crumb)| {
                let is_last = i == item_count - 1;
                let row = gpui::div().flex().items_center().gap(px(8.));

                let mut label_el = gpui::div()
                    .id(gpui::ElementId::Name(format!("crumb-{i}").into()))
                    .text_size(text_size)
                    .line_height(text_size * 1.3)
                    .text_color(if is_last { active_color } else { muted })
                    .when(
                        !is_last && on_navigate.is_some() && !disabled,
                        |el| el.cursor_pointer().hover(move |s| s.text_color(active_color)),
                    )
                    .when(disabled, |el| el.opacity(disabled_opacity))
                    .child(crumb.label.to_string());

                if !is_last && !disabled {
                    if let (Some(on_nav), Some(href)) = (&on_navigate, crumb.href.clone()) {
                        let crumb2 = crumb.clone();
                        let idx = i;
                        let on_nav2 = on_nav.clone();
                        label_el = label_el.on_click(move |ev, w, cx| {
                            on_nav2(idx, &crumb2, ev, w, cx);
                            cx.open_url(&href);
                        });
                    } else if let Some(on_nav) = &on_navigate {
                        let crumb2 = crumb.clone();
                        let idx = i;
                        let on_nav2 = on_nav.clone();
                        label_el =
                            label_el.on_click(move |ev, w, cx| on_nav2(idx, &crumb2, ev, w, cx));
                    }
                }

                row.child(label_el).when(!is_last, |row| {
                    row.child(if separator == BreadcrumbSeparator::Chevron {
                        gpui::svg()
                            .size(px(12.))
                            .path(icons::CHEVRON_RIGHT)
                            .text_color(muted)
                            .into_any_element()
                    } else {
                        gpui::div()
                            .text_size(text_size)
                            .text_color(muted)
                            .child(sep_icon.to_string())
                            .into_any_element()
                    })
                })
                .into_any_element()
            },
            )
            .collect();

        gpui::div()
            .flex()
            .items_center()
            .flex_wrap()
            .children(crumbs)
    }
}
