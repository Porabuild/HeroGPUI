/**
 * URLs and messages for the embedded HeroGPUI/WebAssembly gallery, shared by
 * the component pages' `GalleryFrame` and the landing page showcase so both
 * load the same cache-versioned artifact the same way.
 *
 * The checked-in artifact is served from `/gallery`; `NEXT_PUBLIC_GALLERY_URL`
 * may point at another path or origin.
 */

import { publicUrl } from "@/lib/public-url";

const GALLERY_BASE = process.env.NEXT_PUBLIC_GALLERY_URL || "/gallery";
const ABSOLUTE_URL_RE = /^[a-z][a-z0-9+.-]*:\/\//i;

export type GalleryTheme = "light" | "dark";

export function readTheme(): GalleryTheme {
  if (typeof document === "undefined") return "dark";
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

/**
 * The gallery base prefixed with the site's basePath, the same way
 * `publicUrl` prefixes public/ assets, unless it is already an absolute URL
 * (a separately hosted gallery), which is used as-is.
 */
export function galleryOrigin(): string {
  return ABSOLUTE_URL_RE.test(GALLERY_BASE) ? GALLERY_BASE : publicUrl(GALLERY_BASE);
}

/** The artifact version carried as `?v=`: the first 12 hex of its SHA-256. */
export function artifactVersion(sha256: string): string {
  return sha256.slice(0, 12);
}

export function embedUrl(
  slug: string,
  section: string,
  theme: GalleryTheme,
  wasmVersion: string,
): string {
  const base = galleryOrigin().replace(/\/+$/, "");
  const query = new URLSearchParams({ preview: "component", section, story: slug, theme });
  // The artifact hash rides along as the version. `index.html` forwards it
  // onto the glue and the .wasm, which next.config.ts then serves as
  // immutable: a new build is a new URL, never a stale cache hit.
  if (wasmVersion) query.set("v", artifactVersion(wasmVersion));
  // Point at the real file so relative imports resolve to
  // /gallery/herogpui_web.js. The bare "/gallery/?story=…" form 308s to
  // "/gallery?story=…" (Next strips the trailing slash), and the module
  // then resolves relative to "/herogpui/gallery" → 404.
  return `${base}/index.html?${query.toString()}`;
}

function post(frame: HTMLIFrameElement | null, message: Record<string, unknown>): void {
  const target = frame?.contentWindow;
  if (!target) return;
  const origin = new URL(galleryOrigin(), window.location.href).origin;
  target.postMessage(message, origin);
}

/** Switch the live instance's example, and its component when `story` is set. */
export function postPreview(
  frame: HTMLIFrameElement | null,
  section: string,
  story?: string,
): void {
  post(frame, { type: "herogpui:preview-section", section, ...(story ? { story } : {}) });
}

/** Switch the live instance's theme without reloading it. */
export function postTheme(frame: HTMLIFrameElement | null, theme: GalleryTheme): void {
  post(frame, { type: "herogpui:set-theme", dark: theme === "dark" });
}

/** True for the instance's one-time "booted" message from `frame`. */
export function isReadyMessage(event: MessageEvent, frame: HTMLIFrameElement | null): boolean {
  const data: unknown = event.data;
  return (
    event.source === frame?.contentWindow &&
    typeof data === "object" &&
    data !== null &&
    (data as { type?: unknown }).type === "herogpui:ready"
  );
}
