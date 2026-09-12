# Select clear control — source and upstream evidence

Target: HeroUI v3.2.5, commit `5f13f6ed355bdbd5d5f69e5944685438a3591793`.
Recorded 2026-09-11; refreshed 2026-09-12. The port now implements the clear part and shortcuts; this is **not a completed
Select parity comparison**. [contract.json](contract.json) pins the reviewed sources,
distinguishes source contracts from observed paths, and lists remaining cases.

## Observed upstream

The live [With Clear Button demo](https://heroui.com/en/docs/react/components/select#with-clear-button)
is a Primary single-select with uncontrolled California selection. The header
in these captures identifies v3.2.5. Only temporary DOM ids and scrolling were
used to address it; its React props and CSS were not modified.

- [Rest](upstream-rest.png), [hover](upstream-hover.png), and
  [held primary press](upstream-pressed.png) were captured and visually inspected.
  The clear background changes while the trigger retains its resting fill.
  Computed [pressed state](upstream-pressed.json) confirms scale 0.93, focus on
  the trigger, California still selected and the list closed before release.
- [After release](upstream-cleared.png) and [computed state](upstream-cleared.json):
  placeholder visible, clear opacity zero and pointer-events none, 20px layout
  box retained, focus still on trigger and list closed. The pointer now reaches
  the trigger, so its hover fill becomes visible.
- [Secondary/middle clicks](upstream-secondary-buttons.json): California remains
  selected and the list stays closed after both button sequences.
- [Expanded target](upstream-expanded-edge.json): primary click at (695,316),
  one pixel left of the clear control's resting x=696 edge, clears successfully.
  Initial computed pseudo-element measurements were 24×24 with -2px inset,
  surrounding a 20×20 layout box inside a 256×36 trigger.
- [Backspace](upstream-backspace.json) and [Delete](upstream-delete.json), each
  after choosing California anew: placeholder visible, trigger focused, list
  closed. [Keyboard screenshot](upstream-keyboard.png) records the first path.

An additional open-popup Backspace/Delete attempt lost the page and the next
URL observation was `about:blank`. Its null-selector outputs and blank image
were discarded. This does not prove the component's open-popup behavior or a
component defect. That path must be rerun in a controlled local fixture.

- [Fresh geometry](upstream-geometry.json), after reloading the live demo:
  the declared 14px icon has a 12×14 bounding box because of flex shrink inside
  the 20px control's 4px padding. The indicator is 16px, eight pixels from the
  trigger's inline-end edge. Its default transition curve is
  cubic-bezier(0.4, 0, 0.2, 1), while the clear opacity uses CSS ease.
- [Light resting state](upstream-light.png), captured using the site's actual
  light-theme button after reloading, was also visually inspected. It is resting
  evidence only; the full light-theme interactive sequence is still pending.

## Port findings and implementation constraints

The starting `crates/herogpui-components/src/select.rs` had no clear composition
part, root on_clear callback, or clear shortcut. These are now implemented with
`SelectClearButton`, `Select::clear_button` and `Select::on_clear`. The onClick row belongs to the
upstream **Select.ClearButton** part; adding a root on_click would not satisfy
it. A hidden clear part still enables the shortcut by its composition, while a
plain Select remains non-clearable. Root and trigger disabled states need the
upstream precedence; the current Rust Select has only a root disabled builder.

Required status does not prohibit clearing: the upstream handler directly uses
SelectionManager.setSelectedKeys(empty), which has no disallow-empty guard.
The different SelectionManager.clearSelection method does have that guard and
must not be substituted. React Stately 3.50.0's useSelectState maps the request
to null for single mode and an empty array for multiple mode. Controlled owners
must retain control of both the displayed selection and live form value.

The existing port also uses a flow-positioned chevron and swaps its SVG path;
the reference already marks rotation and trigger property transitions partial.
Pinned GPUI 0.3.3 **does** provide Svg.with_transformation and
Transformation::rotate; the spinner already uses it. The general absence of a
div paint transform is not a reason to omit this SVG motion.

Both variants, full light-theme state coverage, controlled/multiple selection, disabled/required
composition, custom clear content, pointer cancellation, callback order,
intermediate animation frames, reduced motion and native input remain pending.


## HeroGPUI implementation and exercised paths

The new gallery section uses Primary single selection and Secondary multiple
selection with uncontrolled seeds. Both [light](port-light.png) and
[dark](port-dark.png) resting views were visually inspected. The clear part uses
its own click callback after root selection/on_clear reporting, while its
keyboard shortcut keeps focus on the trigger and does not call the part's
pointer callback. Repeated clear parts have distinct instance-derived state.

[Pressed](port-pressed.png) and [cleared](port-cleared.png) show the single-select
path. [Expanded-edge click](port-expanded-edge.png) clears at (719,292), one pixel
inside the 24px target and outside the default 20px visual. The list stays
closed. Keyboard testing opens the Secondary multiple list, escapes, and clears
Rust and Go together with Backspace. A missing keyboard focus ring was found in
the first screenshot: stopping propagation skipped app_focus_root's modality
update. A regression assertion failed before adding set_focus_visible to the
consumed clear key path. The final keyboard capture uses artifact `8dd94fb1feb5` after that fix. Earlier
light/press/release captures used `273dbd4c5365`; dark/edge captures used
`3efce8a999f8`. Their pointer behavior is unchanged in the final artifact.

Four focused tests exercise both variants and selection modes, controlled
owners accepting/refusing requests, required fields, composed/plain controls,
disabled and open gates, Backspace/Delete, repeated empty clears, two clear
parts, callback order, retained layout, expanded hit testing, pointer down,
secondary/middle buttons, cancellation and empty-target click fallthrough.
ParentElement composition is exercised with a custom child.

The default visual scales while pressed, but custom descendants do not yet
scale as a subtree, and the interactive target stays 24px during a press rather
than shrinking with the CSS transform. Both remain explicitly partial in the
reference. Separate trigger-disabled override, complete variant/theme/state
coverage, opacity intermediate frames/reversal, native input and the existing
chevron/trigger transition gaps remain pending. Pointer and keyboard behavior
proof here comes from native headless GPUI tests and the shared WASM gallery.

Final checks: 1,798 component tests, 133 gallery library tests, native workspace
and stable WASM builds, workspace Clippy with warnings denied, format and
extraction/manifests passed. The final browser showed the expected keyboard
focus ring and reported no errors. No real native pointer dispatch or complete
motion-frame comparison is claimed.
