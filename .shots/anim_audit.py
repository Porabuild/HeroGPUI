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


def check_switch_motion():
    """The Switch property transitions against its component CSS."""
    css_path = os.path.join(CACHE, 'switch.css')
    src_path = os.path.join(SRC, 'switch.rs')
    if not os.path.exists(css_path):
        print('switch motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    control = re.search(
        r'\.switch__control\s*\{(.*?)(?=\n/\* Switch content)', css, re.S
    )
    thumb = re.search(
        r'/\* Switch thumb \*/\s*\.switch__thumb\s*\{(.*?)(?=\n/\* Size-specific thumb dimensions)',
        css,
        re.S,
    )
    want_track = re.search(
        r'background-color\s+(\d+)ms\s+var\(--ease-([\w-]+)\)',
        control.group(1) if control else '',
    )
    want_thumb = re.search(
        r'margin\s+(\d+)ms\s+var\(--ease-([\w-]+)\)',
        thumb.group(1) if thumb else '',
    )
    got_track_ms = re.search(r'const TRACK_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_track_curve = re.search(
        r'TRACK_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    got_thumb_ms = re.search(r'const THUMB_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_thumb_curve = re.search(
        r'THUMB_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    track_parts = src.split('fn track_motion(', 1)
    track_source = (
        track_parts[1].split('/// HeroUI Switch', 1)[0]
        if len(track_parts) == 2
        else ''
    )
    reduced = (
        control is not None
        and thumb is not None
        and control.group(1).find('transition:')
        < control.group(1).find('motion-reduce:transition-none')
        and thumb.group(1).find('transition:')
        < thumb.group(1).find('motion-reduce:transition-none')
        and 'let reduce_motion = cx.reduce_motion();' in track_source
        and 'if reduce_motion' in track_source
        and 'let animate = !reduce_motion' in track_source
    )
    child_fill = bool(
        re.search(
            r'track\s*=\s*track\s*\.child\(\s*track_motion_frame\.render\(\s*'
            r'gpui::div\(\)\s*'
            r'\.absolute\(\)\s*\.inset_0\(\)\s*\.rounded\(track_r\)\s*\)\s*\)',
            src,
            re.S,
        )
        and re.search(
            r'track\s*=\s*crate::util::track_interaction\(track,\s*&interaction\)',
            src,
        )
    )
    if (
        want_track is None
        or want_thumb is None
        or got_track_ms is None
        or got_track_curve is None
        or got_thumb_ms is None
        or got_thumb_curve is None
    ):
        print('switch motion: unreadable')
        return 1
    want_track_curve = CURVES.get(want_track.group(2))
    want_thumb_curve = CURVES.get(want_thumb.group(2))
    rows = [
        (
            int(want_track.group(1)) == int(got_track_ms.group(1))
            and want_track_curve == got_track_curve.group(1),
            'track background',
            '%sms %s' % (want_track.group(1), want_track_curve),
            '%sms %s' % (got_track_ms.group(1), got_track_curve.group(1)),
        ),
        (
            int(want_thumb.group(1)) == int(got_thumb_ms.group(1))
            and want_thumb_curve == got_thumb_curve.group(1),
            'thumb margin',
            '%sms %s' % (want_thumb.group(1), want_thumb_curve),
            '%sms %s' % (got_thumb_ms.group(1), got_thumb_curve.group(1)),
        ),
        (reduced, 'reduced motion', 'transition-none', 'direct fill' if reduced else 'missing'),
        (child_fill, 'animation owner', 'listener-free fill', 'child fill' if child_fill else 'track'),
    ]
    print('switch motion (v3 CSS vs Switch):')
    for same, name, want, got in rows:
        print('%s %-14s %-16s %-22s %s' % (' ' if same else '!', 'switch', name, want, got))
    print('SWITCH MISMATCHES : %d' % sum(not same for same, _, _, _ in rows))
    print()
    return sum(not same for same, _, _, _ in rows)


def check_toggle_button_motion():
    """ToggleButton's size scales and the group's transform suppression."""
    css_path = os.path.join(CACHE, 'toggle-button.css')
    group_css_path = os.path.join(CACHE, 'toggle-button-group.css')
    src_path = os.path.join(SRC, 'toggle_button.rs')
    anim_path = os.path.join(SRC, 'anim.rs')
    if not os.path.exists(css_path) or not os.path.exists(group_css_path):
        print('toggle button motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    group_css = io.open(group_css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    anim = io.open(anim_path, encoding='utf-8').read()

    def scale_for(selector, end):
        block = re.search(re.escape(selector) + r'\s*\{(.*?)(?=' + end + r')', css, re.S)
        scale = re.search(r'transform:\s*scale\(([\d.]+)\)', block.group(1) if block else '')
        return float(scale.group(1)) if scale else None

    wants = {
        'Sm': scale_for('.toggle-button--sm', r'\n\.toggle-button--md'),
        'Md': scale_for('.toggle-button', r'\n/\* ={10,}\n   Size variants'),
        'Lg': scale_for('.toggle-button--lg', r'\n/\* ={10,}\n   Variant styles'),
    }
    constants = {}
    for name in ('PRESSED_SCALE_SUBTLE', 'PRESSED_SCALE', 'PRESSED_SCALE_FIRM'):
        match = re.search(r'pub const %s:\s*f32\s*=\s*([\d.]+)' % name, anim)
        constants[name] = float(match.group(1)) if match else None
    names = {
        'Sm': 'PRESSED_SCALE_SUBTLE',
        'Md': 'PRESSED_SCALE',
        'Lg': 'PRESSED_SCALE_FIRM',
    }
    scale_wired = bool(re.search(r'scale:\s*press_scale', src))
    rows = []
    for size in ('Sm', 'Md', 'Lg'):
        symbol = names[size]
        mapped = bool(re.search(r'Size::%s\s*=>\s*\(.*?crate::anim::%s\)' % (size, symbol), src, re.S))
        same = wants[size] is not None and constants[symbol] == wants[size] and mapped and scale_wired
        rows.append((same, size, wants[size], constants[symbol] if mapped else None))

    css_suppresses = bool(re.search(
        r'\.toggle-button-group \.toggle-button(?:\[data-pressed="true"\]|:active).*?transform:\s*none',
        group_css,
        re.S,
    ))
    grouped_from_edge = 'let is_grouped = self.group_edge.is_some();' in src
    rust_suppresses = bool(re.search(
        r'if is_grouped\s*\{.*?\}\s*else\s*\{\s*el\s*=\s*crate::anim::pressed_with_background',
        src,
        re.S,
    ))
    rows.append((css_suppresses and grouped_from_edge and rust_suppresses, 'group', 'none', 'none' if rust_suppresses else 'scale'))

    print('toggle button motion (v3 CSS vs ToggleButton):')
    for same, size, want, got in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'toggle-button', size, str(want), str(got)
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('TOGGLE BUTTON MISMATCHES : %d' % bad)
    print()
    return bad


def check_pagination_motion():
    """Pagination's size-specific link scale, including its nav links."""
    css_path = os.path.join(CACHE, 'pagination.css')
    src_path = os.path.join(SRC, 'pagination.rs')
    anim_path = os.path.join(SRC, 'anim.rs')
    if not os.path.exists(css_path):
        print('pagination motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    anim = io.open(anim_path, encoding='utf-8').read()

    def scale_in(pattern):
        block = re.search(pattern, css, re.S)
        scale = re.search(r'transform:\s*scale\(([\d.]+)\)', block.group(1) if block else '')
        return float(scale.group(1)) if scale else None

    wants = {
        'Sm': scale_in(r'\.pagination--sm\s*\{(.*?)(?=\n\.pagination--md)'),
        'Md': scale_in(r'\.pagination__link\s*\{(.*?)(?=\n/\* Ellipsis)'),
        'Lg': scale_in(r'\.pagination--lg\s*\{(.*)\Z'),
    }
    names = {
        'Sm': 'PRESSED_SCALE_SUBTLE',
        'Md': 'PRESSED_SCALE',
        'Lg': 'PRESSED_SCALE_FIRM',
    }
    constants = {}
    for name in names.values():
        match = re.search(r'pub const %s:\s*f32\s*=\s*([\d.]+)' % name, anim)
        constants[name] = float(match.group(1)) if match else None

    scale_map = re.search(
        r'let press_scale = match self\.size \{(.*?)\n\s*\};', src, re.S
    )
    map_body = scale_map.group(1) if scale_map else ''
    wired = src.count('scale: press_scale') >= 2
    rows = []
    for size in ('Sm', 'Md', 'Lg'):
        symbol = names[size]
        mapped = bool(re.search(r'Size::%s\s*=>\s*crate::anim::%s' % (size, symbol), map_body))
        same = wants[size] is not None and constants[symbol] == wants[size] and mapped and wired
        rows.append((same, size, wants[size], constants[symbol] if mapped else None))

    print('pagination motion (v3 CSS vs Pagination):')
    for same, size, want, got in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'pagination', size, str(want), str(got)
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('PAGINATION MISMATCHES : %d' % bad)
    print()
    return bad


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
    motion_bad = (
        check_motions()
        + check_switch_motion()
        + check_toggle_button_motion()
        + check_pagination_motion()
    )
    print('UNIMPLEMENTED : %d' % len(missing_impl))
    print('MOTION BAD    : %d' % motion_bad)
    return len(missing_impl) + len(stale_docs) + motion_bad


if __name__ == '__main__':
    sys.exit(main())
