import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { chmod, mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { spawn } from "node:child_process";

const REPOSITORY = "https://github.com/heroui-inc/HeroGPUI";
const packageRoot = new URL("..", import.meta.url);

export function platformKey(platform = process.platform, arch = process.arch) {
  const supported = new Set([
    "win32-x64",
    "darwin-arm64",
    "linux-x64",
    "linux-arm64",
  ]);
  const key = `${platform}-${arch}`;
  if (!supported.has(key)) {
    throw new Error(
      `no gallery binary is published for ${platform}/${arch}; see ${REPOSITORY}/releases`,
    );
  }
  return key;
}

export function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export function verifySha256(bytes, expected, name) {
  const actual = sha256(bytes);
  if (!/^[a-f0-9]{64}$/u.test(expected) || actual !== expected) {
    throw new Error(`SHA-256 verification failed for ${name}`);
  }
}

function cacheRoot(platform = process.platform) {
  if (platform === "win32") {
    return process.env.LOCALAPPDATA ?? join(homedir(), "AppData", "Local");
  }
  if (platform === "darwin") {
    return join(homedir(), "Library", "Caches");
  }
  return process.env.XDG_CACHE_HOME ?? join(homedir(), ".cache");
}

async function loadManifest() {
  const packageJson = JSON.parse(await readFile(new URL("package.json", packageRoot), "utf8"));
  const manifest = JSON.parse(
    await readFile(new URL("release-manifest.json", packageRoot), "utf8"),
  );
  if (manifest.version !== packageJson.version) {
    throw new Error("the npm package and gallery release manifest versions differ");
  }
  return manifest;
}

async function verifiedCachePath(asset, version, key, refresh) {
  const executable = process.platform === "win32" ? "herogpui.exe" : "herogpui";
  const destination = join(cacheRoot(), "herogpui", version, key, executable);

  if (!refresh && existsSync(destination)) {
    const cached = await readFile(destination);
    verifySha256(cached, asset.sha256, asset.name);
    return destination;
  }

  const url = `${REPOSITORY}/releases/download/v${version}/${asset.name}`;
  const response = await fetch(url, { redirect: "follow", signal: AbortSignal.timeout(120_000) });
  if (!response.ok) {
    throw new Error(`download failed with HTTP ${response.status}: ${url}`);
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  verifySha256(bytes, asset.sha256, asset.name);

  await mkdir(dirname(destination), { recursive: true });
  const temporary = `${destination}.tmp-${process.pid}`;
  await rm(temporary, { force: true });
  try {
    await writeFile(temporary, bytes, { mode: 0o755, flag: "wx" });
    if (process.platform !== "win32") {
      await chmod(temporary, 0o755);
    }
    await rm(destination, { force: true });
    await rename(temporary, destination);
  } finally {
    await rm(temporary, { force: true });
  }
  return destination;
}

function launch(executable, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, { stdio: "inherit", windowsHide: false });
    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (signal) {
        reject(new Error(`gallery terminated by ${signal}`));
      } else {
        resolve(code ?? 1);
      }
    });
  });
}

export async function run(args) {
  const manifest = await loadManifest();
  if (args.includes("--help") || args.includes("-h")) {
    console.log("Usage: herogpui [--refresh] [gallery arguments]");
    return;
  }
  if (args.includes("--version") || args.includes("-V")) {
    console.log(manifest.version);
    return;
  }

  const refresh = args.includes("--refresh");
  const forwarded = args.filter((argument) => argument !== "--refresh");
  const key = platformKey();
  const asset = manifest.assets[key];
  if (!asset) {
    throw new Error(`release ${manifest.version} has no ${key} asset`);
  }
  const executable = await verifiedCachePath(asset, manifest.version, key, refresh);
  process.exitCode = await launch(executable, forwarded);
}
