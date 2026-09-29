//! Theme hot reload: re-register a directory's theme files when they change
//! (HeroGPUI extension; `watch` feature).
//!
//! [`watch_themes_dir`] polls the directory [`load_themes_dir`] read, on
//! GPUI's executors, every [`THEME_WATCH_INTERVAL`]. A file whose contents
//! changed (or a new file) is parsed and registered again under its id, which
//! swaps the tokens of the active theme in place when the file defines it and
//! repaints every window, so an edit to the theme in use shows up live.
//!
//! Polling rather than an OS file-notification API keeps the feature free of
//! new dependencies and identical on every platform; the cost is reading a
//! handful of small JSON files twice a second on a background thread, and
//! only while a [`ThemeWatcher`] is alive.
//!
//! ```
//! # fn startup(cx: &mut gpui::App) -> Result<(), herogpui_theme::ThemeLoadError> {
//! use herogpui_theme::{load_themes_dir, watch_themes_dir};
//!
//! load_themes_dir("themes", cx)?;
//! let watcher = watch_themes_dir("themes", |reload, _cx| {
//!     for error in &reload.errors {
//!         eprintln!("theme reload: {error}");
//!     }
//! }, cx)?;
//! // Keep `watcher` alive (e.g. in the root view); dropping it stops watching.
//! # drop(watcher);
//! # Ok(())
//! # }
//! ```
//!
//! [`load_themes_dir`]: crate::load_themes_dir

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{App, AppContext as _, AsyncApp, SharedString, Task};

use crate::theme_registry::{register_theme_file, theme_files};
use crate::ThemeLoadError;

/// How often [`watch_themes_dir`] looks for changed theme files.
pub const THEME_WATCH_INTERVAL: Duration = Duration::from_millis(500);

/// What one poll of [`watch_themes_dir`] changed. The callback is called only
/// for a poll that found something: a reloaded file or a new error.
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct ThemeReload {
    /// The ids registered again, in file-name order.
    pub reloaded: Vec<SharedString>,
    /// Files (or the directory) that could not be read or parsed. A file that
    /// fails keeps its previously registered theme, and is reported again only
    /// after its contents change.
    pub errors: Vec<ThemeLoadError>,
}

/// A running [`watch_themes_dir`]. Dropping it stops the watch.
#[must_use = "dropping a ThemeWatcher stops watching the directory"]
pub struct ThemeWatcher {
    dir: PathBuf,
    _task: Task<()>,
}

impl ThemeWatcher {
    /// The directory being watched.
    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl std::fmt::Debug for ThemeWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeWatcher")
            .field("dir", &self.dir)
            .finish_non_exhaustive()
    }
}

/// The callback [`watch_themes_dir`] reports each changing poll to.
type OnReload = Box<dyn FnMut(&ThemeReload, &mut App)>;

/// One file as read by a poll: its path and its contents (or why not).
type Scan = std::io::Result<Vec<(PathBuf, std::io::Result<String>)>>;

fn scan(dir: &Path) -> Scan {
    Ok(theme_files(dir)?
        .into_iter()
        .map(|path| {
            let contents = std::fs::read_to_string(&path);
            (path, contents)
        })
        .collect())
}

/// Watches `dir` for changed theme files and registers each again, calling
/// `on_reload` after every poll that changed something.
///
/// The files present now are taken as already loaded (call
/// [`load_themes_dir`](crate::load_themes_dir) first to register them); only
/// later edits and new files are reloaded. A deleted file's theme stays
/// registered. Returns an error only when `dir` cannot be read now.
///
/// Reloading only registers themes, exactly as loading does: it never
/// switches the active theme, but an edit to the active theme's own file
/// applies immediately.
pub fn watch_themes_dir(
    dir: impl AsRef<Path>,
    on_reload: impl FnMut(&ThemeReload, &mut App) + 'static,
    cx: &mut App,
) -> Result<ThemeWatcher, ThemeLoadError> {
    let dir = dir.as_ref().to_path_buf();
    let seen: HashMap<PathBuf, String> = scan(&dir)
        .map_err(|source| ThemeLoadError::Io {
            path: dir.clone(),
            source,
        })?
        .into_iter()
        .filter_map(|(path, contents)| contents.ok().map(|c| (path, c)))
        .collect();
    let mut state = WatchState {
        dir: dir.clone(),
        seen,
        dir_unreadable: false,
        on_reload: Box::new(on_reload),
    };
    let task = cx.spawn(async move |cx: &mut AsyncApp| loop {
        cx.background_executor().timer(THEME_WATCH_INTERVAL).await;
        let scan_dir = state.dir.clone();
        let result = cx.background_spawn(async move { scan(&scan_dir) }).await;
        cx.update(|cx| state.apply(result, cx));
    });
    Ok(ThemeWatcher { dir, _task: task })
}

struct WatchState {
    dir: PathBuf,
    /// The last contents seen per file, whether or not they parsed.
    seen: HashMap<PathBuf, String>,
    /// Set after an unreadable-directory error was reported, so a directory
    /// that stays missing is reported once rather than on every poll.
    dir_unreadable: bool,
    on_reload: OnReload,
}

impl WatchState {
    fn apply(&mut self, scan: Scan, cx: &mut App) {
        let mut reload = ThemeReload::default();
        match scan {
            Err(source) => {
                if !self.dir_unreadable {
                    self.dir_unreadable = true;
                    reload.errors.push(ThemeLoadError::Io {
                        path: self.dir.clone(),
                        source,
                    });
                }
            }
            Ok(files) => {
                self.dir_unreadable = false;
                for (path, contents) in files {
                    // An unreadable file is typically one caught mid-write or
                    // deleted between listing and reading; the next poll sees
                    // it settled.
                    let Ok(json) = contents else { continue };
                    if self.seen.get(&path) == Some(&json) {
                        continue;
                    }
                    match register_theme_file(&path, &json, cx) {
                        Ok(id) => reload.reloaded.push(id),
                        Err(err) => reload.errors.push(err),
                    }
                    self.seen.insert(path, json);
                }
            }
        }
        if !reload.reloaded.is_empty() || !reload.errors.is_empty() {
            (self.on_reload)(&reload, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::{load_themes_dir, use_theme, ActiveTheme, ThemeProvider};
    use gpui::TestAppContext;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("herogpui-watch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn poll(cx: &mut TestAppContext) {
        cx.executor().advance_clock(THEME_WATCH_INTERVAL);
        cx.run_until_parked();
    }

    #[gpui::test]
    fn an_edited_active_theme_reapplies_and_new_files_register(cx: &mut TestAppContext) {
        let dir = temp_dir("edit");
        let file = dir.join("brand.json");
        std::fs::write(
            &file,
            r##"{"id":"brand","base":"light","accent":"#ff0000"}"##,
        )
        .unwrap();
        // (reloaded ids, error count) per callback.
        type Log = Rc<RefCell<Vec<(Vec<SharedString>, usize)>>>;
        let reloads: Log = Rc::default();
        let log = reloads.clone();
        let watcher = cx.update(|cx| {
            ThemeProvider::init(cx);
            load_themes_dir(&dir, cx).unwrap();
            use_theme("brand", cx).unwrap();
            watch_themes_dir(
                &dir,
                move |reload, _| {
                    log.borrow_mut()
                        .push((reload.reloaded.clone(), reload.errors.len()));
                },
                cx,
            )
            .unwrap()
        });
        assert_eq!(watcher.dir(), dir.as_path());
        let red = cx.read(|cx| cx.colors().accent.color);

        // Nothing changed: no reload, no callback.
        poll(cx);
        assert!(reloads.borrow().is_empty());

        std::fs::write(
            &file,
            r##"{"id":"brand","base":"light","accent":"#00ff00"}"##,
        )
        .unwrap();
        poll(cx);
        let green = cx.read(|cx| cx.colors().accent.color);
        assert_ne!(red, green, "the active theme did not pick up the edit");
        assert_eq!(
            cx.read(|cx| ThemeProvider::get(cx).active_id().clone()),
            "brand"
        );
        assert_eq!(*reloads.borrow(), [(vec![SharedString::from("brand")], 0)]);

        // A broken edit keeps the last good theme and is reported once.
        std::fs::write(&file, r#"{"id":"brand","base":"light","nope":1}"#).unwrap();
        poll(cx);
        poll(cx);
        assert_eq!(cx.read(|cx| cx.colors().accent.color), green);
        assert_eq!(reloads.borrow().len(), 2);
        assert_eq!(reloads.borrow()[1], (vec![], 1));

        // A new file registers without activating.
        std::fs::write(dir.join("extra.json"), r#"{"id":"extra","base":"dark"}"#).unwrap();
        poll(cx);
        assert!(cx.read(|cx| ThemeProvider::get(cx).contains("extra")));
        assert_eq!(
            cx.read(|cx| ThemeProvider::get(cx).active_id().clone()),
            "brand"
        );

        // Dropping the watcher stops it.
        drop(watcher);
        std::fs::write(
            &file,
            r##"{"id":"brand","base":"light","accent":"#0000ff"}"##,
        )
        .unwrap();
        poll(cx);
        assert_eq!(cx.read(|cx| cx.colors().accent.color), green);
        assert_eq!(reloads.borrow().len(), 3);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[gpui::test]
    fn an_unreadable_directory_is_an_error(cx: &mut TestAppContext) {
        let dir = temp_dir("missing");
        std::fs::remove_dir_all(&dir).unwrap();
        cx.update(|cx| {
            ThemeProvider::init(cx);
            let err = watch_themes_dir(&dir, |_, _| {}, cx).unwrap_err();
            assert!(matches!(err, ThemeLoadError::Io { .. }), "{err:?}");
        });
    }
}
