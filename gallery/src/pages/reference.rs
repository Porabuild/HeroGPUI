use std::collections::{BTreeMap, BTreeSet};

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
    examples: &[(&str, AnyElement, &str)],
    cx: &App,
) -> Vec<(&'static str, AnyElement)> {
    if let Some(metadata) = reference_metadata::for_import(import_line) {
        return metadata_panels(metadata, cx);
    }

    let Some(source) = source_for(import_line) else {
        return Vec::new();
    };
    let owners = referenced_types(import_line, examples.iter().map(|(_, _, code)| *code));
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

fn metadata_panels(
    metadata: &reference_metadata::ReferenceMetadata,
    cx: &App,
) -> Vec<(&'static str, AnyElement)> {
    let source_count = [
        metadata.docs_source,
        metadata.api_source,
        metadata.style_source,
    ]
    .iter()
    .flat_map(|source| source.split(" + "))
    .filter(|source| !source.is_empty())
    .count();
    let part_suffix = if metadata.required_parts.len() == 1 {
        "part"
    } else {
        "parts"
    };
    let contract = format!(
        "{} · HeroUI v{} · source module {} · {} official source links checked in · {} required compound {}",
        metadata.page,
        metadata.version,
        metadata.source_module,
        source_count,
        metadata.required_parts.len(),
        part_suffix
    );

    vec![
        (
            "API Reference",
            detail_table(
                [
                    "Part / prop",
                    "Type",
                    "Rust implementation",
                    "Default / description",
                ],
                metadata.api.iter().map(|entry| DetailRow {
                    cells: [
                        format!("{}::{}", entry.owner, entry.prop),
                        entry.ty.to_owned(),
                        format!(
                            "{}::{} · {}",
                            entry.rust_owner,
                            entry.rust,
                            entry.status.label()
                        ),
                        format!("Default: {} — {}", entry.default, entry.description),
                    ],
                }),
                cx,
            ),
        ),
        (
            "Parts & Slots",
            detail_table(
                ["Part", "Slot", "Rust owner / status", "Description"],
                metadata.parts.iter().map(|entry| DetailRow {
                    cells: [
                        entry.name.to_owned(),
                        entry.slot.to_owned(),
                        format!("{} · {}", entry.rust_owner, entry.status.label()),
                        entry.description.to_owned(),
                    ],
                }),
                cx,
            ),
        ),
        (
            "States",
            if metadata.states.is_empty() {
                empty_panel("v3 documents no interactive states for this component.", cx)
            } else {
                detail_table(
                    ["State", "v3 selector", "Rust implementation", "Description"],
                    metadata.states.iter().map(|entry| DetailRow {
                        cells: [
                            entry.state.to_owned(),
                            entry.selector.to_owned(),
                            format!("{} · {}", entry.rust, entry.status.label()),
                            entry.description.to_owned(),
                        ],
                    }),
                    cx,
                )
            },
        ),
        (
            "Styling Reference",
            detail_table(
                [
                    "CSS class / token",
                    "v3 value",
                    "Rust implementation",
                    "Description",
                ],
                metadata
                    .styling
                    .iter()
                    .map(|entry| DetailRow {
                        cells: [
                            entry.class_or_token.to_owned(),
                            entry.value.to_owned(),
                            format!("{} · {}", entry.rust, entry.status.label()),
                            entry.description.to_owned(),
                        ],
                    })
                    .chain(std::iter::once(DetailRow {
                        cells: [
                            "Contract".to_owned(),
                            format!("HeroUI v{}", metadata.version),
                            format!("{source_count} source links"),
                            contract,
                        ],
                    })),
                cx,
            ),
        ),
    ]
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

fn detail_table(
    headers: [&'static str; 4],
    rows: impl Iterator<Item = DetailRow>,
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
        .child(gpui::div().w(px(180.)).flex_shrink_0().child(headers[0]))
        .child(gpui::div().w(px(220.)).flex_shrink_0().child(headers[1]))
        .child(gpui::div().w(px(220.)).flex_shrink_0().child(headers[2]))
        .child(gpui::div().flex_1().min_w_0().child(headers[3]));

    gpui::div()
        .w_full()
        .rounded(px(12.))
        .border_1()
        .border_color(colors.border)
        .overflow_hidden()
        .child(header)
        .children(rows.map(|row| {
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
                        .w(px(180.))
                        .flex_shrink_0()
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(row.cells[0].clone()),
                )
                .child(
                    gpui::div()
                        .w(px(220.))
                        .flex_shrink_0()
                        .font_family(crate::app::MONO_FONT)
                        .text_color(colors.foreground)
                        .child(row.cells[1].clone()),
                )
                .child(
                    gpui::div()
                        .w(px(220.))
                        .flex_shrink_0()
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
                && entry.status == reference_metadata::ImplementationStatus::Partial
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
            "selectionAlignment",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "RangeCalendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Implemented
            }));
        }
        for prop in ["isDateUnavailable", "firstDayOfWeek"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "RangeCalendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
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
            "use herogpui::components::drawer::{Drawer, DrawerPlacement};",
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
            "use herogpui::components::modal::{Modal, ModalSize};",
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
            ("Modal.CloseTrigger", "children"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
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
            "use herogpui::components::popover::{Popover, PopoverPlacement};",
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
        assert!(metadata.styling.iter().any(|entry| {
            entry.class_or_token == ".popover [data-slot=popover-overlay-arrow]"
                && entry.status == reference_metadata::ImplementationStatus::Implemented
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
            ("Select", "fullWidth"),
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
                && entry.status == reference_metadata::ImplementationStatus::Partial
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
            ("Table.Column", "defaultWidth"),
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
            ("Table.Column", "width"),
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
}
