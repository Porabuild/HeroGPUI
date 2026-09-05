# @porabuild/brand

Framework-agnostic Porabuild brand styleguide: design tokens (`tokens.css`) plus
signature devices (`brand.css`), bundled together via `index.css`. This package
is the single source of truth for brand color, type, and reusable devices. The
porabuild.com marketing site consumes it; other Porabuild projects (first: the
HeroGPUI docs site) vendor a copy.

## Consume

### Option A — npm workspace (this repo)

The root `package.json` declares `"workspaces": ["packages/*"]` and depends on
`"@porabuild/brand": "file:packages/brand"`. Import once in CSS:

```css
@import "@porabuild/brand/index.css";
```

Import `tokens.css` alone when you only need variables, or `brand.css` alone
when the host already provides the tokens.

### Option B — vendored copy (other repos, e.g. HeroGPUI docs)

Copy `packages/brand/` into the consumer (e.g. `vendor/brand/`) and import the
local `index.css`. The vendoring sync script lives in the consumer: it copies
the three CSS files plus this README and fails the build when the copy drifts
from this package, so re-sync after every brand release.

## Tokens (`tokens.css`)

All custom properties live on `:root` and are prefixed `--pb-`.

| Token                      | Value                                                                   |
| -------------------------- | ----------------------------------------------------------------------- |
| `--pb-night`               | `#070709`                                                               |
| `--pb-tile`                | `#0e0e14`                                                               |
| `--pb-tile-2`              | `#14141c`                                                               |
| `--pb-moon`                | `#eaf0fb`                                                               |
| `--pb-dim`                 | `#9ba6be`                                                               |
| `--pb-accent`              | `#8b7bff`                                                               |
| `--pb-accent-soft`         | `#b6acff`                                                               |
| `--pb-ice`                 | `#5ee6e0`                                                               |
| `--pb-line`                | `rgba(234, 240, 251, 0.08)`                                             |
| `--pb-line-strong`         | `rgba(234, 240, 251, 0.14)`                                             |
| `--pb-accent-glow`         | `rgba(139, 123, 255, 0.65)`                                             |
| `--pb-accent-tint`         | `rgba(139, 123, 255, 0.035)` (hover wash)                               |
| `--pb-site-max`            | `1920px`                                                                |
| `--pb-font-sans`           | `var(--font-geist), Inter, system-ui, sans-serif`                       |
| `--pb-font-mono`           | `var(--font-mono), ui-monospace, "SF Mono", Menlo, Consolas, monospace` |
| `--pb-mono-size-xs`        | `9px`                                                                   |
| `--pb-mono-size-sm`        | `10px`                                                                  |
| `--pb-mono-size-md`        | `11px`                                                                  |
| `--pb-mono-size-lg`        | `12px`                                                                  |
| `--pb-mono-tracking`       | `0.06em`                                                                |
| `--pb-mono-tracking-wide`  | `0.08em`                                                                |
| `--pb-display-tracking`    | `-0.052em`                                                              |
| `--pb-heading-tracking`    | `-0.045em`                                                              |
| `--pb-subheading-tracking` | `-0.035em`                                                              |
| `--pb-display-weight`      | `650`                                                                   |
| `--pb-heading-weight`      | `620`                                                                   |
| `--pb-subheading-weight`   | `600`                                                                   |
| `--pb-ease-out`            | `cubic-bezier(0.16, 1, 0.3, 1)`                                         |
| `--pb-radius-sm`           | `6px`                                                                   |
| `--pb-radius-md`           | `8px`                                                                   |
| `--pb-radius-lg`           | `18px`                                                                  |

## Devices (`brand.css`)

All classes consume the tokens above and are prefixed `pb-`. Keep them small
and composable; page-section layout (hero, products, founder) does not belong
here.

- `.pb-eyebrow` — dim mono 11px section label row (flex, gap 10px).
- `.pb-kicker` — same row in accent; use for kickers on tinted backgrounds.
- `.pb-live-dot` — 7px violet status dot with glow; pair with an eyebrow.
- `.pb-brand-lockup` — wordmark wrapper (Geist, -0.05em tracking, nowrap).
- `.pb-brand-lockup strong` / `.pb-brand-lockup-word` — 700 / 600 lockup weights.
- `.pb-brand-dot` — 0.25em inline violet lockup dot with 0.45em glow.
- `.pb-section-index` — dim mono 10px section number, 0.08em tracking.
- `.pb-mark` — 28px x 1px accent rule above principle headings.
- `.pb-hairline` — 1px divider using `--pb-line`.
- `.pb-hairline-strong` — 1px divider using `--pb-line-strong`.
- `.pb-window-bar` — 45px `1fr auto 1fr` product-window title bar on tile-2.
- `.pb-chip` — pill shell (line-strong border, 99px radius, mono 9px).
- `.pb-chip--ice` — ice-text modifier for coming-soon chips.
- `.pb-link-underline` — underline product link; hover widens to accent-soft.
- `.pb-round-link` — 54px circle link; hover inverts to moon with 3px lift.
- `.pb-cta` — solid moon primary button (mono 11px, 600, 16px 20px).
- `.pb-cta--ghost` — transparent ghost variant; hover adds accent ring.
- `.pb-root :focus-visible` — 2px accent outline, 4px offset, 6px radius.
- `.pb-root ::selection` — night text on accent-soft.
- `prefers-reduced-motion` — collapses animations/transitions under `.pb-root`.

Scope interactive pages with `class="pb-root"` to enable the focus ring,
selection, and reduced-motion behavior.

## Fonts

The host app supplies `--font-geist` and `--font-mono`. On Next.js, expose them
via `next/font` (Geist + Geist Mono with the `variable` option). Otherwise
self-host Geist/Geist Mono (or Inter + a mono fallback) and define the same two
variables before importing this package.

## Rules

- Accent is violet (`#8b7bff`), never blue. Ice (`#5ee6e0`) is reserved for
  coming-soon / recording affordances, not primary actions.
- The brand is dark-only. For light-mode consumers, invert night/moon (and
  tile surfaces against moon) while keeping the violet/ice hues unchanged.

## CHANGELOG

### 0.1.0

- Initial extraction from porabuild.com `app/globals.css`: `--pb-` tokens and
  `pb-` devices (`eyebrow`, `kicker`, `live-dot`, `brand-lockup`, `brand-dot`,
  `section-index`, `mark`, `hairline`, `window-bar`, `chip`, `link-underline`,
  `round-link`, `cta`, focus/selection, reduced motion).
