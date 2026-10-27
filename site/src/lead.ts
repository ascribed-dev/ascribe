// A docs page's lead, the description under its title, as the body's prose
// reads: its code spans as <code>, from the description's formatted form, and
// its quotes and apostrophes curled, as Astro's smartypants curls the body's.

/**
 * `html`, a formatted value's HTML (text, entities, and `<code>` elements),
 * with its straight quotes curled outside its tags and code: an apostrophe
 * or a closing single quote as ’, an opening one as ‘, and double quotes as
 * “ and ”, opening at the start or after a space or an opening bracket.
 */
export function curlQuotes(html: string): string {
  let inCode = false;
  let previous = "";
  return html.replace(/<[^>]*>|&[^;\s]+;|[^<&]/g, (token) => {
    if (token.startsWith("<")) {
      if (/^<code[\s>]/.test(token)) inCode = true;
      else if (token === "</code>") inCode = false;
      previous = "x";
      return token;
    }
    const quote =
      token === "'" || token === "&#39;" || token === "&apos;"
        ? "'"
        : token === '"' || token === "&quot;"
          ? '"'
          : undefined;
    const opens = previous === "" || /[\s([{]/.test(previous);
    previous = quote ?? (token.startsWith("&") ? "x" : token);
    if (quote === undefined || inCode) return token;
    if (quote === "'") return opens ? "‘" : "’";
    return opens ? "“" : "”";
  });
}
