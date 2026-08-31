//! Minimal Rust syntax highlighting and line formatting for the docs' code
//! blocks.
//!
//! The gallery shows its own source (`stringify!` of each demo) plus import
//! lines, so the tokenizer only needs to cover the constructs those snippets
//! use: comments, strings, char literals, lifetimes, numbers, keywords,
//! macros, type names and call sites. It emits byte ranges for
//! `gpui::StyledText::with_highlights`.
//!
//! The formatter reflows the single-line walls `stringify!` produces
//! (which separate every token pair with a space) into rustfmt-shaped text:
//! token spacing is normalized and lines break only at Rust-legal points —
//! after a `,` or before a `.` in a method chain — indented by bracket depth.
//! It is display-only surgery: it never adds, drops or reorders a token.

use std::ops::Range;

use gpui::{rgb, FontStyle, HighlightStyle};

/// Longest formatted code line, in monospace columns. 100 columns fits the
/// 860px docs column at the code text size with room to spare.
pub const CODE_MAX_COLS: usize = 100;

/// A multi-line snippet counts as author-formatted only while every line fits
/// this; `stringify!` walls slip past that check (the expression printer wraps
/// only occasional call arguments, leaving one very long line) and reflow.
const REFLOW_LINE_LIMIT: usize = 120;

const COMMENT: u32 = 0x7F848E;
const KEYWORD: u32 = 0xC678DD;
const STRING: u32 = 0x98C379;
const NUMBER: u32 = 0xD19A66;
const TYPE: u32 = 0xE5C07B;
const FUNCTION: u32 = 0x61AFEF;
const LIFETIME: u32 = 0x56B6C2;

fn color(hex: u32) -> HighlightStyle {
    HighlightStyle::color(rgb(hex).into())
}

fn italic(hex: u32) -> HighlightStyle {
    HighlightStyle {
        color: Some(rgb(hex).into()),
        font_style: Some(FontStyle::Oblique),
        ..Default::default()
    }
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "Some", "None", "Ok", "Err",
];

/// Highlight ranges for a Rust snippet. Never panics on odd input: anything
/// the scanner does not recognise is simply left unstyled.
pub fn rust_highlights(code: &str) -> Vec<(Range<usize>, HighlightStyle)> {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut out: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    let mut i = 0usize;

    while i < len {
        let b = bytes[i];
        match b {
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                let start = i;
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                out.push((start..i, italic(COMMENT)));
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                let start = i;
                i = scan_block_comment(code, i);
                out.push((start..i, italic(COMMENT)));
            }
            b'"' => {
                let start = i;
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => i += 2.min(len - i),
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                out.push((start..i, color(STRING)));
            }
            b'\'' => {
                let start = i;
                let (end, kind, _) = scan_quote(bytes, i);
                i = end;
                match kind {
                    TokKind::Char => out.push((start..end, color(STRING))),
                    TokKind::Lifetime => out.push((start..end, color(LIFETIME))),
                    _ => {}
                }
            }
            b'0'..=b'9' => {
                let start = i;
                while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                    i += 1;
                }
                // fractional part; `12.` is a valid float, but stop before `..`
                if i < len && bytes[i] == b'.' && !matches!(bytes.get(i + 1), Some(b'.')) {
                    i += 1;
                    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                        i += 1;
                    }
                }
                // type suffixes: f32, i64, ...
                while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                out.push((start..i, color(NUMBER)));
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                let end = scan_ident(bytes, i);
                i = end;
                let word = &code[start..end];
                let next = bytes.get(i).copied();
                let style = if next == Some(b'!') {
                    Some(color(FUNCTION)) // macro: vec!, format!, px!
                } else if KEYWORDS.contains(&word) {
                    Some(color(KEYWORD))
                } else if next == Some(b'(') {
                    Some(color(FUNCTION)) // call site / builder method
                } else if word.starts_with(|c: char| c.is_ascii_uppercase()) {
                    Some(color(TYPE))
                } else {
                    None
                };
                if let Some(style) = style {
                    out.push((start..end, style));
                }
            }
            _ => i += 1,
        }
    }

    out
}

fn scan_ident(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

// ---------------------------------------------------------------------------
// Display formatter
// ---------------------------------------------------------------------------

/// Reformat a code snippet for display: `stringify!` walls become
/// rustfmt-shaped text; genuinely author-formatted multi-line snippets keep
/// their formatting. The result is a whitespace-only rewrite of the input.
pub fn format_rust_snippet(code: &str) -> String {
    let code = code.trim_matches(|c: char| c.is_whitespace());
    let hand_formatted = code.contains('\n')
        && !code
            .lines()
            .any(|line| line.chars().count() > REFLOW_LINE_LIMIT);
    if hand_formatted {
        return code.to_owned();
    }
    let toks = tokenize(code);
    if toks.is_empty() {
        return String::new();
    }
    let parts: Vec<Part<'_>> = toks
        .iter()
        .enumerate()
        .map(|(i, tok)| Part {
            text: &tok.text,
            kind: tok.kind,
            space_before: i > 0 && needs_space_before(&toks[i - 1], tok),
        })
        .collect();
    layout(&parts, CODE_MAX_COLS)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokKind {
    Word,
    Str,
    Char,
    Lifetime,
    Punct,
    Comment,
}

struct Tok {
    text: String,
    kind: TokKind,
}

struct Part<'a> {
    text: &'a str,
    kind: TokKind,
    space_before: bool,
}

const TWO_CHAR_PUNCTS: &[&str] = &[
    "::", "->", "=>", "!=", "==", "<=", ">=", "&&", "||", "..", "+=", "-=", "*=", "/=", "%=", "^=",
    "&=", "|=",
];

const THREE_CHAR_PUNCTS: &[&str] = &["..="];

/// Keywords that keep their space before a following `(` or `[` (`if (x)`,
/// `let (a)`), as opposed to calls like `Some(x)` which close up.
const KEYWORDS_BEFORE_PAREN: &[&str] = &[
    "as", "break", "continue", "dyn", "else", "for", "if", "impl", "in", "let", "loop", "match",
    "move", "return", "unsafe", "use", "where", "while",
];

fn tokenize(code: &str) -> Vec<Tok> {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut toks: Vec<Tok> = Vec::new();
    let mut i = 0usize;

    while i < len {
        let start = i;
        let b = bytes[i];
        match b {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
            }
            b'"' => {
                i += 1;
                while i < len {
                    match bytes[i] {
                        b'\\' => i += 2.min(len - i),
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                toks.push(Tok {
                    text: code[start..i].to_owned(),
                    kind: TokKind::Str,
                });
            }
            b'\'' => {
                let (end, kind, text) = scan_quote(bytes, i);
                i = end;
                toks.push(Tok { text, kind });
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                toks.push(Tok {
                    text: code[start..i].to_owned(),
                    kind: TokKind::Comment,
                });
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                i = scan_block_comment(code, i);
                toks.push(Tok {
                    text: code[start..i].to_owned(),
                    kind: TokKind::Comment,
                });
            }
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                i = scan_ident(bytes, i);
                // fractional / suffixed numbers glued to the digits: 12.5,
                // 3.5f32. `12.` keeps its dot a separate token, which the
                // spacing rules rejoin anyway.
                while i + 1 < len && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
                    i = scan_ident(bytes, i + 1);
                }
                toks.push(Tok {
                    text: code[start..i].to_owned(),
                    kind: TokKind::Word,
                });
            }
            _ => {
                let two = if i + 2 <= len && bytes[i].is_ascii() && bytes[i + 1].is_ascii() {
                    &code[i..i + 2]
                } else {
                    ""
                };
                let three = if i + 3 <= len
                    && bytes[i].is_ascii()
                    && bytes[i + 1].is_ascii()
                    && bytes[i + 2].is_ascii()
                {
                    &code[i..i + 3]
                } else {
                    ""
                };
                let width = if THREE_CHAR_PUNCTS.contains(&three) {
                    3
                } else if TWO_CHAR_PUNCTS.contains(&two) {
                    2
                } else {
                    code[i..].chars().next().map_or(1, char::len_utf8)
                };
                i += width;
                toks.push(Tok {
                    text: code[start..i].to_owned(),
                    kind: TokKind::Punct,
                });
            }
        }
    }

    toks
}

fn scan_block_comment(code: &str, start: usize) -> usize {
    let bytes = code.as_bytes();
    let mut depth = 1usize;
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return i;
            }
        } else {
            i += code[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    bytes.len()
}

/// Classify a `'`: char literal, lifetime, or bare quote. `stringify!`
/// separates every token with a space, so `'a'` arrives as `' a '` and
/// `'static` as `' static` — both forms are recognised and the returned text
/// is rebuilt without the inserted spaces.
fn scan_quote(bytes: &[u8], i: usize) -> (usize, TokKind, String) {
    let at = |k: usize| bytes.get(k).copied();
    let after_ws = |mut k: usize| {
        while at(k).is_some_and(|b| b.is_ascii_whitespace()) {
            k += 1;
        }
        k
    };
    // Escaped char literal: glued `'\n'` or the spaced `' \ n '`.
    let backslash = if at(i + 1) == Some(b'\\') {
        Some(i + 1)
    } else if at(i + 1).is_some_and(|b| b.is_ascii_whitespace()) {
        let b = after_ws(i + 1);
        (at(b) == Some(b'\\')).then_some(b)
    } else {
        None
    };
    if let Some(bs) = backslash {
        let esc = after_ws(bs + 1);
        if at(esc).is_some_and(|b| !b.is_ascii_whitespace()) {
            let close = if at(esc + 1) == Some(b'\'') {
                Some(esc + 1)
            } else if at(esc + 1).is_some_and(|b| b.is_ascii_whitespace())
                && at(esc + 2) == Some(b'\'')
            {
                Some(esc + 2)
            } else {
                None
            };
            if let Some(close) = close {
                let inner = at(esc).unwrap() as char;
                return (close + 1, TokKind::Char, format!("'\\{inner}'"));
            }
        }
    }
    let body = if at(i + 1).is_some_and(|b| b.is_ascii_whitespace()) {
        after_ws(i + 1)
    } else {
        i + 1
    };
    let closes = |k: usize| at(k) == Some(b'\'');
    if at(body).is_some_and(|b| !b.is_ascii_whitespace()) {
        let Some(rest) = std::str::from_utf8(&bytes[body..]).ok() else {
            return (i + 1, TokKind::Punct, "'".to_owned());
        };
        let Some(inner) = rest.chars().next() else {
            return (i + 1, TokKind::Punct, "'".to_owned());
        };
        let after_inner = body + inner.len_utf8();
        let close = if closes(after_inner) {
            Some(after_inner)
        } else if at(after_inner).is_some_and(|b| b.is_ascii_whitespace()) {
            let candidate = after_ws(after_inner);
            closes(candidate).then_some(candidate)
        } else {
            None
        };
        if let Some(close) = close {
            return (close + 1, TokKind::Char, format!("'{inner}'"));
        }
    }
    let ident_end = scan_ident(bytes, body);
    if ident_end > body {
        let name = &bytes[body..ident_end];
        let name = std::str::from_utf8(name).unwrap_or("");
        return (ident_end, TokKind::Lifetime, format!("'{name}"));
    }
    (i + 1, TokKind::Punct, "'".to_owned())
}

fn needs_space_before(prev: &Tok, tok: &Tok) -> bool {
    if prev.kind == TokKind::Punct
        && matches!(
            prev.text.as_str(),
            "(" | "[" | "{" | "." | "::" | ".." | "..="
        )
    {
        return false;
    }
    match tok.kind {
        // Glue a quote to a preceding name, bracket or operator (`f('x')`,
        // `&'static`, `f<'a>`); keep the space after separators (`, 'x'`).
        TokKind::Char | TokKind::Lifetime => {
            !(matches!(prev.kind, TokKind::Word)
                || prev.kind == TokKind::Punct
                    && matches!(prev.text.as_str(), "(" | "[" | "{" | "&" | "::" | "<"))
        }
        TokKind::Punct => match tok.text.as_str() {
            "," | ";" | ":" | ")" | "]" | "}" | "." | "::" | ".." | "..=" => false,
            // Macro invocation or postfix `?` closes up; unary `!` keeps its
            // space.
            "!" | "?" => prev.kind != TokKind::Word,
            // A call or index closes up after a name; control keywords keep
            // theirs (`if (x)`, `let (a)`).
            "(" | "[" => {
                prev.kind == TokKind::Word && KEYWORDS_BEFORE_PAREN.contains(&prev.text.as_str())
            }
            _ => true,
        },
        _ => true,
    }
}

fn is_closer(kind: TokKind, text: &str) -> bool {
    kind == TokKind::Punct && matches!(text, ")" | "]" | "}")
}

fn layout(parts: &[Part<'_>], max_cols: usize) -> String {
    let mut out = String::new();
    let mut cur = String::new();
    let mut col = 0usize;
    let mut first = true;
    let mut depth = 0usize;
    // Last break opportunity on this line: the index to resume at, the
    // indentation for the continuation, and how much of `cur` to keep (the
    // break point). Recorded after a `,` or before a `.` in a chain.
    let mut opp: Option<(usize, usize, usize)> = None;
    let mut i = 0usize;

    while i < parts.len() {
        let part = &parts[i];
        let w = part.text.chars().count();
        let sep = if first || !part.space_before { 0 } else { 1 };
        if !first && col + sep + w > max_cols {
            if let Some((resume, indent, cut)) = opp {
                out.push_str(&cur[..cut]);
                out.push('\n');
                cur = " ".repeat(indent);
                col = indent;
                first = true;
                opp = None;
                i = resume;
                continue;
            }
            // No legal break point: the overlong token (usually a long
            // string) stays on its line and the next opportunity is used.
        }
        let was_first = first;
        let cur_before = cur.len();
        if sep == 1 {
            cur.push(' ');
            col += 1;
        }
        cur.push_str(part.text);
        col += w;
        first = false;

        match part.text {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth = depth.saturating_sub(1),
            "," if depth > 0
                && i + 1 < parts.len()
                && !is_closer(parts[i + 1].kind, parts[i + 1].text) =>
            {
                opp = Some((i + 1, 2 * depth, cur.len()));
            }
            "." if !was_first
                && depth > 0
                && matches!(parts.get(i + 1).map(|p| &p.kind), Some(TokKind::Word)) =>
            {
                opp = Some((i, 2 * depth, cur_before));
            }
            _ => {}
        }
        i += 1;
    }
    out.push_str(&cur);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_cover_expected_tokens() {
        let code = "let x = 12.; // note\nh::Button::new(\"id\").size(*s)";
        let runs = rust_highlights(code);
        let painted: Vec<(&str, u32)> = runs
            .iter()
            .filter_map(|(r, s)| {
                let expected = [
                    (COMMENT, italic(COMMENT)),
                    (KEYWORD, color(KEYWORD)),
                    (STRING, color(STRING)),
                    (NUMBER, color(NUMBER)),
                    (TYPE, color(TYPE)),
                    (FUNCTION, color(FUNCTION)),
                    (LIFETIME, color(LIFETIME)),
                ];
                let hex = expected
                    .into_iter()
                    .find(|(_, style)| style.color == s.color)?
                    .0;
                Some((&code[r.clone()], hex))
            })
            .collect();
        assert_eq!(
            painted,
            vec![
                ("let", KEYWORD),
                ("12.", NUMBER),
                ("// note", COMMENT),
                ("Button", TYPE),
                ("new", FUNCTION),
                ("\"id\"", STRING),
                ("size", FUNCTION),
            ]
        );
    }

    #[test]
    fn lifetime_vs_char_literal() {
        let code = "fn f<'a>(c: 'a', x: &'static str)";
        let runs = rust_highlights(code);
        let texts: Vec<&str> = runs.iter().map(|(r, _)| &code[r.clone()]).collect();
        assert!(texts.contains(&"'a'"));
        assert!(texts.contains(&"'static"));
        assert!(texts.contains(&"fn"));
    }

    #[test]
    fn ranges_are_char_boundaries_and_ordered() {
        let code = "h::Button::new(\"äöü\") // ünïcode\nlet 'static = 3.5f32;";
        let runs = rust_highlights(code);
        let mut last = 0usize;
        for (range, _) in &runs {
            assert!(code.is_char_boundary(range.start));
            assert!(code.is_char_boundary(range.end));
            assert!(range.start >= last, "runs must not overlap");
            last = range.end;
        }
    }

    #[test]
    fn nested_utf8_block_comments_highlight_through_unclosed_comments() {
        let code = "let value = /* outer café\n /* inner */ tail */ 'x'; value / 2";
        let runs = rust_highlights(code);
        let comments: Vec<&str> = runs
            .iter()
            .filter(|(_, style)| style.color == italic(COMMENT).color)
            .map(|(range, _)| &code[range.clone()])
            .collect();
        assert_eq!(comments, vec!["/* outer café\n /* inner */ tail */"]);

        let unclosed = "let value = /* outer café\n /* inner */ tail";
        let unclosed_runs = rust_highlights(unclosed);
        let comment = unclosed_runs
            .iter()
            .find(|(_, style)| style.color == italic(COMMENT).color)
            .map(|(range, _)| &unclosed[range.clone()]);
        assert_eq!(comment, Some("/* outer café\n /* inner */ tail"));
        assert!(runs.iter().any(|(range, style)| {
            style.color == color(STRING).color && &code[range.clone()] == "'x'"
        }));
        for (range, _) in unclosed_runs {
            assert!(unclosed.is_char_boundary(range.start));
            assert!(unclosed.is_char_boundary(range.end));
        }
    }

    fn squeezed(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// Formatting is whitespace-only surgery: every token survives in order.
    fn assert_tokens_preserved(code: &str) {
        let out = format_rust_snippet(code);
        assert_eq!(squeezed(&out), squeezed(code), "tokens changed in {out:?}");
    }

    #[test]
    fn stringify_spacing_is_normalized() {
        let code = "row (vec ! [h :: Button :: new (\"b\") . label (\"Click me\") \
                    . into_any_element ()])";
        let out = format_rust_snippet(code);
        assert_eq!(
            out,
            "row(vec![h::Button::new(\"b\").label(\"Click me\").into_any_element()])"
        );
        assert_eq!(squeezed(&out), squeezed(code));
    }

    #[test]
    fn long_chains_break_at_methods_and_commas() {
        let code = "row (vec ! [h :: ButtonGroup :: new () . separators (true) \
                    . button (h :: Button :: new (\"bgu-1\") . label (\"Merge pull request\")) \
                    . button (h :: Button :: new (\"bgu-2\") . is_icon_only (true) \
                    . child (icon (h :: icons :: CHEVRON_DOWN , cx))) . into_any_element ()])";
        let out = format_rust_snippet(code);
        assert_eq!(squeezed(&out), squeezed(code));

        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.len() > 1, "expected wrapped output, got {out:?}");
        for line in &lines {
            assert!(
                line.chars().count() <= CODE_MAX_COLS,
                "line too long: {line:?}"
            );
        }
        // Continuation lines never start with a dangling closer or comma.
        for line in &lines[1..] {
            let t = line.trim_start();
            assert!(
                !(t.starts_with(')') || t.starts_with(']') || t.starts_with(',')),
                "bad continuation: {line:?}"
            );
        }
        // Chain breaks indent by bracket depth.
        assert!(lines
            .iter()
            .any(|l| l.starts_with("    .") || l.starts_with("      .")));
    }

    #[test]
    fn probe_long_string_after_chain_dot() {
        let long = "w".repeat(93);
        let code = format!(
            "row (vec ! [h :: Link :: new (\"l\") . href (\"{long}\") . label (\"Docs\") \
             . into_any_element ()])"
        );
        let out = format_rust_snippet(&code);
        assert_eq!(squeezed(&out), squeezed(&code));

        // brute-force layout fuzz with a small max to expose non-progress
        let mut seed: u64 = 0x9E3779B97F4A7C15;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let words = ["a", "bb", "ccc", "dddddddddd", "wwwwwwwwwwwwwwwwwwww"];
        for case in 0..20000u64 {
            let mut text = String::new();
            let mut depth = 0i32;
            let n = 2 + (rand() % 24) as usize;
            for _ in 0..n {
                match rand() % 10u64 {
                    0..=3 => {
                        text.push_str(words[(rand() % words.len() as u64) as usize]);
                        text.push(' ');
                    }
                    4 => {
                        if depth < 4 {
                            text.push_str("( ");
                            depth += 1;
                        }
                    }
                    5 => {
                        if depth > 0 {
                            text.push_str(") ");
                            depth -= 1;
                        }
                    }
                    6 => text.push_str(". "),
                    7 => text.push_str(":: "),
                    8 => text.push_str(", "),
                    _ => text.push_str("! "),
                }
            }
            while depth > 0 {
                text.push_str(") ");
                depth -= 1;
            }
            let toks = tokenize(&text);
            if toks.is_empty() {
                continue;
            }
            let parts: Vec<Part<'_>> = toks
                .iter()
                .enumerate()
                .map(|(i, tok)| Part {
                    text: &tok.text,
                    kind: tok.kind,
                    space_before: i > 0 && needs_space_before(&toks[i - 1], tok),
                })
                .collect();
            for max in [8usize, 13, 21] {
                let out = layout(&parts, max);
                assert_eq!(
                    squeezed(&out),
                    squeezed(&text),
                    "case {case} max {max} tokens changed in {out:?} from {text:?}"
                );
            }
        }
    }

    #[test]
    fn string_literals_are_never_split() {
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen \
                    fourteen fifteen sixteen";
        let code = format!("f (\"{long}\") . label (\"end\")");
        let out = format_rust_snippet(&code);
        assert!(out.contains(long), "string literal damaged: {out:?}");
        // No line break falls between the quotes.
        let start = out.find(long).unwrap();
        assert!(!out[..start + long.len()].contains('\n'));
        assert_tokens_preserved(&code);
    }

    #[test]
    fn multiline_snippets_keep_author_formatting() {
        let code = "let a = 1;\nlet b = a ; // spaced\n";
        assert_eq!(
            format_rust_snippet(code),
            "let a = 1;\nlet b = a ; // spaced"
        );
    }

    #[test]
    fn printer_wrapped_stringify_walls_still_reflow() {
        // `stringify!` of a multi-line demo keeps a few printer-inserted
        // newlines around long call arguments — the wall must still reflow.
        let code = "col (vec ! [h :: Surface :: new () . padding (px (24.)) . gap (px (16.)) \
                    . child (h :: Fieldset :: new () . child (h :: FieldsetLegend :: new \
                    (\"Profile\")) . child (h :: TextField :: new (self . demo_text \
                    (\"fset-name\" ,\n\"\" ,\ncx)) . label (\"Name\") . variant (FieldVariant \
                    :: Secondary) ,) . into_any_element ()])";
        let out = format_rust_snippet(code);
        assert_tokens_preserved(code);
        for line in out.lines() {
            assert!(
                line.chars().count() <= CODE_MAX_COLS,
                "line too long: {line:?}"
            );
        }
        assert!(out.contains(".label(\"Name\")"));
    }

    #[test]
    fn short_snippet_stays_on_one_line() {
        let code = "h :: CloseButton :: new (\"cb\")";
        assert_eq!(format_rust_snippet(code), "h::CloseButton::new(\"cb\")");
    }

    #[test]
    fn inclusive_ranges_stay_valid_rust() {
        let code = "(1..=12).map(|n| n..8)";
        let token_list = tokenize("1..=12");
        let tokens: Vec<&str> = token_list.iter().map(|token| token.text.as_str()).collect();
        assert_eq!(tokens, vec!["1", "..=", "12"]);
        let out = format_rust_snippet(code);
        assert!(out.contains("1..=12"), "inclusive range was split: {out:?}");
        assert!(out.contains("n..8"), "range was spaced: {out:?}");
        assert!(!out.contains(".. ="));
        assert_tokens_preserved(code);
    }

    #[test]
    fn import_line_fits_without_breaking() {
        let code = "use herogpui::components::field::{Fieldset, FieldGroup, FieldsetLegend, \
                    FieldsetActions};";
        assert_eq!(format_rust_snippet(code), code);
    }

    #[test]
    fn long_import_breaks_after_commas() {
        let code = "use herogpui::components::color::{ColorArea, ColorField, ColorSlider, \
                    ColorSwatch, ColorSwatchPicker, ColorSwatchPickerDivider};";
        let out = format_rust_snippet(code);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.len() > 1, "expected wrapped import, got {out:?}");
        assert!(lines[0].ends_with(','));
        assert!(lines[1..].iter().all(|l| l.starts_with("  ")));
    }

    #[test]
    fn char_literals_and_lifetimes_are_rejoined() {
        let code = "f (' x ' , & ' static str , tag : ' z ')";
        let out = format_rust_snippet(code);
        assert_eq!(out, "f('x', &'static str, tag: 'z')");
        assert_eq!(squeezed(&out), squeezed(code));
    }

    #[test]
    fn escaped_char_literals_are_preserved() {
        // Glued and `stringify!`-spaced forms, including the escaped quote.
        let code = "f (' \\ n ' , g (' \\ ' ') , c : '\\t' , tag : '\\0')";
        let out = format_rust_snippet(code);
        assert_eq!(out, "f('\\n', g('\\''), c: '\\t', tag: '\\0')");
        assert_eq!(squeezed(&out), squeezed(code));
    }

    #[test]
    fn escaped_and_unicode_char_literals_are_highlighted() {
        let code = "let escaped = '\\n'; let unicode = '·';";
        let runs = rust_highlights(code);
        let strings: Vec<&str> = runs
            .iter()
            .filter(|(_, style)| style.color == color(STRING).color)
            .map(|(range, _)| &code[range.clone()])
            .collect();
        assert!(strings.contains(&"'\\n'"));
        assert!(strings.contains(&"'·'"));
    }

    #[test]
    fn unicode_char_literals_do_not_split_utf8() {
        let code =
            "InputOTP :: new (state) . slot (|index , value| div () . child (value . unwrap_or ('·') . to_string ()))";
        let out = format_rust_snippet(code);
        assert!(out.contains("'·'"), "unicode char literal damaged: {out:?}");
        assert_tokens_preserved(code);
        for (range, _) in rust_highlights(&out) {
            assert!(out.is_char_boundary(range.start));
            assert!(out.is_char_boundary(range.end));
        }
    }

    #[test]
    fn dot_break_terminates_when_the_next_word_still_overflows() {
        // Regression canary for the reported dot-break hang: after breaking at
        // a chain dot, the word following the dot can itself overflow. The
        // break opportunity must not re-arm on resume (the dot is first on the
        // continuation line) or layout would spin forever.
        let word = "w".repeat(200);
        let code = format!("outer (f (x) . {word} (y) . label (\"end\"))");
        let out = format_rust_snippet(&code);
        assert_eq!(squeezed(&out), squeezed(&code));
        assert!(out.contains('\n'), "expected the chain to wrap");
    }

    #[test]
    fn highlights_survive_formatting() {
        let code = "row (vec ! [h :: Button :: new (\"btn\") . label (\"Hi\") . size (* s)])";
        let out = format_rust_snippet(code);
        let runs = rust_highlights(&out);
        let mut last = 0usize;
        for (range, _) in &runs {
            assert!(out.is_char_boundary(range.start));
            assert!(out.is_char_boundary(range.end));
            assert!(
                range.start >= last,
                "runs must not overlap after formatting"
            );
            last = range.end;
        }
        let expected = [
            (COMMENT, italic(COMMENT)),
            (KEYWORD, color(KEYWORD)),
            (STRING, color(STRING)),
            (NUMBER, color(NUMBER)),
            (TYPE, color(TYPE)),
            (FUNCTION, color(FUNCTION)),
            (LIFETIME, color(LIFETIME)),
        ];
        let painted: Vec<(&str, u32)> = runs
            .iter()
            .filter_map(|(r, s)| {
                let hex = expected.iter().find(|(_, style)| style.color == s.color)?.0;
                Some((&out[r.clone()], hex))
            })
            .collect();
        let vec_run = painted
            .iter()
            .find(|(text, _)| *text == "vec")
            .expect("macro `vec!` must be highlighted after formatting");
        assert_eq!(vec_run.1, FUNCTION);
        let texts: Vec<&str> = painted.iter().map(|(t, _)| *t).collect();
        assert!(texts.contains(&"Button"));
        assert!(texts.contains(&"\"btn\""));
        // The macro bang is glued, so `!` follows the `vec` run immediately.
        let vec_range = runs
            .iter()
            .find(|(r, _)| &out[r.clone()] == "vec")
            .unwrap()
            .0
            .clone();
        assert!(out[vec_range.end..].starts_with('!'));
    }
}
