import type { MetadataRoute } from "next";
import { getCatalog } from "@/lib/catalog";
import { AI_LINKS, GETTING_STARTED_LINKS, NAV_LINKS } from "@/lib/nav";
import { getMigrations } from "@/lib/migrations";
import { absoluteUrl } from "@/lib/seo";

/**
 * `/sitemap.xml` (under basePath: `/herogpui/sitemap.xml`): the landing page,
 * every docs page in the navigation, the upgrade guides, and every catalog
 * component page. Development-only routes (`/fixtures`) and the plain-text
 * `llms.txt` are left out.
 */
export default function sitemap(): MetadataRoute.Sitemap {
  const catalog = getCatalog();
  const routes = new Set<string>(["/"]);
  for (const link of [...NAV_LINKS, ...GETTING_STARTED_LINKS, ...AI_LINKS]) routes.add(link.href);
  // NAV_LINKS already carries /docs/extensions; named here so the sitemap
  // keeps listing it even if the navbar link moves.
  routes.add("/docs/extensions");
  routes.add("/docs/migration");
  for (const guide of getMigrations().guides) routes.add(`/docs/migration/${guide.slug}`);
  for (const category of catalog.categories) {
    for (const slug of category.components) {
      if (catalog.components[slug]?.title) routes.add(`/docs/components/${slug}`);
    }
  }
  return [...routes].map((route) => ({ url: absoluteUrl(route) }));
}
