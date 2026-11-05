# Zacess Profile specification

**Spec version 1, draft.** This document is versioned apart from `zprf`, the reference tool in the `zacess-profile` crate. The
tool's version is in `Cargo.toml` and its changes are in [CHANGELOG.md](../CHANGELOG.md). A change to
what a valid profile is raises the spec version, as does a change to what a field means.

The machine-readable form is [profile.schema.json](profile.schema.json), a JSON Schema (draft
2020-12). Where this document and the schema differ, the schema is a bug and this document wins.

## Purpose

A profile records what a person needs to read a screen comfortably: contrast floors, a font and size,
spacing, cursor size and freedom from motion. It may also carry colours. An app that reads a profile
applies the needs it can and leaves the rest alone. A tool that writes colours for an app must not
write any pair that falls below the profile's floors.

## File format

A profile is a [TOML 1.0](https://toml.io/en/v1.0.0) file encoded as UTF-8. The conventional file
extension is `.toml`. The schema describes the same structure as JSON, so an app may store or send a
profile as JSON and validate it against the schema. `zprf` itself reads and writes TOML only.

Unknown fields are errors. A reader must refuse a profile with a field it does not know, rather than
ignore it, so that a misspelt need such as `reduse_motion` is never silently dropped.

A reader must refuse a profile whose `spec_version` is newer than the newest version it supports.

## Top level

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `spec_version` | integer | yes | The spec version the profile follows. `1` for this document |
| `name` | string | yes | The name shown to the person. One line: it must hold at least one character that is not white space and no control characters (U+0000 to U+001F and U+007F to U+009F). When `id` is missing it must also hold an ASCII letter or digit, since the file names come from it |
| `id` | string | no | Lowercase ASCII letters and digits joined by single dashes, matching `^[a-z0-9]+(-[a-z0-9]+)*$`, used to name files. When missing, it is the name in lowercase with every run of other characters replaced by one dash |
| `description` | string | no | One sentence on who the profile is for |
| `appearance` | `dark` or `light` | no | Whether the background is dark or light. When missing, it is worked out from the background: dark when white has more contrast with it than black does. A profile whose appearance disagrees with its background is invalid |
| `needs` | table | no | See [Needs](#needs) |
| `palette` | table | no | See [Palette](#palette) |

## Needs

Every field is optional. A missing field means the person has no need there, so an app keeps its own
setting, except where a default is given.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `contrast` | table | see below | The floors in [Contrast](#contrast) |
| `font.family` | string | none | A font family name with no double quotes, backslashes or control characters, since writers place it inside quoted strings. Empty or missing leaves each app on its own font |
| `font.fallback` | `sans_serif`, `serif` or `monospace` | none | The generic family to use when the font is not installed |
| `font.size_pt` | number, 6 to 96 | none | Text size in points. An app that sizes text in CSS pixels uses four thirds of this value |
| `reduce_motion` | boolean | `true` | No blinking cursors, animation, smooth scrolling or flashing bells |
| `cursor_size` | `standard`, `large` or `extra_large` | none | Large and extra large ask for a block cursor. Where only a bar is offered, the bar is two or three times the standard width |
| `focus_ring_width_px` | integer, 1 to 10 | none | The width of the keyboard focus outline in CSS pixels, for apps that can set it |
| `line_spacing` | number, 1 to 3 | none | Line height as a multiple of the font size |
| `character_spacing` | number, 0 to 0.5 | none | Extra space between letters as a fraction of the font size |
| `column_width` | integer, 20 to 150 | none | The longest comfortable line in characters. Editors show it as a ruler |
| `single_column` | boolean | none | One column with nothing side by side. For apps with a layout to change |
| `hide_decoration` | boolean | none | Hide decorative images and borders that carry no meaning. For apps with decoration to hide |

`reduce_motion` defaults to `true` because animation can be a barrier and its absence rarely is.

### Contrast

Ratios use the WCAG 2.2 definitions of relative luminance and contrast ratio, from 1 to 21. A pair
passes when its unrounded ratio is at least the floor. A reader that prints a ratio should round it
down, so a printed figure never overstates the real one.

| Field | Default | Applies to |
| --- | --- | --- |
| `text` | `7.0` | Text, the accent and syntax colours on the background and on panels. The cursor on the background, the character under a block cursor, selected text on the selection block, text on the accent and the default text over the current line and a soft selection. The default text on a terminal slot used as a fill |
| `non_text` | `3.0` | Borders and the focus ring on the background and on panels. The selection block against the background |
| `overlay` | `4.5` | Syntax colours over the current line and over a soft selection. Interface text and the accent over the current line |
| `terminal` | `7.0` | Each of the sixteen terminal colours on the background |
| `terminal_dim` | `4.5` | Terminal slots listed in `palette.terminal.dim` |
| `highlight_distance` | `0.08` | The OKLab distance from the background to the current line and to a soft selection, so neither disappears |
| `colour_vision_distance` | `0.04` | The OKLab distance between two [paired colours](#paired-colours) as a person with protanopia, deuteranopia or tritanopia sees them |
| `tone_difference` | `0.05` | The difference in tone between two paired colours. Tone is the cube root of relative luminance, the OKLab lightness of the grey with the same luminance, which is all a viewer with no colour vision has to go on |
| `check_error_warning` | `true` | Whether `error` and `warning` are held to the paired colour floors as a pair. The Zacess themes do not pair them, so `false` keeps a themes palette as it is |
| `apca` | none | When set, an APCA Lc floor (absolute value, APCA-W3 0.0.98G-4g) for the default text and every syntax colour on the background, on top of the WCAG ratio |

The defaults are WCAG 2.2 AAA for text (success criterion 1.4.6) and the WCAG 2.2 floor for
non-text contrast (success criterion 1.4.11).

## Palette

Every colour is a string in the form `#rrggbb`, in sRGB. Every colour is optional. A colour that is
missing is derived (see [Derivation](#derivation)) and a colour that is present is never changed.
The one exception is `surface_alt`, which is never derived: when it is missing it takes the value of
`surface` wherever it is drawn and adds no pairs.

### Interface colours

These sit directly in the `[palette]` table.

| Field | Meaning |
| --- | --- |
| `background` | The main background every other colour is measured on |
| `foreground` | Default text |
| `surface` | Panels, bars, sidebars and floating windows |
| `surface_alt` | The second panel colour, for apps that draw two: activity and status bars, inactive tabs and list rows. Optional. Falls back to `surface` when missing |
| `muted` | Secondary text such as line numbers and descriptions |
| `border` | Borders and dividers |
| `accent` | The one accent: the active tab, the current search match and buttons |
| `accent_foreground` | Text drawn on the accent |
| `link` | Links |
| `focus_ring` | The outline that shows keyboard focus |
| `cursor` | The text cursor |
| `cursor_text` | The character under a block cursor |
| `selection_background` | The selection block, where selected text takes one colour |
| `selection_foreground` | Selected text on the selection block |
| `selection_soft` | A background-only selection, where selected text keeps its own colours |
| `line_highlight` | The band behind the current line |
| `error`, `warning`, `info` | Message text |

### Syntax colours

These sit in `[palette.syntax]`. Each role has a font style that a writer of an editor theme should
apply, so that no role is told apart by colour alone.

| Field | Style | Meaning |
| --- | --- | --- |
| `keyword` | bold | Declarations and other keywords such as `const`, `class` and `fn` |
| `control` | bold | Control flow such as `if`, `return`, `for` and `import` |
| `function` | | Function and method names |
| `type` | | Types, classes, interfaces and enums |
| `variable` | | Variables and identifiers |
| `parameter` | italic | Function parameters |
| `property` | | Object properties, fields and keys |
| `string` | | Strings and characters |
| `number` | | Numbers, booleans and other constants |
| `comment` | italic | Comments, held to the same floor as code |
| `operator` | | Operators and punctuation |
| `tag` | bold | Markup tags |
| `attribute` | | Markup attributes |
| `regex` | | Regular expressions and escapes |
| `invalid` | underline | Invalid code |
| `added`, `removed`, `changed` | | Lines in a diff |

### Terminal colours

These sit in `[palette.terminal]`.

| Field | Type | Meaning |
| --- | --- | --- |
| `ansi` | array of exactly 16 colours | Slots 0 to 7 are black, red, green, yellow, blue, magenta, cyan and white. Slots 8 to 15 are their bright forms |
| `dim` | array of slot numbers | Slots checked at `needs.contrast.terminal_dim` rather than `needs.contrast.terminal` |
| `background_slots` | array of slot numbers | Slots that programs use as a fill behind text, such as black on a dark terminal. They are checked as a fill under the default text instead of as text |

## Pairs

A checker measures every pair below and compares it with the floor named. A profile passes only when
every pair passes. A writer of app config should draw only these pairs, so that a palette that
passes is readable everywhere it is written.

A pair on `surface_alt` is measured only when the profile sets it. When it is missing, those pairs
are the pairs on `surface`, which are measured already.

| Foreground | On | Floor |
| --- | --- | --- |
| `foreground` | `background`, `surface`, `surface_alt`, `line_highlight` and `selection_soft` | `text` |
| `muted`, `link`, `error`, `warning`, `info` and `accent` | `background`, `surface` and `surface_alt` | `text` |
| `muted`, `link`, `error`, `warning`, `info` and `accent` | `line_highlight` | `overlay` |
| `cursor` | `background` | `text` |
| `cursor_text` | `cursor` | `text` |
| `selection_foreground` | `selection_background` | `text` |
| `accent_foreground` | `accent` | `text` |
| `border` and `focus_ring` | `background`, `surface` and `surface_alt` | `non_text` |
| `selection_background` | `background` | `non_text` |
| every syntax colour | `background`, `surface` and `surface_alt` | `text` |
| every syntax colour | `line_highlight` and `selection_soft` | `overlay` |
| each terminal slot not in `background_slots` | `background` | `terminal` (`terminal_dim` for slots in `dim`) |
| `foreground` | each terminal slot in `background_slots` | `text` |
| `line_highlight` and `selection_soft` | `background`, as an OKLab distance | `highlight_distance` |
| `foreground` and every syntax colour | `background`, as APCA Lc, only when `apca` is set | `apca` |
| each pair in [Paired colours](#paired-colours) | each other, as a tone difference | `tone_difference` |
| each pair in [Paired colours](#paired-colours) | each other, as an OKLab distance under each of the three simulations | `colour_vision_distance` |

### Paired colours

Some roles are drawn next to each other and are told apart by colour, so they must also stay apart
for a person with a colour vision deficiency. Each pair below is checked for a tone difference of at
least `tone_difference` and an OKLab distance of at least `colour_vision_distance` under each of
protanopia, deuteranopia and tritanopia. A simulation applies the Machado, Oliveira and Fernandes
(2009) matrix at full severity to linear sRGB, clamps each channel to the gamut and rounds the
result to 8 bit sRGB before it is measured.

A pair marked *always* is checked whatever the font styles of its roles, because its colours are
often red and green. Every other pair is checked only when both roles share a font style from
[Syntax colours](#syntax-colours), since a difference in style already tells them apart.

| Pair | Checked |
| --- | --- |
| `keyword` and `function`, `keyword` and `type`, `keyword` and `variable` | same style |
| `control` and `keyword`, `control` and `variable`, `control` and `function` | same style |
| `function` and `variable`, `function` and `type`, `function` and `property`, `function` and `string` | same style |
| `type` and `variable`, `type` and `property` | same style |
| `variable` and `property`, `variable` and `parameter` | same style |
| `parameter` and `type`, `parameter` and `property` | same style |
| `property` and `string`, `string` and `comment` | same style |
| `number` and `variable`, `number` and `operator` | same style |
| `comment` and `variable`, `invalid` and `variable` | same style |
| `tag` and `attribute`, `attribute` and `string` | same style |
| `string` and `number`, `string` and `regex`, `string` and `invalid` | always |
| `added` and `removed`, `added` and `changed`, `removed` and `changed` | always |
| `error` and `warning` | always, unless `check_error_warning` is `false` |

The syntax pairs are the pairs the [Zacess themes](https://github.com/zacesshq/themes) check, with
the same floors.

## Derivation

A tool that completes a profile must keep every colour the profile sets. When no colour can meet a
floor, for example a text floor of 18:1 on a mid grey, it must fail and say which colour and floor
could not be met. It must never hand back a derived colour that fails against the background or
against another derived colour. A pair that fails because of a colour the profile sets, such as a
comment colour that is too dim, is reported as that pair failing.

`zprf` derives colours in OKLCH. Each role has a fixed hue. Lightness moves away from the
background until the floor is met with a small margin on every surface the colour is drawn on,
`surface_alt` included when it is set. When an APCA floor is set the colour must
meet that as well. Chroma is reduced where needed to stay inside the sRGB gamut. Before handing a
profile back it checks every pair a derived colour takes part in and stops on any failure the rule
above puts on a derived colour, naming the colour and the floor. The exact colours it chooses are
not part of this spec and may change between tool versions. The guarantee that they pass is.

Every role solved this way starts at the same floor and so at the same tone, which paired colours
must not share. `zprf` then places the derived syntax colours and the derived error and warning one
at a time, each at the nearest lightness on its own hue where it meets its floors and stays apart
from every role placed before it. Colours the profile sets are placed first and never move. When
the band of lightness above the floors is too narrow for every pair, derivation fails and names the
pair.

## Relation to the Zacess reading profile

The Zacess reader keeps its own reading profiles in JSON. The fields this spec shares with it keep
their meaning.

| Reading profile | This spec |
| --- | --- |
| `font.family` and `font.fallback` | `needs.font.family` and `needs.font.fallback` |
| `font_size_pt` | `needs.font.size_pt` |
| `line_spacing`, `character_spacing` and `column_width` | the same names in `needs` |
| `cursor_size`, `single_column`, `reduce_motion` and `hide_decoration` | the same names in `needs` |
| `contrast` (a theme id) | a starter profile in `profiles/`, such as `zacess-yellow-on-black` |
| `paragraph_spacing` and `reading_ruler` | not in spec version 1 |

## Example

[examples/minimal.toml](examples/minimal.toml) states needs and one colour. The starter profiles in
[profiles](../profiles) are complete examples with every colour set.

```toml
spec_version = 1
name = "Large text, no motion"

[needs]
reduce_motion = true

[needs.font]
size_pt = 18
```

## Versioning

- A new optional field with a default that keeps every existing profile valid and unchanged in
  meaning is a minor change to this document. It does not raise `spec_version`.
- Removing a field, renaming one, changing a default or changing a pair or a floor raises
  `spec_version`.
- Readers must accept every spec version up to the newest one they support.
- While spec version 1 is a draft, a pair or a floor may still be added to it, as the paired colours
  were. A profile that passed before such a change can then fail `check` and name the new pair.
  Marking version 1 final freezes its pairs and floors under the rules above.
