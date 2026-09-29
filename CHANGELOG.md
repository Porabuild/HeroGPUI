# Changelog

All notable changes to HeroGPUI are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). One
version covers `herogpui`, `herogpui-core`, `herogpui-theme`,
`herogpui-components` and `herogpui-gallery`.

## [Unreleased]

### Changed

- Website: the search palette follows the ARIA combobox pattern (a
  `combobox` input with `aria-activedescendant` over a `listbox` of
  `option`s, and a polite result count), and it indexes each component's
  Rust builders and types from the generated reference data, so
  `selection_mode` or `ListBoxItem` finds the component and opens its
  builder table.
- Website: every page has its own canonical URL, Open Graph and Twitter
  title and description; the site serves `sitemap.xml` and `robots.txt`;
  component pages link their primary type on docs.rs and their source file
  on GitHub at the release tag.
- Website: a `Content-Security-Policy` with `frame-ancestors` on every route
  (the WebAssembly gallery gets `'wasm-unsafe-eval'` and hashes of its inline
  scripts), and the security headers now also reach the bare `/herogpui`
  root. The landing specimen starts fetching the artifact on pointer, focus
  or touch intent instead of on the click. The 82 full-page captures no page
  showed are no longer published under `web/public/shots/` (the catalog
  tiles stay; `.shots/` keeps the goldens), and the unused `NativeShot` and
  `ShotWindow` components are removed. `web/DEPLOYMENT.md` records that pull
  requests already get Vercel preview deployments.

### Added

- CI checks the four library crates' public API with `cargo semver-checks`
  against their latest crates.io release on every pull request (a deliberate
  break carries the `semver:breaking` label), and the release workflow runs
  the same check before the GitHub Release and any publish.
- CI builds each library crate with `--no-default-features`.
- `.shots/lint.sh`: the lint gate (workspace-lint inheritance, clippy with
  warnings denied, cargo-deny) in bash, so it runs on macOS and Linux without
  PowerShell; CI calls it with `--require-deny`, and `--self-test` proves the
  inheritance check. `.shots/lint.ps1` now only forwards to it.
- `.shots/stale_docs_audit.py`, part of the parity audit set: fails when a
  retired fact (a test-binary count, "Git GPUI", a stable wasm32 build, the
  PowerShell-only lint gate) reappears in CI or agent guidance.
- `.shots/package_audit.py` fails when `rust-toolchain.toml`'s pin stops
  being a release of `rust-version` or clippy's `msrv` differs, the reason CI
  needs no separate MSRV job.
- `docs/upstream/gpui-block-future-incompat.md`: where the `block` 0.1.6
  future-incompatibility warning on macOS builds comes from (the GPUI macOS
  stack, and HeroGPUI's own `locale_config` dependency) and the upstream fix
  to request.

### Changed

- The `Cargo.toml`-profile (-O1/-O3) test job also runs on pull requests
  labelled `ci:opt-levels` or touching build configuration (`Cargo.toml`,
  `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, the CI Rust
  environment or the test wrapper); it stays push-only otherwise.
- The lint inheritance check now also covers `examples/*` and only accepts
  `workspace = true` inside `[lints]` (or top-level `lints.workspace`), not
  under any other table.

### Removed

- Four dead `cargo-deny` advisory ignores and the `allow-git` list from
  `deny.toml`; the workspace has no git dependencies, so any git source is
  now denied.

### Fixed

- Stale CI, release and agent guidance: test-binary counts, "Git GPUI is not
  registry-publishable", "stable wasm32 build", "unpublished" crates,
  "does not publish to crates.io", and advisory reasons naming `gpui 0.2`.

### Added

- `VirtualList` uniform mode: `VirtualListHandle::uniform(count)` lays every
  row out at the first row's measured height (GPUI's `uniform_list`), and
  without `.height(..)` the list sizes to its rows. `VirtualListScroll::Center`
  centres a row that is not fully visible, clamped at both ends (in measured
  mode, a row laid out in the last frame; any other falls back to `Top`).
  New handle readings for both modes: `viewport_bounds()`,
  `remaining_below()`, `is_scrolled_to_top()`, `is_scrolled_to_end()` and
  `is_uniform()`.
- `TreeView`: Shift range selection in `SelectionMode::Multiple` —
  Shift+Up/Down and a Shift press select the visible, enabled rows between
  the anchor (the last row a press or Space selected) and the target, as
  React Stately's `extendSelection` does and as `ListBox` already did;
  Shift+Home/End only move the cursor. Rows render through a uniform
  `VirtualList`, so under the new `max_h(px)` (or in a height-bounding
  parent) only the rows in view are built and the keyboard cursor scrolls
  into view; an uncapped tree keeps its geometry. Each row reports
  `aria-posinset`/`aria-setsize` among its siblings, as React Aria's
  `useGridListItem` does for tree rows. A "Large Tree View" gallery section
  on the List Box page shows two thousand open rows.
- `ResizablePanel`: pixel limits `min_size_px` / `max_size_px`, combined
  with the percentage limits (the stricter wins) against the group's
  measured length for the initial layout, drags, keys and window resizes;
  `collapsible` and `collapsed_size` (default 0), so a panel can sit below
  its minimum at its collapsed size — a drag past halfway collapses or
  expands it, an arrow key crosses the gap, Home/End reach it, and Enter on
  a handle collapses the panel before it (or after it, when only that one
  is collapsible) and restores its pre-collapse size, per the WAI-ARIA
  window splitter pattern. `ResizablePanelGroup::on_resize_end` fires once
  per gesture: on the release of a drag that changed the sizes, and after
  each key that did. A "Collapsible Panels" gallery section on the
  Separator page shows them.
- Icons: `IconSize` steps (`Xs` 12, `Sm` 14, `Md` 16, `Lg` 20, `Xl` 24px),
  which `Icon::size` takes beside a pixel length and through
  `Sizable<Size = IconSize>`; `Icon::stroke_width` (Lucide's `strokeWidth`,
  in the 24-unit viewBox; `DEFAULT_STROKE_WIDTH` is 2) and
  `Icon::absolute_stroke_width` (Lucide's `absoluteStrokeWidth`). Because
  gpui paints an SVG as one single-colour mask, a stroke width selects
  `IconName::path_with_stroke_width(w)` (`<path>?stroke-width=<w>`), which
  `HeroGpuiAssets` serves as the same Lucide file with its root
  `stroke-width` rewritten; gpui caches it per path and size. Lucide icons
  only.
- `.shots/lucide-icons.txt` lists the embedded Lucide icons and pins the
  `lucide-static` tarball (version and integrity);
  `python3 .shots/sync-lucide.py` downloads and verifies it, copies the
  listed icons and the license, and regenerates `IconName` and
  `LUCIDE_VERSION`. `--check` (offline, run in CI's parity job) fails when
  the list, the files, the enum and the NOTICE disagree.
- Gallery: an Icons page under Getting Started with the size steps, stroke
  widths and a searchable grid of every `IconName`.

### Changed

- **Breaking:** `VirtualListScroll` is `#[non_exhaustive]` (it gained
  `Center`); a `match` on it needs a wildcard arm. See
  [`docs/migration-0.13.md`](docs/migration-0.13.md).
- The fixed `row_height` bodies of `ListBox` and `Table` and ComboBox's
  popover list render through `VirtualList`'s uniform mode instead of calling
  `uniform_list` themselves. Behaviour is unchanged: the keyboard cursor is
  still centred, PageUp/PageDown still step by the declared row height over
  the laid-out viewport, and `Table`'s load-more still arms from the row
  count times the measured row.

Upgrading from 0.12: see the
[migration guide](https://github.com/Porabuild/HeroGPUI/blob/master/docs/migration-0.13.md)
for each breaking change with before/after code.

### Breaking

- `i18n::LOCALES` and `i18n::UiString::ALL` grow (13 locales, 30 keys), so
  code that named their array lengths no longer compiles; use
  `UiString::COUNT` or iterate. `NoResults`, `Loading` and `Search` now have
  translations in every built-in locale instead of falling back to en-US, so
  a de-DE app shows "Wird geladen" rather than "Loading" (override with
  `set_ui_string` to keep English).

- `ThemeBuilder::role`, `ThemeBuilder::role_hover` and `ThemeColors::role`
  take the typed `Color` role instead of a `&str`. An unknown string used to
  fall back to `accent`, so a typo silently recoloured the accent and the
  focus ring; it is now a compile error.
- `ThemeProvider::init` / `init_with` no longer read `HEROGPUI_REDUCE_MOTION`
  and no longer write GPUI's reduced-motion flag: the library reads no
  environment variable. Set the preference with `set_reduce_motion` from the
  application's own settings. The gallery maps `HEROGPUI_REDUCE_MOTION=1` onto
  it, so the gallery's behaviour is unchanged.

### Added

- `#[must_use]` on every component (each `IntoElement` builder) and on the
  builder data types (`ThemeBuilder`, `ComponentTheme(s)`, the `*Style`
  recipes, `Toast`, `TabItem`, `ListBoxItem`, `TableColumn`, `TableRow`,
  `TreeItem`, `ResizablePanel`, …): a builder chain whose result is dropped
  now warns (`unused_must_use`) instead of silently rendering nothing.

- Every public item in `herogpui`, `herogpui-core`, `herogpui-theme` and
  `herogpui-components` is documented, and those crates now
  `#![warn(missing_docs)]`, so CI's `clippy -D warnings` keeps it that way.

- `Color::from_token` and `impl FromStr for Color` (`UnknownColorError`):
  parse a role name from configuration, failing on an unknown name.
- i18n: four more built-in locales, ja-JP, zh-CN, ko-KR and ru-RU, and 21
  more `UiString` keys, so the remaining hard-coded chrome strings resolve
  in the active locale: the NumberField and TimeField stepper names
  (`Increase` / `Decrease`, templates in the locale's word order), the
  DateField / TimeField segment names (`Year` … `DayPeriod`), the
  DatePicker trigger (`Calendar`), a selected calendar day
  (`DateSelected`), the ColorSlider channel names (`Hue` … `Alpha`,
  `ColorChannel::localized_label`), Autocomplete's clear button
  (`ClearSelection`) and Pagination's name (`Pagination`). The translations
  come from the pinned React Aria / React Stately dictionaries.
  `i18n::ui_string_with` and `i18n::fill` fill a template's placeholder.
  The web gallery bundles a Noto Sans KR subset for Hangul and a wider
  Noto Sans SC pre-reduction for the Japanese kanji.
- Theme hot reload (HeroGPUI extension; new opt-in `watch` feature on
  `herogpui-theme` and `herogpui`, implies `serde`): `watch_themes_dir(dir,
  on_reload, cx)` polls a theme directory every `THEME_WATCH_INTERVAL`
  (500ms) and re-registers changed and new `*.json` files, so an edit to the
  active theme applies live; `ThemeReload` reports the reloaded ids and parse
  errors, and dropping the returned `ThemeWatcher` stops it. Polling on
  GPUI's executors keeps it dependency-free. The native gallery enables it
  for `HEROGPUI_THEME_DIR=<dir>`.

## [0.12.0] - 2026-09-28

### Added

- `MenuBar` and `MenuBarMenu` (HeroGPUI extension): a horizontal bar of
  Dropdown menus. One trigger is a tab stop and Left/Right rove between the
  top-level items (wrapping, skipping disabled ones), Home/End jump; a press,
  Enter, Space or Down opens a menu, the keyboard paths with the first item
  focused. While a menu is open, Left/Right or hovering another trigger
  switches menus; Escape, a pick, an outside press or pressing the open
  trigger closes it and returns the focus to its trigger. It reports
  `Role::MenuBar`, and `Role::MenuItem` with `expanded` on each trigger. A
  gallery section on the Dropdown page shows it.
- `ResizablePanelGroup` and `ResizablePanel` (HeroGPUI extension): split
  panels, horizontal or vertical, sized in percent of the space they share,
  with a drag handle between each pair. A drag captures the pointer and
  keeps resizing outside the handle; a focused handle moves with the arrow
  keys along the group's axis (Shift for four times `keyboard_step`) and
  Home/End jump to the limits. Only the two adjacent panels change, clamped
  to their `min_size`/`max_size`; `on_resize` reports every change, and
  `sizes(..)` makes the group controlled. Each handle is a tab stop that
  reports `Role::Splitter` with its orientation and a value range. A
  gallery section on the Separator page shows it.
- `TreeView` and `TreeItem` (HeroGPUI extension): a tree of expandable rows
  with one tab stop. Up/Down/Home/End move over the visible rows (skipping
  disabled ones), Right expands a parent and then enters its first child,
  Left collapses it and then moves to the parent, and typeahead finds a row;
  the chevron toggles a parent under the pointer. Selection follows
  `ListBox` (`None`, `Single`, `Multiple`; Escape clears), and expanded and
  selected keys are each controlled or uncontrolled. It reports
  `Role::Tree`, and `Role::TreeItem` with level, expanded and selected. A
  gallery section on the List Box page shows it.
- `Icon` and `IconName` (HeroGPUI extension): a public icon set of 245
  curated Lucide icons, copied verbatim from `lucide-static` 1.31.0 and
  embedded in `herogpui-components`, so `HeroGpuiAssets` serves them with no
  setup under `herogpui/icons/lucide/<name>.svg`. `IconName` is
  `#[non_exhaustive]` with `ALL`, `name`, `path`, `svg`, `from_name` and
  `from_path`, and converts into `SharedString`, so every builder that takes
  an icon path takes a name. `Icon::new(name).size(px).color(hsla)` draws
  one (16px in the theme foreground by default); `Icon::from_path` draws any
  other served SVG. The `icons` chrome constants are unchanged. Lucide's
  ISC license (with the Feather MIT notice) ships beside the SVGs and is
  attributed in `NOTICE`; `herogpui-components` therefore declares
  `license = "Apache-2.0 AND ISC AND MIT"`, the GitHub Release attaches the
  text as `LICENSE-lucide`, and the website serves it as
  `/LICENSE-lucide.txt`.
- Gallery: a theme picker in the navbar switches live between the light and
  dark bases and every built-in preset (`ocean`, `forest`, `midnight`,
  `rose`), natively and in the web build. A control request's `theme=` now
  selects that base theme by id, so a capture never keeps a preset.
- `ContextMenu` opens from the keyboard: Shift+F10 or the ContextMenu key
  (`menu`; `contextmenu` on the web) while the focus is inside the area opens
  the menu at the area's top-left corner with its first item focused, and
  closing it returns the focus to the element that held it. The area owns a
  focus handle outside the tab order, so a primary press on non-focusable
  content makes it reachable. The area element is now `position: relative`
  (it measures itself with an absolutely placed probe), so absolutely placed
  content inside it positions against the area.

### Fixed

- `HeroGpuiAssets` now embeds every `icons` chrome path. It was missing eight
  (`calendar`, `info_circle`, `check_circle`, `circle_exclamation`,
  `warning_triangle`, `loader_2`, `trash`, `gear`), so an app that registered
  it alone drew no DatePicker calendar glyph and no Alert, AlertDialog or
  Toast status icon; only the gallery's own asset source had them.

### Changed

- `ListBox` and `Table` render their `estimated_row_height` bodies through
  `VirtualList`, on the same `ListState` as before, so measurement, paging,
  load-more and scroll-into-view are unchanged. Their fixed `row_height`
  path and ComboBox's popover list stay on `uniform_list`: it centres the
  keyboard cursor, ListBox and Table page by the declared row height, and
  Table arms load-more from the uniform list's viewport and content size; see
  `docs/customisation-roadmap.md`.
- 34 `radius` source-text assertions in `tests/radius_builders.rs` (the
  builder-stores and render-resolves checks of 17 components) are replaced
  by a painted-scene test (`tests/radius_painted.rs`) that checks, for each
  of those components, that the default paints its owning helper's radius
  and that `.radius(..)` replaces it on the rendered quads.

## [0.11.0] - 2026-09-22

The parity target moves from HeroUI v3.2.5 to **HeroUI v3.2.6** (tag
`v3.2.6`, commit `e385ac20`). Inherited behavior now follows React Aria
3.52.1, React Aria Components 1.21.1 and React Stately 3.50.0. HeroUI 3.2.6
changed no theme tokens, so `herogpui-theme` is unchanged.

Upgrading from 0.10: see the
[migration guide](https://github.com/Porabuild/HeroGPUI/blob/master/docs/migration-0.11.md)
for each breaking change with before/after code.

### Breaking

- `Separator` no longer implements `ParentElement`: the content mode
  (`Separator::new().child("OR")`, a line, the content, a line) is removed.
  v3's `separator.tsx` renders a childless React Aria separator at both 3.2.5
  and 3.2.6, and 3.2.6 deleted the never-applied `.separator__container`,
  `__line` and `__content` rules the mode was modelled on. Place separators
  between blocks instead, as v3's "With Content" example does.
- Public API hardening:
  - `herogpui_components::util` is private. The helpers supported for custom
    widgets move to the new `extend` module (`herogpui::extend`): the radius
    scale (`control_radius`, `soft_radius`, `small_radius`, `mark_radius`,
    `key_radius`, `hairline_radius`, `micro_radius`, `field_radius`,
    `container_radius`), `FIELD_HEIGHT`/`FIELD_TEXT`/`FIELD_ICON`,
    `cursor_interactive`/`interactive_cursor`, `focus_visible`/
    `set_focus_visible`, `inner_fill_radius`, `shift_wheel_scroll_x` and
    `app_focus_root`. The render-prop payloads `FieldFocus`, `SelectionValue`
    and `InteractiveState` are re-exported at the crate root. Every other
    former `util` item (field chrome, overlay stack, focus-ring plumbing,
    `sx` extraction) is no longer public; the unused `prominence_bg`,
    `placed_panel`, `placed_field_panel` and `focusable` are deleted.
  - `pub use anim::*` is gone: motion tokens and wrappers are reached as
    `herogpui::anim::…` (`herogpui_components::anim::…`) instead of at the
    crate root.
  - `#[non_exhaustive]` on `Theme`, `ThemeColors`, `FieldColors`,
    `LayoutTheme`, `ComponentThemes`, `ComponentTheme`, `ButtonStyle`,
    `SliderStyle`, `SwitchStyle`, `SelectStyle`, `MenuStyle`,
    `TextFieldStyle`, `TabItem`, and every render-prop payload
    (`InteractiveState`, `FieldFocus`, `SelectionValue`,
    `AccordionItemState`, `CalendarCellState`, `RangeCalendarCellState`,
    `CheckboxState`, `SwitchState`, `RadioOptionState`,
    `ColorAreaThumbState`, `ColorSliderThumbState`,
    `ColorSwatchPickerItemState`, `ColorFieldRenderState`,
    `DateFieldRenderState`, `DatePickerRenderState`,
    `DateRangePickerRenderState`, `DisclosureRenderState`,
    `TextFieldRenderState`, `SearchFieldRenderState`,
    `NumberFieldRenderState`, `TimeFieldRenderState`). Outside the crates
    they can no longer be built with a struct literal (not even with
    `..Default::default()`); use their constructors, builders or `Default`
    and assign fields.
  - `use_theme` and `ThemeProvider::set_active` return
    `Result<(), UnknownThemeError>`. An unregistered id (ids are
    case-sensitive) is refused and leaves the active theme and every window
    untouched; it previously installed the id and panicked on the next frame.
  - `Modal::on_close` and `Drawer::on_close` receive `&DismissReason`
    (`CloseButton`, `Escape`, `Backdrop`, `Drag`) instead of a `&ClickEvent`,
    which Escape and backdrop dismissal used to fabricate with
    `ClickEvent::default()`. `modal::OnClose` changes accordingly.
- `herogpui::gpui` is now documented (it was `#[doc(hidden)]`) as the stable
  path to GPUI items, including the eight the HeroUI vocabulary shadows at
  the root. The root `use herogpui::*;` still carries all of GPUI.

### Added

- `AvatarGroup` and `AvatarGroupCount`, the v3.2.6 compound: `size`, `color`
  and `variant` flow to direct `Avatar` children that omit them, `max`
  truncates and appends an automatic `+N` count, an explicit count is never
  truncated and suppresses the automatic one, `is_grid` wraps with a 12px gap,
  and `overlap(AvatarGroupOverlap::{Clip,Ring})` selects the stacked seam.
  Upstream's `clip` crescent is a CSS alpha mask that pinned GPUI cannot draw;
  the port paints the seam in the surface colour instead (documented platform
  limitation).
- An AvatarGroup gallery page and reference metadata; the Avatar page's
  hand-built "Avatar Group" section moves there.
- HeroGPUI extensions (not HeroUI v3 APIs), adopted from gpui-kit patterns:
  - Shared builder traits `Disableable` (`is_disabled`), `Sizable`
    (`size`, with the component's own scale as `Sizable::Size`) and
    `Selectable` (`is_selected`), implemented by delegating to the existing
    inherent builders, which stay the documented API.
  - `herogpui_components::i18n`: a component chrome string catalogue with
    `set_locale`, `locale`, `ui_string`, `set_ui_string` and `lookup`. The
    Select/Autocomplete placeholder, Autocomplete's empty state, and the
    CloseButton, Spinner, calendar Previous/Next, Tag remove and Breadcrumbs
    names resolve through it. Built-in translations for de-DE, es-ES, fr-FR,
    it-IT, nl-NL, pl-PL, pt-BR and sv-SE come from the pinned React Aria
    dictionaries; en-US output is unchanged when no locale is set.
  - Theme files (`serde` feature): `register_theme_json`, `load_themes_dir`,
    `ThemeLoadError`, a checked-in JSON Schema (`THEME_SCHEMA`,
    `crates/herogpui-theme/theme.schema.json`) and four preset themes
    (`presets::PRESETS`, `presets::register_presets`: ocean, forest,
    midnight, rose). `ThemeProvider::insert` registers without activating;
    `ThemeProvider::theme_ids` lists the registry.
  - `herogpui::test` (`test-support` feature): `open_window` and the
    `TestWindowExt` helpers (`find`, `expect`, `click`, `click_at`, `hover`,
    `press`, `type_text`, `scroll`, `settle`) for downstream UI tests.
  - `VirtualList` / `VirtualListHandle` / `VirtualListScroll`: a
    variable-row-height virtual list with `scroll_to_item`, `splice` and
    `set_item_count`, with a "Virtual List" section on the ListBox page.
  - `ContextMenu`: the Dropdown `Menu` opened by a secondary press at the
    pointer, with a "Context Menu" section on the Dropdown page.
- `examples/` workspace crates (`hello-button`, `form`, `theme-switch`),
  `CONTRIBUTING.md` with a required `## Public API` pull request section, and
  `.github/pull_request_template.md`.

### Changed

- Internal: `Calendar` and `RangeCalendar` share one keyboard controller
  (`calendar_keys`) for grid navigation, visible-window paging and the year
  picker; `Table::render` is split into named part renderers (header cells,
  resize handle, virtual projection, body rows, row keyboard, load-more).
  Behavior is unchanged.
- Avatar: small fallback text is 12px/16px (`.avatar--sm .avatar__fallback`
  is `text-xs` in 3.2.6).
- Breadcrumbs: the root gaps items by 6px (`gap-1.5`), an item gaps its link
  and separator by 4px (`gap-1`), and neither the item nor the link carries
  padding any more.
- Autocomplete gallery "Custom Value" example follows v3.2.6's currency
  specimen: symbol, code and muted name in the trigger.
- The pinned docs bundle, CSS and demo archives, reference metadata source
  links, audits, inventory and guides move to v3.2.6. The web site pins
  `@heroui/react` 3.2.6, `react-aria` 3.52.1 and `react-aria-components`
  1.21.1, adds `@internationalized/date` (now a HeroUI peer) and drops the
  unused `@react-aria/i18n`.

### Fixed

- Theme documents (`serde` feature) reject a non-finite `oklch()` component
  (`NaN`, `inf`) with `ThemeDocumentError::Color`; `f32` parsing accepted
  them, so a theme file could register colours that are not numbers.
- Text editing (`Input`, `TextField`, `SearchField`, `TextArea`):
  Backspace, Delete and Left/Right (with or without Shift) step by extended
  grapheme cluster, so an emoji with a modifier, a ZWJ sequence, a flag or a
  letter with combining marks is one caret stop and one deletion instead of
  being split. Adds the `unicode-segmentation` dependency (MIT OR Apache-2.0,
  already in the graph through GPUI).
- Switch: the built-in label uses the shared `.label` style, 14px/20px medium.
  It previously painted 16px/24px from a `.switch__label` rule that v3 never
  applied (3.2.6 deleted it); this corrects a latent error, not a 3.2.6
  behavior change.

## [0.10.2] - 2026-09-22

### Security

- `Input` with `InputType::Password` no longer writes its value to the
  clipboard on Cmd/Ctrl+C or Cmd/Ctrl+X, and cut no longer deletes the
  selection, matching browser behavior for `type="password"`.

### Changed

- `gpui-pre` and `gpui-pre-platform` are pinned exactly at `=0.3.5`;
  `.shots/package_audit.py` rejects any non-exact requirement.
- The release workflow runs the full CI workflow on the tagged commit and
  builds, releases and publishes nothing unless it passes.

### Fixed

- Documentation: `herogpui` is on crates.io (`herogpui = "0.10"`); the
  retired setup script, `.vendor/`, git-hook and patch instructions are gone
  from the README; `gpui-pre` is correctly attributed to crates.io user
  huacnlee (Jason Lee, the gpui-kit maintainer), not zed-industries.

## [0.10.1] - 2026-09-18

### Added

- Tabs: per-segment `width`, `height` and `padding_x` overrides for
  icon-only toolbars, a tray-inset override, and `hover_fill(false)` to use a
  text-colour hover instead of the fill wash.

## [0.10.0] - 2026-09-17

### Breaking

- `MenuItem::Item` gained the `is_interactive` field and `MenuItem` is now
  `#[non_exhaustive]`; `MenuItem::new` and its builders are unchanged.

### Changed

- Menu separators default to HeroUI v3.2.5's proportional rule (3% inset per
  edge, 94% width); `MenuStyle::separator_inset`/`separator_thickness` pin
  pixels explicitly.

### Added

- Menu: `is_interactive` rows; Select trigger and option rows stop click
  propagation so hosted controls compose.
- Select: `SelectStyle::padding_y`, `Select::item_leading`.
- Tooltip: element bodies via `Tooltip::body`.
- Theme: `RoleColor::with_hover`, `ThemeBuilder::role_hover`/`accent_hover`,
  `roles.<role>.hover` in `ThemeDocument`.
- Facade: GPUI's `FluentBuilder` re-exported at the crate root.
- Button: `width`, `min_width`, `height`, `padding_x`, `text_size`,
  `font_weight`, `grow`.
- Spinner: `size_px` and a replaceable glyph.
- Tabs: icon-only `TabItem::trigger`, `radius`, `list_bg`, `indicator_bg`,
  `indicator_shadow`.
- Scrollbar: `track`, `inset`, `radius`, `thumb_color`, `thumb_hover_color`,
  `auto_hide(false)`, `sx`.
- ColorField: `on_blur` and built-in revert-on-blur of invalid text.

## [0.9.0] - 2026-09-16

### Added

- First crates.io release of `herogpui`, `herogpui-core`, `herogpui-theme`,
  `herogpui-components` and the `herogpui-gallery` CLI, built on the
  published `gpui-pre` 0.3.5 crates with no GPUI fork.

[Unreleased]: https://github.com/Porabuild/HeroGPUI/compare/v0.12.0...HEAD
[0.12.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.2...v0.11.0
[0.10.2]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/Porabuild/HeroGPUI/releases/tag/v0.9.0
