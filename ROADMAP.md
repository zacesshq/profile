# Roadmap

What Zacess Profile does today, what we are working on next and what we plan after that. Items move
up this page as they ship. [CHANGELOG.md](CHANGELOG.md) records exactly what changed in each
release.

## Done

### In version 0.1.0

- The profile format, spec version 1 (draft), with a spec document and a JSON Schema in
  [spec](spec)
- A contrast engine with OKLab and OKLCH, WCAG 2.2 ratios and an optional APCA floor
- `check`, `derive`, `render`, `apply --live` and `list-targets`
- Targets for Ghostty, kitty, WezTerm, Alacritty, tmux, Neovim and VS Code
- The eight Zacess themes as starter profiles

### In version 0.2.0

- Colour vision checks: paired syntax roles, added and removed lines and errors and warnings must
  differ in tone and stay apart under simulated protanopia, deuteranopia and tritanopia. `derive`
  spreads the colours it chooses to meet them
- `focus_ring_width_px`, `single_column` and `hide_decoration` reach every target with a real setting
  for them. [ACCESSIBILITY.md](ACCESSIBILITY.md#needs-each-target-applies) says where an app has none
- `zprf watch`, which renders again when a profile changes and follows the system's light or dark
  setting when a profile comes in both variants
- A browser extension for Chrome and Firefox in [extension](extension)
- `zprf os --print`, a preview of the system settings a profile maps to on macOS, GNOME and Windows
- CI on Windows, a build of every release binary on each pull request, a release workflow and the
  Homebrew and winget packaging in [packaging](packaging)
- The first release with binaries for macOS, Linux and Windows, on crates.io through Trusted
  Publishing and in `zacesshq/homebrew-tap`
- An optional `surface_alt` role for the second panel colour, measured like `surface`. The warning
  colours of five starter profiles match the Zacess themes, where errors and warnings now stay apart
  under colour vision deficiencies

### In version 0.2.1

- Releases that update every store by themselves: crates.io, the Homebrew tap, winget and
  [zacess.com](https://zacess.com/profile/), which now has a page and full documentation for
  Zacess Profile

### In version 0.2.2

- The browser extension is submitted to the Chrome Web Store, Firefox Add-ons and Microsoft Edge
  Add-ons, with each listing linked from the README

## Next

- **winget.** The first submission, `Zacess.Profile` 0.2.0, is waiting for review in
  microsoft/winget-pkgs. Once it is accepted, each release opens the next one by itself
- **Extension store listings.** Version 0.2.1 is in review on the Chrome Web Store, Firefox
  Add-ons and Microsoft Edge Add-ons. Each listing is linked from the
  [README](README.md#browser-extension) and opens once its review is done
- **Terminal colours against the default text.** The Zacess themes also check each terminal colour
  against the default text in tone and under each simulation. That needs two new spec fields first:
  the slots that stand for the default text and the slots allowed to sit near it
- **Extension styles at the user level.** Inserting the style sheet as user styles through the
  scripting API would let it beat a page's own important rules. Giving fixed and floating
  elements the panel colour would keep them readable over the page
- **Live updates in running apps.** `watch` writes files. Pushing a change into apps that are already
  open (OSC sequences to terminals, kitty remote control and a message to a running Neovim) is the
  next step for it

## Later

- More targets, including JetBrains IDEs, Zed, Helix and Windows Terminal
- The extension talking to `zprf watch` over native messaging, so one profile change reaches the
  terminal, the editor and the browser together
- A Safari build of the extension
- Readers for the spec in other languages, starting from the TypeScript reader in the extension
- Spec version 1 marked final once the colour vision pairs have settled

## Not planned

- Applying system settings from `zprf`. `zprf os` stays a preview: the commands it prints are for a
  person to read and run, since a wrong system setting can lock someone out of their own screen
- Sending a profile anywhere. Profiles stay on the device they are made on
