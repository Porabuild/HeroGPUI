"""Run all parity readers and fail on reported gaps, even if a reader exits zero.

    python .shots/parity_report.py
    python .shots/parity_report.py --output docs/parity/audit-results.json

A passing report describes the readers' mapped static coverage, not complete
visual/interaction parity. Omissions and pending coverage remain in the output.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import re
import subprocess
import sys

from bundle import PINNED_RELEASE, CSS_CACHE, css_cache, resolve
from interaction_inventory import ROOT, digest, json_digest, source_snapshot, write_json_atomic

# Explicit summary contracts: a missing/renamed section must not become zero.
# Each value is (positive input counters, zero-required failure counters).
CONTRACTS = {
    'a11y_audit.py': (('component modules read', 'RenderOnce impls read'), ('failures',)),
    'anatomy_audit.py': (('containment claims', 'documented parts'), ('UNMAPPED', 'NOT COMPOSED', 'NOT RENDERED')),
    'anim_audit.py': (('v3 animations implemented',), ('UNIMPLEMENTED', 'MOTION BAD')),
    'api_audit.py': (('documented props considered',), ('REAL GAPS',)),
    'behaviour_audit.py': (('behaviours claimed',), ('MISSING', 'UNMAPPED')),
    'demo_audit.py': (('pages compared', 'props v3 exercises'), ('NOT DEMONSTRATED',)),
    'design_audit.py': (('metrics compared', 'fills compared'),
                        ('unreadable', 'MISMATCHED', 'fills stale', 'WRONG FILLS',
                         'TOGGLE STYLE BAD', 'PAGINATION STYLE BAD', 'TABS STYLE BAD',
                         'COLOR PICKER STYLE BAD', 'TOOLBAR STYLE BAD', 'CARD STYLE BAD',
                         'SURFACE STYLE BAD', 'KBD/TYPO STYLE BAD')),
    'element_id_audit.py': (('rust source files walked', 'element id sites read'),
                            ('CONSTANT ids', 'FORMAT-built ids', 'stale allowlist entries')),
    'example_audit.py': (('examples documented',), ('MISSING', 'waiting on a feature')),
    'extra_audit.py': ((), ('UNEXPLAINED', 'banned v2 public names', 'scoped removed builders')),
    'fence_audit.py': (('rust files walked', 'doc-comment fences'),
                     ('IGNORED (never compiled)', 'unjustified / unknown', 'stale allowlist entries')),
    'inert_audit.py': (('controlled components', 'driven instances'), ('FROZEN', 'SHARED KEYS')),
    'package_audit.py': (('source packages',), ('PACKAGING ERRORS',)),
    'part_audit.py': (('parts v3 declares',), ('UNVERIFIED',)),
    'reason_audit.py': ((), ()),
    'reference_audit.py': (('component pages', 'metadata pages'), ('generic fallback',)),
    'state_audit.py': (('states claimed',), ('MISSING', 'UNMAPPED')),
    'theme_serde_audit.py': (('builder methods read', 'document fields read'), ('failures',)),
    'token_audit.py': (('variables declared', 'values compared'), ('MISSING', 'WRONG VALUES')),
    'write_only.py': ((), ()),
}
SUMMARY = re.compile(r'^([A-Za-z][A-Za-z0-9 /_()\-]*?)\s*:\s*(\d+)(?:\b.*)?$', re.M)
DIAGNOSTIC = re.compile(r'^(?:MISSING\s{2,}|UNMAPPED\s+\S|FROZEN\s{2,}|SHARED\s{2,}|\?\s|PART TABLE UNOWNED:|API SECTION AMBIGUOUS:|AUDIT READER ERROR:|no impl block matched)', re.M)


def analyze(name, output, exit_code):
    if name not in CONTRACTS:
        raise ValueError(f'no output contract for {name}')
    metrics = {}
    for match in SUMMARY.finditer(output):
        label = ' '.join(match.group(1).split())
        metrics.setdefault(label, []).append(int(match.group(2)))
    positive, zero = CONTRACTS[name]
    errors = []
    for label in (*positive, *zero):
        values = metrics.get(label)
        if values is None or len(values) != 1:
            errors.append(f'missing or duplicated summary: {label}')
        elif label in positive and values[0] <= 0:
            errors.append(f'empty input: {label}')
        elif label in zero and values[0] != 0:
            errors.append(f'{label}: {values[0]}')
    # Existing named subchecks can fail independently of the final summary.
    for label, values in metrics.items():
        if re.search(r'(?:MISMATCHES|STYLE BAD)$', label) and any(values):
            errors.append(f'{label}: {values}')
    if name == 'reason_audit.py':
        if not re.search(r'^REASON: \S', output, re.M):
            errors.append('missing omission reason inventory')
        for line in output.splitlines():
            if 'STALE:' in line or 'no matching doc row' in line:
                errors.append(line.strip())
    if name == 'reference_audit.py' and not re.search(r'^PASS$', output, re.M):
        errors.append('missing reference PASS marker')
    if name == 'write_only.py' and not re.search(r'^no write-only fields$', output, re.M):
        errors.append('missing write-only success marker')
    diagnostics = []
    for line in output.splitlines():
        if DIAGNOSTIC.match(line) and not SUMMARY.fullmatch(line):
            diagnostics.append(line)
    if diagnostics:
        errors.extend(diagnostics)
    if exit_code != 0:
        errors.append(f'process exit: {exit_code}')
    if not output.strip():
        errors.append('empty output')
    return {'audit': name, 'exit': exit_code,
            'status': 'failed' if errors else 'passed', 'metrics': metrics,
            'reported_gaps': list(dict.fromkeys(errors)), 'output_sha256': digest(output.encode()),
            'output': output}


def discover(root):
    names = sorted(p.name for p in (root / '.shots').glob('*audit.py')) + ['write_only.py']
    if set(names) != set(CONTRACTS):
        raise ValueError(f'audit output contracts differ from scripts: {sorted(set(names) ^ set(CONTRACTS))}')
    return names


def inputs(root):
    snapshot = source_snapshot(root)
    hashes = snapshot['files']
    for path in sorted((root / '.shots').glob('*.py')):
        hashes[path.relative_to(root).as_posix()] = digest(path.read_bytes())
    for name in ('web/src/data/reference.json', 'web/src/data/rust-examples.json', 'llms.txt',
                 'README.md', 'RELEASING.md', 'NOTICE', 'LICENSE', 'deny.toml'):
        hashes[name] = digest((root / name).read_bytes())
    for path in sorted((root / '.github').rglob('*')):
        if path.is_file():
            hashes[path.relative_to(root).as_posix()] = digest(path.read_bytes())
    for directory in ('crates', 'gallery'):
        for filename in ('LICENSE', 'NOTICE'):
            for path in sorted((root / directory).rglob(filename)):
                if path.is_file():
                    hashes[path.relative_to(root).as_posix()] = digest(path.read_bytes())
    hashes['effective-docs-bundle'] = digest(Path(resolve()).read_bytes())
    if not css_cache():
        raise ValueError('missing pinned CSS cache')
    cached_styles = sorted(Path(CSS_CACHE).rglob('*.css'))
    if not cached_styles:
        raise ValueError('empty pinned CSS cache')
    for path in cached_styles:
        hashes['effective-css/' + path.relative_to(CSS_CACHE).as_posix()] = digest(path.read_bytes())
    return json_digest(hashes)


def prepare_demo_inputs():
    """Resolve the reader's fallback inputs before fingerprinting a clean cache."""
    import demo_audit
    paths = set()
    fetch = demo_audit.fetch_text

    def record(url, refresh=False):
        text = fetch(url, refresh)
        paths.add(Path(demo_audit.preview_cache_path(url)))
        return text

    for page, blocks in demo_audit.v3_pages().items():
        if page in demo_audit.ex.NOT_A_COMPONENT:
            continue
        if not any(demo_audit.props_used(chunk) for chunk in blocks.values()):
            demo_audit.preview_props(page, read_source=record)
    return sorted(paths)


def run_fingerprint(root, demo_paths):
    return json_digest({'sources': inputs(root),
                        'demo_inputs': {p.name: digest(p.read_bytes()) for p in demo_paths}})


def require_pinned_environment():
    for key in ('HEROUI_BUNDLE', 'HEROUI_BUNDLE_UNPINNED'):
        if os.environ.get(key):
            raise ValueError(f'{key} overrides are not allowed in a pinned parity report')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='write the fresh report; failure still exits nonzero')
    args = parser.parse_args()
    try:
        require_pinned_environment()
        names = discover(ROOT)
        demo_paths = prepare_demo_inputs()
        initial = run_fingerprint(ROOT, demo_paths)
        rows = []
        for name in names:
            try:
                process = subprocess.run([sys.executable, str(ROOT / '.shots' / name)],
                                         cwd=ROOT, capture_output=True, text=True, timeout=180)
                output = process.stdout + process.stderr
                row = analyze(name, output, process.returncode)
            except subprocess.TimeoutExpired as error:
                def decoded(value):
                    return value.decode(errors='replace') if isinstance(value, bytes) else value or ''
                row = analyze(name, decoded(error.stdout) + decoded(error.stderr), 124)
                row['reported_gaps'].append('audit timed out after 180 seconds')
            rows.append(row)
            print(f'{name}: {row["status"]}', flush=True)
            for gap in row['reported_gaps']:
                print('  ' + gap, flush=True)
        final = run_fingerprint(ROOT, demo_paths)
        stable = initial == final
        passed = stable and all(row['status'] == 'passed' for row in rows)
        report = {'schema_version': 2, 'target': PINNED_RELEASE,
                  'recorded_on': datetime.now(timezone.utc).isoformat(),
                  'inputs_sha256': initial, 'inputs_stable_during_run': stable,
                  'status': 'passed' if passed else 'failed',
                  'note': 'Static mapped coverage only; omissions, unreviewed specimens and runtime parity are separate.',
                  'audits': rows}
        if args.output:
            write_json_atomic(args.output, report)
        if not stable:
            print('READER ERROR: audit inputs changed during the run', file=sys.stderr)
        return 0 if passed else 1
    except (ValueError, OSError, RuntimeError) as error:
        print(f'REPORT ERROR: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
