# HeroGPUI agent guide

HeroGPUI is a native Rust/GPUI port of HeroUI v3.2.4. The repository targets
Rust 1.98 and GPUI 0.2.2; newer upstream APIs are not evidence that an API is
available here.

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
- [Component implementation](docs/agents/components.md) — GPUI 0.2.2 state,
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
- the matching Rust gallery example and `reference_metadata.rs` entry;
- `llms.txt` when the public Rust API or behavior changed; and
- the generated website component data in `web/src/data/reference.json` and
  `web/src/data/rust-examples.json`.

Regenerate both website datasets from `web/` with `pnpm run extract`, then
verify them with `pnpm run extract:check`. Do not hand-edit generated JSON.
When rebuilding `web/public/gallery/herogpui_web*`, also regenerate
`web/src/data/wasm-sections.json` and `web/src/data/wasm-parity.json` from the
exact migration source after `wasm-bindgen`. These manifests limit the selector
to compiled examples, pin the native examples and artifact, and reject new
native/WASM drift by default.
