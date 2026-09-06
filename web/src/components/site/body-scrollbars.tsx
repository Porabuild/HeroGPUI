"use client";

import { useOverlayScrollbars } from "overlayscrollbars-react";
import { useEffect } from "react";

/**
 * Ghost scrollbar for the page body. Initialised on `document.body` (deferred
 * to idle) so the native bar disappears without breaking `position: sticky`
 * headers, anchors, or the docs `scroll-margin-top`. Skipped entirely under
 * `prefers-reduced-motion`, where the native bar stays.
 *
 * The theme script in `app/layout.tsx` puts
 * `data-overlayscrollbars-initialize` on the root element under exactly the
 * same condition, so the native bar is never painted before this runs. Keep
 * the two conditions in step.
 */
export function BodyScrollbars() {
  const [initialize] = useOverlayScrollbars({
    defer: true,
    options: {
      overflow: { x: "hidden", y: "scroll" },
      scrollbars: {
        theme: "os-theme-pb",
        visibility: "auto",
        autoHide: "scroll",
        autoHideDelay: 800,
        clickScroll: true,
      },
    },
  });

  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    initialize(document.body);
  }, [initialize]);

  return null;
}
