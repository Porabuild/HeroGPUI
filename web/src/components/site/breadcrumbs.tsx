import { Breadcrumbs } from "@heroui/react";

export interface Crumb {
  label: string;
  /** Omit for the current (last) crumb. */
  href?: string;
}

/**
 * Breadcrumb trail for docs pages, built on HeroUI's Breadcrumbs. React Aria
 * treats the last item as the current page automatically (aria-current, no
 * separator), so only earlier crumbs need an href. Internal hrefs navigate
 * client-side via the site's RouterProvider.
 */
export function BreadcrumbsTrail({ items, className }: { items: Crumb[]; className?: string }) {
  if (items.length === 0) return null;
  const last = items.length - 1;
  return (
    <Breadcrumbs aria-label="Breadcrumbs" className={className}>
      {items.map((item, index) => {
        const current = index === last;
        return (
          <Breadcrumbs.Item
            className={current ? "text-sm font-medium text-foreground" : "text-sm text-muted"}
            href={current ? undefined : item.href}
            key={item.href ?? item.label}
          >
            {item.label}
          </Breadcrumbs.Item>
        );
      })}
    </Breadcrumbs>
  );
}
