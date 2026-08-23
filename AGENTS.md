# AGENTS.md

Guidance for coding agents working in this repository.

## Commands

```bash
cargo check --workspace            # fast typecheck
cargo build --workspace            # full build (library + gallery)
cargo run -p herogpui-gallery      # launch the gallery app
cargo test --workspace             # unit tests (color math, time math)
cargo clippy --workspace           # lint (no warnings policy on new code)
cargo fix --allow-dirty --workspace
```

- First build of `gpui` takes several minutes; incremental after that.
- Incremental compilation is disabled for the dev profile (see `Cargo.toml`).
  Windows antivirus intermittently locks the incremental session directory,
  which used to leave partial artifacts and fail the next link with
  `unresolved external symbol anon.*`. If you re-enable it and hit that, run
  `cargo clean -p herogpui-components -p herogpui-gallery`.
- The gallery is a GUI app and renders lazily, so a page can compile and still
  panic at runtime (gpui asserts on e.g. a second `.hover()` call on one
  element). After touching components, walk every route:
  `powershell -File .shots/smoke.ps1` — it launches each of the 71 pages and
  reports any that exit early, with the panic message.
- `.shots/` holds component screenshots used for visual verification — refresh
  the relevant screenshot when you change a component's appearance:
  `powershell -File .shots/capture2.ps1 -PageList "Button,Calendar"` (sets
  `HEROGPUI_PAGE` per page). Extra flags exist because "it did not panic" is
  not the same as "it looks right":
  - `-Height 1400` for a taller window, `-Scroll 34` to wheel down before
    capturing — the only way to see a section below the fold.
  - `-HoverX 455 -HoverY 544` parks the cursor on a control first, which is the
    only way to capture a hover-only surface such as a Tooltip.
- Gallery env vars: `HEROGPUI_PAGE` opens a page, `HEROGPUI_THEME=dark` picks the
  appearance, `HEROGPUI_OPEN_OVERLAYS=1` starts every overlay demo open (so
  Modal/Drawer/Select/Dropdown can be screenshotted), and
  `HEROGPUI_REDUCE_MOTION=1` stands in for the OS `prefers-reduced-motion`
  setting that gpui does not surface.

## Measuring parity

Do not judge API parity by eye — run the audit:

```bash
curl -sL -o "$TEMP/heroui-full.txt" https://heroui.com/react/llms-full.txt
python .shots/api_audit.py
```

It diffs every documented v3 prop table against our `pub fn` builders and
reports three numbers: implemented, deliberately not ported (with a reason in
`WONT_PORT` — no-intl, no-html-forms, state-entity-seeds-it, …), and **real
gaps**.
Add a prop to `WONT_PORT` only with a reason; add to `ALIAS` when we simply
spell a prop differently. Both tables accept a scoped `Component.prop` key,
which is the form to use whenever a bare name would mean different things in
different components.

**The audit is only as honest as its inputs**, and it has been wrong three
times:

- v3 splits a component's API across the root table *and* one table per
  composed part (`### Tooltip.Content`, `### Table.Column`, `### Dropdown.Menu`).
  Reading only the root table hid whole prop tables and understated the surface
  by ~100 props. `props_for` now folds `Component.Part` tables into the parent.
- Builders are attributed per `impl <Struct>` block, not per file. Several
  components share a module, and a file-level set let `ColorPicker`'s props
  count as `ColorField`'s. Where our port genuinely splits one component across
  several structs (`Toast`/`ToastViewport`, `Table`/`TableColumn`), list them in
  `COMPANIONS` — but never list a *different* component that happens to live in
  the same module, or a gap on one hides behind the other.
- `ALIAS` can launder a gap. `defaultValue` was mapped to `value`, so every
  missing *uncontrolled* seed counted as an implemented *controlled* prop — 18
  of them. An alias is for a prop we spell differently, never for a different
  prop that happens to be adjacent.

The diff also only ever ran in one direction. `api_audit.py` asks "is every
documented prop implemented?", which cannot see a prop held over from v2:

```bash
python .shots/extra_audit.py
```

reports every builder we expose that v3 does not document, and it is how
`Card::is_pressable`, `ProgressBar::is_striped`, `RadioGroup::size` and the
`radius` prop were finally found. It splits the report in two, because v3's
per-component tables are demonstrably incomplete (`Input` lists no `isInvalid`
though every sibling field does, and several say only *"Inherits from React
Aria X"*):

- **documented for a sibling** — informational; a shared spelling is consistent.
- **not documented anywhere in v3** — must reach zero. Delete the builder, or
  record it in `EXTRA_OK` with one of the few allowed reasons (`constructor`,
  `composition`, `no-classname`, `gpui-element-id`, `state-entity`, `accessor`).
  `composition` is the common one: v3 composes `<Label>`, `Modal.Close` or
  `ProgressBar.ValueLabel` as child parts, and a monolithic builder takes them
  as a prop or as a flag that renders the built-in part.

A prop that is stored but never read is worse than a missing one: the API
promises behaviour it does not have. After adding fields, run

```bash
python .shots/write_only.py
```

which reports, per component struct, any field whose only mentions are its
declaration, its initialiser and its builder assignment. When a documented prop
turns out to be unimplementable (a browser attribute, an accessibility name, a
single-valued enum), **remove the builder** and record the reason in
`WONT_PORT` rather than leaving a no-op behind.

The detector matches `self.<field>` module-wide, so two structs in one file that
share a field name cover for each other — that hid an unwired
`SearchField::validate` next to `Input::validate`. It now lists shared names at
the end; check those by hand.

To decide whether a reported gap is a real prop or just a render-prop argument
v3 passes into a child, print its source table and description:

```bash
python .shots/gap_context.py "Table=allowsSorting,isRowHeader"
```

Recorded omissions rot. `python .shots/reason_audit.py [reason ...]` prints each
`WONT_PORT` entry next to the v3 row it excuses, which is how these were caught:

- `disabled` / `readOnly` / `required` were filed under `no-html-forms`, but v3
  documents them as *"Disables the input"* — the plain attribute spellings of
  props already implemented. They are aliases, not omissions.
- `validate` was excused as "the caller validates". v3's `validate` is a
  function the **component** runs and whose message it shows; see
  `components/src/validation.rs`.
- `Dropdown.trigger` and `Dropdown.type` sat under `not-a-prop` beside genuine
  table-parsing artefacts.
- five entries no longer matched any documented row at all.

## Scope: HeroUI v3 only

This is a port of **HeroUI v3**, not v2. When in doubt, the authoritative source
is `https://heroui.com/react/llms-full.txt` (5MB; download it and grep rather
than fetching pages, which are JS-rendered and 404 through plain HTTP).

v2 concepts that must **not** come back:

- `content1`–`content4` → `surface`, `surface_secondary`, `surface_tertiary`,
  and `overlay` for floating panels
- numbered `50`–`900` scales → `RoleColor::hover()` / `soft()` / `soft_hover()`
- `primary` / `secondary` as colors → `accent`; `secondary` is a *variant*
- the `radius` prop → theme radius tokens (v3 removed it from every component)
- `divider` → `separator`; `hover_opacity` → a hover *color* mix
- the v2 props v3 deleted, which `extra_audit.py` is what catches: `color` on
  anything but Avatar/Badge/Chip/ColorSwatch/ColorSwatchPicker/Meter/
  ProgressBar/ProgressCircle/Spinner/Typography; `size` on any form field (v3
  gives them one height — `util::FIELD_HEIGHT`/`FIELD_TEXT`/`FIELD_ICON`); and
  `isStriped`, `isBordered`, `isPressable`, `isHoverable`, `isBlurred`,
  `isLoaded`, `isExternal`, `underline`, `showOutline`, `isInvisible`,
  `strokeWidth`, `hideSeparator`
- `isLoading` → `isPending`; `Divider` → `Separator`; `DateInput` → `DateField`;
  `Progress` → `ProgressBar`; `CircularProgress` → `ProgressCircle`;
  `NumberInput` → `NumberField`

## Architecture

- `crates/herogpui-core` — the shared v3 prop vocabularies (`Color`, `Variant`,
  `FieldVariant`, `Prominence`, `Backdrop`, `Size`, `SizeXl`, `Orientation`,
  `SelectionMode`) and color math. `color.rs` implements `oklch()`,
  `mix_oklab()` and `soft_mix()` so theme tokens can be transcribed verbatim
  from upstream CSS. No component code here.
- `crates/herogpui-theme` — v3 design tokens ported from
  `packages/styles/themes/default/variables.css`. `semantic.rs` holds the base
  OKLCH values plus the derived `color-mix` accessors; `layout.rs` holds radius,
  border, shadow and timing tokens. `ThemeProvider` is a GPUI `Global`; read
  tokens anywhere with the `ActiveTheme` trait (`cx.colors()`,
  `cx.role(Color::Accent)`, `cx.layout()`).
- `crates/herogpui-components` — one module per `@heroui/*` package. Components
  are builder structs deriving `IntoElement` + implementing `RenderOnce`
  (`fn render(self, &mut Window, &mut App) -> impl IntoElement`). Callbacks use
  the shape `Fn(&ClickEvent/&str/f32/usize/bool, &mut Window, &mut App)`.
  Callback fields that get cloned into closures must be `Arc<dyn Fn ...>`
  (`Box<dyn Fn>` is not `Clone`).
- `crates/herogpui` — umbrella re-export facade.
- `gallery/` — docs/gallery binary. Root entity `Gallery` holds routing state
  and all interactive demo state; pages live in
  `gallery/src/pages/{docs,components}.rs`; the page registry and the fifteen
  v3 nav categories live in `gallery/src/pages/mod.rs`. Icons are embedded SVGs
  served by `gallery/src/assets.rs` under `herogpui/icons/*`.

## Conventions & gotchas

- gpui version: **0.2.2** from crates.io (API notes):
  - `Pixels` inner field is private; convert with `f32::from(px_val)`.
  - No `select_none`, `grow`, `uppercase`, div-level `rotate`/`scale` in this
    version; rotation only via `svg().with_transformation(...)` inside
    `with_animation`.
  - `on_click` and `.active(..)` require a stateful element (`.id(...)` first);
    `.hover(..)` does not. Helpers that apply both must take `Stateful<Div>`.
  - `overflow_y_scroll()` requires `.id(...)` too.
  - `svg()` does **not** inherit text color — always set `.text_color(..)`
    explicitly or the glyph renders invisible.
  - A new icon needs **two** edits: a `pub const` in `components/src/icons.rs`
    *and* an entry in `gallery/src/assets.rs`. The asset list is explicit, and
    an unregistered path loads as `None`, so the glyph silently renders nothing
    — indistinguishable from a colour bug.
  - `ParentElement` needs an explicit `extend` impl for custom child-holding
    components.
  - Branches returning different element types must be unified via
    `.into_any_element()`.
  - Holding `cx.colors()` across a nested `render(window, cx)` call is a borrow
    error; clone the tokens you need first.
  - Divs are `Display::Block`, so a block-level `.flex()` child fills its
    parent's width — HeroUI controls are `inline-flex`. Put controls in a flex
    parent (`.flex().items_start()`) or they stretch.
  - `absolute` does not lift a panel above later siblings; gpui paints in tree
    order. Floating surfaces must go through `util::floating` (`deferred`) or
    `anchored`, or the page content below will paint over them.
- Theme tokens are already `Hsla` — do not wrap them in `gpui::Hsla::from(..)`.
- Keep OKLCH token values identical to upstream HeroUI; do not "improve" them.
- Component ids: give every interactive element a distinct id within its page.
- The Bash tool's shell mis-lexes heredocs containing Rust `'static` lifetimes;
  use the Write tool for those files.
