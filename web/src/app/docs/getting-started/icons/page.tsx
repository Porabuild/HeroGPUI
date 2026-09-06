import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";

export const metadata: Metadata = {
  title: "Icons",
  description: "Register HeroGPUI icon assets and color them with the active theme.",
};

const REGISTER = `gpui_platform::application()
    .with_assets(HeroGpuiAssets)
    .run(|cx| { /* open windows */ });`;

const FALLBACK = `gpui_platform::application()
    .with_assets(HeroGpuiAssets::with_fallback(MyAppAssets))
    .run(|cx| { /* both icon sets resolve */ });`;

const DRAW = `use herogpui::prelude::{icons, ActiveTheme};
use gpui::{px, svg};

svg()
    .size(px(16.))
    .path(icons::PLUS)
    .text_color(cx.colors().foreground)`;

export default function IconsPage() {
  return (
    <>
      <PageHeader
        title="Icons"
        description="Built-in chrome uses SVG assets from the crate. Register them once, then draw them as GPUI svg elements."
      />

      <h2 id="register-the-assets">Register the assets</h2>
      <p>
        <code>HeroGpuiAssets</code> embeds the <code>herogpui/icons/*.svg</code> files used by
        checkmarks, chevrons, clear buttons, and other built-in chrome. Pass it to{" "}
        <code>with_assets</code> before you run the app:
      </p>
      <div className="mt-4">
        <CodeBlock code={REGISTER} lang="rust" />
      </div>
      <p>
        If your app already has an asset source, wrap it so both sets resolve. Components keep
        working, and your own paths stay available:
      </p>
      <div className="mt-4">
        <CodeBlock code={FALLBACK} lang="rust" />
      </div>

      <h2 id="draw-an-icon">Draw an icon</h2>
      <p>
        Paths live on <code>herogpui::prelude::icons</code>. Size and color are GPUI style methods —
        icons follow the theme when you read <code>cx.colors()</code>:
      </p>
      <div className="mt-4">
        <CodeBlock code={DRAW} lang="rust" />
      </div>
      <p>
        Put the svg ahead of a label when you compose a{" "}
        <Link href="/docs/components/button">Button</Link>. Icon-only buttons use{" "}
        <code>is_icon_only(true)</code> so the control stays square.
      </p>

      <Callout kind="note" title="No icon font">
        There is no webfont or CSS class for icons. If a glyph is missing, add an SVG to your own
        asset source and point <code>svg().path(..)</code> at it.
      </Callout>
    </>
  );
}
