#![cfg(target_family = "wasm")]

//! Browser entry point for the HeroGPUI gallery, compiled to WebAssembly.
//!
//! Reuses `herogpui_gallery`'s pages, demo state and asset source (see
//! `gallery/src/lib.rs`) so the web build shows the same gallery as the
//! native one, rather than a separate reimplementation. This crate supplies
//! only what a browser needs instead of an environment:
//!
//! - **Deep linking.** The native binary reads `HEROGPUI_PAGE` from the
//!   environment; a browser has no environment. [`run`] takes the page name
//!   as an ordinary argument, so a host page can call `wasm.run("Button")`
//!   directly, and otherwise falls back to this page's own query string:
//!   `?story=<slug>` (the website's catalog slug, e.g. `date-picker`) or
//!   `?page=<Nav Title>` (the same spelling as the native `HEROGPUI_PAGE`
//!   value), both resolved through `herogpui_gallery`.
//! - **The `wasm-bindgen`/GPUI web platform wiring** `gpui_web` needs
//!   (`gpui_platform::web_init`), which does not exist -- and is not
//!   needed -- on the native target.
//! - **App lifetime.** The web platform has no blocking run loop, so the
//!   application is started with [`gpui::Application::run_embedded`] and the
//!   returned handle is kept in [`APPLICATION`] -- see that documentation
//!   for why plain `run` tears the whole app (canvas included) down as soon
//!   as the launch callback returns.
//!
//! The whole crate is `#![cfg(target_family = "wasm")]`. Every item below is
//! wasm-only anyway, and gating the crate once is what lets it be an ordinary
//! workspace member: a native `cargo check --workspace` compiles it as an
//! empty library rather than skipping it, so the manifest, the workspace
//! dependency versions and this file's callers into `herogpui_gallery` stay
//! under the native gate too.
//!
//! Modelled on `longbridge/gpui-component`'s `crates/story-web` (a working,
//! deployed consumer of a GPUI web platform), cross-checked against
//! `gpui_web`'s own `examples/hello_web`.

use std::cell::RefCell;

use gpui::{prelude::*, App, ApplicationHandle, TitlebarOptions, WindowHandle, WindowOptions};
use herogpui_gallery::pages::Page;
use herogpui_gallery::{app::Gallery, assets, control, page_from_slug, page_named};
use herogpui_theme::{Theme, ThemeProvider};
use wasm_bindgen::prelude::*;

thread_local! {
    /// Keeps the launched application alive for the lifetime of the page.
    ///
    /// `WebPlatform::run` starts graphics initialization asynchronously and
    /// invokes the launch callback from a spawned future, then returns --
    /// there is no blocking run loop and no stack frame holding the app
    /// state the way `Application::run`'s documentation assumes. Once that
    /// future completes, the closure it holds is the only strong reference
    /// left; dropping it drops the app, its window, and with them the
    /// canvas. Storing the `run_embedded` handle here is what keeps the
    /// gallery on screen.
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
    static WINDOW: RefCell<Option<WindowHandle<Gallery>>> = const { RefCell::new(None) };
}

/// One `key` from this page's URL query string (`?key=value`), if present.
///
/// Native reads `HEROGPUI_PAGE`/`HEROGPUI_THEME` from the environment; a
/// browser has no environment, so the query string is the equivalent input
/// channel for a plain page load (as opposed to the `run` argument, which
/// covers a host page embedding this gallery and choosing the story itself).
fn query_param(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get(key)
}

/// The page selected by this page's own query string.
///
/// The website embeds the gallery with `?story=<catalog slug>`
/// (`herogpui_gallery::page_from_slug`); `?page=<Nav Title>` is kept as the
/// native-parity spelling of the same lookup (`herogpui_gallery::page_named`,
/// the exact match `HEROGPUI_PAGE` uses). Either spelling deep-links.
fn page_from_query() -> Option<Page> {
    if let Some(slug) = query_param("story") {
        if let Some(page) = page_from_slug(&slug) {
            return Some(page);
        }
    }
    query_param("page").as_deref().and_then(page_named)
}

/// `?theme=dark`, matching the native `HEROGPUI_THEME` value.
fn theme_from_query() -> Theme {
    if query_param("theme").as_deref() == Some("dark") {
        Theme::dark()
    } else {
        Theme::light()
    }
}

/// Runs the gallery in the page's `<canvas>`.
///
/// `page`, if given, is looked up by nav title the same way as the native
/// `HEROGPUI_PAGE` environment variable (e.g. `"Button"`, `"Date Picker"`),
/// falling back to the website's catalog slug for the same page (e.g.
/// `"date-picker"`), so a host page embedding this gallery can deep-link
/// without relying on its own URL.
///
/// `dark`, if given, picks the initial theme directly; this is how an
/// embedding loader passes the host page's appearance so the first frame is
/// already right. Absent, the page's own `?theme=dark` query string is used.
///
/// Anything not covered by the arguments falls back to this page's own
/// `?story=`/`?page=`/`?theme=` query string, then the introduction page and
/// light theme.
#[wasm_bindgen]
pub fn run(page: Option<String>, dark: Option<bool>) {
    // Installs the panic hook and console logger `gpui_web` provides (see
    // `crates/gpui_web/src/logging.rs`) so a Rust panic surfaces in the
    // browser console instead of silently hanging the page.
    gpui_platform::web_init();

    let page = page
        .as_deref()
        .and_then(page_named)
        .or_else(|| page.as_deref().and_then(page_from_slug))
        .or_else(page_from_query)
        .unwrap_or(Page::Introduction);
    let theme = match dark {
        Some(true) => Theme::dark(),
        Some(false) => Theme::light(),
        None => theme_from_query(),
    };
    let section = query_param("section").unwrap_or_else(|| "Usage".to_owned());
    let preview_only = query_param("preview").as_deref() == Some("component");

    // Single-threaded on purpose: the multi-threaded web platform runs its
    // background executors on web workers over a shared wasm memory, which a
    // browser only grants inside a cross-origin-isolated context. See the
    // wasm table in `.cargo/config.toml`.
    let app = gpui_platform::single_threaded_web().with_assets(assets::Assets);
    let launch = move |cx: &mut App| {
        // The pinned web platform has no system fonts. Register the bundled
        // OFL fonts before opening the first window. CosmicText selects static
        // weight faces; it does not apply the variable Inter weight axis.
        cx.text_system()
            .add_fonts(vec![
                std::borrow::Cow::Borrowed(include_bytes!("../fonts/Inter-Regular.ttf").as_slice()),
                std::borrow::Cow::Borrowed(include_bytes!("../fonts/Inter-Medium.ttf").as_slice()),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../fonts/Inter-Semibold.ttf").as_slice(),
                ),
                std::borrow::Cow::Borrowed(include_bytes!("../fonts/Inter-Bold.ttf").as_slice()),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../fonts/NotoSansSC-Regular.ttf").as_slice(),
                ),
                std::borrow::Cow::Borrowed(
                    include_bytes!("../fonts/NotoEmoji-Regular.ttf").as_slice(),
                ),
            ])
            .expect("bundled gallery fonts must load");
        ThemeProvider::init_with(theme, cx);
        control::init_section_filter(cx);
        control::set_section_filter(&section, cx);
        control::set_preview_only(preview_only, cx);

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("HeroGPUI — Gallery".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                move |_, cx| {
                    cx.new(move |cx| {
                        let mut g = Gallery::new(cx);
                        g.set_initial_page(page);
                        g
                    })
                },
            )
            .unwrap();
        WINDOW.with(|current| {
            *current.borrow_mut() = Some(window);
        });
        // `HEROGPUI_CONTROL` is unset on the web -- `std::env::var`
        // returns `Err` there, and `control::spawn` already treats that
        // as "no control file", so this is the same no-op it is when a
        // native run omits the variable.
        control::spawn(window, cx);
        cx.activate(true);
    };

    // Embedded platforms must keep the application handle themselves (see
    // `APPLICATION`): `run` would drop the app the moment the launch
    // callback returns, taking the freshly prepared canvas with it.
    APPLICATION.with(|application| {
        *application.borrow_mut() = Some(app.run_embedded(launch));
    });
}

/// Switches the gallery between light and dark after it is running.
///
/// The embedding documentation page calls this when its own appearance
/// changes, so the gallery never sits in a dark page wearing a light theme.
/// Both themes are pre-registered by `ThemeProvider::init_with`, so this is
/// an activation, not a rebuild.
#[wasm_bindgen]
pub fn set_theme(dark: bool) {
    APPLICATION.with(|application| {
        if let Some(handle) = application.borrow().as_ref() {
            handle.update(|cx| {
                cx.global_mut::<ThemeProvider>()
                    .set_active(if dark { "dark" } else { "light" });
                cx.refresh_windows();
            });
        }
    });
}

/// Switches the focused component preview to one named gallery section.
///
/// The documentation page keeps one wasm application alive and calls this
/// when a reader selects another example, avoiding another wasm instance.
#[wasm_bindgen]
pub fn set_preview_section(section: String) {
    APPLICATION.with(|application| {
        if let Some(handle) = application.borrow().as_ref() {
            handle.update(|cx| {
                control::set_section_filter(&section, cx);
                WINDOW.with(|window| {
                    if let Some(window) = window.borrow().as_ref() {
                        let _ = window.update(cx, |_, _, cx| cx.notify());
                    }
                });
            });
        }
    });
}
