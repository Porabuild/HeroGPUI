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

Keep each component on one page: all generated examples come first, followed by
`Component documentation` with Props, Anatomy, States, and Styling. New gallery
pages must add checked-in reference metadata so the website does not ship an
examples-only component page.
