"""The inverse of `api_audit.py`: builders we expose that v3 does not document.

`api_audit.py` only ever asked "is every documented prop implemented?". That
question cannot catch the opposite failure -- a builder held over from HeroUI
v2, or invented here -- and v2 leftovers are exactly what this port is supposed
to be free of. `Card::is_pressable`, `ProgressBar::is_striped`,
`RadioGroup::size` and the `radius` prop survived four audits because nothing
looked in this direction.

The report has two halves, because "undocumented" means two different things:

* **documented elsewhere** -- the name appears in some other v3 prop table.
  v3's per-component tables are demonstrably incomplete (`Input` lists no
  `isInvalid` though every sibling field does, and several say only "Inherits
  from React Aria X"), so a spelling shared with a sibling is consistent, not
  invented. Informational only.
* **not documented anywhere** -- the name appears in no v3 table at all. Each
  is a v2 leftover to delete or a deliberate addition recorded in `EXTRA_OK`,
  and anything left over fails the audit.
"""
import glob
import io
import os
import re
import sys
from collections import defaultdict

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import api_audit as A  # noqa: E402  (reuses its parsing, tables and bundle)

# Public names v3 removed. This is a hard ban, not an EXTRA_OK-style excuse:
# a name here is deleted, and nothing scoped can bring it back. The scan is
# declaration-anchored over every crate and the gallery -- including components
# `api_audit.py`'s FILES never names -- so it reads what is *declared*, never
# what is *said*: `strip_rust` blanks comments and literal interiors first and
# only `pub` declarations match, so a migration note mentioning `Divider`, a
# doc-commented v2 example, or a `"pub struct Divider"` test fixture string is
# not a reintroduction, and private identifiers are not public API.
#
# `is_destructive` and `hide_close_button` are banned like every other removed
# v2 spelling: the port spells a danger confirm with v3's `variant="danger"`
# on the composed footer `Button`, and close-trigger visibility by composing
# or omitting the `CloseTrigger` part, so neither name is a live spelling of
# anything v3 documents.
BANNED_V2 = {
    # Components and composition parts v3 dropped; the v3 spelling follows.
    'AvatarGroup': 'v2 component (v3 composes Avatar children)',
    'Divider': 'v2 component (v3: Separator)',
    'DateInput': 'v2 component (v3: DateField)',
    'NumberInput': 'v2 component (v3: NumberField)',
    'CircularProgress': 'v2 component (v3: ProgressCircle)',
    'Navbar': 'v2 component (not in v3)',
    'CardBody': 'v2 part (v3 composes Card children)',
    # The same names as Rust builders, constructors or modules.
    'avatar_group': 'v2 component builder (v3 composes Avatar children)',
    'divider': 'v2 component builder (v3: separator)',
    'date_input': 'v2 component builder (v3: date_field)',
    'number_input': 'v2 component builder (v3: number_field)',
    'circular_progress': 'v2 component builder (v3: progress_circle)',
    'navbar': 'v2 component builder (not in v3)',
    'card_body': 'v2 part builder (v3 composes Card children)',
    # Builders the deleted v2 aliases in `api_audit.py` papered over.
    'is_external': 'removed v2 prop (v3: target/rel on Link)',
    'is_striped': 'removed v2 prop (v3: Tailwind on ProgressBar.Fill)',
    'is_bordered': 'removed v2 prop (v3: Tailwind border/ring)',
    'is_blurred': 'removed v2 prop (v3: Tailwind backdrop-blur)',
    'is_hoverable': 'removed v2 prop (v3: Tailwind hover classes)',
    'is_pressable': 'removed v2 prop (v3: button/link inside Card)',
    'is_destructive': 'removed v2 prop (v3: variant="danger" on the composed footer Button)',
    'hide_close_button': 'removed v2 prop (v3: compose or omit the CloseTrigger part)',
}

# Builders v3 removed from one component, banned only inside that component's
# own module. The `BANNED_V2` names above are v2 leftovers everywhere, but
# `content` and `start_content` are not: render-prop `content` builders are
# live port spellings across the fields and lists, and v3's Input legitimately
# spells `startContent` for its leading slot (see `input.rs`, which only the
# button scope must not confuse with its own removed builder). So the ban is
# scoped to the badge, button and chip modules, where v3 composes the removed
# builders' content as the root's own ordered children (`Badge`'s label text
# through `Badge.Label`, `Button`'s icon/label sequence through
# `ParentElement` order, `Chip`'s icon/dot/label sequence through `Chip.Label`
# and `ParentElement` order) -- a reintroduction would re-fork the anatomy the
# compound parts exist to carry. Note `Button` bans both `start_content` and
# `end_content`: its render-prop `content` builder is v3's children-as-a-function
# and stays, while v3 composes icons and the label as ordered `ParentElement`
# children with no slot builders at all.
SCOPED_BANNED = {
    'crates/herogpui-components/src/badge.rs': (
        ('content',
         'removed Badge builder (v3 composes the badge content as ParentElement children)'),
        ('start_content',
         'removed Badge builder (v3 composes badge content as ParentElement children)'),
    ),
    'crates/herogpui-components/src/button.rs': (
        ('start_content',
         'removed Button builder (v3 composes the icon and label as ordered children)'),
        ('end_content',
         'removed Button builder (v3 composes the trailing icon as an ordered child)'),
    ),
    'crates/herogpui-components/src/chip.rs': (
        ('content',
         'removed Chip builder (v3 composes chip content as ParentElement children)'),
        ('start_content',
         'removed Chip builder (v3 composes the icon and Chip.Label as ordered children)'),
    ),
}

# Declaration shapes, anchored on `pub`. The `fn` pattern reads the Rust
# modifiers (`async`, `const`, `unsafe`, `extern "ABI"`, in any order) and the
# whitespace -- newlines included -- that may sit between `pub` and `fn`, so
# `pub async fn navbar` cannot hide the way a bare `pub fn` cannot. The
# const/static pattern refuses a `fn` that follows, or `pub const fn card_body`
# would be reported as a const named `fn` instead of a banned builder.
# Patterns run over the whole stripped source (whitespace crosses newlines),
# and the reported line is the name's own.
DECL_PATTERNS = tuple(re.compile(p) for p in (
    r'\bpub\s+(?:struct|enum|union|trait|type)\s+([A-Za-z_][A-Za-z0-9_]*)',
    r'\bpub\s+(?:const|static)\s+(?!fn\b)([A-Za-z_][A-Za-z0-9_]*)',
    r'\bpub\s+(?:(?:async|const|unsafe|extern)\s+)*fn\s+'
    r'([A-Za-z_][A-Za-z0-9_]*)',
    r'\bpub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*)',
))
PUB_USE = re.compile(r'\bpub\s+use\b')
IDENTIFIER = re.compile(r'\b[A-Za-z_][A-Za-z0-9_]*\b')

# Everything this repository builds. The components `api_audit.py` audits are
# the FILES table; the ban covers the rest of the crates and the gallery too,
# so a removed component cannot come back in a module no audit names.
SCAN_GLOBS = ('crates/*/src/**/*.rs', 'gallery/**/*.rs')


WORD_CHARS = set(
    'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_')

# A `'` opens a char literal only when one escape or one plain character
# follows before the closing `'`; anything else (`'a`, `'static`) is a
# lifetime and stays code. `\\[\s\S]` is the escaped-character form: exactly
# one character after the backslash (`'\n'`, `'\t'`, `'\\'`, `'\''`), so the
# hex and unicode alternatives ahead of it must claim their longer shapes
# first.
CHAR_LITERAL = re.compile(
    r"'(?:\\x[0-9a-fA-F]{2}|\\u\{[^}]*\}|\\[\s\S]|[^\\'])'")


def _blank(chunk):
    """`chunk` with every character but the newlines turned into a space."""
    return ''.join('\n' if ch == '\n' else ' ' for ch in chunk)


def _raw_hashes(text, i):
    """The `#` count of a raw-string prefix at `i` (`r#"` -> 1), else None.

    `r` starts a raw string only as its own token: not inside `for` or a
    longer identifier, and not a raw identifier like `r#type` (which this
    rejects because no `"` follows the hashes). `br#".."#` counts because
    the character before that `r` is a `b` that is itself not part of an
    identifier.
    """
    if i and text[i - 1] in WORD_CHARS:
        if text[i - 1] != 'b' or (i >= 2 and text[i - 2] in WORD_CHARS):
            return None
    j = i + 1
    while j < len(text) and text[j] == '#':
        j += 1
    if j < len(text) and text[j] == '"':
        return j - i - 1
    return None


def strip_rust(text):
    """Rust source of the same length with comments and literals blanked.

    One pass. Line comments, (nested) block comments, and the interiors of
    `".."`, `r#".."#`, `b".."` and `'a'` literals become spaces, so a
    declaration-shaped phrase inside any of them can no longer match, and a
    `/*` inside a literal cannot switch the scanner into comment mode and
    suppress the real declarations that follow it. Newlines survive so line
    numbers stay honest; escapes (including a `'` inside a char literal and
    an escaped quote inside a string) are what keep the scanner aligned with
    rustc's own lexing.

    A literal or comment that opens and never closes raises RuntimeError:
    everything after it is indistinguishable from its interior, and blanking
    to the end of the input would silently hide every declaration that
    follows. Lifetimes (`'a`, `'static`) are not char literals and stay code;
    a truncated one -- an identifier run or escape after `'` that reaches the
    end of the input with no closing quote -- is unreadable and fails loudly
    too.
    """
    out = []
    i = 0
    n = len(text)
    while i < n:
        ch = text[i]
        if text.startswith('//', i):
            end = text.find('\n', i)
            end = n if end < 0 else end
            out.append(_blank(text[i:end]))
            i = end
            continue
        if text.startswith('/*', i):
            depth = 1
            j = i + 2
            while j < n and depth:
                if text.startswith('/*', j):
                    depth += 1
                    j += 2
                elif text.startswith('*/', j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            if depth:
                raise RuntimeError(
                    'unterminated block comment at line %d'
                    % (text.count('\n', 0, i) + 1))
            out.append(_blank(text[i:j]))
            i = j
            continue
        if ch == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == '\\' else 1
            if j >= n:
                raise RuntimeError(
                    'unterminated string literal at line %d'
                    % (text.count('\n', 0, i) + 1))
            out.append(_blank(text[i:j + 1]))
            i = j + 1
            continue
        if ch == 'r':
            hashes = _raw_hashes(text, i)
            if hashes is not None:
                close = '"' + '#' * hashes
                end = text.find(close, i + 1 + hashes + 1)
                if end < 0:
                    raise RuntimeError(
                        'unterminated raw string at line %d'
                        % (text.count('\n', 0, i) + 1))
                end += len(close)
                out.append(_blank(text[i:end]))
                i = end
                continue
        if ch == "'":
            char = CHAR_LITERAL.match(text, i)
            if char:
                out.append(_blank(text[i:char.end()]))
                i = char.end()
                continue
            # No closing quote after one plain character. A lifetime
            # (`'a`, `'static`) stays code, but an escape or an identifier
            # run that reaches the end of the input is a truncated char
            # literal, not a lifetime.
            j = i + 1
            if j < n and text[j] == '\\':
                raise RuntimeError(
                    'unterminated char literal at line %d'
                    % (text.count('\n', 0, i) + 1))
            while j < n and text[j] in WORD_CHARS:
                j += 1
            if j == n and j > i + 1:
                raise RuntimeError(
                    'unterminated char literal at line %d'
                    % (text.count('\n', 0, i) + 1))
        out.append(ch)
        i += 1
    return ''.join(out)


def scoped_banned_in_text(text, banned, source='<synthetic>'):
    """Banned component builders declared in `text`, as `ban_scan` reports them.

    Same declaration shapes and stripper as `banned_in_text`, but the name
    list is the calling module's own `SCOPED_BANNED` entry.
    """
    stripped = strip_rust(text)
    hits = []
    for pattern in DECL_PATTERNS:
        for m in pattern.finditer(stripped):
            name = m.group(1)
            for banned_name, reason in banned:
                if name == banned_name:
                    hits.append((source, stripped.count('\n', 0, m.start(1)) + 1,
                                 name, reason))
    hits.sort(key=lambda hit: hit[1])
    return hits


def scoped_ban_scan():
    """The component-scoped builder ban over the owning modules only."""
    hits = []
    for path, banned in sorted(SCOPED_BANNED.items()):
        with io.open(path, encoding='utf-8') as handle:
            hits.extend(scoped_banned_in_text(handle.read(), banned, path))
    return hits


def banned_in_text(text, source='<synthetic>'):
    """Every banned-name declaration in `text`, as `(source, line, name, reason)`.

    Declarations are matched over the whole stripped source rather than line
    by line, so a Rust modifier or a newline between `pub` and `fn` cannot
    hide a reintroduction; the reported line is the matched name's own.
    """
    stripped = strip_rust(text)
    hits = []
    for pattern in DECL_PATTERNS:
        for m in pattern.finditer(stripped):
            name = m.group(1)
            if name in BANNED_V2:
                hits.append((source, stripped.count('\n', 0, m.start(1)) + 1,
                             name, BANNED_V2[name]))
    # A re-export reintroduces the name without declaring it
    # (`pub use legacy::Navbar;`), so every identifier on a `pub use`
    # statement's first line is checked, not just the trailing one.
    for m in PUB_USE.finditer(stripped):
        end = stripped.find('\n', m.end())
        end = len(stripped) if end < 0 else end
        line = stripped.count('\n', 0, m.start()) + 1
        for name in IDENTIFIER.findall(stripped[m.end():end]):
            if name in BANNED_V2:
                hits.append((source, line, name, BANNED_V2[name]))
    hits.sort(key=lambda hit: hit[1])
    return hits


def ban_scan():
    """The ban scan over the repository's Rust sources."""
    paths = sorted({p for g in SCAN_GLOBS for p in glob.glob(g, recursive=True)})
    if not paths:
        raise SystemExit('BAN SCAN READ NOTHING: no sources matched %s' % (SCAN_GLOBS,))
    hits = []
    for path in paths:
        with io.open(path, encoding='utf-8') as handle:
            hits.extend(banned_in_text(handle.read(), path))
    return hits

# Builders that exist for a reason no v3 prop table can express. The value is
# the reason, and the categories are deliberately few:
#
#   constructor      -- `new`, and the named constructors of a compound type
#   composition      -- v3 composes a child part (`<Label>`, `Modal.Close`,
#                       `ProgressBar.ValueLabel`); a monolithic builder takes it
#                       as a prop, or as a flag that renders the built-in part
#   no-classname     -- gpui has no `className`, so the layout and sizing v3
#                       leaves to Tailwind needs a builder
#   gpui-element-id  -- gpui requires an explicit id on a stateful element
#   state-entity     -- the text/selection lives in a gpui `Entity`, which the
#                       component is handed rather than a plain value
#   accessor         -- reads something back out; not configuration
EXTRA_OK = {
    'new': 'constructor',
    'child': 'composition',
    'children': 'composition',
    'child_toggle': 'composition',
    'button': 'composition',
    'row': 'composition',
    'keyed_row': 'composition',
    'column': 'composition',
    'crumb': 'composition',
    'label': 'composition',
    'description': 'composition',
    'error_message': 'composition',
    'title': 'composition',
    'message': 'composition',
    'content': 'composition',
    'icon': 'composition',
    'separator': 'composition',
    'start_content': 'composition',
    'end_content': 'composition',
    'footer_child': 'composition',
    'show_close_button': 'composition',
    'show_value': 'composition',
    'show_value_label': 'composition',
    'show_label': 'composition',
    'show_alpha': 'composition',
    'show_seconds': 'composition',
    'hide_steppers': 'composition',
    'closable': 'composition',
    'cancel_label': 'composition',
    'confirm_label': 'composition',
    'on_cancel': 'composition',
    'on_confirm': 'composition',
    # Dismissal-report callback: the composed CloseTrigger part and the other
    # close paths (Escape, backdrop) report through it, alongside
    # `on_open_change`.
    'on_close': 'composition',
    # `Input.ClearButton` is a v3 part of `InputGroup`; this renders it.
    'is_clearable': 'composition',
    'code': 'constructor',
    'container': 'constructor',
    'heading': 'constructor',
    'paragraph': 'constructor',
    'kind': 'constructor',
    'id': 'gpui-element-id',
    'w': 'no-classname',
    'h': 'no-classname',
    'max_h': 'no-classname',
    'max_w': 'no-classname',
    'gap': 'no-classname',
    'padding': 'no-classname',
    'mx': 'no-classname',
    'my': 'no-classname',
    'length': 'no-classname',
    'max_items': 'no-classname',
    'state': 'state-entity',
    'push': 'state-entity',
    'push_toast': 'state-entity',
    'dismiss_toast': 'state-entity',
    # v3 reaches the queue through the module-level `toast` object
    # (`toast.clear()`, `toast.pauseAll()`); the store is a gpui global here, so
    # the same calls are free functions over it.
    'clear_toasts': 'state-entity',
    'pause_toasts': 'state-entity',
    'constraints': 'state-entity',
    'validity': 'accessor',
    'selection_key': 'accessor',
    'display_text': 'accessor',
    'parse': 'accessor',
    'key': 'accessor',
    'current_color': 'no-svg-currentcolor',
    'input_type': 'renamed-kind',
    'uncontrolled': 'constructor',
    'submit_handler': 'accessor',
    'reset_handler': 'accessor',
    # `Form` is told which fields it owns, because gpui gives a child no way to
    # reach its ancestor; `data` is the collected submission.
    'field': 'no-context-propagation',
    # A control whose value is a plain prop hands the form the (name, value)
    # pair itself, because gpui gives it no way to reach its ancestor `Form`.
    'form_field': 'no-context-propagation',
    'form_fields': 'no-context-propagation',
    'data': 'accessor',
    'on_navigate': 'composition',
    # v3 spells the Breadcrumbs `separator` a `ReactNode`; a node cannot be
    # rebuilt per crumb in a RenderOnce port, so the builder takes the closure
    # that constructs it — the render-prop inversion the parity guide records.
    'separator_render': 'reactnode-render-port',
    'on_row_click': 'composition',
    'on_selection_change_all': 'composition',
    # v3 spells the multi-thumb slider `value: number[]` / `onChange`; Rust has
    # no untagged union, so the array form gets its own pair.
    'values': 'no-union-types',
    'on_change_all': 'no-union-types',
    'selected_indices': 'composition',
    'on_year_picker_open_change': 'composition',
    'is_year_picker_open': 'composition',
    # In a v3 example but not in any prop table: `<ComboBox allowsCustomValue>`
    # and `<TimeField hourCycle={12}>`.
    'allows_custom_value': 'documented-by-example',
    'hour_cycle': 'documented-by-example',
    # React Aria's ComboBox prop. v3's ComboBox inherits it and its table omits
    # it, the way that table also omits `isOpen` and `onOpenChange`.
    'menu_trigger': 'react-aria-inherited',
}

# Scoped exceptions, when a bare name would excuse the wrong component.
EXTRA_OK_SCOPED = {
    # v3 composes Disclosure children; the monolithic GPUI group names each
    # disclosure and its body in one builder call.
    'DisclosureGroup.item': 'composition',
    # v3 composes the item's supporting text inside Accordion.Item.
    'Accordion.subtitle': 'composition',
    # v3's ColorArea takes its dimensions from `className`.
    'ColorArea.size': 'no-classname',
    # v3 tints `Modal.Icon` with `className="bg-default text-foreground"`.
    'Modal.icon_color': 'no-classname',
    # v3's stylesheet declares `.range-calendar__cell-indicator`; only the
    # Calendar's prop table names the part.
    'RangeCalendar.cell_indicator': 'composition',
    # v3 documents `ColorField.Suffix` as a sub-component
    # (`.color-input-group__suffix`), which a monolithic builder takes as a slot.
    'ColorField.suffix': 'composition',
    # v3 composes `<Tabs.Separator />` inside the tab it precedes.
    'TabItem.separator': 'composition',
    # v3 composes `<Table.Footer>` under the body, where a table's pagination
    # goes.
    'Table.footer': 'composition',
    # HeroUI forwards both inherited React Aria Column resize props even
    # though its Table.Column table lists only the initial/minimum widths.
    'Table.allows_resizing': 'react-aria-inherited',
    # HeroUI forwards the inherited React Aria Column `maxWidth` prop even
    # though its Table.Column table omits it alongside other inherited props.
    'Table.max_width': 'react-aria-inherited',
    # HeroUI forwards React Aria Row's inherited `textValue`; cells are opaque
    # in gpui, so the row must expose that searchable text explicitly.
    'Table.text_value': 'react-aria-inherited',
    # MenuItem/ListBoxItem expose compound child slots rather than root props.
    'Dropdown.shortcut': 'composition',
    'Dropdown.submenu': 'composition',
    'ListBox.section': 'composition',
    'ListBox.shortcut': 'composition',
    # HeroUI forwards React Aria's inherited MultipleSelection contract even
    # though its own ListBox table does not repeat this prop.
    'ListBox.disallow_empty_selection': 'react-aria-inherited',
    'TagGroup.disallow_empty_selection': 'react-aria-inherited',
    # `[data-exiting]` belongs to Dropdown.Menu; the standalone Menu builder
    # carries that composed part state in gpui.
    'Dropdown.exiting': 'composition',
    # v3 positions Toast.Provider through layout classes.
    'Toast.inset': 'no-classname',
    # v3 composes `<Pagination.Summary>Page 1 of 10</Pagination.Summary>`.
    'Pagination.summary': 'composition',
    'Pagination.previous_icon': 'composition',
    'Pagination.next_icon': 'composition',
    # v3's Avatar composes `<Avatar.Fallback>JD</Avatar.Fallback>`.
    'Avatar.name': 'composition',
    # v3 composes these as typed child parts -- `<InputGroup.Prefix>`,
    # `<InputGroup.Input>`, `<DateField.Suffix>`. gpui has no JSX, so a named
    # slot is a builder. `InputGroup.input` takes an `Input` rather than an
    # element on purpose: the group has to strip the field's chrome, and a
    # plain child leaves a second field drawn inside the group.
    # v3 writes `<Table items={users}>{(user) => <Table.Row>}</Table>`: the rows
    # of a virtual table come from a function, because the Virtualizer calls it
    # again every time the viewport moves.
    'Table.virtual_rows': 'composition',
    'Table.virtual_text_value': 'composition',
    'Table.virtual_tree_metadata': 'composition',
    'InputGroup.prefix': 'composition',
    'InputGroup.suffix': 'composition',
    'InputGroup.input': 'composition',
    'InputGroup.text_area': 'composition',
    'DateField.prefix': 'composition',
    'DateField.suffix': 'composition',
    'TimeField.prefix': 'composition',
    'TimeField.suffix': 'composition',
    # v3 composes custom children inside its increment/decrement button parts.
    # The chevron example also uses classes to stack those parts vertically;
    # the monolithic GPUI component exposes that exact anatomy as a typed seam.
    'NumberField.increment_icon': 'composition',
    'NumberField.decrement_icon': 'composition',
    'NumberField.vertical_steppers': 'composition',
    # `ButtonGroup.Separator` is a child part in v3, composed inside whichever
    # member should show one. A monolithic group takes it as a flag.
    'ButtonGroup.separators': 'composition',
    'ToggleButtonGroup.separators': 'composition',
    # `Switch.Thumb` takes children (v3 swaps an icon per state), and the
    # label's side comes from the order of `Switch.Content`'s children.
    'Switch.thumb_icons': 'composition',
    'Switch.label_first': 'composition',
    # v3 changes the spin rate with an animation utility class
    # (`animate-[spin_1.5s_linear_infinite]`), which is its "Speed" example.
    'Spinner.duration_ms': 'no-classname',
    # `SearchField.SearchIcon` is a composed part; replacing it is v3's
    # "Custom Icons" example.
    'SearchField.search_icon': 'composition',
    'SearchField.clear_icon': 'composition',
    # HeroUI composes TriggerIndicator as a child part, while the monolithic
    # GPUI picker exposes the same seam as a typed builder.
    'DatePicker.trigger_indicator': 'composition',
    # DatePickerRoot forwards React Aria's inherited shouldCloseOnSelect even
    # though HeroUI's own root prop table does not repeat it.
    'DatePicker.should_close_on_select': 'react-aria-inherited',
    # DateRangePicker composes both parts in v3; the monolithic GPUI picker
    # exposes the same replacement seams as typed builders.
    'DateRangePicker.trigger_indicator': 'composition',
    'DateRangePicker.range_separator': 'composition',
    # React Aria's inherited close policy is forwarded by HeroUI's root.
    'DateRangePicker.should_close_on_select': 'react-aria-inherited',
    # `Checkbox.Indicator` takes children (the closure receives CheckboxState),
    # and its "Full Rounded" example rounds `Checkbox.Control` with a class.
    'Checkbox.indicator': 'composition',
    # v3 builds a Select's list out of `ListBox` parts and its trigger out of
    # `Select.Value`; a monolithic Select takes each as a slot.
    'Select.section_before': 'composition',
    'ComboBox.section_before': 'composition',
    'Autocomplete.section_before': 'composition',
    # v3 composes `ListBox.ItemIndicator` inside Autocomplete.Popover. The
    # monolithic Autocomplete projects that child render function onto rows.
    'Autocomplete.item_indicator': 'composition',
    # `Calendar.CellIndicator` marks a day (v3's event dots) and
    # `Calendar.NavButton` takes children for the paging glyphs.
    'Calendar.cell_indicator': 'composition',
    'Calendar.nav_icons': 'composition',
    # v3 documents expandable rows in prose rather than in its prop table:
    # `treeColumn` picks the column that carries the chevron, and a row's
    # `children` are the rows it nests. `tree_row` is the constructor that
    # takes a `TableRow` rather than a cell vector, so a row can carry them.
    'Table.tree_column': 'composition',
    'Table.tree_row': 'composition',
    # v3 documents `scrollOffset` on the composed `Table.LoadMore` part -- its
    # own Async Loading example writes `<Table.LoadMore ... scrollOffset={0}
    # ...>`, and React Aria's `LoadMoreSentinelProps` defaults it to one
    # viewport. The monolithic Table projects that part's prop onto itself.
    'Table.scroll_offset': 'composition',
    # HeroUI's Tabs wrapper forwards React Aria Components' Tabs props. The
    # pinned dependency exposes `keyboardActivation`, but HeroUI's own table
    # does not repeat that inherited row.
    'Tabs.keyboard_activation': 'react-aria-inherited',
    # v3 composes `Dropdown.ItemIndicator` inside each item. The monolithic
    # Dropdown projects its render-function child across keyed menu rows.
    'Dropdown.indicator_content': 'composition',
    # v3 composes one `Slider.Thumb` per value and names each thumb there. The
    # monolithic Slider projects those names as an index-ordered collection.
    'Slider.thumb_names': 'composition',
    'Select.indicator': 'composition',
    'Select.value_content': 'composition',
    'Autocomplete.value_content': 'composition',
    'ComboBox.value_content': 'composition',
    'Checkbox.is_round': 'no-classname',
    'Link.icon': 'composition',
    'Link.icon_first': 'composition',
}


def our_spelling(component):
    """Documented prop names for `component`, in our spelling."""
    ours = set()
    for p in A.props_for(component):
        ours.add(A.ALIAS.get('%s.%s' % (component, p)) or A.ALIAS.get(
            p, re.sub(r'(?<!^)(?=[A-Z])', '_', p).lower()))
        ours.add(p.lower())
    return ours


def main():
    everywhere = defaultdict(set)
    for comp in A.FILES:
        for name in our_spelling(comp):
            everywhere[name].add(comp)

    elsewhere, unknown = [], []
    for comp in sorted(A.FILES):
        owners = [comp]
        for heading, structs in A.PART_STRUCTS.items():
            if heading.startswith(comp + '.'):
                owners.extend(structs)
        methods = set().union(*(A.impl_methods.get(owner, set()) for owner in owners))
        if not methods:
            continue
        documented = our_spelling(comp)
        if not documented:
            continue
        for m in sorted(methods):
            if m in documented:
                continue
            if m in EXTRA_OK or ('%s.%s' % (comp, m)) in EXTRA_OK_SCOPED:
                continue
            if m in everywhere:
                elsewhere.append((comp, m, sorted(everywhere[m])))
            else:
                unknown.append((comp, m))

    if elsewhere:
        print('documented for a sibling component, not for this one:')
        for comp, m, where in elsewhere:
            print('  %-18s %-24s (v3 documents it on %s)'
                  % (comp, m, ', '.join(where[:4])))
        print()
    if unknown:
        print('NOT DOCUMENTED ANYWHERE IN v3:')
        for comp, m in unknown:
            print('  %-18s %s' % (comp, m))
        print()
    print('consistent-with-a-sibling : %d' % len(elsewhere))
    print('UNEXPLAINED               : %d' % len(unknown))
    print('(each unexplained name is a v2 leftover to delete, an EXTRA_OK '
          'entry, or an ALIAS)')

    hits = ban_scan()
    scoped_hits = scoped_ban_scan()
    print()
    if hits:
        print('BANNED v2 PUBLIC NAMES:')
        for source, lineno, name, reason in hits:
            print('  %s:%d  %-18s %s' % (source, lineno, name, reason))
        print()
    if scoped_hits:
        print('REMOVED COMPONENT-SCOPED BUILDERS:')
        for source, lineno, name, reason in scoped_hits:
            print('  %s:%d  %-18s %s' % (source, lineno, name, reason))
        print()
    print('banned v2 public names    : %d' % len(hits))
    print('scoped removed builders   : %d' % len(scoped_hits))
    if hits or scoped_hits:
        sys.exit(1)


def self_test():
    """Known-positive and known-negative proof for the ban scan.

    The negatives are reintroductions -- the public component, builder,
    module and re-export spellings of removed v2 names, in every declaration
    shape -- and the positives are the false-positive classes the guard must
    tolerate: migration prose, doc and block comments (nested included),
    every Rust literal shape whose interior could carry declaration text, and
    private identifiers. Two negatives prove the scanner itself stays aligned
    with rustc: a `/*` inside a string must not suppress the real
    declarations after it, and a real `pub` declaration after a literal or
    comment must still be caught. Rust modifiers and newlines between `pub`
    and `fn` must not hide a reintroduction, `fn` must not be miscaptured as
    a const name, and an unterminated literal or comment must fail loudly
    rather than blanking the rest of the input.
    """
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    prose = (
        '//! Separator — port of `@heroui/separator` (v3, formerly `Divider`).\n'
        '/// v2 spelled this `<Card isPressable>`; v3 composes a button inside Card.\n'
        '/* legacy note: AvatarGroup is gone; compose Avatar children */\n'
        'let note = "replaced the v2 Navbar with Toolbar";\n'
        '// https://example.com/Divider still works as a plain URL\n'
    )
    expect(banned_in_text(prose) == [],
           'prose, comments or strings flagged as reintroductions: %r'
           % (banned_in_text(prose),))

    private = (
        'struct DividerState;\n'
        'fn is_striped(&self) -> bool { false }\n'
        'mod navbar_tests;\n'
    )
    expect(banned_in_text(private) == [],
           'private identifiers flagged as public API: %r'
           % (banned_in_text(private),))

    # Literal interiors are blanked like comments, so a declaration-shaped
    # phrase inside any Rust literal is not a reintroduction. Every shape the
    # scanner must know: escaped, raw, byte, raw-byte strings, and a char
    # literal whose contents are a quote -- which would otherwise open a
    # string and swallow the real declarations after it.
    literals = (
        'let note = "pub struct Divider { body: u8 }";\n'
        'let raw = r#"pub fn navbar() -> u8 { 0 }"#;\n'
        'let raw_deep = r##"pub mod card_body;"##;\n'
        'let bytes = b"pub struct AvatarGroup;";\n'
        'let raw_bytes = br#"pub fn is_striped() {}"#;\n'
        'let escaped = "pub fn \\\"divider\\\"() {}";\n'
        'let quoted = \'"\';\n'
        'let hex = \'\\x44\'; let unicode = \'\\u{1F600}\';\n'
        "let newline = '\\n'; let tab = '\\t';\n"
        "let backslash = '\\\\'; let escaped_quote = '\\'';\n"
    )
    expect(banned_in_text(literals) == [],
           'a literal-borne declaration was flagged: %r'
           % (banned_in_text(literals),))

    # A single-character escape (`'\n'`, `'\t'`, `'\\'`, `'\''`) must be read
    # as a terminated char literal, not left as code: real sources compare
    # against these constantly, and a scanner that misaligns on them either
    # desynchronizes or (now) fails loudly on valid Rust.
    escape_stripped = strip_rust("let c = '\\n';\n")
    expect("'" not in escape_stripped and '\\n' not in escape_stripped,
           "a single-character escape ('\\n') was not read as a char literal: "
           '%r' % (escape_stripped,))

    # The known-negative for the old comment-only stripper: a `/*` inside a
    # string must not switch the scanner into comment mode and suppress the
    # real declarations after it.
    smuggled = (
        'let open = "/* this is not a comment";\n'
        'pub fn navbar() -> bool { false }\n'
        'pub use crate::divider::Divider;\n'
    )
    smuggled_hits = banned_in_text(smuggled)
    smuggled_names = {name for _, _, name, _ in smuggled_hits}
    expect({'navbar', 'Divider'} <= smuggled_names,
           'a `/*` inside a string suppressed the real declarations after it: '
           '%r (hit %r)' % (smuggled_names, smuggled_hits))

    # Nested block comments close where rustc closes them, so a declaration
    # inside an inner comment stays invisible while the code after the outer
    # one is still read.
    nested = (
        '/* outer /* inner pub struct Divider; */ still comment */\n'
        'pub fn card_body() -> impl IntoElement { todo!() }\n'
    )
    nested_hits = banned_in_text(nested)
    expect([name for _, _, name, _ in nested_hits] == ['card_body'],
           'a nested block comment leaked or overreached: %r' % (nested_hits,))

    # Rust modifiers and newlines between `pub` and `fn` are part of the same
    # public declarations, and each of these once slipped the scan. `pub const
    # fn` must also not be miscaptured as a const named `fn`.
    modified = (
        'pub async fn navbar() -> u8 { 0 }\n'
        'pub const fn card_body() -> u8 { 0 }\n'
        'pub unsafe fn is_striped() -> bool { false }\n'
        'pub extern "C" fn avatar_group() {}\n'
        'pub async unsafe extern "C" fn divider() {}\n'
        'pub unsafe const fn circular_progress() {}\n'
        'pub\nfn date_input() {}\n'
        'pub\n  async\nfn number_input() {}\n'
    )
    mod_hits = banned_in_text(modified)
    mod_names = {name for _, _, name, _ in mod_hits}
    expect({'navbar', 'card_body', 'is_striped', 'avatar_group', 'divider',
            'circular_progress', 'date_input', 'number_input'} <= mod_names,
           'a modified pub fn slipped the ban scan: %r (hit %r)'
           % (mod_names, mod_hits))
    expect('fn' not in mod_names, '`fn` was miscaptured as a const name')
    expect(any(name == 'date_input' and line == 8
               for _, line, name, _ in mod_hits),
           'the line reported for a name after a split `pub\\nfn` was not the '
           "name's own line: %r" % (mod_hits,))

    reintroduced = (
        'pub struct AvatarGroup { avatars: Vec<Avatar> }\n'
        'pub enum CardBody { V1 }\n'
        'pub type CircularProgress = u8;\n'
        'impl AvatarGroup {\n'
        '    pub fn is_striped(mut self, v: bool) -> Self { self }\n'
        '}\n'
        'pub fn card_body() -> impl IntoElement { todo!() }\n'
        'pub const divider: u8 = 0;\n'
        'pub static navbar: u8 = 0;\n'
        'pub mod date_input;\n'
        'pub use crate::separator::Divider;\n'
    )
    hits = banned_in_text(reintroduced)
    names = {name for _, _, name, _ in hits}
    expect({'AvatarGroup', 'CardBody', 'CircularProgress', 'is_striped',
            'card_body', 'divider', 'navbar', 'date_input', 'Divider'} <= names,
           'reintroduced public names missed: %r (hit %r)' % (names, hits))

    # An unterminated string, raw string, char literal or block comment once
    # blanked everything after it, hiding the real declarations that followed;
    # each must fail loudly instead.
    for unreadable in (
        'let s = "pub struct Divider;\n',
        'let raw = r#"pub struct Divider;\n',
        "let c = 'a",
        "let c = '\\n",
        '/* pub struct Divider;\n',
    ):
        try:
            strip_rust(unreadable)
        except RuntimeError:
            pass
        else:
            failures.append('an unterminated literal or comment was blanked '
                            'silently instead of failing: %r' % unreadable)

    # Ordinary lifetime syntax is not a char literal and stays readable code.
    lifetimes = (
        "fn f<'a>(x: &'a str) -> &'a str { x }\n"
        "static S: &'static str = \"pub struct Divider\";\n"
    )
    expect(banned_in_text(lifetimes) == [],
           'ordinary lifetime syntax was read as a char literal: %r'
           % (banned_in_text(lifetimes),))

    expect('AvatarGroup' in BANNED_V2 and 'Divider' in BANNED_V2,
           'the minimum component bans are missing')
    expect('is_destructive' in BANNED_V2 and 'hide_close_button' in BANNED_V2,
           'the removed v2 prop bans are missing')
    expect('isDestructive' not in A.ALIAS and 'isExternal' not in A.ALIAS,
           'dead v2 aliases are back in api_audit.ALIAS')
    expect('isLoading' not in A.ALIAS
           and A.ALIAS.get('Table.LoadMore.isLoading') == 'is_pending',
           'the isLoading narrowing is missing from api_audit.ALIAS')

    # The component-scoped ban: the removed Badge/Chip `content` and
    # `start_content` builders are flagged inside their own modules in every
    # declaration shape, while the global ban leaves the same spellings alone
    # (they are legitimate elsewhere: `Input.start_content` is v3's real
    # `startContent`, and field/list render props keep the `content` spelling).
    badge_banned = SCOPED_BANNED['crates/herogpui-components/src/badge.rs']
    chip_banned = SCOPED_BANNED['crates/herogpui-components/src/chip.rs']
    scoped_reintroduced = (
        'pub fn content(mut self, el: impl IntoElement) -> Self { self }\n'
        'impl Badge {\n'
        '    pub\nfn start_content(mut self, el: impl IntoElement) -> Self { self }\n'
        '}\n'
    )
    scoped_hits = scoped_banned_in_text(scoped_reintroduced, badge_banned)
    expect({name for _, _, name, _ in scoped_hits} == {'content', 'start_content'},
           'a reintroduced Badge builder escaped the scoped ban: %r' % (scoped_hits,))
    expect(scoped_banned_in_text(scoped_reintroduced, chip_banned) != [],
           'the chip scope did not flag the same reintroductions: %r'
           % (scoped_banned_in_text(scoped_reintroduced, chip_banned),))
    expect(banned_in_text('pub fn content(mut self, el: impl IntoElement) -> Self { self }\n') == [],
           'the global ban flagged a name that is only scoped, breaking Button.content')
    expect(banned_in_text(
        'pub fn start_content(mut self, el: impl IntoElement) -> Self { self }\n') == [],
        'the global ban flagged start_content, breaking Input.start_content')
    expect(banned_in_text(
        'pub fn end_content(mut self, el: impl IntoElement) -> Self { self }\n') == [],
        'the global ban flagged end_content, breaking Input.end_content')
    expect({'content', 'start_content'} == {name for name, _ in badge_banned}
           and {'content', 'start_content'} == {name for name, _ in chip_banned},
           'the scoped builder bans are missing')
    expect(sorted(SCOPED_BANNED) == [
        'crates/herogpui-components/src/badge.rs',
        'crates/herogpui-components/src/button.rs',
        'crates/herogpui-components/src/chip.rs',
    ], 'the scoped ban does not cover exactly the badge, button and chip modules')

    # Known-negative for the Button scope: reintroduced `start_content` and
    # `end_content` builders in `button.rs` are flagged (the removed v2 slot
    # seams), while `Button.content` -- v3's children-as-a-function -- stays
    # allowed there.
    button_banned = SCOPED_BANNED['crates/herogpui-components/src/button.rs']
    button_hits = scoped_banned_in_text(
        'impl Button {\n'
        '    pub\nfn start_content(mut self, el: impl IntoElement) -> Self { self }\n'
        '    pub fn end_content(mut self, el: impl IntoElement) -> Self { self }\n'
        '}\n',
        button_banned)
    expect({name for _, _, name, _ in button_hits}
           == {'start_content', 'end_content'}
           and all(reason.startswith('removed Button builder')
                   for _, _, _, reason in button_hits),
           'a reintroduced Button.start_content/end_content escaped the scoped '
           'ban: %r' % (button_hits,))
    expect(scoped_banned_in_text(
        'impl Button {\n'
        '    pub fn content(\n'
        '        mut self,\n'
        '        render: impl Fn(InteractiveState) -> AnyElement + \'static,\n'
        '    ) -> Self { self }\n'
        '}\n',
        button_banned) == [],
        'the button scope flagged Button.content, which v3 keeps as its '
        'children-as-a-function')

    if failures:
        print('self-test FAIL')
        for failure in failures:
            print('- %s' % failure)
        return 1
    print('self-test PASS: the ban scan flags public v2 reintroductions in '
          'every declaration shape, including modified and split `pub fn`, '
          'fails loudly on unterminated literals and comments, and tolerates '
          'prose, comments, strings, lifetimes and private identifiers')
    return 0


if __name__ == '__main__':
    if '--self-test' in sys.argv[1:]:
        sys.exit(self_test())
    main()
