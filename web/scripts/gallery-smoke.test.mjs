import assert from "node:assert/strict";
import test from "node:test";

import {
  blankResult,
  chromeCandidates,
  exampleErrors,
  parseArgs,
  resolveChrome,
  sectionId,
  sectionSlug,
} from "./gallery-smoke.mjs";

test("sectionId follows the inventory slug rule", () => {
  assert.equal(sectionId("button", "Usage"), "gallery/button/usage");
  assert.equal(
    sectionId("select", "With Disabled Options"),
    "gallery/select/with-disabled-options",
  );
  assert.equal(sectionId("combo-box", "Custom Value!"), "gallery/combo-box/custom-value");
  // A heading with no slug-able characters falls back to "section".
  assert.equal(sectionId("x", "!!!"), "gallery/x/section");
  assert.equal(sectionSlug("Row Hover & States"), "row-hover-states");
});

test("parseArgs reads every flag and rejects bad input", () => {
  const opts = parseArgs([
    "--dir",
    "/tmp/gallery",
    "--chrome",
    "/opt/chrome",
    "--record",
    "run.json",
    "--shots",
    "shots",
    "--only",
    "button,select",
    "--skip-dark",
  ]);
  assert.equal(opts.dir, "/tmp/gallery");
  assert.equal(opts.chrome, "/opt/chrome");
  assert.equal(opts.record, "run.json");
  assert.equal(opts.shots, "shots");
  assert.deepEqual(opts.only, ["button", "select"]);
  assert.equal(opts.skipDark, true);

  const defaults = parseArgs([]);
  assert.equal(defaults.skipDark, false);
  assert.equal(defaults.only, null);
  assert.ok(defaults.dir.endsWith("public/gallery"));

  assert.throws(() => parseArgs(["--bogus"]), /unknown argument/);
  assert.throws(() => parseArgs(["--dir"]), /requires a value/);
});

test("chromeCandidates orders flag, env, macOS bundles, then PATH", () => {
  const exists = (path) =>
    path === "/flag/chrome" ||
    path === "/env/chrome" ||
    path === "/usr/bin/google-chrome" ||
    path === "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
  const candidates = chromeCandidates({
    flag: "/flag/chrome",
    env: { CHROME: "/env/chrome", PATH: "/usr/bin:/opt/bin" },
    platform: "darwin",
    exists,
  });
  assert.deepEqual(candidates, [
    "/flag/chrome",
    "/env/chrome",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
  ]);
});

test("resolveChrome returns the first existing candidate or null", () => {
  const exists = (path) => path === "/env/chrome";
  assert.equal(
    resolveChrome({
      env: { CHROME: "/env/chrome", PATH: "" },
      platform: "linux",
      exists,
    }),
    "/env/chrome",
  );
  assert.equal(resolveChrome({ env: { PATH: "" }, platform: "linux", exists: () => false }), null);
});

test("blankResult fails unexpected blanks and stale allowlist entries", () => {
  const allowlist = { "gallery/x/blank": "verified empty" };
  // Non-blank, unlisted: pass.
  assert.equal(blankResult("gallery/button/usage", false, allowlist).result, "pass");
  // Blank, unlisted: fail.
  assert.equal(blankResult("gallery/button/usage", true, allowlist).result, "fail");
  // Blank, listed: pass (expected).
  assert.equal(blankResult("gallery/x/blank", true, allowlist).result, "pass");
  // Non-blank, listed: fail (stale entry).
  const stale = blankResult("gallery/x/blank", false, allowlist);
  assert.equal(stale.result, "fail");
  assert.match(stale.detail, /stale allowlist/);
});

test("exampleErrors drops expected errors and fails a stale entry", () => {
  const expected = { "gallery/x/broken": { pattern: /broken\.png/, reason: "on purpose" } };
  assert.equal(exampleErrors("gallery/x/ok", [], expected), null);
  assert.equal(exampleErrors("gallery/x/ok", ["boom"], expected), "boom");
  assert.equal(exampleErrors("gallery/x/broken", ["load broken.png"], expected), null);
  assert.equal(exampleErrors("gallery/x/broken", ["load broken.png", "boom"], expected), "boom");
  assert.match(exampleErrors("gallery/x/broken", [], expected), /stale expected-error/);
});
