# 05: gpui_web: Resync the IME mirror after paste

Patch: [`05-gpui-web-ime-mirror-resync-after-paste.patch`](05-gpui-web-ime-mirror-resync-after-paste.patch).
Base: `zed@d89e9c2`. File: `crates/gpui_web/src/events.rs` (+9), two call
sites in `register_paste`.

`git apply --check`: OK on `/tmp/zedapply` and on `zed@d89e9c2`.
`cargo +nightly check -p gpui-pre-web --target wasm32-unknown-unknown`
passes. `rustfmt --check` is clean.

Still needed at d89e9c2: yes. `register_paste` (0.3.5 `src/events.rs:1041`)
calls `event.prevent_default()` and routes the clipboard through
`handler.paste(..)` on both its paths, but calls neither
`schedule_ime_mirror_sync()` nor `ImeMirror::schedule_sync`. Every other
prevented edit in the file resyncs: keydown (`:742`, `:764`), pointer up
(`:368`, `:391`), rejected selection import (`:987`), `beforeinput` line
breaks (`:1019`), composition end (`:1158`).

Re-verified against 0.3.5's rewritten `ime_mirror.rs`. The retired 0.3.3
note predates it. Two things are new:

- The mirror is now a *window* of the document around the selection
  (`CONTEXT_CHARS = 512`), and `sync` skips every write while the element
  still matches the document at its stored alignment. After a prevented
  paste, the document has moved and the element has not, so the next import
  is misaligned. The fix is still a scheduled sync; `sync` itself decides
  whether a write is needed.
- A keyboard paste (Ctrl/Cmd+V left unbound by the app) is already covered by
  accident. `register_key_down` schedules a zero-delay sync (`:764`), and the
  browser dispatches `paste` in the same task, so that sync runs after the
  paste. Three paths are *not* covered. These are the browser's own context
  menu "Paste", the mobile long-press "Paste" bubble, and the Edit menu,
  where no keydown precedes the `paste` event. The image and mixed paste is
  not covered either: it resolves in `spawn_local` after any scheduled sync
  has already run. This is why desktop Ctrl+V testing often does not
  reproduce the bug.

---

**PR title:** `gpui_web: Resync the IME mirror after paste`

**PR body:**

`register_paste` cancels the browser's default action and hands the
clipboard to the focused input handler. The browser therefore never applies
the edit to the hidden IME `<textarea>`, and the mirror still holds the
pre-paste window of text while the application's buffer has moved on. The
next `input`, `beforeinput` or composition event is diffed against that stale
mirror, and the resulting replacement is applied at offsets that no longer
match the buffer. The visible result is duplicated pasted text, or the next
keystroke or suggestion replacing the wrong characters. It is easiest to hit
on mobile, where the IME edits the mirror directly.

Every other prevented edit in `events.rs` already calls
`schedule_ime_mirror_sync()`. The two paste paths were missed:

1. the text-only path, which completes synchronously inside the handler;
2. the image/mixed path, which resolves inside `wasm_bindgen_futures::spawn_local`
   a frame or more later. A sync scheduled by the triggering event has already
   run against the pre-paste document by then, so it needs its own call after
   `handler.paste(..)`.

`schedule_ime_mirror_sync` coalesces to one deferred write per event-loop
turn, and `ImeMirror::sync` skips the write entirely when the element still
matches the document. The extra calls cannot restart the IME mid-gesture, and
they cost nothing when nothing changed.

Why keyboard pastes mostly worked already: an unbound Ctrl/Cmd+V keydown
schedules a zero-delay sync, and the browser fires `paste` in the same task,
so the sync happens to run afterwards. Pastes with no preceding keydown were
broken. These are the browser context menu's Paste, the mobile long-press
Paste bubble, and the Edit menu. Image and mixed pastes were broken on every
input path.

Repro (before this change): in a GPUI web app with a text input, type
`hello `, then right-click and choose the browser's Paste with `world` on the
clipboard. Then type a character through an IME, or on Android with Gboard
type and pick a suggestion. The edit lands at the pre-paste offset or
duplicates `world`. After this change the mirror is refreshed on the next
turn and the edit lands at the caret.

No API change and no new dependency. Native targets are not affected.

Motivation: HeroGPUI, a GPUI component library whose browser gallery runs on
`gpui_web`, carried this fix in a local fork of the crate. It currently
documents the stale-mirror-after-paste behavior as a known web limitation.

Release Notes:

- N/A

---

HeroGPUI follow-up once released: drop the "IME-mirror resync after paste
still waits upstream" limitation from `CLAUDE.md`/`AGENTS.md` and from
`docs/upstream/gpui-web-scroll-and-ime.md`.
