// The changelog page's rules for links and anchors, on rendered Markdown made
// here: they need no build.
import { expect, test } from "vitest";
import {
  ChangelogLinkError,
  anchorReleases,
  collapseEntries,
  firstParagraph,
  pointLinks,
  type Targets,
} from "../src/changelog.ts";

const targets: Targets = {
  pages: new Map([
    ["docs/content/index.md", "/docs/"],
    ["docs/content/agents/index.md", "/docs/agents/"],
  ]),
  repo: "https://github.com/ascribed-dev/ascribe",
  kind: (file) =>
    Promise.resolve(
      ({ "SPEC.md": "file", "packages/elements/README.md": "file", examples: "folder" } as const)[
        file
      ],
    ),
};

/** The links' destinations, as the page writes them. */
function hrefs(html: string): string[] {
  return Array.from(html.matchAll(/href="([^"]*)"/g), (match) => match[1] ?? "");
}

test("a link to a docs page's source becomes its route, with its fragment", async () => {
  const html = await pointLinks(
    '<a href="docs/content/agents/index.md#hooks">Agents</a> <a href="docs/content/index.md">docs</a>',
    "",
    targets,
  );
  expect(hrefs(html)).toEqual(["/docs/agents/#hooks", "/docs/"]);
});

test("a link to another file or folder in the repository becomes its address on GitHub", async () => {
  const html = await pointLinks(
    '<a href="SPEC.md">spec</a> <a href="packages/elements/README.md#theming">theming</a> <a href="examples/">examples</a>',
    "",
    targets,
  );
  expect(hrefs(html)).toEqual([
    "https://github.com/ascribed-dev/ascribe/blob/main/SPEC.md",
    "https://github.com/ascribed-dev/ascribe/blob/main/packages/elements/README.md#theming",
    "https://github.com/ascribed-dev/ascribe/tree/main/examples/",
  ]);
});

test("an address and a heading on the page stay as they are", async () => {
  const links =
    '<a href="https://semver.org/">semver</a> <a href="mailto:a@b.c">mail</a> <a href="#v0.2.0">0.2.0</a>';
  expect(await pointLinks(links, "", targets)).toBe(links);
});

test("a link to a docs file that isn't a published page fails, naming its line", async () => {
  const source = "# Changelog\n\n- One.\n- See [Gone](docs/content/guides/gone.md#x).\n";
  const html = '<a href="docs/content/guides/gone.md#x">Gone</a>';
  const error = await pointLinks(html, source, targets).catch((caught: unknown) => caught);
  expect(error).toBeInstanceOf(ChangelogLinkError);
  expect((error as ChangelogLinkError).problems).toEqual([
    "CHANGELOG.md:4: docs/content/guides/gone.md#x: isn't a page the docs publish",
  ]);
});

test("a link to a file that isn't in the repository, or outside it, fails", async () => {
  const source = "[a](NOPE.md)\n[b](../outside.md)\n[c](/SPEC.md)\n";
  const html = '<a href="NOPE.md">a</a><a href="../outside.md">b</a><a href="/SPEC.md">c</a>';
  const error = await pointLinks(html, source, targets).catch((caught: unknown) => caught);
  expect((error as ChangelogLinkError).problems).toEqual([
    "CHANGELOG.md:1: NOPE.md: isn't a file in the repository",
    "CHANGELOG.md:2: ../outside.md: is outside the repository: write it relative to the repository's root",
    "CHANGELOG.md:3: /SPEC.md: is outside the repository: write it relative to the repository's root",
  ]);
});

test("each release's heading is anchored by its version, and its headings by both", () => {
  const html = anchorReleases(
    [
      '<h2 id="unreleased">Unreleased</h2>',
      "<ul><li>New.</li></ul>",
      '<h2 id="020-2026-10-06">0.2.0 (2026-10-06)</h2>',
      '<h3 id="the-ascribe-command">The <code>ascribe</code> command</h3>',
      '<h2 id="010-2026-10-01">0.1.0 (2026-10-01)</h2>',
      '<h3 id="the-ascribe-command-1">The <code>ascribe</code> command</h3>',
    ].join("\n"),
  );
  expect(Array.from(html.matchAll(/ id="([^"]*)"/g), (match) => match[1])).toEqual([
    "unreleased",
    "v0.2.0",
    "v0.2.0-the-ascribe-command",
    "v0.1.0",
    "v0.1.0-the-ascribe-command",
  ]);
  expect(html).toContain(
    '<h2 id="unreleased">Unreleased</h2>\n<p class="unreleased-note">In <code>main</code>, not yet released.</p>',
  );
});

test("a release being written, `0.3.0 (unreleased)`, is the unreleased section", () => {
  const html = anchorReleases("<h2>0.3.0 (unreleased)</h2><h3>Astro</h3>");
  expect(html).toContain('<h2 id="unreleased">');
  expect(html).toContain('<h3 id="unreleased-astro">');
  expect(html).toContain('class="unreleased-note"');
});

test("the description is the first paragraph's text", () => {
  expect(
    firstParagraph("<h1>Changelog</h1>\n<p>Every <code>ascribe</code> release &amp; more.</p>"),
  ).toBe("Every ascribe release & more.");
});

/** A changelog as Astro renders it, anchored: one release, with two areas. */
const rendered = anchorReleases(
  [
    "<h1>Changelog</h1>",
    "<p>Intro.</p>",
    "<h2>Unreleased</h2>",
    "<h3>The editor</h3>",
    "<ul>",
    "<li><strong>A bold sentence.</strong> The rest. More.</li>",
    "<li><strong>Behavior change: <code>x</code> is gone.</strong> Use y.</li>",
    "<li><strong>Just this.</strong></li>",
    "</ul>",
    "<h3>Fixes</h3>",
    "<ul>",
    "<li>",
    "<p><strong>Fixed:</strong> <code>a. b</code> works again. It didn't.</p>",
    "<details><summary>Old</summary><p>x</p></details>",
    "</li>",
    "<li><strong><code>@snippet</code></strong> puts code in a page. <strong>Behavior change:</strong> more.</li>",
    "</ul>",
    "<h2>0.1.0 (2026-10-01)</h2>",
    "<ul>",
    "<li>A plain entry. With more.</li>",
    "</ul>",
  ].join("\n"),
);

/** Each entry's summary, as the page shows it closed. */
function summaries(html: string): string[] {
  return Array.from(
    html.matchAll(
      /<li class="entry[^"]*">(?:<details><summary>|<p class="summary">)([\s\S]*?)<\/(?:summary|p)>/g,
    ),
    (match) => match[1] ?? "",
  );
}

test("an entry shows its bold sentence, or else its first sentence, and the rest when opened", () => {
  const { html } = collapseEntries(rendered);
  expect(summaries(html)).toEqual([
    "<strong>A bold sentence.</strong>",
    '<strong><span class="badge">Behavior change:</span> <code>x</code> is gone.</strong>',
    "<strong>Just this.</strong>",
    "<strong>Fixed:</strong> <code>a. b</code> works again.",
    '<span class="badge">Behavior change</span> <strong><code>@snippet</code></strong> puts code in a page.',
    "A plain entry.",
  ]);
  // The rest is under the summary, the entry's own blocks included.
  expect(html).toContain(
    '<summary><strong>A bold sentence.</strong></summary>\n<div class="entry-body">\n<p>The rest. More.</p>\n</div>',
  );
  expect(html).toContain("<p>It didn't.</p>\n<details><summary>Old</summary>");
  // An entry with nothing more isn't collapsed.
  expect(html).toContain(
    '<li class="entry"><p class="summary"><strong>Just this.</strong></p></li>',
  );
});

test("each release and area is a section, an area's heading counts its entries, and behavior changes are marked", () => {
  const { html } = collapseEntries(rendered);
  expect(html).toContain('<section class="release" data-behavior="2">\n<h2 id="unreleased">');
  expect(html).toContain(
    '<section class="area" data-behavior="1">\n<h3 id="unreleased-the-editor">The editor <span class="count">3</span></h3>',
  );
  expect(html.match(/<li class="entry behavior">/g)).toHaveLength(2);
  // A release with none says so, for "Behavior changes only".
  expect(html).toContain(
    '<section class="release" data-behavior="0">\n<h2 id="v0.1.0">0.1.0 (2026-10-01)</h2>\n<p class="no-behavior">',
  );
  // What comes before the first release stays as it was.
  expect(html.startsWith("<h1>Changelog</h1>\n<p>Intro.</p>\n<section")).toBe(true);
});

test("the contents list each release and its areas, with their counts", () => {
  const { contents } = collapseEntries(rendered);
  expect(contents).toEqual([
    {
      id: "unreleased",
      html: "Unreleased",
      count: 5,
      areas: [
        { id: "unreleased-the-editor", html: "The editor", count: 3 },
        { id: "unreleased-fixes", html: "Fixes", count: 2 },
      ],
    },
    { id: "v0.1.0", html: "0.1.0 (2026-10-01)", count: 1, areas: [] },
  ]);
});
