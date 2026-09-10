//! Source-shape scanning shared by the customisation wiring tests.
//!
//! These helpers prove *wiring*, not drawn pixels: for a named consumer they
//! find the enclosing top-level function and check a reader appears inside it.
//! Scoping to the enclosing function is the point — a sibling's call must not
//! satisfy an entry. The fixtures in `tests/sx_ownership.rs` pin that behavior
//! with positive and negative snippets.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Reads one file from the component crate's `src/` tree.
pub fn component_src(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} must be readable: {err}", path.display()))
}

/// Every `.rs` file under a directory, recursively.
pub fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(dir).expect("source tree must be readable");
    for entry in entries {
        let path = entry.expect("directory entry must be readable").path();
        if path.is_dir() {
            out.extend(rust_sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

/// The top-level function containing the first occurrence of `marker`.
///
/// The crate is rustfmt-formatted, so a function's body ends at the first
/// following line that is exactly its own indentation plus `}`.
pub fn enclosing_function<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let marker_index = source.find(marker)?;
    let line_start = source[..marker_index].rfind('\n').map_or(0, |i| i + 1);
    let mut start = None;
    if starts_function(&source[line_start..]) {
        start = Some(line_start);
    } else {
        let mut offset = 0;
        for line in source[..line_start].split_inclusive('\n') {
            if starts_function(line) {
                start = Some(offset);
            }
            offset += line.len();
        }
    }
    let start = start?;
    let indent_len = source[start..].len() - source[start..].trim_start().len();
    let indent = &source[start..start + indent_len];
    let rest = &source[start..];
    let close = format!("\n{indent}}}");
    let end = rest.find(&close).map(|index| start + index + 1)?;
    Some(&source[start..end])
}

fn starts_function(line: &str) -> bool {
    let line = line.trim_start();
    let line = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub(super) "))
        .or_else(|| line.strip_prefix("pub "))
        .or_else(|| line.strip_prefix("async "))
        .unwrap_or(line);
    line.starts_with("fn ")
}

/// `Ok` when the function that owns `marker` also reads `needle`.
pub fn scope_contains(source: &str, marker: &str, needle: &str) -> Result<(), String> {
    let scope = enclosing_function(source, marker)
        .ok_or_else(|| format!("no function contains marker {marker:?}"))?;
    if scope.contains(needle) {
        Ok(())
    } else {
        Err(format!(
            "the function owning {marker:?} does not read {needle:?}"
        ))
    }
}
