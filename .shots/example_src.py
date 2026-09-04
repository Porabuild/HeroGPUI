"""Print v3's worked examples for one component, so a port can be written from
the source rather than from the example's name.

    python .shots/example_src.py Button
    python .shots/example_src.py Button "Social Buttons"

With no example name it prints every one on the page. `example_audit.py` says
*which* examples are missing; this says what they contain.
"""
import io
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bundle import resolve as _resolve_bundle

# The pinned v3.2.4 bundle. See .shots/bundle.py: reading upstream live would
# measure this port against whatever HeroUI shipped most recently.
BUNDLE = _resolve_bundle()


def page_body(text, name):
    i = text.find('\n# ' + name + '\n')
    if i < 0:
        return None
    j = text.find('\n# ', i + 3)
    return text[i:j if j > 0 else len(text)]


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    name = sys.argv[1]
    want = sys.argv[2] if len(sys.argv) > 2 else None

    text = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
    body = page_body(text, name)
    if body is None:
        print('no page named %r in the bundle' % name)
        sys.exit(1)

    # `## Usage` holds the basic example; `## Examples` holds the rest.
    blocks = []
    m = re.search(r'^## Usage\s*$(.*?)(?=^## )', body, re.S | re.M)
    if m:
        blocks.append(('Usage', m.group(1)))
    m = re.search(r'^## Examples\s*$(.*?)(?=^## )', body, re.S | re.M)
    if m:
        for em in re.finditer(r'^### (.+?)\s*$(.*?)(?=^### |\Z)', m.group(1), re.S | re.M):
            blocks.append((em.group(1), em.group(2)))

    for title, chunk in blocks:
        if want and title.lower() != want.lower():
            continue
        print('=' * 70)
        print('###', title)
        print(chunk.strip())
        print()


if __name__ == '__main__':
    main()
