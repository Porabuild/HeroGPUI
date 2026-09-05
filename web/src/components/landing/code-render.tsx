import { CodeBlock } from "@/components/ui/code-block";
import { SectionHeading } from "@/components/landing/shared";
import { GalleryFrame } from "@/components/preview/gallery-frame";

/** The README's Button example, verbatim. */
const BUTTON_EXAMPLE = `use gpui::prelude::*;
use herogpui::prelude::*;

impl Render for MyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        app_focus_root(div()
            .size_full()
            .bg(cx.colors().background)          // semantic token
            .font_family("Segoe UI")
            .child(
                Button::new("save")
                    .label("Save changes")
                    .variant(Variant::Primary)
                    .on_press(cx.listener(|this, _, _, cx| this.save(cx))),
            ), window, cx)
    }
}`;

/**
 * The code you write beside the pixels it produces: a real Rust example and
 * the live GPUI WebAssembly runtime rendering the Button component.
 */
export function CodeAndRender() {
  return (
    <section className="landing-code-render border-b border-separator">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-16 sm:px-6 md:py-24">
        <SectionHeading
          eyebrow="Rust code, native output"
          sub="This is a real Rust builder. Beside it is the Button component compiled to WebAssembly and rendered live by GPUI in your browser."
          title="A builder beside the result"
        />

        {/* The README example's longest line is 91 chars — too wide for a
            half-measure column, so the two-up row only starts at xl, where a
            2:1 split fits every line without an inner scrollbar. Below xl the
            code block spans the full measure instead. */}
        <div className="mt-10 grid items-center gap-10 xl:grid-cols-[minmax(0,2fr)_minmax(0,1fr)] xl:gap-14">
          <CodeBlock code={BUTTON_EXAMPLE} filename="main.rs" lang="rust" />
          <GalleryFrame
            bare={false}
            className="mx-auto w-full max-w-2xl xl:max-w-none"
            section="Variants"
            slug="button"
            title="Button"
          />
        </div>
      </div>
    </section>
  );
}
