import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { generateReleaseManifest } from "../scripts/generate-release-manifest.js";
import { validateRelease } from "../scripts/verify-release.js";

test("builds a complete checksummed release manifest", async () => {
  const directory = await mkdtemp(join(tmpdir(), "herogpui-release-"));
  const version = "0.1.0";
  const names = [
    `herogpui-${version}-x86_64-pc-windows-msvc.exe`,
    `herogpui-${version}-aarch64-apple-darwin`,
    `herogpui-${version}-x86_64-unknown-linux-gnu`,
    `herogpui-${version}-aarch64-unknown-linux-gnu`,
  ];
  try {
    await Promise.all(names.map((name) => writeFile(join(directory, name), name)));
    const output = join(directory, "release-manifest.json");
    await generateReleaseManifest(directory, version, output);

    const manifest = JSON.parse(await readFile(output, "utf8"));
    assert.equal(manifest.version, version);
    assert.doesNotThrow(() => validateRelease({ version }, manifest));
    assert.deepEqual(Object.keys(manifest.assets), [
      "win32-x64",
      "darwin-arm64",
      "linux-x64",
      "linux-arm64",
    ]);
    for (const asset of Object.values(manifest.assets)) {
      assert.match(asset.sha256, /^[a-f0-9]{64}$/u);
    }
    const sums = await readFile(join(directory, "SHA256SUMS.txt"), "utf8");
    assert.equal(sums.trim().split("\n").length, names.length);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("refuses to publish an incomplete release manifest", () => {
  assert.throws(
    () => validateRelease({ version: "0.1.0" }, { version: "0.1.0", assets: {} }),
    /win32-x64/u,
  );
});
