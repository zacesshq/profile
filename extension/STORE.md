# Store listing

The text and assets for the Chrome Web Store, Firefox Add-ons and Microsoft Edge Add-ons listings.
Version 0.2.1 is submitted to all three. The listings are linked from the [README](README.md#install).

## Listing text

**Name:** Zacess Profile

**Summary** (Chrome allows 132 characters, Firefox 250):

> State what you need to read a screen once. Every page takes your colours, font, text size,
> spacing, focus ring and reduced motion.

**Category:** Accessibility

**Description:**

> Zacess Profile applies one accessibility profile to every website you visit. The same profile
> sets up your terminal and editor through the zprf command, so a page looks the way your other
> tools already do.
>
> A profile is a short TOML file that says what you need: a contrast floor, a font and size, line
> and letter spacing, a line length, a focus ring width, reduced motion, one column and no
> decoration. Paste it into the options page or import the file. The extension checks it against
> the published profile format before it saves it and tells you exactly what is wrong if anything
> is.
>
> On every page it then:
>
> - draws the profile's colours when every pair it uses meets the profile's own contrast floors
> - underlines links so they never rest on colour alone
> - sets your font, while leaving icons, images and video alone
> - raises small text to your size and keeps headings larger
> - applies your line spacing, letter spacing and line length
> - draws a solid focus ring at your width wherever the page shows keyboard focus. A page whose own
>   styles insist on hiding it can still win
> - stops CSS animations and transitions as well as smooth scrolling. Video, animated images and
>   motion a page drives from script are not stopped
> - shows newspaper columns as one column and hides background images and shadows, if you ask
>
> Turn it off for every page with one switch. Nothing leaves your browser: the profile is stored
> on your device and the extension makes no network requests.
>
> Zacess Profile is open source under the Apache License 2.0: https://github.com/zacesshq/profile

**Support:** <https://github.com/zacesshq/profile/issues>

**Homepage:** <https://zacess.com>

## Permissions and privacy answers

| Question | Answer |
| --- | --- |
| Single purpose | Applies the person's accessibility profile to the pages they visit |
| `storage` | Keeps the profile on the device between sessions |
| Access to all sites | The profile has to reach every page the person reads. The content script only adds one style element |
| Remote code | None. Every script is in the package |
| Data collected | None. Firefox's `data_collection_permissions` is set to `none` |

## Assets

The assets use the approved Zacess Profile mark: the black rounded square and the yellow band, with
a window frame in place of the document.

| Asset | Size in pixels | Used by | File |
| --- | --- | --- | --- |
| Icon | 16 by 16 | the toolbar, both browsers | [`icons/icon-16.png`](icons/icon-16.png) |
| Icon | 32 by 32 | the toolbar on high density screens, Windows | [`icons/icon-32.png`](icons/icon-32.png) |
| Icon | 48 by 48 | the extensions page, both browsers | [`icons/icon-48.png`](icons/icon-48.png) |
| Icon | 128 by 128 | the Chrome Web Store and the install dialog | [`icons/icon-128.png`](icons/icon-128.png) |
| Toolbar icons for a dark toolbar | 16 and 32 | Firefox, through `theme_icons` | [`icons/icon-16-dark.png`](icons/icon-16-dark.png), [`icons/icon-32-dark.png`](icons/icon-32-dark.png) |
| Screenshot | 1280 by 800 | both stores | [`store/screenshot-1280x800.png`](store/screenshot-1280x800.png) |
| Small promotional tile | 440 by 280 | the Chrome Web Store | [`store/profile-promo-440x280.png`](store/profile-promo-440x280.png), with its [SVG](store/profile-promo-440x280.svg) |

Chrome shows one icon on every toolbar; the dark variants have a light edge so the black square
stays visible where Firefox draws a dark toolbar.

The screenshot is a real capture, not a mock-up. `npm run store:screenshot` (after `npm run build`)
loads the extension in Chromium, loads the Zacess Yellow on Black starter profile with line
spacing, a line length, one column and no decoration through the options page, puts keyboard focus
on a link in [`store/demo.html`](store/demo.html) and places the capture in
[`store/screenshot-frame-1280x800.svg`](store/screenshot-frame-1280x800.svg). Its alt text for the
listing:

> A web article titled Reading the night sky shown in yellow text on black in a monospace font,
> with underlined cyan links, a yellow focus ring around the Star maps link and the text set in one
> column with wide line spacing.

More screenshots, up to five per store, can be made the same way with other starter profiles.
