"use client";

import { cn } from "@heroui/react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  artifactVersion,
  embedUrl,
  galleryOrigin,
  isReadyMessage,
  postPreview,
  postTheme,
  readTheme,
  type GalleryTheme,
} from "@/lib/gallery-embed";
import { publicUrl } from "@/lib/public-url";

interface SpecimenTab {
  id: string;
  slug: string;
  section: string;
  label: string;
}

const SPECIMEN_TABS: SpecimenTab[] = [
  { id: "button", slug: "button", section: "Variants", label: "Button" },
  { id: "date-field", slug: "date-field", section: "Usage", label: "DateField" },
  { id: "button-group", slug: "button-group", section: "Merged", label: "ButtonGroup" },
  { id: "chip", slug: "chip", section: "Usage", label: "Chip" },
  { id: "alert", slug: "alert", section: "Usage", label: "Alert" },
];

/**
 * Start downloading the artifact the moment the reader shows intent to run
 * it (pointer over, focus on, or touch of the poster or a tab), so the click
 * that follows finds it in the HTTP cache. Only for a versioned artifact —
 * next.config.ts serves `?v=` URLs as immutable, so the frame's own requests
 * for the same URLs reuse these responses — and never under Save-Data.
 */
function prefetchArtifact(wasmVersion: string): void {
  if (!wasmVersion) return;
  const connection = (navigator as Navigator & { connection?: { saveData?: boolean } }).connection;
  if (connection?.saveData) return;
  const base = new URL(`${galleryOrigin().replace(/\/+$/, "")}/`, window.location.href);
  for (const file of ["herogpui_web.js", "herogpui_web_bg.wasm"]) {
    const url = new URL(file, base);
    url.searchParams.set("v", artifactVersion(wasmVersion));
    const link = document.createElement("link");
    link.rel = "prefetch";
    link.href = url.href;
    document.head.appendChild(link);
  }
}

interface HeroWasmShowcaseProps {
  /** SHA-256 of the installed artifact (public/gallery/build-info.json), for `?v=`. */
  wasmVersion: string;
}

/**
 * The landing page's live specimen. The multi-megabyte module is not fetched
 * on page load: the frame is created only when the reader asks for it (the
 * poster button or a specimen tab), and the download starts at the first sign
 * of that intent (`prefetchArtifact`). Mounting on visibility instead would
 * fetch it for every visitor, since the specimen sits in the first viewport. After that one instance stays alive —
 * tabs switch component and example, and the site theme toggle switches the
 * theme, all over the `index.html` message bridge rather than a reload.
 */
export function HeroWasmShowcase({ wasmVersion }: HeroWasmShowcaseProps) {
  const [activeTab, setActiveTab] = useState<SpecimenTab>(SPECIMEN_TABS[0]);
  // The boot URL (story, section, theme) is fixed when the frame is created;
  // later changes travel as messages, so `src` never changes and the
  // instance never restarts.
  const [bootSrc, setBootSrc] = useState<string | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const prefetched = useRef(false);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);
  const stripRef = useRef<HTMLDivElement | null>(null);
  // True while the specimen tab strip can scroll further right. Drives the
  // right-edge fade mask so the last tab does not look unreachable on
  // narrow viewports.
  const [canScrollRight, setCanScrollRight] = useState(false);

  const updateScrollFade = useCallback(() => {
    const node = stripRef.current;
    if (!node) return;
    setCanScrollRight(node.scrollWidth - node.scrollLeft - node.clientWidth > 1);
  }, []);

  useEffect(() => {
    updateScrollFade();
    const node = stripRef.current;
    if (!node) return;
    const observer = new ResizeObserver(updateScrollFade);
    observer.observe(node);
    window.addEventListener("resize", updateScrollFade);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", updateScrollFade);
    };
  }, [updateScrollFade]);

  const warm = useCallback(() => {
    if (prefetched.current || bootSrc) return;
    prefetched.current = true;
    prefetchArtifact(wasmVersion);
  }, [bootSrc, wasmVersion]);

  const boot = useCallback(
    (tab: SpecimenTab) => {
      setBootSrc((current) => current ?? embedUrl(tab.slug, tab.section, readTheme(), wasmVersion));
    },
    [wasmVersion],
  );

  // Follow the site theme without a reload. A same-origin frame also watches
  // the parent itself; the message covers a separately hosted gallery.
  useEffect(() => {
    if (!bootSrc) return;
    let previous: GalleryTheme = readTheme();
    const observer = new MutationObserver(() => {
      const current = readTheme();
      if (current === previous) return;
      previous = current;
      postTheme(iframeRef.current, current);
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class", "data-theme"],
    });
    return () => observer.disconnect();
  }, [bootSrc]);

  // The instance announces itself once booted; that ends the skeleton.
  useEffect(() => {
    if (!bootSrc) return;
    const onMessage = (event: MessageEvent) => {
      if (isReadyMessage(event, iframeRef.current)) setIsLoaded(true);
    };
    window.addEventListener("message", onMessage);
    return () => window.removeEventListener("message", onMessage);
  }, [bootSrc]);

  const handleTabSelect = (tab: SpecimenTab) => {
    if (!bootSrc) {
      setActiveTab(tab);
      boot(tab);
      return;
    }
    if (tab.id === activeTab.id) return;
    setActiveTab(tab);
    // Messages sent before boot completes are queued by index.html.
    postPreview(iframeRef.current, tab.section, tab.slug);
  };

  return (
    <figure className="relative m-0 w-full min-w-0 max-w-2xl lg:max-w-none">
      <div className="product-visual" data-reveal>
        {/* Porabuild-spec Window Title Bar */}
        <div aria-hidden="true" className="window-bar">
          <div>
            <span />
            <span />
            <span />
          </div>
          <span className="font-mono text-[11px] tracking-[0.08em]">HEROGPUI / WEBASSEMBLY</span>
          <span className="shot-window-status flex items-center gap-1.5 font-mono text-[10px]">
            <span className="size-1.5 rounded-full bg-accent animate-pulse" />
            LIVE RUNTIME
          </span>
        </div>

        {/* Specimen Switcher Toolbar */}
        <div className="flex items-center justify-between gap-2 border-b border-separator/70 bg-surface-secondary/60 px-3 py-1.5 backdrop-blur-sm">
          <div
            className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
            onScroll={updateScrollFade}
            ref={stripRef}
            style={
              canScrollRight
                ? {
                    maskImage: "linear-gradient(90deg, #000 calc(100% - 28px), transparent)",
                    WebkitMaskImage: "linear-gradient(90deg, #000 calc(100% - 28px), transparent)",
                  }
                : undefined
            }
          >
            {SPECIMEN_TABS.map((tab) => {
              const active = tab.id === activeTab.id;
              return (
                <button
                  aria-pressed={active}
                  className={cn(
                    "cursor-pointer rounded-md px-2.5 py-1 font-mono text-[11px] font-medium whitespace-nowrap transition-all",
                    active
                      ? "bg-surface text-accent shadow-xs"
                      : "text-muted hover:bg-surface/50 hover:text-foreground",
                  )}
                  key={tab.id}
                  onClick={() => handleTabSelect(tab)}
                  onFocus={warm}
                  onPointerEnter={warm}
                  onTouchStart={warm}
                  type="button"
                >
                  {tab.label}
                </button>
              );
            })}
          </div>

          <span className="hidden shrink-0 font-mono text-[10px] text-muted/70 2xl:inline-block">
            GPUI · wgpu · WebAssembly
          </span>
        </div>

        {/* Canvas / Iframe Viewport */}
        <div className="relative h-[300px] w-full overflow-hidden bg-surface-secondary sm:h-[400px] lg:h-[440px]">
          {!bootSrc ? (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 bg-surface-secondary text-muted">
              <button
                className="cursor-pointer rounded-lg border border-separator bg-surface px-4 py-2 text-sm font-medium text-foreground transition-colors hover:border-accent"
                onClick={() => boot(activeTab)}
                onFocus={warm}
                onPointerEnter={warm}
                onTouchStart={warm}
                type="button"
              >
                Run the live demo
              </button>
              <span className="max-w-xs text-center text-xs text-muted/80">
                Loads HeroGPUI compiled to WebAssembly (about 5 MB, cached afterwards).
              </span>
            </div>
          ) : null}

          {bootSrc && !isLoaded ? (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-surface-secondary text-muted">
              <span className="size-2 rounded-full bg-accent animate-ping" />
              <span className="font-mono text-xs text-muted/80">
                Loading the {activeTab.label} example
              </span>
            </div>
          ) : null}

          {bootSrc ? (
            <iframe
              className={cn(
                "absolute inset-0 h-full w-full border-0 transition-opacity duration-300",
                isLoaded ? "opacity-100" : "opacity-0",
              )}
              ref={iframeRef}
              src={bootSrc}
              title={`HeroGPUI ${activeTab.label} live WebAssembly specimen`}
            />
          ) : null}
        </div>
      </div>

      {/* Caption & External link */}
      <figcaption className="mt-3 flex flex-wrap items-center justify-between gap-2 text-xs text-muted">
        <span className="flex items-center gap-2">
          <span
            aria-hidden="true"
            className="size-1.5 shrink-0 rounded-full bg-accent shadow-[0_0_8px_var(--pb-accent-glow)]"
          />
          HeroGPUI compiled to WebAssembly, running in the browser.
        </span>
        <a
          className="font-mono text-[11px] text-muted transition-colors hover:text-accent no-underline"
          href={publicUrl("/gallery/index.html")}
          rel="noopener noreferrer"
          target="_blank"
        >
          Open standalone gallery ↗
        </a>
      </figcaption>
    </figure>
  );
}
