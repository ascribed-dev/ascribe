// Comments GitHub can't anchor to a line go on the pull request's
// conversation, each ending with a hidden marker naming its block:
//
//   <!-- ascribe:anchor guides/install.md:12-14 build=site -->
//
// Several can share one body (a review's), one after another. A review's
// summary goes before them, ended by another marker:
//
//   Looks good, a few notes below.
//
//   <!-- ascribe:summary -->
//
// A comment quotes its block's source, or its text as the page shows it,
// escaped so GitHub shows it as written; the marker of one that quotes the
// shown text ends ` quote=text`.

/** One anchored comment, as written in a body. */
export interface MarkedSection {
  /** The block's source. */
  source: string;
  build: string | undefined;
  /** The comment, without the quote, the link line, or the marker. */
  body: string;
  /** The block's text the comment quoted, if it did. */
  quote: string | undefined;
}

const MARKER = /<!-- ascribe:anchor (\S+)(?: build=(\S+))?( quote=text)? -->/g;

/**
 * The summary a pending review starts with. GitHub can't edit the summary of
 * a pending review created without one, so the session's reviews start with
 * this, which reading and submitting remove.
 */
export const PLACEHOLDER = "<!-- ascribe:review -->";

/** Ends a review's summary when anchored comments follow it. */
export const SUMMARY_END = "<!-- ascribe:summary -->";

/** `body` without the placeholder summary. */
export function withoutPlaceholder(body: string): string {
  return body.split(PLACEHOLDER).join("").trim();
}

/**
 * The marker for a block. Without a build (or with `""`), the comment is on
 * every build's page. `shown` marks a quote of the shown text.
 */
export function marker(source: string, build: string | undefined, shown = false): string {
  return `<!-- ascribe:anchor ${source}${build === undefined || build === "" ? "" : ` build=${build}`}${shown ? " quote=text" : ""} -->`;
}

/** Joins a mention or reference sign to what follows, so GitHub doesn't link it. */
const JOINER = "\u2060";

/**
 * Plain text as Markdown that GitHub shows as the text: punctuation
 * escaped, `@name` and `#123` not linked, line breaks and leading spaces
 * kept.
 */
export function escapeText(text: string): string {
  const lines = text.split(/\r?\n/).map((line) => {
    const indent = /^ */.exec(line)?.[0].length ?? 0;
    const rest = line
      .slice(indent)
      .replace(/[!-/:-@[-`{-~]/g, (c) => (c === "<" ? "&lt;" : `\\${c}`))
      .replace(/[@#]/g, `$&${JOINER}`);
    return "\u00a0".repeat(indent) + rest;
  });
  // A backslash at the end of a line breaks it, but not before a blank line.
  return lines
    .map((line, i) => (line !== "" && (lines[i + 1] ?? "") !== "" ? `${line}\\` : line))
    .join("\n");
}

/** Text that `escapeText` wrote, as it was. */
export function unescapeText(text: string): string {
  return text
    .split(/\r?\n/)
    .map((line) => {
      const trailing = /\\*$/.exec(line)?.[0].length ?? 0;
      const unbroken = trailing % 2 === 1 ? line.slice(0, -1) : line;
      const indent = /^\u00a0*/.exec(unbroken)?.[0].length ?? 0;
      return (
        " ".repeat(indent) +
        unbroken
          .slice(indent)
          .replace(new RegExp(`([@#])${JOINER}`, "g"), "$1")
          .replace(/\\([!-/:-@[-`{-~])|&lt;/g, (_, c: string | undefined) => c ?? "<")
      );
    })
    .join("\n");
}

/**
 * Writes a section: the block's text quoted, the comment, a line linking the
 * block's lines, and the marker.
 */
export function formatSection(options: {
  source: string;
  build: string | undefined;
  body: string;
  quote: string | undefined;
  /** Whether `quote` is the block's text as the page shows it, rather than its source. */
  shown?: boolean;
  link: { label: string; url: string } | undefined;
}): string {
  const parts: string[] = [];
  const shown = options.shown === true;
  if (options.quote !== undefined && options.quote.trim() !== "") {
    parts.push(
      (shown ? escapeText(options.quote) : options.quote)
        .split(/\r?\n/)
        .map((line) => (line === "" ? ">" : `> ${line}`))
        .join("\n"),
    );
  }
  parts.push(options.body.trim());
  const end = marker(options.source, options.build, shown);
  parts.push(
    options.link === undefined
      ? end
      : `<sub>On [${options.link.label.replace(/[[\]]/g, "\\$&")}](${options.link.url})</sub>\n${end}`,
  );
  return parts.join("\n\n");
}

/**
 * Splits a body into its summary, its marked sections, and the text after
 * the last marker (`rest`), which isn't anchored either.
 */
export function parseSections(text: string): {
  summary: string;
  sections: MarkedSection[];
  rest: string;
} {
  let body = withoutPlaceholder(text);
  let summary = "";
  // The summary ends before the first comment, if it's there at all.
  const end = body.indexOf(SUMMARY_END);
  const first = body.search(new RegExp(MARKER.source));
  if (end >= 0 && (first < 0 || end < first)) {
    summary = body.slice(0, end).trim();
    body = body.slice(end + SUMMARY_END.length).trim();
  }
  const sections: MarkedSection[] = [];
  let start = 0;
  for (const match of body.matchAll(MARKER)) {
    const source = match[1] ?? "";
    const text = body.slice(start, match.index);
    start = match.index + match[0].length;
    const { body: comment, quote } = unwrap(text);
    sections.push({
      source,
      build: match[2],
      body: comment,
      quote: quote !== undefined && match[3] !== undefined ? unescapeText(quote) : quote,
    });
  }
  return { summary, sections, rest: body.slice(start).trim() };
}

function unwrap(text: string): { body: string; quote: string | undefined } {
  let rest = text.replace(/\s*<sub>On \[[^\n]*\]\([^\n)]*\)<\/sub>\s*$/, "").trim();
  let quote: string | undefined;
  const lines = rest.split(/\r?\n/);
  let quoted = 0;
  while (quoted < lines.length && /^>( |$)/.test(lines[quoted] ?? "")) quoted++;
  if (quoted > 0 && quoted < lines.length && (lines[quoted] ?? "").trim() === "") {
    quote = lines
      .slice(0, quoted)
      .map((line) => line.replace(/^> ?/, ""))
      .join("\n");
    rest = lines
      .slice(quoted + 1)
      .join("\n")
      .trim();
  }
  return { body: rest, quote };
}

/**
 * Joins a summary and sections into one body: the summary first, so it
 * doesn't read as a remark on the last section.
 */
export function joinSections(sections: readonly string[], summary: string | undefined): string {
  const held = sections.filter((part) => part.trim() !== "");
  // A marker in the summary would be read as one of Ascribe's.
  const lead = (summary ?? "").trim().replace(/(<!--\s*ascribe):/g, `$1${JOINER}:`);
  if (lead === "") return held.join("\n\n");
  if (held.length === 0) return lead;
  return [lead, SUMMARY_END, ...held].join("\n\n");
}
