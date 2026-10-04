// Comments GitHub can't anchor to a line go on the pull request's
// conversation, each ending with a hidden marker naming its block:
//
//   <!-- ascribe:anchor guides/install.md:12-14 build=site -->
//
// Several can share one body (a review's), one after another.

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

const MARKER = /<!-- ascribe:anchor (\S+)(?: build=(\S+))? -->/g;

/** The marker for a block. */
export function marker(source: string, build: string | undefined): string {
  return `<!-- ascribe:anchor ${source}${build === undefined ? "" : ` build=${build}`} -->`;
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
  link: { label: string; url: string } | undefined;
}): string {
  const parts: string[] = [];
  if (options.quote !== undefined && options.quote.trim() !== "") {
    parts.push(
      options.quote
        .split(/\r?\n/)
        .map((line) => (line === "" ? ">" : `> ${line}`))
        .join("\n"),
    );
  }
  parts.push(options.body.trim());
  const end = marker(options.source, options.build);
  parts.push(
    options.link === undefined
      ? end
      : `<sub>On [${options.link.label.replace(/[[\]]/g, "\\$&")}](${options.link.url})</sub>\n${end}`,
  );
  return parts.join("\n\n");
}

/**
 * Splits a body into its marked sections, and the text after the last marker
 * (`rest`), which isn't anchored.
 */
export function parseSections(body: string): { sections: MarkedSection[]; rest: string } {
  const sections: MarkedSection[] = [];
  let start = 0;
  for (const match of body.matchAll(MARKER)) {
    const source = match[1] ?? "";
    const text = body.slice(start, match.index);
    start = match.index + match[0].length;
    sections.push({ source, build: match[2], ...unwrap(text) });
  }
  return { sections, rest: body.slice(start).trim() };
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

/** Joins sections, and the text after them, into one body. */
export function joinSections(sections: readonly string[], rest: string | undefined): string {
  return [...sections, rest ?? ""].filter((part) => part.trim() !== "").join("\n\n");
}
