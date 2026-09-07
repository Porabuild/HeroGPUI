# Upstream: `gpui_web` shift+wheel horizontal scroll and IME mirror resync after paste

HeroGPUI ships a vendored fork of the published `gpui-pre-web` 0.3.3 crate at
`crates/gpui_web/`, delivered by `[patch.crates-io]` in the workspace root.
The fork is **two changes, +9/-5 lines of code** (+26/-5 counting the 17 lines
of comment that explain them), both in `src/events.rs`,
both marked `HeroGPUI fork:` at the site and recorded in
`[package.metadata.herogpui-vendor]`.

This file exists so the fork can disappear. It carries the isolated patch and a
ready PR description; once either change lands upstream, drop the corresponding
hunk from the vendored crate, and once both land, delete `crates/gpui_web/`, the
`exclude` entry, and the `[patch.crates-io]` table.

## Where to send it

Both changes are Zed's code. The PR goes to
[zed-industries/zed](https://github.com/zed-industries/zed) against
`crates/gpui_web/src/events.rs`. `gpui-pre` is zed-industries' own snapshot
publish of that tree and carries no independent history to patch, so the
snapshot's `zed-rev` is the revision to open the PR against.

Verified still needed at Zed rev `5b055fa` — the revision
`gpui-pre-web` 0.3.3 snapshots, recorded in that crate's
`[package.metadata.gpui-pre] zed-rev`: `register_wheel` still forwards the raw
axes, and neither paste path in `register_paste` calls
`schedule_ime_mirror_sync`, though seven other call sites in the same file
already do. Read that rev out of the crate rather than assuming it: at 0.3.3
`gpui-pre` proper snapshots the same `5b055fa`, but the two are separate stamps
and have no guarantee of agreeing. The published `src/` tree is byte-identical
between 0.3.1 and 0.3.3 — the Zed rev moved from `801c087` to `5b055fa` with no
change to this crate — so the re-base at 0.3.3 was the two hunks re-applied to
the same bytes.

Size the change against the published sources, never against a Zed checkout at
a different revision — unrelated upstream work (pointer-cancel and touch-id
handling, selection-change listeners, gesture tuning, virtual-keyboard policy)
swamps the fourteen touched lines:

```
diff -rq crates/gpui_web/src \
  ~/.cargo/registry/src/index.crates.io-*/gpui-pre-web-0.3.3/src
```

Only `events.rs` differs; the other nine source files are byte-identical.
Unlike the 1.18.1 crate this replaced, 0.3.3 ships no `assets/` tree and no
`README.md` — its published file list is `Cargo.toml`, `Cargo.toml.orig`,
`Cargo.lock`, `LICENSE-APACHE` and the ten files under `src/`. Its
`WebPlatform` starts with an empty font database, and
`crates/herogpui-web` supplies the gallery's fonts through
`App::text_system().add_fonts(..)`.

## Change 1 — map shift+wheel to horizontal scroll

A mouse wheel reports only `deltaY`. Every desktop platform reads shift+wheel
as the horizontal axis, and GPUI's native platforms deliver it that way, so
`ScrollHandle`-based horizontal regions work with a mouse on macOS, Linux and
Windows. On the web platform the raw axes are passed straight through, so a
horizontally scrollable region — Tabs overflow, a wide Table, a code block —
cannot be scrolled at all with a wheel. There is no keyboard or pointer
fallback: the content is simply unreachable.

A trackpad already sends a real `deltaX` for a two-finger horizontal gesture,
and some browsers additionally synthesize `deltaX` for shift+wheel themselves.
So the swap is taken only when `deltaX` is exactly zero, which leaves both of
those cases untouched and makes the change a pure addition for the
wheel-only case.

## Change 2 — refresh the IME mirror after both paste paths

`register_paste` calls `event.prevent_default()` and hands the clipboard to
GPUI's input handler, so the browser never performs the edit on the hidden
mirror textarea. The mirror keeps its pre-paste text while the application's
buffer moves on.

The next `beforeinput` or composition event is diffed against that stale mirror
content, so the diff describes an edit that no longer makes sense against the
real buffer: pasted text gets duplicated, or the following keystroke eats it.
The rest of the file already treats this as an invariant — `schedule_ime_mirror_sync`
is called after cut, after the other prevented edits, and after selection
changes. The paste paths are the two that were missed.

Both need it: the text-only path completes synchronously, and the
image/mixed path resolves inside `spawn_local` a frame or more later, so a
single call at the end of the handler would fire before the async paste has
happened.

## The patch

Against `crates/gpui_web/src/events.rs` at Zed rev `5b055fa`:

```diff
@@ fn register_wheel(self: &Rc<Self>) -> EventListenerHandle {
+            // A mouse wheel reports only `deltaY`, and every desktop platform
+            // treats shift+wheel as the horizontal axis. A trackpad already
+            // sends a real `deltaX`, so the swap is taken only when `deltaX`
+            // is zero and a true horizontal gesture is left untouched.
+            let (delta_x, delta_y) = if modifiers.shift && event.delta_x() == 0.0 {
+                (event.delta_y(), 0.0)
+            } else {
+                (event.delta_x(), event.delta_y())
+            };
             let delta_mode = event.delta_mode();
             let delta = if delta_mode == 1 {
-                ScrollDelta::Lines(point(-event.delta_x() as f32, -event.delta_y() as f32))
+                ScrollDelta::Lines(point(-delta_x as f32, -delta_y as f32))
             } else {
-                ScrollDelta::Pixels(point(
-                    px(-event.delta_x() as f32),
-                    px(-event.delta_y() as f32),
-                ))
+                ScrollDelta::Pixels(point(px(-delta_x as f32), px(-delta_y as f32)))
             };

@@ fn register_paste(self: &Rc<Self>) -> EventListenerHandle {
             if image_files.is_empty() {
                 if let Some(text) = text {
                     this.with_input_handler(|handler| {
                         handler.paste(ClipboardItem::new_string(text));
                     });
+                    // Paste cancels the DOM edit, so the mirror still holds
+                    // the pre-paste text. Refresh it from the application's
+                    // replacement before the next IME edit is diffed against
+                    // it, or that text gets duplicated or eaten.
+                    this.schedule_ime_mirror_sync();
                 }
                 return;
             }
@@
                 this.with_input_handler(|handler| {
                     handler.paste(ClipboardItem { entries });
                 });
+                // Same fix for the async image/mixed paste, which resolves a
+                // frame or more after the handler returns.
+                this.schedule_ime_mirror_sync();
             });
         })
     }
```

`modifiers` is already bound a few lines above the wheel hunk
(`modifiers_from_wheel_event`), and `schedule_ime_mirror_sync` is already a
method on the same type, so neither hunk adds an import or a helper.

## PR description

> **`gpui_web`: shift+wheel scrolls horizontally, and the IME mirror is
> refreshed after paste**
>
> Two small fixes to the web platform's event handling, both in
> `crates/gpui_web/src/events.rs`. They are independent and can be split if
> preferred.
>
> **1. `register_wheel`: shift+wheel maps to the horizontal axis.** A mouse
> wheel reports only `deltaY`, and every desktop platform — including GPUI's
> own native platforms — reads shift+wheel as horizontal scroll. The web
> platform forwards the raw axes, so on the web a horizontally scrollable
> region cannot be scrolled with a wheel at all, with no fallback. The swap is
> taken only when `deltaX` is exactly zero, so a trackpad's real horizontal
> gesture, and browsers that already synthesize `deltaX` for shift+wheel, are
> unaffected.
>
> **2. `register_paste`: call `schedule_ime_mirror_sync` after both paste
> paths.** The handler calls `prevent_default()` and routes the clipboard
> through the input handler, so the browser never applies the edit to the
> hidden mirror textarea and the mirror keeps its pre-paste content. The next
> `beforeinput`/composition event is then diffed against stale text, which
> duplicates or eats the pasted string. Seven other call sites in this file
> already resync the mirror after a prevented edit; the two paste paths were
> missed. Both are patched because the image/mixed path resolves inside
> `spawn_local`, after the synchronous handler has returned.
>
> Verified in a real browser against a GPUI web app with wheel-scrollable Tabs
> and Table regions and a text input driven by IME composition after a paste.
> No API change, no new dependency, no effect on native targets.
