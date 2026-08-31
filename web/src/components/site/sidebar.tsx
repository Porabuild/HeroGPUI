"use client";

import { Button, Drawer, Link, useOverlayState } from "@heroui/react";
import { PanelLeft } from "lucide-react";
import { usePathname } from "next/navigation";
import type { SidebarGroup } from "@/lib/catalog";
import { AI_LINKS, GETTING_STARTED_LINKS, isNavLinkActive } from "@/lib/nav";

interface SidebarNavProps {
  groups: SidebarGroup[];
  /** Called after a link is pressed, so the mobile drawer can close. */
  onNavigate?: () => void;
}

function SidebarSection({
  label,
  links,
  onNavigate,
}: {
  label: string;
  links: { href: string; label: string; exact?: boolean }[];
  onNavigate?: () => void;
}) {
  const pathname = usePathname();
  if (links.length === 0) return null;
  return (
    <section className="docs-sidebar-section py-3">
      <h3 className="docs-sidebar-label px-3 pb-1.5 text-xs font-semibold tracking-wider text-muted uppercase">
        {label}
      </h3>
      <ul>
        {links.map((link) => {
          const active = isNavLinkActive(pathname, link);
          return (
            <li key={link.href}>
              <Link
                aria-current={active ? "page" : undefined}
                className={
                  active
                    ? "block rounded-md bg-accent-soft px-3 py-1.5 text-sm font-medium text-accent-soft-foreground"
                    : "block rounded-md px-3 py-1.5 text-sm text-muted transition-colors hover:bg-surface-secondary hover:text-foreground"
                }
                href={link.href}
                onPress={onNavigate}
              >
                {link.label}
              </Link>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function SidebarNav({ groups, onNavigate }: SidebarNavProps) {
  return (
    <nav aria-label="Documentation" className="docs-sidebar-nav pb-8">
      <SidebarSection
        label="Getting Started"
        links={GETTING_STARTED_LINKS}
        onNavigate={onNavigate}
      />
      <SidebarSection label="AI" links={AI_LINKS} onNavigate={onNavigate} />
      {groups.map((group) => (
        <SidebarSection
          key={group.label}
          label={group.label}
          links={group.links.map((link) => ({ ...link, exact: true }))}
          onNavigate={onNavigate}
        />
      ))}
    </nav>
  );
}

/** Independently-scrolling rail for `lg` and up. */
export function SidebarRail({ groups }: { groups: SidebarGroup[] }) {
  return (
    <aside className="docs-sidebar hidden w-64 shrink-0 lg:block">
      <div className="scrollbar sticky top-16 max-h-[calc(100dvh-4rem)] overflow-y-auto border-r border-separator px-3 pt-6 pb-8">
        <SidebarNav groups={groups} />
      </div>
    </aside>
  );
}

/** "Browse docs" bar and HeroUI drawer for below `lg`. */
export function SidebarMobile({ groups }: { groups: SidebarGroup[] }) {
  const drawer = useOverlayState();

  return (
    <div className="docs-sidebar-mobile border-b border-separator lg:hidden">
      <div className="mx-auto w-full max-w-[1440px] px-4 py-2 sm:px-6">
        {/* Drawer.Root is a react-aria DialogTrigger: the open button must live
            inside it so the trigger slot gets a pressable child. */}
        <Drawer state={drawer}>
          <Button className="h-10" onPress={drawer.open} size="sm" variant="outline">
            <PanelLeft className="size-4" />
            Browse docs
          </Button>

          <Drawer.Backdrop />
          <Drawer.Content placement="left">
            <Drawer.Dialog>
              <Drawer.Header>
                <Drawer.Heading className="text-sm font-semibold text-foreground">
                  Browse docs
                </Drawer.Heading>
                <Drawer.CloseTrigger />
              </Drawer.Header>
              <Drawer.Body>
                <SidebarNav groups={groups} onNavigate={drawer.close} />
              </Drawer.Body>
            </Drawer.Dialog>
          </Drawer.Content>
        </Drawer>
      </div>
    </div>
  );
}
