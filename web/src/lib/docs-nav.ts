import type { Catalog } from "@/lib/catalog";
import { AI_LINKS, GETTING_STARTED_LINKS, NAV_LINKS, SITE, type NavLink } from "@/lib/nav";

export interface SearchItem {
  href: string;
  title: string;
  group: string;
  keywords: string;
}

export interface Crumb {
  href: string;
  label: string;
}

export interface PagerLink {
  href: string;
  label: string;
}

/** Flat search index: site pages plus every catalog component. */
export function buildSearchItems(catalog: Catalog): SearchItem[] {
  const items: SearchItem[] = [
    { href: "/", title: "Home", group: "Site", keywords: "landing start hero" },
    ...NAV_LINKS.map((link) => ({
      href: link.href,
      title: link.label,
      group: "Site",
      keywords: link.label,
    })),
    { href: SITE.llmsTxt, title: "llms.txt", group: "Site", keywords: "api agents reference" },
  ];

  for (const link of GETTING_STARTED_LINKS) {
    items.push({
      href: link.href,
      title: link.label,
      group: "Get started",
      keywords: link.label,
    });
  }
  for (const link of AI_LINKS) {
    items.push({
      href: link.href,
      title: link.label,
      group: "AI",
      keywords: `${link.label} agents`,
    });
  }
  for (const category of catalog.categories) {
    for (const slug of category.components) {
      const component = catalog.components[slug];
      if (!component?.title) continue;
      items.push({
        href: `/docs/components/${component.slug}`,
        title: component.title,
        group: category.name,
        keywords: `${component.description} ${category.name}`,
      });
    }
  }
  return items;
}

export function breadcrumbsFor(pathname: string, catalog: Catalog): Crumb[] {
  const crumbs: Crumb[] = [{ href: "/docs/getting-started", label: "Docs" }];

  if (pathname === "/" || pathname === "") return [];
  if (pathname === "/docs/getting-started") return crumbs;
  if (pathname === "/docs/components") {
    crumbs.push({ href: "/docs/components", label: "Components" });
    return crumbs;
  }
  if (pathname === "/docs/releases") {
    crumbs.push({ href: "/docs/releases", label: "Releases" });
    return crumbs;
  }

  const started = GETTING_STARTED_LINKS.find((link) => pathname === link.href);
  if (started && started.href !== "/docs/getting-started") {
    crumbs.push({ href: started.href, label: started.label });
    return crumbs;
  }

  const ai = AI_LINKS.find((link) => pathname === link.href);
  if (ai) {
    crumbs.push({ href: "/docs/ai/llms-txt", label: "AI" });
    crumbs.push({ href: ai.href, label: ai.label });
    return crumbs;
  }

  const match = pathname.match(/^\/docs\/components\/([^/]+)$/);
  if (match) {
    const slug = match[1];
    const component = catalog.components[slug];
    const category = catalog.categories.find((entry) => entry.components.includes(slug));
    crumbs.push({ href: "/docs/components", label: "Components" });
    if (category) {
      crumbs.push({ href: `/docs/components#category-${category.slug}`, label: category.name });
    }
    if (component) {
      crumbs.push({ href: `/docs/components/${component.slug}`, label: component.title });
    }
    return crumbs;
  }

  return crumbs;
}

function sequenceFor(pathname: string, catalog: Catalog): NavLink[] | null {
  if (GETTING_STARTED_LINKS.some((link) => pathname === link.href)) {
    return GETTING_STARTED_LINKS;
  }
  if (AI_LINKS.some((link) => pathname === link.href)) {
    return AI_LINKS;
  }
  if (/^\/docs\/components\/[^/]+$/.test(pathname)) {
    return catalog.categories.flatMap((category) =>
      category.components.flatMap((slug) => {
        const component = catalog.components[slug];
        return component?.title
          ? [{ href: `/docs/components/${component.slug}`, label: component.title }]
          : [];
      }),
    );
  }
  return null;
}

export function pagerFor(
  pathname: string,
  catalog: Catalog,
): { prev: PagerLink | null; next: PagerLink | null } | null {
  const sequence = sequenceFor(pathname, catalog);
  if (!sequence) return null;
  const index = sequence.findIndex((link) => link.href === pathname);
  if (index < 0) return null;
  const prev = index > 0 ? sequence[index - 1] : null;
  const next = index < sequence.length - 1 ? sequence[index + 1] : null;
  return {
    prev: prev ? { href: prev.href, label: prev.label } : null,
    next: next ? { href: next.href, label: next.label } : null,
  };
}
