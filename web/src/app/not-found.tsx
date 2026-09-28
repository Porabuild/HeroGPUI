import { Link } from "@heroui/react";
import { Navbar } from "@/components/site/navbar";
import { SiteFooter } from "@/components/site/footer";
import { siteSearchItems } from "@/lib/search-api";

export default function NotFound() {
  const searchItems = siteSearchItems();

  return (
    <div className="flex min-h-dvh flex-col">
      <Navbar searchItems={searchItems} />
      <main
        className="mx-auto flex w-full max-w-xl flex-1 flex-col justify-center px-4 py-16"
        id="main"
      >
        <p className="font-mono text-xs tracking-wide text-muted uppercase">404</p>
        <h1 className="mt-3 text-3xl font-semibold tracking-tight">This page is not here</h1>
        <p className="mt-3 text-base leading-relaxed text-muted">
          The path may have moved, or the name is different. Start from a working page:
        </p>
        <ul className="mt-6 flex flex-col gap-2 text-sm">
          <li>
            <Link href="/docs/getting-started/quick-start">Quick Start</Link>
          </li>
          <li>
            <Link href="/docs/components">Component catalog</Link>
          </li>
          <li>
            <Link href="/">Home</Link>
          </li>
        </ul>
      </main>
      <SiteFooter />
    </div>
  );
}
