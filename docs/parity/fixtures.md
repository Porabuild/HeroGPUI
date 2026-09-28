# Matched upstream fixtures

Development-only reference rendering for matched native/WASM comparison, per
[section 5.1](ui-design-plan.md) of the UI design plan: the exact pinned
HeroUI packages (`@heroui/react` 3.2.6 as resolved by this repository's
`web/pnpm-lock.yaml`) render reference compositions that mirror keyed
specimens from the native gallery, so an upstream capture and a port capture
show the same content under the same framing.

The route is private. It is not linked from any nav, sitemap, or footer, and
`web/src/app/fixtures/page.tsx` renders `notFound()` unless
`process.env.NODE_ENV === "development"` — production builds have no working
fixtures page.

## Running

```bash
pnpm --dir web dev
```

then open, for example:

```
http://localhost:3000/fixtures?fixture=button-usage&theme=light&w=640&h=360
```

Query parameters:

| Parameter | Meaning | Default |
| --- | --- | --- |
| `fixture` | Registry name | first fixture (`button-usage`) |
| `theme` | `light` or `dark` | `light` |
| `w` | Stage width in CSS px | 640 |
| `h` | Stage height in CSS px | 360 |

`w` clamps to 200–1600 and `h` to 120–1200; absent or empty falls back to the
default, and a fixture's own stage size (the open Modal) is the default for
that fixture until an explicit `w`/`h` overrides it. A non-numeric size,
an unknown fixture name, or an unknown theme renders an inline error listing
every valid value — still behind the development gate.

## Fixtures

| Fixture id | Mirrors | Port source |
| --- | --- | --- |
| `button-usage` | `btn-usage` — Button Usage row: default (Primary) variant, label "Click me", 8px radius with a squared top-left corner | `gallery/src/pages/components/buttons.rs` |
| `select-usage` | `sel-main` — Select Usage specimen at rest: label "Language", placeholder "Choose one", the fixed six-language collection, closed and unselected (the gallery seeds `select_lang: None`) | `gallery/src/pages/components/pickers.rs` |
| `modal-open` | `md-size-md` — Modal Sizes, Md, rendered open: title "Size: Md", close trigger, one-line body | `gallery/src/pages/components/overlays.rs` |

Each fixture renders inside a fixed stage that frames the composition the way
the gallery's `preview_wrapper` does: centered on the `--background` token,
nothing else. The evidence strip above the stage records the fixture name,
theme, stage size, the resolved `@heroui/react` version from
`web/package.json`, and the SHA-256 of `web/pnpm-lock.yaml` (truncated to 16
hex chars), computed at request time. Crop captures below the strip.

## Determinism contract

Every resting capture must be reproducible. The fixture page holds the line on:

- **Fixed stage size** in exact CSS pixels, `overflow: hidden`, explicit
  background. The inline background fallbacks are this site's own resolved
  token values (porabuild moon `#eaf0fb` in light, night `#070709` in dark),
  so even a stylesheet-loading race cannot change the crop.
- **Wrapper-scoped theme.** The root layout sets a `dark` class on
  `documentElement` from localStorage or the system preference; the fixture
  page re-declares the theme class on both the page wrapper and the stage, so
  token resolution follows the `theme` parameter only — never the browser
  profile capturing it.
- **Pinned font.** The stage sets `font-family: var(--font-sans, …)` — the
  token `@heroui/styles` defines in its theme layer and the token this site's
  body text already resolves to.
- **Fixed collections and explicit initial state.** The language list, labels,
  placeholder, and variant are literals mirrored from the gallery sources; the
  Select starts unselected because that is the port specimen's seeded state.
- **No live content.** No network fetches, no randomness, no moving dates, no
  timers inside the fixture compositions.

## Capture rules

- Set the browser viewport to the stage size plus the evidence strip height
  (the strip is a bordered `data-fixtures-strip` bar above the stage; measure
  it once per dev-server run and record the value). Crop captures to the stage
  rect below the strip.
- Record the device-pixel-ratio in the capture notes and compare in logical
  pixels, per the plan's acceptance policy.
- `modal-open` is the one fixture whose panel escapes the stage: React Aria
  portals the modal to `document.body` and centers it against a
  viewport-fixed backdrop, so the comparison frame for this fixture is the
  **full browser viewport** (stage size plus strip height), backdrop dimming
  of the strip included. Retain the uncropped context, as the plan requires —
  anchoring and clipping are part of the evidence.
- The port's open-overlay resting state is web-reachable without input: append
  `overlays=1` (or `overlays=true`) to the gallery deep link — e.g.
  `?page=Modal&preview=component&section=Sizes&specimen=md-size-md&overlays=1`
  — and the constructed example starts open. This is the browser spelling of
  the native `HEROGPUI_OPEN_OVERLAYS` variable; any other value is ignored. It
  lifts the one-click requirement for resting open-state captures recorded in
  [matched-modal-open](evidence/matched-modal-open/README.md) — but it is
  state construction, not interaction evidence: proving a trigger actually
  opens the overlay still requires a real input event.
- Interaction captures must use real input events (trusted browser input, e.g.
  CUA-driven). The served WASM gallery ignores synthetic JavaScript events —
  a scripted replay dispatched synthetic events at `(0, 0)` and swallowed
  chunked wheels until it was removed
  ([ui-design-progress.md](ui-design-progress.md)) — so synthetic dispatch
  cannot prove a shared wasm path either.
- Keep style-only probes separate from interaction evidence, per the plan:
  a forced data attribute or computed-style read may inspect a CSS endpoint,
  but it does not prove the component reached that state. Name probe files
  `*-probe.json` (or similar) and never present them as interaction captures.

## Provenance

Every evidence set under `docs/parity/evidence/<name>/` must record, from the
strip in each capture or its notes: the fixture name, theme, stage size, the
`@heroui/react` version, and the lockfile hash shown in the strip — alongside
the upstream source hashes and observed-behaviour table that
[select-clear](evidence/select-clear/README.md) established with its
`contract.json`. A capture whose strip values cannot be restated in the set's
contract is not usable evidence: if the lockfile hash differs between two
captures, they were not rendered from the same dependency tree.

## Adding a fixture

1. Name the matching port specimen key (`btn-usage`, `sel-main`, `md-size-md`,
   …) and read its gallery source first.
2. Mirror the composition — same labels, same variants, same collection
   content, same seeded initial state — in
   `web/src/app/fixtures/registry.tsx`, verifying every component and prop
   against the installed `@heroui/react` type declarations, not docs or memory.
3. Add the matching id, label, and any stage-size override to `FIXTURES` in
   `web/src/app/fixtures/page.tsx` (the registry is a client module, so the
   server-side strip cannot read it; the two lists must stay in step).
4. Extend this page's fixture table.
