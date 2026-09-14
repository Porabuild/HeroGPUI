"""Refresh/check the parity work queue without turning inventory into proof.

    python .shots/interaction_inventory.py --refresh
    python .shots/interaction_inventory.py --check

Pass --upstream and --baseline to refresh tagged demo/source inventories too.
Those directories must be verified extractions of the commits already recorded
in the inventory; this command never changes the upstream release or fetches main.
"""
from __future__ import annotations

import argparse
from copy import deepcopy
from datetime import date
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import tempfile

from bundle import PINNED_RELEASE

ROOT = Path(__file__).resolve().parent.parent
INVENTORY = Path('docs/parity/interaction-inventory.json')
EXAMPLES = Path('web/src/data/rust-examples.json')
REFERENCE = Path('web/src/data/reference.json')
CATALOG = Path('web/src/data/catalog.json')
STATUSES = {
    'unreviewed', 'specified', 'measured-gap', 'implemented-unverified',
    'verified', 'intentional-deviation', 'platform-limited', 'not-applicable',
}
SPECIMEN_SCHEMA_VERSION = 3
INPUT_FILES = (
    'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml', '.gitattributes',
    'gallery/Cargo.toml', 'web/pnpm-lock.yaml', 'web/package.json', 'web/next.config.ts',
    'web/public/gallery/herogpui_web_bg.wasm', 'web/public/gallery/herogpui_web.js',
    'web/public/gallery/index.html', 'crates/herogpui-web/index.html',
    '.shots/heroui-bundle.txt.gz', '.shots/heroui-css-v3.2.5.tar.gz',
)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_digest(value):
    return digest(json.dumps(value, sort_keys=True, ensure_ascii=False).encode())


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def write_json_atomic(path, value):
    """Do not truncate recorded evidence if serialization or writing fails."""
    data = json.dumps(value, indent=2, ensure_ascii=False) + '\n'
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', dir=path.parent,
                                         prefix='.' + path.name + '.', delete=False) as handle:
            temporary = Path(handle.name)
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        temporary.replace(path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def source_snapshot(root):
    """Conservative invalidation: any rendering/input/source change needs review."""
    files = set()
    for suffix in ('.py', '.ps1', '.sh'):
        files.update(p for p in (root / '.shots').glob('*' + suffix) if p.is_file())
    for directory in ('crates', 'gallery/src'):
        files.update(p for p in (root / directory).rglob('*.rs') if p.is_file())
        files.update(p for p in (root / directory).rglob('Cargo.toml') if p.is_file())
    for directory in ('gallery/assets', 'crates/herogpui-web/fonts', 'web/public/gallery/fonts',
                      'web/src', 'web/scripts'):
        files.update(p for p in (root / directory).rglob('*') if p.is_file())
    for name in INPUT_FILES:
        path = root / name
        if not path.is_file():
            raise ValueError(f'missing inventory input: {name}')
        files.add(path)
    # Build output follows whatever was last compiled in this checkout, and a
    # fresh CI checkout has none of it; only repository sources invalidate.
    files = {p for p in files if not {'target', '.git'}.intersection(p.relative_to(root).parts)}
    if not any(p.suffix == '.rs' for p in files):
        raise ValueError('no Rust sources found')
    hashes = {p.relative_to(root).as_posix(): digest(p.read_bytes()) for p in sorted(files)}
    return {'sha256': json_digest(hashes), 'files': hashes}


def verification_fingerprint(inventory):
    inputs = {key: inventory.get(key) for key in (
        'source_snapshot', 'gallery_source', 'reference_source', 'catalog_source',
        'upstream', 'upstream_source_snapshot')}
    inputs['upstream_demos'] = {slug: [(row['path'], row['sha256']) for row in rows]
                               for slug, rows in inventory.get('upstream_demos', {}).items()}
    inputs['source_changes'] = [(row['path'], row['before_sha256'], row['after_sha256'])
                                for row in inventory.get('source_changes', [])]
    return json_digest(inputs)


def finish_refresh(previous, current):
    fingerprint = verification_fingerprint(current)
    current['verification_sha256'] = fingerprint
    if fingerprint != previous.get('verification_sha256'):
        for specimen in current.get('specimens', []):
            if specimen.get('status') in ('verified', 'intentional-deviation', 'platform-limited', 'not-applicable'):
                specimen.setdefault('review_history', []).append(deepcopy({
                    k: v for k, v in specimen.items() if k != 'review_history'}))
                specimen['status'] = 'implemented-unverified'
                specimen['invalidation'] = 'upstream, implementation, artifact or documentation inputs changed'
    if current != previous:
        current['recorded_on'] = date.today().isoformat()
    return current


def merge_record(current, previous, fingerprint):
    """Preserve an unchanged review; retain changed evidence as stale history."""
    if previous and previous.get(fingerprint) == current[fingerprint]:
        return {**previous, **current, 'status': previous.get('status', 'unreviewed')}
    result = {**current, 'status': 'unreviewed'}
    if previous:
        result['history'] = [*previous.get('history', []),
                             {k: v for k, v in previous.items() if k != 'history'}]
    return result


def specimen_key_from_code(code):
    """Return an explicitly claimed gallery key when the example exposes one."""
    match = re.search(r'specimen_body\(\s*["\']([^"\']+)', code)
    return match.group(1) if match else None


def gallery_specimen(slug, title, row, reference):
    """Seed one honest, unreviewed queue item for each rendered gallery section.

    A section is only an inventory seed: loops and prose can still contain many
    concrete variants and states. Keeping that distinction in the record makes
    it possible to expand the queue without treating a page heading as proof.
    """
    heading = row['heading']
    section_slug = re.sub(r'[^a-z0-9]+', '-', heading.lower()).strip('-') or 'section'
    section_id = f'gallery/{slug}/{section_slug}'
    code_sha256 = digest(row['code'].encode())
    content_sha256 = json_digest(row)
    return {
        'id': section_id,
        'component': title,
        'scope': 'gallery-section',
        'status': 'unreviewed',
        'upstream': {
            'tag': PINNED_RELEASE,
            'source': None,
            'selector': None,
            'demoSection': heading,
        },
        'port': {
            'module': slug,
            'galleryPage': title,
            'gallerySection': heading,
            'specimenKey': specimen_key_from_code(row['code']),
        },
        'configuration': {},
        'state': 'section-seed',
        'setup': [],
        'assertions': [
            'Expand this section into concrete variant/state specimens before verification.',
        ],
        'evidence': {'upstream': [], 'native': [], 'wasm': [], 'tests': []},
        'source': {
            'gallery': EXAMPLES.as_posix(),
            'reference': REFERENCE.as_posix(),
            'codeSha256': code_sha256,
            'contentSha256': content_sha256,
            'docsSource': reference.get('docsSource'),
        },
    }


def refresh_gallery(old, root):
    examples = read_json(root / EXAMPLES)
    reference = read_json(root / REFERENCE)
    catalog = read_json(root / CATALOG)['components']
    if not examples or set(examples) != set(reference) or set(examples) != set(catalog):
        raise ValueError('catalog, reference and examples must have the same nonempty component set')
    if any(row['version'] != PINNED_RELEASE.removeprefix('v') for row in reference.values()):
        raise ValueError('reference version differs from the pinned release')
    output = deepcopy(old)
    output['schema_version'] = SPECIMEN_SCHEMA_VERSION
    output['gallery_source'] = {'path': EXAMPLES.as_posix(), 'sha256': digest((root / EXAMPLES).read_bytes())}
    output['reference_source'] = {'path': REFERENCE.as_posix(), 'sha256': digest((root / REFERENCE).read_bytes())}
    output['catalog_source'] = {'path': CATALOG.as_posix(), 'sha256': digest((root / CATALOG).read_bytes())}
    output['source_snapshot'] = source_snapshot(root)
    output['gallery_sections'] = {}
    retired = deepcopy(old.get('retired_gallery_sections', {}))
    for slug, rows in sorted(examples.items()):
        headings = [row['heading'] for row in rows]
        if not headings or len(headings) != len(set(headings)):
            raise ValueError(f'{slug}: empty or duplicate gallery sections')
        prior = {row['heading']: row for row in old.get('gallery_sections', {}).get(slug, [])}
        output['gallery_sections'][slug] = [
            merge_record({'heading': row['heading'], 'code_sha256': digest(row['code'].encode()),
                          'content_sha256': json_digest(row)}, prior.get(row['heading']), 'content_sha256')
            for row in rows
        ]
        removed = [row for heading, row in prior.items() if heading not in headings]
        if removed:
            retired.setdefault(slug, []).extend(removed)
    for slug in set(old.get('gallery_sections', {})) - set(examples):
        retired.setdefault(slug, []).extend(old['gallery_sections'][slug])
    output['retired_gallery_sections'] = retired
    prior_specimens = {row['id']: row for row in old.get('specimens', [])
                       if row.get('scope') == 'gallery-section'}
    manual_specimens = [row for row in old.get('specimens', [])
                        if row.get('scope') != 'gallery-section']
    generated_specimens = []
    active_specimen_ids = set()
    for slug, source_rows in sorted(examples.items()):
        title = catalog[slug].get('title', slug)
        ref = reference[slug]
        for section in source_rows:
            current = gallery_specimen(slug, title, section, ref)
            previous = prior_specimens.get(current['id'])
            if previous:
                current = merge_record(current, previous, 'source')
            generated_specimens.append(current)
            active_specimen_ids.add(current['id'])
    retired_specimens = deepcopy(old.get('retired_specimens', []))
    for specimen_id, previous in prior_specimens.items():
        if specimen_id not in active_specimen_ids and not any(
            row.get('id') == specimen_id for row in retired_specimens
        ):
            retired_specimens.append(previous)
    output['specimens'] = [*manual_specimens, *generated_specimens]
    output['retired_specimens'] = retired_specimens
    return finish_refresh(old, output)


def upstream_files(directory, roots):
    files = {}
    for name in roots:
        base = directory / name
        if not base.is_dir():
            raise ValueError(f'missing upstream source root: {base}')
        files.update({p.relative_to(directory).as_posix(): digest(p.read_bytes())
                      for p in sorted(base.rglob('*')) if p.is_file()})
    if not files:
        raise ValueError('empty upstream source inventory')
    return files


def refresh_upstream(old, target, baseline):
    for directory, key in ((target, 'target_tag'), (baseline, 'baseline_tag')):
        version = read_json(directory / 'packages/react/package.json')['version']
        if 'v' + version != old['upstream'][key]:
            raise ValueError(f'{directory}: version does not match {old["upstream"][key]}')
    output = deepcopy(old)
    roots = old['source_change_roots']
    before, after = upstream_files(baseline, roots), upstream_files(target, roots)
    prior = {row['path']: row for row in old['source_changes']}
    output['source_changes'] = []
    for name in sorted(before.keys() | after.keys()):
        if before.get(name) == after.get(name):
            continue
        current = {'path': name, 'before_sha256': before.get(name), 'after_sha256': after.get(name)}
        current['change_sha256'] = json_digest(current)
        previous = prior.get(name)
        if previous and 'change_sha256' not in previous:
            previous = {**previous, 'change_sha256': json_digest({k: previous[k] for k in current if k != 'change_sha256'})}
        output['source_changes'].append(merge_record(current, previous, 'change_sha256'))
    active_changes = {row['path'] for row in output['source_changes']}
    output.setdefault('retired_source_changes', []).extend(
        row for name, row in prior.items() if name not in active_changes)
    demos = target / 'apps/docs/src/demos/en'
    if not demos.is_dir():
        raise ValueError('missing upstream English demos')
    output['upstream_demos'] = {}
    retired_demos = deepcopy(old.get('retired_upstream_demos', {}))
    demo_hashes = {}
    for directory in sorted(p for p in demos.iterdir() if p.is_dir()):
        prior = {row['path']: row for row in old['upstream_demos'].get(directory.name, [])}
        rows = []
        for path in sorted(directory.rglob('*.tsx')):
            name = path.relative_to(target).as_posix()
            demo_hashes[name] = digest(path.read_bytes())
            rows.append(merge_record({'path': name, 'sha256': demo_hashes[name]}, prior.get(name), 'sha256'))
        if rows:
            output['upstream_demos'][directory.name] = rows
    if not output['upstream_demos']:
        raise ValueError('empty upstream demo inventory')
    for slug, rows in old['upstream_demos'].items():
        removed = [row for row in rows if row['path'] not in demo_hashes]
        if removed:
            retired_demos.setdefault(slug, []).extend(removed)
    output['retired_upstream_demos'] = retired_demos
    hashes = dict(sorted((after | demo_hashes).items()))
    output['upstream_source_snapshot'] = {'sha256': json_digest(hashes), 'files': hashes}
    return finish_refresh(old, output)


def validate_specimens(inventory, root):
    if inventory.get('verification_sha256') != verification_fingerprint(inventory):
        raise ValueError('inventory verification fingerprint does not match its inputs')
    seen = set()
    for specimen in inventory.get('specimens', []):
        sid = specimen['id']
        if not sid or sid in seen:
            raise ValueError(f'duplicate/empty specimen id: {sid}')
        seen.add(sid)
        if specimen['status'] not in STATUSES:
            raise ValueError(f'{sid}: unknown verification status')
        if specimen['status'] != 'verified':
            continue
        if specimen.get('verified_snapshot') != inventory['verification_sha256']:
            raise ValueError(f'{sid}: verified evidence has no matching source snapshot')
        for surface in ('upstream', 'native', 'wasm', 'tests'):
            evidence = specimen.get('evidence', {}).get(surface, [])
            if not evidence:
                raise ValueError(f'{sid}: missing {surface} evidence')
            for item in evidence:
                path = (root / item['path']).resolve()
                if not path.is_relative_to(root.resolve()) or not path.is_file():
                    raise ValueError(f'{sid}: evidence must be a repository file')
                if digest(path.read_bytes()) != item['sha256']:
                    raise ValueError(f'{sid}: stale {surface} evidence: {item["path"]}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--check', action='store_true')
    mode.add_argument('--refresh', action='store_true')
    parser.add_argument('--upstream', type=Path)
    parser.add_argument('--baseline', type=Path)
    args = parser.parse_args()
    try:
        old = read_json(ROOT / INVENTORY)
        if old['upstream']['target_tag'] != PINNED_RELEASE:
            raise ValueError('inventory target differs from the pinned release')
        if bool(args.upstream) != bool(args.baseline):
            raise ValueError('--upstream and --baseline must be supplied together')
        current = refresh_gallery(old, ROOT)
        if args.upstream:
            current = refresh_upstream(current, args.upstream, args.baseline)
        validate_specimens(current, ROOT)
        if args.check and current != old:
            print('STALE: refresh the interaction inventory; existing verification is not current')
            return 1
        if args.refresh:
            write_json_atomic(ROOT / INVENTORY, current)
        print('Inventory inputs are current; specimen evidence remains a separate review.')
        print(f'Gallery sections: {sum(map(len, current["gallery_sections"].values()))}')
        print(f'Specimens: {len(current["specimens"])}; verified: {sum(s["status"] == "verified" for s in current["specimens"])}')
        return 0
    except (ValueError, KeyError, OSError) as error:
        print(f'INVENTORY ERROR: {error}', file=sys.stderr)
        return 2


if __name__ == '__main__':
    sys.exit(main())
