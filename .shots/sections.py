"""Print a gallery page's section titles, for `drive.ps1 -Section`.

The deep link takes a section *name*, and hunting for the exact spelling by
screenshotting the page defeats the point. This reads the titles out of the page
function, which is the same list `example_audit.py` compares against v3.

    python .shots/sections.py            # every page
    python .shots/sections.py Table      # one page, by its nav title or fn name
"""
import io
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

PAGES = ('gallery/src/pages/components.rs', 'gallery/src/pages/docs.rs')


def page_sections():
    """`{page_fn_suffix: [title, ...]}`."""
    out = {}
    for path in PAGES:
        src = io.open(path, encoding='utf-8', errors='replace').read()
        parts = re.split(r'\n    pub fn page_([a-z0-9_]+)\(', src)
        for name, body in zip(parts[1::2], parts[2::2]):
            titles = re.findall(r'^                \("([^"]+)",', body, re.M)
            titles += re.findall(r'^ {16}\(\s*\n {20}"([^"]+)",', body, re.M)
            titles += re.findall(r'^ {12}vec!\[\(\s*\n {16}"([^"]+)",', body, re.M)
            out.setdefault(name, []).extend(titles)
    return out


def main():
    pages = page_sections()
    if len(sys.argv) > 1:
        want = re.sub(r'[^a-z0-9]', '', ' '.join(sys.argv[1:]).lower())
        for name, titles in sorted(pages.items()):
            if re.sub(r'[^a-z0-9]', '', name) == want:
                for t in titles:
                    print(t)
                return
        print('no page matching %r; try one of:' % ' '.join(sys.argv[1:]))
        print('  ' + ', '.join(sorted(pages)))
        return
    for name, titles in sorted(pages.items()):
        print('%-22s %s' % (name, ', '.join(titles)))


if __name__ == '__main__':
    main()
