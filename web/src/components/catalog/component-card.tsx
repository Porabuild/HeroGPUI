import { cn, Link } from "@heroui/react";
import type { CatalogComponent } from "@/lib/catalog";
import { publicUrl } from "@/lib/public-url";

/**
 * One card in the components index. The whole card is a HeroUI `Link` so the
 * shell's RouterProvider navigates client-side to `/docs/components/<slug>`.
 * Title is intentionally not a heading — the docs table of contents collects
 * every h2/h3 in the article, and component titles would drown out the
 * category headings.
 */

const STAGE =
  "docs-stage relative aspect-[16/10] w-full overflow-hidden rounded-xl border border-separator";

function ComponentShot({ component }: { component: CatalogComponent }) {
  const light = component.tile;
  const dark = component.tileDark;

  if (!light) {
    return (
      <div className={cn(STAGE, "flex items-center justify-center")} role="presentation">
        <span aria-hidden="true" className="text-3xl font-semibold text-muted">
          {component.title.charAt(0).toUpperCase()}
        </span>
      </div>
    );
  }

  return (
    <div className={STAGE}>
      <img
        alt=""
        className={cn("absolute inset-0 h-full w-full object-cover", dark && "dark:hidden")}
        decoding="async"
        loading="lazy"
        src={publicUrl(light)}
      />
      {dark ? (
        <img
          alt=""
          className="absolute inset-0 hidden h-full w-full object-cover dark:block"
          decoding="async"
          loading="lazy"
          src={publicUrl(dark)}
        />
      ) : null}
    </div>
  );
}

export function ComponentCard({ component }: { component: CatalogComponent }) {
  return (
    <li className="h-full">
      <Link
        className={cn(
          "catalog-card group flex h-full w-full flex-col items-stretch rounded-xl border border-separator bg-surface p-4",
          "no-underline transition-[border-color,box-shadow,color]",
          "hover:border-accent/50 hover:no-underline",
        )}
        href={`/docs/components/${component.slug}`}
      >
        <ComponentShot component={component} />
        <span className="mt-3 line-clamp-1 min-h-[1.25rem] text-sm font-semibold text-foreground transition-colors group-hover:text-accent">
          {component.title}
        </span>
        <span className="mt-1 line-clamp-2 min-h-[2.75rem] flex-1 text-sm leading-relaxed text-muted">
          {component.description}
        </span>
      </Link>
    </li>
  );
}
