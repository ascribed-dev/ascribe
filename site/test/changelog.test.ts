// The changelog page's rules for links and anchors, on rendered Markdown made
// here: they need no build.
import { expect, test } from "vitest";
import {
  ChangelogLinkError,
  anchorReleases,
  firstParagraph,
  pointLinks,
  type Targets,
} from "../src/changelog.ts";

const targets: Targets = {
  pages: new Map([
    ["docs/content/index.md", "/docs/"],
    ["docs/content/guides/agents.md", "/docs/guides/agents/"],
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
    '<a href="docs/content/guides/agents.md#hooks">Agents</a> <a href="docs/content/index.md">docs</a>',
    "",
    targets,
  );
  expect(hrefs(html)).toEqual(["/docs/guides/agents/#hooks", "/docs/"]);
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
