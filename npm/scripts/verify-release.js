import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const PLATFORM_KEYS = ["win32-x64", "darwin-arm64", "linux-x64", "linux-arm64"];

export function validateRelease(packageJson, manifest) {
  if (manifest.version !== packageJson.version) {
    throw new Error("package.json and release-manifest.json versions differ");
  }
  for (const key of PLATFORM_KEYS) {
    const asset = manifest.assets[key];
    if (!asset || typeof asset.name !== "string" || !/^[a-f0-9]{64}$/u.test(asset.sha256)) {
      throw new Error(`release manifest is missing a valid ${key} asset`);
    }
  }
}

if (import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url)));
  const manifest = JSON.parse(await readFile(new URL("../release-manifest.json", import.meta.url)));
  validateRelease(packageJson, manifest);
}
