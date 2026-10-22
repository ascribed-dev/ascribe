// The words every surface takes, read from their one home, design/words.toml:
// the headline, the sentence that says what Ascribe is, its short form, the
// keywords, and the maker. words.test.ts compares each surface with them, and
// assets.ts draws the headline on the social card.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { parse } from "smol-toml";
import { ROOT } from "./specimen.ts";

/** The words' home, relative to the repository's root. */
export const WORDS = "design/words.toml";

/** The longest a short form can be: what a package description shows in a line. */
export const SHORT_MAX = 100;

export interface Words {
  /** The promise: "Docs as code, with the tools docs need." */
  headline: string;
  /** What Ascribe is, in a sentence. */
  sentence: string;
  /** The sentence where only a line fits. */
  short: string;
  /** The keywords every package lists, and the repository's topics. */
  keywords: string[];
  /** Who makes Ascribe, wherever a maker is named. */
  maker: Maker;
}

export interface Maker {
  name: string;
  url: string;
  email: string;
}

/** The maker as a package's `author`: `Name <email> (url)`. */
export const author = (maker: Maker): string => `${maker.name} <${maker.email}> (${maker.url})`;

/** The words in a copy of design/words.toml, checking each is there and of its kind. */
export function readWords(toml: string): Words {
  const doc = parse(toml) as Record<string, unknown>;
  const text = (key: string, table: Record<string, unknown> = doc, where = key): string => {
    const value = table[key];
    if (typeof value !== "string" || value.trim() === "") {
      throw new Error(`${WORDS} ${where}: a line of text`);
    }
    return value;
  };
  const maker = (doc["maker"] ?? {}) as Record<string, unknown>;
  const keywords = doc["keywords"];
  if (!Array.isArray(keywords) || keywords.some((k) => typeof k !== "string")) {
    throw new Error(`${WORDS} keywords: a list of strings`);
  }
  const short = text("short");
  if (short.length >= SHORT_MAX) {
    throw new Error(`${WORDS} short: under ${SHORT_MAX} characters, not ${short.length}`);
  }
  return {
    headline: text("headline"),
    sentence: text("sentence"),
    short,
    keywords: keywords as string[],
    maker: {
      name: text("name", maker, "maker.name"),
      url: text("url", maker, "maker.url"),
      email: text("email", maker, "maker.email"),
    },
  };
}

/** The words as design/words.toml holds them now. */
export function words(): Words {
  return readWords(readFileSync(join(ROOT, WORDS), "utf8"));
}
