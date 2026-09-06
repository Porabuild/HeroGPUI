import type { ReactNode } from "react";
import { cn } from "@heroui/react";
import { StaticTable } from "@/components/ui/static-table";
import { gpuiPartRows, gpuiStateRows, gpuiStyleRows } from "@/lib/gpui-docs";
import type { PartRow, StateRow } from "./data";
import type { ApiRow } from "@/components/ui/props-table";

function Mono({ children }: { children: ReactNode }) {
  if (children === undefined || children === null || children === "" || children === "—") {
    return <span className="text-muted">—</span>;
  }
  return <code className="font-mono text-xs break-all">{children}</code>;
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
        { id: "part", header: "Part", isRowHeader: true, cell: (row) => <Mono>{row.part}</Mono> },
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
          id: "builder",
          header: "Builder",
          isRowHeader: true,
          cell: (row) => <Mono>{row.builder}</Mono>,
        },
        {
          id: "state",
          header: "State",
          cell: (row) => <Mono>{row.state}</Mono>,
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
          cell: (row) => <Mono>{row.style}</Mono>,
        },
        {
          id: "type",
          header: "Type",
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
