// Boot the real wasm gallery in headless Chrome over the DevTools protocol and
// exercise it the way a browser does: render every generated example in light
// (and each page's first example in dark), type into the text fields through
// the IME path, and scroll horizontally with Shift+wheel. It needs no npm
// dependencies — it speaks CDP over the global WebSocket — so CI can run it on
// the artifact the `wasm` job just built, and a developer can run it against a
// local `bash .shots/build-wasm.sh` output or a fetched release.
//
//   node scripts/gallery-smoke.mjs [--dir <gallery dir>] [--chrome <path>]
//                                  [--record <run.json>] [--shots <dir>]
//                                  [--only <slug>[,<slug>...]] [--skip-dark]
//
// `--dir` defaults to public/gallery and must hold index.html,
// herogpui_web.js and herogpui_web_bg.wasm (`pnpm run gallery:fetch` or
// `bash .shots/build-wasm.sh` produce them). `--record` writes the run record
// (schema_version 1, kind "wasm-smoke"); `--shots` writes one PNG per example
// for review. Exit status is 1 when any check fails, 2 on usage errors.
//
// The example list comes from src/data/rust-examples.json, not the served
// page, so a section the gallery fails to construct shows up as a blank stage
// rather than silently passing. Examples that legitimately render nothing at
// rest are listed in EXPECTED_BLANK with the reason they were verified.

import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { existsSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const webRoot = resolve(scriptDir, "..");
const examplesPath = resolve(scriptDir, "../src/data/rust-examples.json");
const defaultDir = resolve(webRoot, "public/gallery");

const REQUIRED_FILES = ["index.html", "herogpui_web.js", "herogpui_web_bg.wasm"];
const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json; charset=utf-8",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};

// The fields the IME-path check types into, by story slug, opened on each
// page's "Usage" example, which centres a single field in the preview stage.
const TEXT_INPUT_STORIES = ["input", "text-field", "text-area", "search-field"];
const TEXT_INPUT_VALUE = "Hello wörld";

// Examples verified to render identical to the blank stage at rest, keyed by
// section id (`gallery/<slug>/<section-slug>`). Empty until a run proves one.
const EXPECTED_BLANK = {};

const EXPECTED_ERRORS = {
  "gallery/avatar/fallback-content": {
    pattern: /Failed to load asset: .*images\/avatar-broken\.png/,
    reason: "v3's Fallback Content example loads a broken image on purpose",
  },
};

/**
 * The page errors one example step raised, minus the ones `EXPECTED_ERRORS`
 * lists for it. A listed example whose expected error never appeared fails as
 * a stale entry.
 */
export function exampleErrors(id, texts, expected = EXPECTED_ERRORS) {
  const entry = Object.prototype.hasOwnProperty.call(expected, id) ? expected[id] : null;
  const unexpected = entry ? texts.filter((text) => !entry.pattern.test(text)) : texts;
  if (unexpected.length) return unexpected.join("; ");
  if (entry && texts.length === unexpected.length) {
    return `stale expected-error entry: no page error matched ${entry.pattern}`;
  }
  return null;
}

/** The inventory's section-id rule (`interaction_inventory.py` `gallery_specimen`). */
export function sectionId(slug, heading) {
  return `gallery/${slug}/${sectionSlug(heading)}`;
}

/** The slugified section alone, for shot filenames under `<shots>/<slug>/`. */
export function sectionSlug(heading) {
  return (
    heading
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "section"
  );
}

const FLAGS = {
  "--dir": "dir",
  "--chrome": "chrome",
  "--record": "record",
  "--shots": "shots",
  "--only": "only",
};

/** Parse the CLI into options; throws on an unknown or valueless flag. */
export function parseArgs(argv) {
  const opts = {
    dir: defaultDir,
    chrome: null,
    record: null,
    shots: null,
    only: null,
    skipDark: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--skip-dark") {
      opts.skipDark = true;
      continue;
    }
    const key = FLAGS[arg];
    if (!key) throw new Error(`unknown argument: ${arg}`);
    const value = argv[i + 1];
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`${arg} requires a value`);
    }
    i++;
    if (key === "only") opts.only = value.split(",").filter(Boolean);
    else opts[key] = value;
  }
  return opts;
}

const MAC_CHROMES = [
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
];
const PATH_CHROMES = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"];

/**
 * Ordered Chrome candidates: `--chrome`, `$CHROME`, the macOS app bundles,
 * then the PATH names resolved through `$PATH`. `exists` and the environment
 * are injectable so the order is unit-testable.
 */
export function chromeCandidates({
  flag = null,
  env = {},
  platform = process.platform,
  pathEnv = env.PATH ?? env.Path ?? "",
  exists = existsSync,
} = {}) {
  const candidates = [];
  if (flag) candidates.push(flag);
  if (env.CHROME) candidates.push(env.CHROME);
  if (platform === "darwin") candidates.push(...MAC_CHROMES);
  const separator = platform === "win32" ? ";" : ":";
  for (const name of PATH_CHROMES) {
    const hit = pathEnv
      .split(separator)
      .filter(Boolean)
      .map((dir) => join(dir, name))
      .find(exists);
    if (hit) candidates.push(hit);
  }
  return candidates;
}

/** First Chrome candidate that exists, or null. */
export function resolveChrome(options = {}) {
  const exists = options.exists ?? existsSync;
  return chromeCandidates({ ...options, exists }).find(exists) ?? null;
}

/**
 * The blank-stage verdict for one rendered example. `isBlank` is whether the
 * settled PNG equals the blank baseline. An unexpected blank and a stale
 * allowlist entry both fail; an expected blank passes.
 */
export function blankResult(id, isBlank, allowlist = EXPECTED_BLANK) {
  const listed = Object.prototype.hasOwnProperty.call(allowlist, id);
  if (isBlank && !listed) {
    return { result: "fail", detail: "rendered identical to the blank stage" };
  }
  if (!isBlank && listed) {
    return { result: "fail", detail: "stale allowlist entry: renders non-blank" };
  }
  return { result: "pass", detail: isBlank ? "expected blank" : null };
}

/** The artifact record for `--record`, from build-info.json or the binaries. */
async function artifactInfo(dir) {
  const infoPath = join(dir, "build-info.json");
  try {
    const info = JSON.parse(await readFile(infoPath, "utf8"));
    return {
      inputsSha256: info.inputsSha256 ?? null,
      artifactSha256: info.artifactSha256 ?? null,
      glueSha256: info.glueSha256 ?? null,
      commit: info.commit ?? null,
      source: info.source ?? null,
    };
  } catch {
    const sha256 = async (name) =>
      createHash("sha256")
        .update(await readFile(join(dir, name)))
        .digest("hex");
    return {
      inputsSha256: null,
      artifactSha256: await sha256("herogpui_web_bg.wasm"),
      glueSha256: await sha256("herogpui_web.js"),
      commit: null,
      source: null,
    };
  }
}

/** Serve `dir` at `/` on an ephemeral loopback port. */
async function serveGallery(dir) {
  const server = createServer(async (req, res) => {
    try {
      const pathname = decodeURIComponent(new URL(req.url, "http://x").pathname);
      if (pathname === "/favicon.ico") {
        res.writeHead(204);
        res.end();
        return;
      }
      const file = join(dir, pathname === "/" ? "index.html" : pathname);
      const body = await readFile(file);
      res.writeHead(200, {
        "content-type": MIME[extname(file)] ?? "application/octet-stream",
        "cache-control": "no-store",
      });
      res.end(body);
    } catch {
      res.writeHead(404);
      res.end();
    }
  });
  await new Promise((ready, fail) => {
    server.once("error", fail);
    server.listen(0, "127.0.0.1", ready);
  });
  return { server, port: server.address().port };
}

/** Launch headless Chrome and connect to its page target over CDP. */
async function launchChrome(chromePath) {
  const profile = await mkdtemp(join(tmpdir(), "herogpui-smoke-"));
  const args = [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    "--window-size=1024,768",
    "--force-device-scale-factor=1",
    "--hide-scrollbars",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-component-extensions-with-background-pages",
  ];
  // Ubuntu 24.04 runners restrict user namespaces; only CI needs the bypass.
  if (process.env.CI) args.push("--no-sandbox");
  args.push("about:blank");
  const proc = spawn(chromePath, args, { stdio: ["ignore", "ignore", "pipe"] });
  let stderr = "";
  proc.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  try {
    const wsUrl = await new Promise((ready, fail) => {
      const timer = setTimeout(
        () => fail(new Error(`Chrome did not expose DevTools:\n${stderr}`)),
        15000,
      );
      proc.stderr.on("data", () => {
        const match = stderr.match(/DevTools listening on (ws:\/\/\S+)/);
        if (match) {
          clearTimeout(timer);
          ready(match[1]);
        }
      });
    });
    const httpBase = wsUrl.replace("ws://", "http://").replace(/\/devtools\/browser\/.*/, "");
    const targets = await (await fetch(`${httpBase}/json`)).json();
    const page = targets.find((target) => target.type === "page");
    if (!page) throw new Error("Chrome opened no page target");
    const cdp = await Cdp.connect(page.webSocketDebuggerUrl);
    return { proc, cdp, profile };
  } catch (error) {
    proc.kill();
    await rm(profile, { recursive: true, force: true, maxRetries: 3 }).catch(() => {});
    throw error;
  }
}

/** A minimal CDP client over one page WebSocket. */
class Cdp {
  #ws;
  #id = 0;
  #pending = new Map();
  #listeners = new Map();

  static async connect(wsUrl) {
    const ws = new WebSocket(wsUrl);
    await new Promise((ready, fail) => {
      ws.addEventListener("open", ready, { once: true });
      ws.addEventListener("error", fail, { once: true });
    });
    return new Cdp(ws);
  }

  constructor(ws) {
    this.#ws = ws;
    ws.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      if (message.id && this.#pending.has(message.id)) {
        const { resolve, reject } = this.#pending.get(message.id);
        this.#pending.delete(message.id);
        if (message.error) reject(new Error(message.error.message));
        else resolve(message.result);
        return;
      }
      const listeners = this.#listeners.get(message.method);
      if (listeners) for (const listener of listeners) listener(message.params);
    });
  }

  send(method, params = {}) {
    const id = ++this.#id;
    return new Promise((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
      this.#ws.send(JSON.stringify({ id, method, params }));
    });
  }

  on(method, listener) {
    if (!this.#listeners.has(method)) this.#listeners.set(method, new Set());
    this.#listeners.get(method).add(listener);
  }

  close() {
    this.#ws.close();
  }
}

/** Two animation frames, awaited inside the page. */
async function nextFrames(cdp) {
  await cdp.send("Runtime.evaluate", {
    expression:
      "new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
    awaitPromise: true,
  });
}

/** A PNG screenshot as a Buffer. */
async function screenshot(cdp) {
  const shot = await cdp.send("Page.captureScreenshot", { format: "png" });
  return Buffer.from(shot.data, "base64");
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Settle after an action: two animation frames, then poll screenshots every
 * 100 ms until two consecutive PNGs are byte-equal (capped at 1500 ms, so an
 * animated example just yields its latest frame). Returns the stable PNG.
 */
async function settleShot(cdp) {
  await nextFrames(cdp);
  const deadline = Date.now() + 1500;
  let previous = await screenshot(cdp);
  while (Date.now() < deadline) {
    await sleep(100);
    const current = await screenshot(cdp);
    if (current.equals(previous)) return current;
    previous = current;
  }
  return previous;
}

/**
 * Settle after a `preview-section` switch. Two consecutive equal frames can
 * both be the *previous* (or blank loading) frame before the new example
 * renders, so first wait until the frame differs from the blank baseline
 * (cap 3000 ms; a still-blank result is returned for the caller's blank check
 * to fail), then until two consecutive frames are byte-equal (cap 1500 ms).
 */
async function settleSwitch(cdp, blank) {
  let png = await screenshot(cdp);
  const diffDeadline = Date.now() + 3000;
  while (png.equals(blank) && Date.now() < diffDeadline) {
    await sleep(100);
    png = await screenshot(cdp);
  }
  const deadline = Date.now() + 1500;
  let previous = png;
  while (Date.now() < deadline) {
    await sleep(100);
    const current = await screenshot(cdp);
    if (current.equals(previous)) return current;
    previous = current;
  }
  return previous;
}

async function mouseMove(cdp, x, y) {
  await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
}

async function wheel(cdp, x, y, deltaX, deltaY, modifiers = 0) {
  await cdp.send("Input.dispatchMouseEvent", {
    type: "mouseWheel",
    x,
    y,
    deltaX,
    deltaY,
    modifiers,
  });
}

/** Press the platform select-all chord (Meta on macOS, Control elsewhere). */
async function chordSelectAll(cdp, modifier) {
  const mod =
    modifier === 4
      ? { key: "Meta", code: "MetaLeft", windowsVirtualKeyCode: 91 }
      : { key: "Control", code: "ControlLeft", windowsVirtualKeyCode: 17 };
  await cdp.send("Input.dispatchKeyEvent", {
    type: "rawKeyDown",
    ...mod,
    modifiers: modifier,
  });
  await sleep(50);
  await cdp.send("Input.dispatchKeyEvent", {
    type: "rawKeyDown",
    key: "a",
    code: "KeyA",
    windowsVirtualKeyCode: 65,
    modifiers: modifier,
    commands: modifier === 4 ? ["selectAll"] : [],
  });
  await sleep(50);
  await cdp.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    key: "a",
    code: "KeyA",
    windowsVirtualKeyCode: 65,
    modifiers: modifier,
  });
  await cdp.send("Input.dispatchKeyEvent", {
    type: "keyUp",
    ...mod,
    modifiers: 0,
  });
  await sleep(50);
}

/** Evaluate an expression in the page and return its value. */
async function evaluate(cdp, expression) {
  const result = await cdp.send("Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (result.exceptionDetails) {
    throw new Error(`page evaluation failed: ${result.exceptionDetails.text}`);
  }
  return result.result?.value;
}

/** Post a gallery-bridge message to the page itself (top-level is its parent). */
async function postGallery(cdp, message) {
  await evaluate(cdp, `window.postMessage(${JSON.stringify(message)}, location.origin)`);
}

// The canvas only exists once the wasm has mounted a frame, so requiring it
// keeps the probe from answering true on the blank document between
// navigations. `#loading` gone and no `.error` node mean a clean boot.
const READY_PROBE =
  "!!document.querySelector('canvas') && !document.getElementById('loading') && !document.querySelector('.error')";

/** Wait for the wasm boot to finish within `timeoutMs`. */
async function waitReady(cdp, timeoutMs = 60000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await evaluate(cdp, READY_PROBE)) return true;
    await sleep(250);
  }
  return false;
}

/** Read the focused editable element's value (the hidden IME textarea). */
async function focusedValue(cdp) {
  return evaluate(
    cdp,
    `(() => {
      const el = document.activeElement;
      if (el && (el.tagName === 'TEXTAREA' || el.tagName === 'INPUT')) return el.value;
      const editable = document.querySelector('textarea:focus, input:focus');
      return editable ? editable.value : null;
    })()`,
  );
}

async function clickAt(cdp, x, y) {
  for (const type of ["mousePressed", "mouseReleased"]) {
    await cdp.send("Input.dispatchMouseEvent", {
      type,
      x,
      y,
      button: "left",
      clickCount: 1,
    });
  }
}

async function pressKey(cdp, key, code, windowsVirtualKeyCode, modifiers = 0) {
  for (const type of ["keyDown", "keyUp"]) {
    await cdp.send("Input.dispatchKeyEvent", {
      type,
      key,
      code,
      windowsVirtualKeyCode,
      modifiers,
    });
  }
}

const sha256 = (buffer) => createHash("sha256").update(buffer).digest("hex");

function loadExamples(only) {
  const data = JSON.parse(readFileSync(examplesPath, "utf8"));
  const rows = [];
  for (const [slug, sections] of Object.entries(data)) {
    if (only && !only.includes(slug)) continue;
    for (const section of sections) {
      rows.push({ slug, heading: section.heading });
    }
  }
  return rows;
}

function firstSectionBySlug(examples) {
  const first = new Map();
  for (const example of examples) {
    if (!first.has(example.slug)) first.set(example.slug, example.heading);
  }
  return first;
}

async function browserVersion(cdp) {
  try {
    const version = await cdp.send("Browser.getVersion");
    return version.product ?? null;
  } catch {
    return null;
  }
}

/**
 * Run the whole smoke pass. Returns the run record; throws only on setup
 * failures (no Chrome, bad dir). Per-step failures land in the record.
 */
async function run(opts) {
  const missing = REQUIRED_FILES.filter((file) => !existsSync(join(opts.dir, file)));
  if (missing.length) {
    const error = new Error(
      `${opts.dir} is missing ${missing.join(", ")}. ` +
        "Fetch or build the artifact first: `pnpm run gallery:fetch` or `bash .shots/build-wasm.sh`.",
    );
    error.exitCode = 2;
    throw error;
  }
  const chrome = resolveChrome({ flag: opts.chrome });
  if (!chrome) {
    const error = new Error(
      "no Chrome found; pass --chrome <path> or set CHROME, or install Chrome/Chromium",
    );
    error.exitCode = 2;
    throw error;
  }
  const examples = loadExamples(opts.only);
  const { server, port } = await serveGallery(opts.dir);
  const base = `http://127.0.0.1:${port}`;
  const record = {
    schema_version: 1,
    kind: "wasm-smoke",
    recorded_on: new Date().toISOString().slice(0, 10),
    artifact: await artifactInfo(opts.dir),
    browser: null,
    platform: `${process.platform}-${process.arch}`,
    viewport: "1024x768@1",
    boot_ms: 0,
    checks: [],
    examples: [],
    allowlist: EXPECTED_BLANK,
    summary: { examples: examples.length, passed: 0, failed: 0, checks_failed: 0 },
  };
  const errors = [];
  let step = "boot";
  const setStep = (name) => {
    step = name;
  };
  const failIfErrors = (mark) =>
    errors
      .slice(mark)
      .map((entry) => entry.text)
      .join("; ") || null;
  const ctx = { errors, setStep, failIfErrors };

  let chromeHandle = null;
  try {
    chromeHandle = await launchChrome(chrome);
    const { cdp, proc, profile } = chromeHandle;
    try {
      record.browser = await browserVersion(cdp);
      const pageContexts = new Set();
      cdp.on("Runtime.executionContextCreated", ({ context }) => {
        if (context.auxData?.isDefault) pageContexts.add(context.id);
      });
      cdp.on("Runtime.executionContextsCleared", () => pageContexts.clear());
      cdp.on("Runtime.consoleAPICalled", (params) => {
        if (!pageContexts.has(params.executionContextId)) return;
        const text = params.args.map((arg) => arg.value ?? arg.description ?? "").join(" ");
        const panicked = /panicked at|RuntimeError|unreachable executed/.test(text);
        if (params.type === "error" || panicked) errors.push({ step, text });
      });
      cdp.on("Runtime.exceptionThrown", (params) => {
        const detail = params.exceptionDetails;
        if (detail.executionContextId && !pageContexts.has(detail.executionContextId)) return;
        errors.push({
          step,
          text: detail.exception?.description ?? detail.text,
        });
      });
      cdp.on("Log.entryAdded", (params) => {
        // Network resource failures (a 404 build-info.json, a missing favicon)
        // are covered by the render assertions; app errors arrive through the
        // console and exception channels instead.
        if (params.entry.level === "error" && params.entry.source !== "network") {
          errors.push({ step, text: params.entry.text });
        }
      });
      await cdp.send("Runtime.enable");
      await cdp.send("Page.enable");
      await cdp.send("Log.enable");

      // Phase 1: the gallery shell boots.
      const bootStart = Date.now();
      let mark = errors.length;
      setStep("boot");
      await cdp.send("Page.navigate", { url: `${base}/index.html` });
      const booted = await waitReady(cdp);
      record.boot_ms = Date.now() - bootStart;
      const bootErrors = failIfErrors(mark);
      record.checks.push({
        name: "boot",
        result: booted && !bootErrors ? "pass" : "fail",
        detail: booted ? (bootErrors ?? `${record.boot_ms} ms`) : "never reached a ready state",
      });

      // Phase 2: blank baselines in single-example preview mode.
      setStep("blank");
      mark = errors.length;
      await cdp.send("Page.navigate", {
        url: `${base}/index.html?preview=component&story=button&section=__smoke_blank__`,
      });
      const blankReady = await waitReady(cdp);
      const blankErrors = failIfErrors(mark);
      record.checks.push({
        name: "blank-baseline",
        result: blankReady && !blankErrors ? "pass" : "fail",
        detail: blankErrors ?? (blankReady ? null : "blank stage never became ready"),
      });
      const blankLight = await settleShot(cdp);
      await postGallery(cdp, { type: "herogpui:set-theme", dark: true });
      const blankDark = await settleShot(cdp);
      await postGallery(cdp, { type: "herogpui:set-theme", dark: false });
      await settleShot(cdp);

      // Phase 3: every example, light.
      const shotLight = new Map();
      for (const example of examples) {
        const id = sectionId(example.slug, example.heading);
        setStep(`light:${id}`);
        const exampleMark = errors.length;
        await postGallery(cdp, {
          type: "herogpui:preview-section",
          story: example.slug,
          section: example.heading,
        });
        const png = await settleSwitch(cdp, blankLight);
        shotLight.set(id, png);
        const verdict = blankResult(id, png.equals(blankLight), EXPECTED_BLANK);
        const stepError = exampleErrors(
          id,
          errors.slice(exampleMark).map((entry) => entry.text),
        );
        record.examples.push({
          id,
          story: example.slug,
          section: example.heading,
          light: stepError ? "fail" : verdict.result,
          dark: null,
          screenshot_sha256: sha256(png),
          detail: stepError ?? verdict.detail,
        });
        if (opts.shots) {
          const dir = join(opts.shots, example.slug);
          await mkdir(dir, { recursive: true });
          await writeFile(join(dir, `${sectionSlug(example.heading)}-light.png`), png);
        }
      }

      // Phase 4: dark, first example of each page.
      if (!opts.skipDark) {
        setStep("dark");
        await postGallery(cdp, { type: "herogpui:set-theme", dark: true });
        await settleShot(cdp);
        const first = firstSectionBySlug(examples);
        for (const [slug, heading] of first) {
          const id = sectionId(slug, heading);
          setStep(`dark:${id}`);
          const darkMark = errors.length;
          await postGallery(cdp, {
            type: "herogpui:preview-section",
            story: slug,
            section: heading,
          });
          const png = await settleSwitch(cdp, blankDark);
          const light = shotLight.get(id);
          const blank = png.equals(blankDark);
          const sameAsLight = light ? png.equals(light) : false;
          const stepError = exampleErrors(
            id,
            errors.slice(darkMark).map((entry) => entry.text),
            {},
          );
          const dark = stepError || blank || sameAsLight ? "fail" : "pass";
          const detail =
            stepError ??
            (blank
              ? "rendered identical to the dark blank stage"
              : sameAsLight
                ? "identical to the light render"
                : null);
          const row = record.examples.find((entry) => entry.id === id);
          if (row) {
            row.dark = dark;
            if (dark === "fail" && !row.detail) row.detail = detail;
          }
          if (opts.shots) {
            const dir = join(opts.shots, slug);
            await mkdir(dir, { recursive: true });
            await writeFile(join(dir, `${sectionSlug(heading)}-dark.png`), png);
          }
        }
        await postGallery(cdp, { type: "herogpui:set-theme", dark: false });
        await settleShot(cdp);
      }

      // Phase 5: the browser IME text-input path.
      await runTextInput(cdp, record, ctx, blankLight);

      // Phase 6: wheel delivery — page vertical, table horizontal, the two
      // real web bug candidates (scroll-shadow horizontal, shift+wheel).
      await runWheelScroll(cdp, record, ctx, blankLight);
    } finally {
      cdp.close();
      proc.kill();
      // Chrome keeps writing to its profile briefly after SIGTERM; wait for
      // the exit before removing it, and never let cleanup mask the result.
      await Promise.race([new Promise((resolve) => proc.once("exit", resolve)), sleep(3000)]);
      await rm(profile, { recursive: true, force: true, maxRetries: 3 }).catch(() => {});
    }
  } finally {
    server.close();
  }
  summarize(record);
  return record;
}

/** Type into each text field through insertText, Backspace and IME compose. */
async function runTextInput(cdp, record, ctx, blank) {
  const modifier = process.platform === "darwin" ? 4 : 2; // Meta on macOS, Ctrl else
  const stories = [...TEXT_INPUT_STORIES, "number-field"];
  for (const slug of stories) {
    const name = `text-input:${slug}/Usage`;
    ctx.setStep(name);
    const mark = ctx.errors.length;
    try {
      await postGallery(cdp, {
        type: "herogpui:preview-section",
        story: slug,
        section: "Usage",
      });
      await settleSwitch(cdp, blank);
      // The preview stage centres every Usage field; the click lands on it.
      const [width, height] = await evaluate(cdp, "[innerWidth, innerHeight]");
      await clickAt(cdp, Math.round(width / 2), Math.round(height / 2));
      const focused = await settleShot(cdp);

      const isNumber = slug === "number-field";
      const expected = isNumber ? "42" : TEXT_INPUT_VALUE;
      const focusProbe = await evaluate(
        cdp,
        `(() => { const el = document.activeElement; return el ? el.tagName + ':' + (el.value ?? '') : 'none'; })()`,
      );
      if (isNumber) await chordSelectAll(cdp, modifier);
      await cdp.send("Input.insertText", { text: expected });
      const typed = await settleShot(cdp);
      let value = await focusedValue(cdp);
      let ok = value === expected && !typed.equals(focused);
      let detail = ok
        ? null
        : value === expected
          ? "the field did not paint the typed text"
          : `value was ${JSON.stringify(value)} (focused ${focusProbe})`;

      if (!isNumber) {
        // Two Backspace key events delete two code points ("d" then "l").
        for (let i = 0; i < 2; i++) await pressKey(cdp, "Backspace", "Backspace", 8);
        await settleShot(cdp);
        value = await focusedValue(cdp);
        if (value !== "Hello wör") {
          ok = false;
          detail = `after Backspace value was ${JSON.stringify(value)}`;
        }
        await cdp.send("Input.imeSetComposition", {
          text: "e",
          selectionStart: 1,
          selectionEnd: 1,
        });
        await sleep(300);
        await cdp.send("Input.insertText", { text: "é" });
        await settleShot(cdp);
        value = await focusedValue(cdp);
        if (value !== "Hello wöré") {
          ok = false;
          detail = `after an IME commit value was ${JSON.stringify(value)}`;
        }
        // Select-all through GPUI's own chord, then delete. The chord must
        // paint a selection (the frame changes) and Backspace must empty the
        // mirror — this exercises GPUI select-all, not a DOM selection.
        const beforeSelect = await settleShot(cdp);
        await chordSelectAll(cdp, modifier);
        const afterSelect = await settleShot(cdp);
        if (afterSelect.equals(beforeSelect)) {
          ok = false;
          detail = "select-all chord did not paint a selection";
        }
        await pressKey(cdp, "Backspace", "Backspace", 8);
        await settleShot(cdp);
        value = await focusedValue(cdp);
        if (value !== "") {
          ok = false;
          detail = `after select-all+Backspace value was ${JSON.stringify(value)}`;
        }
      }
      await clickAt(cdp, 8, 8);
      await settleShot(cdp);
      const stepError = ctx.failIfErrors(mark);
      record.checks.push({
        name,
        result: ok && !stepError ? "pass" : "fail",
        detail: stepError ?? detail,
      });
    } catch (error) {
      record.checks.push({ name, result: "fail", detail: error.message });
    }
  }
}

/**
 * Wheel delivery checks. `wheel:page-vertical` and `wheel:table-horizontal`
 * pass today; `wheel:scroll-shadow-horizontal` and `wheel:table-shift` are
 * real web bug candidates (native scrolls the same components) and are kept
 * as failing assertions, not allowlisted.
 */
async function runWheelScroll(cdp, record, ctx, blank) {
  const cases = [
    // Plain horizontal wheel over Table "Narrow Width" — works.
    {
      name: "wheel:table-horizontal",
      story: "table",
      section: "Narrow Width",
      x: 160,
      y: 336,
      deltaX: 100,
      deltaY: 0,
      modifiers: 0,
    },
    // Plain horizontal wheel over ScrollShadow "Horizontal" — real bug: the
    // frame does not change even though the component overflows horizontally.
    {
      name: "wheel:scroll-shadow-horizontal",
      story: "scroll-shadow",
      section: "Horizontal",
      x: 512,
      y: 339,
      deltaX: 100,
      deltaY: 0,
      modifiers: 0,
    },
    // Shift+wheel over Table "Narrow Width" — real bug: `shift_wheel_scroll_x`
    // exists for exactly this path but the frame does not change.
    {
      name: "wheel:scroll-shadow-shift",
      story: "scroll-shadow",
      section: "Horizontal",
      x: 512,
      y: 339,
      deltaX: 0,
      deltaY: 100,
      modifiers: 8,
    },
    {
      name: "wheel:table-shift",
      story: "table",
      section: "Narrow Width",
      x: 160,
      y: 336,
      deltaX: 0,
      deltaY: 100,
      modifiers: 8,
    },
  ];
  for (const c of cases) {
    ctx.setStep(c.name);
    const mark = ctx.errors.length;
    try {
      await postGallery(cdp, {
        type: "herogpui:preview-section",
        story: c.story,
        section: c.section,
      });
      const before = await settleSwitch(cdp, blank);
      await mouseMove(cdp, c.x, c.y);
      // Three wheel events 100 ms apart total the 300 px the checks assert on.
      for (let i = 0; i < 3; i++) {
        await wheel(cdp, c.x, c.y, c.deltaX, c.deltaY, c.modifiers);
        await sleep(100);
      }
      const after = await settleShot(cdp);
      const changed = !before.equals(after);
      const stepError = ctx.failIfErrors(mark);
      record.checks.push({
        name: c.name,
        result: changed && !stepError ? "pass" : "fail",
        detail: stepError ?? (changed ? null : "wheel did not scroll"),
      });
    } catch (error) {
      record.checks.push({ name: c.name, result: "fail", detail: error.message });
    }
  }

  // Vertical wheel over the gallery shell scrolls the page. A single preview
  // example is too short to scroll, so this one loads the full `?story=button`
  // page (all of its sections) rather than preview-component mode.
  const name = "wheel:page-vertical";
  ctx.setStep(name);
  const mark = ctx.errors.length;
  try {
    const origin = await evaluate(cdp, "location.origin");
    await cdp.send("Page.navigate", { url: `${origin}/index.html?story=button` });
    await waitReady(cdp);
    const before = await settleShot(cdp);
    await mouseMove(cdp, 600, 450);
    for (let i = 0; i < 3; i++) {
      await wheel(cdp, 600, 450, 0, 100, 0);
      await sleep(100);
    }
    const after = await settleShot(cdp);
    const changed = !before.equals(after);
    const stepError = ctx.failIfErrors(mark);
    record.checks.push({
      name,
      result: changed && !stepError ? "pass" : "fail",
      detail: stepError ?? (changed ? null : "wheel did not scroll the page"),
    });
  } catch (error) {
    record.checks.push({ name, result: "fail", detail: error.message });
  }
}

function summarize(record) {
  let failedExamples = 0;
  for (const example of record.examples) {
    if (example.light === "fail" || example.dark === "fail") failedExamples++;
  }
  const failedChecks = record.checks.filter((check) => check.result === "fail").length;
  record.summary = {
    examples: record.examples.length,
    passed: record.examples.length - failedExamples,
    failed: failedExamples,
    checks_failed: failedChecks,
  };
  const failing = record.examples
    .filter((entry) => entry.light === "fail" || entry.dark === "fail")
    .map((entry) => entry.id);
  const failingChecks = record.checks
    .filter((check) => check.result === "fail")
    .map((check) => check.name);
  console.log(
    `wasm-smoke: ${record.summary.passed}/${record.summary.examples} examples passed, ` +
      `${failedChecks} check(s) failed`,
  );
  for (const id of failing) console.log(`  FAIL ${id}`);
  for (const name of failingChecks) console.log(`  FAIL ${name}`);
}

async function main() {
  let opts;
  try {
    opts = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(error.message);
    console.error(
      "usage: gallery-smoke.mjs [--dir <dir>] [--chrome <path>] [--record <run.json>] [--shots <dir>] [--only <slug>[,<slug>...]] [--skip-dark]",
    );
    process.exitCode = 2;
    return;
  }
  let record;
  try {
    record = await run(opts);
  } catch (error) {
    console.error(`gallery-smoke: ${error.message}`);
    process.exitCode = error.exitCode ?? 1;
    return;
  }
  if (opts.record) {
    const path = resolve(opts.record);
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, `${JSON.stringify(record, null, 2)}\n`);
  }
  const failed = record.examples.some((entry) => entry.light === "fail" || entry.dark === "fail");
  process.exitCode = failed || record.checks.some((check) => check.result === "fail") ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
