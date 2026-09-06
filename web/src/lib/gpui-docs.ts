interface ApiRow {
  type: string;
  default?: string | null;
  description: string;
  rust?: string | null;
  status: string;
}

interface PartRow {
  name: string;
  description: string;
  rustOwner: string | null;
  status: string;
}

interface StateRow {
  state: string;
  description: string;
  rust: string | null;
  status: string;
}

/**
 * Display helpers that turn the checked-in HeroUI-shaped reference metadata
 * into the Rust/GPUI API a reader of these docs actually calls. Unavailable
 * web-only rows are dropped; remaining text is rewritten so CSS, React, and
 * HeroUI names are not the primary vocabulary.
 */

const PLACEHOLDER = /^(?:—|-|–)$/;

const TYPE_PHRASES: [string, string][] = [
  ["FormEvent<HTMLFormElement>", "form submit event"],
  ["ChangeEvent<HTMLInputElement>", "change event"],
  ["ChangeEvent<HTMLTextAreaElement>", "change event"],
  ["SyntheticEvent<HTMLImageElement>", "image event"],
  ["HTMLAttributes", "element attributes"],
  ["HTMLFormElement", "form"],
  ["HTMLInputElement", "field"],
  ["HTMLTextAreaElement", "field"],
  ["HTMLButtonElement", "button"],
  ["HTMLDivElement", "element"],
  ["HTMLLegendElement", "legend"],
  ["HTMLFieldSetElement", "fieldset"],
  ["HTMLImageElement", "image"],
  ["HTMLElement", "element"],
  ["RefObject", "element reference"],
  ["ValidityState", "validity state"],
  ["DOMRenderFunction", "render closure"],
  ["RenderFunction", "render closure"],
  ["ReactNode", "AnyElement"],
  ["CSSProperties", "style"],
  ["Iterable<Key>", "iterable of keys"],
  ["Iterable<T>", "iterable of T"],
  ["boolean", "bool"],
];

const SCRUB_PHRASES: [string, string][] = [
  ["rather than React Aria's PressEvent", ""],
  ["rather than a browser PressEvent", ""],
  ["; the pinned stylesheet also defines danger-soft.", "."],
  ["from the pinned stylesheet", ""],
  ["the pinned stylesheet", "the theme"],
  ["the pinned React Aria", "the"],
  ["the React Aria", "the"],
  ["React Aria's", ""],
  ["React Aria", ""],
  ["React Stately", ""],
  ["React collection nodes", "collection items"],
  ["a React element", "an element"],
  ["React element", "element"],
  ["React prop", "prop"],
  ["ReactNode", "AnyElement"],
  ["React", ""],
  ["HeroUI's", ""],
  ["HeroUI", ""],
  ["the pinned ", "the "],
  ["DOM element substitution", "root substitution"],
  ["DOM render function", "render closure"],
  ["DOM root substitution", "root-element substitution"],
  ["browser DOM root", "root element"],
  ["browser DOM", "the platform"],
  ["DOM element", "element"],
  ["DOM form", "form"],
  ["DOM id", "element id"],
  ["DOM", ""],
  ["HTML", ""],
  ["Additional CSS classes.", ""],
  ["Additional CSS classes", ""],
  ["CSS classes", "styles"],
  ["className", "style"],
  ["PressEvent", "ClickEvent"],
  ["pointer-events", "hit testing"],
  ["this port shows", "this example shows"],
  ["The port ", ""],
  ["the port ", ""],
  ["v3 ", ""],
];

function replaceWord(text: string, from: string, to: string): string {
  const pattern = new RegExp(`(?<![A-Za-z0-9_])${escapeRegExp(from)}(?![A-Za-z0-9_])`, "g");
  return text.replace(pattern, to);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function tidy(text: string): string {
  let out = text;
  out = out.replace(/\b([A-Za-z][\w:]*)\b rather than \1\b/g, "$1");
  while (out.includes("  ")) out = out.replaceAll("  ", " ");
  while (out.includes("the the")) out = out.replaceAll("the the", "the");
  out = out.replace(/\s+([.,;:])/g, "$1");
  out = out.replace(/,\s*\./g, ".");
  out = out.replace(/\.\s*\./g, ".");
  return out.trim();
}

export function scrubProse(text: string): string {
  let out = text;
  for (const [from, to] of SCRUB_PHRASES) {
    out =
      /[A-Za-z]/.test(from) && /^[A-Za-z0-9]+$/.test(from)
        ? replaceWord(out, from, to)
        : out.replaceAll(from, to);
  }
  return tidy(out);
}

export function scrubDescription(text: string): string {
  return scrubProse(text).replace(/(^|[.!?]\s+)([a-z])/g, (_, lead: string, letter: string) => {
    return `${lead}${letter.toUpperCase()}`;
  });
}

export function rustValueType(ty: string): string {
  let out = ty.replace("keyof React.JSX.IntrinsicElements, ", "").replaceAll("React.", "");
  for (const [from, to] of TYPE_PHRASES) {
    out = out.replaceAll(from, to);
  }
  out = rustClosures(out).replaceAll("void", "()");
  return scrubProse(out);
}

function rustClosures(text: string): string {
  let out = "";
  let rest = text;
  while (true) {
    const arrow = rest.indexOf("=>");
    if (arrow < 0) {
      out += rest;
      break;
    }
    const before = rest.slice(0, arrow);
    const open = before.lastIndexOf("(");
    if (open < 0) {
      out += rest.slice(0, arrow + 2);
      rest = rest.slice(arrow + 2);
      continue;
    }
    const after = rest.slice(arrow + 2);
    const close = after.indexOf(")");
    const end = close < 0 ? after.length : close;
    let params = before.slice(open + 1).trim();
    if (params.endsWith(")")) params = params.slice(0, -1);
    out += before.slice(0, open);
    out += `Fn(${params}) -> ${after.slice(0, end).trim()}`;
    rest = after.slice(end);
  }
  return out;
}

function isBlank(value: string | null | undefined): boolean {
  return !value || PLACEHOLDER.test(value.trim());
}

export function isCallableRust(rust: string | null | undefined, status?: string): boolean {
  if (status === "unavailable") return false;
  return !isBlank(rust);
}

function pascalIdent(value: string): string {
  return value
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function builderType(rust: string, fallbackType: string): string {
  const pieces = rust.split(" / ").map((piece) => piece.trim());
  const types = pieces.map((piece) => {
    const match = piece.match(/^[\w:]+(?:<[^>]+>)?\((.+)\)$/);
    if (!match) return rustValueType(fallbackType);
    const inner = match[1].trim();
    if (
      inner === "callback" ||
      inner === "value" ||
      inner === "render" ||
      inner === "element" ||
      inner === "name" ||
      inner === "keys" ||
      inner === "indices"
    ) {
      return rustValueType(fallbackType);
    }
    return inner;
  });
  return [...new Set(types)].join(" / ");
}

function quotedLiterals(ty: string): string[] | null {
  const parts = ty.split("|").map((part) => part.trim());
  if (parts.length === 0) return null;
  const literals: string[] = [];
  for (const part of parts) {
    const match = part.match(/^['"]([^'"]+)['"]$/);
    if (!match) return null;
    literals.push(match[1]);
  }
  return literals;
}

/** The values a styling builder accepts, in Rust spelling. */
export function acceptedStyleValues(apiType: string, rustType: string): string {
  const trimmed = apiType.trim();
  if (/^boolean$/i.test(trimmed) || rustType === "bool") {
    return "true | false";
  }
  const literals = quotedLiterals(trimmed);
  if (literals) {
    return literals.map((literal) => rustDefault(`'${literal}'`, rustType)).join(" | ");
  }
  return rustValueType(trimmed);
}

export function rustDefault(raw: string | null | undefined, rustType: string): string {
  if (isBlank(raw) || !raw) return "—";
  const cleaned = raw.trim().replace(/^['"]|['"]$/g, "");
  if (cleaned === "true" || cleaned === "false") return cleaned;
  if (cleaned === "null") return "None";
  const typeName =
    rustType
      .split(" / ")[0]
      ?.replace(/<[^>]+>/g, "")
      .trim() ?? "";
  if (/^[A-Z][A-Za-z0-9]+$/.test(typeName) && /^[a-z]/.test(cleaned)) {
    return `${typeName}::${pascalIdent(cleaned)}`;
  }
  return rustValueType(raw);
}

export function uniqueBy<T>(rows: T[], key: (row: T) => string): T[] {
  const seen = new Set<string>();
  return rows.filter((row) => {
    const id = key(row);
    if (!id || seen.has(id)) return false;
    seen.add(id);
    return true;
  });
}

export interface GpuiPropRow {
  builder: string;
  type: string;
  default: string;
  description: string;
}

export function gpuiPropRows(rows: ApiRow[]): GpuiPropRow[] {
  return uniqueBy(
    rows
      .filter((row) => isCallableRust(row.rust, row.status))
      .map((row) => {
        const builder = row.rust!.trim();
        const type = builderType(builder, row.type);
        return {
          builder,
          type,
          default: rustDefault(row.default, type),
          description: scrubDescription(row.description),
        };
      })
      .filter((row) => row.description.length > 0 || row.builder.length > 0),
    (row) => row.builder,
  );
}

export interface GpuiPartRow {
  part: string;
  description: string;
}

export function gpuiPartRows(rows: PartRow[]): GpuiPartRow[] {
  return uniqueBy(
    rows
      .filter((row) => row.status !== "unavailable" && !isBlank(row.rustOwner))
      .map((row) => ({
        part: row.rustOwner!.trim(),
        description: scrubDescription(row.description),
      })),
    (row) => `${row.part}\0${row.description}`,
  );
}

export interface GpuiStateRow {
  state: string;
  builder: string;
  description: string;
}

function publicStateBuilder(rust: string | null | undefined): string {
  if (isBlank(rust) || !rust) return "—";
  const parts = rust
    .split(" / ")[0]
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  const preferred = parts.find(
    (part) => /^is_[a-z_]+(?:\(|$)/.test(part) || /^InteractiveState::is_/.test(part),
  );
  if (preferred) {
    return preferred.replace(/^InteractiveState::/, "");
  }
  const kept = parts.filter(
    (part) => !part.startsWith("anim::") && part !== "disabled_opacity" && !part.includes("="),
  );
  return kept[0] ?? parts[0] ?? "—";
}

export function gpuiStateRows(rows: StateRow[]): GpuiStateRow[] {
  return uniqueBy(
    rows
      .filter((row) => row.status !== "unavailable")
      .map((row) => ({
        state: row.state,
        builder: publicStateBuilder(row.rust),
        description: scrubDescription(row.description),
      })),
    (row) => row.state.toLowerCase(),
  );
}

export interface GpuiStyleRow {
  style: string;
  type: string;
  description: string;
}

/** Builders that change how a component looks, not how it behaves. */
const APPEARANCE_BUILDERS = new Set([
  "variant",
  "size",
  "color",
  "radius",
  "full_width",
  "is_icon_only",
  "orientation",
  "placement",
  "shadow",
  "backdrop",
  "shape",
]);

function builderName(builder: string): string {
  const call = builder.split(" / ")[0]?.trim() ?? builder;
  const name = call.replace(/\(.*$/, "").trim();
  const last = name.split("::").pop() ?? name;
  return last;
}

/**
 * Appearance builders the component actually exposes. CSS-port internals
 * (`flex + items_center + gap(8)`) are not listed; those are how the port is
 * drawn, not methods a caller can set.
 */
export function gpuiStyleRows(api: ApiRow[]): GpuiStyleRow[] {
  return uniqueBy(
    api
      .filter((row) => isCallableRust(row.rust, row.status))
      .filter((row) => APPEARANCE_BUILDERS.has(builderName(row.rust!.trim())))
      .map((row) => {
        const builder = row.rust!.trim();
        const rustType = builderType(builder, row.type);
        return {
          style: builder,
          type: acceptedStyleValues(row.type, rustType),
          description: scrubDescription(row.description),
        };
      }),
    (row) => row.style,
  );
}

export function rustRequiredParts(requiredParts: string[], parts: PartRow[]): string[] {
  const names = requiredParts
    .map((name) => {
      const match = parts.find((part) => part.name === name);
      return match && !isBlank(match.rustOwner) && match.status !== "unavailable"
        ? match.rustOwner!.trim()
        : null;
    })
    .filter((name): name is string => Boolean(name));
  return [...new Set(names)];
}
