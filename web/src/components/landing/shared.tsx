import type { ReactNode } from "react";
import { cn } from "@heroui/react";

/**
 * The section heading shared by the landing sections. (The one client-side
 * piece, CtaLink, lives in cta-link.tsx.)
 */

export interface SectionHeadingProps {
  eyebrow: string;
  title: string;
  sub?: ReactNode;
  align?: "left" | "center" | "stacked";
  className?: string;
}

/**
 * Eyebrow + display heading + optional standfirst, shared by all sections.
 * Left-aligned headings use poratake's split header: the oversized h2 on the
 * left, the dim standfirst bottom-aligned on the right.
 * Stacked headings flow the eyebrow, title, and standfirst vertically, suited
 * for multi-column section layouts.
 */
export function SectionHeading({
  eyebrow,
  title,
  sub,
  align = "left",
  className,
}: SectionHeadingProps) {
  const centered = align === "center";
  const stacked = align === "stacked";
  return (
    <div className={cn("landing-section-heading", centered && "text-center", className)}>
      <p className="font-mono text-xs font-medium tracking-[0.16em] text-accent uppercase">
        {eyebrow}
      </p>
      {centered ? (
        <>
          <h2 className="landing-section-title mt-3 text-balance">{title}</h2>
          {sub && <p className="landing-section-sub mx-auto mt-4 max-w-2xl">{sub}</p>}
        </>
      ) : stacked ? (
        <>
          <h2 className="landing-section-title mt-3 text-balance">{title}</h2>
          {sub ? <p className="landing-section-sub mt-4 max-w-xl">{sub}</p> : null}
        </>
      ) : (
        <div className="landing-section-heading-grid mt-3 grid items-end gap-4 md:grid-cols-[minmax(0,1fr)_minmax(280px,0.42fr)] md:gap-16">
          <h2 className="landing-section-title text-balance">{title}</h2>
          {sub ? <p className="landing-section-sub md:pb-1.5">{sub}</p> : null}
        </div>
      )}
    </div>
  );
}
