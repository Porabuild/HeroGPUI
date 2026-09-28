//! Loading [`ThemeDocument`] files into the [`ThemeProvider`], plus the
//! built-in preset themes (HeroGPUI extension; `serde` feature).
//!
//! A theme file is one sparse [`ThemeDocument`] in JSON. The checked-in
//! [`THEME_SCHEMA`] (`crates/herogpui-theme/theme.schema.json`) describes the
//! format for editors and validators; point a `"$schema"`-aware editor at it
//! through its settings (a document itself may not carry a `$schema` key,
//! because documents reject unknown keys).
//!
//! Loading only *registers* themes; switch with [`use_theme`](crate::use_theme),
//! which repaints every window.
//!
//! ```
//! use herogpui_theme::{presets, ThemeDocument};
//!
//! for (id, json) in presets::PRESETS {
//!     let theme = ThemeDocument::theme_from_json(json).unwrap();
//!     assert_eq!(theme.id.as_ref(), *id);
//! }
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

use gpui::{App, SharedString};

use crate::{ThemeDocument, ThemeDocumentError, ThemeProvider};

/// The JSON Schema (draft 2020-12) for a [`ThemeDocument`] file.
pub const THEME_SCHEMA: &str = include_str!("../theme.schema.json");

/// Why [`load_themes_dir`] or [`register_theme_json`] failed.
#[derive(Debug)]
pub enum ThemeLoadError {
    /// A directory or file could not be read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A file was read but is not a valid [`ThemeDocument`].
    Document {
        path: Option<PathBuf>,
        source: ThemeDocumentError,
    },
}

impl fmt::Display for ThemeLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Document {
                path: Some(path),
                source,
            } => write!(f, "{}: {source}", path.display()),
            Self::Document { path: None, source } => source.fmt(f),
        }
    }
}

impl std::error::Error for ThemeLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Document { source, .. } => Some(source),
        }
    }
}

/// Parses one JSON [`ThemeDocument`] and registers it without activating
/// it. Returns the theme id. A theme with the same id is replaced.
pub fn register_theme_json(json: &str, cx: &mut App) -> Result<SharedString, ThemeLoadError> {
    let theme = ThemeDocument::theme_from_json(json)
        .map_err(|source| ThemeLoadError::Document { path: None, source })?;
    let id = theme.id.clone();
    cx.global_mut::<ThemeProvider>().insert(theme);
    cx.refresh_windows();
    Ok(id)
}

/// Registers every `*.json` file in `dir` (not recursive) as a theme,
/// without activating any. Files load in name order, so the result is
/// deterministic; the first invalid file aborts the load with its path, and
/// the files before it stay registered. Returns the registered ids.
pub fn load_themes_dir(
    dir: impl AsRef<Path>,
    cx: &mut App,
) -> Result<Vec<SharedString>, ThemeLoadError> {
    let dir = dir.as_ref();
    let io = |path: &Path| {
        let path = path.to_path_buf();
        move |source| ThemeLoadError::Io { path, source }
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(io(dir))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut ids = Vec::with_capacity(files.len());
    for path in files {
        let json = std::fs::read_to_string(&path).map_err(io(&path))?;
        let id = register_theme_json(&json, cx).map_err(|err| match err {
            ThemeLoadError::Document { source, .. } => ThemeLoadError::Document {
                path: Some(path.clone()),
                source,
            },
            other => other,
        })?;
        ids.push(id);
    }
    Ok(ids)
}

/// Built-in preset themes, shipped as [`ThemeDocument`] JSON under
/// `crates/herogpui-theme/themes/`. They are HeroGPUI's own palettes over
/// v3's light and dark bases, not HeroUI themes.
pub mod presets {
    use super::*;

    /// `(id, json)` for every preset, in display order.
    pub const PRESETS: &[(&str, &str)] = &[
        ("ocean", include_str!("../themes/ocean.json")),
        ("forest", include_str!("../themes/forest.json")),
        ("midnight", include_str!("../themes/midnight.json")),
        ("rose", include_str!("../themes/rose.json")),
    ];

    /// Registers every preset without activating one; returns their ids.
    pub fn register_presets(cx: &mut App) -> Vec<SharedString> {
        PRESETS
            .iter()
            .map(|(_, json)| {
                register_theme_json(json, cx).expect("built-in presets are tested to parse")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{use_theme, ActiveTheme, Appearance};
    use gpui::TestAppContext;

    #[test]
    fn every_preset_parses_and_keeps_its_id() {
        for (id, json) in presets::PRESETS {
            let theme = ThemeDocument::theme_from_json(json).unwrap();
            assert_eq!(theme.id.as_ref(), *id);
        }
    }

    /// The checked-in schema names exactly the document's keys, requires
    /// `id` and `base`, and rejects unknown keys, as the struct does.
    #[test]
    fn schema_matches_the_document_struct() {
        let schema: serde_json::Value = serde_json::from_str(THEME_SCHEMA).unwrap();
        let mut keys: Vec<&str> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let source = include_str!("theme_document.rs");
        let body = &source[source.find("pub struct ThemeDocument {").unwrap()..];
        let body = &body[body.find('\n').unwrap()..body.find("\n}").unwrap()];
        let mut fields: Vec<&str> = body
            .lines()
            .filter_map(|line| line.trim().strip_prefix("pub "))
            .filter_map(|rest| rest.split(':').next())
            .collect();
        fields.sort_unstable();
        assert_eq!(keys, fields);
        assert_eq!(schema["required"], serde_json::json!(["id", "base"]));
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
    }

    #[gpui::test]
    fn presets_register_without_activating_then_switch(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ThemeProvider::init(cx);
            let ids = presets::register_presets(cx);
            assert_eq!(ids, ["ocean", "forest", "midnight", "rose"]);
            assert_eq!(ThemeProvider::get(cx).active_id().as_ref(), "light");
            use_theme("midnight", cx).unwrap();
            assert_eq!(cx.theme().appearance, Appearance::Dark);
            assert!(ThemeProvider::get(cx)
                .theme_ids()
                .iter()
                .any(|id| id == "rose"));
        });
    }

    #[gpui::test]
    fn load_dir_registers_json_files_in_name_order(cx: &mut TestAppContext) {
        let dir = std::env::temp_dir().join(format!("herogpui-themes-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b.json"), r#"{"id":"beta","base":"dark"}"#).unwrap();
        std::fs::write(dir.join("a.json"), r#"{"id":"alpha","base":"light"}"#).unwrap();
        std::fs::write(dir.join("notes.txt"), "ignored").unwrap();
        cx.update(|cx| {
            ThemeProvider::init(cx);
            let ids = load_themes_dir(&dir, cx).unwrap();
            assert_eq!(ids, ["alpha", "beta"]);
            std::fs::write(
                dir.join("c.json"),
                r#"{"id":"gamma","base":"light","nope":1}"#,
            )
            .unwrap();
            let err = load_themes_dir(&dir, cx).unwrap_err();
            assert!(err.to_string().contains("c.json"), "{err}");
        });
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
