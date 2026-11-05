# Packaging

How Zacess Profile reaches crates.io, Homebrew and winget after a release. The binaries and the
crates.io upload come from [`release.yml`](../.github/workflows/release.yml). The Homebrew formula
and the winget manifests are submitted by a maintainer from their own machine with their own
credentials. No token is ever stored in this repository.

Before any of these, the release must exist: the tag `vX.Y.Z` pushed, the GitHub release made from
it and `release.yml` finished, so the five archives and their `.sha256` files are attached.
`gh release view vX.Y.Z --repo zacesshq/profile` lists them.

## In this folder

| File | Does |
| --- | --- |
| [`homebrew/zacess-profile.rb`](homebrew/zacess-profile.rb) | The formula for the tap, at the current version with a `SHA256_` placeholder per platform |
| [`winget/`](winget) | The three winget manifests (version, installer and default locale) in manifest schema 1.12.0, at the current version, with one placeholder for the Windows zip |
| [`fill-checksums.sh`](fill-checksums.sh) | Downloads the `.sha256` files of a release and writes the formula and the manifests with the sums filled in to a folder of your choice |

The files here keep their placeholders. A release bumps the version in them (the four URLs and the
`version` line in the formula, `PackageVersion`, the zip path and `ReleaseNotesUrl` in the
manifests) in the same pull request as `Cargo.toml`. `tests/packaging.rs` fails until they match.

## What each archive holds

`zacess-profile-vX.Y.Z-<target>.tar.gz` (`.zip` on Windows) holds the `zprf` binary, the eight
starter profiles in `profiles/`, the spec in `spec/` and the README, licence, notice, changelog,
roadmap and accessibility statement. The targets are `aarch64-apple-darwin`, `x86_64-apple-darwin`,
`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `x86_64-pc-windows-msvc`. The Linux
binaries are static and the Windows binary links its C runtime statically.

## crates.io

`release.yml` publishes the crate once every archive is attached, through Trusted Publishing from
the `crates-io` environment, so no token is stored anywhere. The first version of a new crate has
to be published once by hand. The
[workflows README](../.github/workflows/README.md#publishing-to-cratesio) lists the trusted publisher
to add after that. To publish by hand:

```sh
# check what would be packaged and that the metadata is complete
cargo publish --dry-run
cargo package --list

# publish with a token read from a file, never typed into the shell history
CARGO_REGISTRY_TOKEN="$(cat /path/to/token)" cargo publish
```

## Homebrew

The tap is `zacesshq/homebrew-tap`, which Homebrew reads as `zacesshq/tap`. The formula installs
the release archive for the person's platform rather than building from source, so it needs the
SHA-256 of each of the four Unix archives.

1. Fill the formula from the release files and copy it into the tap:

   ```sh
   packaging/fill-checksums.sh vX.Y.Z /tmp/packaging-X.Y.Z
   cp /tmp/packaging-X.Y.Z/homebrew/zacess-profile.rb homebrew-tap/Formula/
   ```

2. Check it from the tap checkout, then commit and push it:

   ```sh
   brew install --formula ./Formula/zacess-profile.rb
   brew test zacess-profile
   brew style ./Formula/zacess-profile.rb
   zprf --version
   ```

Once the tap holds the formula, the install line is `brew install zacesshq/tap/zacess-profile`. The
starter profiles land in `$(brew --prefix)/share/zacess-profile/profiles`.

## winget

The package is `Zacess.Profile`. The installer is the Windows zip from the release, installed as a
portable program with the command alias `zprf`.

1. Fill the manifests from the release files:

   ```sh
   packaging/fill-checksums.sh vX.Y.Z /tmp/packaging-X.Y.Z
   ```

2. Copy the three files from `/tmp/packaging-X.Y.Z/winget/` into a fork of
   [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) under
   `manifests/z/Zacess/Profile/X.Y.Z/`.
3. Validate and test them on a Windows machine:

   ```powershell
   winget validate --manifest manifests\z\Zacess\Profile\X.Y.Z
   winget install --manifest manifests\z\Zacess\Profile\X.Y.Z
   zprf --version
   ```

4. Open a pull request to microsoft/winget-pkgs. Their checks take a day or two and the package is
   live once it merges.

Once accepted, the install line is `winget install Zacess.Profile`. Every later release is submitted
the same way with its own version folder.
