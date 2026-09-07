import type { Metadata } from "next";
import { Button } from "@heroui/react";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";
import { StaticTable } from "@/components/ui/static-table";

export const metadata: Metadata = {
  title: "Styling",
  description:
    "Style HeroGPUI components with typed props, theme tokens, slots, and render closures.",
};

const VARIANTS = `// Variants and sizes are enums, checked at compile time.
Button::new("edit")
    .label("Edit")
    .variant(Variant::Secondary)
    .size(Size::Lg)`;

const STATES = `// .hover() styles an element you own; components do the same
// internally with anim::hover_fade, and a press is anim::pressed.
let colors = cx.colors();
let resting = colors.surface.background;
let hovered = colors.default.soft();

div()
    .id("row")
    .bg(resting)
    .hover(move |s| s.bg(hovered))`;

const RENDER = `// The closure is handed the value the component computed.
Slider::new("volume", 50.)
    .thumb(|index, value| {
        div().child(format!("thumb {index}: {value}")).into_any_element()
    })`;

const WRAPPER = `/// A save button, everywhere the same.
fn save_button(id: impl Into<ElementId>) -> Button {
    Button::new(id).variant(Variant::Primary).child("Save")
}

// Still a \`Button\`, so the caller keeps every other prop.
save_button("save").is_pending(saving).full_width()`;

const SX = `// Every component carries one sx slot: GPUI's styling methods,
// refined over the root element after the theme's values.
Button::new("save")
    .label("Save")
    .sx(|el| el.bg(gpui::rgba(0xffa500ff)).text_color(gpui::rgba(0x000000ff)))`;

interface MappingRow {
  route: string;
  rust: string;
  detail: string;
}

const STYLE_ROUTES: MappingRow[] = [
  {
    route: "Override",
    rust: ".sx(|el| el.bg(..))",
    detail:
      "One slot per component for caller-owned low-level styling; it refines the root element last, so it wins.",
  },
  {
    route: "Layout",
    rust: "full_width(true)",
    detail: "Use the builder the component documents, or wrap it in a styled div you own.",
  },
  {
    route: "Colour",
    rust: "cx.role(Color::Accent)",
    detail: "Read a theme token so the value follows light and dark instead of pinning a shade.",
  },
  {
    route: "Radius",
    rust: "util::soft_radius(cx)",
    detail:
      "One helper per radius step — see the table below. Each component names its own radius.",
  },
  {
    route: "Spacing and type",
    rust: ".px(px(12.)).text_sm()",
    detail: "GPUI's Styled methods on the element you own. Inside a component, they stay there.",
  },
];

const RADII: { rust: string; value: string; usedBy: string }[] = [
  {
    rust: "util::control_radius(cx)",
    value: "3xl (24px)",
    usedBy: "button, toggle button, avatar",
  },
  {
    rust: "util::soft_radius(cx)",
    value: "2xl (16px)",
    usedBy: "chip, menu and list rows, colour area",
  },
  {
    rust: "util::small_radius(cx)",
    value: "xl (12px)",
    usedBy: "close button, tag, link, tooltip",
  },
  { rust: "util::key_radius(cx)", value: "lg (8px)", usedBy: "Kbd" },
  {
    rust: "util::hairline_radius(cx)",
    value: "sm (4px)",
    usedBy: "separator, skeleton",
  },
  { rust: "util::field_radius(cx)", value: "12px", usedBy: "every form field" },
  {
    rust: "util::container_radius(cx)",
    value: "min(32px, 3xl)",
    usedBy:
      "cards, the table and every floating panel. Surface carries none — `.surface` declares no radius.",
  },
];

export default function StylingPage() {
  return (
    <>
      <PageHeader
        title="Styling"
        description="Style HeroGPUI components with typed props, theme tokens, slots, and render closures."
      />

      <p>
        Use documented builders for variants, theme tokens for shared values, GPUI&apos;s styling
        methods for elements you own, and render closures for state-aware content.
      </p>

      <h2 id="variants-carry-the-intent">Variants carry the intent</h2>
      <p>
        Use the documented builder first. Variants, sizes and colors are typed values, so the
        component API makes the available choices explicit. Use the hierarchy below to compare the
        meaning of each button variant:
      </p>
      <div className="docs-stage mt-4 flex flex-wrap items-center gap-3 rounded-xl border border-separator p-4">
        <Button variant="primary">Save</Button>
        <Button variant="secondary">Edit</Button>
        <Button variant="tertiary">Cancel</Button>
        <Button variant="danger">Delete</Button>
      </div>
      <div className="mt-4">
        <CodeBlock code={VARIANTS} lang="rust" />
      </div>

      <h2 id="how-to-style">How to style</h2>
      <p>
        Use a documented builder for variants, a theme token for shared values, and GPUI&apos;s
        styling methods for elements you own. There are no class strings to pass through.
      </p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "Route", id: "route", isRowHeader: true },
          { header: "Rust", id: "rust" },
          { header: "When", id: "detail" },
        ]}
        label="Styling routes"
        layout="prose"
        rows={STYLE_ROUTES.map((row) => ({
          cells: [
            <span className="text-sm font-medium" key="route">
              {row.route}
            </span>,
            <code className="font-mono text-xs break-all" key="rust">
              {row.rust}
            </code>,
            <span className="text-sm text-muted" key="detail">
              {row.detail}
            </span>,
          ],
          id: row.route.replace(/\s+/g, "-"),
        }))}
      />

      <h2 id="the-sx-slot">The sx slot</h2>
      <p>
        Every component builder carries one <code>sx</code> slot for caller-owned low-level styling:
        GPUI&apos;s styling methods, refined over the component&apos;s root element after every
        value the variant and the theme chose, so an override wins. State-driven layers the
        component draws itself — a hover fade, a press scale — read the override where they can.
      </p>
      <div className="mt-4">
        <CodeBlock code={SX} lang="rust" />
      </div>

      <h3 id="the-radius-helpers">The radius helpers</h3>
      <p>
        Each component uses a specific radius step, so <code>util</code> exposes one helper per step
        rather than a single universal radius:
      </p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "Helper", id: "rust", isRowHeader: true },
          { header: "Step", id: "value" },
          { header: "Used by", id: "used-by" },
        ]}
        label="Corner radius helpers"
        layout="prose"
        rows={RADII.map((row) => ({
          cells: [
            <code className="font-mono text-xs break-all" key="rust">
              {row.rust}
            </code>,
            <code className="font-mono text-xs" key="value">
              {row.value}
            </code>,
            <span className="text-sm text-muted" key="used-by">
              {row.usedBy}
            </span>,
          ],
          id: row.rust.replace(/[^a-z0-9]+/gi, "-"),
        }))}
      />

      <h2 id="state-based-styling">State-based styling</h2>
      <p>
        Components expose hover, press and disabled state through the Rust API. Use{" "}
        <code>.hover()</code> for an element you own, the built-in animation helpers for presses,
        and <code>is_disabled</code> for disabled controls.
      </p>
      <div className="mt-4">
        <CodeBlock code={STATES} lang="rust" />
      </div>

      <h2 id="render-closures">Render closures</h2>
      <p>
        Render closures let you draw a component part from the state or value the component already
        computed. The closure receives that value, so the caller does not need to re-derive it.
      </p>
      <div className="mt-4">
        <CodeBlock code={RENDER} lang="rust" />
      </div>

      <h2 id="wrapper-components">Wrapper components</h2>
      <p>
        To standardize a set of props, return a configured builder from a function. Builders are
        plain Rust values, so the caller can still set every remaining option.
      </p>
      <div className="mt-4">
        <CodeBlock code={WRAPPER} lang="rust" />
      </div>

      <h2 id="style-through-the-rust-api">Style through the Rust API</h2>
      <p>
        The <code>Button</code> struct and its <code>variant</code> method select the look, part
        builders such as <code>CardHeader</code> place the pieces, and theme tokens supply the
        values.
      </p>
      <Callout kind="note" title="Control heights and widths">
        Desktop control heights are 32/36/40 for sm/md/lg. A labelled button has no minimum width:
        it hugs its content.
      </Callout>
    </>
  );
}
