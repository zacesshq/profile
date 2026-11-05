// loads the built Chrome extension into Chromium, loads a profile through the options page the way
// a person would and checks the computed styles it gives a local test page

import { type BrowserContext, chromium, expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "..", "dist", "chrome");
const starter = readFileSync(join(here, "..", "..", "profiles", "zacess-deep-contrast-dark.toml"), "utf8");

// the starter profile with every need the extension applies turned on
const profile = starter.replace(
  "focus_ring_width_px = 2\n",
  "focus_ring_width_px = 4\nline_spacing = 1.5\ncharacter_spacing = 0.12\ncolumn_width = 60\nsingle_column = true\nhide_decoration = true\n",
);

let context: BrowserContext;
let server: Server;
let origin: string;
let userData: string;
let extensionId: string;
const leaks: string[] = [];

test.beforeAll(async () => {
  const page = readFileSync(join(here, "fixtures", "page.html"));
  server = createServer((req, res) => {
    // anything a hostile profile could make the page fetch lands here and fails the test
    if (req.url !== "/" && req.url !== "/favicon.ico") {
      leaks.push(req.url ?? "");
      res.writeHead(404);
      res.end();
      return;
    }
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(page);
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("the test server has no port");
  origin = `http://127.0.0.1:${address.port}`;

  userData = mkdtempSync(join(tmpdir(), "zprf-extension-"));
  context = await chromium.launchPersistentContext(userData, {
    // the full Chromium build, since the headless shell cannot load extensions
    channel: "chromium",
    args: [`--disable-extensions-except=${dist}`, `--load-extension=${dist}`],
  });
  let [worker] = context.serviceWorkers();
  worker ??= await context.waitForEvent("serviceworker");
  extensionId = new URL(worker.url()).host;
});

test.afterAll(async () => {
  await context?.close();
  server?.close();
  if (userData) rmSync(userData, { recursive: true, force: true });
});

/** Focuses the button the way a keyboard user does, so :focus-visible matches. */
async function tabToButton(page: import("@playwright/test").Page) {
  for (let i = 0; i < 10; i++) {
    await page.keyboard.press("Tab");
    if (await page.evaluate(() => document.activeElement?.id === "button")) return;
  }
  throw new Error("Tab never reached the button");
}

async function loadProfile(text: string) {
  const options = await context.newPage();
  await options.goto(`chrome-extension://${extensionId}/options.html`);
  await options.getByLabel("Profile TOML").fill(text);
  await options.getByRole("button", { name: "Use this profile" }).click();
  return options;
}

test("a bad profile is refused with the reason", async () => {
  const options = await loadProfile('spec_version = 1\nname = "x"\n[needs]\nreduse_motion = true\n');
  await expect(options.getByRole("status")).toHaveText(/was not loaded/);
  await expect(options.getByRole("alert")).toContainText("needs.reduse_motion is not a field the profile format knows");

  await options.getByLabel("Profile TOML").fill('spec_version = 1\nname = "x"\n[needs.contrast]\ntext = 30\n');
  await options.getByRole("button", { name: "Use this profile" }).click();
  await expect(options.getByRole("alert")).toContainText("needs.contrast.text must be at most 21, not 30");

  // zprf reads TOML integers strictly, so a float where an integer belongs is refused here too
  for (const [body, message] of [
    ['spec_version = 1.0\nname = "x"\n', "spec_version must be a whole number"],
    ['spec_version = 1\nname = "x"\n[needs]\nfocus_ring_width_px = 2.0\n', "needs.focus_ring_width_px must be a whole number"],
    ['spec_version = 1\nname = "x"\nneeds = 1979-05-27\n', "needs must be a table, not a date"],
  ]) {
    await options.getByLabel("Profile TOML").fill(body!);
    await options.getByRole("button", { name: "Use this profile" }).click();
    await expect(options.getByRole("alert")).toContainText(message!);
  }

  await options.getByLabel("Profile TOML").fill("spec_version = 1\nname = \n");
  await options.getByRole("button", { name: "Use this profile" }).click();
  await expect(options.getByRole("alert")).toContainText("line 2");
  await options.close();
});

test("a profile reaches the page through custom properties", async () => {
  const options = await loadProfile(profile);
  await expect(options.getByRole("status")).toHaveText(/Loaded Zacess Deep Contrast Dark/);
  await expect(options.locator("#applied")).toContainText("colours from Zacess Deep Contrast Dark");

  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.bringToFront();
  await expect(page.locator("style#zacess-profile")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe("rgb(0, 0, 0)");
  await tabToButton(page);
  const styles = await page.evaluate(() => {
    const css = (id: string) => getComputedStyle(document.getElementById(id)!);
    const root = getComputedStyle(document.documentElement);
    const button = css("button");
    return {
      background: getComputedStyle(document.body).backgroundColor,
      text: css("para").color,
      link: css("link").color,
      linkLine: css("link").textDecorationLine,
      prop: root.getPropertyValue("--zprf-background").trim(),
      focusProp: root.getPropertyValue("--zprf-focus-ring").trim(),
      font: css("para").fontFamily,
      iconFont: css("icon").fontFamily,
      paraSize: css("para").fontSize,
      smallSize: css("small").fontSize,
      headingSize: css("heading").fontSize,
      lineHeight: css("para").lineHeight,
      letterSpacing: css("para").letterSpacing,
      maxWidth: css("para").maxWidth,
      sixtyCh: (() => {
        const probe = document.createElement("span");
        probe.style.cssText = "display: inline-block; width: 60ch";
        document.getElementById("para")!.append(probe);
        const width = getComputedStyle(probe).width;
        probe.remove();
        return width;
      })(),
      outlineWidth: button.outlineWidth,
      outlineColour: button.outlineColor,
      outlineStyle: button.outlineStyle,
      animation: css("spin").animationDuration,
      iterations: css("spin").animationIterationCount,
      transition: css("spin").transitionDuration,
      columns: css("columns").columnCount,
      image: css("decorated").backgroundImage,
      shadow: css("decorated").boxShadow,
    };
  });
  // colours
  expect(styles.prop).toBe("#000000");
  expect(styles.focusProp).toBe("#ffff00");
  expect(styles.background).toBe("rgb(0, 0, 0)");
  expect(styles.text).toBe("rgb(255, 255, 255)");
  expect(styles.link).toBe("rgb(29, 180, 255)");
  expect(styles.linkLine).toBe("underline");
  // font, with icon fonts left alone
  expect(styles.font).toContain("Atkinson Hyperlegible Mono");
  expect(styles.iconFont).toContain("Material Icons");
  // 14 points is 18.67 CSS pixels: a minimum, so the heading keeps its 40 pixels
  expect(parseFloat(styles.paraSize)).toBeCloseTo(18.67, 1);
  expect(parseFloat(styles.smallSize)).toBeCloseTo(18.67, 1);
  expect(styles.headingSize).toBe("40px");
  // spacing and line length
  expect(parseFloat(styles.lineHeight)).toBeCloseTo(18.67 * 1.5, 0);
  expect(parseFloat(styles.letterSpacing)).toBeCloseTo(18.67 * 0.12, 1);
  // a computed max-width is in pixels, so it is compared with a box 60ch wide in the same font
  expect(parseFloat(styles.maxWidth)).toBeCloseTo(parseFloat(styles.sixtyCh), 0);
  // focus ring
  expect(styles.outlineWidth).toBe("4px");
  expect(styles.outlineStyle).toBe("solid");
  expect(styles.outlineColour).toBe("rgb(255, 255, 0)");
  // reduced motion
  expect(styles.animation).toBe("1e-05s");
  expect(styles.iterations).toBe("1");
  expect(styles.transition).toBe("0s");
  // one column and no decoration
  expect(styles.columns).toBe("auto");
  expect(styles.image).toBe("none");
  expect(styles.shadow).toBe("none");

  // turning it off on the options page removes it from the open page without a reload
  await options.getByLabel("Apply to every page").uncheck();
  await expect(page.locator("style#zacess-profile")).toHaveCount(0);
  expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe("rgb(255, 255, 255)");
  await options.getByLabel("Apply to every page").check();
  await expect(page.locator("style#zacess-profile")).toHaveCount(1);
  await options.close();
  await page.close();
});

test("a profile that leaves colours out changes everything but the colours", async () => {
  const minimal = readFileSync(join(here, "..", "..", "spec", "examples", "every-need.toml"), "utf8");
  const options = await loadProfile(minimal);
  await expect(options.locator("#skipped")).toContainText("Run zprf derive");
  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.bringToFront();
  await expect(page.locator("style#zacess-profile")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.getElementById("para")!).fontSize)).toBe("21.33px");
  await tabToButton(page);
  const styles = await page.evaluate(() => {
    return {
      background: getComputedStyle(document.body).backgroundColor,
      size: getComputedStyle(document.getElementById("para")!).fontSize,
      outline: getComputedStyle(document.getElementById("button")!).outlineWidth,
      code: getComputedStyle(document.getElementById("code")!).fontFamily,
    };
  });
  expect(styles.background).toBe("rgb(255, 255, 255)");
  expect(parseFloat(styles.size)).toBeCloseTo(21.33, 1);
  expect(styles.outline).toBe("4px");
  // a monospace profile font reaches code as well
  expect(styles.code).toContain("Atkinson Hyperlegible Mono");
  await options.getByRole("button", { name: "Remove the profile" }).click();
  await expect(page.locator("style#zacess-profile")).toHaveCount(0);
  await options.close();
  await page.close();
});

test("a light profile without an appearance gets light page controls", async () => {
  const light = readFileSync(join(here, "..", "..", "profiles", "zacess-light.toml"), "utf8").replace('appearance = "light"\n', "");
  const options = await loadProfile(light);
  await expect(options.getByRole("status")).toHaveText(/Loaded Zacess Light/);
  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.bringToFront();
  await expect(page.locator("style#zacess-profile")).toHaveCount(1);
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe("light");
  await options.getByRole("button", { name: "Remove the profile" }).click();
  await options.close();
  await page.close();
});

test("a hostile profile name cannot reach the style sheet or fetch anything", async () => {
  // the reviewed proof of concept: a name that closed the style sheet comment and added rules
  // that fetched an address for each matching form value
  const name = `**//body{background:url(${origin}/leak-body)!important}input[value^="42"]{background-image:url(${origin}/leak-pin)!important}/*`;
  const options = await loadProfile(`spec_version = 1\nid = "hostile"\nname = '${name}'\n[needs]\nreduce_motion = false\n`);
  await expect(options.getByRole("status")).toHaveText(/Loaded/);
  // the name is shown as text, never as markup
  await expect(options.locator("#current-name")).toHaveText(name);
  const page = await context.newPage();
  await page.goto(`${origin}/`);
  await page.bringToFront();
  await expect(page.locator("style#zacess-profile")).toHaveCount(1);
  await page.evaluate(() => {
    const input = document.createElement("input");
    input.value = "4242";
    document.body.append(input);
  });
  await page.waitForTimeout(1000);
  const sheet = (await page.locator("style#zacess-profile").textContent()) ?? "";
  expect(sheet).not.toContain("leak");
  expect(sheet).not.toContain("url(");
  expect(leaks).toEqual([]);
  await options.getByRole("button", { name: "Remove the profile" }).click();
  await options.close();
  await page.close();
});
