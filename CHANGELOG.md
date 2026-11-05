# Changelog

Every notable change to Zacess Profile is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the tool follows
[Semantic Versioning](https://semver.org/). The profile format has its own version, recorded in
[spec/SPEC.md](spec/SPEC.md).

## [Unreleased]

### Added

- The `zprf` command, built from the `zacess-profile` crate, with a minimum supported Rust of 1.88.
- The profile format, spec version 1 (draft), with a spec document and a JSON Schema in `spec/`.
- A contrast engine with OKLab and OKLCH conversions, WCAG 2.2 contrast ratios and an optional APCA
  floor.
- `check`, which measures every foreground and background pair against the profile's floors and
  names each pair that misses.
- `derive`, which fills every missing colour in OKLCH to meet the profile's floors and marks each one
  it chose.
- `render` with targets for Ghostty, kitty, WezTerm, Alacritty, tmux, Neovim and VS Code, written
  from minijinja templates.
- `apply --live`, which recolours the running terminal with OSC 4, 10, 11 and 12.
- `list-targets`.
- The eight Zacess themes as starter profiles.
- Unit tests on the colour maths, an integration test over every shipped profile, a snapshot per
  target and CI on Ubuntu and macOS.
