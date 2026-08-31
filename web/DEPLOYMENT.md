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
- A component page (`/herogpui/docs/components/button`) shows its GPUI
  screenshot; the `<img src>` resolves under `/herogpui/shots/…`.
- Navigate Docs → Components → a component page: internal navigation stays
  inside the `/herogpui` prefix (it is one zone, so these are soft
  navigations).
- View source: canonical/Open Graph URLs begin with
  `https://porabuild.com/herogpui` (that is `NEXT_PUBLIC_SITE_URL` doing
  its job via `metadataBase`).

### Known issue found while verifying the base path

The component pages' native screenshots use `next/image`
(`src/components/preview/native-shot.tsx`), and under a basePath the
optimizer URLs it emits do not resolve: `<Image src="/shots/x.png">`
renders `src="/herogpui/_next/image?url=%2Fshots%2Fx.png&…"` — endpoint
prefixed, `url` parameter not — and Next 16.3.3's optimizer rejects that
with 400 `The requested resource isn't a valid image`. The identical
endpoint answers 200 when the parameter is prefixed
(`url=%2Fherogpui%2Fshots%2Fx.png`). Verified against a production build
served by `next start`; whether Vercel's edge optimizer behaves the same
cannot be confirmed before the first deployment, so check the component
pages in the list above. The likely fix, for the file's owner to apply, is
the same `publicUrl()` wrapper the plain `<img>` sites already use — pass
`publicUrl(component.shot)` to `<Image>` (and keep passing the raw path to
`pngSize()`) — which works identically when the base path is empty.

## 6. The live WebAssembly gallery

Component pages embed the real HeroGPUI gallery — the Rust application
compiled to wasm — in an iframe (`GalleryFrame`,
`src/components/preview/gallery-frame.tsx`). Three files make that work;
they live in `public/gallery/` and are served by the same deployment:

| file | what it is |
|---|---|
| `index.html` | the hosting page (loading spinner, error UI, boot script; canonical source: `crates/herogpui-web/index.html` in the wasm worktree) |
| `herogpui_web.js` | `wasm-bindgen` glue |
| `herogpui_web_bg.wasm` | the application, ~26 MB raw / ~7 MB gzipped |

`next.config.ts` maps `/gallery` onto `/gallery/index.html` (public/ has no
directory-index resolution), so `NEXT_PUBLIC_GALLERY_URL=/gallery` is the
intended value. It is a build-time variable: setting or changing it takes
effect on the next deployment. It is unset by default — without the artifact
at that path, previews render nothing and pages fall back to the native
screenshot, which is the honest state.

The three files are **tracked in git** (alongside `public/shots/`): remote
builds run `next build` alone — no Rust toolchain, no capture rig — so the
artifact and the screenshots must ship in the tree. The 26 MB binary only
changes when the wasm build is regenerated; rebuilding it means running
the commands below and committing the result.

Because the artifact lives under the same origin, the embedded gallery
follows the site's live light/dark toggle (`GalleryFrame` also passes
`?theme=` at boot for the first paint). Deep links ride the query string:
`/gallery/?story=button&theme=dark` opens the Button page in dark — the
same slug the component URLs use, resolved by the wasm itself.

### Rebuilding the artifact (Rust side)

From the wasm worktree `D:\herogpui-wasm` (full log: its
`WASM-MIGRATION.md`):

```powershell
$env:CARGO_TARGET_DIR='D:/herogpui-wasm-target'; $env:CARGO_HOME='D:/cargo-home'
cargo build --target wasm32-unknown-unknown --release -p herogpui-web
D:\cargo-home\bin\wasm-bindgen.exe `
  D:\herogpui-wasm-target\wasm32-unknown-unknown\release\herogpui_web.wasm `
  --out-dir <this repo>\web\public\gallery --target web --no-typescript
```

Copy `index.html` from `crates/herogpui-web/` alongside (the bindgen output
only produces the two `herogpui_web.*` files). The `wasm-bindgen` CLI
version must match the `wasm-bindgen` crate in `Cargo.lock` exactly
(0.2.127 when written) — a mismatched CLI refuses the binary.

Two load-bearing details on the Rust side, both verified empirically:

- **`D:\herogpui-wasm\.cargo\config.toml` pins `rustflags = []` for the wasm
  target — deliberately.** This mirrors `longbridge/gpui-component`'s own
  `story-web` config and produces a *plain* (non-shared-memory) wasm. The
  alternative — the shared-memory/atomics build copied from `gpui_web`'s
  `hello_web` example — imports a `SharedArrayBuffer`-backed memory, which
  browsers only grant inside a cross-origin-isolated context, and that
  requirement inherits to *every ancestor page*: the docs site, and the
  porabuild.com page mounting it. Verified: the shared-memory build never
  renders under plain hosting (canvas stuck at 1×1); the plain build
  renders with no isolation headers anywhere. Do not reintroduce the
  atomics flags.
- **The app must be started with `run_embedded` and its
  `ApplicationHandle` stored** (`crates/herogpui-web/src/lib.rs`). Plain
  `run` lets the whole application — canvas included — tear down the
  moment the launch callback returns; that was the original "canvas never
  paints" bug.

### Verifying the embed

1. `pnpm dev` with `NEXT_PUBLIC_GALLERY_URL=/gallery` (or `.env.local`),
   open a component page, and let the preview scroll into view: the
   "HeroGPUI / WebAssembly" frame boots the gallery.
2. Toggle the site theme: the embedded gallery follows live.
3. `http://localhost:3000/gallery/?story=<slug>` directly: the gallery
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

1. **The GitHub repository is private.** Every GitHub link **on the site**
   — the nav, hero, and final-CTA buttons point at
   `github.com/Porabuild/HeroGPUI` — 404s for visitors until it is made
   public. (Links to `heroui-inc/heroui` in the component reference
   sections are upstream and already work.)
2. **The Vercel project is not Git-connected.** The deploy was CLI-based;
   the git integration (Pull Request previews, deploy-on-push) is not set
   up. Until it is, deploys are manual, exactly as above.
3. **The parent-zone rewrites are not committed** to `Porabuild/website`
   (applied and deployed from the checkout; a future Git-connected parent
   deploy without them would drop the mount).
4. **The registry release is not published.** Not a deployment blocker, but
   the site says so honestly: `herogpui` on crates.io is prepared, not
   published, and the install snippets show the path-dependency workaround.

## Why there is no `vercel.json`

Deliberately not created. Everything Vercel needs for this project — Root
Directory, framework preset, install/build commands, Node version,
environment variables — is settable in the dashboard, and the settings that
truly affect behaviour in code are already in `next.config.ts` (security
headers) or the environment (`basePath`). The multi-zone mount lives in the
**parent** project's `next.config.ts`, which no `vercel.json` here could
express. A `vercel.json` would only duplicate dashboard state that can
drift from it. If the team later wants the project settings version-controlled,
revisit — this document records the exact values in the meantime.
