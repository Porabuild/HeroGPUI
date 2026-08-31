import { cn } from "@heroui/react";
import { createHighlighter, type Highlighter } from "shiki";
import { CopyButton } from "@/components/ui/copy-button";

/**
 * Server component. Highlights with shiki using a github-light/github-dark
 * pair: light colors are inlined by shiki, the dark pair is emitted as
 * `--shiki-dark` variables that globals.css applies under `.dark`, so one
 * piece of HTML works in both themes without re-rendering on the client.
 */

const THEMES = {
  light: "github-light",
  dark: "github-dark",
} as const;

export const CODE_LANGS = [
  "rust",
  "tsx",
  "bash",
  "json",
  "toml",
  "powershell",
  "plaintext",
] as const;
export type CodeLang = (typeof CODE_LANGS)[number];

// Created once per process and reused; shiki grammars are heavy.
let highlighterPromise: Promise<Highlighter> | null = null;

function getHighlighter(): Promise<Highlighter> {
  highlighterPromise ??= createHighlighter({
    themes: [THEMES.light, THEMES.dark],
    langs: [...CODE_LANGS],
  });
  return highlighterPromise;
}

const LANG_LABEL: Record<CodeLang, string> = {
  rust: "Rust",
  tsx: "TSX",
  bash: "Shell",
  json: "JSON",
  toml: "TOML",
  powershell: "PowerShell",
  plaintext: "Text",
};

export interface CodeBlockProps {
  code: string;
  lang?: CodeLang;
  /** Optional header title. */
  filename?: string;
  className?: string;
  /** Stable id for the native code visibility control. */
  id?: string;
  /** Whether long snippets should start collapsed. */
  collapsible?: boolean;
}

const COLLAPSE_AFTER_LINES = 18;

function fallbackId(code: string): string {
  let hash = 2166136261;
  for (let index = 0; index < code.length; index += 1) {
    hash ^= code.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return `code-${(hash >>> 0).toString(36)}`;
}

export async function CodeBlock({
  code,
  lang = "tsx",
  filename,
  className,
  id,
  collapsible = true,
}: CodeBlockProps) {
  // Unknown langs would throw inside shiki; fall back to unstyled plaintext.
  const safeLang: CodeLang = CODE_LANGS.includes(lang) ? lang : "plaintext";
  const normalizedCode = code.replace(/\n$/, "");
  const lineCount = normalizedCode ? normalizedCode.split("\n").length : 0;
  const isCollapsible = collapsible && lineCount > COLLAPSE_AFTER_LINES;
  const controlId = id ?? fallbackId(normalizedCode);
  const contentId = `${controlId}-content`;
  const highlighter = await getHighlighter();
  const html = highlighter.codeToHtml(normalizedCode, {
    lang: safeLang,
    themes: { light: THEMES.light, dark: THEMES.dark },
    transformers: [
      {
        name: "line-numbers",
        line(node, line) {
          node.children.unshift({
            type: "element",
            tagName: "span",
            properties: {
              // The border makes each line contribute one hairline segment;
              // stacked with no gap between lines, they read as a single
              // continuous gutter rule rather than a tinted background block.
              class:
                "code-gutter me-3 inline-block w-10 select-none border-e border-separator pe-3 text-end",
            },
            children: [{ type: "text", value: String(line) }],
          });
          return node;
        },
      },
    ],
  });

  return (
    <figure
      className={cn(
        "overflow-hidden rounded-xl border border-separator bg-surface-secondary",
        className,
      )}
    >
      {isCollapsible ? (
        <input
          aria-controls={contentId}
          aria-label="Toggle code visibility"
          className="peer sr-only"
          defaultChecked={false}
          id={controlId}
          type="checkbox"
        />
      ) : null}
      <figcaption className="flex items-center gap-2 border-b border-separator px-3 py-1.5">
        {/* Filename when the snippet is genuinely a file; otherwise the
            language name takes the same slot, in the same muted treatment.
            Never both, and never in the accent colour. */}
        <span className="min-w-0 truncate font-mono text-xs text-muted">
          {filename ?? LANG_LABEL[safeLang]}
        </span>
        <span className="ml-auto shrink-0">
          <CopyButton value={code} />
        </span>
      </figcaption>
      {/* shiki's output is trusted, statically generated markup. */}
      <div
        className={cn(
          "relative overflow-x-auto p-4 font-mono",
          isCollapsible &&
            "max-h-80 overflow-y-hidden bg-surface-secondary after:pointer-events-none after:absolute after:inset-x-0 after:bottom-0 after:h-16 after:bg-gradient-to-t after:from-surface-secondary after:to-transparent after:transition-opacity peer-checked:max-h-none peer-checked:after:opacity-0",
        )}
        dangerouslySetInnerHTML={{ __html: html }}
        id={isCollapsible ? contentId : undefined}
      />
      {isCollapsible ? (
        <>
          <label
            className="flex cursor-pointer items-center justify-center border-t border-separator px-3 py-2 text-xs font-medium text-muted transition-colors hover:text-foreground peer-checked:hidden peer-focus-visible:ring-2 peer-focus-visible:ring-accent"
            htmlFor={controlId}
          >
            Expand code
          </label>
          <label
            className="hidden cursor-pointer items-center justify-center border-t border-separator px-3 py-2 text-xs font-medium text-muted transition-colors hover:text-foreground peer-checked:flex peer-focus-visible:ring-2 peer-focus-visible:ring-accent"
            htmlFor={controlId}
          >
            Collapse code
          </label>
        </>
      ) : null}
    </figure>
  );
}
