# Component implementation

Read this guide before changing `crates/herogpui-components`. It records the
GPUI 0.2.2 constraints and test patterns that repeatedly produce plausible but
incorrect ports.

## Component model

Components are builder structs deriving `IntoElement` and implementing
`RenderOnce`:

```rust
fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement
```

Controlled props are caller-owned and must report changes. Uncontrolled props
seed keyed internal state through `default_*`. A constructor argument is an
uncontrolled seed, not a controlled value. Never initialize the field consumed
by `util::controlled` with `Some(..)` from `new`; doing so freezes interaction
unless the caller happens to drive it.

When an empty value is valid in controlled mode, record control mode with a flag
set by the controlled builder. Do not infer it from whether the stored value or
uncontrolled default is empty; that creates a control owned by nobody.

A documented builder that stores a value but never changes rendering or
behavior is not partial support. Implement it or remove it and record the exact
reason in the parity audit.

## State lifetime

- State that must survive a repaint or connect two events belongs in keyed
  state (`window.use_keyed_state` through the local helpers). An `Rc<Cell<_>>`
  created during `render` lasts one frame; a press itself may repaint before the
  move or release arrives.
- Derive every keyed-state key from the component's instance id. Bare literal
  keys make separate controls share state.
- Two controls created from the same helper or loop still need distinct ids;
  `#[track_caller]` is not an instance identity.
- Live form state must survive builder reconstruction. Render-time snapshots do
  not track current values, validity, focus, reset, or successful/disabled
  status.
- Disabled, read-only, and inert are distinct. Disabled controls do not activate
  or enter the tab order. Read-only controls remain focusable and navigable but
  reject mutations, and they still participate in form submission.

## Events and focus

- `on_click` and `.active(..)` require a stateful element: assign `.id(..)`
  first. `.hover(..)` does not.
- GPUI activates a focused clickable element on Enter and Space. Do not bind a
  second key handler for the same action or it will fire twice.
- A root key handler must verify that its own focus handle owns the event before
  acting; keys from focused descendants bubble.
- Gate `track_focus` for disabled controls so they leave the tab order.
- `tab_group` orders stops but does not trap focus. Use `util::trap_tab` for
  dialogs and stop propagation so `util::app_focus_root` does not advance twice.
- A component that reads its own focus state uses `util::tab_stop_handle` so the
  handle it tracks is also the tab stop; `.tab_index(0)` creates a different
  internal handle.
- Roving controls use one focus handle whose `track_focus` owner moves. A handle
  per row loses the collection's single-tab-stop semantics.
- Pointer coordinates are window-relative. Subtract the element's laid-out
  origin before converting a point into a local value.
- A drag that can leave its hitbox needs keyed drag state and paint-time
  `Window::on_mouse_event` listeners re-registered each frame. GPUI 0.2.2 has no
  `Window::capture_pointer`.
- Add `.occlude()` when a floating control covers another interactive element;
  GPUI otherwise lets a press reach the element underneath.

## Overlays

- Render floating surfaces with `util::floating`/`anchored` so later siblings do
  not paint above them. `absolute` alone does not change tree paint order.
- Put `on_mouse_down_out` on the panel whose bounds define "outside", not on an
  empty absolute wrapper or a full-window backdrop.
- Route Escape and outside presses through `util::overlay_scope`; only the
  topmost active overlay may dismiss.
- Use `util::panel_focus` only for surfaces that own focus, and pass the real
  open state. Claiming focus on a closed frame spends the one-shot. Pickers that
  leave focus in an inner field/grid handle Escape at that owning root instead.
- Composite menus and submenus are a union of real trigger/panel bounds, not the
  bounding rectangle around them.
- Keep exit surfaces mounted through `util::overlay_phase`; an immediately
  removed `RenderOnce` tree cannot animate out.
- Trigger-versus-outside latches are dispatch-local. Clear them with
  `App::defer`, not on click and not on a later frame.
- Do not focus a trigger during the same Enter dispatch that selected a row;
  GPUI may activate the newly focused trigger and reopen the surface.

## Layout, rendering, and animation

- `Pixels` has a private inner field; use `f32::from(value)`.
- GPUI 0.2.2 has no div-level rotate/scale, `select_none`, `grow`, or
  `uppercase`. Verify alternatives against installed 0.2.2 source.
- `svg()` does not inherit text color. Set `.text_color(..)` explicitly.
- A new icon needs both a constant in `components/src/icons.rs` and an asset
  registration in `gallery/src/assets.rs`.
- Custom components that hold children need an explicit `ParentElement::extend`
  implementation.
- Branches returning different element types need `.into_any_element()`.
- Clone theme tokens before a nested `render(window, cx)` call rather than
  holding a borrow from `cx.colors()`.
- Theme tokens are already `Hsla`; do not wrap them in `Hsla::from(..)`.
- Divs are block layout by default. Put controls in a flex parent when they
  should hug content like HeroUI's `inline-flex` elements.
- GPUI has no ancestor context propagation. Composite owners such as `Form`
  receive child state explicitly instead of discovering it through the tree.
- `overflow_y_scroll()` requires an id. A `w_full` child gives a scroller no
  horizontal range; use `min_w_full().flex_shrink_0()` where content may exceed
  the viewport.
- Percentage `max_h` resolves against the parent content box. Use an absolute
  viewport-derived cap when the contract is about the window.
- A GPUI shadow with zero blur paints transparent. Focus rings use at least a
  one-pixel blur, and `shadow()` replaces existing shadows rather than appending.
- Table focus is inset. Use `util::inset_focus_ring`; an outset ring bleeds into
  adjacent cells.
- Keep the listener/state-owning element's id stable across animations. Animate
  a child, not a wrapper whose animation generation changes the full id path.
- Use the repository's `Curve` cubic-bezier implementation for v3 easing tokens;
  do not substitute GPUI's nearest built-in easing.
- Paint and interaction must use the same color model, axis, range, and
  orientation. Preserve hue endpoints and degenerate black/white states.

## Virtual collections

- `uniform_list` is for fixed-height virtual rows and requires an explicit
  height. Its callback is `'static`, so move owned data and copied tokens into
  it; do not borrow `self` or `cx.colors()`.
- Share one row builder between plain and virtual paths so rendering does not
  drift.
- `AnyElement` is single-use. Tables with element cells take a row factory, not
  prebuilt elements.
- A virtual factory is not a cheap iterator. Do not call every row factory to
  fingerprint a collection; carry an explicit stable collection identity.
- A load-more sentinel can compare its canvas bounds with
  `window.content_mask()` after ancestor clipping. Key `(was_visible, row_count)`
  so continuous visibility fires once and appended rows re-arm it.

## Behavior tests

Use the headless harness in `tests/harness/mod.rs`. It initializes the theme,
opens a GPUI test window, rebuilds one `RenderOnce` component per frame, and
records callbacks.

```powershell
cargo test -p herogpui-components --test overlays
cargo test -p herogpui-components <test_name>
cargo test -p herogpui-components
```

Harness rules:

- Call `ThemeProvider::init` before opening the window.
- `simulate_keystrokes` sends key-down only; `harness::press` adds the key-up
  needed for focused click activation.
- GPUI does not implement app-level Tab movement. The harness reproduces the
  minimum behavior of `util::app_focus_root`.
- A drag is down, move with `pressed_button`, and up, with refreshes between
  events because hit testing uses the last rendered frame.
- Set reduced motion before the first frame. After advancing past an exit,
  force another update before probing the old panel location.
- Modifier chords work in the headless harness, but `Keystroke::parse` uses
  `-` syntax. Posted gallery input has different modifier limitations.
- Scrolling down uses a negative pixel `dy`. Refresh after every mouse event.
- Coordinates may come from stable component constants, but measure label text
  and remeasure after error content changes layout.
- Prove a closed surface by clicking where its row used to be and asserting no
  callback, not merely by checking an open-change callback.

An intentionally failing test is useful only after its expectation is checked
against the exact upstream contract. Correct the expectation when it encodes
the bug; do not distort the component to make a bad test pass. After the focused
headless test, drive the same user path in the gallery when the failure depends
on real composition, paint order, or hit testing.
