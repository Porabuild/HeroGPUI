#!/usr/bin/env python3
"""Record and verify renderer forks against their exact published packages."""

import argparse
import difflib
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ("gpui-pre", "gpui-pre-apple", "gpui-pre-wgpu", "gpui-pre-windows")
# Cargo uses the workspace lockfile. These registry/cache files are not fork
# code: `Cargo.toml.orig` is the packager's pre-normalization backup and is
# gitignored, so it exists only in an unpacked registry copy, never in the fork.
IGNORED = {".cargo-checksum.json", ".cargo-ok", "Cargo.lock", "Cargo.toml.orig"}


def files(root):
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in root.rglob("*")
        if path.is_file()
        and path.name not in IGNORED
        and not {"target", ".git"}.intersection(path.relative_to(root).parts)
    }


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


def check_patch(base, fork, patch, package):
    expected = patch_text(base, fork, package)
    if not patch.is_file():
        raise ValueError(f"missing patch: {patch}")
    if patch.read_text() != expected:
        raise ValueError(f"stale patch: {patch}; regenerate with --write")
    with tempfile.TemporaryDirectory(prefix="herogpui-patch-") as temporary:
        replay = Path(temporary) / "source"
        shutil.copytree(base, replay)
        if expected:
            result = subprocess.run(
                ["patch", "--batch", "--forward", "-p2", "-i", str(patch.resolve())],
                cwd=replay, capture_output=True, text=True,
            )
            if result.returncode or "fuzz" in result.stdout or "offset" in result.stdout:
                raise ValueError(f"patch does not apply exactly: {patch}\n{result.stdout}{result.stderr}")
        # BSD patch leaves a zero-length file for a /dev/null deletion. Remove
        # only explicitly deleted files; -E would also erase intentional empties.
        original, changed = files(base), files(fork)
        for name in original.keys() - changed.keys():
            deleted = replay / name
            if deleted.is_file() and deleted.stat().st_size == 0:
                deleted.unlink()
        if files(replay) != changed:
            raise ValueError(f"patch replay differs from local fork: {package}")


def registry_package(name, version):
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
    print("GPUI patch self-test: replay passed; stale and missing patches rejected")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true", help="regenerate versioned patches")
    mode.add_argument("--check", action="store_true", help="verify freshness and exact replay (default)")
    mode.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    version = manifest["workspace"]["dependencies"]["gpui"]["version"]
    if not version.startswith("="):
        raise ValueError("the workspace GPUI dependency must have an exact version pin")
    version = version[1:]
    for name in PACKAGES:
        fork = ROOT / manifest["patch"]["crates-io"][name]["path"]
        package = tomllib.loads((fork / "Cargo.toml").read_text())["package"]
        if package["name"] != name or package["version"] != version:
            raise ValueError(f"{fork}: fork name/version differs from the workspace pin")
        base = registry_package(name, version)
        label = f"{name}-{version}"
        patch = ROOT / "docs/upstream/patches" / f"{label}.patch"
        if args.write:
            patch.parent.mkdir(parents=True, exist_ok=True)
            patch.write_text(patch_text(base, fork, label))
        check_patch(base, fork, patch, label)
        print(f"{label}: patch matches fork and replays exactly")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError) as error:
        raise SystemExit(f"GPUI PATCH ERROR: {error}") from error
