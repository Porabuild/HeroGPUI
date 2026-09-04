# Design notes

Decisions that are easy to undo by accident. Read before restyling anything
listed here.

## Code blocks — no accent bar, no shouty language chip

**Rejected:** `border-l-2 border-l-accent` on Rust blocks, plus a language
eyebrow set in the accent colour. Both were in `src/components/ui/code-block.tsx`
and both are out.

A coloured left bar is the convention for an *admonition* — a note, a warning,
a callout. Borrowing it for a code block asserts an importance the block does
not have, and it is the single most common tell of machine-generated UI. The
accent-coloured language label has the same fault: syntax highlighting already
tells a reader this is Rust, so the chip repeats a known fact loudly.

**The direction instead: the block is an editor pane.**

That is not decoration — it is the subject's own world. GPUI is the framework
behind the Zed editor, this library is written to build editors and desktop
tools, and the code genuinely lives in a file. So:

- A **header strip** across the top of the block: the filename on the left
  (`main.rs`, `Cargo.toml`), set in the mono face at small size in `text-muted`,
  never in the accent colour. The copy control sits at the right of that strip.
  When there is no filename, the language name takes its place, in the same
  muted treatment.
- A **line-number gutter**, right-aligned, `text-muted` at reduced opacity,
  separated from the code by a hairline in `border-separator` rather than a
  block of tinted background. The numbers are useful — the examples are read,
  referred to and collapsed — so the gutter earns its place.
- **Hairline border and surface only.** `border-separator` plus the surface
  token carry the container. No left bar, no glow, no gradient, no shadow.
- **One accent use per block, at most**, and only on interaction: the copy
  control's hover and focus-visible state. Nothing accent-coloured at rest.

Distinguish languages by the header label alone. A Rust block and a TOML block
should differ in what they say, not in what colour they are.

## Component example cards — the target shape

Each example is one bordered card: **the live component above, its Rust below**,
in the same card, with a collapse control at the foot of the code.

The pane above is **HeroGPUI itself, compiled to WebAssembly and running** —
that one example, rendered by GPUI. Not a screenshot of the gallery, and not a
recreation in another framework. A reader sees the real component and the exact
Rust that produced it, together.

A whole-window gallery screenshot parked at the top of the page with code-only
cards below is **not** the destination. It is a placeholder that stands while
the WebAssembly work lands, and it should be replaced, not built upon.

### The shape, confirmed against a working implementation

`gpui_web` attaches one canvas to `document.body` and supports one top-level
GPUI window per application, so live examples cannot each be a canvas in the
host document.

`longbridge/gpui-component` solves this in production, and their docs pages
were inspected directly rather than guessed at. Each component page embeds:

```html
<iframe src="/gallery/?story=Accordion" title allow>
```

The essentials, measured on their Accordion page:

- **One shared wasm gallery**, embedded per page with a `?story=` query
  parameter. Not a separate build per component.
- **One iframe per page, not per example.** That single frame carries the
  component's whole story — Default, Custom style, and so on, each labelled
  inside it. This is the detail worth copying: one GPUI instance per page
  rather than six, which is what makes a multi-megabyte bundle affordable
  across a large component catalogue. The module is cached across navigations.
- Zero canvases and zero wasm requests in the host document.
- The frame is presented as an application window — title bar, traffic
  lights — and badged so a reader knows it is real Rust running, not a video
  or a picture.
- The page's code blocks sit alongside the frame, nine of them on that page.

So: one iframe per component page, framed as a window, with the Rust for each
example below it. An `IntersectionObserver` still earns its place — boot the
frame when it scrolls into view rather than on page load.

The embed mode this needs in the gallery: render one named story, no
navigation chrome, sized to the frame, theme from the URL. `HEROGPUI_PAGE` is
an environment variable and does not survive into wasm, so page selection
comes from the query string — the same choice Longbridge made.

Until the wasm artifact renders, the card shows code only. Do not fill an
empty preview with a placeholder graphic, and do not treat the interim as
finished.

## Screenshots are screenshots

Captures in `public/shots/` are of the real desktop application, including its
window chrome. Frame them so a reader understands they are looking at a native
application, and never present one in a way that implies the browser is
rendering it.

## Porabuild brand layer over HeroUI

This site ships under the Porabuild umbrella at `porabuild.com/herogpui`,
alongside `porabuild.com/poratake`. It must look like a Porabuild property.

Two systems, and they do not conflict because they govern different things:

- **HeroUI supplies the component design language** — the shapes, control
  heights, corner radii, field anatomy, interaction states, the whole
  structural vocabulary. It stays. It is also, appropriately, the design
  system this library ports.
- **Porabuild supplies the brand layer** — palette, typography and a small set
  of signature devices. HeroUI v3 is built for exactly this: override the base
  tokens and every derived value follows.

The canonical source is `E:\work\porabuild\packages\brand`
(`@porabuild/brand`: `tokens.css` plus the `pb-*` devices in `brand.css`).
Read it rather than trusting this summary.

### Brand package

This site is built on Vercel from its own tree, so it cannot depend on the
sibling repo. It vendors a copy at `web/src/styles/porabuild/`, refreshed by
`pnpm run brand:sync` and checked by `pnpm run brand:check` (the check passes
vacuously where the sibling repo is absent, so CI without it stays green).
Never hand-edit the vendored copy — change the package, then re-sync.
`globals.css` imports the vendored `index.css` into cascade layer `porabuild`,
and that import MUST stay above the framework import: layer order follows
first declaration, so this declares `porabuild` before Tailwind's `base` and
the site's OKLCH token conversions keep winning where both define a value
(an unlayered import would beat the layered ones outright).

### Palette

| Porabuild | Value | Maps to |
|---|---|---|
| `--night` | `#070709` | page background |
| `--tile` | `#0e0e14` | surface |
| `--tile-2` | `#14141c` | secondary surface |
| `--moon` | `#eaf0fb` | foreground |
| `--dim` | `#9ba6be` | muted foreground |
| `--accent` | `#8b7bff` | accent (violet) |
| `--accent-soft` | `#b6acff` | accent hover / soft |
| `--ice` | `#5ee6e0` | secondary accent, used sparingly |
| `--line` | `rgba(234,240,251,.08)` | separator |
| `--line-strong` | `rgba(234,240,251,.14)` | border |

The accent is **violet**, not blue. Any blue on this site is a leftover of
HeroUI's default theme and is wrong.

### Typography

**Geist** for display and body, **Geist Mono** for the utility face — not Inter
and JetBrains Mono, which are the scaffold's defaults.

The mono face is a real part of the brand, not just for code: eyebrows, section
indices, nav links, footer, chips and captions all use it at 9–12px, weight
500, `letter-spacing: .06em`–`.08em`, frequently uppercase. Headings run tight:
`letter-spacing` `-0.045em` to `-0.052em`, weight 560–650 rather than 700.

### Signature devices

- **The dot.** A violet dot with a soft glow
  (`box-shadow: 0 0 .45em rgba(139,123,255,.65)`), used in the brand lockup and
  as a live indicator. It is the single most recognisable Porabuild mark.
- **Hairline rules** at `--line` doing structural work — section dividers, cell
  borders, the underline on a link-button.
- **Short accent rules**: a 28×1px accent bar marking a section.
- **Window chrome** for product captures: `.window-bar` with its three dots and
  a mono label. Our GPUI screenshots are captures of a real desktop window, so
  this device fits them exactly rather than being decoration.

### Light mode

The Porabuild brand is dark-only. This is documentation, so it keeps both
themes: derive light from the same hues rather than inventing a second palette,
and keep the violet accent identical in both.

## Never comment on release status

The site does not say "Released", "Now available", "Coming soon", "In
development", "Unreleased", or anything else about where the project sits in a
release cycle. It simply documents the library as it is.

The navbar chip carries the **version** (`v0.1.0`) and nothing more. A version
number is a fact a reader can use; a status badge is an announcement, and it
dates the moment it ships.

## HeroUI v2 must leave no trace — with one deliberate exception

HeroGPUI implements the current HeroUI design system. The previous major
version had a different vocabulary, and anything carrying it is a defect: it
teaches a reader an API that does not exist here, and it invites a future
change to drift back toward it.

**Delete on sight**, anywhere in the repository or the site:

- Component names that no longer exist: `Navbar`, `Image`, `User`, `Spacer`,
  `Code`, `Snippet`, `AvatarGroup`, `Divider`.
- The old variant vocabulary: `solid`, `bordered`, `light`, `flat`, `faded`,
  `shadow` as button variants. The current set is `Primary`, `Secondary`,
  `Tertiary`, `Outline`, `Ghost`, `Danger`, `DangerSoft`.
- `primary` and `secondary` as **colors**. Colors are `Default`, `Accent`,
  `Success`, `Warning`, `Danger`.
- `content1`–`content4` surface tokens, numbered 50–900 colour scales, and the
  per-component `radius` prop.
- Props like `isStriped`, `isBordered`, `isPressable`, `isHoverable`,
  `isBlurred`.
- **Screenshots of the old gallery.** Five `-dark-v3.png` captures shipped in
  the original port commit and still showed the old sidebar and variant set
  years later. `scripts/extract-catalog.mjs` now refuses any dark capture whose
  geometry disagrees with its light pair, which is what caught them. An image
  is as much an API claim as a code sample.

**The exception, which must survive:** the places that name the old version in
order to rule it out. `llms.txt`, the root `AGENTS.md`, the repository-guide
page and the `.shots/extra_audit.py` backward diff all list the retired names
precisely so an agent working from stale training data does not reintroduce
them. That backward audit is the thing that caught `Card::is_pressable`,
`ProgressBar::is_striped`, `RadioGroup::size` and the `radius` prop surviving
four earlier reviews.

Stripping those guardrails would make regression *more* likely, not less. Keep
them, keep them accurate, and keep them out of marketing copy — they belong in
the agent-facing documentation, not on the landing page.

## Re-capturing a screenshot needs the image cache cleared

Two traps caught this once and will again.

**`capture2.ps1` does not name dark output differently.** It writes
`<page>-v3.png` regardless of `-Theme`, so running it with `-Theme` against
the default `-OutDir` silently overwrites the *light* capture with a dark one.
Capture to a scratch `-OutDir`, then copy each file into `.shots/` under its
`-dark-v3.png` name.

**Next.js caches optimised images across builds.** `.next/cache/images` is
keyed by source URL, and the URL does not change when the file behind it does.
A rebuild alone keeps serving the previous image — which is how a replaced
screenshot appeared not to have been replaced at all. After swapping any file
in `public/shots/`, delete `.next/cache/images` before rebuilding.

The full sequence: capture to a scratch directory, verify the geometry matches
the light pair, copy into `.shots/`, run `node scripts/copy-shots.mjs` and
`node scripts/extract-catalog.mjs`, delete `.next/cache/images`, rebuild.
