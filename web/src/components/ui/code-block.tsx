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
  /** Wrap long lines instead of scrolling horizontally. */
  wrap?: boolean;
  /** Ids of further `CodeBlock`s whose code Copy appends (an example's helpers). */
  copyAlso?: string[];
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

// Token classes are a short hash of the style they stand for, so the same
// colour pair gets the same class in every page and every build worker (a
// soft navigation keeps the previous page's hoisted rules). A collision would
// paint the wrong colours, so it fails the build instead.
const TOKEN_CLASSES = new Map<string, string>();

function tokenClass(style: string): string {
  const className = `k${fallbackId(style).slice(5, 9)}`;
  const owner = TOKEN_CLASSES.get(className);
  if (owner === undefined) TOKEN_CLASSES.set(className, style);
  else if (owner !== style) {
    throw new Error(`code-block token class ${className} collides: ${owner} / ${style}`);
  }
  return className;
}

export async function CodeBlock({
  code,
  lang = "tsx",
  filename,
  className,
  id,
  collapsible = true,
  wrap = false,
  copyAlso = [],
}: CodeBlockProps) {
  // Unknown langs would throw inside shiki; fall back to unstyled plaintext.
  const safeLang: CodeLang = CODE_LANGS.includes(lang) ? lang : "plaintext";
  const normalizedCode = code.replace(/\n$/, "");
  const lineCount = normalizedCode ? normalizedCode.split("\n").length : 0;
  const isCollapsible = collapsible && lineCount > COLLAPSE_AFTER_LINES;
  const controlId = id ?? fallbackId(normalizedCode);
  const contentId = `${controlId}-content`;
  const highlighter = await getHighlighter();
  // Compact markup. Component pages list every example, so the highlighted
  // HTML (which the RSC payload repeats) dominates page weight:
  //   * a token in the block's default colours carries no style at all -- it
  //     inherits `color` and `--shiki-dark` from the <pre>;
  //   * any other colour pair becomes one short class, defined once per page
  //     by a hoisted, deduplicated <style> (React `precedence`);
  //   * lines are not wrapped in elements: the numbers are one gutter element
  //     beside <code> (`.code-lines` in globals.css). Wrapping blocks keep
  //     their per-line spans for the hanging indent and number them with a
  //     CSS counter (`.code-counter`).
  const tokenClasses = new Map<string, string>();
  // Span hooks run before the <pre> hook, so the default pair comes from the
  // themes rather than from the <pre>'s own style.
  const defaultStyle =
    `color:${highlighter.getTheme(THEMES.light).fg};--shiki-dark:${highlighter.getTheme(THEMES.dark).fg}`.toLowerCase();
  const html = highlighter.codeToHtml(normalizedCode, {
    lang: safeLang,
    themes: { light: THEMES.light, dark: THEMES.dark },
    transformers: [
      {
        name: "compact-tokens",
        pre(node) {
          if (wrap) {
            this.addClassToHast(node, "code-counter");
            return;
          }
          this.addClassToHast(node, "code-lines");
          node.children.unshift({
            type: "element",
            tagName: "span",
            properties: { "aria-hidden": "true", class: "code-gutter" },
            children: [
              {
                type: "text",
                value: Array.from({ length: lineCount }, (_, at) => at + 1).join("\n"),
              },
            ],
          });
        },
        code(node) {
          if (wrap) return;
          node.children = node.children.flatMap((child) =>
            child.type === "element" && child.tagName === "span" ? child.children : [child],
          );
        },
        line(node) {
          // A default-coloured token lost its style above; drop its element.
          node.children = node.children.flatMap((child) =>
            child.type === "element" &&
            child.tagName === "span" &&
            child.properties.class === undefined &&
            child.properties.className === undefined
              ? child.children
              : [child],
          );
        },
        span(node) {
          const style = String(node.properties.style ?? "");
          if (style === "") return;
          delete node.properties.style;
          if (style.toLowerCase() === defaultStyle) return;
          let className = tokenClasses.get(style);
          if (!className) {
            className = tokenClass(style);
            tokenClasses.set(style, className);
          }
          this.addClassToHast(node, className);
        },
      },
    ],
  });

  return (
    <figure
      className={cn(
        "overflow-hidden rounded-xl border border-separator bg-surface-secondary",
        wrap && "code-wrap",
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
      <figcaption className="flex h-9 items-center gap-2 border-b border-separator px-3">
        {/* Filename when the snippet is genuinely a file; otherwise the
            language name takes the same slot, in the same muted treatment.
            Never both, and never in the accent colour. */}
        <span className="min-w-0 truncate font-mono text-[0.6875rem] text-muted">
          {filename ?? LANG_LABEL[safeLang]}
        </span>
        <span className="ml-auto shrink-0">
          <CopyButton sourceIds={[contentId, ...copyAlso.map((other) => `${other}-content`)]} />
        </span>
      </figcaption>
      {[...tokenClasses].map(([style, className]) => (
        <style href={`shiki-${className}`} key={className} precedence="shiki">
          {`.${className}{${style}}`}
        </style>
      ))}
      {/* shiki's output is trusted, statically generated markup. The
          scroller keeps a CSS-only ghost bar so this server component never
          needs a client scrollbar wrapper. */}
      <div
        className={cn(
          "code-scroll relative overflow-x-auto p-4 font-mono",
          isCollapsible &&
            "max-h-80 overflow-y-hidden bg-surface-secondary after:pointer-events-none after:absolute after:inset-x-0 after:bottom-0 after:h-16 after:bg-gradient-to-t after:from-surface-secondary after:to-transparent after:transition-opacity peer-checked:max-h-none peer-checked:after:opacity-0",
        )}
        dangerouslySetInnerHTML={{ __html: html }}
        id={contentId}
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
