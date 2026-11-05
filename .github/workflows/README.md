# Workflows

| Workflow | Runs on | What it does |
| --- | --- | --- |
| [`ci.yml`](ci.yml) | Push to `main` and pull requests | On Ubuntu and macOS: checks formatting with `cargo fmt`, runs `cargo clippy` with warnings as errors, runs `cargo test` (the colour maths unit tests, the integration test over every shipped profile, a snapshot per target, the schema test and the command line tests), builds the release binary and runs `check --no-derive` over every profile in `profiles/` with it. **Minimum supported Rust** runs `cargo test --locked` on Rust 1.88, the `rust-version` in `Cargo.toml`, on Ubuntu |

Every action is pinned to a full commit SHA with its version in a comment. `dtolnay/rust-toolchain`
has no release tags, so it is pinned to the commit its `stable` branch pointed at and is given
the toolchain explicitly: `stable` for the main job and `1.88` for the minimum supported Rust
job. Dependabot proposes updates to the actions and the Rust crates weekly.
