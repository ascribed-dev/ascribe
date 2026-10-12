// What a page's head says about the page, from its record: the tags
// `Head.astro` writes, as data, so each can be tested against the record. And
// the check `astro dev` makes that a layout doesn't write one of them again.
import type { PageRecord } from "./records.js";

/** A page's social card: its image, what it shows, and its size in pixels. */
export interface Card {
  /** The image's address: absolute, or from the host's root, resolved against the site's address. */
  src: string;
  /** What the image shows, for a reader who can't see it. */
  alt: string;
  width?: number;
  height?: number;
}

/** What a site says about itself, which a record doesn't hold. */
export interface HeadOptions {
  /** The whole title, from the page's title and its record. Default: the page's title. */
  title?: ((title: string, record: PageRecord) => string) | undefined;
  /** The site's name, for `og:site_name` and, on the page at the host's root, the site-name structured data. */
  name?: string | undefined;
  /** The page's social card. Without one, no image tags. */
  card?: Card | undefined;
  /** The site's address (Astro's `site`, or the request's), for addresses the record doesn't make absolute. */
  address: string | URL;
}

/** One tag of the head. */
export type HeadTag =
  | { tag: "title"; text: string }
  | { tag: "meta" | "link"; attributes: Record<string, string> }
  | { tag: "script"; json: unknown };

/**
 * The tags of a page's head that are about the page, in order: the title and
 * the description; the canonical link; the robots rule; the link to the
 * Markdown version, when the build publishes one and the page is offered; the
 * Open Graph tags a link preview reads, the card's among them when there is
 * one; the breadcrumbs, as structured data, when there are two steps or more;
 * and, on the page at the host's root, the site's name, as structured data.
 * The title, the description, and the address are the record's, so the head
 * can't disagree with the sitemap or `llms.txt`.
 */
export function headTags(record: PageRecord, options: HeadOptions): HeadTag[] {
  const tags: HeadTag[] = [];
  const meta = (attributes: Record<string, string>) => tags.push({ tag: "meta", attributes });
  const link = (attributes: Record<string, string>) => tags.push({ tag: "link", attributes });
  // A step's address is on the record's site when it has one.
  const absolute = (route: string) => new URL(route, record.canonical ?? options.address).href;

  const title =
    record.title === undefined
      ? undefined
      : options.title === undefined
        ? record.title
        : options.title(record.title, record);
  if (title !== undefined) tags.push({ tag: "title", text: title });
  if (record.description !== undefined) meta({ name: "description", content: record.description });
  if (record.canonical !== undefined) link({ rel: "canonical", href: record.canonical });
  if (record.listing === "unlisted") meta({ name: "robots", content: "noindex" });
  if (record.listing === "unquoted") meta({ name: "robots", content: "nosnippet" });
  if (record.markdown !== undefined && record.listing !== "unlisted")
    link({ rel: "alternate", type: "text/markdown", href: record.markdown });

  meta({ property: "og:type", content: "website" });
  if (options.name !== undefined) meta({ property: "og:site_name", content: options.name });
  if (title !== undefined) meta({ property: "og:title", content: title });
  if (record.description !== undefined)
    meta({ property: "og:description", content: record.description });
  if (record.canonical !== undefined) meta({ property: "og:url", content: record.canonical });
  const card = options.card;
  if (card !== undefined) {
    meta({ property: "og:image", content: new URL(card.src, options.address).href });
    if (card.width !== undefined) meta({ property: "og:image:width", content: String(card.width) });
    if (card.height !== undefined)
      meta({ property: "og:image:height", content: String(card.height) });
    meta({ property: "og:image:alt", content: card.alt });
  }

  // Google reads a trail of two steps or more.
  const steps = record.breadcrumbs ?? [];
  if (steps.length >= 2) {
    tags.push({
      tag: "script",
      json: {
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        itemListElement: steps.map((step, at) => ({
          "@type": "ListItem",
          position: at + 1,
          name: step.title,
          item: absolute(step.route),
        })),
      },
    });
  }
  // The site's name, on the home page alone: the host's root.
  if (options.name !== undefined && record.route === "/") {
    tags.push({
      tag: "script",
      json: {
        "@context": "https://schema.org",
        "@type": "WebSite",
        name: options.name,
        url: absolute("/"),
      },
    });
  }
  return tags;
}

/**
 * JSON for a `<script>` element: `<` written as `\u003c`, so no string in it
 * can close the element.
 */
export function scriptJson(json: unknown): string {
  return JSON.stringify(json).replaceAll("<", "\\u003c");
}

/** The tags `Head.astro` writes that a head should have once, each with how to find it. */
const ONCE: { name: string; pattern: RegExp }[] = [
  { name: "<title>", pattern: /<title[\s>]/gi },
  ...["description", "robots"].map((name) => ({
    name: `<meta name="${name}">`,
    pattern: new RegExp(`<meta\\s[^>]*name=["']?${name}["'\\s/>]`, "gi"),
  })),
  { name: '<link rel="canonical">', pattern: /<link\s[^>]*rel=["']?canonical["'\s/>]/gi },
  ...["og:type", "og:site_name", "og:title", "og:description", "og:url"].map((name) => ({
    name: `<meta property="${name}">`,
    pattern: new RegExp(`<meta\\s[^>]*property=["']?${name}["'\\s/>]`, "gi"),
  })),
];

/**
 * The tags that a page's `<head>` has more than once, of those `Head.astro`
 * writes, named as `<title>` or `<meta property="og:title">` are. A layout
 * that includes the component and writes one of them too has two.
 */
export function repeatedHeadTags(html: string): string[] {
  const head = /<head[\s>][\s\S]*?<\/head>/i.exec(html)?.[0];
  if (head === undefined) return [];
  return ONCE.filter(({ pattern }) => (head.match(pattern) ?? []).length > 1).map(
    ({ name }) => name,
  );
}
