# HeroGPUI UI and design parity implementation plan

Prepared 2026-09-12 against HeroGPUI commit `cdc93ba4d4686c68cc6e4a2e1923b6c36ddb2127`.
Status: implementation handoff proposal. This document does not claim the work below has been implemented or visually verified.

Execution has started. Read [execution progress](ui-design-progress.md) before continuing; the baseline findings below are historical and must not replace the refreshed work queue.

Read this together with the [component checklist](ui-design-component-checklist.md). The checklist is part of the plan, not an optional appendix: it specifies the component-by-component work, variants, states, compositions, source locations, gallery examples, and focused tests.

## 1. Outcome and target

Bring the Rust components, native gallery, live WASM previews, reference documentation, and website to a consistent, measured HeroUI v3 visual and interaction standard. Preserve HeroGPUI's native implementation and its explicitly documented product decisions.

The [official release index](https://heroui.com/en/docs/react/releases) lists **HeroUI React v3.2.5**, released September 8, 2026, as the latest stable release when this plan was prepared. Its [release notes](https://heroui.com/en/docs/react/releases/v3-2-5) identify the recent Select, Tabs, Toast, overlay, and collection changes. The repository already targets that version. This is primarily a completion and correction project, not a dependency upgrade.

Freeze the execution baseline to:

- HeroUI tag `v3.2.5`, commit `5f13f6ed355bdbd5d5f69e5944685438a3591793`.
- React Aria `3.52.0`, React Stately `3.50.0`, React Aria Components `1.21.0`, with additional inherited packages resolved from that tag's lockfile.
- Rust and the entire `gpui-pre` family exactly as pinned by this workspace; currently Rust 1.98 and `gpui-pre =0.3.3`.
- HeroUI **React** as the design reference. HeroUI Native, HeroUI Pro, v2 examples, and unreleased main-branch changes do not define this contract.

At execution start, check the release index again. If a newer stable release exists, first make a release-delta worksheet and deliberately refresh all affected pins, source archives, metadata, and inherited behavior references as one baseline change. Do not silently mix releases or chase new releases halfway through a component batch. The user's request for the latest HeroUI permits planning that migration; it does not make an unreviewed moving target useful evidence.

The website should retain HeroGPUI/Porabuild branding while adopting the clarity and consistency of HeroUI's component documentation. Component parity is exact to the chosen contract; marketing layout is a product design task with its own acceptance checks.

## 2. What the current checkout already establishes

These are source and recorded-evidence findings, not a new full runtime audit.

| Finding | Consequence for execution |
|---|---|
| Components and the website already pin v3.2.5. | Do not spend the first batch repeating the version migration. |
| [interaction-sweep.md](interaction-sweep.md) records work on Tag removal, Tabs alignment, Select clearing, Toast APIs, and focus modality. | Read the follow-ups before assigning a feature as missing. Preserve their regression cases. |
| [interaction-inventory.json](interaction-inventory.json) stores gallery and upstream demo inventories, but its recorded gallery dataset hash differs from the current `rust-examples.json`. | Refresh the inventory and expand sections into specimens before using it as a work queue. |
| [audit-results.json](audit-results.json) still lists Toast APIs and the Expanded Stack example as missing; the current implementation and gallery contain them. | Re-run the owning audits and repair evidence freshness. Do not implement duplicate Toast APIs based on this snapshot. |
| [the earlier visual sweep](../parity-sweep.md) mostly compares native captures with native goldens. | Retain it for regression history; add direct pinned HeroUI versus HeroGPUI evidence. |
| Reference metadata marks substantial visual contracts partial. | Seed the first review queue from those rows, but verify the implementation before claiming a current defect. |
| Some metadata says GPUI has no accessibility tree, while `src/a11y.rs` implements an AccessKit-backed contract. | Revalidate individual limitations against pinned GPUI; replace obsolete blanket explanations. |
| Native gallery fonts vary by OS; WASM uses Inter Variable/JetBrains Mono; the website shell uses Geist. | Normalize fonts in comparison fixtures before judging text width or baseline differences. Keep platform-font behavior as a separate test. |
| Component pages already use a single lazy WASM instance with an example switcher. | Improve this flow in place; keep its one-instance, one-requested-example architecture. |

Source locations for these findings: [Toast](../../crates/herogpui-components/src/toast.rs), [overlay examples](../../gallery/src/pages/components/overlays.rs), [reference metadata](../../gallery/src/pages/reference_metadata/), [accessibility](../../crates/herogpui-components/src/a11y.rs), [gallery fonts](../../gallery/src/app.rs), and [preview host](../../web/src/components/preview/gallery-frame.tsx).

Known priorities recorded by the current metadata include Calendar/RangeCalendar today and range styling, Table cell styling and shared column tracks, Select option styling and long values, vertical Tabs label wrapping, Toast absolute stack geometry, field transitions, and Accordion/Disclosure collapse motion. The component checklist makes these concrete. Treat each as a review-and-fix task, since metadata can lag code in either direction. Collection rows now include the pinned 98% ease-out-quart press transition in the shared stable-slot animation path.

## 3. Repository rules and ownership

Read [AGENTS.md](../../AGENTS.md), [workflow](../agents/workflow.md), [component implementation](../agents/components.md), [parity](../agents/parity.md), [gallery verification](../agents/gallery.md), and scoped instructions before changing the corresponding files. Use the current instructions where historical roadmaps conflict.

| Surface | Owning paths |
|---|---|
| Shared vocabulary and color math | `crates/herogpui-core/src/` |
| Theme, semantic colors, layout, component defaults | `crates/herogpui-theme/src/` |
| Rendering, interaction, focus, motion, accessibility | `crates/herogpui-components/src/` and `tests/` |
| Public consumer facade and examples | `crates/herogpui/`, root `README.md`, `llms.txt` |
| Native gallery shell, routing, specimen composition | `gallery/src/app.rs`, `gallery/src/control.rs`, `gallery/src/pages/` |
| Checked-in component contract | `gallery/src/pages/reference_metadata/` |
| Audit readers, pinned inputs, captures | `.shots/` |
| Progress and visual evidence | `docs/parity/`, `docs/parity-sweep.md` |
| Web shell and documentation | `web/src/app/`, `web/src/components/`, `web/src/lib/` |
| Extractors and generated data | `web/scripts/`, `web/src/data/` |
| Shared Rust browser runtime | `crates/herogpui-web/`, `web/public/gallery/` |

Every implementation slice must carry its affected tests, example, metadata, public docs, generated datasets, and current WASM artifact/manifests into the same mergeable change. Do not merge an example/API change while promising to synchronize its live preview later. Internal work can be staged in small commits within that complete slice.

Maintain these constraints:

- Preserve unrelated changes; start and finish each slice with `git status --short` and the relevant diff.
- Read implementation, callers, tests, tagged HeroUI source, and exact dependency behavior before editing.
- Use unpacked `~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.3/` as GPUI API evidence. Historical references to a Zed checkout or `gpui` 0.2.2 are not current evidence.
- Keep controlled values caller-owned, uncontrolled seeds stable, per-instance state keyed, ids unique, and callbacks exact-once.
- Preserve the recorded focus decision: Escape does not itself enable focus rings; keyboard-control modality enables them, pointer input clears them. Document this intentional deviation when comparing React Aria.
- Preserve supported native customization. Do not remove `sx`, radius, or size extensions merely because they are not upstream props. Keep them separately documented and owner-scoped in `extra_audit.py`; the default specimen must still match upstream.
- Do not invent v2 props, no-op builders, undocumented common variant enums, or new public APIs solely to satisfy a parser. Internal composition helpers are preferable where the parent already owns the contract.
- Reuse existing helpers, but do not homogenize component-specific radii, fills, focus treatment, or motion.

## 4. Define complete coverage before changing pixels

### 4.1 Extend the existing inventory

Extend `docs/parity/interaction-inventory.json` with a versioned specimen and transition schema. Keep the existing upstream-demo and gallery-section records and their hashes; map them rather than replacing them with a second unrelated inventory.

A specimen needs:

```json
{
  "id": "select/primary/single/clear/light/keyboard-focus",
  "component": "Select",
  "part": "Select.Trigger",
  "upstream": {
    "tag": "v3.2.5",
    "source": "packages/react/src/components/select/select.tsx",
    "selector": "the exact owning selector",
    "sourceSha256": "computed during inventory refresh"
  },
  "port": {
    "module": "crates/herogpui-components/src/select.rs",
    "owner": "Select",
    "galleryPage": "Select",
    "gallerySection": "With Clear Button",
    "specimenKey": "primary-single"
  },
  "configuration": {
    "variant": "primary",
    "selectionMode": "single",
    "theme": "light",
    "motion": "normal",
    "direction": "ltr"
  },
  "state": "keyboard-focus",
  "setup": ["reset", "select a value", "Tab to trigger"],
  "assertions": ["selection retained", "trigger focused", "keyboard ring visible"],
  "evidence": {"upstream": [], "native": [], "wasm": [], "tests": []},
  "status": "unreviewed"
}
```

This schema is proposed, not an existing driver API. Add layout dimensions, scale factor, font identity, browser/OS, frozen date/locale, expected callback trace, animation timestamps, artifact hash, and implementation commit to actual capture records. Keep assertions structured enough to validate, not just free-form claims.

Use statuses with separate meanings: `unreviewed`, `specified`, `measured-gap`, `implemented-unverified`, `verified`, `intentional-deviation`, `platform-limited`, and `not-applicable`. A driver failure is `unobserved` evidence attached to a still-incomplete case, never `verified` or a component failure by assumption. Preserve the existing public metadata status vocabulary unless deliberately migrating its consumers; verification status belongs in the evidence ledger.

### 4.2 Enumerate the contract from its full source union

For every component and companion:

1. Read tagged TSX/types, styles and composed parts, docs examples, stories, upstream tests, and the inherited React Aria/Stately behavior actually used.
2. Enumerate all finite props that change pixels or interaction: variants, sizes, colors/statuses, orientation, placement, selection mode/behavior, backdrop, scroll mode, alignment, animation type, composition flags and visual boolean states.
3. Expand loops such as `Variant::ALL`, `Size::ALL`, and palette lists into named specimens. A heading called “Variants” is not an individual passing case.
4. Include every named part, including parts with only children or styling props. Track a part folded into a Rust owner explicitly.
5. Compare upstream-only and port-only entries. Resolve missing aliases and grouped-page mappings before interpreting them as missing components.
6. Attach all existing Rust gallery examples to the queue. Extra HeroGPUI customization examples are a compatibility suite with an unchanged-default check.
7. Check every `partial` and `unavailable` row, not just a missing-prop report. Separate browser-only mechanics from portable visible or behavioral outcomes.

The checklist is the starting enumeration, not permission to ignore a state found in source. The baseline metadata omitted `danger-soft` from one Button API union even though the pinned stylesheet and Rust enum supported it; that documentation gap is now corrected, as recorded in the execution progress. Include it in the full state matrix. Resolve other omissions or contradictions the same way.

### 4.3 State matrix applied to every applicable component

| State family | Required observable cases |
|---|---|
| Rest | Default value; empty/placeholder; populated; short and wrapping text; content/icon-only composition where supported. |
| Pointer | Enter, hover, press held, release inside, release outside, leave, cancellation, re-entry, secondary/middle button isolation. |
| Focus | Tab, Shift+Tab, pointer focus, focus-visible, focus-within, blur, re-entry, focus repair after item removal, focus restoration after overlays. |
| Value | Unselected/selected, checked/unchecked/mixed, empty/non-empty, current/active, min/mid/max, default seed, caller-driven change and rejected controlled change. |
| Restrictions | Disabled root, disabled child/key, read-only, required, invalid; disabled controls reject activation and leave tab order; read-only keeps applicable navigation. |
| Async | Idle, loading/pending, success, error, empty results, retry, stale response arriving after newer input, canceled/unmounted request. Use only cases supported by that component or its documented composition. |
| Disclosure/overlay | Closed, opening, open, closing, interrupted/reversed, external controlled open/close, pointer and keyboard dismissal, nested overlay, viewport edge and resize. |
| Collection | No items, one item, many items, disabled first/middle/last/all, single/multiple selection, typeahead, paging, scroll-to-focused item, virtualization and dynamic removal. |
| Drag | Start, move, outside bounds, endpoint clamp, reversal, release, cancellation, disabled transition, nested ownership and post-drag click behavior. |
| Feedback/motion | Start, progress, finish, interruption, repeated activation, rapid reversal, reduced motion, theme change mid-state. |

Static presentational parts should not acquire fake hover or disabled props. Mark genuinely inapplicable state families with a source-backed reason. Pending/loading is not a universal API: Button uses `is_pending`, Toast has its own loading contract, and CloseButton's CSS pending state does not establish a public pending builder.

For each finite visual variant × size × semantic role, cover light and dark resting states and every applicable interactive endpoint. Cross orientation/placement with every state that changes geometry. Explicitly enumerate combined selectors and precedence: selected+hovered, selected+pressed, mixed+disabled, invalid+focused, disabled+selected, required+empty, open+focused, and customized+hover/press. Do not substitute a pairwise sample for a promised finite variant/state combination.

Continuous values, unbounded content, and arbitrary child trees need named equivalence classes: minimum, maximum, interior, out-of-range, empty, long/wrapping, RTL text, multiline, and custom fixed-size child. Record why these partitions exercise the relevant branches. Impossible combinations may be omitted only with a contract reason. Coverage reports must show remaining cases and exclusions separately; do not inflate completion with automatic skips.

## 5. Build a trustworthy comparison and interaction harness

### 5.1 Matched upstream fixtures

Create a private, development-only fixture runner using the exact pinned HeroUI packages and tagged example compositions. Keep it outside public component previews. Prefer the existing dependency installation where it meets the pins; otherwise isolate the fixture lockfile so reference work cannot upgrade the production site accidentally.

Render the same content, dimensions, chosen font, theme, viewport, scale factor, and component configuration on both sides. Use local deterministic images, frozen dates/timezones, deterministic async responses, fixed collection data and explicit initial state. Avoid live-network content and a moving “today” in goldens. Preserve upstream layout constraints; a wider fixture can conceal the vertical Tabs and narrow Table defects.

Capture actual interactions. Data attributes forced onto a DOM node may help inspect a CSS endpoint, but they do not prove the component reached that state. Store style-only probes separately from interaction evidence. Source fixtures and package/tag hashes must accompany live-site captures because documentation websites may advance beyond the chosen release.

### 5.2 Native and WASM drivers

Reuse the gallery's page/section controls and frame acknowledgement. The
control protocol now also accepts a stable `specimen` key in component-preview
mode, and the Button/Select/Modal pages claim representative keys. Extend that
registry as each component batch gains concrete cases; keep test control in the
gallery/test harness rather than the public component API.

- On Windows, follow `.shots/rebuild.ps1`, `.shots/drive.ps1`, `.shots/batch.ps1`, and the scoped driver guide. Focus-sensitive checks need the real-input path.
- On macOS/Linux, the Windows scripts are not portable. Build the workspace, resolve Cargo's actual target directory, launch the freshly built binary, and record its identity. Implement or use a host-specific input/capture driver with the same reset/state/ack semantics. Do not claim native input from a WASM-only drive.
- Capture the actual application window. Validate route, section, specimen, theme, visible state, and nonblank content before saving a verdict.
- Posted-input limitations must be explicit. For a callback test, do not hide a missing first activation with an unconditional double click; assert what each delivered action did.
- Make every overlay and Toast state addressable. `HEROGPUI_OPEN_OVERLAYS` currently opens only selected examples; it does not establish complete open-state coverage.
- After navigating or switching specimens, verify that stale timers, overlay registrations, callbacks, focus handles and keyed state do not affect the new example.
- Add deterministic time control for motion/timer tests if existing test facilities support it. Keep a separate real-time smoke so simulated clocks cannot conceal runtime scheduling failures.

### 5.3 Pixel and motion acceptance

Store upstream, native, and WASM images with a shared specimen id. Generate a crop-aligned comparison and difference image; retain uncropped context so clipping and anchoring remain visible. Never change the reference crop to hide overflow.

Proposed acceptance policy:

- Source tokens, resolved metric formulas, endpoint values, duration and cubic-bezier parameters must match the pinned contract exactly unless a named deviation applies.
- Compare bounds in logical pixels; allow only documented device-pixel rounding. A difference above one logical pixel in a critical edge, baseline, track, target, or anchor requires review, not an automatic pass.
- Compare solid interior colors with the same color pipeline; investigate meaningful differences independently of text antialiasing. Do not globally blur images or loosen thresholds to make failures disappear.
- No clipped rings/text, overlapping sibling targets, unintended layout shift, escaping overlays, disappearing controls, stale surfaces, or blank captures.
- Motion must have start/end plus intermediate samples (for example 25%, 50%, 75%), a measured duration, and interruption/reversal/reduced-motion evidence. A matching end screenshot cannot prove animation parity.
- Characterize font rasterization and native shadow limitations before setting an image-diff threshold. Separate those differences from geometry; record platform-specific baselines where needed.

Do not require a full Cartesian visual sweep for every tiny intermediate edit. Run focused cases while iterating, then complete the family matrix before accepting that family, and the affected shared-helper regression set before accepting a foundation change.

## 6. Shared design and behavior work

Complete these in small, evidence-led slices before broad component polish.

### 6.1 Theme and typography

Audit the full token dependency chain in `semantic.rs`, `layout.rs`, `theme.rs`, `components.rs`, and `provider.rs` against the checked-in CSS theme inputs. Include OKLCH/OKLab mixing, alpha compositing over each surface, field backgrounds/placeholders, semantic foregrounds, borders, separators, shadows, focus offset, disabled opacity, spacing, radii, delays and easing.

Build token specimens in both themes and a custom accent theme. Exercise theme changes while a control is focused, invalid, selected, pressed, and open. Verify overlay shadows and foregrounds follow the active theme. Do not simulate per-subtree theme propagation with global mutation; determine whether a scoped requirement belongs to the web shell or needs a deliberate GPUI architecture change.

Measure font family, weight, size, line height, baseline, letter spacing, truncation, wrapping, tabular numerals and icon alignment. Normalize comparison fonts without rewriting platform typography by accident. Use existing font seams and font subsetting checks. Preserve fallback glyphs, non-Latin text and code-font legibility.

### 6.2 State precedence and customization

Write a part-specific precedence table before modifying shared field/button helpers. Resolve theme/default → instance configuration → matching `sx` for a resting property; derive state endpoints from that resolved value according to the current owner contract. Preserve Button's existing special behavior for a solid `sx` background and explicit `hover_bg`.

Use `tests/sx_ownership.rs` and [customisation-roadmap.md](../customisation-roadmap.md) as starting inventories, with current instructions taking precedence over historical proposals. Convert pending owners only after checking each painted part. A root override must not silently recolor every descendant, slider fill or selected tab. Keep per-edge/per-corner handling and document unresolved rem/fraction/gradient extraction rather than misrepresenting it as fully supported.

### 6.3 Motion and transforms

Review `anim.rs`, `util.rs`, `Motion`, `Curve`, keyed tweens, `overlay_phase`, `entering_zoom`, and pressed geometry. Spike the pinned GPUI rendering APIs before declaring full subtree scale, translation, rotation, inset shadow, or backdrop blur impossible.

The priority is to close visible approximations: unscaled fixed-size children during presses, swapped chevrons instead of rotation, immediate field/focus changes, placement-specific overlay translation, disappearing collapse content and inaccurate stack overlap. Keep the event-owning element's identity stable. Combine fade/translation/scale in one visual transition so opacity is not multiplied by nested helpers.

For each feasible primitive, prove two actual consumers before generalizing it. If the pinned framework prevents an exact result, document the API evidence and smallest possible framework/fork proposal separately. Do not quietly change GPUI pins, add broad unsafe code, or call an approximation exact. A platform-limited contract remains visibly incomplete against full parity.

### 6.4 Focus, overlays and accessibility

Verify `tab_stop_handle`, `shows_focus_ring`, `app_focus_root`, `trap_tab`, `overlay_scope`, `panel_focus`, `floating`, and exit lifetime against real dispatch tests. Check topmost dismissal, occlusion, owned outside bounds, trigger restoration, Enter-release reopening, nested menus, and an exiting popup beneath a newly opened dialog.

Use the existing `a11y::A11y` extension. Confirm roles, names, values, selected/checked/expanded state, focus ownership and hidden/exiting behavior in the available native accessibility tree and browser surface separately. Revalidate against the newer inherited Aria packages before updating old source citations. Put current limitations in the reference, with an exact reason; do not copy “no accessibility tree” from obsolete rows.

Treat high contrast, zoom, RTL and localization as explicit verification axes. Where native directional layout or platform accessibility is unsupported, identify affected parts and preserve legibility and navigation; do not advertise support from a web-shell-only pass.

## 7. Component execution order

Every item in the [component checklist](ui-design-component-checklist.md) is in scope. The following order reduces repeated fixes without turning the work into a framework rewrite.

| Batch | Work | Exit evidence |
|---|---|---|
| 0 | Refresh source pins/inventory and audit snapshot; classify stale metadata; implement specimen addressing and matched fixtures. | Known-positive and known-negative harness cases, reproducible Button/Select/Modal specimens on upstream/native/WASM, truthful initial gap report. |
| 1 | Tokens, fonts, field chrome, focus, state precedence, animation primitives and `sx` ownership. | Token/metric tests, affected helper tests, both-theme representative composites, customized states and motion samples. |
| 2 | Button, ButtonGroup, CloseButton, ToggleButton/Group, Checkbox/Group, RadioGroup, Switch, Slider. | All finite variants, sizes and applicable state combinations; callback, focus, form and drag tests. |
| 3 | Input, InputGroup, TextField, TextArea, SearchField, NumberField, InputOTP, labels/messages, Fieldset and Form. | Real edit/validation/reset flows, correct part chrome, configurable focus-ring visibility without losing focus or accessibility, no unexpected geometry changes, native/WASM text-input evidence. |
| 4 | ListBox, TagGroup, Dropdown, Select, Autocomplete and ComboBox. | Selection/typeahead/async/virtualization proofs; open panels, clear/removal targets and viewport-edge coverage. |
| 5 | Modal, AlertDialog, Drawer, Popover, Tooltip and Toast; Accordion/Disclosure motion using the same proven primitives. | Enter/open/exit/interruption captures, nested layers and drag ownership, focus return, timers and hotkeys. |
| 6 | Calendar/RangeCalendar, DateField/TimeField/DatePicker/DateRangePicker, all color controls. | Calendar and color geometry, segments, constraints, range caps, year-picker/open states and mixed-input flows. |
| 7 | Table, Tabs, Pagination, Breadcrumbs, Link, Toolbar and ScrollShadow. | Shared tracks, constrained labels, scroll/resize/focus transitions, nested editable content and overflow. |
| 8 | Alert, Meter, ProgressBar/Circle, Spinner, Skeleton, Avatar, Badge, Chip, Card, Surface, Separator, Kbd and Typography. | Remaining visual variants, async presentation, compositions and motion with complete part coverage. |
| 9 | Gallery, documentation and website finish; full integrated verification. | Complete route/reference/example coverage, synchronized live artifact, accessible responsive site, final evidence report. |

Gallery, reference and website data updates accompany **each** batch; Batch 9 is the final integration and shell-design pass. Pull high-impact confirmed defects such as unusable narrow Tables forward once their dependencies are ready.

Do not give another agent an unbounded “make everything match” task. Give it one family or a named shared primitive with its specimen ids, current evidence, paths, expected outcomes, dependencies, and gates. Default to serial ownership for `util.rs`, `anim.rs`, theme files, route registries, generated data and the WASM artifact. If parallel implementation is later explicitly requested, designate one integration owner for those paths.

## 8. Native gallery improvements

Keep one page per existing component route, with grouped companion components clearly discoverable. Keep all examples before reference panels; each example has a title, optional description, then a live bordered specimen. Explanatory prose stays outside the specimen. Preserve source extraction rather than maintaining separate website examples.

Implement the following improvements:

1. **Consistent shell:** verify navigation hierarchy, active category/item, title/import line, content width, vertical rhythm, code surface, search/navigation affordances and both themes. Distinguish component-owned geometry from example framing.
2. **Default example first:** show the uncustomized upstream default in Usage. Move extension demonstrations into explicitly named customization sections. The current Button Usage demonstrates radius/`sx`; keep that example, but make its role clear.
3. **Complete example menus:** make every documented variant, composition and important state reachable. Label size/color/variant specimens individually; avoid duplicate ambiguous headings. Do not leave dynamic examples controlled without feedback state.
4. **State demonstrations:** add real actions for opening, closing, disabling, validation, pending completion, empty results, errors and controlled resets. Keep forced-state inspection controls in a developer comparison mode, outside public component UI.
5. **Stable reset/deep links:** address page, section and specimen; reset values, timers, focus, scroll and overlays predictably. Existing environment variables remain supported. Add deterministic data/time at the harness boundary.
6. **Layout stress:** provide narrow/wide, multiline, long-option, overflow and nested examples. Use `stretch_col()` where a full-width demo needs a real available width; do not widen a fixture to disguise a component problem.
7. **Open-state framing:** show complete popup/dialog/toast bounds without embedding them in an artificially oversized frame. Verify overlay anchoring after window resize and scroll.
8. **Reference consistency:** show Props, Anatomy/Parts, States and Styling with accurate ownership, supported Rust names and useful limitations. Companion names must be searchable and linkable within their owning page.
9. **Capture maintenance:** refresh only goldens affected by verified corrections; record why. Keep old evidence as history and new upstream comparison evidence under versioned specimen paths. Copy public catalog images only when their source goldens intentionally change.

Acceptance: every route draws without panic; every example is reachable and shows its stated behavior; independent instances remain independent; both themes and reduced motion work; closing or leaving a page leaves no active overlay or timer affecting another page.

## 9. Documentation improvements

### 9.1 Per-component documentation contract

For every component/companion, update the Rust docs, `llms.txt`, gallery metadata/example and generated site together. Explain:

- Purpose and when to use it; related component distinctions, especially Select versus Autocomplete versus ComboBox.
- Exact Rust import/constructor and minimal working composition.
- Every public variant/size/role and its default; distinguish HeroGPUI extensions.
- Required/optional parts, ownership, nesting and part customization.
- Pointer, keyboard and focus behavior; values, callbacks and controlled/uncontrolled examples.
- Empty, disabled, read-only, invalid, required, selected, pending, open and exit states as applicable.
- Sizing, wrapping, alignment, overflow, orientation, placement, motion and reduced-motion behavior.
- Styling precedence, supported theme tokens, supported `sx` seams and their limitations.
- Accessibility and platform differences described in user terms, with evidence links in maintainer docs.

Separate three claims: an API exists, an event path is tested, and a visual state matches upstream. An `implemented` API row must not imply all visual states are verified. Keep partial behavior visible where it affects users; rewriting React terminology must not erase a limitation.

### 9.2 Guides and maintainer evidence

Review root `README.md`, `llms.txt`, public crate docs, gallery getting-started pages, web getting-started/AI pages, `web/README.md`, `web/ARCHITECTURE.md`, `web/DEPLOYMENT.md`, agent guides and relevant upstream notes. Reconcile historical references to old GPUI sources, old Aria versions, Windows-only commands, historical hosting assumptions, outdated artifact sizes and package publication status using current evidence.

Use the facade's actual supported installation contract; compile changed public examples and doctests. Do not promise published crates, platform support, a benchmark or complete parity without verification. Preserve attribution and the existing brand source-of-truth workflow.

Add a maintainer coverage report generated from the inventory: outstanding measured gaps, unobserved cases, intentional deviations, platform limits and the last evidence commit. Preserve history in `interaction-sweep.md` without presenting old audit output as current. Do not hand-copy changing audit totals into guides.

## 10. Website UI, content and live preview work

Read [web/AGENTS.md](../../web/AGENTS.md), [architecture](../../web/ARCHITECTURE.md) and [deployment](../../web/DEPLOYMENT.md). Before Next.js implementation, read the installed `web/node_modules/next/dist/docs/` and real pinned HeroUI typings. Retain pnpm, existing routing and the Porabuild brand synchronization workflow.

### 10.1 Shell and navigation

Audit navbar, sidebar, breadcrumbs, table of contents, footer, command palette, theme toggle and previous/next navigation. Standardize spacing, typography hierarchy, hover/focus states, active-route contrast, scroll behavior and touch target sizes. Search should find both component and companion names, Rust names and useful aliases without presenting removed v2 APIs as supported.

Verify the desktop and collapsed mobile navigation, long titles, keyboard-only use, Escape dismissal of site-owned menus, persistent theme and first-paint theme consistency. Honor reduced motion. Keep code/table overflow local rather than creating horizontal document overflow.

### 10.2 Component detail pages

Keep the established order: header/import → Usage → compact Anatomy → Customization → API reference (Props, Parts and slots, States) → Related components.

Improve the existing example browser with clear selection, a matching description/code block, copy feedback and an addressable example anchor. If reset or expand-preview controls are added, they must operate on the same live instance and accurately describe what they do. Expose important state examples through the selector rather than mounting dozens of canvases.

Preserve exactly one lazy GPUI/WASM instance per component page and construct only the requested example. Keep the full gallery shell out of this embed. Verify rapid switches before and after boot, theme changes, back/forward navigation, resize, focus entry/exit, unmount and loading/error/retry presentation. Text/code must remain useful if the preview cannot initialize; do not disguise that failure with a screenshot presented as live.

Review `gpui-docs.ts` transformations so Rust terminology is precise and partial-support descriptions survive cleanup. Keep readable dense tables with wrapping, copyable Rust methods, clear defaults, and accessible status labels. Provide companion anchors for ToggleButtonGroup, DisclosureGroup, Label, Description, ErrorMessage and FieldError on their owning pages.

### 10.3 Landing page and catalog

Use authentic components and measured product facts in the hero, component atlas, feature descriptions, installation code, agent-doc links and CTA flow. Make examples legible in both themes and at smaller widths. Keep Porabuild typography and brand assets intact unless the project explicitly changes that brand contract.

Refresh catalog tiles after the underlying component correction. Link directly into relevant live examples and make categories/search useful. Do not present screenshots as interactive previews or use a HeroUI React recreation as proof of HeroGPUI rendering. Keep any developer upstream comparison runner out of public product flows.

### 10.4 Responsive and platform coverage

Verify the website at representative 360, 390, 768, 1024 and 1440 CSS-pixel widths, plus 200% zoom, long content and browser text-size changes. These are proposed test sizes, not hard-coded design breakpoints. Test Chromium, WebKit and Firefox where the gallery platform supports them; keep browser-specific limitations visible.

For native component metrics, the existing parity contract uses the largest applicable upstream breakpoint. Keep that as the baseline. On narrow website frames, verify usable containment and overflow; do not silently shrink native controls to HeroUI's mobile rules. If responsive component metrics are introduced, design an explicit shared viewport/density contract, apply it to native and WASM from the same source, and verify both desktop and narrow states.

Test IME, composed characters, clipboard, selection, pointer/wheel and focus transfer in the WASM input path; the vendored web-platform fork and the native backend have different dispatch risks despite sharing component code.

### 10.5 Data, artifacts, performance and delivery

Regenerate `reference.json` and `rust-examples.json` with `pnpm run extract`; never hand-edit generated JSON. Regenerate catalog data when routes/categories/imports change and public shots when their source captures change, using the existing extractors. Run `pnpm run extract:check`.

After any Rust component or gallery change that affects the shipped runtime, rebuild `herogpui-web` on pinned stable, run the matching `wasm-bindgen`, regenerate `wasm-sections.json` and `wasm-parity.json`, and verify the live preview against its displayed code. Do not regenerate hashes around an old binary to make checks pass.

Measure preview cold start, warm navigation, WASM transfer/parse, input responsiveness, long-list scrolling and memory before and after work. Set budgets from that baseline and investigate regressions. Preserve lazy boot and browser caching; repeated example switching should not leak canvases, applications, handlers or timers. Do not claim performance numbers without the device/browser and procedure.

Verify both local root paths and the documented `/herogpui` mount: pages, fonts, WASM/glue, images, anchors, metadata URLs and `llms.txt`. Follow `publicUrl()` and existing base-path handling. If a deployment is subsequently requested, use the current Vercel runbook and verify the actual resulting build and public routes; this planning task does not deploy or modify the parent Porabuild website.

## 11. Verification commands and scope

The current [CI workflow](../../.github/workflows/ci.yml) is authoritative. Re-read it at execution time instead of relying on an older copied command list.

While fixing one component, use its named focused binary from the checklist. This repository provides `.shots/run-tests.sh` to validate test-harness summaries as well as process results:

```sh
bash .shots/run-tests.sh -p herogpui-components --test <focused-binary> --locked
cargo fmt --all -- --check
python3 .shots/design_audit.py --coverage
```

Select relevant audits by claim: tokens/design, API/extra/write-only, state/behavior/motion, anatomy/parts, demo/example/inert/reference, or accessibility/id. An exit code alone is insufficient while scripts can report gaps and still return zero. Refresh the structured audit report from real output, and implement a fail-closed aggregate gate for unresolved/unmapped claims with explicit baseline handling during migration. Prove changed readers with known-negative fixtures.

For a complete broad parity handoff, run the current CI-equivalent set, including the workspace test wrapper, feature-specific theme tests, formatting, lint, dependency/feature checks, rustdoc, packaging/install validation when affected, audit self-tests and all audits. Gallery library tests, native route smoke, actual interaction/capture and WASM/browser verification remain separate gates.

Typical broad commands, subject to the current workflow:

```sh
bash .shots/run-tests.sh --workspace --locked
bash .shots/run-tests.sh -p herogpui-theme --features serde --locked
cargo check --workspace --locked
cargo fmt --all -- --check
cargo machete
cargo hack check --workspace --each-feature --locked
cargo doc --workspace --no-deps --locked
python3 .shots/test_bundle.py
python3 .shots/design_audit.py --self-test
python3 .shots/a11y_audit.py --self-test
python3 .shots/subset-fonts.py --check
python3 .shots/write_only.py
```

Run `.shots/lint.ps1` using an appropriate PowerShell host, and every `.shots/*audit.py` with output checked for actual unresolved rows. Preserve CI's rustdoc flags and host matrix when reproducing that gate. Do not describe the commands above alone as a full CI pass.

Rebuild the browser artifact from the repository root:

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web
wasm-bindgen --target web --no-typescript --out-dir web/public/gallery \
  target/wasm32-unknown-unknown/wasm-release/herogpui_web.wasm
```

Use the actual Cargo target directory if overridden. The CLI must match `wasm-bindgen` in `Cargo.lock`. Never set `RUSTFLAGS` or a nightly toolchain override for this build; preserve the vendored web platform's single-threaded default.

Then from `web/`:

```sh
pnpm run extract
pnpm run wasm:manifest
pnpm run extract:check
pnpm run check
pnpm run build
```

Run `node scripts/extract-catalog.mjs` or the existing offline `build-data.mjs` only when their inputs change; the latter also copies shots. Run `pnpm run brand:check` when brand presentation or vendored styles are affected. Verify pages visually after starting the development/production server and exercise the complete browser → WASM → callback/value → displayed result flow.

## 12. Acceptance, reporting and handoff instructions

A component is complete only when:

1. Every source-owned part and every finite variant/state case in its expanded inventory has a disposition.
2. Portable gaps have been fixed with matching source, interaction and visual evidence, including both themes and relevant geometry/motion combinations.
3. Intentional differences are named, and remaining framework limitations have concrete pinned-source evidence. They do not disappear into a green parity percentage.
4. Focused tests exercise changed behavior, and shared-helper changes pass affected sibling tests.
5. Native and WASM evidence cover their actual input paths; compilation or source matching is not substituted for observation.
6. Gallery examples are live, isolated, discoverable and representative; reference/API/docs explain the final behavior accurately.
7. Generated data and the committed live artifact agree with the exact source being handed off.
8. The current relevant gates pass, or a clearly identified external/platform blocker remains reported as incomplete.

The overall implementation goal is not complete while required specimens are unreviewed or unobserved. If a framework gap prevents full visual parity, hand off a concrete remaining-gap proposal rather than claiming full parity. Completed work should remain independently useful and reviewable.

For every slice, record: changed contracts, previous/current screenshots, source selectors and versions, tests and runtime actions, artifact identity, current limitations, and the next unresolved specimen. Keep completion status derived from evidence freshness. Revalidate a case when its implementation, upstream source, example, theme, font or runtime artifact hash changes.

Suggested instruction to the implementing agent:

> Execute `docs/parity/ui-design-plan.md` and its companion component checklist in dependency order. Start by refreshing the existing inventory and audit baseline; do not repeat already implemented features. Turn grouped examples into individually addressable specimens. For each family, read the pinned source, reproduce the mismatch, fix the owning implementation, verify pointer/keyboard/open/exit/motion states in native and WASM, and synchronize tests, gallery, metadata, public docs, website datasets and the rebuilt artifact in the same mergeable change. Preserve the existing focus-modality decision and documented customization extensions. Keep an honest remaining-case ledger and continue until all required cases are verified or a concrete platform limitation is documented. Do not mark the project complete from static audit passes or default screenshots alone.

Preparation validation: this plan was based on repository/source inspection and an official release lookup. No component code, application rendering, builds, test suites, deployment, or existing evidence status was changed while preparing it.
