# HeroGPUI agent guide

HeroGPUI is a native Rust/GPUI port of HeroUI v3.2.4. The repository targets
Rust 1.98 and the published `gpui-pre` crates pinned in `Cargo.toml`
and `Cargo.lock` at exactly `=0.3.3` — zed-industries' own prerelease publish
of the GPUI sources, 0.3.3 being a snapshot of `zed@5b055fa`. Use those
unpacked sources for API evidence —
`~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.3/` — not a Zed git
checkout, not the older third-party `gpui-unofficial` republish this replaced,
and not the unrelated crates.io `gpui` 0.2.2 crate. `gpui-pre`'s version
numbers are its own; they do not track Zed release tags. The pin is exact
because `gpui-pre-platform` requires the family at an exact `=` version and the
vendored `gpui-pre-web` fork's version must satisfy that same requirement or
`[patch.crates-io]` silently stops applying; 0.3.1 and 0.3.2 additionally must
not be resolved, because `gpui-pre-macros` 0.3.1 breaks every
`debug_assertions`-off build (see `RELEASING.md`).

## Before editing

1. Run `git status --short` and inspect the relevant diff. Preserve unrelated
   work in this frequently dirty checkout.
2. Read the target implementation, its callers, and its focused tests before
   changing it. Keep fixes narrow.
3. Use HeroUI v3.2.4 and its pinned React Aria/Stately versions for parity work.
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
cargo build --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web
wasm-bindgen --target web --no-typescript --out-dir web/public/gallery \
  target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm
```

No nightly, and no `RUSTUP_TOOLCHAIN` override: this target builds on
`rust-toolchain.toml`'s pinned stable like every other. Older revisions of this
guide said at length that nightly "is required and is not a preference", and
the reason was real — `wasm_thread`, pulled in by the GPUI web platform's
`multithreaded` feature, opens its `lib.rs` with a `#![feature]` attribute, so
stable failed with `error[E0554]` from a dependency this repository does not
own. That feature is now off, so `wasm_thread` is not in the wasm32 graph at
all. Two facts about how, because both are easy to get wrong:

- The switch is **not** `default-features = false` on a dependency edge of
  ours. `gpui_platform` depends on `gpui_web` with default features on, so a
  second edge from this workspace is unioned with that one and changes nothing.
  The only lever is `default` in the vendored fork's own manifest,
  `crates/gpui_web/Cargo.toml`, which `[patch.crates-io]` substitutes. It is
  set to `default = []` there, documented at the `[features]` table as the
  fork's third deviation from upstream.
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
have parted company. `docs/upstream/gpui-web-scroll-and-ime.md` covers the one
remaining fork, the +9/-5 lines in `crates/gpui_web`.
