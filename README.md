# Zacess Profile

State what you need to read a screen once. Get a checked palette in every terminal and editor.

[![CI](https://github.com/zacesshq/profile/actions/workflows/ci.yml/badge.svg)](https://github.com/zacesshq/profile/actions/workflows/ci.yml)
[![Licence](https://img.shields.io/badge/licence-Apache--2.0-blue.svg)](LICENSE)

`zprf` reads a small TOML file that describes needs: a contrast floor, a font and size,
reduced motion, a cursor size, line spacing and a line length. It derives any colour the file leaves
out, checks every foreground and background pair against WCAG 2.2 and writes config files for
Ghostty, kitty, WezTerm, Alacritty, tmux, Neovim and VS Code.

## Why it exists

Every theming tool we know of stores colours. None of them stores needs. A person who needs 7:1
contrast, no blinking and a 16 point font has to find a theme that happens to meet the first need,
then set the other two in every app by hand, then check nothing slipped when the theme changed.

Zacess Profile turns that round. The profile says what has to be true and the tool refuses to write
anything that breaks it:

- **Needs first, palette second.** A profile can be two lines long. Every colour it leaves out is
  solved in OKLCH to meet the profile's own floors.
- **Fails loudly.** A pair below its floor is named with both colours, the measured ratio and the
  floor it missed. `render` writes nothing until every pair passes.
- **More than colour.** Font, size, line spacing, cursor size, reduced motion and a line length
  ruler are written to each app that has a setting for them.
- **An open format.** The profile is specified in [spec/SPEC.md](spec/SPEC.md) with a
  [JSON Schema](spec/profile.schema.json), versioned apart from the tool, so any app can read a
  profile without it.

```mermaid
flowchart LR
    P["profile.toml<br/>needs plus any colours"] --> D["derive<br/>fill the gaps in OKLCH"]
    D --> C{"check<br/>every pair against<br/>the floors"}
    C -- "a pair fails" --> F["stop and name<br/>the pair and its ratio"]
    C -- "every pair passes" --> R["render<br/>config files for each target"]
    C -- "every pair passes" --> A["apply --live<br/>OSC 4, 10, 11 and 12"]
```

## Install

There are no prebuilt binaries yet. Build from source with a Rust toolchain (1.88 or newer). This
installs the `zprf` command:

```sh
git clone https://github.com/zacesshq/profile.git
cd profile
cargo install --path .
```

## Quick start

Check one of the eight starter profiles, then write its files into a folder of your choosing:

```sh
zprf check profiles/zacess-deep-contrast-dark.toml
zprf render profiles/zacess-deep-contrast-dark.toml --target all --out ./build
```

Or start from needs alone. This profile sets one colour and lets the tool choose the rest:

```toml
spec_version = 1
name = "Warm dark, large text"

[needs]
cursor_size = "large"
line_spacing = 1.3
column_width = 80

[needs.contrast]
text = 7.0

[needs.font]
family = "Atkinson Hyperlegible Mono"
size_pt = 18

[palette]
background = "#1b1612"
```

```sh
zprf derive spec/examples/minimal.toml --out mine.toml
zprf check mine.toml
```

Each colour `derive` chose is marked `# derived`, so you can tell your choices from the tool's.

> [!NOTE]
> `render` only ever writes inside the folder given to `--out`. Copying a file into an app's
> config folder is a step you take yourself. Each rendered file says in its header how to load it.

## Commands

| Command | What it does | Exit code |
| --- | --- | --- |
| `check <profile>...` | Derives any missing colour, then checks every pair of every file given. `--verbose` lists every pair. `--no-derive` also fails a profile that leaves colours to derive | 0 when every profile passes. 1 when a pair fails. 2 when any file is unreadable or invalid or when no colour can be derived to meet a floor. A bad file is reported and the rest are still checked |
| `derive <profile>` | Prints the complete profile with every derived colour marked. `--out <file>` writes it instead | 0 when the result passes. 1 when a colour you set fails, with the profile still printed. 2 when the file is unreadable or invalid or when no colour can be derived to meet a floor |
| `render <profile> --target <name> --out <dir>` | Writes `<dir>/<target>/` from the templates. `--target all` writes every target | 0 when written. 2 when the profile is invalid, cannot be completed or fails a pair, with nothing written |
| `apply <profile> --live` | Sends OSC 4, 10, 11 and 12 to standard output, which recolours the terminal it runs in until the terminal is reset. `--escaped` prints them as readable text with nothing sent, with or without `--live`. `--tmux` wraps them for tmux passthrough | 0 when sent. 2 when the profile is invalid, cannot be completed or fails a pair |
| `list-targets` | Lists every target with the file it writes | 0 |

A mistyped command or option also exits 2.

## What is checked

Every pair is measured with the WCAG 2.2 contrast formula and held to a floor from
`[needs.contrast]`. The defaults follow the Zacess themes.

| Floor | Default | Pairs |
| --- | --- | --- |
| `text` | 7:1 | Text, muted text, links, errors, warnings, information and the accent on the background and on panels. Every syntax colour on the background and on panels. The cursor, the selected text, the text on the accent and the default text over the current line and a soft selection. The default text on any terminal slot listed as a background fill |
| `non_text` | 3:1 | Borders and the focus ring on the background and on panels. The selection block against the background |
| `overlay` | 4.5:1 | Every syntax colour over the current line and over a soft selection. Muted text, links, errors, warnings, information and the accent over the current line |
| `terminal` | 7:1 | The sixteen ANSI colours on the background, apart from slots listed as background fills |
| `terminal_dim` | 4.5:1 | ANSI slots a profile lists in `palette.terminal.dim`, which the light starter profiles use to keep their colours vivid |
| `highlight_distance` | 0.08 | OKLab distance between the background and both the current line and a soft selection, so each stays visible |
| `apca` | off | An optional APCA Lc floor for text on the background, checked on top of WCAG |

## Targets

| Target | App | File written | What it sets |
| --- | --- | --- | --- |
| `ghostty` | Ghostty | `ghostty/<id>` | Palette, cursor, selection, `minimum-contrast` at the text floor, a block cursor kept through shell integration, cursor blink, font, size and cell height |
| `kitty` | kitty | `kitty/<id>.conf` | Palette, cursor, selection, tabs, borders, URL colour, a block cursor kept through shell integration, cursor blink, font, size and cell height |
| `wezterm` | WezTerm | `wezterm/<id>.lua` | Palette, tab bar, cursor style and blink, animation rate, font, size and line height |
| `alacritty` | Alacritty | `alacritty/<id>.toml` | Palette, cursor, selection, cursor shape and blink, font, size and line offset |
| `tmux` | tmux | `tmux/<id>.tmux.conf` | Status bar, windows, pane borders, messages, copy mode and clock |
| `neovim` | Neovim | `neovim/colors/<id>.lua` | Editor, syntax, Tree-sitter, diff and diagnostic groups, terminal colours, cursor blink and a column ruler |
| `vscode` | VS Code | `vscode/<id>.settings.json` | A `settings.json` fragment: interface and terminal colours, token colours with font styles, font, size, line height, cursor, reduced motion, rulers and `terminal.integrated.minimumContrastRatio` |

Templates are [minijinja](https://github.com/mitsuhiko/minijinja) files in [templates](templates).
Adding a target is a template plus one entry in `src/render.rs`. [CONTRIBUTING.md](CONTRIBUTING.md)
has the steps.

## Starter profiles

The eight Zacess themes ship as complete profiles in [profiles](profiles). Each one passes `check`
with no colour left to derive. CI keeps it that way.

| Profile | Background | Text | Text contrast |
| --- | --- | --- | --- |
| `zacess-light` | `#ffffff` | `#111111` | 18.88:1 |
| `zacess-dark` | `#121212` | `#f5f5f5` | 17.18:1 |
| `zacess-yellow-on-black` | `#000000` | `#ffeb3b` | 17.19:1 |
| `zacess-cream` | `#fdf6e3` | `#000000` | 19.46:1 |
| `zacess-high-contrast-dark` | `#000000` | `#ffffff` | 21.00:1 |
| `zacess-high-contrast-light` | `#ffffff` | `#000000` | 21.00:1 |
| `zacess-deep-contrast-dark` | `#000000` | `#ffffff` | 21.00:1 |
| `zacess-deep-contrast-light` | `#ffffff` | `#000000` | 21.00:1 |

## Roadmap

These are planned and not in version 0.1:

- A daemon that watches the profile and the system's light and dark setting, then pushes changes
  to running apps
- A browser extension that applies the profile to every site through CSS custom properties
- System accessibility settings on macOS, GNOME and Windows
- More targets, including JetBrains IDEs, Zed, Helix and Windows Terminal
- Prebuilt binaries and a Homebrew formula
- Colour vision checks on paired tokens, as the Zacess themes already run

## Licence

Zacess Profile is licensed under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for the
copyright notice and <https://www.apache.org/licenses/LICENSE-2.0> for the licence on the web.
