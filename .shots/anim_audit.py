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

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bundle import css_cache as _css_cache, resolve as _resolve_bundle

# The pinned v3.2.4 bundle. See .shots/bundle.py: reading upstream live would
# measure this port against whatever HeroUI shipped most recently.
BUNDLE = _resolve_bundle()
SRC = 'crates/herogpui-components/src/'
THEME = 'crates/herogpui-theme/src/'
# The v3 stylesheets `design_audit.py --fetch` caches. Motion lives in the CSS,
# not the docs bundle, so the per-overlay checks read from here.
CACHE = os.path.join(os.environ.get('TEMP', '/tmp'), 'heroui-css')
# An empty cache is not a clean motion report: without the stylesheets this
# audit measured every timing against nothing and called 22 of them wrong.
if not _css_cache():
    sys.stderr.write('anim_audit: no v3 stylesheets; run design_audit.py --fetch\n')
    raise SystemExit(2)

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
    # v3 presses five different amounts: a button 0.97, a menu row 0.98, a
    # pagination arrow 0.96, a calendar cell and radio control 0.95, and a
    # RangeCalendar cell 0.9.
    'scale(0.98)': 'PRESSED_SCALE_SUBTLE',
    'scale(0.96)': 'PRESSED_SCALE_FIRM',
    'scale(0.95)': 'PRESSED_SCALE_DEEP',
    'scale: 0.9': 'PRESSED_SCALE_RANGE',
    '@keyframes caret-blink': 'pub fn caret_blink',
    '@keyframes progress-bar-indeterminate': 'pub fn progress_bar_indeterminate_ease',
    '@keyframes progress-circle-spin': 'pub fn progress_circle_spin_turn',
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
}

# `reduce_motion` is reached as a free method on `cx` or as a trait call taking
# it, depending on the gpui/theme vintage. Which one a file spells is not what
# any of these checks measure -- they measure that the code consults it at all
# -- so every reader below matches either. Pinning one spelling made six
# readers go stale on an API bump while the behaviour was untouched.
REDUCE_MOTION = r'(?:cx\.reduce_motion\(\)|ActiveTheme::reduce_motion\(cx\))'

_RM_TAIL = {
    '--tooltip-delay': 'tooltip_delay',
    '--tooltip-close-delay': 'tooltip_close_delay',
    '--skeleton-animation': 'skeleton_animation',
    'transition-colors': 'pub const TRANSITION_MS',
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
        # Anchored on the binding and the concept rather than the call
        # path: which module spells `reduce_motion` is a gpui/theme
        # detail, and pinning it made this read stale on an API bump
        # while the behaviour it checks had not changed.
        and re.search(r'let reduce_motion = [^;]*reduce_motion\(', track_source)
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


def check_color_area_motion():
    """ColorArea thumb width/height transition against its component CSS."""
    css_path = os.path.join(CACHE, 'color-area.css')
    src_path = os.path.join(SRC, 'color_picker.rs')
    if not os.path.exists(css_path):
        print('color area motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    thumb = re.search(r'\.color-area__thumb\s*\{(.*)\n\}', css, re.S)
    body = thumb.group(1) if thumb else ''
    want_width = re.search(r'width\s+(\d+)ms\s+var\(--ease-([\w-]+)\)', body)
    want_height = re.search(r'height\s+(\d+)ms\s+var\(--ease-([\w-]+)\)', body)
    got_ms = re.search(r'const COLOR_AREA_THUMB_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_curve = re.search(
        r'COLOR_AREA_THUMB_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    required = (thumb, want_width, want_height, got_ms, got_curve)
    if any(value is None for value in required):
        print('color area motion: unreadable')
        return 1
    want_curve = CURVES.get(want_width.group(2))
    timing = (
        want_width.group(1) == want_height.group(1) == got_ms.group(1)
        and want_width.group(2) == want_height.group(2)
        and want_curve == got_curve.group(1)
    )
    geometry = (
        'COLOR_AREA_THUMB_IDLE_PX' in src
        and 'COLOR_AREA_THUMB_DRAGGING_PX' in src
        and 'place_color_area_thumb(thumb, next)' in src
    )
    reduced = (
        body.find('transition:') < body.find('motion-reduce:transition-none')
        and re.search('if ' + REDUCE_MOTION,
                      src.split('fn color_area_thumb_motion(', 1)[1].split(
                          '/// ColorArea', 1)[0])
    )
    listener_free = (
        '.child(thumb_motion.render(thumb_visual))' in src
        and 'thumb = thumb.on_hover' in src
    )
    reversal = 'current.from = current.size.get();' in src
    rows = [
        (timing, 'width + height', '%sms %s' % (want_width.group(1), want_curve),
         '%sms %s' % (got_ms.group(1), got_curve.group(1))),
        (geometry, 'drag geometry', '16px to 20px', 'animated size' if geometry else 'missing'),
        (reduced, 'reduced motion', 'transition-none', 'direct size' if reduced else 'missing'),
        (listener_free, 'animation owner', 'listener-free visual',
         'child visual' if listener_free else 'listener wrapper'),
        (reversal, 'reversal', 'current rendered size', 'preserved' if reversal else 'endpoint jump'),
    ]
    print('color area motion (v3 CSS vs ColorArea):')
    for same, name, want, got in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'color-area', name, want, got
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('COLOR AREA MISMATCHES : %d' % bad)
    print()
    return bad


def check_select_motion():
    """Select must apply both halves of its pinned list-surface motion."""
    src_path = os.path.join(SRC, 'select.rs')
    if not os.path.exists(src_path):
        print('select motion: no source')
        return 1
    src = io.open(src_path, encoding='utf-8').read()
    rows = [
        (
            'crate::anim::entering_zoom(' in src
            and 'crate::anim::Motion::LIST_IN' in src,
            'enter path',
            'LIST_IN',
        ),
        (
            'overlay_phase == util::OverlayPhase::Exiting' in src
            and 'crate::anim::exiting(' in src
            and 'crate::anim::Motion::LIST_OUT' in src,
            'exit path',
            'LIST_OUT',
        ),
    ]
    print('select motion wiring:')
    for same, name, want in rows:
        print('%s %-14s %-12s %s' % (' ' if same else '!', 'select', name, want))
    print('SELECT MISMATCHES : %d' % sum(not same for same, _, _ in rows))
    print()
    return sum(not same for same, _, _ in rows)


def check_autocomplete_motion():
    """Autocomplete must apply both halves of its unique fluid motion."""
    src_path = os.path.join(SRC, 'autocomplete.rs')
    if not os.path.exists(src_path):
        print('autocomplete motion: no source')
        return 1
    src = io.open(src_path, encoding='utf-8').read()
    rows = [
        (
            'crate::anim::entering_zoom(' in src
            and 'crate::anim::Motion::FLUID_IN' in src,
            'enter path',
            'FLUID_IN',
        ),
        (
            'overlay_phase == util::OverlayPhase::Exiting' in src
            and 'crate::anim::exiting(' in src
            and 'crate::anim::Motion::FLUID_OUT' in src,
            'exit path',
            'FLUID_OUT',
        ),
    ]
    print('autocomplete motion wiring:')
    for same, name, want in rows:
        print('%s %-14s %-12s %s' % (' ' if same else '!', 'autocomplete', name, want))
    print('AUTOCOMPLETE MISMATCHES : %d' % sum(not same for same, _, _ in rows))
    print()
    return sum(not same for same, _, _ in rows)


def check_tabs_motion():
    """Tabs indicator and separator property transitions."""
    css_path = os.path.join(CACHE, 'tabs.css')
    src_path = os.path.join(SRC, 'tabs.rs')
    if not os.path.exists(css_path):
        print('tabs motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    separator = re.search(
        r'/\* Tab separator \*/\s*\.tabs__separator\s*\{(.*?)(?=\n/\* Tab panel)',
        css,
        re.S,
    )
    indicator = re.search(
        r'/\* Tab indicator.*?\*/\s*\.tabs__indicator\s*\{(.*?)(?=\n/\* ={10,})',
        css,
        re.S,
    )
    want_separator = re.search(
        r'opacity\s+(\d+)ms\s+var\(--ease-([\w-]+)\)',
        separator.group(1) if separator else '',
    )
    want_indicator_ms = re.search(
        r'transition-duration:\s*(\d+)ms',
        indicator.group(1) if indicator else '',
    )
    want_indicator_curve = re.search(
        r'transition-timing-function:\s*var\(--ease-([\w-]+)\)',
        indicator.group(1) if indicator else '',
    )
    got_separator_ms = re.search(r'const SEPARATOR_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_separator_curve = re.search(
        r'SEPARATOR_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    got_indicator_ms = re.search(r'const INDICATOR_TRANSITION_MS:\s*u64\s*=\s*(\d+)', src)
    got_indicator_curve = re.search(
        r'INDICATOR_TRANSITION_MS\)\)\s*\.with_easing\(\|t\|\s*'
        r'crate::anim::Curve::(\w+)\.at\(t\)\)',
        src,
        re.S,
    )
    required = (
        separator,
        indicator,
        want_separator,
        want_indicator_ms,
        want_indicator_curve,
        got_separator_ms,
        got_separator_curve,
        got_indicator_ms,
        got_indicator_curve,
    )
    if any(value is None for value in required):
        print('tabs motion: unreadable')
        return 1

    separator_reduced = (
        separator.group(1).find('transition:')
        < separator.group(1).find('motion-reduce:transition-none')
        and 'fn separator_motion(' in src
        and re.search('if ' + REDUCE_MOTION,
                      src.split('fn separator_motion(', 1)[1].split(
                          '/// HeroUI Tabs', 1)[0])
    )
    indicator_reduced = (
        indicator.group(1).find('transition-duration:')
        < indicator.group(1).find('motion-reduce:transition-none')
        and 'fn indicator_motion(' in src
        and re.search('if ' + REDUCE_MOTION,
                      src.split('fn indicator_motion(', 1)[1].split(
                          '#[derive(Clone, Debug, Default)]', 1)[0])
    )
    properties = (
        'transition-property: translate, width, height' in indicator.group(1)
        and all(('to.%s - from.%s' % (name, name)) in src for name in ('x', 'y', 'width', 'height'))
    )
    listener_free = (
        'frame.render(indicator)' in src
        and '.render(separator)' in src
        and 'tabs-indicator-slide-' in src
        and 'tabs-separator-fade-' in src
    )
    reversal = (
        'current.from = current.rect.get();' in src
        and 'current.from = current.opacity.get();' in src
    )
    want_separator_curve = CURVES.get(want_separator.group(2))
    want_indicator_curve = CURVES.get(want_indicator_curve.group(1))
    rows = [
        (
            int(want_separator.group(1)) == int(got_separator_ms.group(1))
            and want_separator_curve == got_separator_curve.group(1),
            'separator opacity',
            '%sms %s' % (want_separator.group(1), want_separator_curve),
            '%sms %s' % (got_separator_ms.group(1), got_separator_curve.group(1)),
        ),
        (
            int(want_indicator_ms.group(1)) == int(got_indicator_ms.group(1))
            and want_indicator_curve == got_indicator_curve.group(1),
            'indicator geometry',
            '%sms %s' % (want_indicator_ms.group(1), want_indicator_curve),
            '%sms %s' % (got_indicator_ms.group(1), got_indicator_curve.group(1)),
        ),
        (properties, 'indicator properties', 'translate width height', 'all four' if properties else 'missing'),
        (separator_reduced and indicator_reduced, 'reduced motion', 'transition-none', 'direct geometry' if separator_reduced and indicator_reduced else 'missing'),
        (listener_free, 'animation owner', 'listener-free children', 'child elements' if listener_free else 'listener owner'),
        (reversal, 'reversal', 'current rendered values', 'preserved' if reversal else 'endpoint jump'),
    ]
    print('tabs motion (v3 CSS vs Tabs):')
    for same, name, want, got in rows:
        print('%s %-14s %-22s %-26s %s' % (' ' if same else '!', 'tabs', name, want, got))
    bad = sum(not same for same, _, _, _ in rows)
    print('TABS MISMATCHES : %d' % bad)
    print()
    return bad


def check_button_motion():
    """Button's size-specific pressed scale."""
    css_path = os.path.join(CACHE, 'button.css')
    src_path = os.path.join(SRC, 'button.rs')
    anim_path = os.path.join(SRC, 'anim.rs')
    if not os.path.exists(css_path):
        print('button motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    anim = io.open(anim_path, encoding='utf-8').read()

    def scale_in(pattern):
        block = re.search(pattern, css, re.S)
        scale = re.search(r'transform:\s*scale\(([\d.]+)\)', block.group(1) if block else '')
        return float(scale.group(1)) if scale else None

    wants = {
        'Sm': scale_in(r'\.button--sm\s*\{(.*?)(?=\n\.button--md)'),
        'Md': scale_in(r'\.button\s*\{(.*?)(?=\n/\* Size variants)'),
        'Lg': scale_in(r'\.button--lg\s*\{(.*?)(?=\n/\* Color variants)'),
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
    wired = 'scale: press_scale' in src
    rows = []
    for size in ('Sm', 'Md', 'Lg'):
        symbol = names[size]
        mapped = bool(re.search(r'Size::%s\s*=>\s*crate::anim::%s' % (size, symbol), map_body))
        same = wants[size] is not None and constants[symbol] == wants[size] and mapped and wired
        rows.append((same, size, wants[size], constants[symbol] if mapped else None))

    print('button motion (v3 CSS vs Button):')
    for same, size, want, got in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'button', size, str(want), str(got)
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('BUTTON MISMATCHES : %d' % bad)
    print()
    return bad


def check_number_field_motion():
    """NumberField's spin buttons use the pinned pressed scale and fill."""
    css_path = os.path.join(CACHE, 'number-field.css')
    src_path = os.path.join(SRC, 'number_field.rs')
    anim_path = os.path.join(SRC, 'anim.rs')
    if not os.path.exists(css_path):
        print('number field motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    src = io.open(src_path, encoding='utf-8').read()
    anim = io.open(anim_path, encoding='utf-8').read()

    block = re.search(
        r'\.number-field__increment-button,\s*\n\.number-field__decrement-button\s*\{(.*?)\n\}',
        css,
        re.S,
    )
    want = re.search(r'transform:\s*scale\(([\d.]+)\)', block.group(1) if block else '')
    constant = re.search(r'pub const PRESSED_SCALE:\s*f32\s*=\s*([\d.]+)', anim)
    stepper = re.search(r'fn stepper_btn\((.*?)\n\}', src, re.S)
    body = stepper.group(1) if stepper else ''
    wired = (
        'pressed_with_background' in body
        and 'scale: crate::anim::PRESSED_SCALE' in body
        and 'pressed_bg' in body
    )
    want_value = float(want.group(1)) if want else None
    got_value = float(constant.group(1)) if constant and wired else None
    same = want_value is not None and want_value == got_value
    print('number field motion (v3 CSS vs NumberField):')
    print('%s %-14s %-16s %-22s %s' % (
        ' ' if same else '!', 'number-field', 'stepper press', str(want_value), str(got_value)
    ))
    bad = int(not same)
    print('NUMBER FIELD MISMATCHES : %d' % bad)
    print()
    return bad


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


def check_drawer_motion():
    """The Drawer's 200ms exit must stay mounted for the whole slide."""
    css_path = os.path.join(CACHE, 'drawer.css')
    if not os.path.exists(css_path):
        print('drawer motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    anim = io.open(os.path.join(SRC, 'anim.rs'), encoding='utf-8').read()
    drawer = io.open(os.path.join(SRC, 'drawer.rs'), encoding='utf-8').read()
    util = io.open(os.path.join(SRC, 'util.rs'), encoding='utf-8').read()
    want_enter = re.search(r'--drawer-enter-duration:\s*(\d+)ms', css)
    want_exit = re.search(r'--drawer-exit-duration:\s*(\d+)ms', css)
    got_enter = re.search(
        r'pub const DRAWER_IN:\s*Motion\s*=\s*Motion\s*\{\s*ms:\s*(\d+)',
        anim,
        re.S,
    )
    got_exit = re.search(
        r'pub const DRAWER_OUT:\s*Motion\s*=\s*Motion\s*\{\s*ms:\s*(\d+)',
        anim,
        re.S,
    )
    wired = bool(
        re.search(
            r'overlay_scope_with_exit\(\s*window,\s*cx,.*?self\.is_open,\s*true,'
            r'\s*crate::anim::Motion::DRAWER_OUT\.ms',
            drawer,
            re.S,
        )
        and re.search(
            r'pub fn overlay_scope_with_exit\(.*?from_millis\(exit_ms\)',
            util,
            re.S,
        )
    )
    want_enter_ms = int(want_enter.group(1)) if want_enter else None
    want_exit_ms = int(want_exit.group(1)) if want_exit else None
    got_enter_ms = int(got_enter.group(1)) if got_enter else None
    got_exit_ms = int(got_exit.group(1)) if got_exit else None
    enter_same = want_enter_ms is not None and want_enter_ms == got_enter_ms
    exit_same = want_exit_ms is not None and want_exit_ms == got_exit_ms and wired
    print('drawer motion (v3 CSS vs Drawer):')
    print('%s %-14s %-16s %-22s %s' % (
        ' ' if enter_same else '!',
        'drawer',
        'enter duration',
        '%sms' % want_enter_ms if want_enter_ms is not None else 'unreadable',
        '%sms' % got_enter_ms if got_enter_ms is not None else 'unreadable',
    ))
    print('%s %-14s %-16s %-22s %s' % (
        ' ' if exit_same else '!',
        'drawer',
        'exit lifetime',
        '%sms' % want_exit_ms if want_exit_ms is not None else 'unreadable',
        '%sms, duration-aware scope' % got_exit_ms if wired else 'not wired',
    ))
    bad = (0 if enter_same else 1) + (0 if exit_same else 1)
    print('DRAWER MISMATCHES : %d' % bad)
    print()
    return bad


def check_progress_circle_motion():
    """ProgressCircle's pinned spin duration, loop and reduced-motion gate."""
    css_path = os.path.join(CACHE, 'progress-circle.css')
    if not os.path.exists(css_path):
        print('progress circle motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    anim = io.open(os.path.join(SRC, 'anim.rs'), encoding='utf-8').read()
    progress = io.open(os.path.join(SRC, 'progress.rs'), encoding='utf-8').read()

    want = re.search(r'animation:\s*progress-circle-spin\s+(\d+)s\s+linear\s+infinite', css)
    got = re.search(r'pub const PROGRESS_CIRCLE_SPIN_MS:\s*u64\s*=\s*(\d+)', anim)
    want_ms = int(want.group(1)) * 1000 if want else None
    got_ms = int(got.group(1)) if got else None
    wired = bool(re.search(
        r'with_animation\(\s*"progress-circle-spin"[\s\S]{0,300}?'
        r'PROGRESS_CIRCLE_SPIN_MS[\s\S]{0,120}?\.repeat\(\)',
        progress,
    ))
    reduced = (
        'motion-reduce:animate-none' in css
        and re.search(r'self\.is_indeterminate && !' + REDUCE_MOTION, progress)
    )
    linear = 'pub fn progress_circle_spin_turn' in anim

    rows = [
        (want_ms is not None and want_ms == got_ms and wired and linear,
         'spin', '%sms linear infinite' % want_ms if want_ms is not None else 'unreadable',
         '%sms linear repeat' % got_ms if got_ms is not None and wired and linear else 'not wired'),
        (reduced, 'reduced motion', 'animate-none', 'static arc' if reduced else 'missing'),
    ]
    print('progress circle motion (v3 CSS vs ProgressCircle):')
    for same, name, want_value, got_value in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'progress-circle', name, want_value, got_value
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('PROGRESS CIRCLE MISMATCHES : %d' % bad)
    print()
    return bad


def check_progress_bar_motion():
    """ProgressBar's pinned indeterminate geometry, easing and fallback."""
    css_path = os.path.join(CACHE, 'progress-bar.css')
    if not os.path.exists(css_path):
        print('progress bar motion: no stylesheet')
        return 1
    css = io.open(css_path, encoding='utf-8', errors='replace').read()
    anim = io.open(os.path.join(SRC, 'anim.rs'), encoding='utf-8').read()
    progress = io.open(os.path.join(SRC, 'progress.rs'), encoding='utf-8').read()

    want = re.search(
        r'animation:\s*progress-bar-indeterminate\s+([\d.]+)s\s+'
        r'cubic-bezier\(0\.65,\s*0,\s*0\.35,\s*1\)\s+infinite',
        css,
    )
    got = re.search(r'pub const PROGRESS_BAR_INDETERMINATE_MS:\s*u64\s*=\s*(\d+)', anim)
    want_ms = round(float(want.group(1)) * 1000) if want else None
    got_ms = int(got.group(1)) if got else None
    curve = bool(re.search(
        r'pub fn progress_bar_indeterminate_ease\(\).*?'
        r'cubic_bezier\(0\.65,\s*0\.0,\s*0\.35,\s*1\.0,\s*t\)',
        anim,
        re.S,
    ))
    wired = bool(re.search(
        r'with_animation\(\s*"progress-bar-indeterminate"[\s\S]{0,360}?'
        r'PROGRESS_BAR_INDETERMINATE_MS[\s\S]{0,180}?'
        r'with_easing\(crate::anim::progress_bar_indeterminate_ease\(\)\)'
        r'[\s\S]{0,80}?\.repeat\(\)',
        progress,
    ))
    want_width_match = re.search(
        r'&:not\(\[aria-valuenow\]\)[\s\S]*?'
        r'\.progress-bar__fill\s*\{[\s\S]*?@apply\s+w-(\d+)/(\d+)',
        css,
    )
    keyframes = re.search(
        r'@keyframes\s+progress-bar-indeterminate\s*\{[\s\S]*?'
        r'translateX\((-?[\d.]+)%\)[\s\S]*?translateX\((-?[\d.]+)%\)',
        css,
    )
    got_geometry = re.search(
        r'let track = if self\.is_indeterminate\s*&&\s*!' + REDUCE_MOTION +
        r'[\s\S]{0,900}?'
        r'\.w\(gpui::relative\(([\d.]+)\)\)[\s\S]{0,700}?'
        r'delta\s*\*\s*([\d.]+)\s*([-+])\s*([\d.]+)',
        progress,
    )
    want_width = (
        int(want_width_match.group(1)) / int(want_width_match.group(2))
        if want_width_match and int(want_width_match.group(2)) != 0
        else None
    )
    want_start = (
        want_width * float(keyframes.group(1)) / 100
        if want_width is not None and keyframes
        else None
    )
    want_span = (
        want_width * (float(keyframes.group(2)) - float(keyframes.group(1))) / 100
        if want_width is not None and keyframes
        else None
    )
    got_width = float(got_geometry.group(1)) if got_geometry else None
    got_span = float(got_geometry.group(2)) if got_geometry else None
    got_start = (
        float(got_geometry.group(4)) * (-1 if got_geometry.group(3) == '-' else 1)
        if got_geometry
        else None
    )
    geometry = (
        want_width is not None
        and want_start is not None
        and want_span is not None
        and got_width is not None
        and got_start is not None
        and got_span is not None
        and abs(want_width - got_width) < 0.0001
        and abs(want_start - got_start) < 0.0001
        and abs(want_span - got_span) < 0.0001
    )
    static_width = re.search(
        r'else if self\.is_indeterminate\s*\{[\s\S]{0,500}?'
        r'\.w\(gpui::relative\(([\d.]+)\)\)',
        progress,
    )
    static_width_value = float(static_width.group(1)) if static_width else None
    reduced = (
        'motion-reduce:animate-none' in css
        and want_width is not None
        and static_width_value is not None
        and abs(want_width - static_width_value) < 0.0001
    )
    want_fill = re.search(
        r'transition:\s*width\s+(\d+)ms\s+var\(--ease-out\)', css
    )
    got_fill = re.search(r'pub const PROGRESS_BAR_FILL_MS:\s*u64\s*=\s*(\d+)', anim)
    want_fill_ms = int(want_fill.group(1)) if want_fill else None
    got_fill_ms = int(got_fill.group(1)) if got_fill else None
    fill_transition = (
        want_fill_ms is not None
        and want_fill_ms == got_fill_ms
        and 'progress_bar_motion(' in progress
        and 'PROGRESS_BAR_FILL_MS' in progress
        and '.with_easing(|t| crate::anim::Curve::Out.at(t))' in progress
        and re.search(r'!self\.is_indeterminate && !' + REDUCE_MOTION, progress)
    )
    reversal = (
        'self.from = self.width.get();' in progress
        and 'width.set(next);' in progress
    )
    instance_scoped = '.id(self.id.clone())' in progress
    rows = [
        (want_ms is not None and want_ms == got_ms and curve and wired,
         'spin', '%sms cubic-bezier' % want_ms if want_ms is not None else 'unreadable',
         '%sms pinned curve' % got_ms if got_ms is not None and curve and wired else 'not wired'),
        (geometry, 'geometry',
         ('%.0f%%; %.0f%% to %.0f%%' %
          (want_width * 100, float(keyframes.group(1)), float(keyframes.group(2))))
         if want_width is not None and keyframes else 'unreadable',
         'derived full sweep' if geometry else 'mismatch'),
        (reduced, 'reduced motion',
         'animate-none at %.0f%%' % (want_width * 100) if want_width is not None else 'unreadable',
         'static %.0f%%' % (static_width_value * 100) if reduced else 'missing'),
        (fill_transition, 'fill width',
         '%sms ease-out' % want_fill_ms if want_fill_ms is not None else 'unreadable',
         '%sms Out' % got_fill_ms if got_fill_ms is not None and fill_transition else 'not wired'),
        (reversal, 'fill reversal', 'current rendered width', 'preserved' if reversal else 'endpoint jump'),
        (instance_scoped, 'instance scope', 'independent timelines',
         'stable root id' if instance_scoped else 'shared animation chain'),
    ]
    print('progress bar motion (v3 CSS vs ProgressBar):')
    for same, name, want_value, got_value in rows:
        print('%s %-14s %-16s %-22s %s' % (
            ' ' if same else '!', 'progress-bar', name, want_value, got_value
        ))
    bad = sum(not same for same, _, _, _ in rows)
    print('PROGRESS BAR MISMATCHES : %d' % bad)
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
        + check_color_area_motion()
        + check_select_motion()
        + check_autocomplete_motion()
        + check_tabs_motion()
        + check_button_motion()
        + check_number_field_motion()
        + check_toggle_button_motion()
        + check_pagination_motion()
        + check_drawer_motion()
        + check_progress_circle_motion()
        + check_progress_bar_motion()
    )
    print('UNIMPLEMENTED : %d' % len(missing_impl))
    print('MOTION BAD    : %d' % motion_bad)
    return len(missing_impl) + len(stale_docs) + motion_bad


if __name__ == '__main__':
    sys.exit(main())
