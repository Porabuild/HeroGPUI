import type { ReactNode } from "react";
import { cn } from "@heroui/react";
import { StaticTable } from "@/components/ui/static-table";
import { StatusChip } from "@/components/ui/status-chip";
import {
  gpuiPartRows,
  gpuiStateRows,
  gpuiStyleRows,
  type NotPortedRow,
  type RowStatus,
} from "@/lib/gpui-docs";
import type { PartRow, StateRow } from "./data";
import type { ApiRow } from "@/components/ui/props-table";

function Mono({ children }: { children: ReactNode }) {
  if (children === undefined || children === null || children === "" || children === "—") {
    return <span className="text-muted">—</span>;
  }
  return <code className="font-mono text-xs break-all">{children}</code>;
}

/** A row's name cell, with a visible "Partial" chip when the port is partial. */
function Named({ children, status }: { children: ReactNode; status: RowStatus }) {
  return (
    <span className="inline-flex flex-wrap items-center gap-2">
      <Mono>{children}</Mono>
      {status === "partial" ? <StatusChip status="partial" /> : null}
    </span>
  );
}

function Description({ children }: { children: ReactNode }) {
  return <span className="text-sm text-muted">{children}</span>;
}

export interface ReferenceColumn<Row> {
  id: string;
  header: string;
  isRowHeader?: boolean;
  cell: (row: Row) => ReactNode;
}

export interface ReferenceTableProps<Row> {
  rows: Row[];
  columns: ReferenceColumn<Row>[];
  /** Accessible name, e.g. "Switch parts". */
  label: string;
  /** Message rendered when no rows of this kind are available. */
  empty: string;
  rowId: (row: Row) => string;
  className?: string;
}

export function ReferenceTable<Row>({
  rows,
  columns,
  label,
  empty,
  rowId,
  className,
}: ReferenceTableProps<Row>) {
  if (rows.length === 0) {
    return <p className={cn("text-sm text-muted", className)}>{empty}</p>;
  }

  return (
    <StaticTable
      className={className}
      columns={columns.map(({ id, header, isRowHeader }) => ({ header, id, isRowHeader }))}
      label={label}
      rows={rows.map((row, index) => ({
        cells: columns.map((column) => column.cell(row)),
        id: `${rowId(row)}-${index}`,
      }))}
    />
  );
}

export function PartsTable({
  rows,
  title,
  className,
}: {
  rows: PartRow[];
  title: string;
  className?: string;
}) {
  const visible = gpuiPartRows(rows);
  return (
    <ReferenceTable
      className={className}
      columns={[
        {
          id: "part",
          header: "Part",
          isRowHeader: true,
          cell: (row) => <Named status={row.status}>{row.part}</Named>,
        },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
      ]}
      empty="This component has no separate parts — it is a single builder."
      label={`${title} parts`}
      rowId={(row) => row.part}
      rows={visible}
    />
  );
}

export function StatesTable({
  rows,
  title,
  className,
}: {
  rows: StateRow[];
  title: string;
  className?: string;
}) {
  const visible = gpuiStateRows(rows);
  return (
    <ReferenceTable
      className={className}
      columns={[
        {
          id: "state",
          header: "State",
          isRowHeader: true,
          cell: (row) => <Named status={row.status}>{row.state}</Named>,
        },
        {
          id: "builder",
          header: "Builder",
          cell: (row) => <Mono>{row.builder}</Mono>,
        },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
      ]}
      empty="No interaction states are listed for this component."
      label={`${title} states`}
      rowId={(row) => `${row.state}-${row.builder}`}
      rows={visible}
    />
  );
}

export function StylingTable({
  api,
  title,
  className,
}: {
  api: ApiRow[];
  title: string;
  className?: string;
}) {
  const visible = gpuiStyleRows(api);
  return (
    <ReferenceTable
      className={className}
      columns={[
        {
          id: "method",
          header: "Method",
          isRowHeader: true,
          cell: (row) => <Named status={row.status}>{row.style}</Named>,
        },
        {
          id: "type",
          header: "Values",
          cell: (row) => <Mono>{row.type}</Mono>,
        },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
      ]}
      empty="This component follows the active theme and has no extra appearance builders."
      label={`${title} styling`}
      rowId={(row) => row.style}
      rows={visible}
    />
  );
}

/**
 * Upstream HeroUI surfaces the port deliberately omits, each with its
 * documented reason. Collapsed by default; "Not ported" is a design decision,
 * not a failure, so it is listed rather than hidden.
 */
export function NotPortedTable({
  rows,
  label,
}: {
  rows: NotPortedRow[];
  /** Accessible name, e.g. "Button builders not ported". */
  label: string;
}) {
  if (rows.length === 0) return null;
  return (
    <details className="mt-3 rounded-xl border border-separator px-4 py-2">
      <summary className="cursor-pointer text-sm font-medium text-foreground">
        <span className="inline-flex items-center gap-2">
          <StatusChip status="unavailable" />
          {rows.length} HeroUI {rows.length === 1 ? "entry" : "entries"} not ported
        </span>
      </summary>
      <div className="mt-3 mb-2">
        <StaticTable
          columns={[
            { header: "HeroUI v3", id: "name", isRowHeader: true },
            { header: "Why it is not ported", id: "reason" },
          ]}
          label={label}
          rows={rows.map((row, index) => ({
            cells: [
              <Mono key="name">{row.name}</Mono>,
              <Description key="reason">{row.reason}</Description>,
            ],
            id: `${row.name}-${index}`,
          }))}
        />
      </div>
    </details>
  );
}
