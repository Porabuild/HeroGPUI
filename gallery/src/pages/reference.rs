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
    let contract = format!(
        "{} · HeroUI v{} · source module {} · {} official source links checked in · {} required compound parts",
        metadata.page,
        metadata.version,
        metadata.source_module,
        source_count,
        metadata.required_parts.len()
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
            ),
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
    Some(match module {
        "accordion" => include_str!("../../../crates/herogpui-components/src/accordion.rs"),
        "alert" => include_str!("../../../crates/herogpui-components/src/alert.rs"),
        "alert_dialog" => include_str!("../../../crates/herogpui-components/src/alert_dialog.rs"),
        "autocomplete" => include_str!("../../../crates/herogpui-components/src/autocomplete.rs"),
        "avatar" => include_str!("../../../crates/herogpui-components/src/avatar.rs"),
        "badge" => include_str!("../../../crates/herogpui-components/src/badge.rs"),
        "breadcrumbs" => include_str!("../../../crates/herogpui-components/src/breadcrumbs.rs"),
        "button" => include_str!("../../../crates/herogpui-components/src/button.rs"),
        "button_group" => include_str!("../../../crates/herogpui-components/src/button_group.rs"),
        "calendar" => include_str!("../../../crates/herogpui-components/src/calendar.rs"),
        "card" => include_str!("../../../crates/herogpui-components/src/card.rs"),
        "checkbox" => include_str!("../../../crates/herogpui-components/src/checkbox.rs"),
        "chip" => include_str!("../../../crates/herogpui-components/src/chip.rs"),
        "close_button" => include_str!("../../../crates/herogpui-components/src/close_button.rs"),
        "color_picker" => include_str!("../../../crates/herogpui-components/src/color_picker.rs"),
        "combo_box" => include_str!("../../../crates/herogpui-components/src/combo_box.rs"),
        "date_picker" => include_str!("../../../crates/herogpui-components/src/date_picker.rs"),
        "disclosure" => include_str!("../../../crates/herogpui-components/src/disclosure.rs"),
        "drawer" => include_str!("../../../crates/herogpui-components/src/drawer.rs"),
        "dropdown" => include_str!("../../../crates/herogpui-components/src/dropdown.rs"),
        "field" => include_str!("../../../crates/herogpui-components/src/field.rs"),
        "form" => include_str!("../../../crates/herogpui-components/src/form.rs"),
        "input" => include_str!("../../../crates/herogpui-components/src/input.rs"),
        "input_group" => include_str!("../../../crates/herogpui-components/src/input_group.rs"),
        "input_otp" => include_str!("../../../crates/herogpui-components/src/input_otp.rs"),
        "kbd" => include_str!("../../../crates/herogpui-components/src/kbd.rs"),
        "link" => include_str!("../../../crates/herogpui-components/src/link.rs"),
        "list_box" => include_str!("../../../crates/herogpui-components/src/list_box.rs"),
        "meter" => include_str!("../../../crates/herogpui-components/src/meter.rs"),
        "modal" => include_str!("../../../crates/herogpui-components/src/modal.rs"),
        "number_field" => include_str!("../../../crates/herogpui-components/src/number_field.rs"),
        "pagination" => include_str!("../../../crates/herogpui-components/src/pagination.rs"),
        "popover" => include_str!("../../../crates/herogpui-components/src/popover.rs"),
        "progress" => include_str!("../../../crates/herogpui-components/src/progress.rs"),
        "radio_group" => include_str!("../../../crates/herogpui-components/src/radio_group.rs"),
        "range_calendar" => {
            include_str!("../../../crates/herogpui-components/src/range_calendar.rs")
        }
        "scroll_shadow" => include_str!("../../../crates/herogpui-components/src/scroll_shadow.rs"),
        "select" => include_str!("../../../crates/herogpui-components/src/select.rs"),
        "separator" => include_str!("../../../crates/herogpui-components/src/separator.rs"),
        "skeleton" => include_str!("../../../crates/herogpui-components/src/skeleton.rs"),
        "slider" => include_str!("../../../crates/herogpui-components/src/slider.rs"),
        "spinner" => include_str!("../../../crates/herogpui-components/src/spinner.rs"),
        "surface" => include_str!("../../../crates/herogpui-components/src/surface.rs"),
        "switch" => include_str!("../../../crates/herogpui-components/src/switch.rs"),
        "table" => include_str!("../../../crates/herogpui-components/src/table.rs"),
        "tabs" => include_str!("../../../crates/herogpui-components/src/tabs.rs"),
        "tag_group" => include_str!("../../../crates/herogpui-components/src/tag_group.rs"),
        "textarea" => include_str!("../../../crates/herogpui-components/src/textarea.rs"),
        "time_field" => include_str!("../../../crates/herogpui-components/src/time_field.rs"),
        "toast" => include_str!("../../../crates/herogpui-components/src/toast.rs"),
        "toggle_button" => include_str!("../../../crates/herogpui-components/src/toggle_button.rs"),
        "toolbar" => include_str!("../../../crates/herogpui-components/src/toolbar.rs"),
        "tooltip" => include_str!("../../../crates/herogpui-components/src/tooltip.rs"),
        "typography" => include_str!("../../../crates/herogpui-components/src/typography.rs"),
        _ => return None,
    })
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
        let methods = methods_for(
            include_str!("../../../crates/herogpui-components/src/button.rs"),
            &owners,
        );

        assert!(methods.iter().any(|method| method.name == "variant"));
        assert!(methods.iter().any(|method| method.name == "on_press"));
        assert!(methods
            .iter()
            .any(|method| method.name == "variant" && method.default == "Variant::Primary"));
        assert!(methods.iter().any(|method| method.name == "child"));
        assert!(!methods.iter().any(|method| method.name == "render"));
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
        let methods = methods_for(
            include_str!("../../../crates/herogpui-components/src/form.rs"),
            &owners,
        );

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
        let methods = methods_for(
            include_str!("../../../crates/herogpui-components/src/table.rs"),
            &owners,
        );

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
        for prop in ["focusedValue", "minValue", "maxValue", "firstDayOfWeek"] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "Calendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        for (owner, prop) in [
            ("Calendar.YearPickerTriggerHeading", "offset"),
            ("Calendar.YearPickerGrid", "visibleYears"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
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
            "firstDayOfWeek",
            "selectionAlignment",
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == "RangeCalendar"
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
        for (owner, prop) in [
            ("RangeCalendar.YearPickerTriggerHeading", "offset"),
            ("RangeCalendar.YearPickerGrid", "visibleYears"),
        ] {
            assert!(metadata.api.iter().any(|entry| {
                entry.owner == owner
                    && entry.prop == prop
                    && entry.status == reference_metadata::ImplementationStatus::Partial
            }));
        }
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
        let methods = methods_for(
            include_str!("../../../crates/herogpui-components/src/toggle_button.rs"),
            &owners,
        );

        assert_eq!(
            methods
                .iter()
                .filter(|method| method.name == "child")
                .count(),
            1
        );
    }
}
