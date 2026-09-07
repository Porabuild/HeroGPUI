"""Build the small offline font set `crates/herogpui-web` embeds in the wasm gallery.

The web platform pinned by this port has no system fonts, so `herogpui-web`
registers its typefaces with `include_bytes!` -- every byte of every TTF lands
in `web/public/gallery/herogpui_web_bg.wasm`. Shipped whole, the seven OFL
faces are ~13.0 MiB of a ~29.1 MiB artifact, and `NotoSansSC-Regular.ttf`
alone is 10.1 MiB of CJK the gallery never draws: the pages need the handful of
ideographs their own labels spell, not the whole standard.

So every source face stays in the repository as `*.source.ttf` and this script
emits the subsets under the names `crates/herogpui-web/src/lib.rs` already
names. The `include_bytes!` paths do not change -- the Rust source is not
touched by adopting this -- which also means a stale subset is invisible to the
compiler. That is what `--check` is for.

`NotoSansSC-Regular.source.ttf` is the one exception on the source side: it is
not the 10.1 MiB upstream face but its ~1.3 MiB pre-reduction, produced with
these same pyftsubset flags over the scrape set plus headroom -- GB2312 Level I
(the 3755 most common hanzi: every two-byte pair of the gb2312 codec's B0-D7
rows) and the kana syllabaries the existing Japanese labels already spell
from. A subset of that closure re-subsets to byte-identical output, so the
pre-reduction changed no embedded bytes. A literal that outgrows the headroom
fails `--check` as an unjustified glyph "in no source face"; grow it back by
re-running the pre-reduction over the full NotoSansSC-Regular face (Google's
noto-fonts release) with that same headroom text plus whatever tripped the
check, then run this script as usual.

The glyph set is discovered, never guessed. It is the union of:

  * every character in every `*.rs` file under the four source roots below,
    which is a deliberate superset of their string literals -- scraping literals
    needs a Rust parser, and being wrong there silently drops a glyph;
  * every character named by a `\\u{..}`/`\\x..` escape in those files, which
    whole-file text alone would miss (the file holds `\\`, `u`, `{`, not the
    ideograph);
  * every character in `llms.txt`, whose documented examples are the same
    strings the gallery pages render;
  * printable ASCII, so an unexercised label can never lose its alphabet; and
  * a fixed set of UI punctuation, arrows and symbols.

Determinism: `pyftsubset` copies `head.modified` from the source and defaults
to `--no-recalc-timestamp`, which this passes explicitly; `SOURCE_DATE_EPOCH`
is pinned in the child environment as well. Output is therefore byte-identical
across runs, so `--check` can compare bytes. Because different FontTools
releases do not agree byte for byte, the version is pinned in `FONTTOOLS`;
a locally installed FontTools is used only when it matches that pin.

What a developer needs installed: nothing, if `uv`/`uvx` is on PATH -- FontTools
is fetched into uvx's own cache at the pinned version. Otherwise install that
exact version (`pip install 'fonttools[woff]==4.63.0'`). `HEROGPUI_PYFTSUBSET`
overrides the command outright for an unusual environment.

Usage:
    python .shots/subset-fonts.py            # rebuild the committed subsets
    python .shots/subset-fonts.py --check    # fail if the committed subsets are stale
    python .shots/subset-fonts.py --coverage # glyph cross-check only, no writes

Exit codes: 0 clean, 1 a stale subset or an uncovered glyph, 2 unusable input
or tooling (an audit reader that cannot read its input is not a passing run).
"""

import filecmp
import io
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

_HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(_HERE)
FONTS = os.path.join(ROOT, 'crates', 'herogpui-web', 'fonts')

# Pinned so `--check` compares against the same compiler that produced the
# committed bytes. Bump this and the subsets in one commit.
FONTTOOLS = '4.63.0'

# The wasm payload is the gallery, so the gallery's own sources are as much a
# glyph source as the components they demonstrate.
SOURCE_ROOTS = (
    os.path.join('crates', 'herogpui-components', 'src'),
    os.path.join('crates', 'herogpui-core', 'src'),
    os.path.join('crates', 'herogpui-web', 'src'),
    os.path.join('gallery', 'src'),
)

# Read as text, never rewritten: `llms.txt` documents the examples the gallery
# renders, so a character it spells is a character the artifact must draw.
SOURCE_FILES = ('llms.txt',)

# Dashes, quotes, bullets, arrows, check/cross, warning and star marks used by
# empty, error and selection states, plus the space variants text layout picks.
UI_SYMBOLS = (
    '\u00a0\u00ab\u00bb\u00b7\u00b0\u00b1\u00d7\u00f7'
    '\u2010\u2011\u2013\u2014\u2018\u2019\u201a\u201c\u201d\u201e'
    '\u2020\u2022\u2026\u2030\u2039\u203a\u2044\u2060'
    '\u2190\u2191\u2192\u2193\u2194\u2195\u21b5\u21e5'
    '\u2212\u2215\u2248\u2260\u2264\u2265'
    '\u25b2\u25b6\u25bc\u25c0\u25cf\u25cb'
    '\u2605\u2606\u2611\u2713\u2714\u2715\u2717'
    '\u26a0\u2764\u263a\ufe0f\u2318\u2325\u21e7\u2303\u23ce\u232b'
)

# Characters the sources spell that no bundled face draws, with why that is
# accepted rather than fixed. `coverage()` separates these from a newly
# unavailable character so the second one cannot hide behind the first.
KNOWN_UNAVAILABLE = {
    '\u0635': 'Arabic AM marker, asserted by a time_field unit test only; '
    'the gallery renders no Arabic and bundles no Arabic face',
    '\u0645': 'Arabic PM marker, same test; see above',
}

# (source face, embedded name). The embedded names are exactly the paths
# `crates/herogpui-web/src/lib.rs` passes to `include_bytes!`.
FACES = (
    ('Inter-Regular.source.ttf', 'Inter-Regular.ttf'),
    ('Inter-Medium.source.ttf', 'Inter-Medium.ttf'),
    ('Inter-Semibold.source.ttf', 'Inter-Semibold.ttf'),
    ('Inter-Bold.source.ttf', 'Inter-Bold.ttf'),
    ('JetBrainsMono-Regular.source.ttf', 'JetBrainsMono-Regular.ttf'),
    ('NotoSansSC-Regular.source.ttf', 'NotoSansSC-Regular.ttf'),
    ('NotoEmoji-Regular.source.ttf', 'NotoEmoji-Regular.ttf'),
)

ESCAPES = re.compile(r'\\u\{([0-9A-Fa-f]{1,6})\}|\\x([0-9A-Fa-f]{2})')


def _fail(message):
    """Tool or input we cannot vouch for is not a clean run."""
    sys.stderr.write('subset-fonts: %s\n' % message)
    raise SystemExit(2)


def _read(path):
    return io.open(path, encoding='utf-8', errors='replace').read()


def scraped_text():
    """Every source file the glyph set is derived from, concatenated."""
    parts = []
    for relative in SOURCE_ROOTS:
        root = os.path.join(ROOT, relative)
        if not os.path.isdir(root):
            _fail('%s is missing; the glyph set is derived from it' % relative)
        found = []
        for base, _dirs, names in os.walk(root):
            found += [os.path.join(base, n) for n in names if n.endswith('.rs')]
        if not found:
            _fail('%s holds no .rs files; refusing to subset from nothing' % relative)
        for path in sorted(found):
            parts.append(_read(path))
    for relative in SOURCE_FILES:
        path = os.path.join(ROOT, relative)
        if not os.path.exists(path):
            _fail('%s is missing; the glyph set is derived from it' % relative)
        parts.append(_read(path))
    return ''.join(parts)


def glyph_set():
    """The characters the artifact must be able to draw, sorted."""
    text = scraped_text()
    chars = set(text)
    for hexes, byte in ESCAPES.findall(text):
        code = int(hexes or byte, 16)
        if code and code <= 0x10FFFF and not 0xD800 <= code <= 0xDFFF:
            chars.add(chr(code))
    chars.update(chr(c) for c in range(0x20, 0x7F))
    chars.update(UI_SYMBOLS)
    # Control characters and the escapes' own surrogates are not glyphs.
    chars = {c for c in chars if c.isprintable() or c == ' '}
    return ''.join(sorted(chars))


def _subset_command():
    """The pyftsubset invocation to use, and how it was resolved."""
    override = os.environ.get('HEROGPUI_PYFTSUBSET')
    if override:
        return shlex.split(override), 'HEROGPUI_PYFTSUBSET'
    try:
        import fontTools
    except ImportError:
        fontTools = None
    if fontTools is not None and fontTools.version == FONTTOOLS:
        return [sys.executable, '-m', 'fontTools.subset'], 'fontTools %s (local)' % FONTTOOLS
    if shutil.which('uvx'):
        return (
            ['uvx', '--from', 'fonttools[woff]==%s' % FONTTOOLS, 'pyftsubset'],
            'uvx fonttools==%s' % FONTTOOLS,
        )
    have = 'not installed' if fontTools is None else fontTools.version
    _fail(
        'need FontTools %s; local is %s and uvx is not on PATH.\n'
        "  Install it (pip install 'fonttools[woff]==%s'), install uv, or set\n"
        '  HEROGPUI_PYFTSUBSET to the pyftsubset command to use.'
        % (FONTTOOLS, have, FONTTOOLS)
    )


def _child_env():
    env = dict(os.environ)
    # Belt and braces beside --no-recalc-timestamp: FontTools honours this when
    # it does stamp a table, so neither path can leak a build time.
    env['SOURCE_DATE_EPOCH'] = '0'
    env['PYTHONHASHSEED'] = '0'
    return env


def subset(command, source, output, text_file):
    result = subprocess.run(
        command
        + [
            source,
            '--text-file=%s' % text_file,
            '--output-file=%s' % output,
            '--layout-features=*',
            '--glyph-names',
            '--symbol-cmap',
            '--legacy-cmap',
            '--notdef-glyph',
            '--notdef-outline',
            '--recommended-glyphs',
            '--name-IDs=*',
            '--name-legacy',
            '--name-languages=*',
            '--no-recalc-timestamp',
        ],
        env=_child_env(),
        capture_output=True,
        text=True,
    )
    if result.returncode:
        sys.stderr.write(result.stdout + result.stderr)
        _fail('pyftsubset failed on %s' % os.path.basename(source))


_CMAP_PROBE = """
import json, sys
from fontTools.ttLib import TTFont
out = {}
for path in sys.argv[1:]:
    font = TTFont(path, lazy=True, fontNumber=0)
    codes = set()
    for table in font["cmap"].tables:
        codes.update(table.cmap.keys())
    out[path] = sorted(codes)
print(json.dumps(out))
"""


def cmaps(paths):
    """Mapping of font path -> set of characters its cmap covers."""
    paths = [p for p in paths if os.path.exists(p)]
    if not paths:
        return {}
    try:
        import fontTools  # noqa: F401

        runner = [sys.executable, '-c', _CMAP_PROBE]
    except ImportError:
        if not shutil.which('uv'):
            _fail('reading a font cmap needs FontTools or uv on PATH')
        runner = [
            'uv',
            'run',
            '--no-project',
            '--with',
            'fonttools==%s' % FONTTOOLS,
            'python',
            '-c',
            _CMAP_PROBE,
        ]
    result = subprocess.run(runner + paths, capture_output=True, text=True)
    if result.returncode:
        sys.stderr.write(result.stdout + result.stderr)
        _fail('could not read font cmaps')
    raw = json.loads(result.stdout.strip().splitlines()[-1])
    return {path: {chr(c) for c in codes} for path, codes in raw.items()}


def _name(char):
    return 'U+%04X %s' % (ord(char), char if char.isprintable() else '')


def coverage(wanted, subset_dir):
    """Report scraped characters no emitted subset can draw.

    A character missing from one face is normal -- Inter carries no ideographs,
    which is why NotoSansSC is bundled at all. It is a gap only when no subset
    covers it while some full source face does. A character no source face has
    is reported separately: no amount of subsetting can add it.
    """
    subsets = cmaps([os.path.join(subset_dir, out) for _src, out in FACES])
    sources = cmaps([os.path.join(FONTS, src) for src, _out in FACES])
    covered = set().union(*subsets.values()) if subsets else set()
    available = set().union(*sources.values()) if sources else set()

    dropped = sorted(c for c in wanted if c not in covered and c in available)
    absent = sorted(c for c in wanted if c not in covered and c not in available)

    print()
    print('glyph coverage')
    print('  scraped                 %6d' % len(wanted))
    print('  covered by the subsets  %6d' % len(wanted - set(dropped) - set(absent)))
    print('  dropped by subsetting   %6d' % len(dropped))
    print('  in no source face       %6d' % len(absent))
    if dropped:
        print('  DROPPED: %s' % ', '.join(_name(c) for c in dropped[:40]))
    for char in absent:
        why = KNOWN_UNAVAILABLE.get(char)
        print(
            '  %s no bundled face -- %s'
            % (_name(char), why if why else 'NOT JUSTIFIED; add a face or a KNOWN_UNAVAILABLE entry')
        )
    # An unjustified absence fails the run. With a whole face behind every
    # `*.source.ttf` a new literal makes the committed subset stale, which
    # `--check` already catches; against NotoSansSC's pre-reduced source it
    # can only surface here, so this must be an error, not advice.
    return dropped, [c for c in absent if c not in KNOWN_UNAVAILABLE]


def build(destination, text):
    command, how = _subset_command()
    print('pyftsubset via %s' % how)
    rows = []
    with tempfile.TemporaryDirectory() as temp:
        text_file = os.path.join(temp, 'characters.txt')
        io.open(text_file, 'w', encoding='utf-8').write(text)
        for source_name, output_name in FACES:
            source = os.path.join(FONTS, source_name)
            if not os.path.exists(source):
                _fail(
                    '%s is missing. The source faces are kept in-repo so the\n'
                    '  subsetting is re-runnable; restore it before rebuilding.'
                    % os.path.join('crates/herogpui-web/fonts', source_name)
                )
            output = os.path.join(destination, output_name)
            subset(command, source, output, text_file)
            rows.append(
                (output_name, os.path.getsize(source), os.path.getsize(output))
            )
    return rows


def table(rows):
    print()
    print('%-32s %12s %12s %8s' % ('face', 'source', 'subset', 'kept'))
    for name, before, after in rows:
        print(
            '%-32s %11.1fK %11.1fK %7.1f%%'
            % (name, before / 1024.0, after / 1024.0, 100.0 * after / before)
        )
    before = sum(r[1] for r in rows)
    after = sum(r[2] for r in rows)
    print(
        '%-32s %11.1fK %11.1fK %7.1f%%'
        % ('total (%d faces)' % len(rows), before / 1024.0, after / 1024.0,
           100.0 * after / before)
    )
    print('saved %.1fK (%.2f MiB) of embedded font bytes'
          % ((before - after) / 1024.0, (before - after) / 1048576.0))


def main(argv):
    modes = [a for a in argv if a.startswith('-')]
    unknown = [a for a in modes if a not in ('--check', '--coverage')]
    if unknown or len(modes) != len(argv):
        _fail('usage: subset-fonts.py [--check | --coverage]')
    check = '--check' in modes
    only_coverage = '--coverage' in modes and not check

    text = glyph_set()
    print('glyph set: %d characters scraped from %d source roots plus %s'
          % (len(text), len(SOURCE_ROOTS), ', '.join(SOURCE_FILES)))

    destination = FONTS
    temp = None
    if check or only_coverage:
        temp = tempfile.mkdtemp(prefix='herogpui-fonts-')
        destination = temp
    try:
        rows = build(destination, text)
        table(rows)
        dropped, unjustified = coverage(set(text), destination)

        stale = []
        if check:
            for _source_name, output_name in FACES:
                committed = os.path.join(FONTS, output_name)
                if not os.path.exists(committed):
                    stale.append((output_name, 'not committed'))
                elif not filecmp.cmp(
                    committed, os.path.join(temp, output_name), shallow=False
                ):
                    stale.append((output_name, 'differs from a fresh subset'))
            print()
            if stale:
                print('STALE SUBSETS')
                for name, why in stale:
                    print('  %-32s %s' % (name, why))
                print()
                print('Run `python .shots/subset-fonts.py` and commit the result.')
            else:
                print('committed subsets match a fresh build of the scraped glyph set')
    finally:
        if temp:
            shutil.rmtree(temp, ignore_errors=True)

    if check and stale:
        return 1
    if dropped or unjustified:
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
