# HeroGPUI

HeroGPUI is a UI library for Rust desktop applications. It brings the HeroUI
design system to Rust, built on GPUI, the GPU-accelerated framework behind the
Zed editor. It runs on Windows, macOS and Linux from one codebase.

It implements every component HeroUI documents — 71 components, shown in the
gallery and on the website as 66 pages because a few pages cover a component
together with its group or slot siblings.

```rust
use gpui::prelude::*;
use herogpui::prelude::*;

impl Render for MyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        app_focus_root(div()
            .size_full()
            .bg(cx.colors().background)
            .child(
                Button::new("save")
                    .label("Save changes")
                    .variant(Variant::Primary)
                    .on_press(cx.listener(|this, _, _, cx| this.save(cx))),
            ), window, cx)
    }
}
```

## Installation

Prerequisites: Rust 1.98 and the platform tooling GPUI needs (Xcode on macOS;
Wayland/X11 dev packages on Linux; nothing extra on Windows).

The crates are not published on crates.io. Clone this repository and add the
source dependency with the matching GPUI revision to `Cargo.toml`:

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee", features = ["font-kit", "wayland", "x11", "runtime_shaders"] }
herogpui = { path = "../HeroGPUI/crates/herogpui" }
```

The same three dependencies from the command line:

```sh
ZED=https://github.com/zed-industries/zed
REV=ee3b5558c581429633937e458fad8d109f29e9ee
cargo add --git $ZED --rev $REV gpui
cargo add --git $ZED --rev $REV --features font-kit,wayland,x11,runtime_shaders gpui_platform
cargo add herogpui --path <checkout>/crates/herogpui
```

The matching GPUI API is available from the pinned Zed git revision, not its
crates.io release. `gpui` and `gpui_platform` are direct dependencies, not just
HeroGPUI's: the example above calls both. Then:

1. Register the embedded icons with
   `gpui_platform::application().with_assets(HeroGpuiAssets)`.
2. Register the theme provider with `ThemeProvider::init(cx)` before opening
   a window.
3. Wrap the root element with `app_focus_root(root, window, cx)` so Tab and
   focus-visible behavior work across components.
4. Set the background and font family on the root view.

## Gallery

A desktop gallery ships with the library and documents every component:

```bash
cargo run -p herogpui-gallery   # open the component gallery
cargo install --path gallery --locked  # install the gallery CLI from this checkout
```

`HEROGPUI_PAGE` and `HEROGPUI_THEME` select the page and appearance;
`HEROGPUI_WINDOW_SIZE` sets the window size.

## Documentation

- Website: <https://porabuild.com/herogpui> — every component page embeds
  HeroGPUI compiled to WebAssembly, running live next to its Rust code.
- `llms.txt` at the repository root: the full component API reference for
  agents. It is served verbatim at
  <https://porabuild.com/herogpui/llms.txt>.
- `AGENTS.md` and `docs/agents/`: the contributor and agent guides.

## Verification

Audit tooling under `.shots/` checks the implementation against the upstream
design system. See `docs/agents/parity.md` for what each audit reads and what
it proves. The lint gate is `.shots/lint.ps1`.

## License

Apache-2.0. This library derives component behavior, design tokens, styles,
and documentation structure from HeroUI, Copyright 2025 NextUI Inc., also
Apache-2.0. See `LICENSE` and `NOTICE`.
