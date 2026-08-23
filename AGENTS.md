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
cargo fmt --all                    # format; --check is the CI-shaped gate
.shots/lint.ps1                    # the real lint gate: see below
.shots/lint.ps1 -Fix               # apply the machine-applicable fixes first
```

## Lint and format

Both are configured, not left to each machine's defaults, so `--check` means
something:

- `rustfmt.toml` sets edition, `max_width = 100`, LF newlines and the two
  non-default shorthands. Only **stable** options are listed — rustfmt silently
  ignores nightly-only keys on a stable toolchain, so an unstable option there
  would read as enforced while doing nothing. `.gitattributes` pins the same
  line endings so a reformat is never a whole-file diff.
- `[workspace.lints]` in the root `Cargo.toml` holds the whole policy, and every
  crate inherits it with `lints.workspace = true`. A crate that forgets that
  line silently opts out of everything, which is why `.shots/lint.ps1` checks
  for it *before* running clippy.
- `clippy.toml` carries `msrv`, which mirrors `rust-version = "1.87"` in the
  manifest. That number is read off the code, not chosen: `u64::is_multiple_of`
  (`format.rs`, `range_calendar.rs`) is stable since 1.87.

Run the gate with `.shots/lint.ps1`; it is `cargo clippy --workspace
--all-targets -- -D warnings` plus the inheritance check. `--all-targets`
matters — float comparisons and unused imports hide in `#[cfg(test)]`.

The policy is `clippy::all` plus named pedantic/nursery members rather than the
`pedantic` group, which is unusable here: every `pub fn size(mut self, ..)`
trips `must_use_candidate`. Three deliberate exceptions, each with its reason in
the manifest:

- **`wrong_self_convention` is allowed.** Prop builders are named after their v3
  props, so `isDisabled` ports to `fn is_disabled(mut self, ..) -> Self`.
  `extra_audit.py` checks those names against v3's tables; renaming them to
  satisfy clippy would fail the parity audit instead.
- **`redundant_closure_for_method_calls` is allowed.** It rewrites
  `.when_some(top, |b, t| b.top(t))` into `.when_some(top, gpui::Styled::top)`,
  naming a trait path that appears nowhere else in the codebase.
- **`float_cmp` is on**, and the handful of exact comparisons that remain are
  each `#[allow]`ed at the site with a comment saying why the comparison is
  exact on purpose — `max == r` in `from_rgb` asks which channel won a
  `r.max(g).max(b)`, and an epsilon there would let two equal channels both
  match.

`clippy --fix` is worth running but not worth trusting blind: it turned 25
builder closures into `gpui::Styled::*` paths before that lint was switched off.
Read the diff.

- First build of `gpui` takes several minutes; incremental after that.
- Incremental compilation is disabled for the dev profile (see `Cargo.toml`).
  Windows antivirus intermittently locks the incremental session directory,
  which used to leave partial artifacts and fail the next link with
  `unresolved external symbol anon.*`. If you re-enable it and hit that, run
  `cargo clean -p herogpui-components -p herogpui-gallery`.
- The gallery is a GUI app and renders lazily, so a page can compile and still
  panic at runtime (gpui asserts on e.g. a second `.hover()` call on one
  element). After touching components, walk every route:
  `.shots/smoke.ps1` — it launches each of the 73 pages and reports any that
  exit early, with the panic message. Run it in the current shell, not through
  `powershell -File`. A page is only reported as failed if it dies **twice**:
  launching 73 gpui windows back to back intermittently kills one during
  startup — exit -1, empty stderr, a different page each run — and reporting
  those made the gate unheedable. A real panic reproduces on the retry and
  prints; a line reading `retry rendered` is the transient kind.
- `.shots/` holds component screenshots used for visual verification — refresh
  the relevant screenshot when you change a component's appearance:
  `.shots/capture2.ps1 -PageList "Button,Calendar"` (sets `HEROGPUI_PAGE` per
  page). Call it in the current shell; `powershell -File ...` starts a second
  console, which pops a window and takes focus. Extra flags exist because "it
  did not panic" is not the same as "it looks right":
  - `-Fullscreen` sizes the window to the monitor; the default 1200 x
    (screen height) already fits most pages in one shot, since the nav rail plus
    content column come to ~1200px and the pages are long rather than wide.
  - `-Scroll 34` wheels down before capturing, for a page longer than the
    screen. `-HoverX 455 -HoverY 544` parks the cursor on a control, the only
    way to capture a hover-only surface such as a Tooltip. Both drive the real
    cursor, so they need the window on screen and focused — the script refuses
    `-Offscreen` together with them rather than producing a wrong shot.
  - `-Click` presses and releases at the hover point, `-DragX/-DragY` presses
    there and drags to that point in twelve steps (a single jump lands as a
    click, because the app never sees the motion), and `-Keys "12252025"` types
    once something has focus (`-Keys` clicks first).
    This is the only way to check *behaviour*: a screenshot proves a control is
    drawn, not that it answers a key. Injected input reaches the window only
    when it is topmost **and** foreground, which the script arranges with
    `SwitchToThisWindow`; a `SetForegroundWindow` call on top of that is refused
    for a background process and costs the foreground it had just been given, at
    which point every click lands somewhere else and the component looks broken.

  **Capture must never read the screen.** Both scripts had three separate ways
  of interrupting whatever the user was doing, and one of them silently wrote
  the user's own screen into the repo:

  - `Graphics.CopyFromScreen` reads the *monitor* at the window's coordinates.
    When Windows refuses the foreground steal — which it does whenever another
    app is active, and always for a fullscreen game — the PNG holds whatever was
    in front instead. Use `PrintWindow(hwnd, hdc, 2)`, which asks the window to
    render itself; `PW_RENDERFULLCONTENT` (2) is required, because flag 0 comes
    back blank for anything presenting through DirectComposition, as gpui does.
    Check the result is not a uniform frame before saving.
  - `gallery.exe` is a **console-subsystem** binary, so every launch pops a
    console window and takes focus — 73 times in a smoke run. Launch it through
    `ProcessStartInfo` with `CreateNoWindow = $true` (the CREATE_NO_WINDOW
    creation flag), which suppresses that console and nothing else.
    `Start-Process -WindowStyle Hidden` hides the gpui window too, and then
    `MainWindowHandle` stays zero and there is nothing to capture.
  - `HEROGPUI_UNFOCUSED=1` opens the gpui window with `focus: false`, and the
    scripts park it at -32000,-32000 with `SWP_NOACTIVATE`. Off-screen still
    renders; **minimized may not**, which would let the smoke test pass without
    having drawn anything.

  Windows clamps a window to the display, so asking for more height than the
  screen has silently gives less. `capture2.ps1` prints the size it actually
  captured for that reason.
- Build with `.shots/rebuild.ps1`, not plain `cargo build`, after any capture or
  smoke run. Windows keeps `gallery.exe` locked for a while after the process
  exits, so the build fails with `Access is denied. (os error 5)` — and the next
  capture then silently screenshots the *previous* binary, which is worse than a
  failed build.
- Gallery env vars: `HEROGPUI_PAGE` opens a page, `HEROGPUI_THEME=dark` picks the
  appearance, `HEROGPUI_OPEN_OVERLAYS=1` starts every overlay demo open (so
  Modal/Drawer/Select/Dropdown can be screenshotted), `HEROGPUI_UNFOCUSED=1`
  opens the window without taking focus, and `HEROGPUI_REDUCE_MOTION=1` stands in
  for the OS `prefers-reduced-motion` setting that gpui does not surface.

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

**The audit is only as honest as its inputs**, and it has been wrong five
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
- **The window truncated the widest tables.** `props_for` read 4000 characters
  after a `### Component` heading. ComboBox's type column is wide enough that
  the last six rows of its table fell outside it and were never compared:
  `validate`, `validationBehavior`, `name`, `form`, `formValue` and
  `autoComplete`. Both this and `reason_audit.py` now run to the next heading of
  level three or shallower, with no cap -- 592 documented props became 624.
- `ALIAS` can launder a gap. `defaultValue` was mapped to `value`, so every
  missing *uncontrolled* seed counted as an implemented *controlled* prop — 18
  of them. An alias is for a prop we spell differently, never for a different
  prop that happens to be adjacent.
- **Not every table on a page is named after the component.** Matching
  `### <Comp>` and `### <Comp>.<Part>` skipped `### ListLayout` and
  `### TableLayout` (the virtualization props), `### Tag` and
  `### Tag.RemoveButton` on the TagGroup page, `### SwitchGroup`, `### Radio.*`,
  `### Disclosure{Trigger,Content}`, `### Composition Components` on the three
  field pages, `### ToastQueue`, `### toast Function` and `### useFilter Hook`:
  139 documented rows that were never checked against anything. `props_for`
  reads the page's whole `## API Reference` **section** now, which is also the
  boundary that keeps the v2 migration guides out — their `### 2. Prop Changes`
  tables list props v3 *removed*. A heading pattern cannot tell those apart; the
  section can. Every component must resolve to exactly one section, and that is
  asserted rather than assumed.
- **A table of values is not a table of props.** Reading the first column of
  every table turned `### Kbd.Content Type` — the key names `keyValue` accepts,
  under `| Modifier Keys | Special Keys |` — into five missing Kbd props. A
  table is only read when its first header cell says `Prop`, `Name`, `Option`,
  `Function`, `Method` or `Event`.

Because one number cannot say what it covers, the report breaks the omissions
down by reason. 61 of them are `drawn-not-delegated`: values v3 hands *into* a
`render` closure (`isHovered`, `isPressed`, `formattedDate`, `percentage`) which
this port computes and draws itself. Where the part *is* overridable the value is
an alias instead — `ListBox.isSelected` reaches `indicator`,
`Select.selectedItems` reaches `value_content`, `Slider.getThumbValueLabel`
reaches `thumb`. A blanket reason is only honest if the thing it claims is true
of every row it covers, which is what `reason_audit.py` prints them for: it
caught four `isFocusWithin` entries on components that never document it.

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

Neither prop audit says anything about whether a control is the right *size*.
`api_audit.py` was perfectly happy with a button whose corner radius was a third
of v3's, so design has its own audit — and it reads the **React repo**, not the
docs bundle:

```bash
python .shots/design_audit.py --fetch   # caches the v3 stylesheets under $TEMP
python .shots/design_audit.py
```

It pulls `packages/styles/components/*.css` from the `v3` branch, resolves the
Tailwind utilities through v3's own token scales (`--spacing: 0.25rem`,
`--radius: 0.5rem` with `radius-3xl = radius * 3`), and diffs the result against
the Rust that defines each metric. Both sides are mechanical, so neither can go
stale; a check whose pattern stops matching is reported as unreadable, never
skipped. Three things it got wrong first, all worth remembering:

- **Scope each rule.** Reading every `@apply` in a file mixes the base rule with
  the size modifiers, which made a medium button measure 32px because
  `.button--sm` lives in the same file.
- **Prefer the largest breakpoint.** v3's sheet is mobile-first: `.button` is
  `h-10 md:h-9`, so 40px on a phone and **36px** from `md` up. A desktop app is
  past every breakpoint, so the `md` value is the one to match — reading the base
  value made every control a step too tall.
- **Absent is not unreadable.** `.button` declares no `min-w-*` because a v3
  button hugs its label. That is an expectation of zero, recorded in
  `ABSENT_IS_ZERO`, not a broken pattern.

What it found on its first run: **v3 has no single "control" radius.** Each
component names its own step — button and avatar `rounded-3xl` (24px), chip and
menu rows `2xl` (16), close button, tag, link and tooltip `xl` (12), the keyboard
key `lg` (8), separator and skeleton `sm` (4), fields `--field-radius` (12), and
every floating panel `min(32px, --radius-3xl)`. This port had collapsed all of
that into two helpers at 8px and 12px. `util` now has one helper per step, and
the audit checks the mapping rather than trusting it.

Motion needs its own audit too, because a prop diff says nothing about it — a
component can expose every prop and still not move:

```bash
python .shots/anim_audit.py
```

It maps each animation v3's stylesheet defines to the symbol implementing it and
fails when a mapped symbol is missing. That is how `transition-colors` was caught
pointing at a `TRANSITION_MS` constant nothing read; the fix was
`anim::hover_fade`, not a softer claim.

It also checks the **per-overlay motion**, which is where reading the guide
instead of the stylesheets did the most damage. The guide says
`zoom-in-90 fade-in-0 duration-200`, so this port animated every overlay that
way. The component CSS says otherwise, and each surface names its own:

| surface | enter | exit |
|---|---|---|
| Modal / AlertDialog panel | 250ms `ease-out-quad` **zoom-in-105** | 100ms `ease-out-quad` zoom-out-95 |
| their backdrops | 150ms `ease-out`, fade only | 100ms `ease-out` |
| Popover, Dropdown, Tooltip | 150ms `ease-smooth` zoom-in-90 | 100ms `ease-smooth` zoom-out-95 |
| Select, ComboBox, date & colour pickers | 150ms `ease-smooth` zoom-in-95 | same |
| Autocomplete | 250ms `ease-out-fluid` zoom-in-95 | 100ms `ease-out-quad` |
| Drawer | `translate` 250ms `ease-out-fluid` | 200ms, same curve |

A modal panel *shrinks* onto the page from 105%; it does not grow from 90%. Every
exit is 100ms, not 150. `Motion` holds one constant per group and `anim_audit.py`
diffs all twelve against the CSS, so adding an overlay means adding its row
rather than reusing whichever constant is nearest.

The `--ease-*` tokens are real cubic-beziers, and gpui's `with_easing` takes any
function, so use `Curve` rather than reaching for whichever of gpui's two
built-ins looks closest. `ease_out_quint` was standing in for four different
curves.

Before recording an omission, check which kind it is. Four categories that
looked structural were not:

- **A prop the constructor takes positionally** is implemented, not missing.
  `constructor_args` now reads `pub fn new(..)` per struct; fourteen entries had
  been filed under `constructor-arg` for a prop that was already there.
- **A render-prop argument** (`Table.sortDirection`, `Pagination.isActive`,
  `InputOTP.index`, `Dropdown.isSelected`, `Slider.index`) is implementable by
  inverting it: the builder takes a closure and *hands over* the value it
  computes anyway. `ALIAS` then points the prop at that builder.
- **"gpui cannot do X"** deserves a second look, and has been wrong twice. The
  overlay `zoom-in-90` was recorded as impossible because transforms only reach
  `paint_svg`; the press animation had already shown a scale can be reproduced
  geometrically. And "gpui has no multi-line text layout" was simply false —
  `WhiteSpace::Normal` is the *default* and wraps; it was `whitespace_nowrap` on
  the single-line field suppressing it. Check the gpui source before writing the
  reason down.
- **An exit animation** looks impossible because a `RenderOnce` component leaves
  the tree the moment `isOpen` goes false. `util::overlay_phase` keeps it for
  `EXITING_MS` first, which is what gives `[data-exiting]` something to play.

What is genuinely out of reach: ARIA attributes with no accessibility tree,
`locale` without CLDR data, browser image and soft-keyboard hints, the HTTP half
of a `<form>`, and single-valued enums. One more is a missing *mode*, named that
way (`single-wrap-mode`) rather than dressed up as an unportable prop. Say which
is which rather than reporting one number.

None of the prop audits asks whether the gallery ever *shows* a feature. v3's
docs are a list of worked examples per component, so that is its own diff:

```bash
python .shots/example_audit.py     # v3's `### ` examples vs the gallery's sections
python .shots/example_src.py Button "Social Buttons"   # what one contains
```

It reads the `### ` headings under each page's `## Examples` on one side, and the
section titles each `page_*` hands to `doc_page` on the other. Four things it got
wrong first, each of which silently inflated or deflated the number:

- **Match on indentation, not on `("..",`.** The loose form also counted every
  element id and tab key in the page body, which made "104 demonstrated" mean
  nothing.
- **A single-section page writes `vec![(` on one line**, so that tuple's paren
  never starts a line and the section did not count at all.
- **A page whose v3 name is not the snake_case of ours needs a `PAGE_ALIAS`.**
  `ToggleButtonGroup`, `Label`, `Description`, `ErrorMessage` and `FieldError`
  were being skipped entirely -- 26 examples that never appeared in the total.
- **Three categories, not two.** `WONT_DEMO_NAMES` is for an example name
  wherever it appears (v3 documents "Render Function" on 31 pages and it is the
  same DOM-substituting prop every time); `WONT_DEMO` is for one page's example;
  and `NEEDS_FEATURE` is for an example waiting on a component feature this port
  has not built -- counted and named separately rather than excused, so the
  number cannot hide. It is empty now: the last two entries were virtualization
  (`uniform_list`, for the list, the table and the three pickers) and a date
  field spanning a date *and* a time (`granularity`).

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

None of those audits asks whether a control answers a key. v3 says what each
one does under `## Accessibility`, in prose:

```bash
python .shots/behaviour_audit.py
```

`CLAIMS` turns the prose into claim ids (`arrows`, `home-end`, `page-up-down`,
`typeahead`, `escape`, `long-press`, `submenu`, `drag-dismiss`), `EVIDENCE` names
the code that implements one, and a claim with neither evidence nor a recorded
reason is a gap. It found seven on its first run, and they were not small: the
Slider and ColorSlider had **no `on_key_down` at all** — a slider that only
answers the pointer — the Dropdown menu had no arrow keys, typeahead existed
nowhere, and the Drawer could not be dragged shut. `list_nav.rs` now holds the
typeahead beside the arrow-key resolver, so a listbox, a menu and a select
search the same way.

The prose is this audit's weak side: a claim only counts if it is *written*, so a
component with a short Accessibility section is asked less. That is why an
unmapped claim is an error rather than a skip — the mapping table is where the
reading gets pinned down. Two things it excuses with reasons: there is no
document scroll to lock and no page outside the window to trap focus in.

Verify a claim by driving it, not by reading the diff — `capture2.ps1 -Keys`
proved the typeahead moves the ring to "Sent" and `-DragX` proved the drawer
closes. Both took several tries to *aim*: the click lands at the bitmap
coordinate you pass (the client offset cancels out), and a demo that wires no
`on_change` cannot change, so a static specimen looks exactly like a broken
handler. Instrument the handler with a one-line `std::fs::write` before
concluding it never ran.

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
  - No context propagation: a child cannot reach an ancestor component. `Form`
    is therefore *told* its fields (`Form::field`), and a field's `name` rides
    on its state entity so the form can read it back.
  - `absolute` does not lift a panel above later siblings; gpui paints in tree
    order. Floating surfaces must go through `util::floating` (`deferred`) or
    `anchored`, or the page content below will paint over them.
  - `uniform_list(id, count, |range, window, cx| ..)` is the virtual list, and
    it is what `<Virtualizer layout={ListLayout}>` ports to: one fixed row
    height, which is why `row_height` is the builder that turns virtualization
    on. Three things it needs. Its callback is `'static` and runs again on every
    scroll, so it can borrow neither `self` nor `cx.colors()` — copy the tokens
    out and move owned data in. A row is laid out on its own, so it takes the
    width it is *given*: without `w_full` the table's columns bunched at the left
    edge. And it sizes from the style, so the list needs an explicit height.
  - A row builder shared by the plain and the virtual path is what keeps a
    thousand-row list drawing the same row as a three-row one. Extracting one
    from a loop has two catches: `continue` cannot cross a closure boundary (the
    closure returns the row it has finished instead), and a section header stops
    being a sibling — header and row are one element, because a virtual row is
    one slot tall.
  - `AnyElement` is built once and consumed once, so a component whose cells are
    elements cannot be handed them up front: `Table::virtual_rows` takes the row
    *factory* v3 spells as `<Table items={users}>{(user) => …}</Table>`.
- Theme tokens are already `Hsla` — do not wrap them in `gpui::Hsla::from(..)`.
- Keep OKLCH token values identical to upstream HeroUI; do not "improve" them.
- Component ids: give every interactive element a distinct id within its page.
- The Bash tool's shell mis-lexes heredocs containing Rust `'static` lifetimes;
  use the Write tool for those files.
