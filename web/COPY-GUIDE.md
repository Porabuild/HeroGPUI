# HeroGPUI website — copy guide

The binding editorial contract for every word on this site. It sits alongside
`ARCHITECTURE.md`, which still governs structure and code.

## What this site is now

A product site for **HeroGPUI**, a UI library for Rust desktop applications.
It ships together with the library. Write it the way you would write the site
for any released library: confident, specific, present tense.

It is **not** a progress report, a porting diary, or a parity scoreboard.

## Three rules that override everything

**1. Say "HeroUI", not "HeroUI v3".**

HeroGPUI brings HeroUI's design system to Rust. That is the whole story. Drop
`v3`, `v3.2.4`, "the v3 token system", "v3 components", "the v3 contract".
Version numbers appear only where they are load-bearing:

- inside a code block that pins a dependency;
- in a link to a specific upstream source file;
- in `ARCHITECTURE.md` and other developer-facing files, which are not site copy.

"HeroUI v3.2.4" in body prose is wrong. "HeroUI" is right.

**2. Never comment on release status.**

Delete every trace of "not published yet", "prepared but not yet published",
"the names were unclaimed", "before v0.1.0 use a path dependency", "in
development", "no git tags". Do not comment on whether the library is
released. The install instructions are simply the install instructions: quote
the dependency block from the installation page — GPUI from the pinned Zed git
revision plus `herogpui` as a git/path dependency — and never invent a
registry version such as `herogpui = "0.1"`.

**3. Drop the parity framing.**

No "parity", "faithful port", "measured, not asserted", "0 real gaps",
"deliberately not ported", "documented props considered". A reader wants to
know what the library does, not how it was verified against something else.

Where a fact is genuinely useful to a user, keep the fact and drop the
scoreboard. "Every component HeroUI documents, implemented in Rust" is useful.
"763 documented props considered, 714 implemented, 49 deliberately not ported"
is an internal audit result and belongs in the repository, not on the site.

The one place a comparison still earns its place is the component preview,
where a reader is looking at a React demo next to Rust code and deserves to
know what each panel is. Say it plainly, once, and move on.

## Voice

Write like a good library's documentation: plain, precise, unhurried. Short
declarative sentences. Concrete nouns. Second person for instructions.

## Kill these — they are why the current copy reads as machine-written

- Adjective inflation: *seamless, powerful, robust, blazing, beautiful,
  modern, elegant, comprehensive, rich, delightful, first-class, world-class*.
- Verb inflation: *leverage, unlock, empower, supercharge, dive into,
  harness, craft, elevate*.
- The negation-reversal cadence: "It's not just a port — it's a rethinking."
  "This isn't X. It's Y." Never use it.
- Rule-of-three padding: "fast, modern and cross-platform" where one accurate
  word would do.
- Em-dash asides stacked two or three to a sentence.
- Sentences that would be true of any library. If you can swap in a different
  product name and the sentence still reads fine, it says nothing — cut it or
  make it specific.
- Restating the heading in the first line of the paragraph beneath it.
- "Whether you're building X or Y…" openers.
- Closing exhortations: "Start building today", "The possibilities are
  endless", "Happy coding".

## Prefer

- The specific number, name or token over the abstraction: "OKLCH tokens that
  derive hover, soft and foreground variants from one base colour" beats
  "a powerful theming system".
- The user's task over the library's architecture: "Toggle light and dark at
  runtime with one call" beats "Runtime theme switching is supported".
- Saying it once. If a page states something well in the intro, the section
  below it should add detail, not repeat the claim.

## Facts you may state

Sourced from the repository, and true:

- HeroGPUI is a UI library for Rust desktop applications, built on GPUI, the
  GPU-accelerated framework behind the Zed editor.
- It runs on Windows, macOS and Linux from one codebase.
- It implements every component HeroUI documents — 71 of them, indexed here as
  66 pages because a few pages cover a component together with its group or
  slot siblings. State the 66/71 relationship once, where a count appears, and
  do not belabour it.
- The colour system is OKLCH semantic tokens: base, surfaces, roles
  (`default`, `accent`, `success`, `warning`, `danger`) and field tokens, each
  deriving its hover, soft and foreground variants.
- Components are typed Rust builders. State is explicit and either controlled
  or uncontrolled.
- Animations respect a reduced-motion setting.
- A desktop gallery ships with the library and documents every component.
- `llms.txt`, `AGENTS.md` and agent skills ship in the repository.
- HeroGPUI is Apache-2.0. It derives from HeroUI, Copyright 2025 NextUI Inc,
  also Apache-2.0. Keep this attribution accurate and intact.

Do not invent benchmarks, download counts, stars, users, testimonials or
company logos.

## The component preview labels

Each component page embeds HeroGPUI itself, compiled to WebAssembly and
running live in one frame, with the Rust code below it. Say so once per page,
calmly, in the spirit of: "The frame runs HeroGPUI compiled to WebAssembly.
The code below it is the Rust that produced the example."

Never write anything implying the frame is a screenshot or a recreation in
another framework.

## Scope

Change prose, headings, metadata titles and descriptions, and the strings
inside components. Do not restructure pages, rename routes, change data
shapes, or alter code that is not a user-visible string. `pnpm run build`,
`typecheck`, `lint` and `format:check` must still pass.
