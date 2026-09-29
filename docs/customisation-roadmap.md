# Customisation roadmap

Status: **Phases 0–6 are closed for 0.13.** Written as a proposal against
`b72f285` on 2026-09-10; most of it landed in 0.9.0–0.10.1, and the 0.13
close-out (2026-09-29) implemented the remaining gaps and decided the rest.
[Status of Phases 0–6](#status-of-phases-06-013-close-out) is the item-by-item
matrix; the phase sections below keep the original plan and its reasoning,
which the matrix cites. Repository policy lives in the
[parity guide](agents/parity.md) (including the size/radius extension
exception this plan asked for), not here.

Authority: [workflow](agents/workflow.md), [component guide](agents/components.md),
[parity policy](agents/parity.md), and the root [AGENTS.md](../AGENTS.md).
Upstream evidence comes from the checked-in HeroUI **v3.2.4** bundle/CSS and
the unpacked **gpui-pre 0.3.5** registry sources. Recheck source symbols when
starting each phase; line numbers are not durable contracts.

## Status of Phases 0–6 (0.13 close-out)

Cross-checked against the tree on 2026-09-29. "Done" gives the first release
that shipped the item (from `git tag --contains` on the commit that
introduced it); "0.13" is this close-out; "declined" gives the policy or
evidence that decided it. Every new builder is a HeroGPUI extension recorded
in `.shots/extra_audit.py`, kept out of `reference_metadata`, documented in
`llms.txt` and shown in a gallery section.

| Phase | Item | Status |
|---|---|---|
| 0.0 | Owner-scoped reasons for the existing extension surface | Done 0.9.0 — `extra_audit.py` reports 0 unexplained names |
| 0.1 | `sx_padding` / `sx_radius` (pixels only, per edge/corner) | Done 0.9.0 (`util.rs`, with `sx_pixel_size`) |
| 0.1 | Part-scoped `sx` ownership inventory with scoped scanner fixtures | Done 0.9.0 (`tests/sx_ownership.rs`) |
| 0.1 | The inventory's 18 pending parts | **0.13**: Accordion's Surface triggers wired (their fade rested on the stock surface over an `sx` card); the other 17 ruled *independent* — the root `sx` refines a wrapper they do not paint over (own control fill, field chrome, transparent rest, detached panel), principle 2 forbids forwarding it, and each has a named hover seam. An independent part that starts reading the extractor now fails the test |
| 0.2 | `tabs_hover_opacity`, `tooltip_cooldown_ms`, `long_press_ms`, `hover_fade_ms`; builder and document keys for `tooltip_delay_ms` / `tooltip_close_delay_ms` | Done 0.9.0 (layout field, `ThemeBuilder`, `ThemeDocument`, consumer) |
| 0.2 | Tests that a themed token changes the consumer's behaviour | **0.13** (`tests/theme_tokens.rs`: delays, cooldown, long press, fade duration, Tabs wash); document keys → layout stay in `herogpui-theme --features serde` |
| 0.2 | `overlay_zoom_offset` | Declined — `ZoomBox::panel` takes resting padding, not a motion amplitude (§0.2); padding is Phase 3's seam |
| 0.3 | Crate-internal `FieldBox` | Done 0.9.0 |
| 1 | `height` / `padding_x` / `is_bare` on NumberField, Select, ComboBox, Autocomplete, DateField, TimeField, ColorField, InputGroup, SearchField | Done 0.9.0 |
| 1 | Row geometry (`row_padding_x` / `row_padding_y`) on Select, ListBox, ComboBox, Autocomplete; `Select::padding_y` | Done 0.9.0; `padding_y` 0.10.0 |
| 1 | ColorField `sx` on both render paths | **0.13** (`ColorField::sx`) |
| 2 | `hover_bg` on Button, ToggleButton, CloseButton (Button's endpoint contract) | Done 0.9.0 |
| 2 | Pagination, TagGroup tag/remove, Toast close, Dropdown/Menu rows | Done 0.9.0 |
| 2 | Row hovers on Select, ComboBox, Autocomplete, ListBox; TimeField stepper, InputOTP slot, Input clear, DateRangePicker trigger | Done 0.9.0 |
| 2 | Select trigger, Autocomplete trigger and clear button, NumberField group, InputGroup | **0.13** (`trigger_hover_bg`, `clear_hover_bg`, `group_hover_bg`); focus, invalid, disabled and bare chrome keep precedence |
| 2 | Calendar/RangeCalendar day, nav, year; Accordion header; Switch and Checkbox (Tween paths) | Done 0.9.0 |
| 2 | Table `row_hover_bg` for unselected interactive rows | Done 0.9.0 |
| 2 | Table selected-row hover | Declined — in the pinned `table.css` the `[data-selected]` cell fill follows the `:hover` rule at equal specificity, so a selected row has no hover state to recolour |
| 2 | Independent Pagination `pressed_bg` | Declined — hover and pressed share one upstream value; §2 keeps the pair coupled and the override feeds both |
| 2 | Per-instance hover-fade duration builders | Declined — no consumer; the theme's `hover_fade_ms` covers it (§ principles) |
| 2 | Tabs opacity token, `list_bg`, indicator parts, `hover_fill` | Done 0.9.0–0.10.1 |
| 2 | Link underline/icon seam | Declined — no distinct consumer requirement (§2's own condition) |
| 3 | `CheckboxSize`, `RadioSize`, `TabsSize` (with `SliderSize`) | Done 0.9.0, under the parity guide's size exception |
| 3 | Panel padding on Popover, Toast, Select, Dropdown (resting and zoom geometry) | Done 0.9.0 |
| 3 | Panel padding on the other overlays | Declined until a consumer asks — §3 adds the seam "to a concrete overlay when needed" |
| 4 | Per-corner `sx` reconciliation for child and animated shapes | Done 0.9.0 (`util::fill_unspecified_corners`) |
| 4 | Per-component `radius` builders | Done 0.9.0 (Tabs 0.10.0), under the parity guide's radius exception |
| 4 | `is_pill` | Declined — `radius(px)` with a large value is already a pill (the painter clamps to half the shorter side) and `sx` names corners; §4 defers it unless it adds a shorthand beyond those. Checkbox keeps `is_round` |
| 5 | `font_family` on TimeField, DateField, ColorPicker, Typography; InputGroup forwarding; `row_font_family` on detached rows | Done 0.9.0 |
| 5 | `font_family` on SearchField, NumberField, ColorField, Select, ComboBox, Autocomplete | **0.13** — composed fields forward it to their `Input` (caret measurement), pickers to the trigger |
| 5 | `text_size` on Input, TextField, SearchField, Badge, Chip, Breadcrumbs, Button; Select trigger/row | Done 0.9.0–0.10.0 |
| 5 | `text_size` on Checkbox, RadioGroup, Tabs, Switch | **0.13** — label type only; control geometry keeps its size step |
| 5 | Font knobs on Kbd, Table cells, Toast | Declined — none hardcodes a family (§5), so a knob would be a new capability with no consumer |
| 6 | `full_width` on TagGroup, Pagination, Breadcrumbs, horizontal RadioGroup, Toolbar | Done 0.9.0 |
| 6 | ComboBox root minimum; vertical Tabs item minimum | **0.13** (`ComboBox::min_width`, `Tabs::vertical_tab_min_width`) |

Totals: 20 done in 0.9.0–0.10.1, 7 implemented in 0.13, 8 declined.

Not part of Phases 0–6 and still open: `Styled` on components (0.14, after
this ownership contract), and keyboard `ContextMenu` anchoring at the focused
element (blocked on GPUI reporting focused-element bounds).

## Corrections to the previous proposal

- The policy gate covers per-component **`radius` as well as field `size`**.
  `extra_audit.py` mentions `RadioGroup::size` as a historical failure, but its
  current explicit ban tables do not mechanically ban that method. An
  informational sibling match is not policy permission.
- `anim::hover_fade` is public (as `anim::hover_fade`; since 0.11.0 it is no
  longer re-exported at the crate root), so replacing its signature is not
  additive: read the duration from the theme inside the existing fade helper.
  `util::apply_field_chrome` became crate-private in 0.11.0 and may change
  freely.
- `ZoomBox::panel` takes **resting vertical padding**, not a zoom offset.
  Padding builders must update both the real panel and its animation geometry.
- The proposed file-level hover grep cannot prove that a particular painted
  part consumes an override. Use an owner/part inventory and scoped checks.
- `height(28px)` is not automatically an exact 28px box for `min_h` controls,
  multiline fields, or groups containing a fixed-height Input.
- `is_bare` suppresses field chrome, not all paint. Text, icons, selection,
  caret and explicitly supplied `sx` paint remain meaningful.
- Root `sx` propagation to children changes existing customised rendering.
  Preserve stock defaults, but describe this as an intentional compatibility
  change rather than promising that every existing override is unchanged.
- Generated website data must accompany the API/gallery change. Batching the
  artifact is valid within one mergeable integration change, not by merging
  independently incomplete PRs and promising to synchronise them later.
- `demo_audit.py` unpacks `.shots/heroui-demos-v3.2.6.tar.gz` on a routine run
  and only fetches on `--fetch`. A clean machine stays offline.
  The current CI test command uses `.shots/run-tests.sh`; the lint script's
  actual Clippy invocation does not include `--all-features`.

## Principles and decisions

1. Preserve stock rendering, interaction and reduced-motion behavior. Keep
   audited defaults associated with their owning selector/binding, with
   overrides applied afterwards. Exact physical line numbers need not stay
   fixed. Some design readers are bounded regexes; others structurally inspect
   branches and builder chains. Read the affected reader before changing its
   source shape, and add negative fixtures if the reader must change.
2. Resolve styling **per painted part and per property**. Theme/default →
   instance configuration → matching `sx` override is the default order for
   the same resting property. A distinct `hover_bg` is a state endpoint, not a
   competing resting background. Do not forward a wrapper background to every
   descendant, list row, track fill or thumb indiscriminately.
3. Preserve the existing Button contract: solid `sx` alone pins both fade
   endpoints; explicit `hover_bg` fades from the resolved resting background
   to that color; neither preserves the component's current state pair.
   Define selected, disabled, invalid and focused precedence separately.
4. Record repository-only builders in narrow, owner-scoped audit exceptions
   where needed. `no-classname` means that GPUI needs a seam for styling v3
   leaves to classes; it does not require a builder for every CSS property.
   The current scanner skips allowed names before reporting sibling matches;
   those matches are informational and need no entry merely to pass the audit.
   Document why an extension belongs on its owner regardless of that bucket.
5. Never add invented upstream rows to `reference_metadata`. Review the
   matching entry for accuracy; add examples and Rust API documentation for
   extensions without presenting them as HeroUI props.
6. Distinguish evidence: bounds probes prove layout; resolver/style-refinement
   tests prove their outputs; source checks prove wiring; screenshots prove
   drawn pixels. None alone proves all of these.
7. Audit public compatibility. Adding fields to public `LayoutTheme` or
   `ThemeDocument` can break exhaustive downstream struct literals even with
   unchanged defaults. Do not label the entire rollout semver-compatible
   without a release decision. Keep existing public function signatures.

Recommended scope decisions for this proposal:

- Defer Checkbox/Radio/Tabs size enums and per-component `radius` builders
  until a narrow, separately reviewed policy amendment defines the allowed
  owners and metrics. Phase 0 can proceed without that amendment.
- Add the shared hover-fade theme duration first. Defer per-instance duration
  builders until a concrete consumer needs one; no required argument is added
  to the existing helper.
- Include Calendar, RangeCalendar, Accordion, Table, Switch and Checkbox in
  the Phase 2 inventory, split into small component changes with explicit
  state precedence. An inventory entry is not a promise of one generic knob.
- Use one mergeable integration PR per chosen batch, with complete generated
  data and a freshly built artifact when its gallery examples change.

## Phase 0 — shared infrastructure

### 0.0 Account for the existing extension surface

The baseline `extra_audit.py` run reports Input/TextField's `height`,
`padding_x`, `is_bare`, `font_family` and TextArea's three delegated styling
builders as unexplained. It still exits successfully. Before extending that
surface, record owner-scoped reasons for the existing intentional extensions
and verify their docs/examples. Read audit findings as well as exit status;
do not use this baseline as proof that every exported builder is accounted for.
If making unexplained builders fail CI, treat that as a separate audit-parser/
gate change with a known-negative fixture and the full required audit checks.

### 0.1 Property extraction, target ownership and verification

`apply_sx` refines the root last, so it wins over that element's resting base
style. GPUI then refines focus/hover/active pseudo-styles at render time.
Child surfaces and the animated fill inserted by `hover_fade` can also cover
the root. Slider has no hover fill; Tabs primarily hovers opacity; Input's
field chrome has no hover background. Treat these as different cases.

Add crate-internal helpers alongside `sx_background` and `sx_pixel_size`:

- `sx_padding(&sx) -> Edges<Option<Pixels>>`, preserving each edge separately.
- `sx_radius(&sx) -> Corners<Option<Pixels>>`, reading `corner_radii`.

Initially extract **Pixels only**. GPUI calls both Pixels and Rems
`AbsoluteLength`; “absolute only” is therefore insufficiently precise.
Padding also admits relative lengths. Corners contain `AbsoluteLength`, so
the unsupported corner case is Rems, not percentage radii. Return `None` only
for the unsupported or absent edge/corner; do not discard the other values.
Keep unsupported values in the final root refinement and document that child
geometry does not reconcile them yet. Likewise, `sx_background` extracts only
solid colors, not gradients or patterned fills.

Before propagation, record each component's root, painted target, supported
properties and state rules. A tab-list wrapper background is not implicitly
a selected-tab background; a slider track color is not implicitly its value
fill. Prefer a named part seam when forwarding would be ambiguous. Preserve
existing wrapper layout semantics for `sx` padding rather than applying the
same padding a second time to a child.

Add helper tests for no override, zero, uniform/per-edge/per-corner values,
and mixed Pixels/Rems/relative padding. Test `Div::style()` directly for
helpers that stamp a style: `tests/cursor_token.rs` already does this. The
headless bounds harness does not expose the final computed fill/radii of
arbitrary descendants; a general post-interaction readback harness remains
an optional spike, not a prerequisite for all work.

Replace the proposed file-level grep with a part-scoped inventory covering
`hover_fade`, multiline/move hover closures, delegated helpers and Tween
paths. Scoped source checks must fail when the actual consumer is removed;
a sibling's `sx_background` call must not satisfy them. Keep reasoned pending
entries and validate scanners with positive and negative fixtures. Update
the component guide with the settled extraction and ownership contract.

### 0.2 Theme tokens

These are proposed HeroGPUI configuration tokens unless already present;
do not invent corresponding upstream CSS variables.

| Token | Default | Consumer and constraints |
|---|---|---|
| `tabs_hover_opacity: f32` | 0.7 | Tabs' two tab paths and overflow arrow. Component-scoped name avoids suggesting every hover is opacity-based. Define finite 0–1 input handling. |
| `tooltip_cooldown_ms: u64` | 500 | Replace the global cooldown floor; retain `max(close_delay)` and generation cancellation. |
| `long_press_ms: u64` | 500 | Dropdown long-press recognition. Preserve cancellation/release behavior. |
| `hover_fade_ms: u64` | 100 | Read inside existing `hover_fade`; preserve `TRANSITION_MS` as the public default constant. Zero duration and reduced motion must resolve immediately. |
| `tooltip_delay_ms: u64` | 1500 | Already in LayoutTheme; expose through builder/document. |
| `tooltip_close_delay_ms: u64` | 500 | Already in LayoutTheme; expose through builder/document. |

Do not change Checkbox, Tabs or Switch's separate Tween/indicator timings.
Link's root hover is an underline and its icon brightens from 0.6 to 1.0;
neither consumes the Tabs token.

Each new token needs a layout field/default, ThemeBuilder method,
ThemeDocument optional key/serde support, application path and tests proving
both builder and JSON affect the consuming behavior. Existing layout fields
need the missing pieces only. `theme_serde_audit.py` compares builder/document
names; it does not prove LayoutTheme coverage or that rendering reads a token.
Run theme tests explicitly with the non-default `serde` feature.

Drop `overlay_zoom_offset`. `ZoomBox::panel`'s per-overlay values represent
real padding (including asymmetric x/y values), not motion amplitude. Handle
padding locally in Phase 3; retain each overlay's existing motion specification.

### 0.3 Field configuration

A crate-internal `FieldBox` can hold optional `height`, `padding_x`,
`font_family`, plus `is_bare`. Adopt it only where it removes real repetition;
do not require a public trait or a builder-generating macro. The audits scan
explicit owner `impl` methods, so macros would add parser work to this phase.

Keep `apply_field_chrome`'s existing signature and gate it at each owning
component. It paints background, border, radius and state shadows; it does
not own height, padding or typography. Keep those values at component-local
sites, including grouped-input rules and custom text measurement. Preserve
the separate multiline path. Phase 1 supplies geometry/chrome builders;
Phase 5 supplies missing font builders and propagation.

## Phase 1 — field geometry and chrome

Audit/build the missing geometry seams on NumberField, Select, ComboBox,
Autocomplete, DateField, TimeField, ColorField, InputGroup and SearchField.
Input already has `height`, `padding_x`, `is_bare`, `font_family`; Input's
TextField wrapper delegates all four. TextArea delegates the last three and
uses `rows` for multiline height; SearchField forwards none of the four.
ColorField has no `sx` and has both delegated editable and direct-rendered
paths: new settings must work in both.

Define `height` as the single-line field box height when explicitly supplied,
with fixed height and minimum constraints reconciled so a standard 28px
specimen actually measures 28px. Preserve today's `min_h` behavior when
unset, including Select's growing content and InputGroup's textarea path.
Do not apply the 36px acceptance criterion to whole wrappers containing
labels/errors, multiline TextArea, or oversized custom content. Document the
overflow policy for content taller than an explicit height.

InputGroup needs dedicated rules: it has no root padding today; addons and
Input carry their own edge padding. Override the exposed outer edges once,
and propagate single-line height to the inner Input so its 36px box does not
defeat a smaller group height. Keep textarea groups content-sized. Apply
`is_bare` consistently to the group's chrome and any separate focus/invalid
chrome, following Input's existing meaning.

Row geometry is a separate opt-in seam on each owning collection. Select and
ListBox currently use 10px horizontal/6px vertical padding. Preserve those
defaults and both plain/virtual row paths. Do not mix row hover colors into
this phase.

Width recipes must state the real receiver and signature. Input/TextField,
SearchField and TextArea use `.full_width()`; Select/Autocomplete/DateField
and several other fields use `.full_width(true)`. For capped fields, remove
the default cap with that builder and use a sized host or suitable `sx`.
Uncapped fields still need a constrained host for percentage width. ColorField
editable mode delegates to capped Input, so it is not universally uncapped.

Acceptance: measure the actual field box with and without label/error content;
default and explicit height; padding delta; grouped addon edges; multiline
growth; and unchanged caret/hit testing. Check bare chrome via helper output
and scoped wiring plus a visual check. Preserve existing text-field and picker
behavior tests; add focused cases rather than freezing entire test files.

## Phase 2 — state colors and opacity

Use Button's existing endpoint tests as the starting contract. Each change
must identify the painted part and account for selected/checked, hover,
pressed, disabled, focus and invalid state where applicable.

| Batch | Inventory |
|---|---|
| Simple controls | ToggleButton, CloseButton, Pagination controls, TagGroup tag/remove parts, Toast close, Dropdown rows |
| Field/collection parts | Select trigger/rows, ComboBox rows, Autocomplete trigger/rows/clear, ListBox rows, NumberField group, InputGroup, TimeField stepper, InputOtp slot, Input search clear, DateRangePicker trigger |
| Stateful controls | Calendar and RangeCalendar nav/year/day parts, Accordion header, Table rows, Switch track and Checkbox control |
| Other styling | Tabs opacity token and selected/indicator ownership; Link underline/icon behavior only if a distinct consumer requirement justifies a seam |

Do not give every listed component one generic `hover_bg`. Trigger, row and
clear-button ownership differ. Switch/Checkbox use Tween state, so an
ordinary `.hover()` rewrite would bypass their animation and press semantics.

Table **can** have one `row_hover_bg` if explicitly scoped to unselected
interactive rows: preserve `selected_bg.unwrap_or(custom_or_default_hover)`.
A selected-row hover override would be a separate decision. Calendar needs
equally explicit selected/today/range/disabled rules.

Pagination's hover and pressed colors share the same default upstream value;
they are distinct CSS variables, so a `pressed_bg` is not inherently invalid.
Defer that independent knob as unnecessary scope. Specify whether a new
hover override also feeds pressed state; recommendation: keep the pair
coupled for this rollout. Press scale is size-dependent upstream, not always
0.97 (Sm/Lg override it). Preserve the existing size-specific contract.

Acceptance: resolver cases for defaults, `sx` only, named hover only and both;
selected/disabled/invalid precedence; reduced-motion endpoints; and wiring
checks for each real paint consumer. Existing render-prop state tests prove
interaction flags, not fill colors. Use focused screenshots for drawn states;
do not describe source-shape checks as pixel measurements.

## Phase 3 — opt-in sizing and overlay padding

Policy-gated work: CheckboxSize, RadioSize and TabsSize require a documented
HeroGPUI-only extension policy. SliderSize is precedent, not automatic
permission. Before implementation, specify every Sm/Md metric: control,
indicator, hitbox, gap, padding, typography, radius and orientation behavior.
Keep Md equal to the pinned default; do not claim an upstream compact recipe
that does not exist. No enum lands with an undefined metric table.

Independent work: add a narrowly named panel-padding seam to a concrete
overlay when needed. A scalar override can set both axes while an absent
override preserves asymmetric defaults. Feed the same resolved padding into
the resting panel and `ZoomBox`, including entering/exiting/reduced-motion
paths and anchored bounds. ModalSize already exists.

Acceptance: bounds probes for dimensions, centers and hit regions; overlay
rest/enter/exit geometry and no unintended origin shift. Bounds do not prove
corner radii.

## Phase 4 — radius and shape

Start with unambiguous per-corner `sx` reconciliation from Phase 0. Do not
copy one scalar into Slider track/fill/knob, Switch track/thumb or Tabs
selected/indicator without defining their separate targets and defaults.
For the same target, explicit `sx` corners should win individually over an
instance radius, then the size/theme default; this avoids conflicting root
and child precedence.

Per-component `radius` builders remain policy-gated. Do not bypass the policy
by renaming the same removed prop. Defer `is_pill` unless it provides a
necessary consumer shorthand beyond `sx`; preserve Checkbox's existing
`is_round`. Stock shapes are component/size-specific. Slider Md is not the
same shape as its already fully rounded Sm variant; ProgressBar and Meter
derive their radii from size; Badge uses per-size radius steps.

Acceptance: resolver/style-refinement tests for each corner and precedence,
scoped wiring checks for child/animated shapes, and focused visual checks
under stock theme and `ThemeBuilder::radius(px(2.))`. Include partially
specified corners and unsupported Rems fallback. Do not claim `debug_bounds`
measures radii.

## Phase 5 — typography

Implement missing field `font_family` builders and forwarding after the field
configuration settles. Input, TextField and TextArea already support it.
Group fonts must reach the internal Input's shaping/caret measurement as well
as inherited text style. Detached popover rows need explicit forwarding or
a separately documented row font seam; trigger inheritance is insufficient.

Actual hardcoded family sites are TimeField, DateField, ColorPicker's text
field and Typography's mono branch. Kbd, Table cells and Toast do not hardcode
a family; knobs there are optional new capabilities rather than replacements.

For `text_size`/`line_height`, preserve each selector's exact current pair.
The design audit includes the 12/16, 14/20 and 16/24 Tailwind pairs, but these
are not a universal replacement for fractional/component-specific leading.
Checkbox and Switch contain relevant literal label pairs. Define how an
override affects line boxes without silently changing control geometry.

Acceptance: font propagation in composed/detached paths, measured text/caret
agreement, labels/descriptions and multiline wrapping; default typography
still passes the owning design readers.

## Phase 6 — layout sizing

ButtonGroup and Tabs already expose `full_width`. Candidate additions are
TagGroup, Pagination, Breadcrumbs, horizontal RadioGroup and Toolbar. Define
whether each builder expands just the outer box or distributes children;
do not imply equal item widths from the name alone. Preserve vertical,
wrapping and overflow behavior. Reuse Tabs' existing equal-division test as
evidence for Tabs' own contract only.

Use separate, descriptive seams for ComboBox's 180px root minimum and the
80px minimum on vertical tab items. These have different owners/roles.
Preserve the condition that ComboBox's current minimum applies only when
`full_width` is false, unless an explicit override contract says otherwise.

## Sequencing and integration

| Work | Dependencies |
|---|---|
| 0.0 extension accounting | None; finish before expanding the existing field styling API |
| 0.1 helpers/ownership | None |
| 0.2 tokens | Compatibility decision for new public fields; no dependency on size policy |
| 0.3 field configuration | Read existing Input/group contracts; no forced dependency on helper implementation |
| 1 field geometry | 0.3; 0.1 for any promised `sx`-derived geometry |
| 2 state styles | 0.1; 0.2 when consuming duration/opacity tokens |
| 3 size variants | Policy amendment and complete metric tables |
| 3 overlay padding | 0.1 if reconciling `sx`; existing animation geometry contract |
| 4 shapes | 0.1; settle affected size metrics with 3; policy amendment for radius builders |
| 5 typography | 0.3 and settled Phase 1 field/group forwarding |
| 6 layout | No global prerequisite; coordinate owners shared with 1/3/5 |

Row hover does not logically require new row padding. Phases 1 and 2 may be
implemented independently when they do not edit the same row builder;
otherwise sequence that owner. Likewise, fonts and sizes share files even
when their conceptual dependencies differ. All token changes touch shared
theme files, and all public builder changes touch common audit/docs surfaces.
This table describes dependencies, not a guarantee of conflict-free worktrees.

Every mergeable API/behavior/gallery change includes its implementation,
focused tests, discoverable gallery example, accurate reference metadata,
`llms.txt`, audit exceptions and regenerated website datasets as applicable.
Do not defer these to a later merge. A batch may have preparatory draft
branches, but the integration PR must satisfy the complete contract.

When gallery example bodies/descriptions change, rebuild the checked-in wasm
artifact, then run `pnpm run wasm:manifest`, `pnpm run extract` and
`pnpm run extract:check` from `web/` before the batch is mergeable. Follow the
root guide's nightly/wasm-bindgen instructions and never set RUSTFLAGS.
Do not regenerate manifests against a stale binary merely to make checks pass.

The manifests hash the artifact/glue and example code/descriptions; they do
not hash every component implementation. Thus `extract:check` alone cannot
prove the binary implements current Rust internals. Account for source-only
behavior changes when choosing the artifact rebuild batch as well.

## Verification per change

- Start with `git status --short`, read scoped guides and preserve unrelated
  work. Finish by inspecting the exact diff and status.
- Use focused component tests while iterating. For broad handoff, follow the
  workflow matrix and current `.github/workflows/ci.yml`, not a copied command
  list claiming full CI equivalence.
- Relevant audits include design/token/state/animation, API/extra/write-only,
  and theme-serde. Run the complete audit set for broad contract or parser
  changes, including known-negative parser checks where applicable.
- Current CI uses `bash .shots/run-tests.sh --workspace --locked` and a
  separate `bash .shots/run-tests.sh -p herogpui-theme --features serde --locked`.
  Format check is `cargo fmt --all -- --check`. `bash .shots/lint.sh` runs on
  any host and includes `cargo clippy --workspace --all-targets -- -D warnings`;
  its cargo-deny check depends on the tool being installed (CI passes
  `--require-deny`). Feature isolation,
  Rustdoc, website and wasm build jobs are additional CI gates.
- Rebuild and visually verify component/gallery changes using the gallery
  guide. The checked-in rebuild/capture scripts contain Windows-specific
  paths/APIs; PowerShell alone does not make those drivers usable on macOS.
  Report platform limits and the actual native/web verification used.
- Ordinary bundle/CSS/demo audits use pinned local inputs. `--fetch` is the
  only network path and deliberately refreshes preview sources; `--pack`
  rewrites the checked-in archive afterwards. Record which mode ran.
- Documentation-only edits require link/path/command verification, not Rust
  compilation. Report checks actually run and distinguish plan review from
  implementation verification.

## Evidence map

| Claim | Local source |
|---|---|
| `sx`, field chrome, pixel extraction | [util.rs](../crates/herogpui-components/src/util.rs) |
| Existing endpoint precedence and tests | [button.rs](../crates/herogpui-components/src/button.rs) |
| Public fade API and real overlay geometry | [anim.rs](../crates/herogpui-components/src/anim.rs), [exports](../crates/herogpui-components/src/lib.rs) |
| Field height/group/custom font behavior | [input.rs](../crates/herogpui-components/src/input.rs), [textarea.rs](../crates/herogpui-components/src/textarea.rs), [input_group.rs](../crates/herogpui-components/src/input_group.rs) |
| ColorField's two render paths | [color field](../crates/herogpui-components/src/color_picker/field.rs) |
| Select minimum and row ownership | [select.rs](../crates/herogpui-components/src/select.rs) |
| Selected-row hover precedence | [table.rs](../crates/herogpui-components/src/table.rs) |
| Theme fields and sparse serialization | [layout.rs](../crates/herogpui-theme/src/layout.rs), [theme.rs](../crates/herogpui-theme/src/theme.rs), [theme_document.rs](../crates/herogpui-theme/src/theme_document.rs) |
| Scope/classification and metric readers | [extra audit](../.shots/extra_audit.py), [design audit](../.shots/design_audit.py), [theme serde audit](../.shots/theme_serde_audit.py) |
| Bounds versus source/refinement evidence | [slider tests](../crates/herogpui-components/tests/slider_geometry_deep.rs), [cursor tests](../crates/herogpui-components/tests/cursor_token.rs) |
| Artifact check coverage | [manifest tests](../web/scripts/extract-rust-examples.test.mjs), [website commands](../web/package.json) |
| Current verification and network behavior | [CI](../.github/workflows/ci.yml), [lint](../.shots/lint.sh), [demo audit](../.shots/demo_audit.py) |

GPUI evidence: re-derive from the unpacked `gpui-pre-0.3.5` registry sources
(the earlier `gpui-pre-0.3.3/src/style.rs` `corner_radii` reference was
retired-fork API and no longer exists on vanilla), plus `src/geometry.rs`
(`AbsoluteLength`) and `src/elements/div.rs` (`compute_style_internal`). Pagination CSS was checked directly in the pinned CSS archive,
including the size-specific pressed-scale rules.

## gpui-kit adoptions: done in 0.11.0 and 0.12.0, and next

Source: `tmp/review/gpui-kit-comparison.md` §4. Landed in 0.11.0 (Phase 4):
`CONTRIBUTING.md` and the pull request template, the `examples/` crates, the
`Disableable`/`Sizable`/`Selectable` traits, the `i18n` chrome-string
catalogue, theme files with a checked-in schema and presets, the
`herogpui::test` kit, `VirtualList` and `ContextMenu`. Landed in 0.12.0:
`MenuBar`, `ResizablePanelGroup` (split panels with keyed drag state,
per-frame pointer capture, keyboard resizing and per-panel limits),
`TreeView` (on `list_nav` and the shared selection rules, with the same
open-parents-only walk as `Table`'s tree rows), keyboard opening for `ContextMenu` (Shift+F10 and the ContextMenu
key, anchored at the area's corner because GPUI reports no bounds for the
focused element), and `VirtualList` behind the `estimated_row_height` bodies
of `ListBox` and `Table`; a public Lucide icon set (`IconName` and `Icon`,
245 icons from `lucide-static` 1.31.0) as a module of `herogpui-components`
rather than the `herogpui-icons` crate §4 #8 proposed — `HeroGpuiAssets`
already embeds the chrome icons, so a second crate would only add a sixth
crates.io name and release step for no build-graph saving — and a gallery
theme picker over the light/dark bases and the presets. 34 source-text radius assertions (the store and
resolve checks of 17 components) became a painted-scene test (`tests/radius_painted.rs`); the remaining source-text
assertions cover overlay panels, fields and wiring a headless paint does not
reach yet, and are the next candidates.

Deliberately not done, and next in this order:

1. **`Styled` on components** (comparison §4 #3, L). Replaces per-prop box
   builders with GPUI's full style surface backed by the existing `sx`
   refinement. Needs the per-part ownership rules above first, because a root
   `Styled` call must not silently restyle child parts.
2. **VirtualList for the fixed-height paths.** Done for 0.13.
   `VirtualListHandle::uniform` gives `VirtualList` a uniform mode (GPUI's
   `uniform_list` underneath, so a row is still measured once and
   multiplied), and the fixed `row_height` paths of `ListBox` and `Table`
   and ComboBox's popover list render through it. The handle keeps what
   they relied on: `VirtualListScroll::Center` is `ScrollStrategy::Center`
   (non-strict, clamped), `viewport_bounds()` is the laid-out viewport
   PageUp/PageDown step over by the declared row height, and
   `remaining_below()` is `Table`'s load-more reading (row count times the
   measured row, less the scroll offset and the viewport); the
   `collection_contracts`, `virtual_and_feedback` and `table_deep` suites
   pass unchanged. `Select` and `Autocomplete` still call `uniform_list`
   directly and are the next candidates.
3. **Theme hot reload** (`watch_dir`); **i18n** for more locales (non-Latin locales also need the web
   font subsets extended) and for the remaining hard-coded strings (NumberField
   stepper names, ColorPicker channel names, DateField segment names). Done
   for 0.13: `watch_themes_dir`, four more locales, and every chrome string,
   the last being `Table`'s load-more row (`UiString::LoadingMore`).
4. **Follow-ups on the 0.12.0 split, tree and icon extensions.** Done for
   0.13. `ResizablePanel` takes pixel limits (`min_size_px`/`max_size_px`,
   converted against the measured group length at every clamp, from the
   frame after the first layout), collapses (`collapsible`,
   `collapsed_size`; drag past halfway, arrows, Home/End, Enter on the
   handle) and reports `on_resize_end`. `TreeView` extends a multiple
   selection with Shift, renders through a uniform `VirtualList` (`max_h`
   or a bounding parent builds only the rows in view) and reports
   `aria-posinset` / `aria-setsize` among siblings. `Icon` takes an
   `IconSize` step (`Sizable`) and a stroke width (`stroke_width`,
   `absolute_stroke_width`: the asset source rewrites the SVG's
   `stroke-width` per width, since gpui paints an SVG as one mask); the set
   is `.shots/lucide-icons.txt`, synced by `.shots/sync-lucide.py` (`--check`
   in CI's parity job); and the gallery has an Icons page. Left: stroke
   width reaches only the Lucide set, not the chrome icons or an app's own
   SVGs, and a pixel limit cannot hold on the very first frame, before the
   group has been measured.

### Extensions from the gpui-kit gap list (0.13, unreleased)

Landed on the 0.13 line, each a labelled HeroGPUI extension outside
`reference_metadata`, with a focused behaviour binary, a gallery section on a
related component page, an `llms.txt` entry and an `a11y_audit.py` row:
`Sidebar` (groups, collapsible headings on `Disclosure`'s panel motion, one
roving tab stop, collapse to icons with tooltips), `TitleBar` and
`WindowBorder` (drag, double-click zoom and the per-platform control split
on the pinned window APIs), `CommandPalette` (a modal search on the overlay
stack and `matches.rs`), `HoverCard` (the Tooltip's generation-timer scheme
with Radix's 700/300ms delays), and on `Table` column reordering and cell
selection. The comparison's "Toolbar extras" row names no single feature;
the concrete gaps against gpui-kit's `toolbar.rs` were its size propagation
to the controls and a named group, which landed as `Toolbar::size` /
`sized_child` and `Toolbar::label`. gpui-kit's toolbar has no overflow menu,
so there is none to port.

Not done, with the reason:

- **Frozen (sticky) table columns.** GPUI 0.3.5 has no sticky positioning:
  `style.rs`'s `Position` is `Relative` or `Absolute` only. The two
  emulations were checked against the pinned sources and neither is a
  localized, correct change yet. (a) An overlay: paint the leading cells with
  `Window::defer_draw(.., Some(window.content_mask()))` shifted back by the
  horizontal scroll offset. Paint order and clipping come out right, but hit
  testing does not: `Window::hit_test` collects every non-blocking hitbox
  under the pointer (`hitbox.bounds ∩ content_mask`), so a press on a frozen
  cell also reaches the cell (and any caller control) scrolled underneath,
  and making the frozen cell `HitboxBehavior::BlockMouse` also blocks the
  row element beneath it, which owns the row's press, selection and hover.
  Fixing that means moving the row interactions onto the cells. (b) Split
  rows: each row and the header as a frozen part plus a clipped scrolling
  part moved by a manual offset. That is correct, but it replaces the
  table's native `overflow_x_scroll` with manual wheel and scroll-bar
  handling and splits every row builder path; it belongs with the planned
  `table.rs` render split (0.14), after the fixed-row VirtualList move.
- **Sidebar width drag.** `ResizablePanelGroup` sizes panels in percent
  (see item 4 above), so a sidebar in a panel resizes but has no pixel
  minimum; the sidebar's own widths are fixed pixels.
- **TitleBar resize edges on Windows and macOS** are the OS's; `WindowBorder`
  draws edges only for Linux client-side decorations. The browser has no
  window to move, zoom or close.

## Remaining work after 0.12.0

Status: reviewed on 2026-09-29, after the 0.12.0 release, against this
document, the review reports (`tmp/review/*.md`: gallery-website,
gpui-kit-comparison, heroui-3.2.6-upgrade, library-architecture) and the
release run. Each item was re-checked in the tree; items the reports raise
that 0.10.2–0.12.0 already closed are left out. Effort: S under a day, M one
to five days, L over a week.

| Pri | Item | Why it matters | Effort | Target |
|---|---|---|---|---|
| P1 | ~~Release hygiene: `cargo semver-checks` in CI and the release, an MSRV (1.98) job, a `--no-default-features` job~~ **Done (0.13):** `semver` PR job (label `semver:breaking` to accept a deliberate break) and a release `semver` job gating the GitHub Release; `no-default-features` job; no MSRV job because the toolchain pin is the MSRV release, asserted by `package_audit.py` | Public-API breaks in a minor or patch release are caught by review only | S | 0.12.1 |
| P1 | ~~Stale CI and agent comments ("70 test binaries" in `ci.yml` and `rust-env`, now 110+; "Git GPUI is not registry-publishable" in `ci.yml`; "stable wasm32 build" in `docs/agents/workflow.md`)~~ **Done (0.13):** fixed with the other stale facts found (deny.toml, RELEASING.md, clippy/toolchain comments); `.shots/stale_docs_audit.py` fails on a reintroduced phrase | Contradictory guidance misleads contributors and agents | S | 0.12.1 |
| P1 | ~~Website command palette a11y (`combobox`/`listbox`/`aria-activedescendant`) and search over API items~~ Done in 0.13: ARIA combobox pattern; builders and types indexed from `reference.json` | Keyboard selection is silent to screen readers; Rust users search by builder name | M | 0.12.1 |
| P1 | ~~Website SEO and links: canonical URLs, sitemap, robots, per-page OG metadata, docs.rs and source links on component pages~~ Done in 0.13 (the parent zone's `robots.txt` should list the zone sitemap; see `web/DEPLOYMENT.md`) | Discoverability and a path from the site to rustdoc | M | 0.12.1 |
| P1 | Wasm cold load: `wasm-opt` pass and ~~a lazily mounted hero embed~~ (the hero already mounts on demand; 0.13 adds an intent prefetch of the versioned artifact) | The ~19 MB artifact is the slowest thing on the site | S | 0.12.1 |
| P1 | ~~Cross-platform lint gate: port `.shots/lint.ps1` to bash (like `run-tests.sh`) and make `demo_audit` runnable offline~~ **Done (0.13):** `.shots/lint.sh` (CI calls it; `lint.ps1` forwards to it); `demo_audit` already ran offline from the checked-in archive since #15, and its self-test now runs in CI | The documented gate cannot run on macOS or Linux without pwsh | M | 0.13 |
| P2 | ~~Remaining ~116 source-text assertions (overlay panels, fields, wiring) to painted-scene or behaviour tests~~ Done in 0.13: `include_str!` 115 → 0; the 63 source-reading tests in those 21 files → 19, each kept one listed with its reason in `crates/herogpui-components/tests/README.md` (svg/sprite, shadow, cursor and AccessKit output, and text with no probe slot are not exposed headlessly) | They pinned source shape and blocked refactoring the large render functions | L | 0.13 |
| P2 | VirtualList uniform mode for the fixed `row_height` paths of ListBox, Table and ComboBox (see item 2 above) | Removes the last `uniform_list` split without losing centred scrolling, paging or load-more | M | done (0.13) |
| P2 | TreeView: Shift range selection, virtualisation, `aria-posinset`/`aria-setsize` | Large trees build every visible row; set position is missing for assistive technology | M | done (0.13) |
| P2 | ResizablePanel: pixel min/max, collapsible panels, `on_resize_end` | The common split-pane needs beyond percentages | M | done (0.13) |
| P2 | Icon: `Sizable` steps, stroke width, a Lucide sync script, a gallery Icons page | The set cannot grow or be browsed without hand work | M | done (0.13) |
| P2 | Theme API safety: typed roles for `ThemeBuilder::role` (a typo silently recolours the accent); stop reading `HEROGPUI_REDUCE_MOTION` from the environment inside the library | Silent misconfiguration in a library API | S | **Done** (0.13): `ThemeBuilder::role`/`role_hover` and `ThemeColors::role` take `Color` (`FromStr` fails on an unknown name); `set_reduce_motion` is the only setter, the gallery maps the variable |
| P2 | `#[must_use]` on builders (none today) and a `missing_docs` ratchet | A dropped builder does nothing, silently; public docs have gaps | S / M | **Done** (0.13): type-level `#[must_use]` on every component and builder type; `missing_docs` warns (so `clippy -D warnings` fails) in core, theme, components and the facade. Every public item is documented |
| P2 | i18n: more locales, the hard-coded NumberField/ColorPicker/DateField strings, non-Latin web font subsets | Localisation is incomplete for real users | M | **Done** (0.13): ja-JP, zh-CN, ko-KR, ru-RU; steppers, segments, channels, DatePicker trigger, selected day, Autocomplete clear, Pagination; Noto Sans KR subset and a wider SC pre-reduction; `Table`'s "Loading…" row (`UiString::LoadingMore`) |
| P2 | Theme hot reload (`watch_dir`; `ThemeRegistry` has `load_dir` only) | Faster theming workflow | S–M | **Done** (0.13): `watch_themes_dir` behind the `watch` feature (polling, no new dependency); the native gallery watches `HEROGPUI_THEME_DIR` |
| P2 | ~~Website hardening: CSP and `frame-ancestors`, remove the unused `web/public/shots/` images, PR preview deploys~~ Done in 0.13 (previews were already on through Vercel's Git integration) | Security headers and deploy size | M | 0.13 |
| P3 | Stop committing the wasm artifact; build it in CI and publish it with the site | Repository weight grows ~19 MB per gallery change | L | 0.14 |
| P3 | `Styled` on components (item 1 above), after the per-part ownership rules of Phase 0.1 | Largest API change and semver risk; needs part ownership first | L | 0.14 |
| P3 | Split the largest `render` functions (`table.rs` and others), merge the 110+ test binaries into a few suites | Review cost and link time | L | 0.14 |
| P3 | ~~Customisation Phases 0.2–6 (theme tokens, field geometry, state colours, sizing, shape, typography)~~ **Done (0.13):** most landed in 0.9.0–0.10.1; the close-out implemented the rest and recorded each decline (see [Status of Phases 0–6](#status-of-phases-06-013-close-out)) | The largest documented backlog | L | 0.13 |
| P3 | ~~Opt-in -O1/-O3 (`Cargo.toml` profile) test job on pull requests, by label or path filter~~ **Done (0.13):** `ci:opt-levels` label, or a PR touching build configuration | It is push-only by design, so an optimisation-level regression first shows on master or at tag time | S | 0.13 |
| P3 | Extensions from the gpui-kit gap list: Sidebar, TitleBar, CommandPalette, HoverCard, Toolbar extras, data-table extras | Done on the 0.13 line except frozen table columns (see "Extensions from the gpui-kit gap list" above) | L | 0.13 |
| P3 | Frozen table columns | GPUI has no sticky positioning; the overlay emulation mis-routes presses and the split-row one needs the `table.rs` render split | M | 0.13 (with the `table.rs` render split) |
| P3 | Keyboard ContextMenu anchored at the focused element | Needs GPUI to report focused-element bounds | S once upstream lands | later |
| P3 | Upstream GPUI work: IME-mirror resync after paste, the retired patches, the `block` 0.1.6 future-incompatibility warning every macOS build prints (chain and requested fix in [`docs/upstream/gpui-block-future-incompat.md`](upstream/gpui-block-future-incompat.md); one path is HeroGPUI's own `locale_config` dependency); multithreaded wasm needs a COOP/COEP deployment | External dependencies | M each | later |

Ordering: P1 is cheap, user-visible or process-critical and safe for a patch
release; P2 is the 0.12.0 follow-ups and library-quality debt sized for one
minor release; P3 is large, policy-gated or blocked upstream. The
source-text-assertion work comes before the render-function split because it
unblocks it, and `Styled` comes last because of its semver reach.
