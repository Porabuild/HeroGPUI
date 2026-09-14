#!/usr/bin/env python3
"""Materialize, record and verify the renderer forks of the published GPUI packages.

The repository carries the *patches*, never the patched sources. Five published
`gpui-pre*` packages are forked (`docs/upstream/gpui-rounded-content-mask.md`
and `docs/upstream/gpui-web-scroll-and-ime.md` say why), and each fork is stored
as a single unified patch under `docs/upstream/patches/`. The build materializes
the patched tree into the gitignored `.vendor/` root, which the workspace's
`[patch.crates-io]` table points at.

    python3 .shots/gpui_patches.py --materialize   # bootstrap: required before *any* cargo command
    python3 .shots/gpui_patches.py --check         # patch still reproduces `.vendor/` exactly
    python3 .shots/gpui_patches.py --write         # re-record a patch after editing `.vendor/`
    python3 .shots/gpui_patches.py --self-test     # the recorder's own regressions

A missing `.vendor/` tree is not a soft failure: cargo aborts at manifest load
with "failed to load source for dependency" because a `[patch.crates-io]` path
does not exist. That is why `--materialize` runs first in every CI job that
touches cargo, and why a fresh clone must run it before `cargo`, `rustup`
component tooling, or rust-analyzer.
"""

import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PATCH_DIR = ROOT / "docs/upstream/patches"
PACKAGES = (
    "gpui-pre",
    "gpui-pre-apple",
    "gpui-pre-web",
    "gpui-pre-wgpu",
    "gpui-pre-windows",
)
# Written into each materialized tree so a warm run can decide in milliseconds
# whether the tree is already the patch applied to the pinned published source.
STAMP = ".herogpui-materialized.json"
# Cargo uses the workspace lockfile. These registry/cache files are not fork
# code: `Cargo.toml.orig` is the packager's pre-normalization backup and
# `.cargo-ok`/`.cargo-checksum.json` are cargo's own unpack bookkeeping, so
# none of them belongs in a materialized tree or in a recorded patch.
IGNORED = {".cargo-checksum.json", ".cargo-ok", "Cargo.lock", "Cargo.toml.orig", STAMP}


def digest(payload):
    return hashlib.sha256(payload).hexdigest()


def files(root):
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in root.rglob("*")
        if path.is_file()
        and path.name not in IGNORED
        and not {"target", ".git"}.intersection(path.relative_to(root).parts)
    }


def tree_digest(root):
    return digest(
        json.dumps(
            {name: digest(content) for name, content in files(root).items()},
            sort_keys=True,
        ).encode()
    )


def patch_text(base, fork, package):
    original, changed = files(base), files(fork)
    output = []
    for name in sorted(original.keys() | changed.keys()):
        if original.get(name) == changed.get(name):
            continue
        before = original.get(name, b"").decode("utf-8").splitlines(keepends=True)
        after = changed.get(name, b"").decode("utf-8").splitlines(keepends=True)
        diff = difflib.unified_diff(
            before, after,
            fromfile=f"a/{package}/{name}" if name in original else "/dev/null",
            tofile=f"b/{package}/{name}" if name in changed else "/dev/null",
        )
        for line in diff:
            output.append(line if line.endswith("\n") else line + "\n\\ No newline at end of file\n")
    return "".join(output)


def parse_patch(body, package):
    """Split a `patch_text` unified diff into `(path, deleted, created, hunks)` per file.

    Only the dialect `patch_text` emits is accepted -- no index lines, no
    renames, no binary hunks, one `---`/`+++` pair per file. Anything else is a
    hand-edited patch, which is exactly what this refuses to guess at.
    """
    lines = body.splitlines(keepends=True)
    files_, index = [], 0
    while index < len(lines):
        line = lines[index]
        if not line.startswith("--- ") or index + 1 >= len(lines):
            raise ValueError(f"unexpected line {index + 1} in the patch for {package}: {line!r}")
        source, target = line[4:].rstrip("\n"), lines[index + 1][4:].rstrip("\n")
        if not lines[index + 1].startswith("+++ "):
            raise ValueError(f"patch for {package} has a `---` line with no `+++` line")
        index += 2
        deleted, created = target == "/dev/null", source == "/dev/null"
        named = source if deleted else target
        prefix = "a/" if deleted else "b/"
        if not named.startswith(f"{prefix}{package}/") or deleted and created:
            raise ValueError(f"patch for {package} names a foreign file: {named!r}")
        path, hunks = named[len(prefix) + len(package) + 1:], []
        while index < len(lines) and lines[index].startswith("@@ "):
            header = lines[index].split(" ")
            old_start, old_count = hunk_range(header[1])
            _, new_count = hunk_range(header[2])
            index, body_lines = index + 1, []
            # The two counts say exactly how many body lines this hunk has, so
            # the scan never has to guess where it ends -- a `-` line that opens
            # the next file's `--- a/...` header cannot be mistaken for a removal.
            seen_old = seen_new = 0
            while seen_old < old_count or seen_new < new_count:
                if index >= len(lines):
                    raise ValueError(f"patch for {package} ends inside a hunk for {path}")
                entry = lines[index]
                index += 1
                if entry.startswith("\\"):
                    body_lines[-1] = strip_final_newline(body_lines[-1])
                    continue
                if entry == "\n":
                    entry = " \n"  # an empty context line, if an editor ate the space
                tag, text = entry[0], entry[1:]
                seen_old += tag in " -"
                seen_new += tag in " +"
                if tag not in " -+":
                    raise ValueError(f"patch for {package} has an unreadable hunk line: {entry!r}")
                body_lines.append((tag, text))
            # A `\ No newline at end of file` marker belongs to the line above
            # it, whose recorded text carries a newline the file does not have.
            if index < len(lines) and lines[index].startswith("\\"):
                body_lines[-1] = strip_final_newline(body_lines[-1])
                index += 1
            hunks.append((old_start, old_count, body_lines))
        if not hunks:
            raise ValueError(f"patch for {package} has no hunk for {path}")
        files_.append((path, deleted, created, hunks))
    return files_


def hunk_range(field):
    """`-12,3` or `+12` into `(start, count)`; a bare number means one line."""
    start, _, count = field[1:].partition(",")
    return int(start), int(count) if count else 1


def strip_final_newline(entry):
    tag, text = entry
    return tag, text[:-1] if text.endswith("\n") else text


def apply_hunks(original, hunks, path):
    """Replay `hunks` against `original`, requiring an exact positional match.

    There is no search and no fallback, so there is nothing for a fuzz factor or
    a line offset to hide in: every context and removed line has to be the line
    the patch says it is, or the whole application fails.
    """
    result, cursor = [], 0
    for start, count, body in hunks:
        # difflib writes a zero-length range as the line *before* the insertion
        # point, so the 0-based index is `start` there and `start - 1` otherwise.
        index = start if count == 0 else start - 1
        if index < cursor or index > len(original):
            raise ValueError(f"hunk at line {start} of {path} is out of order or past the file end")
        result.extend(original[cursor:index])
        cursor = index
        for tag, text in body:
            if tag == "+":
                result.append(text)
                continue
            if cursor >= len(original) or original[cursor] != text:
                found = original[cursor] if cursor < len(original) else "<end of file>"
                raise ValueError(
                    f"{path}: line {cursor + 1} is {found!r}, but the patch expects {text!r}"
                )
            if tag == " ":
                result.append(text)
            cursor += 1
    result.extend(original[cursor:])
    return result


def apply_patch(tree, patch, package):
    """Apply `patch` inside `tree`, rejecting anything short of an exact application.

    Deliberately not a `patch(1)` subprocess. This runs on all three CI hosts
    now that materialization precedes every cargo command, and the GNU/BSD
    implementations disagree about deletions, whitespace and exit codes -- BSD
    leaves a zero-length file where GNU unlinks it, and "applied with fuzz" is
    reported on stdout rather than in the exit status, so strictness depended on
    sniffing English prose. An in-process applier is the same behaviour on every
    host, and `--check` proves it round-trips against the recorded diff.
    """
    body = patch.read_text()
    if not body:
        return
    for path, deleted, created, hunks in parse_patch(body, package):
        target = tree / path
        # A header says whether the published package has this file, so a
        # rebase that adds or removes one upstream fails here rather than
        # quietly producing a tree the patch no longer describes.
        if target.is_file() == created:
            state = "already exists" if created else "is absent"
            raise ValueError(f"patch does not apply exactly: {patch}\n{path} {state} in the published source")
        if deleted:
            target.unlink()
            continue
        # Bytes, not `read_text`/`write_text`: those translate newlines, and a
        # published source file with CRLF endings has to survive the round trip
        # byte for byte or the recorded patch stops matching it.
        original = [] if created else target.read_bytes().decode("utf-8").splitlines(keepends=True)
        try:
            updated = apply_hunks(original, hunks, path)
        except ValueError as error:
            raise ValueError(f"patch does not apply exactly: {patch}\n{error}") from error
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes("".join(updated).encode("utf-8"))


def check_patch(base, fork, patch, package):
    expected = patch_text(base, fork, package)
    if not patch.is_file():
        raise ValueError(f"missing patch: {patch}")
    if patch.read_text() != expected:
        raise ValueError(f"stale patch: {patch}; regenerate with --write")
    with tempfile.TemporaryDirectory(prefix="herogpui-patch-") as temporary:
        replay = Path(temporary) / "source"
        copy_pristine(base, replay)
        apply_patch(replay, patch, package)
        if files(replay) != files(fork):
            raise ValueError(f"patch replay differs from the materialized tree: {package}")


def copy_pristine(base, destination):
    """Copy the published source into a writable tree, minus cargo's bookkeeping.

    Registry unpacks are read-only, so the bytes are rewritten rather than
    `copytree`d; `patch` has to be able to edit what lands here. Every file in
    the published packages is mode 644 (verified against all five), so no
    executable bit is carried across.
    """
    destination.mkdir(parents=True)
    for name, content in files(base).items():
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        path.chmod(0o644)


def registry_package(name, version):
    """The pristine published source for the exact pin, from cargo's own cache.

    Nothing is downloaded by this script, so there is no unverified download to
    trust: the tree returned here is one cargo unpacked itself after checking
    the `.crate` tarball against the registry index checksum. On a cold cache
    `cargo info` is what fetches it, outside this workspace so the patched
    `[patch.crates-io]` entries cannot redirect the fetch to the fork.
    """
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    candidates = sorted((cargo_home / "registry/src").glob(f"*/{name}-{version}"))
    candidates = [path for path in candidates if (path / "Cargo.toml").is_file()]
    if not candidates:
        # A patched workspace never fetches the original package. Resolve it
        # outside the workspace so Cargo retrieves the published source.
        with tempfile.TemporaryDirectory(prefix="herogpui-upstream-") as temporary:
            result = subprocess.run(
                ["cargo", "info", f"{name}@{version}", "--registry", "crates-io"],
                cwd=temporary, capture_output=True, text=True,
            )
        if result.returncode:
            raise ValueError(f"cannot fetch published {name} {version}: {result.stderr}")
        candidates = sorted((cargo_home / "registry/src").glob(f"*/{name}-{version}"))
        if not candidates:
            raise ValueError(f"published {name} {version} source missing after cargo info")
    source = candidates[0]
    package = tomllib.loads((source / "Cargo.toml").read_text())["package"]
    if package["name"] != name or package["version"] != version:
        raise ValueError(f"registry package identity mismatch: {source}")
    return source


def write_stamp(tree, name, version, patch_sha256, upstream_sha256):
    (tree / STAMP).write_text(json.dumps({
        "note": "Generated by .shots/gpui_patches.py --materialize. Not source; do not edit.",
        "package": name,
        "version": version,
        "patch_sha256": patch_sha256,
        "upstream_sha256": upstream_sha256,
        "tree_sha256": tree_digest(tree),
    }, indent=2) + "\n")


def materialize(name, version, destination, patch, package, resolve_base=registry_package):
    """Make `destination` the pinned published source with `patch` applied.

    Idempotent and self-healing: a tree whose stamp still matches both the patch
    and its own contents is left alone, and anything else is rebuilt from
    scratch under a `.partial` name and swapped in, so an interrupted run never
    leaves a half-patched tree behind for cargo to compile.
    """
    if not patch.is_file():
        raise ValueError(f"missing patch: {patch}")
    fingerprint = digest(patch.read_bytes())
    stamp = destination / STAMP
    if stamp.is_file():
        try:
            record = json.loads(stamp.read_text())
        except (json.JSONDecodeError, UnicodeDecodeError):
            record = {}
        if (record.get("package") == name
                and record.get("version") == version
                and record.get("patch_sha256") == fingerprint
                and record.get("tree_sha256") == tree_digest(destination)):
            return "already materialized"
    base = resolve_base(name, version)
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging = destination.parent / f".{destination.name}.partial"
    shutil.rmtree(staging, ignore_errors=True)
    try:
        copy_pristine(base, staging)
        apply_patch(staging, patch, package)
        write_stamp(staging, name, version, fingerprint, tree_digest(base))
        shutil.rmtree(destination, ignore_errors=True)
        staging.replace(destination)
    finally:
        shutil.rmtree(staging, ignore_errors=True)
    return "materialized"


def workspace_pin():
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    version = manifest["workspace"]["dependencies"]["gpui"]["version"]
    if not version.startswith("="):
        raise ValueError("the workspace GPUI dependency must have an exact version pin")
    return manifest, version[1:]


def patched_path(manifest, name, version):
    """Where `[patch.crates-io]` says this package lives, asserted to the convention.

    The path, the patch filename and the pin are one fact written three times,
    so this refuses any `[patch.crates-io]` entry that does not spell the
    materialization root the way `--materialize` writes it.
    """
    configured = manifest["patch"]["crates-io"][name]["path"]
    expected = f".vendor/{name}-{version}"
    if configured != expected:
        raise ValueError(
            f"[patch.crates-io].{name} points at {configured!r}, not {expected!r}"
        )
    return ROOT / configured


def require_materialized(destination, name, version):
    if not (destination / "Cargo.toml").is_file():
        raise ValueError(
            f"{destination.relative_to(ROOT).as_posix()} is not materialized; "
            "run `python3 .shots/gpui_patches.py --materialize` first"
        )
    package = tomllib.loads((destination / "Cargo.toml").read_text())["package"]
    if package["name"] != name or package["version"] != version:
        raise ValueError(
            f"{destination.relative_to(ROOT).as_posix()}: materialized tree is "
            f"{package['name']} {package['version']}, not the pinned {name} {version}"
        )


def self_test():
    with tempfile.TemporaryDirectory(prefix="herogpui-patch-test-") as temporary:
        root = Path(temporary)
        base, fork, patch = root / "base", root / "fork", root / "test.patch"
        base.mkdir()
        (base / "changed.rs").write_text("old\n")
        (base / "deleted.rs").write_text("removed\n")
        shutil.copytree(base, fork)
        (fork / "changed.rs").write_text("new\n")
        (fork / "deleted.rs").unlink()
        (fork / "added.rs").write_text("no trailing newline")
        patch.write_text(patch_text(base, fork, "example-0.3.3"))
        check_patch(base, fork, patch, "example-0.3.3")

        # Materialization reproduces the fork from the patch alone, and says so
        # cheaply on a second run instead of rebuilding the tree.
        vendor = root / "vendor" / "example-0.3.3"
        resolve = lambda *_: base  # noqa: E731 - one-line stub for the registry lookup
        assert materialize("example", "0.3.3", vendor, patch, "example-0.3.3", resolve) == "materialized"
        assert files(vendor) == files(fork), "materialized tree differs from the fork"
        assert json.loads((vendor / STAMP).read_text())["package"] == "example"
        assert materialize("example", "0.3.3", vendor, patch, "example-0.3.3", resolve) == "already materialized"

        # A hand-edited or half-written tree is rebuilt, not trusted.
        (vendor / "changed.rs").write_text("tampered\n")
        assert materialize("example", "0.3.3", vendor, patch, "example-0.3.3", resolve) == "materialized"
        assert files(vendor) == files(fork), "tampered tree was not rebuilt"

        # ... and `--check` reports that tampering rather than repairing it.
        (vendor / "changed.rs").write_text("tampered\n")
        try:
            check_patch(base, vendor, patch, "example-0.3.3")
        except ValueError as error:
            assert "stale patch" in str(error)
        else:
            raise AssertionError("a tampered materialized tree was accepted")

        patch.write_text("")
        try:
            check_patch(base, fork, patch, "example-0.3.3")
        except ValueError as error:
            assert "stale patch" in str(error)
        else:
            raise AssertionError("an incomplete patch was accepted")
        patch.unlink()
        try:
            check_patch(base, fork, patch, "example-0.3.3")
        except ValueError as error:
            assert "missing patch" in str(error)
        else:
            raise AssertionError("a missing patch was accepted")
        try:
            materialize("example", "0.3.3", vendor, patch, "example-0.3.3", resolve)
        except ValueError as error:
            assert "missing patch" in str(error)
        else:
            raise AssertionError("materialization accepted a missing patch")
    print("GPUI patch self-test: materialize, replay, tamper, stale and missing patches all handled")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--materialize", action="store_true",
                      help="build .vendor/ from the published sources plus the recorded patches")
    mode.add_argument("--write", action="store_true", help="regenerate versioned patches from .vendor/")
    mode.add_argument("--check", action="store_true",
                      help="verify each patch reproduces its materialized tree exactly (default)")
    mode.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    manifest, version = workspace_pin()
    for name in PACKAGES:
        destination = patched_path(manifest, name, version)
        package = f"{name}-{version}"
        patch = PATCH_DIR / f"{package}.patch"
        if args.materialize:
            state = materialize(name, version, destination, patch, package)
            print(f"{package}: {state} in {destination.relative_to(ROOT).as_posix()}")
            continue
        require_materialized(destination, name, version)
        base = registry_package(name, version)
        if args.write:
            patch.parent.mkdir(parents=True, exist_ok=True)
            patch.write_text(patch_text(base, destination, package))
            write_stamp(destination, name, version, digest(patch.read_bytes()), tree_digest(base))
        check_patch(base, destination, patch, package)
        print(f"{package}: patch applies exactly to the published source and reproduces .vendor/")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError) as error:
        raise SystemExit(f"GPUI PATCH ERROR: {error}") from error
