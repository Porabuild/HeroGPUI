"""Generate the maintainer coverage report from the parity inventory.

    python .shots/coverage_report.py --refresh
    python .shots/coverage_report.py --check

The report is a work-queue summary, not a parity verdict.  It deliberately
keeps unreviewed and unobserved cases visible and derives every count from the
checked-in interaction inventory.  Generated JSON and Markdown are checked in
so a handoff can inspect the same snapshot without running a Python command.
"""
from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import json
import os
from pathlib import Path
import sys
import tempfile

from interaction_inventory import (
    INVENTORY,
    ROOT,
    STATUSES,
    read_json,
    validate_specimens,
    verification_fingerprint,
    write_json_atomic,
)


REPORT = Path("docs/parity/coverage-report.json")
MARKDOWN = Path("docs/parity/coverage-report.md")
SURFACES = ("upstream", "native", "wasm", "tests")
UNOBSERVED_STATUSES = ("unreviewed", "specified", "implemented-unverified")


def _status_counts(specimens):
    counts = Counter(row.get("status") for row in specimens)
    return {status: counts.get(status, 0) for status in sorted(STATUSES)}


def _scope_counts(specimens):
    counts = Counter(row.get("scope", "unspecified") for row in specimens)
    return {scope: counts[scope] for scope in sorted(counts)}


def _evidence_counts(specimens):
    """Count specimens with evidence on each surface, without judging it."""
    output = {}
    for surface in SURFACES:
        present = sum(bool(row.get("evidence", {}).get(surface, [])) for row in specimens)
        output[surface] = {
            "specimens_with_evidence": present,
            "specimens_missing_evidence": len(specimens) - present,
        }
    complete = sum(
        all(row.get("evidence", {}).get(surface, []) for surface in SURFACES)
        for row in specimens
    )
    output["all_surfaces"] = {
        "specimens_with_evidence": complete,
        "specimens_missing_evidence": len(specimens) - complete,
    }
    return output


def _last_evidence_commit(inventory):
    """Return an explicitly recorded evidence commit, never an upstream pin.

    The inventory is often reviewed in a dirty worktree.  Falling back to
    ``git HEAD`` would make a generated report claim that an uncommitted change
    is evidence, so the honest value is ``None`` until a reviewer records one.
    Per-specimen aliases are accepted for forward compatibility with expanded
    capture records.
    """
    candidates = []
    for key in ("last_evidence_commit", "evidence_commit", "implementation_commit"):
        value = inventory.get(key)
        if isinstance(value, str) and value:
            candidates.append(value)
    for specimen in inventory.get("specimens", []):
        for key in ("evidence_commit", "implementation_commit", "review_commit"):
            value = specimen.get(key)
            if isinstance(value, str) and value:
                candidates.append(value)
    return sorted(set(candidates))[-1] if candidates else None


def build_report(inventory, root=ROOT):
    """Build a deterministic report from one inventory value."""
    if inventory.get("verification_sha256") != verification_fingerprint(inventory):
        raise ValueError("inventory verification fingerprint does not match its inputs")
    # Reuse the inventory's strict duplicate/status/evidence checks.  This also
    # prevents a report from silently accepting stale verified captures.
    validate_specimens(inventory, root)

    specimens = sorted(inventory.get("specimens", []), key=lambda row: row["id"])
    statuses = _status_counts(specimens)
    unobserved = sum(statuses.get(status, 0) for status in UNOBSERVED_STATUSES)
    outstanding = len(specimens) - statuses.get("verified", 0)

    by_component = defaultdict(list)
    for specimen in specimens:
        by_component[specimen.get("component", "Unassigned")].append(specimen)
    components = []
    for component in sorted(by_component):
        rows = by_component[component]
        component_statuses = _status_counts(rows)
        unresolved = [
            row["id"] for row in rows if row.get("status") != "verified"
        ]
        components.append({
            "component": component,
            "specimens": len(rows),
            "statuses": component_statuses,
            "verified": component_statuses["verified"],
            "outstanding": len(unresolved),
            "unresolved_ids": unresolved,
        })

    unresolved = [
        {
            "id": row["id"],
            "component": row.get("component", "Unassigned"),
            "scope": row.get("scope", "unspecified"),
            "status": row.get("status"),
            "state": row.get("state"),
        }
        for row in specimens if row.get("status") != "verified"
    ]

    return {
        "schema_version": 1,
        "target": inventory.get("upstream", {}).get("target_tag"),
        "target_commit": inventory.get("upstream", {}).get("target_commit"),
        "inventory_recorded_on": inventory.get("recorded_on"),
        "inventory_verification_sha256": inventory["verification_sha256"],
        "source_snapshot_sha256": inventory.get("source_snapshot", {}).get("sha256"),
        "last_evidence_commit": _last_evidence_commit(inventory),
        "last_evidence_commit_recorded": _last_evidence_commit(inventory) is not None,
        "totals": {
            "specimens": len(specimens),
            "verified": statuses["verified"],
            "outstanding": outstanding,
            "unobserved": unobserved,
            "components": len(components),
            "gallery_sections": sum(
                len(rows) for rows in inventory.get("gallery_sections", {}).values()
            ),
        },
        "status_counts": statuses,
        "scope_counts": _scope_counts(specimens),
        "evidence_counts": _evidence_counts(specimens),
        "unobserved_statuses": list(UNOBSERVED_STATUSES),
        "measured_gaps": [row for row in unresolved if row["status"] == "measured-gap"],
        "intentional_deviations": [
            row for row in unresolved if row["status"] == "intentional-deviation"
        ],
        "platform_limits": [
            row for row in unresolved if row["status"] == "platform-limited"
        ],
        "unresolved": unresolved,
        "components": components,
        "notes": [
            "This report summarizes the inventory; it does not establish visual or interaction parity.",
            "Gallery-section seeds must be expanded into concrete variant/state specimens before verification.",
            "A verified specimen requires current upstream, native, WASM and test evidence.",
            "last_evidence_commit is null until a review records a commit; the upstream target commit is not evidence.",
        ],
    }


def _table(rows, headers):
    lines = ["| " + " | ".join(headers) + " |", "| " + " | ".join("---" for _ in headers) + " |"]
    lines.extend("| " + " | ".join(str(row.get(header, "")) for header in headers) + " |" for row in rows)
    return "\n".join(lines)


def render_markdown(report):
    totals = report["totals"]
    status_rows = [
        {"Status": status, "Specimens": count}
        for status, count in report["status_counts"].items()
    ]
    component_rows = [
        {
            "Component": row["component"],
            "Specimens": row["specimens"],
            "Verified": row["verified"],
            "Outstanding": row["outstanding"],
        }
        for row in report["components"]
    ]
    evidence_rows = [
        {
            "Surface": surface,
            "With evidence": values["specimens_with_evidence"],
            "Missing evidence": values["specimens_missing_evidence"],
        }
        for surface, values in report["evidence_counts"].items()
    ]
    commit = report["last_evidence_commit"] or "(none recorded)"
    lines = [
        "<!-- Generated by .shots/coverage_report.py; do not edit by hand. -->",
        "# HeroGPUI parity coverage report",
        "",
        f"Target: `{report['target']}` (`{report['target_commit']}`)",
        f"Inventory date: `{report['inventory_recorded_on']}`",
        f"Inventory verification: `{report['inventory_verification_sha256']}`",
        f"Last evidence commit: `{commit}`",
        "",
        "This is a maintainer work-queue summary. It does not turn gallery section headings, static audits, or successful builds into parity evidence.",
        "",
        "## Totals",
        "",
        _table([
            {"Metric": "Specimens", "Count": totals["specimens"]},
            {"Metric": "Verified", "Count": totals["verified"]},
            {"Metric": "Outstanding", "Count": totals["outstanding"]},
            {"Metric": "Unobserved/unreviewed", "Count": totals["unobserved"]},
            {"Metric": "Components", "Count": totals["components"]},
            {"Metric": "Gallery sections", "Count": totals["gallery_sections"]},
        ], ["Metric", "Count"]),
        "",
        "## Status distribution",
        "",
        _table(status_rows, ["Status", "Specimens"]),
        "",
        "## Evidence surfaces",
        "",
        _table(evidence_rows, ["Surface", "With evidence", "Missing evidence"]),
        "",
        "## Component queue",
        "",
        _table(component_rows, ["Component", "Specimens", "Verified", "Outstanding"]),
        "",
        "## Explicit gap queues",
        "",
        f"- Measured gaps: **{len(report['measured_gaps'])}**",
        f"- Intentional deviations: **{len(report['intentional_deviations'])}**",
        f"- Platform limits: **{len(report['platform_limits'])}**",
        f"- Unresolved specimen records: **{len(report['unresolved'])}**",
        "",
        "The JSON file contains the complete unresolved id list and per-component detail. Add a concrete evidence record before changing a specimen to `verified`; record intentional deviations and platform limits with user-facing notes.",
        "",
    ]
    return "\n".join(lines)


def write_text_atomic(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                         prefix="." + path.name + ".", delete=False) as handle:
            temporary = Path(handle.name)
            handle.write(text)
            handle.flush()
            os.fsync(handle.fileno())
        temporary.replace(path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="fail if generated files are stale")
    mode.add_argument("--refresh", action="store_true", help="rewrite generated files")
    parser.add_argument("--output", type=Path, default=REPORT)
    parser.add_argument("--markdown", type=Path, default=MARKDOWN)
    args = parser.parse_args(argv)
    try:
        inventory = read_json(ROOT / INVENTORY)
        report = build_report(inventory, ROOT)
        data = json.dumps(report, indent=2, ensure_ascii=False) + "\n"
        markdown = render_markdown(report)
        output = ROOT / args.output
        markdown_path = ROOT / args.markdown
        if args.check:
            stale = []
            if not output.is_file() or output.read_text(encoding="utf-8") != data:
                stale.append(str(args.output))
            if not markdown_path.is_file() or markdown_path.read_text(encoding="utf-8") != markdown:
                stale.append(str(args.markdown))
            if stale:
                print("STALE: refresh coverage report: " + ", ".join(stale))
                return 1
        else:
            write_json_atomic(output, report)
            write_text_atomic(markdown_path, markdown)
        print(f"Coverage report: {report['totals']['specimens']} specimens; {report['totals']['verified']} verified")
        print(f"Unobserved: {report['totals']['unobserved']}; outstanding: {report['totals']['outstanding']}")
        print(f"Last evidence commit: {report['last_evidence_commit'] or '(none recorded)'}")
        return 0
    except (ValueError, KeyError, OSError, json.JSONDecodeError) as error:
        print(f"COVERAGE REPORT ERROR: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
