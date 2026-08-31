import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import type { ReactNode } from "react";
import { SiteProviders } from "@/components/site/providers";
import "./globals.css";

const geist = Geist({
  subsets: ["latin"],
  variable: "--font-geist",
  display: "swap",
});

const geistMono = Geist_Mono({
  subsets: ["latin"],
  variable: "--font-geist-mono",
  display: "swap",
});

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL ?? "https://porabuild.com/herogpui";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: "HeroGPUI — Rust UI for desktop apps",
    template: "%s — HeroGPUI",
  },
  description:
    "HeroGPUI is a typed Rust UI library for desktop applications, built on GPUI with OKLCH semantic tokens and light and dark themes for Windows, macOS and Linux.",
  openGraph: {
    type: "website",
    siteName: "HeroGPUI",
    url: "/",
    title: "HeroGPUI — Rust UI for desktop apps",
    description:
      "Build desktop interfaces in Rust with typed builders, OKLCH semantic tokens and a native gallery for Windows, macOS and Linux.",
  },
  twitter: {
    card: "summary",
    title: "HeroGPUI — Rust UI for desktop apps",
    description:
      "Build desktop interfaces in Rust with typed builders, OKLCH semantic tokens and a native gallery for Windows, macOS and Linux.",
  },
};

/**
 * Applies the persisted theme class before first paint so there is no flash.
 * HeroUI's default theme is class-based: `.dark` on an ancestor switches
 * the token set (see @heroui/styles themes/default/variables.css).
 */
const themeInitScript = `(function(){try{var s=localStorage.getItem("herogpui-theme");var d=s?s==="dark":window.matchMedia("(prefers-color-scheme: dark)").matches;var r=document.documentElement;r.classList.toggle("dark",d);r.style.colorScheme=d?"dark":"light";}catch(e){}})();`;

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" className={`${geist.variable} ${geistMono.variable}`} suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
      </head>
      <body className="min-h-dvh bg-background font-sans text-foreground antialiased">
        <SiteProviders>{children}</SiteProviders>
      </body>
    </html>
  );
}
