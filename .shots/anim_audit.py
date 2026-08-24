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
# The v3 stylesheets `design_audit.py --fetch` caches. Motion lives in the CSS,
# not the docs bundle, so the per-overlay checks read from here.
CACHE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')

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
    'scale(0.97)': 'PRESSED_SCALE',
    # v3 presses four different amounts: a button 0.97, a menu row 0.98, a
    # pagination arrow 0.96, a calendar cell and a radio control 0.95.
    'scale(0.98)': 'PRESSED_SCALE_SUBTLE',
    'scale(0.96)': 'PRESSED_SCALE_FIRM',
    'scale(0.95)': 'PRESSED_SCALE_DEEP',
    '@keyframes caret-blink': 'pub fn caret_blink',
    '@keyframes skeleton': 'SkeletonAnimation',
    'animate-pulse': 'SkeletonAnimation',
    'duration-150': 'pub const ENTERING_MS',
    'duration-100': 'pub const EXITING_MS',
    'duration-250': 'PANEL_IN',
    'ease-smooth': 'Curve::Smooth',
    'ease-out-quad': 'Curve::OutQuad',
    'ease-out-fluid': 'Curve::OutFluid',
    'zoom-in-105': 'PANEL_IN',
    'zoom-in-95': 'LIST_IN',
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


# v3 declares a duration, easing and zoom *per overlay* -- reading the guide
# instead of the stylesheets had suggested one global 200ms/zoom-in-90. Each
# entry maps a component's CSS to the `Motion` constant that must match it.
MOTIONS = {
    # (css file, which `@apply animate-*` block) -> Motion constant
    ('modal', 1): 'PANEL_IN', ('modal', 0): 'BACKDROP_IN',
    ('alert-dialog', 1): 'PANEL_IN', ('alert-dialog', 0): 'BACKDROP_IN',
    ('popover', 0): 'POPOVER_IN',
    ('dropdown', 0): 'POPOVER_IN',
    ('tooltip', 0): 'POPOVER_IN',
    ('select', 0): 'LIST_IN',
    ('combo-box', 0): 'LIST_IN',
    ('date-picker', 0): 'LIST_IN',
    ('color-picker', 0): 'LIST_IN',
    ('autocomplete', 0): 'FLUID_IN',
}
CURVES = {'out': 'Out', 'smooth': 'Smooth', 'out-quad': 'OutQuad',
          'out-fluid': 'OutFluid', 'linear': 'Linear'}


def declared_motions(css):
    """The `animate-in` blocks in one stylesheet, in file order."""
    out = []
    for m in re.finditer(r'@apply ([^;]*animate-in[^;]*);', css):
        toks = m.group(1).split()
        dur = next((int(t[9:]) for t in toks if t.startswith('duration-')), None)
        curve = next((CURVES.get(t[5:]) for t in toks if t.startswith('ease-')), None)
        zoom = next((int(t[8:]) / 100.0 for t in toks if t.startswith('zoom-in-')), 1.0)
        out.append({'ms': dur, 'curve': curve, 'scale': zoom})
    return out


def check_motions():
    """Each `Motion` constant against the CSS it transcribes."""
    src = io.open('crates/herogpui-components/src/anim.rs', encoding='utf-8').read()
    ours = {}
    # `cargo fix`/rustfmt may split a constant across lines, so match the
    # fields rather than the one-line form -- this regex silently found nothing
    # after a reformat, which read as twelve mismatches.
    for m in re.finditer(
            r'pub const (\w+): Motion = Motion \{\s*ms:\s*(\d+),'
            r'\s*scale:\s*([\d.]+),\s*curve:\s*Curve::(\w+),?\s*\}',
            src, re.S):
        ours[m.group(1)] = {'ms': int(m.group(2)), 'scale': float(m.group(3)),
                            'curve': m.group(4)}
    rows, bad = [], 0
    for (comp, index), name in sorted(MOTIONS.items()):
        path = os.path.join(CACHE, comp + '.css')
        if not os.path.exists(path):
            rows.append(('?', comp, name, 'no stylesheet', ''))
            bad += 1
            continue
        blocks = declared_motions(io.open(path, encoding='utf-8', errors='replace').read())
        if index >= len(blocks) or name not in ours:
            rows.append(('?', comp, name, 'not found', ''))
            bad += 1
            continue
        want, got = blocks[index], ours[name]
        same = (want['ms'] == got['ms']
                and abs(want['scale'] - got['scale']) < 1e-6
                and want['curve'] == got['curve'])
        if not same:
            bad += 1
        rows.append((' ' if same else '!', comp, name,
                     '%sms %s %g' % (want['ms'], want['curve'], want['scale']),
                     '%sms %s %g' % (got['ms'], got['curve'], got['scale'])))
    print('per-overlay motion (v3 CSS vs our Motion constants):')
    for mark, comp, name, want, got in rows:
        print('%s %-14s %-12s %-22s %s' % (mark, comp, name, want, got))
    print('MOTION MISMATCHES : %d' % bad)
    print()
    return bad


def check_switch_thumb():
    """The Switch thumb's property transition against its component CSS."""
    css_path = os.path.join(CACHE, 'switch.css')
    src_path = os.path.join(SRC, 'switch.rs')
    if not os.path.exists(css_path):
        print('switch thumb motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    want = re.search(r'margin\s+(\d+)ms\s+var\(--ease-([\w-]+)\)', css)
    got_ms = re.search(r'const THUMB_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_curve = re.search(
        r'THUMB_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    if want is None or got_ms is None or got_curve is None:
        print('switch thumb motion: unreadable')
        return 1
    want_curve = CURVES.get(want.group(2))
    same = int(want.group(1)) == int(got_ms.group(1)) and want_curve == got_curve.group(1)
    print('switch thumb motion (v3 CSS vs Switch):')
    print('%s %-14s %-12s %-22s %s' % (
        ' ' if same else '!',
        'switch',
        'thumb margin',
        '%sms %s' % (want.group(1), want_curve),
        '%sms %s' % (got_ms.group(1), got_curve.group(1)),
    ))
    print('SWITCH MISMATCHES : %d' % (0 if same else 1))
    print()
    return 0 if same else 1


def corpus():
    """Everything v3 ships that could name an animation.

    The docs bundle alone is not enough: `duration-250` and `zoom-in-105` appear
    only in the component stylesheets, so checking presence against the bundle
    reported them as stale when they are exactly what the modal declares.
    """
    text = [io.open(BUNDLE, encoding='utf-8', errors='replace').read()]
    for path in glob.glob(os.path.join(CACHE, '*.css')):
        text.append(io.open(path, encoding='utf-8', errors='replace').read())
    return '\n'.join(text)


def main():
    bundle = corpus()
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
    print()
    motion_bad = check_motions() + check_switch_thumb()
    print('UNIMPLEMENTED : %d' % len(missing_impl))
    print('MOTION BAD    : %d' % motion_bad)


if __name__ == '__main__':
    main()
