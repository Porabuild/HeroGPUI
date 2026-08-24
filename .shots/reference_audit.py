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


def array_body(source, name):
    match = re.search(
        rf"const\s+{re.escape(name)}\b.*?=\s*&\[(.*?)\n\];",
        source,
        re.S,
    )
    if not match:
        raise ValueError(f"missing metadata array {name}")
    return match.group(1)


def metadata_rows(source, ref, item):
    body = array_body(source, ref)
    return body.count(f"{item} {{"), body


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
    body = array_body(source, ref)
    return re.findall(rf"{item}\s*\{{(.*?)\n\s*\}},?", body, re.S)


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


def main():
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

        counts = {}
        for name, item in (
            ("api", "ApiDoc"),
            ("parts", "PartDoc"),
            ("states", "StateDoc"),
            ("styling", "StyleDoc"),
        ):
            if name not in refs:
                continue
            try:
                counts[name], _ = metadata_rows(metadata, refs[name], item)
            except ValueError as error:
                errors.append(f"{page}: {error}")
                continue
            totals[name] += counts[name]
            if counts[name] == 0 and name in ("api", "parts", "styling"):
                errors.append(f"{page}: zero {name} input")
        if all(counts.get(name, 0) for name in ("api", "parts", "styling")):
            detailed += 1

        try:
            required_body = array_body(metadata, refs["required_parts"])
            parts_body = array_body(metadata, refs["parts"])
            api_body = array_body(metadata, refs["api"])
            state_body = array_body(metadata, refs["states"])
            style_body = array_body(metadata, refs["styling"])
        except (KeyError, ValueError) as error:
            errors.append(f"{page}: {error}")
            continue

        required = set(re.findall(r'"([^\"]+)"', required_body))
        part_names = set(re.findall(r'name:\s*"([^\"]+)"', parts_body))
        if required != part_names:
            errors.append(
                f"{page}: compound parts changed; missing={sorted(required - part_names)}, extra={sorted(part_names - required)}"
            )

        api_records = records(metadata, refs["api"], "ApiDoc")
        for entry in api_records:
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

        for entry in records(metadata, refs["parts"], "PartDoc"):
            owner = field(entry, "rust_owner")
            if not owner or owner not in methods:
                errors.append(f"{page}: unresolved part Rust owner {owner or '<empty>'}")

        for entry in records(metadata, refs["states"], "StateDoc"):
            if not field(entry, "selector") or not field(entry, "rust"):
                errors.append(f"{page}: state row has incomplete selector/source mapping")

        for entry in records(metadata, refs["styling"], "StyleDoc"):
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
