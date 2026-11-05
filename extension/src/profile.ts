// reads a profile the way zprf does: TOML 1.0, then the JSON Schema from spec/, then the one rule
// the schema cannot state, that the appearance agrees with the background

import { parse, TomlError } from "smol-toml";
import schemaJson from "../../spec/profile.schema.json";
import { isDark } from "./colour";
import { type Schema, validate } from "./schema";

export type Hex = string;

export interface Profile {
  spec_version: number;
  name: string;
  id?: string;
  description?: string;
  appearance?: "dark" | "light";
  needs?: {
    contrast?: { text?: number; non_text?: number; overlay?: number };
    font?: { family?: string; fallback?: "sans_serif" | "serif" | "monospace"; size_pt?: number };
    reduce_motion?: boolean;
    cursor_size?: "standard" | "large" | "extra_large";
    focus_ring_width_px?: number;
    line_spacing?: number;
    character_spacing?: number;
    column_width?: number;
    single_column?: boolean;
    hide_decoration?: boolean;
  };
  palette?: { [role: string]: unknown };
}

export type ParseResult = { ok: true; profile: Profile } | { ok: false; errors: string[] };

export const schema = schemaJson as Schema;

export function parseProfile(text: string): ParseResult {
  let data: unknown;
  try {
    // a key such as __proto__ is refused outright, since an unknown field is an error in the format
    data = parse(text, { integersAsBigInt: true, unsafeKeyBehaviour: "throw" });
  } catch (err) {
    if (err instanceof TomlError) {
      return { ok: false, errors: [`line ${err.line}, column ${err.column}: ${err.message.split("\n")[0]}`] };
    }
    return { ok: false, errors: [String(err)] };
  }
  const errors = validate(schema, data).map((e) => `${e.path} ${e.message}`);
  if (errors.length > 0) return { ok: false, errors };
  // integers were read as bigint only to tell them from floats; the rest of the extension uses numbers
  const profile = toNumbers(data) as Profile;
  const bg = profile.palette?.background;
  if (typeof bg === "string" && profile.appearance) {
    const dark = isDark(bg);
    if ((profile.appearance === "dark") !== dark) {
      return {
        ok: false,
        errors: [`appearance is ${profile.appearance} but the background ${bg} is ${dark ? "dark" : "light"}: change one of them`],
      };
    }
  }
  return { ok: true, profile };
}

function toNumbers(v: unknown): unknown {
  if (typeof v === "bigint") return Number(v);
  if (Array.isArray(v)) return v.map(toNumbers);
  if (v && typeof v === "object") {
    return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, toNumbers(x)]));
  }
  return v;
}
