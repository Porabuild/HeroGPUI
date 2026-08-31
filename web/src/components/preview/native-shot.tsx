import { open } from "node:fs/promises";
import path from "node:path";

import Image from "next/image";
import { publicUrl } from "@/lib/public-url";

/**
 * The real GPUI capture from the desktop gallery, with intrinsic sizes read
 * from the PNG header at build time so the image reserves the right box in
 * both themes. `publicUrl` prefixes the `src` with the base path — under a
 * basePath mount, next/image's optimizer endpoint is prefixed but its raw
 * `url` parameter would not be, and the optimizer rejects that mismatch; a
 * fully prefixed path is served as-is.
 */

async function pngSize(publicPath: string): Promise<{ width: number; height: number } | null> {
  try {
    const handle = await open(path.join(process.cwd(), "public", publicPath), "r");
    try {
      const buffer = Buffer.alloc(24);
      const { bytesRead } = await handle.read(buffer, 0, 24, 0);
      // PNG: 8-byte signature, then the IHDR chunk at offset 12.
      if (bytesRead < 24 || buffer.readUInt32BE(12) !== 0x49484452) return null;
      const width = buffer.readUInt32BE(16);
      const height = buffer.readUInt32BE(20);
      return width > 0 && height > 0 ? { width, height } : null;
    } finally {
      await handle.close();
    }
  } catch {
    return null;
  }
}

export interface NativeShotProps {
  /** Gallery capture path from the catalog, e.g. `/shots/button-v3.png`. */
  shot: string;
  /** Dark-theme capture, when the gallery produced one. */
  shotDark: string | null;
  alt: string;
}

/** Light/dark GPUI captures, or null when no capture can be shown. */
export async function NativeShot({ shot, shotDark, alt }: NativeShotProps) {
  const light = await pngSize(shot);
  if (!light) return null;
  const dark = shotDark ? await pngSize(shotDark) : null;

  return (
    <figure className="space-y-2">
      <Image
        alt={alt}
        className={
          dark && shotDark
            ? "h-auto w-full rounded-lg border border-separator bg-surface-secondary dark:hidden"
            : "h-auto w-full rounded-lg border border-separator bg-surface-secondary"
        }
        height={light.height}
        sizes="(min-width: 1280px) 720px, 100vw"
        src={publicUrl(shot)}
        width={light.width}
      />
      {dark && shotDark ? (
        <Image
          alt={`${alt} (dark theme)`}
          className="hidden h-auto w-full rounded-lg border border-separator bg-surface-secondary dark:block"
          height={dark.height}
          sizes="(min-width: 1280px) 720px, 100vw"
          src={publicUrl(shotDark)}
          width={dark.width}
        />
      ) : null}
      <figcaption className="text-xs text-muted">
        Captured from the HeroGPUI desktop gallery — not a browser render.
        {!dark && !shotDark
          ? " (Light-theme capture; the gallery has not produced a dark one.)"
          : ""}
      </figcaption>
    </figure>
  );
}
