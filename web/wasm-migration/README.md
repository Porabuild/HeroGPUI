# WASM migration source

`web/public/gallery/herogpui_web{.js,_bg.wasm}` is compiled from a *different*
checkout than this one: an older, wasm32-compatible tree of this same
repository. Until now that tree lived only on one machine
(`D:\herogpui-wasm`), so the committed artifact recorded a result whose source
was not reviewable or reproducible from this repository.

This directory closes that gap. It does not vendor a second crates tree; it
vendors the exact recipe:

- `source.json` — the baseline commit of this repository the migration starts
  from, plus SHA256 pins for the patch, the compiled `.wasm` and the
  `wasm-bindgen` glue.
- `wasm-migration.patch` — the migration checkout's tracked changes and new
  crate/gallery sources against that baseline, including bundled OFL fonts.
  This includes the pinned toolchain, `.cargo` flags, and wasm32 adaptations.

## Reproducing the artifact

```powershell
git clone <this repo> D:\herogpui-wasm
cd D:\herogpui-wasm
git checkout 39eafe365b8546762fd7458cc051bb5ea9ffd3ee
git apply <repo>\web\wasm-migration\wasm-migration.patch

$env:CARGO_TARGET_DIR = 'D:/herogpui-wasm-target'
$env:CARGO_HOME = 'D:/cargo-home'
cargo build --target wasm32-unknown-unknown --profile wasm-release -p herogpui-web
D:\cargo-home\bin\wasm-bindgen.exe `
  D:\herogpui-wasm-target\wasm32-unknown-unknown\wasm-release\herogpui_web.wasm `
  --out-dir <repo>\web\public\gallery --target web --no-typescript
```

`wasm-bindgen` output is not bit-reproducible across toolchain versions, so
treat the `source.json` artifact pins as *what this patch produced here*, not
as a build-determinism claim. The binding that matters is enforced elsewhere:
`web/src/data/wasm-parity.json` pins the shipped artifact and glue by hash and
fails generation when native and WASM examples drift.

## Keeping it current

Refresh the patch whenever the migration source changes, in the same commit as
the artifact it produced:

```powershell
cd <repo>\web
pnpm run wasm:vendor
pnpm run wasm:vendor:check
```

The vendor command updates `source.json` with the patch and artifact hashes.
It uses a temporary index so staged and untracked build sources are included
without changing the migration checkout's staging area.
