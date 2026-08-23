"""Diff v3's documented animations against `components/src/anim.rs`.

The prop audits say nothing about motion: a component can expose every prop and
still not move. This lists every animation v3's stylesheet defines — the
`animate-in`/`animate-out` pairs, the `@keyframes`, the timing tokens and the
reduced-motion rule — and checks that something in this port implements each.

`IMPLEMENTS` maps a v3 animation to the symbol that provides it, and
`WONT_ANIMATE` records the ones that cannot be reproduced, with the reason. Both
are checked against the source, so a mapping that names a symbol which no longer
exists fails rather than going stale.
"""
import io
import os
import re
import sys
import glob

sys.stdout.reconfigure(encoding='utf-8', errors='replace')

BUNDLE = os.environ.get(
    'HEROUI_BUNDLE',
    os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-full.txt'),
)
SRC = 'crates/herogpui-components/src/'
THEME = 'crates/herogpui-theme/src/'

# v3 animation -> the symbol implementing it. The symbol must exist in the
# source, or this entry is stale.
IMPLEMENTS = {
    'zoom-in-90': 'pub fn entering_zoom',
    'fade-in-0': 'pub fn entering',
    'animate-in': 'pub fn entering_zoom',
    'zoom-out-95': 'pub fn exiting',
    'fade-out': 'pub fn exiting',
    'animate-out': 'pub fn exiting',
    'slide-in-from-bottom-4': 'pub fn entering_from',
    'slide-out-to-bottom-2': 'pub fn exiting_to',
    'scale(0.97)': 'pub fn pressed',
    '@keyframes caret-blink': 'pub fn caret_blink',
    '@keyframes skeleton': 'SkeletonAnimation',
    'animate-pulse': 'SkeletonAnimation',
    'duration-200': 'pub const ENTERING_MS',
    'duration-150': 'pub const EXITING_MS',
    'motion-reduce': 'reduce_motion',
    '--tooltip-delay': 'tooltip_delay',
    '--tooltip-close-delay': 'tooltip_close_delay',
    '--skeleton-animation': 'skeleton_animation',
    'transition-colors': 'pub const TRANSITION_MS',
    'progress indeterminate': 'progress-indeterminate',
    'spinner': 'with_transformation',
}

# Animations that cannot be reproduced here, with the reason.
WONT_ANIMATE = {
    # A caller-supplied easing/duration per overlay. The tokens exist; the
    # per-instance override is a `className`, which gpui has no analogue for.
    'data-[entering]:duration-400': 'per-instance-classname',
    'data-[entering]:duration-500': 'per-instance-classname',
    'data-[entering]:ease-[cubic-bezier(0.16,1,0.3,1)]': 'per-instance-classname',
    'data-[entering]:ease-[cubic-bezier(0.25,1,0.5,1)]': 'per-instance-classname',
    'data-[exiting]:ease-[cubic-bezier(0.7,0,0.84,0)]': 'per-instance-classname',
    'data-[exiting]:ease-[cubic-bezier(0.5,0,0.75,0)]': 'per-instance-classname',
    # `transform: scale()` on a quad. Reproduced geometrically for the press and
    # the overlay zoom; a bare `transition-transform` on arbitrary content has
    # nothing to scale.
    'transition-transform': 'no-div-transform',
    'transition-[scale': 'no-div-transform',
    # v3's own docs list these as Framer Motion examples, not component styles.
    '@keyframes bounce': 'docs-example-only',
    'animate-fade-in': 'docs-example-only',
    # gpui animates a style per frame rather than interpolating a declared
    # property, so there is no "transition every property" mode.
    'transition-all': 'no-property-transitions',
    'transition-none': 'no-property-transitions',
    'transition-background': 'no-property-transitions',
    'transition-opacity': 'no-property-transitions',
    'transition-width': 'no-property-transitions',
    'transition-height': 'no-property-transitions',
    'transition-size': 'no-property-transitions',
    'transition-left': 'no-property-transitions',
    'transition-[box': 'no-property-transitions',
}


def source():
    text = []
    for path in glob.glob(SRC + '*.rs') + glob.glob(THEME + '*.rs'):
        text.append(io.open(path, encoding='utf-8').read())
    return '\n'.join(text)


def main():
    bundle = io.open(BUNDLE, encoding='utf-8', errors='replace').read()
    src = source()

    # Which of v3's animations actually appear in its docs, so a mapping for
    # something v3 no longer ships shows up as stale too.
    present = {}
    for name in list(IMPLEMENTS) + list(WONT_ANIMATE):
        needle = name
        if name == 'scale(0.97)':
            needle = 'scale(0.97)'
        elif name == 'progress indeterminate':
            needle = 'isIndeterminate'
        elif name == 'spinner':
            needle = 'Spinner'
        present[name] = needle in bundle

    stale_docs = [n for n, ok in present.items() if not ok]
    missing_impl = []
    for name, symbol in sorted(IMPLEMENTS.items()):
        if symbol not in src:
            missing_impl.append((name, symbol))

    implemented = len(IMPLEMENTS) - len(missing_impl)
    print('v3 animations implemented      : %d' % implemented)
    for name, symbol in sorted(IMPLEMENTS.items()):
        mark = ' ' if symbol in src else '!'
        print('  %s %-28s %s' % (mark, name, symbol))
    print()
    print('deliberately not animated      : %d' % len(WONT_ANIMATE))
    reasons = {}
    for name, reason in WONT_ANIMATE.items():
        reasons.setdefault(reason, []).append(name)
    for reason, names in sorted(reasons.items()):
        print('  %-26s %s' % (reason, ', '.join(sorted(names))))
    print()
    if missing_impl:
        print('BROKEN MAPPINGS (symbol not in source):')
        for name, symbol in missing_impl:
            print('  %-28s %s' % (name, symbol))
    if stale_docs:
        print('no longer in the v3 docs (stale entry?): %s'
              % ', '.join(sorted(stale_docs)))
    print('UNIMPLEMENTED : %d' % len(missing_impl))


if __name__ == '__main__':
    main()
