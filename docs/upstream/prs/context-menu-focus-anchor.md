# ContextMenu: anchor a keyboard open at the focused element

For the components agent. **Conclusion: this can be done in HeroGPUI, with no
upstream change.** `gpui-pre` 0.3.5 has no public "bounds of the focused
element" API. It does expose everything needed to build one: a prepaint
callback with window-space bounds, focus-handle identity comparison, and a
per-app global. Each focusable HeroGPUI component records its bounds while it
holds focus. `ContextMenu` looks them up for `window.focused(cx)` and falls
back to today's area anchor.

All file:line references are to
`~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.5/src/`.

## Current behavior

`crates/herogpui-components/src/context_menu.rs`:

- `opens_context_menu` (`:132`) accepts Shift+F10, `menu` and `contextmenu`
  with no other modifier;
- the area's `on_key_down` (`:215`) anchors a zero-size rect at
  `area_bounds.origin` (`:222`). `area_bounds` is filled by an absolutely
  positioned `canvas` probe over the area (`:196-202`). The handler then saves
  `window.focused(cx)` for restoration, sets `focus_first`, and opens;
- the module docs (`:16`) say: "GPUI reports no bounds for the focused
  element, so the area is the anchor";
- tests: `crates/herogpui-components/tests/context_menu.rs`,
  `shift_f10_opens_at_the_area_with_the_first_item_focused` (`:178`, which
  asserts `bounds.origin == area.origin`), and
  `the_context_menu_key_opens_and_escape_restores_focus`.

## What gpui 0.3.5 offers, and what it does not

Usable, public:

| API | Location | Use |
|---|---|---|
| `canvas(prepaint, paint)` | `elements/canvas.rs:10` | `prepaint: FnOnce(Bounds<Pixels>, &mut Window, &mut App)` receives the element's laid-out bounds in window coordinates, every frame. `context_menu.rs` already uses this. |
| `Window::focused(&self, &App) -> Option<FocusHandle>` | `window.rs:2261` | the focused handle at key-down time |
| `FocusHandle::is_focused(&self, &Window)` | `window.rs:606` | decide at render time whether to attach the probe |
| `FocusHandle::downgrade() -> WeakFocusHandle` | `window.rs:593` | store without keeping the handle alive |
| `impl PartialEq<FocusHandle> for WeakFocusHandle` | `window.rs:687` | identity check (`FocusHandle` is `Eq` but not `Hash`, and `FocusHandle::id` is `pub(crate)` at `window.rs:528`, so there is no map key) |
| `Window::window_handle().window_id()` | `window.rs:2243`, `:7147` | key the registry per window |
| `App::try_global` / `set_global` | `app.rs:2075`, `app.rs:2105` | hold the registry |
| `Div::on_children_prepainted` | `elements/div.rs:1811` | alternative probe where a component already lays out children it owns |

Not usable, private (this is why there is no direct API):

- `window.a11y.node_bounds: FxHashMap<NodeId, Bounds<Pixels>>`
  (`window/a11y.rs:150`), filled in `element.rs:378` and paired with
  `a11y.focus_ids` (`window/a11y.rs:149`). It is `pub(crate)` and populated
  only while an assistive technology is active.
- `Interactivity` calls `window.set_focus_handle(..)` in prepaint
  (`elements/div.rs:2322`, `window.rs:5051`) with the element's bounds in
  scope, but records only the focus id (`next_frame.focus`), not the bounds.
- Hitboxes (`Window::insert_hitbox`) carry bounds but no focus association.

Avoid `App::default_global`/`global_mut` for the per-frame write. They push
`Effect::NotifyGlobalObservers` on every call (`app.rs:2083-2100`). Mutate
through a shared cell instead, as below.

## Design

### Registry (`util.rs`, crate-private)

```rust
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use gpui::{App, Bounds, FocusHandle, Global, ParentElement, Pixels, Styled, WeakFocusHandle, Window, WindowId};

/// The last painted bounds of the focused HeroGPUI element, per window.
#[derive(Clone, Default)]
struct FocusedBounds(Rc<RefCell<HashMap<WindowId, (WeakFocusHandle, Bounds<Pixels>)>>>);
impl Global for FocusedBounds {}

fn focused_bounds_registry(cx: &mut App) -> FocusedBounds {
    if let Some(registry) = cx.try_global::<FocusedBounds>() {
        return registry.clone();
    }
    let registry = FocusedBounds::default();
    cx.set_global(registry.clone()); // once per app: one observer notification
    registry
}

/// Appends an invisible, layout-free probe that records `handle`'s bounds
/// each frame while it holds the focus. A no-op when it does not.
pub(crate) fn record_focus_bounds<T: ParentElement>(
    el: T,
    handle: &FocusHandle,
    window: &Window,
    cx: &mut App,
) -> T {
    if !handle.is_focused(window) {
        return el;
    }
    let registry = focused_bounds_registry(cx);
    let weak = handle.downgrade();
    el.child(
        gpui::canvas(
            move |bounds, window, _| {
                let id = window.window_handle().window_id();
                registry.0.borrow_mut().insert(id, (weak, bounds));
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0(),
    )
}

/// The painted bounds of the currently focused element, if a HeroGPUI
/// component recorded them for this focus.
pub(crate) fn focused_element_bounds(window: &Window, cx: &App) -> Option<Bounds<Pixels>> {
    let focused = window.focused(cx)?;
    let registry = cx.try_global::<FocusedBounds>()?;
    let map = registry.0.borrow();
    let (weak, bounds) = map.get(&window.window_handle().window_id())?;
    (*weak == focused).then_some(*bounds)
}
```

Notes:

- Only the focused element attaches the probe, so there is at most one
  per window per frame. An absolute child takes no part in flex flow or
  `gap`, and taffy positions it against its direct parent, so no
  `.relative()` is needed. Append it last, after existing children, so
  `on_children_prepainted` indices elsewhere stay stable.
- The stored handle is weak. A stale entry is harmless, because
  `focused_element_bounds` checks identity against `window.focused(cx)`.
  Entries for closed windows are a few bytes each and can be ignored, or
  removed on the gallery's window-close path.
- The recorded bounds come from the last prepaint. If the focus moved and no
  frame has been drawn since, the identity check fails and the menu falls
  back to the area. This is a correct, if less precise, result.
- HeroGPUI uses no cached views (`AnyView::cached`). If one is introduced, a
  reused prepaint does not re-run the probe, so a scrolled cached subtree
  could report stale bounds.

### Recording sites

Call `record_focus_bounds` wherever a component calls
`.track_focus(&handle)` on a `Div` it owns, on that same element. There are
82 call sites in 43 files (`grep -rn track_focus
crates/herogpui-components/src`). The call-site counts are:
range_calendar(5), color_picker/picker(5), calendar(5), tabs(4), table(4),
toast(3), tag_group(3), popover(3), pagination(3), link(3),
date_picker/range(3), date_picker/picker(3), toggle_button(2), select(2),
input(2), dropdown(2), color_picker/swatch_picker(2), button(2),
autocomplete(2), and one each in util, tree_view, tooltip, toolbar,
time_field, switch, slider, resizable, radio_group, modal, menu_bar,
list_box, input_otp, drawer, date_picker/field, context_menu, combo_box,
color_picker/slider, color_picker/area, close_button, checkbox, breadcrumbs,
alert_dialog and accordion.

Priorities, if not all at once:

1. Leaf controls a context menu typically wraps: `button`, `toggle_button`,
   `link`, `checkbox`, `switch`, `radio_group`, `input`/`textarea`
   (`input.rs`), `chip`/`tag_group`, and the rows or cells of `list_box`,
   `tree_view`, `table`, `tabs`.
2. The `ContextMenu` area itself (`context_menu.rs:193`, `area_focus`).
   Then an area with no focusable content resolves to the same bounds it
   uses today.
3. Overlays and panels (`modal`, `drawer`, `popover`, `toast`): low value for
   this feature. Skip them unless it is free.

Composite widgets that keep the focus on a container and move a virtual
active item (active-descendant style) should record the **active item's**
bounds under the **container's** handle. Otherwise the menu anchors at the
whole list. The same helper works: call it on the active row's element with
the container handle.

`ring_if_focused`/`ring_overlay_if_focused` (`util.rs`) already take
`(handle, window)` in 26 files. They are tempting as a single hook, but
`ring_if_focused` is bounded on `Styled` only (no `ParentElement`), and not
every focusable element draws a ring. Keep the probe a separate call.

Arbitrary application content inside a `ContextMenu` (the tests' plain focusable
`div`, for example) records nothing and keeps the area fallback. To let apps
opt in, a public wrapper could be exported next to `ContextMenu` (for example
`herogpui_components::record_focus_bounds`). That is a public-API change: it
needs `llms.txt` and the reference data to be updated.

### ContextMenu change (`context_menu.rs:215-227`)

```rust
area = area.on_key_down(move |event, window, cx| {
    if is_open || !opens_context_menu(&event.keystroke) {
        return;
    }
    let Some(area) = area_bounds.get() else { return };
    // The focused element when a component recorded it and it lies in the
    // area; the area otherwise (unrecorded content, focus not yet painted).
    let target = crate::util::focused_element_bounds(window, cx)
        .filter(|bounds| area.intersects(bounds))
        .unwrap_or(area);
    anchor_key.set(Some(target));                 // see "Placement" below
    *restore_key.borrow_mut() = window.focused(cx);
    focus_first_key.update(cx, |focus, _| *focus = true);
    set_open(&own_key, &on_open_change_key, true, window, cx);
    cx.stop_propagation();
});
```

Placement: today the keyboard anchor is a zero-size rect at the area's
top-left, with `Placement::BottomStart` and offset 0, so the panel's corner
sits on that point. For a focused element, the native convention (Windows
Shift+F10 in lists and trees, and macOS VoiceOver's context-menu command) is to
open at the element, below it. Pass the element's full bounds as the anchor
and keep `Placement::BottomStart`: the shared positioner
(`popover::popover_with_resolved_placement`, `popover.rs:727`) places the
panel under the element and flips above it near the window's bottom edge.
The pointer-open path keeps its zero-size anchor. If the whole area is the
fallback, keep today's zero-size top-left anchor so that behavior does not
change.

The module docs (`:12-19`) need updating: "opens the menu at the focused
element when a HeroGPUI component holds the focus, otherwise at the area's
top-left corner".

### Tests to add (`tests/context_menu.rs`)

- With a HeroGPUI `Button` (or any recording component) inside the area,
  focused through `focus(..)` followed by `frame(cx)`, Shift+F10 opens with
  the panel's origin at the button's bottom-left
  (`cx.debug_bounds(..)` on the button).
- With the existing plain child (not recording), the current assertion
  `bounds.origin == area.origin` still holds. That is the fallback.
- Focus moved and Shift+F10 pressed with no `frame(cx)` between them falls
  back to the area. The identity check guards this case.
- A recorded element scrolled inside the area: after scrolling and a
  frame, the anchor follows the new bounds.

### Scope and surfaces

This is a behavior change of a public component, so per `CLAUDE.md` update
the ContextMenu gallery example text if it describes the anchor, its
`reference_metadata` entry, `llms.txt` (the keyboard-open sentence), and
regenerate `web/src/data/*.json` with `pnpm run extract`.

## Optional upstream follow-up (not required)

A general gpui API would let arbitrary content work without per-component
probes. `Interactivity::prepaint` already has both the tracked focus handle
and its bounds at `elements/div.rs:2321-2322`. Recording
`next_frame.focused_bounds = Some(bounds)` when `focus_handle.is_focused(window)`,
and exposing `Window::focused_bounds(&self) -> Option<Bounds<Pixels>>` from
`rendered_frame`, is roughly a 10-line PR in `crates/gpui/src/window.rs` and
`elements/div.rs`. It is not drafted here because the HeroGPUI-side design
above is sufficient. Propose it if application-content anchoring becomes a
requirement.
