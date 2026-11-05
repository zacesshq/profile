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
- `check` also measures every [paired colour](spec/SPEC.md#paired-colours): neighbouring syntax
  roles, the three diff colours and errors and warnings. Each pair must differ in tone and stay
  apart in OKLab under simulated protanopia, deuteranopia and tritanopia, with the same matrices and
  floors as the Zacess themes. `derive` spreads the colours it chooses until every pair passes or
  names the pair it cannot separate.
- `render`, `watch` and `apply` refuse a profile that fails any pair. Nothing is written or sent.
  When `watch` sees a change that fails, it says so and leaves the last good files in place.
- `render` and `watch` write only inside the folder given to `--out`. They never edit an app's real
  config.
- `os --print` only prints. It has no option that applies a setting and a test fails the build if
  its code ever runs a program or touches a file.
- The browser extension draws the profile's colours only when every pair it uses is set and meets
  its floor: text and links on the background and on panels, selected text, text on the accent and
  borders and the focus ring. Otherwise it leaves the page's colours alone and says why.
- Every pair a template draws is a pair `check` measures, so a profile that passes is not drawn
  with an unmeasured pair in any target.
- Every shipped profile is complete and passes. CI runs the same check on every pull request.

## The starter profiles

The lowest measured value in each class of pair, from `zprf check --verbose`. Interface text
includes the accent wherever it is drawn as text. Panels are both panel colours, `surface` and
`surface_alt`. A highlight is the current line or a soft selection. Every
syntax colour keeps its font style in every target (bold keywords, control flow and tags, italic
comments and parameters, underlined invalid code), so no role rests on colour alone.

| Profile | Text on background | Lowest syntax colour | Lowest interface text | Lowest over a highlight | Selection | Cursor |
| --- | --- | --- | --- | --- | --- | --- |
| Zacess Light | 18.88:1 | 7.08:1 (comment on panels) | 7.04:1 (warning on panels) | 4.59:1 (comment on a soft selection) | 7.52:1 | 18.88:1 |
| Zacess Dark | 17.18:1 | 7.21:1 (comment on panels) | 8.04:1 (link on panels) | 4.57:1 (comment on a soft selection) | 8.97:1 | 13.08:1 |
| Zacess Yellow on Black | 17.19:1 | 9.03:1 (comment on panels) | 12.29:1 (error on panels) | 4.60:1 (comment on a soft selection) | 9.00:1 | 17.19:1 |
| Zacess Cream | 19.46:1 | 7.24:1 (added on panels) | 7.04:1 (warning on panels) | 4.58:1 (added on a soft selection) | 7.48:1 | 19.46:1 |
| Zacess High Contrast Dark | 21.00:1 | 9.09:1 (comment on panels) | 9.64:1 (error on panels) | 4.59:1 (comment on a soft selection) | 9.02:1 | 12.77:1 |
| Zacess High Contrast Light | 21.00:1 | 7.50:1 (comment on panels) | 8.58:1 (link on panels) | 4.61:1 (comment on a soft selection) | 7.50:1 | 21.00:1 |
| Zacess Deep Contrast Dark | 21.00:1 | 8.49:1 (property on panels) | 7.63:1 (information on panels) | 4.60:1 (property on a soft selection) | 9.01:1 | 19.55:1 |
| Zacess Deep Contrast Light | 21.00:1 | 7.71:1 (number) | 7.00:1 (link) | 4.88:1 (link on the current line) | 7.47:1 | 21.00:1 |

| Profile | Lowest border, focus ring or selection block | Lowest terminal colour | Current line from the background (OKLab) |
| --- | --- | --- | --- |
| Zacess Light | 4.22:1 (border on panels) | 4.67:1 (bright green, a dim slot) | 0.105 |
| Zacess Dark | 3.66:1 (border on panels) | 7.42:1 (black) | 0.122 |
| Zacess Yellow on Black | 4.52:1 (border on panels) | 8.32:1 (black) | 0.268 |
| Zacess Cream | 4.57:1 (border on panels) | 4.67:1 (bright green, a dim slot) | 0.105 |
| Zacess High Contrast Dark | 7.87:1 (border on panels) | 10.01:1 (black) | 0.268 |
| Zacess High Contrast Light | 7.50:1 (selection block) | 4.67:1 (bright green, a dim slot) | 0.106 |
| Zacess Deep Contrast Dark | 9.01:1 (selection block) | 7.01:1 (blue) | 0.268 |
| Zacess Deep Contrast Light | 6.60:1 (focus ring) | 4.52:1 (yellow, a dim slot) | 0.134 |

The closest paired colours in each profile. Of the 31 pairs, 19 apply to each starter profile: the
rest join roles with different font styles.

| Profile | Smallest tone difference (floor 0.05) | Smallest distance under a simulation (floor 0.04) |
| --- | --- | --- |
| Zacess Light | 0.053 (function and string) | 0.041 (error and warning, deuteranopia) |
| Zacess Dark | 0.052 (error and warning) | 0.050 (error and warning, tritanopia) |
| Zacess Yellow on Black | 0.051 (error and warning) | 0.040 (string and number, deuteranopia) |
| Zacess Cream | 0.053 (function and string) | 0.041 (error and warning, deuteranopia) |
| Zacess High Contrast Dark | 0.053 (function and property) | 0.041 (function and property, deuteranopia) |
| Zacess High Contrast Light | 0.052 (function and type) | 0.041 (error and warning, deuteranopia) |
| Zacess Deep Contrast Dark | 0.052 (added and removed) | 0.045 (string and number, tritanopia) |
| Zacess Deep Contrast Light | 0.051 (control and keyword) | 0.043 (error and warning, protanopia) |

> [!NOTE]
> The Zacess themes do not pair errors with warnings. In five of their palettes the two share a tone
> and merge under a simulated deficiency, so Zacess Light, Dark, Cream, High Contrast Light and Deep
> Contrast Light move the warning colour in lightness on its own hue. Each file names the themes'
> colour beside the new one. Every other colour matches the themes.

> [!NOTE]
> The four light profiles list their twelve coloured terminal slots in `palette.terminal.dim`, so
> those slots are held to 4.5:1 rather than 7:1. That is the Zacess themes' own allowance, kept so
> red, green and blue stay recognisable on white. Set `terminal_dim = 7.0` in your copy to hold every
> slot to 7:1. `check` will then name each slot that misses.

## Needs each target applies

"No setting" means the app has nothing to set for that need. "No" means it has one that the target
does not write yet.

| Need | Ghostty | kitty | WezTerm | Alacritty | tmux | Neovim | VS Code |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Colours | yes | yes | yes | yes | interface only | yes | yes |
| Font family and size | yes | yes | yes | yes | no setting | no setting | yes |
| Line spacing | cell height | cell height | line height | line offset | no setting | no setting | line height |
| Character spacing | no | no | no | no | no setting | no setting | yes |
| Reduced motion | cursor blink off | cursor blink, trail and bell off | steady cursor, text blink off, animation at 1 frame per second, bell fade off | cursor blink off | visual bell off | cursor blink off | `workbench.reduceMotion`, cursor blink, caret animation and smooth scrolling off |
| Large cursor | block cursor, kept at the prompt with `shell-integration-features = no-cursor` | block cursor, kept at the prompt with `shell_integration no-cursor`, beam width in points | steady block cursor, bar width in pixels | block cursor | no setting | no | block cursor, cursor width |
| Focus ring width | no setting: Ghostty marks the focused split by dimming the others | `window_border_width`, the border of the active window | no setting: WezTerm marks the focused pane by dimming the others | no setting: one window, no panes | `pane-border-lines heavy` at 2 pixels or more, since borders are drawn in character cells | no setting: window separators are characters | no setting: VS Code sets the focus colour but not its width |
| One column | no setting: splits are made by hand | `enabled_layouts vertical,stack` | no setting: splits are made by hand | no setting: one window, no panes | no setting: splits are made by hand | diffs open one above the other (`diffopt` `horizontal`) | editors opened to the side go below (`workbench.editor.openSideBySideDirection`) and diffs are inline |
| Hide decoration | a solid background with no blur | a solid background with no blur or image | a solid background with no blur or image | a solid background with no blur, unless the main config sets its own opacity | no setting | no tildes past the end of a buffer and no see-through floating windows or menus | no minimap |
| Line length | no | no | no | no | no | `colorcolumn` | `editor.rulers` |
| The text floor for programs' own colours | `minimum-contrast` | no setting | no setting | no setting | no setting | no | `terminal.integrated.minimumContrastRatio` |

### The browser extension

| Need | What the extension does |
| --- | --- |
| Colours | The background, text, links (underlined), panels behind form controls, the selection, marked text and borders, all as `--zprf-*` custom properties, when every pair it uses passes |
| Font family | Every element apart from media, icon fonts and, unless the profile font is monospace, code |
| Font size | A minimum: body text and text elements are raised to the profile size and larger text such as a heading keeps its size |
| Line and character spacing | Every element apart from media |
| Line length | Paragraphs, list items, quotes and captions are at most `column_width` characters wide |
| Focus ring | A solid outline of `focus_ring_width_px` (2 pixels when unset) in the profile's focus colour on whatever matches `:focus-visible`. A control the page draws itself without real focus gets no ring. When the colours are not applied it takes the element's text colour |
| Reduced motion | CSS animations end at once, no CSS transitions start and scrolling is never smooth. Video, animated images and motion a page drives from script are not stopped |
| One column | Text set in newspaper columns is shown as one column |
| Hide decoration | No background images, text shadows or box shadows |
| Large cursor | No setting: pages cannot widen the text cursor |

When `reduce_motion` is off a target leaves the app's own blink setting alone rather than turning
blinking on. When `cursor_size` is missing or `standard` the app keeps its own cursor shape.

## Known limits

- CI does not load the rendered files in each app. For this change every starter profile and both
  examples were rendered and the Ghostty output passed `ghostty +validate-config` in Ghostty 1.4.0,
  the kitty output loaded in kitty 0.50.0's own config parser with no bad or unknown lines, the
  Neovim output loaded in Neovim 0.12.5, the tmux output was sourced by tmux 3.7c and the WezTerm
  module ran under LuaJIT. The VS Code and Alacritty output parse as JSON and TOML in the tests.
- Terminal colours are not yet checked against the default text in tone and under each simulation,
  as the Zacess themes check them. That needs the spec to say which slots stand for the default
  text. See [ROADMAP.md](ROADMAP.md).
- On a mid grey the band of lightness left above the text floor is too narrow for four
  regular-style syntax roles to sit 0.05 apart in tone, so `derive` names the paired roles and the
  background instead of choosing colours. In a sweep of greys and tinted greys this happened from
  about `#373737` to `#5f5f5f` and from `#919191` to `#a5a5a5` at a 4.5:1 text floor. At 7:1 it
  happened from about `#2d2d2d` to `#5f5f5f` and from `#919191` to `#afafaf`. Set those colours by hand,
  move the background towards black or white or lower `tone_difference` in your copy. Most backgrounds
  between those ranges fail on contrast instead, since almost no text colour reaches the floor
  there. `derive` names the floor it cannot meet.
- When an `apca` floor is set, `derive` aims for it only in the syntax colours, which are the
  colours `check` measures with APCA besides the default text.
- Alacritty reads its imports first and its own `alacritty.toml` after them, so an `opacity` or any
  other setting the main file sets itself still wins over the rendered import. Its `blur` works on
  macOS and KDE only.
- `watch` looks for a change every half second rather than being told of one. It reads the system
  appearance the same way. It writes files only: an app that is already open needs reloading
  to pick up a change.
- `os --print` maps each need to the closest system setting it knows. The text size scale is the
  profile size over 11 points on GNOME and over 9 points on Windows. Increased contrast is suggested
  whenever the text floor is 7:1 or more. Read every line before running it.
- The browser extension's styles are page styles marked important, so a page's own important rule
  that is more specific or comes later can still win, for example one that removes focus outlines.
  Moving them to the browser's user style level is on the [roadmap](ROADMAP.md).
- With the colours applied, every element's own background is cleared so the profile background
  shows through. A fixed header, a tooltip or a menu without a dialog, menu or listbox role then
  sits over the page text with no panel behind it. Text over a background image keeps the
  image. Set `hide_decoration` to remove background images.
- The browser extension cannot reach inside shadow roots, so a page built from web components keeps
  some of its own styles. Text sized in fixed pixels below the minimum is raised only on the text
  elements it lists. With reduced motion, a page that waits for a transition to end before showing
  something can wait forever, since no transition starts.
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
