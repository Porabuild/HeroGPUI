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

Alias resolution mirrors `api_audit.main`'s precedence for one prop, most
specific first: the part- or fold-scoped alias (`Comp.Part.prop`,
`Comp.Heading.prop`) answers when the heading that documents the row has one --
the fold-scoped alias is honored before `FOLD_STRUCTS` ownership gating, exactly
as `api_audit.main` does -- then the component-scoped alias (`Comp.prop`), then
the global. That order is what keeps a narrowed global working where its scope
survives: the global `isLoading -> is_pending` was deleted because v3 renamed
the prop to `isPending` everywhere but the `Table.LoadMore` sentinel row, and
the part heading `props_for_state` reports is what scopes the mapping back. A
wrong part never reaches the alias: its key is simply absent, and the raw
spelling falls through to be checked against the implementation.

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


def resolve_alias(component, prop, parts, folds):
    """The builder spelling `prop` resolves to for `component`.

    The chain is `api_audit.main`'s, read for one prop, most specific first:
    the part-scoped alias (`Comp.Part.prop`) when a part heading documents the
    row, then the fold-scoped one (`Comp.Heading.prop`) -- honored whether or
    not `FOLD_STRUCTS` scopes the heading, as `api_audit.main` honors it before
    its ownership gate -- then the component-scoped alias (`Comp.prop`), then
    the global. That order is what keeps a narrowed global working where its
    scope survives: the global `isLoading -> is_pending` was deleted because v3
    renamed the prop to `isPending` everywhere but the `Table.LoadMore`
    sentinel row, and the part heading `props_for_state` reports is what scopes
    the mapping back. A wrong part never reaches the alias: its key is simply
    absent, and the resolution falls through to the component and global tiers
    exactly as `api_audit.main` would. (The owned-structs check a scoped fold
    applies to a global answer stays `api_audit.main`'s business: this resolver
    returns a spelling, and the caller checks it against the whole component's
    implementation set.)
    """
    alias = api_audit.ALIAS
    part = next((pt for pt in parts if prop in parts[pt]), None)
    fold = next((h for h in folds if prop in folds[h]), None)
    part_key = '%s.%s.%s' % (component, part, prop) if part else None
    fold_key = '%s.%s.%s' % (component, fold, prop) if fold else None
    comp_key = '%s.%s' % (component, prop)
    if part:
        # A part row: the part's own alias first, then the component's, then
        # the global -- `api_audit.main`'s `if part and part_key in ALIAS`
        # branch, order preserved.
        if part_key in alias:
            return alias[part_key]
        if comp_key in alias:
            return alias[comp_key]
        return alias.get(prop, prop)
    if fold:
        # The fold-scoped alias (`Comp.Heading.prop`) answers first, whether
        # or not `FOLD_STRUCTS` scopes the heading -- `api_audit.main` honors
        # it before its ownership gate. A scoped fold then answers through the
        # component and global tiers; an unscoped fold (`### Render Props`,
        # `### ToastQueue`) documents the component's own state, so the same
        # tiers answer it -- `api_audit.main`'s no-entry branch.
        if fold_key in alias:
            return alias[fold_key]
        if comp_key in alias:
            return alias[comp_key]
        return alias.get(prop, prop)
    if comp_key in alias:
        return alias[comp_key]
    return alias.get(prop, prop)


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


def self_test():
    """Known-positive and known-negative proof for the alias resolution.

    The precedence proof is the real Dropdown conflict: `isSelected` is
    documented on the `Dropdown.ItemIndicator` part and carries a
    component-scoped alias too, so the part-scoped `indicator_content` must
    win, and a wrong part must fall through to the component mapping rather
    than borrow the part alias. The known-positive after that is the
    regression this script had: narrowing `api_audit`'s global
    `isLoading -> is_pending` to the part-scoped `Table.LoadMore.isLoading`
    dropped Table's row out of the exercised tally, because the resolver only
    read component-scoped and global keys. The known-negatives are the two
    ways that fix could rot: a global alias creeping back, and a wrong part
    reaching a part-scoped alias.
    """
    failures = []

    def expect(condition, message):
        if not condition:
            failures.append(message)

    def ports(component, prop, parts, folds, implemented):
        ported = resolve_alias(component, prop, parts, folds)
        names = {camel_to_snake(ported), camel_to_snake(prop)}
        return bool(names & implemented)

    # Part-scoped beats component-scoped: v3 documents `isSelected` on the
    # `### Dropdown.ItemIndicator` part, `api_audit.ALIAS` answers it with the
    # part-scoped `indicator_content`, and the component-scoped
    # `Dropdown.isSelected -> item_content` also exists -- so this is the
    # known conflict: the part tier must win.
    expect(resolve_alias('Dropdown', 'isSelected',
                         {'ItemIndicator': {'isSelected'}}, {})
           == 'indicator_content',
           'the part-scoped Dropdown.ItemIndicator.isSelected lost to the '
           'component-scoped Dropdown.isSelected')
    real_dropdown = api_audit.props_for_state('Dropdown')
    expect(real_dropdown is not None and
           resolve_alias('Dropdown', 'isSelected',
                         real_dropdown[1], real_dropdown[2]) == 'indicator_content',
           'the real Dropdown part tables did not resolve isSelected to '
           'indicator_content')
    # A wrong part cannot borrow the Indicator alias: without that part
    # heading, the resolution falls through to the component-scoped mapping,
    # never to the part-scoped one.
    expect(resolve_alias('Dropdown', 'isSelected', {'Popover': {'isSelected'}}, {})
           == 'item_content',
           'a wrong Dropdown part borrowed the ItemIndicator alias')

    # Part-scoped resolution works: the heading that documents the row is
    # the context, whether handed in directly or read from the real bundle.
    expect(resolve_alias('Table', 'isLoading', {'LoadMore': {'isLoading'}}, {})
           == 'is_pending', 'part-scoped Table.LoadMore.isLoading did not resolve')
    real = api_audit.props_for_state('Table')
    expect(real is not None and
           resolve_alias('Table', 'isLoading', real[1], real[2]) == 'is_pending',
           'the real Table part tables did not resolve isLoading to is_pending')
    expect(ports('Table', 'isLoading', {'LoadMore': {'isLoading'}}, {},
                 {'is_pending'}), 'the resolved part alias did not port')

    # An unscoped fold alias is honored before FOLD_STRUCTS gating, exactly as
    # `api_audit.main` honors `Comp.Heading.prop` before its ownership check:
    # v3 documents the render-prop rows under a bare `### Render Props`
    # heading, which `FOLD_STRUCTS` deliberately does not scope, and
    # `NumberField.Render Props.isDisabled -> content` answers it anyway. The
    # rot this catches is re-gating the fold tier on `FOLD_STRUCTS`, which
    # dropped these rows to the global tier.
    expect(('NumberField', 'Render Props') not in api_audit.FOLD_STRUCTS
           and api_audit.ALIAS.get('NumberField.Render Props.isDisabled')
           == 'content',
           'the unscoped NumberField render-prop alias fixture is missing')
    expect(resolve_alias('NumberField', 'isDisabled', {},
                         {'Render Props': {'isDisabled'}}) == 'content',
           'an unscoped fold ignored its Comp.Heading.prop alias')
    real_number = api_audit.props_for_state('NumberField')
    expect(real_number is not None and
           resolve_alias('NumberField', 'isDisabled', real_number[1],
                         real_number[2]) == 'content',
           'the real NumberField fold tables did not resolve isDisabled to '
           'content')
    expect(ports('NumberField', 'isDisabled', {}, {'Render Props': {'isDisabled'}},
                 {'content'}), 'the resolved unscoped fold alias did not port')

    # Component-scoped and global behavior is unchanged.
    expect(resolve_alias('Toast', 'isLoading', {}, {}) == 'is_loading',
           'component-scoped Toast.isLoading needs no part context')
    expect(resolve_alias('Button', 'isPending', {}, {}) == 'is_pending',
           'global isPending did not resolve')
    expect(resolve_alias('Button', 'isDisabled', {}, {}) == 'is_disabled',
           'global isDisabled did not resolve to the snake spelling')

    # Known negatives: the narrowed global stays deleted, and a wrong part
    # must not reach the part-scoped alias -- the raw spelling falls through
    # and, matching nothing implemented, stays unexercised.
    expect('isLoading' not in api_audit.ALIAS,
           'the global isLoading alias is back')
    expect(resolve_alias('Button', 'isLoading', {}, {}) == 'isLoading',
           'a bare isLoading row resolved although the global alias is deleted')
    expect(not ports('Table', 'isLoading', {'Body': {'isLoading'}}, {},
                     {'is_pending'}),
           'a wrong part still reached the Table.LoadMore.isLoading alias')

    if failures:
        print('self-test FAIL')
        for failure in failures:
            print('- %s' % failure)
        return 1
    print('self-test PASS: part-scoped Table.LoadMore.isLoading resolves through '
          'its heading; the unscoped NumberField render-prop fold resolves '
          'through its alias before FOLD_STRUCTS gating; component and global '
          'aliases and the deleted global isLoading behave unchanged')
    return 0


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
        state = api_audit.props_for_state(component)
        if state is None:
            continue
        root_doc, parts, folds = state
        documented = set(root_doc)
        for chunk in parts.values():
            documented |= chunk
        for chunk in folds.values():
            documented |= chunk
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
            ported = resolve_alias(component, prop, parts, folds)
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
    if '--self-test' in sys.argv[1:]:
        sys.exit(self_test())
    main()
