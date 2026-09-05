use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use gpui::{prelude::*, px, AnyElement, App};
use herogpui_theme::ActiveTheme;

#[path = "reference_metadata.rs"]
mod reference_metadata;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ApiMethod {
    owner: String,
    name: String,
    signature: String,
    default: String,
    description: String,
}

pub fn panels(
    import_line: &str,
    examples: &[(&str, Option<&str>, AnyElement, &str)],
    cx: &App,
) -> Vec<(&'static str, AnyElement)> {
    if !REFERENCE_PANEL_HEADINGS
        .iter()
        .any(|heading| crate::control::section_wanted(heading, cx))
    {
        return Vec::new();
    }

    if let Some(metadata) = reference_metadata::for_import(import_line) {
        return metadata_panels(metadata, cx);
    }

    let Some(source) = source_for(import_line) else {
        return Vec::new();
    };
    let owners = referenced_types(import_line, examples.iter().map(|(_, _, _, code)| *code));
    let methods = methods_for(source, &owners);
    if methods.is_empty() {
        return Vec::new();
    }

    let styling: Vec<_> = methods
        .iter()
        .filter(|method| is_styling_method(&method.name))
        .cloned()
        .collect();
    let composition: Vec<_> = methods
        .iter()
        .filter(|method| is_composition_method(method))
        .cloned()
        .collect();

    vec![
        (
            "Styling Reference",
            if styling.is_empty() {
                empty_panel(
                    "This component has no component-specific appearance builders; it follows the active theme tokens.",
                    cx,
                )
            } else {
                method_table(&styling, cx)
            },
        ),
        ("API Reference", method_table(&methods, cx)),
        (
            "Element Composition & Callbacks",
            if composition.is_empty() {
                empty_panel(
                    "This component has no slot or callback builders; its rendered structure is fixed.",
                    cx,
                )
            } else {
                method_table(&composition, cx)
            },
        ),
    ]
}

#[derive(Clone, Debug)]
struct DetailRow {
    cells: [String; 4],
}

const REFERENCE_PANEL_HEADINGS: &[&str] = &[
    "Styling Reference",
    "API Reference",
    "Element Composition & Callbacks",
    "Parts & Slots",
    "States",
];

struct DisplayMetadata {
    api: Vec<DetailRow>,
    parts: Vec<DetailRow>,
    states: Vec<DetailRow>,
    styling: Vec<DetailRow>,
    contract: DetailRow,
}

static DISPLAY_METADATA: OnceLock<Vec<DisplayMetadata>> = OnceLock::new();

fn display_metadata() -> &'static [DisplayMetadata] {
    DISPLAY_METADATA.get_or_init(|| {
        reference_metadata::ALL
            .iter()
            .map(|metadata| {
                let source_count = metadata_source_count(metadata);
                DisplayMetadata {
                    api: metadata.api.iter().map(api_display_row).collect(),
                    parts: metadata.parts.iter().map(part_display_row).collect(),
                    states: metadata.states.iter().map(state_display_row).collect(),
                    styling: metadata.styling.iter().map(style_display_row).collect(),
                    contract: contract_display_row(metadata, source_count),
                }
            })
            .collect()
    })
}

fn cached_display_metadata(metadata: &reference_metadata::ReferenceMetadata) -> &DisplayMetadata {
    let index = display_cache_index(reference_metadata::ALL, metadata)
        .expect("registered reference metadata has a display cache entry");
    &display_metadata()[index]
}

/// Cache slots follow `ALL` order. Identity is `(page, import_line)`, the
/// same key `for_route` uses — a title-only lookup would collapse two
/// entries that share a page name. `const ALL` is copied at each use site,
/// so pointer equality cannot be the key.
fn display_cache_index(
    all: &[reference_metadata::ReferenceMetadata],
    metadata: &reference_metadata::ReferenceMetadata,
) -> Option<usize> {
    all.iter()
        .position(|entry| entry.page == metadata.page && entry.import_line == metadata.import_line)
}

fn metadata_source_count(metadata: &reference_metadata::ReferenceMetadata) -> usize {
    [
        metadata.docs_source,
        metadata.api_source,
        metadata.style_source,
    ]
    .iter()
    .flat_map(|source| source.split(" + "))
    .filter(|source| !source.is_empty())
    .count()
}

fn metadata_panels(
    metadata: &reference_metadata::ReferenceMetadata,
    cx: &App,
) -> Vec<(&'static str, AnyElement)> {
    let display = cached_display_metadata(metadata);

    vec![
        (
            "API Reference",
            detail_table(
                [
                    "Rust owner / prop",
                    "Value type (translated)",
                    "Builder / status",
                    "Default / behavior",
                ],
                display.api.iter(),
                cx,
            ),
        ),
        (
            "Parts & Slots",
            detail_table(
                [
                    "Rust owner / part",
                    "v3 slot (translated)",
                    "Status",
                    "Description",
                ],
                display.parts.iter(),
                cx,
            ),
        ),
        (
            "States",
            if metadata.states.is_empty() {
                empty_panel("v3 documents no interactive states for this component.", cx)
            } else {
                detail_table(
                    [
                        "State",
                        "v3 evidence (translated)",
                        "Rust implementation",
                        "Description",
                    ],
                    display.states.iter(),
                    cx,
                )
            },
        ),
        (
            "Styling Reference",
            detail_table(
                [
                    "v3 styling target (pinned)",
                    "Rust implementation",
                    "Status",
                    "Behavior",
                ],
                display
                    .styling
                    .iter()
                    .chain(std::iter::once(&display.contract)),
                cx,
            ),
        ),
    ]
}

// ---------------------------------------------------------------------------
// Display translation
//
// The checked-in metadata deliberately mirrors the pinned HeroUI v3.2.4
// contract, including the React/TypeScript spellings the audits verify. These
// helpers are the only place that metadata reaches a user, and they translate
// it at render time: Rust owners and builders are the actionable names,
// web-only rows are marked as not callable from GPUI, and React/JSX/DOM/CSS
// jargon is reworded into the pinned upstream evidence it stands for.
// `forbidden_display_token` and the display tests keep each other honest.
// ---------------------------------------------------------------------------

fn display_status(status: reference_metadata::ImplementationStatus) -> String {
    match status {
        reference_metadata::ImplementationStatus::Unavailable => {
            "Web-only — not callable from GPUI".to_owned()
        }
        other => other.label().to_owned(),
    }
}

fn api_display_row(entry: &reference_metadata::ApiDoc) -> DetailRow {
    let unavailable = entry.status == reference_metadata::ImplementationStatus::Unavailable;
    DetailRow {
        cells: [
            format!("{}::{}", entry.rust_owner, rust_prop_name(entry.prop)),
            rust_value_type(entry.ty),
            if unavailable {
                display_status(entry.status)
            } else {
                format!(
                    "{} · {}",
                    scrub_phrases(entry.rust),
                    display_status(entry.status)
                )
            },
            format!(
                "Default: {} — {}",
                scrub_prose(entry.default),
                scrub_prose(entry.description)
            ),
        ],
    }
}

fn part_display_row(entry: &reference_metadata::PartDoc) -> DetailRow {
    DetailRow {
        cells: [
            format!("{} · {}", entry.rust_owner, class_words(entry.name)),
            class_words(entry.slot),
            display_status(entry.status),
            scrub_prose(entry.description),
        ],
    }
}

fn rust_prop_name(prop: &str) -> String {
    prop.split(" / ")
        .map(|prop| match prop {
            "className" => "style_class".to_owned(),
            "htmlFor" => "label_for".to_owned(),
            _ => rust_identifier(prop),
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn rust_identifier(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch == '-' || ch == ' ' {
            if !out.ends_with('_') {
                out.push('_');
            }
        } else if ch.is_ascii_uppercase() {
            if !out.is_empty() && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out.trim_end_matches('_').to_owned()
}

fn state_display_row(entry: &reference_metadata::StateDoc) -> DetailRow {
    DetailRow {
        cells: [
            entry.state.to_owned(),
            scrub_prose(entry.selector),
            format!(
                "{} · {}",
                scrub_phrases(entry.rust),
                display_status(entry.status)
            ),
            scrub_prose(entry.description),
        ],
    }
}

fn style_display_row(entry: &reference_metadata::StyleDoc) -> DetailRow {
    DetailRow {
        cells: [
            format!(
                "{} — {}",
                scrub_prose(entry.class_or_token),
                scrub_prose(entry.value)
            ),
            scrub_phrases(entry.rust),
            display_status(entry.status),
            scrub_prose(entry.description),
        ],
    }
}

fn contract_display_row(
    metadata: &reference_metadata::ReferenceMetadata,
    source_count: usize,
) -> DetailRow {
    let part_suffix = if metadata.required_parts.len() == 1 {
        "part"
    } else {
        "parts"
    };
    DetailRow {
        cells: [
            "Pinned contract".to_owned(),
            format!("HeroUI v{}", metadata.version),
            format!("{source_count} pinned source links"),
            format!(
                "{} · source module {} · {} required compound {} · pinned upstream: {}",
                metadata.page,
                metadata.source_module,
                metadata.required_parts.len(),
                part_suffix,
                [
                    metadata.docs_source,
                    metadata.api_source,
                    metadata.style_source,
                ]
                .join(" · "),
            ),
        ],
    }
}

const TYPE_PHRASES: &[(&str, &str)] = &[
    ("FormEvent<HTMLFormElement>", "form submit event"),
    ("ChangeEvent<HTMLInputElement>", "change event"),
    ("ChangeEvent<HTMLTextAreaElement>", "change event"),
    ("SyntheticEvent<HTMLImageElement>", "web image event"),
    ("HTMLAttributes", "upstream attributes (not portable)"),
    ("HTMLFormElement", "form"),
    ("HTMLInputElement", "field"),
    ("HTMLTextAreaElement", "field"),
    ("HTMLButtonElement", "button"),
    ("HTMLDivElement", "element"),
    ("HTMLLegendElement", "legend"),
    ("HTMLFieldSetElement", "fieldset"),
    ("HTMLImageElement", "image"),
    ("HTMLElement", "element"),
    ("RefObject", "element reference (not portable)"),
    ("ValidityState", "web validity state"),
    ("DOMRenderFunction", "render closure"),
    ("RenderFunction", "render closure"),
    ("ReactNode", "AnyElement"),
    ("CSSProperties", "upstream style object (not portable)"),
    ("Iterable<Key>", "iterable of keys"),
    ("Iterable<T>", "iterable of T"),
    ("boolean", "bool"),
];

fn rust_value_type(ty: &str) -> String {
    let mut out = ty.replace("keyof React.JSX.IntrinsicElements, ", "");
    out = out.replace("React.", "");
    for (from, to) in TYPE_PHRASES {
        out = if *from == "RenderFunction" {
            replace_glued(&out, from, to)
        } else {
            out.replace(from, to)
        };
    }
    scrub_prose(&rust_closures(&out))
}

/// Replace `from` with `to`, inserting a space when the match is glued to a
/// preceding identifier (`CheckboxFieldRenderFunction` → `CheckboxField
/// render closure`).
fn replace_glued(text: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(from) {
        out.push_str(&rest[..pos]);
        if !out.is_empty()
            && out
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphanumeric())
            && to
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric())
        {
            out.push(' ');
        }
        out.push_str(to);
        rest = &rest[pos + from.len()..];
    }
    out.push_str(rest);
    out
}

/// Rewrites `(params) => return` arrow shapes as `Fn(params) -> return`.
fn rust_closures(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(arrow) = rest.find("=>") {
        let before = &rest[..arrow];
        let Some(open) = before.rfind('(') else {
            out.push_str(&rest[..arrow + 2]);
            rest = &rest[arrow + 2..];
            continue;
        };
        let after = &rest[arrow + 2..];
        let close = after.find(')').unwrap_or(after.len());
        out.push_str(&before[..open]);
        let params = before[open + 1..].trim();
        let params = params.strip_suffix(')').unwrap_or(params);
        out.push_str("Fn(");
        out.push_str(params);
        out.push_str(") -> ");
        out.push_str(after[..close].trim());
        rest = &after[close..];
    }
    out.push_str(rest);
    out.replace("void", "()")
}

const SCRUB_PHRASES: &[(&str, &str)] = &[
    // Pinned upstream framework names.
    ("the React Aria", "the pinned upstream"),
    ("the pinned React Aria", "the pinned upstream"),
    ("The pinned React Aria", "The pinned upstream"),
    (
        "Pinned React Aria Components 1.20.0",
        "The pinned upstream 1.20.0",
    ),
    (
        "pinned React Aria Components 1.20.0",
        "the pinned upstream 1.20.0",
    ),
    ("React Aria Components 1.20.0", "the pinned upstream 1.20.0"),
    ("React Aria/Stately", "the pinned upstream"),
    ("pinned React Aria's", "the pinned upstream's"),
    ("React Aria's", "the pinned upstream's"),
    ("Pinned React Aria", "The pinned upstream"),
    ("pinned React Aria", "the pinned upstream"),
    ("React Aria", "the pinned upstream"),
    ("pinned React Stately", "the pinned upstream"),
    ("React Stately", "the pinned upstream"),
    ("React collection nodes", "upstream collection items"),
    ("a React element", "an upstream element"),
    ("React element", "upstream element"),
    ("React prop", "upstream prop"),
    ("React", "upstream"),
    // DOM element, attribute, and render substitution.
    ("DOM element substitution", "upstream element substitution"),
    ("DOM render seam", "upstream render seam"),
    ("DOM render override", "upstream render override"),
    ("DOM render function", "upstream render closure"),
    (
        "DOM root substitution",
        "upstream root-element substitution",
    ),
    ("DOM props", "upstream props"),
    ("DOM attributes", "upstream attributes"),
    ("DOM classes", "upstream style classes"),
    ("DOM semantics", "web semantics"),
    ("DOM isolation", "web isolation"),
    ("DOM portal", "web portal"),
    ("DOM form boundary", "web form boundary"),
    ("DOM form element", "web form element"),
    ("DOM form owner", "web form owner"),
    ("DOM id attribute", "web id attribute"),
    ("DOM id graph", "web id graph"),
    ("caller DOM id", "caller element id"),
    ("DOM id", "web element id"),
    ("DOM nodes", "web elements"),
    ("DOM part", "web part"),
    ("DOM pointer-events", "web hit testing"),
    ("browser DOM root", "browser root element"),
    ("browser DOM", "the browser's element graph"),
    ("DOM element", "upstream element"),
    ("DOM", "web"),
    ("HTML", "web"),
    // Pointer and class instructions.
    ("pointer-events: none", "non-interactive"),
    ("pointer-events-none", "non-interactive"),
    ("no pointer-events", "no hit-testing control"),
    ("pointer-events", "hit testing"),
    ("className skeleton", "style-class skeleton"),
    ("through className", "through a style class"),
    ("className work", "style-class work"),
    ("className is unavailable", "style classes are unavailable"),
    ("className", "style class"),
    // Upstream data flags.
    ("v3's [data-current] rule", "v3's current-page marker rule"),
    ("[data-default-icon=true]", "the default-icon flag"),
    ("[data-current]", "the upstream current-page marker"),
    (
        "data-slot=popover-overlay-arrow",
        "the upstream arrow slot flag",
    ),
    ("data-slot", "the upstream slot flag"),
    ("data-direction", "the upstream direction flag"),
    ("data-default-icon", "the default-icon flag"),
    // Raw tokens that leak into prose.
    ("px-1", "a 4px horizontal inset"),
    ("shrink-0", "a fixed size"),
    (" .active", " active"),
    ("bg-field-hover", "the hover field fill"),
    ("--field-border-hover", "the hover field border token"),
    ("--default-hover", "the default hover token"),
    ("status-disabled", "the disabled status flag"),
    ("opacity-100", "full opacity"),
    ("opacity-0", "zero opacity"),
    (
        "preventDefault() to customize that focus",
        "the web-only cancelable-event mechanism to customize that focus",
    ),
    ("the FormData", "the submitted name/value record"),
    ("FormData", "a submitted name/value record"),
    ("PressEvent", "press event"),
    ("SyntheticEvent", "web event"),
    ("FormEvent", "form submit event"),
    ("ChangeEvent", "change event"),
    ("MouseEvent", "mouse event"),
    (
        "--tooltip-close-delay",
        "the pinned tooltip close-delay token",
    ),
    ("--tooltip-delay", "the pinned tooltip delay token"),
];

fn scrub_prose(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    loop {
        match rest.find('`') {
            Some(open) => {
                out.push_str(&scrub_plain(&rest[..open]));
                match rest[open + 1..].find('`') {
                    Some(close) => {
                        let span = &rest[open + 1..open + 1 + close];
                        out.push_str(&scrub_backticked(span));
                        rest = &rest[open + close + 2..];
                    }
                    None => {
                        out.push('`');
                        rest = &rest[open + 1..];
                    }
                }
            }
            None => {
                out.push_str(&scrub_plain(rest));
                break;
            }
        }
    }
    tidy(&mut out);
    out.trim().to_owned()
}

fn scrub_backticked(span: &str) -> String {
    if is_markup_tag_span(span) {
        return "upstream element".to_owned();
    }
    if span.contains("::")
        || (span.contains('(')
            && span.ends_with(')')
            && !span.contains(':')
            && !span.contains('=')
            && !span.contains("var(")
            && !span.starts_with("--"))
        || (!span.contains('-') && !span.contains('.') && span.contains('_'))
    {
        return format!("`{span}`");
    }
    if matches!(span, "body-sm" | "sm" | "md" | "lg" | "xs" | "xl") {
        return format!("`{span}`");
    }
    if span.ends_with(".rs") {
        return "the checked-in reference reader".to_owned();
    }
    if span.ends_with(".tsx") || span.ends_with(".css") {
        return "the pinned upstream source".to_owned();
    }
    if span.starts_with('.') || span.contains("__") {
        return class_words(span);
    }
    if span.starts_with("is") && span.chars().nth(2).is_some_and(|ch| ch.is_uppercase()) {
        return class_words(span);
    }
    if span.contains(':')
        || span.contains('[')
        || span.contains(']')
        || span.contains('{')
        || span.contains('}')
        || span.contains('=')
        || span.starts_with("--")
        || span.contains("var(")
        || span.contains('.')
        || span.contains('-')
    {
        return "(upstream CSS)".to_owned();
    }
    let scrubbed = scrub_plain(span);
    if forbidden_display_token(&scrubbed).is_some() {
        "(pinned upstream evidence)".to_owned()
    } else {
        scrubbed
    }
}

/// Exact-phrase scrubbing only; safe for Rust evidence strings like
/// `gap(px(16.))`, which the structural passes would reword.
fn scrub_phrases(text: &str) -> String {
    let mut out = text.to_owned();
    for (from, to) in SCRUB_PHRASES {
        if from.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            out = replace_word(&out, from, to);
        } else {
            out = out.replace(from, to);
        }
    }
    let mut out = out.replace("RenderProps", " render props");
    tidy(&mut out);
    out
}

/// Collapses doubled spaces and doubled articles left by replacements;
/// some doubles only appear once text segments are assembled.
fn tidy(text: &mut String) {
    while text.contains("  ") {
        *text = text.replace("  ", " ");
    }
    while text.contains("the the") {
        *text = text.replace("the the", "the");
    }
    while text.contains("The the") {
        *text = text.replace("The the", "The");
    }
    let fixed = fix_articles(text);
    *text = fixed;
}

/// Repairs `a`/`an` agreement after replacements ("a React Aria" →
/// "a the pinned upstream") and drops the now-redundant article in front of
/// replacements that embed their own ("an href" → "an the link URL").
/// Lowercase followers only, so Rust identifiers are never touched.
fn fix_articles(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let at_word_start = i == 0 || !chars[i - 1].is_ascii_alphanumeric();
        let (article, capitalized) = if at_word_start && chars[i..].starts_with(&['a', 'n']) {
            ("an", false)
        } else if at_word_start && chars[i..].starts_with(&['a', ' ']) {
            ("a", false)
        } else if at_word_start && chars[i..].starts_with(&['A', 'n']) {
            ("an", true)
        } else if at_word_start && chars[i..].starts_with(&['A', ' ']) {
            ("a", true)
        } else {
            out.push(chars[i]);
            i += 1;
            continue;
        };
        let mut cursor = i + article.len();
        if chars.get(cursor) != Some(&' ') {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        cursor += 1;
        let follower: String = chars[cursor..]
            .iter()
            .take_while(|ch| ch.is_ascii_lowercase())
            .collect();
        if follower == "the" || follower == "a" || follower == "an" {
            // The follower carries its own article; drop ours.
            if follower == "the" && capitalized {
                out.push('T');
            }
            i = cursor;
            continue;
        }
        if !follower.is_empty() {
            let wants_an = wants_an_article(&follower);
            let is_an = article == "an";
            if wants_an != is_an {
                if capitalized {
                    out.push_str(if wants_an { "An " } else { "A " });
                } else if wants_an {
                    out.push_str("an ");
                } else {
                    out.push_str("a ");
                }
                i = cursor;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// "an" goes before vowel sounds; u-words pronounced "yu" ("a uniform")
/// are the exception.
fn wants_an_article(word: &str) -> bool {
    const YU_WORDS: &[&str] = &[
        "uniform", "unit", "user", "unique", "usual", "union", "utility", "use",
    ];
    match word.as_bytes().first() {
        Some(b'a' | b'e' | b'i' | b'o') => true,
        Some(b'u') => !YU_WORDS.iter().any(|yu| word.starts_with(yu)),
        _ => false,
    }
}

fn scrub_plain(text: &str) -> String {
    let out = scrub_phrases(text);
    let out = scrub_markup(&scrub_prose_href(&out));
    let mut out = scrub_bracket_flags(&out);
    out = scrub_class_tokens(&out);
    out = scrub_pseudo_selectors(&out);
    out = scrub_css_vars(&out);
    let mut out = scrub_utility_tokens(&out);
    tidy(&mut out);
    out
}

fn scrub_markup(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        let Some(close_offset) = rest[open..].find('>') else {
            out.push_str(rest);
            return out;
        };
        let close = open + close_offset;
        if is_markup_tag_at(rest, open, close) {
            out.push_str(&rest[..open]);
            out.push_str("upstream element");
            rest = &rest[close + 1..];
        } else {
            out.push_str(&rest[..open + 1]);
            rest = &rest[open + 1..];
        }
    }
    out.push_str(rest);
    out
}

fn is_markup_tag_span(span: &str) -> bool {
    span.starts_with('<') && span.ends_with('>') && is_markup_tag(&span[1..span.len() - 1])
}

fn is_markup_tag(tag: &str) -> bool {
    let tag = tag.trim();
    let tag = tag.strip_prefix('/').unwrap_or(tag).trim_start();
    let name_end =
        tag.find(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')));
    let (name, _rest) = name_end.map_or((tag, ""), |end| tag.split_at(end));
    if name.is_empty() {
        return false;
    }
    let component = name.len() > 1 && name.starts_with(char::is_uppercase);
    let html = matches!(
        name,
        "a" | "abbr"
            | "button"
            | "code"
            | "div"
            | "fieldset"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "img"
            | "input"
            | "label"
            | "p"
            | "span"
    );
    component || html
}

fn contains_markup_tag(text: &str) -> bool {
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        let Some(close_offset) = rest[open..].find('>') else {
            return false;
        };
        let close = open + close_offset;
        if is_markup_tag_at(rest, open, close) {
            return true;
        }
        rest = &rest[open + 1..];
    }
    false
}

fn is_markup_tag_at(text: &str, open: usize, close: usize) -> bool {
    text[..open]
        .chars()
        .next_back()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_' && ch != ':')
        && is_markup_tag(&text[open + 1..close])
}

/// Prose-only rewording of `href`; Rust builder evidence such as
/// `href(url)` is handled by `scrub_phrases` and must keep the call name.
/// A `href` followed by `(` or preceded by `.` is a builder call, not prose.
fn scrub_prose_href(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let at_word_start = i == 0 || !chars[i - 1].is_ascii_alphanumeric();
        if at_word_start && chars[i..].starts_with(&['h', 'r', 'e', 'f']) {
            let after = i + 4;
            let boundary = chars
                .get(after)
                .is_none_or(|ch| !ch.is_ascii_alphanumeric());
            let preceded_by_dot = i > 0 && chars[i - 1] == '.';
            let is_call = chars.get(after).is_some_and(|ch| *ch == '(' || *ch == '!');
            if boundary && !preceded_by_dot && !is_call {
                out.push_str("the link URL");
                i = after;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Word-boundary replace so `DOM` does not hit inside longer identifiers.
fn replace_word(text: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(from) {
        let after = pos + from.len();
        let before_ok = pos == 0
            || !rest[..pos]
                .chars()
                .next_back()
                .is_some_and(|ch| ch.is_ascii_alphanumeric());
        let after_ok = after == rest.len()
            || !rest[after..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric());
        if before_ok && after_ok {
            out.push_str(&rest[..pos]);
            out.push_str(to);
        } else {
            out.push_str(&rest[..after]);
        }
        rest = &rest[after..];
    }
    out.push_str(rest);
    out
}

/// `[data-x="y"]` and `[aria-x]` attribute spellings become words.
fn scrub_bracket_flags(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find(']') else {
            break;
        };
        out.push_str(&rest[..open]);
        if out
            .chars()
            .next_back()
            .is_some_and(|ch| ch.is_ascii_alphanumeric())
        {
            out.push(' ');
        }
        out.push_str(&flag_words(rest[open + 1..open + close].trim()));
        rest = &rest[open + close + 1..];
    }
    out.push_str(rest);
    out
}

fn flag_words(inner: &str) -> String {
    let (name, value) = match inner.split_once('=') {
        Some((name, value)) => (
            name,
            Some(value.trim().trim_matches('"').trim_matches('\'')),
        ),
        None => (inner, None),
    };
    let name = name
        .strip_prefix("data-")
        .or_else(|| name.strip_prefix("aria-"))
        .unwrap_or(name);
    let mut words = class_words(name);
    if let Some(value) = value {
        if !value.is_empty() && value != "true" {
            words.push(' ');
            words.push_str(&class_words(value));
        }
    }
    words
}

/// Leading-dot CSS class tokens become their component words.
fn scrub_class_tokens(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '.'
            && !chars[..i]
                .last()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || *ch == ')')
            && chars.get(i + 1).is_some_and(|ch| ch.is_ascii_lowercase())
        {
            let mut j = i + 1;
            while j < chars.len()
                && (chars[j].is_ascii_alphanumeric() || matches!(chars[j], '_' | '-'))
            {
                j += 1;
            }
            if chars.get(j) == Some(&'(') {
                // `.px(` style Rust method calls are not class tokens.
                out.push('.');
                i += 1;
                continue;
            }
            let token: String = chars[i + 1..j].iter().collect();
            out.push_str(&class_words(&token));
            i = j;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn scrub_pseudo_selectors(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ':' {
            let mut run_end = i;
            while chars.get(run_end) == Some(&':') {
                run_end += 1;
            }
            let mut j = run_end;
            while j < chars.len() && (chars[j].is_ascii_lowercase() || chars[j] == '-') {
                j += 1;
            }
            if j > run_end {
                let name: String = chars[run_end..j].iter().collect();
                match pseudo_words(&name) {
                    Some(words) => out.push_str(&words),
                    // Unknown name: keep the original spelling so Rust paths
                    // like `ParentElement::extend` survive display untouched.
                    None => out.extend(chars[i..j].iter()),
                }
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn pseudo_words(name: &str) -> Option<String> {
    let words = match name {
        "hover" => " hover",
        "active" => " pressed",
        "focus-visible" => " keyboard focus",
        "focus-within" => " focus within",
        "focus" => " focus",
        "disabled" => " disabled",
        "empty" => " empty",
        "first-child" => " first member",
        "last-child" => " last member",
        "checked" => " selected",
        "indeterminate" => " indeterminate",
        "read-only" => " read-only",
        "required" => " required",
        "invalid" => " invalid",
        "not" => " not",
        "has" => " has",
        "is" => " is",
        "where" => " where",
        "placeholder-shown" => " placeholder shown",
        "before" => " before",
        "after" => " after",
        "placeholder" => " placeholder",
        "autofill" => " autofill",
        _ => return None,
    };
    Some(words.to_owned())
}

fn scrub_css_vars(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '-'
            && chars.get(i + 1) == Some(&'-')
            && chars.get(i + 2).is_some_and(|ch| ch.is_ascii_lowercase())
        {
            let mut j = i + 2;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '-') {
                j += 1;
            }
            let name: String = chars[i + 2..j].iter().collect();
            out.push_str("the upstream token ");
            out.push_str(&class_words(&name));
            i = j;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

const UTILITY_PREFIXES: &[&str] = &[
    "bg",
    "text",
    "border",
    "px",
    "py",
    "p",
    "m",
    "mx",
    "my",
    "ms",
    "me",
    "mt",
    "mb",
    "ps",
    "pe",
    "pt",
    "pb",
    "pl",
    "pr",
    "gap",
    "h",
    "w",
    "size",
    "min",
    "max",
    "rounded",
    "opacity",
    "shadow",
    "items",
    "justify",
    "font",
    "leading",
    "tracking",
    "inset",
    "translate",
    "rotate",
    "scale",
    "duration",
    "transition",
    "animate",
    "delay",
    "ease",
    "overflow",
    "whitespace",
    "wrap",
    "flex",
    "grid",
    "place",
    "self",
    "order",
    "basis",
    "grow",
    "shrink",
    "aspect",
    "ring",
    "outline",
    "divide",
    "space",
    "underline",
    "decoration",
    "select",
    "cursor",
    "caret",
    "accent",
    "resize",
    "appearance",
    "will",
    "contain",
    "isolation",
    "touch",
    "scroll",
    "snap",
    "fill",
    "stroke",
    "data",
    "after",
    "before",
    "peer",
    "group",
    "motion",
    "supports",
    "dark",
    "print",
    "sm",
    "md",
    "lg",
    "xl",
];

/// Leftover Tailwind-style utility tokens become a pinned-evidence note.
fn scrub_utility_tokens(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let word_start = i == 0 || !chars[i - 1].is_ascii_alphanumeric() && chars[i - 1] != '-';
        let negative =
            chars[i] == '-' && chars.get(i + 1).is_some_and(|ch| ch.is_ascii_lowercase());
        if word_start && (chars[i].is_ascii_lowercase() || negative) {
            // `sm:mt-0`, `hover:bg-accent`, `dark:sm:p-2`, `-mx-0.5` — optional
            // leading minus, variant prefixes, then the utility. A Rust path
            // uses `::` and is left alone.
            let mut j = if negative { i + 1 } else { i };
            let mut saw_variant = false;
            loop {
                let mut k = j;
                while k < chars.len()
                    && (chars[k].is_ascii_lowercase()
                        || chars[k].is_ascii_digit()
                        || chars[k] == '-')
                {
                    k += 1;
                }
                if k > j
                    && chars.get(k) == Some(&':')
                    && chars.get(k + 1) != Some(&':')
                    && chars.get(k + 1).is_some_and(|ch| ch.is_ascii_lowercase())
                {
                    saw_variant = true;
                    j = k + 1;
                    continue;
                }
                break;
            }
            let mut end = j;
            while end < chars.len()
                && (chars[end].is_ascii_alphanumeric()
                    || chars[end] == '-'
                    || chars[end] == '%'
                    || (chars[end] == '.'
                        && chars.get(end + 1).is_some_and(|ch| ch.is_ascii_digit()))
                    || (chars[end] == '/'
                        && chars.get(end + 1).is_some_and(|ch| ch.is_ascii_digit())))
            {
                end += 1;
            }
            let token: String = chars[j..end].iter().collect();
            if utility_token(&token)
                || (saw_variant
                    && !token.is_empty()
                    && token.chars().all(|ch| {
                        ch.is_ascii_lowercase()
                            || ch.is_ascii_digit()
                            || matches!(ch, '-' | '/' | '.' | '%')
                    }))
            {
                out.push_str("(upstream CSS)");
                i = end;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn utility_token(token: &str) -> bool {
    let mut segments = token.split('-');
    let Some(first) = segments.next() else {
        return false;
    };
    if !UTILITY_PREFIXES.contains(&first) {
        return false;
    }
    let mut rest: Vec<&str> = Vec::new();
    for segment in segments {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '/' | '%'))
        {
            return false;
        }
        rest.push(segment);
    }
    if rest.is_empty() {
        return false;
    }
    // Prose compounds like "size-specific" are not utilities; real utilities
    // carry a numeric step or a known value word.
    rest.iter().any(|segment| {
        segment.chars().any(|ch| ch.is_ascii_digit())
            || matches!(
                *segment,
                "sm" | "md"
                    | "lg"
                    | "xl"
                    | "xs"
                    | "full"
                    | "fit"
                    | "auto"
                    | "none"
                    | "center"
                    | "start"
                    | "end"
                    | "between"
                    | "around"
                    | "evenly"
                    | "baseline"
                    | "nowrap"
                    | "wrap"
                    | "clip"
                    | "hidden"
                    | "visible"
                    | "current"
                    | "stretch"
                    | "default"
                    | "accent"
                    | "success"
                    | "warning"
                    | "danger"
                    | "muted"
                    | "foreground"
                    | "background"
                    | "field"
                    | "overlay"
                    | "placeholder"
                    | "break"
                    | "word"
                    | "ellipsis"
            )
    })
}

/// Splits BEM, kebab, and camel spellings into component words.
fn class_words(token: &str) -> String {
    let cleaned: String = token
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect();
    let mut words: Vec<String> = Vec::new();
    for word in cleaned.split_whitespace() {
        let mut current = String::new();
        let mut previous_lower = false;
        for ch in word.chars() {
            if ch.is_uppercase() && previous_lower {
                words.push(current.clone());
                current.clear();
            }
            current.extend(ch.to_lowercase());
            previous_lower = ch.is_lowercase() || ch.is_ascii_digit();
        }
        if !current.is_empty() {
            words.push(current);
        }
    }
    words.join(" ")
}

fn forbidden_display_token(text: &str) -> Option<&'static str> {
    if contains_markup_tag(text) {
        return Some("JSX/HTML tag");
    }
    const TOKENS: &[&str] = &[
        "React",
        "JSX",
        "DOM",
        "RenderFunction",
        "RenderProps",
        "CSSProperties",
        "className",
        "HTML",
        "pointer-events",
    ];
    for token in TOKENS {
        if text.contains(token) {
            return Some(token);
        }
    }
    let chars: Vec<char> = text.chars().collect();
    let ident_end = |start: usize| -> usize {
        let mut end = start;
        while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        end
    };
    for (i, ch) in chars.iter().enumerate() {
        match ch {
            '.' => {
                let boundary =
                    i == 0 || (!chars[i - 1].is_ascii_alphanumeric() && chars[i - 1] != ')');
                if boundary
                    && chars.get(i + 1).is_some_and(|ch| ch.is_ascii_lowercase())
                    // `.px(` style Rust method calls are not class tokens.
                    && chars.get(ident_end(i + 1)) != Some(&'(')
                {
                    return Some("CSS class token");
                }
            }
            ':' => {
                let path = i > 0 && chars[i - 1] == ':' || chars.get(i + 1) == Some(&':');
                if !path && chars.get(i + 1).is_some_and(|ch| ch.is_ascii_lowercase()) {
                    return Some("pseudo selector or CSS declaration");
                }
            }
            '-' if chars.get(i + 1) == Some(&'-')
                && chars.get(i + 2).is_some_and(|ch| ch.is_ascii_lowercase()) =>
            {
                return Some("CSS custom property");
            }
            _ => {}
        }
    }
    let mut start = 0;
    while start < chars.len() {
        if chars[start].is_ascii_alphanumeric() {
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '-') {
                end += 1;
            }
            let token: String = chars[start..end].iter().collect();
            if utility_token(&token) {
                return Some("utility class token");
            }
            start = end;
        } else {
            start += 1;
        }
    }
    None
}

fn referenced_types<'a>(
    import_line: &str,
    examples: impl Iterator<Item = &'a str>,
) -> BTreeSet<String> {
    let mut owners = BTreeSet::new();
    collect_uppercase_identifiers(import_line, &mut owners);
    for example in examples {
        let mut rest = example;
        while let Some(start) = rest.find("h::") {
            rest = &rest[start + 3..];
            let end = rest
                .find(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .unwrap_or(rest.len());
            let name = &rest[..end];
            if name.starts_with(char::is_uppercase) {
                owners.insert(name.to_owned());
            }
            rest = &rest[end..];
        }
    }
    owners
}

fn collect_uppercase_identifiers(text: &str, out: &mut BTreeSet<String>) {
    for token in text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_') {
        if token.starts_with(char::is_uppercase) {
            out.insert(token.to_owned());
        }
    }
}

fn methods_for(source: &str, owners: &BTreeSet<String>) -> Vec<ApiMethod> {
    let lines: Vec<_> = source.lines().collect();
    let mut methods = Vec::new();
    let mut defaults = BTreeMap::new();
    let mut owner: Option<&str> = None;
    let mut depth = 0_i32;
    let mut docs = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if owner.is_none() {
            if let Some(candidate) = inherent_impl_owner(trimmed) {
                if owners.contains(candidate) {
                    owner = Some(candidate);
                    depth = brace_delta(line);
                }
            }
            index += 1;
            continue;
        }

        if depth == 1 && trimmed.starts_with("///") {
            docs.push(trimmed.trim_start_matches("///").trim().to_owned());
        } else if depth == 1 && trimmed.starts_with("pub fn ") {
            let mut signature = trimmed.to_owned();
            let mut method_depth = brace_delta(line);
            while !signature.contains(" {") && !signature.ends_with('{') {
                index += 1;
                if index == lines.len() {
                    break;
                }
                signature.push(' ');
                signature.push_str(lines[index].trim());
                method_depth += brace_delta(lines[index]);
            }
            let current_owner = owner.expect("owner is set");
            let mut body = Vec::new();
            while method_depth > 0 {
                index += 1;
                if index == lines.len() {
                    break;
                }
                method_depth += brace_delta(lines[index]);
                body.push(lines[index]);
            }
            if signature.starts_with("pub fn new(") {
                collect_defaults(current_owner, &body, &mut defaults);
            }
            if let Some(method) = api_method(current_owner, &signature, &docs) {
                methods.push(method);
            }
            docs.clear();
            index += 1;
            continue;
        } else if depth == 1 && !trimmed.is_empty() && !trimmed.starts_with("#[") {
            docs.clear();
        }

        depth += brace_delta(line);
        if depth == 0 {
            owner = None;
            docs.clear();
        }
        index += 1;
    }

    methods.sort_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then_with(|| left.name.cmp(&right.name))
    });
    methods.dedup_by(|left, right| left.owner == right.owner && left.name == right.name);
    for method in &mut methods {
        method.default = defaults
            .get(&(method.owner.clone(), method.name.clone()))
            .cloned()
            .unwrap_or_else(|| "—".to_owned());
        if method.description.is_empty() {
            method.description = inferred_description(method);
        }
    }
    for owner in owners {
        if source.contains(&format!("impl ParentElement for {owner}"))
            && !methods
                .iter()
                .any(|method| method.owner == owner.as_str() && method.name == "child")
        {
            methods.push(ApiMethod {
                owner: owner.clone(),
                name: "child".to_owned(),
                signature: "child(self, element: impl IntoElement) -> Self".to_owned(),
                default: "—".to_owned(),
                description: format!("Adds composed content to {owner}."),
            });
        }
    }
    methods.sort_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then_with(|| left.name.cmp(&right.name))
    });
    methods
}

fn collect_defaults(owner: &str, body: &[&str], defaults: &mut BTreeMap<(String, String), String>) {
    let mut in_self = false;
    for line in body {
        let trimmed = line.trim();
        if trimmed.starts_with("Self {") {
            in_self = true;
            continue;
        }
        if !in_self || trimmed == "}" || trimmed == "}," {
            continue;
        }
        let Some((field, value)) = trimmed.trim_end_matches(',').split_once(':') else {
            continue;
        };
        if field
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            defaults.insert(
                (owner.to_owned(), field.to_owned()),
                value.trim().to_owned(),
            );
        }
    }
}

fn inherent_impl_owner(line: &str) -> Option<&str> {
    let body = line.strip_prefix("impl ")?.strip_suffix('{')?.trim();
    if body.contains(" for ") || body.starts_with('<') {
        return None;
    }
    body.split('<').next().map(str::trim)
}

fn brace_delta(line: &str) -> i32 {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '/' && chars.peek() == Some(&'/') {
            break;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }
    depth
}

fn api_method(owner: &str, signature: &str, docs: &[String]) -> Option<ApiMethod> {
    let signature = signature
        .strip_prefix("pub fn ")?
        .trim_end_matches('{')
        .trim();
    let name_end = signature.find(['(', '<'])?;
    let name = signature[..name_end].to_owned();
    let description = docs
        .iter()
        .take_while(|line| !line.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('`', "");
    Some(ApiMethod {
        owner: owner.to_owned(),
        name,
        signature: signature.to_owned(),
        default: String::new(),
        description,
    })
}

fn inferred_description(method: &ApiMethod) -> String {
    let owner = split_camel_case(&method.owner).to_lowercase();
    if method.name == "new" {
        return format!("Creates a {owner}.");
    }
    let words = method.name.replace('_', " ");
    if let Some(event) = words.strip_prefix("on ") {
        format!("Registers the {event} callback.")
    } else if let Some(state) = words.strip_prefix("is ") {
        format!("Sets whether the {owner} is {state}.")
    } else {
        format!("Sets the {owner} {words}.")
    }
}

fn split_camel_case(text: &str) -> String {
    let mut result = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index > 0 && ch.is_uppercase() {
            result.push(' ');
        }
        result.push(ch);
    }
    result
}

fn is_styling_method(name: &str) -> bool {
    if name.starts_with("on_") {
        return false;
    }
    const WORDS: [&str; 36] = [
        "variant",
        "size",
        "color",
        "radius",
        "orientation",
        "placement",
        "position",
        "align",
        "width",
        "height",
        "weight",
        "shape",
        "full_width",
        "icon_only",
        "disabled",
        "pending",
        "invalid",
        "read_only",
        "selected",
        "open",
        "loading",
        "dismissible",
        "required",
        "pressable",
        "striped",
        "indeterminate",
        "compact",
        "wrap",
        "gap",
        "padding",
        "offset",
        "max_h",
        "max_w",
        "min_h",
        "min_w",
        "spacing",
    ];
    WORDS.iter().any(|word| name.contains(word))
}

fn is_composition_method(method: &ApiMethod) -> bool {
    const NAMES: [&str; 22] = [
        "child",
        "children",
        "content",
        "label",
        "description",
        "indicator",
        "icon",
        "start_content",
        "end_content",
        "header",
        "footer",
        "trigger",
        "empty_content",
        "value_content",
        "render",
        "item",
        "button",
        "crumb",
        "column",
        "row",
        "tag",
        "field",
    ];
    method.name.starts_with("on_")
        || NAMES.iter().any(|name| method.name == *name)
        || method.signature.contains("Fn(")
        || method.signature.contains("IntoElement")
        || method.signature.contains("AnyElement")
}

fn method_table(methods: &[ApiMethod], cx: &App) -> AnyElement {
    let colors = cx.colors();
    let header = gpui::div()
        .flex()
        .w_full()
        .px(px(14.))
        .py(px(10.))
        .rounded_t(px(12.))
        .bg(colors.surface_secondary)
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .gap(px(12.))
        .child(gpui::div().w(px(160.)).flex_shrink_0().child("Builder"))
        .child(gpui::div().w(px(220.)).flex_shrink_0().child("Signature"))
        .child(gpui::div().w(px(130.)).flex_shrink_0().child("Default"))
        .child(gpui::div().w(px(286.)).flex_shrink_0().child("Description"));

    gpui::div()
        .w_full()
        .rounded(px(12.))
        .border_1()
        .border_color(colors.border)
        .overflow_hidden()
        .child(header)
        .children(methods.iter().map(|method| {
            gpui::div()
                .flex()
                .w_full()
                .px(px(14.))
                .py(px(11.))
                .border_t_1()
                .border_color(colors.border)
                .text_size(px(12.5))
                .line_height(px(20.))
                .gap(px(12.))
                .child(
                    gpui::div()
                        .w(px(160.))
                        .flex_shrink_0()
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(format!("{}::{}", method.owner, method.name)),
                )
                .child(
                    gpui::div()
                        .w(px(220.))
                        .flex_shrink_0()
                        .pr(px(16.))
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(method.signature.clone()),
                )
                .child(
                    gpui::div()
                        .w(px(130.))
                        .flex_shrink_0()
                        .pr(px(12.))
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(method.default.clone()),
                )
                .child(
                    gpui::div()
                        .w(px(286.))
                        .flex_shrink_0()
                        .text_color(colors.muted)
                        .child(if method.description.is_empty() {
                            "—".to_owned()
                        } else {
                            method.description.clone()
                        }),
                )
        }))
        .into_any_element()
}

fn detail_table<'a>(
    headers: [&'static str; 4],
    rows: impl IntoIterator<Item = &'a DetailRow>,
    cx: &App,
) -> AnyElement {
    let colors = cx.colors();
    let header = gpui::div()
        .flex()
        .w_full()
        .px(px(14.))
        .py(px(10.))
        .rounded_t(px(12.))
        .bg(colors.surface_secondary)
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .gap(px(12.))
        .child(gpui::div().flex_1().min_w_0().child(headers[0]))
        .child(gpui::div().flex_1().min_w_0().child(headers[1]))
        .child(gpui::div().flex_1().min_w_0().child(headers[2]))
        .child(gpui::div().flex_1().min_w_0().child(headers[3]));

    gpui::div()
        .w_full()
        .rounded(px(12.))
        .border_1()
        .border_color(colors.border)
        .overflow_hidden()
        .child(header)
        .children(rows.into_iter().map(|row| {
            gpui::div()
                .flex()
                .w_full()
                .px(px(14.))
                .py(px(11.))
                .border_t_1()
                .border_color(colors.border)
                .text_size(px(12.5))
                .line_height(px(20.))
                .gap(px(12.))
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(row.cells[0].clone()),
                )
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(row.cells[1].clone()),
                )
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .text_color(colors.foreground)
                        .child(row.cells[2].clone()),
                )
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .text_color(colors.muted)
                        .child(row.cells[3].clone()),
                )
        }))
        .into_any_element()
}

fn empty_panel(message: &str, cx: &App) -> AnyElement {
    let colors = cx.colors();
    gpui::div()
        .w_full()
        .px(px(16.))
        .py(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(colors.border)
        .text_size(px(13.))
        .text_color(colors.muted)
        .child(message.to_owned())
        .into_any_element()
}

fn source_for(import_line: &str) -> Option<&'static str> {
    let module = if import_line.contains("herogpui::prelude") {
        "button"
    } else {
        import_line
            .split_once("components::")?
            .1
            .split("::")
            .next()?
    };
    source_for_module(module)
}

fn source_for_module(module: &str) -> Option<&'static str> {
    herogpui_components::gallery_source::source_for(module)
}

#[cfg(test)]
fn mapping_method_name(mapping: &str) -> Option<&str> {
    let prefix = mapping.split_once('(')?.0.trim();
    let name = prefix.rsplit("::").next()?.trim();
    (!name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
    .then_some(name)
}

#[cfg(test)]
fn mapping_matches_method(methods: &[ApiMethod], owner: &str, mapping: &str) -> bool {
    let Some(name) = mapping_method_name(mapping) else {
        return false;
    };
    methods.iter().any(|method| {
        method.owner == owner
            && method.name == name
            && method.signature.contains('(')
            && argument_count(mapping)
                .is_some_and(|count| method_argument_count(&method.signature) == Some(count))
    })
}

#[cfg(test)]
fn argument_count(signature: &str) -> Option<usize> {
    let start = signature.find('(')?;
    let mut depth = 0;
    let mut count = 0;
    let mut has_argument = false;
    for ch in signature[start + 1..].chars() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' if depth == 0 => {
                return Some(count + usize::from(has_argument));
            }
            ')' | ']' | '>' if depth > 0 => depth -= 1,
            ',' if depth == 0 => {
                count += 1;
                has_argument = false;
            }
            ch if !ch.is_whitespace() => has_argument = true,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
fn method_argument_count(signature: &str) -> Option<usize> {
    let count = argument_count(signature)?;
    let start = signature.find('(')? + 1;
    let mut depth = 0;
    let mut end = signature.len();
    for (offset, ch) in signature[start..].char_indices() {
        match ch {
            '(' | '[' | '<' => depth += 1,
            ')' if depth == 0 => {
                end = start + offset;
                break;
            }
            ',' if depth == 0 => {
                end = start + offset;
                break;
            }
            ')' | ']' | '>' => depth -= 1,
            _ => {}
        }
    }
    let first = &signature[start..end];
    Some(count.saturating_sub(usize::from(
        first.trim_start().starts_with("self")
            || first.trim_start().starts_with("mut self")
            || first.trim_start().starts_with("&self")
            || first.trim_start().starts_with("&mut self"),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_reference_reads_public_inherent_builders() {
        let owners = BTreeSet::from(["Button".to_owned()]);
        let methods = methods_for(source_for_module("button").unwrap(), &owners);

        assert!(methods.iter().any(|method| method.name == "variant"));
        assert!(methods.iter().any(|method| method.name == "on_press"));
        assert!(methods
            .iter()
            .any(|method| method.name == "variant" && method.default == "Variant::Primary"));
        assert!(methods.iter().any(|method| method.name == "child"));
        assert!(!methods.iter().any(|method| method.name == "render"));
    }

    #[test]
    fn form_metadata_tracks_pinned_submission_contract() {
        let metadata =
            reference_metadata::for_route("Form", "use herogpui::components::form::Form;")
                .expect("Form metadata is registered");

        // Exactly the pinned v3.2.4 API table — 14 rows, and v3 documents no
        // `isDisabled` on Form: a form-level disable was a v2 leftover and is
        // gone from the port entirely.
        assert_eq!(metadata.api.len(), 14);
        assert!(metadata.api.iter().all(|entry| entry.prop != "isDisabled"));
        let implemented = reference_metadata::ImplementationStatus::Implemented;
        let partial = reference_metadata::ImplementationStatus::Partial;
        let unavailable = reference_metadata::ImplementationStatus::Unavailable;
        for prop in [
            "action",
            "className",
            "children",
            "encType",
            "method",
            "onInvalid",
            "onReset",
            "onSubmit",
            "target",
            "validationBehavior",
            "validationErrors",
            "aria-label",
            "aria-labelledby",
            "render",
        ] {
            assert!(
                metadata.api.iter().any(|entry| entry.prop == prop),
                "the pinned row {prop} must be documented"
            );
        }

        // The HTTP half and the DOM/accessibility spellings are honest holes.
        for prop in [
            "action",
            "className",
            "encType",
            "method",
            "target",
            "aria-label",
            "aria-labelledby",
            "render",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.prop == prop && entry.status == unavailable && entry.rust == "—"
            }));
        }

        // onSubmit carries the record-shape wording: what the submission
        // looks like, however it arrived.
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "onSubmit"
                && entry.status == implemented
                && entry.description.contains("name=value")
                && entry.description.contains("registration order")
        }));
        // onInvalid keeps the default focus claim and names the missing
        // cancelable event rather than implying it.
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "onInvalid"
                && entry.status == partial
                && entry.description.contains("first invalid field")
                && entry.description.contains("preventDefault")
        }));
        // validationErrors is typed exactly as v3's alias, and now that the
        // record is routed the Implemented row names every load-bearing
        // mechanism rather than the flat Vec it replaced.
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "validationErrors"
                && entry.status == implemented
                && entry.ty.contains("Record<string, string | string[]>")
                && entry.description.contains("routed by field name")
                && entry.description.contains("suppresses only its messages")
                && entry.description.contains("re-arms every named field")
                && entry
                    .description
                    .contains("clone keeps the record's identity")
                && entry.description.contains("never block invisibly")
        }));
        for prop in ["onReset", "validationBehavior"] {
            assert!(metadata
                .api
                .iter()
                .any(|entry| entry.prop == prop && entry.status == implemented));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "children"
                && entry.status == partial
                && entry.description.contains("field(..)")
        }));

        // Anatomy: one root, no composition parts.
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(metadata.parts.len(), 1);
        assert_eq!(metadata.parts[0].name, "Form");

        // States are backed by behaviour: the blocked-submit focus, the
        // read-only bar and the disabled omission are Implemented; the
        // Enter/default-submitter row is Partial and spells the exact GPUI
        // limitation instead of implying a browser rule.
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Blocked submit focuses first invalid"
                && entry.status == implemented
                && entry.rust.contains("first_invalid_focus")
        }));
        let enter = metadata
            .states
            .iter()
            .find(|entry| entry.state == "Enter / default submitter")
            .expect("the Enter row is the honest core of the form");
        assert_eq!(enter.status, partial);
        assert!(enter.description.contains("default submitter"));
        assert!(enter.description.contains("opaque"));
        assert!(enter.description.contains("GPUI substitute"));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Read-only bar"
                && entry.status == implemented
                && entry.selector.contains("readonly")
        }));
        assert!(metadata
            .states
            .iter()
            .any(|entry| entry.state == "Disabled omission" && entry.status == implemented));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Server errors displayed"
                && entry.status == implemented
                && entry.rust.contains("deliver_server_errors")
                && entry.description.contains("keyed to the record's identity")
        }));

        // Classless styling: no form.css, and the built-in stack is named.
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token.contains("no form.css") && entry.rust.contains("gap(px(16.))")
        }));

        assert_eq!(metadata.version, "3.2.4");
        assert!(metadata.api_source.contains("form.tsx"));
        assert!(metadata.api_source.contains("react-aria-components@1.20.0"));
        for url in [
            metadata.docs_source,
            metadata.api_source,
            metadata.style_source,
        ] {
            assert!(url.contains("/blob/v3.2.4/"));
        }
    }

    #[test]
    fn button_metadata_tracks_render_state_and_pinned_style_limits() {
        let metadata = reference_metadata::for_route(
            "Button",
            "use herogpui::prelude::{Button, Size, Variant};",
        )
        .expect("Button metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for prop in [
            "variant",
            "size",
            "fullWidth",
            "isDisabled",
            "isPending",
            "isIconOnly",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Button"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in [
            "isPending",
            "isPressed",
            "isHovered",
            "isFocused",
            "isFocusVisible",
            "isDisabled",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ButtonRenderProps"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".button--sm / .button--md / .button--lg"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".button transform / box-shadow transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "render"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
    }

    #[test]
    fn close_button_metadata_tracks_render_state_and_press_geometry() {
        let metadata = reference_metadata::for_route(
            "CloseButton",
            "use herogpui::components::close_button::CloseButton;",
        )
        .expect("CloseButton metadata is registered");

        for prop in ["isHovered", "isPressed", "isFocused", "isDisabled"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "CloseButtonRenderProps"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.parts.iter().any(|part| {
            part.name == "CloseButton.Icon"
                && part.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Pressed"
                && entry.rust.contains("root-bounds")
                && entry.description.contains("fixed child")
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".close-button--default:active / [data-pressed=\"true\"]"
                && entry.rust.contains("root-bounds")
                && entry.description.contains("fixed child")
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));

        let source = include_str!("reference_metadata.rs");
        assert!(!source.contains("CloseButton .active opacity 0.7"));
        assert!(!source.contains("close button dims on press"));
    }

    #[test]
    fn button_group_metadata_tracks_child_precedence_and_outline_collapse() {
        let metadata = reference_metadata::for_route(
            "ButtonGroup",
            "use herogpui::components::button_group::ButtonGroup;",
        )
        .expect("ButtonGroup metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for prop in ["variant", "size", "orientation", "fullWidth", "isDisabled"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ButtonGroup"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        // fullWidth is context-merged exactly like the other child defaults:
        // pinned button.tsx resolves `fullWidth ?? context.fullWidth` per
        // member, so the metadata must name the group_defaults fold, not a
        // bare group-wide flag.
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ButtonGroup"
                && entry.prop == "fullWidth"
                && entry.rust.contains("group_defaults")
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled"
                && entry.rust.contains("group_defaults")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.parts.iter().any(|part| {
            part.name == "ButtonGroup.Separator"
                && part.rust_owner == "ButtonGroup"
                && !part.rust_owner.contains("::")
                && part.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".button-group outline border handling"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.value.contains("border-e-0")
                && entry.value.contains("border-y-0")
                && entry.rust.contains("collapsed_border_sides")
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".button-group__separator"
                && entry.rust.contains("separator_variant")
        }));
        assert!(metadata.api_source.contains("button-group.test.tsx"));
    }

    #[test]
    fn input_group_metadata_tracks_field_propagation_and_pinned_hover() {
        let metadata = reference_metadata::for_route(
            "InputGroup",
            "use herogpui::components::input_group::InputGroup;",
        )
        .expect("InputGroup metadata is registered");

        // The exact pinned composition: Root, Input, TextArea, Prefix, Suffix.
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(metadata.parts.len(), 5);
        for name in [
            "InputGroup",
            "InputGroup.Input",
            "InputGroup.TextArea",
            "InputGroup.Prefix",
            "InputGroup.Suffix",
        ] {
            assert!(
                metadata
                    .parts
                    .iter()
                    .any(|part| part.name == name && part.rust_owner == "InputGroup"),
                "the pinned part {name} must be documented"
            );
        }

        // The group-level flags the port folds from the pinned TextField
        // context, and the width that lands on the outer wrapper.
        for prop in [
            "fullWidth",
            "variant",
            "isDisabled",
            "label",
            "errorMessage",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.prop == prop
                    && entry.rust_owner == "InputGroup"
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        // Disabled reaches the held field itself — the fixed defect — so the
        // mapping must name the propagation, not a dimming-only story.
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "isDisabled"
                && entry.rust.contains("Input::is_disabled")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));

        // Exactly the four states the pinned page claims, with hover drawn by
        // the group's own module evidence.
        assert_eq!(metadata.states.len(), 4);
        for state in ["Hover", "Focus Within", "Invalid", "Disabled"] {
            assert!(metadata.states.iter().any(|entry| entry.state == state));
        }
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Hover"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.selector.contains(":not(:focus-within)")
                && entry.rust.contains("border_hover()")
        }));
        // The pinned hover accessor: `--default-hover` is `RoleColor::hover()`,
        // and `soft_hover()` is a different, lighter token — the recorded
        // mapping must name the right one.
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Hover"
                && entry.rust.contains("default.hover()")
                && !entry.rust.contains("soft_hover")
        }));
        // A disabled group paints no hover: the recorded mapping must gate the
        // refinement on the disabled flag as well as the focus.
        assert!(metadata
            .states
            .iter()
            .any(|entry| { entry.state == "Hover" && entry.rust.contains("&& !is_disabled") }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled" && entry.rust.contains("Input::is_disabled")
        }));
        // The disabled story must stay honest in both directions: one dim over
        // the box, and the children limitation stated rather than implied.
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled"
                && entry
                    .description
                    .contains("one dim covers the whole group box")
                && entry.description.contains("pointer- or tab-inert")
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token.contains("data-disabled")
                && entry.description.contains("pointer-events: none")
        }));

        // Styling rows must carry the fixed behavior evidence: the hover fill,
        // the textarea group geometry, and the full-width stretch.
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token.contains(":hover:not(:focus-within)")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.value.contains("bg-field-hover")
                && entry.rust.contains("default.hover()")
                && !entry.rust.contains("soft_hover")
        }));
        // The textarea rows math is proven, but the pinned 38px one-row floor
        // is not drawn — the record must say so rather than claim a match.
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token.contains("input-group-textarea")
                && entry.value.contains("38px")
                && entry
                    .description
                    .contains("the pinned 38px one-row floor does not")
        }));
        // The documented `InputGroup.TextArea.variant`, shadowed by the
        // group's shared chrome.
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "variant"
                && entry.owner == "InputGroup.TextArea"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token.contains(":has(")
                && entry.value.contains("items-start")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.rust.contains("items_start")
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".input-group--full-width"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.rust.contains("w_full")
        }));

        // v3.2.4 pins on every source.
        assert_eq!(metadata.version, "3.2.4");
        assert!(metadata.api_source.contains("input-group.tsx"));
        assert!(metadata.style_source.contains("input-group.css"));
        for url in [
            metadata.docs_source,
            metadata.api_source,
            metadata.style_source,
        ] {
            assert!(url.contains("/blob/v3.2.4/"));
        }
    }

    #[test]
    fn referenced_types_include_companions_used_by_examples() {
        let examples = ["h::Table::new().column(h::TableColumn::new())"];
        let owners = referenced_types(
            "use herogpui::components::table::Table;",
            examples.into_iter(),
        );

        assert!(owners.contains("Table"));
        assert!(owners.contains("TableColumn"));
    }

    #[test]
    fn methods_after_multiline_signature_are_included() {
        let owners = BTreeSet::from(["Form".to_owned()]);
        let methods = methods_for(source_for_module("form").unwrap(), &owners);

        assert!(methods.iter().any(|method| method.name == "on_submit"));
    }

    #[test]
    fn braces_in_docs_do_not_change_impl_scope() {
        let source = r#"
impl Widget {
    /// Uses the {Widget syntax.
    pub fn first(mut self) -> Self {
        self
    }

    pub fn second(mut self) -> Self {
        self
    }
}
"#;
        let owners = BTreeSet::from(["Widget".to_owned()]);
        let methods = methods_for(source, &owners);

        assert!(methods.iter().any(|method| method.name == "second"));
    }

    #[test]
    fn every_component_import_resolves_to_public_api() {
        for section in crate::pages::nav_sections() {
            for page in section.items {
                let import = page.import_line();
                if import.is_empty() {
                    continue;
                }
                assert!(
                    reference_metadata::for_import(import).is_some(),
                    "no reference metadata for {page:?}: {import}"
                );
                let source = source_for(import).unwrap_or_else(|| panic!("no source for {page:?}"));
                let owners = referenced_types(import, std::iter::empty());
                assert!(
                    !methods_for(source, &owners).is_empty(),
                    "no API methods for {page:?}"
                );
            }
        }
    }

    #[test]
    fn table_parser_reaches_methods_after_braces_in_docs() {
        let owners = BTreeSet::from(["Table".to_owned()]);
        let methods = methods_for(source_for_module("table").unwrap(), &owners);

        assert!(methods.iter().any(|method| method.name == "row_height"));
    }

    #[test]
    fn state_change_callbacks_are_not_styling_builders() {
        assert!(!is_styling_method("on_open_change"));
    }

    #[test]
    fn dropdown_metadata_keeps_compound_part_ownership() {
        let metadata = reference_metadata::for_import(
            "use herogpui::components::dropdown::{Dropdown, MenuItem};",
        )
        .expect("Dropdown metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Dropdown part disappeared: {required}"
            );
        }
        assert_eq!(
            metadata
                .parts
                .iter()
                .find(|part| part.name == "Dropdown.Menu")
                .expect("Dropdown.Menu is registered")
                .rust_owner,
            "Menu"
        );
        assert_eq!(
            metadata
                .parts
                .iter()
                .find(|part| part.name == "Dropdown.Item")
                .expect("Dropdown.Item is registered")
                .rust_owner,
            "MenuItem"
        );
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Dropdown.Item"
                && entry.prop == "children"
                && entry.rust_owner == "Menu"
                && entry.rust == "item_content(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Dropdown.ItemIndicator"
                && entry.prop == "children"
                && entry.rust_owner == "Menu"
                && entry.rust == "indicator_content(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn input_metadata_tracks_native_limits_and_desktop_substitutions() {
        let metadata = reference_metadata::for_route(
            "Input",
            "use herogpui::components::input::{Input, InputState};",
        )
        .expect("Input metadata is registered");

        assert_eq!(metadata.parts.len(), 1);
        for prop in [
            "value",
            "defaultValue",
            "onChange",
            "maxLength",
            "minLength",
            "fullWidth",
            "variant",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "type"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "autoComplete"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".input transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn text_area_metadata_tracks_multiline_limits_and_wrap_substitution() {
        let metadata = reference_metadata::for_route(
            "TextArea",
            "use herogpui::components::textarea::TextArea;",
        )
        .expect("Text Area metadata is registered");

        assert_eq!(metadata.parts.len(), 1);
        for prop in [
            "rows",
            "value",
            "defaultValue",
            "onChange",
            "maxLength",
            "minLength",
            "fullWidth",
            "variant",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "cols"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.prop == "wrap"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".textarea transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn text_field_metadata_tracks_complete_render_state_and_composition() {
        let metadata = reference_metadata::for_route(
            "TextField",
            "use herogpui::components::input::TextField;",
        )
        .expect("TextField metadata is registered");

        assert_eq!(metadata.parts.len(), 6);
        for prop in [
            "children",
            "defaultValue",
            "onChange",
            "isDisabled",
            "isReadOnly",
            "isRequired",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "TextField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "TextField"
                && entry.prop == "value"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".textfield .description"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn search_field_metadata_tracks_complete_render_state_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "SearchField",
            "use herogpui::components::input::SearchField;",
        )
        .expect("SearchField metadata is registered");

        assert_eq!(metadata.parts.len(), 5);
        for prop in [
            "children",
            "defaultValue",
            "onChange",
            "onSubmit",
            "onClear",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "SearchField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "SearchField"
                && entry.prop == "value"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "SearchField.ClearButton"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".search-field__group transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn date_picker_metadata_tracks_inherited_behavior_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "DatePicker",
            "use herogpui::components::date_picker::DatePicker;",
        )
        .expect("DatePicker metadata is registered");

        assert_eq!(metadata.parts.len(), 4);
        for prop in [
            "children",
            "defaultValue",
            "defaultOpen",
            "isRequired",
            "autoFocus",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "DatePicker"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "DatePicker.TriggerIndicator"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".date-picker__popover"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn date_range_picker_metadata_tracks_inherited_behavior_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "DateRangePicker",
            "use herogpui::components::date_picker::{DateRangePicker, DateRangeState};",
        )
        .expect("DateRangePicker metadata is registered");

        assert_eq!(metadata.parts.len(), 5);
        for prop in [
            "children",
            "defaultValue",
            "defaultOpen",
            "isRequired",
            "autoFocus",
            "startName",
            "endName",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "DateRangePicker"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for owner in [
            "DateRangePicker.TriggerIndicator",
            "DateRangePicker.RangeSeparator",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == "children"
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".date-range-picker__popover"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn list_box_metadata_tracks_inherited_selection_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "ListBox",
            "use herogpui::components::list_box::{ListBox, ListBoxItem};",
        )
        .expect("ListBox metadata is registered");

        assert_eq!(metadata.parts.len(), 4);
        for prop in [
            "selectionMode",
            "selectedKeys",
            "defaultSelectedKeys",
            "disallowEmptySelection",
            "shouldFocusWrap",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ListBox"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ListBox.ItemIndicator"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".list-box-item"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn tag_group_metadata_tracks_inherited_selection_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "TagGroup",
            "use herogpui::components::tag_group::{Tag, TagGroup};",
        )
        .expect("TagGroup metadata is registered");

        assert_eq!(metadata.parts.len(), 4);
        for prop in [
            "selectionMode",
            "selectedKeys",
            "defaultSelectedKeys",
            "disallowEmptySelection",
            "onRemove",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "TagGroup"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Tag"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".tag--surface"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn color_picker_metadata_tracks_compound_parts_and_internal_open_state() {
        let metadata = reference_metadata::for_route(
            "ColorPicker",
            "use herogpui::components::color_picker::ColorPicker;",
        )
        .expect("ColorPicker metadata is registered");

        assert_eq!(metadata.parts.len(), 3);
        for prop in ["value", "defaultValue", "onChange"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ColorPicker"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorPicker.Popover"
                && entry.prop == "placement"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Open"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".color-picker__popover"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn color_area_metadata_tracks_thumb_state_and_pinned_motion() {
        let metadata = reference_metadata::for_route(
            "ColorArea",
            "use herogpui::components::color_picker::ColorArea;",
        )
        .expect("ColorArea metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorArea.Thumb"
                && entry.prop == "children"
                && entry.rust == "thumb(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for state in ["Dragging", "Hovered", "Focused", "Focus visible"] {
            assert!(metadata.states.iter().any(|entry| {
                entry.state == state
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".color-area__thumb transition"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata
            .api_source
            .contains("react-aria-components/src/ColorThumb.tsx"));
    }

    #[test]
    fn color_slider_metadata_tracks_compound_parts_and_thumb_state() {
        let metadata = reference_metadata::for_route(
            "ColorSlider",
            "use herogpui::components::color_picker::{ColorChannel, ColorSlider};",
        )
        .expect("ColorSlider metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorSlider.Thumb"
                && entry.prop == "children"
                && entry.rust == "thumb(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for state in ["Dragging", "Hovered", "Focused", "Focus visible"] {
            assert!(metadata.states.iter().any(|entry| {
                entry.state == state
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".color-slider__track horizontal / vertical"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata
            .api_source
            .contains("react-aria-components/src/ColorThumb.tsx"));
    }

    #[test]
    fn color_swatch_picker_metadata_tracks_item_state_and_size_geometry() {
        let metadata = reference_metadata::for_route(
            "ColorSwatchPicker",
            "use herogpui::components::color_picker::ColorSwatchPicker;",
        )
        .expect("ColorSwatchPicker metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorSwatchPicker.Item"
                && entry.prop == "children"
                && entry.rust == "item_content(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorSwatchPicker.Indicator"
                && entry.rust == "indicator(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for state in [
            "Hovered",
            "Pressed",
            "Selected",
            "Focused",
            "Focus visible",
            "Disabled",
        ] {
            assert!(metadata.states.iter().any(|entry| {
                entry.state == state
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".color-swatch-picker__item sizes"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata
            .api_source
            .contains("react-aria-components/src/ColorSwatchPicker.tsx"));
    }

    #[test]
    fn toast_metadata_tracks_queue_lifecycle_and_frontmost_interaction() {
        let metadata = reference_metadata::for_route(
            "Toast",
            "use herogpui::components::toast::{Toast, ToastViewport};",
        )
        .expect("Toast metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for prop in [
            "add",
            "close",
            "pauseAll",
            "resumeAll",
            "clear",
            "subscribe",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ToastQueue"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Frontmost"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".toast__title"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata
            .api_source
            .contains("react-stately/src/toast/useToastState.ts"));
    }

    #[test]
    fn color_field_metadata_tracks_complete_render_state_and_compound_parts() {
        let metadata = reference_metadata::for_route(
            "ColorField",
            "use herogpui::components::color_picker::ColorField;",
        )
        .expect("ColorField metadata is registered");

        assert_eq!(metadata.parts.len(), 8);
        for prop in [
            "children",
            "onChange",
            "validate",
            "isWheelDisabled",
            "name",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ColorField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["value", "defaultValue", "isRequired"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "ColorField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ColorField.Suffix"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for state in ["Invalid", "Disabled", "Read only", "Focus within"] {
            assert!(metadata.states.iter().any(|entry| {
                entry.state == state
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".color-input-group__suffix"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn dropdown_metadata_does_not_classify_behavior_as_styling() {
        let metadata = reference_metadata::for_import(
            "use herogpui::components::dropdown::{Dropdown, MenuItem};",
        )
        .expect("Dropdown metadata is registered");

        assert!(metadata
            .api
            .iter()
            .any(|entry| entry.prop == "onOpenChange"));
        assert!(metadata
            .styling
            .iter()
            .all(|entry| !entry.class_or_token.contains("onOpenChange")));
        assert!(metadata
            .styling
            .iter()
            .all(|entry| !entry.class_or_token.contains("onSelectionChange")));
    }

    #[test]
    fn dropdown_metadata_records_contextual_css_overrides_as_implemented() {
        let metadata = reference_metadata::for_import(
            "use herogpui::components::dropdown::{Dropdown, MenuItem};",
        )
        .expect("Dropdown metadata is registered");

        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".dropdown__popover [data-slot=\"dropdown-menu\"]"
                && entry.value == "p-1.5"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".dropdown__popover [data-slot=\"menu-item\"]"
                && entry.value == "px-2.5"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata
            .styling
            .iter()
            .filter(|entry| { entry.class_or_token == ".dropdown__trigger" })
            .all(|entry| entry.status == reference_metadata::ImplementationStatus::Partial));
    }

    #[test]
    fn registered_metadata_has_inputs_and_resolvable_rust_owners() {
        for metadata in reference_metadata::ALL {
            assert!(
                !metadata.api.is_empty(),
                "{} has no API metadata",
                metadata.page
            );
            assert!(!metadata.parts.is_empty(), "{} has no parts", metadata.page);
            assert!(
                !metadata.styling.is_empty(),
                "{} has no styling",
                metadata.page
            );

            let source = source_for_module(metadata.source_module)
                .expect("metadata source module is embedded");
            let owners = metadata
                .parts
                .iter()
                .map(|part| part.rust_owner)
                .chain(metadata.api.iter().map(|entry| entry.rust_owner))
                .map(str::to_owned)
                .collect();
            let methods = methods_for(source, &owners);
            for owner in metadata
                .parts
                .iter()
                .map(|part| part.rust_owner)
                .chain(metadata.api.iter().map(|entry| entry.rust_owner))
            {
                assert!(
                    source.contains(&format!("pub struct {owner}"))
                        || source.contains(&format!("pub enum {owner}"))
                        || source.contains(&format!("impl {owner}")),
                    "{} has unresolved Rust owner {owner}",
                    metadata.page
                );
            }
            for entry in metadata.api.iter().filter(|entry| {
                entry.status != reference_metadata::ImplementationStatus::Unavailable
            }) {
                if mapping_method_name(entry.rust).is_some() {
                    assert!(
                        mapping_matches_method(&methods, entry.rust_owner, entry.rust),
                        "{}::{} has unresolved Rust mapping {}",
                        entry.owner,
                        entry.prop,
                        entry.rust
                    );
                } else {
                    assert_eq!(
                        entry.status,
                        reference_metadata::ImplementationStatus::Partial,
                        "{}::{} needs a method mapping or Partial status",
                        entry.owner,
                        entry.prop
                    );
                }
            }
        }
    }

    #[test]
    fn tabs_metadata_keeps_parts_and_remaining_style_gaps_explicit() {
        let metadata = reference_metadata::for_route(
            "Tabs",
            "use herogpui::components::tabs::{TabItem, Tabs};",
        )
        .expect("Tabs metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Tabs part disappeared: {required}"
            );
        }
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Hovered"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".tabs__tab transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".tabs__list-container__scroller"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".tabs__panel[data-exiting=\"true\"]"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
    }

    #[test]
    fn separator_metadata_keeps_declared_surface_honest() {
        let metadata = reference_metadata::for_route(
            "Separator",
            "use herogpui::components::separator::Separator;",
        )
        .expect("Separator metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for prop in ["orientation", "variant"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Separator"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["className", "render"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Unavailable
            }));
        }
        assert!(metadata.states.is_empty());
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".separator--vertical"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn toolbar_metadata_tracks_focus_on_children_and_attached_chrome() {
        let metadata =
            reference_metadata::for_route("Toolbar", "use herogpui::components::toolbar::Toolbar;")
                .expect("Toolbar metadata is registered");

        // The root is the only declared part.
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(metadata.parts.len(), 1);

        // The exact v3 table: two props this port implements, the browser
        // naming and classes it cannot, and the render-prop orientation the
        // port computes instead of handing over.
        for prop in ["isAttached", "orientation"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Toolbar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["aria-label", "aria-labelledby", "className"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Toolbar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Unavailable
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Toolbar"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "ToolbarRenderProps"
                && entry.prop == "orientation"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));

        // The keyboard contract lives on the children: they hold the focus
        // and draw the ring, disabled children are skipped, the pinned
        // `useToolbar` lastFocused record survives in keyed state (and a
        // removed child's handle is never restored), and a nested toolbar
        // acts as a group that binds no management of its own.
        for state in ["Focused child", "Disabled child", "Last focused child"] {
            assert!(metadata.states.iter().any(|entry| {
                entry.state == state
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Last focused child" && entry.rust.contains("ToolbarFocusEdge")
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Nested toolbar as group"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.rust.contains("ToolbarScopes")
                && entry.rust.contains("FocusHandle::contains")
        }));

        // The pinned attached chrome: the surface fill and 8px rhythm on the
        // base rule, rounding plus overlay shadow with no border on the
        // attached modifier, and the vertical sheet's start-edge alignment —
        // with only the nested button-group re-justification honestly
        // partial, because gpui cannot restyle an opaque child.
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".toolbar"
                && entry.value.contains("gap-2")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".toolbar--attached"
                && entry.value.contains("bg-surface")
                && entry.value.contains("shadow-overlay")
                && entry.value.contains("no border")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".toolbar--vertical"
                && entry.value.contains("items-start")
                && entry.value.contains("justify-start")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".toolbar--vertical .button-group"
                && entry.value == "justify-start"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api_source.contains("useToolbar.ts"));
    }

    #[test]
    fn calendar_metadata_keeps_defaults_parts_and_style_gaps_explicit() {
        let metadata = reference_metadata::for_route(
            "Calendar",
            "use herogpui::components::calendar::{Calendar, CalendarState};",
        )
        .expect("Calendar metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Calendar part disappeared: {required}"
            );
        }
        for prop in ["focusedValue", "minValue", "maxValue"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Calendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Calendar"
                && entry.prop == "firstDayOfWeek"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.description.contains("regional date preferences")
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Calendar.YearPickerTriggerHeading"
                && entry.prop == "offset"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Calendar.YearPickerGrid"
                && entry.prop == "visibleYears"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Outside month"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Today"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token
                == ".calendar:has(.calendar-year-picker__year-grid) > [data-slot=\"calendar-grid\"]"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".calendar__cell-indicator"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn date_field_metadata_keeps_compound_anatomy_and_limits_explicit() {
        let metadata = reference_metadata::for_route(
            "DateField",
            "use herogpui::components::date_picker::DateField;",
        )
        .expect("DateField metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered DateField part disappeared: {required}"
            );
        }
        for prop in [
            "fullWidth",
            "isRequired",
            "isInvalid",
            "validationBehavior",
            "granularity",
            "isDisabled",
            "isReadOnly",
            "name",
            "autoFocus",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "DateField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "DateField.InputContainer"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "DateFieldRenderProps"
                && entry.prop == "isDisabled / isInvalid / isReadOnly / isRequired"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == "unsupported trailing steppers"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".date-input-group transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn time_field_metadata_keeps_render_state_and_style_limits_explicit() {
        let metadata = reference_metadata::for_route(
            "TimeField",
            "use herogpui::components::time_field::{TimeField, TimeState};",
        )
        .expect("TimeField metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered TimeField part disappeared: {required}"
            );
        }
        for prop in [
            "fullWidth",
            "isRequired",
            "isInvalid",
            "validationBehavior",
            "granularity",
            "isDisabled",
            "isReadOnly",
            "name",
            "autoFocus",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "TimeField"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in [
            "isDisabled / isInvalid / isReadOnly / isRequired",
            "isFocused / isFocusWithin / isFocusVisible",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "TimeFieldRenderProps"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".date-input-group transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn number_field_metadata_tracks_compound_buttons_render_state_and_style_limits() {
        let metadata = reference_metadata::for_route(
            "NumberField",
            "use herogpui::components::number_field::{NumberField, NumberState};",
        )
        .expect("NumberField metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered NumberField part disappeared: {required}"
            );
        }
        for (owner, prop) in [
            ("NumberField", "defaultValue"),
            ("NumberField", "minValue"),
            ("NumberField", "maxValue"),
            ("NumberField", "step"),
            ("NumberField", "validationBehavior"),
            ("NumberField.IncrementButton", "children"),
            ("NumberField.DecrementButton", "children"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in [
            "isDisabled / isInvalid / isReadOnly / isRequired",
            "isFocused / isFocusWithin / isFocusVisible",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "NumberFieldRenderProps"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == "pinned With Chevrons composition"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for class_or_token in [
            ".number-field__group transitions",
            ".number-field buttons:active",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
    }

    #[test]
    fn range_calendar_metadata_keeps_range_state_and_style_gaps_explicit() {
        let metadata = reference_metadata::for_route(
            "RangeCalendar",
            "use herogpui::components::range_calendar::RangeCalendar;",
        )
        .expect("RangeCalendar metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered RangeCalendar part disappeared: {required}"
            );
        }
        for prop in [
            "onChange",
            "focusedValue",
            "minValue",
            "maxValue",
            "isDateUnavailable",
            "selectionAlignment",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "RangeCalendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "RangeCalendar"
                && entry.prop == "isDateUnavailable"
                && entry.description.contains("one visible duration")
                && entry.description.contains("sentinel day")
                && entry.description.contains("cells, focus and navigation")
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "RangeCalendar"
                && entry.prop == "firstDayOfWeek"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
                && entry.description.contains("regional date preferences")
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "RangeCalendar.YearPickerTriggerHeading"
                && entry.prop == "offset"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "RangeCalendar.YearPickerGrid"
                && entry.prop == "visibleYears"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "RangeCalendar.YearPickerGrid"
                && entry.slot == "calendar-year-picker-grid"
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Outside month"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Pressed"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Range middle"
                && entry.selector == "[data-selection-in-range=\"true\"]"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".range-calendar__nav-button"
                && entry.rust.contains("small_radius")
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token
                == ".range-calendar:has(.calendar-year-picker__year-grid) > [data-slot=\"range-calendar-grid\"]"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
    }

    #[test]
    fn drawer_metadata_keeps_drag_contract_and_style_gaps_explicit() {
        let metadata = reference_metadata::for_route(
            "Drawer",
            "use herogpui::components::drawer::{Drawer, DrawerCloseTrigger, DrawerPlacement};",
        )
        .expect("Drawer metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Drawer part disappeared: {required}"
            );
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Drawer.Backdrop"
                && entry.prop == "isDismissable"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        for class_or_token in [
            ".drawer__backdrop",
            ".drawer__body",
            ".drawer__handle / [data-slot=\"drawer-handle-bar\"]",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == "useDrawerDrag contract"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn modal_metadata_tracks_compound_parts_and_full_size_parity() {
        let metadata = reference_metadata::for_route(
            "Modal",
            "use herogpui::components::modal::{Modal, ModalCloseTrigger, ModalSize};",
        )
        .expect("Modal metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Modal part disappeared: {required}"
            );
        }
        for (owner, prop) in [
            ("Modal.Backdrop", "isDismissable"),
            ("Modal.Container", "size"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        // v3.2.4 parity: the composed close trigger accepts custom children while
        // staying wired to the modal's dismissal paths, so it is Implemented.
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Modal.CloseTrigger"
                && entry.prop == "children"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        for class_or_token in [
            ".modal__container--full",
            ".modal__dialog--full",
            ".modal__container--full[data-entering=\"true\"] / [data-exiting=\"true\"]",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token
                == ".modal__container--scroll-outside / .modal__backdrop:has(.modal__container--scroll-outside)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn tooltip_metadata_tracks_parts_states_and_visual_limits() {
        let metadata =
            reference_metadata::for_route("Tooltip", "use herogpui::components::tooltip::Tooltip;")
                .expect("Tooltip metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Tooltip part disappeared: {required}"
            );
        }
        for prop in [
            "delay",
            "closeDelay",
            "trigger",
            "isDisabled",
            "shouldSkipAnimation",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Tooltip"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for class_or_token in [
            ".tooltip max-w-xs",
            "--tooltip-delay",
            "--tooltip-close-delay",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for class_or_token in [
            ".tooltip break-all",
            ".tooltip [data-slot=\"overlay-arrow\"]",
            ".tooltip__trigger",
            ".tooltip__trigger focus-visible",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
    }

    #[test]
    fn popover_metadata_tracks_compound_anatomy_and_true_flipping() {
        let metadata = reference_metadata::for_route(
            "Popover",
            "use herogpui::components::popover::{Popover, PopoverArrow, PopoverPlacement};",
        )
        .expect("Popover metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Popover part disappeared: {required}"
            );
        }
        for prop in ["isOpen", "defaultOpen", "onOpenChange"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Popover"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["offset", "shouldFlip"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Popover.Content"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.parts.iter().any(|part| {
            part.name == "Popover.Arrow"
                && part.status == reference_metadata::ImplementationStatus::Partial
        }));
        // The arrow matches size, curve, fill and flip-aware rotation, but a
        // custom child cannot inherit the placement rotation because GPUI 0.2.2
        // transforms only svg elements; the row stays Partial by design.
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".popover [data-slot=popover-overlay-arrow]"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn select_metadata_tracks_compound_ownership_and_visual_limits() {
        let metadata =
            reference_metadata::for_route("Select", "use herogpui::components::select::Select;")
                .expect("Select metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Select part disappeared: {required}"
            );
        }
        for prop in [
            "placeholder",
            "selectionMode",
            "isOpen",
            "defaultOpen",
            "onOpenChange",
            "disabledKeys",
            "isInvalid",
            "isRequired",
            "name",
            "variant",
            "fullWidth",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Select"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["value", "defaultValue", "onChange"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Select"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for (owner, prop) in [
            ("Select.Indicator", "children"),
            ("Select.Popover", "placement"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        for class_or_token in [
            ".select__value",
            ".select__indicator",
            ".select__popover[data-entering]",
        ] {
            assert!(metadata.styling.iter().any(|entry| {
                entry.class_or_token == class_or_token
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".select--full-width / .select__trigger--full-width"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".select__popover[data-exiting]"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".select__popover .list-box / .list-box-item"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn autocomplete_metadata_tracks_filter_control_indicator_and_motion() {
        let metadata = reference_metadata::for_route(
            "Autocomplete",
            "use herogpui::components::autocomplete::Autocomplete;",
        )
        .expect("Autocomplete metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Autocomplete part disappeared: {required}"
            );
        }
        for (owner, prop) in [
            ("Autocomplete", "selectionMode"),
            ("Autocomplete", "onChange"),
            ("Autocomplete", "isOpen"),
            ("Autocomplete", "isRequired"),
            ("Autocomplete.Indicator", "children"),
            ("Autocomplete.Filter", "inputValue"),
            ("Autocomplete.Filter", "onInputChange"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".autocomplete__trigger transitions"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".autocomplete__popover[data-exiting=\"true\"]"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
    }

    #[test]
    fn table_metadata_tracks_compound_parts_and_honest_gaps() {
        let metadata =
            reference_metadata::for_route("Table", "use herogpui::components::table::Table;")
                .expect("Table metadata is registered");

        assert_eq!(metadata.required_parts.len(), 15);
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for required in metadata.required_parts {
            assert!(
                metadata.parts.iter().any(|part| part.name == *required),
                "registered Table part disappeared: {required}"
            );
        }
        assert_eq!(
            metadata
                .parts
                .iter()
                .find(|part| part.name == "Table.Column")
                .expect("Table.Column is registered")
                .rust_owner,
            "TableColumn"
        );
        assert_eq!(
            metadata
                .parts
                .iter()
                .find(|part| part.name == "Table.Row")
                .expect("Table.Row is registered")
                .rust_owner,
            "TableRow"
        );
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "Table.Cell"
                && entry.rust_owner == "TableRow"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "Table.ColumnResizer"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "Table.ResizableContainer"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "Table.Collection"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.parts.iter().any(|entry| {
            entry.name == "Table.LoadMoreContent"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        for prop in [
            "variant",
            "selectionMode",
            "onSelectionChange",
            "sortDescriptor",
            "onSortChange",
            "disabledKeys",
            "expandedKeys",
            "onExpandedChange",
            "allowsSorting",
            "minWidth",
            "maxWidth",
            "allowsResizing",
            "textValue",
            "sortDirection",
            "showIndicator",
            "isLoading",
            "onLoadMore",
            "scrollOffset",
            "rowHeight",
            "estimatedRowHeight",
            "loaderHeight",
            "gap",
            "padding",
        ] {
            assert!(
                metadata.api.iter().any(|entry| {
                    entry.prop == prop
                        && entry.status == reference_metadata::ImplementationStatus::Implemented
                }),
                "Table metadata lost Implemented {prop}"
            );
        }
        for (owner, prop) in [
            ("Table.Content", "selectedKeys"),
            ("Table.Content", "onRowAction"),
            ("Table.Content", "treeColumn"),
            ("Table.Column", "id"),
            ("Table.Column", "isRowHeader"),
            ("Table.Column", "width"),
            ("Table.Column", "defaultWidth"),
            ("Table.ResizableContainer", "onResizeStart"),
            ("Table.ResizableContainer", "onResize"),
            ("Table.ResizableContainer", "onResizeEnd"),
            ("Table.Row", "id"),
            ("Table.Row", "isDisabled"),
            ("Table.LoadMore", "children"),
            ("Table.Collection", "items"),
        ] {
            assert!(
                metadata.api.iter().any(|entry| {
                    entry.owner == owner
                        && entry.prop == prop
                        && entry.status == reference_metadata::ImplementationStatus::Partial
                }),
                "{owner}::{prop} should stay Partial"
            );
        }
        for (owner, prop) in [
            ("Table.Content", "aria-label"),
            ("Table.Content", "defaultSelectedKeys"),
            ("Table.Content", "selectionBehavior"),
            ("Table.Content", "dragAndDropHooks"),
            ("Table.Content", "keyboardNavigationBehavior"),
            ("Table.Cell", "colSpan"),
            ("TableLayout", "headingHeight"),
            ("TableLayout", "dropIndicatorThickness"),
        ] {
            assert!(
                metadata.api.iter().any(|entry| {
                    entry.owner == owner
                        && entry.prop == prop
                        && entry.status == reference_metadata::ImplementationStatus::Unavailable
                }),
                "{owner}::{prop} should stay Unavailable"
            );
        }
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled row"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Disabled table"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Dragging"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Selected"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".table-root--primary"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".table__column"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".table__cell[data-tree-column]"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".table__column::after"
                && entry.status == reference_metadata::ImplementationStatus::Unavailable
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".table__footer"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.docs_source.contains("(data-display)/table.mdx"));
        assert!(metadata
            .api_source
            .contains("react-aria-components@1.20.0/packages/react-aria-components/src/Table.tsx"));
        assert!(metadata
            .style_source
            .contains("packages/styles/components/table.css"));
    }

    #[test]
    fn accordion_metadata_tracks_item_ownership_custom_indicator_and_style_limits() {
        let metadata = reference_metadata::for_route(
            "Accordion",
            "use herogpui::components::accordion::{Accordion, AccordionItem};",
        )
        .expect("Accordion metadata is registered");

        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        for prop in ["isDisabled", "defaultExpanded", "onExpandedChange"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Accordion.Item"
                    && entry.prop == prop
                    && entry.rust_owner == "AccordionItem"
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        assert!(metadata.api.iter().any(|entry| {
            entry.owner == "Accordion.Indicator"
                && entry.prop == "children"
                && entry.rust == "indicator(render)"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.states.iter().any(|entry| {
            entry.state == "Hovered"
                && entry.description.contains("closed")
                && entry.status == reference_metadata::ImplementationStatus::Implemented
        }));
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".accordion__panel"
                && entry.status == reference_metadata::ImplementationStatus::Partial
        }));
    }

    #[test]
    fn disclosure_metadata_tracks_live_render_state() {
        let metadata = reference_metadata::for_route(
            "Disclosure",
            "use herogpui::components::disclosure::{Disclosure, DisclosureGroup};",
        )
        .expect("Disclosure route is registered");
        for prop in ["isExpanded", "isDisabled"] {
            let row = metadata
                .api
                .iter()
                .find(|row| row.owner == "DisclosureRenderProps" && row.prop == prop)
                .unwrap_or_else(|| panic!("DisclosureRenderProps.{prop} row"));
            assert_eq!(
                row.status,
                reference_metadata::ImplementationStatus::Implemented
            );
            assert_eq!(row.rust, "content(render)");
        }
        let children = metadata
            .api
            .iter()
            .find(|row| row.owner == "Disclosure" && row.prop == "children")
            .expect("Disclosure.children row");
        assert_eq!(
            children.status,
            reference_metadata::ImplementationStatus::Partial,
            "the render values are live, but the GPUI control still owns the compound trigger"
        );
    }

    #[test]
    fn metadata_validation_rejects_bogus_method_owner_and_page() {
        let metadata = reference_metadata::for_route(
            "Dropdown",
            "use herogpui::components::dropdown::{Dropdown, MenuItem};",
        )
        .expect("Dropdown route is registered");
        let source = source_for_module(metadata.source_module).expect("Dropdown source exists");
        let owners = BTreeSet::from(["Menu".to_owned()]);
        let methods = methods_for(source, &owners);

        assert!(!mapping_matches_method(&methods, "Menu", "made_up()"));
        assert!(!mapping_matches_method(
            &methods,
            "NotMenu",
            "selection_mode(SelectionMode)"
        ));
        assert!(reference_metadata::for_route("NotADropdown", metadata.import_line).is_none());
        assert!(reference_metadata::for_route("Dropdown", "use wrong::import;").is_none());
    }

    #[test]
    fn inherent_child_builder_is_not_duplicated() {
        let owners = BTreeSet::from(["ToggleButton".to_owned()]);
        let methods = methods_for(source_for_module("toggle_button").unwrap(), &owners);

        assert_eq!(
            methods
                .iter()
                .filter(|method| method.name == "child")
                .count(),
            1
        );
    }

    #[test]
    fn alert_metadata_tracks_status_pinning_and_no_close_seam() {
        let metadata =
            reference_metadata::for_route("Alert", "use herogpui::components::alert::Alert;")
                .expect("Alert route is registered");
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(
            metadata.states.len(),
            0,
            "the alert has no interactive states"
        );

        let status = metadata
            .api
            .iter()
            .find(|row| row.owner == "Alert" && row.prop == "status")
            .expect("status row");
        assert_eq!(
            status.status,
            reference_metadata::ImplementationStatus::Implemented
        );
        assert!(
            status.default.contains("\"default\""),
            "the pinned status default is \"default\""
        );
        assert!(
            metadata
                .api
                .iter()
                .all(|row| !row.description.contains("isClosable") || row.prop == "children"),
            "no row may reintroduce a closable API"
        );
    }

    #[test]
    fn link_metadata_tracks_render_state_and_dom_only_props() {
        let metadata =
            reference_metadata::for_route("Link", "use herogpui::components::link::Link;")
                .expect("Link route is registered");
        let row = |prop: &str, owner: &str| {
            metadata
                .api
                .iter()
                .find(|row| row.owner == owner && row.prop == prop)
                .unwrap_or_else(|| panic!("{owner}.{prop} row"))
        };
        assert_eq!(
            row("render", "Link").status,
            reference_metadata::ImplementationStatus::Partial
        );
        assert!(
            row("render", "Link").description.contains("isFocusVisible"),
            "the render closure is handed the interactive state"
        );
        for dom_only in ["target", "rel", "download"] {
            assert_eq!(
                row(dom_only, "Link").status,
                reference_metadata::ImplementationStatus::Unavailable,
                "{dom_only} has no meaning without a browser navigation"
            );
        }
        assert_eq!(
            row("isDisabled", "Link").status,
            reference_metadata::ImplementationStatus::Implemented
        );
        assert_eq!(
            metadata
                .states
                .iter()
                .filter(
                    |state| state.status == reference_metadata::ImplementationStatus::Implemented
                )
                .count(),
            4,
            "hover, press, focus-visible and disabled are all wired"
        );
    }

    /// v3's childless `<Link.Icon />` renders a built-in arrow under
    /// `data-default-icon="true"` (with its `ms-1 pb-1.5` spacing); the port's
    /// `icon(element)` is the custom-children path only. That omission is
    /// honest metadata, and this test keeps it from being edited away.
    #[test]
    fn link_metadata_records_the_childless_builtin_arrow_omission() {
        let metadata =
            reference_metadata::for_route("Link", "use herogpui::components::link::Link;")
                .expect("Link route is registered");
        let children = metadata
            .api
            .iter()
            .find(|row| row.owner == "Link.Icon" && row.prop == "children")
            .expect("Link.Icon children row");
        assert_eq!(
            children.status,
            reference_metadata::ImplementationStatus::Partial,
            "the custom-children path exists, so the row is not Unavailable"
        );
        assert_eq!(children.rust, "icon(element)");
        for marker in ["built-in arrow", "data-default-icon", "icon(element)"] {
            assert!(
                children.description.contains(marker),
                "the Link.Icon children row must keep recording that the \
                 childless built-in arrow has no port path (missing: {marker})"
            );
        }
        let part = metadata
            .parts
            .iter()
            .find(|part| part.name == "Link.Icon")
            .expect("Link.Icon part row");
        assert_eq!(
            part.status,
            reference_metadata::ImplementationStatus::Partial
        );
        assert!(
            part.description.contains("data-default-icon")
                && part.description.contains("ms-1 pb-1.5"),
            "the Link.Icon part row must keep the default-icon spacing scoped \
             to the childless built-in arrow"
        );
    }

    #[test]
    fn avatar_metadata_tracks_load_events_and_fallback_color() {
        let metadata =
            reference_metadata::for_route("Avatar", "use herogpui::components::avatar::Avatar;")
                .expect("Avatar route is registered");
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(
            metadata.states.len(),
            0,
            "the avatar has no interactive states"
        );

        let row = |prop: &str, owner: &str| {
            metadata
                .api
                .iter()
                .find(|row| row.owner == owner && row.prop == prop)
                .unwrap_or_else(|| panic!("{owner}.{prop} row"))
        };
        let on_load = row("onLoad", "Avatar.Image");
        assert_ne!(
            on_load.status,
            reference_metadata::ImplementationStatus::Unavailable,
            "on_load is implemented; recording it as omitted would be a false green"
        );
        assert_eq!(on_load.rust, "on_load(handler)");
        assert_eq!(row("delayMs", "Avatar.Fallback").rust, "delay_ms(u64)");
        let fallback_color = row("color", "Avatar.Fallback");
        assert_eq!(fallback_color.rust, "fallback_color(Color)");
        assert!(
            fallback_color.description.contains("not an alias"),
            "Avatar.Fallback.color is its own prop, distinct from the parent color"
        );
    }

    #[test]
    fn fieldset_metadata_tracks_parts_and_disabled_limitation() {
        let metadata = reference_metadata::for_route(
            "Fieldset",
            "use herogpui::components::field::{Fieldset, FieldGroup, FieldsetLegend, FieldsetActions};",
        )
        .expect("Fieldset route is registered");
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(
            metadata.states.len(),
            0,
            "the fieldset is a static layout container"
        );
        let native = metadata
            .api
            .iter()
            .find(|row| row.owner == "Fieldset" && row.prop == "nativeProps")
            .expect("nativeProps row");
        assert_eq!(
            native.status,
            reference_metadata::ImplementationStatus::Unavailable
        );
        assert!(
            native.description.contains("no disabled state"),
            "the disabled forwarding limitation is recorded, not hidden"
        );
        let actions = metadata
            .parts
            .iter()
            .find(|part| part.name == "Fieldset.Actions")
            .expect("actions part");
        assert!(actions.description.contains("pt-1"));
    }

    #[test]
    fn field_slots_metadata_covers_all_four_slots_and_label_states() {
        let metadata = reference_metadata::for_route(
            "FieldSlots",
            "use herogpui::components::field::{Description, ErrorMessage, FieldError, Label};",
        )
        .expect("FieldSlots route is registered");
        assert_eq!(metadata.parts.len(), metadata.required_parts.len());
        assert_eq!(
            metadata.states.len(),
            3,
            "label required, disabled and invalid are the slot states"
        );
        let field_error = metadata
            .api
            .iter()
            .find(|row| row.owner == "FieldError" && row.prop == "children")
            .expect("FieldError children row");
        assert_eq!(
            field_error.status,
            reference_metadata::ImplementationStatus::Partial,
            "the validation render function is not portable; the message form is"
        );
    }

    #[test]
    fn forbidden_display_token_rejects_known_jargon() {
        assert_eq!(forbidden_display_token("use className"), Some("className"));
        assert_eq!(forbidden_display_token("a ReactNode here"), Some("React"));
        assert_eq!(
            forbidden_display_token(".button--sm"),
            Some("CSS class token")
        );
        assert_eq!(
            forbidden_display_token("hover:bg-accent"),
            Some("pseudo selector or CSS declaration")
        );
        assert_eq!(
            forbidden_display_token("px-4 gap-2"),
            Some("utility class token")
        );
        assert_eq!(
            forbidden_display_token("--badge-bg"),
            Some("CSS custom property")
        );
        assert_eq!(forbidden_display_token("plain gpui builder text"), None);
        assert!(
            forbidden_display_token(&scrub_prose("mt-2; sm:mt-0")).is_none(),
            "styling utilities and breakpoint variants must scrub: {}",
            scrub_prose("mt-2; sm:mt-0")
        );
        assert!(
            forbidden_display_token(&scrub_prose("hover:bg-accent")).is_none(),
            "state-variant utilities must scrub: {}",
            scrub_prose("hover:bg-accent")
        );
    }

    #[test]
    fn display_translates_pinned_type_and_selector_evidence() {
        assert_eq!(
            TYPE_PHRASES
                .iter()
                .find(|(from, _)| *from == "RenderFunction")
                .map(|(_, to)| *to),
            Some("render closure")
        );
        assert_eq!(rust_value_type("RenderFunction"), "render closure");
        assert_eq!(
            rust_value_type("ReactNode | RenderFunction"),
            "AnyElement | render closure"
        );
        assert_eq!(
            rust_value_type("CheckboxFieldRenderFunction"),
            "CheckboxField render closure"
        );
        assert_eq!(
            rust_value_type("ReactNode | (values: ButtonRenderProps) => ReactNode"),
            "AnyElement | Fn(values: Button render props) -> AnyElement"
        );
        assert_eq!(
            rust_value_type(
                "DOMRenderFunction<keyof React.JSX.IntrinsicElements, TabsRenderProps>"
            ),
            "render closure<Tabs render props>"
        );
        assert_eq!(
            rust_value_type("(isOpen: boolean) => void"),
            "Fn(isOpen: bool) -> ()"
        );
        assert_eq!(
            scrub_prose(".button:hover / [data-hovered=\"true\"]"),
            "button hover / hovered"
        );
        assert_eq!(
            scrub_prose("Additional DOM classes have no GPUI analogue."),
            "Additional upstream style classes have no GPUI analogue."
        );
        assert_eq!(
            scrub_prose("Overrides the browser DOM root."),
            "Overrides the browser root element."
        );
    }

    #[test]
    fn metadata_display_carries_no_web_framework_instructions() {
        // Pinned source links are shown verbatim by design; their URL path
        // segments are not user instructions.
        fn without_pinned_links(cell: &str) -> String {
            let mut out = String::new();
            let mut rest = cell;
            while let Some(start) = rest.find("https://") {
                out.push_str(&rest[..start]);
                let end = rest[start..]
                    .find(|ch: char| ch.is_whitespace())
                    .map_or(rest.len(), |end| start + end);
                rest = &rest[end..];
            }
            out.push_str(rest);
            out
        }
        for metadata in reference_metadata::ALL {
            let display = cached_display_metadata(metadata);
            for rows in [
                display.api.as_slice(),
                display.parts.as_slice(),
                display.states.as_slice(),
                display.styling.as_slice(),
            ] {
                for row in rows {
                    for cell in &row.cells {
                        let cell = without_pinned_links(cell);
                        if let Some(token) = forbidden_display_token(&cell) {
                            panic!("{}: display cell contains {token}: {cell}", metadata.page);
                        }
                    }
                }
            }
            for cell in &display.contract.cells {
                let cell = without_pinned_links(cell);
                if let Some(token) = forbidden_display_token(&cell) {
                    panic!("{}: display cell contains {token}: {cell}", metadata.page);
                }
            }
        }
    }

    #[test]
    fn pinned_metadata_still_records_upstream_evidence() {
        // The display layer translates at render time; the checked-in rows
        // that api/reference/reason audits verify must keep the upstream
        // spellings untouched so the pinned v3.2.4 contract stays auditable.
        let all = reference_metadata::ALL;
        let api_rows = || all.iter().flat_map(|metadata| metadata.api.iter());
        assert!(api_rows().any(|row| row.prop == "className"));
        assert!(api_rows().any(|row| row.ty.contains("ReactNode")));
        assert!(api_rows().any(|row| row.ty.contains("DOMRenderFunction")));
        assert!(api_rows().any(|row| row.ty.contains("React.JSX.IntrinsicElements")));
        assert!(api_rows().any(|row| row.ty.contains("CSSProperties")));
        assert!(api_rows().any(|row| row.status == unavailable()));
        assert!(all.iter().any(|metadata| {
            metadata
                .states
                .iter()
                .any(|state| state.selector.contains(':'))
        }));
        assert!(all.iter().any(|metadata| {
            metadata
                .styling
                .iter()
                .any(|style| style.class_or_token.starts_with('.'))
        }));
    }

    fn unavailable() -> reference_metadata::ImplementationStatus {
        reference_metadata::ImplementationStatus::Unavailable
    }

    #[test]
    fn web_only_rows_are_marked_not_callable() {
        let mut marked = 0;
        for metadata in reference_metadata::ALL {
            for row in metadata
                .api
                .iter()
                .filter(|row| row.status == unavailable())
            {
                marked += 1;
                let displayed = api_display_row(row);
                assert_eq!(
                    displayed.cells[2], "Web-only — not callable from GPUI",
                    "{}: web-only row must not display a GPUI builder",
                    metadata.page
                );
            }
        }
        assert!(
            marked > 0,
            "the pinned contract still records web-only rows"
        );
    }

    #[test]
    fn display_keeps_rust_builders_and_pinned_source_links() {
        let metadata = reference_metadata::for_route(
            "Button",
            "use herogpui::prelude::{Button, Size, Variant};",
        )
        .expect("Button metadata is registered");
        let variant = metadata
            .api
            .iter()
            .find(|row| row.owner == "Button" && row.prop == "variant")
            .expect("Button variant row");
        let displayed = api_display_row(variant);
        assert_eq!(displayed.cells[0], "Button::variant");
        assert!(displayed.cells[2].starts_with("variant("));
        assert!(displayed
            .cells
            .iter()
            .all(|cell| forbidden_display_token(cell).is_none()));

        let contract = contract_display_row(metadata, 3);
        assert!(contract.cells[3].contains("/blob/v3.2.4/"));
        assert!(contract.cells[3].contains("https://"));
    }

    #[test]
    fn display_keeps_rust_prop_names_and_translates_markup_defaults() {
        assert_eq!(
            rust_prop_name("isFocusVisible / isFocusWithin"),
            "is_focus_visible / is_focus_within"
        );
        assert_eq!(rust_prop_name("className / render"), "style_class / render");
        assert_eq!(
            scrub_prose("Default: <CloseIcon />"),
            "Default: upstream element"
        );
        assert_eq!(
            scrub_prose("v3 renders a native <form>."),
            "v3 renders a native upstream element."
        );

        let metadata = reference_metadata::for_route(
            "NumberField",
            "use herogpui::components::number_field::{NumberField, NumberState};",
        )
        .expect("NumberField metadata is registered");
        let icon = metadata
            .api
            .iter()
            .find(|row| row.owner == "NumberField.IncrementButton" && row.prop == "children")
            .expect("increment icon row");
        let displayed = api_display_row(icon);
        assert_eq!(displayed.cells[0], "NumberField::children");
        assert!(displayed.cells[2].starts_with("increment_icon(icon)"));
        assert_eq!(
            displayed.cells[3],
            "Default: upstream element — Replaces the increment glyph while preserving the button's spin behavior."
        );
    }

    #[test]
    fn display_metadata_is_translated_once_and_reused() {
        let metadata =
            reference_metadata::for_route("Avatar", "use herogpui::components::avatar::Avatar;")
                .expect("Avatar metadata is registered");
        let first = cached_display_metadata(metadata) as *const DisplayMetadata;
        let second = cached_display_metadata(metadata) as *const DisplayMetadata;
        assert_eq!(first, second);
    }

    #[test]
    fn display_cache_lookup_uses_identity_not_page_title() {
        fn dummy(
            page: &'static str,
            import_line: &'static str,
        ) -> reference_metadata::ReferenceMetadata {
            reference_metadata::ReferenceMetadata {
                page,
                import_line,
                source_module: "slider",
                version: "3.2.4",
                docs_source: "",
                api_source: "",
                style_source: "",
                required_parts: &[],
                api: &[],
                parts: &[],
                states: &[],
                styling: &[],
            }
        }
        let all = [dummy("Shared", "use a;"), dummy("Shared", "use b;")];
        assert_eq!(display_cache_index(&all, &all[0]), Some(0));
        assert_eq!(display_cache_index(&all, &all[1]), Some(1));
        assert_ne!(
            display_cache_index(&all, &all[0]),
            display_cache_index(&all, &all[1]),
            "same page title must not collapse two metadata identities"
        );
    }

    #[test]
    fn display_cache_is_parallel_to_all_by_index() {
        let cache = display_metadata();
        assert_eq!(cache.len(), reference_metadata::ALL.len());
        let mut seen = BTreeSet::new();
        for (index, metadata) in reference_metadata::ALL.iter().enumerate() {
            assert!(
                seen.insert((metadata.page, metadata.import_line)),
                "duplicate metadata identity {} / {}",
                metadata.page,
                metadata.import_line
            );
            assert!(
                std::ptr::eq(cached_display_metadata(metadata), &cache[index]),
                "{} cache slot must follow ALL order, not page title",
                metadata.page
            );
        }
    }

    #[test]
    fn cached_rows_are_borrowed_into_the_table() {
        fn accept_borrowed_rows<'a>(rows: impl IntoIterator<Item = &'a DetailRow>) -> usize {
            rows.into_iter().count()
        }
        let metadata =
            reference_metadata::for_route("Slider", "use herogpui::components::slider::Slider;")
                .expect("Slider metadata is registered");
        let display = cached_display_metadata(metadata);
        assert!(accept_borrowed_rows(display.api.iter()) > 0);
        assert!(
            accept_borrowed_rows(
                display
                    .styling
                    .iter()
                    .chain(std::iter::once(&display.contract))
            ) > 1
        );
    }

    #[test]
    fn slider_track_row_records_axis_inset_as_implemented() {
        let metadata =
            reference_metadata::for_route("Slider", "use herogpui::components::slider::Slider;")
                .expect("Slider metadata is registered");
        let track = metadata
            .styling
            .iter()
            .find(|row| row.class_or_token == ".slider__track")
            .expect("slider track styling row");
        assert_eq!(
            track.status,
            reference_metadata::ImplementationStatus::Implemented
        );
        assert!(
            track.rust.contains("axis_inset(12px)")
                && track.rust.contains("fill_start/fill_end caps"),
            "the track row must name the 12px inset and fill caps: {}",
            track.rust
        );
        assert!(
            !track.rust.contains("no separate transparent end borders"),
            "stale Partial wording must not remain: {}",
            track.rust
        );
    }

    /// The `static` display cache may only hold translated data. A
    /// `gpui::AnyElement` is not `Send`, so an element-typed field added to
    /// `DisplayMetadata` would fail this bound at compile time.
    #[test]
    fn display_metadata_cache_holds_no_elements() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DisplayMetadata>();
        assert_send_sync::<DetailRow>();
    }

    #[test]
    fn display_keeps_rust_href_builders_but_rewords_prose_href() {
        // Builder evidence keeps the call name verbatim.
        assert_eq!(scrub_phrases("href(url)"), "href(url)");
        assert_eq!(scrub_phrases("href(href)"), "href(href)");
        assert_eq!(
            scrub_phrases("placed_field_panel + href(url)"),
            "placed_field_panel + href(url)"
        );
        // Prose href becomes words, including through a backticked span.
        assert_eq!(
            scrub_prose("a href crumb opens it through the OS URL handler"),
            "the link URL crumb opens it through the OS URL handler"
        );
        assert_eq!(
            scrub_prose("so `href` opens through the OS handler"),
            "so the link URL opens through the OS handler"
        );
        let metadata =
            reference_metadata::for_route("Link", "use herogpui::components::link::Link;")
                .expect("Link route is registered");
        let href = metadata
            .api
            .iter()
            .find(|row| row.prop == "href")
            .expect("Link href row");
        assert!(
            api_display_row(href).cells[2].contains("href(url)"),
            "the Link href builder name must survive display"
        );
    }

    #[test]
    fn display_repairs_articles_after_replacements() {
        assert_eq!(
            scrub_prose("does not expose a DOM render seam."),
            "does not expose an upstream render seam."
        );
        assert_eq!(
            scrub_prose("forwards a React Aria state that does not emit it."),
            "forwards the pinned upstream state that does not emit it."
        );
        assert_eq!(
            scrub_prose("cloning a React element with data-direction."),
            "cloning an upstream element with the upstream direction flag."
        );
        assert_eq!(
            scrub_prose("even with an href, and never takes the disabled fade."),
            "even with the link URL, and never takes the disabled fade."
        );
        // Correct articles survive untouched.
        assert_eq!(scrub_prose("a uniform grid"), "a uniform grid");
        assert_eq!(scrub_prose("an iterable of keys"), "an iterable of keys");
        assert_eq!(scrub_prose("a a submitted record"), "a submitted record");
    }

    #[test]
    fn display_preserves_unknown_pseudo_and_css_variable_words() {
        // Known pseudos still translate.
        assert_eq!(scrub_prose("on :hover"), "on hover");
        assert_eq!(
            scrub_prose(".checkbox__control::before"),
            "checkbox control before"
        );
        assert_eq!(scrub_prose("on :autofill"), "on autofill");
        // Unknown pseudo names keep their original spelling (Rust paths too).
        assert_eq!(
            scrub_prose("fed by ParentElement::extend children"),
            "fed by ParentElement::extend children"
        );
        assert_eq!(scrub_prose("matches ::selection"), "matches ::selection");
        // Unknown CSS variables keep their name words.
        assert_eq!(
            scrub_prose("background --field-focus on the input"),
            "background the upstream token field focus on the input"
        );
    }
}
