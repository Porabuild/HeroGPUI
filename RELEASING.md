# Releasing HeroGPUI

HeroGPUI uses one version for four crates.io libraries, the crates.io gallery
CLI, the native gallery binaries, and the Git tag.

## One-time setup

1. Create the public `heroui-inc/HeroGPUI` GitHub repository and push the full
   tracked tree. The repository URL does not resolve until this is done.
2. Create a protected `release` GitHub environment and protect `v*` tags.
3. Enable immutable GitHub Releases.
4. Reserve the five crates.io names. They were unclaimed when checked on
   2026-08-27, but registry ownership is first-come.
5. For the first tag only, add a short-lived `CRATES_IO_TOKEN` secret to the
   `release` environment, scoped to publishing these package names.
6. After that first workflow succeeds, configure trusted publishers for all
   five crates to the `heroui-inc/HeroGPUI` repository, `release.yml`
   workflow, and `release` environment.
7. Only after every trusted publisher is configured, delete the bootstrap
   secret. Future releases use OIDC and need no registry secrets.

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
4. The release workflow builds every supported gallery binary before
   publishing anything. It then attests the binaries, creates the immutable
   GitHub Release, and publishes the Rust crates.
5. Verify a new project with `cargo add herogpui`, and install the gallery
   with `cargo install herogpui-gallery` on at least one clean machine.

If a registry publish partially succeeds, never reuse or overwrite a published
version. Retry only the missing packages when safe; otherwise increment the
patch version. crates.io versions and immutable GitHub Release assets cannot be
replaced.
