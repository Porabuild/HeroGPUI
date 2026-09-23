// The release facts every surface quotes, read from the Cargo workspace so no
// page hand-types them: the workspace version (`[workspace.package].version`
// in Cargo.toml) and the GPUI version the lockfile actually resolved
// (`gpui-pre` in Cargo.lock).

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

export function workspaceVersions(repoRoot) {
  const manifest = readFileSync(resolve(repoRoot, "Cargo.toml"), "utf8");
  const table = manifest.match(/^\[workspace\.package\]\s*$([\s\S]*?)(?=^\[)/m)?.[1] ?? "";
  const version = table.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) throw new Error("Cargo.toml: no [workspace.package].version");

  const lock = readFileSync(resolve(repoRoot, "Cargo.lock"), "utf8");
  const gpuiVersion = lock.match(/^name = "gpui-pre"\r?\nversion = "([^"]+)"/m)?.[1];
  if (!gpuiVersion) throw new Error("Cargo.lock: no gpui-pre package");
  return { version, gpuiVersion };
}
