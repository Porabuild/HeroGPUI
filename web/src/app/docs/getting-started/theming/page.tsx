import type { Metadata } from "next";
import { PageHeader } from "@/components/ui/page-header";
import { CodeBlock } from "@/components/ui/code-block";
import { StaticTable } from "@/components/ui/static-table";

export const metadata: Metadata = {
  title: "Theming",
  description:
    "Use OKLCH semantic tokens, roles, surfaces, fields, and layout values across your application.",
};

const IMPORT_LINE = `use herogpui::theme::{ThemeProvider, ActiveTheme};`;

const READ_TOKENS = `fn my_view(cx: &App) -> impl IntoElement {
    let primary = cx.role(Color::Accent);
    div().bg(primary.color).text_color(primary.foreground)
}`;

const TOKEN_HELPERS = `div().bg(cx.colors().surface.background)
div().bg(cx.colors().accent.soft())
div().text_color(cx.colors().muted)`;

const ROLE_DERIVED = `let accent = cx.role(Color::Accent);

accent.color             // --accent
accent.foreground        // --accent-foreground
accent.hover()           // color-mix(in oklab, var(--accent) 90%, var(--accent-foreground) 10%)
accent.soft()            // color-mix(in oklab, var(--accent) 15%, transparent)
accent.soft_hover()      // color-mix(in oklab, var(--accent) 20%, transparent)
accent.soft_foreground(cx.colors().foreground)
                         // color-mix(in oklab, var(--accent) 70%, var(--foreground) 30%)`;

interface TokenRow {
  token: string;
  rust: string;
  light: string;
  dark: string;
  note: string;
  swatch?: string;
  swatchDark?: string;
}

function Swatch({ value, dark }: { value?: string; dark?: string }) {
  if (!value) return <span className="text-muted">—</span>;
  return (
    <span className="flex items-center gap-1.5">
      <span
        aria-hidden
        className="inline-block size-4 shrink-0 rounded-sm border border-separator"
        style={{ background: value }}
      />
      {dark && (
        <span
          aria-hidden
          className="inline-block size-4 shrink-0 rounded-sm border border-separator"
          style={{ background: dark }}
        />
      )}
    </span>
  );
}

function TokenTable({
  label,
  rows,
  showValues = true,
}: {
  label: string;
  rows: TokenRow[];
  showValues?: boolean;
}) {
  return (
    <StaticTable
      className="mt-4"
      columns={[
        { header: "Token", id: "token", isRowHeader: true },
        { header: "Rust", id: "rust" },
        ...(showValues ? [{ header: "Light / dark", id: "swatch" }] : []),
        { header: "Value & notes", id: "note" },
      ]}
      label={label}
      layout="prose"
      rows={rows.map((row) => ({
        cells: [
          <code className="font-mono text-xs break-all" key="token">
            {row.token}
          </code>,
          <code className="font-mono text-xs break-all" key="rust">
            {row.rust}
          </code>,
          ...(showValues ? [<Swatch dark={row.swatchDark} key="swatch" value={row.swatch} />] : []),
          <span className="text-sm text-muted" key="note">
            <code className="font-mono text-xs">{row.light}</code>
            {row.note ? <> — {row.note}</> : null}
          </span>,
        ],
        id: row.token.replace(/[^a-z0-9]+/gi, "-"),
      }))}
    />
  );
}

const BASE_ROWS: TokenRow[] = [
  {
    token: "--background",
    rust: "colors.background",
    light: "oklch(0.9702 0 0)",
    dark: "oklch(0.12 0.005 285.823)",
    note: "the page",
    swatch: "oklch(0.9702 0 0)",
    swatchDark: "oklch(0.12 0.005 285.823)",
  },
  {
    token: "--foreground",
    rust: "colors.foreground",
    light: "oklch(0.2103 0.0059 285.89)",
    dark: "oklch(0.9911 0 0)",
    note: "body text",
    swatch: "oklch(0.2103 0.0059 285.89)",
    swatchDark: "oklch(0.9911 0 0)",
  },
  {
    token: "--muted",
    rust: "colors.muted",
    light: "oklch(0.5517 0.0138 285.94)",
    dark: "oklch(0.705 0.015 286.067)",
    note: "de-emphasised body text and icons",
    swatch: "oklch(0.5517 0.0138 285.94)",
    swatchDark: "oklch(0.705 0.015 286.067)",
  },
  {
    token: "--scrollbar",
    rust: "colors.scrollbar",
    light: "foreground at 15% alpha",
    dark: "foreground at 15% alpha",
    note: "the thumb",
  },
  {
    token: "--border",
    rust: "colors.border",
    light: "oklch(0.9 0.004 286.32)",
    dark: "oklch(0.28 0.006 286.033)",
    note: "one step darker than --separator",
    swatch: "oklch(0.9 0.004 286.32)",
    swatchDark: "oklch(0.28 0.006 286.033)",
  },
  {
    token: "--separator",
    rust: "colors.separator",
    light: "oklch(0.92 0.004 286.32)",
    dark: "oklch(0.25 0.006 286.033)",
    note: "rules between rows",
    swatch: "oklch(0.92 0.004 286.32)",
    swatchDark: "oklch(0.25 0.006 286.033)",
  },
  {
    token: "--focus",
    rust: "colors.focus",
    light: "same as --accent",
    dark: "same as --accent",
    note: "the focus ring",
    swatch: "oklch(0.6204 0.195 253.83)",
    swatchDark: "oklch(0.6204 0.195 253.83)",
  },
  {
    token: "--link",
    rust: "colors.link",
    light: "same as --foreground",
    dark: "same as --foreground",
    note: "link text",
  },
  {
    token: "--backdrop",
    rust: "colors.backdrop",
    light: "black at 50% alpha",
    dark: "black at 60% alpha",
    note: "the scrim behind modals and drawers",
  },
];

const CONTAINER_ROWS: TokenRow[] = [
  {
    token: "--surface",
    rust: "colors.surface.background / .foreground",
    light: "white",
    dark: "oklch(0.2103 0.0059 285.89)",
    note: "non-floating components: cards, accordions, disclosure groups",
    swatch: "oklch(1 0 0)",
    swatchDark: "oklch(0.2103 0.0059 285.89)",
  },
  {
    token: "--surface-secondary",
    rust: "colors.surface_secondary",
    light: "oklch(0.9524 0.0013 286.37)",
    dark: "oklch(0.257 0.0037 286.14)",
    note: "first step away from the page",
    swatch: "oklch(0.9524 0.0013 286.37)",
    swatchDark: "oklch(0.257 0.0037 286.14)",
  },
  {
    token: "--surface-tertiary",
    rust: "colors.surface_tertiary",
    light: "oklch(0.9373 0.0013 286.37)",
    dark: "oklch(0.2721 0.0024 247.91)",
    note: "second step away from the page",
    swatch: "oklch(0.9373 0.0013 286.37)",
    swatchDark: "oklch(0.2721 0.0024 247.91)",
  },
  {
    token: "--overlay",
    rust: "colors.overlay.background / .foreground",
    light: "white",
    dark: "oklch(0.2103 0.0059 285.89)",
    note: "floating components: tooltips, popovers, modals, menus — --overlay *is* --surface in dark mode; the shadow separates them",
    swatch: "oklch(1 0 0)",
    swatchDark: "oklch(0.2103 0.0059 285.89)",
  },
  {
    token: "--segment",
    rust: "colors.segment.background / .foreground",
    light: "white",
    dark: "oklch(0.3964 0.01 285.93)",
    note: "the selected segment of a segmented control (tabs, toggle groups)",
    swatch: "oklch(1 0 0)",
    swatchDark: "oklch(0.3964 0.01 285.93)",
  },
];

interface RoleRow {
  role: string;
  rust: string;
  light: string;
  dark: string;
  note: string;
}

const ROLE_ROWS: RoleRow[] = [
  {
    role: "--default",
    rust: "colors.default",
    light: "oklch(0.94 0.001 286.375)",
    dark: "oklch(0.274 0.006 286.033)",
    note: "the neutral backbone of the system",
  },
  {
    role: "--accent",
    rust: "colors.accent",
    light: "oklch(0.6204 0.195 253.83)",
    dark: "oklch(0.6204 0.195 253.83)",
    note: "the brand color",
  },
  {
    role: "--success",
    rust: "colors.success",
    light: "oklch(0.7329 0.1935 150.81)",
    dark: "oklch(0.7329 0.1935 150.81)",
    note: "not overridden in dark mode; only its soft shares are",
  },
  {
    role: "--warning",
    rust: "colors.warning",
    light: "oklch(0.7819 0.1585 72.33)",
    dark: "oklch(0.8203 0.1388 76.34)",
    note: "",
  },
  {
    role: "--danger",
    rust: "colors.danger",
    light: "oklch(0.6532 0.2328 25.74)",
    dark: "oklch(0.594 0.1967 24.63)",
    note: "",
  },
];

const FIELD_ROWS: TokenRow[] = [
  {
    token: "--field-background",
    rust: "field.background",
    light: "white",
    dark: "oklch(0.2103 0.0059 285.89)",
    note: "in dark mode this is the surface colour, not --default, which is two steps lighter",
    swatch: "oklch(1 0 0)",
    swatchDark: "oklch(0.2103 0.0059 285.89)",
  },
  {
    token: "--field-foreground",
    rust: "field.foreground",
    light: "oklch(0.2103 0.0059 285.89)",
    dark: "oklch(0.9911 0 0)",
    note: "typed text",
    swatch: "oklch(0.2103 0.0059 285.89)",
    swatchDark: "oklch(0.9911 0 0)",
  },
  {
    token: "--field-placeholder",
    rust: "field.placeholder",
    light: "the muted token",
    dark: "the muted token",
    note: "placeholder text",
  },
  {
    token: "--field-border",
    rust: "field.border",
    light: "transparent",
    dark: "transparent",
    note: "--field-border-width is 0 by default; the token exists for a caller who gives their fields a border",
  },
];

const LAYOUT_ROWS: TokenRow[] = [
  {
    token: "--spacing",
    rust: "layout.spacing",
    light: "0.25rem",
    dark: "",
    note: "the spacing unit",
  },
  {
    token: "--radius",
    rust: "layout.radius",
    light: "0.5rem (8px)",
    dark: "",
    note: "the base every other radius is calculated from",
  },
  {
    token: "--radius-xs … --radius-4xl",
    rust: "layout.radius_xs() … radius_4xl()",
    light: "0.25× … 4× --radius",
    dark: "",
    note: "the calculated steps",
  },
  {
    token: "capped(r)",
    rust: "layout.capped(r)",
    light: "min(32px, r)",
    dark: "",
    note: "caps rounded-* and rounded-full with min() so an oversized --radius cannot distort a component",
  },
  {
    token: "--field-radius",
    rust: "layout.field_radius",
    light: "calc(var(--radius) * 1.5) = 12px",
    dark: "",
    note: "every form field",
  },
  {
    token: "--border-width",
    rust: "layout.border_width",
    light: "1px",
    dark: "",
    note: "the weight a separator draws; there is no per-instance override",
  },
  {
    token: "--field-border-width",
    rust: "layout.field_border_width",
    light: "0px",
    dark: "",
    note: "fields separate with their background, not a border",
  },
  {
    token: "--disabled-opacity",
    rust: "layout.disabled_opacity",
    light: "0.5",
    dark: "",
    note: "",
  },
  {
    token: "--ring-offset-width",
    rust: "layout.ring_offset_width",
    light: "2px",
    dark: "",
    note: "",
  },
  {
    token: "--surface-shadow",
    rust: "layout.surface_shadow",
    light: "cards, accordions and other inline containers",
    dark: "empty",
    note: "dark mode drops all three shadows",
  },
  {
    token: "--overlay-shadow",
    rust: "layout.overlay_shadow",
    light: "tooltips, popovers, modals and menus",
    dark: "empty",
    note: "dark mode keeps only a 1px inset highlight, reproduced as a hairline border",
  },
  {
    token: "--field-shadow",
    rust: "layout.field_shadow",
    light: "inputs and other form controls",
    dark: "empty",
    note: "",
  },
  {
    token: "--skeleton-animation",
    rust: "layout.skeleton_animation",
    light: "shimmer (default)",
    dark: "",
    note: "SkeletonAnimation::{Shimmer, Pulse, None}",
  },
  {
    token: "--tooltip-delay",
    rust: "layout.tooltip_delay_ms",
    light: "1500ms",
    dark: "",
    note: "Tooltip reads it as its default delay",
  },
  {
    token: "--tooltip-close-delay",
    rust: "layout.tooltip_close_delay_ms",
    light: "500ms",
    dark: "",
    note: "",
  },
];

export default function ThemingPage() {
  return (
    <>
      <PageHeader
        title="Theming"
        description="Use OKLCH semantic tokens, roles, surfaces, fields, and layout values across your application."
        importLine={IMPORT_LINE}
      />

      <p>
        Every color in HeroGPUI is a semantic token resolved from the active <code>Theme</code>{" "}
        global. The theme crate transcribes base values from HeroUI&apos;s{" "}
        <code>packages/styles/themes/default/variables.css</code> in <code>oklch()</code>, and
        derived values use the stylesheet&apos;s <code>color-mix(in oklab, …)</code> weights.
      </p>
      <p>
        The <code>ActiveTheme</code> trait reaches the tokens from any GPUI context —{" "}
        <code>&amp;App</code>, <code>&amp;mut App</code>, <code>Context&lt;T&gt;</code> (they all
        deref): <code>cx.colors()</code> for the palette, <code>cx.role(Color::Accent)</code> for a
        role, <code>cx.layout()</code> for the layout tokens.
      </p>
      <div className="mt-4">
        <CodeBlock code={READ_TOKENS} lang="rust" />
      </div>

      <h2 id="base-tokens">Base tokens</h2>
      <p>
        Nine base tokens feed the rest of the palette. The system uses semantic tokens instead of
        numbered scales.
      </p>
      <TokenTable label="Base tokens" rows={BASE_ROWS} />

      <h2 id="containers">Containers</h2>
      <p>
        Layered surfaces, one step at a time away from the page: <code>surface</code> for components
        that sit inline, <code>overlay</code> for the ones that float.
      </p>
      <TokenTable label="Container tokens" rows={CONTAINER_ROWS} />
      <p>
        Four more are derived on <code>ThemeColors</code> with the stylesheet&apos;s weights:
      </p>
      <ul>
        <li>
          <code>background_secondary()</code> —{" "}
          <code>color-mix(in oklab, var(--background) 96%, var(--foreground) 4%)</code>
        </li>
        <li>
          <code>background_tertiary()</code> — <code>… 92% / 8%</code>
        </li>
        <li>
          <code>background_inverse()</code> — <code>var(--foreground)</code>
        </li>
        <li>
          <code>separator_secondary()</code> / <code>separator_tertiary()</code> —{" "}
          <code>var(--surface)</code> mixed 85/15, then 81/19, toward{" "}
          <code>var(--surface-foreground)</code>
        </li>
      </ul>

      <h2 id="roles">Roles</h2>
      <p>
        Five semantic roles cover neutral, accent and status colors: <code>default</code>,{" "}
        <code>accent</code>, <code>success</code>, <code>warning</code> and <code>danger</code>.
        Each <code>RoleColor</code> carries a base value and readable foreground; its other shades
        derive from the stylesheet&apos;s <code>color-mix</code> weights.
      </p>
      <StaticTable
        className="mt-4"
        columns={[
          { header: "Role", id: "role", isRowHeader: true },
          { header: "Rust", id: "rust" },
          { header: "Light / dark", id: "swatch" },
          { header: "Light value", id: "light" },
          { header: "Dark value", id: "dark" },
        ]}
        label="Semantic roles"
        layout="prose"
        rows={ROLE_ROWS.map((row) => ({
          cells: [
            <code className="font-mono text-xs" key="role">
              {row.role}
            </code>,
            <code className="font-mono text-xs" key="rust">
              {row.rust}
            </code>,
            <Swatch dark={row.dark} key="swatch" value={row.light} />,
            <code className="font-mono text-xs break-all" key="light">
              {row.light}
            </code>,
            <code className="font-mono text-xs break-all" key="dark">
              {row.dark}
            </code>,
          ],
          id: row.role.replace(/[^a-z0-9]+/gi, "-"),
        }))}
      />
      <p>
        These are the values and methods a <code>RoleColor</code> derives:
      </p>
      <div className="mt-4">
        <CodeBlock code={ROLE_DERIVED} lang="rust" />
      </div>
      <p>
        The token helpers are just these fields and methods, so any view paints with the same
        vocabulary the components use:
      </p>
      <div className="mt-4">
        <CodeBlock code={TOKEN_HELPERS} lang="rust" />
      </div>

      <h2 id="fields">Fields</h2>
      <p>
        Form-field tokens are kept separate from buttons so inputs can be styled independently.{" "}
        <code>field.hover()</code> mixes the background 90/2 toward its foreground (the weights are
        the stylesheet&apos;s, normalised by CSS), and <code>field.focus()</code> is{" "}
        <code>var(--field-background)</code> — the ring and border do the pointing.
      </p>
      <TokenTable label="Field tokens" rows={FIELD_ROWS} />

      <h2 id="layout-tokens">Layout tokens</h2>
      <p>
        Layout uses one <code>--radius</code> base with calculated steps and component-semantic
        shadows. Read these values through <code>cx.layout()</code>.
      </p>
      <TokenTable label="Layout tokens" rows={LAYOUT_ROWS} showValues={false} />
    </>
  );
}
