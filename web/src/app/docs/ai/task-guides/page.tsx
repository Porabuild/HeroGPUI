import type { Metadata } from "next";
import { readFileSync } from "node:fs";
import path from "node:path";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { C, H2, Md, P, Td, Th } from "@/app/docs/ai/_components/docs";

export const metadata: Metadata = {
  title: "Task guides",
  description:
    "The four agent task guides in docs/agents: what each owns, when to load it, and the change-to-verification matrix.",
};

// Read the guides at build time so this page cannot drift from the
// instructions agents actually receive. The "when" line under each guide is
// the guide's own opening sentence; the verification matrix is parsed out of
// workflow.md rather than copied.
function guideSource(file: string): string {
  return readFileSync(path.join(process.cwd(), "..", "docs", "agents", file), "utf8");
}

const WORKFLOW_MD = guideSource("workflow.md");
const COMPONENTS_MD = guideSource("components.md");
const PARITY_MD = guideSource("parity.md");
const GALLERY_MD = guideSource("gallery.md");

/** The guide's opening "Read this guide …" sentence, joined across line wraps. */
function readWhen(source: string): string {
  const lines = source.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith("Read this guide"));
  const paragraph: string[] = [];
  for (const line of lines.slice(start)) {
    if (line.trim().length === 0) break;
    paragraph.push(line.trim());
  }
  const joined = paragraph.join(" ");
  const end = joined.indexOf(". ");
  return end === -1 ? joined : joined.slice(0, end + 1);
}

/** The change-to-verification table under "Verification by change type". */
function verificationRows(source: string): Array<[string, string, string]> {
  const lines = source.split(/\r?\n/);
  const start = lines.findIndex((line) => line.startsWith("## Verification by change type"));
  const rows: Array<[string, string, string]> = [];
  for (const line of lines.slice(start + 1)) {
    if (line.startsWith("## ")) break;
    const match = /^\|(.+)\|$/.exec(line.trim());
    if (!match) continue;
    const cells = match[1].split("|").map((cell) => cell.trim());
    if (cells.length !== 3 || cells[0] === "Change" || cells[0].startsWith("---")) continue;
    rows.push([cells[0], cells[1], cells[2]]);
  }
  return rows;
}

const GUIDES: Array<{ name: string; file: string; when: string; owns: string }> = [
  {
    name: "Workflow and architecture",
    file: "workflow.md",
    when: readWhen(WORKFLOW_MD),
    owns: "Repository map, source hierarchy, scope discipline, project invariants, and the change-to-verification matrix below. Start here for every implementation task.",
  },
  {
    name: "Component implementation",
    file: "components.md",
    when: readWhen(COMPONENTS_MD),
    owns: "Component model and controlled state, state lifetime, events and focus, overlays, layout and animation, virtual collections, and the headless behavior-test harness.",
  },
  {
    name: "Upstream contract and audits",
    file: "parity.md",
    when: readWhen(PARITY_MD),
    owns: "Pinned upstream contract, the audit suite and which audit owns which claim, API ownership and omission rules, and audit-reader integrity.",
  },
  {
    name: "Gallery and visual verification",
    file: "gallery.md",
    when: readWhen(GALLERY_MD),
    owns: "Rebuild and smoke scripts, off-screen and foreground input drivers, deep links and environment controls, screenshot integrity, and behavioral proof.",
  },
];

const VERIFICATION_ROWS = verificationRows(WORKFLOW_MD);

export default function TaskGuidesPage() {
  return (
    <>
      <PageHeader
        title="Task guides"
        description="Four guides under docs/agents/ carry the per-task detail behind AGENTS.md. An agent loads the guide for the work at hand."
      />

      <P>
        The root <C>AGENTS.md</C> names the rules every task shares. The guides hold the detail that
        only some tasks need, so each task reads the root file plus the guide for its kind of work.
        Scoped <C>AGENTS.md</C> files under <C>.shots/</C>, <C>crates/herogpui-components/</C>, and{" "}
        <C>gallery/</C> add local rules and link back to the guide that owns their topic.
      </P>

      <H2 id="the-four-guides">The four guides</H2>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Guide</Th>
              <Th>File</Th>
              <Th>When to load</Th>
              <Th>Owns</Th>
            </tr>
          </thead>
          <tbody>
            {GUIDES.map((guide) => (
              <tr key={guide.file}>
                <Td className="whitespace-nowrap font-medium text-foreground">{guide.name}</Td>
                <Td className="whitespace-nowrap">
                  <C>docs/agents/{guide.file}</C>
                </Td>
                <Td className="text-muted">
                  <Md text={guide.when} />
                </Td>
                <Td className="text-muted">{guide.owns}</Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H2 id="verification-matches-the-change">Verification matches the change</H2>
      <P>
        The workflow guide maps each change type to the command to iterate with and the gate
        required before broad handoff:
      </P>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Change</Th>
              <Th>Iterate with</Th>
              <Th>Before broad handoff</Th>
            </tr>
          </thead>
          <tbody>
            {VERIFICATION_ROWS.map(([change, iterate, gate]) => (
              <tr key={change}>
                <Td className="whitespace-nowrap font-medium text-foreground">{change}</Td>
                <Td className="text-muted">
                  <Md text={iterate} />
                </Td>
                <Td className="text-muted">
                  <Md text={gate} />
                </Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <P>
        Normal tests run once. Do not claim the full gate passed after a focused test, and finish
        with evidence: report which checks ran, which did not, and why. A screenshot proves pixels,
        a headless behavior test proves the exercised event path, and an audit proves only its
        mapped surface.
      </P>
      <P>
        <Link href="/docs/ai/llms-txt">llms.txt</Link> carries the component API reference and
        supplements these guides. How the layers fit together is described on the{" "}
        <Link href="/docs/ai/agents-md">repository guide</Link> page.
      </P>
    </>
  );
}
