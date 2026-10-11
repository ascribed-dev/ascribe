// The landing page's demonstration: `@source-and-result {page=…}`, a widget of
// home/ascribe.toml whose content is a page's source and whose `page` names
// that page in demo/, the site's third project. The widget's element holds
// only the source; the route draws the page's rendered content after it,
// inside the same element, so the two sit side by side.

/** A run of the page's HTML, or the place to draw a demo page, by its entry id. */
export type Part = { html: string } | { page: string };

const OPENER = /<source-and-result\b[^>]*?\bpage="([^"]*)"[^>]*>/g;
const CLOSER = "</source-and-result>";

/**
 * A page's rendered HTML, cut before each `source-and-result` element's end
 * tag: the runs of HTML, and between them the page each element names.
 * Elements don't nest, since a widget's content is Markdown with no widget of
 * its own kind in it.
 */
export function splitAtResults(html: string): Part[] {
  const parts: Part[] = [];
  let from = 0;
  for (const match of html.matchAll(OPENER)) {
    if (match.index < from) continue;
    const end = html.indexOf(CLOSER, match.index);
    if (end === -1) throw new Error(`a <source-and-result> with no end tag: ${match[0]}`);
    parts.push({ html: html.slice(from, end) }, { page: match[1] ?? "" });
    from = end;
  }
  parts.push({ html: html.slice(from) });
  return parts;
}
