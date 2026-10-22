// The words every surface takes, read from their one home, design/words.toml:
// the headline, the sentence that says what Ascribe is, its short form, and
// the keywords. words.test.ts compares each surface with them, and assets.ts
// draws the headline on the social card.

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
}

/** The words in a copy of design/words.toml, checking each is there and of its kind. */
export function readWords(toml: string): Words {
  const doc = parse(toml) as Record<string, unknown>;
  const text = (key: string): string => {
    const value = doc[key];
    if (typeof value !== "string" || value.trim() === "") {
      throw new Error(`${WORDS} ${key}: a line of text`);
    }
    return value;
  };
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
  };
}

/** The words as design/words.toml holds them now. */
export function words(): Words {
  return readWords(readFileSync(join(ROOT, WORDS), "utf8"));
}
