//! HeroGPUI Gallery — living documentation for the HeroGPUI component
//! library, mirroring heroui.com/docs.
//!
//! Exposed as a library, in addition to this package's native `main.rs`
//! binary, so the wasm web crate (`crates/herogpui-web`) can reuse the same
//! `Gallery` shell, pages and demo state rather than reimplementing them.
//! Everything platform-specific (reading `HEROGPUI_*` environment variables
//! versus a URL query string, opening a native window versus a web canvas)
//! stays out of this crate, in each entry point's own binary.

pub mod app;
pub mod assets;
pub mod control;
mod highlight;
pub mod pages;

use crate::pages::Page;

/// The page whose nav title is `name`, if there is one.
pub fn page_named(name: &str) -> Option<Page> {
    herogpui_gallery_pages::all_pages()
        .into_iter()
        .find(|p| section_title(*p) == name)
}

/// The page whose website slug is `slug`, if there is one (e.g.
/// `"date-picker"` for the nav title `"Date Picker"`).
///
/// A slug is the nav title lowercased with every non-alphanumeric run
/// collapsed to a single `-` and edge separators trimmed -- the same scheme
/// the documentation generator (`web/scripts/lib/rust.mjs` `slugify`) uses
/// for component URLs, so the website's `?story=` values resolve here
/// without a hand-maintained mapping.
pub fn page_from_slug(slug: &str) -> Option<Page> {
    herogpui_gallery_pages::all_pages()
        .into_iter()
        .find(|p| slug_of_title(section_title(*p)) == slug)
}

/// The website slug for a nav title. Mirrors `slugify` in
/// `web/scripts/lib/rust.mjs` for every nav title in use (Title Case words,
/// at most one `&`); the camel-case-splitting rules there never fire on
/// these titles.
fn slug_of_title(title: String) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut previous_was_separator = true;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            slug.push('-');
            previous_was_separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
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
