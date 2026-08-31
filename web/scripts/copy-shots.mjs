// Copy every gallery screenshot (.shots/*.png) into web/public/shots/ so the
// site can serve them at /shots/<name>. Filenames are preserved verbatim —
// catalog.json's `shot`/`shotDark` fields reference them by exact name (see
// scripts/extract-catalog.mjs). Offline step; part of build-data.
//
//   node scripts/copy-shots.mjs

import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const SRC = resolve(repoRoot, ".shots");
const DEST = resolve(webRoot, "public", "shots");

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function run() {
  if (!existsSync(SRC)) {
    throw new Error(`gallery screenshots not found at ${SRC}`);
  }
  mkdirSync(DEST, { recursive: true });
  // Stale copies would advertise shots the gallery no longer produces; the
  // directory is generated output, so .png files are refreshed wholesale.
  for (const entry of readdirSync(DEST)) {
    if (entry.endsWith(".png")) rmSync(join(DEST, entry));
  }

  const files = readdirSync(SRC)
    // The capture scripts prefix their scratch files with `~` (`~tmp.png`
    // and friends); the root .gitignore excludes them from .shots, and the
    // catalog never references them.
    .filter((f) => f.endsWith(".png") && !f.startsWith("~"))
    .sort();
  let bytes = 0;
  for (const file of files) {
    copyFileSync(join(SRC, file), join(DEST, file));
    bytes += statSync(join(SRC, file)).size;
  }

  console.log(`copy-shots: ${files.length} screenshots, ${formatBytes(bytes)} -> public/shots`);
  return { count: files.length, bytes };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  run();
}
