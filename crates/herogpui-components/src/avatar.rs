//! Avatar — port of `@heroui/avatar` (v3.2.4, Radix Avatar 1.1.11 semantics).

use std::sync::Arc;
use std::time::Duration;

use gpui::{
    px, App, ElementId, ImageCacheError, ImageSource, ImgResourceLoader, IntoElement,
    ParentElement, RenderImage, RenderOnce, Resource, SharedString, Styled, Window,
};
use herogpui_core::Color;
use herogpui_theme::ActiveTheme;

/// `Avatar.Image.onError` — v3's `(event) => void`, with no event payload to
/// hand over, exactly the shape `Table.onLoadMore` and `Input.onClear` use.
type OnImageError = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// `Avatar.Image.onLoad` — same ported shape as [`OnImageError`].
type OnImageLoad = Arc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Per-instance image state, keyed in the window under the avatar's instance
/// id alone. Radix Avatar tracks pending/loaded/errored per component
/// instance: two avatars pointing at the same source fire their own
/// callbacks and run their own `delay_ms` windows, even though gpui
/// deduplicates the underlying asset load per source. The image latches reset
/// on a source change; the fallback timer does not, because `Avatar.Fallback`
/// remains mounted while `Avatar.Image` changes.
#[derive(Clone, Default)]
struct AvatarImageState {
    /// The source identity whose image latches this slot tracks. A different
    /// identity resets those latches and bumps [`AvatarImageState::generation`].
    source: Option<SourceIdentity>,
    /// Bumped on every source change. Tasks spawned for one image identity
    /// must not fire a load latch after the identity changed under them.
    generation: u32,
    /// The mounted fallback's `delay_ms` task is armed (running or done).
    armed: bool,
    /// The fallback's `delay_ms` window (if any) has elapsed since mount.
    delay_elapsed: bool,
    /// The load has failed; `on_error` has been fired exactly once.
    errored: bool,
    /// The load has succeeded; `on_load` has been fired exactly once.
    loaded: bool,
}

/// Fill of an avatar fallback (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarVariant {
    /// The pinned base fill (`bg-default`); the color recolours the initials.
    #[default]
    Default,
    /// The color's soft wash with `text-{role}-soft-foreground` initials.
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
    /// The instance's element id, required at construction. The image
    /// lifecycle state (`delay_ms` window, `on_error`/`on_load` latches) is
    /// keyed under it, so two avatars rendered on one page must carry
    /// distinct ids here or they share one state slot.
    id: ElementId,
    name: SharedString,
    source: Option<ImageSource>,
    custom_source_key: Option<SharedString>,
    on_error: Option<OnImageError>,
    on_load: Option<OnImageLoad>,
    /// `Avatar.Fallback.delayMs`, in milliseconds.
    fallback_delay_ms: Option<u64>,
    /// `Avatar.Fallback` children — an icon, a `+N` counter, any element.
    fallback: Option<gpui::AnyElement>,
    /// `Avatar.Fallback.color` — overrides the parent color for everything
    /// the fallback paints.
    fallback_color: Option<Color>,
    /// Edge length, set by [`Avatar::size`]. v3 has no custom-pixel prop.
    size_px: gpui::Pixels,
    /// Whether [`Avatar::size`] was `Sm`, which rounds one step tighter.
    small: bool,
    /// Whether [`Avatar::size`] was `Lg`, whose fallback text steps up.
    large: bool,
    color: Color,
    variant: AvatarVariant,
}

impl Avatar {
    /// The instance's element id is a constructor argument, not an optional
    /// builder: image lifecycle state is keyed under it, and a defaulted
    /// literal would silently merge same-source siblings into one instance.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            name: "".into(),
            source: None,
            custom_source_key: None,
            on_error: None,
            on_load: None,
            fallback_delay_ms: None,
            fallback: None,
            fallback_color: None,
            size_px: px(40.),
            small: false,
            large: false,
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

    /// `Avatar.Image.src` — a plain string or path loads through gpui's asset
    /// system (a parseable URI is fetched, anything else is an embedded
    /// resource); a [`gpui::ImageSource::Image`], `Render` or `Custom` is the
    /// explicit gpui part for images the app already holds or loads itself.
    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.source = Some(src.into());
        self
    }

    /// Sets the stable logical identity for a custom [`ImageSource`] loader.
    /// Rebuilding an equivalent loader per frame keeps the same lifecycle when
    /// this is omitted; change the key when the loader starts serving a new
    /// logical image so its load and error latches reset.
    pub fn custom_source_key(mut self, key: impl Into<SharedString>) -> Self {
        self.custom_source_key = Some(key.into());
        self
    }

    /// `Avatar.Image.onError` — callback when the image fails to load. The
    /// fallback initials replace the image on that same failure.
    pub fn on_error(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_error = Some(Arc::new(f));
        self
    }

    /// `Avatar.Image.onLoad` — callback when the image has loaded and
    /// replaces the fallback.
    pub fn on_load(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_load = Some(Arc::new(f));
        self
    }

    /// `Avatar.Fallback.delayMs` — hold the fallback back this many
    /// milliseconds **from fallback mount**, so a slow load does not flash the
    /// initials behind it (v3: "Delay before showing fallback (prevents
    /// flash)"). A success renders before the window ends; a failure or source
    /// change inside it waits for the same window — it does not restart it.
    pub fn delay_ms(mut self, ms: u64) -> Self {
        self.fallback_delay_ms = Some(ms);
        self
    }

    /// `Avatar.Fallback` children — replaces the name initials with any
    /// element (an icon, a `+N` counter, custom text).
    pub fn fallback(mut self, content: impl IntoElement) -> Self {
        self.fallback = Some(content.into_any_element());
        self
    }

    /// `Avatar.Fallback.color` — overrides the parent [`Avatar::color`] for
    /// the fallback's soft-foreground text and soft fill. It is its own
    /// documented prop, not an alias of the parent's color.
    pub fn fallback_color(mut self, c: Color) -> Self {
        self.fallback_color = Some(c);
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
        self.large = size == herogpui_core::Size::Lg;
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// The initials `name` renders when no [`Avatar::fallback`] children are
    /// set: the uppercase first characters of the first two words, or `?`
    /// when there is nothing to take a character from.
    pub fn initials(name: &str) -> String {
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
}

/// The stable identity of one avatar's image source. Every resource and
/// image variant compares by value: the location, the image content id, or
/// the render image's allocation id. A `Custom` loader has no stable value
/// identity — a closure rebuilt inline each frame gets a fresh allocation
/// every time — so it shares the instance's single lifecycle slot unless the
/// caller supplies an explicit logical source key.
#[derive(Clone, PartialEq)]
enum SourceIdentity {
    Resource(Resource),
    Image(u64),
    Render(usize),
    Custom(Option<SharedString>),
}

/// What the avatar keys its lifecycle state against this frame.
fn source_identity(
    source: &ImageSource,
    custom_source_key: Option<&SharedString>,
) -> SourceIdentity {
    match source {
        ImageSource::Resource(resource) => SourceIdentity::Resource(resource.clone()),
        ImageSource::Image(image) => SourceIdentity::Image(image.id()),
        ImageSource::Render(image) => SourceIdentity::Render(image.id.0),
        ImageSource::Custom(_) => SourceIdentity::Custom(custom_source_key.cloned()),
    }
}

/// What the source reports this frame: `Some(Ok)` loaded, `Some(Err)` failed,
/// `None` still pending. `use_asset` (not `get_asset`) is the call that also
/// arranges a redraw once a resource load settles.
fn observe_load(
    source: &ImageSource,
    window: &mut Window,
    cx: &mut App,
) -> Option<Result<Arc<RenderImage>, ImageCacheError>> {
    match source {
        ImageSource::Resource(resource) => window.use_asset::<ImgResourceLoader>(resource, cx),
        ImageSource::Image(image) => image.clone().use_render_image(window, cx).map(Ok),
        ImageSource::Render(image) => Some(Ok(image.clone())),
        ImageSource::Custom(loader) => loader(window, cx),
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        // `Avatar.Fallback.color` overrides the parent color for everything
        // the fallback paints; the wiring is explicit so the parent's
        // same-named `color` never silently stands in for it.
        let fb_role = cx.role(self.fallback_color.unwrap_or(self.color));
        // Both variants paint the fallback text `text-{role}-soft-foreground`;
        // for `--default` that resolves to `--default-foreground`.
        let soft_fg = fb_role.soft_foreground(colors.foreground);
        let (bg, fallback_bg) = match self.variant {
            // `.avatar` fills `bg-default` for every color: the color recolours
            // the initials, never the fill.
            AvatarVariant::Default => (colors.default.color, colors.default.color),
            // `.avatar--soft` clears the base fill and the fallback paints
            // `bg-{role}-soft` with the same soft foreground.
            AvatarVariant::Soft => (gpui::transparent_black(), fb_role.soft()),
        };
        // `.avatar__fallback` is `text-sm`; `.avatar--lg .avatar__fallback`
        // steps the fallback text up to `text-base`.
        let font = if self.large { px(16.) } else { px(14.) };
        let radius = if self.small {
            crate::util::soft_radius(cx)
        } else {
            crate::util::control_radius(cx)
        };

        let el = gpui::div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size_px)
            .rounded(radius)
            .bg(bg)
            .text_color(soft_fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::MEDIUM)
            .overflow_hidden()
            .flex_shrink_0();

        let fallback_content: gpui::AnyElement = match self.fallback {
            Some(content) => content,
            None => Avatar::initials(&self.name).into_any_element(),
        };
        // `.avatar__fallback` is `size-full bg-default` and relies on the
        // avatar's `overflow-hidden` to clip its fill to the rounded corner.
        // gpui clips a child to the parent's *rect*, not to its radius, so a
        // square fill here squared off every avatar — the same reason the
        // loaded image below carries the radius.
        let fallback = gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .rounded(radius)
            .bg(fallback_bg)
            .text_color(soft_fg)
            .text_size(font)
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(fallback_content);

        // Keep the image lifecycle slot alive even when the source is absent:
        // image latches must reset when an image is removed and later re-added
        // with the same source, while the mounted fallback timer survives.
        let key = ElementId::Name(format!("{:?}-image", self.id).into());
        let state = window.use_keyed_state(key, cx, |_, _| AvatarImageState::default());
        let source = self.source;
        let identity = source
            .as_ref()
            .map(|source| source_identity(source, self.custom_source_key.as_ref()));
        state.update(cx, |state, _| {
            if state.source.as_ref() != identity.as_ref() {
                let armed = state.armed;
                let delay_elapsed = state.delay_elapsed;
                *state = AvatarImageState {
                    source: identity,
                    generation: state.generation + 1,
                    armed,
                    delay_elapsed,
                    ..AvatarImageState::default()
                };
            }
        });
        let snapshot = state.read(cx).clone();

        // `Avatar.Fallback` stays mounted for the life of this Avatar, so its
        // delay starts once per instance, including when the image is added
        // after the first render.
        if self.fallback_delay_ms.is_some() && !snapshot.armed {
            let weak = state.downgrade();
            let delay_ms = self.fallback_delay_ms.unwrap_or_default();
            window
                .spawn(cx, async move |cx| {
                    // The latch makes a double spawn (two frames before the
                    // task first runs) wait-free.
                    let start = weak
                        .update(cx, |s, _| {
                            if s.armed {
                                None
                            } else {
                                s.armed = true;
                                Some(())
                            }
                        })
                        .unwrap_or(None);
                    if start.is_some() {
                        cx.background_executor()
                            .timer(Duration::from_millis(delay_ms))
                            .await;
                        weak.update(cx, |s, cx| {
                            s.delay_elapsed = true;
                            cx.notify();
                        })
                        .ok();
                    }
                })
                .detach();
        }
        let fallback_visible = self.fallback_delay_ms.is_none() || snapshot.delay_elapsed;

        match source {
            None => {
                if fallback_visible {
                    el.child(fallback)
                } else {
                    el
                }
            }
            Some(source) => {
                let got = observe_load(&source, window, cx);
                match &got {
                    // `Avatar.Image.onLoad` fires once, on the first observed
                    // success, outside the layout phase.
                    Some(Ok(_)) if !snapshot.loaded => {
                        let weak = state.downgrade();
                        let on_load = self.on_load.clone();
                        let generation = snapshot.generation;
                        window
                            .spawn(cx, async move |cx| {
                                let first = weak
                                    .update(cx, |s, _| {
                                        if s.loaded || s.generation != generation {
                                            false
                                        } else {
                                            s.loaded = true;
                                            true
                                        }
                                    })
                                    .unwrap_or(false);
                                if first {
                                    let _ = cx.update(|window, cx| {
                                        if let Some(on_load) = &on_load {
                                            on_load(window, cx);
                                        }
                                    });
                                }
                            })
                            .detach();
                    }
                    // `Avatar.Image.onError` fires once, on the first observed
                    // failure, outside the layout phase.
                    Some(Err(_)) if !snapshot.errored => {
                        let weak = state.downgrade();
                        let on_error = self.on_error.clone();
                        let generation = snapshot.generation;
                        window
                            .spawn(cx, async move |cx| {
                                let first = weak
                                    .update(cx, |s, _| {
                                        if s.errored || s.generation != generation {
                                            false
                                        } else {
                                            s.errored = true;
                                            true
                                        }
                                    })
                                    .unwrap_or(false);
                                if first {
                                    let _ = cx.update(|window, cx| {
                                        if let Some(on_error) = &on_error {
                                            on_error(window, cx);
                                        }
                                    });
                                }
                            })
                            .detach();
                    }
                    _ => {}
                }

                match got {
                    // Success: the image replaces the fallback inside the
                    // `.avatar__image` box.
                    Some(Ok(data)) => el.child(
                        gpui::img(data)
                            .absolute()
                            .inset_0()
                            .size_full()
                            .rounded(radius),
                    ),
                    // Pending and error both keep the fallback box; with a
                    // `delay_ms` window still running the box stays empty.
                    Some(Err(_)) | None => {
                        if fallback_visible {
                            el.child(fallback)
                        } else {
                            el
                        }
                    }
                }
            }
        }
    }
}

// The pinned `.avatar` fills `bg-default` for every color and paints the
// initials `text-{role}-soft-foreground`; a solid role fill or a muted
// fallback looks plausible on screen, so the check is mechanical.
#[cfg(test)]
mod fill_tokens {
    #[test]
    fn the_fills_and_foregrounds_follow_the_pinned_css() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessors.
        let source = include_str!("avatar.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source
                .contains("AvatarVariant::Default => (colors.default.color, colors.default.color)"),
            "the base avatar and its fallback slot must fill `bg-default` \
             (pinned `.avatar` + `.avatar__fallback`)"
        );
        assert!(
            source.contains("AvatarVariant::Soft => (gpui::transparent_black(), fb_role.soft())"),
            "the soft avatar root must be transparent and its fallback slot \
             must fill `bg-{{role}}-soft` (pinned `.avatar--soft`)"
        );
        assert!(
            source.contains("cx.role(self.fallback_color.unwrap_or(self.color))"),
            "`Avatar.Fallback.color` must override the parent color for the \
             fallback painting, not alias it"
        );
        assert!(
            source.contains("let font = if self.large { px(16.) } else { px(14.) }"),
            "`.avatar__fallback` is `text-sm` and `.avatar--lg \
             .avatar__fallback` steps it up to `text-base` (16px)"
        );
        assert!(
            source.contains(".size_full()") && source.contains(".bg(fallback_bg)"),
            "the fallback must be the pinned full-size fallback slot, not a \
             raw child of the root"
        );
        let compact: String = source.split_whitespace().collect();
        assert!(
            compact.contains("gpui::img(data).absolute().inset_0().size_full().rounded(radius)"),
            "the loaded image must carry the avatar radius because not every \
             renderer clips image content at a rounded parent"
        );
        assert!(
            !source.contains("colors().muted"),
            "the soft default initials are `text-default-soft-foreground`, \
             not the muted tone"
        );
        assert!(
            !source.contains("surface_tertiary"),
            "the base avatar fills `bg-default`, not a surface level"
        );
        assert!(
            !source.contains("(fb_role.color, fb_role.foreground)"),
            "the base avatar never paints a solid role fill"
        );
    }
}
