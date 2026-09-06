import { Navbar } from "@/components/site/navbar";
import { SiteFooter } from "@/components/site/footer";
import { getCatalog } from "@/lib/catalog";
import { buildSearchItems } from "@/lib/docs-nav";
import { Atlas } from "@/components/landing/atlas";
import { CodeAndRender } from "@/components/landing/code-render";
import { Features } from "@/components/landing/features";
import { FinalCta } from "@/components/landing/final-cta";
import { ForAgents } from "@/components/landing/for-agents";
import { Hero } from "@/components/landing/hero";
import { ProofStrip } from "@/components/landing/proof-strip";

export default function HomePage() {
  const searchItems = buildSearchItems(getCatalog());
  return (
    <div className="flex min-h-dvh flex-col">
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
