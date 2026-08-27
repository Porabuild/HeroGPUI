import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const ASSETS = {
  "win32-x64": "x86_64-pc-windows-msvc.exe",
  "darwin-arm64": "aarch64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
};

export async function generateReleaseManifest(dist, version, output) {
  const assets = {};
  const sums = [];
  for (const [key, suffix] of Object.entries(ASSETS)) {
    const name = `herogpui-${version}-${suffix}`;
    const digest = createHash("sha256").update(await readFile(join(dist, name))).digest("hex");
    assets[key] = { name, sha256: digest };
    sums.push(`${digest}  ${name}`);
  }

  await writeFile(output, `${JSON.stringify({ version, assets }, null, 2)}\n`);
  await writeFile(join(dist, "SHA256SUMS.txt"), `${sums.join("\n")}\n`);
}

if (import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [dist, version, output] = process.argv.slice(2);
  if (!dist || !version || !output) {
    throw new Error("usage: generate-release-manifest <dist> <version> <output>");
  }
  await generateReleaseManifest(dist, version, output);
}
