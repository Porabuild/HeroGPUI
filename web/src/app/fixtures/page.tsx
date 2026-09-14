import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { notFound } from "next/navigation";
import { FixtureStage } from "./registry";

/**
 * Development-only fixture runner for the parity workflow
 * (docs/parity/fixtures.md, plan section 5.1). Renders one pinned-HeroUI
 * reference composition at an exact stage size with an evidence strip that
 * pins the inputs a capture must record. The route is deliberately not linked
 * anywhere; outside development it does not exist at all.
 */

/**
 * The fixture compositions are client components, but the evidence strip and
 * the request validation run on the server and cannot read across the client
 * boundary, so the ids and stage overrides here mirror `fixtures` in
 * registry.tsx. Keep the two lists in step when adding a fixture.
 */
interface FixtureMeta {
  id: string;
  label: string;
  stageWidth?: number;
  stageHeight?: number;
}

const FIXTURES: FixtureMeta[] = [
  { id: "button-usage", label: "Button Usage" },
  { id: "select-usage", label: "Select Usage" },
  {
    id: "modal-open",
    label: "Modal Sizes — Md, open",
    stageWidth: 720,
    stageHeight: 480,
  },
];

const DEFAULT_FIXTURE_ID = FIXTURES[0].id;
const DEFAULT_WIDTH = 640;
const DEFAULT_HEIGHT = 360;
const MIN_WIDTH = 200;
const MAX_WIDTH = 1600;
const MIN_HEIGHT = 120;
const MAX_HEIGHT = 1200;
const THEMES = ["light", "dark"] as const;

type Theme = (typeof THEMES)[number];

interface FixturesPageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}

/**
 * The strip and the stage must be rebuilt per request: the provenance hash has
 * to track the current lockfile, and a fixture route that depends on
 * `searchParams` must never be cached or prerendered.
 */
export const dynamic = "force-dynamic";

/**
 * Collapses a searchParams entry to its first value so `?w=1&w=2` cannot
 * produce different server and strip sizes.
 */
function firstParam(
  params: { [key: string]: string | string[] | undefined },
  key: string,
): string | undefined {
  const value = params[key];
  return Array.isArray(value) ? value[0] : value;
}

/**
 * Parses one stage dimension. Absent or empty falls back to the default;
 * unparseable input is a reported error (and falls back); out-of-range numbers
 * clamp — a 2000px request still gets a bounded, deterministic stage rather
 * than a failed capture.
 */
function parseStageSize(
  raw: string | undefined,
  name: string,
  fallback: number,
  min: number,
  max: number,
  errors: string[],
): number {
  if (raw === undefined || raw === "") {
    return fallback;
  }
  const value = Number(raw);
  if (!Number.isFinite(value)) {
    errors.push(`${name} must be a number of CSS pixels between ${min} and ${max} (got "${raw}")`);
    return fallback;
  }
  return Math.min(max, Math.max(min, value));
}

/**
 * The @heroui/react version the site actually resolves, read from
 * package.json at request time so the strip cannot drift from the install.
 */
async function readHerouiVersion(): Promise<string> {
  try {
    const raw = await readFile(path.join(process.cwd(), "package.json"), "utf8");
    const pkg = JSON.parse(raw) as { dependencies?: Record<string, string> };
    return pkg.dependencies?.["@heroui/react"] ?? "unavailable";
  } catch {
    // The strip records provenance; a missing file is itself evidence.
    return "unavailable";
  }
}

/**
 * SHA-256 of the resolved lockfile, truncated to 16 hex chars. Two captures
 * whose strips disagree hashed against different dependency trees, which is
 * exactly the comparison the parity plan forbids.
 */
async function readLockfileHash(): Promise<string> {
  try {
    const raw = await readFile(path.join(process.cwd(), "pnpm-lock.yaml"));
    return createHash("sha256").update(raw).digest("hex").slice(0, 16);
  } catch {
    return "unavailable";
  }
}

/**
 * Inline error panel for bad fixture/theme/size requests. Still dev-only —
 * it is reachable only past the NODE_ENV gate — and it always lists the valid
 * values so a typo is fixed in one round trip.
 */
function InvalidRequest({ errors }: { errors: string[] }) {
  return (
    <div className="mx-auto max-w-xl px-4 py-16">
      <h1 className="text-lg font-semibold">Invalid fixture request</h1>
      <ul className="mt-4 list-disc space-y-1 pl-5 text-sm text-muted">
        {errors.map((error, index) => (
          <li key={index}>{error}</li>
        ))}
      </ul>
      <p className="mt-4 text-sm">
        Valid fixtures: {FIXTURES.map((entry) => entry.id).join(", ")}. Valid themes:{" "}
        {THEMES.join(", ")}. Stage sizes: {MIN_WIDTH}–{MAX_WIDTH} for w, {MIN_HEIGHT}–{MAX_HEIGHT}{" "}
        for h.
      </p>
    </div>
  );
}

export default async function FixturesPage({ searchParams }: FixturesPageProps) {
  if (process.env.NODE_ENV !== "development") {
    notFound();
  }

  const params = await searchParams;
  const errors: string[] = [];

  const fixtureParam = firstParam(params, "fixture");
  const fixtureId = fixtureParam ?? DEFAULT_FIXTURE_ID;
  const fixture = FIXTURES.find((entry) => entry.id === fixtureId);
  if (!fixture) {
    errors.push(
      `Unknown fixture "${fixtureParam ?? ""}". Valid fixtures: ${FIXTURES.map(
        (entry) => entry.id,
      ).join(", ")}.`,
    );
  }

  let theme: Theme = "light";
  const themeParam = firstParam(params, "theme");
  if (themeParam !== undefined && themeParam !== "") {
    if (themeParam === "light" || themeParam === "dark") {
      theme = themeParam;
    } else {
      errors.push(`Unknown theme "${themeParam}". Valid themes: ${THEMES.join(", ")}.`);
    }
  }

  // Explicit w/h wins; a fixture's own stage size is its default so the open
  // Modal gets room without a longer URL.
  const width = parseStageSize(
    firstParam(params, "w"),
    "w",
    fixture?.stageWidth ?? DEFAULT_WIDTH,
    MIN_WIDTH,
    MAX_WIDTH,
    errors,
  );
  const height = parseStageSize(
    firstParam(params, "h"),
    "h",
    fixture?.stageHeight ?? DEFAULT_HEIGHT,
    MIN_HEIGHT,
    MAX_HEIGHT,
    errors,
  );

  const [herouiVersion, lockfileHash] = await Promise.all([
    readHerouiVersion(),
    readLockfileHash(),
  ]);

  if (errors.length > 0) {
    return <InvalidRequest errors={errors} />;
  }

  // Past validation the id is one of FIXTURES; the fallback only satisfies the
  // type checker.
  const resolvedFixture = fixture ?? FIXTURES[0];

  // The root layout's theme-init script sets `dark` on documentElement from
  // localStorage or the system preference, which would otherwise leak that
  // browser profile's token set into every capture. The theme class is
  // declared here on both the page wrapper and the stage wrapper so the token
  // set is re-declared inside the subtree and the capture depends only on the
  // `theme` query parameter.
  const themeClass = theme === "dark" ? "dark" : "light";

  return (
    // bg-background/text-foreground resolve through the wrapper's own theme
    // class, so the whole page (not just the stage) follows the `theme`
    // parameter — the body's documentElement-scoped background would
    // otherwise show through around the stage in full-viewport captures.
    <div className={`${themeClass} min-h-dvh bg-background text-foreground`}>
      <div
        data-fixtures-strip
        className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-separator bg-surface px-3 py-2 font-mono text-xs text-muted"
      >
        {/* Crop below this element; everything under it is the fixture stage. */}
        <span>fixture={resolvedFixture.id}</span>
        <span>theme={theme}</span>
        <span>
          stage={width}x{height}
        </span>
        <span>@heroui/react@{herouiVersion}</span>
        <span>pnpm-lock.yaml sha256:{lockfileHash}</span>
      </div>
      <div
        data-fixtures-stage
        className={themeClass}
        style={{
          width,
          height,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          // Explicit background so the cropped stage rect is deterministic
          // even before stylesheet tokens load: the fallbacks are the site's
          // own resolved light/dark --background values (porabuild moon
          // #eaf0fb / night #070709, see the conversion table in globals.css).
          backgroundColor:
            theme === "dark" ? "var(--background, #070709)" : "var(--background, #eaf0fb)",
          // @heroui/styles defines --font-sans (Tailwind's default sans stack)
          // in its theme layer, and that token is what body text on this site
          // already resolves to. Pin it explicitly with the same stack as the
          // fallback so text metrics cannot vary with the host font config.
          fontFamily: "var(--font-sans, ui-sans-serif, system-ui, sans-serif)",
        }}
      >
        {/* Centered on the stage exactly like the gallery's preview_wrapper
            (flex column, both axes centered) so upstream and port compositions
            share one framing. */}
        <FixtureStage id={resolvedFixture.id} />
      </div>
    </div>
  );
}
