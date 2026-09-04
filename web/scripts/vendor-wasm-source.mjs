// Vendor the WASM migration source into this repository.
//
//   node scripts/vendor-wasm-source.mjs [--wasm-root D:/herogpui-wasm] [--check]
//
// The shipped gallery artifact is compiled from a separate, older wasm32
// checkout of this repository. Committing only the artifact records a result
// whose source cannot be reviewed or rebuilt from here, so every artifact
// commit also carries that checkout's baseline commit and its working diff.
//
// `--check` verifies the vendored copy still matches the live checkout and the
// shipped artifact instead of rewriting it.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const outDir = join(repoRoot, "web", "wasm-migration");
const patchPath = join(outDir, "wasm-migration.patch");
const metaPath = join(outDir, "source.json");
const parityPath = join(repoRoot, "web", "src", "data", "wasm-parity.json");

function flag(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index === -1) return fallback;
  const value = process.argv[index + 1];
  return !value || value.startsWith("--") ? true : value;
}

const wasmRoot = flag("--wasm-root", "D:/herogpui-wasm");
const check = flag("--check", false) === true;

if (!existsSync(join(wasmRoot, ".git"))) {
  throw new Error(`${wasmRoot} is not a git checkout; pass --wasm-root`);
}

const git = (...args) =>
  execFileSync("git", ["-C", wasmRoot, ...args], { encoding: "utf8", maxBuffer: 1 << 28 });

const baselineCommit = git("rev-parse", "HEAD").trim();
// Line endings differ between the two checkouts; a literal diff keeps the
// patch appliable and its hash stable across machines.
const patch = git("-c", "core.autocrlf=false", "diff");

const sha = (buffer) => createHash("sha256").update(buffer).digest("hex");
const parity = JSON.parse(readFileSync(parityPath, "utf8"));
const meta = {
  baselineCommit,
  patch: "wasm-migration.patch",
  patchSha256: sha(Buffer.from(patch, "utf8")),
  artifactSha256: parity.artifactSha256,
  glueSha256: parity.glueSha256,
};

if (check) {
  const failures = [];
  if (!existsSync(patchPath) || !existsSync(metaPath)) {
    failures.push("web/wasm-migration is missing; run vendor-wasm-source.mjs");
  } else {
    const vendored = JSON.parse(readFileSync(metaPath, "utf8"));
    const onDisk = sha(readFileSync(patchPath));
    if (vendored.baselineCommit !== baselineCommit) {
      failures.push(`baseline ${vendored.baselineCommit} != checkout ${baselineCommit}`);
    }
    if (vendored.patchSha256 !== meta.patchSha256) {
      failures.push("vendored patch is stale against the live migration checkout");
    }
    if (onDisk !== vendored.patchSha256) {
      failures.push("wasm-migration.patch does not match its recorded hash");
    }
    for (const key of ["artifactSha256", "glueSha256"]) {
      if (vendored[key] !== meta[key]) failures.push(`${key} does not match wasm-parity.json`);
    }
  }
  if (failures.length) {
    for (const failure of failures) console.error(`FAIL ${failure}`);
    process.exit(1);
  }
  console.log(`vendored wasm source matches ${baselineCommit.slice(0, 8)} + patch`);
} else {
  mkdirSync(outDir, { recursive: true });
  writeFileSync(patchPath, patch);
  writeFileSync(metaPath, `${JSON.stringify(meta, null, 2)}\n`);
  const lines = patch.split("\n").length;
  console.log(`vendored ${baselineCommit.slice(0, 8)} + ${lines} patch lines`);
}
