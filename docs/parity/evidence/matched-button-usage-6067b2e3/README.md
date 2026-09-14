# Button Usage — freeze recapture (artifact `6067b2e3`)

Resting-state framing evidence for `btn-usage` against HEAD `6811eac0` and
the checked-in WASM artifact
`6067b2e3e8810b32b8fc5af431b4f5a2e1e40825b2bee6aed7fa31fe19e7cfce`.
This is **not** a completed Button parity verdict: no interaction was driven.

## What was recaptured

- **Port (WASM):** `port-light.png` / `port-dark.png` at 640×360, DPR 1,
  after `#loading` was gone. Chrome `--headless=old` + WebGL + CDP
  `Emulation.setDeviceMetricsOverride` 640×360. Provenance JSON beside each
  PNG. The PNG bytes are identical to the earlier
  [`matched-button-usage`](../matched-button-usage/) port captures (artifact
  `e2bfda29`): Button Usage pixels did not change when the freeze artifact
  advanced.
- **Upstream:** copied unchanged from `matched-button-usage/`. Fixture
  registry, `@heroui/react@3.2.5` and `pnpm-lock.yaml` (`96fe7abea3effa8a`)
  are byte-unchanged since that capture.

## Named deviations (do not treat as geometry failures)

- Fixture fonts are Geist; the port bundles Inter. The 0.92px width gap
  (89.08×36 upstream vs 90×36 port) is label advance width.
- Fixture tokens are the website porabuild accent (`#8B7BFF`); the port
  uses HeroUI primary (`#0485F7`). Color diffs are expected until a
  same-token fixture run exists.

## Native surface

HEAD recapture of the macOS gallery did not land in this directory:

- `native_capture.py` finds the window but never receives a rendered-frame
  acknowledgement when launched unfocused from this agent host (seq=1
  times out; the control loop never writes `.ack`).
- `screencapture -l` from the agent shell cannot create an image from the
  window (TCC). Computer-use lists the gallery window but then reports it
  unavailable for capture.
- Existing [`native-button-usage`](../native-button-usage/) captures remain
  geometrically plausible: `button.rs` and `gallery/.../buttons.rs` did not
  change between `7b192268` and `6811eac0`. Their provenance still names
  `7b192268` and a dirty tree, so they are not HEAD-bound evidence.

Re-run from a Screen Recording-granted terminal:

```sh
python3 .shots/native_capture.py --page Button --section Usage \
    --preview component --specimen btn-usage \
    --theme light --theme dark \
    --ack-timeout-ms 30000 \
    --out-template 'docs/parity/evidence/native-button-usage-6811eac0/native-usage-{theme}.png'
```
