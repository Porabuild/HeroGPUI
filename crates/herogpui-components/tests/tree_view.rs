//! `TreeView` (HeroGPUI extension): expandable rows with chevron and
//! Right/Left expansion, Up/Down/Home/End over the visible rows, Right to the
//! first child and Left to the parent, typeahead, single and multiple
//! selection, and disabled rows that the cursor skips.

mod harness;

use std::collections::HashSet;

use gpui::{prelude::*, px, Modifiers, SharedString, TestAppContext, VisualTestContext};
use harness::{events, open_host, press, Events};
use herogpui_components::{SelectionMode, TreeItem, TreeView};

#[derive(Clone, Copy)]
struct Config {
    mode: SelectionMode,
    open_src: bool,
    controlled_expansion: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: SelectionMode::Single,
            open_src: false,
            controlled_expansion: false,
        }
    }
}

fn items() -> Vec<TreeItem> {
    vec![
        TreeItem::new("docs", "Docs").children(vec![
            TreeItem::new("guide", "Guide"),
            TreeItem::new("api", "API").child(TreeItem::new("v1", "Version 1")),
        ]),
        TreeItem::new("src", "Sources").children(vec![
            TreeItem::new("lib", "lib.rs"),
            TreeItem::new("main", "main.rs").is_disabled(true),
        ]),
        TreeItem::new("readme", "Readme"),
        TreeItem::new("secret", "Secret"),
    ]
}

fn sorted(keys: &HashSet<SharedString>) -> String {
    let mut keys: Vec<&str> = keys.iter().map(|k| k.as_ref()).collect();
    keys.sort_unstable();
    keys.join(",")
}

fn host(cx: &mut TestAppContext, config: Config) -> (Events, &mut VisualTestContext) {
    let seen = events();
    let s = seen.clone();
    let cx = open_host(cx, move || {
        let (sel, exp) = (s.clone(), s.clone());
        let mut tree = TreeView::new("tree", items())
            .selection_mode(config.mode)
            .disabled_keys(["secret".into()])
            .on_selection_change(move |keys, _, _| {
                sel.borrow_mut().push(format!("sel:{}", sorted(keys)));
            })
            .on_expanded_change(move |keys, _, _| {
                exp.borrow_mut().push(format!("exp:{}", sorted(keys)));
            });
        if config.open_src {
            tree = tree.default_expanded_keys(["src".into()]);
        }
        if config.controlled_expansion {
            tree = tree.expanded_keys(Vec::<SharedString>::new());
        }
        gpui::div().w(px(300.)).child(tree).into_any_element()
    });
    (seen, cx)
}

fn frame(cx: &mut VisualTestContext) {
    for _ in 0..2 {
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
    }
}

fn row(cx: &mut VisualTestContext, key: &str) -> Option<gpui::Bounds<gpui::Pixels>> {
    cx.debug_bounds(Box::leak(format!("tree-row-{key}").into_boxed_str()))
}

fn click(cx: &mut VisualTestContext, name: &str) {
    let at = cx
        .debug_bounds(Box::leak(name.to_owned().into_boxed_str()))
        .unwrap_or_else(|| panic!("{name} laid out"))
        .center();
    cx.simulate_click(at, Modifiers::none());
    frame(cx);
}

fn keys(cx: &mut VisualTestContext, list: &[&str]) {
    for key in list {
        press(cx, key);
        frame(cx);
    }
}

fn log(seen: &Events) -> Vec<String> {
    seen.borrow().clone()
}

#[gpui::test]
fn the_chevron_expands_and_collapses_without_selecting(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    assert!(row(cx, "guide").is_none(), "children start hidden");
    assert!(
        cx.debug_bounds("tree-toggle-readme").is_none(),
        "a leaf has no chevron"
    );
    click(cx, "tree-toggle-docs");
    let guide = row(cx, "guide").expect("expanded");
    let docs = row(cx, "docs").unwrap();
    assert!(guide.origin.y > docs.origin.y, "the child renders below");
    assert!(
        row(cx, "src").unwrap().origin.y > row(cx, "api").unwrap().origin.y,
        "siblings move down"
    );
    click(cx, "tree-toggle-docs");
    assert!(row(cx, "guide").is_none());
    assert_eq!(log(&seen), ["exp:docs", "exp:"]);
}

#[gpui::test]
fn right_and_left_walk_the_hierarchy(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    // Tab enters on the first row; Right expands Docs, then moves to Guide;
    // Down reaches API, Right expands it and moves to Version 1; Left goes
    // back to API, collapses it, then goes up to Docs, which Enter selects.
    keys(
        cx,
        &[
            "tab", "right", "right", "down", "right", "right", "left", "left", "left", "enter",
        ],
    );
    assert_eq!(
        log(&seen),
        ["exp:docs", "exp:api,docs", "exp:docs", "sel:docs"]
    );
    // Left on the expanded Docs collapses it, a second Left on the collapsed
    // root does nothing, and Space toggles the selection off.
    keys(cx, &["left", "left", "space"]);
    assert_eq!(log(&seen)[4..], ["exp:", "sel:"]);
}

#[gpui::test]
fn up_down_home_end_skip_disabled_rows(cx: &mut TestAppContext) {
    let config = Config {
        open_src: true,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    // Rows: docs, src, lib, main (disabled), readme, secret (disabled).
    keys(cx, &["tab", "end", "enter"]);
    keys(cx, &["home", "down", "down", "enter"]);
    keys(cx, &["down", "enter"]);
    keys(cx, &["up", "up", "enter"]);
    // Down at the last stop holds rather than wrapping.
    keys(cx, &["end", "down", "enter"]);
    assert_eq!(
        log(&seen),
        [
            "sel:readme",
            "sel:lib",
            "sel:readme",
            "sel:src",
            "sel:readme"
        ]
    );
}

#[gpui::test]
fn a_press_selects_and_escape_clears(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    click(cx, "tree-row-readme");
    click(cx, "tree-row-src");
    click(cx, "tree-row-secret");
    keys(cx, &["escape"]);
    assert_eq!(log(&seen), ["sel:readme", "sel:src", "sel:"]);
}

#[gpui::test]
fn multiple_selection_toggles_each_row(cx: &mut TestAppContext) {
    let config = Config {
        mode: SelectionMode::Multiple,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    click(cx, "tree-row-readme");
    click(cx, "tree-row-docs");
    click(cx, "tree-row-readme");
    assert_eq!(log(&seen), ["sel:readme", "sel:docs,readme", "sel:docs"]);
}

#[gpui::test]
fn without_selection_a_press_or_enter_toggles_a_parent(cx: &mut TestAppContext) {
    let config = Config {
        mode: SelectionMode::None,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    click(cx, "tree-row-docs");
    assert!(row(cx, "guide").is_some());
    keys(cx, &["enter"]);
    assert!(row(cx, "guide").is_none());
    click(cx, "tree-row-readme");
    assert_eq!(log(&seen), ["exp:docs", "exp:"]);
}

#[gpui::test]
fn typeahead_moves_to_the_matching_row(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    keys(cx, &["tab", "r", "enter"]);
    assert_eq!(log(&seen), ["sel:readme"]);
}

#[gpui::test]
fn typeahead_extends_its_buffer_and_skips_disabled_rows(cx: &mut TestAppContext) {
    let (seen, cx) = host(cx, Config::default());
    frame(cx);
    // "s" reaches Sources; "se" matches only the disabled Secret, so the
    // cursor stays where it is.
    keys(cx, &["tab", "s", "e", "enter"]);
    assert_eq!(log(&seen), ["sel:src"]);
}

#[gpui::test]
fn controlled_expansion_reports_without_expanding(cx: &mut TestAppContext) {
    let config = Config {
        controlled_expansion: true,
        ..Config::default()
    };
    let (seen, cx) = host(cx, config);
    frame(cx);
    click(cx, "tree-toggle-docs");
    keys(cx, &["right"]);
    assert!(row(cx, "guide").is_none());
    assert_eq!(log(&seen), ["exp:docs", "exp:docs"]);
}
