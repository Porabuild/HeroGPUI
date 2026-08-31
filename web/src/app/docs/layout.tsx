import type { ReactNode } from "react";
import { Navbar } from "@/components/site/navbar";
import { SidebarMobile, SidebarRail } from "@/components/site/sidebar";
import { SiteFooter } from "@/components/site/footer";
import { Toc } from "@/components/site/toc";
import { getCatalog, getComponentSidebarGroups } from "@/lib/catalog";

/**
 * Three-column documentation shell: sidebar (rail on desktop, drawer on
 * mobile), article, and "on this page" table of contents. Server component —
 * the interactive leaves (navbar, sidebar, toc, theme toggle) are clients.
 */
export default function DocsLayout({ children }: { children: ReactNode }) {
  // Empty catalog until the data pipeline lands; sidebar then shows only the
  // hand-listed Getting Started and AI groups.
  const groups = getComponentSidebarGroups(getCatalog());

  return (
    <div className="flex min-h-dvh flex-col">
      <Navbar />
      <SidebarMobile groups={groups} />

      <div className="mx-auto flex w-full max-w-[1440px] flex-1 items-stretch">
        <SidebarRail groups={groups} />

        {/* data-docs-main marks the size container that `.docs-bleed` measures
            against, so a reference table can use the whole column while the
            prose stays at `.docs-measure`. */}
        <main className="min-w-0 flex-1 px-4 py-8 sm:px-8 lg:py-10" data-docs-main id="main">
          <div className="docs-measure mx-auto">
            <article data-docs-article>{children}</article>
          </div>
        </main>

        <Toc />
      </div>

      <SiteFooter />
    </div>
  );
}
