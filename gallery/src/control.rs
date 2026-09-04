//! `HEROGPUI_CONTROL=<file>` — drive the running gallery from a file.
//!
//! Every screenshot and every smoke check used to be its own process: launch,
//! wait for the first frame, act, capture, kill. That is about four seconds of
//! startup per page, which is five minutes for the 73-page sweep and most of the
//! wall-clock of any verification round.
//!
//! With this, one process serves the whole batch. The file is polled (200ms) and
//! re-read whenever it changes; each line is `key=value`:
//!
//! ```text
//! page=Table
//! section=Sorting
//! theme=dark
//! overlays=1
//! ```
//!
//! `section` and `theme` are absent-means-default, so a batch never has to undo
//! the previous step by hand. `seq` is echoed back into `<file>.ack` once the
//! change has been applied *and* a frame has been drawn with it, which is what
//! the driver waits for instead of sleeping and hoping.
use std::path::PathBuf;
use std::time::Duration;

use gpui::{App, AsyncApp, Global, WindowHandle};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::Page;

/// The section filter `doc_page` reads. A global rather than an env var, so it
/// can change while the app runs.
#[derive(Default)]
pub struct SectionFilter(pub Vec<String>);

impl Global for SectionFilter {}

#[derive(Default)]
pub struct PreviewOnly(pub bool);

impl Global for PreviewOnly {}

/// `HEROGPUI_SECTION` seeds it; the control file replaces it.
pub fn init_section_filter(cx: &mut App) {
    let raw = std::env::var("HEROGPUI_SECTION").unwrap_or_default();
    set_section_filter(&raw, cx);
}

pub fn set_section_filter(raw: &str, cx: &mut App) {
    cx.set_global(SectionFilter(parse_sections(raw)));
}

pub fn set_preview_only(preview: bool, cx: &mut App) {
    cx.set_global(PreviewOnly(preview));
}

pub fn preview_only(cx: &App) -> bool {
    cx.try_global::<PreviewOnly>()
        .is_some_and(|preview| preview.0)
}

fn parse_sections(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Whether `heading` survives the current filter.
pub fn section_wanted(heading: &str, cx: &App) -> bool {
    let wanted = &cx.global::<SectionFilter>().0;
    if wanted.is_empty() {
        return true;
    }
    let lower = heading.to_lowercase();
    wanted.iter().any(|w| lower.contains(w.as_str()))
}

/// Starts the poll loop, if `HEROGPUI_CONTROL` names a file.
pub fn spawn(window: WindowHandle<Gallery>, cx: &mut App) {
    let Ok(path) = std::env::var("HEROGPUI_CONTROL") else {
        return;
    };
    let path = PathBuf::from(path);
    let ack = path.with_extension("ack");
    cx.spawn(async move |cx: &mut AsyncApp| {
        let mut last = String::new();
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(120))
                .await;
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text == last {
                continue;
            }
            last = text.clone();
            let applied = cx.update(|cx| apply(&text, window, cx));
            if applied.is_err() {
                break;
            }
            // One more frame, then acknowledge: the driver captures on the ack,
            // so it never photographs the previous page.
            cx.background_executor()
                .timer(Duration::from_millis(180))
                .await;
            let seq = text
                .lines()
                .find_map(|l| l.strip_prefix("seq="))
                .unwrap_or("")
                .trim()
                .to_owned();
            let _ = std::fs::write(&ack, seq);
        }
    })
    .detach();
}

fn apply(text: &str, window: WindowHandle<Gallery>, cx: &mut App) {
    let mut page: Option<Page> = None;
    let mut sections = Vec::new();
    let mut dark = false;
    let mut overlays = false;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "page" => page = crate::page_named(value.trim()),
            "section" => sections = parse_sections(value),
            "theme" => dark = value.trim() == "dark",
            "overlays" => overlays = value.trim() == "1",
            _ => {}
        }
    }
    cx.set_global(SectionFilter(sections));
    if cx.is_dark_theme() != dark {
        herogpui_theme::toggle_light_dark(cx);
    }
    let _ = window.update(cx, |gallery, _, cx| {
        if let Some(page) = page {
            gallery.set_initial_page(page);
        }
        gallery.set_overlays_open(overlays);
        cx.notify();
    });
}
