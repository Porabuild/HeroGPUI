# Releasing HeroGPUI

HeroGPUI uses one version for four crates.io libraries, the crates.io gallery
CLI, the native gallery binaries, and the Git tag.

## One-time setup

1. Confirm the public `Porabuild/HeroGPUI` GitHub repository is in place and
   the working tree is pushed to it.
2. Create a protected `release` GitHub environment and protect `v*` tags.
3. Enable immutable GitHub Releases.
4. Reserve the five crates.io names. They were unclaimed when checked on
   2026-08-27, but registry ownership is first-come.

`release.yml` has no publish job, so steps 1 to 4 are all the workflow needs.
The registry credentials below are for a future publish job; until that job
exists, a maintainer publishes from a local machine with their own crates.io
credentials and none of these steps are required.

5. For the first automated publish only, add a short-lived `CRATES_IO_TOKEN`
   secret to the `release` environment, scoped to publishing these package
   names.
6. After that first publish succeeds, configure trusted publishers for all
   five crates to the `Porabuild/HeroGPUI` repository, `release.yml`
   workflow, and `release` environment.
7. Only after every trusted publisher is configured, delete the bootstrap
   secret. Later automated releases use OIDC and need no registry secrets.

## Release checklist

1. Update `[workspace.package].version` and all four version requirements
   under `[workspace.dependencies]` to the same SemVer value.
2. Run the complete local gate from `AGENTS.md`, plus:

   ```powershell
   cargo package -p herogpui-core --allow-dirty --no-verify --list
   cargo package -p herogpui-theme --allow-dirty --no-verify --list
   cargo package -p herogpui-components --allow-dirty --no-verify --list
   cargo package -p herogpui --allow-dirty --no-verify --list
   cargo package -p herogpui-gallery --allow-dirty --no-verify --list
   cargo publish --workspace --dry-run --allow-dirty --locked --no-verify
   ```

3. Commit, create an annotated `vX.Y.Z` tag, and push the commit and tag.
4. The release workflow builds every supported gallery binary, attests them,
   and creates the immutable GitHub Release with those binaries plus
   `LICENSE` and `NOTICE`. It does not publish to crates.io.
5. Publish the crates by hand, in dependency order, from the tagged commit:

   ```powershell
   cargo publish -p herogpui-core --locked
   cargo publish -p herogpui-theme --locked
   cargo publish -p herogpui-components --locked
   cargo publish -p herogpui --locked
   cargo publish -p herogpui-gallery --locked
   ```

6. Verify a new project with `cargo add herogpui`, and install the gallery
   with `cargo install herogpui-gallery` on at least one clean machine.

If a registry publish partially succeeds, never reuse or overwrite a published
version. Retry only the missing packages when safe; otherwise increment the
patch version. crates.io versions and immutable GitHub Release assets cannot be
replaced.
