// The design tokens, design/tokens.toml, and the stylesheet blocks generated
// from them. design/README.md describes the file; tokens.test.ts rewrites the
// blocks with ASCRIBE_BLESS=1 and fails when one is stale.

import { readFileSync } from "node:fs";
import { parse } from "smol-toml";

export const ROOT = new URL("../../", import.meta.url);

/** How a block writes a color. */
export type Mode = "light" | "dark" | "light-dark";

export interface Tokens {
  /** Palette name (`blue.460`) to its CSS value. */
  palette: Map<string, string>;
  /** Color token (`color.review.added`) to its light and dark palette names. */
  colors: Map<string, { light: string; dark: string }>;
  /** Font, type, space, and radius token (`font.ui`, `type.body.size`) to its CSS value. */
  values: Map<string, string>;
  /** Stylesheet, then block name, then property to its token or template. */
  emit: Map<string, Map<string, Map<string, string>>>;
}

type Table = Record<string, unknown>;

const isTable = (value: unknown): value is Table =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** Every leaf under `table`, by its dotted path from `prefix`. */
function leaves(table: Table, prefix: string, found: Map<string, unknown>): void {
  for (const [key, value] of Object.entries(table)) {
    const path = `${prefix}.${key}`;
    if (isTable(value) && !("light" in value)) leaves(value, path, found);
    else found.set(path, value);
  }
}

const COLOR_VALUE = /^(#[0-9a-f]{6}|rgb\(\d+ \d+ \d+ \/ [\d.]+\))$/;

/** The tokens in `text`, checked; an error lists every problem. */
export function load(text: string): Tokens {
  const source = parse(text) as Table;
  const problems: string[] = [];
  const known = new Set(["palette", "color", "font", "type", "space", "radius", "emit"]);
  for (const key of Object.keys(source)) {
    if (!known.has(key)) problems.push(`[${key}] isn't a section tokens.toml has`);
  }

  const palette = new Map<string, string>();
  for (const [hue, steps] of Object.entries((source.palette ?? {}) as Table)) {
    for (const [step, value] of Object.entries(steps as Table)) {
      if (typeof value !== "string" || !COLOR_VALUE.test(value)) {
        problems.push(`palette ${hue}.${step}: write #rrggbb in lowercase, or rgb(r g b / a)`);
      }
      palette.set(`${hue}.${step}`, String(value));
    }
  }

  const colors = new Map<string, { light: string; dark: string }>();
  const found = new Map<string, unknown>();
  leaves((source.color ?? {}) as Table, "color", found);
  for (const [path, value] of found) {
    const entry = value as Table;
    const keys = isTable(value) ? Object.keys(entry).sort().join(",") : "";
    if (keys !== "dark,light") {
      problems.push(`${path}: a color has a light and a dark value, and nothing else`);
      continue;
    }
    for (const scheme of ["light", "dark"] as const) {
      if (!palette.has(String(entry[scheme]))) {
        problems.push(`${path}.${scheme}: "${String(entry[scheme])}" isn't in the palette`);
      }
    }
    colors.set(path, { light: String(entry.light), dark: String(entry.dark) });
  }

  const values = new Map<string, string>();
  for (const section of ["font", "space", "radius"]) {
    for (const [key, value] of Object.entries((source[section] ?? {}) as Table)) {
      if (typeof value !== "string") problems.push(`${section}.${key}: a value is a string`);
      values.set(`${section}.${key}`, String(value));
    }
  }

  for (const [step, entry] of Object.entries((source.type ?? {}) as Table)) {
    const keys = isTable(entry) ? Object.keys(entry).sort().join(",") : "";
    if (keys !== "line,size,weight") {
      problems.push(`type.${step}: a step has a size, a line, and a weight, and nothing else`);
      continue;
    }
    for (const [key, value] of Object.entries(entry as Table)) {
      values.set(`type.${step}.${key}`, String(value));
    }
  }

  const emit = new Map<string, Map<string, Map<string, string>>>();
  for (const [file, blocks] of Object.entries((source.emit ?? {}) as Table)) {
    const byBlock = new Map<string, Map<string, string>>();
    for (const [block, properties] of Object.entries(blocks as Table)) {
      const byProperty = new Map<string, string>();
      for (const [property, value] of Object.entries(properties as Table)) {
        for (const token of references(String(value))) {
          if (!colors.has(token) && !values.has(token)) {
            problems.push(`emit ${file} ${block} ${property}: "${token}" isn't a token`);
          }
        }
        byProperty.set(property, String(value));
      }
      byBlock.set(block, byProperty);
    }
    emit.set(file, byBlock);
  }

  const used = new Set<string>();
  for (const blocks of emit.values()) {
    for (const properties of blocks.values()) {
      for (const value of properties.values())
        for (const token of references(value)) used.add(token);
    }
  }
  for (const token of [...colors.keys(), ...values.keys()]) {
    if (!used.has(token)) problems.push(`${token} isn't used by any stylesheet`);
  }
  const pointed = new Set([...colors.values()].flatMap(({ light, dark }) => [light, dark]));
  for (const name of palette.keys()) {
    if (!pointed.has(name)) problems.push(`palette ${name} isn't used by any color`);
  }

  if (problems.length > 0) throw new Error(`design/tokens.toml:\n  ${problems.join("\n  ")}`);
  return { palette, colors, values, emit };
}

/** The tokens a template or bare token names. */
export function references(value: string): string[] {
  if (!value.includes("{")) return [value];
  return [...value.matchAll(/\{([^}]+)\}/g)].map((match) => match[1] ?? "");
}

/** `value` for a block of `mode`. */
function write(tokens: Tokens, value: string, mode: Mode): string {
  const one = (token: string): string => {
    const color = tokens.colors.get(token);
    if (color === undefined) return tokens.values.get(token) ?? token;
    const light = tokens.palette.get(color.light) ?? "";
    const dark = tokens.palette.get(color.dark) ?? "";
    if (mode === "light") return light;
    if (mode === "dark") return dark;
    return `light-dark(${light}, ${dark})`;
  };
  if (!value.includes("{")) return one(value);
  return value.replace(/\{([^}]+)\}/g, (_, token: string) => one(token));
}

/** A generated block's first line, and the groups: indent, block, mode. */
const START =
  /^([ \t]*)\/\* Generated from design\/tokens\.toml by scripts\/design\/tokens\.ts: ([\w-]+), (light|dark|light-dark)\. \*\/$/gm;

const end = (block: string): string => `/* End of generated ${block}. */`;

/** A generated block in a stylesheet. */
export interface Block {
  block: string;
  mode: Mode;
  /** The start marker's indentation, which the declarations take. */
  indent: string;
  /** Offsets of the whole block, markers included: [start, end). */
  start: number;
  end: number;
  /** Offsets of the lines between the markers: [start, end). */
  body: [number, number];
}

/** Each generated block of `text`, in order. */
export function blocks(text: string): Block[] {
  const found: Block[] = [];
  for (const match of text.matchAll(START)) {
    const [, indent = "", block = "", mode = "light"] = match;
    const close = text.indexOf(end(block), match.index);
    if (close === -1) throw new Error(`no "${end(block)}" after the start of ${block}`);
    const bodyStart = match.index + match[0].length + 1;
    const bodyEnd = text.lastIndexOf("\n", close) + 1;
    found.push({
      block,
      indent,
      mode: mode as Mode,
      start: match.index,
      end: close + end(block).length,
      body: [bodyStart, bodyEnd] as [number, number],
    });
  }
  return found;
}

/** `text`, the stylesheet `file`, with its generated blocks rewritten from `tokens`. */
export function render(tokens: Tokens, file: string, text: string): string {
  const emit = tokens.emit.get(file);
  if (emit === undefined) throw new Error(`${file}: not in tokens.toml's [emit]`);
  const found = blocks(text);
  const named = new Set(found.map(({ block }) => block));
  for (const block of emit.keys()) {
    if (!named.has(block)) throw new Error(`${file}: no generated block named ${block}`);
  }
  let out = text;
  for (const { block, mode, indent, body } of found.reverse()) {
    const properties = emit.get(block);
    if (properties === undefined) throw new Error(`${file}: no [emit] block named ${block}`);
    const lines = [...properties]
      .filter(
        ([, value]) => mode === "light" || references(value).some((t) => tokens.colors.has(t)),
      )
      .map(([property, value]) => `${indent}${property}: ${write(tokens, value, mode)};\n`);
    out = out.slice(0, body[0]) + lines.join("") + out.slice(body[1]);
  }
  return out;
}

/** design/tokens.toml, loaded. */
export function source(): Tokens {
  return load(readFileSync(new URL("design/tokens.toml", ROOT), "utf8"));
}
