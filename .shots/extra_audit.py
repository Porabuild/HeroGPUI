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
import io
import os
import re
import sys
from collections import defaultdict

sys.stdout.reconfigure(encoding='utf-8', errors='replace')
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import api_audit as A  # noqa: E402  (reuses its parsing, tables and bundle)

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
    'hide_close_button': 'composition',
    'show_close_button': 'composition',
    'show_value': 'composition',
    'show_value_label': 'composition',
    'show_label': 'composition',
    'show_alpha': 'composition',
    'show_seconds': 'composition',
    'hide_steppers': 'composition',
    'closable': 'composition',
    'is_closable': 'composition',
    'cancel_label': 'composition',
    'confirm_label': 'composition',
    'on_cancel': 'composition',
    'on_confirm': 'composition',
    'is_destructive': 'composition',
    # `Modal.Close` / `Drawer.Close` are v3 parts with their own press handler;
    # here the part is built in and this is its handler.
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
    # v3's ColorArea takes its dimensions from `className`.
    'ColorArea.size': 'no-classname',
    # v3 tints `Modal.Icon` with `className="bg-default text-foreground"`.
    'Modal.icon_color': 'no-classname',
    # v3 composes `<Tabs.Separator />` inside the tab it precedes.
    'TabItem.separator': 'composition',
    # v3 composes `<Table.Footer>` under the body, where a table's pagination
    # goes.
    'Table.footer': 'composition',
    # v3 composes `<Pagination.Summary>Page 1 of 10</Pagination.Summary>`.
    'Pagination.summary': 'composition',
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
    'InputGroup.prefix': 'composition',
    'InputGroup.suffix': 'composition',
    'InputGroup.input': 'composition',
    'InputGroup.text_area': 'composition',
    'DateField.prefix': 'composition',
    'DateField.suffix': 'composition',
    'TimeField.prefix': 'composition',
    'TimeField.suffix': 'composition',
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
    # `Checkbox.Indicator` takes children (v3 swaps the glyph per state), and
    # its "Full Rounded" example rounds `Checkbox.Control` with a class.
    'Checkbox.indicator': 'composition',
    # v3 builds a Select's list out of `ListBox` parts and its trigger out of
    # `Select.Value`; a monolithic Select takes each as a slot.
    'Select.section_before': 'composition',
    'ComboBox.section_before': 'composition',
    'Autocomplete.section_before': 'composition',
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
    'Select.indicator': 'composition',
    'Select.value_content': 'composition',
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
        if comp not in A.impl_methods:
            continue
        documented = our_spelling(comp)
        if not documented:
            continue
        for m in sorted(A.impl_methods[comp]):
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


if __name__ == '__main__':
    main()
