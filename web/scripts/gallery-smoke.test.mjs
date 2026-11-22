import assert from "node:assert/strict";
import test from "node:test";

import {
  Cdp,
  blankResult,
  chromeCandidates,
  exampleErrors,
  parseArgs,
  partitionExamples,
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
    "--workers",
    "4",
  ]);
  assert.equal(opts.dir, "/tmp/gallery");
  assert.equal(opts.chrome, "/opt/chrome");
  assert.equal(opts.record, "run.json");
  assert.equal(opts.shots, "shots");
  assert.deepEqual(opts.only, ["button", "select"]);
  assert.equal(opts.skipDark, true);
  assert.equal(opts.workers, 4);

  const defaults = parseArgs([]);
  assert.equal(defaults.skipDark, false);
  assert.equal(defaults.only, null);
  assert.equal(defaults.workers, 1);
  assert.ok(defaults.dir.endsWith("public/gallery"));

  assert.throws(() => parseArgs(["--bogus"]), /unknown argument/);
  assert.throws(() => parseArgs(["--dir"]), /requires a value/);
  assert.throws(() => parseArgs(["--workers"]), /requires a value/);
  assert.throws(() => parseArgs(["--workers", "04"]), /positive integer/);
  assert.throws(() => parseArgs(["--workers", "0"]), /positive integer/);
  assert.throws(() => parseArgs(["--workers", "-1"]), /positive integer/);
  assert.throws(() => parseArgs(["--workers", "1.5"]), /positive integer/);
  assert.throws(() => parseArgs(["--workers", "abc"]), /positive integer/);
});

test("partitionExamples balances across workers while keeping each story together", () => {
  const examples = [
    { slug: "a", heading: "1" },
    { slug: "a", heading: "2" },
    { slug: "a", heading: "3" },
    { slug: "b", heading: "1" },
    { slug: "b", heading: "2" },
    { slug: "c", heading: "1" },
  ];
  assert.deepEqual(partitionExamples([], 4), []);
  assert.deepEqual(partitionExamples(examples, 1), [examples]);
  assert.deepEqual(partitionExamples(examples, Number.NaN), [examples]);
  assert.deepEqual(partitionExamples(examples, undefined), [examples]);
  assert.deepEqual(partitionExamples(examples, 0), [examples]);
  assert.deepEqual(partitionExamples(examples, -2), [examples]);

  // Fewer workers than slugs: LPT assigns a(3) to bucket 0, then b(2) and c(1) to bucket 1.
  const two = partitionExamples(examples, 2);
  assert.equal(two.length, 2);
  assert.deepEqual(
    two[0].map((e) => `${e.slug}/${e.heading}`),
    ["a/1", "a/2", "a/3"],
  );
  assert.deepEqual(
    two[1].map((e) => `${e.slug}/${e.heading}`),
    ["b/1", "b/2", "c/1"],
  );

  // More workers than slugs: capped at slug count, each slug kept whole.
  const buckets = partitionExamples(examples, 4);
  assert.equal(buckets.length, 3);
  assert.equal(buckets.flat().length, examples.length);
  for (const bucket of buckets) {
    const slugs = new Set(bucket.map((e) => e.slug));
    for (const slug of slugs) {
      const inBucket = bucket.filter((e) => e.slug === slug).length;
      const total = examples.filter((e) => e.slug === slug).length;
      assert.equal(inBucket, total);
    }
  }
});

test("Cdp rejects pending and future sends when the WebSocket closes", async () => {
  const ws = new EventTarget();
  ws.readyState = 1;
  ws.send = () => {};
  ws.close = () => {
    ws.readyState = 3;
    ws.dispatchEvent(new Event("close"));
  };
  const cdp = new Cdp(ws);
  const pending = cdp.send("Runtime.enable");
  cdp.close();
  await assert.rejects(pending, /CDP WebSocket closed/);
  await assert.rejects(cdp.send("Page.enable"), /not open/);
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
