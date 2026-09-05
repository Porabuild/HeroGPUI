# Gallery visual-parity sweep ledger

Read this before claiming a component matches HeroUI v3.2.4 visually, before
refreshing a pinned golden, or before recording a verdict about an overlay.

This is the committed result of the two-image comparison pass over every
component route in the gallery. It records what was compared, what was judged,
what was fixed, and what could not be observed — so a later sweep can resume
instead of repeating it.

## Method

Each comparison is a pinned native gallery golden, `.shots/<slug>-v3.png`,
against a fresh capture of the same route from `.shots/drive.ps1`.

- Goldens are full-page captures at 1200x1392. Captures are section-filtered
  (`-Section Usage` unless noted) at 1200x1460, which is the window rect
  including its frame. **Page-level Y positions are therefore not comparable.**
  Only element-level geometry inside the demo is judged: size, gaps, radii,
  internal alignment, and colour.
- Judged defects: misposition, wrong gap, non-round geometry, content escaping
  its bounds, and obvious colour or size drift.
- Measure first with pixel scans and only then view the image pair. Every
  capture below is transient scratch output, reproducible from its drive row;
  the ledger keeps the route, section, and flags rather than the file.
- A verdict never rests on a frame that does not contain the thing being
  judged. Where an overlay would not open, the row says NOT OBSERVABLE instead
  of guessing.

Reproduce a row with, for example:

```powershell
.shots/drive.ps1 -Page Table -Section Usage -Out E:\tmp\check.png
.shots/drive.ps1 -Page Select -Section Usage -Overlays -Out E:\tmp\check-open.png
.shots/drive.ps1 -Page Calendar -Section Usage -Dark -Out E:\tmp\check-dark.png
```

## Systematic gallery-chrome deltas

These repeat across the ledger and are **not** component defects.

1. **Demo centring.** `example_frame_with_code` (`gallery/src/pages/mod.rs`)
   centres its demo child with `.items_center().justify_center()`, and the
   `col()` demo helper hugs content (`.items_start()`, no `w_full`). Narrow
   col-based demos render centred where older goldens show them left-aligned or
   full width; `row()` keeps `w_full`, so row-based demos still span the frame.
   Component-internal measurements match the goldens exactly.
   The one case where centring *is* a defect: a `w-full` component inside
   `col()` keeps its intrinsic width, and the Table body then staggered against
   its header. Fixed by `stretch_col()` — see below.
2. **Row pitch from spec changes.** Where a label line-height changed, the demo
   stack pitch changed with it. Switch labels now use 24px lines, so its demo
   pitch is 36 instead of the golden's 32; the track, thumb, and label
   cap-height are identical.
3. **Overlay reachability.** `-Overlays` opens only the popups the gallery maps
   to it (`gallery/src/app.rs`): Select, Dropdown, and DateRangePicker open;
   Autocomplete, ComboBox, and DatePicker Usage do not. Posted clicks never fire
   a Toast push. Open-state rows below say which applied.

## Defects found and fixed

| Commit | Defect |
|---|---|
| `293c60e7` | ColorPicker popup escaped short windows |
| `4d0a2658` | Select popup options unreachable near window edges |
| `70382447` | ComboBox popup anchoring; wrapper line heights |
| `00e97a8f` | Autocomplete search and options unreachable near window edges |
| `a876aab8` | Dropdown menus and submenus escaped the viewport |
| `cfae6f48` | Virtual lists paged by the full content height, not the viewport |
| `a3317f05` | Virtual Table could not cross focus between header and body |
| `acff6364` | Table examples staggered the body against the header |
| `22fdae20` | Driver hardcoded the wheel point at client (600,400), silently no-oping in short windows; settle delay was fixed |
| `3166ad38` | Docs overclaimed a 12px viewport inset on the overlay main axis |

**Table body stagger (`acff6364`).** The Table page wrapped its `w-full` Table
in the hug-content `col()`, so inside the centred demo frame the table kept its
intrinsic width (282px against the golden's 797px). Each row then resolved its
own column tracks and the body drifted off the header. Switching the 13
wrappers to `stretch_col()` gives the table the frame width; measured header
text starts at x326/591/855 and body rows at 325/591/860, 325/590/861,
325/591/860 — aligned within 1px. The regression test
`table_examples_give_the_w_full_table_the_frame_width` fails on the old
wrappers.

**Residual limitation, recorded in `reference_metadata.rs` under
`.table__content`.** The port is a flex column, not upstream's
`border-separate` table, so column tracks resolve per row from
`flex-basis:0; flex-grow:1` with `min-width:auto`. Alignment therefore holds
only while the table is at least as wide as the sum of its columns' widest
cells; a narrower table still staggers. Closing that needs shared tracks, which
is a component change rather than a gallery one.

## Stale and orphaned goldens

| Golden | State |
|---|---|
| `table-v3.png`, `table-dark-v3.png` | Refreshed in `acff6364`; the committed pair predated the header-case fix and the demo width fix |
| `colorslider-v3.png`, `disclosure-v3.png`, `surface-v3.png`, `taggroup-v3.png`, `fieldset-v3.png`, `colorfield-v3.png` | Stale against intentional v3 realignments (Surface and Card lost their default rounding, Disclosure's trigger is a Tertiary Button, Switch and Tag Group heights follow current specs, Color Field and Fieldset examples changed). The current render is the correct one; refresh each golden when its page next changes |
| `fieldslots-v3.png` | Deleted. It captured the Introduction page, not `Page::FieldSlots`, whose title is "Label & Messages"; `label&messages-v3.png` is the real golden. Nothing referenced it |

## Ledger

Wave numbers are the batch in which the comparison ran. "Closed" and "open"
describe the overlay state actually present in the frame.

| Component | Golden | Section / flags | Wave | Verdict |
|---|---|---|---|---|
| Accordion | `accordion-v3.png` | Usage | 3 | NO DEFECT (centring delta) |
| Alert | `alert-v3.png` | Usage | 3 | NO DEFECT (centring delta) |
| Alert Dialog | `alertdialog-v3.png` | Sizes, `-Overlays` | 1 | NO DEFECT — centered, round, contained |
| Autocomplete | `autocomplete-v3.png` | Usage; Usage `-Overlays` | 8 | NO DEFECT closed; open NOT OBSERVABLE, then closed by follow-up |
| Avatar | `avatar-v3.png` | Usage | 3 | NO DEFECT |
| Badge | `badge-v3.png` | Usage | 3 | NO DEFECT |
| Breadcrumbs | `breadcrumbs-v3.png` | Usage | 3 | NO DEFECT (centring delta) |
| Button | `button-v3.png`, `button-dark-v3.png` | Usage; Usage `-Dark` | 3 | NO DEFECT both themes |
| Button Group | `buttongroup-v3.png` | Usage | 3 | NO DEFECT |
| Calendar | `calendar-dark-v3.png`, `calendar-v3.png` | Usage `-Dark`; Usage | 8, fu | NO DEFECT both themes |
| Card | `card-v3.png`, `card-dark-v3.png` | Usage; Usage `-Dark` | 3 | NO DEFECT both themes |
| Checkbox | `checkbox-v3.png` | Usage | 3 | NO DEFECT (centring delta) |
| Checkbox Group | `checkboxgroup-v3.png` | Usage | 3 | NO DEFECT (centring delta) |
| Chip | `chip-v3.png` | Usage | 4 | NO DEFECT |
| Close Button | `closebutton-v3.png` | Usage | 4 | NO DEFECT |
| Color Area | `colorarea-v3.png` | Usage | 4 | NO DEFECT |
| Color Field | `colorfield-v3.png` | Usage | 4 | NO DEFECT (example width changed intentionally) |
| Color Picker | `colorpicker-v3.png` | With Sliders; Usage `-Overlays` | 4, fu | NO DEFECT closed and open |
| Color Slider | `colorslider-v3.png` | Usage | 4 | NO DEFECT (golden stale) |
| Color Swatch | `colorswatch-v3.png` | Usage | 4 | NO DEFECT |
| Color Swatch Picker | `colorswatchpicker-v3.png` | Usage | 4 | NO DEFECT |
| Combo Box | `combobox-v3.png` | Usage; Usage `-Overlays` | 8 | NO DEFECT closed; open NOT OBSERVABLE, then closed by follow-up |
| Date Field | `datefield-v3.png` | Usage | 2 | NO DEFECT |
| Date Picker | `datepicker-v3.png` | Usage; Usage `-Overlays` | 2, fu | NO DEFECT closed and open |
| Date Range Picker | `daterangepicker-v3.png` | Usage `-Overlays` | 2 | NO DEFECT — open calendar aligned, range band unbroken |
| Disclosure | `disclosure-v3.png` | Usage | 4 | NO DEFECT (golden stale) |
| Drawer | `drawer-v3.png` | Placement, `-Overlays` | 1 | NO DEFECT |
| Dropdown | `dropdown-v3.png` | Usage; Usage `-Overlays` | 8 | NO DEFECT closed and open |
| Fieldset | `fieldset-v3.png` | Usage | 4 | NO DEFECT (example changed intentionally) |
| Form | `form-v3.png` | Usage | 5 | NO DEFECT |
| Input | `input-v3.png`, `input-dark-v3.png` | Usage; Usage `-Dark` | 5 | NO DEFECT both themes |
| Input Group | `inputgroup-v3.png` | Usage | 5 | NO DEFECT |
| Input OTP | `inputotp-v3.png` | Usage | 5 | NO DEFECT |
| Kbd | `kbd-v3.png` | Usage (also matched Inline Usage) | 5 | NO DEFECT |
| Label & Messages | `label&messages-v3.png` | Usage | 5, fu | NOT OBSERVABLE against `fieldslots-v3.png` (wrong page); NO DEFECT against the real golden |
| Link | `link-v3.png` | Usage | 5 | NO DEFECT |
| List Box | `listbox-v3.png` | Usage | 2 | NO DEFECT on the component; demo wrapper width and centring drift recorded at `page_list_box` |
| Meter | `meter-v3.png` | Usage | 5 | NO DEFECT |
| Modal | `modal-v3.png` | Sizes, `-Overlays` | 1 | NO DEFECT |
| Number Field | `numberfield-v3.png` | Usage | 5 | NO DEFECT |
| Pagination | `pagination-v3.png` | Usage | 5 | NO DEFECT |
| Popover | `popover-v3.png` | Usage | 1 | NO DEFECT — anchored below trigger |
| Progress Bar | `progressbar-v3.png` | Usage | 6 | NO DEFECT (centring delta) |
| Progress Circle | `progresscircle-v3.png` | Usage | 6 | NO DEFECT |
| Radio Group | `radiogroup-v3.png` | Usage | 6 | NO DEFECT (centring delta) |
| Range Calendar | `rangecalendar-v3.png` | Usage, `-Width 1600` | 6 | NO DEFECT (golden lacks the Usage section) |
| Scroll Shadow | `scrollshadow-v3.png` | Usage | 6 | NO DEFECT (centring delta) |
| Search Field | `searchfield-v3.png` | Usage | 6 | NO DEFECT (centring delta) |
| Select | `select-v3.png` | Usage; Usage `-Overlays` | 8 | NO DEFECT closed and open |
| Separator | `separator-v3.png` | Usage | 6 | NO DEFECT on the component; page-level layout drift recorded |
| Skeleton | `skeleton-v3.png` | Usage | 6 | NO DEFECT (centring delta) |
| Slider | `slider-v3.png` | Usage | 6 | NO DEFECT — fill end at 30% of the 320px track |
| Spinner | `spinner-v3.png` | Usage | 6 | NO DEFECT |
| Surface | `surface-v3.png` | Usage | 7 | NO DEFECT (radius removed intentionally in `2b805047`) |
| Switch | `switch-v3.png` | Usage | 7 | NO DEFECT (pitch delta is spec) |
| Table | `table-v3.png`, `table-dark-v3.png` | full page; full page `-Dark` | 7 | DEFECT — body staggered against header; fixed in `acff6364`, goldens refreshed |
| Tabs | `tabs-v3.png`, `tabs-dark-v3.png` | Usage; Usage `-Dark` | 7 | NO DEFECT both themes |
| Tag Group | `taggroup-v3.png` | Usage | 7 | NO DEFECT (height delta is an intentional fix) |
| Text Area | `textarea-v3.png` | Usage | 7 | NO DEFECT |
| Text Field | `textfield-v3.png` | Usage | 7 | NO DEFECT |
| Time Field | `timefield-v3.png` | Usage | 2, fu | NO DEFECT |
| Toast | `toast-v3.png` | Usage; Usage `-Overlays`; posted clicks | 7 | Trigger NO DEFECT; pushed-toast corner offset NOT OBSERVABLE |
| Toggle Button | `togglebutton-v3.png` | Usage | 7 | NO DEFECT |
| Toolbar | `toolbar-v3.png` | Usage | 7 | NO DEFECT |
| Tooltip | `tooltip-v3.png` | Usage, Avatar, Custom, Focus | 1, fu | Closed triggers NO DEFECT; open tip closed by follow-up |
| Typography | `typography-v3.png` | Usage | 8 | NO DEFECT |

`fu` marks a follow-up comparison run after the wave, which closed the
open-overlay gaps the waves recorded as NOT OBSERVABLE: DatePicker,
ColorPicker, ComboBox, Autocomplete, and Tooltip open states, Time Field
against its own golden, Label & Messages against `label&messages-v3.png`, and
Calendar light.

Non-component routes (Introduction, Installation, Theming, Dark Mode,
Customization, Styling, Design Principles, All Components, Releases) are outside
this sweep: they are prose and index pages, not component renderings.

## Still not observable

**Pushed Toast viewport corner offset.** Three posted-click drives and an
`-Overlays` drive all returned byte-identical frames, so no toast was ever on
screen to measure. Posted input does not fire the push. Observing it needs a
foreground `.shots/capture2.ps1 -Click` run, which moves the real cursor and
raises the gallery window over the user's desktop — opt-in, and deliberately
not taken.
