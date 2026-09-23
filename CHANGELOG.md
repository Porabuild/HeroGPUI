# Changelog

All notable changes to HeroGPUI are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). One
version covers `herogpui`, `herogpui-core`, `herogpui-theme`,
`herogpui-components` and `herogpui-gallery`.

## [Unreleased]

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

[Unreleased]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.2...HEAD
[0.10.2]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/Porabuild/HeroGPUI/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/Porabuild/HeroGPUI/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/Porabuild/HeroGPUI/releases/tag/v0.9.0
