import { SectionHeading } from "@/components/landing/shared";

/** The library's core building blocks, presented as a compact index. */

const FEATURES = [
  {
    title: "OKLCH semantic tokens",
    body: "Define base colors, surfaces and roles with OKLCH semantic tokens. Hover, soft and foreground variants derive from each base color.",
  },
  {
    title: "Typed Rust builders",
    body: "Build interfaces from typed Rust builders. The API stays close to the structure you render.",
  },
  {
    title: "Controlled or uncontrolled state",
    body: "Choose controlled state with is_selected, is_open or selected_key, or let a component manage its own default.",
  },
  {
    title: "Validation for fields and forms",
    body: "Use validation::resolve for field state. Form carries submit, invalid and reset handlers.",
  },
  {
    title: "Reduced-motion support",
    body: "Components respect reduced-motion settings, including overlay transitions.",
  },
  {
    title: "A desktop gallery",
    body: "Open the gallery for examples of every component, with a theme switcher and deep links for captures.",
  },
] as const;

export function Features() {
  return (
    <section className="landing-features">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-16 sm:px-6 md:py-24">
        <SectionHeading
          eyebrow="What you get"
          sub="Typed builders, explicit state and semantic tokens keep the API close to the UI you render."
          title="The pieces desktop apps need"
        />

        <ol className="mt-12">
          {FEATURES.map((feature, index) => (
            <li
              className="grid items-baseline gap-2 border-t border-separator py-6 last:border-b md:grid-cols-[3rem_minmax(0,15rem)_1fr] md:gap-6 md:py-7"
              key={feature.title}
            >
              <span aria-hidden="true" className="font-mono text-xs text-accent tabular-nums">
                {String(index + 1).padStart(2, "0")}
              </span>
              <h3 className="text-base font-semibold tracking-tight text-foreground">
                {feature.title}
              </h3>
              <p className="max-w-2xl text-sm leading-relaxed text-muted">{feature.body}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
