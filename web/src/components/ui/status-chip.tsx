import { Chip } from "@heroui/react";

export type PortStatus = "implemented" | "partial" | "unavailable";

/**
 * Status of one API row against upstream HeroUI. `unavailable` renders as
 * "Not ported" — a HeroUI prop with no HeroGPUI equivalent by design, with a
 * documented reason; not a failure.
 */
export function StatusChip({ status }: { status: PortStatus }) {
  switch (status) {
    case "implemented":
      return (
        <Chip className="docs-status-chip" color="success" size="sm" variant="soft">
          Implemented
        </Chip>
      );
    case "partial":
      return (
        <Chip className="docs-status-chip" color="warning" size="sm" variant="soft">
          Partial
        </Chip>
      );
    case "unavailable":
      return (
        <Chip className="docs-status-chip" color="default" size="sm" variant="soft">
          Not ported
        </Chip>
      );
  }
}
