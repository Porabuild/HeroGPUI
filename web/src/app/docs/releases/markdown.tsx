import type { ReactNode } from "react";
import { Link } from "@heroui/react";

/**
 * Dependency-free renderer for GitHub release bodies (GitHub-flavoured
 * markdown, stored verbatim in `src/data/releases.json`).
 *
 * Supports the shapes release notes use: fenced code blocks, ATX headings,
 * unordered and ordered lists, paragraphs, and inline code, links, bold and
 * italic. Everything else renders as plain paragraphs. All text goes through
 * React children (never `dangerouslySetInnerHTML`), and link targets with an
 * unsafe scheme render as text.
 */

const HEADING = /^(#{1,4})\s+(.*?)\s*#*\s*$/;
const UNORDERED_ITEM = /^[*+-]\s+(.*)$/;
const ORDERED_ITEM = /^\d+[.)]\s+(.*)$/;
const FENCE = /^```\s*(\S*)\s*$/;
const HR = /^(?:-{3,}|\*{3,}|_{3,})\s*$/;

// code | link | bold | italic, in one pass so markers cannot nest wrongly.
const INLINE = /(`[^`\n]+`|\[[^\]\n]+\]\([^)\n]+\)|\*\*[^*\n]+\*\*|\*[^*\n]+(?<!\*)\*|_[^_\n]+_)/g;

/**
 * A release body is author-supplied text — GitHub's generated notes quote
 * contributor pull request titles verbatim — so a link target is only honoured
 * when its scheme is one of these. Protocol-relative `//host` is rejected: it
 * leaves the site while looking like a site-root path, so it would otherwise
 * be classed as internal and rendered without `rel`/`target`.
 */
function safeHref(href: string): { href: string; external: boolean } | null {
  const lower = href.trim().toLowerCase();
  if (lower.startsWith("//")) return null;
  if (lower.startsWith("https://") || lower.startsWith("http://")) {
    return { href, external: true };
  }
  if (lower.startsWith("/") || lower.startsWith("#") || lower.startsWith("mailto:")) {
    return { href, external: false };
  }
  return null;
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  return text.split(INLINE).map((part, index) => {
    const key = `${keyPrefix}-${index}`;
    if (part.startsWith("`") && part.endsWith("`") && part.length >= 2) {
      return <code key={key}>{part.slice(1, -1)}</code>;
    }
    if (part.startsWith("**") && part.endsWith("**") && part.length >= 4) {
      return <strong key={key}>{part.slice(2, -2)}</strong>;
    }
    if (
      ((part.startsWith("*") && part.endsWith("*")) ||
        (part.startsWith("_") && part.endsWith("_"))) &&
      part.length >= 2
    ) {
      return <em key={key}>{part.slice(1, -1)}</em>;
    }
    const link = part.match(/^\[([^\]\n]+)\]\(([^)\n]+)\)$/);
    if (link) {
      const target = safeHref(link[2].trim());
      if (!target) return <span key={key}>{link[1]}</span>;
      return (
        <Link
          key={key}
          href={target.href}
          {...(target.external ? { target: "_blank", rel: "noreferrer" } : {})}
        >
          {link[1]}
        </Link>
      );
    }
    return <span key={key}>{part}</span>;
  });
}

type Block =
  | { kind: "code"; lang: string; code: string[] }
  | { kind: "heading"; level: number; text: string }
  | { kind: "list"; ordered: boolean; items: string[] }
  | { kind: "paragraph"; text: string[] }
  | { kind: "hr" };

function parseBlocks(body: string): Block[] {
  const blocks: Block[] = [];
  const lines = body.replace(/\r\n?/g, "\n").split("\n");

  let i = 0;
  while (i < lines.length) {
    const line = lines[i];

    const fence = line.match(FENCE);
    if (fence) {
      const code: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].startsWith("```")) {
        code.push(lines[i]);
        i += 1;
      }
      i += 1; // consume the closing fence when present
      blocks.push({ kind: "code", lang: fence[1], code });
      continue;
    }

    if (HR.test(line.trim())) {
      blocks.push({ kind: "hr" });
      i += 1;
      continue;
    }

    const heading = line.match(HEADING);
    if (heading) {
      blocks.push({ kind: "heading", level: heading[1].length, text: heading[2] });
      i += 1;
      continue;
    }

    const firstItem = line.match(UNORDERED_ITEM) ?? line.match(ORDERED_ITEM);
    if (firstItem) {
      const ordered = ORDERED_ITEM.test(line);
      const items = [firstItem[1]];
      i += 1;
      while (i < lines.length) {
        const next = lines[i].match(ordered ? ORDERED_ITEM : UNORDERED_ITEM);
        if (!next) break;
        items.push(next[1]);
        i += 1;
      }
      blocks.push({ kind: "list", ordered, items });
      continue;
    }

    if (line.trim() === "") {
      i += 1;
      continue;
    }

    const text = [line];
    i += 1;
    while (
      i < lines.length &&
      lines[i].trim() !== "" &&
      !lines[i].match(FENCE) &&
      !lines[i].match(HEADING) &&
      !HR.test(lines[i].trim()) &&
      !(lines[i].match(UNORDERED_ITEM) ?? lines[i].match(ORDERED_ITEM))
    ) {
      text.push(lines[i]);
      i += 1;
    }
    blocks.push({ kind: "paragraph", text });
  }
  return blocks;
}

/** Render a release body. Release notes sit under an `h2`, so `#`/`##` demote. */
export function ReleaseBody({ body }: { body: string }) {
  if (body.trim() === "") return null;
  return (
    <>
      {parseBlocks(body).map((block, index) => {
        switch (block.kind) {
          case "code":
            return (
              <pre
                key={index}
                className="mt-3 overflow-x-auto rounded-xl border border-border/70 bg-surface-secondary p-4 font-mono text-sm"
              >
                <code>{block.code.join("\n")}</code>
              </pre>
            );
          case "heading":
            // The release tag owns the h2, so a body `#`/`##`/`###` all land on
            // h3 and anything deeper on h4. Emitting h1/h2 here would break the
            // page outline and put release-note sections in the top-level TOC.
            if (block.level <= 3)
              return <h3 key={index}>{renderInline(block.text, `h${index}`)}</h3>;
            return <h4 key={index}>{renderInline(block.text, `h${index}`)}</h4>;
          case "list":
            if (block.ordered) {
              return (
                <ol key={index}>
                  {block.items.map((item, at) => (
                    <li key={at}>{renderInline(item, `li${index}-${at}`)}</li>
                  ))}
                </ol>
              );
            }
            return (
              <ul key={index}>
                {block.items.map((item, at) => (
                  <li key={at}>{renderInline(item, `li${index}-${at}`)}</li>
                ))}
              </ul>
            );
          case "hr":
            return <hr key={index} />;
          case "paragraph":
            return <p key={index}>{renderInline(block.text.join(" "), `p${index}`)}</p>;
        }
      })}
    </>
  );
}
