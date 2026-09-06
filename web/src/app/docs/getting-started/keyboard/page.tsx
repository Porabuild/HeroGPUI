import type { Metadata } from "next";
import { Link } from "@heroui/react";
import { Callout } from "@/components/ui/callout";
import { CodeBlock } from "@/components/ui/code-block";
import { PageHeader } from "@/components/ui/page-header";

export const metadata: Metadata = {
  title: "Keyboard and focus",
  description: "Turn on window focus, Tab order, and the keys overlays and fields already handle.",
};

const ROOT = `app_focus_root(
    div()
        .size_full()
        .bg(cx.colors().background)
        .text_color(cx.colors().foreground)
        .child(/* your tree */),
    window,
    cx,
)`;

export default function KeyboardPage() {
  return (
    <>
      <PageHeader
        title="Keyboard and focus"
        description="HeroGPUI components participate in GPUI focus. The window root has to opt in, then Tab, Escape, and the field keys work as documented on each page."
      />

      <h2 id="wrap-the-window-root">Wrap the window root</h2>
      <p>
        Call <code>app_focus_root</code> around the element you return from <code>Render</code>.
        Without it, focus-visible rings and Tab movement across components do not run:
      </p>
      <div className="mt-4">
        <CodeBlock code={ROOT} lang="rust" />
      </div>

      <h2 id="what-you-get">What you get</h2>
      <ul>
        <li>
          <strong>Tab and Shift+Tab</strong> move between focusable controls in tree order.
        </li>
        <li>
          <strong>Focus-visible rings</strong> appear for keyboard focus, not every pointer click.
        </li>
        <li>
          <strong>Enter and Space</strong> press a focused{" "}
          <Link href="/docs/components/button">Button</Link>.
        </li>
        <li>
          <strong>Escape</strong> dismisses an open <Link href="/docs/components/modal">Modal</Link>
          , <Link href="/docs/components/drawer">Drawer</Link>,{" "}
          <Link href="/docs/components/popover">Popover</Link>, or{" "}
          <Link href="/docs/components/tooltip">Tooltip</Link>.
        </li>
        <li>
          <strong>Arrow keys</strong> move inside lists, menus, tabs, sliders, and segmented date
          fields. Each component page lists the exact keys under States.
        </li>
      </ul>

      <h2 id="disabled-and-pending">Disabled and pending</h2>
      <p>
        <code>is_disabled(true)</code> removes the control from the tab order and ignores presses.{" "}
        <code>is_pending(true)</code> blocks presses but keeps the focus stop, so a submit button
        can stay on the keyboard while work finishes.
      </p>

      <Callout kind="note" title="Your window, your shortcuts">
        Application shortcuts — save, new file, command palettes — belong on the window or view you
        own. Components do not steal global keys beyond the interactions they document.
      </Callout>
    </>
  );
}
