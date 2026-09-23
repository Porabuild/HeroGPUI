"use client";

import { Button } from "@heroui/react";
import { Check, Copy } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

/**
 * Small client child of `CodeBlock`; copies the snippet to the clipboard.
 *
 * The text is read from the rendered code elements (`sourceIds`, joined by a
 * blank line) rather than passed as a prop, so a page does not ship every
 * snippet a second time in its RSC payload. A component example lists its
 * own block and then the helper blocks it calls, so the copy is complete.
 */
export function CopyButton({ sourceIds }: { sourceIds: string[] }) {
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timeoutRef.current), []);

  const onCopy = useCallback(() => {
    const value = sourceIds
      .map((id) => document.getElementById(id)?.querySelector("code")?.textContent ?? "")
      .filter((text) => text !== "")
      .join("\n\n");
    navigator.clipboard
      .writeText(value)
      .then(() => {
        setCopied(true);
        window.clearTimeout(timeoutRef.current);
        timeoutRef.current = window.setTimeout(() => setCopied(false), 2000);
      })
      .catch(() => {
        // Clipboard unavailable (permissions/insecure context); no-op.
      });
  }, [sourceIds]);

  return (
    <Button
      aria-label={copied ? "Copied" : "Copy to clipboard"}
      className="code-copy-button size-7 hover:text-accent data-[hovered=true]:text-accent"
      isIconOnly
      onPress={onCopy}
      size="sm"
      variant="ghost"
    >
      {copied ? <Check className="size-3.5 text-success" /> : <Copy className="size-3.5" />}
    </Button>
  );
}
