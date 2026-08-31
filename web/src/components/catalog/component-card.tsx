import { Chip, cn, Link } from "@heroui/react";
import type { CSSProperties } from "react";
import type { CatalogComponent } from "@/lib/catalog";
import { publicUrl } from "@/lib/public-url";

/**
 * One card in the components index. The whole card is a HeroUI `Link` (so the
 * shell's RouterProvider navigates client-side to `/docs/components/<slug>`)
 * styled as the card surface itself. Title is intentionally not a heading —
 * the docs table of contents collects every h2/h3 in the article, and 66
 * component titles would drown out the category headings.
 */

/** Source captures are full 1200x1392 gallery-window shots. */
const SHOT_WIDTH = 1200;
const SHOT_HEIGHT = 1392;

/**
 * Four fifths of a capture is window furniture, which left the component it
 * advertises a few unreadable pixels wide once the shot was scaled into a
 * card. These offsets crop it to the window's content pane, and were measured
 * off the real files rather than estimated: a horizontal scan across any
 * capture finds the sidebar's trailing separator at x=239, a vertical scan
 * finds the top nav's at y=129, and the last drawn content column is 1139.
 * All three are identical on every card capture.
 */
const CONTENT_LEFT = 240;
const CONTENT_TOP = 130;
const CROP_WIDTH = 900;
/** The frame is `aspect-[16/10]`, so the crop has to be too. */
const CROP_HEIGHT = (CROP_WIDTH * 10) / 16;

/**
 * Scale the untouched image so `CROP_WIDTH` source pixels span the frame, then
 * shift the crop's top-left corner onto the frame's. Every percentage resolves
 * against the frame, whose box is exactly the crop, so the arithmetic holds at
 * any card width. `max-w-none` is required — preflight's `max-width: 100%`
 * would otherwise clamp the scaled width straight back down.
 */
const CROP_STYLE: CSSProperties = {
  height: `${(SHOT_HEIGHT / CROP_HEIGHT) * 100}%`,
  left: `${(-CONTENT_LEFT / CROP_WIDTH) * 100}%`,
  top: `${(-CONTENT_TOP / CROP_HEIGHT) * 100}%`,
  width: `${(SHOT_WIDTH / CROP_WIDTH) * 100}%`,
};

/**
 * The captures are light-themed. Bleeding one to the edge of a dark card read
 * as a rendering fault, so the shot sits inside a hairline mat instead — the
 * way a screenshot is framed rather than embedded. The radii nest exactly:
 * 8px outer minus the 4px mat leaves 4px inside.
 */
const MAT = "w-full shrink-0 rounded-lg border border-separator bg-surface-secondary p-1";
const PANE = "relative aspect-[16/10] w-full overflow-hidden rounded-sm bg-surface-secondary";

function ComponentShot({ component }: { component: CatalogComponent }) {
  // The data contract allows a missing capture; a quiet monogram keeps the
  // grid intact instead of a broken image.
  if (!component.shot) {
    return (
      <div className={MAT} role="presentation">
        <div className={cn(PANE, "flex flex-col items-center justify-center gap-1")}>
          <span aria-hidden="true" className="text-3xl font-semibold text-muted">
            {component.title.charAt(0).toUpperCase()}
          </span>
          <span className="text-xs text-muted">No native capture yet</span>
        </div>
      </div>
    );
  }

  return (
    <div className={MAT}>
      <div className={PANE}>
        <img
          alt={`${component.title} rendered in the HeroGPUI gallery`}
          className="absolute max-w-none"
          decoding="async"
          height={SHOT_HEIGHT}
          loading="lazy"
          src={publicUrl(component.shot)}
          style={CROP_STYLE}
          width={SHOT_WIDTH}
        />
      </div>
    </div>
  );
}

function demoLabel(count: number): string {
  return count === 1 ? "1 demo" : `${count} demos`;
}

export function ComponentCard({ component }: { component: CatalogComponent }) {
  return (
    <li className="h-full">
      <Link
        className={cn(
          // items-stretch overrides the .link base's items-center so card text
          // reads left-aligned.
          "catalog-card group flex h-full w-full flex-col items-stretch rounded-xl border border-separator bg-surface p-4",
          "no-underline transition-[border-color,box-shadow,color]",
          "hover:border-accent/50 hover:no-underline",
        )}
        href={`/docs/components/${component.slug}`}
      >
        <ComponentShot component={component} />
        <span className="mt-3 text-sm font-semibold text-foreground transition-colors group-hover:text-accent">
          {component.title}
        </span>
        <span className="mt-1 line-clamp-2 text-sm leading-relaxed text-muted">
          {component.description}
        </span>
        <span className="mt-auto flex flex-wrap items-center gap-2 pt-3">
          <span className="text-xs text-muted">{demoLabel(component.demos.length)}</span>
          {component.hasReference && (
            <Chip color="accent" size="sm" variant="soft">
              Full reference
            </Chip>
          )}
        </span>
      </Link>
    </li>
  );
}
