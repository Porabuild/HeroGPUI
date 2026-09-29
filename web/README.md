# HeroGPUI website

The documentation site for
[HeroGPUI](https://github.com/Porabuild/HeroGPUI) — a native Rust/GPUI port
of [HeroUI v3.2.6](https://heroui.com). Deployed to Vercel under the
Porabuild team and mounted at <https://porabuild.com/herogpui> via a
Next.js multi-zone rewrite; see [DEPLOYMENT.md](DEPLOYMENT.md) for the
operator runbook and [ARCHITECTURE.md](ARCHITECTURE.md) for the
architecture contract and directory ownership.

## How components are shown

Each component page embeds the real HeroGPUI gallery — the Rust application
compiled to WebAssembly — in one lazily booted iframe
(`src/components/preview/gallery-frame.tsx`, served from
`public/gallery/`). The frame runs the component live; an example switcher
above it selects among the gallery's examples for that component, and the
matching Rust code sits below. The caption on the frame says it is HeroGPUI
compiled to WebAssembly. It is never a screenshot and never a recreation in
another framework.

Component pages otherwise show: header with the Rust import line, Usage (the
live frame plus code), Anatomy, Customization (the styling table), API
reference (props, parts and slots, then states), and Related components.

The catalog indexes **66 documentation pages covering all 71 components
HeroUI v3 documents** — a few pages cover a component and its group or slot
siblings together (`toggle-button` covers ToggleButton and ToggleButtonGroup,
`disclosure` covers Disclosure and DisclosureGroup, `label-messages` covers
Label, Description, ErrorMessage and FieldError).

## Stack

- Next.js **16.3.3** (App Router, Turbopack), React **19.2.8**
- `@heroui/react` **3.2.6** for site UI chrome, `@heroui/styles` for tokens;
  Tailwind CSS **4.3.3**, TypeScript 5, Shiki for Rust highlighting
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
`NEXT_PUBLIC_BASE_PATH`) and [`.env.example`](.env.example). Routes and
components prefix public asset paths through the `publicUrl()` helper in
`src/lib/public-url.ts` so catalog tiles and the gallery iframe resolve under
the prefix.

## The generated data pipeline

The pages are driven by JSON extracted from the Rust workspace — the same
source files the desktop gallery renders. **The generated outputs are
committed** (`src/data/*.json`, `public/shots/`, `public/gallery/index.html`),
so a plain `pnpm run build` works from a fresh clone; Vercel never runs the
pipeline. The one exception is the WebAssembly artifact, which CI builds:
`pnpm run build` first downloads the one for the checkout
(`scripts/gallery-artifact.mjs`, see `DEPLOYMENT.md` section 6). Re-run the
pipeline by hand when the Rust sources it reads change:

| Command | Reads | Produces |
|---|---|---|
| `node scripts/extract-reference.mjs` | `gallery/src/pages/reference_metadata/` | `src/data/reference.json` — per-component API/parts/states/styling tables with implementation status |
| `node scripts/extract-catalog.mjs` | `gallery/src/pages/mod.rs` (`Page` enum), `.shots/`, `reference.json`, `../Cargo.toml`, `../Cargo.lock` | `src/data/catalog.json` — the component pages grouped into categories, plus the workspace and GPUI versions the site shows |
| `node scripts/extract-rust-examples.mjs` | `gallery/src/pages/components/` | `src/data/rust-examples.json` — the per-component Rust snippets the pages display, public-API only; `gallery/build.rs` compiles each one under `cargo test -p herogpui-gallery` |
| `node scripts/copy-shots.mjs` | `.shots/*-tile*-v3.png` | `public/shots/` — the cropped catalog tiles the components index shows (the full-page captures stay in `.shots/` as parity goldens) |
| `node scripts/extract-releases.mjs` | `../CHANGELOG.md` | `src/data/releases.json` — the `/docs/releases` notes, one per dated version |
| `node scripts/build-data.mjs` | — | runs the four offline extractors in dependency order with one summary |
| `node scripts/extract-wasm-sections.mjs` (`pnpm run wasm:manifest`) | `gallery/src/pages/components/` — the same native source the desktop gallery builds from — plus every wasm build input | `src/data/wasm-sections.json` + `src/data/wasm-parity.json` — the examples compiled into the wasm artifact and the artifact key (an inputs hash) CI publishes it under; `--check` fails when Rust sources change without regenerating |
| `node scripts/gallery-artifact.mjs fetch` (first step of `pnpm run build`; `pnpm run gallery:fetch`) | the checkout's wasm inputs, the checkout's `gallery-<key16>` GitHub prerelease | `public/gallery/herogpui_web*` + `build-info.json` — CI's artifact for this checkout, verified by SHA-256 (not committed) |
| `node scripts/sync-porabuild-brand.mjs` (`pnpm run brand:sync`) | the sibling `@porabuild/brand` package | `src/styles/porabuild/` — the vendored brand layer (never hand-edit; re-sync instead) |

`pnpm run extract` runs the reference, catalog, snippet and releases steps;
`pnpm run extract:check` (CI) checks all of them plus the wasm manifest,
without network. The `/llms.txt`
route handler is not generated — it serves the repository root's `llms.txt`,
read once at build time.
