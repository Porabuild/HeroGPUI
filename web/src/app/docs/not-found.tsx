import { Link } from "@heroui/react";

export default function DocsNotFound() {
  return (
    <>
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
    </>
  );
}
