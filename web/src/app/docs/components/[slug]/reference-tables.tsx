import type { ReactNode } from "react";
import { cn } from "@heroui/react";
import { StatusChip } from "@/components/ui/status-chip";
import { StaticTable } from "@/components/ui/static-table";
import type { PartRow, StateRow, StylingRow } from "./data";

/**
 * The three non-prop reference tables — parts, states, styling tokens —
 * mirroring `@/components/ui/props-table`. Server components rendering static
 * markup (see `static-table.tsx` for why not HeroUI's Table); the only
 * interactive leaf on these pages is HeroUI's own Table-free chrome.
 */

function Mono({ children }: { children: ReactNode }) {
  if (children === undefined || children === null || children === "") {
    return <span className="text-muted">—</span>;
  }
  return <code className="font-mono text-xs break-all">{children}</code>;
}

function Description({ children }: { children: ReactNode }) {
  return <span className="text-sm text-muted">{children}</span>;
}

function PortedCell({ code, status }: { code: string | null; status: PartRow["status"] }) {
  return (
    <div className="flex max-w-56 flex-col items-start gap-1.5 py-1">
      {code ? <Mono>{code}</Mono> : null}
      <StatusChip status={status} />
    </div>
  );
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
      rows={rows.map((row) => ({
        cells: columns.map((column) => column.cell(row)),
        id: rowId(row),
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
  return (
    <ReferenceTable
      className={className}
      columns={[
        { id: "part", header: "Part", isRowHeader: true, cell: (row) => <Mono>{row.name}</Mono> },
        { id: "slot", header: "Slot", cell: (row) => <Mono>{row.slot}</Mono> },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
        {
          id: "herogpui",
          header: "Rust equivalent",
          cell: (row) => <PortedCell code={row.rustOwner} status={row.status} />,
        },
      ]}
      empty="No compound parts are listed for this component."
      label={`${title} parts`}
      rowId={(row) => row.name}
      rows={rows}
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
  return (
    <ReferenceTable
      className={className}
      columns={[
        {
          id: "state",
          header: "State",
          isRowHeader: true,
          cell: (row) => <Mono>{row.state}</Mono>,
        },
        { id: "selector", header: "Upstream selector", cell: (row) => <Mono>{row.selector}</Mono> },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
        {
          id: "herogpui",
          header: "Rust equivalent",
          cell: (row) => <PortedCell code={row.rust} status={row.status} />,
        },
      ]}
      empty="No interaction states are listed for this component."
      label={`${title} states`}
      rowId={(row) => row.state}
      rows={rows}
    />
  );
}

export function StylingTable({
  rows,
  title,
  className,
}: {
  rows: StylingRow[];
  title: string;
  className?: string;
}) {
  return (
    <ReferenceTable
      className={className}
      columns={[
        {
          id: "token",
          header: "Token",
          isRowHeader: true,
          cell: (row) => <Mono>{row.token}</Mono>,
        },
        {
          id: "description",
          header: "Description",
          cell: (row) => <Description>{row.description}</Description>,
        },
        {
          id: "herogpui",
          header: "Rust equivalent",
          cell: (row) => <PortedCell code={row.rust} status={row.status} />,
        },
      ]}
      empty="No styling tokens are listed for this component."
      label={`${title} styling tokens`}
      rowId={(row) => row.token}
      rows={rows}
    />
  );
}
