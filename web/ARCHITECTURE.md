# HeroGPUI website — architecture contract

Every agent working in `web/` follows this document. It is the shared contract:
directory ownership, data shapes, and the verified toolchain recipe. Update it
when the website structure or contracts change.

## What this site is

Public documentation and marketing site for **HeroGPUI**, a native Rust/GPUI
port of **HeroUI v3.2.4**. Modelled on <https://heroui.com> (structure, docs
experience) and <https://longbridge.github.io/gpui-component/> (a Rust UI
library presenting itself on the web).

Deployed to Vercel under the Porabuild team and mounted at
`https://porabuild.com/herogpui` via a Next.js multi-zone rewrite, so every
route lives under `basePath: "/herogpui"`.

## How components are shown (the central design decision)

GPUI is a native GPU renderer with no WebAssembly target in 0.2.2, so the
browser cannot run HeroGPUI itself. Each component page follows this order:

1. **Header** — the component title, description, and Rust import line.
2. **Native preview** — one real capture from the desktop gallery near the top
   of the page. The caption identifies it as a native gallery capture.
3. **Examples** — one bordered card per Rust gallery snippet. Each card has a
   heading and a Shiki-highlighted, line-numbered Rust block. Long blocks start
   at a readable height with a fade and expand from a control at the foot;
   short blocks remain open. Cards accept an optional `preview` slot for a
   future per-example capture and render no preview pane when it is absent.
4. **Reference** — API, anatomy, states, and styling tables when the extracted
   reference data exists.

The browser never presents a live GPUI render. The native capture is the only
component preview until per-example captures or a WebAssembly gallery exist.

## Verified toolchain recipe (do not re-derive)

- Next.js **16.3.3** (App Router, Turbopack), React **19.2.8**,
  `@heroui/react` **3.2.4**, Tailwind CSS **4.3.3**, TypeScript 5.9.
- Package manager is **pnpm** (`pnpm install`, `pnpm run build`). Never npm.
- `src/app/globals.css` must keep these lines. `@heroui/styles` already
  imports `tailwindcss` and `tw-animate-css`; do not import `tailwindcss`
  again. The `@source` lines are required because Tailwind 4 does not scan
  `node_modules` by default, and pnpm nests real packages under `.pnpm/`:

  ```css
  @import "@heroui/styles";
  @source "../../node_modules/.pnpm/@heroui+react@*/node_modules/@heroui/react/dist";
  @source "../../node_modules/.pnpm/@heroui+styles@*/node_modules/@heroui/styles/dist";
  ```

- Lint/format is **oxlint** and **oxfmt** (`pnpm run check`), matching the
  Porabuild website. There is no ESLint.
- Next.js 16 ships its own docs at `node_modules/next/dist/docs/`. Read them
  before using an API you are unsure about; your training data predates this
  release.

## HeroUI v3 is compound-only — read the types before writing a demo

v3 bears no resemblance to v2. Every component is a compound namespace, and
guessing the shape will produce code that type-checks against nothing. For
example a switch is **not** `<Switch>Label</Switch>`:

```tsx
<Switch>
  <Switch.Control><Switch.Thumb /></Switch.Control>
  <Label>Wi-Fi</Label>
</Switch>
```

Before writing any usage of a component, read its real typings:

```
node_modules/.pnpm/@heroui+react@3.2.4_*/node_modules/@heroui/react/dist/components/<name>/*.d.ts
```

v3 also removed v2's `color`/`variant` matrix. Colors are
`default | accent | success | warning | danger` — there is no `primary` or
`secondary` color. Button variants are
`primary | secondary | tertiary | outline | ghost | danger | dangerSoft`.

## Directory ownership

Each agent writes **only** inside its own paths. Never edit another agent's
files, `package.json`, `pnpm-lock.yaml`, or this file.

| Owner | Paths |
|---|---|
| Data pipeline | `scripts/**`, `src/data/**`, `public/shots/**` |
| Shell & design system | `src/app/layout.tsx`, `src/app/globals.css`, `src/components/site/**`, `src/components/ui/**`, `src/lib/**`, `next.config.ts` |
| Landing | `src/app/page.tsx`, `src/components/landing/**` |
| Getting started | `src/app/docs/getting-started/**` |
| AI docs | `src/app/docs/ai/**`, `src/app/llms.txt/**` |
| Components index | `src/app/docs/components/page.tsx` |
| Component detail | `src/app/docs/components/[slug]/**`, `src/components/preview/**` |
| Releases | `src/app/docs/releases/**` |

## Routes

```
/                                  landing / presentation
/docs                              → redirect to /docs/getting-started
/docs/getting-started              introduction
/docs/getting-started/installation quick start
/docs/getting-started/theming
/docs/getting-started/dark-mode
/docs/getting-started/customization
/docs/getting-started/styling
/docs/getting-started/design-principles
/docs/ai/llms-txt                  what llms.txt is, and ours
/docs/ai/agent-skills              the skills this repo ships
/docs/ai/agents-md                 AGENTS.md / CLAUDE.md guidance
/docs/components                   all components, grouped by category
/docs/components/[slug]            one component
/docs/releases                     changelog
/llms.txt                          route handler serving the repo's llms.txt
```

## Data contract

The data pipeline generates these files. Every other agent **reads** them and
must not hand-maintain the same information.

### `src/data/catalog.json`

```jsonc
{
  "version": "0.1.0",
  "categories": [
    { "name": "Buttons", "slug": "buttons",
      "components": ["button", "button-group", "close-button", "toggle-button"] }
  ],
  "components": {
    "button": {
      "slug": "button",
      "title": "Button",
      "description": "A pressable button with variants and states.",
      "category": "Buttons",
      "importLine": "use herogpui::prelude::{Button, Size, Variant};",
      "shot": "/shots/button-v3.png",
      "shotDark": "/shots/button-dark-v3.png",
      "demos": [],
      "hasReference": true
    }
  }
}
```

`shot`, `shotDark` and `hasReference` may be `null` / `false`. `demos` is kept
as an empty compatibility field. Consumers must handle all three.

### `src/data/reference.json`

Extracted from `../gallery/src/pages/reference_metadata.rs`. Keyed by component
slug:

```jsonc
{
  "button": {
    "page": "Button",
    "importLine": "...",
    "version": "3.2.4",
    "docsSource": "https://github.com/heroui-inc/heroui/blob/v3.2.4/...",
    "apiSource": "...",
    "styleSource": "...",
    "requiredParts": ["Button", "Button.Label"],
    "api":   [{ "owner": "Button", "prop": "variant", "type": "...", "default": "...",
                "description": "...", "rustOwner": "Button", "rust": "variant(Variant)",
                "status": "implemented" }],
    "parts": [{ "name": "...", "slot": "...", "description": "...",
                "rustOwner": "...", "status": "implemented" }],
    "states":  [{ "state": "...", "selector": "...", "description": "...",
                  "rust": "...", "status": "implemented" }],
    "styling": [{ "token": "...", "description": "...", "rust": "...",
                  "status": "implemented" }]
  }
}
```

`status` is one of `"implemented" | "partial" | "unavailable"`. Render
`unavailable` as "Not ported" — it is a deliberate omission with a documented
reason, not a bug, and the site must not present it as a failure.

### `src/data/rust-examples.json`

Rust snippets per component, extracted from `../gallery/src/pages/components.rs`:

```jsonc
{ "button": [ { "heading": "Variants", "imports": "...", "code": "row(Variant::ALL.iter()...)" } ] }
```

`imports` is optional; when present it is shown above the example expression.

## Attribution

**HeroUI v3.2.4 is Apache-2.0**, Copyright 2025 NextUI Inc. Verify this against
`https://raw.githubusercontent.com/heroui-inc/heroui/v3.2.4/LICENSE` rather
than the `@heroui/react` npm manifest, whose `license` field says "MIT" while
the tarball ships the Apache-2.0 text. The repository's own
`E:\work\HeroGPUI\NOTICE` already states Apache-2.0; match its wording.

HeroGPUI itself is Apache-2.0 and derives from HeroUI, Copyright 2025 NextUI
Inc., also Apache-2.0. The site footer credits HeroUI and links to
<https://heroui.com>; `web/NOTICE` carries the derivation attribution.

## Quality bar

- Server Components by default. `"use client"` only where interactivity
  demands it — copy controls, theme toggle, and nav state.
- No `any`. `pnpm run check` must pass (typecheck + oxlint + oxfmt).
- Dark mode is a real toggle, class-based, persisted, with no flash on load.
- Accessible: real landmarks, focus-visible rings, keyboard-operable nav.
- Every claim about HeroGPUI must be traceable to the repo (`llms.txt`,
  `README.md`, the audits). Do not invent benchmarks, stars, or adoption
  numbers.
