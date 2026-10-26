// What a docs page's table of contents lists: its sections and their
// subsections, each heading as HTML, with its code spans as <code>, from the
// page as rendered. Astro's `headings` give a heading's text alone.

/** A heading as Astro's markdown pipeline gives it. */
export interface Heading {
  depth: number;
  slug: string;
  text: string;
}

/** A heading in the table of contents, with its content as HTML. */
export interface TocHeading extends Heading {
  html: string;
}

/**
 * The page's level-2 and level-3 headings, each with its content as HTML from
 * `rendered`, the page's HTML: the heading's own markup, less the links and
 * ids inside it, which mustn't nest in the contents' link or repeat an id.
 * A heading not found in `rendered` is its text, escaped.
 */
export function tocHeadings(headings: readonly Heading[], rendered = ""): TocHeading[] {
  return headings
    .filter((heading) => heading.depth === 2 || heading.depth === 3)
    .map((heading) => ({ ...heading, html: headingHtml(rendered, heading) }));
}

function headingHtml(rendered: string, heading: Heading): string {
  const id = escapeRegExp(encodeAttribute(heading.slug));
  const found = new RegExp(
    `<h${heading.depth}(?:\\s[^>]*)?\\sid="${id}"[^>]*>([\\s\\S]*?)</h${heading.depth}>`,
  ).exec(rendered)?.[1];
  if (found === undefined) return escape(heading.text);
  return found
    .replace(/<\/?a(?:\s[^>]*)?>/g, "")
    .replace(/\s(?:id|aria-[a-z-]+)="[^"]*"/g, "")
    .trim();
}

function escape(text: string): string {
  return text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

function encodeAttribute(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll('"', "&quot;");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
