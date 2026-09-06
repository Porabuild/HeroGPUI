import { cn } from "@heroui/react";

/**
 * Brand lockup shared by the navbar and footer: `Hero • GPUI` and `Pora • build`.
 * Same device as porabuild.com's `BrandLockup`: a bold brand word, a 0.25em
 * accent dot sitting on the baseline like a period, then a 600-weight word.
 * Type metrics come from the vendored `.pb-brand-*` CSS. The dot is pinned to
 * the absolute size porabuild.com renders it at (4px dot, 3px side margins at
 * its 17px lockup) so every lockup on the page shows the same dot regardless
 * of text size, and the margin clears the neighbouring glyphs under the
 * lockup's negative letter-spacing. Weights are set explicitly because
 * Tailwind's preflight `strong { font-weight: bolder }` would otherwise
 * compound with a semibold parent.
 */
export function BrandLockup({
  brand,
  word,
  className,
}: {
  brand: string;
  word: string;
  className?: string;
}) {
  return (
    <span className={cn("pb-brand-lockup", className)}>
      <strong className="font-bold text-foreground">{brand}</strong>
      <span aria-hidden="true" className="pb-brand-dot mx-[3px] size-[4px]" />
      <span className="pb-brand-lockup-word font-semibold text-accent">{word}</span>
    </span>
  );
}
