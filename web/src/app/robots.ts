import type { MetadataRoute } from "next";
import { absoluteUrl } from "@/lib/seo";

/**
 * `/robots.txt`. Under basePath this is `/herogpui/robots.txt`, which crawlers
 * do not read (they fetch `/robots.txt` at the host root, served by the parent
 * zone); it exists for the zone's own domain and to advertise the sitemap.
 * The parent zone's robots.txt should list the same `Sitemap:` line — see
 * DEPLOYMENT.md, "Search engines".
 */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: { userAgent: "*", allow: "/" },
    sitemap: absoluteUrl("/sitemap.xml"),
  };
}
