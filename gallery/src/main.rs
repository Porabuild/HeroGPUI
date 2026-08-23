//! HeroGPUI Gallery — living documentation for the HeroGPUI component
//! library, mirroring heroui.com/docs.

mod app;
mod assets;
mod control;
mod pages;

use gpui::{
    prelude::*, px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};
use herogpui_theme::ThemeProvider;

use crate::app::Gallery;
use crate::pages::Page;

/// The page whose nav title is `name`, if there is one.
pub fn page_named(name: &str) -> Option<Page> {
    herogpui_gallery_pages::all_pages()
        .into_iter()
        .find(|p| section_title(*p) == name)
}

fn initial_page() -> Page {
    std::env::var("HEROGPUI_PAGE")
        .ok()
        .and_then(|arg| page_named(&arg))
        .unwrap_or(Page::Introduction)
}

/// Flat list of every page for CLI lookup.
mod herogpui_gallery_pages {
    pub fn all_pages() -> Vec<crate::pages::Page> {
        use crate::pages::{nav_sections, Page};
        let mut out = Vec::new();
        for s in nav_sections() {
            for p in &s.items {
                out.push(*p);
            }
        }
        let _ = std::marker::PhantomData::<Page>;
        out
    }
}

fn section_title(p: Page) -> String {
    // match on the enum's Debug-ish title used by nav (e.g. "Date Picker")
    p.title().to_owned()
}

fn initial_theme() -> herogpui_theme::Theme {
    match std::env::var("HEROGPUI_THEME").as_deref() {
        Ok("dark") => herogpui_theme::Theme::dark(),
        _ => herogpui_theme::Theme::light(),
    }
}

fn main() {
    let page = initial_page();
    let theme = initial_theme();

    Application::new()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            ThemeProvider::init_with(theme, cx);
            control::init_section_filter(cx);

            // `HEROGPUI_WINDOW_SIZE=1200x2000` opens the window at that size.
            // A capture is one PrintWindow of the whole window, so a taller
            // window is more of a page per screenshot -- and a window created
            // oversized keeps its size where a *resize* of a visible one is
            // clamped to the monitor.
            let (w, h) = std::env::var("HEROGPUI_WINDOW_SIZE")
                .ok()
                .and_then(|v| {
                    let (w, h) = v.split_once(['x', 'X'])?;
                    Some((w.trim().parse::<f32>().ok()?, h.trim().parse::<f32>().ok()?))
                })
                .unwrap_or((1280., 820.));
            // Centring clamps to the display, which is exactly what a taller
            // window is trying to escape, so an explicit size gets explicit
            // bounds parked at the top-left.
            let bounds = if std::env::var("HEROGPUI_WINDOW_SIZE").is_ok() {
                Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: size(px(w), px(h)),
                }
            } else {
                Bounds::centered(None, size(px(w), px(h)), cx)
            };
            let start_page = page;
            // `HEROGPUI_UNFOCUSED=1` opens the window without taking focus, so
            // the screenshot and smoke scripts do not interrupt whatever you are
            // doing. The window still renders, which is what those scripts need;
            // only the activation is skipped.
            let unfocused = std::env::var("HEROGPUI_UNFOCUSED")
                .map(|v| v == "1")
                .unwrap_or(false);
            let window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        focus: !unfocused,
                        titlebar: Some(TitlebarOptions {
                            title: Some("HeroGPUI — Gallery".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    move |_, cx| {
                        cx.new(move |cx| {
                            let mut g = Gallery::new(cx);
                            g.set_initial_page(start_page);
                            g
                        })
                    },
                )
                .unwrap();
            // `HEROGPUI_CONTROL=<file>` lets one process serve a whole batch of
            // checks: the page, section, theme and overlay state all come from
            // that file while the app runs.
            control::spawn(window, cx);

            // Activating raises the window and takes the focus, which is the
            // one thing `HEROGPUI_UNFOCUSED=1` is asked not to do: a capture run
            // would interrupt whatever the user is doing, once per page.
            if !unfocused {
                cx.activate(true);
            }
        });
}
