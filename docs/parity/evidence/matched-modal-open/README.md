# Modal open (Md) — first matched upstream/port capture set

Captured 2026-09-13. One of the first matched fixture sets (plan
[section 5.1](../ui-design-plan.md), [5.3](../ui-design-plan.md)); evidence
for the `md-size-md` specimen rendered **open**, **not a completed Modal
parity verdict**. No motion samples, no dismissal paths, no sizes other than
Md.

## Target versions

- Upstream: `@heroui/react` 3.2.5 as resolved by `web/pnpm-lock.yaml`,
  rendered by the dev-only fixture runner (`docs/parity/fixtures.md`), which
  mounts `ModalOpenFixture` (`isOpen`, size `md`, no trigger). Every capture's
  strip records `fixture=modal-open`, `theme=…`, `stage=720x480`,
  `@heroui/react@3.2.5`, `pnpm-lock.yaml sha256:96fe7abea3effa8a`.
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
Protocol on macOS, `deviceScaleFactor` forced to 1 (all PNGs logical px).

- **Frame.** React Aria portals the modal to `document.body` against a
  viewport-fixed backdrop, so per `docs/parity/fixtures.md` the upstream frame
  is the **full browser viewport**: 800 × 533 (the 720×480 stage plus the
  53px evidence strip, which wraps at this width). The port's modal likewise
  centers against its whole window, so the port canvas was captured at the
  same 800 × 533; the two panels center at the same point and the frames
  compare 1:1 with no cropping.
- **Portal theme fix.** The portaled modal mounts outside the fixture page's
  theme-class wrapper, so it followed the browser profile's
  `prefers-color-scheme` (dark on this Mac) and the first light capture
  rendered a dark panel. `Emulation.setEmulatedMedia` now forces
  `prefers-color-scheme` to the requested `theme` for every capture; light
  was re-captured after the fix (dark was byte-identical before/after). The
  strip and stage-scoped components were never affected.
- Next.js dev-tools badge removed before upstream capture (dev-server chrome
  only).
- Upstream URL: `http://localhost:3100/fixtures?fixture=modal-open&theme=light|dark`
- Port URL: `http://127.0.0.1:8377/gallery/index.html?page=Modal&preview=component&section=Sizes&specimen=md-size-md&theme=light|dark&v=e2bfda29`
  served from `web/public` by `python3 -m http.server`. The `specimen` filter
  constructs only the `md-size-md` demo inside the `Sizes` section — the
  closed composition is its single "Md" trigger button, centered.

### Open-state reachability on the port (limitation)

The web bootstrap exposes `?page/&story/&preview/&section/&specimen/&theme`
only — **there is no URL parameter that opens an overlay**
(`HEROGPUI_OPEN_OVERLAYS` is a native environment variable and the
`HEROGPUI_CONTROL` file protocol is native-only; the browser has neither).
The open state is reachable only through real input, and the served WASM
canvas ignores synthetic JS events. For each theme the capture therefore:

1. loaded the page fresh (modal closed),
2. captured a probe frame and located the "Md" trigger by a PIL fill-color
   scan — 54 × 36 at (373, 242)–(426, 277), center (399, 259),
3. delivered **one real CDP `Input.dispatchMouseEvent` click** (trusted
   browser input, press + release) at that point,
4. waited 2s for the open animation to settle and captured.

No synthetic events were dispatched and the state was not fabricated; the
click and its visible effect are part of the evidence.

## Images

| File | What it shows |
|---|---|
| [upstream-light.png](upstream-light.png) / [upstream-dark.png](upstream-dark.png) | Pinned HeroUI fixture, full 800×533 viewport with evidence strip, backdrop dimming included; provenance in [upstream-light.json](upstream-light.json) / [upstream-dark.json](upstream-dark.json) |
| [port-light.png](port-light.png) / [port-dark.png](port-dark.png) | WASM gallery canvas 800×533, modal opened by one real click on the "Md" trigger; provenance in [port-light.json](port-light.json) / [port-dark.json](port-dark.json) |
| [comparison-light.png](comparison-light.png) / [comparison-dark.png](comparison-dark.png) | Upstream left, port right, 1px separator, labels; 1601×563 |
| [diff-light.png](diff-light.png) / [diff-dark.png](diff-dark.png) | Raw PIL `ImageChops.difference`, 800×533 — includes strip/trigger/backdrop-tone differences below, not pass/fail |

## Measured geometry (logical px)

Method for both sides: PIL fill/ring-color bbox scan of the captures
(threshold ±10 per-channel sum, scan window x 80–720, y 100–460). The upstream
DOM could not be used for the port side, and the upstream panel sits in a
portal, so both numbers come from the same image method.

| Metric | Upstream | Port |
|---|---|---|
| Md panel size | 448 × 106 (light), 446 × 104 (dark) | 448 × 100 (light; dark visually identical) |
| Panel center | (399, 266) = viewport center, both themes | (399, 266) — same |
| Content | Title "Size: Md", circular close trigger top-right, one-line body "Every size shares one panel style." | Same content, same order |

Panel width matches exactly (448 vs 448). The height difference has since
been diagnosed and fixed in source: the pinned sheet's `.modal__body` is
`-m-[3px] my-0 p-[3px]` (modal.css), and because `my-0` zeroes only the
vertical margins, the 3px padding there is 6px of real body height — the
upstream composition is 24 (`p-6`) + 24 (heading line) + 8 (`mt-2`) + 3 +
20 (`leading-[1.43]` line) + 3 + 24 (`p-6`) = 106, confirmed by PIL band
measurement in both themes. The port omitted that body padding and drew
100. `modal.rs` now spells the same `.mx(px(-3.))` + `.p(px(3.))` pair
AlertDialog and Drawer already carried, and
`modal_md_panel_height_matches_the_pinned_composition` pins the 106px
composition. **The port-side captures above predate that fix** — they show
the pre-fix 100px panel; a re-capture against a rebuilt artifact is future
work (the WASM artifact is synchronized separately from component source).

(Re-measurement note: with a strict panel-fill match the upstream panel is
448 × 106 in dark as well; the recorded 446 × 104 came from the scan
threshold catching the shadow edge against the dark backdrop. The 2px
light/dark split is measurement noise, not layout — modal.css has no
theme-dependent vertical value.)

## Known limitations

- **Fonts differ by design** (site Geist stack vs bundled Inter); raw diffs
  include rasterization and advance-width noise. Geometry is the comparison
  target.
- **Token source differs**: fixture = website porabuild tokens (backdrops over
  `#eaf0fb`/`#070709`), port = its ported HeroUI tokens (`#f5f5f5`/`#060607`).
  The backdrop-dimmed field differs in tone for that reason (sampled
  (94,96,100) upstream vs (123,123,123) port in light).
- **Composition differences inside the frame** (documented, not hidden): the
  upstream fixture renders the open modal only — no trigger — while the port
  frame necessarily shows the closed "Md" trigger behind the backdrop, above
  the panel; and the upstream frame retains the wrapped evidence strip per the
  fixture capture rules. Both add fixed offsets to the raw diff.
- **One click per capture** reaches the open state (see above); the click path
  itself has no separate interaction evidence here, and no dismissal
  (escape/backdrop/close trigger) was driven after the capture.
- **Motion is unproven**: single settled end-state frames only — no
  intermediate samples, duration, interruption, or reduced-motion evidence,
  which the plan requires before any motion claim.
- Only `Md` is captured; `Xs/Sm/Lg/Cover/Full` and placements remain.

**Not a completed verdict:** these are the first matched captures for the open
Md Modal. Sizing ladder, placement, dismissal, focus behavior, and motion
parity remain unproven by this set.
