import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";
import { SITE } from "@/lib/nav";

export const metadata: Metadata = {
  title: "Quick Start",
  description: "Create a Rust desktop window and render a HeroGPUI button in a few minutes.",
};

const CREATE = `cargo new hello-herogpui --bin
cd hello-herogpui`;

const CARGO_TOML = `[dependencies]
herogpui = { git = "https://github.com/Porabuild/HeroGPUI" }`;

const MAIN_RS = `use herogpui::*;

struct MyRoot;

impl Render for MyRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        app_focus_root(
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(cx.colors().background)
                .text_color(cx.colors().foreground)
                .child(Button::new("hello").label("Click me")),
            window,
            cx,
        )
    }
}

fn main() {
    application()
        .with_assets(HeroGpuiAssets)
        .run(|cx: &mut App| {
            herogpui::init(cx);
            let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|_| MyRoot),
            )
            .unwrap();
        });
}`;

const RUN = `cargo run`;

export default function QuickStartPage() {
  return (
    <>
      <PageHeader
        title="Quick Start"
        description="Create a Rust binary, add HeroGPUI, and put a button on a themed window."
      />

      <p>
        You need a Rust toolchain that supports <strong>Rust 1.98</strong>. GPUI itself is not a
        dependency you add: <code>herogpui</code> depends on the published <code>gpui-pre</code>{" "}
        crates (<code>{SITE.gpuiVersion}</code>) and re-exports them, so{" "}
        <code>use herogpui::*;</code> <em>is</em> GPUI. The unrelated crates.io <code>gpui</code>{" "}
        0.2.2 crate will not compile against this library.
      </p>

      <h2 id="create-the-app">1. Create the app</h2>
      <div className="mt-4">
        <CodeBlock code={CREATE} lang="bash" />
      </div>

      <h2 id="add-the-crates">2. Add the crate</h2>
      <p>
        One dependency, and only one. Paste this into <code>Cargo.toml</code>. If you cloned
        HeroGPUI next to the app, you can swap the <code>herogpui</code> line for{" "}
        <code>{`herogpui = { path = "../HeroGPUI/crates/herogpui" }`}</code>.
      </p>
      <div className="mt-4">
        <CodeBlock code={CARGO_TOML} filename="Cargo.toml" lang="toml" />
      </div>

      <h2 id="open-a-window">3. Open a window</h2>
      <p>
        Replace <code>src/main.rs</code>. <code>herogpui::init</code> must run before the first
        window, and <code>app_focus_root</code> turns on Tab and focus rings.
      </p>
      <div className="mt-4">
        <CodeBlock code={MAIN_RS} filename="src/main.rs" lang="rust" />
      </div>

      <h2 id="run-it">4. Run it</h2>
      <div className="mt-4">
        <CodeBlock code={RUN} lang="bash" />
      </div>
      <p>
        You should get a small window with a primary button. From here, open the{" "}
        <Link href="/docs/components/button">Button</Link> page or the{" "}
        <Link href="/docs/getting-started/gallery">desktop gallery</Link>.
      </p>

      <Callout kind="tip" title="What the other guide is for">
        <Link href="/docs/getting-started/installation">Installation</Link> covers assets, custom
        themes, platform packages, and the gallery CLI. Use it after this window works.
      </Callout>
    </>
  );
}
