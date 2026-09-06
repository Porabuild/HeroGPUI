import { cn } from "@heroui/react";
import { Fragment, type ReactNode } from "react";
import { StaticTable } from "@/components/ui/static-table";
import type { PortStatus } from "@/components/ui/status-chip";
import { gpuiPropRows } from "@/lib/gpui-docs";

/** One row of the extracted API contract. Displayed as the Rust builder. */
export interface ApiRow {
  owner: string;
  prop: string;
  type: string;
  default?: string | null;
  description: string;
  /** Rust builder method, e.g. `variant(Variant)`. */
  rust?: string | null;
  status: PortStatus;
}

export interface PropsTableProps {
  rows: ApiRow[];
  /** Accessible name and heading context, e.g. "Button props". */
  label: string;
  className?: string;
}

/**
 * Zero-width break opportunities after the joints a reader already sees in a
 * signature: `.`, `::`, `|`, `,`, `<` and `(`. Reference cells keep
 * `word-break: normal` (see `globals.css`), so these end up being the only
 * places a value is allowed to wrap.
 */
function withBreaks(value: string): ReactNode {
  const parts = value.split(/(?<=[.,|<(]|::)/);
  if (parts.length === 1) return value;
  return parts.map((part, index) => (
    <Fragment key={index}>
      {part}
      {index < parts.length - 1 ? <wbr /> : null}
    </Fragment>
  ));
}

function Mono({ children }: { children: string }) {
  if (children.length === 0 || children === "—") {
    return <span className="text-muted">—</span>;
  }
  return <code className="font-mono text-xs">{withBreaks(children)}</code>;
}

/**
 * The public Rust builders for a component. Web-only rows are omitted; the
 * builder is the first column.
 */
export function PropsTable({ rows, label, className }: PropsTableProps) {
  const visible = gpuiPropRows(rows);
  if (visible.length === 0) {
    return (
      <p className={cn("text-sm text-muted", className)}>
        No documented builders for this component.
      </p>
    );
  }

  return (
    <StaticTable
      className={className}
      columns={[
        { header: "Builder", id: "builder", isRowHeader: true },
        { header: "Type", id: "type" },
        { header: "Default", id: "default" },
        { header: "Description", id: "description" },
      ]}
      label={label}
      rows={visible.map((row) => ({
        cells: [
          <Mono key="builder">{row.builder}</Mono>,
          <Mono key="type">{row.type}</Mono>,
          <Mono key="default">{row.default}</Mono>,
          <span className="text-sm text-muted" key="description">
            {row.description}
          </span>,
        ],
        id: row.builder,
      }))}
    />
  );
}
