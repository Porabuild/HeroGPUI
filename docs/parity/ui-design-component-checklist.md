# HeroGPUI component implementation checklist

Prepared 2026-09-12 for HeroUI React v3.2.5 against HeroGPUI commit `cdc93ba4d4686c68cc6e4a2e1923b6c36ddb2127`; source links re-pointed to v3.2.6 on 2026-09-22, when AvatarGroup gained its own entry.

This is the component appendix to the [execution plan](ui-design-plan.md). Apply its source, coverage, interaction, visual, motion, customization, documentation and native/WASM requirements to every entry. No entry below is a passing parity verdict.

Each entry supplies an explicit starting configuration/state matrix and review work. Expand it from the tagged implementation, CSS, inherited behavior and upstream stories/tests before execution. “Every supported placement/channel/type” means enumerate the exact pinned type into finite specimen records; it is not permission to sample one option. Both light/dark themes, relevant combined states, normal/reduced motion, pointer/keyboard paths, controlled/default ownership and customization stress cases apply wherever meaningful.

Parts, API-axis snapshots and gallery section names below were read from the current generated datasets. They are discovery aids and must be corrected from tagged source where stale. An omitted state table does not mean a component has no dynamic states; an existing example does not prove it works. Existing tests listed below are entry points, not a claim that they cover every listed case.

Companion coverage is explicit: **ToggleButtonGroup** under ToggleButton; **DisclosureGroup** under Disclosure; **Label**, **Description**, **ErrorMessage** and **FieldError** under Label & Messages. Composition-only structures such as Tag, Radio, SwitchGroup and calendar year-picker parts are covered with their owners. Keep the existing combined routes and provide dedicated anchors/examples for discoverability.

For each component, synchronize the implementation, focused tests, listed gallery source, matching reference metadata, `llms.txt`/Rust docs, generated website datasets and rebuilt WASM/manifests before accepting its slice. Use the existing gallery headings as a baseline and fill missing state/composition examples; do not delete customization examples merely to make the page resemble upstream.

**Snapshot fingerprints**

- `web/src/data/catalog.json`: `99816a71d82bb9d62f9b86bf6f61ec67566cd63e478735f22dd5ba5c97784291`.
- `web/src/data/reference.json`: `3599672da3a3037571f6d1a8621218afc52b807de524b26dc4b12f760b991c9d`.
- `web/src/data/rust-examples.json`: `bbcfd3263b6b7f10054968ce50b3cb32b53cc60de75021df0cc8cfd6ea93f2f8`.

If one of these inputs changes, refresh affected inventory records and revalidate this checklist against source. Do not treat the fingerprints as runtime proof.

## Shared finite axes and source gaps

| Axis | Values and required treatment |
|---|---|
| Semantic roles | Default, Accent, Success, Warning, Danger. Apply only to components that expose a semantic color/status; Primary and Secondary are variants, not color roles. Spinner has Current instead of Default. |
| Common sizes | Sm, Md, Lg where upstream declares them. Spinner adds Xl; swatches use Xs/Sm/Md/Lg/Xl. Components without an upstream size prop retain their exact default; native size extensions get separate cases. |
| Field variants | Primary and Secondary, including both inside each documented Surface composition. |
| Container prominence | Transparent, Default, Secondary, Tertiary for Card/Surface; Separator omits Transparent. |
| Upstream popup placement vocabulary | `bottom`, `bottom left`, `bottom right`, `bottom start`, `bottom end`; `top`, `top left`, `top right`, `top start`, `top end`; `left`, `left top`, `left bottom`, `start`, `start top`, `start bottom`; `right`, `right top`, `right bottom`, `end`, `end top`, `end bottom`. Reconcile the exact accepted type for each owner before generating cases. Physical and logical aliases share pixels only when direction permits it. |
| Current port placement gap | Closed: shared `Placement` now enumerates the full 22-value React Aria union — every side in both physical and logical spellings — and `TooltipPlacement` is an alias of it. The port stays LTR-only, so each logical start/end spelling resolves to the same pixels as its left/right twin; direction-aware RTL behavior remains outside the port and any owner that cannot accept a spelling must still document its own limit. |
| Color spaces/channels | HSB: Hue/Saturation/Brightness; HSL: Hue/Saturation/Lightness; RGB: Red/Green/Blue; Alpha for controls whose upstream type admits it. ColorArea must enumerate valid ordered distinct channel pairs and preserve the remaining channel. Include reversed axes; do not assume alpha is valid for every area. |
| Current native InputType | Text, Password, Email, Number, Tel, Url, Search. Reconcile other upstream string types and browser-only hints without advertising unsupported native widgets. |
| Direction and context | LTR/RTL where supported; page/default/secondary/tertiary surfaces; narrow and wide constraints; normal/reduced motion; pointer and keyboard focus modality. Scope unsupported directions explicitly instead of treating a physical alias as logical parity. |

Use the component-specific lists below for selection, backdrop, scroll, time/date formatting, animation and typography values. Do not apply a shared vocabulary to an owner that does not expose it.

## Buttons

### Button (`button`)

- **Implementation:** [button.rs](../../crates/herogpui-components/src/button.rs).
- **Gallery / reference:** [buttons.rs](../../gallery/src/pages/components/buttons.rs) / [metadata](../../gallery/src/pages/reference_metadata/buttons.rs).
- **Focused tests to read/extend:** [buttons](../../crates/herogpui-components/tests/buttons.rs), [hover_overrides](../../crates/herogpui-components/tests/hover_overrides.rs), [focus_visible_deep](../../crates/herogpui-components/tests/focus_visible_deep.rs), [radius_builders](../../crates/herogpui-components/tests/radius_builders.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/button/button.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Button.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/button.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28buttons%29/button.mdx).

**Required configuration and variant cases:** Primary, Secondary, Tertiary, Outline, Ghost, Danger and DangerSoft; Sm/Md/Lg; text, leading/trailing icons, icon-only, full width, custom children and label composition. The baseline omission of danger-soft from the API metadata union is corrected; its variant/state matrix still requires verification.

**Required state and transition cases:** Rest, hover, held press, release/cancel, keyboard focus and activation, disabled and pending. Exercise pending entry/completion without label-width shift; ensure pointer and keyboard activation emit once, and pending semantics follow the pinned behavior rather than a blanket disabled shortcut.

**Review and implementation work:** Match each variant’s own fill, foreground, border, shadow, radius and icon treatment. Complete press/focus interpolation where missing, including fixed-size custom children. Preserve intrinsic layout and existing sx/hover_bg/radius precedence; ensure custom hover leave returns to the resolved resting color.

- **Anatomy to account for:** `Button`.
- **Current metadata axes to reconcile:** `Button.variant` = 'primary' | 'secondary' | 'tertiary' | 'outline' | 'ghost' | 'danger' (default: 'primary'); `Button.size` = 'sm' | 'md' | 'lg' (default: 'md').
- **Current named state rows to retain/revalidate:** Hovered, Pressed, Focus visible, Disabled, Pending.
- **Existing gallery sections to map into specimens:** Usage; Variants; Sizes; With Icons; Icon Only; Loading; Loading State; Full Width; Disabled State; Social Buttons; Adding custom variants; Press handler; Sx slot; Custom hover fill.

### Button Group (`button-group`)

- **Implementation:** [button_group.rs](../../crates/herogpui-components/src/button_group.rs), [button.rs](../../crates/herogpui-components/src/button.rs).
- **Gallery / reference:** [buttons.rs](../../gallery/src/pages/components/buttons.rs) / [metadata](../../gallery/src/pages/reference_metadata/buttons.rs).
- **Focused tests to read/extend:** [buttons](../../crates/herogpui-components/tests/buttons.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs), [radius_builders](../../crates/herogpui-components/tests/radius_builders.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/button-group/button-group.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/tests/components/button-group/button-group.test.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/button-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28buttons%29/button-group.mdx).

**Required configuration and variant cases:** Every Button variant, Sm/Md/Lg, horizontal/vertical, attached/default grouping, full-width and intrinsic-width groups, with/without separators, mixed content, disabled group and member overrides.

**Required state and transition cases:** Focus each edge and middle member, hover/press, keyboard activation, whole-group disabled and individual disabled. Verify inherited defaults and explicit member overrides; use only keyboard navigation owned by the upstream group contract.

**Review and implementation work:** Match shared seams, outer corner ownership, separator color, overlapping focus-ring visibility and equal sizing. Grouped Buttons suppress standalone pressed scale; do not spread standalone motion into merged edges. Custom radii and sizing must not expose gaps.

- **Anatomy to account for:** `ButtonGroup`, `ButtonGroup.Separator`.
- **Current metadata axes to reconcile:** `ButtonGroup.variant` = 'primary' | 'secondary' | 'tertiary' | 'outline' | 'ghost' | 'danger' | 'danger-soft' (default: —); `ButtonGroup.size` = 'sm' | 'md' | 'lg' (default: —); `ButtonGroup.orientation` = horizontal | vertical (default: horizontal).
- **Current named state rows to retain/revalidate:** Horizontal, Vertical, Full width, Disabled, Pressed, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; Merged; Sizes; With Icons; Variants; Orientation; Full Width; Without Separator; Disabled State.

### Close Button (`close-button`)

- **Implementation:** [close_button.rs](../../crates/herogpui-components/src/close_button.rs).
- **Gallery / reference:** [buttons.rs](../../gallery/src/pages/components/buttons.rs) / [metadata](../../gallery/src/pages/reference_metadata/buttons.rs).
- **Focused tests to read/extend:** [close_button_deep](../../crates/herogpui-components/tests/close_button_deep.rs), [hover_overrides](../../crates/herogpui-components/tests/hover_overrides.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/close-button/close-button.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/close-button.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28buttons%29/close-button.mdx).

**Required configuration and variant cases:** Default variant; standard glyph, custom icon, custom subtree and render function; standalone and composed in every overlay/tag context. Do not invent a pending prop from the stylesheet alone.

**Required state and transition cases:** Hover, press held, release/cancel, disabled, pointer focus and keyboard-visible focus; exact-once close action. Distinguish glyph, visual box and expanded interaction target.

**Review and implementation work:** Complete scale and state transitions while keeping fixed child content consistent. Compare focus ring, icon size, centering and hit area independently; prevent layout shift and neighbor activation.

- **Anatomy to account for:** `CloseButton`, `CloseButton.Icon`.
- **Current metadata axes to reconcile:** `CloseButton.variant` = 'default' (default: 'default').
- **Current named state rows to retain/revalidate:** Hovered, Pressed, Focus visible, Disabled, Pending.
- **Existing gallery sections to map into specimens:** Usage; Interactive; With Custom Icon; Hover Colour; Render Function; Disabled.

### Toggle Button (`toggle-button`)

- **Implementation:** [toggle_button.rs](../../crates/herogpui-components/src/toggle_button.rs).
- **Gallery / reference:** [buttons.rs](../../gallery/src/pages/components/buttons.rs) / [metadata](../../gallery/src/pages/reference_metadata/buttons.rs).
- **Focused tests to read/extend:** [buttons](../../crates/herogpui-components/tests/buttons.rs), [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs), [hover_overrides](../../crates/herogpui-components/tests/hover_overrides.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/toggle-button/toggle-button.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/toggle-button-group/toggle-button-group.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/toggle-button.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/toggle-button-group.css) · [docs 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28buttons%29/toggle-button.mdx) · [docs 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28buttons%29/toggle-button-group.mdx).

**Required configuration and variant cases:** ToggleButton: Default/Ghost × Sm/Md/Lg; text/icon-only/custom content. ToggleButtonGroup is a separate required component on this page: single/multiple selection, horizontal/vertical, attached/detached, full width, separators, empty-selection policy and controlled/default keys.

**Required state and transition cases:** ToggleButton: off/on, hover, press, focus, disabled and controlled rejection. ToggleButtonGroup: none/one/many selected as allowed, disabled root/member, directional roving, focus restoration, member removal and disallow-empty selection. Verify selected+hover and selected+press for each variant.

**Review and implementation work:** Complete selected/unselected color and press interpolation. Match attached corner/separator ownership, inset focus treatment and currentColor-derived separators; validate detached spacing separately. Document and link ToggleButtonGroup independently within this route.

- **Anatomy to account for:** `ToggleButton`, `ToggleButtonGroup`, `ToggleButtonGroup.Separator`.
- **Current metadata axes to reconcile:** `ToggleButton.variant` = 'default' | 'ghost' (default: 'default'); `ToggleButton.size` = 'sm' | 'md' | 'lg' (default: 'md'); `ToggleButtonGroup.selectionMode` = "single" | "multiple" (default: "single"); `ToggleButtonGroup.orientation` = "horizontal" | "vertical" (default: "horizontal"); `ToggleButtonGroup.size` = "sm" | "md" | "lg" (default: "md").
- **Current named state rows to retain/revalidate:** Selected, Hovered, Pressed, Focus visible, Disabled, Controlled selection, Uncontrolled selection, Single selection, Multiple selection, Disallow empty selection, Horizontal group, Vertical group, Full width group, Attached group, Detached group, Group pressed, Group focus visible.
- **Existing gallery sections to map into specimens:** Usage; Hover Colour; Sizes; Icon Only; Disabled; Controlled; Single selection; Multiple selection; Variants; Orientation; Full Width; Without Separator; Selection Mode; Default Selected Keys; Vertical; Detached.

## Collections

### Dropdown (`dropdown`)

- **Implementation:** [dropdown.rs](../../crates/herogpui-components/src/dropdown.rs), [list_nav.rs](../../crates/herogpui-components/src/list_nav.rs).
- **Gallery / reference:** [collections.rs](../../gallery/src/pages/components/collections.rs) / [metadata](../../gallery/src/pages/reference_metadata/collections.rs).
- **Focused tests to read/extend:** [dropdown_anatomy_deep](../../crates/herogpui-components/tests/dropdown_anatomy_deep.rs), [dropdown_close](../../crates/herogpui-components/tests/dropdown_close.rs), [dropdown_seed_deep](../../crates/herogpui-components/tests/dropdown_seed_deep.rs), [dropdown_viewport_deep](../../crates/herogpui-components/tests/dropdown_viewport_deep.rs), [overlay_stack_menus_deep](../../crates/herogpui-components/tests/overlay_stack_menus_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/dropdown/dropdown.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/dropdown.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28collections%29/dropdown.mdx).

**Required configuration and variant cases:** Press/long-press triggers; every supported popup placement/alignment; none/single/multiple menu selection and section-level selection; Default/Danger items; checkmark/dot/custom indicators; icons, descriptions, shortcuts, sections, submenu triggers and custom triggers.

**Required state and transition cases:** Closed/opening/open/closing, hover/focus/press/selected/disabled items; open by pointer/keyboard, long-press threshold and cancellation, typeahead, arrow/Home/End/Page navigation, submenu enter/leave and nested dismissal. Check keyboard selection does not reopen the trigger and exiting menus do not intercept a new dialog.

**Review and implementation work:** Match menu padding, item text hierarchy, selected indicator and focus layer, submenu glyph and placement motion. Verify full trigger+submenu bounds rather than a union bounding rectangle, short viewports, scrolled parents, flipping and final option reachability.

- **Anatomy to account for:** `Dropdown`, `Dropdown.Trigger`, `Dropdown.Popover`, `Dropdown.Menu`, `Dropdown.Section`, `Dropdown.Item`, `Dropdown.ItemIndicator`, `Dropdown.SubmenuIndicator`, `Dropdown.SubmenuTrigger`.
- **Current metadata axes to reconcile:** `Dropdown.trigger` = "press" | "longPress" (default: "press"); `Dropdown.Popover.placement` = PopoverPlacement (default: "bottom"); `Dropdown.Menu.selectionMode` = "single" | "multiple" | "none" (default: "none"); `Dropdown.Section.selectionMode` = "single" | "multiple" (default: —); `Dropdown.Item.variant` = "default" | "danger" (default: "default"); `Dropdown.ItemIndicator.type` = "checkmark" | "dot" (default: "checkmark").
- **Current named state rows to retain/revalidate:** Hovered, Focused, Focus visible, Pressed, Disabled, Selected, Entering, Exiting.
- **Existing gallery sections to map into specimens:** Usage; Row Hover; Standalone Compact Menu; With Icons; With Descriptions; With Disabled Items; With Sections; Controlled; Controlled Open State; With Single Selection; Single With Custom Indicator; Render Props; With Section Level Selection; With Keyboard Shortcuts; With Submenus; With Custom Submenu Indicator; Custom Trigger; Long Press Trigger; Basic Usage; Controlled Selection; With Multiple Selection.

### List Box (`list-box`)

- **Implementation:** [list_box.rs](../../crates/herogpui-components/src/list_box.rs), [list_nav.rs](../../crates/herogpui-components/src/list_nav.rs), [selection.rs](../../crates/herogpui-components/src/selection.rs).
- **Gallery / reference:** [collections.rs](../../gallery/src/pages/components/collections.rs) / [metadata](../../gallery/src/pages/reference_metadata/collections.rs).
- **Focused tests to read/extend:** [collections](../../crates/herogpui-components/tests/collections.rs), [collections_deep](../../crates/herogpui-components/tests/collections_deep.rs), [collection_contracts](../../crates/herogpui-components/tests/collection_contracts.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/list-box/list-box.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/list-box-item/list-box-item.tsx) · [API/source 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/list-box-section/list-box-section.tsx) · [API/source 4](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ListBox.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/list-box.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/list-box-item.css) · [styles 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/list-box-section.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28collections%29/list-box.mdx).

**Required configuration and variant cases:** Default/Danger item treatments; none/single/multiple selection, toggle/replace behavior, empty-selection rules, sections/descriptions/icons/custom indicator, finite/virtualized and controlled/default keys.

**Required state and transition cases:** Unselected/selected/hovered/pressed/focused/focus-visible/disabled, empty collection, all-disabled and dynamically removed items. Exercise typeahead, directional/page navigation, modifiers, escape behavior, focus wrapping and scrolling to a selected/focused off-screen item where supported.

**Review and implementation work:** Separate selected, hover and focus fills according to source precedence. The selected/custom indicator now uses the pinned absolute inline-end slot with reserved end padding; remaining review covers row heights, section gaps, label/description alignment and virtual row geometry. Prevent intrinsic demo width from concealing full-width and narrow-layout failures.

- **Anatomy to account for:** `ListBox`, `ListBox.Item`, `ListBox.ItemIndicator`, `ListBox.Section`.
- **Current metadata axes to reconcile:** `ListBox.selectionMode` = "none" | "single" | "multiple" (default: "none"); `ListBox.variant` = "default" | "danger" (default: "default"); `ListBox.selectionBehavior` = "toggle" | "replace" (default: "toggle"); `ListBox.Item.variant` = "default" | "danger" (default: "default").
- **Current named state rows to retain/revalidate:** Selected, Focused, Focus visible, Pressed, Hovered, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Row Padding; With Disabled Items; With Sections; Multi Select; Controlled; Disallow Empty Selection; Escape Key Behavior; Virtualization; Custom Check Icon; Single selection; Multiple selection; Basic Usage; Controlled Selection; Custom Indicator.

### Tag Group (`tag-group`)

- **Implementation:** [tag_group.rs](../../crates/herogpui-components/src/tag_group.rs), [selection.rs](../../crates/herogpui-components/src/selection.rs).
- **Gallery / reference:** [collections.rs](../../gallery/src/pages/components/collections.rs) / [metadata](../../gallery/src/pages/reference_metadata/collections.rs).
- **Focused tests to read/extend:** [tag_geometry_deep](../../crates/herogpui-components/tests/tag_geometry_deep.rs), [collection_contracts](../../crates/herogpui-components/tests/collection_contracts.rs), [collections_deep](../../crates/herogpui-components/tests/collections_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/tag-group/tag-group.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/tag/tag.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/TagGroup.tsx) · [API/source 4](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria/src/tag/useTagGroup.ts) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/tag-group.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/tag.css) · [styles 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/close-button.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28collections%29/tag-group.mdx).

**Required configuration and variant cases:** TagGroup Default/Surface × Sm/Md/Lg; none/single/multiple and toggle/replace selection; plain/prefix/custom/removable tags; wrapped rows, controlled keys and list mutation. Tag and Tag.RemoveButton each need their own part cases.

**Required state and transition cases:** Selected/unselected, hover/press, keyboard focus, disabled group/tag/key, removal by pointer and keyboard, last-tag removal, selection isolation, disallow-empty and Escape behavior. Repeat removal in multiple Select, Autocomplete and ComboBox.

**Review and implementation work:** Preserve the existing expanded removal target fix: 24px interaction region around a 12px visual glyph without changing layout. Complete remaining remove-part composition, description/error presentation and neighbor target/focus checks. Reuse existing evidence; do not repeat the fixed hit-area implementation.

- **Anatomy to account for:** `TagGroup`, `TagGroup.List`, `Tag`, `Tag.RemoveButton`.
- **Current metadata axes to reconcile:** `TagGroup.selectionMode` = "none" | "single" | "multiple" (default: "none"); `TagGroup.size` = "sm" | "md" | "lg" (default: "md"); `TagGroup.variant` = "default" | "surface" (default: "default"); `TagGroup.selectionBehavior` = "toggle" | "replace" (default: "toggle").
- **Current named state rows to retain/revalidate:** Selected, Disabled, Hovered, Pressed, Focused, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; Hover Colour; Disabled; Selection Modes; Controlled; Disallow Empty Selection; Escape Key Behavior; With Error Message; With List Data; With Prefix; With Remove Button; Removable; Selectable; Sizes; Variants.

## Colors

### Color Area (`color-area`)

- **Implementation:** [color_picker/area.rs](../../crates/herogpui-components/src/color_picker/area.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [color_geometry_deep](../../crates/herogpui-components/tests/color_geometry_deep.rs), [parts_thumbs](../../crates/herogpui-components/tests/parts_thumbs.rs), [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-area/color-area.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorArea.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorThumb.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-area.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-area.mdx).

**Required configuration and variant cases:** Every valid x/y channel pair in supported RGB/HSL/HSB spaces; dots on/off; controlled/default color, custom thumb and non-square dimensions. Distinguish color-space channels from arbitrary CSS dimensions.

**Required state and transition cases:** Rest, hover, focus-visible, dragging, disabled; corner/edge/interior positions, drag beyond bounds, keyboard changes and min/max. Preserve hue at black/white or zero-saturation degeneracies; validate change versus change-end callbacks.

**Review and implementation work:** Make painted gradient and value math agree for both axes. Compare dot pattern, checker/transparency composition if applicable, thumb ring, inset edge treatment and coordinate origin. Replace approximate border/shadow treatment where the pinned renderer permits it.

- **Anatomy to account for:** `ColorArea`, `ColorArea.Thumb`.
- **Current metadata axes to reconcile:** `ColorArea.xChannel` = ColorChannel (default: saturation); `ColorArea.yChannel` = ColorChannel (default: brightness); `ColorArea.colorSpace` = ColorSpace (default: —).
- **Current named state rows to retain/revalidate:** Disabled, Dragging, Hovered, Focused, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; With Dots; Color Space & Channels; Controlled; Render Function; Saturation & brightness; Disabled.

### Color Field (`color-field`)

- **Implementation:** [color_picker/field.rs](../../crates/herogpui-components/src/color_picker/field.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-field/color-field.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-input-group/color-input-group.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorField.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-field.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-input-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-field.mdx).

**Required configuration and variant cases:** Primary/Secondary; whole-color and supported channel editing, supported color spaces, full-width/intrinsic, prefix/suffix, controlled/default values, validation modes and required/optional fields.

**Required state and transition cases:** Empty/null, valid, invalid text, intermediate editing, commit/revert, hover/focus, disabled/read-only, required and wheel-disabled. Test keyboard/wheel bounds and form reset/submission; assess nullable value support explicitly instead of accepting a required marker on a concrete-only state.

**Review and implementation work:** Correct invalid focused fill and secondary hover where still missing. Match shared field geometry, placeholder, label/error order and swatch/channel synchronization. Make any empty-value gap a real implementation task with tests and public docs.

- **Anatomy to account for:** `ColorField`, `Label`, `ColorField.Group`, `ColorField.Prefix`, `ColorField.Input`, `ColorField.Suffix`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `ColorField.colorSpace` = ColorSpace (default: —); `ColorField.channel` = ColorChannel (default: —); `ColorField.validationBehavior` = native | aria (default: native); `ColorField.Group.variant` = primary | secondary (default: primary).
- **Current named state rows to retain/revalidate:** Invalid, Required, Disabled, Read only, Focused, Focus within, Focus visible, Hovered.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Variants; On Surface; With Description; Required Field; Disabled State; Full Width; Validation; Channel Editing; Controlled; Render Function; Form Example; Hex value; Read-only display; Single channel.

### Color Picker (`color-picker`)

- **Implementation:** [color_picker/picker.rs](../../crates/herogpui-components/src/color_picker/picker.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs), [overlay_stack_color_deep](../../crates/herogpui-components/tests/overlay_stack_color_deep.rs), [placement](../../crates/herogpui-components/tests/placement.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-picker/color-picker.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorPicker.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Button.tsx) · [API/source 4](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Popover.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-picker.mdx).

**Required configuration and variant cases:** All supported popup placements/offsets; standard/custom trigger and combinations with ColorArea, sliders, swatches and fields; controlled/default colors and controlled/default opening where exposed.

**Required state and transition cases:** Closed/opening/open/closing; pointer/keyboard trigger focus, disabled, changes in each child, outside/Escape dismissal, nested overlay, viewport-edge resize, immediate reopen and reduced motion.

**Review and implementation work:** Match trigger state delegation, panel padding/radius/shadow, full placement-specific motion and synchronized content. Keep popup dimensions inside short windows and avoid independent child controls drifting to different color representations.

- **Anatomy to account for:** `ColorPicker`, `ColorPicker.Trigger`, `ColorPicker.Popover`.
- **Current metadata axes to reconcile:** `ColorPicker.Popover.placement` = Placement (default: bottom left).
- **Current named state rows to retain/revalidate:** Open, Focused, Focus visible, Hovered, Pressed, Disabled, Entering, Exiting.
- **Existing gallery sections to map into specimens:** Usage; Controlled; With Swatches; With Fields; With Sliders.

### Color Slider (`color-slider`)

- **Implementation:** [color_picker/slider.rs](../../crates/herogpui-components/src/color_picker/slider.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [color_geometry_deep](../../crates/herogpui-components/tests/color_geometry_deep.rs), [parts_thumbs](../../crates/herogpui-components/tests/parts_thumbs.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-slider/color-slider.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorSlider.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorThumb.tsx) · [API/source 4](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Slider.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-slider.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-slider.mdx).

**Required configuration and variant cases:** Horizontal/vertical; every supported hue, saturation, lightness, brightness, RGB and alpha channel; custom output/thumb, controlled/default values and custom dimensions.

**Required state and transition cases:** Hover, focus-visible, drag and keyboard change, disabled, min/mid/max, both hue endpoints, zero alpha, release/cancel and pointer outside bounds. Compare continuous changes and change-end callbacks.

**Review and implementation work:** Match track caps, endpoint inset, gradient direction, transparency backing, inner shadows, thumb shadow/border and label/output alignment. Verify hit/value geometry after padding/size overrides; keep vertical label/output layout source-correct.

- **Anatomy to account for:** `ColorSlider`, `Label`, `ColorSlider.Output`, `ColorSlider.Track`, `ColorSlider.Thumb`.
- **Current metadata axes to reconcile:** `ColorSlider.channel` = ColorChannel (default: —); `ColorSlider.colorSpace` = ColorSpace (default: value color space); `ColorSlider.orientation` = horizontal | vertical (default: horizontal).
- **Current named state rows to retain/revalidate:** Disabled, Dragging, Hovered, Focused, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; Disabled; Vertical; Controlled; Render Function; Alpha Channel; HSL Channels; Channels; RGB Channels.

### Color Swatch (`color-swatch`)

- **Implementation:** [color_picker/swatch.rs](../../crates/herogpui-components/src/color_picker/swatch.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [color_geometry_deep](../../crates/herogpui-components/tests/color_geometry_deep.rs), [a11y_pickers_deep](../../crates/herogpui-components/tests/a11y_pickers_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-swatch/color-swatch.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-swatch.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-swatch.mdx).

**Required configuration and variant cases:** Circle/Square × Xs/Sm/Md/Lg/Xl; opaque, translucent and fully transparent colors; named/unnamed swatch and custom dimensions.

**Required state and transition cases:** Static display, value replacement and theme change. Test out-of-gamut conversion policy from the supported color model; do not add selection semantics to a standalone decorative swatch.

**Review and implementation work:** Implement or accurately account for the upstream transparency checkerboard rather than blending everything over one surface. Match border, radius, color fill and optional accessible name; compare against both light and dark backdrops.

- **Anatomy to account for:** `ColorSwatch`.
- **Current metadata axes to reconcile:** `ColorSwatch.color` = string | Color (default: —); `ColorSwatch.shape` = 'circle' | 'square' (default: 'circle'); `ColorSwatch.size` = 'xs' | 'sm' | 'md' | 'lg' | 'xl' (default: 'md').
- **Existing gallery sections to map into specimens:** Usage; Transparency; Accessibility; Sizes; Shapes; Palette; Alpha.

### Color Swatch Picker (`color-swatch-picker`)

- **Implementation:** [color_picker/swatch_picker.rs](../../crates/herogpui-components/src/color_picker/swatch_picker.rs).
- **Gallery / reference:** [colors.rs](../../gallery/src/pages/components/colors.rs) / [metadata](../../gallery/src/pages/reference_metadata/colors.rs).
- **Focused tests to read/extend:** [color_geometry_deep](../../crates/herogpui-components/tests/color_geometry_deep.rs), [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/color-swatch-picker/color-swatch-picker.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ColorSwatchPicker.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ListBox.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/color-swatch-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28colors%29/color-swatch-picker.mdx).

**Required configuration and variant cases:** Circle/Square × Xs/Sm/Md/Lg/Xl; row/grid/stack layout, standard/custom indicator, default/controlled color, disabled group and item.

**Required state and transition cases:** Selected/unselected, selected+hovered, pressed, focused/focus-visible, disabled and all-disabled, keyboard movement and dynamic removal. Check selection by value, including alpha, and callback ownership.

**Review and implementation work:** Match nested swatch/selection-ring geometry and source state precedence. Complete transform interpolation, preserve spacing when selected/pressed, and ensure wrap/grid keyboard behavior matches the supported layout contract.

- **Anatomy to account for:** `ColorSwatchPicker`, `ColorSwatchPicker.Item`, `ColorSwatchPicker.Swatch`, `ColorSwatchPicker.Indicator`.
- **Current metadata axes to reconcile:** `ColorSwatchPicker.size` = xs | sm | md | lg | xl (default: md); `ColorSwatchPicker.variant` = circle | square (default: circle); `ColorSwatchPicker.Item.color` = string | Color (default: required).
- **Current named state rows to retain/revalidate:** Hovered, Pressed, Selected, Focused, Focus visible, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Variants; Sizes; Disabled; Disabled Item; Stack Layout; Default Value; Controlled; Custom Indicator; Item Render State; Square, stacked.

## Controls

### Slider (`slider`)

- **Implementation:** [slider.rs](../../crates/herogpui-components/src/slider.rs).
- **Gallery / reference:** [controls.rs](../../gallery/src/pages/components/controls.rs) / [metadata](../../gallery/src/pages/reference_metadata/controls.rs).
- **Focused tests to read/extend:** [slider_geometry_deep](../../crates/herogpui-components/tests/slider_geometry_deep.rs), [slider_number_deep](../../crates/herogpui-components/tests/slider_number_deep.rs), [slider_form_deep](../../crates/herogpui-components/tests/slider_form_deep.rs), [parts_thumbs](../../crates/herogpui-components/tests/parts_thumbs.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/slider/slider.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/slider.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28controls%29/slider.mdx).

**Required configuration and variant cases:** Horizontal/vertical, scalar and multiple-thumb/range values, steps and format/output options, custom fill/thumbs, named thumbs and per-thumb disabled. Exercise documented HeroGPUI size extensions separately with the default matching upstream.

**Required state and transition cases:** Rest, hover if styled, keyboard-visible focus, dragging, disabled slider/thumb, value endpoints and interior, overlapping/adjacent thumbs, track seeking, drag outside, change/change-end, controlled rejection and form reset. Include fill-start/fill-end semantics and negative/nonzero minima.

**Review and implementation work:** Match track, fill, thumb, ring, drag expansion, labels/output and disabled layer opacity. Paint, keyboard and pointer math must use the same range/orientation/inner bounds. Preserve geometry and hit areas under sx padding/radius/size.

- **Anatomy to account for:** `Slider`, `Label`, `Slider.Output`, `Slider.Track`, `Slider.Fill`, `Slider.Thumb`.
- **Current metadata axes to reconcile:** `Slider.orientation` = "horizontal" | "vertical" (default: "horizontal").
- **Current named state rows to retain/revalidate:** Horizontal, Vertical, Focus visible, Dragging, Disabled slider, Disabled thumb, Fill start, Fill end.
- **Existing gallery sections to map into specimens:** Usage; Format options; Range Slider Anatomy; Controlled Value; Custom Value Formatting; Custom Output Display; Range; Range Slider; Basic Usage; Disabled; Native Seeking; Sizes; Vertical Orientation; Disabled Thumb; Form Example; Vertical; Step & disabled.

### Switch (`switch`)

- **Implementation:** [switch.rs](../../crates/herogpui-components/src/switch.rs).
- **Gallery / reference:** [controls.rs](../../gallery/src/pages/components/controls.rs) / [metadata](../../gallery/src/pages/reference_metadata/controls.rs).
- **Focused tests to read/extend:** [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs), [checkbox_motion](../../crates/herogpui-components/tests/checkbox_motion.rs), [checkbox_form_deep](../../crates/herogpui-components/tests/checkbox_form_deep.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/switch/switch.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/switch.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28controls%29/switch.mdx).

**Required configuration and variant cases:** Sm/Md/Lg; label absent/start/end, description, icons, custom thumb/control and controlled/default selected. Include the existing SwitchGroup composition in horizontal/vertical layouts, without claiming it is a separate upstream catalog route.

**Required state and transition cases:** Off/on, hover, press, focus-visible, disabled, read-only, required, invalid and form participation. Exercise selected+hover/press, pointer versus keyboard activation, rapid reversal and group/child label behavior.

**Review and implementation work:** Match thumb travel/size, pressed width, track colors, content alignment, label line height and transition curves. Verify no duplicate accessible label or callback, no double disabled opacity, and correct reduced-motion endpoints.

- **Anatomy to account for:** `Switch`, `Switch.Control`, `Switch.Thumb`, `Switch.Icon`, `Switch.Content`, `Label`, `Description`, `FieldError`, `SwitchGroup`, `SwitchGroup.Items`.
- **Current metadata axes to reconcile:** `Switch.size` = 'sm' | 'md' | 'lg' (default: 'md'); `Switch.validationBehavior` = 'native' | 'aria' (default: 'native'); `SwitchGroup.orientation` = 'horizontal' | 'vertical' (default: 'vertical').
- **Current named state rows to retain/revalidate:** Selected, Hovered, Pressed, Focus visible, Disabled, Read only, Invalid, Required, Vertical group, Horizontal group.
- **Existing gallery sections to map into specimens:** Usage; Sizes; With Icons; Without Label; With Description; Default Selected; Controlled; Label Position; Group; Group Horizontal; Form Integration; Render Props; Disabled.

## Data display

### Badge (`badge`)

- **Implementation:** [badge.rs](../../crates/herogpui-components/src/badge.rs).
- **Gallery / reference:** [data_display.rs](../../gallery/src/pages/components/data_display.rs) / [metadata](../../gallery/src/pages/reference_metadata/data_display.rs).
- **Focused tests to read/extend:** [badge_parts](../../crates/herogpui-components/tests/badge_parts.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/badge/badge.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/badge.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28data-display%29/badge.mdx).

**Required configuration and variant cases:** Primary/Secondary/Soft × Default/Accent/Success/Warning/Danger × Sm/Md/Lg; four corner placements, dot and content badges, short/long counts, custom anchor children.

**Required state and transition cases:** Static placement, content update, long-content growth and anchor resize in both themes. Any visibility state must come from actual documented composition, not a resurrected v2 invisibility prop.

**Review and implementation work:** Measure overhang using the badge’s resolved size, including grown content; metadata records a minimum-size approximation. Match ring/background clipping, count baseline and dot geometry without changing the anchor’s layout or hit target.

- **Anatomy to account for:** `Badge`, `Badge.Anchor`, `Badge.Label`.
- **Current metadata axes to reconcile:** `Badge.color` = "default" | "accent" | "success" | "warning" | "danger" (default: "default"); `Badge.variant` = "primary" | "secondary" | "soft" (default: "primary"); `Badge.size` = "sm" | "md" | "lg" (default: "md"); `Badge.placement` = "top-right" | "top-left" | "bottom-right" | "bottom-left" (default: "top-right").
- **Existing gallery sections to map into specimens:** Usage; Sizes; Dot Badge; With Content; Variants; Colors; Placements.

### Chip (`chip`)

- **Implementation:** [chip.rs](../../crates/herogpui-components/src/chip.rs).
- **Gallery / reference:** [data_display.rs](../../gallery/src/pages/components/data_display.rs) / [metadata](../../gallery/src/pages/reference_metadata/data_display.rs).
- **Focused tests to read/extend:** [chip_deep](../../crates/herogpui-components/tests/chip_deep.rs), [tag_geometry_deep](../../crates/herogpui-components/tests/tag_geometry_deep.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/chip/chip.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/chip.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28data-display%29/chip.mdx).

**Required configuration and variant cases:** Primary/Secondary/Tertiary/Soft × Default/Accent/Success/Warning/Danger × Sm/Md/Lg; plain text, leading/trailing icons and long content.

**Required state and transition cases:** Static display and content/theme changes. Use TagGroup for selectable/removable semantics unless the pinned Chip composition explicitly supplies an action.

**Review and implementation work:** Match per-size padding, radius, icon gap, foreground and the compiled CSS line-height cascade. Verify intrinsic width in ordinary and flex parents. Do not infer hover or press states for a noninteractive Chip.

- **Anatomy to account for:** `Chip`, `Chip.Label`.
- **Current metadata axes to reconcile:** `Chip.color` = "default" | "accent" | "success" | "warning" | "danger" (default: "default"); `Chip.variant` = "primary" | "secondary" | "tertiary" | "soft" (default: "secondary"); `Chip.size` = "sm" | "md" | "lg" (default: "md").
- **Existing gallery sections to map into specimens:** Usage; Statuses; With Icons; Variants; Colors; Sizes.

### Table (`table`)

- **Implementation:** [table.rs](../../crates/herogpui-components/src/table.rs), [scrollbar.rs](../../crates/herogpui-components/src/scrollbar.rs), [selection.rs](../../crates/herogpui-components/src/selection.rs).
- **Gallery / reference:** [data_display.rs](../../gallery/src/pages/components/data_display.rs) / [metadata](../../gallery/src/pages/reference_metadata/data_display.rs).
- **Focused tests to read/extend:** [table_deep](../../crates/herogpui-components/tests/table_deep.rs), [table_and_drag](../../crates/herogpui-components/tests/table_and_drag.rs), [table_tabs_accordion](../../crates/herogpui-components/tests/table_tabs_accordion.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs), [a11y_collections_deep](../../crates/herogpui-components/tests/a11y_collections_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/table/table.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Table.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/table.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28data-display%29/table.mdx).

**Required configuration and variant cases:** Primary/Secondary; none/single/multiple selection and toggle/replace behavior; tab/arrow keyboard navigation; sortable and custom-indicator columns, resizable columns, expandable rows, custom editable cells, virtualization, pagination and async loading.

**Required state and transition cases:** Empty/loading/loaded/error composition, hover, selected, focus-visible, disabled rows, ascending/descending sort, resize hover/drag/keyboard mode, expansion, nested input focus, typeahead in noneditable cells and dynamic row/column changes. Retain explicit exclusions for unsupported drag/drop; do not infer those features from CSS states.

**Review and implementation work:** Implement shared column tracks so narrow headers and rows align; verify mixed widths, long cells, virtualized rows, horizontal scrolling and resizing. Reconcile per-cell hover/selected fills, secondary header corners, separators, sort layout/motion, tree indentation and segmented inset focus. Tree-column indentation now follows the pinned 16px-per-level rule in both painted cells and intrinsic-width measurement. Read existing behavior before changing the v3.2.5 nested-editor/typeahead work.

- **Anatomy to account for:** `Table`, `Table.ScrollContainer`, `Table.Content`, `Table.Header`, `Table.Column`, `Table.Body`, `Table.Row`, `Table.Cell`, `Table.SortableColumnHeader`, `Table.Footer`, `Table.ColumnResizer`, `Table.ResizableContainer`, `Table.LoadMore`, `Table.LoadMoreContent`, `Table.Collection`.
- **Current metadata axes to reconcile:** `Table.variant` = "primary" | "secondary" (default: "primary"); `Table.Content.selectionMode` = "none" | "single" | "multiple" (default: "none"); `Table.Content.selectionBehavior` = "toggle" | "replace" (default: "toggle", now implemented with replace-on-focus semantics); `Table.Content.keyboardNavigationBehavior` = "arrow" | "tab" (default: "arrow"). `defaultSelectedKeys` and `disallowEmptySelection` are also implemented; tab-mode child navigation remains a separate framework-limited slice. Native intrinsic tracks are retained, while wasm canvas tracks stay fluid across state rerenders unless a width is explicitly pinned.
- **Current named state rows to retain/revalidate:** Hovered, Selected, Focus visible, Disabled row, Disabled table, Sortable, Resizing, Empty, Loading, Dragging, Drop target.
- **Existing gallery sections to map into specimens:** Usage; Row Hover; Variants; Custom sort indicator; Selection; Sorting; Virtualization; Column Resizing; Expandable Rows; Secondary Variant; Async Loading; Pagination; Custom Cells; Empty State; Loading.

## Date and time

### Calendar (`calendar`)

- **Implementation:** [calendar.rs](../../crates/herogpui-components/src/calendar.rs), [calendar_view.rs](../../crates/herogpui-components/src/calendar_view.rs), [calendar_system.rs](../../crates/herogpui-components/src/calendar_system.rs), [date_constraints.rs](../../crates/herogpui-components/src/date_constraints.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [calendars_deep](../../crates/herogpui-components/tests/calendars_deep.rs), [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs), [a11y_pickers_deep](../../crates/herogpui-components/tests/a11y_pickers_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/calendar/calendar.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/calendar.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/calendar-year-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/calendar.mdx).

**Required configuration and variant cases:** Single/multiple selection; day/week/month/multiple-month views, weeksInMonth, all first-day values, visible/single page behavior, start/center/end selection alignment; min/max and unavailable predicates; supported international calendars, heading offsets and custom navigation/cell indicators.

**Required state and transition cases:** Empty/selected/today, today+selected/hovered, hovered/pressed/focus-visible, disabled/read-only/invalid, unavailable/outside-month/out-of-range and month/year boundaries. Year picker closed/open/enter/exit, selected/focused year, paging, controlled focusedValue and keyboard navigation need independent cases.

**Review and implementation work:** Correct today soft fill and hover, selected press, unavailable/disabled treatment and indicator inset where still approximate. Match day-grid geometry and nav/year-trigger styling; implement year-picker crossfade/chevron rotation and stable retained layout. Normalize time/date for captures and use source-correct calendar conversions.

- **Anatomy to account for:** `Calendar`, `Calendar.Header`, `Calendar.Heading`, `Calendar.NavButton`, `Calendar.Grid`, `Calendar.GridHeader`, `Calendar.GridBody`, `Calendar.HeaderCell`, `Calendar.Cell`, `Calendar.CellIndicator`, `Calendar.YearPickerTrigger`, `Calendar.YearPickerTriggerHeading`, `Calendar.YearPickerTriggerIndicator`, `Calendar.YearPickerGrid`, `Calendar.YearPickerGridBody`, `Calendar.YearPickerCell`.
- **Current metadata axes to reconcile:** `Calendar.selectionMode` = 'single' | 'multiple' (default: 'single'); `Calendar.firstDayOfWeek` = 'sun' | 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' (default: Locale default); `Calendar.pageBehavior` = 'visible' | 'single' (default: 'visible'); `Calendar.selectionAlignment` = 'start' | 'center' | 'end' (default: 'center').
- **Current named state rows to retain/revalidate:** Selected, Today, Unavailable, Outside month, Hovered, Pressed, Focus visible, Disabled, Read only, Year picker open, Year cell selected.
- **Existing gallery sections to map into specimens:** International Calendars; Usage; Default Value; Controlled; Min and Max Dates; Unavailable Dates; Weeks in Month; Multiple Selection; Focused Value; Cell Indicators; Custom Navigation Icons; Real-World Example; Constraints; First day of week; Disabled; Read Only; Multiple Months; Week View; Day View; Year Picker; Heading Offset.

### Date Field (`date-field`)

- **Implementation:** [date_picker/field.rs](../../crates/herogpui-components/src/date_picker/field.rs), [date_constraints.rs](../../crates/herogpui-components/src/date_constraints.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [date_field_picker_deep](../../crates/herogpui-components/tests/date_field_picker_deep.rs), [date_field_form_deep](../../crates/herogpui-components/tests/date_field_form_deep.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/date-field/date-field.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/DateField.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/date-field.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/date-input-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/date-field.mdx).

**Required configuration and variant cases:** Primary/Secondary; day/hour/minute/second granularity, 12/24-hour display, forced leading zeros, timezone visibility and supported calendar/locale formats; prefix/suffix, full width, validation mode and controlled/default nullable values.

**Required state and transition cases:** Empty/partial/complete, focused segment and placeholder segments, hover/focus-within, invalid, required, disabled/read-only, min/max and cross-month/year editing. Test arrows, deletion, digit entry, increment/decrement, blur/commit and form reset.

**Review and implementation work:** Match segment gap, end alignment/tabular figures, selected segment foreground including invalid state, field chrome, label/helper/error order and per-state interpolation. Avoid replacing the native segment interaction with a visually plausible static date string.

- **Anatomy to account for:** `DateField`, `Label`, `DateField.Group`, `DateField.Input`, `DateField.InputContainer`, `DateField.Segment`, `DateField.Prefix`, `DateField.Suffix`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `DateField.validationBehavior` = 'native' | 'aria' (default: 'native'); `DateField.granularity` = 'day' | 'hour' | 'minute' | 'second' (default: 'day'); `DateField.hourCycle` = 12 | 24 (default: locale); `DateField.Group.variant` = 'primary' | 'secondary' (default: 'primary').
- **Current named state rows to retain/revalidate:** Hovered, Focus within, Invalid, Disabled, Read only, Required, Segment focused, Segment placeholder.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Granularity; Forced Leading Zeros; With Icons; Variants; In Surface; With Description; Required Field; Disabled State; Full Width; Validation; Controlled; With Validation; Form Example.

### Date Picker (`date-picker`)

- **Implementation:** [date_picker/picker.rs](../../crates/herogpui-components/src/date_picker/picker.rs), [date_picker/field.rs](../../crates/herogpui-components/src/date_picker/field.rs), [calendar.rs](../../crates/herogpui-components/src/calendar.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [date_field_picker_deep](../../crates/herogpui-components/tests/date_field_picker_deep.rs), [date_picker_close](../../crates/herogpui-components/tests/date_picker_close.rs), [date_picker_placement](../../crates/herogpui-components/tests/date_picker_placement.rs), [overlay_stack_date_deep](../../crates/herogpui-components/tests/overlay_stack_date_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/date-picker/date-picker.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/DatePicker.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/date-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/date-picker.mdx).

**Required configuration and variant cases:** Supported field variants/formats and calendar systems; popup placements/offsets, custom indicator/render function, controlled/default date/open state, min/max/unavailable dates and validation modes.

**Required state and transition cases:** Closed/opening/open/closing; field-focused versus calendar-focused, date selected/cleared, keyboard and pointer selection, invalid/required/disabled/read-only, year picker, outside/Escape and programmatic close. Verify focus return does not reopen the popup.

**Review and implementation work:** Keep the unified field and calendar anatomy source-correct, with complete popup padding, shadow, the React Aria default eight-pixel trigger gap and placement-aware anchor positioning. The shared `Placement` builder now covers HeroUI's default centered bottom position plus the portable top/side variants, keyed entry/exit motion follows the pinned 150ms ease-smooth zoom/fade and 100ms retained zoom/fade timing with a four-pixel placement slide, and the shared positioner flips to the opposite physical side when the preferred side overflows. The positioner now caps the panel to short viewports and the retained exit passes an internal inert state through the calendar, removing stale cell/navigation/year-picker focus and pointer handlers without dimming the painted exit. Verify scroll-to-trigger, invalid segment presentation, keyboard-only focus modality and consistency between typed and picked values.

- **Anatomy to account for:** `DatePicker.Root`, `DatePicker.Trigger`, `DatePicker.TriggerIndicator`, `DatePicker.Popover`.
- **Current metadata axes to reconcile:** `DatePicker.validationBehavior` = "native" | "aria" (default: "native"); `DatePicker.firstDayOfWeek` = string (default: locale); `DatePicker.Popover.placement` = Placement (default: bottom).
- **Current named state rows to retain/revalidate:** Open, Focus within, Focus visible, Disabled, Read only, Required, Invalid.
- **Existing gallery sections to map into specimens:** International Calendar; Disabled; Controlled; Validation; Format Options; Form Example; Custom Indicator; Render Function; Usage; Placement.

### Date Range Picker (`date-range-picker`)

- **Implementation:** [date_picker/range.rs](../../crates/herogpui-components/src/date_picker/range.rs), [date_picker/field.rs](../../crates/herogpui-components/src/date_picker/field.rs), [range_calendar.rs](../../crates/herogpui-components/src/range_calendar.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [date_field_picker_deep](../../crates/herogpui-components/tests/date_field_picker_deep.rs), [date_picker_close](../../crates/herogpui-components/tests/date_picker_close.rs), [date_picker_placement](../../crates/herogpui-components/tests/date_picker_placement.rs), [overlay_stack_date_deep](../../crates/herogpui-components/tests/overlay_stack_date_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/date-range-picker/date-range-picker.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/DatePicker.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/date-range-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/date-range-picker.mdx).

**Required configuration and variant cases:** Supported variants, range formats/calendar systems, placement, custom indicator and controlled/default values; min/max, unavailable/anchor-dependent dates and allowed noncontiguous ranges where exposed.

**Required state and transition cases:** Empty range, start-only, complete/same-day/reversed request, invalid range, disabled/read-only/required, active segment, open/enter/exit and year-picker. Test picking across months, typed endpoints, cancellation, clearing, outside dismissal and focus restoration.

**Review and implementation work:** Match separator, segment spacing, shared trigger chrome, the React Aria default eight-pixel trigger gap and placement-aware open multi-month panel geometry. The shared `Placement` builder now covers HeroUI's default centered bottom position plus the portable top/side variants, keyed entry/exit motion follows the pinned 150ms ease-smooth zoom/fade and 100ms retained zoom/fade timing with a four-pixel placement slide, and the shared positioner flips to the opposite physical side when the preferred side overflows. The positioner now caps the panel to short viewports and the retained exit passes an internal inert state through the range calendar, removing stale cell/navigation/year-picker focus and pointer handlers without dimming the painted exit. Verify consistent range bands/caps and no clipped last week/options at viewport edges; controlled selection must not be overwritten by local draft changes.

- **Anatomy to account for:** `DateRangePicker.Root`, `DateRangePicker.Trigger`, `DateRangePicker.TriggerIndicator`, `DateRangePicker.RangeSeparator`, `DateRangePicker.Popover`.
- **Current metadata axes to reconcile:** `DateRangePicker.validationBehavior` = "native" | "aria" (default: "native"); `DateRangePicker.firstDayOfWeek` = string (default: locale); `DateRangePicker.Popover.placement` = Placement (default: bottom).
- **Current named state rows to retain/revalidate:** Open, Focus within, Focus visible, Disabled, Read only, Required, Invalid.
- **Existing gallery sections to map into specimens:** International Calendar; Disabled; Controlled; Validation; Format Options; Form Example; Custom Indicator; Render Function; Usage; Placement.

### Range Calendar (`range-calendar`)

- **Implementation:** [range_calendar.rs](../../crates/herogpui-components/src/range_calendar.rs), [calendar_view.rs](../../crates/herogpui-components/src/calendar_view.rs), [calendar_system.rs](../../crates/herogpui-components/src/calendar_system.rs), [date_constraints.rs](../../crates/herogpui-components/src/date_constraints.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [calendars_deep](../../crates/herogpui-components/tests/calendars_deep.rs), [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs), [a11y_pickers_deep](../../crates/herogpui-components/tests/a11y_pickers_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/range-calendar/range-calendar.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/range-calendar.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/calendar-year-picker.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/range-calendar.mdx).

**Required configuration and variant cases:** Day/week/month/multiple-month views, first-day values, visible/single paging, start/center/end alignment, year grid options and supported calendar systems; contiguous/noncontiguous policy and anchor-based unavailability.

**Required state and transition cases:** No anchor, anchor-only, preview, completed range, same-day and cross-week/month range; start/middle/end/today/hover/press/focus combinations; unavailable/outside/disabled/read-only/invalid and year-picker states. Test pointer preview and keyboard commit agreement.

**Review and implementation work:** Replace simplified endpoint pills/row caps with source-correct continuous range tracks and directional endpoints. Correct today fill, indicator inset and press interpolation; inspect grid-body/grid-row ownership previously reported unverified. Preserve disabled outside-month exceptions and range layout across month panels.

- **Anatomy to account for:** `RangeCalendar`, `RangeCalendar.Header`, `RangeCalendar.Heading`, `RangeCalendar.NavButton`, `RangeCalendar.Grid`, `RangeCalendar.GridHeader`, `RangeCalendar.GridBody`, `RangeCalendar.HeaderCell`, `RangeCalendar.Cell`, `RangeCalendar.CellIndicator`, `RangeCalendar.YearPickerTrigger`, `RangeCalendar.YearPickerTriggerHeading`, `RangeCalendar.YearPickerTriggerIndicator`, `RangeCalendar.YearPickerGrid`, `RangeCalendar.YearPickerGridBody`, `RangeCalendar.YearPickerCell`.
- **Current metadata axes to reconcile:** `RangeCalendar.firstDayOfWeek` = 'sun' | 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' (default: Locale default); `RangeCalendar.pageBehavior` = 'visible' | 'single' (default: 'visible'); `RangeCalendar.selectionAlignment` = 'start' | 'center' | 'end' (default: 'center').
- **Current named state rows to retain/revalidate:** Selected range, Range middle, Selection start, Selection end, Today, Unavailable, Outside month, Hovered, Pressed, Focus visible, Disabled, Read only, Year picker open.
- **Existing gallery sections to map into specimens:** International Calendars; Disabled; Cell Indicators; Year Picker; Heading Offset; Default Value; Controlled; Min and Max Dates; Unavailable Dates; Anchor-Based Unavailable Dates; Allows Non-Contiguous Ranges; Weeks in Month; Week View; Day View; Multiple Months; Read Only; Invalid; Focused Value; Real-World Example; Usage.

### Time Field (`time-field`)

- **Implementation:** [time_field.rs](../../crates/herogpui-components/src/time_field.rs).
- **Gallery / reference:** [date_and_time.rs](../../gallery/src/pages/components/date_and_time.rs) / [metadata](../../gallery/src/pages/reference_metadata/date_and_time.rs).
- **Focused tests to read/extend:** [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [date_field_form_deep](../../crates/herogpui-components/tests/date_field_form_deep.rs), [calendars_and_more](../../crates/herogpui-components/tests/calendars_and_more.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/time-field/time-field.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/TimeField.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/time-field.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/date-input-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28date-and-time%29/time-field.mdx).

**Required configuration and variant cases:** Primary/Secondary; hour/minute/second, 12/24-hour, leading zeros, supported timezone display, prefix/suffix, controlled/default values and validation modes.

**Required state and transition cases:** Empty/partial/complete, focused segment, AM/PM changes, boundary rollover, invalid, required, disabled/read-only and form reset. Test digit entry, arrows, deletion, blur and pointer focus separately.

**Review and implementation work:** Match segment gap, placeholder/invalid selected text, tabular/end-aligned numerals, label/helper/error and field transitions. Compare 12-hour and seconds layouts independently instead of judging from one compact value.

- **Anatomy to account for:** `TimeField`, `Label`, `TimeField.Group`, `TimeField.Input`, `TimeField.Segment`, `TimeField.Prefix`, `TimeField.Suffix`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `TimeField.validationBehavior` = 'native' | 'aria' (default: 'native'); `TimeField.granularity` = 'hour' | 'minute' | 'second' (default: 'minute'); `TimeField.hourCycle` = 12 | 24 (default: locale); `TimeField.Group / TimeField.Input.variant` = 'primary' | 'secondary' (default: 'primary').
- **Current named state rows to retain/revalidate:** Hovered, Focus within, Invalid, Disabled, Read only, Required, Segment focused, Segment placeholder.
- **Existing gallery sections to map into specimens:** 24-hour; 12-hour with seconds; Forced Leading Zeros; Usage; Box Customisation; With Icons; On Surface; With Description; Required Field; Disabled State; Full Width; Validation; Controlled; With Validation; Form Example.

## Feedback

### Alert (`alert`)

- **Implementation:** [alert.rs](../../crates/herogpui-components/src/alert.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [alert_deep](../../crates/herogpui-components/tests/alert_deep.rs), [feedback_compose](../../crates/herogpui-components/tests/feedback_compose.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/alert/alert.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/alert.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/alert.mdx).

**Required configuration and variant cases:** Default/Accent/Success/Warning/Danger status; title only, title+description, long wrapping content, indicator composition and a composed CloseButton. Do not re-add removed isClosable/onClose props.

**Required state and transition cases:** Static visible alert and actual composed dismissal; keyboard focus/hover/press belongs to the action or close control. Exercise asynchronous insertion and both themes where a consumer shows alerts dynamically.

**Review and implementation work:** Match status glyph/box, title and description hierarchy, content column, surface and spacing. Review missing custom indicator and arbitrary content seams as portable composition work; do not label them a generic DOM limitation.

- **Anatomy to account for:** `Alert`, `Alert.Indicator`, `Alert.Content`, `Alert.Title`, `Alert.Description`.
- **Current metadata axes to reconcile:** `Alert.status` = "default" | "accent" | "success" | "warning" | "danger" (default: "default").
- **Existing gallery sections to map into specimens:** Usage; Colors; Closable.

### Meter (`meter`)

- **Implementation:** [meter.rs](../../crates/herogpui-components/src/meter.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [feedback](../../crates/herogpui-components/tests/feedback.rs), [feedback_compose](../../crates/herogpui-components/tests/feedback_compose.rs), [value_props](../../crates/herogpui-components/tests/value_props.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/meter/meter.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Meter.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/meter.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/meter.mdx).

**Required configuration and variant cases:** Sm/Md/Lg × Default/Accent/Success/Warning/Danger; label/output on/off, custom scale and formatted output, custom track/fill composition.

**Required state and transition cases:** Determinate min/interior/max, zero and nonzero minimum, clamping and programmatic value updates. Meter is not an indeterminate loading control.

**Review and implementation work:** Match track height/radius, fill ratio and label/output baseline without misleading percent math for nonzero minima. Verify role/value text and per-part customization do not recolor the wrong surface.

- **Anatomy to account for:** `Meter`, `Meter.Output`, `Meter.Track`, `Meter.Fill`.
- **Current metadata axes to reconcile:** `Meter.size` = 'sm' | 'md' | 'lg' (default: 'md'); `Meter.color` = 'default' | 'accent' | 'success' | 'warning' | 'danger' (default: 'accent').
- **Current named state rows to retain/revalidate:** Determinate.
- **Existing gallery sections to map into specimens:** Usage; Colors; Sizes; Without Label; Custom Value Scale.

### Progress Bar (`progress-bar`)

- **Implementation:** [progress.rs](../../crates/herogpui-components/src/progress.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [feedback](../../crates/herogpui-components/tests/feedback.rs), [feedback_compose](../../crates/herogpui-components/tests/feedback_compose.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs), [value_props](../../crates/herogpui-components/tests/value_props.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/progress-bar/progress-bar.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ProgressBar.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria@3.52.0/packages/react-aria/src/progress/useProgressBar.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/progress-bar.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/progress-bar.mdx).

**Required configuration and variant cases:** Sm/Md/Lg × Default/Accent/Success/Warning/Danger; label/output on/off, custom scales and composed track/fill.

**Required state and transition cases:** Determinate min/mid/max, indeterminate, disabled where supported, value changes and normal/reduced motion. Test transition between determinate and indeterminate and the accessibility value omission for indeterminate.

**Review and implementation work:** Match bar geometry, movement, fill clipping, label/output typography and tabular figure treatment. Keep motion continuous without a repaint loop when reduced motion is active.

- **Anatomy to account for:** `ProgressBar`, `ProgressBar.Output`, `ProgressBar.Track`, `ProgressBar.Fill`.
- **Current metadata axes to reconcile:** `ProgressBar.size` = "sm" | "md" | "lg" (default: "md"); `ProgressBar.color` = "default" | "accent" | "success" | "warning" | "danger" (default: "accent").
- **Current named state rows to retain/revalidate:** Determinate, Indeterminate, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Colors; Sizes; Without Label; Indeterminate; Custom Value Scale.

### Progress Circle (`progress-circle`)

- **Implementation:** [progress.rs](../../crates/herogpui-components/src/progress.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [feedback](../../crates/herogpui-components/tests/feedback.rs), [feedback_compose](../../crates/herogpui-components/tests/feedback_compose.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/progress-circle/progress-circle.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ProgressBar.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/progress-circle.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/progress-circle.mdx).

**Required configuration and variant cases:** Sm/Md/Lg × Default/Accent/Success/Warning/Danger; label/custom center content and supported SVG-equivalent presentation seams.

**Required state and transition cases:** Determinate zero/interior/full, indeterminate, disabled if exposed, value interpolation and determinate/indeterminate switching; reduced motion.

**Review and implementation work:** Match circle diameter, track thickness, arc caps/origin and label centering. Complete value-change arc interpolation where current redraw is immediate, while retaining the correct value and range accessibility contract.

- **Anatomy to account for:** `ProgressCircle`, `ProgressCircle.Track`, `ProgressCircle.TrackCircle`, `ProgressCircle.FillCircle`.
- **Current metadata axes to reconcile:** `ProgressCircle.size` = "sm" | "md" | "lg" (default: "md"); `ProgressCircle.color` = "default" | "accent" | "success" | "warning" | "danger" (default: "accent").
- **Current named state rows to retain/revalidate:** Determinate, Indeterminate, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Indeterminate; With Label; Custom SVG Props; Colors; Sizes.

### Skeleton (`skeleton`)

- **Implementation:** [skeleton.rs](../../crates/herogpui-components/src/skeleton.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [feedback](../../crates/herogpui-components/tests/feedback.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs), [theme_platform](../../crates/herogpui-components/tests/theme_platform.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/skeleton/skeleton.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/skeleton.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/skeleton.mdx).

**Required configuration and variant cases:** Shimmer/Pulse/None; lines, circles, cards, user profiles, lists and grids through composition and explicit dimensions.

**Required state and transition cases:** Loading to revealed content, normal/reduced motion, nested/background surfaces and dynamically resized skeletons. Verify a single shimmer field and any group timing described upstream.

**Review and implementation work:** Match surface/radius/opacity and animation timing, with no content leak outside clipping. Preserve layout through replacement where the example promises it; static None must do no unnecessary animation work.

- **Anatomy to account for:** `Skeleton`.
- **Current metadata axes to reconcile:** `Skeleton.animationType` = 'shimmer' | 'pulse' | 'none' (default: theme).
- **Current named state rows to retain/revalidate:** Shimmer, Pulse, No animation.
- **Existing gallery sections to map into specimens:** Usage; Text Content; User Profile; List Items; Grid; Single Shimmer; Animation Types; Loading.

### Spinner (`spinner`)

- **Implementation:** [spinner.rs](../../crates/herogpui-components/src/spinner.rs).
- **Gallery / reference:** [feedback.rs](../../gallery/src/pages/components/feedback.rs) / [metadata](../../gallery/src/pages/reference_metadata/feedback.rs).
- **Focused tests to read/extend:** [feedback](../../crates/herogpui-components/tests/feedback.rs), [theme_platform](../../crates/herogpui-components/tests/theme_platform.rs), [a11y_deep](../../crates/herogpui-components/tests/a11y_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/spinner/spinner.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/spinner.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28feedback%29/spinner.mdx).

**Required configuration and variant cases:** Sm/Md/Lg/Xl × Current/Accent/Success/Warning/Danger; custom speed where supported, standalone and inside pending controls.

**Required state and transition cases:** Spinning, normal/reduced motion, mounted/unmounted and inherited current-color changes. Ensure the composed button’s pending lifecycle does not create a second announcement or leaked animation.

**Review and implementation work:** Match dimensions, segment geometry, speed and color, including explicit SVG current-color resolution. Preserve the upstream loading role/name and compare at fixed phase rather than arbitrary timestamps.

- **Anatomy to account for:** `Spinner`.
- **Current metadata axes to reconcile:** `Spinner.size` = 'sm' | 'md' | 'lg' | 'xl' (default: 'md'); `Spinner.color` = 'current' | 'accent' | 'success' | 'warning' | 'danger' (default: 'accent').
- **Current named state rows to retain/revalidate:** Spinning.
- **Existing gallery sections to map into specimens:** Usage; Speed; Colors; Sizes.

## Forms

### Checkbox (`checkbox`)

- **Implementation:** [checkbox.rs](../../crates/herogpui-components/src/checkbox.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [checkbox_form_deep](../../crates/herogpui-components/tests/checkbox_form_deep.rs), [checkbox_motion](../../crates/herogpui-components/tests/checkbox_motion.rs), [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/checkbox/checkbox.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/checkbox.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/checkbox.mdx).

**Required configuration and variant cases:** Primary/Secondary; standard/custom indicator, label/content/description, external label, default/controlled selected and mixed state. Include documented HeroGPUI sizes/radii as an extension suite, preserving the upstream default.

**Required state and transition cases:** Unchecked/checked/indeterminate × hover/press/focus-visible; invalid, disabled, read-only and required; controlled rejection, mixed-state activation, form reset and rapid reversal.

**Review and implementation work:** Match unselected border/background, selected and mixed indicators, hover layers, focus ring, press geometry and check animation. Verify long labels and nested content activation do not double toggle or duplicate labels.

- **Anatomy to account for:** `Checkbox`, `Checkbox.Content`, `Checkbox.Control`, `Checkbox.Indicator`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `Checkbox.validationBehavior` = "native" | "aria" (default: "native"); `Checkbox.variant` = "primary" | "secondary" (default: "primary").
- **Current named state rows to retain/revalidate:** Selected, Indeterminate, Invalid, Hovered, Focus visible, Disabled, Pressed, Read only.
- **Existing gallery sections to map into specimens:** Usage; Sizes; Variants; Full Rounded; Disabled; External Label; With Description; Default Selected; Invalid; Controlled; Indeterminate; Form Integration; Render Props; Custom Indicator.

### Checkbox Group (`checkbox-group`)

- **Implementation:** [checkbox.rs](../../crates/herogpui-components/src/checkbox.rs), [field.rs](../../crates/herogpui-components/src/field.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [checkbox_form_deep](../../crates/herogpui-components/tests/checkbox_form_deep.rs), [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/checkbox-group/checkbox-group.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/checkbox-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/checkbox-group.mdx).

**Required configuration and variant cases:** Vertical/horizontal composition, Primary/Secondary member variants, label/description/error, disabled group/member, controlled/default arrays and select-all/mixed examples.

**Required state and transition cases:** None/some/all checked, indeterminate aggregate, disabled/read-only/required/invalid, controlled updates and field-level/group-level errors; keyboard traversal and form reset.

**Review and implementation work:** Match group label-to-options-to-description/error rhythm and child content alignment. Group restrictions must preserve child state ownership and apply disabled opacity once; validation must reflect the live selection.

- **Anatomy to account for:** `CheckboxGroup`, `Label`, `Description`, `Checkbox`, `Checkbox.Content`, `Checkbox.Control`, `Checkbox.Indicator`, `FieldError`.
- **Current named state rows to retain/revalidate:** Disabled, Read only, Invalid, Required, Controlled, Uncontrolled.
- **Existing gallery sections to map into specimens:** Usage; In Surface; Disabled; Indeterminate; Controlled; Validation; Features and Add-ons Example; With Custom Indicator; Vertical; Horizontal & invalid; Uncontrolled.

### Fieldset (`fieldset`)

- **Implementation:** [field.rs](../../crates/herogpui-components/src/field.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [field_slots_deep](../../crates/herogpui-components/tests/field_slots_deep.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs), [a11y_deep](../../crates/herogpui-components/tests/a11y_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/fieldset/fieldset.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/fieldset.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/fieldset.mdx).

**Required configuration and variant cases:** Legend, description and grouped controls; plain and Surface composition, wrapping legend/help, optional id and disabled behavior only where actually supported.

**Required state and transition cases:** Content/error updates and focus movement among children; static fieldset does not acquire hover/press semantics. Verify child disabling from the actual supported owner rather than presumed browser inheritance.

**Review and implementation work:** Match legend/header/content spacing, text hierarchy and full-width layout. Ensure optional accessibility naming reflects the grouping without inventing unsupported roles or leaving stale claims about the tree.

- **Anatomy to account for:** `Fieldset`, `Fieldset.Legend`, `Fieldset.Group`, `Fieldset.Actions`.
- **Existing gallery sections to map into specimens:** In Surface; Usage.

### Label & Messages (`label-messages`)

- **Implementation:** [field.rs](../../crates/herogpui-components/src/field.rs), [validation.rs](../../crates/herogpui-components/src/validation.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [field_slots_deep](../../crates/herogpui-components/tests/field_slots_deep.rs), [fields](../../crates/herogpui-components/tests/fields.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs), [a11y_deep](../../crates/herogpui-components/tests/a11y_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/label/label.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/description/description.tsx) · [API/source 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/error-message/error-message.tsx) · [API/source 4](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/field-error/field-error.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/label.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/description.css) · [styles 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/error-message.css) · [styles 4](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/field-error.css) · [docs 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/label.mdx) · [docs 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/description.mdx) · [docs 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/error-message.mdx) · [docs 4](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/field-error.mdx).

**Required configuration and variant cases:** This page owns four separate required components: Label, Description, ErrorMessage and FieldError. Label: optional/required and enabled/disabled. Description: short/long/multiline helper text. ErrorMessage: explicit static/dynamic errors. FieldError: resolved validation, custom render and multiple errors.

**Required state and transition cases:** Label: focus forwarding where supported, required marker, disabled and invalid color. Description: shown/hidden/replaced according to the field contract. ErrorMessage: appearance/update/removal. FieldError: valid/invalid, built-in/custom/server errors, multiple messages and reset. Test these inside text, choice, date and picker owners.

**Review and implementation work:** Match each primitive’s typography, inset, wrapping and ordering. Avoid nested/duplicate label naming in Checkbox/Switch/Radio content. Required markers and error text must remain readable; changing validation must update both visible and accessible descriptions. Provide a dedicated anchor and minimal example for each of the four components.

- **Anatomy to account for:** `Label`, `Description`, `ErrorMessage`, `FieldError`.
- **Current named state rows to retain/revalidate:** Required, Disabled, Invalid.
- **Existing gallery sections to map into specimens:** Usage; With Required Indicator; With Disabled State; With Invalid State; With Form Fields; Integration with TextField; Basic Validation; With Dynamic Messages; Custom Validation Logic; Multiple Error Messages; Label; Description & error; FieldError.

### Form (`form`)

- **Implementation:** [form.rs](../../crates/herogpui-components/src/form.rs), [validation.rs](../../crates/herogpui-components/src/validation.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs), [checkbox_form_deep](../../crates/herogpui-components/tests/checkbox_form_deep.rs), [date_field_form_deep](../../crates/herogpui-components/tests/date_field_form_deep.rs), [slider_form_deep](../../crates/herogpui-components/tests/slider_form_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/form/form.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Form.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/form.mdx).

**Required configuration and variant cases:** Native/aria validation behavior, client/server validation errors, submit/reset controls, text/choice/date/color/range fields and custom layout. Include controlled and uncontrolled fields in the same form.

**Required state and transition cases:** Pristine/edited/invalid/valid/submitting-result composition, first-invalid focus, submit via Enter/action, reset, disabled-field omission and read-only inclusion. Test default submitter limitations, stale server errors, dynamically removed fields and live state after builder reconstruction.

**Review and implementation work:** Make validation messages/required markers/layout consistent across every field family; avoid moving focus on mere repaint. Document the native form transport/default-submitter difference accurately and prove supported reset/submission semantics with actual field values.

- **Anatomy to account for:** `Form`.
- **Current metadata axes to reconcile:** `Form.validationBehavior` = 'native' | 'aria' (default: 'native').
- **Current named state rows to retain/revalidate:** Blocked submit focuses first invalid, Enter / default submitter, Read-only bar, Disabled omission, Server errors displayed.
- **Existing gallery sections to map into specimens:** Usage; Server Errors.

### Input (`input`)

- **Implementation:** [input.rs](../../crates/herogpui-components/src/input.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [fields](../../crates/herogpui-components/tests/fields.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [sx_slot](../../crates/herogpui-components/tests/sx_slot.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/input/input.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Input.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/input.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/input.mdx).

**Required configuration and variant cases:** Primary/Secondary; every actually supported InputType, empty/populated, placeholder, full/intrinsic width, custom box/font and bare appearance. Native/HTML-only hints must be identified separately from supported input behavior.

**Required state and transition cases:** Rest/hover/focus/focus-visible, invalid+focused, disabled/read-only, selection/caret, insertion/deletion, keyboard shortcuts, clipboard, composition/IME, horizontal scrolling and controlled value updates. Test active edits across rerenders and theme changes.

**Review and implementation work:** Match field metrics, baseline, placeholder, focused/invalid precedence, text selection and caret. Complete property transitions where missing; verify custom dimensions do not clip text or let child chrome cover the intended sx override.

- **Anatomy to account for:** `Input`.
- **Current metadata axes to reconcile:** `Input.type` = string (default: "text"); `Input.variant` = "primary" | "secondary" (default: "primary").
- **Current named state rows to retain/revalidate:** Hover, Focused, Focus visible, Invalid, Disabled, Read only.
- **Existing gallery sections to map into specimens:** Variants; Usage; In Surface; Full Width; Input Types; Controlled; States; Compact Box.

### Input Group (`input-group`)

- **Implementation:** [input_group.rs](../../crates/herogpui-components/src/input_group.rs), [input.rs](../../crates/herogpui-components/src/input.rs), [textarea.rs](../../crates/herogpui-components/src/textarea.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [fields](../../crates/herogpui-components/tests/fields.rs), [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs), [sx_ownership](../../crates/herogpui-components/tests/sx_ownership.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/input-group/input-group.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Group.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/input-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/input-group.mdx).

**Required configuration and variant cases:** Primary/Secondary; prefix, suffix, text/icon, action button, password toggle, badge, shortcut, loading indicator; Input/TextArea children and with/without each slot.

**Required state and transition cases:** Empty/populated, hover/focus-within, invalid, disabled/read-only, required composition, pending suffix, action focus/press and clear/toggle/copy results. Verify container and child focus do not compete.

**Review and implementation work:** Match slot padding, baseline, group radius/fill and child bare chrome, including absent slots. Do not double ring or double dim the input. Prefix/suffix actions need correct hit areas and must not steal text-edit keys.

- **Anatomy to account for:** `InputGroup`, `InputGroup.Input`, `InputGroup.TextArea`, `InputGroup.Prefix`, `InputGroup.Suffix`.
- **Current metadata axes to reconcile:** `InputGroup.variant` = 'primary' | 'secondary' (default: 'primary'); `InputGroup.Input.variant` = 'primary' | 'secondary' (default: 'primary'); `InputGroup.Input.type` = string (default: 'text'); `InputGroup.TextArea.variant` = 'primary' | 'secondary' (default: 'primary').
- **Current named state rows to retain/revalidate:** Hover, Focus Within, Invalid, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Variants; In Surface; Loading State; Required Field; Disabled State; Full Width; Text Prefix; Text Suffix; Icon Prefix and Text Suffix; Copy Button Suffix; Icon Prefix and Copy Button; Password Toggle; Keyboard Shortcut; Badge Suffix; Validation; With Prefix Icon; With Suffix Icon; With Prefix and Suffix; With TextArea; Usage Example; TextArea Usage Example; Addons; With a trailing action.

### Input OTP (`input-otp`)

- **Implementation:** [input_otp.rs](../../crates/herogpui-components/src/input_otp.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [fields](../../crates/herogpui-components/tests/fields.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/input-otp/input-otp.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/input-otp.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/input-otp.mdx).

**Required configuration and variant cases:** Primary/Secondary; four/six and arbitrary supported lengths, groups/separators, custom slots, pattern, paste transformer, alignment, placeholders and controlled values. Treat inputMode as a native/browser hint according to implementation.

**Required state and transition cases:** Empty/partially filled/complete, active/hovered/filled slot, caret blink, invalid, disabled, selection/replacement, deletion across slots, paste valid/filtered/overlong input and completion callback. Include keyboard, pointer, IME/input-platform behavior and reset.

**Review and implementation work:** Review the full slot geometry/chrome ledger: current metadata marks much of it partial. Match shared borders/corners, active/invalid rings, filled background, character entrance and caret motion. Add a focused OTP behavior binary only if existing text-field tests cannot own the regression coherently.

- **Anatomy to account for:** `InputOTP`, `InputOTP.Group`, `InputOTP.Slot`, `InputOTP.Separator`.
- **Current metadata axes to reconcile:** `InputOTP.variant` = "primary" | "secondary" (default: "primary"); `InputOTP.textAlign` = 'left' | 'center' | 'right' (default: 'left').
- **Current named state rows to retain/revalidate:** Hovered, Active, Filled, Disabled, Invalid.
- **Existing gallery sections to map into specimens:** Usage; Variants; In Surface; Disabled State; Four Digits; Controlled; On Complete; Custom Slots; Form Example; With Pattern; With Validation.

### Number Field (`number-field`)

- **Implementation:** [number_field.rs](../../crates/herogpui-components/src/number_field.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [slider_number_deep](../../crates/herogpui-components/tests/slider_number_deep.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/number-field/number-field.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/NumberField.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/number-field.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/number-field.mdx).

**Required configuration and variant cases:** Primary/Secondary; both steppers, increment-only, decrement-only and neither; custom icons, steps/min/max, decimal/percent/currency/unit formatting, full width and controlled/default nullable values.

**Required state and transition cases:** Empty/intermediate/valid/invalid, focus/hover/press, required, disabled/read-only, min/max with one stepper disabled, repeat press and wheel policy. Test rounding, text commit, keyboard changes and form reset.

**Review and implementation work:** Ensure absent stepper slots reserve no columns; check current implementation before repeating the v3.2.5 fix. Match input baseline, button targets, border/corner ownership, long formatted values and disabled stepper opacity. Validate accessible behavior against the owning Aria hook.

- **Anatomy to account for:** `NumberField`, `NumberField.Group`, `NumberField.Input`, `NumberField.IncrementButton`, `NumberField.DecrementButton`, `Label`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `NumberField.variant` = "primary" | "secondary" (default: "primary"); `NumberField.validationBehavior` = 'native' | 'aria' (default: 'native'); `NumberField.Input.variant` = "primary" | "secondary" (default: "primary").
- **Current named state rows to retain/revalidate:** Invalid, Disabled, Read only, Focus within, Focus visible, Hovered, Pressed, Required.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Without steppers; Format Options; Variants; In Surface; With Description; Required Field; Disabled State; Full Width; Validation; Controlled; Step Values; Form Example; With Validation; Custom Icons; With Chevrons.

### Radio Group (`radio-group`)

- **Implementation:** [radio_group.rs](../../crates/herogpui-components/src/radio_group.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/radio-group/radio-group.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/radio-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/radio-group.mdx).

**Required configuration and variant cases:** Primary/Secondary × horizontal/vertical; text/description/custom indicator, disabled options, controlled/default value and validation. Include repository-only size/radius examples separately.

**Required state and transition cases:** Unselected/selected × hover/press/focus-visible; disabled group/option, read-only, invalid/required, no initial selection, controlled rejection and keyboard roving/selection.

**Review and implementation work:** Complete missing unselected hover indicator/background and field-border layers. Match selected dot motion, ring, group spacing, long-label alignment and indicator geometry. Verify one collection tab stop, disabled skipping and no unintended changes in read-only mode.

- **Anatomy to account for:** `RadioGroup`, `Label`, `Description`, `Radio`, `Radio.Content`, `Radio.Control`, `Radio.Indicator`, `FieldError`.
- **Current metadata axes to reconcile:** `RadioGroup.variant` = "primary" | "secondary" (default: "primary"); `RadioGroup.orientation` = "horizontal" | "vertical" (default: "vertical").
- **Current named state rows to retain/revalidate:** Selected, Hovered, Focus visible, Pressed, Disabled group, Disabled option, Read only, Invalid, Vertical, Horizontal.
- **Existing gallery sections to map into specimens:** Usage; Sizes; Variants; In Surface; Validation; Delivery & Payment; Custom Indicator; Vertical; Uncontrolled; Horizontal Orientation; Controlled; Disabled.

### Search Field (`search-field`)

- **Implementation:** [input.rs](../../crates/herogpui-components/src/input.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [fields](../../crates/herogpui-components/tests/fields.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/search-field/search-field.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/SearchField.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/search-field.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/search-field.mdx).

**Required configuration and variant cases:** Primary/Secondary; default/custom search and clear icons, shortcut suffix, description/error, full width, validation and controlled/default values.

**Required state and transition cases:** Empty/nonempty, hover/focus-within/focus-visible, invalid/required, disabled/read-only where supported, clear affordance shown/hidden and search submit. Test clear mouse-down/up/cancel and keyboard clearing exactly as the source specifies.

**Review and implementation work:** Match icon/value/clear alignment, hidden clear layout and field transition colors. Ensure a clear action reports the value/callback once, keeps the intended focus and does not incorrectly submit the form.

- **Anatomy to account for:** `SearchField`, `SearchField.Group`, `SearchField.Input`, `SearchField.SearchIcon`, `SearchField.ClearButton`.
- **Current metadata axes to reconcile:** `SearchField.variant` = "primary" | "secondary" (default: "primary"); `SearchField.validationBehavior` = "native" | "aria" (default: "native"); `SearchField.Input.variant` = "primary" | "secondary" (default: "primary"); `SearchField.Input.type` = string (default: "search").
- **Current named state rows to retain/revalidate:** Invalid, Disabled, Focus within, Focus visible, Hovered, Empty.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Variants; In Surface; With Description; Required Field; Disabled State; Full Width; Validation; Controlled; Render Props; With Validation; Form Example; Custom Icons; With Keyboard Shortcut.

### Text Area (`text-area`)

- **Implementation:** [textarea.rs](../../crates/herogpui-components/src/textarea.rs), [input.rs](../../crates/herogpui-components/src/input.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [text_size_knobs](../../crates/herogpui-components/tests/text_size_knobs.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/textarea/textarea.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/TextArea.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/textarea.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/text-area.mdx).

**Required configuration and variant cases:** Primary/Secondary; rows, resizing behavior actually supported, full/intrinsic width, controlled/default text, bare/custom box and use inside TextField/InputGroup.

**Required state and transition cases:** Empty/multiline/long lines, hover/focus, invalid, disabled/read-only, caret/selection, newline, clipboard, IME and scroll. Include resize and content changes around minimum/maximum dimensions.

**Review and implementation work:** Match line height, padding, baseline, local overflow and field chrome with multiline content. Complete state interpolation without clipping the last line or painting a second inner field background.

- **Anatomy to account for:** `TextArea`.
- **Current metadata axes to reconcile:** `TextArea.variant` = "primary" | "secondary" (default: "primary").
- **Current named state rows to retain/revalidate:** Hover, Focused, Focus visible, Invalid, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Variants; In Surface; Full Width; Controlled; Rows and Resizing.

### Text Field (`text-field`)

- **Implementation:** [input.rs](../../crates/herogpui-components/src/input.rs), [field.rs](../../crates/herogpui-components/src/field.rs), [validation.rs](../../crates/herogpui-components/src/validation.rs).
- **Gallery / reference:** [forms.rs](../../gallery/src/pages/components/forms.rs) / [metadata](../../gallery/src/pages/reference_metadata/forms.rs).
- **Focused tests to read/extend:** [text_fields](../../crates/herogpui-components/tests/text_fields.rs), [fields](../../crates/herogpui-components/tests/fields.rs), [forms_deep](../../crates/herogpui-components/tests/forms_deep.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/textfield/textfield.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/TextField.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/textfield.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28forms%29/text-field.mdx).

**Required configuration and variant cases:** Input and TextArea composition, Primary/Secondary inner variants, supported input types, label/description/error, required, full width, render props and controlled/default values; native/aria validation.

**Required state and transition cases:** Focus-within/focus-visible, valid/invalid, disabled/read-only/required, dynamic validation and helper/error replacement, form reset and programmatic updates while editing.

**Review and implementation work:** Match root label-to-field-to-message order, helper/error inset and shared field styles. Ensure replacement content receives live state and no-op validation/render props are not claimed implemented. Document TextField versus raw Input responsibilities.

- **Anatomy to account for:** `TextField`, `Label`, `Input`, `TextArea`, `Description`, `FieldError`.
- **Current metadata axes to reconcile:** `TextField.validationBehavior` = "native" | "aria" (default: "native").
- **Current named state rows to retain/revalidate:** Invalid, Disabled, Focus within, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; In Surface; With Description; Required Field; Disabled State; Full Width; Validation; Controlled; Render Props; Error Message; TextArea; Input Types.

## Layout

### Card (`card`)

- **Implementation:** [card.rs](../../crates/herogpui-components/src/card.rs).
- **Gallery / reference:** [layout.rs](../../gallery/src/pages/components/layout.rs) / [metadata](../../gallery/src/pages/reference_metadata/layout.rs).
- **Focused tests to read/extend:** [card_deep](../../crates/herogpui-components/tests/card_deep.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs), [feedback_compose](../../crates/herogpui-components/tests/feedback_compose.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/card/card.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/card.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28layout%29/card.mdx).

**Required configuration and variant cases:** Transparent/Default/Secondary/Tertiary; header/title/description/content/footer, horizontal/vertical composition, images/avatars/forms, optional sections and long content.

**Required state and transition cases:** Static card/theme change and interactive child states. Do not add v2 pressable/hoverable/blurred/card-loading props; those behaviors belong to explicit compositions.

**Review and implementation work:** Match surface, padding and typography per part, with source-correct default corner treatment. Verify content growth and footer alignment without reintroducing flex behavior that existing auto-height tests intentionally reject; test constrained height as a separate composition.

- **Anatomy to account for:** `Card`, `Card.Header`, `Card.Title`, `Card.Description`, `Card.Content`, `Card.Footer`.
- **Current metadata axes to reconcile:** `Card.variant` = "transparent" | "default" | "secondary" | "tertiary" (default: "default").
- **Existing gallery sections to map into specimens:** Usage; Variants; Horizontal Layout; With Avatar; With Images; With Form.

### Separator (`separator`)

- **Implementation:** [separator.rs](../../crates/herogpui-components/src/separator.rs).
- **Gallery / reference:** [layout.rs](../../gallery/src/pages/components/layout.rs) / [metadata](../../gallery/src/pages/reference_metadata/layout.rs).
- **Focused tests to read/extend:** [toolbar_divider_deep](../../crates/herogpui-components/tests/toolbar_divider_deep.rs), [surface_deep](../../crates/herogpui-components/tests/surface_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/separator/separator.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Separator.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/separator.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28layout%29/separator.mdx).

**Required configuration and variant cases:** Default/Secondary/Tertiary × horizontal/vertical; standalone, within Surface/Card/Toolbar and content-adjacent compositions.

**Required state and transition cases:** Static, container resize, theme change and changing sibling height. No invented interactive states.

**Review and implementation work:** Match thickness, opacity, logical size and self-stretch semantics for vertical separators. Keep color ownership independent of surrounding text where the source requires it. Preserve any genuine AccessKit role limitation without substituting an unrelated splitter role.

- **Anatomy to account for:** `Separator`.
- **Current metadata axes to reconcile:** `Separator.orientation` = 'horizontal' | 'vertical' (default: 'horizontal'); `Separator.variant` = 'default' | 'secondary' | 'tertiary' (default: 'default').
- **Existing gallery sections to map into specimens:** Usage; With Surface; With Content; Variants; Vertical.

### Surface (`surface`)

- **Implementation:** [surface.rs](../../crates/herogpui-components/src/surface.rs).
- **Gallery / reference:** [layout.rs](../../gallery/src/pages/components/layout.rs) / [metadata](../../gallery/src/pages/reference_metadata/layout.rs).
- **Focused tests to read/extend:** [surface_deep](../../crates/herogpui-components/tests/surface_deep.rs), [component_themes](../../crates/herogpui-components/tests/component_themes.rs), [full_width](../../crates/herogpui-components/tests/full_width.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/surface/surface.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/surface.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28layout%29/surface.mdx).

**Required configuration and variant cases:** Transparent/Default/Secondary/Tertiary; nested surfaces, form controls, custom spacing and normal/constrained dimensions.

**Required state and transition cases:** Static/theme change, nested foregrounds, and hover/focus/invalid states of children on every surface. Do not invent surface-level press/selection states.

**Review and implementation work:** Match source surface/foreground/shadow and default radius without adding aesthetic rounding. Preserve documented native layout conveniences while explaining differences from upstream; ensure nested field variants use the correct surface-dependent colors.

- **Anatomy to account for:** `Surface`.
- **Current metadata axes to reconcile:** `Surface.variant` = "transparent" | "default" | "secondary" | "tertiary" (default: "default"); `SurfaceContext.variant` = "transparent" | "default" | "secondary" | "tertiary" | undefined (default: —).
- **Existing gallery sections to map into specimens:** Usage; Variants; With Form Components.

### Toolbar (`toolbar`)

- **Implementation:** [toolbar.rs](../../crates/herogpui-components/src/toolbar.rs).
- **Gallery / reference:** [layout.rs](../../gallery/src/pages/components/layout.rs) / [metadata](../../gallery/src/pages/reference_metadata/layout.rs).
- **Focused tests to read/extend:** [toolbar_divider_deep](../../crates/herogpui-components/tests/toolbar_divider_deep.rs), [choice_controls_deep](../../crates/herogpui-components/tests/choice_controls_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/toolbar/toolbar.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria@3.52.0/packages/react-aria/src/toolbar/useToolbar.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/toolbar.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28layout%29/toolbar.mdx).

**Required configuration and variant cases:** Horizontal/vertical, attached/detached child groups, separators, Buttons/ToggleButtonGroup/mixed controls, nested toolbar and custom content.

**Required state and transition cases:** First/last/remembered child focus, disabled children/all disabled, arrow navigation, Tab exit, child removal, pointer re-entry and nested toolbar traversal.

**Review and implementation work:** Match compact spacing, separator stretch and child baselines. Container must not add a competing focus ring or trap; retain the tested nearest/nested toolbar ownership and verify that editable child keys remain with the child.

- **Anatomy to account for:** `Toolbar`.
- **Current metadata axes to reconcile:** `Toolbar.orientation` = "horizontal" | "vertical" (default: "horizontal").
- **Current named state rows to retain/revalidate:** Focused child, Disabled child, Last focused child, Nested toolbar as group.
- **Existing gallery sections to map into specimens:** Usage; With ButtonGroup; Horizontal; Attached; Vertical.

## Media

### Avatar (`avatar`)

- **Implementation:** [avatar.rs](../../crates/herogpui-components/src/avatar.rs).
- **Gallery / reference:** [media.rs](../../gallery/src/pages/components/media.rs) / [metadata](../../gallery/src/pages/reference_metadata/media.rs).
- **Focused tests to read/extend:** [avatar_deep](../../crates/herogpui-components/tests/avatar_deep.rs), [a11y_deep](../../crates/herogpui-components/tests/a11y_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/avatar/avatar.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/avatar.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28media%29/avatar.mdx).

**Required configuration and variant cases:** Default/Soft × Default/Accent/Success/Warning/Danger × Sm/Md/Lg; loaded image, initials/icon/custom fallback, custom image component. Group composition moved to AvatarGroup in v3.2.6.

**Required state and transition cases:** Loading, success, broken/absent image, delayed fallback where supported, image replacement, long/missing name and theme change. Focus/hover belongs to a composed trigger if the avatar is used interactively.

**Review and implementation work:** Match image crop/clipping, radius, fallback color/type, group overlap and fallback-to-image transition. Use deterministic local images to distinguish component behavior from network timing.

- **Anatomy to account for:** `Avatar`, `Avatar.Image`, `Avatar.Fallback`.
- **Current metadata axes to reconcile:** `Avatar.size` = 'sm' | 'md' | 'lg' (default: 'md'); `Avatar.color` = 'default' | 'accent' | 'success' | 'warning' | 'danger' (default: 'default'); `Avatar.variant` = 'default' | 'soft' (default: 'default'); `Avatar.Fallback.color` = 'default' | 'accent' | 'success' | 'warning' | 'danger' (default: —).
- **Existing gallery sections to map into specimens:** Usage; Fallback Content; Sizes; Colors; Variants; Avatar Group; Custom Image Component.

### AvatarGroup (`avatar-group`)

- **Implementation:** [avatar_group.rs](../../crates/herogpui-components/src/avatar_group.rs).
- **Gallery / reference:** [media.rs](../../gallery/src/pages/components/media.rs) / [metadata](../../gallery/src/pages/reference_metadata/media.rs).
- **Focused tests to read/extend:** [avatar_group_deep](../../crates/herogpui-components/tests/avatar_group_deep.rs), [avatar_deep](../../crates/herogpui-components/tests/avatar_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/avatar-group/avatar-group.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/avatar-group.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28media%29/avatar-group.mdx).

**Required configuration and variant cases:** Stacked clip/ring × Sm/Md/Lg; grid; `max` truncation with the automatic `+N` count; explicit `AvatarGroup.Count` with and without `max`; group size/color/variant flowing to direct children only, with a direct prop winning and nested avatars not inheriting.

**Required state and transition cases:** Children added/removed across `max`, image load/failure inside a stacked member, and theme change of the seam colour.

**Review and implementation work:** Upstream's `clip` overlap is a CSS radial-gradient `mask-image` crescent; pinned GPUI has no alpha masks, so the port paints the seam in the surface colour (a documented platform limitation, not transparent-seam parity). Verify the approximation on solid surfaces and record the difference over images/gradients.

- **Anatomy to account for:** `AvatarGroup`, `AvatarGroup.Count`, direct `Avatar` children.
- **Existing gallery sections to map into specimens:** Usage; Max; With Count; Sizes; Grid; Overlap.

## Navigation

### Accordion (`accordion`)

- **Implementation:** [accordion.rs](../../crates/herogpui-components/src/accordion.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [table_tabs_accordion](../../crates/herogpui-components/tests/table_tabs_accordion.rs), [nav_deep](../../crates/herogpui-components/tests/nav_deep.rs), [hover_overrides](../../crates/herogpui-components/tests/hover_overrides.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/accordion/accordion.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Disclosure.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/accordion.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/accordion.mdx).

**Required configuration and variant cases:** Default/Surface; single/multiple expanded, separators on/off, subtitle/custom indicator, disabled items and controlled/default keys; short/long/multiline content.

**Required state and transition cases:** All closed, one/many open, opening/closing, interrupted/reversed height changes, hovered closed/open trigger, keyboard focus, disabled and controlled rejection.

**Review and implementation work:** The built-in down-chevron now uses the shared keyed 250ms default-curve rotation primitive with a reduced-motion endpoint; custom indicator content remains caller-owned. The shared panel helper measures natural content height, animates height and opacity over the pinned 200ms curves, retains closing content through the exit lifetime, and snaps under reduced motion. Match the exact closed-trigger hover mix and custom hover-leave behavior from v3.2.5. Verify content reflow and focus safety during collapse.

- **Anatomy to account for:** `Accordion`, `Accordion.Item`, `Accordion.Heading`, `Accordion.Trigger`, `Accordion.Indicator`, `Accordion.Panel`, `Accordion.Body`.
- **Current metadata axes to reconcile:** `Accordion.variant` = "default" | "surface" (default: "default").
- **Current named state rows to retain/revalidate:** Expanded, Hovered, Focus visible, Disabled, Single expansion, Multiple expansion.
- **Existing gallery sections to map into specimens:** Usage; Hover Colour; Without Separator; Multiple Expanded; Disabled State; Controlled; Custom Indicator; FAQ Layout; Default; Surface; Hidden separator.

### Breadcrumbs (`breadcrumbs`)

- **Implementation:** [breadcrumbs.rs](../../crates/herogpui-components/src/breadcrumbs.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [nav_deep](../../crates/herogpui-components/tests/nav_deep.rs), [link_deep](../../crates/herogpui-components/tests/link_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/breadcrumbs/breadcrumbs.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/breadcrumbs.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/breadcrumbs.mdx).

**Required configuration and variant cases:** One/two/many levels, default/custom separators, disabled trail/item, long labels, current item and navigation destinations.

**Required state and transition cases:** Link hover/press/focus-visible, current and disabled states, keyboard traversal and route change. Current item should expose the actual supported semantics rather than pretending to be another link.

**Review and implementation work:** Match text/separator spacing, muted/current emphasis, baseline and wrapping/truncation. Check RTL separator direction as an explicit capability case; do not claim native RTL from a browser-only visual pass.

- **Anatomy to account for:** `Breadcrumbs`, `Breadcrumbs.Item`.
- **Current named state rows to retain/revalidate:** Current, Hover, Focus, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Navigation Levels; Disabled State; Custom Separator; Separators.

### Disclosure (`disclosure`)

- **Implementation:** [disclosure.rs](../../crates/herogpui-components/src/disclosure.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [nav_deep](../../crates/herogpui-components/tests/nav_deep.rs), [table_tabs_accordion](../../crates/herogpui-components/tests/table_tabs_accordion.rs), [render_props](../../crates/herogpui-components/tests/render_props.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/disclosure/disclosure.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/disclosure-group/disclosure-group.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Disclosure.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/disclosure.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/disclosure-group.css) · [docs 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/disclosure.mdx) · [docs 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/disclosure-group.mdx).

**Required configuration and variant cases:** Disclosure: default/custom trigger/body, controlled/default expanded, disabled and render function. DisclosureGroup is a separate required component: single/multiple expansion, default/controlled key sets, disabled group and item composition.

**Required state and transition cases:** Each component needs collapsed/expanding/expanded/collapsing, pointer/keyboard activation, focus, disabled, controlled rejection, rapid reversal and item removal; groups additionally need cross-item expansion/focus behavior.

**Review and implementation work:** The built-in indicator now uses the shared keyed 250ms default-curve rotation primitive with a reduced-motion endpoint. The shared panel helper now provides source-correct measured height and opacity motion, retains the body through the 200ms close lifetime, and snaps under reduced motion. Match Tertiary Button trigger styling and heading composition. Make the group independently discoverable and document its key ownership.

- **Anatomy to account for:** `DisclosureGroup`, `Disclosure`, `Disclosure.Heading`, `Disclosure.Trigger`, `Disclosure.Indicator`, `Disclosure.Content`, `Disclosure.Body`.
- **Current named state rows to retain/revalidate:** Expanded, Disabled, Focus visible, Single expansion, Multiple expansion.
- **Existing gallery sections to map into specimens:** Usage; Render Function; Controlled; Single; Group; Disabled Group.

### Link (`link`)

- **Implementation:** [link.rs](../../crates/herogpui-components/src/link.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [link_deep](../../crates/herogpui-components/tests/link_deep.rs), [focus_visible_deep](../../crates/herogpui-components/tests/focus_visible_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/link/link.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/link.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/link.mdx).

**Required configuration and variant cases:** Default/custom icon, start/end icon placement, text decoration through supported styling, inline and long/multiline links, URL/action semantics.

**Required state and transition cases:** Hover/pressed/focus-visible/disabled; pointer and keyboard activation, modifier handling and repeated clicks. Verify disabled navigation and activation from the actual native link contract.

**Review and implementation work:** Match underline/foreground/icon state and inline baseline/wrapping. Do not re-add v2 underline/isExternal props; preserve native URL-opening behavior and explain platform-specific navigation.

- **Anatomy to account for:** `Link`, `Link.Icon`.
- **Current named state rows to retain/revalidate:** Hovered, Pressed, Focus visible, Disabled.
- **Existing gallery sections to map into specimens:** Usage; Icon Placement; Text Decoration; Custom Icon; Render Function.

### Pagination (`pagination`)

- **Implementation:** [pagination.rs](../../crates/herogpui-components/src/pagination.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [parts_pagination](../../crates/herogpui-components/tests/parts_pagination.rs), [nav_deep](../../crates/herogpui-components/tests/nav_deep.rs), [radius_builders](../../crates/herogpui-components/tests/radius_builders.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/pagination/pagination.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/pagination.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/pagination.mdx).

**Required configuration and variant cases:** Sm/Md/Lg; previous/next, numbered pages, ellipses, summary/custom icons/render props, hidden controls and controlled/default page.

**Required state and transition cases:** First/middle/last/current page, hover/press/focus-visible, disabled group/link, page-count growth/shrink and controlled rejection. Include one-page and supported empty-data compositions.

**Review and implementation work:** Match current page treatment, item target/spacing, ellipsis alignment and edge control disabled styles. Preserve per-size press geometry and custom sx corners. Verify focus after page-count changes without inventing focus on an ellipsis.

- **Anatomy to account for:** `Pagination`, `Pagination.Summary`, `Pagination.Content`, `Pagination.Item`, `Pagination.Link`, `Pagination.Previous`, `Pagination.PreviousIcon`, `Pagination.Next`, `Pagination.NextIcon`, `Pagination.Ellipsis`.
- **Current metadata axes to reconcile:** `Pagination.size` = "sm" | "md" | "lg" (default: "md").
- **Current named state rows to retain/revalidate:** Active page, Hovered, Focused, Disabled, Pressed.
- **Existing gallery sections to map into specimens:** Usage; Hover Colour; Sizes; Disabled; Disabled Links; Simple (Previous / Next); Controlled; With Ellipsis; With Summary; Render Props; Custom Icons; Without controls.

### Tabs (`tabs`)

- **Implementation:** [tabs.rs](../../crates/herogpui-components/src/tabs.rs), [scroll_shadow.rs](../../crates/herogpui-components/src/scroll_shadow.rs).
- **Gallery / reference:** [navigation.rs](../../gallery/src/pages/components/navigation.rs) / [metadata](../../gallery/src/pages/reference_metadata/navigation.rs).
- **Focused tests to read/extend:** [tabs_deep](../../crates/herogpui-components/tests/tabs_deep.rs), [table_tabs_accordion](../../crates/herogpui-components/tests/table_tabs_accordion.rs), [scroll_shadow_deep](../../crates/herogpui-components/tests/scroll_shadow_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/tabs/tabs.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/tabs.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28navigation%29/tabs.mdx).

**Required configuration and variant cases:** Primary/Secondary × horizontal/vertical × Start/Center/End alignment; intrinsic/full width, separators, overflow chevrons, nested tabs, disabled tabs, controlled/default selection and actual supported activation policy. Test repository size extensions separately.

**Required state and transition cases:** Unselected/selected/hovered/focus-visible/disabled, selection/panel switch, dynamic tab removal, overflow start/middle/end, narrow resize and rapid indicator reversal. Test pointer and keyboard selection while preserving the chosen focus-modality rule.

**Review and implementation work:** Keep the implemented align API and evidence; constrained vertical label wrapping now matches the pinned 32px tab box, with overflow wrap painting past the pill. Complete remaining state/indicator interpolation. Verify nested alignment isolation, correct selected-panel sizing and scroll limits; do not mark the entire component complete from the alignment demo.

- **Anatomy to account for:** `Tabs`, `Tabs.ListContainer`, `Tabs.List`, `Tabs.Tab`, `Tabs.Separator`, `Tabs.Indicator`, `Tabs.Panel`.
- **Current metadata axes to reconcile:** `Tabs.align` = "start" | "center" | "end" (default: "center"); `Tabs.variant` = "primary" | "secondary" (default: "primary"); `Tabs.orientation` = "horizontal" | "vertical" (default: "horizontal").
- **Current named state rows to retain/revalidate:** Selected, Hovered, Focus visible, Disabled, Controlled selection, Uncontrolled selection.
- **Existing gallery sections to map into specimens:** Usage; Sizes; Vertical; Full Width; Alignment; Overflow; Disabled Tab; With Separator; Secondary Variant; Secondary Variant Vertical; Primary.

## Overlays

### Alert Dialog (`alert-dialog`)

- **Implementation:** [alert_dialog.rs](../../crates/herogpui-components/src/alert_dialog.rs), [modal.rs](../../crates/herogpui-components/src/modal.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [overlays](../../crates/herogpui-components/tests/overlays.rs), [overlay_stack_dialogs_deep](../../crates/herogpui-components/tests/overlay_stack_dialogs_deep.rs), [overlay_padding](../../crates/herogpui-components/tests/overlay_padding.rs), [panel_padding](../../crates/herogpui-components/tests/panel_padding.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/alert-dialog/alert-dialog.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/alert-dialog.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/alert-dialog.mdx).

**Required configuration and variant cases:** Xs/Sm/Md/Lg/Cover; Auto/Center/Top/Bottom; Opaque/Blur/Transparent backdrops; status/custom icon, title/description/body/footer, custom trigger/close content and supported dismissal settings.

**Required state and transition cases:** Closed/entering/open/exiting, focus trap/initial focus/restore, pointer/keyboard actions, Escape/outside rules, controlled changes, nested popup/dialog, interruption and reduced motion. Use source defaults for alert dismissal, not generic Modal assumptions.

**Review and implementation work:** Match constrained panel/body/footer geometry, backdrop and status icon hierarchy, placement-specific motion and close target. Resolve portable trigger/header/icon composition gaps; retain an explicit blur limitation if the pinned renderer cannot reproduce it.

- **Anatomy to account for:** `AlertDialog`, `AlertDialog.Trigger`, `AlertDialog.Backdrop`, `AlertDialog.Container`, `AlertDialog.Dialog`, `AlertDialog.Header`, `AlertDialog.Heading`, `AlertDialog.Body`, `AlertDialog.Footer`, `AlertDialog.Icon`, `AlertDialog.CloseTrigger`.
- **Current metadata axes to reconcile:** `AlertDialog.Backdrop.variant` = "opaque" | "blur" | "transparent" (default: "opaque"); `AlertDialog.Container.placement` = "auto" | "center" | "top" | "bottom" (default: "auto"); `AlertDialog.Container.size` = "xs" | "sm" | "md" | "lg" | "cover" (default: "md"); `AlertDialog.Icon.status` = "default" | "accent" | "success" | "warning" | "danger" (default: "danger").
- **Current named state rows to retain/revalidate:** Focus, Hover, Active, Entering, Exiting, Placement.
- **Existing gallery sections to map into specimens:** Usage; Sizes; Statuses; Placements; Backdrop Variants; Controlled State; Custom Icon; Custom Backdrop; Dismiss Behavior; Close Methods; Custom Animations; Custom Trigger.

### Drawer (`drawer`)

- **Implementation:** [drawer.rs](../../crates/herogpui-components/src/drawer.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [drawer_deep](../../crates/herogpui-components/tests/drawer_deep.rs), [overlay_stack_dialogs_deep](../../crates/herogpui-components/tests/overlay_stack_dialogs_deep.rs), [panel_padding](../../crates/herogpui-components/tests/panel_padding.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/drawer/drawer.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/drawer.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/drawer.mdx).

**Required configuration and variant cases:** Top/Bottom/Left/Right × Opaque/Blur/Transparent; dismissable/nondismissable, scrollable content, forms, custom body/footer/trigger, handle composition and nested drawers.

**Required state and transition cases:** Closed/entering/open/exiting, drag start/move/release/cancel/snapback, rapid reversal, focus trap/restore, Escape/outside policies and controlled open. A nested drawer handle must move and dismiss only its nearest owning drawer.

**Review and implementation work:** Match placement-specific dimensions/corners, viewport cap, handle and body spacing; use the Drawer’s own backdrop/panel timing instead of generic Modal timing. Verify scrolling does not accidentally start dismissal and resized windows do not strand the panel.

- **Anatomy to account for:** `Drawer`, `Drawer.Trigger`, `Drawer.Backdrop`, `Drawer.Content`, `Drawer.Dialog`, `Drawer.Header`, `Drawer.Heading`, `Drawer.Body`, `Drawer.Footer`, `Drawer.Handle`, `Drawer.CloseTrigger`.
- **Current metadata axes to reconcile:** `Drawer.Backdrop.variant` = "opaque" | "blur" | "transparent" (default: "opaque"); `Drawer.Content.placement` = "top" | "bottom" | "left" | "right" (default: "bottom").
- **Current named state rows to retain/revalidate:** Focus, Hover, Active, Entering, Exiting, Placement.
- **Existing gallery sections to map into specimens:** Placement; Non-Dismissable; Scrollable Content; Controlled State; With Form; Navigation Drawer; Backdrop Variants; Usage.

### Modal (`modal`)

- **Implementation:** [modal.rs](../../crates/herogpui-components/src/modal.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [overlays](../../crates/herogpui-components/tests/overlays.rs), [overlay_stack_dialogs_deep](../../crates/herogpui-components/tests/overlay_stack_dialogs_deep.rs), [overlay_padding](../../crates/herogpui-components/tests/overlay_padding.rs), [panel_padding](../../crates/herogpui-components/tests/panel_padding.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/modal/modal.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/modal.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/modal.mdx).

**Required configuration and variant cases:** Xs/Sm/Md/Lg/Cover/Full × Auto/Center/Top/Bottom; inside/outside scrolling; Opaque/Blur/Transparent backdrop; custom trigger/backdrop/close content, form and long-body composition.

**Required state and transition cases:** Closed/entering/open/exiting, initial focus/trap/restore, Escape/outside dismissal policies, controlled changes, nested popups/dialogs, submit/error state, interrupted open/close and reduced motion. Verify backdrop dismissal with outside scrolling.

**Review and implementation work:** Match panel max sizes, header/body/footer padding, close placement and source-specific enter/exit including top/bottom translation. Verify an exiting popup cannot paint/intercept over the modal, and long content remains reachable in short windows.

- **Anatomy to account for:** `Modal`, `Modal.Trigger`, `Modal.Backdrop`, `Modal.Container`, `Modal.Dialog`, `Modal.Header`, `Modal.Icon`, `Modal.Heading`, `Modal.Body`, `Modal.Footer`, `Modal.CloseTrigger`.
- **Current metadata axes to reconcile:** `Modal.Backdrop.variant` = "opaque" | "blur" | "transparent" (default: "opaque"); `Modal.Container.placement` = "auto" | "center" | "top" | "bottom" (default: "auto"); `Modal.Container.scroll` = "inside" | "outside" (default: "inside"); `Modal.Container.size` = "xs" | "sm" | "md" | "lg" | "cover" | "full" (default: "md").
- **Current named state rows to retain/revalidate:** Focus, Hover, Active, Entering, Exiting, Placement.
- **Existing gallery sections to map into specimens:** Sizes; Placement; Scroll Behavior; Controlled State; With Form; Custom Trigger; Backdrop Variants; Custom Backdrop; Dismiss Behavior; Close Methods; Custom Animations; Usage.

### Popover (`popover`)

- **Implementation:** [popover.rs](../../crates/herogpui-components/src/popover.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [popover_stack_deep](../../crates/herogpui-components/tests/popover_stack_deep.rs), [placement](../../crates/herogpui-components/tests/placement.rs), [placement_extra](../../crates/herogpui-components/tests/placement_extra.rs), [overlay_stack_dialogs_deep](../../crates/herogpui-components/tests/overlay_stack_dialogs_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/popover/popover.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Popover.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/popover.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/popover.mdx).

**Required configuration and variant cases:** Every placement/alignment supported by the pinned Placement type, offsets, arrow on/off, custom trigger/content/render function, interactive content and nested overlays.

**Required state and transition cases:** Closed/entering/open/exiting, pointer/keyboard trigger, focus inside, Escape/outside dismissal, resize/scroll/flip, interrupted transitions and reopening. Test interactive child controls without unintended parent dismissal.

**Review and implementation work:** Match arrow shape/location, panel radius/shadow, transform origin, placement-specific translation and content spacing. Verify union bounds, topmost ownership and focus return; include long/wrapping content near every viewport edge.

- **Anatomy to account for:** `Popover`, `Popover.Trigger`, `Popover.Content`, `Popover.Arrow`, `Popover.Dialog`, `Popover.Heading`.
- **Current metadata axes to reconcile:** `Popover.Content.placement` = Placement (default: "bottom").
- **Current named state rows to retain/revalidate:** Entering, Exiting, Placement, Focus visible.
- **Existing gallery sections to map into specimens:** Usage; With Arrow; Interactive Content; Placement; Render Function; Custom Styles.

### Toast (`toast`)

- **Implementation:** [toast.rs](../../crates/herogpui-components/src/toast.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [toast_stack_deep](../../crates/herogpui-components/tests/toast_stack_deep.rs), [toast_update_deep](../../crates/herogpui-components/tests/toast_update_deep.rs), [toast_promise_deep](../../crates/herogpui-components/tests/toast_promise_deep.rs), [virtual_and_feedback](../../crates/herogpui-components/tests/virtual_and_feedback.rs), [a11y_overlays_deep](../../crates/herogpui-components/tests/a11y_overlays_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/toast/toast.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/toast/toast-queue.ts) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-stately/src/toast/useToastState.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/toast.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/toast.mdx).

**Required configuration and variant cases:** Default/Accent/Success/Warning/Danger × top-start/top/top-end/bottom-start/bottom/bottom-end; one/multiple/overflow queues, persistent/timed, action/close/custom indicator/rendering, loading/promise, configurable hotkey/exitDuration/isExpanded.

**Required state and transition cases:** Queued/frontmost/collapsed/expanded/hidden/exiting; hover/focus expansion, timer pause/resume on hover/focus/background, update retaining omitted options, reset restarting time, promise success/error, click/action/close exactly once and queue cleanup. Exercise exact Alt+T modifiers, Escape/F6 behavior from source, narrow width remeasurement and reduced motion.

**Review and implementation work:** Do not reimplement APIs already present. Compare real absolute overlap, front-height clipping, depth transforms, stack direction/gaps, viewport width, close visibility/position and region focus. Replace remaining flex-stack approximations where feasible; runtime captures must actually contain toasts, including every placement.

- **Anatomy to account for:** `Toast.Provider`, `Toast`, `Toast.Indicator`, `Toast.Content`, `Toast.Title`, `Toast.Description`, `Toast.ActionButton`, `Toast.CloseButton`.
- **Current metadata axes to reconcile:** `Toast.Provider.placement` = top start | top | top end | bottom start | bottom | bottom end (default: bottom); `Toast.variant` = default | accent | success | warning | danger (default: default); `Toast.placement` = ToastPlacement (default: Provider); `Toast.Indicator.variant` = ToastVariant (default: Toast variant); `toast Function.variant` = ToastVariant (default: default).
- **Current named state rows to retain/revalidate:** Frontmost, Index, Placement, Hidden, Expanded, Exiting, Loading.
- **Existing gallery sections to map into specimens:** Usage; Variants; Placements; Simple Toasts; Custom Indicators; Custom Toast Rendering; Promise & Loading; Callbacks; Expanded Stack; Custom Queues; Setup; Push a toast.

### Tooltip (`tooltip`)

- **Implementation:** [tooltip.rs](../../crates/herogpui-components/src/tooltip.rs).
- **Gallery / reference:** [overlays.rs](../../gallery/src/pages/components/overlays.rs) / [metadata](../../gallery/src/pages/reference_metadata/overlays.rs).
- **Focused tests to read/extend:** [overlays](../../crates/herogpui-components/tests/overlays.rs), [popover_stack_deep](../../crates/herogpui-components/tests/popover_stack_deep.rs), [placement_extra](../../crates/herogpui-components/tests/placement_extra.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/tooltip/tooltip.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/tooltip.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28overlays%29/tooltip.mdx).

**Required configuration and variant cases:** Hover/focus trigger modes, all pinned placement/alignment values, arrow on/off, offset, delay/closeDelay, custom triggers, skip-animation and disabled.

**Required state and transition cases:** Cold delay, warm sibling sequence, one-open-tooltip arbitration, hover leave/re-entry, focus opening, Escape/dismissal, entering/open/exiting, disabled and reduced motion. Test switching triggers during delays and unmounting an active trigger.

**Review and implementation work:** Match content typography, arrow, panel shadow, offset and full placement-specific motion. The port now supports the eight pinned top/bottom cardinal and start/end placements; continue checking edge-only placements and transform-origin limitations separately rather than flattening aligned placements to one side.

- **Anatomy to account for:** `Tooltip`, `Tooltip.Trigger`, `Tooltip.Content`, `Tooltip.Arrow`.
- **Current metadata axes to reconcile:** `Tooltip.trigger` = "hover" | "focus" (default: "hover"); `Tooltip.Content.placement` = Placement (default: "top").
- **Current named state rows to retain/revalidate:** Global sequence, Entering, Exiting, Placement.
- **Existing gallery sections to map into specimens:** With Arrow; Custom Triggers; Usage; Placement; Delay.

## Pickers

### Autocomplete (`autocomplete`)

- **Implementation:** [autocomplete.rs](../../crates/herogpui-components/src/autocomplete.rs), [picker_item.rs](../../crates/herogpui-components/src/picker_item.rs), [list_nav.rs](../../crates/herogpui-components/src/list_nav.rs).
- **Gallery / reference:** [pickers.rs](../../gallery/src/pages/components/pickers.rs) / [metadata](../../gallery/src/pages/reference_metadata/pickers.rs).
- **Focused tests to read/extend:** [autocomplete_hover_deep](../../crates/herogpui-components/tests/autocomplete_hover_deep.rs), [pickers_deep](../../crates/herogpui-components/tests/pickers_deep.rs), [overlay_stack_pickers_deep](../../crates/herogpui-components/tests/overlay_stack_pickers_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/autocomplete/autocomplete.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Autocomplete.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-stately@3.50.0/packages/react-stately/src/autocomplete/useAutocompleteState.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/autocomplete.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28pickers%29/autocomplete.mdx).

**Required configuration and variant cases:** Primary/Secondary; single/multiple, controlled/default selection/open state, disabled options, sections, empty-collection policy, virtualized/async filtering, clear/custom value/custom indicator and constrained/full width.

**Required state and transition cases:** Closed/opening/open/closing; query empty/filtered/no results/loading/error composition, hover/focus, selected/disabled options, invalid/required/read-only/disabled root, clear/remove, controlled rejection and keyboard navigation. Verify the query lives in the upstream-owned part rather than borrowing ComboBox anatomy.

**Review and implementation work:** Match trigger/query/tag/list composition, wrapped selected values, selected/focused option chrome, clear hit area/press/fade, indicator rotation and panel placement motion. Recheck lower option reachability and trigger hover exclusion around clear controls.

- **Anatomy to account for:** `Autocomplete`, `Autocomplete.Trigger`, `Autocomplete.Value`, `Autocomplete.Indicator`, `Autocomplete.ClearButton`, `Autocomplete.Popover`, `Autocomplete.Filter`.
- **Current metadata axes to reconcile:** `Autocomplete.selectionMode` = "single" | "multiple" (default: "single"); `Autocomplete.variant` = "primary" | "secondary" (default: "primary"); `Autocomplete.Popover.placement` = PopoverPlacement (default: "bottom").
- **Current named state rows to retain/revalidate:** Open, Hovered, Focus visible, Disabled, Read only, Invalid, Required, Placeholder, Empty selection, Multiple selection, Empty collection, Entering, Exiting.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Virtualization; Variants; In Surface; Full Width; With Description; Required; Disabled; With Disabled Options; Allows Empty Collection; With Sections; Multiple Select; Controlled; Controlled Multiple; Controlled Open State; Asynchronous Filtering; Custom Indicator; Custom Value.

### Combo Box (`combo-box`)

- **Implementation:** [combo_box.rs](../../crates/herogpui-components/src/combo_box.rs), [picker_item.rs](../../crates/herogpui-components/src/picker_item.rs), [list_nav.rs](../../crates/herogpui-components/src/list_nav.rs).
- **Gallery / reference:** [pickers.rs](../../gallery/src/pages/components/pickers.rs) / [metadata](../../gallery/src/pages/reference_metadata/pickers.rs).
- **Focused tests to read/extend:** [combo_box_open](../../crates/herogpui-components/tests/combo_box_open.rs), [pickers_deep](../../crates/herogpui-components/tests/pickers_deep.rs), [field_keyboard_contracts](../../crates/herogpui-components/tests/field_keyboard_contracts.rs), [overlay_stack_pickers_deep](../../crates/herogpui-components/tests/overlay_stack_pickers_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/combo-box/combo-box.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/ComboBox.tsx) · [API/source 3](https://github.com/adobe/react-spectrum/blob/react-aria@3.52.0/packages/react-aria/src/combobox/useComboBox.ts) · [API/source 4](https://github.com/adobe/react-spectrum/blob/react-aria@3.52.0/packages/react-aria/src/i18n/useFilter.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/combo-box.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28pickers%29/combo-box.mdx).

**Required configuration and variant cases:** Primary/Secondary; single/multiple, controlled/default selection and input text, supported menu-trigger policies, custom values/filter, disabled options/sections, async/virtualized data, form value and validation modes; every supported placement.

**Required state and transition cases:** Empty/querying/filtered/no results/loading, selected/custom value/multiple tags, input-focused versus trigger-hover/press, open/enter/exit, disabled/read-only/required/invalid, option navigation, commit/revert/clear, async stale responses and controlled ownership.

**Review and implementation work:** Match input+button+popup anatomy, trigger text/pressed treatment, input/tag wrapping, indicator and option styles. The built-in trigger chevron now uses the keyed 150ms rotation, and the popup retains a visual-only 100ms exit without blocking later pointer targets. Review locale-sensitive filtering limits and input accessibility ownership using internal seams where possible; do not add an invented public role prop to compensate for an implementation boundary.

- **Anatomy to account for:** `ComboBox`, `ComboBox.InputGroup`, `ComboBox.Value`, `ComboBox.Trigger`, `ComboBox.Popover`.
- **Current metadata axes to reconcile:** `ComboBox.selectionMode` = "single" | "multiple" (default: "single"); `ComboBox.validationBehavior` = "native" | "aria" (default: "native"); `ComboBox.menuTrigger` = "focus" | "input" | "manual" (default: "focus"); `ComboBox.variant` = "primary" | "secondary" (default: "primary"); `ComboBox.Popover.placement` = "bottom" | "bottom left" | "bottom right" | "bottom start" | "bottom end" | "top" | "top left" | "top right" | "top start" | "top end" | "left" | "left top" | "left bottom" | "start" | "start top" | "start bottom" | "right" | "right top" | "right bottom" | "end" | "end top" | "end bottom" (default: "bottom").
- **Current named state rows to retain/revalidate:** Open, Focused, Hovered trigger, Focus visible trigger, Pressed trigger, Disabled, Read only, Invalid, Required, Filtered, Empty collection, Disabled option, Selected option, Multiple selection, Custom value, Menu trigger, Focus wrap, Entering, Exiting.
- **Existing gallery sections to map into specimens:** Usage; Box Customisation; Virtualization; Full Width; With Description; Required; Disabled; Read Only; In Surface; With Disabled Options; With Sections; Controlled; Controlled Input Value; Controlled Selection; Multiple Selection; Value Render Props; Default Selected Key; Allows Custom Value; Asynchronous Loading; Custom Indicator; Custom Filtering; Menu Trigger; Form Value; Validation Behavior; Custom Validation; Custom Value; Basic Usage.

### Select (`select`)

- **Implementation:** [select.rs](../../crates/herogpui-components/src/select.rs), [picker_item.rs](../../crates/herogpui-components/src/picker_item.rs), [list_nav.rs](../../crates/herogpui-components/src/list_nav.rs).
- **Gallery / reference:** [pickers.rs](../../gallery/src/pages/components/pickers.rs) / [metadata](../../gallery/src/pages/reference_metadata/pickers.rs).
- **Focused tests to read/extend:** [select_clear_deep](../../crates/herogpui-components/tests/select_clear_deep.rs), [pickers_deep](../../crates/herogpui-components/tests/pickers_deep.rs), [overlay_stack_pickers_deep](../../crates/herogpui-components/tests/overlay_stack_pickers_deep.rs), [focus_visible_deep](../../crates/herogpui-components/tests/focus_visible_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/select/select.tsx) · [API/source 2](https://github.com/adobe/react-spectrum/blob/react-aria-components@1.21.0/packages/react-aria-components/src/Select.tsx) · [styles 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/select.css) · [styles 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/list-box.css) · [styles 3](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/list-box-item.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28pickers%29/select.mdx).

**Required configuration and variant cases:** Primary/Secondary; single/multiple, placeholder/value/custom value, sections/disabled options, virtualized/async collection, controlled/default selection/open state, clear part absent/present/custom, full/intrinsic and every supported popup placement.

**Required state and transition cases:** Empty/selected/multiple, hover/press/focus-visible, required/invalid/disabled and closed/enter/open/exit; option selected/focused/disabled, clear shown/hidden/reappearing, release/cancel/secondary buttons, closed Backspace/Delete and open-popup keyboard behavior. Include required clearing, whole-set multiple clear, callback ownership and controlled rejection.

**Review and implementation work:** Preserve the implemented ClearButton contract and evidence. Correct option-selected extra accent/weight and focused-option border if they still differ; support source-correct long-value wrapping. The built-in trigger indicator now uses one keyed down-chevron with the pinned 150ms rotation, and `trigger_indicator` exposes the live open state for custom content. Continue with placement slide and clear custom-child/pressed-target geometry; verify trigger hover exclusion and hidden retained layout. The 24px pointer target must remain stable while its 20px visual keeps the resolved radius. A caller-provided fixed-size `AnyElement` cannot inherit a subtree scale on pinned GPUI 0.3.3; either add a proven render-prop seam or keep that case explicitly measured as a framework limitation rather than claiming transform parity.

- **Anatomy to account for:** `Select`, `Select.Trigger`, `Select.Value`, `Select.Indicator`, `Select.ClearButton`, `Select.Popover`.
- **Current metadata axes to reconcile:** `Select.selectionMode` = "single" | "multiple" (default: "single"); `Select.variant` = "primary" | "secondary" (default: "primary"); `Select.Popover.placement` = Placement (default: "bottom").
- **Current named state rows to retain/revalidate:** Hovered trigger, Focus visible, Disabled, Invalid, Placeholder, Open indicator, Entering, Exiting, Selected option, Focused option, Pressed option, Disabled option, Multiple selection.
- **Existing gallery sections to map into specimens:** Usage; With Clear Button; Box Customisation; Virtualization; With Description; Required; Disabled; With Disabled Options; With Sections; In Surface; Controlled Multiple; Controlled Open State; Asynchronous Loading; Custom Item Indicator; Custom Trigger Indicator; Custom Value; Uncontrolled; Variants; Full Width; Multiple Select; Controlled.

## Typography

### Kbd (`kbd`)

- **Implementation:** [kbd.rs](../../crates/herogpui-components/src/kbd.rs).
- **Gallery / reference:** [typography.rs](../../gallery/src/pages/components/typography.rs) / [metadata](../../gallery/src/pages/reference_metadata/typography.rs).
- **Focused tests to read/extend:** [kbd_deep](../../crates/herogpui-components/tests/kbd_deep.rs), [typography_deep](../../crates/herogpui-components/tests/typography_deep.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/kbd/kbd.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/kbd.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28typography%29/kbd.mdx).

**Required configuration and variant cases:** Default/Light, modifier/special/navigation keys, one/multiple keys and inline instructional text; supported platform key naming and custom key content.

**Required state and transition cases:** Static display and theme/platform label changes. Kbd is an instruction display, not a pressable key control.

**Review and implementation work:** Match cap height, padding, radius, border/shadow, mono font and spacing between combined keys. Align naturally in text and keep narrow/long shortcut rows readable.

- **Anatomy to account for:** `Kbd`, `Kbd.Abbr`, `Kbd.Content`.
- **Current metadata axes to reconcile:** `Kbd.variant` = "default" | "light" (default: "default").
- **Existing gallery sections to map into specimens:** Navigation Keys; Special Keys; Inline Usage; Instructional Text; Usage; Variants.

### Typography (`typography`)

- **Implementation:** [typography.rs](../../crates/herogpui-components/src/typography.rs).
- **Gallery / reference:** [typography.rs](../../gallery/src/pages/components/typography.rs) / [metadata](../../gallery/src/pages/reference_metadata/typography.rs).
- **Focused tests to read/extend:** [typography_deep](../../crates/herogpui-components/tests/typography_deep.rs), [text_size_knobs](../../crates/herogpui-components/tests/text_size_knobs.rs), [font_seams](../../crates/herogpui-components/tests/font_seams.rs).
- **Pinned upstream:** [API/source](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/typography/typography.tsx) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/typography.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28typography%29/typography.mdx).

**Required configuration and variant cases:** H1/H2/H3/H4/H5/H6/Body/BodySm/BodyXs/Code; Default/Muted; Start/Center/End/Justify; supported weights/truncation; paragraph Base/Sm/Xs, primitive and Prose composition.

**Required state and transition cases:** Static short/long/multiline/empty text, truncation, resize, zoom, mixed-script and theme changes. Interactive links/code-copy actions follow their own component contracts.

**Review and implementation work:** Match type scale, weight, leading, paragraph rhythm, list/code/quote composition where present, line wrapping and baseline. Make normalized fixture fonts and intentional platform typography explicit; do not call a font mismatch a spacing defect.

- **Anatomy to account for:** `Typography`, `Typography.Prose`.
- **Current metadata axes to reconcile:** `Typography.type` = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'body' | 'body-sm' | 'body-xs' | 'code' (default: 'body'); `Typography.align` = 'start' | 'center' | 'end' | 'justify' (default: 'start'); `Typography.color` = 'default' | 'muted' (default: 'default'); `Typography.Paragraph.size` = 'base' | 'sm' | 'xs' (default: 'base').
- **Existing gallery sections to map into specimens:** Usage; Render Props; Scale; Colors & weights; Alignment & truncation; Primitives; Prose.

## Utilities

### Scroll Shadow (`scroll-shadow`)

- **Implementation:** [scroll_shadow.rs](../../crates/herogpui-components/src/scroll_shadow.rs), [scrollbar.rs](../../crates/herogpui-components/src/scrollbar.rs).
- **Gallery / reference:** [utilities.rs](../../gallery/src/pages/components/utilities.rs) / [metadata](../../gallery/src/pages/reference_metadata/utilities.rs).
- **Focused tests to read/extend:** [scroll_shadow_deep](../../crates/herogpui-components/tests/scroll_shadow_deep.rs), [tabs_deep](../../crates/herogpui-components/tests/tabs_deep.rs).
- **Pinned upstream:** [API/source 1](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/scroll-shadow/scroll-shadow.tsx) · [API/source 2](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/react/src/components/scroll-shadow/use-scroll-shadow.ts) · [styles](https://github.com/heroui-inc/heroui/blob/v3.2.6/packages/styles/components/scroll-shadow.css) · [docs](https://github.com/heroui-inc/heroui/blob/v3.2.6/apps/docs/content/docs/en/react/components/%28utilities%29/scroll-shadow.mdx).

**Required configuration and variant cases:** Vertical/Horizontal; Fade variant; shadow size/offset, scrollbar shown/hidden, controlled/automatic visibility and disabled shadows.

**Required state and transition cases:** No overflow, leading/trailing/both edges, first paint, programmatic scroll, content mutation/resize, viewport resize and visibility callbacks. Check Tabs integration and direction changes with state retained.

**Review and implementation work:** Match fade depth/colors and clipping at every scroll edge, including initial frame. Recompute overflow only when necessary and notify only on real state transitions; compare source-equivalent controlled behavior without requiring browser-specific scroll-timeline internals.

- **Anatomy to account for:** `ScrollShadow`.
- **Current metadata axes to reconcile:** `ScrollShadow.orientation` = 'vertical' | 'horizontal' (default: 'vertical'); `ScrollShadow.variant` = 'fade' (default: 'fade'); `ScrollShadow.size` = number (default: 40).
- **Current named state rows to retain/revalidate:** Leading edge, Trailing edge, Both edges.
- **Existing gallery sections to map into specimens:** Usage; Orientation; Shadow Size; With Card; Hide Scroll Bar; Visibility Change; Vertical; Horizontal; Shadows disabled.

## Cross-component integration scenarios

Run these after the relevant families, because isolated component passes do not exercise their composition boundaries.

| Scenario | Required proof |
|---|---|
| Form inside Modal/Drawer | Text/choice/date/color controls validate, first invalid focus stays inside the topmost dialog, helper/error changes fit, submit/reset works and close restores the trigger. |
| Dropdown item opens AlertDialog/Modal | Menu enters exit phase; the new dialog is topmost, clickable and focused; the exiting menu cannot intercept input or restore focus prematurely. |
| Nested Drawer with a picker | Inner handle owns drag, parent remains stationary; popup stays in the right layer; Escape dismisses only the owning topmost surface. |
| Multiple Select/Autocomplete/ComboBox with Tag removal | Removal at the expanded edge removes exactly one intended value, preserves controlled ownership and focus, does not select the tag body or toggle the popup unexpectedly, and wraps cleanly. |
| Tabs containing Table and form fields | Label wrapping, panel width, overflow chevrons and scroll shadows remain correct; table/editor keys do not activate tabs; changing panel does not leak focus or handlers. |
| Virtualized list/table inside an overlay | Last row reachable near viewport edges, page navigation uses visible height, focus remains on a live item and resize/scroll anchoring stays correct. |
| Calendar/year picker inside date/range popovers | Year/day state and typed segments agree; range across months stays continuous; nested focus/dismissal, disabled dates and constrained viewport work. |
| ColorPicker with area/sliders/field/swatches | Every control shares one color representation, including alpha and degenerate hue cases; changes propagate once; cancel/commit and controlled rejection stay coherent. |
| Async field action produces Toast | Pending state completes, error/success is reflected, toast update preserves options, background/hover/focus pauses timers, and rapid actions do not duplicate results. |
| Theme and motion change during active interaction | Focused/invalid/open/selected/customized components recolor together; animation reversal and reduced motion settle correctly; no invisible hit targets remain. |
| Repeated native route / web example navigation | Old overlays, tooltip managers, toasts, timers, asynchronous callbacks, scroll handles and state do not affect the next specimen; the web page keeps one live preview. |
| Long content, narrow layout and mixed fonts | Table tracks align; vertical Tabs wrap; field messages, badges, tags and overlays fit; baseline/rasterization differences are separated from geometry defects. |

Finish by generating the remaining-case report from the refreshed inventory. Do not replace source-based exclusions, unobserved native paths or unresolved platform limits with a blanket “all components complete” statement.
