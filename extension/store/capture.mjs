// makes the 1280 by 800 store screenshot: loads the built Chrome extension in Chromium, loads a
// starter profile through the options page, captures the demo page with keyboard focus on a link
// and places the capture in the approved frame. Run it from extension/ after npm run build:
//
//   node store/capture.mjs
import { chromium } from "@playwright/test";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "..", "dist", "chrome");
const slot = { x: 56, y: 120, width: 1168, height: 624 };

// the starter profile with the spacing needs a reader is likely to want, so they show in the shot
const profile = readFileSync(join(here, "..", "..", "profiles", "zacess-yellow-on-black.toml"), "utf8").replace(
  "[needs]\n",
  "[needs]\nline_spacing = 1.5\ncolumn_width = 72\nsingle_column = true\nhide_decoration = true\n",
);

const page = readFileSync(join(here, "demo.html"));
const server = createServer((_req, res) => {
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  res.end(page);
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const origin = `http://127.0.0.1:${server.address().port}`;

const userData = mkdtempSync(join(tmpdir(), "zprf-store-"));
const context = await chromium.launchPersistentContext(userData, {
  channel: "chromium",
  args: [`--disable-extensions-except=${dist}`, `--load-extension=${dist}`],
  viewport: { width: slot.width, height: slot.height },
});
try {
  let [worker] = context.serviceWorkers();
  worker ??= await context.waitForEvent("serviceworker");
  const id = new URL(worker.url()).host;
  const options = await context.newPage();
  await options.goto(`chrome-extension://${id}/options.html`);
  await options.getByLabel("Profile TOML").fill(profile);
  await options.getByRole("button", { name: "Use this profile" }).click();
  await options.getByRole("status").filter({ hasText: "Loaded" }).waitFor();

  const shot = await context.newPage();
  await shot.goto(`${origin}/`);
  await shot.bringToFront();
  await shot.locator("style#zacess-profile").waitFor({ state: "attached" });
  // two presses of Tab put keyboard focus on the second link, so the focus ring shows
  await shot.keyboard.press("Tab");
  await shot.keyboard.press("Tab");
  const capture = await shot.screenshot({ type: "png" });

  const frame = readFileSync(join(here, "screenshot-frame-1280x800.svg"), "utf8")
    .replace("screenshot frame placeholder", "on a web page")
    .replace("screenshot frame placeholder", "on a web page");
  const image =
    `<clipPath id="slot"><rect x="${slot.x}" y="${slot.y}" width="${slot.width}" height="${slot.height}" rx="6"/></clipPath>` +
    `<image x="${slot.x}" y="${slot.y}" width="${slot.width}" height="${slot.height}" clip-path="url(#slot)" href="data:image/png;base64,${capture.toString("base64")}"/>` +
    `<rect x="${slot.x}" y="${slot.y}" width="${slot.width}" height="${slot.height}" rx="6" fill="none" stroke="#B4BAC1" stroke-width="2"/>`;
  const svg = frame.replace(/<\/svg>\s*$/, `${image}</svg>`);
  const render = await context.newPage();
  await render.setViewportSize({ width: 1280, height: 800 });
  await render.setContent(`<!doctype html><html><body style="margin:0">${svg}</body></html>`);
  writeFileSync(join(here, "screenshot-1280x800.png"), await render.screenshot({ type: "png" }));
  console.log("wrote store/screenshot-1280x800.png");
} finally {
  await context.close();
  server.close();
  rmSync(userData, { recursive: true, force: true });
}
