// bundles the extension for Chrome and Firefox. The two differ only in the manifest: Chrome runs
// the background code as a service worker and Firefox as an event page with its own add-on id
import { build } from "esbuild";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = JSON.parse(readFileSync(join(here, "package.json"), "utf8"));

const base = {
  manifest_version: 3,
  name: "Zacess Profile",
  version: pkg.version,
  description: "Applies your Zacess Profile to every page: colours, font, text size, spacing, focus ring and reduced motion.",
  homepage_url: "https://github.com/zacesshq/profile",
  permissions: ["storage"],
  host_permissions: ["<all_urls>"],
  icons: { 16: "icons/icon-16.png", 32: "icons/icon-32.png", 48: "icons/icon-48.png", 128: "icons/icon-128.png" },
  action: {
    default_title: "Zacess Profile options",
    default_icon: { 16: "icons/icon-16.png", 32: "icons/icon-32.png" },
  },
  options_ui: { page: "options.html", open_in_tab: true },
  content_scripts: [
    {
      matches: ["<all_urls>"],
      js: ["content.js"],
      run_at: "document_start",
      all_frames: true,
      match_about_blank: true,
    },
  ],
};

const manifests = {
  chrome: { ...base, minimum_chrome_version: "120", background: { service_worker: "background.js" } },
  firefox: {
    ...base,
    action: {
      ...base.action,
      // Firefox swaps the toolbar icon with the browser theme. Its "light" key names the icon for a
      // dark toolbar, which is the variant with a light edge around the black square
      theme_icons: [
        { light: "icons/icon-16-dark.png", dark: "icons/icon-16.png", size: 16 },
        { light: "icons/icon-32-dark.png", dark: "icons/icon-32.png", size: 32 },
      ],
    },
    background: { scripts: ["background.js"] },
    browser_specific_settings: {
      gecko: {
        id: "profile@zacess.com",
        strict_min_version: "142.0",
        // nothing leaves the browser: the profile is kept in extension storage on this device
        data_collection_permissions: { required: ["none"] },
      },
    },
  },
};

for (const [browser, manifest] of Object.entries(manifests)) {
  const out = join(here, "dist", browser);
  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });
  await build({
    entryPoints: ["content", "background", "options"].map((n) => join(here, "src", `${n}.ts`)),
    outdir: out,
    bundle: true,
    format: "iife",
    target: ["chrome120", "firefox142"],
    legalComments: "none",
    logLevel: "warning",
  });
  for (const f of ["options.html", "options.css"]) cpSync(join(here, "src", f), join(out, f));
  cpSync(join(here, "icons"), join(out, "icons"), { recursive: true });
  cpSync(join(here, "..", "LICENSE"), join(out, "LICENSE"));
  // the bundle includes smol-toml, whose BSD licence asks for its notice to travel with it
  cpSync(join(here, "node_modules", "smol-toml", "LICENSE"), join(out, "LICENSE.smol-toml"));
  writeFileSync(join(out, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`built dist/${browser}`);
}
