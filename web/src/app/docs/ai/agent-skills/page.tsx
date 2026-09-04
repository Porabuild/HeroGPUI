import type { Metadata } from "next";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { C, H2, H3, Li, Md, P, Td, Th, Ul } from "@/app/docs/ai/_components/docs";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Agent Skills",
  description:
    "The repository's gpui and rust-best-practices skills, their source paths, mirrors, and content hashes.",
};

const LOCK_JSON = `{
  "version": 1,
  "skills": {
    "gpui": {
      "source": "longbridge/gpui-component",
      "sourceType": "github",
      "skillPath": "skills/gpui/SKILL.md",
      "computedHash": "123346a2eb877e8221889611f20025eb742f87913dfa2a5a4f4958cbbbc12bab"
    },
    "rust-best-practices": {
      "source": "apollographql/skills",
      "sourceType": "github",
      "skillPath": "skills/rust-best-practices/SKILL.md",
      "computedHash": "5ae68ae0241d65dac8a73f06139457206dde55912aecd4c36cd462fa85570513"
    }
  }
}`;

const INSTALL_SH = `# from a checkout of the repo consuming the skills
mkdir -p .agents/skills
cp -R /path/to/HeroGPUI/.agents/skills/gpui .agents/skills/gpui

# point your agent's skill directory at the canonical copy, as HeroGPUI does
mkdir -p .claude/skills
ln -s ../.agents/skills/gpui .claude/skills/gpui`;

const GPUI_CORE_REFERENCES: Array<[string, string, string]> = [
  ["Actions & keybindings", "action.md", "`actions!`, `bind_keys`, `on_action`, `key_context`"],
  ["Async & background tasks", "async.md", "`cx.spawn`, `background_spawn`, `Task`, async I/O"],
  ["Context management", "context.md", "`App`, `Window`, `Context<T>`, `AsyncApp`"],
  [
    "Custom elements (low-level)",
    "element.md",
    "`Element` trait, `request_layout`, `prepaint`, `paint`",
  ],
  ["Entity state", "entity.md", "`Entity<T>`, `WeakEntity`, state management"],
  ["Events & subscriptions", "event.md", "`cx.emit`, `cx.subscribe`, `cx.observe`"],
  ["Focus & keyboard nav", "focus-handle.md", "`FocusHandle`, `track_focus`, Tab navigation"],
  ["Global state", "global.md", "`Global` trait, `cx.set_global`, app-wide config"],
  [
    "Layout & styling",
    "layout-style.md",
    "`div()`, `h_flex()`, `v_flex()`, flexbox, overflow, positioning",
  ],
  ["ElementId", "element-id.md", "`ElementId`, `.id()`, uniqueness rules, stateful elements"],
  ["Testing", "test.md", "`#[gpui::test]`, `TestAppContext`, `VisualTestContext`"],
];

const GPUI_EXTENDED_REFERENCES: Array<[string, string[]]> = [
  [
    "Element trait",
    [
      "element-api.md (complete API, hitbox system, event handling)",
      "element-patterns.md (text, interactive, container, composite)",
      "element-examples.md (full worked examples)",
      "element-best-practices.md (performance, state, pitfalls)",
      "element-advanced.md (masonry/circular layouts, async updates, virtual lists)",
    ],
  ],
  [
    "Entity management",
    [
      "entity-api.md (complete Entity API, lifecycle)",
      "entity-patterns.md (model-view, cross-entity communication, observer)",
      "entity-best-practices.md (memory, performance, lifecycle)",
      "entity-advanced.md (collections, registry, debounce, state machines)",
    ],
  ],
  [
    "Testing",
    [
      "test-examples.md (testing examples and patterns)",
      "test-reference.md (complete testing API reference)",
    ],
  ],
];

const RUST_CHAPTERS: Array<[string, string, string]> = [
  [
    "1",
    "chapter_01.md",
    "Coding styles and idioms — borrowing vs cloning, `Copy`, `Option`/`Result` handling, iterators, when to extract a function",
  ],
  [
    "2",
    "chapter_02.md",
    "Clippy and linting — configuration, important lints, workspace lint setup",
  ],
  [
    "3",
    "chapter_03.md",
    "Performance mindset — profiling, redundant clones, stack vs heap, zero-cost abstractions",
  ],
  [
    "4",
    "chapter_04.md",
    "Error handling — `Result` vs panic, `thiserror` vs `anyhow`, error hierarchies",
  ],
  ["5", "chapter_05.md", "Automated testing — naming, one assertion per test, snapshot testing"],
  ["6", "chapter_06.md", "Generics and dispatch — static vs dynamic dispatch, trait objects"],
  ["7", "chapter_07.md", "The type state pattern — compile-time state safety, when to use it"],
  ["8", "chapter_08.md", "Comments vs documentation — when to comment, doc comments, rustdoc"],
  ["9", "chapter_09.md", "Understanding pointers — thread safety, `Send`/`Sync`, pointer types"],
];

export default function AgentSkillsPage() {
  return (
    <>
      <PageHeader
        title="Agent Skills"
        description="The repository's gpui and rust-best-practices skills, their source paths, mirrors, and content hashes."
      />

      <P>
        A skill is a folder of instructions (<C>SKILL.md</C> plus reference files) that a coding
        agent loads when a task touches its topic. This repository uses two: one for GPUI 0.2.2 and
        one for idiomatic Rust. They document the <em>framework</em>; the component-library contract
        they assume is in <Link href="/docs/ai/llms-txt">llms.txt</Link>.
      </P>

      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Skill</Th>
              <Th>Source</Th>
              <Th>Covers</Th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <Td>
                <C>gpui</C>
              </Td>
              <Td>
                <a
                  className="text-link transition-colors hover:text-accent-soft no-underline"
                  href="https://github.com/longbridge/gpui-component"
                  target="_blank"
                  rel="noreferrer"
                >
                  longbridge/gpui-component
                </a>
              </Td>
              <Td className="text-muted">
                <Md text="GPUI framework knowledge: actions, async tasks, context types, the low-level `Element` trait, entity state, events, focus, globals, layout, and testing." />
              </Td>
            </tr>
            <tr>
              <Td>
                <C>rust-best-practices</C>
              </Td>
              <Td>
                <a
                  className="text-link transition-colors hover:text-accent-soft no-underline"
                  href="https://github.com/apollographql/skills"
                  target="_blank"
                  rel="noreferrer"
                >
                  apollographql/skills
                </a>
              </Td>
              <Td className="text-muted">
                <Md text="Idiomatic Rust: borrowing vs cloning, clippy, performance, error handling, testing, generics and dispatch, the type state pattern, documentation." />
              </Td>
            </tr>
          </tbody>
        </table>
      </div>

      <Callout kind="note" title="One canonical copy">
        The source of truth is <C>.agents/skills/</C>. <C>.claude/skills/</C> (Claude Code) and{" "}
        <C>.factory/skills/</C> (Factory) are symlinks to it, so every tool reads the same bytes and
        an update lands once.
      </Callout>

      <H2 id="gpui">The gpui skill</H2>
      <P>
        Vendored from <C>longbridge/gpui-component</C> (<C>skills/gpui/SKILL.md</C> upstream). Its
        front matter scopes it to &ldquo;working with any GPUI framework concept, building GPUI
        applications, or needing guidance on GPUI-specific APIs and patterns&rdquo;. The skill is
        progressive: a short <C>SKILL.md</C> holds a navigation table, and the substance lives in 22
        reference files — 11 core topics plus 11 extended deep-dives — that the agent loads for the
        topic at hand.
      </P>

      <H3 id="gpui-core">Core references</H3>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Topic</Th>
              <Th>File</Th>
              <Th>When to load</Th>
            </tr>
          </thead>
          <tbody>
            {GPUI_CORE_REFERENCES.map(([topic, file, when]) => (
              <tr key={file}>
                <Td className="whitespace-nowrap font-medium text-foreground">{topic}</Td>
                <Td>
                  <C>{file}</C>
                </Td>
                <Td className="text-muted">
                  <Md text={when} />
                </Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <H3 id="gpui-extended">Extended references</H3>
      <Ul>
        {GPUI_EXTENDED_REFERENCES.map(([group, files]) => (
          <Li key={group}>
            <strong>{group}:</strong> <Md text={files.join(" · ")} />
          </Li>
        ))}
      </Ul>

      <H2 id="rust-best-practices">The rust-best-practices skill</H2>
      <P>
        Vendored from <C>apollographql/skills</C> (<C>skills/rust-best-practices/SKILL.md</C>{" "}
        upstream). Its front matter declares version 1.1.1, an MIT license, and compatibility with
        Rust 1.70+ and Cargo. Where the gpui skill answers &ldquo;how does this framework
        work&rdquo;, this one answers &ldquo;is this good Rust&rdquo; — it is a distillation of
        Apollo GraphQL&apos;s Rust Best Practices Handbook across nine chapter files.
      </P>
      <div className="mt-6 overflow-x-auto">
        <table className="w-full border-collapse text-sm leading-6">
          <thead>
            <tr>
              <Th>Chapter</Th>
              <Th>File</Th>
              <Th>Focus</Th>
            </tr>
          </thead>
          <tbody>
            {RUST_CHAPTERS.map(([n, file, focus]) => (
              <tr key={file}>
                <Td className="text-muted">{n}</Td>
                <Td>
                  <C>{file}</C>
                </Td>
                <Td className="text-muted">
                  <Md text={focus} />
                </Td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <P>
        The <C>SKILL.md</C> itself also carries a quick-reference section — borrowing rules,
        <C>thiserror</C> vs <C>anyhow</C>, clippy invocations, test naming — so common judgement
        calls do not even require opening a chapter.
      </P>

      <H2 id="skills-lock-json">Pinning with skills-lock.json</H2>
      <P>
        <C>skills-lock.json</C> at the repository root records, for each skill, where it came from
        and a digest of the pinned content:
      </P>
      <div className="mt-6">
        <CodeBlock code={LOCK_JSON} lang="json" filename="skills-lock.json" />
      </div>
      <Ul>
        <Li>
          <C>source</C> and <C>skillPath</C> — the upstream GitHub repository and the path of the
          skill inside it, so provenance is checkable.
        </Li>
        <Li>
          <C>computedHash</C> — a 64-character content digest. If the upstream skill changes, the
          digest no longer matches, so a refresh is a deliberate, reviewable diff rather than silent
          drift.
        </Li>
      </Ul>
      <P>
        The lock file has no companion installer script in this repository: the mirrors under{" "}
        <C>.claude/skills/</C> and <C>.factory/skills/</C> are symlinks committed alongside it, and
        the three locations move together in review.
      </P>

      <H2 id="installing">Installing the skills elsewhere</H2>
      <P>
        A skill is a directory with a <C>SKILL.md</C> and its <C>references/</C>. To use these in
        another project, copy or symlink the directory into the location your agent tool scans, and
        record its source plus digest so future updates stay deliberate:
      </P>
      <div className="mt-6">
        <CodeBlock code={INSTALL_SH} lang="bash" filename="install" />
      </div>
      <P>
        Keep the canonical copy in <C>.agents/skills/</C> and link each per-tool directory to it, as
        this repository does.
      </P>

      <H2 id="why-these-skills">Why these two</H2>
      <P>
        The two skills cover the repository&apos;s main sources of drift: GPUI code that uses an
        unavailable API, and Rust code that fights the borrow checker or repository conventions. Use{" "}
        <C>gpui</C> for framework behavior and <C>rust-best-practices</C> for code quality, while{" "}
        <Link href="/docs/ai/agents-md">AGENTS.md</Link> and{" "}
        <Link href="/docs/ai/llms-txt">llms.txt</Link> carry the repository-specific rules and the
        component contract. These skills support repository work and are not included in the
        published crates.
      </P>
    </>
  );
}
