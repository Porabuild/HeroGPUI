import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "Dark Mode",
  description: "Register and switch light and dark themes at runtime in a GPUI application.",
};

const INIT = `Application::new().with_assets(HeroGpuiAssets).run(|cx: &mut App| {
    ThemeProvider::init(cx); // registers light + dark
    // or ThemeProvider::init_with(Theme::dark(), cx);
    // ...open windows
});`;

const TOGGLE = `herogpui::theme::toggle_light_dark(cx);`;

const SWITCH = `herogpui::theme::use_theme("dark", cx);
// or
herogpui::theme::set_theme(my_custom_dark_theme, cx);`;

const READ = `if cx.is_dark_theme() {
    // your own dark-mode branch
}

// or ask the provider for the active id: "light", "dark", or a custom one
let id = herogpui::theme::ThemeProvider::get(cx).active_id().clone();`;

export default function DarkModePage() {
  return (
    <>
      <PageHeader
        title="Dark Mode"
        description="Register and switch light and dark themes at runtime in a GPUI application."
        importLine={TOGGLE}
      />

      <h2 id="registering-both-appearances">Registering both appearances</h2>
      <p>
        <code>ThemeProvider::init(cx)</code> registers the provider with <code>light</code>{" "}
        <em>and</em> <code>dark</code>, so both themes are available without extra setup. Initialize
        it before opening the first window; rendering a themed component before initialization
        panics. To boot into dark, start from the dark theme instead:
      </p>
      <div className="mt-4">
        <CodeBlock code={INIT} lang="rust" />
      </div>
      <p>
        Every theme, including your own, is registered under an id. Activating one repaints every
        open window. <Link href="/docs/getting-started/customization">Customization</Link> covers
        building one.
      </p>

      <h2 id="toggling-at-runtime">Toggling at runtime</h2>
      <p>
        <code>toggle_light_dark(cx)</code> switches between the light and dark defaults. Call it
        from a menu item, keybinding or gallery control. Every themed component re-renders on the
        next frame.
      </p>
      <Callout kind="tip" title="Try it live">
        The gallery (<code>cargo run -p herogpui-gallery</code>) has the toggle in its top bar, and{" "}
        <code>HEROGPUI_THEME=dark</code> boots it straight into dark.
      </Callout>

      <h2 id="switching-to-a-registered-theme">Switching to a registered theme</h2>
      <p>
        <code>use_theme(id, cx)</code> activates one of the registered themes by id —{" "}
        <code>&quot;light&quot;</code>, <code>&quot;dark&quot;</code>, or a custom one you have
        registered. <code>toggle_light_dark</code> uses it to select the other default theme.
      </p>

      <h2 id="setting-a-custom-theme">Setting a custom theme</h2>
      <p>
        <code>set_theme(theme, cx)</code> registers <em>and</em> activates a theme in one step — use
        it when the theme was built with <code>Theme::builder</code> and has not been registered
        yet.
      </p>
      <div className="mt-4">
        <CodeBlock code={SWITCH} lang="rust" />
      </div>

      <h2 id="reading-the-active-appearance">Reading the active appearance</h2>
      <p>
        <code>ActiveTheme</code> exposes <code>cx.is_dark_theme()</code> for a boolean. The provider
        also keeps the active id if you need the name. Both update when a switch lands.
      </p>
      <div className="mt-4">
        <CodeBlock code={READ} lang="rust" />
      </div>

      <h2 id="reduced-motion">Reduced motion</h2>
      <p>
        The same provider holds the app-level reduced-motion preference, because GPUI does not
        surface the OS <code>prefers-reduced-motion</code> setting. Seed it at startup with{" "}
        <code>HEROGPUI_REDUCE_MOTION=1</code>, or override it at any time with{" "}
        <code>theme::set_reduce_motion(bool, cx)</code> / <code>toggle_reduce_motion(cx)</code> —
        the app-level equivalent of enabling a reduced-motion setting. Every animated component
        honours it through <code>cx.reduce_motion()</code> without caller opt-in.
      </p>
    </>
  );
}
