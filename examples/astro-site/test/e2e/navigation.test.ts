// The sidebar from the project's navigation: the layout calls `sidebar()`
// with the collection alone, so its groups are the default order with no
// `ascribe.nav`, and the file's order with one, with no list of the site's.
import { readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import { buildSite, copySite, siteDir } from "./harness.js";

afterAll(async () => {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/** The built home page's sidebar: each group's label, then its pages' addresses and labels as HTML. */
async function sidebarOf(root: string): Promise<string[]> {
  const html = await readFile(path.join(root, "dist", "index.html"), "utf8");
  const nav = /<nav aria-label="Docs">([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? "";
  const lines: string[] = [];
  for (const section of nav.split("<section").slice(1)) {
    lines.push(/<h2[^>]*>([\s\S]*?)<\/h2>/.exec(section)?.[1] ?? "");
    for (const link of section.matchAll(/<a href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/g)) {
      lines.push(`  ${link[1]} ${link[2]?.trim()}`);
    }
  }
  return lines;
}

const START = ["Start", "  /docs Loom documentation"];
const GUIDES = ["Guides", "  /docs/guides/my-setup Set up Loom"];
const REFERENCE = ["Reference", "  /docs/reference/options <code>loom.yaml</code> options"];

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
  });
});
