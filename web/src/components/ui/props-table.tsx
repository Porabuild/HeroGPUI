import { cn } from "@heroui/react";
import { Fragment, type ReactNode } from "react";
import { StatusChip, type PortStatus } from "@/components/ui/status-chip";
import { StaticTable } from "@/components/ui/static-table";

/** One row of the upstream API contract, as extracted into reference.json. */
export interface ApiRow {
  /** HeroUI component owning the prop, e.g. `Button`. */
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
 * places a value is allowed to wrap. `false` and `'md'` stay whole however
 * narrow the column gets, a union wraps at its `|` separators, and a
 * qualified name like `ButtonRenderProps.isFocusVisible` wraps after the dot
 * instead of forcing the column wider than the docs allow.
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
  if (children.length === 0) {
    return <span className="text-muted">—</span>;
  }
  return <code className="font-mono text-xs">{withBreaks(children)}</code>;
}

/**
 * Upstream prop contract vs the Rust port. "Not ported" marks a deliberate,
 * documented omission from the API audit — not a defect. Rendered with the
 * static table (see `static-table.tsx` for why not HeroUI's Table).
 */
export function PropsTable({ rows, label, className }: PropsTableProps) {
  if (rows.length === 0) {
    return (
      <p className={cn("text-sm text-muted", className)}>No documented props for this component.</p>
    );
  }

  return (
    <div className={cn("space-y-2", className)}>
      <StaticTable
        columns={[
          { header: "Prop", id: "prop", isRowHeader: true },
          { header: "Type", id: "type" },
          { header: "Default", id: "default" },
          { header: "Description", id: "description" },
          { header: "HeroGPUI", id: "herogpui" },
        ]}
        label={label}
        rows={rows.map((row) => {
          const rowId = `${row.owner}.${row.prop}`;
          return {
            cells: [
              <Mono key="prop">{rowId}</Mono>,
              <Mono key="type">{row.type}</Mono>,
              <Mono key="default">{row.default ?? ""}</Mono>,
              <span className="text-sm text-muted" key="description">
                {row.description}
              </span>,
              <div className="flex max-w-44 flex-col items-start gap-1.5 py-1" key="herogpui">
                {row.rust ? <Mono>{row.rust}</Mono> : null}
                <StatusChip status={row.status} />
              </div>,
            ],
            id: rowId,
          };
        })}
      />

      <p className="text-xs text-muted">
        <span className="font-medium text-foreground">Not ported</span> marks a deliberate,
        documented omission from the port — measured by the repo&apos;s API audit, not a defect.
      </p>
    </div>
  );
}
