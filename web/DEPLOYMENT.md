# Deploying the HeroGPUI website

The operator runbook for putting this site at
**<https://porabuild.com/herogpui>**. The site is its own Vercel project
(the "child" zone) under the **Porabuild** team
(`team_bnQfB2EkDKTik1A4ABzsrK1Y`), and the Porabuild marketing site
(`E:\work\porabuild`, Vercel project `prj_JoY3Y9VQiURE4gVw2nZOoTfdcAnQ`)
proxies the `/herogpui` path prefix to it — the Next.js
[multi-zone](https://nextjs.org/docs/app/guides/multi-zones) pattern.

```
visitor ──> porabuild.com/herogpui/docs/components
              │  rewrites() in E:\work\porabuild\next.config.ts
              ▼
            <herogpui-zone-domain>/herogpui/docs/components
              │  this project (Root Directory web/, basePath /herogpui)
              ▼
            the page, all asset URLs under /herogpui/…
```

## 1. Create the Vercel project

Import **`Porabuild/HeroGPUI`** into Vercel under the Porabuild team, then
set, on the **Root Directory** step or in Project → Settings → General:

| Setting | Value | Why |
|---|---|---|
| Root Directory | `web` | The repository root is a Rust/Cargo workspace. Built from the root, Vercel would find no `package.json` and no Next.js app. Everything Vercel needs — `package.json`, `pnpm-lock.yaml`, `next.config.ts`, `public/` — lives in `web/`. |
| Framework Preset | Next.js (auto-detected) | Detected from `web/package.json`. |
| Install Command | `pnpm install --frozen-lockfile` | Set explicitly via Project Settings (API `installCommand`). Required: Vercel's auto-detected install appends `--unsafe-perm`, which pnpm 10+ rejects, and its cached pnpm 12.1.0 Linux artifact is corrupt (a broken `bin/pnpm` launcher — "syntax error near unexpected token"). `web/package.json` pins `packageManager: pnpm@10.34.5`; don't bump to 12.x until its artifact is fixed. |
| Build Command | `pnpm run build` | The default `next build` is equivalent; pinning the pnpm form keeps it unambiguous. |
| Node.js Version | 24.x | `web/package.json` requires `>=22.13.0`; 24.x is what the project runs on. |

With Root Directory set to `web`, Vercel still clones the **whole**
repository and runs the commands inside `web/`. That matters: the build
reads the sibling `../llms.txt` (the `/llms.txt` route handler prerenders
it at build time) and the committed generated data was extracted from
`../gallery` — but the pipeline itself never runs on Vercel (see
[README.md](README.md), "The generated data pipeline").

## 2. Environment variables

Project → Settings → Environment Variables:

| Name | Value | Environments |
|---|---|---|
| `NEXT_PUBLIC_BASE_PATH` | `/herogpui` | Production, Preview |
| `NEXT_PUBLIC_SITE_URL` | `https://porabuild.com/herogpui` | Production, Preview |

`web/.env.example` documents both in detail. Two properties to remember:

- **They are build-time values.** `NEXT_PUBLIC_*` variables are inlined
  into the client bundle when `next build` runs. Adding or changing one
  takes effect on the *next* deployment, never on the running one.
- **Previews should use the production value.** A preview deployment serves
  from a `*.vercel.app` URL with the same `/herogpui` prefix. Note the bare
  root (`/`) is a 404 on a basePath build — Next does not redirect it — so
  open preview URLs with the `/herogpui` suffix. That mirrors production
  exactly, which is what a preview is for.

## 3. Mount it on porabuild.com

Add this `rewrites()` block to `E:\work\porabuild\next.config.ts` (the
parent zone) and deploy the Porabuild project. Substitute the HeroGPUI
project's production domain — the `…vercel.app` domain Vercel assigns the
project, or a custom domain you assign it — for `HEROGPUI_ZONE_DOMAIN`
(keep it in the parent's own environment variables rather than inlining it
if you prefer it out of git):

```ts
  async rewrites() {
    return [
      // HeroGPUI docs zone (separate Vercel project, built from web/ with
      // basePath "/herogpui"). Identity mapping: the zone serves every
      // route — including its own static assets, because basePath prefixes
      // them too — under /herogpui, so the path is forwarded unchanged.
      {
        source: "/herogpui",
        destination: `${process.env.HEROGPUI_ZONE_DOMAIN}/herogpui`,
      },
      {
        source: "/herogpui/:path*",
        destination: `${process.env.HEROGPUI_ZONE_DOMAIN}/herogpui/:path*`,
      },
    ];
  },
```

Notes:

- Both rules are needed: `:path*` alone does not match the bare
  `/herogpui` prefix.
- No separate asset rule is required. Next.js 15+ resolves zone assets
  through the zone's own prefix, and this zone's asset URLs all start with
  `/herogpui/_next/…` because `basePath` is set — the `:path*` rule covers
  them.
- Rewrites (not redirects): the visitor stays on `porabuild.com`; the
  parent proxy fetches from the child on the server side.
- This is the only change the mount needs in the Porabuild repository. That
  repository belongs to a different project — do not change anything else
  in it, and coordinate the deploy, because the mount activates the moment
  its deployment goes out.

## 4. Why `basePath` must match the mount path

`NEXT_PUBLIC_BASE_PATH=/herogpui` feeds Next's
[`basePath`](https://nextjs.org/docs/app/api-reference/config/next-config-js/basePath).
Every URL the app emits — `<script>`/`<link>` asset tags, `<img>` sources,
`<Link>` hrefs, router pushes — is prefixed with `/herogpui`. That is what
makes the two-rule proxy above sufficient:

- The browser never requests a URL outside `porabuild.com/herogpui/…`, so
  the parent can forward every request identity-mapped to the child, and
  the child recognises the prefix as its own and strips it before routing.
- If the child were built **without** the basePath, its HTML would point at
  `/_next/static/…` and `/docs/…`; the browser would resolve those against
  `porabuild.com`'s root, where the parent zone — a different application —
  answers. Styling 404s, navigation leaves the zone.
- If the basePath **disagreed** with the mount path (say `/docs-site`
  mounted at `/herogpui`), every emitted URL would 404 through the proxy.

The two values have two independent sources — the Vercel environment
variable on this project, and the `source:` patterns in the parent's
`next.config.ts` — so changing either one means changing both.

## 5. Post-deploy checks

Run through these on the production URL:

- `porabuild.com/herogpui` renders the landing page (the bare prefix, no
  trailing slash).
- `porabuild.com/herogpui/llms.txt` serves `text/plain` — the repo's
  llms.txt, prerendered at build time.
- A component page (`/herogpui/docs/components/button`) lists every example
  (heading, description, code) and lazily mounts one live GPUI/WASM canvas.
  "Show live" on an example, or a `#rust-<example>` link, switches that same
  canvas without creating another iframe.
- The landing page does not fetch the wasm until "Run the live demo" (or a
  specimen tab) is pressed; toggling the site theme does not restart it.
- `curl -sI "https://porabuild.com/herogpui/gallery/herogpui_web_bg.wasm?v=<12 hex>"`
  (the `v` of any embed URL) answers `cache-control: public, max-age=31536000, immutable`.
- Navigate Docs → Components → a component page: internal navigation stays
  inside the `/herogpui` prefix (it is one zone, so these are soft
  navigations).
- View source: canonical/Open Graph URLs begin with
  `https://porabuild.com/herogpui` (that is `NEXT_PUBLIC_SITE_URL` doing
  its job via `metadataBase`).

## 6. The live WebAssembly gallery

Component pages embed the real HeroGPUI gallery — the Rust application
compiled to wasm — in an iframe (`GalleryFrame`,
`src/components/preview/gallery-frame.tsx`; URLs and messages in
`src/lib/gallery-embed.ts`). Three files make that work; they live in
`public/gallery/` and are served by the same deployment:

| file | what it is |
|---|---|
| `index.html` | the hosting page (loading spinner, error UI, boot script, message bridge); a copy of `crates/herogpui-web/index.html` |
| `herogpui_web.js` | `wasm-bindgen` glue |
| `herogpui_web_bg.wasm` | the application, ~18 MiB raw / ~5.4 MB brotli |

Keep this as one browser-cached module. A component page defers the download
until its preview nears the viewport; the landing page defers it until the
reader presses "Run the live demo". Either way one instance stays alive:
examples, components (`story`) and the theme switch over the message bridge
(`herogpui:preview-section`, `herogpui:set-theme`; the instance answers
`herogpui:ready` once booted), never by reloading the frame.

**Caching.** Every embed URL carries `?v=<first 12 hex of the artifact
SHA-256>` (from `src/data/wasm-parity.json`), and `index.html` forwards it
onto the glue and the `.wasm`. A new build is therefore a new URL, so
`next.config.ts` serves versioned requests of those two files with
`Cache-Control: public, max-age=31536000, immutable`; an unversioned request
(a direct gallery link) keeps the default revalidating policy.

`next.config.ts` maps `/gallery` onto `/gallery/index.html` (public/ has no
directory-index resolution), so `/gallery` is the default.
`NEXT_PUBLIC_GALLERY_URL` is an optional build-time override for hosting the
artifact at another path or origin; changing it takes effect on the next
deployment.

The three files are **tracked in git**: remote builds run `next build` alone
— no Rust toolchain — so the artifact must ship in the tree. See "Why the
artifact is committed" below.

Component previews use
`/gallery/index.html?preview=component&story=button&section=Usage&theme=dark&v=…`:
`story` selects the component and `section` its example. Preview mode omits
the gallery shell and does not construct unrelated examples. Point at the
real file path, not the bare `/gallery/?story=…` form: production answers that
with `308 → /gallery?story=…` (Next strips the trailing slash), and
`index.html` resolves its module through an `assetUrl()` helper so both URL
forms boot.

### Rebuilding the artifact (Rust side)

The artifact is compiled from this repository's own workspace:
`crates/herogpui-web` is a normal member of the root `Cargo.toml` workspace,
linking the same `gallery/src/lib.rs` gallery shell the desktop build uses.
The recipe is the one the CI `wasm` job runs (`.github/workflows/ci.yml`);
from the repository root:

```bash
rustup toolchain install nightly --profile minimal -t wasm32-unknown-unknown
cargo +nightly build --locked --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web
wasm-bindgen --target web --no-typescript --out-dir web/public/gallery \
  target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm
cp crates/herogpui-web/index.html web/public/gallery/index.html
cd web && pnpm run wasm:manifest
```

- **`+nightly` is required.** Vanilla `gpui-pre-web` enables its
  `multithreaded` feature by default, which pulls in `wasm_thread`, whose
  `lib.rs` opens with `#![feature]`; stable fails with `error[E0554]`. No
  downstream `default-features = false` edge can turn that default off (cargo
  unions feature sets). Everything native stays on `rust-toolchain.toml`'s
  pinned stable. There is no patch or materialization step: the retired fork
  lives under `docs/upstream/retired-patches/` for the upstream-PR effort only.
- **The multi-threaded platform is unused.** The app starts with
  `gpui_platform::single_threaded_web()`; web workers over shared wasm memory
  need a cross-origin-isolated context this deployment does not provide.
- **Never set `RUSTFLAGS`.** `.cargo/config.toml`'s
  `[target.wasm32-unknown-unknown]` table sets `rustflags = []` deliberately
  (a plain, non-shared-memory wasm that renders without COOP/COEP headers on
  any ancestor page), and the environment variable replaces that list.
- **`wasm-bindgen` CLI = the crate in `Cargo.lock`** (0.2.127 when written); a
  mismatched CLI refuses the binary with a descriptor-schema error.
- **The app is started with `run_embedded`** and its `ApplicationHandle`
  stored (`crates/herogpui-web/src/lib.rs`); plain `run` tears the canvas down
  when the launch callback returns.

`pnpm run wasm:manifest` writes `src/data/wasm-sections.json` (the example
headings compiled into the artifact) and `src/data/wasm-parity.json`, which
pins the artifact and glue bytes, every example body, and a hash of every
wasm build input (the component/theme/core/facade/web/gallery sources, the
workspace manifests and the lockfile). `pnpm run extract:check` (CI's `web`
job) recomputes all of it, so a Rust change that invalidates the committed
artifact fails CI until the artifact is rebuilt. The CI `wasm` job also builds
the artifact and runs `wasm-bindgen` on every PR.

### Why the artifact is committed

The artifact stays in git for now. Building it only in CI and deploying from
CI would need a deploy workflow with Vercel credentials (none are configured
in this repository), and the project's current deploys are CLI deploys from a
checkout; removing the file from the tree before a CI deploy exists would
publish component pages with no live preview. The drift guard above keeps the
committed file honest meanwhile. Moving it out is a follow-up: a workflow that
builds it (the `wasm` job already does), uploads it as an artifact, and runs
`vercel build` + `vercel deploy --prebuilt` with `VERCEL_TOKEN`,
`VERCEL_ORG_ID` and `VERCEL_PROJECT_ID` secrets, validating the manifest
against the freshly built bytes. History is not rewritten.

### Verifying the embed

1. `pnpm dev`, open a component page, and let the preview scroll into view:
   the "HeroGPUI / WebAssembly" frame boots the gallery. "Show live" on a
   later example switches it without a reload.
2. Toggle the site theme: the embedded gallery follows live.
3. `http://localhost:3000/gallery/index.html?story=<slug>` directly: the gallery
   fills the tab, deep-linked to that component.

## Current status

**Deployed 2026-08-30.** The site is live at
[porabuild.com/herogpui](https://porabuild.com/herogpui) (zone project
`herogpui`, production alias `herogpui.vercel.app`, mounted through the
parent-zone rewrites of section 3 — those rewrites are applied in the
parent's checkout and deployed; committing them to the parent repository,
`Porabuild/website`, is still owed). The live WebAssembly gallery is
verified on production: `/herogpui/gallery/herogpui_web_bg.wasm` serves
`application/wasm`, and the embedded frame boots and renders on
`/herogpui/docs/components/button`.

What was deployed, for reproduction: CLI deploys
(`vercel deploy --prod`) from the repository root — not from `web/` — so
the whole workspace uploads and the builder's Root Directory setting
(`web`) picks the app; that is what makes the sibling `../llms.txt`
visible to the `/llms.txt` route. Project settings that matter beyond the
defaults: Root Directory `web`, Install Command
`pnpm install --frozen-lockfile` (see below), Node 24.x. Deploys from
`web/` alone will fail at build (the route cannot read `../llms.txt`).

## Remaining items

1. **The registry release is published.** The `herogpui` crates are on
   crates.io (0.9.0 onward), and the install snippets show `herogpui = "0.11"`.
2. **The Vercel project's git connection is not verified since 2026-08-30.**
   The deploy was CLI-based; if the git integration (Pull Request previews,
   deploy-on-push) is still not set up, deploys remain manual, exactly as in
   "Current status" above.
3. **The parent-zone rewrite commit is not verified since 2026-08-30.** The
   rewrites were applied and deployed from the parent checkout; if they were
   never committed to `Porabuild/website`, a future Git-connected parent
   deploy without them would drop the mount.

## Why there is no `vercel.json`

Deliberately not created. Everything Vercel needs for this project — Root
Directory, framework preset, install/build commands, Node version,
environment variables — is settable in the dashboard, and the settings that
truly affect behaviour in code are already in `next.config.ts` (security and
artifact cache headers) or the environment (`basePath`). The multi-zone mount lives in the
**parent** project's `next.config.ts`, which no `vercel.json` here could
express. A `vercel.json` would only duplicate dashboard state that can
drift from it. If the team later wants the project settings version-controlled,
revisit — this document records the exact values in the meantime.
