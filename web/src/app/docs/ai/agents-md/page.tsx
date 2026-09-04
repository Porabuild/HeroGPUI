import type { Metadata } from "next";
import { readFileSync } from "node:fs";
import path from "node:path";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { C, H2, H3, Li, P, Td, Th, Ul } from "@/app/docs/ai/_components/docs";
import Link from "next/link";

export const metadata: Metadata = {
  title: "AGENTS.md",
  description:
    "How the repository layers AGENTS.md, task guides, and scoped rules so coding agents load the right context.",
};

// Build the copyable guide from its authoritative repository source so this
// documentation cannot drift from the instructions agents actually receive.
const ROOT_AGENTS_MD = readFileSync(path.join(process.cwd(), "..", "AGENTS.md"), "utf8");

const LAYERS: Array<[string, string, string]> = [
  [
    "Root AGENTS.md",
    "AGENTS.md",
    "The entry file every agent reads. Names the supported targets, before-editing rules, core commands, task guides, and component-surface synchronization contract. This page reads that file directly at build time instead of maintaining a second copy.",
  ],
  [
    "Task guides",
    "docs/agents/*.md (four files)",
    "The middle layer, loaded per task. Each guide owns one kind of work and points to the next when a task crosses boundaries.",
  ],
  [
    "Scoped AGENTS.md",
    ".shots/, crates/herogpui-components/, gallery/",
    "Local rules that apply only inside a subtree — audit-reader integrity for .shots, builder and test discipline for the component crate, demo and reference-metadata rules for the gallery. Each links back to the task guide that owns its topic.",
  ],
  [
    "Website subtree",
    "web/AGENTS.md",
    "The Next.js app carries its own warning — that its Next version has breaking changes versus an agent's training data and that the vendored docs under node_modules/next/dist/docs/ are authoritative. next dev re-adds this block; web/CLAUDE.md pulls it in with the @AGENTS.md import directive.",
  ],
];

const GUIDES: Array<[string, string, string]> = [
  [
    "Workflow and architecture",
    "workflow.md",
    "The repository map and source hierarchy — which source owns which claim, down to the pinned React Aria / Stately versions for inherited behavior. Also the project invariants and the change-to-verification matrix below.",
  ],
  [
    "Component implementation",
    "components.md",
    "GPUI 0.2.2 constraints that repeatedly produce plausible-but-wrong component code: controlled/uncontrolled semantics, keyed-state lifetimes, focus and overlay rules, and the headless behavior-test harness patterns.",
  ],
  [
    "Upstream contract and audits",
    "parity.md",
    "The pinned upstream contract and the audit suite that proves it — which audit owns which claim, what a recorded omission must look like, and audit-reader integrity (an audit must fail loudly when it cannot find its input; empty input is not a zero-gap result).",
  ],
  [
    "Gallery and visual verification",
    "gallery.md",
    "How to rebuild, smoke-test, deep-link, drive real input off-screen, and capture screenshots without stealing the user's desktop — plus the rule that a screenshot proves pixels, not behavior.",
  ],
];

export default function AgentsMdPage() {
  return (
    <>
      <PageHeader
        title="AGENTS.md"
        description="The repository layers a short root instruction file, four task guides, and scoped rules so coding agents load the right context."
      />

      <P>
        <C>AGENTS.md</C> is the emerging convention for repository-level instructions that coding
        agents read before working; <C>CLAUDE.md</C> is Claude Code&apos;s name for the same thing.
        A single monolithic file scales badly — everything an agent might ever need lands in the
        context of every task, and the rules that actually matter drown. HeroGPUI instead layers its
        instructions so each task reads only what it needs.
      </P>

      <H2 id="structure">The structure</H2>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Layer</Th>
              <Th>Where</Th>
              <Th>Role</Th>
            </tr>
          </thead>
          <tbody>
            {LAYERS.map(([layer, where, role]) => (
              <tr key={layer}>
                <Td className="whitespace-nowrap font-medium text-foreground">{layer}</Td>
                <Td>
                  <C>{where}</C>
                </Td>
                <Td className="text-muted">{role}</Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <P>
        There is exactly one root file to maintain: the root <C>CLAUDE.md</C> is a symlink to{" "}
        <C>AGENTS.md</C>, so both agent tools read the identical text. The task guides are plain
        markdown under <C>docs/agents/</C>, referenced from the root file and from the scoped{" "}
        <C>AGENTS.md</C> files.
      </P>

      <H3 id="the-four-task-guides">The four task guides</H3>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Guide</Th>
              <Th>File</Th>
              <Th>Owns</Th>
            </tr>
          </thead>
          <tbody>
            {GUIDES.map(([guide, file, owns]) => (
              <tr key={file}>
                <Td className="whitespace-nowrap font-medium text-foreground">{guide}</Td>
                <Td>
                  <C>docs/agents/{file}</C>
                </Td>
                <Td className="text-muted">{owns}</Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <P>
        The root file names the layering explicitly: <C>llms.txt</C> (see{" "}
        <Link href="/docs/ai/llms-txt">llms.txt</Link>) supplements the task guides and does not
        replace reading the implementation and tests.
      </P>

      <H2 id="the-rules-and-why">The rules, and why they exist</H2>

      <H3 id="preserve-the-dirty-checkout">Preserve the dirty checkout</H3>
      <P>
        Rule one of the root file: run <C>git status --short</C> and inspect the relevant diff
        before editing, and treat every pre-existing modification as user-owned. This checkout
        routinely carries unfinished component, gallery, audit, and screenshot work in parallel with
        whatever an agent was asked to do. The workflow guide extends the rule: do not format,
        revert, regenerate, or include unrelated paths, and a read-only review does not authorize
        edits, builds, or test runs. The cost of ignoring it is destroying someone&apos;s
        in-progress work; the cost of following it is one extra command.
      </P>

      <H3 id="pin-every-upstream-contract">Pin every upstream contract</H3>
      <P>
        Rule three names the supported framework targets: GPUI <strong>0.2.2</strong> and Rust 1.98.
        &ldquo;Newer upstream APIs are not evidence that an API is available here&rdquo; is the
        first paragraph of the root file. Check <Link href="/docs/ai/llms-txt">llms.txt</Link> and
        the task guides before using an API that is not present in the checkout.
      </P>

      <H3 id="verification-matches-the-change">Verification matches the change</H3>
      <P>
        The workflow guide maps each change type to the command that iterates on it and the gate
        required before broad handoff — for example, a Rust logic change iterates with a focused{" "}
        <C>cargo test -p herogpui-components --test &lt;name&gt;</C> and finishes with the component
        suite, format, lint, and the relevant audits, while a documentation change verifies links
        and commands rather than compiling Rust. Two integrity rules close the loop: do not claim
        the full gate passed after running a focused test, and finish with evidence — report which
        checks ran, which did not, and why.
      </P>

      <Callout kind="tip" title="Why the file stays short">
        The root <C>AGENTS.md</C> is 44 lines. Everything else is one link away, scoped to the
        subtree or task that needs it. When a rule would only matter for one kind of work, it
        belongs in the guide for that work — not in the file every task pays for.
      </Callout>

      <H2 id="adapting-the-pattern">Adapting the pattern</H2>
      <P>
        The root file in full — short enough to read on every task, specific enough that an agent
        who stops here still avoids the two classic mistakes (clobbering the dirty checkout and
        using an unsupported upstream API):
      </P>
      <div className="mt-6">
        <CodeBlock code={ROOT_AGENTS_MD} lang="plaintext" filename="AGENTS.md" />
      </div>
      <P>What transfers to another repository:</P>
      <Ul>
        <Li>
          Keep the entry file under a screenful; move per-task detail into guides it links to.
        </Li>
        <Li>
          State the pinned dependency versions in the first paragraph; it is the cheapest defense
          against training-data drift.
        </Li>
        <Li>
          Put &ldquo;inspect the working tree first&rdquo; in the rules if contributors (human or
          agent) share one checkout.
        </Li>
        <Li>
          Add scoped <C>AGENTS.md</C> files only where a subtree has genuinely local rules, and have
          each one link back to the owning guide.
        </Li>
        <Li>
          Symlink <C>CLAUDE.md</C> to <C>AGENTS.md</C> so the two tool conventions never diverge.
        </Li>
      </Ul>
    </>
  );
}
