"""Diff the props v3's examples *use* against the ones the gallery's page shows.

`example_audit.py` matches example **names**: it says every one of v3's 616
worked examples has a section on the page. It cannot say whether those sections
demonstrate the same thing -- the Tabs "With Separator" demo stood a `Separator`
next to two plain tabs for months and the name matched perfectly.

So this reads the *code* on both sides:

- v3: every JSX attribute in every ```tsx block under a page's `## Usage` and
  `## Examples`, kept only if v3 documents it as a prop of that component
  (`api_audit.props_for`) *and* this port implements it. A prop we have not
  built is `api_audit.py`'s business.
- ours: the Rust of that page's demo sections, and whether the builder porting
  the prop is called anywhere in them.

The unit is the **page**, not the example. v3's snippets set plenty of props to
their default value (`selectionMode="single"`, `delay={700}`), and a demo that
leaves those out shows the same thing -- comparing example by example reported
656 of those. What is worth knowing is whether the gallery ever exercises a prop
v3's docs exercise, because a prop nothing demonstrates is a prop nobody has
looked at since it was written.

`WONT_DEMO_PROPS` records the ones that stay unshown, with a reason.

    python .shots/demo_audit.py            # the report
    python .shots/demo_audit.py Tabs       # one page, verbosely
    python .shots/demo_audit.py --fetch    # refresh cached preview sources
"""
import hashlib
import io
import os
import re
import sys
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import api_audit  # noqa: E402  (reads the bundle and the sources on import)
import example_audit as ex  # noqa: E402

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

# JSX attributes that are not props of the component under test: DOM plumbing,
# SVG geometry from the inline icons, and the form attributes v3 documents but a
# gpui port has nothing to submit to.
IGNORE = {
    'className', 'style', 'key', 'ref', 'id', 'slot', 'children', 'render',
    'asChild', 'htmlFor', 'type', 'href', 'target', 'rel', 'src', 'alt',
    'width', 'height', 'viewBox', 'fill', 'stroke', 'strokeWidth', 'd',
    'strokeLinecap', 'strokeLinejoin', 'xmlns', 'aria', 'role', 'tabIndex',
    'data', 'name', 'form', 'autoComplete', 'autoFocus', 'inputMode',
    'enterKeyHint', 'spellCheck', 'maxLength', 'minLength', 'pattern',
}

# A prop v3's examples exercise that this port's gallery deliberately does not,
# with the reason. Keyed `Component.prop`.
WONT_DEMO_PROPS = {}

PREVIEW_CACHE = os.path.join(os.environ.get('TEMP', '/tmp'),
                             'herogpui-demo-audit')


def camel_to_snake(name):
    return re.sub(r'(?<!^)(?=[A-Z])', '_', name).lower().replace('__', '_')


def v3_pages():
    """`{page: {example: code}}` for every documented example."""
    text = api_audit.bundle
    pages = {}
    for pm in re.finditer(r'^# (.+?)[ \t]*$', text, re.M):
        name = pm.group(1).strip()
        start = pm.end()
        nxt = re.search(r'^# ', text[start:], re.M)
        body = text[start:start + nxt.start()] if nxt else text[start:]
        blocks = {}
        m = re.search(r'^## Usage[ \t]*$(.*?)(?=^## )', body, re.S | re.M)
        if m:
            blocks['Usage'] = m.group(1)
        m = re.search(r'^## Examples[ \t]*$(.*?)(?=^## )', body, re.S | re.M)
        if m:
            for em in re.finditer(r'^### (.+?)[ \t]*$(.*?)(?=^### |\Z)',
                                  m.group(1), re.S | re.M):
                blocks[em.group(1).strip()] = em.group(2)
        if blocks:
            pages[name] = blocks
    return pages


def props_used(chunk):
    """JSX attributes named in one example's code."""
    used = set()
    for code in re.findall(r'```tsx(.*?)```', chunk, re.S):
        used |= set(re.findall(r'[\s{]([a-z][A-Za-z0-9]*)=', code))
        # Boolean JSX props have no `=` (`<Tabs.Tab isDisabled id="x">`).
        # Read them only inside opening component tags so prose and JavaScript
        # identifiers cannot masquerade as exercised props.
        for attrs in re.findall(r'<[A-Z][A-Za-z0-9.]*\b(.*?)/?>', code, re.S):
            used |= set(re.findall(
                r'(?:^|\s)([a-z][A-Za-z0-9]*)'
                r'(?=\s+(?:[a-z][A-Za-z0-9]*)(?:\s*=|\s|$)|\s*$)',
                attrs,
            ))
    return used - IGNORE


def fetch_text(url, refresh=False):
    """Read one v3 source file, cached so the full gate stays fast."""
    os.makedirs(PREVIEW_CACHE, exist_ok=True)
    name = hashlib.sha256(url.encode('utf-8')).hexdigest() + '.txt'
    path = os.path.join(PREVIEW_CACHE, name)
    if refresh or not os.path.exists(path):
        try:
            with urllib.request.urlopen(url, timeout=20) as response:
                text = response.read().decode('utf-8')
        except Exception as error:
            if not refresh and os.path.exists(path):
                return io.open(path, encoding='utf-8', errors='replace').read()
            raise RuntimeError('cannot read v3 demo source %s: %s' %
                               (url, error)) from error
        with io.open(path, 'w', encoding='utf-8', newline='\n') as cached:
            cached.write(text)
    return io.open(path, encoding='utf-8', errors='replace').read()


def bundle_page(page):
    """The English bundle body for `page`, including its source URL."""
    for pm in re.finditer(r'^# (.+?)[ \t]*$', api_audit.bundle, re.M):
        if pm.group(1).strip() != page:
            continue
        start = pm.end()
        nxt = re.search(r'^# ', api_audit.bundle[start:], re.M)
        return api_audit.bundle[start:start + nxt.start()] if nxt else api_audit.bundle[start:]
    return ''


def preview_props(page, refresh=False):
    """Props in ComponentPreview sources omitted from the rendered bundle."""
    body = bundle_page(page)
    source = re.search(r'^\*\*Source\*\*: (https://\S+)[ \t]*$', body, re.M)
    if not source:
        return set()
    mdx = fetch_text(source.group(1), refresh)
    previews = re.findall(r'<ComponentPreview\b.*?\bname="([^"]+)".*?/>',
                          mdx, re.S)
    if not previews:
        return set()

    slug = ex.suffix_for(page).replace('_', '-')
    used = set()
    for preview in previews:
        prefix = slug + '-'
        if not preview.startswith(prefix):
            raise RuntimeError('%s preview %s does not start with %s' %
                               (page, preview, prefix))
        demo = preview[len(prefix):]
        url = ('https://raw.githubusercontent.com/heroui-inc/heroui/v3/'
               'apps/docs/src/demos/en/%s/%s.tsx' % (slug, demo))
        code = fetch_text(url, refresh)
        used |= props_used('```tsx\n%s\n```' % code)
    return used


def check_jsx_parser():
    """Keep boolean shorthand from silently falling out of the input set."""
    probe = '```tsx\n<Tabs.Tab isDisabled id="disabled" />\n```'
    if props_used(probe) != {'isDisabled'}:
        raise RuntimeError('JSX prop reader lost boolean shorthand attributes')


def our_pages():
    """`{page_fn_suffix: rust source of every demo section}`."""
    out = {}
    for path in ex.PAGES:
        src = io.open(path, encoding='utf-8', errors='replace').read()
        parts = re.split(r'\n    pub fn page_([a-z0-9_]+)\(', src)
        for name, body in zip(parts[1::2], parts[2::2]):
            out[name] = out.get(name, '') + body
    return out


def main():
    check_jsx_parser()
    refresh = '--fetch' in sys.argv[1:]
    pages = [arg for arg in sys.argv[1:] if arg != '--fetch']
    only = pages[0] if pages else None
    v3 = v3_pages()
    ours = our_pages()

    rows = []
    pages_checked = exercised = shown = excused = 0
    for page in sorted(v3):
        if only and page != only:
            continue
        if page in ex.NOT_A_COMPONENT:
            continue
        suffix = ex.suffix_for(page)
        if suffix not in ours:
            continue
        component = page.replace(' ', '')
        documented = api_audit.props_for(component)
        if not documented:
            continue
        implemented = set()
        ctor = set()
        for struct in [component] + list(api_audit.COMPANIONS.get(component, ())):
            implemented |= api_audit.impl_methods.get(struct, set())
            ctor |= api_audit.constructor_args.get(struct, set())

        used = set()
        for chunk in v3[page].values():
            used |= props_used(chunk)
        # llms-full normally expands ComponentPreview source into each example,
        # but a missing expansion used to make Link and Tabs report a vacuous
        # zero. Fall back to the page's real preview files rather than treating
        # an empty extraction as a pass.
        if not used:
            used |= preview_props(page, refresh)

        pages_checked += 1
        missing = []
        for prop in sorted(used):
            if prop not in documented:
                continue
            ported = api_audit.ALIAS.get('%s.%s' % (component, prop),
                                         api_audit.ALIAS.get(prop, prop))
            names = {camel_to_snake(ported), camel_to_snake(prop)}
            if names & ctor:
                # A prop the constructor takes positionally cannot be shown by
                # name: `Table::new(columns)` *is* `columns`, and every demo that
                # builds a table exercises it.
                continue
            if not (names & implemented):
                continue  # not implemented: api_audit's business
            exercised += 1
            if any(re.search(r'\b%s\b' % re.escape(n), ours[suffix]) for n in names):
                shown += 1
                continue
            if '%s.%s' % (component, prop) in WONT_DEMO_PROPS:
                excused += 1
                continue
            missing.append(prop)
        if missing:
            rows.append((page, missing))
        if only:
            print('%s: v3 exercises %d props, %d documented+ported'
                  % (page, len(used), exercised))

    for page, missing in rows:
        print('! %-22s %s' % (page, ', '.join(missing)))
    print()
    print('pages compared      : %d' % pages_checked)
    print('props v3 exercises  : %d' % exercised)
    print('demonstrated here   : %d' % shown)
    print('recorded won-t-demo : %d' % excused)
    print('NOT DEMONSTRATED    : %d  (on %d pages)'
          % (sum(len(m) for _, m in rows), len(rows)))


if __name__ == '__main__':
    main()
