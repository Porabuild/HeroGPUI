"""The pinned HeroUI documentation bundle every prop and prose audit reads.

The repository ports HeroUI v3.2.4. `heroui.com/react/llms-full.txt` serves the
`v3` *branch*, not that tag, so reading it live measures the port against
whatever upstream shipped most recently. Nine audits did exactly that: the day
HeroUI publishes v3.3, they would quietly start reporting gaps against a
contract this port never claimed -- or accept a newly documented prop as one it
already owed.

So the bundle is checked in, compressed, frozen at the release the port names.
`resolve()` hands back a path to the decompressed text and refuses to return
anything whose latest release is not the pinned one. `HEROUI_BUNDLE` still wins
for exploring a newer upstream by hand, and is verified the same way -- pass
`HEROUI_BUNDLE_UNPINNED=1` to look at a different release deliberately.

Refreshing the pin is a deliberate act, not a side effect of running an audit:

    curl -sL https://heroui.com/react/llms-full.txt | gzip -9 > .shots/heroui-bundle.txt.gz
    # then update PINNED_RELEASE below and re-run every audit
"""

import gzip
import io
import os
import re
import shutil
import sys
import tarfile

PINNED_RELEASE = 'v3.2.4'

_HERE = os.path.dirname(os.path.abspath(__file__))
ARCHIVE = os.path.join(_HERE, 'heroui-bundle.txt.gz')


def _fail(message):
    """Audit-reader integrity: a bundle we cannot vouch for is not a zero-gap run."""
    sys.stderr.write('bundle: %s\n' % message)
    raise SystemExit(2)


def latest_release(text):
    """The release the bundle's own index calls current, or None."""
    match = re.search(r'##\s*Latest Release\s*\n+\s*###\s*(v[\d.]+)', text)
    return match.group(1) if match else None


def _verify(path):
    with io.open(path, encoding='utf-8', errors='replace') as handle:
        # The marker sits in the releases index; reading the whole 5MB bundle
        # twice per audit is wasteful, and it is well inside the first slice.
        head = handle.read(4_000_000)
    found = latest_release(head)
    if found is None:
        _fail('%s has no "Latest Release" heading; it is not an llms-full bundle' % path)
    if found != PINNED_RELEASE and not os.environ.get('HEROUI_BUNDLE_UNPINNED'):
        _fail(
            'bundle is %s but this port pins %s.\n'
            '  Set HEROUI_BUNDLE_UNPINNED=1 to audit against another release on purpose,\n'
            '  or refresh .shots/heroui-bundle.txt.gz and PINNED_RELEASE together.'
            % (found, PINNED_RELEASE)
        )
    return path


def resolve():
    """Path to the pinned bundle text, decompressing the archive on first use."""
    override = os.environ.get('HEROUI_BUNDLE')
    if override:
        if not os.path.exists(override):
            _fail('HEROUI_BUNDLE points at %s, which does not exist' % override)
        return _verify(override)

    if not os.path.exists(ARCHIVE):
        _fail('%s is missing; the pinned bundle is part of the parity contract' % ARCHIVE)

    cache = os.path.join(
        os.environ.get('TEMP', '/tmp'), 'heroui-%s-full.txt' % PINNED_RELEASE
    )
    if not os.path.exists(cache) or os.path.getmtime(cache) < os.path.getmtime(ARCHIVE):
        with gzip.open(ARCHIVE, 'rb') as source:
            with io.open(cache + '.part', 'wb') as target:
                shutil.copyfileobj(source, target)
        os.replace(cache + '.part', cache)
    return _verify(cache)


def read():
    """The pinned bundle's full text."""
    return io.open(resolve(), encoding='utf-8', errors='replace').read()


# The v3.2.4 component stylesheets, vendored the same way and for the same
# reason: design_audit, anim_audit and anatomy_audit all read them, and an
# empty cache does not read as "no findings" in any of the three -- anim_audit
# reported 22 phantom motion mismatches against one.
CSS_ARCHIVE = os.path.join(_HERE, 'heroui-css-v3.2.4.tar.gz')
CSS_CACHE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')


def css_cache():
    """Restore the vendored stylesheet cache. Returns whether it now exists."""
    if os.path.isdir(CSS_CACHE):
        return True
    if not os.path.exists(CSS_ARCHIVE):
        return False
    os.makedirs(CSS_CACHE, exist_ok=True)
    with tarfile.open(CSS_ARCHIVE, 'r:gz') as archive:
        archive.extractall(CSS_CACHE)
    return True
