# Button Usage — first native-surface (macOS) capture set

Captured 2026-09-13 (local; `2026-09-14T05:19Z`). This is the first `native`
surface capture produced by the macOS driver
([`.shots/native_capture.py`](../../../.shots/native_capture.py)) — framing
evidence for the `btn-usage` specimen against the **native macOS gallery
binary**, not a completed Button parity verdict and not an upstream
comparison; the matched upstream/WASM set for the same specimen is
[matched-button-usage](../matched-button-usage/README.md). No interaction was
driven on either side.

## Target

Port: `herogpui-gallery` (native macOS binary, `debug` profile), sha256
`18874c56a9fb634e9ea674f3feb28d2e6f59dc82b114c986329c5b365695cb61`, at
commit `7b192268`, captured from the driver's own worktree (dirty — the
driver files themselves were uncommitted at capture time; recorded in each
provenance JSON).

## Capture setup

- Host: macOS 26.6.2, arm64, Retina (DPR 2.0). One gallery process served
  both themes through the `HEROGPUI_CONTROL` file protocol; each theme was a
  separate control request whose case-sensitive `seq` acknowledgement was
  awaited before its capture, so every image proves a rendered frame of that
  request. The exact control transcript is in each JSON
  (`seq=1`/`seq=2`, `page=Button`, `section=Usage`, `specimen=btn-usage`,
  `theme=light|dark`, `preview=component`).
- Window: launched unfocused at a requested 1200x800; the window settled at
  a **1200x832 pt** frame (`kCGWindowBounds`, recorded in the JSONs) — the
  driver reports the window's own bounds rather than assuming the request,
  per the gallery guide.
- Capture: `screencapture -l<window-id> -x -o` (window backing store, no
  shadow, no sound), the window found by owner process id over CGWindowList.
  Screen Recording permission was granted to the host app, so the
  computer-use fallback described in the driver doc was not needed.
- Images are **2400x1664 px @ DPR 2.0**; logical size 1200x832 px. Logical
  values below are PNG pixels divided by 2. Unlike the WASM set's 640x360
  canvas crop, the frame is the whole window including the macOS titlebar;
  the specimen itself is centered by the same `preview_wrapper` (with its
  dot grid) the browser preview uses.

## Images

| File | What it shows |
|---|---|
| [native-usage-light.png](native-usage-light.png) / [native-usage-dark.png](native-usage-dark.png) | Native gallery window, `btn-usage` specimen in component preview, light / dark; provenance in [native-usage-light.json](native-usage-light.json) / [native-usage-dark.json](native-usage-dark.json) |

## Measured geometry (logical px)

| Metric | Both themes — method: raw pixel scan of the 2x capture for the primary-blue fill, bbox divided by 2 |
|---|---|
| Button box | 87 x 36, center (599.75, 431.75) |
| Window | 1200 x 832; button centered in the content area below the 32 pt titlebar |
| Label | "Click me" |

The box is identical in both themes, and the 36 px height matches the
desktop value the WASM set measured (upstream 89.08 x 36, port 90 x 36 by
PIL bbox scan). The 87 px width is not comparable to those at sub-pixel
level: this scan thresholds the blue fill directly (antialiased edge columns
drop out) while the sibling set scanned against the background, and the
rasterizer differs. Geometry here is indicative, not a verdict.

## Known limitations

- **Framing only.** The control protocol's acknowledgement proves a rendered
  frame; it moves no input. Interaction evidence on macOS needs the
  computer-use path documented in
  [`native_capture.md`](../../../.shots/native_capture.md) and was not used
  here.
- **Native window chrome and DPR.** The PNG includes the macOS titlebar and
  is 2x; any comparison against the DPR-1 WASM captures must crop to the
  content area and convert through the recorded DPR.
- **No upstream side.** This set pairs nothing: it records the port's native
  surface for the `native` evidence column. Token and font differences noted
  in the matched set apply to this rendering as well (bundled Inter, ported
  token set).
- The debug-profile binary is hashed in provenance; a release-profile run
  (`--release`) would produce a different hash and should be re-recorded, not
  substituted.

**Not a completed verdict:** variants, sizes, states, motion, and
interaction parity remain unproven by this set.
