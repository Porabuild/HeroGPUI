import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { sourcePatch } from "./vendor-wasm-source.mjs";

test("source patch reconstructs staged, unstaged and new build files without changing the index", () => {
  const temporary = mkdtempSync(join(tmpdir(), "herogpui-vendor-test-"));
  const source = join(temporary, "source");
  const restored = join(temporary, "restored");
  const git = (cwd, ...args) =>
    execFileSync("git", ["-C", cwd, ...args], {
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
  try {
    mkdirSync(join(source, "crates"), { recursive: true });
    mkdirSync(join(source, "gallery"));
    git(source, "init");
    writeFileSync(join(source, "crates", "lib.rs"), "baseline\n");
    writeFileSync(join(source, "gallery", "old.rs"), "removed\n");
    git(source, "add", ".");
    git(
      source,
      "-c",
      "user.name=Vendor test",
      "-c",
      "user.email=test@example.invalid",
      "commit",
      "-m",
      "baseline",
    );
    git(temporary, "clone", "--no-hardlinks", source, restored);
    writeFileSync(join(source, "crates", "lib.rs"), "staged\n");
    git(source, "add", "crates/lib.rs");
    writeFileSync(join(source, "crates", "lib.rs"), "working tree\n");
    writeFileSync(join(source, "crates", "new.rs"), "new entry crate\n");
    writeFileSync(join(source, "gallery", "asset.bin"), Buffer.from([0, 1, 2, 255]));
    rmSync(join(source, "gallery", "old.rs"));
    writeFileSync(join(source, "scratch.log"), "do not vendor\n");
    const staged = git(source, "diff", "--cached");
    const status = git(source, "status", "--porcelain");
    const patch = sourcePatch(source);
    assert.equal(git(source, "diff", "--cached"), staged);
    assert.equal(git(source, "status", "--porcelain"), status);
    assert.ok(!patch.includes("scratch.log"));
    execFileSync("git", ["-C", restored, "apply", "--binary", "-"], { input: patch });
    for (const file of ["crates/lib.rs", "crates/new.rs", "gallery/asset.bin"]) {
      assert.deepEqual(readFileSync(join(restored, file)), readFileSync(join(source, file)));
    }
    assert.match(git(restored, "status", "--porcelain"), /D gallery\/old.rs/);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
