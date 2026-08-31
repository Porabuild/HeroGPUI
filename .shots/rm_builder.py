"""Remove a builder, its field and its initialiser from one component struct.

Used to strip HeroUI v2 props that `extra_audit.py` reports as undocumented in
v3. Prints the `self.<field>` sites left behind so the render body is fixed by
hand rather than silently.

Usage: python .shots/rm_builder.py card.rs:Card:is_pressable,on_press ...
"""
import io
import re
import sys

SRC = 'crates/herogpui-components/src/'


def impl_span(src, struct):
    """Byte range of `impl <struct> {` ... matching close brace."""
    m = re.search(r'^impl %s \{$' % re.escape(struct), src, re.M)
    if not m:
        raise SystemExit('no `impl %s {` block' % struct)
    depth, i = 0, m.end() - 1
    while i < len(src):
        if src[i] == '{':
            depth += 1
        elif src[i] == '}':
            depth -= 1
            if depth == 0:
                return m.start(), i + 1
        i += 1
    raise SystemExit('unbalanced braces in impl %s' % struct)


def struct_span(src, struct):
    m = re.search(r'^pub struct %s \{$' % re.escape(struct), src, re.M)
    if not m:
        raise SystemExit('no `pub struct %s {`' % struct)
    end = src.index('\n}\n', m.end())
    return m.start(), end


def rm(name, struct, fields):
    path = SRC + name
    src = io.open(path, encoding='utf-8').read()
    for f in fields:
        # builder, with any doc comment above it, inside this struct's impl
        lo, hi = impl_span(src, struct)
        block = src[lo:hi]
        pat = re.compile(
            r'\n(?:    ///[^\n]*\n)*    pub fn %s\(\s*mut self[^\n]*\n'
            r'(?:        [^\n]*\n)*?    \}\n' % re.escape(f))
        new, n = pat.subn('\n', block, count=1)
        if not n:
            print('  %-22s BUILDER NOT FOUND' % f)
        src = src[:lo] + new + src[hi:]
        # field declaration, with its doc comment
        lo, hi = struct_span(src, struct)
        decl = re.compile(
            r'\n(?:    ///[^\n]*\n)*    %s: [^\n]*,' % re.escape(f))
        new, n = decl.subn('', src[lo:hi], count=1)
        if not n:
            print('  %-22s FIELD NOT FOUND' % f)
        src = src[:lo] + new + src[hi:]
        # initialiser inside this impl's `Self { .. }`
        lo, hi = impl_span(src, struct)
        init = re.compile(r'\n            %s: [^\n]*,' % re.escape(f))
        new, n = init.subn('', src[lo:hi], count=1)
        if not n:
            print('  %-22s INIT NOT FOUND' % f)
        src = src[:lo] + new + src[hi:]
    io.open(path, 'w', encoding='utf-8').write(src)
    for f in fields:
        left = len(re.findall(r'self\s*\.\s*%s\b' % re.escape(f), src))
        print('%-18s %-22s %s' % (name, f,
                                  'clean' if not left else
                                  '%d use(s) left by hand' % left))


if __name__ == '__main__':
    for arg in sys.argv[1:]:
        name, struct, fields = arg.split(':')
        rm(name, struct, fields.split(','))
