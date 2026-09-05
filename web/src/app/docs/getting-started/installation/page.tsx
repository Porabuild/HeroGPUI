import type { Metadata } from "next";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "Installation",
  description:
    "Add HeroGPUI to a Rust desktop app, open a themed window, and run the component gallery.",
};

const CARGO_TOML = `[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "ee3b5558c581429633937e458fad8d109f29e9ee", features = ["font-kit", "wayland", "x11", "runtime_shaders"] }
herogpui = { path = "../HeroGPUI/crates/herogpui" }`;

const MAIN_RS = `use gpui::*;
use herogpui::prelude::*;

struct MyRoot;

impl Render for MyRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        app_focus_root(
            div().size_full().bg(cx.colors().background).text_color(cx.colors().foreground),
            window,
            cx,
        )
    }
}

fn main() {
    gpui_platform::application().with_assets(HeroGpuiAssets).run(|cx: &mut App| {
        ThemeProvider::init(cx); // light + dark
        // or ThemeProvider::init_with(Theme::dark(), cx);
        let bounds = Bounds::centered(None, size(px(1280.), px(820.)), cx);
        cx.open_window(WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            |_, cx| cx.new(|_| MyRoot)).unwrap();
    });
}`;

const ROOT_TOKENS = `div().bg(cx.colors().background).text_color(cx.colors().foreground).font_family("Segoe UI")`;

const GALLERY = `cargo build                     # builds library + gallery
cargo run -p herogpui-gallery   # open the component gallery`;

const GALLERY_ENV = `HEROGPUI_PAGE="Button" HEROGPUI_THEME=dark cargo run -p herogpui-gallery`;

export default function InstallationPage() {
  return (
    <>
      <PageHeader
        title="Installation"
        description="Add HeroGPUI to a Rust desktop app, open a themed window, and run the component gallery."
      />

      <h2 id="prerequisites">Prerequisites</h2>
      <ul>
        <li>
          A Rust toolchain that supports <strong>Rust 1.98</strong>.
        </li>
        <li>
          <strong>GPUI from the pinned Zed revision below</strong>.
        </li>
        <li>
          Platform tooling: Xcode on macOS; Wayland/X11 dev packages on Linux; nothing extra on
          Windows.
        </li>
      </ul>

      <h2 id="add-the-dependency">Add the dependency</h2>
      <p>
        Clone this repository and add the source dependency with the matching GPUI revision to{" "}
        <code>Cargo.toml</code>:
      </p>
      <div className="mt-4">
        <CodeBlock code={CARGO_TOML} lang="toml" filename="Cargo.toml" />
      </div>

      <h2 id="your-first-window">Your first window</h2>
      <p>
        Register the embedded assets and theme provider, open a window, and wrap its root in{" "}
        <code>app_focus_root</code>:
      </p>
      <div className="mt-4">
        <CodeBlock code={MAIN_RS} lang="rust" filename="main.rs" />
      </div>

      <h3 id="what-the-pieces-are">What the pieces are</h3>
      <p>
        <strong>
          <code>HeroGpuiAssets</code>
        </strong>{" "}
        embeds the <code>herogpui/icons/*.svg</code> files used by built-in component chrome —
        checkmarks, chevrons, the clear button. Register it with{" "}
        <code>gpui_platform::application().with_assets(HeroGpuiAssets)</code>. Apps that bring their
        own assets use <code>HeroGpuiAssets::with_fallback(MyAppAssets)</code> so both are served.
      </p>
      <p>
        <strong>
          <code>ThemeProvider::init(cx)</code>
        </strong>{" "}
        registers the provider with both the <code>light</code> and <code>dark</code> themes, and
        must run before the first window opens — rendering a themed component before initialization
        panics. Start dark with <code>ThemeProvider::init_with(Theme::dark(), cx)</code>.
      </p>
      <p>
        <strong>
          <code>app_focus_root(root, window, cx)</code>
        </strong>{" "}
        wraps the window&apos;s root element to enable Tab traversal, focus-visible state and root
        capture handlers. Without it, focus-visible rings and keyboard movement across components do
        not work.
      </p>
      <p>
        Finally, set the root background and text color from the tokens, and your font family — GPUI
        does not inherit one for you:
      </p>
      <div className="mt-4">
        <CodeBlock code={ROOT_TOKENS} lang="rust" />
      </div>

      <h2 id="run-the-gallery">Run the gallery</h2>
      <p>
        The gallery is the library&apos;s desktop documentation: one page per component, 15
        categories, runnable examples and a theme switcher.
      </p>
      <div className="mt-4">
        <CodeBlock code={GALLERY} lang="bash" />
      </div>
      <p>
        It opens as a 1690×700 window. Two environment variables pick the page and the appearance,
        which is also how the screenshots that gate the visual regressions are captured:
      </p>
      <div className="mt-4">
        <CodeBlock code={GALLERY_ENV} lang="bash" />
      </div>
      <Callout kind="tip" title="Installed launcher">
        Install the gallery as a CLI with <code>cargo install --path gallery --locked</code> when
        you want to run the documentation app outside the workspace.
      </Callout>
    </>
  );
}
