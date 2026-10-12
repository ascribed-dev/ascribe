import { describe, expect, it } from "vitest";
import { headWarnings } from "../src/head-check.js";
import { headTags, repeatedHeadTags, scriptJson, type HeadTag } from "../src/head.js";
import type { PageRecord } from "../src/records.js";

const RECORD: PageRecord = {
  route: "/docs/guides/install/",
  source: "guides/install.md",
  title: "Install the agent",
  description: "Install and connect it.",
  canonical: "https://docs.example.com/docs/guides/install/",
  listing: "listed",
  listing_from: { level: "default" },
  markdown: "/docs/guides/install.md",
  breadcrumbs: [
    { route: "/docs/", title: "Loom" },
    { route: "/docs/guides/", title: "Guides" },
    { route: "/docs/guides/install/", title: "Install the agent" },
  ],
};

const ADDRESS = "https://preview.example.com";

/** A tag as one line, as it reads in HTML, for comparing a list of them. */
function line(tag: HeadTag): string {
  if (tag.tag === "title") return `<title>${tag.text}</title>`;
  if (tag.tag === "script") return `<script type="application/ld+json">${scriptJson(tag.json)}`;
  const attributes = Object.entries(tag.attributes).map(([name, value]) => `${name}="${value}"`);
  return `<${tag.tag} ${attributes.join(" ")}>`;
}

function lines(record: PageRecord, options: Partial<Parameters<typeof headTags>[1]> = {}) {
  return headTags(record, { address: ADDRESS, ...options }).map(line);
}

/** The structured data among the tags, by its type. */
function data(record: PageRecord, options: Partial<Parameters<typeof headTags>[1]> = {}) {
  return Object.fromEntries(
    headTags(record, { address: ADDRESS, ...options })
      .filter((tag) => tag.tag === "script")
      .map((tag) => [(tag.json as { "@type": string })["@type"], tag.json]),
  );
}

describe("headTags", () => {
  it("writes each tag from the record, and the site's name and card from the site", () => {
    expect(
      lines(RECORD, {
        title: (page) => `${page} · Loom`,
        name: "Loom",
        card: { src: "/social-card.png", alt: "Loom's docs", width: 1200, height: 630 },
      }),
    ).toEqual([
      "<title>Install the agent · Loom</title>",
      '<meta name="description" content="Install and connect it.">',
      '<link rel="canonical" href="https://docs.example.com/docs/guides/install/">',
      '<link rel="alternate" type="text/markdown" href="/docs/guides/install.md">',
      '<meta property="og:type" content="website">',
      '<meta property="og:site_name" content="Loom">',
      '<meta property="og:title" content="Install the agent · Loom">',
      '<meta property="og:description" content="Install and connect it.">',
      '<meta property="og:url" content="https://docs.example.com/docs/guides/install/">',
      // The card's address is on the site's, which a preview's build changes.
      '<meta property="og:image" content="https://preview.example.com/social-card.png">',
      '<meta property="og:image:width" content="1200">',
      '<meta property="og:image:height" content="630">',
      `<meta property="og:image:alt" content="Loom's docs">`,
      expect.stringMatching(/^<script type="application\/ld\+json">.*"BreadcrumbList"/),
    ]);
  });

  it("gives the title function the record, with its route", () => {
    const title = (page: string, record: PageRecord) => (record.route === "/" ? "Loom" : page);
    const home = { ...RECORD, route: "/", title: "Home" };
    expect(lines(home, { title })).toContain("<title>Loom</title>");
    expect(lines(home, { title })).toContain('<meta property="og:title" content="Loom">');
    expect(lines(RECORD, { title })).toContain("<title>Install the agent</title>");
  });

  it("writes no image tags without a card, and no site name without one", () => {
    const written = lines(RECORD);
    expect(written.filter((tag) => tag.includes("og:image"))).toEqual([]);
    expect(written.filter((tag) => tag.includes("og:site_name"))).toEqual([]);
    expect(lines(RECORD, { card: { src: "https://cdn.example.com/c.png", alt: "C" } })).toEqual(
      expect.arrayContaining([
        '<meta property="og:image" content="https://cdn.example.com/c.png">',
        '<meta property="og:image:alt" content="C">',
      ]),
    );
  });

  it("writes the robots rule for a page that isn't listed or isn't quoted, and offers no Markdown version when it isn't listed", () => {
    const unlisted = lines({ ...RECORD, listing: "unlisted" });
    expect(unlisted).toContain('<meta name="robots" content="noindex">');
    expect(unlisted.filter((tag) => tag.includes("text/markdown"))).toEqual([]);
    const unquoted = lines({ ...RECORD, listing: "unquoted" });
    expect(unquoted).toContain('<meta name="robots" content="nosnippet">');
    expect(unquoted).toContain(
      '<link rel="alternate" type="text/markdown" href="/docs/guides/install.md">',
    );
    expect(lines(RECORD).filter((tag) => tag.includes('"robots"'))).toEqual([]);
  });

  it("leaves out what the record doesn't have", () => {
    const bare: PageRecord = {
      route: "/a/",
      source: "a.md",
      listing: "listed",
      listing_from: { level: "default" },
    };
    expect(lines(bare, { name: "Loom" })).toEqual([
      '<meta property="og:type" content="website">',
      '<meta property="og:site_name" content="Loom">',
    ]);
  });

  it("writes the breadcrumbs as structured data, each step's address on the record's site", () => {
    expect(data(RECORD).BreadcrumbList).toEqual({
      "@context": "https://schema.org",
      "@type": "BreadcrumbList",
      itemListElement: [
        { "@type": "ListItem", position: 1, name: "Loom", item: "https://docs.example.com/docs/" },
        {
          "@type": "ListItem",
          position: 2,
          name: "Guides",
          item: "https://docs.example.com/docs/guides/",
        },
        {
          "@type": "ListItem",
          position: 3,
          name: "Install the agent",
          item: "https://docs.example.com/docs/guides/install/",
        },
      ],
    });
    // Without a site, on the site's address.
    const { canonical: _, ...unsited } = RECORD;
    const steps = data(unsited).BreadcrumbList as { itemListElement: { item: string }[] };
    expect(steps.itemListElement[0]?.item).toBe("https://preview.example.com/docs/");
    // One step is no trail.
    expect(data({ ...RECORD, breadcrumbs: RECORD.breadcrumbs?.slice(0, 1) ?? [] })).toEqual({});
    const { breadcrumbs: __, ...without } = RECORD;
    expect(data(without)).toEqual({});
  });

  it("writes the site's name as structured data on the page at the host's root only", () => {
    const home: PageRecord = {
      ...RECORD,
      route: "/",
      canonical: "https://docs.example.com/",
      breadcrumbs: [{ route: "/", title: "Home" }],
    };
    expect(data(home, { name: "Loom" })).toEqual({
      WebSite: {
        "@context": "https://schema.org",
        "@type": "WebSite",
        name: "Loom",
        url: "https://docs.example.com/",
      },
    });
    expect(data(home)).toEqual({});
    expect(data(RECORD, { name: "Loom" }).WebSite).toBeUndefined();
  });

  it("writes JSON that can't close its script element", () => {
    expect(scriptJson({ name: "</script><b>" })).toBe('{"name":"\\u003c/script>\\u003cb>"}');
  });

  it("matches the record, tag for tag", () => {
    // Each tag's value is the record's: none is written from anything else.
    const written = headTags(RECORD, { address: ADDRESS, name: "Loom" });
    const content = (property: string) =>
      written.find((tag) => tag.tag === "meta" && tag.attributes["property"] === property);
    expect(content("og:title")).toMatchObject({ attributes: { content: RECORD.title } });
    expect(content("og:description")).toMatchObject({
      attributes: { content: RECORD.description },
    });
    expect(content("og:url")).toMatchObject({ attributes: { content: RECORD.canonical } });
  });
});

describe("repeatedHeadTags", () => {
  it("names each tag the component writes that the head has twice", () => {
    const html =
      "<html><head><title>A</title><title>B</title>" +
      '<meta name="description" content="x"><meta name=description content="y">' +
      '<link rel="canonical" href="/a/"><meta property="og:title" content="A">' +
      '<meta property="og:title" content="B"><meta property="og:image" content="1">' +
      '<meta property="og:image" content="2"></head><body><title>Not the head</title></body></html>';
    expect(repeatedHeadTags(html)).toEqual([
      "<title>",
      '<meta name="description">',
      '<meta property="og:title">',
    ]);
    expect(repeatedHeadTags("<p>No head</p>")).toEqual([]);
  });

  it("is warned about once a tag, naming the page", () => {
    const seen = new Set<string>();
    const html = "<head><title>A</title><title>B</title></head>";
    expect(headWarnings(html, "/docs/a/", seen)).toEqual([
      "/docs/a/'s head has <title> twice. Head.astro writes it from the page's record, so the layout doesn't need to.",
    ]);
    expect(headWarnings(html, "/docs/b/", seen)).toEqual([]);
  });
});
