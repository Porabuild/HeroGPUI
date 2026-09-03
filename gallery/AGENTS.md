# Gallery instructions

These rules apply under `gallery` in addition to the root guide.

Read [gallery and visual verification](../docs/agents/gallery.md) before running
the app, driving input, or capturing images. Read
[parity and audits](../docs/agents/parity.md) when changing examples or checked-in
reference metadata.

- Component demos live in `src/pages/components.rs`; shared documentation
  rendering and route/category registration live in `src/pages/mod.rs`.
- Keep each documentation example in title, optional description, then live
  component order. Explanatory gallery prose belongs outside the bordered
  preview; only text owned by the demonstrated component (labels, helper text,
  values, and composed content) belongs inside it.
- Every interactive controlled prop in a demo must store and feed back its
  callback value. Otherwise use the matching `default_*` seed.
- Repeated components need unique ids, even when a helper constructs them.
- Reference metadata must name real Rust owners/methods and pin source links to
  HeroUI v3.2.4.
- Do not hard-code the current release page or route totals; derive them from
  workspace metadata and the page registry.
- Use `.shots/rebuild.ps1` before smoke/capture so a locked executable cannot
  leave you testing stale code.

Run the relevant example/demo/reference audit, focused gallery tests, route
smoke, and a focused visual/behavior drive appropriate to the change.
