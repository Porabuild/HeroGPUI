import type { Metadata } from "next";
import { Card, Link } from "@heroui/react";
import { PageHeader } from "@/components/ui/page-header";
import { StaticTable } from "@/components/ui/static-table";

export const metadata: Metadata = {
  title: "Introduction",
  description:
    "Build native desktop interfaces in Rust with typed HeroGPUI components, semantic themes, and GPUI.",
};

const CRATES = [
  {
    crate: "herogpui",
    role: "Umbrella crate and prelude for building HeroGPUI applications.",
  },
  {
    crate: "herogpui-core",
    role: "Shared types such as Color, Variant, FieldVariant, Prominence, Backdrop and Size, plus OKLCH and Oklab color math.",
  },
  {
    crate: "herogpui-theme",
    role: "Semantic OKLCH colors, layout tokens and the ThemeProvider.",
  },
  {
    crate: "herogpui-components",
    role: "Typed component builders and their state behavior.",
  },
];

export default function IntroductionPage() {
  return (
    <>
      <PageHeader
        title="Introduction"
        description="Build native desktop interfaces in Rust with typed HeroGPUI components, semantic themes, and GPUI."
      />

      <h2 id="what-is-herogpui">What is HeroGPUI</h2>
      <p>
        HeroGPUI is a UI library for Rust desktop applications built on GPUI, the GPU-accelerated
        framework behind the Zed editor. It runs on Windows, macOS and Linux from one codebase, with
        typed builders, explicit component state and semantic themes. HeroGPUI brings HeroUI&apos;s
        design system to Rust, including its OKLCH color vocabulary, component patterns and desktop
        gallery.
      </p>
      <p>
        The workspace separates shared types and color math, theme tokens, component builders and
        the umbrella crate:
      </p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "Crate", id: "crate", isRowHeader: true },
          { header: "What it holds", id: "role" },
        ]}
        label="The four crates of the workspace"
        layout="prose"
        rows={CRATES.map((row) => ({
          cells: [
            <code className="font-mono text-xs" key="crate">
              {row.crate}
            </code>,
            <span className="text-sm text-muted" key="role">
              {row.role}
            </span>,
          ],
          id: row.crate,
        }))}
      />

      <h2 id="beautiful-fast-and-modern">What you get</h2>
      <ul>
        <li>
          <strong>Native rendering</strong> — GPUI renders the interface without a browser DOM.
        </li>
        <li>
          <strong>Typed builders</strong> — component options are Rust types, and state is explicit
          and either controlled or uncontrolled.
        </li>
        <li>
          <strong>Semantic themes</strong> — OKLCH colors, layout tokens, light and dark themes, and
          reduced-motion support are shared across components.
        </li>
        <li>
          <strong>One codebase</strong> — target Windows, macOS and Linux with the same Rust API.
        </li>
      </ul>

      <h2 id="highlights">Highlights</h2>
      <div className="mt-4 grid gap-4 sm:grid-cols-3">
        <Card.Root>
          <Card.Content>
            <Card.Title>71 components</Card.Title>
            <Card.Description>
              HeroGPUI implements every component documented by HeroUI. The catalog indexes them as
              66 pages because related components share a page.
            </Card.Description>
          </Card.Content>
        </Card.Root>
        <Card.Root>
          <Card.Content>
            <Card.Title>Semantic themes</Card.Title>
            <Card.Description>
              OKLCH roles, surfaces and field tokens with derived hover and soft variants in light
              and dark themes.
            </Card.Description>
          </Card.Content>
        </Card.Root>
        <Card.Root>
          <Card.Content>
            <Card.Title>Gallery &amp; docs</Card.Title>
            <Card.Description>
              A desktop gallery ships with the library and documents every component with runnable
              examples.
            </Card.Description>
          </Card.Content>
        </Card.Root>
      </div>

      <h2 id="measured-parity">Build from the public surface</h2>
      <p>
        The component pages, theme guides and root <code>llms.txt</code> describe the public Rust
        API. Start with the <code>herogpui</code> prelude, then use the component reference and
        gallery examples as you compose your application.
      </p>

      <h2 id="what-is-deliberately-not-ported">Desktop application scope</h2>
      <p>
        HeroGPUI is designed for native desktop applications. Its components provide GPUI focus,
        keyboard and theme behavior, while your application owns the surrounding window and domain
        logic.
      </p>

      <h2 id="next-steps">Next steps</h2>
      <ul>
        <li>
          <Link href="/docs/getting-started/installation">Installation</Link> — add the crate,
          register the assets and theme provider, render your first component.
        </li>
        <li>
          <Link href="/docs/getting-started/state">State</Link> — controlled and uncontrolled
          components, and which ones hand you a state entity to own.
        </li>
        <li>
          <Link href="/docs/getting-started/theming">Theming</Link> — the OKLCH semantic token
          system shared by every component.
        </li>
        <li>
          <Link href="/docs/getting-started/composition">Composition</Link> — ordered children,
          composed parts, and the render props v3 inverts.
        </li>
        <li>
          <Link href="/docs/getting-started/animation">Animation</Link> — v3's per-overlay curves,
          reduced motion, and what GPUI's missing transforms cost.
        </li>
        <li>
          <Link href="/docs/components">Components</Link> — browse the catalog, grouped by 15
          categories.
        </li>
        <li>
          <Link href="/docs/releases">Releases</Link> — read the release notes.
        </li>
        <li>
          <Link href="/llms.txt">llms.txt</Link> — the full public API reference, written for
          agents.
        </li>
      </ul>
    </>
  );
}
