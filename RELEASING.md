# Releasing HeroGPUI

HeroGPUI uses one version for four crates.io libraries, the crates.io gallery
CLI, the native gallery binaries, and the Git tag.

This flow only became real once the workspace stopped depending on a Zed GPUI
git revision: cargo refuses to publish any crate that carries a git dependency,
so every `cargo publish` below would have failed outright. GPUI now comes from
the published `gpui-pre` crates named in `[workspace.dependencies]`, with no
`[patch.crates-io]` overrides of any kind, and that registry dependency is
what makes the steps below executable. Reintroducing a git dependency — or a
`[patch]` override, which downstream builds silently ignore — re-breaks the
single-dependency installation contract (see `.shots/package_audit.py`, which
gates both).

## No materialization step

There is none. An earlier revision carried the GPUI deviations as patches
under `docs/upstream/retired-patches/`; that mechanism is retired (see `docs/upstream/retired-patches/`), and every
step below runs on a fresh clone with plain `cargo`.

## Why the GPUI pin is exact

`[workspace.dependencies]` pins `gpui-pre` and `gpui-pre-platform` at
`=0.3.5`, exactly as gpui-kit does. `gpui-pre` is a prerelease publish of the
Zed GPUI sources by crates.io user huacnlee (Jason Lee, the gpui-kit
maintainer), and each 0.3.x is a snapshot of different Zed sources, so a
caret would let a downstream build resolve GPUI code no HeroGPUI release was
tested against. `package_audit.py` rejects a non-`=` requirement and fails the
release if the lockfile drifts off the pinned version.

## One-time setup

1. Confirm the public `Porabuild/HeroGPUI` GitHub repository is in place and
   the working tree is pushed to it.
2. Create a protected `release` GitHub environment and protect `v*` tags.
3. Enable immutable GitHub Releases. This is safe for the website: its
   gallery artifacts are published one prerelease per build key, with every
   asset attached while the release is still a draft, and CI afterwards only
   edits titles and deletes whole old releases, both of which immutability
   allows (`web/DEPLOYMENT.md`, section 6). Do not add a tag ruleset that
   blocks GitHub Actions from creating or deleting `gallery-*` tags.
4. Reserve the five crates.io names. They were unclaimed when checked on
   2026-08-27, but registry ownership is first-come.

`release.yml` builds the gallery binaries, creates the immutable GitHub
Release, and then publishes the five crates to crates.io over OIDC trusted
publishing (`publish-crates` job, `release` environment). No registry token
is stored in GitHub.

5. The first release of every crate is published by hand (see the checklist
   below), because crates.io only lets an existing crate declare a trusted
   publisher. 0.9.0 was published that way on 2026-09-16.
6. On crates.io, for each of `herogpui`, `herogpui-core`, `herogpui-theme`,
   `herogpui-components` and `herogpui-gallery`: Settings -> Trusted
   Publishing -> GitHub, repository `Porabuild/HeroGPUI`, workflow
   `release.yml`, environment `release`.
7. From then on a pushed `vX.Y.Z` tag publishes automatically. The job skips
   any version the registry already has, so a re-run after a partial
   publish continues where it stopped, and a hand publish before the tag
   is harmless.

## Release checklist

1. Update `[workspace.package].version` and all four version requirements
   under `[workspace.dependencies]` to the same SemVer value.
2. In `CHANGELOG.md`, set the release heading to `## [X.Y.Z] - YYYY-MM-DD`
   with the actual release date, point `[Unreleased]` at `vX.Y.Z...HEAD` and
   add the `[X.Y.Z]` compare link. Bump `"version"` in `web/package.json` to
   the same value, then run `pnpm run extract` from `web/` so
   `web/src/data/releases.json` is regenerated from the changelog (and
   `pnpm run extract:check` passes).
3. Run the complete local gate from `AGENTS.md`, plus:

   ```powershell
   cargo package -p herogpui-core --allow-dirty --no-verify --list
   cargo package -p herogpui-theme --allow-dirty --no-verify --list
   cargo package -p herogpui-components --allow-dirty --no-verify --list
   cargo package -p herogpui --allow-dirty --no-verify --list
   cargo package -p herogpui-gallery --allow-dirty --no-verify --list
   cargo publish --workspace --dry-run --allow-dirty --locked --no-verify
   # What the release's `semver` job runs (install: cargo install
   # cargo-semver-checks --locked --version 0.50.0, the version CI pins):
   cargo semver-checks -p herogpui-core -p herogpui-theme -p herogpui-components -p herogpui
   ```

4. Commit, create an annotated `vX.Y.Z` tag, and push the commit and tag.
5. The release workflow builds every supported gallery binary, attests them,
   and creates the immutable GitHub Release with those binaries plus
   `LICENSE`, `NOTICE` and `LICENSE-lucide` (the ISC/MIT license of the
   Lucide icons the binaries embed). It is created only after the `semver`
   job has checked the four library crates' public API against their latest
   crates.io release with `cargo semver-checks`: a breaking change needs a
   0.x minor (or major) version, never a patch.
6. The workflow's `publish-crates` job publishes the crates. To publish by
   hand instead (first release of a crate, or a workflow outage), from the
   tagged commit and in dependency order:

   ```powershell
   cargo publish -p herogpui-core --locked
   cargo publish -p herogpui-theme --locked
   cargo publish -p herogpui-components --locked
   cargo publish -p herogpui --locked
   cargo publish -p herogpui-gallery --locked
   ```

7. Verify a new project with `cargo add herogpui`, and install the gallery
   with `cargo install herogpui-gallery` on at least one clean machine.

The web gallery (the WebAssembly build the website embeds) is **not** a
release asset. CI builds it on every pull request and publishes it on master pushes as its
own `gallery-<key16>` prerelease, keyed by its build inputs, and the website
deploys from master on its own (`web/DEPLOYMENT.md`, section 6). The release
workflow's CI run builds the tagged tree again. The reusable call allows
`contents: write` because GitHub validates its master-only publisher job's
permission even when that job is skipped on a tag; every running tag CI job
remains read-only. The master push has already published its artifact.
Those prereleases are never marked Latest and their `gallery-*` tags do not
match the `v*` pattern this workflow triggers on, so they neither show as a
HeroGPUI version nor start a release.

If a registry publish partially succeeds, never reuse or overwrite a published
version. Retry only the missing packages when safe; otherwise increment the
patch version. crates.io versions and immutable GitHub Release assets cannot be
replaced.
