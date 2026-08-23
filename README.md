# HeroGPUI

> 🚀 Beautiful, fast and modern **cross-platform Rust UI library** — a faithful
> port of [HeroUI v3](https://github.com/heroui-inc/heroui) to
> [GPUI](https://gpui.rs), Zed's GPU-accelerated UI framework.

HeroGPUI mirrors the HeroUI v3 design system: its OKLCH semantic tokens, layout
tokens, component variants/props and docs experience — rebuilt natively in Rust
for Windows, macOS and Linux.

```rust
use gpui::prelude::*;
use herogpui::prelude::*;

impl Render for MyApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(cx.colors().background)          // semantic token
            .font_family("Segoe UI")
            .child(
                Button::new("save")
                    .label("Save changes")
                    .variant(Variant::Primary)
                    .on_press(cx.listener(|this, _, _, cx| this.save(cx))),
            )
    }
}
```

## Status

**v3 parity** — all 71 components documented at
[heroui.com/docs/react/components](https://heroui.com/en/docs/react/components),
the v3 OKLCH token system, and a gallery that mirrors the upstream docs with the
same fifteen categories, light/dark and `llms.txt`.

Parity is measured, not asserted. `python .shots/api_audit.py` diffs every
documented v3 prop table — including the per-part tables (`Tooltip.Content`,
`Table.Column`, `Dropdown.Menu`, …) that make up most of v3's surface — against
the builders this crate exposes:

| | |
|---|---|
| documented props considered | 592 |
| implemented | 559 |
| deliberately not ported | 33 |
| real gaps | 0 |

Every omission carries a reason (`no-a11y-attrs`, `no-http`, `render-prop-arg`,
`constructor-arg`, `single-valued`, `state-entity-seeds-it`, …) in the audit's
`WONT_PORT` table, so nothing hides behind a blanket claim. Reasons can be
scoped per component, so a name that is genuinely absent in one place cannot be
excused by a blanket entry made for another.

Three more scripts keep the measurement honest in the directions a
prop-by-prop diff cannot see:

- `python .shots/extra_audit.py` runs the diff **backwards** — every builder we
  expose that v3 does not document. Asking only "is every documented prop
  implemented?" cannot catch a prop held over from v2, which is how
  `Card::is_pressable`, `ProgressBar::is_striped`, `RadioGroup::size` and the
  `radius` prop survived four audits. It separates names v3 documents on a
  sibling component (its per-component tables are incomplete: `Input` lists no
  `isInvalid` though every sibling field does) from names v3 documents nowhere,
  and every one of the latter is either deleted or recorded in `EXTRA_OK` with a
  reason.
- `python .shots/write_only.py` checks that no builder stores a value nothing
  ever reads — a prop that is accepted and ignored is worse than one that is
  missing.
- `python .shots/reason_audit.py` prints every recorded omission beside the v3
  row that documents it, so an excuse written once cannot quietly outlive the
  reason for it.

Components work **controlled or uncontrolled**, as they do in v3: pass
`is_selected` / `is_open` / `selected_key` to own the state, or
`default_selected` / `default_open` / `default_selected_key` / `default_value`
and let the component keep it.

The audit counts a prop the constructor takes positionally
(`ColorArea::new(id, value)`) as implemented; reading only the builders had
filed fourteen of those as omissions.

What is left is what a desktop toolkit does not have: ARIA attributes with no
accessibility tree to expose them to, `locale` without CLDR data, browser image
and soft-keyboard hints, the HTTP half of a `<form>` (`action` / `method` /
`encType` / `target`), and a handful of single-valued enums. Two are missing
*features* rather than unportable props, and are named as such in `WONT_PORT`:
`DateField` is a text field here, not v3's segmented one, and `TextArea` renders
one tall line because gpui 0.2.2 has no multi-line text layout.

This library tracks **v3 only**. Removed with the v2 token names
(`content1..4`, numbered 50–900 scales, `primary`/`secondary` as colors) and the
v2-only components (`Navbar`, `Image`, `User`, `Spacer`, `Code`, `Snippet`) are
the v2 props v3 deleted: `radius` everywhere, `color` on everything but the ten
components that still document it, `size` on every form field (v3 gives them one
height), and `isStriped`, `isBordered`, `isPressable`, `isHoverable`,
`isBlurred`, `isLoaded`, `isExternal`, `underline`, `showOutline`, `isInvisible`,
`strokeWidth` and `hideSeparator`.

| Category | Components |
|---|---|
| **Buttons** | Button, ButtonGroup, CloseButton, ToggleButton + ToggleButtonGroup |
| **Collections** | Dropdown, ListBox, TagGroup |
| **Colors** | ColorArea, ColorField, ColorPicker, ColorSlider, ColorSwatch, ColorSwatchPicker |
| **Controls** | Slider, Switch |
| **Data Display** | Badge, Chip, Table |
| **Date and Time** | Calendar, DateField, DatePicker, DateRangePicker, RangeCalendar, TimeField |
| **Feedback** | Alert, Meter, ProgressBar, ProgressCircle, Skeleton, Spinner |
| **Forms** | Checkbox, CheckboxGroup, Description, ErrorMessage, FieldError, Fieldset, Form, Input, InputGroup, InputOTP, Label, NumberField, RadioGroup, SearchField, TextArea, TextField |
| **Layout** | Card, Separator, Surface, Toolbar |
| **Media** | Avatar + AvatarGroup |
| **Navigation** | Accordion, Breadcrumbs, Disclosure + DisclosureGroup, Link, Pagination, Tabs |
| **Overlays** | AlertDialog, Drawer, Modal, Popover, Toast, Tooltip |
| **Pickers** | Autocomplete, ComboBox, Select |
| **Typography** | Kbd, Typography |
| **Utilities** | ScrollShadow |

**LLM docs:** `llms.txt` at repo root — full API reference for agents, mirroring
`heroui.com/react/llms-full.txt`.

## Workspace layout

```
crates/
  herogpui-core/        shared v3 vocabularies (Color, Variant, FieldVariant,
                         Prominence, Backdrop, Size), OKLCH + Oklab color math
  herogpui-theme/       v3 tokens from packages/styles/themes/default:
                         semantic OKLCH colors, layout tokens, ThemeProvider
  herogpui-components/  one module per @heroui/* package
  herogpui/             umbrella crate (like @heroui/react) + prelude
gallery/                showcase app: the fifteen v3 categories, theme switcher,
                         live examples for every component
  assets (gallery)/     icon set embedded via AssetSource
llms.txt                full API for LLMs
```

## Getting started

Prerequisites: latest stable Rust. GPUI needs platform tooling (Xcode on macOS;
Wayland/X11 dev packages on Linux; nothing extra on Windows).

```bash
cargo build                     # builds library + gallery
cargo run -p herogpui-gallery   # open the component gallery
```

### Using HeroGPUI in your app

```toml
[dependencies]
gpui = "0.2"
herogpui = { path = "../HeroGPUI/crates/herogpui" }
```

1. Register the theme provider at startup:
   `herogpui::theme::ThemeProvider::init(cx);`
2. Serve the icon SVGs (`gallery/assets/herogpui/icons/*`) from your
   `AssetSource` (`Application::new().with_assets(...)`).
3. Set `.bg(cx.colors().background)` + your font family on the root view.
4. Toggle dark mode anywhere with `herogpui::theme::toggle_light_dark(cx);`

## Theming

A faithful port of v3's `packages/styles/themes/default/variables.css`. Every
base value is transcribed verbatim in `oklch()`; every derived value is computed
with the same `color-mix(in oklab, …)` weights the stylesheet uses, so a
HeroGPUI theme and a HeroUI theme resolve to identical pixels.

- **Base** — `background`, `foreground`, `muted`, `scrollbar`, `border`,
  `separator`, `focus`, `link`, `backdrop`.
- **Containers** — `surface` (+ `surface_secondary`, `surface_tertiary`),
  `overlay` for floating panels, `segment` for segmented controls.
- **Roles** — `default`, `accent`, `success`, `warning`, `danger`. Each exposes
  `color`, `foreground`, and the derived `hover()`, `soft()`, `soft_hover()`
  and `soft_foreground()`. There are no numbered scales in v3.
- **Fields** — `field.background`, `field.foreground`, `field.placeholder`,
  `field.border`, plus `field.hover()` and `field.focus()`.
- **Layout** — `--radius` (8px) with `radius_xs()`…`radius_xl()`,
  `field_radius`, `border_width`, `disabled_opacity`, `ring_offset_width`,
  and the three semantic shadows `surface_shadow` / `overlay_shadow` /
  `field_shadow` (all transparent in dark mode, as in v3).
- **Custom themes** — override a base token and every derived value follows:

  ```rust
  use herogpui::core::oklch;

  let brand = Theme::builder("brand", Theme::dark())
      .accent(oklch(0.55, 0.23, 295.0))   // hover/soft/focus all derive
      .radius(px(6.))                     // field_radius follows at 1.5x
      .build();
  herogpui::theme::set_theme(brand, cx);
  ```

Read any token from any context via the `ActiveTheme` trait: `cx.colors()`,
`cx.role(Color::Accent)`, `cx.layout()`.

## Component API model

Components are plain builder structs implementing GPUI's `RenderOnce` — they
feel like React function components but compile to zero-allocation element
trees. Controlled state follows React semantics: you own the value, components
emit change callbacks (`on_change`, `on_press`, …). Text editing lives in
`InputState` entities (typing, selection via Shift+arrows, Ctrl/Cmd+A, caret,
clear button when `is_clearable`, backspace/delete, Ctrl/Cmd+V).

v3 replaced v2's `variant` × `color` matrix with distinct vocabularies, modelled
as separate enums so an invalid combination cannot be expressed:

| Enum | Values | Used by |
|---|---|---|
| `Variant` | `Primary`, `Secondary`, `Tertiary`, `Outline`, `Ghost`, `Danger`, `DangerSoft` | Button, ButtonGroup |
| `FieldVariant` | `Primary`, `Secondary` | every form control |
| `Prominence` | `Transparent`, `Default`, `Secondary`, `Tertiary` | Surface, Card |
| `Backdrop` | `Opaque`, `Blur`, `Transparent` | Modal, Drawer, AlertDialog |
| `Color` | `Default`, `Accent`, `Success`, `Warning`, `Danger` | components with a color role |

```rust
Button::new("save").label("Save").variant(Variant::Primary)
    .is_pending(self.saving)
    .on_press(cx.listener(|this, _, _, cx| this.save(cx)))

TextField::new(name.clone()).label("Email").is_required(true)
Switch::new("wifi").checked(self.wifi_on).on_change(move |v, _w, cx| { /* ... */ })
```

## Documentation

- **Gallery:** `cargo run -p herogpui-gallery` — the fifteen v3 categories,
  live examples, `HEROGPUI_PAGE` / `HEROGPUI_THEME` env vars for screenshots,
  `.shots/capture2.ps1`
- **LLM docs:** `llms.txt` at repo root
- **Getting Started in-gallery:** Introduction, Installation, Theming, Dark
  Mode, Customization

## License

MIT — matching upstream HeroUI.
