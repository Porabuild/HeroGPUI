import type { Metadata } from "next";
import Link from "next/link";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { Callout } from "@/components/ui/callout";

export const metadata: Metadata = {
  title: "Animation",
  description:
    "Where the motion lives, how reduced motion is honoured, and what GPUI's missing transforms cost.",
};

const REDUCE = `// Anywhere you have an App context.
herogpui::theme::set_reduce_motion(true, cx);
herogpui::theme::toggle_reduce_motion(cx);

// Read it if your own view animates.
if herogpui::theme::ActiveTheme::reduce_motion(cx) {
    // draw the end state directly
}`;

const ENV = `HEROGPUI_REDUCE_MOTION=1 cargo run -p herogpui-gallery`;

export default function AnimationPage() {
  return (
    <>
      <PageHeader
        title="Animation"
        description="Where the motion lives, how reduced motion is honoured, and what GPUI's missing transforms cost."
        importLine={"herogpui::theme::set_reduce_motion(true, cx);"}
      />

      <p>
        v3 drives motion from data attributes and CSS. There is no CSS here, so the timings live in
        one module — <code>anim.rs</code> — and every component reads them from there. You do not
        wire animation up per component; composing a modal gets you v3&apos;s modal motion.
      </p>

      <h2 id="timings">The timings are v3&apos;s, per overlay</h2>
      <p>
        Each overlay declares its own curve and duration upstream, and the port evaluates those
        cubic-béziers exactly rather than substituting a built-in easing. A panel enters over 250ms
        from <strong>1.05</strong> — a modal settles <em>down</em> onto the page rather than growing
        into it — while a popover enters over 150ms from 0.90, a list from 0.95, and the backdrop
        fades alone over 150ms. Exits are 100ms.
      </p>
      <p>
        A <code>RenderOnce</code> component leaves the tree the moment its open flag goes false, so
        an overlay is held for the exit duration first: the phase reports <code>Open</code>,{" "}
        <code>Exiting</code> or <code>Closed</code>, and the component picks the animation from it.
        That is why a dismissed dialog is still on screen for a frame or two.
      </p>

      <h2 id="reduced-motion">Reduced motion</h2>
      <p>
        Every animation is gated on one flag, with no opt-in from the caller — a component cannot
        forget to check it:
      </p>
      <div className="mt-4">
        <CodeBlock code={REDUCE} lang="rust" />
      </div>
      <p>
        HeroGPUI keeps its own animation preference, separate from GPUI&apos;s App setting. Seed it
        at startup:
      </p>
      <div className="mt-4">
        <CodeBlock code={ENV} lang="bash" />
      </div>
      <Callout kind="warning" title="Wire this to your own settings">
        Use HeroGPUI&apos;s setter when wiring your application settings so every component observes
        the same animation preference.
      </Callout>

      <h2 id="no-transforms">Why the press is geometric</h2>
      <p>
        The pinned GPUI plumbs its transformation matrix into SVG painting alone, so quads and text
        cannot be scaled or rotated. v3&apos;s <code>scale(0.97)</code> press and{" "}
        <code>zoom-in-90</code> overlay entry are therefore reproduced by changing geometry rather
        than by transforming: the press scales height, padding, gap, corner radius, minimum width{" "}
        <em>and type size</em>, with margins absorbing exactly what the box gives up, so the outer
        footprint never changes and pressing a control cannot nudge its neighbour.
      </p>
      <p>Two differences from a real CSS transform remain, and they are visible:</p>
      <ul>
        <li>
          A label wider than the control&apos;s minimum width narrows the control by about 3% of
          that overflow, because GPUI cannot shrink text without affecting layout.
        </li>
        <li>
          An icon child keeps its size through both the press and the overlay zoom, because its
          dimensions belong to you rather than to the component.
        </li>
      </ul>
      <p>
        These are the only two places the port knowingly diverges from v3&apos;s motion, and they
        are consequences of the framework rather than choices. Everything else — durations, curves,
        scales, and which components animate at all — is measured against v3&apos;s stylesheets by{" "}
        <Link href="/docs/ai/agents-md">the motion audit</Link> on every change.
      </p>
    </>
  );
}
