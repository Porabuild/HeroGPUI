# Integration test index

This indexes the ~96 integration test binaries under this directory by
feature area. Each `.rs` file here is its own `cargo test` binary. Unit tests
for internals (`#[cfg(test)]` modules) live alongside the implementation
under `src/` instead, and are not listed here.

## Buttons, toggles & choice controls

| Binary | Covers | Tests |
|---|---|---|
| `buttons` | Button, ButtonGroup, ToggleButton, CloseButton, Link, Chip-close and Alert-close press behavior. | 31 |
| `close_button_deep` | CloseButton geometry and press against pinned HeroUI v3.2.4 metrics. | 9 |
| `choice_controls_deep` | Deeper keyboard/read-only behaviour for Switch, RadioGroup and ToggleButtonGroup. | 48 |
| `checkbox_form_deep` | Checkbox keyboard, form, validation and focus contracts. | 22 |
| `checkbox_motion` | Checkbox selection motion (fill/scale/fade) contracts. | 8 |

## Text fields & forms

| Binary | Covers | Tests |
|---|---|---|
| `text_fields` | TextField, TextArea, SearchField and related text component behaviour. | 61 |
| `fields` | FIELD/FORM group components: checkbox/radio/switch groups, number field, form submit/validation. | 21 |
| `field_keyboard_contracts` | Keyboard/pointer contracts inherited by HeroUI v3's field family (search, date, time, number fields). | 20 |
| `field_slots_deep` | Focused slot tests for `field.rs` composition parts (label, error, fieldset). | 11 |
| `forms_deep` | Form, InputOTP and Fieldset edge cases: validation, server errors, submit/reset, autofocus. | 79 |
| `form_state_lifecycle` | Form reset callbacks must not retain the fields that store them (no leak). | 1 |
| `slider_form_deep` | Slider form integration: live values, disabled controls, reset. | 11 |
| `slider_number_deep` | Deep Slider/NumberField contracts inherited from pinned React Aria/Stately. | 18 |
| `text_size_knobs` | Phase 5 `text_size` knobs: builder stores the size and the label site reads it. | 1 |
| `font_seams` | Phase 5 field font seam: mono token default and explicit family override. | 1 |

## Pickers (select/combo/autocomplete/dropdown/list box)

| Binary | Covers | Tests |
|---|---|---|
| `pickers` | Behaviour for Autocomplete, ComboBox, Select search-and-select components. | 44 |
| `pickers_deep` | Pickers' remaining props and Drawer-adjacent picker behaviour. | 104 |
| `combo_box_open` | What opens a ComboBox's suggestion list (typing, focus, chevron, arrow, escape). | 14 |
| `combo_box_selection` | Single-selection callback parity with React Stately 3.50.0. | 4 |
| `autocomplete_hover_deep` | Pinned trigger-hover suppression contract around the clear button. | 8 |
| `select_clear_deep` | Select.ClearButton: real pointer and keyboard dispatch. | 5 |
| `dropdown_anatomy_deep` | Dropdown composition metrics, explicit identity, nested overlay behavior. | 4 |
| `dropdown_close` | Dropdown's close-on-activate contract, including submenus. | 25 |
| `dropdown_seed_deep` | Dropdown.Menu's controlled/uncontrolled selection seeding. | 12 |
| `dropdown_viewport_deep` | Dropdown menu viewport correction (flip/shift/clamp near edges). | 19 |
| `collections` | ListBox, Tabs, Accordion, Pagination, Breadcrumbs and TagGroup collection behaviour. | 16 |
| `collections_deep` | Deeper collection behaviour: ListBox `onAction`, TagGroup edge cases, row padding. | 7 |
| `collection_contracts` | Collection contracts inherited from pinned React Aria: ListBox/TagGroup/Table selection and keyboard. | 70 |
| `allocation_axes` | Keyed selection axis allocation probe (positive control for allocation tests). | 2 |

## Date & time

| Binary | Covers | Tests |
|---|---|---|
| `calendars_and_more` | Calendar/RangeCalendar family plus other un-driven behaviour (Select, ColorPicker, Toolbar, Disclosure). | 61 |
| `calendars_deep` | Edge dates and constraints on the calendar family: bounds, year picker, unavailable dates. | 64 |
| `date_field_form_deep` | Date/time field form submission, reset and disabled/read-only successful-control rules. | 11 |
| `date_field_picker_deep` | Deep inherited contracts for the v3 date field and picker family (segments, validation, value builders). | 45 |
| `date_picker_close` | Date pickers' close-on-select rule. | 10 |
| `date_picker_placement` | Placement contracts for the DatePicker family. | 9 |
| `overlay_stack_date_deep` | Explicit overlay-stack contracts for DatePicker and DateRangePicker. | 8 |

## Colour

| Binary | Covers | Tests |
|---|---|---|
| `color_geometry_deep` | Deep pointer-geometry tests for ColorArea and ColorSlider (drag, keyboard, form, fields). | 44 |
| `overlay_stack_color_deep` | ColorPicker overlay-stack behavior against the pinned React Aria contract. | 10 |
| `theme_tokens` | Customisation tokens reach their consumers. | 2 |

## Overlays (modal/drawer/popover/toast/tooltip)

| Binary | Covers | Tests |
|---|---|---|
| `overlays` | Modal, Drawer, AlertDialog and related overlay component behaviour. | 35 |
| `overlay_padding` | Overlay panel padding measured on the resting panel. | 3 |
| `panel_padding` | Phase 3 overlay panel padding: unset override keeps v3 stock insets. | 1 |
| `overlay_stack_dialogs_deep` | Explicit overlay-stack behavior for Modal, Drawer, AlertDialog. | 4 |
| `overlay_stack_menus_deep` | Nested Dropdown and Tooltip overlay-stack contracts. | 3 |
| `overlay_stack_pickers_deep` | Explicit overlay-stack coverage for the three picker surfaces (Select/DatePicker/ColorPicker family). | 10 |
| `popover_stack_deep` | Deep focus-scope and nested-dismissal contracts for Popover. | 15 |
| `drawer_deep` | Drawer anatomy: drag-to-dismiss, handle/header/footer, focus trap. | 15 |
| `feedback` | Toast, Alert, Avatar, Badge, Progress, Meter, Spinner, Skeleton behaviour. | 35 |
| `feedback_compose` | Three feedback/composition parity gaps (avatar load, alert composed button, badge default color). | 4 |
| `toast_promise_deep` | `toast.promise`: loading toast updates in place. | 5 |
| `toast_stack_deep` | Toast stack: `isExpanded`, hover/focus expand, timer pause. | 9 |
| `toast_update_deep` | `toast.update`: in-place content replacement. | 5 |
| `placement` | Positional behaviour: surface placement, arrow, collision handling. | 19 |
| `placement_extra` | Tooltip delays and the slider's other axis. | 10 |

## Navigation (tabs/breadcrumbs/pagination/link)

| Binary | Covers | Tests |
|---|---|---|
| `nav_deep` | Navigation family keyboard and edge cases (tabs, breadcrumbs, pagination, links). | 44 |
| `tabs_deep` | Deeper Tabs keyboard behaviour: vertical axis and the ends. | 28 |
| `parts_pagination` | Per-part disabling added to Pagination. | 8 |
| `link_deep` | Deep behaviour tests for `Link` against pinned HeroUI v3.2.4 Link contract. | 10 |
| `focus_visible_deep` | Focus rings follow keyboard-control modality, not HeroUI's Escape restore. | 7 |

## Data display (table/avatar/badge/chip/skeleton/progress)

| Binary | Covers | Tests |
|---|---|---|
| `table_and_drag` | Table and drag-driven controls (Slider, etc.) behaviour. | 31 |
| `table_deep` | Deeper Table behaviour not covered by sorting/selection/resize suites. | 42 |
| `table_tabs_accordion` | Three surfaces (Table, Tabs, Accordion) left half-driven by earlier suites. | 20 |
| `avatar_deep` | Avatar runtime behavior against pinned v3.2.4 contract (load/error/fallback lifecycle). | 21 |
| `badge_parts` | Badge part-composition: `BadgeAnchor`, `Badge`, `BadgeLabel`. | 6 |
| `chip_deep` | Headless coverage for the measurable half of `Chip`. | 5 |
| `alert_deep` | Alert anatomy and geometry against pinned v3.2.4 `alert.css`/`alert.tsx`. | 12 |
| `card_deep` | Card anatomy and geometry against pinned v3.2.4 `card.css`. | 6 |
| `kbd_deep` | Kbd anatomy and geometry against pinned v3.2.4 `kbd.css`. | 5 |
| `tag_geometry_deep` | Tag line boxes match v3.2.4 outside the gallery's default text root. | 4 |
| `parts_thumbs` | `Slider.Thumb` and `ColorSwatchPicker.Item` per-part state. | 16 |
| `slider_geometry_deep` | Slider paint geometry: the 12px start/end border inset. | 13 |

## Layout & surfaces

| Binary | Covers | Tests |
|---|---|---|
| `surface_deep` | Headless coverage for the measurable half of `Surface`. | 4 |
| `scroll_shadow_deep` | ScrollShadow paths not exercised by the vertical-scroll suite. | 3 |
| `toolbar_divider_deep` | Headless coverage for the divider a `Toolbar` draws between its groups. | 4 |
| `full_width` | Phase 6 `full_width` seams: containers store the flag and expand their root. | 1 |
| `size_enums_deep` | Additive size enums step the pinned geometry they own. | 4 |
| `radius_builders` | Per-component `radius` builders under the narrow parity exception. | 12 |
| `typography_deep` | Typography and Prose against pinned v3.2.4 `typography.css`. | 9 |

## Accessibility

| Binary | Covers | Tests |
|---|---|---|
| `a11y_deep` | Baseline accessibility contract: AccessKit tree is not observable headlessly; keyed-instance separation. | 6 |
| `a11y_overlays_deep` | Wave 2: accordion/disclosure/popover/dropdown/toast accessibility behaviour. | 6 |
| `a11y_collections_deep` | Wave 3: list box/tabs/toolbar/breadcrumbs/pagination/tag group/table/autocomplete accessibility behaviour. | 10 |
| `a11y_pickers_deep` | Wave 5: calendar/date field/time field/date picker/color picker accessibility behaviour. | 7 |

## Theming & tokens

| Binary | Covers | Tests |
|---|---|---|
| `component_themes` | Live `Theme.components` resolution: stock, defaults, recipes, instance overrides. | 11 |
| `theme_platform` | Platform integration of the theme provider: OS appearance and GPUI window theme. | 4 |
| `theme_repaint` | Theme provider repaint: global theme mutations must redraw every open window. | 1 |
| `cursor_token` | The interactive cursor is a theme token, not a hard-coded call (source-shape check). | 2 |
| `hover_overrides` | Phase 2 hover overrides: every owner resolves the named fill and its source shape. | 1 |
| `hover_repaint` | A pointer crossing must repaint the view for every control whose hover state affects it. | 3 |
| `sx_ownership` | The part-scoped `sx` ownership inventory (source-shape check). | 4 |
| `sx_radius_parts` | Per-corner `sx` reconciliation for painted parts. | 1 |
| `sx_slot` | The `sx` slot: the one caller-owned styling slot every component exposes. | 4 |

## Animation & motion

| Binary | Covers | Tests |
|---|---|---|
| `interaction` | Interaction tests driving controls (not just observing them). | 3 |

## Source-shape / parity contracts

| Binary | Covers | Tests |
|---|---|---|
| `migration_extensions` | Native application composition: seek gestures, compact pickers, standalone menus. | 7 |

## Render-prop / value-prop contracts

| Binary | Covers | Tests |
|---|---|---|
| `render_props` | Collection render-prop closures — the inverted per-row rendering API. | 39 |
| `value_props` | Inverted value/output render props: the closures a caller supplies for value text. | 42 |
| `virtual_and_feedback` | Paths a screenshot cannot see: virtualized lists/tables and related feedback. | 29 |

## Test harness / support

| Binary | Covers | Tests |
|---|---|---|
| `source_scan/mod.rs` | Shared helper module (not a standalone test binary): finds a named consumer's enclosing function and asserts a reader/call appears inside it, for source-shape wiring tests. | 0 |

## How to run

Run one binary:

```
cargo test -p herogpui-components --test <name>
```

Filter by test-name substring within a binary:

```
cargo test -p herogpui-components --test <name> <substring>
```

Run the full suite:

```
cargo test -p herogpui-components
```

Local workaround: on this machine, an Xcode license/toolchain issue sometimes
requires pointing `DEVELOPER_DIR` at the command-line tools before running
tests. This is a local workaround, not a repository requirement:

```
DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo test -p herogpui-components
```

## Conventions

- **Source-shape / contract tests.** Several binaries (`cursor_token`,
  `hover_overrides`, `sx_ownership`, `sx_radius_parts`) read implementation
  source with `include_str!` (or the `source_scan` helper's
  `component_src`) and assert with `source.contains(...)` / `source.matches(...)`
  that a call site is wired the way the contract requires — they check
  wiring, not painted pixels.
- **`source_scan` helper.** `tests/source_scan/mod.rs` is a shared module
  (`#[allow(dead_code)]`, no tests of its own) that reads a file from
  `src/` and asserts a given call appears inside the enclosing top-level
  function of a named consumer, so a sibling function's call cannot
  satisfy the check. `tests/sx_ownership.rs` documents the fixtures it
  relies on.
- **`#[gpui::test]` render/behavior tests.** Most binaries use
  `#[gpui::test]` with a `TestAppContext`/`VisualTestContext` to mount a
  component, drive it with simulated pointer/keyboard input (including
  focus and typeahead), then assert on reported state or painted
  `debug_bounds` geometry — the headless platform cannot observe drawn
  colors/shadows or the AccessKit tree (see `a11y_deep.rs`), only layout
  and callback-reported state.
- **`test-support` feature.** `gpui`'s own `test-support` feature is
  enabled only under `[dev-dependencies]` in
  `crates/herogpui-components/Cargo.toml`; it is what supplies the
  headless platform, `VisualTestContext`, and simulated input.
  `herogpui-components` itself defines no `test-support` feature — its
  only feature is `gallery-source`, unrelated to testing.
