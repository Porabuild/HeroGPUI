import type { ReactNode } from "react";

/**
 * Prose primitives for the AI docs pages. Kept inside this route folder so
 * the shell-level design system (`src/components/ui/**`) stays untouched;
 * the three pages share them so headings, spacing, and code styling stay
 * consistent. All h2/h3 elements take an explicit `id` for the docs
 * table of contents, which scrapes `[data-docs-article] h2, h3`.
 */

export function H2({ id, children }: { id: string; children: ReactNode }) {
  return (
    <h2
      id={id}
      className="mt-14 scroll-mt-24 border-t border-separator pt-8 text-2xl font-semibold tracking-tight text-foreground"
    >
      {children}
    </h2>
  );
}

export function H3({ id, children }: { id: string; children: ReactNode }) {
  return (
    <h3 id={id} className="mt-10 scroll-mt-24 text-lg font-semibold tracking-tight text-foreground">
      {children}
    </h3>
  );
}

export function P({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <p className={`mt-4 leading-7 text-foreground first:mt-0 ${className ?? ""}`}>{children}</p>
  );
}

export function Ul({ children }: { children: ReactNode }) {
  return <ul className="mt-4 list-disc space-y-2 pl-6 leading-7 text-foreground">{children}</ul>;
}

export function Li({ children }: { children: ReactNode }) {
  return <li className="pl-1">{children}</li>;
}

/** Inline code, e.g. a file name, crate, or prop. */
export function C({ children }: { children: ReactNode }) {
  return (
    <code className="rounded-md border border-separator bg-surface-secondary px-1.5 py-0.5 font-mono text-[0.85em] text-foreground">
      {children}
    </code>
  );
}

/**
 * Renders one-line prose containing markdown-style `` `code` `` spans, so
 * table cells can name identifiers without embedding JSX. Text outside
 * backticks is untouched; unbalanced backticks render verbatim.
 */
export function Md({ text }: { text: string }) {
  const parts = text.split(/(`[^`]+`)/g);
  return (
    <>
      {parts.map((part, index) =>
        part.length > 1 && part.startsWith("`") && part.endsWith("`") ? (
          <C key={index}>{part.slice(1, -1)}</C>
        ) : (
          part
        ),
      )}
    </>
  );
}

export function Td({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <td className={`border-t border-separator px-3 py-2 align-top ${className ?? ""}`}>
      {children}
    </td>
  );
}

export function Th({ children }: { children: ReactNode }) {
  return (
    <th className="border-b border-separator px-3 py-2 text-left text-xs font-semibold tracking-wider text-muted uppercase">
      {children}
    </th>
  );
}
