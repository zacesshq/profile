# Workflows

| Workflow | Runs on | What it does |
| --- | --- | --- |
| [`ci.yml`](ci.yml) | Push to `main`, pull requests and by hand | **Rust** on Ubuntu, macOS and Windows: checks formatting with `cargo fmt`, runs `cargo clippy` with warnings as errors, runs `cargo test` (the colour maths and colour vision unit tests, the integration test over every shipped profile, a snapshot per target, the schema, packaging and preview tests and the command line tests, including the watcher), builds the release binary and runs `check --no-derive` over every profile in `profiles/` with it. **Minimum supported Rust** runs `cargo test --locked` on Rust 1.88, the `rust-version` in `Cargo.toml`. **Release binary** builds the five release targets the way `release.yml` does and checks every profile with each binary that can run on its runner, the aarch64 Linux one under QEMU. **Browser extension** type checks and builds the extension, loads it in Chromium with Playwright to check the computed styles of a test page and lints the Firefox build with `web-ext lint` |
| [`release.yml`](release.yml) | A pushed `v*` tag | Checks the tag matches the version in `Cargo.toml`, builds `zprf` for macOS (arm64 and x86_64), Linux (x86_64 and aarch64, static musl builds) and Windows (x86_64), packs each with the docs and the starter profiles, waits for the GitHub release of the tag and attaches each archive with its SHA-256 sum through `gh release upload`. Once every archive is attached it publishes the crate to crates.io through Trusted Publishing from the `crates-io` environment, skipping cleanly when crates.io already has that version. Once every platform's files are attached, three more jobs run side by side. **Tell the website** sends a `profile-release` repository dispatch naming the tag to `zacesshq/website`, so zacess.com rebuilds straight away. **Update the Homebrew tap** fills `Formula/zacess-profile.rb` from the release's `.sha256` files with `packaging/fill-checksums.sh` and lands it in `zacesshq/homebrew-tap` through a pull request it merges itself. **Submit to winget** opens the pull request for the new version of `Zacess.Profile` to `microsoft/winget-pkgs` from the fork `zaccesss/winget-pkgs`. Each of the three skips with a note in the run summary when its secret is missing and only reports a failure, never failing the release. It never creates the release itself |

## Publishing to crates.io

The crate is published with [Trusted Publishing](https://crates.io/docs/trusted-publishing): the
publish job proves its identity to crates.io with GitHub's OIDC token through
`rust-lang/crates-io-auth-action`, so no crates.io token is stored as a secret. Trusted Publishing
can only publish a crate that already exists, so the first version of `zacess-profile` is published
once by hand with `cargo publish`. After that, a crate owner adds the trusted publisher in the
crate's settings on crates.io:

| Field | Value |
| --- | --- |
| Repository owner | `zacesshq` |
| Repository name | `profile` |
| Workflow file | `release.yml` |
| Environment | `crates-io` |

Until the crate exists, the publish job reports that and ends without failing. Once a version is on
crates.io, running the workflow again for its tag skips the upload.

## Secrets for the stores

| Secret | Kind | Used by | Holds |
| --- | --- | --- | --- |
| `SITE_DISPATCH_TOKEN` | Organisation secret, shared with this repository | Tell the website | A fine-grained token limited to `zacesshq/website` with Contents read and write, which is what a repository dispatch needs |
| `TAP_TOKEN` | Repository secret | Update the Homebrew tap | A fine-grained token from the `zaccesssbot` account limited to `zacesshq/homebrew-tap` with Contents and Pull requests read and write |
| `WINGET_TOKEN` | Repository secret | Submit to winget | A classic token from the `zaccesss` account with the `public_repo` scope. Adding the `workflow` scope lets the action sync the fork when upstream workflow files change |

The tap's `protect-main` ruleset takes changes on `main` only through a squash-merged pull request,
so the Homebrew job never pushes to `main`. It writes the formula to a branch named
`zacess-profile-X.Y.Z` as a commit by Zac Bot, opens a pull request from it and squash merges it.
A rerun after the formula landed finds nothing to change and stops.

The winget job uses [WinGet Releaser](https://github.com/vedantmgoyal9/winget-releaser), which runs
[Komac](https://github.com/russellbanks/Komac) to build the new manifests from the last version in
`microsoft/winget-pkgs`. winget-pkgs takes submissions from the maintainer's own account, which is
why the token and the fork belong to `zaccesss`. The first version,
[Zacess.Profile 0.2.0](https://github.com/microsoft/winget-pkgs/pull/471488), was submitted by hand
and is waiting for review. Until it is accepted there is nothing to build on, so the job skips with
a note. From then on it submits every new version.

Every action is pinned to a full commit SHA with its version in a comment. `dtolnay/rust-toolchain`
has no release tags, so it is pinned to the commit its `stable` branch pointed at and is given
the toolchain explicitly: `stable` for most jobs and `1.88` for the minimum supported Rust job.
WinGet Releaser is pinned to the commit its `v2` tag points at. Dependabot proposes updates to the
actions, the Rust crates and the extension's npm packages weekly.
