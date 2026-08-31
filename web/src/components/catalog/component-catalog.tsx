"use client";

import { Button, Chip, EmptyState, SearchField, Switch, cn } from "@heroui/react";
import { SearchX } from "lucide-react";
import { useMemo, useState } from "react";
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
 * description, an optional restriction to components with full API reference
 * data, and one card grid per category. It is the only client component on
 * the page — everything else is server-rendered from the catalog.
 */
export function ComponentCatalog({ groups, totalCount }: ComponentCatalogProps) {
  const [query, setQuery] = useState("");
  const [referenceOnly, setReferenceOnly] = useState(false);

  const normalizedQuery = query.trim().toLowerCase();

  const visibleGroups = useMemo(() => {
    return groups
      .map((group) => ({
        ...group,
        components: group.components.filter((component) => {
          if (referenceOnly && !component.hasReference) return false;
          if (normalizedQuery.length === 0) return true;
          return (
            component.title.toLowerCase().includes(normalizedQuery) ||
            component.description.toLowerCase().includes(normalizedQuery)
          );
        }),
      }))
      .filter((group) => group.components.length > 0);
  }, [groups, normalizedQuery, referenceOnly]);

  const visibleCount = visibleGroups.reduce((sum, group) => sum + group.components.length, 0);
  const isFiltering = normalizedQuery.length > 0 || referenceOnly;

  const resetFilters = () => {
    setQuery("");
    setReferenceOnly(false);
  };

  return (
    <div>
      {/*
       * Filter controls. Live region + count so screen readers hear the
       * result size change as the query is typed.
       */}
      <div className="catalog-filters flex flex-wrap items-center justify-between gap-x-6 gap-y-4">
        <SearchField
          aria-label="Filter components by name or description"
          className="w-full sm:max-w-xs"
          value={query}
          onChange={setQuery}
        >
          <SearchField.Group>
            <SearchField.SearchIcon />
            <SearchField.Input placeholder="Filter components…" />
            <SearchField.ClearButton />
          </SearchField.Group>
        </SearchField>
        <Switch isSelected={referenceOnly} onChange={setReferenceOnly}>
          <Switch.Content>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            Full reference only
          </Switch.Content>
        </Switch>
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

      {/*
       * Category jump-nav. Mirrors the visible sections: everything while
       * unfiltered, only matching categories while filtering, so no anchor
       * ever points at a heading that is not on the page.
       */}
      <nav aria-label="Jump to a category" className="catalog-category-nav mt-6">
        {/* list-none/ps-0: the docs article styles give plain uls bullets. */}
        <ul className="flex list-none flex-wrap gap-2 ps-0">
          {visibleGroups.map((group) => (
            <li key={group.slug}>
              <a
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-full border border-separator bg-surface px-3 py-1",
                  "text-xs font-medium text-muted transition-colors no-underline hover:bg-surface-secondary hover:text-foreground",
                  "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                )}
                href={`#category-${group.slug}`}
              >
                {group.name}
                <span className="font-mono text-[0.6875rem] text-muted">
                  {group.components.length}
                </span>
              </a>
            </li>
          ))}
        </ul>
      </nav>

      {visibleCount === 0 ? (
        <EmptyState className="mt-10 rounded-xl border border-dashed border-separator py-12">
          <div className="flex flex-col items-center gap-3 text-center">
            <SearchX aria-hidden="true" className="size-6 text-muted" />
            <p className="text-sm font-medium text-foreground">
              No components match &ldquo;{query.trim()}&rdquo;
              {referenceOnly ? " with full reference data" : ""}
            </p>
            <p className="max-w-sm text-sm text-muted">
              Try a shorter query, or clear the filters to see all {totalCount} components.
            </p>
            <Button onPress={resetFilters} size="sm" variant="outline">
              Clear filters
            </Button>
          </div>
        </EmptyState>
      ) : (
        visibleGroups.map((group) => (
          // mt-10 carries the section rhythm because the docs article styles
          // put the h2's top margin on the heading itself, which would shove
          // the h2 off-center inside this heading row.
          <section
            key={group.slug}
            aria-labelledby={`category-${group.slug}`}
            className="catalog-category mt-10"
          >
            <div className="flex items-center gap-3">
              <h2 className="mt-0" id={`category-${group.slug}`}>
                {group.name}
              </h2>
              <Chip color="default" size="sm" variant="soft">
                {group.components.length}
              </Chip>
            </div>
            <ul className="mt-4 grid list-none grid-cols-1 gap-4 ps-0 sm:grid-cols-2">
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
