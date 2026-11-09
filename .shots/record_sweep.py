#!/usr/bin/env python3
"""Run the gallery interaction sweep and record its result as a checked file.

`cargo test -p herogpui-gallery --lib interaction_sweep` hovers and presses
every component example headlessly. With `HEROGPUI_SWEEP_REPORT_DIR` set,
each shard writes the examples it swept and the checks each one failed. This
script runs the sweep, merges the shards into
`docs/parity/runs/interaction-sweep.json` keyed by the inventory's section
ids, and stamps the source snapshot the run saw, so
`interaction_inventory.py` can tell a current run from a stale one.

    python3 .shots/record_sweep.py

Exits with the sweep's own status after writing the record.
"""
from __future__ import annotations

import json
import os
import platform
import subprocess
import sys
import tempfile
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / '.shots'))
import interaction_inventory  # noqa: E402

RECORD = ROOT / 'docs' / 'parity' / 'runs' / 'interaction-sweep.json'
COMMAND = ['cargo', 'test', '-p', 'herogpui-gallery', '--lib', 'interaction_sweep']


def section_ids(root):
    """(page title, heading) -> inventory section id, for every gallery example."""
    examples = json.loads((root / interaction_inventory.EXAMPLES).read_text(encoding='utf-8'))
    catalog = json.loads((root / interaction_inventory.CATALOG).read_text(encoding='utf-8'))['components']
    ids = {}
    for slug, rows in examples.items():
        title = catalog[slug].get('title', slug)
        for row in rows:
            ids[(title, row['heading'])] = interaction_inventory.gallery_section_id(slug, row['heading'])
    return ids


def read_shards(directory):
    rows = []
    shards = sorted(Path(directory).glob('shard-*.tsv'))
    for shard in shards:
        for line in shard.read_text(encoding='utf-8').splitlines():
            page, heading, checks = line.split('\t')
            rows.append((page, heading, [int(c) for c in checks.split(',') if c]))
    return len(shards), rows


def build_record(rows, shards, ids, source_sha256, status):
    sections = []
    for page, heading, checks in sorted(rows):
        key = (page, heading)
        if key not in ids:
            raise ValueError(f'swept example {page} / {heading} is not a gallery section')
        sections.append({'id': ids[key], 'page': page, 'heading': heading,
                         'result': 'fail' if checks else 'pass', 'failed_checks': checks})
    passed = sum(s['result'] == 'pass' for s in sections)
    return {
        'schema_version': 1,
        'kind': 'interaction-sweep',
        'recorded_on': date.today().isoformat(),
        'source_sha256': source_sha256,
        'platform': f'{platform.system()} {platform.machine()} (gpui headless test platform)',
        'command': ' '.join(COMMAND),
        'exit_status': status,
        'shards': shards,
        'coverage_limit': 'Light theme, overlays closed, pointer hover/press/leave only; '
                          'glyphs, icons and shadows are not observable.',
        'sections': sections,
        'summary': {'sections': len(sections), 'passed': passed, 'failed': len(sections) - passed},
    }


def main():
    source_sha256 = interaction_inventory.source_snapshot(ROOT)['sha256']
    with tempfile.TemporaryDirectory(prefix='herogpui-sweep-') as directory:
        env = {**os.environ, 'HEROGPUI_SWEEP_REPORT_DIR': directory}
        status = subprocess.run(COMMAND, cwd=ROOT, env=env).returncode
        shards, rows = read_shards(directory)
    if not rows:
        print('record_sweep: the sweep wrote no shard report', file=sys.stderr)
        return status or 1
    record = build_record(rows, shards, section_ids(ROOT), source_sha256, status)
    if interaction_inventory.source_snapshot(ROOT)['sha256'] != source_sha256:
        print('record_sweep: sources changed while the sweep ran; not recording', file=sys.stderr)
        return 1
    RECORD.parent.mkdir(parents=True, exist_ok=True)
    interaction_inventory.write_json_atomic(RECORD, record)
    summary = record['summary']
    print(f"record_sweep: {summary['passed']}/{summary['sections']} sections passed "
          f"across {shards} shards -> {RECORD.relative_to(ROOT)}")
    return status


if __name__ == '__main__':
    sys.exit(main())
