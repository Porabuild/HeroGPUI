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
    'LabelAndMessages': 'field_slots',
    'Label & Messages': 'field_slots',
}

# v3 example name -> the gallery section that covers it, where the titles are
# genuinely different words for the same demo.
ALIAS = {}

# Examples that cannot be demonstrated in this port, with the reason. Keyed
# `Component.Example`.
WONT_DEMO = {}

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
            out.setdefault(name, []).extend(titles)
    return out


def suffix_for(page):
    if page in PAGE_ALIAS:
        return PAGE_ALIAS[page]
    return re.sub(r'(?<!^)(?=[A-Z])', '_', page.replace(' ', '')).lower()


def main():
    v3 = v3_examples()
    ours = our_sections()

    rows, total, covered, unportable = [], 0, 0, 0
    missing_by_page = {}
    for page in sorted(v3):
        suffix = suffix_for(page)
        if suffix not in ours:
            continue          # a docs page, not a component page
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
            if key in WONT_DEMO:
                unportable += 1
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
    print('MISSING             : %d  (in %d pages)'
          % (total - covered - unportable, len(missing_by_page)))


if __name__ == '__main__':
    main()
