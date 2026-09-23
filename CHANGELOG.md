# Changelog

All notable changes to HeroGPUI are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). One
version covers `herogpui`, `herogpui-core`, `herogpui-theme`,
`herogpui-components` and `herogpui-gallery`.

## [Unreleased]

## [0.11.0] - 2026-09-22

The parity target moves from HeroUI v3.2.5 to **HeroUI v3.2.6** (tag
`v3.2.6`, commit `e385ac20`). Inherited behavior now follows React Aria
3.52.1, React Aria Components 1.21.1 and React Stately 3.50.0. HeroUI 3.2.6
changed no theme tokens, so `herogpui-theme` is unchanged.

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

[Unreleased]: https://github.com/Porabuild/HeroGPUI/compare/v0.11.0...HEAD
[0.11.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.2...v0.11.0
[0.10.2]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/Porabuild/HeroGPUI/releases/tag/v0.9.0
