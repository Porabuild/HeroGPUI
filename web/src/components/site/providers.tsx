"use client";

import { useRouter } from "next/navigation";
import type { Route } from "next";
import { I18nProvider, RouterProvider } from "react-aria-components";
import type { ReactNode } from "react";
import { publicUrl } from "@/lib/public-url";

const BASE_PATH = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

/**
 * React Aria renders raw `<a href>` and knows nothing about Next's `basePath`.
 * Without this, every HeroUI `Link` emits an unprefixed href into the
 * server-rendered HTML: middle-click, "open in new tab", copy-link and no-JS
 * visits all land outside the `/herogpui` mount (404 or the parent zone).
 *
 * `useHref` transforms only the DOM `href`; the click handler still forwards
 * the original unprefixed path to `navigate`, whose `router.push` applies the
 * base path itself. Already-absolute URLs (https://, mailto:), in-page
 * fragments, and already-prefixed paths pass through untouched.
 */
function useSiteHref(href: string): string {
  if (
    !BASE_PATH ||
    !href.startsWith("/") ||
    href === BASE_PATH ||
    href.startsWith(`${BASE_PATH}/`)
  ) {
    return href;
  }
  // The site root maps to the bare prefix, matching the mount URL
  // (porabuild.com/herogpui) rather than adding a trailing slash.
  if (href === "/") {
    return BASE_PATH;
  }
  return publicUrl(href);
}

/**
 * HeroUI's provider stack, on the client so it can hand the Next.js router
 * to React Aria: every HeroUI `Link` (nav, sidebar, breadcrumbs) then
 * navigates client-side instead of triggering full page loads.
 */
export function SiteProviders({ children }: { children: ReactNode }) {
  const router = useRouter();
  return (
    <I18nProvider locale="en-US">
      <RouterProvider navigate={(href) => router.push(href as Route)} useHref={useSiteHref}>
        {children}
      </RouterProvider>
    </I18nProvider>
  );
}
