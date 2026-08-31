"""One-shot helper: remove a v2 `size` prop from a component module.

v3 keeps `size` on nineteen components (Button, Chip, Switch, Modal, ...) and
removed it everywhere else; the form fields have exactly one height. This
collapses `match self.size { Sm => .., Md => .., Lg => .. }` to the `Md` arm and
deletes the field, its initialiser and its builder, so the removal is uniform
rather than hand-edited twenty times.

Usage: python .shots/strip_size.py input.rs kbd.rs ...
"""
import io
import re
import sys

SRC = 'crates/herogpui-components/src/'

BUILDER = re.compile(
    r'\n(?:    ///[^\n]*\n)*    pub fn size\(\s*mut self,\s*\w+: Size\w*\s*\)'
    r' -> Self \{\n        self\.size = \w+(?:\.into\(\))?;\n        self\n    \}\n')

MATCH = re.compile(
    r'match self\.size \{\n'
    r'(?:\s*Size(?:Xl)?::\w+ => [^\n]*?,\n)*?'
    r'\s*Size(?:Xl)?::Md => (.*?),\n'
    r'(?:\s*Size(?:Xl)?::\w+ => [^\n]*?,\n)*?'
    r'\s*\};',
    re.S)


def strip(name):
    path = SRC + name
    src = io.open(path, encoding='utf-8').read()
    before = src
    src = BUILDER.sub('\n', src, count=1)
    src = re.sub(r'    size: Size\w*,\n', '', src, count=1)
    src = re.sub(r'            size: Size\w*::\w+,\n', '', src, count=1)
    src = MATCH.sub(lambda m: m.group(1).strip() + ';', src)
    if src == before:
        print('%-20s NO CHANGE' % name)
        return
    io.open(path, 'w', encoding='utf-8').write(src)
    left = src.count('self.size')
    print('%-20s stripped%s' % (name, '' if not left else
                                ' (%d `self.size` left by hand)' % left))


if __name__ == '__main__':
    for n in sys.argv[1:]:
        strip(n)
