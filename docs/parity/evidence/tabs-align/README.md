# Tabs alignment evidence — HeroUI v3.2.5

Captured and visually inspected on 2026-09-11. These images establish partial
alignment evidence, not full Tabs parity or pixel equality.

- [Light matrix](tabs-align-light.png) and [dark matrix](tabs-align-dark.png):
  shared WASM gallery, Alignment section, Primary/Secondary × Start/Center/End.
  General and Privacy visibly follow each alignment. Final artifact manifest
  prefix: `48e898894139`; light capture explicitly requests that version.
- [Keyboard selection](tabs-align-keyboard.png): clicking General in Primary
  Start and pressing ArrowDown selects Subscription & Billing and moves the
  indicator. This capture predates a metadata-only artifact rebuild; component
  code is identical.
- [Unmodified upstream demo](tabs-upstream-align.png): live
  [v3.2.5 Tabs documentation](https://heroui.com/en/docs/react/components/tabs#alignment),
  Secondary Start, including its adjacent panel and wrapped long label.
- `tabs-upstream-{primary,secondary}-{start,center,end}.png` and matching JSON:
  six **local CSS fixture variants** of that observed upstream demo. Only its
  root variant/alignment classes were toggled to those emitted by the tagged
  styles; this is not a React prop-control interaction test. The embedded code
  example still shows the original props. Temporary width experiments were
  removed before these captures. JSON records computed flex and text alignment.
  Short labels move while panel text stays left-aligned.

Source contract: tagged `packages/styles/src/components/tabs/tabs.styles.ts`
and `packages/styles/components/tabs.css` at commit
`5f13f6ed355bdbd5d5f69e5944685438a3591793`. Center is default; Start/End style
only the owning list's tabs. HeroGPUI applies flex/text alignment to each tab,
with Start/End following its left-to-right layout.

Focused integration tests cover both variants and orientations, omitted and
explicit alignment, disabled-tab skipping, selection callbacks, and unchanged
indicator geometry. All 27 tests in `tabs_deep` pass.

## Remaining proof and discrepancy

Upstream wraps constrained vertical labels beside a panel; HeroGPUI currently
keeps them on one line and can widen the list. Removing `whitespace_nowrap`
alone did not fix its intrinsic/min-content layout. The reference marks this
partial. The port matrix is not the same constrained layout as upstream and
does not establish wrapping parity. Primary matrix overflow chevrons also need
separate investigation. Nested lists, horizontal visual positioning, every
interactive state, motion frames and native input remain unverified here.
