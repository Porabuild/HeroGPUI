/** Shared site constants and hand-maintained route lists for the chrome. */

export const SITE = {
  name: "HeroGPUI",
  // The chip states the version and nothing else. Release status — "Released",
  // "Now available", or the reverse — is not something this site comments on.
  version: "v0.1.0",
  /** The checkout's actual remote (github.com/Porabuild/HeroGPUI). */
  github: "https://github.com/Porabuild/HeroGPUI",
  cratesio: "https://crates.io/crates/herogpui",
  upstream: "https://heroui.com",
  llmsTxt: "/llms.txt",
} as const;

export interface NavLink {
  href: string;
  label: string;
  /** Match only this exact path instead of the path prefix. */
  exact?: boolean;
}

/** The seven getting-started guides exposed in the site navigation. */
export const GETTING_STARTED_LINKS: NavLink[] = [
  { href: "/docs/getting-started", label: "Overview", exact: true },
  { href: "/docs/getting-started/installation", label: "Install" },
  { href: "/docs/getting-started/theming", label: "Theming" },
  { href: "/docs/getting-started/dark-mode", label: "Dark mode" },
  { href: "/docs/getting-started/customization", label: "Customize" },
  { href: "/docs/getting-started/styling", label: "Styling" },
  { href: "/docs/getting-started/design-principles", label: "Design principles" },
];

/** The three guides for coding agents. */
export const AI_LINKS: NavLink[] = [
  { href: "/docs/ai/llms-txt", label: "llms.txt" },
  { href: "/docs/ai/agent-skills", label: "Agent skills" },
  { href: "/docs/ai/agents-md", label: "Repository guide" },
];

/** Top-level navbar sections. */
export const NAV_LINKS: NavLink[] = [
  { href: "/docs/getting-started", label: "Docs" },
  { href: "/docs/components", label: "Components", exact: true },
  { href: "/docs/releases", label: "Releases" },
];

export function isNavLinkActive(pathname: string, link: NavLink): boolean {
  if (link.exact) return pathname === link.href;
  return pathname === link.href || pathname.startsWith(`${link.href}/`);
}
