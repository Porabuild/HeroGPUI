import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { StaticTable } from "@/components/ui/static-table";

export const metadata: Metadata = {
  title: "Customization",
  description: "Create a named HeroGPUI theme by overriding semantic colors and layout tokens.",
};

const VIOLET = `use gpui::px;
use herogpui::core::oklch;
use herogpui::theme::{snow, Theme};

let violet = Theme::builder("violet", Theme::light())
    .accent(oklch(0.55, 0.23, 295.0))   // hover / soft / focus all derive
    .role("success", oklch(0.73, 0.19, 150.0), snow())
    .radius(px(6.))                     // field_radius follows at 1.5x
    .build();

herogpui::theme::set_theme(violet, cx);`;

const DERIVE = `// Override one base token; every derived value follows.
let violet = Theme::builder("violet", Theme::light())
    .accent(oklch(0.55, 0.23, 295.0))
    .build();

// \`accent.hover()\` and \`accent.soft()\` are the same color-mix
// expressions, so they move with the base color.`;

const COLOR_MATH = `use herogpui::core::{
    oklch, oklcha, mix_oklab, soft_mix, with_alpha, readable_color,
};

let brand    = oklch(0.55, 0.23, 295.0);   // lightness, chroma, hue
let translucent = oklcha(0.55, 0.23, 295.0, 0.6);
let hover    = mix_oklab(brand, foreground, 0.10);  // 10% toward the foreground
let soft     = soft_mix(brand, 0.15);               // 15% over transparent
let faint    = with_alpha(brand, 0.4);
let label    = readable_color(brand);               // a readable foreground for it`;

interface BuilderRow {
  method: string;
  sets: string;
}

const BUILDER_ROWS: BuilderRow[] = [
  { method: "id(id)", sets: "Names the theme — the id `use_theme(id, cx)` activates it by." },
  {
    method: "appearance(appearance)",
    sets: "Light or dark — decides which shadow set and hairline the layout tokens use.",
  },
  {
    method: "radius(px)",
    sets: "`--radius`. `--field-radius` follows at 1.5× unless overridden afterwards.",
  },
  { method: "field_radius(px)", sets: "`--field-radius` on its own." },
  { method: "border_width(px)", sets: "`--border-width`, the weight a separator draws." },
  { method: "disabled_opacity(f32)", sets: "`--disabled-opacity`." },
  { method: "background / foreground / muted", sets: "The base page tokens." },
  {
    method: "border / separator / focus / link / backdrop",
    sets: "The remaining base tokens. `--separator` defaults to the same value as `--border`.",
  },
  {
    method: "surface(bg, fg)",
    sets: "`--surface` / `--surface-foreground` — cards, accordions, disclosure groups.",
  },
  {
    method: "surface_levels(secondary, tertiary)",
    sets: "`--surface-secondary` and `--surface-tertiary` together.",
  },
  { method: "overlay(bg, fg)", sets: "`--overlay` — tooltips, popovers, modals, menus." },
  { method: "segment(bg, fg)", sets: "`--segment` — the selected segment of a segmented control." },
  {
    method: "role(name, color, foreground)",
    sets: 'Any role\'s base value and foreground. `"accent"` is the fallback name; `"default"` also re-seeds `field.background`.',
  },
  {
    method: "accent(color)",
    sets: "`--accent` and a foreground derived for readability when you do not supply one. `--focus` tracks it.",
  },
  {
    method: "field(bg, fg) / field_placeholder / field_border",
    sets: "The field tokens individually.",
  },
  { method: "build()", sets: "Returns the `Theme`." },
];

export default function CustomizationPage() {
  return (
    <>
      <PageHeader
        title="Customization"
        description="Create a named HeroGPUI theme by overriding semantic colors and layout tokens."
      />

      <p>
        Start from a light or dark <code>Theme</code> and override the semantic colors or layout
        values your application needs. <code>Theme::builder(id, base)</code> names the result, and
        derived values follow the base token they came from.
      </p>

      <h2 id="the-builder">The builder</h2>
      <div className="mt-4">
        <CodeBlock code={VIOLET} lang="rust" filename="custom theme" />
      </div>
      <p>Every method the builder exposes:</p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "Method", id: "method", isRowHeader: true },
          { header: "What it sets", id: "sets" },
        ]}
        label="Theme builder methods"
        layout="prose"
        rows={BUILDER_ROWS.map((row) => ({
          cells: [
            <code className="font-mono text-xs break-all" key="method">
              {row.method}
            </code>,
            <span className="text-sm text-muted" key="sets">
              {row.sets}
            </span>,
          ],
          id: row.method.replace(/[^a-z0-9]+/gi, "-"),
        }))}
      />

      <h2 id="overriding-a-base-token">Overriding a base token</h2>
      <p>
        Override a base token and its derived values follow. The override replaces the{" "}
        <em>input</em> while preserving the mix weights: a role&apos;s hover and soft ratios carry
        over, <code>--focus</code> keeps tracking <code>--accent</code>, and a{" "}
        <code>foreground</code> override flows into <code>soft_foreground</code> at render time
        without rebuilding the theme.
      </p>
      <div className="mt-4">
        <CodeBlock code={DERIVE} lang="rust" />
      </div>

      <h2 id="registering-the-result">Registering the result</h2>
      <p>
        <code>set_theme(theme, cx)</code> registers the theme under its id, activates it, and
        schedules every open window to repaint. The built-ins stay registered, so{" "}
        <code>use_theme(&quot;light&quot;, cx)</code> switches back — and a theme registered with{" "}
        <code>set_theme</code> is thereafter switchable to by id like any other. See{" "}
        <Link href="/docs/getting-started/dark-mode">Dark Mode</Link> for the switching rules.
      </p>

      <h2 id="colour-maths">Colour maths</h2>
      <p>
        The color functions used by the theme crate are public in <code>herogpui-core</code>. Use
        them to create OKLCH values, mix colors in Oklab and choose readable foregrounds:
      </p>
      <div className="mt-4">
        <CodeBlock code={COLOR_MATH} lang="rust" />
      </div>
    </>
  );
}
