//! Breadcrumbs — port of `@heroui/breadcrumbs`.

use gpui::{
    prelude::*, px, App, ClickEvent, FontWeight, InteractiveElement, IntoElement, RenderOnce,
    SharedString, Styled, Window,
};
use herogpui_core::element_id;
use herogpui_theme::ActiveTheme;

use crate::{
    a11y::{self, A11y as _},
    icons,
};

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

type OnNavigate =
    std::sync::Arc<dyn Fn(&usize, &Crumb, &ClickEvent, &mut Window, &mut App) + 'static>;

/// v3's `separator?: ReactNode`: custom content rebuilt for every non-last
/// crumb, painted inside the 12px `breadcrumbs__separator` slot.
type SeparatorRender = std::sync::Arc<dyn Fn(usize) -> gpui::AnyElement + 'static>;

/// HeroUI Breadcrumbs.
#[derive(IntoElement)]
pub struct Breadcrumbs {
    /// Instance identity for the keyed focus handles and link ids; without
    /// it the fallback derives from the crumb labels, unique unless two
    /// id-less instances hold identical labels.
    id: Option<gpui::ElementId>,
    items: Vec<Crumb>,
    separator: BreadcrumbSeparator,
    separator_render: Option<SeparatorRender>,
    is_disabled: bool,
    on_navigate: Option<OnNavigate>,
    /// Expands the root to the available width.
    full_width: bool,
    /// The `sx` slot, refined over the root style at the end of render.
    sx: Option<Box<gpui::StyleRefinement>>,
}

impl Breadcrumbs {
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    pub fn new(items: Vec<Crumb>) -> Self {
        Self {
            id: None,
            items,
            separator: BreadcrumbSeparator::Chevron,
            separator_render: None,
            is_disabled: false,
            on_navigate: None,
            full_width: false,
            sx: None,
        }
    }

    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn separator(mut self, s: BreadcrumbSeparator) -> Self {
        self.separator = s;
        self
    }

    /// Custom separator content between crumbs (v3 `separator: ReactNode`).
    /// The closure is called with each non-last crumb's index and must build a
    /// fresh element per call; its output paints inside the 12px separator
    /// slot and takes the slot's muted color. Overrides `separator`.
    pub fn separator_render(mut self, f: impl Fn(usize) -> gpui::AnyElement + 'static) -> Self {
        self.separator_render = Some(std::sync::Arc::new(f));
        self
    }

    /// Called with the index and crumb when a segment is clicked.
    pub fn on_navigate(
        mut self,
        f: impl Fn(&usize, &Crumb, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(std::sync::Arc::new(f));
        self
    }

    /// The one slot for caller-owned low-level styling: GPUI's styling methods
    /// (`bg`, `text_color`, `w`, `h`, `p`, `rounded`, `border_color`, …)
    /// applied to the breadcrumbs' root element after every value the active
    /// theme chose, so they win.
    /// `fullWidth` — expands the root to the available width without
    /// redistributing the children.
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }

    pub fn sx(mut self, style: impl FnOnce(gpui::Div) -> gpui::Div) -> Self {
        self.sx = Some(crate::util::capture_sx(style));
        self
    }
}

impl RenderOnce for Breadcrumbs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // The keyed focus handles must come first (`use_keyed_state` takes
        // `cx` mutably), and they must be keyed by this instance: bare
        // `crumb-{i}` literals made a second Breadcrumbs re-use the first
        // one's tab stops.
        let base_id = match &self.id {
            Some(id) => id.clone(),
            None => gpui::ElementId::Name(
                self.items
                    .iter()
                    .map(|c| c.label.as_ref())
                    .collect::<Vec<_>>()
                    .join("-")
                    .into(),
            ),
        };
        // The `debug_selector` spelling the deep tests query on. It is the
        // id's `Debug` form, not an id: `debug_selector` is a label, and
        // keeping it verbatim keeps those queries readable.
        let base = format!("{base_id:?}");
        let colors = cx.colors();
        let text_size = px(14.);
        let muted = colors.muted;
        let disabled = self.is_disabled;
        let disabled_opacity = cx.layout().disabled_opacity;
        // `.breadcrumbs__link[data-current="true"]` is `text-link`: the current
        // page takes the link token, not the foreground.
        let current_color = colors.link;
        let separator = self.separator;
        let separator_render = self.separator_render;
        let items = self.items.clone();
        let on_navigate = self.on_navigate;
        let item_count = items.len();

        let crumbs: Vec<gpui::AnyElement> = items
            .into_iter()
            .enumerate()
            .map(|(i, crumb)| {
                // `.breadcrumbs__item` is `flex shrink-0 items-center
                // justify-center gap-0.5 px-0.5`: a 2px gap between link and
                // separator, 2px of horizontal padding, and content that never
                // compresses when the bar outgrows its parent.
                let is_last = i == item_count - 1;
                let row_base = base.clone();
                let row = gpui::div()
                    .id(element_id::indexed(&base_id, "item", i))
                    // Each item is an RAC `Breadcrumb`, which
                    // `react-aria-components/dist/private/Breadcrumbs.mjs`
                    // renders as a `<li>` — role `listitem` — holding a
                    // `<Link>` (`@heroui/react/.../breadcrumbs/breadcrumbs.js`
                    // composes exactly that) and, when it is not the current
                    // page, the separator. Its `aria-current="page"` half is
                    // a recorded omission: gpui has no builder for it.
                    .a11y(a11y::Role::ListItem)
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .gap(px(2.))
                    .px(px(2.))
                    .debug_selector(move || format!("{row_base}-item-{i}"));

                // v3's Accessibility section claims "Keyboard navigation
                // support": every link crumb is a tab stop (React Aria link
                // semantics) and gpui already activates a focused element's
                // click listeners on Enter/Space, so no Enter handler of our
                // own is bound. The last crumb — the current page, disabled
                // upstream regardless of `href` — stays inert, and a disabled
                // crumb leaves the tab order like any other disabled control.
                let focus = (!is_last && !disabled).then(|| {
                    crate::util::tab_stop_handle(
                        element_id::scoped(&element_id::indexed(&base_id, "crumb", i), "focus"),
                        window,
                        cx,
                    )
                });

                // Every upstream item renders a Link: one with an `href`
                // navigates itself, one without is a span link that still
                // presses. A crumb is a link — cursor, ring, tab stop — unless
                // it is the current page or the whole bar is disabled; it
                // navigates only when it carries an `href` or the builder set
                // `on_navigate`.
                let is_link = !is_last && !disabled;
                let navigable = is_link && (on_navigate.is_some() || crumb.href.is_some());

                let mut label_el = gpui::div()
                    .id(element_id::indexed(&base_id, "crumb", i))
                    // Every item — the current page included — holds a
                    // `<Link>` upstream, and RAC hands it
                    // `isDisabled: isDisabled || isCurrent` rather than
                    // dropping it, so the last crumb is a disabled link and
                    // not a bare span. `react-aria/.../link/useLink.js` adds
                    // the explicit role only for a non-anchor element; the
                    // node is a link either way.
                    .a11y_named(a11y::Role::Link, &a11y::Name::labelled(crumb.label.clone()))
                    .when_some(focus.as_ref(), |el, handle| el.track_focus(handle))
                    .text_size(text_size)
                    // `.breadcrumbs__link` is `text-sm leading-5 font-medium`:
                    // leading-5 is a fixed 20px line box, not a ratio of the
                    // text size, and the weight is medium for every link.
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if is_last { current_color } else { muted })
                    // `px-0.5` pads the link itself. Hover underline comes from
                    // both `.breadcrumbs__link:hover` and the shared `.link`
                    // class — plain CSS `:hover`, so it applies to every
                    // enabled crumb including the inert span link.
                    .px(px(2.))
                    .when(is_link, |el| el.cursor(crate::util::interactive_cursor(cx)))
                    .when(is_link, |el| el.hover(|s| s.underline()))
                    // `[data-current]` carries `opacity-100` so the current
                    // page never takes the disabled fade.
                    .when(disabled && !is_last, |el| el.opacity(disabled_opacity))
                    .child(crumb.label.to_string());

                if navigable {
                    let crumb2 = crumb;
                    let idx = i;
                    let on_nav = on_navigate.clone();
                    let href = crumb2.href.clone();
                    label_el = label_el.on_click(move |ev, w, cx| {
                        if let Some(on_nav) = &on_nav {
                            on_nav(&idx, &crumb2, ev, w, cx);
                        }
                        if let Some(href) = &href {
                            cx.open_url(href);
                        }
                    });
                }
                // v3.2.4's breadcrumbs CSS defines no focus rule of its own,
                // but a keyboard-focused crumb is a focused Link, and the
                // port's links draw the `:focus-visible` status ring when
                // focused. Mouse focus leaves that ring off. Every link crumb
                // draws it — including the span link without `href` or
                // `on_navigate`.
                if is_link {
                    let focus = focus.as_ref().expect("a link crumb is a tab stop");
                    label_el =
                        crate::util::ring_if_focused(label_el, focus, true, Vec::new(), window, cx);
                }

                row.child(label_el)
                    .when(!is_last, |row| {
                        row.child(if let Some(render) = &separator_render {
                            // A custom node paints inside v3's separator slot:
                            // `.breadcrumbs__separator` is `size-3 text-muted`.
                            let sep_base = base.clone();
                            gpui::div()
                                .id(element_id::indexed(&base_id, "separator", i))
                                .debug_selector(move || format!("{sep_base}-separator-{i}"))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(12.))
                                .text_color(muted)
                                .child(render(i))
                                .into_any_element()
                        } else if separator == BreadcrumbSeparator::Chevron {
                            gpui::svg()
                                .size(px(12.))
                                .path(icons::CHEVRON_RIGHT)
                                .text_color(muted)
                                .into_any_element()
                        } else {
                            // The slash/dash glyphs paint in the same 12px
                            // slot box the chevron icon and custom nodes use.
                            let glyph = match separator {
                                BreadcrumbSeparator::Slash => "/",
                                _ => "-",
                            };
                            gpui::div()
                                .id(element_id::indexed(&base_id, "separator", i))
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(12.))
                                .text_size(px(12.))
                                .line_height(px(12.))
                                .text_color(muted)
                                .child(glyph.to_owned())
                                .into_any_element()
                        })
                    })
                    .into_any_element()
            })
            .collect();

        // `.breadcrumbs` is `flex items-center`: one line, no wrap.
        let mut el = gpui::div().flex().items_center().children(crumbs);
        if self.full_width {
            el = el.w_full();
        }
        // Stated after the layout chain so the wrapper stays readable to the
        // character-windowed regexes in `.shots/design_audit.py`.
        //
        // RAC's `Breadcrumbs` puts `useBreadcrumbs`' `navProps` on an `<ol>`,
        // not on a `<nav>` (`Breadcrumbs.mjs`: `...dom.ol, {...mergeProps(
        // DOMProps, navProps)}`), so the node is a list carrying the hook's
        // `'aria-label': ariaLabel || strings.format('breadcrumbs')` — the
        // pinned en-US string is `Breadcrumbs`
        // (`react-aria/dist/private/intl/breadcrumbs/en-US.mjs`). The port has
        // no locale plumbing, so the pinned en-US string is inlined, as the
        // toast region's would be.
        let el = el
            .id(base_id)
            .a11y_named(a11y::Role::List, &a11y::Name::labelled("Breadcrumbs"));
        crate::util::apply_sx(el, &self.sx)
    }
}
