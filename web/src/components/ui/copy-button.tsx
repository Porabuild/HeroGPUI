"use client";

import { Button } from "@heroui/react";
import { Check, Copy } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

/** Small client child of `CodeBlock`; copies the snippet to the clipboard. */
export function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);
  const timeoutRef = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timeoutRef.current), []);

  const onCopy = useCallback(() => {
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
  }, [value]);

  return (
    <Button
      aria-label={copied ? "Copied" : "Copy to clipboard"}
      className="size-10 hover:text-accent data-[hovered=true]:text-accent"
      isIconOnly
      onPress={onCopy}
      size="sm"
      variant="ghost"
    >
      {copied ? <Check className="size-3.5 text-success" /> : <Copy className="size-3.5" />}
    </Button>
  );
}
