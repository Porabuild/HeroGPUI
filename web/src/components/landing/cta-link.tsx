"use client";

import type { ComponentProps } from "react";
import { Link } from "react-aria-components";
import { cn } from "@heroui/react";

export interface CtaLinkProps extends ComponentProps<typeof Link> {
  /** Poratake button recipe: filled primary or hairline ghost. */
  variant?: "primary" | "outline" | "ghost";
}

const VARIANT_CLASS: Record<NonNullable<CtaLinkProps["variant"]>, string> = {
  primary: "site-cta site-cta--primary",
  outline: "site-cta site-cta--ghost",
  ghost: "site-cta site-cta--ghost",
};

/**
 * A real anchor in the poratake button recipe (48px mono rectangle, moon-fill
 * primary, hairline ghost), replacing the HeroUI pill so every call to action
 * on the site reads as porabuild chrome. React Aria's Link picks up the
 * shell's RouterProvider, so navigation stays client-side.
 */
export function CtaLink({ variant = "primary", className, ...props }: CtaLinkProps) {
  return <Link className={cn(VARIANT_CLASS[variant], className)} {...props} />;
}
