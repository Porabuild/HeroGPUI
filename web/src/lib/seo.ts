import type { Metadata } from "next";

/**
 * Canonical URLs and per-page social metadata.
 *
 * `NEXT_PUBLIC_SITE_URL` is the absolute URL the site is served from,
 * basePath included (`https://porabuild.com/herogpui`, see DEPLOYMENT.md and
 * `.env.example`). The root layout feeds it to `metadataBase`, which joins
 * each page's root-relative canonical and Open Graph URL onto that path.
 */
export const SITE_URL = (
  process.env.NEXT_PUBLIC_SITE_URL || "https://porabuild.com/herogpui"
).replace(/\/+$/, "");

const SITE_NAME = "HeroGPUI";

/** Absolute URL of a site route (`/docs/components` → `https://…/herogpui/docs/components`). */
export function absoluteUrl(route: string): string {
  return route === "/" ? SITE_URL : `${SITE_URL}${route}`;
}

export interface PageMetadataInput {
  /** The route without basePath, e.g. `/docs/getting-started/theming`. */
  path: string;
  title: string;
  description: string;
}

/**
 * Title, description, canonical URL, and matching Open Graph and Twitter
 * cards for one page. The layout's `openGraph`/`twitter` objects are replaced
 * wholesale by a page's, not merged, so every field is restated here.
 */
export function pageMetadata({ path, title, description }: PageMetadataInput): Metadata {
  const socialTitle = `${title} — ${SITE_NAME}`;
  return {
    title,
    description,
    alternates: { canonical: path },
    openGraph: {
      type: "website",
      siteName: SITE_NAME,
      url: path,
      title: socialTitle,
      description,
    },
    twitter: { card: "summary", title: socialTitle, description },
  };
}
