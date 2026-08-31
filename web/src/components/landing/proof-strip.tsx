/** Product facts that help a visitor place the library quickly. */

interface Stat {
  value: string;
  label: string;
  accent?: boolean;
}

const STATS: Stat[] = [
  { value: "71", label: "Components across 66 catalog pages" },
  { value: "3", label: "Desktop platforms" },
  { value: "OKLCH", label: "Semantic color tokens" },
  { value: "Rust", label: "Typed builder APIs" },
  { value: "GPUI", label: "Native desktop renderer", accent: true },
];

export function ProofStrip() {
  return (
    <section className="landing-proof-strip border-y border-separator bg-surface-secondary/50">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-14 sm:px-6 md:py-16">
        <dl className="grid grid-cols-2 gap-y-8 sm:grid-cols-3 lg:grid-cols-5">
          {STATS.map((stat) => (
            <div
              className="landing-stat border-separator pr-6 lg:border-l lg:px-6 lg:first:border-l-0 lg:first:pl-0"
              key={stat.label}
            >
              <dd
                className={`text-3xl font-semibold tracking-tight tabular-nums sm:text-4xl ${
                  stat.accent ? "text-accent" : "text-foreground"
                }`}
              >
                {stat.value}
              </dd>
              <dt className="mt-1.5 text-xs leading-snug text-muted">{stat.label}</dt>
            </div>
          ))}
        </dl>
      </div>
    </section>
  );
}
