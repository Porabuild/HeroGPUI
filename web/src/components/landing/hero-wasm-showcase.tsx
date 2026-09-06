"use client";

import { cn } from "@heroui/react";
import { useCallback, useEffect, useRef, useState } from "react";
import { publicUrl } from "@/lib/public-url";

const GALLERY_BASE = process.env.NEXT_PUBLIC_GALLERY_URL || "/gallery";
const ABSOLUTE_URL_RE = /^[a-z][a-z0-9+.-]*:\/\//i;

function readTheme(): "light" | "dark" {
  if (typeof document === "undefined") return "dark";
  return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

function galleryOrigin(base: string): string {
  return ABSOLUTE_URL_RE.test(base) ? base : publicUrl(base);
}

function embedUrl(slug: string, section: string, theme: "light" | "dark"): string {
  const base = galleryOrigin(GALLERY_BASE).replace(/\/+$/, "");
  const query = new URLSearchParams({
    preview: "component",
    section,
    story: slug,
    theme,
  }).toString();
  return `${base}/index.html?${query}`;
}

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

export function HeroWasmShowcase() {
  const [activeTab, setActiveTab] = useState<SpecimenTab>(SPECIMEN_TABS[0]);
  const [theme, setTheme] = useState<"light" | "dark">("dark");
  // The iframe is created only after mount: a server-rendered `src` would
  // start (and possibly finish) loading before hydration, so its `load`
  // event would fire with no React listener attached and the frame would
  // sit at opacity 0 behind the skeleton forever.
  const [mounted, setMounted] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement | null>(null);

  useEffect(() => {
    setMounted(true);
  }, []);

  // Sync with host page theme
  useEffect(() => {
    setTheme(readTheme());
    const observer = new MutationObserver(() => {
      const current = readTheme();
      setTheme(current);
      // Also broadcast theme change to iframe if alive
      const target = iframeRef.current?.contentWindow;
      if (target) {
        const origin = new URL(galleryOrigin(GALLERY_BASE), window.location.href).origin;
        target.postMessage({ type: "herogpui:set-theme", dark: current === "dark" }, origin);
      }
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class", "data-theme"],
    });

    return () => observer.disconnect();
  }, []);

  const sendSectionChange = useCallback((section: string) => {
    const target = iframeRef.current?.contentWindow;
    if (!target) return;
    const origin = new URL(galleryOrigin(GALLERY_BASE), window.location.href).origin;
    target.postMessage({ type: "herogpui:preview-section", section }, origin);
  }, []);

  const handleTabSelect = (tab: SpecimenTab) => {
    if (tab.id === activeTab.id) return;
    if (tab.slug === activeTab.slug) {
      // Same story: keep the one live frame and switch its section over
      // the message bridge instead of rebooting WebAssembly.
      setActiveTab(tab);
      sendSectionChange(tab.section);
    } else {
      setIsLoaded(false);
      setActiveTab(tab);
    }
  };

  const iframeSrc = embedUrl(activeTab.slug, activeTab.section, theme);

  // A fresh story or theme reboots the frame, so the skeleton returns
  // until the new document reports in.
  useEffect(() => {
    setIsLoaded(false);
  }, [activeTab.slug, theme]);

  const markLoaded = useCallback(
    (section: string) => {
      setIsLoaded(true);
      sendSectionChange(section);
    },
    [sendSectionChange],
  );

  // Second guarantee against a missed `load` event: once the frame exists,
  // treat `readyState === "complete"` as loaded. Same-origin only; a
  // cross-origin gallery throws on `contentDocument` access and keeps the
  // `onLoad` path.
  useEffect(() => {
    if (!mounted || isLoaded) return;
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
        markLoaded(activeTab.section);
      }
    }, 250);
    return () => window.clearInterval(timer);
  }, [mounted, isLoaded, iframeSrc, activeTab.section, markLoaded]);

  return (
    <figure className="relative m-0 w-full min-w-0 max-w-2xl lg:max-w-none">
      <div className="relative overflow-hidden rounded-xl border border-separator bg-surface shadow-2xl">
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
          <div className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
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
        <div className="relative h-[300px] w-full bg-surface-secondary sm:h-[400px] lg:h-[440px]">
          {/* Subtle loading skeleton before iframe loads */}
          {!isLoaded && (
            <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-surface-secondary text-muted">
              <span className="size-2 rounded-full bg-accent animate-ping" />
              <span className="font-mono text-xs text-muted/80">
                Loading the {activeTab.label} example
              </span>
            </div>
          )}

          {mounted && (
            <iframe
              className={cn(
                "absolute inset-0 h-full w-full border-0 transition-opacity duration-300",
                isLoaded ? "opacity-100" : "opacity-0",
              )}
              key={`${activeTab.slug}-${theme}`}
              onLoad={() => markLoaded(activeTab.section)}
              ref={iframeRef}
              src={iframeSrc}
              title={`HeroGPUI ${activeTab.label} live WebAssembly specimen`}
            />
          )}
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
          href={publicUrl(`/gallery/index.html?theme=${theme}`)}
          rel="noopener noreferrer"
          target="_blank"
        >
          Open standalone gallery ↗
        </a>
      </figcaption>
    </figure>
  );
}
