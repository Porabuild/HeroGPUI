//! Compile gate for the website's Rust examples.
//!
//! `web/src/data/rust-examples.json` is the code the website shows under each
//! live example. It is generated from this crate's sources by
//! `web/scripts/extract-rust-examples.mjs`, which rewrites gallery-only
//! helpers into public API. Nothing else checks that the rewritten text still
//! compiles, so this script turns every snippet into a function in a
//! `#[cfg(test)]` module (`src/snippet_check.rs`). `cargo test` and
//! `cargo clippy --all-targets` then fail on any example a reader could not
//! paste.
//!
//! Each snippet's module sees only its own `use` lines plus the `herogpui` and
//! `gpui` crates -- none of the gallery's helpers. A snippet whose `context`
//! is `"view"` reads demo state through `self`, so it becomes a method of the
//! gallery view (the only state type it can be checked against); an `"app"`
//! snippet is a free function over `&mut App`, and a `"statements"` snippet
//! (setup code shown as a code block) is the body of one.
//!
//! The JSON lives outside this package, so a packaged (crates.io) build finds
//! no file and compiles an empty module.

use std::{env, fmt::Write as _, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let json = manifest_dir.join("../web/src/data/rust-examples.json");
    println!("cargo::rerun-if-changed={}", json.display());
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("out dir")).join("snippet_check.rs");

    let mut code = String::new();
    let mut count = 0usize;
    if let Ok(text) = fs::read_to_string(&json) {
        let data: serde_json::Value = serde_json::from_str(&text).expect("rust-examples.json");
        let pages = data.as_object().expect("rust-examples.json is an object");
        for (slug, examples) in pages {
            for (index, example) in examples
                .as_array()
                .expect("examples array")
                .iter()
                .enumerate()
            {
                let field = |name: &str| example.get(name).and_then(|v| v.as_str()).unwrap_or("");
                let module = format!("{}_{index}", slug.replace('-', "_"));
                let body = field("code");
                // Asset paths in the shown code are relative to the gallery
                // source file they were lifted from; resolve them here.
                let helpers = rebase_assets(field("helpers"));
                let imports = field("imports");
                let _ = writeln!(code, "// {slug} / {}", field("heading"));
                let view = field("context") == "view";
                // A view snippet's helpers may name the view type they run in.
                let view_type = if view { "use crate::app::Gallery;" } else { "" };
                let _ = writeln!(
                    code,
                    "mod {module} {{\n{imports}\n{view_type}\n\n{helpers}\n"
                );
                if view {
                    let _ = writeln!(
                        code,
                        "impl crate::app::Gallery {{\n\
                         fn snippet_{module}(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<'_, Self>) -> gpui::AnyElement {{\n\
                         gpui::IntoElement::into_any_element({{\n{body}\n}})\n}}\n}}"
                    );
                } else if field("context") == "statements" {
                    let _ = writeln!(
                        code,
                        "fn snippet(window: &mut gpui::Window, cx: &mut gpui::App) {{\n{body}\n}}"
                    );
                } else {
                    let _ = writeln!(
                        code,
                        "fn snippet(window: &mut gpui::Window, cx: &mut gpui::App) -> gpui::AnyElement {{\n\
                         gpui::IntoElement::into_any_element({{\n{body}\n}})\n}}"
                    );
                }
                let _ = writeln!(code, "}}\n");
                count += 1;
            }
        }
    }
    let _ = writeln!(
        code,
        "/// Snippets compiled from rust-examples.json.\nconst SNIPPETS: usize = {count};"
    );
    fs::write(&out, code).expect("write snippet_check.rs");
}

/// `include_bytes!("../../../assets/x")` in a helper is relative to its gallery
/// source file; the generated module lives in `OUT_DIR`, so anchor the path
/// at this package instead.
fn rebase_assets(text: &str) -> String {
    const FROM: &str = "include_bytes!(\"../../../";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(FROM) {
        out.push_str(&rest[..at]);
        let tail = &rest[at + FROM.len()..];
        let close = tail.find("\")").expect("include_bytes! path");
        let _ = write!(
            out,
            "include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\"))",
            &tail[..close]
        );
        rest = &tail[close + 2..];
    }
    out.push_str(rest);
    out
}
