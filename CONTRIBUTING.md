# Contributing to HeroGPUI

HeroGPUI is a native Rust/GPUI port of HeroUI v3.2.6. Contributions are
welcome: bug reports, parity fixes, tests, documentation and clearly labelled
extensions.

Parts of this guide (one-thing pull requests, the AI-assisted policy and the
required `## Public API` section) are adapted from
[gpui-kit's CONTRIBUTING.md](https://github.com/longbridge/gpui-kit/blob/main/CONTRIBUTING.md)
(Apache-2.0, Copyright Longbridge).

## Before you start

- Read [`AGENTS.md`](AGENTS.md) and the task guides under
  [`docs/agents/`](docs/agents/): [workflow](docs/agents/workflow.md),
  [components](docs/agents/components.md), [parity](docs/agents/parity.md)
  and [gallery](docs/agents/gallery.md). They apply to human contributors as
  much as to agents.
- Parity work follows HeroUI **v3.2.6** and its pinned React Aria/Stately
  versions, never HeroUI v2 or the latest docs.
- GPUI claims must hold for the exact `gpui-pre` version in `Cargo.lock`.
  Read its unpacked registry sources, not a Zed checkout.
- A component or builder that HeroUI v3 does not have is a **HeroGPUI
  extension**. Label it as one in its docs, record it in the audit tables
  (`extra_audit.py` `EXTRA_OK`, and the extension lists in
  `reference_audit.py`), and never present it as a v3 prop.

## One pull request, one thing

Submit one pull request per change. A focused diff is reviewed and merged
quickly; a diff mixing a fix, a refactor and formatting is not. Do not
reformat, regenerate or rename unrelated files.

## AI-assisted contributions

AI-written code is welcome, including pull requests generated entirely by an
agent. The contributor stays responsible for the result: the problem is real,
the change follows existing patterns, you have reviewed it, and you have run
the relevant tests and gallery pages. Ask your agent to avoid unrelated
cleanup. Less is better.

## Keep every surface in sync

When a component's public API, behavior or gallery example changes, update in
the same pull request:

- the implementation and focused tests under `crates/herogpui-components/`;
- the gallery example and its `reference_metadata` entry;
- `llms.txt`;
- `CHANGELOG.md`;
- the generated website data (`pnpm run extract`, then
  `pnpm run extract:check`, from `web/`); and, when gallery sources change,
  the wasm artifact and its manifests (see `AGENTS.md`), the web font subsets
  (`python3 .shots/subset-fonts.py --check`) and the interaction inventory
  (`python3 .shots/interaction_inventory.py --refresh`, then
  `python3 .shots/coverage_report.py --refresh`).

## Examples

`examples/` holds small runnable workspace crates. They are compiled by
`cargo check --workspace` and by CI's clippy and test jobs, so an API change
that breaks them fails the build:

```sh
cargo run -p herogpui-example-hello-button
cargo run -p herogpui-example-form
cargo run -p herogpui-example-theme-switch
```

New examples use only the `herogpui` facade, set `publish = false`, and
inherit `[lints] workspace = true`.

## Verification

Run what CI runs (`.github/workflows/ci.yml` is authoritative):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
bash .shots/run-tests.sh --workspace --locked
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc --workspace --no-deps --locked
cargo deny check
python3 .shots/parity_report.py --output /tmp/parity-report.json
(cd web && pnpm run extract:check && pnpm run typecheck && pnpm run lint)
```

Report in the pull request which checks you ran and which you could not run.

## Describe public API changes

Every pull request that adds, changes or removes anything public (a type, a
function, a builder method, a trait, a feature flag or a re-export) lists it
in the description under a `## Public API` section, grouped by crate, with the
signature as a reviewer would read it in rustdoc and one line on what it is
for. The reviewer should be able to judge the API from the description alone.
Write `## Public API` followed by `None.` when nothing public changed. The
[pull request template](.github/pull_request_template.md) has the section.
