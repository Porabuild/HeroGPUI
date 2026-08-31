"use client";

import type { ComponentProps } from "react";
import { Link } from "react-aria-components";
import { Button, buttonVariants, cn } from "@heroui/react";

export interface CtaLinkProps extends ComponentProps<typeof Link> {
  /** HeroUI button variant to style the anchor with. */
  variant?: ComponentProps<typeof Button>["variant"];
  size?: ComponentProps<typeof Button>["size"];
}

/**
 * A real anchor styled with HeroUI's own `buttonVariants` — the link-shaped
 * sibling of `<Button>`, so CTA hover/press/focus behavior uses HeroUI's own
 * styles. React Aria's Link also picks up the shell's
 * RouterProvider, so navigation stays client-side.
 */
export function CtaLink({ variant, size, className, ...props }: CtaLinkProps) {
  return <Link className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}
