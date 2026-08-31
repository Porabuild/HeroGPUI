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
    ];
  },
};

export default nextConfig;
