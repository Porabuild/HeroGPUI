# AGENTS.md

Guidance for coding agents working in this repository.

## Commands

```bash
cargo check --workspace            # fast typecheck
cargo build --workspace            # full build (library + gallery)
cargo run -p herogpui-gallery      # launch the gallery app
cargo test --workspace             # unit tests (color math, time math)
cargo nextest run --workspace      # same suite, faster and per-test isolated (fallback: cargo test)
bacon                              # background check/clippy/test loop from the repo root
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
  `.shots/smoke.ps1` — it launches each of the 76 pages and reports any that
  exit early, with the panic message. Run it in the current shell, not through
  `powershell -File`. A page is only reported as failed if it dies **twice**:
  launching 76 gpui windows back to back intermittently kills one during
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

  **PowerShell variable names are case-insensitive**, so a local `$vk` holding a
  looked-up key code *is* the `$VK` table it was looked up in. `batch.ps1` and
  `drive.ps1` both did that: the first keystroke of a step worked and the second
  threw `unknown key 'tab'`, which reads like a missing entry rather than a
  clobbered hashtable.

  **Posted keys carry no modifiers**, so `key:shift+tab` arrives as a plain Tab
  and moves focus *forward*. To prove a focus-only behaviour, click the control
  (which focuses it but leaves `:focus-visible` off, because a pointer is not a
  keyboard) and then press any key: the root's `on_key_down` sets focus-visible
  as the event bubbles, and the ring -- and anything keyed to it, such as a
  `trigger="focus"` tooltip -- appears.

  **A single posted click sometimes does not register.** The first
  `WM_LBUTTONDOWN`/`UP` pair after the pointer moves onto a control can be
  swallowed -- the element takes the hover and nothing else -- so a check that
  reads "the handler never ran" may only mean the press did not. Send the click
  twice (`do='click:X,Y click:X,Y'`) and assert on the *effect*: the button
  labelled "Pressed 0 times" said 1 on the second attempt with no code change.

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
    console window and takes focus — 76 times in a smoke run. Launch it through
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
- **One process, many checks.** `HEROGPUI_CONTROL=<file>` lets the running
  gallery be told which page, section, theme and overlay state to show, so a
  batch of checks costs one launch instead of one launch each (`control.rs`; the
  file is polled, and the app writes the sequence number back to `<file>.ack`
  once it has *drawn* the change, which is what the driver waits for -- a fixed
  sleep either wastes time or photographs the previous page):

  ```powershell
  .shots/batch.ps1 -Steps @(
      @{ page='Table'; section='Sorting'; do='click:353,387 key:enter' }
      @{ page='Switch'; section='Usage'; out='...\~sw.png' }
  )
  .shots/refresh.ps1              # all 76 reference shots, one process
  .shots/smoke.ps1                # all 76 routes, one process
  ```

  This is not a small saving. Startup is about four seconds and a render is about
  a third of one, so the sweeps went from **five minutes to 24 seconds** (smoke)
  and from eight minutes to **23 seconds** (a full screenshot refresh). A gate
  that takes half a minute gets run after every change; one that takes ten
  minutes gets run "later".

  A panic still takes the process with it, so `smoke.ps1` retries the page that
  died *alone* and reports it only if it dies again -- the shared process makes
  the "died during startup" flake more likely, not less, so that retry matters
  more than before. `-PerProcess` keeps the old behaviour.

  `Process::Start(psi)` returns **null** in pwsh for a console-subsystem binary
  launched with `CreateNoWindow`; construct the `Process` and call `Start()` on
  it, or the script quietly polls a null handle for a few minutes.
- **`.shots/drive.ps1` drives one page without taking the focus**, and it is
  the single-shot version of the same thing:

  ```powershell
  python .shots/sections.py Table                      # what sections exist
  .shots/drive.ps1 -Page Table -Section Sorting -Do "click:353,387 key:enter"
  ```

  `capture2.ps1` injects *real* input, which Windows only delivers to the
  foreground window, so every interactive capture raises the gallery and
  interrupts whatever the user is doing — 76 times in a smoke run. `drive.ps1`
  posts the input messages to the window instead (`PostMessage`), so the window
  stays parked off-screen and unfocused throughout, and `PrintWindow` never
  needed it on screen anyway. Steps are `click:X,Y`, `dblclick:`, `drag:X,Y>X,Y`,
  `key:tab`, `key:down*15`, `type:hello_world`, `wheel:N`, `wait:400`;
  coordinates are the ones you read off the PNG, converted to client space from
  the window's own frame offset.

  What a posted message cannot carry is a **modifier**: Windows keeps the
  shift/ctrl state for real input and gpui asks it, so a capital, a shifted
  symbol or a chord (`ctrl+a`, `shift+pageup`) still needs `capture2.ps1` and the
  foreground. A posted `WM_CHAR` alone types nothing either — gpui reads the
  character from the key event it is handling, so the key-down has to arrive
  with it.
- **`HEROGPUI_SECTION` is a deep link into a page**, and it replaces scrolling:
  a page is far longer than any window, and wheeling down N notches to
  photograph a section is both slow and fragile (the count changes whenever a
  section above it does). `HEROGPUI_SECTION="Sorting"` renders only the sections
  whose title contains that text, so the one under test sits at the top of an
  otherwise empty page. `python .shots/sections.py <Page>` prints the names.
- `HEROGPUI_WINDOW_SIZE=1200x2000` asks for a taller window, but **Windows
  clamps it to the monitor** — a 2000px request comes back as ~1460 on a 1440p
  screen, whether it is set at creation or by `SetWindowPos`. The default
  `MaxTrackSize` is the work area and gpui only overrides `MinTrackSize`, so the
  section deep link is the way to fit a subject in one capture, not a bigger
  window.
- `cx.activate(true)` in `gallery/src/main.rs` is gated on `HEROGPUI_UNFOCUSED`:
  it raises *and focuses* the window, which is exactly what that variable exists
  to prevent, so an unfocused launch no longer steals the foreground on the way
  up.
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
- **A builder-name match does not validate its value type.** Calendar's
  `onChange` counted as implemented while its callback accepted only one date,
  even though v3 documents a date array in multiple-selection mode. When a prop
  is a scalar/array union, map the scoped alias to the plural builder that proves
  the full signature (`Calendar.onChange -> on_change_all`), and drive it.
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

**A render-prop argument is not an unportable prop.** v3 hands a component's
children a function and passes the state in: `{isHovered, isPressed, isSelected,
percentage, formattedDate}`. This port computes every one of those in order to
draw the control, so the honest port is the inversion the audit's own notes
describe -- the builder takes a closure and hands the values over:

| v3 render props | builder |
|---|---|
| Button / CloseButton / ToggleButton / Switch children | `content(\|state\|)` |
| ListBox.Item / Dropdown.Item children | `item_content(\|key, state\|)` |
| Tag children | `tag_content(\|tag, state\|)` |
| Radio children | `option_content(\|label, state\|)` |
| Calendar.Cell / RangeCalendar.Cell | `cell(\|state\|)` |
| ProgressBar / Meter / ProgressCircle ValueLabel | `value_content(\|percentage, text\|)` |
| ColorSlider.Output | `output(\|color, text\|)` |

`util::InteractiveState` is the state struct, and two of its fields cost a frame:
gpui reports a hover and a press to a *handler*, so a render can only read what
the last frame recorded. `util::interaction` is that keyed slot and
`util::track_interaction` wires the handlers -- both are only attached when a
closure is set, because they cost a frame of state.

Two audits pushed back on the first attempt, which is what they are for:
`inert_audit.py` reads an instance from `new(` to its first
`.into_any_element()`, so a render closure that ends with one hides the callback
that follows it -- put the callback first. And `demo_audit.py` then wanted the
new closure exercised, which is how the Switch's "Render Props" demo came to
print `selected + hovered + focus-visible` instead of deriving a label from the
value.

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

`--coverage` answers the question the pass rate cannot: how much of what v3
declares any row actually reads. It said 143 of 392, and closing that gap found
more than the checks did -- along with three ways the *reader* was wrong:

- **A border width is not always a utility.** The colour-area thumb is
  `border: 3px solid white` in plain CSS, and every field applies Tailwind's bare
  `border` and then overrides the width with `var(--border-width-field)`, which
  chains through `--field-border-width: 0px` to nothing at all. Reading the
  utility alone claims a 1px border on ten components that draw none -- v3's
  field states are rings for exactly that reason. `resolve_width` follows the
  variable, and an arbitrary utility (`[border-width:var(--x)]`) no longer parses
  as a `state:` variant.
- **`size-*` and `h-*`/`w-*` set the same properties**, so a rule that applies
  both keeps whichever comes last: the autocomplete clear button is `h-6 w-6` and
  then `size-5`, which makes it 20px, not a box that is two sizes at once.
- **`p-[3px]` cancelled by `-m-[3px]`** is a focus-ring allowance, not padding.
  The three dialog bodies have no inset.

Coverage is zero unchecked now, and the metrics no row can name are recorded in
`COVERED_ELSEWHERE` with the reason, which is the same discipline `WONT_PORT`
keeps: `drives-the-height` for a field whose `py-2` this port spells as a 36px
height, `restated-by-dropdown-menu` for the two `.menu` values a dropdown
overrides, `trigger-is-the-field` for a picker that draws one field instead of a
padded box around a nested one, `accordion-body` for what a Disclosure borrows,
and `no-such-part` for four parts this port does not render.

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

`example_audit.py` matches example *names*, and a name is not a demo. The Tabs
"With Separator" section stood a `Separator` next to two plain tabs for months
and matched perfectly, so the code on both sides needs comparing too:

```bash
python .shots/demo_audit.py            # the report
python .shots/demo_audit.py Tabs       # one page
```

It reads every JSX attribute in every ```tsx block under a page's `## Usage` and
`## Examples`, keeps the ones v3 documents as props of that component *and* this
port implements, and asks whether the gallery's page ever calls the builder that
ports it. 308 props, and 51 of them were exercised by v3's docs and by nothing
here -- which is how these were found:

- **Eleven pages had no uncontrolled demo at all.** v3's Usage examples are
  `defaultValue={...}` with a separate "Controlled" example below; ours were
  controlled twice over, so the uncontrolled path -- the one that broke Tabs --
  was neither shown nor exercised.
- **The Toast "Placements" demo ignored the placement it named**: six buttons
  mapped over `ToastPlacement::*` and every one pushed into the same corner
  (`|(label, _placement)|`). The shell's viewport takes the page's choice now.
- **The Switch "Group" demos never used `SwitchGroup`** -- bare switches in a
  `col` and a `row`, so the component and its one prop appeared nowhere.
- Bounds (`minValue`/`maxValue` on the date, time and number fields), the
  invalid state on two fields, `disabledKeys`, `onAction`, `onSubmit`,
  `onFocusChange`, `firstDayOfWeek`, `startName`/`endName` and the v3-named
  aliases (`onChange`, `onSelectionChange`, `onExpandedChange`) were all
  implemented, documented and undemonstrated.

The unit is the page, not the example: v3's snippets set plenty of props to their
default (`selectionMode="single"`, `delay={700}`) and a demo that leaves those
out shows the same thing, which is why comparing example by example reported 656
of them. What matters is whether a prop is exercised *somewhere*, because one
that is exercised nowhere is one nobody has looked at since it was written.

Two of its fixes tripped other audits, which is the system working: a comment
between `(` and a section title hides the title from `example_audit.py`, and a
`value` written on every render freezes the control unless the caller stores what
comes back -- `inert_audit.py` caught both new demos doing it.

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

A demo that cannot change is the same failure one level up, and it is invisible
in a screenshot:

```bash
python .shots/inert_audit.py
```

reports two things. First, every gallery instance that sets a **controlled**
prop with no `on_*` callback. `Tabs::new`'s positional key filled `selectedKey`, so
`util::controlled` handed the value straight back with no state entity and the
component skipped its whole interactive block — every Tabs demo that passed a
literal was inert, and nine green audits said nothing. It also found four
selects that dropped the choice on the floor, six swatch pickers that could not
be pressed, a dead "select all" checkbox and eight `is_selected(true)` demos that
should have said `default_selected(true)` (same look, still toggles).

Which builders count is read two ways, because our own code only names half of
them: a component with a fallback calls `util::controlled`, and the field is in
the call; a fully controlled one (`Select`, `ColorSwatchPicker`) has no such
call, so v3's tables decide — a prop `P` documented next to `defaultP` or
`onPChange` is state by definition. `.value(None)` is the uncontrolled path, not
a frozen demo, and `ALLOW` holds the instances that are frozen on purpose
(a disabled control cannot change whatever it is passed).

Second, every `use_keyed_state`, `controlled`, `overlay_phase`, `focus_once`,
`panel_focus` or `tab_stop_handle` key that is a **bare literal** rather than one
derived from the component's id. `Dropdown` keyed its open flag by the constant
`"dropdown-open"`, so pressing any trigger on a page opened *every* menu on it;
Modal, Drawer and AlertDialog shared one exit phase, one focus handle and one
drag offset, which is exactly what `HEROGPUI_OPEN_OVERLAYS=1` puts on screen. All
four take an `id` now (`EXTRA_OK`'s `gpui-element-id`), and every keyed state
hangs off it.

Two lessons that belong with it. A constructor must never seed the controlled
prop — a positional seed is `defaultX`. And two components on one page sharing an
id share their keyed state silently: two `TagGroup`s both called `tg-remove`
shared one focus cursor.

v3's theming story *is* its variables: override the custom properties and every
component follows. The port's equivalent is `ThemeColors` and `LayoutTokens`, so
a variable v3 declares and this port does not expose is a hole in the theming
surface -- nothing can read it and no caller can set it.

```bash
python .shots/token_audit.py
```

89 variables, and it found four real holes: `--border-secondary`,
`--border-tertiary` and the two surface foregrounds, the last of which is why a
`Surface`'s secondary and tertiary variants were painting a fill and leaving the
text colour alone. Most of v3's variables are `color-mix`es, and this port
*computes* those rather than storing them -- which is what keeps a derived colour
from drifting out of step with what it mixes -- so the audit accepts an accessor
as readily as a field.

A second pass compares the **values**, per appearance, and found four more:

- `--border` is a step darker than `--separator` (90% against 92%); both had
  been transcribed as the separator's value, in light *and* dark.
- dark `--separator` is 25%, not 22%.
- dark `--field-background` is the surface colour; it had `--default`, two steps
  lighter.
- dark `--overlay` **is** `--surface`. This port had lightened it "so floating
  panels read in dark mode" -- exactly the improvement the no-improvements rule
  forbids. A v3 dark popover is the colour of a v3 dark card and the shadow is
  what separates them, which a screenshot confirms it still does.

A third pass covers `layout.rs` -- the lengths, the two tooltip delays and the
three shadows, compared layer by layer. That is how the overlay shadow was
caught having drifted to an older sheet's two layers instead of v3's three, one
of which throws its blur *upward*; and it is what led to the discovery that **v3
gives a floating panel no border at all.** Light mode separates a panel with that
shadow, and dark mode with `0 0 1px rgba(255,255,255,.3) inset` -- a hairline
just inside the edge, which gpui cannot paint as an inset shadow, so
`overlay_hairline` reproduces it as a one-pixel border and light mode has none.
Every panel here had been drawing a `--separator` border instead.

Reading the Rust side needs a small parser rather than a regex, and four things
make that so: `background` is a field of the theme, of every surface *and* of the
field colours, so a flat name map resolves the wrong one; three of the theme's
colours are written with Rust's field-init shorthand (`foreground,`), which has
no `:` to match on; every layout field is documented with the CSS it ports, so a
whole-file search finds the doc comment first; and the struct's own declaration
(`pub spacing: Pixels,`) looks exactly like an initialiser. Search the
constructor bodies, with the comments stripped.

Two sweeps the audits cannot do, worth running after a token or state change:
`HEROGPUI_THEME=dark .shots/smoke.ps1` and
`HEROGPUI_REDUCE_MOTION=1 HEROGPUI_OPEN_OVERLAYS=1 .shots/smoke.ps1`. Both were
clean at 73/73.

The design audit measures the *resting* look. What a control does when it is
hovered, pressed, focused or disabled is a different list, and v3 states it in
the stylesheets: every component sheet reaches for the same handful of
`status-*` utilities.

```bash
python .shots/state_audit.py
```

It reads which ones each sheet applies -- 100 claims across 46 sheets -- and maps
each to the code that draws it. That is how it was discovered that **41 of v3's
sheets ring a focused control and this port rang none of them**: keyboard focus
was invisible everywhere. Four things worth keeping:

- **A `status-*` whose evidence is a call needs its argument checked too.**
  `apply_field_chrome(.., is_invalid, false, cx)` compiles, reads fine and draws
  no ring; eight fields shipped that way. The check demands at least one call in
  the module whose focus argument is not the literal `false`.
- **One module can serve several sheets**, and then a module-wide search proves
  nothing: `color_picker.rs` answers for five. Those name their own handle
  (`ring_if_focused(area,` / `(track,` / `(trigger,`) so each is asked
  separately.
- **A state is often drawn by a different component.** A `TextArea` composes an
  `Input`, a `Disclosure` is a one-item `Accordion`, an overlay's disabled state
  belongs to the close button inside it. `ELSEWHERE` records where to look.
- **Some states have nothing to trigger them.** v3 styles `[aria-disabled]` on a
  progress bar and `[data-pending]` on a close button but documents no prop that
  could set either, so `no-disabled-prop` / `no-pending-prop` are reasons rather
  than gaps -- inventing the builder would fail `extra_audit.py`.

The stylesheets are only half of it. Each page also lists its states in prose
under `### Interactive States`, and that half covers what the utilities do not:
selected, open, dragging, today, outside-month, frontmost. The same script reads
those 208 claims across 46 pages, which is how the presses were found -- v3
scales four different amounts (`0.97` a button, `0.98` a menu row, `0.96` a
pagination arrow, `0.95` a calendar cell and a radio control) and this port had
one. Two of the prose claims contradict the CSS: `slider.css` styles no hover
and `tag.css` no press, and following the sheet rather than the sentence is the
same choice `design_audit.py` makes.

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

### What no audit measures: the anatomy

Fourteen audits at zero still said nothing about whether a component is *built*
the way v3 builds it, and the Autocomplete was not. v3's `autocomplete.css` is a
`.autocomplete__trigger` (a field-shaped box holding `.autocomplete__value` and a
chevron `.autocomplete__indicator`) plus an `.autocomplete__popover` that stacks
`[data-slot="search-field"]` above the list -- a Select whose popover searches.
This port had built it as an `Input` with a suggestion panel, which is the
*ComboBox*'s anatomy (`combo-box.css`: an input group with a chevron trigger at
its end). Every audit passed anyway, and each for its own reason:

- `design_audit.py` compared `.autocomplete__trigger`'s height, radius and
  padding against the `Input` the component composed. A field and a select
  trigger are both `min-h-9 rounded-field px-3`, so the numbers matched while
  naming the wrong element. Two of its rows said `-> Input` in their label, which
  is the audit *recording* the substitution rather than questioning it.
- `part_audit.py` counts a selector as accounted for when the source names it in
  a comment, and the old file cited `.autocomplete__indicator` and
  `.autocomplete__clear-button` from inside the input's `end_content`.
- `api_audit.py` mapped the props onto whatever builder had the right name:
  `inputValue` and `onInputChange` really do belong to `Autocomplete.Filter`, so
  they matched -- while `value` (v3: the *selection*) was implemented as the
  input's text.
- `example_audit.py` matched "Custom Value" by name, and the demo underneath it
  explained free-text entry.

A prop diff cannot find this, and neither can a screenshot of a control that
looks plausible. What finds it is mechanical, because the stylesheets say it:
v3 marks every component's root with `data-slot`, so a `[data-slot="X"]`
selector *nested inside* sheet C is v3 stating that a C contains an X.

```bash
python .shots/anatomy_audit.py          # the report
python .shots/anatomy_audit.py --all    # every claim, met or not
```

49 containment claims across the sheets, and it found two more the same day:
**a `RadioGroup` had no label, description or error message at all** -- v3's every
documented example opens with `<Label>Plan selection</Label>` and its Validation
example closes with a `<FieldError>` -- and the `Input`'s clear affordance drew a
literal `"×"` in a `rounded_full` box where v3 composes a `CloseButton`
(`rounded-xl p-1 text-muted`, a `size-3` glyph at the search field's size). Three
things worth keeping about how it reads:

- **`:not(...)` is the opposite of containment.** `.button` sizes every svg
  `:not([data-slot="link-icon"] svg)`, which says a Button does *not* hold a link
  icon; reading that as a claim asked `button.rs` for something v3 excludes.
- **A slot named after its own sheet is a part, not a component.**
  `autocomplete-clear-button-icon` under `.autocomplete` is `part_audit.py`'s
  business; only a *foreign* slot is an anatomy claim.
- **Composing and drawing both count.** A `ComboBox` hands its label to the
  `Input` it wraps, a `Meter` writes its own label element, and v3 nests
  `[data-slot="label"]` in both. Which of the two it is, `design_audit.py`
  measures; whether it exists at all is this audit's question.

Reading order is part of the anatomy too: v3 puts a group's `<Description>`
*between* the label and the options and its `<FieldError>` after them, which is
where the RadioGroup now draws them.

A second pass in the same script reads the other half of an anatomy: v3's docs
give each component a table per **composition part** -- `### Autocomplete.Value`,
`### Table.ColumnResizer`, `### Toast.ActionButton` -- 151 of them across 41
components. A part whose props are only `className` and `children` contributes
nothing to `api_audit.py`, so a part this port never renders would not appear
there at all. The evidence is the part's own name, because the convention here is
to cite the v3 spelling in a comment where the part is drawn; 26 of them needed a
`PART_EVIDENCE` entry instead, and every one turned out to be a different
*spelling* rather than a missing part (`Dropdown.Section` is `MenuItem::
SectionLabel`, `Table.ColumnResizer` is `allows_resizing`, `Modal.CloseTrigger`
is the built-in `CloseButton`). Three parts are recorded as not rendered: the two
overlay arrows and `Kbd.Abbr`.

**Never put a backslash escape in a Bash heredoc.** A word-boundary escape in a
patch script arrived as a literal backspace character, so the audit's default
pattern became `Toast\.Title|<BS>title<BS>` and reported 73 parts missing that
were all present -- the file *looked* right on screen, because a control
character reads as nothing. `cat -A` is what shows it. Use the Write tool for
anything with escapes, which this file already said for `'static` and now says
for this.

The three pickers are the case to keep straight:

| v3 component | field | where the query is typed | selection shows in |
|---|---|---|---|
| Select | trigger | nowhere (typeahead only) | `.select__value` |
| Autocomplete | trigger | a `SearchField` **inside** the popover | `.autocomplete__value` |
| ComboBox | input | the input itself | the input, and `.combo-box__value` |

Two things the rebuild turned up that a diff would not have:

- **Focusing an element inside a keystroke fires its click listener.** The Enter
  that picks a row closed the popover and then reopened it: the handler moved the
  focus back to the trigger, and gpui activates a focused element on Enter, so
  the trigger's own `on_click` ran in the same event. Escape can refocus safely;
  Enter cannot. A `std::fs::write` probe in both handlers is what showed the
  order (`click was_open=false` *after* `key-close`).
- **An unset controlled prop is not an empty controlled value.** Passing
  `Some(selected_keys)` to `util::controlled` whenever `default_value` was absent
  made every plain `Autocomplete` controlled-by-nobody: clicking a row wrote to a
  set no one owned and the trigger never changed. `Select` had the answer already
  -- an `is_controlled` flag set by the builder itself -- and `inert_audit.py`
  cannot see this one, because the demo passes no controlled prop at all.

`inert_audit.py` did report the new demo, for the wrong reason: v3's render-prop
table documents `defaultChildren` beside `children`, which reads exactly like a
`default*`/controlled pair. It is not one -- `defaultChildren` is *what the slot
would have drawn*, handed in so a caller can return it unchanged -- so the three
components that take a value closure are recorded in `NOT_STATE`.

## Driving the components: the behaviour suite

Sixteen audits at zero say nothing about whether a control *works*. That was not
a hypothesis: the Autocomplete rebuild shipped with an unset `value` read as a
controlled empty selection, so clicking a row wrote to state nobody owned, and
every audit stayed green. `crates/herogpui-components/tests/` is the answer --
gpui has a headless test platform, so a component can be rendered, clicked and
typed at for real.

```bash
cargo test -p herogpui-components                    # all unit + behaviour binaries
cargo test -p herogpui-components --test overlays    # one binary
```

The docs bundle is the authority for HeroUI's prop surface; the **dependency
versions HeroUI pins are the authority for inherited behaviour**. Read that
exact React Aria/React Stately release rather than `main`: v3.2.4 pins
`react-aria` 3.51.0 and `react-stately` 3.49.0. The distinction is observable --
Toolbar's Home/End handling and Calendar's adjacent-day navigation bounds both
differ from plausible behaviour inferred from a newer source or from the prop
table alone.

`tests/harness/mod.rs` opens a window on the test platform with a host view that
rebuilds one `RenderOnce` component per frame -- exactly as a gallery page does,
which is what keeps the component's keyed state alive between two clicks -- and
records callbacks into an `Rc<RefCell<Vec<String>>>`. `dev-dependencies` enables
gpui's `test-support`. Six things the harness had to learn, each of which cost an
hour:

- **`ThemeProvider::init` must run before the window opens**, or the first draw
  panics: every component reads its tokens from that global.
- **`simulate_keystrokes` sends only the key *down*.** gpui activates a focused
  element's click listeners on key **up**, so `harness::press` dispatches an
  explicit `KeyUpEvent` for the last key. Without it a Space on a focused switch
  does nothing at all, and the test passes for the wrong reason.
- **gpui does not move focus on Tab** -- the app root does. The harness
  reproduces the minimum of `util::app_focus_root`.
- **A drag is down, `MouseMoveEvent` with `pressed_button`, up**, with a
  `window.refresh()` between: events hit-test the *last rendered frame*.
- **Reduced motion has to be set before the first frame.** Flipping it later
  changes the animated wrapper mid-click and the press is lost between down and
  up.
- **After `advance_clock` past an exit, force one more update**, or the ghost of
  the exiting panel answers the probe.

Coordinates come from this port's own constants -- `util::FIELD_HEIGHT`, a 16px
checkbox, `CALENDAR_WIDTH`, a panel 6px below its trigger -- and every test
writes the arithmetic in a comment. Two exceptions worth knowing: label widths
are *measured* with `Window::text_system().shape_line` rather than guessed, and a
table with `default_width` columns must **not** be wrapped in a fixed-width div
(sortable headers flex and need one; pinned columns shift if you add one).

**A closed panel is not observable from a callback**, so closure is proved
behaviourally: click where the row was and assert nothing is recorded. That probe
is how `select.rs`'s row handler was found closing the panel without calling
`on_open_change`.

### What driving them found

Five defects in two days, none of which any audit could see -- every one was a
prop that is *read* somewhere, just not where it mattered:

- **A slider could not be dragged.** Its drag flag was a per-render
  `Rc<Cell<bool>>`, and the press itself repaints the window, so the new frame's
  listeners held a fresh `false` and every move was ignored. Confirmed in the
  gallery before fixing: a click on the track jumped the value, a drag did
  nothing. The Table's column resize survives because it keys its drag through
  `use_keyed_state`; that is now what the Slider does. **Cross-event state must
  be keyed state** -- a per-render cell is a frame long, and a press is not.
- **A press inside a dismissible Modal or Drawer dismissed it.** gpui has no
  hitbox occlusion, so a full-window backdrop `on_click` fires for a press that
  landed on the panel painted above it. The dismissal belongs on the panel,
  where `on_mouse_down_out` reads *its own bounds* -- which is what Popover
  always did. The drawer's drag threshold had never once been consulted: the
  backdrop claimed every pull first.
- **A ColorArea answered no key.** `behaviour_audit.py` had no claim for it
  because v3's ColorArea page has no prose to read -- the audit's weak side,
  now covered by a derived claim like NumberField's.
- **Escape could not hide a focus-opened Tooltip.** The dismissal cleared the
  hover flag; a `trigger="focus"` tip is gated on `contains_focused &&
  focus_visible`, a different condition. It has a per-focus-session latch now,
  dropped when the focus leaves, so the tip returns on the next focus.
- **`Radio.isDisabled` was not ported at all**, and `api_audit.py` stayed green
  because its `COMPANIONS` table named a `RadioOption` struct **no source file
  defines**: the per-option prop resolved against the group-wide `is_disabled`
  builder of the same name. A companion that does not exist contributes an empty
  method set, so the row can never fail. Every name in that table must resolve
  to a real struct.

Two of those were only *visible* in the app: the slider drag and the modal
press. Drive the gallery to confirm a fix, with `.shots/batch.ps1 -Steps
@(@{ page=..; do='drag:625,437>820,437' })`, not just the test.

Driving the rest of the library found six more, and the pattern held: every one
was a path the gallery never renders or a state no screenshot can show.

- **`Button::content` panicked on its first frame.** Two helpers bound `on_hover`
  on one element and gpui refuses the second -- but the real fault was older: an
  animation id carries a *generation*, gpui keys element state by the full id
  **path**, and `hover_fade`'s wrapper therefore reset the button's internal
  hover latch on every restart, so hover-out never fired at all. **An animated
  wrapper must not sit above an element that owns listeners or state**; animate a
  child instead, and keep the interactive element's id constant.
- **A disabled Link stayed a tab stop**, which is the rule this file already
  states, unfollowed in one component.
- **A ScrollShadow shaded content that fits**: gpui's wheel listener adds the
  delta to the offset cell *before* prepaint clamps it, and the render read the
  raw value. Anything reading a scroll offset outside prepaint has to clamp it
  itself.
- **Table.LoadMore was replaced with a click because a `RenderOnce` child does
  not own its parent's scroll handle.** It does not need one: a canvas receives
  its laid-out bounds during prepaint, after every overflowing ancestor has
  intersected its viewport into `window.content_mask()`. Comparing those two is
  the intersection sentinel; keep keyed `(was_visible, row_count)` state so
  continuous visibility fires once and appended rows re-arm it.
- **A Select row closed its panel without reporting it**, so a controlled caller
  never learned; **the Toolbar's arrows walked the whole window** rather than
  staying inside it. Both were invisible with one component on a page, which is
  exactly what a gallery page is.

Two v3 facts worth keeping, because both look like port gaps and are not: **Chip
has no close affordance** (a removable chip is a `TagGroup`), and **Fieldset has
no `isDisabled`** -- only `className`, `children` and `nativeProps`, so a
fieldset disabling its children would be an invention.

Three more harness facts: modifier chords *do* carry here (`ctrl-a` selects all)
unlike the screenshot driver's posted keys, but `Keystroke::parse` joins with
`-`, so `"shift+pagedown"` silently parses as the key `"+pagedown"`; a wheel is
`ScrollDelta::Pixels` with a **negative** dy to scroll down, and every mouse
event needs a redraw after it because events hit-test the last rendered frame;
and taffy shrinks flex children that grow mid-test, so a growth test needs
`flex_shrink_0` or it measures a box that still fits.

Three gpui layout and hit-test rules are now defect-backed, not guesses:

- **A percentage `max_h` resolves against the parent's content box, not the
  viewport.** A Modal panel capped at `85%` inside its container therefore
  never became the viewport-height scroller the CSS anatomy describes. Use an
  absolute viewport-derived cap when the contract is about the window, then put
  `overflow_y_scroll` on the body v3 makes scrollable.
- **A `w_full` child gives a scroller no horizontal range.** It commits the only
  child to the viewport width, so `max_offset` is zero and every wheel clamps
  away even when the child's descendants need more room. The Table's content
  column is `min_w_full().flex_shrink_0()`: it fills a roomy viewport but may
  exceed a narrow one, which gives `overflow_x_scroll` something to move.
- **Use `.occlude()` when a floating control covers another interactive
  element.** gpui otherwise sends the press through to the element underneath;
  that made a Tabs chevron scroll and select the covered tab in one click.
  Backdrop dismissal is a different anatomy: put `on_mouse_down_out` on the
  panel so its own bounds decide what is outside.

`[profile.dev]` sets `debug = "line-tables-only"` because ten test binaries with
full debug info filled the disk, and the link failed as `link.exe: exit code
1318` -- which reads as a broken toolchain rather than as "no space left".

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
  - **A focus ring is a shadow, and its blur cannot be zero.** gpui's shadow
    shader integrates a Gaussian over `3 * blur_radius`; at zero the interval is
    empty and the shadow paints fully transparent -- the first version of the
    ring drew nothing at all. One pixel is the smallest blur that draws. Rings
    are shadows rather than borders because a border moves the content inside
    it: a 2px border on a 36px calendar cell shrinks the date as the cursor
    lands on it. `shadow()` *replaces* the list, so a ring on an element that
    already casts one (a field, a checkbox) has to be appended --
    `util::with_focus_ring` takes the base list for that reason.
  - **gpui already activates a focused element on Enter and Space.** When an
    element with click listeners holds the focus, gpui fires them with
    `ClickEvent::Keyboard` -- which is React Aria's press exactly, mouse, touch
    and keyboard in one handler. A helper that bound Enter to the same handler
    fired everything twice (a switch flipped and flipped back). The corollary
    bites the other way too: an element that has *both* a click listener and its
    own Enter handling does the thing twice, which is why the Select trigger
    keeps only the arrows and lets the click own the open and close.
  - **A table rings *inside* itself.** `status-focused` is an outset ring, and a
    table is v3's exception: `.table__cell` and `.table__column` are
    `shadow-[inset_0_0_0_2px_var(--focus)]` with `rounded-lg`, and a focused row
    draws the same ring split across its cells (three-sided on the first and
    last) so it reads as one continuous outline. That is not a style choice — the
    next cell is flush, so a ring drawn outside is either clipped or, on a cell
    with no background of its own, bleeds *through* and fills it: a focused
    sortable header came out solid accent. `util::inset_focus_ring` is an
    absolutely-positioned 2px border to hang inside the element (gpui has no
    inset shadow, and a real border would move the content).
  - **A disabled control must leave the tab order.** `track_focus` is what puts
    it in, so gate it: v3 gives a disabled control `pointer-events-none` and
    nothing to move, and a Tab that lands on a disabled calendar looks like the
    keyboard is broken.
  - **Tab cannot be trapped by a tab group, only by moving and checking.**
    gpui's `tab_group` gives its children their own *ordering* and nothing else,
    and there is no API for "the stops inside this subtree", so a dialog cannot
    ask where Tab would go. `util::trap_tab` steps with `focus_next`, asks
    whether the focus is still inside the dialog's handle, and re-enters from the
    far end when it is not (backwards means walking forward until it leaves and
    stepping back once, bounded so an empty dialog cannot spin). It also has to
    `cx.stop_propagation()`, because `util::app_focus_root` binds Tab to
    `focus_next` higher up and both firing moves twice — and then set
    `focus_visible` itself, since that is what the root's handler would have
    done and a trapped Tab that moves without ringing looks like it did nothing.
  - **A tab stop comes from the handle, not the element.** `.tab_index(0)`
    configures a handle the element creates for itself, which a component that
    reads its own focus state cannot use; `util::tab_stop_handle` marks the
    handle it returns. Tab itself is the app's job: `util::app_focus_root` binds
    it to `window.focus_next()`, holds the focus when nothing else does (with no
    focus there is no key-event chain at all) and records keyboard-versus-pointer
    input, which is what `:focus-visible` means.
  - **Where a dismissal handler goes is forced by how events reach it.** v3's
    floating surfaces all close on Escape and on a press outside, and no prop
    table says so — React Aria's `useOverlay` does it, so the docs only mention
    dismissal where it is configurable (`isDismissable` on a dialog backdrop).
    `util::dismissable` is both halves, and they attach in different places:
    `on_mouse_down_out` reads the element's *own bounds*, so it belongs on the
    panel (the wrapper an absolute panel sits in has none, which would make every
    press inside the panel count as outside), while a key event goes to the
    focused element and bubbles *up*, so a panel that claims the focus silences
    the keyboard of everything inside it. Popover and the dropdown menu hold the
    focus themselves (`util::panel_focus`); the date and colour pickers read
    Escape on their root and leave the arrows to the calendar grid; Select and
    ComboBox already read Escape where they read the arrows, and binding it twice
    closes twice. `util::panel_focus` takes `open` for a reason — claiming the
    focus on a closed frame spends the one-shot, and Escape then does nothing.
  - **A roving tab stop is one handle, claimed by a different element.** A
    radio group, a tab list and a tag group are each *one* tab stop with the
    arrows moving inside, and the obvious port -- a handle per row, and
    `window.focus(next)` on an arrow -- loses the focus outright: only a handle
    an element is currently tracking receives keys. `tab_stop` is fixed where the
    handle is made, so what moves is the `track_focus` call. RadioGroup and Tabs
    let the *selected* row claim the group's handle (arrows select, and the focus
    follows for free); TagGroup keeps its cursor in a keyed `usize`, because
    moving between tags there does not select. Clamp that cursor to the enabled
    tags -- Delete shortens the list, and a stop pointing past the end or at a
    disabled tag takes the group out of the tab order.
  - **A constructor must not seed the *controlled* prop.** `Tabs::new(id, items,
    "photos")` filled `selected_key`, so `util::controlled` handed back the
    caller's value with no state entity, and the whole interactive block --
    clicks and arrows both -- was skipped: every Tabs demo that passed a literal
    was inert while looking perfectly normal. A positional seed is
    `defaultSelectedKey`; `selected_key` is the builder a controlled caller adds.
    The same check applies to any `pub fn new` that assigns `Some(..)` to the
    field `controlled()` reads.
  - Two components on one gallery page sharing an id share their keyed state,
    which is silent: two `TagGroup`s both called `tg-remove` shared one focus
    cursor. The ids are per page, so grep the page function, not the file.
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
