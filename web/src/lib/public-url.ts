/**
 * Public-asset and internal-path prefixing for the base-path mount.
 *
 * Local dev serves at "/", production is mounted at
 * https://porabuild.com/herogpui (NEXT_PUBLIC_BASE_PATH=/herogpui), so every
 * URL emitted into HTML must carry the prefix. Next applies `basePath` to its
 * own tags (`<script>`, `next/link`, `next/image` endpoints) automatically,
 * but not to hand-built URLs: public/ asset paths passed to plain `<img>`, or
 * `href`s rendered by React Aria components (see `useSiteHref` in
 * `components/site/providers.tsx`). Those go through this helper.
 */

const BASE_PATH = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

/**
 * Prefix a public/ path with the configured base path. With the default empty
 * base path this is the identity, so dev builds are unchanged.
 */
export function publicUrl(publicPath: string): string {
  return `${BASE_PATH}${publicPath}`;
}
