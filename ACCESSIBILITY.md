# Accessibility

Zacess Profile exists so that a person's reading needs are written down once and kept everywhere.
This file says what the tool does today, what it measures, what it cannot promise yet and how to
report a barrier.

## What the tool guarantees

- `check` measures every pair listed in the [spec](spec/SPEC.md#pairs) with the WCAG 2.2 contrast
  formula. A pair passes only when its unrounded ratio meets the floor. Printed ratios are rounded
  down, so a figure never overstates the real one.
- `derive` keeps every colour a profile sets. Colours it finds by a lightness search aim 0.15 above
  their floor, so rounding to hex cannot drop them under it. Others are copies (a link that follows
  the accent) or fallbacks (a current line that cannot move far from the background). Before handing a profile back, `derive` checks every pair a colour it chose takes part
  in. A failing pair against the background or another chosen colour stops it with the colour and
  the floor named. A failing pair against a colour you set is reported by `check` as that pair.
- `render` and `apply` refuse a profile that fails any pair. Nothing is written or sent.
- `render` writes only inside the folder given to `--out`. It never edits an app's real config.
- Every pair a template draws is a pair `check` measures, so a profile that passes is not drawn
  with an unmeasured pair in any target.
- Every shipped profile is complete and passes. CI runs the same check on every pull request.

## The starter profiles

The lowest measured value in each class of pair, from `zprf check --verbose`. Interface text
includes the accent wherever it is drawn as text. A highlight is the current line or a soft
selection. Every
syntax colour keeps its font style in every target (bold keywords, control flow and tags, italic
comments and parameters, underlined invalid code), so no role rests on colour alone.

| Profile | Text on background | Lowest syntax colour | Lowest interface text | Lowest over a highlight | Selection | Cursor |
| --- | --- | --- | --- | --- | --- | --- |
| Zacess Light | 18.88:1 | 7.53:1 (comment on panels) | 7.69:1 (link on panels) | 4.59:1 (comment on a soft selection) | 7.52:1 | 18.88:1 |
| Zacess Dark | 17.18:1 | 7.98:1 (comment on panels) | 8.91:1 (link on panels) | 4.57:1 (comment on a soft selection) | 8.97:1 | 13.08:1 |
| Zacess Yellow on Black | 17.19:1 | 9.76:1 (comment) | 13.28:1 (error) | 4.60:1 (comment on a soft selection) | 9.00:1 | 17.19:1 |
| Zacess Cream | 19.46:1 | 7.92:1 (added on panels) | 8.02:1 (link on panels) | 4.58:1 (added on a soft selection) | 7.48:1 | 19.46:1 |
| Zacess High Contrast Dark | 21.00:1 | 9.95:1 (comment) | 10.57:1 (error) | 4.59:1 (comment on a soft selection) | 9.02:1 | 12.77:1 |
| Zacess High Contrast Light | 21.00:1 | 8.05:1 (comment on panels) | 9.21:1 (link on panels) | 4.61:1 (comment on a soft selection) | 7.50:1 | 21.00:1 |
| Zacess Deep Contrast Dark | 21.00:1 | 9.01:1 (property) | 8.10:1 (information) | 4.60:1 (property on a soft selection) | 9.01:1 | 19.55:1 |
| Zacess Deep Contrast Light | 21.00:1 | 7.71:1 (number) | 7.00:1 (link) | 4.88:1 (link on the current line) | 7.47:1 | 21.00:1 |

| Profile | Lowest border, focus ring or selection block | Lowest terminal colour | Current line from the background (OKLab) |
| --- | --- | --- | --- |
| Zacess Light | 4.50:1 (border on panels) | 4.67:1 (bright green, a dim slot) | 0.105 |
| Zacess Dark | 4.05:1 (border on panels) | 7.42:1 (black) | 0.122 |
| Zacess Yellow on Black | 4.89:1 (border) | 8.32:1 (black) | 0.268 |
| Zacess Cream | 5.00:1 (border on panels) | 4.67:1 (bright green, a dim slot) | 0.105 |
| Zacess High Contrast Dark | 8.62:1 (border) | 10.01:1 (black) | 0.268 |
| Zacess High Contrast Light | 7.50:1 (selection block) | 4.67:1 (bright green, a dim slot) | 0.106 |
| Zacess Deep Contrast Dark | 9.01:1 (selection block) | 7.01:1 (blue) | 0.268 |
| Zacess Deep Contrast Light | 6.60:1 (focus ring) | 4.52:1 (yellow, a dim slot) | 0.134 |

> [!NOTE]
> The four light profiles list their twelve coloured terminal slots in `palette.terminal.dim`, so
> those slots are held to 4.5:1 rather than 7:1. That is the Zacess themes' own allowance, kept so
> red, green and blue stay recognisable on white. Set `terminal_dim = 7.0` in your copy to hold every
> slot to 7:1. `check` will then name each slot that misses.

## Needs each target applies

| Need | Ghostty | kitty | WezTerm | Alacritty | tmux | Neovim | VS Code |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Colours | yes | yes | yes | yes | interface only | yes | yes |
| Font family and size | yes | yes | yes | yes | no setting | no setting | yes |
| Line spacing | cell height | cell height | line height | line offset | no setting | no setting | line height |
| Character spacing | no | no | no | no | no setting | no setting | yes |
| Reduced motion | cursor blink off | cursor blink, trail and bell off | steady cursor, text blink off, animation at 1 frame per second, bell fade off | cursor blink off | visual bell off | cursor blink off | `workbench.reduceMotion`, cursor blink, caret animation and smooth scrolling off |
| Large cursor | block cursor, kept at the prompt with `shell-integration-features = no-cursor` | block cursor, kept at the prompt with `shell_integration no-cursor`, beam width in points | steady block cursor, bar width in pixels | block cursor | no setting | no | block cursor, cursor width |
| Line length | no | no | no | no | no | `colorcolumn` | `editor.rulers` |
| The text floor for programs' own colours | `minimum-contrast` | no setting | no setting | no setting | no setting | no | `terminal.integrated.minimumContrastRatio` |

When `reduce_motion` is off a target leaves the app's own blink setting alone rather than turning
blinking on. When `cursor_size` is missing or `standard` the app keeps its own cursor shape.

## Known limits

- CI does not load the rendered files in each app. Before this release the Ghostty output passed
  `ghostty +validate-config` in Ghostty 1.4.0, the kitty output loaded in kitty 0.50.0's own
  config parser with no bad lines, the Neovim output loaded in Neovim, the tmux output was sourced
  by tmux 3.7 and the WezTerm module ran under LuaJIT. The VS Code and Alacritty output parse as
  JSON and TOML in the tests.
- `derive` does not yet test paired syntax colours under simulated colour vision deficiencies, as
  the Zacess themes do. Two derived roles can sit close in hue. The starter profiles carry the
  themes' measured palette, which passed those checks.
- When an `apca` floor is set, `derive` aims for it only in the syntax colours, which are the
  colours `check` measures with APCA besides the default text.
- `focus_ring_width_px`, `single_column` and `hide_decoration` are part of the spec but no target in
  version 0.1 has a setting for them.
- VS Code cannot colour selected text in a normal theme, so its fragment uses the soft selection,
  over which every syntax colour keeps the `overlay` floor.
- The VS Code fragment is layered over whichever colour theme is active. Every colour it sets is
  checked, but any interface colour it does not set, such as a notification or a hover, still comes
  from that theme and is not checked. Pick a base theme close to the profile, such as one of the
  Zacess themes.
- `apply --live` lasts until the terminal resets its colours, often when it is restarted. It
  changes only the terminal it is run in.

## Preferences

Many settings are preferences. Change them in your own copy of a profile: a lower or higher floor, a
different font, motion back on. Open an issue or a pull request if a change would help others.

## Reporting a barrier

An accessibility problem is a bug. Open an issue with the
[accessibility form](https://github.com/zacesshq/profile/issues/new?template=accessibility.yml).
You never need to share a diagnosis or anything personal. To report privately, email
[contact@zacess.com](mailto:contact@zacess.com).

## The shared statement

The organisation's [accessibility statement](https://github.com/zacesshq/.github/blob/main/ACCESSIBILITY.md)
covers every Zacess repository. This file takes precedence where the two differ.
