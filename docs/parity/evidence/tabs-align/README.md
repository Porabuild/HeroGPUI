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

## Constrained vertical labels

The pinned stylesheet uses normal whitespace for vertical tab labels and a
fixed `h-8` (32px) tab box, with no truncate, nowrap or overflow utility.
HeroGPUI matches that box. A long label wraps inside the available tab
column; overflow lines paint past the pill and stay pointer-inert in the
port. The vertical scroll axis and chevron hit targets remain stable.

The regression `tabs_vertical_labels_wrap_inside_the_pinned_fixed_height`
in `crates/herogpui-components/tests/tabs_deep.rs` measures the selected
tab at 320px root width and requires wrapped labels inside the pinned 32px
height with a released column width.
`tabs_vertical_overflow_chevrons_scroll_the_list` continues to pass,
confirming that the cross-axis constraint does not move the vertical
chevrons or change the 80%-of-viewport scroll step. A matching constrained
vertical specimen is rendered in the gallery's **Wrapping Labels** section.

`.tabs__tab` is Implemented in reference metadata. Nested lists, every
interactive state, motion frames and native input remain unverified here.
