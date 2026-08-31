import type { Metadata } from "next";
import { Button } from "@heroui/react";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";

export const metadata: Metadata = {
  title: "Design Principles",
  description: "Guidance for building consistent HeroGPUI interfaces in Rust.",
};

const SEMANTIC = `// Hierarchy, not appearance.
Button::new("save").label("Save")                              // primary
Button::new("edit").label("Edit").variant(Variant::Secondary)
Button::new("cancel").label("Cancel").variant(Variant::Tertiary)
Button::new("del").label("Delete").variant(Variant::Danger)`;

const COMPOSITION = `// The same parts, as slots. \`input\` takes an \`Input\`, not an
// element, so the group can strip the field's own chrome.
InputGroup::new()
    .prefix(InputAddon::new("$"))
    .input(Input::new(amount).placeholder("0.00"))
    .suffix(InputAddon::new("USD"))`;

const DISCLOSURE = `// Level 1
Button::new("go").label("Click me")

// Level 2
Button::new("go").size(Size::Lg).child(check).child("Submit")

// Level 3
Button::new("go").label("Submitting").is_pending(true)`;

const PREDICTABLE = `// The same three props, on three different components.
Button::new("b").size(Size::Lg).is_disabled(true)
Chip::new().size(Size::Lg).child(ChipLabel::new().child("c"))
Avatar::new("a").size(Size::Lg)

// And one callback shape everywhere.
.on_change(|value: &str, _window, _cx| { /* ... */ })`;

const TYPES = `// A variant is an enum, so this does not compile:
//     Button::new("b").variant(Variant::Solid)
//                                      ^^^^^ no variant named \`Solid\`
//
// and an exhaustive match cannot miss one:
match variant {
    Variant::Primary => ..,
    Variant::Secondary => ..,
    Variant::Tertiary => ..,
    Variant::Outline => ..,
    Variant::Ghost => ..,
    Variant::Danger => ..,
    Variant::DangerSoft => ..,
}`;

const SEPARATION = `herogpui-core        // Color, Variant, Size, oklch(), mix_oklab()
herogpui-theme       // the tokens + ThemeProvider (no component code)
herogpui-components  // the components
herogpui             // umbrella re-export

// Read a token without touching a component:
let accent = cx.role(Color::Accent).color;
let radius = herogpui::components::util::field_radius(cx);`;

const CUSTOM = `// Override one base token; every derived value follows.
let violet = Theme::builder("violet", Theme::light())
    .accent(oklch(0.55, 0.23, 295.0))
    .build();

// \`accent.hover()\` and \`accent.soft()\` are the same color-mix
// expressions, so they move with the base color.`;

export default function DesignPrinciplesPage() {
  return (
    <>
      <PageHeader
        title="Design Principles"
        description="Guidance for building consistent HeroGPUI interfaces in Rust."
      />

      <p>
        Use these ten principles when choosing components, structuring state and shaping an
        interface. They cover the decisions that keep a HeroGPUI application clear as it grows.
      </p>

      <h2 id="1-semantic-intent-over-visual-style">1. Semantic intent over visual style</h2>
      <p>
        Choose variants by the action they represent. Use primary for the main action, secondary for
        an alternative, tertiary for a low-emphasis action, and danger for destructive work. The
        names communicate hierarchy without relying on color alone:
      </p>
      <div className="docs-stage mt-4 flex flex-wrap items-center gap-3 rounded-xl border border-separator p-4">
        <Button variant="primary">Save</Button>
        <Button variant="secondary">Edit</Button>
        <Button variant="tertiary">Cancel</Button>
      </div>
      <div className="mt-4">
        <CodeBlock code={SEMANTIC} lang="rust" />
      </div>

      <h2 id="2-accessibility-as-foundation">2. Accessibility as foundation</h2>
      <p>
        Design keyboard and focus behavior into every interactive flow. GPUI provides focus
        handling, keyboard navigation and dismissal keys, but it does not expose an accessibility
        tree, so ARIA-only annotations are not part of the component API.
      </p>
      <p>
        Test the behavior that users operate directly: <code>Escape</code> to dismiss, arrow keys
        through a menu, and typing into a date-field segment.
      </p>

      <h2 id="3-composition-over-configuration">3. Composition over configuration</h2>
      <p>
        Compose parts through named builder slots: <code>Modal.Close</code>,{" "}
        <code>Card.Header</code> and <code>InputGroup.Prefix</code> become methods on the Rust
        builder. Use the typed component form when a slot carries behavior so the parent can still
        configure it.
      </p>
      <div className="mt-4">
        <CodeBlock code={COMPOSITION} lang="rust" />
      </div>

      <h2 id="4-progressive-disclosure">4. Progressive disclosure</h2>
      <p>
        Start with the constructor and add options only when the interface needs them. The three
        buttons below show increasing levels of configuration:
      </p>
      <div className="docs-stage mt-4 flex flex-wrap items-center gap-3 rounded-xl border border-separator p-4">
        <Button>Click me</Button>
        <Button size="lg">Submit</Button>
        <Button isPending>Submitting</Button>
      </div>
      <div className="mt-4">
        <CodeBlock code={DISCLOSURE} lang="rust" />
      </div>

      <h2 id="5-predictable-behaviour">5. Predictable behaviour</h2>
      <p>
        Keep shared props consistent across the application. <code>size</code> uses <code>sm</code>/
        <code>md</code>/<code>lg</code>, <code>is_disabled</code> has the same meaning on each
        control, and callbacks use the component&apos;s documented signature.
      </p>
      <div className="mt-4">
        <CodeBlock code={PREDICTABLE} lang="rust" />
      </div>

      <h2 id="6-type-safety-first">6. Type safety first</h2>
      <p>
        Prefer the Rust types over stringly-typed configuration. A variant is an enum, so a typo is
        a compile error rather than a silently unstyled control, and an exhaustive{" "}
        <code>match</code> over <code>Variant</code> covers every case.
      </p>
      <div className="mt-4">
        <CodeBlock code={TYPES} lang="rust" />
      </div>

      <h2 id="7-separation-of-styles-and-logic">7. Separation of styles and logic</h2>
      <p>
        Keep shared vocabulary, theme tokens and component implementations separate.{" "}
        <code>herogpui-core</code> provides shared types and color math, <code>herogpui-theme</code>{" "}
        provides tokens, and <code>herogpui-components</code> provides the components. The theme
        crate has no component code, so other widgets can read the same tokens.
      </p>
      <div className="mt-4">
        <CodeBlock code={SEPARATION} lang="rust" />
      </div>

      <h2 id="8-developer-experience">8. Developer experience</h2>
      <p>
        Use <code>rustdoc</code> for builder-level API details and the gallery for runnable
        examples. The gallery has one page per component and includes the documented examples.
      </p>

      <h2 id="9-complete-customization">9. Complete customization</h2>
      <p>
        Start from a built-in theme and override a base token when your application needs a
        different value. Derived colors follow through the same <code>color-mix</code> rules used by
        the theme.
      </p>
      <div className="mt-4">
        <CodeBlock code={CUSTOM} lang="rust" />
      </div>

      <h2 id="10-open-and-extensible">10. Open and extensible</h2>
      <p>
        The tokens, color math and motion curves are public. A component outside this crate can read{" "}
        <code>cx.colors()</code>, use <code>util::field_radius(cx)</code> for its corners and
        animate with <code>Motion::LIST_IN</code>.
      </p>

      <h2 id="supported-vocabulary">Use the supported vocabulary</h2>
      <p>
        Use semantic roles, surfaces and the builder methods documented for each component. For
        example, <code>isPending</code> is spelled <code>is_pending</code> in Rust; the component
        reference is the source of truth for every available option.
      </p>
    </>
  );
}
