"""Sync the embedded Lucide icon set with `.shots/lucide-icons.txt`.

The list pins a `lucide-static` npm release (version and tarball integrity)
and names the icons `herogpui-components` embeds. Without flags this script
downloads that tarball (or reads `--tarball PATH`), verifies its integrity,
copies each listed `icons/<name>.svg` byte for byte into
`crates/herogpui-components/assets/lucide/` (deleting SVGs no longer listed),
copies the package LICENSE there and to `web/public/LICENSE-lucide.txt`, and
regenerates the `lucide_icons!` variant list and `LUCIDE_VERSION` in
`crates/herogpui-components/src/icon.rs`.

`--check` changes nothing and needs no network: it fails when the list, the
files, the enum, `LUCIDE_VERSION` and the NOTICE disagree, or when a file is
not the canonical icon of the pinned release (its `@license` header and
`lucide-<name>` class). With `--tarball PATH` it also compares every file
with the tarball byte for byte. `--self-test` proves the check fails on
known-bad input.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import re
import sys
import tarfile
import tempfile
import urllib.request
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / ".shots/lucide-icons.txt"
COMPONENTS = ROOT / "crates/herogpui-components"
ASSET_DIR = COMPONENTS / "assets/lucide"
ICON_RS = COMPONENTS / "src/icon.rs"
NOTICES = [ROOT / "NOTICE", COMPONENTS / "NOTICE"]
SITE_LICENSE = ROOT / "web/public/LICENSE-lucide.txt"
REGISTRY = "https://registry.npmjs.org/lucide-static/-/lucide-static-{version}.tgz"

NAME = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
BLOCK = re.compile(r"(?m)^lucide_icons! \{\n(?P<body>.*?)^\}\n", re.S)
VERSION_CONST = re.compile(r'(?m)^pub const LUCIDE_VERSION: &str = "(?P<version>[^"]*)";$')


@dataclass
class Manifest:
    version: str
    integrity: str | None
    names: list[str]


def parse_manifest(text: str) -> tuple[Manifest | None, list[str]]:
    errors: list[str] = []
    version = integrity = None
    names: list[str] = []
    for number, raw in enumerate(text.splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("version "):
            version = line.split(None, 1)[1]
        elif line.startswith("integrity "):
            integrity = line.split(None, 1)[1]
        elif NAME.match(line):
            names.append(line)
        else:
            errors.append(f"lucide-icons.txt:{number}: not an icon name: {line!r}")
    if version is None:
        errors.append("lucide-icons.txt: no `version` line")
        return None, errors
    if names != sorted(set(names)):
        errors.append("lucide-icons.txt: icon names must be unique and sorted")
    if not names:
        errors.append("lucide-icons.txt: no icons listed")
    return Manifest(version, integrity, names), errors


def variant(name: str) -> str:
    """Lucide's kebab-case name in PascalCase: `volume-2` is `Volume2`."""
    return "".join(part[:1].upper() + part[1:] for part in name.split("-"))


def enum_body(names: list[str]) -> str:
    return "".join(f'    {variant(name)} => "{name}",\n' for name in names)


def check(
    manifest_text: str,
    icon_rs: str,
    files: dict[str, bytes],
    notices: dict[str, str],
    tarball: dict[str, bytes] | None = None,
) -> list[str]:
    """Every disagreement between the list, the files, the enum and the
    NOTICE. `files` maps each asset file name (`<name>.svg`, `LICENSE`) to its
    bytes; `tarball` maps the same names to the pinned release's copies."""
    manifest, errors = parse_manifest(manifest_text)
    if manifest is None:
        return errors

    version = VERSION_CONST.search(icon_rs)
    if not version:
        errors.append("icon.rs: `pub const LUCIDE_VERSION` not found")
    elif version["version"] != manifest.version:
        errors.append(
            f"icon.rs: LUCIDE_VERSION is {version['version']}, the list pins {manifest.version}"
        )
    block = BLOCK.search(icon_rs)
    if not block:
        errors.append("icon.rs: the `lucide_icons! { .. }` invocation not found")
    elif block["body"] != enum_body(manifest.names):
        errors.append(
            "icon.rs: the IconName list is not generated from lucide-icons.txt "
            "(run python3 .shots/sync-lucide.py)"
        )

    expected = {f"{name}.svg" for name in manifest.names} | {"LICENSE"}
    for missing in sorted(expected - files.keys()):
        errors.append(f"assets/lucide: {missing} is listed but missing")
    for extra in sorted(files.keys() - expected):
        errors.append(f"assets/lucide: {extra} is not in lucide-icons.txt")

    header = f"<!-- @license lucide-static v{manifest.version} - ISC -->"
    for name in manifest.names:
        data = files.get(f"{name}.svg")
        if data is None:
            continue
        text = data.decode("utf-8", "replace")
        if not text.startswith(header):
            errors.append(f"assets/lucide/{name}.svg is not from lucide-static {manifest.version}")
        if f'class="lucide lucide-{name}"' not in text:
            errors.append(f"assets/lucide/{name}.svg is an alias, not the canonical icon")
    if "LICENSE" in files and not files["LICENSE"].startswith(b"ISC License"):
        errors.append("assets/lucide/LICENSE is not Lucide's ISC license")

    for path, text in notices.items():
        if f"lucide-static {manifest.version}" not in text:
            errors.append(f"{path}: does not name lucide-static {manifest.version}")

    if tarball is not None:
        for file, data in sorted(files.items()):
            upstream = tarball.get(file)
            if upstream is None:
                errors.append(f"assets/lucide/{file} is not in the pinned tarball")
            elif upstream != data:
                errors.append(f"assets/lucide/{file} differs from the pinned tarball")
    return errors


def integrity_of(data: bytes) -> str:
    return "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()


def load_tarball(manifest: Manifest, path: Path | None) -> dict[str, bytes]:
    if path is not None:
        data = path.read_bytes()
    else:
        url = REGISTRY.format(version=manifest.version)
        print(f"downloading {url}")
        with urllib.request.urlopen(url, timeout=60) as response:
            data = response.read()
    actual = integrity_of(data)
    if manifest.integrity is None:
        sys.exit(f"lucide-icons.txt has no `integrity` line; the tarball's is\n  integrity {actual}")
    if actual != manifest.integrity:
        sys.exit(
            "the tarball does not match the pinned integrity\n"
            f"  pinned {manifest.integrity}\n  actual {actual}"
        )
    out: dict[str, bytes] = {}
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            if member.name == "package/LICENSE":
                out["LICENSE"] = archive.extractfile(member).read()
            elif member.name.startswith("package/icons/") and member.name.endswith(".svg"):
                out[member.name.rsplit("/", 1)[1]] = archive.extractfile(member).read()
    return out


def read_files() -> dict[str, bytes]:
    return {path.name: path.read_bytes() for path in ASSET_DIR.iterdir() if path.is_file()}


def read_notices() -> dict[str, str]:
    return {str(path.relative_to(ROOT)): path.read_text(encoding="utf-8") for path in NOTICES}


def sync(tarball_path: Path | None) -> int:
    manifest, errors = parse_manifest(MANIFEST.read_text(encoding="utf-8"))
    if manifest is None or errors:
        print("\n".join(errors))
        return 1
    upstream = load_tarball(manifest, tarball_path)
    unknown = [name for name in manifest.names if f"{name}.svg" not in upstream]
    if unknown:
        print(f"not in lucide-static {manifest.version}: {', '.join(unknown)}")
        return 1
    wanted = {f"{name}.svg" for name in manifest.names}
    for path in ASSET_DIR.glob("*.svg"):
        if path.name not in wanted:
            path.unlink()
            print(f"removed {path.name}")
    for file in sorted(wanted):
        target = ASSET_DIR / file
        if not target.is_file() or target.read_bytes() != upstream[file]:
            target.write_bytes(upstream[file])
            print(f"wrote {file}")
    (ASSET_DIR / "LICENSE").write_bytes(upstream["LICENSE"])
    SITE_LICENSE.write_bytes(upstream["LICENSE"])

    icon_rs = ICON_RS.read_text(encoding="utf-8")
    icon_rs = BLOCK.sub(lambda _: f"lucide_icons! {{\n{enum_body(manifest.names)}}}\n", icon_rs, 1)
    icon_rs = VERSION_CONST.sub(
        f'pub const LUCIDE_VERSION: &str = "{manifest.version}";', icon_rs, 1
    )
    ICON_RS.write_text(icon_rs, encoding="utf-8")

    errors = check(
        MANIFEST.read_text(encoding="utf-8"),
        icon_rs,
        read_files(),
        read_notices(),
        tarball=upstream,
    )
    if errors:
        # Only the NOTICE text is left to the maintainer.
        print("\n".join(errors))
        return 1
    print(f"{len(manifest.names)} Lucide icons from lucide-static {manifest.version}")
    return 0


def self_test() -> int:
    manifest = "version 1.0.0\nintegrity sha512-x\nalpha\nbeta-2\n"
    icon_rs = (
        'pub const LUCIDE_VERSION: &str = "1.0.0";\n'
        "lucide_icons! {\n"
        '    Alpha => "alpha",\n'
        '    Beta2 => "beta-2",\n'
        "}\n"
    )
    header = "<!-- @license lucide-static v1.0.0 - ISC -->\n"

    def svg(name: str) -> bytes:
        return f'{header}<svg class="lucide lucide-{name}"/>'.encode()

    files = {"alpha.svg": svg("alpha"), "beta-2.svg": svg("beta-2"), "LICENSE": b"ISC License"}
    notices = {"NOTICE": "lucide-static 1.0.0"}
    cases = {
        "a clean set": (manifest, icon_rs, files, notices, None, 0),
        "an unsorted list": ("version 1.0.0\nbeta-2\nalpha\n", icon_rs, files, notices, None, 1),
        "a stale enum": (manifest, icon_rs.replace("    Beta2", "    Beta"), files, notices, None, 1),
        "a stale version": (manifest, icon_rs.replace('"1.0.0"', '"0.9.0"'), files, notices, None, 1),
        "a missing file": (manifest, icon_rs, {k: v for k, v in files.items() if k != "alpha.svg"}, notices, None, 1),
        "an unlisted file": (manifest, icon_rs, {**files, "gamma.svg": svg("gamma")}, notices, None, 1),
        "a file of another release": (
            manifest, icon_rs, {**files, "alpha.svg": svg("alpha").replace(b"v1.0.0", b"v0.9.0")}, notices, None, 1,
        ),
        "an alias file": (manifest, icon_rs, {**files, "alpha.svg": svg("alpha-alias")}, notices, None, 1),
        "a stale NOTICE": (manifest, icon_rs, files, {"NOTICE": "lucide-static 0.9.0"}, None, 1),
        "a file unlike the tarball": (manifest, icon_rs, files, notices, {**files, "alpha.svg": b"x"}, 1),
        "a file matching the tarball": (manifest, icon_rs, files, notices, dict(files), 0),
    }
    failed = 0
    for label, (m, rs, fs, ns, tb, want_errors) in cases.items():
        errors = check(m, rs, fs, ns, tb)
        if bool(errors) != bool(want_errors):
            failed += 1
            print(f"self-test FAIL: {label}: {errors or 'no errors'}")
    if variant("volume-2") != "Volume2" or variant("x") != "X":
        failed += 1
        print("self-test FAIL: PascalCase variants")
    if failed:
        return 1
    print(f"self-test PASS: {len(cases)} cases (a clean set and a matching tarball pass; each defect fails)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--check", action="store_true", help="verify only; no network, no writes")
    parser.add_argument("--tarball", type=Path, help="a local lucide-static .tgz instead of the registry")
    parser.add_argument("--self-test", action="store_true", help="prove the check fails on bad input")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.check:
        upstream = None
        if args.tarball is not None:
            manifest, errors = parse_manifest(MANIFEST.read_text(encoding="utf-8"))
            if manifest is None:
                print("\n".join(errors))
                return 1
            upstream = load_tarball(manifest, args.tarball)
        errors = check(
            MANIFEST.read_text(encoding="utf-8"),
            ICON_RS.read_text(encoding="utf-8"),
            read_files(),
            read_notices(),
            tarball=upstream,
        )
        if errors:
            print("\n".join(errors))
            print(f"lucide sync check: {len(errors)} problem(s)")
            return 1
        print("lucide sync check: list, files, IconName, LUCIDE_VERSION and NOTICE agree")
        return 0
    return sync(args.tarball)


if __name__ == "__main__":
    sys.exit(main())
