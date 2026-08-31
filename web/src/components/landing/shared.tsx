import { existsSync } from "node:fs";
import path from "node:path";
import type { ReactNode } from "react";
import { cn } from "@heroui/react";
import { publicUrl } from "@/lib/public-url";

/**
 * Helpers shared by the landing sections: the public-asset base path (local
 * dev serves at "/", production is mounted under /herogpui), honest window
 * framing for the native GPUI screenshots, and the shared section heading.
 * (The one client-side piece, CtaLink, lives in cta-link.tsx.)
 */

/**
 * Whether a public/ asset exists at render time. The screenshots are copied
 * into public/shots by the data pipeline and may legitimately be absent;
 * callers render a quiet placeholder instead of a broken image.
 */
export function publicFileExists(publicPath: string): boolean {
  return existsSync(path.join(process.cwd(), "public", publicPath.replace(/^\//, "")));
}

export interface ShotWindowProps {
  /** Public path of the screenshot, e.g. "/shots/button.png". */
  src: string;
  alt: string;
  /** Aspect ratio class for the frame, e.g. "aspect-[12/7]". */
  aspect: string;
  /** Width/height of the source image, for layout stability. */
  width: number;
  height: number;
  caption: string;
  className?: string;
  loading?: "eager" | "lazy";
}

/**
 * A real GPUI screenshot framed as the native application window it was
 * captured from. The branded window bar identifies the image as a native
 * render while the screenshot remains visible as the source of truth. When
 * the image has not been copied into public/ yet, a quiet placeholder keeps
 * the layout instead of a broken <img>.
 */
export function ShotWindow({
  src,
  alt,
  aspect,
  width,
  height,
  caption,
  className,
  loading = "lazy",
}: ShotWindowProps) {
  const missing = !publicFileExists(src);
  return (
    <figure className={cn("shot-window m-0", className)}>
      <div
        className={cn(
          "shot-window-frame relative overflow-hidden rounded-xl border border-separator bg-surface shadow-none",
          aspect,
        )}
      >
        <div aria-hidden="true" className="window-bar shot-window-bar">
          <div>
            <span />
            <span />
            <span />
          </div>
          <span>HeroGPUI / native render</span>
          <span className="shot-window-status">GPUI</span>
        </div>
        {missing ? (
          <div
            aria-hidden="true"
            className="shot-window-placeholder absolute inset-x-0 bottom-0 top-9 flex items-center justify-center bg-surface-secondary"
          >
            <span className="px-6 text-center font-mono text-xs text-muted">{`public${src}`}</span>
          </div>
        ) : (
          <img
            alt={alt}
            className="shot-window-image absolute inset-x-0 bottom-0 top-9 h-full w-full object-cover object-top"
            decoding="async"
            height={height}
            loading={loading}
            src={publicUrl(src)}
            width={width}
          />
        )}
      </div>
      <figcaption className="shot-window-caption mt-3 flex items-center gap-2 text-xs text-muted">
        <span
          aria-hidden="true"
          className="shot-window-dot size-1.5 shrink-0 rounded-full bg-accent"
        />
        {caption}
      </figcaption>
    </figure>
  );
}

export interface SectionHeadingProps {
  eyebrow: string;
  title: string;
  sub?: ReactNode;
  align?: "left" | "center";
  className?: string;
}

/** Eyebrow + display heading + optional standfirst, shared by all sections. */
export function SectionHeading({
  eyebrow,
  title,
  sub,
  align = "left",
  className,
}: SectionHeadingProps) {
  return (
    <div className={cn("landing-section-heading", align === "center" && "text-center", className)}>
      <p className="font-mono text-xs font-medium tracking-[0.16em] text-accent uppercase">
        {eyebrow}
      </p>
      <h2 className="mt-3 text-3xl font-semibold tracking-tight text-foreground text-balance sm:text-4xl">
        {title}
      </h2>
      {sub && (
        <p
          className={cn(
            "mt-4 max-w-2xl text-base leading-relaxed text-muted",
            align === "center" && "mx-auto",
          )}
        >
          {sub}
        </p>
      )}
    </div>
  );
}
