"use client";

import { Button, Chip, EmptyState, SearchField } from "@heroui/react";
import { SearchX } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { ComponentCard } from "@/components/catalog/component-card";
import type { CatalogComponent } from "@/lib/catalog";

export interface CatalogGroup {
  name: string;
  slug: string;
  components: CatalogComponent[];
}

interface ComponentCatalogProps {
  groups: CatalogGroup[];
  /** Total component count across all groups, straight from the catalog. */
  totalCount: number;
}

/**
 * The interactive half of the components index: a live filter by title and
 * description, and one card grid per category. It is the only client
 * component on the page — everything else is server-rendered from the catalog.
 */
function queryFromLocation(): string {
  if (typeof window === "undefined") return "";
  return new URLSearchParams(window.location.search).get("q") ?? "";
}

export function ComponentCatalog({ groups, totalCount }: ComponentCatalogProps) {
  const [query, setQuery] = useState("");
  const skipUrlWrite = useRef(true);

  useEffect(() => {
    const initial = queryFromLocation();
    if (initial) {
      setQuery(initial);
    } else {
      skipUrlWrite.current = false;
    }
  }, []);

  useEffect(() => {
    if (skipUrlWrite.current) {
      skipUrlWrite.current = false;
      return;
    }
    const url = new URL(window.location.href);
    const next = query.trim();
    if (next) url.searchParams.set("q", next);
    else url.searchParams.delete("q");
    const nextHref = `${url.pathname}${url.search}${url.hash}`;
    const currentHref = `${window.location.pathname}${window.location.search}${window.location.hash}`;
    if (nextHref === currentHref) return;
    window.history.replaceState(window.history.state, "", nextHref);
  }, [query]);

  useEffect(() => {
    const scrollToHash = () => {
      const id = window.location.hash.replace(/^#/, "");
      if (!id) return;
      document.getElementById(id)?.scrollIntoView();
    };
    scrollToHash();
    window.addEventListener("hashchange", scrollToHash);
    return () => window.removeEventListener("hashchange", scrollToHash);
  }, []);

  const normalizedQuery = query.trim().toLowerCase();

  const visibleGroups = useMemo(() => {
    return groups
      .map((group) => ({
        ...group,
        components: group.components.filter((component) => {
          if (normalizedQuery.length === 0) return true;
          return (
            component.title.toLowerCase().includes(normalizedQuery) ||
            component.description.toLowerCase().includes(normalizedQuery)
          );
        }),
      }))
      .filter((group) => group.components.length > 0);
  }, [groups, normalizedQuery]);

  const visibleCount = visibleGroups.reduce((sum, group) => sum + group.components.length, 0);
  const isFiltering = normalizedQuery.length > 0;

  return (
    <div>
      <div>
        <SearchField
          aria-label="Filter components by name or description"
          className="w-full"
          value={query}
          onChange={setQuery}
        >
          <SearchField.Group>
            <SearchField.SearchIcon />
            <SearchField.Input id="catalog-search" placeholder="Filter components…  / to focus" />
            <SearchField.ClearButton />
          </SearchField.Group>
        </SearchField>
      </div>

      <p aria-live="polite" className="mt-4 text-sm text-muted">
        {isFiltering ? (
          <>
            {visibleCount} of {totalCount} components
          </>
        ) : (
          <>
            {totalCount} components in {groups.length} categories
          </>
        )}
      </p>

      {visibleCount === 0 ? (
        <EmptyState className="mt-10 rounded-xl border border-dashed border-separator py-12">
          <div className="flex flex-col items-center gap-3 text-center">
            <SearchX aria-hidden="true" className="size-6 text-muted" />
            <p className="text-sm font-medium text-foreground">
              No components match &ldquo;{query.trim()}&rdquo;
            </p>
            <p className="max-w-sm text-sm text-muted">
              Try a shorter query, or clear the search to see all {totalCount} components.
            </p>
            <Button onPress={() => setQuery("")} size="sm" variant="outline">
              Clear search
            </Button>
          </div>
        </EmptyState>
      ) : (
        visibleGroups.map((group) => (
          <section
            key={group.slug}
            aria-labelledby={`category-${group.slug}`}
            className="catalog-category mt-10"
          >
            <div className="catalog-category-heading flex items-center gap-3">
              <h2 className="mt-0" id={`category-${group.slug}`}>
                {group.name}
              </h2>
              <Chip color="default" size="sm" variant="soft">
                {group.components.length}
              </Chip>
            </div>
            <ul className="catalog-grid mt-4 grid list-none grid-cols-1 items-stretch gap-4 ps-0 sm:grid-cols-2">
              {group.components.map((component) => (
                <ComponentCard component={component} key={component.slug} />
              ))}
            </ul>
          </section>
        ))
      )}
    </div>
  );
}
