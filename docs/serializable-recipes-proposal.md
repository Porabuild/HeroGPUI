# Proposal: Serializable Component Recipes in `herogpui-theme`

**Status:** Design proposal (no schema committed yet; additive to `ComponentThemes` and `ThemeDocument`)  
**Starting points:**
- `crates/herogpui-theme/src/components.rs` (`ComponentThemes`, `ComponentTheme<T>`, `ComponentStyle`, `ComponentColor`, and typed `*Style` structs)
- `crates/herogpui-theme/src/theme_document.rs` (`ThemeDocument`, `#[serde(deny_unknown_fields)]`, `oklch()` / `oklcha()` / hex color parsing)
- Consumer bridge reference: Poratake's `src/main/app-gpui/src/theme/bridge.rs` and `src/main/app-gpui/src/editor/toolbar_select.rs`

---

## 1. Motivation & What Already Exists

HeroGPUI's customization surface already separates two concerns:

1. **Theme tokens & JSON documents (`ThemeDocument`, `--features serde`):** sparse overrides for `ThemeColors` and `LayoutTheme` (`radius`, `field_radius`, `tooltip_delay_ms`, roles, surfaces) that compose cleanly with `Theme::builder(..)` and live reload (`theme_watch`).
2. **Typed component defaults and named recipes (`Theme.components: ComponentThemes`):**
   - Already implemented: `slider`, `switch`, `select`, `menu`, `button`, `text_field`, and newly added `tooltip`, `popover`, `textarea`, `tabs`.
   - Each entry is a `ComponentTheme<T>` carrying `defaults: T` plus `recipes: HashMap<SharedString, T>`, resolved at render time via `resolve_named(component, &names)` with stackable `ComponentStyle::refine` semantics and opt-in `UnknownRecipeDiagnostic` hooks.
   - To eliminate manual row-width measurement in consumers such as Poratake's `toolbar_select.rs`, `Select` now provides [`Select::intrinsic_panel_width`] and [`Select::fit_panel_width`], which read the active `SelectStyle` recipe directly.

However, `ComponentThemes` currently lives only in Rust (`#[cfg(feature = "serde")]` covers `Theme`, colors, and layout via `ThemeDocument`, not `Theme.components`). Consumer theme bridges (such as Poratake's `bridge.rs`) therefore duplicate JSON/CSS token tables into Rust helper code that constructs `ComponentThemes` imperatively.

---

## 2. Why Closure-Based Styles Cannot Simply Be Serialized

Treating `ComponentThemes` as `#[derive(Serialize, Deserialize)]` fails for four structural reasons, three of which are visible directly in `ButtonStyle` and Poratake's `bridge.rs`:

1. **`ButtonStyle::style(impl FnOnce(Div) -> Div)` captures arbitrary Rust code and GPUI `StyleRefinement`:**
   - Although `ButtonStyle::style` evaluates `style(gpui::div()).style().clone()` immediately into `Option<gpui::StyleRefinement>`, `gpui::StyleRefinement` itself is an opaque GPUI struct containing flexbox/grid/overflow/paint fields without `Serialize`/`Deserialize` implementations (and `serde` derives cannot be added upstream without changes to `gpui` across every transitive layout type).
   - Worse, closures in real apps also capture computed expressions and layout constants (for example, `bridge.rs`'s `compact`, `overlay-trigger`, and `video-aspect` recipes compute `.h(...)`, `.w(...)`, `.px(...)`, `.gap(...)`, `.text_size(...)`, `.line_height(...)`, and `.opacity(...)`).

2. **Closure-driven content slots are behavioral, not stylistic:**
   - Slots like `Select::item_content`, `Select::value_content`, `Tooltip::body`, or `Switch::thumb_icons` construct dynamic GPUI element subtrees per state/key. They belong in Rust component composition, not JSON theme files.

3. **Runtime palette derivation inside Rust recipe definitions:**
   - In `bridge.rs`, recipes such as `overlay-trigger` (`vars.foreground.opacity(0.15)`), `video-aspect` (`vars.muted_background`), and `danger-text` (`vars.destructive`) compute derived alpha/color values from resolved `ThemeVars` instead of only naming fixed semantic roles. Plain `"background": "#..."` literals would freeze light/dark polarity unless the document format supports either token references or alpha modifiers.

4. **GPUI newtypes (`Pixels`, `SharedString`, `Hsla`) need stable wire representations:**
   - `ThemeDocument` intentionally uses plain JSON scalar conventions (`f32` pixels instead of serialized `Pixels`, `"oklch(...)"` / `"#rrggbb"` strings instead of raw `Hsla` fields, and `#[serde(deny_unknown_fields)]`) so theme files stay readable, versionable, and decoupled from internal crate representations.

**Core rule of the proposal:** Separate **declarative component style data** (serializable in `ThemeDocument`) from **imperative GPUI refinements and element closures** (kept in Rust, Layered cleanly on top).

---

## 3. Proposed Architecture

### 3.1 Keep All Pure-Data Component Styles Serializable

Ten component styles are pure declarative records of `Option<T>` primitives (`Pixels`, `u64`, `bool`, `ComponentColor`, `Variant`, `FieldVariant`):

- `SliderStyle` (`radius`)
- `SwitchStyle` (`radius`)
- `SelectStyle` (`variant`, `height`, `padding_x`, `padding_y`, `trigger_text_size`, `row_height`, `row_padding_x`, `row_padding_y`, `row_text_size`, `panel_padding`, `panel_gap`, `panel_min_width`, `radius`, `row_hover_bg`, `is_bare`)
- `MenuStyle` (`panel_min_width`, `panel_max_width`, `panel_max_height`, `panel_padding`, `panel_gap`, `row_height`, `row_padding_x`, `row_padding_y`, `row_text_size`, `row_gap`, `row_hover_bg`, `row_hover_foreground`, `radius`, `separator_inset`, `separator_thickness`, `animate_entry`)
- `TextFieldStyle` (`variant`, `height`, `padding_x`, `text_size`, `radius`, `is_bare`, `focus_ring`, `background`, `foreground`, `placeholder`)
- `TextAreaStyle` (`variant`, `padding_x`, `text_size`, `radius`, `is_bare`, `focus_ring`, `background`, `foreground`, `placeholder`)
- `TooltipStyle` (`radius`, `offset`, `delay_ms`, `close_delay_ms`, `padding`, `text_size`, `background`, `foreground`)
- `PopoverStyle` (`radius`, `padding`, `offset`, `panel_width`, `background`, `foreground`)
- `TabsStyle` (`radius`, `list_bg`, `indicator_bg`, `indicator_shadow`, `list_padding`, `hover_fill`, `text_size`, `vertical_tab_min_width`)
- **Pure-data fields of `ButtonStyle`**, plus an optional structured box-metrics sub-record (Section 3.2)

For these types, parallel serde DTOs (matching `ThemeDocument`'s pattern rather than changing runtime `Option<Pixels>` storage) can deserialize cleanly under `#[cfg(feature = "serde")]`.

### 3.2 Promote Common Button Box Metrics Out of `StyleRefinement` Closures

Inspecting Poratake's `bridge.rs` reveals that **every** `ButtonStyle::style(|el| ...)` closure uses only six layout/text primitives:

- `Compact`, `video-aspect`: `h`, `px`, `text_size`, `line_height`
- `overlay-trigger`: `h`, `w`, `px`, `gap`
- `overlay-disabled`: `opacity`

Representing those explicitly as typed `Option` fields on `ButtonStyle` (or a `ButtonLayoutMetrics` struct):

```rust
// Additive fields on ButtonStyle (all Option<_>, refined like every scalar field):
// height, min_width, width, padding_x, text_size, line_height, gap, opacity
```

means **every single button recipe in Poratake** (`compact`, `muted-action`, `overlay-trigger`, `overlay-disabled`, `overlay-trigger-tinted`, `video-aspect`, `video-aspect-selected`, `muted`, `danger`, `danger-text`, `overlay`, `preview`, `preview-pill`) becomes 100% expressible as pure data — no `.style(|el| ...)` closure required!

`ButtonStyle::style(|el| ...)` remains available in Rust as an escape hatch for arbitrary GPUI `Div` properties (`border_*`, `flex_shrink_0`, custom transforms) and continues to fail loudly/explicitly or be omitted when serializing a Rust theme to JSON rather than silently dropping rules.

### 3.3 Serializable `ComponentColorRef` (Semantic Roles + Alpha Modifiers)

`ComponentColor` already covers semantic references (`Background`, `Foreground`, `Muted`, `Surface`, `Role(Color)`, `RoleHover(Color)`, `RoleSoft(Color)`, etc.) plus `Literal(Hsla)`. To express Poratake's `vars.foreground.opacity(0.15)` cleanly in JSON without hardcoding theme-specific hex values:

```json
{
  "token": "foreground",
  "alpha": 0.15
}
```

Wire schema options accepted by deserialization (`#[serde(untagged)]`):
1. **Color string:** `"oklch(0.62 0.21 259)"`, `"#ff3b30"`, `"#00000020"` (reusing `ThemeDocument`'s parser) -> `ComponentColor::Literal(...)`.
2. **Token name string:** `"background"`, `"foreground"`, `"muted"`, `"surface"`, `"surface-foreground"`, `"surface-secondary"`, `"surface-tertiary"`, `"border"`, `"field-background"`, `"field-foreground"`, `"field-placeholder"`, `"accent"`, `"accent-foreground"`, `"accent-hover"`, `"accent-soft"`, `"danger"`, `"danger-foreground"`, etc.
3. **Token + alpha object:** `{ "token": "foreground", "alpha": 0.15 }` (adds an `Alpha(Box<ComponentColor>, f32)` or pre-resolved modifier variant so a single JSON recipe adapts automatically to Light and Dark palettes).

### 3.4 `ThemeDocument::components` Wire Schema

Extend `ThemeDocument` with a single optional field so existing documents and `#[serde(deny_unknown_fields)]` remain backwards-compatible:

```json
{
  "id": "compact-desktop",
  "base": "dark",
  "radius": 2.0,
  "components": {
    "tooltip": {
      "defaults": { "delay_ms": 150, "padding": 6.0, "radius": 4.0 },
      "recipes": {
        "compact": { "padding": 4.0, "text_size": 11.0 }
      }
    },
    "select": {
      "defaults": { "variant": "secondary", "padding_y": 8.0 },
      "recipes": {
        "toolbar": { "height": 28.0, "padding_x": 8.0, "panel_padding": 6.0, "panel_min_width": 64.0 }
      }
    },
    "button": {
      "recipes": {
        "overlay-trigger": {
          "size": "sm",
          "height": 32.0,
          "width": 48.0,
          "padding_x": 6.0,
          "gap": 4.0,
          "foreground": { "token": "foreground", "alpha": 0.85 },
          "hover_bg": { "token": "foreground", "alpha": 0.15 }
        }
      }
    }
  }
}
```

### 3.5 Precedence & Hybrid Composition (`JSON Document + Rust Extension`)

Consumers often load palette + standard data recipes from JSON while still attaching Rust-only `ButtonStyle::style` hooks or computed constants. `ThemeBuilder` supports this with zero breakage if `components` resolution merges rather than blindly replacing:

1. Start from `ThemeDocument::apply(&mut ThemeBuilder)` -> populates `ComponentThemes` data defaults and recipes.
2. Add a non-destructive builder method `ThemeBuilder::refine_components(ComponentThemes)` (or `ComponentTheme::refine(&mut self, other)`) that **merges** recipe maps by name (`ComponentStyle::refine`) instead of overwriting the map wholesale, while keeping `.components(...)` replace semantics intact for callers migrating gradually.
3. At render time, components keep the existing strict cascade:
   `stock component fallback` < `ComponentTheme.defaults` < `recipes[0..n]` (in stack order, with optional `UnknownRecipeDiagnostic`) < `instance builders (.padding(..), .height(..), ...)` < `instance .sx(|el| ...)` (scoped per painted part: root/wrapper `sx` refines the owning root element rather than implicitly overriding independent child chrome such as detached popovers or field shells unless a part explicitly extracts it via `util::sx_*`).

### 3.6 Validation, Diagnostics & Audit Parity

1. **Schema validation at load time (`ThemeDocumentError`):**
   - Unknown component keys or unknown style property names fail fast via `#[serde(deny_unknown_fields)]`.
   - Invalid numeric ranges (for example, negative `padding`/`radius`, `opacity` outside `0.0..=1.0`, `alpha` outside `0.0..=1.0`) return structured errors with field paths (e.g. `components.tooltip.recipes.compact.padding`).
2. **Runtime unknown-recipe diagnostics:**
   - The opt-in `set_unknown_recipe_diagnostic_handler` already ships on `herogpui-theme`. In dev/CI builds, JSON-loaded themes automatically surface misspelled recipe names (`requested` vs `available`) at first render.
3. **Round-trip / export semantics:**
   - Exporting a `ButtonStyle` that contains a Rust `StyleRefinement` (`style.is_some()`) should return an explicit `Err(ThemeSerializeError::NonSerializableClosureStyle { component: "button", recipe })` rather than silently omitting rules. Pure-data styles round-trip losslessly.
4. **Serde parity audit:**
   - Extend `.shots/theme_serde_audit.py` so every typed field in `*Style` structs has a matching DTO key in `theme_document.rs` (excluding opt-in opaque `ButtonStyle::style`).

---

## 4. Migration Path for Consumers (e.g., Poratake)

1. **Immediate (available today):** Move `Tooltip`, `Popover`, `TextArea`, and `Tabs` surface customizations onto `ComponentThemes::{tooltip, popover, textarea, tabs}`, remove manual `toolbar_select` pixel shaping in favor of `Select::fit_panel_width(true)` / `Select::intrinsic_panel_width`, and enable `set_unknown_recipe_diagnostic_handler` in debug builds / tests.
2. **Phase 2 (this proposal, data recipes):** Add serde DTO structs for the 9 pure-data styles + `ButtonStyle`'s scalar/token fields + `height`/`width`/`padding_x`/`text_size`/`line_height`/`gap`/`opacity` promotion on `ButtonStyle`, wired under `ThemeDocument::components`.
3. **Phase 3 (live reload):** Hot-reloaded theme JSON (`theme_watch`) can tune spacing, radii, delays, and named recipes in running apps without recompiling `bridge.rs`.
