# `block` 0.1.6 future-incompatibility warning (macOS builds)

Every macOS build of this workspace ends with:

```text
warning: the following packages contain code that will be rejected by a future version of Rust: block v0.1.6
```

It is not HeroGPUI code. Every path to it is inside the pinned GPUI family,
which cannot be changed from this workspace without a `[patch]` (the
single-dependency install contract forbids one; see `RELEASING.md`); the one
path that was a HeroGPUI dependency is gone since 0.13 (below). The warning
is deliberately **not** silenced: turning off cargo's `future-incompat-report` would hide the next such
warning too. This note records the chain and the fix to request upstream.

## What the lint is

`cargo report future-incompatibilities` names one lint, `uninhabited_static`
([rust-lang/rust#74840](https://github.com/rust-lang/rust/issues/74840)), in
`block-0.1.6/src/lib.rs:64`:

```rust
enum Class { }
extern {
    static _NSConcreteStackBlock: Class;   // a static of an uninhabited type
    ...
}
```

When the lint becomes a hard error, every crate that compiles `block` stops
building on macOS. `block` 0.1.6 is the crate's latest release
(`SSheldon/rust-block`, unmaintained), so no semver-compatible update exists.
Linux and Windows graphs do not contain it (`cargo tree -i block --target
x86_64-unknown-linux-gnu` and `--target x86_64-pc-windows-msvc` print nothing).

## The chain

From `cargo tree -i block --target aarch64-apple-darwin -e normal,build
--locked` at `gpui-pre` `=0.3.5` (`zed@d89e9c2`). No HeroGPUI manifest names
`block`; every row is inside the GPUI family:

| Direct dependent of `block` | Reached through |
|---|---|
| `gpui-pre-macos` 0.3.5 (own `block = "0.1"`, macOS only) | `gpui-pre-platform` -> `herogpui` |
| `gpui-pre-apple` 0.3.5 (own `block = "0.1"`, macOS only) | `gpui-pre-macos` |
| `metal` 0.33.0 | `gpui-pre-apple`, `gpui-pre-macos`, `core-video` |
| `cocoa` 0.26.0, `cocoa-foundation` 0.2.0 | `gpui-pre-apple`, `gpui-pre-macos` |
| `core-video` 0.5.2 (`metal` feature), `core-graphics2` 0.5.2 | `gpui-pre`, `gpui-pre-apple`, `gpui-pre-media` |

GPUI's own direct uses are three `block::ConcreteBlock` call sites in two files:
`gpui-pre-apple-0.3.5/src/metal_renderer.rs:523` (the Metal command-buffer
completion handler) and `gpui-pre-macos-0.3.5/src/screen_capture.rs:148,191`.
`gpui-pre-macos` already depends on `block2` 0.6 and uses its `RcBlock` in
`window.rs` and `system_notifications.rs`.

## HeroGPUI's own edge (removed in 0.13)

Until 0.13, `herogpui-components` depended on `locale_config` `=0.3.0` for
the platform's date/time locale, and on macOS that crate reached `objc` 0.2
and `objc-foundation` 0.1.1, which depends on `block` -- a path to the
warning no GPUI fix could remove. `crates/herogpui-components/src/system_locale.rs`
replaced it and reads the same sources `locale_config` read for its `time`
category: the POSIX environment first on every platform (`LC_ALL`, then
`LC_TIME`, then `LANG`, with `LANGUAGE` fallbacks), and only when it names no
locale `NSLocale.currentLocale` through `objc2-foundation` 0.3 on macOS or
`HKCU\Control Panel\International\LocaleName` through `windows-registry`
on Windows, with `sys-locale`'s preferred languages as fallbacks. All three
crates were already in the GPUI graph, none depends on `block`, and
`cargo tree -i objc-foundation --target aarch64-apple-darwin` now prints
nothing. `sys-locale` alone was not a replacement: it reports the interface
languages (`LC_MESSAGES`, `CFLocaleCopyPreferredLanguages`,
`GetUserPreferredUILanguages`), not the regional format that decides date
order, first weekday and hour cycle. The remaining paths are all GPUI's.

## The upstream fix to request

Against `zed-industries/zed` (`crates/gpui_macos`, `crates/gpui_apple`), in
two steps, because removing GPUI's direct edge alone leaves `block` in the
graph through `metal`, `cocoa` and `core-video`:

1. Replace the three remaining `block::ConcreteBlock` sites with
   `block2::RcBlock` (the pattern `window.rs` already uses) and drop the
   `block` dependency from both crates.
2. Finish moving the macOS renderer and platform layer off the
   `metal`/`cocoa`/`objc` 0.2 stack onto the `objc2` family
   (`objc2-metal`, `objc2-app-kit`, `objc2-foundation`, `block2`), and take a
   `core-video` build without its `metal` feature, so no crate in the macOS
   graph depends on `block` any more.

Once a `gpui-pre` release carries that, bump the exact `=0.3.5` pin (see `RELEASING.md`), confirm
`cargo tree -i block --target aarch64-apple-darwin` prints nothing, and delete
this note.
