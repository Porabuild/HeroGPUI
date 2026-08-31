import { readFileSync } from "node:fs";
import { resolve } from "node:path";

/**
 * Serves the repository's `llms.txt` (the HeroGPUI agent-facing API
 * reference) as plain text. The Next app lives in `web/`, one level below the
 * repository root that owns the file.
 *
 * The file is read once at module scope and the route is exported as
 * `force-static`: `next build` executes this module on the build machine —
 * where the sibling Rust checkout is present — and stores the prerendered
 * response body. The deployed function therefore serves static bytes and
 * never needs `../llms.txt` at request time, which matters because the
 * Vercel deployment ships only the `web/` build output.
 */
const llmsTxt = readFileSync(resolve(process.cwd(), "..", "llms.txt"), "utf8");

export const dynamic = "force-static";

export function GET(): Response {
  return new Response(llmsTxt, {
    headers: {
      "Content-Type": "text/plain; charset=utf-8",
    },
  });
}
