import { cn } from "@heroui/react";
import { CodeBlock } from "@/components/ui/code-block";

export interface PageHeaderProps {
  title: string;
  description?: string;
  /** Optional import line, rendered as a Rust code block. */
  importLine?: string;
  className?: string;
}

/** Title, description, and optional import line at the top of a page. */
export function PageHeader({ title, description, importLine, className }: PageHeaderProps) {
  return (
    <header className={cn("docs-page-header mb-10", className)}>
      <h1 className="text-3xl font-semibold tracking-tight text-foreground">{title}</h1>
      {description && <p className="mt-3 text-base leading-relaxed text-muted">{description}</p>}
      {importLine && (
        <div className="mt-6">
          <CodeBlock code={importLine} lang="rust" />
        </div>
      )}
    </header>
  );
}
