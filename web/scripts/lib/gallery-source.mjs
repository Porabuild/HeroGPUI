// Gallery component demos live in `gallery/src/pages/components/`, one
// category module per HeroUI v3 sidebar group. Extractors and wasm
// manifests read every `.rs` file in that directory as one source.

import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

export function galleryComponentsDir(repoRoot) {
  return resolve(repoRoot, "gallery", "src", "pages", "components");
}

export function readGallerySource(path) {
  if (statSync(path).isDirectory()) {
    return readdirSync(path)
      .filter((name) => name.endsWith(".rs"))
      .sort()
      .map((name) => readFileSync(join(path, name), "utf8"))
      .join("\n");
  }
  return readFileSync(path, "utf8");
}

export function readGalleryComponentSource(repoRoot) {
  return readGallerySource(galleryComponentsDir(repoRoot));
}

/** Read `foo.rs`, or concatenate every `.rs` file under `foo/` when split. */
export function readRustPath(path) {
  if (statSync(path).isFile()) return readFileSync(path, "utf8");
  return readGallerySource(path);
}

export function readRustModulePath(path) {
  try {
    return readRustPath(path);
  } catch (error) {
    if (path.endsWith(".rs")) {
      return readGallerySource(path.slice(0, -3));
    }
    throw error;
  }
}
