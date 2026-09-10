# Customisation roadmap

Status: proposal, source-reviewed against `b72f285` on 2026-09-10. This
document plans implementation; it does not amend repository policy or claim
that the proposed APIs or tests already exist.

Authority: [workflow](agents/workflow.md), [component guide](agents/components.md),
[parity policy](agents/parity.md), and the root [AGENTS.md](../AGENTS.md).
Upstream evidence comes from the checked-in HeroUI **v3.2.4** bundle/CSS and
the unpacked **gpui-pre 0.3.3** registry sources. Recheck source symbols when
starting each phase; line numbers are not durable contracts.

## Corrections to the previous proposal

- The policy gate covers per-component **`radius` as well as field `size`**.
  `extra_audit.py` mentions `RadioGroup::size` as a historical failure, but its
  current explicit ban tables do not mechanically ban that method. An
  informational sibling match is not policy permission.
- `anim::hover_fade` and `util::apply_field_chrome` are public functions;
  `hover_fade` is also re-exported at the crate root. Replacing their signatures
  is not additive. Read the duration
  from the theme inside the existing fade helper; retain the chrome helper's
  signature.
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
- `demo_audit.py` fetches preview sources on a cache miss or `--fetch`; it is
  neither unconditionally online nor guaranteed offline on a clean machine.
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
root guide's pinned stable/wasm-bindgen instructions and never set RUSTFLAGS.
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
  Format check is `cargo fmt --all -- --check`. `.shots/lint.ps1` requires
  PowerShell and includes `cargo clippy --workspace --all-targets -- -D warnings`;
  its cargo-deny check depends on the tool being installed. Feature isolation,
  Rustdoc, website and wasm build jobs are additional CI gates.
- Rebuild and visually verify component/gallery changes using the gallery
  guide. The checked-in rebuild/capture scripts contain Windows-specific
  paths/APIs; PowerShell alone does not make those drivers usable on macOS.
  Report platform limits and the actual native/web verification used.
- Ordinary bundle/CSS audits use pinned local inputs. `demo_audit.py` additionally
  needs cached preview sources or network; `--fetch` deliberately refreshes
  them. Record which mode ran instead of calling the whole set offline.
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
| Current verification and network behavior | [CI](../.github/workflows/ci.yml), [lint](../.shots/lint.ps1), [demo audit](../.shots/demo_audit.py) |

GPUI evidence: `gpui-pre-0.3.3/src/style.rs` (`corner_radii`),
`src/geometry.rs` (`AbsoluteLength`), and
`src/elements/div.rs` (`compute_style_internal`) in the unpacked Cargo
registry. Pagination CSS was checked directly in the pinned CSS archive,
including the size-specific pressed-scale rules.
