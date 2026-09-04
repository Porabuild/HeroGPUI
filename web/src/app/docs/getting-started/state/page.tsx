import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "State",
  description:
    "Controlled and uncontrolled components, and which ones need a state entity you own.",
};

const SEED = `let agreed = cx.new(|_| false);

// Controlled: you hold the value, the component reports changes.
Checkbox::new("terms").is_selected(*agreed.read(cx))

// Uncontrolled: the component holds it, seeded from default_*.
Checkbox::new("terms").default_selected(true)`;

const ENTITY = `// Built once, in your view's constructor -- never per frame.
let name = cx.new(|cx| InputState::new(cx));

// ...then handed to the field on every render.
TextField::new(name.clone()).label("Name").is_required(true)`;

const SEEDED = `let name = cx.new(|cx| InputState::with_value(cx, "Ada"));`;

const READ = `let typed = name.read(cx).value().to_owned();`;

const IDS = `// Two uncontrolled popovers must not share an id: the keyed state is
// the id, so both would open together.
Popover::new(trigger).id("row-1-actions")
Popover::new(trigger).id("row-2-actions")`;

export default function StatePage() {
  return (
    <>
      <PageHeader
        title="State"
        description="Controlled and uncontrolled components, and which ones need a state entity you own."
        importLine={"let name = cx.new(|cx| InputState::new(cx));"}
      />

      <p>
        HeroUI v3 has one answer to state: React hooks. This port has two, and which one a component
        uses is the single thing worth learning before writing a form. A checkbox holds its own
        value if you let it; a text field never does, and hands you a state entity to hold instead.
      </p>

      <h2 id="controlled-and-uncontrolled">Controlled and uncontrolled</h2>
      <p>
        Every component takes its controlled prop as an <code>Option</code>. Setting it makes the
        component controlled; leaving it unset hands the value to the component, seeded from the
        matching <code>default_*</code>.
      </p>
      <div className="mt-4">
        <CodeBlock code={SEED} lang="rust" />
      </div>
      <p>
        Where the controlled prop is itself an <code>Option</code> — <code>RadioGroup::value(None)</code>{" "}
        — supplying it at all is what makes the component controlled. The pairs are{" "}
        <code>is_selected</code>/<code>default_selected</code>, <code>is_open</code>/
        <code>default_open</code>, <code>selected_key</code>/<code>default_selected_key</code>,{" "}
        <code>expanded_keys</code>/<code>default_expanded_keys</code>, and <code>value</code>/
        <code>default_value</code>. The full list per component is in{" "}
        <Link href="/llms.txt">llms.txt</Link>.
      </p>

      <h2 id="ids-are-the-keyed-state">An uncontrolled value lives under the id</h2>
      <p>
        Uncontrolled state lives in <code>Window::use_keyed_state</code>, keyed on the component&apos;s
        id. That is why <code>Popover</code>, <code>Accordion</code> and <code>Tooltip</code> take an{" "}
        <code>id</code> even though they render no label from it: it is what tells two instances
        apart.
      </p>
      <div className="mt-4">
        <CodeBlock code={IDS} lang="rust" />
      </div>
      <Callout kind="note" title="One id, one value">
        Two uncontrolled components that share an id share their state. If a list renders the same
        component per row, derive the id from the row key.
      </Callout>

      <h2 id="state-entities">Components that hand you a state entity</h2>
      <p>
        Text, number, date and time inputs do not keep their value internally at all. Their value is
        a GPUI <code>Entity</code> you construct and own, because it is the thing you read on submit
        and the thing two components share when they edit the same value:
      </p>
      <div className="mt-4">
        <CodeBlock code={ENTITY} lang="rust" />
      </div>
      <p>
        The six are <code>InputState</code> (Input, TextField, SearchField, TextArea),{" "}
        <code>NumberState</code> (NumberField), <code>OtpState</code> (InputOTP),{" "}
        <code>CalendarState</code> (Calendar, DatePicker), <code>DateRangeState</code>{" "}
        (RangeCalendar, DateRangePicker) and <code>TimeState</code> (TimeField).
      </p>
      <Callout kind="warning" title="Build it once">
        Build the entity in your view&apos;s constructor, not inside <code>render</code>. A fresh
        entity every frame is a field that forgets what was typed into it.
      </Callout>
      <p>
        For these, the constructor <em>is</em> the uncontrolled seed — there is no{" "}
        <code>default_value</code> to set, because the entity already holds one:
      </p>
      <div className="mt-4">
        <CodeBlock code={SEEDED} lang="rust" />
      </div>
      <p>
        The others are <code>NumberState::with_value</code>, <code>CalendarState::with_selected</code>,{" "}
        <code>DateRangeState::with_range</code> and <code>TimeState::with_value</code>. Read the
        current value back through the entity:
      </p>
      <div className="mt-4">
        <CodeBlock code={READ} lang="rust" />
      </div>

      <h2 id="callbacks">Callbacks</h2>
      <p>
        Change callbacks take the new value, the window and the app:{" "}
        <code>Fn(&amp;T, &amp;mut Window, &amp;mut App)</code>. Anything captured by one must be{" "}
        <code>Arc</code>-cloned, which is GPUI&apos;s constraint rather than this
        library&apos;s — a callback outlives the frame that built it.
      </p>
      <p>
        The verbs follow v3 exactly rather than being regularised: <code>on_press</code> on the
        controls v3 documents <code>onPress</code> for, <code>on_close</code> where it documents{" "}
        <code>onClose</code>, <code>on_change</code> where it documents <code>onChange</code>. If
        you know the React API you already know which one a component takes.
      </p>
    </>
  );
}
