# HeroUI v3 parity and audits

Read this guide when changing a public builder, documented behavior, styling,
tokens, motion, component anatomy, demos, reference metadata, or an audit.

## Contract and exclusions

This repository ports HeroUI v3.2.4. Use tagged HeroUI source for component
anatomy and styles, and the exact dependency versions HeroUI pins for inherited
behavior: React Aria 3.51.0, React Stately 3.49.0, and React Aria Components
1.20.0. GPUI framework claims must be valid for the Zed revision in `Cargo.lock`.

Do not reintroduce v2 concepts:

- `content1` through `content4`, numbered color scales, or primary/secondary as
  color roles; v3 uses surfaces, semantic roles, and accent.
- The removed `radius` prop or field `size` props.
- V2-only spellings such as `Divider`, `DateInput`, `Progress`,
  `CircularProgress`, `NumberInput`, or `isLoading`.
- Removed props including `isStriped`, `isBordered`, `isPressable`,
  `isHoverable`, `isBlurred`, `isLoaded`, `isExternal`, `underline`,
  `showOutline`, `isInvisible`, `strokeWidth`, and `hideSeparator`.

`extra_audit.py` is the mechanical guard against v2 leftovers and invented API.

## Choose the audit that owns the claim

| Command | What it proves |
|---|---|
| `python .shots/api_audit.py` | Every documented v3 prop/part row maps to the correct Rust owner and builder/constructor |
| `python .shots/extra_audit.py` | Exported builders are documented by v3 or have a narrow allowed repository reason |
| `python .shots/reason_audit.py` | Recorded omissions still match real documented rows and their reasons remain honest |
| `python .shots/write_only.py` | Builder fields are read rather than stored as no-ops; shared field names still need manual review |
| `python .shots/design_audit.py` | Resting metrics match tagged v3 CSS and token scales |
| `python .shots/token_audit.py` | Theme variables, light/dark values, layout tokens, delays, and shadows are exposed accurately |
| `python .shots/state_audit.py` | CSS and prose interactive states map to rendering code |
| `python .shots/anim_audit.py` | Motion symbols exist and per-overlay enter/exit timing, easing, and scale match |
| `python .shots/behaviour_audit.py` | Documented keyboard/pointer behavior claims map to implementation evidence |
| `python .shots/anatomy_audit.py` | Foreign component slots and documented composition parts exist in the right component |
| `python .shots/part_audit.py` | Every component stylesheet part is implemented, translated, or narrowly excluded |
| `python .shots/example_audit.py` | Gallery section names cover v3's documented examples |
| `python .shots/demo_audit.py` | Gallery code exercises the props used by v3's examples |
| `python .shots/inert_audit.py` | Gallery controlled examples are driven and keyed state is instance-scoped |
| `python .shots/reference_audit.py` | Checked-in reference metadata resolves to real routes, owners, methods, and pinned source links |
| `python .shots/package_audit.py` | Crate packaging and gallery CLI metadata are coherent |

No individual audit proves full parity. In particular, prop coverage does not
prove behavior, design metrics do not prove anatomy, screenshots do not prove
interaction, and static evidence mappings do not replace focused behavior tests.
Use `python .shots/design_audit.py --coverage` when changing the design reader;
an all-green mapped subset is not proof that every upstream metric is covered.

## Running the audit set

Every input is checked in, so the set needs no network and measures the same
v3.2.4 contract on every machine. `.shots/heroui-bundle.txt.gz` is the docs
bundle the prop and prose audits read; `.shots/heroui-css-v3.2.4.tar.gz` is the
component stylesheets the design, motion and anatomy audits read. Both unpack
themselves on first use.

```powershell
Get-ChildItem .shots/*audit.py | ForEach-Object {
    python $_.FullName
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
python .shots/write_only.py
```

CI runs exactly this set, so a local pass is the same evidence CI produces.

Refreshing either pin is deliberate, never a side effect of a run. The bundle
audits refuse any copy whose latest release is not `PINNED_RELEASE` in
`.shots/bundle.py`; point `HEROUI_BUNDLE` at another file and set
`HEROUI_BUNDLE_UNPINNED=1` to read a different release on purpose. To move the
pin, refresh the archive and `PINNED_RELEASE` together and re-run the set:

```powershell
curl -sL https://heroui.com/react/llms-full.txt | gzip -9 > .shots/heroui-bundle.txt.gz
python .shots/design_audit.py --fetch   # then re-pack heroui-css-v3.2.4.tar.gz
```

Run focused scripts while iterating. Run the set when a broad parity claim,
audit parser, shared metadata table, or release surface changes.

## API ownership and omission rules

- Read the entire `## API Reference` section, not a fixed character window or
  only a heading named after the page. V3 uses root tables, part tables,
  composition tables, hooks, functions, and layout tables.
- Attribute builders per `impl <Struct>`, never per file. Several modules define
  multiple components.
- Preserve the owner on folded part rows. A parent builder cannot satisfy a
  child part's same-named prop, and a child cannot satisfy the parent.
- Every name in companion/part/fold tables must resolve to a real `impl`.
- A table is a prop table only when its headers say so. Value/key tables are not
  API rows. Handle tables where a `Component` column precedes `Prop`.
- A builder-name match is insufficient when the documented type is wider. Map
  scalar/array unions and callback shapes to the builder that proves the full
  signature.
- An alias is only a spelling difference. Never map `defaultValue` to `value` or
  otherwise use an adjacent prop to hide a missing state mode.
- Constructor arguments are implemented props when their semantics match.
- Render-prop arguments are normally portable by inversion: accept a closure
  and pass it the state the component already computes.
- Shared interactive render-prop state uses `util::InteractiveState` plus
  `util::interaction`/`util::track_interaction`. Attach the tracking handlers
  only when a render closure is configured because hover/press state costs a
  frame and another event binding.
- A recorded omission must name the real limitation: no accessibility tree, no
  CLDR locale data, browser-only hints, HTTP form transport, a single-valued
  enum, or a genuinely missing mode. "GPUI cannot" requires checking the pinned GPUI
  source first.
- Remove no-op builders. Do not keep a public promise just to make an audit row
  appear implemented.

Use `python .shots/gap_context.py "Table=allowsSorting,isRowHeader"` to print
source descriptions for suspicious gaps, and `reason_audit.py` to re-evaluate
existing exclusions.

Keep audit configuration narrow and named: `WONT_PORT` records real omissions,
`ALIAS` spelling differences, `EXTRA_OK` allowed repository-only builders,
`COMPANIONS`/`PART_STRUCTS`/`FOLD_STRUCTS` ownership, `ABSENT_IS_ZERO` legitimate
zero metrics, and `COVERED_ELSEWHERE` metrics proved by another row. Demo/state
exceptions belong in their specific tables (`WONT_DEMO`, `NEEDS_FEATURE`,
`NOT_STATE`), not in a broad prop omission.

## Audit-reader integrity

An audit must fail loudly when it cannot locate an expected page, section,
table, source block, owner, or symbol. Empty input is not a zero-gap result.

When editing an audit:

- Anchor parsers on structural boundaries such as headings, `impl` blocks, or
  declarations, not arbitrary character windows.
- Scope CSS reads to the owning selector and apply later breakpoint/utility
  overrides in cascade order.
- Resolve variables and arbitrary values before comparing widths or colors.
- Distinguish "absent means zero" from "pattern unreadable" explicitly.
- Report coverage in addition to pass counts. An unmapped source claim is an
  error or a reasoned exclusion, never a silent skip.
- Test a known-negative case so the changed reader demonstrates that it can
  fail. A green result alone may mean it read nothing.
- Avoid putting escape-heavy regexes through a shell heredoc; hidden control
  characters can make a readable-looking pattern match nothing.

## Design, state, and anatomy

- Use the largest applicable responsive breakpoint for this desktop port.
- Respect CSS cascade conflicts such as `size-*` after `h-*`/`w-*` and custom
  property overrides of generic border utilities.
- Focus allowances such as matching positive padding and negative margin are
  not content padding.
- V3 has component-specific radii and motion. Do not collapse them into one
  generic control radius or one generic overlay animation.
- V3 floating panels use shadow separation; dark-mode hairlines are not proof
  that light mode should have a border.
- State evidence must check meaningful arguments. A function call with a
  literal `false` focus flag does not prove focus styling is wired.
- Anatomy comes from containment and composition, not visual plausibility.
  Select, Autocomplete, and ComboBox have different trigger/query structures.
- A composition part with only `children`/`className` still matters even if it
  contributes no prop row.
- Render order is part of anatomy. Labels, descriptions, options, and field
  errors must appear in the order upstream composes them.

## Demos are executable evidence

Section-name coverage and code coverage are separate. A correctly named demo
can still ignore the feature it claims to show.

- Use uncontrolled `default_*` props for an interactive specimen unless the
  demo stores and feeds back the controlled value.
- Wire every controlled gallery prop to its matching callback.
- Give every repeated instance a unique id.
- Exercise a new builder/render closure in the gallery when the public API is
  meant to be discoverable.
- Compare the page's code, not just its headings, and drive behavior that cannot
  be inferred from a screenshot.

When parity work changes interaction, follow the component test guide and the
gallery guide as well as the static audits.
