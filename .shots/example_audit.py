"""Diff v3's documented *examples* against the gallery's demo sections.

The prop, design and motion audits all ask whether a component *can* do
something. None of them asks whether the gallery ever shows it. v3's docs are a
list of worked examples per component -- `## Examples` with one `### Name` each
-- and the demo app is only a faithful port of the docs if every one of them is
on the page.

Both sides are read mechanically:

- v3: `# <Component>` page, then the `### ` headings under its `## Examples`
  heading, in `llms-full.txt`.
- ours: the section titles in the `vec![("Title", ..)]` list each
  `pub fn page_*` hands to `doc_page`.

Titles are matched on a normalised form (lowercase, alphanumeric only, a few
synonyms), because v3 writes "Disabled State" where the gallery writes
"Disabled". `ALIAS` records the pairs that normalisation cannot reach, and
`WONT_DEMO` records examples that cannot be shown here, with a reason.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

BUNDLE = os.environ.get(
    'HEROUI_BUNDLE',
    os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt'),
)
PAGES = ('gallery/src/pages/components.rs', 'gallery/src/pages/docs.rs')

# v3 page name -> our `page_*` function suffix, where the mechanical
# CamelCase -> snake_case conversion does not land on it.
PAGE_ALIAS = {
    'InputOTP': 'input_otp',
    # v3 documents the field parts on their own pages; this port shows all four
    # on one "Label & Messages" page, the way v3's sidebar groups them.
    'Label': 'field_slots',
    'Description': 'field_slots',
    'ErrorMessage': 'field_slots',
    'FieldError': 'field_slots',
    # One page per component pair, as v3's sidebar has it.
    'ToggleButtonGroup': 'toggle_button',
    'DisclosureGroup': 'disclosure',
}
# v3 pages that are not component documentation at all.
NOT_A_COMPONENT = ('MCP Server',)

# v3 example name -> the gallery section that covers it, where the titles are
# genuinely different words for the same demo.
ALIAS = {
    # v3 names the same demo two ways across a page pair: `ListBox` documents
    # "Custom Check Icon" on its own page and "Custom Indicator" in the
    # `ListBox` section of another, and both replace `ListBox.ItemIndicator`.
    'ListBox.Custom Indicator': 'Custom Check Icon',
}

# Examples that cannot be demonstrated in this port, with the reason.
#
# `WONT_DEMO_NAMES` applies to an example name wherever it appears -- v3
# documents "Render Function" on 31 pages and it is the same prop every time --
# and `WONT_DEMO` is for one page's example, keyed `Component.Example`.
WONT_DEMO_NAMES = {
    # v3's `render` prop replaces the *DOM element* a component renders
    # (`render={(props) => <div {...props} data-custom="foo" />}`). gpui has no
    # DOM and no element to substitute, which is why `api_audit.py` skips the
    # prop too. The state those functions receive is reachable a different way:
    # the caller owns the state entity, so "Render Props" is demonstrated.
    'Render Function': 'no-dom-element',
}
WONT_DEMO = {
    # v3's example is about integrating TanStack Table, an npm package.
    'Table.TanStack Table': 'third-party-lib',
    # v3 composes a third-party npm ripple component as a child
    # (`<Button><Ripple /></Button>`); the example is about that library.
    'Button.Adding Ripple Effect': 'third-party-lib',
    # Both need CLDR data for non-Gregorian calendars; the port is Gregorian.
    'Calendar.International Calendars': 'no-intl',
    'RangeCalendar.International Calendars': 'no-intl',
    'DatePicker.International Calendar': 'no-intl',
    'DateRangePicker.International Calendar': 'no-intl',
    # A React portal renders outside the tree. gpui paints in tree order and
    # `util::floating` (deferred) is the only lift there is, so there is no
    # "render this dialog somewhere else" to show.
    'AlertDialog.Custom Portal': 'no-portal',
    'Modal.Custom Portal': 'no-portal',
}

# Examples that need a component feature this port has not built yet.
#
# These are not excused: they are counted separately so the number cannot hide
# behind "unportable", and each one names the feature it is waiting on.
NEEDS_FEATURE = {}

SYNONYM = {
    'usage': 'basic',
    'basicusage': 'basic',
    'disabledstate': 'disabled',
    'loadingstate': 'pending',
    'loading': 'pending',
    'controlledstate': 'controlled',
    'controlledopenstate': 'controlledopen',
    'requiredfield': 'required',
    'errormessage': 'error',
    'witherrormessage': 'error',
    'renderfunction': 'render',
    'renderprop': 'render',
    'fullwidth': 'fullwidth',
}


def norm(title):
    t = re.sub(r'[^a-z0-9]', '', title.lower())
    return SYNONYM.get(t, t)


def v3_examples():
    """`{page: [example, ...]}` from the docs bundle."""
    text = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
    pages, page, section = {}, None, None
    for line in text.split('\n'):
        if line.startswith('# '):
            page = line[2:].strip()
            pages.setdefault(page, [])
            section = None
        elif line.startswith('## ') and page:
            section = line[3:].strip()
            # The `## Usage` block holds the basic example every page opens with.
            if section == 'Usage':
                pages[page].append('Usage')
        elif line.startswith('### ') and page and section == 'Examples':
            name = line[4:].strip()
            if name not in pages[page]:
                pages[page].append(name)
    return {k: v for k, v in pages.items() if v}


def our_sections():
    """`{page_fn_suffix: [section title, ...]}` from the gallery source."""
    out = {}
    for path in PAGES:
        src = io.open(path, encoding='utf-8', errors='replace').read()
        # Split on the page functions so a title is attributed to its own page.
        parts = re.split(r'\n    pub fn page_([a-z0-9_]+)\(', src)
        for name, body in zip(parts[1::2], parts[2::2]):
            # A section is the first element of a tuple in `doc_page`'s vec.
            # Match on indentation, not on `("..",` anywhere: the loose form
            # also picked up every element id and tab key in the page body,
            # which made the coverage number meaningless.
            titles = re.findall(r'^                \("([^"]+)",', body, re.M)
            titles += re.findall(
                r'^ {16}\(\s*\n {20}"([^"]+)",', body, re.M)
            # A page with a single section is written `vec![(` on one line, so
            # that tuple's `(` never starts a line of its own.
            titles += re.findall(r'^ {12}vec!\[\(\s*\n {16}"([^"]+)",', body, re.M)
            out.setdefault(name, []).extend(titles)
    return out


def suffix_for(page):
    if page in PAGE_ALIAS:
        return PAGE_ALIAS[page]
    return re.sub(r'(?<!^)(?=[A-Z])', '_', page.replace(' ', '')).lower()


def main():
    v3 = v3_examples()
    ours = our_sections()

    rows, total, covered, unportable, waiting = [], 0, 0, 0, 0
    missing_by_page = {}
    for page in sorted(v3):
        if page in NOT_A_COMPONENT:
            continue
        suffix = suffix_for(page)
        if suffix not in ours:
            # Not a component page (a getting-started guide, a migration note).
            # Anything that *is* one and lands here is a missing `PAGE_ALIAS`,
            # which would silently drop its examples from the count.
            continue
        have = {norm(t) for t in ours[suffix]}
        # A gallery section often covers two v3 examples ("Pending & disabled");
        # split on the separators so each half counts.
        for t in ours[suffix]:
            for part in re.split(r'[&,/]| and ', t):
                if part.strip():
                    have.add(norm(part))
        miss = []
        for ex in v3[page]:
            total += 1
            key = '%s.%s' % (page, ex)
            if key in WONT_DEMO or ex in WONT_DEMO_NAMES:
                unportable += 1
                continue
            if key in NEEDS_FEATURE:
                waiting += 1
                continue
            target = norm(ALIAS.get(key, ALIAS.get(ex, ex)))
            if target in have or any(target in h or h in target for h in have if len(h) > 3):
                covered += 1
            else:
                miss.append(ex)
        if miss:
            missing_by_page[page] = miss
        rows.append((page, len(v3[page]), len(v3[page]) - len(miss), miss))

    print('%-24s %5s %5s  %s' % ('v3 page', 'docs', 'ours', 'not demonstrated'))
    for page, n, ok, miss in rows:
        mark = ' ' if not miss else '!'
        print('%s %-22s %5d %5d  %s' % (mark, page, n, ok, ', '.join(miss)))
    print()
    print('examples documented : %d' % total)
    print('demonstrated        : %d' % covered)
    print('recorded unportable : %d' % unportable)
    print('waiting on a feature: %d  (%s)'
          % (waiting, ', '.join(sorted(set(NEEDS_FEATURE.values())))))
    print('MISSING             : %d  (in %d pages)'
          % (total - covered - unportable - waiting, len(missing_by_page)))


if __name__ == '__main__':
    main()
