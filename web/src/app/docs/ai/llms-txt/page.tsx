import type { Metadata } from "next";
import Link from "next/link";
import { readFileSync } from "node:fs";
import path from "node:path";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { C, H2, H3, Li, Md, P, Td, Th, Ul } from "@/app/docs/ai/_components/docs";
import { SITE } from "@/lib/nav";

export const metadata: Metadata = {
  title: "llms.txt",
  description:
    "HeroGPUI publishes a plain-text llms.txt with the Rust API, theme model, component patterns, and GPUI conventions for coding agents.",
};

// Read the file at build time so the size below and the excerpt stay true to
// the reference agents actually receive.
const LLMS_TXT = readFileSync(path.join(process.cwd(), "..", "llms.txt"), "utf8");
const LLMS_TXT_LINES = LLMS_TXT.trimEnd().split(/\r?\n/).length;
const LLMS_TXT_KB = Math.max(1, Math.round(Buffer.byteLength(LLMS_TXT, "utf8") / 1024));

const BUTTON_EXCERPT = `Button::new("save")
    .child(icon)          // ordered children: leading icon first, then label text
    .child("Save")
    .variant(Variant::Primary)
    .size(Size::Md)
    .is_pending(true)
    .full_width(true)
    .on_press(cx.listener(|this, _, _, cx| this.save(cx)))`;

function llmsSectionKey(heading: string): string {
  return heading.replace(/\s+\(\d+\)$/, "");
}

// Hand-written "what an agent gets" blurbs, keyed by the stable section key
// (the `## ` heading with any trailing " (N)" count stripped) so a count bump
// cannot silently detach a row. Both directions are checked below and throw at
// build time, so the table and the file cannot drift silently.
const SECTION_DESCRIPTIONS: Record<string, string> = {
  Overview:
    "The crate layout — the `herogpui` umbrella, `herogpui-theme` (`ThemeProvider`, `ActiveTheme`), `herogpui-core` (shared enums and OKLCH math), `herogpui-components`, and the gallery app — plus the unsupported legacy names: `content1..4` tokens, numbered color scales, `primary`/`secondary` as colors, the `radius` prop, and components such as `Navbar`, `Image`, `User`, `Spacer`, `Code`, and `Snippet`.",
  Installation:
    "The Cargo dependency lines, and a complete minimal bootstrap: `gpui_platform::application()` with `HeroGpuiAssets`, `ThemeProvider::init`, a window with `app_focus_root`, and root background/foreground from tokens with a light/dark toggle.",
  Theming:
    "The OKLCH token vocabulary: base tokens (`background`, `muted`, `border`, `focus`, `link`, …), containers (`surface`, `overlay`, `segment`), roles (`accent`, `success`, `warning`, `danger` with derived `soft()`/`soft_hover()`), fields, layout tokens (the radius scale, spacing, shadows, tooltip delays), the custom theme builder, and the color-math helpers.",
  "Prop vocabularies":
    "The enum tables for `Variant`, `FieldVariant`, `Prominence`, `Backdrop`, `Color`, `Size`, `SelectionMode`, `Placement`, and related types, with their values and users. It also covers radius helpers, desktop control heights, `NumberFormat`, and floating panels through `util::floating`.",
  "Render props":
    "How render props are spelled in Rust: closures that receive the values the component already computes — `Table::indicator`, `Pagination::link`, `InputOTP::slot`, `Dropdown::item_content`, `Slider::thumb`, `TimeField::segment`, `DateField::segment`.",
  "Controlled and uncontrolled":
    "Every controlled prop is an `Option`; leaving it unset seeds keyed internal state from the matching `default_*`. The full list of controlled/uncontrolled pairs per component, and why `Popover`, `Accordion`, and `Tooltip` take an `id`.",
  Validation:
    "The `validate` closure contract, `validation_errors`, the `validation::resolve` precedence (controlled `is_invalid`, then `validationErrors`, then `validate`), and how `Form` routes a server `ValidationErrors` record into per-field slots.",
  "Component API pattern":
    "The shape every component shares: `#[derive(IntoElement)]` builders implementing `RenderOnce`, caller-owned state entities (`InputState`, `CalendarState`, `TimeState`, …), and callback signatures with `Arc` for closures that capture shared fields.",
  Components:
    "The bulk of the file: per-category API rundowns across sixteen subsections (Buttons, Collections, Colors, Controls, Data Display, Date and Time, Feedback, the calendar view model (`calendar_view`), Forms, Layout, Media, Navigation, Overlays, Pickers, Typography, Utilities), naming every documented builder, part, and its Rust spelling. A few related components share one entry, including ToggleButton/ToggleButtonGroup, Disclosure/DisclosureGroup, and the Label/Description/ErrorMessage/FieldError slots.",
  Gallery:
    "How to run and capture the documentation app: `cargo run -p herogpui-gallery`, the `HEROGPUI_PAGE` / `HEROGPUI_THEME` environment controls, and the screenshot scripts used as the visual-regression source.",
  "Code style":
    "Pinned-GPUI notes that produce wrong code silently: `f32::from(px)` for `Pixels`, no div transforms, `svg()` never inherits text color, block-by-default divs, and `util::floating` for paint order.",
  Animation:
    "The `anim` module that maps data-attribute motion onto GPUI: enter/exit/press helpers, the `Motion` timing and easing curves transcribed from the theme's `--ease-*` tokens, reduced-motion gating, and geometric press and zoom techniques.",
  License: "Apache-2.0 for HeroGPUI and HeroUI, with the Copyright 2025 NextUI Inc. attribution.",
};

// Section names come from the file itself, so a heading or count change lands
// here without a hand edit. `SECTIONS` keeps the file's order.
const LLMS_SECTION_HEADINGS = [...LLMS_TXT.matchAll(/^## (.+)$/gm)].map((m) => m[1].trim());

const SECTIONS: Array<{ name: string; gets: string }> = LLMS_SECTION_HEADINGS.map((heading) => {
  const gets = SECTION_DESCRIPTIONS[llmsSectionKey(heading)];
  if (gets === undefined) {
    throw new Error(`llms.txt section "${heading}" has no description in SECTION_DESCRIPTIONS`);
  }
  return { name: heading, gets };
});

for (const key of Object.keys(SECTION_DESCRIPTIONS)) {
  if (!LLMS_SECTION_HEADINGS.some((heading) => llmsSectionKey(heading) === key)) {
    throw new Error(`SECTION_DESCRIPTIONS key "${key}" has no matching heading in llms.txt`);
  }
}

export default function LlmsTxtPage() {
  return (
    <>
      <PageHeader
        title="llms.txt"
        description="HeroGPUI ships a plain-text llms.txt at the repository root with the Rust API, theme model, component patterns, and GPUI conventions for coding agents."
      />

      <P>
        This page explains the llms.txt convention and the information in HeroGPUI&apos;s file. It
        puts the repository&apos;s Rust spellings, state patterns and GPUI constraints in one place
        for coding agents.
      </P>

      <H2 id="what-llms-txt-is">What llms.txt is</H2>
      <P>
        <C>llms.txt</C> is a community convention (proposed at{" "}
        <a href="https://llmstxt.org" target="_blank" rel="noreferrer">
          llmstxt.org
        </a>
        ) for a markdown file placed at a site&apos;s root that is written{" "}
        <em>for language models</em> rather than for browsers: an <C>H1</C> naming the project, a
        blockquote summarising it, then <C>H2</C> sections carrying the details an agent needs — API
        references, conventions, usage. It complements <C>robots.txt</C> (which governs crawling,
        not comprehension) and <C>README.md</C> (which is written for humans who have already cloned
        the repository).
      </P>
      <P>
        Documentation sites commonly publish two flavours: the index file, and a{" "}
        <C>llms-full.txt</C> bundle containing the entire documentation corpus. HeroUI itself
        publishes both, and the repository&apos;s audits read HeroUI&apos;s{" "}
        <a href="https://heroui.com/react/llms-full.txt" target="_blank" rel="noreferrer">
          heroui.com/react/llms-full.txt
        </a>{" "}
        bundle as upstream reference data. An agent pointed at an <C>llms.txt</C> gets the
        repository&apos;s API contract instead of having to infer it.
      </P>

      <H2 id="herogpuis-llms-txt">HeroGPUI&apos;s llms.txt</H2>
      <P>
        The file lives at the repository root, next to <C>README.md</C>. It is{" "}
        <strong>
          {LLMS_TXT_LINES} lines (~{LLMS_TXT_KB} KB)
        </strong>{" "}
        of plain markdown, and the repository&apos;s own{" "}
        <Link href="/docs/ai/agents-md">agent guide</Link> designates it the public component API
        reference. The site serves it as <C>text/plain</C> — see{" "}
        <Link href={SITE.llmsTxt}>/llms.txt</Link>.
      </P>

      <H3 id="sections">What each section gives an agent</H3>
      <P>
        The file is organised top-down: crate map, bootstrap, theme system, the shared prop
        vocabulary, then per-component API. Sections and their purpose:
      </P>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Section</Th>
              <Th>What an agent gets</Th>
            </tr>
          </thead>
          <tbody>
            {SECTIONS.map((section) => (
              <tr key={section.name}>
                <Td className="whitespace-nowrap font-mono text-xs font-medium text-foreground">
                  {section.name}
                </Td>
                <Td className="text-muted">
                  <Md text={section.gets} />
                </Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H3 id="excerpt">Excerpt</H3>
      <P>
        The per-component sections name builders with their exact Rust spellings, including the
        details that are easy to miss — here the note that a button&apos;s icon and label are
        ordered children because the API has no start/end content slots:
      </P>
      <div className="mt-6">
        <CodeBlock code={BUTTON_EXCERPT} lang="rust" filename="llms.txt — Component API pattern" />
      </div>

      <H2 id="why-this-matters-here">Why this matters here</H2>
      <P>
        HeroGPUI combines a Rust component API with GPUI&apos;s framework rules. <C>llms.txt</C>{" "}
        records the exact spellings, state patterns and platform assumptions an agent needs to work
        in this repository:
      </P>
      <Ul>
        <Li>
          <strong>Use the supported vocabulary.</strong> There is no <C>color</C> × <C>variant</C>{" "}
          matrix (colors are <C>default | accent | success | warning | danger</C>), no <C>radius</C>{" "}
          prop, no <C>content1..4</C> surfaces, and components such as <C>Navbar</C> and{" "}
          <C>Image</C> are not part of the library.
        </Li>
        <Li>
          <strong>Use the pinned framework assumptions.</strong> The repository targets the Zed GPUI
          git revision in <C>Cargo.toml</C> and <C>Cargo.lock</C>, with <strong>Rust 1.98</strong>.
          A newer GPUI API may not be available here. Inherited behavior follows React Aria 3.51.0,
          React Stately 3.49.0 and React Aria Components 1.20.0. Check this file and the repository
          task guides before using an API.
        </Li>
      </Ul>
      <P>
        <C>llms.txt</C> describes the checked-in API in Rust spellings and names unsupported
        concepts instead of suggesting an equivalent that does not exist. It supplements the
        repository&apos;s task guides; those guides still require reading the implementation and
        tests when a task depends on behavior.
      </P>
      <Callout kind="warning" title="Read the component contract first">
        Components use compound composition — a switch is{" "}
        <C>{"<Switch><Switch.Control>…</Switch.Control><Label>…</Label></Switch>"}</C>, not{" "}
        <C>{"<Switch>Label</Switch>"}</C>. Start from <Link href={SITE.llmsTxt}>/llms.txt</Link>{" "}
        when you need the exact component shape.
      </Callout>

      <H2 id="serving">How the site serves it</H2>
      <P>
        A Next.js route handler reads the repository file once at module scope and exports the route
        as <C>force-static</C>, so <C>next build</C> prerenders the response on the build machine —
        where the Rust checkout sits next to the web app — and the deployed site serves static bytes
        without needing the repository at request time.
      </P>
      <P>
        The site is deployed under the base path <C>/herogpui</C>, so the canonical URL is{" "}
        <C>https://porabuild.com/herogpui/llms.txt</C>; in local development it is simply{" "}
        <Link href={SITE.llmsTxt}>/llms.txt</Link>. Either way the response body is the
        repository&apos;s <C>llms.txt</C> verbatim, served as <C>text/plain; charset=utf-8</C>.
      </P>
    </>
  );
}
