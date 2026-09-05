// Sync the vendored @porabuild/brand copy into web/src/styles/porabuild/.
//
// The HeroGPUI site is built on Vercel from its own tree, so it cannot depend
// on the sibling porabuild repo; it vendors a copy of
// porabuild/packages/brand instead. Never hand-edit the vendored files.
//
// Usage:
//   node scripts/sync-porabuild-brand.mjs [--from <dir>] [--check]
//   pnpm run brand:sync   copies source -> web/src/styles/porabuild/
//   pnpm run brand:check  exits 1 when the vendored copy drifts from source
//
// Source dir resolution: --from <dir>, then $PORABUILD_BRAND_DIR, then
// ../porabuild/packages/brand relative to the HeroGPUI repo root
// (worktree-aware: unwraps .poracode/worktrees/<name> to the real root).

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve, basename } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
let repoRoot = resolve(webRoot, "..");
// Poracode worktrees live under .poracode/worktrees/<name>/; unwrap to the real repo root.
if (
  basename(dirname(repoRoot)) === "worktrees" &&
  basename(dirname(dirname(repoRoot))) === ".poracode"
) {
  repoRoot = dirname(dirname(dirname(repoRoot)));
}
const DEST = resolve(webRoot, "src", "styles", "porabuild");

const PACKAGE_NAME = "@porabuild/brand";
const FILES = ["tokens.css", "brand.css", "index.css", "README.md"];
const CSS_FILES = new Set(["tokens.css", "brand.css", "index.css"]);

function resolveSource(argv) {
  const flagIndex = argv.indexOf("--from");
  if (flagIndex !== -1) {
    const value = argv[flagIndex + 1] ?? "";
    if (!value || value.startsWith("--")) {
      throw new Error("missing directory after --from (usage: --from <dir>)");
    }
    return { dir: resolve(process.cwd(), value), origin: `--from ${value}` };
  }
  const env = process.env.PORABUILD_BRAND_DIR;
  if (env) {
    return { dir: resolve(process.cwd(), env), origin: "$PORABUILD_BRAND_DIR" };
  }
  return {
    dir: resolve(repoRoot, "..", "porabuild", "packages", "brand"),
    origin: "default (../porabuild/packages/brand relative to the repo root)",
  };
}

function readVersion(sourceDir) {
  const pkgPath = resolve(sourceDir, "package.json");
  const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
  if (pkg.name !== PACKAGE_NAME) {
    throw new Error(`expected ${PACKAGE_NAME} at ${pkgPath}, found ${pkg.name ?? "unknown"}`);
  }
  if (!pkg.version) {
    throw new Error(`package.json at ${pkgPath} has no version`);
  }
  return pkg.version;
}

function headerLine(version) {
  return `/* Vendored from ${PACKAGE_NAME} ${version}. Do not edit; run pnpm run brand:sync. */`;
}

function expectedContent(name, sourceDir, version) {
  const raw = readFileSync(resolve(sourceDir, name), "utf8");
  if (!CSS_FILES.has(name)) {
    return raw;
  }
  return `${headerLine(version)}\n${raw}`;
}

// Few-line excerpt of the first divergence, for --check output.
function summarizeDiff(expected, actual, maxLines = 12) {
  const a = expected.split("\n");
  const b = actual.split("\n");
  const out = [];
  const len = Math.max(a.length, b.length);
  let shown = 0;
  for (let i = 0; i < len && shown < maxLines; i++) {
    if (a[i] === b[i]) {
      continue;
    }
    if (a[i] !== undefined) {
      out.push(`  expected L${i + 1}: ${a[i]}`);
      shown++;
    }
    if (b[i] !== undefined && shown < maxLines) {
      out.push(`  vendored L${i + 1}: ${b[i]}`);
      shown++;
    }
  }
  if (shown >= maxLines) {
    out.push("  ... (truncated)");
  }
  return out.join("\n");
}

export function run({ check = false, sourceArgv = process.argv.slice(2) } = {}) {
  const { dir: sourceDir, origin } = resolveSource(sourceArgv);

  if (!existsSync(sourceDir)) {
    if (check) {
      console.log(
        `brand:check: source dir not found at ${sourceDir} (${origin}); ` +
          "skipping so CI without the sibling repo still passes.",
      );
      return { status: "skipped", sourceDir };
    }
    console.error(
      `ERROR: brand source dir not found at ${sourceDir} (${origin}). ` +
        "Pass --from <dir> or set $PORABUILD_BRAND_DIR.",
    );
    process.exitCode = 1;
    return { status: "missing-source", sourceDir };
  }

  let version;
  try {
    version = readVersion(sourceDir);
  } catch (error) {
    console.error(`ERROR: cannot read brand package version: ${error.message}`);
    process.exitCode = 1;
    return { status: "bad-source", sourceDir };
  }

  const drifts = [];
  for (const name of FILES) {
    let expected;
    try {
      expected = expectedContent(name, sourceDir, version);
    } catch (error) {
      console.error(`ERROR: cannot read ${name} from ${sourceDir}: ${error.message}`);
      process.exitCode = 1;
      return { status: "bad-source", sourceDir };
    }
    let current = null;
    try {
      current = readFileSync(resolve(DEST, name), "utf8");
    } catch {
      // Missing vendored file counts as drift in --check mode.
    }
    if (current === null) {
      drifts.push({
        name,
        reason: "missing",
        diff: "  (file absent from web/src/styles/porabuild/)",
      });
    } else if (current !== expected) {
      drifts.push({ name, reason: "differs", diff: summarizeDiff(expected, current) });
    }
  }

  if (check) {
    if (drifts.length > 0) {
      console.error(
        `ERROR: vendored ${PACKAGE_NAME} copy differs from ${sourceDir} ` +
          `(${drifts.length} file(s)):`,
      );
      for (const { name, reason, diff } of drifts) {
        console.error(`- ${name} (${reason})\n${diff}`);
      }
      console.error("Run `pnpm run brand:sync` to refresh the vendored copy.");
      process.exitCode = 1;
      return { status: "drift", sourceDir, drifts };
    }
    console.log(
      `brand:check: vendored copy matches ${PACKAGE_NAME} ${version} (${FILES.length} files ok).`,
    );
    return { status: "ok", sourceDir };
  }

  mkdirSync(DEST, { recursive: true });
  for (const name of FILES) {
    writeFileSync(resolve(DEST, name), expectedContent(name, sourceDir, version));
  }
  console.log(
    `brand:sync: vendored ${PACKAGE_NAME} ${version} from ${sourceDir} ` +
      `-> ${DEST} (${FILES.length} files).`,
  );
  return { status: "synced", sourceDir };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run({ check: process.argv.includes("--check") });
}
