//! Toast — port of `@heroui/toast` (v3).
//!
//! Create the store once (`toast_store(cx)`), render [`ToastViewport`] — the
//! equivalent of `Toast.Provider` — from your root view, and fire toasts
//! anywhere with [`Toast::push`].
//!
//! v3 names the colour prop `variant`, and its values are the semantic colour
//! roles, so [`Color`] is the variant type here.

use std::time::Duration;

use gpui::{prelude::*, px, App, Entity, Global, IntoElement, RenderOnce, SharedString, Styled, Window};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

use crate::icons;

/// `maxVisibleToasts` default from `Toast.Provider`.
pub const DEFAULT_MAX_VISIBLE_TOASTS: usize = 3;

/// Where the toast region sits (`placement` on `Toast.Provider`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPlacement {
    TopStart,
    Top,
    TopEnd,
    BottomStart,
    #[default]
    Bottom,
    BottomEnd,
}

impl ToastPlacement {
    pub const ALL: [ToastPlacement; 6] = [
        ToastPlacement::TopStart,
        ToastPlacement::Top,
        ToastPlacement::TopEnd,
        ToastPlacement::BottomStart,
        ToastPlacement::Bottom,
        ToastPlacement::BottomEnd,
    ];

    fn is_top(self) -> bool {
        matches!(self, Self::TopStart | Self::Top | Self::TopEnd)
    }

    pub fn label(self) -> &'static str {
        match self {
            ToastPlacement::TopStart => "Top start",
            ToastPlacement::Top => "Top",
            ToastPlacement::TopEnd => "Top end",
            ToastPlacement::BottomStart => "Bottom start",
            ToastPlacement::Bottom => "Bottom",
            ToastPlacement::BottomEnd => "Bottom end",
        }
    }
}

/// One toast's data.
#[derive(Clone)]
pub struct ToastData {
    pub id: u64,
    pub color: Color,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub closable: bool,
}

/// Entity holding the active toasts.
pub struct ToastStore {
    toasts: Vec<ToastData>,
    next_id: u64,
}

impl ToastStore {
    fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 1,
        }
    }

    pub fn toasts(&self) -> &[ToastData] {
        &self.toasts
    }

    /// Pushes a toast, evicting the oldest beyond `max_visible`.
    pub fn push_capped(&mut self, data: ToastData, max_visible: usize) -> u64 {
        let cap = max_visible.max(1);
        while self.toasts.len() >= cap {
            self.toasts.remove(0);
        }
        let id = data.id;
        self.toasts.push(data);
        id
    }

    pub fn push(&mut self, data: ToastData) -> u64 {
        self.push_capped(data, DEFAULT_MAX_VISIBLE_TOASTS)
    }

    fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    /// Inserts a fully-formed toast; zero ids are auto-assigned.
    pub fn insert(&mut self, mut data: ToastData) -> u64 {
        if data.id == 0 {
            data.id = self.next_id;
            self.next_id += 1;
        }
        let id = data.id;
        self.toasts.push(data);
        id
    }
}

struct ToastHub {
    store: Entity<ToastStore>,
}
impl Global for ToastHub {}

/// Creates (or returns) the app-wide toast store.
pub fn toast_store(cx: &mut App) -> Entity<ToastStore> {
    if let Some(hub) = cx.try_global::<ToastHub>() {
        return hub.store.clone();
    }
    let store = cx.new(|_| ToastStore::new());
    cx.set_global(ToastHub { store: store.clone() });
    store
}

/// Builder for a toast notification.
pub struct Toast {
    color: Color,
    title: SharedString,
    description: Option<SharedString>,
    closable: bool,
}

impl Toast {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            color: Color::Accent,
            title: title.into(),
            description: None,
            closable: true,
        }
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// `variant` — `default | accent | success | warning | danger`.
    pub fn variant(mut self, variant: Color) -> Self {
        self.color = variant;
        self
    }

    pub fn closable(mut self, v: bool) -> Self {
        self.closable = v;
        self
    }

    /// Pushes the toast; auto-dismisses after `duration` when given.
    /// Requires a window handle only to keep parity with other callbacks.
    pub fn push(self, duration: Option<Duration>, cx: &mut App) -> u64 {
        let store = toast_store(cx);
        let id = store.update(cx, |s, cx| {
            let id = s.next_id;
            s.next_id += 1;
            let pushed = s.push(ToastData {
                id,
                color: self.color,
                title: self.title.clone(),
                description: self.description.clone(),
                closable: self.closable,
            });
            cx.notify();
            pushed
        });

        if let Some(dur) = duration {
            let weak = store.downgrade();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                cx.background_executor().timer(dur).await;
                if let Some(store) = weak.upgrade() {
                    // Update through AsyncApp returns a Result; dropping is
                    // fine since a missing entity just means the app closed.
                    let _ = store.update(cx, |s, cx| {
                        s.dismiss(id);
                        cx.notify();
                    });
                }
            })
            .detach();
        }

        id
    }
}

/// Convenience free function (manual dismissal).
pub fn push_toast(toast: Toast, cx: &mut App) -> u64 {
    toast.push(None, cx)
}

/// Dismisses one toast by id.
pub fn dismiss_toast(id: u64, cx: &mut App) {
    let store = toast_store(cx);
    store.update(cx, |s, cx| {
        s.dismiss(id);
        cx.notify();
    });
}

/// The toast region — `Toast.Provider` in React. Mount once near the root; it
/// reads the store on every root re-render (store mutations notify it via the
/// parent view's `cx.notify()`).
#[derive(IntoElement)]
pub struct ToastViewport {
    placement: ToastPlacement,
    gap: gpui::Pixels,
    max_visible_toasts: usize,
    width: gpui::Pixels,
    inset: gpui::Pixels,
    scale_factor: f32,
}

impl ToastViewport {
    pub fn new() -> Self {
        Self {
            placement: ToastPlacement::default(),
            gap: px(12.),
            max_visible_toasts: DEFAULT_MAX_VISIBLE_TOASTS,
            scale_factor: 0.05,
            width: px(460.),
            inset: px(16.),
        }
    }

    pub fn placement(mut self, placement: ToastPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn gap(mut self, gap: impl Into<gpui::Pixels>) -> Self {
        self.gap = gap.into();
        self
    }

    /// `scaleFactor` on `Toast.Provider` — how much each toast behind the
    /// newest one shrinks, 0.05 in v3.
    ///
    /// gpui cannot scale a div, so the shrink is geometric: a stacked toast is
    /// inset horizontally by its depth's share of the width. Pass `0.0` for a
    /// flat stack.
    pub fn scale_factor(mut self, factor: f32) -> Self {
        self.scale_factor = factor.clamp(0.0, 1.0);
        self
    }

    pub fn max_visible_toasts(mut self, n: usize) -> Self {
        self.max_visible_toasts = n.max(1);
        self
    }

    pub fn width(mut self, width: impl Into<gpui::Pixels>) -> Self {
        self.width = width.into();
        self
    }

    /// Distance from the window edge.
    pub fn inset(mut self, inset: impl Into<gpui::Pixels>) -> Self {
        self.inset = inset.into();
        self
    }
}

impl Default for ToastViewport {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ToastViewport {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let all: Vec<ToastData> = match cx.try_global::<ToastHub>() {
            Some(hub) => hub.store.read(cx).toasts().to_vec(),
            None => Vec::new(),
        };

        // Show the newest `max_visible_toasts`.
        let skip = all.len().saturating_sub(self.max_visible_toasts);
        let toasts: Vec<ToastData> = all.into_iter().skip(skip).collect();

        let mut region = gpui::div().absolute().flex().flex_col().gap(self.gap);

        region = if self.placement.is_top() {
            region.top(self.inset)
        } else {
            region.bottom(self.inset)
        };

        region = match self.placement {
            ToastPlacement::TopStart | ToastPlacement::BottomStart => region.left(self.inset),
            ToastPlacement::TopEnd | ToastPlacement::BottomEnd => region.right(self.inset),
            // Centred placements stretch and centre their children.
            ToastPlacement::Top | ToastPlacement::Bottom => region.left_0().right_0().items_center(),
        };

        // The newest toast is at the end, so depth counts back from there.
        let width = self.width;
        let scale = self.scale_factor;
        let last = toasts.len().saturating_sub(1);
        region.children(
            toasts
                .into_iter()
                .enumerate()
                .map(move |(i, t)| toast_card(t, width, last - i, scale)),
        )
    }
}

fn toast_card(
    t: ToastData,
    width: gpui::Pixels,
    depth: usize,
    scale_factor: f32,
) -> gpui::AnyElement {
    // Each step back shrinks the card by `scale_factor`, expressed as a
    // horizontal inset since a div cannot be scaled.
    let shrink = (1.0 - scale_factor * depth as f32).clamp(0.5, 1.0);
    let width = gpui::px(f32::from(width) * shrink);
    ToastCardEl { t, width }.into_any_element()
}

#[derive(IntoElement)]
struct ToastCardEl {
    t: ToastData,
    width: gpui::Pixels,
}

impl RenderOnce for ToastCardEl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let sem = cx.role(self.t.color);

        let mut card = gpui::div()
            .w(self.width)
            .flex()
            .items_start()
            .gap(px(10.))
            .px(px(12.))
            .py(px(10.))
            .rounded(crate::util::container_radius(cx))
            .bg(colors.overlay.background)
            .text_color(colors.overlay.foreground)
            .border(cx.layout().border_width)
            .border_color(colors.border)
            .when(!cx.layout().overlay_shadow.is_empty(), |c| {
                c.shadow(cx.layout().overlay_shadow.clone())
            })
            .overflow_hidden()
            // accent bar
            .child(
                gpui::div()
                    .w(px(4.))
                    .min_h(px(36.))
                    .rounded_full()
                    .bg(sem.color)
                    .flex_shrink_0(),
            );

        let mut text_col = gpui::div().flex().flex_col().gap(px(2.)).flex_1().min_w_0();
        text_col = text_col.child(
            gpui::div()
                .text_size(px(13.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .truncate()
                .child(self.t.title.to_string()),
        );
        if let Some(desc) = &self.t.description {
            text_col = text_col.child(
                gpui::div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(
                        colors.muted,
                    )
                    .child(desc.to_string()),
            );
        }
        card = card.child(text_col);

        if self.t.closable {
            let id = self.t.id;
            let mut close_btn = gpui::div()
                .id(gpui::ElementId::Name(format!("toast-close-{id}").into()))
                .flex()
                .items_center()
                .justify_center()
                .size(px(22.))
                .rounded_full()
                .cursor_pointer();
            let hover_bg = colors.default.soft_hover();
            close_btn = close_btn.hover(move |s| s.bg(hover_bg));
            close_btn = close_btn.on_click(move |_, _, cx| dismiss_toast(id, cx));
            card = card.child(
                close_btn.child(
                    gpui::svg()
                        .size(px(11.))
                        .path(icons::CLOSE)
                        .text_color(
                            colors.muted,
                        ),
                ),
            );
        }

        crate::anim::entering_zoom(
            card,
            gpui::ElementId::Name(format!("toast-anim-{}", self.t.id).into()),
            crate::anim::ZoomBox::panel(px(10.), crate::util::container_radius(cx))
                .padding_x(px(12.))
                .sized(self.width),
            cx,
        )
    }
}






