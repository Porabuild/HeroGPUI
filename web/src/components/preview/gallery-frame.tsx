"use client";

import { cn } from "@heroui/react";
import { Play } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  embedUrl,
  isReadyMessage,
  postPreview,
  readTheme,
  type GalleryTheme,
} from "@/lib/gallery-embed";

/**
 * One embedded HeroGPUI example — the real Rust/GPUI component, compiled to
 * WebAssembly. Not a screenshot and not a recreation in React.
 *
 * GPUI's web target attaches a single canvas to `document.body` and supports
 * one top-level window per process. The docs keep one iframe alive instead of
 * instantiating wasm for every example. The `story` query parameter selects
 * the component, while `section` selects its first example; later selections
 * use the `index.html` message bridge. The module is cached across
 * navigations by the browser (immutably, keyed by the artifact hash).
 */

/** True once the frame's chrome is at least `rootMargin` from the viewport. */
const ROOT_MARGIN = "200px";

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
   * caption, for embedding in a caller-owned card.
   */
  bare?: boolean;
  /** Artifact hash appended to the embed URL so deploys bust module cache. */
  wasmVersion?: string;
  /** Id of the element describing the example, for `aria-describedby`. */
  describedBy?: string;
}

export function GalleryFrame({
  slug,
  title,
  section,
  className,
  bare = false,
  wasmVersion = "",
  describedBy,
}: GalleryFrameProps) {
  const [frameTheme, setFrameTheme] = useState<GalleryTheme | null>(null);
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
    postPreview(iframeRef.current, section);
  }, [section]);

  // index.html queues a selection that arrives before boot, and announces
  // `herogpui:ready` once live; re-sending then covers a message the frame
  // document was not yet listening for.
  useEffect(selectSection, [selectSection]);
  useEffect(() => {
    if (!frameTheme) return;
    const onMessage = (event: MessageEvent) => {
      if (isReadyMessage(event, iframeRef.current)) selectSection();
    };
    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }, [frameTheme, selectSection]);

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
            aria-describedby={describedBy}
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

/** A live-capable example: its anchor id and its gallery section heading. */
export interface LiveExample {
  id: string;
  heading: string;
}

const SELECT_EVENT = "herogpui:select-example";
const PREVIEW_ID = "live-preview";

function exampleFromHash(examples: LiveExample[]): LiveExample | undefined {
  const id = decodeURIComponent(window.location.hash.replace(/^#/, ""));
  return examples.find((example) => example.id === id);
}

interface LiveExamplePreviewProps {
  slug: string;
  title: string;
  examples: LiveExample[];
  wasmVersion?: string;
}

/**
 * The page's one live instance. Every example is rendered statically below
 * it (heading, description, code); this frame shows whichever one the reader
 * picked with that example's "Show live" button, or the one named by the
 * URL hash (`#rust-variants`), so an example link opens with it running.
 */
export function LiveExamplePreview({
  slug,
  title,
  examples,
  wasmVersion = "",
}: LiveExamplePreviewProps) {
  const [selected, setSelected] = useState<LiveExample | undefined>(examples[0]);

  useEffect(() => {
    const fromHash = () => {
      const match = exampleFromHash(examples);
      if (match) setSelected(match);
    };
    const onSelect = (event: Event) => {
      const id = (event as CustomEvent<string>).detail;
      const match = examples.find((example) => example.id === id);
      if (match) setSelected(match);
    };
    fromHash();
    window.addEventListener("hashchange", fromHash);
    window.addEventListener(SELECT_EVENT, onSelect);
    return () => {
      window.removeEventListener("hashchange", fromHash);
      window.removeEventListener(SELECT_EVENT, onSelect);
    };
  }, [examples]);

  if (!selected) return null;

  return (
    <div className="mt-4" id={PREVIEW_ID}>
      <div className="overflow-hidden rounded-xl border border-separator bg-surface">
        <GalleryFrame
          bare
          describedBy={`${selected.id}-description`}
          section={selected.heading}
          slug={slug}
          title={title}
          wasmVersion={wasmVersion}
        />
      </div>
      <p aria-live="polite" className="mt-3 flex items-center gap-2 text-xs text-muted">
        <span
          aria-hidden="true"
          className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent"
        />
        <span>
          Showing <a href={`#${selected.id}`}>{selected.heading}</a>, live: HeroGPUI compiled to
          WebAssembly. The canvas has no accessibility tree; each example&apos;s description and
          code below carry the same information.
        </span>
      </p>
    </div>
  );
}

/** "Show live" for one statically rendered example. */
export function ShowLiveButton({ id, heading }: LiveExample) {
  return (
    <button
      className="inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-separator bg-surface px-2.5 py-1 text-xs font-medium text-foreground transition-colors hover:border-accent"
      onClick={() => {
        window.history.replaceState(null, "", `#${id}`);
        window.dispatchEvent(new CustomEvent(SELECT_EVENT, { detail: id }));
        document
          .getElementById(PREVIEW_ID)
          ?.scrollIntoView({ behavior: "smooth", block: "center" });
      }}
      type="button"
    >
      <Play aria-hidden="true" className="size-3" />
      Show live<span className="sr-only">: {heading}</span>
    </button>
  );
}
