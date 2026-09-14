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
        "repository": "https://github.com/Porabuild/HeroGPUI",
    }
    for key, value in expected.items():
        if shared.get(key) != value:
            errors.append(f"workspace.package.{key} must be {value!r}")

    # GPUI must stay a registry dependency. A git dependency anywhere in the
    # workspace makes every crate unpublishable, which is why zed-industries'
    # published `gpui-pre` packages are used instead of a Zed git revision.
    internal = workspace["workspace"]["dependencies"]
    gpui = internal.get("gpui", {})
    platform = internal.get("gpui_platform", {})
    requirement = gpui.get("version", "")
    packages = {"gpui": "gpui-pre", "gpui_platform": "gpui-pre-platform"}
    for name, dependency in (("gpui", gpui), ("gpui_platform", platform)):
        if dependency.get("git") or dependency.get("path"):
            errors.append(f"{name}: must come from crates.io, not git or a path")
        if dependency.get("package") != packages[name]:
            errors.append(f"{name}: must rename the {packages[name]} package")
        # The pin is exact on purpose. `gpui-pre-platform` requires the rest of
        # the family at an exact `=` version, and the materialized
        # `gpui-pre-web` fork's version has to satisfy that same requirement or
        # `[patch.crates-io]` stops applying with no error at all. A caret let a
        # bare `cargo update` walk off the pin once already.
        if not dependency.get("version", "").startswith("="):
            errors.append(f"{name}: version must be an exact `=` pin")
    if not requirement:
        errors.append("GPUI must pin a published gpui-pre version")
    if platform.get("version") != requirement:
        errors.append("GPUI and gpui_platform versions differ")
    version = requirement.lstrip("=")
    locked = manifest(ROOT / "Cargo.lock")["package"]
    # The published family remains the dependency contract of every package,
    # while the workspace's `[patch.crates-io]` block substitutes the
    # materialized, version-identical renderer forks from `.vendor/`. The
    # lockfile is therefore intentionally path-resolved for the five patched
    # members.
    patched_gpui = {
        "gpui-pre",
        "gpui-pre-apple",
        "gpui-pre-wgpu",
        "gpui-pre-windows",
        "gpui-pre-web",
    }
    for name in packages.values():
        entries = [entry for entry in locked if entry["name"] == name]
        if len(entries) != 1:
            errors.append(f"{name}: lockfile does not resolve to exactly one version")
            continue
        entry = entries[0]
        if name in patched_gpui:
            if entry.get("source"):
                errors.append(f"{name}: patched lockfile entry unexpectedly has a source")
        elif not (entry.get("source") or "").startswith("registry+"):
            errors.append(f"{name}: lockfile source is not the crates.io registry")
        if version and entry.get("version") != version:
            errors.append(f"{name}: lockfile version is not the pinned {version}")
    if any((entry.get("source") or "").startswith("git+") for entry in locked):
        errors.append("lockfile still contains a git source; the crates cannot be published")

    # No fork source is checked in: the deviation is recorded as a patch under
    # docs/upstream/patches/ and `.shots/gpui_patches.py --materialize` applies
    # it to the pinned published source under the gitignored `.vendor/`. So the
    # version agreement this audit guards is spelled in two committed places --
    # the `[patch.crates-io]` path and the patch filename -- and both have to
    # carry the pin. Cargo drops a `[patch.crates-io]` override silently when
    # the substitute's version no longer satisfies what `gpui-pre-platform`
    # asks for; the `gpui-pre-web` fork's two `events.rs` hunks and its
    # `default = []` feature deviation would just disappear from the browser
    # build, and nothing else would report it. `--check` verifies the patch
    # content; this verifies the wiring, with no Rust build and no `.vendor/`
    # tree, so it still runs in the parity job.
    overrides = workspace.get("patch", {}).get("crates-io", {})
    for name in sorted(patched_gpui):
        entry = overrides.get(name)
        if not isinstance(entry, dict) or not entry.get("path"):
            errors.append(f"{name}: no [patch.crates-io] path override")
            continue
        if entry.get("git"):
            errors.append(f"{name}: [patch.crates-io] must not use a git source")
        expected_path = f".vendor/{name}-{version}"
        if version and entry["path"] != expected_path:
            errors.append(
                f"{name}: [patch.crates-io] path is {entry['path']!r}, not {expected_path!r}; "
                "the override would apply the wrong version or none at all"
            )
        patch_file = ROOT / "docs/upstream/patches" / f"{name}-{version}.patch"
        if version and not patch_file.is_file():
            errors.append(
                f"{name}: docs/upstream/patches/{name}-{version}.patch is missing; "
                "nothing can materialize the fork"
            )
    web_entries = [entry for entry in locked if entry["name"] == "gpui-pre-web"]
    if len(web_entries) != 1:
        errors.append("gpui-pre-web: lockfile does not resolve to exactly one version")
    elif web_entries[0].get("source"):
        errors.append(
            "gpui-pre-web: lockfile carries a source, so the materialized fork is not patched in"
        )
    elif version and web_entries[0].get("version") != version:
        errors.append(f"gpui-pre-web: lockfile version is not the pinned {version}")
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
    if "cargo build --locked --release -p herogpui-gallery" not in release:
        errors.append("release workflow does not build the gallery binary")

    # The facade contract: `herogpui` must carry GPUI itself, so a consumer's
    # dependency list is one line. Both GPUI crates are therefore mandatory --
    # not optional, not behind a feature -- and every layer below the facade is
    # optional so the crate still compiles with `--no-default-features`.
    facade = manifest(PACKAGES["herogpui"] / "Cargo.toml")
    facade_features = facade.get("features", {})
    facade_deps = facade.get("dependencies", {})
    for name in ("gpui", "gpui_platform"):
        dependency = facade_deps.get(name)
        if dependency is None:
            errors.append(f"herogpui: {name} must be a direct dependency of the facade")
        elif isinstance(dependency, dict) and dependency.get("optional"):
            errors.append(
                f"herogpui: {name} must not be optional; the facade exists so a "
                "consumer never names it"
            )
    if facade_features.get("default") != ["components"]:
        errors.append('herogpui: default features must be ["components"]')
    for feature, requires in (
        ("components", "theme"),
        ("theme", "core"),
    ):
        if requires not in facade_features.get(feature, []):
            errors.append(f"herogpui: feature {feature} must imply {requires}")
    for name in ("herogpui-core", "herogpui-theme", "herogpui-components"):
        dependency = facade_deps.get(name, {})
        if not (isinstance(dependency, dict) and dependency.get("optional")):
            errors.append(
                f"herogpui: {name} must be optional so --no-default-features compiles"
            )
    for feature, forwarded in (
        ("test-support", ("gpui/test-support", "gpui_platform/test-support")),
        ("profiler", ("gpui/profiler",)),
        ("serde", ("theme", "herogpui-theme/serde")),
    ):
        enabled = facade_features.get(feature, [])
        for target in forwarded:
            if target not in enabled:
                errors.append(f"herogpui: feature {feature} must forward {target}")

    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    if "cargo install --path gallery --locked" not in readme:
        errors.append("README does not document cargo install --path gallery --locked")
    # The facade is not on crates.io. `cargo add herogpui` resolves a
    # different crate or fails; the documented install is a git dependency
    # on this repository, still the only line a consumer adds.
    git_dep = 'herogpui = { git = "https://github.com/Porabuild/HeroGPUI" }'
    if git_dep not in readme:
        errors.append(f"README does not document `{git_dep}`")
    if "not on crates.io" not in readme:
        errors.append("README does not say herogpui is not on crates.io")
    if "cargo add herogpui" in readme:
        errors.append(
            "README teaches `cargo add herogpui`; the crate is not on crates.io"
        )
    # One dependency is a claim a reader can check, so check it. A README that
    # tells a consumer to add GPUI directly -- a `cargo add gpui-...` line, or a
    # `gpui`/`gpui_platform` line inside a `[dependencies]` block -- has
    # reintroduced the multi-dependency install the facade replaced.
    for command in ("cargo add gpui-pre", "cargo add gpui-pre-platform"):
        if command in readme:
            errors.append(
                f"README instructs consumers to run `{command}`; the facade re-exports "
                "GPUI so herogpui is the only dependency a consumer adds"
            )
    for block in readme.split("```")[1::2]:
        if not block.lstrip().startswith("toml"):
            continue
        body = block.split("\n", 1)[1] if "\n" in block else ""
        if "[dependencies]" not in body:
            continue
        for line in body.splitlines():
            name = line.split("=", 1)[0].strip()
            if name in ("gpui", "gpui_platform"):
                errors.append(
                    f"README's [dependencies] block names {name}; the facade re-exports "
                    "GPUI so herogpui is the only dependency a consumer adds"
                )

    print(f"source packages      : {len(PACKAGES)}")
    print("library install      : git herogpui (the only dependency)")
    print(f"facade features      : {' '.join(sorted(facade_features))}")
    print("gallery install      : cargo install --path gallery --locked")
    print("license contract     : Apache-2.0 + NOTICE")
    print(f"PACKAGING ERRORS     : {len(errors)}")
    for error in errors:
        print(f"- {error}")
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
