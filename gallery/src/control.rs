//! `HEROGPUI_CONTROL=<file>` — drive the running gallery from a file.
//!
//! Publish each complete UTF-8 request by atomic replacement, using a new `seq`:
//!
//! ```text
//! seq=1
//! page=Select
//! section=Usage
//! theme=light
//! overlays=0
//! reset=1
//! preview=component
//! motion=reduce
//! specimen=btn-v-DangerSoft
//! ```
//!
//! `section`, `theme` and `overlays` default to all sections, light and closed.
//! An absent `page`, `preview` or `motion` preserves that setting. `reset=1`
//! replaces the gallery root so old demo callbacks cannot mutate its replacement.
//! The sequence is written to the sibling `.ack` file only after the requested
//! view draws. Rejected requests write `seq` and `error` to the sibling `.error`
//! file. An acknowledgement proves a frame, not that animations have settled.
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use gpui::{App, AsyncApp, Global, Window, WindowHandle};
use herogpui_theme::ActiveTheme;

use crate::app::Gallery;
use crate::pages::Page;

/// The section filter and the matches observed while rendering it.
#[derive(Default)]
pub struct SectionFilter {
    sections: Vec<String>,
    matched: Rc<RefCell<Vec<bool>>>,
}

impl Global for SectionFilter {}

#[derive(Default)]
pub struct PreviewOnly(pub bool);

impl Global for PreviewOnly {}

/// Optional stable key for one live example inside a component preview.
///
/// A page must explicitly claim the key while building its example. The
/// rendered-frame acknowledgement stays pending when the key is unknown, so a
/// typo cannot silently capture a neighboring specimen.
#[derive(Default)]
pub struct SpecimenFilter {
    key: Option<String>,
    matched: Rc<Cell<bool>>,
}

impl Global for SpecimenFilter {}

/// `HEROGPUI_SECTION` seeds it; the control file replaces it.
pub fn init_section_filter(cx: &mut App) {
    let raw = std::env::var("HEROGPUI_SECTION").unwrap_or_default();
    set_section_filter(&raw, cx);
}

pub fn set_section_filter(raw: &str, cx: &mut App) {
    let sections = parse_sections(raw);
    cx.set_global(SectionFilter {
        matched: Rc::new(RefCell::new(vec![false; sections.len()])),
        sections,
    });
}

pub fn set_preview_only(preview: bool, cx: &mut App) {
    cx.set_global(PreviewOnly(preview));
}

pub fn init_specimen_filter(cx: &mut App) {
    let key = std::env::var("HEROGPUI_SPECIMEN").ok();
    set_specimen_filter(key.as_deref(), cx);
}

pub fn set_specimen_filter(key: Option<&str>, cx: &mut App) {
    cx.set_global(SpecimenFilter {
        key: key.filter(|key| !key.is_empty()).map(str::to_owned),
        matched: Rc::new(Cell::new(false)),
    });
}

pub fn has_specimen(cx: &App) -> bool {
    cx.try_global::<SpecimenFilter>()
        .is_some_and(|filter| filter.key.is_some())
}

pub(crate) fn specimen_matched(cx: &App) -> bool {
    cx.try_global::<SpecimenFilter>()
        .is_some_and(|filter| filter.matched.get())
}

/// Claims one example for the active driver request.
pub(crate) fn specimen_wanted(key: &str, cx: &App) -> bool {
    let Some(filter) = cx.try_global::<SpecimenFilter>() else {
        return true;
    };
    match filter.key.as_deref() {
        None => true,
        Some(target) if target == key => {
            filter.matched.set(true);
            true
        }
        Some(_) => false,
    }
}

pub fn preview_only(cx: &App) -> bool {
    cx.try_global::<PreviewOnly>()
        .is_some_and(|preview| preview.0)
}

/// Whether a driver request's overlay switch asks for open: `"1"` or `true`.
/// Every other value -- absent, `"0"`, a typo -- reads closed. The native
/// control file rejects unknown spellings instead (see `choice`), but a query
/// string cannot report an error back to its reader, so the web bootstrap
/// ignores unknown values here, consistently with how an unknown `?theme=`
/// falls back to light.
#[allow(dead_code)] // the web bootstrap is its only production reader; the native control file has its own parser
pub fn overlays_requested(value: Option<&str>) -> bool {
    matches!(value, Some("1" | "true"))
}

fn parse_sections(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Whether `heading` survives the current filter.
pub fn section_wanted(heading: &str, cx: &App) -> bool {
    let filter = cx.global::<SectionFilter>();
    let lower = heading.to_lowercase();
    filter.sections.is_empty()
        || filter
            .sections
            .iter()
            .any(|section| lower.contains(section))
}

/// Records a section only when the caller includes it in the rendered view.
pub(crate) fn include_section(heading: &str, cx: &App) -> bool {
    if !section_wanted(heading, cx) {
        return false;
    }
    let filter = cx.global::<SectionFilter>();
    let lower = heading.to_lowercase();
    let mut matched = filter.matched.borrow_mut();
    for (index, section) in filter.sections.iter().enumerate() {
        if lower.contains(section) {
            matched[index] = true;
        }
    }
    true
}

/// Shared with the poller; only a render can schedule its completion.
#[derive(Clone, Default)]
pub(crate) struct FrameAck(Rc<Cell<Option<bool>>>);

impl FrameAck {
    pub(crate) fn rendered(self, window: &Window, cx: &App) {
        let filter = cx.global::<SectionFilter>();
        let matched = filter.matched.clone();
        matched.borrow_mut().fill(false);
        let specimen = cx.try_global::<SpecimenFilter>().map(|filter| {
            filter.matched.set(false);
            (filter.key.is_some(), filter.matched.clone())
        });
        // In gpui-pre 0.3.3 callbacks run before the next draw. Scheduling
        // here, inside Gallery::render, places this after the current draw
        // and presentation; scheduling it from apply would be too early.
        window.on_next_frame(move |_, _| {
            let sections_match = matched.borrow().iter().all(|matched| *matched);
            let specimen_match =
                specimen.is_none_or(|(required, matched)| !required || matched.get());
            self.0.set(Some(sections_match && specimen_match));
        });
    }
}

#[derive(Debug)]
struct Request {
    seq: String,
    page: Option<Page>,
    section: String,
    dark: bool,
    overlays: bool,
    reset: bool,
    preview: Option<bool>,
    reduce_motion: Option<bool>,
    specimen: Option<String>,
}

impl Request {
    fn parse(text: &str) -> Result<Self, String> {
        let mut request = Self {
            seq: String::new(),
            page: None,
            section: String::new(),
            dark: false,
            overlays: false,
            reset: false,
            preview: None,
            reduce_motion: None,
            specimen: None,
        };
        let mut seen = HashSet::new();
        for line in text.trim_start_matches('\u{feff}').lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or("expected key=value")?;
            let (key, value) = (key.trim(), value.trim());
            if !seen.insert(key) {
                return Err(format!("duplicate key: {key}"));
            }
            match key {
                "seq" => request.seq = value.to_owned(),
                "page" => {
                    request.page = Some(
                        crate::page_named(value).ok_or_else(|| format!("unknown page: {value}"))?,
                    );
                }
                "section" => request.section = value.to_owned(),
                "theme" => request.dark = choice(value, "light", "dark", key)?,
                "overlays" => request.overlays = choice(value, "0", "1", key)?,
                "reset" => request.reset = choice(value, "0", "1", key)?,
                "preview" => request.preview = Some(choice(value, "gallery", "component", key)?),
                "motion" => request.reduce_motion = Some(choice(value, "full", "reduce", key)?),
                "specimen" => {
                    if value.is_empty() {
                        return Err("specimen must be nonempty".to_owned());
                    }
                    request.specimen = Some(value.to_owned());
                }
                _ => return Err(format!("unknown key: {key}")),
            }
        }
        if request.seq.is_empty() {
            return Err("seq must be nonempty".to_owned());
        }
        if request.specimen.is_some() && request.preview != Some(true) {
            return Err("specimen requires preview=component".to_owned());
        }
        Ok(request)
    }
}

fn choice(value: &str, no: &str, yes: &str, key: &str) -> Result<bool, String> {
    match value {
        value if value == no => Ok(false),
        value if value == yes => Ok(true),
        _ => Err(format!("{key} must be {no} or {yes}")),
    }
}

/// A sibling temporary file keeps readers from seeing a partial sequence.
fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    let mut name = path.as_os_str().to_owned();
    name.push(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let temporary = PathBuf::from(name);
    let mut file = std::fs::File::create_new(&temporary)?;
    let result = file
        .write_all(text.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| std::fs::rename(&temporary, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn report_error(path: &Path, text: &str, message: &str) {
    let seq = text
        .lines()
        .find_map(|line| {
            let (key, value) = line.trim_start_matches('\u{feff}').split_once('=')?;
            (key.trim() == "seq").then(|| value.trim())
        })
        .unwrap_or("");
    if let Err(error) = write_atomic(path, &format!("seq={seq}\nerror={message}\n")) {
        eprintln!("gallery control could not report {message}: {error}");
    }
}

/// Starts the poll loop, if `HEROGPUI_CONTROL` names a file.
pub fn spawn(window: WindowHandle<Gallery>, cx: &mut App) {
    let Ok(path) = std::env::var("HEROGPUI_CONTROL") else {
        return;
    };
    let path = PathBuf::from(path);
    let ack = path.with_extension("ack");
    let error = path.with_extension("error");
    if path == ack || path == error {
        eprintln!("HEROGPUI_CONTROL must use a different extension from .ack and .error");
        return;
    }
    cx.spawn(async move |cx: &mut AsyncApp| {
        let mut last = String::new();
        let mut pending: Option<(String, FrameAck)> = None;
        loop {
            cx.background_executor()
                .timer(Duration::from_millis(120))
                .await;
            if cx.update(|cx| window.entity(cx).is_err()) {
                break;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text != last {
                last = text.clone();
                pending = None;
                match Request::parse(&text) {
                    Ok(request) => {
                        let completion = FrameAck::default();
                        let seq = request.seq.clone();
                        if !cx.update(|cx| apply(request, window, completion.clone(), cx)) {
                            break;
                        }
                        pending = Some((seq, completion));
                    }
                    Err(message) => report_error(&error, &text, &message),
                }
            }
            if let Some((seq, completion)) = &pending {
                if let Some(matched) = completion.0.get() {
                    if !matched {
                        report_error(&error, &text, "section did not match the requested page");
                        pending = None;
                    } else if let Err(error) = write_atomic(&ack, seq) {
                        eprintln!("gallery control could not acknowledge {seq}: {error}");
                    } else {
                        pending = None;
                    }
                }
            }
        }
    })
    .detach();
}

fn apply(
    request: Request,
    window: WindowHandle<Gallery>,
    completion: FrameAck,
    cx: &mut App,
) -> bool {
    window
        .update(cx, |gallery, window, cx| {
            let page = request.page.unwrap_or(gallery.page);
            set_section_filter(&request.section, cx);
            set_specimen_filter(request.specimen.as_deref(), cx);
            if cx.is_dark_theme() != request.dark {
                herogpui_theme::toggle_light_dark(cx);
            }
            if let Some(preview) = request.preview {
                set_preview_only(preview, cx);
            }
            if let Some(reduce) = request.reduce_motion {
                cx.set_reduce_motion(reduce);
            }
            if request.reset {
                window.blur(cx);
                herogpui_components::extend::set_focus_visible(false, cx);
                gallery.toast_promises.clear();
                Gallery::reset_shared_demo_state(cx);
                window.replace_root(cx, |_, cx| {
                    let mut fresh = Gallery::new(cx);
                    fresh.set_initial_page(page);
                    fresh.set_overlays_open(request.overlays);
                    fresh.control_frame = Some(completion);
                    fresh
                });
            } else {
                gallery.set_initial_page(page);
                gallery.set_overlays_open(request.overlays);
                gallery.control_frame = Some(completion);
                cx.notify();
            }
        })
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{prelude::*, Focusable, TestAppContext};
    use herogpui_theme::ThemeProvider;

    fn open_gallery(cx: &mut TestAppContext) -> WindowHandle<Gallery> {
        cx.update(|cx| {
            ThemeProvider::init(cx);
            set_section_filter("", cx);
            set_specimen_filter(None, cx);
            set_preview_only(true, cx);
            cx.set_reduce_motion(true);
        });
        cx.add_window(|_, cx| {
            let mut gallery = Gallery::new(cx);
            gallery.set_initial_page(Page::Button);
            gallery
        })
    }

    #[test]
    fn requests_validate_before_they_can_change_the_gallery() {
        let request = Request::parse("\u{feff} seq = one\r\npage=Select\r\nsection=Usage\nreset=1\npreview=component\nmotion=reduce\ntheme=dark\noverlays=1").unwrap();
        assert_eq!(request.seq, "one");
        assert_eq!(request.page, Some(Page::Select));
        assert_eq!(request.section, "Usage");
        assert!(request.dark && request.overlays && request.reset);
        assert_eq!(request.preview, Some(true));
        assert_eq!(request.reduce_motion, Some(true));
        assert_eq!(request.specimen, None);
        let specimen = Request::parse(
            "seq=specimen\npage=Button\nsection=Variants\npreview=component\nspecimen=btn-v-DangerSoft",
        )
        .unwrap();
        assert_eq!(specimen.specimen.as_deref(), Some("btn-v-DangerSoft"));
        let defaults = Request::parse("seq=2").unwrap();
        assert!(
            defaults.page.is_none()
                && defaults.preview.is_none()
                && defaults.reduce_motion.is_none()
        );
        assert!(!defaults.dark && !defaults.overlays && !defaults.reset);
        assert!(defaults.section.is_empty());
        for invalid in [
            "",
            "seq=",
            "seq=1\npage=not a page",
            "seq=1\ntheme=purple",
            "seq=1\noverlays=2",
            "seq=1\nreset=yes",
            "seq=1\nmotion=off",
            "seq=1\npreview=unknown",
            "seq=1\nspecimen=btn-usage",
            "seq=1\npreview=gallery\nspecimen=btn-usage",
            "seq=1\npreview=component\nspecimen=",
            "seq=1\npage=Button\npage=Select",
            "seq=1\nunknown=1",
            "seq=1\nbroken line",
            "seq=1\nseq=2",
        ] {
            assert!(Request::parse(invalid).is_err(), "{invalid:?} must fail");
        }
    }

    #[test]
    fn overlay_switch_reads_only_explicit_open_values() {
        assert!(overlays_requested(Some("1")));
        assert!(overlays_requested(Some("true")));
        for closed in [
            None,
            Some(""),
            Some("0"),
            Some("false"),
            Some("yes"),
            Some(" open "),
        ] {
            assert!(!overlays_requested(closed), "{closed:?} must read closed");
        }
    }

    #[gpui::test]
    fn acknowledgements_require_the_requested_render_and_a_later_frame(cx: &mut TestAppContext) {
        let window = open_gallery(cx);
        for (page, section, preview, expected) in [
            ("Button", "Usage", "component", true),
            ("Select", "Usage", "component", true),
            ("Modal", "Usage", "component", true),
            ("Button", "missing section", "component", false),
            ("Button", "Usage,missing section", "component", false),
            ("Button", "Usage,API Reference", "gallery", true),
            (
                "Button",
                "Element Composition & Callbacks",
                "gallery",
                false,
            ),
        ] {
            let completion = FrameAck::default();
            cx.update(|cx| {
                let request = Request::parse(&format!(
                    "seq=test\npage={page}\nsection={section}\npreview={preview}"
                ))
                .unwrap();
                assert!(apply(request, window, completion.clone(), cx));
                cx.update_window(window.into(), |_, window, cx| {
                    window.simulate_next_frame(cx);
                    assert_eq!(
                        completion.0.get(),
                        None,
                        "apply cannot acknowledge before rendering"
                    );
                    window.draw(cx).clear(cx);
                    assert_eq!(
                        completion.0.get(),
                        None,
                        "render must finish before acknowledgement"
                    );
                    window.simulate_next_frame(cx);
                    assert_eq!(completion.0.get(), Some(expected), "{page}: {section}");
                })
                .unwrap();
            });
        }

        for (page, section, specimen, expected) in [
            ("Button", "Variants", "btn-v-DangerSoft", true),
            ("Button", "", "btn-v-DangerSoft", true),
            ("Button", "Variants", "btn-v-does-not-exist", false),
            ("Select", "Usage", "sel-main", true),
            ("Select", "Placement", "sel-placement-top", true),
            (
                "Select",
                "Long Values & Option Focus",
                "sel-long-values",
                true,
            ),
            ("Select", "With Clear Button", "sel-clear", true),
            ("Select", "Disabled", "sel-disabled", true),
            ("Select", "Variants", "sel-variant-Secondary", true),
            (
                "Select",
                "Controlled Multiple",
                "sel-controlled-multiple",
                true,
            ),
            ("Autocomplete", "Usage", "ac-main", true),
            ("Autocomplete", "Placement", "ac-placement-top", true),
            ("Autocomplete", "Variants", "ac-variant-Secondary", true),
            (
                "Autocomplete",
                "Controlled Open State",
                "ac-controlled-open",
                true,
            ),
            ("Combo Box", "Usage", "cb-main", true),
            ("Combo Box", "Placement", "cb-placement-top", true),
            ("Combo Box", "Controlled", "cb-controlled", true),
            ("Combo Box", "Value Render Props", "cb-value-render", true),
            ("Number Field", "Usage", "nf-main", true),
            ("Number Field", "Variants", "nf-variant-Secondary", true),
            ("Number Field", "Validation", "nf-validation", true),
            ("Text Field", "Usage", "tf-main", true),
            ("Text Field", "Validation", "tf-validation", true),
            ("Text Field", "Render Props", "tf-render-props", true),
            ("Input", "Usage", "in-main", true),
            ("Input", "Variants", "in-variant-Secondary", true),
            ("Text Area", "Usage", "ta-main", true),
            ("Text Area", "Variants", "ta-variant-Secondary", true),
            ("Input Group", "Usage", "ig-main", true),
            ("Input Group", "Variants", "ig-variant-Secondary", true),
            ("Input Group", "Password Toggle", "ig-password-toggle", true),
            ("Input Group", "Validation", "ig-validation", true),
            ("Input OTP", "Usage", "otp-main", true),
            ("Input OTP", "Filled & Active", "otp-filled-active", true),
            ("Input OTP", "Variants", "otp-variant-Secondary", true),
            ("Input OTP", "On Complete", "otp-on-complete", true),
            ("Search Field", "Usage", "sf-main", true),
            ("Search Field", "Variants", "sf-variant-Secondary", true),
            ("Search Field", "Validation", "sf-validation", true),
            ("Checkbox", "Usage", "cb-main", true),
            ("Checkbox", "Variants", "cb-variant-Secondary", true),
            ("Checkbox", "Invalid", "cb-invalid", true),
            ("Checkbox Group", "Usage", "cbg-main", true),
            ("Checkbox Group", "Controlled", "cbg-controlled", true),
            ("Checkbox Group", "Validation", "cbg-validation", true),
            ("Radio Group", "Usage", "rg-main", true),
            ("Radio Group", "Variants", "rg-variant-Secondary", true),
            ("Radio Group", "Disabled", "rg-disabled", true),
            ("Switch", "Usage", "sw-main", true),
            ("Switch", "Sizes", "sw-size-Md", true),
            ("Switch", "Form Integration", "sw-form", true),
            ("Alert", "Usage", "alert-main", true),
            ("Alert", "Colors", "alert-color-Danger", true),
            ("Alert", "Closable", "alert-closable", true),
            ("Meter", "Usage", "meter-main", true),
            ("Meter", "Colors", "meter-color-Success", true),
            ("Meter", "Sizes", "meter-size-Lg", true),
            ("Progress Bar", "Usage", "progress-main", true),
            (
                "Progress Bar",
                "Indeterminate",
                "progress-indeterminate",
                true,
            ),
            ("Progress Circle", "Usage", "progress-circle-main", true),
            (
                "Progress Circle",
                "Indeterminate",
                "progress-circle-indeterminate",
                true,
            ),
            ("Skeleton", "Usage", "skeleton-main", true),
            (
                "Skeleton",
                "Animation Types",
                "skeleton-animation-Pulse",
                true,
            ),
            ("Spinner", "Usage", "spinner-main", true),
            ("Spinner", "Sizes", "spinner-size-Md", true),
            ("Alert Dialog", "Sizes", "ad-size-md", true),
            ("Alert Dialog", "Statuses", "ad-st-danger", true),
            ("Alert Dialog", "Placements", "ad-pl-top", true),
            ("Alert Dialog", "Placements", "ad-pl-bottom", true),
            ("Alert Dialog", "Dismiss Behavior", "ad-dismiss", true),
            ("Drawer", "Placement", "dr-left", true),
            ("Drawer", "Placement", "dr-right", true),
            ("Drawer", "Placement", "dr-top", true),
            ("Drawer", "Placement", "dr-bottom", true),
            ("Drawer", "Controlled State", "dr-controlled", true),
            ("Modal", "Sizes", "md-size-lg", true),
            ("Modal", "Placement", "md-place-top", true),
            ("Modal", "Placement", "md-place-bottom", true),
            ("Modal", "Scroll Behavior", "md-scroll-inside", true),
            ("Modal", "Controlled State", "md-controlled", true),
            ("Modal", "Controlled State", "md-does-not-exist", false),
            ("Popover", "Placement", "po-pl-top", true),
            ("Popover", "Placement", "po-pl-bottom", true),
            ("Popover", "Placement", "po-pl-left", true),
            ("Popover", "Placement", "po-pl-right", true),
            ("Tooltip", "Placement", "tt-placement", true),
            ("Accordion", "Usage", "acc-main", true),
            ("Accordion", "Multiple Expanded", "acc-multiple", true),
            ("Accordion", "Disabled State", "acc-disabled", true),
            ("Breadcrumbs", "Usage", "bc-main", true),
            ("Breadcrumbs", "Separators", "bc-separators", true),
            ("Disclosure", "Usage", "dis-main", true),
            ("Disclosure", "Controlled", "dis-controlled", true),
            ("Disclosure", "Disabled Group", "dis-disabled", true),
            ("Link", "Usage", "link-main", true),
            ("Link", "Render Function", "link-render-props", true),
            ("Pagination", "Usage", "pagination-main", true),
            (
                "Pagination",
                "Disabled Links",
                "pagination-disabled-links",
                true,
            ),
            (
                "Pagination",
                "Render Props",
                "pagination-render-props",
                true,
            ),
            ("Tabs", "Usage", "tabs-main", true),
            ("Tabs", "Overflow", "tabs-overflow", true),
            ("Tabs", "Secondary Variant", "tabs-secondary", true),
            ("Badge", "Usage", "badge-main", true),
            ("Badge", "Variants", "badge-variant-Soft", true),
            ("Badge", "Colors", "badge-color-Danger", true),
            ("Chip", "Usage", "chip-main", true),
            ("Chip", "Statuses", "chip-statuses", true),
            ("Chip", "Sizes", "chip-sizes", true),
            ("Card", "Usage", "card-main", true),
            ("Card", "Variants", "card-variants", true),
            ("Card", "With Form", "card-form", true),
            ("Avatar", "Usage", "avatar-main", true),
            ("Avatar", "Fallback Content", "avatar-fallback", true),
            ("Avatar", "Variants", "avatar-variants", true),
            ("Dropdown", "Usage", "dd-main", true),
            (
                "Dropdown",
                "Controlled Open State",
                "dd-controlled-open",
                true,
            ),
            ("Dropdown", "With Submenus", "dd-submenus", true),
            ("Dropdown", "Long Press Trigger", "dd-long-press", true),
            (
                "Dropdown",
                "With Multiple Selection",
                "dd-multiple-selection",
                true,
            ),
            ("List Box", "Usage", "lb-main", true),
            ("List Box", "Virtualization", "lb-virtualization", true),
            ("List Box", "Controlled", "lb-controlled", true),
            (
                "List Box",
                "Multiple selection",
                "lb-multiple-selection",
                true,
            ),
            ("Tag Group", "Usage", "tg-main", true),
            ("Tag Group", "Selection Modes", "tg-selection-modes", true),
            ("Tag Group", "With Remove Button", "tg-remove-button", true),
            ("Tag Group", "Variants", "tg-variants", true),
            ("Color Area", "Usage", "ca-main", true),
            (
                "Color Area",
                "Color Space & Channels",
                "ca-color-channels",
                true,
            ),
            ("Color Area", "Disabled", "ca-disabled", true),
            ("Color Field", "Usage", "cf-main", true),
            ("Color Field", "Variants", "cf-variants", true),
            ("Color Field", "Validation", "cf-validation", true),
            ("Color Picker", "Usage", "cp-main", true),
            ("Color Picker", "Open / Placement", "cp-open", true),
            ("Color Picker", "With Fields", "cp-fields", true),
            ("Color Picker", "With Sliders", "cp-sliders", true),
            ("Color Slider", "Usage", "cs-main", true),
            ("Color Slider", "Vertical", "cs-vertical", true),
            ("Color Slider", "Channels", "cs-channels", true),
            ("Color Swatch", "Usage", "csw-main", true),
            ("Color Swatch", "Shapes", "csw-shapes", true),
            ("Color Swatch Picker", "Usage", "csp-main", true),
            (
                "Color Swatch Picker",
                "Disabled Item",
                "csp-disabled-item",
                true,
            ),
            (
                "Color Swatch Picker",
                "Custom Indicator",
                "csp-indicator",
                true,
            ),
            ("Button Group", "Usage", "bgroup-usage", true),
            ("Button Group", "Variants", "bgroup-v-Outline", true),
            ("Button Group", "Sizes", "bgroup-size-Lg", true),
            ("Close Button", "Usage", "close-usage", true),
            ("Toggle Button", "Controlled", "toggle-controlled", true),
            ("Toggle Button", "Sizes", "toggle-size-Lg", true),
            (
                "Toggle Button",
                "Orientation",
                "toggle-orientation-vertical",
                true,
            ),
        ] {
            let completion = FrameAck::default();
            cx.update(|cx| {
                let request = Request::parse(&format!(
                    "seq=specimen\npage={page}\nsection={section}\npreview=component\nspecimen={specimen}"
                ))
                .unwrap();
                assert!(apply(request, window, completion.clone(), cx));
                cx.update_window(window.into(), |_, window, cx| {
                    window.draw(cx).clear(cx);
                    window.simulate_next_frame(cx);
                    assert_eq!(completion.0.get(), Some(expected), "{page}/{specimen}");
                })
                .unwrap();
            });
        }
    }

    #[gpui::test]
    fn overlay_requests_construct_open_examples_and_stay_quiet_without_one(
        cx: &mut TestAppContext,
    ) {
        let window = open_gallery(cx);
        // Positive: the Modal "Controlled State" specimen constructed under
        // `overlays=1` starts open -- the same selection the native
        // `HEROGPUI_OPEN_OVERLAYS` seeds `Gallery::new` with, and the state a
        // `?overlays=1` deep link reaches on the web.
        let completion = FrameAck::default();
        cx.update(|cx| {
            let request = Request::parse(
                "seq=open\npage=Modal\nsection=Controlled State\npreview=component\nspecimen=md-controlled\noverlays=1",
            )
            .unwrap();
            assert!(apply(request, window, completion.clone(), cx));
            window
                .entity(cx)
                .unwrap()
                .read_with(cx, |gallery, _| {
                    assert!(gallery.overlays_open && gallery.modal_open);
                    assert!(gallery.demo_overlay("md-controlled"));
                });
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            assert_eq!(
                completion.0.get(),
                Some(true),
                "the open specimen must acknowledge"
            );
        })
        .unwrap();
        // Negative: an example without an overlay accepts the same switch
        // without an error -- nothing opens because nothing overlay-shaped is
        // constructed, and the request acknowledges like any other.
        let completion = FrameAck::default();
        cx.update(|cx| {
            let request = Request::parse(
                "seq=plain\npage=Button\nsection=Usage\npreview=component\nspecimen=btn-usage\noverlays=1",
            )
            .unwrap();
            assert!(apply(request, window, completion.clone(), cx));
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            window.simulate_next_frame(cx);
            assert_eq!(
                completion.0.get(),
                Some(true),
                "the switch must be ignored, not rejected"
            );
        })
        .unwrap();
        window.entity(cx).unwrap().read_with(cx, |gallery, _| {
            assert!(gallery.overlays_open);
            assert_eq!(gallery.button_clicks, 0);
        });
    }

    #[gpui::test]
    fn resetting_replaces_the_root_and_retires_its_demo_state(cx: &mut TestAppContext) {
        let window = open_gallery(cx);
        let old = window.entity(cx).unwrap();
        let input = old.read_with(cx, |gallery, _| gallery.input_name.clone());
        old.update(cx, |gallery, cx| {
            gallery.button_clicks = 42;
            gallery.modal_open = true;
            gallery.set_demo_flag("example", true);
            gallery.set_demo_flag("toast-expanded", true);
            gallery
                .input_name
                .update(cx, |input, _| input.set_value("old value"));
        });
        let completion = FrameAck::default();
        cx.update(|cx| {
            crate::app::bump_toast_closed(cx);
            herogpui_components::Toast::new("Old toast").push(None, cx);
            herogpui_components::Toast::new("Another old toast").push(None, cx);
            herogpui_components::extend::set_focus_visible(true, cx);
            window
                .update(cx, |gallery, window, cx| {
                    assert_eq!(gallery.toasts.read(cx).visible_toasts(10).len(), 2);
                    window.focus(&input.read(cx).focus_handle(cx), cx);
                })
                .unwrap();
            cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                .unwrap();
            assert!(
                old.read(cx).toasts.read(cx).is_expanded(),
                "component previews must render the toast region"
            );
            let request = Request::parse("seq=reset\npage=Select\nsection=Usage\nreset=1").unwrap();
            assert!(apply(request, window, completion.clone(), cx));
        });
        let fresh = window.entity(cx).unwrap();
        assert_ne!(old.entity_id(), fresh.entity_id());
        fresh.read_with(cx, |gallery, cx| {
            assert_eq!(gallery.page, Page::Select);
            assert_eq!(gallery.button_clicks, 0);
            assert!(!gallery.modal_open && gallery.demo_flags.is_empty());
            assert_ne!(input.entity_id(), gallery.input_name.entity_id());
            assert!(gallery.input_name.read(cx).value().is_empty());
            assert_eq!(crate::app::toasts_closed(cx), 0);
            assert!(gallery.toasts.read(cx).visible_toasts(10).is_empty());
            assert!(!herogpui_components::extend::focus_visible(cx));
        });
        // Even a late callback which keeps the old owner alive cannot change
        // the replacement. A field-only reset would share that owner.
        old.update(cx, |gallery, _| gallery.button_clicks += 1);
        assert_eq!(fresh.read_with(cx, |gallery, _| gallery.button_clicks), 0);
        cx.update(|cx| {
            cx.update_window(window.into(), |_, window, cx| {
                assert!(!input.read(cx).focus_handle(cx).is_focused(window));
                window.draw(cx).clear(cx);
                window.simulate_next_frame(cx);
            })
            .unwrap();
        });
        assert_eq!(completion.0.get(), Some(true));
    }

    #[gpui::test]
    fn reset_cancels_pending_toast_work_even_if_the_old_root_is_retained(cx: &mut TestAppContext) {
        let window = open_gallery(cx);
        let old = window.entity(cx).unwrap();
        let completed = Rc::new(Cell::new(false));
        old.update(cx, |gallery, cx| {
            let completed = completed.clone();
            let executor = cx.background_executor().clone();
            let (_, task) = herogpui_components::Toast::promise_task(
                async move {
                    executor.timer(Duration::from_millis(1500)).await;
                    completed.set(true);
                    Ok("Old upload completed".into())
                },
                "Uploading…",
                cx,
            );
            gallery.track_toast_promise(task);
            assert_eq!(gallery.toasts.read(cx).visible_toasts(10).len(), 1);
        });
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(apply(
                Request::parse("seq=reset\nreset=1").unwrap(),
                window,
                FrameAck::default(),
                cx,
            ));
        });
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        assert!(!completed.get(), "reset must cancel the old future itself");
        old.read_with(cx, |gallery, cx| {
            assert!(gallery.toast_promises.is_empty());
            assert!(gallery.toasts.read(cx).visible_toasts(10).is_empty());
        });
    }

    #[test]
    fn result_replacement_preserves_the_previous_file_on_failure() {
        let directory = std::env::temp_dir().join(format!(
            "herogpui-control-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let ack = directory.join("control.ack");
        write_atomic(&ack, "previous-sequence").unwrap();
        write_atomic(&ack, "next").unwrap();
        assert_eq!(std::fs::read_to_string(&ack).unwrap(), "next");
        let blocked = directory.join("blocked");
        std::fs::create_dir(&blocked).unwrap();
        let prior = blocked.join("keep");
        std::fs::write(&prior, "keep").unwrap();
        assert!(write_atomic(&blocked, "new").is_err());
        assert_eq!(std::fs::read_to_string(&prior).unwrap(), "keep");
        assert_eq!(
            std::fs::read_dir(&directory).unwrap().count(),
            2,
            "failed temporary output must be removed"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[gpui::test]
    fn input_examples_keep_separate_input_owners(cx: &mut TestAppContext) {
        let window = open_gallery(cx);
        let mut missing = Vec::new();
        for (page, sections, first, second, initial) in [
            (
                "Color Field",
                "Controlled,Hex value",
                "cf-ctl",
                "cf-hex",
                "#0085F5",
            ),
            (
                "Text Field",
                "Usage,Error Message",
                "tf-usage",
                "tf-error",
                "",
            ),
            (
                "Combo Box",
                "Usage,Custom Value",
                "cb-usage",
                "cb-custom-value",
                "",
            ),
        ] {
            cx.update(|cx| {
                assert!(apply(
                    Request::parse(&format!(
                        "seq=inputs\npage={page}\nsection={sections}\nreset=1\npreview=gallery"
                    ))
                    .unwrap(),
                    window,
                    FrameAck::default(),
                    cx,
                ));
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
            });
            let gallery = window.entity(cx).unwrap();
            let owners = gallery.read_with(cx, |gallery, _| {
                gallery
                    .demo_text
                    .get(first)
                    .zip(gallery.demo_text.get(second))
                    .map(|(first, second)| (first.clone(), second.clone()))
            });
            let Some((first, second)) = owners else {
                missing.push(page);
                continue;
            };
            assert_ne!(first.entity_id(), second.entity_id(), "{page}");
            cx.update_window(window.into(), |_, window, cx| {
                window.focus(&first.read(cx).focus_handle(cx), cx);
                window.draw(cx).clear(cx);
            })
            .unwrap();
            cx.simulate_input(window.into(), "P");
            assert_ne!(
                first.read_with(cx, |input, _| input.value().to_owned()),
                initial,
                "{page}"
            );
            assert_eq!(
                second.read_with(cx, |input, _| input.value().to_owned()),
                initial,
                "{page}"
            );
            if page == "Combo Box" {
                gallery.read_with(cx, |gallery, cx| {
                    let allows_custom = &gallery.demo_text["cb-custom"];
                    assert_ne!(allows_custom.entity_id(), first.entity_id());
                    assert_ne!(allows_custom.entity_id(), second.entity_id());
                    assert_eq!(allows_custom.read(cx).value(), "Zig");
                    assert!(gallery.demo_flag("cb-usage-open", false));
                    assert!(!gallery.demo_flag("cb-custom-open", false));
                });
            }
        }
        assert!(
            missing.is_empty(),
            "examples missing separate input owners: {missing:?}"
        );
    }

    #[gpui::test]
    fn date_examples_have_distinct_persistent_state(cx: &mut TestAppContext) {
        let window = open_gallery(cx);
        let cases: &[(&str, &[&str])] = &[
            (
                "Calendar",
                &[
                    "cal-usage",
                    "cal-constraints",
                    "cal-first-day",
                    "cal-disabled",
                    "cal-readonly",
                    "cal-months",
                    "cal-week-view",
                    "cal-day-view",
                    "cal-year-picker",
                ],
            ),
            ("Date Range Picker", &["drp-controlled", "drp-usage"]),
            ("Time Field", &["tmf-24-hour", "tmf-12-hour-seconds"]),
        ];
        let mut missing = Vec::new();
        for (page, keys) in cases {
            cx.update(|cx| {
                assert!(apply(
                    Request::parse(&format!("seq=dates\npage={page}\nreset=1\npreview=gallery"))
                        .unwrap(),
                    window,
                    FrameAck::default(),
                    cx,
                ));
                cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                    .unwrap();
            });
            let gallery = window.entity(cx).unwrap();
            let read_ids = |gallery: &Gallery| -> Vec<_> {
                keys.iter()
                    .map(|key| match *page {
                        "Calendar" => gallery
                            .demo_calendar
                            .get(key)
                            .map(|state| state.entity_id()),
                        "Date Range Picker" => {
                            gallery.demo_range.get(key).map(|state| state.entity_id())
                        }
                        "Time Field" => gallery.demo_time.get(key).map(|state| state.entity_id()),
                        _ => unreachable!(),
                    })
                    .collect()
            };
            let before = gallery.read_with(cx, |gallery, _| read_ids(gallery));
            for (key, id) in keys.iter().zip(&before) {
                if id.is_none() {
                    missing.push(format!("{page}/{key}"));
                }
            }
            let distinct: HashSet<_> = before.iter().flatten().collect();
            assert_eq!(distinct.len(), before.iter().flatten().count(), "{page}");
            cx.update_window(window.into(), |_, window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .unwrap();
            assert_eq!(
                before,
                gallery.read_with(cx, |gallery, _| read_ids(gallery)),
                "{page}"
            );
        }
        assert!(
            missing.is_empty(),
            "examples missing distinct owners: {missing:?}"
        );
    }
}
