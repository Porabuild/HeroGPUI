"use client";

import { Button } from "@heroui/react";
import { Moon, Sun } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";

const STORAGE_KEY = "herogpui-theme";

/**
 * Class-based dark mode toggle, persisted to localStorage. Both icons are
 * always rendered and shown/hidden with the `dark:` variant, so server and
 * client markup match exactly — no hydration mismatch, and the first paint
 * already shows the icon matching the pre-hydration theme script.
 */
export function ThemeToggle() {
  const [isDark, setIsDark] = useState(false);
  const timeoutRef = useRef<number | undefined>(undefined);

  // Read the (script-applied) state after mount only.
  useEffect(() => {
    setIsDark(document.documentElement.classList.contains("dark"));
    return () => window.clearTimeout(timeoutRef.current);
  }, []);

  const toggle = useCallback(() => {
    const next = !document.documentElement.classList.contains("dark");
    document.documentElement.classList.toggle("dark", next);
    document.documentElement.style.colorScheme = next ? "dark" : "light";
    try {
      localStorage.setItem(STORAGE_KEY, next ? "dark" : "light");
    } catch {
      // Private mode: the toggle still works for this page view.
    }
    setIsDark(next);
  }, []);

  return (
    <Button
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
      className="size-10"
      isIconOnly
      onPress={toggle}
      size="sm"
      variant="ghost"
    >
      <Sun className="size-4 dark:hidden" />
      <Moon className="hidden size-4 dark:block" />
    </Button>
  );
}
