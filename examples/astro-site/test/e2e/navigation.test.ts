// The sidebar from the project's navigation: the layout calls `sidebar()`
// with the collection alone, so its groups are the default order with no
// `ascribe.nav`, and the file's order with one, with no list of the site's.
// Its previous and next, from `neighbors()`, follow the same order, and a
// page's `nav-label` is its name in both. A nested section's pages are in
// their top section's group, a level deeper.
import { readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { buildSite, copySite, siteDir } from "./harness.js";

afterAll(async () => {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/** A built page's previous and next: each link's `rel`, address, and label as HTML. */
async function pagerOf(root: string, page: string): Promise<string[]> {
  const html = await readFile(path.join(root, "dist", page), "utf8");
  const nav = /<nav aria-label="Previous and next">([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? "";
  return [...nav.matchAll(/<a rel="([^"]*)" href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/g)].map(
    (link) => `${link[1]} ${link[2]} ${link[3]?.replace(/\s+/g, " ").trim()}`,
  );
}

/**
 * The built home page's sidebar: each group's label, then its pages' depths,
 * addresses, and labels as HTML.
 */
async function sidebarOf(root: string): Promise<string[]> {
  const html = await readFile(path.join(root, "dist", "index.html"), "utf8");
  const nav = /<nav aria-label="Docs">([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? "";
  const lines: string[] = [];
  for (const section of nav.split("<section").slice(1)) {
    lines.push(/<h2[^>]*>([\s\S]*?)<\/h2>/.exec(section)?.[1] ?? "");
    for (const item of section.matchAll(
      /<li data-depth="(\d+)">\s*<a href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/g,
    )) {
      lines.push(`  ${item[1]} ${item[2]} ${item[3]?.trim()}`);
    }
  }
  return lines;
}

const START = ["Start", "  0 /docs Loom documentation"];
const NESTED = "  1 /docs/guides/teams/share-a-loom Share a loom";
const GUIDES = ["Guides", "  0 /docs/guides/my-setup Set up Loom", NESTED];
const REFERENCE = ["Reference", "  0 /docs/reference/options <code>loom.yaml</code> options"];

describe("the sidebar", () => {
  it("is the default order with no ascribe.nav, and the file's with one", async () => {
    const root = await copySite("navigation");
    await buildSite(root);
    expect(await sidebarOf(root)).toEqual([...START, ...GUIDES, ...REFERENCE]);

    await writeFile(
      path.join(root, "ascribe.nav"),
      'version = 1\n\n[[sections]]\nfolder = ""\n\n[[sections]]\nfolder = "Reference/"\n\n[[sections]]\nfolder = "Guides/"\n',
    );
    await buildSite(root);
    expect(await sidebarOf(root)).toEqual([...START, ...REFERENCE, ...GUIDES]);
    expect(await pagerOf(root, "reference/options.html")).toEqual([
      "prev /docs Previous: Loom documentation",
      "next /docs/guides/my-setup Next: Set up Loom",
    ]);
  });

  it("names a page by its nav-label in the sidebar and in its neighbors' links", async () => {
    const root = await copySite("nav-label");
    const page = path.join(root, "content", "Guides", "My Setup.md");
    const text = await readFile(page, "utf8");
    await writeFile(page, text.replace("title: Set up Loom\n", "title: Set up Loom\nnav-label: Setup\n"));
    await buildSite(root);
    expect(await sidebarOf(root)).toEqual([
      ...START,
      "Guides",
      "  0 /docs/guides/my-setup Setup",
      NESTED,
      ...REFERENCE,
    ]);
    expect(await pagerOf(root, "index.html")).toEqual(["next /docs/guides/my-setup Next: Setup"]);
    const html = await readFile(path.join(root, "dist", "guides", "my-setup.html"), "utf8");
    expect(html).toContain("<h1>Set up Loom</h1>");
  });
});
