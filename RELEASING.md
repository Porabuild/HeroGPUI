# Releasing HeroGPUI

HeroGPUI uses one version for four crates.io libraries, the native gallery
binaries, the npm gallery launcher, and the Git tag.

## One-time setup

1. Create the public `heroui-inc/HeroGPUI` GitHub repository and push the full
   tracked tree. The repository URL does not resolve until this is done.
2. Create a protected `release` GitHub environment and protect `v*` tags.
3. Enable immutable GitHub Releases.
4. Reserve the four crates.io names and the `herogpui` npm name. They were
   unclaimed when checked on 2026-08-27, but registry ownership is first-come.
5. For the first tag only, add short-lived `CRATES_IO_TOKEN` and `NPM_TOKEN`
   secrets to the `release` environment. Scope them to publishing these package
   names.
6. After that first workflow succeeds, configure trusted publishers for all four crates and the npm package to the
   `heroui-inc/HeroGPUI` repository, `release.yml` workflow, and `release`
   environment. npm must allow `npm publish`.
7. Only after every trusted publisher is configured, delete the two bootstrap
   secrets. Future releases use OIDC and need no registry secrets.

## Release checklist

1. Update `[workspace.package].version`, all four version requirements under
   `[workspace.dependencies]`, and `version` in `npm/package.json` to the same
   SemVer value.
2. Run the complete local gate from `AGENTS.md`, plus:

   ```powershell
   cargo package -p herogpui-core --allow-dirty --no-verify --list
   cargo package -p herogpui-theme --allow-dirty --no-verify --list
   cargo package -p herogpui-components --allow-dirty --no-verify --list
   cargo package -p herogpui --allow-dirty --no-verify --list
   Push-Location npm
   npm ci
   npm test
   npm pack --dry-run
   Pop-Location
   ```

3. Commit, create an annotated `vX.Y.Z` tag, and push the commit and tag.
4. The release workflow builds every supported gallery binary before publishing
   anything. It then generates SHA-256 checksums and attestations, creates the
   immutable GitHub Release, publishes the Rust crates, and finally publishes
   the npm launcher after its downloadable binaries exist.
5. Verify a new project with `cargo add herogpui` and launch the gallery with
   `npx herogpui` on at least one clean machine.

If a registry publish partially succeeds, never reuse or overwrite a published
version. Retry only the missing packages when safe; otherwise increment the
patch version. crates.io versions and immutable GitHub Release assets cannot be
replaced.
