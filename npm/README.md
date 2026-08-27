# HeroGPUI Gallery

Launch the native HeroGPUI component gallery without cloning the Rust workspace:

```bash
npx herogpui
```

The npm package is only a zero-dependency launcher. The Rust component library
is installed from crates.io with `cargo add herogpui`.

The launcher downloads the version-matched native gallery binary from the
project's GitHub Release, verifies its SHA-256 digest, caches it per version and
platform, and starts it with inherited arguments and terminal I/O. Use
`npx herogpui --refresh` to replace the cached binary.

Published platforms are Windows x64, macOS arm64, and Linux x64/arm64. Linux
requires a Vulkan loader plus the normal Wayland or X11 runtime libraries used
by GPUI.

Apache-2.0. See `LICENSE` and `NOTICE`.
