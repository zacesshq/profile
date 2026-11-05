# Zacess Profile for the browser

A WebExtension (Manifest V3, TypeScript, no framework) that applies a
[Zacess Profile](../README.md) to every web page through CSS custom properties. It builds for Chrome
and Firefox from one source.

What it applies, need by need, is listed in
[ACCESSIBILITY.md](../ACCESSIBILITY.md#the-browser-extension). It is not in any store yet:
[STORE.md](STORE.md) holds the listing text and the store assets.

## How it works

| Part | File | Does |
| --- | --- | --- |
| Options page | [`src/options.ts`](src/options.ts) | Reads a pasted or imported TOML profile, checks it and saves it in extension storage on this device |
| Profile reader | [`src/profile.ts`](src/profile.ts), [`src/schema.ts`](src/schema.ts) | Parses TOML 1.0 and validates the result against [`spec/profile.schema.json`](../spec/profile.schema.json), the same schema `zprf` follows |
| Style sheet | [`src/css.ts`](src/css.ts) | Turns the profile into `--zprf-*` custom properties on `:root` and the rules that use them, after checking each colour pair it draws against the profile's floors |
| Content script | [`src/content.ts`](src/content.ts) | Keeps one style element in every frame in step with the stored profile, so a change reaches open tabs without a reload |
| Background | [`src/background.ts`](src/background.ts) | Opens the options page on first install and from the toolbar button |

A profile that leaves colours to be derived still applies its font, size, spacing, focus ring and
motion needs. Its colours apply once every colour is set: run `zprf derive profile.toml --out
complete.toml` and load the result.

Nothing leaves the browser. The profile is kept in `storage.local` and the extension makes no network
requests.

## Build

With Node.js 24 or newer:

```sh
cd extension
npm ci
npm run build
```

This type checks the code and writes `dist/chrome` and `dist/firefox`.

## Load it unpacked

- **Chrome:** open `chrome://extensions`, turn on **Developer mode**, choose **Load unpacked** and
  pick `extension/dist/chrome`.
- **Firefox:** open `about:debugging#/runtime/this-firefox`, choose **Load Temporary Add-on** and
  pick `extension/dist/firefox/manifest.json`. Firefox asks before an extension can change pages, so
  the options page shows an **Allow on every site** button until that is granted.

The options page opens on its own the first time. The toolbar button opens it again.

## Test

```sh
npx playwright install chromium
npm test
npm run lint:firefox
```

`npm test` loads the Chrome build in Chromium with Playwright, loads a profile through the options
page and checks the computed styles of a local test page. `npm run lint:firefox` runs `web-ext lint`
on the Firefox build. CI runs both on every pull request.
