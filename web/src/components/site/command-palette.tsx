"use client";

import { cn } from "@heroui/react";
import { Search } from "lucide-react";
import type { Route } from "next";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { SearchItem } from "@/lib/docs-nav";
import { isSearchShortcutMessage } from "@/lib/gallery-embed";

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

/** Case-, space-, `_`- and `-`-insensitive form, so `selectionMode` finds `selection_mode`. */
function squash(text: string): string {
  return text.toLowerCase().replace(/[\s_-]+/g, "");
}

interface SearchResult {
  item: SearchItem;
  /** Where the result navigates: the builder table when a builder matched. */
  href: string;
  /** The Rust builder the query matched, shown beside the title. */
  builder: string | null;
  /** Lower ranks first: title, then page keywords, then builder names. */
  rank: number;
}

function matchItem(item: SearchItem, parts: string[]): SearchResult | null {
  const title = item.title.toLowerCase();
  const haystack = `${title} ${item.group} ${item.keywords}`.toLowerCase();
  let rank = 0;
  let builder: string | null = null;
  for (const part of parts) {
    if (title.includes(part)) {
      rank = Math.max(rank, title.startsWith(part) ? 0 : 1);
      continue;
    }
    if (haystack.includes(part)) {
      rank = Math.max(rank, 2);
      continue;
    }
    const wanted = squash(part);
    const builders = item.builders ?? [];
    const found =
      builders.find((name) => squash(name) === wanted) ??
      builders.find((name) => squash(name).startsWith(wanted)) ??
      builders.find((name) => squash(name).includes(wanted));
    if (!found) return null;
    builder ??= found;
    rank = Math.max(rank, squash(found) === wanted ? 3 : 4);
  }
  return { item, builder, rank, href: builder ? `${item.href}#props` : item.href };
}

function search(items: SearchItem[], query: string): SearchResult[] {
  const parts = query.split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return items.slice(0, 12).map((item) => ({ item, href: item.href, builder: null, rank: 0 }));
  }
  return items
    .flatMap((item) => {
      const result = matchItem(item, parts);
      return result ? [result] : [];
    })
    .sort((a, b) => a.rank - b.rank)
    .slice(0, 20);
}

function focusableElements(root: HTMLElement): HTMLElement[] {
  return [
    ...root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ].filter((node) => node.tabIndex !== -1 && !node.closest("[hidden]"));
}

export function CommandPalette({ items }: { items: SearchItem[] }) {
  const router = useRouter();
  const inputRef = useRef<HTMLInputElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const titleId = useId();
  const listboxId = useId();
  const optionId = (index: number) => `${listboxId}-option-${index}`;
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [mounted, setMounted] = useState(false);
  const [shortcut, setShortcut] = useState("Ctrl K");

  const results = useMemo(() => search(items, query.trim().toLowerCase()), [items, query]);

  const close = useCallback(() => {
    setOpen(false);
    setQuery("");
    setActive(0);
    window.requestAnimationFrame(() => triggerRef.current?.focus());
  }, []);

  const go = useCallback(
    (href: string) => {
      close();
      router.push(href as Route);
    },
    [close, router],
  );

  useEffect(() => {
    setMounted(true);
    if (/Mac|iPhone|iPad/.test(navigator.platform)) setShortcut("⌘K");
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const metaK = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k";
      if (metaK) {
        event.preventDefault();
        if (open) close();
        else setOpen(true);
        return;
      }
      if (event.key === "Escape" && open) {
        event.preventDefault();
        close();
        return;
      }
      if (event.key !== "/" || event.metaKey || event.ctrlKey || event.altKey) return;
      if (isTypingTarget(event.target) || open) return;
      event.preventDefault();
      const catalog = document.getElementById("catalog-search");
      if (catalog instanceof HTMLInputElement) {
        catalog.focus();
        return;
      }
      setOpen(true);
    };
    // An embedded gallery the reader has clicked into holds keyboard focus,
    // so it forwards Cmd/Ctrl+K here instead of swallowing it.
    const onMessage = (event: MessageEvent) => {
      if (!isSearchShortcutMessage(event)) return;
      if (open) close();
      else setOpen(true);
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("message", onMessage);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("message", onMessage);
    };
  }, [close, open]);

  useEffect(() => {
    if (!open) return;
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const id = window.requestAnimationFrame(() => inputRef.current?.focus());
    return () => {
      document.body.style.overflow = prevOverflow;
      window.cancelAnimationFrame(id);
    };
  }, [open]);

  useEffect(() => {
    setActive(0);
  }, [query]);

  // Keep the active option visible while arrowing through a long list.
  useEffect(() => {
    if (!open) return;
    document.getElementById(`${listboxId}-option-${active}`)?.scrollIntoView({ block: "nearest" });
  }, [active, listboxId, open]);

  const activeResult = results[active] ?? null;

  const dialog =
    open && mounted
      ? createPortal(
          <div className="fixed inset-0 z-50 flex items-start justify-center px-4 pt-[15vh]">
            <button
              aria-label="Close search"
              className="absolute inset-0 bg-background/70 backdrop-blur-sm"
              onClick={close}
              tabIndex={-1}
              type="button"
            />
            <div
              aria-labelledby={titleId}
              aria-modal="true"
              className="relative z-10 w-full max-w-xl overflow-hidden rounded-xl border border-separator bg-surface shadow-xl"
              onKeyDown={(event) => {
                if (event.key !== "Tab" || !panelRef.current) return;
                const nodes = focusableElements(panelRef.current);
                if (nodes.length === 0) return;
                const first = nodes[0];
                const last = nodes[nodes.length - 1];
                if (event.shiftKey && document.activeElement === first) {
                  event.preventDefault();
                  last.focus();
                } else if (!event.shiftKey && document.activeElement === last) {
                  event.preventDefault();
                  first.focus();
                }
              }}
              ref={panelRef}
              role="dialog"
            >
              <h2 className="sr-only" id={titleId}>
                Search the documentation
              </h2>
              <div className="flex items-center gap-2 border-b border-separator px-3">
                <Search aria-hidden="true" className="size-4 text-muted" />
                <input
                  aria-activedescendant={activeResult ? optionId(active) : undefined}
                  aria-autocomplete="list"
                  aria-controls={listboxId}
                  aria-expanded={results.length > 0}
                  aria-label="Search pages, components and builders"
                  autoComplete="off"
                  className="h-12 w-full bg-transparent text-sm text-foreground outline-none placeholder:text-muted"
                  onChange={(event) => setQuery(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "ArrowDown") {
                      event.preventDefault();
                      setActive((index) => Math.min(index + 1, Math.max(results.length - 1, 0)));
                    } else if (event.key === "ArrowUp") {
                      event.preventDefault();
                      setActive((index) => Math.max(index - 1, 0));
                    } else if (event.key === "Home" && results.length > 0) {
                      event.preventDefault();
                      setActive(0);
                    } else if (event.key === "End" && results.length > 0) {
                      event.preventDefault();
                      setActive(results.length - 1);
                    } else if (event.key === "Enter" && activeResult) {
                      event.preventDefault();
                      go(activeResult.href);
                    }
                  }}
                  placeholder="Search components, docs and builders…"
                  ref={inputRef}
                  role="combobox"
                  spellCheck={false}
                  type="text"
                  value={query}
                />
              </div>
              <ul
                aria-label="Search results"
                className={cn("max-h-80 overflow-auto py-2", results.length === 0 && "hidden")}
                id={listboxId}
                role="listbox"
              >
                {results.map((result, index) => (
                  <li
                    aria-selected={index === active}
                    className={cn(
                      "flex w-full cursor-pointer items-center justify-between gap-3 px-4 py-2 text-left text-sm",
                      index === active
                        ? "bg-accent-soft text-accent-soft-foreground"
                        : "text-foreground",
                    )}
                    id={optionId(index)}
                    key={`${result.item.group}-${result.href}`}
                    onClick={() => go(result.href)}
                    onMouseDown={(event) => event.preventDefault()}
                    onMouseMove={() => setActive(index)}
                    role="option"
                  >
                    <span className="flex min-w-0 items-baseline gap-2">
                      <span className="truncate font-medium">{result.item.title}</span>
                      {result.builder ? (
                        <code className="truncate font-mono text-xs text-muted">
                          .{result.builder}()
                        </code>
                      ) : null}
                    </span>
                    <span className="shrink-0 font-mono text-[10px] uppercase tracking-wide text-muted">
                      {result.item.group}
                    </span>
                  </li>
                ))}
              </ul>
              {results.length === 0 ? (
                <p className="px-4 pb-6 pt-4 text-sm text-muted">No matching pages.</p>
              ) : null}
              <p aria-live="polite" className="sr-only">
                {query.trim() === ""
                  ? ""
                  : results.length === 0
                    ? "No results"
                    : `${results.length} result${results.length === 1 ? "" : "s"}`}
              </p>
            </div>
          </div>,
          document.body,
        )
      : null;

  return (
    <>
      <button
        aria-expanded={open}
        aria-haspopup="dialog"
        aria-label="Search documentation"
        className={cn(
          "site-search-trigger inline-flex h-9 items-center gap-2 rounded-md border px-2.5",
          open ? "border-accent/60 text-foreground" : "border-separator",
        )}
        onClick={() => (open ? close() : setOpen(true))}
        ref={triggerRef}
        type="button"
      >
        <Search aria-hidden="true" className="size-3.5" />
        <span className="hidden text-xs sm:inline">Search</span>
        <kbd className="hidden rounded border border-separator px-1.5 py-0.5 font-mono text-[10px] text-muted md:inline">
          {shortcut}
        </kbd>
      </button>
      {dialog}
    </>
  );
}
