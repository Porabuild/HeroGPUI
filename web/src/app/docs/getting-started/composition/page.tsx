import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "Composition",
  description:
    "Compound components as Rust builders: ordered children, composed parts, and render closures.",
};

const BUILDER = `Button::new("save")
    .child("Save")              // ordered children: icon first,
    .variant(Variant::Primary)   // then the label text
    .on_press(|_, _, _| { /* save */ })`;

const PARTS = `Card::new()
    .child(CardHeader::new().child(CardTitle::new().child("Invoice")))
    .child(CardContent::new().child("Due in 14 days"))
    .child(CardFooter::new().child(Button::new("pay").child("Pay")))`;

const COMPOSED_PART = `// The X is drawn only where it is composed.
Modal::new()
    .id("confirm")
    .is_open(open)
    .title("Delete project")
    .child(ModalCloseTrigger::new())`;

const RENDER_PROP = `// The builder already computes the sort direction,
// so it hands it to a closure instead of asking for a part.
Table::new(vec!["Name".into(), "Size".into()])
    .indicator(|direction| match direction {
        SortDirection::Ascending => chevron_up(),
        SortDirection::Descending => chevron_down(),
    })`;

export default function CompositionPage() {
  return (
    <>
      <PageHeader
        title="Composition"
        description="Compound components as Rust builders: ordered children, composed parts, and render closures."
        importLine={"Card::new().child(CardHeader::new())"}
      />

      <p>
        A card is a <code>Card</code> holding a <code>CardHeader</code> holding a{" "}
        <code>CardTitle</code>. Each part is its own builder, and you nest them in layout order.
        Three patterns cover every component, and telling them apart is most of learning the API.
      </p>

      <h2 id="every-component-is-a-builder">Every component is a builder</h2>
      <p>
        Components are builders implementing <code>RenderOnce</code>. Props are methods; children
        are ordered <code>.child(..)</code> calls, and the order is the layout order:
      </p>
      <div className="mt-4">
        <CodeBlock code={BUILDER} lang="rust" />
      </div>

      <h2 id="parts-are-components-too">Parts are components too</h2>
      <p>
        A named part is its own builder, nested the same way. The parent keeps the padding and the
        geometry; the parts carry only their own text styling:
      </p>
      <div className="mt-4">
        <CodeBlock code={PARTS} lang="rust" />
      </div>
      <Callout kind="note" title="Composed parts draw only where composed">
        A part that renders conditionally behaves the same here. A modal draws its close X only if
        you compose one, so omitting it is how you get a modal without one — <code>Modal</code> has
        no boolean for it. <code>Popover</code> takes <code>show_close_button</code> instead. Each
        component documents its own rule.
      </Callout>
      <div className="mt-4">
        <CodeBlock code={COMPOSED_PART} lang="rust" />
      </div>

      <h2 id="closures-receive-computed-values">Closures receive computed values</h2>
      <p>
        Where a part needs a value the parent computes — the sort direction of a column header, the
        active page of a pagination link — the builder hands that value to a closure:
      </p>
      <div className="mt-4">
        <CodeBlock code={RENDER_PROP} lang="rust" />
      </div>
      <p>
        The others are <code>Pagination::link(|page, is_active|)</code>,{" "}
        <code>InputOTP::slot(|index, Option&lt;char&gt;|)</code>,{" "}
        <code>Slider::thumb(|index, value|)</code>,{" "}
        <code>Dropdown::item_content(|key, state|)</code> with the item key and its interaction
        state, and <code>DateField</code>/<code>TimeField</code>&apos;s{" "}
        <code>segment(|segment, text|)</code>.
      </p>

      <h2 id="which-one-a-component-uses">Which one a component uses</h2>
      <p>
        Each component page lists its parts and slots under the{" "}
        <Link href="/docs/components/button#api-reference">Button API reference</Link>, generated
        from the same source the library is built from. When a part takes a value the parent
        computes, it appears there as a render closure rather than as a nested builder.
      </p>
      <p>
        State is the other half of this: see <Link href="/docs/getting-started/state">State</Link>{" "}
        for which components hold their own value and which hand you an entity.
      </p>
    </>
  );
}
