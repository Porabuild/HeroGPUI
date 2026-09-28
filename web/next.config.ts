import type { NextConfig } from "next";

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
        source: "/(.*)",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
        ],
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
