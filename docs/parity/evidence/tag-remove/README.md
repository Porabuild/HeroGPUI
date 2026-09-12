# Tag remove target: v3.2.5 evidence

Captured 2026-09-11 with agent-browser, at 1280×633. These images cover the
remove-target change, not every Tag state or animation. Labels and surrounding
demo composition differ, so they are not whole-page pixel-diff references.

Upstream: `https://heroui.com/en/docs/react/components/tag-group`, With Remove
Button section. The site reports v3.2.5. The visible remove buttons have the
classes `close-button close-button--default tag__remove-button`, a computed
12×12 layout box and a 24×24 `::after` target, agreeing with the tagged source.

HeroGPUI: rebuilt workspace WASM, artifact hash prefix `b0d88e7904d5`, served
from the local site at `/gallery/index.html?preview=component&section=With%20Remove%20Button&story=tag-group&theme=light&v=b0d88e7904d5`
(or `theme=dark`). The native gallery was separately observed on the same
section, but native clicks failed at the CUA driver with `noWindowsAvailable`.

| Image | Observed state |
|---|---|
| [GPUI light resting](gpui-light-rest.png) | Default and custom remove controls retain the small slot |
| [GPUI light hover](gpui-light-hover.png) | Pointer at (530,271), right of Design's glyph, within its expanded target; fill remains on the small visual |
| [GPUI pointer result](gpui-light-pointer-removed.png) | Clicking that location removes Design; the two demos share the gallery's tag data |
| [GPUI dark focus](gpui-dark-focus.png) | Click Design's body at (485,271), then Tab: ring surrounds the remove glyph |
| [GPUI keyboard result](gpui-dark-keyboard-removed.png) | Enter removes Design and focuses the next tag |
| [Upstream dark resting](upstream-dark-rest.png) | Actual v3.2.5 default and custom remove examples |
| [Upstream dark hover](upstream-dark-hover.png) | Pointer at (452,275), outside News button's right edge (448.34), inside the expanded target |
| [Upstream pointer result](upstream-dark-pointer-removed.png) | The expanded-edge click removes News |
| [Upstream dark focus](upstream-dark-focus.png) | Tab after removal focuses Travel's 12×12 remove button; ring surrounds the small control |

No capture is evidence of the untested composed Autocomplete/ComboBox path,
every tag variation's animation, or native pointer dispatch. Focus after actual
removal differs in ownership between native keyed state and React Aria; the
existing collection tests document that pre-existing port policy.
