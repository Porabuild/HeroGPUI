import type { ReactNode } from "react";
import { cn } from "@heroui/react";

/**
 * A deliberately static, server-rendered table styled with HeroUI v3's table
 * design tokens. Used for the read-only reference tables (props, parts,
 * states, styling tokens) instead of HeroUI's `Table`, and why:
 *
 * HeroUI's `Table` wraps react-aria-components' collection machinery, whose
 * server rendering builds rows into a mutable "collection document" behind a
 * hidden copy of the children. On large tables that snapshot can be read
 * before the document is fully populated, so the prerendered HTML is missing
 * trailing rows — the client then renders the full set and hydration fails
 * with React error #418, tearing down and regenerating the whole page
 * (reproduced on the production build; the server HTML for
 * /docs/components/button had a 9-of-15-row props table while the client
 * rendered all 15). These tables have no interaction — no selection, sorting
 * or keyboard cell navigation — so the collection runtime buys nothing here.
 *
 * The class names are HeroUI v3.2.4's own `tableVariants` slots (see
 * `@heroui/styles` dist/components/table/table.styles.js in the pinned
 * version: base "table-root table-root--primary", content "table__content",
 * …), so the styling is identical to HeroUI's Table.
 *
 * `layout` picks the presentation. "reference" adds two site classes defined
 * in `globals.css`: `docs-bleed` widens the table from the prose measure to
 * the full docs column, and `docs-reference-table` sets the column
 * proportions and stops code cells breaking mid-token. "prose" adds neither,
 * which is what the short guide tables want — they read as part of the
 * sentence above them and belong at the article's measure. The scroll
 * container below stays the narrow-viewport fallback either way.
 */

export interface StaticTableColumn {
  id: string;
  header: string;
  /** Rendered as `<th scope="row">` in each body row. */
  isRowHeader?: boolean;
}

export interface StaticTableRow {
  /** Stable React key, e.g. "Button.variant". */
  id: string;
  /** One cell per column, in column order. */
  cells: ReactNode[];
}

export interface StaticTableProps {
  /** Accessible name for the table, e.g. "Button props". */
  label: string;
  columns: StaticTableColumn[];
  rows: StaticTableRow[];
  className?: string;
  /**
   * "reference" (the default, and what the component reference tables use)
   * breaks the table out of the prose measure and sizes its code columns.
   * "prose" leaves it at the article's measure.
   */
  layout?: "reference" | "prose";
}

export function StaticTable({
  label,
  columns,
  rows,
  className,
  layout = "reference",
}: StaticTableProps) {
  return (
    <div
      className={cn(
        "docs-static-table table-root table-root--primary",
        layout === "reference" && "docs-bleed docs-reference-table",
        className,
      )}
      data-slot="table"
    >
      <div className="table__scroll-container" data-slot="table-scroll-container">
        <table className="table__content" data-slot="table-content">
          <caption className="sr-only">{label}</caption>
          <thead className="table__header" data-slot="table-header">
            <tr>
              {columns.map((column) => (
                <th className="table__column" data-slot="table-column" key={column.id} scope="col">
                  {column.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="table__body" data-slot="table-body">
            {rows.map((row) => (
              <tr className="table__row" data-slot="table-row" key={row.id}>
                {row.cells.map((cell, index) => {
                  const column = columns[index];
                  return column?.isRowHeader ? (
                    <th className="table__cell" data-slot="table-cell" key={column.id} scope="row">
                      {cell}
                    </th>
                  ) : (
                    <td className="table__cell" data-slot="table-cell" key={column?.id ?? index}>
                      {cell}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
