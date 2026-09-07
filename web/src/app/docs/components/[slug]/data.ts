import { readFileSync } from "node:fs";
import path from "node:path";

import type { ApiRow } from "@/components/ui/props-table";
import type { PortStatus } from "@/components/ui/status-chip";

/**
 * Typed readers for the data pipeline's `src/data/reference.json` and
 * `src/data/rust-examples.json`. Like `@/lib/catalog`, the loaders parse
 * defensively and fall back to empty data so the site builds even while the
 * pipeline is regenerating; `@/lib/catalog` stays the only catalog source.
 */

export interface PartRow {
  /** Compound part as upstream spells it, e.g. `Switch.Control`. */
  name: string;
  /** Upstream data-slot value, e.g. `switch-control`. */
  slot: string;
  description: string;
  /** Rust type or builder owning the part, when ported. */
  rustOwner: string | null;
  status: PortStatus;
}

export interface StateRow {
  /** Human state name, e.g. `Selected`. */
  state: string;
  /** Upstream CSS selector for the state. */
  selector: string;
  description: string;
  /** Rust style or state hook, when ported. */
  rust: string | null;
  status: PortStatus;
}

export interface StylingRow {
  /** Upstream token or class, e.g. `.switch__content`. */
  token: string;
  description: string;
  /** Rust builder equivalent, when ported. */
  rust: string | null;
  status: PortStatus;
}

export interface RustExample {
  heading: string;
  description?: string;
  code: string;
  imports?: string;
}

export interface ComponentReference {
  /** Upstream page title, e.g. `FieldSlots` for the field-parts merge. */
  page: string;
  importLine: string;
  /** Pinned upstream version the audit was measured against. */
  version: string;
  docsSource: string | null;
  apiSource: string | null;
  styleSource: string | null;
  requiredParts: string[];
  api: ApiRow[];
  parts: PartRow[];
  states: StateRow[];
  styling: StylingRow[];
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asStringOrNull(value: unknown): string | null {
  return typeof value === "string" ? value : null;
}

function asStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === "string")
    : [];
}

function asStatus(value: unknown): PortStatus {
  return value === "implemented" || value === "unavailable" ? value : "partial";
}

function asApiRow(value: unknown): ApiRow | null {
  if (!isRecord(value)) return null;
  return {
    owner: asString(value.owner),
    prop: asString(value.prop),
    type: asString(value.type),
    default: asStringOrNull(value.default),
    description: asString(value.description),
    rust: asStringOrNull(value.rust),
    status: asStatus(value.status),
  };
}

function asRows<T>(value: unknown, parse: (row: unknown) => T | null): T[] {
  return Array.isArray(value)
    ? value.flatMap((row) => {
        const parsed = parse(row);
        return parsed ? [parsed] : [];
      })
    : [];
}

function parseReference(raw: unknown): ComponentReference | null {
  if (!isRecord(raw)) return null;
  return {
    page: asString(raw.page),
    importLine: asString(raw.importLine),
    version: asString(raw.version),
    docsSource: asStringOrNull(raw.docsSource),
    apiSource: asStringOrNull(raw.apiSource),
    styleSource: asStringOrNull(raw.styleSource),
    requiredParts: asStringArray(raw.requiredParts),
    api: asRows(raw.api, asApiRow),
    parts: asRows<PartRow>(raw.parts, (row) =>
      isRecord(row)
        ? {
            name: asString(row.name),
            slot: asString(row.slot),
            description: asString(row.description),
            rustOwner: asStringOrNull(row.rustOwner),
            status: asStatus(row.status),
          }
        : null,
    ),
    states: asRows<StateRow>(raw.states, (row) =>
      isRecord(row)
        ? {
            state: asString(row.state),
            selector: asString(row.selector),
            description: asString(row.description),
            rust: asStringOrNull(row.rust),
            status: asStatus(row.status),
          }
        : null,
    ),
    styling: asRows<StylingRow>(raw.styling, (row) =>
      isRecord(row)
        ? {
            token: asString(row.token),
            description: asString(row.description),
            rust: asStringOrNull(row.rust),
            status: asStatus(row.status),
          }
        : null,
    ),
  };
}

function persistDataCache(): boolean {
  return process.env.NODE_ENV === "production";
}

function readJsonRecord(name: string): Record<string, unknown> {
  try {
    const file = path.join(process.cwd(), "src", "data", name);
    const parsed: unknown = JSON.parse(readFileSync(file, "utf8"));
    return isRecord(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

let referenceFile: Record<string, unknown> | null = null;

function referenceData(): Record<string, unknown> {
  if (!persistDataCache()) return readJsonRecord("reference.json");
  referenceFile ??= readJsonRecord("reference.json");
  return referenceFile;
}

const referenceCache = new Map<string, ComponentReference | null>();

/** The upstream/port audit entry for one component, or null when absent. */
export function getComponentReference(slug: string): ComponentReference | null {
  if (!persistDataCache()) {
    return parseReference(referenceData()[slug]);
  }
  if (!referenceCache.has(slug)) {
    referenceCache.set(slug, parseReference(referenceData()[slug]));
  }
  return referenceCache.get(slug) ?? null;
}

let examplesFile: Record<string, unknown> | null = null;
let wasmSectionsFile: Record<string, unknown> | null = null;

function exampleData(): Record<string, unknown> {
  if (!persistDataCache()) return readJsonRecord("rust-examples.json");
  examplesFile ??= readJsonRecord("rust-examples.json");
  return examplesFile;
}

/** The HeroGPUI gallery snippets for one component, in repo order. */
export function getRustExamples(slug: string): RustExample[] {
  const cached = exampleData()[slug];
  return asRows<RustExample>(cached, (row) =>
    isRecord(row)
      ? {
          heading: asString(row.heading),
          description: typeof row.description === "string" ? row.description : undefined,
          code: asString(row.code),
          imports: typeof row.imports === "string" ? row.imports : undefined,
        }
      : null,
  );
}

/** Example headings the compiled web-gallery artifact can render live. */
export function getWasmSections(slug: string): string[] {
  if (!persistDataCache()) {
    return asStringArray(readJsonRecord("wasm-sections.json")[slug]);
  }
  wasmSectionsFile ??= readJsonRecord("wasm-sections.json");
  return asStringArray(wasmSectionsFile[slug]);
}

let wasmArtifactVersionFile: string | undefined;

/**
 * The pinned artifact hash, used as the embed's cache-busting version: the
 * browser caches the gallery module and WASM across visits, so a deploy that
 * only swaps those files would otherwise leave visitors on the previous
 * build.
 */
export function getWasmArtifactVersion(): string {
  const read = () => {
    const parsed = readJsonRecord("wasm-parity.json");
    return typeof parsed.artifactSha256 === "string" ? parsed.artifactSha256 : "";
  };
  if (!persistDataCache()) return read();
  wasmArtifactVersionFile ??= read();
  return wasmArtifactVersionFile;
}
