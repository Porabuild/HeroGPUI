//! HeroGPUI Gallery — living documentation for the HeroGPUI component
//! library, mirroring heroui.com/docs.

mod app;
mod assets;
mod pages;

use gpui::{
    prelude::*, px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};
use herogpui_theme::ThemeProvider;

use crate::app::Gallery;
use crate::pages::Page;

fn initial_page() -> Page {
    if let Ok(arg) = std::env::var("HEROGPUI_PAGE") {
        for section in herogpui_gallery_pages::all_pages() {
            if section_title(section) == arg {
                return section;
            }
        }
    }
    Page::Introduction
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

            let bounds = Bounds::centered(None, size(px(1280.), px(820.)), cx);
            let start_page = page;
            // `HEROGPUI_UNFOCUSED=1` opens the window without taking focus, so
            // the screenshot and smoke scripts do not interrupt whatever you are
            // doing. The window still renders, which is what those scripts need;
            // only the activation is skipped.
            let unfocused = std::env::var("HEROGPUI_UNFOCUSED")
                .map(|v| v == "1")
                .unwrap_or(false);
            cx.open_window(
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

            cx.activate(true);
        });
}
