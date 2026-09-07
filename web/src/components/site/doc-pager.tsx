"use client";

import { Link } from "@heroui/react";
import { ArrowLeft, ArrowRight } from "lucide-react";
import { usePathname } from "next/navigation";
import type { Catalog } from "@/lib/catalog";
import { pagerFor } from "@/lib/docs-nav";

export function DocPager({ catalog }: { catalog: Catalog }) {
  const pathname = usePathname();
  const pager = pagerFor(pathname, catalog);
  if (!pager || (!pager.prev && !pager.next)) return null;

  return (
    <nav
      aria-label="Adjacent pages"
      className="mt-16 grid grid-cols-1 gap-3 border-t border-separator pt-6 sm:grid-cols-2"
    >
      {pager.prev ? (
        <Link
          className="group flex min-h-16 w-full flex-col justify-center rounded-xl border border-separator px-4 py-3 no-underline transition-colors hover:border-accent/50 hover:no-underline"
          href={pager.prev.href}
        >
          <span className="flex items-center gap-1.5 text-[11px] font-medium tracking-wide text-muted uppercase">
            <ArrowLeft aria-hidden="true" className="size-3.5" />
            Previous
          </span>
          <span className="mt-1 text-sm font-medium text-foreground group-hover:text-accent">
            {pager.prev.label}
          </span>
        </Link>
      ) : (
        <span className="hidden sm:block" />
      )}
      {pager.next ? (
        <Link
          className="group flex min-h-16 w-full flex-col items-end justify-center rounded-xl border border-separator px-4 py-3 text-right no-underline transition-colors hover:border-accent/50 hover:no-underline"
          href={pager.next.href}
        >
          <span className="flex items-center gap-1.5 text-[11px] font-medium tracking-wide text-muted uppercase">
            Next
            <ArrowRight aria-hidden="true" className="size-3.5" />
          </span>
          <span className="mt-1 text-sm font-medium text-foreground group-hover:text-accent">
            {pager.next.label}
          </span>
        </Link>
      ) : null}
    </nav>
  );
}
