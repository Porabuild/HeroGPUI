# Multithreaded wasm (`gpui_web` `multithreaded`) and cross-origin isolation

Evaluated for 0.13; **not adopted**. The gallery keeps starting the app with
`gpui_platform::single_threaded_web()`, and the artifact keeps the plain
(non-shared) memory that `.cargo/config.toml`'s `rustflags = []` produces.
This note records what turning the multi-threaded platform on would take,
and the exact blockers, so the question is not re-derived from scratch.

## What the multi-threaded platform needs

From `gpui-pre-web` 0.3.5 (`zed@d89e9c2`):

- **A shared-memory module.** `WebDispatcher::new`
  (`src/dispatcher.rs:151-209`) spawns `navigator.hardwareConcurrency`
  background workers only when `shared_memory_supported()` (the module's
  memory is a `SharedArrayBuffer`) and `Atomics.waitAsync` exists; otherwise
  it logs a warning and falls back to the single-threaded dispatcher. A
  shared memory needs the wasm built with
  `-C target-feature=+atomics,+bulk-memory,+mutable-globals` and
  `-Z build-std=std,panic_abort` on nightly -- the `rustflags` this
  repository deliberately removed.
- **Cross-origin isolation of the top-level page.** Browsers expose
  `SharedArrayBuffer` (and accept a shared `WebAssembly.Memory`) only when
  `crossOriginIsolated` is true, which requires the *top-level* document to
  be served with `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp` (or `credentialless`), and
  every frame in between to carry COEP too. An iframe cannot become isolated
  on its own: isolation is decided by the top-level browsing context.
- **Blob workers and eval in the worker bootstrap.** `wasm_thread` 0.3.3
  starts each worker from a `Blob` URL (`src/wasm32/utils.rs:63`) and ships a
  module-workers polyfill that compiles code with `new Function`
  (`src/wasm32/js/module_workers_polyfill.min.js`). The gallery CSP
  (`web/next.config.ts`) would need `worker-src blob:` and, for the polyfill
  path, `'unsafe-eval'` in the worker's `script-src`.

## Where the gallery runs, and what isolation would cost there

| Surface | Top-level document | Can it be isolated? |
|---|---|---|
| Component pages and the landing specimen (the embeds) | a site page, proxied through `porabuild.com` (a different project's zone, `web/DEPLOYMENT.md` section 3) | Only by sending COOP/COEP on every site page. Every subresource must then be same-origin or CORP/CORS-enabled; Vercel's preview toolbar (`vercel.live`, injected on previews) does not send CORP; Safari implements `require-corp` but not `credentialless`; and COOP `same-origin` severs `window.opener` for anything the site opens or is opened by. The parent zone would have to pass the headers through unchanged. |
| The standalone gallery (`/gallery`, `/gallery/index.html` opened directly) | the gallery document itself | Yes: COOP/COEP on the `/gallery` routes alone would isolate it, and it loads nothing cross-origin. |

The same `/gallery/index.html` URL serves both rows, so a standalone-only
isolation still has to boot the embeds single-threaded. A shared-memory
module cannot be instantiated in a non-isolated document, so that means
**two artifacts**: the current one for the embeds and an atomics/`build-std`
one for the standalone page, chosen at boot by `crossOriginIsolated`. That
doubles the CI `wasm` job (a second full nightly `build-std` build) and the
published artifact set, adds the CSP loosening above to the gallery route,
and gives up the `wasm-opt`/cache-versioning simplicity of one module.

## Why it is not worth it now

- **No measured benefit.** The gallery's background-executor use is timers
  (`tooltip.rs`, `avatar.rs`, `number_field.rs`) and theme watching, which is
  native-only; rendering, layout and input run on the foreground thread in
  both platforms. The 0.3.3 comparison (root `AGENTS.md`) found the
  single-threaded artifact indistinguishable in booting, rendering, input and
  timers. `gpui_web`'s own `TODO-Wasm` notes its workers block for their
  whole lifetime.
- **The embeds are the product.** Nearly every visitor meets the gallery
  inside a component page, which is the surface isolation cannot reach
  without site-wide COOP/COEP across two Vercel projects.

## What would change the decision

Revisit when either holds, and then implement standalone-first:

1. GPUI moves real work (text shaping, image decoding, layout) onto the
   background executor on the web, so threads buy something measurable; or
2. the site can serve COOP/COEP on every page (both zones, previews
   included) and drop Safari-incompatible `credentialless` requirements.

The implementation then is: a second `.shots/build-wasm.sh` target with the
atomics `rustflags` in a dedicated cargo profile or `--config` (never the
`RUSTFLAGS` variable), `gpui_platform`'s multi-threaded init in
`crates/herogpui-web` behind a runtime `crossOriginIsolated` check,
`Cross-Origin-Opener-Policy`/`Cross-Origin-Embedder-Policy` headers and the
`worker-src blob:` CSP on the `/gallery` routes in `web/next.config.ts`, and
a second artifact pair in each `gallery-<key16>` release.
