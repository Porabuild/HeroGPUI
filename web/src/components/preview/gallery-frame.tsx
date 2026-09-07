"use client";

import { cn } from "@heroui/react";
import { ChevronDown } from "lucide-react";
import type { ReactNode } from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { publicUrl } from "@/lib/public-url";

/**
 * One embedded HeroGPUI example — the real Rust/GPUI component, compiled to
 * WebAssembly. Not a screenshot and not a recreation in React.
 *
 * GPUI's web target attaches a single canvas to `document.body` and supports
 * one top-level window per process. The docs intentionally keep one iframe
 * alive instead of instantiating wasm for every example. The `story` query
 * parameter selects the component, while `section` selects its first example;
 * later selections use the message bridge. The module is cached across
 * navigations by the browser.
 *
 * The checked-in artifact is served from `/gallery` by default. Deployments
 * may override `NEXT_PUBLIC_GALLERY_URL` when the artifact is hosted elsewhere.
 */

const GALLERY_BASE = process.env.NEXT_PUBLIC_GALLERY_URL || "/gallery";

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

function embedUrl(
  slug: string,
  section: string,
  theme: "light" | "dark",
  wasmVersion: string = "",
): string {
  const base = galleryOrigin(GALLERY_BASE).replace(/\/+$/, "");
  const query = new URLSearchParams({
    preview: "component",
    section,
    story: slug,
    theme,
  }).toString();
  // Point at the real file so relative imports resolve to
  // /gallery/herogpui_web.js. The bare "/gallery/?story=…" form 308s to
  // "/gallery?story=…" (Next strips the trailing slash), and the module
  // then resolves relative to "/herogpui/gallery" → 404.
  //
  // The artifact hash rides along as the version: the browser caches the
  // gallery module and WASM across visits, so a deploy that only swaps
  // those files would otherwise leave visitors on the previous build.
  const version = wasmVersion ? `&v=${wasmVersion.slice(0, 12)}` : "";
  return `${base}/index.html?${query}${version}`;
}

export interface GalleryFrameProps {
  /** Catalog slug, sent as the embed's `story` query parameter. */
  slug: string;
  /** Component title, used to build an honest, specific iframe title. */
  title: string;
  /** Exact native gallery section rendered as the live specimen. */
  section: string;
  className?: string;
  /**
   * Render the window bar and viewport without the outer card border and
   * caption, for embedding in a caller-owned card (the component page Usage
   * card stacks the frame above the first example's code in one border).
   */
  bare?: boolean;
  /** Artifact hash appended to the embed URL so deploys bust module cache. */
  wasmVersion?: string;
}

export function GalleryFrame({
  slug,
  title,
  section,
  className,
  bare = false,
  wasmVersion = "",
}: GalleryFrameProps) {
  const [frameTheme, setFrameTheme] = useState<"light" | "dark" | null>(null);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const initialSection = useRef(section).current;
  const viewportRef = useRef<HTMLDivElement | null>(null);

  // Boot lazily: the iframe itself is not created until the frame scrolls
  // near the viewport. Capture the boot theme at that moment; the embedded
  // runtime follows later same-origin theme changes without reloading wasm.
  useEffect(() => {
    const node = viewportRef.current;
    if (!node) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((entry) => entry.isIntersecting)) return;
        setFrameTheme(readTheme());
        observer.disconnect();
      },
      { rootMargin: ROOT_MARGIN },
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, []);

  const selectSection = useCallback(() => {
    const target = iframeRef.current?.contentWindow;
    if (!target) return;
    const origin = new URL(galleryOrigin(GALLERY_BASE), window.location.href).origin;
    target.postMessage({ type: "herogpui:preview-section", section }, origin);
  }, [section]);

  useEffect(selectSection, [selectSection]);

  // Backstop for a `load` event that fired before React attached `onLoad`:
  // once the frame document reads complete, deliver the section message.
  // Same-origin only; a cross-origin gallery throws on `contentDocument`
  // access and keeps the `onLoad` path.
  useEffect(() => {
    if (!frameTheme) return;
    const frame = iframeRef.current;
    if (!frame) return;
    const timer = window.setInterval(() => {
      let complete = false;
      try {
        complete = frame.contentDocument?.readyState === "complete";
      } catch {
        complete = false;
      }
      if (complete) {
        window.clearInterval(timer);
        selectSection();
      }
    }, 250);
    return () => window.clearInterval(timer);
  }, [frameTheme, slug, selectSection]);

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
        className="docs-stage relative h-[320px] overflow-hidden sm:h-[360px] lg:h-[400px]"
        ref={viewportRef}
      >
        {frameTheme ? (
          <iframe
            className="absolute inset-0 h-full w-full border-0"
            onLoad={selectSection}
            ref={iframeRef}
            src={embedUrl(slug, initialSection, frameTheme, wasmVersion)}
            title={`${title} ${section}, rendered live by HeroGPUI compiled to WebAssembly`}
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
        <span
          aria-hidden="true"
          className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent"
        />
        HeroGPUI itself, compiled to WebAssembly and running in this frame. Not a screenshot, not a
        recreation.
      </figcaption>
    </figure>
  );
}

export interface ComponentPreviewExample {
  id: string;
  heading: string;
  description?: string;
  code: ReactNode;
}

interface ComponentExampleBrowserProps {
  slug: string;
  title: string;
  examples: ComponentPreviewExample[];
  wasmVersion?: string;
}

export function ComponentExampleBrowser({
  slug,
  title,
  examples,
  wasmVersion = "",
}: ComponentExampleBrowserProps) {
  const [selectedId, setSelectedId] = useState(examples[0]?.id ?? "");
  const selected = examples.find((example) => example.id === selectedId) ?? examples[0];
  if (!selected) return null;

  return (
    <section aria-labelledby="usage">
      <h2 id="usage">Usage</h2>
      {selected.description ? (
        <p className="mt-2 text-sm leading-6 text-muted">{selected.description}</p>
      ) : null}
      {examples.length > 1 ? (
        <label className="mt-4 flex flex-col gap-2 text-sm font-medium text-foreground sm:flex-row sm:items-center sm:justify-between sm:gap-4">
          Live example
          <div className="relative w-full sm:w-auto">
            <select
              className="w-full cursor-pointer appearance-none rounded-lg border border-separator bg-surface py-2 pl-3 pr-9 text-sm text-foreground outline-none transition-colors hover:border-foreground/30 focus:border-accent sm:min-w-48"
              onChange={(event) => setSelectedId(event.currentTarget.value)}
              value={selected.id}
            >
              {examples.map((example) => (
                <option key={example.id} value={example.id}>
                  {example.heading}
                </option>
              ))}
            </select>
            <ChevronDown
              aria-hidden="true"
              className="pointer-events-none absolute right-3 top-1/2 size-4 -translate-y-1/2 text-muted"
            />
          </div>
        </label>
      ) : null}
      <div className="mt-4 overflow-hidden rounded-xl border border-separator bg-surface">
        <GalleryFrame
          bare
          section={selected.heading}
          slug={slug}
          title={title}
          wasmVersion={wasmVersion}
        />
        <div className="border-t border-separator">{selected.code}</div>
      </div>
      <p className="mt-3 flex items-center gap-2 text-xs text-muted">
        <span
          aria-hidden="true"
          className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent"
        />
        Live HeroGPUI compiled to WebAssembly. Select any example without loading another WASM
        instance.
      </p>
    </section>
  );
}
