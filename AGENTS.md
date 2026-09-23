# HeroGPUI agent guide

HeroGPUI is a native Rust/GPUI port of HeroUI v3.2.5. The repository targets
Rust 1.98 and the published `gpui-pre` crates pinned in `Cargo.toml`
and `Cargo.lock` at exactly `=0.3.5` — a prerelease publish of the Zed GPUI sources by
crates.io user huacnlee (Jason Lee, the gpui-kit maintainer), 0.3.5 being a
snapshot of `zed@d89e9c2`. `gpui-pre`'s
version numbers are its own; they do not track Zed release tags. The pin is
exact (`=0.3.5`), as gpui-kit's is: `gpui-pre` is a 0.x prerelease snapshot,
so a caret would let any `0.3.x` snapshot of different Zed sources resolve in
a downstream build. `.shots/package_audit.py` rejects a non-`=` requirement.

## No setup step: vanilla registry GPUI

HeroGPUI builds on the published `gpui-pre` family as-is. There is no
`[patch.crates-io]` table, no generated source tree, and no materialization
command: a fresh clone (and rust-analyzer on it) runs `cargo` directly, and
a downstream crate needs only `herogpui` itself. The retired renderer,
accessibility, and web-platform deviations live under
`docs/upstream/retired-patches/` for the upstream-PR effort only; they are
not wired into any build. Component-level replacements for them live in
`crates/herogpui-components/src/` (`a11y.rs`, `util::shift_wheel_scroll_x`,
`util::inner_fill_radius`).

Use the unpacked registry sources for API evidence —
`~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.5/`. That is neither a
Zed git checkout, the older third-party `gpui-unofficial` republish this
replaced, nor the unrelated crates.io `gpui` 0.2.2 crate.

## Before editing

1. Run `git status --short` and inspect the relevant diff. Preserve unrelated
   work in this frequently dirty checkout.
2. Read the target implementation, its callers, and its focused tests before
   changing it. Keep fixes narrow.
3. Use HeroUI v3.2.5 and its pinned React Aria/Stately versions for parity work.
   Do not infer behavior from HeroUI v2, latest docs, or a newer GPUI checkout.
4. Read the task guide below before acting. Scoped `AGENTS.md` files under
   `.shots/`, `crates/herogpui-components/`, and `gallery/` add local rules.

## Core commands

```powershell
cargo check --workspace
cargo test -p herogpui-components
cargo fmt --all -- --check
.shots/lint.ps1
```

Use a focused test binary while iterating. After a component or gallery change,
build with `.shots/rebuild.ps1`; the gallery executable is often locked after a
smoke or capture run. Select gates from the workflow matrix; reserve the full
CI-shaped set for release-facing code changes or an explicit request.

## Task guides

- [Workflow and architecture](docs/agents/workflow.md) — repository map,
  source hierarchy, scope discipline, and change-to-verification matrix.
- [Component implementation](docs/agents/components.md) — pinned GPUI state,
  events, focus, overlays, layout, and behavior-test patterns.
- [Parity and audits](docs/agents/parity.md) — pinned upstream contract,
  audit selection, omission rules, and audit-reader integrity.
- [Gallery and visual verification](docs/agents/gallery.md) — rebuild, smoke,
  deep links, off-screen input, screenshots, and focus-sensitive capture.

`llms.txt` is the public component API reference. It supplements the
task guides; it does not replace reading the implementation and tests.

## Keep component surfaces in sync

When a component's public API, behavior, reference status, or gallery example
changes, update the complete affected surface in the same change:

- the implementation and focused behavior tests under
  `crates/herogpui-components/`;
- the matching Rust gallery example and `reference_metadata` entry;
- `llms.txt` when the public Rust API or behavior changed; and
- the generated website component data in `web/src/data/reference.json` and
  `web/src/data/rust-examples.json`.

Regenerate both website datasets from `web/` with `pnpm run extract`, then
verify them with `pnpm run extract:check`. Do not hand-edit generated JSON.

## The web gallery is built from this workspace

There is one copy of every component, and it needs no wasm-specific code.
`crates/herogpui-web` is an ordinary workspace member that links the
`herogpui-gallery` library (`gallery/src/lib.rs`, which `gallery/src/main.rs`
also uses) and compiles for `wasm32-unknown-unknown`, so the browser runs the
same sources the native binary does. Earlier layouts built the artifact from a
second checkout pinned to an older commit and kept the two trees in step by
hand; nothing of that survives, and no tool reports a component as behind.

Rebuild the checked-in artifact from the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo +nightly build --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web
wasm-bindgen --target web --no-typescript --out-dir web/public/gallery \
  target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm
```

The `+nightly` is required and is not a preference — `wasm_thread`, pulled
in by the GPUI web platform's `multithreaded` default feature, opens its
`lib.rs` with a `#![feature]` attribute, so stable fails with `error[E0554]`
from a dependency this repository does not own. Two facts about how, because
both are easy to get wrong:

- The switch is **not** `default-features = false` on a dependency edge of
  ours. `gpui_platform` depends on `gpui_web` with default features on, so a
  second edge from this workspace is unioned with that one and changes nothing.
  Upstream's `multithreaded` default is therefore accepted as-is, which pulls
  `wasm_thread` (a `#![feature]` crate) into the wasm32 graph: the `wasm` CI
  job builds that target on nightly, while everything native stays on
  `rust-toolchain.toml`'s pinned stable. See `docs/upstream/gpui-web-scroll-and-ime.md`.
- Nothing was using it. `crates/herogpui-web` starts the app with
  `gpui_platform::single_threaded_web()`, and the multi-threaded platform runs
  its background executors on web workers over shared wasm memory, which a
  browser grants only in a cross-origin-isolated context — and the gallery is
  served from plain GitHub Pages, which sends no COOP/COEP headers. Verified in
  a browser at `gpui-pre` 0.3.3: the stable `wasm-release` artifact boots,
  renders, presses, focuses, takes keyboard input and resolves a
  `background_executor().timer()`, indistinguishably from the
  nightly/`multithreaded` artifact, which is 23 KB larger. Turning
  `multithreaded` back on means going back to a nightly pin, so do it only
  together with a deployment that can actually use web workers.

Never set `RUSTFLAGS` for this target —
`.cargo/config.toml` states `rustflags = []` there deliberately, and the
environment variable replaces that list rather than adding to it. Use the
`wasm-bindgen` CLI whose version matches the `wasm-bindgen` crate in
`Cargo.lock`; a mismatch fails on a descriptor schema neither side names. CI's
`wasm` job does all of this on every pull request.

The artifact is a committed ~19 MB binary that no compiler checks against the
sources, so after rebuilding it regenerate its manifests in the same change
with `pnpm run wasm:manifest` from `web/`. They pin the artifact and every
gallery example body by hash, and `pnpm run extract:check` fails when the two
have parted company. `docs/upstream/gpui-web-scroll-and-ime.md` covers the retired source fork of
that crate (+9/-5 lines in `src/events.rs`, kept under
`docs/upstream/retired-patches/` for the upstream-PR effort) and the known
web limitations vanilla accepts: shift+wheel reaches horizontal scrollers
through `util::shift_wheel_scroll_x`, while the IME-mirror resync after paste
still waits upstream.
