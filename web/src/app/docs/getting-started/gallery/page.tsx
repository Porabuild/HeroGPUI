import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";

export const metadata: Metadata = {
  title: "Gallery",
  description: "Run the native HeroGPUI gallery and jump to a component, theme, or example.",
};

const RUN = `cargo run -p herogpui-gallery`;

const PAGE = `HEROGPUI_PAGE="Button" HEROGPUI_THEME=dark cargo run -p herogpui-gallery`;

const SECTION = `HEROGPUI_PAGE="Button" HEROGPUI_SECTION="Usage" cargo run -p herogpui-gallery`;

const INSTALL = `cargo install --path gallery --locked
herogpui-gallery`;

export default function GalleryPage() {
  return (
    <>
      <PageHeader
        title="Gallery"
        description="The desktop gallery is the library's runnable documentation: one page per component, every example, and a theme switcher."
      />

      <p>
        Run it from a HeroGPUI checkout. The first build compiles GPUI and can take several minutes;
        later runs are much faster.
      </p>
      <div className="mt-4">
        <CodeBlock code={RUN} lang="bash" />
      </div>
      <p>
        The window opens at 1280×820. Browse the sidebar, or jump straight to a page and appearance:
      </p>
      <div className="mt-4">
        <CodeBlock code={PAGE} lang="bash" />
      </div>

      <h2 id="open-one-example">Open one example</h2>
      <p>
        <code>HEROGPUI_SECTION</code> filters the page to a matching heading — the same names the
        website Usage selector uses:
      </p>
      <div className="mt-4">
        <CodeBlock code={SECTION} lang="bash" />
      </div>
      <p>
        <code>HEROGPUI_WINDOW_SIZE=1200x2000</code> overrides the size when you need a taller
        capture. <code>HEROGPUI_PREVIEW=component</code> draws only that example, which is how the
        catalog tiles are taken.
      </p>

      <h2 id="installed-launcher">Installed launcher</h2>
      <p>Install the gallery as a CLI when you want it without keeping a cargo workspace open:</p>
      <div className="mt-4">
        <CodeBlock code={INSTALL} lang="bash" />
      </div>

      <Callout kind="tip" title="Same examples as the website">
        Each <Link href="/docs/components">component page</Link> embeds the gallery compiled to
        WebAssembly. Use this desktop app when you want the real window, keyboard, and overlays.
      </Callout>
    </>
  );
}
