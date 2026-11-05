# Changelog

Every notable change to Zacess Profile is listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the tool follows
[Semantic Versioning](https://semver.org/). The profile format has its own version, recorded in
[spec/SPEC.md](spec/SPEC.md).

## [Unreleased]

### Added

- Colour vision checks. `check` now measures paired colours: the neighbouring syntax roles the
  Zacess themes pair, added, removed and changed lines and errors and warnings. Each pair must
  differ in tone (`needs.contrast.tone_difference`, 0.05) and stay apart in OKLab under simulated
  protanopia, deuteranopia and tritanopia (`needs.contrast.colour_vision_distance`, 0.04), using
  the Machado, Oliveira and Fernandes (2009) matrices the themes use. A pair of syntax roles with
  different font styles is told apart by style and is not checked, except for the pairs that are
  often red and green. A failing pair is reported like a contrast failure, naming both colours, the
  simulation and the distance.
- `derive` spreads the colours it chooses so every paired role meets the new floors, moving each on
  its own hue to the nearest lightness that works.
- An optional `surface_alt` palette role, the second panel colour some apps draw behind activity
  and status bars, inactive tabs and list rows. When it is set, `check` measures on it every pair it
  measures on `surface`: the default text, muted text, links, errors, warnings, information, the
  accent and every syntax colour at the text floor, with borders and the focus ring at the non-text
  floor. `derive` solves the colours it chooses against it as well but never fills it in. A profile
  that leaves it out renders it as `surface` and gains no pairs, so existing profiles stay valid.
  This is a minor change to spec version 1.
- `zprf watch`, which renders a profile again each time it changes. When the profile comes in a dark
  and a light variant it follows the system's appearance on macOS, Windows and freedesktop
  desktops and writes both under one id. A change that fails a pair is reported and the last good
  files stay in place. It writes only inside `--out`.
- `zprf os --print`, a preview of the system accessibility settings a profile maps to, as
  `defaults write` lines on macOS, `gsettings set` lines on GNOME and `reg add` values with their
  Settings page on Windows. It never applies anything and has no option to.
- A browser extension for Chrome and Firefox in `extension/` (Manifest V3, TypeScript, no
  framework). It applies a profile's colours, font, minimum text size, spacing, line length, focus
  ring and reduced motion to every page through CSS custom properties. Its options page reads a
  pasted or imported TOML profile and checks it against the JSON Schema. Not yet in any store.
- `focus_ring_width_px` sets the focus border in kitty and heavy pane borders in tmux.
  `single_column` sets kitty's layouts, stacked diffs in Neovim and editors and diffs below one
  another in VS Code. `hide_decoration` gives Ghostty, kitty, WezTerm and Alacritty a solid
  background, turns off see-through windows in Neovim and hides the minimap in VS Code.
- `spec/examples/every-need.toml`, a profile with every need set, rendered as a snapshot for each
  target.
- CI on Windows, a build of each release binary on every pull request and a job that builds the
  extension, loads it in Chromium with Playwright and lints the Firefox build with `web-ext`.
- A release workflow that attaches a binary for macOS (arm64 and x86_64), Linux (x86_64 and aarch64)
  and Windows (x86_64) with its SHA-256 sum to each release, then publishes the crate to crates.io
  through Trusted Publishing, skipping a version crates.io already has.
- A Homebrew formula and winget manifests in `packaging/`, with a script that fills in their sums.
- `ROADMAP.md`.
- The Zacess Profile mark in `assets/` in light, dark, automatic and single colour versions, with a
  light and a dark lockup at the head of the README. The extension carries its icons at 16, 32, 48
  and 128 pixels, plus toolbar icons for Firefox's dark themes. `extension/store/` holds the
  promotional tile and a real 1280 by 800 screenshot made by `npm run store:screenshot`.

### Changed

- The warning colour of Zacess Light, Dark, Cream, High Contrast Light and Deep Contrast Light
  changes, because it matched the error colour in tone and merged with it under simulated colour
  vision deficiencies. On the four light profiles it becomes a burnt orange, the most colourful
  shade that still meets 7:1. Zacess Dark keeps its hue and moves in lightness. The colours match
  zacesshq/themes.
- Every starter profile states the two new floors.
- Every starter profile sets `surface_alt` from the Zacess themes palette, so the colours drawn on
  the second panel colour are measured. Each profile now measures 224 pairs.
- The VS Code activity bar, status bar and title bar, the kitty and WezTerm inactive tabs and the
  Neovim status line are drawn on `surface_alt`, as the Zacess themes draw them.
- `check` names the colour vision floors in its summary line.
- A profile that leaves its syntax colours to be derived now fails on a dark grey from about
  `#2d2d2d` to `#414141` at a 7:1 text floor, tinted greys of the same lightness included, because
  four regular-style roles cannot sit 0.05 apart in tone in the band left above the floor. `derive`
  names the paired roles and the background. Set those colours by hand, choose a darker background
  or lower `tone_difference`. In a sweep of 416 backgrounds at 4.5:1 and 7:1, these were 9 of the
  112 that derived before.
- `needs.contrast.check_error_warning`, on by default, decides whether errors and warnings are held
  to the paired colour floors.

### Fixed

- `derive` now solves the colours it chooses against a current line or a soft selection the
  profile sets, at the overlay floor. Before, a derived colour could sit at 4.3:1 over a current
  line of `#333333` on black. In a sweep, 207 of 208 backgrounds tried at a 4.5:1 text floor
  failed to derive for the same reason. In the same
  sweep, 129 backgrounds that failed before now derive.
- `render` and `watch` refuse to write over the profile they read or over the other variant,
  whatever folders or links lie in between.
- `render` and `watch` write each file beside its final name and then rename it into place, so an
  app never reads half a file.
- `watch` renders the matching variant when the other one is broken and says which is broken. A
  write that fails is tried again on the next look rather than waiting for the profile to change.
- `check --verbose` names a failing pair's floor once.

## [0.1.0] - 2026-11-05

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

### Changed

- Routine tidy-up.

[Unreleased]: https://github.com/zacesshq/profile/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/zacesshq/profile/releases/tag/v0.1.0
