# Button Usage — first matched upstream/port capture set

Captured 2026-09-13. This is one of the first matched fixture sets (plan
[section 5.1](../ui-design-plan.md), [5.3](../ui-design-plan.md)); it is
resting-state framing evidence for the `btn-usage` specimen, **not a completed
Button parity verdict**. No interaction was driven on either side.

## Target versions

- Upstream: `@heroui/react` 3.2.5 as resolved by `web/pnpm-lock.yaml`,
  rendered by the dev-only fixture runner (`docs/parity/fixtures.md`). The
  evidence strip in every capture's provenance records
  `fixture=button-usage`, `theme=…`, `stage=640x360`,
  `@heroui/react@3.2.5`, `pnpm-lock.yaml sha256:96fe7abea3effa8a`.
- Port: checked-in WASM gallery artifact
  `web/public/gallery/herogpui_web_bg.wasm`, sha256
  `e2bfda29ccfbeadda03924d2733d57c7e992329baf2f84b2c696afb4952a3a5d`
  (prefix `e2bfda29`), glue `77c23f63…`; both verified live against
  `web/src/data/wasm-parity.json` and the served files before and after the
  capture run. An earlier pass in the same session captured the identical
  frames against the previous artifact `1712dd01…` before a concurrent
  gallery rebuild advanced the manifest; the artifact hash was stable across
  every run retained here, and the rendered pixels were byte-identical
  between the two artifacts.

## Capture setup

Both sides captured with headless Chrome 153.0.8010.37 driven over the Chrome
DevTools Protocol on macOS, `deviceScaleFactor` forced to 1, so every PNG is
in logical pixels and compares 1:1. `prefers-color-scheme` was emulated to the
requested theme (the fixture page re-declares the theme class on its wrapper,
but forcing the profile preference keeps provenance symmetric).

- Upstream viewport 800×413 — wide enough that Tailwind's `md:` breakpoint
  (48rem) applies (see limitations); stage 640×360 clipped exactly to the
  `[data-fixtures-stage]` rect. At this width the evidence strip wraps to two
  lines (53px), which is outside the stage clip.
- Next.js's dev-tools badge (`nextjs-portal`) was removed from the DOM before
  every upstream capture; it is dev-server chrome that otherwise lands in the
  stage crop's bottom-left corner. No fixture DOM, props, or CSS were touched.
- Upstream URL: `http://localhost:3100/fixtures?fixture=button-usage&theme=light|dark`
- Port URL: `http://127.0.0.1:8377/gallery/index.html?page=Button&preview=component&section=Usage&theme=light|dark&v=e2bfda29`
  served from `web/public` by `python3 -m http.server`. The preview constructs
  only the `Usage` section, whose whole body is the keyed `btn-usage`
  specimen, centered by `preview_wrapper` exactly like the fixture stage.
- Resting captures only; no interaction on either side.

## Images

| File | What it shows |
|---|---|
| [upstream-light.png](upstream-light.png) / [upstream-dark.png](upstream-dark.png) | Pinned HeroUI fixture stage crop, 640×360; provenance in [upstream-light.json](upstream-light.json) / [upstream-dark.json](upstream-dark.json) |
| [port-light.png](port-light.png) / [port-dark.png](port-dark.png) | WASM gallery canvas, 640×360; provenance in [port-light.json](port-light.json) / [port-dark.json](port-dark.json) |
| [comparison-light.png](comparison-light.png) / [comparison-dark.png](comparison-dark.png) | Upstream left, port right, 1px separator, labels; 1281×390 |
| [diff-light.png](diff-light.png) / [diff-dark.png](diff-dark.png) | Raw PIL `ImageChops.difference`, 640×360 — no blur, no threshold, not pass/fail |

## Measured geometry (logical px)

| Metric | Upstream — method: DOM `getBoundingClientRect` + computed style | Port — method: PIL bbox scan of the capture, threshold vs background; radius from gallery source |
|---|---|---|
| Button box | 89.08 × 36, centered at stage center (320, 180) | 90 × 36, centered at (319, 179) — both themes |
| Radius | computed `border-radius: 0px 8px 8px` (top-left squared) | source: `radius(px(8.))` + `sx(rounded_tl(0))` (`gallery/src/pages/components/buttons.rs`); capture corner scan confirms a square top-left |
| Padding / type | `0 16px`, 14px label | `padding_x` 16, `text` 14 (`button_metrics`) |
| Label | "Click me" | "Click me" |

The 0.92px width difference is the label's advance width in Geist (site font)
vs Inter (port's bundled font): the box is content-sized, so text metrics move
the edges. Height, position, and the squared-top-left asymmetry — the
specimen's point — match exactly.

## Known limitations

- **Fonts differ by design.** Upstream resolves the site's `--font-sans`
  (Geist stack); the port bundles Inter. Raw pixel diffs include font
  rasterization and advance-width noise; geometry (bounds, radii, gaps) is the
  meaningful comparison, per the plan's acceptance policy.
- **Token source differs.** The fixture stage resolves the *website's*
  porabuild theme tokens (primary accent `#8B7BFF`; backgrounds `#eaf0fb`
  light, `#070709` dark), while the port resolves its ported HeroUI token set
  (primary `#0485F7`; backgrounds `#f5f5f5` light, `#060607` dark). The global
  accent/background difference in the diff images is a theme-source
  difference, not a component defect. A same-token comparison run is open
  work.
- **Breakpoint choice.** Upstream `.button` is `h-10 md:h-9`; these captures
  use a ≥768px viewport so the desktop 36px height applies — the value the
  port deliberately matches ("a desktop app is past every breakpoint",
  `crates/herogpui-core/src/enums.rs`). A <768px viewport would render the
  upstream button 40px tall.
- **Dot grid.** The port's `preview_wrapper` draws a faint dot grid; the
  fixture stage is flat. Adds sparse speckle to the diffs.
- The diff images additionally include the accent/background token difference
  above; they characterize, they do not judge.

**Not a completed verdict:** these are the first matched captures for the
Button Usage specimen. Variants, sizes, states, motion, and interaction parity
remain unproven by this set.
