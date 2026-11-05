// a small JSON Schema (draft 2020-12) validator covering exactly the keywords that
// spec/profile.schema.json uses. A general validator such as Ajv compiles schemas with new
// Function, which the extension content security policy forbids, so this one interprets the schema

export type Schema = { [key: string]: unknown };

export interface SchemaError {
  path: string;
  message: string;
}

const KNOWN = new Set([
  "$schema", "$id", "$ref", "$defs", "title", "description", "default",
  "type", "const", "enum", "pattern", "minimum", "maximum",
  "items", "minItems", "maxItems", "uniqueItems",
  "properties", "additionalProperties", "required", "if", "then", "not",
]);

// TOML tells integers from floats but JavaScript numbers do not, so the reader parses integers as
// bigint and only a bigint counts as an integer here, as it does for zprf. A TOML date comes back
// as a Date, which is not an object in the profile format
const typeOf = (v: unknown): string => {
  if (v === null) return "null";
  if (Array.isArray(v)) return "array";
  if (v instanceof Date) return "date";
  if (typeof v === "bigint") return "integer";
  return typeof v;
};

const asNumber = (v: unknown): number | undefined =>
  typeof v === "number" ? v : typeof v === "bigint" ? Number(v) : undefined;

const matchesType = (v: unknown, type: string): boolean => {
  const actual = typeOf(v);
  return actual === type || (type === "number" && actual === "integer");
};

const TYPE_NAMES: Record<string, string> = {
  integer: "a whole number",
  number: "a number",
  string: "text in quotes",
  boolean: "true or false",
  object: "a table",
  array: "a list",
};

const show = (v: unknown): string =>
  JSON.stringify(v, (_key, x: unknown) => (typeof x === "bigint" ? Number(x) : x));

const join = (path: string, key: string | number): string =>
  typeof key === "number" ? `${path}[${key}]` : path ? `${path}.${key}` : key;

export function validate(root: Schema, value: unknown): SchemaError[] {
  const errors: SchemaError[] = [];
  const resolve = (s: Schema): Schema => {
    const ref = s["$ref"];
    if (typeof ref !== "string") return s;
    const name = ref.replace(/^#\/\$defs\//, "");
    const defs = root["$defs"] as Record<string, Schema> | undefined;
    const target = defs?.[name];
    if (!target) throw new Error(`the schema refers to ${ref}, which it does not define`);
    // a $ref may sit beside a description, which adds nothing to check
    return target;
  };

  const check = (schemaIn: Schema, v: unknown, path: string, out: SchemaError[]): void => {
    const s = resolve(schemaIn);
    for (const key of Object.keys(s)) {
      if (!KNOWN.has(key)) throw new Error(`the schema uses ${key}, which this validator does not support`);
    }
    const at = path || "the profile";
    const fail = (message: string) => out.push({ path: at, message });
    if (typeof s.type === "string" && !matchesType(v, s.type)) {
      fail(`must be ${TYPE_NAMES[s.type] ?? s.type}, not ${v instanceof Date ? "a date" : show(v)}`);
      return;
    }
    if ("const" in s && (asNumber(v) ?? v) !== s.const) fail(`must be ${show(s.const)}`);
    if (Array.isArray(s.enum) && !s.enum.includes(v)) {
      fail(`must be one of ${s.enum.map(show).join(", ")}`);
    }
    if (typeof v === "string" && typeof s.pattern === "string" && !new RegExp(s.pattern, "u").test(v)) {
      fail(`${show(v)} is not in the allowed form`);
    }
    const n = asNumber(v);
    if (n !== undefined) {
      if (typeof s.minimum === "number" && n < s.minimum) fail(`must be at least ${s.minimum}, not ${n}`);
      if (typeof s.maximum === "number" && n > s.maximum) fail(`must be at most ${s.maximum}, not ${n}`);
    }
    if (Array.isArray(v)) {
      if (typeof s.minItems === "number" && v.length < s.minItems) fail(`needs at least ${s.minItems} items, not ${v.length}`);
      if (typeof s.maxItems === "number" && v.length > s.maxItems) fail(`takes at most ${s.maxItems} items, not ${v.length}`);
      if (s.uniqueItems === true) {
        const seen = v.map(show);
        const dup = seen.find((x, i) => seen.indexOf(x) !== i);
        if (dup !== undefined) fail(`lists ${dup} twice`);
      }
      if (s.items && typeof s.items === "object") {
        v.forEach((item, i) => check(s.items as Schema, item, join(path, i), out));
      }
    }
    if (typeOf(v) === "object") {
      const obj = v as Record<string, unknown>;
      const props = (s.properties ?? {}) as Record<string, Schema>;
      if (Array.isArray(s.required)) {
        for (const key of s.required as string[]) {
          if (!(key in obj)) out.push({ path: join(path, key), message: "is required" });
        }
      }
      for (const [key, child] of Object.entries(obj)) {
        const sub = props[key];
        if (sub) check(sub, child, join(path, key), out);
        else if (s.additionalProperties === false) {
          out.push({ path: join(path, key), message: "is not a field the profile format knows" });
        }
      }
    }
    if (s.not && typeof s.not === "object") {
      const inner: SchemaError[] = [];
      check(s.not as Schema, v, path, inner);
      if (inner.length === 0) fail("matches a form it must not");
    }
    if (s.if && typeof s.if === "object") {
      const inner: SchemaError[] = [];
      check(s.if as Schema, v, path, inner);
      if (inner.length === 0 && s.then && typeof s.then === "object") check(s.then as Schema, v, path, out);
    }
  };

  check(root, value, "", errors);
  return errors;
}
