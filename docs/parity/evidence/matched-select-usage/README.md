# Select Usage — first matched upstream/port capture set

Captured 2026-09-13. One of the first matched fixture sets (plan
[section 5.1](../ui-design-plan.md), [5.3](../ui-design-plan.md)); resting
framing evidence for the `sel-main` specimen (closed, unselected), **not a
completed Select parity verdict**. No interaction was driven on either side;
the open list, selection, and clear paths remain separate work (see also
[select-clear](../select-clear/README.md) for the clear-target evidence).

## Target versions

- Upstream: `@heroui/react` 3.2.5 as resolved by `web/pnpm-lock.yaml`,
  rendered by the dev-only fixture runner (`docs/parity/fixtures.md`).
  Every capture's strip records `fixture=select-usage`, `theme=…`,
  `stage=640x360`, `@heroui/react@3.2.5`,
  `pnpm-lock.yaml sha256:96fe7abea3effa8a`.
- Port: checked-in WASM gallery artifact
  `web/public/gallery/herogpui_web_bg.wasm`, sha256
  `e2bfda29ccfbeadda03924d2733d57c7e992329baf2f84b2c696afb4952a3a5d`
  (prefix `e2bfda29`), glue `77c23f63…`; verified live against
  `web/src/data/wasm-parity.json` and the served files before and after the
  capture run. (An earlier pass captured identical frames against the previous
  artifact `1712dd01…`; a concurrent gallery rebuild advanced the manifest
  between passes and the final set pins `e2bfda29`.)

## Capture setup

Both sides captured with headless Chrome 153.0.8010.37 over the Chrome DevTools
Protocol on macOS, `deviceScaleFactor` forced to 1 (all PNGs logical px), and
`prefers-color-scheme` emulated to the requested theme.

- Upstream viewport 800×413 (past Tailwind's `md:` breakpoint; the Select has
  no `md:`-gated sizes but the shared frame is kept uniform with
  [matched-button-usage](../matched-button-usage/README.md)); stage 640×360
  clipped to the `[data-fixtures-stage]` rect. Next.js dev-tools badge
  removed before capture (dev-server chrome only).
- Upstream URL: `http://localhost:3100/fixtures?fixture=select-usage&theme=light|dark`
- Port URL: `http://127.0.0.1:8377/gallery/index.html?page=Select&preview=component&section=Usage&theme=light|dark&v=e2bfda29`
  served from `web/public` by `python3 -m http.server`. The preview constructs
  only the `Usage` section body — the keyed `sel-main` specimen, seeded
  `select_lang: None`, so both sides show the placeholder "Choose one" with
  the list closed.
- Resting captures only.

## Images

| File | What it shows |
|---|---|
| [upstream-light.png](upstream-light.png) / [upstream-dark.png](upstream-dark.png) | Pinned HeroUI fixture stage crop, 640×360; provenance in [upstream-light.json](upstream-light.json) / [upstream-dark.json](upstream-dark.json) |
| [port-light.png](port-light.png) / [port-dark.png](port-dark.png) | WASM gallery canvas, 640×360; provenance in [port-light.json](port-light.json) / [port-dark.json](port-dark.json) |
| [comparison-light.png](comparison-light.png) / [comparison-dark.png](comparison-dark.png) | Upstream left, port right, 1px separator, labels; 1281×390 |
| [diff-light.png](diff-light.png) / [diff-dark.png](diff-dark.png) | Raw PIL `ImageChops.difference`, 640×360 — includes font and token-source noise, not pass/fail |

## Measured geometry (logical px)

| Metric | Upstream — method: DOM `getBoundingClientRect` + computed style | Port — method: PIL bbox scan of the capture (fill-color match); radius from code path |
|---|---|---|
| Field column | `.w-64` wrapper 256 wide, 60 tall, at stage-relative (192, 150) | `field_col`/`DEMO_FIELD_W` = 256 (`gallery/src/pages/components/mod.rs`); trigger scan starts at x=192 |
| Trigger box | `button.select__trigger` 256 × 36 at (192, 174), both themes | 256 × 36 at (192, 174), both themes — same box, same position |
| Trigger radius | computed `12px` (`min-h-9` + `rounded-field`); corner scan of the capture agrees | field chrome radius via `util::field_radius(cx)` (`--field-radius` token); the specimen's `radius(8px)` belongs to the detached panel, not the trigger — corner scan consistent with the large chrome radius |
| Label | "Language" 65.08 × 20, 14px, above trigger with 4px gap (column gap-3) | "Language" 14px (capture); same stacked layout |
| Value / placeholder | "Choose one" 216 × 20, inset 12px, muted color | "Choose one" (capture); placeholder seeded by `select_lang: None` |
| Indicator | 16 × 16 chevron, 8px from the trigger's inline-end edge | chevron visible at the matching position (capture) |

## Known limitations

- **Fonts differ by design** (site Geist stack vs bundled Inter); raw diffs
  include rasterization and advance-width noise. Geometry is the comparison
  target.
- **Token source differs**: the fixture resolves the website's porabuild
  tokens (backgrounds `#eaf0fb`/`#070709`; field fills to match), the port its
  ported HeroUI tokens (`#f5f5f5`/`#060607`; near-white/near-black trigger
  fills). The global color difference in the diffs is theme-source, not
  component, behavior.
- **Dot grid** on the port preview wrapper; fixture stage is flat.
- Resting only: the open popup, flip/scroll behavior, selection, keyboard
  paths, and the clear interaction are not covered here.

**Not a completed verdict:** these are the first matched captures for the
Select Usage specimen at rest. Open-state, selection, and state-matrix parity
remain unproven by this set.
