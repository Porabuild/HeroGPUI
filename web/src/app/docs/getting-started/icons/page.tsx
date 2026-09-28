import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";

export const metadata: Metadata = {
  title: "Icons",
  description:
    "Register HeroGPUI icon assets, draw the embedded Lucide set, and color icons with the active theme.",
};

const LUCIDE_VERSION = "1.31.0";
const LUCIDE_COUNT = 245;

const REGISTER = `application()
    .with_assets(HeroGpuiAssets)
    .run(|cx| { /* open windows */ });`;

const FALLBACK = `application()
    .with_assets(HeroGpuiAssets::with_fallback(MyAppAssets))
    .run(|cx| { /* both icon sets resolve */ });`;

const DRAW = `use herogpui::prelude::{icons, ActiveTheme};
use herogpui::{px, svg};

svg()
    .size(px(16.))
    .path(icons::PLUS)
    .text_color(cx.colors().foreground)`;

const LUCIDE = `use herogpui::prelude::*;
use herogpui::px;

// 16px in the theme foreground unless told otherwise.
Icon::new(IconName::Search)
Icon::new(IconName::Heart).size(px(24.)).color(cx.colors().danger.color)

// A size step (the Sizable trait: Xs 12, Sm 14, Md 16, Lg 20, Xl 24px).
Icon::new(IconName::Folder).size(IconSize::Lg)

// Lucide's strokeWidth and absoluteStrokeWidth.
Icon::new(IconName::Star).stroke_width(1.5)
Icon::new(IconName::Star).size(px(48.)).stroke_width(1.5).absolute_stroke_width(true)

// Builders that take an icon path take a name too.
TreeItem::new("src", "Sources").icon(IconName::Folder)`;

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
        checkmarks, chevrons, clear buttons, and other built-in chrome, plus the Lucide set below.
        Pass it to <code>with_assets</code> before you run the app:
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

      <h2 id="lucide-icons">Lucide icons</h2>
      <p>
        <code>IconName</code> names a curated set of {LUCIDE_COUNT} icons from{" "}
        <Link href="https://lucide.dev">Lucide</Link> {LUCIDE_VERSION}, embedded in the crate and
        served by <code>HeroGpuiAssets</code> under <code>herogpui/icons/lucide/</code> with no
        extra setup. Each variant is the Lucide name in PascalCase (<code>circle-check</code> is{" "}
        <code>IconName::CircleCheck</code>), and <code>Icon</code> draws one:
      </p>
      <div className="mt-4">
        <CodeBlock code={LUCIDE} lang="rust" />
      </div>
      <p>
        GPUI paints an SVG as one single-colour mask, so a stroke width is a different asset:{" "}
        <code>stroke_width</code> draws <code>IconName::path_with_stroke_width(w)</code>, which{" "}
        <code>HeroGpuiAssets</code> serves as the same file with its <code>stroke-width</code>{" "}
        rewritten, rasterised once per width and size. The gallery&apos;s Icons page lists every
        name with a search box.
      </p>
      <p>
        An icon is decorative and reports no accessibility node, as <code>lucide-react</code> marks
        its svg <code>aria-hidden</code>: name the control that holds it. Lucide is ISC-licensed
        (the Feather-derived icons are MIT); the license ships beside the SVGs and is attributed in{" "}
        <code>NOTICE</code>.
      </p>

      <Callout kind="note" title="No icon font">
        There is no webfont or CSS class for icons. If a glyph is missing, add an SVG to your own
        asset source and draw it with <code>Icon::from_path(..)</code> or{" "}
        <code>svg().path(..)</code>.
      </Callout>
    </>
  );
}
