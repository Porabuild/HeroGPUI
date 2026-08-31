# HeroGPUI website

The documentation and marketing site for
[HeroGPUI](https://github.com/Porabuild/HeroGPUI) — a native Rust/GPUI port
of [HeroUI v3.2.4](https://heroui.com). Deployed to Vercel under the
Porabuild team and mounted at <https://porabuild.com/herogpui> via a
Next.js multi-zone rewrite; see [DEPLOYMENT.md](DEPLOYMENT.md) for the
operator runbook and [ARCHITECTURE.md](ARCHITECTURE.md) for the
architecture contract and directory ownership.

## How components are shown

GPUI is a native GPU renderer with no WebAssembly target, so the browser
cannot run HeroGPUI itself. Every component page shows three things and
labels each one honestly:

1. **Live preview** — the upstream **HeroUI React 3.2.4** demo, running for
   real in the browser. Legitimate because HeroGPUI ports exactly that
   version: the React component *is* the parity reference.
2. **Rust** — the equivalent `herogpui` builder code, syntax-highlighted.
3. **Native render** — a real GPUI screenshot captured from the desktop
   gallery.

The catalog indexes **66 documentation pages covering all 71 components
HeroUI v3 documents** — a few pages cover a component and its group or slot
siblings together (`toggle-button` covers ToggleButton and ToggleButtonGroup,
`disclosure` covers Disclosure and DisclosureGroup, `label-messages` covers
Label, Description, ErrorMessage and FieldError).

## Stack

- Next.js **16.3.3** (App Router, Turbopack), React **19.2.8**
- `@heroui/react` **3.2.4** (the live previews), Tailwind CSS **4.3.3**,
  TypeScript 5.9, Shiki for Rust highlighting
- Lint/format: **oxlint** + **oxfmt** (`pnpm run check`). No ESLint.
- Package manager: **pnpm**. Never npm. Node `>=22.13.0`.

## Run it

```sh
pnpm install
pnpm run dev        # http://localhost:3000, basePath unset
```

Other commands:

```sh
pnpm run build      # production build
pnpm run start      # serve the production build
pnpm run check      # typecheck + oxlint + oxfmt
pnpm run typecheck  # tsc --noEmit (runs `next typegen` first)
```

In local development the site serves at `/`. In production it serves under
`/herogpui` — see `next.config.ts` (`basePath` from
`NEXT_PUBLIC_BASE_PATH`) and [`.env.example`](.env.example). The landing
and catalog code routes public assets through a `publicUrl()` helper so
screenshots resolve under the prefix.

## The generated data pipeline

The pages are driven by JSON extracted from the Rust workspace — the same
source files the desktop gallery renders — plus the upstream HeroUI v3.2.4
demos. **The generated outputs are committed** (`src/data/*.json`,
`public/shots/`, `src/demos/`), so a plain `pnpm run build` works from a
fresh clone; Vercel never runs the pipeline. Re-run it by hand when the
Rust sources they read change:

| Command | Reads | Produces |
|---|---|---|
| `node scripts/extract-reference.mjs` (also `pnpm run extract`) | `gallery/src/pages/reference_metadata.rs` | `src/data/reference.json` — per-component API/parts/states/styling tables with implementation status |
| `node scripts/extract-catalog.mjs` | `gallery/src/pages/mod.rs` (`Page` enum), `.shots/`, `src/demos/`, `reference.json` | `src/data/catalog.json` — the 66 component pages grouped into the 15 v3 categories |
| `node scripts/extract-rust-examples.mjs` | `gallery/src/pages/components.rs` | `src/data/rust-examples.json` — the per-component Rust snippets the pages display |
| `node scripts/copy-shots.mjs` | `.shots/*.png` | `public/shots/` — the GPUI screenshots |
| `node scripts/extract-changelog.mjs` | the repository's git history | `src/data/changelog.json` — the `/docs/releases` development log |
| `node scripts/build-data.mjs` | — | runs the four offline extractors in dependency order with one summary |
| `node scripts/vendor-demos.mjs [--fresh]` | the HeroUI v3.2.4 git tree (network) | `src/demos/**` + `src/demos/registry.ts` + `NOTICE` — the vendored live-preview sources, with attribution headers |

The vendoring step is the only network step and is intentionally outside
`build-data`; the changelog step reads git history and is run manually. The
`/llms.txt` route handler is not generated — it serves the repository
root's `llms.txt`, read once at build time.
