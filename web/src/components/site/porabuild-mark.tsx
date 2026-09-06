import { cn } from "@heroui/react";

/**
 * Porabuild wordmark: Pora • build. Same lockup device as Hero • GPUI.
 */
export function PorabuildMark({ className }: { className?: string }) {
  return (
    <span className={cn("pb-brand-lockup inline-flex items-center tracking-[-0.04em]", className)}>
      <strong className="font-semibold text-foreground">Pora</strong>
      <span
        aria-hidden="true"
        className="pb-brand-dot mx-1 inline-block size-1.5 rounded-full bg-accent shadow-[0_0_8px_var(--pb-accent-glow)]"
      />
      <span className="pb-brand-lockup-word font-semibold text-accent">build</span>
    </span>
  );
}
