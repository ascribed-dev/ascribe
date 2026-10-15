// A built page's head, read as its tags, for test/head.test.ts.

/** One tag of a head: its name, its attributes, and a title's text. */
export interface Tag {
  name: string;
  attributes: Record<string, string>;
  text?: string;
}

function decode(text: string): string {
  return text
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&amp;", "&");
}

/**
 * The tags of a page's head that say something about the page or the site:
 * its title, metas, links, and structured data, less the styles and scripts
 * the build bundles.
 */
export function tagsOf(html: string): Tag[] {
  const head = /<head>([\s\S]*?)<\/head>/.exec(html)?.[1] ?? "";
  const tags: Tag[] = [];
  const found =
    /<title>([^<]*)<\/title>|<(meta|link)\s([^>]*)>|<script type="application\/ld\+json">([\s\S]*?)<\/script>/g;
  for (const match of head.matchAll(found)) {
    if (match[1] !== undefined)
      tags.push({ name: "title", attributes: {}, text: decode(match[1]) });
    else if (match[4] !== undefined) tags.push({ name: "script", attributes: {}, text: match[4] });
    else {
      const attributes = Object.fromEntries(
        [...(match[3] ?? "").matchAll(/([\w:-]+)(?:="([^"]*)")?/g)].map((a) => [
          a[1] ?? "",
          decode(a[2] ?? ""),
        ]),
      );
      if (["stylesheet", "preload", "modulepreload"].includes(attributes["rel"] ?? "")) continue;
      tags.push({ name: match[2] ?? "", attributes });
    }
  }
  return tags;
}

/** A tag as it's named in the saved copy: `<meta property="og:title">`, without its value. */
export function key(tag: Tag): string {
  if (tag.name === "title") return "<title>";
  if (tag.name === "script") {
    const type = (JSON.parse(tag.text ?? "{}") as { "@type"?: string })["@type"];
    return `<script type="application/ld+json"> ${type}`;
  }
  const naming = ["charset", "name", "property", "http-equiv", "rel", "type", "media", "sizes"]
    .filter((attribute) => attribute in tag.attributes)
    .map((attribute) =>
      attribute === "charset" ? "charset" : `${attribute}="${tag.attributes[attribute]}"`,
    );
  return `<${tag.name} ${naming.join(" ")}>`;
}
