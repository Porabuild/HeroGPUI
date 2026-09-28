import type { MetadataRoute } from "next";
import { getCatalog } from "@/lib/catalog";
import { AI_LINKS, GETTING_STARTED_LINKS, NAV_LINKS } from "@/lib/nav";
import { absoluteUrl } from "@/lib/seo";

/**
 * `/sitemap.xml` (under basePath: `/herogpui/sitemap.xml`): the landing page,
 * every docs page in the navigation, and every catalog component page.
 * Development-only routes (`/fixtures`) and the plain-text `llms.txt` are
 * left out.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const catalog = getCatalog();
  const routes = new Set<string>(["/"]);
  for (const link of [...NAV_LINKS, ...GETTING_STARTED_LINKS, ...AI_LINKS]) routes.add(link.href);
  for (const category of catalog.categories) {
    for (const slug of category.components) {
      if (catalog.components[slug]?.title) routes.add(`/docs/components/${slug}`);
    }
  }
  return [...routes].map((route) => ({ url: absoluteUrl(route) }));
}
