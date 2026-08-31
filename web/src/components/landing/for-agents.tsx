import { Link } from "@heroui/react";
import { ArrowUpRight } from "lucide-react";
import { SectionHeading } from "@/components/landing/shared";

/** The repository's agent-facing guides and their site routes. */

const AGENT_LINKS = [
  {
    path: "llms.txt",
    href: "/docs/ai/llms-txt",
    what: "The component API reference, with builders, parts, states and styling notes.",
  },
  {
    path: "AGENTS.md",
    href: "/docs/ai/agents-md",
    what: "Repository layout, verification commands and contribution guidance.",
  },
  {
    path: ".agents/skills/",
    href: "/docs/ai/agent-skills",
    what: "Loadable skills for GPUI and idiomatic Rust.",
  },
] as const;

export function ForAgents() {
  return (
    <section className="landing-for-agents">
      <div className="mx-auto grid w-full max-w-[1440px] gap-10 px-4 py-16 sm:px-6 md:py-24 lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] lg:gap-20">
        <SectionHeading
          eyebrow="For coding agents"
          sub="The repository includes a machine-readable API reference, repository guidance and loadable skills. Give your agent the same context you use."
          title="A repository your agent can navigate"
        />

        <ul>
          {AGENT_LINKS.map((link) => (
            <li className="border-t border-separator last:border-b" key={link.path}>
              <Link
                className="group flex items-center justify-between gap-6 py-5 transition-colors"
                href={link.href}
              >
                <span className="min-w-0">
                  <span className="block font-mono text-sm font-medium text-foreground transition-colors group-hover:text-accent">
                    {link.path}
                  </span>
                  <span className="mt-1 block text-sm leading-relaxed text-muted">{link.what}</span>
                </span>
                <ArrowUpRight
                  aria-hidden="true"
                  className="size-5 shrink-0 text-muted transition-colors group-hover:text-accent"
                />
              </Link>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
