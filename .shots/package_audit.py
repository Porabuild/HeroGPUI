"""Audit source distribution and the installable gallery CLI contract."""

from pathlib import Path
import sys
import tomllib


ROOT = Path(__file__).resolve().parent.parent
PACKAGES = {
    "herogpui-core": ROOT / "crates/herogpui-core",
    "herogpui-theme": ROOT / "crates/herogpui-theme",
    "herogpui-components": ROOT / "crates/herogpui-components",
    "herogpui": ROOT / "crates/herogpui",
    "herogpui-gallery": ROOT / "gallery",
}


def manifest(path):
    with path.open("rb") as source:
        return tomllib.load(source)


def inherited(package, key):
    value = package.get(key)
    return isinstance(value, dict) and value.get("workspace") is True


def main():
    errors = []
    workspace = manifest(ROOT / "Cargo.toml")
    shared = workspace["workspace"]["package"]
    expected = {
        "edition": "2024",
        "rust-version": "1.98",
        "license": "Apache-2.0",
        "repository": "https://github.com/heroui-inc/HeroGPUI",
    }
    for key, value in expected.items():
        if shared.get(key) != value:
            errors.append(f"workspace.package.{key} must be {value!r}")

    internal = workspace["workspace"]["dependencies"]
    gpui = internal.get("gpui", {})
    platform = internal.get("gpui_platform", {})
    revision = gpui.get("rev", "")
    repository = "https://github.com/zed-industries/zed"
    if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
        errors.append("GPUI must pin a complete Zed git revision")
    if gpui.get("git") != repository or platform.get("git") != repository:
        errors.append("GPUI and gpui_platform must come from Zed")
    if platform.get("rev") != revision:
        errors.append("GPUI and gpui_platform revisions differ")
    locked = manifest(ROOT / "Cargo.lock")["package"]
    for name in ("gpui", "gpui_platform"):
        sources = [entry.get("source") for entry in locked if entry["name"] == name]
        if sources != [f"git+{repository}?rev={revision}#{revision}"]:
            errors.append(f"{name}: lockfile does not match the pinned revision")
    for name in ("herogpui-core", "herogpui-theme", "herogpui-components", "herogpui"):
        dependency = internal.get(name, {})
        if not dependency.get("version") or not dependency.get("path"):
            errors.append(f"workspace dependency {name} needs both version and path")

    for name, directory in PACKAGES.items():
        data = manifest(directory / "Cargo.toml")
        package = data["package"]
        if package.get("name") != name:
            errors.append(f"{directory}: package name is not {name}")
        for key in (
            "version",
            "edition",
            "rust-version",
            "license",
            "readme",
            "repository",
            "keywords",
            "categories",
        ):
            if not inherited(package, key):
                errors.append(f"{name}: {key} is not inherited from workspace.package")
        if package.get("publish") is False:
            errors.append(f"{name}: publish is disabled")

        notice = directory / "NOTICE"
        if not notice.is_file():
            errors.append(f"{name}: NOTICE is missing")
        local_license = directory / "LICENSE"
        license_file = package.get("license-file")
        resolved_license = (
            (directory / license_file).resolve() if license_file else local_license
        )
        if not resolved_license.is_file():
            errors.append(f"{name}: packaged Apache license text is missing")

    components = manifest(PACKAGES["herogpui-components"] / "Cargo.toml")
    if "gallery-source" not in components.get("features", {}):
        errors.append("herogpui-components: gallery-source feature is missing")

    gallery = manifest(PACKAGES["herogpui-gallery"] / "Cargo.toml")
    binaries = gallery.get("bin", [])
    if not any(binary.get("name") == "herogpui-gallery" for binary in binaries):
        errors.append("herogpui-gallery: installable herogpui-gallery binary is missing")
    gallery_components = gallery.get("dependencies", {}).get("herogpui-components", {})
    if "gallery-source" not in gallery_components.get("features", []):
        errors.append("herogpui-gallery: gallery-source feature is not enabled")

    gallery_sources = "\n".join(
        path.read_text(encoding="utf-8")
        for path in (PACKAGES["herogpui-gallery"] / "src").rglob("*.rs")
    )
    if "../../../crates/herogpui-components" in gallery_sources:
        errors.append("herogpui-gallery: source still reaches outside its package")

    ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
    release = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
    if "cargo install --path gallery" not in ci:
        errors.append("CI does not exercise Cargo installation of the gallery CLI")
    if "cargo package" in ci or "cargo publish" in ci or "cargo publish" in release:
        errors.append("git GPUI dependencies cannot be packaged for crates.io")
    if "cargo build --locked --release -p herogpui-gallery" not in release:
        errors.append("release workflow does not build the gallery binary")

    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    for command in ("cargo add herogpui --path", "cargo install --path gallery --locked"):
        if command not in readme:
            errors.append(f"README does not document {command}")

    print(f"source packages      : {len(PACKAGES)}")
    print("library install      : cargo add herogpui --path <checkout>/crates/herogpui")
    print("gallery install      : cargo install --path gallery --locked")
    print("license contract     : Apache-2.0 + NOTICE")
    print(f"PACKAGING ERRORS     : {len(errors)}")
    for error in errors:
        print(f"- {error}")
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
