// Searching an archive's index: which pages and headings a query finds,
// best first, and the words around each match.

/** The index `ascribe versions archive` writes in `search-index.js`. */
export interface Index {
  version: number;
  /** Every page, in the sidebar's order. */
  pages: IndexPage[];
}

/** A page of the index. */
interface IndexPage {
  /** Its file, from the top of the folder. */
  href: string;
  /** Its title, as text. */
  title: string;
  /** Its text by heading, the text before its first heading first. */
  sections: Section[];
}

/** A heading and the text under it, up to the next heading. */
interface Section {
  /** The heading's id on the page: empty for the text before the first heading. */
  id: string;
  /** The heading, as text: empty for the text before the first heading. */
  heading: string;
  /** The text under it. */
  text: string;
}

/** Where a result's words were found: the best is first. */
type Rank = "title" | "heading" | "text";

/** A page or a heading a query finds. */
export interface Result {
  rank: Rank;
  /** Its address from the top of the folder, with the heading's `#id`. */
  href: string;
  /** The page's title. */
  title: string;
  /** The heading, or empty for a match in the page's title or its text before any heading. */
  heading: string;
  /** The words around the match, with each of the query's words marked. */
  context: Piece[];
}

/** A piece of a result's context: marked when it's one of the query's words. */
export interface Piece {
  text: string;
  match: boolean;
}

/** The query's words, lowercased, once each. */
export function words(query: string): string[] {
  return [
    ...new Set(
      query
        .toLowerCase()
        .split(/\s+/u)
        .filter((w) => w.length > 0),
    ),
  ];
}

const ORDER: Rank[] = ["title", "heading", "text"];

/** How many characters of context come before a match, and in all. */
const BEFORE = 50;
const LENGTH = 160;

/**
 * Every page and heading the query finds, each once, best first: a page whose
 * title has every word, then a heading that has every word, then a heading's
 * text. In each, the sidebar's order, and each address once. Case is ignored, and a word matches
 * inside a longer one.
 */
export function search(index: Index, query: string): Result[] {
  const terms = words(query);
  if (terms.length === 0) return [];
  const has = (lower: string): boolean => terms.every((t) => lower.includes(t));
  const found: Record<Rank, Result[]> = { title: [], heading: [], text: [] };
  for (const page of index.pages) {
    if (has(lowered(page, page.title))) {
      const first = page.sections.find((s) => s.text.length > 0)?.text ?? "";
      found.title.push(result("title", page, null, around(first, terms)));
    }
    for (const section of page.sections) {
      if (section.heading.length > 0 && has(section.heading.toLowerCase())) {
        found.heading.push(result("heading", page, section, around(section.text, terms)));
      } else if (has(lowered(section, `${section.heading} ${section.text}`))) {
        found.text.push(result("text", page, section, around(section.text, terms)));
      }
    }
  }
  // A page found by its title isn't listed again for its text before any heading.
  const seen = new Set<string>();
  return ORDER.flatMap((rank) => found[rank]).filter((r) => !seen.has(r.href) && seen.add(r.href));
}

/** The text of each page and section, lowercased once: a query is typed a letter at a time. */
const lowercased = new WeakMap<object, string>();

function lowered(of: object, text: string): string {
  let lower = lowercased.get(of);
  if (lower === undefined) {
    lower = text.toLowerCase();
    lowercased.set(of, lower);
  }
  return lower;
}

function result(rank: Rank, page: IndexPage, section: Section | null, context: Piece[]): Result {
  const id = section?.id ?? "";
  return {
    rank,
    href: id.length > 0 ? `${page.href}#${id}` : page.href,
    title: page.title,
    heading: section?.heading ?? "",
    context,
  };
}

/**
 * The words around the first of the query's words in `text`, cut at spaces,
 * with each word of the query in it marked.
 */
export function around(text: string, terms: string[]): Piece[] {
  if (text.length === 0) return [];
  let lower = text.toLowerCase();
  // Lowercasing can change a string's length; then the lowercased text is shown.
  const shown = lower.length === text.length ? text : lower;
  const first = Math.min(
    ...terms.map((t) => lower.indexOf(t)).filter((i) => i >= 0),
    Number.MAX_SAFE_INTEGER,
  );
  const at = first === Number.MAX_SAFE_INTEGER ? 0 : first;
  let start = Math.max(0, at - BEFORE);
  if (start > 0) {
    const space = shown.indexOf(" ", start);
    start = space >= 0 && space < at ? space + 1 : start;
  }
  let end = Math.min(shown.length, start + LENGTH);
  if (end < shown.length) {
    const space = shown.lastIndexOf(" ", end);
    end = space > at ? space : end;
  }
  const cut = shown.slice(start, end);
  lower = lower.slice(start, end);
  const pieces: Piece[] = [];
  const push = (piece: Piece): void => {
    if (piece.text.length > 0) pieces.push(piece);
  };
  if (start > 0) push({ text: "…", match: false });
  let i = 0;
  while (i < cut.length) {
    let next = -1;
    let length = 0;
    for (const t of terms) {
      const j = lower.indexOf(t, i);
      if (j >= 0 && (next < 0 || j < next || (j === next && t.length > length))) {
        next = j;
        length = t.length;
      }
    }
    if (next < 0) {
      push({ text: cut.slice(i), match: false });
      break;
    }
    push({ text: cut.slice(i, next), match: false });
    push({ text: cut.slice(next, next + length), match: true });
    i = next + length;
  }
  if (end < shown.length) push({ text: "…", match: false });
  return pieces;
}
