# UI and design parity execution progress

Target: HeroUI React v3.2.5 at `5f13f6ed355bdbd5d5f69e5944685438a3591793`.
Starting HeroGPUI commit: `cdc93ba4d4686c68cc6e4a2e1923b6c36ddb2127`.
Updated 2026-09-14. The [plan](ui-design-plan.md) and
[component checklist](ui-design-component-checklist.md) remain the full scope.

## Current frontier

Batch 0 is in progress. Inventory/report tooling, CI integration, the first
Slider gallery fixes, the ComboBox callback correction and the gallery
frame/reset control protocol are implemented. Form-state retention and duplicate
gallery identities are corrected; the final native accessibility route sweep
passes all 76 routes. Browser wheel input now reaches GPUI unchanged.
The dev-only matched-fixture runner (plan §5.1, [fixtures.md](fixtures.md))
now renders pinned `@heroui/react` 3.2.5 compositions for Button/Select/Modal
in the browser, and the first three matched capture sets exist under
`docs/parity/evidence/matched-{button-usage,select-usage,modal-open}/`
(upstream fixture vs port captures in both themes, comparison and difference
images, per-side provenance JSONs; Select's 256×36 trigger measures identical
on both sides). The web gallery's `?overlays=1` query now reaches the same
pre-open state as the native `HEROGPUI_OPEN_OVERLAYS` seed, so the Modal set's
resting open capture is addressable without a click; real input is still
required for interaction evidence. The shared control protocol now supports stable per-instance
`specimen` keys in component previews and rejects unknown keys after the frame
check. Button variant examples, Select Usage and Controlled Open State, and all
Modal overlay helpers claim keys; the focused gallery regression covers positive
and negative Button/Select/Modal requests. The complete date-time gallery batch
now has stable keys, including grouped locale, heading-offset, granularity,
leading-zero and variant cases. No component has earned a complete
variant/state verdict from this pass. The remaining foundation and component
batches have not been completed. The rapid browser ComboBox burst is now
resolved at the shared text-input layer and has a passing native/WASM capture.
The latest cache-busted browser pass also checked the ColorArea and ColorSlider
edge thumbs, Select's wrapped trigger and natural-height option rows,
Autocomplete's selected-value clear affordance, the ProgressCircle value
transition, Skeleton's `Single Shimmer` composition and Tooltip's `Long
Content` wrapping, Spinner's HeroUI arc asset, ColorField's read-only display
group, ColorPicker's focused/open trigger, and the shared Input/TextArea/InputGroup chrome;
no browser errors were reported. The latest pass also tabbed to the persistent
`toast-close-reveal` card and confirmed the close control's rounded keyboard
focus ring and Enter dismissal.
The synchronized artifact is
`898bf0269260d8c3d4373827234607398e36b0045cbdb6ef7366aabae742f110`.

The follow-up visual audit on 2026-09-13 rechecked the user-reported failure
shapes against that cache-busted artifact: ColorSlider's Hue strip and Slider's
blue fill stay inside the rounded track at both endpoints; the Toast close
control's keyboard ring follows its 20px rounded target; InputGroup Usage uses
the pinned 12px `rounded-field` shell; and Select's long selected value and
natural-height option row wrap without truncation or marker overlap. The
rounded-mask, close-button, Select.ClearButton and focused browser checks all
pass. These captures do not close the inventory's remaining unobserved state
queue; they confirm the shared root clipping/focus fixes are present in the
current artifact.

The generated [coverage report](coverage-report.md) is now the handoff queue
for the remaining work. It derives status, scope, evidence-surface and
per-component unresolved records from the inventory, and keeps the known Tabs
Select measured gap, the accepted focus-modality deviation, and the native
and web harness limits visible. Refresh it with
`.shots/coverage_report.py --refresh` and use `--check` in CI; it does not
claim parity from the current unreviewed section seeds.

The 2026-09-14 implementation wave is source-frozen. Named goal-item-2 gaps
are closed or recorded: InputOTP slot chrome and NumberField group chrome
interpolate on the pinned 150ms field-chrome ramps; Tabs match the fixed
32px tab box, with overflow wrap painting past the pill; Table narrow width
shares one measured track triple on native GPUI while wasm32 stays fluid
(documented Partial); Select clear pressed-target scaling is implemented
and custom-child subtree scaling remains `platform-limited` (gpui-pre 0.3.3
Svg-only transforms). A macOS native capture driver
(`.shots/native_capture.py`) landed with a Button Usage pilot set under
[native-button-usage](evidence/native-button-usage/). Gallery metadata
tests, extract checks, inventory/coverage `--check`, `parity_report.py`,
and the component and gallery suites are green. The synchronized artifact
is
`6067b2e3e8810b32b8fc5af431b4f5a2e1e40825b2bee6aed7fa31fe19e7cfce`.
The coverage report is 0 verified of 726 specimens after expanding the
first Button family records. `button/usage/rest-geometry` and
`button/usage/press-motion` are `implemented-unverified`: WASM `btn-usage`
was recaptured at artifact `6067b2e3` (PNG bytes identical to the
`e2bfda29` set), upstream fixture captures were reused, and press motion
is test-backed only. HEAD native recapture is blocked in the agent host
(control-file ack never fires unfocused; `screencapture -l` / computer-use
cannot photograph the unsigned gallery binary). Named deviations: Geist vs
Inter advance width, porabuild vs HeroUI tokens. Next: HEAD-native
`btn-usage` from a Screen Recording-granted terminal, then press-ramp
visual samples before any `verified` row.

## Implemented

- Shared field chrome (`anim::field_chrome_ramp`) now interpolates InputOTP
  slot fill/border (150ms ease-smooth) and focus ring (150ms ease-out) plus
  NumberField group hover/focus/invalid chrome on the same keyed ramps,
  resuming from the painted frame and snapping under reduced motion. Slot
  transitions and `.number-field__group transitions` metadata rows are
  Implemented; NumberField stepper `:active` scale of custom icon children
  remains Partial. `fields`, `hover_overrides` and gallery metadata tests
  cover the contract.
- Tabs keep the pinned `h-8` (32px) box. Constrained labels wrap and overflow
  lines paint past the pill because the v3.2.5 sheet ships no truncate,
  nowrap or overflow utility; those lines stay pointer-inert in the port.
  `.tabs__tab` metadata is Implemented. The wrapping-labels specimen and
  `tabs_vertical_labels_wrap_inside_the_pinned_fixed_height` lock it.
- Table columns narrower than their content now share one measured track
  triple across header and body on native GPUI, so a 320px table overflows
  as one aligned grid (`tbl-narrow-width`). wasm32 still cannot take
  intrinsic minima (prepaint reports a flex child as the whole track), so
  web tables stay fluid unless a column width is pinned — recorded Partial
  under `.table__content`. `border-separate` spacing remains Partial.
- Select's clear-button pressed target now scales about the 24px slot center
  (`select_clear_deep`). Custom child subtree scaling is `platform-limited`
  because gpui-pre 0.3.3 exposes paint transforms only on Svg.
- `.shots/native_capture.py` is the macOS counterpart of the Windows capture
  drivers (plan §5.2): same control/ack protocol, `screencapture -l` instead
  of PrintWindow. CI runs `test_native_capture.py`. The Button Usage pilot
  set lives at `docs/parity/evidence/native-button-usage/`.

- Table selection now follows the pinned React Stately collection contract for
  `selectionBehavior`, `defaultSelectedKeys`, and `disallowEmptySelection`.
  Replace mode selects a row when keyboard focus moves, collapses a plain
  multi-selection activation to the focused key, and still honors Shift ranges
  and platform non-contiguous modifiers; explicit selection checkboxes retain
  toggle semantics. The default seed is kept in keyed uncontrolled state, and
  Escape/final-key toggles are guarded when empty selection is disallowed.
  `table_deep` covers focus replacement and the empty-selection guard, and the
  gallery's `Selection Behavior` specimen exercises the live contract. Table
  `keyboardNavigationBehavior="tab"` remains separate because arbitrary cell
  children need a tab-cycle seam that pinned GPUI does not provide.
  Browser table tracks also stay fluid across selection rerenders: the
  platform's prepaint callback can report a flex child as the whole track, so
  intrinsic minimum feedback is retained for native GPUI and disabled for the
  wasm canvas path unless a column has an explicit width.

- ListBox and Select option rows now use the pinned HeroUI pressed contract:
  enabled rows ease to a 98% scale over 250ms with `ease-out-quart` inside a
  stable rounded slot, preserving the measured row width, focus ownership and
  trailing indicators during pointer presses in natural and virtual lists.
  The shared `LIST_ITEM_PRESS` timing, source regressions, gallery prose,
  reference metadata, `llms.txt`, and browser artifact are synchronized at
  `898bf0269260d8c3d4373827234607398e36b0045cbdb6ef7366aabae742f110`.

- Calendar and RangeCalendar navigation buttons now interpolate their enabled
  hover fill through the pinned 100ms ease-out transition while preserving the
  stable focus owner and deep press scale. DateField and TimeField groups now
  interpolate enabled, unfocused hover fills through the pinned 150ms
  ease-smooth transition; border and focus/invalid shadow endpoints remain
  direct so state precedence is unchanged. The focused calendar,
  range-calendar and text-field suites pass; gallery metadata and `llms.txt`
  record the remaining partial shadow/opacity transition scope. The rebuilt
  artifact is `df3b50e4d27f25674e519e7a88b214fc5276063308ee0615edf8f44d3d41acf9`.

- The development-only matched-fixture runner now exists at
  `web/src/app/fixtures` per plan §5.1 and [fixtures.md](fixtures.md): a
  notFound()-gated route (development builds only) renders `@heroui/react`
  3.2.5 compositions mirroring the `btn-usage`, `sel-main` and `md-size-md`
  gallery specimens at a fixed stage size, with a `theme` wrapper that is
  isolated from browser theme state and an evidence strip recording the
  fixture, theme, stage size, resolved package version and lockfile hash.
  `button-usage` (light) and `select-usage` (dark) were captured in the
  browser; the composition centers to sub-pixel accuracy and the typecheck
  and lint gates pass.
- The first three matched capture sets now exist under
  `docs/parity/evidence/matched-button-usage/`, `matched-select-usage/` and
  `matched-modal-open/` (plan §5.1/§5.3): pinned-fixture upstream captures
  and WASM-gallery port captures in both themes at DPR 1, side-by-side and
  raw-difference images, per-side provenance JSONs (artifact `e2bfda29…`,
  `@heroui/react@3.2.5`, lockfile `96fe7abea3effa8a`) and READMEs that name
  their measurement methods. Measured geometry: the Select trigger is
  256×36 on both sides in both themes; the Button is 89.08×36 upstream vs
  90×36 port (label advance width, Geist vs Inter) with the squared
  top-left confirmed both sides; the Md modal panel is 448 wide on both
  sides with a 4–6px height difference flagged for review. The Modal set
  documents that the web bootstrap cannot open overlays by URL (native-only
  `HEROGPUI_OPEN_OVERLAYS`/control file), so its open state was reached with
  one real CDP click per capture — a harness gap recorded for §5.2.
- Anchored Badge placement now follows the pinned `.badge--{placement}`
  quarter-box translate (`translate(±25%, ∓25%)` of the badge's own resolved
  box, `badge.css:91-109`): a `PlacedBadge` element pins the corner in
  layout and offsets the laid-out badge by its measured quarter-box in
  prepaint, replacing the old min-box fixed inset whose GPUI-0.2.2
  justification no longer applies. Grown labels ("999+") now overhang
  proportionally at all four corners; the four placement metadata rows
  flipped to Implemented, and a keyed `badge-placements` gallery specimen
  exercises the grown-label case. Badge and feedback suites pass; full
  component gate rerun green.
- Button, standalone ToggleButton and CloseButton now run the pinned press
  transitions instead of snapping: `transform 250ms ease-smooth` (CloseButton:
  `ease-out-quart`, `cubic-bezier(0.165, 0.84, 0.44, 1)`) beside the
  100ms `ease-out` background track, with per-size pressed scales
  0.97/0.98 (sm)/0.96 (lg) and CloseButton's 0.93. `anim::
  pressed_with_background_ramp` keeps the stable press slot (id, hit
  testing, focus ring) while a keyed listener-free visual child tweens both
  tracks; mid-press release resumes from the painted frame and reduced
  motion snaps per `motion-reduce:transition-none`. Grouped buttons keep
  the instant swap (their pinned group CSS is `transform: none`), and the
  three press-transition metadata rows flipped to Implemented. The full
  component gate, `anim_audit` and the parity report pass, and the
  cache-busted Button preview renders and settles cleanly through a real
  press with the synchronized artifact
  `f94ec8bf35e62f7a70ace272ca931142795310d331058b0d5fe3f54593751ac3`.
- `Placement` now names the full 22-value React Aria union v3 forwards
  (verified against the pinned `react-aria@3.52.0` `useOverlayPosition`
  types): the four sides' centred/start/end forms, the side-with-alignment
  spellings (`left top`, `right bottom`, …) and the logical `start`/`end`
  aliases, which resolve to their left/right pixels in this LTR-only port
  (test-proven by exact bounds equality). `placed_panel`/`placed_field_panel`
  and the Popover/Tooltip/date/color positioners classify placements by side
  predicates; a flip still changes only the primary side while the named
  alignment survives, edge-clamped triggers land on the 12px container
  inset, and motion/arrow stay keyed to the physical side per the pinned
  popover CSS split. Every popover-family consumer (Popover, Dropdown,
  Select, Autocomplete, ComboBox, DatePicker, DateRangePicker, Tooltip)
  reaches the new values with defaults untouched; `TooltipPlacement` became
  the shared enum alias. Six placement metadata rows flipped to Implemented
  with the union size, two count claims reworded, `llms.txt` and the
  checklist gap row updated, and placement/dropdown/date-picker suites
  extended (alias equivalence, pinned edges, flip-preserves-alignment).
  One stale shape audit was updated alongside the concurrent session's
  calendar nav hover fade (the override now resolves into the fade
  endpoint). Full component gate, parity report, extraction and the
  rebuilt artifact are synchronized; the current manifest is
  `df3b50e4d27f25674e519e7a88b214fc5276063308ee0615edf8f44d3d41acf9`.
- The web bootstrap now opens overlay demo state by URL: `?overlays=1` (or
  `true`; any other value ignored, matching the bootstrap's treatment of
  unknown params) funnels through one shared `overlays_requested` helper
  that also serves the native `HEROGPUI_OPEN_OVERLAYS` path and calls the
  same `set_overlays_open` the control protocol drives, so web and native
  semantics cannot drift. Works in component-preview (the constructed
  example's overlay starts open) and full-gallery modes; positive and
  negative gallery regressions cover both. This closes the §5.2 harness gap
  the matched-modal evidence recorded — validated end-to-end after the
  artifact rebuild by loading the Md specimen with `overlays=1` and
  observing the open panel with no input delivered.
- The Md Modal panel now matches the pinned vertical composition: upstream
  measures 448×106 in both themes (24+24 dialog padding, 8 header→body
  margin, `.modal__body`'s `p-[3px]`/`-m-[3px]` pair adding 6 real pixels,
  20px body line box — modal.css:181/235/278/295/333). The port had omitted
  the body's 3px/m−3px pair (AlertDialog and Drawer already carried it); a
  `debug_bounds` regression now asserts 106, the metadata row's old
  "inset is zero" reading is corrected, and the matched-modal README's
  review flag is resolved with a note that its port captures predate the
  fix. A post-rebuild capture of `?overlays=1` measures 105.6 logical px —
  the 0.4px is screenshot-rounding, not geometry.
- Rounded overflow clipping now lives in the renderer shared by every
  component. Exact `gpui-pre` 0.3.3 path forks are selected through
  `[patch.crates-io]`; `ContentMask` carries corner radii through the native
  and web scene paths, and WGPU/Metal/Windows evaluate the rounded shape per
  fragment so narrow gradient strips cannot turn a circle into a square.
  Spread-shadow geometry now expands the painted corner radius with the
  shadow's spread, so shared focus rings and offset gaps keep the same curve
  as the control instead of becoming square at the outside edge. A renderer
  regression checks both the expanded shadow radius and the unchanged control
  radius; the fix is recorded in the versioned GPUI patch and applies to every
  component that uses a focus shadow.
  The regression covers nested masks and ColorArea RGB strips, the storage and
  WebGL shader variants validate, and the rebuilt browser ColorArea capture
  shows rounded corners on both HSB and RGB surfaces. The fork/rebase record is
  [documented here](../upstream/gpui-rounded-content-mask.md), with exact patch
  files under `docs/upstream/patches/`.
- Grouped text fields now leave corner ownership to the shared `InputGroup`
  shell. The inner `Input` follows HeroUI's `.input-group__input` contract with
  a transparent `rounded-none` middle, preventing a second radius from
  exposing inner corners when a prefix, suffix or custom fill is present. The
  source regression and InputGroup behavior suite pass; the synchronized
  browser artifact is
  `e95c8c06d6030c896fc5c622d3b4ee9c35512104ae12e1883913ce2cb440514b`.
- TextArea row sizing now enforces HeroUI's 38px minimum for `rows(0)` and
  `rows(1)` while retaining the 20px-line plus 16px-padding formula for larger
  values. The gallery exposes a one-row floor specimen beside the six-row case;
  the source regression and generated website data are synchronized with the
  rebuilt artifact
  `e95c8c06d6030c896fc5c622d3b4ee9c35512104ae12e1883913ce2cb440514b`.
- DatePicker and DateRangePicker popovers now use the shared viewport-bounded
  positioner. Their rounded `p-3` panels own a vertical scroll surface, cap to
  the available side of short windows, and keep per-instance scroll ids so
  multiple pickers cannot share state. New placement tests exercise both
  pickers in a 180px viewport; metadata and generated reference data are
  synchronized after the implementation.
- Retained DatePicker and DateRangePicker exits now carry an internal inert
  state through Calendar and RangeCalendar. Cells, navigation buttons, year
  pickers, focus stops and keyboard handlers are removed while the 100ms exit
  remains painted at full visual opacity. Close regressions now press the old
  navigation target during the exit and prove that the date state does not
  move; the implementation and generated metadata are synchronized with the
  rebuilt artifact
  `e95c8c06d6030c896fc5c622d3b4ee9c35512104ae12e1883913ce2cb440514b`.
- Slider and ColorSlider filled ends now paint a full cross-axis cap under the
  clipped inset, so the default medium rail keeps HeroUI's rounded-xl ends
  while custom `sx` corner overrides still own their specified corners. The
  geometry, pixel-mask and browser slider previews pass with the synchronized
  artifact `64fd41b6f5cdf5b323c15c14cdc58852f2d4cfd22191b8de54316862a7db4c7b`.
- Navigation current-page semantics now use the local `gpui-pre` 0.3.3 patch
  rather than dropping `aria-current`: the fork adds an `aria_current` builder
  backed by AccessKit 0.24, stores the state in `AriaProperties`, and writes it
  into the accessibility node. Breadcrumbs marks its last crumb and Pagination
  marks its active page with `AriaCurrent::Page`; the source contracts,
  reference metadata, `llms.txt`, generated website data and WASM manifest are
  synchronized. The Pagination and Breadcrumbs previews were rebuilt and
  visually checked with no browser errors. All four checked-in gpui-pre patch
  files also pass a dry-run against fresh 0.3.3 registry sources, so a future
  dependency refresh has an explicit reapply path. The artifact is
  `1b538a8210660fae969c233a9a2948e5cf16383d7e30fac2e6b78a9483618196`.
- Calendar and RangeCalendar year-picker indicators now share a keyed
  listener-free SVG animation: one down chevron rotates through HeroUI's
  90-degree angle over 150ms, reverses from the painted frame, and settles
  immediately under reduced motion. Source contracts and the calendar deep suite pass;
  Calendar and Range Calendar Year Picker previews were visually checked in
  closed/open states with no browser diagnostics. The artifact is
  `8f958084984a4e74a1c71eb96da02f43a24a8762aa08a0514d87d2496fea4f2b`.
- Pressed Button skins now carry the resting per-corner radius onto their stable
  press slots, so the keyboard focus ring follows rounded controls and partial
  ButtonGroup seams instead of becoming a square. Tooltip text keeps short
  placement labels such as `Left` on one line while still adding zero-width
  break opportunities to content that exceeds the 320px cap. The shared animation
  and tooltip unit tests pass; rebuilt Placement and Long Content previews show a
  rounded focus ring, an unbroken `Left` tip, and a two-line long URL without
  browser diagnostics. The artifact is
  `7addc654ef9d59758616dfe2d125334eaf326e3fcb940e7ee02dde3a63e852fc`.
- Slider endpoint fills now apply the same rounded clip to the cap container and
  its painted child, preserving the HeroUI radius at the blue start edge in both
  orientations and custom-corner cases. Tooltip placement anchors now span the
  trigger and center the panel before the side offset, so a long top tooltip's
  arrow remains over its button. Slider geometry tests and cache-busted Slider,
  Color Slider, and Long Content previews pass with no browser diagnostics. The
  artifact is
  `73a40295640fc9faecfde03166cb7aad2fe57326a67f36c8501f044de1547772`.
- Accordion closed-trigger hover now follows the pinned HeroUI foreground
  3%-mix (or Surface `bg-default`) endpoint through the shared 150ms ease-out
  fill transition. The fill is installed before the title and indicator so
  their hit testing, focus ring and layout remain stable; open and disabled
  rows still do not hover. The Accordion source contract, website extraction,
  and cache-busted WASM artifact are synchronized at
  `112ff2182abaec1803406e1fdf6e31081260d4132b2615d22d0f5c3b1642a7f3`.
- NumberField's enabled, unfocused group hover now interpolates its background
  through HeroUI's 150ms `ease-smooth` transition while retaining the immediate
  border endpoint and the existing stable focus, press and repeat listeners.
  The NumberField source contract, focused stepper tests, website extraction,
  and rebuilt WASM artifact are synchronized at
  `1f2fe0a154dac5e8d36c5666a53c415aa9f8146c8d15f774f1fe545be89b8094`.
- ColorSlider endpoint caps now round both the outer slots and their nested
  checker/color paint children. This closes the GPUI renderer path where a
  saturated Hue strip could leak square pixels beyond the track's rounded
  corners. Horizontal and vertical cap source contracts plus color geometry
  tests pass; the rebuilt artifact is
  `73a40295640fc9faecfde03166cb7aad2fe57326a67f36c8501f044de1547772`.
- Autocomplete trigger hover now interpolates the field background with the
  pinned 150ms `ease-smooth` fill transition, keeps the border endpoint
  separate, and drives a suppression-aware fade while the nested clear button
  is hovered. Autocomplete hover-token and clear-button interaction tests pass;
  website data and the rebuilt WASM artifact are synchronized at
  `e07918213838f81a44ab652e20c7a288c993e9ee2b51f7ce09855d9d95d9ec7f`.
- Spinner now uses HeroUI v3.2.5's `animate-spin-fast` default (750ms linear)
  and the pinned two-segment gradient arc asset, while the existing duration
  extension keeps deterministic speed specimens available. The default-speed,
  size/color, reduced-motion, and constrained-flex tests pass; the rebuilt
  Spinner Usage preview has no browser diagnostics. The synchronized artifact
  is `3c0375dd7581c1d31bc9721493a10eb5165e30e89242fa38a0e1c6c78ce07f01`.
- ColorField's static display group now follows the pinned enabled hover
  contract: primary fields fade to `field-hover`, secondary fields fade to
  `default-hover`, and the field-hover border endpoint remains immediate.
  Focused, invalid, disabled, and bare paths retain their existing chrome
  precedence. The ColorField source regression, color-field behavior tests,
  generated website data, and cache-busted Read-only Display preview pass with
  no browser diagnostics. The synchronized artifact is
  `135e64bf50222ed2d657f6687fa4d56f6e9e702ba214af9ca0ecd92c2d356ad4`.
- ColorPicker's trigger focus shadow now uses the shared keyed 150ms color-ring
  tween, preserving trigger identity and reduced-motion endpoints while the
  existing open/dismiss behavior remains intact. The focused source regression,
  color geometry suite, generated reference data, and cache-busted Usage preview
  (keyboard focus plus an open panel) pass with no browser diagnostics. The
  synchronized artifact is
  `04c5aaad473b5b6f0eca089605821e81fba9b4b98aa3ef791f55855bc5b46ff1`.
- Input's standalone field chrome now interpolates enabled, unfocused primary
  and secondary fills through HeroUI's 150ms `ease-smooth` hover transition,
  with the border endpoint remaining immediate and custom theme backgrounds
  preserved. TextArea inherits the same fix through Input. The shared source
  regression, Input/TextArea behavior tests, generated reference data, and
  cache-busted Input focus and TextArea previews pass with no browser
  diagnostics. The synchronized artifact is
  `a1e0ae1c2eac15a5359b5d6bd31085835e50ddf8d4e047bbbb97f3b4231341fe`.
- InputGroup now paints HeroUI's field-focus endpoint for focus-within and
  invalid states, suppresses hover while either state is active, and eases the
  enabled primary/secondary hover fills over 150ms with `ease-smooth`. Prefix
  and suffix wrappers own the pinned field-border seams, including arbitrary
  icon/button addons, while textarea groups retain their top alignment. Every
  animated field fill now carries the owning radius into its overlay, closing
  the square-corner leak during hover transitions across Input, InputGroup,
  TextArea, ColorField, Autocomplete and NumberField. Source contracts,
  renderer mask tests, generated website data and the rebuilt artifact are
  synchronized at
  `1230ada96e7b7d949832e3cab951ba2ba51180bdacab456cf84edd0cb2c5f72b`.
- ColorField invalid chrome now paints HeroUI's field-focus surface endpoint
  for primary and secondary variants while retaining the danger border/ring.
  The source contract, static invalid geometry test, generated reference data,
  and rebuilt WASM artifact are synchronized at
  `90c4bb3593568de27244fe1e48fd9bc08262235ef2048a076db2e760630a584d`.
- Select's enabled, unfocused trigger now eases its primary or secondary hover
  surface over HeroUI's pinned 150ms `ease-smooth` transition, keeps the
  border endpoint immediate, and suppresses the parent fill while the nested
  clear affordance is hovered. The overlay carries the trigger's resolved
  radius so the shared renderer clip cannot leak square pixels. The source
  contract, picker tests, generated website data, rebuilt WASM artifact, and
  cache-busted Select Usage preview (closed and open states) are synchronized
  at `476fd3b395a57f767b52d11f2854517744246353165d229e6c7bd46c5a4dc4c2`.
- Canonical Input and InputGroup Usage specimens now use the HeroUI default
  `--field-radius` instead of an accidental 4px demo override. The pinned
  v3.2.5 default remains 12px (`--radius * 1.5`); explicit `.radius(...)`
  builders remain available for the separate customization paths. Website
  extraction, WASM manifests and the rebuilt InputGroup Usage preview are
  synchronized at
  `476fd3b395a57f767b52d11f2854517744246353165d229e6c7bd46c5a4dc4c2`.
- ToastViewport now clamps its configured desktop width to the live viewport
  minus both edge insets, matching HeroUI's narrow-window `calc(100vw - 2rem)`
  behavior while preserving the 460px desktop default. Toast cards now enter
  and frontmost exits translate toward the physical top/bottom placement edge
  with the pinned 350ms ease-out-fluid motion; non-frontmost exits use the
  200ms stack scale path, with an independent 150ms opacity track matching
  the pinned CSS. The Toast Usage specimen uses the uncustomized HeroUI card
  defaults; explicit padding, radius and close-hover extensions remain
  available in named customization paths. The remaining motion difference is
  the browser-only view-transition wrapper. The source contracts, overlay
  metadata, generated site data, rebuilt WASM artifact and cache-busted Toast
  Usage preview are synchronized at
  `ad27bf2091fdd0a5d342a1a4e8f518175aeaea5bf2e113d793eaa3a08b6b61b9`.
- The complete date-time gallery batch now has stable specimen keys. DateField's
  usage, compact box, granularity variants, leading-zero variants, primary and
  secondary variants, surface, description, required, disabled, full-width,
  invalid, controlled, bounded-validation and form examples are addressable.
  Calendar covers locale, selection, constraints, unavailable, navigation,
  visible-duration, disabled/read-only and year-picker cases, including grouped
  locale and heading-offset variants. DatePicker and DateRangePicker cover
  locale, disabled, controlled, invalid, format, form, custom-indicator,
  render-function and open/closed usage cases. RangeCalendar covers range
  constraints, indicators, unavailable predicates, visible durations,
  read-only/invalid/focused states and heading offsets. TimeField covers hour
  cycles, seconds, locale/forced padding, field variants, validation,
  controlled and form cases. Grouped loops render only the requested key in a
  component preview, while the full gallery still renders every example.
  Website extraction and the cache-busted WASM artifact were regenerated; CUA
  observed the `df-secondary`, `cal-hebrew` and `in-focus-ring` previews. The artifact is
  `305da02d3a17b239e71bdeb22d6f3d503543f8c0fcb2aa4041eec9b7f826bd81`.
- The shared platform text editor now clears a collapsed selection anchor
  after each inserted character. A multi-character browser/IME edit therefore
  keeps every character instead of treating the second character as a
  replacement for the first. The focused component regression covers a
  platform replacement of `Go`; a plain Input and the exact WASM ComboBox
  `typeText("Go")`, immediate Down, Return burst both finish with `Go` in the
  field and value display. Temporary browser diagnostics were removed after
  the fix, and the artifact above includes the correction.
- Field focus chrome now has a separate `focus_ring(bool)` configuration path.
  `false` suppresses only the visual focus ring while preserving focus,
  editing and invalid feedback; the setting is shared by `Input`, its text
  wrappers, grouped fields and picker/date/color field boxes. The component
  theme can set the shared Input default through `TextFieldStyle::focus_ring`,
  and the Input gallery has a keyed opt-out specimen.
- Field validation rows now share a retained, measured feedback panel. Input,
  InputGroup, NumberField, InputOTP, CheckboxGroup, RadioGroup and Switch keep
  the last error mounted while it exits, animate opacity over 150ms ease-out
  and height over 350ms ease-smooth, and retarget from the painted frame when
  validation changes quickly. Standalone `FieldError` can opt into the same
  lifecycle with a stable `.id(...)`; no-id one-shot compositions retain their
  allocation-free static behavior. The focused field-slot, text-field,
  number-field and choice suites pass, and the metadata/API docs record the
  extension.
- Rounded clipping and focus geometry now share one root contract across the
  interactive surfaces. The patch-backed `gpui-pre` renderer preserves each
  ancestor's original rounded mask through nested overflow and animated child
  layers, so ColorArea/ColorSlider caps, field fills and pressed skins cannot
  leak square pixels. Focus rings are applied to the measured interactive
  target after press/hover wrappers, preserving the target's size and resolved
  per-corner radius; Toast and dialog close controls therefore ring around the
  20/24px rounded button instead of an oversized square. Button press reversal
  resumes from the painted frame, while reduced motion snaps without adding a
  transition. Rounded-mask, radius, button, overlay and full component suites
  pass, and cache-busted ColorSlider, Input, Tooltip and Toast Close Reveal
  previews were visually checked. The rebuilt stable WASM artifact is
  `f94ec8bf35e62f7a70ace272ca931142795310d331058b0d5fe3f54593751ac3`.
- Button pressed styling now follows HeroUI's `--button-bg-pressed` endpoint
  for every variant and paints it through the scaled press skin. The previous
  whole-button `opacity(0.85)` active shortcut was removed; grouped members
  keep the endpoint while still suppressing standalone scale. A source contract
  regression covers the endpoint and prevents the opacity fallback. The rebuilt
  artifact is `08e34888f68039db98999ae15014b4a97c77b679f53fb20cbdcd23d3a3302a3a`.
- Select option/value styling now follows the pinned HeroUI contract: selected
  rows keep normal foreground and use the default check indicator, keyboard
  option focus uses the shared status-ring overlay without a layout-shifting
  border, and long natural-height labels wrap in both the trigger value and
  non-virtual list rows. The built-in trigger chevron now stays mounted and
  rotates over the pinned 150ms transition; `trigger_indicator` exposes the
  live open state for custom trigger content while `indicator` remains the row
  checkmark seam. Keyed `sel-long-values`, `sel-trigger-indicator` and source/
  behavior regressions cover the selected, wrapped, focused and open-indicator
  states. Focused trigger and option rings were observed in the cache-busted
  WASM preview. The rebuilt artifact is
  `fb148a51b3259d6deff9924182ba4af1294d197dd82c702a1c216101a10428b6`.
- The upstream Select/ListBox audit found no ellipsis rule for option labels:
  `Select.Value` uses `wrap-break-word`, while `ListBox.Item` stays in normal
  text flow and the list root owns clipping. Select, Autocomplete and ComboBox
  therefore keep normal wrapping in natural and explicitly virtualized rows;
  fixed `row_height` remains caller-owned geometry rather than a port-added
  truncate rule. The source regression, full picker suite, regenerated website
  data and rebuilt stable WASM artifact are synchronized at
  `936646450ddd181c5781f2bd9183a09742c8bb1864956db2cb3c1b58b2cd5886`.
- Ordinary ListBox rows now paint HeroUI's `scale(0.98)` pressed state through
  the shared stable-footprint press skin; custom `item_content` keeps receiving
  the same live `is_pressed` state. The source regression and focused action
  test pass; the remaining documented Partial is only GPUI's shared press
  curve versus HeroUI's exact ease-out-quart curve. The rebuilt stable WASM
  artifact is
  `0f6e0a4f547e4fc4fb89bb8ea12aa3184dbca396588d9cde1a3bba95e42a14ce`.
- The ColorArea visual audit found GPUI's element border is painted after its
  children, which covered a thumb sitting on an edge. The visible border is
  now a rounded child overlay painted before the thumb, while the gradient
  stack keeps its own rounded overflow mask and the root remains overflow
  visible. The cache-busted `ca-color-channels` preview shows both edge thumbs
  fully above the border; the focused ColorArea and mask suites pass. The
  rebuilt stable WASM artifact is
  `a431629966496871e30c786e3e6c471b0c86b24575fd2be3c762b5cf271c0f5e`.
- The same paint-order correction now covers ColorSlider tracks: the border is
  a rounded overlay child and the thumb is appended afterward, so an end thumb
  cannot be painted beneath the track border. ColorArea and ColorSlider source
  regressions plus the color geometry suite pass; the cache-busted ColorSlider
  preview shows the thumb above the gradient chrome. ColorArea's overlay also
  carries an explicit `sx(border_color(...))` override, preserving the public
  root styling contract after the paint-order fix. The rebuilt stable WASM
  artifact is
  `2ca79bffeb3ba206654f1c3c61b3ef306f7ca201fe4eb8d99377d43f665d670a`.
- ColorSwatch and the alpha ColorSlider now share an 8px checker-cell helper,
  so translucent values are composited over HeroUI's transparency pattern in
  both horizontal and vertical slider tracks. The cache-busted Alpha and
  ColorSwatch previews visibly show the checkerboard; the source contract,
  color geometry, and rounded-mask suites pass. The rebuilt stable WASM
  artifact is
  `17a241c2ba001ceca67119b5f21031233cced2a098e41fbe97d3be623bf0c418`.
- ColorSlider now paints separate rounded start/end cap layers over the
  full-length gradient. The endpoint colors align with the existing 10px
  pointer inset, and alpha caps retain the checkerboard backing. The alpha
  checker dimensions are axis-aware, so a vertical track receives the full
  checker tile instead of a horizontal strip. Horizontal, vertical, and
  alpha specimens were visually checked in the rebuilt gallery; the cap source
  contract and color geometry suites pass. The rebuilt stable WASM artifact is
  `183e9687fb998b0d9b4d75311886674157c203f3ddad744548e00ff7f8252731`.
- ColorArea, ColorSlider and ColorSwatch now carry HeroUI's translucent depth chrome as
  shared GPUI shadow layers: the area edge, directional slider-track edges,
  and both outer/inset thumb hairlines. The existing child-overlay ordering is
  preserved, so the depth layers cannot cover an edge thumb. ColorSlider thumb
  movement now follows the pinned 250ms ease-out transition through a stable
  visual child, and its open/closed hand cursor follows the live dragging
  state. Color-picker unit, picker behavior, metadata and generated-data
  checks pass; horizontal, vertical and area previews were rebuilt for visual
  verification. The current stable WASM artifact is
  `376ddca72895bd48fe051c0dd7ab6936ab945d54b52bd9295d1336414f5328a7`.
- ColorSwatchPicker now animates the built-in swatch hover/selection scale over
  100ms, fades the selected border through the same ease-out path, and reveals
  its built-in or custom indicator over 150ms. ColorSlider and
  ColorSwatchPicker focus-ring shadows now interpolate through the pinned
  150ms/100ms ease-out paths as well, with the shared color motion helper
  preserving depth shadows and reduced-motion endpoints. Stable item owners
  retain focus, pointer and render-prop state while the listener-free visual
  children animate. The scale/focus precedence, metadata, gallery extraction,
  picker behavior suites and all parity readers pass; ColorArea, ColorSlider,
  ColorSwatchPicker, and the keyboard-open ColorPicker popover were visually
  checked. ColorPicker entry motion now interpolates the pinned four-pixel
  resolved-placement slide; the shared positioner feeds flips and resizes back
  into the animation, and the controlled-open specimen was visually checked
  in the browser. The current
  stable WASM artifact is
  `7aa3256fa109302648720add06ae79771fc3ce59ed92af5b022f58ad96130d3d`.
- Toast stacks now use measured card heights and absolute slots once the
  first layout pass has recorded them. Expanded cards offset by the measured
  heights plus the configured gap; collapsed non-front cards reuse the
  front-card height and clip their content while retaining the scale-width
  approximation. The focused stack suite now verifies the rendered slot
  geometry, the Toast metadata marks the index and non-frontmost contracts as
  implemented, and the gallery exposes a stable `toast-expanded-stack`
  specimen. CUA checked the rebuilt specimen in both collapsed and forced-open
  states: the front card remains the only visible card while collapsed, and
  the expanded cards use their measured heights with a consistent 12px gap.
  The rebuilt stable WASM artifact is
  `3aef3b172d276426af387b2d7c42b26886dd2790440e2c96175746bbcf3b1ef2`.
- Table secondary headers now paint HeroUI's surface-filled first/last rounded
  cells, secondary body cells carry the tertiary separator on every row, and
  header columns draw the short separator edge. Measured intrinsic minima keep
  long body content on shared header/body tracks. Built-in sortable indicators
  use the shared 100ms rotation primitive while custom indicators remain
  caller-owned. Table source, metadata, gallery and focused interaction suites
  pass; cache-busted secondary and usage previews were rebuilt for visual
  verification. The
  stable WASM artifact is
  `f23d707c6cd5e7b71e43f14bbe6b99b74288adbc39e1fea04157bb9ec1e13652`.
- Table hover and selection now paint on each cell through a row-scoped hover
  group, matching HeroUI's cell clipping and keeping selected cells above the
  hover wash. Focused rows now use split top/bottom/outer-edge strips per cell,
  preserving rounded outer corners without a row-wide overlay bleeding through
  shared boundaries. Table source tests, metadata extraction and the rebuilt
  WASM artifact are synchronized at
  `64fd41b6f5cdf5b323c15c14cdc58852f2d4cfd22191b8de54316862a7db4c7b`.
- Pagination now follows HeroUI's responsive `sm` shell: below 640px the
  summary and page controls stack in a column and align to the start, while
  wider windows retain the horizontal row. The breakpoint helper, narrow-width
  source tests, metadata, generated website data and rebuilt WASM artifact are
  synchronized at
  `64fd41b6f5cdf5b323c15c14cdc58852f2d4cfd22191b8de54316862a7db4c7b`.
- Tabs now fade unselected-tab and overflow-arrow hover endpoints through the
  shared stable-element motion while preserving tab focus, indicator geometry,
  arrow occlusion and existing reduced-motion behavior. Tabs now compose the
  shared ScrollShadow primitive for 64px fading edges, one scroll handle for
  wheel input and arrows, hidden scrollbars, and full-height vertical lists.
  Tabs/theme-token tests, metadata extraction and the rebuilt WASM artifact are
  synchronized at
  `64fd41b6f5cdf5b323c15c14cdc58852f2d4cfd22191b8de54316862a7db4c7b`.
- Indeterminate ProgressBar animation keys are now scoped beneath each bar's
  element id, so sibling bars animate independently instead of sharing one
  global timeline. The source regression, metadata extraction and rebuilt
  WASM manifest are synchronized at
  `482be9c1f4c36e80ecd8fc2cfc8a0f30a26c67dd9c7ecc2b8cc5af409df5b582`.
- Toast close affordances now remain hidden until the frontmost card is hovered
  or keyboard-focused, then fade through the pinned 150ms ease-smooth opacity
  track. Hidden controls reject pointer activation, reduced motion settles the
  endpoint immediately, and the `toast-close-reveal` specimen covers the
  interaction in the gallery and generated website data. The close hit target
  keeps the resolved small radius, so its keyboard focus ring follows the 20px
  rounded control instead of the square outer box; a source-shape regression
  pins the radius on the focus owner, and a cache-busted browser pass tabbed to
  the revealed control — ring rounding with the card corner — and dismissed the
  card with Enter. The synchronized
  WASM artifact is
  `0cd90860665825af3f83386e2219656f701ae31d8571c3a262b6ef97271753c2`.
- ColorField's suffix row now cites the pinned `.color-input-group__suffix`
  selector where it is drawn, closing the part audit's last unverified part;
  the parity report is fully green again.
- ColorField now accepts HeroUI's nullable `Color | null` value contract. A
  controlled `None` renders an empty field, `default_value(None)` seeds an
  uncontrolled empty field, required form data treats it as missing, and the
  built-in swatch is omitted until a color exists. Focused ColorField tests,
  the Empty Value gallery specimen, generated website data, and a cache-busted
  browser capture are synchronized at
  `e2b62faabf767b9f8b018793b257ae982da5a52b8e99263226c5ea6f866222b9`.
- CloseButton focus now paints on the rounded 24px button surface itself, so
  Alert's composed close control keeps a compact rounded focus ring instead of
  an oversized square. The CloseButton focus geometry test and a cache-busted
  Alert Closable browser capture confirm the shared fix.
- ColorSwatch now supports HeroUI's `colorName` accessible-name override while
  retaining the canonical hex fallback. The source contract, Accessibility
  gallery specimen, generated website data, and rebuilt WASM artifact are
  synchronized. Cache-busted Accessibility and Transparency captures show the
  named specimen and clipped translucent swatches without corner leaks. The
  artifact is
  `346a0ae52b407fab5fa6fb8ad0ca6ecc9c94bc8a60b1f70680af088f229f5cf2`.
- ListBox custom item render props now keep their press listener on the stable
  press slot after the subtle 0.98 skin wraps the row. The previously failing
  pointer press regression and all 39 render-prop tests pass; built-in rows
  retain the same HeroUI press geometry.
- Autocomplete's default selected-value slot now follows the same pinned
  `wrap-break-word` contract as Select. Long labels grow the trigger instead
  of being ellipsized; a keyed `ac-long-value` specimen, source contract test,
  geometry regression and generated website example cover the change. CUA
  observed the wrapped value in the rebuilt preview. The current artifact is
  `585472b3147c3cdbe6922f85ade03a3f8e9df16c18a761c4a71c635574b0f26f`.
- Autocomplete's clear affordance now keeps its stable 20px hit target while
  matching HeroUI's immediate empty-state hide and keyed 150ms ease-smooth
  fade-in when a selection reappears. The visual child owns the animation,
  reduced motion snaps it, and the source/hover suites cover the existing
  pressed-scale and pointer-inert behavior. The cache-busted WASM preview was
  visually checked after the rebuild; the current artifact is
  `aed3915e043837595e102f5ef60baf2595d6970cc90008bba058b94979ceb743`.
- The Autocomplete item-indicator behavior test now treats repeated render
  passes as valid while requiring every complete row pass to report the pinned
  `[selected, unselected]` state sequence. This keeps the test aligned with
  keyed placement settling without hiding a wrong row state; the complete
  `herogpui-components` suite and all parity readers pass.
- ComboBox's optional `Value` render-prop default content now follows the
  pinned `wrap-break-word` rule with a released width floor. A keyed
  `cb-long-value` specimen, source contract test, measured value-row
  regression, metadata assertion and generated website example cover the
  wrapped selected label. CUA observed the two-line value row in the rebuilt
  preview. The current artifact is
  `d8992557a1442106c950c30d5d476b1141e6d6a38e39789508dd3abfe7b8fd6c`.
- Vertical Tabs keep the pinned 32px tab box. Long labels wrap inside the
  constrained panel instead of widening the list; overflow lines paint past
  the pill because the v3.2.5 sheet ships no truncation utility. The
  vertical chevrons keep their 80%-of-viewport step and remain on top of
  the list. Native regressions cover constrained wrapping and overflow hit
  testing; the gallery Wrapping Labels specimen shows both orientations.
  The wrap-floor work first landed with artifact
  `898bf0269260d8c3d4373827234607398e36b0045cbdb6ef7366aabae742f110`; the
  2026-09-14 wave resynchronized at
  `6067b2e3e8810b32b8fc5af431b4f5a2e1e40825b2bee6aed7fa31fe19e7cfce`.
- Calendar and RangeCalendar day-state styling now follows the pinned calendar
  sheet for today, hover and selected/pressed endpoints. Unavailable and
  out-of-range cells also carry disabled opacity and the operation-not-allowed
  cursor while remaining keyboard-navigable where React Aria allows focus;
  line-through remains limited to disabled dates. Metadata, reference tests and
  the unavailable-focus regression now describe the corrected state precedence.
  CUA observed the cache-busted `cal-unavailable` preview with the ring moving
  onto the dimmed unavailable 13th while the selected/today 12th retained its
  soft accent fill.
  The rebuilt WASM artifact is
  `b621a3f8b88d9d3c661d9b1999f7d730317d78625382bcefe620bc08ab9c022e`.
- Accordion and Disclosure now follow the pinned HeroUI disclosure motion:
  built-in down-chevrons rotate through 180 degrees over a keyed 250ms default
  transition curve, while a shared panel helper measures natural content height,
  animates height and opacity over 200ms, retains closing content through the
  exit lifetime, resumes from live frames on reversal, and snaps under reduced
  motion. Custom Accordion indicators remain caller-owned. Source-shape,
  metadata and retained-lifetime regressions cover the shared primitive. The
  rebuilt WASM artifact is
  `5935951f36b29e9c33805e794058a0a4e957dcac26b22342ae10ae2cf6ceeb9a`.
- Drawer now follows the pinned sheet geometry for the shared overlay path:
  bottom placement rounds the top corners, top placement rounds the bottom
  corners and applies the stylesheet's `pb-2`/handle override, while the body
  uses the scrollbar-safe `-m-3 p-3` allowance without an extra gap. Drawer
  backdrops use the component-specific 250ms/200ms out-fluid timing. All four
  placement examples are addressable as `dr-left`, `dr-right`, `dr-top` and
  `dr-bottom`; native placement/drag tests, metadata/reference tests and the
  generated website data pass. CUA visually checked the rounded bottom and top
  sheets in the cache-busted WASM gallery. The current artifact is
  `0d422c24e320064cb3bebd4794f85d4a203b5173a86ed61a794fe11fa1e40203`.
- Popover and Tooltip entry motion now includes HeroUI's four-pixel
  placement-specific translation. Popover's shared positioner feeds its
  resolved physical side back through keyed state after a flip or resize;
  Tooltip uses the same side offsets for its four cardinal placements. The
  Popover placement examples (`po-pl-top`, `po-pl-bottom`, `po-pl-left`,
  `po-pl-right`) and the Tooltip placement row (`tt-placement`) are addressable
  in the gallery. Offset unit tests, metadata/reference checks and generated
  website data pass. The current artifact is
  `9c47ec2a82f5653ad382886e01ba07cb995f11eda846a7c1c25296c2d4d82a99`.
- Tooltip now also supports the pinned top/bottom `start` and `end` placements.
  Aligned panels and arrows attach to the corresponding trigger edge while
  retaining the cardinal side motion and centered behavior. The placement
  gallery expands through the shared `TooltipPlacement::ALL` list; metadata,
  public API docs and source contracts are synchronized. The current
  rebuilt artifact is
  `346a0ae52b407fab5fa6fb8ad0ca6ecc9c94bc8a60b1f70680af088f229f5cf2`.
- Tooltip now also supports the four portable side-edge placements from the
  pinned React Aria vocabulary: `left top`, `left bottom`, `right top` and
  `right bottom`. Each placement owns its matching absolute anchor and arrow
  edge, while centered `left`/`right` behavior and the existing top/bottom
  aliases remain unchanged. The placement unit contract, metadata and gallery
  loop are synchronized; logical `start`/`end` aliases remain an explicit
  LTR-only limitation. The rebuilt artifact is
  `5a6d758ff9338084df58c5ac931f2b1e8d0a72d7926a2b2af8d630b61d59ddd3`.
- Table tree-column indentation now follows HeroUI's exact `1rem * aria-level`
  rule. Both the painted cells and intrinsic-width measurement advance by one
  16px step per nested level, with a focused deep-tree regression covering the
  root, child and grandchild positions. The rebuilt Expandable Rows preview was
  visually checked after expanding both parent levels; generated website data,
  manifests and parity readers pass. The current synchronized artifact is
  `d90b72dba2e61b6803785924ac471c62dc1d13990bf006e55313f3af75431854`.
- DatePicker and DateRangePicker now expose HeroUI v3.2.5's composed popover
  `placement` prop through the shared `Placement` builder, defaulting to a
  centered bottom panel while supporting the portable top, aligned and side
  values. Existing click-geometry tests pin their explicit bottom-start
  coordinates, and a placement suite measures both default-bottom and top
  panels relative to their triggers. The new Placement gallery specimens and
  generated website data were rebuilt; native tests and cache-busted browser
  previews pass. The synchronized artifact is
  `efcd2a6ff5951fbb0d33b516779ece769baa990aff0bd0d8debc01007bf989d8`.
- DatePicker and DateRangePicker popovers now follow HeroUI's keyed
  placement-aware entry and retained exit motion: 150ms ease-smooth
  fade/zoom-in-95 with a four-pixel side slide, then 100ms zoom-out-95/fade on
  close. Calendar changes and outside dismissal are gated while the retained
  panel exits, and the retained Calendar/RangeCalendar is now internally inert
  so stale navigation, year-picker and focus handlers cannot consume input.
  Focused placement/offset, short-viewport and close regressions pass. The
  synchronized artifact is
  `b1c7ab6bc6969df9634dd43b7025b8d0ed20acbd1568ac9e9e84eafe784bceea`.
- DatePicker and DateRangePicker now use React Aria's pinned default 8px
  popover offset instead of the earlier 6px approximation. Geometry probes,
  placement metadata and generated website data were updated; the rebuilt
  artifact is
  `b5f715d2a952347db4d3df973798a5de3690ec67dab63263f6f5c91ecbd1cbf3`.
- DatePicker and DateRangePicker now measure their actual field trigger and
  share the Popover positioner, so a preferred bottom/top side flips to the
  opposite physical side when the viewport cannot fit it. Resolved-side
  feedback also keeps the four-pixel entry slide aligned after a flip; native
  placement and edge regression tests pass. The synchronized artifact is
  `978bc63e4c3cfdff4935ef25ebc8a121802b1246d586313d742b888a2cad7048`.
- Modal and AlertDialog now use the same four-pixel placement entry motion as
  HeroUI for explicit top and bottom placements. The shared ModalPlacement
  helper keeps centered and desktop Auto layouts at zero offset, and Full
  Modal retains its zero-slide rule. Focused offset and reference metadata
  tests pass; the stable WASM artifact is
  `b9a2bf92df707cc3125ec33356277ce39969d8b8dc9f7bf19eec5406f2408e11`.
- Select, Autocomplete and ComboBox field popovers now feed their resolved
  physical placement into the shared four-pixel entry translation. Above-field
  specimens (`sel-placement-top`, `ac-placement-top`, `cb-placement-top`) are
  addressable in the gallery; picker metadata, section tests and the stable
  WASM artifact are synchronized at
  `50ecfff821719090a59eeb2387ad999a3caaae7b070780e99cd5ddcec8bc2891`.
- `.shots/interaction_inventory.py` refreshes the existing queue to schema 3,
  includes rendering, package, font, preview, generated-data and upstream source
  inputs, preserves review history and retired rows, and invalidates stale
  specimen verdicts. It now seeds one explicit, unreviewed gallery-section
  specimen for each of the 711 rendered sections, retaining the section's
  component/page/key/source hashes while clearly requiring expansion into
  concrete variant and state cases. Verified evidence must exist and match its
  recorded hashes.
  Writes are atomic, including failure cleanup. Capture/audit/test drivers and
  `.gitattributes` now participate in the source fingerprint. A temporary Git
  checkout regression verifies byte-identical hashed driver files with Windows
  and Unix checkout filters.
- `.shots/parity_report.py` runs every existing parity reader and `write_only`,
  checking output contracts in addition to process status. It fails on missing
  or duplicate counters, empty input, unowned/ambiguous API sections, reader
  diagnostics, nonzero gaps, timeouts and input changes during a run. It records
  full output and effective input hashes, including fetched demo fallbacks, and
  rejects inherited bundle overrides.
- `demo_audit.py` normalizes both moving-branch URL spellings in the pinned docs
  bundle to the target tag before fetching or choosing a cache key.
- `inert_audit.py` separates direct builder chains from nested/sibling callbacks,
  comments, raw strings and test modules. It reads controlled arguments rather
  than configuration in seed expressions, tracks scalar/range Slider ownership,
  and checks callbacks by owner and state axis. Disabled trigger state does not
  exempt an open Select panel from dismissal feedback. InputState seeds and
  DatePicker custom-content paths have explicit source-backed exceptions.
- Slider's Custom Value Formatting examples now opt into uncontrolled state;
  their percentage and currency values can change. The disabled Step example
  has a distinct instance ID. Dynamic controlled range examples now specify an
  editable scalar fallback if the collection becomes empty. The default-value
  reference explains when an interactive example needs an uncontrolled seed.
- ComboBox now reports changed single selections through
  `on_selection_change_all` for pointer, Enter and Tab activation. Re-picking
  the current key remains silent for that value callback while retaining the
  scalar pick notification. Controlled owners can accept or reject the request.
  Read-only rows reject selection in both plain and virtual lists, including
  when the caller keeps the popup open. The Value Render Props example,
  reference metadata and public behavior documentation describe the contract.
- ComboBox now keeps its popup mounted through the pinned 100ms exit, shrinking
  and fading with the shared list motion. Exit rows are visual-only and the
  retained panel does not occlude a later pointer target. The trigger chevron
  uses the shared keyed 150ms rotation, with focused open/close and retained-
  lifetime regressions in `combo_box_open`.
- Toast cards now keep long titles naturally wrapped and anchor their close
  affordance absolutely at the pinned top/end offset instead of consuming
  message width in the flex row. Collapsed cards alone clip overflow; front
  and exiting cards keep the outside close target visible. The placement
  regression was updated to exercise the new hit target, and the keyed
  `toast-long-title` gallery specimen documents the geometry. The rebuilt
  artifact is `fbe7cf3f96a087f12c6a649884b766432e3bf5534880527e4a55b84b05771674`.
- ListBox selected indicators now use the pinned absolute 16px inline-end slot
  with 28px reserved end padding, for both the built-in checkmark and custom
  `indicator` render function. This keeps long labels from pushing the marker
  through flex layout; the focused source contract, collection suite, metadata
  and generated website data cover the change. The rebuilt artifact is
  `1e03bd4b93414359afb6aa61fd3ffcc2ef5f91f6d65bb691fdcbf6e9f879e231`.
- Select, Autocomplete and ComboBox now use the same absolute trailing item
  indicator slot as ListBox; Select's trigger chevron also follows HeroUI's
  absolute end slot with 28px reserved padding. Natural and virtual option
  labels keep normal text flow, and changing row padding moves the label inset
  without moving the marker from the pinned end offset. A fixed virtual row
  height remains caller-owned geometry; the port no longer invents an ellipsis
  rule that HeroUI does not have. Focused picker geometry/source tests and
  reference/API documentation cover all three collection-backed pickers.
- RangeCalendar's comments now identify where the month and linear layouts
  implement the tagged `grid-body` and `grid-row` parts. Existing headless
  geometry tests cover the first-row spacing and row pitch in month, week and
  day modes. No geometry was changed or visual verdict promoted by this fix.
- CI now runs the strict aggregate report, checks interaction-inventory
  freshness and exercises the new reader regressions. It retains generated
  reports, including failed reader output, as a `parity-report` artifact.
  The workflow guide now describes the actual CI jobs.
- Gallery control requests now validate page, section and option values, publish
  atomic acknowledgements/errors, and acknowledge only after the requested
  render and a subsequent frame. Section probes are pure; only included sections
  count as matches, including reference panels. Unknown pages preserve the
  previous view; unmatched sections report failure instead of a blank success.
- `reset=1` replaces the gallery root, clears focus and shared validation/toast
  state, and cancels retained promise tasks before clearing the queue. The
  additive GPUI `Toast::promise_task` API exposes native task ownership while
  `Toast::promise` retains HeroUI's detached completion/upsert semantics.
  Promise examples, metadata, `llms.txt` and the narrow extension audit entry
  describe that distinction.
- Component previews now mount the same configured ToastViewport as the full
  gallery. This fixes invisible notifications from otherwise working preview
  buttons and resets the viewport's shared interaction state after root changes.
- PowerShell batch/smoke use atomic requests, exact sequence matching and real
  frame acknowledgements, including isolated retries. Their process lifecycles
  stop/wait/dispose and remove result files on failure; child environment
  settings leave the calling session intact. CI runs the helper regressions on
  Windows. The gallery/workflow guides document the protocol and its limits.
- Form reset closures now hold weak references to the live form state that
  stores them. This removes self-retention in Slider, ColorField, ColorSlider,
  Checkbox, CheckboxGroup, Switch, RadioGroup, Select, Autocomplete, ComboBox,
  DateField, DatePicker, DateRangePicker and TimeField. Registered fields and
  retained reset handlers still own their live state strongly. The regression
  covers callback and entity release after window closure, including range
  Slider and named date/time fields.
- Simultaneously rendered ColorField, Calendar, DateRangePicker, TimeField,
  TextField and ComboBox examples now keep distinct persistent entities.
  DatePicker Usage also uses the per-demo calendar helper. The two legacy
  ComboBox examples have separate open owners. Three legacy Accordion examples
  now supply explicit IDs instead of deriving the same identity from repeated
  item keys. Matching reference descriptions and the public API-pattern guide
  explain state identity.
- The served gallery now lets GPUI receive the original wheel event, matching
  the standalone WASM bootstrap. The removed JavaScript replay changed pointer
  coordinates to `(0, 0)`, discarded modifiers and split one input into many
  events. GPUI already handles wheel hit testing, units and Shift-axis mapping;
  NumberField and ColorField step by event direction, so replay also changes
  their step count. Boundary regressions cover both bootstraps and run in the
  existing website extraction gate.
- Button's reference type now includes `danger-soft`, matching the tagged
  `ButtonVariants` type and the existing Rust enum and gallery loop. `llms.txt`
  no longer incorrectly excludes `Outline` from ButtonGroup. The group inherits
  the full Button variant type in both pinned HeroUI and the Rust implementation.
- The file control protocol accepts `specimen=<stable-key>` only with
  `preview=component`. The component preview walks sections until the keyed
  example claims the request; the rendered-frame acknowledgement combines the
  section and specimen matches. The batch driver emits the field, and the
  gallery guide documents it. The WASM entry point accepts the same key through
  `?specimen=`. Unknown keys fail rather than acknowledging an empty or
  neighboring preview.
- The Button family now claims stable specimen keys for Button Group, Close
  Button and Toggle Button usage, sizes, variants, orientation, disabled and
  selection examples. The same key drives native and WASM component previews
  without adding a driver-only prop to a public component.
- Select now claims stable keys across clearable, box-customized, virtualized,
  required/disabled, grouped, surface, controlled, async, custom indicator and
  value, variant, full-width and multiple-selection examples. Its focused
  control tests cover both a finite variant and controlled selection path.
- Autocomplete now claims keys for its usage, customization, collection,
  variant, restriction, selection, controlled-open, async-filter and render
  function examples. ComboBox uses the same protocol for its controlled,
  validation, form, menu-trigger, filtering, custom-value and value-render
  cases, including the previously reproduced value-render callback path.
- NumberField and TextField now claim keys for their numeric and text field
  geometry, variants, validation, controlled values, format/customization,
  render-prop, multiline and input-type examples. The key registry now reaches
  the core collection and form states used by the next capture batch.
- Input and TextArea now claim keys for primary/secondary variants, platform
  input types, required/invalid/disabled/clearable states, controlled values,
  compact/full-width geometry, multiline rows and surface usage.
- Input Group now claims stable keys for its primary/secondary variants,
  loading, required/disabled/invalid, addon/icon compositions, password toggle,
  keyboard shortcut, multiline and trailing-action examples. Input OTP claims
  usage, variants, disabled, controlled, completion, custom-slot, form, pattern
  and validation examples.
- SearchField now claims keys for usage, compact geometry, variants, surface,
  required/disabled/full-width/invalid, controlled, render-prop, validation,
  form, custom-icon and keyboard-shortcut examples.
- Checkbox and Checkbox Group now claim stable keys for usage, sizes, primary
  and secondary variants, rounded geometry, disabled, validation, controlled,
  indeterminate, form, render-prop, group orientation, add-ons and custom
  indicators.
- Radio Group now claims keys for usage, sizes, primary and secondary variants,
  surface, validation, delivery/payment composition, custom indicators,
  orientation, controlled, uncontrolled and disabled group/option states.
- Switch now claims keys for usage, sizes, icon and no-label states, description,
  default selection, controlled state, label position, vertical/horizontal
  groups, form integration, render props and disabled on/off states. Its
  clickable/focusable row now matches HeroUI Switch.Content, so label clicks
  toggle once; the rounded track clips its fill, thumb shadow and icons, and
  thumb background transitions use the pinned 200ms ease-out endpoint. The
  rebuilt WASM artifact is
  `51b4fe11e915ebe558a58f12a7154c4b6b8e1f11a253da1ed1ece4aedbfa1575`.
- Feedback primitives now claim stable keys for Alert usage/status/closable,
  Meter usage/colors/sizes/scaled values, Progress Bar usage/colors/sizes,
  indeterminate and scaled values, Progress Circle determinate/indeterminate,
  labels, colors and sizes, Skeleton content/layout/animation/loading cases,
  and Spinner speed/color/size cases.
- The existing overlay demo helper now has acknowledgement coverage for Alert
  Dialog sizes/status/dismissal, Drawer placement/controlled state, and Modal
  size/scroll/controlled state keys; these keys are claimed by the live
  `overlay_demo` helper so closed and open transitions remain addressable.
- Navigation now claims stable keys for Accordion expansion, separators,
  multiple/disabled/controlled/custom-indicator states, Breadcrumbs levels and
  separators, Disclosure single/group/controlled/disabled/render states, Link
  icon/decoration/render states, Pagination sizing/restrictions/rendering, and
  Tabs sizing/orientation/full-width/alignment/overflow/disabled/secondary
  variants.
- Data display, layout and media now claim stable keys for Badge content,
  colors, variants, sizes and placements; Chip status/icon/variant/color/size;
  Card variants, horizontal/avatar/image/form compositions; and Avatar
  fallback, size/color/variant, group and custom-image cases.
- Collections now claim stable keys for Dropdown controlled/open/selection,
  submenu and long-press paths; ListBox disabled, sectioned, controlled,
  escape, virtualization and selection paths; and TagGroup selection,
  restriction, removal, sizing and variant paths.
- Color components now claim stable keys for ColorArea channels/disabled/render,
  ColorField variants/validation/controlled/channel/form cases, ColorPicker
  field/swatch/slider composition, ColorSlider channel/axis/render states,
  ColorSwatch transparency/size/shape/palette states, and ColorSwatchPicker
  disabled, controlled, stacked, indicator and item-render states.

## Resolved audit findings

The earlier ComboBox **Value Render Props** failure was real: the input changed
while the selected-value owner received no pick. The component fix restores the
matching callback; it does not add an unrelated callback to silence the reader.
New tests failed before the implementation correction and pass afterwards,
including the independently reproduced read-only row-selection defect.

The RangeCalendar part failure was missing ownership evidence. The tagged
stylesheet's first-row margin belongs to the parent header/body gap, separately
from each cell's vertical margin. Exact React Aria Components 1.21.0 source
confirms the `td` / inner-cell structure. The existing layout tests exercise
this separation in both render paths. Further range-cap, theme and animation
comparison remains part of the component checklist.

The current [aggregate report](audit-results.json) passes every mapped reader
with stable inputs. The inventory now contains 711 section-level seeds, all
unreviewed; those seeds establish the work queue but do not certify complete
visual or interaction parity until they are expanded and evidenced.

The control review found false matches from a speculative reference heading,
late promise completion after reset, checkout-dependent fingerprint bytes and
process leaks on publication failure. All were fixed with focused regressions.
Case-sensitive sequence matching and accurate retry errors were also corrected.
The Toast viewport omission was reproduced in the native preview and by a
failing headless regression before fixing the shared shell renderer.

## Verification and limits

Reader regressions cover positive and negative cases, stale/retired evidence,
atomic replacement failures, production state discovery, callback ownership,
empty/unknown range cardinality, disabled open overlays, effective cache changes
and misleading legacy exit codes. Two read-only review lanes examined the new
tooling; validated issues were fixed and the affected paths re-reviewed.
Two later read-only lanes examined the ComboBox slice against exact React
Stately 3.50.0, then reviewed the separate CI/part-ownership slice. Neither
later review reported an unresolved finding.

The full component suite and gallery library tests passed. The final focused
ComboBox and Calendar binaries also passed without filtering after the comment
and test-formatting changes. Native workspace build, Clippy with warnings
denied, lint inheritance, Rust formatting, stable WASM compilation and matching
wasm-bindgen generation passed. Clippy first found a missing statement
semicolon in the new test; it was corrected before the passing lint run.
PowerShell and cargo-deny are absent on this macOS host, so lint inheritance and
Clippy ran directly; cargo-deny was not run locally. The current controller
helpers were additionally exercised in the official PowerShell 7.4 Linux
container. Real failed file publication after child startup verified the actual
batch/smoke cleanup blocks. A relaunch-failure probe confirmed that cleanup keeps
the original launch error. The Windows locked-file branch remains CI-only.

Both website datasets and WASM manifests are synchronized;
`pnpm run extract:check`, inventory freshness and the fresh aggregate passed.
The preceding CI/ComboBox slice used WASM artifact hash
`4499eb5213468456ad1f732b93552492e08c95717424ea51e64f2ea00444368b`.
An independent clean Ubuntu 24.04/Python 3.12 run, without Node or Rust,
passed the reader regressions, bundle/design/accessibility self-tests and
cold-cache aggregate. Its inventory check correctly rejected the snapshot
before the final local artifact/inventory regeneration. This was a local CI
command reproduction; no hosted GitHub Actions run was triggered.
Use [the parity guide](../agents/parity.md) to reproduce the checks. Older
passing runs do not certify a later source or artifact hash.

The native Custom Value Formatting page was observed with both initial values.
A temporary macOS app bundle contained an exact copy of Cargo's native binary;
its launch environment selected that page and section. CUA keyboard/accessibility
interaction subsequently reported Opacity at 36% and Budget at $1,250.00, with
independent values. Pointer actions returned `noWindowsAvailable`, including
after repackaging. Captured pixels retained the initial values even after the
accessibility values changed, so these captures are not accepted as post-action
visual evidence. The temporary bundle is test infrastructure, not a packaging
change to the repository. Those earlier Slider captures remain historical and
have not been promoted to passing visual evidence.

The later ComboBox run used an exact copy of the current native binary
(`e259864e76f6e28624a3ec8182c9f7adb7525d40080d10936b773cc42202c391`).
LaunchServices did not apply the revised bundle environment reliably; launching
its executable directly with `HEROGPUI_PAGE='Combo Box'` and
`HEROGPUI_SECTION='Value Render Props'` reached the correct specimen. CUA then
delivered real pointer and keyboard input, and screenshots updated correctly.
Observed: the popup flipped above the field within the available window;
pointer-picking Python updated both input and controlled value text; clearing
displayed “No language selected”; typing Go, Down and Return updated both
displays to Go. This confirms the native gallery path for this slice, not a
matched upstream comparison or complete ComboBox state coverage. Screenshots
were inspected in the session; no new golden set was generated.

The control slice first ran native binary
`535b633e54ee807d2f89576bf426f87ec8b003d8a205998f4f9e0fb57c5fe0ae`.
The real file poller acknowledged Select/Modal/Slider requests. CUA verified
Python selection followed by a reset to “Choose one”; an open Modal reset to
its trigger, reopened and dismissed with Escape; and independent Slider edits
to 37% / $1,350 followed by reset to 35% / $1,200. Screenshots updated and were
inspected for those states, resolving the earlier Slider observation limit for
this run. Unknown pages preserved the current view and returned errors.
A missing preview section and the speculative full-gallery reference heading
also returned errors. Two preliminary negative requests were superseded by the
ad-hoc caller before completion; they were repeated serially and are not counted
as passing runs.

The final native binary for the controller/Toast-preview slice is
`81ea05ae5d122ce09dbae3db35ec8bdb98a204d5e8eb435d172bc6bd69624ed8`.
CUA observed the previously missing “Uploading…” notification in component
preview, then reset cleared the queue and it stayed empty beyond the pending
operation's completion window. A retained-old-root headless test separately
proves that reset cancels the future itself. These are native runtime checks;
no matched golden set or complete component state verdict was generated.

Browser navigation initially failed with “Unable to load browser request-header
policy.” It became available later in this slice without a policy bypass.
The browser then ran the rebuilt artifact
`a9eafe56917d2ebfc45d8b51ddd65870381fa94e861b077b42ff4e24b032d915`;
an HTTP byte-hash check matched the local artifact. CUA screenshots showed
Toast loading and success, Select pointer selection and keyboard selection,
Modal opening and Escape dismissal, and ComboBox pointer selection, clearing
to “No language selected”, filtering to Go and keyboard acceptance. Both
ComboBox displays followed the accepted value. Browser console checks returned
no warnings or errors for these flows. The canvas accessibility tree exposes
the input bridge rather than all GPUI component roles, so those state checks
used actual pixels. These runs are not matched upstream goldens.

A rapid WASM ComboBox `typeText("Go")`, Down, Return burst initially selected
the previous Python row once. The same reproduction isolated the loss to the
shared platform text editor: a multi-character replacement left a collapsed
anchor live between characters, so the second character replaced the first.
Clearing that anchor in the shared insertion helper fixed the plain Input and
ComboBox paths together.

A later browser repeat accepted Go correctly. A headless back-to-back input
test also passes for open/closed and plain/virtual lists. Inspection of pinned
GPUI's `Window::dispatch_key_event` shows that dirty state is redrawn before
dispatch, so consecutive calls do not bypass all rendering. The shared input
regression and the final no-wait browser burst now cover the event ordering.

The native all-route smoke subsequently aborted on Color Field with a duplicate
AccessKit node ID. An isolated Color Field launch reproduced the crash when CUA
activated accessibility, independently of the reset controller. Controlled and
Hex value reused one InputState, and the editable component delegates identity
to that Input. The correction gives the examples distinct seeded input owners
and removes the unused shared field. The new typing/ownership regression first exposed a separate retention cycle:
ColorField's form-reset callback strongly captured the same form state that
stored it, retaining InputState after removal. Slider (single/range) and
ColorSlider used the same pattern. A new window-close regression failed for all
four before correction. Reset callbacks now refer weakly to their own form
state; range callbacks retain individual weak thumb identities so registered
FormFields can still outlive the builder. The new lifecycle regression and the
existing Slider/color form-reset suites pass. The expanded regression then
failed for eleven more field types before the same correction; all 299 tests
across seven focused lifecycle/date/choice/form/picker binaries pass afterwards.
Two read-only review lanes found no remaining lifecycle or compatibility issue.
The gallery suite passed 139 tests including Color Field ownership.
Native binary `a589d0bfb56a8fa3253f4ad0a9881f3c4c5bdfabdc64f2aeb81d88f8f230e735`
rendered the complete Color Field page with native accessibility active.
The route sweep then acknowledged 28 routes before Calendar aborted with
another duplicate node ID. Several Calendar examples reuse one state; Date
Range Picker and Time Field also contain repeated state owners. Their gallery
identity corrections and their new ownership regression passed. A subsequent
native build (`68cf28cf864296161462bae74140335db72bb4cbf19d6f0aeb9b89e8b6be0919`)
rendered the full Calendar tree and acknowledged DateRangePicker and TimeField.
The next sweep reached TextField before another duplicate-ID abort; an isolated
ComboBox page also reproduced the crash. Both pages reused editable InputState
owners. Their examples are now separated, with independent ComboBox open flags.
The expanded typing regression initially caught a new key colliding with the
existing `cb-custom` / “Zig” example; the new key is `cb-custom-value`, and the
test checks all three distinct owners and preserves Zig. The focused corrected
test passes. Read-only review found the same collision and cleared the fix.

Native binary `e9c7c1f4d08b68c1bc9cef43ce8fbf5e1454ba7d3da462cef889a71f320bbfe8`
then acknowledged 58 routes with accessibility active, including TextField,
before Accordion aborted. Its Default, Surface and Hidden separator examples
omitted IDs despite reusing the same item keys. They now have explicit unique
IDs; the existing component regression already exercises identical item keys
under distinct accordion IDs. The final native binary
`ce7bd3af7f37bd3776a2961b7200c1337576ea0b285d09d4901ddcf31d5071c9`
acknowledged all 76 routes with accessibility active. The checked-in
[route run](runs/native-ce7bd3af.json) records the binary, route-list hash and
individual acknowledgements. This is default-route smoke, not variant/state
coverage. The headless platform cannot build an AccessKit tree; existing tests
document this pinned GPUI limitation.

The final lifecycle/identity slice passes 1,850 component tests across 96 test
binaries and 140 gallery tests. Workspace Clippy with warnings denied and Rust
formatting pass. Clippy caught the new lifecycle test enum's redundant `Field`
name; it is now `Control`, and that focused binary passed again after the rename.
The synchronized stable WASM artifact is
`7b8f63033501ddae5ffa60def373eada9b0a32ac52849ead21c264342f8374af`.
The subsequent HTML-only wheel fix does not change its Rust sources or binary.

A standalone browser diagnostic confirmed that CUA delivers trusted wheel
events. On the served gallery, the JavaScript replay swallowed chunky wheels
and emitted synthetic events at `(0, 0)`. Removing the replay restored real
vertical scrolling and horizontal Tabs overflow scrolling. CUA then selected
the newly revealed Section 12 and observed Content 12. A focused NumberField
changed from 1024 to 1023 after one wheel action. Start time changed from 09:30
to 10:30; after scrolling to the second TimeField, Reminder remained 09:30:00 AM.
Browser console inspection returned no warnings or errors. Screenshots were
inspected in the session; these are runtime checks, not matched goldens. The
temporary wheel diagnostic page was removed.

The wheel boundary regression rejects both the original replay and a discarded
coordinate-preserving replay: both split one action into multiple field steps.
It passes for the final direct-input bootstraps, including chunky, line-mode,
precise, diagonal and modified input. `pnpm run extract:check` passes all 16
script tests and generated-data checks; website typecheck, lint and production
build pass. Lint reports six existing warnings in unchanged files. The full
`pnpm run check` stops at pre-existing formatting in
`web/src/app/docs/components/[slug]/page.tsx` and
`web/src/app/docs/getting-started/styling/page.tsx`; both were verified unchanged
against HEAD. The changed script and package manifest pass the formatter.

The subsequent Button documentation correction passes the reference audit,
Rust formatting, all 140 gallery tests, all 16 website script tests, generated
data checks and the website production build. Native binary
`2fb0134e09d5de4bca5958b85efd1de8130d87b6cc1d38e7804c79bf732aa207`
acknowledged the Button and ButtonGroup API Reference routes. CUA observed all
seven Button variant names in the native table and the rebuilt WASM table.
The latest rebuilt WASM artifact is
`666966972c13ffb54f4344bff40d866468f0712529daf95c39a05814b2360d90`;
the locally served bytes match it and both WASM manifests are regenerated.
The earlier 76-route run remains tied to its own recorded native hash; this
metadata-only change received the focused reference checks above.

The specimen-addressing slice passes the gallery test binary's 140 tests and
the Rust formatter. Native binary
`50236339fc4157b1f68739f31c8e268fc033190051d58ab5d547a630b257ab5c`
acknowledged `btn-v-DangerSoft`, `sel-main` and `md-controlled`; CUA observed
the three previews and opened the controlled Modal. The rebuilt WASM artifact
`9bf08c7c3ddd1eb3e30d839d06258120ea7cf32d3938a25decb447c5a3f6255d`
serves the same query-addressed Button, Select and Modal previews, including the
open Modal state. Unknown keys return the same negative frame result used for
unmatched sections. The protocol remains a driver address, not a public
component prop, and coverage is limited to the explicitly claimed examples
described above.

The Button-family specimen extension passes the same 140 gallery tests,
website extraction checks and WASM manifest checks. Native binary
`c817bf18dcd054f02e3f27b2908098937f6bd12470634ecaca6bbac00e9a11f4`
acknowledged and rendered `bgroup-usage`, `close-usage` and
`toggle-controlled`; CUA observed the Button Group, Close Button and Toggle
Button previews. The current WASM artifact is
`12e274dc80690a1a5a536181125bb44c57ea9e9e7b8f7e047b90af9c20734bbf` and the
generated section/parity manifests pin that artifact and the 697 gallery
examples.

The gallery bootstrap now carries the documentation iframe's `v` artifact
query onto both the JavaScript glue and explicit WASM URL. This makes the
website's existing hash-based cache-busting contract effective for the module
and the binary together; both bootstraps share the implementation and the
website gate covers it.

The Select specimen extension passes the same 140 gallery tests, the website
extraction checks and generated-data checks. The current WASM artifact is
`c672dde0ccd15dcd000a0fa3f3f560a428839437efb02f8dafcbd88a0a8d644a`; CUA
observed the `sel-variant-Secondary` and controlled-open `sel-open` previews
from cache-busted deep links.

The Autocomplete and ComboBox extension also passes the same 140 gallery tests,
the extraction checks and generated-data checks. The current WASM artifact is
`7de55f470c9bd98d25c307ba2aa14a014837963d0a2e2cce8ef9f6b77dcb8c96`; CUA
observed the `ac-variant-Secondary` and `cb-value-render` previews from
cache-busted deep links. The inventory contains 711 unreviewed section seeds
and still reports zero verified specimens until concrete upstream/native/WASM
evidence is recorded in the specimen ledger.

The NumberField and TextField extension passes the same 140 gallery tests,
website extraction checks and generated-data checks. The current WASM artifact
is `362328ee7222bf1f372eb3b595f9b9efbce7e287b997eea27dcd97db03101618`; CUA
observed the `nf-variant-Secondary` and `tf-validation` previews from
cache-busted deep links.

The Input and TextArea extension passes the same 140 gallery tests, website
extraction checks and generated-data checks. The current WASM artifact is
`6a84c637ab4847164553796ece2ffc44a15cc0f86818c3624daa8a75542d19bc`; CUA
observed the `in-variant-Secondary` and `ta-variant-Secondary` previews after
the cache-busted module finished loading.

The Input Group and Input OTP extension passes the same 140 gallery tests,
website extraction checks and generated-data checks. The current WASM artifact
is `9cb1472c3cf7b72e06926113de7855b1baeaa5a615afbc1e67d6658f002a3b0f`; CUA
observed the `ig-password-toggle` and `otp-variant-Secondary` previews from
cache-busted deep links.

The SearchField extension passes the same 140 gallery tests, website extraction
checks and generated-data checks. The current WASM artifact is
`b1eeb3b600d39b8f82c56aa36dd206bc1180d99602904c454862c84f79bebbb9`; CUA
observed the `sf-variant-Secondary` preview from a cache-busted deep link.

The Checkbox, Checkbox Group, Radio Group and Switch extension passes the same
140 gallery tests, website extraction checks and generated-data checks. The
current WASM artifact is
`1c3c8361b3f8ed2264672c91dff142907586425cd11adeec3b7ae6c97abfacf7`; CUA
observed `cb-variant-Secondary`, `cbg-controlled`, `rg-variant-Secondary` and
`sw-form` from cache-busted component deep links.

The feedback extension passes the same 140 gallery tests, website extraction
checks and generated-data checks. The current WASM artifact is
`0fb9741922b611211b7412c984a2a8d0ac9d2e7d5cf89e5bcac485c85ee45e2a`.
CUA observed the `alert-color-Danger`, `progress-indeterminate` and
`skeleton-animation-Pulse` specimens from cache-busted component deep links.

The navigation extension passes the same 140 gallery tests, website extraction
checks and generated-data checks. The current WASM artifact is
`e49ea61ad6ac7bcab606dd2b202a1acb0c9f43b6b0d9516626b73fc62dde39ca`.
CUA observed the `tabs-secondary` and `acc-multiple` specimens from
cache-busted component deep links.

The data-display/layout/media extension passes the same 140 gallery tests,
website extraction checks and generated-data checks. The current WASM artifact
is `602e5da24842d6c5453c2ddce1b1d1502bc29fc25088dd7bd749c19a7c610940`.
CUA observed the `badge-color-Danger`, `card-form` and `avatar-fallback`
specimens from cache-busted component deep links.

The collections extension passes the same 140 gallery tests, website extraction
checks and generated-data checks. The current WASM artifact is
`2402c655ad48fd2c691a0f8a646ea08341ad53b8c9a103d95cdf1421df46f93a`.
CUA observed `dd-submenus`, `lb-virtualization` and `tg-remove-button` from
cache-busted component deep links.

The color extension passes the same 140 gallery tests, website extraction checks
and generated-data checks. The current WASM artifact is
`7e0a54d3f3f47594a5370b27fe61fc0d9a83c851da16cdc25e805e086974105b`.
CUA observed `ca-color-channels`, `cf-validation` and `csp-indicator` from
cache-busted component deep links.

The renderer-level rounded-mask patch was then rebuilt as WASM artifact
`4fbbc08c30004ab32db5fe58e6b7696740eec4a9f9a0153970f1f05bd0de543e`.
The cache-busted `ca-color-channels` capture shows rounded corners on both the
HSB background and the RGB strip stack; the focused mask regression has three
passing tests, and the full component suite passes.

RadioGroup hover now resolves the HeroUI selected and variant-specific control
fills from the live hover state while disabled rows remain inert. The existing
`rg-main` and `rg-variant-Secondary` gallery specimens were cache-busted and
visually checked through selected and unselected hover states; no browser errors
were reported. The current synchronized WASM
artifact is `44275a9443b2e68e416b82b8cfb9002be7686c945910f9a98479509cfb27affa`.
Selected controls keep the accent fill on hover, while unselected controls use
the primary/secondary hover fill; the field-border hover layer remains
documented as partial when the theme's field border width is zero.

InputOTP slots now follow the pinned slot-state precedence: field border and
hover-border tokens are present, active and filled slots use the focus
background, invalid slots retain that background with a danger outline, and
the fake caret is an absolutely positioned layer instead of a flex child. A
keyed `otp-filled-active` specimen exercises seeded characters and the active
slot; the cache-busted capture also checked the hover and validation specimens
with no browser errors. Character entrance uses the portable 250ms smooth
opacity portion of HeroUI's `slot-value-in`; scale/translate remain documented
as a GPUI renderer limitation. The synchronized WASM artifact is
`c22ae40685ab2abf4445d6e0602542246a53f5e0043517dfba60dc4c9dbf39a8`.

NumberField stepper cells now carry the shared field border width and color
tokens in addition to their 40px geometry and seam override. The default theme
keeps the width at zero, while custom themes with a visible field border no
longer lose the shell border inside the increment/decrement cells. The source
contract and NumberField behavior suites remain green; cache-busted CUA captures
of the default and vertical-stepper specimens show the rounded group without
square overflow and report no browser errors. The synchronized WASM artifact is
`eb79293840c2e79a169746f7d9150c3746534dfa86a51bd584b83bb0f1aa3979`.

Radio controls now clear the configured field border in the selected resting
state, then reapply the pinned hover or danger precedence. This matters when a
theme opts into a nonzero field border: selected controls keep a clean accent
edge instead of exposing the unselected border beneath the fill. The RadioGroup
source and gallery metadata checks are synchronized with the cache-busted
artifact `2bac2501e1069376ee454e724f5515cb68515950028f6cc9030711432d87fd00`;
the usage and secondary-variant captures show the selected control retaining
its accent fill and the rounded, unselected controls staying clean, with no
browser errors.

ProgressCircle indeterminate motion now rotates the track wrapper containing
both the full default ring and the quarter fill arc, matching HeroUI's
`progress-circle__track` animation instead of leaving a stationary ring behind
the moving arc. The reduced-motion endpoint remains static, and the source plus
feedback tests cover the shared visual wrapper. The cache-busted indeterminate
preview showed the arc advancing between captures with no browser errors; the
synchronized WASM artifact is
`34c5636c588ab432d0206db1fbd42c010919f9ec3fde0341fa3d9318bfc4beb6`.

ProgressCircle determinate value changes now use a keyed retained-fraction
tween for HeroUI's 300ms ease-out stroke transition. Reversal snapshots the
painted arc, reduced motion settles directly, and the new `Value Transition`
gallery specimen exposes the path behind a real control. The source, feedback,
gallery, extraction and parity checks pass; CUA captured immediate, mid-flight
and settled frames without browser errors. The synchronized WASM artifact is
`936d11f4ed3f353a1b12dbd5965473939a0d2ee1c77a4461dea1eb7e482a77fe`.

Skeleton's composed parent shimmer now follows HeroUI's v3.2.5 contract:
children stay visible and retain their natural layout, each child can opt out
of its own animation, and the parent owns the single clipped shimmer band.
Leaf Skeletons still use the 24px fallback, while an explicit parent height
remains authoritative. The source, feedback/gallery tests, generated website
data and parity report pass; the rebuilt `Single Shimmer` preview was checked
in CUA for three visible child blocks and shared rounded clipping without
browser errors. The synchronized WASM artifact is
`22752320602e548424f9cb4fe9d380229394ffcf9b80961b8b8831614521950d`.

Tooltip long content now honors HeroUI's `overflow-wrap: anywhere` behavior.
The 320px capped surface keeps the accessible text unchanged while the display
run receives zero-width break opportunities, so long URLs and unbroken tokens
wrap instead of escaping the panel. A dedicated `Long Content` gallery
specimen, source unit test, metadata row, generated website data and the
rebuilt browser artifact are synchronized; the pinned overlay behavior suite
passes. The synchronized WASM artifact is
`890673c6f95424e40b4e1127a6e7cba75c02aad0efcc1d50f16eee14fcd48a5f`.

Numeric output rows now request the shared GPUI `tnum` OpenType feature,
matching HeroUI's `tabular-nums` on ProgressBar/Meter, Slider and ColorSlider
outputs without applying it to their labels. The implementation is centralized
in `util::tabular_font_features`, covered by ProgressBar and Slider source
tests, and reflected in the ProgressBar, Slider and ColorSlider metadata.
Focused feedback/value/geometry tests, website extraction and parity checks
pass; CUA inspected the rebuilt ProgressBar, Slider and ColorSlider previews
with no browser diagnostics.

Alert now accepts caller-owned `indicator(content)` children while retaining
HeroUI's fixed 24px indicator box, 4px inset and status-color fallback glyph
when no child is supplied. A keyed `alert-custom-indicator` gallery specimen,
source/compose tests, reference metadata, regenerated website data and the
712-snippet WASM manifest cover the custom and fallback paths. The cache-busted
Alert preview was visually checked with no browser diagnostics; the
synchronized artifact is
`4f1eac6706dd9ffb305f30ccf2c2c115ffd7c76562beb33a45f9f1bcacba03dc`.

Toast now accepts `indicator_content(|cx| element)` render factories for
arbitrary caller-owned GPUI content. The factory is cloned with queue data and
invoked per mounted render, while the shared 24px indicator box, `indicator`
asset-path compatibility, null suppression and loading precedence remain
unchanged. A custom-content gallery action, Toast queue/source tests,
reference metadata, `llms.txt`, regenerated website data and the 712-snippet
WASM manifest cover the path. CUA triggered the cache-busted preview and
observed the custom icon inside the toast card with no browser diagnostics; the
synchronized artifact is
`caa3ee67ee3d4a52e3329a0d636c48bccb14127cc4d3e65df6f4dca29ee85f81`.

Slider default thumbs now animate the pinned drag scale through a keyed
listener-free visual child: the outer 28x20/20x28 hit target stays stable while
the inner 24x16/16x24 geometry eases to 90% over 250ms and returns on release.
Reduced motion settles directly, and custom thumb content remains caller-owned.
Focused slider and range suites pass; cache-busted CUA captures show the usage
and range specimens with stable rounded hit geometry and no browser errors. The
synchronized WASM artifact is
`da450663c318c494f705957670337729541cd4f68e4ca4b88d0c0e4e6f15a7ce`.

Drop shadows now expand their corner radii by the same spread used to expand
their bounds. This fixes the shared focus-ring path for Toast close buttons,
buttons and other rounded controls: the ring follows the control's curve
instead of becoming a large square outside it. A renderer regression asserts
the separate element and spread-shadow radii. The patch is recorded in
`docs/upstream/patches/gpui-pre-0.3.3.patch`, the focused rounded-mask suite
and full component suite pass, and cache-busted CUA captures confirm the
compact rounded Toast close focus ring, ColorSlider caps and Input field.
The synchronized WASM artifact is
`f94ec8bf35e62f7a70ace272ca931142795310d331058b0d5fe3f54593751ac3`.

TagGroup's enabled chips now use the pinned HeroUI 100ms `ease-smooth`
background transition. A stable per-tag interaction slot drives both the
plain chip and `tag_content` render-prop paths, while a rounded animated fill
keeps focus, selection, removal targets and custom children out of the keyed
animation wrapper. Disabled tags remain inert and the existing selected,
surface and custom-hover endpoints are preserved. The source contract,
collection/render-prop suites, new `tg-transitions` gallery specimen and
generated website data pass; the rebuilt browser preview shows selection
settling without layout or corner changes. The synchronized WASM artifact is
`16c764cf162d755b7d62d659ac497b381abdab393106b21354bc0d782adce18f`.

The Windows `.shots/rebuild.ps1` driver uses a Windows launcher path; on this
host builds used Cargo's resolved shared target directory instead. A native
file-protocol route sweep uses the current smoke script's route list, validates
real acknowledgements and retains any failed route for diagnosis.

## Next complete slice

1. Execute the remaining batches in the plan, carrying tests, documentation,
   gallery metadata/examples and current website/WASM outputs in each slice.

## Review disposition for the completed slices

- **Scope reviewed:** inventory/report/inert readers and Slider examples;
  ComboBox callbacks, restrictions, tests and documentation; CI integration and
  RangeCalendar ownership comments; gallery controller/reset and driver
  lifecycle changes, with targeted re-review of their fixes; form-state lifetime
  and gallery identity corrections; direct browser wheel delivery and its
  regression gate. Two read-only wheel review lanes returned no findings.
  The Button/ButtonGroup wording correction used the documentation fast path,
  checked directly against pinned styles/types and Rust owners. Binary and
  generated outputs were checked
  through their source provenance and synchronization tools.
- **Fixed:** the confirmed tooling failures recorded above; inert Slider
  examples; missing ComboBox value callbacks and read-only row gating; controller
  acknowledgement/reset lifecycle, driver cleanup and fingerprint portability;
  missing ToastViewport in previews; self-retaining form reset callbacks and
  duplicate native accessibility identities; wheel interception; incomplete
  Button/ButtonGroup variant documentation. All review findings within these slices
  were resolved.
- **Rejected after validation:** an additional RangeCalendar spacing change;
  the existing parent gap already supplies the first-row margin, and both
  render paths have geometry coverage.
- **Deferred major or dangerous refactors:** none.
- **Verification:** the commands and runtime observations above apply to the
  completed slices only.
- **Blocked or unverified:** matched comparisons, per-instance fixture
  addressing and remaining full-plan work are outstanding. Existing website
  formatting issues are recorded above. The implementation goal remains active.
- **Suggested guide rule:** stored form reset callbacks must hold only a weak
  reference to their owning form state; registered fields retain the state
  strongly. This repeated failure pattern was corrected in fourteen component
  types. No instruction file was changed.
