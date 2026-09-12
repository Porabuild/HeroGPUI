"""A ThemeDocument key that is not a ThemeBuilder method is a drift.

Theme serialization is a sparse override document, not a dump of `Theme`.
Every JSON key except `base` (which picks `Theme::light` or `Theme::dark`)
must map onto one `ThemeBuilder` method, because that is how derived hover /
soft mixes stay live. A builder method added without a document key — or a
document key with no builder — is a hole in that contract.

Both sides are read mechanically:

- builder: `pub fn name(` in `impl ThemeBuilder` in
  `crates/herogpui-theme/src/theme.rs`
- document: struct fields of `ThemeDocument` in
  `crates/herogpui-theme/src/theme_document.rs`

`role` on the builder is the map `roles` in JSON. `build` is not a key.
`base` is not a builder method.
"""
import os
import re
import sys

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUILDER = os.path.join(ROOT, 'crates/herogpui-theme/src/theme.rs')
DOCUMENT = os.path.join(ROOT, 'crates/herogpui-theme/src/theme_document.rs')
THEME_TOML = os.path.join(ROOT, 'crates/herogpui-theme/Cargo.toml')
FACADE_TOML = os.path.join(ROOT, 'crates/herogpui/Cargo.toml')

# Builder method -> JSON key, where the names differ.
RENAME = {
    'role': 'roles',
}

# JSON keys that are not builder methods.
DOCUMENT_ONLY = {'base'}

# Builder methods that are not JSON keys.
# `components` is a typed Rust subtree (`ComponentThemes` + recipes), not a
# CSS-variable-style sparse JSON token.
BUILDER_ONLY = {'build', 'components'}

PUB_FN = re.compile(r'pub fn ([A-Za-z_][A-Za-z0-9_]*)\s*\(')
# A field of ThemeDocument: `pub name:` at struct indent, skipping serde attrs.
FIELD = re.compile(r'^\s+pub ([A-Za-z_][A-Za-z0-9_]*)\s*:', re.M)
IMPL_BUILDER = re.compile(r'impl ThemeBuilder \{')
STRUCT_DOCUMENT = re.compile(r'pub struct ThemeDocument \{')


def builder_methods(source):
    start = IMPL_BUILDER.search(source)
    if start is None:
        return set()
    body = source[start.end():]
    # The impl ends at the next top-level `impl ` or module tests.
    end = re.search(r'\nimpl |\n#\[cfg\(test\)\]', body)
    if end:
        body = body[:end.start()]
    return set(PUB_FN.findall(body))


def document_fields(source):
    start = STRUCT_DOCUMENT.search(source)
    if start is None:
        return set()
    body = source[start.end():]
    end = body.find('\n}')
    if end >= 0:
        body = body[:end]
    return set(FIELD.findall(body))


def audit(builder_src=None, document_src=None, theme_toml=None, facade_toml=None):
    failures = []
    builder_src = builder_src if builder_src is not None else open(BUILDER, encoding='utf-8').read()
    document_src = document_src if document_src is not None else open(DOCUMENT, encoding='utf-8').read()
    theme_toml = theme_toml if theme_toml is not None else open(THEME_TOML, encoding='utf-8').read()
    facade_toml = facade_toml if facade_toml is not None else open(FACADE_TOML, encoding='utf-8').read()

    methods = builder_methods(builder_src)
    fields = document_fields(document_src)
    if not methods:
        failures.append(('NO BUILDER', 'ThemeBuilder has no pub fn methods'))
    if not fields:
        failures.append(('NO DOCUMENT', 'ThemeDocument has no pub fields'))

    expected_keys = {RENAME.get(m, m) for m in methods - BUILDER_ONLY} | DOCUMENT_ONLY
    expected_methods = {k if k != 'roles' else 'role' for k in fields - DOCUMENT_ONLY} | BUILDER_ONLY

    for key in sorted(expected_keys - fields):
        failures.append(('MISSING KEY', f'ThemeBuilder has a method with no ThemeDocument field {key!r}'))
    for key in sorted(fields - expected_keys):
        failures.append(('EXTRA KEY', f'ThemeDocument field {key!r} is not a ThemeBuilder method'))
    for method in sorted(expected_methods - methods):
        failures.append(('MISSING METHOD', f'ThemeDocument implies ThemeBuilder::{method} which is gone'))

    if 'serde = ["dep:serde", "dep:serde_json"]' not in theme_toml and \
            'serde = ["dep:serde_json", "dep:serde"]' not in theme_toml:
        # Accept either order of the optional deps.
        if not re.search(r'^serde\s*=\s*\[', theme_toml, re.M) or 'dep:serde' not in theme_toml:
            failures.append(('NO FEATURE', 'herogpui-theme must gate the document on a non-default serde feature'))
    if re.search(r'^default\s*=\s*\[[^\]]*serde', theme_toml, re.M):
        failures.append(('DEFAULT FEATURE', 'serde must not be a default feature of herogpui-theme'))

    if '"herogpui-theme/serde"' not in facade_toml:
        failures.append(('NO FORWARD', 'herogpui must forward its serde feature to herogpui-theme/serde'))
    if re.search(r'^default\s*=\s*\[[^\]]*serde', facade_toml, re.M):
        failures.append(('DEFAULT FEATURE', 'serde must not be a default feature of herogpui'))

    return failures, methods, fields


def self_test():
    builder = open(BUILDER, encoding='utf-8').read()
    document = open(DOCUMENT, encoding='utf-8').read()
    theme_toml = open(THEME_TOML, encoding='utf-8').read()
    facade_toml = open(FACADE_TOML, encoding='utf-8').read()

    def expect(cond, message):
        if not cond:
            raise SystemExit('self-test FAIL: ' + message)

    # A builder method with no document field.
    injected = builder.replace(
        '    pub fn build(self) -> Theme {',
        '    pub fn invented(self) -> Self { self }\n    pub fn build(self) -> Theme {',
        1,
    )
    verdicts = [f[0] for f in audit(injected, document, theme_toml, facade_toml)[0]]
    expect('MISSING KEY' in verdicts,
           'a ThemeBuilder method with no ThemeDocument field must fail: %r' % verdicts)

    # A document field with no builder method.
    injected_doc = document.replace(
        '    pub field_border: Option<String>,',
        '    pub field_border: Option<String>,\n    pub invented: Option<String>,',
        1,
    )
    verdicts = [f[0] for f in audit(builder, injected_doc, theme_toml, facade_toml)[0]]
    expect('EXTRA KEY' in verdicts,
           'a ThemeDocument field with no ThemeBuilder method must fail: %r' % verdicts)

    # serde on by default.
    on_by_default = theme_toml.replace(
        '[features]\n',
        '[features]\ndefault = ["serde"]\n',
        1,
    )
    verdicts = [f[0] for f in audit(builder, document, on_by_default, facade_toml)[0]]
    expect('DEFAULT FEATURE' in verdicts,
           'serde as a default feature of herogpui-theme must fail: %r' % verdicts)

    print('self-test PASS: the scanner fails a ThemeBuilder method with no '
          'ThemeDocument field, a document key with no builder, and serde '
          'turned on by default.')


def main():
    self_test()
    failures, methods, fields = audit()
    print('builder methods read    : %d' % len(methods))
    print('document fields read    : %d' % len(fields))
    print('failures                : %d' % len(failures))
    for tag, message in failures:
        print('  %-16s %s' % (tag, message))
    if failures:
        sys.exit(1)


if __name__ == '__main__':
    main()
