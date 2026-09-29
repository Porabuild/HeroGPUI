import type { Metadata } from "next";
import { Navbar } from "@/components/site/navbar";
import { SiteFooter } from "@/components/site/footer";
import { SITE_URL } from "@/lib/seo";
import { siteSearchItems } from "@/lib/search-api";
import { Atlas } from "@/components/landing/atlas";
import { CodeAndRender } from "@/components/landing/code-render";
import { Features } from "@/components/landing/features";
import { FinalCta } from "@/components/landing/final-cta";
import { ForAgents } from "@/components/landing/for-agents";
import { Hero } from "@/components/landing/hero";
import { ProofStrip } from "@/components/landing/proof-strip";

// Title, description and the social cards come from the root layout; only
// the canonical is page-specific (a layout canonical would leak into every
// page that does not set its own). Absolute, because "/" joined onto the
// basePath'd metadataBase would gain a trailing slash the sitemap lacks.
export const metadata: Metadata = { alternates: { canonical: SITE_URL } };

export default function HomePage() {
  const searchItems = siteSearchItems();
  return (
    <div className="flex min-h-dvh flex-col">
      <a className="site-skip-link" href="#main">
        Skip to content
      </a>
      <Navbar searchItems={searchItems} />
      <main className="flex-1" id="main">
        <Hero />
        <ProofStrip />
        <CodeAndRender />
        <Features />
        <Atlas />
        <ForAgents />
        <FinalCta />
      </main>
      <SiteFooter />
    </div>
  );
}
