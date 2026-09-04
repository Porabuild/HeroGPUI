"use client";

import { cn } from "@heroui/react";
import { useEffect, useRef, useState } from "react";
import { publicUrl } from "@/lib/public-url";

/**
 * One embedded instance of the HeroGPUI desktop gallery — the real Rust/GPUI
 * application, compiled to WebAssembly — showing every example for one
 * component. Not a screenshot and not a recreation in React.
 *
 * GPUI's web target attaches a single canvas to `document.body` and supports
 * one top-level window per process, so a page can only ever host one of
 * these, never one per example. The `story` query parameter selects which
 * component the shared gallery module renders; the module itself is cached
 * across navigations by the browser.
 *
 * Renders nothing when `NEXT_PUBLIC_GALLERY_URL` is unset — the wasm
 * artifact is not hosted yet, and an empty preview is the honest state until
 * it is. Callers that need the existing screenshot fallback in that case
 * (see the component detail page) check the same env var themselves rather
 * than relying on this returning null.
 */

const GALLERY_BASE = process.env.NEXT_PUBLIC_GALLERY_URL ?? "";

const ABSOLUTE_URL_RE = /^[a-z][a-z0-9+.-]*:\/\//i;

/** True once the frame's chrome is at least `rootMargin` from the viewport. */
const ROOT_MARGIN = "200px";

function readTheme(): "light" | "dark" {
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

/**
 * `NEXT_PUBLIC_GALLERY_URL` prefixed with the site's basePath, the same way
 * `publicUrl` prefixes public/ assets — unless it is already an absolute
 * URL (a separately hosted gallery), which is used as-is.
 */
function galleryOrigin(base: string): string {
  return ABSOLUTE_URL_RE.test(base) ? base : publicUrl(base);
}

function embedUrl(slug: string, theme: "light" | "dark"): string {
  const base = galleryOrigin(GALLERY_BASE).replace(/\/+$/, "");
  const query = new URLSearchParams({ story: slug, theme }).toString();
  // Point at the real file so relative imports resolve to
  // /gallery/herogpui_web.js. The bare "/gallery/?story=…" form 308s to
  // "/gallery?story=…" (Next strips the trailing slash), and the module
  // then resolves relative to "/herogpui/gallery" → 404.
  return `${base}/index.html?${query}`;
}

export interface GalleryFrameProps {
  /** Catalog slug, sent as the embed's `story` query parameter. */
  slug: string;
  /** Component title, used to build an honest, specific iframe title. */
  title: string;
  className?: string;
  /**
   * Render the window bar and viewport without the outer card border and
   * caption, for embedding in a caller-owned card (the component page Usage
   * card stacks the frame above the first example's code in one border).
   */
  bare?: boolean;
}

export function GalleryFrame({ slug, title, className, bare = false }: GalleryFrameProps) {
  const [visible, setVisible] = useState(false);
  const [theme, setTheme] = useState<"light" | "dark">("light");
  const viewportRef = useRef<HTMLDivElement | null>(null);

  // Keep the embedded theme in sync with the host's, including a toggle
  // after the frame has already booted. Reading `document` only happens
  // here (post-mount), never during render, so server and first-paint
  // client markup match exactly.
  useEffect(() => {
    if (!GALLERY_BASE) return;
    setTheme(readTheme());
    const observer = new MutationObserver(() => setTheme(readTheme()));
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    return () => observer.disconnect();
  }, []);

  // Boot lazily: the iframe itself is not created until the frame scrolls
  // near the viewport, so the multi-megabyte wasm module never loads on
  // page view alone. Disconnects itself on first intersection.
  useEffect(() => {
    if (!GALLERY_BASE) return;
    const node = viewportRef.current;
    if (!node) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        setVisible(true);
        observer.disconnect();
      },
      { rootMargin: ROOT_MARGIN },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  if (!GALLERY_BASE) return null;

  const frame = (
    <>
      <div aria-hidden="true" className="window-bar">
        <div>
          <span />
          <span />
          <span />
        </div>
        <span>HeroGPUI / WebAssembly</span>
        <span className="shot-window-status">Live</span>
      </div>
      <div
        className="relative h-[380px] bg-surface-secondary sm:h-[480px] lg:h-[600px]"
        ref={viewportRef}
      >
        {visible ? (
          <iframe
            className="absolute inset-0 h-full w-full border-0"
            src={embedUrl(slug, theme)}
            title={`${title}, rendered live by HeroGPUI compiled to WebAssembly`}
          />
        ) : (
          <div aria-hidden="true" className="absolute inset-0" />
        )}
      </div>
    </>
  );

  if (bare) {
    return <div className={cn("m-0", className)}>{frame}</div>;
  }

  return (
    <figure className={cn("m-0", className)}>
      <div className="relative overflow-hidden rounded-xl border border-separator bg-surface shadow-none">
        {frame}
      </div>
      <figcaption className="mt-3 flex items-center gap-2 text-xs text-muted">
        <span aria-hidden="true" className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent" />
        HeroGPUI itself, compiled to WebAssembly and running in this frame. Not a screenshot, not a
        recreation.
      </figcaption>
    </figure>
  );
}
