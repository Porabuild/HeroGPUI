<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->

## Generated component pages

Component pages consume checked-in data generated from the Rust gallery. When
component API/reference metadata or examples change, regenerate both outputs
with `pnpm run extract`. Run `pnpm run extract:check` before handoff; CI runs
the same non-mutating check.
Keep `gallery/src/pages/reference_metadata/`,
`gallery/src/pages/components/`, `web/src/data/reference.json`,
`web/src/data/rust-examples.json`, and the public `llms.txt` description aligned;
do not hand-edit generated JSON.

Example presentation follows the Rust gallery source: section title, optional
description, then the bordered example. Keep explanatory prose outside the
example surface on both sites; component-owned labels, helper text, values, and
composed content remain inside the demonstrated component.

Keep each component on one page: `Usage` (one lazy GPUI/WASM instance, then
every generated example as its own anchored heading with description and
code; "Show live" or a `#rust-<example>` link switches the one instance),
compact `Anatomy`, `Customization`, `API reference` (Props, Parts and slots,
then States, each marking Partial rows and listing Not ported entries), and
`Related components`.

The shown Rust is public-API code: `scripts/extract-rust-examples.mjs` lifts
the page `let`s and gallery helper definitions an example needs, rewrites
gallery-only paths, and refuses any snippet that still names one.
`gallery/build.rs` compiles every snippet in `rust-examples.json` as a test
module, so `cargo test -p herogpui-gallery` fails on an example a reader
could not paste. Versions shown on the site come from `Cargo.toml` and
`Cargo.lock` (`catalog.json`), and releases from `CHANGELOG.md`.
Never substitute a checked-in screenshot or embed the full gallery shell on a
component page. The preview query and message bridge must select and construct
only one requested example at a time. New gallery pages must add checked-in
reference metadata so the website does not ship an examples-only component page.

`public/gallery/herogpui_web*` is compiled from this repository's own
workspace: `crates/herogpui-web` links the `herogpui-gallery` library and
builds for `wasm32-unknown-unknown`, so the embed runs the same
`gallery/src/pages/components/` the native gallery does. The build needs
`cargo +nightly` (upstream's `multithreaded` default pulls a `#![feature]`
crate) and no `RUSTFLAGS`; the full recipe is in `DEPLOYMENT.md` section 6
and the CI `wasm` job.

After rebuilding that artifact, regenerate `src/data/wasm-sections.json` and
`src/data/wasm-parity.json` with `pnpm run wasm:manifest` in the same change.
`wasm-sections.json` tells the component page which headings get a live embed;
`wasm-parity.json` pins the artifact, the glue, every example body and every
wasm build input (Rust sources, manifests, lockfile) by hash, which is the only
thing standing between a committed 19 MB binary and a page whose code block and
embed disagree. `pnpm run extract:check` fails when they have parted company,
so a Rust change without a rebuild and regeneration does not merge.
