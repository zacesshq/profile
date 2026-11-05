// turns a profile into one style sheet: custom properties on :root carrying every value, then
// rules that use them. Every rule is marked important, since a page's own styles would otherwise
// win and the whole point is that the person's needs come first

import { contrast, isDark, shownRatio } from "./colour";
import type { Profile } from "./profile";

export interface Built {
  css: string;
  /** What was applied, in plain words, for the options page. */
  applied: string[];
  /** What was left alone and why. */
  skipped: string[];
}

/** The colours the extension draws, each with what it is drawn on and the floor it must meet. */
const PAIRS: [fg: string, bg: string, floor: "text" | "non_text"][] = [
  ["foreground", "background", "text"],
  ["link", "background", "text"],
  ["foreground", "surface", "text"],
  ["link", "surface", "text"],
  ["selection_foreground", "selection_background", "text"],
  ["accent_foreground", "accent", "text"],
  ["border", "background", "non_text"],
  ["border", "surface", "non_text"],
  ["focus_ring", "background", "non_text"],
  ["focus_ring", "surface", "non_text"],
];

const COLOUR_ROLES = [
  "background", "foreground", "surface", "muted", "border", "accent", "accent_foreground",
  "link", "focus_ring", "selection_background", "selection_foreground", "selection_soft",
  "error", "warning", "info",
];

/** Media and replaced content keep their own look; recolouring them would destroy what they show. */
const MEDIA = "img, picture, video, canvas, svg, svg *, iframe, embed, object, math, math *";

/** Icon fonts draw glyphs from private code points, so replacing their font turns icons into boxes. */
const ICONS = [
  '[class*="icon" i]', ".material-icons", '[class*="material-symbols"]', ".fa", ".fas", ".far",
  ".fab", ".fa-solid", ".fa-regular", ".fa-brands", '[class^="fa-"]', '[class*=" fa-"]',
  ".glyphicon", ".codicon", '[aria-hidden="true"]',
].join(", ");

const CODE = "code, pre, kbd, samp, var, tt";

/** Text-level elements that take the minimum size. Headings are left out so they stay larger. */
const TEXT = [
  "p", "li", "dd", "dt", "td", "th", "blockquote", "figcaption", "caption", "label", "legend",
  "summary", "input", "textarea", "select", "option", "button", "small", "sub", "sup", "cite",
  "time", "abbr", "address", "code", "pre", "kbd", "samp",
].join(", ");

const BLOCKS = "p, li, dd, dt, blockquote, figcaption";

// every value that reaches the style sheet is checked here again, at the point of use, rather than
// trusting that the schema ran: a colour must be #rrggbb and a font name a plain quoted string.
// Free text such as the profile name never goes into the style sheet at all
const HEX = /^#[0-9a-fA-F]{6}$/;
const SAFE_FAMILY = /^[^"'\\\u0000-\u001F\u007F-\u009F<>{};]*$/u;
const cssString = (s: string): string => `"${s}"`;

const generic = (fallback: string | undefined): string =>
  fallback === "serif" ? "serif" : fallback === "monospace" ? "monospace" : fallback === "sans_serif" ? "sans-serif" : "";

export function buildCss(profile: Profile): Built {
  const needs = profile.needs ?? {};
  const palette = profile.palette ?? {};
  const floors = { text: needs.contrast?.text ?? 7, non_text: needs.contrast?.non_text ?? 3 };
  const vars: string[] = [];
  const rules: string[] = [];
  const applied: string[] = [];
  const skipped: string[] = [];
  const hex = (role: string): string | undefined => {
    const v = palette[role];
    return typeof v === "string" && HEX.test(v) ? v : undefined;
  };

  // colours are drawn only as a whole: half a palette over a page's own colours could put the
  // profile's text on the page's background, a pair nobody has checked
  const missing = [...new Set(PAIRS.flatMap(([a, b]) => [a, b]))].filter((r) => !hex(r));
  const failing = missing.length
    ? []
    : PAIRS.filter(([fg, bg, floor]) => contrast(hex(fg)!, hex(bg)!) < floors[floor]).map(
        ([fg, bg, floor]) =>
          `${fg} ${hex(fg)} on ${bg} ${hex(bg)} is ${shownRatio(contrast(hex(fg)!, hex(bg)!))}, below needs.contrast.${floor} of ${floors[floor]}:1`,
      );
  const colours = missing.length === 0 && failing.length === 0;
  if (colours) {
    for (const role of COLOUR_ROLES) {
      const v = hex(role);
      if (v) vars.push(`--zprf-${role.replace(/_/g, "-")}: ${v};`);
    }
    // the appearance is optional, so a profile without one is read from its background as zprf does
    const scheme = profile.appearance ?? (isDark(hex("background")!) ? "dark" : "light");
    rules.push(
      `:root { color-scheme: ${scheme} !important; }`,
      `html, body { background-color: var(--zprf-background) !important; color: var(--zprf-foreground) !important; }`,
      `body *:not(${MEDIA}) { background-color: transparent !important; color: inherit !important; border-color: var(--zprf-border) !important; }`,
      `body :is(input, textarea, select, button, dialog, [role="dialog"], [role="menu"], [role="listbox"]) { background-color: var(--zprf-surface) !important; color: var(--zprf-foreground) !important; }`,
      `body :is(a:link, a:visited), body :is(a:link, a:visited) *:not(${MEDIA}) { color: var(--zprf-link) !important; }`,
      // links are underlined so they never rest on colour alone
      `body :is(a:link, a:visited) { text-decoration: underline !important; }`,
      `body mark { background-color: var(--zprf-accent) !important; color: var(--zprf-accent-foreground) !important; }`,
      `::selection { background-color: var(--zprf-selection-background) !important; color: var(--zprf-selection-foreground) !important; }`,
    );
    applied.push(`colours from ${profile.name}`);
  } else if (missing.length) {
    skipped.push(
      `colours: the profile leaves ${missing.join(", ")} to be derived. Run zprf derive on it and load the result to apply colours`,
    );
  } else {
    skipped.push(`colours: ${failing.join("; ")}`);
  }

  const font = needs.font ?? {};
  const family = font.family?.trim();
  if (family && !SAFE_FAMILY.test(family)) {
    skipped.push("the font: its name holds characters that cannot be used safely in a style sheet");
  } else if (family) {
    const fallback = generic(font.fallback);
    vars.push(`--zprf-font-family: ${cssString(family)}${fallback ? `, ${fallback}` : ""};`);
    // a reading font is kept off code, which needs its fixed width; a monospace profile font is not
    const skip = font.fallback === "monospace" ? ICONS : `${ICONS}, ${CODE}`;
    rules.push(`body, body *:not(${MEDIA}, ${skip}) { font-family: var(--zprf-font-family) !important; }`);
    applied.push(`the font ${family}`);
  }
  if (typeof font.size_pt === "number") {
    const px = Math.round(((font.size_pt * 4) / 3) * 100) / 100;
    vars.push(`--zprf-font-size: ${px}px;`);
    // a minimum, not a fixed size: 1em is the parent's size, so larger text such as a heading
    // keeps its size and only smaller text is raised
    rules.push(`body, body :is(${TEXT}) { font-size: max(1em, var(--zprf-font-size)) !important; }`);
    applied.push(`text at least ${font.size_pt} points (${px} CSS pixels)`);
  }
  if (typeof needs.line_spacing === "number") {
    vars.push(`--zprf-line-height: ${needs.line_spacing};`);
    rules.push(`body, body *:not(${MEDIA}) { line-height: var(--zprf-line-height) !important; }`);
    applied.push(`line spacing ${needs.line_spacing}`);
  }
  if (typeof needs.character_spacing === "number") {
    vars.push(`--zprf-letter-spacing: ${needs.character_spacing}em;`);
    rules.push(`body, body *:not(${MEDIA}) { letter-spacing: var(--zprf-letter-spacing) !important; }`);
    applied.push(`character spacing ${needs.character_spacing} em`);
  }
  if (typeof needs.column_width === "number") {
    vars.push(`--zprf-column-width: ${needs.column_width}ch;`);
    rules.push(`body :is(${BLOCKS}) { max-width: var(--zprf-column-width) !important; }`);
    applied.push(`lines of text at most ${needs.column_width} characters wide`);
  }

  const ring = needs.focus_ring_width_px;
  vars.push(`--zprf-focus-ring-width: ${ring ?? 2}px;`);
  // without the profile's colours the ring takes the element's own text colour, which the page
  // already shows on that background
  const ringColour = colours ? "var(--zprf-focus-ring)" : "currentColor";
  rules.push(
    `:focus-visible { outline: var(--zprf-focus-ring-width) solid ${ringColour} !important; outline-offset: 2px !important; }`,
  );
  applied.push(`a ${ring ?? 2} pixel focus ring${ring ? "" : " (the profile sets no width, so 2 pixels)"}`);

  if (needs.reduce_motion ?? true) {
    // an animation only runs when the page names one, so a near zero duration ends it at its last
    // frame and still fires animationend; a transition runs on every property by default, so any
    // duration above zero would start new ones the page never asked for
    rules.push(
      `*, *::before, *::after { animation-duration: 0.01ms !important; animation-delay: 0s !important; animation-iteration-count: 1 !important; transition-duration: 0s !important; transition-delay: 0s !important; scroll-behavior: auto !important; }`,
      `html { scroll-behavior: auto !important; }`,
    );
    applied.push("no animation, transitions or smooth scrolling");
  }
  if (needs.single_column === true) {
    rules.push(`body *:not(${MEDIA}) { column-count: auto !important; columns: auto !important; }`);
    applied.push("text set in newspaper columns is shown as one column");
  }
  if (needs.hide_decoration === true) {
    rules.push(
      `body, body *:not(${MEDIA}) { background-image: none !important; text-shadow: none !important; box-shadow: none !important; }`,
    );
    applied.push("no background images or shadows");
  }

  const css = `/* Zacess Profile */\n:root {\n  ${vars.join("\n  ")}\n}\n${rules.join("\n")}\n`;
  return { css, applied, skipped };
}
