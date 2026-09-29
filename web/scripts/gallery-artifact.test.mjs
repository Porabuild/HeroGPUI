import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";

import { inputsHash } from "./extract-wasm-sections.mjs";
import {
  GLUE_FILE,
  INFO_FILE,
  WASM_FILE,
  binaryAssetNames,
  describeBuild,
  fetchArtifact,
  keyAssetName,
  newestMasterTag,
  parseReleaseFeed,
  releaseTag,
  validBuildInfo,
} from "./gallery-artifact.mjs";

const repoRoot = resolve(import.meta.dirname, "../..");
const key = inputsHash(repoRoot);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** A build-info record plus its binaries, as CI would publish them. */
function build(inputs, wasm, glue) {
  const dir = mkdtempSync(join(tmpdir(), "gallery-build-"));
  writeFileSync(join(dir, WASM_FILE), wasm);
  writeFileSync(join(dir, GLUE_FILE), glue);
  const info = describeBuild(dir, inputs, { source: "ci", commit: "c".repeat(40) });
  rmSync(dir, { recursive: true });
  // One release per key: its three assets under download/<tag>/.
  const at = `download/${releaseTag(inputs)}/`;
  return {
    info,
    assets: {
      [at + info.wasm]: Buffer.from(wasm),
      [at + info.glue]: Buffer.from(glue),
      [at + keyAssetName(inputs)]: info,
    },
  };
}

/** A releases Atom feed listing `releases` ({ tag, title }), newest first. */
function feed(releases) {
  const entries = releases
    .map(
      ({ tag, title }) =>
        `<entry><link rel="alternate" type="text/html" href="https://github.com/o/r/releases/tag/${tag}"/><title>${title.replace(/&/g, "&amp;")}</title></entry>`,
    )
    .join("");
  return Buffer.from(`<?xml version="1.0"?><feed>${entries}</feed>`);
}

/** Serve `assets` (name -> bytes or JSON value) the way the release download URL does. */
async function withRelease(assets, body) {
  const requests = [];
  const server = createServer((request, response) => {
    const name = decodeURIComponent(request.url.slice(1));
    requests.push(name);
    const asset = assets[name];
    if (asset === undefined) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200).end(Buffer.isBuffer(asset) ? asset : JSON.stringify(asset));
  });
  await new Promise((done) => server.listen(0, "127.0.0.1", done));
  const base = `http://127.0.0.1:${server.address().port}/`;
  try {
    return await body(base, requests);
  } finally {
    server.close();
  }
}

function scratch() {
  return mkdtempSync(join(tmpdir(), "gallery-install-"));
}

const quiet = { log: console.log, warn: console.warn };
test.beforeEach(() => {
  console.log = () => {};
  console.warn = () => {};
});
test.afterEach(() => {
  console.log = quiet.log;
  console.warn = quiet.warn;
});

test("asset names are keyed by inputs and content-addressed binaries", () => {
  assert.equal(releaseTag("ab".repeat(32)), "gallery-abababababababab");
  assert.equal(keyAssetName("ab".repeat(32)), "herogpui-gallery-abababababababab.json");
  assert.deepEqual(binaryAssetNames("1".repeat(64), "2".repeat(64)), {
    wasm: "herogpui-gallery-1111111111111111.wasm",
    glue: "herogpui-gallery-2222222222222222.js",
  });
});

test("build-info validation rejects malformed records and another key", () => {
  const { info } = build(key, "wasm", "glue");
  assert.ok(validBuildInfo(info, key));
  assert.ok(!validBuildInfo(info, "0".repeat(64)));
  assert.ok(!validBuildInfo({ ...info, schema: 2 }));
  assert.ok(!validBuildInfo({ ...info, artifactSha256: "nope" }));
  assert.ok(!validBuildInfo(null));
});

test("fetch installs the exact artifact for this checkout and verifies it", async () => {
  const { assets } = build(key, "exact wasm", "exact glue");
  const dir = scratch();
  await withRelease(assets, async (base) => {
    const env = { HEROGPUI_GALLERY_BASE_URL: base };
    assert.equal(await fetchArtifact({ env, dir }), "exact");
    assert.equal(readFileSync(join(dir, WASM_FILE), "utf8"), "exact wasm");
    const installed = JSON.parse(readFileSync(join(dir, INFO_FILE), "utf8"));
    assert.equal(installed.status, "exact");
    assert.equal(installed.artifactSha256, sha256("exact wasm"));
    // A second run finds it installed and downloads nothing.
    await withRelease({}, async (emptyBase, requests) => {
      assert.equal(
        await fetchArtifact({ env: { HEROGPUI_GALLERY_BASE_URL: emptyBase }, dir }),
        "exact",
      );
      assert.deepEqual(requests, []);
    });
  });
  rmSync(dir, { recursive: true });
});

test("fetch refuses bytes that do not match the published hash", async () => {
  const { info, assets } = build(key, "real wasm", "real glue");
  const tampered = {
    ...assets,
    [`download/${releaseTag(key)}/${info.wasm}`]: Buffer.from("evil wasm"),
  };
  const dir = scratch();
  await withRelease(tampered, async (base) => {
    await assert.rejects(
      fetchArtifact({ env: { HEROGPUI_GALLERY_BASE_URL: base }, dir }),
      /expected .* bytes with SHA-256/,
    );
    assert.ok(!existsSync(join(dir, WASM_FILE)), "nothing unverified is installed");
  });
  rmSync(dir, { recursive: true });
});

test("a preview falls back to master's artifact; production requires the exact one", async () => {
  const { assets } = build("f".repeat(64), "master wasm", "master glue");
  const older = build("e".repeat(64), "old wasm", "old glue");
  const releases = {
    ...assets,
    ...older.assets,
    "releases.atom": feed([
      { tag: "gallery-1111111111111111", title: "Web gallery 1111111111111111" },
      { tag: releaseTag("f".repeat(64)), title: "Web gallery ffff [master 2026-09-29T10:00:00Z]" },
      { tag: releaseTag("e".repeat(64)), title: "Web gallery eeee [master 2026-09-28T10:00:00Z]" },
    ]),
  };
  const dir = scratch();
  await withRelease(releases, async (base) => {
    const env = { HEROGPUI_GALLERY_BASE_URL: base, HEROGPUI_GALLERY_WAIT_SECONDS: "0" };
    assert.equal(await fetchArtifact({ env: { ...env, VERCEL_ENV: "preview" }, dir }), "fallback");
    const installed = JSON.parse(readFileSync(join(dir, INFO_FILE), "utf8"));
    assert.equal(installed.status, "fallback");
    assert.equal(
      readFileSync(join(dir, WASM_FILE), "utf8"),
      "master wasm",
      "the newest master build",
    );
    assert.equal(installed.requestedInputsSha256, key);

    await assert.rejects(
      fetchArtifact({ env: { ...env, VERCEL: "1", VERCEL_ENV: "production" }, dir }),
      /production build ships the exact artifact or nothing/,
    );
  });
  rmSync(dir, { recursive: true });
});

test("fetch waits for CI to publish the exact artifact", async () => {
  const { info, assets } = build(key, "late wasm", "late glue");
  const jsonPath = `download/${releaseTag(key)}/${keyAssetName(key)}`;
  const release = { ...assets };
  delete release[jsonPath];
  const dir = scratch();
  await withRelease(release, async (base, requests) => {
    setTimeout(() => {
      release[jsonPath] = info;
    }, 150);
    const env = {
      HEROGPUI_GALLERY_BASE_URL: base,
      HEROGPUI_GALLERY_WAIT_SECONDS: "5",
      HEROGPUI_GALLERY_POLL_MS: "50",
    };
    assert.equal(await fetchArtifact({ env, dir }), "exact");
    assert.ok(requests.filter((name) => name === jsonPath).length > 1);
  });
  rmSync(dir, { recursive: true });
});

test("with nothing published the build goes on without a live preview", async () => {
  const dir = scratch();
  writeFileSync(join(dir, INFO_FILE), JSON.stringify({ schema: 1, status: "exact", source: "ci" }));
  writeFileSync(join(dir, WASM_FILE), "stale");
  writeFileSync(join(dir, GLUE_FILE), "stale");
  await withRelease({}, async (base) => {
    // No master build anywhere either (the list URL is local, never GitHub's).
    const env = { HEROGPUI_GALLERY_BASE_URL: base, HEROGPUI_GALLERY_API_URL: `${base}api` };
    assert.equal(await fetchArtifact({ env, dir }), "missing");
  });
  assert.ok(!existsSync(join(dir, WASM_FILE)), "a stale download is removed");
  assert.equal(JSON.parse(readFileSync(join(dir, INFO_FILE), "utf8")).status, "missing");
  rmSync(dir, { recursive: true });
});

test("a local build is never replaced", async () => {
  const dir = scratch();
  writeFileSync(join(dir, WASM_FILE), "my wasm");
  writeFileSync(join(dir, GLUE_FILE), "my glue");
  await withRelease({}, async (base, requests) => {
    assert.equal(await fetchArtifact({ env: { HEROGPUI_GALLERY_BASE_URL: base }, dir }), "local");
    assert.deepEqual(requests, []);
  });
  const info = JSON.parse(readFileSync(join(dir, INFO_FILE), "utf8"));
  assert.equal(info.status, "local");
  assert.equal(info.artifactSha256, sha256("my wasm"));
  rmSync(dir, { recursive: true });
});

test("HEROGPUI_GALLERY_ARTIFACT=off builds without a download", async () => {
  const dir = scratch();
  await withRelease({}, async (base, requests) => {
    const env = { HEROGPUI_GALLERY_BASE_URL: base, HEROGPUI_GALLERY_ARTIFACT: "off" };
    assert.equal(await fetchArtifact({ env, dir }), "missing");
    assert.deepEqual(requests, []);
  });
  rmSync(dir, { recursive: true });
});

test("the preview fallback reads master builds from the feed, then the release list", async () => {
  assert.deepEqual(
    parseReleaseFeed(
      feed([{ tag: "gallery-0123456789abcdef", title: "A & B [master 1]" }]).toString(),
    ),
    [
      {
        tag: "gallery-0123456789abcdef",
        title: "A & B [master 1]",
      },
    ],
  );
  assert.equal(
    newestMasterTag([
      { tag: "gallery-aaaaaaaaaaaaaaaa", title: "x [master 2026-01-02T00:00:00Z]" },
      { tag: "gallery-bbbbbbbbbbbbbbbb", title: "x [master 2026-01-03T00:00:00Z]" },
      { tag: "v0.12.0", title: "x [master 2026-02-01T00:00:00Z]" },
      { tag: "gallery-cccccccccccccccc", title: "x (pull request build)" },
    ]),
    "gallery-bbbbbbbbbbbbbbbb",
  );

  // Nothing master in the feed: the REST list answers.
  const { assets } = build("d".repeat(64), "api wasm", "api glue");
  const releases = {
    ...assets,
    "releases.atom": feed([{ tag: "gallery-1111111111111111", title: "Web gallery 1111" }]),
    api: [
      {
        tag_name: releaseTag("d".repeat(64)),
        name: "Web gallery dddd [master 2026-09-29T00:00:00Z]",
      },
    ],
  };
  const dir = scratch();
  await withRelease(releases, async (base) => {
    const env = { HEROGPUI_GALLERY_BASE_URL: base, HEROGPUI_GALLERY_API_URL: `${base}api` };
    assert.equal(await fetchArtifact({ env, dir }), "fallback");
    assert.equal(readFileSync(join(dir, WASM_FILE), "utf8"), "api wasm");
  });
  rmSync(dir, { recursive: true });
});
