# Contributing to Zacess Profile

Thank you for helping. This guide covers how a change is made and checked.

## How a change happens

1. Every change starts from an issue that says what "done" means. A new app uses the
   [new target form](https://github.com/zacesshq/profile/issues/new?template=new_target.yml) and an
   accessibility problem uses the
   [accessibility form](https://github.com/zacesshq/profile/issues/new?template=accessibility.yml).
2. Work on a branch named after its type and topic, such as `feat/helix-target` or
   `fix/kitty-cursor`.
3. Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`,
   `fix:`, `docs:`, `refactor:`, `test:` and `chore:`.
4. Open a pull request that closes its issue. It is squash-merged once the checks pass and it is
   reviewed.

## Checks

With a stable Rust toolchain installed, run the same four checks CI runs:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

Then check every starter profile with the binary you just built:

```sh
./target/release/zprf check profiles/*.toml
```

## Adding a target

1. Write a [minijinja](https://docs.rs/minijinja) template in `templates/`. The context is built by
   `context()` in `src/render.rs`. `c` holds the interface colours, `syntax` the syntax colours,
   `ansi` the sixteen terminal colours, `styles` the font style of each syntax role and `needs` the
   font, spacing, cursor and motion settings.
2. Add a `Target` entry to `TARGETS` in `src/render.rs`.
3. Draw only pairs that `check` measures. A target that puts a colour on a surface the spec does not
   list can pass `check` and still be unreadable. When an app needs a pair the spec lacks, open an
   issue to add it to the spec first.
4. Write the snapshots with `UPDATE_SNAPSHOTS=1 cargo test --test targets`, read every new file in
   `tests/snapshots/` and load it in the real app.
5. Add the target to the tables in `README.md` and `ACCESSIBILITY.md`.

> [!IMPORTANT]
> Never point a test or a manual run at a real config folder. Render into a temporary folder and
> load the result from there.

## Changing the spec

The format is specified in `spec/SPEC.md` and `spec/profile.schema.json`. `tests/schema.rs` fails
when the schema and the Rust types disagree, so a change to a field touches all three. Follow the
[versioning rules](spec/SPEC.md#versioning): a change that alters what an existing profile means
raises `spec_version`.

## Changing a starter profile

The eight profiles in `profiles/` carry the measured palette of the
[Zacess themes](https://github.com/zacesshq/themes). A colour change belongs in the themes palette
first, then here, so the two stay identical. `tests/profiles.rs` fails if a profile leaves any
colour to derive or misses any floor.

## Project conventions

- **Contrast is a requirement.** A change that lets a pair fall below its floor cannot merge.
- **Fail loudly.** Every error names the field, the colour or the pair and says what to change.
- **Write only where asked.** Nothing in the tool writes outside the path it was given.
- **Colour is never the only signal.** Templates keep each syntax role's font style.

## Using AI

AI tools are welcome within the [AI policy](https://github.com/zacesshq/.github/blob/main/AI_POLICY.md).
**Every pull request must say how AI was used, even when the answer is None.** Write your own issue
and pull request titles and descriptions, understand everything you submit and test it before you
open it.
