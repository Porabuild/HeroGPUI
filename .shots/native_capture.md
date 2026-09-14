# macOS native-surface capture driver

`.shots/native_capture.py` is the macOS counterpart of the Windows capture
drivers (plan section 5.2: same reset/state/ack semantics, host-specific
input and capture). The `.ps1` drivers own two Windows-only levers — posted
`WM_*` input and `PrintWindow` — and neither exists here, so this driver
ports the half that travels (build, launch, control file, acknowledgements,
cleanup) and replaces the half that does not with its macOS equivalent
(`screencapture -l<window-id>`). It is evidence tooling for the `native`
surface in `docs/parity/interaction-inventory.json`; it changes no component
source.

```sh
python3 .shots/native_capture.py --page Button --section Usage \
    --preview component --specimen btn-usage \
    --theme light --theme dark \
    --out-template 'docs/parity/evidence/native-button-usage/native-usage-{theme}.png'
```

One process serves every `--theme` step, like `batch.ps1`: the first request
is seeded before startup (`seq=0` plus `--page`), each theme is a control
request that must be acknowledged before its capture, and the child is
stopped and its control/result/log files removed before exit whether the run
passed or failed. `--skip-build` launches the existing image; without it the
driver runs `cargo build -p herogpui-gallery` and lets Cargo decide what is
stale. The binary is resolved from `gallery/Cargo.toml`'s `[[bin]]` table and
`cargo metadata`'s target directory — never an assumed path, so a
`CARGO_TARGET_DIR` build cannot be missed.

## Prerequisites

- macOS, python 3.11+ (`tomllib`), and a Rust toolchain for the build. No
  third-party python packages: CGWindowList is reached through `ctypes`, and
  the captured PNG is verified by a stdlib decoder.
- **Screen Recording permission** for the process that runs the script — in
  practice the host terminal or app, since the permission belongs to the
  calling process's TCC identity. The denial is silent: `screencapture`
  exits 0 and writes a uniform frame or the desktop wallpaper instead of the
  window. The driver therefore refuses to record a capture whose decoded
  pixels are uniform, or whose aspect ratio does not match the window's
  CGWindow bounds, and fails with that explanation.
- When the permission cannot be granted, the fallback is the computer-use
  tool chain, which captures through its own granted permission: load the
  computer-use skill, `list_windows` to find the gallery window (title
  `HeroGPUI — Gallery`; the driver prints the child pid to match against),
  then `get_window_state` with `include_screenshot`. That path cannot be
  scripted inside `native_capture.py` and does not exercise the control
  protocol's capture half, so its output must be filed by hand with the same
  provenance fields (window id, bounds, DPR, binary sha256, control
  transcript) and the capture method named in the evidence README. The pilot
  set in `docs/parity/evidence/native-button-usage/` did not need it:
  `screencapture -l` worked.

## Semantics mapping

| Windows driver | native_capture.py |
|---|---|
| `rebuild.ps1` moves a locked image aside; Windows keeps it open after a capture | Unix unlinks freely; `cargo build -p herogpui-gallery` is run in-process and Cargo is the staleness oracle |
| `control.ps1` `Write-GalleryControl`: sibling temp file, UTF-8 no BOM, results retired, replacing move | `write_control`: identical order of operations on `tempfile` + `os.replace` |
| `control.ps1` `Wait-GalleryControl`: case-sensitive `seq` in `.ack`, matching `seq=`/`error` record surfaced from `.error`, dead-process and timeout failures | `wait_ack`: same comparisons, same defaults (3600 ms, 50 ms poll) |
| `batch.ps1`: one process, one seq per step, env overrides on the child only | one process, one seq per theme step; every `HEROGPUI_*` variable is stripped from the child's environment before `HEROGPUI_CONTROL`/`HEROGPUI_UNFOCUSED`/`HEROGPUI_WINDOW_SIZE` are set |
| `drive.ps1`/`batch.ps1` post `WM_*` messages for clicks, keys, text, wheel | **not ported** — see "Input" below |
| `PrintWindow(hwnd, hdc, 2)` reads the window's own bitmap, off-screen | `screencapture -l<id> -x -o` reads the window's backing store (`-o` drops the shadow, so the PNG is exactly the window) |
| window parked off-screen at −32000, never minimized | macOS has no off-screen parking here: with `--window-size` the gallery creates the window at (0, 0); unfocused, it still renders. Minimizing would stop rendering — the same rule as Windows |
| window found via `Process.MainWindowHandle` | window found via CGWindowList by **owner pid**, never by title (titles are redacted without permission, and another app's window can carry the same words) |
| `capture2.ps1` real foreground input | not ported — see "Input" below |

The gallery already renders the requested page before the first ack: the
`seq` acknowledgement is scheduled from the requested render and delivered
on a later frame, identical to the Windows protocol. An acknowledgement
proves one rendered frame — not asset completion, settled motion, or the
success of any later input — exactly as `docs/agents/gallery.md` states for
the PowerShell drivers.

## Provenance

Every capture writes a JSON beside the PNG (`native-usage-light.png` →
`native-usage-light.json`): capture timestamp, host and git state, the
binary's path and sha256 with its Cargo profile, the CGWindow id, owner pid
and title, the window's logical bounds (points), the PNG's pixel size, the
device pixel ratio (`png px / bounds pt`; 2.0 on a Retina display), the
logical-size conversion, and the exact control lines that produced the
frame. Comparisons against the WASM evidence sets (captured at DPR 1) must
go through that conversion.

Captures are verified before they are recorded: non-uniform pixels, PNG
dimensions consistent with the window bounds, and a rendered-frame ack for
the exact `seq`. Still open the image and confirm the requested page,
section, theme and state — a correctly rendered wrong page is a failed
capture, here as in the Windows guide.

## Input

The control protocol moves no input; it changes view state. This driver
adds no input injection of its own: macOS has no posted-message equivalent
of `drive.ps1`'s `PostMessage` path, and synthetic events through the
accessibility or event system need their own permission grants and a
running harness. For interaction evidence on macOS:

1. Launch and position the gallery (this driver, left running via a
   foreground terminal, or any launch with the wanted env).
2. Drive real input through the computer-use tool chain (its skill
   documents `click`, `type` and `press_key`), which moves the real cursor
   and can interrupt the user — the same intrusiveness contract as
   `capture2.ps1`, so it is opt-in, never a default.
3. Assert on the resulting rendered state, and record in the evidence
   README that interaction came from that path.

A control-file ack combined with a computer-use screenshot of an
un-interacted window is framing evidence, not an interaction proof.

## Tests

`python3 .shots/test_native_capture.py` covers the protocol port
(publication atomicity, result retirement, ack and error matching including
case sensitivity, dead-process reporting), binary resolution from the
manifest, the CGWindow choice filter, and the PNG verifier (filters,
uniformity, truncated input). Nothing there launches the gallery, Cargo, or
any GUI, so CI runs it on every platform; the driver itself remains
macOS-only and says so loudly when launched elsewhere.
