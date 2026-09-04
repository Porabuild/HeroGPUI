# HeroGPUI website — handoff

State as of 2026-08-30. Read `ARCHITECTURE.md`, `DESIGN-NOTES.md` and
`COPY-GUIDE.md` — they are binding contracts, not background reading.
`DEPLOYMENT.md` is the operator runbook.

## What is finished and verified

The site is at `E:\work\HeroGPUI\web`. Next.js 16.3.3, React 19.2.8,
`@heroui/react` 3.2.4, Tailwind 4.3.3, **pnpm — never npm**.

`pnpm run build`, `typecheck`, `lint` and `format:check` all pass. 84 static
pages, zero warnings.

| | |
|---|---|
| Components | 66 pages across 15 categories |
| API reference | 61 components, extracted from `gallery/src/pages/reference_metadata.rs` |
| Rust examples | 637, extracted from `gallery/src/pages/components.rs` |
| Changelog | 288 commits, from git history |
| Screenshots | 89, from `.shots/` |

Routes: landing, seven getting-started pages, three AI pages, the component
index, 66 component pages, releases, and `/llms.txt` served as prerendered
plain text.

All page content is generated from the Rust workspace by `scripts/*.mjs`. Run
`node scripts/build-data.mjs` to regenerate. Do not hand-edit `src/data/`.

## Live WebAssembly previews: working, pending hosting

**Goal.** Each component page shows the real HeroGPUI component running in the
browser, with its Rust below — the gallery pages, on the web, for people to
try. Not screenshots, not a recreation in another framework.

**Host side: done.** `src/components/preview/gallery-frame.tsx` renders an
iframe pointing at `${NEXT_PUBLIC_GALLERY_URL}/?story=<slug>&theme=<theme>`,
booted lazily by an `IntersectionObserver` so a multi-megabyte download never
fires on page load, with the host's light/dark passed through. When
`NEXT_PUBLIC_GALLERY_URL` is unset — the shipped default — it renders nothing
and the page falls back to the native screenshot. That is deliberate: see
`DESIGN-NOTES.md` on never filling an empty preview with a placeholder.

**Gallery side: renders.** The work lives in a separate git worktree at
`D:\herogpui-wasm` (detached HEAD). Full log in
`D:\herogpui-wasm\WASM-MIGRATION.md` — Phase 5 there is the resolution and
supersedes every theory in Phases 1–4.

- `gpui` moved from crates.io 0.2.2 to Zed git, pinned at rev
  `f66ed399cdde86092af8af3dc7b418abf45f37f8` — the commit
  `longbridge/gpui-component` resolves. `gpui_platform` added.
- `rust-toolchain.toml` on `nightly-2026-08-30` with `wasm32-unknown-unknown`.
- `cargo check --workspace` clean. `cargo test -p herogpui-components`:
  **1310 passed, 32 failed** across 12 binaries (`calendars_and_more`,
  `calendars_deep`, `drawer_deep`, `feedback`, `overlays`, `pickers_deep`,
  `placement`, `placement_extra`, `popover_stack_deep`, `render_props`,
  `theme_repaint`, `virtual_and_feedback`).
- **Baseline confirmed 2026-08-30: the unmigrated crates.io checkout passes
  everything** — the same suite on `E:\work\HeroGPUI` (GPUI 0.2.2, 58
  binaries, `--no-fail-fast`, log at `D:\baseline-test.log`) reports **0
  failed**, including every one of those 12 binaries. So the 32 are **not**
  pre-existing: they are behaviour deltas introduced by the GPUI swap, in
  exactly the clusters the migration log suspected (overlay/drawer/popover
  hit-testing, toast dismiss-on-click, keyboard-close ordering, calendar
  colour rounding — the pinned GPUI's live-hover recompute and hit-testing
  changes are documented root causes for two of the fixed clusters). This is
  a real v3-parity debt carried by the web pin, not background noise. Fixing
  them means the same GPUI-source-grade investigation as the two clusters
  already fixed; none were touched.
- `crates/herogpui-web` builds with the `wasm-release` profile: **15.73 MiB
  raw / 4.97 MiB gzipped**. Artifact, production `index.html` and an
  instrumented `index-diagnostic.html` are in `D:\herogpui-wasm-serve\`.
- A real bug found and fixed: `jiff` panicked on every current-time lookup on
  wasm32; the `js` feature in the root `Cargo.toml` fixes it.

**The former blocker, resolved 2026-08-30.** "The canvas never paints" was
never a graphics failure: initialization succeeded every time, and the canvas
was being removed by the window's own `Drop` **~95 ms after a successful
launch**, because `run()` used plain `Application::run` — whose keep-alive
assumes a blocking platform run loop the web platform does not have. The
launch future completes, drops the last strong `Rc` to the app state, and the
whole application tears down before its first animation frame. The fix is
`Application::run_embedded` plus a thread-local `ApplicationHandle` — what
Longbridge's `story-web` does. The proof was a JS stack captured at canvas
removal time by the diagnostic instrumentation (`Rc<AppCell>::drop_slow`
inside the launch future); it had been recorded but never read.

Verified in the browser, by eye: full gallery renders (text, icons, cards,
syntax-highlighted code); `?story=button&theme=dark` deep-links to the Button
page in dark; clicks navigate; `set_theme` flips the running app in place;
and a 608×680 iframe — the site's embed shape — hosts the gallery and follows
the host page's live theme toggle.

### The embed contract (matching Longbridge, confirmed working)

```html
<iframe src="<gallery>/index.html?story=<slug>&preview=component&section=<heading>&theme=<theme>" title>
```

- Deep link is a **query parameter**: `?story=<catalog slug>` (e.g.
  `date-picker`) — the slug scheme the site already generates — or
  `?page=<Nav Title>` as the native-parity alias.
- **One lazily loaded instance per page.** Preview mode constructs only the
  selected example and omits the gallery shell. The host switches examples by
  posting `herogpui:preview-section`; the wasm application stays alive.
- **No documentation prose in the canvas.** The artifact build runs
  `lift-wasm-descriptions.mjs` so legacy static paragraphs become section
  descriptions rendered by the website above the live component.
- **Theme**: `?theme=` sets the boot theme; if the frame can read the parent
  document (same origin), it follows the host's live `<html>` class changes
  via the exported `set_theme(dark)` — no reload.

**What remains is deploying, not building.** The plain-memory artifact is
staged in `web/public/gallery/` (`index.html` + `herogpui_web.js` +
`herogpui_web_bg.wasm`), `next.config.ts` maps `/gallery` onto its
`index.html`, `.env.example` documents `NEXT_PUBLIC_GALLERY_URL=/gallery`,
and `DEPLOYMENT.md` §6 is the operator runbook. Verified end-to-end on a
dev server: the exact iframe `GalleryFrame` computes
(`/gallery/index.html?story=button&preview=component&section=Usage&theme=light`)
boots inside a component page,
deep-links, sizes to the frame, and renders. One caveat: the browser tool's
`IntersectionObserver` never fires in an occluded window, so the
lazy-boot trigger itself was exercised by code review only; on a visible
browser it is the same observer pattern the screenshots' lazy loading
uses.

Two traps that each masqueraded as a graphics bug — both now encoded in the
tree, do not regress them:

- **`D:\herogpui-wasm\.cargo\config.toml` pins `rustflags = []` for wasm,
  deliberately** (mirroring Longbridge's `story-web`). A shared-memory build
  would demand cross-origin isolation on every ancestor page up to
  porabuild.com itself, and simply never renders under plain hosting.
- **The app is started with `run_embedded` + a stored `ApplicationHandle`.**
  Plain `run` tears the whole app — canvas included — down the moment the
  launch callback returns.

`D:\herogpui-wasm-serve\` remains the scratch serving copy, with
`index-diagnostic.html` (the instrumented bootstrap that caught the
teardown) and `iframetest.html` (embed-shape harness) for future debugging.

## Environment traps that have each cost real time

- **`CARGO_TARGET_DIR=E:\work\.cargo-target` and `CARGO_HOME` are ambient
  environment variables pointing at `E:`, regardless of working directory.**
  This filled the disk twice and produced a build that reported success while
  its output landed somewhere the capture scripts do not look. Pass both
  explicitly on every cargo command when working in the `D:` worktree.
- **`E:` is small and fills fast.** Keep an eye on it; `D:` has room.
- **`capture2.ps1` writes `<page>-v3.png` regardless of `-Theme`.** Running it
  with `-Theme` against the default output directory overwrites the *light*
  captures. Capture to a scratch `-OutDir`, then rename in.
- **Next caches optimised images across builds.** `.next/cache/images` is
  keyed by source URL, which does not change when the file behind it does.
  After swapping anything in `public/shots/`, delete that directory before
  rebuilding, or you will conclude your replacement failed.
- **`capture2.ps1` uses the `debug` binary**, not release.

## Known open items

- **The GitHub repository is private.**
  `git@github.com:Porabuild/HeroGPUI.git` 404s to anonymous requests, so
  every GitHub link **on the site** (nav, hero, final CTA) breaks for
  visitors until it is made public. The repo itself exists and is live:
  branch `v3-parity` (open PR #1 against `master`, the trunk), `web/`
  committed, screenshots and the wasm artifact tracked.
- **The site is deployed** — [porabuild.com/herogpui](https://porabuild.com/herogpui),
  live since 2026-08-30 via Vercel CLI deploys from the repository root
  (project `herogpui`, alias `herogpui.vercel.app`, parent-zone rewrites
  applied and deployed). Reproduction and the two project settings that
  are load-bearing (Root Directory, Install Command — see the pnpm 12.1.0
  corrupt-artifact note) are in `DEPLOYMENT.md` "Current status" and §1.
  Still owed: commit the parent-zone rewrites to `Porabuild/website`, and
  a Git connection on the Vercel project so previews/deploy-on-push work.
- The two extractor bugs below were **fixed 2026-08-30** (kept here so the
  commit history of `rust-examples.json` makes sense): `ActiveTheme` now
  lands in generated imports for snippets calling `.colors()`, and
  `fixed_demo`'s expansion wraps its width in `px()`.

## Deployment history

- 2026-08-30 — first production deploy, then the mount. Failures hit on
  the way and are all recorded in `DEPLOYMENT.md`: Vercel's pnpm
  autodetection picked pnpm 10.x and appended `--unsafe-perm` (rejected),
  the explicit Install Command fixed that; pnpm 12.1.0's cached Linux
  launcher on the builder is corrupt (fixed by pinning `pnpm@10.34.5`);
  a `web/`-rooted CLI upload lacks `../llms.txt` (fixed by deploying from
  the repository root with Root Directory `web`); CLI uploads honour
  `.gitignore`, which is why `public/shots/` and `public/gallery/` are now
  tracked rather than ignored.
