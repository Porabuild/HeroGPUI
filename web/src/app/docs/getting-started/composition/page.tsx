import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "Composition",
  description:
    "How v3's compound components map onto Rust builders: ordered children, composed parts, and render props.",
};

const BUILDER = `Button::new("save")
    .child(icon)          // ordered children: leading icon first,
    .child("Save")        // then the label text
    .variant(Variant::Primary)
    .on_press(cx.listener(|this, _, _, cx| this.save(cx)))`;

const PARTS = `Card::new()
    .child(CardHeader::new().child(CardTitle::new().child("Invoice")))
    .child(CardContent::new().child("Due in 14 days"))
    .child(CardFooter::new().child(Button::new("pay").child("Pay")))`;

const COMPOSED_PART = `// The X is drawn only where it is composed, exactly as in v3.
Modal::new()
    .id("confirm")
    .is_open(open)
    .title("Delete project")
    .child(ModalCloseTrigger::new())`;

const RENDER_PROP = `// v3 hands sortDirection to Table.SortableColumnHeader's indicator.
// The builder already computes it, so it hands it to a closure instead.
Table::new(rows).indicator(|direction| match direction {
    SortDirection::Ascending => chevron_up(),
    SortDirection::Descending => chevron_down(),
})`;

export default function CompositionPage() {
  return (
    <>
      <PageHeader
        title="Composition"
        description="How v3's compound components map onto Rust builders: ordered children, composed parts, and render props."
        importLine={"Card::new().child(CardHeader::new())"}
      />

      <p>
        HeroUI v3 is a compound-component library: a card is a <code>Card.Root</code> holding a{" "}
        <code>Card.Header</code> holding a <code>Card.Title</code>. There is no JSX here, so the
        same shape is expressed three ways depending on what the part actually is. Telling them
        apart is most of learning the API.
      </p>

      <h2 id="builders">Every component is a builder</h2>
      <p>
        Components are <code>#[derive(IntoElement)]</code> builders implementing{" "}
        <code>RenderOnce</code>. Props are methods; children are ordered <code>.child(..)</code>{" "}
        calls, and the order is the layout order:
      </p>
      <div className="mt-4">
        <CodeBlock code={BUILDER} lang="rust" />
      </div>

      <h2 id="parts">Parts are components too</h2>
      <p>
        Where v3 nests a named part, this port exports it as its own builder and you nest it the
        same way. The parent keeps the padding and the geometry; the parts carry only their own text
        styling:
      </p>
      <div className="mt-4">
        <CodeBlock code={PARTS} lang="rust" />
      </div>
      <Callout kind="note" title="Composed parts draw only where composed">
        A part that v3 renders conditionally behaves the same here. A modal draws its close X only
        if you compose one, so omitting it is how you get a modal without one — <code>Modal</code>{" "}
        has no boolean for it. <code>Popover</code> does take <code>show_close_button</code>,
        because v3 gives it one; the port follows v3 per component rather than imposing one rule on
        both.
      </Callout>
      <div className="mt-4">
        <CodeBlock code={COMPOSED_PART} lang="rust" />
      </div>

      <h2 id="render-props">Render props are inverted, not dropped</h2>
      <p>
        v3 passes values <em>into</em> a child render function —{" "}
        <code>Table.SortableColumnHeader</code> receives <code>sortDirection</code>,{" "}
        <code>Pagination.Link</code> receives <code>isActive</code>. A monolithic builder already
        computes those values, so instead of asking you to supply the part, it hands the values to a
        closure:
      </p>
      <div className="mt-4">
        <CodeBlock code={RENDER_PROP} lang="rust" />
      </div>
      <p>
        The prop is real, just inverted. The others are{" "}
        <code>Pagination::link(|page, is_active|)</code>,{" "}
        <code>InputOTP::slot(|index, Option&lt;char&gt;|)</code>,{" "}
        <code>Slider::thumb(|index, value|)</code>,{" "}
        <code>Dropdown::item_content(|key, is_selected, is_indeterminate|)</code>, and{" "}
        <code>DateField</code>/<code>TimeField</code>&apos;s <code>segment(..)</code>.
      </p>

      <h2 id="which-one">Which one a component uses</h2>
      <p>
        Each component page lists its parts and slots under{" "}
        <Link href="/docs/components">API reference</Link>, generated from the same source the
        library is built from. When a part takes a value the parent computes, it appears there as a
        render prop rather than as a nested builder.
      </p>
      <p>
        State is the other half of this: see <Link href="/docs/getting-started/state">State</Link>{" "}
        for which components hold their own value and which hand you an entity.
      </p>
    </>
  );
}
