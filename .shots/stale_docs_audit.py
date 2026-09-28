"""Fail when a known-stale fact reappears in the current CI and agent guidance.

Each phrase below was once true, stopped being true, and then survived in a
comment or guide long enough to mislead a contributor or an agent: a test
binary count that tripled, "Git GPUI" after the move to the registry, a
"stable" wasm32 build that has needed nightly since the web-platform fork was
retired, the PowerShell-only lint gate. The fix for each was made once; this
keeps it made.

It reads only files that state how the repository works *now* (see `SCANNED`).
Dated history -- CHANGELOG.md, docs/parity/, docs/upstream/, the roadmap --
quotes these phrases on purpose and is not scanned.

    python3 .shots/stale_docs_audit.py              # the report
    python3 .shots/stale_docs_audit.py --self-test  # prove the guard fires

To retire a new stale fact: fix every occurrence, then add a `STALE` entry
with the reason a reader should see.
"""
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parent.parent

# Globs relative to the repository root. Guidance, workflow and config files
# only; a glob that matches nothing is reported as a reader error.
SCANNED = (
    'AGENTS.md', 'CONTRIBUTING.md', 'RELEASING.md', 'README.md',
    'Cargo.toml', 'clippy.toml', 'rust-toolchain.toml', 'deny.toml',
    '.cargo/config.toml',
    '.github/workflows/*.yml', '.github/actions/*/action.yml',
    'docs/agents/*.md',
    '.shots/AGENTS.md', '.shots/*.sh', '.shots/*.ps1',
    'crates/*/AGENTS.md', 'gallery/AGENTS.md',
    'web/AGENTS.md', 'web/DEPLOYMENT.md',
    'skills/*/SKILL.md',
)

# (pattern, reason, files exempt from it). Case-insensitive.
STALE = (
    (r'(?<![\w"])~?\d+\+?\s+test binaries\b(?!\s+announced)',
     'a count of test binaries rots with every new tests/*.rs file; describe '
     'them without a number',
     ()),
    (r'not registry-publishable',
     'GPUI comes from the published gpui-pre crates and every library crate is '
     'on crates.io',
     ()),
    (r'\bgit GPUI\b',
     'GPUI is a crates.io dependency; package_audit.py rejects a git source',
     ()),
    (r'\bstable\s+wasm(?:32)?\b',
     'the wasm32 build needs nightly (gpui-pre-web enables `multithreaded`, '
     'which pulls in the #![feature] crate wasm_thread)',
     ()),
    (r'back to a nightly pin',
     'the wasm32 build is already on nightly; only a forked web platform could '
     'take it back to stable',
     ()),
    (r'gpui_patches\.py',
     'the patch materializer is retired; there is no setup step',
     ()),
    # A line naming lint.ps1 *as* the wrapper also names lint.sh; one naming
    # it alone presents it as the gate.
    (r'^(?!.*lint\.sh).*\.shots/lint\.ps1',
     'the lint gate is `.shots/lint.sh` (bash, runs everywhere); lint.ps1 is '
     'only its Windows wrapper',
     ('.shots/lint.ps1', '.shots/lint.sh')),
    (r'does not publish to crates\.io',
     "release.yml's publish-crates job publishes over trusted publishing",
     ()),
    (r'unpublished pre-1\.0|pre-1\.0 and unpublished',
     'the library crates are published on crates.io',
     ()),
)


def scan(files, stale=STALE, root=ROOT):
    """Every (path, line number, reason, line) where a stale phrase matches."""
    compiled = [(re.compile(pattern, re.I), reason, exempt) for pattern, reason, exempt in stale]
    hits = []
    for path in files:
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding='utf-8', errors='replace')
        for number, line in enumerate(text.splitlines(), 1):
            for pattern, reason, exempt in compiled:
                if rel not in exempt and pattern.search(line):
                    hits.append((rel, number, reason, line.strip()))
    return hits


def scanned_files(root=ROOT, globs=SCANNED):
    files, empty = set(), []
    for pattern in globs:
        matched = [p for p in root.glob(pattern) if p.is_file()]
        if not matched:
            empty.append(pattern)
        files.update(matched)
    return sorted(files), empty


def self_test():
    import tempfile

    failures = []
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / '.shots').mkdir()
        bad = root / 'AGENTS.md'
        bad.write_text(
            'all 70 test binaries finish in five seconds\n'
            'its 110+ test binaries\n'
            '# Git GPUI is not registry-publishable\n'
            'and a stable wasm32 build\n'
            'run .shots/lint.ps1 first\n',
            encoding='utf-8')
        good = root / 'CONTRIBUTING.md'
        good.write_text(
            'every test binary finishes in seconds\n'
            'reported "24 test binaries announced" against a full-suite 68\n'
            'the wasm32 build runs on nightly\n'
            'run bash .shots/lint.sh first\n'
            '`.shots/lint.ps1` forwards to `.shots/lint.sh`\n',
            encoding='utf-8')
        wrapper = root / '.shots' / 'lint.ps1'
        wrapper.write_text('#   .shots/lint.ps1 -Fix\n', encoding='utf-8')

        hits = scan([bad, good, wrapper], root=root)
        by_file = {}
        for rel, number, _reason, _line in hits:
            by_file.setdefault(rel, []).append(number)
        # Line 3 carries two stale phrases, so it is reported twice.
        if by_file.get('AGENTS.md') != [1, 2, 3, 3, 4, 5]:
            failures.append('known-stale lines not all reported: %r' % by_file.get('AGENTS.md'))
        if 'CONTRIBUTING.md' in by_file:
            failures.append('current wording flagged: %r' % by_file['CONTRIBUTING.md'])
        if '.shots/lint.ps1' in by_file:
            failures.append('an exempt file was flagged')

        _files, empty = scanned_files(root=root, globs=('docs/agents/*.md',))
        if empty != ['docs/agents/*.md']:
            failures.append('a glob matching nothing was not reported')

    for failure in failures:
        print('! self-test: ' + failure)
    if failures:
        print('self-test FAIL')
        return 1
    print('self-test PASS: every known-stale shape is reported, current wording '
          'and exempt files are not, and an empty glob is a reader error')
    return 0


def main():
    if self_test() != 0:
        return 1
    files, empty = scanned_files()
    for pattern in empty:
        print('AUDIT READER ERROR: no file matches %s' % pattern)
    hits = scan(files)
    print('files scanned        : %d' % len(files))
    print('phrases guarded      : %d' % len(STALE))
    print('STALE PHRASES        : %d' % len(hits))
    for rel, number, reason, line in hits:
        print('- %s:%d: %s' % (rel, number, line))
        print('    %s' % reason)
    return 1 if hits or empty else 0


if __name__ == '__main__':
    if '--self-test' in sys.argv[1:]:
        sys.exit(self_test())
    sys.exit(main())
