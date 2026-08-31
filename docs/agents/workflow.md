# Workflow and architecture

Read this guide for every implementation task. It defines where code belongs,
which sources are authoritative, and how much verification a change needs.

## Start from the current checkout

```powershell
git status --short
git branch --show-current
git diff -- <paths-you-will-touch>
```

This repository commonly contains unfinished component, gallery, audit, and
screenshot work. Treat every pre-existing modification as user-owned. Do not
format, revert, regenerate, or include unrelated paths. A read-only review does
not authorize edits, builds, formatting, or test runs unless the request says
otherwise.

Read before writing:

1. The complete target file and the relevant `impl` block.
2. Callers and sibling components that establish the local pattern.
3. Focused tests in `crates/herogpui-components/tests/`.
4. The exact pinned upstream source when the change claims parity.
5. The current diff again after editing.

Do not turn a focused fix into a refactor. Delete unsupported or obsolete code
instead of leaving compatibility aliases, no-op builders, or speculative flags.

## Repository map

- `crates/herogpui-core` owns shared v3 vocabularies and color math. It contains
  no component code.
- `crates/herogpui-theme` owns semantic and layout tokens. `ThemeProvider` is a
  GPUI global; consumers read it with `ActiveTheme` (`cx.colors()`,
  `cx.role(..)`, `cx.layout()`).
- `crates/herogpui-components` owns the component implementations and headless
  behavior tests. Components are builder structs implementing `RenderOnce`.
- `crates/herogpui` is the umbrella re-export crate and prelude.
- `gallery` is the documentation app. Page routing and categories live in
  `gallery/src/pages/mod.rs`; component demos live primarily in
  `gallery/src/pages/components.rs`; checked-in v3.2.4 API metadata lives in
  `gallery/src/pages/reference_metadata.rs`.
- `.shots` contains parity audits, headless gallery drivers, reference images,
  and the real lint gate.
- `llms.txt` is the public API reference intended for LLM consumers.

## Source hierarchy

Use the narrowest source that actually owns the contract:

1. This checkout's tests and code for its current behavior and supported API.
2. HeroUI v3.2.4 component code and styles for port parity.
3. HeroUI's exact pinned dependencies for inherited interaction semantics:
   React Aria 3.51.0, React Stately 3.49.0, and React Aria Components 1.20.0.
4. Installed GPUI 0.2.2 source for framework behavior and available APIs.
5. Zed `main` or other GPUI projects only as precedent, never as proof that
   GPUI 0.2.2 supports an API.

The live `https://heroui.com/react/llms-full.txt` bundle is an input to several
audits, but tagged source and the checked-in reference metadata establish the
repository's v3.2.4 contract. A green audit is only evidence for what that audit
actually reads.

## Project invariants

- HeroUI v3 only. Do not reintroduce v2 tokens, components, or props.
- Keep upstream OKLCH values and component metrics exact; do not aesthetically
  "improve" a parity value.
- Builder names follow v3 prop names in Rust spelling. This is why the workspace
  allows Clippy's `wrong_self_convention` lint.
- Callback fields cloned into GPUI closures use `Arc<dyn Fn ...>` rather than
  `Box<dyn Fn ...>`.
- Interactive elements need unique ids per page and per component instance.
- Every workspace crate must inherit `[workspace.lints]` with
  `[lints] workspace = true`; `.shots/lint.ps1` enforces this.
- Do not copy totals from audit output into instructions. Route, prop, example,
  and coverage counts change; let the scripts print the current values.

## Verification by change type

| Change | Iterate with | Before broad handoff |
|---|---|---|
| Rust logic in one component | Focused `cargo test -p herogpui-components --test <name>` | Component suite, format, lint, relevant audits |
| Public builder/API | Focused tests plus `api_audit.py`, `extra_audit.py`, `write_only.py` | All audits and package checks when release-facing |
| Tokens or component metrics | `token_audit.py`, `design_audit.py`, focused screenshots | Format, lint, affected behavior tests, visual check |
| Interaction, focus, overlay, or state | Focused behavior binary | Gallery drive for the real path, then component suite |
| Gallery demo or reference metadata | Gallery tests and relevant demo/reference audit | Rebuild, route smoke, focused capture |
| Audit parser or mapping | Run the changed audit against known-positive and known-negative input | Run every `*audit.py` as CI does |
| Documentation only | Check links, commands, and current file names | No Rust build unless the documentation changes code generation |

Normal tests are one-shot. `bacon` is the opt-in persistent check loop; do not
replace ordinary test commands with watch mode.

## CI-shaped verification

`.github/workflows/ci.yml` is authoritative. Its Rust job currently runs:

1. `cargo fmt --all -- --check`
2. `cargo test -p herogpui-components --locked`
3. `.shots/lint.ps1`
4. Fetch parity inputs with `design_audit.py --fetch` and
   `demo_audit.py --fetch`
5. Every `.shots/*audit.py`
6. Per-crate `cargo package --list`, workspace publish dry-run, and a gallery
   install smoke

Do not claim the full gate passed after running only a focused test or one audit.
For documentation-only changes, verify the documentation directly rather than
spending minutes compiling unrelated Rust.

## Finish with evidence

Re-run `git status --short` and inspect the exact diff. Report which checks ran,
which did not, and why. A screenshot proves pixels, a headless behavior test
proves the exercised event path, and an audit proves only its mapped surface;
none substitutes for the others.
