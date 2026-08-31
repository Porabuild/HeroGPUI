"""Audit checked-in reference metadata without contacting the network.

Metadata is deliberately partial while later component waves are populated.
Every registered page still has to resolve to one real route, one component
source module, and mechanically verifiable Rust mappings.
"""

import io
import os
import re
import sys


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
METADATA = os.path.join(ROOT, "gallery", "src", "pages", "reference_metadata.rs")
PAGES = os.path.join(ROOT, "gallery", "src", "pages", "mod.rs")
SOURCE_ROOT = os.path.join(ROOT, "crates", "herogpui-components", "src")


def skip_string(source, index):
    """Return the index just past the string literal opening at index."""
    i = index + 1
    while i < len(source):
        if source[i] == "\\":
            i += 2
        elif source[i] == '"':
            return i + 1
        else:
            i += 1
    raise ValueError(f"unterminated string literal at offset {index}")


def skip_comment(source, index):
    if source.startswith("//", index):
        end = source.find("\n", index)
        return len(source) if end < 0 else end
    if source.startswith("/*", index):
        end = source.find("*/", index + 2)
        if end < 0:
            raise ValueError(f"unterminated block comment at offset {index}")
        return end + 2
    return index + 1


def skip_balanced(source, index, open_char, close_char):
    """Return the index just past the bracket group opening at index."""
    depth = 1
    i = index + 1
    while i < len(source):
        char = source[i]
        if char == '"':
            i = skip_string(source, i)
        elif char == "/" and source[i : i + 2] in ("//", "/*"):
            i = skip_comment(source, i)
        elif char == open_char:
            depth += 1
            i += 1
        elif char == close_char:
            depth -= 1
            i += 1
            if depth == 0:
                return i
        else:
            i += 1
    raise ValueError(f"unclosed {open_char}...{close_char} group at offset {index}")


def masked(source):
    """Blank string and comment text so token counts ignore their contents."""
    out = list(source)
    i = 0
    while i < len(source):
        char = source[i]
        if char == '"':
            end = skip_string(source, i)
            for j in range(i + 1, end - 1):
                if out[j] != "\n":
                    out[j] = " "
            i = end
        elif char == "/" and source[i : i + 2] in ("//", "/*"):
            end = skip_comment(source, i)
            for j in range(i, end):
                if out[j] != "\n":
                    out[j] = " "
            i = end
        else:
            i += 1
    return "".join(out)


def array_body(source, name):
    """Return the element text between the brackets of one declared array."""
    match = re.search(
        rf"(?m)^(?:pub(?:\([^)]*\))?\s+)?const\s+{re.escape(name)}\b", source
    )
    if not match:
        raise ValueError(f"missing metadata array {name}")
    following = re.search(
        r"(?m)^(?:pub(?:\([^)]*\))?\s+)?const\s+[A-Za-z_]", source[match.end() :]
    )
    limit = match.end() + following.start() if following else len(source)
    initializer = re.compile(r"=\s*&\[").search(source, match.end(), limit)
    if not initializer:
        raise ValueError(f"metadata array {name} has no &[ initializer")
    start = initializer.end() - 1
    end = skip_balanced(source, start, "[", "]")
    return source[start + 1 : end - 1]


def field(block, name):
    match = re.search(
        rf'(?<![A-Za-z0-9_]){name}:\s*"((?:\\.|[^"\\])*)"',
        block,
    )
    return match.group(1).replace(r'\"', '"') if match else None


def enum_field(block, name):
    match = re.search(rf"(?<![A-Za-z0-9_]){name}:\s*ImplementationStatus::(\w+)", block)
    return match.group(1) if match else None


def records(source, ref, item):
    """Parse every record literal in one declared array, single- or multi-line."""
    body = array_body(source, ref)
    opener = re.compile(rf"\b{item}\s*\{{")
    declared = len(opener.findall(masked(body)))
    parsed = []
    i = 0
    while i < len(body):
        char = body[i]
        if char == '"':
            i = skip_string(body, i)
        elif char == "/" and body[i : i + 2] in ("//", "/*"):
            i = skip_comment(body, i)
        else:
            match = opener.match(body, i)
            if match:
                end = skip_balanced(body, match.end() - 1, "{", "}")
                parsed.append(body[match.start() : end])
                i = end
            else:
                i += 1
    if declared != len(parsed):
        raise ValueError(f"{ref}: declared {declared} {item} rows but parsed {len(parsed)}")
    return parsed


def api_row_errors(page, entry, methods):
    errors = []
    status = enum_field(entry, "status")
    rust_owner = field(entry, "rust_owner")
    rust = field(entry, "rust") or ""
    if not rust_owner:
        errors.append(f"{page}: API row has no rust_owner")
    elif rust_owner not in methods:
        errors.append(f"{page}: unresolved Rust owner {rust_owner}")
    if status in ("Implemented", "Partial") and rust != "—":
        method = mapping_method(rust)
        if method:
            source_signature = methods.get(rust_owner, {}).get(method)
            if source_signature is None:
                errors.append(f"{page}: {rust_owner}::{method} is not a real public method")
            elif argument_count(rust) != method_argument_count(source_signature):
                errors.append(
                    f"{page}: {rust_owner}::{method} mapping arity does not match source signature"
                )
        elif status == "Implemented":
            errors.append(f"{page}: Implemented API row has no method mapping: {rust}")
    return errors


def route_imports(page_source):
    section = page_source.split("pub fn import_line", 1)
    if len(section) != 2:
        raise ValueError("could not locate Page::import_line")
    section = section[1].split("pub fn docs_root", 1)[0]
    entries = list(re.finditer(r"Page::([A-Za-z0-9_]+)\s*=>", section))
    routes = {}
    for index, entry in enumerate(entries):
        end = entries[index + 1].start() if index + 1 < len(entries) else len(section)
        chunk = section[entry.end() : end]
        imports = re.findall(r'"(use\s+[^"\n]+)"', chunk)
        routes.setdefault(entry.group(1), []).extend(imports[:1])
    return routes


def owner_methods(source):
    """Return public inherent method names and signatures grouped by owner."""
    result = {}
    for match in re.finditer(r"\bimpl\s+([A-Za-z_][A-Za-z0-9_]*)([^\{]*)\{", source):
        header = match.group(0)
        if " for " in header:
            continue
        depth = 1
        index = match.end()
        while index < len(source) and depth:
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
            index += 1
        body = source[match.end() : index - 1]
        result.setdefault(match.group(1), {}).update(
            {
                name: signature
                for name, signature in re.findall(
                    r"\bpub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(\([^\{;]*\))",
                    body,
                    re.S,
                )
            }
        )
    return result


def mapping_method(mapping):
    match = re.match(
        r"\s*(?:[A-Za-z_][A-Za-z0-9_]*::)?([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        mapping,
    )
    return match.group(1) if match else None


def argument_count(signature):
    start = signature.find("(")
    if start < 0:
        return None
    depth = 0
    count = 0
    has_argument = False
    for char in signature[start + 1 :]:
        if char in "([<":
            depth += 1
        elif char == ")" and depth == 0:
            return count + int(has_argument)
        elif char in ")]>" and depth:
            depth -= 1
        elif char == "," and depth == 0:
            count += 1
            has_argument = False
        elif not char.isspace():
            has_argument = True
    return None


def method_argument_count(signature):
    count = argument_count(signature)
    if count is None:
        return None
    depth = 0
    first = []
    for char in signature.split("(", 1)[1]:
        if (char == "," and depth == 0) or (char == ")" and depth == 0):
            break
        if char in "([<":
            depth += 1
        elif char in ")]>" and depth:
            depth -= 1
        first.append(char)
    first = "".join(first).lstrip()
    return count - int(first.startswith(("self", "mut self", "&self", "&mut self")))


def rust_blocks_after(source, marker):
    """Yield balanced Rust blocks whose opening statement contains marker."""
    for match in re.finditer(re.escape(marker), source):
        opening = source.find("{", match.end())
        if opening < 0:
            continue
        depth = 1
        index = opening + 1
        while index < len(source) and depth:
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
            index += 1
        if depth == 0:
            yield source[opening + 1 : index - 1]


def dropdown_context_value(source, selector):
    """Read the numeric proof for one Dropdown-only nested override."""
    if selector.endswith('[data-slot="dropdown-menu"]'):
        pattern = r'panel\s*=\s*panel\.p\(px\(([0-9.]+)\.\)\)'
    elif selector.endswith('[data-slot="menu-item"]'):
        pattern = r'row\s*=\s*row\.px\(px\(([0-9.]+)\.\)\)'
    else:
        return None
    for body in rust_blocks_after(source, "if dropdown_composition"):
        match = re.search(pattern, body)
        if match:
            return float(match.group(1))
    return None


def self_test():
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    def expect_error(parse, message):
        try:
            parse()
        except ValueError as error:
            expect(message in str(error), f"expected {message!r} in {str(error)!r}")
        else:
            failures.append(f"expected ValueError: {message}")

    component_source = """impl Widget {
    pub fn variant(&mut self, variant: Variant) -> &mut Self {
        self
    }

    pub fn full_width(&mut self, full: bool) -> &mut Self {
        self
    }
}
"""
    metadata_source = r'''
const WIDGET_REQUIRED_PARTS: &[&str] = &["Widget"];

const WIDGET_SPLIT: &[&str] =
    &["Widget.Split"];

const WIDGET_API: &[ApiDoc] = &[
    ApiDoc {
        owner: "Widget",
        prop: "variant",
        ty: "'solid' | 'outline'",
        default: "'solid'",
        description: "Visual weight with {brace} and \"quoted\" text.",
        rust_owner: "Widget",
        rust: "variant(Variant)",
        status: ImplementationStatus::Implemented,
    },
    ApiDoc { owner: "Widget", prop: "fullWidth", ty: "boolean", default: "false", description: "One row on one line.", rust_owner: "Widget", rust: "full_width(bool)", status: ImplementationStatus::Implemented },
];

const WIDGET_PARTS: &[PartDoc] = &[PartDoc {
    name: "Widget",
    slot: "widget",
    description: "Root part.",
    rust_owner: "Widget",
    status: ImplementationStatus::Implemented,
}];
'''

    rows = records(metadata_source, "WIDGET_API", "ApiDoc")
    expect(len(rows) == 2, f"expected 2 ApiDoc rows, parsed {len(rows)}")
    expect(
        field(rows[0], "description") == 'Visual weight with {brace} and "quoted" text.',
        f"multi-line row field read {field(rows[0], 'description')!r}",
    )
    expect(field(rows[1], "prop") == "fullWidth", "single-line row not parsed")
    expect(len(records(metadata_source, "WIDGET_PARTS", "PartDoc")) == 1, "PartDoc row not parsed")
    expect(array_body(metadata_source, "WIDGET_SPLIT") == '"Widget.Split"', "initializer split across lines not read")

    methods = owner_methods(component_source)
    expect("variant" in methods.get("Widget", {}), "owner_methods missed Widget::variant")
    clean = rows[1]
    expect(api_row_errors("Widget", clean, methods) == [], f"clean row flagged: {api_row_errors('Widget', clean, methods)}")

    corrupt = clean.replace('rust_owner: "Widget"', 'rust_owner: "Widge"')
    flagged = api_row_errors("Widget", corrupt, methods)
    expect(
        any("unresolved Rust owner Widge" in error for error in flagged),
        f"corrupt rust_owner not flagged: {flagged}",
    )

    ghost = clean.replace("full_width(bool)", "no_such_method(bool)")
    flagged = api_row_errors("Widget", ghost, methods)
    expect(
        any("Widget::no_such_method is not a real public method" in error for error in flagged),
        f"non-existent method mapping not flagged: {flagged}",
    )

    unmapped = clean.replace("full_width(bool)", "resolved in render")
    flagged = api_row_errors("Widget", unmapped, methods)
    expect(
        any("Implemented API row has no method mapping" in error for error in flagged),
        f"implemented row without mapping not flagged: {flagged}",
    )

    arity = clean.replace("full_width(bool)", "full_width(bool, Variant)")
    flagged = api_row_errors("Widget", arity, methods)
    expect(
        flagged
        == ["Widget: Widget::full_width mapping arity does not match source signature"],
        f"arity mismatch not flagged exactly: {flagged}",
    )

    unclosed = 'const A: &[ApiDoc] = &[\n    ApiDoc {\n        prop: "x",\n'
    expect_error(lambda: records(unclosed, "A", "ApiDoc"), "unclosed")

    mismatched = (
        "const A: &[ApiDoc] = &[\n"
        "    ApiDoc {\n"
        '        prop: "x",\n'
        '        inner: ApiDoc { prop: "nested" },\n'
        "    },\n"
        "];"
    )
    expect_error(
        lambda: records(mismatched, "A", "ApiDoc"),
        "A: declared 2 ApiDoc rows but parsed 1",
    )

    commented = (
        "const A: &[ApiDoc] = &[\n"
        '    // ApiDoc { prop: "commented out" }\n'
        '    ApiDoc { owner: "A", prop: "y", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    expect(len(records(commented, "A", "ApiDoc")) == 1, "commented-out row miscounted")

    block_commented = (
        "const A: &[ApiDoc] = &[\n"
        "    /*\n"
        "    ApiDoc {\n"
        '        prop: "commented out",\n'
        "    },\n"
        "    */\n"
        '    ApiDoc { owner: "A", prop: "y", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    expect(len(records(block_commented, "A", "ApiDoc")) == 1, "block-commented row miscounted")

    backslashed = (
        "const A: &[ApiDoc] = &[\n"
        '    ApiDoc { owner: "A", prop: "y", description: "ends with a backslash \\\\", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    backslash_rows = records(backslashed, "A", "ApiDoc")
    expect(
        len(backslash_rows) == 1
        and field(backslash_rows[0], "description") == "ends with a backslash \\\\",
        "escaped backslash before the closing quote misparsed",
    )

    pub_crate = (
        "pub(crate) const P_API: &[ApiDoc] = &[\n"
        '    ApiDoc { owner: "P", prop: "y", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    expect(len(records(pub_crate, "P_API", "ApiDoc")) == 1, "pub(crate) array not located")

    quoted = (
        "const A: &[ApiDoc] = &[\n"
        '    ApiDoc { owner: "A", prop: "y", description: "mentions ApiDoc { inline", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    expect(len(records(quoted, "A", "ApiDoc")) == 1, "record opener inside a string miscounted")

    scoped = (
        'const A: &[&str] = &["x"];\n'
        "const A_EXTRA: &[ApiDoc] = &[\n"
        '    ApiDoc { owner: "A", prop: "y", status: ImplementationStatus::Unavailable },\n'
        "];"
    )
    expect(records(scoped, "A", "ApiDoc") == [], "array scoping crossed into A_EXTRA")
    expect(len(records(scoped, "A_EXTRA", "ApiDoc")) == 1, "A_EXTRA array not scoped correctly")

    expect_error(lambda: array_body(scoped, "MISSING"), "missing metadata array MISSING")

    if failures:
        print("self-test FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("self-test PASS: record parser and API row checks")
    return 0


def main():
    if "--self-test" in sys.argv[1:]:
        return self_test()
    metadata = io.open(METADATA, encoding="utf-8").read()
    pages = io.open(PAGES, encoding="utf-8").read()
    errors = []
    routes = route_imports(pages)
    component_pages = sum(bool(imports) for imports in routes.values())

    registered = re.findall(
        r"(?:pub\(crate\)|pub)\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*ReferenceMetadata\s*=\s*ReferenceMetadata\s*\{(.*?)\n\};",
        metadata,
        re.S,
    )
    if not registered:
        errors.append("no registered metadata pages")

    seen_pages = {}
    seen_imports = {}
    detailed = 0
    totals = {"api": 0, "parts": 0, "states": 0, "styling": 0}
    for constant, block in registered:
        page = field(block, "page") or constant
        import_line = field(block, "import_line")
        module = field(block, "source_module")
        seen_pages[page] = seen_pages.get(page, 0) + 1
        if import_line:
            seen_imports[import_line] = seen_imports.get(import_line, 0) + 1

        matching_routes = routes.get(page, [])
        if len(matching_routes) != 1:
            errors.append(f"{page}: expected exactly one route, found {len(matching_routes)}")
        elif import_line != matching_routes[0]:
            errors.append(f"{page}: metadata import does not match its Page route")
        if not module:
            errors.append(f"{page}: missing source_module")
            continue
        source_path = os.path.join(SOURCE_ROOT, f"{module}.rs")
        if not os.path.isfile(source_path):
            errors.append(f"{page}: source module does not exist: {module}.rs")
            continue
        source = io.open(source_path, encoding="utf-8").read()
        methods = owner_methods(source)

        refs = {}
        for name in ("api", "parts", "states", "styling", "required_parts"):
            match = re.search(
                rf"(?<![A-Za-z0-9_]){name}:\s*([A-Z][A-Z0-9_]*)",
                block,
            )
            if not match:
                errors.append(f"{page}: missing {name} reference")
            else:
                refs[name] = match.group(1)

        parsed = {}
        for name, item in (
            ("api", "ApiDoc"),
            ("parts", "PartDoc"),
            ("states", "StateDoc"),
            ("styling", "StyleDoc"),
        ):
            if name not in refs:
                continue
            try:
                parsed[name] = records(metadata, refs[name], item)
            except ValueError as error:
                errors.append(f"{page}: {error}")
                continue
            totals[name] += len(parsed[name])
            if not parsed[name] and name in ("api", "parts", "styling"):
                errors.append(f"{page}: zero {name} input")
        if all(parsed.get(name) for name in ("api", "parts", "styling")):
            detailed += 1

        try:
            required_body = array_body(metadata, refs["required_parts"])
        except (KeyError, ValueError) as error:
            errors.append(f"{page}: {error}")
            continue

        required = set(re.findall(r'"([^\"]+)"', required_body))
        part_names = {
            name
            for name in (field(entry, "name") for entry in parsed.get("parts", []))
            if name
        }
        if "parts" in parsed and required != part_names:
            errors.append(
                f"{page}: compound parts changed; missing={sorted(required - part_names)}, extra={sorted(part_names - required)}"
            )

        for entry in parsed.get("api", []):
            errors.extend(api_row_errors(page, entry, methods))

        for entry in parsed.get("parts", []):
            owner = field(entry, "rust_owner")
            if not owner or owner not in methods:
                errors.append(f"{page}: unresolved part Rust owner {owner or '<empty>'}")

        for entry in parsed.get("states", []):
            if not field(entry, "selector") or not field(entry, "rust"):
                errors.append(f"{page}: state row has incomplete selector/source mapping")

        for entry in parsed.get("styling", []):
            selector = field(entry, "class_or_token") or ""
            value = field(entry, "value") or ""
            rust = field(entry, "rust") or ""
            status = enum_field(entry, "status")
            if not selector or not value or not field(entry, "rust"):
                errors.append(f"{page}: style row has incomplete selector/source mapping")
            if ".dropdown__trigger" in selector and status == "Implemented":
                errors.append(f"{page}: trigger styling cannot claim Implemented without trigger source proof")
            if selector == ".menu-item" and "px-2.5" in value and status != "Partial":
                errors.append(f"{page}: Dropdown menu-item px-2.5 override must remain Partial")
            if page == "Dropdown" and selector.startswith(".dropdown__popover [data-slot="):
                proof = dropdown_context_value(source, selector)
                mapping = re.search(r'[.](?:p|px)[(]px[(]([0-9.]+)[)][)]', rust)
                if status == "Implemented" and (
                    proof is None
                    or mapping is None
                    or abs(proof - float(mapping.group(1))) > 0.01
                ):
                    errors.append(
                        f"{page}: Implemented contextual style has no matching Dropdown source proof: {selector}"
                    )

        for key in ("docs_source", "api_source", "style_source"):
            url = field(block, key) or ""
            if "/blob/v3.2.4/" not in url:
                errors.append(f"{page}: {key} is not pinned to v3.2.4")

    for page, count in seen_pages.items():
        if count != 1:
            errors.append(f"{page}: duplicate metadata registration ({count})")
    for import_line, count in seen_imports.items():
        if count != 1:
            errors.append(f"metadata import is registered {count} times")

    fallback = component_pages - detailed
    print(f"component pages : {component_pages}")
    print(f"metadata pages  : {len(registered)}")
    print(f"detailed        : {detailed}/{component_pages}")
    print(f"generic fallback: {fallback}/{component_pages}")
    print(
        "metadata rows   : API={api}, parts={parts}, states={states}, styling={styling}".format(
            **totals
        )
    )
    print("contract        : HeroUI v3.2.4, checked in; runtime network calls: 0")
    if errors:
        print("FAIL")
        for error in errors:
            print(f"- {error}")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
