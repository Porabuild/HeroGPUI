"use client";

import { cn } from "@heroui/react";
import { Search } from "lucide-react";
import type { Route } from "next";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { SearchItem } from "@/lib/docs-nav";

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

function matches(item: SearchItem, query: string): boolean {
  const haystack = `${item.title} ${item.group} ${item.keywords}`.toLowerCase();
  return query.split(/\s+/).every((part) => haystack.includes(part));
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
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [mounted, setMounted] = useState(false);
  const [shortcut, setShortcut] = useState("Ctrl K");

  const results = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (q.length === 0) return items.slice(0, 12);
    return items.filter((item) => matches(item, q)).slice(0, 20);
  }, [items, query]);

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
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
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
                  aria-label="Search pages and components"
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
                    } else if (event.key === "Enter" && results[active]) {
                      event.preventDefault();
                      go(results[active].href);
                    }
                  }}
                  placeholder="Search components and docs…"
                  ref={inputRef}
                  value={query}
                />
              </div>
              <ul className="max-h-80 overflow-auto py-2">
                {results.length === 0 ? (
                  <li className="px-4 py-6 text-sm text-muted">No matching pages.</li>
                ) : (
                  results.map((item, index) => (
                    <li key={`${item.group}-${item.href}`}>
                      <button
                        className={cn(
                          "flex w-full items-center justify-between gap-3 px-4 py-2 text-left text-sm",
                          index === active
                            ? "bg-accent-soft text-accent-soft-foreground"
                            : "text-foreground",
                        )}
                        onClick={() => go(item.href)}
                        onMouseEnter={() => setActive(index)}
                        type="button"
                      >
                        <span className="truncate font-medium">{item.title}</span>
                        <span className="shrink-0 font-mono text-[10px] uppercase tracking-wide text-muted">
                          {item.group}
                        </span>
                      </button>
                    </li>
                  ))
                )}
              </ul>
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
