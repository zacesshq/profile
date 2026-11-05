# Workflows

| Workflow | Runs on | What it does |
| --- | --- | --- |
| [`ci.yml`](ci.yml) | Push to `main`, pull requests and by hand | **Rust** on Ubuntu, macOS and Windows: checks formatting with `cargo fmt`, runs `cargo clippy` with warnings as errors, runs `cargo test` (the colour maths and colour vision unit tests, the integration test over every shipped profile, a snapshot per target, the schema, packaging and preview tests and the command line tests, including the watcher), builds the release binary and runs `check --no-derive` over every profile in `profiles/` with it. **Minimum supported Rust** runs `cargo test --locked` on Rust 1.88, the `rust-version` in `Cargo.toml`. **Release binary** builds the five release targets the way `release.yml` does and checks every profile with each binary that can run on its runner, the aarch64 Linux one under QEMU. **Browser extension** type checks and builds the extension, loads it in Chromium with Playwright to check the computed styles of a test page and lints the Firefox build with `web-ext lint` |
| [`release.yml`](release.yml) | A pushed `v*` tag | Checks the tag matches the version in `Cargo.toml`, builds `zprf` for macOS (arm64 and x86_64), Linux (x86_64 and aarch64, static musl builds) and Windows (x86_64), packs each with the docs and the starter profiles, waits for the GitHub release of the tag and attaches each archive with its SHA-256 sum through `gh release upload`. Once every archive is attached it publishes the crate to crates.io through Trusted Publishing from the `crates-io` environment, skipping cleanly when crates.io already has that version. It never creates the release itself |

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

Every action is pinned to a full commit SHA with its version in a comment. `dtolnay/rust-toolchain`
has no release tags, so it is pinned to the commit its `stable` branch pointed at and is given
the toolchain explicitly: `stable` for most jobs and `1.88` for the minimum supported Rust job.
Dependabot proposes updates to the actions, the Rust crates and the extension's npm packages
weekly.
