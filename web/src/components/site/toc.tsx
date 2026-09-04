"use client";

import { cn } from "@heroui/react";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

interface TocItem {
  id: string;
  text: string;
  level: 2 | 3;
}

function slugify(text: string): string {
  return (
    text
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9\s-]/g, "")
      .replace(/\s+/g, "-")
      .replace(/-+/g, "-") || "section"
  );
}

/**
 * A heading's text, with its element children kept apart. `textContent` runs
 * every text node together, so the release log's
 * `<h3>August 29, 2026<span>5 commits</span></h3>` came out of it as
 * "August 29, 20265 commits". Only an element boundary earns a space: React
 * splits interpolated text with comment nodes, so `What v{version} contains`
 * arrives as three text nodes and has to stay "What v0.1.0 contains" rather
 * than becoming "What v 0.1.0 contains". Comments are skipped outright — a
 * React `<!-- -->` marker's own textContent is a space.
 */
function headingText(heading: HTMLElement): string {
  let text = "";
  for (const node of heading.childNodes) {
    if (node.nodeType === Node.ELEMENT_NODE) text += ` ${node.textContent ?? ""} `;
    else if (node.nodeType === Node.TEXT_NODE) text += node.nodeValue ?? "";
  }
  return text.replace(/\s+/g, " ").trim();
}

/**
 * "On this page" list built from the article's h2/h3 elements, with
 * scroll-spy highlighting. Headings without an id get one assigned so the
 * links always work. Hidden below `xl` by the docs layout.
 */
export function Toc({ articleSelector = "[data-docs-article]" }: { articleSelector?: string }) {
  const pathname = usePathname();
  const [items, setItems] = useState<TocItem[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);

  // Collect headings for the current page, and keep collecting while the
  // article's DOM changes (e.g. the components index filters its list).
  useEffect(() => {
    const collect = () => {
      const article = document.querySelector(articleSelector);
      if (!article) {
        setItems([]);
        return;
      }
      const headings = Array.from(article.querySelectorAll<HTMLElement>("h2, h3"));
      const entries = headings.map((heading) => {
        const text = headingText(heading);
        if (!heading.id) heading.id = slugify(text);
        return {
          id: heading.id,
          text,
          level: heading.tagName === "H2" ? (2 as const) : (3 as const),
        };
      });
      const next = entries.filter((entry) => entry.text.length > 0);
      // Keep the previous array when nothing changed so the scroll-spy
      // effect below does not re-arm on unrelated mutations.
      setItems((prev) =>
        prev.length === next.length &&
        prev.every((item, index) => item.id === next[index].id && item.text === next[index].text)
          ? prev
          : next,
      );
    };

    collect();

    const article = document.querySelector(articleSelector);
    if (!article) return;

    // Debounce bursts of mutations (one per keystroke of a filter field).
    let timer = 0;
    const schedule = () => {
      window.clearTimeout(timer);
      timer = window.setTimeout(collect, 120);
    };

    const observer = new MutationObserver(schedule);
    // childList only: collect() assigns heading ids, which must not re-trigger.
    observer.observe(article, { childList: true, subtree: true });
    return () => {
      observer.disconnect();
      window.clearTimeout(timer);
    };
  }, [articleSelector, pathname]);

  // Scroll-spy: the last heading above the reading line is active.
  useEffect(() => {
    if (items.length === 0) return;
    let frame = 0;
    const update = () => {
      frame = 0;
      const readingLine = 120;
      let current: string | null = items[0].id;
      for (const item of items) {
        const element = document.getElementById(item.id);
        if (element && element.getBoundingClientRect().top <= readingLine) current = item.id;
      }
      const atBottom =
        window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 4;
      if (atBottom) current = items[items.length - 1].id;
      setActiveId(current);
    };
    const onScroll = () => {
      if (frame === 0) frame = window.requestAnimationFrame(update);
    };
    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [items]);

  if (items.length === 0) return null;

  return (
    <aside className="docs-toc hidden w-64 shrink-0 xl:block">
      <nav
        aria-label="On this page"
        className="scrollbar sticky top-16 max-h-[calc(100dvh-4rem)] overflow-y-auto py-10 pl-10"
      >
        <h3 className="docs-toc-label text-xs font-semibold tracking-wider text-muted uppercase">
          On this page
        </h3>
        <ul className="mt-3 space-y-1 border-l border-separator">
          {items.map((item) => {
            const active = item.id === activeId;
            return (
              <li key={item.id}>
                <a
                  aria-current={active ? "location" : undefined}
                  className={cn(
                    "-ml-px block border-l py-1 text-sm transition-colors",
                    item.level === 3 && "pl-7",
                    item.level === 2 && "pl-4",
                    active
                      ? "border-accent font-medium text-accent"
                      : "border-transparent text-muted hover:border-separator hover:text-foreground",
                  )}
                  href={`#${item.id}`}
                >
                  {item.text}
                </a>
              </li>
            );
          })}
        </ul>
      </nav>
    </aside>
  );
}
