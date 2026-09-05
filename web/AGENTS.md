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
Keep `gallery/src/pages/reference_metadata.rs`,
`gallery/src/pages/components.rs`, `web/src/data/reference.json`,
`web/src/data/rust-examples.json`, and the public `llms.txt` description aligned;
do not hand-edit generated JSON.

Example presentation follows the Rust gallery source: section title, optional
description, then the bordered example. Keep explanatory prose outside the
example surface on both sites; component-owned labels, helper text, values, and
composed content remain inside the demonstrated component.

Keep each component on one page: `Usage` (one lazy GPUI/WASM instance that
switches among every generated example, description, and matching code),
compact `Anatomy`, `Customization`, `API reference` (Props, Parts and slots,
then States), and `Related components`.
Never substitute a checked-in screenshot or embed the full gallery shell on a
component page. The preview query and message bridge must select and construct
only one requested example at a time. New gallery pages must add checked-in
reference metadata so the website does not ship an examples-only component page.

When regenerating `public/gallery/herogpui_web*`, also regenerate
`src/data/wasm-sections.json` and `src/data/wasm-parity.json` from that build's
`gallery/src/pages/components.rs` with
`node scripts/extract-wasm-sections.mjs --source <components.rs>`. This keeps
the live selector from advertising examples absent from the wasm artifact,
pins the native source and artifact hash, requires descriptions to match, and
rejects newly introduced native/WASM code drift. Use `--accept-drift` only for
a reviewed GPUI-version adaptation that cannot share the native source.
Run `node scripts/lift-wasm-descriptions.mjs <components.rs>` before the build;
it idempotently moves legacy static prose out of the live component canvas.
