// The web-gallery artifact (`herogpui_web_bg.wasm` + `herogpui_web.js`) is not
// committed. CI builds it and publishes it to this repository's
// `gallery-artifacts` GitHub prerelease, keyed by the hash of every wasm build
// input (`inputsHash`, the same `inputsSha256` `src/data/wasm-parity.json`
// records); the site build downloads the one its own checkout keys to and
// verifies it. See web/DEPLOYMENT.md, "The live WebAssembly gallery".
//
//   node scripts/gallery-artifact.mjs key
//       Print this checkout's artifact key (the wasm build-input hash).
//   node scripts/gallery-artifact.mjs stamp --dir <dir> --source ci|local
//         [--toolchain <rustc -V>] [--wasm-bindgen <v>] [--wasm-opt <v>]
//       Write <dir>/build-info.json for a freshly built artifact
//       (.shots/build-wasm.sh calls this).
//   node scripts/gallery-artifact.mjs fetch
//       Make public/gallery/ hold an artifact for this checkout (the first
//       step of `pnpm run build`). A local build there always wins.
//
// Environment (all optional):
//   HEROGPUI_GALLERY_ARTIFACT       auto (default) | require | off
//   HEROGPUI_GALLERY_WAIT_SECONDS   how long to wait for CI to publish the
//                                   exact artifact (default: 1800 on a Vercel
//                                   production build, 600 on a Vercel preview,
//                                   0 elsewhere)
//   HEROGPUI_GALLERY_REPO           owner/name holding the release
//   HEROGPUI_GALLERY_BASE_URL       download base, overriding the release URL

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { inputsHash } from "./extract-wasm-sections.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const repoRoot = resolve(webRoot, "..");
const galleryDir = resolve(webRoot, "public", "gallery");

export const RELEASE_TAG = "gallery-artifacts";
export const BUILD_INFO_SCHEMA = 1;
export const WASM_FILE = "herogpui_web_bg.wasm";
export const GLUE_FILE = "herogpui_web.js";
export const INFO_FILE = "build-info.json";
/** The pointer CI replaces on every master push: the preview fallback. */
export const MASTER_POINTER = "herogpui-gallery-master.json";
const DEFAULT_REPO = "Porabuild/HeroGPUI";
const POLL_INTERVAL_MS = 20_000;
const HEX64 = /^[0-9a-f]{64}$/;

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** The release asset holding the build-info of the artifact for `key`. */
export function keyAssetName(key) {
  return `herogpui-gallery-${key.slice(0, 16)}.json`;
}

/** Content-addressed binary names, so two builds racing for one key cannot mix files. */
export function binaryAssetNames(artifactSha256, glueSha256) {
  return {
    wasm: `herogpui-gallery-${artifactSha256.slice(0, 16)}.wasm`,
    glue: `herogpui-gallery-${glueSha256.slice(0, 16)}.js`,
  };
}

function argument(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index === -1) return fallback;
  const value = process.argv[index + 1];
  if (value === undefined) throw new Error(`${name} requires a value`);
  return value;
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

/** The build-info record for the artifact in `dir` (see `stamp`). */
export function describeBuild(dir, key, extra = {}) {
  const artifact = readFileSync(resolve(dir, WASM_FILE));
  const glue = readFileSync(resolve(dir, GLUE_FILE));
  const artifactSha256 = sha256(artifact);
  const glueSha256 = sha256(glue);
  return {
    schema: BUILD_INFO_SCHEMA,
    inputsSha256: key,
    artifactSha256,
    artifactBytes: artifact.length,
    glueSha256,
    glueBytes: glue.length,
    ...binaryAssetNames(artifactSha256, glueSha256),
    ...extra,
  };
}

/**
 * Whether `info` is a well-formed build-info record, and (with `key`) the one
 * for that key. Everything the fetch trusts comes through here.
 */
export function validBuildInfo(info, key) {
  return (
    !!info &&
    typeof info === "object" &&
    info.schema === BUILD_INFO_SCHEMA &&
    HEX64.test(info.inputsSha256 ?? "") &&
    HEX64.test(info.artifactSha256 ?? "") &&
    HEX64.test(info.glueSha256 ?? "") &&
    Number.isInteger(info.artifactBytes) &&
    Number.isInteger(info.glueBytes) &&
    typeof info.wasm === "string" &&
    typeof info.glue === "string" &&
    (key === undefined || info.inputsSha256 === key)
  );
}

function stamp() {
  const dir = resolve(argument("--dir", galleryDir));
  const source = argument("--source", "local");
  if (source !== "ci" && source !== "local") throw new Error("--source must be ci or local");
  const env = process.env;
  const info = describeBuild(dir, inputsHash(repoRoot), {
    source,
    commit: env.GITHUB_SHA ?? null,
    ref: env.GITHUB_REF ?? null,
    run:
      env.GITHUB_RUN_ID && env.GITHUB_REPOSITORY
        ? `${env.GITHUB_SERVER_URL ?? "https://github.com"}/${env.GITHUB_REPOSITORY}/actions/runs/${env.GITHUB_RUN_ID}`
        : null,
    builtAt: new Date().toISOString(),
    toolchain: {
      rustc: argument("--toolchain", null),
      wasmBindgen: argument("--wasm-bindgen", null),
      wasmOpt: argument("--wasm-opt", null),
    },
  });
  writeJson(resolve(dir, INFO_FILE), info);
  console.log(
    `stamped ${dir}: key ${info.inputsSha256.slice(0, 16)}, artifact ${info.artifactSha256.slice(0, 12)} (${info.artifactBytes} bytes)`,
  );
}

function releaseBase(env) {
  if (env.HEROGPUI_GALLERY_BASE_URL) return env.HEROGPUI_GALLERY_BASE_URL.replace(/\/*$/, "/");
  const repo =
    env.HEROGPUI_GALLERY_REPO ||
    (env.VERCEL_GIT_REPO_OWNER && env.VERCEL_GIT_REPO_SLUG
      ? `${env.VERCEL_GIT_REPO_OWNER}/${env.VERCEL_GIT_REPO_SLUG}`
      : env.GITHUB_REPOSITORY || DEFAULT_REPO);
  return `https://github.com/${repo}/releases/download/${RELEASE_TAG}/`;
}

async function download(url) {
  const response = await fetch(url, { redirect: "follow" });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`GET ${url}: HTTP ${response.status}`);
  return Buffer.from(await response.arrayBuffer());
}

async function downloadJson(url) {
  const bytes = await download(url);
  if (!bytes) return null;
  try {
    return JSON.parse(bytes.toString("utf8"));
  } catch {
    throw new Error(`GET ${url}: not JSON`);
  }
}

/** Poll for `name` until it exists or `seconds` pass; a network error is retried too. */
async function waitForJson(base, name, seconds, interval) {
  const deadline = Date.now() + seconds * 1000;
  let announced = false;
  for (;;) {
    try {
      const value = await downloadJson(base + name);
      if (value) return value;
    } catch (error) {
      console.warn(`gallery: ${error.message}`);
    }
    if (Date.now() + interval > deadline) return null;
    if (!announced) {
      console.log(
        `gallery: ${name} is not published yet; waiting up to ${seconds}s for CI's wasm job`,
      );
      announced = true;
    }
    await new Promise((done) => setTimeout(done, interval));
  }
}

/** Download both binaries `info` names and install them only if every byte checks out. */
async function install(base, info, dir) {
  const files = [
    [WASM_FILE, info.wasm, info.artifactSha256, info.artifactBytes],
    [GLUE_FILE, info.glue, info.glueSha256, info.glueBytes],
  ];
  const fetched = [];
  for (const [file, asset, expected, size] of files) {
    const bytes = await download(base + asset);
    if (!bytes) throw new Error(`${asset} is missing from the ${RELEASE_TAG} release`);
    const actual = sha256(bytes);
    if (bytes.length !== size || actual !== expected) {
      throw new Error(
        `${asset}: expected ${size} bytes with SHA-256 ${expected}, got ${bytes.length} bytes with ${actual}`,
      );
    }
    fetched.push([file, bytes]);
  }
  mkdirSync(dir, { recursive: true });
  for (const [file, bytes] of fetched) {
    const temporary = resolve(dir, `.${file}.download`);
    writeFileSync(temporary, bytes);
    renameSync(temporary, resolve(dir, file));
  }
}

function removeArtifact(dir) {
  for (const file of [WASM_FILE, GLUE_FILE]) rmSync(resolve(dir, file), { force: true });
}

/** An installed artifact whose bytes still match its build-info. */
function installedMatches(info, dir) {
  try {
    return (
      sha256(readFileSync(resolve(dir, WASM_FILE))) === info.artifactSha256 &&
      sha256(readFileSync(resolve(dir, GLUE_FILE))) === info.glueSha256
    );
  } catch {
    return false;
  }
}

/**
 * Install the artifact for this checkout into `dir` and return how:
 * "local", "exact", "fallback" or "missing". Throws when a required exact
 * artifact never appears or a download fails verification.
 */
export async function fetchArtifact({ env = process.env, dir = galleryDir } = {}) {
  const mode = env.HEROGPUI_GALLERY_ARTIFACT || "auto";
  if (!["auto", "require", "off"].includes(mode)) {
    throw new Error(`HEROGPUI_GALLERY_ARTIFACT must be auto, require or off, not ${mode}`);
  }
  const key = inputsHash(repoRoot);
  const pinned = readJson(resolve(webRoot, "src", "data", "wasm-parity.json"))?.inputsSha256;
  if (pinned !== key) {
    console.warn(
      `gallery: src/data/wasm-parity.json pins inputs ${String(pinned).slice(0, 16)}, this checkout hashes to ${key.slice(0, 16)}; ` +
        "run `pnpm run wasm:manifest` (extract:check fails on this in CI). Using the checkout's key.",
    );
  }
  mkdirSync(dir, { recursive: true });
  const infoPath = resolve(dir, INFO_FILE);
  const current = readJson(infoPath);
  const haveFiles = [WASM_FILE, GLUE_FILE].every((file) => existsSync(resolve(dir, file)));

  // A developer's own build (.shots/build-wasm.sh) is never replaced.
  if (haveFiles && (current?.source === "local" || !current)) {
    const info = validBuildInfo(current) ? current : describeBuild(dir, null, { source: "local" });
    writeJson(infoPath, { ...info, status: "local" });
    if (info.inputsSha256 !== key) {
      console.warn(
        "gallery: keeping the locally built artifact in public/gallery/, which was built from other sources than this checkout; " +
          "rebuild it with `bash .shots/build-wasm.sh`, or delete it to download CI's build.",
      );
    } else {
      console.log("gallery: using the locally built artifact in public/gallery/");
    }
    return "local";
  }

  if (mode === "off") {
    removeArtifact(dir);
    writeJson(infoPath, { schema: BUILD_INFO_SCHEMA, status: "missing", reason: "disabled" });
    console.log("gallery: HEROGPUI_GALLERY_ARTIFACT=off; building without the live preview");
    return "missing";
  }

  if (
    current?.status === "exact" &&
    validBuildInfo(current, key) &&
    installedMatches(current, dir)
  ) {
    console.log(`gallery: artifact for ${key.slice(0, 16)} already installed`);
    return "exact";
  }

  const onVercel = !!env.VERCEL;
  const production = env.VERCEL_ENV === "production";
  const required = mode === "require" || (mode === "auto" && production);
  const wait = Number(
    env.HEROGPUI_GALLERY_WAIT_SECONDS ?? (onVercel ? (production ? 1800 : 600) : 0),
  );
  const base = releaseBase(env);

  const interval = Number(env.HEROGPUI_GALLERY_POLL_MS ?? POLL_INTERVAL_MS);
  const exact = await waitForJson(
    base,
    keyAssetName(key),
    Number.isFinite(wait) ? wait : 0,
    interval,
  );
  if (exact) {
    if (!validBuildInfo(exact, key)) throw new Error(`${keyAssetName(key)} is malformed`);
    await install(base, exact, dir);
    writeJson(infoPath, { ...exact, status: "exact", fetchedFrom: base });
    console.log(
      `gallery: installed CI's artifact ${exact.artifactSha256.slice(0, 12)} for ${key.slice(0, 16)}` +
        (exact.commit ? ` (built from ${exact.commit.slice(0, 12)})` : ""),
    );
    return "exact";
  }

  const reason =
    `no artifact for this checkout (${keyAssetName(key)}) in the ${RELEASE_TAG} release of ${base}` +
    (wait > 0 ? ` after waiting ${wait}s` : "");
  if (required) {
    throw new Error(
      `gallery: ${reason}. A production build ships the exact artifact or nothing: ` +
        "check the CI `wasm`/`wasm-publish` jobs for this commit, then redeploy (the next master push also does).",
    );
  }

  let fallback = null;
  try {
    fallback = await downloadJson(base + MASTER_POINTER);
  } catch (error) {
    console.warn(`gallery: ${error.message}`);
  }
  if (validBuildInfo(fallback)) {
    try {
      await install(base, fallback, dir);
      writeJson(infoPath, {
        ...fallback,
        status: "fallback",
        requestedInputsSha256: key,
        fetchedFrom: base,
      });
      console.warn(
        `gallery: ${reason}; using master's latest artifact ${fallback.artifactSha256.slice(0, 12)} instead ` +
          "(the embed shows an 'earlier build' banner)",
      );
      return "fallback";
    } catch (error) {
      console.warn(`gallery: master's artifact failed verification: ${error.message}`);
    }
  }

  removeArtifact(dir);
  writeJson(infoPath, { schema: BUILD_INFO_SCHEMA, status: "missing", reason });
  console.warn(`gallery: ${reason}, and no fallback; building without the live preview`);
  return "missing";
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const command = process.argv[2];
  try {
    if (command === "key") console.log(inputsHash(repoRoot));
    else if (command === "stamp") stamp();
    else if (command === "fetch") await fetchArtifact();
    else {
      console.error(
        "usage: gallery-artifact.mjs key | stamp --dir <dir> --source ci|local | fetch",
      );
      process.exitCode = 2;
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
