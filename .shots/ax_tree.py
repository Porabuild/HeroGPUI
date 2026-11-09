#!/usr/bin/env python3
"""Native macOS accessibility-tree audit for the HeroGPUI gallery.

GPUI exposes its window through AccessKit (`accesskit_macos`), which answers the
same NSAccessibility attributes VoiceOver reads. Any AX client activates the
tree, so this driver launches `herogpui-gallery` per component page, dumps the
AX tree with `.shots/ax_dump` (a tiny Swift helper compiled on demand), and
applies naming/state/focus/press rules. It is a *stand-in* for screen-reader
testing: it reads what VoiceOver reads but does not verify speech output,
reading order, or rotor navigation.

macOS-only and stdlib-only. Exits 2 off macOS or without Accessibility (TCC)
permission; exits 1 when any R1-R3 rule fails; 0 otherwise.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import time

from interaction_inventory import source_snapshot

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "web" / "src" / "data" / "catalog.json"
HELPER_SRC = ROOT / ".shots" / "ax_dump.swift"
HELPER_BIN = ROOT / "target" / "ax" / "ax_dump"

# `HEROGPUI_WINDOW_SIZE=WxH` opens the window at that size (parked top-left, no
# centring clamp). A tall window is used so below-fold examples stay inside the
# rendered/AX region. Verified on this build that GPUI does not viewport-cull
# the AX tree: Checkbox and Combo Box report identical node counts at the
# default 1280x820 and at 1280x6000, so tall is coverage insurance, not a
# requirement.
WINDOW_SIZE = "1280x6000"

# ---------------------------------------------------------------------------
# Rules (pure functions over node dicts; unit-tested in test_ax_tree.py)
# ---------------------------------------------------------------------------

# Roles VoiceOver treats as operable controls. AXTab has no dedicated role in
# this tree (tabs surface as AXRadioButton/AXTabButton), but is kept for safety.
INTERACTIVE_ROLES = {
    "AXButton", "AXCheckBox", "AXRadioButton", "AXTextField", "AXTextArea",
    "AXSlider", "AXPopUpButton", "AXComboBox", "AXMenuButton", "AXMenuItem",
    "AXLink", "AXIncrementor", "AXDisclosureTriangle", "AXTab", "AXColorWell",
    "AXDateField",
}
# Subroles that make a node operable even when the role alone would not.
INTERACTIVE_SUBROLES = {"AXTabButton", "AXSwitch", "AXSearchField"}
# macOS window-chrome buttons (traffic lights). VoiceOver names them by subrole,
# so they are system-provided, not app content, and are excluded everywhere.
SYSTEM_SUBROLES = {
    "AXCloseButton", "AXMinimizeButton", "AXFullScreenButton", "AXZoomButton",
    "AXToolbarButton",
}
# Roles whose `placeholder` counts as an accessible name (text inputs only).
TEXT_INPUT_ROLES = {"AXTextField", "AXTextArea"}
TEXT_INPUT_SUBROLES = {"AXSearchField"}
# Gallery chrome that is always present and is not a component under test.
CHROME_TITLES = {
    "GitHub", "Motion on", "Motion off", "Copy", "Expand code", "Collapse code",
}
# Per-example code affordances mark the start of example content; unnamed
# interactive nodes before the first one are the two gallery-header icon buttons.
EXAMPLE_MARK_TITLES = {"Copy", "Expand code", "Collapse code"}

# Named gallery-header buttons (theme picker + light/dark toggle). Unlike
# CHROME_TITLES these are position-gated inside is_chrome to the header region
# before the first example marker, so a component element that happens to share
# the name is not swallowed.
HEADER_BUTTON_NAMES = {"Theme", "Switch to dark mode", "Switch to light mode"}

# Pages whose examples are purely presentational (no operable control). Verified
# against the gallery source and the AX tree (each reports interactive == 0);
# a listed page that does expose an interactive example control fails as a
# stale entry. Color Area is deliberately absent: it is an operable drag
# picker that exposes no AX control, so its empty interactive set is a defect
# the audit must keep flagging, not whitelist.
NON_INTERACTIVE_PAGES = {
    "Typography", "Separator", "Skeleton", "Spinner", "Kbd",
    "Badge", "Chip", "Meter", "Progress Bar", "Avatar", "AvatarGroup",
    "Color Swatch", "Scroll Shadow",
}

# Allowlist of accepted unnamed interactive nodes, keyed by
# (page, role, discriminator) -> reason. Start empty; stale entries fail.
EXPECTED_UNNAMED = {}


def _text(value) -> str:
    return value if isinstance(value, str) else ""


def node_name(node) -> str:
    """The accessible name a screen reader would announce, or ''."""
    for key in ("title", "description", "titleElement"):
        text = _text(node.get(key)).strip()
        if text:
            return text
    if is_text_input(node):
        return _text(node.get("placeholder")).strip()
    return ""


def is_text_input(node) -> bool:
    return node.get("role") in TEXT_INPUT_ROLES or node.get("subrole") in TEXT_INPUT_SUBROLES


def is_interactive(node) -> bool:
    if node.get("subrole") in SYSTEM_SUBROLES:
        return False
    return node.get("role") in INTERACTIVE_ROLES or node.get("subrole") in INTERACTIVE_SUBROLES


def flatten(nodes):
    """Depth-first (document order) list of (node, path_tuple)."""
    out = []

    def walk(node, trail):
        path = tuple(node.get("path") or trail)
        out.append((node, path))
        for i, child in enumerate(node.get("children") or []):
            walk(child, path + (i,))

    for node in nodes:
        walk(node, ())
    return out


def example_cutoff(flat) -> int:
    """Flat index of the first per-example code button; header icons precede it."""
    for i, (node, _path) in enumerate(flat):
        if _text(node.get("title")) in EXAMPLE_MARK_TITLES:
            return i
    return len(flat)


def is_chrome(node, flat_index, cutoff) -> bool:
    """True for gallery chrome (not a component example under test)."""
    if node.get("subrole") in SYSTEM_SUBROLES:
        return True
    if _text(node.get("title")) in CHROME_TITLES:
        return True
    # Gallery-header controls sit before the first example's code button. The
    # theme picker/toggle are named chrome now; unnamed pre-cutoff interactives
    # are chrome too. Every later interactive node is example content.
    name = node_name(node)
    if is_interactive(node) and flat_index < cutoff and (not name or name in HEADER_BUTTON_NAMES):
        return True
    return False


def example_interactive(nodes):
    """Interactive nodes that belong to component examples (R3/R4/R5 pool)."""
    flat = flatten(nodes)
    cutoff = example_cutoff(flat)
    return [
        (node, path)
        for i, (node, path) in enumerate(flat)
        if is_interactive(node) and not is_chrome(node, i, cutoff)
    ]


def unnamed_findings(nodes, page):
    """R1: interactive nodes with no accessible name (system chrome excluded)."""
    findings = []
    for node, path in flatten(nodes):
        if not is_interactive(node):
            continue
        if node_name(node):
            continue
        findings.append({
            "page": page,
            "role": node.get("role"),
            "subrole": node.get("subrole"),
            "value": node.get("value"),
            "frame": node.get("frame"),
            "path": list(path),
        })
    return findings


def _as_number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _range_number(value):
    """Coerce a slider/incrementor AXValue to float; None when not numeric.

    GPUI exposes slider values as strings ("30"); a screen reader announces
    those correctly, so numeric strings count as valid state. bool, None and
    non-numeric strings do not.
    """
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        try:
            return float(value.strip())
        except ValueError:
            return None
    return None


def state_violations(nodes):
    """R2: checkbox values in {0,1,2}; range controls expose a value and both
    bounds, with numeric values inside them. A formatted (aria-valuetext)
    string is valid beside a full range."""
    bad = []
    for node, path in flatten(nodes):
        role = node.get("role")
        if role == "AXCheckBox":
            value = node.get("value")
            if not (isinstance(value, (int, float)) and value in (0, 1, 2)):
                bad.append({"kind": "checkbox-value", "role": role,
                            "subrole": node.get("subrole"), "value": value,
                            "path": list(path)})
        elif role in ("AXSlider", "AXIncrementor"):
            value = node.get("value")
            lo = _range_number(node.get("minValue"))
            hi = _range_number(node.get("maxValue"))
            num = _range_number(value)
            if value is None or (isinstance(value, str) and not value.strip()):
                bad.append({"kind": "range-value", "role": role, "value": value,
                            "path": list(path)})
                continue
            if lo is None or hi is None:
                bad.append({"kind": "range-bounds", "role": role, "value": value,
                            "min": node.get("minValue"), "max": node.get("maxValue"),
                            "path": list(path)})
                continue
            if num is not None:
                if num < lo:
                    bad.append({"kind": "range-below-min", "role": role,
                                "value": value, "min": lo, "path": list(path)})
                if num > hi:
                    bad.append({"kind": "range-above-max", "role": role,
                                "value": value, "max": hi, "path": list(path)})
            elif not isinstance(value, str):
                bad.append({"kind": "range-value", "role": role, "value": value,
                            "path": list(path)})
            # A non-numeric string beside both bounds is `aria-valuetext`
            # ("207°", "35%", ",200.00"): VoiceOver announces it, so it is
            # valid state, not a violation.
    return bad


def unnamed_key(page, finding):
    """Stable allowlist discriminator for an unnamed finding.

    Role+subrole+value alone collides (every unnamed AXButton with a null value
    shares a key), so the index path is included to pin one node.
    """
    return (page, finding.get("role"), finding.get("subrole"),
            json.dumps(finding.get("value"), sort_keys=True),
            json.dumps(finding.get("path")))


def split_allowlist(findings, page):
    """Split R1 findings into (kept, stale-allowlist-keys) for one page."""
    kept = [f for f in findings if unnamed_key(page, f) not in EXPECTED_UNNAMED]
    present = {unnamed_key(page, f) for f in findings}
    stale = [k for k in EXPECTED_UNNAMED if k[0] == page and k not in present]
    return kept, stale


# ---------------------------------------------------------------------------
# Driver (macOS only)
# ---------------------------------------------------------------------------

def ensure_platform():
    if sys.platform != "darwin":
        print("ax_tree: macOS-only (the gallery exposes AX/AccessKit on macOS)",
              file=sys.stderr)
        raise SystemExit(2)


def compile_helper():
    """swiftc -O the helper into target/ax, rebuilt when the source hash moves."""
    digest = hashlib.sha256(HELPER_SRC.read_bytes()).hexdigest()[:16]
    stamp = HELPER_BIN.with_suffix(".sha")
    if HELPER_BIN.is_file() and stamp.is_file() and stamp.read_text().strip() == digest:
        return HELPER_BIN
    HELPER_BIN.parent.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["swiftc", "-O", str(HELPER_SRC), "-o", str(HELPER_BIN)],
        capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"swiftc failed: {result.stderr.strip()}")
    stamp.write_text(digest + "\n")
    return HELPER_BIN


def gallery_binary():
    sys.path.insert(0, str(ROOT / ".shots"))
    import native_capture  # reuse its resolver; do not copy it
    return native_capture.build_gallery(ROOT)


def page_titles():
    data = json.loads(CATALOG.read_text())
    titles = [c["title"] for c in data["components"].values()]
    return titles


def launch(binary, page):
    env = dict(os.environ)
    env.update({
        "HEROGPUI_PAGE": page,
        "HEROGPUI_REDUCE_MOTION": "1",
        "HEROGPUI_THEME": "light",
        "HEROGPUI_WINDOW_SIZE": WINDOW_SIZE,
        # Open without taking focus so the audit does not interrupt the user.
        "HEROGPUI_UNFOCUSED": "1",
    })
    log = ROOT / "target" / "ax" / f"run-{page.lower().replace(' ', '-')}.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    out = open(log, "wb")
    proc = subprocess.Popen([str(binary)], env=env,
                            stdout=out, stderr=subprocess.STDOUT)
    proc._ax_log = out  # keep the handle alive for kill/cleanup
    return proc


def ax_json(helper, *args):
    result = subprocess.run([str(helper), *map(str, args)],
                            capture_output=True, text=True, timeout=120)
    if result.returncode == 3:
        print("ax_tree: Accessibility permission not granted "
              "(grant it to the terminal in System Settings > Privacy & "
              "Security > Accessibility)", file=sys.stderr)
        raise SystemExit(2)
    if result.returncode != 0:
        raise RuntimeError(f"ax_dump {' '.join(map(str, args))} failed "
                           f"({result.returncode}): {result.stderr.strip()}")
    return json.loads(result.stdout)


def dump_tree(helper, pid, settle_ms=8000):
    data = ax_json(helper, "tree", pid, "--settle-ms", settle_ms)
    return data.get("nodes", []), data.get("node_count", 0), data.get("settled", True)


def path_arg(path):
    return ",".join(str(i) for i in path)


def audit_page(helper, binary, page, dump_dir):
    """Launch one page, dump its AX tree, run R1-R5, kill the process."""
    proc = launch(binary, page)
    record = {"page": page, "nodes": 0, "interactive": 0, "unnamed": [],
              "state_violations": [], "focus": {"tried": 0, "landed": 0},
              "press": None, "result": "pass", "detail": None}
    try:
        # Wait for the window/AccessKit tree to settle.
        nodes, count, settled = [], 0, False
        deadline = time.time() + 30
        last_err = None
        while time.time() < deadline:
            try:
                nodes, count, settled = dump_tree(helper, proc.pid, settle_ms=8000)
                last_err = None
            except Exception as exc:
                last_err = exc
                nodes, count, settled = [], 0, False
            if count > 8:
                break
            time.sleep(0.5)
        if count <= 8 and last_err is not None:
            raise last_err
        record["nodes"] = count
        interactive = example_interactive(nodes)
        record["interactive"] = len(interactive)

        # R1 + allowlist
        findings = unnamed_findings(nodes, page)
        kept, stale = split_allowlist(findings, page)
        record["unnamed"] = kept
        if stale:
            record["result"] = "fail"
            record["detail"] = f"stale allowlist entries: {stale}"

        # R2
        record["state_violations"] = state_violations(nodes)

        # R3
        if page in NON_INTERACTIVE_PAGES:
            if interactive:
                record["result"] = "fail"
                record["detail"] = ("listed NON_INTERACTIVE but exposes "
                                    f"{len(interactive)} interactive node(s)")
        elif not interactive:
            record["result"] = "fail"
            record["detail"] = "no interactive example nodes"

        # Any R1/R2 failure fails the page.
        if kept or record["state_violations"]:
            record["result"] = "fail"

        if dump_dir:
            dump_dir.mkdir(parents=True, exist_ok=True)
            safe = page.lower().replace(" ", "-")
            (dump_dir / f"{safe}.json").write_text(json.dumps(nodes, indent=1))

        # R4 focus (report-only): up to 10 example-interactive nodes.
        for node, path in interactive[:10]:
            record["focus"]["tried"] += 1
            try:
                focused = ax_json(helper, "focus", proc.pid, path_arg(path))
            except Exception:
                continue
            landed = focused.get("role") == node.get("role")
            # Tiebreak on a shared name/value when both sides carry one.
            if landed and node_name(node) and node_name(focused) != node_name(node):
                landed = False
            if landed:
                record["focus"]["landed"] += 1

        # R5 press (report-only) on the three control-heavy pages.
        if page in ("Checkbox", "Switch", "Button"):
            record["press"] = press_probe(helper, proc, nodes, interactive)
    finally:
        try:
            proc.kill()
        except Exception:
            pass
        try:
            proc.wait(timeout=5)
        except Exception:
            pass
        try:
            proc._ax_log.close()
        except Exception:
            pass
    return record


def press_probe(helper, proc, nodes, interactive):
    """Press the first enabled example control; report value change/acceptance."""
    target = None
    for node, path in interactive:
        if node.get("enabled") is False:
            continue
        if node.get("role") in ("AXCheckBox", "AXButton") or \
           node.get("subrole") in ("AXSwitch",):
            target = (node, path)
            break
    if target is None:
        return {"skipped": "no enabled example control"}
    node, path = target
    before = node.get("value")
    try:
        after = ax_json(helper, "press", proc.pid, path_arg(path))
    except Exception as exc:
        return {"error": str(exc)}
    out = {"role": node.get("role"), "subrole": node.get("subrole"),
           "before": before, "after": after.get("value"),
           "actions": after.get("actions")}
    if node.get("role") == "AXButton":
        out["accepted"] = "AXPress" in (after.get("actions") or []) or after.get("role") == "AXButton"
    else:
        out["value_changed"] = after.get("value") != before
    return out


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", help="gallery binary path "
                        "(default: cargo build -p herogpui-gallery)")
    parser.add_argument("--page", action="append", default=[],
                        help="component page title (repeatable; default: all)")
    parser.add_argument("--record", help="write the JSON run record here")
    parser.add_argument("--dump-dir", help="write each page's raw AX tree JSON")
    parser.add_argument("--settle-ms", type=int, default=8000,
                        help="per-page AccessKit settle budget")
    args = parser.parse_args(argv)

    ensure_platform()
    helper = compile_helper()
    binary = Path(args.binary) if args.binary else gallery_binary()
    # The snapshot names the exact tree the binary was built from; a caller-
    # supplied --binary is not this tree, so it gets null instead.
    source_sha = (source_snapshot(ROOT)["sha256"] if args.binary is None else None)
    titles = args.page or page_titles()
    dump_dir = Path(args.dump_dir) if args.dump_dir else None

    platform_str = f"macOS {platform.mac_ver()[0]} {platform.machine()}"
    record = {
        "schema_version": 1,
        "kind": "native-a11y",
        "recorded_on": time.strftime("%Y-%m-%d"),
        "native_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "source_sha256": source_sha,
        "platform": platform_str,
        "window_size": WINDOW_SIZE,
        "pages": [],
        "summary": {},
    }

    for page in titles:
        try:
            page_record = audit_page(helper, binary, page, dump_dir)
        except KeyboardInterrupt:
            raise
        except Exception as exc:
            page_record = {"page": page, "result": "fail", "detail": str(exc)}
        record["pages"].append(page_record)
        # Short summary: failing pages only.
        if page_record.get("result") != "pass":
            print(f"FAIL {page}: {page_record.get('detail') or ''} "
                  f"(unnamed={len(page_record.get('unnamed', []))}, "
                  f"state={len(page_record.get('state_violations', []))})")

    failed = [p for p in record["pages"] if p.get("result") != "pass"]
    record["summary"] = {
        "pages": len(record["pages"]),
        "failed_pages": len(failed),
        "unnamed": sum(len(p.get("unnamed", [])) for p in record["pages"]),
        "state_violations": sum(len(p.get("state_violations", [])) for p in record["pages"]),
    }
    if args.record:
        Path(args.record).parent.mkdir(parents=True, exist_ok=True)
        Path(args.record).write_text(json.dumps(record, indent=1))
    print(f"ax-tree: {len(record['pages']) - len(failed)}/{len(record['pages'])} "
          f"pages clean, {len(failed)} failed, "
          f"{record['summary']['unnamed']} unnamed, "
          f"{record['summary']['state_violations']} state violations")
    return 1 if failed else 0


if __name__ == "__main__":
    signal.signal(signal.SIGINT, lambda *_: sys.exit(130))
    raise SystemExit(main())
