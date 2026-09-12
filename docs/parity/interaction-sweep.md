# Interaction and animation sweep

## Target and evidence boundary

The requested target is the latest HeroUI, including every component variation,
interactive state, animation, and visual presentation. On 2026-09-11 the
[official release index](https://heroui.com/en/docs/react/releases) lists
[v3.2.5](https://heroui.com/en/docs/react/releases/v3-2-5), released September 8.
`git ls-remote --tags https://github.com/heroui-inc/heroui.git 'refs/tags/v3.2.*'`
confirms its annotated tag resolves to
`5f13f6ed355bdbd5d5f69e5944685438a3591793`.
The downloaded tag's `packages/react/package.json` also reports 3.2.5.

The user explicitly approved upgrading to v3.2.5. Audit inputs, active guides,
reference source links and the web dependency now target that release. The
upstream lockfile resolves React Aria 3.52.0, React Aria Components 1.21.0 and
React Stately 3.50.0; the web's two direct Aria dependencies are pinned exactly.
Historical tests still cite the versions against which they were written;
those citations are not proof of revalidation against the newer dependency.

The new CSS archive comes from the tagged source, including both theme files.
CSS caches now include release and archive hash so a different release or
worktree cannot silently reuse the old styles. `python .shots/test_bundle.py`
verifies old-cache isolation, archive replacement and rejection of an old docs
bundle. Both docs and CSS inputs are checked in for offline audits.

Changing these pins does **not** establish completed parity. The first full
audit run against v3.2.5 reported seven API gaps (Select callbacks, Tabs align,
Toast options/update), an unowned Select.ClearButton table, three missing
examples (Select clear button, Tabs alignment, Toast expanded stack), and
missing Toast exiting/expanded/hidden states. The state audit exits 1. Some
other audits print gaps but exit 0; exit status alone is not a clean verdict.
Design and animation mappings reported no mismatches, which only proves their
mapped coverage. The reference metadata version denotes the target contract;
its existing status rows still need the specimen-by-specimen review below.

The [previous sweep](../parity-sweep.md) compares native output with native
goldens, mostly for Usage. It is useful regression evidence but does not prove
visual equivalence to upstream or coverage of every variation.

## Inventory

[interaction-inventory.json](interaction-inventory.json) records separately:

- Every current gallery section from `web/src/data/rust-examples.json`, with
  its code hash and the source dataset hash.
- Every English upstream demo TSX file under the v3.2.5 tag, with its hash.
- Every changed file under `packages/react/src` and `packages/styles` between
  the v3.2.4 and v3.2.5 tags, including added and removed files.

All entries start **unreviewed**. These are inventories, not matched pairs or
passing checks. Grouped examples such as Sizes and Variants must be expanded
into each rendered specimen. Upstream stories, CSS modifiers, prop combinations,
and inherited behavior can require cases absent from either demo inventory.
The inventories deliberately retain different upstream and gallery naming;
map composition parts explicitly instead of treating unmatched names as absent
components. Refresh the inventory if its source hashes change.

## Confirmed source discrepancy

Tag removal: v3.2.5's `packages/styles/components/tag.css` adds `touch-target`
to `.tag__remove-button`. The utility expands the interactive region to at
least 24×24 while retaining the 12×12 layout box and glyph. The starting
`crates/herogpui-components/src/tag_group.rs` assigned the remove element
`.size(px(12.))` with no corresponding expansion. This was a source-confirmed
gap in the starting tree. It is now fixed with a 24px absolute action inside a
12px layout slot. The glyph, hover fill and keyboard ring remain on the 12px
visual child. Increasing the layout size alone would change chip spacing and
would not reproduce the upstream utility.

The new `remove_target_extends_without_changing_tag_layout` test failed on the
original implementation and passes after the fix for all sizes, both variants,
and default/custom content. It checks unchanged width/height and all four
extended edges without selecting the tag body. The disabled/neighbor test
covers group disable, per-tag disable and disabled keys. Existing Tag
collection-contract tests pass, including Enter/Space removal, focus continuity
and selection isolation. The rebuilt WASM gallery also removes Design on a
click at (530,271), outside its glyph, and removes it with Enter after keyboard
focus. The live upstream v3.2.5 page removes News on the corresponding expanded
edge at (452,275). Its computed button is 12×12 and its `::after` target is
24×24. Dark-mode screenshots show the small remove focus ring on both sides.
See [capture notes and images](evidence/tag-remove/README.md).

The rebuilt native gallery's With Remove Button page was visually observed via
CUA, but native input dispatch remains unverified: CUA can capture this window
yet returns `noWindowsAvailable` for clicks. This is a driver limitation, not
an observed component failure. Composed picker proof and the remaining Tag
states/animations are still pending; this is not a fully verified component
verdict.

Verification must cover removal just outside the glyph, no accidental tag
selection, disabled tags, all sizes, adjacent tags, custom remove content,
keyboard removal, and tags composed inside multi-select controls.

## Completed checks for this change

- Full component test suite: passed, including doctests.
- Gallery library tests: passed.
- Native workspace build and stable `wasm-release` build: passed. The
  Windows-specific rebuild script cannot run on this macOS host, so native
  validation used `cargo build --workspace` and launched that exact output.
- Matching `wasm-bindgen` 0.2.127 output and WASM manifests: regenerated.
- Web extraction checks, typecheck, lint/format checks and production build:
  passed. Web lint still prints warnings in unchanged files.
- Workspace Clippy with warnings denied, workspace lint inheritance, Rust
  format, diff whitespace, cache tests and a11y scanner self-test: passed.
- Both deep-review lanes found no introduced defect. A stale audit comment
  was corrected to distinguish its historical source review from the new target.
- Full audit set: ran; the outstanding v3.2.5 gaps above remain visible.
  This is **not** a passing full-parity gate.

## Tabs alignment follow-up

Implemented `TabsAlign::{Start, Center, End}` with Center default in both
variants. Alignment is scoped to tab content; indicator boxes and panel styles
are unchanged. The gallery adds six vertical specimens. API docs, reference
metadata, both generated datasets and the committed WASM were synchronized.
The inventory now contains 693 gallery sections.

[Inspected captures and scope notes](evidence/tabs-align/README.md) include the
light/dark port matrix, keyboard selection, the actual upstream demo and six
upstream CSS fixture variants. These are partial evidence, not a complete Tabs
verdict. In particular, upstream wraps constrained vertical labels beside a
panel; the port's single-line/intrinsic sizing can widen the list. Removing
nowrap alone did not resolve this. The reference now explicitly marks that
style contract partial. Nested/horizontal visual proof, overflow behavior,
native input and full state/motion comparisons remain pending.

The final focused Tabs binary passed all 27 tests, covering alignment across
variants/orientations, disabled skipping, callbacks and indicator geometry.
The full component suite, 133 gallery tests, native workspace build, stable
WASM build, Clippy with warnings denied, extraction/manifests, format and diff
checks passed. The design reader's updated Tabs radius anchor has positive and
negative self-test fixtures; no design values are unreadable or mismatched.
Both deep-review lanes found no introduced defect.

The latest audit run reports six API gaps (Select and Toast), two missing
examples (Select and Toast), and the same three missing Toast states. Some
audits exit zero despite reporting gaps; these results do not establish full
parity. See `audit-results.json` for the current structured report.

## Select clear contract review

Reviewed the v3.2.5 implementation, CSS, demo, upstream unit/browser tests and
React Stately 3.50.0 selection manager/state. [Source and runtime evidence](evidence/select-clear/README.md)
now records the missing clear part's input, focus, callback and motion contract.
The live single-select demo confirms release-only clearing, secondary/middle
button isolation, the expanded pointer target, retained layout when hidden and
both closed-trigger keyboard shortcuts. That review informed the implementation follow-up below.

The next implementation must preserve part ownership (`onClick` on ClearButton,
`onClear` on Select), controlled values, whole-set multiple clearing, required
clearing, resolved disabled precedence, hidden-layout behavior, trigger-hover
exclusion and the 0.93 pressed transform. Open-popup runtime proof was interrupted
by browser navigation and remains pending, as do the remaining variations.

## Select clear implementation follow-up

Added the optional `SelectClearButton` part, root `on_clear`, part `on_click`,
and closed-trigger Backspace/Delete clearing. The selection request respects
controlled ownership and clears all selected keys in multiple mode. Required
fields can clear, disabled fields cannot; hidden controls retain layout without
intercepting the trigger. Pointer down, canceled presses and secondary/middle
buttons do not clear. Default visual pressing, immediate hiding and a 150ms
reappearance tween are wired with instance-keyed state.

The gallery adds Primary single and Secondary multiple specimens, and API/docs,
metadata, datasets and WASM are synchronized. Runtime screenshots caught a
keyboard focus-ring omission after stopped propagation; the regression test
failed before explicitly recording keyboard modality in the clear handler.
See [evidence and remaining limits](evidence/select-clear/README.md).

Final verification passed: 1,798 component tests including doctests, 133 gallery
library tests, native workspace build, stable WASM build, workspace Clippy with
warnings denied, Rust formatting, extraction/manifests and diff whitespace.
The Windows-only rebuild driver is unavailable on this macOS host; native build
validation used `cargo build --workspace`. The committed WASM artifact prefix is
`8dd94fb1feb5`. Browser inspection confirmed the repaired focus ring, both shown
themes, default press/clear visuals and the expanded pointer target, with no
reported browser errors. The two review lanes found no remaining functional
regression; custom-child scaling remains a disclosed styling gap. An in-memory
negative audit check removed the ClearButton owner mapping and correctly
reported it as unowned, then restored the mapping.

The API audit now reports only four Toast gaps; the example audit only lacks
Toast's Expanded Stack. Toast's three state gaps remain. The inventory contains
694 gallery sections. This is not a complete Select or full-parity verdict:
custom clear subtree scaling, pressed target shrink, trigger-disabled overrides,
remaining variations and motion frames still need work.

## Toast update follow-up

Implemented `toast.update` as `Toast::update(id, cx)`. Content fields replace
the existing card; omitted `timeout` / `onClose` stay; a missing id adds a new
toast. Stack order is unchanged. Expanded-stack hover, `isExpanded`,
`exitDuration` and hotkeys remain open.

## Focus-ring modality (intentional HeroUI deviation)

HeroUI / React Aria treat Escape and restored trigger focus as `focus-visible`.
The user rejected that: rings must appear only after switching to keyboard
controls, never while continuing with the pointer. `app_focus_root` now leaves
modality unchanged on Escape and modifier-only keys. Collection, calendar and
OTP cursor rings are gated on the same flag, so a pointer hover or pick no
longer paints `status-focused`. Keyboard sessions still keep the ring through
Escape; a later pointer press hides it. See `tests/focus_visible_deep.rs`.

## Next source reviews

The v3.2.5 release introduces or changes these user-visible contracts. Read the
tagged implementation before treating release prose as an implementation spec:

| Area | Required comparison |
|---|---|
| Toast | Collapsed stack; hover/focus expansion; timer pause/resume; update/reset; exit timing; hotkeys; every placement |
| Select | Clear affordance, onClear callback, closed-popup Backspace/Delete, single/multiple selection, disabled/required states |
| Tabs | Start/center/end alignment, nested lists, vertical orientation, overflow and keyboard activation |
| Popover-level overlays | Exiting surfaces must not intercept a newly opened dialog; stacking and focus restoration |
| Drawer | Nested drag ownership, all placements, interruption, reduced motion |
| Table | Typeahead with nested editable controls, navigation behavior, non-editable cells |
| ScrollShadow | Initial paint, content resize, controlled state, callback transitions, Tabs integration |
| NumberField | Missing stepper slots must not reserve columns |
| Accordion | Hover leave with custom background, trigger transitions |
| Shared styles | Focus-visible paths, scoped theme tokens and shadows, label composition, text sharpness after overlay animations |

## Required proof for each specimen

Record the upstream source and matching Rust/gallery instance first. Compare
resting, hover, pointer-down/up, keyboard focus, activation, and state change;
exercise disabled, read-only, invalid, selected, controlled and uncontrolled
modes wherever supported. Overlay cases additionally need enter, exit,
outside dismissal, Escape, nested layers and focus return. Drag cases need
capture outside bounds, endpoints, cancellation and reversal. Motion needs
intermediate frames, duration/easing evidence, interruption/reversal and reduced
motion. Compare both themes and relevant orientations/placements.

A completed row needs real upstream and HeroGPUI images of the same state,
observed interaction results, source evidence for timing, and focused regression
tests for fixes. Use native runtime evidence as well as the shared WASM gallery
where platform dispatch matters. A static audit, native golden, section title,
or successful compilation alone cannot mark a row verified.

After fixes, synchronize the affected API docs, gallery and reference metadata,
regenerate website datasets, rebuild the native gallery and committed WASM
artifact with matching manifests, and run the applicable repository gates.
Do not mark the overall goal complete while any required variation or state
remains unreviewed, unobserved, or backed only by indirect evidence.
