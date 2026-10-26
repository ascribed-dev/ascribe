// What Search Pages as Readers See Them shows for each place the server
// finds, worked out from plain values so the unit tests can check it without
// VS Code. The server's `ascribe/search` finds the places; this says them as
// `ascribe search` does.

import type { SearchPlace, SearchResult } from "../shapes.js";

/** How many characters of a block a place shows, around the match: as many as `ascribe search` does. */
const WIDTH = 80;

/** An item of the quick pick, as the tests see it. */
export interface SearchItem {
  /** The reader's text, cut around the match. */
  label: string;
  /** The file and line, the heading, and what it reads through. */
  description: string;
  /** Its arm, its builds when not all, and how many pages show it or places use it. */
  detail: string;
  /** The place, to open; none for an item that only says something. */
  place?: SearchPlace;
}

/** What the quick pick's placeholder says it searches. */
export function searchPlaceholder(build: string | undefined): string {
  return build === undefined
    ? "Words, or a phrase in quotes, as readers read them in every build"
    : `Words, or a phrase in quotes, as readers read them in the build ${build}`;
}

/**
 * The items for a search's answer: one for each place, then one that says
 * how many there are when they were cut. When nothing matched, or the query
 * couldn't be used, one item that says so.
 */
export function searchItems(result: SearchResult): SearchItem[] {
  if (result.problem !== null) return [{ label: result.problem, description: "", detail: "" }];
  if (result.nothing !== null) return [{ label: result.nothing, description: "", detail: "" }];
  const items = result.places.map((place) => searchItem(place));
  if (result.truncated) {
    items.push({
      label: `Showing ${result.places.length} of ${result.total} places. Type more to narrow them.`,
      description: "",
      detail: "",
    });
  }
  return items;
}

/** A place's item. */
export function searchItem(place: SearchPlace): SearchItem {
  return {
    label: excerpt(place.text, place.match),
    description: describe(place).join(" · "),
    detail: detail(place).join(" · "),
    place,
  };
}

/**
 * Where a reader meets a place, as `ascribe search`'s second line says it:
 * `install-agent.md:62 · Connect to Quill · through phrase cloud`.
 */
function describe(place: SearchPlace): string[] {
  const parts = [`${place.file}:${place.range.start.line + 1}`];
  // A place many pages show is under a heading on each; a heading is its own.
  const heading = place.pages[0]?.heading;
  if (place.pagesTotal === 1 && place.in !== "heading" && place.in !== "phrase" && heading) {
    parts.push(heading);
  }
  for (const through of place.through) {
    if ("phrase" in through) {
      parts.push(
        place.in === "phrase" ? `phrase ${through.phrase}` : `through phrase ${through.phrase}`,
      );
    }
  }
  for (const through of place.through) {
    if ("title" in through) parts.push(`through ${through.title}'s title`);
  }
  for (const through of place.through) {
    if ("code" in through) parts.push(`through ${through.code}`);
  }
  return parts;
}

/** Its arm, its builds when they aren't all searched, and its reach. */
function detail(place: SearchPlace): string[] {
  const parts: string[] = [];
  if (place.arm !== null) parts.push(`${place.arm} arm`);
  if (place.builds.length > 0) {
    parts.push(`in ${place.builds.length === 1 ? "build" : "builds"} ${place.builds.join(", ")}`);
  }
  const count = place.pagesTotal;
  if (place.in === "phrase") {
    parts.push(`${count} ${count === 1 ? "place uses" : "places use"} it`);
  } else if (count > 1) {
    parts.push(`on ${count} pages`);
  }
  return parts;
}

/**
 * The block's text, cut to about 80 characters around the match's first
 * character at word boundaries, with `…` where it's cut, as `ascribe search`
 * cuts it. `match` is in Unicode characters of `text`; a match of several
 * words can span a long block, so the excerpt keeps its first character only.
 */
export function excerpt(text: string, match: readonly [number, number]): string {
  const chars = Array.from(text);
  if (chars.length <= WIDTH) return text;
  const first = Math.min(match[0], chars.length);
  let start = Math.max(0, first - Math.floor(WIDTH / 4));
  // Start at a word, and end before one that doesn't fit, unless that would
  // cut the match's first word.
  while (start > 0 && start < first && !isSpace(chars[start - 1])) start += 1;
  let end = Math.min(start + WIDTH, chars.length);
  if (end < chars.length) {
    for (let e = end; e > first; e -= 1) {
      if (isSpace(chars[e])) {
        end = e;
        break;
      }
    }
  }
  const before = start > 0 ? "…" : "";
  const after = end < chars.length ? "…" : "";
  return `${before}${chars.slice(start, end).join("").trim()}${after}`;
}

function isSpace(char: string | undefined): boolean {
  return char !== undefined && /\s/u.test(char);
}
