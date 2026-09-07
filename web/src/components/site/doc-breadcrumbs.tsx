"use client";

import { Link } from "@heroui/react";
import { usePathname } from "next/navigation";
import type { Catalog } from "@/lib/catalog";
import { breadcrumbsFor } from "@/lib/docs-nav";

export function DocBreadcrumbs({ catalog }: { catalog: Catalog }) {
  const pathname = usePathname();
  const crumbs = breadcrumbsFor(pathname, catalog);
  if (crumbs.length <= 1) return null;

  return (
    <nav aria-label="Breadcrumb" className="mb-6">
      {/* Opt out of the article prose list rules: the indent and sibling
          margins would offset this flex row left and vertically. */}
      <ol className="m-0 flex flex-wrap list-none items-center gap-1.5 ps-0 text-xs text-muted">
        {crumbs.map((crumb, index) => {
          const last = index === crumbs.length - 1;
          return (
            <li className="mt-0 flex items-center gap-1.5" key={`${crumb.href}-${crumb.label}`}>
              {index > 0 ? <span aria-hidden="true">/</span> : null}
              {last ? (
                <span aria-current="page" className="text-foreground">
                  {crumb.label}
                </span>
              ) : (
                <Link
                  className="text-muted no-underline transition-colors hover:text-foreground"
                  href={crumb.href}
                >
                  {crumb.label}
                </Link>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
