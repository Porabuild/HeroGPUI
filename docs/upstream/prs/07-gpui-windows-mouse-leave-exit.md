# 07: gpui_windows: Emit pointer exit and out-of-window move on `WM_MOUSELEAVE` / `WM_NCMOUSELEAVE`

Patch: [`07-gpui-windows-mouse-leave-exit.patch`](07-gpui-windows-mouse-leave-exit.patch).
Base: `gpui-pre-windows 0.3.11` (`zed@f816e1f`). File: `crates/gpui_windows/src/events.rs` (+78/-2).

`git apply --check`: OK on the `gpui-pre-windows 0.3.11` source tree (`crates/gpui_windows`).
Cross-compile `cargo check -p gpui-pre-windows --tests --target x86_64-pc-windows-msvc`: OK (with `llvm-rc`). Live Windows message-loop verification must be run on Windows.

Still needed at `f816e1f` (`gpui-pre 0.3.11`): yes. `handle_mouse_leave_msg` clears `self.state.hovered` and notifies the cursor-visibility callback, but never updates `mouse_position` or emits `PlatformInput::MouseMove` / `PlatformInput::MouseExited`, leaving hover states stuck when the pointer leaves fast or across window edges (`events.rs:395-409`).

---

**PR title:** `gpui_windows: Emit MouseExited and update pointer position on WM_MOUSELEAVE`

**PR body:**

When the pointer leaves a Windows window (or transitions across the non-client frame on `WM_MOUSELEAVE` / `WM_NCMOUSELEAVE`), `WindowsWindowInner::handle_mouse_leave_msg` currently sets `self.state.hovered` to `false` and invokes `callbacks.hovered_status_change`, but does not dispatch `PlatformInput::MouseMove` or `PlatformInput::MouseExited` to `callbacks.input`.

Without those events:
- elements using `on_hover` / GPUI `hovered()` state keep thinking the pointer is over their last in-window coordinates until the pointer re-enters;
- tooltip/hover-card open or close timers that rely on hover-exit delivery miss the leave transition;
- when Windows leaves the OS cursor coordinates just inside client bounds even though `WindowFromPoint` points to another window (for example, a floating popup or external window placed above the client area), `self.state.mouse_position` stays inside the control hitbox.

This updates `handle_mouse_leave_msg(&self, handle: HWND)`:
- queries `GetCursorPos`, `WindowFromPoint`, `IsChild`, `ScreenToClient`, and `GetClientRect` to determine `pointer_over_window` and `position_inside_client`;
- updates `self.state.mouse_position` (falling back to `logical_point(-1.0, -1.0, scale_factor)` if coordinates could not be read or the cursor is now over another window while still geometrically inside client bounds);
- emits `PlatformInput::MouseExited(MouseExitEvent)` first (when `!pointer_over_window`, while GPUI's `mouse_hit_test` still references the previously hovered elements so `on_mouse_exit` listeners fire), followed by `PlatformInput::MouseMove` with `pressed_button` (`current_pressed_button()`) and `modifiers` to update `mouse_hit_test` out of the window.

Motivation: HeroGPUI controls and downstream Windows apps (including Poratake) rely on prompt hover cleanup when the pointer leaves a compact window or crosses over another window.

Testing: `cargo test -p gpui_windows` on Windows, plus manual verification moving the cursor rapidly out of a window with hovered buttons/tooltips and behind an overlapping window.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: remove the corresponding local `events.rs` workaround in downstream Windows forks (`gpui-pre-windows`).
