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

const VARIANTS = `// Same prop names, checked at compile time.
Button::new("edit")
    .label("Edit")
    .variant(Variant::Secondary)
    .size(Size::Lg)`;

const STATES = `div()
    .id("row")
    .bg(colors.surface.background)
    .hover(move |s| s.bg(colors.default.soft()))

// Components do this internally: \`anim::hover_fade\` fades the resting
// surface, and a press is \`anim::pressed\`.`;

const RENDER = `// The closure is handed the value the component computed.
Slider::new("volume", 50.)
    .thumb(|index, value| {
        div().child(format!("thumb {index}: {value}")).into_any_element()
    })`;

const WRAPPER = `/// A save button, everywhere the same.
fn save_button(id: impl Into<ElementId>) -> Button {
    Button::new(id)
        .variant(Variant::Primary)
        .child(icon(icons::CHECK))
        .child("Save")
}

// Still a \`Button\`, so the caller keeps every other prop.
save_button("save").is_pending(saving).full_width()`;

interface MappingRow {
  v3: string;
  route: string;
  detail: string;
}

const CLASS_MAPPINGS: MappingRow[] = [
  {
    v3: 'className="w-full"',
    route: "Layout",
    detail:
      "Wrap the control in a styled div, or use the prop the component documents for it (full_width).",
  },
  {
    v3: 'className="bg-accent"',
    route: "Colour",
    detail:
      "Read the token — cx.colors(), cx.role(Color::Accent) — so the value follows the active theme instead of pinning a shade.",
  },
  {
    v3: 'className="rounded-2xl"',
    route: "Radius",
    detail:
      "util::soft_radius(cx) and its siblings, one per radius step — see the table below. Each component names its own radius.",
  },
  {
    v3: 'className="px-3 text-sm"',
    route: "Spacing and type",
    detail:
      "gpui's Styled methods on the element you own. Inside a component, they are the component's business.",
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
      "cards, the table and every floating panel. Surface carries none — upstream `.surface` declares no radius.",
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
        In HeroGPUI, use documented props for component variants, theme tokens for shared values,
        GPUI&apos;s styling methods for elements you own, and render closures for state-aware
        content. There are no CSS classes to pass through.
      </p>

      <h2 id="variants-carry-the-intent">Variants carry the intent</h2>
      <p>
        Use the documented prop first. Variants, sizes and colors are typed values, so the component
        API makes the available choices explicit. Use the hierarchy below to compare the meaning of
        each button variant:
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

      <h2 id="where-classname-goes">Where `className` goes</h2>
      <p>
        A CSS class usually maps to one of four choices below. Nothing in the Rust API is styled by
        a class string.
      </p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "In HeroUI", id: "v3", isRowHeader: true },
          { header: "Route", id: "route" },
          { header: "In HeroGPUI", id: "detail" },
        ]}
        label="className routes"
        layout="prose"
        rows={CLASS_MAPPINGS.map((row) => ({
          cells: [
            <code className="font-mono text-xs break-all" key="v3">
              {row.v3}
            </code>,
            <span className="text-sm font-medium" key="route">
              {row.route}
            </span>,
            <span className="text-sm text-muted" key="detail">
              {row.detail}
            </span>,
          ],
          id: row.route.replace(/\s+/g, "-"),
        }))}
      />

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

      <h2 id="render-props">Render props</h2>
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

      <h2 id="the-class-reference-translated">The class reference, translated</h2>
      <p>
        HeroUI&apos;s BEM class list (<code>.button</code>, <code>.button--primary</code>,{" "}
        <code>.card__header</code>) maps to Rust modules, component structs and builder methods:{" "}
        <code>herogpui::components::button::Button</code>, <code>Button::variant</code> and{" "}
        <code>Card::header</code>.
      </p>
      <Callout kind="note" title="Control heights and widths">
        Desktop control heights are 32/36/40 for sm/md/lg. A labelled button has no minimum width:
        it hugs its content.
      </Callout>
    </>
  );
}
