//! Avatar — port of `@heroui/avatar`.

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    prelude::*, px, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

/// `Avatar.Image.onError` — v3's `(event) => void`, with no event payload to
/// hand over, exactly the shape `Table.onLoadMore` and `Input.onClear` use.
type OnImageError = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Per-`src` image state, keyed in the window. The img element paints the
/// image, the fallback, or nothing depending on what its loader returns, so
/// the one thing that has to survive frames is whether the load failed and
/// whether `delay_ms`' window has passed.
#[derive(Default)]
struct AvatarImageState {
    /// The load has failed; `on_error` has been fired exactly once.
    errored: bool,
    /// The `delay_ms` window (if any) has elapsed since the error.
    ready: bool,
}

/// Visual style of an avatar fallback (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarVariant {
    /// Solid fill in the avatar color.
    #[default]
    Default,
    /// The color at 15% with colored initials.
    Soft,
}

impl AvatarVariant {
    pub const ALL: [AvatarVariant; 2] = [AvatarVariant::Default, AvatarVariant::Soft];

    pub fn label(self) -> &'static str {
        match self {
            AvatarVariant::Default => "Default",
            AvatarVariant::Soft => "Soft",
        }
    }
}

/// HeroUI Avatar: image or name-initials fallback.
#[derive(IntoElement)]
pub struct Avatar {
    name: SharedString,
    src: Option<SharedString>,
    on_error: Option<OnImageError>,
    /// `Avatar.Fallback.delayMs`, in milliseconds.
    fallback_delay_ms: Option<u64>,
    /// Edge length, set by [`Avatar::size`]. v3 has no custom-pixel prop.
    size_px: gpui::Pixels,
    /// Whether [`Avatar::size`] was `Sm`, which rounds one step tighter.
    small: bool,
    color: Color,
    variant: AvatarVariant,
}

impl Avatar {
    pub fn new() -> Self {
        Self {
            name: "".into(),
            src: None,
            on_error: None,
            fallback_delay_ms: None,
            size_px: px(40.),
            small: false,
            color: Color::Default,
            variant: AvatarVariant::Default,
        }
    }

    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = name.into();
        self
    }

    pub fn variant(mut self, variant: AvatarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Asset path for the avatar image (`src`).
    pub fn src(mut self, src: impl Into<SharedString>) -> Self {
        self.src = Some(src.into());
        self
    }

    /// `Avatar.Image.onError` — callback when the image fails to load. The
    /// fallback initials replace the image on that same failure.
    pub fn on_error(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_error = Some(Arc::new(f));
        self
    }

    /// `Avatar.Fallback.delayMs` — hold the fallback back this many
    /// milliseconds after the image fails, so a slow load does not flash the
    /// initials behind it (v3: "Delay before showing fallback (prevents
    /// flash)").
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.fallback_delay_ms = Some(ms);
        self
    }

    pub fn size(mut self, size: herogpui_core::Size) -> Self {
        self.size_px = match size {
            herogpui_core::Size::Sm => px(32.),
            herogpui_core::Size::Md => px(40.),
            herogpui_core::Size::Lg => px(48.),
        };
        // `.avatar--sm` is `rounded-2xl` where the other two are `rounded-3xl`:
        // at 32px a 24px radius would be all but a circle, so v3 steps it down.
        self.small = size == herogpui_core::Size::Sm;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
}

impl Default for Avatar {
    fn default() -> Self {
        Self::new()
    }
}

fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let mut out = String::new();
    for w in words.iter().take(2) {
        if let Some(c) = w.chars().next() {
            out.extend(c.to_uppercase());
        }
    }
    if out.is_empty() {
        "?".to_owned()
    } else {
        out
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let sem = cx.role(self.color);
        let neutral = self.color == Color::Default;
        let (bg, fg) = match self.variant {
            AvatarVariant::Default if neutral => {
                (cx.colors().surface_tertiary, cx.colors().foreground)
            }
            AvatarVariant::Default => (sem.color, sem.foreground),
            AvatarVariant::Soft if neutral => (cx.colors().default.soft(), cx.colors().muted),
            AvatarVariant::Soft => (sem.soft(), sem.soft_foreground()),
        };
        // `.avatar__fallback` is `text-sm`, not a share of the box.
        let font = px(14.);

        let mut el = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size_px)
            .rounded(if self.small {
                crate::util::soft_radius(cx)
            } else {
                crate::util::control_radius(cx)
            })
            .bg(bg)
            .text_color(fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::MEDIUM)
            .overflow_hidden()
            .flex_shrink_0();

        match self.src {
            Some(path) => {
                // The resource goes through gpui's own `&str` classification
                // (a parseable URI is fetched, anything else is embedded), so
                // this loader watches the same load a plain `img(src)` would.
                let gpui::ImageSource::Resource(resource) = gpui::ImageSource::from(path.clone())
                else {
                    unreachable!("a plain string src is always a Resource")
                };
                let key = ElementId::Name(format!("avatar-image:{path}").into());
                let state = window.use_keyed_state(key, cx, |_, _| AvatarImageState::default());
                let on_error = self.on_error.clone();
                let fallback_delay_ms = self.fallback_delay_ms;
                // The img source is the loader itself, so it *sees* the load:
                // gpui's `request_layout` draws the image when the loader
                // returns `Some(Ok)`, the `with_fallback` element on
                // `Some(Err)`, and nothing on `None` -- which is how the
                // `ready` latch turns the error into nothing until `delay_ms`
                // has passed. `errored` stays in keyed state because a
                // per-render cell is a frame long and the error is not.
                let loader = move |window: &mut Window, cx: &mut App| {
                    // `use_asset` (not `get_asset`): it is the call that
                    // arranges a redraw once the load settles.
                    let got = window.use_asset::<gpui::ImgResourceLoader>(&resource, cx);
                    match got {
                        Some(Ok(_)) => got,
                        None => None,
                        Some(Err(_)) if state.read(cx).ready => got,
                        Some(Err(_)) => {
                            if !state.read(cx).errored {
                                let weak = state.downgrade();
                                let on_error = on_error.clone();
                                // The latch, the callback and the delay all
                                // run outside the layout phase, from the
                                // window's own async context.
                                window
                                    .spawn(cx, async move |cx| {
                                        let _ = cx.update(|window, cx| {
                                            let first = weak
                                                .update(cx, |s, _| {
                                                    let first = !s.errored;
                                                    s.errored = true;
                                                    first
                                                })
                                                .unwrap_or(false);
                                            // only the transition reports
                                            if first {
                                                if let Some(on_error) = &on_error {
                                                    on_error(window, cx);
                                                }
                                            }
                                        });
                                        if let Some(ms) = fallback_delay_ms {
                                            cx.background_executor()
                                                .timer(Duration::from_millis(ms))
                                                .await;
                                        }
                                        weak.update(cx, |s, cx| {
                                            s.ready = true;
                                            cx.notify();
                                        })
                                        .ok();
                                    })
                                    .detach();
                            }
                            None
                        }
                    }
                };
                let initials = initials(&self.name);
                // `.avatar__image` is `absolute inset-0 aspect-square
                // size-full`; the fallback replaces the image inside that box
                // and centers like `.avatar__fallback` does. The text color
                // and size are set on the replacement, which gpui does not
                // inherit into a substituted element.
                el = el.child(
                    gpui::img(loader)
                        .with_fallback(move || {
                            gpui::div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_full()
                                .text_color(fg)
                                .text_size(font)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(initials.clone())
                                .into_any_element()
                        })
                        .size_full(),
                );
            }
            None => {
                el = el.child(initials(&self.name));
            }
        }

        el
    }
}
