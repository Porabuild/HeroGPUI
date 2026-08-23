//! Toast — port of `@heroui/toast` (v3).
//!
//! Create the store once (`toast_store(cx)`), render [`ToastViewport`] — the
//! equivalent of `Toast.Provider` — from your root view, and fire toasts
//! anywhere with [`Toast::push`].
//!
//! v3 names the colour prop `variant`, and its values are the semantic colour
//! roles, so [`Color`] is the variant type here.

use std::time::Duration;

use gpui::{
    prelude::*, px, App, Entity, Global, IntoElement, RenderOnce, SharedString, Styled,
    Subscription, Window,
};
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

/// What a toast's action button, or its `onClose`, runs.
pub type ToastHandler = std::sync::Arc<dyn Fn(&mut App) + 'static>;

/// One toast's data.
#[derive(Clone)]
pub struct ToastData {
    pub id: u64,
    pub color: Color,
    pub title: SharedString,
    pub description: Option<SharedString>,
    pub closable: bool,
    /// `indicator` — the glyph before the text.
    ///
    /// Two states in one prop, as in v3: left alone it is the variant's own
    /// glyph, and `indicator={null}` hides it. `indicator_set` is which of the
    /// two an empty `indicator` means.
    pub indicator: Option<SharedString>,
    pub indicator_set: bool,
    /// `isLoading` — a spinner stands in for the indicator.
    pub is_loading: bool,
    /// `actionProps` — a button inside the toast: its label and its handler.
    pub action: Option<(SharedString, ToastHandler)>,
    /// `onClose` — run when the toast goes away, however it goes.
    pub on_close: Option<ToastHandler>,
}

/// Entity holding the active toasts — v3's `ToastQueue`.
pub struct ToastStore {
    toasts: Vec<ToastData>,
    next_id: u64,
    /// `pauseAll` / `resumeAll`. Every toast's timer reads this on each tick,
    /// which is why the timer ticks rather than sleeping once: a gpui timer
    /// cannot be cancelled, so a paused toast is one whose clock stops moving.
    paused: bool,
}

impl ToastStore {
    fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 1,
            paused: false,
        }
    }

    /// `ToastQueue.subscribe` — run `f` whenever the queue changes.
    ///
    /// v3 returns an unsubscribe function; gpui returns a `Subscription` whose
    /// drop does the same, so the caller keeps it for as long as it wants the
    /// callback.
    pub fn subscribe(
        store: &Entity<Self>,
        cx: &mut App,
        mut f: impl FnMut(&mut App) + 'static,
    ) -> Subscription {
        cx.observe(store, move |_, cx| f(cx))
    }

    pub fn toasts(&self) -> &[ToastData] {
        &self.toasts
    }

    /// `pauseAll` — stop every toast's dismissal clock.
    pub fn pause_all(&mut self) {
        self.paused = true;
    }

    /// `resumeAll` — start them again.
    pub fn resume_all(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// `add` — queue a toast and return its id, v3's toast key.
    pub fn add(&mut self, data: ToastData) -> u64 {
        self.insert(data)
    }

    /// `close` — drop one toast by id.
    pub fn close(&mut self, id: u64) {
        self.dismiss(id);
    }

    /// `clear` — drop all of them.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// The `onClose` of the toast with this id, so a caller closing a toast runs
    /// the same handler the timer would have.
    fn on_close(&self, id: u64) -> Option<ToastHandler> {
        self.toasts
            .iter()
            .find(|t| t.id == id)
            .and_then(|t| t.on_close.clone())
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
    cx.set_global(ToastHub {
        store: store.clone(),
    });
    store
}

/// v3's default toast timeout: four seconds, and `timeout: 0` for one that
/// stays until it is closed.
pub const DEFAULT_TOAST_TIMEOUT: Duration = Duration::from_secs(4);

/// Builder for a toast notification.
pub struct Toast {
    color: Color,
    title: SharedString,
    description: Option<SharedString>,
    closable: bool,
    indicator: Option<SharedString>,
    indicator_set: bool,
    is_loading: bool,
    action: Option<(SharedString, ToastHandler)>,
    on_close: Option<ToastHandler>,
    timeout: Option<Duration>,
}

impl Toast {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            color: Color::Accent,
            title: title.into(),
            description: None,
            closable: true,
            indicator: None,
            indicator_set: false,
            is_loading: false,
            action: None,
            on_close: None,
            timeout: Some(DEFAULT_TOAST_TIMEOUT),
        }
    }

    /// `toast.success(..)` — the same toast in the success variant.
    pub fn success(title: impl Into<SharedString>) -> Self {
        Self::new(title).variant(Color::Success)
    }

    /// `toast.danger(..)`, and the `error` message of `toast.promise`.
    pub fn error(title: impl Into<SharedString>) -> Self {
        Self::new(title).variant(Color::Danger)
    }

    /// The `loading` message of `toast.promise`: a spinner, and no timeout, so
    /// the caller closes it when the work finishes.
    pub fn loading(title: impl Into<SharedString>) -> Self {
        Self::new(title).is_loading(true).timeout(Duration::ZERO)
    }

    pub fn description(mut self, d: impl Into<SharedString>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// `indicator` — the glyph before the text.
    ///
    /// Left alone it is the variant's own; `None` hides it, which is v3's
    /// `indicator={null}`.
    pub fn indicator(mut self, icon: impl Into<Option<SharedString>>) -> Self {
        self.indicator = icon.into();
        self.indicator_set = true;
        self
    }

    /// `isLoading` — a spinner in place of the indicator. Pair it with a zero
    /// `timeout` for a toast that waits on something.
    pub fn is_loading(mut self, v: bool) -> Self {
        self.is_loading = v;
        self
    }

    /// `actionProps` — a button in the toast. v3 passes `{children, onPress}`;
    /// here that is the label and the handler.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        on_press: impl Fn(&mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), std::sync::Arc::new(on_press)));
        self
    }

    /// `onClose` — run when the toast goes away, whether it timed out or was
    /// dismissed.
    pub fn on_close(mut self, f: impl Fn(&mut App) + 'static) -> Self {
        self.on_close = Some(std::sync::Arc::new(f));
        self
    }

    /// `timeout` — how long the toast stays. `Duration::ZERO` is v3's
    /// `timeout: 0`: it stays until something closes it.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
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

    /// Pushes the toast, and starts its clock unless the timeout is zero.
    ///
    /// `duration` overrides [`Self::timeout`], which is how the caller that
    /// spells the timeout at the push site keeps working.
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
                indicator: self.indicator.clone(),
                indicator_set: self.indicator_set,
                is_loading: self.is_loading,
                action: self.action.clone(),
                on_close: self.on_close.clone(),
            });
            cx.notify();
            pushed
        });

        // A zero timeout is v3's persistent toast, and so is `Some(ZERO)` from
        // the builder: either way there is no clock to start.
        let dur = duration.or(self.timeout).unwrap_or(Duration::ZERO);
        if !dur.is_zero() {
            let weak = store.downgrade();
            let on_close = self.on_close.clone();
            cx.spawn(async move |cx: &mut gpui::AsyncApp| {
                // Tick instead of sleeping once: `pauseAll` has to be able to
                // stop the clock, and a gpui timer cannot be cancelled. The
                // tick is a tenth of a second, so a paused toast stays put
                // within a frame of being asked to.
                const TICK: Duration = Duration::from_millis(100);
                let mut left = dur;
                loop {
                    cx.background_executor().timer(TICK).await;
                    let Some(store) = weak.upgrade() else { return };
                    // Update through AsyncApp returns a Result; dropping is
                    // fine since a missing entity just means the app closed.
                    let Ok(finished) = store.update(cx, |s, cx| {
                        if s.paused {
                            return false;
                        }
                        // Gone already -- closed by hand -- so the clock stops
                        // and the handler is left to whoever closed it.
                        if !s.toasts.iter().any(|t| t.id == id) {
                            return true;
                        }
                        left = left.saturating_sub(TICK);
                        if left.is_zero() {
                            s.dismiss(id);
                            cx.notify();
                            return true;
                        }
                        false
                    }) else {
                        return;
                    };
                    if finished {
                        break;
                    }
                }
                if let Some(cb) = on_close {
                    let _ = cx.update(|cx| cb(cx));
                }
            })
            .detach();
        }

        id
    }
}

/// Convenience free function (manual dismissal).
pub fn push_toast(toast: Toast, cx: &mut App) -> u64 {
    toast.timeout(Duration::ZERO).push(None, cx)
}

/// Dismisses one toast by id, running its `onClose`.
pub fn dismiss_toast(id: u64, cx: &mut App) {
    let store = toast_store(cx);
    let on_close = store.update(cx, |s, cx| {
        let cb = s.on_close(id);
        s.dismiss(id);
        cx.notify();
        cb
    });
    if let Some(cb) = on_close {
        cb(cx);
    }
}

/// `toast.clear()` — closes every toast.
pub fn clear_toasts(cx: &mut App) {
    let store = toast_store(cx);
    store.update(cx, |s, cx| {
        s.clear();
        cx.notify();
    });
}

/// `toast.pauseAll()` / `toast.resumeAll()` — stops and restarts every clock.
pub fn pause_toasts(paused: bool, cx: &mut App) {
    let store = toast_store(cx);
    store.update(cx, |s, cx| {
        if paused {
            s.pause_all();
        } else {
            s.resume_all();
        }
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
            ToastPlacement::Top | ToastPlacement::Bottom => {
                region.left_0().right_0().items_center()
            }
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
    let width = px(f32::from(width) * shrink);
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
            .gap(px(6.))
            .px(px(16.))
            .py(px(12.))
            .rounded(crate::util::container_radius(cx))
            .bg(colors.surface.background)
            .text_color(colors.surface.foreground)
            .border(cx.layout().border_width)
            .border_color(colors.border)
            .when(!cx.layout().overlay_shadow.is_empty(), |c| {
                c.shadow(cx.layout().overlay_shadow.clone())
            })
            .overflow_hidden();

        // `.toast__indicator` — `flex shrink-0 items-center justify-center p-1`
        // at `size-4`, in the variant's soft foreground. v3 draws a glyph here;
        // this port used to draw a coloured bar, which is not in the stylesheet
        // at all.
        if self.t.is_loading {
            card = card.child(
                gpui::div().flex().flex_shrink_0().p(px(4.)).child(
                    crate::spinner::Spinner::new(gpui::ElementId::Name(
                        format!("toast-spinner-{}", self.t.id).into(),
                    ))
                    .size(herogpui_core::Size::Sm),
                ),
            );
        } else if let Some(icon) = self.t.indicator.clone().or_else(|| {
            // Not set at all means the variant's own glyph; set to nothing
            // means v3's `indicator={null}`.
            if self.t.indicator_set {
                None
            } else {
                default_indicator(self.t.color).map(SharedString::from)
            }
        }) {
            card = card.child(
                gpui::div()
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .p(px(4.))
                    .child(
                        gpui::svg()
                            .size(px(16.))
                            .path(icon)
                            .text_color(sem.soft_foreground()),
                    ),
            );
        }

        // `.toast__content` -- the title and description column, beside the
        // indicator and inside the card.
        let mut text_col = gpui::div().flex().flex_col().gap(px(2.)).flex_1().min_w_0();
        text_col = text_col.child(
            gpui::div()
                // `.toast__title` is `text-sm leading-5 font-medium`.
                .text_size(px(14.))
                .line_height(px(20.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .truncate()
                .child(self.t.title.to_string()),
        );
        if let Some(desc) = &self.t.description {
            text_col = text_col.child(
                gpui::div()
                    // `.toast__description` is `text-sm text-muted`.
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .text_color(colors.muted)
                    .child(desc.to_string()),
            );
        }
        card = card.child(text_col);

        // `.toast__action` — the button v3 configures with `actionProps`.
        if let Some((label, on_press)) = self.t.action.clone() {
            let id = self.t.id;
            card = card.child(
                crate::button::Button::new(gpui::ElementId::Name(
                    format!("toast-action-{id}").into(),
                ))
                .label(label)
                .variant(herogpui_core::Variant::Secondary)
                .size(herogpui_core::Size::Sm)
                .on_press(move |_, _, cx| {
                    on_press(cx);
                    // v3's action closes the toast it belongs to.
                    dismiss_toast(id, cx);
                }),
            );
        }

        if self.t.closable {
            let id = self.t.id;
            let mut close_btn = gpui::div()
                .id(gpui::ElementId::Name(format!("toast-close-{id}").into()))
                .flex()
                .items_center()
                .justify_center()
                // `.toast__close-button` is `size-5` with `sm:border
                // border-border sm:bg-overlay`, and its icon follows the close
                // button's own `size-3`.
                .size(px(20.))
                .border(cx.layout().border_width)
                .border_color(colors.border)
                .bg(colors.overlay.background)
                .rounded(crate::util::small_radius(cx))
                .cursor_pointer();
            let hover_bg = colors.default.soft_hover();
            close_btn = close_btn.hover(move |s| s.bg(hover_bg));
            close_btn = close_btn.on_click(move |_, _, cx| dismiss_toast(id, cx));
            card = card.child(
                close_btn.child(
                    gpui::svg()
                        .size(px(12.))
                        .path(icons::CLOSE)
                        .text_color(colors.muted),
                ),
            );
        }

        crate::anim::entering_zoom(
            card,
            gpui::ElementId::Name(format!("toast-anim-{}", self.t.id).into()),
            crate::anim::ZoomBox::panel(px(10.), crate::util::container_radius(cx))
                .padding_x(px(16.))
                .sized(self.width),
            crate::anim::Motion::LIST_IN,
            cx,
        )
    }
}

/// The glyph a variant shows when the caller names none —
/// `[data-slot="toast-default-icon"]` in v3, coloured by
/// `.toast--<variant> .toast__indicator`.
fn default_indicator(color: Color) -> Option<&'static str> {
    match color {
        Color::Success => Some(icons::CHECK),
        Color::Warning => Some(icons::ALERT_TRIANGLE),
        Color::Danger => Some(icons::CLOSE_CIRCLE),
        // A default or accent toast carries no status, so it carries no glyph.
        _ => None,
    }
}
