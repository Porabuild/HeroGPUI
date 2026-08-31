import type { Metadata } from "next";
import type { ComponentProps } from "react";
import { Button, Chip, Disclosure, Link } from "@heroui/react";
import { PageHeader } from "@/components/ui/page-header";
import { Callout } from "@/components/ui/callout";
import changelog from "@/data/changelog.json";

const RELEASE_VERSION = changelog.version;

export const metadata: Metadata = {
  title: "Releases",
  description: `HeroGPUI v${RELEASE_VERSION} brings HeroUI's component system, typed Rust builders, OKLCH tokens and a native desktop gallery to Rust applications.`,
};

/*
 * `changelog.json` is generated from git by scripts/extract-changelog.mjs.
 * The log below keeps its subjects, short SHAs and kinds verbatim.
 */

type Kind = "feature" | "parity" | "fix" | "docs" | "infra";

const KIND_CHIP: Record<Kind, { color: ComponentProps<typeof Chip>["color"]; label: string }> = {
  feature: { color: "accent", label: "Feature" },
  parity: { color: "success", label: "Components" },
  fix: { color: "warning", label: "Fix" },
  docs: { color: "default", label: "Docs" },
  infra: { color: "default", label: "Infra" },
};

const KIND_ORDER: Kind[] = ["feature", "parity", "fix", "docs", "infra"];

function KindChip({ kind }: { kind: string }) {
  const chip = KIND_CHIP[kind as Kind] ?? { color: "default" as const, label: kind };
  return (
    <Chip color={chip.color} size="sm" variant="soft">
      {chip.label}
    </Chip>
  );
}

const RELEASE_HIGHLIGHTS = [
  {
    title: "Component library",
    detail: "Use HeroUI's component system as typed Rust builders with explicit state.",
  },
  {
    title: "Theme tokens",
    detail: "Build with OKLCH semantic tokens and switch light and dark themes at runtime.",
  },
  {
    title: "Desktop gallery",
    detail: "Explore every component in a native gallery that ships with the library.",
  },
];

const DAY_FORMAT = new Intl.DateTimeFormat("en-US", {
  year: "numeric",
  month: "long",
  day: "numeric",
  timeZone: "UTC",
});

function formatDay(date: string) {
  return DAY_FORMAT.format(new Date(`${date}T00:00:00Z`));
}

/** The N most recent days render open; the rest collapse behind the disclosure. */
const RECENT_DAYS = 3;

type Commit = (typeof changelog.days)[number]["commits"][number];
type Day = (typeof changelog.days)[number];

function DayLog({ day }: { day: Day }) {
  return (
    <section aria-labelledby={`log-${day.date}`} className="mt-8 first:mt-0">
      <h3 id={`log-${day.date}`}>
        {formatDay(day.date)}
        <span className="ml-2 text-sm font-normal text-muted">
          {day.commits.length} {day.commits.length === 1 ? "commit" : "commits"}
        </span>
      </h3>
      <ol className="mt-3 space-y-1.5">
        {day.commits.map((commit: Commit) => (
          <li key={commit.sha} className="flex items-baseline gap-3 text-sm">
            <code className="shrink-0 font-mono text-xs text-muted" title={commit.sha}>
              {commit.sha}
            </code>
            <span className="min-w-0 flex-1">{commit.subject}</span>
            <span className="shrink-0">
              <KindChip kind={commit.kind} />
            </span>
          </li>
        ))}
      </ol>
    </section>
  );
}

const allCommits: Commit[] = changelog.days.flatMap((day) => day.commits);
const kindCounts = KIND_ORDER.map((kind) => ({
  kind,
  count: allCommits.filter((commit) => commit.kind === kind).length,
})).filter((entry) => entry.count > 0);

const recentDays = changelog.days.slice(0, RECENT_DAYS);
const olderDays = changelog.days.slice(RECENT_DAYS);
const olderCommitCount = olderDays.reduce((sum, day) => sum + day.commits.length, 0);

const FIRST_COMMIT_DAY = changelog.days[changelog.days.length - 1]?.date;
const LAST_COMMIT_DAY = changelog.days[0]?.date;

export default function ReleasesPage() {
  return (
    <>
      <PageHeader
        title="Releases"
        description={`The v${RELEASE_VERSION} release, its contents, the development log, and how versions are published.`}
      />

      <Callout kind="note" title={`What v${RELEASE_VERSION} contains`}>
        <p>
          HeroGPUI brings HeroUI&apos;s component system to Rust desktop applications as typed
          builders, with OKLCH semantic tokens and a native desktop gallery.
        </p>
        <p className="mt-2">
          It runs on Windows, macOS and Linux from one codebase. The gallery documents every
          component and ships with the library.
        </p>
      </Callout>

      <h2 id="current-development-line" className="mt-12">
        What v{RELEASE_VERSION} contains
      </h2>
      <p>
        This release gives Rust desktop applications the full HeroUI component system, typed
        builders with explicit state, OKLCH semantic tokens and a desktop gallery with live
        documentation. Components support reduced motion, and the library runs on Windows, macOS and
        Linux from one codebase.
      </p>
      <ul className="mt-4 space-y-2">
        {RELEASE_HIGHLIGHTS.map((item) => (
          <li key={item.title}>
            <strong>{item.title}</strong> — {item.detail}
          </li>
        ))}
      </ul>

      <h2 id="development-log" className="mt-12">
        Development log
      </h2>
      <p>
        Every commit from <time dateTime={FIRST_COMMIT_DAY}>{formatDay(FIRST_COMMIT_DAY)}</time> to{" "}
        <time dateTime={LAST_COMMIT_DAY}>{formatDay(LAST_COMMIT_DAY)}</time> —{" "}
        {changelog.commitCount} commits, subjects verbatim, newest first. It records the work that
        shaped v{RELEASE_VERSION}.
      </p>
      <p className="mt-3 flex flex-wrap items-center gap-2 text-sm text-muted">
        <span>Commit categories:</span>
        {kindCounts.map(({ kind, count }) => (
          <Chip key={kind} color={KIND_CHIP[kind].color} size="sm" variant="soft">
            {KIND_CHIP[kind].label} · {count}
          </Chip>
        ))}
      </p>

      <div className="mt-6">
        {recentDays.map((day) => (
          <DayLog key={day.date} day={day} />
        ))}

        {olderDays.length > 0 && (
          <Disclosure defaultExpanded={false} className="mt-8">
            <Disclosure.Heading>
              <Button
                className="w-full justify-between rounded-xl border border-border/70 px-4 py-3 font-medium text-foreground hover:bg-surface-secondary"
                slot="trigger"
                variant="ghost"
              >
                The older {olderDays.length} days — {olderCommitCount} commits
                <Disclosure.Indicator className="text-muted" />
              </Button>
            </Disclosure.Heading>
            <Disclosure.Content>
              <Disclosure.Body className="pt-2">
                {olderDays.map((day) => (
                  <DayLog key={day.date} day={day} />
                ))}
              </Disclosure.Body>
            </Disclosure.Content>
          </Disclosure>
        )}
      </div>

      <h2 id="release-process" className="mt-12">
        How releases work
      </h2>
      <p>
        HeroGPUI uses one version across its crates and the <code>herogpui-gallery</code> CLI. Each
        release is built from a <code>vX.Y.Z</code> Git tag, and the tagged build publishes the
        crates and gallery artifacts together. See the{" "}
        <Link href="/docs/getting-started/installation">installation guide</Link> to add HeroGPUI to
        a Rust project.
      </p>
    </>
  );
}
