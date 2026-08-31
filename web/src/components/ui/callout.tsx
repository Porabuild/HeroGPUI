import { Alert, cn } from "@heroui/react";
import { Info, Lightbulb, TriangleAlert } from "lucide-react";
import type { ComponentProps, ReactNode } from "react";

type CalloutKind = "note" | "warning" | "tip";

const CALLOUT_KINDS: Record<
  CalloutKind,
  {
    status: ComponentProps<typeof Alert.Root>["status"];
    label: string;
    Icon: typeof Info;
  }
> = {
  note: { status: "default", label: "Note", Icon: Info },
  warning: { status: "warning", label: "Warning", Icon: TriangleAlert },
  tip: { status: "accent", label: "Tip", Icon: Lightbulb },
};

export interface CalloutProps {
  kind?: CalloutKind;
  /** Overrides the kind's default title. */
  title?: string;
  children: ReactNode;
  className?: string;
}

/** Doc-page aside built on HeroUI's Alert. */
export function Callout({ kind = "note", title, children, className }: CalloutProps) {
  const { status, label, Icon } = CALLOUT_KINDS[kind];
  return (
    <Alert className={cn("docs-callout", className)} status={status}>
      <Alert.Indicator>
        <Icon className="size-4" />
      </Alert.Indicator>
      <Alert.Content>
        <Alert.Title>{title ?? label}</Alert.Title>
        <Alert.Description>{children}</Alert.Description>
      </Alert.Content>
    </Alert>
  );
}
