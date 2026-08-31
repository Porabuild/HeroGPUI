# Gallery and visual verification

Read this guide before changing `gallery`, capturing screenshots, driving a
component, or running route smoke. Gallery pages render lazily, so compilation
does not prove that a route draws without a GPUI panic.

## Build and smoke

After a component or gallery change, use:

```powershell
.shots/rebuild.ps1
.shots/smoke.ps1
```

Run the scripts in the current PowerShell session, not through
`powershell -File`. The rebuild script moves aside a recently used executable
before Cargo writes a new one; plain `cargo build` can fail with `Access is
denied` after a capture and leave the next screenshot using the old binary.

The gallery scripts expect
`E:\work\HeroGPUI\target\debug\herogpui-gallery.exe`. If `CARGO_TARGET_DIR`
points elsewhere, clear it for these scripts or verify the executable path
before diagnosing a stale render.

`smoke.ps1` uses one off-screen process and the `HEROGPUI_CONTROL` protocol to
walk every current route, then retries a suspected crash alone. Trust the
script's current route list and output rather than a copied page count.

When maintaining a driver, keep the GPUI window off-screen rather than
minimized; a minimized window may never render. The gallery is a
console-subsystem executable, so launch it with `CreateNoWindow = $true` by
constructing a `Process` and calling `Start()`. `Process::Start(psi)` can return
null in PowerShell here, while `Start-Process -WindowStyle Hidden` also hides the
GPUI window and leaves nothing to capture.

## Choose the least intrusive driver

Use `.shots/drive.ps1` for most interaction checks. It posts input to an
off-screen, unfocused gallery window and captures with `PrintWindow`, so it does
not take over the user's desktop.

```powershell
python .shots/sections.py Table
.shots/drive.ps1 -Page Table -Section Sorting `
  -Do "click:353,387 key:enter"
```

Supported steps include `click:X,Y`, `dblclick:X,Y`, `drag:X,Y>X,Y`,
`key:tab`, `key:down*15`, `type:hello_world`, `wheel:N`, and `wait:400`.

Use `.shots/batch.ps1` when several checks can share one process:

```powershell
.shots/batch.ps1 -Steps @(
    @{ page='Table'; section='Sorting'; do='click:353,387 key:enter' },
    @{ page='Switch'; section='Usage'; out="$env:TEMP\switch.png" }
)
```

Use `.shots/capture2.ps1` only when real foreground input is necessary, such as
modifier chords or hover/focus-sensitive states. It moves the real cursor,
raises the gallery, and can interrupt the user.

```powershell
.shots/capture2.ps1 -PageList "Button,Calendar"
.shots/capture2.ps1 -PageList Tooltip -HoverX 455 -HoverY 544
```

## Deep links and environment controls

- `HEROGPUI_PAGE` selects a route.
- `HEROGPUI_SECTION` filters the page to matching section titles. Prefer this
  over fragile wheel counts for long pages.
- `HEROGPUI_THEME=dark` selects dark appearance.
- `HEROGPUI_OPEN_OVERLAYS=1` starts overlay demos open.
- `HEROGPUI_UNFOCUSED=1` prevents the app from taking focus.
- `HEROGPUI_REDUCE_MOTION=1` substitutes for an OS preference GPUI does not
  expose.
- `HEROGPUI_CONTROL=<file>` selects page, section, theme, and overlay state in a
  running process. Wait for the matching `<file>.ack`; it is written after the
  requested frame draws.

Window sizing differs between the foreground capture path and the off-screen
driver, which creates its window at the requested size. Prefer a section deep
link for a stable subject, and trust the driver's reported/captured dimensions
rather than assuming a requested height was honored.

## Input limitations

- Posted input does not carry Windows modifier state. `shift+tab`, Ctrl/Cmd
  chords, capitals, and shifted symbols need real input through `capture2.ps1`.
- GPUI derives characters from the key event it handles; a posted `WM_CHAR`
  alone does not type.
- A click may first establish hover and be swallowed. When testing a click
  handler, send two clicks if necessary and assert on the effect rather than
  assuming the first press reached it.
- Pointer clicks focus a control without enabling `:focus-visible`. To capture a
  focus-only surface, click it and then send a key so the root records keyboard
  modality.
- Coordinates are bitmap/client coordinates used by the driver. Verify the
  target visually before concluding that a handler is broken.

PowerShell variable names are case-insensitive. In driver scripts, a local `$vk`
would overwrite a `$VK` key-code table; choose names that cannot collide.

## Screenshot integrity

Capture the window, never the monitor. `Graphics.CopyFromScreen` can save the
user's foreground app when Windows refuses a focus steal. The scripts use
`PrintWindow(hwnd, hdc, 2)` (`PW_RENDERFULLCONTENT`) because GPUI presents via
DirectComposition.

For every new or refreshed image:

1. Check the captured frame is not uniform or blank.
2. Open it and verify the requested route and section are visible.
3. Verify the intended theme, overlay, hover, focus, or post-action state.
4. Treat a correctly rendered wrong page as a failed capture.

Use `.shots/refresh.ps1` only when a broad reference refresh is requested.
Focused appearance changes should update only the relevant screenshots.

## Behavioral proof

A screenshot proves what was drawn, not whether a control responds. For state,
focus, keyboard, overlay, or drag changes:

1. Run the focused headless component test.
2. Rebuild with `.shots/rebuild.ps1`.
3. Drive the exact gallery path, preferably off-screen.
4. Assert on the resulting visible state or callback effect.

Use dark and reduced-motion/open-overlay smokes when tokens, overlays, or motion
change. Do not claim those variants from a default route smoke.
