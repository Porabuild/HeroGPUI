import type { Metadata } from "next";
import { Card, Link } from "@heroui/react";
import { PageHeader } from "@/components/ui/page-header";
import { StaticTable } from "@/components/ui/static-table";
import { getCatalog } from "@/lib/catalog";

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
  const catalog = getCatalog();
  const componentCount = Object.keys(catalog.components).length;

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
        typed builders, explicit component state, OKLCH semantic tokens and a desktop gallery.
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

      <h2 id="what-you-get">What you get</h2>
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
            <Card.Title>{componentCount} components</Card.Title>
            <Card.Description>
              The catalog indexes every component as focused pages, grouped by what they help you
              build. Related builders share a page.
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

      <h2 id="build-from-the-public-surface">Build from the public surface</h2>
      <p>
        The component pages, theme guides and root <code>llms.txt</code> describe the public Rust
        API. Start with the <code>herogpui</code> prelude, then use the component reference and
        gallery examples as you compose your application.
      </p>

      <h2 id="desktop-application-scope">Desktop application scope</h2>
      <p>
        HeroGPUI is designed for native desktop applications. Its components provide GPUI focus,
        keyboard and theme behavior, while your application owns the surrounding window and domain
        logic.
      </p>

      <h2 id="start-here">Start here</h2>
      <ol>
        <li>
          <Link href="/docs/getting-started/quick-start">Quick Start</Link> — a window and a button
          you can run.
        </li>
        <li>
          <Link href="/docs/getting-started/gallery">Gallery</Link> — every example in a native
          desktop app.
        </li>
        <li>
          <Link href="/docs/components/button">Button</Link> — the first component page, with live
          WebAssembly and the Rust builders.
        </li>
        <li>
          <Link href="/docs/getting-started/theming">Theming</Link> — light, dark, and the tokens
          every control reads.
        </li>
      </ol>

      <h2 id="faq">FAQ</h2>
      <h3 id="is-it-open-source">Is it open source?</h3>
      <p>
        Yes. Apache License 2.0. The repository is{" "}
        <Link href="https://github.com/Porabuild/HeroGPUI">github.com/Porabuild/HeroGPUI</Link>.
      </p>
      <h3 id="what-platforms">What platforms?</h3>
      <p>
        Windows, macOS, and Linux from one Rust API. It is a native desktop library, not a web UI.
      </p>
      <h3 id="how-do-i-run-examples">How do I run the examples?</h3>
      <p>
        The website embeds the gallery as WebAssembly on each component page. For the real window,
        run the <Link href="/docs/getting-started/gallery">desktop gallery</Link>.
      </p>
      <h3 id="how-does-this-relate-to-gpui">How does this relate to GPUI?</h3>
      <p>
        GPUI is the renderer and window runtime (the engine behind Zed). HeroGPUI is the component
        kit you call from Rust: buttons, fields, overlays, and a theme.
      </p>

      <h2 id="next-steps">Next steps</h2>
      <ul>
        <li>
          <Link href="/docs/getting-started/installation">Installation</Link> — assets, the theme
          provider, and platform notes.
        </li>
        <li>
          <Link href="/docs/getting-started/state">State</Link> — controlled and uncontrolled
          components.
        </li>
        <li>
          <Link href="/docs/getting-started/composition">Composition</Link> — parts and render
          closures.
        </li>
        <li>
          <Link href="/docs/getting-started/keyboard">Keyboard and focus</Link> — Tab, Escape, and{" "}
          <code>app_focus_root</code>.
        </li>
        <li>
          <Link href="/docs/components">Components</Link> — the full catalog.
        </li>
        <li>
          <Link href="/llms.txt">llms.txt</Link> — the public API as plain text for agents.
        </li>
      </ul>
    </>
  );
}
