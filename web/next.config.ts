import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import type { NextConfig } from "next";

// ---------------------------------------------------------------------------
// Content-Security-Policy
//
// The pages are statically prerendered, so there is no per-request nonce:
// Next's inline bootstrap/RSC scripts and the layout's no-flash theme script
// need 'unsafe-inline' in script-src (the documented "without nonces" CSP).
// What the policy does pin: every fetch, frame, image, font and script URL is
// same-origin, no plugins, no <base> rewriting, no cross-origin form posts,
// and only this site (or the parent zone that proxies it) may frame a page.
//
// The WebAssembly gallery (/gallery/index.html, also the component-page
// iframe) gets a stricter script-src: its two inline scripts are allowed by
// SHA-256, computed from the checked-in file at build time, plus
// 'wasm-unsafe-eval', which WebAssembly compilation requires.
// ---------------------------------------------------------------------------

const isDev = process.env.NODE_ENV === "development";

/** The parent zone's origin (porabuild.com) — the only other allowed framer. */
function siteOrigin(): string | null {
  try {
    return new URL(process.env.NEXT_PUBLIC_SITE_URL || "https://porabuild.com/herogpui").origin;
  } catch {
    return null;
  }
}

/** An absolute NEXT_PUBLIC_GALLERY_URL is a separately hosted gallery to frame. */
function galleryFrameOrigin(): string | null {
  const url = process.env.NEXT_PUBLIC_GALLERY_URL ?? "";
  if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(url)) return null;
  try {
    return new URL(url).origin;
  } catch {
    return null;
  }
}

/** `'sha256-…'` sources for each inline <script> of public/gallery/index.html. */
function galleryScriptHashes(): string[] {
  let html: string;
  try {
    html = readFileSync(path.join(process.cwd(), "public", "gallery", "index.html"), "utf8");
  } catch {
    return [];
  }
  return [...html.matchAll(/<script\b(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/gi)].map(
    (match) => `'sha256-${createHash("sha256").update(match[1], "utf8").digest("base64")}'`,
  );
}

/**
 * Vercel injects its comment/feedback toolbar (vercel.live) into preview
 * deployments only; production needs none of these origins.
 */
const vercelToolbar =
  process.env.VERCEL_ENV === "preview"
    ? {
        script: ["https://vercel.live"],
        connect: ["https://vercel.live", "wss://ws-us3.pusher.com"],
        img: ["https://vercel.live", "https://vercel.com"],
        frame: ["https://vercel.live"],
        style: ["https://vercel.live"],
        font: ["https://vercel.live", "https://assets.vercel.com"],
      }
    : { script: [], connect: [], img: [], frame: [], style: [], font: [] };

function policy(directives: Record<string, string[]>): string {
  return Object.entries(directives)
    .map(([name, sources]) => [name, ...sources].join(" "))
    .join("; ");
}

const frameAncestors = ["'self'", siteOrigin()].filter((value): value is string => !!value);

const shared = {
  "default-src": ["'self'"],
  "connect-src": ["'self'", ...(isDev ? ["ws:"] : []), ...vercelToolbar.connect],
  "img-src": ["'self'", "data:", "blob:", ...vercelToolbar.img],
  "font-src": ["'self'", "data:", ...vercelToolbar.font],
  "style-src": ["'self'", "'unsafe-inline'", ...vercelToolbar.style],
  "object-src": ["'none'"],
  "base-uri": ["'self'"],
  "form-action": ["'self'"],
  "frame-ancestors": frameAncestors,
};

const siteCsp = policy({
  ...shared,
  "script-src": [
    "'self'",
    "'unsafe-inline'",
    ...(isDev ? ["'unsafe-eval'"] : []),
    ...vercelToolbar.script,
  ],
  "frame-src": [
    "'self'",
    ...[galleryFrameOrigin()].filter((value): value is string => !!value),
    ...vercelToolbar.frame,
  ],
  "worker-src": ["'self'"],
});

const galleryCsp = policy({
  ...shared,
  "script-src": ["'self'", "'wasm-unsafe-eval'", ...galleryScriptHashes()],
  "frame-src": ["'none'"],
  "worker-src": ["'self'"],
});

const nextConfig: NextConfig = {
  // Local dev runs at "/", production is mounted at https://porabuild.com/herogpui.
  basePath: process.env.NEXT_PUBLIC_BASE_PATH ?? "",
  typedRoutes: true,
  poweredByHeader: false,
  images: { formats: ["image/avif", "image/webp"] },
  // public/ is served by exact path only — map the gallery directory onto
  // its index.html so NEXT_PUBLIC_GALLERY_URL can be the clean "/gallery"
  // (the source is basePath-prefixed automatically, like headers sources).
  async rewrites() {
    return [{ source: "/gallery", destination: "/gallery/index.html" }];
  },
  async headers() {
    return [
      {
        // `/:path*` (not `/(.*)`) so the bare basePath root `/herogpui` matches too.
        source: "/:path*",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
          { key: "Content-Security-Policy", value: siteCsp },
        ],
      },
      // Later entries override earlier ones for the same key, so the gallery
      // document (both URL forms: /gallery and /gallery/index.html) gets its
      // own policy.
      {
        source: "/gallery",
        headers: [{ key: "Content-Security-Policy", value: galleryCsp }],
      },
      {
        source: "/gallery/:path*",
        headers: [{ key: "Content-Security-Policy", value: galleryCsp }],
      },
      // The web-gallery artifact is content-addressed by its query: every
      // embed (src/lib/gallery-embed.ts) asks for `?v=<first 12 hex of the
      // artifact SHA-256>` (wasm-parity.json), and public/gallery/index.html
      // forwards that onto the glue and the .wasm. A new build is therefore a
      // new URL, so a versioned request can be cached for good instead of
      // revalidating ~5 MB on every page view. An unversioned request (a
      // direct link to the gallery) keeps the default revalidating policy.
      {
        source: "/gallery/:file(herogpui_web\\.js|herogpui_web_bg\\.wasm)",
        has: [{ type: "query", key: "v", value: "[0-9a-f]{12}" }],
        headers: [{ key: "Cache-Control", value: "public, max-age=31536000, immutable" }],
      },
    ];
  },
};

export default nextConfig;
