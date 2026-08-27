import assert from "node:assert/strict";
import { test } from "node:test";

import { platformKey, sha256, verifySha256 } from "../lib/launcher.js";

test("maps every published platform", () => {
  assert.equal(platformKey("win32", "x64"), "win32-x64");
  assert.equal(platformKey("darwin", "arm64"), "darwin-arm64");
  assert.equal(platformKey("linux", "x64"), "linux-x64");
  assert.equal(platformKey("linux", "arm64"), "linux-arm64");
});

test("rejects an unpublished platform", () => {
  assert.throws(() => platformKey("darwin", "x64"), /no gallery binary/u);
});

test("verifies release bytes", () => {
  const bytes = Buffer.from("gallery");
  const digest = sha256(bytes);
  assert.doesNotThrow(() => verifySha256(bytes, digest, "gallery"));
  assert.throws(() => verifySha256(bytes, "0".repeat(64), "gallery"), /SHA-256/u);
  assert.throws(() => verifySha256(bytes, "pending", "gallery"), /SHA-256/u);
});
